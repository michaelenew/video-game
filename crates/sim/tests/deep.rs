//! **The Deep** (`sim::arena::deep`, exploration 0019 §2d): caves under the
//! Shelves. The path from the road reaches the lid; every warren's maze
//! leads from its way in to its way down; a slide down each shaft's wall
//! carries a fighter from the mouth to the Caverns' floor unhurt, and every
//! class climbs back up each by its ledges; the lid seals the pit everywhere
//! but its mouth; and each range in it is only at its own level.

mod pilot;

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

/// A box in the pit's coordinates, on the map.
fn pit_box(min: [i32; 3], max: [i32; 3]) -> Solid {
    Solid::cm(
        [AT[0] + min[0], AT[1] + min[1], AT[2] + min[2]],
        [AT[0] + max[0], AT[1] + max[1], AT[2] + max[2]],
        sim::arena::Material::Rock,
    )
}

/// **Down every shaft by its wall**: dropped in beside a shaft's east wall
/// -- the one with no ledges -- and pushing into it, a fighter slides to the
/// floor below, and the whole way down from the mouth to the Caverns costs
/// nothing.
#[test]
fn a_slide_down_every_shaft_s_wall_is_free() {
    let mut w = World::arrive(
        [Class::Champion, Class::Elementalist],
        journey(),
        ArenaId::DEEP,
        Some(0),
        1,
    );
    w.players[0].health = 1000;
    let health = w.players[0].health;
    for (k, &(from, to)) in deep::SHAFTS.iter().enumerate() {
        let (x, z) = deep::cell_middle(HOLES[k]);
        // In the shaft beside its east wall, under the floor it goes
        // through: lower than a hand's reach of the rim, which would catch.
        let at = pit(x + 300, from - 600, z);
        w.players[0].pos = at.sub(w.map_origin());
        w.players[0].vel = V3::ZERO;
        w.players[0].grounded = false;
        w.players[0].fall_over = w.players[0].pos.y;
        // East is +x: looking that way, forward is into the wall.
        for _ in 0..60 * 30 {
            w.advance([Input::looking_at(Input::W, 0, 0), Input::new(0)]);
            if w.players[0].grounded {
                break;
            }
        }
        let feet = w.players[0].pos.add(w.map_origin());
        assert!(
            feet.y.sub(cm(AT[1] + to)).abs().raw() < Fx::ONE.raw(),
            "shaft {k}: at {} m, not on the floor at {}",
            feet.y.to_f32_for_render(),
            (AT[1] + to) / 100
        );
        // Walk out under the walls, on to the next shaft (the maze is
        // walked by the test above).
        let _ = (x, z);
    }
    assert_eq!(w.players[0].health, health, "the way down hurt her");
}

/// **Every class climbs back up every shaft**: from the floor under it,
/// ledge by ledge -- landed, or caught at the hands and climbed -- and off
/// the last on to the floor it comes up through.
#[test]
fn every_class_climbs_every_shaft() {
    let mut failed = Vec::new();
    for class in sim::class::ALL_CLASSES {
        let mut w = World::arrive([class, class], journey(), ArenaId::DEEP, Some(0), 1);
        for (k, &(from, to)) in deep::SHAFTS.iter().enumerate() {
            let (x, z) = deep::cell_middle(HOLES[k]);
            let h = 400;
            let ledges = deep::shaft_ledges(k);
            // The floor under it, but for what is under the first ledge (on
            // the south wall) and a step short of it: a jump from right
            // beside a ledge goes up under it.
            let mut route = vec![pit_box([x - h, to - 100, z - h + 370], [x + h, to, z + h])];
            route.extend(ledges.iter().map(|&(a, b)| pit_box(a, b)));
            // Up the wall the last ledge is on, over its rim: the ledges
            // start on the south wall and turn about.
            let north = ledges.len() % 2 == 0;
            route.push(if north {
                pit_box([x - h, from - 300, z + h], [x + h, from, z + h + 400])
            } else {
                pit_box([x - h, from - 300, z - h - 400], [x + h, from, z - h])
            });
            for (n, pair) in route.windows(2).enumerate() {
                let (a, b) = (pilot::local(&w, pair[0]), pilot::local(&w, pair[1]));
                if let Err(e) = pilot::jump(&mut w, a, b) {
                    failed.push(format!("{}: shaft {k}, step {}: {e}", class.name(), n + 1));
                    break;
                }
            }
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
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
