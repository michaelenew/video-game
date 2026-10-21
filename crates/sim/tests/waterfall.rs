//! **The Waterfall** (`sim::arena::waterfall`): the cliff the valley's road
//! climbs at the head of the Pinewood, and the jump course made of the same
//! table. Every hop is one every class makes with a plain running jump --
//! read off the table and checked against each class's measured jump, and
//! then jumped, by every class, in the valley and in the course -- the
//! falling water pushes down, and the cliff cannot be walked round.

mod pilot;

use sim::arena::waterfall::{self, PIECES, ROUTE, SHORE};
use sim::arena::{ArenaId, Solid, cm};
use sim::{Class, Fx, Input, V3, World};

/// **Every hop is inside every class's plain running jump, caught at the
/// hands** (2026-10-16: landed until ledges caught), with the same margin
/// the valley asks of a crag: high enough with 30 cm to spare, and far
/// enough at that height.
#[test]
fn every_class_can_make_every_hop_on_paper() {
    for class in sim::class::MOVERS {
        let path = sim::envelope::off_the_edge(class, sim::envelope::Extra::Nothing);
        let reach = sim::envelope::path_apex(&path).add(sim::tuning::ledge_reach_high());
        for (piece, gap, rise) in waterfall::hops() {
            let (g, r) = (Fx::ratio(gap, 100), Fx::ratio(rise, 100));
            let widest = sim::envelope::widest_caught(std::slice::from_ref(&path), r);
            assert!(
                reach.raw() >= r.add(Fx::ratio(3, 10)).raw()
                    && widest.is_some_and(|w| w.raw() >= g.raw()),
                "{}: cannot make the hop to piece {piece} ({} m out, {} m up)",
                class.name(),
                gap as f32 / 100.0,
                rise as f32 / 100.0
            );
        }
    }
}

/// The course's route is the waterfall's own, hop for hop.
#[test]
fn the_course_climbs_the_same_cliff() {
    let c = sim::course::of(ArenaId::CLIMB_WATERFALL).expect("the Waterfall is a course");
    let hops: Vec<_> = waterfall::hops().collect();
    assert_eq!(
        c.route.len(),
        hops.len() + 3,
        "start, shore, the climb, the nest"
    );
    for (k, (piece, gap, rise)) in hops.iter().enumerate() {
        let step = c.route[k + 2];
        assert_eq!(step.solid as usize, 4 + piece);
        assert_eq!((step.gap, step.rise), (*gap, *rise), "step {}", k + 2);
        let top = c.top(k + 2);
        let want = waterfall::solid(&PIECES[*piece], waterfall::COURSE_AT);
        assert_eq!(top, want);
    }
    // The shelves are checkpoints, and so is the top.
    let checks: Vec<_> = c
        .route
        .iter()
        .filter(|s| s.check)
        .map(|s| s.solid as usize)
        .collect();
    for s in waterfall::SHELVES {
        assert!(checks.contains(&(4 + s)), "shelf {s} is not a checkpoint");
    }
    assert!(checks.contains(&(4 + ROUTE[ROUTE.len() - 1])));
}

/// The flat box a top makes, in some frame's centimetres, as a solid.
fn top_in(piece: &waterfall::Piece, at: [i32; 3]) -> Solid {
    waterfall::solid(piece, at)
}

/// Every hop of the climb, jumped by every class, in a world whose frame is
/// `at` in `arena` -- with the shore under the first.
fn climb_everywhere(make: impl Fn(Class) -> World, at: [i32; 3], origin: impl Fn(&World) -> V3) {
    let mut failed = Vec::new();
    for class in sim::class::MOVERS {
        let mut w = make(class);
        let o = origin(&w);
        let local = |s: Solid| Solid {
            min: s.min.sub(o),
            max: s.max.sub(o),
            material: s.material,
            shape: s.shape,
        };
        for k in 0..ROUTE.len() {
            let from = if k == 0 {
                &SHORE
            } else {
                &PIECES[ROUTE[k - 1]]
            };
            let to = &PIECES[ROUTE[k]];
            if let Err(e) = pilot::jump(&mut w, local(top_in(from, at)), local(top_in(to, at))) {
                failed.push(format!(
                    "{}: to piece {} ({}): {e}",
                    class.name(),
                    ROUTE[k],
                    waterfall::ASKS[k]
                ));
            }
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

/// **In the valley**: every class jumps every hop of the cliff at the head
/// of the Pinewood, land and trees and all.
#[test]
fn every_class_climbs_the_waterfall_in_the_valley() {
    climb_everywhere(
        |class| {
            World::arrive(
                [class, class],
                sim::valley::Journey {
                    beaten: u32::MAX,
                    ..Default::default()
                },
                ArenaId::PINEWOOD,
                Some(0),
                1,
            )
        },
        waterfall::VALLEY_AT,
        |w| w.map_origin(),
    );
}

/// **In the course**: the same, on the cliff hanging in the air.
#[test]
fn every_class_climbs_the_waterfall_course() {
    climb_everywhere(
        |class| World::versus_in([class, class], ArenaId::CLIMB_WATERFALL),
        waterfall::COURSE_AT,
        |_| V3::ZERO,
    );
}

/// **The falling water pushes down**: a jump into it from under the lip
/// rises nowhere, where the same jump beside it rises metres.
#[test]
fn the_falling_water_is_not_a_way_up() {
    let rise = |z: i32| {
        let mut w = World::versus_in(
            [Class::Elementalist, Class::Elementalist],
            ArenaId::CLIMB_WATERFALL,
        );
        let at = waterfall::COURSE_AT;
        let sheet = waterfall::SHEET;
        let x = cm(at[0] + (sheet.min[0] + sheet.max[0]) / 2);
        let y = cm(at[1] + 600);
        let p = &mut w.players[0];
        p.pos = V3::new(x, y, cm(at[2] + z));
        p.vel = V3::new(Fx::ZERO, Fx::from_int(16), Fx::ZERO);
        p.grounded = false;
        p.fall_over = y;
        let mut top = y;
        for _ in 0..40 {
            w.advance([Input::new(Input::SPACE), Input::new(0)]);
            top = top.max(w.players[0].pos.y);
        }
        top.sub(y)
    };
    let beside = rise(-1000);
    let inside = rise(0);
    assert!(
        beside.raw() > Fx::from_int(2).raw(),
        "beside the falls rose only {beside:?}"
    );
    assert!(
        inside.raw() < Fx::ratio(1, 2).raw(),
        "rose {} m up through the falling water",
        inside.to_f32_for_render()
    );
    assert!(sim::valley::sheet_on(
        &sim::arena::Terrain::placed(sim::atlas::valley(), ArenaId::PINEWOOD.get()),
        waterfall::zone(&waterfall::SHEET, waterfall::VALLEY_AT)
            .middle()
            .add(V3::new(Fx::ZERO, Fx::from_int(5), Fx::ZERO))
    ));
}

/// **The cliff cannot be walked round**: from the foot of the falls, steering
/// at the road above it for half a minute -- jumping now and then -- gets
/// nobody up it. The climb is the way.
#[test]
fn the_cliff_cannot_be_walked_round() {
    for side in [-1i32, 1] {
        let mut w = World::arrive(
            [Class::DualMage, Class::DualMage],
            sim::valley::Journey {
                beaten: u32::MAX,
                ..Default::default()
            },
            ArenaId::PINEWOOD,
            Some(0),
            1,
        );
        let at = waterfall::VALLEY_AT;
        let o = w.map_origin();
        let foot = V3::new(cm(at[0] - 1900), Fx::from_int(200), cm(at[2] + side * 1500)).sub(o);
        let ground = w.terrain().floor_below(foot);
        w.players[0].pos = V3::new(foot.x, ground, foot.z);
        // Round the end of the cliff on this side, then up to the road.
        let goals = [
            (cm(at[0] - 600), cm(at[2] + side * 4000)),
            (cm(at[0] + 1500), cm(at[2] + side * 4000)),
            (cm(at[0] + 3500), cm(at[2])),
        ];
        for (n, f) in (0..1800u32).enumerate() {
            let g = goals[(n / 600).min(2)];
            let p = w.players[0].pos.add(o);
            let aim = sim::math::atan2_turns(g.1.sub(p.z), g.0.sub(p.x)).raw() as u16;
            let jump = if f % 50 < 20 { Input::SPACE } else { 0 };
            w.advance([Input::looking_at(Input::W | jump, aim, 0), Input::new(0)]);
        }
        let up = w.players[0].pos.add(o).y.sub(cm(at[1]));
        // Nowhere within a jump of the top.
        assert!(
            up.raw() < cm(waterfall::HEIGHT - 700).raw(),
            "walked {} m up round the cliff's {} end",
            up.to_f32_for_render(),
            if side < 0 { "right" } else { "left" }
        );
    }
}
