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
}
