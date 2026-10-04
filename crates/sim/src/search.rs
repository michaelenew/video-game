//! **A search over input programs**: the best a class can do with its feet,
//! found rather than guessed. The instrument behind `courses.md` since round
//! two, after hand-written routes were shown to underestimate every class.
//!
//! A [`Program`] is a few numbers and a short list of timed inputs, built out
//! of a handful of blocks a player actually has:
//!
//! - **where to start**: how far back from the edge (`back`), and so how long
//!   the run-up;
//! - **the stick in the air**: let go until frame `strafe_at`, then Quake
//!   strafing, as a player does it -- the stick held at `strafe` degrees off
//!   the line she is moving along, turned toward the goal, the camera turning
//!   with it (zero degrees is the stick held at the goal);
//! - **events**: a jump held so many frames, the airdodge, any click (which in
//!   the air is an aerial, and an aerial hangs her), the mechanic key, a
//!   button held with the crosshair on a point (the Reaver's shadow, a stone,
//!   the Blood mage's Grasp), the dash to the shadow, the Champion's vault.
//!
//! [`run`] plays a program on a [`Stage`] -- a top to leave, a top to reach --
//! in a real `World`, frame by frame, and reports where she got to. [`search`]
//! is a seeded random search with hill climbing over programs: mutate the
//! best few, keep what scores better. It is deterministic for a seed.
//!
//! **A search is a lower bound.** It finds what it finds in its budget; a
//! player, or a longer search, may find more. Every program it keeps is
//! written out (`Program`'s `Display`, read back by `Program::parse`) and
//! replayed by `tests/search.rs`, so every number claimed is a run that
//! actually happens.
//!
//! It builds worlds, keeps lists and hashes nothing; a tool, not a frame.
//! Integer arithmetic only, like the rest of the crate.

use crate::arena::{ArenaId, Solid, cm};
use crate::class::Class;
use crate::fixed::Fx;
use crate::input::Input;
use crate::math::{self, V3};
use crate::state::World;

// ---------------------------------------------------------------------------
// Programs
// ---------------------------------------------------------------------------

/// What a point an event looks at is measured from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Base {
    /// Her own feet, where she is on the frame.
    Feet,
    /// The near edge of the top she is trying to reach.
    Lip,
    /// The middle of the top in between -- for a two-hop line, the stepping
    /// stone; otherwise the goal's middle.
    Mid,
}

/// A point to put the crosshair on: from a base, so far along the line of
/// travel and so far up, in centimetres.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Aim {
    pub base: Base,
    pub fwd: i16,
    pub up: i16,
}

/// The buttons an event can press, by number.
pub const BUTTONS: [u16; 7] = [
    Input::LEFT,
    Input::RIGHT,
    Input::MIDDLE,
    Input::SPECIAL,
    Input::KEY_F,
    Input::KEY_R,
    Input::MECHANIC,
];

/// One timed input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    /// Space, held this many frames.
    Jump(u8),
    /// Shift and the stick: the dodge, in the air the airdodge.
    Dodge,
    /// Button `n` of [`BUTTONS`], looking where she is steering. In the air,
    /// a click is that class's aerial.
    Press(u8),
    /// Button `n`, the crosshair on a point, held so many frames.
    At(u8, Aim, u8),
    /// Shift and forward with the crosshair on her shadow: the Reaver's dash.
    Dash,
    /// Right click with the crosshair under the vault pitch: the Champion's
    /// pole vault, out of a Rush.
    Vault,
}

impl Act {
    /// How many frames the input is held.
    pub fn frames(self) -> u16 {
        match self {
            Act::Jump(n) => n.max(1) as u16,
            Act::At(_, _, n) => n.max(1) as u16,
            // Held until the dash takes: it cannot start while the send is
            // still recovering, and a player holds the button through that.
            Act::Dash => 10,
            _ => 2,
        }
    }
}

/// A program: where to start, how to strafe, and what to press when.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    /// How far back from the edge she starts, in centimetres.
    pub back: u16,
    /// The strafe: degrees off her line of motion, toward the goal. Zero is
    /// the stick held at the goal.
    pub strafe: u8,
    /// From which frame the strafe is held.
    pub strafe_at: u16,
    /// Frames she stands still before she moves at all: a stone raised under
    /// her own feet wants her standing on it.
    pub still: u16,
    /// What is pressed when, frame numbers from the start.
    pub events: Vec<(u16, Act)>,
}

/// The frame a run reaches the edge from `back` centimetres, at a walk.
pub fn edge_frame(back: u16) -> u16 {
    let walk = crate::tuning::move_speed().mul(crate::DT);
    let frames = cm(back as i32).div(walk.max(crate::DT));
    frames.to_int().max(0) as u16
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}@{} {} ;",
            self.back, self.strafe, self.strafe_at, self.still
        )?;
        for (at, act) in &self.events {
            write!(f, " {at}:")?;
            match act {
                Act::Jump(n) => write!(f, "J{n}")?,
                Act::Dodge => write!(f, "D")?,
                Act::Press(b) => write!(f, "P{b}")?,
                Act::At(b, aim, n) => {
                    let base = match aim.base {
                        Base::Feet => 'f',
                        Base::Lip => 'l',
                        Base::Mid => 'm',
                    };
                    write!(f, "A{b}{base}{},{},{n}", aim.fwd, aim.up)?
                }
                Act::Dash => write!(f, "X")?,
                Act::Vault => write!(f, "V")?,
            }
        }
        Ok(())
    }
}

impl Program {
    /// Read one back from its `Display`.
    pub fn parse(s: &str) -> Option<Program> {
        let (head, tail) = s.split_once(';')?;
        let mut head = head.split_whitespace();
        let back = head.next()?.parse().ok()?;
        let (strafe, strafe_at) = head.next()?.split_once('@')?;
        let still = head.next().and_then(|n| n.parse().ok()).unwrap_or(0);
        let mut events = Vec::new();
        for tok in tail.split_whitespace() {
            let (at, act) = tok.split_once(':')?;
            let at: u16 = at.parse().ok()?;
            let (kind, rest) = act.split_at(1);
            let act = match kind {
                "J" => Act::Jump(rest.parse().ok()?),
                "D" => Act::Dodge,
                "P" => Act::Press(rest.parse().ok()?),
                "X" => Act::Dash,
                "V" => Act::Vault,
                "A" => {
                    let b: u8 = rest.get(..1)?.parse().ok()?;
                    let base = match rest.get(1..2)? {
                        "f" => Base::Feet,
                        "l" => Base::Lip,
                        _ => Base::Mid,
                    };
                    let mut nums = rest.get(2..)?.split(',');
                    let fwd = nums.next()?.parse().ok()?;
                    let up = nums.next()?.parse().ok()?;
                    let n = nums.next()?.parse().ok()?;
                    Act::At(b, Aim { base, fwd, up }, n)
                }
                _ => return None,
            };
            events.push((at, act));
        }
        Some(Program {
            back,
            strafe: strafe.parse().ok()?,
            strafe_at: strafe_at.parse().ok()?,
            still,
            events,
        })
    }

    /// The same program with event `i` moved by `by` frames.
    pub fn shifted(&self, i: usize, by: i32) -> Program {
        let mut p = self.clone();
        if let Some(e) = p.events.get_mut(i) {
            e.0 = (e.0 as i32 + by).max(0) as u16;
        }
        p
    }
}

// ---------------------------------------------------------------------------
// Stages
// ---------------------------------------------------------------------------

/// Where a program is played: an arena, a top to leave and the point on its
/// edge she leaves from, the direction of travel, and the top to reach.
#[derive(Clone, Copy, Debug)]
pub struct Stage {
    pub arena: ArenaId,
    pub class: Class,
    /// The top she starts on.
    pub from: Solid,
    /// The point on its edge she leaves from, and which way is forward.
    pub edge: V3,
    pub fwd: V3,
    /// The top between, for a two-hop line; the goal itself otherwise.
    pub mid: Solid,
    /// The top to end on.
    pub to: Solid,
    /// The frames a run gets.
    pub frames: u32,
    /// Measure the gap she crossed (the lab's lanes), rather than only
    /// whether she arrived.
    pub measure: bool,
}

/// The flat middle of a top.
pub fn middle(s: &Solid) -> V3 {
    let half = |a: Fx, b: Fx| Fx::from_raw(a.raw() + (b.raw() - a.raw()) / 2);
    V3::new(half(s.min.x, s.max.x), s.max.y, half(s.min.z, s.max.z))
}

/// The point of a top nearest a point, flat, at the top's height.
fn nearest_on(s: &Solid, p: V3) -> V3 {
    V3::new(
        p.x.clamp(s.min.x, s.max.x),
        s.max.y,
        p.z.clamp(s.min.z, s.max.z),
    )
}

/// Where the line from a top's middle along `fwd` leaves it.
fn exit(s: &Solid, fwd: V3) -> V3 {
    let mid = middle(s);
    let step = cm(5);
    let mut at = mid;
    for _ in 0..4000 {
        let next = at.add(fwd.scale(step));
        if !s.over(next.x, next.z, Fx::ZERO) {
            return at;
        }
        at = next;
    }
    at
}

impl Stage {
    /// A hop between two tops of an arena: from the middle of one, along the
    /// line between the middles, to the other.
    pub fn hop(arena: ArenaId, class: Class, from: Solid, mid: Solid, to: Solid) -> Stage {
        let d = middle(&mid).sub(middle(&from));
        let fwd = V3::new(d.x, Fx::ZERO, d.z).normalized();
        Stage {
            arena,
            class,
            from,
            edge: exit(&from, fwd),
            fwd,
            mid,
            to,
            frames: 720,
            measure: false,
        }
    }

    /// Lane `i` of the lab: off the runway onto that lane's ledge.
    pub fn lane(class: Class, i: usize) -> Stage {
        let a = crate::arena::lab::ARENA.solids();
        let (z, _) = crate::arena::lab::LANES[i];
        let edge = V3::new(cm(crate::arena::lab::RUNWAY.0), a[0].max.y, cm(z));
        Stage {
            arena: ArenaId::LAB,
            class,
            from: a[0],
            edge,
            fwd: V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
            mid: a[i + 1],
            to: a[i + 1],
            frames: 900,
            measure: true,
        }
    }

    /// The point an aim names, from where she stands.
    fn point(&self, aim: Aim, feet: V3) -> V3 {
        let base = match aim.base {
            Base::Feet => feet,
            Base::Lip => nearest_on(&self.to, self.edge),
            Base::Mid => middle(&self.mid),
        };
        base.add(self.fwd.scale(cm(aim.fwd as i32))).add(V3::new(
            Fx::ZERO,
            cm(aim.up as i32),
            Fx::ZERO,
        ))
    }

    /// The world, with her standing `back` from the edge.
    pub fn world(&self, back: u16) -> World {
        let mut w = World::versus_in([self.class, Class::Champion], self.arena);
        w.players[1].health = i32::MAX / 2;
        // Clear of every run: the second fighter in the lab's corner, or
        // left where a course stands it, on the start.
        let mut at = self.edge.sub(self.fwd.scale(cm(back as i32)));
        let r = crate::tuning::body_radius();
        while !self.from.over(at.x, at.z, Fx::ZERO) && at.sub(self.edge).flat_len().raw() > 0 {
            at = at.add(self.fwd.scale(r));
        }
        let p = &mut w.players[0];
        p.pos = V3::new(at.x, self.from.max.y, at.z);
        p.vel = V3::ZERO;
        p.facing = self.fwd;
        p.grounded = true;
        if let crate::class::Mechanic::Shadow(_) = p.mechanic {
            p.mechanic =
                crate::class::Mechanic::Shadow(crate::class::Shadow::attending(p.pos, p.facing));
        }
        w
    }
}

// ---------------------------------------------------------------------------
// Playing one
// ---------------------------------------------------------------------------

/// What a run did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    /// She ended standing on the goal.
    pub landed: bool,
    /// The gap she crossed, on a lab lane: from the last place her feet were
    /// on the runway to where they came down on the ledge, plus a body's
    /// radius -- the edge a player standing there could have been on. The
    /// ledge's face is half a metre out, so where she lands on it is where a
    /// ledge as far out as that would still have caught her. Zero elsewhere.
    pub gap: Fx,
    /// The frame she landed on.
    pub frame: u32,
    /// How near a miss was: horizontally outside the goal, plus how far below
    /// its top she was, at the closest. Zero on a landing.
    pub short: Fx,
    /// On a landing, how far from the goal's middle her feet came down: a
    /// line that lands in the middle has room either side to be wrong.
    pub off: Fx,
}

/// Whole metres: a size in the instrument's own bookkeeping, not a feel.
fn metres(n: i32) -> Fx {
    Fx::from_int(n)
}

fn yaw_of(v: V3) -> u16 {
    (math::atan2_turns(v.z, v.x).raw() & 0xFFFF) as u16
}

/// The stick keys that walk `wish` while looking along `aim`.
fn stick(wish: u16, aim: u16) -> u16 {
    let rel = wish.wrapping_sub(aim);
    let sector = ((rel as u32 + 4096) / 8192) % 8;
    [
        Input::W,
        Input::W | Input::D,
        Input::D,
        Input::S | Input::D,
        Input::S,
        Input::S | Input::A,
        Input::A,
        Input::W | Input::A,
    ][sector as usize]
}

/// The degree, in turns.
fn degrees(d: i32) -> i32 {
    d * 65536 / 360
}

/// Play `prog` on `stage`. Calls `watch` with the world after every frame.
pub fn run_watching(stage: &Stage, prog: &Program, mut watch: impl FnMut(&World)) -> Outcome {
    let mut w = stage.world(prog.back);
    let r = crate::tuning::body_radius();
    let to = stage.to;
    let goal = middle(&to);
    let mut last = w.players[0].pos;
    let mut out = Outcome {
        short: metres(1000),
        ..Outcome::default()
    };
    let falls = w.course[0].falls;
    for f in 0..stage.frames {
        let p = w.players[0];
        let ff = f as u16;
        // The stick: at the goal on the ground and until the strafe starts;
        // then held off the line of motion, turning toward the goal; and at
        // the goal's middle again once she is over it.
        let to_goal = yaw_of(goal.sub(p.pos));
        let flat = V3::new(p.vel.x, Fx::ZERO, p.vel.z);
        let over = to.over(p.pos.x, p.pos.z, Fx::ZERO);
        // Off the floor before the strafe starts, the stick is let go: she
        // drifts on what she took off with, which is how a climb straight up
        // keeps clear of the underside of what it is climbing to.
        let drifting = (!p.grounded && !over && ff < prog.strafe_at) || ff < prog.still;
        let wish = if p.grounded
            || over
            || prog.strafe == 0
            || ff < prog.strafe_at
            || flat.flat_len().raw() < Fx::ONE.raw()
        {
            to_goal
        } else {
            let moving = yaw_of(flat);
            let turn = to_goal.wrapping_sub(moving) as i16;
            let side = if turn >= 0 { 1 } else { -1 };
            (moving as i32 + side * degrees(prog.strafe as i32)) as u16
        };
        let mut bits = 0u16;
        let mut look: Option<(u16, i16)> = None;
        for (at, act) in &prog.events {
            if ff < *at || ff >= at + act.frames() {
                continue;
            }
            match *act {
                Act::Jump(_) => bits |= Input::SPACE,
                Act::Dodge => bits |= Input::SHIFT,
                Act::Press(b) => bits |= BUTTONS[b as usize % BUTTONS.len()],
                Act::At(b, aim, _) => {
                    bits |= BUTTONS[b as usize % BUTTONS.len()];
                    look = Some(crate::envelope::look_at(&w, stage.point(aim, p.pos)));
                }
                Act::Dash => {
                    bits |= Input::SHIFT;
                    if let Some(s) = crate::shadow::of(&p) {
                        look = Some(crate::envelope::look_at(&w, s.pos));
                    }
                }
                Act::Vault => {
                    bits |= Input::RIGHT;
                    look = Some((wish, -(1 << 13)));
                }
            }
        }
        let dash = prog
            .events
            .iter()
            .any(|(at, a)| *a == Act::Dash && ff >= *at && ff < at + a.frames());
        let input = match look {
            Some((yaw, pitch)) => {
                let keys = if dash {
                    Input::W
                } else if drifting {
                    0
                } else {
                    stick(wish, yaw)
                };
                Input::looking_at(bits | keys, yaw, pitch)
            }
            None if drifting => Input::aimed(bits, wish),
            None => Input::aimed(bits | Input::W, wish),
        };
        crate::envelope::tick(&mut w, input);
        watch(&w);
        let p = w.players[0];
        if p.grounded
            && stage.from.over(p.pos.x, p.pos.z, r)
            && p.pos.y.sub(stage.from.max.y).abs().raw() <= crate::arena::SKIN.raw()
        {
            last = p.pos;
        }
        // How near the goal she is: outside it flat, and below its top.
        let outside = p.pos.sub(nearest_on(&to, p.pos)).flat_len();
        let below = to.max.y.sub(p.pos.y).max(Fx::ZERO);
        out.short = out.short.min(outside.add(below));
        // Standing on the goal -- its edge included, as the resolve holds a
        // body up by its radius. The first touch is the landing: a jump
        // that catches the near corner on the way up is stood on the top
        // there, and what it does after that is a second jump, not this one.
        if p.grounded
            && to.over(p.pos.x, p.pos.z, r)
            && p.pos.y.sub(to.max.y).abs().raw() <= crate::arena::SKIN.raw()
        {
            out.landed = true;
            out.frame = f;
            out.short = Fx::ZERO;
            out.off = p.pos.sub(middle(&to)).flat_len();
            break;
        }
        // Gone: into a course's pit, or well under both tops.
        let lowest = to.max.y.min(stage.from.max.y);
        if w.course[0].falls > falls || p.pos.y.raw() < lowest.sub(metres(10)).raw() {
            break;
        }
    }
    if stage.measure && out.landed {
        let along = w.players[0].pos.sub(last);
        out.gap = along
            .x
            .mul(stage.fwd.x)
            .add(along.z.mul(stage.fwd.z))
            .add(r);
    } else if stage.measure {
        // Short: credit what was crossed toward the face, so a search has a
        // slope to climb.
        let face = nearest_on(&to, stage.edge);
        let along = face.sub(last);
        out.gap = along
            .x
            .mul(stage.fwd.x)
            .add(along.z.mul(stage.fwd.z))
            .add(r);
    }
    out
}

/// Play `prog` on `stage`.
pub fn run(stage: &Stage, prog: &Program) -> Outcome {
    run_watching(stage, prog, |_| {})
}

/// A number to maximise: on a lane the gap crossed, less any shortfall; on a
/// hop, landing, then nearness.
pub fn score(stage: &Stage, o: &Outcome) -> i64 {
    let short = o.short.raw() as i64;
    if stage.measure {
        let gap = o.gap.raw() as i64;
        if o.landed {
            gap
        } else {
            gap - 2 * short - metres(50).raw() as i64
        }
    } else if o.landed {
        metres(1000).raw() as i64 - o.off.raw() as i64
    } else {
        -short
    }
}

// ---------------------------------------------------------------------------
// The search
// ---------------------------------------------------------------------------

/// A seeded xorshift: the search is the same search every time.
#[derive(Clone, Copy, Debug)]
pub struct Rng(pub u64);

impl Rng {
    pub fn draw(&mut self) -> u64 {
        let mut x = self.0.max(1);
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Uniform in `lo..=hi`.
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        lo + (self.draw() % (hi - lo + 1) as u64) as i32
    }

    pub fn chance(&mut self, of: u32) -> bool {
        self.draw() % of as u64 == 0
    }
}

/// What a search may press: the universal blocks, or the class's whole kit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kit {
    /// The jump, the airdodge, the clicks (aerials), the strafe.
    Shared,
    /// That, and the class's own tools.
    Full,
}

/// The acts a class may be given, for a kit.
fn palette(class: Class, kit: Kit, rng: &mut Rng) -> Act {
    let shared = |rng: &mut Rng| match rng.range(0, 5) {
        0 | 1 => {
            let top = if rng.chance(4) { 90 } else { 18 };
            Act::Jump(rng.range(1, top) as u8)
        }
        2 => Act::Dodge,
        3 => Act::Press(0),
        4 => Act::Press(1),
        _ => Act::Press(2),
    };
    if kit == Kit::Shared || rng.chance(2) {
        return shared(rng);
    }
    let aim = |rng: &mut Rng, base: Base| Aim {
        base,
        fwd: rng.range(-200, 400) as i16,
        up: rng.range(-100, 300) as i16,
    };
    match class {
        Class::ShadowReaver => match rng.range(0, 2) {
            0 => {
                let base = if rng.chance(2) { Base::Mid } else { Base::Lip };
                Act::At(1, aim(rng, base), 2)
            }
            1 => Act::Dash,
            _ => Act::Press(3),
        },
        Class::Elementalist => match rng.range(0, 3) {
            0 | 1 => Act::At(
                6,
                Aim {
                    base: Base::Feet,
                    fwd: rng.range(0, 450) as i16,
                    up: 0,
                },
                1,
            ),
            2 => Act::Press(4),
            _ => Act::Press(3),
        },
        Class::BloodMage => match rng.range(0, 2) {
            0 | 1 => {
                let base = if rng.chance(2) { Base::Mid } else { Base::Lip };
                Act::At(3, aim(rng, base), rng.range(1, 30) as u8)
            }
            _ => Act::Press(3),
        },
        Class::Champion => match rng.range(0, 2) {
            0 => Act::Press(6),
            1 => Act::Vault,
            _ => Act::Press(rng.range(0, 2) as u8),
        },
        Class::DualMage => match rng.range(0, 1) {
            0 => Act::Press(6),
            _ => Act::Press(3),
        },
        Class::Bulwark => shared(rng),
    }
}

/// The programs a search starts from: the plain jump at the edge, and the
/// lines a player of each class would try first.
pub fn seeds(class: Class, kit: Kit) -> Vec<Program> {
    let mut out = Vec::new();
    for back in [300u16, 800, 1500] {
        let e = edge_frame(back);
        out.push(Program {
            back,
            strafe: 0,
            strafe_at: 0,
            still: 0,
            events: vec![(e.saturating_sub(1), Act::Jump(18))],
        });
        out.push(Program {
            back,
            strafe: 60,
            strafe_at: e + 2,
            still: 0,
            events: vec![
                (e.saturating_sub(1), Act::Jump(18)),
                (e + 25, Act::Press(0)),
                (e + 40, Act::Dodge),
            ],
        });
    }
    if kit == Kit::Shared {
        return out;
    }
    let lip = |fwd: i16, up: i16| Aim {
        base: Base::Lip,
        fwd,
        up,
    };
    let mid = Aim {
        base: Base::Mid,
        fwd: 0,
        up: 0,
    };
    match class {
        Class::ShadowReaver => {
            // The dash jump off her own edge: the shadow sent along the top
            // she is on to its edge, the dash to it, and the jump out of the
            // dash, at every point of the carry.
            for back in [600u16, 900] {
                for jump in [11u16, 13, 15, 17] {
                    out.push(Program {
                        back,
                        strafe: 60,
                        strafe_at: jump + 18,
                        still: 0,
                        events: vec![
                            (
                                0,
                                Act::At(
                                    1,
                                    Aim {
                                        base: Base::Feet,
                                        fwd: back as i16 - 30,
                                        up: 0,
                                    },
                                    2,
                                ),
                            ),
                            (18, Act::Dash),
                            (jump + 14, Act::Jump(18)),
                            (jump + 25, Act::Press(0)),
                            (jump + 45, Act::Press(1)),
                        ],
                    });
                }
            }
            for back in [100u16, 400, 900] {
                let e = edge_frame(back);
                out.push(Program {
                    back,
                    strafe: 0,
                    strafe_at: 0,
                    still: 0,
                    events: vec![
                        (0, Act::At(1, mid, 2)),
                        (18, Act::Dash),
                        (40, Act::Jump(18)),
                    ],
                });
                out.push(Program {
                    back,
                    strafe: 50,
                    strafe_at: e + 30,
                    still: 0,
                    events: vec![
                        (0, Act::At(1, lip(100, 100), 2)),
                        (18, Act::Dash),
                        (38, Act::Jump(18)),
                    ],
                });
                // Thrown from the top of a jump, at a top above her: aimed
                // high enough that the ray clears the near edge, so the
                // shadow lands on the top rather than in the pit under its
                // face, and the dash's line clears the corner.
                for k in [8u16, 16, 24, 32] {
                    out.push(Program {
                        back,
                        strafe: 0,
                        strafe_at: 0,
                        still: 0,
                        events: vec![
                            (e.saturating_sub(1), Act::Jump(18)),
                            (e + k, Act::At(1, lip(150, 150), 2)),
                            (e + k + 18, Act::Dash),
                        ],
                    });
                }
            }
        }
        Class::Elementalist => {
            // Standing at the edge: two stones under her own feet, two
            // frames apart, and the jump timed onto both -- the double.
            for jump in 0u16..24 {
                out.push(Program {
                    back: 60,
                    strafe: 0,
                    strafe_at: 90,
                    still: 90,
                    events: vec![
                        (
                            0,
                            Act::At(
                                6,
                                Aim {
                                    base: Base::Feet,
                                    fwd: 0,
                                    up: 0,
                                },
                                1,
                            ),
                        ),
                        (
                            2,
                            Act::At(
                                6,
                                Aim {
                                    base: Base::Feet,
                                    fwd: 0,
                                    up: 0,
                                },
                                1,
                            ),
                        ),
                        (jump, Act::Jump(90)),
                    ],
                });
            }
            for back in [200u16, 800] {
                let e = edge_frame(back);
                let feet = |fwd: i16| Aim {
                    base: Base::Feet,
                    fwd,
                    up: 0,
                };
                out.push(Program {
                    back,
                    strafe: 0,
                    strafe_at: 0,
                    still: 0,
                    events: vec![
                        (e.saturating_sub(14), Act::At(6, feet(150), 1)),
                        (e.saturating_sub(12), Act::At(6, feet(150), 1)),
                        (e.saturating_sub(4), Act::Jump(90)),
                    ],
                });
                out.push(Program {
                    back,
                    strafe: 60,
                    strafe_at: e + 20,
                    still: 0,
                    events: vec![
                        (e.saturating_sub(14), Act::At(6, feet(150), 1)),
                        (e.saturating_sub(4), Act::Jump(60)),
                        (e + 30, Act::Press(0)),
                        (e + 45, Act::Press(1)),
                    ],
                });
            }
        }
        Class::BloodMage => {
            for back in [300u16, 800] {
                let e = edge_frame(back);
                out.push(Program {
                    back,
                    strafe: 0,
                    strafe_at: 0,
                    still: 0,
                    events: vec![
                        (e.saturating_sub(1), Act::Jump(18)),
                        (e + 12, Act::At(3, lip(100, 0), 8)),
                    ],
                });
                out.push(Program {
                    back,
                    strafe: 40,
                    strafe_at: e + 5,
                    still: 0,
                    events: vec![
                        (e.saturating_sub(1), Act::Jump(18)),
                        (e + 20, Act::At(3, lip(50, 50), 15)),
                        (e + 70, Act::Dodge),
                    ],
                });
            }
        }
        Class::Champion => {
            for back in [500u16, 1500] {
                let e = edge_frame(back);
                out.push(Program {
                    back,
                    strafe: 0,
                    strafe_at: 0,
                    still: 0,
                    events: vec![
                        (e.saturating_sub(12), Act::Press(6)),
                        (e.saturating_sub(6), Act::Vault),
                    ],
                });
                out.push(Program {
                    back,
                    strafe: 40,
                    strafe_at: e + 20,
                    still: 0,
                    events: vec![
                        (e.saturating_sub(12), Act::Press(6)),
                        (e.saturating_sub(6), Act::Vault),
                        (e + 30, Act::Press(0)),
                        (e + 50, Act::Press(2)),
                    ],
                });
                out.push(Program {
                    back,
                    strafe: 0,
                    strafe_at: 0,
                    still: 0,
                    events: vec![(e.saturating_sub(1), Act::Jump(18)), (e, Act::Press(1))],
                });
            }
        }
        _ => {}
    }
    out
}

/// One mutation.
fn mutate(p: &Program, class: Class, kit: Kit, rng: &mut Rng) -> Program {
    let mut q = p.clone();
    let n = rng.range(1, 3);
    for _ in 0..n {
        match rng.range(0, 10) {
            0 => {
                // Start further or nearer, keeping every event where it was
                // relative to the edge.
                let by = rng.range(-300, 300);
                let back = (q.back as i32 + by).clamp(0, 3000) as u16;
                let shift = edge_frame(back) as i32 - edge_frame(q.back) as i32;
                q.back = back;
                q.strafe_at = (q.strafe_at as i32 + shift).max(0) as u16;
                for e in q.events.iter_mut() {
                    e.0 = (e.0 as i32 + shift).max(0) as u16;
                }
            }
            1 => q.strafe = (q.strafe as i32 + rng.range(-20, 20)).clamp(0, 120) as u8,
            2 => q.strafe_at = (q.strafe_at as i32 + rng.range(-15, 15)).max(0) as u16,
            9 => q.still = (q.still as i32 + rng.range(-20, 20)).max(0) as u16,
            3 | 4 if !q.events.is_empty() => {
                let i = rng.range(0, q.events.len() as i32 - 1) as usize;
                q.events[i].0 = (q.events[i].0 as i32 + rng.range(-6, 6)).max(0) as u16;
            }
            5 if !q.events.is_empty() => {
                let i = rng.range(0, q.events.len() as i32 - 1) as usize;
                q.events[i].1 = match q.events[i].1 {
                    Act::Jump(h) => Act::Jump((h as i32 + rng.range(-8, 8)).clamp(1, 90) as u8),
                    Act::At(b, mut aim, h) => {
                        aim.fwd = (aim.fwd as i32 + rng.range(-80, 80)).clamp(-600, 900) as i16;
                        aim.up = (aim.up as i32 + rng.range(-80, 80)).clamp(-600, 900) as i16;
                        let h = if b == 3 {
                            (h as i32 + rng.range(-5, 5)).clamp(1, 30) as u8
                        } else {
                            h
                        };
                        Act::At(b, aim, h)
                    }
                    other => other,
                };
            }
            6 | 7 if q.events.len() < 9 => {
                let around = q
                    .events
                    .get(rng.range(0, q.events.len() as i32 - 1).max(0) as usize)
                    .map_or(edge_frame(q.back) as i32, |e| e.0 as i32);
                let at = (around + rng.range(-10, 40)).max(0) as u16;
                q.events.push((at, palette(class, kit, rng)));
            }
            8 if q.events.len() > 1 => {
                let i = rng.range(0, q.events.len() as i32 - 1) as usize;
                q.events.remove(i);
            }
            _ => {}
        }
    }
    q.events.sort_by_key(|e| e.0);
    q
}

/// **The search**: from `seeds`, `budget` runs of mutate-and-keep over an
/// elite of the best few. The best program and what it did.
pub fn search(stage: &Stage, kit: Kit, budget: usize, seed: u64) -> (Program, Outcome) {
    search_from(stage, kit, budget, seed, Vec::new())
}

/// The class's own tools, appended to a line at frame `at`: what a player
/// does at the end of a long jump that is not going to make it.
pub fn finishers(class: Class, at: u16) -> Vec<Vec<(u16, Act)>> {
    let lip = |fwd: i16, up: i16| Aim {
        base: Base::Lip,
        fwd,
        up,
    };
    match class {
        Class::ShadowReaver => [100i16, 300, 600]
            .iter()
            .flat_map(|fwd| {
                [18u16, 22].map(|d| {
                    vec![
                        (at, Act::At(1, lip(*fwd, 0), 2)),
                        (at + d, Act::Dash),
                        (at + d + 14, Act::Jump(18)),
                    ]
                })
            })
            .collect(),
        Class::BloodMage => [(50i16, 50i16, 8u8), (100, 0, 15), (0, 100, 25)]
            .iter()
            .map(|(f, u, h)| vec![(at, Act::At(3, lip(*f, *u), *h))])
            .collect(),
        Class::Champion => vec![vec![(at, Act::Press(6))]],
        _ => Vec::new(),
    }
}

/// [`search`], with more programs to start from: a line found with the
/// shared blocks, say, which the whole kit then tries to finish better.
pub fn search_from(
    stage: &Stage,
    kit: Kit,
    budget: usize,
    seed: u64,
    extra: Vec<Program>,
) -> (Program, Outcome) {
    let mut rng = Rng(seed ^ 0x9E37_79B9_7F4A_7C15);
    let mut starts = seeds(stage.class, kit);
    for base in extra {
        let end = base.events.last().map_or(0, |e| e.0);
        starts.push(base.clone());
        for at in [end / 2, end.saturating_sub(10), end, end + 15] {
            for tail in finishers(stage.class, at) {
                let mut p = base.clone();
                p.events.extend(tail);
                p.events.sort_by_key(|e| e.0);
                starts.push(p);
            }
        }
    }
    let elite = elite_of(stage, kit, budget, &mut rng, starts);
    let (p, o) = elite.into_iter().next().expect("seeds are never empty");
    (p, o)
}

/// [`search_from`], keeping the best few rather than the best: what a caller
/// that wants the most forgiving of several lines picks from.
pub fn search_elite(
    stage: &Stage,
    kit: Kit,
    budget: usize,
    seed: u64,
    extra: Vec<Program>,
) -> Vec<(Program, Outcome)> {
    let mut rng = Rng(seed ^ 0x9E37_79B9_7F4A_7C15);
    let mut starts = seeds(stage.class, kit);
    starts.extend(extra);
    elite_of(stage, kit, budget, &mut rng, starts)
}

fn elite_of(
    stage: &Stage,
    kit: Kit,
    budget: usize,
    rng: &mut Rng,
    starts: Vec<Program>,
) -> Vec<(Program, Outcome)> {
    let mut rng = *rng;
    let mut elite: Vec<(i64, Program, Outcome)> = starts
        .into_iter()
        .map(|p| {
            let o = run(stage, &p);
            (score(stage, &o), p, o)
        })
        .collect();
    elite.sort_by_key(|e| std::cmp::Reverse(e.0));
    let keep = 6;
    elite.truncate(keep);
    for _ in 0..budget {
        let pick = if rng.chance(3) {
            0
        } else {
            rng.range(0, elite.len() as i32 - 1) as usize
        };
        let child = mutate(&elite[pick].1, stage.class, kit, &mut rng);
        let o = run(stage, &child);
        let s = score(stage, &o);
        if s > elite[elite.len() - 1].0 && !elite.iter().any(|e| e.1 == child) {
            elite.push((s, child, o));
            elite.sort_by_key(|e| std::cmp::Reverse(e.0));
            elite.truncate(keep);
        }
    }
    elite.into_iter().map(|(_, p, o)| (p, o)).collect()
}

/// How far either way [`window`] moves an input: fifteen frames, a quarter
/// of a second, so a window of 31 is an input that does not care.
pub const fn reach() -> i32 {
    15
}

/// **How forgiving a program is**: for each of its events, how many frames
/// of the 31 from fifteen early to fifteen late it can be pressed on and
/// still do the job -- landing, and on a lane landing within `slack` of the
/// same gap. The tightest event's count, and which event that is.
pub fn window(stage: &Stage, prog: &Program, slack: Fx) -> (u32, usize) {
    let base = run(stage, prog);
    let mut tightest = (u32::MAX, 0);
    for i in 0..prog.events.len() {
        let mut ok = 0;
        for by in -reach()..=reach() {
            let o = run(stage, &prog.shifted(i, by));
            let good = o.landed && (!stage.measure || o.gap.raw() >= base.gap.sub(slack).raw());
            if good {
                ok += 1;
            }
        }
        if ok < tightest.0 {
            tightest = (ok, i);
        }
    }
    if tightest.0 == u32::MAX {
        (17, 0)
    } else {
        tightest
    }
}
