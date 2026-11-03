//! **Off the ledge** (the owner, 2026-11-03, from play): standing high and
//! raining shots down, steering an Updraft by its tilt, riding a column, and
//! coming down on a lip.
//!
//! - A crosshair through one of the Elementalist's stones aims at the place
//!   behind it, for an attack as for a placement (`aim::nearest_terrain`).
//! - The Updraft in the air reads the walk keys as a tilt: the best share of
//!   any short run round the press that a direction was held
//!   (`state::updraft_tilt`).
//! - Coming down on the lip of a box lands on it, however little of the body
//!   is over the top (`arena::resolve_among`).
//! - An updraft's column draws a body toward its middle and calls off a slam.

use sim::aim;
use sim::class::{Mechanic, Structure};
use sim::moves::elementalist::keys;
use sim::state::updraft_tilt;
use sim::tuning as t;
use sim::{Class, Fx, Input, V3, World};

fn down(degrees: i32) -> i16 {
    (-degrees * 65536 / 360) as i16
}

fn run(w: &mut World, frames: u32, bits: u16, pitch: i16) {
    for _ in 0..frames {
        w.advance([Input::looking_at(bits, 0, pitch), Input::default()]);
    }
}

fn with_scene<T>(w: &World, ask: impl FnOnce(&aim::Scene) -> T) -> T {
    let stones = sim::stones::gather(&w.players);
    let players = w.players;
    let effects = w.effects;
    ask(&aim::Scene {
        stones: &stones,
        players: &players,
        effects: &effects,
        quarry: &w.monsters,
        critters: &sim::critter::Critters::NONE,
        arena: &w.terrain(),
    })
}

fn stones_of(w: &World) -> Vec<Structure> {
    let Mechanic::Structures(slots) = w.players[0].mechanic else {
        panic!("not the class that owns stones");
    };
    slots.iter().flatten().copied().collect()
}

fn f(x: Fx) -> f32 {
    x.to_f32_for_render()
}

// ---------------------------------------------------------------------------
// Aiming through a stone
// ---------------------------------------------------------------------------

#[test]
fn a_crosshair_through_a_stone_from_above_aims_at_the_floor_behind_it() {
    // Up on a ledge, looking down past a stone of hers at the floor: the ray
    // goes through the stone, so the shot goes to the floor -- not to the
    // stone's top, which was the dead zone.
    let mut w = World::with_classes([Class::Elementalist, Class::Bulwark]);
    w.players[0].pos = V3::new(Fx::from_int(-6), Fx::ZERO, Fx::from_int(8));
    w.players[1].pos = V3::new(Fx::from_int(10), Fx::ZERO, Fx::from_int(-8));
    run(&mut w, 2, keys::EARTH, down(30));
    run(&mut w, 40, 0, down(30));
    let stone = stones_of(&w)[0];
    let top = V3::new(stone.at.x, stone.top(), stone.at.z);

    // Six metres up, a few metres back from the stone, aimed at its lid.
    let p = &mut w.players[0];
    p.pos = V3::new(stone.at.x.sub(Fx::from_int(5)), Fx::from_int(6), stone.at.z);
    p.grounded = true;
    let p = w.players[0];
    let pitch = aim::look_onto_closely(p.pos, 0, p.aloft, p.clear, top);
    let look = Input::looking_at(0, 0, pitch);

    for (name, seen) in [
        (
            "an attack",
            with_scene(&w, |s| aim::sight_for_attack(0, look, Fx::from_int(40), s)),
        ),
        (
            "a placement",
            with_scene(&w, |s| aim::sight(0, look, Fx::from_int(40), s)),
        ),
    ] {
        assert!(
            f(seen.at.y) < 0.05,
            "{name}: the crosshair through the stone's lid stopped {:.2} m up, on the stone",
            f(seen.at.y)
        );
        assert!(
            seen.at.x.raw() > stone.at.x.raw(),
            "{name}: the aim point is short of the stone it was aimed through"
        );
        assert_eq!(seen.met, aim::Met::Ground, "{name}: the floor is ground");
    }
    let shot = with_scene(&w, |s| aim::skillshot_path(0, look, Fx::from_int(40), s));
    assert!(
        shot.to.x.raw() > stone.at.x.raw() && f(shot.to.y) < f(stone.top()),
        "the shot aimed through the stone went to ({:.2}, {:.2}), on the stone",
        f(shot.to.x),
        f(shot.to.y)
    );
}

// ---------------------------------------------------------------------------
// The tilt
// ---------------------------------------------------------------------------

/// A walk log with `key` (0 W, 1 A, 2 S, 3 D) down on each of `frames` --
/// frames ago, 0 being now.
fn log_of(key: u32, frames: impl IntoIterator<Item = u32>) -> u128 {
    frames
        .into_iter()
        .fold(0u128, |log, k| log | 1 << (4 * k + key))
}

#[test]
fn a_key_held_through_the_window_is_a_full_tilt_and_a_tap_is_a_share() {
    let window = t::updraft_tilt_window();
    let press = 14;
    // W held from the press to now.
    let (x, z) = updraft_tilt(log_of(0, 0..=press), press);
    assert_eq!((x, z), (Fx::ZERO, Fx::ONE), "held W is a full tilt forward");
    // Two frames of W, at the press.
    let (x, z) = updraft_tilt(log_of(0, [press, press - 1]), press);
    assert_eq!(x, Fx::ZERO);
    assert_eq!(
        z,
        Fx::ratio(2, window as i32),
        "a two-frame tap is two frames' share"
    );
    // Pressed a moment *before* the click still counts.
    let early = press + t::updraft_tilt_before();
    let (_, z) = updraft_tilt(log_of(0, early + 1 - window..=early), press);
    assert_eq!(
        z,
        Fx::ONE,
        "a key held just before the press is a full tilt"
    );
    // Too early is nothing.
    let (_, z) = updraft_tilt(log_of(0, [early + 2, early + 3]), press);
    assert_eq!(
        z,
        Fx::ZERO,
        "a key let go before the window opened counts for nothing"
    );
    // Nothing held is no tilt; a diagonal is no longer than a straight.
    assert_eq!(updraft_tilt(0, press), (Fx::ZERO, Fx::ZERO));
    let both = log_of(0, 0..=press) | log_of(3, 0..=press);
    let (x, z) = updraft_tilt(both, press);
    let long = V3::new(x, Fx::ZERO, z).flat_len();
    assert!(
        (f(long) - 1.0).abs() < 0.01,
        "W and D held together is a tilt {:.3} long, not one",
        f(long)
    );
}

#[test]
fn the_updraft_in_the_air_goes_further_the_longer_the_key_was_held() {
    // The same Updraft from the same height, with W tapped for 0, 2 and all
    // of its frames: the push grows with the hold, and none of them is
    // all-or-nothing.
    let push = |held: u32| {
        let mut w = World::with_classes([Class::Elementalist, Class::Bulwark]);
        w.players[0].pos = V3::new(Fx::from_int(-6), Fx::from_int(8), Fx::from_int(8));
        w.players[1].pos = V3::new(Fx::from_int(10), Fx::ZERO, Fx::from_int(-8));
        w.players[0].grounded = false;
        w.players[0].rise.used = false;
        let updraft = Input::SPACE | keys::WIND;
        for frame in 0..40u32 {
            let walk = if frame < held { Input::W } else { 0 };
            let click = if frame < 2 { updraft } else { 0 };
            w.advance([Input::looking_at(walk | click, 0, 0), Input::default()]);
        }
        f(w.players[0].vel.x)
    };
    let (none, tap, held) = (push(0), push(2), push(40));
    assert!(none.abs() < 0.5, "no key, and she was pushed {none:.2} m/s");
    assert!(
        tap > none + 0.5 && held > tap + 2.0,
        "the push should grow with the hold: none {none:.2}, a tap {tap:.2}, held {held:.2} m/s"
    );
}

// ---------------------------------------------------------------------------
// The lip
// ---------------------------------------------------------------------------

#[test]
fn a_body_coming_down_on_the_lip_of_a_box_lands_on_it() {
    let w = World::with_classes([Class::Champion, Class::Bulwark]);
    let ground = w.terrain();
    let platform = ground
        .solids()
        .find(|s| {
            s.max.y.raw() > Fx::ONE.raw()
                && s.max.x.sub(s.min.x).raw() > Fx::from_int(2).raw()
                && s.max.z.sub(s.min.z).raw() > Fx::from_int(2).raw()
        })
        .expect("a platform in the proving ground");
    let top = platform.max.y;
    let z = Fx::from_raw((platform.min.z.raw() + platform.max.z.raw()) / 2);
    // A tenth of a metre of the body over the top, feet a skin under it,
    // falling at a quarter of a metre a frame.
    let x = platform.min.x.sub(t::body_radius()).add(Fx::ratio(1, 10));
    let fall = V3::new(Fx::ZERO, Fx::from_int(-15), Fx::ZERO);
    let at = V3::new(x, top.sub(Fx::ratio(5, 100)), z);
    let r = ground.resolve(at, fall, false);
    assert!(
        r.grounded && r.pos.y == top,
        "a body over the lip, coming down, was put at ({:.2}, {:.2}) off the side of a top at {:.2}",
        f(r.pos.x),
        f(r.pos.y),
        f(top)
    );
    // From below it is still a wall: a body under the top a frame ago is put
    // off the side, not up on to it.
    let under = V3::new(x, top.sub(Fx::ONE), z);
    let r = ground.resolve(under, fall, false);
    assert!(
        r.pos.y.raw() < top.raw(),
        "a body from below the top was put up on to it"
    );
}

// ---------------------------------------------------------------------------
// The columns
// ---------------------------------------------------------------------------

/// A versus stage with an updraft, and the updraft.
fn a_stage_with_a_vent() -> (sim::arena::ArenaId, sim::valley::Vent) {
    sim::versus::STAGES
        .iter()
        .find_map(|s| s.vents.first().map(|v| (s.arena, *v)))
        .expect("a versus stage with an updraft")
}

#[test]
fn a_column_draws_a_body_toward_its_middle_and_lets_it_walk_out() {
    let (arena, vent) = a_stage_with_a_vent();
    let place = |w: &mut World, out: Fx, speed: Fx| {
        let p = &mut w.players[0];
        p.pos = V3::new(
            vent.at.0.add(out),
            vent.from.max(Fx::ZERO).add(Fx::ONE),
            vent.at.1,
        );
        p.vel = V3::new(speed, Fx::ZERO, Fx::ZERO);
        p.grounded = false;
    };
    let away = |w: &World| f(w.players[0].pos.x.sub(vent.at.0));

    // Flying in across the column at a run and letting go: the column takes
    // the speed off and holds her, rather than letting her sail out the far
    // side.
    let mut w = World::versus_in([Class::Elementalist, Class::Bulwark], arena);
    place(&mut w, vent.radius.mul(Fx::ratio(-9, 10)), Fx::from_int(4));
    for _ in 0..60 {
        w.advance([Input::looking_at(0, 0, 0), Input::default()]);
    }
    assert!(
        away(&w).abs() < f(vent.radius),
        "flown in at 4 m/s and let go, she is {:.2} m from the middle of a column {:.2} wide",
        away(&w),
        f(vent.radius)
    );

    // Holding the way out: she leaves it.
    let mut w = World::versus_in([Class::Elementalist, Class::Bulwark], arena);
    place(&mut w, Fx::ZERO, Fx::ZERO);
    for _ in 0..90 {
        w.advance([Input::looking_at(Input::W, 0, 0), Input::default()]);
    }
    assert!(
        away(&w) > f(vent.radius),
        "holding out of the column she is still {:.2} m from its middle, inside a radius of {:.2}",
        away(&w),
        f(vent.radius)
    );
}

#[test]
fn a_column_calls_off_landfall() {
    let (arena, vent) = a_stage_with_a_vent();
    let mut w = World::versus_in([Class::Elementalist, Class::Bulwark], arena);
    let p = &mut w.players[0];
    p.pos = V3::new(
        vent.at.0,
        vent.from.max(Fx::ZERO).add(Fx::from_int(2)),
        vent.at.1,
    );
    p.grounded = false;
    // Landfall is the earth click in the air.
    w.advance([Input::looking_at(keys::EARTH, 0, 0), Input::default()]);
    let mut plunged = false;
    for _ in 0..30 {
        w.advance([Input::looking_at(0, 0, 0), Input::default()]);
        if let sim::state::Action::Startup { kind, .. } = w.players[0].action {
            plunged |= kind == sim::moves::elementalist::LANDFALL && w.players[0].vel.y.raw() > 0;
        }
    }
    assert!(
        !matches!(
            w.players[0].action,
            sim::state::Action::Startup { kind, .. } if kind == sim::moves::elementalist::LANDFALL
        ),
        "carried up by the column she is still in Landfall's plunge"
    );
    assert!(
        !plunged,
        "the column carried her up with the plunge still owed"
    );
}

// ---------------------------------------------------------------------------
// The jump is a press
// ---------------------------------------------------------------------------

/// Drop a Champion from `height` and feed it `space(frame)`; how many frames
/// after the landing it was back in the air, if it was.
fn hops_after_landing(height: i32, space: impl Fn(u32) -> bool) -> Option<u32> {
    let mut w = World::with_classes([Class::Champion, Class::Bulwark]);
    w.players[0].pos = V3::new(Fx::from_int(-6), Fx::from_int(height), Fx::from_int(8));
    w.players[1].pos = V3::new(Fx::from_int(10), Fx::ZERO, Fx::from_int(-8));
    w.players[0].grounded = false;
    let mut landed = None;
    for frame in 0..120u32 {
        let bits = if space(frame) { Input::SPACE } else { 0 };
        w.advance([Input::looking_at(bits, 0, 0), Input::default()]);
        let p = &w.players[0];
        match landed {
            None if p.grounded => landed = Some(frame),
            Some(at) if !p.grounded && p.vel.y.raw() > 0 => return Some(frame - at),
            _ => {}
        }
    }
    None
}

#[test]
fn space_held_through_a_landing_does_not_jump_again() {
    // Held from the top of the fall: she lands, and stays.
    assert_eq!(
        hops_after_landing(3, |_| true),
        None,
        "space held from before the fall threw her straight back off the floor"
    );
}

#[test]
fn space_pressed_just_before_a_landing_jumps_on_it() {
    // The fall from three metres takes this long; a press a few frames
    // before the feet arrive is the buffer, and jumps on landing.
    let mut w = World::with_classes([Class::Champion, Class::Bulwark]);
    w.players[0].pos = V3::new(Fx::from_int(-6), Fx::from_int(3), Fx::from_int(8));
    w.players[0].grounded = false;
    let mut fall = 0;
    while !w.players[0].grounded {
        w.advance([Input::default(), Input::default()]);
        fall += 1;
    }
    let early = t::jump_buffer().saturating_sub(2);
    let press = fall - 1 - early;
    let hop = hops_after_landing(3, |f| f >= press);
    assert!(
        hop.is_some_and(|h| h <= 2),
        "space pressed {early} frames before landing hopped {hop:?} frames after it"
    );
}
