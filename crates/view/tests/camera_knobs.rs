//! What happens when the camera sliders are dragged somewhere unreasonable.
//!
//! Its own file, and therefore its own process, because these move the Oven's
//! live values and the rest of the suite reads them from the same place while
//! running alongside. A test that quietly retunes the camera under another
//! test is worse than no test.

use view::{CameraRig, camera::RigConfig};

fn settled(pitch: f32) -> view::camera::Framing {
    let mut rig = CameraRig::new(RigConfig::default());
    let at = [0.0, 0.0, 8.0];
    let look = sim::Input::looking_at(0, 0, view::pitch_from_radians(pitch));
    let from = sim::aim::origin(sim::V3::new(
        sim::Fx::ZERO,
        sim::Fx::ZERO,
        sim::Fx::from_int(8),
    ));
    let far = sim::Fx::from_int(60);
    let dir = look.look_dir();
    let stones = [None; sim::stones::MAX_STONES];
    let hit = sim::aim::trace(from, dir, far, &stones).unwrap_or(far);
    let at_point = from.add(dir.scale(hit));
    let target = [
        at_point.x.to_f32_for_render(),
        at_point.y.to_f32_for_render(),
        at_point.z.to_f32_for_render(),
    ];
    for _ in 0..200 {
        rig.update(0.016, at, 0.0, pitch, target);
    }
    rig.update(0.016, at, 0.0, pitch, target)
}

#[test]
fn crossing_the_two_elevation_sliders_does_not_break_the_camera() {
    // The eye's floor and ceiling are independent knobs that sit on the same
    // number as tuned, so dragging one past the other is a single click away
    // while the player is watching the camera move. Worth pinning because the
    // obvious way to write that bound -- `clamp` -- panics on exactly this, and
    // a tuning slider must never be able to take the game down.
    use sim::oven::{ViewKnob as V, set_view};
    set_view(V::MinElevation, 80);
    set_view(V::MaxElevation, 40);

    let zones = view::camera::Zones::tuned();
    for step in 0..=20 {
        let pitch = -zones.down_limit * step as f32 / 20.0;
        let f = settled(pitch);
        assert!(
            f.eye.iter().all(|v| v.is_finite()),
            "crossed sliders put the eye at {:?} at {:.0} degrees",
            f.eye,
            pitch.to_degrees()
        );
        // And the ceiling is the one that wins, so the camera stays where the
        // knob that was lowered says rather than where the raised one asks.
        let up = f.eye[1];
        let back = (f.eye[0].powi(2) + (f.eye[2] - 8.0).powi(2)).sqrt();
        assert!(
            up.atan2(back) <= zones.max_elevation + 0.01,
            "the eye climbed past the ceiling it was given"
        );
    }
}
