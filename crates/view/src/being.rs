//! The being behind the Dual mage: her two bars, drawn.
//!
//! **The meter is the thing she holds back.** Two beings goad her hands, and
//! as their bars rise one incorporeal body takes shape behind her: **dark on
//! her left, light on her right** -- the same sides as the arms that feed
//! them -- each half condensing as its own bar fills. It is a gradient, not a
//! count: a quarter of a bar is a quarter of a body, faint, and the gap
//! between the two bars is the gap between the two halves, readable across
//! the arena by both players. Full, it is about to break out; ascension is
//! all of it, burning.
//!
//! **It arrives limb first** (the owner, 2026-10-25). The shoulder and the
//! fist come first, then the arm between them and the torso, then the head
//! last: a presence that is mostly a blow to begin with, and only at the top
//! of the bar has a face. [`presence`] is that order, and the shader that
//! draws it (`game/src/being.wgsl`) is the same function written again on the
//! GPU; `view/tests/being.rs` holds the order here.
//!
//! **It is big, and it rises out of the seal.** Its hips are at her feet,
//! where the seal on the floor is, and its chin at her forehead, so the
//! whole of it above the seal is torso, arms and head, and below the seal
//! there is nothing: it dissolves into the floor at the waist. [`scale`]
//! works the size out of her own skeleton, so a retuned body keeps the
//! proportion.
//!
//! **It casts with her, a beat behind** (2026-10-25). It is technically the
//! thing doing the casting, so it is posed with her own pose from
//! [`LAG_SECONDS`] ago: the same throw, larger, landing a moment after hers.
//! **Its fists are up** ([`guarded`]): its shoulders are at her chest, so an
//! arm hanging at rest would put the fist -- the first thing there is of it
//! -- under the floor. Its arms never drop below a guard, fists raised in
//! front of her; what she throws higher than that, it throws too.
//!
//! The rule the overlay lives by holds here in the other direction: what is
//! drawn *is* the bar, by construction, and `view/tests/being.rs` says so.
//! The being is drawn and never tested against; it is not a hurtbox.

use crate::pose::Pose;
use crate::skeleton::{Joint, Skeleton, solve};
use sim::aim::Hand;
use sim::class::Force;
use sim::state::Player;

/// How full each half of the being is, 0 to 1, and whether she is ascending.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Being {
    /// The half on her left, fed by the dark bar.
    pub dark: f32,
    /// The half on her right, fed by the light bar.
    pub light: f32,
    /// Ascending: the whole of it, both halves, whatever the bars say.
    pub ascending: bool,
}

impl Being {
    /// How full a force's half is.
    pub fn of(&self, force: Force) -> f32 {
        match force {
            Force::Dark => self.dark,
            Force::Light => self.light,
        }
    }

    /// Whether any of it is there at all.
    pub fn shown(&self) -> bool {
        self.dark > 0.0 || self.light > 0.0
    }
}

/// How much of one half a bar of `bar` shows, 0 to 1: a straight share of
/// the way to the top tier (`sim::dual::Tier::Wings`), which is where the
/// HUD's track reads full. Linear, so the half thickens as steadily as the
/// bar it is fed by, and full exactly when that tier is reached.
pub fn fill(bar: sim::Fx) -> f32 {
    let top = sim::dual::Tier::Wings.threshold().max(1) as f32;
    (bar.to_f32_for_render() / top).clamp(0.0, 1.0)
}

/// The being this fighter has behind her, if she has bars. `None` for the
/// other five classes, which is what hides it rather than drawing nothing.
pub fn being(p: &Player) -> Option<Being> {
    let (dark, light) = sim::dual::bars(p)?;
    let ascending = sim::dual::ascending(p);
    if ascending {
        return Some(Being {
            dark: 1.0,
            light: 1.0,
            ascending,
        });
    }
    Some(Being {
        dark: fill(dark),
        light: fill(light),
        ascending,
    })
}

/// Which arm a force's half grows from. The simulation's own answer, so the
/// dark half is on the side the dark auto comes out of whatever else changes.
pub const fn hand_of(force: Force) -> Hand {
    match force {
        Force::Dark => Hand::Left,
        Force::Light => Hand::Right,
    }
}

/// The order it arrives in: the fill at which each part of a half starts to
/// show. The shoulder and the fist at once, the arm between them and the
/// torso with it, the head last.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Order {
    /// Where the shoulder and the fist start: the first thing there is.
    pub first: f32,
    /// What is furthest from either -- the middle of the arm, the torso at
    /// the hips -- starts here.
    pub body: f32,
    /// The head starts here, and nothing is later.
    pub head: f32,
    /// How much more bar a part takes from starting to show to being
    /// whole: the width of the gradient, so the body condenses rather than
    /// switching on.
    pub soft: f32,
}

/// The order, as tuned. `head + soft` is one: the head is whole exactly as
/// the bar is full.
pub const ORDER: Order = Order {
    first: 0.0,
    body: 0.45,
    head: 0.72,
    soft: 0.28,
};

/// How far from the shoulder or the fist, in metres of *her* size (the
/// being's are [`scale`] times as many), a point has to be to arrive with
/// `ORDER.body` rather than with the shoulder: about half an arm.
pub const REACH: f32 = 0.42;

/// How much of a part at a point is there, 0 to 1. `reach` is how far the
/// point is from the nearer of that half's shoulder and fist, as a share of
/// [`REACH`]; `head` how much of the point is head, 0 to 1; `fill` the half's
/// fill. **The shader is this function** (`being.wgsl`, `presence`).
pub fn presence(reach: f32, head: f32, fill: f32) -> f32 {
    let o = ORDER;
    let body = o.first + (o.body - o.first) * reach.clamp(0.0, 1.0);
    let start = body + (o.head.max(body) - body) * head.clamp(0.0, 1.0);
    smoothstep(start, start + o.soft, fill)
}

fn smoothstep(from: f32, to: f32, x: f32) -> f32 {
    let t = ((x - from) / (to - from)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// How far its pose runs behind hers: long enough to see that it follows,
/// short enough that it is still the same throw.
pub const LAG_SECONDS: f32 = 0.12;

/// The lowest its upper arms hang, in degrees forward of straight down.
pub const GUARD_SWING: f32 = 55.0;

/// The least its elbows are bent, in degrees.
pub const GUARD_ELBOW: f32 = 80.0;

/// Her pose, as the being holds it: everything hers, except that neither
/// upper arm hangs lower than [`GUARD_SWING`] and neither elbow is straighter
/// than [`GUARD_ELBOW`]. A larger reach of hers is kept, so a throw that
/// raises an arm past the guard raises the being's too.
pub fn guarded(pose: &Pose) -> Pose {
    let mut out = *pose;
    let rad = core::f32::consts::PI / 180.0;
    for (arm, floor) in [
        (Joint::ArmL, GUARD_SWING),
        (Joint::ArmR, GUARD_SWING),
        (Joint::ForearmL, GUARD_ELBOW),
        (Joint::ForearmR, GUARD_ELBOW),
    ] {
        let mut a = out.angles(arm);
        a[0] = a[0].max(floor * rad);
        out.set_angles(arm, a);
    }
    out
}

/// How far behind her it stands, in metres: enough that she is in front of
/// its body rather than inside it.
pub const BEHIND: f32 = 0.18;

/// How high on her the being's chin comes, as a share of her height: her
/// forehead.
pub const CHIN_AT: f32 = 0.93;

/// How many times her size the being is, so that **its hips are at her feet
/// and its chin at her forehead**. Its chin is where its head joint sits,
/// standing at rest; both heights are read off her own skeleton.
pub fn scale(skeleton: &Skeleton) -> f32 {
    let rest = solve(skeleton, &crate::pose::Pose::rest());
    let chin = rest.origin[Joint::Head.index()][1];
    (CHIN_AT * skeleton.total_height / (chin - skeleton.hip_height).max(0.05)).max(1.0)
}

/// The seal on the floor round her feet, where the being's hips are, as a
/// share of the being's height.
pub const SEAL_SHARE: f32 = 0.3;

/// The seal's radius, in metres: wide enough that the being's waist and its
/// hanging fists stand inside it, so it reads as what the being rises from.
pub fn seal_radius(skeleton: &Skeleton) -> f32 {
    SEAL_SHARE * scale(skeleton) * skeleton.total_height
}
