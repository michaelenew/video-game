//! What happens when the camera sliders are dragged somewhere unreasonable.
//!
//! Its own file, and therefore its own process, because these move the Oven's
//! live values and the rest of the suite reads them from the same place while
//! running alongside. A test that quietly retunes the camera under another test
//! is worse than no test.

use view::{CameraRig, camera::RigConfig};

fn settled(pitch: f32) -> view::camera::Framing {
    let mut rig = CameraRig::new(RigConfig::default());
    let at = [0.0, 0.0, 8.0];
    for _ in 0..120 {
        rig.update(0.016, at, 0.0, pitch);
    }
    rig.update(0.016, at, 0.0, pitch)
}

fn swept(what: &str) {
    let z = view::camera::Zones::tuned();
    for step in 0..=40 {
        let pitch = -z.down_limit + step as f32 / 40.0 * (z.down_limit + z.up_limit);
        let f = settled(pitch);
        assert!(
            f.eye.iter().all(|v| v.is_finite()) && f.look_at.iter().all(|v| v.is_finite()),
            "{what} put the camera at {:?} looking at {:?} at {:.0} degrees",
            f.eye,
            f.look_at,
            pitch.to_degrees()
        );
        assert!(
            (0.0..=1.0).contains(&f.hidden),
            "{what} asked for {:.2} of the body to be hidden",
            f.hidden
        );
    }
}

#[test]
fn sliders_dragged_somewhere_unreasonable_do_not_break_the_camera() {
    // One test rather than three, walked in sequence, because these all write
    // the same live values: run as separate tests they would be retuning the
    // camera underneath each other and none of them would mean anything.
    use sim::oven::{ViewKnob as V, set_view, view};

    // A sphere of no radius has no direction to be behind: the eye would sit
    // exactly on the fighter with nothing to point away from. One drag away,
    // and the arithmetic after it divides by what is left.
    let (sphere, head) = (view(V::Sphere), view(V::HeadSphere));
    set_view(V::Sphere, 0);
    set_view(V::HeadSphere, 0);
    swept("a sphere of no radius");
    set_view(V::Sphere, sphere);
    set_view(V::HeadSphere, head);

    // The zone boundaries are independent sliders with no ordering between
    // them, so a handover can be asked to finish before it starts. Every zone
    // is a straight interpolation and the fractions are clamped rather than
    // trusted, precisely so that an inside-out zone reads as an abrupt camera
    // rather than as a crash.
    let (floor, neutral, lock) = (
        view(V::FloorZoneFrom),
        view(V::NeutralZoneTo),
        view(V::HeadLockAt),
    );
    set_view(V::FloorZoneFrom, 10);
    set_view(V::NeutralZoneTo, 45);
    set_view(V::HeadLockAt, 1);
    swept("zones dragged past each other");
    set_view(V::FloorZoneFrom, floor);
    set_view(V::NeutralZoneTo, neutral);
    set_view(V::HeadLockAt, lock);

    // And the radius is still what decides how big the fighter is drawn, which
    // is what the player's own distance setting used to do. It cannot be one
    // any more: the eye is where the aiming ray starts, so a player who pulled
    // the camera back would be aiming somewhere else.
    {
        // What used to be the player's own distance setting. It cannot be one any
        // more: the eye is where the aiming ray starts, so a player who pulled the
        // camera back would be aiming somewhere else -- see `sim::camera`. The
        // radius is a tuned number now, shared like every other number that decides
        // what happens, and pulling it back is still what makes the fighter smaller.
        let fov = RigConfig::default().fov;
        let on_screen = |f: view::camera::Framing, point: [f32; 3]| {
            let to = |p: [f32; 3]| {
                let v = [p[0] - f.eye[0], p[1] - f.eye[1], p[2] - f.eye[2]];
                let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-4);
                [v[0] / len, v[1] / len, v[2] / len]
            };
            let centre = to(f.look_at);
            let target = to(point);
            let dot: f32 = (0..3).map(|i| centre[i] * target[i]).sum();
            let angle = dot.clamp(-1.0, 1.0).acos();
            let signed = if target[1] < centre[1] { -angle } else { angle };
            0.5 + signed.tan() / (2.0 * (fov * 0.5).tan())
        };

        let pitch = view::camera::Zones::tuned().neutral_pitch();
        let original = view(V::Sphere);
        let mut sizes = Vec::new();
        for metres in [4.0f32, 7.0, 11.0] {
            set_view(V::Sphere, (metres * 65536.0) as i32);
            let f = settled(pitch);
            let body = sim::tuning::body_height().to_f32_for_render();
            sizes.push(on_screen(f, [0.0, body, 8.0]) - on_screen(f, [0.0, 0.0, 8.0]));
        }
        set_view(V::Sphere, original);
        assert!(
            sizes[0] > sizes[1] && sizes[1] > sizes[2],
            "the fighter did not shrink as the sphere grew: {sizes:?}"
        );
    }
}
