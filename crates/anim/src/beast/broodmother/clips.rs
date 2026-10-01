//! Every animation the Broodmother has, as poses and timing.
//!
//! Baked into `crates/sim/src/species/broodmother/baked.rs` by
//! `cargo run -p anim --bin bake_beast -- --species broodmother`.
//!
//! The house style is the Ridgeback's: **the windup is the move** (most of
//! the frames before the hit, the silhouette changing early and a lot), **the
//! hit is short and enormous**, and **the recovery is the fight**. A spider's
//! outline is unusually good at saying what is coming, because most of her
//! moves change the shape of her against the cave's light rather than the
//! angle of one limb (§6): the abdomen *up and held* is the slam, the abdomen
//! *tipped down toward you* the web shot, the thorax *rearing* the screech,
//! the thorax *dropping* the lunge, the abdomen *raised high with the legs
//! gathered* the web line. The one-leg stab is lifted by her own layer at
//! run time, toward the disc it will land on; its clip is the body's half.
//!
//! Every foot in every key is planted (`BroodPose::plant`), so no pose puts a
//! foot through the floor however the body moves.

use super::{BroodPose, home, mark, tetrapod};
use crate::beast::{Feel, Key, Looseness, Pose, Recipe};
use crate::ease::Ease;
use sim::species::broodmother::{Clip, LEG_COUNT, bones};
use std::f32::consts::TAU;

pub fn all() -> Vec<Recipe> {
    vec![
        idle(),
        walk(),
        run(),
        stab(),
        lunge(),
        slam(),
        screech(),
        web_shot(),
        web_line(),
        flinch(),
        stumble(),
        collapse(),
        dead(),
    ]
}

/// Her heavy moves: the abdomen swings through more than a right angle, and
/// a ringing spring on a swing that size sends it through the floor and back.
/// The waist and the abdomen are damped nearly dead; the rest is `HEAVY`.
const SWING: Looseness = Looseness {
    body: Feel::new(3.0, 0.75),
    neck: Feel::new(4.0, 0.5),
    tail: Feel::new(3.0, 1.0),
    legs: Feel::new(1.6, 0.95),
};

fn rest() -> Pose {
    Pose::rest(bones::COUNT)
}

/// Every foot at its stance, pushed out by `spread` metres along its own
/// bearing and lifted by `lift`.
fn feet(p: Pose, spread: f32, lift: f32) -> Pose {
    (0..LEG_COUNT).fold(p, |p, leg| {
        let h = home(leg);
        let r = (h[0] * h[0] + h[2] * h[2]).sqrt();
        let k = (r + spread) / r;
        p.plant(leg, [h[0] * k, lift, h[2] * k])
    })
}

// ---------------------------------------------------------------------------
// Standing and moving
// ---------------------------------------------------------------------------

fn idle() -> Recipe {
    // She breathes through the abdomen: it rises and settles a hand's width,
    // and the thorax with it a little. Eight feet stay where they are.
    let breathe = |k: f32| {
        rest()
            .hips(0.0, 0.03 * k, 0.0)
            .body(0.4 * k, 0.0, 0.0)
            .head(1.0 * k, 0.0, 0.0)
            .fangs(4.0 + 3.0 * k)
            .abdomen(0.0, -1.5 * k)
            .planted()
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
        "Breathing through the abdomen; the fangs work. Eight feet planted: a \
         spider that stands still is very still, and that is what makes the \
         first leg to lift read."
            .to_string(),
    )
}

/// One leg at a point in its cycle: on the floor for the first half, the body
/// travelling over the foot; then a swing forward, lifted.
fn stride(p: Pose, leg: usize, u: f32, reach: f32, lift: f32) -> Pose {
    let u = (u + tetrapod(leg)).rem_euclid(1.0);
    let h = home(leg);
    let (dx, y) = if u < 0.5 {
        let k = u / 0.5;
        (reach * (0.5 - k), 0.0)
    } else {
        let k = (u - 0.5) / 0.5;
        (reach * (k - 0.5), lift * (k * std::f32::consts::PI).sin())
    };
    p.plant(leg, [h[0] + dx, y, h[2]])
}

fn gait(clip: Clip, reach: f32, lift: f32, low: f32, sway: f32, notes: &str) -> Recipe {
    let at = |t: f32| {
        let bob = -low - 0.05 * (2.0 * t * TAU).cos();
        let roll = sway * (t * TAU).sin();
        let mut p = rest()
            .hips(0.0, bob, 0.0)
            .body(-1.0, 0.0, roll)
            .head(-2.0, -roll, 0.0)
            .fangs(4.0)
            .abdomen(0.0, 1.5 * (2.0 * t * TAU).sin());
        for leg in 0..LEG_COUNT {
            p = stride(p, leg, t, reach, lift);
        }
        p
    };
    Recipe::new(
        clip,
        (0..16)
            .map(|i| {
                let t = i as f32 / 16.0;
                Key::eased(t, at(t), Ease::LINEAR)
            })
            .collect(),
        Looseness::GAIT,
    )
    .noting(notes.to_string())
}

fn walk() -> Recipe {
    gait(
        Clip::Walk,
        1.5,
        0.7,
        0.0,
        1.2,
        "Two alternating tetrapods, four feet down at every instant. A spider \
         does not lumber: the body glides level and the legs do the work.",
    )
}

fn run() -> Recipe {
    gait(
        Clip::Run,
        1.5,
        1.1,
        0.35,
        2.0,
        "The enraged run: the same tetrapods, the body lower and the feet \
         thrown higher. Nothing to protect, so nothing held back.",
    )
}

// ---------------------------------------------------------------------------
// The moves
// ---------------------------------------------------------------------------

fn stab() -> Recipe {
    // The body's half of a stab: she settles onto the seven legs that stay,
    // then drops her weight into the one that drives. The leg is her layer's.
    let set = rest()
        .hips(0.0, -0.15, 0.0)
        .body(2.0, 0.0, 0.0)
        .fangs(10.0)
        .planted();
    let drive = rest()
        .hips(0.0, -0.4, 0.0)
        .body(-4.0, 0.0, 0.0)
        .fangs(6.0)
        .planted();
    let m = |phase, u| mark(Clip::Stab, phase, u);
    Recipe::new(
        Clip::Stab,
        vec![
            Key::at(0.0, Pose::standing()),
            Key::eased(m(0, 0.6), set, Ease::IN),
            Key::eased(m(1, 0.0), drive, Ease::OUT),
            Key::at(m(2, 0.4), drive),
            Key::eased(1.0, Pose::standing(), Ease::SMOOTH),
        ],
        Looseness::SNAP,
    )
    .noting(
        "The body settles onto seven legs and drops its weight into the eighth. \
         Which leg is decided as the move commits; her layer lifts it toward the \
         disc it will land on."
            .to_string(),
    )
}

fn lunge() -> Recipe {
    // The thorax drops, the fangs spread, the front of her gathers: then she
    // throws it forward, and lies there for a second and a half.
    let crouch = feet(
        rest()
            .hips(-0.5, -0.9, 0.0)
            .body(-10.0, 0.0, 0.0)
            .head(-12.0, 0.0, 0.0)
            .fangs(40.0)
            .abdomen(0.0, -6.0),
        -0.3,
        0.0,
    );
    let thrown = feet(
        rest()
            .hips(1.4, -1.0, 0.0)
            .body(-6.0, 0.0, 0.0)
            .head(-4.0, 0.0, 0.0)
            .fangs(-5.0)
            .abdomen(4.0, 0.0),
        0.3,
        0.0,
    );
    let spent = feet(
        rest()
            .hips(1.2, -1.1, 0.0)
            .body(-8.0, 0.0, 0.0)
            .head(-10.0, 0.0, 0.0)
            .fangs(10.0)
            .abdomen(4.0, 4.0),
        0.3,
        0.0,
    );
    let m = |phase, u| mark(Clip::Lunge, phase, u);
    Recipe::new(
        Clip::Lunge,
        vec![
            Key::at(0.0, Pose::standing()),
            Key::eased(m(0, 0.5), crouch, Ease::OUT),
            Key::eased(m(0, 1.0), crouch, Ease::IN),
            Key::eased(m(1, 1.0), thrown, Ease::OUT),
            Key::at(m(2, 0.7), spent),
            Key::eased(1.0, Pose::standing(), Ease::SMOOTH),
        ],
        Looseness::SNAP,
    )
    .noting(
        "Thorax down, fangs spread, the front of her gathered for the whole \
         windup; then the throw. The recovery is ninety frames of her lying \
         out flat at the end of it, which is the only time the front of her \
         is open."
            .to_string(),
    )
}

/// The abdomen down on the floor, the slam's crash: the waist folded back
/// and the abdomen lying on its belly, sunk into the floor as far as a
/// crashed body squashes. `tail` tips it tail-down by so many degrees.
fn crashed(tail: f32) -> Pose {
    feet(
        rest()
            .hips(-0.2, -1.2, 0.0)
            .body(-3.0, 0.0, 0.0)
            .head(-6.0, 0.0, 0.0)
            .fangs(20.0)
            .abdomen(118.0, -118.0 + tail),
        0.4,
        0.0,
    )
}

fn slam() -> Recipe {
    // Up, and held at the top: the tell is the hold. Then everything comes
    // down at once, and stays down for most of the recovery -- that is the
    // window -- before it lifts in the last twenty frames.
    let up = feet(
        rest()
            .hips(0.3, 0.3, 0.0)
            .body(6.0, 0.0, 0.0)
            .head(8.0, 0.0, 0.0)
            .fangs(10.0)
            .abdomen(-22.0, -14.0),
        -0.2,
        0.0,
    );
    let top = feet(
        rest()
            .hips(0.35, 0.35, 0.0)
            .body(7.0, 0.0, 0.0)
            .head(9.0, 0.0, 0.0)
            .fangs(12.0)
            .abdomen(-26.0, -16.0),
        -0.2,
        0.0,
    );
    let down = crashed(8.0);
    let m = |phase, u| mark(Clip::Slam, phase, u);
    Recipe::new(
        Clip::Slam,
        vec![
            Key::at(0.0, Pose::standing()),
            Key::eased(m(0, 0.45), up, Ease::OUT),
            Key::eased(m(0, 0.9), top, Ease::IN),
            Key::eased(m(1, 0.6), down, Ease::OUT),
            Key::at(m(2, 0.75), crashed(9.0)),
            Key::eased(1.0, Pose::standing(), Ease::SMOOTH),
        ],
        SWING,
    )
    .noting(
        "The abdomen up and held: the one silhouette in the fight that says \
         'get out from under it'. The crash folds the waist back and lays the \
         abdomen on the floor, sunk as a soft body squashes, so every sac on \
         it is at a standing fighter's reach until the lift in the last \
         twenty frames."
            .to_string(),
    )
}

fn screech() -> Recipe {
    // The thorax rears, the fangs go up and open, and she holds it,
    // trembling, for the forty frames of the scream.
    let rear = |k: f32| {
        feet(
            rest()
                .hips(-0.3, 0.6, 0.0)
                .body(24.0 + k, 0.0, k * 0.5)
                .head(18.0, 0.0, 0.0)
                .fangs(45.0)
                .abdomen(-8.0, 6.0),
            0.0,
            0.0,
        )
    };
    let m = |phase, u| mark(Clip::Screech, phase, u);
    let mut keys = vec![
        Key::at(0.0, Pose::standing()),
        Key::eased(m(0, 0.7), rear(0.0), Ease::OUT),
    ];
    for i in 0..6 {
        let u = i as f32 / 5.0;
        keys.push(Key::at(m(1, u), rear(if i % 2 == 0 { 2.0 } else { -2.0 })));
    }
    keys.push(Key::eased(1.0, Pose::standing(), Ease::SMOOTH));
    Recipe::new(Clip::Screech, keys, Looseness::BEAST).noting(
        "Thorax up, fangs up and open, and held trembling for the scream. The \
         slam follows every time, so this is the hundred-frame tell for the \
         window, read from anywhere in the cave by the shape of her front."
            .to_string(),
    )
}

fn web_shot() -> Recipe {
    // The abdomen tips down toward whoever is behind her, the spinnerets
    // pointed: a silhouette from any angle. A pump, and back.
    let aim = feet(
        rest()
            .hips(0.2, 0.1, 0.0)
            .body(-6.0, 0.0, 0.0)
            .fangs(6.0)
            .abdomen(8.0, 20.0),
        0.0,
        0.0,
    );
    let pump = feet(
        rest()
            .hips(0.3, 0.0, 0.0)
            .body(-8.0, 0.0, 0.0)
            .fangs(6.0)
            .abdomen(10.0, 28.0),
        0.0,
        0.0,
    );
    let m = |phase, u| mark(Clip::WebShot, phase, u);
    Recipe::new(
        Clip::WebShot,
        vec![
            Key::at(0.0, Pose::standing()),
            Key::eased(m(0, 0.7), aim, Ease::OUT),
            Key::eased(m(1, 0.0), pump, Ease::IN),
            Key::at(m(2, 0.3), aim),
            Key::eased(1.0, Pose::standing(), Ease::SMOOTH),
        ],
        Looseness::BEAST,
    )
    .noting(
        "The abdomen tipped down toward a target behind her: her ranged move \
         comes out of her back, and this is the tell for it from any side."
            .to_string(),
    )
}

fn web_line() -> Recipe {
    // Her back to the anchor and the abdomen raised high, the legs gathered
    // under her; then reeled across with every leg tucked, body first.
    let ready = feet(
        rest()
            .hips(0.0, 0.2, 0.0)
            .body(-6.0, 0.0, 0.0)
            .head(-4.0, 0.0, 0.0)
            .fangs(8.0)
            .abdomen(-14.0, -24.0),
        -0.8,
        0.0,
    );
    let tucked = feet(
        rest()
            .hips(0.0, 0.6, 0.0)
            .body(-4.0, 0.0, 0.0)
            .head(-4.0, 0.0, 0.0)
            .fangs(4.0)
            .abdomen(-14.0, -20.0),
        -1.6,
        1.2,
    );
    let m = |phase, u| mark(Clip::WebLine, phase, u);
    Recipe::new(
        Clip::WebLine,
        vec![
            Key::at(0.0, Pose::standing()),
            Key::eased(m(0, 0.7), ready, Ease::OUT),
            Key::eased(m(1, 0.05), tucked, Ease::OUT),
            Key::at(m(1, 1.0), tucked),
            Key::eased(m(2, 0.4), ready, Ease::OUT),
            Key::eased(1.0, Pose::standing(), Ease::SMOOTH),
        ],
        Looseness::BEAST,
    )
    .noting(
        "Back to the wall and the abdomen up; then reeled across the cave, \
         every leg tucked under her, body first. The strands she leaves are \
         drawn on the floor; this is the shape that says they are coming."
            .to_string(),
    )
}

fn flinch() -> Recipe {
    let hit = feet(
        rest()
            .hips(-0.2, -0.2, 0.0)
            .body(6.0, 0.0, 3.0)
            .head(10.0, 0.0, 0.0)
            .fangs(30.0)
            .abdomen(-6.0, 4.0),
        0.0,
        0.0,
    );
    Recipe::new(
        Clip::Flinch,
        vec![
            Key::at(0.0, Pose::standing()),
            Key::eased(0.25, hit, Ease::OUT),
            Key::eased(1.0, Pose::standing(), Ease::SMOOTH),
        ],
        Looseness::BEAST,
    )
    .noting("A recoil: front up, fangs open, the abdomen jerked.".to_string())
}

fn stumble() -> Recipe {
    // Down at the front: the thorax to the floor's reach and the waist
    // exposed under the raised abdomen.
    let down = feet(
        rest()
            .hips(0.0, -1.0, 0.0)
            .body(-6.0, 0.0, 8.0)
            .head(-10.0, 0.0, 0.0)
            .fangs(25.0)
            .abdomen(20.0, -14.0),
        0.5,
        0.0,
    );
    Recipe::new(
        Clip::Stumble,
        vec![
            Key::at(0.0, Pose::standing()),
            Key::eased(0.15, down, Ease::OUT),
            Key::at(0.75, down),
            Key::eased(1.0, Pose::standing(), Ease::SMOOTH),
        ],
        Looseness::COLLAPSE,
    )
    .noting(
        "A leg gone, or control while she was hurting: down at the front, the \
         thorax at a fighter's chest and the waist open beneath the abdomen."
            .to_string(),
    )
}

/// Flat on the floor: every leg out, the thorax on the ground and the
/// abdomen behind it, the waist low.
fn flat(curl: f32) -> Pose {
    let p = rest()
        .hips(0.0, -1.9, 0.0)
        .body(-2.0, 0.0, 0.0)
        .head(-6.0, 0.0, 0.0)
        .fangs(30.0)
        .abdomen(100.0, -100.0);
    feet(p, 0.9 - curl, curl)
}

fn collapse() -> Recipe {
    Recipe::new(
        Clip::Collapse,
        vec![
            Key::at(0.0, Pose::standing()),
            Key::eased(0.12, flat(0.0), Ease::OUT),
            Key::at(0.85, flat(0.0)),
            Key::eased(1.0, Pose::standing(), Ease::SMOOTH),
        ],
        Looseness::COLLAPSE,
    )
    .noting(
        "The last sac gone: she drops flat, every leg out, and the waist is at \
         a fighter's chest for four seconds. What gets up is the enraged run."
            .to_string(),
    )
}

fn dead() -> Recipe {
    Recipe::new(
        Clip::Dead,
        vec![Key::at(0.0, flat(1.4)), Key::at(1.0, flat(1.4))],
        Looseness::COLLAPSE,
    )
    .noting("On her belly with the legs curled under, as spiders die.".to_string())
}
