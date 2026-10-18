//! **The Blood mage's territory** (2026-10-18): her ground, the strong
//! version, the swim, hearts, and her thralls on her ground. What a player
//! would notice if any of it broke.
//!
//! `docs/design/exploration/0021_blood_mage_territory.md` is the direction:
//! her abilities lay ground where they touch, stains are ground and bleed
//! into it as they fade, patches near each other bridge; on her ground she
//! hits harder, off it she seeds more; `shift` on it is the swim; `E` plants
//! a heart that holds ground and sends out thralls.

use sim::class::Class;
use sim::effects::{Effect, EffectKind};
use sim::moves::blood as b;
use sim::moves::blood::keys;
use sim::state::{ON_GROUND, SWIM_NONE, SWIM_UNDER};
use sim::tuning as t;
use sim::{Fx, Input, V3, World};

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
    w.players[1].pos = at(30.0, 0.0, 8.0);
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

fn place(w: &mut World, effect: Effect) {
    let slot = w.effects.iter().position(|s| s.is_none()).expect("room");
    w.effects[slot] = Some(effect);
}

/// A patch of her ground at `pos`, `radius` wide, for `life` frames.
fn claim(w: &mut World, pos: V3, radius: f32, life: u16) {
    sim::ground::lay(&mut w.ground, 0, pos, metres(radius), life);
}

/// Her ground under her feet, wide and long-lived.
fn home(w: &mut World) {
    let feet = w.players[0].pos;
    claim(w, feet, 4.0, 900);
}

fn patches(w: &World) -> Vec<sim::ground::Patch> {
    w.ground.iter().filter(|p| p.live()).copied().collect()
}

fn effects_of(w: &World, kind: EffectKind) -> Vec<Effect> {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.kind == kind)
        .copied()
        .collect()
}

fn thralls(w: &World) -> Vec<Effect> {
    effects_of(w, EffectKind::Thrall)
}

/// A thrall of hers, of `essence`, at `pos`.
fn thrall(w: &mut World, pos: V3, essence: i32) {
    let mut e = Effect::pool(0, Class::BloodMage, b::HEART, pos, essence);
    e.kind = EffectKind::Thrall;
    e.life = u16::MAX;
    place(w, e);
}

// ---------------------------------------------------------------------------
// The ground
// ---------------------------------------------------------------------------

#[test]
fn a_spike_lays_her_ground_where_it_comes_up() {
    let mut w = mage();
    assert!(patches(&w).is_empty(), "clean floor to start");
    step_looking(&mut w, keys::SPIKES, -2500);
    run(&mut w, 40, 0);
    let ground = patches(&w);
    assert!(!ground.is_empty(), "the spike claimed the floor");
    let her = w.players[0].pos;
    assert!(
        ground
            .iter()
            .any(|p| p.at.sub(her).flat_len().raw() > metres(2.0).raw()),
        "out where it came up, not at her feet"
    );
}

#[test]
fn the_thorn_lash_lays_ground_where_it_lands_hit_or_miss() {
    let mut w = mage();
    // Down at the floor a few metres ahead: a miss.
    step_looking(&mut w, keys::THORNS, -5000);
    run(&mut w, 40, 0);
    let her = w.players[0].pos;
    assert!(
        patches(&w)
            .iter()
            .any(|p| p.at.sub(her).flat_len().raw() > metres(1.0).raw()),
        "a missed lash still claimed where it struck the floor"
    );
}

#[test]
fn thrown_off_her_ground_a_move_seeds_more_than_on_it() {
    let seeded = |on: bool| {
        let mut w = mage();
        if on {
            home(&mut w);
        }
        let before = patches(&w).len();
        step_looking(&mut w, keys::SPIKES, -2500);
        run(&mut w, 40, 0);
        let her = w.players[0].pos;
        let far: Vec<_> = patches(&w)
            .into_iter()
            .filter(|p| p.at.sub(her).flat_len().raw() > metres(4.5).raw())
            .collect();
        assert!(patches(&w).len() > before, "it laid ground");
        far.iter().map(|p| p.radius().raw()).max().unwrap_or(0)
    };
    let (weak, strong) = (seeded(false), seeded(true));
    assert!(
        weak > strong,
        "off her ground the patch is wider: {} against {}",
        weak,
        strong
    );
}

#[test]
fn a_stain_is_her_ground_and_bleeds_into_it_as_it_fades() {
    let mut w = mage();
    let spot = at(0.0, 0.0, 8.0);
    place(
        &mut w,
        Effect::pool(0, Class::BloodMage, b::THORN_LASH, spot, 12),
    );
    assert!(w.on_blood(0, spot), "a standing stain is her ground");
    assert!(patches(&w).is_empty(), "and no patch yet");
    run(&mut w, 240, 0);
    assert!(
        w.effects.iter().flatten().all(|e| !e.is_a_pool()),
        "the stain drained away"
    );
    assert!(w.on_blood(0, spot), "and left her ground where it stood");
}

#[test]
fn a_lone_patch_goes_out() {
    let mut w = mage();
    claim(&mut w, at(0.0, 0.0, 8.0), 1.0, t::ground_touch_lasts());
    run(&mut w, t::ground_touch_lasts() as u32 + 2, 0);
    assert!(patches(&w).is_empty(), "nothing fed it, and it went out");
}

#[test]
fn two_patches_near_each_other_bridge_and_hold_longer() {
    let mut w = mage();
    let life = t::ground_touch_lasts();
    claim(&mut w, at(0.0, 0.0, 8.0), 1.0, life);
    claim(&mut w, at(3.5, 0.0, 8.0), 1.0, life);
    run(&mut w, 2 * t::ground_bridge_every() as u32 + 1, 0);
    assert!(
        w.on_blood(0, at(1.75, 0.0, 8.0)),
        "the gap between them was bridged"
    );
    run(&mut w, life as u32, 0);
    assert!(
        !patches(&w).is_empty(),
        "joined, they outlived a lone patch's life"
    );
}

#[test]
fn islands_too_far_apart_do_not_bridge() {
    let mut w = mage();
    claim(&mut w, at(0.0, 0.0, 8.0), 1.0, 300);
    claim(&mut w, at(8.0, 0.0, 8.0), 1.0, 300);
    run(&mut w, 3 * t::ground_bridge_every() as u32, 0);
    assert!(
        !w.on_blood(0, at(4.0, 0.0, 8.0)),
        "no bridge across six metres"
    );
}

#[test]
fn her_ground_is_hers_alone() {
    let mut w = World::with_classes([Class::BloodMage, Class::BloodMage]);
    w.players[0].pos = at(-6.0, 0.0, 8.0);
    w.players[1].pos = at(-4.0, 0.0, 8.0);
    claim(&mut w, at(-5.0, 0.0, 8.0), 3.0, 900);
    step(&mut w, 0);
    assert!(w.players[0].on_blood, "she stands on her ground");
    assert!(!w.players[1].on_blood, "the other Blood mage does not");
}

#[test]
fn a_change_of_frame_moves_her_ground_with_everything() {
    let mut w = mage();
    claim(&mut w, at(0.0, 0.0, 8.0), 2.0, 900);
    let before = patches(&w)[0].at;
    w.shift(at(10.0, 0.0, -3.0));
    let after = patches(&w)[0].at;
    assert_eq!(after, before.add(at(10.0, 0.0, -3.0)));
}

// ---------------------------------------------------------------------------
// The strong version
// ---------------------------------------------------------------------------

/// What one Sweep does to him, standing in its reach, with her on her ground
/// or off it.
fn sweep_on(home_ground: bool) -> i32 {
    let mut w = mage();
    if home_ground {
        home(&mut w);
    }
    w.players[1].pos = at(-4.2, 0.0, 8.0);
    let his = w.players[1].health;
    step(&mut w, keys::SCYTHE);
    for _ in 0..30 {
        w.players[1].pos = at(-4.2, 0.0, 8.0);
        step(&mut w, 0);
    }
    his - w.players[1].health
}

#[test]
fn on_her_ground_her_moves_hit_harder() {
    let (weak, strong) = (sweep_on(false), sweep_on(true));
    assert!(weak > 0, "the Sweep landed");
    assert!(
        strong > weak,
        "on her ground it hit harder: {weak} then {strong}"
    );
}

#[test]
fn on_her_ground_the_reap_lunges_further() {
    let lunge = |home_ground: bool| {
        let mut w = mage();
        if home_ground {
            home(&mut w);
        }
        let from = w.players[0].pos;
        let open =
            (sim::moves::get(Class::BloodMage, b::SWEEP).startup as u32).max(t::hold_from() as u32);
        for _ in 0..open + 10 {
            step_looking(&mut w, keys::SCYTHE, -2000);
        }
        run(&mut w, 40, 0);
        w.players[0].pos.sub(from).flat_len()
    };
    let (weak, strong) = (lunge(false), lunge(true));
    assert!(
        strong.raw() > weak.raw(),
        "{} then {}",
        as_f(weak),
        as_f(strong)
    );
}

#[test]
fn the_strong_version_is_no_wider() {
    let reach = |home_ground: bool| {
        let mut w = mage();
        if home_ground {
            home(&mut w);
        }
        step_looking(&mut w, keys::SPIKES, -2500);
        run(&mut w, 16, 0);
        effects_of(&w, EffectKind::BlackSpike)
            .first()
            .map(|e| e.field_radius().raw())
    };
    assert_eq!(reach(false), reach(true), "the same spike, harder only");
}

// ---------------------------------------------------------------------------
// The swim
// ---------------------------------------------------------------------------

#[test]
fn shift_on_her_ground_sinks_her_into_it_and_off_it_is_a_dodge() {
    let mut w = mage();
    home(&mut w);
    step(&mut w, Input::SHIFT | Input::W);
    assert_eq!(w.players[0].swim, SWIM_UNDER, "on her ground she swims");
    assert!(
        matches!(w.players[0].action, sim::state::Action::Free),
        "and it is not a dodge"
    );
    assert!(
        w.players[0].hurt_height().raw() < t::body_height().raw(),
        "sunk, she is short"
    );

    let mut w = mage();
    step(&mut w, Input::SHIFT | Input::W);
    assert_eq!(w.players[0].swim, SWIM_NONE, "off it, no swim");
    assert!(
        matches!(w.players[0].action, sim::state::Action::Dodge { .. }),
        "an ordinary dodge"
    );
}

#[test]
fn swimming_is_faster_than_walking() {
    let went = |swim: bool| {
        let mut w = mage();
        claim(&mut w, at(-2.0, 0.0, 8.0), 6.0, 900);
        let from = w.players[0].pos;
        if swim {
            step(&mut w, Input::SHIFT);
        }
        let bits = if swim {
            Input::SHIFT | Input::W
        } else {
            Input::W
        };
        run(&mut w, 30, bits);
        w.players[0].pos.sub(from).flat_len()
    };
    let (walked, swam) = (went(false), went(true));
    assert!(
        swam.raw() > walked.raw(),
        "{} walking, {} swimming",
        as_f(walked),
        as_f(swam)
    );
}

#[test]
fn letting_go_of_shift_brings_her_up() {
    let mut w = mage();
    home(&mut w);
    run(&mut w, 5, Input::SHIFT);
    assert_eq!(w.players[0].swim, SWIM_UNDER);
    step(&mut w, 0);
    assert_eq!(w.players[0].swim, SWIM_NONE, "up again");
    assert_eq!(w.players[0].hurt_height(), t::body_height());
}

#[test]
fn swimming_off_the_edge_pops_her_out() {
    let mut w = mage();
    let feet = w.players[0].pos;
    claim(&mut w, feet, 1.5, 900);
    step(&mut w, Input::SHIFT);
    assert_eq!(w.players[0].swim, SWIM_UNDER);
    let mut popped = false;
    for _ in 0..60 {
        step(&mut w, Input::SHIFT | Input::W);
        if w.players[0].swim != SWIM_UNDER {
            popped = true;
            break;
        }
    }
    assert!(popped, "she came up at the edge");
    let edge = w.players[0].pos;
    run(&mut w, 10, Input::SHIFT);
    assert!(
        w.players[0].pos.sub(edge).flat_len().raw() > metres(1.0).raw(),
        "and burst out past it"
    );
    run(&mut w, 30, Input::SHIFT | Input::W);
    assert!(
        matches!(w.players[0].action, sim::state::Action::Free),
        "a shift still held from the swim is not a dodge"
    );
}

#[test]
fn a_click_brings_her_up_as_its_strong_move() {
    let mut w = mage();
    home(&mut w);
    run(&mut w, 5, Input::SHIFT);
    step(&mut w, Input::SHIFT | keys::SPIKES);
    assert_ne!(w.players[0].swim, SWIM_UNDER, "she came up");
    assert!(
        w.players[0].action.attack_kind() == Some(b::BLACK_SPIKE),
        "as the spike"
    );
    assert!(
        w.players[0].thrown_at.raw() > Fx::ONE.raw(),
        "and the strong one"
    );
}

#[test]
fn a_blow_brings_her_up() {
    let mut w = mage();
    home(&mut w);
    run(&mut w, 3, Input::SHIFT);
    w.players[0].wound(1);
    w.players[0].action = sim::state::Action::Stagger { left: 10 };
    step(&mut w, Input::SHIFT);
    assert_ne!(w.players[0].swim, SWIM_UNDER, "struck, she is up");
}

// ---------------------------------------------------------------------------
// Hearts
// ---------------------------------------------------------------------------

/// Plant a heart a few metres ahead and let it come out.
/// Her heart meter, full.
fn earned(w: &mut World) {
    w.players[0].heart_meter = t::heart_meter_full() as u16;
}

fn planted() -> World {
    let mut w = mage();
    earned(&mut w);
    step_looking(&mut w, keys::PLANT, -2500);
    run(&mut w, 30, 0);
    w
}

#[test]
fn e_plants_a_heart_where_she_aims_and_it_holds_ground() {
    let w = planted();
    let hearts = effects_of(&w, EffectKind::Heart);
    assert_eq!(hearts.len(), 1, "a heart");
    let heart = hearts[0];
    assert!(
        heart.pos.sub(w.players[0].pos).flat_len().raw() > metres(2.0).raw(),
        "out where she aimed"
    );
    assert!(heart.banked >= t::heart_least(), "holding its blood");
    assert!(w.on_blood(0, heart.pos), "and the floor round it is hers");
    assert!(
        w.effects
            .iter()
            .flatten()
            .all(|e| !e.is_a_pool() || e.pos.sub(heart.pos).flat_len().raw() > metres(0.5).raw()),
        "what she paid went into it, not onto the floor"
    );
}

#[test]
fn a_heart_sends_out_thralls_from_its_own_blood() {
    let mut w = planted();
    let before = effects_of(&w, EffectKind::Heart)[0].banked;
    run(&mut w, t::heart_every() as u32 + 2, 0);
    assert!(!thralls(&w).is_empty(), "a thrall came out of it");
    let after = effects_of(&w, EffectKind::Heart)[0].banked;
    assert!(
        before - after >= t::heart_thrall(),
        "made of its blood: {before} to {after}"
    );
}

#[test]
fn a_heart_takes_more_than_one_blow() {
    let mut w = planted();
    let heart = effects_of(&w, EffectKind::Heart)[0];
    let fresh = heart.banked;
    // Him beside it, swinging once.
    w.players[1].pos = heart.pos.add(at(1.2, 0.0, 0.0));
    for f in 0..30 {
        let his = if f == 0 { Input::LEFT } else { 0 };
        w.advance([
            Input::looking_at(0, LOOK_RIGHT, 0),
            Input::looking_at(his, LOOK_LEFT, 0),
        ]);
        w.players[1].pos = heart.pos.add(at(1.2, 0.0, 0.0));
    }
    let after = effects_of(&w, EffectKind::Heart);
    assert_eq!(after.len(), 1, "one blow did not kill it");
    assert!(after[0].banked < fresh, "but it hurt it");
}

#[test]
fn no_more_hearts_than_the_cap() {
    let mut w = mage();
    for _ in 0..(t::hearts_most() + 1) {
        earned(&mut w);
        step_looking(&mut w, keys::PLANT, -2500);
        run(&mut w, 60, 0);
    }
    assert_eq!(
        effects_of(&w, EffectKind::Heart).len() as i32,
        t::hearts_most(),
        "the oldest gave way"
    );
}

// ---------------------------------------------------------------------------
// Thralls on her ground
// ---------------------------------------------------------------------------

#[test]
fn on_her_ground_a_thrall_does_not_drain_and_knows_it() {
    let mut w = mage();
    claim(&mut w, at(0.0, 0.0, 8.0), 3.0, 900);
    thrall(&mut w, at(0.0, 0.0, 8.0), 100);
    thrall(&mut w, at(0.0, 0.0, -20.0), 100);
    run(&mut w, 120, 0);
    let mut th = thralls(&w);
    th.sort_by_key(|t| -t.pos.z.raw());
    assert!(th[0].struck.has(ON_GROUND), "on her ground it knows it");
    assert_eq!(th[0].banked, 100, "and has not drained");
    assert!(th[1].banked < 100, "off it, it drains");
}

#[test]
fn off_her_ground_a_thrall_walks_back_to_it() {
    let mut w = mage();
    claim(&mut w, at(0.0, 0.0, 6.0), 2.0, 900);
    thrall(&mut w, at(0.0, 0.0, 12.0), 300);
    run(&mut w, 120, 0);
    let th = thralls(&w)[0];
    assert!(
        w.on_blood(0, th.pos),
        "it went home: {}, {}",
        as_f(th.pos.x),
        as_f(th.pos.z)
    );
}

#[test]
fn on_her_ground_a_thrall_bites_harder() {
    let bite = |on: bool| {
        let mut w = mage();
        w.players[1].pos = at(4.0, 0.0, 8.0);
        let ground = if on {
            at(3.5, 0.0, 8.0)
        } else {
            at(6.5, 0.0, 8.0)
        };
        claim(&mut w, ground, 1.0, 900);
        thrall(&mut w, at(3.3, 0.0, 8.0), 200);
        let his = w.players[1].health;
        for _ in 0..(t::thrall_bite_every() as u32 + 4) {
            w.players[1].pos = at(4.0, 0.0, 8.0);
            step(&mut w, 0);
            if w.players[1].health < his {
                return his - w.players[1].health;
            }
        }
        0
    };
    let (off, on) = (bite(false), bite(true));
    assert!(off > 0, "it bit off her ground");
    assert!(on > off, "and harder on it: {off} then {on}");
}

// ---------------------------------------------------------------------------
// The heart meter
// ---------------------------------------------------------------------------

#[test]
fn a_heart_is_refused_until_she_has_dealt_enough() {
    let mut w = mage();
    step_looking(&mut w, keys::PLANT, -2500);
    run(&mut w, 30, 0);
    assert!(
        effects_of(&w, EffectKind::Heart).is_empty(),
        "an empty meter plants nothing"
    );
    assert_ne!(
        w.players[0].action.attack_kind(),
        Some(b::HEART),
        "and throws nothing"
    );
}

#[test]
fn the_meter_fills_with_the_damage_she_deals() {
    let mut w = mage();
    w.players[1].pos = at(-4.2, 0.0, 8.0);
    let his = w.players[1].health;
    step(&mut w, keys::SCYTHE);
    for _ in 0..30 {
        w.players[1].pos = at(-4.2, 0.0, 8.0);
        step(&mut w, 0);
    }
    let dealt = his - w.players[1].health;
    assert!(dealt > 0, "the Sweep landed");
    assert_eq!(
        w.players[0].heart_meter as i32,
        dealt.min(t::heart_meter_full()),
        "every point of it counts toward a heart"
    );
}

#[test]
fn her_own_costs_fill_nothing() {
    let mut w = mage();
    step_looking(&mut w, keys::SPIKES, -2500);
    run(&mut w, 60, 0);
    assert!(w.players[0].grey > 0, "fixture: it cost her");
    assert_eq!(w.players[0].heart_meter, 0, "a miss deals nothing");
}

#[test]
fn planting_a_heart_empties_the_meter() {
    let w = planted();
    assert_eq!(effects_of(&w, EffectKind::Heart).len(), 1);
    assert_eq!(w.players[0].heart_meter, 0, "spent on the heart");
    assert!(!w.players[0].heart_ready());
}
