//! Tests for the parts of presentation that must be correct rather than pretty.
//!
//! Everything here runs without a window, which is the point of `view` having
//! no engine dependency.

use sim::state::Action;
use sim::{Input, World};
use view::interp::{TickClock, interpolate};
use view::play::{PoseInput, pose_for};
use view::skeleton::Joint;
use view::{CameraRig, camera::RigConfig};

// ---------------------------------------------------------------------------
// Interpolation
// ---------------------------------------------------------------------------

// The Reaver's shadow is drawn as a second body on a second skeleton, so the
// renderer has to be handed one -- and only for the class that has one. These
// are the two ways that goes wrong silently: no shadow at all, and a shadow
// pinned to her so the pair read as one figure with a rendering fault.

fn reaver() -> World {
    World::with_classes([sim::Class::ShadowReaver, sim::Class::Bulwark])
}

#[test]
fn only_the_reaver_is_handed_a_second_body() {
    let mut w = reaver();
    for _ in 0..30 {
        w.advance([Input::default(); 2]);
    }
    let frame = interpolate(&w, &w, 1.0);
    assert!(
        frame.shadows[0].is_some(),
        "the Reaver has no shadow to draw"
    );
    assert!(
        frame.shadows[1].is_none(),
        "the Bulwark was handed a shadow it has no mechanic for"
    );
}

#[test]
fn the_shadow_is_drawn_behind_her_rather_than_on_her() {
    // Its copy of a swing comes out of wherever it is standing, so two bodies
    // in one place is two threats a player cannot tell apart.
    let mut w = reaver();
    for _ in 0..40 {
        w.advance([Input::default(); 2]);
    }
    let frame = interpolate(&w, &w, 1.0);
    let her = frame.players[0];
    let it = frame.shadows[0].expect("the Reaver has a shadow");
    let gap = [it.pos[0] - her.pos[0], it.pos[2] - her.pos[2]];
    let apart = (gap[0] * gap[0] + gap[1] * gap[1]).sqrt();
    assert!(apart > 0.3, "the shadow is drawn {apart:.2} m from her");
    let ahead = gap[0] * her.facing[0] + gap[1] * her.facing[2];
    assert!(ahead < 0.0, "the shadow is drawn in front of her");
}

#[test]
fn a_travelling_shadow_holds_the_dash_and_a_standing_one_does_not() {
    // Two clips, and the renderer's cross-fade between them is the arrival. If
    // both states picked the same pose there would be nothing to fade.
    use view::play::Ghosting;
    let mut w = reaver();
    for _ in 0..20 {
        w.advance([Input::default(); 2]);
    }
    let mut seen = Vec::new();
    for frame in 0..60 {
        // Right click sends the shadow on this class -- see `moves::on_e`.
        let bits = if frame == 0 { Input::RIGHT } else { 0 };
        w.advance([Input::new(bits), Input::default()]);
        if let Some(it) = interpolate(&w, &w, 1.0).shadows[0] {
            if !seen.contains(&std::mem::discriminant(&it.doing)) {
                seen.push(std::mem::discriminant(&it.doing));
            }
        }
    }
    assert!(
        seen.contains(&std::mem::discriminant(&Ghosting::Dashing(0))),
        "the shadow crossed the arena without ever being in the dash"
    );
    assert!(
        seen.contains(&std::mem::discriminant(&Ghosting::Ready(0))),
        "the shadow arrived without ever reaching the ready stance"
    );
}

fn walked(frames: u32) -> World {
    let mut w = World::new();
    for _ in 0..frames {
        w.advance([Input::new(Input::W), Input::default()]);
    }
    w
}

#[test]
fn alpha_zero_is_the_old_snapshot_and_one_is_the_new() {
    let prev = walked(10);
    let cur = walked(11);
    let a = interpolate(&prev, &cur, 0.0);
    let b = interpolate(&prev, &cur, 1.0);
    assert!((a.players[0].pos[0] - prev.players[0].pos.x.to_f32_for_render()).abs() < 1e-4);
    assert!((b.players[0].pos[0] - cur.players[0].pos.x.to_f32_for_render()).abs() < 1e-4);
}

#[test]
fn interpolation_lands_between_the_snapshots() {
    let prev = walked(10);
    let cur = walked(11);
    let mid = interpolate(&prev, &cur, 0.5).players[0].pos[0];
    let lo = prev.players[0].pos.x.to_f32_for_render();
    let hi = cur.players[0].pos.x.to_f32_for_render();
    assert!(mid > lo && mid < hi, "{lo} < {mid} < {hi}");
}

#[test]
fn facing_stays_unit_length_through_a_turn() {
    // A plain component lerp shortens the vector mid-rotation, which shows up
    // as the character visibly shrinking.
    let prev = walked(4);
    let cur = walked(30);
    for step in 0..=10 {
        let f = interpolate(&prev, &cur, step as f32 / 10.0).players[0].facing;
        let len = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-3, "facing length {len} at t={step}");
    }
}

#[test]
fn discrete_state_comes_from_the_newer_snapshot() {
    // There is no meaningful halfway point between startup frame 3 and 4.
    let mut prev = World::new();
    prev.advance([Input::new(Input::LEFT), Input::default()]);
    let mut cur = prev.clone();
    cur.advance([Input::default(), Input::default()]);
    let f = interpolate(&prev, &cur, 0.5);
    assert_eq!(f.players[0].action, cur.players[0].action);
    assert_eq!(f.sim_frame, cur.frame);
}

// ---------------------------------------------------------------------------
// Tick clock
// ---------------------------------------------------------------------------

#[test]
fn one_second_of_real_time_is_sixty_ticks() {
    let mut clock = TickClock::new();
    let mut ticks = 0;
    for _ in 0..120 {
        ticks += clock.advance(1.0 / 120.0); // 120 Hz display
    }
    assert_eq!(ticks, 60);
}

#[test]
fn a_long_stall_does_not_fast_forward_the_match() {
    let mut clock = TickClock::new();
    let ticks = clock.advance(30.0);
    assert!(ticks <= 5, "a stall produced {ticks} ticks of simulation");
}

#[test]
fn alpha_stays_in_range() {
    let mut clock = TickClock::new();
    for i in 0..500 {
        clock.advance(0.001 * i as f32);
        let a = clock.alpha();
        assert!((0.0..=1.0).contains(&a), "alpha {a}");
    }
}

// ---------------------------------------------------------------------------
// Camera
// ---------------------------------------------------------------------------

/// Flat direction the camera is pointed.
///
/// Measured eye-to-target. The camera points *at* the aim point rather than
/// along the look axis, and the eye is lifted above the fighter -- but lifted
/// straight up, on their own centre line, so the bearing is untouched and only
/// the pitch differs.
fn aim_dir(f: view::camera::Framing, _player: [f32; 3]) -> (f32, f32) {
    let (dx, dz) = (f.look_at[0] - f.eye[0], f.look_at[2] - f.eye[2]);
    let len = (dx * dx + dz * dz).sqrt();
    (dx / len, dz / len)
}

/// The tuned zones, which is what the camera is a solution to.
fn zones() -> view::camera::Zones {
    view::camera::Zones::tuned()
}

fn fov() -> f32 {
    view::camera::RigConfig::default().fov
}

fn settled(pitch: f32) -> (view::camera::Framing, [f32; 3]) {
    let mut rig = CameraRig::new(RigConfig::default());
    // Past the platforms in z, which reach four metres either side of the
    // middle, so the occlusion pull-in is not what these are measuring.
    let at = [0.0, 0.0, 8.0];
    for _ in 0..200 {
        rig.update(0.016, at, 0.0, pitch);
    }
    (rig.update(0.016, at, 0.0, pitch), at)
}

/// Where a world point lands, as a fraction up the screen from the bottom. The
/// crosshair is at one half by definition, because the camera points at it.
///
/// Through a tangent, because the screen is a flat plane a fixed distance in
/// front of the eye rather than an arc: twice the angle off centre is more than
/// twice the distance up the glass. It matters at the edges, which is where the
/// zones put the fighter.
fn on_screen(f: view::camera::Framing, point: [f32; 3]) -> f32 {
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
    0.5 + signed.tan() / (2.0 * (fov() * 0.5).tan())
}

/// How high around the sphere the eye has climbed, in radians.
fn elevation_at(pitch: f32) -> f32 {
    let (f, at) = settled(pitch);
    let back = ((f.eye[0] - at[0]).powi(2) + (f.eye[2] - at[2]).powi(2)).sqrt();
    (f.eye[1] - at[1]).atan2(back)
}

/// Where a waypoint written as a screen fraction actually lands on screen.
///
/// The two are the same number only when the player's field of view matches the
/// one the framing is written against. They are deliberately allowed to differ:
/// the framing has its own tuned field of view so that widening your view cannot
/// move your aim, which means a wider view shows more of the arena and puts the
/// fighter a little nearer the middle than the knob's percentage reads. This is
/// that conversion, so the waypoint tests stay exact through a change to either.
fn as_drawn(waypoint: f32) -> f32 {
    let framing = (sim::oven::view(sim::oven::ViewKnob::FramingFov) as f32).to_radians();
    let tilt = ((1.0 - 2.0 * waypoint) * (framing * 0.5).tan()).atan();
    0.5 - tilt.tan() / (2.0 * (fov() * 0.5).tan())
}

fn feet_of(at: [f32; 3]) -> [f32; 3] {
    [at[0], 0.0, at[2]]
}

fn head_of(at: [f32; 3]) -> [f32; 3] {
    [at[0], sim::tuning::body_height().to_f32_for_render(), at[2]]
}

fn ramp(from: f32, to: f32, at: f32) -> f32 {
    from + (to - from) * at
}

#[test]
fn the_camera_sits_behind_the_fighter() {
    let mut rig = CameraRig::new(RigConfig::default());
    let at = [2.0, 0.0, -1.0];
    let f = rig.update(0.016, at, 0.0, 0.0);
    // Looking down +X means the camera is at smaller X than the fighter.
    assert!(f.eye[0] < 2.0, "camera at {} is not behind", f.eye[0]);
    // And it is pointed straight down the look axis. That is what makes screen
    // centre the direction the player is pointing, and it is why the reticle
    // can sit exactly in the middle of the screen by construction rather than
    // by correction.
    let ahead = [
        f.look_at[0] - f.eye[0],
        f.look_at[1] - f.eye[1],
        f.look_at[2] - f.eye[2],
    ];
    let len = (ahead[0] * ahead[0] + ahead[1] * ahead[1] + ahead[2] * ahead[2]).sqrt();
    assert!(
        (ahead[1] / len).asin().abs() < 0.01 && (ahead[2] / len).abs() < 0.01,
        "the camera is looking along {ahead:?}, not down the look axis"
    );
    assert!(f.eye[1] > 0.0, "camera is underground");
}

#[test]
fn the_camera_and_the_simulation_agree_on_forward() {
    // The invariant that makes camera-relative movement work at all. If the
    // renderer's yaw convention and `move_dir`'s disagree, W walks sideways and
    // nothing about the code looks wrong.
    //
    // **Exactly**, and that is why the eye sits on the fighter's own centre
    // line. The camera is pointed at the target rather than along the look
    // axis, so an eye slid sideways would turn the whole view and W would stop
    // walking up the screen.
    for eighth in 0..8u32 {
        let aim = (eighth * (u16::MAX as u32 + 1) / 8) as u16;
        let radians = aim as f32 / 65536.0 * std::f32::consts::TAU;
        let at = [0.0, 0.0, 8.0];

        let mut rig = CameraRig::new(RigConfig::default());
        let f = rig.update(0.016, at, radians, 0.0);
        let (cx, cz) = aim_dir(f, at);

        let sim_fwd = sim::state::move_dir(Input::aimed(0, aim).aim_turns(), 0, 1);
        let (sx, sz) = (sim_fwd.x.to_f32_for_render(), sim_fwd.z.to_f32_for_render());
        assert!(
            (cx - sx).abs() < 0.02 && (cz - sz).abs() < 0.02,
            "at {aim}: camera looks ({cx:.3}, {cz:.3}), W walks ({sx:.3}, {sz:.3})"
        );
    }
}

#[test]
fn the_eye_stays_on_the_fighters_own_centre_line() {
    // Reported: the camera sat behind and to the right, and it was wrong.
    // Directly behind is not a preference, it is what keeps the bearing exact
    // once the camera is pointed at the aim point.
    let at = [0.0, 0.0, 8.0];
    for eighth in 0..8u32 {
        let yaw = eighth as f32 / 8.0 * std::f32::consts::TAU;
        let mut rig = CameraRig::new(RigConfig::default());
        let f = rig.update(0.016, at, yaw, 0.0);
        let (dx, dz) = (f.eye[0] - at[0], f.eye[2] - at[2]);
        let sideways = (dx * yaw.sin() - dz * yaw.cos()).abs();
        assert!(
            sideways < 0.01,
            "at yaw {yaw:.2} the eye is {sideways:.2} m off the fighter's centre line"
        );
    }
}

#[test]
fn strafe_is_perpendicular_to_forward() {
    // D should be exactly ninety degrees off W, at every aim angle, or
    // circle-strafing drifts.
    for eighth in 0..8u32 {
        let aim = Input::aimed(0, (eighth * (u16::MAX as u32 + 1) / 8) as u16).aim_turns();
        let fwd = sim::state::move_dir(aim, 0, 1);
        let right = sim::state::move_dir(aim, 1, 0);
        let dot = fwd.dot(right).to_f32_for_render();
        assert!(dot.abs() < 0.01, "forward . right = {dot}");
    }
}

#[test]
fn the_mouse_is_never_smoothed() {
    // A smoothed mouse feels broken in a way players cannot name. The camera
    // must be where it is asked on the very frame it is asked.
    let at = [0.0, 0.0, 8.0];
    let yaw = std::f32::consts::FRAC_PI_2;
    let mut snapped = CameraRig::new(RigConfig::default());
    for _ in 0..60 {
        snapped.update(0.016, at, 0.0, 0.0);
    }
    let turned = snapped.update(0.016, at, yaw, 0.0);

    let mut held = CameraRig::new(RigConfig::default());
    for _ in 0..60 {
        held.update(0.016, at, yaw, 0.0);
    }
    let rested = held.update(0.016, at, yaw, 0.0);

    let (ax, az) = aim_dir(turned, at);
    let (bx, bz) = aim_dir(rested, at);
    assert!(
        (ax - bx).abs() < 0.01 && (az - bz).abs() < 0.01,
        "camera lagged the mouse: ({ax:.3}, {az:.3}) against ({bx:.3}, {bz:.3})"
    );
}

// ---------------------------------------------------------------------------
// The zones
//
// The camera is prescribed as waypoints in the vertical aim angle, each one
// saying where the fighter should appear. These are those waypoints, checked
// against what the rig actually draws. The numbers come from the Oven, so what
// is pinned here is the *shape* -- which is the design -- and never a value.
// ---------------------------------------------------------------------------

#[test]
fn the_eye_rides_a_fixed_sphere() {
    // The one thing the rig is not allowed to do while the player is only
    // steering: change how far away it is. The eye has exactly one place to be
    // -- somewhere on a sphere of the tuned radius centred on the fighter's
    // feet -- so aiming around moves it *along* that sphere and never off it.
    //
    // This is what a camera that dollies in and out feels wrong about, and it
    // is worth pinning as a distance rather than as a framing, because a
    // framing can be met by a hundred cameras and only one of them is this one.
    let z = zones();
    for step in 0..=34 {
        let pitch = -ramp(z.neutral_to, z.down_limit, step as f32 / 34.0);
        let (f, at) = settled(pitch);
        let feet = feet_of(at);
        let radius = ((f.eye[0] - feet[0]).powi(2)
            + (f.eye[1] - feet[1]).powi(2)
            + (f.eye[2] - feet[2]).powi(2))
        .sqrt();
        assert!(
            (radius - z.sphere).abs() < 0.02,
            "at {:.0} degrees the eye is {radius:.2} m out, not the {:.2} m sphere",
            pitch.to_degrees(),
            z.sphere
        );
    }
}

#[test]
fn the_neutral_zone_holds_the_fighter_low() {
    // Where most of a match is spent, and the waypoint that decides it: the
    // feet near the bottom of the frame, so the fighter is not sitting on the
    // reticle.
    //
    // Across the whole zone, exactly, and for free: the sphere is centred on
    // the feet and the tilt is fixed, so the feet sit at the same place on the
    // screen wherever around the sphere the eye has walked to. Nothing is
    // solved and nothing can run out of room.
    let z = zones();
    let steps = 20;
    for step in 0..=steps {
        let pitch = -ramp(z.neutral_to, z.floor_from, step as f32 / steps as f32);
        let (f, at) = settled(pitch);
        let feet = on_screen(f, feet_of(at));
        // Tight, because this is arithmetic rather than an approximation.
        let want = as_drawn(z.feet_neutral);
        assert!(
            (feet - want).abs() < 0.005,
            "at {:.0} degrees the feet are at {:.1}% instead of {:.1}%",
            pitch.to_degrees(),
            feet * 100.0,
            want * 100.0
        );
    }
}

#[test]
fn the_mouse_walks_the_eye_around_the_sphere_at_its_own_rate() {
    // **The property the whole rig is built to have.** Nothing is solved: the
    // eye's place on the sphere is the tilt minus the pitch, so a degree of
    // mouse is a degree around the sphere, exactly, everywhere in the zone.
    //
    // It is worth pinning as an equality rather than as "it moves", because the
    // failure this replaced was a camera that answered the mouse over part of
    // its range and then quietly stopped -- which looked fine in every framing
    // test, since an eye parked in the right place still frames correctly.
    let z = zones();
    let shallow = elevation_at(-z.neutral_to);
    let steep = elevation_at(-z.floor_from);
    let swept = steep - shallow;
    let asked = z.floor_from - z.neutral_to;
    assert!(
        (swept - asked).abs() < 0.02,
        "the mouse moved {:.1} degrees and the eye moved {:.1} around the sphere",
        asked.to_degrees(),
        swept.to_degrees()
    );
    // And evenly, not in a rush at one end.
    for step in 1..=10 {
        let from = -ramp(z.neutral_to, z.floor_from, (step - 1) as f32 / 10.0);
        let to = -ramp(z.neutral_to, z.floor_from, step as f32 / 10.0);
        let step_swept = elevation_at(to) - elevation_at(from);
        assert!(
            (step_swept - asked / 10.0).abs() < 0.02,
            "the eye moved {:.1} degrees over a tenth of the zone that asked for {:.1}",
            step_swept.to_degrees(),
            (asked / 10.0).to_degrees()
        );
    }
}

#[test]
fn past_the_neutral_zone_the_eye_moves_and_the_view_pans() {
    // The floor zone does two things at once, and it takes both. The eye keeps
    // working around the sphere, *and* the view pans the fighter up toward the
    // crosshair. An earlier pass pinned the eye and left the pan to do all of
    // it, which reads as the camera stopping halfway through a turn the player
    // can feel they are still making.
    let z = zones();
    let eye_at = |pitch: f32| {
        let (f, at) = settled(pitch);
        (
            [f.eye[0] - at[0], f.eye[1] - at[1], f.eye[2] - at[2]],
            on_screen(f, feet_of(at)),
        )
    };
    let steps = 40;
    let mut travelled = 0.0;
    let (mut was, started_at) = eye_at(-z.floor_from);
    for step in 1..=steps {
        let pitch = -ramp(z.floor_from, z.down_limit, step as f32 / steps as f32);
        let (eye, _) = eye_at(pitch);
        travelled +=
            ((eye[0] - was[0]).powi(2) + (eye[1] - was[1]).powi(2) + (eye[2] - was[2]).powi(2))
                .sqrt();
        was = eye;
    }
    let ended_at = eye_at(-z.down_limit).1;
    assert!(
        travelled > z.sphere * 0.15,
        "the eye only travelled {travelled:.2} m around a {:.2} m sphere across the floor zone",
        z.sphere
    );
    assert!(
        ended_at - started_at > 0.2,
        "the view barely panned: the feet went from {:.0}% to {:.0}%",
        started_at * 100.0,
        ended_at * 100.0
    );
}

#[test]
fn the_camera_never_jumps_as_the_aim_sweeps() {
    // The failure a solved camera is prone to, and the one a player would
    // notice instantly: two roots, a clamp releasing, or a condition going from
    // impossible to possible, and the eye teleports across the arena between
    // one frame and the next.
    //
    // Swept finely, and against the eye rather than the framing, because the
    // framing can stay put while the camera swings around behind it.
    let z = zones();
    let steps = 400;
    let mut previous: Option<[f32; 3]> = None;
    for step in 0..=steps {
        let pitch = -z.down_limit + step as f32 / steps as f32 * (z.down_limit + z.up_limit);
        let (f, at) = settled(pitch);
        let eye = [f.eye[0] - at[0], f.eye[1] - at[1], f.eye[2] - at[2]];
        if let Some(was) = previous {
            let moved =
                ((eye[0] - was[0]).powi(2) + (eye[1] - was[1]).powi(2) + (eye[2] - was[2]).powi(2))
                    .sqrt();
            let per_degree = moved / (z.down_limit + z.up_limit).to_degrees() * steps as f32;
            // As a share of the sphere rather than in metres, because the eye
            // slides along that sphere and a bigger one covers more ground for
            // the same change of angle. The fastest the rig moves on purpose is
            // the handover into the fighter's head, at about a tenth of the
            // radius per degree; a clamp releasing used to manage most of it in
            // a single degree.
            assert!(
                per_degree < 0.2 * z.sphere,
                "the eye jumped {per_degree:.2} m per degree at {:.1} degrees",
                pitch.to_degrees()
            );
        }
        previous = Some(eye);
    }
}

#[test]
fn the_floor_zone_walks_the_fighter_up_the_screen() {
    // Below the neutral zone the fighter climbs toward the crosshair, so that
    // at the bottom of the range the camera is looking at their own feet --
    // which is the shot that puts a stone underneath you.
    //
    // Two things do this at once, and it takes both. The eye keeps working
    // around the sphere, and the view pans -- the camera is pointed at the
    // crosshair's mark, which is itself sweeping onto the fighter's feet. The
    // neutral zone below has only the first of those; this zone adds the
    // second, which is what brings the eye back down off the top of the sphere
    // as the fighter comes to the middle of the frame.
    let z = zones();
    let mut last = f32::MIN;
    for step in 0..=8 {
        let pitch = -ramp(z.floor_from, z.down_limit, step as f32 / 8.0);
        let (f, at) = settled(pitch);
        let feet = on_screen(f, feet_of(at));
        assert!(
            feet > last - 0.01,
            "the fighter went back down the screen at {:.0} degrees",
            pitch.to_degrees()
        );
        last = feet;
    }
    assert!(
        last > z.feet_neutral + (z.feet_floor - z.feet_neutral) * 0.5,
        "at the bottom of the range the feet only reached {:.1}%, from {:.1}% toward {:.1}%",
        last * 100.0,
        z.feet_neutral * 100.0,
        z.feet_floor * 100.0
    );
}

#[test]
fn the_crosshair_rides_just_above_the_head_at_level() {
    // Aimed level there is no ground under the crosshair to read it against, so
    // the fighter's own head becomes the reference: keeping it just below the
    // mark is what makes a mid-range skillshot look like it is going where it
    // is going.
    let z = zones();
    let (f, at) = settled(0.0);
    let head = on_screen(f, head_of(at));
    assert!(
        (head - (0.5 - z.head_gap)).abs() < 0.03,
        "at level the head is at {:.1}%, not the {:.1}% just under the crosshair",
        head * 100.0,
        (0.5 - z.head_gap) * 100.0
    );
}

#[test]
fn aiming_up_hands_over_to_the_fighters_own_eyes() {
    // Above the horizon the camera walks into the fighter and is theirs by the
    // handover angle, so a skillshot aimed at the sky follows the line the
    // crosshair draws rather than one parallel to it.
    let z = zones();
    assert_eq!(
        settled(0.0).0.first_person,
        0.0,
        "the handover had already started at the horizon"
    );
    let mut last = 0.0;
    for step in 1..=8 {
        let now = settled(step as f32 / 8.0 * z.head_lock).0.first_person;
        assert!(
            now >= last - 0.001,
            "the handover went backwards at step {step}"
        );
        last = now;
    }
    assert_eq!(last, 1.0, "the eye never arrived at the fighter");
    assert_eq!(
        settled(z.up_limit).0.first_person,
        1.0,
        "the eye left the fighter again further up"
    );
}

#[test]
fn the_handed_over_eye_is_on_the_line_the_ability_travels() {
    // The reason that zone exists. Past the handover the sphere has shrunk onto
    // the fighter's own head and stays there, so a shot aimed at the sky
    // follows the line the crosshair draws rather than one parallel to it.
    //
    // The head, and not the point abilities actually come out of: those are a
    // little lower, and the rig is deliberately not splitting the difference.
    let z = zones();
    let (f, at) = settled(z.head_lock * 1.5);
    let head = sim::tuning::body_height().to_f32_for_render();
    let off = ((f.eye[0] - at[0]).powi(2)
        + (f.eye[1] - at[1] - head).powi(2)
        + (f.eye[2] - at[2]).powi(2))
    .sqrt();
    assert!(
        off <= z.head_sphere + 0.01,
        "the eye is {off:.2} m from the fighter's head, past the {:.2} m sphere it should be on",
        z.head_sphere
    );
}

#[test]
fn the_look_stops_short_of_straight_up_and_straight_down() {
    // At the pole the fighter's own vertical plane stops being defined and the
    // camera has nothing left to be behind.
    let z = zones();
    let quarter = std::f32::consts::FRAC_PI_2;
    assert!(
        z.down_limit < quarter && z.up_limit < quarter,
        "the limits reach the pole: {:.2} / {:.2}",
        z.down_limit,
        z.up_limit
    );
    for pitch in [-10.0f32, 10.0] {
        let mut rig = CameraRig::new(RigConfig::default());
        let at = [0.0, 0.0, 8.0];
        let f = rig.update(0.016, at, 0.0, pitch);
        assert!(f.eye[1] > 0.0, "a wild mouse put the eye underground");
        assert!(
            f.eye.iter().all(|v| v.is_finite()),
            "a wild mouse produced {:?}",
            f.eye
        );
    }
}

#[test]
fn the_camera_never_ends_up_under_the_floor() {
    let z = zones();
    for step in 0..=40 {
        let pitch = -z.down_limit + step as f32 / 40.0 * (z.down_limit + z.up_limit);
        let f = settled(pitch).0;
        assert!(f.eye[1] > 0.0, "eye underground at pitch {pitch:.2}");
    }
}

// ---------------------------------------------------------------------------
// Posing -- the property that matters is purity
// ---------------------------------------------------------------------------

fn input_at(action: Action, stride: f32, frame: u32) -> PoseInput {
    PoseInput {
        class: sim::Class::Bulwark,
        action,
        grounded: true,
        crouching: false,
        crouched_for: 0,
        speed: 0.0,
        travel: [0.0, 0.0],
        eased_speed: 0.0,
        eased_travel: [0.0, 0.0],
        stride,
        air_frames: 0,
        since_landed: frame as u16,
        parried: 0,
        stun_total: 0,
        rise: 0.0,
        turn_rate: 0.0,
        aim_pitch: 0.0,
        health: 1000,
        round_left: None,
        sim_frame: frame,
        bind_pose: false,
    }
}

#[test]
fn posing_is_a_pure_function_of_state() {
    // The whole rollback-safe animation argument rests on this.
    let cases = [
        input_at(Action::Free, 0.0, 41),
        input_at(Action::Startup { kind: 0, left: 2 }, 0.0, 41),
        input_at(Action::Active { kind: 1, left: 1 }, 0.0, 41),
        input_at(Action::Guard { held: 3 }, 0.0, 41),
        input_at(Action::HitStun { left: 4 }, 0.0, 41),
    ];
    for c in cases {
        assert_eq!(pose_for(c), pose_for(c));
    }
}

#[test]
fn replaying_a_frame_reproduces_its_pose() {
    // Simulate past a frame, then come back to it the way a rollback would.
    let mut w = World::new();
    let script: Vec<_> = (0..40u32)
        .map(|i| {
            [
                Input::new(if i % 9 == 0 { Input::LEFT } else { Input::W }),
                Input::default(),
            ]
        })
        .collect();
    for i in &script[..20] {
        w.advance(*i);
    }
    let snapshot = w.clone();
    let at_20 = pose_of(&w);
    for i in &script[20..] {
        w.advance(*i);
    }
    let mut replay = snapshot;
    assert_eq!(pose_of(&replay), at_20);
    for i in &script[20..] {
        replay.advance(*i);
    }
    assert_eq!(pose_of(&replay), pose_of(&w));
}

/// Pose the way the renderer does: from the interpolated view of two identical
/// snapshots, so every animation clock in the snapshot is exercised.
fn pose_of(w: &World) -> view::Pose {
    let frame = view::interpolate(w, w, 0.0);
    pose_for(PoseInput::of(&frame.players[0], w.players[0].class, &frame))
}

#[test]
fn walking_moves_the_legs_and_idling_does_not() {
    // And it has to be *distance* that moves them, not time: a walk cycle on a
    // fixed cadence skates the moment the body moves at any other speed.
    let mut moving = input_at(Action::Free, 0.0, 12);
    moving.speed = 7.0;
    moving.travel = [0.0, 7.0];
    moving.eased_speed = 7.0;
    moving.eased_travel = [0.0, 7.0];
    let leg = |p: &view::Pose| p.degrees(Joint::ThighL, 0);

    let mut swung: f32 = 0.0;
    for i in 0..20 {
        let mut at = moving;
        at.stride = i as f32 * 0.06;
        swung = swung.max((leg(&pose_for(at)) - leg(&pose_for(moving))).abs());
    }
    assert!(
        swung > 8.0,
        "legs did not swing over a stride: {swung} degrees"
    );

    let idle = input_at(Action::Free, 0.0, 12);
    let mut idle_swing: f32 = 0.0;
    for i in 0..20 {
        let mut at = idle;
        at.stride = i as f32 * 0.06;
        idle_swing = idle_swing.max((leg(&pose_for(at)) - leg(&pose_for(idle))).abs());
    }
    assert!(idle_swing < 1.0, "a standing character is striding");
}

#[test]
fn camera_pulls_in_rather_than_sitting_inside_a_platform() {
    // The arena has platforms around x = +/-7. Put the fight beside one so the
    // camera arm has to pass through it, and check the camera does not end up
    // buried in geometry.
    let mut rig = CameraRig::new(RigConfig::default());
    // Beside the left-hand platform, not on it: looking away from the platform
    // swings the camera arm straight through it.
    let spot = [-4.0, 0.0, 0.0];
    // Let the smoothed focus settle, then sweep the whole circle: the player
    // can look anywhere, so every angle has to be safe, not just the easy ones.
    for _ in 0..200 {
        rig.update(0.016, spot, 0.0, 0.0);
    }
    for step in 0..64 {
        let yaw = step as f32 / 64.0 * std::f32::consts::TAU;
        for pitch in [-0.8, 0.0, 0.8] {
            let framing = rig.update(0.016, spot, yaw, pitch);
            let inside = sim::arena::SOLIDS.iter().any(|s| {
                (0..3).all(|i| {
                    let lo = [s.min.x, s.min.y, s.min.z][i].to_f32_for_render();
                    let hi = [s.max.x, s.max.y, s.max.z][i].to_f32_for_render();
                    framing.eye[i] > lo && framing.eye[i] < hi
                })
            });
            assert!(
                !inside,
                "camera inside geometry at yaw {yaw:.2} pitch {pitch}: {:?}",
                framing.eye
            );
            assert!(
                framing.eye[1] > 0.0,
                "camera underground at yaw {yaw:.2} pitch {pitch}: {:?}",
                framing.eye
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Playing a whole match through the animation system
// ---------------------------------------------------------------------------

/// Run a scripted match and hand every frame to the poser the way the renderer
/// does. Returns the pose drawn on each frame for player one.
fn play_through(frames: u32) -> Vec<view::Pose> {
    let mut w = World::with_classes([sim::Class::Champion, sim::Class::Bulwark]);
    let mut fade = view::play::Crossfade::default();
    let mut drawn = Vec::new();
    for i in 0..frames {
        // A script that visits every branch of the selection: walking, running,
        // turning, jumping, dodging, attacking, guarding, being hit.
        let phase = i % 120;
        let bits = match phase {
            0..=20 => Input::W,
            21..=32 => Input::W | Input::A,
            33..=40 => Input::SPACE,
            41..=48 => Input::SHIFT | Input::D,
            49..=60 => Input::LEFT,
            61..=72 => Input::SHIFT | Input::LEFT,
            73..=86 => Input::RIGHT,
            87..=96 => Input::CROUCH,
            97..=104 => Input::S,
            _ => 0,
        };
        let aim = ((i * 700) % 65536) as u16;
        let prev = w.clone();
        w.advance([Input::aimed(bits, aim), Input::new(Input::RIGHT)]);
        let frame = view::interpolate(&prev, &w, 1.0);
        let input = view::play::PoseInput::of(&frame.players[0], w.players[0].class, &frame);
        drawn.push(fade.pose(input, 1.0 / 60.0));
    }
    drawn
}

#[test]
fn nothing_the_animation_system_draws_is_a_jump() {
    // Clips are switched, not cross-faded, inside `pose_for`; the fade on top is
    // what stops walking into a wind-up from being a visible cut. This is the
    // test that says it works, over a match that visits every branch.
    let drawn = play_through(600);
    let skeleton = view::pose::reference();
    let mut worst = 0.0f32;
    let mut at = (0usize, "");
    for (i, pair) in drawn.windows(2).enumerate() {
        let before = view::skeleton::solve(skeleton, &pair[0]);
        let after = view::skeleton::solve(skeleton, &pair[1]);
        // Relative to the hips, like the clip standard in `anim`: a body that
        // is travelling moves every joint on it, and that is the character
        // going somewhere rather than the pose jumping.
        let root = {
            let (p, q) = (before.origin[0], after.origin[0]);
            [q[0] - p[0], q[1] - p[1], q[2] - p[2]]
        };
        for j in view::skeleton::JOINTS {
            let (p, q) = (before.origin[j.index()], after.origin[j.index()]);
            let moved = ((q[0] - p[0] - root[0]).powi(2)
                + (q[1] - p[1] - root[1]).powi(2)
                + (q[2] - p[2] - root[2]).powi(2))
            .sqrt();
            if moved > worst {
                worst = moved;
                at = (i, j.name());
            }
        }
    }
    // Comfortably above what any single clip is allowed and far below a cut:
    // switching from a sprint into a wind-up without a fade moves a hand about
    // a metre in one frame, and that is the failure this exists to catch.
    assert!(
        worst < 0.45,
        "{} moved {worst:.3} m relative to the hips between frames {} and {}",
        at.1,
        at.0,
        at.0 + 1
    );
}

#[test]
fn nothing_the_animation_system_draws_is_broken() {
    let skeleton = view::pose::reference();
    for (i, pose) in play_through(600).iter().enumerate() {
        for (c, v) in pose.channels.iter().enumerate() {
            assert!(v.is_finite(), "frame {i} channel {c} is {v}");
        }
        let bad = pose.violations(skeleton);
        assert!(bad.is_empty(), "frame {i}: {bad:?}");
    }
}

#[test]
fn the_stride_phase_blends_the_short_way_round() {
    // The phase wraps, so a tick that crosses the wrap is a small step forward
    // rather than a large one backwards -- and getting that wrong makes the
    // legs snap backwards once per stride.
    let mut w = World::new();
    // Walk for long enough to cross the wrap a few times.
    let mut last = 0.0f32;
    let mut worst = 0.0f32;
    for i in 0..400 {
        let prev = w.clone();
        w.advance([Input::new(Input::W), Input::default()]);
        for step in 0..4 {
            let alpha = step as f32 / 4.0;
            let frame = view::interpolate(&prev, &w, alpha);
            let phase = frame.players[0].stride;
            assert!((0.0..1.0).contains(&phase), "phase {phase} out of range");
            if i > 2 {
                let step = (phase - last).rem_euclid(1.0);
                worst = worst.max(step);
            }
            last = phase;
        }
    }
    // A quarter of a tick at seven metres per second over a 2.7 m stride is
    // about a hundredth of a cycle. Anything near a whole cycle is a wrap
    // handled the long way round.
    assert!(worst < 0.1, "the phase jumped {worst:.3} of a cycle");
}

// ---------------------------------------------------------------------------
// The camera, with a creature in the arena
//
// It gets the two rules the rig already has rather than a third: geometry when
// you are beside it, a surface when you are on it. Both halves are visible
// enough to be worth pinning -- the first as a camera inside a ribcage, the
// second as a camera jammed against the rider's back.
// ---------------------------------------------------------------------------

fn a_creature_at(x: f32, yaw_turns: f32) -> sim::Monster {
    let mut beast = sim::Monster::new();
    beast.pos = sim::V3::new(
        sim::Fx::ratio((x * 100.0) as i32, 100),
        sim::Fx::ZERO,
        sim::Fx::ZERO,
    );
    beast.yaw = sim::Fx::ratio((yaw_turns * 1000.0) as i32, 1000);
    beast
}

#[test]
fn the_camera_never_ends_up_inside_the_creature() {
    let beast = a_creature_at(0.0, 0.0);
    let mut rig = CameraRig::new(RigConfig::default());
    // Beside it, at arm's length -- the case where the animal is between the
    // eye and the fighter for most of a turn.
    let spot = [0.0, 0.0, 3.0];
    for _ in 0..40 {
        rig.update_around(
            0.016,
            spot,
            0.0,
            0.0,
            view::Surroundings {
                beast: Some(&beast),
                aboard: false,
            },
        );
    }
    for step in 0..48 {
        let yaw = step as f32 / 48.0 * std::f32::consts::TAU;
        let framing = rig.update_around(
            0.016,
            spot,
            yaw,
            0.0,
            view::Surroundings {
                beast: Some(&beast),
                aboard: false,
            },
        );
        let eye = sim::V3::new(
            sim::Fx::ratio((framing.eye[0] * 1000.0) as i32, 1000),
            sim::Fx::ratio((framing.eye[1] * 1000.0) as i32, 1000),
            sim::Fx::ratio((framing.eye[2] * 1000.0) as i32, 1000),
        );
        assert!(
            !beast.contains(eye, sim::Fx::ZERO),
            "camera inside the creature at yaw {yaw:.2}: {:?}",
            framing.eye
        );
    }
}

#[test]
fn riding_does_not_jam_the_camera_against_your_own_back() {
    // An arm pointing backwards from someone standing on an animal goes
    // straight into the animal. Treated as geometry it clamps to nothing and
    // the player is left looking at the back of their own head.
    let beast = a_creature_at(0.0, 0.0);
    let back = sim::monster::shape(sim::monster::BARREL)
        .max
        .y
        .to_f32_for_render();
    let spot = [0.0, back, 0.0];
    let mut rig = CameraRig::new(RigConfig::default());
    let mut framing = rig.update_around(
        0.016,
        spot,
        0.0,
        0.0,
        view::Surroundings {
            beast: Some(&beast),
            aboard: true,
        },
    );
    for _ in 0..60 {
        framing = rig.update_around(
            0.016,
            spot,
            0.0,
            0.0,
            view::Surroundings {
                beast: Some(&beast),
                aboard: true,
            },
        );
    }
    // Against the same shot with nothing underfoot, rather than against a bare
    // number: what matters is that the creature did not shorten the arm, and
    // how long the arm is in the first place is the sphere's business.
    let mut alone = CameraRig::new(RigConfig::default());
    let mut loose = alone.update(0.016, spot, 0.0, 0.0);
    for _ in 0..60 {
        loose = alone.update(0.016, spot, 0.0, 0.0);
    }
    let reach = |f: view::camera::Framing| {
        ((f.eye[0] - spot[0]).powi(2) + (f.eye[2] - spot[2]).powi(2)).sqrt()
    };
    let (arm, free) = (reach(framing), reach(loose));
    assert!(
        arm > free * 0.9,
        "the arm collapsed from {free:.2} m to {arm:.2} m while riding -- the creature \
         is being treated as something to dodge rather than something to stand on"
    );
    assert!(
        framing.eye[1] > back,
        "the eye is at {:.2} m, below the back it is looking along at {back:.2}",
        framing.eye[1]
    );
}

#[test]
fn the_body_is_simply_drawn_while_the_fighter_is_out_in_the_open() {
    // The fade is measured on the eye the camera actually ended up at, which is
    // what lets a wall behind the fighter take their body away -- and is also
    // what would let it wink out for no reason if the two distances were tuned
    // against each other badly. Aimed where the rig rests, in the open, the
    // fighter is simply drawn.
    let f = settled(zones().neutral_pitch()).0;
    assert_eq!(
        f.hidden,
        0.0,
        "the fighter was {:.0}% faded away while standing in the open",
        f.hidden * 100.0
    );
}

#[test]
fn only_a_close_eye_ever_takes_the_whole_body() {
    // The rule, swept across the entire look range. There are two reasons to
    // stop drawing the fighter you are driving and only one of them is allowed
    // to finish the job: the crosshair reason **dims**, and the eye being close
    // is what makes somebody disappear.
    //
    // Why it matters is a spacing argument rather than a rendering one. A body
    // that winks out while the camera is still a full arm behind it takes its
    // own position with it, and where you are standing is what every decision
    // about range is measured from -- how far the other fighter is, whether you
    // are inside your own reach, which way you would dodge.
    let z = zones();
    let mut seen_dim = false;
    let mut seen_gone = false;
    for step in 0..=120 {
        let pitch = -z.down_limit + (z.down_limit + z.up_limit) * step as f32 / 120.0;
        let (f, at) = settled(pitch);
        let body = sim::tuning::body_height().to_f32_for_render();
        let middle = [at[0], at[1] + body * 0.5, at[2]];
        let reach = (0..3)
            .map(|i| (f.eye[i] - middle[i]).powi(2))
            .sum::<f32>()
            .sqrt();
        if reach > z.fade_near {
            assert!(
                f.hidden < 1.0,
                "at {:.0} deg the eye is {reach:.2} m out and the body is gone \
                 anyway -- only being close is supposed to do that",
                pitch.to_degrees()
            );
            assert!(
                f.hidden <= z.crosshair_dim + 0.001,
                "at {:.0} deg the body is {:.0}% faded, past the {:.0}% the \
                 crosshair reason is allowed",
                pitch.to_degrees(),
                f.hidden * 100.0,
                z.crosshair_dim * 100.0
            );
        }
        seen_dim |= f.hidden > 0.01 && f.hidden < 1.0;
        seen_gone |= f.hidden >= 1.0;
    }
    // Both halves have to actually happen in the sweep, or the assertion above
    // is being satisfied by a fade that never fires at all.
    assert!(seen_dim, "the body was never dimmed anywhere in the range");
    assert!(
        seen_gone,
        "the body never went away, so looking up no longer hands over to first \
         person -- see `Framing::hidden`"
    );
}

#[test]
fn the_eye_never_changes_pace_abruptly_at_a_zone_boundary() {
    // The camera being continuous is not the same as the camera being smooth.
    // Every version of this rig has held the eye's *position* together across a
    // boundary; what gave them away was the eye's *speed*, which arrived at a
    // boundary moving one way and left moving another. That is not seen so much
    // as felt -- the camera reads as changing its mind -- and it is what the
    // per-zone easing is for.
    //
    // Measured as the change in speed from one step to the next, against the
    // change within a zone at the same sampling. A boundary is not allowed to be
    // meaningfully worse than the ordinary motion either side of it.
    let z = zones();
    let steps = 1800;
    let sweep = z.down_limit + z.up_limit;
    let eye_at = |pitch: f32| {
        let (f, at) = settled(pitch);
        [f.eye[0] - at[0], f.eye[1] - at[1], f.eye[2] - at[2]]
    };
    let apart = |a: [f32; 3], b: [f32; 3]| {
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
    };

    let mut speeds = Vec::with_capacity(steps);
    let mut was = eye_at(-z.down_limit);
    for step in 1..=steps {
        let pitch = -z.down_limit + step as f32 / steps as f32 * sweep;
        let now = eye_at(pitch);
        speeds.push((pitch, apart(now, was)));
        was = now;
    }

    // What an ordinary step of the mouse does to the pace, away from any
    // boundary, with the top few discarded so that one noisy sample from the
    // simulation's fixed point does not set the bar.
    let mut ordinary: Vec<f32> = speeds
        .windows(2)
        .filter(|w| {
            [-z.floor_from, -z.neutral_to, 0.0, z.head_lock]
                .iter()
                .all(|b| (w[1].0 - b).abs() > 0.05)
        })
        .map(|w| (w[1].1 - w[0].1).abs())
        .collect();
    ordinary.sort_by(f32::total_cmp);
    let usual = ordinary[ordinary.len() * 99 / 100];

    for boundary in [-z.floor_from, -z.neutral_to, 0.0, z.head_lock] {
        let worst = speeds
            .windows(2)
            .filter(|w| (w[1].0 - boundary).abs() <= 0.05)
            .map(|w| (w[1].1 - w[0].1).abs())
            .fold(0.0f32, f32::max);
        assert!(
            worst <= usual * 3.0,
            "crossing {:.0} degrees changed the eye's pace by {worst:.5} m in one step, \
             against {usual:.5} for an ordinary step -- the zones are being swapped rather \
             than handed over",
            boundary.to_degrees()
        );
    }
}

// ---------------------------------------------------------------------------
// Aiming a shot that is a line
// ---------------------------------------------------------------------------

/// The Elementalist mid-shot, aimed `pitch` degrees above the horizon.
fn shooting(pitch: f32) -> PoseInput {
    PoseInput {
        class: sim::Class::Elementalist,
        action: Action::Active { kind: 0, left: 1 },
        aim_pitch: pitch.to_radians(),
        ..input_at(Action::Active { kind: 0, left: 1 }, 0.0, 40)
    }
}

/// Where a joint ends up once the whole pose has been solved.
fn joint_at(input: PoseInput, joint: Joint) -> [f32; 3] {
    let skeleton = view::skeleton::skeleton_for(input.class);
    view::skeleton::solve(&skeleton, &pose_for(input)).origin[joint.index()]
}

#[test]
fn a_shot_aimed_up_is_thrown_up() {
    // The animation has to agree with the shot. The Elementalist's auto is a
    // ray along the crosshair, so a shot forty degrees above the horizon that
    // is animated as a level flick has the character pointing one way while
    // the attack goes another -- which is precisely the complaint the whole
    // rework answers, and it is not fixed until the body shows it.
    let level = joint_at(shooting(0.0), Joint::HandR);
    let high = joint_at(shooting(40.0), Joint::HandR);
    let low = joint_at(shooting(-40.0), Joint::HandR);

    assert!(
        high[1] > level[1] + 0.08,
        "aiming up moved the throwing hand from {:.2} m to {:.2} m -- not enough to read",
        level[1],
        high[1]
    );
    assert!(
        low[1] < level[1] - 0.08,
        "aiming down moved the throwing hand from {:.2} m to {:.2} m",
        level[1],
        low[1]
    );

    // And the head goes with it, or she is shooting at something she is not
    // looking at.
    let head_level = joint_at(shooting(0.0), Joint::Head);
    let head_high = joint_at(shooting(40.0), Joint::Head);
    assert!(
        head_high[1] >= head_level[1] - 0.01,
        "the head dropped while the aim went up"
    );
}

#[test]
fn a_swing_is_drawn_at_the_angle_it_comes_out_at() {
    // Melee is tilted too, because melee happens in the air and on slopes. The
    // *dead zone* that keeps it level while you look slightly down at somebody
    // is the simulation's job and is tested there; by the time the pose sees
    // the angle it is the one the attack actually came out at, so the body
    // follows it.
    let swinging = |pitch: f32| PoseInput {
        aim_pitch: pitch.to_radians(),
        ..input_at(Action::Active { kind: 0, left: 1 }, 0.0, 40)
    };
    let level = joint_at(swinging(0.0), Joint::HandR);
    let high = joint_at(swinging(40.0), Joint::HandR);
    assert!(
        high[1] > level[1] + 0.08,
        "a swing aimed forty degrees up drew its hand at {:.2} m against {:.2} m level",
        high[1],
        level[1]
    );
}

#[test]
fn a_grounded_cast_is_not_tilted_by_the_aim() {
    // The other half of the rule. A pillar of flame comes out of the floor
    // wherever the crosshair put it, and the caster is gesturing at the place
    // rather than throwing anything along a line -- so looking down to place
    // one must not double her over.
    let casting = |pitch: f32| PoseInput {
        class: sim::Class::Elementalist,
        aim_pitch: pitch.to_radians(),
        ..input_at(Action::Active { kind: 2, left: 1 }, 0.0, 40)
    };
    assert_eq!(
        joint_at(casting(0.0), Joint::HandR),
        joint_at(casting(-40.0), Joint::HandR),
        "the fire pillar's cast followed the camera down"
    );
}

#[test]
fn the_opening_pitch_is_inside_the_neutral_zone() {
    // Where a match opens is the first thing anybody sees, and on this rig it
    // decides more than which way the camera faces: the eye rides further out
    // the further below the horizon you look, so the opening angle is what
    // makes the difference between seeing your own fighter standing in an arena
    // and seeing a patch of floor with your own shield across it.
    //
    // Both boundaries matter and for different reasons. Past `floor_from` the
    // view starts tilting toward your own feet, so a match would open pointed
    // at the ground. Above `neutral_to` the sphere starts travelling up to the
    // head and the eye comes in, so a match would open too close to see
    // yourself. Between them there is room to steer either way without
    // crossing anything, which is the whole point of resting in a zone rather
    // than on an edge.
    let z = zones();
    let opening = -z.start_pitch().to_degrees();
    let (floor, neutral) = (z.floor_from.to_degrees(), z.neutral_to.to_degrees());

    assert!(
        opening < floor,
        "a match opens at {opening:.0} degrees below level, past the floor zone at \
         {floor:.0} -- the view is tilted at your own feet before anybody has moved"
    );
    assert!(
        opening > neutral,
        "a match opens at {opening:.0} degrees below level, above the neutral zone at \
         {neutral:.0} -- the eye has come in and you cannot see your own fighter"
    );

    // Not on either edge, either. A knob nudged one degree should not change
    // which zone the game starts in.
    let margin = (floor - neutral) * 0.15;
    assert!(
        opening < floor - margin && opening > neutral + margin,
        "a match opens at {opening:.0} degrees, within a nudge of a zone boundary \
         ({neutral:.0} to {floor:.0})"
    );
}

#[test]
fn a_match_opens_showing_the_fighter_rather_than_the_floor() {
    // The property the number is for, rather than the number. At the opening
    // angle the eye has to be far enough back that the fighter is in front of
    // it and drawn, which is what "third person" means and is not true at every
    // pitch this rig allows -- near level the eye is close enough that the body
    // is faded out of the way.
    let (f, at) = settled(zones().start_pitch());
    // Flat distance, not one axis: the eye goes wherever the yaw sends it.
    let back = (0..3)
        .step_by(2)
        .map(|i| (f.eye[i] - at[i]).powi(2))
        .sum::<f32>()
        .sqrt();
    assert!(
        back > 1.0,
        "the eye opens {back:.2} m from the fighter, which is not a view of them"
    );
    assert_eq!(
        f.hidden,
        0.0,
        "the fighter is {:.0}% faded away at the angle the match opens at",
        f.hidden * 100.0
    );
}
