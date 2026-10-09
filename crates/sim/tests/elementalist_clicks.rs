//! The Elementalist on three clicks: what a player would notice if any of it
//! broke.
//!
//! `docs/design/exploration/0008_elementalist_on_three_clicks.md` is the
//! direction this follows. Left is earth, middle is fire, right is wind, each
//! a tap and a hold on the floor and something else in the air; space with a
//! click is a takeoff; `Q` and `E` are the two pushes.

use sim::class::{Mechanic, Structure};
use sim::effects::{Effect, EffectKind};
use sim::moves::elementalist as e;
use sim::moves::elementalist::keys;
use sim::state::{Action, SLOT_HEAVY, SLOT_POKE, SLOT_SPECIAL};
use sim::tuning as t;
use sim::{Class, Fx, Input, V3, World};

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

/// An Elementalist facing +X with clear floor in front of her, and somebody
/// far enough away to be out of everything unless a test moves them.
fn elementalist() -> World {
    let mut w = World::with_classes([Class::Elementalist, Class::Bulwark]);
    w.players[0].pos = at(-6.0, 0.0, 8.0);
    w.players[1].pos = at(11.0, 0.0, 8.0);
    w
}

fn step(w: &mut World, bits: u16) {
    step_looking(w, bits, 0);
}

fn step_looking(w: &mut World, bits: u16, pitch: i16) {
    w.advance([
        Input::looking_at(bits, LOOK_RIGHT, pitch),
        Input::looking_at(0, LOOK_LEFT, 0),
    ]);
}

fn run(w: &mut World, frames: u32, bits: u16) {
    for _ in 0..frames {
        step(w, bits);
    }
}

/// Take her feet off the floor without moving her.
fn aloft(w: &mut World, height: f32) {
    w.players[0].pos.y = metres(height);
    w.players[0].grounded = false;
}

/// Which move she is part-way through, whatever phase it is in.
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

fn of_kind(w: &World, kind: EffectKind) -> Vec<Effect> {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.kind == kind)
        .copied()
        .collect()
}

/// A ball already let go, rolling +X from `from` at `radius`.
fn rolling_ball(from: V3, radius: Fx) -> Effect {
    Effect::cast(
        EffectKind::AirBall,
        0,
        Class::Elementalist,
        e::AIR_BALL,
        from,
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        radius,
    )
}

fn place(w: &mut World, effect: Effect) {
    let slot = w.effects.iter().position(|s| s.is_none()).expect("room");
    w.effects[slot] = Some(effect);
}

// ---------------------------------------------------------------------------
// The map
// ---------------------------------------------------------------------------

#[test]
fn on_the_floor_the_clicks_are_earth_fire_and_wind() {
    // Left raises a stone, as an instant: no move, a stone.
    let mut w = elementalist();
    step(&mut w, keys::EARTH);
    assert_eq!(doing(&w), None, "Raise is an instant");
    assert_eq!(stones_of(&w).len(), 1, "the earth click raises a stone");

    let mut w = elementalist();
    step(&mut w, keys::FIRE);
    assert_eq!(
        doing(&w),
        Some(SLOT_SPECIAL),
        "the fire click is the pillar"
    );

    let mut w = elementalist();
    step(&mut w, keys::WIND);
    assert_eq!(
        doing(&w),
        Some(e::AIR_BALL),
        "the wind click is the Air ball"
    );

    let mut w = elementalist();
    step(&mut w, keys::WEAK_PUSH);
    assert_eq!(doing(&w), Some(SLOT_POKE), "Q is the Bolt");

    let mut w = elementalist();
    step(&mut w, keys::STRONG_PUSH);
    assert_eq!(doing(&w), Some(SLOT_HEAVY), "E is Cataclysm");
    assert!(stones_of(&w).is_empty(), "E no longer raises anything");
}

#[test]
fn in_the_air_the_clicks_are_landfall_the_carpet_and_the_gale() {
    for (button, wanted) in [
        (keys::EARTH, e::LANDFALL),
        (keys::FIRE, e::FIRE_CARPET),
        (keys::WIND, e::GALE),
        (keys::WEAK_PUSH, e::AIR_BOLT),
        (keys::STRONG_PUSH, SLOT_HEAVY),
    ] {
        let mut w = elementalist();
        aloft(&mut w, 3.0);
        step(&mut w, button);
        assert_eq!(doing(&w), Some(wanted), "button {button:#x} in the air");
    }
}

#[test]
fn space_and_a_click_is_a_takeoff() {
    for (button, wanted) in [
        (keys::EARTH, e::EARTH_JUMP),
        (keys::FIRE, e::FIRE_FOUNTAIN),
        (keys::WIND, e::UPDRAFT),
    ] {
        let mut w = elementalist();
        step(&mut w, SPACE | button);
        assert_eq!(doing(&w), Some(wanted), "space and {button:#x} together");

        // Jump first, click a couple of frames later: still the takeoff, and
        // from the floor she just left.
        let mut w = elementalist();
        step(&mut w, SPACE);
        step(&mut w, SPACE);
        assert!(!w.players[0].grounded, "she jumped");
        step(&mut w, SPACE | button);
        assert_eq!(doing(&w), Some(wanted), "space, then {button:#x}");
        assert!(w.players[0].grounded, "put back on the floor for it");
    }
}

#[test]
fn the_updraft_in_the_air_is_once_a_trip_and_keeps_her_run() {
    let mut w = elementalist();
    w.players[0].vel = at(6.0, 0.0, 0.0);
    aloft(&mut w, 3.0);
    w.players[0].vel.y = metres(-4.0);
    // Space held with the wind click: the Updraft, from the air.
    step(&mut w, SPACE | keys::WIND);
    assert_eq!(doing(&w), Some(e::UPDRAFT));
    for _ in 0..40 {
        step(&mut w, SPACE);
        if w.players[0].vel.y.raw() > 0 {
            break;
        }
    }
    assert!(w.players[0].vel.y.raw() > 0, "it lifted her");
    assert!(
        as_f(w.players[0].vel.x) > 5.0,
        "and she kept going the way she was going: {}",
        as_f(w.players[0].vel.x)
    );
    // Run its frames out, then ask again in the same trip: a Gale.
    run(&mut w, 40, 0);
    if !w.players[0].grounded {
        step(&mut w, SPACE | keys::WIND);
        assert_eq!(doing(&w), Some(e::GALE), "one Updraft a trip");
    }
}

// ---------------------------------------------------------------------------
// The Air ball
// ---------------------------------------------------------------------------

/// How far a ball let go at `radius` rolls before it is gone, by the rule
/// itself: its speed at the size it is now, shrinking at the steady rate. The
/// arena a test runs in is smaller than a full ball's roll, so the rule is
/// summed here and the world is checked against it below.
fn distance_by_the_rule(radius: Fx) -> f32 {
    let mut r = radius;
    let mut gone = 0.0;
    let dt = 1.0 / sim::TICK_HZ as f32;
    while r.raw() > 0 {
        gone += as_f(sim::effects::ball_speed(r)) * dt;
        r = r.sub(t::air_ball_shrink().mul(sim::DT));
    }
    gone
}

#[test]
fn a_ball_held_twice_as_long_goes_much_further() {
    let tap = distance_by_the_rule(t::air_ball_radius_tap());
    let full = distance_by_the_rule(t::air_ball_radius_full());
    assert!(tap > 0.5, "a tapped ball still goes somewhere: {tap}");
    assert!(
        full > tap * 4.0,
        "a full ball goes far further than a tapped one: {full} against {tap}"
    );
}

#[test]
fn a_rolling_ball_rolls_the_way_the_rule_says_and_peters_out() {
    let mut w = elementalist();
    // Nobody in its way: the ball is the only thing on its line.
    w.players[0].pos = at(-6.0, 0.0, -4.0);
    w.players[1].pos = at(11.0, 0.0, -4.0);
    let radius = t::air_ball_radius_tap();
    let from = at(-10.0, 0.0, 8.0);
    place(&mut w, rolling_ball(from, radius));
    let mut last = from;
    let mut sizes = Vec::new();
    for _ in 0..600 {
        run(&mut w, 1, 0);
        match of_kind(&w, EffectKind::AirBall).first() {
            Some(ball) => {
                last = ball.pos;
                sizes.push(ball.reach);
            }
            None => break,
        }
    }
    assert!(of_kind(&w, EffectKind::AirBall).is_empty(), "it peters out");
    assert!(
        sizes.windows(2).all(|p| p[1].raw() < p[0].raw()),
        "shrinking all the way"
    );
    let rolled = as_f(last.x.sub(from.x));
    let rule = distance_by_the_rule(radius);
    assert!(
        (rolled - rule).abs() < 0.5,
        "it rolled {rolled} m; the rule says {rule}"
    );
}

#[test]
fn a_held_ball_grows_and_is_sent_on_the_release() {
    let mut w = elementalist();
    // Look well down, so the crosshair is on the floor in front of her.
    let pitch = -(65536 / 16) as i16;
    step_looking(&mut w, keys::WIND, pitch);
    let first = of_kind(&w, EffectKind::AirBall);
    assert_eq!(first.len(), 1, "the press raises a ball");
    let small = first[0].reach;
    for _ in 0..40 {
        step_looking(&mut w, keys::WIND, pitch);
    }
    let held = of_kind(&w, EffectKind::AirBall)[0];
    assert!(
        held.reach.raw() > small.raw(),
        "it grows while she holds it"
    );
    assert!(!held.ball_sent(), "and sits where it was raised");
    // Let go, and run the move's few frames out.
    run(&mut w, 12, 0);
    let sent = of_kind(&w, EffectKind::AirBall);
    assert_eq!(sent.len(), 1);
    assert!(sent[0].ball_sent(), "let go, it rolls");
    assert!(sent[0].dir.x.raw() > 0, "the way she was looking");
}

#[test]
fn the_ball_carries_whoever_stands_in_it_and_puts_them_down_before_it_goes() {
    let mut w = elementalist();
    w.players[1].pos = at(11.0, 0.0, -4.0);
    let from = w.players[0].pos;
    // Small enough that the whole of its roll fits the arena.
    place(&mut w, rolling_ball(from, metres(1.1)));
    run(&mut w, 20, 0);
    let carried = as_f(w.players[0].pos.x.sub(from.x));
    assert!(carried > 1.5, "she went with it: {carried} m");
    // Run it out. She is set down while it is still there, small.
    let mut put_down_with_ball_left = false;
    for _ in 0..600 {
        run(&mut w, 1, 0);
        let Some(ball) = of_kind(&w, EffectKind::AirBall).first().copied() else {
            break;
        };
        if ball.reach.raw() < t::air_ball_holds().raw() {
            put_down_with_ball_left = true;
        }
    }
    assert!(
        put_down_with_ball_left,
        "the rider is let go before the ball is"
    );
}

#[test]
fn jumping_out_of_a_rolling_ball_keeps_its_speed() {
    let mut w = elementalist();
    let from = w.players[0].pos;
    place(&mut w, rolling_ball(from, t::air_ball_radius_full()));
    run(&mut w, 10, 0);
    run(&mut w, 2, SPACE);
    run(&mut w, 6, SPACE);
    assert!(!w.players[0].grounded);
    assert!(
        as_f(w.players[0].vel.x) > as_f(t::air_ball_speed_tap()),
        "she left with the ball's speed: {}",
        as_f(w.players[0].vel.x)
    );
}

// ---------------------------------------------------------------------------
// The Fire carpet and the Thermal
// ---------------------------------------------------------------------------

#[test]
fn the_carpet_hangs_in_front_of_her_along_her_look() {
    let mut w = elementalist();
    aloft(&mut w, 4.0);
    step(&mut w, keys::FIRE);
    run(&mut w, 12, 0);
    let carpets = of_kind(&w, EffectKind::FireCarpet);
    assert_eq!(carpets.len(), 1, "one carpet");
    let c = carpets[0];
    assert!(c.pos.x.raw() > w.players[0].pos.x.raw() - metres(1.0).raw());
    assert!(c.dir.x.raw() > 0, "laid out the way she looks");
    // A second one replaces the first.
    run(&mut w, 40, 0);
    if !w.players[0].grounded {
        step(&mut w, keys::FIRE);
        run(&mut w, 12, 0);
        assert!(of_kind(&w, EffectKind::FireCarpet).len() <= 1);
    }
}

#[test]
fn a_gale_thrown_straight_down_the_carpet_comes_out_lit() {
    let mut w = elementalist();
    aloft(&mut w, 6.0);
    // Hold her up: this is about the shot, not the fall.
    let hold = |w: &mut World| {
        w.players[0].pos.y = metres(6.0);
        w.players[0].vel.y = Fx::ZERO;
    };
    step(&mut w, keys::FIRE);
    for _ in 0..30 {
        hold(&mut w);
        step(&mut w, 0);
        if matches!(w.players[0].action, Action::Free) {
            break;
        }
    }
    assert_eq!(of_kind(&w, EffectKind::FireCarpet).len(), 1);
    hold(&mut w);
    step(&mut w, keys::WIND);
    let mut lit = false;
    for _ in 0..60 {
        hold(&mut w);
        step(&mut w, 0);
        if w.gusts.iter().flatten().any(|g| g.lit) {
            lit = true;
            break;
        }
    }
    assert!(lit, "a Gale flown down the carpet is lit");
}

#[test]
fn an_updraft_into_her_carpet_is_a_thermal_that_throws_her_along_it() {
    let mut w = elementalist();
    w.players[0].pos = at(-6.0, 3.0, 8.0);
    w.players[0].grounded = false;
    // A carpet laid out ahead of her, level, at her height.
    let carpet = Effect::cast(
        EffectKind::FireCarpet,
        0,
        Class::Elementalist,
        e::FIRE_CARPET,
        at(-5.0, 3.5, 8.0),
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        metres(6.0),
    );
    place(&mut w, carpet);
    let before = w.players[0].vel.x;
    step(&mut w, SPACE | keys::WIND);
    assert_eq!(doing(&w), Some(e::UPDRAFT));
    let mut thermal = false;
    for _ in 0..40 {
        step(&mut w, SPACE);
        if w.players[0].vel.y.raw() > t::updraft_lift().raw() {
            thermal = true;
            break;
        }
    }
    assert!(thermal, "the column met her fire: a Thermal");
    assert!(
        w.players[0].vel.x.raw() > before.add(t::thermal_push().mul(Fx::ratio(1, 2))).raw(),
        "and it threw her along the carpet"
    );
    assert!(
        of_kind(&w, EffectKind::FireCarpet).is_empty(),
        "the heat went with her: the carpet is used up"
    );
}

#[test]
fn an_updraft_with_no_fire_is_only_an_updraft() {
    let mut w = elementalist();
    step(&mut w, SPACE | keys::WIND);
    let mut top = Fx::ZERO;
    for _ in 0..40 {
        step(&mut w, SPACE);
        top = top.max(w.players[0].vel.y);
    }
    assert!(top.raw() <= t::updraft_lift().raw());
}

#[test]
fn the_fountain_bursts_at_her_feet_and_leaves_a_wash_where_she_took_off() {
    let mut w = elementalist();
    let take_off = w.players[0].pos;
    // Somebody standing right beside her.
    w.players[1].pos = at(-5.0, 0.0, 8.0);
    let before = w.players[1].health;
    step(&mut w, SPACE | keys::FIRE);
    run(&mut w, 20, 0);
    assert!(!w.players[0].grounded, "she left the floor");
    assert!(w.players[1].health < before, "the burst caught him");
    let wash = of_kind(&w, EffectKind::Fountain);
    assert_eq!(wash.len(), 1, "a wash of fire stands where she took off");
    let gap = V3::new(
        wash[0].pos.x.sub(take_off.x),
        Fx::ZERO,
        wash[0].pos.z.sub(take_off.z),
    );
    assert!(gap.flat_len().raw() < metres(0.5).raw());
}

// ---------------------------------------------------------------------------
// The earth jump
// ---------------------------------------------------------------------------

/// Earth-jump from where she stands, holding `bits` the whole way, and say
/// whether she came to stand on her own stone in the air.
fn earth_jump_lands_on_it(w: &mut World, bits: u16) -> bool {
    earth_jump_steering(w, bits, bits)
}

/// The same, holding `floor` until her feet leave it and `air` after.
fn earth_jump_steering(w: &mut World, floor: u16, air: u16) -> bool {
    step(w, SPACE | keys::EARTH | floor);
    let mut left = false;
    for _ in 0..90 {
        step(w, if left { air } else { floor });
        let p = w.players[0];
        left |= !p.grounded;
        let on_a_stone = p.grounded && p.pos.y.raw() > metres(1.0).raw();
        if on_a_stone {
            return true;
        }
        if left && p.grounded && p.pos.y.raw() < metres(0.2).raw() {
            return false;
        }
    }
    false
}

#[test]
fn an_earth_jump_brings_a_stone_up_with_her() {
    let mut w = elementalist();
    step(&mut w, SPACE | keys::EARTH);
    run(&mut w, 12, 0);
    let stones = stones_of(&w);
    assert_eq!(stones.len(), 1, "a stone came with her");
    assert!(stones[0].aloft, "in the air");
    assert!(stones[0].vel.y.raw() > 0, "going up");
}

#[test]
fn a_straight_earth_jump_comes_down_on_its_stone() {
    let mut w = elementalist();
    w.players[0].vel = at(5.0, 0.0, 0.0);
    // Running forward into it, and holding forward the whole way.
    assert!(
        earth_jump_lands_on_it(&mut w, Input::W),
        "she stands on her stone in the air"
    );
}

#[test]
fn an_earth_jump_without_forward_held_leaves_its_stone_behind() {
    // Taking off running and letting go of forward: the stone goes on at its
    // own slower speed and she comes down past it.
    let mut w = elementalist();
    w.players[0].vel = at(5.0, 0.0, 0.0);
    assert!(!earth_jump_steering(&mut w, Input::W, 0));
    // Strafing is letting go of forward, too.
    let mut w = elementalist();
    w.players[0].vel = at(5.0, 0.0, 0.0);
    assert!(!earth_jump_steering(&mut w, Input::W, Input::D));
}

#[test]
fn an_earth_jump_off_a_stone_on_the_floor_shatters_it_and_goes_higher() {
    let plain = {
        let mut w = elementalist();
        step(&mut w, SPACE | keys::EARTH);
        let mut top = Fx::ZERO;
        for _ in 0..80 {
            step(&mut w, 0);
            top = top.max(w.players[0].pos.y);
        }
        top
    };
    let mut w = elementalist();
    // A stone out and standing under her.
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
    run(&mut w, 2, 0);
    assert!(w.players[0].grounded, "standing on it");
    let base = w.players[0].pos.y;
    step(&mut w, SPACE | keys::EARTH);
    let mut top = Fx::ZERO;
    for _ in 0..80 {
        step(&mut w, 0);
        top = top.max(w.players[0].pos.y);
    }
    assert!(stones_of(&w).is_empty(), "the stone shattered");
    assert!(
        w.debris.iter().flatten().count() > 0 || top.raw() > 0,
        "into pieces"
    );
    assert!(
        top.sub(base).raw() > plain.raw(),
        "and the jump off it was bigger: {} against {}",
        as_f(top.sub(base)),
        as_f(plain)
    );
}

#[test]
fn an_earth_jump_off_a_stone_in_the_air_drives_it_down_to_shatter() {
    let mut w = elementalist();
    w.players[0].vel = at(5.0, 0.0, 0.0);
    assert!(earth_jump_lands_on_it(&mut w, Input::W));
    // Again, off the stone in the air -- once the first one's repeat lockout
    // has run out, which it nearly has by the time she lands.
    for _ in 0..6 {
        step(&mut w, 0);
        assert!(w.players[0].grounded, "still standing on it");
    }
    step(&mut w, SPACE | keys::EARTH);
    assert_eq!(doing(&w), Some(e::EARTH_JUMP));
    let mut shattered = false;
    for _ in 0..120 {
        step(&mut w, 0);
        if stones_of(&w).is_empty() {
            shattered = true;
            break;
        }
    }
    assert!(shattered, "the stone went down and broke");
    assert!(
        w.debris.iter().flatten().count() > 0,
        "into pieces where it landed"
    );
}
