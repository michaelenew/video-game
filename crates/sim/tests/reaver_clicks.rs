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
fn on_the_floor_the_clicks_are_the_reave_the_shuriken_and_the_send() {
    for (button, wanted) in [
        (keys::FINISHER, r::REAVE),
        (keys::NEUTRAL, r::SHURIKEN),
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
fn in_the_air_the_clicks_are_the_plunge_the_fan_and_the_whirl() {
    for (button, wanted) in [
        (keys::FINISHER, r::PLUNGE),
        (keys::NEUTRAL, r::FAN),
        (keys::SHADOW, r::WHIRL),
    ] {
        let mut w = alone();
        aloft(&mut w, 3.0);
        step(&mut w, button);
        assert_eq!(doing(&w), Some(wanted), "click {button:#x} in the air");
    }
}

#[test]
fn space_and_a_click_in_the_air_is_gallows_or_the_tumble() {
    for (button, wanted) in [(keys::FINISHER, r::GALLOWS), (keys::NEUTRAL, r::TUMBLE)] {
        let mut w = alone();
        aloft(&mut w, 3.0);
        step(&mut w, SPACE | button);
        assert_eq!(doing(&w), Some(wanted), "space and {button:#x} in the air");
    }
}
#[test]
fn space_and_a_click_is_a_takeoff() {
    for (button, wanted) in [
        (keys::FINISHER, r::GALLOWS),
        (keys::NEUTRAL, r::TUMBLE),
        (keys::SHADOW, r::RISING_WHIRL),
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
fn the_plunge_goes_where_she_looks_inside_its_cone() {
    // Looking 30 degrees down from 3 m up: inside the 45-degree cone round
    // straight down? No -- 30 below the horizon is 60 off straight down, so it
    // goes down the cone's edge on her bearing: as far forward as down.
    let mut w = alone();
    aloft(&mut w, 3.0);
    let from = w.players[0].pos;
    run(&mut w, 1, keys::FINISHER, down(30));
    let mut ring = false;
    for _ in 0..60 {
        run(&mut w, 1, 0, down(30));
        ring |= effects_of(&w, EffectKind::ShadowRing) > 0;
    }
    let to = w.players[0].pos;
    assert!(w.players[0].grounded, "she came down");
    let ahead = as_f(to.x.sub(from.x));
    let fell = as_f(from.y.sub(to.y));
    assert!(ahead > 2.0, "she went forward along her look: {ahead}");
    assert!(
        ahead <= fell * 1.15 + 0.3,
        "but no shallower than the cone allows: {ahead} forward for {fell} down"
    );
    assert!(ring, "the ring ran out from where she landed");
    // Looking 70 down is inside the cone: steeper than the edge.
    let mut w = alone();
    aloft(&mut w, 3.0);
    let from = w.players[0].pos;
    run(&mut w, 1, keys::FINISHER, down(70));
    run(&mut w, 60, 0, down(70));
    let ahead = as_f(w.players[0].pos.x.sub(from.x));
    assert!(
        ahead < 1.6,
        "aimed inside the cone, it goes that steeply: {ahead}"
    );
}

#[test]
fn the_plunge_lands_on_a_body_it_was_aimed_at() {
    let mut w = facing(3.0);
    aloft(&mut w, 3.0);
    run(&mut w, 1, keys::FINISHER, down(45));
    run(&mut w, 60, 0, down(45));
    assert!(hurt(&w) > 0, "the plunge landed on him");
}
#[test]
fn the_plunges_ring_reaches_past_the_blade() {
    let mut w = alone();
    aloft(&mut w, 1.5);
    // Too far for the blade's own hit, inside the ring.
    let ring = as_f(t::shadow_ring_reach());
    w.players[1].pos = at(-6.0 + ring * 0.8 + 0.4, 0.0, 8.0);
    // Straight down, so the dive lands where she is.
    run(&mut w, 1, keys::FINISHER, down(89));
    run(&mut w, 60, 0, down(89));
    assert_eq!(
        hurt(&w),
        t::shadow_ring_damage(),
        "the ring's small hit, and only that"
    );
}

/// Throw Gallows with the camera at `pitch` and follow it to the frame its
/// drive ends: where she went from, and to.
fn gallows_drive(w: &mut World, pitch: i16) -> (V3, V3) {
    let from = w.players[0].pos;
    run(w, 1, SPACE | keys::FINISHER, pitch);
    assert_eq!(doing(w), Some(r::GALLOWS));
    let mut to = w.players[0].pos;
    for _ in 0..40 {
        run(w, 1, 0, pitch);
        if shadow(w).rising == 0 {
            break;
        }
        to = w.players[0].pos;
    }
    (from, to)
}

fn climb_of(from: V3, to: V3) -> (f32, f32) {
    let d = to.sub(from);
    (as_f(V3::new(d.x, Fx::ZERO, d.z).flat_len()), as_f(d.y))
}

#[test]
fn gallows_drives_her_up_at_an_angle_with_the_crosshair_level_or_low() {
    // 2026-10-18: "a mid length drive upwards and at an angle (if cursor is
    // down or around neutral)". Level and well down both climb at the
    // floor's angle, forward along the crosshair's bearing.
    for pitch in [0, down(40)] {
        let mut w = alone();
        let (from, to) = gallows_drive(&mut w, pitch);
        let (along, up) = climb_of(from, to);
        let gone = (along * along + up * up).sqrt();
        let want = as_f(t::gallows_drive());
        assert!(
            (gone - want).abs() < 1.0,
            "pitch {pitch}: drove {gone:.2} m, not {want:.1}"
        );
        assert!(
            (up - along).abs() < 0.5,
            "pitch {pitch}: up {up:.2} and along {along:.2} is not the floor's 45 degrees"
        );
        assert!(
            to.x.raw() > from.x.raw(),
            "pitch {pitch}: forward, the way she faces"
        );
    }
}

#[test]
fn gallows_drives_exactly_where_the_crosshair_points_when_it_points_up() {
    // "... or to where the cursor is pointing precisely (if cursor is
    // pointing up)". Steeper than the floor, it is steeper than the floor.
    let mut w = alone();
    let (from, to) = gallows_drive(&mut w, down(-70));
    let (along, up) = climb_of(from, to);
    assert!(
        up > along * 2.0,
        "looking well up, it went up {up:.2} and along {along:.2}"
    );
    // And where it ends is where the crosshair was: the reticle's point at
    // the drive's length, from her chest.
    let mut w = alone();
    let look = Input::looking_at(0, 0, down(-70));
    let reach = t::gallows_drive();
    let aimed = {
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
        sim::aim::sight(0, look, reach, &scene).at
    };
    let chest = sim::aim::origin(w.players[0].pos);
    let (_, to) = gallows_drive(&mut w, down(-70));
    let ended = sim::aim::origin(to);
    assert!(
        ended.sub(aimed).len().raw() < metres(1.0).raw(),
        "ended {:.2} m off the crosshair's point (from {:.2} m away)",
        as_f(ended.sub(aimed).len()),
        as_f(aimed.sub(chest).len())
    );
}

#[test]
fn gallows_cuts_on_the_way_up() {
    // A Bulwark up and ahead of her, where the drive goes: the finisher
    // lands on him on the way, rather than waiting for a floor.
    let mut w = facing(4.0);
    w.players[1].pos.y = metres(3.0);
    w.players[1].grounded = false;
    let mut struck = false;
    run(&mut w, 1, SPACE | keys::FINISHER, 0);
    for _ in 0..20 {
        w.players[1].pos.y = metres(3.0);
        w.players[1].vel = V3::ZERO;
        run(&mut w, 1, 0, 0);
        struck |= hurt(&w) > 0;
    }
    assert!(struck, "the cut at the top of the drive landed");
}

#[test]
fn gallows_works_in_the_air() {
    let mut w = alone();
    aloft(&mut w, 2.0);
    let (from, to) = gallows_drive(&mut w, 0);
    assert!(
        to.y.sub(from.y).raw() > metres(3.0).raw(),
        "she drove up from the air: {}",
        as_f(to.y.sub(from.y))
    );
}

// ---------------------------------------------------------------------------
// The neutral blades
// ---------------------------------------------------------------------------

#[test]
fn the_shuriken_flies_to_the_crosshair_and_the_shadow_throws_one_too() {
    let mut w = facing(6.0);
    run(&mut w, 1, keys::NEUTRAL, down(8));
    let mut most = 0;
    for _ in 0..40 {
        run(&mut w, 1, 0, down(8));
        most = most.max(effects_of(&w, EffectKind::ShadowFan));
    }
    assert_eq!(most, 2, "hers and the shadow's");
    assert!(hurt(&w) > 0, "a shuriken reached him at 6 m");
    assert_eq!(w.players[1].marks, 2, "two throws, two marks");
}
#[test]
fn the_fan_is_level_and_holds_her_in_the_air() {
    let mut w = facing(5.0);
    aloft(&mut w, 0.6);
    w.players[0].vel.y = metres(-3.0);
    run(&mut w, 1, keys::NEUTRAL, 0);
    assert_eq!(doing(&w), Some(r::FAN));
    let mut fan = None;
    let start = w.players[0].pos.y;
    for _ in 0..10 {
        run(&mut w, 1, 0, 0);
        if fan.is_none() {
            fan = w
                .effects
                .iter()
                .flatten()
                .find(|e| e.kind == EffectKind::ShadowFan)
                .copied();
        }
    }
    let fan = fan.expect("the fan was thrown");
    assert_eq!(r::blades(r::FAN), 5);
    for b in 0..r::blades(r::FAN) {
        let d = fan.fan_dir(b);
        assert_eq!(
            d.y, fan.dir.y,
            "blade {b} is level with the line, not turned down"
        );
    }
    assert!(
        start.sub(w.players[0].pos.y).raw() < metres(0.5).raw(),
        "it held her: fell {}",
        as_f(start.sub(w.players[0].pos.y))
    );
}
#[test]
fn the_fan_hangs_her_less_each_time_she_keeps_throwing_it() {
    // 2026-10-18, from play: "the shuriken air stall needs to diminish the
    // more often it's used consecutively". Each Fan's hang is worth less than
    // the last, and a landing between them does not give it back -- only
    // going a while without one does.
    let mut w = alone();
    let mut hangs = Vec::new();
    for _ in 0..3 {
        aloft(&mut w, 3.0);
        w.players[0].vel.y = Fx::ZERO;
        step(&mut w, keys::NEUTRAL);
        assert_eq!(doing(&w), Some(r::FAN));
        hangs.push(w.players[0].air_stall);
        // Down, and stood on the floor, and up again: a fresh airtime.
        w.players[0].pos.y = Fx::ZERO;
        run(&mut w, 40, 0, 0);
        assert!(w.players[0].grounded, "she landed between throws");
    }
    assert!(
        hangs[0] > hangs[1] && hangs[1] > hangs[2],
        "the hang wore down: {hangs:?}"
    );
    run(&mut w, t::fan_forget() as u32, 0, 0);
    aloft(&mut w, 3.0);
    step(&mut w, keys::NEUTRAL);
    assert_eq!(
        w.players[0].air_stall, hangs[0],
        "after a rest the next Fan hangs in full"
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
        Some(r::FAN),
        "a second tumble in one trip is the Fan instead"
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
fn the_whirl_goes_as_far_as_the_send_cuts_more_than_once_and_comes_home() {
    let mut w = facing(6.0);
    aloft(&mut w, 1.0);
    run(&mut w, 1, keys::SHADOW, down(8));
    assert_eq!(doing(&w), Some(r::WHIRL));
    let mut cuts = 0;
    let mut last = 0;
    let mut home = false;
    for _ in 0..(t::shadow_return() as u32 + 120) {
        run(&mut w, 1, 0, down(8));
        if hurt(&w) > last {
            cuts += 1;
            last = hurt(&w);
        }
        home |= matches!(shadow(&w).doing, Ghost::Attending) && cuts > 0;
    }
    let whirl = sim::moves::get(Class::ShadowReaver, r::WHIRL).reach;
    let send = sim::moves::get(Class::ShadowReaver, r::SEND).reach;
    assert_eq!(whirl, send, "as far as the send");
    assert!(
        cuts >= 2,
        "it cut a body six metres out more than once: {cuts}"
    );
    assert!(home, "and the shadow came home on its own");
}

#[test]
fn a_second_whirl_moves_the_blades_rather_than_adding_a_set() {
    let mut w = alone();
    aloft(&mut w, 3.0);
    run(&mut w, 1, keys::SHADOW, down(20));
    run(&mut w, 30, 0, down(20));
    w.players[0].pos.y = metres(3.0);
    w.players[0].grounded = false;
    run(&mut w, 1, keys::SHADOW, down(40));
    run(&mut w, 6, 0, down(40));
    assert_eq!(
        effects_of(&w, EffectKind::ShadowWhirl),
        1,
        "one set of blades"
    );
    let e = w
        .effects
        .iter()
        .flatten()
        .find(|e| e.kind == EffectKind::ShadowWhirl)
        .unwrap();
    assert!(e.age < 10, "and its clock started over: age {}", e.age);
}
#[test]
fn the_rising_whirl_is_the_whirl_with_a_jump() {
    let mut w = alone();
    step(&mut w, SPACE | keys::SHADOW);
    assert_eq!(doing(&w), Some(r::RISING_WHIRL));
    let mut rose = Fx::ZERO;
    let mut spun = false;
    for _ in 0..20 {
        run(&mut w, 1, 0, 0);
        rose = rose.max(w.players[0].pos.y);
        spun |= effects_of(&w, EffectKind::ShadowWhirl) > 0;
    }
    assert!(rose.raw() > metres(1.0).raw(), "she jumped: {}", as_f(rose));
    assert!(spun, "and the shadow whirled");
}
#[test]
fn a_sent_shadow_comes_home_after_its_clock_and_another_send_starts_it_over() {
    let mut w = alone();
    send_ahead(&mut w, down(25));
    // Most of the clock gone, then sent again: it starts over. (Its clock
    // started when it arrived, part-way through `send_ahead`.)
    run(&mut w, t::shadow_return() as u32 - 60, 0, 0);
    assert!(shadow(&w).is_waiting(), "still out before its time");
    run(&mut w, 3, keys::SHADOW, down(35));
    run(&mut w, 40, 0, 0);
    assert!(
        shadow(&w).is_waiting(),
        "a second send starts the clock over"
    );
    run(&mut w, t::shadow_return() as u32 + 60, 0, 0);
    assert!(
        matches!(shadow(&w).doing, Ghost::Attending),
        "and when it runs out it comes home: {:?}",
        shadow(&w).doing
    );
}

#[test]
fn coming_home_on_its_clock_is_not_a_collection() {
    let mut w = alone();
    send_ahead(&mut w, down(25));
    let mut burst = false;
    for _ in 0..(t::shadow_return() as u32 + 80) {
        run(&mut w, 1, 0, 0);
        burst |= effects_of(&w, EffectKind::ShadowBurst) > 0;
    }
    assert!(matches!(shadow(&w).doing, Ghost::Attending));
    assert!(!burst, "a shadow that came home by itself bursts nothing");
}

#[test]
fn standing_on_the_shadow_collects_it_and_bursts() {
    let mut w = facing(2.0);
    // Sent at her own feet: it lands beside her and is hers again.
    run(&mut w, 2, keys::SHADOW, down(80));
    let mut burst = false;
    for _ in 0..30 {
        run(&mut w, 1, 0, 0);
        burst |= effects_of(&w, EffectKind::ShadowBurst) > 0;
    }
    assert!(matches!(shadow(&w).doing, Ghost::Attending), "collected");
    assert!(burst, "and collecting it is a burst");
    assert!(hurt(&w) > 0, "which cut him beside her");
    assert!(w.players[1].marks > 0, "and marked him");
}

#[test]
fn the_dash_is_only_offered_where_it_can_arrive() {
    let mut w = alone();
    let reach = t::shadow_dash_speed()
        .mul(sim::DT)
        .mul(Fx::from_int(t::dodge_frames() as i32));
    let mut s = shadow(&w);
    s.pos = w.players[0]
        .pos
        .add(V3::new(reach.add(metres(2.0)), Fx::ZERO, Fx::ZERO));
    s.doing = Ghost::Waiting;
    w.players[0].mechanic = Mechanic::Shadow(s);
    run(&mut w, 1, SHIFT | W, 0);
    assert_eq!(shadow(&w).dash, 0, "too far for one dodge's worth of dash");
}

// ---------------------------------------------------------------------------
// The tally
// ---------------------------------------------------------------------------

#[test]
fn only_the_finishers_cash_the_tally() {
    // A Shuriken on a marked body adds to the tally; it does not spend it.
    let mut w = facing(6.0);
    w.players[1].marks = 3;
    w.players[1].mark_clock = t::mark_fade();
    run(&mut w, 1, keys::NEUTRAL, down(8));
    run(&mut w, 30, 0, down(8));
    assert!(hurt(&w) > 0, "fixture: the throws landed");
    assert_eq!(w.players[1].marks, 5, "two more marks, nothing spent");
    // The Reave spends it, for a lot.
    let mut w = facing(2.5 + 1.6);
    w.players[1].marks = 5;
    w.players[1].mark_clock = t::mark_fade();
    step(&mut w, keys::FINISHER);
    let mut first = 0;
    for _ in 0..20 {
        step(&mut w, 0);
        if hurt(&w) > 0 {
            first = hurt(&w);
            break;
        }
    }
    let m = sim::moves::get(Class::ShadowReaver, r::REAVE);
    assert!(
        first > m.damage * 5,
        "a full tally on the sweet spot: {first} from a {} cut",
        m.damage
    );
    assert_eq!(w.players[1].marks, 0, "and spent");
}

#[test]
fn each_mark_is_worth_more_than_the_last() {
    let steps: Vec<i32> = (0..=t::mark_cap())
        .map(|m| sim::shadow::cash_multiple(m).raw())
        .collect();
    for pair in steps.windows(3) {
        assert!(
            pair[2] - pair[1] > pair[1] - pair[0],
            "the tally's steps do not grow: {steps:?}"
        );
    }
}

#[test]
fn the_marks_fade_on_one_clock() {
    let mut w = alone();
    w.players[1].pos = at(-3.0, 0.0, 8.0);
    w.players[1].marks = 5;
    w.players[1].mark_clock = t::mark_fade();
    run(&mut w, t::mark_fade() as u32 - 1, 0, 0);
    assert_eq!(
        w.players[1].marks, 5,
        "nothing fades before the clock runs out"
    );
    run(&mut w, 1, 0, 0);
    assert_eq!(w.players[1].marks, 4, "then one");
    run(&mut w, t::mark_fade() as u32, 0, 0);
    assert_eq!(w.players[1].marks, 3, "and one more each cycle");
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
    // Her jump and the flip's hop, cut to three quarters with the carry since
    // 2026-10-18 ("shadow dash jump speeds nerfed to 75% of current values
    // across the board") -- which puts it under an ordinary jump's rise.
    let jump = t::jump_speed().mul(Class::ShadowReaver.mobility().jump);
    let want = jump.add(t::dash_jump_hop()).mul(t::dash_jump_rise());
    assert!(
        // A frame of held gravity has already been taken off it.
        up.sub(want).abs().raw() < metres(1.0).raw(),
        "and hops at its share of a jump and the hop: {} against {}",
        as_f(up),
        as_f(want)
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
