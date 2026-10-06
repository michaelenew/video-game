//! **Jump courses**: arenas with no creature, where the challenge is the
//! movement system itself -- world.md §2's *trails*, built first as measuring
//! instruments (`docs/design/courses.md`).
//!
//! A course is an arena ([`crate::arena::climb`]) and a table: a **route** of
//! islands -- each a solid of the arena, the hop to it written down as built
//! -- some of which are **gates** (the start, the checkpoints, the finish: a
//! body whose feet are on that top has reached it, and a fall stands it back
//! on the middle of it); and a **pit**, the height below which a body has
//! fallen. Everything in it is data, the way an arena is.
//!
//! What a fight keeps of it is [`Run`], one per fighter, twelve bytes: the
//! furthest gate reached, how often she fell, when she left the start and how
//! long she took. In the snapshot, because a fall is gameplay -- it puts a
//! body somewhere else -- and both peers have to put it there on the same
//! frame. **A fall is not a death**: she is stood on the last gate she reached
//! with everything she had out taken back (her stones, her shadow, her
//! shield), as a round starts, and the clock keeps running.
//!
//! Nothing here reads the Oven: a course's sizes are its arena's, and a gate
//! is part of the course the way a ledge is.

use crate::arena::{ArenaId, Solid, cm};
use crate::class::Class;
use crate::fixed::Fx;
use crate::math::V3;

/// One step of a course's route: an island to reach, by its index in the
/// arena's solids, and what is known about getting there from the step
/// before.
#[derive(Clone, Copy, Debug)]
pub struct Step {
    /// The solid whose top is this step.
    pub solid: u8,
    /// A checkpoint: a fall after reaching it stands you back here. The
    /// start and the finish always are.
    pub check: bool,
    /// The hop from the step before, as built: the gap between the two tops'
    /// facing edges and how far this top is above that one, in centimetres.
    /// Zero for the start.
    pub gap: i32,
    pub rise: i32,
    /// What the hop asks of you, in a few words: "level 7 m", "stones".
    pub ask: &'static str,
}

/// How hard a course was built to be.
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Tier {
    /// Every class clears it with a plain jump, with room to spare.
    Easy,
    /// Every class clears it, near the edge of what its jump and airdodge
    /// can do. The main route the game is meant to be.
    Hard,
    /// Barely possible, and only for some.
    Edge,
    /// Not a route but a playground: one hub, and targets at marked
    /// distances to try each class's mechanics against. A fall always stands
    /// you back on the hub.
    Proving,
}

impl Tier {
    pub const fn name(self) -> &'static str {
        match self {
            Tier::Easy => "easy",
            Tier::Hard => "hard",
            Tier::Edge => "barely possible",
            Tier::Proving => "proving ground",
        }
    }
}

/// One course.
#[derive(Debug)]
pub struct Course {
    pub arena: ArenaId,
    /// What the HUD calls it.
    pub name: &'static str,
    pub tier: Tier,
    /// The class a barely-possible course was built against, if it was.
    pub for_class: Option<Class>,
    /// Below this height, in centimetres, a body has fallen.
    pub pit: i32,
    /// A line for the course panel: where the big jumps are, or a proving
    /// ground's distances. Empty for none.
    pub note: &'static str,
    /// The route: the start, every island on the way, the finish.
    pub route: &'static [Step],
}

/// A gate: a step of the route that counts -- the start, a checkpoint, the
/// finish -- as the box its top makes.
#[derive(Clone, Copy, Debug)]
pub struct Gate {
    /// Which step of the route.
    pub step: usize,
    pub top: Solid,
}

/// How far above a top the feet still count as on it: three bodies, so a
/// hop on the spot is still on the checkpoint and a flight far over it is not.
fn gate_height() -> Fx {
    let body = crate::tuning::body_height();
    body.add(body).add(body)
}

impl Gate {
    /// Is a body whose feet are at `pos` on it?
    pub fn holds(&self, pos: V3) -> bool {
        self.top.over(pos.x, pos.z, Fx::ZERO)
            && pos.y.raw() >= self.top.max.y.sub(crate::arena::SKIN).raw()
            && pos.y.raw() <= self.top.max.y.add(gate_height()).raw()
    }

    /// The middle of its top: where a fall stands you.
    pub fn stand(&self) -> V3 {
        let t = &self.top;
        V3::new(midway(t.min.x, t.max.x), t.max.y, midway(t.min.z, t.max.z))
    }
}

/// Halfway between two values: the middle of a span.
fn midway(a: Fx, b: Fx) -> Fx {
    Fx::from_raw(a.raw() + (b.raw() - a.raw()) / 2)
}

impl Course {
    pub fn arena(&self) -> &'static crate::arena::Arena {
        self.arena.get()
    }

    /// The solid a step stands on.
    pub fn top(&self, step: usize) -> Solid {
        let s = self.route[step.min(self.route.len() - 1)].solid as usize;
        self.arena().solids()[s.min(self.arena().solids().len() - 1)]
    }

    /// The gates, in order: the start, each checkpoint, the finish.
    pub fn gates(&self) -> impl Iterator<Item = Gate> + '_ {
        let last = self.route.len() - 1;
        self.route
            .iter()
            .enumerate()
            .filter(move |(i, s)| *i == 0 || *i == last || s.check)
            .map(|(i, _)| Gate {
                step: i,
                top: self.top(i),
            })
    }

    /// Gate `n`, or the finish if there are fewer.
    pub fn gate(&self, n: usize) -> Gate {
        let mut last = None;
        for (i, g) in self.gates().enumerate() {
            if i == n {
                return g;
            }
            last = Some(g);
        }
        last.unwrap_or(Gate {
            step: 0,
            top: self.top(0),
        })
    }

    /// How many gates: the start, the checkpoints, the finish.
    pub fn gate_count(&self) -> usize {
        self.gates().count()
    }

    /// The last gate's number.
    pub fn finish(&self) -> usize {
        self.gate_count() - 1
    }

    /// Which way a fall stood on gate `n` faces: toward the step after it.
    pub fn facing(&self, n: usize) -> V3 {
        let g = self.gate(n);
        let next = self.top((g.step + 1).min(self.route.len() - 1));
        let to = Gate { step: 0, top: next }.stand();
        let d = to.sub(g.stand());
        let flat = V3::new(d.x, Fx::ZERO, d.z);
        if flat.flat_len().raw() == 0 {
            V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO)
        } else {
            flat.normalized()
        }
    }
}

/// The course an arena is, if it is one.
pub fn of(arena: ArenaId) -> Option<&'static Course> {
    crate::arena::climb::COURSES
        .iter()
        .copied()
        .find(|c| c.arena == arena)
}

/// Every course, in the order the picker steps through them: by tier, then
/// as listed.
/// The height a body has fallen below in this arena, if it is a course: the
/// floor under its islands is a drop, not somewhere to stand. `None` for every
/// arena whose floor is a floor.
pub fn pit_of(arena: ArenaId) -> Option<Fx> {
    of(arena).map(|c| cm(c.pit))
}

pub fn all() -> impl Iterator<Item = &'static Course> {
    crate::arena::climb::COURSES.iter().copied()
}

/// The course after this arena's in [`all`], wrapping; the first from an
/// arena that is not a course.
pub fn after(arena: ArenaId) -> &'static Course {
    let list = crate::arena::climb::COURSES;
    let here = list.iter().position(|c| c.arena == arena);
    list[here.map_or(0, |i| (i + 1) % list.len())]
}

/// How one fighter's run is going. Twelve bytes; all zero before it starts.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Run {
    /// The furthest gate reached: where a fall sends you back to.
    pub reached: u8,
    /// How many times she has fallen.
    pub falls: u16,
    /// The frame she first left the start gate, plus one; zero until she has.
    pub started: u32,
    /// Frames from leaving the start to reaching the finish; zero until she
    /// has.
    pub time: u32,
}

impl Run {
    /// Has she reached the finish?
    pub fn finished(&self) -> bool {
        self.time > 0
    }

    /// Frames on the clock at `frame`: still running, stopped at the finish,
    /// or zero before the start.
    pub fn clock(&self, frame: u32) -> u32 {
        if self.finished() {
            self.time
        } else if self.started > 0 {
            frame.saturating_sub(self.started - 1)
        } else {
            0
        }
    }
}

/// What one frame of a course did to one fighter's run.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Event {
    Nothing,
    /// Fell: stand her on this gate.
    Fell(usize),
}

/// One frame of a fighter's run: has she fallen, left the start, reached a
/// gate? Pure: the world does what it says.
pub fn step(course: &Course, run: &mut Run, pos: V3, frame: u32) -> Event {
    if pos.y.raw() < cm(course.pit).raw() {
        run.falls = run.falls.saturating_add(1);
        // A proving ground has no way forward to keep: every try starts from
        // the hub.
        let back = if course.tier == Tier::Proving {
            0
        } else {
            run.reached as usize
        };
        return Event::Fell(back);
    }
    if run.started == 0 && !course.gate(0).holds(pos) {
        run.started = frame.saturating_add(1).max(1);
    }
    // Any later gate counts, so a route that skips a checkpoint is a route.
    for (i, gate) in course.gates().enumerate().skip(run.reached as usize + 1) {
        if gate.holds(pos) {
            run.reached = i as u8;
        }
    }
    if run.reached as usize == course.finish() && !run.finished() {
        run.time = run.clock(frame).max(1);
    }
    Event::Nothing
}
