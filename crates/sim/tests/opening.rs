//! **The opening can be walked, and the Warden waits for you**
//! (`docs/design/opening.md` §5-§6, `sim::arena::warden`,
//! `sim::species::warden`; the play check, `docs/design/delivery.md` D3).
//!
//! Every class that moves (`sim::class::MOVERS`) walks from the cell, out
//! through the cut, across the yard and the causeway to the keep's gate; and
//! from there round the keep and up its four great steps to the rampart, the
//! way in from above. The Warden sleeps while they come, and wakes when
//! somebody comes into the keep.

use sim::arena::warden::{KEEP, KEEP_FLOOR};
use sim::species::SpeciesId;
use sim::{Class, Fx, Input, V3, World};

const HOLD: u32 = 50;

fn at(x: i32, z: i32) -> (Fx, Fx) {
    (Fx::ratio(x, 100), Fx::ratio(z, 100))
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

/// The way from the cell to the gate, in centimetres: out of the cut,
/// across the yard and over the causeway.
const TO_THE_GATE: [(i32, i32); 6] = [
    (1000, 0),
    (3500, 0),
    (6000, 0),
    (6800, 0),
    (8000, 0),
    (8300, 0),
];

/// From the causeway down into the ravine beside it, out, along the keep's
/// south wall and up the climb -- four pillars, then seven ledges up the
/// wall's outside face and round its corner -- onto the rampart,
/// twenty-nine metres up.
const TO_THE_RAMPART: [(i32, i32, i32); 13] = [
    (6000, -2800, 200),
    (8600, -2900, 400),
    (9075, -2900, 650),
    (9550, -2900, 900),
    (10050, -2900, 1150),
    (10300, -2725, 1400),
    (10700, -2725, 1650),
    (11100, -2725, 1900),
    (11500, -2725, 2150),
    (11880, -2725, 2400),
    (12306, -2547, 2700),
    (12589, -2264, 2950),
    (12626, -1626, 3000),
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
    let mut way = vec![(8000, 0, 200), (9500, -500, 200)];
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
        let way = TO_THE_GATE
            .iter()
            .map(|&(x, z)| (at(x, z), Fx::from_int(2)))
            .chain(
                TO_THE_RAMPART
                    .iter()
                    .map(|&(x, z, h)| (at(x, z), Fx::ratio(h, 100))),
            );
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
        for (k, &(x, z, h)) in up_the_spiral().iter().enumerate() {
            if let Err(e) = walk_to(&mut w, at(x, z), Fx::ratio(h, 100)) {
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
    for &(x, z) in &TO_THE_GATE[..3] {
        walk_to(&mut w, at(x, z), Fx::from_int(2)).expect("the yard is walkable");
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
    for &(x, z) in &TO_THE_GATE[3..] {
        walk_to(&mut w, at(x, z), Fx::from_int(2)).expect("the causeway is walkable");
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
            w.players[0].pos = V3::new(Fx::from_int(110), Fx::from_int(29), Fx::from_int(-23));
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
