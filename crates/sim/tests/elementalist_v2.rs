//! The Elementalist's v2: what a player would notice if any of it broke.
//!
//! `docs/design/elementalist-v2.md` is the specification and
//! `docs/design/plans/elementalist-v2.md` is the brief this file follows,
//! milestone by milestone. The first milestone is **fire in the air**: the
//! Cinder spray on middle click in both rows, the cloud of embers it bursts
//! into, the two air shots coming out lit when they fly through fire, and the
//! Gale shoving a stone it passes.

use sim::class::{Mechanic, Structure};
use sim::effects::{Effect, EffectKind};
use sim::gust::Gale;
use sim::moves::elementalist as e;
use sim::state::Action;
use sim::tuning as t;
use sim::{Class, Fx, Input, V3, World};

const L: u16 = Input::LEFT;
const R: u16 = Input::RIGHT;
const M: u16 = Input::MIDDLE;
const E: u16 = Input::MECHANIC;
const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

fn metres(v: f32) -> Fx {
    Fx::ratio((v * 1000.0) as i32, 1000)
}

fn at(x: f32, y: f32, z: f32) -> V3 {
    V3::new(metres(x), metres(y), metres(z))
}

/// An Elementalist facing +X with clear floor in front of her, and somebody
/// far enough away to be out of everything unless a test moves them.
fn elementalist() -> World {
    let mut w = World::with_classes([Class::Elementalist, Class::Bulwark]);
    w.players[0].pos = at(-6.0, 0.0, 8.0);
    w.players[1].pos = at(11.0, 0.0, 8.0);
    w
}

fn run(w: &mut World, frames: u32, a: u16, b: u16) {
    for _ in 0..frames {
        w.advance([
            Input::looking_at(a, LOOK_RIGHT, 0),
            Input::looking_at(b, LOOK_LEFT, 0),
        ]);
    }
}

/// Take her feet off the floor without moving her.
fn aloft(w: &mut World, height: f32) {
    w.players[0].pos.y = metres(height);
    w.players[0].grounded = false;
}

/// Which move she is part-way through, whatever phase it is in.
fn doing(w: &World) -> Option<u8> {
    match w.players[0].action {
        Action::Startup { kind, .. }
        | Action::Active { kind, .. }
        | Action::Recovery { kind, .. }
        | Action::Channel { kind, .. } => Some(kind),
        _ => None,
    }
}

fn stones_of(w: &World) -> Vec<Structure> {
    let Mechanic::Structures(slots) = w.players[0].mechanic else {
        panic!("not the class that owns stones");
    };
    slots.iter().flatten().copied().collect()
}

fn shots(w: &World) -> Vec<sim::gust::Gust> {
    w.gusts.iter().flatten().copied().collect()
}

fn clouds(w: &World) -> Vec<Effect> {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.kind == EffectKind::Embers)
        .copied()
        .collect()
}

/// A cloud of embers standing in the world, as a burst would leave it.
fn cloud(owner: u8, pos: V3, radius: Fx) -> Effect {
    Effect::cast(
        EffectKind::Embers,
        owner,
        Class::Elementalist,
        e::CINDER,
        pos,
        V3::ZERO,
        radius,
    )
}

/// Press a button until the move it asks for is out of her hands, then let go.
fn throw(w: &mut World, button: u16) {
    throw_looking(w, button, 0);
}

/// The same, holding a pitch.
fn throw_looking(w: &mut World, button: u16, pitch: i16) {
    let step = |w: &mut World, bits: u16| {
        w.advance([
            Input::looking_at(bits, LOOK_RIGHT, pitch),
            Input::looking_at(0, LOOK_LEFT, 0),
        ]);
    };
    step(w, button);
    let (startup, active, _) = sim::moves::frames(Class::Elementalist, doing(w).expect("a move"));
    for _ in 0..(startup + active) {
        step(w, button);
    }
    step(w, 0);
}

/// Ask `aim` something about the world as it stands, the way the simulation
/// builds the same question.
fn with_scene<T>(w: &World, ask: impl FnOnce(&sim::aim::Scene) -> T) -> T {
    let stones = sim::stones::gather(&w.players);
    let players = w.players;
    let effects = w.effects;
    ask(&sim::aim::Scene {
        stones: &stones,
        players: &players,
        effects: &effects,
        quarry: w.monster.as_ref(),
    })
}

/// The pitch that puts the crosshair on the floor at somebody's feet -- aimed
/// the way a player does, rather than typed in. Looking at the floor is what
/// makes a shot travel *level* at her cast height (`aim::skillshot_path`), so
/// it goes through a standing body instead of over it. See
/// `tests/elementalist_air.rs`, which this is taken from.
fn crosshair_onto_the_floor_at(w: &World, target: V3, reach: Fx) -> i16 {
    (-800..0)
        .rev()
        .find_map(|step| {
            let pitch = (step * 65536 / 3600) as i16;
            let look = Input::looking_at(0, LOOK_RIGHT, pitch);
            let seen = with_scene(w, |scene| sim::aim::sight(0, look, reach, scene));
            let gap =
                V3::new(seen.at.x.sub(target.x), Fx::ZERO, seen.at.z.sub(target.z)).flat_len();
            (gap.raw() < metres(0.5).raw()).then_some(pitch)
        })
        .expect("no pitch puts the crosshair anywhere near those feet")
}

/// Fly every shot out, at a held pitch, until none is left or `most` frames
/// have passed.
fn fly_out(w: &mut World, pitch: i16, most: u32) {
    for _ in 0..most {
        if shots(w).is_empty() {
            break;
        }
        w.advance([
            Input::looking_at(0, LOOK_RIGHT, pitch),
            Input::looking_at(0, LOOK_LEFT, 0),
        ]);
    }
}

// ---------------------------------------------------------------------------
// The third click
// ---------------------------------------------------------------------------

#[test]
fn middle_click_is_the_cinder_spray_on_the_floor_and_off_it() {
    let mut w = elementalist();
    run(&mut w, 1, M, 0);
    assert_eq!(
        doing(&w),
        Some(e::CINDER),
        "standing, the third click is the spray"
    );

    let mut w = elementalist();
    aloft(&mut w, 3.0);
    run(&mut w, 1, M, 0);
    assert_eq!(
        doing(&w),
        Some(e::CINDER),
        "airborne, the third click is the same spray"
    );
}

#[test]
fn the_cinder_spray_is_a_skillshot_with_no_volume_of_its_own() {
    let m = sim::moves::get(Class::Elementalist, e::CINDER);
    assert_eq!(m.aim(), sim::aim::Kind::Skillshot);
    assert!(!m.strikes(), "the ember does the hitting, not her body");
    let mut w = elementalist();
    run(&mut w, 1, M, 0);
    let (startup, _, _) = sim::moves::frames(Class::Elementalist, e::CINDER);
    run(&mut w, startup as u32, M, 0);
    assert!(
        sim::state::hitbox(&w.players[0]).is_none(),
        "the spray puts a shot in the world and nothing out of her hands"
    );
}

#[test]
fn the_cinder_leaves_her_hand_as_an_ember_with_a_speed() {
    let mut w = elementalist();
    throw(&mut w, M);
    let out = shots(&w);
    assert_eq!(out.len(), 1, "one ember in flight");
    assert_eq!(out[0].gale, Gale::Ember);
    assert!(!out[0].lit, "an ember is fire already and is never lit");
    let first = out[0].pos;
    run(&mut w, 3, 0, 0);
    let later = shots(&w)[0].pos;
    assert!(
        later.x.raw() > first.x.raw(),
        "it travels along the line it was thrown"
    );
}

#[test]
fn the_ember_bursts_into_a_cloud_where_its_range_runs_out() {
    let mut w = elementalist();
    let from = w.players[0].pos;
    throw(&mut w, M);
    // Nothing in its way: let it fly out to the range sphere.
    for _ in 0..240 {
        if shots(&w).is_empty() {
            break;
        }
        run(&mut w, 1, 0, 0);
    }
    assert!(shots(&w).is_empty(), "the ember is spent at its range");
    let c = clouds(&w);
    assert_eq!(c.len(), 1, "and one cloud is what it left");
    let reach = sim::moves::get(Class::Elementalist, e::CINDER).reach;
    let flew = c[0].pos.x.sub(from.x);
    assert!(
        (flew.sub(reach)).abs().raw() < Fx::ONE.raw(),
        "the cloud is at the range sphere: {} m out of {}",
        flew.to_int(),
        reach.to_int()
    );
    assert_eq!(c[0].reach, t::embers_radius(), "at the spray's own radius");
    assert_eq!(c[0].owner, 0);
}

#[test]
fn an_ember_bursts_on_the_first_thing_it_meets() {
    // A stone in the lane, well short of the range.
    let mut w = elementalist();
    let stone_at = at(-2.0, 0.0, 8.0);
    sim::stones::raise(&mut w.players[0], Structure::raised(stone_at));
    run(&mut w, 30, 0, 0);
    throw(&mut w, M);
    for _ in 0..120 {
        if shots(&w).is_empty() {
            break;
        }
        run(&mut w, 1, 0, 0);
    }
    let c = clouds(&w);
    assert_eq!(c.len(), 1);
    assert!(
        c[0].pos.x.raw() < stone_at.x.raw(),
        "it burst on the near face of the stone, not past it"
    );

    // A fighter in the lane: hit, and the cloud left on them.
    let mut w = elementalist();
    w.players[1].pos = at(1.0, 0.0, 8.0);
    let full = w.players[1].health;
    throw(&mut w, M);
    for _ in 0..120 {
        if shots(&w).is_empty() {
            break;
        }
        run(&mut w, 1, 0, 0);
    }
    assert!(
        w.players[1].health < full,
        "the ember itself is a small hit"
    );
    let c = clouds(&w);
    assert_eq!(c.len(), 1, "and it bursts on them");
    assert!(
        (c[0].pos.x.sub(w.players[1].pos.x)).abs().raw() < Fx::from_int(2).raw(),
        "where they stand"
    );
}

#[test]
fn a_cloud_on_the_floor_stands_on_it_rather_than_under_it() {
    let mut w = elementalist();
    // A burst worked out below the floor is stood on the floor: the same ball,
    // now a low burning patch. Simulated by a spray aimed steeply down.
    let steep = -(65536 / 8) as i16;
    throw_looking(&mut w, M, steep);
    fly_out(&mut w, steep, 120);
    let c = clouds(&w);
    assert_eq!(c.len(), 1);
    assert!(c[0].pos.y.raw() >= 0, "never below the floor");
    assert!(
        c[0].pos.y.raw() <= t::embers_radius().raw(),
        "and no higher than its own radius: a patch, not a balloon"
    );
}

#[test]
fn a_cloud_burns_the_other_fighter_and_never_its_caster() {
    let mut w = elementalist();
    w.players[1].pos = at(-4.0, 0.0, 8.0);
    // Both of them inside it.
    let centre = at(-5.0, 0.0, 8.0);
    w.effects[0] = Some(cloud(0, centre, t::embers_radius()));
    let (hers, theirs) = (w.players[0].health, w.players[1].health);
    run(&mut w, 60, 0, 0);
    assert!(
        w.players[1].health < theirs,
        "standing in embers costs the other fighter"
    );
    assert_eq!(w.players[0].health, hers, "and never the one who made them");
    // And it goes out on its own clock.
    run(&mut w, t::embers_life() as u32 + 2, 0, 0);
    assert!(clouds(&w).is_empty(), "a cloud has a life");
}

// ---------------------------------------------------------------------------
// Lit shots
// ---------------------------------------------------------------------------

/// Throw an Air bolt at a body standing eight metres out on level ground, with
/// or without a cloud of embers in the way, and say what it dealt.
fn air_bolt_through(fire: bool) -> (World, i32) {
    let mut w = elementalist();
    w.players[1].pos = at(2.0, 0.0, 8.0);
    if fire {
        // A cloud on the line, at her cast height, well short of the target.
        w.effects[0] = Some(cloud(0, at(-2.0, 1.25, 8.0), t::embers_radius()));
    }
    aloft(&mut w, 1.0);
    let full = w.players[1].health;
    // The crosshair on their feet: the bolt flies level at her cast height
    // and meets the standing body.
    let reach = sim::moves::get(Class::Elementalist, e::AIR_BOLT).reach;
    let pitch = crosshair_onto_the_floor_at(&w, w.players[1].pos, reach);
    throw_looking(&mut w, L, pitch);
    fly_out(&mut w, pitch, 120);
    let dealt = full - w.players[1].health;
    (w, dealt)
}

#[test]
fn an_air_bolt_flown_through_fire_comes_out_lit_and_worth_more() {
    let (plain_world, plain) = air_bolt_through(false);
    let (lit_world, lit) = air_bolt_through(true);
    assert!(plain > 0, "the plain bolt lands");
    assert!(
        lit > plain,
        "the lit one is worth more: {lit} against {plain}"
    );
    let bonus = Fx::from_int(plain).mul(t::lit_bonus()).to_int();
    assert_eq!(lit, plain + bonus, "by exactly the lit bonus");
    // The plain one left nothing; the lit one burst where it landed.
    assert!(clouds(&plain_world).is_empty());
    let left = clouds(&lit_world);
    assert_eq!(
        left.len(),
        2,
        "the cloud it flew through, and the burst it left"
    );
    assert!(
        left.iter().any(|c| c.reach == t::lit_burst_radius()),
        "the burst is the small one"
    );
}

#[test]
fn a_shot_is_lit_by_a_fire_pillar_as_well_as_by_a_cloud() {
    let mut w = elementalist();
    w.players[1].pos = at(2.0, 0.0, 8.0);
    // A grown pillar standing on the line.
    let mut pillar = Effect::cast(
        EffectKind::FirePillar,
        0,
        Class::Elementalist,
        sim::state::SLOT_SPECIAL,
        at(-2.0, 0.0, 8.0),
        V3::ZERO,
        Fx::ZERO,
    );
    pillar.age = pillar.life - 1;
    w.effects[0] = Some(pillar);
    aloft(&mut w, 1.0);
    let reach = sim::moves::get(Class::Elementalist, e::AIR_BOLT).reach;
    let pitch = crosshair_onto_the_floor_at(&w, w.players[1].pos, reach);
    throw_looking(&mut w, L, pitch);
    assert_eq!(shots(&w).len(), 1);
    // A few frames of flight is enough to cross the pillar's base.
    for _ in 0..30 {
        if shots(&w).is_empty() || shots(&w)[0].lit {
            break;
        }
        w.advance([
            Input::looking_at(0, LOOK_RIGHT, pitch),
            Input::looking_at(0, LOOK_LEFT, 0),
        ]);
    }
    assert!(
        shots(&w).first().is_none_or(|s| s.lit),
        "the bolt is carrying fire, or has already landed it"
    );
}

#[test]
fn an_ember_is_never_lit_and_a_cloud_does_not_light_itself() {
    let mut w = elementalist();
    w.effects[0] = Some(cloud(0, at(-2.0, 1.25, 8.0), t::embers_radius()));
    throw(&mut w, M);
    for _ in 0..30 {
        if shots(&w).is_empty() {
            break;
        }
        assert!(!shots(&w)[0].lit);
        run(&mut w, 1, 0, 0);
    }
}

// ---------------------------------------------------------------------------
// The Gale and the field
// ---------------------------------------------------------------------------

#[test]
fn the_gale_shoves_a_stone_it_passes_and_keeps_flying() {
    let mut w = elementalist();
    let stone_at = at(-1.0, 0.0, 8.0);
    sim::stones::raise(&mut w.players[0], Structure::raised(stone_at));
    run(&mut w, 30, 0, 0);
    let before = stones_of(&w)[0].at;
    aloft(&mut w, 1.0);
    let reach = sim::moves::get(Class::Elementalist, e::GALE).reach;
    let pitch = crosshair_onto_the_floor_at(&w, at(3.0, 0.0, 8.0), reach);
    throw_looking(&mut w, R, pitch);
    // Fly it past the stone.
    fly_out(&mut w, pitch, 60);
    let after = stones_of(&w)[0].at;
    assert!(
        after.x.raw() > before.x.add(Fx::ratio(1, 2)).raw(),
        "the stone was shoved along the disc's travel: {} to {}",
        before.x.to_int(),
        after.x.to_int()
    );
    let flying = shots(&w);
    assert!(
        flying.is_empty() || flying[0].pos.x.raw() > stone_at.x.raw(),
        "and the disc was not stopped by it"
    );
}

#[test]
fn the_gale_shoves_each_stone_once() {
    let mut w = elementalist();
    sim::stones::raise(&mut w.players[0], Structure::raised(at(-1.0, 0.0, 8.0)));
    run(&mut w, 30, 0, 0);
    aloft(&mut w, 1.0);
    let reach = sim::moves::get(Class::Elementalist, e::GALE).reach;
    let pitch = crosshair_onto_the_floor_at(&w, at(3.0, 0.0, 8.0), reach);
    throw_looking(&mut w, R, pitch);
    let mut pushed_frames = 0;
    let mut last = stones_of(&w)[0].vel.x;
    for _ in 0..60 {
        if shots(&w).is_empty() {
            break;
        }
        w.advance([
            Input::looking_at(0, LOOK_RIGHT, pitch),
            Input::looking_at(0, LOOK_LEFT, 0),
        ]);
        let now = stones_of(&w)[0].vel.x;
        if now.raw() > last.raw() {
            pushed_frames += 1;
        }
        last = now;
    }
    assert!(
        pushed_frames <= 1,
        "one shove, not one a frame: {pushed_frames}"
    );
}

#[test]
fn the_four_new_inputs_are_pressed_buttons_and_the_word_is_full() {
    for bit in [Input::SIDE_A, Input::SIDE_B, Input::KEY_F, Input::KEY_R] {
        assert!(
            Input::PRESSES & bit != 0,
            "a press, read on the frame it goes down"
        );
    }
    let all = Input::LEFT
        | Input::RIGHT
        | Input::MIDDLE
        | Input::SPECIAL
        | Input::SHIFT
        | Input::W
        | Input::A
        | Input::S
        | Input::D
        | Input::SPACE
        | Input::CROUCH
        | Input::MECHANIC
        | Input::SIDE_A
        | Input::SIDE_B
        | Input::KEY_F
        | Input::KEY_R;
    assert_eq!(
        all,
        u16::MAX,
        "sixteen buttons in sixteen bits, none shared"
    );
}

#[test]
fn the_mechanic_key_still_raises_a_stone_with_the_new_bits_held() {
    // The new bits mean nothing to the mechanic path: a stray side button
    // held while `E` is pressed changes nothing.
    let mut w = elementalist();
    run(&mut w, 1, E | Input::SIDE_A | Input::KEY_R, 0);
    assert_eq!(stones_of(&w).len(), 1);
}
