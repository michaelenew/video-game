//! **A fight's rim** (`sim::arena::rim`): the bank of land behind its walls
//! that keeps everybody in, where hedges and thickets used to. Inside it the
//! floor is exactly what it was, so nothing a fight was tuned on has moved;
//! outside, the bank is too steep to climb below its crest; and nobody walks
//! or jumps out of a fight over it -- alone, or in a room on the valley's
//! map, where the only way out is the door.

use sim::arena::{self, relief};
use sim::{Class, Fx, Input, V3, World};

const CLASSES: [Class; 2] = [Class::Champion, Class::Elementalist];

/// Every registered arena with a rim.
fn rimmed() -> impl Iterator<Item = &'static arena::Arena> {
    arena::all().filter(|a| a.rim.is_some())
}

#[test]
fn inside_a_rim_the_floor_is_what_it_was() {
    for a in rimmed() {
        let rim = a.rim.unwrap();
        let ((lx, lz), (hx, hz)) = rim.rect();
        for i in 0..=20 {
            for j in 0..=20 {
                let x = lx.add(hx.sub(lx).mul(Fx::ratio(i, 20)));
                let z = lz.add(hz.sub(lz).mul(Fx::ratio(j, 20)));
                assert_eq!(
                    a.relief_at(x, z),
                    relief::height_at(a.id, x, z),
                    "{}: the floor moved inside its rim at ({}, {})",
                    a.name,
                    x.to_f32_for_render(),
                    z.to_f32_for_render()
                );
            }
        }
        assert!(
            a.is_flat_at(Fx::ZERO, Fx::ZERO) == relief::is_flat(a.id),
            "{}: its middle is not as flat as it was",
            a.name
        );
    }
}

/// Points round a rim's rectangle, `d` metres out, every two metres.
fn round(rim: &arena::rim::Rim, d: Fx) -> Vec<(Fx, Fx)> {
    let ((lx, lz), (hx, hz)) = rim.rect();
    let mut out = Vec::new();
    let step = Fx::from_int(2);
    let mut x = lx;
    while x.raw() <= hx.raw() {
        out.push((x, lz.sub(d)));
        out.push((x, hz.add(d)));
        x = x.add(step);
    }
    let mut z = lz;
    while z.raw() <= hz.raw() {
        out.push((lx.sub(d), z));
        out.push((hx.add(d), z));
        z = z.add(step);
    }
    out
}

#[test]
fn a_rims_bank_is_too_steep_to_climb_below_its_crest() {
    for a in rimmed() {
        let rim = a.rim.unwrap();
        let ground = arena::Terrain::bare(a);
        // From a hand's breadth out to halfway up its run.
        let run = rim.run();
        for k in 1..=5 {
            let d = run.mul(Fx::ratio(k, 12)).max(Fx::ratio(1, 4));
            for (x, z) in round(&rim, d) {
                assert!(
                    ground.on_land(x, z),
                    "{}: the rim at ({}, {}) is not land",
                    a.name,
                    x.to_f32_for_render(),
                    z.to_f32_for_render()
                );
                assert!(
                    ground.downhill(x, z).is_some(),
                    "{}: the bank {} m out at ({}, {}) can be stood on",
                    a.name,
                    d.to_f32_for_render(),
                    x.to_f32_for_render(),
                    z.to_f32_for_render()
                );
            }
        }
    }
}

/// The yaw that faces along one of the four flat axes: +x, +z, -x, -z.
fn heading(k: u16) -> u16 {
    k.wrapping_mul(1 << 14)
}

#[test]
fn nobody_walks_or_jumps_out_of_a_fight() {
    for a in rimmed() {
        // A place of the valley that is not a room is the valley's map,
        // which `rooms_on_the_map_are_closed_but_for_their_door` covers.
        if sim::valley::place(a.id).is_some_and(|p| !matches!(p.kind, sim::valley::Kind::Room(_))) {
            continue;
        }
        let rim = a.rim.unwrap();
        let ((lx, lz), (hx, hz)) = rim.rect();
        let slack = Fx::ratio(3, 2);
        for k in 0..4u16 {
            let mut w = World::versus_in(CLASSES, a.id);
            w.players[0].pos = V3::new(
                Fx::from_raw(lx.raw() / 2 + hx.raw() / 2),
                a.relief_at(Fx::ZERO, Fx::ZERO),
                Fx::from_raw(lz.raw() / 2 + hz.raw() / 2),
            );
            let walk = Input::looking_at(Input::W, heading(k), 0);
            let jump = Input::looking_at(Input::W | Input::SPACE, heading(k), 0);
            for f in 0..1500 {
                let i = if f > 600 && f % 40 < 20 { jump } else { walk };
                w.advance([i, Input::new(0)]);
            }
            let p = w.players[0].pos;
            assert!(
                p.x.raw() > lx.sub(slack).raw()
                    && p.x.raw() < hx.add(slack).raw()
                    && p.z.raw() > lz.sub(slack).raw()
                    && p.z.raw() < hz.add(slack).raw(),
                "{}: walking {k} quarters round, she got out to ({}, {}, {})",
                a.name,
                p.x.to_f32_for_render(),
                p.y.to_f32_for_render(),
                p.z.to_f32_for_render()
            );
        }
    }
}

#[test]
fn rooms_on_the_map_are_closed_but_for_their_door() {
    let atlas = sim::atlas::valley();
    let land = atlas.land.as_ref().expect("the valley has land");
    let steepest = sim::tuning::terrain_steepest();
    for room in &sim::valley::layout::ROOMS {
        let a = room.arena.get();
        let Some(rim) = a.rim.filter(|r| !r.drop) else {
            continue;
        };
        let placed = atlas.placed(room.arena).expect("on the map");
        let door = sim::valley::layout::approach(room);
        // The last two points of the way in are either side of the door.
        let (dx, dz) = door[door.len() - 2];
        // `land::DOORWAY` round the door, and a metre.
        let clear = Fx::from_int(21);
        let mut checked = 0;
        for (x, z) in round(&rim, Fx::ONE) {
            let (mx, mz) = (x.add(placed.at.x), z.add(placed.at.z));
            let near_door =
                mx.sub(dx).abs().raw() < clear.raw() && mz.sub(dz).abs().raw() < clear.raw();
            if near_door {
                continue;
            }
            // Walkable ground here, level with the floor, would be a way out.
            let h = land.height(mx, mz);
            let floor = placed.at.y;
            let open = land.steepness(mx, mz).raw() <= steepest.raw()
                && h.sub(floor).raw() < Fx::from_int(2).raw();
            assert!(
                !open,
                "{}: the ground a metre outside its rim at ({}, {}) is open, {} m over its floor",
                a.name,
                mx.to_f32_for_render(),
                mz.to_f32_for_render(),
                h.sub(floor).to_f32_for_render()
            );
            checked += 1;
        }
        assert!(checked > 20, "{}: hardly any rim checked", a.name);
    }
}
