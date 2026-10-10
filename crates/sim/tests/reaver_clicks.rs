//! The Shadow Reaver on three clicks (2026-10-09): what a player would notice
//! if any of it broke.
//!
//! `docs/design/exploration/0011_shadow_reaver_on_three_clicks.md` is the
//! design. Left is the blade -- quick cuts the tally is built on; middle is
//! the execution -- committed cuts that come down onto a body; right is the
//! shadow -- where the other body is. `E` is Deadly mistake, the counter.

use sim::class::{Ghost, Mechanic, Shadow};
use sim::moves::reaver as r;
use sim::moves::reaver::keys;
use sim::state::{Action, SLOT_MECHANIC};
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

fn down(degrees: i32) -> i16 {
    (-degrees * 65536 / 360) as i16
}

/// A Reaver facing +X in the open and a Bulwark `gap` metres in front of
/// her, facing her; the shadow settled at her heel.
fn facing(gap: f32) -> World {
    let mut w = World::with_classes([Class::ShadowReaver, Class::Bulwark]);
    w.players[0].pos = at(-6.0, 0.0, 8.0);
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    w.players[1].pos = at(-6.0 + gap, 0.0, 8.0);
    w.players[1].facing = V3::new(Fx::ONE.neg(), Fx::ZERO, Fx::ZERO);
    run(&mut w, 30, 0, 0);
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

fn shadow(w: &World) -> Shadow {
    match w.players[0].mechanic {
        Mechanic::Shadow(s) => s,
        _ => panic!("player one is not the Reaver"),
    }
}

fn hurt(w: &World) -> i32 {
    w.players[1].full_health() - w.players[1].health
}

/// Send the shadow `d` metres ahead along the floor and let it settle.
fn send_ahead(w: &mut World) {
    run(w, 2, keys::SHADOW, down(20));
    run(w, 40, 0, 0);
    assert!(
        matches!(shadow(w).doing, Ghost::Waiting),
        "the shadow did not go out"
    );
}

// ---------------------------------------------------------------------------
// The map
// ---------------------------------------------------------------------------

#[test]
fn on_the_floor_the_clicks_are_slash_executioner_and_the_send() {
    for (button, wanted) in [
        (keys::BLADE, r::SLASH),
        (keys::EXECUTION, r::EXECUTIONER),
        (keys::SHADOW, r::SEND),
    ] {
        let mut w = alone();
        step(&mut w, button);
        assert_eq!(doing(&w), Some(wanted), "click {button:#x}");
    }
    assert_eq!(
        r::SEND,
        SLOT_MECHANIC,
        "the send is still her mechanic slot"
    );
}

#[test]
fn in_the_air_the_clicks_are_the_kite_the_drop_and_the_swap() {
    for (button, wanted) in [
        (keys::BLADE, r::KITE_CUT),
        (keys::EXECUTION, r::GUILLOTINE_DROP),
    ] {
        let mut w = alone();
        aloft(&mut w, 3.0);
        step(&mut w, button);
        assert_eq!(doing(&w), Some(wanted), "click {button:#x} in the air");
    }
    // With the shadow at her shoulder, right click in the air is the send.
    let mut w = alone();
    aloft(&mut w, 3.0);
    step(&mut w, keys::SHADOW);
    assert_eq!(doing(&w), Some(r::SEND), "attending: the air click sends");
    // With it out and waiting, it is the swap.
    let mut w = alone();
    send_ahead(&mut w);
    aloft(&mut w, 3.0);
    step(&mut w, keys::SHADOW);
    assert_eq!(doing(&w), Some(r::SWAP), "waiting: the air click swaps");
}

#[test]
fn space_and_a_click_is_a_takeoff() {
    for (button, wanted) in [
        (keys::BLADE, r::MOONSAULT),
        (keys::EXECUTION, r::GALLOWS),
        (keys::SHADOW, r::HANG),
    ] {
        let mut w = alone();
        step(&mut w, SPACE | button);
        assert_eq!(doing(&w), Some(wanted), "space and {button:#x}");
    }
}

#[test]
fn e_is_deadly_mistake_and_q_is_still_the_lotus() {
    let mut w = alone();
    step(&mut w, keys::MISTAKE);
    assert_eq!(doing(&w), Some(r::DEADLY_MISTAKE));
    let mut w = alone();
    step(&mut w, keys::LOTUS);
    assert_eq!(doing(&w), Some(r::LOTUS));
}

// ---------------------------------------------------------------------------
// The blade
// ---------------------------------------------------------------------------

#[test]
fn the_moonsault_goes_up_and_back_and_launches() {
    let mut w = facing(1.6);
    let x0 = w.players[0].pos.x;
    step(&mut w, SPACE | keys::BLADE);
    let mut highest = Fx::ZERO;
    let mut theirs = Fx::ZERO;
    for _ in 0..20 {
        step(&mut w, 0);
        highest = highest.max(w.players[0].pos.y);
        theirs = theirs.max(w.players[1].pos.y);
    }
    assert!(
        highest.raw() > metres(0.8).raw(),
        "she went up: {}",
        as_f(highest)
    );
    assert!(
        w.players[0].pos.x.raw() < x0.raw(),
        "she went back: {} from {}",
        as_f(w.players[0].pos.x),
        as_f(x0)
    );
    assert!(hurt(&w) > 0, "the blade caught them");
    assert!(
        theirs.raw() > metres(0.3).raw(),
        "and launched them: {}",
        as_f(theirs)
    );
}

#[test]
fn a_kite_cut_on_a_marked_body_gives_the_airdodge_back() {
    for marks in [0u8, 2] {
        let mut w = facing(1.2);
        aloft(&mut w, 1.5);
        w.players[1].pos.y = metres(1.0);
        w.players[1].grounded = false;
        w.players[1].marks = marks;
        w.players[0].air_dodged = true;
        step(&mut w, keys::BLADE);
        assert_eq!(doing(&w), Some(r::KITE_CUT));
        let mut back = false;
        for _ in 0..12 {
            w.players[1].pos.y = metres(1.0);
            w.players[1].vel = V3::ZERO;
            step(&mut w, 0);
            back |= !w.players[0].air_dodged;
        }
        assert!(hurt(&w) > 0, "the kite cut landed ({marks} marks)");
        assert_eq!(
            back,
            marks > 0,
            "the airdodge came back with {marks} marks on them"
        );
    }
}

// ---------------------------------------------------------------------------
// The execution
// ---------------------------------------------------------------------------

#[test]
fn the_guillotine_drop_goes_straight_down_and_spikes() {
    let mut w = facing(0.6);
    aloft(&mut w, 4.0);
    w.players[1].pos.y = metres(2.0);
    w.players[1].grounded = false;
    step(&mut w, keys::EXECUTION);
    assert_eq!(doing(&w), Some(r::GUILLOTINE_DROP));
    let mut spiked = false;
    for _ in 0..16 {
        step(&mut w, 0);
        spiked |= hurt(&w) > 0 && w.players[1].vel.y.raw() < 0;
    }
    assert!(
        w.players[0].pos.y.raw() < metres(3.0).raw(),
        "she dropped: {}",
        as_f(w.players[0].pos.y)
    );
    assert!(spiked, "the body under her was driven down");
}

#[test]
fn gallows_blinks_up_hangs_and_lands_blade_first() {
    let mut w = facing(1.6);
    step(&mut w, SPACE | keys::EXECUTION);
    assert_eq!(doing(&w), Some(r::GALLOWS));
    assert!(
        w.players[0].pos.y.raw() >= t::gallows_blink().raw(),
        "she is gone upward: {}",
        as_f(w.players[0].pos.y)
    );
    let mut landed = false;
    for _ in 0..60 {
        step(&mut w, 0);
        landed |= w.players[0].grounded;
        if landed {
            break;
        }
    }
    assert!(landed, "she came down");
    run(&mut w, 12, 0, 0);
    assert!(hurt(&w) > 0, "the blade landed where she did");
}

// ---------------------------------------------------------------------------
// The shadow
// ---------------------------------------------------------------------------

#[test]
fn a_hung_shadow_waits_in_the_air_then_sinks_to_the_floor() {
    let mut w = alone();
    // Space and right click, looking up a little: the point in the air.
    run(&mut w, 1, SPACE | keys::SHADOW, down(-15));
    assert_eq!(doing(&w), Some(r::HANG));
    run(&mut w, t::shadow_send_frames() as u32 + 4, 0, 0);
    let s = shadow(&w);
    assert!(matches!(s.doing, Ghost::Waiting), "it arrived");
    assert!(
        s.pos.y.raw() > metres(1.5).raw(),
        "it hangs in the air: {}",
        as_f(s.pos.y)
    );
    let high = s.pos.y;
    run(&mut w, t::hang_waits() as u32 - 10, 0, 0);
    assert_eq!(shadow(&w).pos.y, high, "it waits");
    run(&mut w, 240, 0, 0);
    assert_eq!(shadow(&w).pos.y, Fx::ZERO, "then sinks to the floor");
}

#[test]
fn a_shadow_put_on_the_floor_never_sinks() {
    let mut w = alone();
    send_ahead(&mut w);
    let y = shadow(&w).pos.y;
    run(&mut w, 200, 0, 0);
    assert_eq!(shadow(&w).pos.y, y);
}

#[test]
fn the_swap_trades_her_and_the_shadow() {
    let mut w = alone();
    send_ahead(&mut w);
    let there = shadow(&w).pos;
    aloft(&mut w, 3.0);
    let here = w.players[0].pos;
    run(&mut w, 2, keys::SHADOW, 0);
    assert_eq!(doing(&w), Some(r::SWAP));
    run(&mut w, 3, 0, 0);
    assert_eq!(w.players[0].pos.x, there.x, "she is where it was");
    let s = shadow(&w);
    assert_eq!(s.pos.x, here.x, "it is where she was");
    assert!(s.pos.y.raw() > metres(2.0).raw(), "up in the air");
    assert!(matches!(s.doing, Ghost::Waiting));
}

// ---------------------------------------------------------------------------
// Deadly mistake
// ---------------------------------------------------------------------------

#[test]
fn struck_in_the_stance_she_is_behind_the_attacker_and_takes_nothing() {
    let mut w = facing(1.6);
    let stood = w.players[0].pos;
    // He swings; she answers with the stance as it comes out.
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    let health = w.players[0].health;
    w.advance([
        Input::looking_at(keys::MISTAKE, 0, 0),
        Input::looking_at(Input::LEFT, 1 << 15, 0),
    ]);
    let mut caught = false;
    for _ in 0..20 {
        w.advance([Input::looking_at(0, 0, 0), Input::looking_at(0, 1 << 15, 0)]);
        if w.players[0].pos.x.raw() > w.players[1].pos.x.raw() {
            caught = true;
            break;
        }
    }
    assert!(caught, "she ended up behind him");
    assert_eq!(w.players[0].health, health, "she took nothing");
    assert!(
        matches!(w.players[0].action, Action::Free),
        "and is free to act"
    );
    let s = shadow(&w);
    assert!(matches!(s.doing, Ghost::Waiting), "her shadow stays");
    assert!(
        s.pos.sub(stood).flat_len().raw() < metres(0.5).raw(),
        "where she stood"
    );
}

#[test]
fn a_whiffed_stance_is_a_long_recovery() {
    let mut w = alone();
    step(&mut w, keys::MISTAKE);
    let m = sim::moves::get(Class::ShadowReaver, r::DEADLY_MISTAKE);
    assert!(
        m.recovery >= 24,
        "a stance with a short recovery is a free parry: {}",
        m.recovery
    );
    run(&mut w, (m.startup + m.active) as u32 + 3, 0, 0);
    assert!(matches!(w.players[0].action, Action::Recovery { .. }));
}
