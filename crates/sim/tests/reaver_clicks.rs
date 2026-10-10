//! The Shadow Reaver on three clicks, second pass (2026-10-10): what a player
//! would notice if any of it broke.
//!
//! `docs/design/exploration/0013_shadow_reaver_dual_blades.md` is the design.
//! Left is the finishers -- a dash strike, a plunge, a slam out of the sky;
//! middle is the neutral blades -- a ring, a spin, a tumbling double jump that
//! throws a fan; right is the shadow -- the send that moves it, the Whirl, the
//! arc. `E` is Executioner.

use sim::class::{Ghost, Mechanic, Shadow};
use sim::effects::EffectKind;
use sim::moves::reaver as r;
use sim::moves::reaver::keys;
use sim::state::{Action, SLOT_MECHANIC};
use sim::tuning as t;
use sim::{Class, Fx, Input, V3, World};

const SPACE: u16 = Input::SPACE;
const SHIFT: u16 = Input::SHIFT;
const W: u16 = Input::W;

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

fn effects_of(w: &World, kind: EffectKind) -> usize {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.kind == kind)
        .count()
}

/// Send the shadow ahead along the floor and let it settle.
fn send_ahead(w: &mut World, pitch: i16) {
    run(w, 2, keys::SHADOW, pitch);
    run(w, 40, 0, pitch);
    assert!(
        matches!(shadow(w).doing, Ghost::Waiting),
        "the shadow did not go out"
    );
}

// ---------------------------------------------------------------------------
// The map
// ---------------------------------------------------------------------------

#[test]
fn on_the_floor_the_clicks_are_the_reave_the_ring_and_the_send() {
    for (button, wanted) in [
        (keys::FINISHER, r::REAVE),
        (keys::NEUTRAL, r::BLADE_RING),
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
fn in_the_air_the_clicks_are_the_plunge_the_spin_and_the_whirl() {
    for (button, wanted) in [
        (keys::FINISHER, r::PLUNGE),
        (keys::NEUTRAL, r::BLADE_SPIN),
        (keys::SHADOW, r::WHIRL),
    ] {
        let mut w = alone();
        aloft(&mut w, 3.0);
        step(&mut w, button);
        assert_eq!(doing(&w), Some(wanted), "click {button:#x} in the air");
    }
}

#[test]
fn space_and_a_click_is_a_takeoff() {
    for (button, wanted) in [
        (keys::FINISHER, r::GALLOWS),
        (keys::NEUTRAL, r::TUMBLE),
        (keys::SHADOW, r::SHADOW_ARC),
    ] {
        let mut w = alone();
        step(&mut w, SPACE | button);
        assert_eq!(doing(&w), Some(wanted), "space and {button:#x}");
    }
}

#[test]
fn e_is_executioner_and_q_is_still_the_lotus() {
    let mut w = alone();
    step(&mut w, keys::EXECUTIONER);
    assert_eq!(doing(&w), Some(r::EXECUTIONER));
    let mut w = alone();
    step(&mut w, keys::LOTUS);
    assert_eq!(doing(&w), Some(r::LOTUS));
}

// ---------------------------------------------------------------------------
// The finishers
// ---------------------------------------------------------------------------

/// What her own Reave deals a body standing `gap` in front of her, before the
/// shadow's copy can add anything.
fn reave_on(gap: f32) -> i32 {
    let mut w = facing(gap);
    step(&mut w, keys::FINISHER);
    for _ in 0..20 {
        step(&mut w, 0);
        if hurt(&w) > 0 {
            return hurt(&w);
        }
    }
    0
}

#[test]
fn the_reave_dashes_in_and_its_tip_is_a_sweet_spot() {
    // Out at the end of the dash, on the middle of the blade's disc.
    let sweet = reave_on(2.5 + 1.6);
    // Close in: the dash runs into him, and the disc's middle is past him.
    let close = reave_on(1.2);
    let m = sim::moves::get(Class::ShadowReaver, r::REAVE);
    assert!(
        sweet > m.damage,
        "the sweet spot is worth more than the hit: {sweet} against {}",
        m.damage
    );
    assert!(
        close > 0 && close < sweet,
        "off the sweet spot is a plain hit: {close} against {sweet}"
    );
    // And the dash is a dash.
    let mut w = alone();
    let x0 = w.players[0].pos.x;
    step(&mut w, keys::FINISHER);
    run(&mut w, 20, 0, 0);
    assert!(
        w.players[0].pos.x.sub(x0).raw() > metres(1.5).raw(),
        "she dashed forward: {}",
        as_f(w.players[0].pos.x.sub(x0))
    );
}

#[test]
fn the_plunge_dives_a_little_forward_and_lands_a_ring() {
    let mut w = facing(1.0);
    aloft(&mut w, 2.5);
    // A body out past the blade, inside the ring's reach.
    w.players[1].pos = at(-6.0 + 1.2, 0.0, 8.0);
    let x0 = w.players[0].pos.x;
    step(&mut w, keys::FINISHER);
    let mut ring = false;
    for _ in 0..60 {
        step(&mut w, 0);
        ring |= effects_of(&w, EffectKind::ShadowRing) > 0;
    }
    assert!(w.players[0].grounded, "she came down");
    assert!(
        w.players[0].pos.x.raw() > x0.raw(),
        "a little forward: {} from {}",
        as_f(w.players[0].pos.x),
        as_f(x0)
    );
    assert!(ring, "the ring ran out from where she landed");
    assert!(hurt(&w) > 0, "and the plunge landed");
}

#[test]
fn the_plunges_ring_reaches_past_the_blade() {
    let mut w = alone();
    aloft(&mut w, 1.5);
    // Too far for the blade's own hit, inside the ring.
    let ring = as_f(t::shadow_ring_reach());
    w.players[1].pos = at(-6.0 + ring * 0.8 + 0.4, 0.0, 8.0);
    step(&mut w, keys::FINISHER);
    run(&mut w, 60, 0, 0);
    assert_eq!(
        hurt(&w),
        t::shadow_ring_damage(),
        "the ring's small hit, and only that"
    );
}

#[test]
fn gallows_blinks_up_slams_down_and_is_slow_to_recover() {
    let mut w = facing(1.4);
    step(&mut w, SPACE | keys::FINISHER);
    let mut highest = Fx::ZERO;
    for _ in 0..8 {
        step(&mut w, 0);
        highest = highest.max(w.players[0].pos.y);
    }
    assert!(
        highest.raw() > metres(1.5).raw(),
        "she blinked up: {}",
        as_f(highest)
    );
    let mut landed_recovering = 0;
    for _ in 0..90 {
        step(&mut w, 0);
        if w.players[0].grounded
            && matches!(w.players[0].action, Action::Recovery { kind, .. } if kind == r::GALLOWS)
        {
            landed_recovering += 1;
        }
    }
    assert!(hurt(&w) > 0, "the slam landed");
    assert!(
        landed_recovering >= 28,
        "a long recovery on the floor: {landed_recovering} frames"
    );
}

// ---------------------------------------------------------------------------
// The neutral blades
// ---------------------------------------------------------------------------

#[test]
fn the_blade_ring_cuts_behind_her_too() {
    let mut w = facing(-1.6);
    step(&mut w, keys::NEUTRAL);
    run(&mut w, 20, 0, 0);
    assert!(hurt(&w) > 0, "a body behind her was in the ring");
}

#[test]
fn the_blade_spin_is_a_thin_disc_with_a_long_reach() {
    // Level with her waist, far out: caught.
    let mut w = facing(2.6);
    aloft(&mut w, 0.0);
    w.players[0].pos.y = Fx::ZERO;
    w.players[0].grounded = false;
    step(&mut w, keys::NEUTRAL);
    run(&mut w, 8, 0, 0);
    assert!(hurt(&w) > 0, "the spin reached 2.6 m");
    // Well below her: not a ball, so not caught.
    let mut w = facing(1.0);
    aloft(&mut w, 3.5);
    step(&mut w, keys::NEUTRAL);
    assert_eq!(doing(&w), Some(r::BLADE_SPIN));
    run(&mut w, 6, 0, 0);
    assert_eq!(
        hurt(&w),
        0,
        "a body three metres under the disc is not in it"
    );
}

#[test]
fn the_tumble_is_a_double_jump_once_a_trip() {
    let mut w = alone();
    aloft(&mut w, 3.0);
    w.players[0].vel.y = metres(-4.0);
    step(&mut w, SPACE | keys::NEUTRAL);
    assert_eq!(doing(&w), Some(r::TUMBLE), "space and middle in the air");
    let mut rose = false;
    for _ in 0..10 {
        step(&mut w, 0);
        rose |= w.players[0].vel.y.raw() > 0;
    }
    assert!(rose, "the tumble lifted her");
    run(&mut w, 20, 0, 0);
    assert!(!w.players[0].grounded, "still in the air for the test");
    step(&mut w, SPACE | keys::NEUTRAL);
    assert_eq!(
        doing(&w),
        Some(r::BLADE_SPIN),
        "a second tumble in one trip is the spin instead"
    );
}

#[test]
fn the_tumble_throws_its_blades_forward_and_down() {
    let mut w = facing(4.0);
    aloft(&mut w, 2.5);
    run(&mut w, 1, SPACE | keys::NEUTRAL, down(25));
    let mut thrown = false;
    for _ in 0..30 {
        run(&mut w, 1, 0, down(25));
        thrown |= effects_of(&w, EffectKind::ShadowFan) > 0;
    }
    assert!(thrown, "the fan left her hand");
    assert!(hurt(&w) > 0, "and reached the body ahead of and below her");
}

// ---------------------------------------------------------------------------
// The shadow
// ---------------------------------------------------------------------------

#[test]
fn pressed_again_the_send_moves_the_shadow_rather_than_recalling_it() {
    let mut w = alone();
    send_ahead(&mut w, down(25));
    let first = shadow(&w).pos;
    // Turn and send again: it goes from where it was to the new spot.
    for _ in 0..3 {
        w.advance([
            Input::looking_at(keys::SHADOW, 1 << 14, down(25)),
            Input::default(),
        ]);
    }
    assert!(
        matches!(shadow(&w).doing, Ghost::Casting { .. }),
        "it is on its way somewhere, not coming home: {:?}",
        shadow(&w).doing
    );
    run(&mut w, 40, 0, 0);
    let second = shadow(&w).pos;
    assert!(shadow(&w).is_waiting(), "and it waits there");
    assert!(
        second.sub(first).flat_len().raw() > metres(1.0).raw(),
        "somewhere else"
    );
}

#[test]
fn the_whirl_goes_a_short_way_cuts_more_than_once_and_comes_home() {
    let mut w = facing(3.0);
    aloft(&mut w, 1.0);
    let her = w.players[0].pos;
    run(&mut w, 1, keys::SHADOW, down(15));
    assert_eq!(doing(&w), Some(r::WHIRL));
    let mut furthest = Fx::ZERO;
    let mut cuts = 0;
    let mut last = 0;
    let mut home = false;
    for _ in 0..(t::whirl_lasts() as u32 + 80) {
        run(&mut w, 1, 0, down(15));
        let s = shadow(&w);
        furthest = furthest.max(s.pos.sub(her).flat_len());
        if hurt(&w) > last {
            cuts += 1;
            last = hurt(&w);
        }
        home |= matches!(s.doing, Ghost::Attending) && cuts > 0;
    }
    let reach = sim::moves::get(Class::ShadowReaver, r::WHIRL).reach;
    assert!(
        furthest.raw() <= reach.add(metres(1.5)).raw(),
        "a short way: {}",
        as_f(furthest)
    );
    assert!(cuts >= 2, "it cut more than once: {cuts}");
    assert!(home, "and it came home on its own");
}

#[test]
fn the_shadow_arc_hangs_the_shadow_up_ahead_then_brings_it_home() {
    let mut w = alone();
    let her = w.players[0].pos;
    step(&mut w, SPACE | keys::SHADOW);
    assert_eq!(doing(&w), Some(r::SHADOW_ARC));
    run(&mut w, t::arc_flight() as u32 + 4, 0, 0);
    let s = shadow(&w);
    assert!(s.is_waiting(), "it hangs: {:?}", s.doing);
    assert!(
        s.pos.y.sub(her.y).raw() > metres(2.0).raw(),
        "up: {}",
        as_f(s.pos.y)
    );
    assert!(
        s.pos.x.sub(her.x).raw() > metres(3.0).raw(),
        "and ahead: {}",
        as_f(s.pos.x.sub(her.x))
    );
    run(&mut w, t::arc_waits() as u32 + 60, 0, 0);
    assert!(
        matches!(shadow(&w).doing, Ghost::Attending),
        "and then it comes home: {:?}",
        shadow(&w).doing
    );
}

#[test]
fn the_shadow_arc_cuts_what_it_flies_through() {
    let mut w = facing(2.5);
    step(&mut w, SPACE | keys::SHADOW);
    run(&mut w, t::arc_flight() as u32, 0, 0);
    assert!(hurt(&w) > 0, "the arc's slash");
}

#[test]
fn a_forward_dodge_follows_the_shadow_while_it_is_still_flying() {
    let mut w = alone();
    run(&mut w, 2, keys::SHADOW, down(12));
    // Into the throw's active frames: the shadow has left her.
    run(&mut w, 2, 0, down(12));
    assert!(matches!(shadow(&w).doing, Ghost::Casting { .. }));
    assert!(doing(&w).is_some(), "still in the send's frames");
    // Look away: following does not need the crosshair on it.
    w.advance([Input::looking_at(SHIFT | W, 1 << 13, 0), Input::default()]);
    assert!(
        matches!(w.players[0].action, Action::Dodge { .. }),
        "the dodge cut the throw: {:?}",
        w.players[0].action
    );
    assert!(shadow(&w).dash > 0, "and it is the dash, chasing it");
    run(&mut w, 30, 0, 0);
    assert!(
        matches!(shadow(&w).doing, Ghost::Attending),
        "she caught it up"
    );
}

#[test]
fn the_dash_jump_flips_and_hops() {
    // Send it, follow it with the dash, and press space as she closes on it.
    let mut w = alone();
    run(&mut w, 4, keys::SHADOW, down(12));
    let close = t::shadow_dash_speed().mul(sim::DT).mul(Fx::from_int(2));
    let mut pressing = false;
    let mut leaving = None;
    for _ in 0..60 {
        let near =
            shadow(&w).dash > 0 && shadow(&w).pos.sub(w.players[0].pos).len().raw() <= close.raw();
        pressing |= near;
        let bits = if pressing { W | SPACE } else { SHIFT | W };
        run(&mut w, 1, bits, 0);
        if leaving.is_none() && shadow(&w).flip > 0 {
            leaving = Some(w.players[0].vel.y);
        }
    }
    let up = leaving.expect("a dash jump flips her");
    let jump = t::jump_speed().mul(Class::ShadowReaver.mobility().jump);
    assert!(
        up.raw() > jump.raw(),
        "and hops higher than her jump: {} against {}",
        as_f(up),
        as_f(jump)
    );
}

#[test]
fn the_send_does_not_put_the_shadow_where_she_could_not_walk() {
    // Aim at the top of the dais from the floor in front of it: the shadow
    // goes to the floor at its foot, not up onto it.
    let dais = *sim::arena::proving_ground::ARENA
        .solids()
        .iter()
        .filter(|s| s.max.x.raw() < sim::arena::proving_ground::half().raw())
        .max_by_key(|s| s.min.x.raw())
        .expect("the arena has a platform");
    let mid_z = dais.min.z.add(dais.max.z).mul(Fx::ratio(1, 2));
    let mut w = World::with_classes([Class::ShadowReaver, Class::Bulwark]);
    w.players[0].pos = V3::new(dais.min.x.sub(metres(4.0)), Fx::ZERO, mid_z);
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    w.players[1].pos = at(-12.0, 0.0, -12.0);
    run(&mut w, 20, 0, 0);
    let stones = sim::stones::gather(&w.players);
    let players = w.players;
    let effects = w.effects;
    let scene = sim::aim::Scene {
        stones: &stones,
        players: &players,
        effects: &effects,
        quarry: &w.monsters,
        critters: &sim::critter::Critters::NONE,
        arena: &w.terrain(),
    };
    let on_top = V3::new(dais.min.x.add(metres(1.0)), dais.max.y, mid_z);
    let to = sim::aim::walk_toward(w.players[0].pos, on_top, &scene).expect("the floor in front");
    assert_eq!(to.y, Fx::ZERO, "on the floor, not on the dais");
    assert!(
        to.x.raw() < dais.min.x.raw(),
        "short of the dais: {} against its face at {}",
        as_f(to.x),
        as_f(dais.min.x)
    );
    // And on open floor it is where it was pointed.
    let open = V3::new(w.players[0].pos.x.add(metres(2.0)), Fx::ZERO, mid_z);
    assert_eq!(
        sim::aim::walk_toward(w.players[0].pos, open, &scene),
        Some(open)
    );
}
