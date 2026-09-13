//! The Elementalist's heavy: Cataclysm, on right click.
//!
//! Structurally the same trick her auto is -- an instant line, resolved on the
//! spot rather than by the hitbox loop -- but heavier, and with the opposite
//! answer to what it is aimed through: a structure breaks into thrown debris
//! rather than being kicked, and a fire pillar is torn loose into a
//! travelling tornado rather than merely charging the shot. These are the
//! assertions for what a player would notice if any of that broke.

use sim::class::{MAX_STRUCTURES, Mechanic, Structure};
use sim::effects::{Effect, EffectKind};
use sim::state::{Action, SLOT_HEAVY, SLOT_SPECIAL};
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

/// A fully risen structure, standing at rest on the ground -- the same
/// fixture `crates/sim/tests/stones.rs` uses.
fn standing_at(x: i32) -> Structure {
    Structure {
        at: V3::new(Fx::from_int(x), Fx::ZERO, Fx::ZERO),
        vel: V3::ZERO,
        age: sim::tuning::structure_rise() + 1,
        struck: 0,
        launched: false,
        launch_from: V3::ZERO,
        knock_struck: 0,
    }
}

/// Put exactly these structures on the caster's own mechanic, bypassing
/// Raise -- for the fixtures below that need more than one on the field at
/// once, or one already fully grown.
fn place(w: &mut World, stones: &[Structure]) {
    let mut slots = [None; MAX_STRUCTURES];
    for (slot, s) in slots.iter_mut().zip(stones) {
        *slot = Some(*s);
    }
    w.players[0].mechanic = Mechanic::Structures(slots);
}

fn fire_pillars(w: &World) -> usize {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.kind == EffectKind::FirePillar)
        .count()
}

fn tornadoes(w: &World) -> usize {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.kind == EffectKind::FireTornado)
        .count()
}

/// Light a tornado directly, bypassing a fire pillar's own startup and
/// growth, for the tests below that are about what a tornado already out
/// does rather than about Cataclysm finding one.
fn light_tornado(w: &mut World, owner: u8, at: V3, dir: V3) -> usize {
    let slot = w
        .effects
        .iter()
        .position(|e| e.is_none())
        .expect("no free effect slot in the fixture");
    w.effects[slot] = Some(Effect::cast(
        EffectKind::FireTornado,
        owner,
        Class::Elementalist,
        SLOT_SPECIAL,
        at,
        dir,
        Fx::ZERO,
    ));
    slot
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
fn cataclysm_destroys_a_structure_and_scatters_it_as_debris() {
    let mut w = elementalist();
    tap(&mut w, E, 30); // raise one ahead, and let it finish rising
    assert!(has_structure(&w), "fixture never raised a structure");

    // Stand just past where the structure lands, on the same line Cataclysm
    // is aimed along -- the centre piece of the fan flies straight down that
    // line, so this is the one spot a shotgun spread is guaranteed to reach.
    // The beam itself should not reach this far: a structure in the way is
    // exactly what stops the auto's own shot dead.
    let past = sim::tuning::raise_reach().add(Fx::from_int(2));
    w.players[1].pos = V3::new(past, Fx::ZERO, Fx::ZERO);
    let before = w.players[1].health;

    // Longer than the other fixtures wait: destroying the structure is not
    // the hit, only the moment the debris is thrown, and it still has to fly
    // the distance.
    tap(&mut w, R, 60);

    assert!(!has_structure(&w), "the structure survived Cataclysm");
    assert!(
        w.players[1].health < before,
        "nobody standing near the broken structure was caught by its debris"
    );
}

#[test]
fn the_debris_cone_stands_in_space_rather_than_lying_flat() {
    // Aimed up and to the side. A fan that only rotated the aimed line's
    // horizontal bearing -- the bug this pins -- hands every piece `dir`'s
    // own pitch back unchanged; a real cone around `dir` does not.
    let dir = V3::new(Fx::ONE, Fx::ONE, Fx::ZERO).normalized();
    let mut shrapnel = [None; sim::debris::MAX_DEBRIS];
    sim::debris::blast(&mut shrapnel, 0, V3::ZERO, dir);
    let pieces: Vec<V3> = shrapnel.iter().flatten().map(|s| s.dir).collect();
    assert_eq!(
        pieces.len(),
        sim::debris::PIECES_PER_BLAST,
        "the blast did not throw every piece"
    );

    assert!(
        pieces.iter().any(|p| p.y.raw() != dir.y.raw()),
        "every piece came out at dir's own pitch -- the fan is flat, not a cone"
    );
    // Every piece stays close to the line it was aimed along, inside its own
    // half-angle -- see `tuning::debris_spread` -- rather than scattering
    // arbitrarily once it is allowed to leave the horizontal plane.
    for p in &pieces {
        assert!(
            p.dot(dir).raw() > Fx::ratio(9, 10).raw(),
            "a piece flew well outside the cone Cataclysm was aimed along"
        );
    }
}

#[test]
fn debris_shatters_on_the_first_stone_it_hits() {
    let mut w = elementalist();
    // Two on the same line: the near one is what Cataclysm actually breaks,
    // the far one is what its debris should not be able to reach.
    place(&mut w, &[standing_at(4), standing_at(8)]);

    // Well past the far stone, so nothing but a piece punching through it
    // could ever reach him.
    w.players[1].pos = V3::new(Fx::from_int(11), Fx::ZERO, Fx::ZERO);
    let before = w.players[1].health;

    tap(&mut w, R, 90);

    let far_survived = matches!(
        w.players[0].mechanic,
        Mechanic::Structures(slots) if slots[1].is_some()
    );
    assert!(
        far_survived,
        "the far stone was destroyed -- debris should stop at the first solid thing, not break it"
    );
    assert_eq!(
        w.players[1].health, before,
        "debris punched through the stone in its way to reach whoever was behind it"
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
    let slot = light_tornado(
        &mut w,
        0,
        V3::new(Fx::from_int(-10), Fx::ZERO, Fx::ZERO),
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
    );
    let start = w.effects[slot].expect("fixture never lit one").pos;
    run(&mut w, 10, 0, 0);
    let moved = w.effects[slot].expect("it vanished early").tornado_pos();
    assert!(
        moved.x.raw() > start.x.raw(),
        "the tornado did not move along the direction it was lit on"
    );

    run(&mut w, 300, 0, 0);
    assert!(tornadoes(&w) == 0, "the tornado outlived its own lifetime");
}

#[test]
fn the_tornado_pulls_and_burns_whoever_it_catches() {
    let mut w = elementalist();
    w.players[1].pos = V3::new(Fx::from_int(3), Fx::ZERO, Fx::ZERO);
    let start = w.players[1].pos;
    let before = w.players[1].health;

    // Lit right where he is standing and off sideways from there, so a good
    // stretch of its travel keeps him inside its own volumes -- what is under
    // test is the pull and the tick, not a chase across the arena.
    light_tornado(&mut w, 0, start, V3::new(Fx::ZERO, Fx::ZERO, Fx::ONE));
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
    light_tornado(&mut w, 0, V3::ZERO, V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO));
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
