//! **The valley** (`docs/design/valley.md`, `sim::valley`): the places join
//! up into one map you walk, a room's hunt starts when you walk into it,
//! waystones shut the climb, a death in a reach is a cairn, and the props do
//! what they say. The map itself -- tiles, doorways, nothing overlapping --
//! is `tests/atlas.rs`.

mod pilot;

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
/// the Saddle, every waystone lit -- but for its one climb: one fighter,
/// steering from point to point of each leg of the road, never more than
/// twenty seconds between two of them. Every floor meets the next, nothing
/// stands across the road, no slope on it is too steep, and the world is in
/// the Saddle at the end. Between two legs is the Waterfall
/// (`tests/waterfall.rs` climbs it): she is stood on its top, and walks back
/// down the first leg a way to show the pool is not a dead end.
#[test]
fn the_whole_road_can_be_walked() {
    let mut w = World::arrive(CLASSES, open_journey(), valley::START, Some(0), 1);
    let mut seen = vec![w.arena];
    let legs = sim::valley::layout::road();
    assert_eq!(legs.len(), 2, "the road is two walks and the Waterfall");
    for (n, leg) in legs.iter().enumerate() {
        if n > 0 {
            // Back from the pool along the way she came, and to it again.
            let before = &legs[n - 1];
            for &to in before
                .iter()
                .rev()
                .take(3)
                .chain(before.iter().rev().take(1))
            {
                if let Err(e) = walk_to(&mut w, to, 1200) {
                    panic!("back from the Waterfall's pool: {e}");
                }
            }
            let at = sim::arena::waterfall::VALLEY_AT;
            let top = sim::arena::waterfall::solid(
                &sim::arena::waterfall::PIECES[*sim::arena::waterfall::ROUTE.last().unwrap()],
                at,
            );
            let mid = V3::new(
                Fx::from_raw(top.min.x.raw() / 2 + top.max.x.raw() / 2),
                top.max.y,
                Fx::from_raw(top.min.z.raw() / 2 + top.max.z.raw() / 2),
            );
            w.players[0].pos = mid.sub(w.map_origin());
            w.players[0].vel = V3::ZERO;
        }
        for &to in leg {
            if let Err(e) = walk_to(&mut w, to, 1200) {
                panic!("{e}");
            }
            if seen.last() != Some(&w.arena) {
                seen.push(w.arena);
            }
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

/// **A step out of the door is not giving up** (2026-10-12): backing out of
/// a room mid-fight leaves its hunt going, the creature hurt and where it
/// was, and walking back in finds it so. Only walking well away -- past
/// `tuning::hunt_held_past` of the room's edge -- ends the hunt, unwon. The
/// margin is less than the nearest road junction's distance from any room
/// (16 m), so a hunt never holds somebody who is back on the road.
#[test]
fn stepping_out_of_a_room_keeps_its_hunt_and_walking_away_is_followed() {
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
    let way = sim::valley::layout::approach(r);
    for &to in &way {
        walk_to(&mut w, to, 1200).unwrap();
        if w.arena == ArenaId::HIGHLANDS {
            break;
        }
    }
    assert!(w.hunting() && w.valley.on && !w.pvp());
    assert_eq!(w.monster().map(|m| m.species), Some(SpeciesId::RIDGEBACK));
    // Hurt it, so a fresh one would show.
    let hurt = w.monster().unwrap().health / 2;
    for m in w.monsters.iter_mut().flatten() {
        m.health = hurt;
    }
    let room = sim::atlas::valley()
        .placed(ArenaId::HIGHLANDS)
        .copied()
        .unwrap();
    // Out of the door, a few metres: still the same hunt.
    let door = way[way.len() - 1];
    let before = way[way.len() - 2];
    let out = {
        let (dx, dz) = (before.0.sub(door.0), before.1.sub(door.1));
        let d = V3::new(dx, Fx::ZERO, dz).normalized();
        (
            door.0.add(d.x.mul(Fx::from_int(12))),
            door.1.add(d.z.mul(Fx::from_int(12))),
        )
    };
    walk_to(&mut w, out, 1200).unwrap();
    let feet = on_map(&w, 0);
    assert!(!room.holds(feet.x, feet.z), "never left the room");
    assert!(w.hunting(), "a step out of the door ended the hunt");
    assert_eq!(w.arena, ArenaId::HIGHLANDS);
    assert!(
        w.monster().unwrap().health <= hurt,
        "the creature was healed by a step out of the door"
    );
    // And back in: the same creature, still hurt.
    walk_to(&mut w, door, 1200).unwrap();
    assert!(w.hunting());
    assert!(w.monster().unwrap().health <= hurt, "a fresh creature");
    // Away, all the way back to the road: since 2026-10-19 (exploration
    // 0023) a creature that hunts follows, out of its room by its door --
    // it is no longer kept in its room -- and gives up only in time.
    let mut came_out = false;
    for &to in way.iter().rev().skip(1) {
        for _ in 0..1200 {
            for p in w.players.iter_mut() {
                p.health = p.health.max(1);
            }
            if walk_to(&mut w, to, 1).is_ok() {
                break;
            }
            if let Some(m) = w.monster() {
                let at = m.pos.add(w.map_origin());
                came_out |= !room.holds(at.x, at.z);
            }
        }
    }
    // And stood on the road a while.
    for _ in 0..900 {
        for p in w.players.iter_mut() {
            p.health = p.health.max(1);
        }
        w.advance(idle());
        if let Some(m) = w.monster() {
            let at = m.pos.add(w.map_origin());
            came_out |= !room.holds(at.x, at.z);
        }
    }
    let feet = on_map(&w, 0);
    assert!(
        !room.holds_within(feet.x, feet.z, sim::tuning::hunt_held_past()),
        "the road is within the hunt's margin: a hunt would hold a body back on the road"
    );
    assert!(w.hunting(), "walking away ended the hunt of a creature that hunts");
    assert!(came_out, "the creature never came out of its room after them");
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
        for f in 0..120 {
            w.advance([script(f), script(f + 45)]);
        }
        let mut moved = w.clone();
        moved.shift(bank.sub(den));
        moved.arena = ArenaId::GNAWERS;
        // Two seconds in rather than four: the Champion's Rush covers enough
        // ground since 2026-10-12 (exploration 0015) that by four seconds this
        // script has him at the Bank's far edge, where a world framed in the
        // den reads the nearer place first -- a crossing out of a frame the
        // test made up, which is not the question.
        for f in 120..180 {
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

/// **A change of frame changes nothing, in a hunt** (exploration 0023): a
/// hunt goes on when the world moves into another place's coordinates, so
/// every creature's every point -- its bodies, its critters, its pack, its
/// lore, and whatever its species keeps of its own -- moves with it, and
/// everything it reads of its own arena it reads through its den. Every
/// creature in the valley, at home, fought for a while; then over and over
/// moved into the coordinates of the nearest other place and played on (the
/// world comes back home on the frame, as it does when somebody walks back
/// in), and checked against the same fight never moved.
#[test]
fn a_change_of_frame_changes_nothing_in_a_hunt() {
    let atlas = sim::atlas::valley();
    let mut dens: Vec<ArenaId> = valley::all()
        .filter(|p| matches!(p.kind, Kind::Room(_)))
        .map(|p| p.arena)
        .collect();
    dens.sort_by_key(|a| a.0);
    dens.dedup();
    let mut failed = Vec::new();
    for den in dens {
        let Some(home) = atlas.placed(den).copied() else {
            continue;
        };
        // The nearest other place with its own origin: points stay in reach
        // of a centimetre's sixteen bits.
        let other = atlas
            .places
            .iter()
            .filter(|p| p.arena != den && p.at != V3::ZERO)
            .min_by_key(|p| sim::math::wide_len(p.at.sub(home.at)).raw())
            .copied()
            .unwrap();
        let mut w = World::arrive(
            [Class::Champion, Class::DualMage],
            open_journey(),
            den,
            Some(0),
            2,
        );
        if !w.hunting() {
            continue;
        }
        let script = |f: u32| {
            let bits = match f % 80 {
                0..=9 => Input::LEFT,
                10..=29 => Input::W | Input::LEFT,
                30..=34 => Input::SPECIAL,
                35..=49 => Input::W | Input::RIGHT,
                _ => Input::A | Input::LEFT,
            };
            Input::looking_at(bits, (f * 131) as u16, -1500)
        };
        for f in 0..240 {
            w.advance([script(f), script(f + 40)]);
        }
        let mut moved = w.clone();
        let mut ok = true;
        for round in 0..24u32 {
            if moved.hunting() {
                moved.shift(home.at.sub(other.at));
                moved.arena = other.arena;
            }
            for f in 0..10 {
                let f = 240 + round * 10 + f;
                w.advance([script(f), script(f + 40)]);
                moved.advance([script(f), script(f + 40)]);
            }
            moved.valley.came = w.valley.came;
            if moved != w {
                failed.push(format!(
                    "{} (round {round}, framed in {}): {}",
                    den.get().name,
                    other.get().name,
                    differs(&w, &moved)
                ));
                ok = false;
                break;
            }
        }
        let _ = ok;
    }
    assert!(failed.is_empty(), "a hunt moved and back came out different:\n{}", failed.join("\n"));
}

/// What of two worlds differs, roughly.
fn differs(a: &World, b: &World) -> String {
    let mut out = Vec::new();
    if a.arena != b.arena {
        out.push(format!("arena {:?} / {:?}", a.arena, b.arena));
    }
    let fields = |x: String, y: String| -> String {
        let (x, y): (Vec<&str>, Vec<&str>) = (x.split(", ").collect(), y.split(", ").collect());
        x.iter()
            .zip(y.iter())
            .filter(|(p, q)| p != q)
            .take(6)
            .map(|(p, q)| format!("[{p} | {q}]"))
            .collect::<Vec<_>>()
            .join(" ")
    };
    if a.monsters != b.monsters {
        out.push(format!("monsters {}", fields(format!("{:?}", a.monsters), format!("{:?}", b.monsters))));
    }
    if a.critters != b.critters {
        out.push(format!("critters {}", fields(format!("{:?}", a.critters), format!("{:?}", b.critters))));
    }
    if a.pack != b.pack {
        out.push(format!("pack {}", fields(format!("{:?}", a.pack), format!("{:?}", b.pack))));
    }
    if a.players != b.players {
        out.push(format!("players {}", fields(format!("{:?}", a.players), format!("{:?}", b.players))));
    }
    if a.lore != b.lore {
        let (x, y) = (a.lore.cells(), b.lore.cells());
        for k in 0..x.len() {
            if x[k] != y[k] {
                out.push(format!("lore cell {k}: {:?} / {:?}", x[k], y[k]));
            }
        }
    }
    if a.valley != b.valley {
        out.push(format!("journey {:?} / {:?}", (a.valley.den, a.valley.away, a.valley.homing, a.valley.roused, a.valley.stock), (b.valley.den, b.valley.away, b.valley.homing, b.valley.roused, b.valley.stock)));
    }
    if a.effects != b.effects || a.bolts != b.bolts || a.debris != b.debris {
        out.push("effects/bolts/debris".into());
    }
    if out.is_empty() {
        out.push("something else".into());
    }
    out.join("; ")
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
    assert!(w.players[..w.present()].iter().all(|p| p.health > 0));
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
        shape: s.shape,
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
fn outside_the_valley_there_are_no_seams() {
    let mut w = World::with_classes(CLASSES);
    let before = w.checksum();
    w.advance(idle());
    assert!(!w.valley.on);
    assert_ne!(before, w.checksum());
    assert!(valley::place(ArenaId::PROVING_GROUND).is_none());
}

/// **The hops on the valley's climbs**: since the valley became land the
/// road climbs on foot and nothing on it needs a jump; the climbs are the
/// crags beside it, each ledge a rise up and across from the last, and the
/// pillar's top a rise over the last ledge. Every class has to make each of
/// them with a plain running jump **caught at the hands** -- landed is the
/// quick way, caught and climbed the forgiving one (2026-10-16: the gaps went
/// out to the catch when the ledges started catching) -- because a crag is
/// for everybody (`docs/design/courses.md` §0), as gap and rise in
/// centimetres.
const REQUIRED: &[(&str, i32, i32)] = &[
    (
        "a crag: from one ledge to the next",
        sim::valley::Crag::ACROSS,
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
            // High enough to catch with room to spare, and far enough at that
            // height: a short hop up is a jump from closer in with less run,
            // which a person does without thinking, so the running path's own
            // reach at the rise is what bounds it.
            let (gap, rise) = (Fx::ratio(*gap, 100), Fx::ratio(*rise, 100));
            let reach = apex.add(sim::tuning::ledge_reach_high());
            let widest = sim::envelope::widest_caught(std::slice::from_ref(&path), rise);
            assert!(
                reach.raw() >= rise.add(Fx::ratio(3, 10)).raw()
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

/// **Back on the road is out of the hunt**: every room's road junction is
/// further from the room than `tuning::hunt_held_past`, so the margin that
/// keeps a hunt going through a step out of its door never holds somebody who
/// has walked back to the road and means to go on.
#[test]
fn no_road_junction_is_inside_a_hunts_margin() {
    let margin = sim::tuning::hunt_held_past();
    for r in sim::valley::layout::ROOMS {
        let room = sim::atlas::valley().placed(r.arena).copied().unwrap();
        let (x, z) = (Fx::ratio(r.from.0, 100), Fx::ratio(r.from.1, 100));
        assert!(
            !room.holds_within(x, z, margin),
            "{}'s road junction is within {} m of it",
            r.arena.get().name,
            margin.to_f32_for_render()
        );
    }
}

/// **Every class climbs a crag**, the Mouth's: from the meadow at its foot up
/// its ledges -- landed, or caught at the hands and climbed -- and on to its
/// top.
#[test]
fn every_class_climbs_a_crag() {
    use sim::arena::{Material, Solid};
    use sim::valley::Crag;
    let c = sim::arena::mouth::CRAGS[0];
    let atlas = sim::atlas::valley();
    let cm = sim::arena::cm;
    let (x, z) = (cm(c.x), cm(c.z));
    let r = Fx::from_int(8);
    let near: Vec<Solid> = atlas
        .near((x.sub(r), z.sub(r)), (x.add(r), z.add(r)))
        .map(|i| atlas.solids[i as usize])
        .collect();
    let face = cm(c.x - Crag::HALF);
    let mut ledges: Vec<Solid> = near
        .iter()
        .filter(|s| s.max.x == face && s.min.x.raw() < face.raw())
        .copied()
        .collect();
    ledges.sort_by_key(|s| s.max.y.raw());
    assert_eq!(ledges.len(), c.ledges as usize, "the crag's ledges");
    let pillar = *near
        .iter()
        .filter(|s| s.min.x == face && s.max.x == cm(c.x + Crag::HALF))
        .max_by_key(|s| s.max.y.raw())
        .expect("the pillar");
    // The meadow west of the face, beside the half the first ledge is not on.
    let gx = face.sub(cm(Crag::OUT + 300));
    let gz = z.add(Fx::from_int(2));
    let ground = atlas.relief_at(gx, gz);
    let start = Solid {
        min: V3::new(gx.sub(Fx::ONE), ground.sub(Fx::ONE), gz.sub(Fx::ONE)),
        max: V3::new(gx.add(Fx::ONE), ground, gz.add(Fx::ONE)),
        material: Material::Grass,
        shape: sim::arena::Shape::Box,
    };
    let mut route = vec![start];
    route.extend(ledges);
    // The top's west edge, where the last ledge comes up beside it.
    let mut top = pillar;
    top.max.x = face.add(Fx::from_int(2));
    route.push(top);
    let mut failed = Vec::new();
    for class in sim::class::ALL_CLASSES {
        let mut w = World::arrive(
            [class, class],
            valley::Journey::default(),
            ArenaId::MOUTH,
            Some(0),
            1,
        );
        for (k, pair) in route.windows(2).enumerate() {
            let (a, b) = (pilot::local(&w, pair[0]), pilot::local(&w, pair[1]));
            if let Err(e) = pilot::jump(&mut w, a, b) {
                failed.push(format!("{}: step {}: {e}", class.name(), k + 1));
                break;
            }
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}
