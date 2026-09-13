//! The frame has to behave like a body.
//!
//! These are the tests that let every animation downstream be written without
//! re-deriving the sign conventions. If `positive swing is forward` stops being
//! true, fifty clips silently become wrong, so it is pinned here rather than
//! in a comment.

use view::ik;
use view::math;
use view::pose::{Pose, reference};
use view::skeleton::{self, Build, JOINTS, Joint, Skeleton};

fn rest_skin(s: &Skeleton) -> skeleton::Skin {
    skeleton::solve(s, &Pose::rest())
}

// ---------------------------------------------------------------------------
// Proportions
// ---------------------------------------------------------------------------

#[test]
fn the_reference_body_is_the_height_the_simulation_thinks_it_is() {
    // A stand-in that does not match its own hurtbox teaches the wrong
    // spacing, which is worse than a stand-in that looks crude.
    let s = reference();
    let sim_height = sim::tuning::body_height().to_f32_for_render();
    assert!(
        (s.total_height - sim_height).abs() < 0.06,
        "skeleton is {:.2} m, the simulation's body is {sim_height:.2} m",
        s.total_height
    );
}

#[test]
fn at_rest_the_feet_are_on_the_floor() {
    for class in sim::class::ALL_CLASSES {
        let s = skeleton::skeleton_for(class);
        let sole = Pose::rest().lowest_foot(&s);
        assert!(
            sole.abs() < 0.02,
            "{} stands with its soles at {sole:.3}",
            class.name()
        );
    }
}

#[test]
fn every_class_is_recognisably_a_different_body() {
    // Weight is the most legible thing a character can have. If two builds are
    // within a few percent of each other on every axis, one of them is not
    // earning its place.
    let builds: Vec<_> = sim::class::ALL_CLASSES
        .iter()
        .map(|c| (c.name(), skeleton::build_for(*c)))
        .collect();
    for (i, (an, a)) in builds.iter().enumerate() {
        for (bn, b) in builds.iter().skip(i + 1) {
            let spread = (a.scale - b.scale).abs()
                + (a.bulk - b.bulk).abs()
                + (a.limb - b.limb).abs()
                + (a.shoulders - b.shoulders).abs()
                + (a.stance - b.stance).abs()
                + (a.head - b.head).abs();
            assert!(spread > 0.08, "{an} and {bn} are the same body ({spread})");
        }
    }
}

#[test]
fn parents_come_before_children() {
    // Forward kinematics is one pass with no recursion, which is only sound if
    // the order holds.
    for (i, j) in JOINTS.iter().enumerate() {
        if let Some(p) = j.parent() {
            assert!(p.index() < i, "{} is solved before its parent", j.name());
        }
    }
}

// ---------------------------------------------------------------------------
// Sign conventions
// ---------------------------------------------------------------------------

#[test]
fn positive_swing_is_forward_on_both_sides() {
    let s = reference();
    for (hip, ankle) in [(Joint::ThighL, Joint::FootL), (Joint::ThighR, Joint::FootR)] {
        let rest = skeleton::solve(s, &Pose::rest()).origin[ankle.index()];
        let swung =
            skeleton::solve(s, &Pose::rest().set(hip, 30.0, 0.0, 0.0)).origin[ankle.index()];
        assert!(
            swung[2] > rest[2] + 0.15,
            "{}: swinging forward moved the foot to z={:.3} from {:.3}",
            hip.name(),
            swung[2],
            rest[2]
        );
    }
    for (shoulder, wrist) in [(Joint::ArmL, Joint::HandL), (Joint::ArmR, Joint::HandR)] {
        let rest = skeleton::solve(s, &Pose::rest()).origin[wrist.index()];
        let swung =
            skeleton::solve(s, &Pose::rest().set(shoulder, 45.0, 0.0, 0.0)).origin[wrist.index()];
        assert!(
            swung[2] > rest[2] + 0.2,
            "{}: swinging forward moved the hand to z={:.3}",
            shoulder.name(),
            swung[2]
        );
    }
}

#[test]
fn positive_spread_is_away_from_the_body_on_both_sides() {
    // This is the property that makes a symmetric pose a symmetric set of
    // numbers. Without it, every two-handed pose is written twice with a sign
    // flip somewhere in the middle, and half of them are wrong.
    let s = reference();
    let out = Pose::rest().shoulders(0.0, 40.0, 0.0);
    let skin = skeleton::solve(s, &out);
    let rest = rest_skin(s);
    assert!(
        skin.origin[Joint::HandL.index()][0] < rest.origin[Joint::HandL.index()][0] - 0.1,
        "left hand did not go left"
    );
    assert!(
        skin.origin[Joint::HandR.index()][0] > rest.origin[Joint::HandR.index()][0] + 0.1,
        "right hand did not go right"
    );
}

#[test]
fn bending_a_knee_lifts_the_heel_backwards() {
    let s = reference();
    let bent = Pose::rest().knee_l(70.0);
    let skin = skeleton::solve(s, &bent);
    let rest = rest_skin(s);
    let ankle = skin.origin[Joint::FootL.index()];
    let was = rest.origin[Joint::FootL.index()];
    assert!(ankle[1] > was[1] + 0.1, "heel did not lift");
    assert!(ankle[2] < was[2] - 0.1, "heel did not go backwards");
}

#[test]
fn bending_an_elbow_brings_the_hand_forward_and_up() {
    let s = reference();
    let skin = skeleton::solve(s, &Pose::rest().elbow_r(90.0));
    let rest = rest_skin(s);
    let hand = skin.origin[Joint::HandR.index()];
    let was = rest.origin[Joint::HandR.index()];
    assert!(hand[2] > was[2] + 0.15, "hand did not come forward");
    assert!(hand[1] > was[1] + 0.15, "hand did not come up");
}

#[test]
fn mirroring_a_pose_mirrors_the_body() {
    let s = reference();
    let pose = Pose::rest()
        .shoulder_r(-50.0, 20.0, 10.0)
        .elbow_r(80.0)
        .hip_l(25.0, 8.0, 0.0)
        .spine(5.0, 7.0, -20.0)
        .hips(0.06, -0.04, 0.02);
    let a = skeleton::solve(s, &pose);
    let b = skeleton::solve(s, &pose.mirrored());
    for j in JOINTS {
        let p = a.origin[j.index()];
        let q = b.origin[j.opposite().index()];
        let err = (p[0] + q[0]).abs() + (p[1] - q[1]).abs() + (p[2] - q[2]).abs();
        assert!(err < 1e-4, "{} did not mirror: {p:?} vs {q:?}", j.name());
    }
}

#[test]
fn mirroring_twice_is_the_original() {
    let pose = Pose::rest()
        .chest(3.0, -6.0, 25.0)
        .shoulder_l(-30.0, 45.0, -12.0)
        .knee_r(40.0);
    let there_and_back = pose.mirrored().mirrored();
    for i in 0..view::pose::CHANNELS {
        assert!(
            (pose.channels[i] - there_and_back.channels[i]).abs() < 1e-6,
            "channel {i} came back as {}",
            there_and_back.channels[i]
        );
    }
}

// ---------------------------------------------------------------------------
// Joint limits
// ---------------------------------------------------------------------------

#[test]
fn a_knee_cannot_bend_the_wrong_way() {
    // Springs overshoot on purpose. Without a clamp, every overshoot on a leg
    // is one frame of a broken doll, and one frame is enough to see.
    let s = reference();
    let broken = Pose::rest().knee_l(-90.0).elbow_r(-90.0);
    let fixed = broken.clamped(s);
    assert!(fixed.degrees(Joint::ShinL, 0) > -5.0);
    assert!(fixed.degrees(Joint::ForearmR, 0) > -5.0);
    assert!(!broken.violations(s).is_empty(), "limits did not notice");
    assert!(fixed.violations(s).is_empty(), "clamping left a violation");
}

#[test]
fn forward_kinematics_enforces_the_limits_whatever_the_pose_says() {
    // Belt and braces: even a pose that was never clamped cannot draw a broken
    // body, because the clamp is inside the solve as well.
    let s = reference();
    let insane = Pose::rest().set(Joint::ShinR, -400.0, 300.0, 300.0);
    let skin = skeleton::solve(s, &insane);
    let knee = skin.origin[Joint::ShinR.index()];
    let ankle = skin.origin[Joint::FootR.index()];
    assert!(
        ankle[1] < knee[1] + 0.05,
        "the shin bent up through the thigh"
    );
}

// ---------------------------------------------------------------------------
// Inverse kinematics
// ---------------------------------------------------------------------------

#[test]
fn a_planted_foot_lands_where_it_was_asked_to() {
    let s = reference();
    for target in [
        [-0.11, 0.06, 0.30],
        [-0.11, 0.06, -0.25],
        [-0.30, 0.20, 0.10],
        [-0.11, 0.35, 0.35],
    ] {
        let mut pose = Pose::rest().hips(0.0, -0.12, 0.0);
        let miss = ik::foot_to(&mut pose, s, true, target);
        assert!(miss < 0.01, "foot missed {target:?} by {miss:.4} m");
    }
}

#[test]
fn a_foot_stays_put_while_the_hips_move_over_it() {
    // The entire reason the solver exists. A planted foot given the same target
    // on consecutive frames does not move, however much the body does -- which
    // is what "no foot skate" means mechanically.
    let s = reference();
    let target = [-0.11, 0.06, 0.22];
    let mut last: Option<[f32; 3]> = None;
    for i in 0..12 {
        let t = i as f32 / 11.0;
        let mut pose = Pose::rest().hips(0.0, -0.10 - 0.06 * t, -0.06 + 0.22 * t);
        ik::foot_to(&mut pose, s, true, target);
        let ankle = skeleton::solve(s, &pose).origin[Joint::FootL.index()];
        if let Some(prev) = last {
            let slide = math::length(math::sub(ankle, prev));
            assert!(slide < 0.01, "planted foot slid {slide:.4} m");
        }
        last = Some(ankle);
    }
}

#[test]
fn a_target_out_of_reach_gives_a_straight_limb_pointed_at_it() {
    let s = reference();
    let mut pose = Pose::rest();
    ik::foot_to(&mut pose, s, false, [0.11, 0.0, 4.0]);
    assert!(
        pose.degrees(Joint::ShinR, 0) < 12.0,
        "straining leg kept a bent knee: {}",
        pose.degrees(Joint::ShinR, 0)
    );
}

#[test]
fn a_hand_reaches_its_target() {
    let s = reference();
    for target in [[0.35, 1.30, 0.45], [0.20, 1.05, 0.30], [0.45, 1.60, 0.10]] {
        let mut pose = Pose::rest();
        let miss = ik::hand_to(&mut pose, s, false, target);
        assert!(miss < 0.01, "hand missed {target:?} by {miss:.4} m");
    }
}

#[test]
fn the_knee_points_forward_when_the_leg_is_bent_by_the_solver() {
    // The pole is what picks between the two ways a leg can fold, and a knee
    // that folds the wrong way is the most obvious rig failure there is.
    let s = reference();
    let mut pose = Pose::rest();
    ik::foot_to(&mut pose, s, true, [-0.11, 0.10, 0.05]);
    let skin = skeleton::solve(s, &pose);
    let hip = skin.origin[Joint::ThighL.index()];
    let knee = skin.origin[Joint::ShinL.index()];
    let ankle = skin.origin[Joint::FootL.index()];
    let mid = [
        (hip[0] + ankle[0]) * 0.5,
        (hip[1] + ankle[1]) * 0.5,
        (hip[2] + ankle[2]) * 0.5,
    ];
    assert!(
        knee[2] > mid[2] + 0.05,
        "the knee folded backwards: knee z {:.3}, midline {:.3}",
        knee[2],
        mid[2]
    );
}

// ---------------------------------------------------------------------------
// Retargeting
// ---------------------------------------------------------------------------

#[test]
fn one_pose_plays_on_every_build() {
    // The payoff for storing angles rather than positions: a clip authored
    // once is correct on all six bodies, and none of them ends up with a foot
    // through the floor.
    let pose = Pose::rest()
        .hips(0.0, -0.10, 0.05)
        .hip_l(28.0, 4.0, 0.0)
        .knee_l(35.0)
        .hip_r(-18.0, 4.0, 0.0)
        .knee_r(15.0)
        .spine(8.0, 0.0, -10.0);
    for class in sim::class::ALL_CLASSES {
        let s = skeleton::skeleton_for(class);
        let skin = skeleton::solve(&s, &pose);
        for j in JOINTS {
            for k in 0..3 {
                assert!(
                    skin.origin[j.index()][k].is_finite(),
                    "{} on {}: not a number",
                    j.name(),
                    class.name()
                );
            }
        }
        // Angles are proportion-free, so the *shape* should survive: a forward
        // leg stays a forward leg.
        assert!(
            skin.origin[Joint::FootL.index()][2] > skin.origin[Joint::FootR.index()][2],
            "{}: the stride swapped legs",
            class.name()
        );
    }
}

#[test]
fn a_taller_build_is_taller_everywhere() {
    let small = Skeleton::new(Build {
        scale: 0.9,
        ..Build::REFERENCE
    });
    let big = Skeleton::new(Build {
        scale: 1.1,
        ..Build::REFERENCE
    });
    assert!(big.total_height > small.total_height + 0.2);
    assert!(big.leg_reach() > small.leg_reach());
    assert!(big.arm_reach() > small.arm_reach());
}

// ---------------------------------------------------------------------------
// The two frames, and the one thing they must agree about
// ---------------------------------------------------------------------------
//
// The simulation swings a one-armed move from a hand (`sim::aim::hand_origin`)
// and the renderer draws the arm from the skeleton. Those are two different
// pieces of arithmetic in two different crates, and if they disagree about
// which side "left" is on, the Dual mage's dark auto comes out of the arm the
// player can see *not* swinging -- and since which of her two autos landed is
// the whole of how her meter is steered, that is not a cosmetic mistake.
//
// Nothing forces them to agree except this.

/// A fighter's hand, as the renderer puts it in the arena.
fn drawn_hand(pose: &Pose, facing: [f32; 2], left: bool) -> math::V3 {
    let s = reference();
    let joint = if left { Joint::HandL } else { Joint::HandR };
    let local = skeleton::solve(s, pose).origin[joint.index()];
    view::into_world(local, [0.0, 0.0, 0.0], facing)
}

#[test]
fn the_arm_the_simulation_swings_from_is_the_arm_the_renderer_draws() {
    use sim::aim::Hand;
    // Every facing, because the two frames are related by a rotation and a
    // sign error would hide at exactly one angle.
    for eighth in 0..8 {
        let a = eighth as f32 / 8.0 * std::f32::consts::TAU;
        let facing = [a.cos(), a.sin()];
        let sim_facing = sim::V3::new(fx(facing[0]), sim::Fx::ZERO, fx(facing[1]));
        for (hand, left) in [(Hand::Left, true), (Hand::Right, false)] {
            let drawn = drawn_hand(&Pose::rest(), facing, left);
            let swung = sim::aim::across(sim_facing, hand);
            let across = [swung.x.to_f32_for_render(), swung.z.to_f32_for_render()];
            // Sideways only: the simulation's hand is at the height everything
            // is cast from, and the drawn one hangs wherever the pose put it.
            let side = drawn[0] * across[0] + drawn[2] * across[1];
            assert!(
                side > 0.05,
                "at facing {facing:?} the {} hand is drawn {side:.3} m along the side \
                 the simulation swings it from",
                if left { "left" } else { "right" }
            );
        }
    }
}

#[test]
fn a_hand_is_about_as_far_out_as_the_shoulder_it_hangs_from() {
    // The simulation carries one number for every build, because a hit volume
    // that changed size with the model would make the same move a different
    // move on six characters. It still has to be the right *sort* of number:
    // half a metre would put the Dual mage's two autos in different postcodes.
    let out = sim::tuning::hand_offset().to_f32_for_render();
    for class in sim::class::ALL_CLASSES {
        let s = skeleton::skeleton_for(class);
        let shoulder = skeleton::solve(&s, &Pose::rest()).origin[Joint::ArmR.index()][0];
        assert!(
            (shoulder - out).abs() < 0.08,
            "{}: shoulders at {shoulder:.3} m, the simulation swings from {out:.3} m",
            class.name()
        );
    }
}

#[test]
fn the_wing_is_sized_off_the_arm_that_throws_it() {
    // The Dual mage's autos are punches that throw a section of a torus around
    // her -- the shape is `moves::Shape::Wing` and the fantasy is in
    // `docs/design/kits/dual-mage.md`. Two numbers in it are stated against her
    // own body rather than in metres, and this is the only place they can be
    // checked, because the simulation has no idea where an elbow is:
    //
    //   the inner arc passes through where the punching elbow starts
    //   the outer arc is two to three times further out than the hand finishes,
    //   measured from that elbow
    //
    // Read off the baked clip, so retiming the punch or re-authoring the pose
    // moves the check with it.
    let s = skeleton::skeleton_for(sim::Class::DualMage);
    let clip = view::Clip::DualDark;
    let (last_startup, _, first_recovery) = clip.phases().expect("an attack clip");
    let out = |frame: u16, joint: Joint| {
        let skin = skeleton::solve(&s, &clip.at(frame as u32));
        let p = skin.origin[joint.index()];
        (p[0] * p[0] + p[2] * p[2]).sqrt()
    };
    // The cock, and the end of the punch: where the elbow is drawn back to, and
    // how far the fist gets. The furthest the hand reaches across the active
    // window rather than its position on one frame -- "where their hand
    // finishes" is the end of the extension, and which frame that lands on is
    // the animator's business.
    let elbow = out(1, Joint::ForearmL);
    let hand = (last_startup..=first_recovery)
        .map(|f| out(f, Joint::HandL))
        .fold(0.0f32, f32::max);
    assert!(
        hand > elbow + 0.2,
        "the punch does not travel: elbow {elbow:.2} m, hand {hand:.2} m"
    );

    let m = sim::moves::get(sim::Class::DualMage, sim::moves::dual::DARK_AUTO);
    let inner = m.reach.to_f32_for_render() * sim::tuning::wing_inner().to_f32_for_render();
    assert!(
        (inner - elbow).abs() < 0.1,
        "the inner arc is at {inner:.2} m and the elbow starts at {elbow:.2} m"
    );

    let travel = hand - elbow;
    let times = (m.reach.to_f32_for_render() - elbow) / travel;
    assert!(
        (2.0..=3.0).contains(&times),
        "the outer arc is {times:.1} times the punch's own travel past the elbow \
         (elbow {elbow:.2} m, hand {hand:.2} m, reach {:.2} m)",
        m.reach.to_f32_for_render()
    );
}

#[test]
fn both_autos_are_sized_the_same() {
    // The light one is the dark one mirrored, and the check above only looks at
    // the dark one.
    let dark = sim::moves::get(sim::Class::DualMage, sim::moves::dual::DARK_AUTO);
    let light = sim::moves::get(sim::Class::DualMage, sim::moves::dual::LIGHT_AUTO);
    assert_eq!(dark.reach.raw(), light.reach.raw());
    assert_eq!(dark.arc.raw(), light.arc.raw());
}

fn fx(v: f32) -> sim::Fx {
    sim::Fx::from_raw((v * 65536.0).round() as i32)
}
