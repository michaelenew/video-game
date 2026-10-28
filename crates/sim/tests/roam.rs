//! **Creatures that leave their range** (exploration 0023, `sim::valley::roam`):
//! a creature after its egg follows the carrier across the valley and can
//! still hit them; with no egg to get back it gives up and walks home; a
//! creature beaten far from home drops its trophies where it fell and its
//! range rests; a chase that crosses another creature's range does not wake
//! it until the chase is over; and a long chase is deterministic.

use sim::Class;
use sim::arena::ArenaId;
use sim::atlas::{self, Placed};
use sim::fixed::Fx;
use sim::input::Input;
use sim::keep::Item;
use sim::math::V3;
use sim::species::SpeciesId;
use sim::state::{Phase, World};
use sim::valley::{self, ranges};

fn on_map(w: &World, i: usize) -> V3 {
    w.players[i].pos.add(w.map_origin())
}

/// The place of the map a point is in, a reach for the road.
fn place_of(p: V3) -> ArenaId {
    atlas::valley()
        .place_for(p)
        .map_or(ArenaId::HEARTH, |p| p.arena)
}

fn range(name: &str) -> &'static ranges::Range {
    ranges::RANGES.iter().find(|r| r.name == name).unwrap()
}

fn placed(id: ArenaId) -> Placed {
    *atlas::valley().placed(id).unwrap()
}

/// One frame of fighter one walking toward a map point; true once there.
fn step_to(w: &mut World, to: (Fx, Fx)) -> bool {
    let feet = on_map(w, 0);
    let (dx, dz) = (to.0.sub(feet.x), to.1.sub(feet.z));
    if sim::math::wide_len(V3::new(dx, Fx::ZERO, dz)).raw() < Fx::ratio(3, 2).raw() {
        return true;
    }
    let aim = sim::math::atan2_turns(dz, dx).raw() as u16;
    w.advance([Input::looking_at(Input::W, aim, -1500)]);
    false
}

/// The road from the point of it nearest `from`, toward the town.
fn road_home_from(from: V3) -> Vec<(Fx, Fx)> {
    let road = &sim::valley::layout::road()[0];
    let k = (0..road.len())
        .min_by_key(|&k| {
            sim::math::wide_len(V3::new(
                road[k].0.sub(from.x),
                Fx::ZERO,
                road[k].1.sub(from.z),
            ))
            .raw()
        })
        .unwrap();
    road[..=k].iter().rev().copied().collect()
}

/// Upper Meadow's two Ridgebacks, walked in, and their egg picked up.
fn carrying_the_meadows_egg() -> World {
    let r = range("Upper Meadow");
    let mut w = World::arrive(
        [Class::Champion],
        valley::Journey::default(),
        r.arena,
        Some(0),
        1,
    );
    assert!(w.hunting());
    let egg = w
        .loose
        .iter()
        .flatten()
        .find(|l| l.item == Item::Egg(SpeciesId::RIDGEBACK))
        .unwrap()
        .at;
    w.players[0].pos = egg;
    w.advance([Input::new(0)]);
    assert_eq!(w.egg_carrier().map(|(i, _)| i), Some(0));
    w
}

/// Fighter one walks the road toward the town for at most `frames`, never
/// dying (their health is put back each frame -- what is being measured is
/// whether the creatures keep up, not who wins); every place they were hit
/// in, in order.
fn flee_down_the_road(w: &mut World, frames: u32) -> Vec<ArenaId> {
    let mut hit_in = Vec::new();
    let path = road_home_from(on_map(w, 0));
    let mut f = 0;
    'walk: for to in path {
        loop {
            if f >= frames {
                break 'walk;
            }
            f += 1;
            let before = w.players[0].health;
            if step_to(w, to) {
                break;
            }
            if w.players[0].health < before {
                let here = place_of(on_map(w, 0));
                if hit_in.last() != Some(&here) {
                    hit_in.push(here);
                }
            }
            w.players[0].health = sim::tuning::health_of(Class::Champion);
        }
    }
    hit_in
}

#[test]
fn an_egg_carrier_is_chased_through_two_places_and_hit_in_each() {
    let mut w = carrying_the_meadows_egg();
    let home = range("Upper Meadow").arena;
    let hit_in = flee_down_the_road(&mut w, 3600);
    assert!(w.hunting(), "the hunt ended with its egg still carried");
    assert_eq!(w.valley.den, Some(home), "the hunt forgot its home");
    // Out on the road, in the reaches: not the range.
    let reaches: Vec<ArenaId> = hit_in
        .iter()
        .copied()
        .filter(|&a| ranges::of(a).is_none())
        .collect();
    assert!(
        reaches.len() >= 2,
        "hit in {:?}: the chase did not carry through two places",
        hit_in.iter().map(|a| a.get().name).collect::<Vec<_>>()
    );
    assert!(w.den().out(), "the creatures are not out of their range");
    assert!(
        w.past_home(&w.den()).raw() > Fx::from_int(50).raw(),
        "the creatures did not follow far"
    );
    // And the world ran in coordinates near the fighters all the way: a
    // place with an origin of its own, not a reach's.
    assert!(
        ranges::of(w.arena).is_some() || w.arena == valley::START || {
            let p = placed(w.arena);
            p.at != V3::ZERO
        }
    );
}

#[test]
fn without_its_egg_it_gives_up_and_walks_home() {
    // The Snowfield's Ridgeback hunts whoever walks in.
    let r = range("Snowfield");
    let mut w = World::arrive(
        [Class::Champion],
        valley::Journey::default(),
        r.arena,
        Some(0),
        1,
    );
    assert!(w.hunting() && w.pursuing());
    assert!(w.egg_carrier().is_none());
    let home = placed(r.arena);
    // Out of the range and away down the road a way, and stood there, never
    // dying, while it comes after them.
    let path = road_home_from(on_map(&w, 0));
    let mut f = 0u32;
    let mut followed = false;
    let mut gave_up = false;
    let mut home_again: Option<Fx> = None;
    for to in path.iter().take(4) {
        while !step_to(&mut w, *to) && f < 1200 {
            f += 1;
            w.players[0].health = sim::tuning::health_of(Class::Champion);
        }
    }
    let gone = |w: &World| sim::math::wide_flat_dist(on_map(w, 0), home.at);
    assert!(
        gone(&w).raw() > Fx::from_int(40).raw(),
        "never got away from the range"
    );
    for _ in 0..(sim::tuning::roam_give_up_frames() * 4) {
        if !w.hunting() {
            break;
        }
        followed |= w.den().out();
        if w.valley.homing && !gave_up {
            gave_up = true;
            assert!(
                w.egg_carrier().is_none(),
                "it gave up on a chase for its egg"
            );
        }
        if gave_up {
            home_again = Some(w.past_home(&w.den()));
        }
        w.advance([Input::new(0)]);
        w.players[0].health = sim::tuning::health_of(Class::Champion);
    }
    assert!(followed, "it never left its range after them");
    assert!(gave_up, "it never gave up");
    // The last it was seen before its hunt calmed: home.
    let last = home_again.expect("it gave up and its hunt ended at once");
    assert!(
        last.raw() < Fx::ONE.raw(),
        "its hunt calmed {} m from home: it did not walk home",
        last.to_f32_for_render()
    );
    assert!(
        !w.hunting(),
        "home, with nobody near, its hunt did not calm"
    );
    assert!(!w.valley.homing && w.valley.den.is_none());
}

#[test]
fn under_an_egg_chase_it_never_gives_up() {
    let mut w = carrying_the_meadows_egg();
    flee_down_the_road(&mut w, 1500);
    // Stood a long way off, far past the distance it gives up at: it is
    // still after its egg.
    for _ in 0..(sim::tuning::roam_give_up_frames() + 600) {
        w.advance([Input::new(0)]);
        w.players[0].health = sim::tuning::health_of(Class::Champion);
    }
    assert!(w.hunting() && !w.valley.homing, "it gave up on its egg");
}

#[test]
fn beaten_far_from_home_it_drops_its_trophies_where_it_fell_and_its_range_rests() {
    let mut w = carrying_the_meadows_egg();
    let home = range("Upper Meadow").arena;
    flee_down_the_road(&mut w, 1500);
    let den = w.den();
    assert!(den.out());
    let fell: Vec<V3> = w.monsters.iter().flatten().map(|m| m.pos).collect();
    for m in w.monsters.iter_mut().flatten() {
        m.health = 0;
    }
    w.advance([Input::new(0)]);
    assert!(w.hunt_won().is_some(), "beaten, and not won");
    let trophies: Vec<V3> = w
        .loose
        .iter()
        .flatten()
        .filter(|l| matches!(l.item, Item::Trophy(..)))
        .map(|l| l.at)
        .collect();
    assert!(!trophies.is_empty(), "no trophies");
    for t in &trophies {
        let near = fell
            .iter()
            .any(|f| sim::math::wide_flat_dist(*f, *t).raw() < Fx::from_int(3).raw());
        assert!(near, "a trophy is not where a creature fell");
    }
    for _ in 0..sim::tuning::round_over_frames() + 2 {
        w.advance([Input::new(0)]);
    }
    assert!(!w.hunting() && matches!(w.phase, Phase::Fighting));
    assert!(w.resting(home), "its range does not rest");
    assert!(w.valley.has_beaten(SpeciesId::RIDGEBACK));
}

#[test]
fn lost_far_from_home_it_takes_its_egg_back() {
    let mut w = carrying_the_meadows_egg();
    flee_down_the_road(&mut w, 1500);
    assert!(w.den().out());
    w.players[0].health = 0;
    for _ in 0..sim::tuning::round_over_frames() + 3 {
        w.advance([Input::new(0)]);
    }
    assert!(!w.hunting(), "the hunt went on with nobody left");
    assert!(
        !w.valley.keep.carries_egg(0, SpeciesId::RIDGEBACK),
        "lost, and still carrying its egg"
    );
    assert!(
        !w.loose
            .iter()
            .flatten()
            .any(|l| l.item == Item::Egg(SpeciesId::RIDGEBACK)),
        "lost, and its egg lies about"
    );
    assert!(w.players[0].health > 0, "nobody stood back up");
}

/// **One live hunt**: a chase into another creature's range does not wake
/// it -- the world has two creature slots and one pack, and they are the
/// chaser's -- and once the chase is over, with somebody standing in it,
/// that herd wakes as if just walked up to.
#[test]
fn a_chase_through_another_range_does_not_wake_it_until_it_is_over() {
    let mut w = carrying_the_meadows_egg();
    let other = range("South Bank");
    flee_down_the_road(&mut w, 1500);
    // Up to the South Bank's Gnawers, where they graze -- a herd that roams
    // is woken by coming near it, not by its ground.
    let o = w.map_origin();
    let mid = sim::valley::sight::herd_middle(other, w.frame);
    let floor = atlas::valley().relief_at(mid.x, mid.z);
    w.players[0].pos = V3::new(mid.x, floor.add(Fx::ratio(1, 2)), mid.z).sub(o);
    w.players[0].vel = V3::ZERO;
    for _ in 0..30 {
        w.advance([Input::new(0)]);
        w.players[0].health = sim::tuning::health_of(Class::Champion);
    }
    assert_eq!(w.valley.den, Some(range("Upper Meadow").arena));
    assert!(
        w.monsters
            .iter()
            .flatten()
            .all(|m| m.species == SpeciesId::RIDGEBACK),
        "another creature joined the chase"
    );
    assert!(w.pack.is_none(), "the South Bank's pack woke mid-chase");
    // The chase is over: the creatures beaten.
    for m in w.monsters.iter_mut().flatten() {
        m.health = 0;
    }
    for _ in 0..sim::tuning::round_over_frames() + 4 {
        w.advance([Input::new(0)]);
    }
    assert_eq!(
        w.arena, other.arena,
        "the world is not in the range they stand by"
    );
    assert!(w.hunting(), "the herd they stand by did not wake");
    assert_eq!(w.valley.den, Some(other.arena));
    assert_eq!(w.pack.map(|p| p.species), Some(SpeciesId::GNAWERS));
}

#[test]
fn a_long_chase_is_deterministic() {
    let run = || {
        let mut w = carrying_the_meadows_egg();
        let path = road_home_from(on_map(&w, 0));
        let mut f = 0u32;
        for to in path {
            while f < 4000 {
                f += 1;
                // A little of everything as they go, so the creatures fight.
                let feet = on_map(&w, 0);
                let (dx, dz) = (to.0.sub(feet.x), to.1.sub(feet.z));
                if sim::math::wide_len(V3::new(dx, Fx::ZERO, dz)).raw() < Fx::ratio(3, 2).raw() {
                    break;
                }
                let aim = sim::math::atan2_turns(dz, dx).raw() as u16;
                let extra = match f % 120 {
                    0..=4 => Input::LEFT,
                    40..=44 => Input::SPACE,
                    _ => 0,
                };
                w.advance([Input::looking_at(Input::W | extra, aim, -1500)]);
                w.players[0].health = sim::tuning::health_of(Class::Champion);
            }
        }
        (w.checksum(), w.arena, w.hunting())
    };
    let a = run();
    assert!(a.2, "the chase was over before the end");
    assert_eq!(a, run());
}
