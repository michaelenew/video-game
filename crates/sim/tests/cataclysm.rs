//! The Elementalist's heavy: Cataclysm, on right click.
//!
//! Structurally the same trick her auto is -- an instant line, resolved on the
//! spot rather than by the hitbox loop -- but heavier, and with the opposite
//! answer to what it is aimed through: a structure is destroyed rather than
//! kicked, and a fire pillar is torn loose into a travelling tornado rather
//! than merely charging the shot. These are the assertions for what a player
//! would notice if any of that broke.

use sim::class::Mechanic;
use sim::effects::EffectKind;
use sim::state::{Action, SLOT_HEAVY};
use sim::{Class, Fx, Input, V3, World};

const R: u16 = Input::RIGHT;
const E: u16 = Input::MECHANIC;
const Q: u16 = Input::SPECIAL;

fn run(w: &mut World, frames: u32, a: u16, b: u16) {
    for _ in 0..frames {
        w.advance([Input::aimed(a, 0), Input::aimed(b, 0)]);
    }
}

/// Press a button, let go, and let the move play out.
fn tap(w: &mut World, button: u16, then: u32) {
    run(w, 2, button, 0);
    run(w, then, 0, 0);
}

fn as_class(one: Class) -> World {
    World::with_classes([one, Class::Bulwark])
}

/// An Elementalist at the origin, facing +X, with nothing in her way. Every
/// grounded and skillshot ability she has lands somewhere down this line by
/// default, which is what lets these fixtures place a structure or a pillar
/// "ahead" and then fire the heavy through it without having to aim it.
fn elementalist() -> World {
    let mut w = as_class(Class::Elementalist);
    w.players[0].pos = V3::ZERO;
    w.players[1].pos = V3::new(Fx::from_int(-6), Fx::ZERO, Fx::ZERO);
    w
}

fn has_structure(w: &World) -> bool {
    matches!(w.players[0].mechanic, Mechanic::Structures(slots) if slots.iter().any(|s| s.is_some()))
}

fn fire_pillars(w: &World) -> usize {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.kind == EffectKind::FirePillar)
        .count()
}

fn tornadoes(w: &World) -> usize {
    w.tornadoes.iter().flatten().count()
}

// ---------------------------------------------------------------------------
// The button, and the gate
// ---------------------------------------------------------------------------

#[test]
fn right_click_throws_cataclysm_for_the_elementalist() {
    let mut w = elementalist();
    run(&mut w, 1, R, 0);
    assert!(
        matches!(
            w.players[0].action,
            Action::Startup {
                kind: SLOT_HEAVY,
                ..
            }
        ),
        "right click did not start the fourth move"
    );
}

#[test]
fn cataclysm_needs_no_structure_or_pillar_to_be_thrown() {
    // The lesson Fire pillar already learned: a heavy that interacts with
    // structures and fire is not the same thing as a heavy that requires one
    // to exist. Nothing has been raised and nothing has been planted here.
    let mut w = elementalist();
    assert!(
        w.players[0].mechanic_ready(SLOT_HEAVY),
        "Cataclysm is gated on having a structure or pillar out"
    );
    run(&mut w, 1, R, 0);
    assert!(
        matches!(
            w.players[0].action,
            Action::Startup {
                kind: SLOT_HEAVY,
                ..
            }
        ),
        "the fixture never actually threw it"
    );
}

#[test]
fn a_direct_hit_is_a_real_hit_not_the_autos_no_stagger_poke() {
    // The auto trades a fighter's charge for nothing at all -- see
    // `docs/design/kits/elementalist.md`. The heavy is not that: it costs a
    // long wind-up and it pays for it with a real hit, stagger and all.
    let mut w = elementalist();
    w.players[1].pos = V3::new(Fx::from_int(4), Fx::ZERO, Fx::ZERO); // ahead, inside reach
    let before = w.players[1].health;
    tap(&mut w, R, 30);
    assert!(
        w.players[1].health < before,
        "a fighter caught by Cataclysm took no damage"
    );
    assert!(
        matches!(
            w.players[1].action,
            Action::HitStun { .. } | Action::BlockStun { .. }
        ),
        "a fighter caught by Cataclysm was free again immediately, like the auto's poke"
    );
}

// ---------------------------------------------------------------------------
// A structure in the way
// ---------------------------------------------------------------------------

#[test]
fn cataclysm_destroys_a_structure_and_blasts_the_area_around_it() {
    let mut w = elementalist();
    tap(&mut w, E, 30); // raise one ahead, and let it finish rising
    assert!(has_structure(&w), "fixture never raised a structure");

    // Stand just past where the structure lands, so the blast has to reach
    // past the stone to catch him -- the beam itself should not, since a
    // structure in the way is exactly what stops the auto's own shot dead.
    let past = sim::tuning::raise_reach().add(Fx::from_int(2));
    w.players[1].pos = V3::new(past, Fx::ZERO, Fx::ZERO);
    let before = w.players[1].health;

    tap(&mut w, R, 30);

    assert!(!has_structure(&w), "the structure survived Cataclysm");
    assert!(
        w.players[1].health < before,
        "nobody standing near the broken structure was caught in the blast"
    );
}

#[test]
fn cataclysm_never_blasts_its_own_caster() {
    let mut w = elementalist();
    tap(&mut w, E, 30);
    assert!(has_structure(&w), "fixture never raised a structure");
    let before = w.players[0].health;
    tap(&mut w, R, 30);
    assert!(
        !has_structure(&w),
        "fixture never destroyed its own structure"
    );
    assert_eq!(
        w.players[0].health, before,
        "the Elementalist blasted herself with her own Cataclysm"
    );
}

// ---------------------------------------------------------------------------
// A fire pillar in the way
// ---------------------------------------------------------------------------

#[test]
fn cataclysm_turns_a_fire_pillar_into_a_travelling_tornado() {
    let mut w = elementalist();
    tap(&mut w, Q, 50); // plant a pillar ahead, and let her fully recover
    assert_eq!(
        fire_pillars(&w),
        1,
        "fixture planted no pillar to aim through"
    );
    assert_eq!(
        tornadoes(&w),
        0,
        "fixture started with a tornado already out"
    );
    assert!(
        w.players[0].action.actionable(),
        "fixture is still recovering from Fire pillar"
    );

    tap(&mut w, R, 30);

    assert_eq!(
        fire_pillars(&w),
        0,
        "the pillar was still standing -- Cataclysm should tear it loose, not just charge it"
    );
    assert_eq!(tornadoes(&w), 1, "no tornado came out of the pillar");
}

// ---------------------------------------------------------------------------
// The tornado itself
// ---------------------------------------------------------------------------

#[test]
fn the_tornado_travels_and_burns_out_on_its_own_clock() {
    let mut w = elementalist();
    sim::tornado::spawn(
        &mut w.tornadoes,
        0,
        V3::new(Fx::from_int(-10), Fx::ZERO, Fx::ZERO),
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
    );
    let start = w.tornadoes[0].expect("fixture never spawned one").pos;
    run(&mut w, 10, 0, 0);
    let moved = w.tornadoes[0].expect("it vanished early").pos;
    assert!(
        moved.x.raw() > start.x.raw(),
        "the tornado did not move along the direction it was lit on"
    );

    run(&mut w, 300, 0, 0);
    assert!(
        w.tornadoes.iter().all(|t| t.is_none()),
        "the tornado outlived its own lifetime"
    );
}

#[test]
fn the_tornado_pulls_and_burns_whoever_it_catches() {
    let mut w = elementalist();
    w.players[1].pos = V3::new(Fx::from_int(3), Fx::ZERO, Fx::ZERO);
    let start = w.players[1].pos;
    let before = w.players[1].health;

    // Lit right where he is standing and off sideways from there, so a good
    // stretch of its travel keeps him inside the pull radius -- what is under
    // test is the pull and the tick, not a chase across the arena.
    sim::tornado::spawn(
        &mut w.tornadoes,
        0,
        start,
        V3::new(Fx::ZERO, Fx::ZERO, Fx::ONE),
    );
    run(&mut w, 5, 0, 0);
    assert!(
        w.players[1].vel.z.raw() > 0,
        "standing where the tornado is, he was not pulled toward it"
    );

    run(&mut w, 10, 0, 0); // past the first damage tick
    assert!(
        w.players[1].health < before,
        "standing inside the tornado cost no health"
    );
}

#[test]
fn the_tornado_never_catches_its_own_owner() {
    let mut w = elementalist();
    let before = w.players[0].health;
    let start = w.players[0].pos;
    sim::tornado::spawn(
        &mut w.tornadoes,
        0,
        V3::ZERO,
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
    );
    run(&mut w, 40, 0, 0);
    assert_eq!(
        w.players[0].health, before,
        "the tornado burned its own owner"
    );
    assert_eq!(
        w.players[0].pos, start,
        "the tornado pulled at its own owner"
    );
}
