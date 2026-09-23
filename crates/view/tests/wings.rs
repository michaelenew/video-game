//! The wings are the bars.
//!
//! The Dual mage's two bars are drawn as three wings a side, and the rule that
//! makes that worth anything is the overlay's own: what is drawn is what the
//! simulation holds. A wing count that could drift from its bar would be a
//! meter that lies to the one player who cannot see the HUD -- the opponent.

use sim::class::{Class, Force, Mechanic};
use sim::{Fx, World};
use view::wings::{self, ORDER, OUTLINE, OUTLINE_CENTRE, PER_SIDE, Slot};

fn mage_with(dark: i32, light: i32, ascending: u16) -> World {
    let mut w = World::with_classes([Class::DualMage, Class::Bulwark]);
    w.players[0].mechanic = Mechanic::Meter {
        dark: Fx::from_int(dark),
        light: Fx::from_int(light),
        colour: Force::Dark,
        ascending,
        jumped: false,
        landed: 0,
        singe: Fx::ZERO,
    };
    w
}

fn shown(w: &World, force: Force) -> Vec<Slot> {
    wings::wings(&w.players[0])
        .expect("the Dual mage has wings")
        .iter()
        .filter(|wing| wing.force == force && wing.shown)
        .map(|wing| wing.slot)
        .collect()
}

#[test]
fn a_side_shows_one_wing_per_third_of_its_bar() {
    // Five bar values including empty, full and lopsided, the way the plan
    // asks. A count, not a length: a wing is there or it is not, and the third
    // appears at the same "full" that ascension reads.
    let full = sim::tuning::tier_wings();
    for (dark, light, want_dark, want_light) in [
        (0, 0, 0, 0),
        (100, 100, 3, 3),
        (50, 20, 1, 0),
        (70, full, 2, 3),
        (33, 34, 0, 1),
    ] {
        let w = mage_with(dark, light, 0);
        assert_eq!(
            shown(&w, Force::Dark).len(),
            want_dark,
            "dark bar {dark} shows the wrong number of wings"
        );
        assert_eq!(
            shown(&w, Force::Light).len(),
            want_light,
            "light bar {light} shows the wrong number of wings"
        );
    }
}

#[test]
fn the_biggest_comes_first_then_the_lower_then_the_small_one_on_top() {
    // From the top of her back down: smallest, biggest, middle. And they
    // arrive biggest first, so a third of a bar already reads across the
    // arena.
    assert!(Slot::Middle.length() > Slot::Bottom.length());
    assert!(Slot::Bottom.length() > Slot::Top.length());
    assert_eq!(ORDER, [Slot::Middle, Slot::Bottom, Slot::Top]);
    assert_eq!(shown(&mage_with(34, 0, 0), Force::Dark), vec![Slot::Middle]);
    assert_eq!(
        shown(&mage_with(67, 0, 0), Force::Dark),
        vec![Slot::Middle, Slot::Bottom]
    );
    let all = wings::wings(&mage_with(100, 100, 0).players[0]).unwrap();
    let height = |slot: Slot| {
        all.iter()
            .find(|w| w.force == Force::Dark && w.slot == slot)
            .unwrap()
            .root[1]
    };
    assert!(height(Slot::Top) > height(Slot::Middle));
    assert!(height(Slot::Middle) > height(Slot::Bottom));
}

#[test]
fn the_wings_materialise_and_never_grow() {
    // The same wing at a third of the bar and at the top: same root, same
    // direction, same length. What changes with the bar is how many.
    let low = mage_with(34, 34, 0);
    let high = mage_with(100, 100, 0);
    let a = wings::wings(&low.players[0]).unwrap();
    let b = wings::wings(&high.players[0]).unwrap();
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!((x.force, x.slot), (y.force, y.slot));
        assert_eq!(x.root, y.root, "a wing moved its root as the bar rose");
        assert_eq!(x.along, y.along, "a wing turned as the bar rose");
        assert_eq!(x.length, y.length, "a wing grew as the bar rose");
    }
    assert!(a[0].shown && !a[1].shown && b[2].shown);
}

#[test]
fn ascending_is_all_six_whatever_the_bars_say() {
    let w = mage_with(0, 0, 90);
    assert_eq!(shown(&w, Force::Dark).len(), PER_SIDE);
    assert_eq!(shown(&w, Force::Light).len(), PER_SIDE);
}

#[test]
fn dark_is_on_her_left_and_light_on_her_right_and_the_vanes_face_the_players() {
    // The same sides as the arms that goad them, read off the simulation's own
    // idea of which side is which -- `aim::across` -- so the wing and the auto
    // can never disagree about where the dark being lives. The three fan out:
    // the top pair reaches up, the bottom pair down. And each vane stands in
    // the vertical plane through its own length, so its face looks forward and
    // back rather than lying flat like a shelf.
    let w = mage_with(100, 100, 0);
    let p = &w.players[0];
    for wing in wings::wings(p).unwrap() {
        let across = sim::aim::across(p.facing, wings::hand_of(wing.force));
        let along = wing.along[0] * across.x.to_f32_for_render()
            + wing.along[2] * across.z.to_f32_for_render();
        assert!(
            along > 0.25,
            "the {:?} {:?} wing reaches {along:+.2} along its own arm's side",
            wing.force,
            wing.slot
        );
        assert!(
            wing.across[1] > 0.2,
            "the {:?} {:?} wing's chord does not run up: {:?}",
            wing.force,
            wing.slot,
            wing.across
        );
        let dot = wing.along[0] * wing.across[0]
            + wing.along[1] * wing.across[1]
            + wing.along[2] * wing.across[2];
        assert!(dot.abs() < 0.01, "a wing's two axes are not square");
        // The face normal is horizontal: forward or back, never up.
        let n = [
            wing.along[1] * wing.across[2] - wing.along[2] * wing.across[1],
            wing.along[2] * wing.across[0] - wing.along[0] * wing.across[2],
            wing.along[0] * wing.across[1] - wing.along[1] * wing.across[0],
        ];
        assert!(
            n[1].abs() < 0.05,
            "the {:?} {:?} wing lies flat: its face looks {:+.2} up",
            wing.force,
            wing.slot,
            n[1]
        );
        match wing.slot {
            Slot::Top => assert!(wing.along[1] > 0.5, "the top wing does not reach up"),
            Slot::Bottom => assert!(wing.along[1] < -0.2, "the bottom wing does not reach down"),
            Slot::Middle => assert!(wing.along[1] > 0.2 && wing.along[1] < 0.7),
        }
    }
}

#[test]
fn the_silhouette_is_a_wing_and_fills_from_its_centre() {
    // A real outline: longer than it is deep, its tip at full length, its
    // trailing edge cut into feathers -- and star-shaped about the centre the
    // renderer fans it from, or the fan would fold over itself.
    let tip = OUTLINE.iter().map(|p| p[0]).fold(0.0f32, f32::max);
    assert!(
        (tip - 1.0).abs() < 1e-6,
        "the tip is not at the wing's length"
    );
    let depth = OUTLINE.iter().map(|p| p[1]).fold(f32::MIN, f32::max)
        - OUTLINE.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
    assert!(
        depth > 0.5 && depth < 1.0,
        "the wing is {depth:.2} deep for 1.0 long: a paddle, or a blade"
    );
    let notches = OUTLINE[5..]
        .windows(3)
        .filter(|w| w[1][1] > w[0][1] && w[1][1] > w[2][1])
        .count();
    assert!(notches >= 3, "only {notches} feathers on the trailing edge");
    // Star-shaped: every edge's winding about the centre has the same sign.
    let (cx, cy) = (OUTLINE_CENTRE[0], OUTLINE_CENTRE[1]);
    let mut signs = OUTLINE
        .iter()
        .zip(OUTLINE.iter().cycle().skip(1))
        .map(|(a, b)| ((a[0] - cx) * (b[1] - cy) - (a[1] - cy) * (b[0] - cx)).signum());
    let first = signs.next().unwrap();
    assert!(
        signs.all(|s| s == first),
        "the outline is not star-shaped about its centre; a fan from there folds"
    );
}

#[test]
fn only_the_class_with_bars_has_wings() {
    for class in sim::class::ALL_CLASSES {
        let w = World::with_classes([class, Class::Bulwark]);
        assert_eq!(
            wings::wings(&w.players[0]).is_some(),
            class == Class::DualMage,
            "{} disagrees about having wings",
            class.name()
        );
    }
}
