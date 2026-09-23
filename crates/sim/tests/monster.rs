//! The creature's rules, and the rules of standing on one.
//!
//! These encode design decisions, so a failure here is either a bug or a
//! decision that changed without `docs/design/monsters.md` changing with it.

use sim::fixed::Fx;
use sim::math::{atan2_turns, wrap_turns};
use sim::monster::{self, Doing, Monster, Quarry};
use sim::state::{MAX_PLAYERS, Phase};
use sim::{Class, Input, V3, World};

fn hunt() -> World {
    hunt_as(Class::Champion)
}

/// A hunt fought by somebody else. Most of these are about the animal and do
/// not care who is standing on it; the one that asks where a shot went needs a
/// class that has one.
fn hunt_as(class: Class) -> World {
    World::hunt([class; MAX_PLAYERS])
}

fn run(w: &mut World, frames: u32, a: Input) {
    for _ in 0..frames {
        w.advance([a, Input::default()]);
    }
}

/// Run a move and report whether the rider ever came off during it.
///
/// Not "were they aboard at the end": a thrown rider lands, and the creature is
/// right there, so they get back on. What is being tested is that the buck
/// removed them, not where they were standing a second later.
fn thrown_during(w: &mut World, kind: u8, input: Input) -> bool {
    w.monster.as_mut().expect("a hunt has a creature").doing = Doing::Startup {
        kind,
        left: monster::attack(kind).startup,
    };
    for _ in 0..monster::attack(kind).total() {
        w.advance([input, Input::default()]);
        if !w.players[0].aboard() {
            return true;
        }
    }
    false
}

/// Hold the creature in its walking state, so a test about being carried is
/// about being carried rather than about surviving whatever it decided to do.
fn walk_only(w: &mut World, frames: u32) {
    for _ in 0..frames {
        let beast = w.monster.as_mut().expect("a hunt has a creature");
        beast.doing = Doing::Prowl;
        // Its pause between moves, held open. Left to itself it would start
        // something on the first frame, and a creature mid-move neither steers
        // nor walks -- which would make this a test of standing still.
        beast.brain.think_left = u16::MAX;
        w.advance([Input::default(); MAX_PLAYERS]);
    }
}

/// Put a fighter on the creature's back, the way landing on it would.
fn board(w: &mut World, part: usize) {
    let beast = w.monster.expect("a hunt has a creature");
    let shape = monster::shape(part);
    let mid = shape.min.add(shape.max).scale(Fx::ratio(1, 2));
    let spot = V3::new(mid.x, shape.max.y.add(Fx::ratio(1, 8)), mid.z);
    w.players[0].pos = beast.world_of(part, spot);
    w.players[0].vel = V3::new(Fx::ZERO, Fx::ratio(-1, 1), Fx::ZERO);
    w.players[0].grounded = false;
    // One tick lands them.
    run(w, 1, Input::default());
}

// ---------------------------------------------------------------------------
// Body space
// ---------------------------------------------------------------------------

#[test]
fn body_space_and_world_space_are_exact_inverses() {
    // Everything about the ride rests on this. If the round trip drifted, a
    // rider would creep across the creature's back every frame it turned.
    let mut beast = Monster::new();
    beast.pos = V3::new(Fx::from_int(3), Fx::ZERO, Fx::from_int(-2));
    beast.yaw = Fx::ratio(37, 100);
    let s = beast.rig();
    for local in [
        V3::new(Fx::from_int(1), Fx::from_int(2), Fx::ratio(-3, 4)),
        V3::new(Fx::from_int(-4), Fx::ratio(7, 4), Fx::from_int(1)),
        V3::ZERO,
    ] {
        let back = s.to_body(s.to_world(local));
        for (a, b) in [(back.x, local.x), (back.y, local.y), (back.z, local.z)] {
            // Two millimetres, which is the sine table's own resolution
            // compounded through four multiplies. It matters that this is
            // *bounded* rather than that it is zero: a rider's body-space
            // position is kept across frames rather than re-derived, precisely
            // so that a bound like this cannot accumulate into a crawl.
            assert!(
                a.sub(b).abs().raw() < 160,
                "round trip drifted: {a:?} vs {b:?}"
            );
        }
    }
}

#[test]
fn an_angle_survives_the_trip_through_a_direction_and_back() {
    // `atan2_turns` is what the control algorithm steers by. Half a degree of
    // error there is a creature that visibly aims past you.
    let mut worst = 0;
    for i in 0..1000 {
        let turns = Fx::ratio(i, 1000);
        let v = V3::from_turns(turns);
        worst = worst.max(wrap_turns(atan2_turns(v.z, v.x).sub(turns)).abs().raw());
    }
    assert!(worst < 20, "angle round trip is off by {worst} raw units");
}

#[test]
fn the_ridge_is_the_only_place_worth_hitting() {
    // The whole reason to climb. If the armour ever stopped mattering, the
    // fight would be a damage race on the nearest surface.
    let ridge = monster::vulnerability(monster::RIDGE);
    for part in [
        monster::HEAD,
        monster::NECK,
        monster::BARREL,
        monster::TAIL_BASE,
        monster::TAIL_TIP,
        monster::FORELEG_L,
        monster::HINDLEG_R,
    ] {
        assert!(
            monster::vulnerability(part).raw() < ridge.raw(),
            "{} is as soft as the ridge",
            monster::PART_NAMES[part]
        );
    }
}

#[test]
fn both_weak_points_are_out_of_reach_from_the_ground() {
    // Geometry rather than a rule: both soft places sit above a standing
    // fighter's hurtbox, so getting at either means getting off the floor.
    //
    // Measured in the **world**, through the rig. The boxes are authored in
    // their own bone's frame now, and a local `y` is not a height: the same
    // number means one thing on a level spine and another the moment anything
    // is pitched.
    let beast = Monster::new();
    let rig = beast.rig();
    for part in [monster::RIDGE, monster::NAPE] {
        let shape = monster::shape(part);
        let low = rig
            .part_to_world(part, V3::new(shape.min.x, shape.min.y, Fx::ZERO))
            .y;
        assert!(
            low.raw() > sim::tuning::body_height().raw(),
            "the {} starts at {:?} m, inside a standing fighter's reach",
            monster::PART_NAMES[part],
            low
        );
    }
}

#[test]
fn the_nape_pays_better_than_the_ridge_and_is_harder_to_get_to() {
    // The reason to keep climbing once you are on. The ridge is where a rider
    // arrives; the nape is another walk forward, past the shoulders, and it is
    // worth the trip.
    assert!(
        monster::vulnerability(monster::NAPE).raw() > monster::vulnerability(monster::RIDGE).raw(),
        "the nape is no softer than the ridge, so there is no reason to walk to it"
    );
    let beast = Monster::new();
    let rig = beast.rig();
    let forward = |part: usize| rig.part_to_world(part, monster::shape(part).max).x;
    assert!(
        forward(monster::NAPE) > forward(monster::RIDGE),
        "the nape is not further forward than the ridge, so it is not further to walk"
    );
}

// ---------------------------------------------------------------------------
// The ride
// ---------------------------------------------------------------------------

#[test]
fn landing_on_a_mountable_part_puts_you_on_it() {
    let mut w = hunt();
    board(&mut w, monster::BARREL);
    assert!(w.players[0].aboard(), "landing on the back did not mount");
    assert_eq!(w.players[0].mount as usize, monster::BARREL);
}

#[test]
fn a_rider_is_carried_by_the_creature_rather_than_left_behind() {
    let mut w = hunt();
    board(&mut w, monster::BARREL);
    // Something on the ground for it to walk at, so that it actually goes
    // somewhere and the test is about being carried rather than about standing
    // on a stationary object.
    w.players[1].pos = V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(12));
    let held = w.players[0].local;
    let before = w.monster.unwrap().pos;
    walk_only(&mut w, 150);
    let after = w.monster.unwrap();
    assert!(w.players[0].aboard(), "fell off while doing nothing");
    let travelled = after.pos.sub(before).flat_len();
    assert!(
        travelled.raw() > Fx::ratio(1, 4).raw(),
        "the creature did not move, so this proves nothing"
    );
    // Standing still means standing still *on the creature*.
    let drift = w.players[0].local.sub(held).flat_len();
    assert!(
        drift.raw() < Fx::ratio(1, 4).raw(),
        "a rider holding no keys slid {drift:?} across the back"
    );
}

#[test]
fn the_creature_turning_carries_the_riders_aim_with_it() {
    // This is what makes movement relative to the surface. Without it, holding
    // forward walks you off the side as soon as the animal turns.
    let mut w = hunt();
    board(&mut w, monster::BARREL);
    w.players[1].pos = V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(12));
    let start = w.monster.unwrap().yaw;
    walk_only(&mut w, 150);
    let turned = wrap_turns(w.monster.unwrap().yaw.sub(start));
    assert!(
        turned.abs().raw() > Fx::ratio(1, 50).raw(),
        "the creature did not turn, so this proves nothing"
    );
    let carried = w.players[0].carry_yaw;
    assert!(
        carried.sub(turned).abs().raw() < Fx::ratio(1, 100).raw(),
        "carried {carried:?} of a {turned:?} turn"
    );
}

#[test]
fn the_creature_turning_carries_the_camera_too() {
    // The other half of the line above, and the half that went missing.
    // `carry_yaw` is the rider's whole frame of reference turning, not just
    // their body: the eye, the ray the crosshair draws out of it, the facing
    // and the direction `W` walks are one angle, and the moment two of them
    // disagree the character is drawn looking off to one side of the screen
    // and the crosshair stops meaning what it says.
    //
    // Asked through a skillshot, because that is the observable: a shot is
    // aimed by the camera's ray and the body is turned by the facing, so if
    // the two are still the same angle after the animal has swung a long way
    // round, nothing is being added in one place and forgotten in another.
    let mut w = hunt_as(Class::Elementalist);
    board(&mut w, monster::BARREL);
    w.players[1].pos = V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(12));
    walk_only(&mut w, 150);
    let carried = w.players[0].carry_yaw;
    assert!(
        carried.abs().raw() > Fx::ratio(1, 50).raw(),
        "the creature barely turned ({carried:?}), so this proves nothing"
    );

    // Her bolt, thrown with the hand perfectly still. The mouse has not moved
    // all ride; everything the shot knows about where to go came from the
    // animal turning underneath her.
    run(&mut w, 1, Input::aimed(Input::LEFT, 0));
    let me = w.players[0];
    assert!(me.aboard(), "fell off before the shot went out");
    let shot = me.aim_path;
    assert!(
        shot.length().raw() > 0,
        "the bolt was never aimed, so this proves nothing"
    );
    let dir = shot.dir();
    let bearing = atan2_turns(dir.z, dir.x);
    let facing = atan2_turns(me.facing.z, me.facing.x);
    let apart = wrap_turns(bearing.sub(facing));
    assert!(
        apart.abs().raw() < Fx::ratio(1, 200).raw(),
        "the shot went {apart:?} of a turn away from where she is facing, \
         which is a camera that has come loose from the fighter"
    );
}

#[test]
fn jumping_is_how_you_leave() {
    let mut w = hunt();
    board(&mut w, monster::BARREL);
    run(&mut w, 1, Input::new(Input::SPACE));
    assert!(!w.players[0].aboard(), "jumping did not leave the ride");
    assert!(w.players[0].vel.y.raw() > 0, "the jump had no rise in it");
}

#[test]
fn the_slam_beats_even_a_brace() {
    // The move you are meant to leave for rather than answer.
    let mut w = hunt();
    board(&mut w, monster::BARREL);
    assert!(
        thrown_during(&mut w, monster::SLAM, Input::new(Input::CROUCH)),
        "a rider braced through a rear-and-slam, which is supposed to be the \
         one nothing holds through"
    );
}

#[test]
fn walking_off_the_edge_drops_you() {
    let mut w = hunt();
    board(&mut w, monster::BARREL);
    // Straight off the side, in the creature's own frame. The aim has to point
    // along the animal's own axis, because movement is camera-relative and the
    // camera is wherever the fighter is looking.
    w.players[0].local.z = monster::shape(monster::BARREL).max.z;
    let side = V3::from_turns(w.monster.unwrap().yaw.add(Fx::from_raw(1 << 14)));
    let aim = (atan2_turns(side.z, side.x).raw() as u32 & 0xFFFF) as u16;
    for _ in 0..30 {
        w.advance([Input::aimed(Input::W, aim), Input::default()]);
    }
    assert!(
        !w.players[0].aboard(),
        "walked off the back and stayed on it"
    );
}

#[test]
fn the_tail_and_the_back_are_one_animal_to_walk_around() {
    // The tail sits two thirds of a metre below the barrel. Without a step-up
    // the only route between them is a jump nobody would think to try, and the
    // climb dead-ends on the tail.
    let mut w = hunt();
    board(&mut w, monster::TAIL_BASE);
    assert_eq!(w.players[0].mount as usize, monster::TAIL_BASE);
    let beast = w.monster.unwrap();
    let aim = sim::math::atan2_turns(V3::from_turns(beast.yaw).z, V3::from_turns(beast.yaw).x);
    let forward = Input::aimed(Input::W, (aim.raw() as u32 & 0xFFFF) as u16);
    for _ in 0..90 {
        // Held in its pause between moves, so this is a test of the staircase
        // rather than of surviving a slam halfway up it.
        w.monster
            .as_mut()
            .expect("a hunt has a creature")
            .brain
            .think_left = u16::MAX;
        w.advance([forward, Input::default()]);
        if w.players[0].mount as usize == monster::BARREL {
            return;
        }
    }
    panic!(
        "walked forward off the tail for a second and a half and never reached \
         the back -- ended on {}",
        if w.players[0].aboard() {
            monster::PART_NAMES[w.players[0].mount as usize]
        } else {
            "the floor"
        }
    );
}

// ---------------------------------------------------------------------------
// The control algorithm
// ---------------------------------------------------------------------------

#[test]
fn a_committed_move_locks_the_creatures_facing() {
    // Same rule the fighters have, for the same reason: without it a whiff can
    // be rescued by turning after the fact, and whiff punishment is most of the
    // game.
    let mut beast = Monster::new();
    let far = Quarry {
        pos: V3::new(Fx::from_int(6), Fx::ZERO, Fx::from_int(6)),
        vel: V3::ZERO,
        alive: true,
        aboard: false,
    };
    beast.doing = Doing::Startup {
        kind: monster::BITE,
        left: 30,
    };
    beast.yaw_rate = Fx::ZERO;
    let before = beast.yaw;
    for _ in 0..20 {
        beast.step(&[far]);
    }
    assert!(
        wrap_turns(beast.yaw.sub(before)).abs().raw() < Fx::ratio(1, 200).raw(),
        "it steered mid-move: {:?} to {:?}",
        before,
        beast.yaw
    );
}

#[test]
fn it_turns_toward_a_target_but_not_instantly() {
    let mut beast = Monster::new();
    let behind = Quarry {
        pos: V3::new(Fx::from_int(-8), Fx::ZERO, Fx::ZERO),
        vel: V3::ZERO,
        alive: true,
        aboard: false,
    };
    // Held in its pause between moves for the whole test. Its charge reaches
    // eight metres, and a committed move locks the yaw -- which is correct, and
    // makes this a test of committing rather than of the turn controller it is
    // about.
    beast.brain.think_left = u16::MAX;
    beast.step(&[behind]);
    let after_one = wrap_turns(beast.yaw).abs();
    assert!(
        after_one.raw() < Fx::ratio(1, 50).raw(),
        "it snapped half a turn in one frame"
    );
    for _ in 0..180 {
        beast.brain.think_left = u16::MAX;
        beast.step(&[behind]);
    }
    let error = wrap_turns(Fx::ratio(1, 2).sub(beast.yaw)).abs();
    assert!(
        error.raw() < Fx::ratio(1, 10).raw(),
        "three seconds was not enough to come about: still {error:?} off"
    );
}

#[test]
fn it_acts_on_what_it_last_looked_at_rather_than_on_the_present() {
    // The whole difficulty model. If it tracked continuously there would be no
    // such thing as a good read, because there would be nothing to read.
    let mut beast = Monster::new();
    let seen = Quarry {
        pos: V3::new(Fx::from_int(6), Fx::ZERO, Fx::ZERO),
        vel: V3::ZERO,
        alive: true,
        aboard: false,
    };
    beast.step(&[seen]);
    let remembered = beast.brain.seen;
    // Teleport the target. Until its next glance it should still be working
    // from where the target was.
    let moved = Quarry {
        pos: V3::new(Fx::from_int(-6), Fx::ZERO, Fx::from_int(6)),
        ..seen
    };
    beast.step(&[moved]);
    assert_eq!(
        beast.brain.seen, remembered,
        "it noticed a move inside its own glance window"
    );
    for _ in 0..sim::tuning::glance_frames() + 1 {
        beast.step(&[moved]);
    }
    assert_ne!(beast.brain.seen, remembered, "it never looked again");
}

#[test]
fn a_flinch_never_interrupts_a_live_hitbox() {
    // A creature whose hit can be cancelled by being hit is a creature you
    // never have to trade with, and trading is most of the ground game.
    let mut beast = Monster::new();
    beast.doing = Doing::Active {
        kind: monster::BITE,
        left: 3,
    };
    // Enough to flinch it, and short of what breaks its poise *or* its nerve.
    // Two things do interrupt a live hitbox -- a topple and a big enough burst
    // -- and both of them have to be earned. An ordinary hit does not.
    let flinching = sim::tuning::flinch_threshold();
    assert!(
        flinching < beast.interrupt_bar(),
        "a single flinching hit is already enough to interrupt, so there is no \
         window in which trading works"
    );
    beast.take_hit(monster::BARREL, flinching);
    assert!(
        matches!(beast.doing, Doing::Active { .. }),
        "an active frame was cancelled by an ordinary hit: {:?}",
        beast.doing
    );
}

#[test]
fn enough_damage_in_a_short_enough_window_does_interrupt_it() {
    // The other half, and the reason there is a threshold rather than a flat
    // rule: a burst big enough to stop a charge is a decision worth building a
    // kit around. See `docs/design/monsters.md` §4.
    let mut beast = Monster::new();
    beast.doing = Doing::Active {
        kind: monster::CHARGE,
        left: 12,
    };
    let bar = beast.interrupt_bar();
    beast.take_hit(monster::BARREL, bar * 4);
    assert!(
        matches!(beast.doing, Doing::Flinch { .. }),
        "a burst past the interrupt threshold left the charge running: {:?}",
        beast.doing
    );
    assert_eq!(
        beast.strain, 0,
        "the interrupt did not spend the strain, so the next hit gets one free"
    );
}

#[test]
fn both_thresholds_fall_as_it_is_worn_down() {
    // The arc of a hunt in one property: methodical while the animal is fresh,
    // frantic once it is not.
    let fresh = Monster::new();
    let mut spent = Monster::new();
    spent.health = fresh.health / 10;
    assert!(
        spent.cc_bar() < fresh.cc_bar() && spent.interrupt_bar() < fresh.interrupt_bar(),
        "a nearly-dead creature is no easier to move around than a fresh one"
    );
}

#[test]
fn crowd_control_does_nothing_to_a_creature_that_is_not_hurting() {
    // The whole point of a threshold. A knock-up thrown at a fresh Ridgeback is
    // a wasted button; the same knock-up after a burst is a trip.
    let mut fresh = Monster::new();
    let lift = monster::Control::launching(Fx::from_int(12));
    assert!(
        !fresh.take_control(lift).anything(),
        "a fresh creature took crowd control cold"
    );

    let mut reeling = Monster::new();
    reeling.take_hit(monster::BARREL, reeling.cc_bar() * 4);
    assert!(
        reeling.susceptible(),
        "a burst four times the threshold did not make it susceptible"
    );
    let took = reeling.take_control(lift);
    assert!(
        took.stumbled,
        "a knock-up on a reeling creature did nothing"
    );
    assert!(
        matches!(reeling.doing, Doing::Stumble { .. }),
        "it took the knock-up without going down: {:?}",
        reeling.doing
    );
}

#[test]
fn enough_ridge_damage_puts_it_on_the_ground() {
    let mut beast = Monster::new();
    let tough = sim::tuning::flinch_threshold();
    for _ in 0..40 {
        beast.doing = Doing::Prowl;
        beast.take_hit(monster::RIDGE, tough);
        if matches!(beast.doing, Doing::Toppled { .. }) {
            return;
        }
    }
    panic!("forty ridge hits and it never lost its footing");
}

#[test]
fn armour_means_the_barrel_is_not_a_shortcut_to_the_ridge() {
    let mut by_ridge = Monster::new();
    let mut by_barrel = Monster::new();
    for _ in 0..10 {
        by_ridge.doing = Doing::Prowl;
        by_barrel.doing = Doing::Prowl;
        by_ridge.take_hit(monster::RIDGE, 100);
        by_barrel.take_hit(monster::BARREL, 100);
    }
    assert!(
        by_ridge.health < by_barrel.health,
        "ten hits on the soft strip did no more than ten on the armour"
    );
}

// ---------------------------------------------------------------------------
// The exchange
// ---------------------------------------------------------------------------

#[test]
fn the_creature_can_actually_hurt_a_fighter() {
    let mut w = hunt();
    let before = w.players[0].health;
    for _ in 0..1800 {
        w.advance([Input::default(); MAX_PLAYERS]);
        if w.players[0].health < before {
            return;
        }
    }
    panic!("thirty seconds in front of it and nothing landed");
}

#[test]
fn hunters_cannot_hurt_each_other_while_there_is_something_else_to_fight() {
    let mut w = hunt();
    w.players[1].pos = w.players[0].pos.add(V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO));
    let before = w.players[1].health;
    for _ in 0..60 {
        w.advance([Input::new(Input::LEFT), Input::default()]);
    }
    assert_eq!(
        w.players[1].health, before,
        "friendly fire landed during a hunt"
    );
}

#[test]
fn the_hunt_ends_when_the_creature_does() {
    let mut w = hunt();
    w.monster.as_mut().unwrap().health = 1;
    w.players[0].pos = w.monster.unwrap().pos;
    for _ in 0..240 {
        w.advance([Input::new(Input::LEFT), Input::default()]);
        if let Phase::RoundOver { winner, .. } = w.phase {
            assert_ne!(winner, sim::state::QUARRY, "it won while on its last point");
            return;
        }
    }
    panic!("killed it and the round never ended");
}

#[test]
fn a_dead_creature_does_not_keep_anyone_standing_on_it() {
    let mut w = hunt();
    board(&mut w, monster::BARREL);
    w.monster.as_mut().unwrap().health = 0;
    run(&mut w, 1, Input::default());
    assert!(!w.players[0].aboard(), "still riding a corpse");
}

#[test]
fn nothing_it_throws_can_reach_its_own_back() {
    // **The rule the whole ride phase rests on.** Every attack the creature has
    // is aimed at the floor around it, and its back is four and a half metres
    // up, so a rider is threatened by the *buck* and by nothing else. That is
    // what makes riding a distinct phase with its own answers -- brace, leave,
    // or jump it -- rather than the ground game played at a different altitude.
    //
    // It is also why the bucks have to carry the real cost, and they do: see
    // `the_slam_beats_even_a_brace` and the shake's pair.
    let mut w = hunt();
    for part in [monster::BARREL, monster::SHOULDERS, monster::HAUNCH] {
        board(&mut w, part);
        let rider = w.players[0].pos;
        let mut beast = w.monster.expect("a hunt has a creature");
        for kind in 0..monster::MOVES as u8 {
            if monster::attack(kind).damage == 0 {
                continue;
            }
            beast.doing = Doing::Active { kind, left: 1 };
            assert!(
                !beast.reaches(
                    rider,
                    sim::tuning::body_height(),
                    sim::tuning::body_radius()
                ),
                "{} reaches somebody standing on the {}",
                monster::MOVE_NAMES[kind as usize],
                monster::PART_NAMES[part]
            );
        }
    }
}

/// A body standing at a point in the creature's body space -- `+x` toward the
/// head -- and whether the move it has out reaches them.
fn reaches_from(kind: u8, mirror: bool, x: Fx, z: Fx) -> bool {
    let mut beast = Monster::new();
    beast.doing = Doing::Active { kind, left: 1 };
    beast.brain.mirror = mirror;
    let at = beast.rig().to_world(V3::new(x, Fx::ZERO, z));
    beast.reaches(at, sim::tuning::body_height(), sim::tuning::body_radius())
}

#[test]
fn nowhere_behind_it_is_safe_to_stand() {
    // The tail root was. Every forward move needs you in front of it, and the
    // sweep is a whip about the hips, so the patch directly astern was outside
    // every volume the creature had -- a player found it and named it. Two
    // things cover it now: the kick, aimed there, and a sweep whose volume
    // rides the tail's motion from rest rather than sitting four metres
    // behind it. Both, because each has a different answer.
    let root = monster::shape(monster::TAIL_BASE);
    let astern = sim::beast::rest(sim::beast::ROOT)
        .x
        .add(root.min.x)
        .sub(sim::tuning::body_radius());
    for z in [Fx::ZERO, Fx::ONE, Fx::ONE.neg()] {
        assert!(
            reaches_from(monster::KICK, false, astern, z),
            "the kick does not reach somebody standing {z:?} m off the tail root"
        );
        let mirror = z.raw() < 0;
        assert!(
            reaches_from(monster::SWEEP, mirror, astern, z),
            "the sweep does not reach somebody standing {z:?} m off the tail root"
        );
    }
}

#[test]
fn the_sweep_goes_to_the_side_you_are_on() {
    // The clip is baked going one way. Unmirrored, the whip is on the
    // creature's right and a fighter on its left flank is outside it; the
    // creature picks the side when it commits, and plays the clip that way
    // round. Without this the left flank was the side the tail never came to.
    let flank = |z: Fx| -> V3 {
        let hip = monster::shape(monster::HINDFOOT_L);
        let x = sim::beast::rest(sim::beast::ROOT).x.add(hip.min.x);
        V3::new(x, Fx::ZERO, z)
    };
    let left = flank(Fx::from_int(-3));
    let right = flank(Fx::from_int(3));
    assert!(reaches_from(monster::SWEEP, false, right.x, right.z));
    assert!(!reaches_from(monster::SWEEP, false, left.x, left.z));
    assert!(reaches_from(monster::SWEEP, true, left.x, left.z));
    assert!(!reaches_from(monster::SWEEP, true, right.x, right.z));

    // And it is the target's side that decides, at the moment of choosing.
    for (z, expect) in [(Fx::from_int(-3), true), (Fx::from_int(3), false)] {
        let mut beast = Monster::new();
        beast.brain.think_left = 0;
        beast.brain.seen = beast.rig().to_world(flank(z));
        // Only the sweep can score from there: it wants a target behind.
        for kind in 0..monster::MOVES as u8 {
            if kind != monster::SWEEP {
                beast.brain.cooldown[kind as usize] = u16::MAX;
            }
        }
        let behind = Quarry {
            pos: beast.brain.seen,
            vel: V3::ZERO,
            alive: true,
            aboard: false,
        };
        beast.step(&[behind]);
        assert_eq!(
            beast.doing.attacking(),
            Some(monster::SWEEP),
            "it did not sweep at somebody on its flank"
        );
        assert_eq!(
            beast.brain.mirror, expect,
            "the sweep went to the wrong side for a target at z = {z:?}"
        );
    }
}

#[test]
fn it_can_still_reach_somebody_standing_where_it_is_looking() {
    // The companion, so the rule above reads as a fact about height rather than
    // as riders being quietly immune to something.
    let mut w = hunt();
    // **At the bite's own ideal range**, in front of it -- the distance the
    // control algorithm throws it at. This fixture used to stand the fighter
    // at the volume's anchor instead, with a note explaining why the anchor
    // was "further out than `hit_x` says": it was nine to thirteen metres out
    // on a move thrown at six, because the head bone's whole rest offset was
    // being added to an anchor already authored in body space, and the bite
    // whiffed at its ideal range on every throw. A test that measures where
    // the volume is cannot say whether it is where it should be.
    let beast = w.monster.expect("a hunt has a creature");
    let ideal = monster::attack(monster::BITE).ideal_range;
    w.players[0].pos = beast.rig().to_world(V3::new(ideal, Fx::ZERO, Fx::ZERO));
    w.players[0].grounded = true;
    let before = w.players[0].health;
    w.monster.as_mut().expect("a hunt has a creature").doing = Doing::Startup {
        kind: monster::BITE,
        left: monster::attack(monster::BITE).startup,
    };
    for _ in 0..monster::attack(monster::BITE).total() {
        w.monster
            .as_mut()
            .expect("a hunt has a creature")
            .brain
            .think_left = u16::MAX;
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    assert!(
        w.players[0].health < before,
        "a bite thrown at somebody standing at its ideal range missed them"
    );
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn a_hunt_is_reproducible() {
    // The creature chooses, so it has a generator -- and a generator is the
    // easiest thing in a simulation to make non-deterministic by accident.
    let checksum = |seed: u32| {
        let mut w = hunt();
        w.monster.as_mut().unwrap().brain.rng = seed;
        for i in 0..900u32 {
            let bits = if i % 37 < 6 { Input::LEFT } else { Input::W };
            w.advance([Input::aimed(bits, (i * 700) as u16), Input::default()]);
        }
        w.checksum()
    };
    assert_eq!(
        checksum(1),
        checksum(1),
        "the same hunt diverged from itself"
    );
    assert_ne!(
        checksum(1),
        checksum(2),
        "the generator seed changed nothing, so it is not being used"
    );
}

#[test]
fn re_simulating_a_hunt_from_a_snapshot_lands_in_the_same_place() {
    // What rollback does, in miniature.
    let mut w = hunt();
    let inputs = |i: u32| {
        [
            Input::aimed(
                if i % 11 < 3 { Input::LEFT } else { Input::W },
                (i * 411) as u16,
            ),
            Input::default(),
        ]
    };
    for i in 0..300 {
        w.advance(inputs(i));
    }
    let saved = w.clone();
    for i in 300..340 {
        w.advance(inputs(i));
    }
    let truth = w.checksum();
    let mut replay = saved;
    for i in 300..340 {
        replay.advance(inputs(i));
    }
    assert_eq!(replay.checksum(), truth, "re-simulation did not converge");
}

// ---------------------------------------------------------------------------
// What a hunter leaves behind
// ---------------------------------------------------------------------------

/// Hold the creature still and put it right in front of the first hunter, so a
/// test about a hazard is about the hazard rather than about chasing.
fn parked() -> World {
    let mut w = World::hunt([Class::BloodMage, Class::BloodMage]);
    let mut beast = w.monster.expect("a hunt has a creature");
    beast.pos = V3::new(w.players[0].pos.x.add(Fx::from_int(6)), Fx::ZERO, Fx::ZERO);
    beast.yaw = Fx::from_raw(1 << 15);
    beast.doing = Doing::Prowl;
    beast.brain.think_left = u16::MAX;
    w.monster = Some(beast);
    w.players[0].pos = V3::new(w.players[0].pos.x, Fx::ZERO, Fx::ZERO);
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::ZERO);
    w
}

fn beast_health(w: &World) -> i32 {
    w.monster.expect("a hunt has a creature").health
}

#[test]
fn a_drain_field_hurts_the_creature() {
    // It did not, for a long time, and the bug was invisible because it only
    // showed up in a hunt: effects were applied to fighters and nobody else, so
    // a Blood mage hunting alone put a spike in the ground, drained an empty
    // patch of arena and got nothing back. Half a kit doing nothing in one of
    // the game's two modes.
    let mut w = parked();
    let full = beast_health(&w);
    for _ in 0..2 {
        w.advance([Input::new(Input::MECHANIC), Input::default()]);
    }
    for _ in 0..200 {
        w.advance([Input::default(), Input::default()]);
    }
    assert!(
        beast_health(&w) < full,
        "the field never touched the creature"
    );
}

#[test]
fn a_drain_field_feeds_the_hunter_who_laid_it() {
    // The other half. The Blood mage pays health to cast, so if the return only
    // worked in versus the class would be unplayable in coop by its own
    // numbers.
    //
    // **It has to start measuring after the eruption**, and the first version
    // of this test did not. The spike hits once on the frame it arrives and
    // that hit pays out correctly; the *field* that stands there afterwards was
    // taking health off the creature and returning none of it, for weeks,
    // behind an assertion that only asked whether the caster's bar went up at
    // all. It went up -- a few frames earlier, from the other thing. What is
    // measured here is a window containing nothing but field ticks.
    let mut w = parked();
    let spike = sim::moves::get(Class::BloodMage, sim::state::SLOT_MECHANIC);
    for _ in 0..2 {
        w.advance([Input::new(Input::MECHANIC), Input::default()]);
    }
    for _ in 0..spike.startup + spike.active + 4 {
        w.advance([Input::default(), Input::default()]);
    }
    assert!(
        w.effects.iter().flatten().next().is_some(),
        "fixture: the eruption is not over, or it left no field"
    );

    // Hurt, so there is room on the bar for the return to show.
    w.players[0].health = sim::tuning::max_health() / 2;
    let paid = w.players[0].health;
    let beast = beast_health(&w);
    for _ in 0..120 {
        w.advance([Input::default(), Input::default()]);
    }
    let drained = beast - beast_health(&w);
    assert!(drained > 0, "fixture: the field drained nothing");
    assert!(
        w.players[0].health > paid,
        "the field drained {drained} off the creature and gave the caster none of it"
    );
}

#[test]
fn a_topple_is_a_disable_and_a_flinch_is_not() {
    // Drawn on the same line as the fighters' own list. A topple is the long
    // window the whole climb exists to earn and the one state the creature
    // cannot act out of; a flinch is the cheap one that happens whenever it is
    // hit hard enough, and it is excluded for exactly the reason hitstun is.
    let state = |doing: Doing| {
        let mut w = parked();
        let mut beast = w.monster.expect("a hunt has a creature");
        beast.doing = doing;
        w.monster = Some(beast);
        w.monster.expect("a hunt has a creature").disabled()
    };
    assert!(!state(Doing::Prowl), "a prowling creature is disabled");
    assert!(
        !state(Doing::Flinch { left: 10 }),
        "a flinch counts, so the bonus fires on every heavy hit"
    );
    assert!(
        state(Doing::Toppled { left: 200 }),
        "a topple does not count, so the whole climb pays nothing extra"
    );
}

#[test]
fn a_blood_mage_hits_a_toppled_creature_harder() {
    // The trait has to mean something in a hunt, or it is a versus-only feature
    // on a class that has just had its versus-only bugs fixed.
    //
    // Disabled means on its side *or on a knee* (`Monster::disabled`), and the
    // knee is the state this fixture uses: held on the first frame of a
    // stumble the creature is still standing in exactly the pose it stands in,
    // so the same throw from the same spot lands on the same foot, and the only
    // thing that differs between the two runs is the rule under test. A
    // toppled creature lies lower and pitched, and every version of this
    // fixture that used it was really a search for a spot where two different
    // poses happened to agree -- which the animal's geometry stopped offering
    // the day its legs grew.
    //
    // The Bloodletter, because it is her auto and the one attack in her kit
    // with a button: shift stopped modifying clicks on 2026-09-16, and this
    // fixture used to name the claw and press the poke's input.
    let blade = sim::moves::get(Class::BloodMage, sim::state::SLOT_POKE);
    let up = Doing::Prowl;
    let knee = Doing::Stumble {
        left: sim::tuning::stumble_frames(),
        front: true,
    };

    // Throw from behind it, at a hind foot; report the damage and which foot.
    let throw = |doing: Doing, stand_at: Fx| -> (i32, Option<usize>) {
        let mut w = parked();
        let feet_before = w.monster.expect("a hunt has a creature").part_health;
        let before = beast_health(&w);
        let flight = sim::tuning::bloodletter_flight();
        for f in 0..(blade.startup + blade.active + flight + 4) {
            // Held down, and held still: the creature would otherwise stand up,
            // walk off, or decide to bite. Held on the *first* frame of the
            // stumble, so its pose is the standing one.
            let mut beast = w.monster.expect("a hunt has a creature");
            beast.doing = doing;
            beast.pos = V3::ZERO;
            beast.yaw = Fx::from_raw(1 << 15);
            beast.speed = Fx::ZERO;
            beast.brain.think_left = u16::MAX;
            w.monster = Some(beast);
            // In line with the left hind leg rather than the spine: a blade
            // thrown down the centreline passes between the legs and grazes
            // whichever one the animal's breathing happens to sway into it.
            w.players[0].pos = V3::new(stand_at, Fx::ZERO, Fx::ratio(9, 10));
            let held = if f < 2 { Input::LEFT } else { 0 };
            // Facing the creature's tail, which is toward -x: it stands at the
            // origin pointed the other way.
            w.advance([Input::aimed(held, 1 << 15), Input::default()]);
        }
        let feet_after = w.monster.expect("a hunt has a creature").part_health;
        let foot = monster::BREAKABLE
            .iter()
            .copied()
            .find(|p| feet_after[*p] < feet_before[*p]);
        (before - beast_health(&w), foot)
    };

    let mut found = None;
    for tenth in 30..100 {
        let x = Fx::ratio(tenth, 10);
        let (standing, a) = throw(up, x);
        let (kneeling, b) = throw(knee, x);
        if standing > 0 && a.is_some() && a == b {
            found = Some((standing, kneeling));
            break;
        }
    }
    let (standing, kneeling) =
        found.expect("nowhere the blade reaches the same hind foot whether it is up or down");
    assert!(
        kneeling > standing,
        "a creature on a knee took {kneeling} where a standing one took {standing}"
    );
}

#[test]
fn a_hazard_never_touches_a_hunting_partner() {
    // Friendly fire is off in a hunt, and the condition is the creature being
    // there rather than a flag -- the same rule direct hits already follow. A
    // drain field was the one thing in the game that could kill a team-mate.
    let mut w = parked();
    for _ in 0..2 {
        w.advance([Input::new(Input::MECHANIC), Input::default()]);
    }
    // Stand the partner in it, wherever it landed, and hold them there.
    let mut stood_in_it = 0;
    for _ in 0..200 {
        if let Some(field) = w.effects.iter().flatten().next().copied() {
            w.players[1].pos = V3::new(field.pos.x, w.players[1].pos.y, field.pos.z);
            stood_in_it += 1;
        }
        w.advance([Input::default(), Input::default()]);
    }
    assert!(stood_in_it > 60, "fixture: nobody stood in anything");
    assert_eq!(
        w.players[1].health,
        w.players[1].full_health(),
        "a hunter's own hazard hurt their partner"
    );
}

// ---------------------------------------------------------------------------
// The buck, part by part
//
// The shape of the ride phase, stated as relationships rather than numbers.
// Every one of these is a decision, and every one of them moved while the
// creature was being rebuilt -- see `docs/design/feel-log.md`.
// ---------------------------------------------------------------------------

/// Where the rider ends up, boarding at a spot and sitting through a move.
fn rides_out(part: usize, kind: u8, brace: bool) -> bool {
    let mut w = hunt();
    board(&mut w, part);
    // The haunch's middle is inside the barrel's footprint -- consecutive
    // treads overlap so the climb has no seams -- and a body standing in an
    // overlap is standing on the higher of the two, which is correct and is not
    // the haunch. Standing on the haunch means standing on the back of it.
    if part == monster::HAUNCH {
        w.players[0].local.x = w.players[0].local.x.sub(Fx::ratio(6, 10));
    }
    let held = if brace {
        Input::new(Input::CROUCH)
    } else {
        Input::default()
    };
    !thrown_during(&mut w, kind, held)
}

#[test]
fn the_shake_throws_a_loose_rider_off_the_back_and_a_braced_one_holds() {
    // The move's whole job, on the two parts the climb is *for*: the ridge lies
    // along the barrel and the nape is off the shoulders, so those are where a
    // rider who is getting paid is standing.
    for part in [monster::BARREL, monster::SHOULDERS] {
        assert!(
            !rides_out(part, monster::SHAKE, false),
            "a shake did nothing to someone standing loose on the {}",
            monster::PART_NAMES[part]
        );
        assert!(
            rides_out(part, monster::SHAKE, true),
            "bracing on the {} did not hold through a shake, so crouch is not \
             an answer where it needs to be one",
            monster::PART_NAMES[part]
        );
    }
}

#[test]
fn the_hips_and_the_tail_root_are_the_calm_places_to_stand() {
    // A gradient rather than a flag, and it is what makes *where* you stand on
    // the animal a decision. Both are near an axis the shake turns about, and
    // both are a long walk from anything worth hitting -- which is the trade.
    for part in [monster::HAUNCH, monster::TAIL_BASE] {
        assert!(
            rides_out(part, monster::SHAKE, false),
            "the {} is as violent as the back, so retreating to it buys nothing",
            monster::PART_NAMES[part]
        );
    }
    // Out along the tail is a different matter: that is where a sweep has a
    // lever on you.
    assert!(
        !rides_out(monster::TAIL_MID, monster::SWEEP, false),
        "a tail sweep did not throw somebody standing out along the tail"
    );
}

#[test]
fn the_moves_that_are_aimed_at_the_ground_do_not_throw_a_braced_rider() {
    // Otherwise bracing is not a decision, it is a thing you hold down. The
    // slam is the exception on purpose and has its own test.
    for kind in [
        monster::BITE,
        monster::STOMP,
        monster::CHARGE,
        monster::SWEEP,
        monster::KICK,
    ] {
        for part in [monster::BARREL, monster::SHOULDERS, monster::HAUNCH] {
            assert!(
                rides_out(part, kind, true),
                "{} threw a braced rider off the {}",
                monster::MOVE_NAMES[kind as usize],
                monster::PART_NAMES[part]
            );
        }
    }
}

#[test]
fn you_can_jump_the_shake_if_you_commit_before_the_whip() {
    // **The strategy the ride phase is built around**: bait the shake by being
    // up there, read its startup, leave the ground over it, land back on, and
    // spend the window it cannot shake again in.
    //
    // What makes it a read rather than a reflex is that the answer is *early*.
    // A jump lasts about as long as the whip does, so committing during the
    // startup clears the whole thing, and committing once it has begun is a
    // jump that never leaves. See `docs/design/monsters.md` §3.
    let startup = monster::attack(monster::SHAKE).startup;
    let window = |lead: u16| -> u32 {
        let mut w = hunt();
        board(&mut w, monster::BARREL);
        let total = monster::attack(monster::SHAKE).total();
        w.monster.as_mut().expect("a hunt has a creature").doing = Doing::Startup {
            kind: monster::SHAKE,
            left: startup,
        };
        let mut run = 0;
        let mut best = 0;
        let mut left_at_all = false;
        for f in 0..total + 120 {
            let held = if f >= lead && f < lead + 30 {
                Input::new(Input::SPACE)
            } else {
                Input::default()
            };
            w.advance([held, Input::default()]);
            if !w.players[0].aboard() {
                left_at_all = true;
                run = 0;
            } else if left_at_all {
                run += 1;
                best = u32::max(best, run);
            }
        }
        best
    };

    // In the last part of the startup: back on, and back on for long enough
    // to throw something committed at the ridge. Not anywhere in it -- a hop
    // taken at the first frame of a forty-frame windup is back on the animal
    // fifteen frames into the whip, and that is a throw. The read is *late
    // in the tell*, which is the version of "read the startup" that is
    // actually a read; the earlier claim that any frame of it would do was
    // true only because a thrown rider used to land straight back on the
    // animal and stay, at the fall's cost. See `docs/design/monsters.md` §3.
    for lead in [
        startup * 5 / 8,
        startup * 3 / 4,
        startup * 7 / 8,
        startup - 2,
    ] {
        let got = window(lead);
        assert!(
            got > 50,
            "jumping at frame {lead} of a {startup}-frame windup earned only \
             {got} frames back on the animal"
        );
    }
    // Once the whip is running, a jump is a jump that never left. Averaged
    // over a span rather than sampled at one frame: the whip reverses nine
    // times and a late jump that happens to land between two reversals gets
    // away with it, which is a player getting lucky rather than a player
    // getting it right. What has to hold is that it is no longer a plan.
    let mean = |range: std::ops::Range<u16>| -> u32 {
        let n = range.len() as u32;
        range.map(window).sum::<u32>() / n.max(1)
    };
    let early = mean(startup * 5 / 8..startup);
    let late = mean(startup..startup + 12);
    // **Two bars, and the absolute one is the real one.** A late jump must not
    // earn the fifty frames the first half of this test calls long enough to be
    // worth having; that is what "no longer a plan" means, and it is stated
    // against this test's own definition rather than against the early jump.
    //
    // The ratio was four to one until the mobility pass of 2026-09-17, which
    // cut the takeoff across the cast: a full hop is now a little *shorter*
    // than the whip rather than a little longer, so an early commit clears
    // slightly less of it and the gap narrowed to 3.7 without anything about
    // the read changing. A margin that moves every time somebody tunes the jump
    // is measuring the jump rather than the timing. See
    // `docs/design/feel-log.md`.
    assert!(
        late < 50,
        "jumping once the whip had started still earned {late} frames back on \
         the animal -- long enough to throw something committed at the ridge, \
         so reading the startup buys nothing"
    );
    assert!(
        late * 3 < early,
        "jumping once the whip had started still earned {late} frames back on \
         the animal on average, against {early} for jumping before it -- so \
         there is no timing in it"
    );
}

#[test]
fn a_move_it_has_just_thrown_cannot_come_straight_back() {
    // What makes baiting one worth doing. Without a lockout the answer to every
    // buck is another buck, and a rider who read the shake and jumped it has
    // earned nothing at all.
    let mut beast = Monster::new();
    beast.brain.think_left = 0;
    let aboard = [Quarry {
        pos: beast.pos,
        vel: V3::ZERO,
        alive: true,
        aboard: true,
    }];
    // Let it pick something; with a rider aboard and nothing on the ground, the
    // shake is what it wants.
    for _ in 0..4 {
        beast.step(&aboard);
    }
    let shake = monster::attack(monster::SHAKE);
    assert!(
        shake.cooldown > shake.total(),
        "the shake's lockout is shorter than the shake, so it can chain"
    );
    beast.doing = Doing::Startup {
        kind: monster::SHAKE,
        left: shake.startup,
    };
    beast.brain.cooldown[monster::SHAKE as usize] = shake.cooldown;
    assert_eq!(
        beast.appetite(monster::SHAKE, 1),
        0,
        "a move on its lockout still scored"
    );
}

// ---------------------------------------------------------------------------
// The rig
// ---------------------------------------------------------------------------

/// The lowest point any part of the creature reaches, through a whole clip.
fn lowest(clip: sim::beast::Clip) -> (Fx, usize) {
    let (_, count) = sim::beast_baked::SPAN[clip.index()];
    let mut worst = (Fx::MAX, 0);
    for i in 0..count.max(1) {
        let at = Fx::from_int(i as i32).div(Fx::from_int((count.max(2) - 1) as i32));
        let rig = sim::beast::Rig::build(V3::ZERO, Fx::ZERO, &sim::beast::sample(clip, at));
        for part in 0..monster::PARTS {
            let sh = monster::shape(part);
            for x in [sh.min.x, sh.max.x] {
                for y in [sh.min.y, sh.max.y] {
                    for z in [sh.min.z, sh.max.z] {
                        let low = rig.part_to_world(part, V3::new(x, y, z)).y;
                        if low.raw() < worst.0.raw() {
                            worst = (low, part);
                        }
                    }
                }
            }
        }
    }
    worst
}

#[test]
fn the_creature_keeps_itself_roughly_out_of_the_floor() {
    // A blockout made of boxes will always clip a corner: springs overshoot on
    // purpose, a foot is a box rather than a sole, and a creature on its side
    // has legs pointing at things. This is not a polish bar -- it is the bar
    // that catches a **broken** pose, which is what the first version of every
    // one of these clips had. A foreleg half a metre under is a wireframe
    // artefact; one a metre and a half under is an animal standing in the
    // ground, and the slam, the stumble and the rear all did exactly that.
    //
    // `cargo run -p anim --bin preview_beast` draws it, which is how you tell
    // the two apart.
    let floor = Fx::from_int(1).neg();
    for clip in sim::beast::Clip::ALL {
        let (low, part) = lowest(clip);
        assert!(
            low.raw() > floor.raw(),
            "the {} clip puts the {} {:?} m into the floor",
            clip.name(),
            monster::PART_NAMES[part],
            low
        );
    }
}

#[test]
fn a_creature_on_broken_legs_is_lower_but_still_standing_on_them() {
    // Both halves matter. The drop is what brings the back within a jump, and
    // it has to be a *lean* rather than the whole animal sinking -- the hips
    // take the average of the two ends and the pitch says which end went, so
    // three broken feet cannot add up to a creature buried to its knees.
    let mut lame = Monster::new();
    for leg in sim::beast::LEGS {
        lame.part_health[leg.foot] = 0;
    }
    let rig = lame.rig();
    let mut low = Fx::MAX;
    for part in 0..monster::PARTS {
        let sh = monster::shape(part);
        low = low.min(rig.part_to_world(part, sh.min).y);
    }
    assert!(
        low.raw() > Fx::from_int(1).neg().raw(),
        "an animal with every foot broken is {low:?} m into the floor"
    );

    let sound = Monster::new();
    let top = |m: &Monster| {
        let sh = monster::shape(monster::SHOULDERS);
        m.rig()
            .part_to_world(monster::SHOULDERS, V3::new(sh.min.x, sh.max.y, Fx::ZERO))
            .y
    };
    assert!(
        top(&lame).raw() < top(&sound).raw(),
        "breaking every leg it has did not lower its back at all"
    );
}

#[test]
fn every_bone_in_the_rig_reaches_the_world_through_its_parents() {
    // The one property forward kinematics has to have, and the one that would
    // fail silently: a child bone must move when its parent does. A typo in
    // `PARENTS` gives a creature whose tail hangs in the air where the tail
    // used to be, and nothing else would say so.
    let mut bent = sim::beast::Pose::rest();
    bent.bone[sim::beast::ROOT] = V3::new(Fx::ratio(1, 8), Fx::ZERO, Fx::ZERO);
    let rest = sim::beast::Rig::build(V3::ZERO, Fx::ZERO, &sim::beast::Pose::rest());
    let moved = sim::beast::Rig::build(V3::ZERO, Fx::ZERO, &bent);
    for bone in 0..sim::beast::BONES {
        if bone == sim::beast::ROOT {
            continue;
        }
        let shift = moved.bone[bone].at.sub(rest.bone[bone].at).len();
        assert!(
            shift.raw() > Fx::ratio(1, 100).raw(),
            "pitching the root left {} exactly where it was",
            sim::beast::BONE_NAMES[bone]
        );
    }
}
