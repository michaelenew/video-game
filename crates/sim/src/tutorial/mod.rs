//! **The tutorials** (2026-10-23): one hall per class, walked end to end.
//!
//! A tutorial is a long hall of rooms, one after the other along `+x`
//! ([`crate::arena::hall`]). The first says what the class is, how it plays
//! and how it wins; then a room for each click -- every way that click can
//! be thrown, in order: on the floor, held, in the air, off a jump -- and a
//! room for the keys that move her somewhere (a dash, a blink, a send). Past
//! those, **the advanced rooms**, one technique each: the things that get the
//! most out of a class and that nobody needs to play it.
//!
//! **A room is where you stand.** Which room you are in is a function of how
//! far down the hall you are ([`room_at`]), so walking into one is what opens
//! it; the dummy follows you in (`World::step_hall`), standing where the room
//! wants it, and is never knocked out.
//!
//! **A task is a question asked of two frames.** Each [`Task`] has a
//! [`Check`]: a function of the world before and after one tick, true on the
//! frame she did the thing -- a move begun, a stone raised, a jump off one --
//! read the way `sound::cue` reads a blow and the fight report reads a
//! round, so nothing about teaching is in the step. A task that says it
//! [`lands`](Task::lands) waits a little after it is done to see whether the
//! dummy was struck, and says which.
//!
//! What a player has done is [`Progress`]: how many times each task, per
//! class, and the last thing worth saying. **Not in the snapshot.** It
//! changes nothing in the fight -- it is a tally the game keeps beside it,
//! as it keeps trophies -- and it is fixed size, so keeping it never
//! allocates.

use crate::arena::ArenaId;
use crate::class::Class;
use crate::fixed::Fx;
use crate::state::{Action, Player, World};

pub mod blood;
pub mod champion;
pub mod dual;
pub mod elementalist;
pub mod reaver;

/// The most rooms a hall has.
pub const MAX_ROOMS: usize = 14;

/// The most tasks a room has.
pub const MAX_TASKS: usize = 8;

/// One class's tutorial.
#[derive(Debug)]
pub struct Tutorial {
    pub class: Class,
    /// Its hall.
    pub arena: ArenaId,
    /// **The premise**, read in the first room: what she is, how she plays,
    /// how she wins. Plain words.
    pub what: &'static str,
    pub plays: &'static str,
    pub wins: &'static str,
    pub rooms: &'static [Room],
}

/// One room of a hall.
#[derive(Debug)]
pub struct Room {
    pub title: &'static str,
    /// Past the basics: a technique to get the most out of the class, which
    /// she can be played well without.
    pub advanced: bool,
    /// What the room is for, in a few sentences.
    pub intro: &'static str,
    pub tasks: &'static [Task],
    /// Where the dummy stands while you are in here.
    pub dummy: Dummy,
    /// What the room has in it besides its floor.
    pub bay: Bay,
}

/// What the dummy does in a room.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dummy {
    /// In the middle of the room, facing you: something to hit.
    Stands,
    /// The same, swinging now and then: something to answer.
    Swings,
    /// Out of the way in a corner: the room is about getting somewhere.
    Aside,
}

/// What a room has in it besides its floor ([`crate::arena::hall`] builds
/// it). Heights in centimetres.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bay {
    /// Nothing: a floor to fight on.
    Flat,
    /// **A shelf across the far end**, this high, with a walk round it at
    /// one side so nobody is stuck below it: somewhere to get up to.
    Shelf(i32),
    /// **A wall across the middle**, this high, with a gap at one side: a
    /// face to slide down and kick off, or to get over.
    Wall(i32),
    /// **Two shelves with a gap between**, this high and this wide: a
    /// crossing, with the floor below to walk back from. Low enough to jump
    /// onto.
    Gap(i32, i32),
}

/// One thing to do.
#[derive(Debug)]
pub struct Task {
    /// What it is, in a few words: "Sword, on the ground".
    pub what: &'static str,
    /// How, in keys: "Tap left click".
    pub keys: &'static str,
    /// Is it done this frame?
    pub check: Check,
    /// Does it have to strike the dummy to count? A move that hits waits
    /// [`LANDING`] frames after it begins to be told whether it did.
    pub lands: bool,
}

/// A task's question: did she do it, between these two frames?
pub type Check = fn(&Seen) -> bool;

/// Two frames of the world, as a task reads them: fighter zero's.
#[derive(Clone, Copy)]
pub struct Seen<'a> {
    pub before: &'a World,
    pub after: &'a World,
}

/// Which move an action is part of, if any.
pub const fn kind_of(a: Action) -> Option<u8> {
    match a {
        Action::Startup { kind, .. }
        | Action::Active { kind, .. }
        | Action::Recovery { kind, .. }
        | Action::Channel { kind, .. } => Some(kind),
        _ => None,
    }
}

impl Seen<'_> {
    /// Her, before.
    pub fn was(&self) -> &Player {
        &self.before.players[0]
    }

    /// Her, after.
    pub fn now(&self) -> &Player {
        &self.after.players[0]
    }

    /// **Did move `kind` begin this frame?** Its windup began: every move
    /// has one, and a hold's comes when the hold is done -- so a channel
    /// that becomes a tap is the tap beginning, not the hold.
    pub fn began(&self, kind: u8) -> bool {
        let windup = |a: Action| matches!(a, Action::Startup { kind: k, .. } if k == kind);
        windup(self.now().action) && !windup(self.was().action)
    }

    /// **Did move `kind` come out live this frame?** For a click whose tap
    /// and hold share a windup, the tap is only certain once it is out.
    pub fn out(&self, kind: u8) -> bool {
        let live = |a: Action| matches!(a, Action::Active { kind: k, .. } if k == kind);
        live(self.now().action) && !live(self.was().action)
    }

    /// **Did a hold of move `kind` begin this frame?** The button is down
    /// and the move is being wound up for as long as it stays down.
    pub fn channelled(&self, kind: u8) -> bool {
        let held = |a: Action| matches!(a, Action::Channel { kind: k, .. } if k == kind);
        held(self.now().action) && !held(self.was().action)
    }

    /// Her shadow, before and after, if she has one.
    pub fn shadow(&self) -> Option<(crate::class::Shadow, crate::class::Shadow)> {
        Some((
            crate::shadow::of(self.was())?,
            crate::shadow::of(self.now())?,
        ))
    }

    /// How many of her effects of this kind there are, before and after.
    pub fn effects(&self, kind: crate::effects::EffectKind) -> (usize, usize) {
        let count = |w: &World| {
            w.effects
                .iter()
                .flatten()
                .filter(|e| e.owner == 0 && e.kind == kind)
                .count()
        };
        (count(self.before), count(self.after))
    }

    /// Did a new effect of hers of this kind appear this frame?
    pub fn appeared(&self, kind: crate::effects::EffectKind) -> bool {
        let (was, now) = self.effects(kind);
        now > was
    }

    /// How far she moved this frame, flat, in metres.
    pub fn moved(&self) -> Fx {
        crate::math::wide_flat_dist(self.was().pos, self.now().pos)
    }

    /// Did any of these begin this frame?
    pub fn began_any(&self, kinds: &[u8]) -> bool {
        kinds.iter().any(|&k| self.began(k))
    }

    /// Did `kind` begin with her feet off the floor?
    pub fn began_aloft(&self, kind: u8) -> bool {
        self.began(kind) && !self.was().grounded
    }

    /// Did `kind` begin with her standing?
    pub fn began_standing(&self, kind: u8) -> bool {
        self.began(kind) && self.was().grounded
    }

    /// **Did move `kind` end its channel** -- the button let go of after a
    /// hold -- this frame? She was channelling it and now is not.
    pub fn released(&self, kind: u8) -> bool {
        matches!(self.was().action, Action::Channel { kind: k, .. } if k == kind)
            && !matches!(self.now().action, Action::Channel { kind: k, .. } if k == kind)
    }

    /// Did she leave the floor this frame, by any means?
    pub fn took_off(&self) -> bool {
        self.was().grounded && !self.now().grounded
    }

    /// Did a dodge begin this frame?
    pub fn dodged(&self) -> bool {
        matches!(self.now().action, Action::Dodge { .. })
            && !matches!(self.was().action, Action::Dodge { .. })
    }

    /// How high her feet are, in metres.
    pub fn height(&self) -> Fx {
        self.now().pos.y
    }

    /// The dummy, before and after.
    pub fn dummy(&self) -> (&Player, &Player) {
        (&self.before.players[1], &self.after.players[1])
    }
}

/// **Was the dummy struck this frame?** It is stood back up to a full bar
/// at the top of every frame in a hall (`World::step_hall`), so a bar short
/// of full by more than a curse's tick is a blow this frame; or a stun
/// began, or it was marked. A hit from anything: a blade, a stone, a bolt
/// arriving a second after it was thrown.
pub fn struck(was: &Player, now: &Player) -> bool {
    let stun = |a: Action| {
        matches!(
            a,
            Action::HitStun { .. }
                | Action::BlockStun { .. }
                | Action::Stagger { .. }
                | Action::Held { .. }
        )
    };
    now.health <= now.full_health() - TICK
        || (stun(now.action) && !stun(was.action))
        || now.marks > was.marks
}

/// The most a frame of something lingering takes -- a curse's tick, a
/// bleed -- which is not a blow.
const TICK: i32 = 4;

/// **How long a move has to land**, in frames from when it began: long
/// enough for a thrown thing to fly the length of a room, short enough that
/// a miss is said while the swing is still in mind.
pub const LANDING: u32 = 75;

/// How long the last thing said stays said, in frames.
pub const SAID_FOR: u32 = 150;

/// The length of a room along the hall, in centimetres.
pub const ROOM_CM: i32 = 2400;

/// **What counts as done** -- a coach's lines, not a feel: past them a move
/// has plainly happened. A dodge that carried her further than a slide in
/// one frame was a teleport or a blink, in metres; one that did not was a
/// dodge.
pub const BLINKED: i32 = 3;
pub const SLID: i32 = 2;
/// A Dual mage's bars a third full, out of the hundred they hold.
pub const BARS_THIRD: i32 = 30;

/// Every tutorial, in the rotation's order.
pub const ALL: [&Tutorial; 5] = [
    &champion::TUTORIAL,
    &reaver::TUTORIAL,
    &elementalist::TUTORIAL,
    &blood::TUTORIAL,
    &dual::TUTORIAL,
];

/// The tutorial whose hall this is.
pub fn of(arena: ArenaId) -> Option<&'static Tutorial> {
    ALL.iter().copied().find(|t| t.arena == arena)
}

/// The tutorial for a class, if it has one. The Bulwark is not on offer
/// (`crate::class::ROTATION`) and has none.
pub fn for_class(class: Class) -> Option<&'static Tutorial> {
    ALL.iter().copied().find(|t| t.class == class)
}

/// **Which room a point is in**: how far down the hall it is, a room to
/// every [`ROOM_CM`], the first and last holding everything before and
/// after them.
pub fn room_at(t: &Tutorial, x: Fx) -> usize {
    let k = x.raw().div_euclid(crate::arena::cm(ROOM_CM).raw());
    (k.max(0) as usize).min(t.rooms.len().saturating_sub(1))
}

/// Where room `k` begins and ends along the hall, in metres.
pub fn span(k: usize) -> (Fx, Fx) {
    let lo = crate::arena::cm(ROOM_CM * k as i32);
    (lo, lo.add(crate::arena::cm(ROOM_CM)))
}

/// **How often a swinging dummy swings**, in frames: a count the panel shows.
pub const SWING_EVERY: u32 = 150;

/// **The dummy's buttons in a hall**, or `None` anywhere else: still, or --
/// in a room that wants it -- a spear thrust at fighter zero every
/// [`SWING_EVERY`] frames. A spear rather than a sword because a clash is
/// her finisher live as the blow arrives, and a quick sword up close is a
/// blow her finisher reaches first: the dummy is simply hit, its swing taken
/// back, and the clash a frame wide. The spear reaches her from further than
/// the hammer reaches back, which opens it to a few frames.
pub fn dummy_input(w: &World) -> Option<crate::Input> {
    let t = of(w.arena)?;
    let room = &t.rooms[room_at(t, w.players[0].pos.x)];
    if room.dummy != Dummy::Swings {
        return Some(crate::Input::default());
    }
    let d = w.players[0].pos.sub(w.players[1].pos);
    let aim = (crate::math::atan2_turns(d.z, d.x).raw() & 0xFFFF) as u16;
    let bits = if w.frame % SWING_EVERY < 2 {
        crate::Input::RIGHT
    } else {
        0
    };
    Some(crate::Input::looking_at(bits, aim, 0))
}

/// What is worth saying, after a frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    /// Task `task` of room `room` is done, `times` times now.
    Done { room: u8, task: u8, times: u16 },
    /// It was done but did not touch the dummy.
    Missed { room: u8, task: u8 },
}

/// A task done that has to land, waiting to hear whether it did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Waiting {
    room: u8,
    task: u8,
    since: u32,
}

/// The most tasks waiting to land at once: a string thrown fast.
const WAITING: usize = 4;

/// **What a player has done in the tutorials**: a tally for every task of
/// every class's hall, and the last thing worth saying. Kept by the game
/// beside the fight (`game::tutorial`), fed every tick.
#[derive(Clone, Copy, Debug)]
pub struct Progress {
    tally: [[[u16; MAX_TASKS]; MAX_ROOMS]; crate::class::ALL_CLASSES.len()],
    waiting: [Option<Waiting>; WAITING],
    /// The last thing said, and the frame it was said on.
    pub said: Option<(Word, u32)>,
}

impl Default for Progress {
    fn default() -> Self {
        Progress {
            tally: [[[0; MAX_TASKS]; MAX_ROOMS]; crate::class::ALL_CLASSES.len()],
            waiting: [None; WAITING],
            said: None,
        }
    }
}

fn slot(class: Class) -> usize {
    crate::class::ALL_CLASSES
        .iter()
        .position(|&c| c == class)
        .unwrap_or(0)
}

impl Progress {
    /// How many times task `task` of room `room` of `class`'s hall is done.
    pub fn times(&self, class: Class, room: usize, task: usize) -> u16 {
        self.tally[slot(class)]
            .get(room)
            .and_then(|r| r.get(task))
            .copied()
            .unwrap_or(0)
    }

    /// Is every task of that room done at least once?
    pub fn room_done(&self, t: &Tutorial, room: usize) -> bool {
        t.rooms
            .get(room)
            .is_some_and(|r| (0..r.tasks.len()).all(|k| self.times(t.class, room, k) > 0))
    }

    /// How many of a hall's rooms are done, of how many: the basics and the
    /// advanced ones apart.
    pub fn rooms_done(&self, t: &Tutorial, advanced: bool) -> (usize, usize) {
        let rooms = t
            .rooms
            .iter()
            .enumerate()
            .filter(|(_, r)| r.advanced == advanced);
        let (mut done, mut of) = (0, 0);
        for (k, r) in rooms {
            if r.tasks.is_empty() {
                continue;
            }
            of += 1;
            if self.room_done(t, k) {
                done += 1;
            }
        }
        (done, of)
    }

    /// Start a class's hall over.
    pub fn forget(&mut self, class: Class) {
        self.tally[slot(class)] = [[0; MAX_TASKS]; MAX_ROOMS];
        self.waiting = [None; WAITING];
    }

    /// **One tick of the hall**: what she did between these two frames, in
    /// the room she is standing in. Returns what is worth saying, which is
    /// also kept as [`Progress::said`].
    pub fn observe(&mut self, before: &World, after: &World) -> Option<Word> {
        let t = of(after.arena)?;
        if before.arena != after.arena || after.players[0].class != t.class {
            self.waiting = [None; WAITING];
            return None;
        }
        let seen = Seen { before, after };
        let room = room_at(t, after.players[0].pos.x);
        let mut word = None;
        let class = slot(t.class);

        // Waited too long: a miss.
        for w in self.waiting.iter_mut() {
            if let Some(x) = *w
                && after.frame.wrapping_sub(x.since) > LANDING
            {
                *w = None;
                word = word.or(Some(Word::Missed {
                    room: x.room,
                    task: x.task,
                }));
            }
        }

        if let Some(r) = t.rooms.get(room) {
            for (k, task) in r.tasks.iter().enumerate().take(MAX_TASKS) {
                if !(task.check)(&seen) {
                    continue;
                }
                if task.lands {
                    // The oldest waiting makes way.
                    let free = (0..WAITING)
                        .find(|&i| self.waiting[i].is_none())
                        .or_else(|| {
                            (0..WAITING).min_by_key(|&i| self.waiting[i].map_or(0, |w| w.since))
                        })
                        .unwrap_or(0);
                    self.waiting[free] = Some(Waiting {
                        room: room as u8,
                        task: k as u8,
                        since: after.frame,
                    });
                } else {
                    let n = &mut self.tally[class][room][k];
                    *n = n.saturating_add(1);
                    word = Some(Word::Done {
                        room: room as u8,
                        task: k as u8,
                        times: *n,
                    });
                }
            }
        }
        // The dummy struck: the newest task waiting to land, landed -- one
        // begun this very frame too, since a blow can land on its first.
        if struck(&before.players[1], &after.players[1])
            && let Some(k) = (0..WAITING)
                .filter(|&k| self.waiting[k].is_some())
                .max_by_key(|&k| self.waiting[k].map_or(0, |w| w.since))
            && let Some(w) = self.waiting[k].take()
        {
            let n = &mut self.tally[class][w.room as usize][w.task as usize];
            *n = n.saturating_add(1);
            word = Some(Word::Done {
                room: w.room,
                task: w.task,
                times: *n,
            });
        }
        if let Some(w) = word {
            self.said = Some((w, after.frame));
        }
        word
    }

    /// The last thing said, while it is still worth showing.
    pub fn saying(&self, frame: u32) -> Option<Word> {
        self.said
            .filter(|(_, at)| frame.wrapping_sub(*at) < SAID_FOR)
            .map(|(w, _)| w)
    }
}
