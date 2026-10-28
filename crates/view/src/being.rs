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
//! dark warrior of obsidian and a celestial knight, each on its own
//! skeleton ([`skeleton`]).
//!
//! **Two halves, either side of her** (the owner, 2026-10-28). Each being
//! stands with its centre line to her side -- the dark one's to her left,
//! the light's to her right ([`SIDE`]) -- and only its outer half is drawn,
//! fading into its own middle ([`FADE`]), so she stands in the gap between
//! a dark half and a light one. Each is as tall as she is at the shoulder
//! ([`scale`]) and comes out of the seal ([`SEAL_AT`]): nothing of it below.
//! Each half is there as far as its own bar has filled, in [`presence`]'s
//! order.
//!
//! **It moves as itself** ([`pose`]): a bound thing straining at what holds
//! it, not a puppet of hers. Her auto is its blow, thrown with its own arm
//! at her frames, so the fuller its bar the plainer it is that the being is
//! what hits.
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

/// Where she is in a move, as the being reads it: which move (`kind`, a
/// slot of her kit), how many frames into it, and its three phases' lengths;
/// and the frame the world is on, the being's own clock. All of it the
/// simulation's, so the being's motion is a function of the snapshot -- a
/// rollback redraws it exactly, and nothing pops.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Moment {
    pub kind: Option<u8>,
    pub at: u16,
    pub frames: (u16, u16, u16),
    pub clock: u32,
}

/// What `p` is doing, on world frame `clock`.
pub fn moment(p: &Player, clock: u32) -> Moment {
    use sim::state::Action;
    let (kind, at) = match p.action {
        Action::Startup { kind, left } => {
            let (s, _, _) = sim::moves::frames(sim::Class::DualMage, kind);
            (Some(kind), s.saturating_sub(left))
        }
        Action::Active { kind, left } => {
            let (s, a, _) = sim::moves::frames(sim::Class::DualMage, kind);
            (Some(kind), s + a.saturating_sub(left))
        }
        Action::Recovery { kind, left } => {
            let (s, a, r) = sim::moves::frames(sim::Class::DualMage, kind);
            (Some(kind), s + a + r.saturating_sub(left))
        }
        _ => (None, 0),
    };
    let frames = kind.map_or((1, 1, 1), |k| sim::moves::frames(sim::Class::DualMage, k));
    Moment {
        kind,
        at,
        frames,
        clock,
    }
}

/// Her autos' slots: the dark auto, thrown with her left arm, and the
/// light, with her right (`docs/design/kits/dual-mage.md`).
pub const DARK_AUTO: u8 = 0;
pub const LIGHT_AUTO: u8 = 4;

/// **Being `k`'s pose** (0 dark, 1 light), its own and never hers (the
/// owner, 2026-10-28: "each being needs its own animations ... it's bound,
/// not a puppet, and trying to change that"):
///
/// - **Her auto is its blow** ("the being is _the thing doing that auto_
///   with its own hands"): on its side's auto it draws its arm back behind
///   its shoulder and drives it through, at her frames -- the arm landing
///   as her ball leaves, which is where the ball starts (behind and outside
///   her shoulder, `orb, starts behind her`).
/// - **The other being's blow, it turns from**, and **her spells, it
///   channels**: its arm up and out, the hand open.
/// - **Between, it strains** ([`strain`]): a thing held, pulling against
///   what holds it.
///
/// Authored once, for a left arm, and put onto each being's outer arm
/// ([`drawn`]).
pub fn pose(k: usize, m: &Moment) -> Pose {
    let own = strain(k, m.clock);
    let side = |p: Pose| drawn(k, p);
    let Some(kind) = m.kind else {
        return own;
    };
    let mine = if k == 0 { DARK_AUTO } else { LIGHT_AUTO };
    let (s, a, r) = m.frames;
    let t = m.at as f32;
    let (s, a, r) = (s.max(1) as f32, a.max(1) as f32, r.max(1) as f32);
    if kind == mine {
        let wind = side(wind_up(k));
        let blow = side(blow(k));
        if t < s - 1.0 {
            own.blend(&wind, ease(t / (s - 1.0).max(1.0)))
        } else if t < s + a {
            // Driven through from the last frame of the wind-up to the first
            // active one -- as the ball leaves -- and held after.
            wind.blend(&blow, ease(((t - (s - 1.0)) / 1.5).min(1.0)))
        } else {
            blow.blend(&own, ease((t - s - a) / r))
        }
    } else if kind == DARK_AUTO || kind == LIGHT_AUTO {
        // The other one's blow: it is not its own to throw. It turns from
        // it, a beat of resistance, and goes back to straining.
        let up = ease((t / (s + a)).min(1.0));
        let down = if t > s + a {
            ease((t - s - a) / r)
        } else {
            0.0
        };
        own.blend(&side(recoil(k)), 0.6 * up * (1.0 - down))
    } else {
        // Anything else she throws: it reaches with her, in its own way.
        let up = ease((t / s).min(1.0));
        let down = if t > s + a {
            ease((t - s - a) / r)
        } else {
            0.0
        };
        own.blend(&side(channel(k)), up * (1.0 - down))
    }
}

fn ease(t: f32) -> f32 {
    smoothstep(0.0, 1.0, t)
}

/// The arm drawn back for the blow: the shoulder behind it, the chest
/// turned away, the elbow folded.
fn wind_up(k: usize) -> Pose {
    rest(k)
        .chest(-4.0, 4.0, -26.0)
        .spine(0.0, 0.0, -10.0)
        .shoulder_l(-38.0, 38.0, 0.0)
        .elbow_l(112.0)
        .clavicle_l(-14.0, 6.0)
        .wrist_l(10.0, 0.0, 0.0)
        .head(4.0, 0.0, -6.0)
}

/// The blow landed: the arm driven out level and straight ahead of its own
/// shoulder -- never across its chest, which is the half that is not drawn,
/// so the chest is turned a little away rather than into it -- the weight
/// forward.
fn blow(k: usize) -> Pose {
    rest(k)
        .root(6.0, 0.0, 0.0)
        .chest(4.0, -3.0, -6.0)
        .spine(2.0, 0.0, 0.0)
        .shoulder_l(84.0, 24.0, 0.0)
        .elbow_l(6.0)
        .clavicle_l(8.0, 2.0)
        .wrist_l(-28.0, 0.0, 0.0)
        .head(-4.0, 0.0, 6.0)
}

/// Turned from the other being's blow: the chest away, the arm drawn in.
fn recoil(k: usize) -> Pose {
    rest(k)
        .chest(-2.0, -6.0, -12.0)
        .shoulder_l(-10.0, REST_SPREAD[k] + 6.0, 0.0)
        .elbow_l(40.0)
        .head(0.0, 4.0, -16.0)
}

/// Reaching with her spells: the arm up and forward, open-handed.
fn channel(k: usize) -> Pose {
    rest(k)
        .chest(-6.0, -2.0, 8.0)
        .shoulder_l(62.0, 40.0, 0.0)
        .elbow_l(24.0)
        .clavicle_l(8.0, 10.0)
        .wrist_l(-40.0, 0.0, 0.0)
        .head(-8.0, 0.0, 0.0)
}

/// **Straining at its binding**, the being's own idle: it breathes, and
/// every few seconds it pulls -- the arm hauled back against what holds it,
/// the body leaning away from her, the head turning to her as if to the
/// one who holds the leash. The dark being heaves, quick and hard; the
/// knight tests it, slow and controlled. `clock` is the world's frame.
pub fn strain(k: usize, clock: u32) -> Pose {
    let t = clock as f32 / 60.0;
    let (breath_s, tug_s, phase, force) = if k == 0 {
        (2.4, 4.4, 0.0, 1.0)
    } else {
        (3.6, 7.0, 2.1, 0.55)
    };
    let tau = std::f32::consts::TAU;
    let breath = (tau * t / breath_s).sin();
    // A pull: sharp in, slow out.
    let tug = ((tau * t / tug_s + phase).sin().max(0.0)).powf(5.0) * force;
    let p = rest(k)
        .chest(-3.0 - 2.0 * breath - 6.0 * tug, -7.0 * tug, -8.0 * tug)
        .spine(0.0, -5.0 * tug, 0.0)
        .shoulder_l(-14.0 * tug, REST_SPREAD[k] + 12.0 * tug, 0.0)
        .elbow_l(REST_ELBOW + 55.0 * tug)
        .clavicle_l(-6.0 - 12.0 * tug, 2.0 * breath + 9.0 * tug)
        .wrist_l(18.0 * tug, 0.0, 0.0)
        .head(-3.0 * breath + 6.0 * tug, 5.0 * tug, -14.0 - 10.0 * tug);
    drawn(k, p)
}

/// A pose authored for the left arm, onto being `k`'s outer arm. Its
/// skeleton is solved as written, without the reflection a fighter's clips
/// go through (`crate::drawn_joint`), so the slot named left is drawn on
/// her right: the dark being, on her left, takes the pose mirrored.
fn drawn(k: usize, p: Pose) -> Pose {
    if k == 0 { p.mirrored() } else { p }
}

/// How far behind her each being stands, in metres of her size: enough
/// that she is in front of it rather than inside it.
pub const BEHIND: f32 = 0.22;

/// How far to her side each being's centre line is, in metres of her size:
/// the dark being's to her left, the light's to her right (the owner,
/// 2026-10-28: "the beings' body center lines probably need to be to the
/// left and right of the dual mage's"). Each keeps only its outer half, so
/// she stands in the gap between two halves.
pub const SIDE: f32 = 0.3;

/// How wide each half's fade into its own middle is, in metres of her size
/// ("each half should fade into the middle").
pub const FADE: f32 = 0.09;

/// How many times its own size a being built on `body` is drawn at, so that
/// **its shoulders are at her shoulders' height** (the owner, 2026-10-28),
/// both standing on her floor. Every height is read off the two skeletons.
pub fn scale(her: &Skeleton, body: &Skeleton) -> f32 {
    let shoulder = |sk: &Skeleton| {
        solve(sk, &crate::pose::Pose::rest()).origin[crate::drawn_joint(Joint::ArmL).index()][1]
    };
    shoulder(her) / shoulder(body).max(0.05)
}

/// Where the seal is, and so where each being comes out of: round her
/// waist, or on the floor round her feet. The owner has not chosen yet
/// (2026-10-28, "either around the mage's waist or feet (not sure yet)").
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SealAt {
    Waist,
    Feet,
}

pub const SEAL_AT: SealAt = SealAt::Waist;

/// How high the seal is, in metres: at her hip joint for the waist.
pub fn seal_height(her: &Skeleton, at: SealAt) -> f32 {
    match at {
        SealAt::Waist => her.hip_height + 0.06 * her.scale,
        SealAt::Feet => 0.0,
    }
}

/// The seal's radius, in metres: round both beings' waists where they come
/// out of it.
pub fn seal_radius(her: &Skeleton) -> f32 {
    (SIDE + 0.3) * her.scale
}
