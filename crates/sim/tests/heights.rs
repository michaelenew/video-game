//! **The High Plane** (`sim::arena::heights`, exploration 0019 §2c): a table
//! mountain 300 m up and the two routes to it. The plane stands where the
//! table says, over the mountains, with the land held up to its edge so what
//! is drawn is what is stood on; no stone of either route is in the land; the
//! Sky Road's gaps are what its table says; and a Bulwark -- the plainest
//! jump there is -- climbs the Hanging Stones from the Bank's road to the
//! plane by the vines between them.

use sim::arena::heights::{self, LADDER, PLANE_AT, PLANE_HALF, SPAN, Stone};
use sim::arena::{ArenaId, Solid};
use sim::valley::{self, Journey};
use sim::{Class, Fx, Input, V3, World};

fn cm(v: i32) -> Fx {
    sim::arena::cm(v)
}

#[test]
fn the_plane_stands_over_the_mountains_and_the_land_is_held_up_to_it() {
    let atlas = sim::atlas::valley();
    let (x, z) = (cm(PLANE_AT[0]), cm(PLANE_AT[2]));
    let floor = atlas.relief_at(x, z);
    assert!(
        floor.sub(cm(PLANE_AT[1])).abs().raw() < Fx::ONE.raw(),
        "the plane's floor is {} m",
        floor.to_f32_for_render()
    );
    // What the renderer draws there -- the land -- is the floor stood on.
    let land = atlas.land.as_ref().expect("land").height(x, z);
    assert!(
        land.sub(floor).abs().raw() < Fx::from_int(2).raw(),
        "the land is drawn at {} m under a floor at {}",
        land.to_f32_for_render(),
        floor.to_f32_for_render()
    );
    // Sixty metres past its south edge, the mountains, well below it.
    let below = atlas.relief_at(x, cm(PLANE_AT[2] - PLANE_HALF.1 - 6000));
    assert!(
        below.raw() < cm(PLANE_AT[1] - 3000).raw(),
        "sixty metres off its edge the land is at {} m",
        below.to_f32_for_render()
    );
}

/// Every stone of both routes, on the map.
fn stones() -> Vec<Stone> {
    let mut all: Vec<Stone> = LADDER.iter().chain(SPAN.iter()).copied().collect();
    all.extend(heights::sky_road());
    all
}

#[test]
fn no_stone_of_a_route_is_in_the_land() {
    let atlas = sim::atlas::valley();
    for s in stones().iter().filter(|s| !s.grounded) {
        for (dx, dz) in [(-1, -1), (1, -1), (-1, 1), (1, 1), (0, 0)] {
            let (x, z) = (cm(s.x + dx * s.half), cm(s.z + dz * s.half));
            let ground = atlas.relief_at(x, z);
            assert!(
                ground.raw() < cm(s.top - 600).raw(),
                "a stone at ({}, {}) with its top at {} m is in the land, at {} m there",
                s.x / 100,
                s.z / 100,
                s.top / 100,
                ground.to_f32_for_render()
            );
        }
    }
}

#[test]
fn the_sky_road_runs_from_the_needle_s_dome_to_the_plane() {
    let road = heights::sky_road();
    let first = road[0];
    let last = *road.last().unwrap();
    let axis = sim::arena::spire::AT;
    let from_axis =
        (((first.x - axis.0) as i64).pow(2) + ((first.z - axis.1) as i64).pow(2)).isqrt();
    assert!(
        (from_axis as i32) < sim::arena::spire::RIM + first.half + 300,
        "the first island is {} m from the Needle's axis",
        from_axis / 100
    );
    assert!(
        first.top <= sim::arena::spire::TOP,
        "the first island is over the dome"
    );
    // The last is a jump short of the plane's south-west corner, over it.
    assert!(last.top > PLANE_AT[1], "the last island is under the plane");
    let to_plane = PLANE_AT[2] - PLANE_HALF.1 - (last.z + last.half);
    assert!(
        (0..2600).contains(&to_plane),
        "the last island is {} m short of the plane",
        to_plane / 100
    );
}

/// The solids of the map near a point.
fn near(x: Fx, z: Fx, r: Fx) -> Vec<Solid> {
    let atlas = sim::atlas::valley();
    atlas
        .near((x.sub(r), z.sub(r)), (x.add(r), z.add(r)))
        .map(|i| atlas.solids[i as usize])
        .collect()
}

/// **From one stone to the next**, as somebody with no mechanic does it:
/// run at it and jump; and if the jump falls short, catch the vine by it and
/// climb. `Ok` once she stands on its top.
fn onto(w: &mut World, from: &Stone, to: &Stone) -> Result<(), String> {
    let target = (cm(to.x), cm(to.z));
    for f in 0..60 * 40 {
        let o = w.map_origin();
        let feet = w.players[0].pos.add(o);
        let p = &w.players[0];
        // On its top, or on the cairn on it.
        if p.grounded
            && feet.y.sub(cm(to.top)).raw() > Fx::ratio(-1, 5).raw()
            && feet.y.sub(cm(to.top)).raw() < Fx::ratio(3, 5).raw()
            && feet.x.sub(target.0).abs().raw() <= cm(to.half).raw()
            && feet.z.sub(target.1).abs().raw() <= cm(to.half).raw()
        {
            return Ok(());
        }
        let (dx, dz) = (target.0.sub(feet.x), target.1.sub(feet.z));
        let aim = sim::math::atan2_turns(dz, dx).raw() as u16;
        let on_vine = valley::vine_on(&w.terrain(), p.pos);
        let below = feet.y.raw() < cm(to.top).add(Fx::ratio(1, 2)).raw();
        // On a vine and under the top: climb. Otherwise run at it, jumping
        // when the stone underfoot runs out.
        let bits = if on_vine && below {
            Input::SPACE
                | if feet.y.raw() > cm(to.top).raw() {
                    Input::W
                } else {
                    0
                }
        } else {
            // At the edge of the stone she is on: jump, and hold it.
            let d = sim::math::wide_normalized(V3::new(dx, Fx::ZERO, dz));
            let ahead = feet.add(d.scale(Fx::ratio(1, 2)));
            let on = |s: &Stone| {
                ahead.x.sub(cm(s.x)).abs().raw() <= cm(s.half).raw()
                    && ahead.z.sub(cm(s.z)).abs().raw() <= cm(s.half).raw()
            };
            let edge = p.grounded && !on(from) && !on(to);
            Input::W
                | if edge || !p.grounded || f > 600 {
                    Input::SPACE
                } else {
                    0
                }
        };
        w.advance([Input::looking_at(bits, aim, 0), Input::new(0)]);
        if w.players[0].health <= 0 {
            return Err("fell".into());
        }
        let _ = near;
    }
    let feet = w.players[0].pos.add(w.map_origin());
    let p = &w.players[0];
    Err(format!(
        "never stood on it: at {feet:?}, grounded {}, vel {:?}, on a vine {}",
        p.grounded,
        p.vel,
        valley::vine_on(&w.terrain(), p.pos)
    ))
}

#[test]
fn a_bulwark_climbs_the_hanging_stones_to_the_plane_by_the_vines() {
    let j = Journey {
        beaten: u32::MAX,
        ..Journey::default()
    };
    let mut w = World::arrive([Class::Bulwark, Class::Champion], j, ArenaId::BANK, None, 1);
    let s0 = LADDER[0];
    let o = w.map_origin();
    w.players[0].pos = V3::new(cm(s0.x), cm(s0.top), cm(s0.z)).sub(o);
    w.players[0].vel = V3::ZERO;
    w.players[0].health = 10_000;
    let route: Vec<Stone> = LADDER.iter().chain(SPAN.iter()).copied().collect();
    for (k, pair) in route.windows(2).enumerate() {
        if let Err(e) = onto(&mut w, &pair[0], &pair[1]) {
            panic!(
                "stone {} of the climb ({} m up): {e}",
                k + 1,
                pair[1].top / 100
            );
        }
    }
    // And off the last onto the plane.
    let plane = Stone {
        x: SPAN.last().unwrap().x,
        z: PLANE_AT[2] - PLANE_HALF.1 + 800,
        top: PLANE_AT[1],
        half: 600,
        grounded: true,
    };
    onto(&mut w, SPAN.last().unwrap(), &plane).expect("onto the plane");
    assert_eq!(w.arena, ArenaId::HIGH_PLANE, "ended in {}", w.arena().name);
}
