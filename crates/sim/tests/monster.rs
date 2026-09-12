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
    World::hunt([Class::Champion; MAX_PLAYERS])
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
    let s = beast.stance();
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
fn the_ridge_is_out_of_reach_from_the_ground() {
    // Geometry rather than a rule: the soft strip sits above a standing
    // fighter's hurtbox, so getting at it means getting off the floor.
    let ridge = monster::shape(monster::RIDGE);
    assert!(
        ridge.min.y.raw() > sim::tuning::body_height().raw(),
        "the ridge starts at {:?}, inside a standing fighter's reach",
        ridge.min.y
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
fn jumping_is_how_you_leave() {
    let mut w = hunt();
    board(&mut w, monster::BARREL);
    run(&mut w, 1, Input::new(Input::SPACE));
    assert!(!w.players[0].aboard(), "jumping did not leave the ride");
    assert!(w.players[0].vel.y.raw() > 0, "the jump had no rise in it");
}

#[test]
fn a_shake_throws_a_rider_who_is_not_holding_on() {
    let mut w = hunt();
    board(&mut w, monster::BARREL);
    // Away from the axis the shake turns about. How hard the back throws you
    // is how far round it swings you, so the middle of the spine is calm and
    // the ends are not -- which is a rule nobody wrote down and everybody can
    // feel.
    w.players[0].local.x = Fx::from_int(-2);
    assert!(
        thrown_during(&mut w, monster::SHAKE, Input::default()),
        "a shake did nothing to someone standing loose on the back"
    );
}

#[test]
fn bracing_holds_you_through_a_shake() {
    // Crouch is the third answer, alongside dodging and leaving, and it is the
    // only one available while you are up there.
    let mut w = hunt();
    board(&mut w, monster::BARREL);
    // Stand where the back is calmest: near the axis the shake turns about.
    w.players[0].local.x = Fx::ratio(-12, 10);
    w.players[0].local.z = Fx::ZERO;
    assert!(
        !thrown_during(&mut w, monster::SHAKE, Input::new(Input::CROUCH)),
        "bracing near the middle of the back did not hold through a shake"
    );
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
    // Nothing to throw at that range, so it only prowls.
    beast.step(&[behind]);
    let after_one = wrap_turns(beast.yaw).abs();
    assert!(
        after_one.raw() < Fx::ratio(1, 50).raw(),
        "it snapped half a turn in one frame"
    );
    for _ in 0..180 {
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
    // Enough to flinch it, and well short of what breaks its poise. A topple
    // *does* interrupt a live hitbox -- it is the one thing that may, and it
    // costs a whole climb to earn.
    beast.take_hit(monster::RIDGE, sim::tuning::flinch_threshold() * 3);
    assert!(
        matches!(beast.doing, Doing::Active { .. }),
        "an active frame was cancelled by a hit: {:?}",
        beast.doing
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
fn being_hit_takes_you_off_the_back() {
    let mut w = hunt();
    board(&mut w, monster::BARREL);
    // The sweep reaches the haunch, which is the point of it reaching that
    // high: where you stand on the back decides whether the tail can find you.
    w.players[0].local.x = monster::shape(monster::BARREL).min.x.add(Fx::ratio(1, 10));
    let before = w.players[0].health;
    let came_off = thrown_during(&mut w, monster::SWEEP, Input::default());
    assert!(
        came_off && w.players[0].health < before,
        "a tail sweep across the haunch left the rider standing there"
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
    let mut w = parked();
    for _ in 0..2 {
        w.advance([Input::new(Input::MECHANIC), Input::default()]);
    }
    // Hurt, after the cast, so there is room on the bar for the return to show.
    w.players[0].health = sim::tuning::max_health() / 2;
    let paid = w.players[0].health;
    let beast = beast_health(&w);
    for _ in 0..200 {
        w.advance([Input::default(), Input::default()]);
    }
    assert!(beast_health(&w) < beast, "fixture: nothing was drained");
    assert!(
        w.players[0].health > paid,
        "the field drained the creature and gave the caster none of it"
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
        sim::tuning::max_health(),
        "a hunter's own hazard hurt their partner"
    );
}
