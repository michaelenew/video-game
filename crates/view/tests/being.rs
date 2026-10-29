//! The being is the bars.
//!
//! The Dual mage's two bars are drawn as one incorporeal body taking shape
//! behind her, and the rule that makes that worth anything is the overlay's
//! own: what is drawn is what the simulation holds. A half that could drift
//! from its bar would be a meter that lies to the one player who cannot see
//! the HUD -- the opponent.

use sim::class::{Class, Force, Mechanic};
use sim::{Fx, World};
use view::being::{self, ORDER, presence};
use view::skeleton::{Joint, skeleton_for, solve};

fn mage_with(dark: i32, light: i32, ascending: u16) -> World {
    let mut w = World::with_classes([Class::DualMage, Class::Bulwark]);
    w.players[0].mechanic = Mechanic::Meter {
        dark: Fx::from_int(dark),
        light: Fx::from_int(light),
        colour: Force::Dark,
        ascending,
        jumped: false,
        hopped: false,
        dash_jump: false,
        landed: 0,
        singe: Fx::ZERO,
        veil: 0,
        warded: false,
    };
    w
}

fn of(w: &World) -> being::Being {
    being::being(&w.players[0]).expect("the Dual mage has a being")
}

#[test]
fn each_half_is_its_own_bar_and_a_gradient_not_a_count() {
    // Empty is nothing; the top tier is all of it; and in between it
    // thickens with every point of bar, not at the tiers.
    let full = sim::tuning::tier_wings();
    assert_eq!(of(&mage_with(0, 0, 0)).dark, 0.0);
    assert!(!of(&mage_with(0, 0, 0)).shown());
    assert_eq!(of(&mage_with(full, full, 0)).dark, 1.0);
    assert_eq!(of(&mage_with(100, 100, 0)).light, 1.0);
    let mut last = -1.0;
    for bar in 0..=full {
        let b = of(&mage_with(bar, 0, 0));
        assert!(
            b.dark > last,
            "a dark bar of {bar} shows no more than {}",
            bar - 1
        );
        assert_eq!(b.light, 0.0, "the light half is fed by the dark bar");
        last = b.dark;
    }
    // Lopsided is lopsided, on the right sides.
    let b = of(&mage_with(full / 4, full * 3 / 4, 0));
    assert!(b.light > 2.5 * b.dark);
    assert_eq!(b.of(Force::Dark), b.dark);
    assert_eq!(being::hand_of(Force::Dark), sim::aim::Hand::Left);
    assert_eq!(being::hand_of(Force::Light), sim::aim::Hand::Right);
}

#[test]
fn ascension_is_all_of_it() {
    let b = of(&mage_with(0, 40, 300));
    assert!(b.ascending);
    assert_eq!((b.dark, b.light), (1.0, 1.0));
}

#[test]
fn the_shoulder_and_fist_first_then_the_arm_and_torso_then_the_head() {
    // `presence(reach, head, fill)`: reach 0 is at the shoulder or the
    // fist, 1 is half an arm from both (the middle of the arm, the torso).
    let shoulder = |fill| presence(0.0, 0.0, fill);
    let arm = |fill| presence(0.6, 0.0, fill);
    let torso = |fill| presence(1.0, 0.0, fill);
    let head = |fill| presence(1.0, 1.0, fill);
    // Nothing of it with nothing in the bar.
    for part in [shoulder, arm, torso, head] {
        assert_eq!(part(0.0), 0.0);
        assert_eq!(part(1.0), 1.0, "a full bar is the whole of it");
    }
    // At every fill on the way, each part is at least as there as the next.
    for i in 1..100 {
        let f = i as f32 / 100.0;
        assert!(shoulder(f) >= arm(f) && arm(f) >= torso(f) && torso(f) >= head(f));
    }
    // A quarter of a bar: the shoulder and fist are showing and the head
    // is not.
    assert!(shoulder(0.25) > 0.5);
    assert_eq!(head(0.25), 0.0);
    // The head has not started until the torso has.
    assert_eq!(head(ORDER.body + ORDER.soft * 0.5), 0.0);
    assert!(torso(ORDER.body + ORDER.soft * 0.5) > 0.0);
    // And it fades in rather than switching on: never a step bigger than a
    // tenth of the part for a hundredth of the bar.
    for part in [shoulder, arm, torso, head] {
        for i in 0..100 {
            let (a, b) = (i as f32 / 100.0, (i + 1) as f32 / 100.0);
            assert!((part(b) - part(a)).abs() < 0.1, "a step at fill {a}");
        }
    }
}

#[test]
fn it_is_twice_her_size_with_its_shoulders_at_hers() {
    let her = skeleton_for(Class::DualMage);
    let shoulder = |sk: &view::skeleton::Skeleton| {
        solve(sk, &view::pose::Pose::rest()).origin[view::drawn_joint(Joint::ArmL).index()][1]
    };
    for k in 0..2 {
        let skeleton = being::skeleton(k);
        let s = being::scale(&her, &skeleton);
        // Twice her size, sunk below her floor so its shoulders are at hers.
        let tall = s * skeleton.total_height;
        assert!(
            (tall / her.total_height - 2.0).abs() < 0.01,
            "being {k} is {tall} tall to her {}",
            her.total_height
        );
        let at = s * shoulder(&skeleton) - being::sink(&her, &skeleton);
        assert!(being::sink(&her, &skeleton) > 0.5, "being {k} is not sunk");
        assert!(
            (at - shoulder(&her)).abs() < 0.005,
            "being {k}'s shoulder at {at}, hers at {}",
            shoulder(&her)
        );
    }
    // The seal at her feet, where they come out; at the waist, the other
    // place it was tried, under her shoulders.
    assert_eq!(being::SEAL_AT, being::SealAt::Feet);
    let seal = being::seal_height(&her, being::SealAt::Waist);
    assert!(seal > her.hip_height * 0.9 && seal < shoulder(&her));
    assert_eq!(being::seal_height(&her, being::SealAt::Feet), 0.0);
}

/// Being `k`'s striking hand -- the dark one's left, the light one's right --
/// and its shoulder, posed.
fn hand_and_shoulder(k: usize, pose: &view::pose::Pose) -> ([f32; 3], [f32; 3], f32) {
    let skeleton = being::skeleton(k);
    let solved = solve(&skeleton, pose);
    let left = k == 0;
    let hand = solved.origin[view::hand_joint(left).index()];
    let shoulder =
        solved.origin[view::drawn_joint(if left { Joint::ArmL } else { Joint::ArmR }).index()];
    (hand, shoulder, skeleton.hip_height)
}

fn auto_moment(kind: u8, at: u16) -> being::Moment {
    being::Moment {
        kind: Some(kind),
        at,
        frames: sim::moves::frames(Class::DualMage, kind),
        clock: 0,
    }
}

#[test]
fn it_strains_at_rest_and_its_auto_is_its_own_blow() {
    for k in 0..2 {
        // At rest, between pulls (the world's first frame), its arm hangs.
        let (hand, shoulder, hips) = hand_and_shoulder(k, &being::strain(k, 0));
        assert!(hand[1] < hips + 0.15, "being {k}: a fist up at {}", hand[1]);
        assert!(
            hand[0].abs() > shoulder[0].abs() * 0.8,
            "being {k}: an arm against the body"
        );
        // Every frame of the world it is somewhere a little different: it
        // breathes and pulls, it does not stand still.
        let moved = (0..600)
            .map(|c| being::strain(k, c).separation(&being::strain(k, 0)))
            .fold(0.0f32, f32::max);
        // Summed over every joint, in radians: thirty degrees of pull at
        // least -- the knight, which only tests its binding.
        assert!(moved > 0.5, "being {k} stands still");
    }
    // Its side's auto: drawn back behind its shoulder before the ball leaves,
    // then driven out in front of it, at its shoulder's height.
    for (k, kind) in [(0, being::DARK_AUTO), (1, being::LIGHT_AUTO)] {
        let (s, _, _) = sim::moves::frames(Class::DualMage, kind);
        let wound = being::pose(k, &auto_moment(kind, s - 1));
        let (hand, shoulder, _) = hand_and_shoulder(k, &wound);
        assert!(hand[2] < shoulder[2], "being {k}: not drawn back");
        let landed = being::pose(k, &auto_moment(kind, s + 2));
        let (hand, shoulder, _) = hand_and_shoulder(k, &landed);
        assert!(
            hand[2] > shoulder[2] + 0.4,
            "being {k}: the blow is not out in front"
        );
        // On its own side of its middle: the half across it is not drawn.
        assert!(
            hand[0] * shoulder[0] > 0.0 && hand[0].abs() > shoulder[0].abs() * 0.6,
            "being {k}: the blow crosses its body, the fist at {} with the shoulder at {}",
            hand[0],
            shoulder[0]
        );
        assert!(
            (hand[1] - shoulder[1]).abs() < 0.25,
            "being {k}: the blow lands at {} with the shoulder at {}",
            hand[1],
            shoulder[1]
        );
        // The other side's auto is not its blow.
        let other = if kind == being::DARK_AUTO {
            being::LIGHT_AUTO
        } else {
            being::DARK_AUTO
        };
        let (hand, shoulder, _) = hand_and_shoulder(k, &being::pose(k, &auto_moment(other, s + 2)));
        assert!(
            hand[2] < shoulder[2] + 0.4,
            "being {k} throws the other side's blow"
        );
    }
}

#[test]
fn only_the_class_with_bars_has_a_being() {
    for class in sim::class::ALL_CLASSES {
        let w = World::with_classes([class, Class::Bulwark]);
        assert_eq!(
            being::being(&w.players[0]).is_some(),
            class == Class::DualMage,
            "{} disagrees about having a being",
            class.name()
        );
    }
}
