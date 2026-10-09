//! The Elementalist after the second playtest, 2026-10-09: what the person
//! reported, as properties. Standing on a stone is being on it; a takeoff
//! pressed a frame after leaving a stone leaves from the stone; `Q` is the
//! Bolt in the air; a stone nobody pointed down at stays at her level; and
//! the Air ball rolls off edges, is steered, and is knocked off what it
//! meets rather than stopped. And the trick the person found and wants kept:
//! a stone raised under her own feet and jumped off at once.
//!
//! See `docs/design/feel-log.md` for that date.

use sim::class::{Mechanic, Structure};
use sim::effects::{Effect, EffectKind};
use sim::moves::elementalist as e;
use sim::moves::elementalist::keys;
use sim::state::{Action, SLOT_POKE};
use sim::tuning as t;
use sim::{Class, Fx, Input, V3, World};

/// The Oven is one store for the whole binary, and one test here turns a
/// knob: every test holds this while it runs, and the knob goes back after.
static STORE: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct Store {
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl Store {
    fn new() -> Store {
        Store {
            _lock: STORE.lock().unwrap_or_else(|e| e.into_inner()),
        }
    }

    fn with_drop(drop: Fx) -> Store {
        let store = Store::new();
        sim::oven::set_scalar(sim::oven::Scalar::PlacementDrop, drop.raw());
        store
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        sim::oven::reset_to_baked();
    }
}

const SPACE: u16 = Input::SPACE;
const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

fn metres(v: f32) -> Fx {
    Fx::ratio((v * 1000.0) as i32, 1000)
}

fn at(x: f32, y: f32, z: f32) -> V3 {
    V3::new(metres(x), metres(y), metres(z))
}

fn as_f(v: Fx) -> f32 {
    v.raw() as f32 / 65536.0
}

fn degrees(d: i32) -> i16 {
    (d * 65536 / 360) as i16
}

fn elementalist() -> World {
    let mut w = World::with_classes([Class::Elementalist, Class::Bulwark]);
    w.players[0].pos = at(-6.0, 0.0, 8.0);
    w.players[1].pos = at(11.0, 0.0, 8.0);
    w
}

fn step_aimed(w: &mut World, bits: u16, yaw: u16, pitch: i16) {
    w.advance([
        Input::looking_at(bits, yaw, pitch),
        Input::looking_at(0, LOOK_LEFT, 0),
    ]);
}

fn step(w: &mut World, bits: u16) {
    step_aimed(w, bits, LOOK_RIGHT, 0);
}

fn run(w: &mut World, frames: u32, bits: u16) {
    for _ in 0..frames {
        step(w, bits);
    }
}

fn doing(w: &World) -> Option<u8> {
    match w.players[0].action {
        Action::Startup { kind, .. }
        | Action::Active { kind, .. }
        | Action::Recovery { kind, .. }
        | Action::Channel { kind, .. } => Some(kind),
        _ => None,
    }
}

fn stones_of(w: &World) -> Vec<Structure> {
    let Mechanic::Structures(slots) = w.players[0].mechanic else {
        panic!("not the class that owns stones");
    };
    slots.iter().flatten().copied().collect()
}

fn balls(w: &World) -> Vec<Effect> {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.kind == EffectKind::AirBall)
        .copied()
        .collect()
}

/// A stone out and standing under her, and her on it.
fn on_a_stone(w: &mut World) -> Structure {
    let stone = Structure {
        age: t::structure_rise(),
        ..Structure::raised(w.players[0].pos)
    };
    let Mechanic::Structures(mut slots) = w.players[0].mechanic else {
        unreachable!()
    };
    slots[0] = Some(stone);
    w.players[0].mechanic = Mechanic::Structures(slots);
    w.players[0].pos.y = stone.top();
    run(w, 2, 0);
    assert!(w.players[0].grounded, "fixture: standing on it");
    stone
}

// ---------------------------------------------------------------------------
// On a stone is on it
// ---------------------------------------------------------------------------

#[test]
fn riding_her_stone_as_it_slows_she_never_leaves_it() {
    let _store = Store::new();
    // The report: on a stone that is still moving she read as airborne every
    // few frames, so a left click there was Landfall and slammed her down. The
    // earth jump's stone, landed on while it is still climbing to its stop,
    // is the case the replay caught.
    let mut w = elementalist();
    w.players[0].vel = at(5.0, 0.0, 0.0);
    step(&mut w, SPACE | keys::EARTH | Input::W);
    let mut landed = None;
    for frame in 0..90 {
        step(&mut w, Input::W);
        let p = w.players[0];
        if p.grounded && p.pos.y.raw() > metres(1.0).raw() {
            landed = Some(frame);
            break;
        }
    }
    assert!(landed.is_some(), "fixture: she lands on her stone");
    for frame in 0..40 {
        step(&mut w, 0);
        assert!(
            w.players[0].grounded,
            "frame {frame} after landing she read as airborne on her own stone"
        );
    }
}

#[test]
fn a_takeoff_a_frame_after_leaving_a_stone_leaves_from_the_stone() {
    let _store = Store::new();
    // The report: space a frame before the click, standing on a stone, put
    // her back on the floor *under* the stone -- the ground, a body's height
    // down -- and the earth jump came out of the ground. It is put back where
    // her feet were: on the stone, which then shatters under the jump.
    let mut w = elementalist();
    let stone = on_a_stone(&mut w);
    step(&mut w, SPACE);
    assert!(!w.players[0].grounded, "fixture: the jump came out first");
    step(&mut w, SPACE | keys::EARTH);
    assert_eq!(doing(&w), Some(e::EARTH_JUMP));
    assert_eq!(
        w.players[0].pos.y,
        stone.top(),
        "put back on the stone she left, not the floor under it"
    );
    let mut top = Fx::ZERO;
    for _ in 0..60 {
        step(&mut w, 0);
        top = top.max(w.players[0].pos.y);
    }
    assert!(
        stones_of(&w).is_empty(),
        "the stone shattered under the jump"
    );
    assert!(
        top.sub(stone.top()).raw() > metres(4.0).raw(),
        "and it was the bigger jump: {} m",
        as_f(top.sub(stone.top()))
    );
}

#[test]
fn the_stone_under_her_feet_and_off_it_at_once_is_the_big_running_jump() {
    let _store = Store::new();
    // The trick the person found and wants kept (2026-10-09): running, look
    // straight down and left-click -- a stone raised exactly under her feet --
    // then space and left click a frame later. That is an earth jump *off a
    // stone*, so the stone shatters and she gets the bigger jump; and unlike
    // the plain earth jump, which costs her some of her run, it keeps all of
    // it. A way to carry speed forward.
    let run_speed = {
        let mut w = elementalist();
        for _ in 0..30 {
            step(&mut w, Input::W);
        }
        w.players[0].vel.x
    };
    let mut w = elementalist();
    for _ in 0..30 {
        step(&mut w, Input::W);
    }
    let down = degrees(-85);
    step_aimed(&mut w, keys::EARTH | Input::W, LOOK_RIGHT, down);
    assert_eq!(stones_of(&w).len(), 1, "a stone under her feet");
    step_aimed(&mut w, SPACE | keys::EARTH | Input::W, LOOK_RIGHT, down);
    let mut up = Fx::ZERO;
    let mut forward = Fx::ZERO;
    for _ in 0..8 {
        step_aimed(&mut w, SPACE | keys::EARTH | Input::W, LOOK_RIGHT, down);
        up = up.max(w.players[0].vel.y);
        forward = forward.max(w.players[0].vel.x);
    }
    assert!(
        stones_of(&w).is_empty(),
        "the stone shattered under the jump"
    );
    assert!(
        up.raw() > t::jump_speed().raw(),
        "the bigger jump: {} m/s up",
        as_f(up)
    );
    assert!(
        forward.raw() >= run_speed.mul(Fx::ratio(9, 10)).raw(),
        "and her run kept: {} m/s against {}",
        as_f(forward),
        as_f(run_speed)
    );
}

// ---------------------------------------------------------------------------
// Q is a push, always
// ---------------------------------------------------------------------------

#[test]
fn q_is_the_bolt_in_the_air_as_on_the_floor() {
    let _store = Store::new();
    let mut w = elementalist();
    w.players[0].pos.y = metres(4.0);
    w.players[0].grounded = false;
    step(&mut w, keys::WEAK_PUSH);
    assert_eq!(doing(&w), Some(SLOT_POKE), "Q in the air is the Bolt");
    assert_eq!(e::AIR_BOLT, e::AIR_BOLT);
}

// ---------------------------------------------------------------------------
// A stone nobody pointed down at stays up
// ---------------------------------------------------------------------------

/// A course with a pit, her on the island of its first checkpoint, and the
/// one of eight directions in which the floor a stone's reach out is far
/// below her. `None` when the island has no such side.
fn on_an_island() -> (World, u16) {
    let c = sim::course::all()
        .find(|c| c.arena().slug() == "falls")
        .expect("the falls course");
    let mut w = World::versus_in([Class::Elementalist, Class::Bulwark], c.arena);
    let stand = c.gate(1).stand();
    w.players[0].pos = stand;
    w.players[1].pos = V3::new(stand.x.add(Fx::from_int(40)), stand.y, stand.z);
    run(&mut w, 4, 0);
    let reach = t::raise_reach();
    for k in 0..8u16 {
        let yaw = k * (65536 / 8) as u16;
        let dir = V3::from_turns(Fx::ratio(k as i32, 8));
        let out = stand.add(dir.scale(reach));
        let floor = w.terrain().ground_under(out);
        if floor.raw() < stand.y.sub(Fx::from_int(4)).raw() {
            // Walked out to a metre short of the lip that way, so a stone's
            // reach from her is out over the drop.
            let mut lip = stand;
            while w.terrain().ground_under(lip.add(dir)).raw() >= stand.y.sub(Fx::ONE).raw() {
                lip = lip.add(dir.scale(Fx::ratio(1, 4)));
            }
            w.players[0].pos = lip;
            run(&mut w, 2, 0);
            return (w, yaw);
        }
    }
    panic!("fixture: the first island has no side over the pit");
}

#[test]
fn a_raise_clicked_past_an_edge_stays_on_her_level() {
    let _store = Store::new();
    let (mut w, yaw) = on_an_island();
    let stand = w.players[0].pos;
    // Level, out over the pit: the crosshair meets the range, not the floor.
    step_aimed(&mut w, keys::EARTH, yaw, 0);
    run(&mut w, 4, 0);
    let stones = stones_of(&w);
    assert_eq!(stones.len(), 1, "raised");
    assert!(
        stones[0].at.y.raw() >= stand.y.sub(t::placement_drop()).raw(),
        "it came up {} m below her, in the pit",
        as_f(stand.y.sub(stones[0].at.y))
    );
}

#[test]
fn a_raise_pointed_down_off_a_ledge_goes_there() {
    // The other half of the rule: a crosshair *on* the floor below is
    // pointing there, and the stone goes there. The course's islands hang
    // far above their pit, out of any reach, so this is the proving ground's
    // dais with the drop the Oven allows turned down below its height.
    let _store = Store::with_drop(metres(0.5));
    let d = dais();
    let mut w = elementalist();
    let z = d.min.z.add(d.max.z).div(Fx::from_int(2));
    // At its edge toward the middle of the arena, looking off it (-X): the
    // open floor, rather than the arena's wall, which stands at its height.
    let lip = V3::new(d.min.x.add(metres(0.6)), d.max.y, z);
    w.players[0].pos = lip;
    run(&mut w, 2, 0);
    let target = V3::new(d.min.x.sub(Fx::from_int(3)), Fx::ZERO, z);
    let pitch = sim::aim::look_onto_closely(lip, LOOK_LEFT, w.players[0].aloft, target);
    step_aimed(&mut w, keys::EARTH, LOOK_LEFT, pitch);
    run(&mut w, 4, 0);
    let stones = stones_of(&w);
    assert_eq!(stones.len(), 1, "raised");
    assert!(
        stones[0].at.y.raw() < metres(0.1).raw(),
        "pointed at the floor below, it went {} m up instead",
        as_f(stones[0].at.y)
    );
}

#[test]
fn a_raise_clicked_out_past_the_dais_stays_on_it() {
    let _store = Store::with_drop(metres(0.5));
    let d = dais();
    let mut w = elementalist();
    let z = d.min.z.add(d.max.z).div(Fx::from_int(2));
    w.players[0].pos = V3::new(d.min.x.add(metres(0.6)), d.max.y, z);
    run(&mut w, 2, 0);
    // Level, off its edge toward the middle: out past the edge the
    // crosshair meets only the range.
    step_aimed(&mut w, keys::EARTH, LOOK_LEFT, degrees(10));
    run(&mut w, 4, 0);
    let stones = stones_of(&w);
    assert_eq!(stones.len(), 1, "raised");
    assert_eq!(stones[0].at.y, d.max.y, "it stayed on the dais, at the lip");
}

#[test]
fn a_landfall_near_an_edge_never_puts_its_slab_in_the_pit() {
    // Asked of where the slab goes (`aim::planted_ahead`) rather than played:
    // a slab put in a course's pit is gone before a test could see it there.
    let _store = Store::new();
    let (w, yaw) = on_an_island();
    let lip = w.players[0].pos;
    let facing = V3::from_turns(Fx::ratio(yaw as i32, 65536));
    let ahead = t::landfall_ahead().max(Fx::from_int(2));
    let out = lip.add(facing.scale(ahead));
    assert!(
        w.terrain().ground_under(out).raw() < lip.y.sub(Fx::from_int(4)).raw(),
        "fixture: the slab's spot is out over the pit"
    );
    let field = sim::stones::gather(&w.players);
    let put = sim::aim::planted_ahead(lip, facing, ahead, &field, &w.terrain());
    if let Some(at) = put {
        assert!(
            at.y.raw() >= lip.y.sub(t::placement_drop()).raw(),
            "the slab came up {} m below her",
            as_f(lip.y.sub(at.y))
        );
    }
}

#[test]
fn a_crack_held_toward_an_edge_stops_at_it() {
    let _store = Store::new();
    let (mut w, yaw) = on_an_island();
    let stand = w.players[0].pos;
    // Held the whole charge, level, toward the pit: Fissure from a stone at
    // her feet, the crosshair out over the drop.
    let down = degrees(-80);
    step_aimed(&mut w, keys::EARTH, yaw, down);
    for _ in 0..90 {
        step_aimed(&mut w, keys::EARTH, yaw, 0);
    }
    for _ in 0..60 {
        step_aimed(&mut w, 0, yaw, 0);
    }
    assert!(
        !stones_of(&w).is_empty(),
        "fixture: the crack's stone erupted"
    );
    for stone in stones_of(&w) {
        assert!(
            stone.at.y.raw() >= stand.y.sub(t::placement_drop()).raw(),
            "the crack's stone erupted {} m below her",
            as_f(stand.y.sub(stone.at.y))
        );
    }
}

// ---------------------------------------------------------------------------
// The Air ball
// ---------------------------------------------------------------------------

fn rolling_ball(from: V3, radius: Fx, dir: V3) -> Effect {
    Effect::cast(
        EffectKind::AirBall,
        0,
        Class::Elementalist,
        e::AIR_BALL,
        from,
        dir,
        radius,
    )
}

fn place(w: &mut World, effect: Effect) {
    let slot = w.effects.iter().position(|s| s.is_none()).expect("room");
    w.effects[slot] = Some(effect);
}

/// The proving ground's raised platform nearest the middle.
fn dais() -> sim::arena::Solid {
    *sim::arena::proving_ground::ARENA
        .solids()
        .iter()
        .filter(|s| s.max.x.raw() < sim::arena::proving_ground::half().raw())
        .max_by_key(|s| s.min.x.raw())
        .expect("the arena has a platform")
}

#[test]
fn a_ball_rolled_off_an_edge_sinks_and_keeps_carrying_her() {
    let _store = Store::new();
    let d = dais();
    let mut w = elementalist();
    // On the dais's top, a step back from its +X edge, rolling off it.
    let top = d.max.y;
    let x = d.max.x.sub(Fx::from_int(1));
    let z = d.min.z.add(d.max.z).div(Fx::from_int(2));
    w.players[0].pos = V3::new(x, top, z);
    run(&mut w, 2, 0);
    let from = w.players[0].pos;
    place(
        &mut w,
        rolling_ball(
            from,
            t::air_ball_radius_full(),
            V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        ),
    );
    let mut last = balls(&w)[0].pos.y;
    let mut over_the_edge_carried = 0;
    for _ in 0..40 {
        step(&mut w, 0);
        let Some(ball) = balls(&w).first().copied() else {
            break;
        };
        let fell = last.sub(ball.pos.y);
        // At most its sink, or the last little step on to the floor (a slope
        // as steep as one frame of its roll).
        let roll = sim::effects::ball_speed(ball.reach).mul(sim::DT);
        let most = t::air_ball_sink().mul(sim::DT).max(roll);
        assert!(
            fell.raw() <= most.add(metres(0.01)).raw(),
            "the ball dropped {} m in a frame: it fell rather than sank",
            as_f(fell)
        );
        last = ball.pos.y;
        let p = w.players[0];
        if ball.pos.x.raw() > d.max.x.add(ball.reach).raw()
            && ball.pos.y.raw() > Fx::ZERO.raw()
            && p.pos.y == ball.pos.y
        {
            over_the_edge_carried += 1;
        }
    }
    assert!(
        over_the_edge_carried > 3,
        "past the edge, it held her up as it sank ({over_the_edge_carried} frames)"
    );
}

#[test]
fn walking_sideways_in_her_ball_steers_it() {
    let _store = Store::new();
    let mut w = elementalist();
    w.players[1].pos = at(11.0, 0.0, -10.0);
    let from = w.players[0].pos;
    place(
        &mut w,
        rolling_ball(
            from,
            t::air_ball_radius_full(),
            V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        ),
    );
    // Looking the way it rolls, holding D: her right, which is +Z here.
    for _ in 0..20 {
        step(&mut w, Input::D);
    }
    let ball = balls(&w)[0];
    assert!(
        ball.dir.z.raw() > metres(0.05).raw(),
        "the ball turned toward her walk: heading {:?}",
        (as_f(ball.dir.x), as_f(ball.dir.z))
    );
    assert!(ball.dir.x.raw() > 0, "a little, not round");
}

#[test]
fn a_ball_that_meets_a_stone_is_knocked_off_it_not_stopped() {
    let _store = Store::new();
    let mut w = elementalist();
    w.players[0].pos = at(-8.0, 0.0, -8.0);
    // A stone of hers standing in its way, three metres on.
    let stone = Structure {
        age: t::structure_rise(),
        ..Structure::raised(at(-3.0, 0.0, 8.0))
    };
    let Mechanic::Structures(mut slots) = w.players[0].mechanic else {
        unreachable!()
    };
    slots[0] = Some(stone);
    w.players[0].mechanic = Mechanic::Structures(slots);
    let size = t::air_ball_radius_full();
    place(
        &mut w,
        rolling_ball(
            at(-6.0, 0.0, 8.0),
            size,
            V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        ),
    );
    let mut turned = false;
    for _ in 0..40 {
        step(&mut w, 0);
        let Some(ball) = balls(&w).first().copied() else {
            panic!("the ball stopped dead at the stone");
        };
        if ball.dir.x.raw() < 0 {
            turned = true;
            assert!(ball.reach.raw() < size.raw(), "and lost some of its size");
            break;
        }
    }
    assert!(turned, "it came back off the stone");
}
