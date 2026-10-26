//! **Groups on the wire** (exploration 0029 §12–13, G2).
//!
//! A peer is in one **group**: the players near enough to touch, sharing one
//! world (`sim::group`). The group plays in lockstep with rollback among its
//! members, exactly as a match does -- every member's input every frame, a
//! wrong guess corrected by playing again. Everybody else hears from me only
//! a **heartbeat** ([`Packet::Beat`]): where my fighter is on the map, whose
//! group I am in, whether it is hunting. A fight far off is nobody else's to
//! wait on, send to or roll back for.
//!
//! **A split** is decided by the world itself: every [`SPLIT_EVERY`] frames,
//! at a frame every member has confirmed, the members' fighters are cut into
//! groups by [`SPLIT_AT`]; whoever is not in mine is fed `Travel::LEFT` from
//! the next frame on. Every member cuts the same world the same way, so no
//! word is needed. A group in a hunt does not split.
//!
//! **A merge** is agreed on the wire, when two groups' fighters come within
//! [`MERGE_AT`]. The base is the group in a hunt, else the one whose leader
//! (its lowest id) is lower; its leader calls the other's ([`Packet::Call`]).
//! That leader names a frame in its own inputs ([`Control::hand_over`]); at
//! it, its members stop, and it sends its world there to the base group's
//! members ([`Packet::Chunk`]). The base leader, once it has all of it, names
//! a frame of the base's ([`Control::absorb`]); at it, every base member
//! makes the same merged world (`sim::group::merge`) and the leader sends
//! it to the newcomers, who play on as members from there.

use crate::meet::Line;
use crate::raw;
use regions::{TAPE, Tape};
use sim::group;
use sim::input::{Input, Travel, Tune};
use sim::state::MAX_PLAYERS;
use sim::{Fx, V3, World};
use std::collections::BTreeMap;

/// How many frames of my inputs every packet repeats.
pub const REDUNDANCY: usize = 8;
/// How far past the confirmed frame a member runs before it waits.
pub const WINDOW: u32 = 8;
/// How often a peer outside my group hears where I am, in frames.
pub const HEARTBEAT: u32 = 6;
/// Two groups' fighters this near, in metres, are one group.
pub const MERGE_AT: i32 = 120;
/// A group's fighters this far from the rest, in metres, are a group of
/// their own: further than [`MERGE_AT`], so a pair on the line between does
/// not merge and split on alternate seconds.
pub const SPLIT_AT: i32 = 200;
/// How often a group asks itself whether it has come apart, in frames.
pub const SPLIT_EVERY: u32 = 60;
/// A snapshot chunk's payload.
const CHUNK: usize = 1024;
/// How often a hand-over or a merged world is sent again until it is taken.
const RESEND: u32 = 10;
/// How many frames ahead of the newest a leader names a frame: past what
/// any member can have confirmed, so every member stops on it.
const AHEAD: u32 = WINDOW + 2;

/// What a leader says in its inputs beside them: the group's decisions,
/// which every member reads at the frame they confirm.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Control {
    /// This group stops at this frame and hands its world to the group whose
    /// leader is `to`.
    pub hand_over: Option<(u32, u16)>,
    /// This group takes in the world handed over by the group of leader
    /// `from`, at this frame.
    pub absorb: Option<(u32, u16)>,
}

/// What goes on a line, by its first byte.
#[derive(Clone, Debug, PartialEq)]
pub enum Packet {
    /// My inputs for the frames ending at `frame`, newest last, to my group.
    Inputs {
        from: u16,
        frame: u32,
        inputs: Vec<Input>,
        control: Control,
    },
    /// Where I am, to everybody outside my group: my fighter on the map, in
    /// centimetres, my group's leader and how many it is, and whether it is
    /// in a hunt.
    Beat {
        from: u16,
        leader: u16,
        at: [i32; 3],
        members: u8,
        hunting: bool,
    },
    /// The base leader to the other group's: hand me your world.
    Call { from: u16 },
    /// A world, in `total` chunks: the hand-over (`merged` false, at the
    /// handing group's frame) or the merged world (`merged` true).
    Chunk {
        merged: bool,
        frame: u32,
        index: u16,
        total: u16,
        bytes: Vec<u8>,
    },
}

const K_INPUTS: u8 = 11;
const K_BEAT: u8 = 12;
const K_CALL: u8 = 13;
const K_CHUNK: u8 = 14;

fn put_input(b: &mut Vec<u8>, i: &Input) {
    b.extend_from_slice(&i.bits.to_le_bytes());
    b.extend_from_slice(&i.aim.to_le_bytes());
    b.extend_from_slice(&i.pitch.to_le_bytes());
    b.extend_from_slice(&i.travel.0.to_le_bytes());
    b.extend_from_slice(&i.tune.cell.to_le_bytes());
    if i.tune.cell != 0 {
        b.extend_from_slice(&i.tune.raw.to_le_bytes());
    }
}

impl Packet {
    pub fn encode(&self) -> Vec<u8> {
        let mut b = Vec::new();
        match self {
            Packet::Inputs {
                from,
                frame,
                inputs,
                control,
            } => {
                b.push(K_INPUTS);
                b.extend_from_slice(&from.to_le_bytes());
                b.extend_from_slice(&frame.to_le_bytes());
                let put = |b: &mut Vec<u8>, c: Option<(u32, u16)>| {
                    let (f, who) = c.unwrap_or((0, 0));
                    b.extend_from_slice(&f.to_le_bytes());
                    b.extend_from_slice(&who.to_le_bytes());
                };
                put(&mut b, control.hand_over);
                put(&mut b, control.absorb);
                b.push(inputs.len() as u8);
                for i in inputs {
                    put_input(&mut b, i);
                }
            }
            Packet::Beat {
                from,
                leader,
                at,
                members,
                hunting,
            } => {
                b.push(K_BEAT);
                b.extend_from_slice(&from.to_le_bytes());
                b.extend_from_slice(&leader.to_le_bytes());
                for c in at {
                    b.extend_from_slice(&c.to_le_bytes());
                }
                b.push(*members);
                b.push(*hunting as u8);
            }
            Packet::Call { from } => {
                b.push(K_CALL);
                b.extend_from_slice(&from.to_le_bytes());
            }
            Packet::Chunk {
                merged,
                frame,
                index,
                total,
                bytes,
            } => {
                b.push(K_CHUNK);
                b.push(*merged as u8);
                b.extend_from_slice(&frame.to_le_bytes());
                b.extend_from_slice(&index.to_le_bytes());
                b.extend_from_slice(&total.to_le_bytes());
                b.extend_from_slice(bytes);
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
        let u16_ = |s: &[u8]| u16::from_le_bytes([s[0], s[1]]);
        let u32_ = |s: &[u8]| u32::from_le_bytes([s[0], s[1], s[2], s[3]]);
        match take(1)?[0] {
            K_INPUTS => {
                let from = u16_(take(2)?);
                let frame = u32_(take(4)?);
                let mut control = Control::default();
                for k in 0..2 {
                    let f = u32_(take(4)?);
                    let who = u16_(take(2)?);
                    let c = (f != 0).then_some((f, who));
                    if k == 0 {
                        control.hand_over = c;
                    } else {
                        control.absorb = c;
                    }
                }
                let n = take(1)?[0] as usize;
                let mut inputs = Vec::with_capacity(n);
                for _ in 0..n {
                    let bits = u16_(take(2)?);
                    let aim = u16_(take(2)?);
                    let pitch = u16_(take(2)?) as i16;
                    let travel = u16_(take(2)?);
                    let cell = u16_(take(2)?);
                    let raw = if cell != 0 { u32_(take(4)?) as i32 } else { 0 };
                    inputs.push(
                        Input::looking_at(bits, aim, pitch)
                            .travelling(Travel(travel))
                            .tuning(Tune { cell, raw }),
                    );
                }
                Some(Packet::Inputs {
                    from,
                    frame,
                    inputs,
                    control,
                })
            }
            K_BEAT => {
                let from = u16_(take(2)?);
                let leader = u16_(take(2)?);
                let mut pos = [0i32; 3];
                for c in pos.iter_mut() {
                    *c = u32_(take(4)?) as i32;
                }
                let members = take(1)?[0];
                let hunting = take(1)?[0] != 0;
                Some(Packet::Beat {
                    from,
                    leader,
                    at: pos,
                    members,
                    hunting,
                })
            }
            K_CALL => Some(Packet::Call {
                from: u16_(take(2)?),
            }),
            K_CHUNK => {
                let merged = take(1)?[0] != 0;
                let frame = u32_(take(4)?);
                let index = u16_(take(2)?);
                let total = u16_(take(2)?);
                Some(Packet::Chunk {
                    merged,
                    frame,
                    index,
                    total,
                    bytes: b[at..].to_vec(),
                })
            }
            _ => None,
        }
    }
}

/// What a peer knows of another member: inputs by frame, the newest, and
/// the decisions its packets carried.
#[derive(Clone)]
struct Known {
    inputs: Box<[Option<(u32, Input)>; TAPE]>,
    last: Option<(u32, Input)>,
    newest: u32,
    control: Control,
}

impl Known {
    fn new() -> Known {
        Known {
            inputs: Box::new([None; TAPE]),
            last: None,
            newest: 0,
            control: Control::default(),
        }
    }

    fn input_at(&self, f: u32) -> Option<Input> {
        self.inputs[f as usize % TAPE]
            .filter(|(g, _)| *g == f)
            .map(|(_, i)| i)
    }
}

/// The last heartbeat from a peer outside my group.
#[derive(Clone, Copy, Debug)]
pub struct Heard {
    pub leader: u16,
    pub at: V3,
    pub members: u8,
    pub hunting: bool,
}

/// A world on its way in, in chunks.
#[derive(Clone, Default)]
struct Incoming {
    frame: u32,
    parts: Vec<Option<Vec<u8>>>,
}

impl Incoming {
    fn take(&mut self, frame: u32, index: u16, total: u16, bytes: Vec<u8>) -> Option<World> {
        if self.parts.len() != total as usize || self.frame != frame {
            *self = Incoming {
                frame,
                parts: vec![None; total as usize],
            };
        }
        if let Some(p) = self.parts.get_mut(index as usize) {
            *p = Some(bytes);
        }
        if self.parts.iter().all(Option::is_some) {
            let all: Vec<u8> = self.parts.iter().flatten().flatten().copied().collect();
            return raw::world_from(&all);
        }
        None
    }
}

/// Where a peer's group is in a merge.
#[derive(Clone, Debug, PartialEq)]
enum Merging {
    Idle,
    /// The base leader has called the other group's leader.
    Calling {
        to: u16,
        since: u32,
    },
    /// My group stops at `at` and hands its world to `base`'s group.
    Handing {
        at: u32,
        base: u16,
    },
    /// My group stopped; I wait for the merged world.
    Waiting {
        base: u16,
        sent: u32,
    },
}

/// **One peer of a world of groups**, on real lines.
pub struct Session<L: Line> {
    pub me: u16,
    lines: Vec<(u16, L)>,
    /// My group's world.
    pub world: World,
    tape: Box<Tape>,
    mine: Box<[Option<(u32, Input)>; TAPE]>,
    known: BTreeMap<u16, Known>,
    /// Members gone from my group (a split), and the frame from which.
    gone: Vec<(u16, u32)>,
    /// The newest frame every member's input is known for.
    pub confirmed: u32,
    dirty: Option<u32>,
    /// The last heartbeat from every peer outside my group.
    pub heard: BTreeMap<u16, Heard>,
    merging: Merging,
    /// My decisions, as a leader, carried in every packet of inputs.
    control: Control,
    /// The other group's world as handed over, on its way in or whole.
    handed: Incoming,
    handed_world: Option<(u32, World)>,
    merged_in: Incoming,
    /// The merged world I made as a base leader, to send to the newcomers.
    merged_out: Option<(World, Vec<u16>, u32)>,
    /// Ticks played, frame made or not: what resending is timed by while
    /// the world stands still.
    ticks: u32,
    pub rollbacks: u32,
    pub stalls: u32,
    pub merges: u32,
    pub splits: u32,
}

impl<L: Line> std::fmt::Debug for Session<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "peer {} frame {} confirmed {} merging {:?} control {:?} handed {}",
            self.me,
            self.world.frame,
            self.confirmed,
            self.merging,
            self.control,
            self.handed_world.is_some()
        )
    }
}

/// Map centimetres of a position.
fn cm(v: V3) -> [i32; 3] {
    let c = |f: Fx| (f.raw() as i64 * 100 / Fx::ONE.raw() as i64) as i32;
    [c(v.x), c(v.y), c(v.z)]
}

fn from_cm(c: [i32; 3]) -> V3 {
    let m = |c: i32| Fx::from_raw((c as i64 * Fx::ONE.raw() as i64 / 100) as i32);
    V3::new(m(c[0]), m(c[1]), m(c[2]))
}

/// Flat distance squared, in whole metres, between two map points.
fn apart(a: V3, b: V3) -> i64 {
    let dx = (a.x.raw() - b.x.raw()) as i64 >> 16;
    let dz = (a.z.raw() - b.z.raw()) as i64 >> 16;
    dx * dx + dz * dz
}

impl<L: Line> Session<L> {
    /// A peer with its group's world from the start. `lines` reach every
    /// other peer, by id.
    pub fn new(me: u16, world: World, lines: Vec<(u16, L)>) -> Session<L> {
        Session {
            me,
            lines,
            confirmed: world.frame,
            world,
            tape: Box::new(Tape::new()),
            mine: Box::new([None; TAPE]),
            known: BTreeMap::new(),
            gone: Vec::new(),
            dirty: None,
            heard: BTreeMap::new(),
            merging: Merging::Idle,
            control: Control::default(),
            handed: Incoming::default(),
            handed_world: None,
            merged_in: Incoming::default(),
            merged_out: None,
            ticks: 0,
            rollbacks: 0,
            stalls: 0,
            merges: 0,
            splits: 0,
        }
    }

    /// The slot of a player in a world, by id.
    fn slot(w: &World, id: u16) -> Option<usize> {
        (0..MAX_PLAYERS).find(|&k| w.ids[k] == id)
    }

    /// **My group at frame `f`**: every id in the world in the fight, but
    /// those gone by then.
    pub fn members_at(&self, w: &World, f: u32) -> Vec<u16> {
        (0..MAX_PLAYERS)
            .filter(|&k| w.ids[k] != 0 && !w.is_dummy(k) && w.is_present(k))
            .map(|k| w.ids[k])
            .filter(|id| !self.gone.iter().any(|(g, from)| g == id && *from <= f))
            .collect::<std::collections::BTreeSet<u16>>()
            .into_iter()
            .collect()
    }

    /// My group now.
    pub fn members(&self) -> Vec<u16> {
        self.members_at(&self.world, self.world.frame + 1)
    }

    /// My group's leader: its lowest id.
    pub fn leader(&self) -> u16 {
        self.members().into_iter().min().unwrap_or(self.me)
    }

    fn send_to(&mut self, to: u16, p: &[u8]) {
        if let Some((_, line)) = self.lines.iter_mut().find(|(id, _)| *id == to) {
            line.send(p);
        }
    }

    /// **Hear everything on every line.**
    pub fn receive(&mut self) {
        let mut heard = Vec::new();
        for (from, line) in self.lines.iter_mut() {
            while let Some(b) = line.recv() {
                if let Some(p) = Packet::decode(&b) {
                    heard.push((*from, p));
                }
            }
        }
        for (_, p) in heard {
            match p {
                Packet::Inputs {
                    from,
                    frame,
                    inputs,
                    control,
                } => {
                    let now = self.world.frame;
                    let k = self.known.entry(from).or_insert_with(Known::new);
                    k.control = control;
                    k.newest = k.newest.max(frame);
                    let first = (frame + 1).saturating_sub(inputs.len() as u32);
                    for (n, input) in inputs.into_iter().enumerate() {
                        let f = first + n as u32;
                        let slot = &mut k.inputs[f as usize % TAPE];
                        let news = !matches!(slot, Some((g, i)) if *g == f && *i == input);
                        *slot = Some((f, input));
                        if k.last.is_none_or(|(g, _)| f > g) {
                            k.last = Some((f, input));
                        }
                        if news
                            && f <= now
                            && self.tape.get(f).is_some_and(|e| {
                                Self::slot(&e.before, from).is_some_and(|s| e.inputs[s] != input)
                            })
                        {
                            self.dirty = Some(self.dirty.map_or(f, |d| d.min(f)));
                        }
                    }
                }
                Packet::Beat {
                    from,
                    leader,
                    at,
                    members,
                    hunting,
                } => {
                    self.heard.insert(
                        from,
                        Heard {
                            leader,
                            at: from_cm(at),
                            members,
                            hunting,
                        },
                    );
                }
                Packet::Call { from } => {
                    // The other leader wants my world: name a frame past any
                    // member's confirmed one, if I lead and am free.
                    if self.leader() == self.me && self.merging == Merging::Idle {
                        let at = self.world.frame + AHEAD;
                        self.control.hand_over = Some((at, from));
                        self.merging = Merging::Handing { at, base: from };
                    }
                }
                Packet::Chunk {
                    merged,
                    frame,
                    index,
                    total,
                    bytes,
                } => {
                    if merged {
                        if let Merging::Waiting { .. } = self.merging
                            && let Some(w) = self.merged_in.take(frame, index, total, bytes)
                        {
                            self.take_merged(w);
                        }
                    } else if self.handed_world.is_none()
                        && let Some(w) = self.handed.take(frame, index, total, bytes)
                    {
                        self.handed_world = Some((frame, w));
                    }
                }
            }
        }
    }

    /// The merged world arrived: it is mine, and so is its group.
    fn take_merged(&mut self, w: World) {
        self.world = w;
        self.confirmed = self.world.frame;
        *self.tape = Tape::new();
        // Frames are the base's from here: what was heard and pressed by
        // the old count would read as the new.
        *self.mine = [None; TAPE];
        self.known.clear();
        self.gone.clear();
        self.dirty = None;
        self.merging = Merging::Idle;
        self.control = Control::default();
        self.merges += 1;
    }

    /// The input frame `f` plays for the player in slot `k` of `w`.
    fn input_for(&self, w: &World, f: u32, k: usize) -> Input {
        let id = w.ids[k];
        if id == 0 || w.is_dummy(k) {
            return Input::default();
        }
        if self.gone.iter().any(|(g, from)| *g == id && *from <= f) {
            return Input::default().travelling(Travel::LEFT);
        }
        if id == self.me {
            return self.mine[f as usize % TAPE]
                .filter(|(g, _)| *g == f)
                .map_or(Input::default(), |(_, i)| i);
        }
        match self.known.get(&id) {
            Some(k) => k.input_at(f).or(k.last.map(|(_, i)| i)).unwrap_or_default(),
            None => Input::default(),
        }
    }

    fn inputs_for(&self, w: &World, f: u32) -> [Input; MAX_PLAYERS] {
        std::array::from_fn(|k| self.input_for(w, f, k))
    }

    /// Play again from `from` with what is known now.
    fn replay_from(&mut self, from: u32) {
        let now = self.world.frame;
        let Some(start) = self.tape.get(from).map(|e| e.before.clone()) else {
            return;
        };
        let mut w = start;
        let mut f = from;
        while f <= now {
            let inputs = self.inputs_for(&w, f);
            self.tape.record(&w, inputs);
            w.advance(inputs);
            f += 1;
        }
        self.world = w;
        self.rollbacks += 1;
    }

    /// The world at frame `f`, off the tape or in hand.
    pub fn world_at(&self, f: u32) -> Option<World> {
        if f == self.world.frame {
            Some(self.world.clone())
        } else {
            self.tape.get(f + 1).map(|e| e.before.clone())
        }
    }

    /// **One frame**, given my input. Returns whether the world advanced.
    pub fn tick(&mut self, input: Input) -> bool {
        self.ticks += 1;
        self.receive();
        if let Merging::Waiting { base, sent } = self.merging {
            // Waiting for the merged world: the hand-over again, now and
            // then, to whoever is behind the base, if it was mine to hand.
            if self.leader() == self.me && self.ticks >= sent + RESEND {
                self.hand_over(base);
            }
            return false;
        }
        let next = self.world.frame + 1;
        self.mine[next as usize % TAPE] = Some((next, input));
        let members = self.members();
        // Inputs to my group, with what I have decided; where I am to
        // everybody else, a heartbeat.
        let first = (next + 1).saturating_sub(REDUNDANCY as u32).max(1);
        let inputs: Vec<Input> = (first..=next)
            .map(|f| {
                self.mine[f as usize % TAPE]
                    .filter(|(g, _)| *g == f)
                    .map_or(Input::default(), |(_, i)| i)
            })
            .collect();
        let packet = Packet::Inputs {
            from: self.me,
            frame: next,
            inputs,
            control: self.control,
        }
        .encode();
        let at = Self::slot(&self.world, self.me).map_or(V3::ZERO, |k| {
            self.world.players[k].pos.add(self.world.map_origin())
        });
        let beat = (next % HEARTBEAT == 0).then(|| {
            Packet::Beat {
                from: self.me,
                leader: self.leader(),
                at: cm(at),
                members: members.len() as u8,
                hunting: self.world.hunting(),
            }
            .encode()
        });
        let ids: Vec<u16> = self.lines.iter().map(|(id, _)| *id).collect();
        for id in ids {
            if members.contains(&id) {
                self.send_to(id, &packet);
            } else if let Some(b) = &beat {
                self.send_to(id, b);
            }
        }
        self.lead();
        // A merged world I made goes to the newcomers again now and then,
        // whoever leads now, until they play.
        if self.merged_out.is_some() && self.ticks % RESEND == 0 {
            self.send_merged();
        }
        if let Some(from) = self.dirty.take() {
            self.replay_from(from);
        }
        if next.saturating_sub(self.confirmed) > WINDOW {
            self.stalls += 1;
            self.confirm();
            return false;
        }
        let inputs = self.inputs_for(&self.world, next);
        let w = &mut self.world;
        self.tape.record(w, inputs);
        w.advance(inputs);
        self.confirm();
        true
    }

    /// The newest frame every member's input is known for, and what the
    /// group decides at the frames it reaches.
    fn confirm(&mut self) {
        let mut f = self.confirmed;
        while f < self.world.frame {
            let next = f + 1;
            let Some(w) = self.world_at(f) else { break };
            let members = self.members_at(&w, next);
            let all = members.iter().all(|id| {
                *id == self.me
                    || self
                        .known
                        .get(id)
                        .is_some_and(|k| k.input_at(next).is_some())
            });
            // An absorb named for this frame waits for the world it takes.
            let held = self
                .control_of_leader()
                .absorb
                .is_some_and(|(at, _)| at == next && self.handed_world.is_none());
            if !all || held {
                break;
            }
            f = next;
            self.confirmed = f;
            if self.decide(f) {
                return;
            }
        }
    }

    /// **What the group decides at a confirmed frame `f`**: a hand-over or
    /// an absorb its leader named for it, or a split. True when the world
    /// was replaced or stopped, and confirming goes no further this tick.
    /// What my group's leader has decided, as its packets carry it.
    fn control_of_leader(&self) -> Control {
        let leader = self.leader();
        if leader == self.me {
            self.control
        } else {
            self.known
                .get(&leader)
                .map(|k| k.control)
                .unwrap_or_default()
        }
    }

    fn decide(&mut self, f: u32) -> bool {
        let leader = self.leader();
        let control = self.control_of_leader();
        if let Some((at, base)) = control.hand_over
            && at == f
        {
            // My group stops here; its leader hands the world over.
            if let Some(w) = self.world_at(f) {
                self.world = w;
            }
            self.merging = Merging::Waiting {
                base,
                sent: self.ticks,
            };
            if leader == self.me {
                self.hand_over(base);
            }
            return true;
        }
        if let Some((at, _)) = control.absorb
            && at == f
            && let Some((_, theirs)) = self.handed_world.clone()
            && let Some(ours) = self.world_at(f)
        {
            if let Ok(m) = group::merge(&ours, &theirs) {
                let newcomers: Vec<u16> = (0..MAX_PLAYERS)
                    .filter(|&k| m.from_b[k].is_some())
                    .map(|k| theirs.ids[k])
                    .collect();
                // Play on from the merged world: what was played past `f`
                // is played again with it.
                let now = self.world.frame;
                self.world = m.world.clone();
                while self.world.frame < now {
                    let next = self.world.frame + 1;
                    let inputs = self.inputs_for(&self.world, next);
                    self.tape.record(&self.world, inputs);
                    self.world.advance(inputs);
                }
                if leader == self.me {
                    self.merged_out = Some((m.world, newcomers, f));
                    self.send_merged();
                }
                // The decision stays in my packets: a member short of `f`
                // reads it when it gets there.
                self.handed_world = None;
                self.merging = Merging::Idle;
                self.merges += 1;
            }
            return true;
        }
        if f % SPLIT_EVERY == 0
            && let Some(w) = self.world_at(f)
            && !w.hunting()
        {
            let members = self.members_at(&w, f + 1);
            let origin = w.map_origin();
            let pos = |id: u16| Self::slot(&w, id).map(|k| w.players[k].pos.add(origin));
            // My component: everybody joined to me by steps under SPLIT_AT.
            let far = (SPLIT_AT as i64) * (SPLIT_AT as i64);
            let mut mine = vec![self.me];
            let mut grew = true;
            while grew {
                grew = false;
                for id in &members {
                    if mine.contains(id) {
                        continue;
                    }
                    let near = mine.iter().any(|m| match (pos(*m), pos(*id)) {
                        (Some(a), Some(b)) => apart(a, b) < far,
                        _ => false,
                    });
                    if near {
                        mine.push(*id);
                        grew = true;
                    }
                }
            }
            let leaving: Vec<u16> = members
                .iter()
                .filter(|id| !mine.contains(id))
                .copied()
                .collect();
            if !leaving.is_empty() {
                for id in leaving {
                    self.gone.push((id, f + 1));
                }
                self.splits += 1;
                if f < self.world.frame {
                    self.dirty = Some(self.dirty.map_or(f + 1, |d| d.min(f + 1)));
                }
            }
        }
        false
    }

    /// **What a leader does each frame**: call a group near enough to
    /// merge with, as the base; absorb a world handed over once it is all
    /// here; send a merged world to newcomers until they play.
    fn lead(&mut self) {
        if self.leader() != self.me {
            return;
        }
        let now = self.world.frame;
        // A hand-over in hand: name the frame to take it in at, unless one
        // is named and still to come.
        if !self
            .control
            .absorb
            .is_some_and(|(at, _)| at > self.confirmed)
            && let Some((_, theirs)) = &self.handed_world
        {
            let from = theirs
                .ids
                .iter()
                .copied()
                .filter(|&i| i != 0)
                .min()
                .unwrap_or(0);
            self.control.absorb = Some((now + AHEAD, from));
        }
        if self.merging != Merging::Idle {
            if let Merging::Calling { since, .. } = self.merging
                && now > since + 120
            {
                self.merging = Merging::Idle;
            }
            return;
        }
        let members = self.members();
        let origin = self.world.map_origin();
        let mine: Vec<V3> = members
            .iter()
            .filter_map(|id| Self::slot(&self.world, *id))
            .map(|k| self.world.players[k].pos.add(origin))
            .collect();
        let near = (MERGE_AT as i64) * (MERGE_AT as i64);
        let hunting = self.world.hunting();
        let call = self.heard.iter().find(|(id, h)| {
            !members.contains(id)
                && h.leader != self.me
                && !(hunting && h.hunting)
                // The base calls: the group in a hunt, else the lower leader.
                && (hunting || (!h.hunting && self.me < h.leader))
                && mine.iter().any(|m| apart(*m, h.at) < near)
        });
        if let Some((_, h)) = call {
            let to = h.leader;
            self.merging = Merging::Calling { to, since: now };
            let p = Packet::Call { from: self.me }.encode();
            self.send_to(to, &p);
        }
    }

    /// Hand my group's world over to `base`'s group: in chunks, to every
    /// peer whose heartbeat says it is in that group.
    fn hand_over(&mut self, base: u16) {
        let bytes = raw::world_bytes(&self.world);
        let total = bytes.len().div_ceil(CHUNK) as u16;
        let to: Vec<u16> = self
            .heard
            .iter()
            .filter(|(_, h)| h.leader == base)
            .map(|(id, _)| *id)
            .collect();
        for (index, part) in bytes.chunks(CHUNK).enumerate() {
            let p = Packet::Chunk {
                merged: false,
                frame: self.world.frame,
                index: index as u16,
                total,
                bytes: part.to_vec(),
            }
            .encode();
            for id in &to {
                self.send_to(*id, &p);
            }
        }
        if let Merging::Waiting { sent, .. } = &mut self.merging {
            *sent = self.ticks;
        }
    }

    /// The merged world to the newcomers.
    fn send_merged(&mut self) {
        let Some((w, to, _)) = self.merged_out.clone() else {
            return;
        };
        let bytes = raw::world_bytes(&w);
        let total = bytes.len().div_ceil(CHUNK) as u16;
        for (index, part) in bytes.chunks(CHUNK).enumerate() {
            let p = Packet::Chunk {
                merged: true,
                frame: w.frame,
                index: index as u16,
                total,
                bytes: part.to_vec(),
            }
            .encode();
            for id in &to {
                self.send_to(*id, &p);
            }
        }
        // Once the newcomers play, their inputs are what says so.
        if to
            .iter()
            .all(|id| self.known.get(id).is_some_and(|k| k.newest > w.frame))
        {
            self.merged_out = None;
        }
    }
}
