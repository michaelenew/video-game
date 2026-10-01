//! The follow camera against a creature that flies: the two camera rules
//! `docs/design/creatures/galewing.md` §6 pins, as sentences.

use sim::fixed::Fx;
use sim::species::SpeciesId;
use sim::species::galewing as gw;
use sim::{Class, Input, V3, World};
use view::camera::RigConfig;
use view::{CameraRig, Framing, Surroundings};

/// A hunt on the Cliffs, a frame in so the bird is set up.
fn cliffs() -> World {
    let mut w = World::hunt_of(
        [Class::Champion; sim::state::MAX_PLAYERS],
        SpeciesId::GALEWING,
    );
    w.advance([Input::default(); sim::state::MAX_PLAYERS]);
    w
}

/// Where the fighter stands: on the plateau, out in the open.
fn spot(w: &World) -> [f32; 3] {
    let top = gw::fight::base(w).to_f32_for_render();
    [0.0, top, -10.0]
}

fn fx(v: f32) -> Fx {
    Fx::from_raw((v * 65536.0) as i32)
}

/// The rig, settled on a fighter standing at `at` looking along +x at
/// `pitch` radians.
fn settled(w: &World, at: [f32; 3], pitch: f32, beasts: bool, aboard: bool) -> Framing {
    let mut rig = CameraRig::new(RigConfig::default());
    let around = Surroundings {
        beasts: if beasts { &w.monsters } else { &[] },
        aboard,
        arena: w.arena(),
        ..Default::default()
    };
    for _ in 0..240 {
        rig.update_around(0.016, at, 0.0, pitch, around);
    }
    rig.update_around(0.016, at, 0.0, pitch, around)
}

/// The aiming eye: `sim::camera`, for a fighter at `at` looking up at
/// `pitch` radians along +x.
fn aiming_eye(w: &World, at: [f32; 3], pitch: f32) -> [f32; 3] {
    let look = Input::looking_at(0, 0, view::pitch_from_radians(pitch));
    let e = sim::camera::eye_under(
        V3::new(fx(at[0]), fx(at[1]), fx(at[2])),
        look,
        Fx::ZERO,
        w.arena(),
    );
    [
        e.x.to_f32_for_render(),
        e.y.to_f32_for_render(),
        e.z.to_f32_for_render(),
    ]
}

fn near(a: [f32; 3], b: [f32; 3], by: f32) -> bool {
    (0..3).all(|i| (a[i] - b[i]).abs() < by)
}

/// **A creature overhead must not lift the camera** (§6). The camera settles
/// the drawn eye onto the surface under it, and asking for the highest solid
/// top over that spot at any height put it on the back of a bird passing a
/// few metres over the eye. The floor under the eye is the highest surface
/// below the fighter's own feet.
#[test]
fn a_creature_overhead_does_not_lift_the_camera() {
    let mut w = cliffs();
    let at = spot(&w);
    for pitch in [-0.3f32, 0.0, 0.2] {
        let bare = settled(&w, at, pitch, false, false);
        // The bird low over the eye: its belly a few metres over the
        // fighter's head.
        let m = w.monsters[0].as_mut().unwrap();
        m.pos = V3::new(fx(bare.eye[0]), fx(at[1] + 4.5), fx(bare.eye[2]));
        m.yaw = Fx::ZERO;
        let over = m.top_under(m.pos, Fx::from_int(1000));
        assert!(
            over.to_f32_for_render() > at[1] + 4.5,
            "the bird is not over the eye at all (top {over:?})"
        );
        let under = settled(&w, at, pitch, true, false);
        assert!(
            near(bare.eye, under.eye, 0.01),
            "at pitch {pitch} a bird overhead moved the camera from {:?} to {:?}",
            bare.eye,
            under.eye
        );
    }
}

/// **The drawn eye and the aiming eye stay one point looking up** (§6): the
/// fight is spent looking up, and if the drawn camera were clamped somewhere
/// the aiming eye is not, the crosshair would sit on one thing while the ray
/// went past it. On the plateau, and aboard.
#[test]
fn the_drawn_eye_is_the_aiming_eye_looking_up() {
    let mut w = cliffs();
    let at = spot(&w);
    for degrees in [10.0f32, 40.0, 70.0] {
        let pitch = degrees.to_radians();
        let drawn = settled(&w, at, pitch, true, false);
        let aimed = aiming_eye(&w, at, pitch);
        assert!(
            near(drawn.eye, aimed, 0.02),
            "on the plateau at {degrees} degrees up the drawn eye is {:?} and the \
             aiming eye {aimed:?}",
            drawn.eye
        );
    }
    // Aboard: standing on its back, level, low over the plateau.
    let m = w.monsters[0].as_mut().unwrap();
    m.pos = V3::new(Fx::ZERO, fx(at[1] + 1.0), fx(at[2]));
    m.yaw = Fx::ZERO;
    let back = m.rig().top_face(gw::BACK);
    let half = |a: Fx, b: Fx| a.add(b).to_f32_for_render() * 0.5;
    let top = back
        .iter()
        .map(|c| c.y.to_f32_for_render())
        .fold(f32::MIN, f32::max);
    let riding = [half(back[0].x, back[2].x), top, half(back[0].z, back[2].z)];
    for degrees in [10.0f32, 40.0, 70.0] {
        let pitch = degrees.to_radians();
        let drawn = settled(&w, riding, pitch, true, true);
        let aimed = aiming_eye(&w, riding, pitch);
        assert!(
            near(drawn.eye, aimed, 0.02),
            "aboard at {degrees} degrees up the drawn eye is {:?} and the aiming \
             eye {aimed:?}",
            drawn.eye
        );
    }
}
