//! On a wall: sliding down it, and kicking off it (2026-10-20,
//! `sim::state::Player::sliding`, `wall_jumps`).
//!
//! ## Where the face is
//!
//! Both clips are cut for **a wall on her left**, and mirrored for one on her
//! right (`view::play::on_the_wall`). The renderer turns the drawn body
//! side-on to the wall for them (`interp::OnWall::turn`), so in character
//! space the face is a plane at `x = FACE`: the simulation holds a sliding
//! body **a body's radius off the face** (`Body radius`, 0.5 m), measured from
//! the feet's mark. The near hand and foot are put on it with `reach_l` and
//! `plant_l` rather than keyed as angles, so they press on the face whatever
//! the build.
//!
//! ## The judder is stick and slip
//!
//! A hand dragged down a wall does not glide: it holds, the body goes on
//! sliding under it, and it lets go and catches again lower down. In
//! character space a held hand on a body going down **rises**, then drops
//! back when it slips. That is the whole of the slide's loop -- the hand and
//! the foot dragged up the body by how far it has slid, and let go -- and the
//! renderer plays more of it the faster the slide goes, so a slow slide is a
//! body pressed still on the face and a fast one shudders.
//!
//! ## The kick is indexed by distance
//!
//! `wall_kick` is played by how far off the face she is since she jumped
//! (`interp::OnWall::kick`), not by time: frame 0 is against it, the last
//! frame `view::play::KICK_REACH` out. So the feet are keyed on the face where
//! it is on each key's frame -- driving it away from her -- and then gathered
//! under her as she leaves it behind.

use crate::bake::{Key, Looseness, Recipe};
use crate::ease::Ease;
use view::clips::Clip;
use view::pose::Pose;

/// The face, in character space: a body's radius to her left, less the
/// width of a palm so the hand presses on it rather than into it.
const FACE: f32 = -0.47;
/// Where the near wrist and ankle are pressed on the face, at rest in the
/// slide, and how far up the body the drag carries them before they slip.
const HAND_Y: f32 = 1.22;
const FOOT_Y: f32 = 0.46;
const DRAG: f32 = 0.11;

pub fn clips() -> Vec<Recipe> {
    vec![slide(), kick()]
}

// ---------------------------------------------------------------------------
// The slide
// ---------------------------------------------------------------------------

/// Pressed to the face: side-on and turned a little back to it, leaning in,
/// the near hand flat on it at the shoulder and the near foot on it at the
/// knee's height, the far leg hanging braced under her and the far arm out
/// for balance, the head turned down and away to where she is going. `drag`
/// is how far the hand and foot have been carried up the body by the slide
/// while they held, nought to one; `jolt` is the body's shake as they slip.
fn pressed(drag: f32, jolt: f32) -> Pose {
    Pose::rest()
        // Over towards the face and leaning in to it, and down into a brace.
        .hips(-0.08 + 0.015 * jolt, -0.12 - 0.03 * drag, 0.02)
        .root(8.0, -7.0 - 2.0 * jolt, 22.0)
        .spine(6.0, 5.0 - 2.0 * drag, -4.0)
        .chest(4.0, 4.0 - 3.0 * drag, -6.0)
        .head(22.0 - 4.0 * jolt, -4.0, 34.0)
        // The far arm out from the body and a little forward: balance, not a
        // reach.
        .shoulder_r(10.0, 52.0 + 6.0 * jolt, 0.0)
        .elbow_r(38.0)
        .wrist_r(10.0, 0.0, 0.0)
        // The far leg hangs braced under her: bent at the knee, the toe
        // pointed down at the floor she is going to meet.
        .hip_r(16.0 - 4.0 * drag, 6.0, 0.0)
        .knee_r(34.0 + 6.0 * drag)
        .ankle_r(28.0, 0.0, 0.0)
        // The near hand flat on the face at the shoulder, dragged up as it
        // holds.
        .shoulder_l(10.0, 40.0, 0.0)
        .elbow_l(30.0)
        .reach_l([FACE, HAND_Y + DRAG * drag, 0.10])
        .wrist_l(-30.0, 0.0, 0.0)
        // The near foot on the face, the knee out to it, the sole turned in
        // against it.
        .hip_l(34.0, 30.0, 0.0)
        .knee_l(64.0)
        .plant_l([FACE + 0.08, FOOT_Y + 0.8 * DRAG * drag, 0.18])
        .ankle_l(10.0, -16.0, 0.0)
}

fn slide() -> Recipe {
    Recipe {
        clip: Clip::WallSlide,
        // Crisp: the slip is a snap, and anything loose smears it into a
        // wobble.
        looseness: Looseness::CRISP,
        notes: "A twelve-frame loop of stick and slip, for a wall on her \
                left (mirrored for the right). The near hand and foot are \
                pressed on the face by `reach` and `plant`; they hold while \
                the body slides on under them, so they ride up it for seven \
                frames, then let go in two and catch again lower down, and \
                the body jolts as they do. The renderer plays this by how \
                fast she is going down the wall and holds frame 0 when she \
                is not, so frame 0 is the still press."
            .into(),
        keys: vec![
            Key::eased(0, pressed(0.0, 0.0), Ease::LINEAR),
            Key::eased(7, pressed(1.0, 0.0), Ease::SNAP),
            Key::eased(9, pressed(-0.15, 1.0), Ease::OUT),
        ],
    }
}

// ---------------------------------------------------------------------------
// The kick
// ---------------------------------------------------------------------------

/// Coiled against the face to push: both feet on it, the knees deep, the
/// near hand still on it, the body bunched and turned away from it.
fn coiled() -> Pose {
    Pose::rest()
        .hips(-0.10, -0.24, 0.0)
        .root(14.0, 14.0, 30.0)
        .spine(10.0, 6.0, 0.0)
        .chest(6.0, 4.0, 0.0)
        .head(4.0, -6.0, 30.0)
        .shoulder_r(0.0, 55.0, 0.0)
        .elbow_r(45.0)
        .reach_l([FACE, 1.22, 0.10])
        .plant_l([FACE + 0.06, 0.52, 0.22])
        .plant_r([FACE + 0.10, 0.34, -0.06])
        .ankle_l(0.0, -16.0, 0.0)
        .ankle_r(0.0, -16.0, 0.0)
}

/// Driving off it: the legs straightening out along the push, the feet on
/// the face as it falls away (`away` metres further off it than the slide),
/// the arms thrown up and out the way she is going.
fn driving(away: f32) -> Pose {
    Pose::rest()
        .hips(0.02, -0.08, 0.0)
        .root(4.0, -10.0, 42.0)
        .spine(-4.0, -6.0, 6.0)
        .chest(-6.0, -4.0, 6.0)
        .head(-14.0, 6.0, 20.0)
        .shoulder_l(-30.0, 70.0, 0.0)
        .elbow_l(20.0)
        .shoulder_r(-40.0, 80.0, 0.0)
        .elbow_r(16.0)
        .plant_l([FACE - away, 0.36, 0.20])
        .plant_r([FACE - away + 0.04, 0.22, -0.04])
        .ankles(20.0, -10.0, 0.0)
}

/// Clear of it, gathering: the knees coming up under her for the flight, the
/// arms coming round, still turned away from the wall she left.
fn gathered() -> Pose {
    Pose::rest()
        .hips(0.0, -0.06, 0.0)
        .root(-4.0, -4.0, 40.0)
        .spine(-2.0, 0.0, 4.0)
        .chest(-4.0, 0.0, 4.0)
        .head(-8.0, 0.0, 10.0)
        .shoulders(-20.0, 46.0, 0.0)
        .elbows(30.0)
        .hip_l(40.0, 10.0, 0.0)
        .knee_l(70.0)
        .hip_r(24.0, 8.0, 0.0)
        .knee_r(56.0)
        .ankles(30.0, 0.0, 0.0)
}

fn kick() -> Recipe {
    Recipe {
        clip: Clip::WallKick,
        looseness: Looseness::CRISP,
        notes: "Fourteen frames over the first metre and a bit off the face \
                (`view::play::KICK_REACH`), for a wall on her left: coiled \
                on it, both legs driving it away -- keyed on the face where \
                it is at each frame's distance -- and gathered and turned \
                away from it as she leaves it. The renderer blends it into \
                the flight as it runs out, so the last frame is a flight's \
                shape, not a landing's."
            .into(),
        keys: vec![
            Key::eased(0, coiled(), Ease::SMOOTH),
            Key::eased(7, driving(0.5), Ease::SMOOTH),
            Key::eased(13, gathered(), Ease::OUT),
        ],
    }
}
