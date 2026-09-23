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

// ---------------------------------------------------------------------------
// Slam: M2
// ---------------------------------------------------------------------------

const M: u16 = Input::MIDDLE;

/// Two Bulwarks, the left one holding `load`, the other a dummy in Slam's
/// reach.
fn slam_at(load: Fx) -> World {
    let mut w = World::with_classes([Class::Bulwark, Class::Bulwark]);
    w.players[0].pos.x = Fx::from_int(-2);
    w.players[1].pos.x = Fx::ratio(-1, 2);
    set_weight(&mut w, 0, load);
    w
}

/// One standing Slam from `load`, into a dummy doing `guard`. Damage dealt,
/// the widest the drawn volume was, weight after, and whether the dummy was
/// left staggered.
fn slam_from(load: Fx, dummy: u16) -> (i32, Fx, Fx, bool) {
    let mut w = slam_at(load);
    run(&mut w, 20, 0, dummy);
    let before = w.players[1].health;
    let mut radius = Fx::ZERO;
    let mut staggered = false;
    for f in 0..60 {
        run(&mut w, 1, if f == 0 { M } else { 0 }, dummy);
        if let Some(h) = sim::state::hitbox(&w.players[0]) {
            radius = radius.max(h.radius);
        }
        staggered |= matches!(w.players[1].action, Action::Stagger { .. });
    }
    (
        before - w.players[1].health,
        radius,
        weight(&w, 0),
        staggered,
    )
}

#[test]
fn middle_click_is_slam() {
    let mut w = slam_at(Fx::ZERO);
    run(&mut w, 1, M, 0);
    assert!(
        matches!(
            w.players[0].action,
            Action::Startup { kind, .. } if kind == sim::state::SLOT_COMMITTED
        ),
        "middle click threw {:?}",
        w.players[0].action
    );
}

#[test]
fn slam_hits_harder_and_shakes_wider_with_weight_and_spends_it() {
    let mut last = (0, Fx::ZERO);
    for quarter in 0..=4 {
        let load = t::weight_cap().mul(Fx::ratio(quarter, 4));
        let (damage, radius, after, _) = slam_from(load, 0);
        assert!(damage > 0, "a slam from {load:?} did not connect");
        if quarter > 0 {
            assert!(
                damage > last.0 && radius.raw() > last.1.raw(),
                "a slam from {load:?} dealt {damage} over {radius:?}, no more than {last:?}"
            );
        }
        assert_eq!(after, Fx::ZERO, "a slam from {load:?} left {after:?}");
        last = (damage, radius);
    }
}

#[test]
fn only_a_nearly_full_slam_staggers() {
    let (_, _, _, full) = slam_from(t::weight_cap(), 0);
    assert!(full, "a full slam did not stagger");
    let short = t::weight_cap()
        .mul(t::slam_stagger_share())
        .sub(Fx::from_int(20));
    let (_, _, _, partly) = slam_from(short, 0);
    assert!(!partly, "a slam from {short:?} staggered");
}

#[test]
fn a_blocked_slam_costs_the_same_frames_at_any_weight() {
    // Frames do not change with weight -- only damage and radius do -- so a
    // blocked full slam is exactly as punishable as an empty one.
    let blockstun = |load: Fx| {
        let mut w = slam_at(load);
        run(&mut w, 20, 0, R);
        let mut longest = 0;
        for f in 0..60 {
            run(&mut w, 1, if f == 0 { M } else { 0 }, R);
            if let Action::BlockStun { left } = w.players[1].action {
                longest = longest.max(left);
            }
            assert!(
                !matches!(w.players[1].action, Action::Stagger { .. }),
                "a blocked slam from {load:?} staggered through the guard"
            );
        }
        longest
    };
    let empty = blockstun(Fx::ZERO);
    assert!(empty > 0, "the fixture's guard never blocked");
    assert_eq!(blockstun(t::weight_cap()), empty);
}

#[test]
fn a_crouch_does_not_duck_the_shake() {
    let (damage, _, _, _) = slam_from(Fx::ZERO, Input::CROUCH);
    assert!(damage > 0, "a crouching dummy ducked the slam");
}

/// A Slam thrown on the way down from a full jump, the frame the fall begins.
fn slam_out_of_a_jump() -> (World, i32) {
    let mut w = slam_at(Fx::ZERO);
    w.players[1].pos.x = Fx::from_int(8);
    let mut pressed = false;
    for f in 0..90 {
        let p = w.players[0];
        let bits = if f <= 20 {
            Input::SPACE
        } else if !pressed && !p.grounded && p.vel.y.raw() < 0 {
            pressed = true;
            M
        } else {
            0
        };
        run(&mut w, 1, bits, 0);
    }
    let fell = w.players[0].pos;
    // Again, with the dummy where the shake lands.
    let mut again = slam_at(Fx::ZERO);
    again.players[1].pos.x = fell.x.add(Fx::from_int(2));
    let before = again.players[1].health;
    let mut pressed = false;
    for f in 0..90 {
        let p = again.players[0];
        let bits = if f <= 20 {
            Input::SPACE
        } else if !pressed && !p.grounded && p.vel.y.raw() < 0 {
            pressed = true;
            M
        } else {
            0
        };
        run(&mut again, 1, bits, 0);
    }
    let dealt = before - again.players[1].health;
    (again, dealt)
}

#[test]
fn a_slam_out_of_a_fall_hits_harder_and_lands_with_the_feet() {
    let (standing, _, _, _) = slam_from(Fx::ZERO, 0);
    let (_, fallen) = slam_out_of_a_jump();
    assert!(
        fallen > standing,
        "a slam out of a full jump dealt {fallen}, a standing one {standing}"
    );
}

#[test]
fn landing_does_not_skip_the_slams_wind_up() {
    // Hop, and press Slam the frame before the feet arrive. The wind-up is
    // still owed in full.
    let mut w = slam_at(Fx::ZERO);
    run(&mut w, 1, Input::SPACE, 0);
    let mut pressed_at = None;
    let mut active_at = None;
    for f in 0..80 {
        let p = w.players[0];
        // One frame of fall left: this frame's step would reach the floor.
        let landing =
            !p.grounded && p.vel.y.raw() < 0 && p.pos.y.add(p.vel.y.mul(sim::DT)).raw() <= 0;
        let bits = if pressed_at.is_none() && landing {
            pressed_at = Some(f);
            M
        } else {
            0
        };
        run(&mut w, 1, bits, 0);
        if active_at.is_none()
            && matches!(w.players[0].action, Action::Active { kind, .. } if kind == sim::state::SLOT_COMMITTED)
        {
            active_at = Some(f);
        }
    }
    let (Some(pressed), Some(active)) = (pressed_at, active_at) else {
        panic!("the fixture never slammed: pressed {pressed_at:?}, active {active_at:?}");
    };
    let startup = sim::moves::get(Class::Bulwark, sim::state::SLOT_COMMITTED).startup as i32;
    assert!(
        active - pressed >= startup,
        "pressed a frame before landing, active {} frames later against a {startup}-frame wind-up",
        active - pressed
    );
}

#[test]
fn the_leap_catches_the_shield_in_the_air() {
    let mut w = World::new();
    let load = Fx::from_int(300);
    set_weight(&mut w, 0, load);
    run(&mut w, 1, Input::MECHANIC, 0);
    run(&mut w, 5, 0, 0);
    run(&mut w, 1, Input::MECHANIC, 0);
    let mut caught_airborne = false;
    for _ in 0..30 {
        run(&mut w, 1, 0, 0);
        let p = w.players[0];
        if p.shield().is_some_and(|s| s.in_hand()) {
            caught_airborne = !p.grounded;
            break;
        }
    }
    assert!(
        caught_airborne,
        "the leap did not bring the shield back in the air"
    );
    assert!(weight(&w, 0).raw() > 0, "the shield came back empty");
}
