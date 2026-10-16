//! On a ledge: hanging from it, going along it hand over hand, and climbing
//! up on to it (2026-10-16, `sim::state::hang_on`).
//!
//! ## Where the edge is
//!
//! Everything here is placed against one fact the simulation decides: a body
//! hanging from an edge stands in the air **1.9 m under the top**
//! (`Ledge, hanging, feet under the top`) and **a body's radius and two skins
//! out from the face** -- the edge is 0.56 m in front of the feet and 1.9 m
//! above them, in character space. The hands are put there with `reach_*`
//! rather than keyed as angles, so they close on the rim whatever the build,
//! and the hang reads as *hanging from that edge* rather than as arms raised
//! near it.
//!
//! ## The climb is carried
//!
//! The body does not climb by itself: the simulation carries it along the
//! climb's path (`Grip::Climb`) -- straight up for the first 65% of the climb
//! (`Ledge, climbing up, share spent rising`), from the hang to the edge's
//! height, then forward on to the top. So in character space **the edge comes
//! down to meet the body** and then slides back under it, and the hands are
//! keyed on the edge where it is on each key's frame: 1.16 m up at frame 12,
//! 0.43 m at 24, on the floor at 31, then behind. The clip is as long as the
//! climb (`Length::LedgeClimb`), so the keys sit on the same frames however it
//! is tuned only if `ledge_climb` stays at 48 -- the frame numbers below are
//! written for that, and a retune of it wants them moved with it.

use crate::bake::{Key, Looseness, Recipe};
use crate::clips::locomotion::stance;
use crate::ease::Ease;
use view::clips::Clip;
use view::pose::{ANKLE_ON_GROUND as GROUND, Pose};

/// Where the wrists close on the edge, either side, in a still hang.
const GRIP: f32 = 0.19;
/// The edge, in front of the feet and over them, in a hang.
const EDGE_Z: f32 = 0.44;
const EDGE_Y: f32 = 1.86;
/// How far forward of the feet's mark a hanging body's hips are: under the
/// hands rather than behind them.
const HUNG_IN: f32 = 0.24;
/// And how far under standing height: the arms straight from the rim. A body
/// hung at its standing height has the rim inside its reach, and bends its
/// elbows out -- hands up, not hanging.
const HUNG_DOWN: f32 = 0.16;

pub fn clips() -> Vec<Recipe> {
    vec![hang(), shuffle(), climb()]
}

// ---------------------------------------------------------------------------
// The hang
// ---------------------------------------------------------------------------

/// Hanging: the body long under the hands, leaning in a little toward the
/// wall, the head up looking at the top it came for, the legs loose and a
/// little bent with the toes down. `k` sways it between the two ends of the
/// drift the held clip runs on.
fn hanging(k: f32) -> Pose {
    // The hips well forward of the feet's mark: the simulation holds the body
    // a collision radius off the face, which is wider than the drawn body, so
    // a body hung straight down from there would hang behind its own hands.
    // Forward, it hangs under them with its toes at the wall.
    Pose::rest()
        .hips(0.0, -HUNG_DOWN, HUNG_IN + 0.015 * k)
        .root(2.0 + 1.5 * k, 0.0, 0.0)
        .spine(-4.0, 0.0, 0.0)
        .chest(-4.0, 0.0, 0.0)
        .head(-24.0, 0.0, 0.0)
        .hip_l(12.0 + 3.0 * k, 4.0, 0.0)
        .knee_l(22.0 - 4.0 * k)
        .hip_r(8.0 - 3.0 * k, 4.0, 0.0)
        .knee_r(16.0 + 4.0 * k)
        .ankle_l(34.0, 0.0, 0.0)
        .ankle_r(30.0, 0.0, 0.0)
        .reach_l([-GRIP, EDGE_Y, EDGE_Z])
        .reach_r([GRIP, EDGE_Y, EDGE_Z])
}

fn hang() -> Recipe {
    Recipe {
        clip: Clip::LedgeHang,
        looseness: Looseness::HEAVY,
        notes: "A held shape with a slow sway on it, like the rise: indexed \
                by frames since the catch and entered from the fall by the \
                cross-fade, which is the catch's settle. Both hands on the rim \
                by `reach`, so they close on the edge on every build; the body \
                long and a little in toward the wall, the head up at the top, \
                the legs hanging loose. Heavy, because the whole weight is on \
                the hands and nothing in it should look as if it is resting."
            .into(),
        keys: vec![
            Key::eased(0, hanging(1.0), Ease::SMOOTH),
            Key::eased(45, hanging(-1.0), Ease::SMOOTH),
        ],
    }
}

// ---------------------------------------------------------------------------
// The shuffle
// ---------------------------------------------------------------------------

/// The hang, with the wrists at `left` and `right` across the body and each
/// lifted off the rim by its own height: the hands as the shuffle moves them,
/// and the legs swinging a little against the hand that is reaching.
fn shuffling(left: f32, right: f32, lift_l: f32, lift_r: f32, swing: f32) -> Pose {
    hanging(0.0)
        .root(4.0, 0.0, 3.0 * swing)
        .hip_l(6.0 - 6.0 * swing, 6.0, 0.0)
        .hip_r(2.0 + 6.0 * swing, 6.0, 0.0)
        .reach_l([left, EDGE_Y + lift_l, EDGE_Z])
        .reach_r([right, EDGE_Y + lift_r, EDGE_Z])
}

/// How high a hand comes off the rim as it goes over: enough to see, not a
/// reach.
const LIFT: f32 = 0.07;

fn shuffle() -> Recipe {
    // Sixteen frames, moving to her right at `Ledge, hand over hand at`
    // (1.5 m/s, 2.5 cm a frame): forty centimetres a cycle, so each hand
    // travels forty centimetres once. A planted hand slides back across the
    // body at the body's own speed, 2.5 cm a frame; the lifted one goes over
    // it twice as fast. The right leads, the left follows half a cycle later,
    // and they meet in the middle on frame 8 -- hand over hand.
    let (out, mid) = (GRIP, 0.02);
    Recipe {
        clip: Clip::LedgeShuffle,
        looseness: Looseness::HEAVY,
        notes: "Moving to her right: the right hand lands wide, slides back \
                across the body as it carries her, and is lifted over to land \
                wide again; the left does the same half a cycle behind it, so \
                the two meet in the middle and part. Played backwards it goes \
                left, because a planted hand sliding one way across the body, \
                run the other way in time, slides the other way. The planted \
                hand slides at the shuffle's own speed, so it stays on the \
                edge; retune `ledge_shuffle` and this wants its widths moved."
            .into(),
        keys: vec![
            Key::eased(0, shuffling(-out, out, 0.0, 0.0, 0.0), Ease::LINEAR),
            Key::eased(
                4,
                shuffling(-0.5 * (out + mid), 0.5 * (out + mid), LIFT, 0.0, -1.0),
                Ease::LINEAR,
            ),
            Key::eased(8, shuffling(-mid, mid, 0.0, 0.0, 0.0), Ease::LINEAR),
            Key::eased(
                12,
                shuffling(-0.5 * (out + mid), 0.5 * (out + mid), 0.0, LIFT, 1.0),
                Ease::LINEAR,
            ),
        ],
    }
}

// ---------------------------------------------------------------------------
// The climb
// ---------------------------------------------------------------------------

/// The start of the pull: the hands still on the edge, which the climb has
/// brought down to 1.49 m over the feet, and the elbows starting to bend under
/// it. Without it the arms go from overhead to the pull-up by swinging out
/// straight in front.
fn starting_to_pull() -> Pose {
    hanging(0.0)
        .hips(0.0, -0.10, HUNG_IN)
        .root(6.0, 0.0, 0.0)
        .hip_l(22.0, 6.0, 0.0)
        .knee_l(36.0)
        .hip_r(14.0, 6.0, 0.0)
        .knee_r(28.0)
        .reach_l([-0.22, 1.49, EDGE_Z])
        .reach_r([0.22, 1.49, EDGE_Z])
}

/// The pull-up: chin to the rim, elbows bent hard and out, the knees coming up
/// under the body to help. The edge is 1.16 m over the feet by now.
fn pulling() -> Pose {
    Pose::rest()
        .hips(0.0, -0.06, HUNG_IN)
        .root(10.0, 0.0, 0.0)
        .spine(8.0, 0.0, 0.0)
        .chest(6.0, 0.0, 0.0)
        .head(-12.0, 0.0, 0.0)
        .hip_l(34.0, 6.0, 0.0)
        .knee_l(58.0)
        .hip_r(22.0, 6.0, 0.0)
        .knee_r(46.0)
        .ankle_l(24.0, 0.0, 0.0)
        .ankle_r(20.0, 0.0, 0.0)
        .reach_l([-0.24, 1.16, EDGE_Z])
        .reach_r([0.24, 1.16, EDGE_Z])
}

/// The press: the arms straight under the shoulders and the torso up over the
/// edge, the right knee brought high to go on to the top. The edge is 0.43 m
/// over the feet.
fn pressing() -> Pose {
    Pose::rest()
        .hips(0.0, -0.10, 0.05)
        .root(26.0, 0.0, 0.0)
        .spine(16.0, 0.0, 0.0)
        .chest(6.0, 0.0, 0.0)
        .head(-24.0, 0.0, 0.0)
        // The left leg hangs straight down the face under the leaned body.
        .hip_l(26.0, 6.0, 0.0)
        .knee_l(16.0)
        .hip_r(84.0, 8.0, 0.0)
        .knee_r(112.0)
        .ankle_l(30.0, 0.0, 0.0)
        .ankle_r(10.0, 0.0, 0.0)
        .reach_l([-0.26, 0.43, 0.46])
        .reach_r([0.26, 0.43, 0.46])
}

/// On the lip: the feet at the top's height, the right foot planted on it, the
/// left still over the drop, the body folded low with both hands flat on the
/// top in front.
fn kneeling() -> Pose {
    Pose::rest()
        .hips(0.0, -0.42, 0.06)
        .root(34.0, 0.0, 0.0)
        .spine(22.0, 0.0, 0.0)
        .chest(8.0, 0.0, 0.0)
        .head(-30.0, 0.0, 0.0)
        .plant_r([0.13, GROUND, 0.38])
        .toe_r(0.0)
        // Over the drop: hanging down off the lip under the leaned body, not
        // stretched back along a floor that is not there.
        .hip_l(36.0, 6.0, 0.0)
        .knee_l(18.0)
        .ankle_l(30.0, 0.0, 0.0)
        .reach_l([-0.28, 0.12, 0.62])
        .reach_r([0.28, 0.12, 0.62])
}

/// Coming up off the lip: the body being carried forward over the top, the
/// left foot brought through to stand beside the right, the hands leaving the
/// floor.
fn standing_up() -> Pose {
    Pose::rest()
        .hips(0.0, -0.22, 0.02)
        .root(16.0, 0.0, -6.0)
        .spine(10.0, 0.0, 3.0)
        .chest(4.0, 0.0, 4.0)
        .head(-10.0, 0.0, 2.0)
        .shoulder_l(30.0, 14.0, 0.0)
        .elbow_l(40.0)
        .shoulder_r(24.0, 16.0, 0.0)
        .elbow_r(36.0)
        .plant_l([-0.13, GROUND + 0.05, 0.04])
        .plant_r([0.13, GROUND, -0.06])
        .toe_l(0.0)
        .toe_r(0.0)
}

fn climb() -> Recipe {
    Recipe {
        clip: Clip::LedgeClimb,
        // Heavy: this is the slow way up, and every part of it should cost
        // something to look at.
        looseness: Looseness::HEAVY,
        notes: "Forty-eight frames, and slow on purpose: the ledge is the \
                forgiving way up and the jump the quick one. The simulation \
                carries the body up the face for the first thirty-one frames \
                and over the top for the rest, so the edge comes down to meet \
                the hands: they are keyed on it where it is. A pull-up with \
                the knees coming up, a press to straight arms with the right \
                knee brought high, a fold on the lip with the right foot \
                planted on the top and both hands flat, and standing up into \
                the stance the idle starts from."
            .into(),
        keys: vec![
            Key::eased(0, hanging(0.0), Ease::SMOOTH),
            Key::eased(6, starting_to_pull(), Ease::SMOOTH),
            Key::eased(12, pulling(), Ease::SMOOTH),
            Key::eased(24, pressing(), Ease::SMOOTH),
            Key::eased(31, kneeling(), Ease::SMOOTH),
            Key::eased(40, standing_up(), Ease::SMOOTH),
            Key::eased(47, stance(), Ease::OUT),
        ],
    }
}
