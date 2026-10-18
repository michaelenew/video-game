//! The Elementalist's quality-of-life pass, 2026-10-18, from play.
//!
//! One property per thing the person asked for, so each survives tuning:
//!
//! - every fire touching a stone lights it, not only the pillar's footprint on
//!   the frame it was cast;
//! - `Q` and `E` (the Bolt and Cataclysm) act on every stone and fire on their
//!   line at once, instead of stopping at the first;
//! - Fissure's stone comes up *while it slides*, carrying whoever stands at
//!   its front -- her included -- and strikes harder the longer it was held;
//! - wind that touches fire burns: the Air ball and the drafts;
//! - she steers her own Air ball with A and D without walking out of it;
//! - half off a stone and walking, the earth jump pushes off that stone;
//! - the held direction pushes her while her Updraft blows.
//!
//! See `docs/design/kits/elementalist.md` §"The QoL pass".

use sim::class::{Mechanic, Structure};
use sim::effects::{Effect, EffectKind};
use sim::moves::elementalist as e;
use sim::moves::elementalist::keys;
use sim::state::{Action, SLOT_COMMITTED, SLOT_SPECIAL};
use sim::tuning as t;
use sim::{Class, Fx, Input, V3, World};

const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

fn metres(v: f32) -> Fx {
    Fx::ratio((v * 1000.0) as i32, 1000)
}

fn at(x: f32, y: f32, z: f32) -> V3 {
    V3::new(metres(x), metres(y), metres(z))
}

fn elementalist() -> World {
    let mut w = World::with_classes([Class::Elementalist, Class::Bulwark]);
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

fn run(w: &mut World, frames: u32, bits: u16) {
    for _ in 0..frames {
        step_looking(w, bits, 0);
    }
}

fn stones_of(w: &World) -> Vec<Structure> {
    let Mechanic::Structures(slots) = w.players[0].mechanic else {
        panic!("not the class that owns stones");
    };
    slots.iter().flatten().copied().collect()
}

/// A stone of hers standing at `base`, fully out of the floor, in `slot`.
fn stand_a_stone(w: &mut World, slot: usize, base: V3) {
    let stone = Structure {
        age: t::structure_rise(),
        ..Structure::raised(base)
    };
    let Mechanic::Structures(mut slots) = w.players[0].mechanic else {
        unreachable!()
    };
    slots[slot] = Some(stone);
    w.players[0].mechanic = Mechanic::Structures(slots);
}

fn place(w: &mut World, effect: Effect) {
    let slot = w.effects.iter().position(|s| s.is_none()).expect("room");
    w.effects[slot] = Some(effect);
}

fn of_kind(w: &World, kind: EffectKind) -> Vec<Effect> {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.kind == kind)
        .copied()
        .collect()
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

// ---------------------------------------------------------------------------
// Fire on a stone, on every contact
// ---------------------------------------------------------------------------

#[test]
fn a_stone_raised_beside_a_standing_pillar_catches() {
    let mut w = elementalist();
    // A pillar of hers already standing, and grown.
    place(
        &mut w,
        Effect::cast(
            EffectKind::FirePillar,
            0,
            Class::Elementalist,
            SLOT_SPECIAL,
            at(0.0, 0.0, 8.0),
            V3::ZERO,
            Fx::ZERO,
        ),
    );
    run(&mut w, 40, 0);
    // A stone that arrives inside its base long after it was cast.
    stand_a_stone(&mut w, 0, at(2.5, 0.0, 8.0));
    run(&mut w, 1, 0);
    assert!(
        stones_of(&w)[0].lit > 0,
        "the pillar lit a stone that came to it"
    );
    // And one well clear of it does not.
    stand_a_stone(&mut w, 1, at(-3.0, 0.0, 8.0));
    run(&mut w, 1, 0);
    assert_eq!(
        stones_of(&w)[1].lit,
        0,
        "nothing lights a stone fire never touches"
    );
}

#[test]
fn a_fire_carpet_lights_a_stone_it_is_laid_across() {
    let mut w = elementalist();
    stand_a_stone(&mut w, 0, at(0.0, 0.0, 8.0));
    // A carpet at knee height running through the stone.
    place(
        &mut w,
        Effect::cast(
            EffectKind::FireCarpet,
            0,
            Class::Elementalist,
            e::FIRE_CARPET,
            at(-3.0, 0.6, 8.0),
            V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
            Fx::from_int(6),
        ),
    );
    run(&mut w, 1, 0);
    assert!(
        stones_of(&w)[0].lit > 0,
        "a carpet through a stone lights it"
    );
}

// ---------------------------------------------------------------------------
// The two pushes act on everything on their line
// ---------------------------------------------------------------------------

/// Two stones one behind the other in front of her, and the push thrown at
/// the near one: both are acted on.
fn two_stones_in_a_row() -> World {
    let mut w = elementalist();
    stand_a_stone(&mut w, 0, at(-3.0, 0.0, 8.0));
    stand_a_stone(&mut w, 1, at(-0.5, 0.0, 8.0));
    w
}

fn push(w: &mut World, button: u16) {
    step_looking(w, button, 0);
    for _ in 0..40 {
        if doing(w).is_none() {
            break;
        }
        step_looking(w, 0, 0);
    }
}

#[test]
fn the_bolt_kicks_the_stone_behind_the_one_it_was_aimed_at() {
    let mut w = two_stones_in_a_row();
    let before: Vec<V3> = stones_of(&w).iter().map(|s| s.at).collect();
    push(&mut w, keys::WEAK_PUSH);
    let after = stones_of(&w);
    assert_eq!(after.len(), 2);
    for (s, was) in after.iter().zip(before) {
        assert!(
            s.at.x.raw() > was.x.add(metres(0.2)).raw(),
            "every stone on the line was kicked along it: {} from {}",
            s.at.x.to_int(),
            was.x.to_int()
        );
    }
}

#[test]
fn cataclysm_breaks_every_stone_on_its_line() {
    let mut w = two_stones_in_a_row();
    push(&mut w, keys::STRONG_PUSH);
    assert!(
        stones_of(&w).is_empty(),
        "both stones broke: {} left",
        stones_of(&w).len()
    );
}

#[test]
fn a_wall_still_ends_a_push() {
    // `aim::every_along` stops at the first wall and reports nothing past it.
    let w = elementalist();
    let path = sim::aim::Path {
        from: at(0.0, 1.0, 0.0),
        to: at(400.0, 1.0, 0.0),
    };
    let field = sim::stones::gather(&w.players);
    let players = w.players;
    let effects = w.effects;
    let scene = sim::aim::Scene {
        stones: &field,
        players: &players,
        effects: &effects,
        quarry: &w.monsters,
        critters: &sim::critter::Critters::NONE,
        arena: &w.terrain(),
    };
    let (met, n) = sim::aim::every_along(
        path,
        Fx::ZERO,
        0,
        &scene,
        sim::aim::Targets::none().stones().walls(),
    );
    if n > 0 {
        assert!(
            matches!(met[n - 1], Some(sim::aim::Contact::Terrain { .. })),
            "the last thing on a line that met a wall is the wall"
        );
    }
}

// ---------------------------------------------------------------------------
// Fissure: the stone comes up while it slides
// ---------------------------------------------------------------------------

/// Raise a stone at the crosshair and hold the earth click `extra` frames
/// past the churn, then let go and let the surge run out.
fn fissure(w: &mut World, pitch: i16, extra: u32) {
    fissure_from(w, pitch, pitch, extra);
}

/// The same, raising the stone at one pitch and holding it at another.
fn fissure_from(w: &mut World, raise: i16, pitch: i16, extra: u32) {
    step_looking(w, keys::EARTH, raise);
    while !matches!(w.players[0].action, Action::Channel { .. }) {
        step_looking(w, keys::EARTH, pitch);
    }
    for _ in 0..extra {
        step_looking(w, keys::EARTH, pitch);
    }
    step_looking(w, 0, pitch);
}

#[test]
fn the_stone_slides_out_along_the_line_as_it_comes_up() {
    let mut w = elementalist();
    fissure(&mut w, 0, 40);
    let held = stones_of(&w)[0].at;
    let m = sim::moves::get(Class::Elementalist, SLOT_COMMITTED);
    run(&mut w, m.startup as u32 + 2, 0);
    let surge = of_kind(&w, EffectKind::Surge);
    assert_eq!(surge.len(), 1, "letting go sent a surge");
    let early = stones_of(&w)[0].at.x;
    run(&mut w, 8, 0);
    let later = stones_of(&w)[0].at.x;
    assert!(
        later.raw() > early.raw() && early.raw() >= held.x.raw(),
        "the stone travels with the surge: {} then {}",
        early.to_int(),
        later.to_int()
    );
    run(&mut w, 120, 0);
    assert!(
        of_kind(&w, EffectKind::Surge).is_empty(),
        "and the surge ends"
    );
    let s = stones_of(&w)[0];
    assert_eq!(
        s.phase(),
        sim::stones::Phase::Standing,
        "standing where it stopped"
    );
    assert!(
        s.at.x.sub(held.x).raw() > metres(2.0).raw(),
        "out along the line it was held toward"
    );
}

/// Struck by a surge from `extra` frames of hold: what it cost the Bulwark
/// standing on its line, and how fast it left him going.
fn struck_by_a_surge(extra: u32) -> (i32, Fx) {
    let mut w = elementalist();
    // Ahead of the stone, which comes up six metres out at x = 0.
    w.players[1].pos = at(1.6, 0.0, 8.0);
    let full = w.players[1].health;
    fissure(&mut w, 0, extra);
    let mut fastest = Fx::ZERO;
    for _ in 0..80 {
        run(&mut w, 1, 0);
        let v = w.players[1].vel;
        fastest = fastest.max(V3::new(v.x, Fx::ZERO, v.z).flat_len());
    }
    (full - w.players[1].health, fastest)
}

#[test]
fn the_surge_passes_through_a_body_and_shoves_it_on() {
    let mut w = elementalist();
    // Raised at the edge of Raise's reach, six metres ahead at x = 0; he is
    // three metres on, on the line it will run.
    w.players[1].pos = at(3.0, 0.0, 8.0);
    let full = w.players[1].health;
    fissure(&mut w, 0, 55);
    run(&mut w, 120, 0);
    assert!(w.players[1].health < full, "the surge struck him");
    assert!(
        w.players[1].pos.x.raw() > metres(4.0).raw(),
        "and shoved him on along its line: he is at {}",
        w.players[1].pos.x.to_int()
    );
    let s = stones_of(&w)[0];
    assert!(
        s.at.x.raw() > metres(4.0).raw(),
        "it did not stop at him: the stone is at {}",
        s.at.x.to_int()
    );
}

#[test]
fn a_longer_hold_strikes_harder_and_throws_further() {
    let (tap_damage, tap_speed) = struck_by_a_surge(1);
    let (full_damage, full_speed) = struck_by_a_surge(55);
    assert!(tap_damage > 0, "even a tap's surge strikes");
    assert!(
        full_damage > tap_damage,
        "a full hold deals more: {full_damage} against {tap_damage}"
    );
    assert!(
        full_speed.raw() > tap_speed.raw(),
        "and throws harder: {} against {}",
        full_speed.to_int(),
        tap_speed.to_int()
    );
}

#[test]
fn cast_at_her_own_feet_the_surge_carries_her() {
    let mut w = elementalist();
    // Straight down: the stone is raised under her feet; then the hold is
    // aimed ahead, at the floor a few metres on.
    let down = -(65536 / 5) as i16;
    let ahead = -(65536 / 24) as i16;
    let from = w.players[0].pos;
    fissure_from(&mut w, down, ahead, 40);
    let mut furthest = Fx::ZERO;
    for _ in 0..60 {
        step_looking(&mut w, 0, ahead);
        furthest = furthest.max(w.players[0].pos.x.sub(from.x));
    }
    assert!(
        furthest.raw() > metres(1.5).raw(),
        "she rode it out from where she stood: {} m",
        furthest.to_int()
    );
}

// ---------------------------------------------------------------------------
// Wind on fire burns
// ---------------------------------------------------------------------------

#[test]
fn an_air_ball_rolled_through_fire_burns_whoever_it_holds() {
    let mut w = elementalist();
    // Her pillar on the ball's line, and the Bulwark standing beyond it.
    place(
        &mut w,
        Effect::cast(
            EffectKind::FirePillar,
            0,
            Class::Elementalist,
            SLOT_SPECIAL,
            at(-2.0, 0.0, 8.0),
            V3::ZERO,
            Fx::ZERO,
        ),
    );
    w.players[1].pos = at(4.0, 0.0, 8.0);
    w.players[0].pos = at(-14.0, 0.0, -8.0);
    let mut ball = Effect::cast(
        EffectKind::AirBall,
        0,
        Class::Elementalist,
        e::AIR_BALL,
        at(-5.0, 0.0, 8.0),
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        t::air_ball_radius_full(),
    );
    ball.age = 1;
    place(&mut w, ball);
    let full = w.players[1].health;
    let mut lit = false;
    for _ in 0..90 {
        run(&mut w, 1, 0);
        lit |= of_kind(&w, EffectKind::AirBall).iter().any(|b| b.lit());
    }
    assert!(lit, "the ball caught fire in the pillar");
    // The pillar burns nobody past it, so whatever he lost was the ball.
    assert!(
        w.players[1].health < full,
        "the burning ball burned the Bulwark it carried"
    );
}

#[test]
fn an_updraft_in_fire_burns_whoever_it_lifts() {
    let mut w = elementalist();
    w.players[1].pos = at(-5.6, 0.0, 8.0);
    place(
        &mut w,
        Effect::cast(
            EffectKind::Embers,
            0,
            Class::Elementalist,
            e::CINDER,
            at(-6.0, 0.0, 8.0),
            V3::ZERO,
            t::embers_radius(),
        ),
    );
    // The cloud burns on its own tick; measure what the draft adds by
    // comparing with the same cloud and no draft.
    let mut alone = w.clone();
    step_looking(&mut w, Input::SPACE | keys::WIND, 0);
    step_looking(&mut alone, Input::SPACE, 0);
    for _ in 0..30 {
        step_looking(&mut w, 0, 0);
        step_looking(&mut alone, 0, 0);
    }
    assert!(
        w.players[1].health < alone.players[1].health,
        "and burned him: {} against {}",
        w.players[1].health,
        alone.players[1].health
    );
}

// ---------------------------------------------------------------------------
// Steering her Air ball
// ---------------------------------------------------------------------------

#[test]
fn a_and_d_turn_the_ball_she_rides_and_she_stays_in_it() {
    let mut w = elementalist();
    w.players[1].pos = at(11.0, 0.0, -10.0);
    let from = w.players[0].pos;
    let mut ball = Effect::cast(
        EffectKind::AirBall,
        0,
        Class::Elementalist,
        e::AIR_BALL,
        from,
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        t::air_ball_radius_full(),
    );
    ball.age = 1;
    place(&mut w, ball);
    run(&mut w, 2, 0);
    // Twenty frames: it turns a quarter before the arena's edge.
    for _ in 0..20 {
        run(&mut w, 1, Input::D);
        let b = of_kind(&w, EffectKind::AirBall)[0];
        let p = w.players[0].pos;
        let apart = V3::new(p.x.sub(b.pos.x), Fx::ZERO, p.z.sub(b.pos.z)).flat_len();
        assert!(
            apart.raw() < b.reach.raw(),
            "holding D she stays in her ball"
        );
    }
    let b = of_kind(&w, EffectKind::AirBall)[0];
    assert!(
        b.dir.z.raw() > metres(0.3).raw(),
        "and it turned to her right: heading ({}, {})",
        b.dir.x.raw(),
        b.dir.z.raw()
    );
}

#[test]
fn the_ball_lasts_far_longer_than_it_did() {
    // A full ball rolled on open floor: the first steering pass had it gone
    // in about three seconds; the person asked for two or three times that.
    let mut w = elementalist();
    w.players[0].pos = at(-14.0, 0.0, -14.0);
    let mut ball = Effect::cast(
        EffectKind::AirBall,
        0,
        Class::Elementalist,
        e::AIR_BALL,
        at(-14.0, 0.0, 0.0),
        V3::new(Fx::ZERO, Fx::ZERO, Fx::ONE),
        t::air_ball_radius_full(),
    );
    ball.age = 1;
    place(&mut w, ball);
    let mut frames = 0;
    while !of_kind(&w, EffectKind::AirBall).is_empty() && frames < 2000 {
        run(&mut w, 1, 0);
        frames += 1;
    }
    let was = t::air_ball_radius_full()
        .div(Fx::ratio(7, 10))
        .mul(Fx::from_int(60));
    assert!(
        frames > was.to_int() * 2,
        "{frames} frames, against about {} before",
        was.to_int()
    );
}

// ---------------------------------------------------------------------------
// The earth jump off the edge of a stone
// ---------------------------------------------------------------------------

#[test]
fn walking_off_a_stones_edge_into_an_earth_jump_pushes_off_that_stone() {
    for off in [0.8f32, 1.0, 1.1] {
        let mut w = elementalist();
        stand_a_stone(&mut w, 0, at(-6.0, 0.0, 8.0));
        let top = stones_of(&w)[0].top();
        w.players[0].pos = V3::new(metres(-6.0 + off), top, metres(8.0));
        run(&mut w, 3, 0);
        assert!(w.players[0].grounded, "standing on its edge");
        // Walking on off it through the jump's wind-up.
        step_looking(&mut w, Input::SPACE | keys::EARTH | Input::W, 0);
        run(&mut w, 16, Input::W);
        assert!(
            stones_of(&w).is_empty(),
            "{off} m off centre: the stone she stood on shattered and no new one was raised ({} out)",
            stones_of(&w).len()
        );
    }
}

// ---------------------------------------------------------------------------
// The Updraft takeoff
// ---------------------------------------------------------------------------

fn updraft_from_the_floor(hold: u16) -> (Fx, Fx) {
    let mut w = elementalist();
    let from = w.players[0].pos;
    step_looking(&mut w, Input::SPACE | keys::WIND, 0);
    let mut top = Fx::ZERO;
    let mut far = Fx::ZERO;
    for _ in 0..90 {
        step_looking(&mut w, hold, 0);
        top = top.max(w.players[0].pos.y.sub(from.y));
        far = far.max(w.players[0].pos.x.sub(from.x));
    }
    (top, far)
}

#[test]
fn holding_a_direction_through_the_updraft_carries_her_that_way() {
    let (_, still) = updraft_from_the_floor(0);
    let (_, leaning) = updraft_from_the_floor(Input::W);
    // Ordinary air steering is there either way; the column adds to it.
    let mut plain = elementalist();
    let from = plain.players[0].pos;
    step_looking(&mut plain, Input::SPACE, 0);
    let mut jumped = Fx::ZERO;
    for _ in 0..90 {
        step_looking(&mut plain, Input::W, 0);
        jumped = jumped.max(plain.players[0].pos.x.sub(from.x));
    }
    assert!(
        leaning.raw() > still.raw() && leaning.raw() > jumped.raw(),
        "held forward it carried her {} m, against {} still and {} on a plain jump",
        leaning.to_int(),
        still.to_int(),
        jumped.to_int()
    );
}
