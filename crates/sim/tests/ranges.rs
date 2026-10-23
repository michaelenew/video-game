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
///
/// A herd that roams wakes as you come near it, wherever it is: walked up to,
/// then -- stood five metres inside the distance that wakes it.
fn walked_in(r: &Range) -> World {
    let mut w = if r.roams() {
        let mid = sim::valley::sight::herd_middle(r, 0);
        let back = cm(ranges::WAKE - 500);
        let (x, z) = (mid.x, mid.z.sub(back));
        let y = sim::atlas::valley().relief_at(x, z);
        standing(r, (x, y.add(Fx::ratio(1, 2)), z))
    } else {
        let (x, z) = (cm(r.at.0), cm(r.at.1 - r.half + 250));
        standing(r, (x, cm(r.floor).add(Fx::ratio(1, 2)), z))
    };
    for _ in 0..3 {
        w.advance([Input::new(0), Input::new(0)]);
    }
    w
}

/// **Where a range's creatures are**, in the world's coordinates: the middle
/// of the live ones -- or, with none, the range's middle.
fn herd_mid(w: &World, r: &Range) -> V3 {
    let bodies: Vec<V3> = w
        .monsters
        .iter()
        .flatten()
        .filter(|m| m.alive())
        .map(|m| m.pos)
        .chain(w.critters.iter().filter(|c| c.alive()).map(|c| c.pos))
        .collect();
    if bodies.is_empty() {
        return V3::new(cm(r.at.0), cm(r.floor), cm(r.at.1)).sub(w.map_origin());
    }
    let n = bodies.len() as i64;
    let sx: i64 = bodies.iter().map(|p| p.x.raw() as i64).sum();
    let sz: i64 = bodies.iter().map(|p| p.z.raw() as i64).sum();
    V3::new(
        Fx::from_raw((sx / n) as i32),
        bodies[0].y,
        Fx::from_raw((sz / n) as i32),
    )
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
                // A herd that roams is on the road on purpose: it is woken by
                // coming near it, not by the ground (`every_roaming_herd_...`).
                for r in RANGES.iter().filter(|r| r.stacked.is_none() && !r.roams()) {
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
    let mid = herd_mid(&w, r);
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
    let mid = herd_mid(&w, r);
    let inside = V3::new(mid.x, mid.y, mid.z.sub(cm(ring / 2)));
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
    let mid = herd_mid(&w, r);
    hurt(&mut w);
    assert!(stock(&w) < before);
    for _ in 0..2 {
        w.advance([Input::new(0), Input::new(0)]);
    }
    let pack = w.pack.expect("the herd");
    assert_eq!(pack.mood, sim::pack::mood::BROKEN, "it did not run");
    // Its home is the far side from her.
    let me = w.players[0].pos.sub(mid);
    assert!(
        pack.home.sub(mid).dot(me).raw() < 0,
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
    // One that keeps to its ground, woken by walking on to it.
    let r = RANGES
        .iter()
        .find(|r| r.stacked.is_none() && !r.roams())
        .expect("a range that keeps to its ground");
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
        let mut at = if r.roams() {
            herd_mid(&w, r)
        } else {
            V3::new(cm(r.at.0), cm(r.floor).add(Fx::ONE), cm(r.at.1)).sub(o)
        };
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

// ---------------------------------------------------------------------------
// Sightings (2026-10-22): where a sleeping range's creatures are
// ---------------------------------------------------------------------------

/// **A sleeping herd is where its frame says**: the same frame stands it in
/// the same place every time, it moves from frame to frame, and over a long
/// while every body of it stays inside its range.
#[test]
fn a_sleeping_herd_wanders_its_range_by_the_frame_alone() {
    use sim::valley::sight;
    let atlas = sim::atlas::valley();
    for r in RANGES.iter().filter(|r| r.stacked.is_none()) {
        let base = sight::base(r.arena).expect("a range has creatures");
        let place = atlas.placed(r.arena).copied().unwrap();
        let ground = sim::arena::Terrain::placed(atlas, r.arena.get());
        let at = |frame: u32| {
            let mut s = base;
            sight::stand(&mut s, frame, V3::ZERO, &ground);
            s
        };
        let bodies = |s: &sight::Sight| -> Vec<V3> {
            s.monsters
                .iter()
                .flatten()
                .map(|m| m.pos)
                .chain(s.critters.iter().filter(|c| c.present()).map(|c| c.pos))
                .collect()
        };
        assert_eq!(
            bodies(&at(4321)),
            bodies(&at(4321)),
            "{}: not a function of the frame",
            r.name
        );
        let mut moved = false;
        for k in 0..60u32 {
            let f = k * 397 + 11;
            let now = bodies(&at(f));
            moved |= now != bodies(&at(f + sight::LEG / 2));
            for p in now {
                let m = p.add(place.at);
                assert!(
                    place.holds(m.x, m.z),
                    "{}: frame {f}, a body at ({}, {}) out of its range",
                    r.name,
                    m.x.to_f32_for_render(),
                    m.z.to_f32_for_render()
                );
            }
        }
        assert!(moved, "{}: its herd never moved", r.name);
    }
}

/// **A range wakes where it was seen**: the creatures a hunt starts with are
/// exactly where the sighting stood them on that frame -- the animal watched
/// from the road is the one that turns to face you.
#[test]
fn a_range_wakes_its_creatures_where_they_were_seen() {
    use sim::valley::sight;
    for r in RANGES.iter().filter(|r| r.stacked.is_none()) {
        let w = World::arrive(
            [Class::Champion, Class::Elementalist],
            journey(),
            r.arena,
            None,
            1,
        );
        assert!(w.hunting(), "{}: nothing woke", r.name);
        let mut seen = sight::base(r.arena).unwrap();
        sight::stand(&mut seen, w.frame, V3::ZERO, &w.terrain());
        let flat = |p: V3| (p.x, p.z);
        let woke: Vec<_> = w.monsters.iter().flatten().map(|m| flat(m.pos)).collect();
        let was: Vec<_> = seen
            .monsters
            .iter()
            .flatten()
            .map(|m| flat(m.pos))
            .collect();
        assert_eq!(woke, was, "{}: its bodies woke somewhere else", r.name);
        let woke: Vec<_> = w
            .critters
            .iter()
            .filter(|c| c.present())
            .map(|c| flat(c.pos))
            .collect();
        let was: Vec<_> = seen
            .critters
            .iter()
            .filter(|c| c.present())
            .map(|c| flat(c.pos))
            .collect();
        assert_eq!(woke, was, "{}: its pack woke somewhere else", r.name);
    }
}

/// **Each range stands in the place `ranges::IN` says**, which is the one
/// whose air and colours it takes.
#[test]
fn every_range_is_in_the_place_it_says() {
    for (k, r) in RANGES.iter().enumerate() {
        let want = match ranges::IN[k] {
            // Stacked over another place: the roost over the Needle, the
            // Deep's levels over its own pit.
            id if r.stacked.is_some() => id,
            id => {
                assert_eq!(
                    reach_of(r),
                    id,
                    "{} stands in {}",
                    r.name,
                    reach_of(r).get().name
                );
                id
            }
        };
        assert_eq!(ranges::in_place(r.arena), Some(want));
    }
}

/// **No range shares its id with another arena**: each range's id is its
/// own arena, by name. Three ranges added on 2026-10-22 were first numbered
/// on into the versus stages' ids, and the Low Meadow came out the Caldera.
#[test]
fn every_range_has_an_id_of_its_own() {
    let mut seen = std::collections::BTreeSet::new();
    for r in &RANGES {
        assert_eq!(
            r.arena.get().name,
            r.name,
            "{}'s id is another arena's",
            r.name
        );
        assert_eq!(ranges::of(r.arena).map(|q| q.name), Some(r.name));
        assert!(seen.insert(r.arena.0), "{} shares an id", r.name);
    }
    for a in sim::arena::all() {
        if ranges::of(a.id).is_none() {
            assert!(!seen.contains(&a.id.0), "{} has a range's id", a.name);
        }
    }
}

// ---------------------------------------------------------------------------
// Roaming (2026-10-22): herds that walk the valley floor, not a pocket
// ---------------------------------------------------------------------------

/// **A roaming herd's loop is open ground in its territory**: every metre of
/// every leg, for its middle and either side of it where its bodies walk, is
/// inside its territory, dry, never steeper than a body walks, and clear of
/// anything taller than a step -- a tree, a boulder, a crag.
#[test]
fn every_roaming_herd_walks_open_ground() {
    let atlas = sim::atlas::valley();
    let land = atlas.land.as_ref().unwrap();
    let mut bad = Vec::new();
    for r in RANGES.iter().filter(|r| r.roams()) {
        let home = atlas.placed(r.arena).copied().unwrap();
        let n = r.walk.len();
        for i in 0..n {
            let (a, b) = (r.walk[i], r.walk[(i + 1) % n]);
            let len = (((b.0 - a.0) as f64).hypot((b.1 - a.1) as f64) / 100.0) as i32 + 1;
            for side in [-r.half / 4, 0, r.half / 4] {
                let mut last: Option<Fx> = None;
                for k in 0..=len {
                    let x = a.0 + (b.0 - a.0) * k / len + side;
                    let z = a.1 + (b.1 - a.1) * k / len + r.half / 3;
                    let (fx, fz) = (cm(x), cm(z));
                    let where_ = format!("{}: leg {i} at ({}, {})", r.name, x / 100, z / 100);
                    if !home.holds(fx, fz) {
                        bad.push(format!("{where_}: out of its territory"));
                        break;
                    }
                    if land.sample(fx, fz).water.is_some() {
                        bad.push(format!("{where_}: in water"));
                        break;
                    }
                    let y = atlas.relief_at(fx, fz);
                    if let Some(l) = last
                        && y.sub(l).abs().raw() > sim::tuning::terrain_steepest().raw()
                    {
                        bad.push(format!("{where_}: too steep"));
                        break;
                    }
                    last = Some(y);
                    let (lo, hi) = (
                        (fx.sub(Fx::ONE), fz.sub(Fx::ONE)),
                        (fx.add(Fx::ONE), fz.add(Fx::ONE)),
                    );
                    let blocked = atlas.near(lo, hi).find(|&j| {
                        let s = &atlas.solids[j as usize];
                        s.min.x.raw() < hi.0.raw()
                            && s.max.x.raw() > lo.0.raw()
                            && s.min.z.raw() < hi.1.raw()
                            && s.max.z.raw() > lo.1.raw()
                            && s.max.y.raw() > y.add(Fx::ONE).raw()
                            && s.min.y.raw() < y.add(Fx::from_int(4)).raw()
                    });
                    if let Some(j) = blocked {
                        let s = &atlas.solids[j as usize];
                        bad.push(format!(
                            "{where_}: {:?} from ({:.1}, {:.1}) to ({:.1}, {:.1}) stands there",
                            s.material,
                            s.min.x.to_f32_for_render(),
                            s.min.z.to_f32_for_render(),
                            s.max.x.to_f32_for_render(),
                            s.max.z.to_f32_for_render()
                        ));
                        break;
                    }
                }
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// **A roaming range's territory is the valley's, and no one else's**: it
/// overlaps no other place but the reach it stands in.
#[test]
fn a_territory_overlaps_no_other_place() {
    let atlas = sim::atlas::valley();
    for (k, r) in RANGES.iter().enumerate().filter(|(_, r)| r.roams()) {
        let me = atlas.placed(r.arena).copied().unwrap();
        for p in &atlas.places {
            if p.arena == r.arena || p.arena == ranges::IN[k] {
                continue;
            }
            if matches!(
                valley::place(p.arena).map(|q| q.kind),
                Some(valley::Kind::Reach)
            ) {
                continue;
            }
            let apart = me.hi.0.raw() <= p.lo.0.raw()
                || p.hi.0.raw() <= me.lo.0.raw()
                || me.hi.1.raw() <= p.lo.1.raw()
                || p.hi.1.raw() <= me.lo.1.raw();
            assert!(
                apart,
                "{}'s territory overlaps {}",
                r.name,
                p.arena.get().name
            );
        }
    }
}

/// **A roaming herd is woken by coming near it, not by its ground**: stood
/// in its territory far from it, nothing wakes; within `ranges::WAKE` of it,
/// it does; and a herd nobody roused sleeps again -- back on its loop -- once
/// everybody is `ranges::RELEASE` from it.
#[test]
fn a_roaming_herd_wakes_when_you_come_near_and_sleeps_when_you_go() {
    let r = RANGES.iter().find(|r| r.name == "Riverside").unwrap();
    assert!(r.roams());
    let mut w = World::arrive(
        [Class::Champion, Class::Elementalist],
        journey(),
        ArenaId::MOUTH,
        None,
        1,
    );
    let land = |x: Fx, z: Fx| sim::atlas::valley().relief_at(x, z);
    let put = |w: &mut World, x: Fx, z: Fx| {
        let o = w.map_origin();
        w.players[0].pos = V3::new(x, land(x, z).add(Fx::ratio(1, 2)), z).sub(o);
        w.players[0].vel = V3::ZERO;
    };
    // In its territory, but well away from where it grazes.
    let home = sim::atlas::valley().placed(r.arena).copied().unwrap();
    let mid = sim::valley::sight::herd_middle(r, w.frame);
    let corners = [
        (
            home.lo.0.add(Fx::from_int(3)),
            home.lo.1.add(Fx::from_int(3)),
        ),
        (
            home.hi.0.sub(Fx::from_int(3)),
            home.lo.1.add(Fx::from_int(3)),
        ),
        (
            home.lo.0.add(Fx::from_int(3)),
            home.hi.1.sub(Fx::from_int(3)),
        ),
        (
            home.hi.0.sub(Fx::from_int(3)),
            home.hi.1.sub(Fx::from_int(3)),
        ),
    ];
    let far = corners
        .iter()
        .copied()
        .max_by_key(|&(x, z)| sim::math::wide_flat_dist(V3::new(x, Fx::ZERO, z), mid).raw())
        .unwrap();
    put(&mut w, far.0, far.1);
    for _ in 0..30 {
        w.advance([Input::new(0), Input::new(0)]);
    }
    assert!(!w.hunting(), "its ground woke it, far from it");
    // Near it.
    let mid = sim::valley::sight::herd_middle(r, w.frame);
    put(&mut w, mid.x, mid.z.sub(cm(ranges::WAKE - 800)));
    w.advance([Input::new(0), Input::new(0)]);
    assert!(
        w.hunting() && w.den_id() == r.arena,
        "coming near it did not wake it"
    );
    // Away again, nobody having touched it: it sleeps.
    let o = w.map_origin();
    let here = w.players[0].pos.add(o);
    put(&mut w, here.x, here.z.sub(cm(ranges::RELEASE * 2)));
    for _ in 0..10 {
        w.advance([Input::new(0), Input::new(0)]);
    }
    assert!(!w.hunting(), "it stayed awake with nobody near it");
}

/// **A cairn in a herd's territory still counts**: the Mouth's crag stands in
/// the Riverside herd's, and the cairn on its top is a checkpoint.
#[test]
fn a_cairn_in_a_territory_counts() {
    let atlas = sim::atlas::valley();
    let (x, z) = (cm(16000), cm(2600));
    let i = atlas
        .near((x, z), (x, z))
        .map(|i| i as usize)
        .filter(|&i| atlas.solids[i].material == sim::arena::Material::Snow)
        .max_by_key(|&i| atlas.solids[i].max.y.raw())
        .expect("the crag's cairn");
    let top = valley::top_of(&atlas.solids[i]);
    assert!(
        atlas
            .place_at(x, z)
            .is_some_and(|p| ranges::of(p.arena).is_some()),
        "the crag is not in a territory any more"
    );
    assert_eq!(valley::cairn_at(atlas, top), Some(i));
}

/// **A beaten roaming herd rests too**: coming near where it would be wakes
/// nothing until its rest is out, and nothing of it is drawn meanwhile (the
/// renderer asks `World::resting`).
#[test]
fn a_beaten_roaming_herd_rests_and_comes_back() {
    let r = RANGES.iter().find(|r| r.roams()).unwrap();
    let k = ranges::index(r.arena).unwrap();
    let mut w = walked_in(r);
    assert!(w.hunting());
    w.valley.rested[k] = w.frame;
    w.monsters = [None; sim::monster::MAX_MONSTERS];
    w.pack = None;
    w.critters = sim::critter::Critters::NONE;
    let near = |w: &mut World| {
        let mid = sim::valley::sight::herd_middle(r, w.frame);
        let y = sim::atlas::valley().relief_at(mid.x, mid.z);
        let o = w.map_origin();
        w.players[0].pos = V3::new(mid.x, y.add(Fx::ONE), mid.z).sub(o);
        w.advance([Input::new(0), Input::new(0)]);
    };
    near(&mut w);
    assert!(!w.hunting(), "a resting herd woke");
    w.frame = w.frame.wrapping_add(sim::tuning::range_rest_frames());
    near(&mut w);
    assert!(w.hunting(), "it never came back");
}
