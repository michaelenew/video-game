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
/// the zone's ceiling.
fn standing_in(zone: &valley::Zone, arena: ArenaId) -> V3 {
    let mid = zone.middle();
    // From the zone's top down: a bridge over the floor is a floor here.
    let ground = Terrain::bare(arena.get()).ground_under(V3::new(mid.x, zone.max.y, mid.z));
    V3::new(mid.x, ground, mid.z)
}

fn inside_a_solid(arena: ArenaId, p: V3) -> bool {
    let up = V3::new(p.x, p.y.add(Fx::ratio(1, 10)), p.z);
    arena.get().solids().iter().any(|s| {
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

/// **Walk both fighters through every doorway of the valley.** Each stands
/// in the seam on the near side and walks along the door's line; within a few
/// seconds the world has to be in the place on the far side -- and in a room,
/// hunting. Every floor meets, nothing is left across a doorway, and the frame
/// follows.
#[test]
fn every_doorway_can_be_walked_through() {
    let atlas = sim::atlas::valley();
    for door in &atlas.doors {
        let (a, seam) = door.a;
        let mut w = World::arrive(CLASSES, open_journey(), a, None, 1);
        let zone = valley::place(a).unwrap().seams[seam as usize].zone;
        let feet = standing_in(&zone, a);
        w.players[0].pos = feet;
        w.players[0].vel = V3::ZERO;
        // Along the door's line, away from the seam: the long axis of the
        // cut, toward its far end.
        let origin = w.map_origin();
        let mid = feet.add(origin);
        let (cx, cz) = (
            door.cut.max.x.sub(door.cut.min.x),
            door.cut.max.z.sub(door.cut.min.z),
        );
        let axis = if cx.raw() >= cz.raw() { 0 } else { 2 };
        let (lo, hi, at) = if axis == 0 {
            (door.cut.min.x, door.cut.max.x, mid.x)
        } else {
            (door.cut.min.z, door.cut.max.z, mid.z)
        };
        let sign = if hi.sub(at).raw() > at.sub(lo).raw() {
            1
        } else {
            -1
        };
        let walk = Input::looking_at(Input::W, heading(axis, sign), 0);
        let mut arrived = false;
        for _ in 0..600 {
            w.advance([walk, Input::new(0)]);
            if w.arena == door.b.0 {
                arrived = true;
                break;
            }
        }
        assert!(
            arrived,
            "{} to {}: still in {} at {:?} on the map",
            a.get().name,
            door.b.0.get().name,
            w.arena().name,
            on_map(&w, 0)
        );
        if let Some(Kind::Room(species)) = valley::place(door.b.0).map(|p| p.kind) {
            assert!(
                w.hunting(),
                "{}: walked in, and no hunt",
                door.b.0.get().name
            );
            assert!(w.hunted().contains(&Some(species)) || w.pack.is_some());
        }
    }
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
        // The frame changes under her; where she is on the map does not jump.
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
fn a_dark_waystone_shuts_its_door_and_a_trophy_opens_it() {
    let mut w = arrived(ArenaId::BANK, Some(0));
    let zone = valley::place(ArenaId::BANK).unwrap().seams[2].zone;
    w.players[0].pos = standing_in(&zone, ArenaId::BANK);
    let walk = Input::looking_at(Input::W, 0, 0);
    for _ in 0..240 {
        w.advance([walk, Input::new(0)]);
        assert_eq!(w.arena, ArenaId::BANK, "the dark waystone let her through");
    }
    // A trophy, on the wire.
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
fn walking_into_a_room_starts_its_hunt_and_walking_out_ends_it() {
    let mut w = arrived(ArenaId::PINEWOOD, Some(0));
    let zone = valley::place(ArenaId::PINEWOOD).unwrap().seams[3].zone;
    let feet = standing_in(&zone, ArenaId::PINEWOOD);
    for p in w.players.iter_mut() {
        p.pos = feet;
        p.vel = V3::ZERO;
    }
    let south = Input::looking_at(Input::W, heading(2, -1), 0);
    for _ in 0..240 {
        w.advance([south, south]);
        if w.arena == ArenaId::HIGHLANDS {
            break;
        }
    }
    assert_eq!(w.arena, ArenaId::HIGHLANDS);
    assert!(w.hunting() && w.valley.on && !w.pvp());
    assert_eq!(w.monster().map(|m| m.species), Some(SpeciesId::RIDGEBACK));
    // Back out the way they came: the hunt is over, unwon.
    let north = Input::looking_at(Input::W, heading(2, 1), 0);
    for _ in 0..400 {
        w.advance([north, north]);
        if w.arena == ArenaId::PINEWOOD {
            break;
        }
    }
    assert_eq!(w.arena, ArenaId::PINEWOOD);
    assert!(!w.hunting() && w.peaceful());
    assert!(!w.valley.has_beaten(SpeciesId::RIDGEBACK));
}

/// **A change of frame changes nothing.** The same world in another place's
/// coordinates, every position shifted by the difference, plays the same
/// frames and lands in the same place: whatever the shift forgot to move
/// would show up as a fighter, a stone or a shadow somewhere else.
#[test]
fn a_change_of_frame_changes_nothing() {
    let atlas = sim::atlas::valley();
    let bank = atlas.placed(ArenaId::BANK).unwrap().at;
    let mouth = atlas.placed(ArenaId::MOUTH).unwrap().at;
    for class in sim::class::ALL_CLASSES {
        let mut w = World::arrive(
            [class; 2],
            valley::Journey::default(),
            ArenaId::BANK,
            Some(0),
            2,
        );
        // A minute of everything, so every mechanic has been out.
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
        moved.shift(bank.sub(mouth));
        moved.arena = ArenaId::MOUTH;
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
        assert!(
            moved == w,
            "{}: a world moved to the Mouth's coordinates and back came out different",
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
    let mouth = atlas.index_of(ArenaId::MOUTH).unwrap() as u16;
    let k = (0..atlas.solids.len())
        .find(|&i| {
            atlas.sources[i].place == mouth
                && atlas.solids[i].material == sim::arena::Material::Snow
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
    w.players[0].pos = V3::new(Fx::from_int(-60), Fx::ZERO, Fx::ZERO);
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
    let foot = standing_in(&vine, ArenaId::MOUTH);
    w.players[0].pos = V3::new(foot.x, w.arena().ground_under(foot), foot.z);
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
fn an_updraft_carries_a_body_in_the_air_up() {
    let mut w = arrived(ArenaId::SADDLE, Some(0));
    let vent = valley::vents(ArenaId::SADDLE)[0];
    let at = V3::new(vent.at.0, vent.top.sub(Fx::from_int(4)), vent.at.1);
    w.players[0].pos = at;
    w.players[0].vel = V3::ZERO;
    w.players[0].grounded = false;
    for _ in 0..20 {
        w.advance(idle());
    }
    assert!(
        w.players[0].pos.y.raw() > at.y.raw(),
        "the updraft did not lift"
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

/// **The hops nobody can go round**: the jumps on the valley's way up that
/// have no vine, no stair and no updraft beside them, as gap and rise in
/// centimetres. Every class has to make each of them with a plain running
/// jump -- the airdodge is a spare, not a requirement -- because the main
/// route is for everybody (`docs/design/courses.md` §0).
const REQUIRED: &[(&str, i32, i32)] = &[
    (
        "the Mouth: the second terrace to the traverse's first ledge",
        260,
        150,
    ),
    ("the Mouth: along the traverse", 200, 200),
    ("the Mouth: the last ledge to the Lip", 0, 90),
    ("the Bank: up the stair", 100, 220),
    ("the Pinewood: the Highlands' stair to its notch", 300, 80),
    ("the Pinewood: up the Highlands' stair", 100, 220),
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
/// anywhere.** The second fighter drops twenty metres onto the Mouth's floor
/// while the first, in the Bank, holds the world in the Bank's coordinates,
/// where the Mouth's floor is under zero. The fall rule used to measure
/// down to zero and call it free.
#[test]
fn a_fall_into_a_lower_place_costs_the_same() {
    let atlas = sim::atlas::valley();
    let mouth = atlas.placed(ArenaId::MOUTH).unwrap().at;
    let drop = |frame: ArenaId| {
        let mut w = World::arrive(CLASSES, valley::Journey::default(), frame, Some(0), 2);
        let origin = w.map_origin();
        // The first fighter where the frame is; the second over the Mouth's
        // floor at x -60, twenty metres up.
        let floor = Terrain::bare(ArenaId::MOUTH.get()).ground_under(V3::new(
            Fx::from_int(-60),
            Fx::from_int(100),
            Fx::ZERO,
        ));
        let at = V3::new(Fx::from_int(-60), floor.add(Fx::from_int(20)), Fx::ZERO)
            .add(mouth)
            .sub(origin);
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
    let in_mouth = drop(ArenaId::MOUTH);
    let from_bank = drop(ArenaId::BANK);
    assert!(in_mouth > 0, "a twenty-metre fall was free");
    assert_eq!(from_bank, in_mouth, "the frame changed what a fall costs");
}
