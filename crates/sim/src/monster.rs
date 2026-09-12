//! The Ridgeback: a creature you fight, and stand on.
//!
//! See `docs/design/monsters.md` for what the fight is and why. The short
//! version: its head, flanks and tail are armoured, its back is not, and the
//! only way to reach the soft strip along its spine for more than one swing is
//! to be standing on the animal.
//!
//! Three pieces live here.
//!
//! **A body made of boxes in the creature's own frame.** Parts are axis-aligned
//! in *body space*, so colliding a player with the creature is the same routine
//! as colliding them with the arena: transform into body space, resolve against
//! boxes, transform back. One collision rule, two frames of reference.
//!
//! **A pose that is five numbers.** A move moves the body and the body carries
//! whoever is on it, so the articulation has to be real -- but it does not have
//! to be a skeleton. Five scalars, each a pure function of `(action, frame)`,
//! are enough to make every move read from across the arena and enough to
//! generate honest shear under a rider. Being a pure function is what keeps
//! rollback exact: there is no pose to snapshot.
//!
//! **A control algorithm that glances rather than watching.** It samples the
//! player every few frames and extrapolates from that sample until the next
//! one, so it cannot read an input and a change of direction between glances is
//! a real thing to get good at.

use crate::DT;
use crate::fixed::{Fx, cos_turns, sin_turns};
use crate::math::V3;
use crate::tuning as t;

// ---------------------------------------------------------------------------
// The body
// ---------------------------------------------------------------------------

/// Part indices. Named consts rather than an enum because every per-part fact
/// in here is an array, and an array index is what an array wants.
pub const HEAD: usize = 0;
pub const NECK: usize = 1;
pub const BARREL: usize = 2;
pub const RIDGE: usize = 3;
pub const TAIL_BASE: usize = 4;
pub const TAIL_TIP: usize = 5;
pub const FORELEG_L: usize = 6;
pub const FORELEG_R: usize = 7;
pub const HINDLEG_L: usize = 8;
pub const HINDLEG_R: usize = 9;

pub const PARTS: usize = 10;

/// Not standing on anything. Shares the sentinel with `state::NOBODY` on
/// purpose: both mean "this index refers to nothing".
pub const NO_PART: u8 = u8::MAX;

pub const PART_NAMES: [&str; PARTS] = [
    "head",
    "neck",
    "barrel",
    "ridge",
    "tail",
    "tail tip",
    "left foreleg",
    "right foreleg",
    "left hindleg",
    "right hindleg",
];

/// One box of the creature, in body space: `+x` forward toward the head, `+y`
/// up, `+z` to the creature's right.
#[derive(Clone, Copy, Debug)]
pub struct Shape {
    pub min: V3,
    pub max: V3,
    /// You can stand on its top face.
    pub mountable: bool,
    /// You collide with it. The ridge is neither solid nor mountable: it is a
    /// soft strip of the back, not a wall, so you walk over it freely and that
    /// is the point.
    pub solid: bool,
    /// Follows the tail swing (1) or the head's reach (2), or neither (0).
    pub rides: u8,
    /// Broken by damage, and being broken costs the creature something.
    pub breakable: bool,
}

const fn v(x: (i32, i32), y: (i32, i32), z: (i32, i32)) -> V3 {
    V3::new(
        Fx::ratio(x.0, x.1),
        Fx::ratio(y.0, y.1),
        Fx::ratio(z.0, z.1),
    )
}

const fn part(min: V3, max: V3, mountable: bool, solid: bool, rides: u8, breakable: bool) -> Shape {
    Shape {
        min,
        max,
        mountable,
        solid,
        rides,
        breakable,
    }
}

/// Nothing follows anything; the tail parts; the head and neck.
const STILL: u8 = 0;
const WITH_TAIL: u8 = 1;
const WITH_HEAD: u8 = 2;

/// The creature's proportions, at scale one.
///
/// A constant for the same reason `arena::SOLIDS` is: it is a shape rather than
/// a feel number, it never changes during a fight, and it is therefore not part
/// of the rollback snapshot. The number you actually want to move while playing
/// -- *how big is it* -- is `tuning::monster_scale`, which multiplies all of
/// this, and that one is in the Oven.
pub const SHAPES: [Shape; PARTS] = [
    // Head and neck: armoured, and between them they stop a jump-in to the
    // ridge from the front.
    part(
        v((34, 10), (20, 10), (-6, 10)),
        v((45, 10), (30, 10), (6, 10)),
        false,
        true,
        WITH_HEAD,
        false,
    ),
    part(
        v((25, 10), (18, 10), (-5, 10)),
        v((34, 10), (28, 10), (5, 10)),
        false,
        true,
        WITH_HEAD,
        false,
    ),
    // The barrel. Its top face at 2.4 m is the walkable back, and its sides are
    // what a player on the ground is actually hitting.
    part(
        v((-24, 10), (13, 10), (-105, 100)),
        v((26, 10), (24, 10), (105, 100)),
        true,
        true,
        STILL,
        false,
    ),
    // The ridge: a soft strip lying along the top of the barrel. Neither solid
    // nor mountable -- it is a target volume, and walking over it is how you
    // get to stand next to it.
    part(
        v((-4, 10), (24, 10), (-5, 10)),
        v((12, 10), (275, 100), (5, 10)),
        false,
        false,
        STILL,
        false,
    ),
    // The tail. The base sits at 1.75 m, which is inside a jump, and that is
    // the way up for a player with no platform to hand.
    part(
        v((-40, 10), (105, 100), (-6, 10)),
        v((-24, 10), (175, 100), (6, 10)),
        true,
        true,
        WITH_TAIL,
        false,
    ),
    part(
        v((-54, 10), (85, 100), (-4, 10)),
        v((-40, 10), (135, 100), (4, 10)),
        false,
        true,
        WITH_TAIL,
        false,
    ),
    // Legs. The forelegs break, and a broken one costs it the turn.
    part(
        v((15, 10), (0, 1), (-10, 10)),
        v((23, 10), (14, 10), (-4, 10)),
        false,
        true,
        STILL,
        true,
    ),
    part(
        v((15, 10), (0, 1), (4, 10)),
        v((23, 10), (14, 10), (10, 10)),
        false,
        true,
        STILL,
        true,
    ),
    part(
        v((-23, 10), (0, 1), (-105, 100)),
        v((-13, 10), (14, 10), (-45, 100)),
        false,
        true,
        STILL,
        true,
    ),
    part(
        v((-23, 10), (0, 1), (45, 100)),
        v((-13, 10), (14, 10), (105, 100)),
        false,
        true,
        STILL,
        true,
    ),
];

/// Where the tail hinges, in body space. Part of the proportions, not a knob.
const TAIL_PIVOT_X: Fx = Fx::ratio(-24, 10);

/// The part's box at the creature's tuned scale.
pub fn shape(index: usize) -> Shape {
    let s = SHAPES[index.min(PARTS - 1)];
    let k = t::monster_scale();
    Shape {
        min: s.min.scale(k),
        max: s.max.scale(k),
        ..s
    }
}

/// How much of an attack's damage a part passes through. Below one is armour;
/// the ridge is above one, which is the whole reason to climb.
pub fn vulnerability(index: usize) -> Fx {
    match index {
        HEAD => t::vuln_head(),
        NECK => t::vuln_neck(),
        BARREL => t::vuln_barrel(),
        RIDGE => t::vuln_ridge(),
        TAIL_BASE => t::vuln_tail(),
        TAIL_TIP => t::vuln_tail_tip(),
        FORELEG_L | FORELEG_R => t::vuln_foreleg(),
        _ => t::vuln_hindleg(),
    }
}

// ---------------------------------------------------------------------------
// Moves
// ---------------------------------------------------------------------------

pub const MOVES: usize = 6;

pub const BITE: u8 = 0;
pub const STOMP: u8 = 1;
pub const SWEEP: u8 = 2;
pub const CHARGE: u8 = 3;
pub const SLAM: u8 = 4;
pub const SHAKE: u8 = 5;

pub const MOVE_NAMES: [&str; MOVES] = [
    "Bite",
    "Stomp",
    "Tail sweep",
    "Charge",
    "Rear and slam",
    "Shake",
];

/// One of the creature's moves, read live from the Oven.
///
/// The hit volume is a vertical cylinder **in body space**: an anchor in the
/// creature's own coordinates, a radius, and a height span. Body space rather
/// than world space because that is where the move is authored -- "the head,
/// one metre in front of it" does not change when the creature turns -- and
/// because it makes `follows` free: the anchor rides the same articulation the
/// parts do.
#[derive(Clone, Copy, Debug)]
pub struct Attack {
    pub name: &'static str,
    pub startup: u16,
    pub active: u16,
    pub recovery: u16,
    pub damage: i32,
    pub hit_x: Fx,
    pub hit_z: Fx,
    pub hit_radius: Fx,
    pub hit_low: Fx,
    pub hit_high: Fx,
    /// `WITH_HEAD` or `WITH_TAIL`, so the volume tracks the part that carries it.
    pub follows: u8,
    pub hitstun: u16,
    pub blockstun: u16,
    pub knockback: Fx,
    pub launch: Fx,
    pub unblockable: bool,
    /// Forward speed while the move is active. The charge is a move that
    /// travels; the rest are moves that happen where you are standing.
    pub advance: Fx,
    // ---- what the control algorithm reads ----
    /// The distance the move is for.
    pub ideal_range: Fx,
    /// How far either side of that it is still worth throwing. Outside this the
    /// move scores nothing at all, which is what stops it being thrown at a
    /// target it cannot reach.
    pub range_span: Fx,
    /// Cosine of the bearing the move wants its target on: one is dead ahead,
    /// zero is beside, minus one is behind. The tail sweep's is negative, which
    /// is the whole of how it knows what it is for.
    pub aim_cos: Fx,
    /// How far either side of that bearing it is still worth throwing. Wide for
    /// a sweep, narrow for a charge, irrelevant for the shake.
    pub aim_span: Fx,
    /// Base appetite for the move, before anything about the situation.
    pub weight: i32,
    /// Added per player standing on the creature. This is what makes a rider
    /// the only thing it cares about.
    pub rider_weight: i32,
}

impl Attack {
    pub const fn total(&self) -> u16 {
        self.startup + self.active + self.recovery
    }

    /// Frames the creature is still busy after the first active frame lands.
    pub const fn busy_after_contact(&self) -> i32 {
        (self.active as i32 - 1) + self.recovery as i32
    }

    /// How long the creature is helpless once the move is over its hit. This is
    /// the window the fight is built around, and the fight report measures it.
    pub const fn opening(&self) -> u16 {
        self.recovery
    }
}

/// Read a move from the live tuning store.
pub fn attack(kind: u8) -> Attack {
    use crate::oven::{self, MonsterField as F};
    let slot = (kind as usize).min(MOVES - 1);
    let raw = |f: F| oven::monster_field(slot, f);
    Attack {
        name: MOVE_NAMES[slot],
        startup: raw(F::Startup) as u16,
        active: raw(F::Active) as u16,
        recovery: raw(F::Recovery) as u16,
        damage: raw(F::Damage),
        hit_x: Fx::from_raw(raw(F::HitX)),
        hit_z: Fx::from_raw(raw(F::HitZ)),
        hit_radius: Fx::from_raw(raw(F::HitRadius)),
        hit_low: Fx::from_raw(raw(F::HitLow)),
        hit_high: Fx::from_raw(raw(F::HitHigh)),
        follows: raw(F::Follows) as u8,
        hitstun: raw(F::Hitstun) as u16,
        blockstun: raw(F::Blockstun) as u16,
        knockback: Fx::from_raw(raw(F::Knockback)),
        launch: Fx::from_raw(raw(F::Launch)),
        unblockable: raw(F::Unblockable) != 0,
        advance: Fx::from_raw(raw(F::Advance)),
        ideal_range: Fx::from_raw(raw(F::IdealRange)),
        range_span: Fx::from_raw(raw(F::RangeSpan)),
        aim_cos: Fx::from_raw(raw(F::AimCos)),
        aim_span: Fx::from_raw(raw(F::AimSpan)),
        weight: raw(F::Weight),
        rider_weight: raw(F::RiderWeight),
    }
}

/// Every move, live.
pub fn attacks() -> [Attack; MOVES] {
    [
        attack(0),
        attack(1),
        attack(2),
        attack(3),
        attack(4),
        attack(5),
    ]
}

// ---------------------------------------------------------------------------
// What it is doing
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Doing {
    /// Standing, walking, turning. The only state a move can start from.
    Prowl,
    Startup {
        kind: u8,
        left: u16,
    },
    Active {
        kind: u8,
        left: u16,
    },
    Recovery {
        kind: u8,
        left: u16,
    },
    /// A flinch. Short, and it interrupts whatever was happening.
    Flinch {
        left: u16,
    },
    /// Poise broken. The long window the climb is for.
    Toppled {
        left: u16,
    },
    Dead,
}

impl Doing {
    pub const fn attacking(self) -> Option<u8> {
        match self {
            Doing::Startup { kind, .. }
            | Doing::Active { kind, .. }
            | Doing::Recovery { kind, .. } => Some(kind),
            _ => None,
        }
    }

    /// Can a new move start?
    pub const fn free(self) -> bool {
        matches!(self, Doing::Prowl)
    }

    /// Is the creature currently punishable -- out of its hit and unable to
    /// answer? What the fight report counts as an opening.
    pub const fn open(self) -> bool {
        matches!(
            self,
            Doing::Recovery { .. } | Doing::Flinch { .. } | Doing::Toppled { .. }
        )
    }

    pub const fn tag(self) -> u32 {
        match self {
            Doing::Prowl => 0,
            Doing::Startup { .. } => 1,
            Doing::Active { .. } => 2,
            Doing::Recovery { .. } => 3,
            Doing::Flinch { .. } => 4,
            Doing::Toppled { .. } => 5,
            Doing::Dead => 6,
        }
    }

    pub const fn frames_left(self) -> u16 {
        match self {
            Doing::Prowl | Doing::Dead => 0,
            Doing::Startup { left, .. }
            | Doing::Active { left, .. }
            | Doing::Recovery { left, .. }
            | Doing::Flinch { left }
            | Doing::Toppled { left } => left,
        }
    }

    /// Frames since the current move began, and how long it runs in total.
    /// Zero-length when nothing is running.
    fn elapsed(self) -> (u16, u16) {
        let Some(kind) = self.attacking() else {
            return (0, 0);
        };
        let m = attack(kind);
        let into = match self {
            Doing::Startup { left, .. } => m.startup.saturating_sub(left),
            Doing::Active { left, .. } => m.startup + m.active.saturating_sub(left),
            Doing::Recovery { left, .. } => m.startup + m.active + m.recovery.saturating_sub(left),
            _ => 0,
        };
        (into, m.total())
    }
}

// ---------------------------------------------------------------------------
// The pose: five numbers, and no skeleton
// ---------------------------------------------------------------------------

/// How the creature is carrying itself this frame.
///
/// A pure function of `(what it is doing, how far into it)`, which is the
/// reason none of it is in the snapshot and the reason a rollback reproduces it
/// exactly. It is also what riders stand on, so these five numbers are the
/// whole of the buck: nothing anywhere tags a move "this one throws people".
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Pose {
    /// Added to the creature's steered yaw. The shake is entirely this.
    pub yaw_extra: Fx,
    /// Nose up, in turns.
    pub pitch: Fx,
    /// The whole body lifted or dropped.
    pub bob: Fx,
    /// An extra yaw applied to the tail alone, about the tail's own hinge.
    pub tail_swing: Fx,
    /// The head and neck sliding forward along the body's own axis.
    pub head_reach: Fx,
}

/// Which phase, and how far through it, as a fraction.
fn phase(doing: Doing) -> (u8, Fx) {
    let Some(kind) = doing.attacking() else {
        return (3, Fx::ZERO);
    };
    let m = attack(kind);
    let part = |left: u16, span: u16| {
        if span == 0 {
            return Fx::ONE;
        }
        Fx::from_int(span.saturating_sub(left) as i32).div(Fx::from_int(span as i32))
    };
    match doing {
        Doing::Startup { left, .. } => (0, part(left, m.startup)),
        Doing::Active { left, .. } => (1, part(left, m.active)),
        Doing::Recovery { left, .. } => (2, part(left, m.recovery)),
        _ => (3, Fx::ZERO),
    }
}

pub fn pose_of(doing: Doing) -> Pose {
    use crate::math::{lerp, smoothstep};
    let mut p = Pose::default();

    if let Doing::Toppled { left } = doing {
        // On its side. Pitched hard and dropped, going over quickly and coming
        // back up slowly -- a fall is fast and standing up is not.
        let total = t::topple_frames().max(1);
        let gone = total.saturating_sub(left);
        let down = if gone < t::topple_fall() {
            smoothstep(Fx::from_int(gone as i32).div(Fx::from_int(t::topple_fall().max(1) as i32)))
        } else if left < t::topple_rise() {
            smoothstep(Fx::from_int(left as i32).div(Fx::from_int(t::topple_rise().max(1) as i32)))
        } else {
            Fx::ONE
        };
        p.pitch = t::topple_pitch().mul(down);
        p.bob = t::topple_drop().mul(down).neg();
        return p;
    }
    if let Doing::Flinch { left } = doing {
        let span = Fx::from_int(t::flinch_frames().max(1) as i32);
        let at = Fx::ONE.sub(Fx::from_int(left as i32).div(span));
        p.pitch = t::flinch_pitch().mul(crate::math::arch(at));
        return p;
    }

    let Some(kind) = doing.attacking() else {
        return p;
    };
    let (which, at) = phase(doing);
    let s = smoothstep(at);
    let back = Fx::ONE.sub(s);

    match kind {
        BITE => {
            // Draws the head back, then throws it. The draw is the tell.
            let draw = t::bite_draw();
            let reach = t::bite_reach();
            p.head_reach = match which {
                0 => draw.neg().mul(s),
                1 => lerp(draw.neg(), reach, s),
                _ => reach.mul(back),
            };
            p.pitch = match which {
                0 => t::bite_rear().mul(s),
                1 => lerp(t::bite_rear(), t::bite_rear().neg(), s),
                _ => t::bite_rear().neg().mul(back),
            };
        }
        STOMP => {
            // The front end rises a little and comes down hard. Short enough
            // that the rise is most of what you get to read.
            let lift = t::stomp_lift();
            let drop = t::stomp_drop();
            p.pitch = match which {
                0 => lift.mul(s),
                1 => lerp(lift, drop.neg(), s),
                _ => drop.neg().mul(back),
            };
            p.bob = match which {
                1 => t::stomp_bob().neg().mul(s),
                2 => t::stomp_bob().neg().mul(back),
                _ => Fx::ZERO,
            };
        }
        SWEEP => {
            // Winds the tail one way and sweeps it the other. The body
            // counter-rotates, because a tail that heavy cannot swing without
            // the rest of the animal answering for it -- which is also what
            // shears anyone standing on the barrel.
            let wind = t::sweep_wind();
            let swing = t::sweep_swing();
            p.tail_swing = match which {
                0 => wind.mul(s),
                1 => lerp(wind, swing.neg(), s),
                _ => swing.neg().mul(back),
            };
            p.yaw_extra = p.tail_swing.neg().mul(t::sweep_counter());
        }
        CHARGE => {
            // Head down and level. The gallop is a small vertical cycle, which
            // is what stops a rider being able to ignore a charge.
            let lean = t::charge_lean();
            p.pitch = match which {
                0 => lean.neg().mul(s),
                1 => lean.neg(),
                _ => lean.neg().mul(back),
            };
            if which == 1 {
                let (into, _) = doing.elapsed();
                let cycles = Fx::from_int(t::charge_gallop());
                p.bob = t::charge_bounce().mul(sin_turns(at.mul(cycles)));
                let _ = into;
            }
        }
        SLAM => {
            // The long telegraph. It rears, hangs, and comes down. Riders on
            // the back feel the whole reversal at once, which is why this is
            // the move you leave for rather than brace against.
            let rear = t::slam_rear();
            let rise = t::slam_rise();
            p.pitch = match which {
                0 => rear.mul(s),
                1 => lerp(rear, t::slam_dip().neg(), s),
                _ => t::slam_dip().neg().mul(back),
            };
            p.bob = match which {
                0 => rise.mul(s),
                1 => lerp(rise, Fx::ZERO, s),
                _ => Fx::ZERO,
            };
        }
        _ => {
            // Shake. No hitbox at all: it is the buck, and only the buck.
            //
            // The startup is a **lean, not a shudder**. Oscillating through the
            // windup made the startup as violent as the move and threw riders
            // before the telegraph had finished being a telegraph -- which is
            // the one thing a long startup is for. It leans over, shakes, and
            // settles back.
            //
            // The three pieces join at the same value on purpose. A step in the
            // pose between two frames is an arbitrarily large acceleration, and
            // an arbitrarily large acceleration throws everyone regardless of
            // what the move was supposed to do.
            let amount = t::shake_yaw();
            let cycles = t::shake_cycles();
            let ramp = t::shake_ramp();
            p.yaw_extra = match which {
                0 => amount.mul(s),
                1 => amount.mul(cos_turns(at.mul(cycles))),
                _ => amount.mul(cos_turns(cycles)).mul(back),
            };
            p.pitch = match which {
                0 => Fx::ZERO,
                1 => t::shake_pitch().mul(sin_turns(at.mul(cycles).mul(ramp))),
                _ => t::shake_pitch().mul(sin_turns(cycles.mul(ramp))).mul(back),
            };
        }
    }
    p
}

// ---------------------------------------------------------------------------
// Body space
// ---------------------------------------------------------------------------

/// Where the creature is and how it is oriented this frame -- everything needed
/// to move a point between its frame and the world's.
///
/// Yaw and pitch, and no roll. Nothing the creature does rolls it, and a third
/// angle would cost a full rotation matrix for a motion the move set never
/// makes.
#[derive(Clone, Copy, Debug)]
pub struct Stance {
    pub origin: V3,
    pub yaw: Fx,
    pub pitch: Fx,
    pub tail_swing: Fx,
    pub head_reach: Fx,
}

impl Stance {
    /// Body space to world. Pitch first, in the creature's own frame, then yaw.
    pub fn to_world(&self, local: V3) -> V3 {
        let (cp, sp) = (cos_turns(self.pitch), sin_turns(self.pitch));
        let x = local.x.mul(cp).sub(local.y.mul(sp));
        let y = local.x.mul(sp).add(local.y.mul(cp));
        let z = local.z;
        let (cy, sy) = (cos_turns(self.yaw), sin_turns(self.yaw));
        V3::new(
            self.origin.x.add(x.mul(cy)).sub(z.mul(sy)),
            self.origin.y.add(y),
            self.origin.z.add(x.mul(sy)).add(z.mul(cy)),
        )
    }

    /// World to body space. The exact inverse of `to_world`.
    pub fn to_body(&self, world: V3) -> V3 {
        let d = world.sub(self.origin);
        let (cy, sy) = (cos_turns(self.yaw), sin_turns(self.yaw));
        let x = d.x.mul(cy).add(d.z.mul(sy));
        let z = d.z.mul(cy).sub(d.x.mul(sy));
        let (cp, sp) = (cos_turns(self.pitch), sin_turns(self.pitch));
        V3::new(x.mul(cp).add(d.y.mul(sp)), d.y.mul(cp).sub(x.mul(sp)), z)
    }

    /// A direction, not a point: rotated but not translated. Used to ask where
    /// "up off the back" and "along the spine" point in the world, and to read
    /// an acceleration in the frame where the surface normal is simply `+y`.
    pub fn dir_to_body(&self, world: V3) -> V3 {
        self.to_body(world.add(self.origin))
    }

    pub fn dir_to_world(&self, local: V3) -> V3 {
        self.to_world(local).sub(self.origin)
    }

    /// Apply the articulation a part carries: the tail's swing about its hinge,
    /// or the head's slide along the body axis.
    pub fn articulate(&self, rides: u8, local: V3) -> V3 {
        match rides {
            WITH_TAIL => self.swing_tail(local, self.tail_swing),
            WITH_HEAD => V3::new(local.x.add(self.head_reach), local.y, local.z),
            _ => local,
        }
    }

    /// Undo it, so a landing spot can be recorded in the part's own rest frame.
    pub fn unarticulate(&self, rides: u8, local: V3) -> V3 {
        match rides {
            WITH_TAIL => self.swing_tail(local, self.tail_swing.neg()),
            WITH_HEAD => V3::new(local.x.sub(self.head_reach), local.y, local.z),
            _ => local,
        }
    }

    fn swing_tail(&self, local: V3, by: Fx) -> V3 {
        let pivot = TAIL_PIVOT_X.mul(t::monster_scale());
        let dx = local.x.sub(pivot);
        let (c, s) = (cos_turns(by), sin_turns(by));
        V3::new(
            pivot.add(dx.mul(c)).sub(local.z.mul(s)),
            local.y,
            dx.mul(s).add(local.z.mul(c)),
        )
    }
}

// ---------------------------------------------------------------------------
// The creature
// ---------------------------------------------------------------------------

/// What the creature is allowed to know about one fighter.
///
/// A deliberately small window. The control algorithm is handed this and
/// nothing else -- no buttons, no action state, no frame counters -- so
/// "it reads your inputs" is not a thing that can accidentally become true.
#[derive(Clone, Copy, Debug, Default)]
pub struct Quarry {
    pub pos: V3,
    pub vel: V3,
    pub alive: bool,
    /// Standing on the creature.
    pub aboard: bool,
}

/// Where its attention is, and what it remembers seeing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Brain {
    /// The last sample it took, and when the next one is due.
    pub seen: V3,
    pub seen_vel: V3,
    pub target: u8,
    pub glance_left: u16,
    /// The pause between moves. Without one it is a chain gun.
    pub think_left: u16,
    pub last_move: u8,
    /// Frames of variety penalty left on `last_move`.
    pub repeat_left: u16,
    /// Deterministic, advanced only from inside the tick, and part of the
    /// snapshot -- so a rollback replays the same choices.
    pub rng: u32,
}

impl Default for Brain {
    fn default() -> Brain {
        Brain {
            seen: V3::ZERO,
            seen_vel: V3::ZERO,
            target: 0,
            glance_left: 0,
            think_left: 0,
            last_move: NO_PART,
            repeat_left: 0,
            // Any odd seed. Xorshift is stuck at zero, and one is as good a
            // starting point as any other.
            rng: 0x2545_F491,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Monster {
    pub pos: V3,
    /// Steered facing, in turns. The pose's `yaw_extra` is added on top and is
    /// not stored, because it is a pure function of the action.
    pub yaw: Fx,
    /// Turns per second. Having a rate at all is what gives the creature mass:
    /// it cannot reverse a swing instantly, so it overshoots, and the overshoot
    /// is the window a player cuts back across its nose for.
    pub yaw_rate: Fx,
    /// Forward speed along its facing. It does not strafe; a quadruped that
    /// could would make its turn limit meaningless.
    pub speed: Fx,
    pub health: i32,
    /// Fills with ridge damage. Full, it goes over.
    pub poise: i32,
    pub part_health: [i32; PARTS],
    pub doing: Doing,
    pub brain: Brain,
    /// The current active window has already connected.
    pub hit_used: bool,
}

impl Default for Monster {
    fn default() -> Monster {
        Monster::new()
    }
}

impl Monster {
    pub fn new() -> Monster {
        Monster {
            pos: V3::ZERO,
            yaw: Fx::ZERO,
            yaw_rate: Fx::ZERO,
            speed: Fx::ZERO,
            health: t::monster_health(),
            poise: 0,
            part_health: [t::limb_health(); PARTS],
            doing: Doing::Prowl,
            brain: Brain::default(),
            hit_used: false,
        }
    }

    /// On its side, with its options gone.
    ///
    /// The creature's answer to `state::Player::disabled`, and drawn on the
    /// same line: a topple is the long window the whole climb exists to earn,
    /// and a flinch is the cheap one that happens whenever it is hit hard
    /// enough. Only the earned one counts.
    pub fn disabled(&self) -> bool {
        matches!(self.doing, Doing::Toppled { .. })
    }

    pub fn alive(&self) -> bool {
        self.health > 0
    }

    /// A leg with no health left. The creature turns worse toward a broken one.
    pub fn broken(&self, part: usize) -> bool {
        SHAPES[part].breakable && self.part_health[part] <= 0
    }

    pub fn pose(&self) -> Pose {
        pose_of(self.doing)
    }

    pub fn stance(&self) -> Stance {
        let p = self.pose();
        Stance {
            origin: V3::new(self.pos.x, self.pos.y.add(p.bob), self.pos.z),
            yaw: self.yaw.add(p.yaw_extra),
            pitch: p.pitch,
            tail_swing: p.tail_swing,
            head_reach: p.head_reach,
        }
    }

    /// Body-space position of a world point, with the part's articulation
    /// removed -- the point in the part's own rest frame, which is what a rider
    /// holds on to.
    pub fn rest_frame(&self, part: usize, world: V3) -> V3 {
        let s = self.stance();
        s.unarticulate(SHAPES[part].rides, s.to_body(world))
    }

    /// The world position of a point held in a part's rest frame.
    pub fn world_of(&self, part: usize, rest: V3) -> V3 {
        let s = self.stance();
        s.to_world(s.articulate(SHAPES[part].rides, rest))
    }

    /// The mountable part a body at `world` is standing on, if any.
    ///
    /// Mounting is landing: the feet have to be within `mount_snap` of a
    /// mountable top face and over its footprint. No button, because a surface
    /// is a surface.
    ///
    /// **The same question answers "are you still on it".** How far you may
    /// overhang an edge is `edge_grace`, as a fraction of your own width, and
    /// it is applied here rather than in a second test -- two rules for
    /// standing on something is how a rider ends up hovering off the side.
    pub fn surface_under(&self, world: V3, body_radius: Fx) -> Option<(usize, Fx)> {
        let s = self.stance();
        let lip = body_radius.mul(t::edge_grace());
        let mut best: Option<(usize, Fx)> = None;
        for (index, part) in SHAPES.iter().enumerate() {
            if !part.mountable {
                continue;
            }
            let sh = shape(index);
            let rest = s.unarticulate(part.rides, s.to_body(world));
            let over = rest.x.raw() > sh.min.x.sub(lip).raw()
                && rest.x.raw() < sh.max.x.add(lip).raw()
                && rest.z.raw() > sh.min.z.sub(lip).raw()
                && rest.z.raw() < sh.max.z.add(lip).raw();
            if !over {
                continue;
            }
            let gap = rest.y.sub(sh.max.y);
            if gap.raw() < t::mount_snap().neg().raw() || gap.raw() > t::mount_snap().raw() {
                continue;
            }
            if best.is_none_or(|(_, top)| sh.max.y.raw() > top.raw()) {
                best = Some((index, sh.max.y));
            }
        }
        best
    }
}

// ---------------------------------------------------------------------------
// Collision, in the creature's own frame
// ---------------------------------------------------------------------------

/// What resolving a body against the creature did to it.
#[derive(Clone, Copy, Debug)]
pub struct Contact {
    pub pos: V3,
    /// The part it came to rest on top of, if any.
    pub landed: Option<usize>,
    /// It was pushed sideways -- it walked into the animal.
    pub shoved: bool,
}

/// Distance from a point to a box in the horizontal plane. Zero inside.
fn flat_gap(x: Fx, z: Fx, min: V3, max: V3) -> Fx {
    let dx = min.x.sub(x).max(x.sub(max.x)).max(Fx::ZERO);
    let dz = min.z.sub(z).max(z.sub(max.z)).max(Fx::ZERO);
    V3::new(dx, Fx::ZERO, dz).flat_len()
}

/// Signed push needed to leave a span by the nearer edge. Same rule as the
/// arena's, and for the same reason: least penetration is the only resolution
/// that does not teleport a body through a thin box.
fn out_of(v: Fx, lo: Fx, hi: Fx) -> Fx {
    let low = lo.sub(v);
    let high = hi.sub(v);
    if high.abs().raw() < low.abs().raw() {
        high
    } else {
        low
    }
}

impl Monster {
    /// Push a body out of the creature's solid parts.
    ///
    /// The whole routine runs in **body space**, where every part is an
    /// axis-aligned box, so this is the arena's own collision rule pointed at a
    /// different frame of reference. That is the reason the parts are authored
    /// in body space in the first place.
    pub fn resolve(&self, world: V3, body_radius: Fx, body_height: Fx) -> Contact {
        let s = self.stance();
        let mut rest_of_world = s.to_body(world);
        let mut landed = None;
        let mut shoved = false;

        for (index, part) in SHAPES.iter().enumerate() {
            if !part.solid {
                continue;
            }
            let sh = shape(index);
            // Articulated parts are resolved in their own swung frame, so a
            // tail that has moved is a wall where it now is rather than where
            // it was authored.
            let rides = part.rides;
            let mut p = s.unarticulate(rides, rest_of_world);

            let min_x = sh.min.x.sub(body_radius);
            let max_x = sh.max.x.add(body_radius);
            let min_z = sh.min.z.sub(body_radius);
            let max_z = sh.max.z.add(body_radius);
            let head = p.y.add(body_height);
            let inside = p.x.raw() > min_x.raw()
                && p.x.raw() < max_x.raw()
                && p.z.raw() > min_z.raw()
                && p.z.raw() < max_z.raw()
                && head.raw() > sh.min.y.raw()
                && p.y.raw() < sh.max.y.raw();
            if !inside {
                continue;
            }

            let px = out_of(p.x, min_x, max_x);
            let pz = out_of(p.z, min_z, max_z);
            let up = sh.max.y.sub(p.y);
            let down = head.sub(sh.min.y);
            let vertical = up.min(down).abs();

            if vertical.raw() <= px.abs().raw() && vertical.raw() <= pz.abs().raw() {
                if up.raw() <= down.raw() {
                    p.y = sh.max.y;
                    landed = Some(index);
                } else {
                    p.y = sh.min.y.sub(body_height);
                }
            } else if px.abs().raw() <= pz.abs().raw() {
                p.x = p.x.add(px);
                shoved = true;
            } else {
                p.z = p.z.add(pz);
                shoved = true;
            }
            rest_of_world = s.articulate(rides, p);
        }

        Contact {
            pos: s.to_world(rest_of_world),
            landed,
            shoved,
        }
    }

    /// Is a point inside the creature's solid parts, padded outward?
    ///
    /// For the camera. The rig's rule is that level geometry never gets between
    /// the eye and the fighter and that the arm pulls *in* rather than swinging
    /// away; a nine metre animal is level geometry for as long as it is
    /// standing between the two.
    pub fn contains(&self, world: V3, pad: Fx) -> bool {
        let s = self.stance();
        let body = s.to_body(world);
        SHAPES.iter().enumerate().any(|(index, part)| {
            if !part.solid {
                return false;
            }
            let sh = shape(index);
            let p = s.unarticulate(part.rides, body);
            p.x.raw() > sh.min.x.sub(pad).raw()
                && p.x.raw() < sh.max.x.add(pad).raw()
                && p.y.raw() > sh.min.y.sub(pad).raw()
                && p.y.raw() < sh.max.y.add(pad).raw()
                && p.z.raw() > sh.min.z.sub(pad).raw()
                && p.z.raw() < sh.max.z.add(pad).raw()
        })
    }

    /// World height of the highest solid part beneath a point, or zero.
    ///
    /// The companion to `arena::ground_under`, and used for the same thing: the
    /// camera treats a surface as something to rest on rather than something to
    /// dodge, and the creature's back is a surface.
    pub fn top_under(&self, world: V3) -> Fx {
        let s = self.stance();
        let body = s.to_body(world);
        let mut best = Fx::ZERO;
        for (index, part) in SHAPES.iter().enumerate() {
            if !part.solid {
                continue;
            }
            let sh = shape(index);
            let p = s.unarticulate(part.rides, body);
            let over = p.x.raw() > sh.min.x.raw()
                && p.x.raw() < sh.max.x.raw()
                && p.z.raw() > sh.min.z.raw()
                && p.z.raw() < sh.max.z.raw();
            if !over {
                continue;
            }
            // Back into the world: the body's own `y` is not a height once it
            // is pitched.
            let top = s
                .to_world(s.articulate(part.rides, V3::new(p.x, sh.max.y, p.z)))
                .y;
            if top.raw() > best.raw() {
                best = top;
            }
        }
        best
    }

    /// The attack volume out this frame, in body space: an anchor, a radius and
    /// a height span, with the articulation already applied.
    ///
    /// Exposed rather than kept private so the debug overlay draws *the volume
    /// the hit test uses*. An overlay that rebuilds the shape itself can drift
    /// from the rule it illustrates, and then it is confidently wrong at
    /// exactly the moment you are using it to work out why something missed.
    pub fn hit_volume(&self) -> Option<(V3, Fx, Fx, Fx)> {
        let Doing::Active { kind, .. } = self.doing else {
            return None;
        };
        let m = attack(kind);
        if m.damage <= 0 {
            return None;
        }
        let s = self.stance();
        let anchor = s.articulate(m.follows, V3::new(m.hit_x, Fx::ZERO, m.hit_z));
        Some((anchor, m.hit_radius, m.hit_low, m.hit_high))
    }

    /// Does the volume out this frame reach a body standing at `world`?
    ///
    /// A vertical cylinder, tested in body space. Height genuinely decides this
    /// one -- a stomp aimed at the ground does not reach someone standing on
    /// the creature's back -- which is the difference between a monster and a
    /// fighter, and the reason this is not the flat test `state::resolve_hit`
    /// uses.
    pub fn reaches(&self, world: V3, hurt_height: Fx, body_radius: Fx) -> bool {
        let Some((anchor, radius, low, high)) = self.hit_volume() else {
            return false;
        };
        let b = self.stance().to_body(world);
        let flat = V3::new(b.x.sub(anchor.x), Fx::ZERO, b.z.sub(anchor.z)).flat_len();
        if flat.raw() > radius.add(body_radius).raw() {
            return false;
        }
        b.y.add(hurt_height).raw() >= low.raw() && b.y.raw() <= high.raw()
    }

    /// Which part an attack touches, or `None`.
    ///
    /// When it touches more than one, the **softest** wins: the ridge lies over
    /// the barrel, and a player who reached the weak point has hit the weak
    /// point. Rewarding the aim rather than the array order is the only version
    /// of this that a player could predict.
    pub fn part_struck(&self, centre: V3, radius: Fx, body_height: Fx) -> Option<usize> {
        let s = self.stance();
        let feet = s.to_body(centre);
        let head = s.to_body(V3::new(centre.x, centre.y.add(body_height), centre.z));
        let low = feet.y.min(head.y);
        let high = feet.y.max(head.y);

        let mut best: Option<(usize, Fx)> = None;
        for (index, part) in SHAPES.iter().enumerate() {
            let sh = shape(index);
            let p = s.unarticulate(part.rides, feet);
            if flat_gap(p.x, p.z, sh.min, sh.max).raw() > radius.raw() {
                continue;
            }
            if high.raw() < sh.min.y.raw() || low.raw() > sh.max.y.raw() {
                continue;
            }
            let soft = vulnerability(index);
            if best.is_none_or(|(_, seen)| soft.raw() > seen.raw()) {
                best = Some((index, soft));
            }
        }
        best.map(|(i, _)| i)
    }

    /// Which part a *beam* passes through, and how far along it that happens.
    ///
    /// The Elementalist's auto is a ray rather than a volume at a point, so it
    /// asks a different question of the creature: not "which parts does this
    /// bubble overlap" but "which part does this line reach first". That also
    /// makes the softest-wins tiebreak in `part_struck` unnecessary here --
    /// that rule exists because a sphere sits inside several boxes at once and
    /// the player could not have chosen between them. A line can: the ridge is
    /// in front of the barrel from above and behind it from the side, and
    /// whichever the shot reaches first is the one that was aimed at.
    ///
    /// The whole test runs in **body space**, where every part is an
    /// axis-aligned box, which is the reason the parts are authored there.
    /// `swell` is the shot's own radius, added to the box.
    pub fn part_struck_along(
        &self,
        from: V3,
        dir: V3,
        limit: Fx,
        swell: Fx,
    ) -> Option<(usize, Fx)> {
        let s = self.stance();
        let start = s.to_body(from);
        let along = s.dir_to_body(dir);
        let out = V3::new(swell, swell, swell);

        let mut best: Option<(usize, Fx)> = None;
        for (index, part) in SHAPES.iter().enumerate() {
            let sh = shape(index);
            // The articulation is rigid, so undoing it on two points a unit
            // apart gives back the direction as well as the origin.
            let o = s.unarticulate(part.rides, start);
            let d = s.unarticulate(part.rides, start.add(along)).sub(o);
            let Some(dist) = crate::math::ray_hits_box(o, d, sh.min.sub(out), sh.max.add(out))
            else {
                continue;
            };
            if dist.raw() > limit.raw() {
                continue;
            }
            if best.is_none_or(|(_, seen)| dist.raw() < seen.raw()) {
                best = Some((index, dist));
            }
        }
        best
    }
}

// ---------------------------------------------------------------------------
// Damage
// ---------------------------------------------------------------------------

impl Monster {
    /// Land a hit on a part. Returns the damage that actually went in, after
    /// the part's own vulnerability, which is what the caller should show.
    pub fn take_hit(&mut self, part: usize, raw: i32) -> i32 {
        if !self.alive() {
            return 0;
        }
        let dealt = Fx::from_int(raw).mul(vulnerability(part)).to_int().max(0);
        self.health = (self.health - dealt).max(0);
        if SHAPES[part].breakable {
            self.part_health[part] = (self.part_health[part] - dealt).max(0);
        }
        if part == RIDGE {
            self.poise += dealt;
        }

        if self.health <= 0 {
            self.doing = Doing::Dead;
            return dealt;
        }
        if self.poise >= t::poise_max() && !matches!(self.doing, Doing::Toppled { .. }) {
            // Over it goes. This is the window the climb is for, so it resets
            // the pool rather than draining it: you earn the next one again.
            //
            // Unlike a flinch, this **does** interrupt a live hitbox. A flinch
            // may not, because a creature whose swing can be cancelled by being
            // hit is one you never have to trade with. Legs going out from
            // under it mid-bite is a different event, and one the player has
            // spent a whole climb earning.
            self.poise = 0;
            self.doing = Doing::Toppled {
                left: t::topple_frames(),
            };
            self.speed = Fx::ZERO;
            self.yaw_rate = Fx::ZERO;
            return dealt;
        }
        // A flinch interrupts, but never an active frame: a creature whose hit
        // could be cancelled by being hit is a creature you never have to
        // trade with, and trading is most of what the ground game is.
        let interruptible = !matches!(self.doing, Doing::Active { .. } | Doing::Toppled { .. });
        if dealt >= t::flinch_threshold() && interruptible {
            self.doing = Doing::Flinch {
                left: t::flinch_frames(),
            };
        }
        dealt
    }
}

// ---------------------------------------------------------------------------
// The control algorithm
// ---------------------------------------------------------------------------

impl Monster {
    /// Xorshift32. Deterministic, seeded from the snapshot, advanced only from
    /// inside the tick -- so a rollback re-rolls the same numbers.
    fn roll(&mut self) -> u32 {
        let mut x = self.brain.rng;
        if x == 0 {
            x = 0x2545_F491;
        }
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.brain.rng = x;
        x
    }

    /// Take a look, if one is due.
    ///
    /// Between glances the creature is working from stale information. That is
    /// the whole difficulty model: it cannot react to a button because it is
    /// not looking, and a player who changes direction between two glances has
    /// done something real.
    fn glance(&mut self, quarry: &[Quarry]) {
        if self.brain.glance_left > 0 {
            self.brain.glance_left -= 1;
            return;
        }
        self.brain.glance_left = t::glance_frames();

        // It looks at whoever is nearest and *not* on its back: there is
        // nothing to bite at up there, and if a second hunter is on the ground
        // they are the one worth facing. Only when everyone is aboard does it
        // fall back to a rider, at which point every attack scores badly and
        // the bucking moves win by default -- which is the behaviour we wanted
        // and did not have to write down twice.
        let nearest = |aboard_counts: bool| -> Option<usize> {
            let mut best: Option<(usize, Fx)> = None;
            for (i, q) in quarry.iter().enumerate() {
                if !q.alive || (!aboard_counts && q.aboard) {
                    continue;
                }
                let d = q.pos.sub(self.pos).flat_len();
                if best.is_none_or(|(_, seen)| d.raw() < seen.raw()) {
                    best = Some((i, d));
                }
            }
            best.map(|(i, _)| i)
        };
        let Some(pick) = nearest(false).or_else(|| nearest(true)) else {
            return;
        };
        self.brain.target = pick as u8;
        self.brain.seen = quarry[pick].pos;
        self.brain.seen_vel = quarry[pick].vel;
    }

    /// Where it thinks the target will be in `frames` frames.
    ///
    /// The horizon is the move's own startup, which is the physically right
    /// scaling and costs one knob instead of one per move: a lunge that takes
    /// half a second leads half a second's worth. `lead` is how much of that
    /// extrapolation it actually trusts -- at zero it swipes at where you were.
    pub fn lead_point(&self, frames: u16) -> V3 {
        let horizon = Fx::from_int(frames as i32).mul(DT).mul(t::lead());
        self.brain.seen.add(self.brain.seen_vel.scale(horizon))
    }

    /// How much the creature wants to throw a given move right now.
    ///
    /// Range and angle multiply rather than add, because a move with perfect
    /// spacing and the target behind it is not half a good idea.
    pub fn appetite(&self, kind: u8, riders: i32) -> i32 {
        let m = attack(kind);
        let aim = self.lead_point(m.startup);
        let to = V3::new(aim.x.sub(self.pos.x), Fx::ZERO, aim.z.sub(self.pos.z));
        let range = to.flat_len();
        let facing = V3::from_turns(self.yaw);
        let cos = facing.dot(to.normalized());

        let range_fit = Fx::ONE
            .sub(range.sub(m.ideal_range).abs().div(m.range_span))
            .clamp(Fx::ZERO, Fx::ONE);
        let aim_fit = Fx::ONE
            .sub(cos.sub(m.aim_cos).abs().div(m.aim_span))
            .clamp(Fx::ZERO, Fx::ONE);

        let mut score = Fx::from_int(m.weight).mul(range_fit).mul(aim_fit).to_int();
        score += riders * m.rider_weight;

        // Wounded animals commit harder, and they commit to bigger swings.
        let max = t::monster_health().max(1);
        let missing = ((max - self.health).max(0) * 100) / max;
        score += (t::hurt_aggression() * missing * m.damage) / 10_000;

        if self.brain.last_move == kind && self.brain.repeat_left > 0 {
            let span = t::variety_frames().max(1) as i32;
            score -= (t::variety_penalty() * self.brain.repeat_left as i32) / span;
        }
        score
    }

    /// Pick something to do, if anything is worth doing.
    ///
    /// Not the maximum. Everything scoring at least `decisiveness` of the best
    /// goes into a weighted draw: at full decisiveness it always throws the
    /// best move and is therefore a script you can memorise, and at zero it is
    /// noise. In between, the *distribution* is learnable while the next move
    /// is not, which is the only version of "hard but fair" that survives a
    /// player who has fought it fifty times.
    fn choose(&mut self, riders: i32) {
        let mut scores = [0i32; MOVES];
        let mut best = 0;
        for (kind, slot) in scores.iter_mut().enumerate() {
            *slot = self.appetite(kind as u8, riders);
            best = best.max(*slot);
        }
        if best <= 0 {
            return;
        }
        let cut = ((best as i64 * t::decisiveness() as i64) / 100) as i32;
        let total: i32 = scores.iter().filter(|s| **s >= cut && **s > 0).sum();
        if total <= 0 {
            return;
        }

        let mut ticket = (self.roll() % total as u32) as i32;
        let mut chosen = None;
        for (kind, score) in scores.iter().enumerate() {
            if *score < cut || *score <= 0 {
                continue;
            }
            ticket -= *score;
            if ticket < 0 {
                chosen = Some(kind as u8);
                break;
            }
        }
        let kind = chosen.unwrap_or(0);

        let m = attack(kind);
        self.doing = Doing::Startup {
            kind,
            left: m.startup,
        };
        self.hit_used = false;
        self.brain.last_move = kind;
        self.brain.repeat_left = t::variety_frames();
        self.brain.think_left = 0;
    }

    /// Turn. A proportional controller behind a rate limit and an acceleration
    /// limit.
    ///
    /// The rate limit is how fast it can turn. The **acceleration** limit is
    /// what gives it mass: it cannot reverse a swing instantly, so it overshoots
    /// when it has been turning hard, and that overshoot is the window a player
    /// gets for cutting back across its nose. A rate limit alone would not give
    /// them one.
    fn steer(&mut self) {
        if !self.doing.free() {
            // Committed, or down. Whatever swing was in progress bleeds off
            // rather than stopping dead, so a move does not visibly clamp the
            // animal mid-turn.
            self.yaw_rate = self.yaw_rate.mul(t::turn_settle());
            self.yaw = self.yaw.add(self.yaw_rate.mul(DT));
            return;
        }
        let aim = self.lead_point(t::prowl_lead());
        let want = crate::math::atan2_turns(aim.z.sub(self.pos.z), aim.x.sub(self.pos.x));
        let error = crate::math::wrap_turns(want.sub(self.yaw));

        // Turning toward a side is driven off the foreleg on that side, so
        // breaking one is something the player can see the consequence of.
        // Increasing yaw swings toward positive Z, which is the creature's own
        // right.
        let lame = if error.raw() > 0 {
            self.broken(FORELEG_R)
        } else {
            self.broken(FORELEG_L)
        };
        let mut cap = t::turn_rate_max();
        if lame {
            cap = cap.mul(t::turn_hurt());
        }

        let desired = error.mul(t::turn_gain()).clamp(cap.neg(), cap);
        let budget = t::turn_accel().mul(DT);
        let change = desired.sub(self.yaw_rate).clamp(budget.neg(), budget);
        self.yaw_rate = self.yaw_rate.add(change);
        self.yaw = self.yaw.add(self.yaw_rate.mul(DT));
    }

    /// Walk. Forward along its own facing, and never sideways -- a quadruped
    /// that could strafe would make its turn limit decorative.
    fn walk(&mut self) {
        let want = match self.doing {
            Doing::Active { kind, .. } => attack(kind).advance,
            Doing::Prowl => {
                let range = V3::new(
                    self.brain.seen.x.sub(self.pos.x),
                    Fx::ZERO,
                    self.brain.seen.z.sub(self.pos.z),
                )
                .flat_len();
                range
                    .sub(t::prowl_range())
                    .mul(t::approach_gain())
                    .clamp(t::monster_back().neg(), t::monster_walk())
            }
            _ => Fx::ZERO,
        };
        let budget = t::monster_accel().mul(DT);
        self.speed = self
            .speed
            .add(want.sub(self.speed).clamp(budget.neg(), budget));

        let forward = V3::from_turns(self.yaw);
        self.pos = self.pos.add(forward.scale(self.speed.mul(DT)));

        // It stays inside the arena and on the floor. Nothing in the move set
        // takes it off the ground, and a creature this size on a platform would
        // be a camera problem rather than a fight.
        let limit = crate::arena::ARENA_HALF.sub(t::monster_margin());
        self.pos.x = self.pos.x.clamp(limit.neg(), limit);
        self.pos.z = self.pos.z.clamp(limit.neg(), limit);
        self.pos.y = Fx::ZERO;
    }

    /// Advance the frame-data clock, exactly the way a fighter's does.
    fn tick_action(&mut self) {
        let rest = |m: &mut Monster| {
            m.brain.think_left = t::think_frames();
            Doing::Prowl
        };
        self.doing = match self.doing {
            Doing::Dead => Doing::Dead,
            Doing::Prowl => Doing::Prowl,
            Doing::Startup { kind, left } if left > 0 => Doing::Startup {
                kind,
                left: left - 1,
            },
            Doing::Startup { kind, .. } => {
                self.hit_used = false;
                Doing::Active {
                    kind,
                    left: attack(kind).active,
                }
            }
            Doing::Active { kind, left } if left > 0 => Doing::Active {
                kind,
                left: left - 1,
            },
            Doing::Active { kind, .. } => Doing::Recovery {
                kind,
                left: attack(kind).recovery,
            },
            Doing::Recovery { kind, left } if left > 0 => Doing::Recovery {
                kind,
                left: left - 1,
            },
            Doing::Recovery { .. } => rest(self),
            Doing::Flinch { left } if left > 0 => Doing::Flinch { left: left - 1 },
            Doing::Flinch { .. } => rest(self),
            Doing::Toppled { left } if left > 0 => Doing::Toppled { left: left - 1 },
            Doing::Toppled { .. } => rest(self),
        };
    }

    /// One tick of creature.
    pub fn step(&mut self, quarry: &[Quarry]) {
        if self.health <= 0 {
            self.doing = Doing::Dead;
            self.speed = Fx::ZERO;
            self.yaw_rate = Fx::ZERO;
            return;
        }
        self.glance(quarry);
        self.tick_action();

        let riders = quarry.iter().filter(|q| q.alive && q.aboard).count() as i32;
        if self.doing.free() {
            if self.brain.think_left > 0 {
                self.brain.think_left -= 1;
            } else {
                self.choose(riders);
            }
            // Poise comes back while it is on its feet, so a topple has to be
            // earned again rather than saved up across a whole fight.
            self.poise = (self.poise - t::poise_regen()).max(0);
        }
        self.brain.repeat_left = self.brain.repeat_left.saturating_sub(1);

        self.steer();
        self.walk();
    }
}
