//! The eye's bubble: off the ground, looking up, the camera goes back out to
//! third person as far as the floor under the fighter lets it.
//!
//! On the ground, looking up shrinks the camera's sphere onto the head (the
//! handover, `camera::ball`), because an eye swung down behind a standing
//! fighter would go into the floor. In the air the floor is somewhere else,
//! so the shrink is undone as far as keeping the eye `Eye bubble` off that
//! floor allows (2026-10-12, the owner's brief: "the camera should pull back
//! to third person" in the air, "I DO NOT want regressions").
//!
//! These are the no-regression promises, as properties:
//!
//! - standing, the eye is the eye it always was, at every pitch;
//! - looking level or down, height changes nothing the airborne hold did not;
//! - the crosshair stays on the fighter: the look passes through the head
//!   however far back the eye is let out;
//! - the eye never comes within the bubble of the floor by being let out;
//! - nothing jumps -- in pitch, in height, or stepping off a ledge.

use sim::camera;
use sim::{Class, Fx, Input, V3, World};

fn m(v: f32) -> Fx {
    Fx::ratio((v * 1000.0) as i32, 1000)
}

fn f(v: Fx) -> f32 {
    v.to_f32_for_render()
}

/// Degrees in the wire's unit, up positive.
fn deg(d: f32) -> i16 {
    (d * 65536.0 / 360.0) as i16
}

fn look(pitch: f32) -> Input {
    Input::looking_at(0, 0, deg(pitch))
}

fn bubble() -> f32 {
    f(Fx::from_raw(sim::oven::view(
        sim::oven::ViewKnob::EyeBubble,
    )))
}

fn sphere() -> f32 {
    f(Fx::from_raw(sim::oven::view(sim::oven::ViewKnob::Sphere)))
}

const FEET: f32 = 10.0;

fn feet() -> V3 {
    V3::new(Fx::ZERO, m(FEET), Fx::ZERO)
}

fn dist(a: V3, b: V3) -> f32 {
    let d = a.sub(b);
    (f(d.x).powi(2) + f(d.y).powi(2) + f(d.z).powi(2)).sqrt()
}

#[test]
fn standing_the_eye_is_the_eye_it_always_was() {
    // `clear` is nought with the feet on anything, and nought is the zones'
    // own sphere exactly -- at every pitch, with any airborne framing.
    for p in -85..=85 {
        for aloft in [Fx::ZERO, Fx::ratio(1, 2), Fx::ONE] {
            let pitch = Fx::ratio(p, 360);
            assert_eq!(camera::let_out_share(pitch, aloft, Fx::ZERO), Fx::ZERO);
            let b = camera::ball(pitch, aloft, Fx::ZERO);
            // The handover's own radius: the full sphere at level, the head's
            // from the head lock up.
            if p >= 10 {
                assert!(f(b.radius) < 0.3, "{p}°: {:?}", b);
            }
        }
    }
}

#[test]
fn looking_level_or_down_height_changes_nothing() {
    // Those are the zones that already keep the full sphere: there is
    // nothing for the bubble to let out.
    for p in -85..=0 {
        for c in [0.5, 2.0, 6.0, 20.0] {
            let at = look(p as f32);
            assert_eq!(
                camera::eye(feet(), at, Fx::ONE, m(c)),
                camera::eye(feet(), at, Fx::ONE, Fx::ZERO),
                "{p}° at {c} m"
            );
        }
    }
}

#[test]
fn in_the_air_looking_up_the_camera_is_third_person_and_on_the_head() {
    let body = f(sim::tuning::body_height());
    let head = V3::new(Fx::ZERO, m(FEET + body), Fx::ZERO);
    for p in [12.0, 20.0, 30.0, 45.0, 60.0, 80.0] {
        let at = look(p);
        let eye = camera::eye(feet(), at, Fx::ONE, m(6.0));
        let back = dist(eye, head);
        assert!(back > 3.0, "{p}°: the eye is only {back} m off the head");
        assert!(back <= sphere() + 0.01, "{p}°: past the sphere: {back}");
        // The crosshair is the look direction through the screen's middle; it
        // passes through the head however far back the eye has gone.
        let dir = at.look_dir();
        let (d, to) = (
            [f(dir.x), f(dir.y), f(dir.z)],
            [
                f(head.x.sub(eye.x)),
                f(head.y.sub(eye.y)),
                f(head.z.sub(eye.z)),
            ],
        );
        let along = d[0] * to[0] + d[1] * to[1] + d[2] * to[2];
        let cross = [
            d[1] * to[2] - d[2] * to[1],
            d[2] * to[0] - d[0] * to[2],
            d[0] * to[1] - d[1] * to[0],
        ];
        let off = (cross[0].powi(2) + cross[1].powi(2) + cross[2].powi(2)).sqrt();
        assert!(
            along > 0.0 && off < 0.05,
            "{p}°: the look misses the head by {off} m"
        );
    }
}

#[test]
fn letting_out_never_brings_the_eye_within_the_bubble_of_the_floor() {
    for c in [0.2, 0.7, 1.0, 1.5, 2.5, 4.0, 8.0] {
        for p in 0..=85 {
            let at = look(p as f32);
            let floor = FEET - c;
            let out = f(camera::eye(feet(), at, Fx::ONE, m(c)).y) - floor;
            let inside = f(camera::eye(feet(), at, Fx::ONE, Fx::ZERO).y) - floor;
            // Where the zones' own eye was already lower (a body hanging just
            // off the floor, looking up), it stays where the zones put it.
            assert!(
                out >= bubble().min(inside) - 0.02,
                "{p}° at {c} m: the eye is {out} m off the floor"
            );
        }
    }
}

#[test]
fn the_eye_moves_continuously_with_pitch_and_with_height() {
    // A quarter of a degree of mouse, or a centimetre of height, never moves
    // the eye further than a smooth camera would. (The handover itself, on
    // the ground, moves it up to about a fifth of a metre a quarter-degree:
    // nine metres of sphere shrunk to the head over ten degrees.)
    let sweep = |c: f32| {
        let mut last: Option<V3> = None;
        let mut most = 0.0f32;
        for q in -340..=340 {
            let eye = camera::eye(feet(), look(q as f32 / 4.0), Fx::ONE, m(c));
            if let Some(prev) = last {
                most = most.max(dist(eye, prev));
            }
            last = Some(eye);
        }
        most
    };
    // The handover on the ground is the fastest the eye has ever moved with
    // the mouse -- nine metres of sphere shrunk onto the head over ten
    // degrees -- and in the air it is never much faster than that. Not
    // never faster at all: a metre or so off the floor, a few degrees up,
    // the handover's shrink and the floor's limit taking hold land in the
    // same degree, and together they run about a fifth over it with a
    // two-metre soft edge -- half over it with a one-metre one, which is why
    // the soft edge is two. Continuous either way: this is a speed, not a
    // jump.
    let ground = sweep(0.0);
    for c in [0.3, 1.0, 1.9, 3.0, 7.0, 30.0] {
        let air = sweep(c);
        assert!(
            air <= ground * 1.3,
            "at {c} m the eye jumps {air} m a step, against {ground}"
        );
    }
    for p in [5.0, 10.0, 20.0, 45.0, 80.0] {
        let mut last: Option<V3> = None;
        for cm in 0..=900 {
            let c = cm as f32 / 100.0;
            let eye = camera::eye(feet(), look(p), Fx::ONE, m(c));
            if let Some(prev) = last {
                let step = dist(eye, prev);
                assert!(step < 0.1, "at {p}°, {c} m: the eye jumped {step} m");
            }
            last = Some(eye);
        }
    }
}

#[test]
fn the_room_opens_at_a_limited_rate_and_closes_at_once() {
    let rate = f(Fx::from_raw(sim::oven::view(
        sim::oven::ViewKnob::EyeBubbleRate,
    )));
    // A ledge stepped off: the floor is eight metres away in one frame.
    let mut clear = Fx::ZERO;
    for _ in 0..4 {
        let next = camera::clear_step(clear, m(8.0));
        assert!(f(next.sub(clear)) <= rate + 0.001);
        clear = next;
    }
    // And a floor arriving is believed the frame it arrives.
    assert_eq!(camera::clear_step(clear, m(0.3)), m(0.3));
    assert_eq!(camera::clear_step(clear, Fx::ZERO), Fx::ZERO);
}

#[test]
fn a_fighter_knows_the_room_in_the_air_and_none_on_the_floor() {
    let mut w = World::with_classes([Class::Bulwark, Class::Bulwark]);
    assert_eq!(w.players[0].clear, Fx::ZERO);
    let mut most = Fx::ZERO;
    let mut was = Fx::ZERO;
    for frame in 0..90 {
        let bits = if frame < 2 { Input::SPACE } else { 0 };
        w.advance([Input::looking_at(bits, 0, deg(30.0)), Input::default()]);
        let p = &w.players[0];
        if p.grounded {
            assert_eq!(p.clear, Fx::ZERO, "frame {frame}: standing, and room");
        }
        most = most.max(p.clear);
        // The room under the eye never runs ahead of the feet.
        let height = p.pos.y;
        assert!(p.clear.raw() <= height.raw().max(0) + 1, "frame {frame}");
        // And never opens faster than its rate.
        let rate = Fx::from_raw(sim::oven::view(sim::oven::ViewKnob::EyeBubbleRate));
        assert!(p.clear.sub(was).raw() <= rate.raw(), "frame {frame}");
        was = p.clear;
    }
    assert!(f(most) > 0.5, "a jump should leave the floor: {}", f(most));
    assert_eq!(w.players[0].clear, Fx::ZERO, "landed again");
}
