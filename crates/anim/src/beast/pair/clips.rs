//! Every animation a cat has, as poses and timing.
//!
//! Baked into `crates/sim/src/species/pair/baked.rs` by
//! `cargo run -p anim --bin bake_beast -- --species pair`.
//!
//! The house style is the Ridgeback's (`ridgeback/clips.rs`): **the windup is
//! the move**, the hit is short and enormous, and the recovery is long and
//! readable. Three things are the cat's own:
//!
//! 1. **Everything goes down before it goes anywhere.** A cat's tell is the
//!    body dropping -- haunches for the pounce, belly for the ambush, the
//!    shoulder for the swat -- so the silhouette shrinks where the Ridgeback's
//!    grew. That is what reads at fight distance on a 1.8 m animal.
//! 2. **The tail is the signal.** The pounce and the feint share their coil
//!    frame for frame up to frame ten; at ten the pounce's whole tail snaps up
//!    and over and the feint's stays down (`the-pair.md` §2). It is drawn
//!    big, not a twitch at the tip, because it is the one thing a new player
//!    has to learn to see.
//! 3. **The leaps carry the body, and the clip carries the height.** The
//!    simulation flies a leaping cat across the floor (`sim::species::pair::
//!    fight::fly`); its clip lifts the hips through the arc, so the belly is
//!    two metres up at the top of a pounce and a dodge toward it passes under.

use super::{CatPose, at_frame, mark};
use crate::beast::{Key, Looseness, Pose, Recipe};
use crate::ease::Ease;
use sim::species::pair::Clip;

pub fn all() -> Vec<Recipe> {
    vec![
        idle(),
        walk(),
        bound(),
        pounce(),
        rake(),
        cock(),
        rake2(),
        swat(),
        feint(),
        ambush(),
        trip(),
        perch(),
        dive(),
        drop(),
        twin(),
        interpose(),
        howl(),
        flinch(),
        stumble(),
        topple(),
        dead(),
    ]
}

/// A cat, quick: the bite and the swat. Tighter than the Ridgeback's SNAP,
/// because the animal is a tenth of the mass.
const QUICK: Looseness = Looseness {
    body: crate::beast::Feel::new(1.2, 0.95),
    neck: crate::beast::Feel::new(1.6, 0.75),
    tail: crate::beast::Feel::new(2.8, 0.50),
    legs: crate::beast::Feel::new(1.2, 0.95),
};

/// A cat, coiled and loosed: the leaps. The tail is the loosest thing on it,
/// which is what makes the flick read as a whip rather than a lever.
const SPRING: Looseness = Looseness {
    body: crate::beast::Feel::new(1.8, 0.90),
    neck: crate::beast::Feel::new(2.4, 0.70),
    tail: crate::beast::Feel::new(2.2, 0.55),
    legs: crate::beast::Feel::new(1.4, 0.92),
};

// ---------------------------------------------------------------------------
// Standing and moving
// ---------------------------------------------------------------------------

fn idle() -> Recipe {
    let breathe = |k: f32| {
        Pose::standing()
            .hips(0.0, 0.015 * k, 0.0)
            .spine(0.8 * k, 0.0, 0.0)
            .chest(1.0 * k, 0.0, 0.0)
            .neck(2.0 * k, 3.0 * k)
            .tail(4.0 * k, 12.0 * k)
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
        "Breathing, and the tail moving on its own the whole time. A cat that \
         is still is a cat about to do something, so the idle is never still: \
         the tail's slow sweep is what a cat at rest looks like, and its \
         stopping is the first thing the coil does."
            .to_string(),
    )
}

/// One leg at a point in its own cycle: planted for most of it and swinging
/// for the rest, solved onto the floor (the Ridgeback's `stride`).
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

fn walk() -> Recipe {
    // The four-beat lateral walk, hind then fore on one side: the same order
    // the Ridgeback walks in, and every cat. The shoulders roll over each
    // planted foreleg, which is the cat's walk and nobody else's.
    const OFFSET: [f32; 4] = [0.5, 0.0, 0.75, 0.25];
    let at = |t: f32| {
        let mut p = Pose::standing();
        for (which, offset) in OFFSET.into_iter().enumerate() {
            p = stride(p, which, t + offset, 0.42, 0.22);
        }
        let wheel = t * std::f32::consts::TAU;
        let bob = -0.025 * (2.0 * wheel.sin().abs());
        let sway = 4.0 * (wheel + std::f32::consts::FRAC_PI_2).sin();
        p.hips(0.0, bob, 0.0)
            .root(0.8 * wheel.cos(), 0.0, sway * 0.5)
            .spine(0.0, sway * 0.6, 0.0)
            .chest(0.0, -sway * 0.5, sway * 0.6)
            .neck(-4.0, -sway * 0.6)
            .tail(-6.0, sway * 3.0)
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
    .noting(
        "The walk: four beats, three feet down, and the shoulder blades rolling \
         over each foreleg as it takes the weight. Head carried low and level. \
         Linear between keys because a stride is constant speed."
            .to_string(),
    )
}

fn bound() -> Recipe {
    // The rotary gallop of a big cat: the hind pair drive, the spine folds
    // under and throws, the forelegs reach out and catch. More spine than the
    // Ridgeback's gallop -- the flex *is* a cat at speed.
    const OFFSET: [f32; 4] = [0.40, 0.48, 0.0, 0.08];
    let at = |t: f32| {
        let mut p = Pose::standing();
        for (which, offset) in OFFSET.into_iter().enumerate() {
            p = stride(p, which, t + offset, 0.75, 0.45);
        }
        let wheel = t * std::f32::consts::TAU;
        let bob = 0.14 * (wheel - 0.9).sin() - 0.04;
        let flex = 16.0 * (wheel - 0.4).sin();
        p.hips(0.0, bob, 0.0)
            .root(-flex * 0.5, 0.0, 0.0)
            .spine(flex, 0.0, 0.0)
            .chest(flex * 0.8, 0.0, 0.0)
            .neck(-12.0 - flex * 0.7, 0.0)
            .tail(4.0 - flex * 0.8, 0.0)
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
        "The bound. Hind pair driving together, the spine folding under at the \
         gather and opening right out at the reach, head low and steady over \
         it. Twelve metres a second is a cat going flat out, and the flex is \
         what makes it read as running rather than sliding."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// The coil, which the pounce, the feint and the twin share
// ---------------------------------------------------------------------------

/// Frame ten of the coil: the haunches down, the forequarters level, the head
/// low and still -- and the tail still low. Pounce and feint both pass
/// through exactly this.
fn coil_early() -> Pose {
    Pose::standing()
        .hips(-0.10, -0.22, 0.0)
        .root(8.0, 0.0, 0.0)
        .spine(-2.0, 0.0, 0.0)
        .chest(-6.0, 0.0, 0.0)
        .neck(-12.0, 0.0)
        .head(8.0, 0.0, 0.0)
        .tail(-14.0, 0.0)
        .plant_fore(0.14, 0.0)
        .plant_hind(-0.30, 0.0)
}

/// The full coil: everything a cat has is under it. `tail` is how high the
/// tail is carried -- up and over for a pounce, flat for a feint.
fn coil_full(tail: f32) -> Pose {
    Pose::standing()
        .hips(-0.16, -0.34, 0.0)
        .root(12.0, 0.0, 0.0)
        .spine(-4.0, 0.0, 0.0)
        .chest(-10.0, 0.0, 0.0)
        .neck(-16.0, 0.0)
        .head(12.0, 0.0, 0.0)
        .tail(tail, 0.0)
        .plant_fore(0.20, 0.0)
        .plant_hind(-0.36, 0.0)
}

/// **The flick**: the whole tail snapped up and over the back. Not a twitch at
/// the tip -- the base goes up with it, so the shape of the animal changes.
const FLICK: f32 = 105.0;
/// The feint's tail, held flat and still through the whole coil.
const FLAT: f32 = -18.0;

/// The top of a leap: stretched out, forelegs reaching, hind legs trailing,
/// the belly a metre higher than standing.
fn airborne(lift: f32) -> Pose {
    Pose::rest(sim::species::pair::bones::COUNT)
        .hips(0.0, lift, 0.0)
        .root(-4.0, 0.0, 0.0)
        .spine(4.0, 0.0, 0.0)
        .chest(6.0, 0.0, 0.0)
        .neck(-6.0, 0.0)
        .head(4.0, 0.0, 0.0)
        .tail(10.0, 0.0)
        .forelegs(58.0, -24.0)
        .hindlegs(-52.0, 18.0)
}

/// Landed: forepaws first and far forward, the claws out, the hind end coming
/// down behind -- the skid's pose.
fn landed() -> Pose {
    Pose::standing()
        .hips(0.10, -0.24, 0.0)
        .root(-10.0, 0.0, 0.0)
        .spine(-4.0, 0.0, 0.0)
        .chest(-6.0, 0.0, 0.0)
        .neck(10.0, 0.0)
        .head(-6.0, 0.0, 0.0)
        .tail(18.0, 0.0)
        .plant_fore(0.42, 0.0)
        .plant_hind(-0.30, 0.0)
}

fn pounce() -> Recipe {
    let ready = Pose::standing();
    let flicked = coil_early().tail(FLAT.abs() + FLICK, 0.0);
    Recipe::new(
        Clip::Pounce,
        vec![
            Key::eased(0.0, ready, Ease::OUT),
            // Frame ten: identical to the feint's.
            Key::eased(at_frame(Clip::Pounce, 10.0), coil_early(), Ease::SNAP),
            // The flick, three frames later: the tail goes up and over.
            Key::eased(at_frame(Clip::Pounce, 13.0), flicked, Ease::SMOOTH),
            Key::eased(
                at_frame(Clip::Pounce, 22.0),
                coil_full(FLICK - 10.0),
                Ease::HOLD,
            ),
            // Leaves the ground at twenty-four (`PounceLeave`).
            Key::eased(
                at_frame(Clip::Pounce, 24.0),
                coil_full(FLICK - 20.0),
                Ease::STRIKE,
            ),
            // The top of the arc on the first live frame.
            Key::eased(mark(Clip::Pounce, 1, 0.0), airborne(1.05), Ease::IN),
            // Forepaws down at the sixth (`PounceLand`), and the skid.
            Key::eased(mark(Clip::Pounce, 1, 0.42), landed(), Ease::OUT),
            Key::eased(mark(Clip::Pounce, 1, 1.0), landed(), Ease::HOLD),
            Key::eased(mark(Clip::Pounce, 2, 0.45), landed(), Ease::SMOOTH),
            Key::at(1.0, ready),
        ],
        SPRING,
    )
    .noting(
        "Haunches down from the first frame; at frame ten the tail is still low \
         and the pose is the feint's to the degree; at thirteen the whole tail \
         has snapped up and over the back. Leaves the ground at twenty-four, is \
         at the top of the arc -- belly two metres up -- on the first live \
         frame, forepaws down six frames later and skidding. The recovery holds \
         the skid's crouch for half its length: it is the window, and it faces \
         away from whoever dodged under it."
            .to_string(),
    )
}

fn feint() -> Recipe {
    let ready = Pose::standing();
    Recipe::new(
        Clip::Feint,
        vec![
            Key::eased(0.0, ready, Ease::OUT),
            Key::eased(at_frame(Clip::Feint, 10.0), coil_early(), Ease::SNAP),
            // No flick: the tail stays flat on the ground behind it.
            Key::eased(mark(Clip::Feint, 0, 0.9), coil_full(FLAT), Ease::HOLD),
            Key::eased(mark(Clip::Feint, 2, 0.6), ready, Ease::SMOOTH),
            Key::at(1.0, ready),
        ],
        SPRING,
    )
    .noting(
        "The pounce's coil, key for key, up to frame ten -- and then the tail \
         stays flat on the floor where the pounce's goes up and over. That is \
         the only difference, by design: a feint is read by the tail or not at \
         all. It stands up over ten frames and is a Holder again."
            .to_string(),
    )
}

fn twin() -> Recipe {
    let ready = Pose::standing();
    // The roar: head up, chest up, the whole front of the animal lifted --
    // the loudest silhouette it has.
    let roar = Pose::standing()
        .hips(-0.06, 0.02, 0.0)
        .root(4.0, 0.0, 0.0)
        .spine(6.0, 0.0, 0.0)
        .chest(10.0, 0.0, 0.0)
        .neck(34.0, 0.0)
        .head(18.0, 0.0, 0.0)
        .tail(30.0, 0.0)
        .plant_fore(0.10, 0.0)
        .plant_hind(-0.20, 0.0);
    // Then flat and still: belly almost on the floor, head on the line.
    let flat = Pose::standing()
        .hips(-0.20, -0.45, 0.0)
        .root(10.0, 0.0, 0.0)
        .spine(-4.0, 0.0, 0.0)
        .chest(-12.0, 0.0, 0.0)
        .neck(-14.0, 0.0)
        .head(14.0, 0.0, 0.0)
        .tail(-10.0, 0.0)
        .plant_fore(0.26, 0.0)
        .plant_hind(-0.40, 0.0);
    Recipe::new(
        Clip::Twin,
        vec![
            Key::eased(0.0, ready, Ease::OUT),
            Key::eased(at_frame(Clip::Twin, 8.0), roar, Ease::HOLD),
            Key::eased(at_frame(Clip::Twin, 18.0), roar, Ease::SNAP),
            Key::eased(at_frame(Clip::Twin, 30.0), flat, Ease::HOLD),
            // Leaves the ground at forty (`TwinLeave`).
            Key::eased(at_frame(Clip::Twin, 40.0), flat, Ease::STRIKE),
            Key::eased(mark(Clip::Twin, 1, 0.0), airborne(1.0), Ease::IN),
            Key::eased(mark(Clip::Twin, 1, 0.5), landed(), Ease::OUT),
            Key::eased(mark(Clip::Twin, 2, 0.5), landed(), Ease::SMOOTH),
            Key::at(1.0, ready),
        ],
        SPRING,
    )
    .noting(
        "The roar and then the stillness: two cats on opposite sides of you, \
         flat and not moving, which nothing else in the fight looks like. They \
         leave the ground on frame forty, together, and that is the moment to \
         dodge: before it their windups follow you, after it they cannot turn."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// The paws
// ---------------------------------------------------------------------------

/// The right paw cocked at the height of the head, weight on the left: the
/// rake's held pose, which reads over the body from the front.
fn cocked() -> Pose {
    Pose::standing()
        .hips(-0.06, -0.08, 0.0)
        .root(6.0, 0.0, 4.0)
        .chest(6.0, 0.0, 6.0)
        .neck(6.0, -6.0)
        .head(-6.0, 0.0, 0.0)
        .tail(6.0, 20.0)
        .plant(0, 0.10, 0.0)
        .plant_hind(-0.24, 0.0)
        .leg(1, 88.0, -70.0)
}

/// The swipe come down: the right paw low and forward, across the line.
fn swiped() -> Pose {
    Pose::standing()
        .hips(0.06, -0.12, 0.0)
        .root(-4.0, 0.0, -3.0)
        .chest(-8.0, 0.0, -6.0)
        .neck(-10.0, 8.0)
        .head(6.0, 0.0, 0.0)
        .tail(4.0, -16.0)
        .plant(0, 0.06, 0.0)
        .plant_hind(-0.20, 0.0)
        .leg(1, 34.0, -8.0)
}

fn rake() -> Recipe {
    let ready = Pose::standing();
    Recipe::new(
        Clip::Rake,
        vec![
            Key::eased(0.0, ready, Ease::ANTICIPATE),
            Key::eased(mark(Clip::Rake, 0, 0.75), cocked(), Ease::SNAP),
            Key::eased(mark(Clip::Rake, 1, 0.2), swiped(), Ease::STRIKE),
            // Straight back up: the paw does not come down to the floor
            // between the two, which is the read.
            Key::at(1.0, cocked()),
        ],
        QUICK,
    )
    .noting(
        "Paw up to head height over three quarters of the startup, down across \
         the line on the first active frame -- and straight back up to the \
         cocked pose by the end of the active window, with no recovery at all. \
         The paw never goes back to the floor between the two hits: a paw \
         still up means it is not finished."
            .to_string(),
    )
}

fn cock() -> Recipe {
    // Held. The length of this is drawn as the second hit commits, so the
    // pose must not change across it: anything that moved would be a clock.
    let tremble = cocked().neck(2.0, -2.0);
    Recipe::new(
        Clip::Cock,
        vec![
            Key::at(0.0, cocked()),
            Key::at(0.5, tremble),
            Key::at(1.0, cocked()),
        ],
        QUICK,
    )
    .noting(
        "The paw held up for a beat drawn from the seed: six to thirty frames, \
         read as a fraction of thirty, so a short hold plays only the start of \
         this. Nothing in it moves enough to count down from."
            .to_string(),
    )
}

fn rake2() -> Recipe {
    let ready = Pose::standing();
    let driven = swiped().hips(0.10, -0.18, 0.0).chest(-12.0, 0.0, -8.0);
    Recipe::new(
        Clip::Rake2,
        vec![
            Key::eased(0.0, cocked(), Ease::IN),
            Key::eased(mark(Clip::Rake2, 0, 0.5), cocked().chest(10.0, 0.0, 8.0), Ease::SNAP),
            Key::eased(mark(Clip::Rake2, 1, 0.0), driven, Ease::STRIKE),
            Key::eased(mark(Clip::Rake2, 2, 0.35), driven, Ease::OUT),
            Key::at(1.0, ready),
        ],
        QUICK,
    )
    .noting(
        "The cocked paw starting to drop is the second tell: fifteen frames, \
         with the shoulder rising first so the drop is legible. Down hard, and \
         a long recovery with the paw on the floor -- this is the Holder's \
         window, if you know where the other one is."
            .to_string(),
    )
}

fn swat() -> Recipe {
    let ready = Pose::standing();
    // The shoulder drops and the paw draws back across the chest.
    let drawn = Pose::standing()
        .hips(-0.04, -0.10, 0.0)
        .root(2.0, 6.0, 6.0)
        .chest(0.0, 8.0, 10.0)
        .neck(-6.0, 10.0)
        .tail(4.0, -10.0)
        .plant(0, 0.06, 0.0)
        .plant_hind(-0.22, 0.0)
        .leg(1, 40.0, -60.0)
        .leg_spread(1, -18.0);
    let cuffed = Pose::standing()
        .hips(0.04, -0.08, 0.0)
        .root(-2.0, -8.0, -6.0)
        .chest(-4.0, -10.0, -10.0)
        .neck(-4.0, -12.0)
        .tail(4.0, 14.0)
        .plant(0, 0.06, 0.0)
        .plant_hind(-0.18, 0.0)
        .leg(1, 62.0, -4.0)
        .leg_spread(1, 26.0);
    Recipe::new(
        Clip::Swat,
        vec![
            Key::eased(0.0, ready, Ease::IN),
            Key::eased(mark(Clip::Swat, 0, 0.7), drawn, Ease::SNAP),
            Key::eased(mark(Clip::Swat, 1, 0.0), cuffed, Ease::STRIKE),
            Key::eased(mark(Clip::Swat, 2, 0.3), cuffed, Ease::OUT),
            Key::at(1.0, ready),
        ],
        QUICK,
    )
    .noting(
        "Twelve frames, below reaction on purpose: answered by not standing at \
         its nose. The tell is still drawn -- the shoulder drops and the paw \
         crosses the chest -- because a cuff out of nothing reads as a bug, not \
         as a rule about where to stand."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// The Striker's moves
// ---------------------------------------------------------------------------

fn ambush() -> Recipe {
    let ready = Pose::standing();
    // Belly to the floor, the whole animal a hand off the ground, head flat
    // on the line it is about to run.
    let flat = Pose::standing()
        .hips(-0.10, -0.55, 0.0)
        .root(4.0, 0.0, 0.0)
        .spine(-2.0, 0.0, 0.0)
        .chest(-6.0, 0.0, 0.0)
        .neck(-18.0, 0.0)
        .head(16.0, 0.0, 0.0)
        .tail(-14.0, 0.0)
        .plant_fore(0.30, 0.0)
        .plant_hind(-0.44, 0.0);
    // The lunge: stretched flat out along the ground, forelegs reaching.
    let lunge = Pose::standing()
        .hips(0.20, -0.30, 0.0)
        .root(-2.0, 0.0, 0.0)
        .spine(2.0, 0.0, 0.0)
        .chest(2.0, 0.0, 0.0)
        .neck(-10.0, 0.0)
        .head(8.0, 0.0, 0.0)
        .tail(4.0, 0.0)
        .forelegs(62.0, -10.0)
        .plant_hind(-0.70, 0.0);
    let skid = Pose::standing()
        .hips(0.0, -0.28, 0.0)
        .root(-6.0, 0.0, 0.0)
        .neck(8.0, 0.0)
        .tail(14.0, 0.0)
        .plant_fore(0.36, 0.0)
        .plant_hind(-0.36, 0.0);
    Recipe::new(
        Clip::Ambush,
        vec![
            Key::eased(0.0, ready, Ease::IN),
            Key::eased(mark(Clip::Ambush, 0, 0.55), flat, Ease::HOLD),
            Key::eased(mark(Clip::Ambush, 0, 0.95), flat, Ease::STRIKE),
            Key::eased(mark(Clip::Ambush, 1, 0.2), lunge, Ease::LINEAR),
            Key::eased(mark(Clip::Ambush, 1, 1.0), lunge, Ease::OUT),
            Key::eased(mark(Clip::Ambush, 2, 0.35), skid, Ease::SMOOTH),
            Key::at(1.0, ready),
        ],
        SPRING,
    )
    .noting(
        "Belly to the floor and still -- from behind you, which is where it is, \
         the lane drawn under your feet is the tell you see, and this is the one \
         your partner sees. Then flat out along the ground at twenty-five metres \
         a second, and a long slide out of it."
            .to_string(),
    )
}

fn trip() -> Recipe {
    let ready = Pose::standing();
    // The hips swing away first and the tail drops flat on the far side --
    // authored going to the cat's right; mirrored to the target's side.
    let wound = Pose::standing()
        .hips(0.0, -0.10, -0.10)
        .root(2.0, -16.0, -8.0)
        .spine(0.0, -8.0, 0.0)
        .chest(0.0, 6.0, 0.0)
        .neck(4.0, 22.0)
        .tail(-30.0, -40.0)
        .tail_swing_base(-30.0)
        .plant_fore(0.06, 0.0)
        .plant_hind(-0.22, 0.0);
    let swept = Pose::standing()
        .hips(0.0, -0.12, 0.10)
        .root(-2.0, 18.0, 8.0)
        .spine(0.0, 10.0, 0.0)
        .chest(0.0, -6.0, 0.0)
        .neck(2.0, 30.0)
        .tail(-34.0, 50.0)
        .tail_swing_base(60.0)
        .plant_fore(0.10, 0.0)
        .plant_hind(-0.18, 0.0);
    Recipe::new(
        Clip::Trip,
        vec![
            Key::eased(0.0, ready, Ease::ANTICIPATE),
            Key::eased(mark(Clip::Trip, 0, 0.7), wound, Ease::SNAP),
            Key::eased(mark(Clip::Trip, 1, 0.5), swept, Ease::OUT),
            Key::eased(mark(Clip::Trip, 2, 0.3), swept, Ease::SMOOTH),
            Key::at(1.0, ready),
        ],
        Looseness::SNAP,
    )
    .noting(
        "The hips swing away and the tail drops flat on the floor first -- the \
         tell, eighteen frames, readable from beside it, which is where the \
         person it is for is standing. Then a low sweep, the tail carried round \
         from its base rather than whipped, so the whole of it is at ankle \
         height and the answer is a jump."
            .to_string(),
    )
}

fn perch() -> Recipe {
    let ready = Pose::standing();
    let crouch = Pose::standing()
        .hips(-0.10, -0.38, 0.0)
        .root(16.0, 0.0, 0.0)
        .chest(-4.0, 0.0, 0.0)
        .neck(10.0, 0.0)
        .tail(-10.0, 0.0)
        .plant_fore(0.16, 0.0)
        .plant_hind(-0.34, 0.0);
    let up = airborne(0.2).root(22.0, 0.0, 0.0).neck(8.0, 0.0);
    let landed_on = Pose::standing()
        .hips(0.0, -0.26, 0.0)
        .root(-4.0, 0.0, 0.0)
        .neck(6.0, 0.0)
        .tail(12.0, 0.0)
        .plant_fore(0.14, 0.0)
        .plant_hind(-0.30, 0.0);
    Recipe::new(
        Clip::Perch,
        vec![
            Key::eased(0.0, ready, Ease::IN),
            Key::eased(mark(Clip::Perch, 0, 0.85), crouch, Ease::STRIKE),
            Key::eased(mark(Clip::Perch, 1, 0.4), up, Ease::SMOOTH),
            Key::eased(mark(Clip::Perch, 1, 1.0), landed_on, Ease::OUT),
            Key::at(1.0, ready),
        ],
        SPRING,
    )
    .noting(
        "Up onto a top in one leap: a deep crouch looking up at where it is \
         going, the stretch, and a crouched landing on the top. The height is \
         the simulation's -- the cat's body goes up onto the wall -- so the clip \
         only says it is jumping."
            .to_string(),
    )
}

fn dive() -> Recipe {
    // Head down over the lip, shoulders rolled forward: from below, the cat
    // hanging over the edge at you.
    let over = Pose::standing()
        .hips(-0.10, -0.12, 0.0)
        .root(-10.0, 0.0, 0.0)
        .spine(-6.0, 0.0, 0.0)
        .chest(-14.0, 0.0, 0.0)
        .neck(-26.0, 0.0)
        .head(18.0, 0.0, 0.0)
        .tail(26.0, 0.0)
        .plant_fore(0.30, 0.0)
        .plant_hind(-0.30, 0.0);
    let falling = airborne(0.1).root(-24.0, 0.0, 0.0).forelegs(70.0, -10.0);
    Recipe::new(
        Clip::Dive,
        vec![
            Key::eased(0.0, Pose::standing(), Ease::IN),
            Key::eased(mark(Clip::Dive, 0, 0.6), over, Ease::HOLD),
            Key::eased(mark(Clip::Dive, 0, 0.95), over, Ease::STRIKE),
            Key::eased(mark(Clip::Dive, 1, 0.3), falling, Ease::IN),
            Key::eased(mark(Clip::Dive, 1, 0.55), landed(), Ease::OUT),
            Key::eased(mark(Clip::Dive, 2, 0.5), landed(), Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        SPRING,
    )
    .noting(
        "Thirty frames hanging over the lip -- the head down at you, the \
         shoulders rolled forward -- then the drop, forelegs first, and the \
         landing crouch it was always going to need a long time to get out of."
            .to_string(),
    )
}

fn drop() -> Recipe {
    let ready = Pose::standing();
    let hop = airborne(0.0).root(-14.0, 0.0, 0.0);
    Recipe::new(
        Clip::Drop,
        vec![
            Key::eased(0.0, ready, Ease::IN),
            Key::eased(mark(Clip::Drop, 0, 1.0), hop, Ease::SMOOTH),
            Key::eased(mark(Clip::Drop, 1, 1.0), landed(), Ease::OUT),
            Key::at(1.0, ready),
        ],
        SPRING,
    )
    .noting(
        "Down from a perch the slow way, because whoever it wanted is under the \
         lip. Nothing to read in it but that it is coming down."
            .to_string(),
    )
}

fn interpose() -> Recipe {
    // Ears flat and head low toward the line: the snarl, drawn.
    let snarl = Pose::standing()
        .hips(-0.06, -0.16, 0.0)
        .root(6.0, 0.0, 0.0)
        .chest(-8.0, 0.0, 0.0)
        .neck(-22.0, 0.0)
        .head(18.0, 0.0, 0.0)
        .tail(-6.0, 0.0)
        .plant_fore(0.18, 0.0)
        .plant_hind(-0.32, 0.0);
    // The run, as a bound keyed into the active window: gather, reach.
    let gather = Pose::standing()
        .hips(0.0, -0.06, 0.0)
        .root(8.0, 0.0, 0.0)
        .spine(-14.0, 0.0, 0.0)
        .neck(-18.0, 0.0)
        .tail(10.0, 0.0)
        .plant_fore(-0.30, 0.25)
        .plant_hind(0.30, 0.0);
    let reach = Pose::standing()
        .hips(0.0, 0.08, 0.0)
        .root(-8.0, 0.0, 0.0)
        .spine(14.0, 0.0, 0.0)
        .neck(-10.0, 0.0)
        .tail(-4.0, 0.0)
        .plant_fore(0.60, 0.0)
        .plant_hind(-0.60, 0.30);
    let set = snarl.neck(-6.0, 0.0).hips(0.0, -0.20, 0.0);
    let mut keys = vec![
        Key::eased(0.0, Pose::standing(), Ease::IN),
        Key::eased(mark(Clip::Interpose, 0, 0.8), snarl, Ease::SNAP),
    ];
    for i in 0..6 {
        let u = i as f32 / 6.0;
        let pose = if i % 2 == 0 { gather } else { reach };
        keys.push(Key::eased(mark(Clip::Interpose, 1, u), pose, Ease::SMOOTH));
    }
    keys.push(Key::eased(mark(Clip::Interpose, 2, 0.3), set, Ease::HOLD));
    keys.push(Key::at(1.0, Pose::standing()));
    Recipe::new(Clip::Interpose, keys, Looseness::BEAST).noting(
        "The snarl, head low toward the line it is about to run; a straight run \
         to a place you can predict; and set, square to you, in front of its \
         mate. The run is a commitment and looks like one."
            .to_string(),
    )
}

fn howl() -> Recipe {
    let ready = Pose::standing();
    let up = Pose::standing()
        .hips(-0.10, -0.20, 0.0)
        .root(10.0, 0.0, 0.0)
        .spine(8.0, 0.0, 0.0)
        .chest(14.0, 0.0, 0.0)
        .neck(46.0, 0.0)
        .head(24.0, 0.0, 0.0)
        .tail(-20.0, 0.0)
        .plant_fore(0.04, 0.0)
        .plant_hind(-0.40, 0.0);
    Recipe::new(
        Clip::Howl,
        vec![
            Key::eased(0.0, ready, Ease::OUT),
            Key::eased(mark(Clip::Howl, 2, 0.2), up, Ease::HOLD),
            Key::eased(mark(Clip::Howl, 2, 0.8), up, Ease::SMOOTH),
            Key::at(1.0, ready),
        ],
        Looseness::BEAST,
    )
    .noting(
        "Over its mate's body, head to the sky, for a second and a quarter: \
         threatening nothing. It is the opening between the two fights."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// Being in trouble
// ---------------------------------------------------------------------------

fn flinch() -> Recipe {
    let hit = Pose::standing()
        .hips(-0.08, -0.06, 0.0)
        .root(6.0, -6.0, 5.0)
        .spine(-6.0, -4.0, 0.0)
        .neck(-18.0, -12.0)
        .head(10.0, 0.0, 0.0)
        .tail(16.0, -20.0)
        .plant_fore(-0.02, 0.0)
        .plant_hind(-0.24, 0.0);
    Recipe::new(
        Clip::Flinch,
        vec![
            Key::eased(0.0, hit, Ease::OUT),
            Key::eased(0.45, hit.blend(&Pose::standing(), 0.5), Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::LIMP,
    )
    .noting("A recoil, held at the start and bled off.".to_string())
}

fn stumble() -> Recipe {
    // Down on its chest, forelegs folded under, the hind end still up: a cat
    // tripped by crowd control, getting its feet back.
    let down = Pose::standing()
        .hips(0.0, -0.50, 0.0)
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
    .noting(
        "Tripped: down onto its chest with the forelegs folded under it, and a \
         slow push back up. A tripped Striker is a split bought with crowd \
         control, and this is how long it lasts."
            .to_string(),
    )
}

fn topple() -> Recipe {
    // On its side: the twin pounce's crash. The only pose with real roll.
    let over = Pose::standing()
        .hips(0.0, -0.80, 0.20)
        .root(-4.0, 0.0, 68.0)
        .spine(-4.0, 0.0, 10.0)
        .chest(-6.0, 0.0, 8.0)
        .neck(-20.0, 24.0)
        .head(16.0, 0.0, 0.0)
        .forelegs(30.0, -40.0)
        .hindlegs(36.0, -50.0)
        .leg_spread(0, 24.0)
        .leg_spread(1, 24.0)
        .leg_spread(2, 20.0)
        .leg_spread(3, 20.0)
        .tail(0.0, 34.0);
    let heaving = over.blend(&Pose::standing(), 0.45).hips(0.0, -0.45, 0.10);
    Recipe::new(
        Clip::Topple,
        vec![
            Key::eased(0.0, Pose::standing(), Ease::IN),
            Key::eased(0.10, over, Ease::HOLD),
            Key::eased(0.70, over, Ease::SMOOTH),
            Key::eased(0.86, heaving, Ease::OUT),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::COLLAPSE,
    )
    .noting(
        "Two cats on their sides, touching, a metre from where you were: the \
         longest opening in the fight. Over in a tenth of the clip, on the \
         floor for most of it, and the last stretch spent getting up."
            .to_string(),
    )
}

fn dead() -> Recipe {
    let gone = Pose::standing()
        .hips(0.0, -0.86, 0.24)
        .root(-6.0, 0.0, 78.0)
        .spine(-6.0, 0.0, 12.0)
        .chest(-8.0, 0.0, 10.0)
        .neck(-30.0, 30.0)
        .head(22.0, 0.0, 0.0)
        .forelegs(34.0, -30.0)
        .hindlegs(40.0, -44.0)
        .leg_spread(0, 28.0)
        .leg_spread(1, 28.0)
        .leg_spread(2, 24.0)
        .leg_spread(3, 24.0)
        .tail(-6.0, 40.0);
    Recipe::new(Clip::Dead, vec![Key::at(0.0, gone)], Looseness::LIMP)
        .noting("One pose.".to_string())
}

/// Every clip that has no recipe. Empty is the answer we want.
pub fn missing() -> Vec<Clip> {
    let have = all();
    Clip::ALL
        .into_iter()
        .filter(|c| !have.iter().any(|r| r.clip == c.index()))
        .collect()
}
