//! Where abilities go.
//!
//! One rule under all of it, and `crate::aim` is the only place that knows it:
//! **one raycast, from the camera through the crosshair, and the ability goes
//! to the first thing it meets.** These are the assertions for what that
//! buys — the thing you are pointing at is the thing you hit, whether you are
//! pointing at the floor, at a wall, at a person, or at the sky.

use sim::aim;
use sim::class::{Mechanic, Structure};
use sim::effects::EffectKind;
use sim::state::SLOT_SPECIAL;
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

/// Ask `aim` something about a world as it stands.
///
/// The scene is everything a ray can meet, and it is borrowed from copies —
/// which is also how the simulation builds it, so a test cannot accidentally
/// ask a question the game could not.
fn with_scene<T>(w: &World, ask: impl FnOnce(&aim::Scene) -> T) -> T {
    let stones = sim::stones::gather(&w.players);
    let players = w.players;
    let effects = w.effects;
    ask(&aim::Scene {
        stones: &stones,
        players: &players,
        effects: &effects,
        quarry: w.monster.as_ref(),
    })
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

/// Raise a stone without moving, for the tests that want one on the field as
/// terrain -- Fire pillar itself no longer needs one out to be cast.
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
    //
    // The steep angle is steeper than it once needed to be, and that is the
    // crosshair being honest rather than the aim being lazy: the ray starts at
    // the eye, which is well behind and above the fighter, so the reticle only
    // comes back to their own feet once the aim is most of the way down. It is
    // where the reticle *is* at each angle, which is the whole contract.
    let mut near = elementalist();
    with_a_stone(&mut near);
    tap(&mut near, Q, down(75), 30);

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

    // Measured from the point it is cast from, because that is what the range
    // is a radius about -- the crosshair's ray only chooses the direction, and
    // it starts at the eye rather than at the fighter's chest.
    let at = pillar(&w).pos;
    let cast = sim::aim::origin(w.players[0].pos);
    let out = at.sub(cast).len().to_f32_for_render();
    // A shade of slack, because a grounded ability is dropped onto the floor
    // after the range is measured, and a point pulled straight down is a little
    // further from the chest than it was.
    assert!(
        out <= reach.to_f32_for_render() + 0.2,
        "the pillar landed {out:.1} m from the caster, past a reach of {:.1}",
        reach.to_f32_for_render()
    );
    assert!(
        out > reach.to_f32_for_render() - 0.3,
        "aiming past everything should still reach about as far as the ability can, not {out:.1} m"
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
        // At ground level, or standing on the stone the caster has up -- which
        // is the only other surface in the arena here, and is still "on the
        // ground" in every sense that matters. What it must never be is in the
        // air, which is the thing aiming at the sky used to risk.
        assert!(
            at.y.raw() >= 0 && at.y.raw() <= t::structure_height().raw(),
            "aiming at tilt {tilt} left the pillar at {:.2} m, off any surface",
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
    let locked = w.players[0].aim_at();
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
    let locked = w.players[0].aim_at();
    run(&mut w, 20, 0, down(45));

    let mut seen = None;
    for _ in 0..10 {
        if let Some(box_out) = sim::state::hitbox(&w.players[0]) {
            seen = Some(box_out.centre());
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

    // Halfway up its face -- worked out from where the *eye* is, because that
    // is where the crosshair's ray starts, and the eye moves as the aim does.
    // A couple of rounds of "aim there, see where that put the eye, aim again"
    // settles it; the rig is smooth enough that it converges immediately.
    let reach = ahead(&w, first.at);
    let face = V3::new(first.at.x, first.top().mul(Fx::ratio(1, 2)), first.at.z);
    let mut tilt = 0i32;
    for _ in 0..6 {
        let eye = sim::camera::eye(w.players[0].pos, Input::looking_at(0, 0, down(tilt)));
        let flat = face.sub(eye).flat_len().to_f32_for_render();
        let rise = face.y.sub(eye.y).to_f32_for_render();
        tilt = (-rise.atan2(flat)).to_degrees().round() as i32;
    }
    tap(&mut w, E, down(tilt), 2);

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
    let mut w = elementalist();
    w.players[0].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::ZERO);
    let reach = Fx::from_int(6);

    let mut last: Option<V3> = None;
    let mut worst = Fx::ZERO;
    for step in 0..400 {
        // Sweep through the angles that graze the near platform's edge.
        let tilt = down(2) + (step * 24) as i16;
        let look = Input::looking_at(0, 0, tilt);
        let at = with_scene(&w, |scene| aim::grounded_path(0, look, reach, scene).to);
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
fn the_ray_stops_on_the_platform_rather_than_running_through_it() {
    // Terrain is a candidate, and the top of a platform counts as ground: it
    // is a surface you stand on, so a pillar planted there stands on it and a
    // shot aimed at it flies level over it. A ray that ran through to the
    // floor beyond would put both a metre and a half below where the crosshair
    // was.
    //
    // Swept rather than aimed at one angle, because where the eye sits is the
    // camera's business and this test is not about the camera.
    let mut w = elementalist();
    w.players[0].pos = V3::new(Fx::ZERO, Fx::ZERO, Fx::ZERO);
    w.players[1].pos = V3::new(Fx::from_int(-13), Fx::ZERO, Fx::from_int(-13));

    let on_the_platform = |at: V3| at.x.to_f32_for_render() > 5.0 && at.x.to_f32_for_render() < 9.0;
    let mut landed_on_top = false;
    for step in 0..90 {
        let look = Input::looking_at(0, 0, down(step));
        let seen = with_scene(&w, |scene| aim::sight(0, look, Fx::from_int(20), scene));
        if !on_the_platform(seen.at) {
            continue;
        }
        assert!(
            seen.at.y.to_f32_for_render() > 1.4,
            "the ray came out {:.2} m up inside the platform at x={:.1}",
            seen.at.y.to_f32_for_render(),
            seen.at.x.to_f32_for_render()
        );
        if (seen.at.y.to_f32_for_render() - 1.5).abs() < 0.1 {
            landed_on_top = true;
            assert_eq!(
                seen.met,
                aim::Met::Ground,
                "the top of a platform is a surface you stand on, so it is ground"
            );
        }
    }
    assert!(
        landed_on_top,
        "the sweep never once landed on the platform, so it proves nothing"
    );
}

#[test]
fn a_skillshot_is_not_dragged_down_to_the_floor() {
    // Grounded is a property of the thing being thrown. Something that is not
    // has to be able to end in the air, or aiming up would mean nothing.
    let mut w = elementalist();
    w.players[0].pos = V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(8));
    w.players[1].pos = V3::new(Fx::from_int(-13), Fx::ZERO, Fx::from_int(-13));
    let look = Input::looking_at(0, 0, up(40));
    let path = with_scene(&w, |scene| {
        aim::skillshot_path(0, look, Fx::from_int(5), scene)
    });
    assert!(
        path.to.y.raw() > path.from.y.raw(),
        "a skillshot aimed forty degrees up ended at {:.2} m, no higher than it left at",
        path.to.y.to_f32_for_render()
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
    // A hair short of flat-out reach rather than exactly it, and the hair is
    // the point: level aim runs out along a ray that starts at the eye, which
    // is above the chest, so it crosses the range sphere a little before the
    // full radius has been spent going forward. Tenths of a metre, on a five
    // metre ability -- the balance the older tests pinned is untouched.
    assert!(
        (ahead(&w, pillar(&w).pos) - reach.to_f32_for_render()).abs() < 0.25,
        "level aim put the pillar {:.2} m out instead of its reach of {:.2}",
        ahead(&w, pillar(&w).pos),
        reach.to_f32_for_render()
    );
}

// ---------------------------------------------------------------------------
// The one rule
// ---------------------------------------------------------------------------
//
// The player's frame of reference is the crosshair, so the property under all
// of these is the same: the thing the reticle is on is the thing the ability
// reaches. Every bug this file has ever caught has been the two drifting apart.

/// A stone standing fully out of the ground at a chosen spot, as terrain.
fn stone_at(w: &mut World, at: V3) {
    let mut stone = Structure::raised(at);
    stone.age = t::structure_rise();
    w.players[0].mechanic = Mechanic::Structures([Some(stone), None, None]);
}

#[test]
fn a_grounded_cast_lands_exactly_where_the_ray_landed() {
    // "If we hit the ground, cast it exactly there. Not a pixel different."
    let mut w = elementalist();
    for step in 0..40 {
        let look = Input::looking_at(0, 0, down(5 + step));
        let (seen, cast) = with_scene(&w, |scene| {
            (
                aim::sight(0, look, Fx::from_int(10), scene),
                aim::grounded_path(0, look, Fx::from_int(10), scene).to,
            )
        });
        if seen.met != aim::Met::Ground {
            continue;
        }
        assert_eq!(
            (cast.x.raw(), cast.y.raw(), cast.z.raw()),
            (seen.at.x.raw(), seen.at.y.raw(), seen.at.z.raw()),
            "the cast landed {:?} and the crosshair was on {:?}",
            cast,
            seen.at
        );
    }
}

#[test]
fn a_skillshot_aimed_at_the_floor_flies_level_over_the_spot() {
    // A shot that cannot land on the ground still has to go *at* the place the
    // player is pointing: straight up from that spot to the height it leaves
    // her at, and level from there.
    let mut w = elementalist();
    let look = Input::looking_at(0, 0, down(20));
    let (seen, path) = with_scene(&w, |scene| {
        (
            aim::sight(0, look, Fx::from_int(10), scene),
            aim::skillshot_path(0, look, Fx::from_int(10), scene),
        )
    });
    assert_eq!(
        seen.met,
        aim::Met::Ground,
        "fixture did not aim at the floor"
    );
    assert_eq!(
        (path.to.x.raw(), path.to.z.raw()),
        (seen.at.x.raw(), seen.at.z.raw()),
        "the shot ended over {:?} and the crosshair was on {:?}",
        path.to,
        seen.at
    );
    assert_eq!(
        path.to.y.raw(),
        path.from.y.raw(),
        "a shot aimed at the floor dived into it instead of flying level"
    );
}

#[test]
fn a_skillshot_ends_on_whatever_the_crosshair_is_on() {
    // The convergence, stated as bluntly as it can be. This is the bug that
    // keeps coming back: a ray from the *chest* along the *look angle* is
    // parallel to the crosshair's ray and never meets it, so the reticle sits
    // on one thing and the shot goes past it -- by more the further away it is.
    let mut w = elementalist();
    // A stone to the side of straight ahead, so a parallel ray would miss it
    // and a converging one cannot.
    stone_at(&mut w, V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(8)));

    let mut checked = 0;
    for step in 0..40 {
        let look = Input::looking_at(0, 0, down(step));
        let (seen, path) = with_scene(&w, |scene| {
            (
                aim::sight(0, look, Fx::from_int(14), scene),
                aim::skillshot_path(0, look, Fx::from_int(14), scene),
            )
        });
        if seen.met != aim::Met::Solid {
            continue;
        }
        checked += 1;
        assert_eq!(
            (path.to.x.raw(), path.to.y.raw(), path.to.z.raw()),
            (seen.at.x.raw(), seen.at.y.raw(), seen.at.z.raw()),
            "the shot ended at {:?} and the crosshair was on {:?}",
            path.to,
            seen.at
        );
    }
    assert!(checked > 0, "the sweep never met anything solid at all");
}

#[test]
fn nothing_between_the_camera_and_the_character_is_aimed_at() {
    // The eye sits behind the shoulder, so the ray starts behind the fighter.
    // A stone back there is scenery the camera is looking through, not a thing
    // the player is pointing at -- and aiming through your own cover is not a
    // mechanic anybody asked for.
    let mut clear = elementalist();
    let look = Input::looking_at(0, 0, down(10));
    let open = with_scene(&clear, |scene| aim::sight(0, look, Fx::from_int(10), scene));

    let behind = clear.players[0]
        .pos
        .sub(V3::new(Fx::from_int(3), Fx::ZERO, Fx::ZERO));
    stone_at(&mut clear, behind);
    let blocked = with_scene(&clear, |scene| aim::sight(0, look, Fx::from_int(10), scene));

    assert_eq!(
        (open.at.x.raw(), open.at.z.raw()),
        (blocked.at.x.raw(), blocked.at.z.raw()),
        "a stone standing behind the fighter moved the aim from {:?} to {:?}",
        open.at,
        blocked.at
    );
}

#[test]
fn a_body_in_the_way_is_what_you_are_pointing_at() {
    // Bodies are on the raycast's list. Putting the reticle on someone and
    // having the shot sail past them is the same complaint as all the others.
    let mut w = elementalist();
    w.players[0].pos = V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(8));
    w.players[1].pos = V3::new(Fx::from_int(6), Fx::ZERO, Fx::from_int(8));

    let mut found = false;
    for step in 0..30 {
        let look = Input::looking_at(0, 0, down(step));
        let seen = with_scene(&w, |scene| aim::sight(0, look, Fx::from_int(12), scene));
        let apart = seen.at.sub(w.players[1].pos).flat_len().to_f32_for_render();
        if apart < t::body_radius().to_f32_for_render() + 0.05 && seen.at.y.raw() > 0 {
            found = true;
            assert_eq!(
                seen.met,
                aim::Met::Solid,
                "a fighter read as ground, so a shot would fly level over their head"
            );
        }
    }
    assert!(found, "the sweep never crossed the other fighter at all");
}

// ---------------------------------------------------------------------------
// The swing, and its dead zone
// ---------------------------------------------------------------------------
//
// A swing is a body moving, so it does not raycast — but its *angle* comes from
// the camera, because melee happens in the air and on slopes and a swing pinned
// to the horizontal misses things plainly in front of you. With a dead zone
// below the horizon, because the camera sits above the shoulder: looking at
// someone at your own height means looking slightly down at them.

/// How far above the horizon a swing at this pitch comes out, in degrees.
fn swing_tilt(w: &World, pitch: i16) -> f32 {
    let reach = Fx::from_int(2);
    let path = aim::swing_path(
        w.players[0].pos,
        w.players[0].facing,
        Input::looking_at(0, 0, pitch),
        reach,
    );
    let rise = path.to.y.sub(path.from.y).to_f32_for_render();
    (rise / reach.to_f32_for_render())
        .clamp(-1.0, 1.0)
        .asin()
        .to_degrees()
}

#[test]
fn a_swing_follows_the_camera_upward_exactly() {
    let w = elementalist();
    for degrees in [5, 15, 30, 60] {
        let tilt = swing_tilt(&w, up(degrees));
        assert!(
            (tilt - degrees as f32).abs() < 1.0,
            "looking {degrees} degrees up swung {tilt:.1} degrees up"
        );
    }
}

#[test]
fn a_swing_stays_level_through_the_dead_zone_below_the_horizon() {
    // The whole point of the dead zone: you fight people by looking slightly
    // down at them, and the swing must not follow that into the floor.
    let w = elementalist();
    let dead = t::swing_level_to();
    for degrees in 0..=dead {
        let tilt = swing_tilt(&w, down(degrees));
        assert!(
            tilt.abs() < 1.0,
            "looking {degrees} degrees down -- inside the {dead}-degree dead zone -- \
             tilted the swing {tilt:.1} degrees"
        );
    }
}

#[test]
fn past_the_dead_zone_a_swing_follows_what_is_left_over() {
    // "-45 is the same swing as 0, -46 is the same swing tilted down 1 degree."
    let w = elementalist();
    let dead = t::swing_level_to();
    for over in [1, 5, 20] {
        let tilt = swing_tilt(&w, down(dead + over));
        assert!(
            (tilt + over as f32).abs() < 1.0,
            "looking {over} degrees past the dead zone tilted the swing {tilt:.1}, \
             not {} degrees down",
            over
        );
    }
}

#[test]
fn the_dead_zone_has_no_step_at_its_edge() {
    // A boundary the swing jumps across would be felt as the blade snapping to
    // a new angle for one degree of mouse movement.
    let w = elementalist();
    let dead = t::swing_level_to();
    let mut worst: f32 = 0.0;
    let mut last: Option<f32> = None;
    for tenth in -((dead + 20) * 10)..=200 {
        let tilt = swing_tilt(&w, (tenth * 65536 / 3600) as i16);
        if let Some(prev) = last {
            worst = worst.max((tilt - prev).abs());
        }
        last = Some(tilt);
    }
    assert!(
        worst < 0.4,
        "one tenth of a degree of mouse movement moved the swing {worst:.2} degrees"
    );
}

#[test]
fn a_swing_aimed_steeply_down_reaches_below_the_body() {
    // What the angle buys, stated as a position: the Champion's poke thrown
    // from the air at something underneath her. Level, the volume sits at her
    // own height; aimed down past the dead zone, it is below her feet.
    let thrown = |pitch: i16| {
        let mut w = World::with_classes([Class::Champion, Class::Bulwark]);
        w.players[1].pos = V3::new(Fx::from_int(12), Fx::ZERO, Fx::from_int(12));
        for _ in 0..30 {
            w.advance([Input::looking_at(Input::LEFT, 0, pitch), Input::default()]);
            if let Some(hb) = sim::state::hitbox(&w.players[0]) {
                return hb.to.y.sub(w.players[0].pos.y).to_f32_for_render();
            }
        }
        panic!("the poke never came out");
    };

    let level = thrown(0);
    let steep = thrown(down(t::swing_level_to() + 40));
    assert!(
        level.abs() < 0.1,
        "a level swing came out {level:.2} m off the body's own height"
    );
    assert!(
        steep < -0.5,
        "a swing aimed well past the dead zone came out {steep:.2} m from the body, \
         which is not below it"
    );
}

// ---------------------------------------------------------------------------
// At the mechanic
// ---------------------------------------------------------------------------

/// A Reaver with the shadow placed ahead of her, and the gap it sits at.
fn with_a_shadow() -> (World, V3) {
    let mut w = World::with_classes([Class::ShadowReaver, Class::Bulwark]);
    w.players[0].pos = V3::new(Fx::from_int(-6), Fx::ZERO, Fx::from_int(8));
    w.players[1].pos = V3::new(Fx::from_int(12), Fx::ZERO, Fx::from_int(-12));
    // `E` places it where the crosshair is, which is a grounded cast.
    run(&mut w, 2, E, down(20));
    run(&mut w, 2, 0, down(20));
    let Mechanic::Shadow { at: Some(spot) } = w.players[0].mechanic else {
        panic!("the fixture never placed a shadow");
    };
    (w, spot)
}

#[test]
fn the_blades_erupt_at_the_shadow_rather_than_on_the_caster() {
    // Guillotine lotus is specced "Range: at the shadow". It used to be
    // declared a swing with a reach of zero, which put its volume on the
    // Reaver's own body -- the move was unusable as written and the overlay
    // showed exactly that, a bubble sitting on her chest.
    let (mut w, shadow) = with_a_shadow();
    let stood = w.players[0].pos;
    assert!(
        shadow.sub(stood).flat_len().raw() > Fx::ONE.raw(),
        "the fixture placed the shadow on top of her, so this proves nothing"
    );

    let mut seen = None;
    for _ in 0..40 {
        run(&mut w, 1, Q, down(20));
        if let Some(hb) = sim::state::hitbox(&w.players[0]) {
            seen = Some(hb);
            break;
        }
    }
    let hb = seen.expect("Guillotine never put a volume out");
    assert!(
        hb.to.sub(shadow).flat_len().raw() < Fx::ONE.raw(),
        "the blades came out {:.1} m from the shadow",
        hb.to.sub(shadow).flat_len().to_f32_for_render()
    );
    assert!(
        hb.to.sub(stood).flat_len().raw() > Fx::ONE.raw(),
        "the blades came out on the caster's own body"
    );
}

#[test]
fn standing_where_the_shadow_is_is_what_gets_you_cut() {
    let (mut w, shadow) = with_a_shadow();
    let health = |w: &World| w.players[1].health;

    // Out of the way: the move is thrown and nothing happens.
    let mut clear = w.clone();
    run(&mut clear, 40, Q, down(20));
    assert_eq!(
        health(&clear),
        sim::state::max_health(),
        "Guillotine cut somebody standing nowhere near the shadow"
    );

    w.players[1].pos = V3::new(shadow.x, Fx::ZERO, shadow.z);
    run(&mut w, 40, Q, down(20));
    assert!(
        health(&w) < sim::state::max_health(),
        "standing on the shadow cost nothing when the blades came up"
    );
}
