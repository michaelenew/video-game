//! The Blood mage's clicks, stains, thralls and grey: what a player would
//! notice if any of it broke.
//!
//! `docs/design/exploration/0038_blood_mage_the_world_bleeds.md` is the
//! direction since 2026-11-04: the left click is the Red Hand (Rend tapped
//! and strung, Seize held), the middle the spike (out of a pool only), the
//! right the scythe (the Sweep tapped and strung, the Cast held); the air is
//! the floor; `E` plants a heart; grey holds two seconds, then goes fast. The
//! Cast, the zip, the plants and the reap are `tests/world_bleeds.rs`; her
//! ground, the swim and the strong version `tests/territory.rs`. Each test
//! below is one piece.

use sim::class::Class;
use sim::effects::{Effect, EffectKind};
use sim::moves::blood as b;
use sim::moves::blood::keys;
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

/// A Blood mage facing +X in the open, and a Bulwark far enough off to be out
/// of everything unless a test moves him.
fn mage() -> World {
    let mut w = World::with_classes([Class::BloodMage, Class::Bulwark]);
    w.players[0].pos = at(-6.0, 0.0, 8.0);
    w.players[1].pos = at(11.0, 0.0, 8.0);
    w
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

fn aloft(w: &mut World, who: usize, height: f32) {
    w.players[who].pos.y = metres(height);
    w.players[who].grounded = false;
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

fn pools(w: &World) -> Vec<Effect> {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.is_a_pool())
        .copied()
        .collect()
}

fn effects_of(w: &World, kind: EffectKind) -> Vec<Effect> {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.kind == kind)
        .copied()
        .collect()
}

fn place(w: &mut World, effect: Effect) {
    let slot = w.effects.iter().position(|s| s.is_none()).expect("room");
    w.effects[slot] = Some(effect);
}

/// A stain, hers to collect, at `pos`.
fn stain(w: &mut World, pos: V3, essence: i32) {
    place(w, Effect::pool(0, Class::BloodMage, b::CAST, pos, essence));
}

/// Hurt her, so there is grey to drink back.
fn wounded(w: &mut World) {
    w.players[0].wound(300);
}

/// Try a throw at a few pitches and keep the first world where `landed` says
/// it did what it was for: a player puts the crosshair on what they mean, and
/// these tests do not know the camera's exact geometry.
fn at_some_pitch(
    w: &World,
    pitches: &[i16],
    throw: impl Fn(&mut World, i16),
    landed: impl Fn(&World) -> bool,
    frames: u32,
) -> Option<World> {
    for &pitch in pitches {
        let mut trial = w.clone();
        throw(&mut trial, pitch);
        for _ in 0..frames {
            step(&mut trial, 0);
            if landed(&trial) {
                return Some(trial);
            }
        }
    }
    None
}

const PITCHES: [i16; 9] = [-6000, -4000, -3000, -2000, -1000, 0, 1000, 2000, 3000];

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// The map
// ---------------------------------------------------------------------------

#[test]
fn on_the_floor_a_tap_is_the_quick_thing_and_q_and_e_gather_and_plant() {
    for (button, wanted) in [
        (keys::HAND, b::REND),
        (keys::SCYTHE, b::SWEEP),
        (keys::GATHER, b::COAGULATE),
        (keys::PLANT, b::HEART),
    ] {
        let mut w = mage();
        // A heart is earned: her meter full, so `E` is a heart.
        w.players[0].heart_meter = t::heart_meter_full() as u16;
        step(&mut w, button);
        assert_eq!(doing(&w), Some(wanted), "button {button:#x} on the floor");
    }
}

#[test]
fn held_the_left_click_is_seize_and_the_right_the_cast() {
    for (button, tap, wanted) in [
        (keys::HAND, b::REND, b::SEIZE),
        (keys::SCYTHE, b::SWEEP, b::CAST),
    ] {
        let mut w = mage();
        hold(&mut w, button, field_opens(tap) + 3);
        assert_eq!(doing(&w), Some(wanted), "button {button:#x} held");
    }
}

#[test]
fn in_the_air_the_clicks_are_the_floors() {
    for (button, wanted) in [(keys::HAND, b::REND), (keys::SCYTHE, b::SWEEP)] {
        let mut w = mage();
        aloft(&mut w, 0, 3.0);
        step(&mut w, button);
        assert_eq!(doing(&w), Some(wanted), "button {button:#x} in the air");
    }
}
#[test]
fn space_and_a_click_is_a_takeoff_on_the_floor_and_once_in_the_air() {
    for (button, wanted) in [
        (keys::HAND, b::RISING_REND),
        (keys::SPIKES, b::RISING_SPIKE),
        (keys::SCYTHE, b::RISING_REAP),
    ] {
        // The Rising spike needs a pool at her feet.
        let mut w = mage();
        let feet = w.players[0].pos;
        stain(&mut w, feet, 60);
        step(&mut w, SPACE | button);
        assert_eq!(doing(&w), Some(wanted), "space and {button:#x}");

        if button == keys::SPIKES {
            continue;
        }
        let mut w = mage();
        aloft(&mut w, 0, 3.0);
        step(&mut w, SPACE | button);
        assert_eq!(doing(&w), Some(wanted), "space and {button:#x} in the air");
    }
}

/// Hold `button` for `frames`, looking a little down.
fn hold(w: &mut World, button: u16, frames: u32) {
    for _ in 0..frames {
        step_looking(w, button, -2000);
    }
}
// ---------------------------------------------------------------------------
// The spike
// ---------------------------------------------------------------------------

#[test]
fn a_chain_of_stains_erupts_link_by_link_not_all_at_once() {
    // Two stains two metres apart; the spike on the first.
    let mut w = mage();
    w.players[1].pos = at(40.0, 0.0, 8.0);
    let first = at(-1.0, 0.0, 8.0);
    let second = first.add(at(0.0, 0.0, 2.0));
    stain(&mut w, first, 100);
    stain(&mut w, second, 100);
    for e in w.effects.iter_mut().flatten() {
        e.age = t::stain_settles() + 1;
    }
    let erupted = |w: &World, p: V3| {
        effects_of(w, EffectKind::BlackSpike)
            .iter()
            .any(|e| e.erupted() && e.pos.sub(p).flat_len().raw() < metres(0.5).raw())
    };
    let mut found = None;
    for pitch in PITCHES {
        let mut trial = w.clone();
        step_looking(&mut trial, keys::SPIKES, pitch);
        let mut gone_first = None;
        let mut gone_second = None;
        for f in 0..80 {
            step(&mut trial, 0);
            if gone_first.is_none() && erupted(&trial, first) {
                gone_first = Some(f);
            }
            if gone_second.is_none() && erupted(&trial, second) {
                gone_second = Some(f);
            }
        }
        if let (Some(a), Some(b)) = (gone_first, gone_second)
            && b > a
        {
            found = Some((a, b));
            break;
        }
    }
    let (a, b) = found.expect("the spike on the first set the second off after it");
    assert!(
        b - a <= 2 * t::chain_delay() as u32 + 2,
        "and soon after: {a} then {b}"
    );
}

// ---------------------------------------------------------------------------
// The takeoffs and the air
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Every stain one kind
// ---------------------------------------------------------------------------

#[test]
fn walking_into_a_stain_drinks_it() {
    let mut w = mage();
    wounded(&mut w);
    let red = w.players[0].health;
    stain(&mut w, at(-4.0, 0.0, 8.0), 100);
    run(&mut w, 40, Input::W);
    assert!(w.players[0].health > red, "drunk by walking into it");
    assert!(pools(&w).is_empty(), "and it is gone");
}

#[test]
fn at_full_health_she_walks_through_a_stain_and_leaves_it() {
    // No Well to pour it into (2026-10-19): what her grey cannot hold stays
    // on the floor, her ground and a spike's.
    let mut w = mage();
    stain(&mut w, at(-4.0, 0.0, 8.0), 100);
    run(&mut w, 40, Input::W);
    assert_eq!(pools(&w).len(), 1, "the stain is still there");
    // Only what a stain drains of itself in the time is gone.
    assert!(
        pools(&w)[0].banked >= 100 - 2 * t::pool_drain(),
        "nothing drunk: {}",
        pools(&w)[0].banked
    );
}

#[test]
fn her_own_spilled_blood_heals_her_like_any() {
    let mut w = mage();
    wounded(&mut w);
    let red = w.players[0].health;
    place(
        &mut w,
        Effect::pool(0, Class::BloodMage, b::RISING_REND, at(-4.0, 0.0, 8.0), 100),
    );
    run(&mut w, 40, Input::W);
    assert!(w.players[0].health > red, "no stain is special");
}

#[test]
fn every_hit_on_a_creature_leaves_a_stain_worth_its_blow() {
    let mut w = World::hunt([Class::BloodMage; 2]);
    let beast = w.monsters[0].expect("a creature").pos;
    let bolt = Effect::cast(
        EffectKind::Haemorrhage,
        0,
        Class::BloodMage,
        b::REND,
        beast.add(at(0.0, 1.5, 0.0)),
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        Fx::ZERO,
    );
    let blow = bolt.damage();
    place(&mut w, bolt);
    let mut biggest = 0;
    for _ in 0..10 {
        w.advance([Input::default(); 2]);
        for p in pools(&w) {
            biggest = biggest.max(p.pool_volume());
        }
    }
    assert!(
        blow > 0 && biggest >= blow - 2 * t::pool_drain(),
        "a stain of {biggest} for a blow of {blow}"
    );
}

// ---------------------------------------------------------------------------
// The scythe and Coagulate
// ---------------------------------------------------------------------------

/// Frames from the press to the field opening: the tap's startup, or the
/// Champion's window (`tuning::hold_from`) if that is longer.
fn field_opens(tap: u8) -> u32 {
    (sim::moves::get(Class::BloodMage, tap).startup as u32).max(sim::tuning::hold_from() as u32)
}

#[test]
fn a_right_click_let_go_inside_the_champions_window_is_the_sweep() {
    // Pressed for most of the window -- longer than the Sweep's own wind-up
    // -- and still a tap.
    let mut w = mage();
    hold(&mut w, keys::SCYTHE, sim::tuning::hold_from() as u32 - 2);
    step(&mut w, 0);
    assert_eq!(doing(&w), Some(b::SWEEP), "a long tap is still the tap");
}

#[test]
fn the_sweep_cuts_him_and_spills_him() {
    let mut w = mage();
    w.players[1].pos = at(-4.2, 0.0, 8.0);
    let his = w.players[1].health;
    step(&mut w, keys::SCYTHE);
    for _ in 0..30 {
        w.players[1].pos = at(-4.2, 0.0, 8.0);
        step(&mut w, 0);
    }
    assert!(w.players[1].health < his, "the scythe cut him");
    assert!(!pools(&w).is_empty(), "and spilled him");
}

/// Sweep, pressing again each time a Sweep is recovering, and give back the
/// string places she reached.
fn strung_sweeps(presses: u32) -> Vec<u8> {
    let mut w = mage();
    w.players[1].pos = at(30.0, 0.0, 8.0);
    let mut places = Vec::new();
    let mut pressed = 0;
    let mut down = false;
    for _ in 0..120 {
        let recovering = matches!(
            w.players[0].action,
            Action::Recovery { kind, .. } if kind == b::SWEEP
        );
        let press = (pressed == 0 || recovering) && !down && pressed < presses;
        if press {
            pressed += 1;
        }
        step(&mut w, if press { keys::SCYTHE } else { 0 });
        down = press;
        if let Action::Active { kind, .. } = w.players[0].action
            && kind == b::SWEEP
            && places.last() != Some(&w.players[0].string)
        {
            places.push(w.players[0].string);
        }
    }
    places
}

#[test]
fn pressed_again_in_its_recovery_the_sweep_strings_three() {
    let places = strung_sweeps(5);
    assert_eq!(
        places,
        vec![0, 1, 2],
        "three blows, and then it starts over"
    );
}

#[test]
fn the_third_sweep_of_a_string_hits_harder() {
    let blow = |third: bool| {
        let mut w = mage();
        w.players[1].pos = at(-4.2, 0.0, 8.0);
        if third {
            w.players[0].string = 2 | sim::state::STRINGING;
        }
        let his = w.players[1].health;
        step(&mut w, keys::SCYTHE);
        for _ in 0..30 {
            w.players[1].pos = at(-4.2, 0.0, 8.0);
            step(&mut w, 0);
        }
        his - w.players[1].health
    };
    let (first, third) = (blow(false), blow(true));
    assert!(third > first, "{first} then {third}");
}

#[test]
fn coagulate_slides_her_stains_together_into_one() {
    let mut w = mage();
    let target = at(0.0, 0.0, 8.0);
    stain(&mut w, target, 60);
    stain(&mut w, target.add(at(4.0, 0.0, 0.0)), 60);
    stain(&mut w, target.add(at(0.0, 0.0, 4.0)), 60);
    let total: i32 = pools(&w).iter().map(|p| p.banked).sum();
    let gathered = at_some_pitch(
        &w,
        &PITCHES,
        |w, pitch| step_looking(w, keys::GATHER, pitch),
        |w| pools(w).len() == 1,
        60,
    )
    .expect("three stains became one");
    let one = pools(&gathered)[0].banked;
    assert!(one > total - 60, "{one} of {total} -- only what drained");
}

// ---------------------------------------------------------------------------
// Thralls
// ---------------------------------------------------------------------------
fn thralls(w: &World) -> Vec<Effect> {
    effects_of(w, EffectKind::Thrall)
}

/// A patch of her ground at `pos`, `radius` wide, for longer than any test.
fn claim(w: &mut World, pos: V3, radius: f32) {
    sim::ground::lay(&mut w.ground, 0, pos, metres(radius), 900);
}

#[test]
fn a_thrall_leaves_nobody_alone_off_her_ground() {
    let mut w = mage();
    w.players[1].pos = at(6.0, 0.0, 8.0);
    thrall(&mut w, at(4.0, 0.0, 8.0), 300);
    let his = w.players[1].health;
    for _ in 0..90 {
        w.players[1].pos = at(6.0, 0.0, 8.0);
        step(&mut w, 0);
    }
    assert_eq!(
        w.players[1].health, his,
        "nowhere near her ground, he is not its prey"
    );
}

/// A thrall of hers, of `essence`, at `pos`.
fn thrall(w: &mut World, pos: V3, essence: i32) {
    let mut e = Effect::pool(0, Class::BloodMage, b::HEART, pos, essence);
    e.kind = EffectKind::Thrall;
    e.life = u16::MAX;
    place(w, e);
}

#[test]
fn a_bite_deals_what_it_spends_and_spills_it() {
    let mut w = mage();
    w.players[1].pos = at(4.0, 0.0, 8.0);
    // Her ground near him, so it goes for him, and not under it, so the
    // bite is the plain one.
    claim(&mut w, at(6.0, 0.0, 8.0), 1.0);
    thrall(&mut w, at(3.5, 0.0, 8.0), 200);
    let his = w.players[1].health;
    let mut bit = None;
    for _ in 0..(t::thrall_bite_every() as u32 + 4) {
        w.players[1].pos = at(4.0, 0.0, 8.0);
        step(&mut w, 0);
        if w.players[1].health < his {
            bit = Some(his - w.players[1].health);
            break;
        }
    }
    let took = bit.expect("it bit him");
    let left = thralls(&w)[0].banked;
    assert!(
        200 - left >= took && 200 - left <= took + 2 * t::pool_drain(),
        "it spent what it dealt: {took}, and has {left} left"
    );
    let spilled: i32 = pools(&w).iter().map(|p| p.banked).sum();
    assert!(spilled >= took - 1, "and spilled it under him: {spilled}");
}

#[test]
fn one_blow_kills_a_thrall_and_it_falls_back_into_a_stain() {
    let mut w = mage();
    w.players[1].pos = at(11.0, 0.0, 8.0);
    thrall(&mut w, at(9.8, 0.0, 8.0), 400);
    let mut fell = false;
    for f in 0..40 {
        let his = if f == 0 { Input::LEFT } else { 0 };
        w.advance([
            Input::looking_at(0, LOOK_RIGHT, 0),
            Input::looking_at(his, LOOK_LEFT, 0),
        ]);
        w.players[1].pos.x = metres(11.0);
        if thralls(&w).is_empty() {
            fell = true;
            break;
        }
    }
    assert!(fell, "his swing killed it");
    let rest: i32 = pools(&w).iter().map(|p| p.banked).sum();
    // All it had, not what the blow left over (2026-10-19): only what it
    // drained in the moment it stood.
    assert!(
        rest > 400 - 2 * t::pool_drain() && rest <= 400,
        "a stain of all it had: {rest}"
    );
}

#[test]
fn a_thrall_goes_for_the_nearest_enemy_on_her_ground_by_itself() {
    let mut w = mage();
    w.players[1].pos = at(6.0, 0.0, 8.0);
    claim(&mut w, at(6.0, 0.0, 8.0), 2.0);
    thrall(&mut w, at(-4.0, 0.0, 8.0), 300);
    let his = w.players[1].health;
    for _ in 0..150 {
        w.players[1].pos = at(6.0, 0.0, 8.0);
        step(&mut w, 0);
    }
    assert!(w.players[1].health < his, "it went ten metres to bite him");
}

#[test]
fn a_thrall_walks_round_a_block_to_its_prey() {
    // The proving ground's block from x 5 to 9 m and z -4 to 4 m, a metre
    // and a half tall: the thrall on one side, him on the other, the
    // straight line through it.
    let mut w = mage();
    w.players[1].pos = at(11.0, 0.0, 0.0);
    claim(&mut w, at(11.0, 0.0, 0.0), 2.0);
    thrall(&mut w, at(3.0, 0.0, 0.0), 400);
    let his = w.players[1].health;
    let mut inside = false;
    for _ in 0..300 {
        w.players[1].pos = at(11.0, 0.0, 0.0);
        step(&mut w, 0);
        if let Some(th) = thralls(&w).first() {
            let (x, z) = (as_f(th.pos.x), as_f(th.pos.z));
            inside |= (5.2..8.8).contains(&x) && (-3.8..3.8).contains(&z);
        }
        if w.players[1].health < his {
            break;
        }
    }
    assert!(!inside, "it went through the block");
    assert!(
        w.players[1].health < his,
        "it found its way round to bite him"
    );
}

#[test]
fn a_thrall_bites_critters() {
    let mut w = World::hunt_of([Class::BloodMage; 2], sim::species::SpeciesId::GNAWERS);
    let Some(c) = (0..sim::critter::MAX_CRITTERS).find(|&c| w.critters[c].alive()) else {
        panic!("a pack");
    };
    let body = w.critters[c].pos;
    w.players[0].pos = body.add(at(20.0, 0.0, 0.0));
    w.players[1].health = 0;
    let before = w.critters[c].health;
    let mut th = Effect::pool(
        0,
        Class::BloodMage,
        b::HEART,
        body.add(at(0.5, 0.0, 0.0)),
        300,
    );
    th.kind = EffectKind::Thrall;
    place(&mut w, th);
    sim::ground::lay(&mut w.ground, 0, body, metres(4.0), 900);
    let mut bit = false;
    for _ in 0..90 {
        w.advance([Input::default(); 2]);
        if !w.critters[c].alive() || w.critters[c].health < before {
            bit = true;
            break;
        }
    }
    assert!(bit, "it bit a critter");
}
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Grey, and the finishers
// ---------------------------------------------------------------------------

#[test]
fn grey_holds_two_seconds_then_goes_fast() {
    let mut w = mage();
    w.players[1].pos = at(40.0, 0.0, 8.0);
    w.players[0].wound(300);
    let grey = w.players[0].grey;
    run(&mut w, t::grey_grace() as u32 - 2, 0);
    assert_eq!(w.players[0].grey, grey, "it held");
    run(&mut w, 60, 0);
    assert!(
        grey - w.players[0].grey >= t::grey_fade() - 2,
        "then a second took {}",
        grey - w.players[0].grey
    );
}

#[test]
fn a_dodge_with_no_stain_under_the_crosshair_is_only_a_dodge() {
    let mut w = mage();
    let from = w.players[0].pos;
    step_looking(&mut w, Input::SHIFT | Input::W, -2500);
    run(&mut w, 22, 0);
    assert!(pools(&w).is_empty(), "no blood thrown");
    let went = w.players[0].pos.sub(from).flat_len();
    assert!(went.raw() < metres(6.0).raw(), "no blink: {}", as_f(went));
}
#[test]
fn the_creature_turns_on_a_heart_in_its_way() {
    let mut w = World::hunt([Class::BloodMage; 2]);
    let beast = w.monsters[0].expect("a creature");
    w.players[0].pos = beast.pos.add(at(30.0, 0.0, 0.0));
    w.players[1].health = 0;
    let mut g = Effect::cast(
        EffectKind::Heart,
        0,
        Class::BloodMage,
        b::HEART,
        beast.pos.add(at(4.0, 0.0, 0.0)),
        V3::ZERO,
        t::heart_ground(),
    );
    g.banked = 2000;
    place(&mut w, g);
    run(&mut w, 120, 0);
    let heart = effects_of(&w, EffectKind::Heart);
    let seen = w.monsters[0].expect("still there").brain.seen;
    let at_heart = heart
        .first()
        .is_none_or(|g| g.pos.sub(seen).flat_len().raw() < metres(3.0).raw());
    assert!(at_heart, "what it watched was the heart, not her");
}
