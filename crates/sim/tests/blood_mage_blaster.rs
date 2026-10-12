//! The Blood mage as the blaster: what a player would notice if any of it
//! broke.
//!
//! `docs/design/exploration/0015_blood_mage_blaster.md` is the direction: the
//! three clicks held on the floor through three tiers, quick in the air,
//! takeoffs with space; a stain collected by walking into it, teleporting to
//! it or hitting through it; Frenzy on `Q` and the Hemoplague on `E`. Each test
//! below is one piece, and the last few are the combos the pieces were chosen
//! for.

use sim::class::Class;
use sim::effects::{Effect, EffectKind};
use sim::moves::blood as b;
use sim::moves::blood::keys;
use sim::state::{Action, NAILED, NO_VESSEL};
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

/// A stain of the other fighter's blood, hers to collect, at `pos`.
fn stain(w: &mut World, pos: V3, essence: i32) {
    place(
        w,
        Effect::pool(0, Class::BloodMage, b::SWEEP, pos, essence),
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

#[test]
fn on_the_floor_the_clicks_charge_and_q_and_e_are_frenzy_and_the_plague() {
    for (button, wanted) in [
        (keys::MY_BLOOD, b::TRANSFUSION),
        (keys::YOUR_BLOOD, b::BLACK_SPIKE),
        (keys::SCYTHE, b::SWEEP),
        (keys::FRENZY, b::FRENZY),
        (keys::PLAGUE, b::HEMOPLAGUE),
    ] {
        let mut w = mage();
        step(&mut w, button);
        assert_eq!(doing(&w), Some(wanted), "button {button:#x} on the floor");
    }
}

#[test]
fn in_the_air_the_clicks_are_haemorrhage_the_stake_and_the_chain() {
    for (button, wanted) in [
        (keys::MY_BLOOD, b::HAEMORRHAGE),
        (keys::YOUR_BLOOD, b::STAKE),
        (keys::SCYTHE, b::CHAIN),
    ] {
        let mut w = mage();
        aloft(&mut w, 0, 3.0);
        step(&mut w, button);
        assert_eq!(doing(&w), Some(wanted), "button {button:#x} in the air");
    }
}

#[test]
fn space_and_a_click_is_a_takeoff() {
    for (button, wanted) in [
        (keys::MY_BLOOD, b::WINGS),
        (keys::YOUR_BLOOD, b::UPHEAVAL),
        (keys::SCYTHE, b::LEAP),
    ] {
        let mut w = mage();
        step(&mut w, SPACE | button);
        assert_eq!(doing(&w), Some(wanted), "space and {button:#x}");
    }
}

// ---------------------------------------------------------------------------
// The charge
// ---------------------------------------------------------------------------

/// Hold left click for `hold` frames and let go: the tier she reached, and
/// what it cost.
fn transfusion(hold: u32) -> (u8, i32, World) {
    let mut w = mage();
    let red = w.players[0].health;
    for _ in 0..hold.max(1) {
        step(&mut w, keys::MY_BLOOD);
    }
    let tier = w.players[0].tier;
    step(&mut w, 0);
    (tier, red - w.players[0].health, w)
}

#[test]
fn holding_buys_a_tier_and_every_tier_costs_more() {
    let (one, tap, _) = transfusion(1);
    let (two, mid, _) = transfusion(t::tier_two() as u32 + 2);
    let (three, full, _) = transfusion(t::tier_three() as u32 + 2);
    assert_eq!((one, two, three), (1, 2, 3), "the three tiers");
    assert!(tap < mid && mid < full, "costs {tap} < {mid} < {full}");
}

#[test]
fn the_bolt_carries_its_tier() {
    let (_, _, mut w) = transfusion(t::tier_three() as u32 + 2);
    run(&mut w, 12, 0);
    let bolt = effects_of(&w, EffectKind::Haemorrhage);
    assert_eq!(bolt.len(), 1, "a bolt in flight");
    assert_eq!(bolt[0].slot, b::TRANSFUSION);
    assert_eq!(bolt[0].banked, 3, "at the third tier");
}

#[test]
fn a_jump_out_of_a_hold_stores_it_and_a_fresh_click_lets_it_go_from_the_air() {
    let mut w = mage();
    for _ in 0..(t::tier_three() as u32 + 2) {
        step(&mut w, keys::MY_BLOOD);
    }
    step(&mut w, keys::MY_BLOOD | SPACE);
    let p = w.players[0];
    assert_eq!(p.vessel, b::TRANSFUSION, "stored");
    assert_eq!(p.vessel_tier, 3, "at the tier it reached");
    run(&mut w, 2, SPACE | keys::MY_BLOOD);
    assert!(!w.players[0].grounded, "and she jumped");
    assert_ne!(
        doing(&w),
        Some(b::HAEMORRHAGE),
        "the held click did not throw the air move"
    );
    // Let go of the click, then press it again in the air.
    step(&mut w, 0);
    let red = w.players[0].health;
    step(&mut w, keys::MY_BLOOD);
    assert_eq!(doing(&w), Some(b::TRANSFUSION), "the stored Transfusion");
    assert_eq!(w.players[0].tier, 3);
    assert_eq!(w.players[0].vessel, NO_VESSEL, "and the vessel is empty");
    assert!(
        red - w.players[0].health <= 1,
        "nothing paid again: {} -> {}",
        red,
        w.players[0].health
    );
}

#[test]
fn a_stored_charge_bleeds_her() {
    let mut w = mage();
    run(&mut w, 3, keys::YOUR_BLOOD);
    step(&mut w, keys::YOUR_BLOOD | SPACE);
    assert_eq!(w.players[0].vessel, b::BLACK_SPIKE);
    run(&mut w, 70, 0);
    let red = w.players[0].health;
    run(&mut w, t::vessel_bleeds_every() as u32 * 3, 0);
    assert!(w.players[0].health < red, "the vessel bled her");
}

#[test]
fn a_hit_during_the_hold_loses_it() {
    let mut w = mage();
    run(&mut w, 20, keys::YOUR_BLOOD);
    assert!(matches!(w.players[0].action, Action::Channel { .. }));
    w.players[0].action = Action::HitStun { left: 10 };
    run(&mut w, 12, keys::YOUR_BLOOD);
    assert_eq!(w.players[0].vessel, NO_VESSEL, "nothing was stored");
}

// ---------------------------------------------------------------------------
// The three charges
// ---------------------------------------------------------------------------

#[test]
fn the_third_tier_lance_throws_him_back() {
    let mut w = mage();
    w.players[1].pos = at(2.0, 0.0, 8.0);
    let hit = at_some_pitch(
        &w,
        &PITCHES,
        |w, pitch| {
            for _ in 0..(t::tier_three() as u32 + 2) {
                step_looking(w, keys::MY_BLOOD, pitch);
            }
        },
        |w| w.players[1].health < w.players[1].full_health(),
        40,
    )
    .expect("the lance found him");
    let mut w = hit;
    let before = w.players[1].pos.x;
    run(&mut w, 15, 0);
    assert!(
        w.players[1].pos.x.raw() > before.add(metres(1.5)).raw(),
        "thrown back along the lance: {} -> {}",
        as_f(before),
        as_f(w.players[1].pos.x)
    );
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
    assert!(field[0].life > t::spike_erupt(), "and it stands for its waves");
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

#[test]
fn a_tap_on_the_scythe_is_the_sweep_and_a_hold_is_tides() {
    let mut w = mage();
    step(&mut w, keys::SCYTHE);
    run(&mut w, 30, 0);
    assert!(effects_of(&w, EffectKind::Nova).is_empty(), "a tap swept");

    let ring = |hold: u32| {
        let mut w = mage();
        for _ in 0..hold {
            step(&mut w, keys::SCYTHE);
        }
        run(&mut w, 12, 0);
        effects_of(&w, EffectKind::Nova)
            .first()
            .map(|e| e.reach)
            .expect("the Tides ring")
    };
    let short = ring(14);
    let long = ring(14 + t::tier_three() as u32);
    assert!(long.raw() > short.raw(), "a longer hold is a wider ring");
}

#[test]
fn tides_throws_off_whoever_is_close() {
    let mut w = mage();
    w.players[1].pos = at(-4.0, 0.0, 8.0);
    let before = w.players[1].health;
    run(&mut w, 40, keys::SCYTHE);
    run(&mut w, 20, 0);
    assert!(w.players[1].health < before, "he was hurt");
    assert!(
        w.players[1].pos.x.raw() > metres(-3.5).raw(),
        "and thrown off: he is at {}",
        as_f(w.players[1].pos.x)
    );
}

// ---------------------------------------------------------------------------
// The air, and the takeoffs
// ---------------------------------------------------------------------------

#[test]
fn blood_wings_carry_her_and_leave_a_door_home() {
    let mut w = mage();
    let from = w.players[0].pos;
    step_looking(&mut w, SPACE | keys::MY_BLOOD, 3000);
    run(&mut w, 20, 0);
    let p = w.players[0];
    let gone = V3::new(p.pos.x.sub(from.x), p.pos.y.sub(from.y), p.pos.z.sub(from.z)).len();
    assert!(
        gone.raw() > metres(5.0).raw(),
        "she burst a long way: {}",
        as_f(gone)
    );
    let own: Vec<Effect> = pools(&w)
        .into_iter()
        .filter(|p| b::own_blood(p.slot))
        .collect();
    assert_eq!(own.len(), 1, "a stain of her own blood where she took off");
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
fn the_chain_yanks_a_fighter_to_her() {
    let mut w = mage();
    aloft(&mut w, 0, 1.0);
    w.players[1].pos = at(4.0, 0.0, 8.0);
    let before = as_f(w.players[1].pos.x.sub(w.players[0].pos.x));
    let pulled = at_some_pitch(
        &w,
        &PITCHES,
        |w, pitch| step_looking(w, keys::SCYTHE, pitch),
        |w| w.players[1].health < w.players[1].full_health(),
        20,
    )
    .expect("the chain caught him");
    let mut w = pulled;
    run(&mut w, 30, 0);
    let after = as_f(w.players[1].pos.x.sub(w.players[0].pos.x)).abs();
    assert!(
        after < before - 4.0,
        "he was yanked to her: {before} m apart, then {after}"
    );
}

#[test]
fn the_chain_on_the_floor_pulls_her_there_and_on_nothing_still_moves_her() {
    // Down at the floor ahead: she is pulled to the spot.
    let mut w = mage();
    aloft(&mut w, 0, 2.0);
    let from = w.players[0].pos;
    step_looking(&mut w, keys::SCYTHE, -5000);
    run(&mut w, 30, 0);
    let moved = as_f(w.players[0].pos.x.sub(from.x));
    assert!(moved > 3.0, "pulled to the floor ahead: {moved} m");
    // At the sky: still a little way along the aim.
    let mut w = mage();
    aloft(&mut w, 0, 2.0);
    let from = w.players[0].pos;
    step_looking(&mut w, keys::SCYTHE, 12000);
    run(&mut w, 20, 0);
    let moved = w.players[0].pos.sub(from).len();
    assert!(
        moved.raw() > metres(2.0).raw(),
        "never empty: {}",
        as_f(moved)
    );
}

#[test]
fn the_leap_lands_where_she_aimed_and_drinks_round_it() {
    let mut w = mage();
    wounded(&mut w);
    let red = w.players[0].health;
    let from = w.players[0].pos;
    // A stain where she will come down.
    let mut landed = None;
    for pitch in [-3000i16, -2500, -2000, -1500] {
        let mut trial = w.clone();
        step_looking(&mut trial, SPACE | keys::SCYTHE, pitch);
        let target = trial.players[0].aim_at();
        stain(&mut trial, target, 100);
        for _ in 0..90 {
            step(&mut trial, 0);
            if trial.players[0].grounded && doing(&trial).is_none() {
                break;
            }
        }
        let gone = as_f(trial.players[0].pos.x.sub(from.x));
        if gone > 3.0 {
            landed = Some(trial);
            break;
        }
    }
    let w = landed.expect("she leapt across");
    assert!(w.players[0].health > red, "and drank where she landed");
}

// ---------------------------------------------------------------------------
// Everything collects
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
fn her_own_blood_never_heals_her() {
    let mut w = mage();
    wounded(&mut w);
    let red = w.players[0].health;
    place(
        &mut w,
        Effect::pool(0, Class::BloodMage, b::WINGS, at(-4.0, 0.0, 8.0), 100),
    );
    run(&mut w, 40, Input::W);
    assert!(w.players[0].health <= red, "her own blood is a door");
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
fn a_hit_through_a_stain_is_harder() {
    let sweep = |with_stain: bool| {
        let mut w = mage();
        w.players[1].pos = at(-4.2, 0.0, 8.0);
        if with_stain {
            stain(&mut w, at(-4.2, 0.0, 8.0), 100);
        }
        run(&mut w, 1, 0);
        step(&mut w, keys::SCYTHE);
        run(&mut w, 20, 0);
        w.players[1].full_health() - w.players[1].health
    };
    let bare = sweep(false);
    let through = sweep(true);
    assert!(bare > 0, "the sweep landed");
    assert!(
        through >= bare + t::stain_hit_bonus(100) - 1,
        "through a stain it hit harder: {bare} then {through}"
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
        b::TRANSFUSION,
        beast.add(at(0.0, 1.5, 0.0)),
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        Fx::ZERO,
    );
    place(&mut w, bolt);
    let mut biggest = 0;
    for _ in 0..10 {
        w.advance([Input::default(); 2]);
        for p in pools(&w) {
            biggest = biggest.max(p.pool_volume());
        }
    }
    let blow = sim::moves::get(Class::BloodMage, b::TRANSFUSION).damage;
    assert!(
        biggest >= blow - 2 * t::pool_drain(),
        "a stain of {biggest} for a blow of {blow}"
    );
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
    assert!(cloud[0].pos.x.raw() > metres(-4.0).raw(), "out where she aimed");
}

// ---------------------------------------------------------------------------
// Frenzy
// ---------------------------------------------------------------------------

fn frenzied() -> World {
    let mut w = mage();
    wounded(&mut w);
    step(&mut w, keys::FRENZY);
    run(&mut w, 30, 0);
    assert!(w.players[0].frenzy > 0, "in a Frenzy");
    w
}

#[test]
fn frenzy_plants_her() {
    let mut w = frenzied();
    let from = w.players[0].pos;
    run(&mut w, 30, Input::W | SPACE);
    assert_eq!(w.players[0].pos, from, "she did not walk or jump");
}

#[test]
fn frenzy_pulls_her_stains_in_and_explodes_them_and_heals_nothing() {
    let mut w = frenzied();
    w.players[1].pos = at(-2.0, 0.0, 8.0);
    stain(&mut w, at(5.0, 0.0, 8.0), 100);
    let red = w.players[0].health;
    let before = w.players[1].health;
    run(&mut w, 60, 0);
    assert!(pools(&w).is_empty(), "the stain came to her and went off");
    assert!(w.players[1].health < before, "and he was hurt");
    assert!(w.players[0].health <= red, "and none of it healed her");
}

#[test]
fn frenzy_hurts_and_bleeds_whoever_is_near() {
    let mut w = frenzied();
    w.players[1].pos = at(-3.0, 0.0, 8.0);
    let before = w.players[1].health;
    run(&mut w, 30, 0);
    assert!(w.players[1].health < before);
    assert!(w.players[1].bleeding > 0, "he bleeds");
}

#[test]
fn when_frenzy_ends_what_she_lost_is_gone() {
    let mut w = frenzied();
    run(&mut w, t::frenzy_lasts() as u32 + 5, 0);
    assert_eq!(w.players[0].frenzy, 0, "over");
    assert_eq!(w.players[0].grey, 0, "and nothing left to win back");
}

#[test]
fn a_teleport_in_a_frenzy_buys_more_of_it() {
    let mut w = frenzied();
    stain(&mut w, at(11.0, 0.0, 8.0), 60);
    // Out of the pull's reach, so it waits to be teleported to.
    let left = w.players[0].frenzy;
    let arrived = at_some_pitch(
        &w,
        &PITCHES,
        |w, pitch| step_looking(w, Input::SHIFT | Input::W, pitch),
        |w| w.players[0].pos.x.raw() > metres(9.0).raw(),
        3,
    );
    let w = arrived.expect("she teleported along the road");
    assert!(
        w.players[0].frenzy > left,
        "more Frenzy: {} -> {}",
        left,
        w.players[0].frenzy
    );
}

// ---------------------------------------------------------------------------
// The combos
// ---------------------------------------------------------------------------

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
    assert!(pinned.is_some(), "a Stake after the Upheaval pins him in the air");
}
