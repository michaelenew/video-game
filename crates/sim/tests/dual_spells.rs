//! The Dual mage, every move a spell (2026-10-09): what a player would
//! notice if any of it broke.
//!
//! `docs/design/exploration/0010_dual_mage_spells.md` is the design. Left is
//! dark, the hex -- things that travel and linger; right is light, the strike
//! -- things that happen now; middle is twilight, both at once. A dark spell
//! hexes Umbra, a light one Radiance, and the other force on a hexed body sets
//! it off: a Shatter or a Wither.

use sim::class::{Class, Force, Mechanic};
use sim::dual::{NO_HEX, RADIANCE, UMBRA};
use sim::effects::EffectKind;
use sim::moves::dual as d;
use sim::moves::dual::keys;
use sim::state::Action;
use sim::tuning as t;
use sim::{Fx, Input, V3, World};

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

/// A Dual mage facing +X in the open, and a Bulwark `gap` metres in front of
/// her, idle.
fn facing(gap: f32) -> World {
    let mut w = World::with_classes([Class::DualMage, Class::Bulwark]);
    w.players[0].pos = at(-6.0, 0.0, 8.0);
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    w.players[1].pos = at(-6.0 + gap, 0.0, 8.0);
    w
}

/// The mage alone, the other one far off.
fn alone() -> World {
    let mut w = facing(0.0);
    w.players[1].pos = at(11.0, 0.0, -10.0);
    w
}

/// The pitch that puts her crosshair on the middle of the other fighter.
fn onto_them(w: &World) -> i16 {
    let them = w.players[1].pos;
    let middle = V3::new(
        them.x,
        them.y.add(t::body_height().div(Fx::from_int(2))),
        them.z,
    );
    sim::aim::look_onto_closely(w.players[0].pos, LOOK_RIGHT, w.players[0].aloft, middle)
}

fn step_looking(w: &mut World, bits: u16, pitch: i16) {
    w.advance([
        Input::looking_at(bits, LOOK_RIGHT, pitch),
        Input::looking_at(0, LOOK_LEFT, 0),
    ]);
}

fn step(w: &mut World, bits: u16) {
    step_looking(w, bits, 0);
}

fn run(w: &mut World, frames: u32, bits: u16) {
    for _ in 0..frames {
        step(w, bits);
    }
}

/// Cast `bits` at them and let it play out, aimed the whole way.
fn cast_at_them(w: &mut World, bits: u16, frames: u32) {
    let pitch = onto_them(w);
    step_looking(w, bits, pitch);
    for _ in 0..frames {
        let pitch = onto_them(w);
        step_looking(w, 0, pitch);
    }
}

fn aloft(w: &mut World, height: f32) {
    w.players[0].pos.y = metres(height);
    w.players[0].grounded = false;
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

fn effects(w: &World, kind: EffectKind) -> usize {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.kind == kind)
        .count()
}

fn bars(w: &World) -> (i32, i32) {
    let (dark, light) = sim::dual::bars(&w.players[0]).expect("her bars");
    (dark.to_int(), light.to_int())
}

/// Both bars level at `level`, carrying dark, so a test starts from a known
/// place on the curve.
fn level(w: &mut World, level: i32) {
    w.players[0].mechanic = Mechanic::Meter {
        dark: Fx::from_int(level),
        light: Fx::from_int(level),
        colour: Force::Dark,
        ascending: 0,
        jumped: false,
        landed: 0,
        singe: Fx::ZERO,
    };
}

/// Let the autos' and a cast's repeat lockout run out between casts.
fn rest(w: &mut World) {
    run(w, 45, 0);
}

// ---------------------------------------------------------------------------
// The map
// ---------------------------------------------------------------------------

#[test]
fn on_the_floor_the_clicks_are_the_bolt_binary_and_the_ray() {
    for (button, wanted) in [
        (keys::DARK, d::SHADE_BOLT),
        (keys::TWILIGHT, d::BINARY),
        (keys::LIGHT, d::SUNRAY),
        (keys::DARK_MAJOR, d::ABYSS),
        (keys::LIGHT_MAJOR, d::JUDGEMENT),
    ] {
        let mut w = alone();
        step(&mut w, button);
        assert_eq!(doing(&w), Some(wanted), "button {button:#x} on the floor");
    }
}

#[test]
fn in_the_air_the_clicks_are_reel_phase_and_flare() {
    for (button, wanted) in [
        (keys::DARK, d::REEL),
        (keys::TWILIGHT, d::PHASE),
        (keys::LIGHT, d::FLARE),
        (keys::DARK_MAJOR, d::ABYSS),
        (keys::LIGHT_MAJOR, d::JUDGEMENT),
    ] {
        let mut w = alone();
        aloft(&mut w, 4.0);
        step(&mut w, button);
        assert_eq!(doing(&w), Some(wanted), "button {button:#x} in the air");
    }
}

#[test]
fn space_and_a_click_is_a_takeoff() {
    for (button, wanted) in [
        (keys::DARK, d::NIGHTFALL),
        (keys::TWILIGHT, d::EQUINOX),
        (keys::LIGHT, d::DAWN),
    ] {
        let mut w = alone();
        step(&mut w, SPACE | button);
        assert_eq!(doing(&w), Some(wanted), "space and {button:#x}");
    }
}

// ---------------------------------------------------------------------------
// Dark you dodge, light you block
// ---------------------------------------------------------------------------

#[test]
fn a_shade_bolt_flies_and_hexes_umbra_and_never_pulls() {
    let mut w = facing(6.0);
    let before = w.players[1].health;
    let was = w.players[1].pos;
    let m = sim::moves::get(Class::DualMage, d::SHADE_BOLT);
    let pitch = onto_them(&w);
    step_looking(&mut w, keys::DARK, pitch);
    run(&mut w, m.startup as u32 + 1, 0);
    assert!(effects(&w, EffectKind::ShadeBolt) == 1, "a bolt in flight");
    assert_eq!(w.players[1].health, before, "in flight, not arrived");
    cast_at_them(&mut w, 0, 30);
    assert!(w.players[1].health < before, "it landed");
    assert_eq!(w.players[1].hex, UMBRA, "and hexed Umbra");
    let towards = was.x.sub(w.players[1].pos.x);
    assert!(
        towards.raw() <= 0,
        "a spell, not a pull: they came {} m toward her",
        as_f(towards)
    );
}

#[test]
fn a_sunray_is_already_there_and_hexes_radiance() {
    let mut w = facing(5.0);
    let before = w.players[1].health;
    let m = sim::moves::get(Class::DualMage, d::SUNRAY);
    cast_at_them(&mut w, keys::LIGHT, m.startup as u32 + 1);
    assert!(
        w.players[1].health < before,
        "the ray landed on its first frame"
    );
    assert_eq!(w.players[1].hex, RADIANCE);
}

// ---------------------------------------------------------------------------
// The reactions
// ---------------------------------------------------------------------------

/// What a Sunray alone takes off a clean body from `level`.
fn a_ray_alone(level_at: i32) -> i32 {
    let mut w = facing(5.0);
    level(&mut w, level_at);
    let before = w.players[1].health;
    cast_at_them(&mut w, keys::LIGHT, 20);
    before - w.players[1].health
}

#[test]
fn dark_then_light_shatters_the_hex() {
    let ray = a_ray_alone(50);
    let mut w = facing(5.0);
    level(&mut w, 50);
    cast_at_them(&mut w, keys::DARK, 30);
    assert_eq!(w.players[1].hex, UMBRA, "fixture: hexed");
    rest(&mut w);
    let before = w.players[1].health;
    let m = sim::moves::get(Class::DualMage, d::SUNRAY);
    cast_at_them(&mut w, keys::LIGHT, m.startup as u32 + 1);
    let dealt = before - w.players[1].health;
    assert!(
        dealt > ray + t::shatter_damage() / 4,
        "a Shatter is the ray and a burst: {dealt} against {ray} for the ray alone"
    );
    assert_eq!(w.players[1].hex, NO_HEX, "the hex is spent");
    assert!(
        matches!(w.players[1].action, Action::Stagger { .. }),
        "and it staggers: {:?}",
        w.players[1].action
    );
}

#[test]
fn light_then_dark_withers_heals_her_and_slows_them() {
    let mut w = facing(5.0);
    level(&mut w, 50);
    w.players[0].health -= 200;
    cast_at_them(&mut w, keys::LIGHT, 20);
    assert_eq!(w.players[1].hex, RADIANCE, "fixture: hexed");
    rest(&mut w);
    let hers = w.players[0].health;
    let pitch = onto_them(&w);
    step_looking(&mut w, keys::DARK, pitch);
    for _ in 0..30 {
        let pitch = onto_them(&w);
        step_looking(&mut w, 0, pitch);
        if w.players[1].hex == NO_HEX {
            break;
        }
    }
    assert_eq!(w.players[1].hex, NO_HEX, "the hex is spent");
    assert!(w.players[0].health > hers, "drained back to her");
    assert!(w.players[1].slowed > 0, "and they are slowed");
}

#[test]
fn the_same_force_twice_only_refreshes() {
    let mut w = facing(5.0);
    cast_at_them(&mut w, keys::LIGHT, 20);
    run(&mut w, 60, 0);
    let left = w.players[1].hex_left;
    cast_at_them(&mut w, keys::LIGHT, 10);
    assert_eq!(w.players[1].hex, RADIANCE);
    assert!(w.players[1].hex_left > left, "refreshed");
}

#[test]
fn a_hex_fades_on_its_own() {
    let mut w = facing(5.0);
    cast_at_them(&mut w, keys::LIGHT, 20);
    assert_eq!(w.players[1].hex, RADIANCE);
    run(&mut w, t::hex_lasts() as u32 + 2, 0);
    assert_eq!(w.players[1].hex, NO_HEX);
}

#[test]
fn binary_on_a_clean_body_sets_off_both_at_half() {
    let mut w = facing(7.0);
    level(&mut w, 50);
    w.players[0].health -= 200;
    let hers = w.players[0].health;
    let before = w.players[1].health;
    let pitch = onto_them(&w);
    step_looking(&mut w, keys::TWILIGHT, pitch);
    for _ in 0..50 {
        let pitch = onto_them(&w);
        step_looking(&mut w, 0, pitch);
        if w.players[1].health < before {
            break;
        }
    }
    assert!(before - w.players[1].health > 0, "it landed");
    assert_eq!(w.players[1].hex, NO_HEX, "and leaves no hex");
    assert!(w.players[0].health > hers, "the Wither's half healed her");
    assert!(w.players[1].slowed > 0, "and slowed them");
}

// ---------------------------------------------------------------------------
// The bars
// ---------------------------------------------------------------------------

#[test]
fn each_column_goads_its_own_bar_and_twilight_goads_both() {
    for (button, dark, light) in [
        (keys::DARK, true, false),
        (keys::LIGHT, false, true),
        (keys::TWILIGHT, true, true),
        (keys::DARK_MAJOR, true, false),
        (keys::LIGHT_MAJOR, false, true),
    ] {
        let mut w = alone();
        level(&mut w, 40);
        let (d0, l0) = bars(&w);
        step(&mut w, button);
        let (d1, l1) = bars(&w);
        assert_eq!(d1 > d0, dark, "button {button:#x}: dark {d0} -> {d1}");
        assert_eq!(l1 > l0, light, "button {button:#x}: light {l0} -> {l1}");
    }
}

// ---------------------------------------------------------------------------
// The majors
// ---------------------------------------------------------------------------

#[test]
fn the_abyss_drags_hexes_and_drains() {
    let mut w = facing(7.0);
    // Light ahead by as much as the band allows, so neither before the cast
    // nor after it does the burn take anything off what the well gives back.
    level(&mut w, 60);
    if let Mechanic::Meter { dark, .. } = &mut w.players[0].mechanic {
        *dark = Fx::from_int(60 - t::meter_band());
    }
    w.players[0].health -= 200;
    let hers = w.players[0].health;
    let them = w.players[1].pos;
    let at_them = V3::new(them.x.sub(metres(1.5)), Fx::ZERO, them.z);
    let pitch =
        sim::aim::look_onto_closely(w.players[0].pos, LOOK_RIGHT, w.players[0].aloft, at_them);
    step_looking(&mut w, keys::DARK_MAJOR, pitch);
    run(&mut w, 70, 0);
    assert_eq!(effects(&w, EffectKind::Abyss), 1, "the well is open");
    let moved = them.x.sub(w.players[1].pos.x);
    assert!(
        moved.raw() > metres(0.3).raw(),
        "dragged toward its middle: {} m",
        as_f(moved)
    );
    assert_eq!(w.players[1].hex, UMBRA, "hexed Umbra");
    assert!(w.players[0].health > hers, "and it drained back to her");
}

#[test]
fn judgement_on_the_well_shatters_what_the_abyss_hexed() {
    let mut w = facing(7.0);
    level(&mut w, 60);
    let them = w.players[1].pos;
    let pitch = sim::aim::look_onto_closely(w.players[0].pos, LOOK_RIGHT, w.players[0].aloft, them);
    step_looking(&mut w, keys::DARK_MAJOR, pitch);
    run(&mut w, 60, 0);
    assert_eq!(w.players[1].hex, UMBRA, "fixture: hexed by the well");
    let pitch = sim::aim::look_onto_closely(
        w.players[0].pos,
        LOOK_RIGHT,
        w.players[0].aloft,
        w.players[1].pos,
    );
    step_looking(&mut w, keys::LIGHT_MAJOR, pitch);
    let mut shattered = false;
    for _ in 0..60 {
        step_looking(&mut w, 0, pitch);
        if matches!(w.players[1].action, Action::Stagger { .. }) {
            shattered = true;
        }
    }
    assert!(shattered, "Judgement set the Umbra off");
}

// ---------------------------------------------------------------------------
// The air and the takeoffs
// ---------------------------------------------------------------------------

#[test]
fn a_flare_at_the_floor_kicks_her_up_and_at_the_sky_does_not() {
    let mut w = alone();
    aloft(&mut w, 2.0);
    let down = (-80 * 65536 / 360) as i16;
    step_looking(&mut w, keys::LIGHT, down);
    let m = sim::moves::get(Class::DualMage, d::FLARE);
    for _ in 0..m.startup + 1 {
        step_looking(&mut w, 0, down);
    }
    assert!(
        w.players[0].vel.y.raw() > metres(5.0).raw(),
        "kicked up off the floor: {} m/s",
        as_f(w.players[0].vel.y)
    );
    let mut w = alone();
    aloft(&mut w, 2.0);
    let up = (60 * 65536 / 360) as i16;
    step_looking(&mut w, keys::LIGHT, up);
    for _ in 0..m.startup + 1 {
        step_looking(&mut w, 0, up);
    }
    assert!(
        w.players[0].vel.y.raw() < metres(5.0).raw(),
        "nothing there, no kick: {} m/s",
        as_f(w.players[0].vel.y)
    );
}

#[test]
fn phase_puts_her_along_the_crosshair_once_a_trip() {
    let mut w = alone();
    aloft(&mut w, 3.0);
    let from = w.players[0].pos;
    step(&mut w, keys::TWILIGHT);
    let m = sim::moves::get(Class::DualMage, d::PHASE);
    run(&mut w, m.startup as u32 + 1, 0);
    let went = w.players[0].pos.x.sub(from.x);
    assert!(
        went.raw() > metres(3.0).raw(),
        "she is through: {} m",
        as_f(went)
    );
    assert!(w.players[0].air_dodged, "it spent the airdodge");
    run(&mut w, 30, 0);
    if !w.players[0].grounded {
        step(&mut w, keys::TWILIGHT);
        assert_ne!(doing(&w), Some(d::PHASE), "once a trip");
    }
}

fn equinox_top(at: i32) -> Fx {
    let mut w = alone();
    level(&mut w, at);
    step(&mut w, SPACE | keys::TWILIGHT);
    let mut top = Fx::ZERO;
    for _ in 0..90 {
        step(&mut w, 0);
        top = top.max(w.players[0].pos.y);
    }
    top
}

#[test]
fn equinox_goes_higher_the_more_level_she_is() {
    let empty = equinox_top(0);
    let full = equinox_top(90);
    assert!(
        full.raw() > empty.add(metres(1.0)).raw(),
        "level and full: {} m, empty: {} m",
        as_f(full),
        as_f(empty)
    );
}

#[test]
fn dawn_takes_somebody_beside_her_up_with_her() {
    let mut w = facing(1.2);
    step(&mut w, SPACE | keys::LIGHT);
    let mut highest = Fx::ZERO;
    for _ in 0..40 {
        step(&mut w, 0);
        highest = highest.max(w.players[1].pos.y);
    }
    assert!(
        highest.raw() > metres(1.0).raw(),
        "they went up: {} m",
        as_f(highest)
    );
}

#[test]
fn nightfall_leaves_a_well_where_she_left_the_floor() {
    let mut w = alone();
    let from = w.players[0].pos;
    step(&mut w, SPACE | keys::DARK);
    run(&mut w, 12, 0);
    let wells: Vec<_> = w
        .effects
        .iter()
        .flatten()
        .filter(|e| e.kind == EffectKind::Abyss)
        .copied()
        .collect();
    assert_eq!(wells.len(), 1, "a well");
    assert!(
        wells[0].pos.sub(from).flat_len().raw() < metres(1.0).raw(),
        "where she left"
    );
    assert!(
        w.players[0].pos.y.raw() > metres(1.0).raw(),
        "and she jumped"
    );
}

#[test]
fn reel_pulls_her_to_the_body_it_catches() {
    let mut w = facing(7.0);
    aloft(&mut w, 1.0);
    w.players[1].pos.y = metres(1.0);
    w.players[1].grounded = false;
    let gap = w.players[1].pos.x.sub(w.players[0].pos.x);
    let pitch = onto_them(&w);
    step_looking(&mut w, keys::DARK, pitch);
    let mut closest = gap;
    for _ in 0..40 {
        let pitch = onto_them(&w);
        step_looking(&mut w, 0, pitch);
        closest = closest.min(w.players[1].pos.x.sub(w.players[0].pos.x).abs());
    }
    assert!(
        closest.raw() < metres(2.5).raw(),
        "she was reeled in from {} m to {} m",
        as_f(gap),
        as_f(closest)
    );
}

// ---------------------------------------------------------------------------
// Creatures
// ---------------------------------------------------------------------------

#[test]
fn a_creature_is_hexed_and_reacts_like_a_fighter() {
    let mut w = World::hunt([Class::DualMage, Class::Bulwark]);
    let beast = w.monsters[0].expect("a creature");
    // Stand her a few metres off its middle, facing it.
    let to = beast.pos.sub(w.players[0].pos);
    let flat = V3::new(to.x, Fx::ZERO, to.z);
    let dir = flat.normalized();
    w.players[0].pos = beast.pos.sub(dir.scale(Fx::from_int(6)));
    w.players[0].pos.y = w.terrain().ground_under(w.players[0].pos);
    w.players[1].pos = beast.pos.sub(dir.scale(Fx::from_int(20)));
    let yaw = sim::math::atan2_turns(dir.z, dir.x);
    let aim = (yaw.raw() as u32 & 0xffff) as u16;
    let middle = V3::new(beast.pos.x, beast.pos.y.add(Fx::from_int(1)), beast.pos.z);
    let pitch = sim::aim::look_onto_closely(w.players[0].pos, aim, w.players[0].aloft, middle);
    let mut hexed = false;
    w.advance([Input::looking_at(keys::LIGHT, aim, pitch), Input::default()]);
    for _ in 0..20 {
        w.advance([Input::looking_at(0, aim, pitch), Input::default()]);
        if w.monsters[0].is_some_and(|b| b.hex == RADIANCE) {
            hexed = true;
        }
    }
    assert!(hexed, "the ray hexed the creature");
}
