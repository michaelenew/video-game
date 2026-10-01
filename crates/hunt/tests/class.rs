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
    let dual = uses(Class::DualMage, SpeciesId::RIDGEBACK, 6_000);
    assert!(
        dual.turned > 0 && dual.goads > 0,
        "the Dual mage never kept her hands level: {dual:?}"
    );
    let bulwark = uses(Class::Bulwark, SpeciesId::HORNBACK, 6_000);
    assert!(
        bulwark.guards > 0,
        "the Bulwark never took a blow on the shield: {bulwark:?}"
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
