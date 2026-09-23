//! The wings are the bars.
//!
//! The Dual mage's two bars are drawn as three wings a side, and the rule that
//! makes that worth anything is the overlay's own: what is drawn is what the
//! simulation holds. A wing count that could drift from its bar would be a
//! meter that lies to the one player who cannot see the HUD -- the opponent.

use sim::class::{Class, Force, Mechanic};
use sim::{Fx, World};
use view::wings::{
    self, COVERTS, FEATHERS, ORDER, PER_SIDE, PRIMARIES, Slot, WRIST, feather_outline,
};

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
fn a_side_shows_one_wing_per_tier_its_bar_has_reached() {
    // A wing is a tier. The first arrives with the blink's threshold, the
    // second with the second jump's, the third at the top the wings read --
    // the marks the HUD already ticks -- so a wing appearing and something
    // changing about her body are one moment. Five bar values including empty,
    // full and lopsided, the way the plan asks. A count, not a length.
    let blink = sim::tuning::tier_blink();
    let jump = sim::tuning::tier_jump();
    let full = sim::tuning::tier_wings();
    for (dark, light, want_dark, want_light) in [
        (0, 0, 0, 0),
        (100, 100, 3, 3),
        (blink, blink - 1, 1, 0),
        (jump, full, 2, 3),
        (blink - 1, jump - 1, 0, 1),
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
    // And the same three numbers the tiers read: a mage level at each
    // threshold holds that tier and shows that many wings a side.
    for (bar, tier, count) in [
        (blink, sim::dual::Tier::Blink, 1),
        (jump, sim::dual::Tier::Jump, 2),
        (full, sim::dual::Tier::Wings, 3),
    ] {
        let w = mage_with(bar, bar, 0);
        assert_eq!(sim::dual::tier(&w.players[0]), tier);
        assert_eq!(shown(&w, Force::Dark).len(), count);
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
    let (blink, jump) = (sim::tuning::tier_blink(), sim::tuning::tier_jump());
    assert_eq!(
        shown(&mage_with(blink, 0, 0), Force::Dark),
        vec![Slot::Middle]
    );
    assert_eq!(
        shown(&mage_with(jump, 0, 0), Force::Dark),
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
    // The same wing at half a bar and at the top: same root, same direction,
    // same length. What changes with the bar is how many.
    let low = mage_with(sim::tuning::tier_blink(), sim::tuning::tier_blink(), 0);
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
            Slot::Top => assert!(wing.along[1] > 0.4, "the top wing does not reach up"),
            Slot::Bottom => assert!(wing.along[1] < -0.2, "the bottom wing does not reach down"),
            Slot::Middle => assert!(wing.along[1] > 0.05 && wing.along[1] < 0.5),
        }
    }
}

#[test]
fn the_wing_is_an_arm_with_primaries_fanning_from_the_wrist() {
    // What tells a bird's wing from an insect's: the leading edge rises from
    // the shoulder to a wrist well short of the tip, the primaries fan from
    // that wrist like fingers -- the outermost reaching the full span, each
    // next one shorter and hung lower, the last hanging down -- and the
    // secondaries hang from the arm behind them, overlapping.
    let wrist = FEATHERS[0].base;
    assert!(
        wrist[1] > 0.2 && wrist[0] < 0.5,
        "the wrist is not up and short of the tip"
    );
    let primaries = &FEATHERS[..PRIMARIES];
    let tip = |f: &view::wings::Feather| {
        [
            f.base[0] + f.length * f.angle.cos(),
            f.base[1] + f.length * f.angle.sin(),
        ]
    };
    assert!(
        primaries.iter().all(|f| f.base == WRIST),
        "a primary does not root at the wrist"
    );
    let reach = primaries.iter().map(|f| tip(f)[0]).fold(0.0f32, f32::max);
    assert!(
        (reach - 1.0).abs() < 0.05,
        "the outermost primary reaches {reach:.2} of the span"
    );
    for pair in primaries.windows(2) {
        assert!(
            pair[1].angle < pair[0].angle,
            "the primaries do not fan downward in order"
        );
        assert!(
            pair[1].length < pair[0].length,
            "the primaries do not shorten inward"
        );
    }
    assert!(
        primaries.last().unwrap().angle < -1.5,
        "the last primary does not hang down"
    );
    let secondaries = &FEATHERS[PRIMARIES..];
    assert!(secondaries.len() >= 4);
    for f in secondaries {
        assert!(f.angle < -1.6, "a secondary does not hang down");
        assert!(
            f.base[0] < WRIST[0] && f.base[1] <= WRIST[1],
            "a secondary roots past the wrist"
        );
    }
    // Every part is convex, so the renderer's fan from its middle cannot fold.
    let convex = |pts: &[[f32; 2]]| {
        let n = pts.len();
        let turn = |i: usize| {
            let (a, b, c) = (pts[i], pts[(i + 1) % n], pts[(i + 2) % n]);
            (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0])
        };
        (0..n).all(|i| turn(i) >= -1e-6) || (0..n).all(|i| turn(i) <= 1e-6)
    };
    assert!(convex(&COVERTS), "the coverts are not convex");
    for f in FEATHERS.iter() {
        assert!(
            convex(&feather_outline(f)),
            "a feather's outline is not convex"
        );
    }
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
