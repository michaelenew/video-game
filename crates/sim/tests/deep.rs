//! **The Deep** (`sim::arena::deep`, exploration 0019 §2d): caves under the
//! Shelves. The path from the road reaches the lid; every warren's maze
//! leads from its way in to its way down; the vines carry a fighter from the
//! mouth to the Caverns' floor unhurt; the lid seals the pit everywhere but
//! its mouth; and each range in it is only at its own level.

use sim::arena::deep::{self, AT, CELL, CELLS, HOLES, LEVELS, LID};
use sim::arena::{ArenaId, Solid};
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

/// A point of the pit, on the map: x, z in the pit's coordinates and a
/// height over its floor (centimetres).
fn pit(x: i32, y: i32, z: i32) -> V3 {
    V3::new(cm(AT[0] + x), cm(AT[1] + y), cm(AT[2] + z))
}

/// The nearest solid of the map along a ray, on the map.
fn ray(from: V3, dir: V3, limit: Fx) -> Option<Fx> {
    let atlas = sim::atlas::valley();
    let lo = V3::new(
        from.x.min(from.x.add(dir.x.mul(limit))),
        from.y.min(from.y.add(dir.y.mul(limit))),
        from.z.min(from.z.add(dir.z.mul(limit))),
    );
    let hi = V3::new(
        from.x.max(from.x.add(dir.x.mul(limit))),
        from.y.max(from.y.add(dir.y.mul(limit))),
        from.z.max(from.z.add(dir.z.mul(limit))),
    );
    atlas
        .near(
            (lo.x.sub(Fx::ONE), lo.z.sub(Fx::ONE)),
            (hi.x.add(Fx::ONE), hi.z.add(Fx::ONE)),
        )
        .map(|i| atlas.solids[i as usize])
        .filter_map(|s: Solid| sim::math::ray_hits_solid(from, dir, &s, Fx::ZERO))
        .filter(|d| d.raw() <= limit.raw())
        .min()
}

#[test]
fn the_path_from_the_shelves_reaches_the_lid() {
    let mut w = World::arrive(
        [Class::Champion, Class::Elementalist],
        journey(),
        ArenaId::SHELVES,
        Some(3),
        1,
    );
    // From the road by the second pass, back along it to where the path
    // leaves, and down the path onto the lid.
    let mut to: Vec<(Fx, Fx)> = vec![(cm(60000), cm(-1200))];
    to.extend(deep::PATH.iter().map(|q| (cm(q.x), cm(q.z))));
    to.push((cm(AT[0]), cm(AT[2] + deep::HALF.1 + 3000)));
    for t in to {
        for _ in 0..1800 {
            let feet = w.players[0].pos.add(w.map_origin());
            let (dx, dz) = (t.0.sub(feet.x), t.1.sub(feet.z));
            if sim::math::wide_len(V3::new(dx, Fx::ZERO, dz)).raw() < Fx::ratio(3, 2).raw() {
                break;
            }
            let aim = sim::math::atan2_turns(dz, dx).raw() as u16;
            w.advance([Input::looking_at(Input::W, aim, -1500), Input::new(0)]);
        }
        let feet = w.players[0].pos.add(w.map_origin());
        let off = sim::math::wide_len(V3::new(t.0.sub(feet.x), Fx::ZERO, t.1.sub(feet.z)));
        assert!(
            off.raw() < Fx::from_int(2).raw(),
            "stuck at {feet:?} in {}, walking to ({}, {})",
            w.arena().name,
            t.0.to_f32_for_render(),
            t.1.to_f32_for_render()
        );
    }
    assert_eq!(w.arena, ArenaId::DEEP, "ended in {}", w.arena().name);
    let feet = w.players[0].pos.add(w.map_origin());
    assert!(
        feet.y.sub(cm(AT[1] + LID)).abs().raw() < Fx::ONE.raw(),
        "on the lid at {} m, its top at {}",
        feet.y.to_f32_for_render(),
        (AT[1] + LID) / 100
    );
}

#[test]
fn every_warren_leads_from_its_way_in_to_its_way_down() {
    for (level, &floor) in LEVELS.iter().enumerate() {
        let (from, to) = (HOLES[level], HOLES[level + 1]);
        let (w, d) = (CELLS.0, CELLS.1);
        let mut seen = vec![false; (w * d) as usize];
        let mut queue = vec![from];
        seen[(from.1 * w + from.0) as usize] = true;
        while let Some(c) = queue.pop() {
            let (cx, cz) = deep::cell_middle(c);
            for (nx, nz) in [
                (c.0 + 1, c.1),
                (c.0 - 1, c.1),
                (c.0, c.1 + 1),
                (c.0, c.1 - 1),
            ] {
                if nx < 0 || nz < 0 || nx >= w || nz >= d || seen[(nz * w + nx) as usize] {
                    continue;
                }
                // A wall between two cells stops a line between their
                // middles, at a body's waist.
                let a = pit(cx, floor + 100, cz);
                let dir = V3::new(cm((nx - c.0) * 100), Fx::ZERO, cm((nz - c.1) * 100));
                if ray(a, dir, cm(CELL)).is_none() {
                    seen[(nz * w + nx) as usize] = true;
                    queue.push((nx, nz));
                }
            }
        }
        let reached = seen.iter().filter(|&&s| s).count();
        assert!(
            seen[(to.1 * w + to.0) as usize],
            "warren {level}: its way down is not reached ({reached} cells are)"
        );
        // A maze, not a hall: every cell reached, by one way each.
        assert_eq!(
            reached as i32,
            w * d,
            "warren {level}: some cells are walled off"
        );
    }
}

#[test]
fn the_lid_seals_the_pit_but_for_its_mouth() {
    // Straight up from every warren's cells and the Caverns: something is
    // overhead before the sky, except under a shaft.
    let holes: Vec<(i32, i32)> = HOLES.to_vec();
    for floor in [0, LEVELS[2], LEVELS[1], LEVELS[0]] {
        for cx in 0..CELLS.0 {
            for cz in 0..CELLS.1 {
                if holes.contains(&(cx, cz)) {
                    continue;
                }
                let (x, z) = deep::cell_middle((cx, cz));
                let from = pit(x + 300, floor + 200, z + 300);
                assert!(
                    ray(
                        from,
                        V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO),
                        Fx::from_int(300)
                    )
                    .is_some(),
                    "open sky over cell ({cx}, {cz}) at {} m",
                    floor / 100
                );
            }
        }
    }
}

#[test]
fn the_vines_carry_a_fighter_from_the_mouth_to_the_caverns() {
    let mut w = World::arrive(
        [Class::Champion, Class::Elementalist],
        journey(),
        ArenaId::DEEP,
        Some(0),
        1,
    );
    let (x, z) = deep::cell_middle(HOLES[0]);
    let start = pit(x, LID + 50, z);
    w.players[0].pos = start.sub(w.map_origin());
    w.players[0].vel = V3::ZERO;
    w.players[0].health = 1000;
    let health = w.players[0].health;
    // Down each shaft with crouch held, and across each warren to its way
    // down: the next shaft.
    for k in 0..HOLES.len() {
        let floor = if k + 1 < HOLES.len() { LEVELS[k] } else { 0 };
        for _ in 0..60 * 60 {
            w.advance([Input::new(Input::CROUCH), Input::new(0)]);
            let feet = w.players[0].pos.add(w.map_origin());
            if w.players[0].grounded && feet.y.sub(cm(AT[1] + floor)).abs().raw() < Fx::ONE.raw() {
                break;
            }
        }
        let feet = w.players[0].pos.add(w.map_origin());
        assert!(
            feet.y.sub(cm(AT[1] + floor)).abs().raw() < Fx::ONE.raw(),
            "shaft {k}: at {} m, not on the floor at {}",
            feet.y.to_f32_for_render(),
            (AT[1] + floor) / 100
        );
        if k + 1 < HOLES.len() {
            // Carried to the next shaft (the maze is walked by the test
            // above); stood over its middle.
            let (nx, nz) = deep::cell_middle(HOLES[k + 1]);
            w.players[0].pos = pit(nx, floor + 20, nz).sub(w.map_origin());
            w.players[0].vel = V3::ZERO;
        }
    }
    assert_eq!(w.players[0].health, health, "the way down hurt her");
}

#[test]
fn a_warren_s_range_is_only_at_its_level() {
    let atlas = sim::atlas::valley();
    for r in valley::ranges::RANGES
        .iter()
        .filter(|r| r.name.contains("Warren"))
    {
        let (x, z) = (cm(r.at.0), cm(r.at.1));
        let on = V3::new(x, cm(r.floor).add(Fx::ONE), z);
        assert_eq!(
            atlas.place_for(on).map(|p| p.arena),
            Some(r.arena),
            "{}",
            r.name
        );
        for other in LEVELS.iter().chain([0].iter()) {
            let y = AT[1] + other;
            if y == r.floor {
                continue;
            }
            let at = V3::new(x, cm(y).add(Fx::ONE), z);
            assert_ne!(
                atlas.place_for(at).map(|p| p.arena),
                Some(r.arena),
                "{} is at {} m too",
                r.name,
                y / 100
            );
        }
    }
}
