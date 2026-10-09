//! **The valley** (`docs/design/valley.md`, `sim::valley`): the places join
//! up into one map you walk, a room's hunt starts when you walk into it,
//! waystones shut the climb, a death in a reach is a cairn, and the props do
//! what they say. The map itself -- tiles, doorways, nothing overlapping --
//! is `tests/atlas.rs`.

use sim::arena::{ArenaId, Terrain};
use sim::input::{Destination, Travel};
use sim::species::SpeciesId;
use sim::state::Phase;
use sim::valley::{self, Kind};
use sim::{Class, Fx, Input, V3, World};

const CLASSES: [Class; 2] = [Class::Champion, Class::Elementalist];

fn idle() -> [Input; 2] {
    [Input::new(0), Input::new(0)]
}

/// The world as a seam into `to` at `at` builds it, both seats played.
fn arrived(to: ArenaId, at: Option<u8>) -> World {
    World::arrive(CLASSES, valley::Journey::default(), to, at, 2)
}

/// Where a body would stand in a zone: its middle, on the highest top under
/// the zone's ceiling -- on the valley's map, the land's.
fn standing_in(zone: &valley::Zone, arena: ArenaId) -> V3 {
    let mid = zone.middle();
    let ground = Terrain::placed(sim::atlas::valley(), arena.get())
        .ground_under(V3::new(mid.x, zone.max.y, mid.z));
    V3::new(mid.x, ground, mid.z)
}

fn inside_a_solid(arena: ArenaId, p: V3) -> bool {
    let up = V3::new(p.x, p.y.add(Fx::ratio(1, 10)), p.z);
    let ground = Terrain::placed(sim::atlas::valley(), arena.get());
    ground.around(up, Fx::ONE).any(|s| {
        up.x.raw() > s.min.x.raw()
            && up.x.raw() < s.max.x.raw()
            && up.y.raw() > s.min.y.raw()
            && up.y.raw() < s.max.y.raw()
            && up.z.raw() > s.min.z.raw()
            && up.z.raw() < s.max.z.raw()
    })
}

#[test]
fn every_seam_leads_to_one_that_leads_back() {
    let mut seams = 0;
    for place in valley::all() {
        for (i, seam) in place.seams.iter().enumerate() {
            seams += 1;
            let there = valley::place(seam.to)
                .unwrap_or_else(|| panic!("{}: seam {i} leads nowhere", place.arena.get().name));
            let back = there.seam(seam.at).unwrap_or_else(|| {
                panic!(
                    "{}: seam {i} comes out of a seam {} has not got",
                    place.arena.get().name,
                    there.arena.get().name
                )
            });
            assert_eq!(
                back.to,
                place.arena,
                "{}: seam {i} and its pair disagree",
                place.arena.get().name
            );
            assert_eq!(
                back.at as usize,
                i,
                "{}: seam {i}'s pair comes back elsewhere",
                place.arena.get().name
            );
        }
    }
    assert!(seams >= 28, "only {seams} seams");
}

#[test]
fn every_seam_can_be_stood_in() {
    for place in valley::all() {
        let id = place.arena;
        for (i, seam) in place.seams.iter().enumerate() {
            let feet = standing_in(&seam.zone, id);
            assert!(
                seam.zone.holds(feet),
                "{}: nowhere to stand in seam {i} ({feet:?})",
                id.get().name
            );
            assert!(
                !inside_a_solid(id, feet),
                "{}: seam {i}'s middle is inside a solid",
                id.get().name
            );
        }
    }
}

#[test]
fn every_arrival_stands_on_footing_out_of_the_rock() {
    for place in valley::all() {
        let id = place.arena;
        let a = id.get();
        for (i, seam) in place.seams.iter().enumerate() {
            for m in seam.marks {
                let feet = V3::new(m.at.x, a.ground_under(m.at), m.at.z);
                let b = a.bounds;
                assert!(
                    feet.x.raw() > b.lo_x.raw()
                        && feet.x.raw() < b.hi_x.raw()
                        && feet.z.raw() > b.lo_z.raw()
                        && feet.z.raw() < b.hi_z.raw(),
                    "{}: seam {i}'s arrival is outside the bounds",
                    a.name
                );
                assert!(
                    !inside_a_solid(id, feet),
                    "{}: seam {i}'s arrival is inside a solid",
                    a.name
                );
                // Off a room's own way out, everywhere else, so arriving never
                // reads as standing in another seam.
                if !matches!(place.kind, Kind::Room(_)) {
                    for (j, other) in place.seams.iter().enumerate() {
                        assert!(
                            !other.zone.holds(feet),
                            "{}: seam {i}'s arrival stands in seam {j}",
                            a.name
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn every_waystone_is_a_solid_by_its_seam() {
    for place in valley::all() {
        for (i, seam) in place.seams.iter().enumerate() {
            let Some(w) = seam.waystone else { continue };
            let s = place.arena.get().solids()[w as usize];
            assert_eq!(
                s.material,
                sim::arena::Material::Stone,
                "{}: seam {i}'s waystone",
                place.arena.get().name
            );
            let mid = seam.zone.middle();
            let dx = (s.min.x.raw() / 2 + s.max.x.raw() / 2 - mid.x.raw()).abs();
            let dz = (s.min.z.raw() / 2 + s.max.z.raw() / 2 - mid.z.raw()).abs();
            assert!(
                dx < Fx::from_int(24).raw() && dz < Fx::from_int(24).raw(),
                "{}: seam {i}'s waystone is far from it",
                place.arena.get().name
            );
            assert!(
                !matches!(seam.gate, valley::Gate::Open),
                "{}: a waystone on an open seam",
                place.arena.get().name
            );
        }
    }
}

#[test]
fn the_valley_starts_in_hearth_at_peace_and_the_ring_is_versus() {
    let w = World::versus_in(CLASSES, valley::START);
    assert!(w.valley.on && w.peaceful() && !w.pvp());
    let ring = arrived(ArenaId::RING, Some(0));
    assert!(ring.valley.on && ring.pvp() && !ring.peaceful());
    // A reach on its own is the valley; a room on its own is not.
    assert!(World::versus_in(CLASSES, ArenaId::MOUTH).valley.on);
    assert!(!World::hunt_of(CLASSES, SpeciesId::HORNBACK).valley.on);
    // Outside the valley nothing changed: versus is versus.
    assert!(World::with_classes(CLASSES).pvp());
}

/// Every creature beaten: every waystone lit, as `--open` has it.
fn open_journey() -> valley::Journey {
    valley::Journey {
        beaten: u32::MAX,
        ..valley::Journey::default()
    }
}

/// The aim that walks a body along +x, +z, -x or -z: a quarter turn each.
fn heading(axis: usize, sign: i32) -> u16 {
    match (axis, sign > 0) {
        (0, true) => 0,
        (2, true) => Input::QUARTER_TURN,
        (0, false) => 2 * Input::QUARTER_TURN,
        _ => 3 * Input::QUARTER_TURN,
    }
}

/// A fighter's feet on the valley's map.
fn on_map(w: &World, i: usize) -> V3 {
    w.players[i].pos.add(w.map_origin())
}

/// **Walk fighter one to a point of the map**, steering at it every frame,
/// within `frames`. `Err` with where she stuck.
fn walk_to(w: &mut World, to: (Fx, Fx), frames: u32) -> Result<(), String> {
    for _ in 0..frames {
        let feet = on_map(w, 0);
        let (dx, dz) = (to.0.sub(feet.x), to.1.sub(feet.z));
        if sim::math::wide_len(V3::new(dx, Fx::ZERO, dz)).raw() < Fx::ratio(3, 2).raw() {
            return Ok(());
        }
        let aim = sim::math::atan2_turns(dz, dx).raw() as u16;
        w.advance([Input::looking_at(Input::W, aim, -1500), Input::new(0)]);
    }
    Err(format!(
        "stuck at {:?} on the map in {}, walking to ({}, {})",
        on_map(w, 0),
        w.arena().name,
        to.0.to_f32_for_render(),
        to.1.to_f32_for_render()
    ))
}

/// **The whole road can be walked**, from the town's east gate to the end of
/// the Saddle, every waystone lit: one fighter, steering from point to point
/// of the road, never more than twenty seconds between two of them. Every
/// floor meets the next, nothing stands across the road, no slope on it is
/// too steep, and the world is in the Saddle at the end.
#[test]
fn the_whole_road_can_be_walked() {
    let mut w = World::arrive(CLASSES, open_journey(), valley::START, Some(0), 1);
    let mut seen = vec![w.arena];
    for to in sim::valley::layout::road() {
        if let Err(e) = walk_to(&mut w, to, 1200) {
            panic!("{e}");
        }
        if seen.last() != Some(&w.arena) {
            seen.push(w.arena);
        }
    }
    assert_eq!(
        w.arena,
        ArenaId::SADDLE,
        "walked the road and ended in {}",
        w.arena().name
    );
    for id in [
        ArenaId::MOUTH,
        ArenaId::BANK,
        ArenaId::SHELVES,
        ArenaId::PINEWOOD,
        ArenaId::SADDLE,
    ] {
        assert!(seen.contains(&id), "never in {}", id.get().name);
    }
    assert!(w.players[0].health > 0, "the road killed her");
}

/// **Every room can be walked into** from its junction on the road, by its
/// side path (or bridge, or switchback), every waystone lit: and walking in
/// starts its hunt -- or, in the Ring, its fight.
#[test]
fn every_room_can_be_walked_into() {
    for r in &sim::valley::layout::ROOMS {
        let (reach, seam) = r.seam;
        let mut w = World::arrive(CLASSES, open_journey(), reach, Some(seam), 1);
        let path = sim::valley::layout::approach(r);
        for to in path {
            if let Err(e) = walk_to(&mut w, to, 1200) {
                panic!("into {}: {e}", r.arena.get().name);
            }
            if w.arena == r.arena {
                break;
            }
        }
        assert_eq!(
            w.arena,
            r.arena,
            "walked the way into {} and never arrived",
            r.arena.get().name
        );
        match valley::place(r.arena).map(|p| p.kind) {
            Some(Kind::Room(_)) => assert!(
                w.hunting(),
                "{}: walked in, and no hunt",
                r.arena.get().name
            ),
            Some(Kind::Ring) => assert!(w.pvp(), "the Ring is not a fight"),
            _ => {}
        }
    }
}

/// **The mountains are the edge**: walking straight off the road up a
/// mountainside gets as far as the slope allows and no further, and the
/// fighter never ends up standing on ground too steep to stand on.
#[test]
fn a_mountainside_is_a_wall() {
    let mut w = arrived(ArenaId::BANK, Some(0));
    let north = Input::looking_at(Input::W, heading(2, 1), 0);
    let start = on_map(&w, 0);
    for _ in 0..1200 {
        w.advance([north, Input::new(0)]);
    }
    let end = on_map(&w, 0);
    assert!(
        end.z.sub(start.z).raw() < Fx::from_int(90).raw(),
        "walked {} m straight up a mountain",
        end.z.sub(start.z).to_f32_for_render()
    );
    let atlas = sim::atlas::valley();
    let land = atlas.land.as_ref().unwrap();
    assert!(
        land.steepness(end.x, end.z).raw()
            <= sim::tuning::terrain_steepest().add(Fx::ratio(3, 10)).raw(),
        "standing on a slope of {}",
        land.steepness(end.x, end.z).to_f32_for_render()
    );
    // And jumping at it the whole way does not get up it either.
    let jump = Input::looking_at(Input::W | Input::SPACE, heading(2, 1), 0);
    for f in 0..1200 {
        let i = if f % 40 < 20 { jump } else { north };
        w.advance([i, Input::new(0)]);
    }
    let after = on_map(&w, 0);
    assert!(
        after.y.sub(end.y).raw() < Fx::from_int(12).raw(),
        "jumped {} m up a mountainside",
        after.y.sub(end.y).to_f32_for_render()
    );
}

#[test]
fn walking_out_of_the_town_gate_is_the_mouth_and_the_map_holds_still() {
    let mut w = World::versus_in(CLASSES, valley::START);
    w.advance(idle());
    let zone = valley::place(ArenaId::HEARTH).unwrap().seams[0].zone;
    w.players[0].pos = standing_in(&zone, ArenaId::HEARTH);
    let walk = Input::looking_at(Input::W, 0, 0);
    let mut last = on_map(&w, 0);
    let mut moved = false;
    for _ in 0..300 {
        w.advance([walk, Input::new(0)]);
        let now = on_map(&w, 0);
        assert!(
            now.sub(last).flat_len().raw() < Fx::ONE.raw(),
            "a step of {:?} on the map",
            now.sub(last)
        );
        last = now;
        moved |= w.arena == ArenaId::MOUTH;
    }
    assert!(moved, "never reached the Mouth");
    assert!(w.peaceful());
}

#[test]
fn a_dark_waystone_shuts_its_pass_and_a_trophy_opens_it() {
    let mut w = arrived(ArenaId::BANK, Some(2));
    let zone = valley::place(ArenaId::BANK).unwrap().seams[2].zone;
    w.players[0].pos = standing_in(&zone, ArenaId::BANK);
    let walk = Input::looking_at(Input::W, 0, 0);
    for _ in 0..240 {
        w.advance([walk, Input::new(0)]);
        assert_eq!(w.arena, ArenaId::BANK, "the dark waystone let her through");
    }
    let credit = walk.travelling(Travel::credit(SpeciesId::GNAWERS));
    assert_eq!(
        credit.travel.destination(),
        Some(Destination::Credit(SpeciesId::GNAWERS))
    );
    w.advance([credit, Input::new(0)]);
    for _ in 0..240 {
        w.advance([walk, Input::new(0)]);
    }
    assert_eq!(w.arena, ArenaId::SHELVES, "lit, and still shut");
    assert!(
        w.valley.has_beaten(SpeciesId::GNAWERS),
        "the journey forgot"
    );
}

#[test]
fn walking_out_of_a_room_ends_its_hunt() {
    let r = sim::valley::layout::ROOMS
        .iter()
        .find(|r| r.arena == ArenaId::HIGHLANDS)
        .unwrap();
    let mut w = World::arrive(
        CLASSES,
        valley::Journey::default(),
        r.seam.0,
        Some(r.seam.1),
        2,
    );
    for to in sim::valley::layout::approach(r) {
        walk_to(&mut w, to, 1200).unwrap();
        if w.arena == ArenaId::HIGHLANDS {
            break;
        }
    }
    assert!(w.hunting() && w.valley.on && !w.pvp());
    assert_eq!(w.monster().map(|m| m.species), Some(SpeciesId::RIDGEBACK));
    // Back out the way they came: the hunt is over, unwon.
    walk_to(
        &mut w,
        (Fx::ratio(r.from.0, 100), Fx::ratio(r.from.1, 100)),
        1200,
    )
    .unwrap();
    assert_eq!(w.arena, ArenaId::PINEWOOD);
    assert!(!w.hunting() && w.peaceful());
    assert!(!w.valley.has_beaten(SpeciesId::RIDGEBACK));
}

/// **A change of frame changes nothing.** The same world in a room's
/// coordinates, every position shifted by the difference, plays the same
/// frames and lands back in the reach the same: whatever the shift forgot to
/// move would show up as a fighter, a stone or a shadow somewhere else.
#[test]
fn a_change_of_frame_changes_nothing() {
    let atlas = sim::atlas::valley();
    let bank = atlas.placed(ArenaId::BANK).unwrap().at;
    let den = atlas.placed(ArenaId::GNAWERS).unwrap().at;
    for class in sim::class::ALL_CLASSES {
        let mut w = World::arrive(
            [class; 2],
            valley::Journey::default(),
            ArenaId::BANK,
            Some(0),
            2,
        );
        // A while of everything, so every mechanic has been out.
        let script = |f: u32| {
            let bits = match f % 90 {
                0..=9 => Input::LEFT,
                10..=19 => Input::RIGHT,
                20..=24 => Input::MECHANIC,
                25..=29 => Input::SPECIAL,
                30..=59 => Input::W | Input::SPACE,
                _ => Input::D,
            };
            Input::looking_at(bits, (f * 97) as u16, -2000)
        };
        for f in 0..240 {
            w.advance([script(f), script(f + 45)]);
        }
        let mut moved = w.clone();
        moved.shift(bank.sub(den));
        moved.arena = ArenaId::GNAWERS;
        for f in 240..300 {
            w.advance([script(f), script(f + 45)]);
            moved.advance([script(f), script(f + 45)]);
        }
        assert_eq!(
            moved.arena,
            ArenaId::BANK,
            "{}: the frame did not come back",
            class.name()
        );
        // Coming back from the den, the world remembers the den's seam as the
        // way it came in: true, and not a position.
        moved.valley.came = w.valley.came;
        assert!(
            moved == w,
            "{}: a world moved to the den's coordinates and back came out different",
            class.name()
        );
    }
}

#[test]
fn the_valley_is_deterministic_across_a_crossing() {
    let run = || {
        let mut w = World::versus_in(CLASSES, valley::START);
        let zone = valley::place(ArenaId::HEARTH).unwrap().seams[0].zone;
        let feet = standing_in(&zone, ArenaId::HEARTH);
        for p in w.players.iter_mut() {
            p.pos = feet;
        }
        for f in 0..400u32 {
            let bits = if f % 50 < 30 {
                Input::W
            } else {
                Input::SPACE | Input::W
            };
            w.advance([Input::looking_at(bits, 0, 0), Input::new(Input::D)]);
        }
        assert_eq!(w.arena, ArenaId::MOUTH);
        w.checksum()
    };
    assert_eq!(run(), run());
}

#[test]
fn a_won_hunt_goes_quiet_and_is_credited() {
    let mut w = arrived(ArenaId::HIGHLANDS, Some(0));
    w.monster_mut().unwrap().health = 0;
    w.advance(idle());
    assert!(matches!(w.phase, Phase::RoundOver { .. }));
    assert!(w.valley.has_beaten(SpeciesId::RIDGEBACK));
    for _ in 0..sim::tuning::round_over_frames() + 2 {
        w.advance(idle());
    }
    assert_eq!(w.arena, ArenaId::HIGHLANDS);
    assert!(!w.hunting() && w.peaceful() && matches!(w.phase, Phase::Fighting));
    assert!(w.players.iter().all(|p| p.health > 0));
}

#[test]
fn a_lost_hunt_wakes_you_outside() {
    let mut w = arrived(ArenaId::HIGHLANDS, Some(0));
    for p in w.players.iter_mut() {
        p.health = 0;
    }
    for _ in 0..sim::tuning::round_over_frames() + 3 {
        w.advance(idle());
    }
    assert_eq!(w.arena, ArenaId::PINEWOOD);
    let marks = valley::place(ArenaId::PINEWOOD).unwrap().seams[3].marks;
    assert_eq!(
        (w.players[0].pos.x, w.players[0].pos.z),
        (marks[0].at.x, marks[0].at.z)
    );
    assert!(!w.valley.has_beaten(SpeciesId::RIDGEBACK));
}

#[test]
fn a_death_in_a_reach_stands_you_on_your_last_cairn() {
    let mut w = arrived(ArenaId::MOUTH, Some(0));
    let atlas = sim::atlas::valley();
    let origin = w.map_origin();
    // The first snow on the map standing in the Mouth: a cairn the land's
    // layout made.
    let k = (0..atlas.solids.len())
        .find(|&i| {
            let s = atlas.solids[i];
            let mid_x = Fx::from_raw(s.min.x.raw() / 2 + s.max.x.raw() / 2);
            let mid_z = Fx::from_raw(s.min.z.raw() / 2 + s.max.z.raw() / 2);
            s.material == sim::arena::Material::Snow
                && atlas.place_at(mid_x, mid_z).map(|p| p.arena) == Some(ArenaId::MOUTH)
        })
        .expect("the Mouth has cairns");
    let s = atlas.solids[k];
    let cairn = sim::arena::Solid {
        min: s.min.sub(origin),
        max: s.max.sub(origin),
        material: s.material,
    };
    w.players[0].pos = V3::new(
        cairn.min.x.add(Fx::ratio(1, 2)),
        cairn.max.y,
        cairn.min.z.add(Fx::ratio(1, 2)),
    );
    w.players[0].health = 10;
    w.advance(idle());
    assert_eq!(w.valley.cairn[0] as usize, k + 1, "the cairn did not count");
    assert!(w.players[0].health > 10, "a cairn rests you");
    w.players[0].pos = V3::new(Fx::from_int(200), Fx::from_int(30), Fx::ZERO);
    w.players[0].health = 0;
    w.advance(idle());
    let p = &w.players[0];
    assert!(p.health > 0);
    assert!(
        cairn.over(p.pos.x, p.pos.z, Fx::ZERO),
        "not stood back on the cairn"
    );
    assert!(
        matches!(w.phase, Phase::Fighting),
        "a death in a reach ended a round"
    );
}

#[test]
fn nobody_can_hurt_anybody_in_a_reach() {
    let mut w = arrived(ArenaId::MOUTH, Some(0));
    w.players[1].pos = w.players[0]
        .pos
        .add(w.players[0].facing.scale(Fx::from_int(1)));
    let before = w.players[1].health;
    for f in 0..240u32 {
        let swing = if f % 30 < 4 { Input::LEFT } else { 0 };
        w.advance([Input::new(swing), Input::new(0)]);
    }
    assert_eq!(w.players[1].health, before, "a blow landed in a reach");
}

#[test]
fn a_vine_is_climbed_with_the_jump_held() {
    let mut w = arrived(ArenaId::MOUTH, Some(0));
    let vine = valley::vines(ArenaId::MOUTH)[0];
    let mid = vine.middle();
    let up = V3::new(mid.x, Fx::from_int(200), mid.z);
    w.players[0].pos = V3::new(mid.x, w.terrain().floor_below(up), mid.z);
    let start = w.players[0].pos.y;
    // Straight up for three seconds: a vine is slow, but it goes.
    for _ in 0..180 {
        w.advance([Input::new(Input::SPACE), Input::new(0)]);
    }
    let climbed = w.players[0].pos.y.sub(start);
    assert!(
        climbed.raw() > Fx::from_int(6).raw(),
        "climbed only {climbed:?}"
    );
    // Let go of everything: it clings.
    let held = w.players[0].pos.y;
    for _ in 0..30 {
        w.advance(idle());
    }
    assert!(
        w.players[0].pos.y.sub(held).abs().raw() < Fx::ratio(1, 10).raw(),
        "it did not cling"
    );
}

#[test]
fn outside_the_valley_there_are_no_vines_or_seams() {
    let mut w = World::with_classes(CLASSES);
    let before = w.checksum();
    w.advance(idle());
    assert!(!w.valley.on);
    assert_ne!(before, w.checksum());
    assert!(valley::vines(ArenaId::PROVING_GROUND).is_empty());
    assert!(valley::place(ArenaId::PROVING_GROUND).is_none());
}

/// **The hops on the valley's climbs**: since the valley became land the
/// road climbs on foot and nothing on it needs a jump; the climbs are the
/// crags beside it, each ledge a rise up and across from the last, and the
/// pillar's top a rise over the last ledge. Every class has to make each of
/// them with a plain running jump, because a crag is for everybody
/// (`docs/design/courses.md` §0), as gap and rise in centimetres.
const REQUIRED: &[(&str, i32, i32)] = &[
    (
        "a crag: from one ledge to the next",
        100,
        sim::valley::Crag::RISE,
    ),
    (
        "a crag: the last ledge to the top",
        0,
        sim::valley::Crag::RISE,
    ),
    ("the Ring: over its wall to the door", 0, 150),
];

#[test]
fn every_class_can_make_the_hops_nobody_can_go_round() {
    for class in sim::class::ALL_CLASSES {
        let path = sim::envelope::off_the_edge(class, sim::envelope::Extra::Nothing);
        let apex = sim::envelope::path_apex(&path);
        for (what, gap, rise) in REQUIRED {
            // High enough with room to spare, and far enough at that height:
            // a short hop up is a jump from closer in with less run, which a
            // person does without thinking, so the running path's own width
            // at the rise is what bounds it.
            let (gap, rise) = (Fx::ratio(*gap, 100), Fx::ratio(*rise, 100));
            let widest = sim::envelope::widest(std::slice::from_ref(&path), rise);
            assert!(
                apex.raw() >= rise.add(Fx::ratio(3, 10)).raw()
                    && widest.is_some_and(|w| w.raw() >= gap.raw()),
                "{}: cannot make {what} ({} m out, {} m up)",
                class.name(),
                gap.to_f32_for_render(),
                rise.to_f32_for_render()
            );
        }
    }
}

/// **A fall into a place lower than the world's costs what it costs
/// anywhere.** The second fighter drops twenty metres onto the Saddle's
/// snow while the first, in the Shrine twenty metres higher, holds the world
/// in the Shrine's coordinates, where the Saddle's floor is under zero. The
/// fall rule used to measure down to zero and call it free.
#[test]
fn a_fall_into_a_lower_place_costs_the_same() {
    let drop = |frame: ArenaId| {
        let mut w = World::arrive(CLASSES, valley::Journey::default(), frame, None, 2);
        let origin = w.map_origin();
        let land = sim::atlas::valley().land.as_ref().unwrap();
        // Over the Saddle's meadow, twenty metres up.
        let (x, z) = (Fx::from_int(905), Fx::ZERO);
        let floor = land.height(x, z);
        let at = V3::new(x, floor.add(Fx::from_int(20)), z).sub(origin);
        let p = &mut w.players[1];
        p.pos = at;
        p.vel = V3::ZERO;
        p.grounded = false;
        p.fall_over = at.y.sub(sim::tuning::fall_free());
        let before = p.health;
        for _ in 0..240 {
            w.advance(idle());
        }
        assert_eq!(w.arena, frame, "the frame moved");
        before - w.players[1].health
    };
    let in_saddle = drop(ArenaId::SADDLE);
    let from_shrine = drop(ArenaId::MANTIS);
    assert!(in_saddle > 0, "a twenty-metre fall was free");
    assert_eq!(
        from_shrine, in_saddle,
        "the frame changed what a fall costs"
    );
}
