//! A desktop in a browser's room: the same board and the same line, without
//! a browser.
//!
//! - [`MqttBoard`]: the same public brokers, over a WebSocket with TLS. One
//!   thread per broker owns the connection, because connecting is blocking
//!   work -- a DNS lookup, a TCP handshake, a TLS handshake -- and a frame
//!   must not wait on any of it. The board itself only touches a queue.
//! - [`RtcLine`]: WebRTC, the same data channel a page opens, from
//!   [str0m](https://github.com/algesten/str0m). str0m does no I/O of its own:
//!   it is handed packets and the time, and hands back packets to send. So
//!   the room's once-a-frame polling is what drives it, and it needs no
//!   thread and no async runtime.
//!
//! Both speak exactly what the browser's halves speak -- the MQTT bytes come
//! from the same `mqtt.rs`, the notes are sealed by the same `seal.rs`, and
//! the channel is negotiated the same way (id 0, unordered, never resent) --
//! which is the whole of what makes a desktop and a page able to meet.

use crate::meet::{BROKERS, Board, Boards, Line, LineState, Reach, Room, STUN, Trace, seconds};
use crate::mqtt;
use crate::seal::{RoomKey, Sealed};
use std::collections::VecDeque;
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, ToSocketAddrs, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, mpsc};
use std::time::{Duration, Instant};
use str0m::change::{SdpAnswer, SdpOffer, SdpPendingOffer};
use str0m::channel::{ChannelConfig, ChannelId, Reliability};
use str0m::net::{Protocol, Receive};
use str0m::{Candidate, Event, IceConnectionState, Input, Output, Rtc};

/// Meet through the public brokers in the room this key names.
pub fn public(key: &RoomKey, me: u64, terms: &str) -> Room<Sealed<Boards>, RtcLine> {
    brokers(BROKERS, key, me, terms)
}

/// Meet through these brokers: `wss://` (or `ws://`) URLs.
pub fn brokers(
    urls: &[&str],
    key: &RoomKey,
    me: u64,
    terms: &str,
) -> Room<Sealed<Boards>, RtcLine> {
    let boards = urls
        .iter()
        .map(|url| Box::new(MqttBoard::open(url, key.topic(), me)) as Box<dyn Board>)
        .collect();
    Room::new(
        Sealed::new(Boards(boards), key.clone()),
        RtcLine::new,
        me,
        terms,
    )
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

// ---------------------------------------------------------------------------
// MQTT over a WebSocket, on a thread
// ---------------------------------------------------------------------------

/// How long one connection attempt may take before the broker counts as down.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(6);

/// How long the broker thread waits for bytes before looking at its outbox.
const READ_SLICE: Duration = Duration::from_millis(20);

struct Mailbox {
    url: String,
    started: Instant,
    /// What happened to the connection, for [`Board::report`].
    trace: Trace,
    /// When it went down, from `started`.
    down_at: Option<u64>,
    reach: Reach,
    notes: VecDeque<String>,
    /// Posted before the subscription was live, to send when it is.
    waiting: VecDeque<String>,
}

/// A room as a topic on one public MQTT broker.
pub struct MqttBoard {
    mailbox: Arc<Mutex<Mailbox>>,
    outbox: mpsc::Sender<String>,
    stop: Arc<AtomicBool>,
}

impl MqttBoard {
    /// Start connecting, on a thread of its own.
    pub fn open(url: &str, topic: &str, me: u64) -> MqttBoard {
        let mut story = Trace::default();
        story.note(0, "starting");
        let mailbox = Arc::new(Mutex::new(Mailbox {
            url: url.to_string(),
            started: Instant::now(),
            trace: story,
            down_at: None,
            reach: Reach::Trying,
            notes: VecDeque::new(),
            waiting: VecDeque::new(),
        }));
        let (outbox, posts) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let (url, topic) = (url.to_string(), topic.to_string());
        let (inbox, stopped) = (mailbox.clone(), stop.clone());
        let spawned = std::thread::Builder::new()
            .name("mqtt board".into())
            .spawn(move || {
                // A panic in here would end the thread with the board still
                // "trying", and the room would wait on it forever. Caught, it
                // is one more way of being down, with its message in the trace.
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    run_broker(&url, &topic, me, &inbox, &posts, &stopped)
                }))
                .unwrap_or_else(|panic| {
                    let why = panic
                        .downcast_ref::<String>()
                        .cloned()
                        .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
                        .unwrap_or_else(|| "no message".into());
                    Err(format!("the connection thread crashed: {why}"))
                });
                if let Err(why) = outcome
                    && !stopped.load(Ordering::Relaxed)
                {
                    eprintln!("meeting point {url}: {why}");
                    trace(&inbox, format!("down: {why}"));
                }
                let mut mailbox = lock(&inbox);
                mailbox.reach = Reach::Down;
                mailbox.down_at = Some(mailbox.started.elapsed().as_millis() as u64);
            });
        if spawned.is_err() {
            let mut mailbox = lock(&mailbox);
            mailbox.reach = Reach::Down;
            mailbox.trace.note(0, "down: could not start its thread");
        }
        MqttBoard {
            mailbox,
            outbox,
            stop,
        }
    }
}

impl Drop for MqttBoard {
    fn drop(&mut self) {
        // The thread notices within one read slice, says goodbye to the broker
        // and ends.
        self.stop.store(true, Ordering::Relaxed);
    }
}

impl Board for MqttBoard {
    fn post(&mut self, note: &str) {
        let mut mailbox = lock(&self.mailbox);
        match mailbox.reach {
            Reach::Up => {
                let _ = self.outbox.send(note.to_string());
            }
            // As in the browser: a hello repeats every second, so only the
            // latest few are worth keeping for when the broker answers.
            Reach::Trying => {
                if mailbox.waiting.len() >= 4 {
                    mailbox.waiting.pop_front();
                }
                mailbox.waiting.push_back(note.to_string());
            }
            Reach::Down => {}
        }
    }

    fn take(&mut self) -> Option<String> {
        lock(&self.mailbox).notes.pop_front()
    }

    fn reach(&mut self, _now_ms: u64) -> Reach {
        lock(&self.mailbox).reach
    }

    fn report(&self, out: &mut Vec<String>) {
        let mailbox = lock(&self.mailbox);
        let age = mailbox.started.elapsed().as_millis() as u64;
        out.push(format!(
            "meeting point {}: {}",
            mailbox.url,
            match (mailbox.reach, mailbox.down_at) {
                (Reach::Up, _) => "up".to_string(),
                (Reach::Trying, _) => format!("still trying after {}", seconds(age)),
                (Reach::Down, Some(at)) => format!("down after {}", seconds(at)),
                (Reach::Down, None) => "down".to_string(),
            }
        ));
        mailbox.trace.lines(out);
    }
}

/// Add an event to a board's trace, timed from when the board was opened.
fn trace(mailbox: &Mutex<Mailbox>, what: impl Into<String>) {
    let mut mailbox = lock(mailbox);
    let at = mailbox.started.elapsed().as_millis() as u64;
    mailbox.trace.note(at, what);
}

/// Name rustls's cryptography for the process, once.
///
/// rustls connects nothing until it knows which to use. It works it out alone
/// only when exactly one is compiled in: with none (as this crate first
/// shipped) or with two (as any new dependency could cause) the handshake
/// panics, and every `wss://` broker fails before sending a byte. Naming it
/// here holds in both cases. aws-lc is the one str0m already builds.
fn install_crypto() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        // An error means somebody installed one first, which is as good.
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });
}

type Socket = tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<TcpStream>>;

/// Open a TCP connection to the broker within [`CONNECT_TIMEOUT`], then the
/// WebSocket on top of it (with TLS for `wss://`), asking for the `mqtt`
/// subprotocol as a browser does.
fn dial(url: &str, mailbox: &Mutex<Mailbox>) -> Result<Socket, String> {
    use tungstenite::client::IntoClientRequest;
    install_crypto();
    let mut request = url.into_client_request().map_err(|e| e.to_string())?;
    request.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        tungstenite::http::HeaderValue::from_static("mqtt"),
    );
    let uri = request.uri().clone();
    let host = uri.host().ok_or("no host in the URL")?;
    let port = uri
        .port_u16()
        .unwrap_or(if uri.scheme_str() == Some("wss") {
            443
        } else {
            80
        });
    trace(mailbox, format!("looking up {host}"));
    let addrs: Vec<SocketAddr> = (host, port)
        .to_socket_addrs()
        .map_err(|e| format!("could not look up {host}: {e}"))?
        .collect();
    trace(
        mailbox,
        format!(
            "{host} is {}",
            addrs
                .iter()
                .map(|a| a.ip().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    );
    let mut last = String::from("no address");
    for addr in addrs {
        trace(mailbox, format!("connecting to {addr}"));
        match TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT) {
            Ok(stream) => {
                trace(
                    mailbox,
                    format!(
                        "connected; {} and WebSocket handshake",
                        if uri.scheme_str() == Some("wss") {
                            "TLS"
                        } else {
                            "no TLS (ws://)"
                        }
                    ),
                );
                stream
                    .set_read_timeout(Some(CONNECT_TIMEOUT))
                    .map_err(|e| e.to_string())?;
                let (socket, response) =
                    tungstenite::client_tls(request, stream).map_err(|e| handshake_failed(&e))?;
                trace(
                    mailbox,
                    format!(
                        "WebSocket open (HTTP {}, subprotocol {})",
                        response.status(),
                        response
                            .headers()
                            .get("Sec-WebSocket-Protocol")
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or("none")
                    ),
                );
                return Ok(socket);
            }
            Err(e) => {
                trace(mailbox, format!("{addr}: {e}"));
                last = format!("could not connect to {addr}: {e}");
            }
        }
    }
    Err(last)
}

/// Why a TLS and WebSocket handshake failed, with the HTTP answer if the
/// server got as far as giving one -- a proxy's 403 reads very differently
/// from a broker's 400.
fn handshake_failed(
    e: &tungstenite::HandshakeError<
        tungstenite::ClientHandshake<tungstenite::stream::MaybeTlsStream<TcpStream>>,
    >,
) -> String {
    match e {
        tungstenite::HandshakeError::Failure(tungstenite::Error::Http(response)) => format!(
            "the WebSocket handshake was refused: HTTP {}{}",
            response.status(),
            response
                .body()
                .as_ref()
                .map(|b| format!(" ({})", String::from_utf8_lossy(b).trim()))
                .unwrap_or_default()
        ),
        tungstenite::HandshakeError::Failure(e) => format!("the handshake failed: {e}"),
        tungstenite::HandshakeError::Interrupted(_) => "the handshake was interrupted".into(),
    }
}

fn tcp_of(socket: &Socket) -> &TcpStream {
    match socket.get_ref() {
        tungstenite::stream::MaybeTlsStream::Plain(s) => s,
        tungstenite::stream::MaybeTlsStream::Rustls(s) => s.get_ref(),
        _ => unreachable!("only plain and rustls streams are enabled"),
    }
}

fn run_broker(
    url: &str,
    topic: &str,
    me: u64,
    mailbox: &Mutex<Mailbox>,
    posts: &mpsc::Receiver<String>,
    stop: &AtomicBool,
) -> Result<(), String> {
    use tungstenite::Message;
    let mut socket = dial(url, mailbox)?;
    tcp_of(&socket)
        .set_read_timeout(Some(READ_SLICE))
        .map_err(|e| e.to_string())?;
    let send = |socket: &mut Socket, bytes: Vec<u8>| {
        socket
            .send(Message::binary(bytes))
            .map_err(|e| e.to_string())
    };
    send(&mut socket, mqtt::connect(&format!("arena-{me:016x}")))?;
    trace(mailbox, "MQTT connect sent");

    let mut reader = mqtt::Reader::default();
    let mut last_ping = Instant::now();
    loop {
        if stop.load(Ordering::Relaxed) {
            let _ = send(&mut socket, mqtt::disconnect());
            let _ = socket.close(None);
            return Ok(());
        }
        match socket.read() {
            Ok(Message::Binary(bytes)) => reader.feed(&bytes),
            Ok(Message::Close(frame)) => {
                return Err(match frame {
                    Some(f) => {
                        format!("the broker closed the connection ({} {})", f.code, f.reason)
                    }
                    None => "the broker closed the connection".into(),
                });
            }
            Ok(_) => {}
            Err(tungstenite::Error::Io(e))
                if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(e) => return Err(e.to_string()),
        }
        while let Some(packet) = reader.packet() {
            match packet {
                mqtt::Incoming::ConnAck(0) => {
                    trace(mailbox, "MQTT accepted; subscribing");
                    send(&mut socket, mqtt::subscribe(1, topic))?;
                }
                mqtt::Incoming::ConnAck(code) => {
                    return Err(format!("the broker refused us: {}", mqtt::refusal(code)));
                }
                mqtt::Incoming::SubAck => {
                    trace(mailbox, "subscribed: up");
                    let waiting = {
                        let mut mailbox = lock(mailbox);
                        mailbox.reach = Reach::Up;
                        std::mem::take(&mut mailbox.waiting)
                    };
                    for note in waiting {
                        send(&mut socket, mqtt::publish(topic, note.as_bytes()))?;
                    }
                }
                mqtt::Incoming::Publish {
                    topic: from,
                    payload,
                } if from == topic => {
                    let mut mailbox = lock(mailbox);
                    if mailbox.notes.len() < 256 {
                        mailbox
                            .notes
                            .push_back(String::from_utf8_lossy(&payload).into_owned());
                    }
                }
                _ => {}
            }
        }
        while let Ok(note) = posts.try_recv() {
            send(&mut socket, mqtt::publish(topic, note.as_bytes()))?;
        }
        if last_ping.elapsed() >= Duration::from_millis(mqtt::PING_EVERY_MS) {
            last_ping = Instant::now();
            send(&mut socket, mqtt::ping())?;
        }
        // Anything tungstenite queued itself -- the answer to a WebSocket
        // ping, say -- goes out now rather than with the next note.
        match socket.flush() {
            Ok(()) => {}
            Err(tungstenite::Error::Io(e))
                if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(e) => return Err(e.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// STUN: what this machine looks like from outside
// ---------------------------------------------------------------------------

/// RFC 5389's magic cookie: in every STUN header, and the mask on the address
/// in the answer.
const COOKIE: u32 = 0x2112_a442;

/// A Binding Request: the whole question, twenty bytes.
fn stun_request(txid: &[u8; 12]) -> [u8; 20] {
    let mut out = [0u8; 20];
    out[0..2].copy_from_slice(&0x0001u16.to_be_bytes());
    out[4..8].copy_from_slice(&COOKIE.to_be_bytes());
    out[8..20].copy_from_slice(txid);
    out
}

/// The address in a Binding Success Response to our question, if this is one.
fn stun_answer(packet: &[u8], txid: &[u8; 12]) -> Option<SocketAddr> {
    if packet.len() < 20
        || packet[0..2] != 0x0101u16.to_be_bytes()
        || packet[4..8] != COOKIE.to_be_bytes()
        || packet[8..20] != txid[..]
    {
        return None;
    }
    let mut at = 20;
    while at + 4 <= packet.len() {
        let kind = u16::from_be_bytes([packet[at], packet[at + 1]]);
        let len = u16::from_be_bytes([packet[at + 2], packet[at + 3]]) as usize;
        let value = packet.get(at + 4..at + 4 + len)?;
        // XOR-MAPPED-ADDRESS, IPv4: the port and address masked with the
        // cookie, so that a router rewriting addresses in packets it does not
        // understand leaves this one alone.
        if kind == 0x0020 && len >= 8 && value[1] == 0x01 {
            let port = u16::from_be_bytes([value[2], value[3]]) ^ (COOKIE >> 16) as u16;
            let ip = u32::from_be_bytes([value[4], value[5], value[6], value[7]]) ^ COOKIE;
            return Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::from(ip)), port));
        }
        at += 4 + len.div_ceil(4) * 4;
    }
    None
}

/// The address other machines on this network would reach us at: the
/// interface the system would route to the internet through. Asked of a UDP
/// socket that is "connected" without a packet being sent.
fn local_ip() -> IpAddr {
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| s.connect("8.8.8.8:80").map(|()| s))
        .and_then(|s| s.local_addr())
        .map(|a| a.ip())
        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
}

// ---------------------------------------------------------------------------
// WebRTC
// ---------------------------------------------------------------------------

/// How long to wait for STUN before describing this machine without its
/// outside address. The same budget a page gives its own gathering.
const GATHER: Duration = Duration::from_millis(2_500);

/// The data channel, configured exactly as the page's: negotiated by both
/// sides with id 0, unordered, never resent.
fn channel_config() -> ChannelConfig {
    ChannelConfig {
        label: "ggrs".into(),
        ordered: false,
        reliability: Reliability::MaxRetransmits { retransmits: 0 },
        negotiated: Some(0),
        protocol: String::new(),
    }
}

enum Step {
    Idle,
    /// Asked to offer; the offer is written once gathering is done.
    Offer,
    /// Asked to answer this offer; likewise.
    Answer(String),
    /// Our offer is out; waiting for the answer.
    Offered(SdpPendingOffer),
    Done,
}

struct Rtc0 {
    rtc: Rtc,
    socket: UdpSocket,
    host: SocketAddr,
    started: Instant,
    /// The STUN servers' addresses, once a thread has looked them up.
    stun_lookup: mpsc::Receiver<Vec<SocketAddr>>,
    stun_txid: [u8; 12],
    stun_asked: bool,
    outside: Option<SocketAddr>,
    step: Step,
    description: Option<String>,
    channel: Option<ChannelId>,
    open: bool,
    failed: bool,
    /// The last ICE state str0m reported, for [`Line::report`].
    ice: Option<IceConnectionState>,
    inbox: VecDeque<Vec<u8>>,
    buf: Box<[u8; 2048]>,
}

/// A WebRTC data channel to a browser, or to another desktop.
pub struct RtcLine(Option<Mutex<Rtc0>>);

impl RtcLine {
    pub fn new() -> RtcLine {
        RtcLine(Rtc0::new().ok().map(Mutex::new))
    }
}

impl Default for RtcLine {
    fn default() -> Self {
        RtcLine::new()
    }
}

impl Rtc0 {
    fn new() -> std::io::Result<Rtc0> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;
        let host = SocketAddr::new(local_ip(), socket.local_addr()?.port());
        let now = Instant::now();
        let mut rtc = Rtc::builder().build(now);
        if let Ok(candidate) = Candidate::host(host, "udp") {
            rtc.add_local_candidate(candidate);
        }
        // DNS is blocking, so it is a thread's job; the line asks STUN once
        // the answer is in, and does not wait for it past `GATHER`.
        let (found, stun_lookup) = mpsc::channel();
        let _ = std::thread::Builder::new()
            .name("stun lookup".into())
            .spawn(move || {
                let addrs = STUN
                    .iter()
                    .filter_map(|s| s.strip_prefix("stun:"))
                    .filter_map(|s| s.to_socket_addrs().ok())
                    .flatten()
                    .filter(SocketAddr::is_ipv4)
                    .collect();
                let _ = found.send(addrs);
            });
        let mut stun_txid = [0u8; 12];
        let _ = getrandom::getrandom(&mut stun_txid);
        Ok(Rtc0 {
            rtc,
            socket,
            host,
            started: now,
            stun_lookup,
            stun_txid,
            stun_asked: false,
            outside: None,
            step: Step::Idle,
            description: None,
            channel: None,
            open: false,
            failed: false,
            ice: None,
            inbox: VecDeque::new(),
            buf: Box::new([0; 2048]),
        })
    }

    /// Everything that has arrived, and everything str0m wants sent.
    fn pump(&mut self) {
        if !self.stun_asked
            && let Ok(servers) = self.stun_lookup.try_recv()
        {
            self.stun_asked = true;
            for server in servers {
                let _ = self.socket.send_to(&stun_request(&self.stun_txid), server);
            }
        }
        loop {
            match self.socket.recv_from(&mut self.buf[..]) {
                Ok((n, source)) => {
                    let packet = &self.buf[..n];
                    if let Some(outside) = stun_answer(packet, &self.stun_txid) {
                        self.outside.get_or_insert(outside);
                        continue;
                    }
                    let Ok(contents) = packet.try_into() else {
                        continue;
                    };
                    let input = Input::Receive(
                        Instant::now(),
                        Receive {
                            proto: Protocol::Udp,
                            source,
                            destination: self.host,
                            contents,
                        },
                    );
                    if self.rtc.handle_input(input).is_err() {
                        self.failed = true;
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == ErrorKind::ConnectionReset => continue,
                Err(_) => break,
            }
        }
        if self
            .rtc
            .handle_input(Input::Timeout(Instant::now()))
            .is_err()
        {
            self.failed = true;
        }
        self.drain();
    }

    fn drain(&mut self) {
        loop {
            match self.rtc.poll_output() {
                Ok(Output::Timeout(_)) => break,
                Ok(Output::Transmit(t)) => {
                    let _ = self.socket.send_to(&t.contents, t.destination);
                }
                Ok(Output::Event(event)) => match event {
                    Event::ChannelOpen(id, _) => {
                        self.channel = Some(id);
                        self.open = true;
                    }
                    Event::ChannelData(data) => {
                        if self.inbox.len() < 1024 {
                            self.inbox.push_back(data.data);
                        }
                    }
                    Event::IceConnectionStateChange(state) => {
                        self.ice = Some(state);
                        if state == IceConnectionState::Disconnected && !self.open {
                            self.failed = true;
                        }
                    }
                    _ => {}
                },
                Err(_) => {
                    self.failed = true;
                    break;
                }
            }
        }
    }

    /// Write the offer or the answer, once the outside address is known or
    /// gathering has run out of time.
    fn describe(&mut self) {
        let gathered = self.outside.is_some() || self.started.elapsed() >= GATHER;
        if !gathered || !matches!(self.step, Step::Offer | Step::Answer(_)) {
            return;
        }
        if let Some(outside) = self.outside
            && let Ok(candidate) = Candidate::server_reflexive(outside, self.host, "udp")
        {
            self.rtc.add_local_candidate(candidate);
        }
        match std::mem::replace(&mut self.step, Step::Done) {
            Step::Offer => {
                let mut api = self.rtc.sdp_api();
                self.channel = Some(api.add_channel_with_config(channel_config()));
                match api.apply() {
                    Some((offer, pending)) => {
                        self.description = Some(offer.to_sdp_string());
                        self.step = Step::Offered(pending);
                    }
                    None => self.failed = true,
                }
            }
            Step::Answer(offer) => {
                let answer = SdpOffer::from_sdp_string(&offer)
                    .ok()
                    .and_then(|offer| self.rtc.sdp_api().accept_offer(offer).ok());
                match answer {
                    Some(answer) => {
                        self.channel =
                            Some(self.rtc.direct_api().create_data_channel(channel_config()));
                        self.description = Some(answer.to_sdp_string());
                    }
                    None => self.failed = true,
                }
            }
            _ => {}
        }
        self.drain();
    }
}

impl RtcLine {
    fn with<R>(&self, f: impl FnOnce(&mut Rtc0) -> R) -> Option<R> {
        self.0.as_ref().map(|m| f(&mut lock(m)))
    }
}

impl Line for RtcLine {
    fn offer(&mut self) {
        self.with(|r| r.step = Step::Offer);
    }

    fn answer(&mut self, offer: &str) {
        self.with(|r| r.step = Step::Answer(offer.to_string()));
    }

    fn accept(&mut self, answer: &str) {
        self.with(|r| {
            let Step::Offered(pending) = std::mem::replace(&mut r.step, Step::Done) else {
                return;
            };
            let accepted = SdpAnswer::from_sdp_string(answer)
                .ok()
                .and_then(|answer| r.rtc.sdp_api().accept_answer(pending, answer).ok());
            if accepted.is_none() {
                r.failed = true;
            }
            r.drain();
        });
    }

    fn description(&mut self, _now_ms: u64) -> Option<String> {
        self.with(|r| {
            r.pump();
            r.describe();
            r.description.clone()
        })
        .flatten()
    }

    fn state(&mut self) -> LineState {
        self.with(|r| {
            r.pump();
            if r.failed || (r.open && !r.rtc.is_alive()) {
                LineState::Failed
            } else if r.open {
                LineState::Open
            } else {
                LineState::Negotiating
            }
        })
        .unwrap_or(LineState::Failed)
    }

    fn send(&mut self, packet: &[u8]) {
        self.with(|r| {
            if let Some(id) = r.channel
                && r.open
                && let Some(mut channel) = r.rtc.channel(id)
            {
                let _ = channel.write(true, packet);
            }
            r.drain();
        });
    }

    fn recv(&mut self) -> Option<Vec<u8>> {
        self.with(|r| {
            if r.inbox.is_empty() {
                r.pump();
            }
            r.inbox.pop_front()
        })
        .flatten()
    }

    fn report(&self, out: &mut Vec<String>) {
        let Some(r) = &self.0 else {
            out.push("direct line: could not open a UDP socket".into());
            return;
        };
        let r = lock(r);
        out.push(format!(
            "direct line: here {}, outside {}, ICE {}, {}",
            r.host,
            r.outside.map_or_else(
                || {
                    if r.stun_asked {
                        "unknown (no STUN answer)".to_string()
                    } else {
                        "not asked yet".to_string()
                    }
                },
                |a| a.to_string()
            ),
            r.ice
                .map_or_else(|| "not started".to_string(), |s| format!("{s:?}")),
            if r.failed {
                "failed"
            } else if r.open {
                "open"
            } else if r.description.is_some() {
                "described"
            } else {
                "idle"
            }
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mailbox(url: &str) -> Mutex<Mailbox> {
        Mutex::new(Mailbox {
            url: url.into(),
            started: Instant::now(),
            trace: Trace::default(),
            down_at: None,
            reach: Reach::Trying,
            notes: VecDeque::new(),
            waiting: VecDeque::new(),
        })
    }

    #[test]
    fn a_tls_handshake_fails_with_a_reason_rather_than_a_panic() {
        // rustls with no cryptography named panics inside the handshake, which
        // is what kept every desktop off every `wss://` broker. Something that
        // accepts the connection and then says nothing TLS is enough to get
        // there: the handshake starts, and must end in an error.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                use std::io::Write;
                let _ = stream.write_all(b"HTTP/1.1 400 Not TLS\r\n\r\n");
            }
        });
        let url = format!("wss://127.0.0.1:{port}/mqtt");
        let mailbox = mailbox(&url);
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            dial(&url, &mailbox).map(|_| ())
        }));
        let Ok(Err(why)) = outcome else {
            panic!("dialling TLS must fail with a reason, not {outcome:?}");
        };
        assert!(why.contains("handshake"), "{why}");
        let mut lines = Vec::new();
        lock(&mailbox).trace.lines(&mut lines);
        assert!(
            lines
                .iter()
                .any(|l| l.contains("TLS and WebSocket handshake")),
            "{lines:#?}"
        );
    }

    #[test]
    fn a_broker_that_refuses_is_down_with_its_story() {
        // A port nothing listens on: bound, its number noted, and let go.
        let port = UdpSocket::bind("127.0.0.1:0")
            .and_then(|_| std::net::TcpListener::bind("127.0.0.1:0"))
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let url = format!("ws://127.0.0.1:{port}/mqtt");
        let mut board = MqttBoard::open(&url, "topic", 7);
        let started = Instant::now();
        while board.reach(0) == Reach::Trying {
            assert!(
                started.elapsed() < Duration::from_secs(10),
                "still trying a closed port"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(board.reach(0), Reach::Down);
        let mut report = Vec::new();
        board.report(&mut report);
        let text = report.join("\n");
        assert!(
            text.contains(&format!("meeting point {url}: down")),
            "{text}"
        );
        assert!(text.contains("connecting to 127.0.0.1"), "{text}");
        assert!(text.contains("down: could not connect"), "{text}");
    }

    #[test]
    fn a_stun_answer_reads_back_the_address_it_masks() {
        // RFC 5769 §2.2's sample response: 192.0.2.1 port 32853.
        let txid = [
            0xb7, 0xe7, 0xa7, 0x01, 0xbc, 0x34, 0xd6, 0x86, 0xfa, 0x87, 0xdf, 0xae,
        ];
        let mut packet = vec![0x01, 0x01, 0x00, 0x0c];
        packet.extend_from_slice(&COOKIE.to_be_bytes());
        packet.extend_from_slice(&txid);
        packet.extend_from_slice(&[
            0x00, 0x20, 0x00, 0x08, 0x00, 0x01, 0xa1, 0x47, 0xe1, 0x12, 0xa6, 0x43,
        ]);
        assert_eq!(
            stun_answer(&packet, &txid),
            Some("192.0.2.1:32853".parse().unwrap())
        );
        // Somebody else's question gets no answer.
        assert_eq!(stun_answer(&packet, &[0; 12]), None);
        assert_eq!(stun_request(&txid)[0..2], [0x00, 0x01]);
    }
}
