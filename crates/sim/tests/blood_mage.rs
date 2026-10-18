//! The Blood mage's clicks, stains, thralls and grey: what a player would
//! notice if any of it broke.
//!
//! `docs/design/exploration/0021_blood_mage_territory.md` is the direction:
//! the left click is thorns (the Thorn lash tapped, the Bramble growing while
//! held), the middle the spikes (one to seven), the right the scythe (the
//! Sweep tapped and strung, the Reap held); a hold kept to its end is a
//! finisher that spends all her grey; `E` plants a heart; grey holds two
//! seconds, then goes fast. Her ground, the swim and the strong version are
//! `tests/territory.rs`. Each test below is one piece.

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
        Effect::pool(0, Class::BloodMage, b::THORN_LASH, pos, essence),
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
// ---------------------------------------------------------------------------
// The map
// ---------------------------------------------------------------------------

#[test]
fn on_the_floor_a_tap_is_the_quick_thing_and_q_and_e_gather_and_plant() {
    for (button, wanted) in [
        (keys::THORNS, b::THORN_LASH),
        (keys::SPIKES, b::BLACK_SPIKE),
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
fn held_the_left_click_is_the_bramble_and_the_right_the_reap() {
    for (button, tap, wanted) in [
        (keys::THORNS, b::THORN_LASH, b::BRAMBLE),
        (keys::SCYTHE, b::SWEEP, b::REAP),
    ] {
        let mut w = mage();
        hold(&mut w, button, field_opens(tap) + 3);
        assert_eq!(doing(&w), Some(wanted), "button {button:#x} held");
    }
}
#[test]
fn in_the_air_the_middle_click_is_the_stake() {
    for (button, wanted) in [
        (keys::THORNS, b::THORN_LASH),
        (keys::SPIKES, b::STAKE),
        (keys::SCYTHE, b::SWEEP),
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
        (keys::THORNS, b::THORNFALL),
        (keys::SPIKES, b::UPHEAVAL),
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
fn holding_the_spike_buys_a_tier_and_every_tier_costs_more() {
    let hold = |frames: u32| {
        let mut w = mage();
        let red = w.players[0].health;
        for _ in 0..frames.max(1) {
            step(&mut w, keys::SPIKES);
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
fn a_longer_hold_raises_more_spikes_up_to_seven() {
    let raised = |frames: u32| {
        let mut w = mage();
        w.players[1].pos = at(40.0, 0.0, 8.0);
        hold(&mut w, keys::SPIKES, frames.max(1));
        run(&mut w, 22, 0);
        effects_of(&w, EffectKind::BlackSpike).len()
    };
    let cap = sim::moves::get(Class::BloodMage, b::BLACK_SPIKE).channel as u32;
    let tap = raised(1);
    let half = raised(cap / 2);
    let most = raised(cap - 2);
    assert_eq!(tap, 1, "a tap is one spike");
    assert!(half > tap && most > half, "{tap} then {half} then {most}");
    assert!(
        most as i32 <= t::spikes_most(),
        "never past the most: {most}"
    );
}

#[test]
fn a_spike_aimed_through_a_stain_standing_goes_to_it() {
    // Where a tapped spike lands on bare floor, looking a little down.
    let pitch = -1000;
    let mut probe = mage();
    probe.players[1].pos = at(40.0, 0.0, 8.0);
    step_looking(&mut probe, keys::SPIKES, pitch);
    run(&mut probe, 22, 0);
    let floor = effects_of(&probe, EffectKind::BlackSpike)[0].pos;
    // A stain well short of it on the same line: the crosshair is on the
    // floor behind it and passes over its disc, through its column.
    let mut w = mage();
    w.players[1].pos = at(40.0, 0.0, 8.0);
    let me = w.players[0].pos;
    let short = V3::new(
        me.x.add(floor.x.sub(me.x).mul(Fx::ratio(3, 5))),
        floor.y,
        me.z.add(floor.z.sub(me.z).mul(Fx::ratio(3, 5))),
    );
    assert!(
        short.sub(floor).flat_len().raw() > metres(2.5).raw(),
        "the stain is not on the floor the crosshair is on"
    );
    stain(&mut w, short, 80);
    for e in w.effects.iter_mut().flatten() {
        e.age = t::stain_settles() + 1;
    }
    step_looking(&mut w, keys::SPIKES, pitch);
    let mut erupted = false;
    for _ in 0..22 {
        step(&mut w, 0);
        erupted |= effects_of(&w, EffectKind::BlackSpike)
            .iter()
            .any(|e| e.erupted() && e.pos.sub(short).flat_len().raw() < metres(0.5).raw());
    }
    assert!(
        erupted,
        "the spike went to the stain the crosshair passed through"
    );
}

#[test]
fn a_chain_of_stains_erupts_link_by_link_not_all_at_once() {
    let mut w = mage();
    w.players[1].pos = at(40.0, 0.0, 8.0);
    let mut spike_at = None;
    let mut aimed = 0;
    for pitch in PITCHES {
        let mut trial = w.clone();
        step_looking(&mut trial, keys::SPIKES, pitch);
        run(&mut trial, 22, 0);
        if let Some(s) = effects_of(&trial, EffectKind::BlackSpike).first() {
            spike_at = Some(s.pos);
            aimed = pitch;
            break;
        }
    }
    let first = spike_at.expect("a spike came up");
    let second = first.add(at(0.0, 0.0, 2.0));
    stain(&mut w, first, 100);
    stain(&mut w, second, 100);
    for e in w.effects.iter_mut().flatten() {
        e.age = t::stain_settles() + 1;
    }
    step_looking(&mut w, keys::SPIKES, aimed);
    let mut gone_first = None;
    let mut gone_second = None;
    for f in 0..80 {
        step(&mut w, 0);
        // An eruption standing on the spot: the stain there went up. (What
        // the spike cost lands there too, so the stains themselves are not
        // the thing to watch.)
        let erupted = |p: V3| {
            effects_of(&w, EffectKind::BlackSpike)
                .iter()
                .any(|e| e.erupted() && e.pos.sub(p).flat_len().raw() < metres(0.5).raw())
        };
        if gone_first.is_none() && erupted(first) {
            gone_first = Some(f);
        }
        if gone_second.is_none() && erupted(second) {
            gone_second = Some(f);
        }
    }
    let (a, b) = (
        gone_first.expect("the first erupted"),
        gone_second.expect("the second erupted"),
    );
    assert!(b > a, "the second went after the first: {a} then {b}");
    assert!(
        b - a <= 2 * t::chain_delay() as u32 + 2,
        "and soon after: {a} then {b}"
    );
}

// ---------------------------------------------------------------------------
// The takeoffs and the air
// ---------------------------------------------------------------------------

#[test]
fn thornfall_carries_her_and_lands_in_a_ring_of_thorns() {
    let mut w = mage();
    let from = w.players[0].pos;
    let mut ring = None;
    step_looking(&mut w, SPACE | keys::THORNS, 3000);
    for _ in 0..20 {
        step(&mut w, 0);
        if let Some(n) = effects_of(&w, EffectKind::Nova).first() {
            ring = ring.or(Some(n.pos));
        }
    }
    let p = w.players[0];
    let gone = p.pos.sub(from).len();
    assert!(
        gone.raw() > metres(5.0).raw(),
        "she burst a long way: {}",
        as_f(gone)
    );
    let ring = ring.expect("a ring of thorns");
    assert!(
        ring.sub(from).len().raw() > metres(4.0).raw(),
        "where she landed, not where she left"
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
        |w, pitch| step_looking(w, SPACE | keys::SPIKES, pitch),
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
            step_looking(w, keys::SPIKES, pitch);
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
        |w, pitch| step_looking(w, keys::SPIKES, pitch),
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
        |w, pitch| step_looking(w, SPACE | keys::SPIKES, pitch),
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
        |w, pitch| step_looking(w, keys::SPIKES, pitch),
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
        Effect::pool(0, Class::BloodMage, b::THORNFALL, at(-4.0, 0.0, 8.0), 100),
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
        b::THORN_LASH,
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
// The Hemoplague, held
// ---------------------------------------------------------------------------

#[test]
fn the_hemoplague_grows_while_held_and_hurts_and_slows() {
    let mut w = mage();
    let start = sim::moves::get(Class::BloodMage, b::THORN_LASH).startup as u32;
    hold(&mut w, keys::THORNS, start + 3);
    let small = effects_of(&w, EffectKind::Plague)
        .first()
        .expect("a cloud")
        .reach;
    let floor = effects_of(&w, EffectKind::Plague)[0].pos;
    let his = w.players[1].health;
    let mut slowed = false;
    for _ in 0..60 {
        w.players[1].pos = floor;
        step_looking(&mut w, keys::THORNS, -2000);
        slowed |= w.players[1].slowed > 0;
    }
    let big = effects_of(&w, EffectKind::Plague)[0].reach;
    assert!(
        big.raw() > small.raw(),
        "it grew: {} to {}",
        as_f(small),
        as_f(big)
    );
    assert!(w.players[1].health < his, "it hurt him");
    assert!(slowed, "and slowed him");
    step(&mut w, 0);
    run(&mut w, 30, 0);
    let left = effects_of(&w, EffectKind::Plague);
    assert_eq!(left.len(), 1, "let go, it stands");
    assert_eq!(left[0].reach, big, "at the size it reached");
}

#[test]
fn the_cloud_is_laid_where_she_aims_not_at_her_feet() {
    let mut w = mage();
    let start = sim::moves::get(Class::BloodMage, b::THORN_LASH).startup as u32;
    hold(&mut w, keys::THORNS, start + 3);
    let cloud = effects_of(&w, EffectKind::Plague)[0].pos;
    let far = cloud.sub(w.players[0].pos).flat_len();
    assert!(
        far.raw() > metres(2.0).raw(),
        "out where she looked: {}",
        as_f(far)
    );
}

// ---------------------------------------------------------------------------
// The Carpet of blood and Coagulate
// ---------------------------------------------------------------------------

/// Hold the right click for `frames`, looking a little down.
fn reap_for(w: &mut World, frames: u32) {
    hold(w, keys::SCYTHE, frames);
}

/// Frames from the press to the field opening: the tap's startup, or the
/// Champion's window (`tuning::hold_from`) if that is longer.
fn field_opens(tap: u8) -> u32 {
    (sim::moves::get(Class::BloodMage, tap).startup as u32).max(sim::tuning::hold_from() as u32)
}

fn sweep_startup() -> u32 {
    field_opens(b::SWEEP)
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
fn holding_the_right_click_reaps_further_the_longer_it_is_held() {
    let lunge = |extra: u32| {
        let mut w = mage();
        w.players[1].pos = at(30.0, 0.0, 30.0);
        let from = w.players[0].pos;
        reap_for(&mut w, sweep_startup() + extra);
        assert_eq!(doing(&w), Some(b::REAP), "the tap became the Reap");
        run(&mut w, 40, 0);
        w.players[0].pos.sub(from).flat_len()
    };
    let (short, long) = (lunge(2), lunge(30));
    assert!(
        long.raw() > short.raw() && short.raw() > metres(0.5).raw(),
        "{} then {}",
        as_f(short),
        as_f(long)
    );
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
fn a_bite_deals_what_it_spends_and_spills_it_and_fills_no_well() {
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
#[test]
fn the_rising_reap_hurls_thralls_and_drinks_the_stains_in_front() {
    let mut w = mage();
    wounded(&mut w);
    let red = w.players[0].health;
    thrall(&mut w, at(-4.5, 0.0, 8.0), 200);
    let mut s = Effect::pool(0, Class::BloodMage, b::THORN_LASH, at(-3.0, 0.0, 8.0), 100);
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

/// Her, with `grey` open.
fn greyed(grey: i32) -> World {
    let mut w = mage();
    w.players[1].pos = at(40.0, 0.0, 8.0);
    w.players[0].wound(grey);
    w
}

#[test]
fn the_hemoplague_held_to_its_end_is_the_vines_and_spends_her_grey() {
    let mut w = greyed(300);
    let start = sim::moves::get(Class::BloodMage, b::THORN_LASH).startup as u32;
    let cap = sim::moves::get(Class::BloodMage, b::BRAMBLE).channel as u32;
    hold(&mut w, keys::THORNS, start + cap + 3);
    assert_eq!(doing(&w), Some(b::VINES));
    assert_eq!(w.players[0].grey, 0, "all her grey went into it");
    run(&mut w, 5, 0);
    let vines = effects_of(&w, EffectKind::Vines);
    assert_eq!(vines.len(), 1, "the vines");
    assert!(
        effects_of(&w, EffectKind::Plague).is_empty(),
        "out of the cloud"
    );
}

#[test]
fn the_vines_bleed_and_slow_and_more_grey_hits_harder() {
    let blow = |grey: i32| {
        let mut w = greyed(grey);
        let start = sim::moves::get(Class::BloodMage, b::THORN_LASH).startup as u32;
        let cap = sim::moves::get(Class::BloodMage, b::BRAMBLE).channel as u32;
        hold(&mut w, keys::THORNS, start + cap + 3);
        run(&mut w, 2, 0);
        let v = effects_of(&w, EffectKind::Vines)[0];
        let his = w.players[1].health;
        let mut bled = false;
        let mut slowed = false;
        for _ in 0..40 {
            w.players[1].pos = v.pos;
            step(&mut w, 0);
            bled |= w.players[1].bleeding > 0;
            slowed |= w.players[1].slowed > 0;
        }
        assert!(bled && slowed, "bled {bled}, slowed {slowed}");
        his - w.players[1].health
    };
    let little = blow(20);
    let lots = blow(400);
    let row = sim::moves::get(Class::BloodMage, b::VINES).damage;
    assert!(little >= row, "always more than its row: {little}");
    assert!(
        lots > little + 100,
        "and more grey, more: {little} then {lots}"
    );
}

/// Her red as the vines, grown from 300 grey, began, what they owed her,
/// and her red after they have stood 40 frames -- with him standing in
/// them, or not.
fn red_after_the_vines(on_him: bool) -> (i32, i32, i32) {
    let mut w = greyed(300);
    let start = sim::moves::get(Class::BloodMage, b::THORN_LASH).startup as u32;
    let cap = sim::moves::get(Class::BloodMage, b::BRAMBLE).channel as u32;
    hold(&mut w, keys::THORNS, start + cap + 3);
    run(&mut w, 2, 0);
    let began = w.players[0].health;
    let owed = w.players[0].owed as i32;
    let v = effects_of(&w, EffectKind::Vines)[0];
    for _ in 0..40 {
        if on_him {
            w.players[1].pos = v.pos;
        }
        step(&mut w, 0);
    }
    (began, owed, w.players[0].health)
}

#[test]
fn a_finisher_that_lands_pays_back_the_grey_it_burned() {
    let (began, owed, landed) = red_after_the_vines(true);
    assert!(owed >= 300, "they owe her all the grey they burned: {owed}");
    assert!(
        landed > began,
        "the vines landed and paid her: {began} to {landed}"
    );
    assert!(
        landed <= began + owed,
        "never more than they burned: {landed}"
    );
    let (began, _, whiffed) = red_after_the_vines(false);
    assert_eq!(whiffed, began, "a finisher that misses pays nothing back");
}

#[test]
fn the_reap_held_to_its_end_is_the_harvest_and_drinks_every_stain_round_her() {
    let mut w = greyed(300);
    w.players[1].pos = at(40.0, 0.0, 8.0);
    for k in 0..3 {
        let mut s = Effect::pool(
            0,
            Class::BloodMage,
            b::THORN_LASH,
            w.players[0].pos.add(at(-2.0 + 2.0 * k as f32, 0.0, 2.5)),
            80,
        );
        s.age = t::stain_settles() + 1;
        place(&mut w, s);
    }
    let cap = sim::moves::get(Class::BloodMage, b::REAP).channel as u32;
    reap_for(&mut w, sweep_startup() + cap + 3);
    assert_eq!(doing(&w), Some(b::HARVEST));
    run(&mut w, 40, 0);
    // What the Reap cost lands at her feet as the Harvest comes out, after it
    // drank: only the three that were there are asked about.
    let left = pools(&w).iter().filter(|p| p.slot == b::THORN_LASH).count();
    assert_eq!(left, 0, "every stain round her was drunk");
}

#[test]
fn more_grey_is_a_harder_harvest() {
    let blow = |grey: i32| {
        let mut w = greyed(grey);
        w.players[1].pos = w.players[0].pos.add(at(2.5, 0.0, 0.0));
        let his = w.players[1].health;
        let cap = sim::moves::get(Class::BloodMage, b::REAP).channel as u32;
        for _ in 0..(sweep_startup() + cap + 3) {
            w.players[1].pos = w.players[0].pos.add(at(2.5, 0.0, 0.0));
            step_looking(&mut w, keys::SCYTHE, -2000);
        }
        for _ in 0..40 {
            w.players[1].pos = w.players[0].pos.add(at(2.5, 0.0, 0.0));
            step(&mut w, 0);
        }
        his - w.players[1].health
    };
    let (little, lot) = (blow(50), blow(400));
    assert!(lot > little && little > 0, "{little} then {lot}");
}

#[test]
fn the_spike_held_to_its_end_is_the_cascade_going_out() {
    let mut w = greyed(300);
    w.players[1].pos = at(40.0, 0.0, 8.0);
    let cap = sim::moves::get(Class::BloodMage, b::BLACK_SPIKE).channel as u32;
    hold(&mut w, keys::SPIKES, cap + 3);
    assert_eq!(doing(&w), Some(b::CASCADE));
    // The impact spills pools of her spent grey round it...
    let mut spilled = 0;
    for _ in 0..60 {
        step(&mut w, 0);
        spilled = spilled.max(pools(&w).iter().filter(|p| p.slot == b::CASCADE).count());
        if spilled > 0 {
            break;
        }
    }
    assert!(
        spilled as i32 >= t::cascade_first(),
        "the impact spilled {spilled} pools"
    );
    // ... which go up a moment later and spill more, further out.
    let mut eruptions = 0;
    let mut furthest = Fx::ZERO;
    let start = pools(&w)
        .iter()
        .find(|p| p.slot == b::CASCADE)
        .map(|p| p.pos)
        .expect("a pool");
    for _ in 0..(t::cascade_fuse() as u32 * (t::cascade_generations() as u32 + 1)) {
        step(&mut w, 0);
        eruptions += effects_of(&w, EffectKind::BlackSpike)
            .iter()
            .filter(|e| e.erupted() && e.age == 1)
            .count();
        for p in pools(&w).iter().filter(|p| p.slot == b::CASCADE) {
            let far = p.pos.sub(start).flat_len();
            if far.raw() > furthest.raw() {
                furthest = far;
            }
        }
    }
    assert!(
        eruptions as i32 > t::cascade_first(),
        "the pools went up and spilled more: {eruptions} eruptions"
    );
    assert!(
        furthest.raw() > t::cascade_step().raw(),
        "and spread: {}",
        as_f(furthest)
    );
}

/// The cascade's pools, summed over its whole life, from `grey` spent.
fn cascade_blood(grey: i32) -> i32 {
    let mut w = greyed(grey);
    w.players[1].pos = at(40.0, 0.0, 8.0);
    let cap = sim::moves::get(Class::BloodMage, b::BLACK_SPIKE).channel as u32;
    hold(&mut w, keys::SPIKES, cap + 3);
    let mut seen = 0;
    for _ in 0..120 {
        step(&mut w, 0);
        seen = seen.max(
            pools(&w)
                .iter()
                .filter(|p| p.slot == b::CASCADE)
                .map(|p| p.banked)
                .sum::<i32>(),
        );
    }
    seen
}

#[test]
fn the_cascade_pays_back_what_it_burned_as_it_lands() {
    let mut w = greyed(300);
    let cap = sim::moves::get(Class::BloodMage, b::BLACK_SPIKE).channel as u32;
    hold(&mut w, keys::SPIKES, cap + 3);
    let began = w.players[0].health;
    let owed = w.players[0].owed as i32;
    assert!(owed >= 300, "owed: {owed}");
    // Where the impact comes up, he stands, and stays.
    let mut him = None;
    for _ in 0..120 {
        if him.is_none() {
            him = effects_of(&w, EffectKind::BlackSpike)
                .first()
                .map(|e| e.pos);
        }
        if let Some(spot) = him {
            w.players[1].pos = spot;
        }
        step(&mut w, 0);
    }
    let red = w.players[0].health;
    assert!(
        red > began,
        "the cascade landed and paid her: {began} to {red}"
    );
    assert!(red <= began + owed, "never more than it burned: {red}");
}

#[test]
fn more_grey_is_a_bigger_cascade() {
    let little = cascade_blood(60);
    let lots = cascade_blood(500);
    assert!(lots > little * 2, "{little} then {lots}");
}

#[test]
fn the_cascade_reaches_for_her_stains_however_small() {
    // A bleed's trail: small stains off to one side, well past the first
    // pools. Small, but not so small they drain before it gets there. The cascade goes for them and sets them off.
    let mut w = greyed(300);
    w.players[1].pos = at(40.0, 0.0, 8.0);
    let cap = sim::moves::get(Class::BloodMage, b::BLACK_SPIKE).channel as u32;
    // Where the impact will land: aim as the hold will.
    let mut probe = w.clone();
    hold(&mut probe, keys::SPIKES, cap + 3);
    run(&mut probe, 22, 0);
    let centre = pools(&probe)
        .iter()
        .filter(|p| p.slot == b::CASCADE)
        .fold(V3::ZERO, |sum, p| sum.add(p.pos))
        .scale(Fx::ratio(1, t::cascade_first().max(1)));
    let trail: Vec<V3> = (0..3)
        .map(|k| centre.add(at(0.0, 0.0, 6.0 + 1.5 * k as f32)))
        .collect();
    for &spot in &trail {
        stain(&mut w, spot, 25);
    }
    for e in w.effects.iter_mut().flatten() {
        e.age = t::stain_settles() + 1;
    }
    hold(&mut w, keys::SPIKES, cap + 3);
    let mut gone = [false; 3];
    for _ in 0..(t::cascade_fuse() as u32 * (t::cascade_generations() as u32 + 2)) {
        step(&mut w, 0);
        for (k, &spot) in trail.iter().enumerate() {
            if effects_of(&w, EffectKind::BlackSpike)
                .iter()
                .any(|e| e.erupted() && e.pos.sub(spot).flat_len().raw() < metres(0.5).raw())
            {
                gone[k] = true;
            }
        }
    }
    assert!(gone.iter().all(|g| *g), "the trail went up: {gone:?}");
}

#[test]
fn a_held_spike_lands_on_her_stains_first() {
    let mut w = mage();
    w.players[1].pos = at(40.0, 0.0, 8.0);
    // Where the middle spike lands.
    let mut probe = w.clone();
    let cap = sim::moves::get(Class::BloodMage, b::BLACK_SPIKE).channel as u32;
    hold(&mut probe, keys::SPIKES, cap - 2);
    run(&mut probe, 22, 0);
    let spikes = effects_of(&probe, EffectKind::BlackSpike);
    let mean = spikes
        .iter()
        .fold(V3::ZERO, |sum, e| sum.add(e.pos))
        .scale(Fx::ratio(1, spikes.len().max(1) as i32));
    let centre = spikes
        .iter()
        .min_by_key(|e| e.pos.sub(mean).flat_len().raw())
        .expect("spikes")
        .pos;
    // Two small stains inside the ring, off its points.
    let a = centre.add(at(1.3, 0.0, 0.7));
    let c = centre.add(at(-0.9, 0.0, -1.6));
    stain(&mut w, a, 25);
    stain(&mut w, c, 25);
    for e in w.effects.iter_mut().flatten() {
        e.age = t::stain_settles() + 1;
    }
    hold(&mut w, keys::SPIKES, cap - 2);
    let mut hit = [false; 2];
    for _ in 0..40 {
        step(&mut w, 0);
        for (k, spot) in [a, c].into_iter().enumerate() {
            if effects_of(&w, EffectKind::BlackSpike)
                .iter()
                .any(|e| e.erupted() && e.pos.sub(spot).flat_len().raw() < metres(0.3).raw())
            {
                hit[k] = true;
            }
        }
    }
    assert!(hit.iter().all(|h| *h), "the swarm found both: {hit:?}");
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
        b::BRAMBLE,
        at(4.0, 0.0, 8.0),
        V3::ZERO,
        sim::moves::get(Class::BloodMage, b::BRAMBLE).radius,
    );
    place(&mut w, cloud);
    let mut stunned = false;
    for _ in 0..(t::plague_tick() as u32 * 4 + 2) {
        w.players[1].pos = at(4.0, 0.0, 8.0);
        step(&mut w, 0);
        stunned |= !w.players[1].action.actionable();
    }
    let took = w.players[1].full_health() - w.players[1].health;
    let tick = sim::moves::get(Class::BloodMage, b::BRAMBLE).damage;
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
