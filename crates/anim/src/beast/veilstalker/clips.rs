//! Every animation the Veilstalker has, as poses and timing.
//!
//! Baked into `crates/sim/src/species/veilstalker/baked.rs` by
//! `cargo run -p anim --bin bake_beast -- --species veilstalker`.
//!
//! The house style is the Ridgeback's: **the windup is the move**, the hit is
//! short and enormous, and the recovery is long and readable. Three things
//! are this animal's own:
//!
//! 1. **The decloak is the windup.** Every strike starts from nothing and is
//!    drawn at the strength of its ramp (`fight::veil`), so whatever shape it
//!    makes in its first eight frames is what a person reads at half
//!    strength. The four strikes make four different shapes by frame eight
//!    (`veilstalker.md` §2): the lunge rears and sits back on its haunches;
//!    the spear stays flat and the tail goes up first; the rake does not rear
//!    at all -- the forelegs spread; the quills rear highest, the tail over
//!    the head.
//! 2. **Slinking is the rest pose.** Shoulder at 2.2 m, belly at a metre,
//!    head forward and level: what it is while it is cloaked, and so what it
//!    is when a paint mark or a mottled flank shows a piece of it.
//! 3. **The panic is on the floor.** Rolling and thrashing at a metre and a
//!    half: every class's swing lands on it, which is the whole of the window.

use super::{StalkerPose, at_frame, mark};
use crate::beast::{Key, Looseness, Pose, Recipe};
use crate::ease::Ease;
use sim::species::veilstalker::Clip;

pub fn all() -> Vec<Recipe> {
    vec![
        idle(),
        slink(),
        bound(),
        lunge(),
        spear(),
        rake(),
        rake2(),
        pounce(),
        quills(),
        smoke(),
        mimic(),
        retreat(),
        climb(),
        getup(),
        flinch(),
        stumble(),
        panic(),
        dead(),
    ]
}

/// Quick: the rake. A long animal's forelegs, snapped.
const QUICK: Looseness = Looseness {
    body: crate::beast::Feel::new(1.4, 0.95),
    neck: crate::beast::Feel::new(1.8, 0.75),
    tail: crate::beast::Feel::new(3.0, 0.55),
    legs: crate::beast::Feel::new(1.2, 0.95),
};

/// A rear: the body comes up as one piece and the tail swings under it as a
/// counterweight, a beat behind -- which is what makes it read as weight
/// standing up out of nothing rather than a prop being lifted.
const REAR: Looseness = Looseness {
    body: crate::beast::Feel::new(2.0, 0.90),
    neck: crate::beast::Feel::new(2.6, 0.72),
    tail: crate::beast::Feel::new(3.6, 0.55),
    legs: crate::beast::Feel::new(1.4, 0.92),
};

// ---------------------------------------------------------------------------
// Standing and moving
// ---------------------------------------------------------------------------

fn idle() -> Recipe {
    let breathe = |k: f32| {
        Pose::standing()
            .hips(0.0, 0.012 * k, 0.0)
            .spine(0.6 * k, 0.0, 0.0)
            .chest(0.8 * k, 0.0, 0.0)
            .neck(1.5 * k, 4.0 * k)
            .tail(3.0 * k, 10.0 * k)
    };
    Recipe::new(
        Clip::Idle,
        vec![
            Key::at(0.0, breathe(-1.0)),
            Key::at(0.5, breathe(1.0)),
            Key::at(0.75, breathe(0.2)),
        ],
        Looseness::BEAST,
    )
    .noting(
        "Low and breathing, the head turning a little, the long tail moving on \
         its own. It is drawn as nothing while it does this; the paint and a \
         mottled flank are what show it, so the idle is the slink's own \
         posture, never a stand."
            .to_string(),
    )
}

/// One leg at a point in its own cycle: planted for most of it and swinging
/// for the rest, solved onto the floor.
fn stride(p: Pose, which: usize, u: f32, reach: f32, lift: f32) -> Pose {
    const STANCE: f32 = 0.6;
    let u = u.rem_euclid(1.0);
    if u < STANCE {
        let k = u / STANCE;
        p.plant(which, reach * (1.0 - 2.0 * k), 0.0)
    } else {
        let k = (u - STANCE) / (1.0 - STANCE);
        let arc = (k * std::f32::consts::PI).sin();
        p.plant(which, reach * (2.0 * k - 1.0), lift * arc)
    }
}

fn slink() -> Recipe {
    // A lateral walk, long and low: the hind foot comes down into the
    // forefoot's print (`fight::trail` stamps the forefeet), so each side's
    // pair is half a cycle from the other side's and the hind lands where the
    // fore has just left.
    const OFFSET: [f32; 4] = [0.0, 0.5, 0.08, 0.58];
    let at = |t: f32| {
        let mut p = Pose::standing().hips(0.0, -0.06, 0.0);
        for (which, offset) in OFFSET.into_iter().enumerate() {
            p = stride(p, which, t + offset, 0.80, 0.22);
        }
        let wheel = t * std::f32::consts::TAU;
        let sway = 5.0 * (wheel + std::f32::consts::FRAC_PI_2).sin();
        p.root(0.6 * (2.0 * wheel).cos(), 0.0, sway * 0.4)
            .spine(0.0, sway * 0.8, 0.0)
            .chest(0.0, -sway * 0.7, sway * 0.5)
            .neck(-4.0, -sway * 0.8)
            .tail(-2.0, sway * 4.0)
    };
    Recipe::new(
        Clip::Slink,
        (0..8)
            .map(|i| {
                let t = i as f32 / 8.0;
                Key::eased(t, at(t), Ease::LINEAR)
            })
            .collect(),
        Looseness::GAIT,
    )
    .noting(
        "The slink: long and low, the spine snaking side to side, the head \
         held forward and still over it while everything behind it moves. Two \
         prints a stride, because the hind foot steps into the fore's -- the \
         trail is a line of pairs, three-toed, a stride and a half apart."
            .to_string(),
    )
}

fn bound() -> Recipe {
    // The rotary gallop of a long cat: the hind pair drive, the spine folds
    // under and throws, the forelegs reach out and catch. Seventeen metres a
    // second: it shimmers, so this is drawn faintly whenever it is drawn.
    const OFFSET: [f32; 4] = [0.40, 0.48, 0.0, 0.08];
    let at = |t: f32| {
        let mut p = Pose::standing();
        for (which, offset) in OFFSET.into_iter().enumerate() {
            p = stride(p, which, t + offset, 0.95, 0.50);
        }
        let wheel = t * std::f32::consts::TAU;
        let bob = 0.16 * (wheel - 0.9).sin() - 0.04;
        let flex = 14.0 * (wheel - 0.4).sin();
        p.hips(0.0, bob, 0.0)
            .root(-flex * 0.5, 0.0, 0.0)
            .spine(flex, 0.0, 0.0)
            .chest(flex * 0.8, 0.0, 0.0)
            .neck(-10.0 - flex * 0.6, 0.0)
            .tail(2.0 - flex * 0.6, 0.0)
    };
    Recipe::new(
        Clip::Bound,
        (0..8)
            .map(|i| {
                let t = i as f32 / 8.0;
                Key::eased(t, at(t), Ease::LINEAR)
            })
            .collect(),
        Looseness::GAIT,
    )
    .noting(
        "The bound: the spine folding under at the gather and opening right \
         out at the reach, the three-metre tail streaming behind as a rudder. \
         It is only ever seen as a shimmer, so the shape is everything: a \
         thing that size moving that fast."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// The rears
// ---------------------------------------------------------------------------

/// **The lunge's rear**: up and sat back on its haunches, forelegs drawn
/// up under the chest, head high and level -- a spring being wound. `k` is
/// how far through it is.
fn sat_back(k: f32) -> Pose {
    Pose::standing()
        .hips(-0.30 * k, -0.30 * k, 0.0)
        .root(32.0 * k, 0.0, 0.0)
        .spine(6.0 * k, 0.0, 0.0)
        .chest(4.0 * k, 0.0, 0.0)
        .neck(6.0 * k, 0.0)
        .head(-24.0 * k, 0.0, 0.0)
        // The tail up as a counterweight: the hips pitching up would drive
        // it into the snow.
        .tail(30.0 * k, 0.0)
        .tail_from_base(26.0 * k)
        .forelegs(-40.0 * k, 70.0 * k)
        .plant_hind(-0.25, 0.0)
}

/// The gather: dropped from the rear onto the forefeet, haunches loaded, head
/// low on the line it will fly along.
fn gathered() -> Pose {
    Pose::standing()
        .hips(-0.22, -0.40, 0.0)
        .root(10.0, 0.0, 0.0)
        .spine(-4.0, 0.0, 0.0)
        .chest(-8.0, 0.0, 0.0)
        .neck(-10.0, 0.0)
        .head(8.0, 0.0, 0.0)
        .tail(-6.0, 0.0)
        .plant_fore(0.30, 0.0)
        .plant_hind(-0.45, 0.0)
}

/// In the air along the floor: stretched out, jaws first, forelegs reaching,
/// hind legs trailing.
fn flying(lift: f32) -> Pose {
    Pose::rest(sim::species::veilstalker::bones::COUNT)
        .hips(0.0, lift, 0.0)
        .root(-4.0, 0.0, 0.0)
        .spine(3.0, 0.0, 0.0)
        .chest(4.0, 0.0, 0.0)
        .neck(-4.0, 0.0)
        .head(-6.0, 0.0, 0.0)
        .tail(6.0, 0.0)
        .forelegs(62.0, -20.0)
        .hindlegs(-58.0, 16.0)
}

/// Landed from a leap: forefeet far forward, chest low, hind end coming down
/// behind -- the pose the long recovery is spent in.
fn landed() -> Pose {
    Pose::standing()
        .hips(0.10, -0.30, 0.0)
        .root(-8.0, 0.0, 0.0)
        .spine(-4.0, 0.0, 0.0)
        .chest(-6.0, 0.0, 0.0)
        .neck(6.0, 0.0)
        .head(-4.0, 0.0, 0.0)
        .tail(14.0, 0.0)
        .plant_fore(0.55, 0.0)
        .plant_hind(-0.40, 0.0)
}

fn lunge() -> Recipe {
    let ready = Pose::standing();
    Recipe::new(
        Clip::Lunge,
        vec![
            Key::eased(0.0, ready, Ease::OUT),
            // Frame eight: half way up, already sitting back -- the shape
            // nothing else makes.
            Key::eased(at_frame(Clip::Lunge, 8.0), sat_back(0.7), Ease::SMOOTH),
            Key::eased(at_frame(Clip::Lunge, 18.0), sat_back(1.0), Ease::HOLD),
            // The gather after the decloak: down onto the forefeet.
            Key::eased(mark(Clip::Lunge, 0, 1.0), gathered(), Ease::SNAP),
            Key::eased(mark(Clip::Lunge, 1, 0.15), flying(0.25), Ease::STRIKE),
            Key::eased(mark(Clip::Lunge, 1, 0.85), flying(0.1), Ease::SMOOTH),
            Key::eased(mark(Clip::Lunge, 2, 0.05), landed(), Ease::OUT),
            Key::eased(mark(Clip::Lunge, 2, 0.6), landed(), Ease::HOLD),
            Key::at(1.0, ready),
        ],
        REAR,
    )
    .noting(
        "Rears out of nothing and sits back on its haunches, forelegs drawn up: \
         by frame eight, at half strength, it is plainly a spring being wound, \
         not a tail going up and not forelegs spreading. Down onto its forefeet \
         for the gather, then flat along the floor jaws first. Lands on its \
         chest with its forefeet far out and stays there for most of a second: \
         the walk-up window, in full view."
            .to_string(),
    )
}

/// The spear's tell: flat to the floor, and the tail rising over the back.
/// `lift` is how far over it has come, in degrees spread down the tail.
fn tail_over(lift: f32) -> Pose {
    Pose::standing()
        .hips(-0.06, -0.30, 0.0)
        .root(-4.0, 0.0, 0.0)
        .spine(-2.0, 0.0, 0.0)
        .chest(-6.0, 0.0, 0.0)
        .neck(-12.0, 0.0)
        .head(10.0, 0.0, 0.0)
        .tail(lift, 0.0)
        .tail_from_base(lift * 0.25)
        .plant_fore(0.20, 0.0)
        .plant_hind(-0.30, 0.0)
}

fn spear() -> Recipe {
    let ready = Pose::standing();
    let driven = tail_over(210.0)
        .hips(0.25, -0.30, 0.0)
        .chest(-10.0, 0.0, 0.0)
        .plant_fore(0.45, 0.0);
    Recipe::new(
        Clip::Spear,
        vec![
            Key::eased(0.0, ready, Ease::IN),
            // Frame eight: belly down, the tail already standing up off the
            // back.
            Key::eased(at_frame(Clip::Spear, 8.0), tail_over(110.0), Ease::SMOOTH),
            // Over the back and pointed past the head by the end of the tell.
            Key::eased(mark(Clip::Spear, 0, 1.0), tail_over(185.0), Ease::SNAP),
            Key::eased(mark(Clip::Spear, 1, 0.5), driven, Ease::STRIKE),
            Key::eased(mark(Clip::Spear, 2, 0.35), driven, Ease::HOLD),
            Key::at(1.0, ready),
        ],
        Looseness::SNAP,
    )
    .noting(
        "The body goes flat and stays flat; what moves is the tail, up off the \
         back first -- the read at frame eight -- and then over it, the tip \
         past the head, aimed down the lane the floor is showing. One short \
         drive forward on the active frames, and a recovery held at full \
         stretch before the tail comes back."
            .to_string(),
    )
}

/// The rake's tell: no rear at all -- low, the forelegs spread wide, the head
/// down between them.
fn spread() -> Pose {
    Pose::standing()
        .hips(0.0, -0.35, 0.0)
        .root(-2.0, 0.0, 0.0)
        .chest(-10.0, 0.0, 0.0)
        .neck(-16.0, 0.0)
        .head(10.0, 0.0, 0.0)
        .tail(4.0, 0.0)
        .leg_spread(0, 34.0)
        .leg_spread(1, 34.0)
        .plant_fore(0.30, 0.0)
        .plant_hind(-0.30, 0.0)
}

/// One foreleg swept across at shin height: `which` 0 is the left, 1 the
/// right.
fn swiped(which: usize) -> Pose {
    let yaw = if which == 0 { 22.0 } else { -22.0 };
    spread()
        .root(0.0, yaw * 0.3, 0.0)
        .chest(-12.0, yaw * 0.6, 0.0)
        .neck(-14.0, yaw * 0.4)
        .leg(which, 68.0, -6.0)
        .leg_spread(which, -18.0)
}

fn rake() -> Recipe {
    Recipe::new(
        Clip::Rake,
        vec![
            Key::eased(0.0, Pose::standing(), Ease::ANTICIPATE),
            Key::eased(at_frame(Clip::Rake, 8.0), spread(), Ease::SNAP),
            Key::eased(mark(Clip::Rake, 0, 1.0), spread(), Ease::HOLD),
            Key::eased(mark(Clip::Rake, 1, 0.4), swiped(0), Ease::STRIKE),
            Key::at(1.0, spread()),
        ],
        QUICK,
    )
    .noting(
        "It does not rear. The forelegs spread wide and the head goes down \
         between them -- a body flattening, where every other strike stands \
         up -- and the left foreleg sweeps across at shin height. Back to the \
         spread for the eight frames between, because the right is coming."
            .to_string(),
    )
}

fn rake2() -> Recipe {
    let ready = Pose::standing();
    Recipe::new(
        Clip::Rake2,
        vec![
            Key::eased(0.0, spread(), Ease::IN),
            Key::eased(mark(Clip::Rake2, 1, 0.4), swiped(1), Ease::STRIKE),
            Key::eased(mark(Clip::Rake2, 2, 0.4), swiped(1), Ease::HOLD),
            Key::eased(mark(Clip::Rake2, 2, 0.8), ready, Ease::SMOOTH),
            Key::at(1.0, ready),
        ],
        QUICK,
    )
    .noting(
        "The right foreleg across, the mirror of the left, and the long \
         recovery held low with the leg still out -- then up into the slink, \
         and the fade begins: the vanish the move is named for, hittable all \
         the way out."
            .to_string(),
    )
}

/// Crouched at a perch's edge, head down over it.
fn over_the_edge() -> Pose {
    Pose::standing()
        .hips(-0.12, -0.28, 0.0)
        .root(-8.0, 0.0, 0.0)
        .spine(-6.0, 0.0, 0.0)
        .chest(-12.0, 0.0, 0.0)
        .neck(-24.0, 0.0)
        .head(18.0, 0.0, 0.0)
        .tail(30.0, 0.0)
        .plant_fore(0.32, 0.0)
        .plant_hind(-0.34, 0.0)
}

fn pounce() -> Recipe {
    let ready = Pose::standing();
    Recipe::new(
        Clip::Pounce,
        vec![
            Key::eased(0.0, ready, Ease::IN),
            Key::eased(at_frame(Clip::Pounce, 8.0), over_the_edge(), Ease::SMOOTH),
            // Off the perch as the decloak ends (`PounceAir` before the hit).
            Key::eased(at_frame(Clip::Pounce, 22.0), over_the_edge(), Ease::STRIKE),
            Key::eased(
                at_frame(Clip::Pounce, 28.0),
                flying(0.3).root(-18.0, 0.0, 0.0),
                Ease::SMOOTH,
            ),
            Key::eased(
                mark(Clip::Pounce, 1, 0.0),
                flying(0.0).root(-24.0, 0.0, 0.0),
                Ease::IN,
            ),
            Key::eased(mark(Clip::Pounce, 1, 1.0), landed(), Ease::OUT),
            Key::eased(mark(Clip::Pounce, 2, 0.6), landed(), Ease::HOLD),
            Key::at(1.0, ready),
        ],
        Looseness::SNAP,
    )
    .noting(
        "Decloaks on its perch hanging over the edge, head down at the circle \
         it has already drawn; drops off at frame twenty-two, forelegs \
         reaching, and lands on all four for the knockdown. The circle does \
         not follow, so the drop is a commitment you can see from the side."
            .to_string(),
    )
}

/// **The quills' rear**: the tallest thing it does -- up on its hind legs,
/// chest high, the tail arched right over its head. `k` is how far.
fn reared_high(k: f32, tail: f32) -> Pose {
    Pose::standing()
        .hips(-0.40 * k, -0.20 * k, 0.0)
        .root(48.0 * k, 0.0, 0.0)
        .spine(10.0 * k, 0.0, 0.0)
        .chest(8.0 * k, 0.0, 0.0)
        .neck(14.0 * k, 0.0)
        .head(-34.0 * k, 0.0, 0.0)
        .tail(tail, 0.0)
        .tail_from_base(tail * 0.3)
        .forelegs(-30.0 * k, 60.0 * k)
        .plant_hind(-0.35, 0.0)
}

fn quills() -> Recipe {
    let ready = Pose::standing();
    Recipe::new(
        Clip::Quills,
        vec![
            Key::eased(0.0, ready, Ease::OUT),
            Key::eased(
                at_frame(Clip::Quills, 8.0),
                reared_high(0.65, 90.0),
                Ease::SMOOTH,
            ),
            Key::eased(
                mark(Clip::Quills, 0, 1.0),
                reared_high(1.0, 175.0),
                Ease::SNAP,
            ),
            // The snap: the tail whipped forward over the head.
            Key::eased(
                at_frame(Clip::Quills, 25.0),
                reared_high(1.0, 230.0),
                Ease::STRIKE,
            ),
            Key::eased(
                mark(Clip::Quills, 1, 1.0),
                reared_high(0.8, 150.0),
                Ease::SMOOTH,
            ),
            Key::eased(
                mark(Clip::Quills, 2, 0.5),
                reared_high(0.3, 40.0),
                Ease::SMOOTH,
            ),
            Key::at(1.0, ready),
        ],
        REAR,
    )
    .noting(
        "Up on its hind legs, the highest it ever stands, and the tail arching \
         right over its head: the tallest silhouette it has, readable across \
         the arena -- which is where the quills are thrown from. The snap \
         whips the tail forward and the fan leaves; it stays up through the \
         flight and comes down slowly."
            .to_string(),
    )
}

fn smoke() -> Recipe {
    let hunched = Pose::standing()
        .hips(0.0, -0.20, 0.0)
        .spine(8.0, 0.0, 0.0)
        .chest(4.0, 0.0, 0.0)
        .neck(-18.0, 0.0)
        .head(14.0, 0.0, 0.0)
        .tail(-10.0, 0.0)
        .plant_fore(0.10, 0.0)
        .plant_hind(-0.20, 0.0);
    let pulse = hunched.spine(14.0, 0.0, 4.0).chest(8.0, 0.0, -4.0);
    Recipe::new(
        Clip::Smoke,
        vec![
            Key::eased(0.0, Pose::standing(), Ease::IN),
            Key::eased(mark(Clip::Smoke, 0, 0.25), hunched, Ease::SMOOTH),
            Key::eased(mark(Clip::Smoke, 0, 0.5), pulse, Ease::SMOOTH),
            Key::eased(mark(Clip::Smoke, 0, 0.75), hunched, Ease::SMOOTH),
            Key::eased(mark(Clip::Smoke, 1, 0.5), pulse, Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::BEAST,
    )
    .noting(
        "Hunched, the flanks pumping as the cloud comes off them. Cloaked \
         throughout: the cloud growing from a point is the tell, not this."
            .to_string(),
    )
}

fn mimic() -> Recipe {
    let flat = Pose::standing()
        .hips(0.0, -0.45, 0.0)
        .chest(-6.0, 0.0, 0.0)
        .neck(-10.0, 0.0)
        .head(10.0, 0.0, 0.0)
        .tail(-8.0, 0.0)
        .plant_fore(0.30, 0.0)
        .plant_hind(-0.35, 0.0);
    Recipe::new(
        Clip::Mimic,
        vec![
            Key::eased(0.0, Pose::standing(), Ease::OUT),
            Key::eased(mark(Clip::Mimic, 0, 0.3), flat, Ease::HOLD),
            Key::eased(mark(Clip::Mimic, 2, 1.0), flat, Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::BEAST,
    )
    .noting(
        "The real animal through a mimic: flat to the snow and utterly still, \
         somewhere else, while its ghost rears where you are looking. It sets \
         no feet and leaves no prints; it is never drawn unless it is painted \
         or mottled -- and then a still patch of hide away from the rear is the \
         giveaway."
            .to_string(),
    )
}

fn retreat() -> Recipe {
    let recoil = Pose::standing()
        .hips(-0.20, -0.10, 0.0)
        .root(16.0, -10.0, 6.0)
        .spine(-6.0, -6.0, 0.0)
        .neck(-20.0, -18.0)
        .head(14.0, 0.0, 0.0)
        .tail(20.0, -24.0)
        .plant_fore(-0.10, 0.0)
        .plant_hind(-0.40, 0.0);
    let gather = Pose::standing()
        .hips(0.0, -0.08, 0.0)
        .root(8.0, 0.0, 0.0)
        .spine(-14.0, 0.0, 0.0)
        .neck(-16.0, 0.0)
        .tail(8.0, 0.0)
        .plant_fore(-0.40, 0.30)
        .plant_hind(0.40, 0.0);
    let reach = Pose::standing()
        .hips(0.0, 0.10, 0.0)
        .root(-8.0, 0.0, 0.0)
        .spine(14.0, 0.0, 0.0)
        .neck(-10.0, 0.0)
        .tail(-4.0, 0.0)
        .plant_fore(0.80, 0.0)
        .plant_hind(-0.80, 0.30);
    let mut keys = vec![
        Key::eased(0.0, Pose::standing(), Ease::OUT),
        Key::eased(mark(Clip::Retreat, 0, 0.4), recoil, Ease::HOLD),
        Key::eased(mark(Clip::Retreat, 0, 1.0), recoil, Ease::SNAP),
    ];
    for i in 0..10 {
        let u = i as f32 / 10.0;
        let pose = if i % 2 == 0 { gather } else { reach };
        keys.push(Key::eased(mark(Clip::Retreat, 1, u), pose, Ease::SMOOTH));
    }
    keys.push(Key::eased(
        mark(Clip::Retreat, 2, 0.3),
        Pose::standing(),
        Ease::OUT,
    ));
    keys.push(Key::at(1.0, Pose::standing()));
    Recipe::new(Clip::Retreat, keys, Looseness::BEAST).noting(
        "The recoil, in full view: flinched back and turned, twelve frames to \
         break it with a burst. Then the bound away, keyed as gather and reach \
         across the active window -- a shimmer to anyone watching -- and the \
         slink again as it fades."
            .to_string(),
    )
}

fn climb() -> Recipe {
    let crouch = Pose::standing()
        .hips(-0.10, -0.40, 0.0)
        .root(18.0, 0.0, 0.0)
        .chest(-4.0, 0.0, 0.0)
        .neck(12.0, 0.0)
        .tail(-10.0, 0.0)
        .plant_fore(0.18, 0.0)
        .plant_hind(-0.40, 0.0);
    let up = flying(0.2).root(30.0, 0.0, 0.0).neck(10.0, 0.0);
    let on_top = Pose::standing()
        .hips(0.0, -0.30, 0.0)
        .neck(6.0, 0.0)
        .tail(12.0, 0.0)
        .plant_fore(0.10, 0.0)
        .plant_hind(-0.20, 0.0);
    Recipe::new(
        Clip::Climb,
        vec![
            Key::eased(0.0, Pose::standing(), Ease::IN),
            Key::eased(mark(Clip::Climb, 0, 0.3), crouch, Ease::STRIKE),
            Key::eased(mark(Clip::Climb, 0, 0.7), up, Ease::SMOOTH),
            Key::eased(mark(Clip::Climb, 1, 1.0), on_top, Ease::OUT),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::BEAST,
    )
    .noting(
        "Up a dead trunk or onto the ledge in one leap and a scramble, \
         cloaked: the climb is never seen, only the perch it decloaks on."
            .to_string(),
    )
}

/// On its side in the snow: the panic, and death.
fn on_its_side(roll: f32, kick: f32) -> Pose {
    Pose::standing()
        .hips(0.0, -0.70, 0.20)
        .root(-4.0, 0.0, roll)
        .spine(-4.0, 0.0, roll * 0.12)
        .chest(-6.0, 0.0, roll * 0.10)
        .neck(8.0, 20.0)
        .head(6.0, 0.0, 0.0)
        .forelegs(30.0 + kick, -40.0 - kick)
        .hindlegs(36.0 - kick, -50.0 + kick)
        .leg_spread(0, 24.0)
        .leg_spread(1, 24.0)
        .leg_spread(2, 20.0)
        .leg_spread(3, 20.0)
        .tail(0.0, 30.0 + kick)
}

fn getup() -> Recipe {
    Recipe::new(
        Clip::Getup,
        vec![
            Key::eased(0.0, on_its_side(70.0, 0.0), Ease::OUT),
            Key::eased(
                mark(Clip::Getup, 0, 0.5),
                on_its_side(30.0, 10.0).hips(0.0, -0.40, 0.0),
                Ease::SMOOTH,
            ),
            Key::eased(mark(Clip::Getup, 0, 1.0), Pose::standing(), Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::COLLAPSE,
    )
    .noting(
        "Out of the snow and onto its feet in half a second, in full view, \
         before it bolts: the panic's last beat."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// Being in trouble
// ---------------------------------------------------------------------------

fn flinch() -> Recipe {
    let hit = Pose::standing()
        .hips(-0.10, -0.08, 0.0)
        .root(6.0, -6.0, 5.0)
        .spine(-6.0, -4.0, 0.0)
        .neck(-18.0, -14.0)
        .head(10.0, 0.0, 0.0)
        .tail(16.0, -24.0)
        .plant_fore(-0.04, 0.0)
        .plant_hind(-0.28, 0.0);
    Recipe::new(
        Clip::Flinch,
        vec![
            Key::eased(0.0, hit, Ease::OUT),
            Key::eased(0.45, hit.blend(&Pose::standing(), 0.5), Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::LIMP,
    )
    .noting(
        "A recoil, held at the start and bled off -- in full view, which is \
         what a hit during a decloak buys."
            .to_string(),
    )
}

fn stumble() -> Recipe {
    let down = Pose::standing()
        .hips(0.0, -0.55, 0.0)
        .root(-10.0, 0.0, 6.0)
        .spine(-6.0, 0.0, 0.0)
        .chest(-8.0, 0.0, 0.0)
        .neck(-6.0, 10.0)
        .head(4.0, 0.0, 0.0)
        .tail(-10.0, -16.0)
        .forelegs(70.0, -120.0)
        .plant_hind(-0.40, 0.0);
    Recipe::new(
        Clip::Stumble,
        vec![
            Key::eased(0.0, Pose::standing(), Ease::IN),
            Key::eased(0.15, down, Ease::HOLD),
            Key::eased(0.70, down, Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::COLLAPSE,
    )
    .noting("Down on its chest from crowd control, and a slow push back up.".to_string())
}

fn panic() -> Recipe {
    // Rolling and thrashing in the snow, a metre and a half high at most:
    // over one way, legs kicking, over the other.
    let mut keys = vec![Key::eased(0.0, Pose::standing(), Ease::IN)];
    let rolls = [70.0, 110.0, 60.0, 120.0, 75.0, 100.0, 70.0];
    for (i, roll) in rolls.into_iter().enumerate() {
        let u = 0.08 + i as f32 * 0.125;
        let kick = if i % 2 == 0 { 25.0 } else { -25.0 };
        keys.push(Key::eased(u, on_its_side(roll, kick), Ease::SMOOTH));
    }
    keys.push(Key::at(1.0, on_its_side(70.0, 0.0)));
    Recipe::new(Clip::Panic, keys, Looseness::COLLAPSE).noting(
        "Fire on it: down in the snow, rolling from side to side and kicking, \
         two and a half seconds of a large animal on the ground and in full \
         view. The window: nothing is out of reach, nothing is thrown."
            .to_string(),
    )
}

fn dead() -> Recipe {
    let gone = on_its_side(80.0, 0.0)
        .hips(0.0, -0.80, 0.24)
        .neck(-30.0, 30.0)
        .tail(-6.0, 40.0);
    Recipe::new(Clip::Dead, vec![Key::at(0.0, gone)], Looseness::LIMP)
        .noting("One pose, on its side, and drawn whole at last.".to_string())
}

/// Every clip that has no recipe. Empty is the answer we want.
pub fn missing() -> Vec<Clip> {
    let have = all();
    Clip::ALL
        .into_iter()
        .filter(|c| !have.iter().any(|r| r.clip == c.index()))
        .collect()
}
