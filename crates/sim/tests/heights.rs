//! **The High Plane** (`sim::arena::heights`, exploration 0019 §2c): a table
//! mountain 300 m up and the two routes to it. The plane stands where the
//! table says, over the mountains, with the land held up to its edge so what
//! is drawn is what is stood on; no stone of either route is in the land; the
//! Sky Road's gaps are what its table says; no plain jump crosses a gully;
//! and every class climbs the Hanging Stones from the Bank's road to the
//! plane -- each stone landed or caught at the hands, each gully crossed off
//! its walls.

mod pilot;

use sim::arena::heights::{self, LADDER, PLANE_AT, PLANE_HALF, SPAN, SPAN_HOPS, Stone};
use sim::arena::{ArenaId, Solid};
use sim::valley::Journey;
use sim::{Fx, V3, World};

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

/// A stone's top as a box, on the map: what a hop is judged against.
fn top_of(s: &Stone) -> Solid {
    Solid::cm(
        [s.x - s.half, s.top - 100, s.z - s.half],
        [s.x + s.half, s.top, s.z + s.half],
        sim::arena::Material::Rock,
    )
}

#[test]
fn the_span_ends_a_hop_short_of_the_plane_and_above_it() {
    let last = SPAN.last().unwrap();
    let edge = PLANE_AT[2] - PLANE_HALF.1;
    let short = edge - (last.z + last.half);
    assert!(
        (100..500).contains(&short),
        "the span ends {} m short of the plane",
        short as f32 / 100.0
    );
    let over = last.top - PLANE_AT[1];
    assert!(
        (200..900).contains(&over),
        "the span's last stone is {} m over the plane",
        over as f32 / 100.0
    );
}

/// **No class's plain jump crosses a gully**, so the walls are the way: the
/// longest plain running jump there is, at the gully's drop, falls short of
/// its length even caught at the hands.
#[test]
fn no_plain_jump_crosses_a_gully() {
    use sim::arena::gully::{DROP, LONG};
    for class in sim::class::ALL_CLASSES {
        let path = sim::envelope::off_the_edge(class, sim::envelope::Extra::Nothing);
        let rise = Fx::ratio(-DROP, 100);
        let caught = sim::envelope::widest_caught(std::slice::from_ref(&path), rise);
        assert!(
            caught.is_none_or(|c| c.raw() < cm(LONG - 50).raw()),
            "{} jumps a gully without a wall: {:?}",
            class.name(),
            caught
        );
    }
}

/// **Every class climbs the Hanging Stones from the Bank's road to the
/// plane**: up the column hop by hop, landed or caught and climbed, along
/// the span, across each gully off its walls, and onto the plane.
#[test]
fn every_class_climbs_the_hanging_stones_to_the_plane() {
    let mut failed = Vec::new();
    for class in sim::class::ALL_CLASSES {
        let j = Journey {
            beaten: u32::MAX,
            ..Journey::default()
        };
        let mut w = World::arrive([class, class], j, ArenaId::BANK, None, 1);
        let s0 = LADDER[0];
        let o = w.map_origin();
        w.players[0].pos = V3::new(cm(s0.x), cm(s0.top), cm(s0.z)).sub(o);
        let route: Vec<Stone> = LADDER.iter().chain(SPAN.iter()).copied().collect();
        let mut kicked = Vec::new();
        for (k, pair) in route.windows(2).enumerate() {
            let (from, to) = (
                pilot::local(&w, top_of(&pair[0])),
                pilot::local(&w, top_of(&pair[1])),
            );
            let span = k + 1 >= LADDER.len();
            let gully = span && matches!(SPAN_HOPS[k + 1 - LADDER.len()], heights::Hop::Gully);
            let done = if gully {
                pilot::kick_across(&mut w, from, to, (0, 1)).map(|n| kicked.push(n))
            } else {
                pilot::jump(&mut w, from, to)
            };
            if let Err(e) = done {
                failed.push(format!(
                    "{}: {} {} ({} m up): {e}",
                    class.name(),
                    if gully {
                        "the gully onto stone"
                    } else {
                        "stone"
                    },
                    k + 1,
                    pair[1].top / 100
                ));
            }
        }
        // And off the last onto the plane.
        let last = *SPAN.last().unwrap();
        let edge = PLANE_AT[2] - PLANE_HALF.1;
        let plane = Solid::cm(
            [last.x - 600, PLANE_AT[1] - 100, edge + 200],
            [last.x + 600, PLANE_AT[1], edge + 1400],
            sim::arena::Material::Grass,
        );
        let (from, to) = (pilot::local(&w, top_of(&last)), pilot::local(&w, plane));
        if let Err(e) = pilot::jump(&mut w, from, to) {
            failed.push(format!("{}: onto the plane: {e}", class.name()));
        } else if w.arena != ArenaId::HIGH_PLANE {
            failed.push(format!("{}: ended in {}", class.name(), w.arena().name));
        }
        println!("{}: kicks per gully {kicked:?}", class.name());
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}
