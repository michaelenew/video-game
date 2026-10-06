//! Meeting the other player.
//!
//! How two people who share nothing but a room name end up with a line between
//! them that GGRS can run a match over. The design is three seams, each a
//! trait, so any one of them can be replaced without touching the other two:
//!
//! - A [`Board`] is **where strangers leave notes**: a public message broker,
//!   two tabs of one browser, a UDP port. It carries a handful of short texts
//!   and nothing of the match. (Networking calls this *signalling*.)
//! - A [`Line`] is **the direct connection itself**, opened by trading two
//!   descriptions of how to reach each end -- an offer and an answer. A WebRTC
//!   data channel in a browser; on a desktop, the UDP port the board was
//!   already using. GGRS runs over it through [`LineSocket`].
//! - A [`Rendezvous`] is **the whole meeting**, polled once a frame until it
//!   hands over a [`Seat`]: a socket, the far end, and which player you are.
//!
//! [`Room`] is the one rendezvous there is, and it is generic over the other
//! two. Its protocol is four lines of text, which is why the board can be
//! anything that carries text:
//!
//! ```text
//! arena1 hello  <from> <waited ms> <terms>          every second, until connected
//! arena1 offer  <from> <to> <attempt> <offerer p1>  + the offer, until answered
//! arena1 answer <from> <to> <attempt>               + the answer, once per offer
//! ```
//!
//! **Terms** are what two clients must agree on before a frame is played: the
//! build, and the checksum of the world the match starts from -- which covers
//! the arena, both classes, and every tuned number, because tuning is folded
//! into `World::checksum`. A hello with different terms is never answered, and
//! the room says why instead of letting the match desync on its first frame.
//!
//! **Who is player one** is decided once, by one side, so it cannot come out
//! differently on the two machines: the peer with the lower id makes the offer,
//! and the offer says which of the two is player one -- whoever had been
//! waiting in the room longer, which is nearly always whoever sent the link.
//! The old rule ordered the two peers' addresses, and each machine only knew
//! its own as `127.0.0.1`, so across two machines both claimed the same slot.

use crate::Peer;
use ggrs::{Message, NonBlockingSocket};

/// Free public MQTT brokers that accept WebSocket connections, from a page or
/// from a desktop.
///
/// None of them promises anything, which is why there are three and every
/// note goes to all of them. Replacing them with one we run is a change here
/// and nothing else; `?broker=` tries one without a rebuild.
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

/// The note format's version. A note from another version is recognised as
/// "a different build" rather than ignored as noise.
pub const PROTOCOL: u32 = 1;

/// How often a hello is repeated while nobody has answered it. Every note can
/// be lost -- a public broker delivers at most once -- so everything is
/// repeated until the thing it asks for has happened.
const HELLO_EVERY_MS: u64 = 1_000;

/// How often an unanswered offer is repeated.
const OFFER_EVERY_MS: u64 = 2_000;

/// A partner who has sent no hello for this long, before the line opened, has
/// gone: closed the tab, or reloaded and come back as somebody new. The room
/// forgets them and starts again.
///
/// Generous, because a page only says hello from its frame loop, and a frame
/// can take seconds: the first one that draws a new fighter compiles its
/// shaders, and a browser rendering in software (the smoke test's) manages
/// two frames a second.
const PARTNER_SILENT_MS: u64 = 15_000;

/// From finding a partner to an open line. Past this the line is not going to
/// open: one of the two networks will not take a direct connection.
const NEGOTIATE_MS: u64 = 30_000;

// ---------------------------------------------------------------------------
// The board
// ---------------------------------------------------------------------------

/// Whether a board can currently carry notes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reach {
    /// Still connecting.
    Trying,
    /// Notes go out and come in.
    Up,
    /// It will not: refused, closed, or unreachable.
    Down,
}

/// Somewhere strangers leave each other short notes under a room name.
///
/// A board may lose a note, deliver one twice, and echo a note back to the
/// peer that posted it. [`Room`] is written for all three.
pub trait Board: Send + Sync {
    /// Leave a note for everyone in the room.
    fn post(&mut self, note: &str);
    /// The next note somebody left, if one has arrived.
    fn take(&mut self) -> Option<String>;
    /// Can it carry notes right now? Called once a frame, so it is also where
    /// a board does its housekeeping (a keep-alive, say).
    fn reach(&mut self, now_ms: u64) -> Reach;
    /// What it can say about how it is going, for somebody working out why a
    /// room never formed: a few lines of plain text, appended to `out`. Read
    /// only in dev mode (F10), so it may be as wordy as it is useful.
    fn report(&self, _out: &mut Vec<String>) {}
}

/// Several boards as one: every note goes on all of them, and a note on any of
/// them is read.
///
/// Public brokers come and go, and a room that used only one would fail with
/// it. The protocol already tolerates a note arriving twice, so posting
/// everywhere costs nothing but bytes.
pub struct Boards(pub Vec<Box<dyn Board>>);

impl Board for Boards {
    fn post(&mut self, note: &str) {
        for board in &mut self.0 {
            board.post(note);
        }
    }

    fn take(&mut self) -> Option<String> {
        self.0.iter_mut().find_map(|board| board.take())
    }

    fn reach(&mut self, now_ms: u64) -> Reach {
        let mut all_down = !self.0.is_empty();
        let mut any_up = false;
        for board in &mut self.0 {
            match board.reach(now_ms) {
                Reach::Up => any_up = true,
                Reach::Trying => all_down = false,
                Reach::Down => {}
            }
        }
        if any_up {
            Reach::Up
        } else if all_down {
            Reach::Down
        } else {
            Reach::Trying
        }
    }

    fn report(&self, out: &mut Vec<String>) {
        for board in &self.0 {
            board.report(out);
        }
    }
}

/// What happened to one connection, in order and with when: "connecting",
/// "open after 210 ms", "closed: code 1006". The raw material of a board's
/// [`Board::report`].
///
/// Kept short, the first events and the latest ones, because what went wrong
/// is nearly always at the start, and a connection that stays up repeats
/// nothing worth reading.
#[derive(Clone, Debug, Default)]
pub struct Trace {
    events: Vec<(u64, String)>,
    /// Events dropped from the middle to keep it short.
    skipped: usize,
}

impl Trace {
    const KEEP: usize = 16;

    /// `at_ms` is from when the connection was started.
    pub fn note(&mut self, at_ms: u64, what: impl Into<String>) {
        if self.events.len() >= Self::KEEP {
            self.events.remove(Self::KEEP / 2);
            self.skipped += 1;
        }
        self.events.push((at_ms, what.into()));
    }

    /// One indented line per event.
    pub fn lines(&self, out: &mut Vec<String>) {
        for (i, (at, what)) in self.events.iter().enumerate() {
            if self.skipped > 0 && i == Self::KEEP / 2 {
                out.push(format!("      ... {} more ...", self.skipped));
            }
            out.push(format!("    {}  {what}", seconds(*at)));
        }
    }
}

/// Milliseconds as a person reads them: `12.3 s`.
pub fn seconds(ms: u64) -> String {
    format!("{}.{} s", ms / 1000, ms % 1000 / 100)
}

// ---------------------------------------------------------------------------
// The line
// ---------------------------------------------------------------------------

/// Where a line is in its life.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineState {
    Negotiating,
    Open,
    Failed,
}

/// A direct connection, opened by trading an offer and an answer.
///
/// Shaped after WebRTC because that is the hardest thing it has to be; a line
/// that needs no negotiation (UDP to a known address) just describes itself
/// with a placeholder. Packets are datagrams: they may be lost or reordered,
/// which is exactly what GGRS expects of UDP.
pub trait Line: Send + Sync {
    /// Start as the side that makes the offer.
    fn offer(&mut self);
    /// Start as the side that answers this offer.
    fn answer(&mut self, offer: &str);
    /// Finish, on the offering side, with the other side's answer.
    fn accept(&mut self, answer: &str);
    /// The offer or answer to post, once it is ready to be posted. A WebRTC
    /// description takes a moment: it lists the ways this machine can be
    /// reached, which includes asking a public server what its address looks
    /// like from outside.
    fn description(&mut self, now_ms: u64) -> Option<String>;
    fn state(&mut self) -> LineState;
    fn send(&mut self, packet: &[u8]);
    fn recv(&mut self) -> Option<Vec<u8>>;
    /// As [`Board::report`]: how the connection is going, for dev mode.
    fn report(&self, _out: &mut Vec<String>) {}
}

/// GGRS over any [`Line`].
///
/// Messages are encoded the way GGRS's own UDP socket encodes them, so the
/// bytes on a data channel are the bytes that would have been in a datagram.
pub struct LineSocket<L: Line>(pub L);

impl<L: Line> NonBlockingSocket<Peer> for LineSocket<L> {
    fn send_to(&mut self, msg: &Message, _to: &Peer) {
        if let Ok(bytes) = bincode::serialize(msg) {
            self.0.send(&bytes);
        }
    }

    fn receive_all_messages(&mut self) -> Vec<(Peer, Message)> {
        let mut out = Vec::new();
        while let Some(bytes) = self.0.recv() {
            // A packet that is not a GGRS message is somebody else's business:
            // the same rule GGRS's own socket follows.
            if let Ok(msg) = bincode::deserialize(&bytes) {
                out.push((Peer::Line, msg));
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// The rendezvous
// ---------------------------------------------------------------------------

/// A boxed socket GGRS can be started on.
pub struct Socket(pub Box<dyn NonBlockingSocket<Peer>>);

impl NonBlockingSocket<Peer> for Socket {
    fn send_to(&mut self, msg: &Message, to: &Peer) {
        self.0.send_to(msg, to);
    }
    fn receive_all_messages(&mut self) -> Vec<(Peer, Message)> {
        self.0.receive_all_messages()
    }
}

/// What a meeting ends with: everything [`crate::p2p::start`] needs.
pub struct Seat {
    pub socket: Socket,
    pub remote: Peer,
    /// 0 for player one.
    pub handle: usize,
}

/// How a meeting is going.
pub enum Progress {
    /// Not yet, and a sentence a player can read about why.
    Waiting(&'static str),
    Ready(Seat),
    /// It will not happen; the sentence says what to do instead.
    Failed(String),
}

/// Any way of finding the other player. Polled once a frame until it is
/// `Ready` or `Failed`, and then dropped.
pub trait Rendezvous: Send + Sync {
    /// `now_ms` is any clock that only goes forward, in milliseconds.
    fn poll(&mut self, now_ms: u64) -> Progress;
    /// Everything it knows about how the meeting is going, as lines of plain
    /// text: who it has heard from, and each meeting point's own story. Shown
    /// in dev mode (F10), and printed when a meeting fails, because "could not
    /// reach any meeting point" is the end of a story and this is the story.
    fn report(&self) -> Vec<String> {
        Vec::new()
    }
}

// ---------------------------------------------------------------------------
// Notes
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq)]
enum Note<'a> {
    Hello {
        from: u64,
        waited: u64,
        terms: &'a str,
    },
    /// `attempt` counts the offerer's tries. An offer from a new attempt means
    /// the offerer started over with a fresh line, and an answer to an old one
    /// would describe a connection that no longer exists.
    Offer {
        from: u64,
        to: u64,
        attempt: u32,
        offerer_first: bool,
        sdp: &'a str,
    },
    Answer {
        from: u64,
        to: u64,
        attempt: u32,
        sdp: &'a str,
    },
    /// Somebody speaking another version of this protocol.
    Foreign,
}

impl Note<'_> {
    fn write(&self) -> String {
        let head = format!("arena{PROTOCOL}");
        match self {
            Note::Hello {
                from,
                waited,
                terms,
            } => format!("{head} hello {from:016x} {waited} {terms}"),
            Note::Offer {
                from,
                to,
                attempt,
                offerer_first,
                sdp,
            } => format!(
                "{head} offer {from:016x} {to:016x} {attempt} {}\n{sdp}",
                u8::from(*offerer_first)
            ),
            Note::Answer {
                from,
                to,
                attempt,
                sdp,
            } => {
                format!("{head} answer {from:016x} {to:016x} {attempt}\n{sdp}")
            }
            Note::Foreign => String::new(),
        }
    }

    /// `None` for anything that is not a note at all: other people's traffic on
    /// a public topic, or garbage.
    fn read(text: &str) -> Option<Note<'_>> {
        let (head, body) = text.split_once('\n').unwrap_or((text, ""));
        let mut words = head.split(' ');
        let version = words.next()?.strip_prefix("arena")?;
        if version.parse::<u32>().ok()? != PROTOCOL {
            return Some(Note::Foreign);
        }
        let id = |w: Option<&str>| u64::from_str_radix(w?, 16).ok();
        match words.next()? {
            "hello" => Some(Note::Hello {
                from: id(words.next())?,
                waited: words.next()?.parse().ok()?,
                terms: words.next()?,
            }),
            "offer" => Some(Note::Offer {
                from: id(words.next())?,
                to: id(words.next())?,
                attempt: words.next()?.parse().ok()?,
                offerer_first: words.next()? == "1",
                sdp: body,
            }),
            "answer" => Some(Note::Answer {
                from: id(words.next())?,
                to: id(words.next())?,
                attempt: words.next()?.parse().ok()?,
                sdp: body,
            }),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// The room
// ---------------------------------------------------------------------------

struct Partner {
    id: u64,
    /// How long their latest hello said they had been waiting.
    waited: u64,
    /// When that hello arrived, on my clock. So they have now been waiting
    /// `waited + (now - last_heard)`: only an estimate -- the note took a
    /// moment to arrive -- which is fine, because it only has to be right
    /// when one of you was there first by seconds. Never `now - waited`: a
    /// clock that started when this page did is often younger than the
    /// other page's wait, and that subtraction is what once seated the
    /// newcomer first.
    last_heard: u64,
}

enum Stage {
    /// No partner, or a partner and I am waiting for their offer.
    Looking,
    /// I made the offer and have no answer yet.
    Offering {
        first: bool,
        attempt: u32,
        posted: Option<u64>,
    },
    /// I accepted their answer; the line is opening.
    Calling { first: bool },
    /// I answered their offer. `answer` is what I posted, kept so a repeated
    /// offer -- my answer was lost -- can be answered again.
    Answering {
        first: bool,
        attempt: u32,
        answer: Option<String>,
    },
}

/// The meeting protocol over any board and any line.
///
/// Two players who open the same room meet; a third waits until one of them
/// leaves. See the module documentation for the notes it trades.
pub struct Room<B: Board, L: Line> {
    board: B,
    line: Option<L>,
    make_line: Box<dyn Fn() -> L + Send + Sync>,
    me: u64,
    terms: String,
    joined: Option<u64>,
    last_hello: Option<u64>,
    partner: Option<Partner>,
    stage: Stage,
    /// Offers I have made, so each one is told apart from the last.
    attempts: u32,
    since: u64,
    /// Somebody with different terms is in the room.
    mismatch: bool,
    /// A partner went quiet and was forgotten; said until somebody arrives.
    left: bool,
    /// For [`Rendezvous::report`]: the last time polled, and what was traded.
    now: u64,
    hellos: u32,
    notes: u32,
    /// Notes the board delivered that were not ours to read.
    noise: u32,
}

impl<B: Board, L: Line> Room<B, L> {
    /// `me` must differ between the two players and should be random:
    /// [`fresh_id`] is one. `terms` is anything without a space that two
    /// clients must agree on to play.
    pub fn new(
        board: B,
        make_line: impl Fn() -> L + Send + Sync + 'static,
        me: u64,
        terms: &str,
    ) -> Self {
        let terms: String = terms
            .chars()
            .map(|c| if c.is_whitespace() { '_' } else { c })
            .collect();
        Room {
            board,
            line: Some(make_line()),
            make_line: Box::new(make_line),
            me,
            terms,
            joined: None,
            last_hello: None,
            partner: None,
            stage: Stage::Looking,
            attempts: 0,
            since: 0,
            mismatch: false,
            left: false,
            now: 0,
            hellos: 0,
            notes: 0,
            noise: 0,
        }
    }

    fn post(&mut self, note: Note) {
        self.board.post(&note.write());
    }

    /// Forget the partner and start over with a fresh line.
    fn reset(&mut self) {
        self.left = true;
        self.partner = None;
        self.stage = Stage::Looking;
        self.line = Some((self.make_line)());
    }

    fn read_notes(&mut self, now: u64) {
        while let Some(text) = self.board.take() {
            let Some(note) = Note::read(&text) else {
                self.noise += 1;
                continue;
            };
            self.notes += 1;
            match note {
                Note::Foreign => self.mismatch = true,
                Note::Hello {
                    from,
                    waited: theirs,
                    terms,
                } => {
                    if from == self.me {
                        continue;
                    }
                    if terms != self.terms {
                        self.mismatch = true;
                        continue;
                    }
                    match &mut self.partner {
                        Some(p) if p.id == from => {
                            p.last_heard = now;
                            p.waited = theirs;
                        }
                        Some(_) => {}
                        None => {
                            self.left = false;
                            self.partner = Some(Partner {
                                id: from,
                                waited: theirs,
                                last_heard: now,
                            })
                        }
                    }
                }
                Note::Offer {
                    from,
                    to,
                    attempt,
                    offerer_first,
                    sdp,
                } => {
                    let from_partner = self.partner.as_ref().is_some_and(|p| p.id == from);
                    if to != self.me || !from_partner {
                        continue;
                    }
                    match &self.stage {
                        // The same offer again: my answer was lost.
                        Stage::Answering {
                            attempt: answered,
                            answer: Some(answer),
                            ..
                        } if *answered == attempt => {
                            let again = answer.clone();
                            self.post(Note::Answer {
                                from: self.me,
                                to: from,
                                attempt,
                                sdp: &again,
                            });
                        }
                        Stage::Answering {
                            attempt: answered, ..
                        } if *answered == attempt => {}
                        // A first offer, or a new attempt: they started over,
                        // so the line I was answering on is dead. Start over
                        // too, on a fresh one.
                        Stage::Looking | Stage::Answering { .. } => {
                            if matches!(self.stage, Stage::Answering { .. }) {
                                self.line = Some((self.make_line)());
                            }
                            if let Some(line) = &mut self.line {
                                line.answer(sdp);
                            }
                            self.stage = Stage::Answering {
                                first: !offerer_first,
                                attempt,
                                answer: None,
                            };
                            self.since = now;
                        }
                        _ => {}
                    }
                }
                Note::Answer {
                    from,
                    to,
                    attempt,
                    sdp,
                } => {
                    let from_partner = self.partner.as_ref().is_some_and(|p| p.id == from);
                    if to != self.me || !from_partner {
                        continue;
                    }
                    if let Stage::Offering {
                        first,
                        attempt: offered,
                        ..
                    } = self.stage
                        && offered == attempt
                    {
                        if let Some(line) = &mut self.line {
                            line.accept(sdp);
                        }
                        self.stage = Stage::Calling { first };
                    }
                }
            }
        }
    }
}

impl<B: Board, L: Line + 'static> Rendezvous for Room<B, L> {
    fn poll(&mut self, now: u64) -> Progress {
        let joined = *self.joined.get_or_insert(now);
        self.now = now;
        let waited = now.saturating_sub(joined);
        let reach = self.board.reach(now);

        self.read_notes(now);

        // Repeat the hello until the line is open: it is also how the partner
        // knows I am still here.
        if self
            .last_hello
            .is_none_or(|t| now.saturating_sub(t) >= HELLO_EVERY_MS)
        {
            self.last_hello = Some(now);
            self.hellos += 1;
            let terms = self.terms.clone();
            self.post(Note::Hello {
                from: self.me,
                waited,
                terms: &terms,
            });
        }

        let Some(partner) = &self.partner else {
            return match reach {
                Reach::Down => Progress::Failed(
                    "Could not reach any meeting point. The public relays this \
                     uses may be down, or this network blocks them."
                        .into(),
                ),
                Reach::Trying => Progress::Waiting("Reaching the meeting point…"),
                Reach::Up if self.mismatch => Progress::Waiting(
                    "Someone is in this room with a different build or different \
                     settings. Both of you open the same link, and reload.",
                ),
                Reach::Up if self.left => {
                    Progress::Waiting("Your friend left. Waiting for them to come back…")
                }
                Reach::Up => Progress::Waiting("Waiting for your friend to open the link…"),
            };
        };
        let partner_id = partner.id;

        let line_state = self
            .line
            .as_mut()
            .map_or(LineState::Failed, |line| line.state());
        if line_state != LineState::Open
            && now.saturating_sub(partner.last_heard) > PARTNER_SILENT_MS
        {
            self.reset();
            return Progress::Waiting("Your friend left. Waiting for them to come back…");
        }

        // The lower id makes the offer, and decides who is player one.
        if matches!(self.stage, Stage::Looking) && self.me < partner_id {
            let theirs = partner.waited + now.saturating_sub(partner.last_heard);
            let first = waited >= theirs;
            if let Some(line) = &mut self.line {
                line.offer();
            }
            self.attempts += 1;
            self.stage = Stage::Offering {
                first,
                attempt: self.attempts,
                posted: None,
            };
            self.since = now;
        }

        let description = match &self.stage {
            Stage::Offering { .. } | Stage::Answering { answer: None, .. } => {
                self.line.as_mut().and_then(|line| line.description(now))
            }
            _ => None,
        };
        match &mut self.stage {
            Stage::Offering {
                first,
                attempt,
                posted,
            } => {
                if let Some(sdp) = description
                    && posted.is_none_or(|t| now.saturating_sub(t) >= OFFER_EVERY_MS)
                {
                    *posted = Some(now);
                    let (first, attempt) = (*first, *attempt);
                    self.post(Note::Offer {
                        from: self.me,
                        to: partner_id,
                        attempt,
                        offerer_first: first,
                        sdp: &sdp,
                    });
                }
            }
            Stage::Answering {
                attempt, answer, ..
            } => {
                if let Some(sdp) = description {
                    *answer = Some(sdp.clone());
                    let attempt = *attempt;
                    self.post(Note::Answer {
                        from: self.me,
                        to: partner_id,
                        attempt,
                        sdp: &sdp,
                    });
                }
            }
            _ => {}
        }

        let first = match self.stage {
            Stage::Looking => {
                return Progress::Waiting("Found your friend. Agreeing who goes where…");
            }
            Stage::Offering { first, .. }
            | Stage::Calling { first }
            | Stage::Answering { first, .. } => first,
        };
        match line_state {
            LineState::Open => {
                let line = self.line.take().expect("an open line");
                Progress::Ready(Seat {
                    socket: Socket(Box::new(LineSocket(line))),
                    remote: Peer::Line,
                    handle: if first { 0 } else { 1 },
                })
            }
            LineState::Failed => Progress::Failed(LINE_FAILED.into()),
            LineState::Negotiating if now.saturating_sub(self.since) > NEGOTIATE_MS => {
                Progress::Failed(LINE_FAILED.into())
            }
            LineState::Negotiating => {
                Progress::Waiting("Found your friend. Opening a direct connection…")
            }
        }
    }

    fn report(&self) -> Vec<String> {
        let mut out = Vec::new();
        let waited = self.now.saturating_sub(self.joined.unwrap_or(self.now));
        out.push(format!(
            "me {:016x}, in the room {}: {} hellos posted, {} notes read, {} unreadable",
            self.me,
            seconds(waited),
            self.hellos,
            self.notes,
            self.noise,
        ));
        out.push(match &self.partner {
            Some(p) => format!(
                "friend {:016x}, last heard {} ago",
                p.id,
                seconds(self.now.saturating_sub(p.last_heard))
            ),
            None if self.left => "friend: went quiet and was forgotten".into(),
            None => "friend: not heard from".into(),
        });
        if self.mismatch {
            out.push("someone with a different build or settings is in the room".into());
        }
        out.push(format!(
            "stage: {}",
            match &self.stage {
                Stage::Looking => "looking".to_string(),
                Stage::Offering {
                    attempt, posted, ..
                } => format!(
                    "offered (attempt {attempt}, {})",
                    if posted.is_some() {
                        "posted"
                    } else {
                        "still describing this machine"
                    }
                ),
                Stage::Calling { .. } => "their answer accepted, line opening".into(),
                Stage::Answering { answer, .. } => format!(
                    "answering their offer ({})",
                    if answer.is_some() {
                        "posted"
                    } else {
                        "still describing this machine"
                    }
                ),
            }
        ));
        if let Some(line) = &self.line {
            line.report(&mut out);
        }
        self.board.report(&mut out);
        out
    }
}

const LINE_FAILED: &str = "Found your friend, but a direct connection between your two \
     networks would not open. Some networks (often mobile data, or a strict office \
     or school network) refuse them; trying from another network usually works.";

/// A random id for this run, from whatever randomness the platform has.
///
/// Only has to differ between two players in one room, so it does not need to
/// be good randomness: a hasher's random keys on a desktop, `Math.random` in a
/// browser (where the standard library's are a constant).
pub fn fresh_id() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        let half = || (js_sys::Math::random() * 4_294_967_296.0) as u64;
        (half() << 32) | half()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::hash::{BuildHasher, Hasher};
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        h.write_u128(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos()),
        );
        h.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_trace_keeps_its_start_and_its_latest() {
        let mut trace = Trace::default();
        for i in 0..40 {
            trace.note(i * 100, format!("event {i}"));
        }
        let mut lines = Vec::new();
        trace.lines(&mut lines);
        assert_eq!(lines.len(), Trace::KEEP + 1, "{lines:#?}");
        assert!(lines[0].ends_with("event 0"), "{}", lines[0]);
        assert!(lines[0].contains("0.0 s"), "{}", lines[0]);
        assert!(lines.iter().any(|l| l.contains("24 more")), "{lines:#?}");
        assert!(lines.last().unwrap().ends_with("event 39"));
        assert!(lines.last().unwrap().contains("3.9 s"));
    }

    #[test]
    fn every_note_reads_back_as_written() {
        let notes = [
            Note::Hello {
                from: 0xabc,
                waited: 1234,
                terms: "dev-00ff",
            },
            Note::Offer {
                from: 1,
                to: u64::MAX,
                attempt: 3,
                offerer_first: true,
                sdp: "v=0\r\no=- 1 2 IN IP4 0.0.0.0\r\n",
            },
            Note::Answer {
                from: 7,
                to: 9,
                attempt: 1,
                sdp: "line one\nline two",
            },
        ];
        for note in notes {
            assert_eq!(Note::read(&note.write()), Some(note));
        }
    }

    #[test]
    fn other_traffic_is_not_a_note_and_another_version_is_foreign() {
        assert_eq!(Note::read("hello world"), None);
        assert_eq!(Note::read(""), None);
        assert_eq!(Note::read("arena1 hello zz 1 x"), None);
        assert_eq!(
            Note::read(&format!("arena{} hello 1 1 x", PROTOCOL + 1)),
            Some(Note::Foreign)
        );
    }
}
