//! The class layer (`hunt::class`): the scripted hunter plays every class's
//! own kit against a creature, the plan's fight is the Champion's own, and
//! the Dual mage's hands keep her out of the ascension a hunt cannot pay for.

use hunt::{Hands, Uses};
use sim::fixed::Fx;
use sim::species::SpeciesId;
use sim::state::MAX_PLAYERS;
use sim::{Class, Input, V3, World};

#[test]
fn the_champion_is_played_exactly_as_the_plan_says() {
    // The plans were written for him, and the Ridgeback's pin hashes his
    // hunts: the class layer must hand back whatever the plan built.
    let w = World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::RIDGEBACK);
    let me = w.players[0];
    let mut hands = Hands::new(0, 7);
    let at = me.pos.add(V3::new(Fx::from_int(2), Fx::ONE, Fx::ZERO));
    let swing = Input::looking_at(Input::LEFT | Input::W, 1234, -300);
    assert_eq!(hands.hit(&w, &me, at, swing, Some(200)), swing);
    assert_eq!(hands.idle(&w, &me, at, at, i32::MAX), None);
    assert_eq!(
        hands.close_in(
            &w,
            &me,
            at.add(V3::new(Fx::from_int(6), Fx::ZERO, Fx::ZERO)),
            200
        ),
        None
    );
    let dodge = Input::aimed(Input::SHIFT | Input::D, 1234);
    let out = V3::new(Fx::ZERO, Fx::ZERO, Fx::ONE);
    assert_eq!(hands.leave(&w, &me, out, dodge), dodge);
    assert_eq!(hands.guard(&me, true, at, 20), None);
    for input in [swing, dodge, Input::default(), Input::new(Input::SPACE)] {
        assert_eq!(hands.finish(&w, &me, input), input);
    }
    assert_eq!(hands.uses, Uses::default());
}

/// A few short hunts of one class against one creature, and what its hands
/// did over them.
fn uses(class: Class, species: SpeciesId, frames: u32) -> Uses {
    let mut all = Uses::default();
    for seed in [0x2545_F491u32, 7] {
        let report = hunt::play_species(species, [class; MAX_PLAYERS], 1, frames, seed);
        all.add(&report.uses);
    }
    all
}

#[test]
fn every_class_plays_its_own_kit_against_a_creature() {
    // What the creature documents asked the harness for and it could not
    // give: the Reaver's shadow, the Elementalist's fire and shots, the
    // Blood mage's pools, the Dual mage's two hands, the Bulwark's guard.
    let reaver = uses(Class::ShadowReaver, SpeciesId::HORNBACK, 6_000);
    assert!(
        reaver.sent > 0 && reaver.copies > 0 && reaver.lotuses + reaver.recalls > 0,
        "the Reaver never worked her shadow: {reaver:?}"
    );
    let caster = uses(Class::Elementalist, SpeciesId::RIDGEBACK, 6_000);
    assert!(
        caster.pillars > 0 && caster.shots > 0,
        "the Elementalist never lit a pillar or shot from range: {caster:?}"
    );
    let blood = uses(Class::BloodMage, SpeciesId::SANDMAW, 6_000);
    assert!(
        blood.blinks > 0,
        "the Blood mage never blinked to a pool: {blood:?}"
    );
    // Since 2026-10-09 every spell of hers has its own force, and the plans
    // already throw the low side's auto, so the layer seldom has to turn a
    // hand: what shows it working is the goading between openings and the
    // majors thrown into the windows (`the_dual_mage_never_ascends_on_a_hunt`
    // holds the other half -- that it stops short of the wings).
    let dual = uses(Class::DualMage, SpeciesId::RIDGEBACK, 6_000);
    assert!(
        dual.goads > 0 && dual.finishers > 0,
        "the Dual mage never goaded her bars or threw a major: {dual:?}"
    );
    let bulwark = uses(Class::Bulwark, SpeciesId::HORNBACK, 6_000);
    assert!(
        bulwark.guards > 0,
        "the Bulwark never took a blow on the shield: {bulwark:?}"
    );
    // And his throw: the shield thrown at a window, the leap after it, and
    // the Slam out of the leap.
    let thrower = uses(Class::Bulwark, SpeciesId::PAIR, 6_000);
    assert!(
        thrower.throws > 0 && thrower.leaps > 0 && thrower.leap_slams > 0,
        "the Bulwark never threw and leapt to his shield: {thrower:?}"
    );
}

#[test]
fn the_bulwarks_shield_comes_home() {
    // Every Bulwark move but the throw needs the shield in his hand, so a
    // throw given up -- a dodge the plan asked for in the middle of it -- is
    // recalled rather than left planted for the rest of the hunt.
    let mut planted_for = 0u32;
    let mut longest = 0u32;
    hunt::play_species_watched(
        SpeciesId::RIDGEBACK,
        [Class::Bulwark; MAX_PLAYERS],
        1,
        6_000,
        101,
        |w| {
            if matches!(
                w.players[0].shield(),
                Some(sim::class::Shield::Planted { .. })
            ) {
                planted_for += 1;
                longest = longest.max(planted_for);
            } else {
                planted_for = 0;
            }
        },
    );
    assert!(
        longest < 120,
        "his shield lay planted for {longest} frames in a row"
    );
}

#[test]
fn the_dual_mage_never_ascends_on_a_hunt() {
    // Six seconds of wings drain her two a frame and pay back only in hits;
    // the first hunts that let her climb there lost seven hundred health to
    // it on the Mireback. Her hands stop short of both bars full.
    for species in [SpeciesId::RIDGEBACK, SpeciesId::GNAWERS] {
        let mut ascended = 0u32;
        hunt::play_species_watched(species, [Class::DualMage; MAX_PLAYERS], 1, 6_000, 3, |w| {
            if sim::dual::ascending(&w.players[0]) {
                ascended += 1;
            }
        });
        assert_eq!(ascended, 0, "she ascended against {species:?}");
    }
}
