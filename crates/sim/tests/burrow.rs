//! Burrowing as a trait that passes on: the worm's way (a body that loses
//! its legs) and the mole's (a body that keeps them and grows spades), and
//! what the plan says about who can carry it.

use sim::breed::archetypes::CAST;
use sim::breed::build::{self, MoveKind};
use sim::breed::cross::{self, Rates, Trophy, TrophyKind};
use sim::breed::plan;
use sim::breed::{Build, Genome, Rng, Stance, Tip};

fn cast(name: &str) -> Genome {
    *CAST.iter().find(|c| c.1 == name).unwrap().2
}

/// The body with its front walking pair turned to spades.
fn with_spades(mut g: Genome) -> Genome {
    let i = (0..4)
        .find(|&i| g.limbs[i].is_some_and(|p| p.role.grounded()))
        .unwrap();
    let p = g.limbs[i].as_mut().unwrap();
    p.build = Build::Spade;
    p.tip = Tip::Claw;
    g
}

#[test]
fn the_sandworm_burrows_and_has_the_moves_for_it() {
    let g = cast("Sandmaw");
    assert!(plan::assess(&g).caps.burrows);
    let set = build::moves(&g);
    assert!(set.has(MoveKind::Burrow) && set.has(MoveKind::Erupt));
    // Nobody else in the cast does.
    for (_, name, g, _) in CAST {
        if name != "Sandmaw" {
            assert!(!plan::assess(g).caps.burrows, "{name} burrows");
        }
    }
}

#[test]
fn a_small_body_on_spades_burrows_and_a_giant_does_not() {
    let mole = with_spades(cast("Hornback"));
    assert!(
        plan::assess(&mole).caps.burrows,
        "a Hornback on spades should dig"
    );
    assert!(build::moves(&mole).has(MoveKind::Erupt));
    // The Ridgeback is too big to go under, spades or not: it keeps them
    // and rakes with them.
    let giant = with_spades(cast("Ridgeback"));
    assert!(!plan::assess(&giant).caps.burrows);
    assert!(build::moves(&giant).has(MoveKind::Rake));
    // And the plan strains under it: a body that big is less at ease on
    // spades than on its own legs.
    assert!(plan::assess(&giant).plausibility < plan::assess(&cast("Ridgeback")).plausibility);
}

#[test]
fn a_legged_body_inherits_burrowing_and_keeps_its_legs() {
    // A Hornback given a Sandmaw's plates, a trophy a generation, grows
    // spades and burrows standing on them. The worm's way hardly ever
    // happens: a stance flipped to limbless on a body that has legs is
    // repaired to low-slung (`plan::repair`), so a body becomes a worm only
    // by losing every pair, which mutation alone does and rarely. Before
    // spades, a legged body could not inherit burrowing at all.
    let worm = cast("Sandmaw");
    let (mut moles, mut worms) = (0, 0);
    for seed in 0..40 {
        let mut rng = Rng::new(seed);
        let mut g = cast("Hornback");
        for _ in 0..30 {
            let t = Trophy {
                kind: TrophyKind::Plate,
                donor: worm,
            };
            g = cross::graft(&g, &t, &Rates::DEFAULT, &mut rng).genome;
            if plan::assess(&g).caps.burrows {
                break;
            }
        }
        if plan::assess(&g).caps.burrows {
            if g.stance == Stance::Limbless {
                worms += 1;
            } else {
                assert!(g.grounded_pairs() > 0);
                assert!(g.pairs().any(|(_, p)| p.build == Build::Spade));
                moles += 1;
            }
        }
    }
    assert!(moles > worms, "moles {moles}, worms {worms}");
    assert!(moles >= 5, "only {moles} of forty lineages dug");
}

#[test]
fn burrowing_passes_by_breeding_too() {
    let mole = with_spades(cast("Hornback"));
    let mut rng = Rng::new(3);
    let digging = (0..200)
        .filter(|_| {
            let child = cross::breed(&mole, &cast("Hornback"), &Rates::DEFAULT, &mut rng).genome;
            plan::assess(&child).caps.burrows
        })
        .count();
    // About half the young take the digging parent's front legs.
    assert!((60..=140).contains(&digging), "{digging} of 200");
}
