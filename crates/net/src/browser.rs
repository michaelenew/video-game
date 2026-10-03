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
//! What a public broker sees: the room name, and the two notes' contents,
//! which include each player's network addresses (that is what an offer
//! *is*). Anyone subscribed to the same topic sees them too. Nothing of the
//! match goes through it.

use crate::meet::{Board, Boards, Line, LineState, Reach, Room};
use crate::mqtt;
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    BinaryType, BroadcastChannel, MessageEvent, RtcConfiguration, RtcDataChannel,
    RtcDataChannelInit, RtcDataChannelState, RtcDataChannelType, RtcIceGatheringState,
    RtcIceServer, RtcPeerConnection, RtcPeerConnectionState, RtcSdpType, RtcSessionDescriptionInit,
    WebSocket,
};

/// Free public MQTT brokers that accept WebSocket connections from a page.
///
/// None of them promises anything, which is why there are three and every
/// note goes to all of them. Replacing them with one we run is a one-line
/// change here and nothing else.
pub const BROKERS: &[&str] = &[
    "wss://broker.emqx.io:8084/mqtt",
    "wss://broker.hivemq.com:8884/mqtt",
    "wss://test.mosquitto.org:8081/mqtt",
];

/// STUN servers: each answers one question, "what does my address look like
/// from outside my router?", which is what lets two home networks connect
/// directly. They carry nothing of the match.
pub const STUN: &[&str] = &[
    "stun:stun.l.google.com:19302",
    "stun:stun.cloudflare.com:3478",
];

/// How long to wait for the full list of ways to reach this machine before
/// posting the ones found so far. A STUN server that does not answer can hold
/// the list open for many seconds, and the local and outside addresses -- the
/// ones that matter -- are nearly always in within a second.
const GATHER_MS: u64 = 2_500;

/// A room name, made safe to be part of a topic: letters, digits, `-` and
/// `_`, lower case, at most 32. MQTT reads `+` and `#` in a topic as
/// wildcards, and two people typing `K3X9` and `k3x9` mean the same room.
pub fn room_name(code: &str) -> String {
    code.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(32)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Meet in `code` through the public brokers.
pub fn public(code: &str, me: u64, terms: &str) -> Room<Boards, RtcLine> {
    brokers(BROKERS, code, me, terms)
}

/// Meet in `code` through these brokers: `wss://` (or `ws://`) URLs of MQTT
/// brokers that take WebSocket connections. One we run, or a local one for a
/// test, is this with a different list.
pub fn brokers(urls: &[&str], code: &str, me: u64, terms: &str) -> Room<Boards, RtcLine> {
    let topic = format!("arena-rollback/room/{}", room_name(code));
    let boards = urls
        .iter()
        .map(|url| Box::new(MqttBoard::open(url, &topic, me)) as Box<dyn Board>)
        .collect();
    Room::new(Boards(boards), RtcLine::new, me, terms)
}

/// Meet in `code` with another tab of this browser.
pub fn tabs(code: &str, me: u64, terms: &str) -> Room<TabsBoard, RtcLine> {
    Room::new(TabsBoard::open(code), RtcLine::new, me, terms)
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

#[derive(Default)]
struct Inbox {
    reader: mqtt::Reader,
    open: bool,
    closed: bool,
}

struct Mqtt {
    socket: WebSocket,
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
        let _ = self.socket.send_with_u8_array(bytes);
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
        }
        loop {
            let next = self.inbox.borrow_mut().reader.packet();
            let Some(packet) = next else { break };
            match packet {
                mqtt::Incoming::ConnAck(0) => self.send(&mqtt::subscribe(1, &self.topic)),
                mqtt::Incoming::ConnAck(_) => self.reach = Reach::Down,
                mqtt::Incoming::SubAck => {
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
        self.socket.set_onopen(None);
        self.socket.set_onmessage(None);
        self.socket.set_onclose(None);
        self.socket.set_onerror(None);
        let _ = self.socket.close();
    }
}

/// A room as a topic on one public MQTT broker.
pub struct MqttBoard(Option<Held<Mqtt>>);

impl MqttBoard {
    /// Start connecting. A board that could not even create its socket is
    /// simply down; [`Boards`] carries on with the others.
    pub fn open(url: &str, topic: &str, me: u64) -> MqttBoard {
        let Ok(socket) = WebSocket::new_with_str(url, "mqtt") else {
            return MqttBoard(None);
        };
        socket.set_binary_type(BinaryType::Arraybuffer);
        let inbox = Rc::new(RefCell::new(Inbox::default()));

        let on_open = {
            let inbox = inbox.clone();
            Closure::<dyn FnMut(JsValue)>::new(move |_| inbox.borrow_mut().open = true)
        };
        let on_message = {
            let inbox = inbox.clone();
            Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
                if let Some(bytes) = event.dyn_ref::<MessageEvent>().and_then(bytes_of) {
                    inbox.borrow_mut().reader.feed(&bytes);
                }
            })
        };
        let on_close = {
            let inbox = inbox.clone();
            Closure::<dyn FnMut(JsValue)>::new(move |_| inbox.borrow_mut().closed = true)
        };
        socket.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        socket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        socket.set_onclose(Some(on_close.as_ref().unchecked_ref()));
        socket.set_onerror(Some(on_close.as_ref().unchecked_ref()));

        MqttBoard(Some(Held::new(Mqtt {
            socket,
            topic: topic.to_string(),
            client_id: format!("arena-{me:016x}"),
            inbox,
            reach: Reach::Trying,
            sent_connect: false,
            waiting: VecDeque::new(),
            notes: VecDeque::new(),
            last_ping: 0,
            _handlers: vec![on_open, on_message, on_close],
        })))
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
    pub fn open(code: &str) -> TabsBoard {
        let Ok(channel) = BroadcastChannel::new(&format!("arena-room-{}", room_name(code))) else {
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
}
