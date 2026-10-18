//! The morph: an egg makes it young, every trophy is a generation you can
//! see, and a handful of one creature's trophies make it more like that
//! creature -- fast enough to read in a short session.

use sim::breed::archetypes::{self, CAST};
use sim::breed::cross::{self, TrophyKind};
use sim::breed::morph::{Line, Morph};
use sim::breed::plan;
use sim::breed::{Genome, Hide};
use sim::fixed::Fx;

fn cast(name: &str) -> Genome {
    *CAST.iter().find(|c| c.1 == name).unwrap().2
}

#[test]
fn an_egg_makes_it_that_creatures_young() {
    let mut m = Morph::hatch(&cast("Hornback"), 1);
    assert_eq!(m.genome, cast("Hornback"));
    assert!(m.growth < Fx::ONE);
    assert!(m.shown().sliders.size < m.genome.sliders.size);
    m.eat(TrophyKind::Feather, &cast("Galewing"));
    m.eat_egg(&cast("Mantis"), 3);
    assert_eq!(m.genome, cast("Mantis"));
    assert_eq!(m.generation, 0);
}

#[test]
fn every_trophy_changes_something_you_can_see() {
    for (_, egg, eg, _) in CAST {
        for (_, donor, dg, _) in CAST {
            if egg == donor {
                continue;
            }
            let Some(kind) = archetypes::drops(dg).into_iter().flatten().next() else {
                continue;
            };
            let mut m = Morph::hatch(eg, 7);
            for feed in 1..=3 {
                let before = m.genome;
                let fed = m.eat(kind, dg);
                // Unless it is already the donor in every part a trophy
                // can move, something shows.
                let same = !cross::visible(&before, dg);
                assert!(
                    fed.iter().next().is_some() || same,
                    "{egg} fed {}'s {kind:?}, feeding {feed}: nothing changed",
                    donor
                );
            }
            assert!(m.growth > Morph::hatch(eg, 7).growth);
        }
    }
}

#[test]
fn a_few_of_one_creatures_trophies_make_it_more_like_that_creature() {
    // Feathers on a Hornback: within six, most seeds fly or have a wing;
    // and its hide has gone over to feathers.
    let mut winged = 0;
    let mut feathered = 0;
    for seed in 0..20 {
        let mut m = Morph::hatch(&cast("Hornback"), seed as u8);
        for _ in 0..6 {
            m.eat(TrophyKind::Feather, &cast("Galewing"));
        }
        if m.genome.has_wings() {
            winged += 1;
        }
        if m.genome.minors.hide == Hide::Feathers {
            feathered += 1;
        }
    }
    assert!(winged >= 10, "{winged} of 20 grew wings in six feathers");
    assert!(feathered >= 15, "{feathered} of 20 took feathers");
}

#[test]
fn a_sandworms_trophies_make_a_digger() {
    let mut dug = 0;
    for seed in 0..20 {
        let mut m = Morph::hatch(&cast("Hornback"), seed as u8);
        for _ in 0..6 {
            // It takes the trait: spades, or burrowing outright. (A plate
            // also pulls the size toward a Sandmaw's, and a body grown past
            // three fifths keeps its spades without going under.)
            let fed = m.eat(TrophyKind::Plate, &cast("Sandmaw"));
            let spades = fed
                .iter()
                .any(|l| matches!(l, Line::Limb(_, sim::breed::Build::Spade, _)));
            if spades || fed.iter().any(|l| *l == Line::Burrows(true)) {
                dug += 1;
                break;
            }
        }
    }
    assert!(dug >= 10, "{dug} of 20 took to digging in six plates");
}

#[test]
fn a_morph_grows_as_it_eats_and_stays_a_creature() {
    let mut m = Morph::hatch(&cast("Ridgeback"), 3);
    let born = m.growth;
    let donors = ["Galewing", "Mantis", "Sandmaw", "Broodmother", "The Pair"];
    for k in 0..15 {
        let d = cast(donors[k % donors.len()]);
        let kind = archetypes::drops(&d).into_iter().flatten().next().unwrap();
        m.eat(kind, &d);
        // Every generation is a body that stands.
        let mut r = m.genome;
        assert_eq!(
            plan::repair(&mut r).count,
            0,
            "generation {} needs repair",
            m.generation
        );
    }
    assert_eq!(m.growth, Fx::ONE);
    assert!(born < Fx::ONE);
    assert_eq!(m.generation, 15);
}
