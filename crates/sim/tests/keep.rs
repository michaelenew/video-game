//! Eggs, trophies and the morph, in the valley: an egg lies by its
//! creature, picking it up sets the creature on you until it is beaten,
//! a beaten creature drops trophies, and crouching by your morph feeds it.

use sim::breed::morph::Morph;
use sim::class::Class;
use sim::fixed::Fx;
use sim::input::Input;
use sim::keep::Item;
use sim::math::V3;
use sim::species::SpeciesId;
use sim::state::{Phase, World};
use sim::valley::{self, ranges};

const CLASSES: [Class; 1] = [Class::Champion];

/// The Mantis's range, walked into: its hunt begun.
fn at_the_mantis() -> World {
    let r = ranges::RANGES
        .iter()
        .find(|r| r.species == SpeciesId::MANTIS)
        .unwrap();
    World::arrive(CLASSES, valley::Journey::default(), r.arena, Some(0), 1)
}

fn idle() -> [Input; 1] {
    [Input::new(0)]
}

fn egg_here(w: &World) -> Option<V3> {
    w.loose
        .iter()
        .flatten()
        .find(|l| l.item == Item::Egg(SpeciesId::MANTIS))
        .map(|l| l.at)
}

/// Stand seat 0 on a point and step a frame.
fn step_onto(w: &mut World, at: V3) {
    w.players[0].pos = at;
    w.players[0].vel = V3::ZERO;
    w.advance(idle());
}

#[test]
fn every_creature_has_an_egg_and_you_pick_it_up_by_walking_into_it() {
    let mut w = at_the_mantis();
    assert!(w.hunting());
    let egg = egg_here(&w).expect("no egg by the Mantis");
    // It lies past the creature, from where you came in.
    let mantis = w.monsters[0].unwrap().pos;
    let from = w.players[0].pos;
    let flat = |a: V3, b: V3| V3::new(a.x.sub(b.x), Fx::ZERO, a.z.sub(b.z)).len();
    assert!(flat(egg, from) > flat(mantis, from));
    step_onto(&mut w, egg);
    assert!(w.valley.keep.carries_egg(0, SpeciesId::MANTIS));
    assert!(egg_here(&w).is_none());
}

#[test]
fn carrying_its_egg_it_hunts_you_and_does_not_stop() {
    let mut w = at_the_mantis();
    let egg = egg_here(&w).unwrap();
    step_onto(&mut w, egg);
    assert_eq!(w.egg_carrier().map(|(i, _)| i), Some(0));
    let m = w.monsters[0].unwrap();
    assert_eq!(m.brain.grace, 0);
    assert_eq!(m.brain.target, 0);
    // Run a long way off: the hunt is not over.
    let far = V3::new(
        w.players[0].pos.x.add(Fx::from_int(150)),
        w.players[0].pos.y,
        w.players[0].pos.z,
    );
    for _ in 0..30 {
        step_onto(&mut w, far);
    }
    assert!(w.hunting(), "the hunt ended with its egg still carried");
    // Its egg will not be eaten while it hunts you.
    let heel = w.heel[0].at;
    w.players[0].pos = heel;
    w.advance([Input::new(Input::CROUCH)]);
    assert!(w.valley.keep.carries_egg(0, SpeciesId::MANTIS));
    assert_eq!(w.valley.keep.morphs[0].of, Morph::UNFORMED);
}

#[test]
fn beaten_it_drops_trophies_and_the_morph_eats_them() {
    let mut w = at_the_mantis();
    let egg = egg_here(&w).unwrap();
    step_onto(&mut w, egg);
    // Beaten.
    for m in w.monsters.iter_mut().flatten() {
        m.health = 0;
    }
    for _ in 0..4 {
        w.advance(idle());
        if matches!(w.phase, Phase::RoundOver { .. }) {
            break;
        }
    }
    let trophies: Vec<V3> = w
        .loose
        .iter()
        .flatten()
        .filter(|l| matches!(l.item, Item::Trophy(_, s) if s == SpeciesId::MANTIS))
        .map(|l| l.at)
        .collect();
    assert!(!trophies.is_empty(), "nothing dropped");
    // Through the pause: the place goes quiet, and what you carry is still
    // yours.
    for _ in 0..400 {
        w.advance(idle());
    }
    assert!(!w.hunting());
    assert!(w.valley.keep.carries_egg(0, SpeciesId::MANTIS));
    for t in &trophies {
        step_onto(&mut w, *t);
    }
    let carried = w.valley.keep.satchel[0].iter().flatten().count();
    assert_eq!(carried, 1 + trophies.len());
    // Crouch by the morph: it eats the egg first -- a young Mantis -- and
    // then each trophy, a generation each.
    let mut most = 0;
    for _ in 0..120 {
        let heel = w.heel[0].at;
        w.players[0].pos = heel;
        w.advance([Input::new(Input::CROUCH)]);
        most = most.max(w.valley.keep.morphs[0].generation);
    }
    let after = w.valley.keep.morphs[0];
    assert_eq!(most, trophies.len() as u16, "it did not eat every trophy");
    assert_eq!(after.of, SpeciesId::MANTIS.0, "it never ate the egg");
    assert_eq!(after.generation, trophies.len() as u16);
    assert!(w.valley.keep.satchel[0].iter().all(|s| s.is_none()));
}

#[test]
fn losing_the_hunt_loses_the_egg() {
    let mut w = at_the_mantis();
    let egg = egg_here(&w).unwrap();
    step_onto(&mut w, egg);
    w.players[0].health = 0;
    for _ in 0..600 {
        w.advance(idle());
        if !w.hunting() {
            break;
        }
    }
    assert!(!w.valley.keep.carries_egg(0, SpeciesId::MANTIS));
}

#[test]
fn the_morph_keeps_at_heel() {
    let mut w = at_the_mantis();
    w.advance(idle());
    let start = w.players[0].pos;
    // Walk ten metres off; it follows to within its follow distance.
    let to = V3::new(start.x.add(Fx::from_int(10)), start.y, start.z);
    for _ in 0..240 {
        w.players[0].pos = V3::new(to.x, w.players[0].pos.y, to.z);
        w.advance(idle());
    }
    let h = w.heel[0].at;
    let p = w.players[0].pos;
    let gap = V3::new(h.x.sub(p.x), Fx::ZERO, h.z.sub(p.z)).len();
    assert!(
        gap <= sim::tuning::morph_follow().add(Fx::ONE),
        "it is {gap:?} behind"
    );
}

#[test]
fn it_is_the_same_every_time() {
    let run = || {
        let mut w = at_the_mantis();
        let egg = egg_here(&w).unwrap();
        step_onto(&mut w, egg);
        for _ in 0..120 {
            w.advance(idle());
        }
        w.checksum()
    };
    assert_eq!(run(), run());
}
