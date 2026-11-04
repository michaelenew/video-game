//! **The opening can be walked, and the Warden waits for you**
//! (`docs/design/opening.md` §5-§6, `sim::arena::warden`,
//! `sim::species::warden`; the play check, `docs/design/delivery.md` D3).
//!
//! The opening is a floating island, its top thirty metres up
//! (`arena::warden::TOP`). Every class that moves (`sim::class::MOVERS`)
//! walks from the cell, out through the cut, round the curtain wall through
//! its gap, through the chapel and over the causeway to the keep's gate;
//! round the keep's lip to the climb, over the pillars rising out of the
//! drop and up the ledges to the rampart; and up the spiral inside. A fall
//! off the island stands you back on the nearest safe spot, a little hurt.
//! The Warden sleeps while they come, wakes when somebody comes into the
//! keep, and goes over when the blow that wakes it comes from above.

use sim::arena::warden::{KEEP, KEEP_FLOOR, SAFE, TOP, WAY};
use sim::species::SpeciesId;
use sim::{Class, Fx, Input, V3, World};

const HOLD: u32 = 50;

fn at(x: i32, z: i32) -> (Fx, Fx) {
    (Fx::ratio(x, 100), Fx::ratio(z, 100))
}

/// A height over the island's top, in centimetres, as a height in the world.
fn up(h: i32) -> Fx {
    Fx::ratio(TOP + h, 100)
}

/// A hunt of the Warden with one hunter: the second seat out of it.
fn hunt(class: Class) -> World {
    World::hunt_of([class, Class::Champion], SpeciesId::WARDEN).seated(1)
}

/// **Walk to a point**, on whatever is under it there: forward held, the
/// look on it, a held jump when the point is higher, the walk is blocked or
/// the ground ahead drops away -- as `tests/versus.rs` walks a stage. `Err`
/// names where it stuck.
fn walk_to(w: &mut World, to: (Fx, Fx), from_height: Fx) -> Result<(), String> {
    let ground = w.terrain();
    let goal = V3::new(
        to.0,
        ground.ground_under(V3::new(to.0, from_height, to.1)),
        to.1,
    );
    let (mut held, mut stuck) = (0u32, 0u32);
    for _ in 0..2400 {
        let p = w.players[0];
        let d = goal.sub(p.pos);
        let level = d.y.abs().raw() < Fx::ratio(30, 100).raw();
        if p.grounded && level && d.flat_len().raw() < Fx::from_int(2).raw() {
            return Ok(());
        }
        let above = d.y.raw() > Fx::ratio(30, 100).raw();
        stuck = if p.grounded && p.vel.flat_len().raw() < Fx::ratio(1, 2).raw() {
            stuck + 1
        } else {
            0
        };
        if p.grounded && held >= HOLD {
            held = 0;
        }
        let jump = if held > 0 && held < HOLD {
            held += 1;
            Input::SPACE
        } else if held == 0 && p.grounded && (above || stuck > 20) {
            held = 1;
            Input::SPACE
        } else {
            if held > 0 {
                held += 1;
            }
            0
        };
        // Hanging from an edge: push into its face to climb up, as a
        // player does (`controls.md`, "Ledges catch").
        if p.ledge.is_some() && p.ledge.grip != sim::state::Grip::Climb {
            let into = p.ledge.edge.normal;
            let aim = (sim::math::atan2_turns(into.z.neg(), into.x.neg()).raw() & 0xFFFF) as u16;
            w.advance([Input::aimed(Input::W, aim), Input::default()]);
            continue;
        }
        let aim = (sim::math::atan2_turns(d.z, d.x).raw() & 0xFFFF) as u16;
        w.advance([Input::aimed(Input::W | jump, aim), Input::default()]);
    }
    let p = w.players[0];
    Err(format!(
        "stuck at ({:.1}, {:.1}, {:.1}) walking to ({:.1}, {:.1}, {:.1}); grounded {}, ledge {:?}, {:?}",
        w.players[0].pos.x.to_f32_for_render(),
        w.players[0].pos.y.to_f32_for_render(),
        w.players[0].pos.z.to_f32_for_render(),
        goal.x.to_f32_for_render(),
        goal.y.to_f32_for_render(),
        goal.z.to_f32_for_render(),
        p.grounded,
        p.ledge.grip,
        p.action
    ))
}

/// From the causeway's end along the keep's island's lip, outside its
/// south-west wall, to the foot of the climb; over its four pillars rising
/// out of the drop; and up the seven ledges on the south wall's outside face
/// and round its corner, onto the rampart, twenty-nine metres up.
/// Centimetres, and the height over the island's top to look down from.
const TO_THE_RAMPART: [(i32, i32, i32); 16] = [
    (8300, -500, 200),
    (8250, -900, 200),
    (8350, -1300, 200),
    (8950, -1900, 200),
    (9450, -2400, 200),
    (9300, -3000, 400),
    (9750, -3300, 650),
    (10250, -3350, 900),
    (10700, -3150, 1150),
    (10300, -2725, 1400),
    (10700, -2725, 1650),
    (11100, -2725, 1900),
    (11500, -2725, 2150),
    (11880, -2725, 2400),
    (12306, -2547, 2700),
    (12589, -2264, 2950),
];

/// **The spiral inside**, from the keep's floor by the gate, ledge by
/// ledge round every face to the top over the gate: along each ledge to
/// the corner where it meets the next, and up. Centimetres, and the height
/// to look down from.
fn up_the_spiral() -> Vec<(i32, i32, i32)> {
    let (kx, kz) = (KEEP.0 as f64, KEEP.1 as f64);
    let at = |deg: f64, r: f64, h: i32| {
        let a = deg.to_radians();
        (
            (kx + r * a.cos()).round() as i32,
            (kz + r * a.sin()).round() as i32,
            h,
        )
    };
    let mut way = vec![(8600, 0, 200), (9500, -500, 200)];
    for k in 0..8 {
        let face = 225.0 + 45.0 * k as f64;
        let top = 325 * (k + 1);
        way.push(at(face - 15.0, 1800.0, top + 100));
        way.push(at(face + 18.0, 1800.0, top + 100));
    }
    // Onto the rampart over the gate's south side.
    way.push(at(200.0, 2300.0, 3000));
    way
}

#[test]
fn every_mover_walks_from_the_cell_to_the_gate_and_up_to_the_rampart() {
    let mut wrong = Vec::new();
    for class in sim::class::MOVERS {
        let mut w = hunt(class);
        let way = WAY
            .iter()
            .map(|&(x, z)| (at(x, z), up(200)))
            .chain(TO_THE_RAMPART.iter().map(|&(x, z, h)| (at(x, z), up(h))));
        for (k, (to, from)) in way.enumerate() {
            if let Err(e) = walk_to(&mut w, to, from) {
                wrong.push(format!("{class:?}, waypoint {k}: {e}"));
                break;
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "the way is not walkable:\n  {}",
        wrong.join("\n  ")
    );
}

/// **Every height inside is reached with the moves every class shares**:
/// each mover walks and jumps the spiral from the floor to the top. In an
/// empty keep -- the Warden would have something to say about it.
#[test]
fn every_mover_climbs_the_spiral_to_the_top() {
    let mut wrong = Vec::new();
    for class in sim::class::MOVERS {
        let mut w = World::versus_in([class, Class::Champion], sim::arena::ArenaId::WARDEN);
        // Stood at the gate: the way there is the other test's.
        w.players[0].pos = V3::new(Fx::from_int(86), up(0), Fx::ZERO);
        for (k, &(x, z, h)) in up_the_spiral().iter().enumerate() {
            if let Err(e) = walk_to(&mut w, at(x, z), up(h)) {
                wrong.push(format!("{class:?}, step {k}: {e}"));
                break;
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "the spiral is not climbable:\n  {}",
        wrong.join("\n  ")
    );
}

fn warden(w: &World) -> sim::Monster {
    w.monsters
        .iter()
        .flatten()
        .copied()
        .find(|m| m.species == SpeciesId::WARDEN)
        .expect("the Warden is in the keep")
}

/// **It sleeps until somebody comes into the keep.** A hunter crossing the
/// yard finds it where it started, having thrown
/// nothing; one who walks in through the gate wakes it, and it comes.
#[test]
fn the_warden_sleeps_until_you_come_into_the_keep() {
    let mut w = hunt(Class::Champion);
    let start = warden(&w).pos;
    for &(x, z) in &WAY[..5] {
        walk_to(&mut w, at(x, z), up(200)).expect("the yard is walkable");
    }
    let m = warden(&w);
    assert!(
        m.pos.sub(start).flat_len().raw() < Fx::ONE.raw(),
        "it walked out to meet you: from {start:?} to {:?}",
        m.pos
    );
    assert!(
        m.brain.grace > 0,
        "its grace ran out with nobody in the keep"
    );
    // In through the gate, to the edge of its floor.
    for &(x, z) in &WAY[5..] {
        walk_to(&mut w, at(x, z), up(200)).expect("the causeway is walkable");
    }
    let inside = at(KEEP.0 - KEEP_FLOOR + 300, 0);
    let mut woke = false;
    for _ in 0..600 {
        let p = w.players[0];
        let d = V3::new(inside.0, p.pos.y, inside.1).sub(p.pos);
        let aim = (sim::math::atan2_turns(d.z, d.x).raw() & 0xFFFF) as u16;
        w.advance([Input::aimed(Input::W, aim), Input::default()]);
        if warden(&w).own[0] != 0 {
            woke = true;
            break;
        }
    }
    assert!(woke, "it did not wake with a hunter in the keep");
}

/// **The climb pays**: a Warden woken by a blow while a hunter is up high
/// -- on the rampart, or diving off it -- opens the fight toppled; woken by
/// a blow from the floor, it only wakes. The blow is stood in for by what a
/// blow does to it, its grace ending (`Monster::take_blow`).
#[test]
fn a_blow_from_above_topples_it_and_one_from_the_floor_does_not() {
    for (from_high, toppled) in [(true, true), (false, false)] {
        let mut w = hunt(Class::Champion);
        w.advance([Input::default(); 2]);
        if from_high {
            // On the rampart over the south wall, outside its floor.
            w.players[0].pos = V3::new(Fx::from_int(110), up(2925), Fx::from_int(-23));
        }
        for m in w.monsters.iter_mut().flatten() {
            m.brain.grace = 0;
        }
        w.advance([Input::default(); 2]);
        let m = warden(&w);
        assert_eq!(m.own[0], 1, "it did not wake");
        assert_eq!(
            matches!(m.doing, sim::monster::Doing::Toppled { .. }),
            toppled,
            "a blow {} the floor: {:?}",
            if from_high { "from above" } else { "from" },
            m.doing
        );
    }
}

/// **A fall off the island is not a death**: a fighter dropped off its
/// edge is stood on the nearest safe spot on its top, hurt by the fall's
/// cost and no more.
#[test]
fn a_fall_off_the_island_stands_you_back_a_little_hurt() {
    let mut w = hunt(Class::Champion);
    w.advance([Input::default(); 2]);
    let full = w.players[0].health;
    // Off the yard's south edge, falling past the island's underside.
    w.players[0].pos = V3::new(Fx::from_int(20), up(-1500), Fx::from_int(-40));
    w.players[0].grounded = false;
    w.advance([Input::default(); 2]);
    let p = w.players[0];
    let (x, z) = SAFE[1];
    assert!(
        p.pos
            .sub(V3::new(Fx::ratio(x, 100), p.pos.y, Fx::ratio(z, 100)))
            .flat_len()
            .raw()
            < Fx::ONE.raw(),
        "stood at {:?}, not on the yard's safe spot",
        p.pos
    );
    assert!(
        p.pos.y.raw() >= up(-10).raw(),
        "not on the island's top: {:?}",
        p.pos
    );
    assert_eq!(p.health, full - sim::species::warden::FALL_COST);
}

/// Is a fighter on the island's top (not falling, not stood back yet)?
fn on_top(p: &sim::state::Player) -> bool {
    p.pos.y.raw() >= up(-50).raw()
}

/// **The curtain wall is crossed at its gap and nowhere else** with the
/// moves every class shares (`arena::warden::CURTAIN`): every mover, from
/// the yard's west side, runs and jumps straight at the far side -- at the
/// wall's middle, at each of its ends where they run out over the drop --
/// and is never on the island's top east of it. Falls off the ends stand
/// it back on the west side.
#[test]
fn the_curtain_wall_is_crossed_only_at_its_gap() {
    let (west, east, _) = sim::arena::warden::CURTAIN;
    let mut wrong = Vec::new();
    // From, and the point straight across.
    let tries = [
        ((2400, -2000), (3600, -2000)),
        ((2400, -500), (3600, -500)),
        ((2500, 600), (3600, 600)),
        // At the ends: along the island's edge and off it, round the wall.
        ((2600, -2600), (3500, -3800)),
        ((2700, 1700), (3400, 3500)),
        ((2700, 1700), (3600, 1900)),
    ];
    for class in sim::class::MOVERS {
        for &(from, to) in &tries {
            let mut w = hunt(class);
            w.players[0].pos = V3::new(Fx::ratio(from.0, 100), up(0), Fx::ratio(from.1, 100));
            w.advance([Input::default(); 2]);
            let goal = at(to.0, to.1);
            // Run at it, jumping whenever stopped, for twenty seconds.
            let mut was_east = None;
            let mut held = 0u32;
            for _ in 0..1200 {
                let p = w.players[0];
                if on_top(&p) && p.pos.x.raw() > Fx::ratio(east, 100).raw() {
                    was_east = Some(p.pos);
                    break;
                }
                let d = V3::new(goal.0, p.pos.y, goal.1).sub(p.pos);
                let aim = (sim::math::atan2_turns(d.z, d.x).raw() & 0xFFFF) as u16;
                held = if p.grounded && held >= HOLD { 0 } else { held };
                let jump = if held < HOLD {
                    held += 1;
                    Input::SPACE
                } else {
                    0
                };
                w.advance([Input::aimed(Input::W | jump, aim), Input::default()]);
            }
            if let Some(pos) = was_east {
                wrong.push(format!("{class:?} from {from:?}: over at {pos:?}"));
            }
            assert!(w.players[0].pos.x.raw() < Fx::ratio(west, 100).raw() || was_east.is_some());
        }
    }
    assert!(
        wrong.is_empty(),
        "the curtain wall is crossed away from its gap:\n  {}",
        wrong.join("\n  ")
    );
}

/// **The chapel is gone through or round, never over**: every mover runs
/// and jumps at its walls from outside, north and south, and comes inside
/// only through a door, never over a wall.
#[test]
fn the_chapels_walls_are_not_climbed() {
    let mut wrong = Vec::new();
    let within = |p: V3, x0: i32, x1: i32, z0: i32, z1: i32| {
        let (x, z) = (p.x.raw(), p.z.raw());
        x > Fx::ratio(x0, 100).raw()
            && x < Fx::ratio(x1, 100).raw()
            && z > Fx::ratio(z0, 100).raw()
            && z < Fx::ratio(z1, 100).raw()
    };
    // Between its walls' inner faces; and its two doorways, west and east.
    let inside = |p: V3| within(p, 4300, 5300, -1500, -300);
    let door = |p: V3| within(p, 4100, 4400, -1100, -700) || within(p, 5200, 5500, -1300, -900);
    let tries = [
        ((4800, -2300), (4800, -900)),
        ((4800, 500), (4800, -900)),
        ((5000, 100), (4900, -1000)),
    ];
    for class in sim::class::MOVERS {
        for &(from, to) in &tries {
            let mut w = hunt(class);
            w.players[0].pos = V3::new(Fx::ratio(from.0, 100), up(0), Fx::ratio(from.1, 100));
            w.advance([Input::default(); 2]);
            let goal = at(to.0, to.1);
            let (mut held, mut was) = (0u32, w.players[0].pos);
            for _ in 0..1200 {
                let p = w.players[0];
                if inside(p.pos) && !inside(was) && !door(was) {
                    wrong.push(format!("{class:?} from {from:?}: in at {:?}", p.pos));
                    break;
                }
                was = p.pos;
                let d = V3::new(goal.0, p.pos.y, goal.1).sub(p.pos);
                let aim = (sim::math::atan2_turns(d.z, d.x).raw() & 0xFFFF) as u16;
                held = if p.grounded && held >= HOLD { 0 } else { held };
                let jump = if held < HOLD {
                    held += 1;
                    Input::SPACE
                } else {
                    0
                };
                w.advance([Input::aimed(Input::W | jump, aim), Input::default()]);
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "the chapel's walls are climbed:\n  {}",
        wrong.join("\n  ")
    );
}

/// **A fall is not a way through the wall**: off the island's north edge
/// just west of the curtain wall, where the nearest safe spot is east of
/// it, a fighter is stood back west of it.
#[test]
fn a_fall_beside_the_wall_stands_you_back_on_your_own_side() {
    let mut w = hunt(Class::Champion);
    w.advance([Input::default(); 2]);
    w.players[0].pos = V3::new(Fx::from_int(28), up(-1500), Fx::from_int(28));
    w.players[0].grounded = false;
    w.advance([Input::default(); 2]);
    let (west, _, _) = sim::arena::warden::CURTAIN;
    let p = w.players[0];
    assert!(on_top(&p), "not stood back: {:?}", p.pos);
    assert!(
        p.pos.x.raw() < Fx::ratio(west, 100).raw(),
        "stood back past the wall: {:?}",
        p.pos
    );
}
