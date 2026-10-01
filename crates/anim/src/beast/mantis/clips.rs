//! Every animation the Mantis has, as poses and timing.
//!
//! Baked into `crates/sim/src/species/mantis/baked.rs` by
//! `cargo run -p anim --bin bake_beast -- --species mantis`.
//!
//! The house style is the Ridgeback's (`ridgeback/clips.rs`): **the windup is
//! the move**, the hit is short and enormous, and the recovery is long and
//! readable. Three things are the Mantis's own:
//!
//! 1. **Four stances, four silhouettes** (§6). Guard: both blades crossed
//!    before the head, tall. Coil: the body dropped, the blades swept back over
//!    the shoulders, the abdomen raised. Ready: one blade cocked out wide, the
//!    head tilted to it. Prayer: arms folded tight, head bowed, wings half out
//!    and still. Each one changes the outline from across the court.
//! 2. **The recoveries put the blades on the floor.** A missed scythe, a
//!    missed lunge and a broken guard all leave a blade lying out low for the
//!    first part of the recovery -- that is where the arm is hit (§4), so the
//!    choice between the body and the arm is a place to stand.
//! 3. **The hit is a line, not a sweep, where the move is a line.** The lunge
//!    and the counter are thrusts: the arm and blade straight out along the
//!    facing on the first live frame.

use super::{LEFT, MantisPose, RIGHT, mark};
use crate::beast::{Feel, Key, Looseness, Pose, Recipe};
use crate::ease::Ease;
use sim::species::mantis::Clip;

pub fn all() -> Vec<Recipe> {
    vec![
        idle(),
        walk(),
        skitter(),
        slash(),
        slash2(),
        lunge(),
        guard(),
        counter(),
        leap(),
        dive(),
        flare(),
        pivot(),
        prayer(),
        ready(),
        broken(),
        stagger(),
        flinch(),
        stumble(),
        topple(),
        dead(),
    ]
}

/// A duellist's: tight and quick in the body, loose only at the blade tips.
const QUICK: Looseness = Looseness {
    body: Feel::new(1.2, 0.95),
    neck: Feel::new(1.8, 0.75),
    tail: Feel::new(1.5, 0.80),
    legs: Feel::new(1.2, 0.95),
};

/// A strike: the blade snaps out and carries past.
const CUT: Looseness = Looseness {
    body: Feel::new(1.4, 0.90),
    neck: Feel::new(2.0, 0.70),
    tail: Feel::new(1.1, 1.05),
    legs: Feel::new(1.2, 0.95),
};

/// Held: a stance, nothing ringing.
const STILL: Looseness = Looseness {
    body: Feel::new(2.0, 0.85),
    neck: Feel::new(2.6, 0.70),
    tail: Feel::new(2.2, 0.75),
    legs: Feel::new(1.4, 0.95),
};

fn ready_pose() -> Pose {
    Pose::standing()
}

// ---------------------------------------------------------------------------
// Standing and moving
// ---------------------------------------------------------------------------

fn idle() -> Recipe {
    let breathe = |k: f32| {
        Pose::standing()
            .thorax(-4.0 + 1.0 * k, 0.0, 0.0)
            .head(-6.0 + 2.0 * k, 4.0 * k, 0.0)
            .arms(22.0 + 2.0 * k, 6.0, 158.0 - 3.0 * k)
            .abdomen(2.0 * k)
            .wings(2.0 + 2.0 * k, 0.0)
    };
    Recipe::new(
        Clip::Idle,
        vec![
            Key::at(0.0, breathe(-1.0)),
            Key::at(0.5, breathe(1.0)),
            Key::at(0.8, breathe(0.2)),
        ],
        STILL,
    )
    .noting(
        "Folded and swaying a little, the head turning: a mantis at rest is \
         never quite still, and the stillness of a stance is the read that \
         something has begun."
            .to_string(),
    )
}

/// One leg at a point in its own cycle: planted for `stance` of it and
/// swinging for the rest.
fn stride(p: Pose, which: usize, u: f32, reach: f32, lift: f32, stance: f32) -> Pose {
    let u = u.rem_euclid(1.0);
    let home = super::STANCE[which];
    if u < stance {
        let k = u / stance;
        p.plant(which, home + reach * (1.0 - 2.0 * k), 0.0)
    } else {
        let k = (u - stance) / (1.0 - stance);
        let arc = (k * std::f32::consts::PI).sin();
        p.plant(which, home + reach * (2.0 * k - 1.0), lift * arc)
    }
}

/// The diagonal pairs: middle left with hind right, then the other two.
const PHASE: [f32; 4] = [0.0, 0.5, 0.5, 0.0];

fn gait(
    clip: Clip,
    reach: f32,
    lift: f32,
    stance: f32,
    low: f32,
    lean: f32,
    notes: &str,
) -> Recipe {
    let at = |t: f32| {
        let wheel = t * std::f32::consts::TAU;
        let bob = -low - 0.03 * (2.0 * wheel).cos();
        let sway = 3.0 * wheel.sin();
        let mut p = Pose::standing()
            .hips(0.0, bob, 0.0)
            .thorax(-4.0 - lean, sway * 0.5, 0.0)
            .head(-6.0 + lean * 0.5, -sway, 0.0)
            .arms(22.0 - lean * 0.3, 6.0, 158.0)
            .abdomen(lean * 0.4 + sway * 0.3);
        for (which, offset) in PHASE.into_iter().enumerate() {
            p = stride(p, which, t + offset, reach, lift, stance);
        }
        p
    };
    Recipe::new(
        clip,
        (0..8)
            .map(|i| {
                let t = i as f32 / 8.0;
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
        0.50,
        0.20,
        0.6,
        0.0,
        2.0,
        "Diagonal pairs, the body carried level and upright over them, the \
         arms folded and still: it walks like something that is watching you.",
    )
}

fn skitter() -> Recipe {
    gait(
        Clip::Skitter,
        0.45,
        0.28,
        0.5,
        0.25,
        16.0,
        "The skitter: low, leaning in, legs a blur. Twelve metres a second to \
         close a gap -- the body drops and tips forward so the change of gear \
         reads before the distance does.",
    )
}

// ---------------------------------------------------------------------------
// The scythes
// ---------------------------------------------------------------------------

/// The right blade cocked out wide and back, the body turned away from it.
fn cocked_right() -> Pose {
    Pose::standing()
        .crouch(0.10)
        .thorax(-6.0, -16.0, 0.0)
        .head(-8.0, 10.0, 0.0)
        .arm(RIGHT, 38.0, 72.0, 70.0)
        .arm(LEFT, 26.0, 10.0, 150.0)
}

/// The right blade swept right across the front, low and straight.
fn cut_right() -> Pose {
    Pose::standing()
        .crouch(0.16)
        .thorax(-14.0, 18.0, 0.0)
        .head(-10.0, -6.0, 0.0)
        .arm(RIGHT, 2.0, -38.0, 8.0)
        .arm(LEFT, 30.0, 18.0, 120.0)
}

/// The left blade cocked, the right lying out after its cut.
fn cocked_left() -> Pose {
    Pose::standing()
        .crouch(0.14)
        .thorax(-8.0, 16.0, 0.0)
        .head(-8.0, -10.0, 0.0)
        .arm(LEFT, 40.0, 76.0, 66.0)
        .arm(RIGHT, -6.0, -24.0, 20.0)
}

/// The left blade swept across.
fn cut_left() -> Pose {
    Pose::standing()
        .crouch(0.16)
        .thorax(-14.0, -18.0, 0.0)
        .head(-10.0, 6.0, 0.0)
        .arm(LEFT, 2.0, -38.0, 8.0)
        .arm(RIGHT, -4.0, -10.0, 24.0)
}

/// After the pair: both blades lying out low on the floor in front.
fn blades_down() -> Pose {
    Pose::standing()
        .crouch(0.22)
        .thorax(-22.0, 0.0, 0.0)
        .head(-12.0, 0.0, 0.0)
        .arms(12.0, 4.0, 35.0)
}

fn slash() -> Recipe {
    Recipe::new(
        Clip::Slash,
        vec![
            Key::eased(0.0, ready_pose(), Ease::OUT),
            Key::eased(mark(Clip::Slash, 0, 0.55), cocked_right(), Ease::HOLD),
            Key::eased(mark(Clip::Slash, 0, 1.0), cocked_right(), Ease::STRIKE),
            Key::eased(mark(Clip::Slash, 1, 0.5), cut_right(), Ease::SMOOTH),
            Key::at(1.0, cut_right()),
        ],
        CUT,
    )
    .noting(
        "The first of the pair: the right blade out wide and back for most of \
         eighteen frames -- the whole outline leans away from it -- then \
         across in four, with a step. It ends in the cut and the second \
         begins from there."
            .to_string(),
    )
}

fn slash2() -> Recipe {
    Recipe::new(
        Clip::Slash2,
        vec![
            Key::eased(0.0, cut_right(), Ease::OUT),
            Key::eased(mark(Clip::Slash2, 0, 0.7), cocked_left(), Ease::HOLD),
            Key::eased(mark(Clip::Slash2, 0, 1.0), cocked_left(), Ease::STRIKE),
            Key::eased(mark(Clip::Slash2, 1, 0.6), cut_left(), Ease::OUT),
            // The window: both blades lie out low on the floor.
            Key::eased(mark(Clip::Slash2, 2, 0.0), blades_down(), Ease::HOLD),
            Key::eased(mark(Clip::Slash2, 2, 0.65), blades_down(), Ease::SMOOTH),
            Key::at(1.0, ready_pose()),
        ],
        CUT,
    )
    .noting(
        "The second: the left blade cocked from the end of the first -- held \
         twelve frames or twenty-six, the same pose stretched, which is the \
         mixup -- then across. Then both blades lie out low on the floor for \
         two thirds of the recovery: the arm is there to be hit."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// The coil and the lunge
// ---------------------------------------------------------------------------

/// The coil: dropped, the blades swept back over the shoulders, the abdomen
/// up. The silhouette everybody must learn.
fn coiled(depth: f32) -> Pose {
    Pose::standing()
        .crouch(0.38 * depth)
        .thorax(-18.0 * depth, 0.0, 0.0)
        .head(-14.0 * depth, 0.0, 0.0)
        .arms(22.0 + 96.0 * depth, 14.0 * depth, 158.0 - 8.0 * depth)
        .abdomen(20.0 * depth)
        .wings(6.0 * depth, 8.0 * depth)
}

/// The thrust: both blades straight out along the facing, the body flat
/// behind them.
fn thrust() -> Pose {
    Pose::standing()
        .crouch(0.30)
        .thorax(-48.0, 0.0, 0.0)
        .head(30.0, 0.0, 0.0)
        .arms(44.0, -6.0, 0.0)
        .abdomen(-6.0)
        .wings(10.0, 4.0)
}

fn lunge() -> Recipe {
    Recipe::new(
        Clip::Lunge,
        vec![
            Key::eased(0.0, ready_pose(), Ease::OUT),
            Key::eased(mark(Clip::Lunge, 0, 0.25), coiled(1.0), Ease::SMOOTH),
            // Held through the coil, sinking a little as it runs long.
            Key::eased(mark(Clip::Lunge, 0, 0.96), coiled(1.12), Ease::STRIKE),
            Key::eased(mark(Clip::Lunge, 1, 0.3), thrust(), Ease::HOLD),
            Key::eased(mark(Clip::Lunge, 2, 0.0), thrust(), Ease::OUT),
            Key::eased(mark(Clip::Lunge, 2, 0.4), blades_down(), Ease::HOLD),
            Key::eased(mark(Clip::Lunge, 2, 0.7), blades_down(), Ease::SMOOTH),
            Key::at(1.0, ready_pose()),
        ],
        CUT,
    )
    .noting(
        "Into the coil in a quarter of its longest length and then held: the \
         body down, the blades back over the shoulders, the abdomen up -- \
         sinking a little more the longer it runs, which is the chitter \
         rising. The release is a thrust, not a swing, and the recovery lays \
         the blades on the floor where the lunge ran out."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// The guard and its branches
// ---------------------------------------------------------------------------

/// Both blades crossed before the head; tall.
fn guarding() -> Pose {
    Pose::standing()
        .hips(0.0, 0.04, 0.0)
        .planted()
        .thorax(2.0, 0.0, 0.0)
        .head(-4.0, 0.0, 0.0)
        .arms(46.0, -26.0, 112.0)
}

fn guard() -> Recipe {
    Recipe::new(
        Clip::Guard,
        vec![
            Key::eased(0.0, guarding(), Ease::OUT),
            Key::eased(mark(Clip::Guard, 1, 0.08), guarding(), Ease::HOLD),
            Key::eased(mark(Clip::Guard, 2, 0.0), guarding(), Ease::SMOOTH),
            Key::at(1.0, ready_pose()),
        ],
        QUICK,
    )
    .noting(
        "Up in its first frames -- the guard has no startup, so the pose is \
         there before anything else -- and held: both blades crossed before \
         the head, tall. Lowering is sixteen frames back to the fold, and \
         that is the window."
            .to_string(),
    )
}

fn counter() -> Recipe {
    let drawn = Pose::standing()
        .crouch(0.12)
        .thorax(-4.0, -10.0, 0.0)
        .arm(RIGHT, 18.0, 6.0, 128.0)
        .arm(LEFT, 44.0, -24.0, 110.0);
    let out = Pose::standing()
        .crouch(0.18)
        .thorax(-26.0, 8.0, 0.0)
        .head(10.0, 0.0, 0.0)
        .arm(RIGHT, -4.0, -4.0, 0.0)
        .arm(LEFT, 40.0, -20.0, 110.0);
    Recipe::new(
        Clip::Counter,
        vec![
            Key::eased(0.0, guarding(), Ease::OUT),
            Key::eased(mark(Clip::Counter, 0, 0.9), drawn, Ease::STRIKE),
            Key::eased(mark(Clip::Counter, 1, 0.5), out, Ease::HOLD),
            Key::eased(mark(Clip::Counter, 2, 0.35), out, Ease::SMOOTH),
            Key::at(1.0, ready_pose()),
        ],
        CUT,
    )
    .noting(
        "Out of the guard: the right blade drawn back off the cross, then \
         straight out along the parried line. Twelve frames is below \
         reaction on purpose -- the answer is not giving it the parry -- and \
         the left blade stays up in the guard the whole time."
            .to_string(),
    )
}

fn ready() -> Recipe {
    let cocked = Pose::standing()
        .thorax(-2.0, -10.0, 6.0)
        .head(-2.0, 22.0, 16.0)
        .arm(RIGHT, 24.0, 84.0, 74.0)
        .arm(LEFT, 26.0, 6.0, 154.0);
    Recipe::new(
        Clip::Ready,
        vec![
            Key::eased(0.0, ready_pose(), Ease::OUT),
            Key::eased(mark(Clip::Ready, 0, 1.0), cocked, Ease::HOLD),
            Key::eased(mark(Clip::Ready, 2, 0.0), cocked, Ease::SMOOTH),
            Key::at(1.0, ready_pose()),
        ],
        QUICK,
    )
    .noting(
        "One blade cocked out wide on one side, the head tilted to it: the \
         habit stance, taken in eight frames and held. Mirrored to the side \
         the remembered move arrives from."
            .to_string(),
    )
}

fn prayer() -> Recipe {
    let folded = |k: f32| {
        Pose::standing()
            .crouch(0.06)
            .thorax(2.0, 0.0, 0.0)
            .head(-28.0 - 2.0 * k, 0.0, 0.0)
            .arms(52.0, -10.0, 172.0)
            .wings(24.0, 6.0)
            .abdomen(4.0)
    };
    Recipe::new(
        Clip::Prayer,
        vec![
            Key::eased(0.0, folded(0.0), Ease::OUT),
            Key::eased(mark(Clip::Prayer, 1, 0.5), folded(1.0), Ease::SMOOTH),
            Key::eased(mark(Clip::Prayer, 2, 0.0), folded(0.0), Ease::SMOOTH),
            Key::at(1.0, ready_pose()),
        ],
        STILL,
    )
    .noting(
        "Arms folded tight and high, head bowed, wings half out and still. A \
         parry all round the whole time; the only motion is the head sinking \
         a little with the hum."
            .to_string(),
    )
}

/// Blades flung wide and low, the body rocked back.
fn flung() -> Pose {
    Pose::standing()
        .crouch(0.20)
        .thorax(12.0, 0.0, 0.0)
        .head(18.0, 0.0, 0.0)
        .arms(-24.0, 74.0, 12.0)
        .wings(18.0, -4.0)
}

fn broken() -> Recipe {
    Recipe::new(
        Clip::Broken,
        vec![
            Key::eased(0.0, guarding(), Ease::STRIKE),
            Key::eased(mark(Clip::Broken, 2, 0.06), flung(), Ease::HOLD),
            Key::eased(mark(Clip::Broken, 2, 0.75), flung(), Ease::SMOOTH),
            Key::at(1.0, ready_pose()),
        ],
        Looseness::LIMP,
    )
    .noting(
        "The guard broken: both blades flung wide and low, the arms at a \
         metre to a metre and a half, the body rocked back on its legs. Held \
         for three quarters of the stagger -- the window, and the arms are in \
         it -- then gathered back up."
            .to_string(),
    )
}

fn stagger() -> Recipe {
    let rocked = Pose::standing()
        .crouch(0.16)
        .thorax(22.0, 0.0, 0.0)
        .head(20.0, 0.0, 0.0)
        .arms(60.0, 30.0, 60.0)
        .abdomen(-10.0);
    Recipe::new(
        Clip::Stagger,
        vec![
            Key::eased(0.0, thrust(), Ease::STRIKE),
            Key::eased(mark(Clip::Stagger, 2, 0.15), rocked, Ease::HOLD),
            Key::eased(mark(Clip::Stagger, 2, 0.6), rocked, Ease::SMOOTH),
            Key::at(1.0, ready_pose()),
        ],
        Looseness::LIMP,
    )
    .noting(
        "Into a wall at full stretch: thrown back off it, arms up, and a long \
         moment to gather."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// In the air
// ---------------------------------------------------------------------------

fn leap() -> Recipe {
    let dropped = Pose::standing()
        .crouch(0.42)
        .thorax(-10.0, 0.0, 0.0)
        .arms(30.0, -10.0, 168.0)
        .abdomen(10.0);
    let rising = Pose::rest(sim::species::mantis::bones::COUNT)
        .thorax(10.0, 0.0, 0.0)
        .head(10.0, 0.0, 0.0)
        .arms(84.0, 24.0, 10.0)
        .wings(40.0, 20.0)
        .abdomen(-14.0);
    Recipe::new(
        Clip::Leap,
        vec![
            Key::eased(0.0, ready_pose(), Ease::OUT),
            Key::eased(mark(Clip::Leap, 0, 0.85), dropped, Ease::STRIKE),
            Key::eased(mark(Clip::Leap, 1, 0.4), rising, Ease::SMOOTH),
            Key::at(1.0, rising),
        ],
        CUT,
    )
    .noting(
        "Drops for its whole startup -- twenty frames of a body going down \
         is the tell -- then springs, blades thrown up overhead, cutting all \
         the way up. The simulation carries the body six metres up; the clip \
         is the limbs."
            .to_string(),
    )
}

fn dive() -> Recipe {
    let hang = Pose::rest(sim::species::mantis::bones::COUNT)
        .thorax(18.0, 0.0, 0.0)
        .head(14.0, 0.0, 0.0)
        .arms(96.0, 30.0, 40.0)
        .wings(56.0, 26.0)
        .abdomen(-20.0);
    let down = Pose::rest(sim::species::mantis::bones::COUNT)
        .thorax(-40.0, 0.0, 0.0)
        .head(24.0, 0.0, 0.0)
        .arms(-30.0, -6.0, 0.0)
        .wings(30.0, 10.0)
        .abdomen(16.0);
    let landed = Pose::standing()
        .crouch(0.40)
        .thorax(-30.0, 0.0, 0.0)
        .arms(16.0, 10.0, 36.0)
        .abdomen(8.0);
    Recipe::new(
        Clip::Dive,
        vec![
            Key::eased(0.0, hang, Ease::HOLD),
            Key::eased(mark(Clip::Dive, 0, 1.0), hang, Ease::STRIKE),
            Key::eased(mark(Clip::Dive, 1, 0.5), down, Ease::IN),
            Key::eased(mark(Clip::Dive, 2, 0.0), landed, Ease::HOLD),
            Key::eased(mark(Clip::Dive, 2, 0.6), landed, Ease::SMOOTH),
            Key::at(1.0, ready_pose()),
        ],
        CUT,
    )
    .noting(
        "The hang at the top -- wings open, blades back -- is the dive's own \
         tell; then head-first down onto its circle, blades leading, and a \
         long crouched landing: fifty-five frames of it."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// The close game: the flare and the pivot
// ---------------------------------------------------------------------------

fn flare() -> Recipe {
    let gather = Pose::standing()
        .crouch(0.12)
        .arms(40.0, -8.0, 150.0)
        .wings(14.0, 10.0)
        .abdomen(12.0);
    let open = Pose::standing()
        .crouch(0.04)
        .thorax(10.0, 0.0, 0.0)
        .head(14.0, 0.0, 0.0)
        .arms(70.0, 60.0, 60.0)
        .wings(86.0, 34.0)
        .abdomen(24.0);
    let hop = Pose::standing()
        .crouch(0.26)
        .thorax(6.0, 0.0, 0.0)
        .arms(30.0, 20.0, 140.0)
        .wings(30.0, 10.0);
    Recipe::new(
        Clip::Flare,
        vec![
            Key::eased(0.0, ready_pose(), Ease::OUT),
            Key::eased(mark(Clip::Flare, 0, 1.0), gather, Ease::STRIKE),
            Key::eased(mark(Clip::Flare, 1, 0.5), open, Ease::OUT),
            Key::eased(mark(Clip::Flare, 2, 0.5), hop, Ease::SMOOTH),
            Key::at(1.0, ready_pose()),
        ],
        CUT,
    )
    .noting(
        "Wings snap open, the arms thrown wide: the silhouette doubles in \
         three frames. Then the hop back, crouched, wings closing -- the \
         spacing reset, and twenty-four frames to walk after it."
            .to_string(),
    )
}

fn pivot() -> Recipe {
    let planted = Pose::standing()
        .crouch(0.14)
        .root(0.0, -30.0, 0.0)
        .thorax(-6.0, -12.0, 0.0)
        .arm(RIGHT, 30.0, 40.0, 110.0)
        .abdomen(-6.0);
    let whirled = Pose::standing()
        .crouch(0.16)
        .root(0.0, 130.0, 0.0)
        .thorax(-4.0, 20.0, 0.0)
        .arm(RIGHT, -4.0, 92.0, 6.0)
        .arm(LEFT, 30.0, 20.0, 150.0);
    Recipe::new(
        Clip::Pivot,
        vec![
            Key::eased(0.0, ready_pose(), Ease::OUT),
            Key::eased(mark(Clip::Pivot, 0, 1.0), planted, Ease::STRIKE),
            Key::eased(mark(Clip::Pivot, 1, 0.8), whirled, Ease::HOLD),
            Key::eased(mark(Clip::Pivot, 2, 0.3), whirled, Ease::SMOOTH),
            Key::at(1.0, ready_pose()),
        ],
        CUT,
    )
    .noting(
        "Hind legs planted, the body wound the other way, then a whirl with \
         one blade straight out at shoulder height behind it: over a crouch. \
         Mirrored to the side the target is on."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// Trouble
// ---------------------------------------------------------------------------

fn flinch() -> Recipe {
    let hit = Pose::standing()
        .crouch(0.08)
        .thorax(10.0, 6.0, 0.0)
        .head(14.0, -8.0, 0.0)
        .arms(30.0, 20.0, 130.0);
    Recipe::new(
        Clip::Flinch,
        vec![
            Key::eased(0.0, ready_pose(), Ease::STRIKE),
            Key::eased(0.25, hit, Ease::SMOOTH),
            Key::at(1.0, ready_pose()),
        ],
        QUICK,
    )
    .noting("Jolted back and up, and straight back into the fold.".to_string())
}

fn stumble() -> Recipe {
    let down = Pose::standing()
        .crouch(0.46)
        .thorax(-20.0, 0.0, 12.0)
        .head(-16.0, 0.0, 0.0)
        .arms(20.0, 30.0, 40.0)
        .wings(20.0, -6.0)
        .abdomen(-12.0);
    Recipe::new(
        Clip::Stumble,
        vec![
            Key::eased(0.0, ready_pose(), Ease::STRIKE),
            Key::eased(0.15, down, Ease::HOLD),
            Key::eased(0.7, down, Ease::SMOOTH),
            Key::at(1.0, ready_pose()),
        ],
        Looseness::COLLAPSE,
    )
    .noting(
        "Down on its legs, the arms dropped: a blade breaking, or crowd \
         control landing on a creature already hurting."
            .to_string(),
    )
}

fn topple() -> Recipe {
    let over = Pose::standing()
        .crouch(0.50)
        .root(0.0, 0.0, 22.0)
        .thorax(-10.0, 0.0, 14.0)
        .arms(16.0, 50.0, 40.0)
        .wings(30.0, -10.0);
    Recipe::new(
        Clip::Topple,
        vec![
            Key::eased(0.0, ready_pose(), Ease::STRIKE),
            Key::eased(0.15, over, Ease::HOLD),
            Key::eased(0.8, over, Ease::SMOOTH),
            Key::at(1.0, ready_pose()),
        ],
        Looseness::COLLAPSE,
    )
    .noting("Over on its side.".to_string())
}

fn dead() -> Recipe {
    let down = Pose::standing()
        .crouch(0.55)
        .root(0.0, 0.0, 26.0)
        .thorax(-30.0, 0.0, 20.0)
        .head(-30.0, 0.0, 0.0)
        .arms(24.0, 60.0, 50.0)
        .wings(40.0, -14.0);
    Recipe::new(Clip::Dead, vec![Key::at(0.0, down)], Looseness::LIMP)
        .noting("Folded over on its side, the blades out.".to_string())
}
