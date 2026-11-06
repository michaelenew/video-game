//! The Dual mage's spellbook (2026-10-12): what a player would notice if any
//! of it broke.
//!
//! `docs/design/exploration/0015_dual_mage_spellbook.md` is the design. The
//! click is what a spell is cast on -- left is dark, on *them*: curses; right
//! is light, on *the ground*: sigils; middle is twilight, on *herself* -- and
//! the row is the verb: on the floor she fights, in the air she moves, leaving
//! the floor she defends. A dark spell lays curse stacks; a light one purges
//! them, every stack at once, each worth more than the last.
//!
//! **The second pass of the same day** (`exploration/0016`): the bare clicks
//! are her wing autos again, a great deal bigger, on the floor and in the air,
//! and each hops her the way she is holding; the Devour and the Smite are on
//! `F` and `R`.

use sim::class::{Class, Force, Mechanic};
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

/// The pitch that puts her crosshair on `point`.
fn onto(w: &World, point: V3) -> i16 {
    sim::aim::look_onto_closely(
        w.players[0].pos,
        LOOK_RIGHT,
        w.players[0].aloft,
        w.players[0].clear,
        point,
    )
}

/// The pitch that puts her crosshair on the middle of the other fighter.
fn onto_them(w: &World) -> i16 {
    let them = w.players[1].pos;
    onto(
        w,
        V3::new(
            them.x,
            them.y.add(t::body_height().div(Fx::from_int(2))),
            them.z,
        ),
    )
}

/// The pitch that puts her crosshair on the other fighter's feet: where a
/// spell cast on the ground has to be pointed to land under them.
fn onto_their_feet(w: &World) -> i16 {
    onto(w, w.players[1].pos)
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

fn run_looking(w: &mut World, frames: u32, pitch: i16) {
    for _ in 0..frames {
        step_looking(w, 0, pitch);
    }
}

/// Press `button` for a frame -- and, for a bare click, let it go the next,
/// which makes it a tap rather than its long hold (2026-10-13).
fn tap(w: &mut World, button: u16) {
    step(w, button);
    if matches!(button, keys::DARK | keys::TWILIGHT | keys::LIGHT) {
        step(w, 0);
    }
}

/// Run until she can act again, or `most` frames.
fn until_free(w: &mut World, most: u32) {
    for _ in 0..most {
        if w.players[0].action.actionable() {
            return;
        }
        step(w, 0);
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

fn effects(w: &World, kind: EffectKind) -> Vec<sim::effects::Effect> {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.kind == kind)
        .copied()
        .collect()
}

fn bars(w: &World) -> (i32, i32) {
    let (dark, light) = sim::dual::bars(&w.players[0]).expect("her bars");
    (dark.to_int(), light.to_int())
}

/// Her bars at `dark` and `light`, carrying dark, so a test starts from a
/// known place on the curve.
fn set_bars(w: &mut World, dark: i32, light: i32) {
    w.players[0].mechanic = Mechanic::Meter {
        dark: Fx::from_int(dark),
        light: Fx::from_int(light),
        colour: Force::Dark,
        ascending: 0,
        jumped: false,
        hopped: false,
        dash_jump: false,
        wave: Fx::ZERO,
        springs: 0,
        storm: 0,
        storm_at: sim::math::V3::ZERO,
        strike: 0,
        strike_at: sim::math::V3::ZERO,
        singe: Fx::ZERO,
        veil: 0,
        warded: false,
        primed: 0,
        chained: 0,
        chain_out: 0,
    };
}

fn level(w: &mut World, at: i32) {
    set_bars(w, at, at);
}

/// The other fighter carrying `n` stacks of her curse.
fn cursed(w: &mut World, n: u8) {
    w.players[1].curse = sim::dual::curse_of(n, 0);
    w.players[1].curse_clock = t::curse_fades();
}

fn stacks(w: &World) -> u8 {
    sim::dual::stacks(w.players[1].curse)
}

// ---------------------------------------------------------------------------
// The map
// ---------------------------------------------------------------------------

#[test]
fn on_the_floor_the_clicks_are_the_autos_and_the_chain() {
    for (button, wanted) in [
        (keys::DARK, d::DARK_AUTO),
        (keys::TWILIGHT, d::CHAIN),
        (keys::LIGHT, d::LIGHT_AUTO),
        (keys::DARK_MAJOR, d::ABYSS),
        (keys::LIGHT_MAJOR, d::JUDGEMENT),
    ] {
        let mut w = alone();
        set_bars(&mut w, 100, 100);
        tap(&mut w, button);
        assert_eq!(doing(&w), Some(wanted), "button {button:#x} on the floor");
    }
}

#[test]
fn in_the_air_the_clicks_are_the_same() {
    for (button, wanted) in [
        (keys::DARK, d::DARK_AUTO),
        (keys::TWILIGHT, d::CHAIN),
        (keys::LIGHT, d::LIGHT_AUTO),
        (keys::DARK_MAJOR, d::ABYSS),
        (keys::LIGHT_MAJOR, d::JUDGEMENT),
    ] {
        let mut w = alone();
        set_bars(&mut w, 100, 100);
        aloft(&mut w, 4.0);
        tap(&mut w, button);
        assert_eq!(doing(&w), Some(wanted), "button {button:#x} in the air");
    }
}

// ---------------------------------------------------------------------------
// The autos: big wings, and a hop after each
// ---------------------------------------------------------------------------

const W: u16 = Input::W;
const D: u16 = Input::D;

/// Did an auto thrown at somebody `gap` metres ahead land?
fn auto_lands(button: u16, gap: f32) -> bool {
    let mut w = facing(gap);
    let before = w.players[1].health;
    step(&mut w, button);
    run(&mut w, 20, 0);
    w.players[1].health < before
}

#[test]
fn the_autos_reach_a_body_close_and_a_body_six_and_a_half_metres_off() {
    for button in [keys::DARK, keys::LIGHT] {
        for gap in [1.0, 2.5, 4.0, 5.5, 6.5] {
            assert!(auto_lands(button, gap), "{button:#x} at {gap} m");
        }
        assert!(!auto_lands(button, 8.0), "{button:#x} reaches eight metres");
    }
}

/// Where an auto thrown with `held` down leaves her, and how high she got.
fn after_an_auto(held: u16, lower: i32, height: f32) -> (V3, Fx) {
    let mut w = alone();
    level(&mut w, lower);
    if height > 0.0 {
        aloft(&mut w, height);
    }
    step(&mut w, keys::DARK | held);
    let mut top = w.players[0].pos.y;
    for _ in 0..40 {
        step(&mut w, held);
        top = top.max(w.players[0].pos.y);
    }
    (w.players[0].pos, top)
}

#[test]
fn an_auto_with_a_direction_held_hops_her_that_way() {
    let from = alone().players[0].pos;
    let (still, _) = after_an_auto(0, 0, 0.0);
    let (forward, _) = after_an_auto(W, 0, 0.0);
    let (aside, _) = after_an_auto(D, 0, 0.0);
    assert!(
        still.sub(from).flat_len().raw() < metres(0.5).raw(),
        "holding nothing, she stays: {} m",
        as_f(still.sub(from).flat_len())
    );
    // Walking alone for the frames of the auto covers a metre or so; the hop
    // is on top of it.
    assert!(
        forward.x.sub(from.x).raw() > t::auto_hop().raw(),
        "forward: {} m",
        as_f(forward.x.sub(from.x))
    );
    assert!(
        aside.sub(from).flat_len().raw() > t::auto_hop().raw(),
        "to the side: {} m",
        as_f(aside.sub(from).flat_len())
    );
}

#[test]
fn a_hop_cuts_the_recovery_short() {
    let free_after = |held: u16| {
        let mut w = alone();
        step(&mut w, keys::DARK | held);
        for f in 1..80 {
            step(&mut w, held);
            if w.players[0].action.actionable() {
                return f;
            }
        }
        80
    };
    let m = sim::moves::get(Class::DualMage, d::DARK_AUTO);
    assert!(free_after(0) >= (m.startup + m.active + m.recovery) as u32 - 1);
    assert!(
        free_after(W) < free_after(0) + 6,
        "the hop is the recovery, not on top of it: {} against {}",
        free_after(W),
        free_after(0)
    );
}

#[test]
fn a_hop_does_not_go_through_a_wall() {
    // The proving ground's low wall is eight metres behind her: hop
    // backwards at it from the floor, with her lower bar empty so the hop
    // cannot lift her over it, and she stays this side of it.
    let mut w = alone();
    let s = Input::S;
    for _ in 0..6 {
        level(&mut w, 0);
        step(&mut w, keys::DARK | s);
        run(&mut w, 40, s);
    }
    assert!(
        w.terrain().inside(w.players[0].pos),
        "she hopped out of the arena to {:?}",
        w.players[0].pos
    );
}

// ---------------------------------------------------------------------------
// The curse
// ---------------------------------------------------------------------------

#[test]
fn the_grasp_roots_whoever_it_closes_on() {
    // No button since the wing autos came back: thrown as the move it is.
    let mut w = facing(3.0);
    let feet = onto_their_feet(&w);
    w.press(0, d::GRASPING_SHADE, Input::looking_at(0, LOOK_RIGHT, feet));
    assert_eq!(doing(&w), Some(d::GRASPING_SHADE));
    let mut rooted = false;
    for _ in 0..30 {
        step_looking(&mut w, 0, feet);
        let q = w.players[1];
        if q.slowed > 0 && q.slow_mul.raw() <= t::root_slow().raw() {
            rooted = true;
        }
    }
    assert!(rooted, "rooted where they stand");
}

#[test]
fn a_curse_ticks_and_drains_to_her() {
    let mut w = facing(8.0);
    // From a full dark bar, so a tick is big enough that her share of it is
    // a whole point of health.
    set_bars(&mut w, 100, 90);
    cursed(&mut w, 5);
    w.players[0].health -= 300;
    let (theirs, mine) = (w.players[1].health, w.players[0].health);
    run(&mut w, t::curse_every() as u32 * 3, 0);
    assert!(w.players[1].health < theirs, "it ticked on them");
    assert!(w.players[0].health > mine, "and she drank it");
}

#[test]
fn a_curse_fades_a_stack_at_a_time() {
    let mut w = facing(8.0);
    cursed(&mut w, 3);
    run(&mut w, t::curse_fades() as u32 + 1, 0);
    assert_eq!(stacks(&w), 2, "one stack went");
    run(&mut w, t::curse_fades() as u32, 0);
    assert_eq!(stacks(&w), 1, "and another");
    run(&mut w, t::curse_fades() as u32, 0);
    assert_eq!(stacks(&w), 0, "and the last");
}

#[test]
fn the_curse_holds_five_at_most() {
    let mut w = facing(2.0);
    cursed(&mut w, 5);
    step(&mut w, keys::DARK);
    until_free(&mut w, 120);
    assert_eq!(stacks(&w), t::curse_most(), "capped");
}

// ---------------------------------------------------------------------------
// The purge
// ---------------------------------------------------------------------------

/// What the light auto deals a fighter carrying `n` stacks, from level.
fn light_wing_on(n: u8) -> i32 {
    let mut w = facing(2.5);
    level(&mut w, 50);
    cursed(&mut w, n);
    let before = w.players[1].health;
    step(&mut w, keys::LIGHT);
    let m = sim::moves::get(Class::DualMage, d::LIGHT_AUTO);
    run(&mut w, (m.startup + m.active) as u32 + 2, 0);
    before - w.players[1].health
}

#[test]
fn light_purges_every_stack_at_once() {
    let mut w = facing(2.5);
    cursed(&mut w, 3);
    step(&mut w, keys::LIGHT);
    until_free(&mut w, 120);
    assert_eq!(stacks(&w), 0, "the curse is spent");
    assert!(
        light_wing_on(3) > light_wing_on(0) + 40,
        "and it was worth something"
    );
}

#[test]
fn every_stack_is_worth_more_than_the_last() {
    let dealt: Vec<i32> = (0..=5).map(light_wing_on).collect();
    for n in 2..=5 {
        let step = dealt[n] - dealt[n - 1];
        let before = dealt[n - 1] - dealt[n - 2];
        assert!(
            step >= before,
            "the {n}th stack is worth {step}, the one before it {before}: {dealt:?}"
        );
    }
    assert!(
        dealt[5] - dealt[0] >= 100,
        "five stacks is the big number: {dealt:?}"
    );
}

#[test]
fn a_big_purge_staggers() {
    let mut w = facing(2.5);
    cursed(&mut w, t::purge_stagger_from());
    step(&mut w, keys::LIGHT);
    let mut staggered = false;
    for _ in 0..20 {
        step(&mut w, 0);
        staggered |= matches!(w.players[1].action, Action::Stagger { .. });
    }
    assert!(
        staggered,
        "staggered from {} stacks",
        t::purge_stagger_from()
    );
}

// ---------------------------------------------------------------------------
// Sigils
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// The twin stars
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Moving: the air row
// ---------------------------------------------------------------------------

#[test]
fn descent_dives_where_she_points_and_writes_a_sigil() {
    let mut w = alone();
    aloft(&mut w, 5.0);
    let down = (-70 * 65536 / 360) as i16;
    w.press(0, d::DESCENT, Input::looking_at(0, LOOK_RIGHT, down));
    assert_eq!(doing(&w), Some(d::DESCENT));
    for _ in 0..60 {
        step_looking(&mut w, 0, down);
        if w.players[0].grounded {
            break;
        }
    }
    assert!(w.players[0].grounded, "down");
    run(&mut w, 3, 0);
    let sigils = effects(&w, EffectKind::Sigil);
    assert_eq!(sigils.len(), 1, "a sigil where she landed");
    assert!(
        sigils[0].pos.sub(w.players[0].pos).flat_len().raw() < metres(0.5).raw(),
        "under her"
    );
}

// ---------------------------------------------------------------------------
// Defending: the takeoffs
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// The majors
// ---------------------------------------------------------------------------

fn abyss_on_them(w: &mut World) {
    // At her own feet since 2026-11-06, and a full bar to vent: so beside them.
    let (dark, light) = bars(w);
    set_bars(w, 100, light.max(dark.min(100)));
    w.players[0].pos = w.players[1]
        .pos
        .sub(V3::new(metres(1.0), Fx::ZERO, Fx::ZERO));
    let feet = onto_their_feet(w);
    step_looking(w, keys::DARK_MAJOR, feet);
    assert_eq!(doing(w), Some(d::ABYSS));
}

#[test]
fn the_abyss_pins_and_curses_everything_under_it() {
    let mut w = facing(5.0);
    level(&mut w, 60);
    let them = w.players[1].pos;
    abyss_on_them(&mut w);
    let m = sim::moves::get(Class::DualMage, d::ABYSS);
    let mut pinned = false;
    for _ in 0..(m.startup + m.active) as u32 + t::abyss_life() as u32 {
        step(&mut w, 0);
        let q = w.players[1];
        pinned |= q.slowed > 0 && q.slow_mul.raw() <= t::root_slow().raw();
    }
    assert!(pinned, "pinned");
    assert!(stacks(&w) >= 3, "cursed again and again: {}", stacks(&w));
    assert!(
        w.players[1].pos.sub(them).flat_len().raw() < metres(1.0).raw(),
        "and dragged nowhere"
    );
}

#[test]
fn the_black_sun_then_the_columns() {
    let mut w = facing(5.0);
    level(&mut w, 60);
    abyss_on_them(&mut w);
    let m = sim::moves::get(Class::DualMage, d::ABYSS);
    run(&mut w, m.startup as u32 + t::abyss_life() as u32, 0);
    let built = stacks(&w);
    assert!(built >= 2, "fixture: the black sun cursed them, {built}");
    until_free(&mut w, 60);
    // Back in beside them with a full light bar: Judgement comes down where
    // she stands (2026-11-06) -- the explosion takes going in twice.
    set_bars(&mut w, 0, 100);
    w.players[0].pos = w.players[1]
        .pos
        .sub(V3::new(metres(1.0), Fx::ZERO, Fx::ZERO));
    let before = w.players[1].health;
    let feet = onto_their_feet(&w);
    step_looking(&mut w, keys::LIGHT_MAJOR, feet);
    let j = sim::moves::get(Class::DualMage, d::JUDGEMENT);
    run_looking(&mut w, (j.startup + j.active) as u32 + 2, feet);
    assert_eq!(stacks(&w), 0, "Judgement purged it");
    assert!(
        before - w.players[1].health > j.damage,
        "for more than its own blow"
    );
}

// ---------------------------------------------------------------------------
// Ascending at the right moment
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Creatures
// ---------------------------------------------------------------------------

#[test]
fn a_creature_is_cursed_and_purged_like_a_fighter() {
    let mut w = World::hunt([Class::DualMage, Class::Bulwark]);
    // On the open floor: the proving ground's Ridgeback starts on the
    // platform its spawn is astride (2026-10-21, `sim::climb`), and this is
    // about the curse, not about a column aimed at a ledge.
    if let Some(b) = w.monsters[0].as_mut() {
        b.pos = V3::new(Fx::from_int(2), Fx::ZERO, Fx::from_int(8));
    }
    let beast = w.monsters[0].expect("a creature");
    let to = beast.pos.sub(w.players[0].pos);
    let flat = V3::new(to.x, Fx::ZERO, to.z);
    let dir = flat.normalized();
    w.players[0].pos = beast.pos.sub(dir.scale(Fx::from_int(5)));
    w.players[0].pos.y = w.terrain().ground_under(w.players[0].pos);
    w.players[1].pos = beast.pos.sub(dir.scale(Fx::from_int(20)));
    let yaw = sim::math::atan2_turns(dir.z, dir.x);
    let aim = (yaw.raw() as u32 & 0xffff) as u16;
    let middle = V3::new(beast.pos.x, beast.pos.y.add(Fx::from_int(1)), beast.pos.z);
    let pitch = sim::aim::look_onto_closely(
        w.players[0].pos,
        aim,
        w.players[0].aloft,
        w.players[0].clear,
        middle,
    );
    let _ = middle;
    if let Some(b) = w.monsters[0].as_mut() {
        b.curse = sim::dual::curse_of(4, 0);
        b.curse_clock = t::curse_fades();
    }
    // Judgement on the ground under a foreleg: its column reaches the animal
    // whatever part is over that spot, and it is light. (Under its middle was
    // under its belly, which the column only reached while the platform it
    // stood inside raised the column's foot.)
    let knee = beast.rig().bone[sim::species::ridgeback::bones::FOREARM_L].at;
    let ground = V3::new(knee.x, w.terrain().ground_under(knee), knee.z);
    // Judgement comes down where she stands (2026-11-06): beside the
    // foreleg, with a full light bar to vent.
    w.players[0].pos = ground.sub(dir.scale(Fx::ONE));
    set_bars(&mut w, 0, 100);
    let at = ground.sub(w.players[0].pos);
    let aim = (sim::math::atan2_turns(at.z, at.x).raw() as u32 & 0xffff) as u16;
    let down = sim::aim::look_onto_closely(
        w.players[0].pos,
        aim,
        w.players[0].aloft,
        w.players[0].clear,
        ground,
    );
    let _ = pitch;
    let before = w.monsters[0].map_or(0, |b| b.health);
    w.advance([
        Input::looking_at(keys::LIGHT_MAJOR, aim, down),
        Input::default(),
    ]);
    let j = sim::moves::get(Class::DualMage, d::JUDGEMENT);
    for _ in 0..(j.startup + j.active) as u32 + 2 {
        w.advance([Input::looking_at(0, aim, down), Input::default()]);
    }
    let beast = w.monsters[0].expect("still there");
    assert_eq!(sim::dual::stacks(beast.curse), 0, "purged");
    assert!(
        before - beast.health > j.damage,
        "for more than the column's own blow: {}",
        before - beast.health
    );
}

#[test]
fn a_creatures_curse_ticks() {
    let mut w = World::hunt([Class::DualMage, Class::Bulwark]);
    set_bars(&mut w, 100, 90);
    if let Some(b) = w.monsters[0].as_mut() {
        b.curse = sim::dual::curse_of(5, 0);
        b.curse_clock = t::curse_fades();
    }
    w.players[0].health -= 200;
    let mine = w.players[0].health;
    let before = w.monsters[0].map_or(0, |b| b.health);
    run(&mut w, t::curse_every() as u32 * 2, 0);
    let after = w.monsters[0].map_or(0, |b| b.health);
    assert!(after < before, "it ticked on the animal");
    assert!(w.players[0].health > mine, "and she drank it");
}

// ---------------------------------------------------------------------------
// The long holds, and the second pass's second day (2026-10-12)
// ---------------------------------------------------------------------------

/// Hold `button` with the crosshair at `pitch` until `wanted` is out of its
/// hold -- in its startup or past it -- or `most` frames go by. True if it
/// came.
fn hold_into(w: &mut World, button: u16, pitch: i16, wanted: u8, most: u32) -> bool {
    // A big hold costs a third of its bar (2026-11-06): topped up here if it
    // could not pay, so a test about the spell is not a test of the bar.
    if !sim::dual::affords(&w.players[0], wanted) {
        let (dark, light) = bars(w);
        set_bars(w, dark.max(100), light.max(100));
    }
    for _ in 0..most {
        step_looking(w, button, pitch);
        let out = matches!(
            w.players[0].action,
            Action::Startup { kind, .. } | Action::Active { kind, .. } if kind == wanted
        );
        if out {
            return true;
        }
    }
    false
}

#[test]
fn holding_a_click_past_its_wing_is_that_hand_s_long_hold() {
    for (button, wanted) in [(keys::DARK, d::STORM), (keys::LIGHT, d::LANCE)] {
        let mut w = alone();
        assert!(
            hold_into(&mut w, button, 0, wanted, 60),
            "holding {button:#x} never threw it"
        );
    }
    let mut w = alone();
    assert!(
        hold_into(&mut w, keys::TWILIGHT, 0, d::FLURRY_DASH, 60),
        "holding the middle click never threw the Flurry dash"
    );
}

#[test]
fn a_click_is_a_tap_or_a_hold_and_never_both() {
    // **Either / or** (2026-10-13, from the owner): while the click is held
    // and the hold is not full, nothing has begun -- no wing, no cloud. Let
    // go and it is the wing, from that frame; hold it full and it is the
    // spell, and the wing never was.
    let cap = sim::moves::get(Class::DualMage, d::STORM).channel as u32;
    for (button, tap, hold) in [
        (keys::DARK, d::DARK_AUTO, d::STORM),
        (keys::LIGHT, d::LIGHT_AUTO, d::LANCE),
    ] {
        // Let go part of the way: the tap, and never the spell.
        let mut w = alone();
        level(&mut w, 50);
        for _ in 0..cap / 2 {
            step(&mut w, button);
            assert!(
                matches!(w.players[0].action, Action::Channel { .. }),
                "{button:#x}: something began before it was decided: {:?}",
                w.players[0].action
            );
            assert!(sim::state::hitbox(&w.players[0]).is_none());
        }
        step(&mut w, 0);
        assert_eq!(doing(&w), Some(tap), "{button:#x} let go early");
        for _ in 0..60 {
            step(&mut w, 0);
            assert_ne!(doing(&w), Some(hold), "{button:#x}: the spell came too");
        }
        // Held full: the spell, and the wing never.
        let mut w = alone();
        level(&mut w, 50);
        for _ in 0..cap + 20 {
            step(&mut w, button);
            assert_ne!(
                doing(&w),
                Some(tap),
                "{button:#x}: the wing came out of a hold"
            );
        }
    }
}

#[test]
fn the_auto_is_a_ball_from_behind_the_shoulder_bowing_out_and_finishing_dead_ahead() {
    // Facing +X: forward is +x, and the punching side is wherever
    // `aim::across` says; out is measured along it.
    let w = alone();
    let p = &w.players[0];
    let chest = sim::aim::origin(p.pos);
    for kind in [d::DARK_AUTO, d::LIGHT_AUTO] {
        let m = sim::moves::get(Class::DualMage, kind);
        let side = sim::aim::across(p.facing, m.hand);
        let at = |t: f32| {
            let v =
                sim::moves::orb_at(p.pos, p.facing, (p.facing, Fx::ZERO), &m, metres(t)).sub(chest);
            (as_f(v.x), as_f(v.dot(side)))
        };
        let (back, out0) = at(0.0);
        assert!(back < -0.2, "it starts behind her: {back}");
        assert!(
            out0 > as_f(t::hand_offset()) + 0.2,
            "outside the shoulder: {out0}"
        );
        let widest = (1..10).map(|k| at(k as f32 / 10.0).1).fold(0.0, f32::max);
        assert!(widest > out0 + 0.2, "it bows out: {widest} from {out0}");
        assert!(
            widest < out0 + as_f(t::orb_bows_out()) + 0.05,
            "by no more than it is tuned to: {widest}"
        );
        let (ahead, out1) = at(1.0);
        assert!(
            (ahead - as_f(m.reach)).abs() < 0.05,
            "it finishes at {ahead}"
        );
        assert!(out1.abs() < 0.05, "dead ahead, not {out1} to the side");
        let forward: Vec<f32> = (0..=10).map(|k| at(k as f32 / 10.0).0).collect();
        assert!(
            forward.windows(2).all(|w| w[1] > w[0]),
            "always going forward"
        );
    }
}

#[test]
fn the_sweet_spot_is_at_full_reach() {
    let reach = as_f(sim::moves::get(Class::DualMage, d::DARK_AUTO).reach);
    // On a Champion, who has no shield up to take it on.
    let dealt = |gap: f32| {
        let mut w = World::with_classes([Class::DualMage, Class::Champion]);
        w.players[0].pos = at(-6.0, 0.0, 8.0);
        w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
        w.players[1].pos = at(-6.0 + gap, 0.0, 8.0);
        let before = w.players[1].health;
        tap(&mut w, keys::DARK);
        run(&mut w, 20, 0);
        before - w.players[1].health
    };
    let sweet = dealt(reach);
    let on_the_way = dealt(reach / 2.0);
    assert!(on_the_way > 0, "the ball hit nothing on its way");
    assert!(
        sweet > on_the_way + on_the_way / 3,
        "at full reach {sweet}, half way {on_the_way}"
    );
}

// ---------------------------------------------------------------------------
// Her magic is aimed at her (2026-11-04)
// ---------------------------------------------------------------------------
//
// `docs/design/exploration/0037_dual_mage_the_razor_edge.md`: every strike
// comes from behind her, through where she stands, at what she is looking
// at, and she gets out of its way so it lands on them.

#[test]
fn the_lance_comes_from_behind_her_through_where_she_stood() {
    let mut w = facing(10.0);
    let pitch = onto_them(&w);
    let before = w.players[1].health;
    let stood = w.players[0].pos;
    assert!(hold_into(&mut w, keys::LIGHT, pitch, d::LANCE, 60));
    let m = sim::moves::get(Class::DualMage, d::LANCE);
    let mut seen = false;
    for _ in 0..m.startup + 2 {
        step_looking(&mut w, 0, pitch);
        if let Some(b) = sim::state::hitbox(&w.players[0]) {
            // Behind her (she faces +X) and over her middle: over her
            // shoulder, narrowly missing her.
            assert!(
                b.from.x.raw() < stood.x.sub(metres(1.5)).raw(),
                "the lance leaves from {} m, in front of her back",
                as_f(b.from.x)
            );
            assert!(
                b.from.y.raw() > metres(1.2).raw(),
                "from under her shoulder"
            );
            seen = true;
            break;
        }
    }
    assert!(seen, "the lance never struck");
    run_looking(&mut w, 30, pitch);
    assert!(w.players[1].health < before, "ten metres off, untouched");
}

#[test]
fn she_lunges_clear_of_her_own_lance() {
    // Holding nothing she goes back from what she aimed at, rising; holding a
    // direction she goes that way.
    let lunge = |held: u16| -> V3 {
        let mut w = facing(10.0);
        let pitch = onto_them(&w);
        let stood = w.players[0].pos;
        assert!(hold_into(&mut w, keys::LIGHT, pitch, d::LANCE, 60));
        let m = sim::moves::get(Class::DualMage, d::LANCE);
        let mut top = Fx::ZERO;
        for _ in 0..m.startup + 30 {
            step_looking(&mut w, held, pitch);
            top = top.max(w.players[0].pos.y);
        }
        assert!(top.raw() > metres(0.5).raw(), "she never left the floor");
        w.players[0].pos.sub(stood)
    };
    let back = lunge(0);
    assert!(
        back.x.raw() < metres(-1.0).raw(),
        "holding nothing: {} m",
        as_f(back.x)
    );
    // `D` is to her right, which facing +X is +Z... or -Z: either way, sideways.
    let side = lunge(Input::D);
    assert!(
        side.z.abs().raw() > metres(1.0).raw(),
        "holding D: {:?}",
        side
    );
    assert!(
        side.z.abs().raw() > side.x.abs().raw(),
        "holding D went {:?}",
        side
    );
}

#[test]
fn both_clicks_at_once_annihilate_where_she_stood_as_she_springs_clear() {
    let mut w = facing(2.0);
    set_bars(&mut w, 50, 50);
    let (mine, theirs) = (w.players[0].health, w.players[1].health);
    step(&mut w, keys::DARK | keys::LIGHT);
    assert_eq!(doing(&w), Some(d::ANNIHILATION));
    let mut top = Fx::ZERO;
    for _ in 0..40 {
        step(&mut w, 0);
        top = top.max(w.players[0].pos.y);
    }
    assert!(
        top.raw() > metres(3.0).raw(),
        "the spring rose {} m",
        as_f(top)
    );
    assert!(
        w.players[1].health < theirs,
        "whoever was beside her was untouched"
    );
    assert_eq!(w.players[0].health, mine, "her own annihilation hurt her");
    // Both beings were goaded: it fills both bars alike, so it never widens
    // the gap.
    let (dark, light) = bars(&w);
    assert!(
        dark > 50 && dark == light,
        "the bars went to {dark} and {light}"
    );
}

#[test]
fn the_other_click_while_one_is_deciding_is_the_same_chord() {
    for (first, second) in [(keys::DARK, keys::LIGHT), (keys::LIGHT, keys::DARK)] {
        let mut w = alone();
        step(&mut w, first);
        step(&mut w, first);
        step(&mut w, first | second);
        assert_eq!(
            doing(&w),
            Some(d::ANNIHILATION),
            "{first:#x} then {second:#x}"
        );
    }
}

/// How high one spring takes her, after `springs` already this airtime.
fn spring_rise(springs: u8) -> Fx {
    let mut w = alone();
    aloft(&mut w, 6.0);
    w.players[0].vel = V3::ZERO;
    if let Mechanic::Meter { springs: s, .. } = &mut w.players[0].mechanic {
        *s = springs;
    }
    let from = w.players[0].pos.y;
    step(&mut w, keys::DARK | keys::LIGHT);
    let mut top = from;
    for _ in 0..40 {
        step(&mut w, 0);
        top = top.max(w.players[0].pos.y);
    }
    top.sub(from)
}

#[test]
fn each_spring_in_one_airtime_is_worth_less_than_the_last() {
    let (one, two, three) = (spring_rise(0), spring_rise(1), spring_rise(2));
    assert!(
        one.raw() > two.raw() && two.raw() > three.raw(),
        "{} {} {}",
        as_f(one),
        as_f(two),
        as_f(three)
    );
    assert!(
        one.raw() > metres(2.5).raw(),
        "the first spring rose {} m",
        as_f(one)
    );
}

#[test]
fn a_hit_she_lands_gives_a_spring_back_and_landing_gives_them_all() {
    let mut w = facing(2.0);
    if let Mechanic::Meter { springs, .. } = &mut w.players[0].mechanic {
        *springs = 2;
    }
    aloft(&mut w, 0.5);
    tap(&mut w, keys::DARK);
    let mut fewer = false;
    for _ in 0..20 {
        step(&mut w, 0);
        if let Mechanic::Meter { springs, .. } = w.players[0].mechanic {
            fewer |= springs < 2;
        }
    }
    assert!(fewer, "a ball that landed gave nothing back");
    run(&mut w, 60, 0);
    assert!(w.players[0].grounded);
    assert!(matches!(
        w.players[0].mechanic,
        Mechanic::Meter { springs: 0, .. }
    ));
}

#[test]
fn her_movement_is_hers_from_an_empty_bar() {
    // The owner, 2026-11-04: movement is the mage herself; power is behind
    // the bars. The spring, the hop, the dodge and the fall are the same at
    // every bar.
    let spring_at = |bars: i32| {
        let mut w = alone();
        set_bars(&mut w, bars, bars);
        step(&mut w, keys::DARK | keys::LIGHT);
        let mut top = Fx::ZERO;
        for _ in 0..40 {
            step(&mut w, 0);
            top = top.max(w.players[0].pos.y);
        }
        top
    };
    assert_eq!(spring_at(0), spring_at(90), "the spring read a bar");
    let (empty, _) = after_an_auto(Input::D, 0, 0.0);
    let (full, _) = after_an_auto(Input::D, 90, 0.0);
    assert_eq!(empty, full, "the hop read a bar");
    let mut w = alone();
    set_bars(&mut w, 90, 90);
    assert!(!sim::dual::may_blink(&w.players[0]));
    assert!(!sim::dual::may_beat_wings(&w.players[0]));
    assert_eq!(sim::dual::fall_scale(&w.players[0]), Fx::ONE);
}

#[test]
fn the_chain_flies_through_where_she_stood_and_binds_the_first_body() {
    let mut w = facing(6.0);
    let pitch = onto_them(&w);
    let before = w.players[1].health;
    step_looking(&mut w, keys::TWILIGHT, pitch);
    step_looking(&mut w, 0, pitch);
    assert_eq!(doing(&w), Some(d::CHAIN));
    let mut bound = false;
    for _ in 0..30 {
        step_looking(&mut w, 0, pitch);
        let q = w.players[1];
        bound |= q.slowed > 0 && q.slow_mul.raw() <= t::root_slow().raw();
        if let Some(b) = sim::state::hitbox(&w.players[0]) {
            assert!(
                b.from.x.raw() < w.players[0].pos.x.raw(),
                "the chain came from in front"
            );
        }
    }
    assert!(w.players[1].health < before, "six metres off, untouched");
    assert!(bound, "not bound");
}

/// Throw the Flurry dash from `facing(gap)`, keeping the other where it
/// stood, and step until she is free or `most` frames have gone, pressing
/// `space` on the frames it answers yes to. What the world was each frame
/// is the caller's to watch through `each`.
fn flurry(
    gap: f32,
    most: u32,
    mut space: impl FnMut(&World) -> bool,
    mut each: impl FnMut(&World),
) -> World {
    let mut w = facing(gap);
    let stood = w.players[1].pos;
    assert!(hold_into(&mut w, keys::TWILIGHT, 0, d::FLURRY_DASH, 60));
    for _ in 0..most {
        let press = if space(&w) { SPACE } else { 0 };
        step(&mut w, press);
        // Keep them where they stood: what is asked is how often they were hit.
        w.players[1].pos = stood;
        each(&w);
        if w.players[0].action.actionable() {
            break;
        }
    }
    w
}

#[test]
fn the_flurry_dash_cascades_down_its_line_and_then_she_runs_to_the_disk_untouchable() {
    let from = at(-6.0, 0.0, 8.0);
    let m = sim::moves::get(Class::DualMage, d::FLURRY_DASH);
    let (mut hits, mut was, mut untouchable) = (0, 1250, false);
    let w = flurry(
        4.0,
        120,
        |_| false,
        |w| {
            if w.players[1].health < was {
                hits += 1;
            }
            was = w.players[1].health;
            untouchable |= w.players[0].untouchable();
        },
    );
    // The blows cascade: a body four metres out is under two of them, each
    // its own hit, rather than under the whole line every frame.
    assert!(hits >= 2, "the flurry struck {hits} times");
    // And she runs the whole line, to the disk at its end.
    let went = w.players[0].pos.x.sub(from.x);
    assert!(
        went.raw() > m.reach.sub(metres(1.5)).raw(),
        "she went {} m down the line of {}",
        as_f(went),
        as_f(m.reach)
    );
    assert!(untouchable, "the run was not untouchable");
}

#[test]
fn the_flurry_dash_breaks_its_disk_where_she_arrives() {
    let mut broke = None;
    flurry(
        30.0,
        120,
        |_| false,
        |w| {
            for e in w.effects.iter().flatten() {
                if e.kind == sim::effects::EffectKind::Burst && e.slot == d::FLURRY_DASH {
                    broke.get_or_insert((e.pos, w.players[0].pos));
                }
            }
        },
    );
    let (disk, her) = broke.expect("the disk never broke");
    let flat = V3::new(disk.x.sub(her.x), Fx::ZERO, disk.z.sub(her.z)).len();
    assert!(
        flat.raw() < metres(1.0).raw(),
        "it broke {} m from where she arrived",
        as_f(flat)
    );
}

#[test]
fn a_jump_timed_for_the_end_of_the_flurry_run_keeps_the_run() {
    // Pressed through the run's last frames: banked, and thrown on arrival
    // carrying the run's speed. Pressed only once she has stopped, it is an
    // ordinary jump from standing.
    let running = |w: &World| {
        matches!(
            w.players[0].action,
            Action::Recovery { kind, .. } if kind == d::FLURRY_DASH
        ) && w.players[0].haul > 0
    };
    let mut kept = Fx::ZERO;
    let mut w = flurry(
        30.0,
        120,
        |w| running(w) && w.players[0].haul <= 2,
        |w| {
            if !w.players[0].grounded && !running(w) {
                kept = kept.max(w.players[0].vel.x);
            }
        },
    );
    let speed = t::flurry_dash_speed().mul(t::flurry_jump_keep());
    assert!(
        kept.raw() > speed.mul(Fx::ratio(9, 10)).raw(),
        "the jump out of the run carried {} m/s, the run keeps {}",
        as_f(kept),
        as_f(speed)
    );
    assert!(w.players[0].vel.y.raw() != 0 || !w.players[0].grounded);
    // Without the press she stops on the disk.
    w = flurry(30.0, 120, |_| false, |_| {});
    assert!(
        w.players[0].vel.x.raw() < metres(1.0).raw(),
        "she ran on at {} m/s with no jump",
        as_f(w.players[0].vel.x)
    );
}

#[test]
fn an_auto_in_the_air_rises_like_a_jump_once_an_airtime() {
    // Up, and an auto with a direction held: the first rises by the air hop;
    // the second in the same airtime only carries her.
    let rise = |w: &mut World| {
        let from = w.players[0].pos.y;
        step(w, keys::DARK | W);
        let mut top = from;
        for _ in 0..30 {
            step(w, W);
            top = top.max(w.players[0].pos.y);
        }
        top.sub(from)
    };
    let mut w = alone();
    aloft(&mut w, 8.0);
    let first = rise(&mut w);
    let second = rise(&mut w);
    assert!(
        first.raw() > t::auto_air_jump().raw(),
        "the first air hop rose {} m",
        as_f(first)
    );
    assert!(
        second.raw() < first.sub(metres(1.0)).raw(),
        "the second rose {} m against the first's {}",
        as_f(second),
        as_f(first)
    );
}

#[test]
fn a_full_hold_is_a_commitment_and_not_a_charge() {
    // **An MMO spell, not a charge** (the owner, 2026-10-13): held long
    // enough, she is locked into the spell, it plays through whether the
    // button stays down or not, and how strong it is has nothing to do with
    // how long it was held.
    for (button, spell) in [
        (keys::DARK, d::STORM),
        (keys::LIGHT, d::LANCE),
        (keys::TWILIGHT, d::FLURRY_DASH),
    ] {
        let thrown = |keep: u32| {
            let mut w = alone();
            level(&mut w, 50);
            assert!(hold_into(&mut w, button, 0, spell, 60));
            let worth = w.players[0].thrown_at;
            let mut came_out = false;
            for f in 0..90 {
                step(&mut w, if f < keep { button } else { 0 });
                // The storm is its dash (2026-11-04): it goes from its first
                // active frame straight into the run to the spot.
                came_out |= matches!(
                    w.players[0].action,
                    Action::Active { kind, .. } if kind == spell
                ) || (spell == d::STORM
                    && matches!(w.players[0].mechanic, Mechanic::Meter { storm, .. } if storm > 0)
                    && matches!(w.players[0].action, Action::Recovery { kind, .. } if kind == spell));
            }
            (came_out, worth)
        };
        let (let_go, worth_let_go) = thrown(0);
        let (held_on, worth_held_on) = thrown(8);
        assert!(
            let_go,
            "{button:#x}: let go at once, the spell did not play through"
        );
        assert!(
            held_on,
            "{button:#x}: held on, the spell did not play through"
        );
        assert_eq!(
            worth_let_go, worth_held_on,
            "{button:#x}: holding on changed what it was worth"
        );
    }
}

/// Hold left at `pitch` until the storm is thrown, then step `frames` holding
/// `held`; where she got to furthest along +X, where she ended, and what the
/// other fighter lost.
fn storm(gap: f32, held: u16, frames: u32) -> (Fx, V3, i32) {
    let mut w = facing(gap);
    let pitch = onto_their_feet(&w);
    let before = w.players[1].health;
    assert!(hold_into(&mut w, keys::DARK, pitch, d::STORM, 60));
    let mut furthest = w.players[0].pos.x;
    for _ in 0..frames {
        step_looking(&mut w, held, pitch);
        furthest = furthest.max(w.players[0].pos.x);
        w.players[1].pos = at(-6.0 + gap, 0.0, 8.0);
    }
    (furthest, w.players[0].pos, before - w.players[1].health)
}

#[test]
fn the_storm_leaps_her_to_the_crosshair_strikes_there_and_carries_her_through() {
    // From play (2026-11-04, 2026-11-06): she leaps to the spot, the
    // lightning aimed at her comes down on whoever is near it, and holding
    // nothing she carries on through, the way she came.
    let (furthest, _, dealt) = storm(7.0, 0, 70);
    assert!(
        furthest.raw() > metres(-6.0 + 4.5).raw(),
        "she only got to {} m",
        as_f(furthest)
    );
    assert!(dealt > 0, "the lightning missed the body she leapt to");
    let mut w = alone();
    set_bars(&mut w, 100, 100);
    let spot = onto(&w, at(-1.0, 0.0, 8.0));
    assert!(hold_into(&mut w, keys::DARK, spot, d::STORM, 60));
    run_looking(&mut w, 70, spot);
    assert!(
        w.players[0].pos.x.raw() > metres(0.0).raw(),
        "holding nothing she ended at {} m, short of where she struck",
        as_f(w.players[0].pos.x)
    );
}

#[test]
fn holding_a_direction_she_dashes_out_of_the_storm_that_way() {
    let (_, back, _) = storm(7.0, 0, 70);
    let (_, side, _) = storm(7.0, Input::D, 70);
    assert!(
        side.sub(back).len().raw() > metres(2.5).raw(),
        "holding D she ended {:?}, holding nothing {:?}",
        side,
        back
    );
}

#[test]
fn her_own_storm_never_strikes_her() {
    let mut w = facing(2.0);
    let mine = w.players[0].health;
    assert!(hold_into(&mut w, keys::DARK, 0, d::STORM, 60));
    run(&mut w, 80, 0);
    assert_eq!(w.players[0].health, mine);
}

// ---------------------------------------------------------------------------
// The bars build to a big moment (2026-11-04, from play)
// ---------------------------------------------------------------------------

/// Annihilate beside the other fighter from these bars; what it dealt and
/// what was left on her bars.
fn annihilate_from(dark: i32, light: i32) -> (i32, (i32, i32)) {
    let mut w = facing(2.0);
    set_bars(&mut w, dark, light);
    let before = w.players[1].health;
    step(&mut w, keys::DARK | keys::LIGHT);
    run(&mut w, 30, 0);
    (before - w.players[1].health, bars(&w))
}

#[test]
fn both_bars_full_make_the_annihilation_grand_and_it_spends_them() {
    let (plain, plain_left) = annihilate_from(60, 60);
    let (grand, grand_left) = annihilate_from(100, 100);
    assert!(grand > plain * 2, "grand {grand} against plain {plain}");
    // Spent to nothing, and its own hit is the first step of the next climb.
    let step_ = t::meter_auto_push();
    assert_eq!(
        grand_left,
        (step_, step_),
        "the grand one left {grand_left:?}"
    );
    assert_eq!(
        plain_left,
        (60 + step_, 60 + step_),
        "short of full it cost {plain_left:?}"
    );
}

#[test]
fn a_hold_thrown_just_after_a_strike_lands_skips_most_of_its_windup() {
    let wound_up = |primed: bool| -> u16 {
        let mut w = facing(4.0);
        set_bars(&mut w, 100, 100);
        if primed {
            sim::dual::prime(&mut w.players[0]);
        }
        let pitch = onto_them(&w);
        assert!(hold_into(&mut w, keys::LIGHT, pitch, d::LANCE, 60));
        match w.players[0].action {
            Action::Startup { left, .. } => left,
            other => panic!("not winding up: {other:?}"),
        }
    };
    let (cold, hot) = (wound_up(false), wound_up(true));
    assert!(hot * 2 < cold, "primed {hot} frames against {cold}");
}

#[test]
fn a_strike_that_lands_primes_the_next_hold() {
    let mut w = facing(4.0);
    let pitch = onto_them(&w);
    step_looking(&mut w, keys::DARK, pitch);
    step_looking(&mut w, 0, pitch);
    let mut primed = false;
    for _ in 0..20 {
        step_looking(&mut w, 0, pitch);
        primed |= matches!(w.players[0].mechanic, Mechanic::Meter { primed: p, .. } if p > 0);
    }
    assert!(primed, "a strike landed and primed nothing");
}

#[test]
fn the_lance_spends_her_light() {
    let mut w = facing(10.0);
    set_bars(&mut w, 50, 50);
    let pitch = onto_them(&w);
    assert!(hold_into(&mut w, keys::LIGHT, pitch, d::LANCE, 60));
    let (dark, light) = bars(&w);
    assert_eq!((dark, light), (50, 50 - t::lance_spends()));
}

#[test]
fn the_storm_spends_her_dark() {
    let mut w = facing(7.0);
    set_bars(&mut w, 50, 50);
    let pitch = onto_their_feet(&w);
    assert!(hold_into(&mut w, keys::DARK, pitch, d::STORM, 60));
    let (dark, light) = bars(&w);
    assert_eq!((dark, light), (50 - t::lance_spends(), 50));
}

// ---------------------------------------------------------------------------
// The second round of play (2026-11-06)
// ---------------------------------------------------------------------------

#[test]
fn a_second_tap_while_the_chain_is_out_yanks_them_to_her() {
    let mut w = facing(7.0);
    let pitch = onto_them(&w);
    step_looking(&mut w, keys::TWILIGHT, pitch);
    step_looking(&mut w, 0, pitch);
    assert_eq!(doing(&w), Some(d::CHAIN));
    for _ in 0..40 {
        step_looking(&mut w, 0, pitch);
        if w.players[0].action.actionable() {
            break;
        }
    }
    assert!(
        matches!(w.players[0].mechanic, Mechanic::Meter { chained, .. } if chained != 0),
        "the chain bound nobody"
    );
    let far = w.players[1].pos.sub(w.players[0].pos).len();
    step_looking(&mut w, keys::TWILIGHT, pitch);
    step_looking(&mut w, 0, pitch);
    assert_eq!(doing(&w), Some(d::YANK), "the second tap was not the yank");
    run_looking(&mut w, 30, pitch);
    let near = w.players[1].pos.sub(w.players[0].pos).len();
    assert!(
        near.raw() < metres(2.5).raw() && near.raw() < far.raw(),
        "yanked from {} m to {} m",
        as_f(far),
        as_f(near)
    );
}

#[test]
fn with_the_chain_not_out_a_second_tap_is_another_chain() {
    let mut w = alone();
    tap(&mut w, keys::TWILIGHT);
    until_free(&mut w, 60);
    tap(&mut w, keys::TWILIGHT);
    assert_eq!(doing(&w), Some(d::CHAIN));
}

#[test]
fn in_the_air_the_storm_goes_toward_the_crosshair_upward_too() {
    let mut w = alone();
    set_bars(&mut w, 100, 100);
    aloft(&mut w, 4.0);
    w.players[0].vel = V3::ZERO;
    let up = onto(&w, at(-2.0, 12.0, 8.0));
    let from = w.players[0].pos;
    assert!(hold_into(&mut w, keys::DARK, up, d::STORM, 60));
    let mut top = from.y;
    for _ in 0..30 {
        step_looking(&mut w, 0, up);
        top = top.max(w.players[0].pos.y);
    }
    assert!(
        top.raw() > from.y.add(metres(2.0)).raw(),
        "aimed up from {} m she rose to {} m",
        as_f(from.y),
        as_f(top)
    );
}

#[test]
fn she_hops_as_her_strike_passes_her_not_after_it_lands() {
    let mut w = alone();
    step(&mut w, keys::DARK | Input::D);
    step(&mut w, Input::D);
    let mut hopped_while_active = false;
    for _ in 0..20 {
        step(&mut w, Input::D);
        if matches!(w.players[0].action, Action::Active { .. }) && w.players[0].haul > 0 {
            hopped_while_active = true;
        }
    }
    assert!(hopped_while_active, "the hop came after the strike");
}
