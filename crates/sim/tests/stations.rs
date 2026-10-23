//! **The stations** (`sim::arena::stations`, 2026-10-22): the creatures of
//! odd ground raised over the land beside the road, each the middle of a jump
//! section. Every route's stones stand clear of the land; every hop is a plain
//! one, inside every mover's plain catch on paper and jumped by the stand-in
//! player (`tests/pilot`), lift and all; a route that ends on a ledge ends on
//! the one it says; and nobody walks up to a station -- it is jumped to.
//!
//! Sized for the movers (`class::MOVERS`), never the Bulwark.

mod pilot;

use std::sync::OnceLock;

use sim::arena::heights::{self, Hop, Need, Stone};
use sim::arena::stations::{self, ROUTES, Route};
use sim::arena::{ArenaId, Solid};
use sim::envelope::{self as e, Extra, Path};
use sim::valley::Journey;
use sim::{Class, Fx, Input, V3, World};

fn cm(v: i32) -> Fx {
    sim::arena::cm(v)
}

/// A stone's top as a box, on the map: what a hop is judged against.
fn top_of(s: &Stone) -> Solid {
    Solid::cm(
        [s.x - s.half, s.top - 100, s.z - s.half],
        [s.x + s.half, s.top, s.z + s.half],
        sim::arena::Material::Rock,
    )
}

/// One hop of a route: the stone it leaves, the one it reaches, and what it
/// is.
struct Leg {
    from: Stone,
    to: Stone,
    hop: Hop,
}

fn legs(r: &Route) -> Vec<Leg> {
    let mut last = r.from;
    r.hops
        .iter()
        .zip(r.stones)
        .map(|(&hop, &to)| {
            let leg = Leg {
                from: last,
                to,
                hop,
            };
            last = to;
            leg
        })
        .collect()
}

fn mover_paths() -> &'static [(Class, Path)] {
    static P: OnceLock<Vec<(Class, Path)>> = OnceLock::new();
    P.get_or_init(|| {
        sim::class::MOVERS
            .iter()
            .map(|&c| (c, e::off_the_edge(c, Extra::Nothing)))
            .collect()
    })
}

/// **Can a path make a hop**: high enough to catch the top with 30 cm to
/// spare, and far enough at that height -- the crags' rule, and the Hanging
/// Stones'.
fn makes(path: &Path, gap: i32, rise: i32) -> bool {
    let apex = e::path_apex(path);
    let (g, r) = (Fx::ratio(gap, 100), Fx::ratio(rise, 100));
    apex.add(sim::tuning::ledge_reach_high()).raw() >= r.add(Fx::ratio(3, 10)).raw()
        && e::widest_caught(std::slice::from_ref(path), r).is_some_and(|w| w.raw() >= g.raw())
}

#[test]
fn every_hop_is_a_plain_one_inside_every_mover_s_jump_on_paper() {
    let mut failed = Vec::new();
    for r in &ROUTES {
        for (k, leg) in legs(r).iter().enumerate() {
            match leg.hop.need() {
                Some(Need::Jump) => {}
                Some(Need::Tool) => failed.push(format!("{}: hop {k} asks a tool", r.name)),
                None => continue,
            }
            let gap = heights::gap(&leg.from, &leg.to);
            let rise = leg.to.top - leg.from.top;
            for (class, path) in mover_paths() {
                if !makes(path, gap, rise) {
                    failed.push(format!(
                        "{}: {} hop {k}, {} m out and {} m up",
                        class.name(),
                        r.name,
                        gap as f32 / 100.0,
                        rise as f32 / 100.0
                    ));
                }
            }
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

/// **The routes join what they say**: the way from the Mire ends on the
/// Pan's ledge, and every route that starts from a ledge starts on its
/// station's.
#[test]
fn the_routes_join_their_stations() {
    let to_pan = &stations::MIRE_TO_PAN[stations::MIRE_TO_PAN.len() - 1];
    let pan = stations::PAN_LEDGE;
    assert_eq!(
        to_pan.top, pan.top,
        "the way from the Mire ends off the Pan's floor"
    );
    assert_eq!(to_pan.z, pan.z);
    assert!(
        (to_pan.x - pan.x).abs() <= pan.half + 100,
        "the way from the Mire misses the Pan's ledge"
    );
    let to_shrine = &stations::CLIFFS_TO_SHRINE[stations::CLIFFS_TO_SHRINE.len() - 1];
    let shrine = stations::SHRINE_LEDGE;
    assert_eq!((to_shrine.x, to_shrine.top), (shrine.x, shrine.top));
    assert!((to_shrine.z - shrine.z).abs() <= shrine.half + 100);
    // On the map in a reach's coordinates, which are the map's: the floor
    // under a point, solids and all (the Cliffs' plateau is a box).
    let map = sim::arena::Terrain::placed(sim::atlas::valley(), ArenaId::SADDLE.get());
    for r in ROUTES.iter().filter(|r| r.from.half >= 400) {
        let ground = map.floor_below(V3::new(cm(r.from.x), cm(r.from.top + 100), cm(r.from.z)));
        assert!(
            ground.sub(cm(r.from.top)).abs().raw() < Fx::ratio(1, 2).raw(),
            "{} starts from a ledge at {} m over ground at {} m",
            r.name,
            r.from.top / 100,
            ground.to_f32_for_render()
        );
    }
}

/// **Every ledge is level ground at its station's floor**, with a drop off
/// its far edge: the land cut through the rim (`land::Pad::ledge`), and not
/// a slope anybody walks up.
#[test]
fn every_ledge_is_its_floor_with_a_drop_past_it() {
    let atlas = sim::atlas::valley();
    for (id, side) in [
        (ArenaId::MIREBACK, (0, 1)),
        (ArenaId::SANDMAW, (0, 1)),
        (ArenaId::BROODMOTHER, (0, 1)),
        (ArenaId::VEILSTALKER, (0, 1)),
        (ArenaId::MANTIS, (-1, 0)),
    ] {
        let l = stations::ledge_of(id);
        let at = atlas.relief_at(cm(l.x), cm(l.z));
        assert!(
            at.sub(cm(l.top)).abs().raw() < Fx::ratio(3, 10).raw(),
            "{}: its ledge is at {} m, its floor {} m",
            id.get().name,
            at.to_f32_for_render(),
            l.top / 100
        );
        // Eight metres out past it, well under it.
        let (x, z) = (l.x + side.0 * 1400, l.z + side.1 * 1400);
        let below = atlas.relief_at(cm(x), cm(z));
        assert!(
            below.raw() < cm(l.top - 500).raw(),
            "{}: past its ledge the land is {} m, its floor {} m",
            id.get().name,
            below.to_f32_for_render(),
            l.top / 100
        );
    }
}

/// **No stone of a route is under the land**: its top is clear of the land
/// under its corners.
#[test]
fn no_stone_of_a_route_is_under_the_land() {
    let atlas = sim::atlas::valley();
    for r in &ROUTES {
        let built = if r.lands {
            &r.stones[..r.stones.len() - 1]
        } else {
            r.stones
        };
        for s in built {
            for (dx, dz) in [(-1, -1), (1, -1), (-1, 1), (1, 1), (0, 0)] {
                let (x, z) = (cm(s.x + dx * s.half), cm(s.z + dz * s.half));
                let land = atlas.land.as_ref().unwrap().height(x, z);
                assert!(
                    land.raw() < cm(s.top - 30).raw(),
                    "{}: a stone at ({}, {}) with its top at {} m is under the land, at {} m",
                    r.name,
                    s.x / 100,
                    s.z / 100,
                    s.top / 100,
                    land.to_f32_for_render()
                );
            }
        }
    }
}

/// **Every mover makes every route**, hop by hop, through the stand-in
/// player: every stone jumped -- landed, or caught and climbed -- and every
/// lift ridden.
#[test]
fn every_mover_makes_every_route() {
    let mut failed = Vec::new();
    let atlas = sim::atlas::valley();
    for class in sim::class::MOVERS {
        for r in &ROUTES {
            let j = Journey {
                beaten: u32::MAX,
                ..Journey::default()
            };
            let start = atlas
                .place_at(cm(r.from.x), cm(r.from.z))
                .map_or(ArenaId::SHELVES, |p| p.arena);
            let mut w = World::arrive([class, class], j, start, None, 1);
            let o = w.map_origin();
            w.players[0].pos = V3::new(cm(r.from.x), cm(r.from.top), cm(r.from.z)).sub(o);
            for (k, leg) in legs(r).iter().enumerate() {
                let (from, to) = (
                    pilot::local(&w, top_of(&leg.from)),
                    pilot::local(&w, top_of(&leg.to)),
                );
                let done = match leg.hop {
                    Hop::Stone { .. } => pilot::jump(&mut w, from, to),
                    Hop::Lift { way, rise } => {
                        pilot::ride(&mut w, &heights::lift_cars(&leg.from, way, rise)[0], to)
                    }
                    _ => Err("a hop the stations do not use".into()),
                };
                if let Err(e) = done {
                    failed.push(format!(
                        "{}: {} hop {k} ({} m out, {} m up): {e}",
                        class.name(),
                        r.name,
                        heights::gap(&leg.from, &leg.to) as f32 / 100.0,
                        (leg.to.top - leg.from.top) as f32 / 100.0,
                    ));
                    break;
                }
            }
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

/// **Nobody walks up to a station**: from the foot of its way up, walking
/// straight at its door for half a minute never gets into it -- its rim's
/// cliff is too steep to climb.
#[test]
fn a_station_is_jumped_to_not_walked_to() {
    let atlas = sim::atlas::valley();
    for room in sim::valley::layout::ROOMS.iter().filter(|r| r.station) {
        let foot = room.from;
        let start = atlas
            .place_at(cm(foot.0), cm(foot.1))
            .map_or(ArenaId::SHELVES, |p| p.arena);
        let mut w = World::arrive([Class::DualMage; 2], Journey::default(), start, None, 1);
        let o = w.map_origin();
        let up = atlas.relief_at(cm(foot.0), cm(foot.1));
        w.players[0].pos = V3::new(cm(foot.0), up, cm(foot.1)).sub(o);
        let door = sim::valley::layout::door_point(room);
        let floor = stations::ledge_of(room.arena).top;
        let place = atlas.placed(room.arena).copied().unwrap();
        for _ in 0..1800 {
            let feet = w.players[0].pos.add(w.map_origin());
            let (dx, dz) = (cm(door[0]).sub(feet.x), cm(door[2]).sub(feet.z));
            let aim = sim::math::atan2_turns(dz, dx).raw() as u16;
            w.advance([Input::looking_at(Input::W, aim, -1500), Input::new(0)]);
            let feet = w.players[0].pos.add(w.map_origin());
            assert!(
                !place.holds(feet.x, feet.z) || feet.y.raw() < cm(floor - 300).raw(),
                "walked up into {} from the foot of its way up",
                room.arena.get().name
            );
        }
    }
}

/// **Nothing the land grows stands in a route**: no tree or boulder of the
/// layout is within a body's width of a stone's column.
#[test]
fn nothing_grows_in_a_route() {
    let atlas = sim::atlas::valley();
    for r in &ROUTES {
        for s in std::iter::once(&r.from).chain(r.stones.iter()) {
            let (lo, hi) = (
                (cm(s.x - s.half - 100), cm(s.z - s.half - 100)),
                (cm(s.x + s.half + 100), cm(s.z + s.half + 100)),
            );
            for i in atlas.near(lo, hi) {
                let b = &atlas.solids[i as usize];
                let src = atlas.sources[i as usize];
                let over = b.min.x.raw() < hi.0.raw()
                    && b.max.x.raw() > lo.0.raw()
                    && b.min.z.raw() < hi.1.raw()
                    && b.max.z.raw() > lo.1.raw();
                let grown = over
                    && src.place == sim::atlas::EXTRA
                    && matches!(b.material, sim::arena::Material::Wood)
                    && b.max.y.sub(b.min.y).raw() > Fx::from_int(5).raw();
                assert!(
                    !grown || b.max.y.raw() < cm(s.top - 600).raw(),
                    "{}: a tree at ({}, {}) in a stone's way",
                    r.name,
                    b.min.x.to_f32_for_render(),
                    b.min.z.to_f32_for_render()
                );
            }
        }
    }
}
