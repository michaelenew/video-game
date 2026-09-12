//! Where abilities go.
//!
//! One rule under all of it: follow the line the player is looking along, out
//! from the point abilities come from, and stop at the first of the terrain or
//! the edge of the ability's reach. These are the assertions for what that buys
//! — placing an area where you want it rather than where you are standing, and
//! never being able to place it somewhere the ability cannot reach.

use sim::aim;
use sim::class::{MAX_STRUCTURES, Mechanic, Structure};
use sim::effects::EffectKind;
use sim::state::{MAX_PLAYERS, SLOT_SPECIAL};
use sim::stones::Phase;
use sim::tuning as t;
use sim::{Class, Fx, Input, V3, World};

const E: u16 = Input::MECHANIC;
const Q: u16 = Input::SPECIAL;

/// Degrees below the horizon, in the wire's own unit: a signed count of
/// 1/65536 of a turn.
fn down(degrees: i32) -> i16 {
    (-degrees * 65536 / 360) as i16
}

fn up(degrees: i32) -> i16 {
    -down(degrees)
}

fn run(w: &mut World, frames: u32, bits: u16, pitch: i16) {
    for _ in 0..frames {
        w.advance([Input::looking_at(bits, 0, pitch), Input::default()]);
    }
}

/// Press, let go, and let it play out.
fn tap(w: &mut World, bits: u16, pitch: i16, then: u32) {
    run(w, 2, bits, pitch);
    run(w, then, 0, pitch);
}

fn elementalist() -> World {
    let mut w = World::with_classes([Class::Elementalist, Class::Bulwark]);
    // Out of the platforms' way and well clear of the far wall, looking down
    // +X, so a trace has plain floor in front of it.
    w.players[0].pos = V3::new(Fx::from_int(-6), Fx::ZERO, Fx::from_int(8));
    w.players[1].pos = V3::new(Fx::from_int(10), Fx::ZERO, Fx::from_int(-8));
    w
}

fn stones_of(w: &World) -> Vec<Structure> {
    let Mechanic::Structures(slots) = w.players[0].mechanic else {
        panic!("not the class that owns stones");
    };
    slots.iter().flatten().copied().collect()
}

fn pillar(w: &World) -> sim::effects::Effect {
    *w.effects
        .iter()
        .flatten()
        .find(|e| e.kind == EffectKind::FirePillar)
        .expect("no fire pillar on the field")
}

/// How far ahead of the caster, along +X, something ended up.
fn ahead(w: &World, at: V3) -> f32 {
    at.x.sub(w.players[0].pos.x).to_f32_for_render()
}

/// Raise a stone, which the special needs, without moving.
fn with_a_stone(w: &mut World) {
    tap(w, E, 0, 4);
}

// ---------------------------------------------------------------------------
// Where an area ability lands
// ---------------------------------------------------------------------------

#[test]
fn an_area_ability_lands_where_you_are_looking_rather_than_a_step_ahead() {
    // The headline. It used to spawn a fixed distance straight ahead, which
    // meant the only way to place one anywhere was to walk there.
    let mut near = elementalist();
    with_a_stone(&mut near);
    tap(&mut near, Q, down(45), 30);

    let mut far = elementalist();
    with_a_stone(&mut far);
    tap(&mut far, Q, down(5), 30);

    let close = ahead(&near, pillar(&near).pos);
    let out = ahead(&far, pillar(&far).pos);
    assert!(
        close < out - 2.0,
        "looking down put the pillar {close:.1} m away and looking level {out:.1} m -- \
         the aim is doing nothing"
    );
    assert!(
        close > 0.0,
        "looking down put the pillar {close:.1} m away, which is behind the caster"
    );
}

#[test]
fn an_area_ability_cannot_be_placed_past_its_reach() {
    // The other half, and the reason the trace stops at a sphere rather than
    // running on to the terrain: reach is what a range number *means*, and an
    // ability you could drop on the far wall by looking at it would have none.
    let mut w = elementalist();
    with_a_stone(&mut w);
    let reach = sim::moves::get(Class::Elementalist, SLOT_SPECIAL).reach;
    tap(&mut w, Q, up(2), 30); // at the far wall, well beyond reach

    let out = ahead(&w, pillar(&w).pos);
    assert!(
        out <= reach.to_f32_for_render() + 0.01,
        "the pillar landed {out:.1} m away, past a reach of {:.1}",
        reach.to_f32_for_render()
    );
    assert!(
        out > reach.to_f32_for_render() - 0.01,
        "aiming past everything should still reach as far as the ability can, not {out:.1} m"
    );
}

#[test]
fn a_grounded_ability_always_lands_on_the_ground() {
    // A pillar of flame comes out of the floor. Aimed at the sky it has to
    // arrive somewhere it can come out of, and "max reach flat ahead" is the
    // only answer that keeps the range meaning what it says.
    for tilt in [up(80), up(40), up(5), 0, down(20), down(60)] {
        let mut w = elementalist();
        with_a_stone(&mut w);
        tap(&mut w, Q, tilt, 30);
        let at = pillar(&w).pos;
        assert_eq!(
            at.y.raw(),
            0,
            "aiming at tilt {tilt} left the pillar {:.2} m off the floor",
            at.y.to_f32_for_render()
        );
    }
}

#[test]
fn the_target_locks_when_the_move_starts() {
    // The same commitment facing is. A target you could drag through the
    // startup would let an area be slid onto someone during the wind-up, which
    // is the telegraph the whole class is built on being worth nothing.
    let mut w = elementalist();
    with_a_stone(&mut w);

    // Throw it looking well down, then look level for the rest of the move.
    run(&mut w, 2, Q, down(45));
    let locked = w.players[0].aim_at;
    run(&mut w, 40, 0, up(2));

    assert!(
        pillar(&w).pos.sub(locked).flat_len().raw() == 0,
        "the pillar followed the mouse after the move had started"
    );
}

#[test]
fn an_aimed_move_hits_where_it_was_aimed() {
    // The hitbox goes with the thing it places. If the pillar moved and the
    // burst that comes with it did not, the ability would be two abilities
    // pointing in different directions.
    let mut w = elementalist();
    with_a_stone(&mut w);
    run(&mut w, 2, Q, down(45));
    let locked = w.players[0].aim_at;
    run(&mut w, 20, 0, down(45));

    let mut seen = None;
    for _ in 0..10 {
        if let Some(box_out) = sim::state::hitbox(&w.players[0]) {
            seen = Some(box_out.centre);
            break;
        }
        run(&mut w, 1, 0, down(45));
    }
    let centre = seen.expect("the special never put a hitbox out");
    assert!(
        centre.sub(locked).flat_len().raw() == 0,
        "the burst came out at {centre:?} and the pillar stood at {locked:?}"
    );
}

// ---------------------------------------------------------------------------
// Stones
// ---------------------------------------------------------------------------

#[test]
fn a_stone_can_be_raised_at_your_own_feet() {
    // What the kit has always said Raise does -- "cast beneath yourself to
    // launch into the air" -- and what it could not do while the stone went a
    // fixed distance straight ahead.
    let mut w = elementalist();
    tap(&mut w, E, down(75), 2);
    let under = ahead(&w, stones_of(&w)[0].at);
    assert!(
        under.abs()
            < t::body_radius()
                .add(t::structure_radius())
                .to_f32_for_render(),
        "looking straight down raised a stone {under:.2} m away, which is not under you"
    );
}

#[test]
fn aiming_at_the_side_of_a_stone_puts_the_next_one_at_its_foot() {
    // A stone stands a whole body height, and abilities come out of the chest,
    // so from the ground you are always looking at a stone's *side* and never
    // at its top. Grounded means grounded: what you place arrives at the foot
    // of the thing you pointed at, the same as pointing at a wall.
    let mut w = elementalist();
    tap(&mut w, E, down(30), 30);
    let first = stones_of(&w)[0];
    assert_eq!(
        first.phase(),
        Phase::Standing,
        "the first stone never stood"
    );

    // Halfway up its face.
    let reach = ahead(&w, first.at);
    let rise = first.top().mul(Fx::ratio(1, 2)).sub(t::cast_height());
    let tilt = (rise.to_f32_for_render() / reach)
        .atan()
        .to_degrees()
        .round() as i32;
    tap(&mut w, E, down(-tilt), 2);

    let second = stones_of(&w)[1];
    assert_eq!(
        second.at.y.raw(),
        0,
        "aiming at the face of a stone put the next one {:.2} m up",
        second.at.y.to_f32_for_render()
    );
    assert!(
        ahead(&w, second.at) < reach,
        "the second stone came up past the one it was aimed at"
    );
}

#[test]
fn aiming_down_onto_a_stone_from_above_puts_the_next_one_on_top() {
    // The other way round, and the way a player actually stacks them: get above
    // the cap and the trace meets it, so what you place stands on it. The cap
    // of the cylinder has to be traced for this -- a tube with no lid would let
    // the ray straight through.
    let mut w = elementalist();
    tap(&mut w, E, down(30), 30);
    let first = stones_of(&w)[0];
    let over = first.at;

    // Straight above it, in the air, looking down -- and close enough that the
    // cap is inside Raise's reach, which is measured from the chest.
    w.players[0].pos = V3::new(over.x, Fx::from_int(3), over.z);
    w.players[0].grounded = false;
    tap(&mut w, E, down(89), 40);

    let second = stones_of(&w)[1];
    let gap = second.at.y.sub(first.top()).abs().to_f32_for_render();
    assert!(
        gap < 0.1,
        "the second stone settled at {:.2} m, not on the cap it was aimed at ({:.2} m)",
        second.at.y.to_f32_for_render(),
        first.top().to_f32_for_render()
    );
}

#[test]
fn a_stone_cannot_be_raised_out_of_reach() {
    let mut w = elementalist();
    tap(&mut w, E, up(2), 2);
    let out = ahead(&w, stones_of(&w)[0].at);
    assert!(
        out <= t::raise_reach().to_f32_for_render() + 0.01,
        "a stone came up {out:.1} m away, past a reach of {:.1}",
        t::raise_reach().to_f32_for_render()
    );
}

// ---------------------------------------------------------------------------
// The trace itself
// ---------------------------------------------------------------------------

#[test]
fn the_reach_sphere_bounds_what_the_terrain_can_do_to_the_aim() {
    // The corner case the sphere exists for. Aim a hair either side of a
    // platform's lip and the terrain hit jumps from the platform to the far
    // wall -- a fraction of a degree of mouse movement swinging the target
    // across the arena. Stopping at the reach bounds that jump to the ability's
    // own range, which is the most it could ever have meant.
    let stones = [None; MAX_PLAYERS * MAX_STRUCTURES];
    let from = V3::new(Fx::from_int(-12), t::cast_height(), Fx::ZERO);
    let reach = Fx::from_int(6);

    let mut last: Option<V3> = None;
    let mut worst = Fx::ZERO;
    for step in 0..400 {
        // Sweep through the angles that graze the near platform's edge.
        let tilt = down(2) + (step * 24) as i16;
        let dir = Input::looking_at(0, 0, tilt).look_dir();
        let at = aim::target(from, dir, reach, true, &stones);
        if let Some(prev) = last {
            worst = worst.max(at.sub(prev).flat_len());
        }
        last = Some(at);
    }
    assert!(
        worst.raw() <= reach.raw(),
        "one step of the mouse moved the target {:.1} m, more than the whole reach",
        worst.to_f32_for_render()
    );
}

#[test]
fn the_trace_stops_at_the_arena_rather_than_running_through_it() {
    let stones = [None; MAX_PLAYERS * MAX_STRUCTURES];
    // Clear of the two raised platforms, which reach four metres either side of
    // the middle -- a level ray from the centre of the arena meets one of those
    // long before it reaches a wall.
    let from = V3::new(Fx::ZERO, t::cast_height(), Fx::from_int(8));
    let far = Fx::from_int(60);
    // Level, down the +X axis: the wall stands just outside the play area.
    let dir = Input::looking_at(0, 0, 0).look_dir();
    let hit = aim::trace(from, dir, far, &stones).expect("level aim hit nothing at all");
    assert!(
        (hit.to_f32_for_render() - 14.0).abs() < 1.5,
        "the level trace stopped at {:.1} m; the wall is at 14",
        hit.to_f32_for_render()
    );
}

#[test]
fn a_free_placement_is_not_dragged_down_to_the_floor() {
    // Grounded is a property of the thing being placed. Something that is not
    // has to be able to sit in the air, or aiming up would mean nothing.
    let stones = [None; MAX_PLAYERS * MAX_STRUCTURES];
    let from = V3::new(Fx::ZERO, t::cast_height(), Fx::ZERO);
    let dir = Input::looking_at(0, 0, up(40)).look_dir();
    let at = aim::target(from, dir, Fx::from_int(5), false, &stones);
    assert!(
        at.y.raw() > t::cast_height().raw(),
        "an ungrounded placement aimed upward landed at {:.2} m",
        at.y.to_f32_for_render()
    );
}

// ---------------------------------------------------------------------------
// The wire
// ---------------------------------------------------------------------------

#[test]
fn pitch_reaches_the_simulation_and_changes_it() {
    // Pitch is gameplay now. If it were not in the snapshot's checksum, two
    // peers aiming differently would place abilities in different places and
    // agree that nothing was wrong.
    let mut level = elementalist();
    let mut tilted = elementalist();
    with_a_stone(&mut level);
    with_a_stone(&mut tilted);
    tap(&mut level, Q, down(5), 30);
    tap(&mut tilted, Q, down(45), 30);
    assert_ne!(
        level.checksum(),
        tilted.checksum(),
        "the same buttons at different pitches produced the same world"
    );
}

#[test]
fn looking_level_is_what_it_always_was() {
    // Every existing test aims level, and the numbers they pin were chosen
    // against a placement a fixed distance straight ahead. Level aim has to
    // still land there, or this pass changed the balance of the game as a side
    // effect of changing how it is aimed.
    let mut w = elementalist();
    with_a_stone(&mut w);
    tap(&mut w, Q, 0, 30);
    let reach = sim::moves::get(Class::Elementalist, SLOT_SPECIAL).reach;
    assert!(
        (ahead(&w, pillar(&w).pos) - reach.to_f32_for_render()).abs() < 0.05,
        "level aim put the pillar {:.2} m out instead of its reach of {:.2}",
        ahead(&w, pillar(&w).pos),
        reach.to_f32_for_render()
    );
}
