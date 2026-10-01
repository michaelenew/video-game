//! Arenas as data (`docs/design/arenas.md`).
//!
//! Three things are checked here. The proving ground is the `const` arena it
//! replaced, number for number -- the pinned hunts check the same thing from
//! the other end, by hashing whole fights. Every registered arena is well
//! formed. And the queries answer for an arena that is nothing like the first
//! one: the range, with its 240 m of floor, its 12 m tower, its cave under a
//! vault and its five floor materials.

use sim::arena::{self, Arena, ArenaId, Material};
use sim::input::{Input, Travel};
use sim::species::{self, SpeciesId};
use sim::state::MAX_PLAYERS;
use sim::{Class, Fx, V3, World};

fn m(v: i32) -> Fx {
    Fx::from_int(v)
}

fn at(x: i32, y: i32, z: i32) -> V3 {
    V3::new(m(x), m(y), m(z))
}

fn range() -> &'static Arena {
    ArenaId::RANGE.get()
}

// ---------------------------------------------------------------------------
// The proving ground is the arena it was
// ---------------------------------------------------------------------------

/// The numbers `arena.rs` had as constants before arenas were data, written
/// the way it wrote them.
#[test]
fn the_proving_ground_is_the_old_constant_arena() {
    let r = |n: i32, d: i32| Fx::ratio(n, d);
    let v = |x: (i32, i32), y: (i32, i32), z: (i32, i32)| {
        V3::new(r(x.0, x.1), r(y.0, y.1), r(z.0, z.1))
    };
    let old = [
        (v((-15, 1), (0, 1), (-15, 1)), v((-14, 1), (3, 2), (15, 1))),
        (v((14, 1), (0, 1), (-15, 1)), v((15, 1), (3, 2), (15, 1))),
        (v((-15, 1), (0, 1), (-15, 1)), v((15, 1), (3, 2), (-14, 1))),
        (v((-15, 1), (0, 1), (14, 1)), v((15, 1), (3, 2), (15, 1))),
        (v((-9, 1), (0, 1), (-4, 1)), v((-5, 1), (3, 2), (4, 1))),
        (v((5, 1), (0, 1), (-4, 1)), v((9, 1), (3, 2), (4, 1))),
    ];
    let ground = ArenaId::PROVING_GROUND.get();
    assert_eq!(ground.solids().len(), old.len());
    for (solid, (min, max)) in ground.solids().iter().zip(old) {
        assert_eq!((solid.min, solid.max), (min, max));
    }
    let b = ground.bounds;
    assert_eq!(
        (b.lo_x, b.hi_x, b.lo_z, b.hi_z),
        (m(-14), m(14), m(-14), m(14))
    );
    assert_eq!(arena::proving_ground::wall_height(), r(3, 2));
    // The versus marks: four metres either side of the middle, facing in.
    let [one, two] = ground.spawns.versus;
    assert_eq!((one.at, one.facing), (at(-4, 0, 0), at(1, 0, 0)));
    assert_eq!((two.at, two.facing), (at(4, 0, 0), at(-1, 0, 0)));
    // And a hunt is placed by the species, as it always was.
    assert!(ground.spawns.hunt.is_none());
    assert_eq!(
        arena::for_species(SpeciesId::RIDGEBACK).id,
        ArenaId::PROVING_GROUND
    );
}

#[test]
fn versus_and_the_ridgeback_are_in_the_proving_ground() {
    let classes = [Class::Bulwark; MAX_PLAYERS];
    assert_eq!(World::with_classes(classes).arena, ArenaId::PROVING_GROUND);
    assert_eq!(World::hunt(classes).arena, ArenaId::PROVING_GROUND);
    assert_eq!(World::new().arena().id, ArenaId::PROVING_GROUND);
}

// ---------------------------------------------------------------------------
// Every registered arena is well formed
// ---------------------------------------------------------------------------

#[test]
fn every_registered_arena_is_well_formed() {
    let mut seen = 0;
    for a in arena::all() {
        seen += 1;
        assert_eq!(arena::lookup(a.id).map(|x| x.id), Some(a.id), "{}", a.name);
        assert_eq!(
            arena::named(&a.slug()).map(|x| x.id),
            Some(a.id),
            "{}",
            a.name
        );
        assert!(
            a.solids.len() <= arena::MAX_SOLIDS,
            "{} has {} solids, over the {} every query is budgeted for",
            a.name,
            a.solids.len(),
            arena::MAX_SOLIDS
        );
        assert!(a.regions.len() <= arena::MAX_REGIONS, "{}", a.name);
        let b = a.bounds;
        assert!(
            b.lo_x.raw() < b.hi_x.raw() && b.lo_z.raw() < b.hi_z.raw(),
            "{}",
            a.name
        );
        for s in a.solids() {
            assert!(
                s.min.x.raw() < s.max.x.raw()
                    && s.min.y.raw() < s.max.y.raw()
                    && s.min.z.raw() < s.max.z.raw(),
                "{}: a solid with no volume: {s:?}",
                a.name
            );
        }
        // Everybody starts inside the arena, on the ground under their mark
        // -- the floor, or the top of what it stands on (the Cliffs'
        // plateau) -- and out of the walls: a body put there is not moved.
        let mut marks: Vec<_> = a.spawns.versus.to_vec();
        if let Some(h) = a.spawns.hunt {
            marks.extend(h.hunters);
            marks.extend(h.creatures);
        }
        for mark in marks {
            assert!(a.inside(mark.at), "{}: a mark outside it: {mark:?}", a.name);
            let ground = a.ground_under(mark.at);
            let stood = V3::new(mark.at.x, ground, mark.at.z);
            let r = a.resolve(stood, V3::ZERO, true);
            assert!(
                r.pos == stood && r.grounded,
                "{}: a mark in a solid: {mark:?} -> {:?}",
                a.name,
                r.pos
            );
            let unit = mark.facing.len();
            assert!(
                unit.sub(Fx::ONE).abs().raw() < 64 && mark.facing.y == Fx::ZERO,
                "{}: a mark facing {:?}",
                a.name,
                mark.facing
            );
        }
        // The creature it names is registered, and finds it.
        if let Some(s) = a.creature {
            assert!(
                species::lookup(s).is_some(),
                "{} names an unregistered creature",
                a.name
            );
            // A creature may have more than one arena (the Hornback's
            // crossing is its defend variant); the one it is hunted in by
            // default is the first that names it, and names it.
            assert_eq!(arena::for_species(s).creature, Some(s), "{}", a.name);
            assert!(arena::for_species(s).id.0 <= a.id.0, "{}", a.name);
        }
    }
    assert!(seen >= 2, "the proving ground and the range, at least");
}

/// An id with no arena reads as the proving ground rather than a crash in the
/// middle of a rollback.
#[test]
fn an_unregistered_arena_is_the_proving_ground() {
    // An id nobody will ever register: the creature arenas are being
    // filled in one branch at a time.
    assert!(arena::lookup(ArenaId(200)).is_none());
    assert_eq!(ArenaId(200).get().id, ArenaId::PROVING_GROUND);
    let w = World::hunt_in(
        [Class::Bulwark; MAX_PLAYERS],
        [Some(SpeciesId::RIDGEBACK), None],
        ArenaId(200),
    );
    assert_eq!(w.arena, ArenaId::PROVING_GROUND);
}

// ---------------------------------------------------------------------------
// The queries, in an arena nothing like the first
// ---------------------------------------------------------------------------

#[test]
fn the_range_has_the_size_the_largest_creature_asks_for() {
    let (wide, deep) = range().bounds.size();
    assert_eq!((wide, deep), (m(240), m(50)), "the Siegeshell's valley");
    assert!(range().inside(at(-119, 0, 24)));
    assert!(range().inside(at(119, 30, -24)));
    assert!(!range().inside(at(121, 1, 0)));
    assert!(!range().inside(at(0, 1, 26)));
    assert!(!range().inside(V3::new(m(0), Fx::from_raw(-1), m(0))));
    // The proving ground's bound is not the range's.
    assert!(!ArenaId::PROVING_GROUND.get().inside(at(60, 1, 0)));
}

#[test]
fn ground_under_reads_a_tall_solid_and_not_a_ceiling() {
    // The tower's flat top is twelve metres up, from anywhere above it or
    // beside it in its footprint.
    assert_eq!(range().ground_under(at(20, 0, 0)), m(12));
    assert_eq!(range().ground_under(at(20, 30, 0)), m(12));
    // A trunk is five.
    assert_eq!(
        range().ground_under(V3::new(m(-36), m(3), Fx::ratio(-8, 1))),
        m(5)
    );
    // In the cave the vault is overhead, not underfoot: the floor is the
    // floor. Above the vault, its top is the ground.
    assert_eq!(range().ground_under(at(60, 0, 0)), Fx::ZERO);
    assert_eq!(range().ground_under(at(60, 6, 0)), Fx::ZERO);
    assert_eq!(range().ground_under(at(60, 20, 0)), m(14));
    // A pillar stands on the floor, so its top counts from anywhere.
    assert_eq!(range().ground_under(at(55, 1, -7)), m(12));
}

#[test]
fn a_body_lands_on_the_tower_and_is_held_under_the_vault() {
    let a = range();
    // Falling onto the tower's top.
    let r = a.resolve(
        V3::new(m(20), Fx::ratio(239, 20), m(0)),
        at(0, -5, 0),
        false,
    );
    assert!(r.grounded, "did not land on the tower");
    assert_eq!(r.pos.y, m(12));
    // Jumping into the vault: the head stops at its underside and the climb
    // stops with it.
    let height = sim::tuning::body_height();
    let r = a.resolve(
        V3::new(m(60), m(12).sub(height).add(Fx::ratio(1, 4)), m(0)),
        at(0, 8, 0),
        false,
    );
    assert_eq!(r.pos.y.add(height), m(12), "the head went into the vault");
    assert_eq!(r.vel.y, Fx::ZERO);
    assert!(!r.grounded);
    // Walking into a trunk is a wall.
    let r = a.resolve(
        V3::new(Fx::ratio(-361, 10), m(0), Fx::ratio(-8, 1)),
        at(5, 0, 0),
        true,
    );
    assert!(r.wall, "walked through a trunk");
}

#[test]
fn the_floor_says_what_it_is_made_of() {
    let a = range();
    assert_eq!(a.material_under(at(-100, 0, 0)), Material::Ground);
    assert_eq!(a.material_under(at(-60, 0, 20)), Material::Sand);
    assert!(a.material_under(at(-60, 0, 20)).soft());
    // An island's top is rock, which a burrower cannot pass under.
    let island = V3::new(m(-58), Fx::ratio(1, 2), m(7));
    assert_eq!(a.material_under(island), Material::Rock);
    assert!(!a.material_under(island).soft());
    // But the floor *at* the island, ignoring the solid, is the sand round it.
    assert_eq!(a.floor_at(m(-58), m(7)), Material::Sand);
    assert_eq!(a.material_under(at(-10, 0, -20)), Material::Snow);
    assert!(a.material_under(at(-10, 0, -20)).takes_prints());
    // A later region wins: the ash pit is in the snow.
    assert_eq!(a.material_under(at(-30, 0, -12)), Material::Ash);
    assert_eq!(a.material_under(at(-20, 0, 0)), Material::Water);
    assert_eq!(a.material_under(at(60, 0, 10)), Material::Rock);
    assert_eq!(a.material_under(at(100, 0, 20)), Material::Grass);
    // On a trunk's top, wood; beside it on the snow, snow.
    assert_eq!(
        a.material_under(V3::new(m(-36), m(5), Fx::ratio(-8, 1))),
        Material::Wood
    );
    // The proving ground is ground all over.
    assert_eq!(
        ArenaId::PROVING_GROUND.get().material_under(at(3, 0, 3)),
        Material::Ground
    );
}

// ---------------------------------------------------------------------------
// The world in another arena
// ---------------------------------------------------------------------------

#[test]
fn a_hunt_in_the_range_stands_on_its_marks() {
    let w = World::hunt_in(
        [Class::Bulwark; MAX_PLAYERS],
        [Some(SpeciesId::RIDGEBACK), None],
        ArenaId::RANGE,
    );
    let marks = range().spawns.hunt.expect("the range places a hunt");
    assert_eq!(w.players[0].pos, marks.hunters[0].at);
    assert_eq!(w.players[1].pos, marks.hunters[1].at);
    let beast = w.monster().expect("a creature");
    assert_eq!(beast.pos, marks.creatures[0].at);
    let versus = World::versus_in([Class::Bulwark; MAX_PLAYERS], ArenaId::RANGE);
    assert_eq!(versus.players[0].pos, range().spawns.versus[0].at);
    assert!(!versus.hunting());
}

#[test]
fn a_creature_is_kept_inside_its_arena() {
    let mut w = World::hunt_in(
        [Class::Bulwark; MAX_PLAYERS],
        [Some(SpeciesId::RIDGEBACK), None],
        ArenaId::RANGE,
    );
    // Far out on the range's floor it is left where it is -- the proving
    // ground's fourteen metres are not this arena's walls -- and past the end
    // wall it is put back inside this one's.
    w.monster_mut().unwrap().pos = at(100, 0, 0);
    w.advance([Input::default(); MAX_PLAYERS]);
    let pos = w.monster().unwrap().pos;
    assert!(
        pos.x.raw() > m(90).raw(),
        "it was clamped to the proving ground: {pos:?}"
    );
    w.monster_mut().unwrap().pos = at(300, 0, 0);
    w.advance([Input::default(); MAX_PLAYERS]);
    let pos = w.monster().unwrap().pos;
    assert!(range().inside(pos), "it left: {pos:?}");
    assert!(
        pos.x.raw() > m(100).raw(),
        "it was clamped to the proving ground: {pos:?}"
    );
}

#[test]
fn the_arena_is_in_the_hash_and_a_restart_keeps_it() {
    let classes = [Class::Bulwark; MAX_PLAYERS];
    let here = World::versus_in(classes, ArenaId::PROVING_GROUND);
    let there = World::versus_in(classes, ArenaId::RANGE);
    assert_eq!(here.checksum(), World::with_classes(classes).checksum());
    assert_ne!(here.checksum(), there.checksum());
    assert_eq!(there.restarted(classes).arena, ArenaId::RANGE);
    let hunt = World::hunt_in(classes, [Some(SpeciesId::RIDGEBACK), None], ArenaId::RANGE);
    let again = hunt.restarted([Class::Champion; MAX_PLAYERS]);
    assert_eq!(again.arena, ArenaId::RANGE);
    assert_eq!(
        again.monster().map(|m| m.species),
        Some(SpeciesId::RIDGEBACK)
    );
}

#[test]
fn a_fight_in_the_range_is_deterministic() {
    let run = || {
        let mut w = World::hunt_in(
            [Class::Elementalist, Class::ShadowReaver],
            [Some(SpeciesId::RIDGEBACK); 2],
            ArenaId::RANGE,
        );
        let mut rng = 0x0bad_5eed_u64;
        for _ in 0..900 {
            let inputs = std::array::from_fn(|_| {
                rng ^= rng << 13;
                rng ^= rng >> 7;
                rng ^= rng << 17;
                Input::looking_at(
                    (rng & 0x7fff) as u16,
                    (rng >> 16) as u16,
                    ((rng >> 32) as i16) / 8,
                )
            });
            w.advance(inputs);
        }
        w.checksum()
    };
    assert_eq!(run(), run());
}

// ---------------------------------------------------------------------------
// The picker, as the simulation sees it
// ---------------------------------------------------------------------------

#[test]
fn a_trip_is_a_fresh_fight_on_the_same_frame() {
    let mut w = World::with_classes([Class::Champion, Class::BloodMage]);
    for _ in 0..30 {
        w.advance([Input::new(Input::W); MAX_PLAYERS]);
    }
    let ask = Input::default().travelling(Travel::hunt(SpeciesId::RIDGEBACK));
    w.advance([Input::default(), ask]);
    assert_eq!(w.frame, 31, "the frame number belongs to the session");
    assert!(w.hunting());
    assert_eq!(w.arena, ArenaId::PROVING_GROUND);
    assert_eq!(
        w.players.map(|p| p.class),
        [Class::Champion, Class::BloodMage]
    );
    let mut fresh = World::hunt([Class::Champion, Class::BloodMage]);
    fresh.frame = 31;
    assert_eq!(w.checksum(), fresh.checksum(), "not a fresh fight");

    w.advance([
        Input::default().travelling(Travel::VERSUS),
        Input::default(),
    ]);
    assert!(!w.hunting());
    assert_eq!(w.frame, 32);
}

#[test]
fn a_trip_to_an_unregistered_creature_goes_nowhere() {
    let mut w = World::with_classes([Class::Bulwark; MAX_PLAYERS]);
    let mut same = w.clone();
    // An id the travel byte carries (five bits) that nobody will ever
    // register: past every planned creature.
    let ask = Input::default().travelling(Travel::hunt(SpeciesId(30)));
    w.advance([ask, Input::default()]);
    same.advance([Input::default(); MAX_PLAYERS]);
    assert_eq!(w.checksum(), same.checksum());
}

#[test]
fn the_cycle_skips_creatures_that_are_not_registered() {
    for id in 0..species::COUNT as u8 {
        let next = species::after(SpeciesId(id));
        assert!(species::lookup(next.id).is_some());
    }
    // With the Ridgeback the only one, every step comes back to it.
    if species::all().count() == 1 {
        assert_eq!(
            species::after(SpeciesId::RIDGEBACK).id,
            SpeciesId::RIDGEBACK
        );
    }
}

// ---------------------------------------------------------------------------
// The ground as it stands, ceilings, and long distances (F3b)
// ---------------------------------------------------------------------------

#[test]
fn bare_ground_answers_exactly_as_the_arena_does() {
    for a in arena::all() {
        let ground = arena::Terrain::bare(a);
        for x in (-120..=120).step_by(7) {
            for z in (-25..=25).step_by(5) {
                for y in [0, 1, 3, 8, 13] {
                    let p = at(x, y, z);
                    assert_eq!(ground.ground_under(p), a.ground_under(p));
                    assert_eq!(ground.material_under(p), a.material_under(p));
                    let v = at(3, -2, -1);
                    let (r1, r2) = (ground.resolve(p, v, true), a.resolve(p, v, true));
                    assert_eq!(
                        (r1.pos, r1.vel, r1.grounded, r1.wall),
                        (r2.pos, r2.vel, r2.grounded, r2.wall)
                    );
                }
            }
        }
    }
}

#[test]
fn the_eye_is_held_under_the_cave_vault_and_nowhere_else() {
    use sim::camera::{eye, eye_under};
    use sim::tuning as t;
    // Under the cave's vault (12 m), and under its 8 m lintel by the wall.
    for (x, z) in [(60, 0), (60, 21)] {
        let pos = at(x, 0, z);
        let ceiling = range()
            .ceiling_over(m(x), m(z), t::body_height())
            .expect("a vault");
        for pitch in (-90..=60).step_by(10) {
            for aim in [0u16, 16384, 32768, 49152] {
                let look = Input::looking_at(0, aim, (pitch * 182) as i16);
                for aloft in [Fx::ZERO, Fx::ONE] {
                    let held = eye_under(pos, look, aloft, range());
                    let free = eye(pos, look, aloft);
                    let cap = ceiling.sub(t::eye_under_ceiling());
                    assert!(
                        held.y.raw() <= cap.raw() || held.y == pos.y.add(t::body_height()),
                        "the eye at {held:?} is in the vault at {ceiling:?}"
                    );
                    assert!(held.y.raw() <= free.y.raw());
                    assert_eq!((held.x, held.z), (free.x, free.z));
                }
            }
        }
    }
    // Anywhere without a ceiling over it -- all of the proving ground -- it
    // is the eye exactly.
    let pg = ArenaId::PROVING_GROUND.get();
    for x in (-13..=13).step_by(3) {
        for pitch in (-90..=60).step_by(15) {
            let look = Input::looking_at(0, 12345, (pitch * 182) as i16);
            let pos = at(x, 0, x / 2);
            assert_eq!(eye_under(pos, look, Fx::ZERO, pg), eye(pos, look, Fx::ZERO));
        }
    }
    // And the aiming ray out of it starts under the rock: a shot straight up
    // from the cave floor meets the vault's underside, not its top.
    let mut w = World::versus_in([Class::Champion; MAX_PLAYERS], ArenaId::RANGE);
    w.players[0].pos = at(60, 0, 0);
    let ground = w.terrain();
    let fighters = w.players;
    let field = sim::stones::gather(&fighters);
    let scene = sim::aim::Scene {
        stones: &field,
        players: &fighters,
        effects: &w.effects,
        quarry: &w.monsters,
        critters: &w.critters,
        arena: &ground,
    };
    let up = Input::looking_at(0, 0, 60 * 182);
    let seen = sim::aim::sight(0, up, m(60), &scene);
    assert!(seen.at.y.raw() <= m(12).raw(), "the ray met {:?}", seen.at);
}

#[test]
fn lengths_across_the_range_do_not_saturate() {
    use sim::math::{wide_flat_dist, wide_len, wide_normalized};
    let (a, b) = (at(-118, 0, -20), at(118, 0, 20));
    let far = wide_flat_dist(a, b);
    assert!(
        far.sub(m(239)).abs().raw() < Fx::ratio(5, 10).raw(),
        "{far:?}"
    );
    assert!(
        b.sub(a).flat_len().raw() < m(182).raw(),
        "16.16 alone saturates"
    );
    let dir = wide_normalized(b.sub(a));
    assert!(dir.len().sub(Fx::ONE).abs().raw() < Fx::ratio(1, 100).raw());
    // Under a hundred metres a component, exactly the short helpers.
    for (x, y, z) in [(3, 1, -4), (99, 0, 99), (-60, 12, 70), (0, 0, 0)] {
        let v = at(x, y, z);
        assert_eq!(wide_len(v), v.len());
        assert_eq!(wide_normalized(v), v.normalized());
    }
}
