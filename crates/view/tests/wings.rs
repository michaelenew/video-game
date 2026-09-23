//! The wings are the bars.
//!
//! The Dual mage's two bars are drawn as two wings on her back, and the rule
//! that makes that worth anything is the overlay's own: what is drawn is what
//! the simulation holds. A wing that could drift from its bar would be a meter
//! that lies to the one player who cannot see the HUD -- the opponent.

use sim::class::{Class, Force, Mechanic};
use sim::{Fx, World};
use view::wings::{self, FULL_SPAN};

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

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[test]
fn each_wing_is_exactly_as_long_as_its_bar() {
    // Five bar values, including empty, full and lopsided, the way the plan
    // asks. The span is the bar's share of the top times the full span, and the
    // root-to-tip distance is that span -- both, because a wing whose declared
    // span disagreed with where its tip was drawn would pass the first and lie
    // on screen.
    let top = sim::tuning::meter_max() as f32;
    for (dark, light) in [(0, 0), (100, 100), (50, 20), (70, 100), (33, 33)] {
        let w = mage_with(dark, light, 0);
        let [d, l] = wings::wings(&w.players[0]).expect("the Dual mage has wings");
        for (wing, bar) in [(d, dark), (l, light)] {
            let want = bar as f32 / top * FULL_SPAN;
            assert!(
                (wing.span - want).abs() < 0.001,
                "{:?} wing spans {:.3} m for a bar of {bar}; the bar says {want:.3}",
                wing.force,
                wing.span
            );
            assert!(
                (dist(wing.root, wing.tip) - want).abs() < 0.001,
                "{:?} wing is drawn {:.3} m long and declares {:.3}",
                wing.force,
                dist(wing.root, wing.tip),
                wing.span
            );
        }
    }
}

#[test]
fn ascending_is_full_span_whatever_the_bars_say() {
    let w = mage_with(0, 0, 90);
    let [d, l] = wings::wings(&w.players[0]).unwrap();
    assert!((d.span - FULL_SPAN).abs() < 0.001 && (l.span - FULL_SPAN).abs() < 0.001);
}

#[test]
fn dark_is_on_her_left_and_light_on_her_right() {
    // The same sides as the arms that goad them, read off the simulation's own
    // idea of which side is which -- `aim::across` -- so the wing and the auto
    // can never disagree about where the dark being lives.
    let w = mage_with(60, 60, 0);
    let p = &w.players[0];
    let [d, l] = wings::wings(p).unwrap();
    for (wing, hand) in [(d, sim::aim::Hand::Left), (l, sim::aim::Hand::Right)] {
        let across = sim::aim::across(p.facing, hand);
        let out = [
            wing.tip[0] - wing.root[0],
            wing.tip[1] - wing.root[1],
            wing.tip[2] - wing.root[2],
        ];
        let along = out[0] * across.x.to_f32_for_render() + out[2] * across.z.to_f32_for_render();
        assert!(
            along > 0.5,
            "the {:?} wing reaches {along:+.2} m along its own arm's side",
            wing.force
        );
        assert!(out[1] > 0.0, "the {:?} wing droops", wing.force);
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
