//! The Bulwark's weight: every blow taken on the shield is stored in it.
//!
//! These encode `docs/design/bulwark-v2.md`, so a failure is either a bug or a
//! decision that changed without the proposal changing with it. The numbers
//! behind them are printed by `cargo run -p sim --bin weight`.

use sim::bulwark;
use sim::class::{Class, Mechanic, Shield};
use sim::fixed::Fx;
use sim::math::atan2_turns;
use sim::monster::{self, Doing};
use sim::state::{Action, MAX_PLAYERS};
use sim::tuning as t;
use sim::{Input, V3, World};

const L: u16 = Input::LEFT;
const R: u16 = Input::RIGHT;
const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

fn run(w: &mut World, frames: u32, a: u16, b: u16) {
    for _ in 0..frames {
        w.advance([Input::aimed(a, LOOK_RIGHT), Input::aimed(b, LOOK_LEFT)]);
    }
}

/// A Champion in sword range of a Bulwark, who is on the right -- in the
/// middle of the arena, so a shove has room to be a shove rather than a wall.
fn champion_on_bulwark() -> World {
    let mut w = World::with_classes([Class::Champion, Class::Bulwark]);
    w.players[0].pos.x = Fx::from_int(-2);
    w.players[1].pos.x = Fx::ratio(-1, 2);
    w
}

fn weight(w: &World, i: usize) -> Fx {
    bulwark::weight(&w.players[i])
}

fn set_weight(w: &mut World, i: usize, weight: Fx) {
    let Mechanic::Shield(s) = w.players[i].mechanic else {
        panic!("player {i} carries no shield");
    };
    w.players[i].mechanic = Mechanic::Shield(s.with_weight(weight));
}

/// One frame of drain, in the same arithmetic the simulation uses.
fn one_frame_of_drain() -> Fx {
    t::weight_cap().div(Fx::from_int(t::weight_drain_frames()))
}

/// The Champion's opening sword swing into the Bulwark, who starts guarding
/// `guard_from` frames before it (negative: after). Returns the most weight
/// the shield held, the damage taken, and whether the guard was a parry.
fn swing_into_guard(guard_from: i32) -> (Fx, i32, bool) {
    let mut w = champion_on_bulwark();
    let before = w.players[1].health;
    let mut heaviest = Fx::ZERO;
    let mut parried = false;
    for f in -40..40 {
        // One press, so it is one swing: held, the sword strings on.
        let a = if f == 0 { L } else { 0 };
        let b = if f >= -guard_from { R } else { 0 };
        w.advance([Input::aimed(a, LOOK_RIGHT), Input::aimed(b, LOOK_LEFT)]);
        heaviest = heaviest.max(weight(&w, 1));
        parried |= w.players[1].parried > 0;
    }
    (heaviest, before - w.players[1].health, parried)
}

/// What the same swing deals to a Bulwark who does not guard.
fn unguarded_damage() -> i32 {
    let (_, dealt, _) = swing_into_guard(-1000);
    assert!(dealt > 0, "the fixture's swing does not connect");
    dealt
}

#[test]
fn a_blocked_hit_loads_the_shield_with_its_damage() {
    let damage = unguarded_damage();
    let (held, taken, parried) = swing_into_guard(30);
    assert_eq!(taken, 0, "the guard took damage");
    assert!(!parried, "a guard raised thirty frames early parried");
    let want = Fx::from_int(damage);
    let slack = one_frame_of_drain().mul(Fx::from_int(2));
    assert!(
        held.raw() <= want.raw() && held.raw() >= want.sub(slack).raw(),
        "blocking a {damage}-damage blow stored {held:?}"
    );
}

#[test]
fn a_parried_hit_loads_it_more() {
    let damage = unguarded_damage();
    let parry = (-10..30)
        .map(swing_into_guard)
        .find(|(_, _, parried)| *parried)
        .expect("no guard timing in forty frames parried the swing");
    let want = Fx::from_int(damage)
        .mul(t::parry_load())
        .min(t::weight_cap());
    let slack = one_frame_of_drain().mul(Fx::from_int(2));
    assert!(
        parry.0.raw() <= want.raw() && parry.0.raw() >= want.sub(slack).raw(),
        "parrying a {damage}-damage blow stored {:?}, wanted {want:?}",
        parry.0
    );
    let (blocked, _, _) = swing_into_guard(30);
    assert!(
        parry.0.raw() > blocked.raw(),
        "a parry stored no more than a block"
    );
}

#[test]
fn weight_never_passes_the_cap() {
    let mut p = sim::state::Player::new(Class::Bulwark);
    for _ in 0..20 {
        bulwark::load(&mut p, 500, true);
    }
    assert_eq!(bulwark::weight(&p), t::weight_cap());
}

#[test]
fn only_a_shield_holds_weight() {
    let mut p = sim::state::Player::new(Class::Champion);
    bulwark::load(&mut p, 500, false);
    assert_eq!(bulwark::weight(&p), Fx::ZERO);
}

#[test]
fn weight_drains_to_nothing_on_its_clock() {
    let mut w = World::new();
    set_weight(&mut w, 0, t::weight_cap());
    let clock = t::weight_drain_frames() as u32;
    run(&mut w, clock / 2, 0, 0);
    let half = weight(&w, 0);
    let expected = t::weight_cap().div(Fx::from_int(2));
    assert!(
        half.sub(expected).abs().raw() <= one_frame_of_drain().mul(Fx::from_int(2)).raw(),
        "half the clock left {half:?} of a full {:?}",
        t::weight_cap()
    );
    // A frame of rounding either way.
    run(&mut w, clock / 2 + 2, 0, 0);
    assert_eq!(
        weight(&w, 0),
        Fx::ZERO,
        "the clock ran out and weight did not"
    );
}

#[test]
fn weight_travels_with_the_thrown_shield() {
    let mut w = World::new();
    let load = Fx::from_int(200);
    set_weight(&mut w, 0, load);
    run(&mut w, 1, Input::MECHANIC, 0);
    let Some(Shield::Flying { weight, .. }) = w.players[0].shield() else {
        panic!("the shield was not thrown");
    };
    assert!(
        weight.raw() > 0 && weight.raw() <= load.raw(),
        "a thrown shield carried {weight:?} of {load:?}"
    );
}

#[test]
fn a_heavy_shield_is_pushed_back_less() {
    let shove = |loaded: bool| {
        let mut w = champion_on_bulwark();
        run(&mut w, 30, 0, R);
        if loaded {
            set_weight(&mut w, 1, t::weight_cap());
        }
        let from = w.players[1].pos;
        run(&mut w, 1, L, R);
        run(&mut w, 30, 0, R);
        assert_eq!(
            w.players[1].health,
            w.players[1].max_health(),
            "the fixture's guard did not hold"
        );
        w.players[1].pos.sub(from).flat_len()
    };
    let empty = shove(false);
    let heavy = shove(true);
    assert!(
        empty.raw() > 0,
        "a blocked swing did not push the guard at all"
    );
    assert!(
        heavy.raw() < empty.raw(),
        "a full shield was pushed {heavy:?}, an empty one {empty:?}"
    );
}

#[test]
fn the_bulwark_has_the_most_health_on_the_roster() {
    let bulwark = t::class_health(Class::Bulwark);
    for class in sim::class::ALL_CLASSES {
        if class != Class::Bulwark {
            assert!(
                t::class_health(class) < bulwark,
                "{class:?} has {} health to the Bulwark's {bulwark}",
                t::class_health(class)
            );
        }
    }
    let w = World::with_classes([Class::Bulwark, Class::Champion]);
    assert_eq!(w.players[0].health, bulwark, "a round starts below the bar");
}

/// A Bulwark with his guard up at the stomp's own range, facing it.
#[test]
fn a_blocked_creature_blow_loads_the_shield() {
    let mut w = World::hunt([Class::Bulwark; MAX_PLAYERS]);
    let beast = w.monster.expect("a hunt has a creature");
    let ideal = monster::attack(monster::STOMP).ideal_range;
    w.players[0].pos = beast.rig().to_world(V3::new(ideal, Fx::ZERO, Fx::ZERO));
    w.players[0].grounded = true;
    // Parked far away, so the second fighter is not a second target.
    w.players[1].pos = V3::new(Fx::from_int(40), w.players[1].pos.y, Fx::from_int(40));
    let toward = beast.pos.sub(w.players[0].pos);
    let aim = (atan2_turns(toward.z, toward.x).raw() & 0xFFFF) as u16;
    w.players[0].facing = V3::new(toward.x, Fx::ZERO, toward.z).normalized();
    w.players[0].action = Action::Guard { held: 60 };
    let before = w.players[0].health;
    w.monster.as_mut().unwrap().doing = Doing::Startup {
        kind: monster::STOMP,
        left: monster::attack(monster::STOMP).startup,
    };
    let mut heaviest = Fx::ZERO;
    for _ in 0..monster::attack(monster::STOMP).total() {
        w.monster.as_mut().unwrap().brain.think_left = u16::MAX;
        w.advance([Input::aimed(R, aim), Input::default()]);
        heaviest = heaviest.max(weight(&w, 0));
    }
    assert_eq!(
        w.players[0].health, before,
        "the stomp went through the guard"
    );
    assert!(heaviest.raw() > 0, "a blocked stomp stored nothing");
}
