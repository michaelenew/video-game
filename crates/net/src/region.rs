//! **The region session, on the wire**: a peer that runs only the regions
//! it stands in (`docs/design/regions.md` §"The session layer";
//! `regions::session` is the same shape with no lines).
//!
//! What GGRS cannot do, because its players are fixed for the session: the
//! people a peer exchanges inputs with change as everybody walks. So this is
//! its own rollback session, with GGRS's rules where they fit:
//!
//! - **Inputs go to the members of my regions**, and only to them: every
//!   frame, the last [`REDUNDANCY`] frames of mine, so a lost packet costs
//!   nothing until that many are lost in a row. A member's missing input is
//!   predicted as its last one, and corrected by rollback when it comes.
//! - **Who is a member is declared, not inferred**: a peer knows exactly
//!   where its own fighter stands, so its packet header says which regions
//!   it is a member of this frame, and everybody takes its word. A region's
//!   members are those who declared it.
//! - **Outcomes go to the neighbours**: every [`OUTCOME_EVERY`] frames, for
//!   each region I am in, the fighters in its checksum zone at my newest
//!   confirmed frame, to every peer that declared a neighbouring region but
//!   not this one. A receiver rolls back to that frame, puts those fighters
//!   where I say, and plays forward again -- `regions::session::Peer::accept`
//!   -- which is what keeps its strip right without the inputs behind it.
//! - **A newcomer asks for a snapshot** (`Packet::Want`) and is sent the
//!   whole world at a confirmed frame (`Packet::Snapshot`, in chunks); it
//!   runs from there, and the inputs still in everybody's redundancy window
//!   catch it up.
//!
//! Nothing here is read by the game yet: the game plays four on GGRS with a
//! full checksum, and this is the machinery proven over the loopback lines
//! (`tests/region.rs`) for the day it is more than four.

use crate::meet::Line;
use crate::raw;
use regions::{TAPE, Tape};
use sim::World;
use sim::input::{Input, Travel};
use sim::region::{Grid, RegionId};
use sim::state::{MAX_PLAYERS, Player};

/// How many frames of my inputs every packet repeats.
pub const REDUNDANCY: usize = 8;

/// How far ahead of the oldest unconfirmed member input a peer will run
/// before it waits: GGRS's prediction window.
pub const WINDOW: u32 = 8;

/// **How often a peer too far to matter hears from me**, in frames: a
/// declaration with no inputs, and a promise (`Packet::Inputs::until`) that
/// what it declares holds that far ahead (exploration 0029 §9). Every frame
/// to everybody was most of what the session sent.
pub const HEARTBEAT: u32 = 6;

/// How far ahead a heartbeat's promise reaches: two beats, so the next one
/// is in hand before the last runs out.
pub const PROMISE: u32 = 2 * HEARTBEAT;

/// The trip a heartbeat is allowed, in frames, when deciding who is far.
const LATENCY: u32 = 18;

/// How often a region's outcome is sent to the neighbours, in frames.
pub const OUTCOME_EVERY: u32 = 4;

/// How long a new member of a region is still sent its outcomes, and still
/// takes them: its state at the frame it joined was a neighbour's guess
/// between two outcomes, and the old members' word settles it. Long enough
/// to hold several outcomes, so that losing some does not lose it.
pub const SETTLE: u32 = 4 * OUTCOME_EVERY;

/// A snapshot chunk's payload.
const CHUNK: usize = 1024;

/// The most regions a peer declares: its home and the six round it.
const DECLARED: usize = 7;

/// What goes on a line, by its first byte.
#[derive(Clone, Debug, PartialEq)]
pub enum Packet {
    /// My inputs for the frames ending at `frame`, newest last, and the
    /// regions I am a member of at `frame`.
    Inputs {
        seat: u8,
        frame: u32,
        inputs: Vec<Input>,
        regions: Vec<RegionId>,
        /// The declaration holds through this frame: `frame` itself every
        /// frame to a peer near enough to need my inputs, and further ahead
        /// in a heartbeat to one too far to (exploration 0029 §9).
        until: u32,
    },
    /// A region's checksum zone at a confirmed frame: the fighters in it.
    Outcome {
        region: RegionId,
        frame: u32,
        fighters: Vec<(u8, Player)>,
    },
    /// A newcomer wants the world: all of it, or the chunks of the snapshot
    /// at `frame` it is still missing (none missing means a fresh one).
    Want {
        seat: u8,
        frame: u32,
        missing: Vec<u16>,
    },
    /// The world at `frame`, in `total` chunks; this is chunk `index`.
    Snapshot {
        frame: u32,
        index: u16,
        total: u16,
        bytes: Vec<u8>,
    },
    /// My books at a confirmed frame: the checksum of every region I am an
    /// old member of, for the neighbours to hold against their own.
    Books {
        seat: u8,
        frame: u32,
        books: Vec<(RegionId, u64)>,
    },
}

const K_INPUTS: u8 = 1;
const K_OUTCOME: u8 = 2;
const K_WANT: u8 = 3;
const K_SNAPSHOT: u8 = 4;
const K_BOOKS: u8 = 5;

impl Packet {
    pub fn encode(&self) -> Vec<u8> {
        let mut b = Vec::new();
        match self {
            Packet::Inputs {
                seat,
                frame,
                inputs,
                regions,
                until,
            } => {
                b.push(K_INPUTS);
                b.push(*seat);
                b.extend_from_slice(&frame.to_le_bytes());
                b.push(until.saturating_sub(*frame).min(255) as u8);
                b.push(inputs.len() as u8);
                for i in inputs {
                    b.extend_from_slice(&i.bits.to_le_bytes());
                    b.extend_from_slice(&i.aim.to_le_bytes());
                    b.extend_from_slice(&i.pitch.to_le_bytes());
                    b.extend_from_slice(&i.travel.0.to_le_bytes());
                }
                b.push(regions.len() as u8);
                for r in regions {
                    b.extend_from_slice(&r.q.to_le_bytes());
                    b.extend_from_slice(&r.r.to_le_bytes());
                }
            }
            Packet::Outcome {
                region,
                frame,
                fighters,
            } => {
                b.push(K_OUTCOME);
                b.extend_from_slice(&region.q.to_le_bytes());
                b.extend_from_slice(&region.r.to_le_bytes());
                b.extend_from_slice(&frame.to_le_bytes());
                b.push(fighters.len() as u8);
                for (seat, p) in fighters {
                    b.push(*seat);
                    b.extend_from_slice(&raw::player_bytes(p));
                }
            }
            Packet::Want {
                seat,
                frame,
                missing,
            } => {
                b.push(K_WANT);
                b.push(*seat);
                b.extend_from_slice(&frame.to_le_bytes());
                b.push(missing.len() as u8);
                for m in missing {
                    b.extend_from_slice(&m.to_le_bytes());
                }
            }
            Packet::Snapshot {
                frame,
                index,
                total,
                bytes,
            } => {
                b.push(K_SNAPSHOT);
                b.extend_from_slice(&frame.to_le_bytes());
                b.extend_from_slice(&index.to_le_bytes());
                b.extend_from_slice(&total.to_le_bytes());
                b.extend_from_slice(bytes);
            }
            Packet::Books { seat, frame, books } => {
                b.push(K_BOOKS);
                b.push(*seat);
                b.extend_from_slice(&frame.to_le_bytes());
                b.push(books.len() as u8);
                for (r, sum) in books {
                    b.extend_from_slice(&r.q.to_le_bytes());
                    b.extend_from_slice(&r.r.to_le_bytes());
                    b.extend_from_slice(&sum.to_le_bytes());
                }
            }
        }
        b
    }

    pub fn decode(b: &[u8]) -> Option<Packet> {
        let mut at = 0usize;
        let mut take = |n: usize| -> Option<&[u8]> {
            let s = b.get(at..at + n)?;
            at += n;
            Some(s)
        };
        let u8_ = |s: &[u8]| s[0];
        let u16_ = |s: &[u8]| u16::from_le_bytes([s[0], s[1]]);
        let i16_ = |s: &[u8]| i16::from_le_bytes([s[0], s[1]]);
        let u32_ = |s: &[u8]| u32::from_le_bytes([s[0], s[1], s[2], s[3]]);
        match u8_(take(1)?) {
            K_INPUTS => {
                let seat = u8_(take(1)?);
                let frame = u32_(take(4)?);
                let until = frame + u8_(take(1)?) as u32;
                let n = u8_(take(1)?) as usize;
                let mut inputs = Vec::with_capacity(n);
                for _ in 0..n {
                    let bits = u16_(take(2)?);
                    let aim = u16_(take(2)?);
                    let pitch = i16_(take(2)?);
                    let travel = u16_(take(2)?);
                    inputs.push(Input::looking_at(bits, aim, pitch).travelling(Travel(travel)));
                }
                let m = u8_(take(1)?) as usize;
                let mut regions = Vec::with_capacity(m);
                for _ in 0..m {
                    let q = i16_(take(2)?);
                    let r = i16_(take(2)?);
                    regions.push(RegionId::new(q, r));
                }
                Some(Packet::Inputs {
                    seat,
                    frame,
                    inputs,
                    regions,
                    until,
                })
            }
            K_OUTCOME => {
                let q = i16_(take(2)?);
                let r = i16_(take(2)?);
                let frame = u32_(take(4)?);
                let n = u8_(take(1)?) as usize;
                let mut fighters = Vec::with_capacity(n);
                for _ in 0..n {
                    let seat = u8_(take(1)?);
                    let p = raw::player_from(take(raw::PLAYER)?)?;
                    fighters.push((seat, p));
                }
                Some(Packet::Outcome {
                    region: RegionId::new(q, r),
                    frame,
                    fighters,
                })
            }
            K_WANT => {
                let seat = u8_(take(1)?);
                let frame = u32_(take(4)?);
                let n = u8_(take(1)?) as usize;
                let mut missing = Vec::with_capacity(n);
                for _ in 0..n {
                    missing.push(u16_(take(2)?));
                }
                Some(Packet::Want {
                    seat,
                    frame,
                    missing,
                })
            }
            K_SNAPSHOT => {
                let frame = u32_(take(4)?);
                let index = u16_(take(2)?);
                let total = u16_(take(2)?);
                Some(Packet::Snapshot {
                    frame,
                    index,
                    total,
                    bytes: b[at..].to_vec(),
                })
            }
            K_BOOKS => {
                let seat = u8_(take(1)?);
                let frame = u32_(take(4)?);
                let n = u8_(take(1)?) as usize;
                let mut books = Vec::with_capacity(n);
                for _ in 0..n {
                    let q = i16_(take(2)?);
                    let r = i16_(take(2)?);
                    let s = take(8)?;
                    let sum = u64::from_le_bytes([s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]]);
                    books.push((RegionId::new(q, r), sum));
                }
                Some(Packet::Books { seat, frame, books })
            }
            _ => None,
        }
    }
}

/// What a peer knows of another: the regions it declared, at which frame,
/// and its inputs by frame.
#[derive(Clone)]
struct Known {
    /// Its declarations, by the frame each was made for, oldest first.
    declared: Vec<(u32, Vec<RegionId>)>,
    /// Inputs by frame, a ring tagged with the frame.
    inputs: Box<[Option<(u32, Input)>; TAPE]>,
    /// The newest input heard: what a missing one is predicted as.
    last: Option<(u32, Input)>,
    /// The newest frame its declarations hold through: its newest, or what
    /// a heartbeat promised past it.
    through: u32,
}

/// How many declarations are kept: the tape's reach and the window.
const DECLARATIONS: usize = TAPE + WINDOW as usize;

/// A peer's regions at `frame`: the newest declaration made for a frame no
/// later than it. Nothing, before its first.
fn declared_at(history: &[(u32, Vec<RegionId>)], frame: u32) -> &[RegionId] {
    history
        .iter()
        .rev()
        .find(|(f, _)| *f <= frame)
        .map_or(&[], |(_, r)| r.as_slice())
}

/// Keep a declaration in a history, in order; true if it is new for its
/// frame or differs from what was held for it.
fn declare(history: &mut Vec<(u32, Vec<RegionId>)>, frame: u32, regions: Vec<RegionId>) -> bool {
    match history.binary_search_by_key(&frame, |(f, _)| *f) {
        Ok(i) => {
            if history[i].1 == regions {
                return false;
            }
            history[i].1 = regions;
        }
        Err(i) => history.insert(i, (frame, regions)),
    }
    if history.len() > DECLARATIONS {
        history.remove(0);
    }
    true
}

/// **Is a peer too far to need my inputs** before a heartbeat's promise
/// runs out? The speed of light says: it stands within membership of every
/// region it declared, so it is at least the farthest of those gaps less
/// membership from me; we share a region only within two memberships and a
/// hexagon's width; and over the promise and a trip each of us closes at
/// most the grid's speed. Never far before it has declared anything.
fn far(grid: &Grid, me: sim::math::V3, theirs: &[(u32, Vec<RegionId>)]) -> bool {
    let Some((_, regions)) = theirs.last() else {
        return false;
    };
    let m = grid.membership();
    let Some(apart) = regions
        .iter()
        .map(|r| grid.gap(*r, me).sub(m))
        .max_by_key(|d| d.raw())
    else {
        return false;
    };
    let t = sim::fixed::Fx::from_int((PROMISE + LATENCY) as i32)
        .div(sim::fixed::Fx::from_int(sim::TICK_HZ as i32));
    let closing = grid.speed.mul(t).mul(sim::fixed::Fx::from_int(2));
    let share = m.add(m).add(grid.size).add(grid.size);
    apart.sub(closing).raw() > share.raw()
}

impl Known {
    fn new() -> Known {
        Known {
            declared: Vec::new(),
            inputs: Box::new([None; TAPE]),
            last: None,
            through: 0,
        }
    }

    /// The newest frame what it has declared holds through, if it has
    /// declared anything: its newest declaration, or further ahead where a
    /// heartbeat promised it.
    fn newest(&self) -> Option<u32> {
        self.declared.last().map(|(f, _)| (*f).max(self.through))
    }

    fn input_at(&self, frame: u32) -> Option<Input> {
        self.inputs[frame as usize % TAPE]
            .filter(|(f, _)| *f == frame)
            .map(|(_, i)| i)
    }

    /// Keep an input; true if it was news (not held, or held differently).
    fn learn(&mut self, frame: u32, input: Input) -> bool {
        let slot = &mut self.inputs[frame as usize % TAPE];
        let news = !matches!(slot, Some((f, i)) if *f == frame && *i == input);
        *slot = Some((frame, input));
        if self.last.is_none_or(|(f, _)| frame > f) {
            self.last = Some((frame, input));
        }
        news
    }
}

/// An outcome as heard: the region, the frame, and the fighters in its
/// zone then.
type Heard = (RegionId, u32, Vec<(u8, Player)>);

/// Books as heard: whose, at which frame, and each region's checksum.
type Books = (u8, u32, Vec<(RegionId, u64)>);

/// **A peer of the open world** on real lines.
pub struct Session<L: Line> {
    pub me: usize,
    grid: Grid,
    /// A line to each peer this one can reach, by seat.
    lines: Vec<(usize, L)>,
    /// The world, once there is one: a newcomer has none until a snapshot.
    pub world: Option<World>,
    tape: Box<Tape>,
    /// My own declarations, by frame: what I told everybody I was in.
    declared: Vec<(u32, Vec<RegionId>)>,
    known: Vec<Known>,
    /// My own inputs by frame, for the redundancy window.
    mine: Box<[Option<(u32, Input)>; TAPE]>,
    /// The newest frame every needed input is known for.
    pub confirmed: u32,
    /// The earliest frame whose inputs or strip have changed since it was
    /// played: where the next rollback starts.
    dirty: Option<u32>,
    /// Outcomes heard and not yet taken.
    outcomes: Vec<Heard>,
    /// Snapshot chunks on their way in.
    chunks: Option<(u32, Vec<Option<Vec<u8>>>)>,
    /// How many times the world was rolled back and played again.
    pub rollbacks: u32,
    /// Frames the session held for want of a member's input.
    pub stalls: u32,
    /// How many times the world was asked for, and ticks since the last.
    pub asked: u32,
    since_asked: u32,
    /// Outcomes sent, and outcomes taken into the tape.
    pub outcomes_sent: u32,
    pub outcomes_taken: u32,
    last_outcome_at: u32,
    /// Outcomes taken, by region and frame: where a neighbour's zone is
    /// exact here, so its books can be held against mine.
    settled: Vec<(RegionId, u32)>,
    /// Books heard and not yet held against mine.
    books: Vec<Books>,
    /// Region checksums held against a peer's and found the same, and
    /// found different: a difference is a desync, and the first is kept.
    pub agreed: u32,
    pub disagreed: u32,
    pub first_disagreement: Option<(u8, RegionId, u32)>,
    /// The frame each peer last had a heartbeat from me.
    beat: [u32; MAX_PLAYERS],
}

impl<L: Line> Session<L> {
    /// A peer with a world of its own from the start.
    pub fn new(me: usize, grid: Grid, world: World, lines: Vec<(usize, L)>) -> Session<L> {
        let mut s = Session::joining(me, grid, lines);
        s.confirmed = world.frame;
        s.last_outcome_at = world.frame;
        s.world = Some(world);
        s
    }

    /// A peer with no world yet: it asks the first line for one.
    pub fn joining(me: usize, grid: Grid, lines: Vec<(usize, L)>) -> Session<L> {
        Session {
            me,
            grid,
            lines,
            world: None,
            tape: Box::new(Tape::new()),
            declared: Vec::new(),
            known: (0..MAX_PLAYERS).map(|_| Known::new()).collect(),
            mine: Box::new([None; TAPE]),
            confirmed: 0,
            dirty: None,
            outcomes: Vec::new(),
            chunks: None,
            rollbacks: 0,
            stalls: 0,
            asked: 0,
            since_asked: 0,
            outcomes_sent: 0,
            outcomes_taken: 0,
            last_outcome_at: 0,
            settled: Vec::new(),
            books: Vec::new(),
            agreed: 0,
            disagreed: 0,
            first_disagreement: None,
            beat: [0; MAX_PLAYERS],
        }
    }

    /// Ask a peer for the world -- the chunks still missing of the snapshot
    /// on its way in, if one is -- a different peer each time, in case the
    /// last never answered.
    pub fn ask_for_the_world(&mut self) {
        let (frame, missing) = match &self.chunks {
            Some((frame, parts)) => (
                *frame,
                parts
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.is_none())
                    .map(|(i, _)| i as u16)
                    .collect(),
            ),
            None => (u32::MAX, Vec::new()),
        };
        let p = Packet::Want {
            seat: self.me as u8,
            frame,
            missing,
        }
        .encode();
        let n = self.lines.len();
        if n > 0 {
            let (_, line) = &mut self.lines[self.asked as usize % n];
            line.send(&p);
        }
        self.asked += 1;
        self.since_asked = 0;
    }

    /// How long a newcomer waits for the rest of a snapshot before asking
    /// for what is missing: well inside the tape, so the asked peer still
    /// has that frame.
    const ASK_AGAIN: u32 = 10;

    /// The regions I am a member of, from where my fighter stands.
    pub fn my_regions(&self) -> Vec<RegionId> {
        let Some(w) = &self.world else {
            return Vec::new();
        };
        let at = w.players[self.me].pos.add(w.map_origin());
        let home = self.grid.home(at);
        let reach = self.grid.membership().add(self.grid.horizon());
        std::iter::once(home)
            .chain(home.neighbours())
            .filter(|r| self.grid.gap(*r, at).raw() <= reach.raw())
            .take(DECLARED)
            .collect()
    }

    /// Whose inputs frame `frame` needs: mine, and every seat that declared
    /// a region I declared, both as of that frame. Asked again on every
    /// replay, because a declaration can arrive after its frame was played.
    fn needed_at(&self, frame: u32) -> [bool; MAX_PLAYERS] {
        let mine = declared_at(&self.declared, frame);
        let mut out = [false; MAX_PLAYERS];
        out[self.me] = true;
        for (seat, k) in self.known.iter().enumerate() {
            if seat != self.me
                && declared_at(&k.declared, frame)
                    .iter()
                    .any(|r| mine.contains(r))
            {
                out[seat] = true;
            }
        }
        out
    }

    /// **Hear everything on every line.**
    pub fn receive(&mut self) {
        let mut heard = Vec::new();
        for (seat, line) in self.lines.iter_mut() {
            while let Some(bytes) = line.recv() {
                if let Some(p) = Packet::decode(&bytes) {
                    heard.push((*seat, p));
                }
            }
        }
        for (from, p) in heard {
            match p {
                Packet::Inputs {
                    seat,
                    frame,
                    inputs,
                    regions,
                    until,
                } => {
                    let seat = seat as usize;
                    if seat >= MAX_PLAYERS {
                        continue;
                    }
                    let k = &mut self.known[seat];
                    k.through = k.through.max(until);
                    let now = self.world.as_ref().map(|w| w.frame);
                    // A declaration for a frame already played that changes
                    // who that frame needed: play it again from there.
                    let before = now.map(|_| self.needed_at(frame));
                    if declare(&mut self.known[seat].declared, frame, regions)
                        && let (Some(now), Some(before)) = (now, before)
                        && frame <= now
                        && self.needed_at(frame) != before
                    {
                        self.dirty = Some(self.dirty.map_or(frame, |d| d.min(frame)));
                    }
                    let first = frame + 1 - inputs.len() as u32;
                    for (n, input) in inputs.into_iter().enumerate() {
                        let f = first + n as u32;
                        if self.known[seat].learn(f, input) {
                            // News for a frame already played, from a seat
                            // that frame needed: play it again from there.
                            if let Some(now) = now
                                && f <= now
                                && self.needed_at(f)[seat]
                                && self.tape.get(f).is_some_and(|e| e.inputs[seat] != input)
                            {
                                self.dirty = Some(self.dirty.map_or(f, |d| d.min(f)));
                            }
                        }
                    }
                }
                Packet::Outcome {
                    region,
                    frame,
                    fighters,
                } => {
                    // Taken unless I was an old member at that frame: then
                    // my own zone is exact already, and it is a member just
                    // like the sender.
                    if !self.old_member(&self.declared, region, frame) {
                        self.outcomes.push((region, frame, fighters));
                    }
                }
                Packet::Want {
                    seat,
                    frame,
                    missing,
                } => {
                    // The world at my confirmed frame: nothing behind it
                    // will change, and the inputs since are still in
                    // everybody's redundancy window. Or the chunks it is
                    // missing of a frame I still have on the tape.
                    let again = !missing.is_empty() && frame <= self.confirmed;
                    let at = if again { frame } else { self.confirmed };
                    if let Some(w) = self.world_at(at) {
                        let bytes = raw::world_bytes(&w);
                        let total = bytes.len().div_ceil(CHUNK) as u16;
                        for (index, chunk) in bytes.chunks(CHUNK).enumerate() {
                            if again && !missing.contains(&(index as u16)) {
                                continue;
                            }
                            let p = Packet::Snapshot {
                                frame: w.frame,
                                index: index as u16,
                                total,
                                bytes: chunk.to_vec(),
                            }
                            .encode();
                            if let Some((_, line)) =
                                self.lines.iter_mut().find(|(s, _)| *s == seat as usize)
                            {
                                line.send(&p);
                            }
                        }
                        let _ = from;
                    }
                }
                Packet::Books { seat, frame, books } => {
                    self.books.push((seat, frame, books));
                }
                Packet::Snapshot {
                    frame,
                    index,
                    total,
                    bytes,
                } => {
                    if self.world.is_some() {
                        continue;
                    }
                    // A newer snapshot replaces one on its way in; an
                    // older one is a late answer, and ignored.
                    match &self.chunks {
                        Some((f, _)) if *f > frame => continue,
                        Some((f, _)) if *f == frame => {}
                        _ => self.chunks = Some((frame, vec![None; total as usize])),
                    }
                    if let Some((_, parts)) = &mut self.chunks
                        && (index as usize) < parts.len()
                    {
                        parts[index as usize] = Some(bytes);
                        if parts.iter().all(Option::is_some) {
                            let all: Vec<u8> = parts
                                .iter()
                                .flatten()
                                .flat_map(|c| c.iter().copied())
                                .collect();
                            if let Some(w) = raw::world_from(&all) {
                                self.confirmed = w.frame;
                                self.last_outcome_at = w.frame;
                                self.world = Some(w);
                            }
                            self.chunks = None;
                        }
                    }
                }
            }
        }
    }

    /// The input frame `f` plays for `seat`: mine, a member's known or
    /// predicted, or nothing for anyone else.
    fn input_for(&self, f: u32, seat: usize, needed: &[bool; MAX_PLAYERS]) -> Input {
        if seat == self.me {
            return self.mine[f as usize % TAPE]
                .filter(|(ff, _)| *ff == f)
                .map_or(Input::default(), |(_, i)| i);
        }
        if !needed[seat] {
            return Input::default();
        }
        let k = &self.known[seat];
        k.input_at(f).or(k.last.map(|(_, i)| i)).unwrap_or_default()
    }

    /// **One frame**, given my input: send it, take what arrived, roll back
    /// what has to be, and advance if the window allows. Returns whether the
    /// world advanced.
    pub fn tick(&mut self, input: Input) -> bool {
        self.receive();
        let Some(world) = self.world.as_ref() else {
            if self.asked > 0 {
                self.since_asked += 1;
                if self.since_asked >= Self::ASK_AGAIN {
                    self.ask_for_the_world();
                }
            }
            return false;
        };
        let next = world.frame + 1;
        let me = world.players[self.me].pos.add(world.map_origin());
        // My input for the frame about to be made, and the word to the
        // members of my regions.
        self.mine[next as usize % TAPE] = Some((next, input));
        let mine = self.my_regions();
        declare(&mut self.declared, next, mine.clone());
        let newest = next;
        let first = newest.saturating_sub(REDUNDANCY as u32 - 1).max(1);
        let inputs: Vec<Input> = (first..=newest)
            .map(|f| {
                self.mine[f as usize % TAPE]
                    .filter(|(ff, _)| *ff == f)
                    .map_or(Input::default(), |(_, i)| i)
            })
            .collect();
        let packet = Packet::Inputs {
            seat: self.me as u8,
            frame: newest,
            inputs,
            regions: mine.clone(),
            until: newest,
        }
        .encode();
        let beat = Packet::Inputs {
            seat: self.me as u8,
            frame: newest,
            inputs: Vec::new(),
            regions: mine.clone(),
            until: newest + PROMISE,
        }
        .encode();
        for (seat, line) in self.lines.iter_mut() {
            // **Near enough to need my inputs** -- or not heard from yet --
            // hears them every frame, with where I am. **Too far** hears
            // where I am once a heartbeat, with a promise it holds: it
            // cannot share a region with me before the promise runs out.
            let far = far(&self.grid, me, &self.known[*seat].declared);
            if !far {
                line.send(&packet);
            } else if newest >= self.beat[*seat] + HEARTBEAT {
                self.beat[*seat] = newest;
                line.send(&beat);
            }
        }

        // Rollback: anything changed behind us is played again.
        self.take_outcomes();
        if let Some(from) = self.dirty.take() {
            self.replay_from(from);
        }
        self.hold_books();

        // Advance, inside the window.
        if next.saturating_sub(self.confirmed) > WINDOW {
            self.stalls += 1;
            self.confirm();
            return false;
        }
        let needed = self.needed_at(next);
        let inputs: [Input; MAX_PLAYERS] =
            std::array::from_fn(|s| self.input_for(next, s, &needed));
        let w = self.world.as_mut().expect("checked above");
        self.tape.record(w, inputs);
        w.advance(inputs);
        self.confirm();
        self.send_outcomes();
        true
    }

    /// The newest frame every needed input is known for.
    fn confirm(&mut self) {
        let Some(w) = &self.world else { return };
        let mut f = self.confirmed;
        while f < w.frame {
            let next = f + 1;
            // Settled when everybody who has ever spoken has declared for
            // this frame or a later one -- so who was a member is known --
            // and every member's input is here.
            let settled = self
                .known
                .iter()
                .enumerate()
                .all(|(s, k)| s == self.me || k.newest().is_none_or(|n| n >= next));
            if !settled {
                break;
            }
            let needed = self.needed_at(next);
            let all = (0..MAX_PLAYERS)
                .all(|s| !needed[s] || s == self.me || self.known[s].input_at(next).is_some());
            if !all {
                break;
            }
            f = next;
        }
        self.confirmed = f;
    }

    /// Play again from `from` with what is known now.
    fn replay_from(&mut self, from: u32) {
        let Some(w) = &self.world else { return };
        let now = w.frame;
        let Some(start) = self.tape.get(from).map(|e| e.before.clone()) else {
            return;
        };
        let mut world = start;
        let mut f = from;
        while f <= now {
            let needed = self.needed_at(f);
            let inputs: [Input; MAX_PLAYERS] =
                std::array::from_fn(|s| self.input_for(f, s, &needed));
            self.tape.record(&world, inputs);
            world.advance(inputs);
            f += 1;
        }
        self.world = Some(world);
        self.rollbacks += 1;
    }

    /// Put the heard outcomes into the tape and mark the replay.
    fn take_outcomes(&mut self) {
        let Some(w) = &self.world else { return };
        let now = w.frame;
        let outcomes = std::mem::take(&mut self.outcomes);
        for (region, frame, fighters) in outcomes {
            if frame > now || frame + (TAPE as u32) <= now + 1 {
                continue;
            }
            self.outcomes_taken += 1;
            self.settled.push((region, frame));
            if self.settled.len() > 4 * TAPE {
                self.settled.remove(0);
            }
            // The world at `frame` is the tape's "before" of `frame + 1`, or
            // the world itself.
            let mut changed = false;
            if frame == now {
                let w = self.world.as_mut().expect("some");
                for (seat, p) in &fighters {
                    if w.players[*seat as usize] != *p {
                        w.players[*seat as usize] = *p;
                        changed = true;
                    }
                }
                if changed {
                    self.rollbacks += 1;
                }
                continue;
            }
            let Some(entry) = self.tape.get(frame + 1) else {
                continue;
            };
            let mut before = entry.before.clone();
            for (seat, p) in &fighters {
                if before.players[*seat as usize] != *p {
                    before.players[*seat as usize] = *p;
                    changed = true;
                }
            }
            if changed {
                let inputs = entry.inputs;
                self.tape.record(&before, inputs);
                self.dirty = Some(self.dirty.map_or(frame + 1, |d| d.min(frame + 1)));
            }
        }
    }

    /// Whether a history had `region` in it `SETTLE` frames before `frame`:
    /// a member long enough to be exact there.
    fn old_member(&self, history: &[(u32, Vec<RegionId>)], region: RegionId, frame: u32) -> bool {
        declared_at(history, frame.saturating_sub(SETTLE)).contains(&region)
    }

    /// The zones of the regions I am an old member of, at my newest
    /// confirmed frame, to the peers next door that are not old members.
    fn send_outcomes(&mut self) {
        if self.confirmed < self.last_outcome_at + OUTCOME_EVERY {
            return;
        }
        let f = self.confirmed;
        let Some(w) = self.world_at(f) else { return };
        self.last_outcome_at = f;
        let origin = w.map_origin();
        let mine: Vec<RegionId> = declared_at(&self.declared, f).to_vec();
        self.send_books(&w, &mine);
        for region in mine {
            if !self.old_member(&self.declared, region, f) {
                continue;
            }
            let fighters: Vec<(u8, Player)> = w
                .players
                .iter()
                .enumerate()
                .filter(|(i, p)| w.is_present(*i) && self.grid.in_zone(region, p.pos.add(origin)))
                .map(|(i, p)| (i as u8, *p))
                .collect();
            if fighters.is_empty() {
                continue;
            }
            let packet = Packet::Outcome {
                region,
                frame: f,
                fighters: fighters.clone(),
            }
            .encode();
            let neighbours = region.neighbours();
            let mut sent = 0;
            // As far from a peer's regions as its strip reaches: past that
            // nothing in the outcome is anything it simulates.
            let strip = self.grid.membership().add(self.grid.horizon());
            let grid = self.grid;
            for (seat, line) in self.lines.iter_mut() {
                let k = &self.known[*seat];
                let theirs = k.declared.last().map_or(&[][..], |(_, r)| r.as_slice());
                let next_door = theirs
                    .iter()
                    .any(|r| *r == region || neighbours.contains(r))
                    && fighters.iter().any(|(_, p)| {
                        let at = p.pos.add(origin);
                        theirs.iter().any(|r| grid.gap(*r, at).raw() <= strip.raw())
                    });
                let settled = declared_at(&k.declared, f.saturating_sub(SETTLE)).contains(&region);
                if next_door && !settled {
                    sent += 1;
                    line.send(&packet);
                }
            }
            self.outcomes_sent += sent;
        }
    }

    /// My books at a confirmed frame -- the checksums of the regions I am
    /// an old member of, exact here -- to every peer that declared one of
    /// them or a neighbour of one. Nothing further off is ever summed up.
    fn send_books(&mut self, w: &World, mine: &[RegionId]) {
        let f = w.frame;
        let books: Vec<(RegionId, u64)> = mine
            .iter()
            .filter(|r| self.old_member(&self.declared, **r, f))
            .map(|r| (*r, w.region_checksum(&self.grid, *r)))
            .collect();
        if books.is_empty() {
            return;
        }
        let packet = Packet::Books {
            seat: self.me as u8,
            frame: f,
            books,
        }
        .encode();
        let near: Vec<RegionId> = mine
            .iter()
            .flat_map(|r| std::iter::once(*r).chain(r.neighbours()))
            .collect();
        for (seat, line) in self.lines.iter_mut() {
            let theirs = self.known[*seat]
                .declared
                .last()
                .map_or(&[][..], |(_, r)| r.as_slice());
            if theirs.iter().any(|r| near.contains(r)) {
                line.send(&packet);
            }
        }
    }

    /// Hold the books heard against my own, where mine are exact: a frame
    /// I have confirmed (a frame still predicted is nobody's books), and in
    /// it a region I am an old member of, or a neighbour's whose outcome
    /// for that frame I took. Anything else is not mine to judge, and is
    /// dropped; a frame not yet confirmed waits.
    fn hold_books(&mut self) {
        let Some(w) = &self.world else { return };
        let now = w.frame;
        let books = std::mem::take(&mut self.books);
        for (seat, frame, theirs) in books {
            if frame > self.confirmed {
                if frame + (TAPE as u32) > now + 2 {
                    self.books.push((seat, frame, theirs));
                }
                continue;
            }
            let Some(world) = self.world_at(frame) else {
                continue;
            };
            for (region, sum) in theirs {
                let exact = self.old_member(&self.declared, region, frame)
                    || self.settled.contains(&(region, frame));
                if !exact {
                    continue;
                }
                if world.region_checksum(&self.grid, region) == sum {
                    self.agreed += 1;
                } else {
                    self.disagreed += 1;
                    if self.first_disagreement.is_none() {
                        self.first_disagreement = Some((seat, region, frame));
                    }
                }
            }
        }
    }

    pub fn world_at(&self, frame: u32) -> Option<World> {
        let w = self.world.as_ref()?;
        if frame == w.frame {
            Some(w.clone())
        } else {
            self.tape.get(frame + 1).map(|e| e.before.clone())
        }
    }

    /// A region's checksum at `frame`, off the tape.
    pub fn zone_checksum(&self, region: RegionId, frame: u32) -> Option<u64> {
        self.world_at(frame)
            .map(|w| w.region_checksum(&self.grid, region))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_packet_reads_back_as_written() {
        let p = Packet::Inputs {
            seat: 2,
            frame: 1000,
            inputs: vec![
                Input::aimed(Input::W, 300),
                Input::looking_at(5, 6, -7).travelling(Travel(0x123)),
            ],
            regions: vec![RegionId::new(-1, 2), RegionId::new(0, 0)],
            until: 1012,
        };
        assert_eq!(Packet::decode(&p.encode()), Some(p));
        let w = World::with_classes([sim::class::Class::Bulwark; 2]);
        let o = Packet::Outcome {
            region: RegionId::new(1, 1),
            frame: 7,
            fighters: vec![(0, w.players[0]), (1, w.players[1])],
        };
        assert_eq!(Packet::decode(&o.encode()), Some(o));
        let s = Packet::Snapshot {
            frame: 9,
            index: 1,
            total: 3,
            bytes: vec![1, 2, 3],
        };
        assert_eq!(Packet::decode(&s.encode()), Some(s));
        let want = Packet::Want {
            seat: 3,
            frame: 77,
            missing: vec![0, 5, 18],
        };
        assert_eq!(Packet::decode(&want.encode()), Some(want));
        let books = Packet::Books {
            seat: 1,
            frame: 500,
            books: vec![(RegionId::new(2, -3), 0xdead_beef_0bad_f00d)],
        };
        assert_eq!(Packet::decode(&books.encode()), Some(books));
    }
}
