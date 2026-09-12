//! Stones: the Elementalist's structures as things that take up space.
//!
//! A structure used to be a marker. It is a solid now, and these are the
//! assertions for what that means — stones moving each other, fighters standing
//! on them, and what it costs to be on the spot when one comes up. Phrased in
//! terms of what happens on the field rather than which number moved.

use sim::class::{MAX_STRUCTURES, Mechanic, Structure};
use sim::state::{Action, MAX_PLAYERS};
use sim::stones::Phase;
use sim::tuning as t;
use sim::{Fx, Input, V3, World};

const E: u16 = Input::MECHANIC;
const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

fn run(w: &mut World, frames: u32, a: u16, b: u16) {
    for _ in 0..frames {
        w.advance([Input::aimed(a, LOOK_RIGHT), Input::aimed(b, LOOK_LEFT)]);
    }
}

/// Press, let go, and let it play out. Holding the mechanic raises one stone,
/// but letting go is what lets the next press raise another.
fn tap(w: &mut World, button: u16, then: u32) {
    run(w, 2, button, 0);
    run(w, then, 0, 0);
}

/// The same, aimed at a point in the world rather than straight ahead.
///
/// Needed because the crosshair is the aim now: the ray starts at the eye,
/// which is well behind and above the fighter, so "level" no longer means
/// "along the ground" and a stone already standing there is something the ray
/// goes *over* rather than into. Where the eye ends up depends on the pitch, so
/// this settles the pitch against it -- a couple of rounds is plenty, the rig
/// being smooth.
fn tap_at(w: &mut World, button: u16, mark: V3, then: u32) {
    let stood = w.players[0].pos;
    let turns = |v: f32| (v * 65536.0 / std::f32::consts::TAU) as i32;
    let yaw = turns(
        mark.z
            .sub(stood.z)
            .to_f32_for_render()
            .atan2(mark.x.sub(stood.x).to_f32_for_render()),
    ) as u16;
    let mut tilt = 0i16;
    for _ in 0..6 {
        let eye = sim::camera::eye(stood, Input::looking_at(0, yaw, tilt));
        let flat = mark.sub(eye).flat_len().to_f32_for_render();
        let rise = mark.y.sub(eye.y).to_f32_for_render();
        tilt = turns(rise.atan2(flat)) as i16;
    }
    for _ in 0..2 {
        w.advance([
            Input::looking_at(button, yaw, tilt),
            Input::aimed(0, LOOK_LEFT),
        ]);
    }
    run(w, then, 0, 0);
}

fn elementalist() -> World {
    World::with_classes([sim::Class::Elementalist, sim::Class::Bulwark])
}

fn stones_of(w: &World, player: usize) -> Vec<Structure> {
    let Mechanic::Structures(slots) = w.players[player].mechanic else {
        panic!("player {player} is not the class that owns stones");
    };
    slots.iter().flatten().copied().collect()
}

fn stone(w: &World, slot: usize) -> Structure {
    let Mechanic::Structures(slots) = w.players[0].mechanic else {
        panic!("the Elementalist lost her mechanic");
    };
    slots[slot].expect("no stone in that slot")
}

/// Put stones on the field directly, at rest and fully out of the ground.
///
/// The knock between two stones needs one of them already moving, and nothing
/// in the kit throws one that far yet — the moves that will are not built. The
/// rule still has to be right before they arrive, so this states it directly.
fn place(w: &mut World, stones: &[Structure]) {
    let mut slots = [None; MAX_STRUCTURES];
    for (slot, s) in slots.iter_mut().zip(stones) {
        *slot = Some(*s);
    }
    w.players[0].mechanic = Mechanic::Structures(slots);
}

fn standing_at(x: i32, z: i32) -> Structure {
    Structure {
        at: V3::new(Fx::from_int(x), Fx::ZERO, Fx::from_int(z)),
        vel: V3::ZERO,
        // Past the rise, so it is a plain solid rather than something still
        // coming up.
        age: t::structure_rise() + 1,
        struck: 0,
        launched: false,
        launch_from: V3::ZERO,
        knock_struck: 0,
    }
}

fn flat_speed(v: V3) -> Fx {
    V3::new(v.x, Fx::ZERO, v.z).flat_len()
}

/// Stand the Elementalist so that raising a stone puts it under the other
/// fighter, and raise one.
fn raise_under_the_other(w: &mut World) {
    let victim = w.players[1].pos;
    w.players[0].pos = V3::new(victim.x.sub(t::raise_reach()), Fx::ZERO, victim.z);
    run(w, 2, E, 0);
}

// ---------------------------------------------------------------------------
// Stones against stones
// ---------------------------------------------------------------------------

#[test]
fn a_stone_raised_underneath_another_throws_it_into_the_air() {
    // The first thing two solids have to answer. A stone coming up where one
    // already stands cannot simply arrive inside it, and the only direction
    // that makes sense for earth erupting from below is up.
    let mut w = elementalist();
    tap(&mut w, E, 30); // one up, fully risen
    let resting = stone(&w, 0).at.y;
    assert_eq!(
        resting.raw(),
        0,
        "the first stone did not settle on the floor"
    );

    // A second, aimed at the foot of the first: the ray stops on its near
    // face, which settles onto the floor just short of its middle. That is as
    // close underneath as a player can put one now that the crosshair is the
    // aim -- pointing at where its base *would* be means pointing at the stone
    // itself, and a stone you point at is a surface you land on top of.
    let foot = {
        let first = stone(&w, 0).at;
        V3::new(first.x, Fx::ratio(1, 10), first.z)
    };
    tap_at(&mut w, E, foot, 0);
    let mut highest = Fx::ZERO;
    for _ in 0..90 {
        run(&mut w, 1, 0, 0);
        highest = highest.max(stone(&w, 0).at.y);
    }
    assert!(
        highest.raw() > t::structure_height().raw(),
        "the stone underneath did not lift the one above it: it reached {} against a stone {} tall",
        highest.to_f32_for_render(),
        t::structure_height().to_f32_for_render(),
    );
}

#[test]
fn a_stone_raised_off_centre_throws_the_other_one_clear() {
    // Straight up only if it sits square on the middle. A boulder coming up
    // under the edge of another flips it off, which is the only thing on the
    // field that gives a stone horizontal speed today -- and therefore the only
    // way one stone ever knocks into another.
    let mut w = elementalist();
    tap(&mut w, E, 30);
    let from = stone(&w, 0).at;

    // Step aside, so the next one comes up under the shoulder of the first --
    // aimed at its foot from where we now stand, which is what a player does.
    run(&mut w, 6, Input::D, 0);
    let foot = V3::new(from.x, Fx::ratio(1, 10), from.z);
    tap_at(&mut w, E, foot, 120);

    let thrown = stone(&w, 0).at.sub(from).flat_len();
    assert!(
        thrown.raw() > t::structure_radius().raw(),
        "an off-centre eruption barely moved the stone above it: {} m",
        thrown.to_f32_for_render()
    );
}

#[test]
fn a_stone_knocked_into_another_hands_over_its_speed_and_both_come_out_slower() {
    // An inelastic exchange, and deliberately so. A stone that kept everything
    // it brought would be relayed the length of the arena through a row of
    // them; one that kept nothing would stop dead and read as hitting a wall.
    let mut w = elementalist();
    let mut moving = standing_at(0, -8);
    let arriving = Fx::from_int(9);
    moving.vel.x = arriving;
    // Just outside touching distance, so the first frame is the one that meets.
    let mut resting = standing_at(0, -8);
    resting.at.x = t::structure_radius()
        .add(t::structure_radius())
        .add(Fx::ratio(1, 8));
    place(&mut w, &[moving, resting]);

    run(&mut w, 4, 0, 0);
    let after = stones_of(&w, 0);
    let (hit, struck) = (after[0].vel, after[1].vel);

    assert!(
        struck.x.raw() > 0,
        "the stone that was run into never moved, so nothing was knocked back"
    );
    assert!(
        hit.x.raw() < arriving.raw(),
        "the stone that did the knocking kept all its speed"
    );
    assert!(
        flat_speed(hit).add(flat_speed(struck)).raw() < arriving.raw(),
        "the pair came out of the exchange with as much speed as went in, so a stone \
         could be relayed across the arena for free"
    );
}

#[test]
fn stones_come_to_rest() {
    // Nothing on the field may slide forever. Without friction a knocked stone
    // travels until it finds a wall, which turns a shove into a delivery.
    let mut w = elementalist();
    let mut moving = standing_at(0, -8);
    moving.vel.x = Fx::from_int(9);
    place(&mut w, &[moving]);

    run(&mut w, 600, 0, 0);
    let speed = flat_speed(stones_of(&w, 0)[0].vel);
    assert!(
        speed.raw() < Fx::ratio(1, 10).raw(),
        "a knocked stone was still travelling at {} m/s ten seconds later",
        speed.to_f32_for_render()
    );
}

#[test]
fn a_stone_stops_at_the_wall_like_anything_else() {
    let mut w = elementalist();
    let mut moving = standing_at(0, -8);
    moving.vel.x = Fx::from_int(20);
    place(&mut w, &[moving]);

    run(&mut w, 300, 0, 0);
    let out = stones_of(&w, 0)[0].at.x;
    assert!(
        out.raw() < sim::arena::ARENA_HALF.raw(),
        "a stone left the arena: it is at {}",
        out.to_f32_for_render()
    );
}

// ---------------------------------------------------------------------------
// Stones against fighters
// ---------------------------------------------------------------------------

#[test]
fn a_fighter_lands_on_a_stone_instead_of_falling_through_it() {
    // The headline. Terrain you cannot get on top of is a wall, and the class
    // is supposed to be building a floor as well as cover.
    let mut w = elementalist();
    place(&mut w, &[standing_at(0, -8)]);
    let top = stones_of(&w, 0)[0].top();

    w.players[1].pos = V3::new(Fx::ZERO, Fx::from_int(4), Fx::from_int(-8));
    w.players[1].grounded = false;
    run(&mut w, 120, 0, 0);

    assert!(w.players[1].grounded, "the fighter never came to rest");
    assert_eq!(
        w.players[1].pos.y.raw(),
        top.raw(),
        "the fighter fell through the stone instead of standing on it"
    );
}

#[test]
fn standing_on_a_stone_stays_standing() {
    // A body resolved exactly on to a surface has nothing left to collide with,
    // so without a tolerance it reads as airborne every other frame -- and a
    // fighter who is airborne every other frame cannot jump, dodge or walk.
    let mut w = elementalist();
    place(&mut w, &[standing_at(0, -8)]);
    w.players[1].pos = V3::new(Fx::ZERO, Fx::from_int(4), Fx::from_int(-8));
    w.players[1].grounded = false;
    run(&mut w, 120, 0, 0);

    for _ in 0..60 {
        run(&mut w, 1, 0, 0);
        assert!(
            w.players[1].grounded,
            "the fighter flickered off the stone they were standing on"
        );
    }
}

#[test]
fn a_stone_is_cover() {
    // The other half of being solid, and the thing the versus ruling turns on:
    // a stone stops whoever walks into it, including the fighter who made it.
    let mut w = elementalist();
    place(&mut w, &[standing_at(0, 0)]);
    w.players[1].pos = V3::new(Fx::from_int(6), Fx::ZERO, Fx::ZERO);

    run(&mut w, 180, 0, Input::W); // player two is facing left, so W walks in
    let stopped = w.players[1].pos.x;
    let clear = t::body_radius().add(t::structure_radius());
    assert!(
        stopped.raw() >= clear.raw(),
        "the fighter walked through the stone: they are at {}",
        stopped.to_f32_for_render()
    );
}

#[test]
fn a_stone_landing_on_a_fighter_does_not_trap_them() {
    // A stone thrown up by an eruption comes down on the spot it left, and
    // somebody may be standing there. Dead centre there is no sideways to be
    // pushed, so the way out has to be up -- a body with nowhere to go is a
    // body stuck inside a stone.
    let mut w = elementalist();
    let victim = w.players[1].pos;
    let mut falling = Structure {
        at: V3::new(victim.x, Fx::from_int(5), victim.z),
        ..standing_at(0, 0)
    };
    falling.vel.y = Fx::from_int(-6);
    place(&mut w, &[falling]);

    run(&mut w, 240, 0, 0);
    let stone = stones_of(&w, 0)[0];
    let flat = V3::new(
        w.players[1].pos.x.sub(stone.at.x),
        Fx::ZERO,
        w.players[1].pos.z.sub(stone.at.z),
    )
    .flat_len();
    let clear = w.players[1].pos.y.raw() >= stone.top().raw()
        || flat.raw() >= t::body_radius().add(t::structure_radius()).raw();
    assert!(
        clear,
        "the fighter was left inside the stone: {} m from its centre, {} m up a {} m stone",
        flat.to_f32_for_render(),
        w.players[1].pos.y.to_f32_for_render(),
        stone.top().to_f32_for_render()
    );
}

#[test]
fn a_stone_erupting_underneath_carries_a_fighter_up_with_it() {
    // What makes raising one under your own feet a way up rather than a way to
    // be shoved aside. The moves that launch stones properly are not built yet;
    // the surface handing over its speed is the part that has to be right first.
    let mut w = elementalist();
    raise_under_the_other(&mut w);
    let mut highest = Fx::ZERO;
    for _ in 0..60 {
        run(&mut w, 1, 0, 0);
        highest = highest.max(w.players[1].pos.y);
    }
    assert!(
        highest.raw() > t::structure_height().raw(),
        "the fighter was lifted to {} m, which is no further than the stone itself came up",
        highest.to_f32_for_render()
    );
}

// ---------------------------------------------------------------------------
// The auto, aimed through a structure
// ---------------------------------------------------------------------------

/// Hold left click until Bolt becomes active, or fail the test trying.
fn cast_bolt(w: &mut World) {
    for _ in 0..30 {
        run(w, 1, Input::LEFT, 0);
        if matches!(w.players[0].action, Action::Active { kind: 0, .. }) {
            return;
        }
    }
    panic!("Bolt never became active");
}

#[test]
fn a_bolt_aimed_through_a_structure_kicks_it_instead_of_reaching_past_it() {
    // The auto reads what it is aimed through. See
    // docs/design/kits/elementalist.md.
    let mut w = elementalist();
    tap(&mut w, E, 30); // raise a structure ahead, and let it fully rise
    assert_eq!(
        stone(&w, 0).vel,
        V3::ZERO,
        "fixture's stone was already moving"
    );

    cast_bolt(&mut w);

    let after = stone(&w, 0);
    assert!(
        after.launched,
        "aiming Bolt through a structure did not kick it"
    );
    assert!(
        flat_speed(after.vel).raw() > 0,
        "a kicked stone did not pick up any speed"
    );
    assert_eq!(
        w.players[1].health,
        sim::state::max_health(),
        "the beam kicked the stone and poked the fighter beyond it as well"
    );
}

#[test]
fn a_kicked_stone_dies_off_over_the_back_of_its_travel() {
    // "Dies off in speed in the last quarter of its path" from the design
    // note this implements: full speed for most of the travel, decaying
    // toward the end rather than a flat friction the whole way.
    let mut w = elementalist();
    tap(&mut w, E, 30);
    cast_bolt(&mut w);
    let launch_speed = flat_speed(stone(&w, 0).vel);
    assert!(launch_speed.raw() > 0, "fixture never kicked the stone");

    // Early in the travel: still close to launch speed.
    run(&mut w, 4, 0, 0);
    let early = flat_speed(stone(&w, 0).vel);
    assert!(
        early.raw() > launch_speed.raw() / 2,
        "the stone lost most of its speed almost immediately: {} of {}",
        early.to_f32_for_render(),
        launch_speed.to_f32_for_render()
    );

    // Let it run out its whole travel; by then it must have died off.
    run(&mut w, 90, 0, 0);
    let late = flat_speed(stone(&w, 0).vel);
    assert!(
        late.raw() < early.raw(),
        "a kicked stone was still at full speed at the end of its travel"
    );
}

#[test]
fn a_kicked_stone_hurts_a_fighter_it_is_still_moving_fast_enough_to_catch() {
    let mut w = elementalist();
    // Put the structure directly between the two fighters, close enough that
    // it is still near launch speed when it reaches the other one.
    //
    // Raise is aimed now, so a level look puts the stone at the far end of its
    // reach rather than a fixed step ahead -- the other fighter stands that
    // much further out again to keep the travel short.
    // Clear of the raised platforms in z, which reach four metres either side
    // of the middle -- the far fighter now stands past where one of them is.
    w.players[0].pos = V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(8));
    let gap = t::raise_reach().add(Fx::ratio(3, 2));
    w.players[1].pos = V3::new(gap, Fx::ZERO, Fx::from_int(8));
    run(&mut w, 2, E, 0);
    run(&mut w, 30, 0, 0); // let the structure fully rise

    let before = w.players[1].health;
    cast_bolt(&mut w);
    let mut staggered = false;
    for _ in 0..40 {
        run(&mut w, 1, 0, 0);
        staggered |= matches!(w.players[1].action, Action::Stagger { .. });
    }

    assert!(
        w.players[1].health < before,
        "a kicked stone passed straight through the fighter beyond it"
    );
    assert!(
        staggered,
        "getting run over by a kicked stone did not stagger anyone"
    );
}

// ---------------------------------------------------------------------------
// The telegraph and the eruption
// ---------------------------------------------------------------------------

#[test]
fn a_stone_churns_first_and_erupts_second() {
    // The phases come off the rise curve rather than a frame count, so
    // reshaping the rise moves the telegraph with it. Both phases have to
    // actually happen, and in that order.
    let mut w = elementalist();
    tap(&mut w, E, 0);
    let mut seen = Vec::new();
    for _ in 0..(t::structure_rise() + 4) {
        let phase = stone(&w, 0).phase();
        if seen.last() != Some(&phase) {
            seen.push(phase);
        }
        run(&mut w, 1, 0, 0);
    }
    assert_eq!(
        seen,
        vec![Phase::Churning, Phase::Erupting, Phase::Standing],
        "a stone did not churn, then erupt, then stand"
    );
}

#[test]
fn the_ground_churning_underfoot_slows_you() {
    // The telegraph made tactile: you feel the stone before you see it.
    let mut w = elementalist();
    raise_under_the_other(&mut w);
    run(&mut w, 1, 0, 0);

    assert_eq!(
        stone(&w, 0).phase(),
        Phase::Churning,
        "fixture is past the churn already"
    );
    assert!(
        w.players[1].slowed > 0,
        "standing over a stone coming up cost nothing at all"
    );
}

#[test]
fn the_churn_is_a_warning_and_the_drain_field_is_a_wall() {
    // Two slows, and they must not be the same slow. A telegraph that also
    // stopped you moving would make the eruption unavoidable rather than
    // readable, which is the opposite of what a telegraph is for.
    assert!(
        t::stone_churn_slow().raw() > t::spike_slow().raw(),
        "the churn slows you as hard as a drain field does"
    );
    assert!(
        t::stone_churn_slow().raw() < Fx::ONE.raw(),
        "the churn does not slow you at all"
    );
}

#[test]
fn jumping_clears_the_churn() {
    // It is felt through the floor. Getting off the floor before it comes up is
    // the answer, and that answer is what makes the telegraph fair.
    let mut w = elementalist();
    run(&mut w, 1, 0, Input::SPACE);
    run(&mut w, 8, 0, 0);
    assert!(
        !w.players[1].grounded,
        "fixture never got the fighter off the ground"
    );

    raise_under_the_other(&mut w);
    run(&mut w, 2, 0, 0);
    assert_eq!(
        stone(&w, 0).phase(),
        Phase::Churning,
        "fixture is past the churn already"
    );
    assert_eq!(
        w.players[1].slowed, 0,
        "a fighter in the air was slowed by the ground churning under them"
    );
}

#[test]
fn the_eruption_costs_health_and_a_stagger() {
    // Ignoring the telegraph has to cost something, or the telegraph is
    // decoration. Small, because the stagger is the real punishment.
    let mut w = elementalist();
    let before = w.players[1].health;
    raise_under_the_other(&mut w);
    run(&mut w, t::structure_rise() as u32 + 2, 0, 0);

    assert!(
        w.players[1].health < before,
        "the eruption went straight through the fighter standing on it"
    );
    assert!(
        before - w.players[1].health <= t::stone_erupt_damage(),
        "the eruption hit more than once"
    );
    assert!(
        matches!(w.players[1].action, Action::Stagger { .. }),
        "the eruption did not stagger, so it buys the Elementalist nothing"
    );
}

#[test]
fn the_eruption_catches_a_fighter_once() {
    // A stone erupts once. Without the per-fighter mark it would hit on every
    // frame of the burst, which is several times the listed damage for free.
    let mut w = elementalist();
    let before = w.players[1].health;
    raise_under_the_other(&mut w);
    run(&mut w, 600, 0, 0);
    assert_eq!(
        before - w.players[1].health,
        t::stone_erupt_damage(),
        "the eruption kept hitting"
    );
}

#[test]
fn a_stone_never_touches_the_fighter_who_raised_it() {
    // She raises them under her own feet on purpose. A class that staggers
    // itself doing the thing it is for is not a class.
    let mut w = elementalist();
    let before = w.players[0].health;
    // Put her on the spot her own stone comes up on: two presses without
    // moving, so the second one comes up where she is standing after walking
    // on to the first.
    w.players[0].pos = V3::new(Fx::ZERO, Fx::ZERO, Fx::ZERO);
    run(&mut w, 2, E, 0);
    w.players[0].pos = V3::new(t::raise_reach(), Fx::ZERO, Fx::ZERO);
    run(&mut w, 600, 0, 0);

    assert_eq!(
        w.players[0].health, before,
        "the Elementalist was hurt by her own stone"
    );
    assert_eq!(
        w.players[0].slowed, 0,
        "the Elementalist was slowed by her own stone"
    );
}

// ---------------------------------------------------------------------------
// The snapshot
// ---------------------------------------------------------------------------

#[test]
fn a_stone_in_flight_is_part_of_the_snapshot() {
    // Stones move now, so a rollback that did not carry their motion would
    // rewind the fighters and leave the terrain where it was.
    let mut a = elementalist();
    let mut moving = standing_at(0, -8);
    moving.vel.x = Fx::from_int(9);
    place(&mut a, &[moving]);
    let mut b = a.clone();
    let still = a.checksum();

    run(&mut a, 1, 0, 0);
    assert_ne!(
        still,
        a.checksum(),
        "a stone moved without the checksum noticing"
    );

    run(&mut b, 1, 0, 0);
    assert_eq!(a.checksum(), b.checksum(), "the same frame twice disagreed");
    assert_eq!(a.players, b.players);
}

#[test]
fn stones_are_stepped_for_both_fighters() {
    // Indexing the field by owner is what lets two Elementalists share an
    // arena. Nothing in the loop may assume there is only one of them.
    let mut w = World::with_classes([sim::Class::Elementalist; MAX_PLAYERS]);
    run(&mut w, 2, E, E);
    run(&mut w, 60, 0, 0);
    for player in 0..MAX_PLAYERS {
        assert_eq!(
            stones_of(&w, player).len(),
            1,
            "player {player} lost the stone they raised"
        );
        assert_eq!(
            stones_of(&w, player)[0].phase(),
            Phase::Standing,
            "player {player}'s stone never finished coming up"
        );
    }
}
