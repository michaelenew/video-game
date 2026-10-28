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
//! **They are not her** (the owner, 2026-10-25): two bodies of their own, a
//! dark warrior of obsidian and a celestial knight, on a broader skeleton
//! than hers ([`skeleton`]), sized by [`scale`].
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
//! Between her casts it stands as itself ([`rest`]): arms down, shoulders
//! broad.
//!
//! The rule the overlay lives by holds here in the other direction: what is
//! drawn *is* the bar, by construction, and `view/tests/being.rs` says so.
//! The being is drawn and never tested against; it is not a hurtbox.

use crate::pose::Pose;
use crate::skeleton::{Build, Joint, Skeleton, skeleton_for, solve};
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

/// Each being's skeleton, dark then light: their own, not hers.
///
/// The knight's is the Champion's. The dark being's is **the body it is**:
/// its mesh is Blender Studio's realistic man, used as Blender made it
/// (the owner, 2026-10-28: "The blender model is definitely modified from
/// what comes directly from blender, put it back"), so the skeleton is
/// built to his joints rather than his body bent to a skeleton's -- bent to
/// the Champion's, his shoulders were pulled in five centimetres a side.
/// The numbers are his, measured off the mesh (`art/sources`), in metres
/// at his own 1.80 m.
pub fn skeleton(k: usize) -> Skeleton {
    if k == 1 {
        return skeleton_for(sim::Class::Champion);
    }
    dark()
}

/// Blender's man, measured: shoulder joints 0.198 out at 1.455 high; upper
/// arm 0.336, forearm 0.254; hip joints 0.082 out at 0.956 high; thigh
/// 0.441, shin 0.434, the ankle 0.081 off the floor.
fn dark() -> Skeleton {
    let mut sk = Skeleton::new(Build {
        shoulders: DARK_SHOULDER / 0.185,
        stance: DARK_HIP_OUT / 0.105,
        ..Build::REFERENCE
    });
    let (thigh, shin, ankle) = (0.441, 0.434, 0.081);
    let (upper, fore) = (0.336, 0.254);
    let hip = 0.05 + thigh + shin + ankle;
    let b = &mut sk.bones;
    b[Joint::Root.index()].offset[1] = hip;
    // The trunk shortened so his shoulders sit where his do.
    let shoulder_at = hip
        + b[Joint::Spine.index()].offset[1]
        + b[Joint::Chest.index()].offset[1]
        + b[Joint::ClavicleL.index()].offset[1];
    b[Joint::Chest.index()].offset[1] -= shoulder_at - DARK_SHOULDER_AT;
    for (arm, fore_j, hand, thigh_j, shin_j, foot) in [
        (
            Joint::ArmL,
            Joint::ForearmL,
            Joint::HandL,
            Joint::ThighL,
            Joint::ShinL,
            Joint::FootL,
        ),
        (
            Joint::ArmR,
            Joint::ForearmR,
            Joint::HandR,
            Joint::ThighR,
            Joint::ShinR,
            Joint::FootR,
        ),
    ] {
        b[arm.index()].length = upper;
        b[fore_j.index()].offset[1] = -upper;
        b[fore_j.index()].length = fore;
        b[hand.index()].offset[1] = -fore;
        b[thigh_j.index()].length = thigh;
        b[shin_j.index()].offset[1] = -thigh;
        b[shin_j.index()].length = shin;
        b[foot.index()].offset[1] = -shin;
    }
    sk.hip_height = hip;
    sk.total_height = hip
        + b[Joint::Spine.index()].offset[1]
        + b[Joint::Chest.index()].offset[1]
        + b[Joint::Neck.index()].offset[1]
        + b[Joint::Head.index()].offset[1]
        + b[Joint::Head.index()].length;
    sk
}

/// How far out his shoulder joints are, and how high; how far out his hips.
const DARK_SHOULDER: f32 = 0.198;
const DARK_SHOULDER_AT: f32 = 1.455;
const DARK_HIP_OUT: f32 = 0.082;

/// How far out from the body each being's arms hang at rest, in degrees,
/// dark then light: enough to clear the mass of the lats, which is what keeps
/// a heavily built figure's arms off its sides; the knight's further, its
/// shoulders broad and its arms out from its plate (the owner, 2026-10-27:
/// "bring the arms and shoulders out wider"). The art draws each being's
/// armour for its own (`art/lib/facet.py`, `rest_frame`).
pub const REST_SPREAD: [f32; 2] = [11.0, 18.0];

/// How bent its elbows are at rest, in degrees: soft, not straight.
pub const REST_ELBOW: f32 = 12.0;

/// **Its own resting stance** (the owner, 2026-10-26: "arms down, shoulders
/// broad. That's the general resting stance"): standing straight, the
/// shoulders drawn back and open, the arms hanging just off the body. Not
/// hers: her idle is a caster's, and on a body this size it read as a cat
/// pretending to stand.
pub fn rest(k: usize) -> Pose {
    Pose::rest()
        .shoulders(0.0, REST_SPREAD[k], 0.0)
        .elbows(REST_ELBOW)
        .clavicle_l(-6.0, 0.0)
        .clavicle_r(-6.0, 0.0)
        .chest(-3.0, 0.0, 0.0)
}

/// How quickly it takes up her pose when she starts to cast, and lets it go
/// when she stops, as a share a second.
pub const CAST_IN: f32 = 9.0;
pub const CAST_OUT: f32 = 3.0;

/// Being `k`'s pose (0 dark, 1 light): [`rest`], turned toward `hers` -- her
/// pose, a beat late -- by `casting`, 0 at rest and 1 throwing with her.
pub fn stance(k: usize, hers: &Pose, casting: f32) -> Pose {
    rest(k).blend(hers, casting.clamp(0.0, 1.0))
}

/// How far behind her it stands, in metres: enough that she is in front of
/// its body rather than inside it.
pub const BEHIND: f32 = 0.18;

/// How high on her the being's chin comes, as a share of her height: her
/// forehead.
pub const CHIN_AT: f32 = 0.93;

/// How many times its own size a being built on `body` is drawn at, so that
/// **its hips are at her feet and its chin at her forehead** -- `her` is the
/// Dual mage's skeleton. Its chin is where its head joint sits, standing at
/// rest; every height is read off the two skeletons.
pub fn scale(her: &Skeleton, body: &Skeleton) -> f32 {
    let rest = solve(body, &crate::pose::Pose::rest());
    let chin = rest.origin[Joint::Head.index()][1];
    (CHIN_AT * her.total_height / (chin - body.hip_height).max(0.05)).max(1.0)
}

/// The seal on the floor round her feet, where the being's hips are, as a
/// share of the being's height.
pub const SEAL_SHARE: f32 = 0.3;

/// The seal's radius, in metres: wide enough that the being's waist and its
/// hanging fists stand inside it, so it reads as what the being rises from.
pub fn seal_radius(her: &Skeleton) -> f32 {
    SEAL_SHARE * scale(her, her) * her.total_height
}
