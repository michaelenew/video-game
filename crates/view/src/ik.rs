//! Two-bone inverse kinematics: say where the hand or foot goes, get the
//! angles that put it there.
//!
//! This is the half of "the kinematics are handled" that an author actually
//! feels. Keying a walk cycle in joint angles means discovering, one frame at a
//! time, that the planted foot is sliding a centimetre a frame -- and foot
//! skate is the single most legible sign of animation done badly. Keying it as
//! *"the foot is at this spot on the ground, and stays there for eight
//! frames"* makes the skate impossible rather than merely avoidable.
//!
//! The solve is closed form. A two-link chain reaching a point has exactly two
//! solutions mirrored through the line between the joint and the target; a
//! **pole** direction picks which one, and it is the same thing the knee does
//! in life -- it decides which way the joint folds.
//!
//! Reach is clamped, not extrapolated. Asking for a target further away than
//! the leg is long gives a straight leg pointed at it, which is what a person
//! straining toward something looks like, rather than a detached foot.

use crate::math::{self, Quat, V3};
use crate::pose::Pose;
use crate::skeleton::{Joint, Skeleton, aim, solve};

/// Which solution to take: the direction the middle joint should stick out in,
/// in character-local space.
///
/// Knees go forward, elbows go back and out. These are the defaults and they
/// are right almost always; a pose that wants a chicken-winged elbow says so
/// explicitly.
pub const KNEE_FORWARD: V3 = [0.0, 0.0, 1.0];
pub const ELBOW_BACK: V3 = [0.0, 0.0, -1.0];

/// Point a two-bone chain at a target, and write the angles back into the pose.
///
/// `upper` is the chain's first joint -- a thigh or an upper arm. The target is
/// in character-local space, the same space the pose's root offset is in, so
/// "the left foot is on the ground 20 cm ahead" is `[-0.11, 0.06, 0.20]`.
///
/// Returns how far short of the target the chain ended up, in metres. Zero
/// almost always; non-zero when the target was out of reach or a joint limit
/// refused the angle, and worth asserting on in a test that cares.
pub fn reach(pose: &mut Pose, skeleton: &Skeleton, upper: Joint, target: V3, pole: V3) -> f32 {
    let middle = child_of(upper);
    let l1 = skeleton.bone(upper).length;
    let l2 = skeleton.bone(middle).length;

    // Everything above this chain has to be posed first: a hand's target is
    // measured from the shoulder, and the shoulder has already been moved by
    // the spine and the chest.
    let skin = solve(skeleton, pose);
    let parent = upper.parent().expect("a limb hangs off something");
    let origin = skin.origin[upper.index()];
    let parent_rot = skin.rot[parent.index()];

    // Into the parent's frame, where the joint angles are expressed.
    let inv = parent_rot.conjugate();
    let delta = inv.rotate(math::sub(target, origin));
    let pole_local = inv.rotate(pole);

    let reach_max = (l1 + l2) * 0.999;
    let reach_min = (l1 - l2).abs() * 1.001 + 1e-4;
    let raw = math::length(delta);
    let d = raw.clamp(reach_min, reach_max);
    let dir = math::normalize_or(delta, [0.0, -1.0, 0.0]);

    // Law of cosines twice: once for how far the middle joint folds, once for
    // how far the first bone sits off the straight line to the target.
    let cos_interior = ((l1 * l1 + l2 * l2 - d * d) / (2.0 * l1 * l2)).clamp(-1.0, 1.0);
    let bend = std::f32::consts::PI - cos_interior.acos();
    let cos_alpha = ((l1 * l1 + d * d - l2 * l2) / (2.0 * l1 * d)).clamp(-1.0, 1.0);
    let alpha = cos_alpha.acos();

    // Swing the first bone off the target line, toward the pole.
    let along = math::dot(pole_local, dir);
    let perp = math::sub(pole_local, math::scale(dir, along));
    let perp = math::normalize_or(perp, fallback_perp(dir));
    let upper_dir = math::add(
        math::scale(dir, alpha.cos()),
        math::scale(perp, alpha.sin()),
    );

    let (swing, spread) = aim(skeleton.bone(upper), upper, upper_dir);

    // The bend has to happen in the plane containing the target, which is what
    // the twist channel is for: it points the hinge's axis.
    let knee = math::add(math::scale(upper_dir, l1), math::ZERO);
    let lower_dir = math::normalize_or(math::sub(delta, knee), upper_dir);
    // The hinge turns about the middle bone's own X axis -- but a joint whose
    // positive direction is *flexion* (an elbow) spins the opposite way from
    // one whose positive direction is *folding back* (a knee), and that sign
    // lives in the bone. Miss it and the legs solve perfectly while every arm
    // reaches a foot wide of the mark.
    let hinge = math::scale(
        math::normalize_or(math::cross(upper_dir, lower_dir), perp),
        skeleton.bone(middle).swing_sign,
    );
    let twist = twist_for_axis(skeleton, upper, swing, spread, hinge);

    pose.set_angles(upper, [swing, spread, twist]);
    pose.set_angles(middle, [bend, 0.0, 0.0]);

    // Report the truth: limits and reach are both allowed to refuse.
    let after = solve(skeleton, pose);
    math::length(math::sub(after.origin[child_of(middle).index()], target))
}

/// Plant a foot. The target is the **ankle**, which is what a pose is actually
/// specifying when it says where a foot is.
pub fn foot_to(pose: &mut Pose, skeleton: &Skeleton, left: bool, target: V3) -> f32 {
    let upper = if left { Joint::ThighL } else { Joint::ThighR };
    reach(pose, skeleton, upper, target, KNEE_FORWARD)
}

/// Put a hand somewhere. The target is the wrist.
pub fn hand_to(pose: &mut Pose, skeleton: &Skeleton, left: bool, target: V3) -> f32 {
    let upper = if left { Joint::ArmL } else { Joint::ArmR };
    let pole = if left {
        [-0.35, -0.2, -1.0]
    } else {
        [0.35, -0.2, -1.0]
    };
    reach(pose, skeleton, upper, target, pole)
}

fn child_of(j: Joint) -> Joint {
    match j {
        Joint::ThighL => Joint::ShinL,
        Joint::ShinL => Joint::FootL,
        Joint::ThighR => Joint::ShinR,
        Joint::ShinR => Joint::FootR,
        Joint::ArmL => Joint::ForearmL,
        Joint::ForearmL => Joint::HandL,
        Joint::ArmR => Joint::ForearmR,
        Joint::ForearmR => Joint::HandR,
        other => other,
    }
}

fn fallback_perp(dir: V3) -> V3 {
    let seed = if dir[2].abs() < 0.9 {
        [0.0, 0.0, 1.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    math::normalize_or(math::cross(math::cross(dir, seed), dir), [0.0, 0.0, 1.0])
}

/// Which twist puts the hinge's axis where the bend needs it.
///
/// The bone's own X axis is the hinge. Under the pose composition, holding
/// swing and spread fixed and varying twist traces that axis around a circle,
/// so the answer is one `atan2` rather than a search.
fn twist_for_axis(skeleton: &Skeleton, joint: Joint, swing: f32, spread: f32, axis: V3) -> f32 {
    let bone = skeleton.bone(joint);
    let mirror = if joint.is_left() { -1.0 } else { 1.0 };
    let base = Quat::from_z(spread * mirror).mul(Quat::from_x(swing * bone.swing_sign));
    let a = base.rotate([1.0, 0.0, 0.0]);
    let b = base.rotate([0.0, 0.0, 1.0]);
    let u = (-math::dot(axis, b)).atan2(math::dot(axis, a));
    u * mirror
}
