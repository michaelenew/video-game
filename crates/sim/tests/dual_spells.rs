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
        landed: 0,
        singe: Fx::ZERO,
        veil: 0,
        warded: false,
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
fn on_the_floor_the_clicks_are_the_autos_and_binary() {
    for (button, wanted) in [
        (keys::DARK, d::DARK_AUTO),
        (keys::TWILIGHT, d::BINARY),
        (keys::LIGHT, d::LIGHT_AUTO),
        (keys::DARK_MAJOR, d::ABYSS),
        (keys::LIGHT_MAJOR, d::JUDGEMENT),
    ] {
        let mut w = alone();
        tap(&mut w, button);
        assert_eq!(doing(&w), Some(wanted), "button {button:#x} on the floor");
    }
}

#[test]
fn in_the_air_the_clicks_are_the_same_autos() {
    for (button, wanted) in [
        (keys::DARK, d::DARK_AUTO),
        (keys::TWILIGHT, d::PHASE),
        (keys::LIGHT, d::LIGHT_AUTO),
        (keys::DARK_MAJOR, d::ABYSS),
        (keys::LIGHT_MAJOR, d::JUDGEMENT),
    ] {
        let mut w = alone();
        aloft(&mut w, 4.0);
        tap(&mut w, button);
        assert_eq!(doing(&w), Some(wanted), "button {button:#x} in the air");
    }
}

#[test]
fn f_and_r_are_her_dark_and_light_spells() {
    for (button, wanted) in [(keys::DARK_SPELL, d::DEVOUR), (keys::LIGHT_SPELL, d::SMITE)] {
        for height in [0.0, 4.0] {
            let mut w = alone();
            if height > 0.0 {
                aloft(&mut w, height);
            }
            step(&mut w, button);
            assert_eq!(doing(&w), Some(wanted), "button {button:#x} at {height} m");
        }
    }
}

#[test]
fn space_and_a_click_defends() {
    for (button, wanted) in [
        (keys::DARK, d::VEIL),
        (keys::TWILIGHT, d::EQUINOX),
        (keys::LIGHT, d::SANCTUARY),
    ] {
        let mut w = alone();
        step(&mut w, SPACE | button);
        assert_eq!(doing(&w), Some(wanted), "space and {button:#x}");
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

#[test]
fn the_dark_auto_curses_and_the_light_auto_purges() {
    let mut w = facing(2.5);
    step(&mut w, keys::DARK);
    until_free(&mut w, 60);
    assert_eq!(stacks(&w), 1, "a stack");
    run(&mut w, 45, 0);
    step(&mut w, keys::LIGHT);
    until_free(&mut w, 60);
    assert_eq!(stacks(&w), 0, "purged");
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
fn the_hop_lifts_her_higher_the_fuller_her_lower_bar() {
    let (_, low) = after_an_auto(W, 0, 0.0);
    let (_, high) = after_an_auto(W, 90, 0.0);
    assert!(low.raw() > 0, "a hop leaves the floor: {}", as_f(low));
    assert!(
        high.raw() > low.add(metres(0.6)).raw(),
        "from a full lower bar she rises {} m, from empty {} m",
        as_f(high),
        as_f(low)
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
fn the_dark_auto_lays_a_curse_and_the_devour_two() {
    let mut w = facing(2.0);
    step(&mut w, keys::DARK);
    until_free(&mut w, 120);
    assert_eq!(stacks(&w), 1, "the auto: one stack");
    // The Devour lands on the ground where she points: at their feet.
    let feet = onto_their_feet(&w);
    step_looking(&mut w, keys::DARK_SPELL, feet);
    assert_eq!(doing(&w), Some(d::DEVOUR));
    run_looking(&mut w, 50, feet);
    assert_eq!(stacks(&w), 3, "the Devour: two more");
}

#[test]
fn the_devour_drains_back_to_her() {
    let mut w = facing(2.0);
    w.players[0].health -= 300;
    let feet = onto_their_feet(&w);
    let before = w.players[0].health;
    step_looking(&mut w, keys::DARK_SPELL, feet);
    assert_eq!(doing(&w), Some(d::DEVOUR));
    run_looking(&mut w, 40, feet);
    assert!(
        w.players[0].health > before,
        "she drank: {} to {}",
        before,
        w.players[0].health
    );
}

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

#[test]
fn a_purge_is_worth_her_light_bar() {
    let from = |light: i32| {
        let mut w = facing(2.5);
        set_bars(&mut w, 0, light);
        cursed(&mut w, 5);
        let before = w.players[1].health;
        step(&mut w, keys::LIGHT);
        run(&mut w, 12, 0);
        before - w.players[1].health
    };
    assert!(from(100) > from(0) + 50, "full light cashes for more");
}

// ---------------------------------------------------------------------------
// Sigils
// ---------------------------------------------------------------------------

fn smite_at_their_feet(w: &mut World) {
    let feet = onto_their_feet(w);
    step_looking(w, keys::LIGHT_SPELL, feet);
    assert_eq!(doing(w), Some(d::SMITE));
    run_looking(w, 30, feet);
}

#[test]
fn the_smite_writes_a_sigil_where_it_lands() {
    let mut w = facing(4.0);
    smite_at_their_feet(&mut w);
    let sigils = effects(&w, EffectKind::Sigil);
    assert_eq!(sigils.len(), 1, "a sigil");
    assert!(
        sigils[0].pos.sub(w.players[1].pos).flat_len().raw() < metres(1.0).raw(),
        "under them"
    );
}

#[test]
fn the_smite_lifts_and_purges() {
    let mut w = facing(4.0);
    level(&mut w, 50);
    cursed(&mut w, 2);
    let mut highest = Fx::ZERO;
    let feet = onto_their_feet(&w);
    step_looking(&mut w, keys::LIGHT_SPELL, feet);
    for _ in 0..40 {
        step_looking(&mut w, 0, feet);
        highest = highest.max(w.players[1].pos.y);
    }
    assert!(
        highest.raw() > metres(0.5).raw(),
        "lifted: {}",
        as_f(highest)
    );
    assert_eq!(stacks(&w), 0, "and purged");
}

#[test]
fn in_her_own_sigil_she_is_warded() {
    let blow = |warded: bool| {
        let mut w = facing(1.2);
        if warded {
            step(&mut w, SPACE | keys::LIGHT);
            until_free(&mut w, 120);
            run(&mut w, 60, 0);
            assert!(w.players[0].grounded, "fixture: back down");
            assert!(sim::dual::warded(&w.players[0]), "she stands in it");
        }
        let before = w.players[0].health;
        let dealt = w.players[0].wound(100);
        assert_eq!(before - w.players[0].health, dealt);
        dealt
    };
    let open = blow(false);
    let warded = blow(true);
    assert_eq!(open, 100);
    assert_eq!(warded, 100 - t::sigil_wards(), "a third off");
}

#[test]
fn a_sigil_burns_anybody_else_in_it() {
    let mut w = facing(4.0);
    smite_at_their_feet(&mut w);
    // They are lifted by the Smite; let them land, then stand them on it.
    run(&mut w, 60, 0);
    let sigil = effects(&w, EffectKind::Sigil)[0];
    w.players[1].pos = sigil.pos;
    let before = w.players[1].health;
    run(&mut w, 60, 0);
    assert!(w.players[1].health < before, "it burns them");
}

#[test]
fn a_third_sigil_replaces_the_oldest() {
    let mut w = alone();
    for _ in 0..3 {
        step(&mut w, SPACE | keys::LIGHT);
        until_free(&mut w, 120);
        run(&mut w, 60, 0);
        w.players[0].pos.x = w.players[0].pos.x.add(metres(8.0));
    }
    assert_eq!(
        effects(&w, EffectKind::Sigil).len(),
        t::sigils_at_once(),
        "two at a time"
    );
}

#[test]
fn sanctuary_writes_a_bigger_sigil_at_her_feet() {
    let mut w = alone();
    let from = w.players[0].pos;
    step(&mut w, SPACE | keys::LIGHT);
    run(&mut w, 12, 0);
    let sigils = effects(&w, EffectKind::Sigil);
    assert_eq!(sigils.len(), 1);
    assert!(sigils[0].pos.sub(from).flat_len().raw() < metres(0.5).raw());
    assert!(sigils[0].reach.raw() > t::sigil_radius().raw(), "bigger");
    assert!(w.players[0].pos.y.raw() > 0, "and a hop out of it");
}

// ---------------------------------------------------------------------------
// The twin stars
// ---------------------------------------------------------------------------

#[test]
fn binary_raises_one_pair_of_stars_that_follows_her() {
    let mut w = alone();
    step(&mut w, keys::TWILIGHT);
    until_free(&mut w, 120);
    assert_eq!(effects(&w, EffectKind::Stars).len(), 1, "a pair");
    w.players[0].pos.x = w.players[0].pos.x.add(metres(5.0));
    run(&mut w, 2, 0);
    let stars = effects(&w, EffectKind::Stars)[0];
    for star in 0..2 {
        let round = stars.star_at(star).sub(w.players[0].pos);
        let flat = V3::new(round.x, Fx::ZERO, round.z).flat_len();
        assert!(
            flat.raw() < t::binary_radius().add(metres(0.1)).raw(),
            "round her"
        );
    }
    run(&mut w, 40, 0);
    step(&mut w, keys::TWILIGHT);
    until_free(&mut w, 120);
    assert_eq!(effects(&w, EffectKind::Stars).len(), 1, "still one pair");
}

#[test]
fn the_dark_star_curses_and_the_light_star_purges() {
    let mut w = facing(1.4);
    step(&mut w, keys::TWILIGHT);
    let before = w.players[1].health;
    let mut most = 0;
    let mut purged = false;
    for _ in 0..t::binary_orbits() as u32 {
        let was = stacks(&w);
        step(&mut w, 0);
        most = most.max(stacks(&w));
        purged |= was > 0 && stacks(&w) == 0;
        // Hold them still beside her: the stars are what this asks about.
        w.players[1].pos = at(-4.6, 0.0, 8.0);
        w.players[1].vel = V3::ZERO;
    }
    assert!(most >= 1, "the dark star cursed them");
    assert!(purged, "the light star purged it");
    assert!(w.players[1].health < before, "and both struck");
}

// ---------------------------------------------------------------------------
// Moving: the air row
// ---------------------------------------------------------------------------

/// A Shadowstep from the air at a fighter `gap` metres off, also aloft.
fn shadowstep_at(gap: f32, curse: u8) -> World {
    let mut w = facing(gap);
    aloft(&mut w, 1.0);
    w.players[1].pos.y = metres(1.0);
    w.players[1].grounded = false;
    if curse > 0 {
        cursed(&mut w, curse);
    }
    let pitch = onto_them(&w);
    w.press(0, d::SHADOWSTEP, Input::looking_at(0, LOOK_RIGHT, pitch));
    assert_eq!(doing(&w), Some(d::SHADOWSTEP));
    let m = sim::moves::get(Class::DualMage, d::SHADOWSTEP);
    for _ in 0..m.startup + 2 {
        // Keep them where they were: what is asked is where she went.
        w.players[1].pos = at(-6.0 + gap, 1.0, 8.0);
        w.players[1].vel = V3::ZERO;
        step_looking(&mut w, 0, pitch);
    }
    w
}

#[test]
fn the_shadowstep_puts_her_behind_the_body_and_curses_it() {
    let w = shadowstep_at(6.0, 0);
    assert!(
        w.players[0].pos.x.raw() > w.players[1].pos.x.raw(),
        "she is behind them: {} past {}",
        as_f(w.players[0].pos.x),
        as_f(w.players[1].pos.x)
    );
    assert!(stacks(&w) >= 1, "and her arrival cursed them");
    assert!(!w.players[0].air_dodged, "a body found keeps the airdodge");
}

#[test]
fn a_cursed_body_is_a_door_from_further() {
    let gap = 11.0;
    let clean = shadowstep_at(gap, 0);
    let cursed = shadowstep_at(gap, 2);
    assert!(
        clean.players[0].pos.x.raw() < clean.players[1].pos.x.raw(),
        "a clean body out of reach: she goes only so far"
    );
    assert!(clean.players[0].air_dodged, "and the airdodge is spent");
    assert!(
        cursed.players[0].pos.x.raw() > cursed.players[1].pos.x.raw(),
        "a cursed one: behind them"
    );
}

#[test]
fn the_shadowstep_with_nobody_still_goes() {
    let mut w = alone();
    aloft(&mut w, 3.0);
    let from = w.players[0].pos;
    w.press(0, d::SHADOWSTEP, Input::looking_at(0, LOOK_RIGHT, 0));
    let m = sim::moves::get(Class::DualMage, d::SHADOWSTEP);
    run(&mut w, m.startup as u32 + 2, 0);
    let went = w.players[0].pos.x.sub(from.x);
    assert!(went.raw() > metres(3.0).raw(), "she went {} m", as_f(went));
    assert!(w.players[0].air_dodged, "and paid for it");
}

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

#[test]
fn phase_puts_her_along_the_crosshair_once_a_trip() {
    let mut w = alone();
    aloft(&mut w, 3.0);
    let from = w.players[0].pos;
    tap(&mut w, keys::TWILIGHT);
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

// ---------------------------------------------------------------------------
// Defending: the takeoffs
// ---------------------------------------------------------------------------

#[test]
fn the_veil_is_untouchable_on_the_way_up() {
    let mut w = facing(1.2);
    step(&mut w, SPACE | keys::DARK);
    assert!(w.players[0].untouchable(), "untouchable from the press");
    let m = sim::moves::get(Class::DualMage, d::VEIL);
    run(&mut w, m.startup as u32 + 1, 0);
    assert!(w.players[0].untouchable(), "untouchable");
    assert!(!w.players[0].grounded, "and rising");
    let before = w.players[0].health;
    // The Bulwark's poke, straight at her.
    w.advance([
        Input::looking_at(0, LOOK_RIGHT, 0),
        Input::looking_at(Input::LEFT, LOOK_LEFT, 0),
    ]);
    run(&mut w, 6, 0);
    assert_eq!(w.players[0].health, before, "nothing touched her");
    run(&mut w, t::veil_for() as u32, 0);
    assert!(!w.players[0].untouchable(), "and it ends");
}

#[test]
fn the_veil_curses_whoever_was_beside_her() {
    let mut w = facing(1.2);
    step(&mut w, SPACE | keys::DARK);
    run(&mut w, 10, 0);
    assert_eq!(stacks(&w), 1);
}

#[test]
fn equinox_levels_her_bars() {
    let mut w = alone();
    set_bars(&mut w, 70, 30);
    step(&mut w, SPACE | keys::TWILIGHT);
    let (dark, light) = bars(&w);
    let gap = dark - light;
    assert!(
        gap <= 40 - 2 * t::equinox_levels() + 1,
        "levelled: {dark} and {light}"
    );
    assert!(dark >= light, "and never crossed");
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

// ---------------------------------------------------------------------------
// The majors
// ---------------------------------------------------------------------------

fn abyss_on_them(w: &mut World) {
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

fn abyss_size(dark: i32) -> Fx {
    let mut w = facing(5.0);
    set_bars(&mut w, dark, dark);
    abyss_on_them(&mut w);
    let m = sim::moves::get(Class::DualMage, d::ABYSS);
    run(&mut w, (m.startup + 2) as u32, 0);
    effects(&w, EffectKind::Abyss)[0].field_radius()
}

#[test]
fn the_majors_grow_from_small_to_the_biggest_thing_on_the_screen() {
    let small = abyss_size(0);
    let big = abyss_size(100);
    let ratio = as_f(big) / as_f(small);
    let want = as_f(t::major_size_full()) / as_f(t::major_size_empty());
    assert!(
        (ratio - want).abs() < 0.05,
        "the black sun grows {ratio:.2}x from empty to full, the curve says {want:.2}x"
    );
    assert!(ratio > 3.0, "and that is a lot");
}

fn judgement_columns(light: i32) -> usize {
    let mut w = facing(5.0);
    set_bars(&mut w, light, light);
    let feet = onto_their_feet(&w);
    step_looking(&mut w, keys::LIGHT_MAJOR, feet);
    let m = sim::moves::get(Class::DualMage, d::JUDGEMENT);
    run_looking(&mut w, (m.startup + 1) as u32, feet);
    1 + effects(&w, EffectKind::Burst)
        .iter()
        .filter(|e| e.slot == d::JUDGEMENT)
        .map(|e| e.banked as usize)
        .sum::<usize>()
}

#[test]
fn judgement_brings_down_more_columns_the_fuller_her_light() {
    assert_eq!(judgement_columns(0), 1, "one from empty");
    assert_eq!(
        judgement_columns(100),
        t::judgement_columns() as usize,
        "all of them from full"
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

#[test]
fn apotheosis_bursts_from_her_as_she_ascends_and_purges_at_full_power() {
    let mut w = facing(2.0);
    set_bars(&mut w, t::tier_wings() - 3, t::tier_wings() - 3);
    cursed(&mut w, 5);
    let before = w.players[1].health;
    // Binary goads both bars: over the top, and the wings come.
    step(&mut w, keys::TWILIGHT);
    run(&mut w, 3, 0);
    assert!(sim::dual::ascending(&w.players[0]), "fixture: she ascended");
    assert!(
        effects(&w, EffectKind::Burst)
            .iter()
            .any(|e| e.slot == sim::effects::APOTHEOSIS),
        "the nova"
    );
    assert_eq!(stacks(&w), 0, "every curse in it purged");
    let full = sim::dual::purge_damage(5, t::depth_ceiling());
    assert!(
        before - w.players[1].health >= full,
        "at the top of her power: {} dealt, the purge alone is {full}",
        before - w.players[1].health
    );
}

#[test]
fn apotheosis_alone_hits_nobody() {
    let mut w = alone();
    set_bars(&mut w, t::tier_wings() - 3, t::tier_wings() - 3);
    let before = w.players[1].health;
    step(&mut w, keys::TWILIGHT);
    run(&mut w, 3, 0);
    assert!(sim::dual::ascending(&w.players[0]));
    assert_eq!(w.players[1].health, before, "nobody beside her, nothing");
}

// ---------------------------------------------------------------------------
// Creatures
// ---------------------------------------------------------------------------

#[test]
fn a_creature_is_cursed_and_purged_like_a_fighter() {
    let mut w = World::hunt([Class::DualMage, Class::Bulwark]);
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
    // Judgement on the ground it stands on: its column reaches the animal
    // whatever part is over that spot, and it is light.
    let ground = V3::new(
        beast.pos.x,
        w.terrain().ground_under(beast.pos),
        beast.pos.z,
    );
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
    for (button, wanted) in [(keys::DARK, d::CHAIN_LIGHTNING), (keys::LIGHT, d::DISK_RAY)] {
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
    let cap = sim::moves::get(Class::DualMage, d::CHAIN_LIGHTNING).channel as u32;
    for (button, tap, hold) in [
        (keys::DARK, d::DARK_AUTO, d::CHAIN_LIGHTNING),
        (keys::LIGHT, d::LIGHT_AUTO, d::DISK_RAY),
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
        for _ in 0..cap + 40 {
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
fn a_long_hold_fills_its_bar() {
    let mut w = alone();
    level(&mut w, 40);
    assert!(hold_into(&mut w, keys::DARK, 0, d::CHAIN_LIGHTNING, 60));
    let p = &w.players[0];
    let (dark, light) = sim::dual::bars(p).unwrap();
    let want = 40 + t::hold_fills();
    assert!(
        (dark.to_int() - want).abs() <= 2,
        "the lightning left the dark bar at {} rather than about {want}",
        as_f(dark)
    );
    assert!(
        (light.to_int() - 40).abs() <= 2,
        "the light bar moved: {}",
        as_f(light)
    );
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

#[test]
fn chain_lightning_strikes_the_body_under_the_crosshair_and_curses_it() {
    let mut w = facing(9.0);
    let pitch = onto_them(&w);
    let before = w.players[1].health;
    assert!(hold_into(&mut w, keys::DARK, pitch, d::CHAIN_LIGHTNING, 60));
    run_looking(&mut w, 30, pitch);
    assert!(w.players[1].health < before, "nine metres off, untouched");
    assert!(stacks(&w) >= 1, "and not cursed");
}

#[test]
fn chain_lightning_leaps_from_body_to_body() {
    // Three gnats in a line, three metres apart, the crosshair on the first.
    use sim::critter::{Critter, is};
    use sim::species::{SpeciesId, gnats};
    let mut w = World::hunt_of([Class::DualMage, Class::DualMage], SpeciesId::GNATS);
    let sp = w.critters.sp();
    for c in w.critters.iter_mut() {
        *c = Critter::EMPTY;
    }
    let lane = sim::critcheck::lane();
    for (k, x) in [6, 9, 12].into_iter().enumerate() {
        let mut c = Critter::new(
            sp,
            gnats::GNAT,
            lane.add(V3::new(Fx::from_int(x), Fx::ZERO, Fx::ZERO)),
            1 << 15,
        );
        c.state = is::FLINCH;
        c.timer = u16::MAX;
        c.health = i16::MAX;
        w.critters[k] = c;
    }
    w.pack.as_mut().unwrap().grace = u16::MAX;
    w.players[0].pos = lane;
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(12));
    let middle = w.critters[0].body(sp).middle();
    let pitch = sim::aim::look_onto_closely(lane, 0, Fx::ZERO, Fx::ZERO, middle);
    let mut struck = [false; 3];
    for _ in 0..60 {
        let before: Vec<i16> = (0..3).map(|k| w.critters[k].health).collect();
        w.advance([
            Input::looking_at(keys::DARK, 0, pitch),
            Input::looking_at(0, 0, 0),
        ]);
        for k in 0..3 {
            struck[k] |= w.critters[k].health < before[k];
        }
        if matches!(
            w.players[0].action,
            Action::Recovery {
                kind: d::CHAIN_LIGHTNING,
                ..
            }
        ) {
            break;
        }
    }
    assert_eq!(struck, [true; 3], "the lightning did not reach all three");
}

#[test]
fn the_disk_ray_strikes_from_above_and_purges() {
    let mut w = facing(10.0);
    cursed(&mut w, 3);
    let pitch = onto_them(&w);
    let before = w.players[1].health;
    assert!(hold_into(&mut w, keys::LIGHT, pitch, d::DISK_RAY, 60));
    let m = sim::moves::get(Class::DualMage, d::DISK_RAY);
    // Its line leaves the disk over her head.
    run_looking(&mut w, m.startup as u32, pitch);
    if let Some(b) = sim::state::hitbox(&w.players[0]) {
        assert!(
            b.from.y.raw() > metres(2.0).raw(),
            "the blast leaves from {} m up",
            as_f(b.from.y)
        );
    }
    run_looking(&mut w, 30, pitch);
    assert!(w.players[1].health < before, "ten metres off, untouched");
    assert_eq!(stacks(&w), 0, "the curse was not purged");
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
fn phase_leaves_her_moving_along_it_and_rising() {
    let mut w = alone();
    aloft(&mut w, 3.0);
    tap(&mut w, keys::TWILIGHT);
    let m = sim::moves::get(Class::DualMage, d::PHASE);
    run(&mut w, m.startup as u32 + 1, 0);
    let v = w.players[0].vel;
    assert!(
        v.x.raw() > metres(3.0).raw(),
        "carrying {} m/s along",
        as_f(v.x)
    );
    assert!(v.y.raw() > 0, "and not rising: {} m/s", as_f(v.y));
}

#[test]
fn the_equinox_blinks_as_she_leaves_the_floor() {
    let mut w = alone();
    let from = w.players[0].pos;
    step(&mut w, SPACE | keys::TWILIGHT);
    assert_eq!(doing(&w), Some(d::EQUINOX));
    let m = sim::moves::get(Class::DualMage, d::EQUINOX);
    run(&mut w, m.startup as u32 + 2, 0);
    let went = w.players[0].pos.x.sub(from.x);
    assert!(went.raw() > metres(2.5).raw(), "she went {} m", as_f(went));
    assert!(
        w.players[0].pos.y.raw() > 0,
        "and she is still on the floor"
    );
}

#[test]
fn a_full_hold_is_a_commitment_and_not_a_charge() {
    // **An MMO spell, not a charge** (the owner, 2026-10-13): held long
    // enough, she is locked into the spell, it plays through whether the
    // button stays down or not, and how strong it is has nothing to do with
    // how long it was held.
    for (button, spell) in [
        (keys::DARK, d::CHAIN_LIGHTNING),
        (keys::LIGHT, d::DISK_RAY),
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
                came_out |= matches!(
                    w.players[0].action,
                    Action::Active { kind, .. } if kind == spell
                );
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
