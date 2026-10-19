//! Eggs and trophies, in the valley: an egg lies by its creature, picking
//! it up sets the creature on you until it is beaten, and a beaten creature
//! drops trophies. What the world keeps of it is only what the fight reads
//! -- what lies on the floor and who carries which egg; a trophy picked up
//! is handed to the game (`World::picked`), and the morph is the game's.

use sim::class::Class;
use sim::fixed::Fx;
use sim::input::{Destination, Input, Travel};
use sim::keep::{Item, Refusal};
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

/// Seat 0 feeding its morph `item`.
fn feed(item: Item) -> [Input; 1] {
    [Input::new(0).travelling(Travel::feed(item))]
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
    // Its egg will not be eaten while it hunts you: the bag says why, and
    // a feeding sent anyway leaves it carried.
    let egg = Item::Egg(SpeciesId::MANTIS);
    assert_eq!(w.refusal(0, egg), Some(Refusal::Chased(SpeciesId::MANTIS)));
    w.advance(feed(egg));
    assert!(w.valley.keep.carries_egg(0, SpeciesId::MANTIS));
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
    // Walked into, each is handed to the game on the frame it is picked
    // up, and gone from the floor.
    for t in &trophies {
        step_onto(&mut w, *t);
        assert!(
            matches!(w.picked[0], Some(Item::Trophy(_, s)) if s == SpeciesId::MANTIS),
            "{:?}",
            w.picked[0]
        );
        w.advance(idle());
        assert_eq!(w.picked[0], None, "a pickup is that frame's alone");
    }
    assert!(
        w.loose
            .iter()
            .flatten()
            .all(|l| !matches!(l.item, Item::Trophy(..)))
    );
    // The egg, beaten for, is eaten: let go of on the frame it is fed.
    let egg = Item::Egg(SpeciesId::MANTIS);
    assert_eq!(w.refusal(0, egg), None);
    w.advance(feed(egg));
    assert!(!w.valley.keep.carries_egg(0, SpeciesId::MANTIS));
    // An egg not carried is not eaten again.
    assert_eq!(w.refusal(0, egg), Some(Refusal::Empty));
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

#[test]
fn an_item_rides_the_wire_by_what_it_is() {
    use sim::breed::cross::TrophyKind;
    for item in [
        Item::Egg(SpeciesId::MANTIS),
        Item::Trophy(TrophyKind::Feather, SpeciesId::MANTIS),
        Item::Trophy(TrophyKind::Tooth, SpeciesId(0)),
    ] {
        assert_eq!(
            Travel::feed(item).destination(),
            Some(Destination::Feed(item))
        );
        assert_eq!(
            Travel::grant(item).destination(),
            Some(Destination::Grant(item))
        );
    }
}

#[test]
fn a_dev_grant_carries_an_egg_and_hands_over_a_trophy() {
    use sim::breed::cross::TrophyKind;
    let mut w = World::arrive(CLASSES, valley::Journey::default(), valley::START, None, 1);
    let feather = Item::Trophy(TrophyKind::Feather, SpeciesId::MANTIS);
    w.advance([Input::new(0).travelling(Travel::grant(feather))]);
    assert_eq!(w.picked[0], Some(feather));
    let egg = Item::Egg(SpeciesId::MANTIS);
    w.advance([Input::new(0).travelling(Travel::grant(egg))]);
    assert!(w.valley.keep.carries_egg(0, SpeciesId::MANTIS));
}
