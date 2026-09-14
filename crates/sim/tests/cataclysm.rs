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

/// An Elementalist at the origin, facing +X. Every grounded and skillshot
/// ability she has lands somewhere down this line by default, which is what
/// lets these fixtures place a structure or a pillar "ahead" and then fire the
/// heavy through it without having to aim it.
///
/// **Clear for about five metres, and not beyond.** The blockout has a dais at
/// `x ∈ [5, 9], z ∈ [-4, 4]`, a metre and a half tall (`arena::SOLIDS`), so a
/// fixture that reaches past x = 5 on this line is standing things on a roof.
/// Anything that needs real distance moves off the band first -- see
/// [`CLEAR_LANE`].
fn elementalist() -> World {
    let mut w = as_class(Class::Elementalist);
    w.players[0].pos = V3::ZERO;
    w.players[1].pos = V3::new(Fx::from_int(-6), Fx::ZERO, Fx::ZERO);
    w
}

/// A line down positive X with genuinely nothing on it, for the fixtures that
/// need more room than the dais leaves.
///
/// `tests/beam.rs` and `tests/aiming.rs` both already sit out here, and for the
/// same reason. It cost a real failure to learn twice: a stone raised inside
/// the dais's footprint is pushed up **on to** it, so it stands a metre and a
/// half in the air with a gap underneath -- and debris does not collide with
/// the arena at all (see the kit's open questions), so anything travelling near
/// the floor sails under the thing that was supposed to stop it. What that
/// looks like from the assertion is "debris punched through a stone", which is
/// not what happened.
const CLEAR_LANE: Fx = Fx::from_int(8);

fn has_structure(w: &World) -> bool {
    matches!(w.players[0].mechanic, Mechanic::Structures(slots) if slots.iter().any(|s| s.is_some()))
}

/// A fully risen structure, standing at rest on the ground -- the same
/// fixture `crates/sim/tests/stones.rs` uses.
fn standing_at(x: i32, z: Fx) -> Structure {
    let mut stone = Structure::raised(V3::new(Fx::from_int(x), Fx::ZERO, z));
    stone.age = stone.rise + 1;
    stone
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
///
/// `age` and `life` stand in for however grown the pillar it came from
/// already was, and however much of its own life was left -- see
/// `state::World::fire_the_cataclysm`, which never resets either. `(0,
/// pillar_life())` is fine for a fixture that only cares about direction or
/// ownership; one that cares about the pull or the burn wants it well grown,
/// with plenty of life left to run the fixture on -- a freshly lit tornado is
/// also freshly tiny, and standing exactly where it was lit is not a fair
/// test of whether it can catch someone once it is actually out.
fn light_tornado(w: &mut World, owner: u8, at: V3, dir: V3, age: u16, life: u16) -> usize {
    let slot = w
        .effects
        .iter()
        .position(|e| e.is_none())
        .expect("no free effect slot in the fixture");
    let mut fresh = Effect::cast(
        EffectKind::FireTornado,
        owner,
        Class::Elementalist,
        SLOT_SPECIAL,
        at,
        dir,
        Fx::ZERO,
    );
    fresh.age = age;
    fresh.life = life;
    // Lit just now: its flight starts at `at` with nothing banked against it
    // yet -- see `Effect::tornado_pos`.
    fresh.banked = age as i32;
    w.effects[slot] = Some(fresh);
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
    // **Run twice, and the second run is what makes the first mean anything.**
    // With a second stone in the way he should take nothing; with the way
    // clear he has to take *something*, or the assertion above is only saying
    // that the debris never got there -- which is exactly the way this test
    // stopped working. See [`CLEAR_LANE`]: at z = 0 the far stone stands on
    // the dais with a metre and a half of air under it, the low pieces of the
    // fan sail beneath it, and the failure reads as "debris punched through a
    // stone" when nothing of the kind happened.
    let fired_through = |blocked: bool| {
        let mut w = elementalist();
        w.players[0].pos = V3::new(Fx::ZERO, Fx::ZERO, CLEAR_LANE);
        // The near stone is what Cataclysm actually breaks. The far one, when
        // it is there, is what its debris should not be able to reach past.
        let near = standing_at(4, CLEAR_LANE);
        let far = standing_at(8, CLEAR_LANE);
        if blocked {
            place(&mut w, &[near, far]);
        } else {
            place(&mut w, &[near]);
        }
        // Well past where the far stone stands, so nothing but a piece
        // punching through it could ever reach him.
        w.players[1].pos = V3::new(Fx::from_int(11), Fx::ZERO, CLEAR_LANE);
        let before = w.players[1].health;
        tap(&mut w, R, 90);
        let far_survived = matches!(
            w.players[0].mechanic,
            Mechanic::Structures(slots) if slots[1].is_some()
        );
        (before - w.players[1].health, far_survived)
    };

    let (through_open_air, _) = fired_through(false);
    assert!(
        through_open_air > 0,
        "with nothing in the way the debris did not reach him either, so a stone          stopping it proves nothing"
    );

    let (past_the_stone, far_survived) = fired_through(true);
    assert!(
        far_survived,
        "the far stone was destroyed -- debris should stop at the first solid thing, not break it"
    );
    assert_eq!(
        past_the_stone, 0,
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
fn a_tornado_grows_on_the_same_curve_its_pillar_did() {
    let mut e = Effect::cast(
        EffectKind::FireTornado,
        0,
        Class::Elementalist,
        SLOT_SPECIAL,
        V3::ZERO,
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        Fx::ZERO,
    );
    e.life = sim::tuning::pillar_life();
    let (young, _) = e.pillar_volumes();
    e.age = e.life;
    let (grown, _) = e.pillar_volumes();
    assert!(
        young.radius.raw() < grown.radius.raw(),
        "a tornado came out already at full size instead of growing into it"
    );
    assert_eq!(
        grown.radius,
        sim::tuning::pillar_base_radius(),
        "a fully grown tornado was not the same size a fully grown pillar is"
    );
}

#[test]
fn a_tornado_gets_its_own_travel_time_no_matter_how_old_the_pillar_was() {
    // As old as a fire pillar can be without having already expired on its
    // own -- catching one this late used to cut its tornado's travel to
    // almost nothing, because expiry read the same age/life pair growth
    // does. The two are different questions now: how grown it is has
    // nothing to do with how much longer it gets to travel.
    let mut w = elementalist();
    let ancient = sim::tuning::pillar_life() - 1;
    let slot = light_tornado(
        &mut w,
        0,
        V3::new(Fx::from_int(-10), Fx::ZERO, Fx::ZERO),
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        ancient,
        sim::tuning::pillar_life(),
    );
    run(&mut w, 60, 0, 0);
    assert!(
        w.effects[slot].is_some(),
        "a tornado cut loose from an old pillar burned out almost immediately"
    );
}

#[test]
fn catching_the_tornado_stuns_before_it_can_drag() {
    // A fighter's own grounded movement sets velocity from the stick every
    // frame he is free to act, which would erase the pull before it ever
    // moved him -- see `state::World::apply_effect`'s `FireTornado` arm. The
    // catch has to cost him control for a moment, or there is no window for
    // the pull to work in at all.
    let mut w = elementalist();
    w.players[1].pos = V3::new(Fx::from_int(3), Fx::ZERO, Fx::ZERO);
    let start = w.players[1].pos;
    light_tornado(
        &mut w,
        0,
        start,
        V3::new(Fx::ZERO, Fx::ZERO, Fx::ONE),
        900,
        1000,
    );
    run(&mut w, 1, 0, 0);
    assert!(
        w.players[1].action.stunned(),
        "the tornado caught him without stunning him"
    );
}

#[test]
fn catching_the_tornado_carries_a_fighter_along_rather_than_leaving_him_behind() {
    // The suck alone only ever tugs a victim toward wherever the tornado's
    // centre currently is, which it is racing away from at full speed --
    // being caught has to mean riding along with it, not being left further
    // and further behind a point it keeps abandoning.
    let mut w = elementalist();
    w.players[1].pos = V3::new(Fx::from_int(3), Fx::ZERO, Fx::ZERO);
    let start = w.players[1].pos;
    light_tornado(
        &mut w,
        0,
        start,
        V3::new(Fx::ZERO, Fx::ZERO, Fx::ONE),
        900,
        1000,
    );

    let gap_at = |w: &World| {
        let tornado = w
            .effects
            .iter()
            .flatten()
            .find(|e| e.kind == EffectKind::FireTornado)
            .expect("it vanished early");
        w.players[1].pos.sub(tornado.tornado_pos()).flat_len()
    };

    run(&mut w, 20, 0, 0);
    let early_gap = gap_at(&w);
    run(&mut w, 20, 0, 0); // still well short of the arena's own edge
    let later_gap = gap_at(&w);
    assert!(
        later_gap.raw() <= early_gap.raw(),
        "the gap behind the tornado's live centre kept growing rather than settling \
         once he was caught: {} m, then {} m",
        early_gap.to_f32_for_render(),
        later_gap.to_f32_for_render()
    );
}

#[test]
fn the_tornado_travels_and_burns_out_on_its_own_clock() {
    let mut w = elementalist();
    let slot = light_tornado(
        &mut w,
        0,
        V3::new(Fx::from_int(-10), Fx::ZERO, Fx::ZERO),
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        0,
        sim::tuning::pillar_life(),
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
    // Off to one side of the line the tornado is about to travel, rather than
    // sitting dead on it -- riding along the axis is the carry's job (see
    // `catching_the_tornado_carries_a_fighter_along_rather_than_leaving_him_behind`),
    // and standing exactly on it leaves the pull nothing to do: an axis the
    // carry already keeps him matched to has no radial drift left for suction
    // to close. A half-metre offset gives it something real to pull him out
    // of, without putting him outside the pillar's own volumes.
    let lit_at = V3::new(Fx::from_int(3), Fx::ZERO, Fx::ZERO);
    w.players[1].pos = V3::new(Fx::from_int(3).add(Fx::ratio(1, 2)), Fx::ZERO, Fx::ZERO);
    let before = w.players[1].health;

    // Already well grown, with plenty of life left to run the fixture on: a
    // freshly lit tornado is also freshly tiny and travelling at full speed
    // from its first frame, so a fixture that lit one right where he stands
    // would be closer to a test of whether it can outrun him than of whether
    // it can catch him.
    light_tornado(
        &mut w,
        0,
        lit_at,
        V3::new(Fx::ZERO, Fx::ZERO, Fx::ONE),
        900,
        1000,
    );
    run(&mut w, 5, 0, 0);
    assert!(
        w.players[1].vel.x.raw() < 0,
        "standing off to the side of the tornado's axis, he was not pulled toward it"
    );

    run(&mut w, 60, 0, 0); // comfortably past a damage tick
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
    light_tornado(
        &mut w,
        0,
        V3::ZERO,
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        0,
        sim::tuning::pillar_life(),
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

// ---------------------------------------------------------------------------
// Aiming at height
// ---------------------------------------------------------------------------

/// The yaw and pitch that put the crosshair on a point in the world, for a
/// fighter standing at `stood`.
///
/// The crosshair's ray starts at the *eye* -- well behind and above the
/// fighter -- so "look at this spot" is not the angle from the fighter to it.
/// Where the eye sits depends on the pitch, so this settles the two against
/// each other; a couple of rounds is plenty, the rig being smooth. The same
/// trick `crates/sim/tests/stones.rs` uses.
fn look_at(stood: V3, mark: V3) -> (u16, i16) {
    let turns = |v: f32| (v * 65536.0 / std::f32::consts::TAU) as i32;
    let yaw = turns(
        mark.z
            .sub(stood.z)
            .to_f32_for_render()
            .atan2(mark.x.sub(stood.x).to_f32_for_render()),
    ) as u16;
    let mut tilt = 0i16;
    for _ in 0..6 {
        let eye = sim::camera::eye(stood, Input::looking_at(0, yaw, tilt), Fx::ZERO);
        let flat = mark.sub(eye).flat_len().to_f32_for_render();
        let rise = mark.y.sub(eye.y).to_f32_for_render();
        tilt = turns(rise.atan2(flat)) as i16;
    }
    (yaw, tilt)
}

/// Press, aimed at a point in the world, let go, and let it play out.
fn tap_at(w: &mut World, button: u16, mark: V3, then: u32) {
    let (yaw, tilt) = look_at(w.players[0].pos, mark);
    for _ in 0..2 {
        w.advance([Input::looking_at(button, yaw, tilt), Input::aimed(0, 0)]);
    }
    run(w, then, 0, 0);
}

/// A dais's top corner is Cataclysm's own worst case: caught with a pitch
/// that is not level (see `aim::skillshot_path`, which only flattens Y when
/// the crosshair reads as `Met::Ground`), a tornado's own line of travel used
/// to inherit that tilt straight from the beam, walk its centre through the
/// floor a couple of frames later, and read that as having wandered off the
/// map -- the pillar vanished instead of ever being seen to move. This is a
/// consistent player repro, not a knife's-edge one: stand beside one of the
/// arena's two low platforms, plant a pillar at its base corner, then aim
/// Cataclysm at the corner above that -- see `crate::arena::SOLIDS`.
#[test]
fn cataclysm_aimed_at_a_dais_corner_still_turns_the_pillar_into_a_tornado() {
    let mut w = elementalist();
    w.players[0].pos = V3::new(Fx::from_int(-11), Fx::ZERO, Fx::from_int(-4));
    w.players[1].pos = V3::new(Fx::from_int(20), Fx::ZERO, Fx::from_int(20));

    // The low platform at x in [-9, -5], z in [-4, 4] -- see `arena::SOLIDS`.
    // The near-ground corner, for the pillar; the same corner on the
    // platform's top, for Cataclysm.
    let lower_corner = V3::new(Fx::from_int(-9), Fx::ZERO, Fx::from_int(4));
    let upper_corner = V3::new(Fx::from_int(-9), Fx::ratio(3, 2), Fx::from_int(4));

    tap_at(&mut w, Q, lower_corner, 50);
    assert_eq!(
        fire_pillars(&w),
        1,
        "fixture planted no pillar at the dais's base corner"
    );

    tap_at(&mut w, R, upper_corner, 30);
    assert_eq!(
        fire_pillars(&w),
        0,
        "the pillar was still standing -- Cataclysm should tear it loose"
    );
    assert_eq!(
        tornadoes(&w),
        1,
        "aimed at the corner above the pillar, Cataclysm made it vanish instead of a tornado"
    );
}
