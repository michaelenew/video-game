//! Every animation the Galewing has, as poses and timing.
//!
//! Baked into `crates/sim/src/species/galewing/baked.rs` by
//! `cargo run -p anim --bin bake_beast -- --species galewing`.
//!
//! The house style is the Ridgeback's: **the windup is the move**, the hit is
//! short and enormous, and the recovery is long and readable. Three things
//! are this bird's own:
//!
//! 1. **The silhouette from below** (`galewing.md` §6). Each air move changes
//!    the shape a person sees from the floor on the first frame of its
//!    startup: an eighteen-metre cross circling; an eight-metre dart (the
//!    Stoop: wings folded in at once); a flat line at head height (the pass,
//!    talons swinging forward twenty frames out); wings standing vertical
//!    over a still body (the Downwash); the bird on its side (the volley).
//! 2. **Where it flies, its body says nothing.** Its bank and its pitch are
//!    the flight's, put on the root by the simulation's `repose`; the clips
//!    keep the root level in the air so the two never fight.
//! 3. **On the floor the wings are terrain.** After a Stoop they lie spread
//!    from 0.4 to 1.6 m; in the crash they lie flat on the floor, ramps from
//!    a 0.3 m tip up to the back at 2.4 m.

use super::{BirdPose, mark};
use crate::beast::{Feel, Key, Looseness, Pose, Recipe};
use crate::ease::Ease;
use sim::species::galewing::{Clip, bones};

pub fn all() -> Vec<Recipe> {
    vec![
        idle(),
        walk(),
        run(),
        fly(),
        glide(),
        stoop(),
        talon(),
        carry(),
        downwash(),
        volley(),
        screech(),
        buffet(),
        hop(),
        perch(),
        lift(),
        roll(),
        flinch(),
        stumble(),
        crash(),
        dead(),
    ]
}

/// A wingbeat: the wings are the loosest thing on it, and they lead with
/// the shoulder -- the tip trails into the downstroke and whips out of it.
const BEAT: Looseness = Looseness {
    body: Feel::new(1.6, 0.92),
    neck: Feel::new(2.4, 0.70),
    tail: Feel::new(2.2, 0.62),
    legs: Feel::new(1.6, 0.90),
};

/// The fold into a dive, the snap of a buffet: quick and tight.
const SNAP: Looseness = Looseness {
    body: Feel::new(1.2, 0.95),
    neck: Feel::new(1.6, 0.80),
    tail: Feel::new(1.6, 0.75),
    legs: Feel::new(1.2, 0.95),
};

// ---------------------------------------------------------------------------
// Standing and moving
// ---------------------------------------------------------------------------

fn idle() -> Recipe {
    let breathe = |k: f32| {
        Pose::standing()
            .hips(0.0, 0.03 * k, 0.0)
            .chest(1.5 * k, 0.0, 0.0)
            .neck(3.0 * k, 6.0 * k)
            .tail(2.0 * k, 0.0)
    };
    Recipe::new(
        Clip::Idle,
        vec![
            Key::at(0.0, breathe(-1.0)),
            Key::at(0.5, breathe(1.0)),
            Key::at(0.8, breathe(0.3)),
        ],
        Looseness::BEAST,
    )
    .noting(
        "Standing on the floor or the perch: wings folded along its back, the \
         head turning and the chest breathing. Three and a half metres to the \
         back, the climb question asked of every class."
            .to_string(),
    )
}

/// One leg at a point in its own cycle: planted for most of it, swinging for
/// the rest.
fn stride(p: Pose, which: usize, u: f32, reach: f32, lift: f32) -> Pose {
    const STANCE: f32 = 0.55;
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

fn walk() -> Recipe {
    // A biped's walk: the legs half a cycle apart, the body rolling over
    // each planted foot and the head bobbing forward with every step.
    let at = |t: f32| {
        let wheel = t * std::f32::consts::TAU;
        let p = Pose::standing()
            .hips(0.0, -0.06 + 0.05 * (2.0 * wheel).cos(), 0.0)
            .root(0.0, 0.0, 3.0 * wheel.sin())
            .neck(18.0 - 8.0 * (2.0 * wheel).sin(), 0.0);
        let p = stride(p, 0, t, 0.65, 0.35);
        stride(p, 1, t + 0.5, 0.65, 0.35)
    };
    Recipe::new(
        Clip::Walk,
        (0..8)
            .map(|i| {
                let t = i as f32 / 8.0;
                Key::eased(t, at(t), Ease::LINEAR)
            })
            .collect(),
        Looseness::GAIT,
    )
    .noting("Grounded for good, it walks: rolling over each foot, the head bobbing.".to_string())
}

fn run() -> Recipe {
    // The grounded bird's run: half a hop, wings half open for balance.
    let at = |t: f32| {
        let wheel = t * std::f32::consts::TAU;
        let p = Pose::standing()
            .wings([20.0 + 10.0 * wheel.sin(), 6.0, 4.0], [40.0, -60.0, 40.0])
            .hips(0.0, -0.10 + 0.12 * (2.0 * wheel).cos(), 0.0)
            .root(-8.0, 0.0, 0.0)
            .neck(4.0, 0.0);
        let p = stride(p, 0, t, 0.95, 0.55);
        stride(p, 1, t + 0.5, 0.95, 0.55)
    };
    Recipe::new(
        Clip::Run,
        (0..8)
            .map(|i| {
                let t = i as f32 / 8.0;
                Key::eased(t, at(t), Ease::LINEAR)
            })
            .collect(),
        Looseness::GAIT,
    )
    .noting("A run with the wings half out for balance: what is left of flying.".to_string())
}

// ---------------------------------------------------------------------------
// In the air
// ---------------------------------------------------------------------------

/// A point in a wingbeat: `u` 0 is the top of the upstroke, the downstroke
/// runs to a quarter of the way round, the upstroke the rest.
fn beat(u: f32) -> Pose {
    let u = u.rem_euclid(1.0);
    // The downstroke is quick and the upstroke slow: the beat's power.
    let (up, twist) = if u < 0.25 {
        let k = u / 0.25;
        (40.0 - 70.0 * k, -8.0 * (k * std::f32::consts::PI).sin())
    } else {
        let k = (u - 0.25) / 0.75;
        (-30.0 + 70.0 * k, 10.0 * (k * std::f32::consts::PI).sin())
    };
    Pose::soaring()
        .wings([up, up * 0.4, up * 0.3], [0.0, 0.0, 8.0 - up * 0.1])
        .twist(twist)
        .hips(0.0, -0.06 * up / 40.0, 0.0)
}

fn fly() -> Recipe {
    Recipe::new(
        Clip::Fly,
        (0..8)
            .map(|i| {
                let t = i as f32 / 8.0;
                Key::eased(t, beat(t), Ease::LINEAR)
            })
            .collect(),
        BEAT,
    )
    .noting(
        "One wingbeat, a breath of its clock long: the downstroke a quarter of \
         it, quick and heavy, the upstroke the rest. Climbing with a rider it \
         is the beat the rider braces for."
            .to_string(),
    )
}

fn glide() -> Recipe {
    let at = |k: f32| {
        Pose::soaring()
            .wings([6.0 + 3.0 * k, 3.0, 2.0 - 2.0 * k], [0.0, 0.0, 6.0])
            .tail(2.0 * k, 4.0 * k)
            .neck(-6.0, 6.0 * k)
    };
    Recipe::new(
        Clip::Glide,
        vec![Key::at(0.0, at(-1.0)), Key::at(0.5, at(1.0))],
        Looseness::BEAST,
    )
    .noting(
        "Circling: wings held spread, the tips flexing, the tail working as a \
         rudder. Eighteen metres across, the cross on the sky and the shadow \
         on the floor."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// The air moves
// ---------------------------------------------------------------------------

/// The dart: wings swept back hard against the body, head and neck
/// straight out, legs tucked. Eight metres long and nothing else.
fn dart() -> Pose {
    Pose::soaring()
        .wings([4.0, 0.0, 0.0], [70.0, -20.0, 30.0])
        .neck(-4.0, 0.0)
        .head(-6.0, 0.0, 0.0)
        .tail(-4.0, 0.0)
}

/// Down on the floor after a Stoop: wings spread low over the ground, body
/// crouched, talons down -- the walk-up window's shape.
fn spread_on_the_floor() -> Pose {
    Pose::rest(bones::COUNT)
        .hips(0.0, -0.75, 0.0)
        .wings([-8.0, -2.0, -2.0], [10.0, 0.0, 10.0])
        .neck(10.0, 0.0)
        .head(-10.0, 0.0, 0.0)
        .tail(-6.0, 0.0)
        .plant_both(0.25, 0.0)
}

fn stoop() -> Recipe {
    let c = Clip::Stoop;
    Recipe::new(
        c,
        vec![
            Key::eased(0.0, Pose::soaring(), Ease::SNAP),
            // **Folded on the first frames**: the cross becomes a dart.
            Key::eased(mark(c, 0, 0.12), dart(), Ease::LINEAR),
            Key::eased(mark(c, 0, 0.92), dart(), Ease::STRIKE),
            // Talons thrown forward, wings flung open to stop.
            Key::eased(
                mark(c, 1, 0.0),
                spread_on_the_floor()
                    .wings([20.0, 10.0, 6.0], [-10.0, 0.0, 0.0])
                    .legs(40.0, -20.0),
                Ease::OUT,
            ),
            Key::eased(mark(c, 2, 0.15), spread_on_the_floor(), Ease::SMOOTH),
            Key::eased(mark(c, 2, 0.85), spread_on_the_floor(), Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        SNAP,
    )
    .noting(
        "The Stoop: the cross folds into a dart in its first frames -- the \
         loudest change of silhouette it has -- and stays a dart to the floor. \
         It hits talons first with its wings flung open, and the recovery is \
         a second on the floor with them spread low: the walk-up."
            .to_string(),
    )
}

/// Low and level along the lane, talons thrown forward and down.
fn talons_out(k: f32) -> Pose {
    Pose::soaring()
        .wings([4.0, 2.0, 2.0], [0.0, 0.0, 4.0])
        .neck(-10.0 * k, 0.0)
        .head(18.0 * k, 0.0, 0.0)
        .legs(-75.0 + 115.0 * k, 40.0 - 50.0 * k)
}

fn talon() -> Recipe {
    let c = Clip::Talon;
    // The talons swing forward twenty frames before they reach you: the last
    // part of the startup.
    let swing = 1.0 - 20.0 / sim::species::galewing::SPECIES.attack(1).startup.max(21) as f32;
    Recipe::new(
        c,
        vec![
            Key::eased(0.0, Pose::soaring(), Ease::SMOOTH),
            // Flattened to a line at once: wings level, no dihedral.
            Key::eased(mark(c, 0, 0.10), talons_out(0.0), Ease::SMOOTH),
            Key::eased(mark(c, 0, swing), talons_out(0.0), Ease::OUT),
            Key::eased(mark(c, 1, 0.0), talons_out(1.0), Ease::SMOOTH),
            Key::eased(mark(c, 1, 1.0), talons_out(1.0), Ease::SMOOTH),
            Key::eased(mark(c, 2, 0.4), beat(0.0), Ease::SMOOTH),
            Key::at(1.0, beat(0.3)),
        ],
        Looseness::BEAST,
    )
    .noting(
        "The pass: a flat line at head height from the first frames, the \
         talons swinging forward twenty frames before they reach you, held \
         out down the lane and pulled up with a beat at the end."
            .to_string(),
    )
}

fn carry() -> Recipe {
    let holding = |u: f32| beat(u).legs(10.0, 0.0).neck(6.0, 0.0);
    Recipe::new(
        Clip::Carry,
        (0..9)
            .map(|i| {
                let t = i as f32 / 8.0;
                Key::eased(t, holding(t * 3.0), Ease::LINEAR)
            })
            .collect(),
        BEAT,
    )
    .noting(
        "Climbing with somebody in its talons: beating hard, legs hanging. The \
         legs are the answer, and they are where they can be reached."
            .to_string(),
    )
}

/// Hanging still, wings standing up over its back.
fn hanging(up: f32) -> Pose {
    Pose::rest(bones::COUNT)
        .root(30.0, 0.0, 0.0)
        .wings([up, 10.0, 6.0], [-10.0, 0.0, 0.0])
        .twist(-20.0)
        .neck(-30.0, 0.0)
        .head(10.0, 0.0, 0.0)
        .tail(-30.0, 0.0)
        .legs(30.0, -10.0)
}

fn downwash() -> Recipe {
    let c = Clip::Downwash;
    let mut keys = vec![
        Key::eased(0.0, Pose::soaring(), Ease::SMOOTH),
        // Wings vertical and the body hanging: the loudest silhouette it has.
        Key::eased(mark(c, 0, 0.15), hanging(80.0), Ease::SMOOTH),
        Key::eased(mark(c, 0, 1.0), hanging(80.0), Ease::IN),
    ];
    // The beats, down and up through the active window.
    for i in 0..8 {
        let u = i as f32 / 8.0;
        let up = if i % 2 == 0 { 20.0 } else { 80.0 };
        keys.push(Key::eased(mark(c, 1, u + 0.06), hanging(up), Ease::SMOOTH));
    }
    keys.push(Key::eased(mark(c, 2, 0.3), beat(0.0), Ease::SMOOTH));
    keys.push(Key::at(1.0, Pose::soaring()));
    Recipe::new(c, keys, BEAT).noting(
        "The Downwash: it stops and hangs, wings standing vertical over a \
         still body, then beats them down through two seconds of wind."
            .to_string(),
    )
}

/// Rolled onto its side, the upper wing lifted with the feathers stood up
/// along it.
fn on_its_side(k: f32) -> Pose {
    Pose::soaring()
        .root(0.0, 0.0, 70.0 * k)
        .wing(1, [60.0 * k, 10.0 * k, 0.0], [0.0, 0.0, 0.0])
        .wing(0, [-10.0 * k, 0.0, 0.0], [10.0 * k, 0.0, 0.0])
        .neck(-6.0, -20.0 * k)
}

fn volley() -> Recipe {
    let c = Clip::Volley;
    Recipe::new(
        c,
        vec![
            Key::eased(0.0, Pose::soaring(), Ease::SMOOTH),
            Key::eased(mark(c, 0, 0.2), on_its_side(1.0), Ease::SMOOTH),
            Key::eased(mark(c, 0, 1.0), on_its_side(1.05), Ease::STRIKE),
            // The rake: the upper wing sweeping down across the lane.
            Key::eased(
                mark(c, 1, 0.5),
                on_its_side(1.0).wing(1, [10.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
                Ease::SMOOTH,
            ),
            Key::eased(mark(c, 1, 1.0), on_its_side(0.9), Ease::SMOOTH),
            Key::at(1.0, Pose::soaring()),
        ],
        Looseness::BEAST,
    )
    .noting(
        "The volley: on its side, its upper wing lifted with the blade \
         feathers stood up along it, then raked down across the lane."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// The ground moves
// ---------------------------------------------------------------------------

/// The Screech's windup: head back, crest up, throat swollen.
fn head_back(k: f32) -> Pose {
    Pose::standing()
        .neck(18.0 + 22.0 * k, 0.0)
        .head(-14.0 + 34.0 * k, 0.0, 0.0)
        .wings(
            [12.0 + 10.0 * k, 8.0, 6.0],
            [82.0 - 20.0 * k, -160.0, 150.0],
        )
        .hips(0.0, 0.10 * k, 0.0)
}

fn screech() -> Recipe {
    let c = Clip::Screech;
    let thrown = Pose::standing()
        .neck(-10.0, 0.0)
        .head(-6.0, 0.0, 0.0)
        .wings([30.0, 10.0, 6.0], [50.0, -140.0, 140.0]);
    Recipe::new(
        c,
        vec![
            Key::eased(0.0, Pose::standing(), Ease::SMOOTH),
            Key::eased(mark(c, 0, 0.3), head_back(1.0), Ease::IN),
            Key::eased(mark(c, 0, 1.0), head_back(1.1), Ease::STRIKE),
            Key::eased(mark(c, 1, 0.2), thrown, Ease::SMOOTH),
            Key::eased(mark(c, 1, 1.0), thrown, Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::BEAST,
    )
    .noting(
        "The Screech: head thrown back, crest up and throat filling through \
         the tell; then neck straight out at whoever is in the cone."
            .to_string(),
    )
}

/// The buffet's wing (the left: mirrored to the side you are on), lifted
/// high and back, then swept along the floor.
fn wing_up() -> Pose {
    Pose::standing()
        .wing(0, [70.0, 20.0, 10.0], [50.0, -40.0, 20.0])
        .root(0.0, 0.0, -8.0)
        .neck(10.0, 20.0)
}

fn wing_swept() -> Pose {
    Pose::standing()
        .wing(0, [-14.0, -4.0, -2.0], [-30.0, 0.0, 0.0])
        .root(0.0, 0.0, 6.0)
        .hips(0.0, -0.2, 0.0)
        .neck(6.0, -10.0)
}

fn buffet() -> Recipe {
    let c = Clip::Buffet;
    Recipe::new(
        c,
        vec![
            Key::eased(0.0, Pose::standing(), Ease::SMOOTH),
            Key::eased(mark(c, 0, 0.5), wing_up(), Ease::IN),
            Key::eased(mark(c, 0, 1.0), wing_up(), Ease::STRIKE),
            Key::eased(mark(c, 1, 1.0), wing_swept(), Ease::SMOOTH),
            Key::eased(mark(c, 2, 0.4), wing_swept(), Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        SNAP,
    )
    .noting(
        "The buffet: the wing on your side comes up high and back, then sweeps \
         the floor to six metres. Jumped."
            .to_string(),
    )
}

fn crouched() -> Pose {
    Pose::standing()
        .hips(0.0, -0.6, 0.0)
        .root(-10.0, 0.0, 0.0)
        .wings([30.0, 10.0, 6.0], [40.0, -80.0, 60.0])
        .plant_both(0.3, 0.0)
}

fn hop() -> Recipe {
    let c = Clip::Hop;
    let up = Pose::standing()
        .wings([50.0, 20.0, 10.0], [10.0, -20.0, 10.0])
        .legs(-30.0, 40.0)
        .hips(0.0, 0.3, 0.0);
    Recipe::new(
        c,
        vec![
            Key::eased(0.0, Pose::standing(), Ease::SMOOTH),
            Key::eased(mark(c, 0, 0.4), crouched(), Ease::SNAP),
            Key::eased(mark(c, 0, 0.6), up, Ease::SMOOTH),
            Key::eased(mark(c, 1, 0.0), crouched(), Ease::OUT),
            Key::eased(mark(c, 2, 0.5), crouched(), Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::HEAVY,
    )
    .noting(
        "Grounded for good, its gap-closer: a crouch, a hop with the broken \
         wings thrown open, and a heavy landing on the circle it marked."
            .to_string(),
    )
}

fn perch() -> Recipe {
    let c = Clip::Perch;
    let braking = Pose::soaring()
        .root(30.0, 0.0, 0.0)
        .wings([30.0, 10.0, 0.0], [-20.0, 0.0, 0.0])
        .twist(-30.0)
        .legs(50.0, -30.0);
    Recipe::new(
        c,
        vec![
            Key::eased(0.0, Pose::soaring(), Ease::SMOOTH),
            // The call: the head thrown up.
            Key::eased(
                mark(c, 0, 0.1),
                Pose::soaring().neck(30.0, 0.0),
                Ease::SMOOTH,
            ),
            Key::eased(mark(c, 0, 0.3), Pose::soaring(), Ease::SMOOTH),
            Key::eased(mark(c, 0, 0.6), beat(0.0), Ease::SMOOTH),
            Key::eased(mark(c, 0, 0.8), braking, Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::BEAST,
    )
    .noting(
        "The perch: a call with the head thrown up, the long glide to the tower, \
         and the braking landing, wings cupped and talons out."
            .to_string(),
    )
}

fn lift() -> Recipe {
    let c = Clip::Lift;
    let gathered = Pose::standing()
        .hips(0.0, -0.4, 0.0)
        .root(20.0, 0.0, 0.0)
        .wings([50.0, 20.0, 10.0], [20.0, -40.0, 20.0])
        .plant_both(0.0, 0.0);
    let mut keys = vec![
        Key::eased(0.0, Pose::standing(), Ease::SMOOTH),
        // The gather: wings coming in and up, the body rearing.
        Key::eased(mark(c, 0, 0.7), gathered, Ease::IN),
        Key::eased(mark(c, 0, 1.0), beat(0.0).legs(20.0, -10.0), Ease::SMOOTH),
    ];
    for i in 1..6 {
        let u = i as f32 / 6.0;
        keys.push(Key::eased(mark(c, 1, u), beat(u * 3.0), Ease::LINEAR));
    }
    keys.push(Key::at(1.0, Pose::soaring()));
    Recipe::new(c, keys, BEAT).noting(
        "The gather and the lift: wings in and up, the body rearing, and big \
         downstrokes up off the floor. Whoever is still aboard goes up."
            .to_string(),
    )
}

fn roll() -> Recipe {
    let c = Clip::Roll;
    let tucked = Pose::soaring()
        .wing(0, [10.0, 0.0, 0.0], [40.0, -20.0, 20.0])
        .neck(-20.0, 0.0)
        .head(-10.0, 0.0, 0.0);
    Recipe::new(
        c,
        vec![
            Key::eased(0.0, Pose::soaring(), Ease::SMOOTH),
            // The head dips and one wing tucks: the tell.
            Key::eased(mark(c, 0, 0.6), tucked, Ease::SMOOTH),
            Key::eased(mark(c, 1, 0.0), tucked, Ease::SMOOTH),
            Key::eased(mark(c, 1, 1.0), Pose::soaring(), Ease::SMOOTH),
            Key::at(1.0, Pose::soaring()),
        ],
        Looseness::BEAST,
    )
    .noting(
        "The barrel roll's tell: the head dips and one wing tucks. The roll \
         itself is the simulation's, on the root, so the hit test and the \
         riders feel exactly what is drawn."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// Trouble
// ---------------------------------------------------------------------------

fn flinch() -> Recipe {
    let c = Clip::Flinch;
    let hit = Pose::standing()
        .neck(30.0, 10.0)
        .head(10.0, 0.0, 0.0)
        .wings([30.0, 10.0, 6.0], [60.0, -140.0, 140.0])
        .hips(0.0, 0.1, 0.0);
    Recipe::new(
        c,
        vec![
            Key::eased(0.0, Pose::standing(), Ease::STRIKE),
            Key::eased(0.25, hit, Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::BEAST,
    )
    .noting("A recoil: the head jerks back and the wings start open.".to_string())
}

fn stumble() -> Recipe {
    let down = Pose::standing()
        .hips(0.0, -0.9, 0.0)
        .root(-12.0, 0.0, 8.0)
        .wings([-10.0, 0.0, 0.0], [40.0, -60.0, 60.0])
        .neck(-10.0, 0.0)
        .plant_both(0.4, 0.0);
    Recipe::new(
        Clip::Stumble,
        vec![
            Key::eased(0.0, Pose::standing(), Ease::IN),
            Key::eased(0.2, down, Ease::SMOOTH),
            Key::eased(0.8, down, Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::COLLAPSE,
    )
    .noting(
        "Down on its breast for a moment, wings dropped: the grounded bird's stumble.".to_string(),
    )
}

/// **The crash**: on its breast, the back down to 2.4 m, the wings flat on
/// the floor -- ramps from a tip a hand off the ground up to the back.
fn crashed() -> Pose {
    Pose::rest(bones::COUNT)
        .hips(0.0, -1.0, 0.0)
        .wings([-12.0, -1.0, -1.0], [4.0, 0.0, 6.0])
        .neck(-20.0, 20.0)
        .head(10.0, 0.0, 0.0)
        .tail(-4.0, 0.0)
        .legs(40.0, -80.0)
}

fn crash() -> Recipe {
    let tumbling = Pose::soaring()
        .root(0.0, 0.0, 30.0)
        .wings([50.0, 20.0, 10.0], [-20.0, 0.0, 0.0])
        .neck(20.0, 30.0);
    Recipe::new(
        Clip::Crash,
        vec![
            Key::eased(0.0, tumbling, Ease::SMOOTH),
            Key::eased(0.08, crashed(), Ease::OUT),
            Key::eased(0.85, crashed().neck(-10.0, 0.0), Ease::SMOOTH),
            Key::at(1.0, crashed().neck(10.0, 0.0)),
        ],
        Looseness::COLLAPSE,
    )
    .noting(
        "The crash: tumbling out of the air and down on its breast with the \
         wings flat on the floor. Four seconds of the fight's big window, the \
         wings ramps up onto its back for every class."
            .to_string(),
    )
}

fn dead() -> Recipe {
    let gone = crashed()
        .root(0.0, 0.0, 20.0)
        .neck(-40.0, 40.0)
        .head(30.0, 0.0, 0.0);
    Recipe::new(Clip::Dead, vec![Key::at(0.0, gone)], Looseness::LIMP)
        .noting("One pose: down on the cliff top, a wing across the floor.".to_string())
}

/// Every clip that has no recipe. Empty is the answer we want.
pub fn missing() -> Vec<Clip> {
    let have = all();
    Clip::ALL
        .into_iter()
        .filter(|c| !have.iter().any(|r| r.clip == c.index()))
        .collect()
}
