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
use crate::skeleton::{Joint, Skeleton, aim_within, pointing, solve};

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

    let held = pose.angles(upper);
    let (swing, spread) = aim_within(skeleton.bone(upper), upper, upper_dir, (held[0], held[1]));

    // What the joint will actually hold.
    //
    // A shoulder asked to point somewhere it cannot has its swing and spread
    // clamped, and the bone then points somewhere else -- so everything after
    // this has to be worked out against where the bone *went*. Folding the
    // elbow for the direction that was refused is what turns "out of range" of
    // a few centimetres into a hand most of a metre from the target, on the
    // wrong side of the body, with no complaint from anything.
    let upper_dir = pointing(skeleton.bone(upper), upper, swing, spread);

    // The bend has to happen in the plane containing the target, which is what
    // the twist channel is for: it points the hinge's axis.
    //
    // Taken from the elbow's real position rather than from the law of
    // cosines, which is the same number whenever the shoulder got what it
    // asked for and a much better one when it did not: the forearm still
    // points at the target, so an unreachable target gives a limb straining
    // toward it and coming up short.
    let knee = math::scale(upper_dir, l1);
    let lower_dir = math::normalize_or(math::sub(delta, knee), upper_dir);
    let bend = math::dot(upper_dir, lower_dir).clamp(-1.0, 1.0).acos();
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

    // Clamped to what the joint can do. The solve is a closed form and will
    // happily return an angle no hip has: asking for a foot behind and across
    // the other leg produces a hundred degrees of adduction, and without this
    // that lands in the authored pose as a number a body cannot hold. The limb
    // reaches as far as the joint allows and stops, which is what a body does.
    let limits = |j: Joint, a: [f32; 3]| {
        let (s, p, t) = skeleton.bone(j).limits.clamp(a[0], a[1], a[2]);
        [s, p, t]
    };
    pose.set_angles(upper, limits(upper, [swing, spread, twist]));
    pose.set_angles(middle, limits(middle, [bend, 0.0, 0.0]));

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
///
/// An elbow is not pinned. With the shoulder and the wrist both fixed, the
/// elbow is still free to travel a whole circle around the line between them
/// -- the *swivel* -- and the pole direction is only a way of naming a point
/// on that circle. A leg barely needs the freedom, because a knee that does
/// not point forward is an injury. An arm needs all of it: hanging by the ribs
/// the elbow sits behind and below, reaching forward it tucks in, and overhead
/// it swings out to the side, which is why a press and a punch look nothing
/// alike from behind.
///
/// So rather than name one point on the circle and hope, this walks the
/// circle. Every candidate is scored on how close the wrist actually lands to
/// where it was sent -- joint limits included, so a swivel the shoulder cannot
/// hold scores badly and loses -- with a small preference for the elbow being
/// down and outward, which is what breaks the tie among the many that work.
///
/// A fixed pole is what this replaced, and it failed in a way worth recording
/// because it was invisible: a wrist asked for 30 cm above its own shoulder,
/// comfortably inside the arm's reach, came out 47 cm away, hanging by the
/// ribs. Overhead with the elbow behind you is a triangle that can only be
/// built with the upper arm pointing up and *behind* the shoulder, no shoulder
/// does that, and the limits quietly refused it. Nothing reported an error;
/// every Champion pose was simply not the pose that had been written.
pub fn hand_to(pose: &mut Pose, skeleton: &Skeleton, left: bool, target: V3) -> f32 {
    let upper = if left { Joint::ArmL } else { Joint::ArmR };
    let side = if left { -1.0 } else { 1.0 };
    let shoulder = solve(skeleton, pose).origin[upper.index()];

    // The axis the elbow swivels around, and the elbow direction we would pick
    // if every one of them worked: down, back and a little out from the body.
    let axis = math::normalize_or(math::sub(target, shoulder), [0.0, -1.0, 0.0]);
    let wish = [side * 0.45, -0.85, -0.55];
    let wish = math::normalize_or(
        math::sub(wish, math::scale(axis, math::dot(wish, axis))),
        fallback_perp(axis),
    );

    // A sixteenth of a turn is finer than the difference is visible, and the
    // score is smooth in between, so there is nothing to gain from more.
    const STEPS: usize = 16;
    let mut best = (f32::INFINITY, wish);
    for i in 0..STEPS {
        let angle = i as f32 / STEPS as f32 * std::f32::consts::TAU;
        let candidate = Quat::axis_angle(axis, angle).rotate(wish);
        let mut trial = pose.clone();
        let miss = reach(&mut trial, skeleton, upper, target, candidate);
        // Centimetres of error against a preference measured in the same
        // units: half the swivel away from the natural elbow is worth about
        // three centimetres of miss, so a solve that lands the wrist wins
        // against one that merely looks comfortable.
        let unnatural = 1.0 - math::dot(candidate, wish);
        let score = miss + 0.03 * unnatural;
        if score < best.0 {
            best = (score, candidate);
        }
    }
    reach(pose, skeleton, upper, target, best.1)
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
