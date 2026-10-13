//! The Blood mage with thralls and the Well: what a player would notice if
//! any of it broke.
//!
//! `docs/design/exploration/0016_blood_mage_thralls_and_the_well.md` is the
//! direction: a tap is the quick thing and a hold is the field -- the
//! Haemorrhage and the Carpet on the left click, Command and the Summon on
//! the right, the spike in tiers on the middle; takeoffs with space, on the
//! floor and once in the air; every stain one kind, collected by walking,
//! teleporting or hitting through it, and what she cannot hold going to the
//! Well; a full Well turning the end of a hold into its finisher. Each test
//! below is one piece.

use sim::class::Class;
use sim::effects::{Effect, EffectKind};
use sim::moves::blood as b;
use sim::moves::blood::keys;
use sim::state::{Action, NAILED};
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
    place(
        w,
        Effect::pool(0, Class::BloodMage, b::HAEMORRHAGE, pos, essence),
    );
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
// The map
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// The map
// ---------------------------------------------------------------------------

#[test]
fn on_the_floor_a_tap_is_the_quick_thing_and_q_and_e_gather_and_plague() {
    for (button, wanted) in [
        (keys::MY_BLOOD, b::HAEMORRHAGE),
        (keys::YOUR_BLOOD, b::BLACK_SPIKE),
        (keys::SCYTHE, b::COMMAND),
        (keys::GATHER, b::COAGULATE),
        (keys::PLAGUE, b::HEMOPLAGUE),
    ] {
        let mut w = mage();
        step(&mut w, button);
        assert_eq!(doing(&w), Some(wanted), "button {button:#x} on the floor");
    }
}

#[test]
fn in_the_air_the_middle_click_is_the_stake() {
    for (button, wanted) in [
        (keys::MY_BLOOD, b::HAEMORRHAGE),
        (keys::YOUR_BLOOD, b::STAKE),
        (keys::SCYTHE, b::COMMAND),
    ] {
        let mut w = mage();
        aloft(&mut w, 0, 3.0);
        step(&mut w, button);
        assert_eq!(doing(&w), Some(wanted), "button {button:#x} in the air");
    }
}

#[test]
fn space_and_a_click_is_a_takeoff_on_the_floor_and_once_in_the_air() {
    for (button, wanted) in [
        (keys::MY_BLOOD, b::WINGS),
        (keys::YOUR_BLOOD, b::UPHEAVAL),
        (keys::SCYTHE, b::RISING_REAP),
    ] {
        let mut w = mage();
        step(&mut w, SPACE | button);
        assert_eq!(doing(&w), Some(wanted), "space and {button:#x}");

        let mut w = mage();
        aloft(&mut w, 0, 3.0);
        step(&mut w, SPACE | button);
        assert_eq!(doing(&w), Some(wanted), "space and {button:#x} in the air");
    }
}

// ---------------------------------------------------------------------------
// The spike
// ---------------------------------------------------------------------------

#[test]
fn holding_the_spike_buys_a_tier_and_every_tier_costs_more() {
    let hold = |frames: u32| {
        let mut w = mage();
        let red = w.players[0].health;
        for _ in 0..frames.max(1) {
            step(&mut w, keys::YOUR_BLOOD);
        }
        (w.players[0].tier, red - w.players[0].health)
    };
    let (one, tap) = hold(1);
    let (two, mid) = hold(t::tier_two() as u32 + 2);
    let (three, top) = hold(t::tier_three() as u32 + 2);
    assert!(one <= 1 && two == 2 && three == 3, "{one} {two} {three}");
    assert!(tap < mid && mid < top, "{tap} < {mid} < {top}");
}

#[test]
fn the_spike_at_two_is_three_spikes_and_at_three_a_field_in_waves() {
    let mut w = mage();
    for _ in 0..(t::tier_two() as u32 + 2) {
        step_looking(&mut w, keys::YOUR_BLOOD, -3000);
    }
    run(&mut w, 40, 0);
    let spikes = effects_of(&w, EffectKind::BlackSpike);
    assert_eq!(spikes.len(), 3, "three spikes at the second tier");

    let mut w = mage();
    for _ in 0..(t::tier_three() as u32 + 2) {
        step_looking(&mut w, keys::YOUR_BLOOD, -3000);
    }
    run(&mut w, 40, 0);
    let field = effects_of(&w, EffectKind::BlackSpike);
    assert_eq!(field.len(), 1, "one field at the third");
    assert_eq!(field[0].reach, t::spike_field_radius());
    assert!(
        field[0].life > t::spike_erupt(),
        "and it stands for its waves"
    );
}

#[test]
fn a_spike_field_hits_him_once_a_wave() {
    let mut w = mage();
    w.players[1].pos = at(4.0, 0.0, 8.0);
    let mut placed = None;
    for pitch in PITCHES {
        let mut trial = w.clone();
        for _ in 0..(t::tier_three() as u32 + 2) {
            step_looking(&mut trial, keys::YOUR_BLOOD, pitch);
        }
        // Let go where she was looking: the aim locks on the release.
        step_looking(&mut trial, 0, pitch);
        run(&mut trial, 40, 0);
        let field = effects_of(&trial, EffectKind::BlackSpike);
        if field.first().is_some_and(|f| {
            V3::new(f.pos.x.sub(trial.players[1].pos.x), Fx::ZERO, Fx::ZERO)
                .flat_len()
                .raw()
                < metres(2.5).raw()
        }) {
            placed = Some(trial);
            break;
        }
    }
    let mut w = placed.expect("a field under him");
    // Hold him there.
    let mut hits = 0;
    let mut last = w.players[1].health;
    for _ in 0..(t::spike_wave_every() as u32 * t::spike_waves() as u32 + 10) {
        w.players[1].pos = V3::new(metres(4.0), w.players[1].pos.y, metres(8.0));
        step(&mut w, 0);
        if w.players[1].health < last {
            hits += 1;
        }
        last = w.players[1].health;
    }
    assert!(hits >= 2, "more than one wave caught him: {hits}");
}

// ---------------------------------------------------------------------------
// The takeoffs and the air
// ---------------------------------------------------------------------------

#[test]
fn blood_wings_carry_her_and_leave_a_stain() {
    let mut w = mage();
    let from = w.players[0].pos;
    step_looking(&mut w, SPACE | keys::MY_BLOOD, 3000);
    run(&mut w, 20, 0);
    let p = w.players[0];
    let gone = p.pos.sub(from).len();
    assert!(
        gone.raw() > metres(5.0).raw(),
        "she burst a long way: {}",
        as_f(gone)
    );
    assert!(!pools(&w).is_empty(), "and what it cost her is a stain");
}

#[test]
fn the_upheaval_launches_him_from_range() {
    let mut w = mage();
    w.players[1].pos = at(6.0, 0.0, 8.0);
    let launched = at_some_pitch(
        &w,
        &PITCHES,
        |w, pitch| step_looking(w, SPACE | keys::YOUR_BLOOD, pitch),
        |w| w.players[1].pos.y.raw() > metres(1.0).raw(),
        40,
    );
    assert!(launched.is_some(), "twelve metres off, and up he went");
}

/// The Stake thrown from above at a body `height` up.
fn stake_at(height: f32) -> Option<World> {
    let mut w = mage();
    aloft(&mut w, 0, 4.0);
    w.players[1].pos = at(-2.0, height, 8.0);
    w.players[1].grounded = height <= 0.0;
    at_some_pitch(
        &w,
        &[-1000, -2000, -3000, -4000, -5000, -6000],
        move |w, pitch| {
            w.players[0].vel.y = Fx::ZERO;
            if height > 0.0 {
                w.players[1].vel.y = Fx::ZERO;
            }
            step_looking(w, keys::YOUR_BLOOD, pitch);
        },
        |w| w.players[1].held_by == NAILED,
        30,
    )
}

#[test]
fn the_stake_pins_somebody_in_the_air() {
    let w = stake_at(3.0).expect("pinned");
    assert!(matches!(w.players[1].action, Action::Held { .. }));
}

#[test]
fn the_stake_lands_in_an_area_on_the_floor() {
    let mut w = mage();
    aloft(&mut w, 0, 4.0);
    w.players[1].pos = at(-1.0, 0.0, 8.0);
    let hit = at_some_pitch(
        &w,
        &[-4000, -5000, -6000, -7000, -8000],
        |w, pitch| step_looking(w, keys::YOUR_BLOOD, pitch),
        |w| w.players[1].health < w.players[1].full_health(),
        40,
    );
    assert!(hit.is_some(), "the landing caught him on the floor");
}

#[test]
fn meteor_storm_the_upheaval_then_the_stake() {
    let mut w = mage();
    w.players[1].pos = at(-1.0, 0.0, 8.0);
    // The Upheaval: he goes up, and she hops.
    let mut launched = at_some_pitch(
        &w,
        &PITCHES,
        |w, pitch| step_looking(w, SPACE | keys::YOUR_BLOOD, pitch),
        |w| w.players[1].pos.y.raw() > metres(1.0).raw() && w.players[0].action.actionable(),
        50,
    )
    .expect("launched, and her free to act");
    if launched.players[0].grounded {
        launched.players[0].vel.y = metres(8.0);
        launched.players[0].grounded = false;
        step(&mut launched, 0);
    }
    // The Stake at him: pinned.
    let pinned = at_some_pitch(
        &launched,
        &[-6000, -4000, -2000, 0, 2000, 4000],
        |w, pitch| step_looking(w, keys::YOUR_BLOOD, pitch),
        |w| w.players[1].held_by == NAILED,
        30,
    );
    assert!(
        pinned.is_some(),
        "a Stake after the Upheaval pins him in the air"
    );
}

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
fn what_she_cannot_hold_goes_to_the_well() {
    let mut w = mage();
    stain(&mut w, at(-4.0, 0.0, 8.0), 100);
    run(&mut w, 40, Input::W);
    assert!(
        pools(&w).is_empty(),
        "at full health the stain is still taken"
    );
    assert!(
        w.players[0].well > 50,
        "into the Well: {}",
        w.players[0].well
    );
}

#[test]
fn her_own_spilled_blood_heals_her_like_any() {
    let mut w = mage();
    wounded(&mut w);
    let red = w.players[0].health;
    place(
        &mut w,
        Effect::pool(0, Class::BloodMage, b::WINGS, at(-4.0, 0.0, 8.0), 100),
    );
    run(&mut w, 40, Input::W);
    assert!(w.players[0].health > red, "no stain is special");
}

#[test]
fn the_teleport_to_a_stain_drinks_it_and_bursts() {
    let mut w = mage();
    wounded(&mut w);
    let red = w.players[0].health;
    w.players[1].pos = at(6.0, 0.0, 8.0);
    stain(&mut w, at(5.0, 0.0, 8.0), 100);
    let arrived = at_some_pitch(
        &w,
        &PITCHES,
        |w, pitch| step_looking(w, Input::SHIFT | Input::W, pitch),
        |w| w.players[0].pos.x.raw() > metres(3.0).raw(),
        3,
    )
    .expect("she teleported to the stain");
    let mut w = arrived;
    run(&mut w, 5, 0);
    assert!(w.players[0].health > red, "it healed her");
    assert!(
        w.players[1].health < w.players[1].full_health(),
        "and the burst caught him beside it"
    );
}

#[test]
fn every_hit_on_a_creature_leaves_a_stain_worth_its_blow() {
    let mut w = World::hunt([Class::BloodMage; 2]);
    let beast = w.monsters[0].expect("a creature").pos;
    let bolt = Effect::cast(
        EffectKind::Haemorrhage,
        0,
        Class::BloodMage,
        b::HAEMORRHAGE,
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
// The Carpet of blood and Coagulate
// ---------------------------------------------------------------------------

/// Hold the left click for `frames`, looking a little down.
fn carpet_for(w: &mut World, frames: u32) {
    for _ in 0..frames {
        step_looking(w, keys::MY_BLOOD, -2000);
    }
}

#[test]
fn holding_the_left_click_lays_a_carpet_that_grows_and_costs_her() {
    let mut w = mage();
    let startup = sim::moves::get(Class::BloodMage, b::HAEMORRHAGE).startup as u32;
    carpet_for(&mut w, startup + 3);
    assert_eq!(doing(&w), Some(b::CARPET), "the tap became the field");
    let small = effects_of(&w, EffectKind::Carpet)
        .first()
        .expect("a carpet")
        .reach;
    let red = w.players[0].health;
    carpet_for(&mut w, 60);
    let big = effects_of(&w, EffectKind::Carpet)[0].reach;
    assert!(
        big.raw() > small.raw(),
        "it grew: {} to {}",
        as_f(small),
        as_f(big)
    );
    assert!(w.players[0].health < red, "and she paid for it");
}

#[test]
fn the_carpet_hurts_and_slows_and_breaks_into_stains() {
    let mut w = mage();
    let startup = sim::moves::get(Class::BloodMage, b::HAEMORRHAGE).startup as u32;
    carpet_for(&mut w, startup + 3);
    let floor = effects_of(&w, EffectKind::Carpet)[0].pos;
    let his = w.players[1].health;
    let mut slowed = false;
    for _ in 0..(t::carpet_tick() as u32 * 2 + 2) {
        w.players[1].pos = floor;
        step_looking(&mut w, keys::MY_BLOOD, -2000);
        slowed |= w.players[1].slowed > 0;
    }
    assert!(w.players[1].health < his, "it hurt him");
    assert!(slowed, "and slowed him");
    let paid = effects_of(&w, EffectKind::Carpet)[0].banked;
    assert!(paid > 0, "what she paid is in it");
    w.players[1].pos = at(30.0, 0.0, 8.0);
    step(&mut w, 0);
    assert!(
        effects_of(&w, EffectKind::Carpet).is_empty(),
        "let go, it breaks"
    );
    assert!(pools(&w).len() >= 2, "into stains");
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

/// A thrall of hers, of `essence`, at `pos`, sent there.
fn thrall(w: &mut World, pos: V3, essence: i32) {
    let mut e = Effect::pool(0, Class::BloodMage, b::SUMMON, pos, essence);
    e.kind = EffectKind::Thrall;
    e.life = u16::MAX;
    place(w, e);
}

#[test]
fn holding_the_right_click_raises_her_stains_as_thralls() {
    let mut w = mage();
    for k in 0..3 {
        stain(&mut w, at(-6.0 + 2.0 * k as f32, 0.0, 10.0), 80);
    }
    run(&mut w, t::summon_at() as u32 + 12, keys::SCYTHE);
    let up = thralls(&w);
    assert_eq!(up.len(), 3, "every stain near her stood up");
    assert!(pools(&w).is_empty(), "a thrall is the stain");
    assert!(up.iter().all(|e| e.banked > 60), "of its essence");
}

#[test]
fn no_more_thralls_than_the_cap() {
    let mut w = mage();
    for k in 0..8 {
        stain(
            &mut w,
            at(-8.0 + 1.6 * (k % 4) as f32, 0.0, 6.0 + 3.0 * (k / 4) as f32),
            80,
        );
    }
    run(&mut w, t::summon_at() as u32 + 12, keys::SCYTHE);
    assert_eq!(thralls(&w).len() as i32, t::thrall_cap().min(8));
}

#[test]
fn a_bite_deals_what_it_spends_and_spills_it_and_fills_no_well() {
    let mut w = mage();
    w.players[1].pos = at(4.0, 0.0, 8.0);
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
    assert_eq!(w.players[0].well, 0, "a bite fills no Well");
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
    assert!(
        rest > 0 && rest < 400,
        "a stain of what it had, less the blow: {rest}"
    );
}

#[test]
fn command_sends_them_where_she_points() {
    let mut w = mage();
    thrall(&mut w, at(-5.0, 0.0, 8.0), 200);
    step_looking(&mut w, keys::SCYTHE, -2000);
    run(&mut w, 40, 0);
    let th = thralls(&w)[0];
    assert!(
        th.pos.x.raw() > metres(-2.0).raw(),
        "it went out along her look: {}",
        as_f(th.pos.x)
    );
}

#[test]
fn the_rising_reap_hurls_thralls_and_drinks_the_stains_in_front() {
    let mut w = mage();
    wounded(&mut w);
    let red = w.players[0].health;
    thrall(&mut w, at(-4.5, 0.0, 8.0), 200);
    let mut s = Effect::pool(0, Class::BloodMage, b::HAEMORRHAGE, at(-3.0, 0.0, 8.0), 100);
    s.age = t::stain_settles();
    place(&mut w, s);
    step(&mut w, SPACE | keys::SCYTHE);
    run(&mut w, 30, 0);
    let th = thralls(&w)[0];
    assert!(
        th.pos.x.raw() > metres(-1.0).raw(),
        "hurled forward: {}",
        as_f(th.pos.x)
    );
    assert!(w.players[0].health > red, "and the stain was drunk");
}

// ---------------------------------------------------------------------------
// The Well and the finishers
// ---------------------------------------------------------------------------

fn brimming() -> World {
    let mut w = mage();
    w.players[0].well = t::well_full() as u16;
    w
}

#[test]
fn a_full_well_turns_the_end_of_the_carpet_into_the_thorn_garden() {
    let mut w = brimming();
    let m = sim::moves::get(Class::BloodMage, b::CARPET);
    let start = sim::moves::get(Class::BloodMage, b::HAEMORRHAGE).startup as u32;
    carpet_for(&mut w, start + m.channel as u32 + 4);
    assert_eq!(doing(&w), Some(b::THORN_GARDEN));
    assert_eq!(w.players[0].well, 0, "the Well is spent on it");
    run(&mut w, 80, 0);
    assert!(!effects_of(&w, EffectKind::Thorns).is_empty(), "thorns");
}

#[test]
fn without_a_full_well_the_hold_just_ends() {
    let mut w = mage();
    let m = sim::moves::get(Class::BloodMage, b::CARPET);
    let start = sim::moves::get(Class::BloodMage, b::HAEMORRHAGE).startup as u32;
    carpet_for(&mut w, start + m.channel as u32 + 4);
    assert_ne!(doing(&w), Some(b::THORN_GARDEN));
}

#[test]
fn exsanguinate_takes_the_stains_in_front_into_one_blow() {
    let mut w = brimming();
    w.players[1].pos = at(0.0, 0.0, 8.0);
    stain(&mut w, at(-2.0, 0.0, 8.0), 200);
    let cap = sim::moves::get(Class::BloodMage, b::BLACK_SPIKE).channel as u32;
    run(&mut w, cap + 4, keys::YOUR_BLOOD);
    assert_eq!(doing(&w), Some(b::EXSANGUINATE));
    let his = w.players[1].health;
    for _ in 0..80 {
        w.players[1].pos = at(0.0, 0.0, 8.0);
        step(&mut w, 0);
    }
    let took = his - w.players[1].health;
    let row = sim::moves::get(Class::BloodMage, b::EXSANGUINATE).damage;
    assert!(took >= row + 150, "the row and the stain: {took}");
}

#[test]
fn the_golem_is_the_well_and_what_it_fused() {
    let mut w = brimming();
    let cap = sim::moves::get(Class::BloodMage, b::SUMMON).channel as u32;
    let start = sim::moves::get(Class::BloodMage, b::COMMAND).startup as u32;
    run(&mut w, start + cap + 4, keys::SCYTHE);
    assert_eq!(doing(&w), Some(b::GOLEM));
    run(&mut w, 80, 0);
    let golem = effects_of(&w, EffectKind::Golem);
    assert_eq!(golem.len(), 1, "a golem");
    assert!(golem[0].banked > t::well_full() - 3 * t::pool_drain());
}

#[test]
fn the_creature_turns_on_the_golem() {
    let mut w = World::hunt([Class::BloodMage; 2]);
    let beast = w.monsters[0].expect("a creature");
    w.players[0].pos = beast.pos.add(at(30.0, 0.0, 0.0));
    w.players[1].health = 0;
    let mut g = Effect::pool(
        0,
        Class::BloodMage,
        b::GOLEM,
        beast.pos.add(at(4.0, 0.0, 0.0)),
        800,
    );
    g.kind = EffectKind::Golem;
    g.life = u16::MAX;
    place(&mut w, g);
    run(&mut w, 120, 0);
    let golem = effects_of(&w, EffectKind::Golem);
    let seen = w.monsters[0].expect("still there").brain.seen;
    let at_golem = golem
        .first()
        .is_none_or(|g| g.pos.sub(seen).flat_len().raw() < metres(3.0).raw());
    assert!(at_golem, "what it watched was the golem, not her");
}

// ---------------------------------------------------------------------------
// The Hemoplague
// ---------------------------------------------------------------------------

#[test]
fn the_plague_hurts_over_time_holds_nobody_and_pools_at_its_centre() {
    let mut w = mage();
    w.players[1].pos = at(4.0, 0.0, 8.0);
    let cloud = Effect::cast(
        EffectKind::Plague,
        0,
        Class::BloodMage,
        b::HEMOPLAGUE,
        at(4.0, 0.0, 8.0),
        V3::ZERO,
        sim::moves::get(Class::BloodMage, b::HEMOPLAGUE).radius,
    );
    place(&mut w, cloud);
    let mut stunned = false;
    for _ in 0..(t::plague_tick() as u32 * 4 + 2) {
        w.players[1].pos = at(4.0, 0.0, 8.0);
        step(&mut w, 0);
        stunned |= !w.players[1].action.actionable();
    }
    let took = w.players[1].full_health() - w.players[1].health;
    let tick = sim::moves::get(Class::BloodMage, b::HEMOPLAGUE).damage;
    assert!(took >= tick * 4, "four ticks: {took}");
    assert!(!stunned, "and it held nobody");
    let centre = pools(&w);
    assert_eq!(centre.len(), 1, "one stain");
    assert!(
        V3::new(centre[0].pos.x.sub(metres(4.0)), Fx::ZERO, Fx::ZERO)
            .flat_len()
            .raw()
            < metres(0.1).raw(),
        "at the cloud's centre"
    );
}

#[test]
fn e_casts_the_plague_where_she_aims() {
    let mut w = mage();
    step_looking(&mut w, keys::PLAGUE, -2000);
    let startup = sim::moves::get(Class::BloodMage, b::HEMOPLAGUE).startup;
    run(&mut w, startup as u32 + 4, 0);
    let cloud = effects_of(&w, EffectKind::Plague);
    assert_eq!(cloud.len(), 1, "a cloud");
    assert!(
        cloud[0].pos.x.raw() > metres(-4.0).raw(),
        "out where she aimed"
    );
}

// ---------------------------------------------------------------------------
// Frenzy
// ---------------------------------------------------------------------------
