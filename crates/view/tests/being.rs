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
fn its_hips_are_at_her_feet_and_its_chin_at_her_forehead() {
    let skeleton = skeleton_for(Class::DualMage);
    let s = being::scale(&skeleton);
    let rest = solve(&skeleton, &view::pose::Pose::rest());
    let chin = rest.origin[Joint::Head.index()][1];
    // Placed with its hips on the floor she stands on, its chin is at her
    // forehead: within a few centimetres of nine tenths of her height.
    let at = s * (chin - skeleton.hip_height);
    let forehead = being::CHIN_AT * skeleton.total_height;
    assert!(
        (at - forehead).abs() < 0.01,
        "chin at {at}, forehead at {forehead}"
    );
    assert!(
        at > rest.origin[Joint::Neck.index()][1],
        "chin under her neck"
    );
    assert!(at < skeleton.total_height, "chin over her head");
    // Big: more than twice her size, and the seal round it wider than she is.
    assert!(s > 2.0, "the being is only {s} of her");
    assert!(being::seal_radius(&skeleton) > 1.0);
}

#[test]
fn its_fists_are_up_in_front_of_her_and_her_throws_still_reach() {
    // Placed as it is drawn -- hips at her feet, `scale` times her size -- a
    // being holding her standing pose has both fists above the floor and in
    // front of its chest, where the first of it can be seen.
    let skeleton = skeleton_for(Class::DualMage);
    let s = being::scale(&skeleton);
    let skin = solve(&skeleton, &being::guarded(&view::pose::Pose::rest()));
    for left in [true, false] {
        let hand = skin.origin[view::hand_joint(left).index()];
        let height = s * (hand[1] - skeleton.hip_height);
        assert!(height > 0.6, "a fist is {height} m off the floor");
        assert!(hand[2] > 0.2, "a fist is not in front of it");
    }
    // A throw that reaches higher than the guard is the being's throw too.
    let mut reach = view::pose::Pose::rest()
        .shoulder_l(150.0, 0.0, 0.0)
        .elbow_l(5.0);
    reach = being::guarded(&reach);
    assert!((reach.degrees(Joint::ArmL, 0) - 150.0).abs() < 1e-3);
    assert!((reach.degrees(Joint::ForearmL, 0) - being::GUARD_ELBOW).abs() < 1e-3);
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
