//! Meeting the other players.
//!
//! How two to four people who share nothing but a room name end up with a line
//! between every pair of them that GGRS can run a match over. The design is three seams, each a
//! trait, so any one of them can be replaced without touching the other two:
//!
//! - A [`Board`] is **where strangers leave notes**: a public message broker,
//!   two tabs of one browser, a UDP port. It carries a handful of short texts
//!   and nothing of the match. (Networking calls this *signalling*.)
//! - A [`Line`] is **the direct connection itself**, opened by trading two
//!   descriptions of how to reach each end -- an offer and an answer. A WebRTC
//!   data channel in a browser; on a desktop, the UDP port the board was
//!   already using. GGRS runs over them through [`Mesh`], one line to each
//!   other player.
//! - A [`Rendezvous`] is **the whole meeting**, polled once a frame until it
//!   hands over a [`Seat`]: a socket, the far ends, and which player you are.
//!
//! [`Room`] is the one rendezvous there is, and it is generic over the other
//! two. Its protocol is four lines of text, which is why the board can be
//! anything that carries text:
//!
//! ```text
//! arena2 hello  <from> <waited ms> <terms>     every second, until connected
//! arena2 roster <from> <id> <id> ...           the seats, player one first
//! arena2 offer  <from> <to> <attempt>          + the offer, until answered
//! arena2 answer <from> <to> <attempt>          + the answer, once per offer
//! ```
//!
//! **Terms** are what two clients must agree on before a frame is played: the
//! build, and the checksum of the world the match starts from -- which covers
//! the arena, both classes, and every tuned number, because tuning is folded
//! into `World::checksum`. A hello with different terms is never answered, and
//! the room says why instead of letting the match desync on its first frame.
//!
//! **Who sits where** is decided once, by one peer, so it cannot come out
//! differently on two machines: when the room is full, the lowest id posts the
//! roster, longest wait first -- player one is whoever had been in the room
//! longest, which is nearly always whoever sent the link. Then every pair of
//! players opens a line, the lower id of the two making the offer. (Before
//! there were four, the one offer of a pair carried who was player one. The
//! rule before that ordered the two peers' addresses, and each machine only
//! knew its own as `127.0.0.1`, so across two machines both claimed the same
//! slot.)

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
///
/// Two since four players: the roster note, and an offer that no longer says
/// who is player one.
pub const PROTOCOL: u32 = 2;

/// How often a hello is repeated while nobody has answered it. Every note can
/// be lost -- a public broker delivers at most once -- so everything is
/// repeated until the thing it asks for has happened.
pub(crate) const HELLO_EVERY_MS: u64 = 1_000;

/// How often an unanswered offer is repeated.
pub(crate) const OFFER_EVERY_MS: u64 = 2_000;

/// A partner who has sent no hello for this long, before the line opened, has
/// gone: closed the tab, or reloaded and come back as somebody new. The room
/// forgets them and starts again.
///
/// Generous, because a page only says hello from its frame loop, and a frame
/// can take seconds: the first one that draws a new fighter compiles its
/// shaders, and a browser rendering in software (the smoke test's) manages
/// two frames a second.
pub(crate) const PARTNER_SILENT_MS: u64 = 15_000;

/// From finding a partner to an open line. Past this the line is not going to
/// open: one of the two networks will not take a direct connection.
pub(crate) const NEGOTIATE_MS: u64 = 30_000;

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
    /// It could not be opened, or it was open and the far end has gone.
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

/// GGRS over any number of [`Line`]s: one to each other player.
///
/// Messages are encoded the way GGRS's own UDP socket encodes them, so the
/// bytes on a data channel are the bytes that would have been in a datagram.
/// Each line is addressed by the seat at its far end, which is what GGRS
/// calls the remote player's address.
///
/// **Crowd packets ride the same lines** (`crate::crowd`): a cell leader's
/// aggregate, tagged so GGRS's own socket would drop it, goes into the post
/// box instead, and the box's outgoing packets go out whenever GGRS polls.
pub struct Mesh<L: Line> {
    pub lines: Vec<(Peer, L)>,
    pub post: crate::crowd::Post,
    /// How many times GGRS has polled: the post box's clock.
    polls: u32,
    /// The lines already said to have closed.
    gone: Vec<Peer>,
    /// **Whose session these packets are**: a room's lineup's epoch, written
    /// in front of every GGRS message and read off every one that arrives
    /// (`crate::lobby`). When a lineup changes, one peer's new session and
    /// another's old one share the lines for a moment, and GGRS takes a
    /// message from a session of another size for its own and indexes past
    /// its players. `None` for a one-off room: no tag, as ever.
    epoch: Option<u32>,
}

/// The first byte of a GGRS message tagged with its session's epoch.
const EPOCH_TAG: u8 = 0xE9;

impl<L: Line> Mesh<L> {
    pub fn new(lines: Vec<(Peer, L)>, post: crate::crowd::Post) -> Mesh<L> {
        Mesh {
            lines,
            post,
            polls: 0,
            gone: Vec::new(),
            epoch: None,
        }
    }

    /// A mesh for one lineup's session: see [`Mesh::epoch`].
    pub fn for_epoch(mut self, epoch: u32) -> Mesh<L> {
        self.epoch = Some(epoch);
        self
    }
}

impl<L: Line> NonBlockingSocket<Peer> for Mesh<L> {
    fn send_to(&mut self, msg: &Message, to: &Peer) {
        let Some((_, line)) = self.lines.iter_mut().find(|(p, _)| p == to) else {
            return;
        };
        if let Ok(bytes) = bincode::serialize(msg) {
            match self.epoch {
                Some(epoch) => {
                    let mut tagged = Vec::with_capacity(bytes.len() + 5);
                    tagged.push(EPOCH_TAG);
                    tagged.extend_from_slice(&epoch.to_le_bytes());
                    tagged.extend_from_slice(&bytes);
                    line.send(&tagged);
                }
                None => line.send(&bytes),
            }
        }
    }

    fn receive_all_messages(&mut self) -> Vec<(Peer, Message)> {
        self.polls = self.polls.wrapping_add(1);
        for (to, packet) in self.post.take_outgoing() {
            if let Some((_, line)) = self.lines.iter_mut().find(|(p, _)| *p == to) {
                line.send(&packet);
            }
        }
        let mut out = Vec::new();
        for (peer, line) in &mut self.lines {
            // **A line that has closed is somebody gone**, said once to the
            // game through the post box: it gives the seat up now, on a frame
            // GGRS agrees with the others, instead of after the session's
            // fifteen silent seconds -- which are there for a page busy
            // drawing a new arena, not for a player who has left.
            if line.state() == LineState::Failed && !self.gone.contains(peer) {
                self.gone.push(*peer);
                self.post.line_closed(*peer);
            }
            while let Some(bytes) = line.recv() {
                if crate::crowd::decode(&bytes).is_some() {
                    self.post.heard(*peer, &bytes, self.polls);
                    continue;
                }
                // Another lineup's session is not this one's business.
                let message = match self.epoch {
                    Some(epoch) => {
                        if bytes.len() < 5
                            || bytes[0] != EPOCH_TAG
                            || bytes[1..5] != epoch.to_le_bytes()
                        {
                            continue;
                        }
                        &bytes[5..]
                    }
                    None => &bytes[..],
                };
                // A packet that is not a GGRS message is somebody else's
                // business: the same rule GGRS's own socket follows.
                if let Ok(msg) = bincode::deserialize(message) {
                    out.push((*peer, msg));
                }
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
    /// Every other player: their seat, and where GGRS reaches them.
    pub remotes: Vec<(usize, Peer)>,
    /// 0 for player one.
    pub handle: usize,
    /// How many play: two to four.
    pub players: usize,
    /// Everybody's id, in seat order: what a cell's leaders are ranked by
    /// (`crate::cells`).
    pub roster: Vec<u64>,
    /// The crowd post box the mesh carries for (`crate::crowd`).
    pub post: crate::crowd::Post,
}

/// How a meeting is going.
pub enum Progress {
    /// Not yet, and a sentence a player can read about why.
    Waiting(&'static str),
    Ready(Seat),
    /// It will not happen; the sentence says what to do instead.
    Failed(String),
}

/// Any way of finding the other players. Polled once a frame until it is
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
    /// **Who sits where**: every id in the room, player one first. Posted by
    /// the lowest id once everybody is there, and repeated until its lines
    /// are open; everybody else takes their seat from it.
    Roster { from: u64, ids: Vec<u64> },
    /// `attempt` counts the offerer's tries. An offer from a new attempt means
    /// the offerer started over with a fresh line, and an answer to an old one
    /// would describe a connection that no longer exists.
    Offer {
        from: u64,
        to: u64,
        attempt: u32,
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
            Note::Roster { from, ids } => {
                let ids: Vec<String> = ids.iter().map(|id| format!("{id:016x}")).collect();
                format!("{head} roster {from:016x} {}", ids.join(" "))
            }
            Note::Offer {
                from,
                to,
                attempt,
                sdp,
            } => format!("{head} offer {from:016x} {to:016x} {attempt}\n{sdp}"),
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
            "roster" => {
                let from = id(words.next())?;
                let ids = words
                    .map(|w| u64::from_str_radix(w, 16).ok())
                    .collect::<Option<Vec<u64>>>()?;
                (ids.len() >= 2).then_some(Note::Roster { from, ids })
            }
            "offer" => Some(Note::Offer {
                from: id(words.next())?,
                to: id(words.next())?,
                attempt: words.next()?.parse().ok()?,
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
    /// Waiting for their offer: they have the lower id.
    Looking,
    /// I made the offer and have no answer yet.
    Offering { attempt: u32, posted: Option<u64> },
    /// I accepted their answer; the line is opening.
    Calling,
    /// I answered their offer. `answer` is what I posted, kept so a repeated
    /// offer -- my answer was lost -- can be answered again.
    Answering {
        attempt: u32,
        answer: Option<String>,
    },
}

/// One line of the mesh: to one other player.
struct Link<L> {
    partner: u64,
    line: L,
    stage: Stage,
}

/// The meeting protocol over any board and any line.
///
/// As many players as the room is for -- two unless [`Room::with_players`]
/// says otherwise -- who open the same room meet; one more waits until
/// somebody leaves. See the module documentation for the notes it trades.
pub struct Room<B: Board, L: Line> {
    board: B,
    make_line: Box<dyn Fn() -> L + Send + Sync>,
    me: u64,
    terms: String,
    /// How many play, me included: two to four.
    players: usize,
    joined: Option<u64>,
    last_hello: Option<u64>,
    /// Everybody else I have heard from, up to `players - 1`.
    partners: Vec<Partner>,
    /// Who sits where, once it is agreed: every id, player one first.
    roster: Option<Vec<u64>>,
    roster_posted: Option<u64>,
    /// A line to each other player, once the roster is agreed.
    links: Vec<Link<L>>,
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
    /// `me` must differ between the players and should be random:
    /// [`fresh_id`] is one. `terms` is anything without a space that the
    /// clients must agree on to play. A room for two; see
    /// [`Room::with_players`].
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
            make_line: Box::new(make_line),
            me,
            terms,
            players: 2,
            joined: None,
            last_hello: None,
            partners: Vec::new(),
            roster: None,
            roster_posted: None,
            links: Vec::new(),
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

    /// **A room for `n` players**, two to [`sim::state::MAX_PLAYERS`]. Every
    /// one of them opens the same link, and the match starts when all `n`
    /// are in and every pair of them has a line.
    pub fn with_players(mut self, n: usize) -> Self {
        self.players = n.clamp(2, sim::state::MAX_PLAYERS);
        self
    }

    fn post(&mut self, note: Note) {
        self.board.post(&note.write());
    }

    /// Forget the seating and every line, and start over. Who is still here
    /// stays known; the partner who went quiet is already gone.
    fn reset(&mut self) {
        self.left = true;
        self.roster = None;
        self.roster_posted = None;
        self.links.clear();
    }

    fn link_mut(&mut self, partner: u64) -> Option<&mut Link<L>> {
        self.links.iter_mut().find(|l| l.partner == partner)
    }

    /// Take the seats this roster gives, and a line to each other player in
    /// it: an offer to everybody with a higher id, and a wait for one from
    /// everybody lower.
    fn seat(&mut self, ids: Vec<u64>, now: u64) {
        self.links.clear();
        for &partner in ids.iter().filter(|&&id| id != self.me) {
            let mut line = (self.make_line)();
            let stage = if self.me < partner {
                line.offer();
                self.attempts += 1;
                Stage::Offering {
                    attempt: self.attempts,
                    posted: None,
                }
            } else {
                Stage::Looking
            };
            self.links.push(Link {
                partner,
                line,
                stage,
            });
        }
        self.roster = Some(ids);
        self.roster_posted = None;
        self.since = now;
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
                    if let Some(p) = self.partners.iter_mut().find(|p| p.id == from) {
                        p.last_heard = now;
                        p.waited = theirs;
                    } else if self.partners.len() + 1 < self.players && self.roster.is_none() {
                        self.left = false;
                        self.partners.push(Partner {
                            id: from,
                            waited: theirs,
                            last_heard: now,
                        });
                    }
                }
                Note::Roster { from, ids } => {
                    // Only the lowest id of a room that is this room -- me,
                    // and exactly the players I know -- seats anybody.
                    let lowest = ids.iter().copied().min();
                    let known =
                        |id: &u64| *id == self.me || self.partners.iter().any(|p| p.id == *id);
                    let ours = ids.len() == self.players
                        && lowest == Some(from)
                        && from != self.me
                        && ids.contains(&self.me)
                        && ids.iter().all(known);
                    if ours && self.roster.as_ref() != Some(&ids) {
                        self.seat(ids, now);
                    }
                }
                Note::Offer {
                    from,
                    to,
                    attempt,
                    sdp,
                } => {
                    if to != self.me {
                        continue;
                    }
                    let fresh = self.make_line.as_ref();
                    let Some(link) = self.links.iter_mut().find(|l| l.partner == from) else {
                        continue;
                    };
                    match &link.stage {
                        // The same offer again: my answer was lost.
                        Stage::Answering {
                            attempt: answered,
                            answer: Some(answer),
                        } if *answered == attempt => {
                            let again = answer.clone();
                            self.board.post(
                                &Note::Answer {
                                    from: self.me,
                                    to: from,
                                    attempt,
                                    sdp: &again,
                                }
                                .write(),
                            );
                        }
                        Stage::Answering {
                            attempt: answered, ..
                        } if *answered == attempt => {}
                        // A first offer, or a new attempt: they started over,
                        // so the line I was answering on is dead. Start over
                        // too, on a fresh one.
                        Stage::Looking | Stage::Answering { .. } => {
                            if matches!(link.stage, Stage::Answering { .. }) {
                                link.line = fresh();
                            }
                            link.line.answer(sdp);
                            link.stage = Stage::Answering {
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
                    if to != self.me {
                        continue;
                    }
                    let Some(link) = self.link_mut(from) else {
                        continue;
                    };
                    if let Stage::Offering {
                        attempt: offered, ..
                    } = link.stage
                        && offered == attempt
                    {
                        link.line.accept(sdp);
                        link.stage = Stage::Calling;
                    }
                }
            }
        }
    }

    /// Waiting for the room to fill: what to say.
    fn waiting_for(&self) -> &'static str {
        match (self.players, self.players - 1 - self.partners.len()) {
            (2, _) => "Waiting for your friend to open the link…",
            (_, 1) => "Waiting for one more friend to open the link…",
            (_, 2) => "Waiting for two more friends to open the link…",
            _ => "Waiting for your friends to open the link…",
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

        // Repeat the hello until the lines are open: it is also how the
        // others know I am still here.
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

        // Somebody gone quiet before their line opened has left: forget
        // them, and the seating that counted on them.
        let open = |links: &mut Vec<Link<L>>, id: u64| {
            links
                .iter_mut()
                .find(|l| l.partner == id)
                .is_some_and(|l| l.line.state() == LineState::Open)
        };
        let mut gone = Vec::new();
        for p in &self.partners {
            if now.saturating_sub(p.last_heard) > PARTNER_SILENT_MS {
                gone.push(p.id);
            }
        }
        gone.retain(|id| !open(&mut self.links, *id));
        if !gone.is_empty() {
            self.partners.retain(|p| !gone.contains(&p.id));
            self.reset();
            return Progress::Waiting(if self.players == 2 {
                "Your friend left. Waiting for them to come back…"
            } else {
                "A friend left. Waiting for them to come back…"
            });
        }

        if self.partners.len() + 1 < self.players {
            return match reach {
                Reach::Down => Progress::Failed(
                    "Could not reach any meeting point. The public relays this \
                     uses may be down, or this network blocks them."
                        .into(),
                ),
                Reach::Trying => Progress::Waiting("Reaching the meeting point…"),
                Reach::Up if self.mismatch => Progress::Waiting(
                    "Someone is in this room with a different build or different \
                     settings. Everybody open the same link, and reload.",
                ),
                Reach::Up if self.left && self.partners.is_empty() => {
                    Progress::Waiting("Your friend left. Waiting for them to come back…")
                }
                Reach::Up if self.left => {
                    Progress::Waiting("A friend left. Waiting for them to come back…")
                }
                Reach::Up => Progress::Waiting(self.waiting_for()),
            };
        }

        // Everybody is here. The lowest id decides who sits where -- whoever
        // had been waiting longest is player one, which is nearly always
        // whoever sent the link -- and says so until its lines are open.
        let host = self
            .partners
            .iter()
            .map(|p| p.id)
            .chain([self.me])
            .min()
            .unwrap_or(self.me);
        if self.roster.is_none() && host == self.me {
            let mut waits: Vec<(u64, u64)> = self
                .partners
                .iter()
                .map(|p| (p.id, p.waited + now.saturating_sub(p.last_heard)))
                .chain([(self.me, waited)])
                .collect();
            // Longest wait first; a tie goes to the lower id, which is mine.
            waits.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            self.seat(waits.into_iter().map(|(id, _)| id).collect(), now);
        }
        let Some(roster) = self.roster.clone() else {
            return Progress::Waiting(if self.players == 2 {
                "Found your friend. Agreeing who goes where…"
            } else {
                "Everybody is here. Agreeing who goes where…"
            });
        };
        let all_open = self
            .links
            .iter_mut()
            .all(|l| l.line.state() == LineState::Open);
        if host == self.me
            && !all_open
            && self
                .roster_posted
                .is_none_or(|t| now.saturating_sub(t) >= HELLO_EVERY_MS)
        {
            self.roster_posted = Some(now);
            self.post(Note::Roster {
                from: self.me,
                ids: roster.clone(),
            });
        }

        // Every line's offer or answer, once it is described.
        let me = self.me;
        let mut out = Vec::new();
        for link in &mut self.links {
            match &mut link.stage {
                Stage::Offering { attempt, posted } => {
                    if posted.is_none_or(|t| now.saturating_sub(t) >= OFFER_EVERY_MS)
                        && let Some(sdp) = link.line.description(now)
                    {
                        *posted = Some(now);
                        out.push((true, link.partner, *attempt, sdp));
                    }
                }
                Stage::Answering {
                    attempt,
                    answer: answer @ None,
                } => {
                    if let Some(sdp) = link.line.description(now) {
                        *answer = Some(sdp.clone());
                        out.push((false, link.partner, *attempt, sdp));
                    }
                }
                _ => {}
            }
        }
        for (offer, to, attempt, sdp) in out {
            self.post(if offer {
                Note::Offer {
                    from: me,
                    to,
                    attempt,
                    sdp: &sdp,
                }
            } else {
                Note::Answer {
                    from: me,
                    to,
                    attempt,
                    sdp: &sdp,
                }
            });
        }

        let mut states = Vec::with_capacity(self.links.len());
        for link in &mut self.links {
            states.push(link.line.state());
        }
        if states.contains(&LineState::Failed) {
            return Progress::Failed(LINE_FAILED.into());
        }
        if states.iter().all(|s| *s == LineState::Open) {
            let handle = roster.iter().position(|&id| id == self.me).unwrap_or(0);
            let seat_of = |id: u64| roster.iter().position(|&r| r == id).unwrap_or(0);
            let lines: Vec<(Peer, L)> = std::mem::take(&mut self.links)
                .into_iter()
                .map(|l| (Peer::Line(seat_of(l.partner) as u8), l.line))
                .collect();
            let remotes = lines
                .iter()
                .map(|(peer, _)| match peer {
                    Peer::Line(h) => (*h as usize, *peer),
                })
                .collect();
            let ids = roster.clone();
            self.roster = None;
            let post = crate::crowd::PostBox::new();
            return Progress::Ready(Seat {
                socket: Socket(Box::new(Mesh::new(lines, post.clone()))),
                remotes,
                handle,
                players: self.players,
                roster: ids,
                post,
            });
        }
        if now.saturating_sub(self.since) > NEGOTIATE_MS {
            return Progress::Failed(LINE_FAILED.into());
        }
        let calling = self
            .links
            .iter()
            .any(|l| !matches!(l.stage, Stage::Looking));
        Progress::Waiting(match (self.players, calling) {
            (2, false) => "Found your friend. Agreeing who goes where…",
            (2, true) => "Found your friend. Opening a direct connection…",
            (_, false) => "Everybody is here. Agreeing who goes where…",
            (_, true) => "Everybody is here. Opening direct connections…",
        })
    }

    fn report(&self) -> Vec<String> {
        let mut out = Vec::new();
        let waited = self.now.saturating_sub(self.joined.unwrap_or(self.now));
        out.push(format!(
            "me {:016x}, in the room {} for {} players: {} hellos posted, {} notes read, {} unreadable",
            self.me,
            seconds(waited),
            self.players,
            self.hellos,
            self.notes,
            self.noise,
        ));
        for p in &self.partners {
            out.push(format!(
                "friend {:016x}, last heard {} ago",
                p.id,
                seconds(self.now.saturating_sub(p.last_heard))
            ));
        }
        if self.partners.is_empty() {
            out.push(if self.left {
                "friend: went quiet and was forgotten".into()
            } else {
                "friend: not heard from".into()
            });
        }
        if self.mismatch {
            out.push("someone with a different build or settings is in the room".into());
        }
        if let Some(roster) = &self.roster {
            let ids: Vec<String> = roster.iter().map(|id| format!("{id:016x}")).collect();
            out.push(format!("seats: {}", ids.join(", ")));
        }
        for link in &self.links {
            out.push(format!(
                "line to {:016x}: {}",
                link.partner,
                match &link.stage {
                    Stage::Looking => "waiting for their offer".to_string(),
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
                    Stage::Calling => "their answer accepted, line opening".into(),
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
            link.line.report(&mut out);
        }
        self.board.report(&mut out);
        out
    }
}

pub(crate) const LINE_FAILED: &str = "Found your friends, but a direct connection between your \
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
                sdp: "v=0\r\no=- 1 2 IN IP4 0.0.0.0\r\n",
            },
            Note::Answer {
                from: 7,
                to: 9,
                attempt: 1,
                sdp: "line one\nline two",
            },
            Note::Roster {
                from: 3,
                ids: vec![9, 3, 0xffff_0000_0000_0001],
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
        assert_eq!(Note::read(&format!("arena{PROTOCOL} hello zz 1 x")), None);
        assert_eq!(
            Note::read(&format!("arena{} hello 1 1 x", PROTOCOL + 1)),
            Some(Note::Foreign)
        );
    }
}
