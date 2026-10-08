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
use sim::state::{Action, NO_STONE, SLOT_COMMITTED, SLOT_SPECIAL};
use sim::stones::Phase;
use sim::tuning as t;
use sim::{Class, Fx, Input, V3, World};

const L: u16 = Input::LEFT;
const R: u16 = Input::RIGHT;
const M: u16 = Input::MIDDLE;
const E: u16 = Input::MECHANIC;
const Q: u16 = Input::SPECIAL;
const F: u16 = Input::KEY_F;
const RK: u16 = Input::KEY_R;
const SIDE_B: u16 = Input::SIDE_B;
const SHIFT: u16 = Input::SHIFT;
const W: u16 = Input::W;
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
        quarry: &w.monsters,
        critters: &sim::critter::Critters::NONE,
        arena: &w.terrain(),
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

// ---------------------------------------------------------------------------
// The charges -- M2. See `moves::Charge`.
// ---------------------------------------------------------------------------

fn pillars(w: &World) -> Vec<Effect> {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.kind == EffectKind::FirePillar)
        .copied()
        .collect()
}

fn scars(w: &World) -> Vec<Effect> {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.kind == EffectKind::Rough)
        .copied()
        .collect()
}

/// Hold `Q` for `frames` past the pillar's startup, at a pitch, then let go
/// and run the active frames. `frames` of zero is a tap.
fn strike(w: &mut World, pitch: i16, hold: u16) {
    let (startup, active, _) = sim::moves::frames(Class::Elementalist, SLOT_SPECIAL);
    let step = |w: &mut World, bits: u16| {
        w.advance([
            Input::looking_at(bits, LOOK_RIGHT, pitch),
            Input::looking_at(0, LOOK_LEFT, 0),
        ]);
    };
    if hold == 0 {
        step(w, Q);
        for _ in 0..(startup + active) {
            step(w, 0);
        }
        return;
    }
    // Hold until the channel has counted `hold` frames, then let go. Counted
    // off the action rather than off the frames pressed, because the hold
    // begins on the frame after the startup ends and a test that guessed
    // that frame would be asserting the countdown's arithmetic.
    for _ in 0..(startup + hold + 4) {
        if matches!(w.players[0].action, Action::Channel { held, .. } if held >= hold) {
            break;
        }
        step(w, Q);
    }
    for _ in 0..(active + 1) {
        step(w, 0);
    }
}

#[test]
fn a_tap_of_q_is_the_pillar_as_built() {
    let mut w = elementalist();
    let pitch = crosshair_onto_the_floor_at(&w, at(-2.0, 0.0, 8.0), t::raise_reach());
    strike(&mut w, pitch, 0);
    let out = pillars(&w);
    assert_eq!(out.len(), 1, "the pillar stands");
    assert_eq!(
        out[0].life,
        t::pillar_life(),
        "with its whole burn ahead of it"
    );
    assert!(
        !matches!(w.players[0].action, Action::Channel { .. }),
        "and nothing was held"
    );
}

#[test]
fn q_held_past_the_startup_is_a_hold_with_the_aim_live() {
    let mut w = elementalist();
    let (startup, _, _) = sim::moves::frames(Class::Elementalist, SLOT_SPECIAL);
    run(&mut w, (startup + 5) as u32, Q, 0);
    assert!(
        matches!(w.players[0].action, Action::Channel { kind: SLOT_SPECIAL, held } if held >= 2),
        "still holding: {:?}",
        w.players[0].action
    );
    assert!(pillars(&w).is_empty(), "nothing has been placed yet");
    // The aim is live: turn, and the marker turns with it.
    let before = w.players[0].aim_path.to;
    for _ in 0..3 {
        w.advance([
            Input::looking_at(Q, 1 << 14, 0),
            Input::looking_at(0, LOOK_LEFT, 0),
        ]);
    }
    let after = w.players[0].aim_path.to;
    assert!(before != after, "a held Strike is still being aimed");
}

#[test]
fn the_strike_is_worth_exactly_the_burn_it_takes_off_the_pillar() {
    let m = sim::moves::get(Class::Elementalist, SLOT_SPECIAL);
    let cap = m.channel;
    assert!(cap > 0, "the pillar charges");
    let burn = t::pillar_burn_total();
    let mut last_hit = 0;
    let mut last_life = u16::MAX;
    for hold in [0u16, cap / 4, cap / 2, cap] {
        let mut w = elementalist();
        // The other fighter standing exactly where the pillar lands.
        let spot = at(-2.0, 0.0, 8.0);
        w.players[1].pos = spot;
        let full = w.players[1].health;
        let pitch = crosshair_onto_the_floor_at(&w, spot, m.reach);
        strike(&mut w, pitch, hold);
        let hit = full - w.players[1].health;
        let want = m.damage
            + Fx::from_int(burn)
                .mul(Fx::ratio(hold as i32, cap as i32))
                .mul(t::strike_worth())
                .to_int();
        assert!(
            (hit - want).abs() <= 1,
            "held {hold} of {cap}: hit for {hit}, expected {want}"
        );
        assert!(hit >= last_hit, "the Strike grows with the hold");
        last_hit = hit;
        let life = pillars(&w).first().map_or(0, |p| p.life);
        assert!(
            life <= last_life,
            "and the pillar left behind shrinks: {life} after {last_life}"
        );
        last_life = life;
        if hold == cap {
            assert!(
                pillars(&w).is_empty(),
                "a full hold leaves nothing standing"
            );
        }
        if hold == 0 {
            assert_eq!(life, t::pillar_life(), "a tap leaves the whole pillar");
        }
    }
    assert!(
        last_hit > m.damage,
        "a full Strike is worth more than the pillar's own hit"
    );
}

#[test]
fn a_hit_during_the_hold_ends_it_and_nothing_is_placed() {
    let mut w = elementalist();
    // A Bulwark standing in her face, ready to bash.
    w.players[1].pos = at(-4.6, 0.0, 8.0);
    let (startup, _, _) = sim::moves::frames(Class::Elementalist, SLOT_SPECIAL);
    run(&mut w, (startup + 3) as u32, Q, 0);
    assert!(matches!(w.players[0].action, Action::Channel { .. }));
    // He bashes; she keeps holding.
    run(&mut w, 1, Q, L);
    for _ in 0..20 {
        if w.players[0].action.stunned() {
            break;
        }
        run(&mut w, 1, Q, L);
    }
    assert!(
        w.players[0].action.stunned(),
        "hit out of the hold: {:?}",
        w.players[0].action
    );
    run(&mut w, 30, 0, 0);
    assert!(
        pillars(&w).is_empty(),
        "a Strike that was interrupted lands nothing"
    );
}

#[test]
fn a_tap_of_e_raises_a_stone_that_erupts_where_it_was_raised() {
    let mut w = elementalist();
    run(&mut w, 1, E, 0);
    let raised = stones_of(&w)[0].at;
    run(&mut w, 40, 0, 0);
    let s = stones_of(&w);
    assert_eq!(s.len(), 1);
    assert_eq!(s[0].phase(), Phase::Standing, "an ordinary raise");
    assert_eq!(s[0].at.x, raised.x, "where it was raised");
    assert_eq!(w.players[0].charging_stone, NO_STONE);
    assert!(scars(&w).is_empty(), "and no crack ran");
}

#[test]
fn e_held_past_the_churn_keeps_the_stone_churning_and_becomes_the_crack_hold() {
    let mut w = elementalist();
    run(&mut w, 30, E, 0);
    let s = stones_of(&w);
    assert_eq!(s.len(), 1);
    assert_eq!(s[0].phase(), Phase::Churning, "held under the floor");
    assert!(
        matches!(
            w.players[0].action,
            Action::Channel {
                kind: SLOT_COMMITTED,
                ..
            }
        ),
        "and the hold is Fissure's: {:?}",
        w.players[0].action
    );
    assert_ne!(w.players[0].charging_stone, NO_STONE);
}

#[test]
fn letting_go_races_the_crack_and_the_stone_erupts_at_its_end() {
    let mut w = elementalist();
    run(&mut w, 40, E, 0);
    let held_at = stones_of(&w)[0].at;
    let Action::Channel { held, .. } = w.players[0].action else {
        panic!("not holding");
    };
    let m = sim::moves::get(Class::Elementalist, SLOT_COMMITTED);
    let want = m.reach_after(held);
    // Let go: the move starts, and the crack comes out on its first active frame.
    run(&mut w, 1, 0, 0);
    assert!(
        matches!(
            w.players[0].action,
            Action::Startup {
                kind: SLOT_COMMITTED,
                ..
            }
        ),
        "released into Fissure: {:?}",
        w.players[0].action
    );
    run(&mut w, (m.startup + 1) as u32, 0, 0);
    let s = stones_of(&w);
    assert_eq!(s.len(), 1, "the same stone");
    let ran = s[0].at.x.sub(held_at.x);
    assert!(
        (ran.sub(want)).abs().raw() < Fx::ONE.raw(),
        "the stone is at the crack's end: {} m out of {}",
        ran.to_int(),
        want.to_int()
    );
    assert_eq!(w.players[0].charging_stone, NO_STONE, "let go");
    let scar = scars(&w);
    assert_eq!(scar.len(), 1, "the crack's line is rough terrain");
    assert!((scar[0].reach.sub(want)).abs().raw() < Fx::ONE.raw());
    run(&mut w, 30, 0, 0);
    assert_eq!(
        stones_of(&w)[0].phase(),
        Phase::Standing,
        "and it erupted there"
    );
}

#[test]
fn the_crack_stops_at_the_first_body_and_hits_it() {
    let mut w = elementalist();
    run(&mut w, 1, E, 0);
    let raised = stones_of(&w)[0].at;
    // Somebody standing four metres past the stone, on the crack's line.
    let body = V3::new(raised.x.add(Fx::from_int(4)), Fx::ZERO, raised.z);
    w.players[1].pos = body;
    let full = w.players[1].health;
    let m = sim::moves::get(Class::Elementalist, SLOT_COMMITTED);
    // Nearly the longest hold: at the cap it lets itself go, so stop short.
    for _ in 0..(m.channel as u32 + 40) {
        if matches!(w.players[0].action, Action::Channel { held, .. } if held + 2 >= m.channel) {
            break;
        }
        run(&mut w, 1, E, 0);
    }
    assert!(matches!(w.players[0].action, Action::Channel { .. }));
    run(&mut w, 1, 0, 0);
    run(&mut w, (m.startup + 1) as u32, 0, 0);
    assert_eq!(full - w.players[1].health, m.damage, "the crack is the hit");
    assert!(w.players[1].action.stunned(), "and it staggers");
    let s = stones_of(&w)[0];
    assert!(
        s.at.x.sub(body.x).abs().raw() < Fx::from_int(2).raw(),
        "the stone came up at them, not past them: {} against {}",
        s.at.x.to_int(),
        body.x.to_int()
    );
    let scar = scars(&w)[0];
    assert!(
        scar.reach.raw() < m.reach.raw(),
        "the scar stops where the crack did"
    );
}

#[test]
fn the_crack_runs_further_the_longer_the_hold() {
    let mut lengths = Vec::new();
    for extra in [2u32, 25, 50] {
        let mut w = elementalist();
        run(&mut w, 1, E, 0);
        // Past the churn, then `extra` more.
        while !matches!(w.players[0].action, Action::Channel { .. }) {
            run(&mut w, 1, E, 0);
        }
        run(&mut w, extra, E, 0);
        run(&mut w, 1, 0, 0);
        let m = sim::moves::get(Class::Elementalist, SLOT_COMMITTED);
        run(&mut w, (m.startup + 1) as u32, 0, 0);
        lengths.push(scars(&w)[0].reach);
    }
    assert!(lengths[0].raw() < lengths[1].raw() && lengths[1].raw() < lengths[2].raw());
}

/// The crack runs from the stone **toward the spot the crosshair is on**, not
/// along the yaw of her look. From a stone raised straight ahead, a look
/// turned a sixteenth of a turn to the side and down onto the floor puts the
/// crosshair on a spot off to that side; the crack goes along the line from
/// the stone through it. Along her yaw it would have run parallel to that
/// line, past the spot by the stone's whole offset from her.
#[test]
fn the_crack_runs_toward_the_spot_the_crosshair_is_on() {
    let mut w = elementalist();
    run(&mut w, 1, E, 0);
    let held_at = stones_of(&w)[0].at;
    let look = |bits: u16| Input::looking_at(bits, 1 << 12, -3000);
    let step =
        |w: &mut World, bits: u16| w.advance([look(bits), Input::looking_at(0, LOOK_LEFT, 0)]);
    while !matches!(w.players[0].action, Action::Channel { .. }) {
        step(&mut w, E);
    }
    for _ in 0..20 {
        step(&mut w, E);
    }
    let m = sim::moves::get(Class::Elementalist, SLOT_COMMITTED);
    let seen = with_scene(&w, |scene| {
        sim::aim::sight(0, look(0), Fx::from_int(40), scene)
    });
    assert_eq!(
        seen.met,
        sim::aim::Met::Ground,
        "the crosshair is on the floor"
    );
    let spot = seen.at;
    step(&mut w, 0);
    for _ in 0..(m.startup + 1) {
        step(&mut w, 0);
    }
    assert_eq!(scars(&w).len(), 1, "the crack ran");
    let end = stones_of(&w)[0].at;
    let ran = V3::new(end.x.sub(held_at.x), Fx::ZERO, end.z.sub(held_at.z));
    let want = V3::new(spot.x.sub(held_at.x), Fx::ZERO, spot.z.sub(held_at.z));
    let (ran, want) = (ran.normalized(), want.normalized());
    let off = ran.x.mul(want.z).sub(ran.z.mul(want.x)).abs();
    assert!(
        off.raw() < metres(0.02).raw() && ran.dot(want).raw() > 0,
        "the crack ran along ({}, {}), and the line through the crosshair is ({}, {})",
        ran.x.raw(),
        ran.z.raw(),
        want.x.raw(),
        want.z.raw(),
    );
    // And that line is not her yaw, or this proves nothing.
    let yaw = V3::from_turns(Fx::ratio(1, 16));
    let apart = yaw.x.mul(want.z).sub(yaw.z.mul(want.x)).abs();
    assert!(
        apart.raw() > metres(0.1).raw(),
        "the spot is not on her yaw from the stone"
    );
}

#[test]
fn rough_terrain_slows_whoever_crosses_it_and_never_her() {
    let mut w = elementalist();
    // A scar from her feet out along +X, ten metres.
    let from = V3::new(w.players[0].pos.x, Fx::ZERO, w.players[0].pos.z);
    w.effects[0] = Some(Effect::cast(
        EffectKind::Rough,
        0,
        Class::Elementalist,
        SLOT_COMMITTED,
        from,
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        Fx::from_int(10),
    ));
    w.players[1].pos = at(-2.0, 0.0, 8.0);
    run(&mut w, 2, 0, 0);
    assert!(w.players[1].slowed > 0, "the other fighter is slowed on it");
    assert_eq!(w.players[1].slow_mul, t::rough_slow());
    assert_eq!(w.players[0].slowed, 0, "she is not");
    // Off the line, nothing.
    w.players[1].pos = at(-2.0, 0.0, 12.0);
    run(&mut w, t::slow_frames() as u32 + 2, 0, 0);
    assert_eq!(w.players[1].slowed, 0);
    // And it wears off.
    run(&mut w, t::rough_life() as u32 + 2, 0, 0);
    assert!(scars(&w).is_empty());
}

#[test]
fn anything_else_ends_the_hold_and_the_stone_erupts_where_it_stands() {
    let mut w = elementalist();
    run(&mut w, 30, E, 0);
    assert!(matches!(w.players[0].action, Action::Channel { .. }));
    let held_at = stones_of(&w)[0].at;
    // She is hit out of it: the Bulwark walks in and bashes.
    w.players[1].pos = at(-4.6, 0.0, 8.0);
    for _ in 0..25 {
        run(&mut w, 1, E, L);
        if w.players[0].action.stunned() {
            break;
        }
    }
    assert!(w.players[0].action.stunned());
    run(&mut w, 40, 0, 0);
    assert_eq!(w.players[0].charging_stone, NO_STONE);
    assert_eq!(
        stones_of(&w)[0].at.x,
        held_at.x,
        "the stone erupted where it was held"
    );
    assert!(scars(&w).is_empty(), "no crack ran");
}

// ---------------------------------------------------------------------------
// Air on her body -- M3. See `moves::elementalist::UPDRAFT`.
// ---------------------------------------------------------------------------

fn effects_of(w: &World, kind: EffectKind) -> Vec<Effect> {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.kind == kind)
        .copied()
        .collect()
}

/// A lifter of `class` standing beside her, and how high the Updraft takes
/// them at the top of what it gave.
fn lifted_apex(class: Class) -> Fx {
    let mut w = World::with_classes([Class::Elementalist, class]);
    w.players[0].pos = at(-6.0, 0.0, 8.0);
    w.players[1].pos = at(-5.0, 0.0, 8.0);
    throw(&mut w, F);
    let mut apex = Fx::ZERO;
    for _ in 0..90 {
        run(&mut w, 1, 0, 0);
        apex = apex.max(w.players[1].pos.y);
    }
    apex
}

#[test]
fn f_is_the_updraft_standing_and_the_downdraft_in_the_air() {
    let mut w = elementalist();
    run(&mut w, 1, F, 0);
    assert_eq!(doing(&w), Some(e::UPDRAFT));
    let mut w = elementalist();
    aloft(&mut w, 3.0);
    run(&mut w, 1, F, 0);
    assert_eq!(doing(&w), Some(e::DOWNDRAFT));
}

#[test]
fn the_updraft_lifts_her_and_whoever_is_in_the_column() {
    let mut w = elementalist();
    w.players[1].pos = at(-5.0, 0.0, 8.0);
    throw(&mut w, F);
    assert!(
        !w.players[0].grounded && w.players[0].vel.y.raw() > 0,
        "she goes up"
    );
    assert!(
        !w.players[1].grounded && w.players[1].vel.y.raw() > 0,
        "and so does the body beside her: {:?}",
        w.players[1].vel
    );
    let column = effects_of(&w, EffectKind::Updraft);
    assert_eq!(column.len(), 1, "the column stands where she cast it");
    assert_eq!(column[0].reach, t::draft_radius());
    // Somebody outside it is untouched.
    let mut w = elementalist();
    w.players[1].pos = at(0.0, 0.0, 8.0);
    throw(&mut w, F);
    assert!(
        w.players[1].grounded,
        "six metres away is out of the column"
    );
}

#[test]
fn everybody_rises_by_their_own_weight() {
    let heavy = lifted_apex(Class::Bulwark);
    let light = lifted_apex(Class::DualMage);
    assert!(heavy.raw() > 0 && light.raw() > 0, "both are lifted");
    assert!(
        light.raw() > heavy.raw(),
        "the Dual mage goes higher than the Bulwark: {} against {}",
        light.to_int(),
        heavy.to_int()
    );
}

#[test]
fn the_updraft_lofts_a_stone_in_it_and_not_one_outside() {
    let mut w = elementalist();
    // Two metres out: inside the column, and clear of her own feet -- a stone
    // raised under her would carry her up and turn the press into a Downdraft.
    sim::stones::raise(&mut w.players[0], Structure::raised(at(-4.0, 0.0, 8.0)));
    sim::stones::raise(&mut w.players[0], Structure::raised(at(2.0, 0.0, 8.0)));
    run(&mut w, 30, 0, 0);
    assert!(w.players[0].grounded);
    throw(&mut w, F);
    run(&mut w, 2, 0, 0);
    let s = stones_of(&w);
    assert!(
        s[0].at.y.raw() > 0 || s[0].vel.y.raw() > 0,
        "the near stone is lofted"
    );
    assert_eq!(s[1].at.y.raw(), 0, "the far one stays put");
    assert_eq!(s[1].vel.y.raw(), 0);
}

#[test]
fn the_downdraft_drives_her_down_and_the_column_follows_her() {
    let mut w = elementalist();
    aloft(&mut w, 6.0);
    // Against a plain fall over the same frames: the drive is a speed the
    // fall cap then argues with, so what is asserted is that she is falling
    // faster than she would have been.
    let mut plain = w.clone();
    throw(&mut w, F);
    let (startup, active, _) = sim::moves::frames(Class::Elementalist, e::DOWNDRAFT);
    run(&mut plain, (startup + active + 2) as u32, 0, 0);
    assert!(
        w.players[0].vel.y.raw() < plain.players[0].vel.y.raw(),
        "driven down harder than a fall: {:?} against {:?}",
        w.players[0].vel,
        plain.players[0].vel
    );
    let before = effects_of(&w, EffectKind::Downdraft);
    assert_eq!(before.len(), 1);
    run(&mut w, 3, 0, 0);
    let after = effects_of(&w, EffectKind::Downdraft);
    assert_eq!(after.len(), 1);
    assert!(
        after[0].pos.y.raw() < before[0].pos.y.raw(),
        "under her feet, live"
    );
}

#[test]
fn the_downdraft_spikes_an_airborne_body_in_it() {
    let mut w = elementalist();
    aloft(&mut w, 6.0);
    w.players[1].pos = at(-5.0, 4.0, 8.0);
    w.players[1].grounded = false;
    throw(&mut w, F);
    assert!(
        w.players[1].vel.y.raw() < 0,
        "driven down: {:?}",
        w.players[1].vel
    );
    assert!(
        w.players[1].slam.raw() > 0,
        "and it is a spike, so the landing collects"
    );
}

#[test]
fn landing_while_it_blows_breaks_the_air_outward() {
    let mut w = elementalist();
    aloft(&mut w, 1.5);
    // Somebody standing two metres from where she will land.
    w.players[1].pos = at(-4.0, 0.0, 8.0);
    let before = w.players[1].pos;
    throw(&mut w, F);
    for _ in 0..60 {
        run(&mut w, 1, 0, 0);
        if w.players[0].grounded {
            break;
        }
    }
    assert!(w.players[0].grounded, "she came down");
    run(&mut w, 1, 0, 0);
    assert!(
        effects_of(&w, EffectKind::Downdraft).is_empty(),
        "the column is spent"
    );
    assert_eq!(
        effects_of(&w, EffectKind::AirRing).len(),
        1,
        "and the air broke outward"
    );
    run(&mut w, 12, 0, 0);
    assert!(
        w.players[1].pos.x.raw() > before.x.add(Fx::ratio(1, 2)).raw(),
        "shoved away from her: {} from {}",
        w.players[1].pos.x.to_int(),
        before.x.to_int()
    );
    let full = w.players[1].full_health();
    assert_eq!(w.players[1].health, full, "no damage in the air ring");
}

#[test]
fn landing_into_fire_puts_it_out_and_sends_a_ring_of_fire_outward() {
    let mut w = elementalist();
    // A grown pillar under where she will land.
    let mut pillar = Effect::cast(
        EffectKind::FirePillar,
        1,
        Class::Elementalist,
        SLOT_SPECIAL,
        at(-6.0, 0.0, 8.0),
        V3::ZERO,
        Fx::ZERO,
    );
    pillar.age = pillar.life / 2;
    w.effects[0] = Some(pillar);
    w.players[1].pos = at(-2.5, 0.0, 8.0);
    let full = w.players[1].health;
    aloft(&mut w, 2.0);
    throw(&mut w, F);
    for _ in 0..60 {
        run(&mut w, 1, 0, 0);
        if w.players[0].grounded {
            break;
        }
    }
    run(&mut w, 1, 0, 0);
    assert!(pillars(&w).is_empty(), "the pillar went out");
    let ring = effects_of(&w, EffectKind::FireRing);
    assert_eq!(ring.len(), 1, "and a ring of fire left her feet");
    // It races out: the body three and a half metres off is reached within
    // its reach, hit once, and shoved outward.
    let before = w.players[1].pos.x;
    run(&mut w, ring[0].life as u32 + 2, 0, 0);
    assert!(w.players[1].health < full, "the ring burns what it passes");
    assert_eq!(full - w.players[1].health, t::fire_ring_damage(), "once");
    assert!(
        w.players[1].pos.x.raw() > before.raw(),
        "and shoves outward"
    );
    assert!(
        effects_of(&w, EffectKind::FireRing).is_empty(),
        "then it is gone"
    );
}

#[test]
fn a_landing_after_the_column_has_died_bursts_nothing() {
    let mut w = elementalist();
    aloft(&mut w, 12.0);
    throw(&mut w, F);
    // Hang her up there until the column is spent, then let her fall.
    for _ in 0..(t::draft_life() as u32 + 2) {
        w.players[0].pos.y = metres(12.0);
        w.players[0].vel.y = Fx::ZERO;
        run(&mut w, 1, 0, 0);
    }
    assert!(effects_of(&w, EffectKind::Downdraft).is_empty());
    for _ in 0..120 {
        run(&mut w, 1, 0, 0);
        if w.players[0].grounded {
            break;
        }
    }
    run(&mut w, 1, 0, 0);
    assert!(effects_of(&w, EffectKind::AirRing).is_empty());
    assert!(effects_of(&w, EffectKind::FireRing).is_empty());
}

#[test]
fn a_standing_stone_under_the_downdraft_is_pressed_into_the_floor() {
    let mut w = elementalist();
    sim::stones::raise(&mut w.players[0], Structure::raised(at(-6.0, 0.0, 8.0)));
    run(&mut w, 30, 0, 0);
    // Directly above it, so it stands in the column.
    aloft(&mut w, 5.0);
    w.players[0].pos.x = metres(-6.0);
    throw(&mut w, F);
    run(&mut w, 2, 0, 0);
    assert!(stones_of(&w).is_empty(), "pressed into the floor");
    assert_eq!(scars(&w).len(), 1, "and broken ground where it stood");
}

// ---------------------------------------------------------------------------
// Fire on earth, and the two Quakes -- M4.
// ---------------------------------------------------------------------------

fn quakes(w: &World) -> Vec<Effect> {
    effects_of(w, EffectKind::Quake)
}

#[test]
fn the_second_side_button_is_quake_and_r_is_tremor() {
    let mut w = elementalist();
    run(&mut w, 1, SIDE_B, 0);
    assert_eq!(doing(&w), Some(e::QUAKE));
    let mut w = elementalist();
    run(&mut w, 1, RK, 0);
    assert_eq!(doing(&w), Some(e::TREMOR));
    // Off the floor, R carries nothing yet; the side button still quakes.
    let mut w = elementalist();
    aloft(&mut w, 3.0);
    run(&mut w, 1, RK, 0);
    assert_eq!(doing(&w), None);
    run(&mut w, 1, SIDE_B, 0);
    assert_eq!(doing(&w), Some(e::QUAKE));
}

#[test]
fn a_quake_shakes_where_the_crosshair_is_and_staggers_only_what_moves() {
    let mut w = elementalist();
    let spot = at(-1.0, 0.0, 8.0);
    let pitch = crosshair_onto_the_floor_at(
        &w,
        spot,
        sim::moves::get(Class::Elementalist, e::QUAKE).reach,
    );
    throw_looking(&mut w, SIDE_B, pitch);
    let q = quakes(&w);
    assert_eq!(q.len(), 1, "the patch is down");
    assert!(
        q[0].pos.x.sub(spot.x).abs().raw() < Fx::ONE.raw(),
        "where the crosshair was: {} against {}",
        q[0].pos.x.to_int(),
        spot.x.to_int()
    );
    // A body standing still in it is fine.
    w.players[1].pos = spot;
    run(&mut w, 6, 0, 0);
    assert!(
        !w.players[1].action.stunned(),
        "standing still is the answer"
    );
    // Walking through it is not.
    run(&mut w, 6, 0, Input::W);
    assert!(w.players[1].action.stunned(), "a mover is staggered");
}

#[test]
fn a_quake_erupts_when_the_shake_ends_and_leaves_a_stone() {
    let mut w = elementalist();
    let spot = at(-1.0, 0.0, 8.0);
    let pitch = crosshair_onto_the_floor_at(
        &w,
        spot,
        sim::moves::get(Class::Elementalist, e::QUAKE).reach,
    );
    throw_looking(&mut w, SIDE_B, pitch);
    w.players[1].pos = spot;
    let full = w.players[1].health;
    run(&mut w, t::quake_shake() as u32 + 2, 0, 0);
    assert!(quakes(&w).is_empty(), "the shake is over");
    assert_eq!(
        full - w.players[1].health,
        t::quake_damage(),
        "and it erupted on them"
    );
    let s = stones_of(&w);
    assert_eq!(s.len(), 1, "leaving a stone");
    assert!(
        s[0].at.x.sub(spot.x).abs().raw() < Fx::ONE.raw(),
        "at its centre"
    );
}

#[test]
fn a_tremor_is_a_quake_on_her_own_feet_and_the_stone_lifts_her() {
    let mut w = elementalist();
    let feet = w.players[0].pos;
    throw(&mut w, RK);
    let q = quakes(&w);
    assert_eq!(q.len(), 1);
    assert!(
        q[0].pos.x.sub(feet.x).abs().raw() < Fx::ratio(1, 2).raw(),
        "centred on her"
    );
    run(&mut w, t::quake_shake() as u32 + 30, 0, 0);
    let s = stones_of(&w);
    assert_eq!(s.len(), 1, "the stone came up");
    assert!(
        w.players[0].pos.y.raw() > Fx::ONE.raw(),
        "under her, and took her with it: {} m up",
        w.players[0].pos.y.to_int()
    );
}

#[test]
fn a_pillar_aimed_at_her_stone_lands_on_the_floor_behind_it() {
    // From play, 2026-10-02: raise a stone, then a pillar past it, and the
    // pillar came up on the stone's lid -- the crosshair's ray met the stone
    // on its way to the floor, and a placement took the stone for ground. At
    // close quarters a stone fills the screen. The placement ray now goes
    // through her stones (`aim::grounded_path`), so the pillar goes to the
    // floor the crosshair would be on if the stone were not there.
    let mut w = elementalist();
    let spot = at(-3.0, 0.0, 8.0);
    let mut stone = Structure::raised(spot);
    stone.age = stone.rise + 1;
    let mut slots = [None; sim::class::MAX_STRUCTURES];
    slots[0] = Some(stone);
    w.players[0].mechanic = Mechanic::Structures(slots);
    let reach = sim::moves::get(Class::Elementalist, SLOT_SPECIAL).reach;
    // The crosshair on the stone itself: its face, or its lid.
    let pitch = (-800..0)
        .rev()
        .find_map(|step| {
            let pitch = (step * 65536 / 3600) as i16;
            let look = Input::looking_at(0, LOOK_RIGHT, pitch);
            let seen = with_scene(&w, |scene| sim::aim::sight(0, look, reach, scene));
            let gap = V3::new(seen.at.x.sub(spot.x), Fx::ZERO, seen.at.z.sub(spot.z)).flat_len();
            (seen.at.y.raw() > metres(0.5).raw() && gap.raw() < stone.radius().raw())
                .then_some(pitch)
        })
        .expect("no pitch put the crosshair on the stone");
    strike(&mut w, pitch, 0);
    let p = pillars(&w);
    assert_eq!(p.len(), 1, "no pillar came up");
    assert_eq!(
        p[0].pos.y.raw(),
        0,
        "the pillar stood {} m up, on the stone",
        p[0].pos.y.to_f32_for_render()
    );
    assert!(
        p[0].pos.x.raw() > spot.x.add(stone.radius()).raw(),
        "the pillar came up at x = {}, not past the stone at x = {}",
        p[0].pos.x.to_f32_for_render(),
        spot.x.to_f32_for_render()
    );
}

#[test]
fn a_pillar_cast_on_a_stone_lights_it_and_a_lit_stone_burns_its_stander() {
    let mut w = elementalist();
    let spot = at(-2.0, 0.0, 8.0);
    sim::stones::raise(&mut w.players[0], Structure::raised(spot));
    run(&mut w, 30, 0, 0);
    assert_eq!(stones_of(&w)[0].lit, 0);
    // On the floor at its near face: the pillar looks *through* her stones
    // since 2026-10-02 (`aim::grounded_path`), so it is lit by a pillar whose
    // footprint takes in its base rather than by one stood on its lid. A
    // metre and a half short of its centre: far enough off its face that the
    // crosshair is on the floor and not the stone, well inside the pillar.
    let short = at(-2.0 - 1.5, 0.0, 8.0);
    let pitch = crosshair_onto_the_floor_at(&w, short, t::raise_reach());
    strike(&mut w, pitch, 0);
    assert!(
        stones_of(&w)[0].lit > 0,
        "lit by the pillar cast against it"
    );
    // Somebody standing on top of it.
    let top = stones_of(&w)[0].top();
    w.players[1].pos = V3::new(spot.x, top, spot.z);
    let full = w.players[1].health;
    run(&mut w, 40, 0, 0);
    assert!(w.players[1].health < full, "a lit stone is a stove");
    // And it goes out on its own clock.
    run(&mut w, t::lit_stone_life() as u32, 0, 0);
    assert_eq!(stones_of(&w)[0].lit, 0);
}

#[test]
fn a_cinder_bursting_beside_a_stone_lights_it() {
    let mut w = elementalist();
    let spot = at(-1.0, 0.0, 8.0);
    sim::stones::raise(&mut w.players[0], Structure::raised(spot));
    run(&mut w, 30, 0, 0);
    // Thrown at the stone: the ember bursts on its near face, beside it.
    throw(&mut w, M);
    fly_out(&mut w, 0, 120);
    assert!(stones_of(&w)[0].lit > 0);
}

#[test]
fn kicking_a_lit_stone_bursts_it_into_burning_debris() {
    let mut w = elementalist();
    let spot = at(-2.0, 0.0, 8.0);
    sim::stones::raise(&mut w.players[0], Structure::raised(spot));
    run(&mut w, 30, 0, 0);
    sim::stones::light(&mut w.players, 0);
    assert!(stones_of(&w)[0].lit > 0);
    // The beam, aimed at the stone.
    let pitch =
        crosshair_onto_the_floor_at(&w, spot, sim::moves::get(Class::Elementalist, 0).reach);
    throw_looking(&mut w, L, pitch);
    assert!(stones_of(&w).is_empty(), "the stone burst");
    assert!(w.debris.iter().flatten().count() > 0, "into debris");
    assert_eq!(clouds(&w).len(), 1, "and a cloud of embers where it stood");
}

#[test]
fn an_unlit_stone_kicked_is_still_only_kicked() {
    let mut w = elementalist();
    let spot = at(-2.0, 0.0, 8.0);
    sim::stones::raise(&mut w.players[0], Structure::raised(spot));
    run(&mut w, 30, 0, 0);
    let pitch =
        crosshair_onto_the_floor_at(&w, spot, sim::moves::get(Class::Elementalist, 0).reach);
    throw_looking(&mut w, L, pitch);
    assert_eq!(stones_of(&w).len(), 1, "still there");
    assert!(stones_of(&w)[0].launched, "and moving");
    assert!(clouds(&w).is_empty());
}

// ---------------------------------------------------------------------------
// The dodge into a stone -- M5.
// ---------------------------------------------------------------------------

/// The pitch that puts the crosshair on a stone standing at `spot`.
fn crosshair_onto_the_stone_at(w: &World, spot: V3) -> i16 {
    let field = sim::stones::gather(&w.players);
    let stone = field
        .iter()
        .flatten()
        .find(|s| s.at.x == spot.x)
        .expect("a stone there");
    let middle = V3::new(
        stone.at.x,
        stone.top().sub(t::structure_height().div(Fx::from_int(2))),
        stone.at.z,
    );
    (-800..400)
        .rev()
        .find_map(|step| {
            let pitch = (step * 65536 / 3600) as i16;
            let look = Input::looking_at(0, LOOK_RIGHT, pitch);
            with_scene(w, |scene| {
                sim::aim::pointing_at(0, look, middle, t::break_lock(), scene).then_some(pitch)
            })
        })
        .expect("no pitch puts the crosshair on the stone")
}

#[test]
fn a_dodge_with_the_crosshair_on_a_stone_breaks_through_it() {
    let mut w = elementalist();
    let spot = at(-4.0, 0.0, 8.0);
    sim::stones::raise(&mut w.players[0], Structure::raised(spot));
    run(&mut w, 30, 0, 0);
    let pitch = crosshair_onto_the_stone_at(&w, spot);
    w.advance([
        Input::looking_at(SHIFT | W, LOOK_RIGHT, pitch),
        Input::looking_at(0, LOOK_LEFT, 0),
    ]);
    assert!(
        matches!(w.players[0].action, Action::Dodge { .. }),
        "she dodged: {:?}",
        w.players[0].action
    );
    assert!(stones_of(&w).is_empty(), "and the stone broke as she went");
    let scar = scars(&w);
    assert_eq!(scar.len(), 1, "leaving broken ground where it stood");
    assert!(
        scar[0].pos.x.raw() < spot.x.raw() && scar[0].pos.x.add(scar[0].reach).raw() > spot.x.raw()
    );
    assert!(clouds(&w).is_empty(), "no fire: it was not lit");
    assert_eq!(w.players[0].breaking, NO_STONE, "the mark is spent");
}

#[test]
fn breaking_through_a_lit_stone_leaves_burning_ground() {
    let mut w = elementalist();
    let spot = at(-4.0, 0.0, 8.0);
    sim::stones::raise(&mut w.players[0], Structure::raised(spot));
    run(&mut w, 30, 0, 0);
    sim::stones::light(&mut w.players, 0);
    let pitch = crosshair_onto_the_stone_at(&w, spot);
    w.advance([
        Input::looking_at(SHIFT | W, LOOK_RIGHT, pitch),
        Input::looking_at(0, LOOK_LEFT, 0),
    ]);
    assert!(stones_of(&w).is_empty());
    assert_eq!(scars(&w).len(), 1);
    assert_eq!(
        clouds(&w).len(),
        1,
        "and a cloud of embers on the broken ground"
    );
}

#[test]
fn a_dodge_pointed_elsewhere_is_the_ordinary_dodge() {
    let mut w = elementalist();
    let spot = at(-4.0, 0.0, 8.0);
    sim::stones::raise(&mut w.players[0], Structure::raised(spot));
    run(&mut w, 30, 0, 0);
    // Looking well away from the stone, and dodging the other way. The
    // camera behind her shoulder is inside the stone's column here, which is
    // exactly why the dodge has to be toward the stone as well.
    w.advance([
        Input::looking_at(SHIFT | W, LOOK_LEFT, 0),
        Input::looking_at(0, LOOK_LEFT, 0),
    ]);
    assert!(matches!(w.players[0].action, Action::Dodge { .. }));
    assert_eq!(stones_of(&w).len(), 1, "the stone stands");
    assert!(scars(&w).is_empty());
}

#[test]
fn a_stone_out_of_a_dodges_reach_is_not_broken() {
    let mut w = elementalist();
    let spot = at(2.0, 0.0, 8.0);
    sim::stones::raise(&mut w.players[0], Structure::raised(spot));
    run(&mut w, 30, 0, 0);
    let pitch = crosshair_onto_the_stone_at(&w, spot);
    w.advance([
        Input::looking_at(SHIFT | W, LOOK_RIGHT, pitch),
        Input::looking_at(0, LOOK_LEFT, 0),
    ]);
    assert!(matches!(w.players[0].action, Action::Dodge { .. }));
    assert_eq!(
        stones_of(&w).len(),
        1,
        "eight metres away is out of a dodge's reach"
    );
}

#[test]
fn the_airdodge_breaks_through_too_and_costs_the_airdodge() {
    let mut w = elementalist();
    let spot = at(-4.0, 0.0, 8.0);
    sim::stones::raise(&mut w.players[0], Structure::raised(spot));
    run(&mut w, 30, 0, 0);
    aloft(&mut w, 1.0);
    let pitch = crosshair_onto_the_stone_at(&w, spot);
    w.advance([
        Input::looking_at(SHIFT | W, LOOK_RIGHT, pitch),
        Input::looking_at(0, LOOK_LEFT, 0),
    ]);
    assert!(matches!(w.players[0].action, Action::Dodge { .. }));
    assert!(stones_of(&w).is_empty());
    assert!(w.players[0].air_dodged, "spent");
}
