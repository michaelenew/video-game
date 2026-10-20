//! **The Spire** (`sim::arena::spire`, exploration 0019 §2a): Hearth's tower,
//! a shaft 315 m tall with an inverted dome on it 90 m across. It stands
//! where the table says, the chimney up the dome's stem is open, the dome's
//! underside is a thing a shot meets, nothing jutting from the shaft is in
//! the stair's way -- and every class climbs the stair up its south face, on
//! up the chimney's ledges, and stands among the ruins 353 m up.

mod pilot;

use sim::arena::spire::{self, AT, CHIMNEY, DOME_FROM, RIM, SHAFT, TOP};
use sim::arena::{ArenaId, Material, Shape, Solid};
use sim::valley;
use sim::{Class, Fx, V3, World};

fn cm(v: i32) -> Fx {
    sim::arena::cm(v)
}

fn world() -> World {
    let j = valley::Journey {
        beaten: u32::MAX,
        ..valley::Journey::default()
    };
    World::arrive(
        [Class::Champion, Class::Bulwark],
        j,
        ArenaId::SPIRE,
        Some(0),
        1,
    )
}

/// The map's solids near the shaft.
fn near() -> Vec<sim::arena::Solid> {
    let atlas = sim::atlas::valley();
    let (x, z) = (cm(AT.0), cm(AT.1));
    let r = cm(RIM + 500);
    atlas
        .near((x.sub(r), z.sub(r)), (x.add(r), z.add(r)))
        .map(|i| atlas.solids[i as usize])
        .collect()
}

/// The nearest solid a ray meets, in the world's frame.
fn ray(w: &World, from: V3, dir: V3, limit: Fx) -> Option<Fx> {
    w.terrain()
        .along(from, dir, limit, Fx::ONE)
        .filter_map(|s| sim::math::ray_hits_solid(from, dir, &s, Fx::ZERO))
        .min()
}

#[test]
fn the_spire_stands_with_its_dome_and_its_chimney() {
    let solids = near();
    let at_axis = |s: &&sim::arena::Solid| {
        let (x, z) = s.middle();
        x.sub(cm(AT.0)).abs().raw() < Fx::ONE.raw() && z.sub(cm(AT.1)).abs().raw() < Fx::ONE.raw()
    };
    let shaft = solids
        .iter()
        .filter(at_axis)
        .find(|s| {
            matches!(s.shape, Shape::Round { hollow, .. } if hollow == Fx::ZERO)
                && s.max.y.to_int() > 200
        })
        .expect("a shaft");
    assert_eq!(shaft.max.y.to_int(), SHAFT.2 / 100);
    let rings: Vec<_> = solids
        .iter()
        .filter(at_axis)
        .filter(|s| {
            matches!(s.shape, Shape::Round { hollow, .. } if hollow.raw() > 0)
                && s.min.y.to_int() >= DOME_FROM / 100
        })
        .collect();
    assert_eq!(rings.len(), 8, "the dome's slabs");
    let top = rings.iter().map(|s| s.max.y).max().unwrap();
    assert_eq!(
        top.to_int(),
        TOP / 100,
        "the dome's top is {} m up",
        top.to_int()
    );
    let widest = rings.iter().map(|s| s.max.x.sub(s.min.x)).max().unwrap();
    assert!(
        widest.to_int() >= 2 * RIM / 100 - 1,
        "the dome is {} m across",
        widest.to_int()
    );

    // Straight down the chimney from over the top: nothing until the shaft's
    // top, 70 m below the dome's.
    let w = world();
    let o = w.map_origin();
    let from = V3::new(cm(AT.0), cm(TOP + 1000), cm(AT.1)).sub(o);
    let scene_hit = ray(
        &w,
        from,
        V3::new(Fx::ZERO, Fx::from_int(-1), Fx::ZERO),
        Fx::from_int(500),
    );
    let d = scene_hit.expect("meets the shaft's top");
    assert!(
        d.sub(cm(TOP + 1000 - SHAFT.2)).abs().raw() < Fx::ONE.raw(),
        "down the chimney met something {} m down",
        d.to_f32_for_render()
    );
    // And up from the plaza, out under the rim: the dome's underside.
    let from = V3::new(cm(AT.0 + 3500), Fx::from_int(2), cm(AT.1)).sub(o);
    let up = ray(
        &w,
        from,
        V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO),
        Fx::from_int(500),
    );
    let d = up.expect("meets the dome");
    assert!(
        d.to_int() > DOME_FROM / 100 && d.to_int() < TOP / 100,
        "met the dome {} m up",
        d.to_int()
    );
}

/// The plaza's height at the shaft: what every height in the table is above.
fn base() -> Fx {
    near()
        .iter()
        .filter(|s| matches!(s.shape, Shape::Round { hollow, .. } if hollow == Fx::ZERO))
        .map(|s| s.max.y)
        .filter(|y| y.to_int() > 200)
        .min()
        .unwrap()
        .sub(cm(SHAFT.2))
}

/// A step's top as a box on the map: the part of it a body stands on, out
/// of the shaft or of the chimney's wall -- what a hop is aimed at.
fn step_box(b: Fx, s: &spire::Step, inside: bool) -> Solid {
    let mut s = *s;
    if inside {
        s.z = if s.z.0 < 0 {
            (-CHIMNEY + 30, s.z.1)
        } else {
            (s.z.0, CHIMNEY - 30)
        };
    } else {
        s.z.1 = -shaft_r(s.top) - 50;
    }
    s.solid(b, 100)
}

fn shaft_r(y: i32) -> i32 {
    SHAFT.0 - (SHAFT.0 - SHAFT.1) * y / SHAFT.2
}
/// **Every class climbs to the ruins**: from the plaza up the stair on the
/// shaft's south face, stone by stone -- landed, or caught at the hands and
/// climbed -- on to the shaft's top, up the chimney's ledges and out of its
/// mouth on to the dome.
#[test]
fn every_class_climbs_to_the_ruins_by_the_stair() {
    let b = base();
    let mut route = Vec::new();
    // The plaza beside the stair's foot, under its second stone.
    let (px, pz) = (cm(AT.0 - 200), cm(AT.1 - SHAFT.0 - 450));
    route.push(Solid {
        min: V3::new(px.sub(Fx::ONE), b.sub(Fx::ONE), pz.sub(Fx::ONE)),
        max: V3::new(px.add(Fx::ONE), b, pz.add(Fx::ONE)),
        material: Material::Stone,
        shape: Shape::Box,
    });
    route.extend(spire::stair().map(|s| step_box(b, &s, false)));
    // The shaft's top, inside the chimney: come up on to it anywhere, and
    // jump for the first ledge (on the south wall) from north of it -- from
    // under it, a jump goes up under it.
    let r = cm(SHAFT.1);
    route.push(Solid {
        min: V3::new(cm(AT.0).sub(r), b.add(cm(SHAFT.2 - 100)), cm(AT.1).sub(r)),
        max: V3::new(cm(AT.0).add(r), b.add(cm(SHAFT.2)), cm(AT.1).add(r)),
        material: Material::Stone,
        shape: Shape::Box,
    });
    route.push(Solid {
        min: V3::new(cm(AT.0 - 150), b.add(cm(SHAFT.2 - 100)), cm(AT.1 - 90)),
        max: V3::new(cm(AT.0 + 150), b.add(cm(SHAFT.2)), cm(AT.1 + 250)),
        material: Material::Stone,
        shape: Shape::Box,
    });
    let ledges: Vec<spire::Step> = spire::chimney().collect();
    route.extend(ledges.iter().map(|s| step_box(b, s, true)));
    // Out over the mouth's rim, up the wall the last ledge is on.
    let last = ledges.last().unwrap();
    let (mx, mz) = (
        cm(AT.0),
        cm(AT.1
            + if last.z.0 < 0 {
                -CHIMNEY - 110
            } else {
                CHIMNEY + 110
            }),
    );
    let h = Fx::ratio(8, 10);
    route.push(Solid {
        min: V3::new(mx.sub(h), b.add(cm(TOP - 100)), mz.sub(h)),
        max: V3::new(mx.add(h), b.add(cm(TOP)), mz.add(h)),
        material: Material::Stone,
        shape: Shape::Box,
    });
    let mut failed = Vec::new();
    for class in sim::class::MOVERS {
        let j = valley::Journey {
            beaten: u32::MAX,
            ..valley::Journey::default()
        };
        let mut w = World::arrive([class, class], j, ArenaId::SPIRE, Some(0), 1);
        for (k, pair) in route.windows(2).enumerate() {
            let (from, to) = (pilot::local(&w, pair[0]), pilot::local(&w, pair[1]));
            if let Err(e) = pilot::jump(&mut w, from, to) {
                failed.push(format!(
                    "{}: step {} ({} m up): {e}",
                    class.name(),
                    k + 1,
                    pair[1].max.y.sub(b).to_int()
                ));
                break;
            }
        }
        if failed.is_empty() {
            // And on the dome's top she is on the roost.
            assert_eq!(w.arena, sim::valley::ranges::RANGES[5].arena);
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

#[test]
fn nothing_jutting_from_the_shaft_is_across_the_stair() {
    // The corkscrew's stones and the balconies: none in the room round a
    // stair stone a body jumps through -- half a metre either side of it,
    // and from under it to a jump over it.
    let b = base();
    let room: Vec<Solid> = spire::stair()
        .map(|mut s| {
            // The part out of the shaft: the rest is in it.
            s.z.1 = -shaft_r(s.top);
            let mut s = s.solid(b, 200);
            let side = Fx::ratio(1, 2);
            s.min = s.min.sub(V3::new(side, Fx::from_int(2), side));
            s.max = s.max.add(V3::new(side, Fx::from_int(3), side));
            s
        })
        .collect();
    for s in near() {
        if !matches!(s.shape, Shape::Turned { .. }) || s.max.y.raw() > b.add(cm(SHAFT.2)).raw() {
            continue;
        }
        for r in &room {
            let meets = s.min.x.raw() < r.max.x.raw()
                && s.max.x.raw() > r.min.x.raw()
                && s.min.y.raw() < r.max.y.raw()
                && s.max.y.raw() > r.min.y.raw()
                && s.min.z.raw() < r.max.z.raw()
                && s.max.z.raw() > r.min.z.raw();
            assert!(
                !meets,
                "a stone at {} m is in the way of the stair's at {} m",
                s.max.y.sub(b).to_f32_for_render(),
                r.max.y.sub(b).to_int() - 4
            );
        }
    }
}
