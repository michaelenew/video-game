//! The Bulwark on three clicks (2026-10-10): what a player would notice if
//! any of it broke.
//!
//! `docs/design/exploration/0012_bulwark_on_three_clicks.md` is the design.
//! Left is the strike -- the shield as a weapon; middle is the weight -- the
//! battery spent; right is the guard -- the shield as a thing. His air game is
//! about being thrown: the shield is a springboard, a sail and a battery.

use sim::class::{Mechanic, Shield};
use sim::moves::bulwark as b;
use sim::moves::bulwark::keys;
use sim::tuning as t;
use sim::{Class, Fx, Input, V3, World};

const SPACE: u16 = Input::SPACE;

fn metres(v: f32) -> Fx {
    Fx::ratio((v * 1000.0) as i32, 1000)
}

fn at(x: f32, y: f32, z: f32) -> V3 {
    V3::new(metres(x), metres(y), metres(z))
}

fn as_f(v: Fx) -> f32 {
    v.raw() as f32 / 65536.0
}

/// A Bulwark facing +X in the open and a Reaver `gap` metres in front of him.
fn facing(gap: f32) -> World {
    let mut w = World::with_classes([Class::Bulwark, Class::ShadowReaver]);
    w.players[0].pos = at(-6.0, 0.0, 8.0);
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    w.players[1].pos = at(-6.0 + gap, 0.0, 8.0);
    w.players[1].facing = V3::new(Fx::ONE.neg(), Fx::ZERO, Fx::ZERO);
    w
}

fn alone() -> World {
    let mut w = facing(0.0);
    w.players[1].pos = at(11.0, 0.0, -10.0);
    w
}

fn run(w: &mut World, frames: u32, bits: u16, pitch: i16) {
    for _ in 0..frames {
        w.advance([Input::looking_at(bits, 0, pitch), Input::default()]);
    }
}

fn step(w: &mut World, bits: u16) {
    run(w, 1, bits, 0);
}

fn aloft(w: &mut World, height: f32) {
    w.players[0].pos.y = metres(height);
    w.players[0].grounded = false;
}

fn doing(w: &World) -> Option<u8> {
    w.players[0].action.attack_kind()
}

fn load(w: &mut World, weight: Fx) {
    w.players[0].mechanic = Mechanic::Shield(Shield::Held { weight });
}

fn hurt(w: &World) -> i32 {
    w.players[1].full_health() - w.players[1].health
}

/// The highest he gets in `frames`.
fn top(w: &mut World, frames: u32) -> Fx {
    let mut top = w.players[0].pos.y;
    for _ in 0..frames {
        step(w, 0);
        top = top.max(w.players[0].pos.y);
    }
    top
}

// ---------------------------------------------------------------------------
// The map
// ---------------------------------------------------------------------------

#[test]
fn on_the_floor_left_is_bash_middle_is_slam_and_right_is_the_guard() {
    let mut w = alone();
    step(&mut w, keys::STRIKE);
    assert_eq!(doing(&w), Some(b::BASH));
    let mut w = alone();
    step(&mut w, keys::WEIGHT);
    assert_eq!(doing(&w), Some(b::SLAM));
    let mut w = alone();
    step(&mut w, keys::GUARD);
    assert!(
        w.players[0].action.guarding(),
        "right click is still the guard"
    );
}

#[test]
fn in_the_air_left_is_rebound_and_middle_is_still_the_slam() {
    let mut w = alone();
    aloft(&mut w, 3.0);
    step(&mut w, keys::STRIKE);
    assert_eq!(doing(&w), Some(b::REBOUND));
    let mut w = alone();
    aloft(&mut w, 3.0);
    step(&mut w, keys::WEIGHT);
    assert_eq!(doing(&w), Some(b::SLAM));
}

#[test]
fn space_and_a_click_is_a_takeoff() {
    for (button, wanted) in [
        (keys::STRIKE, b::RAM),
        (keys::WEIGHT, b::UNLOAD),
        (keys::GUARD, b::SHIELD_STEP),
    ] {
        let mut w = alone();
        step(&mut w, SPACE | button);
        assert_eq!(doing(&w), Some(wanted), "space and {button:#x}");
    }
}

// ---------------------------------------------------------------------------
// The strike
// ---------------------------------------------------------------------------

#[test]
fn a_rebound_off_a_body_throws_him_back_and_up() {
    let mut w = facing(1.2);
    aloft(&mut w, 1.0);
    let x0 = w.players[0].pos.x;
    step(&mut w, keys::STRIKE);
    let mut bounced = false;
    for _ in 0..20 {
        step(&mut w, 0);
        bounced |= w.players[0].vel.y.raw() > 0 && w.players[0].vel.x.raw() < 0;
    }
    assert!(bounced, "he bounced back and up");
    assert!(w.players[0].pos.x.raw() < x0.raw(), "away from what he hit");
    assert!(hurt(&w) > 0, "and it hit them");
}

#[test]
fn a_rebound_at_nothing_is_only_a_bash() {
    let mut w = alone();
    aloft(&mut w, 3.0);
    step(&mut w, keys::STRIKE);
    for _ in 0..10 {
        step(&mut w, 0);
        assert!(
            w.players[0].vel.y.raw() <= 0,
            "nothing to bounce off, and still he went up"
        );
    }
}

#[test]
fn the_battering_ram_goes_along_the_floor_and_takes_them_with_him() {
    let mut w = facing(2.0);
    let x0 = w.players[0].pos.x;
    let them = w.players[1].pos.x;
    step(&mut w, SPACE | keys::STRIKE);
    let high = top(&mut w, 30);
    assert!(hurt(&w) > 0, "the shield met them");
    assert!(
        w.players[0].pos.x.sub(x0).raw() > metres(2.5).raw(),
        "he went forward: {} m",
        as_f(w.players[0].pos.x.sub(x0))
    );
    assert!(
        high.raw() < metres(1.5).raw(),
        "low and flat: {} m",
        as_f(high)
    );
    assert!(
        w.players[1].pos.x.sub(them).raw() > metres(1.5).raw(),
        "and they went with him: {} m",
        as_f(w.players[1].pos.x.sub(them))
    );
}

// ---------------------------------------------------------------------------
// The weight
// ---------------------------------------------------------------------------

#[test]
fn unload_goes_higher_the_more_the_shield_holds_and_spends_it() {
    let mut empty = alone();
    step(&mut empty, SPACE | keys::WEIGHT);
    let low = top(&mut empty, 80);
    let mut full = alone();
    load(&mut full, t::weight_cap());
    step(&mut full, SPACE | keys::WEIGHT);
    let high = top(&mut full, 80);
    assert!(
        high.raw() > low.add(metres(2.0)).raw(),
        "full {} m against empty {} m",
        as_f(high),
        as_f(low)
    );
    assert!(
        sim::bulwark::weight(&full.players[0]).raw() == 0,
        "Unload spent the weight"
    );
}

// ---------------------------------------------------------------------------
// The guard
// ---------------------------------------------------------------------------

#[test]
fn the_guard_held_in_the_air_is_a_sail() {
    let fall = |guard: bool| {
        let mut w = alone();
        aloft(&mut w, 12.0);
        run(&mut w, 40, if guard { keys::GUARD } else { 0 }, 0);
        w.players[0].vel.y
    };
    let sailing = fall(true);
    assert!(
        sailing.raw() >= Fx::ZERO.sub(t::sail_fall()).raw(),
        "falling at {} m/s under the sail",
        as_f(sailing)
    );
    assert!(fall(false).raw() < sailing.raw(), "and faster without it");
}

#[test]
fn the_shield_step_plants_the_shield_and_springs_him_off_it() {
    let mut w = alone();
    load(&mut w, t::weight_cap());
    let x0 = w.players[0].pos.x;
    step(&mut w, SPACE | keys::GUARD);
    let high = top(&mut w, 12);
    assert!(
        matches!(w.players[0].shield(), Some(Shield::Planted { .. })),
        "the shield is planted"
    );
    let Some(Shield::Planted { pos, weight }) = w.players[0].shield() else {
        unreachable!()
    };
    assert_eq!(pos.x, x0, "where he stood");
    assert!(
        w.players[0].pos.x.raw() > x0.raw(),
        "and he went on forward off it"
    );
    assert!(
        weight.raw() > t::weight_cap().mul(Fx::ratio(9, 10)).raw(),
        "with the weight it held: {}",
        as_f(weight)
    );
    assert!(
        high.raw() > metres(2.5).raw(),
        "and he went up: {}",
        as_f(high)
    );
    // And the recall is the way back to having moves, in the air once the
    // spring's recovery is over.
    run(&mut w, 4, 0, 0);
    assert!(!w.players[0].grounded, "still in the air");
    run(&mut w, 2, Input::MECHANIC, 0);
    run(&mut w, 60, 0, 0);
    assert!(
        w.players[0].shield().is_some_and(|s| s.in_hand()),
        "`E` called it home"
    );
}

#[test]
fn without_the_shield_in_hand_space_and_right_click_is_only_a_jump() {
    let mut w = alone();
    step(&mut w, SPACE | keys::GUARD);
    run(&mut w, 40, 0, 0);
    step(&mut w, SPACE | keys::GUARD);
    assert_eq!(doing(&w), None);
}
