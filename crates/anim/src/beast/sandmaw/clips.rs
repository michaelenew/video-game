//! Every animation the Sandmaw has, as poses and timing.
//!
//! Baked into `crates/sim/src/species/sandmaw/baked.rs` by
//! `cargo run -p anim --bin bake_beast -- --species sandmaw`.
//!
//! The house style is the Ridgeback's -- **the windup is the move, the hit is
//! short and enormous, the recovery is the fight** -- with one difference a
//! burrower makes: a move from below has its windup **under the sand**, where
//! nothing of the body shows. Its tell is the floor's (the marker, the lane,
//! the ring) and the wake's, and the clip's job through it is only to arrive
//! where the hit starts, buried. Then the hit is the whole body at once: six
//! metres of worm out of a circle of sand in six frames.
//!
//! Standing, it is an ordinary creature: the spit draws the head back for most
//! of its startup, the lash humps the sand behind it before the tail comes
//! out, the swallow bows the column and opens the tooth ring wide, and the
//! dive shudders for forty frames before it drops. Every standing clip starts
//! and ends on [`SandmawPose::standing`], and every buried one on
//! [`SandmawPose::under`], so a cut between them is never a jump.

use super::{BEACH_HEIGHT, STAND_DEPTH, STAND_LEAN, SandmawPose, UNDER_BACK, UNDER_DEPTH, mark};
use crate::beast::{Key, Looseness, Pose, Recipe};
use crate::ease::Ease;
use sim::species::sandmaw::Clip;

pub fn all() -> Vec<Recipe> {
    vec![
        stand(),
        swim(),
        rise(),
        breach(),
        undertow(),
        spit(),
        lash(),
        grab(),
        sound(),
        hold(),
        dive(),
        flinch(),
        beached(),
        dead(),
    ]
}

// ---------------------------------------------------------------------------
// Standing and swimming
// ---------------------------------------------------------------------------

fn stand() -> Recipe {
    // The column sways and breathes: a slow lean forward and back, the head
    // nodding over the hole. Between moves it must still read as alive and
    // as *up* -- the stand is the window.
    let sway = |k: f32| {
        Pose::standing()
            .root(STAND_LEAN + 2.0 * k, 0.0, 0.0)
            .column([5.0, 5.0 - k, 0.0, -10.0 + 2.0 * k, -15.0, -25.0 - 4.0 * k])
            .mouth(4.0 + 3.0 * k)
    };
    Recipe::new(
        Clip::Stand,
        vec![
            Key::at(0.0, sway(-1.0)),
            Key::at(0.5, sway(1.0)),
            Key::at(0.8, sway(0.3)),
        ],
        Looseness::BEAST,
    )
    .noting(
        "Up out of the sand and breathing: the column leans a little forward \
         and back over the hole and the head nods. The stand is the window, \
         so it has to read as up and as alive."
            .to_string(),
    )
}

fn swim() -> Recipe {
    // A travelling wave down the body under the sand, sideways, like a snake.
    // None of it shows -- the wake and the fin do -- but the wave keeps the
    // chain honest for the frames a move from below begins out of it.
    let at = |t: f32| Pose::under().wave(t, 7.0);
    Recipe::new(
        Clip::Swim,
        (0..8)
            .map(|i| {
                let t = i as f32 / 8.0;
                Key::eased(t, at(t), Ease::LINEAR)
            })
            .collect(),
        Looseness::GAIT,
    )
    .noting(
        "Under the sand, a sideways wave down the body. Indexed by ground \
         covered, so the wave moves with the wake rather than across it."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// From below
// ---------------------------------------------------------------------------

/// Coiled under a point, the column straight up beneath it, its head a hand
/// under the sand: where every move from below starts its hit.
fn coiled() -> Pose {
    Pose::rest(sim::species::sandmaw::bones::COUNT)
        .hips(0.0, -8.4, 0.0)
        .root(90.0, 0.0, 0.0)
        .mouth(0.0)
}

fn rise() -> Recipe {
    let ready = Pose::under();
    // Gathering under the marker: the hips come forward under the circle and
    // the front goes down into a coil, all of it under the floor. The marker
    // is the tell; this only has to be in place when it runs out.
    let gathering = Pose::under()
        .hips(UNDER_BACK * 0.4, UNDER_DEPTH - 2.0, 0.0)
        .root(5.0, 0.0, 0.0);
    let coil = coiled();
    // Out: straighter than the stand, the mouth wide, the head thrown up.
    let out = Pose::standing()
        .root(STAND_LEAN + 22.0, 0.0, 0.0)
        .column([0.0, 0.0, 0.0, 5.0, 10.0, 15.0])
        .mouth(45.0);
    // Settling over into the stand, the mouth closing.
    let over = Pose::standing()
        .root(STAND_LEAN + 6.0, 0.0, 0.0)
        .column([5.0, 5.0, 2.0, -8.0, -12.0, -20.0])
        .mouth(12.0);
    let stood = Pose::standing();
    Recipe::new(
        Clip::Rise,
        vec![
            Key::eased(0.0, ready, Ease::SMOOTH),
            Key::eased(mark(Clip::Rise, 0, 0.5), gathering, Ease::SMOOTH),
            Key::eased(mark(Clip::Rise, 0, 0.85), coil, Ease::SMOOTH),
            Key::eased(mark(Clip::Rise, 0, 1.0), coil, Ease::STRIKE),
            Key::eased(mark(Clip::Rise, 1, 0.7), out, Ease::OUT),
            Key::eased(mark(Clip::Rise, 2, 0.35), over, Ease::SMOOTH),
            Key::eased(mark(Clip::Rise, 2, 0.8), stood, Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::SNAP,
    )
    .noting(
        "The tell is the marker on the sand and the wake converging on it; the \
         body gathers into a coil under the circle, all of it below the floor. \
         Then six metres of worm out of the sand in six frames, the mouth \
         wide, and a long settle into the stand -- the window."
            .to_string(),
    )
}

fn breach() -> Recipe {
    let ready = Pose::under();
    // A dip before the arc: it goes deeper to come up faster.
    let dip = Pose::under().hips(UNDER_BACK, UNDER_DEPTH - 1.0, 0.0);
    // Coming out: the head and the front up through the sand.
    let surfacing = Pose::under()
        .hips(UNDER_BACK, -0.9, 0.0)
        .root(10.0, 0.0, 0.0)
        .column([10.0, 10.0, 8.0, 6.0, 4.0, 0.0])
        .tail([10.0, 5.0, 0.0, 0.0, 0.0]);
    // The top of the arc: the whole body in the air in a bow, back up.
    let arched = Pose::under()
        .hips(UNDER_BACK, 1.6, 0.0)
        .root(4.0, 0.0, 0.0)
        .column([-6.0, -8.0, -10.0, -10.0, -10.0, -12.0])
        .tail([-6.0, -8.0, -10.0, -10.0, -10.0])
        .mouth(20.0);
    // Going back in head first.
    let diving = Pose::under()
        .hips(UNDER_BACK, -0.6, 0.0)
        .root(-12.0, 0.0, 0.0)
        .column([-10.0, -10.0, -8.0, -6.0, -4.0, 0.0])
        .tail([-6.0, -4.0, -2.0, 0.0, 0.0]);
    Recipe::new(
        Clip::Breach,
        vec![
            Key::eased(0.0, ready, Ease::SMOOTH),
            Key::eased(mark(Clip::Breach, 0, 0.6), dip, Ease::SMOOTH),
            Key::eased(mark(Clip::Breach, 0, 1.0), dip, Ease::OUT),
            Key::eased(mark(Clip::Breach, 1, 0.15), surfacing, Ease::OUT),
            Key::eased(mark(Clip::Breach, 1, 0.5), arched, Ease::SMOOTH),
            Key::eased(mark(Clip::Breach, 1, 0.9), diving, Ease::IN),
            Key::eased(mark(Clip::Breach, 2, 0.4), Pose::under(), Ease::SMOOTH),
            Key::at(1.0, Pose::under()),
        ],
        Looseness::HEAVY,
    )
    .noting(
        "Under through the tell (the lane is drawn on the sand), a dip, then a \
         dolphin's arc over the whole lane: the body is in the air for the \
         active frames, which is the one time a breach can be hit. It goes back \
         in head first and is under and deaf for the recovery."
            .to_string(),
    )
}

fn undertow() -> Recipe {
    // Circling tight under the sinkhole, all of it under the floor: the sand
    // sinking is the tell.
    let at = |t: f32| {
        Pose::under()
            .hips(UNDER_BACK * 0.6, UNDER_DEPTH - 0.5, 0.0)
            .wave(t, 12.0)
    };
    Recipe::new(
        Clip::Undertow,
        vec![
            Key::eased(0.0, Pose::under(), Ease::SMOOTH),
            Key::eased(mark(Clip::Undertow, 0, 1.0), at(0.0), Ease::SMOOTH),
            Key::eased(mark(Clip::Undertow, 1, 0.33), at(0.33), Ease::SMOOTH),
            Key::eased(mark(Clip::Undertow, 1, 0.66), at(0.66), Ease::SMOOTH),
            Key::eased(mark(Clip::Undertow, 1, 1.0), at(1.0), Ease::SMOOTH),
            Key::at(1.0, at(1.0)),
        ],
        Looseness::BEAST,
    )
    .noting(
        "Coiling under the sinkhole it made; nothing shows but the ring of \
         sand going down. The rise at its middle is the rise-bite's clip."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// Standing
// ---------------------------------------------------------------------------

fn spit() -> Recipe {
    let ready = Pose::standing();
    // Drawn back and up, the throat full: the column straightens and the head
    // tips back, which reads from across the Pan.
    let loaded = Pose::standing()
        .root(STAND_LEAN + 18.0, 0.0, 0.0)
        .column([0.0, 0.0, -5.0, 0.0, 10.0, 25.0])
        .mouth(2.0);
    // The spray: thrown forward and down, mouth wide.
    let thrown = Pose::standing()
        .root(STAND_LEAN - 12.0, 0.0, 0.0)
        .column([8.0, 8.0, 4.0, -12.0, -20.0, -30.0])
        .mouth(40.0);
    Recipe::new(
        Clip::Spit,
        vec![
            Key::eased(0.0, ready, Ease::ANTICIPATE),
            Key::eased(mark(Clip::Spit, 0, 0.7), loaded, Ease::SNAP),
            Key::eased(mark(Clip::Spit, 1, 0.0), thrown, Ease::STRIKE),
            Key::eased(mark(Clip::Spit, 2, 0.3), thrown, Ease::OUT),
            Key::eased(mark(Clip::Spit, 2, 0.8), Pose::standing(), Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::HEAVY,
    )
    .noting(
        "The head draws back and the column straightens for most of the \
         startup -- the throat swelling -- then the spray goes forward and \
         down onto the sand with the whole column behind it."
            .to_string(),
    )
}

fn lash() -> Recipe {
    let ready = Pose::standing();
    // The sand humps behind it: the body behind the hole rises toward the
    // surface and the tip breaks it, swung round to its left.
    let humped = Pose::standing()
        .root(STAND_LEAN - 6.0, 0.0, 0.0)
        .tail([-60.0, -10.0, -10.0, 0.0, 0.0])
        .tail_swing(-70.0);
    // Out and round: the tail low and level behind it, swept across.
    let swept = Pose::standing()
        .root(STAND_LEAN - 10.0, 0.0, 0.0)
        .tail([-75.0, -12.0, -4.0, 0.0, 0.0])
        .tail_swing(70.0);
    Recipe::new(
        Clip::Lash,
        vec![
            Key::eased(0.0, ready, Ease::ANTICIPATE),
            Key::eased(mark(Clip::Lash, 0, 0.8), humped, Ease::SNAP),
            Key::eased(mark(Clip::Lash, 1, 1.0), swept, Ease::STRIKE),
            Key::eased(mark(Clip::Lash, 2, 0.35), swept, Ease::OUT),
            Key::eased(mark(Clip::Lash, 2, 0.85), Pose::standing(), Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::SNAP,
    )
    .noting(
        "The sand humps behind the column and the tail tip comes out on one \
         side through the tell; then the tail sweeps a half-ring round behind \
         it at chest height, which a crouch goes under."
            .to_string(),
    )
}

fn grab() -> Recipe {
    let ready = Pose::standing();
    // Bowing: the column bends over, the head comes down in front, and the
    // tooth ring opens to its full four metres -- the loudest shape in the
    // fight, held through the startup.
    let bowed = Pose::standing()
        .root(STAND_LEAN - 10.0, 0.0, 0.0)
        .column([-5.0, -15.0, -20.0, -20.0, -20.0, -15.0])
        .mouth(80.0);
    // The lunge: down onto the sand in front.
    let taken = Pose::standing()
        .root(STAND_LEAN - 20.0, 0.0, 0.0)
        .column([-8.0, -18.0, -24.0, -24.0, -22.0, -20.0])
        .mouth(20.0);
    Recipe::new(
        Clip::Grab,
        vec![
            Key::eased(0.0, ready, Ease::ANTICIPATE),
            Key::eased(mark(Clip::Grab, 0, 0.6), bowed, Ease::SMOOTH),
            Key::eased(mark(Clip::Grab, 1, 0.0), bowed, Ease::STRIKE),
            Key::eased(mark(Clip::Grab, 1, 1.0), taken, Ease::OUT),
            Key::eased(mark(Clip::Grab, 2, 0.6), Pose::standing(), Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::HEAVY,
    )
    .noting(
        "The column bows, the head comes down in front and the tooth ring \
         opens to four metres for the whole startup: the window to hit into \
         the mouth. Then down onto the sand, and back up empty -- or not."
            .to_string(),
    )
}

fn hold() -> Recipe {
    // The head down in the sand at the hole with somebody in it, the column
    // bent over; a gulp every thirty frames is a contraction down the neck.
    let down = |k: f32| {
        Pose::standing()
            .root(STAND_LEAN - 22.0 + 4.0 * k, 0.0, 0.0)
            .column([
                -8.0,
                -18.0 - 6.0 * k,
                -24.0 + 4.0 * k,
                -24.0,
                -22.0,
                -20.0 + 6.0 * k,
            ])
            .mouth(6.0)
    };
    let mut keys = vec![Key::eased(0.0, down(0.0), Ease::SMOOTH)];
    // Four gulps across the hold: the swallow's own clock is the hold's
    // (`fight::hold`), and the clip is laid on the move's active frames.
    for g in 1..=4 {
        let at = g as f32 * 0.25;
        keys.push(Key::eased(
            mark(Clip::Hold, 1, at - 0.08),
            down(0.0),
            Ease::IN,
        ));
        keys.push(Key::eased(
            mark(Clip::Hold, 1, at - 0.02),
            down(1.0),
            Ease::OUT,
        ));
    }
    keys.push(Key::eased(
        mark(Clip::Hold, 2, 0.6),
        Pose::standing(),
        Ease::SMOOTH,
    ));
    keys.push(Key::at(1.0, Pose::standing()));
    Recipe::new(Clip::Hold, keys, Looseness::HEAVY).noting(
        "Bent over the hole with the head in the sand and a fighter in its \
         throat. Each gulp is a contraction down the neck a few frames before \
         its window -- the one timed press in any creature fight, so it has to \
         be seen coming."
            .to_string(),
    )
}

fn sound() -> Recipe {
    let ready = Pose::standing();
    // The shudder: the vents shut and the column trembles, forty frames.
    let shake = |k: f32| {
        Pose::standing()
            .root(STAND_LEAN + 3.0 * k, 3.0 * k, 0.0)
            .column([5.0, 5.0 - 4.0 * k, 0.0, -10.0 + 4.0 * k, -15.0, -25.0])
            .mouth(0.0)
    };
    // The plunge: straight down into the hole.
    let gone = coiled();
    Recipe::new(
        Clip::Sound,
        vec![
            Key::eased(0.0, ready, Ease::SMOOTH),
            Key::eased(mark(Clip::Sound, 0, 0.2), shake(1.0), Ease::SMOOTH),
            Key::eased(mark(Clip::Sound, 0, 0.4), shake(-1.0), Ease::SMOOTH),
            Key::eased(mark(Clip::Sound, 0, 0.6), shake(1.0), Ease::SMOOTH),
            Key::eased(mark(Clip::Sound, 0, 0.8), shake(-1.0), Ease::SMOOTH),
            Key::eased(mark(Clip::Sound, 0, 1.0), shake(0.5), Ease::IN),
            Key::eased(mark(Clip::Sound, 1, 0.8), gone, Ease::SMOOTH),
            Key::eased(mark(Clip::Sound, 2, 0.7), Pose::under(), Ease::SMOOTH),
            Key::at(1.0, Pose::under()),
        ],
        Looseness::HEAVY,
    )
    .noting(
        "Forty frames of shudder with the spiracles shut -- the ride's one \
         tell -- then straight down into its own hole, and under."
            .to_string(),
    )
}

fn dive() -> Recipe {
    let ready = Pose::beached();
    let shake = |k: f32| Pose::beached().writhe(k * 0.25, 8.0).wave(k * 0.25, 4.0);
    // Head first into the sand off its side.
    let going = Pose::beached()
        .hips(0.0, BEACH_HEIGHT - 1.0, 0.0)
        .root(-25.0, 0.0, 0.0)
        .column([-15.0, -10.0, -5.0, 0.0, 0.0, 0.0]);
    Recipe::new(
        Clip::Dive,
        vec![
            Key::eased(0.0, ready, Ease::SMOOTH),
            Key::eased(mark(Clip::Dive, 0, 0.25), shake(1.0), Ease::SMOOTH),
            Key::eased(mark(Clip::Dive, 0, 0.5), shake(-1.0), Ease::SMOOTH),
            Key::eased(mark(Clip::Dive, 0, 0.75), shake(1.0), Ease::SMOOTH),
            Key::eased(mark(Clip::Dive, 0, 1.0), shake(0.0), Ease::IN),
            Key::eased(mark(Clip::Dive, 1, 0.5), going, Ease::IN),
            Key::eased(mark(Clip::Dive, 2, 0.6), Pose::under(), Ease::SMOOTH),
            Key::at(1.0, Pose::under()),
        ],
        Looseness::HEAVY,
    )
    .noting(
        "Off the beach: the same forty frames of shudder with the spiracles \
         shut, from its side, then it noses into the sand and is gone. Riders \
         are thrown as it goes."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// Trouble
// ---------------------------------------------------------------------------

fn flinch() -> Recipe {
    let ready = Pose::standing();
    let jolt = Pose::standing()
        .root(STAND_LEAN + 12.0, 0.0, 0.0)
        .column([0.0, 0.0, -5.0, 0.0, 15.0, 25.0])
        .mouth(30.0);
    Recipe::new(
        Clip::Flinch,
        vec![
            Key::eased(0.0, ready, Ease::STRIKE),
            Key::eased(0.18, jolt, Ease::OUT),
            Key::eased(0.7, Pose::standing(), Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::SNAP,
    )
    .noting(
        "Thrown back, the mouth open -- a gag, or a hit it felt -- and back \
         over the hole."
            .to_string(),
    )
}

fn beached() -> Recipe {
    // Over it goes, forward, onto the sand: then writhing, gentle at the
    // middle of the back and violent at the head and the tail, which is what
    // the ride judges.
    let falling = Pose::standing()
        .hips(0.0, STAND_DEPTH + 0.8, 0.0)
        .root(25.0, 0.0, 10.0)
        .column([-5.0, -5.0, -5.0, -5.0, -5.0, -5.0])
        .tail([-30.0, -10.0, 0.0, 0.0, 0.0]);
    let lying = |k: f32| Pose::beached().wave(k, 9.0).writhe(k, 6.0).mouth(20.0);
    let mut keys = vec![
        Key::eased(0.0, Pose::standing(), Ease::IN),
        Key::eased(0.06, falling, Ease::IN),
        Key::eased(0.12, lying(0.0), Ease::OUT),
    ];
    for i in 1..=7 {
        let at = 0.12 + 0.11 * i as f32;
        keys.push(Key::eased(
            at.min(0.95),
            lying(i as f32 * 0.5),
            Ease::SMOOTH,
        ));
    }
    keys.push(Key::at(1.0, Pose::beached().mouth(10.0)));
    Recipe::new(Clip::Beached, keys, Looseness::COLLAPSE).noting(
        "It falls over forward out of its hole and lies on the sand writhing \
         for three seconds, its back at 2.2 m. The writhe is a wave down the \
         body, so the head and the tail move most and the middle least."
            .to_string(),
    )
}

fn dead() -> Recipe {
    let still = Pose::beached()
        .hips(0.0, BEACH_HEIGHT - 0.1, 0.0)
        .root(0.0, 0.0, 25.0)
        .mouth(30.0);
    Recipe::new(Clip::Dead, vec![Key::at(0.0, still)], Looseness::COLLAPSE)
        .noting("Still, on its side on the sand.".to_string())
}

/// The depth the root goes to, and the lean, for tools that ask.
pub const fn stand_numbers() -> (f32, f32) {
    (STAND_DEPTH, STAND_LEAN)
}
