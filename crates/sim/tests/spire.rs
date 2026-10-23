//! **The Spire** (`sim::arena::spire`, exploration 0019 §2a): Hearth's tower,
//! a shaft 315 m tall with an inverted dome on it 90 m across. It stands
//! where the table says, the chimney up the dome's stem is open, the dome's
//! underside is a thing a shot meets, nothing jutting from the shaft is in
//! the stair's way -- and every class climbs to the ruins 353 m up: the
//! stair, two lifts, the round of floating stones, the crown of the shaft
//! (a shuttle, a stack of turn-arounds, a ferry), and the chimney's ledges
//! (since 2026-10-22; until then, the stair the whole height).

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
/// The shaft's top, inside the chimney, and the first ledge's takeoff on
/// it, then the chimney's ledges and the mouth's rim: the climb from the
/// shaft's top to the ruins, as boxes on the map.
fn to_the_ruins(b: Fx) -> Vec<Solid> {
    let mut route = Vec::new();
    // Come up on to the shaft's top anywhere, and jump for the first ledge
    // (on the south wall) from north of it -- from under it, a jump goes up
    // under it.
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
    route
}

/// Jump a route of boxes on the map, hop by hop; the first miss, named.
fn hops(w: &mut World, b: Fx, route: &[Solid], what: &str) -> Result<(), String> {
    for (k, pair) in route.windows(2).enumerate() {
        let (from, to) = (pilot::local(w, pair[0]), pilot::local(w, pair[1]));
        pilot::jump(w, from, to).map_err(|e| {
            format!(
                "{what}, hop {} ({} m up): {e}",
                k + 1,
                pair[1].max.y.sub(b).to_int()
            )
        })?;
    }
    Ok(())
}

/// The plaza beside the stair's foot, under its second stone.
fn plaza(b: Fx) -> Solid {
    let (px, pz) = (cm(AT.0 - 200), cm(AT.1 - SHAFT.0 - 450));
    Solid {
        min: V3::new(px.sub(Fx::ONE), b.sub(Fx::ONE), pz.sub(Fx::ONE)),
        max: V3::new(px.add(Fx::ONE), b, pz.add(Fx::ONE)),
        material: Material::Stone,
        shape: Shape::Box,
    }
}

/// **The whole climb, for one class**: the stair, the first lift, the
/// round, the second lift, and the crown of the shaft -- the shuttle jumped
/// off at its speed, the stack turned round three times, the ferry caught
/// going by and ridden to the tower -- the last stone, the shaft's top and
/// the chimney, to the ruins.
fn climb(class: Class) -> Result<World, String> {
    let b = base();
    let moving = spire::drifts(b);
    let j = valley::Journey {
        beaten: u32::MAX,
        ..valley::Journey::default()
    };
    let mut w = World::arrive([class, class], j, ArenaId::SPIRE, Some(0), 1);
    let stair: Vec<Solid> = core::iter::once(plaza(b))
        .chain(spire::stair().map(|s| step_box(b, &s, false)))
        .collect();
    hops(&mut w, b, &stair, "the stair")?;
    let lift = |w: &World, k: usize| {
        let to = spire::LIFTS[k].arrival().solid(b, 100);
        pilot::local(w, to)
    };
    let to = lift(&w, 0);
    pilot::ride(&mut w, &moving[0], to).map_err(|e| format!("the first lift: {e}"))?;
    let mut round = vec![spire::LIFTS[0].arrival().solid(b, 100)];
    round.extend(
        spire::round()
            .zip(spire::ROUND.iter())
            .map(|(s, r)| s.solid(b, spire::floating_deep(r.3))),
    );
    round.push(spire::LIFTS[1].departure().solid(b, 100));
    hops(&mut w, b, &round, "the round")?;
    let to = lift(&w, 1);
    pilot::ride(&mut w, &moving[2], to).map_err(|e| format!("the second lift: {e}"))?;
    let mut steps = vec![spire::LIFTS[1].arrival().solid(b, 100)];
    steps.extend(
        spire::crown_steps()
            .zip(spire::CROWN_STEPS.iter())
            .map(|(s, r)| s.solid(b, spire::floating_deep(r.3))),
    );
    hops(&mut w, b, &steps, "the crown's first stones")?;
    let rest = pilot::local(&w, spire::REST.solid(b, 150));
    pilot::launch(&mut w, &moving[4], rest).map_err(|e| format!("the shuttle: {e}"))?;
    let floors: Vec<Solid> = spire::stack_floors().map(|s| s.solid(b, 80)).collect();
    for k in 0..floors.len() - 1 {
        let dir = if k % 2 == 0 { (0, 1) } else { (0, -1) };
        let (from, to) = (pilot::local(&w, floors[k]), pilot::local(&w, floors[k + 1]));
        pilot::turn_around(&mut w, from, to, dir)
            .map_err(|e| format!("the stack, turn {}: {e}", k + 1))?;
    }
    let top = pilot::local(&w, spire::stack_top().solid(b, 80));
    pilot::catch(&mut w, top, &moving[5]).map_err(|e| format!("catching the ferry: {e}"))?;
    pilot::carried_over(&mut w, &moving[5]).map_err(|e| format!("on the ferry: {e}"))?;
    let deck = pilot::drift_now(&w, &moving[5]);
    let last = pilot::local(&w, spire::last_stone().solid(b, 200));
    pilot::jump(&mut w, deck, last).map_err(|e| format!("off the ferry: {e}"))?;
    let mut ruins = vec![spire::last_stone().solid(b, 200)];
    ruins.extend(to_the_ruins(b));
    hops(&mut w, b, &ruins, "the shaft's top and the chimney")?;
    Ok(w)
}

/// **Every class climbs to the ruins** (since 2026-10-22, by the stair, two
/// lifts, the round and the crown of the shaft): from the plaza, every
/// stone landed or caught and climbed, every lift ridden, the shuttle and
/// the ferry and the stack made, on to the shaft's top, up the chimney's
/// ledges and out of its mouth on to the dome.
#[test]
fn every_class_climbs_to_the_ruins() {
    let mut failed = Vec::new();
    for class in sim::class::MOVERS {
        match climb(class) {
            // And on the dome's top she is on the roost.
            Ok(w) => assert_eq!(w.arena, sim::valley::ranges::RANGES[5].arena),
            Err(e) => failed.push(format!("{}: {e}", class.name())),
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

/// **About half the tower is climbed by hand** (the owner, 2026-10-22: "the
/// user shouldn't traverse but maybe half the current height of the tower
/// manually"): the lifts carry at least two-fifths of it, and the climb
/// by hand is not more than three-fifths.
#[test]
fn the_lifts_carry_about_half_the_tower() {
    let lifted: i32 = spire::LIFTS.iter().map(|l| l.head - l.foot).sum();
    assert!(
        lifted * 5 >= TOP * 2 && (TOP - lifted) * 5 <= TOP * 3,
        "the lifts carry {} m of {}",
        lifted / 100,
        TOP / 100
    );
    // And no section climbed by hand is taller than sixty metres.
    let sections = [
        spire::STAIR_TOP,
        spire::ROUND_TO - spire::ROUND_FROM,
        SHAFT.2 - spire::CROWN_FROM,
        TOP - SHAFT.2,
    ];
    for s in sections {
        assert!(s <= 6000, "a section {} m tall", s / 100);
    }
}

/// Edge to edge on the flat between two boxes, and how far the second's
/// top is over the first's.
fn gap_rise(a: &Solid, b: &Solid) -> (Fx, Fx) {
    let apart = |lo: Fx, hi: Fx, p: Fx, q: Fx| p.sub(hi).max(lo.sub(q)).max(Fx::ZERO);
    let gx = apart(a.min.x, a.max.x, b.min.x, b.max.x);
    let gz = apart(a.min.z, a.max.z, b.min.z, b.max.z);
    (
        sim::math::wide_len(V3::new(gx, Fx::ZERO, gz)),
        b.max.y.sub(a.max.y),
    )
}

/// Does any plain jump of a class's -- run, jump, airdodge -- catch a gap?
fn plain_catches(class: Class, gap: Fx, rise: Fx) -> bool {
    use sim::envelope::{self as e, Extra};
    let mut paths = vec![e::off_the_edge(class, Extra::Nothing)];
    paths.extend(e::airdodge_paths(class));
    e::widest_caught(&paths, rise).is_some_and(|w| w.raw() >= gap.raw())
}

/// **The crown's tricks are tricks**: no plain jump makes the shuttle's gap
/// from where it stops, or from the stone before it; nobody turns the
/// stack round without its wall; and the ferry is out of reach wherever it
/// waits, from where it is caught. A stone that moves is judged on paper
/// where it stops (against `sim::envelope`, airdodge and all), because a
/// pilot stood where it stops is carried off by it wherever it really is.
#[test]
fn the_crown_is_made_only_the_way_it_is_laid_out() {
    let b = base();
    let moving = spire::drifts(b);
    let rest = spire::REST.solid(b, 150);
    let shuttle = &moving[4];
    let stopped = shuttle.at(shuttle.wait as u32 + shuttle.go as u32 + 2);
    let top = spire::stack_top().solid(b, 80);
    let ferry = &moving[5];
    let waits = [
        ("out east", ferry.at(0)),
        (
            "at the tower",
            ferry.at(ferry.wait as u32 + ferry.go as u32 + 2),
        ),
    ];
    let mut cheats = Vec::new();
    for class in sim::class::MOVERS {
        let (g, r) = gap_rise(&stopped, &rest);
        if plain_catches(class, g, r) {
            cheats.push(format!(
                "{}: the rest, {} m from the shuttle stopped",
                class.name(),
                g.to_f32_for_render()
            ));
        }
        for (what, deck) in waits {
            let (g, r) = gap_rise(&top, &deck);
            if plain_catches(class, g, r) {
                cheats.push(format!(
                    "{}: the ferry waiting {what}, {} m from the stack's top",
                    class.name(),
                    g.to_f32_for_render()
                ));
            }
        }
        let j = valley::Journey {
            beaten: u32::MAX,
            ..valley::Journey::default()
        };
        let mut w = World::arrive([class, class], j, ArenaId::SPIRE, Some(0), 1);
        let l = |w: &World, s: Solid| pilot::local(w, s);
        let takeoff = l(&w, spire::crown_steps().last().unwrap().solid(b, 80));
        let to = l(&w, rest);
        if pilot::jump(&mut w, takeoff, to).is_ok() {
            cheats.push(format!(
                "{}: a plain jump from the takeoff to the rest",
                class.name()
            ));
        }
        let floors: Vec<Solid> = spire::stack_floors().map(|s| s.solid(b, 80)).collect();
        for k in 0..floors.len() - 1 {
            let dir = if k % 2 == 0 { (0, 1) } else { (0, -1) };
            let (from, to) = (l(&w, floors[k]), l(&w, floors[k + 1]));
            if pilot::turn_without_a_wall(&mut w, from, to, dir).is_ok() {
                cheats.push(format!(
                    "{}: the stack's turn {} without the wall",
                    class.name(),
                    k + 1
                ));
            }
        }
    }
    assert!(cheats.is_empty(), "{}", cheats.join("\n"));
}

/// **Nothing stands in a lift's way**: no box of the map is inside the
/// space either lift's cars sweep, a little in from its sides -- a car rises
/// through open air from its foot to its head.
#[test]
fn nothing_stands_in_a_lift_s_way() {
    let b = base();
    let skin = Fx::ratio(1, 10);
    for (k, car) in spire::drifts(b).iter().take(4).enumerate() {
        let mut room = car.swept();
        room.min = room.min.add(V3::new(skin, skin, skin));
        room.max = room.max.sub(V3::new(skin, skin, skin));
        // And the room a body stands in on the deck, all the way.
        room.max.y = room.max.y.add(Fx::from_int(2));
        for s in near() {
            let meets = s.min.x.raw() < room.max.x.raw()
                && s.max.x.raw() > room.min.x.raw()
                && s.min.y.raw() < room.max.y.raw()
                && s.max.y.raw() > room.min.y.raw()
                && s.min.z.raw() < room.max.z.raw()
                && s.max.z.raw() > room.min.z.raw();
            assert!(
                !meets,
                "car {k}: a box from {} to {} m up is in its way",
                s.min.y.sub(b).to_f32_for_render(),
                s.max.y.sub(b).to_f32_for_render()
            );
        }
    }
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

/// **A turn-around of 6.5 m off a wall 2.5 m out** (the bench's lane,
/// `arena::bench::TURNS`, which the stack and the Hanging Stones' turns are
/// sized from): every mover makes it, none without the wall -- and a metre
/// higher is past the shortest jumpers.
#[test]
fn a_turn_around_of_six_and_a_half_metres_is_every_mover_s() {
    use sim::arena::bench;
    let solids = ArenaId::BENCH.get().solids;
    let lane = |k: usize| {
        let at = solids
            .iter()
            .position(|s| s.min.z == cm(bench::turn_z(k)))
            .unwrap();
        (solids[at], solids[at + 1])
    };
    let mut failed = Vec::new();
    for class in sim::class::MOVERS {
        let mut w = World::arrive(
            [class, class],
            valley::Journey::default(),
            ArenaId::BENCH,
            Some(0),
            1,
        );
        let (floor, roof) = lane(0);
        if pilot::turn_around(&mut w, floor, roof, (1, 0)).is_err() {
            failed.push(format!("{}: misses the turn-around", class.name()));
        }
        if pilot::turn_without_a_wall(&mut w, floor, roof, (1, 0)).is_ok() {
            failed.push(format!("{}: makes it without the wall", class.name()));
        }
        let (floor, roof) = lane(1);
        let higher = pilot::turn_around(&mut w, floor, roof, (1, 0)).is_ok();
        if matches!(class, Class::Champion | Class::BloodMage) && higher {
            failed.push(format!("{}: makes a metre higher", class.name()));
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}
