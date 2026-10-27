//! **A room you come and go from** (`docs/design/versus.md`, "A room you come
//! and go from"; owner, 2026-10-22).
//!
//! [`crate::meet::Room`] meets once: when the room is full it seats everybody
//! and hands over one session, and nobody else can come in. A [`Lobby`] is the
//! room for the whole time it is open. Anybody who opens the link comes in,
//! up to the room's size, whenever they like; anybody can leave, and their
//! seat is free for the next.
//!
//! **The lineup** ([`Lineup`]) is who is in the match and in which world seat,
//! numbered by an epoch that goes up each time it changes. It is decided by
//! one peer, the **host** -- the lowest id among those still in it -- and
//! posted on the board until everybody has it. Every pair in a lineup opens a
//! line, the lower id making the offer, exactly as in a room; lines already
//! open stay open from one lineup to the next ([`Shared`]), so a newcomer
//! costs one new line each.
//!
//! - **The first lineup** is everybody there when the host first has company,
//!   longest wait first: the match starts the moment there are two. Until a
//!   session is running, a newcomer simply joins the lineup.
//! - **Once a session is running**, a newcomer is seated by the game, which
//!   knows the world: the host's game picks a seat on the smaller side
//!   (`sim::state::World::open_seat`) and calls [`Lobby::admit`]. The new
//!   lineup is posted, everybody opens a line to the newcomer, and the
//!   newcomer says so ([`Note::Joined`]) once all of theirs are open. Then
//!   the host's game puts the admission in its own input
//!   (`sim::input::Travel::admit`, from [`Lobby::admission`]), so every
//!   machine seats the newcomer on the same frame. When that frame is
//!   confirmed everybody starts a session for the new lineup from the world
//!   at it, and the host sends that world to the newcomer, in chunks over
//!   their line ([`Lobby::send_world`]), until they say they have it.
//! - **A room that is full** says so to anybody else who comes ([`Note::Full`]).
//! - **Somebody leaving** closes their lines; the session gives their seat up
//!   at once (`crate::crowd::PostBox::line_closed`), and the next lineup the
//!   host posts leaves them out.
//!
//! The notes, version three:
//!
//! ```text
//! arena3 hello  <from> <waited ms> <terms> <class>     every second
//! arena3 lineup <from> <epoch> <go> <newcomer> <id>:<seat>:<class> ...   the host's
//! arena3 joined <from> <epoch>                          a newcomer's lines are open
//! arena3 full   <from> <to>                             no seat for you
//! arena3 offer  <from> <to> <attempt>                   + the offer
//! arena3 answer <from> <to> <attempt>                   + the answer
//! ```

use crate::Peer;
use crate::meet::{
    Board, HELLO_EVERY_MS, LINE_FAILED, Line, LineState, Mesh, NEGOTIATE_MS, OFFER_EVERY_MS,
    PARTNER_SILENT_MS, Reach, Seat, Socket,
};
use sim::{Class, World};
use std::sync::{Arc, Mutex, MutexGuard};

/// The notes' version. Three since the room stopped closing.
pub const PROTOCOL: u32 = 3;

/// How often the world on its way to a newcomer is sent again, whole, until
/// they say they have it: lines lose packets.
const RESEND_WORLD_MS: u64 = 300;

/// The bytes of the world per packet.
const CHUNK: usize = 1024;

/// The first bytes of a packet of the world on its way to a newcomer: what
/// tells it from a GGRS message, as a crowd packet's tag does.
const TAG: [u8; 2] = [0x5A, 0x7E];

/// One player of a lineup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Member {
    pub id: u64,
    /// The world seat they play: their side and their colour.
    pub seat: u8,
    /// The class they came in as, for the seat they are given.
    pub class: Class,
}

/// **Who is in the match, and where**: the members by seat, and the epoch,
/// which goes up each time it changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lineup {
    pub epoch: u32,
    /// The host has started a session for it: everybody in it starts theirs
    /// as their lines open, and it grows no more. Before, anybody new simply
    /// joins it.
    pub go: bool,
    /// The member being let into a match already running, who starts from the
    /// world the host sends; nought for none.
    pub newcomer: u64,
    /// By seat, lowest first: GGRS's player `k` is `members[k]`.
    pub members: Vec<Member>,
}

impl Lineup {
    pub fn has(&self, id: u64) -> bool {
        self.members.iter().any(|m| m.id == id)
    }

    /// GGRS's players' world seats, in GGRS's order.
    pub fn seats(&self) -> Vec<usize> {
        self.members.iter().map(|m| m.seat as usize).collect()
    }

    /// The member in a seat.
    pub fn in_seat(&self, seat: usize) -> Option<Member> {
        self.members
            .iter()
            .copied()
            .find(|m| m.seat as usize == seat)
    }
}

/// **A line that outlives a session**: the lobby and every session's mesh
/// hold the same one, so when a lineup changes the session is replaced and
/// the lines are not.
pub struct Shared<L>(Arc<Mutex<Held<L>>>);

/// A shared line, and what the lobby read off it that was not its own.
struct Held<L> {
    line: L,
    /// **Packets set aside**: read by the lobby looking for the world, and
    /// not the world -- a session's, come before this machine's session did.
    /// Handed to the next reader first. Dropped, they were a peer's GGRS
    /// handshake, which its retry timer could put off for good: one side
    /// stuck synchronizing while the other ran (2026-10-22).
    aside: std::collections::VecDeque<Vec<u8>>,
}

/// The most packets set aside on a line: a few seconds' handshake.
const ASIDE: usize = 256;

impl<L> Clone for Shared<L> {
    fn clone(&self) -> Self {
        Shared(self.0.clone())
    }
}

impl<L> Shared<L> {
    pub fn new(line: L) -> Self {
        Shared(Arc::new(Mutex::new(Held {
            line,
            aside: std::collections::VecDeque::new(),
        })))
    }

    fn lock(&self) -> MutexGuard<'_, Held<L>> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl<L: Line> Shared<L> {
    /// The next packet of the world on its way in or out (tagged [`TAG`]),
    /// setting aside anything else for whoever reads next.
    fn recv_world(&mut self) -> Option<Vec<u8>> {
        let mut held = self.lock();
        while let Some(bytes) = held.line.recv() {
            if bytes.len() >= 7 && bytes[..2] == TAG {
                return Some(bytes);
            }
            if held.aside.len() < ASIDE {
                held.aside.push_back(bytes);
            }
        }
        None
    }
}

impl<L: Line> Line for Shared<L> {
    fn offer(&mut self) {
        self.lock().line.offer();
    }
    fn answer(&mut self, offer: &str) {
        self.lock().line.answer(offer);
    }
    fn accept(&mut self, answer: &str) {
        self.lock().line.accept(answer);
    }
    fn description(&mut self, now_ms: u64) -> Option<String> {
        self.lock().line.description(now_ms)
    }
    fn state(&mut self) -> LineState {
        self.lock().line.state()
    }
    fn send(&mut self, packet: &[u8]) {
        self.lock().line.send(packet);
    }
    fn recv(&mut self) -> Option<Vec<u8>> {
        let mut held = self.lock();
        held.aside.pop_front().or_else(|| held.line.recv())
    }
    fn report(&self, out: &mut Vec<String>) {
        self.lock().line.report(out);
    }
}

/// **A room as the game drives it**, for as long as it is open: what the
/// online driver holds instead of a one-off [`crate::Rendezvous`].
pub trait Gathering: Send + Sync {
    /// Read the board, post what is due, negotiate lines. Once a frame.
    fn poll(&mut self, now_ms: u64);
    /// A sentence for the player about where things are.
    fn status(&self) -> &'static str;
    /// It will not work, and why.
    fn failed(&self) -> Option<String>;
    /// Everything it knows, for the connection details (F10).
    fn report(&self) -> Vec<String>;
    /// The newest lineup this machine is in, if any.
    fn lineup(&self) -> Option<&Lineup>;
    /// This machine's id.
    fn me(&self) -> u64;
    /// Whether this machine is the room's host.
    fn hosting(&self) -> bool;
    /// The newest lineup has two at least, this machine in it, and every
    /// line it needs is open.
    fn lines_ready(&mut self) -> bool;
    /// A session's socket and seats for the newest lineup, over the lines
    /// open now, and GGRS's players' world seats.
    fn session(&mut self) -> (Seat, Vec<usize>);
    /// **Host**: somebody waiting to come in, and the class they came as,
    /// while there is a seat and nobody else is on their way in.
    fn newcomer(&self) -> Option<(u64, Class)>;
    /// **Host**: seat them: a new lineup with them in `seat`.
    fn admit(&mut self, id: u64, seat: usize);
    /// **Host**: once, when the newcomer has a line to everybody: the seat
    /// and class to put in this machine's input.
    fn admission(&mut self) -> Option<(usize, Class)>;
    /// **Host**: send the newcomer this world, the one their session starts
    /// from, until they have it.
    fn send_world(&mut self, world: &World);
    /// **Newcomer**: the world the host sent, once it is all here and is
    /// what the host said it was.
    fn take_world(&mut self) -> Option<World>;
    /// The session for the newest lineup is running: everybody in it has
    /// what they need, so nothing more is sent to a newcomer.
    fn settled(&mut self);
    /// **The lines open now** to everybody in the newest lineup, by id: what
    /// a room of groups plays over (`crate::group::Room`), in place of a
    /// session's mesh. They are a session's from then on, as
    /// [`Gathering::session`]'s are, and a host's newcomer is let in.
    fn group_lines(&mut self) -> Vec<(u64, Box<dyn Line>)>;
}

#[derive(Debug, PartialEq, Eq)]
enum Note<'a> {
    Hello {
        from: u64,
        waited: u64,
        terms: &'a str,
        class: Class,
        /// The epoch of the lineup they are in, nought for none: somebody
        /// who comes and hears one is coming into a room already going, and
        /// does not make a first lineup of their own.
        epoch: u32,
    },
    Lineup {
        from: u64,
        lineup: Lineup,
    },
    /// A newcomer has a line open to everybody in this lineup.
    Joined {
        from: u64,
        epoch: u32,
    },
    /// No seat for `to`.
    Full {
        from: u64,
        to: u64,
    },
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
    Foreign,
}

fn class_index(c: Class) -> usize {
    sim::class::ALL_CLASSES
        .iter()
        .position(|&k| k == c)
        .unwrap_or(0)
}

fn class_at(i: usize) -> Option<Class> {
    sim::class::ALL_CLASSES.get(i).copied()
}

impl Note<'_> {
    fn write(&self) -> String {
        let head = format!("arena{PROTOCOL}");
        match self {
            Note::Hello {
                from,
                waited,
                terms,
                class,
                epoch,
            } => format!(
                "{head} hello {from:016x} {waited} {terms} {} {epoch}",
                class_index(*class)
            ),
            Note::Lineup { from, lineup } => {
                let members: Vec<String> = lineup
                    .members
                    .iter()
                    .map(|m| format!("{:016x}:{}:{}", m.id, m.seat, class_index(m.class)))
                    .collect();
                format!(
                    "{head} lineup {from:016x} {} {} {:016x} {}",
                    lineup.epoch,
                    lineup.go as u8,
                    lineup.newcomer,
                    members.join(" ")
                )
            }
            Note::Joined { from, epoch } => format!("{head} joined {from:016x} {epoch}"),
            Note::Full { from, to } => format!("{head} full {from:016x} {to:016x}"),
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
            } => format!("{head} answer {from:016x} {to:016x} {attempt}\n{sdp}"),
            Note::Foreign => String::new(),
        }
    }

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
                class: class_at(words.next()?.parse().ok()?)?,
                epoch: words.next()?.parse().ok()?,
            }),
            "lineup" => {
                let from = id(words.next())?;
                let epoch = words.next()?.parse().ok()?;
                let go = words.next()? == "1";
                let newcomer = id(words.next())?;
                let mut members = Vec::new();
                for w in words {
                    let mut parts = w.split(':');
                    members.push(Member {
                        id: id(parts.next())?,
                        seat: parts.next()?.parse().ok()?,
                        class: class_at(parts.next()?.parse().ok()?)?,
                    });
                }
                members.sort_by_key(|m| m.seat);
                let seats_unique = members.windows(2).all(|w| w[0].seat != w[1].seat);
                (!members.is_empty()
                    && seats_unique
                    && members
                        .iter()
                        .all(|m| (m.seat as usize) < sim::state::MAX_PLAYERS))
                .then_some(Note::Lineup {
                    from,
                    lineup: Lineup {
                        epoch,
                        go,
                        newcomer,
                        members,
                    },
                })
            }
            "joined" => Some(Note::Joined {
                from: id(words.next())?,
                epoch: words.next()?.parse().ok()?,
            }),
            "full" => Some(Note::Full {
                from: id(words.next())?,
                to: id(words.next())?,
            }),
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

struct Partner {
    id: u64,
    waited: u64,
    last_heard: u64,
    class: Class,
    /// The epoch of the lineup their hello says they are in.
    epoch: u32,
}

enum Stage {
    Looking,
    Offering {
        attempt: u32,
        posted: Option<u64>,
    },
    Calling,
    Answering {
        attempt: u32,
        answer: Option<String>,
    },
}

struct Link<L> {
    partner: u64,
    line: Shared<L>,
    stage: Stage,
    /// When negotiating it began, for giving up on it.
    since: u64,
}

/// The world on its way to a newcomer.
struct Outgoing {
    to: u64,
    epoch: u32,
    packets: Vec<Vec<u8>>,
    last_sent: Option<u64>,
    acked: bool,
}

/// The world on its way in.
struct Incoming {
    epoch: u32,
    checksum: u64,
    parts: Vec<Option<Vec<u8>>>,
}

/// The room, for as long as it is open. See the module documentation.
pub struct Lobby<B: Board, L: Line> {
    board: B,
    make_line: Box<dyn Fn() -> L + Send + Sync>,
    me: u64,
    terms: String,
    /// The most there may be, me included.
    capacity: usize,
    /// The class I come in as.
    class: Class,
    joined: Option<u64>,
    last_hello: Option<u64>,
    partners: Vec<Partner>,
    /// The newest lineup heard or made.
    lineup: Option<Lineup>,
    lineup_posted: Option<u64>,
    links: Vec<Link<L>>,
    attempts: u32,
    /// The members of the lineup the running session was started for: their
    /// lines belong to the session's mesh, and are not read here.
    in_session: Vec<u64>,
    running: bool,
    /// **Host**: the epoch of a lineup that seats somebody, until their
    /// session runs; and whether its admission has been handed out.
    admitting: Option<(u32, u64)>,
    admission_given: bool,
    /// **Host**: newcomers who have said their lines are open, by epoch.
    heard_joined: Vec<(u64, u32)>,
    full_told: Option<u64>,
    /// Members of the lineup whose line has closed: gone, and not counted.
    departed: Vec<u64>,
    joined_posted: Option<u64>,
    outgoing: Option<Outgoing>,
    incoming: Option<Incoming>,
    received: Option<World>,
    /// The host said the room is full.
    full: bool,
    mismatch: bool,
    bad_world: bool,
    failed: Option<String>,
    now: u64,
    hellos: u32,
    notes: u32,
    noise: u32,
}

impl<B: Board, L: Line + 'static> Lobby<B, L> {
    /// `capacity` is the room's size, two to four; `class` is what this
    /// player comes in as. `me` and `terms` are as for a room.
    pub fn new(
        board: B,
        make_line: impl Fn() -> L + Send + Sync + 'static,
        me: u64,
        terms: &str,
        capacity: usize,
        class: Class,
    ) -> Self {
        let terms: String = terms
            .chars()
            .map(|c| if c.is_whitespace() { '_' } else { c })
            .collect();
        Lobby {
            board,
            make_line: Box::new(make_line),
            me,
            terms,
            capacity: capacity.clamp(2, sim::state::MAX_PLAYERS),
            class,
            joined: None,
            last_hello: None,
            partners: Vec::new(),
            lineup: None,
            lineup_posted: None,
            links: Vec::new(),
            attempts: 0,
            in_session: Vec::new(),
            running: false,
            admitting: None,
            admission_given: false,
            heard_joined: Vec::new(),
            full_told: None,
            departed: Vec::new(),
            joined_posted: None,
            outgoing: None,
            incoming: None,
            received: None,
            full: false,
            mismatch: false,
            bad_world: false,
            failed: None,
            now: 0,
            hellos: 0,
            notes: 0,
            noise: 0,
        }
    }

    fn post(&mut self, note: Note) {
        self.board.post(&note.write());
    }

    fn link_open(&mut self, id: u64) -> bool {
        self.links
            .iter_mut()
            .find(|l| l.partner == id)
            .is_some_and(|l| l.line.state() == LineState::Open)
    }

    fn link_failed(&mut self, id: u64) -> bool {
        self.links
            .iter_mut()
            .find(|l| l.partner == id)
            .is_some_and(|l| l.line.state() == LineState::Failed)
    }

    /// **The host**: the lowest id still here -- among the lineup's members
    /// whose line has not closed, once there is a lineup; among everybody
    /// heard from before.
    fn host(&mut self) -> u64 {
        let me = self.me;
        match self.lineup.clone() {
            // Never the newcomer a lineup is letting in, whatever their id:
            // they know nothing of the room yet, and the host who seated them
            // must go on posting the lineup until they have it.
            Some(lineup) if lineup.has(me) => {
                let mut ids = Vec::new();
                for m in &lineup.members {
                    if m.id == lineup.newcomer {
                        continue;
                    }
                    if m.id == me || !self.link_failed(m.id) {
                        ids.push(m.id);
                    }
                }
                ids.into_iter().min().unwrap_or(me)
            }
            // **Somebody already in a lineup** means a room going on: its
            // host is one of them, never somebody just come in -- who would
            // otherwise make a first lineup of their own, the lowest id of
            // everybody they had heard a hello from before the host's
            // lineup reached them.
            _ if self.partners.iter().any(|p| p.epoch > 0) => self
                .partners
                .iter()
                .filter(|p| p.epoch > 0)
                .map(|p| p.id)
                .min()
                .unwrap_or(me),
            _ => self
                .partners
                .iter()
                .map(|p| p.id)
                .chain([me])
                .min()
                .unwrap_or(me),
        }
    }

    /// Take this lineup: a line to every member I have none to (offering
    /// to those with higher ids), and none to anybody not in it.
    fn take_lineup(&mut self, lineup: Lineup, now: u64) {
        let me = self.me;
        self.links.retain(|l| lineup.has(l.partner));
        for m in lineup.members.iter().filter(|m| m.id != me) {
            if self.links.iter().any(|l| l.partner == m.id) {
                continue;
            }
            let mut line = Shared::new((self.make_line)());
            let stage = if me < m.id {
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
                partner: m.id,
                line,
                stage,
                since: now,
            });
        }
        self.lineup = Some(lineup);
        self.lineup_posted = None;
        self.joined_posted = None;
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
                    waited,
                    terms,
                    class,
                    epoch,
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
                        p.waited = waited;
                        p.class = class;
                        p.epoch = epoch;
                    } else {
                        self.partners.push(Partner {
                            id: from,
                            waited,
                            last_heard: now,
                            class,
                            epoch,
                        });
                    }
                }
                Note::Lineup { from, lineup } => {
                    // A lineup is the host's, and a newer epoch replaces an
                    // older one; one I am not in only tells me the room is
                    // going on without me.
                    if from == self.me {
                        continue;
                    }
                    let newer = self
                        .lineup
                        .as_ref()
                        .is_none_or(|have| lineup.epoch > have.epoch);
                    if newer && lineup.members.iter().any(|m| m.id == from) {
                        if lineup.has(self.me) {
                            self.full = false;
                            self.take_lineup(lineup, now);
                        } else if !self.running {
                            self.lineup = Some(lineup);
                        }
                    }
                }
                Note::Joined { from, epoch } => {
                    if !self.heard_joined.contains(&(from, epoch)) {
                        self.heard_joined.push((from, epoch));
                    }
                }
                Note::Full { to, .. } => {
                    if to == self.me && !self.lineup.as_ref().is_some_and(|l| l.has(self.me)) {
                        self.full = true;
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
                    let fresh = &self.make_line;
                    let Some(link) = self.links.iter_mut().find(|l| l.partner == from) else {
                        continue;
                    };
                    match &link.stage {
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
                        Stage::Looking | Stage::Answering { .. } => {
                            if matches!(link.stage, Stage::Answering { .. }) {
                                link.line = Shared::new(fresh());
                            }
                            link.line.answer(sdp);
                            link.stage = Stage::Answering {
                                attempt,
                                answer: None,
                            };
                            link.since = now;
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
                    let Some(link) = self.links.iter_mut().find(|l| l.partner == from) else {
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

    /// Every line's offer or answer, once it is described.
    fn negotiate(&mut self, now: u64) {
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
    }

    /// The packets of the world on lines no session reads: the newcomer's,
    /// both ways.
    fn read_lines(&mut self) {
        for k in 0..self.links.len() {
            if self.running && self.in_session.contains(&self.links[k].partner) {
                continue;
            }
            while let Some(bytes) = self.links[k].line.recv_world() {
                let epoch = u32::from_le_bytes([bytes[3], bytes[4], bytes[5], bytes[6]]);
                match bytes[2] {
                    // A chunk of the world.
                    0 if bytes.len() >= 19 => {
                        let checksum = u64::from_le_bytes(bytes[7..15].try_into().unwrap());
                        let index = u16::from_le_bytes([bytes[15], bytes[16]]) as usize;
                        let total = u16::from_le_bytes([bytes[17], bytes[18]]) as usize;
                        if self.received.is_some() || self.running {
                            // Have it: say so again, the last ack was lost.
                            let ack = ack(epoch);
                            self.links[k].line.send(&ack);
                            continue;
                        }
                        let fresh = self
                            .incoming
                            .as_ref()
                            .is_none_or(|i| i.epoch != epoch || i.parts.len() != total);
                        if fresh {
                            self.incoming = Some(Incoming {
                                epoch,
                                checksum,
                                parts: vec![None; total],
                            });
                        }
                        if let Some(inc) = &mut self.incoming
                            && index < inc.parts.len()
                        {
                            inc.parts[index] = Some(bytes[19..].to_vec());
                            if inc.parts.iter().all(Option::is_some) {
                                let whole: Vec<u8> =
                                    inc.parts.iter().flatten().flatten().copied().collect();
                                match crate::raw::world_from(&whole) {
                                    Some(w) if w.checksum() == inc.checksum => {
                                        self.received = Some(w);
                                        let ack = ack(epoch);
                                        self.links[k].line.send(&ack);
                                    }
                                    _ => self.bad_world = true,
                                }
                                self.incoming = None;
                            }
                        }
                    }
                    // They have it.
                    1 => {
                        if let Some(out) = &mut self.outgoing
                            && out.epoch == epoch
                            && out.to == self.links[k].partner
                        {
                            out.acked = true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

impl<B: Board, L: Line> Lobby<B, L> {
    /// How many of a lineup are still here.
    fn live(&self, lineup: &Lineup) -> usize {
        lineup
            .members
            .iter()
            .filter(|m| !self.departed.contains(&m.id))
            .count()
    }
}

fn ack(epoch: u32) -> Vec<u8> {
    let mut p = TAG.to_vec();
    p.push(1);
    p.extend_from_slice(&epoch.to_le_bytes());
    p
}

impl<B: Board, L: Line + 'static> Gathering for Lobby<B, L> {
    fn poll(&mut self, now: u64) {
        let joined = *self.joined.get_or_insert(now);
        self.now = now;
        let waited = now.saturating_sub(joined);
        if self.board.reach(now) == Reach::Down && !self.running {
            self.failed = Some(
                "Could not reach any meeting point. The public relays this uses may be \
                 down, or this network blocks them."
                    .into(),
            );
        }
        self.read_notes(now);
        if self
            .last_hello
            .is_none_or(|t| now.saturating_sub(t) >= HELLO_EVERY_MS)
        {
            self.last_hello = Some(now);
            self.hellos += 1;
            let terms = self.terms.clone();
            let class = self.class;
            let epoch = self
                .lineup
                .as_ref()
                .filter(|l| l.has(self.me))
                .map_or(0, |l| l.epoch);
            self.post(Note::Hello {
                from: self.me,
                waited,
                terms: &terms,
                class,
                epoch,
            });
        }
        // Somebody silent too long, with no open line, has gone.
        let silent: Vec<u64> = self
            .partners
            .iter()
            .filter(|p| now.saturating_sub(p.last_heard) > PARTNER_SILENT_MS)
            .map(|p| p.id)
            .collect();
        for id in silent {
            if !self.link_open(id) {
                self.partners.retain(|p| p.id != id);
            }
        }

        let me = self.me;
        if let Some(lineup) = self.lineup.clone() {
            let mut departed = Vec::new();
            for m in &lineup.members {
                if m.id != me && self.link_failed(m.id) {
                    departed.push(m.id);
                }
            }
            self.departed = departed;
        }
        let host = self.host();
        if host == me {
            let present: Vec<(u64, u64, Class)> = self
                .partners
                .iter()
                .map(|p| (p.id, p.waited + now.saturating_sub(p.last_heard), p.class))
                .collect();
            match self.lineup.clone() {
                // **The first lineup**: everybody here, longest wait first.
                None if !present.is_empty() => {
                    let mut waits: Vec<(u64, u64, Class)> = present;
                    waits.push((me, waited, self.class));
                    waits.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
                    waits.truncate(self.capacity);
                    let members = waits
                        .into_iter()
                        .enumerate()
                        .map(|(seat, (id, _, class))| Member {
                            id,
                            seat: seat as u8,
                            class,
                        })
                        .collect();
                    self.take_lineup(
                        Lineup {
                            epoch: 1,
                            go: false,
                            newcomer: 0,
                            members,
                        },
                        now,
                    );
                }
                // Before a session runs, a newcomer simply joins the lineup.
                Some(lineup) if !self.running && !lineup.go && lineup.has(me) => {
                    let mut grown = lineup.clone();
                    // Somebody gone quiet before any session is not waited for.
                    grown
                        .members
                        .retain(|m| m.id == me || present.iter().any(|p| p.0 == m.id));
                    for (id, _, class) in present {
                        if grown.members.len() >= self.capacity || grown.has(id) {
                            continue;
                        }
                        let free = (0..sim::state::MAX_PLAYERS as u8)
                            .find(|s| !grown.members.iter().any(|m| m.seat == *s));
                        if let Some(seat) = free {
                            grown.members.push(Member { id, seat, class });
                        }
                    }
                    if grown.members != lineup.members {
                        grown.members.sort_by_key(|m| m.seat);
                        grown.epoch += 1;
                        self.take_lineup(grown, now);
                    }
                }
                _ => {}
            }
            // **Go**: my lines to everybody are open, so I start, and so does
            // everybody else as theirs open; the lineup is final.
            if let Some(lineup) = self.lineup.clone()
                && !lineup.go
                && lineup.has(me)
                && self.lines_ready()
            {
                let mut go = lineup;
                go.go = true;
                go.epoch += 1;
                self.take_lineup(go, now);
            }
            // The lineup, until everybody has it -- and for anybody new.
            if let Some(lineup) = self.lineup.clone()
                && lineup.has(me)
                && self
                    .lineup_posted
                    .is_none_or(|t| now.saturating_sub(t) >= HELLO_EVERY_MS)
            {
                self.lineup_posted = Some(now);
                self.post(Note::Lineup { from: me, lineup });
            }
            // A full room says so.
            if self
                .lineup
                .as_ref()
                .is_some_and(|l| self.live(l) >= self.capacity)
                && self
                    .full_told
                    .is_none_or(|t| now.saturating_sub(t) >= HELLO_EVERY_MS)
            {
                self.full_told = Some(now);
                let outside: Vec<u64> = self
                    .partners
                    .iter()
                    .map(|p| p.id)
                    .filter(|id| !self.lineup.as_ref().is_some_and(|l| l.has(*id)))
                    .collect();
                for to in outside {
                    self.post(Note::Full { from: me, to });
                }
            }
        }

        self.negotiate(now);

        // A newcomer to a running match says when its lines are open.
        if let Some(lineup) = self.lineup.clone()
            && lineup.newcomer == me
            && !self.running
            && self.lines_ready()
            && self
                .joined_posted
                .is_none_or(|t| now.saturating_sub(t) >= HELLO_EVERY_MS)
        {
            self.joined_posted = Some(now);
            self.post(Note::Joined {
                from: me,
                epoch: lineup.epoch,
            });
        }

        // The world to a newcomer, again and again until they have it.
        if let Some(out) = &mut self.outgoing
            && !out.acked
            && out
                .last_sent
                .is_none_or(|t| now.saturating_sub(t) >= RESEND_WORLD_MS)
        {
            out.last_sent = Some(now);
            if let Some(link) = self.links.iter_mut().find(|l| l.partner == out.to) {
                for p in &out.packets {
                    link.line.send(p);
                }
            }
        }
        self.read_lines();

        // A line that will not open, before a session runs, is the end.
        if !self.running {
            let mut stuck = false;
            for link in &mut self.links {
                let state = link.line.state();
                if state == LineState::Failed
                    || state != LineState::Open && now.saturating_sub(link.since) > NEGOTIATE_MS
                {
                    stuck = true;
                }
            }
            if stuck {
                self.failed = Some(LINE_FAILED.into());
            }
        }
    }

    fn status(&self) -> &'static str {
        let seated = self.lineup.as_ref().is_some_and(|l| l.has(self.me));
        if self.mismatch && !seated {
            "Someone is in this room with a different build or different settings. \
             Everybody open the same link, and reload."
        } else if self.bad_world {
            "The match could not be joined from this kind of machine: the world it was sent \
             did not match. A bug -- please report it."
        } else if self.full {
            "This room is full. You'll be let in when somebody leaves."
        } else if self.running {
            "In the room."
        } else if seated && self.lineup.as_ref().is_some_and(|l| l.newcomer == self.me) {
            "Joining the match…"
        } else if seated {
            "Found your friends. Opening direct connections…"
        } else if self.lineup.is_some() {
            "Joining a match in progress…"
        } else if self.partners.is_empty() {
            "Waiting for your friends to open the link…"
        } else {
            "Found your friends. Agreeing who goes where…"
        }
    }

    fn failed(&self) -> Option<String> {
        self.failed.clone()
    }

    fn report(&self) -> Vec<String> {
        let mut out = vec![format!(
            "me {:016x}, a room for {}: {} hellos posted, {} notes read, {} unreadable",
            self.me, self.capacity, self.hellos, self.notes, self.noise
        )];
        if let Some(l) = &self.lineup {
            out.push(format!(
                "lineup {}: {}",
                l.epoch,
                l.members
                    .iter()
                    .map(|m| format!("{:016x} seat {} ({:?})", m.id, m.seat + 1, m.class))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        for p in &self.partners {
            out.push(format!(
                "heard {:016x} {} ago",
                p.id,
                crate::meet::seconds(self.now.saturating_sub(p.last_heard))
            ));
        }
        for link in &self.links {
            let stage = match &link.stage {
                Stage::Looking => "waiting for their offer".to_string(),
                Stage::Offering { attempt, .. } => format!("offered (attempt {attempt})"),
                Stage::Calling => "answer accepted".to_string(),
                Stage::Answering { attempt, answer } => format!(
                    "answering attempt {attempt}{}",
                    if answer.is_some() { ", answered" } else { "" }
                ),
            };
            out.push(format!("line to {:016x}: {stage}", link.partner));
            link.line.report(&mut out);
        }
        self.board.report(&mut out);
        out
    }

    fn lineup(&self) -> Option<&Lineup> {
        self.lineup.as_ref().filter(|l| l.has(self.me))
    }

    fn me(&self) -> u64 {
        self.me
    }

    fn hosting(&self) -> bool {
        // `host` reads line states, which needs `&mut`; this is the same
        // rule over what is known without asking the lines.
        match &self.lineup {
            Some(l) if l.has(self.me) => {
                l.members
                    .iter()
                    .map(|m| m.id)
                    .filter(|id| !self.departed.contains(id) && *id != l.newcomer)
                    .min()
                    == Some(self.me)
            }
            _ if self.partners.iter().any(|p| p.epoch > 0) => false,
            _ => self.partners.iter().map(|p| p.id).chain([self.me]).min() == Some(self.me),
        }
    }

    fn lines_ready(&mut self) -> bool {
        let Some(lineup) = self.lineup.clone().filter(|l| l.has(self.me)) else {
            return false;
        };
        if lineup.members.len() < 2 {
            return false;
        }
        let me = self.me;
        lineup
            .members
            .iter()
            .filter(|m| m.id != me)
            .all(|m| self.link_open(m.id))
    }

    fn session(&mut self) -> (Seat, Vec<usize>) {
        let lineup = self
            .lineup
            .clone()
            .filter(|l| l.has(self.me))
            .expect("a session for a lineup this machine is in");
        let me = self.me;
        let handle = lineup.members.iter().position(|m| m.id == me).unwrap_or(0);
        let mut lines = Vec::new();
        let mut remotes = Vec::new();
        for (k, m) in lineup.members.iter().enumerate() {
            if m.id == me {
                continue;
            }
            if let Some(link) = self.links.iter().find(|l| l.partner == m.id) {
                let peer = Peer::Line(k as u8);
                lines.push((peer, link.line.clone()));
                remotes.push((k, peer));
            }
        }
        self.running = true;
        self.in_session = lineup.members.iter().map(|m| m.id).collect();
        if self
            .admitting
            .is_some_and(|(epoch, _)| epoch <= lineup.epoch)
        {
            self.admitting = None;
            self.admission_given = false;
        }
        self.received = None;
        let post = crate::crowd::PostBox::new();
        let seat = Seat {
            socket: Socket(Box::new(
                Mesh::new(lines, post.clone()).for_epoch(lineup.epoch),
            )),
            remotes,
            handle,
            players: lineup.members.len(),
            roster: lineup.members.iter().map(|m| m.id).collect(),
            post,
        };
        (seat, lineup.seats())
    }

    fn newcomer(&self) -> Option<(u64, Class)> {
        let lineup = self.lineup.as_ref().filter(|l| l.has(self.me))?;
        if !self.running || self.admitting.is_some() || self.live(lineup) >= self.capacity {
            return None;
        }
        self.partners
            .iter()
            .find(|p| !lineup.has(p.id))
            .map(|p| (p.id, p.class))
    }

    fn admit(&mut self, id: u64, seat: usize) {
        let Some(mut lineup) = self.lineup.clone() else {
            return;
        };
        let Some(class) = self.partners.iter().find(|p| p.id == id).map(|p| p.class) else {
            return;
        };
        // Whoever has gone is not in the next lineup.
        let me = self.me;
        let mut keep = Vec::new();
        for m in &lineup.members {
            if m.id == me || !self.link_failed(m.id) {
                keep.push(*m);
            }
        }
        keep.retain(|m| m.seat as usize != seat);
        keep.push(Member {
            id,
            seat: seat as u8,
            class,
        });
        keep.sort_by_key(|m| m.seat);
        lineup.members = keep;
        lineup.epoch += 1;
        lineup.go = true;
        lineup.newcomer = id;
        self.admitting = Some((lineup.epoch, id));
        self.admission_given = false;
        let now = self.now;
        self.take_lineup(lineup, now);
    }

    fn admission(&mut self) -> Option<(usize, Class)> {
        let (epoch, id) = self.admitting?;
        if self.admission_given || !self.heard_joined.contains(&(id, epoch)) || !self.link_open(id)
        {
            return None;
        }
        let m = self.lineup.as_ref()?.members.iter().find(|m| m.id == id)?;
        let given = (m.seat as usize, m.class);
        self.admission_given = true;
        Some(given)
    }

    fn send_world(&mut self, world: &World) {
        let Some((epoch, to)) = self.admitting.or_else(|| {
            self.lineup
                .as_ref()
                .and_then(|l| l.members.iter().find(|m| !self.in_session.contains(&m.id)))
                .map(|m| (self.lineup.as_ref().map_or(0, |l| l.epoch), m.id))
        }) else {
            return;
        };
        let bytes = crate::raw::world_bytes(world);
        let checksum = world.checksum();
        let total = bytes.len().div_ceil(CHUNK) as u16;
        let packets = bytes
            .chunks(CHUNK)
            .enumerate()
            .map(|(index, chunk)| {
                let mut p = TAG.to_vec();
                p.push(0);
                p.extend_from_slice(&epoch.to_le_bytes());
                p.extend_from_slice(&checksum.to_le_bytes());
                p.extend_from_slice(&(index as u16).to_le_bytes());
                p.extend_from_slice(&total.to_le_bytes());
                p.extend_from_slice(chunk);
                p
            })
            .collect();
        self.outgoing = Some(Outgoing {
            to,
            epoch,
            packets,
            last_sent: None,
            acked: false,
        });
    }

    fn take_world(&mut self) -> Option<World> {
        self.received.take()
    }

    fn settled(&mut self) {
        self.outgoing = None;
    }

    fn group_lines(&mut self) -> Vec<(u64, Box<dyn Line>)> {
        let Some(lineup) = self.lineup.clone().filter(|l| l.has(self.me)) else {
            return Vec::new();
        };
        let me = self.me;
        let mut out: Vec<(u64, Box<dyn Line>)> = Vec::new();
        for m in &lineup.members {
            if m.id == me {
                continue;
            }
            if let Some(link) = self.links.iter_mut().find(|l| l.partner == m.id)
                && link.line.state() == LineState::Open
            {
                out.push((m.id, Box::new(link.line.clone())));
            }
        }
        self.running = true;
        self.in_session = lineup.members.iter().map(|m| m.id).collect();
        if self
            .admitting
            .is_some_and(|(epoch, _)| epoch <= lineup.epoch)
        {
            self.admitting = None;
            self.admission_given = false;
        }
        self.received = None;
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notes_read_back() {
        let lineup = Lineup {
            epoch: 3,
            go: true,
            newcomer: 0xabc,
            members: vec![
                Member {
                    id: 7,
                    seat: 0,
                    class: Class::Champion,
                },
                Member {
                    id: 0xabc,
                    seat: 3,
                    class: Class::DualMage,
                },
            ],
        };
        for note in [
            Note::Hello {
                from: 9,
                waited: 1200,
                terms: "b1-x",
                class: Class::Elementalist,
                epoch: 4,
            },
            Note::Lineup { from: 7, lineup },
            Note::Joined {
                from: 0xabc,
                epoch: 3,
            },
            Note::Full { from: 7, to: 0xabc },
        ] {
            let text = note.write();
            assert_eq!(Note::read(&text), Some(note), "{text}");
        }
        assert_eq!(Note::read("arena2 hello 1 0 x"), Some(Note::Foreign));
        assert_eq!(
            Note::read("arena3 lineup 1 2 0 0 1:0:0 2:0:0"),
            None,
            "one seat, two players"
        );
    }
}
