//! The Elementalist's auto, as a line.
//!
//! The complaint these were written for: *"even when I aim upwards, the auto
//! attack still just follows along the ground."* It did. The shot was a flat
//! circle at a fixed distance in front of her, and every test below is a way
//! of stating that it is not one any more — it goes where the crosshair is
//! pointing, in three dimensions, and what it meets first is the whole move.
//!
//! Phrased in terms of what happens on the field rather than which number
//! moved, like the rest of the harness.

use sim::class::{Class, Mechanic};
use sim::state::{Action, max_health};
use sim::tuning as t;
use sim::{Fx, Input, V3, World};

const L: u16 = Input::LEFT;
const E: u16 = Input::MECHANIC;
const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

/// Degrees above the horizon, in the aim's own units.
fn up(degrees: f32) -> i16 {
    (degrees / 360.0 * 65536.0) as i16
}

fn metres(v: f32) -> Fx {
    Fx::ratio((v * 1000.0) as i32, 1000)
}

fn at(x: f32, y: f32, z: f32) -> V3 {
    V3::new(metres(x), metres(y), metres(z))
}

/// An Elementalist against someone with no answer to anything.
fn elementalist() -> World {
    World::with_classes([Class::Elementalist, Class::Bulwark])
}

/// Run with player one looking along `pitch` and player two doing nothing.
fn run(w: &mut World, frames: u32, buttons: u16, pitch: i16) {
    for _ in 0..frames {
        w.advance([
            Input::looking_at(buttons, LOOK_RIGHT, pitch),
            Input::aimed(0, LOOK_LEFT),
        ]);
    }
}

/// Throw the auto at a given pitch and let it play out.
fn shoot(w: &mut World, pitch: i16) {
    run(w, 1, L, pitch);
    run(w, 24, 0, pitch);
}

/// Aimed up, but not so far up that the shot goes over the stone: at its near
/// edge, a little below its top.
///
/// Derived rather than typed in, because Raise's reach and a stone's height are
/// both tuned — a pitch that just cleared the top at one pair of values is a
/// miss at the next, and the test would then be passing for the wrong reason.
fn just_under_the_top(w: &World) -> i16 {
    let stone = stones_of(w)[0];
    let from = sim::aim::origin(w.players[0].pos);
    let near = stone
        .at
        .sub(from)
        .flat_len()
        .sub(t::structure_radius())
        .to_f32_for_render();
    let rise = stone
        .top()
        .sub(t::structure_height().div(Fx::from_int(4)))
        .sub(from.y)
        .to_f32_for_render();
    (rise.atan2(near) / std::f32::consts::TAU * 65536.0) as i16
}

fn stones_of(w: &World) -> Vec<sim::class::Structure> {
    match w.players[0].mechanic {
        Mechanic::Structures(slots) => slots.iter().flatten().copied().collect(),
        _ => Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Where the shot goes
// ---------------------------------------------------------------------------

#[test]
fn aiming_over_someone_shoots_over_them() {
    // The regression, stated as directly as it can be. Level, the shot lands.
    // Aimed well above the same fighter standing in the same place, it does
    // not -- because it is a line at the angle it was fired, rather than a
    // circle laid on the floor a fixed distance ahead.
    let mut level = elementalist();
    shoot(&mut level, 0);
    assert!(
        level.players[1].health < max_health(),
        "a level shot did not reach a fighter standing in front of her"
    );

    let mut high = elementalist();
    shoot(&mut high, up(35.0));
    assert_eq!(
        high.players[1].health,
        max_health(),
        "a shot aimed thirty-five degrees above a fighter still hit them, so the aim \
         is decoration and the shot is following the ground"
    );
}

#[test]
fn aiming_up_reaches_someone_standing_above_her() {
    // The other half of the same property, and the half that makes the move
    // worth aiming: someone on a platform is out of reach of a level shot and
    // in reach of one pointed at them.
    let stand = |w: &mut World| {
        w.players[0].pos = at(-1.0, 0.0, 0.0);
        // On the raised platform at the far end, which is a body's height up.
        w.players[1].pos = at(6.0, 1.5, 0.0);
    };

    let mut level = elementalist();
    stand(&mut level);
    shoot(&mut level, 0);
    assert_eq!(
        level.players[1].health,
        max_health(),
        "a level shot climbed a metre and a half on its own to reach a platform"
    );

    let mut aimed = elementalist();
    stand(&mut aimed);
    shoot(&mut aimed, up(12.0));
    assert!(
        aimed.players[1].health < max_health(),
        "aiming up at a fighter standing above her did not reach them"
    );
}

#[test]
fn the_drawn_beam_ends_where_the_shot_stopped() {
    // The overlay draws `state::hitbox`, and for this move that is the line
    // itself. A beam stopped by a stone two metres out is two metres long; one
    // that met nothing runs its whole range. Drawing either as the other is
    // how you end up unable to tell why a shot did not connect.
    let mut w = elementalist();
    run(&mut w, 2, E, 0); // a stone, a few metres ahead
    run(&mut w, 30, 0, 0);
    let stone = stones_of(&w)[0].at;

    let mut seen = None;
    for _ in 0..30 {
        run(&mut w, 1, L, 0);
        if let Some(hb) = sim::state::hitbox(&w.players[0]) {
            seen = Some(hb);
            break;
        }
    }
    let hb = seen.expect("the auto never put a volume out");
    assert!(hb.is_a_beam(), "the auto's volume is not drawn as a line");

    let reach = hb.to.sub(hb.from).len();
    let to_stone = stone.sub(hb.from).flat_len();
    assert!(
        reach.raw() <= to_stone.raw(),
        "the beam was drawn {} m long past a stone standing {} m away",
        reach.to_f32_for_render(),
        to_stone.to_f32_for_render()
    );

    // And a shot with nothing in front of it draws its whole range.
    let mut clear = elementalist();
    clear.players[1].pos = at(13.0, 0.0, 0.0);
    let mut seen = None;
    for _ in 0..30 {
        run(&mut clear, 1, L, 0);
        if let Some(hb) = sim::state::hitbox(&clear.players[0]) {
            seen = Some(hb);
            break;
        }
    }
    let hb = seen.expect("the auto never put a volume out");
    let full = sim::moves::get(Class::Elementalist, 0).reach;
    let drawn = hb.to.sub(hb.from).len();
    assert!(
        full.sub(drawn).abs().raw() < metres(0.05).raw(),
        "a shot that met nothing was drawn {} m long against a range of {} m",
        drawn.to_f32_for_render(),
        full.to_f32_for_render()
    );
}

// ---------------------------------------------------------------------------
// What it does to a fighter
// ---------------------------------------------------------------------------

#[test]
fn the_shot_takes_the_charge_and_gives_the_frames_straight_back() {
    // Small damage, the wind-up they were partway through, and nothing else.
    // No hitstun, no stagger, no shove -- being poked by the auto costs you
    // the move you were holding and not your turn.
    let mut w = elementalist();
    let mut caught = None;
    let mut wound_up = false;
    for _ in 0..40 {
        // Player two winds up their slowest move once, then lets go, so what
        // the beam interrupts is not immediately thrown again.
        let them = if !wound_up && w.players[1].action == Action::Free {
            wound_up = true;
            Input::SHIFT | Input::LEFT
        } else {
            0
        };
        let before = w.players[1];
        w.advance([
            Input::looking_at(L, LOOK_RIGHT, 0),
            Input::aimed(them, LOOK_LEFT),
        ]);
        if w.players[1].health < before.health && caught.is_none() {
            caught = Some((before.action, w.players[1]));
        }
    }
    let (was, now) = caught.expect("the auto never connected");

    assert!(
        matches!(was, Action::Startup { .. }),
        "fixture did not catch player two mid-charge; they were {was:?}"
    );
    assert_eq!(
        now.action,
        Action::Free,
        "the auto left its victim in {:?} -- it is meant to have no stagger at all",
        now.action
    );
    assert_eq!(
        (now.vel.x.raw(), now.vel.z.raw()),
        (0, 0),
        "the auto shoved its victim, which is knockback by another name"
    );
    assert!(
        max_health() - now.health <= max_health() / 10,
        "the auto took {} of {} health; it is meant to be small",
        max_health() - now.health,
        max_health()
    );
}

// ---------------------------------------------------------------------------
// What it does to a structure
// ---------------------------------------------------------------------------

#[test]
fn a_stone_shot_while_aiming_up_is_sent_up() {
    // "Along the ground if aiming down, an impulse along the mouse direction
    // up into the air if aiming up." The stone goes where the mouse points.
    // The stone as it leaves, rather than wherever it has got to afterwards:
    // gravity is already pulling on it the frame after the kick.
    let kicked = |pitch: fn(&World) -> i16| {
        let mut w = elementalist();
        run(&mut w, 2, E, 0);
        run(&mut w, 30, 0, 0); // let it finish climbing out
        let pitch = pitch(&w);
        run(&mut w, 1, L, pitch);
        for _ in 0..24 {
            run(&mut w, 1, 0, pitch);
            let stone = stones_of(&w)[0];
            if stone.launched {
                return Some((stone, pitch));
            }
        }
        None
    };

    let (lifted, pitch) =
        kicked(just_under_the_top).expect("aiming up missed a stone standing in front of her");
    assert!(
        pitch > 0,
        "the fixture is no longer aiming upward at all, so it demonstrates nothing"
    );
    assert!(
        lifted.vel.y.raw() > 0,
        "a stone shot while aiming up was sent along the ground anyway"
    );

    let (flat, _) = kicked(|_| 0).expect("a level shot missed the stone in front of her");
    assert!(
        flat.vel.y.raw() <= 0,
        "a stone shot level left the ground, so pitch is not deciding anything"
    );
}

#[test]
fn a_shot_aimed_over_a_stone_passes_over_it() {
    // A structure is a solid, so it blocks the shot -- but only when the shot
    // actually meets it. Aiming above one is how you shoot past your own
    // terrain, and it could not be done while the trace ignored height.
    let mut w = elementalist();
    run(&mut w, 2, E, 0);
    run(&mut w, 30, 0, 0);
    shoot(&mut w, up(35.0));

    assert!(
        !stones_of(&w)[0].launched,
        "a shot aimed thirty-five degrees above a stone still kicked it"
    );
}

// ---------------------------------------------------------------------------
// What it does to fire
// ---------------------------------------------------------------------------

#[test]
fn a_fire_bolt_carries_the_shot_to_someone_the_beam_could_never_reach() {
    // The beam is short. The bolt it lights is not -- that is the whole trade,
    // and the reason a pillar is worth putting between you and them.
    let mut w = elementalist();
    // Clear of the raised platforms, which reach four metres either side of
    // the middle: a pillar planted on one stands a platform's height up, and
    // a level shot passes underneath it, which is correct and not what this
    // test is about.
    w.players[0].pos = at(-8.0, 0.0, 8.0);
    w.players[1].pos = at(8.0, 0.0, 8.0);

    run(&mut w, 2, Input::SPECIAL, 0); // plant a pillar ahead of her
    run(&mut w, 60, 0, 0); // and let her recover from planting it
    let pillar = w.effects.iter().flatten().next().copied();
    let pillar = pillar.expect("fixture planted no pillar");
    let gap = w.players[1].pos.sub(w.players[0].pos).flat_len();
    let reach = sim::moves::get(Class::Elementalist, 0).reach;
    assert!(
        gap.raw() > reach.raw(),
        "fixture put the two fighters inside the beam's own range"
    );
    assert!(
        pillar.pos.x.raw() < w.players[1].pos.x.raw(),
        "fixture put the pillar behind the target"
    );

    let before = w.players[1].health;
    run(&mut w, 1, L, 0);
    let mut staggered = false;
    for _ in 0..90 {
        run(&mut w, 1, 0, 0);
        staggered |= w.players[1].action.stunned();
    }

    assert!(
        w.players[1].health < before,
        "a bolt lit at a pillar never reached the fighter beyond it"
    );
    assert!(
        staggered,
        "the fire bolt is the part of the auto that staggers, and it did not"
    );
    assert!(
        w.bolts.iter().all(|b| b.is_none()),
        "the bolt carried on past the fighter it hit"
    );
}
