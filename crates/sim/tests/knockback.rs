//! Knockback between fighters, in a real fight (`docs/design/stun.md`).
//!
//! What is pinned is the shape, not the lengths -- those are the Oven's
//! *Knockback* family and move freely. The properties that must survive
//! tuning: the same blow throws a fighter further the more of their bar is
//! gone, a heavy class travels less than a light one, and none of it reaches
//! a creature's fight (`tests/ridgeback_pin.rs` holds that bit for bit).

use sim::class::Class;
use sim::{Fx, Input, V3, World};

const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

/// A Champion a step from `victim` on open floor, the victim at `health` of
/// their own bar, and how far the hammer's first blow carries them.
///
/// Out on open floor for the reason `tests/impact.rs` gives: a body shoved
/// against a platform slides nowhere, which reads exactly like knockback that
/// does not work.
fn hammer_carries(victim: Class, health: (i32, i32)) -> Fx {
    let mut w = World::with_classes([Class::Champion, victim]);
    let y = w.players[0].pos.y;
    w.players[0].pos = V3::new(Fx::ZERO, y, Fx::from_int(8));
    w.players[1].pos = V3::new(Fx::ratio(6, 5), y, Fx::from_int(8));
    let full = w.players[1].full_health();
    w.players[1].health = full * health.0 / health.1;
    for _ in 0..2 {
        w.advance([Input::aimed(0, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
    }
    let start = w.players[1].pos;
    let mut landed = false;
    for f in 0..120 {
        let press = if f == 0 { Input::MIDDLE } else { 0 };
        w.advance([Input::aimed(press, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
        landed |= w.players[1].health < full * health.0 / health.1;
    }
    assert!(landed, "the hammer never connected on the {victim:?}");
    let moved = w.players[1].pos.sub(start);
    V3::new(moved.x, Fx::ZERO, moved.z).flat_len()
}

#[test]
fn the_same_blow_throws_a_hurt_fighter_further() {
    let fresh = hammer_carries(Class::Champion, (1, 1));
    let half = hammer_carries(Class::Champion, (1, 2));
    let low = hammer_carries(Class::Champion, (1, 5));
    println!("hammer on a Champion: full {fresh:?} m, half {half:?} m, a fifth {low:?} m");
    assert!(
        fresh.raw() > Fx::ratio(1, 2).raw(),
        "the hammer barely moves a fresh Champion: {fresh:?} m"
    );
    assert!(
        half.raw() > fresh.raw() && low.raw() > half.raw(),
        "no swell: full {fresh:?}, half {half:?}, a fifth {low:?}"
    );
}

#[test]
fn a_heavy_travels_less_than_a_light() {
    let heavy = hammer_carries(Class::Bulwark, (1, 1));
    let light = hammer_carries(Class::DualMage, (1, 1));
    println!("hammer, fresh: Bulwark {heavy:?} m, Dual mage {light:?} m");
    assert!(
        heavy.raw() < light.raw(),
        "the Bulwark went {heavy:?} m and the Dual mage {light:?} m"
    );
}
