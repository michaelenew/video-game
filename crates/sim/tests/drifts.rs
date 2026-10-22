//! **Drifts: boxes that move** (`sim::drift`, 2026-10-22). A drift is where
//! the frame number says, and nothing about it is in the snapshot; it is a
//! solid like any other near a fighter; it carries a body on it, hanging
//! from it or against it; and a body leaving it keeps its speed. Read off
//! the Needle's first lift and its shuttle, the drifts the valley has.

use sim::arena::spire::{self, AT, SHAFT};
use sim::arena::{ArenaId, Shape};
use sim::drift::{self, Drift, Hold};
use sim::valley::Journey;
use sim::{Class, Fx, Input, V3, World};

fn cm(v: i32) -> Fx {
    sim::arena::cm(v)
}

/// The plaza's height at the shaft, which every height of the Needle's
/// table is over.
fn base() -> Fx {
    let atlas = sim::atlas::valley();
    let (x, z) = (cm(AT.0), cm(AT.1));
    let r = cm(1000);
    atlas
        .near((x.sub(r), z.sub(r)), (x.add(r), z.add(r)))
        .map(|i| atlas.solids[i as usize])
        .filter(|s| matches!(s.shape, Shape::Round { hollow, .. } if hollow == Fx::ZERO))
        .map(|s| s.max.y)
        .filter(|y| y.to_int() > 200)
        .min()
        .unwrap()
        .sub(cm(SHAFT.2))
}

fn needle(class: Class) -> World {
    let j = Journey {
        beaten: u32::MAX,
        ..Journey::default()
    };
    World::arrive([class, class], j, ArenaId::SPIRE, Some(0), 1)
}

/// A map drift's box, in the world's frame on the world's frame.
fn now(w: &World, d: &Drift) -> sim::arena::Solid {
    let o = w.map_origin();
    let s = d.at(w.frame);
    sim::arena::Solid {
        min: s.min.sub(o),
        max: s.max.sub(o),
        ..s
    }
}

/// Stand fighter one on the middle of a box.
fn stand_on(w: &mut World, s: sim::arena::Solid) {
    let p = &mut w.players[0];
    p.pos = V3::new(
        Fx::from_raw(s.min.x.raw() / 2 + s.max.x.raw() / 2),
        s.max.y,
        Fx::from_raw(s.min.z.raw() / 2 + s.max.z.raw() / 2),
    );
    p.vel = V3::ZERO;
    p.grounded = true;
    p.fall_over = p.pos.y;
}

fn idle(w: &mut World) {
    w.advance([Input::new(0), Input::new(0)]);
}

/// Advance until the drift is `share` of the way, going out.
fn until(w: &mut World, d: &Drift, share: Fx) {
    for _ in 0..d.period() * 2 {
        let k = w.frame.wrapping_add(d.phase as u32) % d.period();
        if k >= d.wait as u32 && k < (d.wait + d.go) as u32 && d.share(w.frame).raw() >= share.raw()
        {
            return;
        }
        idle(w);
    }
    panic!("never got there");
}

#[test]
fn a_drift_is_where_its_frame_says() {
    let d = spire::drifts(base())[0];
    let p = d.period();
    for f in [0, 17, 300, 999] {
        assert_eq!(d.at(f), d.at(f + p), "a cycle on, it is where it was");
    }
    assert_eq!(d.share(0), Fx::ZERO, "it starts waiting at its first stop");
    let far = d.wait as u32 + d.go as u32 + 1;
    assert_eq!(d.share(far), Fx::ONE, "and stays at its second");
    // Its velocity is how far it went coming into the frame, a second's worth.
    let mid = d.wait as u32 + d.go as u32 / 2;
    let v = d.velocity(mid);
    assert!(v.y.raw() > 0 && v == d.step(mid).scale(Fx::from_int(sim::TICK_HZ as i32)));
    // The two cars of a lift are half a cycle apart: the second is where
    // the first is half a cycle later.
    let other = spire::drifts(base())[1];
    assert_eq!(other.share(0), d.share(p / 2));
}

/// **A body on a lift goes up with it, and one jumping off a rising lift
/// keeps its rise**: stood on the first car at its foot and left alone, it
/// is on the deck all the way and at the head when the car is; jumped off
/// halfway up, it leaves faster than a jump from the ground.
#[test]
fn a_lift_carries_a_body_and_a_jump_off_it_keeps_its_rise() {
    let d = spire::drifts(base())[0];
    let mut w = needle(Class::Champion);
    // At its foot, waiting.
    for _ in 0..d.period() * 2 {
        let k = w.frame.wrapping_add(d.phase as u32) % d.period();
        if k + 30 < d.wait as u32 {
            break;
        }
        idle(&mut w);
    }
    let deck = now(&w, &d);
    stand_on(&mut w, deck);
    let from = w.players[0].pos.y;
    let mut rose = Fx::ZERO;
    for _ in 0..d.wait as u32 + d.go as u32 {
        idle(&mut w);
        let deck = now(&w, &d);
        assert_eq!(
            drift::attached(&w.players[0], &deck),
            Some(Hold::On),
            "off the deck at {} m",
            w.players[0].pos.y.to_f32_for_render()
        );
        rose = w.players[0].pos.y.sub(from);
    }
    let lift = &spire::LIFTS[0];
    assert!(
        rose.sub(cm(lift.head - lift.foot)).abs().raw() < Fx::ratio(1, 10).raw(),
        "carried {} m",
        rose.to_f32_for_render()
    );

    // Halfway up, a jump.
    let mut w = needle(Class::Champion);
    for _ in 0..d.period() * 2 {
        let k = w.frame.wrapping_add(d.phase as u32) % d.period();
        if k + 30 < d.wait as u32 {
            break;
        }
        idle(&mut w);
    }
    let deck = now(&w, &d);
    stand_on(&mut w, deck);
    until(&mut w, &d, Fx::ratio(1, 2));
    let lifting = d.velocity(w.frame.wrapping_add(1)).y;
    w.advance([Input::new(Input::SPACE), Input::new(0)]);
    let off = w.players[0].vel.y;
    let mut ground = needle(Class::Champion);
    ground.players[0].vel = V3::ZERO;
    ground.advance([Input::new(Input::SPACE), Input::new(0)]);
    let plain = ground.players[0].vel.y;
    assert!(
        off.sub(plain).sub(lifting).abs().raw() < Fx::ONE.raw(),
        "off the lift at {} m/s, from the ground {} m/s, the lift rising {} m/s",
        off.to_f32_for_render(),
        plain.to_f32_for_render(),
        lifting.to_f32_for_render()
    );
}

/// **A jump off the shuttle keeps its run**: stood on it as it sets off and
/// jumped off at its fastest, a body leaves going the shuttle's way at the
/// shuttle's speed and more.
#[test]
fn a_jump_off_a_moving_stone_keeps_its_speed() {
    let d = spire::shuttle(base());
    let mut w = needle(Class::Champion);
    for _ in 0..d.period() * 2 {
        let k = w.frame.wrapping_add(d.phase as u32) % d.period();
        if k + 30 < d.wait as u32 {
            break;
        }
        idle(&mut w);
    }
    let deck = now(&w, &d);
    stand_on(&mut w, deck);
    until(&mut w, &d, Fx::ratio(1, 2));
    let going = d.velocity(w.frame.wrapping_add(1)).x;
    w.advance([Input::new(Input::SPACE), Input::new(0)]);
    let p = &w.players[0];
    assert!(!p.grounded);
    assert!(
        p.vel.x.sub(going).abs().raw() < Fx::ratio(1, 2).raw()
            && going.raw() > Fx::from_int(5).raw(),
        "left at {} m/s with the shuttle going {} m/s",
        p.vel.x.to_f32_for_render(),
        going.to_f32_for_render()
    );
}

/// **A ride replays bit for bit** (rollback): a world copied a third of the
/// way up the first lift, and both advanced with the same inputs -- walking,
/// jumping off, landing -- agree on every frame: nothing about a drift is
/// anywhere but the frame number, so a re-simulated frame puts it where it
/// was.
#[test]
fn a_ride_replays_bit_for_bit() {
    let d = spire::drifts(base())[0];
    let mut w = needle(Class::ShadowReaver);
    for _ in 0..d.period() * 2 {
        let k = w.frame.wrapping_add(d.phase as u32) % d.period();
        if k + 30 < d.wait as u32 {
            break;
        }
        idle(&mut w);
    }
    let deck = now(&w, &d);
    stand_on(&mut w, deck);
    until(&mut w, &d, Fx::ratio(1, 3));
    let mut copy = w.clone();
    for f in 0..240u32 {
        let bits = if f % 50 == 10 { Input::SPACE } else { Input::W };
        let input = [Input::looking_at(bits, (f * 97) as u16, 0), Input::new(0)];
        w.advance(input);
        copy.advance(input);
        assert_eq!(w.checksum(), copy.checksum(), "frame {f}");
    }
}
