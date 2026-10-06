//! Meeting in a browser: boards and a line made of the browser's own parts.
//!
//! - [`MqttBoard`]: a room is a topic on a free public MQTT broker, reached
//!   over a WebSocket. [`public`] uses several brokers at once, so one being
//!   down does not close the room.
//! - [`TabsBoard`]: a room is a `BroadcastChannel`, which only reaches other
//!   tabs of the same browser on the same site. For trying the whole thing on
//!   one machine, and for the smoke test, which has no internet.
//! - [`RtcLine`]: a WebRTC data channel, set to drop rather than resend a lost
//!   packet -- which makes it behave like UDP, the thing GGRS was written for.
//!
//! **Everything here is single-threaded and says so.** GGRS and Bevy both
//! require `Send + Sync`, and a JavaScript object is neither. On wasm32
//! without shared memory there is one thread, so the JS objects live in a
//! thread-local table and the types above are handles into it ([`Held`]).
//! That is honest rather than a trick: a handle used from a second thread
//! would find the table empty and panic, rather than race.
//!
//! What a public broker sees: a topic that names nothing, and notes it cannot
//! read -- every board here is wrapped in [`Sealed`] (see `seal.rs`). Nothing
//! of the match goes through it.

use crate::meet::{BROKERS, Board, Boards, Line, LineState, Reach, Room, STUN, Trace, seconds};
use crate::mqtt;
use crate::seal::{RoomKey, Sealed};
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    BinaryType, BroadcastChannel, CloseEvent, MessageEvent, RtcConfiguration, RtcDataChannel,
    RtcDataChannelInit, RtcDataChannelState, RtcDataChannelType, RtcIceGatheringState,
    RtcIceServer, RtcPeerConnection, RtcPeerConnectionState, RtcSdpType, RtcSessionDescriptionInit,
    WebSocket,
};

/// How long to wait for the full list of ways to reach this machine before
/// posting the ones found so far. A STUN server that does not answer can hold
/// the list open for many seconds, and the local and outside addresses -- the
/// ones that matter -- are nearly always in within a second.
const GATHER_MS: u64 = 2_500;

/// Meet through the public brokers in the room this key names.
pub fn public(key: &RoomKey, me: u64, terms: &str) -> Room<Sealed<Boards>, RtcLine> {
    brokers(BROKERS, key, me, terms)
}

/// Meet through these brokers: `wss://` (or `ws://`) URLs of MQTT brokers
/// that take WebSocket connections. One we run, or a local one for a test, is
/// this with a different list.
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

/// Meet another tab of this browser.
pub fn tabs(key: &RoomKey, me: u64, terms: &str) -> Room<Sealed<TabsBoard>, RtcLine> {
    Room::new(
        Sealed::new(TabsBoard::open(key.topic()), key.clone()),
        RtcLine::new,
        me,
        terms,
    )
}

// ---------------------------------------------------------------------------
// Handles into a thread-local table
// ---------------------------------------------------------------------------

thread_local! {
    static HELD: RefCell<Vec<Option<Box<dyn Any>>>> = const { RefCell::new(Vec::new()) };
}

/// A value that lives in this thread's table, reached through a handle that
/// is `Send + Sync` because it holds nothing but an index.
struct Held<T: 'static> {
    slot: usize,
    _of: PhantomData<fn() -> T>,
}

impl<T: 'static> Held<T> {
    fn new(value: T) -> Self {
        let slot = HELD.with(|held| {
            let mut held = held.borrow_mut();
            let slot = held.iter().position(Option::is_none).unwrap_or_else(|| {
                held.push(None);
                held.len() - 1
            });
            held[slot] = Some(Box::new(value));
            slot
        });
        Held {
            slot,
            _of: PhantomData,
        }
    }

    /// Run `f` on the value. It is taken out of the table for the duration, so
    /// `f` may create or drop other handles.
    fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut value = HELD
            .with(|held| held.borrow_mut()[self.slot].take())
            .expect("a browser handle used off the thread that made it");
        let out = f(value.downcast_mut().expect("a handle of the right type"));
        HELD.with(|held| held.borrow_mut()[self.slot] = Some(value));
        out
    }
}

impl<T: 'static> Drop for Held<T> {
    fn drop(&mut self) {
        // Taken out first and dropped after the borrow ends: dropping a JS
        // wrapper may drop other handles.
        let value = HELD.with(|held| held.borrow_mut().get_mut(self.slot).and_then(Option::take));
        drop(value);
    }
}

fn bytes_of(event: &MessageEvent) -> Option<Vec<u8>> {
    let buffer = event.data().dyn_into::<js_sys::ArrayBuffer>().ok()?;
    Some(js_sys::Uint8Array::new(&buffer).to_vec())
}

// ---------------------------------------------------------------------------
// MQTT over a WebSocket
// ---------------------------------------------------------------------------

struct Inbox {
    reader: mqtt::Reader,
    open: bool,
    closed: bool,
    /// When the socket was made, on the page's clock, and what has happened
    /// to it since -- for [`Board::report`].
    started: f64,
    trace: Trace,
    down_at: Option<u64>,
}

impl Inbox {
    fn age(&self) -> u64 {
        (js_sys::Date::now() - self.started).max(0.0) as u64
    }

    fn note(&mut self, what: impl Into<String>) {
        let at = self.age();
        self.trace.note(at, what);
    }

    fn down(&mut self, why: impl Into<String>) {
        if self.down_at.is_none() {
            self.down_at = Some(self.age());
        }
        self.note(why);
    }
}

struct Mqtt {
    url: String,
    /// `None` when the browser would not even make one.
    socket: Option<WebSocket>,
    topic: String,
    client_id: String,
    inbox: Rc<RefCell<Inbox>>,
    reach: Reach,
    sent_connect: bool,
    /// Posted before the subscription was live, to send when it is.
    waiting: VecDeque<String>,
    notes: VecDeque<String>,
    last_ping: u64,
    _handlers: Vec<Closure<dyn FnMut(JsValue)>>,
}

impl Mqtt {
    fn send(&self, bytes: &[u8]) {
        if let Some(socket) = &self.socket {
            let _ = socket.send_with_u8_array(bytes);
        }
    }

    /// Everything the socket has delivered, turned into state and notes.
    fn pump(&mut self) {
        let (open, closed) = {
            let inbox = self.inbox.borrow();
            (inbox.open, inbox.closed)
        };
        if closed {
            self.reach = Reach::Down;
            return;
        }
        if open && !self.sent_connect {
            self.sent_connect = true;
            self.send(&mqtt::connect(&self.client_id));
            self.inbox.borrow_mut().note("MQTT connect sent");
        }
        loop {
            let next = self.inbox.borrow_mut().reader.packet();
            let Some(packet) = next else { break };
            match packet {
                mqtt::Incoming::ConnAck(0) => {
                    self.inbox.borrow_mut().note("MQTT accepted; subscribing");
                    self.send(&mqtt::subscribe(1, &self.topic));
                }
                mqtt::Incoming::ConnAck(code) => {
                    self.inbox.borrow_mut().down(format!(
                        "down: the broker refused us, {}",
                        mqtt::refusal(code)
                    ));
                    self.reach = Reach::Down;
                }
                mqtt::Incoming::SubAck => {
                    self.inbox.borrow_mut().note("subscribed: up");
                    self.reach = Reach::Up;
                    while let Some(note) = self.waiting.pop_front() {
                        self.send(&mqtt::publish(&self.topic, note.as_bytes()));
                    }
                }
                mqtt::Incoming::Publish { topic, payload }
                    if topic == self.topic && self.notes.len() < 256 =>
                {
                    self.notes
                        .push_back(String::from_utf8_lossy(&payload).into_owned());
                }
                _ => {}
            }
        }
    }
}

impl Drop for Mqtt {
    fn drop(&mut self) {
        if self.reach == Reach::Up {
            self.send(&mqtt::disconnect());
        }
        if let Some(socket) = &self.socket {
            socket.set_onopen(None);
            socket.set_onmessage(None);
            socket.set_onclose(None);
            socket.set_onerror(None);
            let _ = socket.close();
        }
    }
}

/// A room as a topic on one public MQTT broker.
pub struct MqttBoard(Option<Held<Mqtt>>);

impl MqttBoard {
    /// Start connecting. A board that could not even create its socket is
    /// simply down; [`Boards`] carries on with the others.
    pub fn open(url: &str, topic: &str, me: u64) -> MqttBoard {
        let mut inbox = Inbox {
            reader: mqtt::Reader::default(),
            open: false,
            closed: false,
            started: js_sys::Date::now(),
            trace: Trace::default(),
            down_at: None,
        };
        inbox.note("opening a WebSocket");
        let socket = match WebSocket::new_with_str(url, "mqtt") {
            Ok(socket) => socket,
            Err(e) => {
                // A malformed URL, or a port the browser itself forbids.
                inbox.closed = true;
                inbox.down(format!(
                    "down: the browser would not open it: {}",
                    e.as_string()
                        .or_else(|| js_sys::JSON::stringify(&e).ok().map(String::from))
                        .unwrap_or_default()
                ));
                return MqttBoard(Some(Held::new(Mqtt::dead(url, inbox))));
            }
        };
        socket.set_binary_type(BinaryType::Arraybuffer);
        let inbox = Rc::new(RefCell::new(inbox));

        let on_open = {
            let inbox = inbox.clone();
            Closure::<dyn FnMut(JsValue)>::new(move |_| {
                let mut inbox = inbox.borrow_mut();
                inbox.open = true;
                inbox.note("WebSocket open");
            })
        };
        let on_message = {
            let inbox = inbox.clone();
            Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
                if let Some(bytes) = event.dyn_ref::<MessageEvent>().and_then(bytes_of) {
                    inbox.borrow_mut().reader.feed(&bytes);
                }
            })
        };
        // The close event carries the one thing a page is told about a failed
        // connection: its code. 1006 with no reason is "it never answered, or
        // the connection was cut" -- a blocked port, a firewall, a broker down.
        let on_close = {
            let inbox = inbox.clone();
            Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
                let mut inbox = inbox.borrow_mut();
                inbox.closed = true;
                let why = match event.dyn_ref::<CloseEvent>() {
                    Some(close) => format!(
                        "down: closed, code {} ({}){}{}",
                        close.code(),
                        close_code(close.code(), inbox.open),
                        if close.reason().is_empty() {
                            String::new()
                        } else {
                            format!(", reason \"{}\"", close.reason())
                        },
                        if close.was_clean() { ", cleanly" } else { "" }
                    ),
                    None => "down: closed".into(),
                };
                inbox.down(why);
            })
        };
        // An error event says nothing at all, on purpose: a page is not told
        // why a connection failed, so it cannot probe a network. The browser's
        // own console line beside it is the only place the reason is written.
        let on_error = {
            let inbox = inbox.clone();
            Closure::<dyn FnMut(JsValue)>::new(move |_| {
                let mut inbox = inbox.borrow_mut();
                inbox.closed = true;
                inbox.note("error (the browser does not tell the page why; see the console)");
            })
        };
        socket.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        socket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        socket.set_onclose(Some(on_close.as_ref().unchecked_ref()));
        socket.set_onerror(Some(on_error.as_ref().unchecked_ref()));

        MqttBoard(Some(Held::new(Mqtt {
            url: url.to_string(),
            socket: Some(socket),
            topic: topic.to_string(),
            client_id: format!("arena-{me:016x}"),
            inbox,
            reach: Reach::Trying,
            sent_connect: false,
            waiting: VecDeque::new(),
            notes: VecDeque::new(),
            last_ping: 0,
            _handlers: vec![on_open, on_message, on_close, on_error],
        })))
    }
}

impl Mqtt {
    /// A board whose socket could not even be made: down from the start, kept
    /// so the report still has its story.
    fn dead(url: &str, inbox: Inbox) -> Mqtt {
        Mqtt {
            url: url.to_string(),
            socket: None,
            topic: String::new(),
            client_id: String::new(),
            inbox: Rc::new(RefCell::new(inbox)),
            reach: Reach::Down,
            sent_connect: true,
            waiting: VecDeque::new(),
            notes: VecDeque::new(),
            last_ping: 0,
            _handlers: Vec::new(),
        }
    }
}

/// What a WebSocket close code means (RFC 6455 §7.4.1), in a few words.
fn close_code(code: u16, was_open: bool) -> &'static str {
    match code {
        1000 => "closed normally",
        1001 => "the server is going away",
        1002 => "protocol error",
        1003 => "data it does not accept",
        1006 if was_open => "the connection was cut",
        1006 => "never connected: refused, blocked, unreachable, or a bad certificate",
        1008 => "refused by policy",
        1009 => "message too big",
        1011 => "the server hit an error",
        1015 => "the TLS handshake failed",
        _ => "unlisted",
    }
}

impl Board for MqttBoard {
    fn post(&mut self, note: &str) {
        let Some(held) = &self.0 else { return };
        held.with(|m| match m.reach {
            Reach::Up => m.send(&mqtt::publish(&m.topic, note.as_bytes())),
            // A hello is repeated every second, so only the latest few are
            // worth keeping for when the broker answers.
            Reach::Trying => {
                if m.waiting.len() >= 4 {
                    m.waiting.pop_front();
                }
                m.waiting.push_back(note.to_string());
            }
            Reach::Down => {}
        });
    }

    fn take(&mut self) -> Option<String> {
        self.0.as_ref()?.with(|m| {
            m.pump();
            m.notes.pop_front()
        })
    }

    fn reach(&mut self, now_ms: u64) -> Reach {
        let Some(held) = &self.0 else {
            return Reach::Down;
        };
        held.with(|m| {
            m.pump();
            if m.reach == Reach::Up && now_ms.saturating_sub(m.last_ping) >= mqtt::PING_EVERY_MS {
                m.last_ping = now_ms;
                m.send(&mqtt::ping());
            }
            m.reach
        })
    }

    fn report(&self, out: &mut Vec<String>) {
        let Some(held) = &self.0 else { return };
        held.with(|m| {
            let inbox = m.inbox.borrow();
            out.push(format!(
                "meeting point {}: {}",
                m.url,
                match (m.reach, inbox.down_at) {
                    (Reach::Up, _) => "up".to_string(),
                    (Reach::Trying, _) => format!("still trying after {}", seconds(inbox.age())),
                    (Reach::Down, Some(at)) => format!("down after {}", seconds(at)),
                    (Reach::Down, None) => "down".to_string(),
                }
            ));
            inbox.trace.lines(out);
        });
    }
}

// ---------------------------------------------------------------------------
// Tabs of one browser
// ---------------------------------------------------------------------------

struct Tabs {
    channel: BroadcastChannel,
    notes: Rc<RefCell<VecDeque<String>>>,
    _handler: Closure<dyn FnMut(JsValue)>,
}

impl Drop for Tabs {
    fn drop(&mut self) {
        self.channel.set_onmessage(None);
        self.channel.close();
    }
}

/// A room as a `BroadcastChannel`: other tabs of this browser, on this site.
pub struct TabsBoard(Option<Held<Tabs>>);

impl TabsBoard {
    /// `name` is the room's topic, so two rooms never overhear each other.
    pub fn open(name: &str) -> TabsBoard {
        let Ok(channel) = BroadcastChannel::new(name) else {
            return TabsBoard(None);
        };
        let notes = Rc::new(RefCell::new(VecDeque::new()));
        let handler = {
            let notes = notes.clone();
            Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
                let text = event
                    .dyn_ref::<MessageEvent>()
                    .and_then(|e| e.data().as_string());
                if let Some(text) = text {
                    notes.borrow_mut().push_back(text);
                }
            })
        };
        channel.set_onmessage(Some(handler.as_ref().unchecked_ref()));
        TabsBoard(Some(Held::new(Tabs {
            channel,
            notes,
            _handler: handler,
        })))
    }
}

impl Board for TabsBoard {
    fn post(&mut self, note: &str) {
        if let Some(held) = &self.0 {
            held.with(|t| {
                let _ = t.channel.post_message(&JsValue::from_str(note));
            });
        }
    }

    fn take(&mut self) -> Option<String> {
        self.0.as_ref()?.with(|t| t.notes.borrow_mut().pop_front())
    }

    fn reach(&mut self, _now_ms: u64) -> Reach {
        if self.0.is_some() {
            Reach::Up
        } else {
            Reach::Down
        }
    }
}

// ---------------------------------------------------------------------------
// WebRTC
// ---------------------------------------------------------------------------

struct Rtc {
    pc: RtcPeerConnection,
    channel: RtcDataChannel,
    inbox: Rc<RefCell<VecDeque<Vec<u8>>>>,
    /// The local description is set; candidates are being gathered into it.
    described: Rc<Cell<bool>>,
    /// When `described` was first seen, on the room's clock.
    described_at: Option<u64>,
    failed: Rc<Cell<bool>>,
    _handler: Closure<dyn FnMut(JsValue)>,
}

impl Drop for Rtc {
    fn drop(&mut self) {
        self.channel.set_onmessage(None);
        self.channel.close();
        self.pc.close();
    }
}

fn description(kind: RtcSdpType, sdp: &str) -> RtcSessionDescriptionInit {
    let init = RtcSessionDescriptionInit::new(kind);
    init.set_sdp(sdp);
    init
}

fn sdp_of(value: &JsValue) -> String {
    js_sys::Reflect::get(value, &JsValue::from_str("sdp"))
        .ok()
        .and_then(|s| s.as_string())
        .unwrap_or_default()
}

/// A WebRTC data channel to one other browser.
pub struct RtcLine(Option<Held<Rtc>>);

impl RtcLine {
    pub fn new() -> RtcLine {
        let config = RtcConfiguration::new();
        let servers = js_sys::Array::new();
        for url in STUN {
            let server = RtcIceServer::new();
            server.set_urls(&JsValue::from_str(url));
            servers.push(&server);
        }
        config.set_ice_servers(&servers);
        let Ok(pc) = RtcPeerConnection::new_with_configuration(&config) else {
            return RtcLine(None);
        };

        // Negotiated with a fixed id: both sides make the same channel
        // themselves, so neither has to wait to be handed one. Unordered and
        // never resent, so a lost packet stays lost -- the UDP that GGRS
        // expects, rather than a stream that stalls behind a lost frame.
        let init = RtcDataChannelInit::new();
        init.set_negotiated(true);
        init.set_id(0);
        init.set_ordered(false);
        init.set_max_retransmits(0);
        let channel = pc.create_data_channel_with_data_channel_dict("ggrs", &init);
        channel.set_binary_type(RtcDataChannelType::Arraybuffer);

        let inbox = Rc::new(RefCell::new(VecDeque::new()));
        let handler = {
            let inbox = inbox.clone();
            Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
                if let Some(bytes) = event.dyn_ref::<MessageEvent>().and_then(bytes_of) {
                    let mut inbox = inbox.borrow_mut();
                    if inbox.len() < 1024 {
                        inbox.push_back(bytes);
                    }
                }
            })
        };
        channel.set_onmessage(Some(handler.as_ref().unchecked_ref()));

        RtcLine(Some(Held::new(Rtc {
            pc,
            channel,
            inbox,
            described: Rc::new(Cell::new(false)),
            described_at: None,
            failed: Rc::new(Cell::new(false)),
            _handler: handler,
        })))
    }

    /// Run one negotiation step: the browser's API answers with promises, and
    /// the room polls, so the step writes its outcome where `state` and
    /// `description` will find it.
    fn step<F>(&mut self, make: impl FnOnce(RtcPeerConnection) -> F)
    where
        F: std::future::Future<Output = Result<(), JsValue>> + 'static,
    {
        let Some(held) = &self.0 else { return };
        let (pc, described, failed) =
            held.with(|r| (r.pc.clone(), r.described.clone(), r.failed.clone()));
        let future = make(pc);
        wasm_bindgen_futures::spawn_local(async move {
            match future.await {
                Ok(()) => described.set(true),
                Err(e) => {
                    web_sys::console::warn_2(&"webrtc negotiation failed:".into(), &e);
                    failed.set(true);
                }
            }
        });
    }
}

impl Default for RtcLine {
    fn default() -> Self {
        RtcLine::new()
    }
}

impl Line for RtcLine {
    fn offer(&mut self) {
        self.step(|pc| async move {
            let offer = JsFuture::from(pc.create_offer()).await?;
            let local = description(RtcSdpType::Offer, &sdp_of(&offer));
            JsFuture::from(pc.set_local_description(&local)).await?;
            Ok(())
        });
    }

    fn answer(&mut self, offer: &str) {
        let remote = description(RtcSdpType::Offer, offer);
        self.step(|pc| async move {
            JsFuture::from(pc.set_remote_description(&remote)).await?;
            let answer = JsFuture::from(pc.create_answer()).await?;
            let local = description(RtcSdpType::Answer, &sdp_of(&answer));
            JsFuture::from(pc.set_local_description(&local)).await?;
            Ok(())
        });
    }

    fn accept(&mut self, answer: &str) {
        let Some(held) = &self.0 else { return };
        let (pc, failed) = held.with(|r| (r.pc.clone(), r.failed.clone()));
        let remote = description(RtcSdpType::Answer, answer);
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = JsFuture::from(pc.set_remote_description(&remote)).await {
                web_sys::console::warn_2(&"webrtc answer refused:".into(), &e);
                failed.set(true);
            }
        });
    }

    fn description(&mut self, now_ms: u64) -> Option<String> {
        self.0.as_ref()?.with(|r| {
            if !r.described.get() {
                return None;
            }
            let since = *r.described_at.get_or_insert(now_ms);
            let gathered = r.pc.ice_gathering_state() == RtcIceGatheringState::Complete;
            if !gathered && now_ms.saturating_sub(since) < GATHER_MS {
                return None;
            }
            r.pc.local_description().map(|d| d.sdp())
        })
    }

    fn state(&mut self) -> LineState {
        let Some(held) = &self.0 else {
            return LineState::Failed;
        };
        held.with(|r| {
            if r.failed.get() || r.pc.connection_state() == RtcPeerConnectionState::Failed {
                LineState::Failed
            } else if r.channel.ready_state() == RtcDataChannelState::Open {
                LineState::Open
            } else {
                LineState::Negotiating
            }
        })
    }

    fn send(&mut self, packet: &[u8]) {
        if let Some(held) = &self.0 {
            held.with(|r| {
                if r.channel.ready_state() == RtcDataChannelState::Open {
                    let _ = r.channel.send_with_u8_array(packet);
                }
            });
        }
    }

    fn recv(&mut self) -> Option<Vec<u8>> {
        self.0.as_ref()?.with(|r| r.inbox.borrow_mut().pop_front())
    }

    fn report(&self, out: &mut Vec<String>) {
        let Some(held) = &self.0 else {
            out.push("direct line: the browser would not make a WebRTC connection".into());
            return;
        };
        held.with(|r| {
            out.push(format!(
                "direct line: connection {:?}, ICE {:?}, gathering {:?}, channel {:?}{}",
                r.pc.connection_state(),
                r.pc.ice_connection_state(),
                r.pc.ice_gathering_state(),
                r.channel.ready_state(),
                if r.failed.get() {
                    ", negotiation failed (see the console)"
                } else {
                    ""
                }
            ));
        });
    }
}
