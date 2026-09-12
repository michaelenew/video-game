//! Weapon trails, for free, because the pose function is pure.
//!
//! The usual way to draw a swing trail is to record where the weapon was on
//! each of the last few frames and join the dots. That means a history buffer,
//! and under rollback a history buffer is a liability: re-simulating frame 90
//! twice appends the same point twice, a rollback of eight frames leaves eight
//! stale points in the ring, and the trail smears backwards through a swing
//! that never happened.
//!
//! None of that is necessary here. **Pose is a pure function of simulation
//! state**, which the architecture already guarantees for rollback's sake --
//! and a pure function can be asked about the past. Where the hand was eight
//! frames ago is `pose_for` evaluated at `frames_into - 8`. No buffer, nothing
//! in the snapshot, nothing to invalidate, and correct across a rollback by
//! construction, because the recomputation uses the corrected state.
//!
//! That is the same property the architecture leans on for the debug overlay:
//! anything derived from the rules cannot drift from the rules. Here it turns a
//! stateful feature into a stateless one.
//!
//! ## What it cannot do
//!
//! Reach back past the start of the current move. The counter that drives the
//! pose runs from the move's first frame, and the previous move's identity is
//! not recoverable from the current pose input -- so a trail stops at the
//! beginning of the swing it belongs to.
//!
//! Which is exactly as far as a trail should go. A swing's arc is the swing,
//! and a trail that carried on through the move before it would be drawing a
//! motion the player never made.

use crate::pose::{Part, Pose, PoseInput, pose_for};

/// The most frames a trail ever reaches back.
///
/// Eight is a little over an eighth of a second at sixty, which is the length
/// of the arc the eye reads as one motion. Longer starts to look like a
/// ribbon left lying in the air.
pub const MAX_SAMPLES: usize = 8;

/// One sample of where an edge was.
///
/// Two points, not one. A blade has length, and a trail drawn from a single
/// point per frame is a wire -- it has no width to catch the light and reads
/// as a line drawn on the screen rather than as a volume the weapon swept.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge {
    /// Character-local, the same space `Pose` uses: root at the feet, +Y up,
    /// +Z along facing.
    pub near: [f32; 3],
    pub far: [f32; 3],
    /// How far back this sample is, 0 at the present and 1 at the oldest.
    /// The renderer fades on it.
    pub age: f32,
}

/// Where the weapon has been, newest first.
///
/// Returns nothing at all when the action is not a swing -- deciding *that* is
/// the caller's job, because `view` does not read the move tables.
pub fn sweep(input: PoseInput, samples: usize) -> Vec<Edge> {
    let samples = samples.min(MAX_SAMPLES);

    // **How far back the trail can reach is set by whatever drives the pose**,
    // and when a clip is playing that is the clip's own elapsed count, not
    // `frames_into`.
    //
    // This was the bug that made the first version nearly invisible.
    // `frames_into` counts within the current *phase*, so at the start of a
    // move's active window it is zero to three -- and the active window is
    // exactly when the swing is fastest and the trail should be longest. Every
    // swing got a one-quad ribbon a few centimetres long, which read as
    // nothing at all.
    //
    // A clip's elapsed count runs across the whole move, so walking it back
    // crosses from active into startup and picks up the wind-up, which is
    // where most of an arc lives.
    let reach = match input.clip {
        Some((_, elapsed)) => elapsed,
        None => input.frames_into,
    };

    let mut out = Vec::with_capacity(samples);
    for back in 0..samples {
        // Stop at the start rather than wrapping into negative frames, which
        // would silently index the pose function with a huge `u16` and hand
        // back a pose from the end of the move.
        if back as u16 > reach {
            break;
        }
        let past = PoseInput {
            frames_into: input.frames_into.saturating_sub(back as u16),
            clip: input.clip.map(|(c, elapsed)| (c, elapsed - back as u16)),
            ..input
        };
        out.push(edge_of(
            &pose_for(past),
            back as f32 / samples.max(1) as f32,
        ));
    }
    out
}

/// The weapon edge implied by a pose.
///
/// The standin has no weapon, so the edge is taken from the right arm: the
/// hand end of it, and a point carried on past it by the length a weapon would
/// have. When there is a real weapon mesh this reads its tip instead and
/// nothing else here changes -- which is the same bet the pose function itself
/// makes about skeletal animation arriving later.
fn edge_of(pose: &Pose, age: f32) -> Edge {
    let arm = pose.get(Part::ArmR);

    // Down the limb is -Y at rest, which is the convention the poses are
    // written in, turned by the part's own rotation.
    //
    // Applied as three explicit turns rather than a hand-expanded matrix. The
    // first attempt here *was* a hand-expanded matrix and it was wrong in a way
    // that is nearly impossible to see: a trail pointing slightly off the blade
    // still looks like a trail. Three obvious lines and two tests that pin the
    // cases anyone can check by holding their own arm out.
    let unit = normalise(rot_z(
        rot_y(rot_x([0.0, -1.0, 0.0], arm.rot[0]), arm.rot[1]),
        arm.rot[2],
    ));

    const ARM: f32 = 0.55;
    const BLADE: f32 = 0.95;
    let near = [
        arm.pos[0] + unit[0] * ARM,
        arm.pos[1] + unit[1] * ARM,
        arm.pos[2] + unit[2] * ARM,
    ];
    let far = [
        near[0] + unit[0] * BLADE,
        near[1] + unit[1] * BLADE,
        near[2] + unit[2] * BLADE,
    ];
    Edge { near, far, age }
}

fn rot_x(v: [f32; 3], a: f32) -> [f32; 3] {
    let (s, c) = a.sin_cos();
    [v[0], v[1] * c - v[2] * s, v[1] * s + v[2] * c]
}

fn rot_y(v: [f32; 3], a: f32) -> [f32; 3] {
    let (s, c) = a.sin_cos();
    [v[2] * s + v[0] * c, v[1], v[2] * c - v[0] * s]
}

fn rot_z(v: [f32; 3], a: f32) -> [f32; 3] {
    let (s, c) = a.sin_cos();
    [v[0] * c - v[1] * s, v[0] * s + v[1] * c, v[2]]
}

fn normalise(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 1e-5 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0, -1.0, 0.0]
    }
}

/// How far the edge travelled between the newest two samples, in metres.
///
/// The renderer uses it to decide whether a trail is worth drawing at all: a
/// move whose arm barely moves does not need a ribbon, and drawing one
/// anyway puts a smear on every jab.
pub fn speed(sweep: &[Edge]) -> f32 {
    let (Some(a), Some(b)) = (sweep.first(), sweep.get(1)) else {
        return 0.0;
    };
    let d = [
        a.far[0] - b.far[0],
        a.far[1] - b.far[1],
        a.far[2] - b.far[2],
    ];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}
