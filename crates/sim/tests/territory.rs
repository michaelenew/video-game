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
fn a_heart_nobody_beats_down_stands_for_good() {
    // Half an hour: no bleed, and well past where its clock's counter tops
    // out (2026-10-19).
    let mut w = planted();
    run(&mut w, 30 * 60 * 60, 0);
    assert_eq!(effects_of(&w, EffectKind::Heart).len(), 1, "still standing");
}

#[test]
fn a_heart_holds_the_ground_joined_to_it_and_lets_it_go_when_it_falls() {
    // 2026-10-19: ground joined to a heart, through any run of patches,
    // still ages but does not run out; when the heart goes, what was overdue
    // goes with it. Ground not joined to it fades as ever.
    let mut w = planted();
    let heart = effects_of(&w, EffectKind::Heart)[0];
    let h = heart.pos;
    let ring = as_f(t::heart_ground());
    let near = V3::new(h.x, h.y, h.z.add(metres(ring + 0.8)));
    let further = V3::new(h.x, h.y, h.z.add(metres(ring + 2.4)));
    let lone = V3::new(h.x, h.y, h.z.sub(metres(ring + 12.0)));
    claim(&mut w, near, 1.0, 60);
    claim(&mut w, further, 1.0, 60);
    claim(&mut w, lone, 1.0, 60);
    let held = |w: &World, at: V3| w.ground.iter().any(|p| p.live() && p.covers(at));
    run(&mut w, 60 * t::ground_joined_every() as u32 + 30, 0);
    assert!(held(&w, near), "joined to the heart, it still stands");
    assert!(
        held(&w, further),
        "joined through another patch, so does this"
    );
    assert!(!held(&w, lone), "and ground not joined to it went out");
    let slot = w
        .effects
        .iter()
        .position(|e| e.is_some_and(|e| e.is_a_heart()))
        .expect("the heart");
    w.effects[slot] = None;
    run(&mut w, 2, 0);
    assert!(
        !held(&w, near) && !held(&w, further),
        "with the heart gone, ground that had run out goes at once"
    );
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

// ---------------------------------------------------------------------------
// The spike as the finisher of a bleed (2026-10-19)
// ---------------------------------------------------------------------------

#[test]
fn one_bleed_is_worth_more_than_one_spike() {
    // The bolt has to land and the bleed takes its time; the spike is what
    // finishes it. So the bleed, start to end, is the bigger number than a
    // tapped spike and the eruption out of its blow.
    let lash = sim::moves::get(Class::BloodMage, b::THORN_LASH).damage;
    let ticks = (t::bleed_lasts() / t::bleed_tick()) as i32;
    let bleed = lash + ticks * t::bleed_damage();
    let spike = Fx::from_int(sim::moves::get(Class::BloodMage, b::BLACK_SPIKE).damage)
        .mul(t::spike_two_power())
        .to_int();
    let erupts = Fx::from_int(spike).mul(t::spike_hit_erupts()).to_int();
    assert!(
        bleed > spike + erupts,
        "a bleed is {bleed}, a spike and its eruption {}",
        spike + erupts
    );
}

#[test]
fn an_eruption_spreads_out_rather_than_appearing_whole() {
    let mut w = mage();
    let pool = Effect::pool(0, Class::BloodMage, b::THORN_LASH, at(4.0, 0.0, 8.0), 300);
    place(&mut w, pool);
    let mut sizes = Vec::new();
    step_looking(&mut w, keys::SPIKES, -3500);
    for _ in 0..40 {
        step_looking(&mut w, 0, -3500);
        if let Some(e) = effects_of(&w, EffectKind::BlackSpike)
            .into_iter()
            .find(|e| e.erupted() && e.spreads())
        {
            sizes.push(e.spike_volume().radius);
        }
    }
    assert!(sizes.len() > 2, "fixture: the spike missed the stain");
    assert!(
        sizes[0].raw() < sizes[sizes.len() - 1].raw(),
        "it came up at its full size at once"
    );
}

#[test]
fn on_her_ground_her_bleeds_bite_harder() {
    let ticked = |on: bool| {
        let mut w = mage();
        if on {
            home(&mut w);
        }
        w.players[1].pos = at(0.0, 0.0, 8.0);
        w.players[1].bleed(0, t::bleed_tick() + 1);
        let before = w.players[1].health;
        run(&mut w, t::bleed_tick() as u32 + 2, 0);
        before - w.players[1].health
    };
    let (off, on) = (ticked(false), ticked(true));
    assert!(off > 0, "fixture: the bleed never ticked");
    assert!(on > off, "on her ground {on}, off it {off}");
}

#[test]
fn a_spike_a_stride_short_of_a_stain_sets_it_off() {
    // The spike's disc touching a stain is enough (2026-10-19): one that
    // landed short of the stain it was aimed at used to drink it in passing
    // and start no chain.
    let mut w = mage();
    step_looking(&mut w, keys::SPIKES, -3500);
    let mut landed = None;
    for _ in 0..40 {
        step_looking(&mut w, 0, -3500);
        if let Some(e) = effects_of(&w, EffectKind::BlackSpike).first() {
            landed = Some(e.pos);
            break;
        }
    }
    let landed = landed.expect("fixture: the spike came up");
    let spike = sim::moves::get(Class::BloodMage, b::BLACK_SPIKE);
    let mut w = mage();
    let stain = Effect::pool(0, Class::BloodMage, b::THORN_LASH, landed, 50);
    // Off to the side, its edge just inside the spike's disc and its middle
    // well outside what it covers.
    let off = stain.pool_radius().add(spike.radius).sub(metres(0.2));
    let at = V3::new(landed.x, landed.y, landed.z.add(off));
    place(
        &mut w,
        Effect::pool(0, Class::BloodMage, b::THORN_LASH, at, 50),
    );
    step_looking(&mut w, keys::SPIKES, -3500);
    let mut erupted = false;
    for _ in 0..40 {
        step_looking(&mut w, 0, -3500);
        erupted |= effects_of(&w, EffectKind::BlackSpike)
            .iter()
            .any(|e| e.banked == sim::state::SPIKE_ERUPTION);
    }
    assert!(erupted, "the stain it touched did not go up");
}

#[test]
fn an_eruption_is_worth_the_blood_in_its_stain() {
    // 2026-10-19: every stain used to erupt for the spike's row whatever its
    // size, so a bleed's scattered ticks went up as a minefield. Now the blow
    // is the blood that was in it.
    let landed = {
        let mut w = mage();
        step_looking(&mut w, keys::SPIKES, -3500);
        let mut at = None;
        for _ in 0..40 {
            step_looking(&mut w, 0, -3500);
            if let Some(e) = effects_of(&w, EffectKind::BlackSpike).first() {
                at = Some(e.pos);
                break;
            }
        }
        at.expect("fixture: the spike came up")
    };
    let dealt = |volume: i32| {
        let mut w = mage();
        let stain = Effect::pool(0, Class::BloodMage, b::THORN_LASH, landed, volume);
        let edge = stain.pool_radius().max(metres(1.0));
        place(&mut w, stain);
        w.players[1].pos = V3::new(landed.x, landed.y, landed.z.add(edge));
        let full = w.players[1].health;
        step_looking(&mut w, keys::SPIKES, -3500);
        run_looking(&mut w, 60, -3500);
        full - w.players[1].health
    };
    for volume in [50, 200] {
        let got = dealt(volume);
        assert!(
            got <= volume && got >= volume - 2 * t::pool_drain(),
            "a stain of {volume} erupted for {got}"
        );
    }
}

fn run_looking(w: &mut World, frames: u32, pitch: i16) {
    for _ in 0..frames {
        step_looking(w, 0, pitch);
    }
}

#[test]
fn a_full_table_never_pushes_out_what_a_heart_holds() {
    // 2026-10-19: a held patch stands at its last frame, so it was the
    // weakest on the table and the first to make way when it filled -- the
    // home base pushed out by sparks laid anywhere else.
    let mut w = planted();
    let h = effects_of(&w, EffectKind::Heart)[0].pos;
    let ring = as_f(t::heart_ground());
    let home = V3::new(h.x, h.y, h.z.add(metres(ring + 0.8)));
    claim(&mut w, home, 1.0, 30);
    run(&mut w, 30 * t::ground_joined_every() as u32 + 10, 0);
    let held = |w: &World| w.ground.iter().any(|p| p.live() && p.covers(home));
    assert!(held(&w), "fixture: the heart holds it");
    assert!(w.ground_held != 0, "and the world knows it is held");
    // Fill the table with sparks far from anything, longer-lived than it.
    for k in 0..sim::ground::MAX_PATCHES as i32 + 8 {
        let at = V3::new(h.x.sub(metres(40.0)), h.y, h.z.add(Fx::from_int(3 * k)));
        sim::ground::lay_sparing(&mut w.ground, w.ground_held, &[], 0, at, metres(1.0), 150);
    }
    assert!(
        held(&w),
        "a spark laid elsewhere pushed out what the heart holds"
    );
}

#[test]
fn with_every_patch_held_the_one_furthest_from_her_hearts_makes_way() {
    // 2026-10-19: what her hearts hold never runs out, so a long fight can
    // fill the table with it; new ground must still go down where she
    // fights. The edge of the held field goes, not the base.
    let mut ground = sim::ground::NONE;
    for k in 0..sim::ground::MAX_PATCHES {
        let at = V3::new(Fx::from_int(3 * k as i32), Fx::ZERO, Fx::ZERO);
        sim::ground::lay(&mut ground, 0, at, metres(1.0), 100);
    }
    let held: sim::ground::Held = !0;
    let heart = sim::ground::Disc {
        at: V3::ZERO,
        radius: t::heart_ground(),
        owner: 0,
        holds: true,
    };
    let fresh = V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(-30));
    sim::ground::lay_sparing(&mut ground, held, &[heart], 0, fresh, metres(1.0), 100);
    let far_end = V3::new(
        Fx::from_int(3 * (sim::ground::MAX_PATCHES as i32 - 1)),
        Fx::ZERO,
        Fx::ZERO,
    );
    assert!(
        ground.iter().any(|p| p.covers(fresh)),
        "the new ground was refused"
    );
    assert!(
        !ground.iter().any(|p| p.covers(far_end)),
        "the furthest held patch is still there"
    );
    assert!(
        ground.iter().any(|p| p.covers(V3::ZERO)),
        "and the base was not touched"
    );
}
