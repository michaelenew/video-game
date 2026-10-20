//! **Ranges** (`sim::valley::ranges`, exploration 0019 §2e): creatures out on
//! the land. Every range is a place of the map, walking into one wakes it, the
//! road never does, and each keeps its temperament -- the peaceful ones start
//! nothing until hurt, the territorial ones wait for you to come close, the
//! skittish herd runs, the hunting ones come for you -- with a temper that
//! rises the further it is from Hearth. Beaten, a range rests and comes back;
//! lost on, you stand up on your cairn. The roost on the Needle's dome is a
//! range stacked over the Needle's plaza, and the plaza is not in it.

use sim::arena::ArenaId;
use sim::valley::ranges::{self, RANGES, Range, Temperament};
use sim::valley::{self, Journey};
use sim::{Class, Fx, Input, V3, World};

fn cm(v: i32) -> Fx {
    sim::arena::cm(v)
}

fn journey() -> Journey {
    Journey {
        beaten: u32::MAX,
        ..Journey::default()
    }
}

/// The reach a range stands in: the place its middle is in on the map, the
/// stacked roost aside.
fn reach_of(r: &Range) -> ArenaId {
    let atlas = sim::atlas::valley();
    atlas
        .places
        .iter()
        .find(|p| ranges::of(p.arena).is_none() && p.holds(cm(r.at.0), cm(r.at.1)))
        .expect("on the map")
        .arena
}

/// A world in the range's reach with fighter one stood at a map point.
fn standing(r: &Range, at: (Fx, Fx, Fx)) -> World {
    let mut w = World::arrive(
        [Class::Champion, Class::Elementalist],
        journey(),
        reach_of(r),
        None,
        1,
    );
    let o = w.map_origin();
    w.players[0].pos = V3::new(at.0, at.1, at.2).sub(o);
    w.players[0].vel = V3::ZERO;
    w
}

/// Fighter one walked into a range: stood at its near edge, a step inside,
/// on its floor, and a frame advanced.
fn walked_in(r: &Range) -> World {
    let (x, z) = (cm(r.at.0), cm(r.at.1 - r.half + 250));
    let mut w = standing(r, (x, cm(r.floor).add(Fx::ratio(1, 2)), z));
    for _ in 0..3 {
        w.advance([Input::new(0), Input::new(0)]);
    }
    w
}

#[test]
fn every_range_is_a_place_of_the_map() {
    let atlas = sim::atlas::valley();
    for r in &RANGES {
        let p = atlas
            .placed(r.arena)
            .unwrap_or_else(|| panic!("{} is not on the map", r.name));
        assert_eq!(p.band.is_some(), r.stacked.is_some(), "{}", r.name);
        let feet = V3::new(cm(r.at.0), cm(r.floor).add(Fx::ONE), cm(r.at.1));
        assert_eq!(
            atlas.place_for(feet).map(|p| p.arena),
            Some(r.arena),
            "{}: its middle is not in it",
            r.name
        );
        assert!(
            valley::place(r.arena).is_some(),
            "{} is not a place",
            r.name
        );
    }
}

#[test]
fn walking_into_a_range_wakes_it_and_its_creatures_stand_in_it() {
    for r in &RANGES {
        let w = walked_in(r);
        assert_eq!(
            w.arena,
            r.arena,
            "{}: the world is in {}",
            r.name,
            w.arena().name
        );
        assert!(w.hunting(), "{}: nothing woke", r.name);
        let o = w.map_origin();
        for m in w.monsters.iter().flatten() {
            let p = m.pos.add(o);
            // On its floor -- or over it, for a flier.
            let up = p.y.sub(cm(r.floor));
            assert!(
                up.raw() > Fx::from_int(-3).raw() && up.raw() < Fx::from_int(30).raw(),
                "{}: a creature at {} m, its floor at {}",
                r.name,
                p.y.to_f32_for_render(),
                r.floor / 100
            );
            assert_eq!(m.temper, r.temper, "{}: its temper", r.name);
        }
        if let Some(pack) = w.pack {
            assert_eq!(pack.temper, r.temper, "{}: its pack's temper", r.name);
        }
    }
}

#[test]
fn the_road_never_wakes_a_range() {
    let margin = sim::tuning::hunt_held_past();
    let mut legs = sim::valley::layout::road();
    legs.push(
        sim::arena::wood::WAY
            .iter()
            .map(|q| (cm(q.x), cm(q.z)))
            .collect(),
    );
    for leg in legs {
        for pair in leg.windows(2) {
            for k in 0..=20 {
                let t = Fx::ratio(k, 20);
                let x = pair[0].0.add(pair[1].0.sub(pair[0].0).mul(t));
                let z = pair[0].1.add(pair[1].1.sub(pair[0].1).mul(t));
                for r in RANGES.iter().filter(|r| r.stacked.is_none()) {
                    let p = sim::atlas::valley().placed(r.arena).unwrap();
                    assert!(
                        !p.holds_within(x, z, margin),
                        "the road at ({}, {}) is in {}'s reach",
                        x.to_f32_for_render(),
                        z.to_f32_for_render(),
                        r.name
                    );
                }
            }
        }
    }
}

/// A range by its temperament.
fn first(t: fn(&Temperament) -> bool) -> &'static Range {
    RANGES
        .iter()
        .find(|r| t(&r.temperament))
        .expect("a range of it")
}

/// The live creatures' health, all of it.
fn stock(w: &World) -> i32 {
    w.monsters.iter().flatten().map(|m| m.health).sum::<i32>()
        + w.critters
            .iter()
            .filter(|c| c.alive())
            .map(|c| c.health as i32)
            .sum::<i32>()
}

/// Hurt the range's creatures a little, as a blow would.
fn hurt(w: &mut World) {
    if let Some(m) = w.monsters.iter_mut().flatten().next() {
        m.health -= 10;
    } else if let Some(c) = w.critters.iter_mut().find(|c| c.alive()) {
        c.health -= 2;
    }
}

#[test]
fn a_peaceful_herd_starts_nothing_until_it_is_hurt() {
    let r = first(|t| *t == Temperament::Peaceful);
    let mut w = walked_in(r);
    let health = w.players[0].health;
    // Stood among them for ten seconds: nobody is hurt, nothing is roused.
    let o = w.map_origin();
    let mid = V3::new(cm(r.at.0), cm(r.floor), cm(r.at.1)).sub(o);
    w.players[0].pos = mid;
    for _ in 0..600 {
        w.advance([Input::new(0), Input::new(0)]);
    }
    assert_eq!(w.players[0].health, health, "a peaceful creature hurt her");
    assert!(!w.valley.roused);
    assert!(w.monsters.iter().flatten().all(|m| m.brain.grace > 0));
    // Hurt one: it wakes.
    hurt(&mut w);
    w.advance([Input::new(0), Input::new(0)]);
    assert!(w.valley.roused, "a hurt did not rouse it");
    assert!(
        w.monsters.iter().flatten().all(|m| m.brain.grace == 0),
        "roused, it is still held"
    );
}

#[test]
fn a_territorial_creature_waits_until_you_come_close() {
    let r = first(|t| matches!(t, Temperament::Territorial { .. }));
    let Temperament::Territorial { ring } = r.temperament else {
        unreachable!()
    };
    let mut w = walked_in(r);
    for _ in 0..120 {
        w.advance([Input::new(0), Input::new(0)]);
    }
    assert!(!w.valley.roused, "{}: roused from its edge", r.name);
    // Into its ring.
    let o = w.map_origin();
    let inside = V3::new(cm(r.at.0), cm(r.floor), cm(r.at.1 - ring / 2)).sub(o);
    w.players[0].pos = inside;
    w.advance([Input::new(0), Input::new(0)]);
    assert!(w.valley.roused, "{}: not roused in its ring", r.name);
}

#[test]
fn a_skittish_herd_runs_when_it_is_hurt() {
    let r = first(|t| *t == Temperament::Skittish);
    let mut w = walked_in(r);
    let Some(_) = w.pack else {
        // A skittish creature with a body only: it never fights.
        return;
    };
    let before = stock(&w);
    hurt(&mut w);
    assert!(stock(&w) < before);
    for _ in 0..2 {
        w.advance([Input::new(0), Input::new(0)]);
    }
    let pack = w.pack.expect("the herd");
    assert_eq!(pack.mood, sim::pack::mood::BROKEN, "it did not run");
    // Its home is the far side from her.
    let me = w.players[0].pos;
    assert!(
        pack.home.dot(me).raw() < 0,
        "it runs toward her: home {:?}, she is at {me:?}",
        pack.home
    );
}

#[test]
fn a_hunting_range_is_not_held() {
    let r = first(|t| *t == Temperament::Hunting);
    let mut w = walked_in(r);
    for _ in 0..30 {
        w.advance([Input::new(0), Input::new(0)]);
    }
    assert!(
        w.monsters
            .iter()
            .flatten()
            .all(|m| m.brain.grace < u16::MAX / 4),
        "a hunting creature is held"
    );
    if let Some(p) = w.pack {
        assert!(p.grace < u16::MAX / 4, "a hunting pack is held");
    }
}

#[test]
fn a_beaten_range_rests_and_comes_back() {
    let r = &RANGES[2];
    let k = ranges::index(r.arena).unwrap();
    let mut w = walked_in(r);
    assert!(w.hunting());
    // Beaten, a frame ago.
    w.valley.rested[k] = w.frame;
    w.monsters = [None; sim::monster::MAX_MONSTERS];
    w.pack = None;
    w.critters = sim::critter::Critters::NONE;
    assert!(w.resting(r.arena));
    // Out and back in: still resting, nothing wakes.
    let o = w.map_origin();
    let out = V3::new(cm(r.at.0), cm(r.floor), cm(r.at.1 - r.half - 4000)).sub(o);
    w.players[0].pos = out;
    w.advance([Input::new(0), Input::new(0)]);
    let o = w.map_origin();
    w.players[0].pos = V3::new(cm(r.at.0), cm(r.floor).add(Fx::ONE), cm(r.at.1)).sub(o);
    w.advance([Input::new(0), Input::new(0)]);
    assert_eq!(w.arena, r.arena);
    assert!(!w.hunting(), "a resting range woke");
    // After its rest: back.
    w.frame = w.frame.wrapping_add(sim::tuning::range_rest_frames());
    assert!(!w.resting(r.arena));
    let o = w.map_origin();
    w.players[0].pos = V3::new(cm(r.at.0), cm(r.floor), cm(r.at.1 - r.half - 4000)).sub(o);
    w.advance([Input::new(0), Input::new(0)]);
    let o = w.map_origin();
    w.players[0].pos = V3::new(cm(r.at.0), cm(r.floor).add(Fx::ONE), cm(r.at.1)).sub(o);
    w.advance([Input::new(0), Input::new(0)]);
    assert!(w.hunting(), "it never came back");
}

#[test]
fn the_roost_is_on_the_dome_and_not_on_the_plaza_under_it() {
    let r = RANGES
        .iter()
        .find(|r| r.stacked.is_some())
        .expect("the roost");
    // On the plaza under the dome: the Needle, and nothing wakes.
    let mut w = standing(r, (cm(r.at.0 + 2500), Fx::ZERO, cm(r.at.1)));
    let o = w.map_origin();
    let up = V3::new(cm(r.at.0 + 2500), Fx::from_int(5), cm(r.at.1)).sub(o);
    w.players[0].pos.y = w.terrain().floor_below(up);
    for _ in 0..3 {
        w.advance([Input::new(0), Input::new(0)]);
    }
    assert_eq!(
        w.arena,
        ArenaId::SPIRE,
        "the plaza is in {}",
        w.arena().name
    );
    assert!(!w.hunting(), "the roost woke from the plaza");
    // On the dome's top: the roost.
    let w = walked_in(r);
    assert_eq!(w.arena, r.arena);
    assert!(w.hunting(), "the roost did not wake on the dome");
}

#[test]
fn a_hunt_lost_on_a_range_stands_you_up_on_your_cairn() {
    let r = first(|t| *t == Temperament::Hunting);
    let mut w = walked_in(r);
    w.players[0].health = 0;
    for _ in 0..600 {
        w.advance([Input::new(0), Input::new(0)]);
        if w.players[0].health > 0 {
            break;
        }
    }
    assert!(w.players[0].health > 0, "never stood back up");
    assert_ne!(w.arena, ArenaId::HEARTH, "woke in Hearth, not on a cairn");
}

#[test]
fn arriving_on_a_range_wakes_it_as_walking_in_does() {
    let r = RANGES
        .iter()
        .find(|r| r.temper > 0)
        .expect("a tempered range");
    let w = World::arrive(
        [Class::Champion, Class::Elementalist],
        journey(),
        r.arena,
        None,
        1,
    );
    assert!(w.hunting());
    assert!(w.monsters.iter().flatten().all(|m| m.temper == r.temper));
}

#[test]
fn a_roused_range_fights() {
    // Every range but the skittish one, which never does: stood in its middle
    // and hurting it, a fighter who does nothing is hurt within forty seconds.
    let mut quiet = Vec::new();
    for r in RANGES
        .iter()
        .filter(|r| r.temperament != Temperament::Skittish)
    {
        let mut w = walked_in(r);
        let o = w.map_origin();
        // In its middle -- but the roost's middle is the Needle's chimney,
        // which a vine held her in until 2026-10-16: on the dome beside it.
        let mut at = V3::new(cm(r.at.0), cm(r.floor).add(Fx::ONE), cm(r.at.1)).sub(o);
        if r.species == sim::species::SpeciesId::GALEWING && r.stacked.is_some() {
            at.z = at.z.sub(Fx::from_int(8));
        }
        w.players[0].pos = at;
        w.players[0].health = 100_000;
        hurt(&mut w);
        let start = w.players[0].health;
        let mut hurt_her = false;
        for _ in 0..2400 {
            w.advance([Input::new(0), Input::new(0)]);
            if w.players[0].health < start {
                hurt_her = true;
                break;
            }
        }
        if !hurt_her {
            quiet.push(r.name);
        }
    }
    assert!(quiet.is_empty(), "never fought: {quiet:?}");
}

/// **A cairn is a checkpoint in a fight too.** The Needle's top cairn is on
/// the roost, which wakes as you step onto the ruins: kept to peace, it never
/// counted, and a fall off the dome stood you back up a hundred metres down
/// the stair (2026-10-20, from play). It counts now -- and rests nobody while
/// the fight is on, so it is not a place to heal against the Galewing.
#[test]
fn the_needles_top_cairn_counts_while_the_roost_is_awake() {
    let r = RANGES
        .iter()
        .find(|r| r.name == "Roost")
        .expect("the roost");
    let atlas = sim::atlas::valley();
    let (x, z) = (cm(r.at.0), cm(r.at.1));
    let reach = cm(sim::arena::spire::RIM);
    let (k, top) = atlas
        .near((x.sub(reach), z.sub(reach)), (x.add(reach), z.add(reach)))
        .map(|i| (i as usize, atlas.solids[i as usize]))
        .filter(|(_, s)| s.material == sim::arena::Material::Snow)
        .max_by_key(|(_, s)| s.max.y.raw())
        .expect("a cairn on the dome");
    let at = (
        Fx::from_raw(top.min.x.raw() / 2 + top.max.x.raw() / 2),
        top.max.y,
        Fx::from_raw(top.min.z.raw() / 2 + top.max.z.raw() / 2),
    );
    // On the ruins first, so the Galewing is awake -- as it is by the time
    // anybody climbing out of the chimney reaches the cairn.
    let mut w = walked_in(r);
    assert_eq!(w.arena, r.arena, "on the roost");
    assert!(w.hunting(), "the roost is awake");
    let before = w.valley.cairn[0];
    assert_ne!(before, k as u16 + 1, "already on the top cairn");
    w.players[0].pos = V3::new(at.0, at.1, at.2).sub(w.map_origin());
    w.players[0].vel = V3::ZERO;
    for _ in 0..3 {
        w.advance([Input::new(0), Input::new(0)]);
    }
    assert!(w.hunting(), "the roost is still awake");
    assert_eq!(
        w.valley.cairn[0],
        k as u16 + 1,
        "the top cairn did not count"
    );
    w.players[0].health = 100;
    w.advance([Input::new(0), Input::new(0)]);
    assert!(
        w.players[0].health <= 100,
        "a cairn rested somebody mid-fight"
    );
}
