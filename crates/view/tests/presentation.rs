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

#[test]
fn the_camera_sits_behind_the_fighter() {
    let mut rig = CameraRig::new(RigConfig::default());
    let f = rig.update(
        0.016,
        [2.0, 0.0, -1.0],
        0.0,
        0.0,
        aim_at([2.0, 0.0, -1.0], 0.0, 0.0),
    );
    // Looking down +X means the camera is at smaller X than the fighter.
    assert!(f.eye[0] < 2.0, "camera at {} is not behind", f.eye[0]);
    // And it aims past them, not at them, so they are not standing in front of
    // whatever you are trying to look at.
    assert!(
        f.look_at[0] > 2.0,
        "aim point at {} is not ahead of the fighter",
        f.look_at[0]
    );
    // And it is pointed *at the aim point* -- that is the whole contract, and
    // it is what pins the crosshair to the middle of the screen however far the
    // eye has been lifted off the ability's line.
    let target = aim_at([2.0, 0.0, -1.0], 0.0, 0.0);
    assert!(
        (f.look_at[0] - target[0]).abs() < 0.001
            && (f.look_at[1] - target[1]).abs() < 0.001
            && (f.look_at[2] - target[2]).abs() < 0.001,
        "the camera is looking at {:?}, not at the aim point {target:?}",
        f.look_at
    );
    assert!(f.eye[1] > 0.0, "camera is underground");
    // The eye sits above the fighter's head even looking level, which is what
    // keeps their body out of the middle of the frame. It is deliberately not
    // the height abilities come out of: the camera frames the fight, and
    // `sim::aim` decides where the shot goes.
    assert!(
        f.eye[1] > RigConfig::default().look_height + 0.5,
        "the eye at {} is barely above the cast origin at {}",
        f.eye[1],
        RigConfig::default().look_height
    );
}

#[test]
fn the_camera_and_the_simulation_agree_on_forward() {
    // The invariant that makes camera-relative movement work at all. If the
    // renderer's yaw convention and `move_dir`'s disagree, W walks sideways
    // and nothing about the code looks wrong. Both are checked against each
    // other here rather than each against itself.
    //
    // **Exactly**, and that is why the eye sits directly behind the fighter
    // rather than off one shoulder. The camera is pointed at the target rather
    // than along the look axis, so an eye slid sideways would turn the whole
    // view and W would stop walking up the screen. Straight behind, the only
    // parallax left is vertical, which costs pitch and not bearing.
    let cfg = RigConfig::default();
    for eighth in 0..8u32 {
        let aim = (eighth * (u16::MAX as u32 + 1) / 8) as u16;
        let radians = aim as f32 / 65536.0 * std::f32::consts::TAU;

        let mut rig = CameraRig::new(cfg);
        let (cx, cz) = aim_dir(
            rig.update(
                0.016,
                [0.0, 0.0, 0.0],
                radians,
                0.0,
                aim_at([0.0, 0.0, 0.0], radians, 0.0),
            ),
            [0.0, 0.0, 0.0],
        );

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
    // Directly behind is not a preference here, it is what keeps the bearing
    // exact once the camera is pointed at the aim point.
    let at = [0.0, 0.0, 8.0];
    for eighth in 0..8u32 {
        let yaw = eighth as f32 / 8.0 * std::f32::consts::TAU;
        let mut rig = CameraRig::new(RigConfig::default());
        let f = rig.update(0.016, at, yaw, 0.0, aim_at(at, yaw, 0.0));
        // Distance from the eye to the vertical plane the fighter looks along.
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
    // must be looking exactly where asked on the very frame it is asked --
    // checked against where it settles rather than against the raw axis, since
    // the shoulder leaves it a couple of degrees off that by design.
    let at = [0.0, 0.0, 0.0];
    let yaw = std::f32::consts::FRAC_PI_2;
    let mut snapped = CameraRig::new(RigConfig::default());
    for _ in 0..60 {
        snapped.update(0.016, at, 0.0, 0.0, aim_at(at, 0.0, 0.0));
    }
    let turned = snapped.update(0.016, at, yaw, 0.0, aim_at(at, yaw, 0.0));

    let mut held = CameraRig::new(RigConfig::default());
    for _ in 0..60 {
        held.update(0.016, at, yaw, 0.0, aim_at(at, yaw, 0.0));
    }
    let settled = held.update(0.016, at, yaw, 0.0, aim_at(at, yaw, 0.0));

    let (ax, az) = aim_dir(turned, at);
    let (bx, bz) = aim_dir(settled, at);
    assert!(
        (ax - bx).abs() < 0.01 && (az - bz).abs() < 0.01,
        "camera lagged the mouse: ({ax:.3}, {az:.3}) against ({bx:.3}, {bz:.3})"
    );
}

/// Flat distance from the focus point to the eye.
fn flat_arm(f: view::camera::Framing, focus: [f32; 3]) -> f32 {
    let (dx, dz) = (f.eye[0] - focus[0], f.eye[2] - focus[2]);
    (dx * dx + dz * dz).sqrt()
}

/// Where the middle of the screen meets the floor, as a distance in front of
/// the fighter. Negative means behind them.
///
/// This is the number the player is actually steering when they aim at the
/// ground, so it is the number the assertions are written in.
fn ground_mark(f: view::camera::Framing, at: [f32; 3]) -> Option<f32> {
    let dir = [
        f.look_at[0] - f.eye[0],
        f.look_at[1] - f.eye[1],
        f.look_at[2] - f.eye[2],
    ];
    let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    let dir = [dir[0] / len, dir[1] / len, dir[2] / len];
    if dir[1] > -1e-3 {
        return None; // looking at or above the horizon: it never lands
    }
    let travel = f.eye[1] / -dir[1];
    let x = f.eye[0] + dir[0] * travel;
    let z = f.eye[2] + dir[2] * travel;
    // Signed along the way the camera faces, flat.
    let flat = (dir[0] * dir[0] + dir[2] * dir[2]).sqrt().max(1e-4);
    Some((x - at[0]) * dir[0] / flat + (z - at[2]) * dir[2] / flat)
}

/// Where the simulation says a fighter standing here and looking this way is
/// aiming. The game hands the rig exactly this, so the fixture has to as well.
fn aim_at(player: [f32; 3], yaw: f32, pitch: f32) -> [f32; 3] {
    // The game clamps as it reads the mouse, so the simulation and the rig
    // always see the same angle. A fixture that did not would be aiming
    // somewhere the camera cannot look.
    let cfg = RigConfig::default();
    let pitch = pitch.clamp(-cfg.pitch_down, cfg.pitch_up);
    let look = sim::Input::looking_at(
        0,
        view::aim_from_radians(yaw),
        view::pitch_from_radians(pitch),
    );
    let from = sim::aim::origin(sim::V3::new(
        sim::Fx::from_raw((player[0] * 65536.0) as i32),
        sim::Fx::from_raw((player[1] * 65536.0) as i32),
        sim::Fx::from_raw((player[2] * 65536.0) as i32),
    ));
    let stones = [None; sim::stones::MAX_STONES];
    let far = sim::Fx::from_int(60);
    let dir = look.look_dir();
    let reach = sim::aim::trace(from, dir, far, &stones).unwrap_or(far);
    let at = from.add(dir.scale(reach));
    [
        at.x.to_f32_for_render(),
        at.y.to_f32_for_render(),
        at.z.to_f32_for_render(),
    ]
}

fn settled(pitch: f32) -> (view::camera::Framing, [f32; 3]) {
    let mut rig = CameraRig::new(RigConfig::default());
    // Past the platforms in z, which reach four metres either side of the
    // middle. The arm is eleven metres now, so a fixture in the middle of the
    // arena sweeps the camera straight through one and the occlusion pull-in
    // would be what these tests were measuring.
    let at = [0.0, 0.0, 8.0];
    let target = aim_at(at, 0.0, pitch);
    for _ in 0..200 {
        rig.update(0.016, at, 0.0, pitch, target);
    }
    (rig.update(0.016, at, 0.0, pitch, target), at)
}

#[test]
fn looking_down_walks_the_aim_in_toward_your_own_feet() {
    // The thing that was impossible. The aim point used to be pinned flat a
    // fixed distance ahead, so pitching down changed how obliquely you saw the
    // same spot and nothing else -- you could not put the reticle near yourself
    // and you could not push it out.
    let cfg = RigConfig::default();
    let mut last = f32::INFINITY;
    for step in 1..=12 {
        let pitch = -(step as f32 / 12.0) * cfg.pitch_down;
        let (f, at) = settled(pitch);
        let mark = ground_mark(f, at).expect("looking down has to reach the floor");
        assert!(
            mark < last + 0.01,
            "at pitch {pitch:.2} the aim went back out to {mark:.2} from {last:.2}"
        );
        last = mark;
    }
    assert!(
        last < 1.5,
        "looking all the way down still aims {last:.2}m away; you cannot hit your own feet"
    );
}

#[test]
fn a_shallow_look_down_reaches_well_past_the_fighter() {
    // The other half: the range has to be worth steering. If every downward
    // angle lands within a couple of metres there is nothing to aim *with*.
    let cfg = RigConfig::default();
    let (near, at) = settled(-cfg.pitch_down);
    let (far, _) = settled(-cfg.pitch_down * 0.12);
    let near = ground_mark(near, at).expect("steep look misses the floor");
    let far = ground_mark(far, at).expect("shallow look misses the floor");
    assert!(
        far > near + 6.0,
        "the whole range of downward aim is {near:.1}m to {far:.1}m -- too little to steer"
    );
}

#[test]
fn zooming_out_does_not_move_where_you_are_aiming() {
    // The property the whole rig is built on. The camera orbits a point above
    // the fighter, so the mark on the ground is `orbit height / tan(pitch)` --
    // the arm length cancels. That makes distance a pure comfort setting: a
    // player who pulls the camera back to see more of the fight has not also
    // changed where their attacks are going.
    let mut cfg = RigConfig::default();
    let at = [0.0, 0.0, 8.0];
    let mut marks = Vec::new();
    let target = aim_at(at, 0.0, -RigConfig::default().neutral_pitch);
    for distance in [5.0, 8.0, 11.0, 15.0] {
        cfg.distance = distance;
        let mut rig = CameraRig::new(cfg);
        for _ in 0..200 {
            rig.update(0.016, at, 0.0, -cfg.neutral_pitch, target);
        }
        let f = rig.update(0.016, at, 0.0, -cfg.neutral_pitch, target);
        marks.push(ground_mark(f, at).expect("neutral pitch has to reach the floor"));
    }
    let spread = marks.iter().fold(f32::MIN, |a, b| a.max(*b))
        - marks.iter().fold(f32::MAX, |a, b| a.min(*b));
    assert!(
        spread < 0.05,
        "aim moved {spread:.2}m across the zoom range: {marks:?}"
    );
}

#[test]
fn looking_down_never_hauls_the_camera_in() {
    // Reported: it should not zoom toward the character as you pan down. The
    // aim comes in because the orbit centre drops, not because the view does.
    let cfg = RigConfig::default();
    let at = [0.0, 0.0, 8.0];
    let level = flat_arm(
        settled(-cfg.neutral_pitch).0,
        [at[0], RigConfig::default().look_height, at[2]],
    );
    for step in 1..=10 {
        let pitch = -(step as f32 / 10.0) * cfg.pitch_down;
        let (f, at) = settled(pitch);
        let flat = flat_arm(f, [at[0], RigConfig::default().look_height, at[2]]);
        // Pitching down shortens the *flat* arm by cosine alone -- the camera
        // rises over the fighter. What must not happen is the arm itself
        // getting shorter.
        let arm = (flat * flat + (f.eye[1] - RigConfig::default().look_height).powi(2)).sqrt();
        assert!(
            arm > cfg.distance * 0.9,
            "at pitch {pitch:.2} the arm is {arm:.2}, down from {:.2}",
            cfg.distance
        );
    }
    assert!(level > 9.0, "fixture arm was already short: {level:.2}");
}

#[test]
fn the_whole_fighter_is_in_frame_at_rest() {
    // Reported: the feet were chopped off. The whole fighter has to be inside
    // the frame at rest.
    //
    // They now *straddle* the crosshair rather than sitting under it: the rig
    // orbits the cast origin, which is chest height, so the feet are below the
    // middle of the screen and the head is above it. Both still have to be in
    // frame, and the feet are the tight one because the camera is pitched down.
    let cfg = RigConfig::default();
    let (f, at) = settled(-cfg.neutral_pitch);
    let eye = [f.eye[0], f.eye[1], f.eye[2]];
    let forward = {
        let d = [
            f.look_at[0] - eye[0],
            f.look_at[1] - eye[1],
            f.look_at[2] - eye[2],
        ];
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        [d[0] / len, d[1] / len, d[2] / len]
    };
    // Angle below screen centre of the fighter's feet and head.
    let angle_of = |point: [f32; 3]| {
        let to = [point[0] - eye[0], point[1] - eye[1], point[2] - eye[2]];
        let len = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
        let dot = (to[0] * forward[0] + to[1] * forward[1] + to[2] * forward[2]) / len;
        dot.clamp(-1.0, 1.0).acos()
    };
    let feet = angle_of([at[0], 0.0, at[2]]);
    let head = angle_of([at[0], 1.8, at[2]]);
    // Half of a 58-degree vertical field of view, with a little margin for the
    // HUD along the bottom.
    let half_frame = 58f32.to_radians() / 2.0 * 0.85;
    assert!(
        feet < half_frame,
        "the fighter's feet are {:.0} degrees off centre, past the {:.0} the frame holds",
        feet.to_degrees(),
        half_frame.to_degrees()
    );
    assert!(
        head < half_frame,
        "the fighter's head is {:.0} degrees off centre, past the {:.0} the frame holds",
        head.to_degrees(),
        half_frame.to_degrees()
    );
}

#[test]
fn aiming_above_the_horizon_hands_over_quickly() {
    // Reported, and the shape asked for: below the horizon it is third person;
    // above it the camera comes in *quickly* and the fighter fades out as it
    // goes. It used to start forty degrees up and finish at the pitch limit,
    // which left a wide band with the arm dragging along the floor behind the
    // fighter and every bump in the terrain shoving the view.
    let cfg = RigConfig::default();
    assert_eq!(
        settled(0.0).0.first_person,
        0.0,
        "the handover had already started at the horizon"
    );
    assert!(
        settled(-cfg.neutral_pitch).0.first_person == 0.0,
        "the handover had started at rest"
    );

    let mut last = 0.0;
    for step in 1..=10 {
        let pitch = step as f32 / 10.0 * cfg.sky_full;
        let now = settled(pitch).0.first_person;
        assert!(
            now >= last - 0.001,
            "the handover went backwards at {pitch:.2}"
        );
        last = now;
    }
    assert!(
        last > 0.99,
        "half a radian above the horizon the rig is only {last:.2} of the way in"
    );
}

#[test]
fn the_fighter_sits_low_at_rest_and_rises_to_the_middle_as_you_aim_up() {
    // Reported: the fighter was almost on the crosshair wherever you aimed, and
    // it felt cramped. They should be down in the lower part of the frame at
    // rest -- which is what the eye sitting well above their head buys -- and
    // come up to the middle as the aim goes over the horizon and they fade.
    let cfg = RigConfig::default();
    let (rest, at) = settled(-cfg.neutral_pitch);
    let resting = below_centre(rest, [at[0], 1.8, at[2]]);
    assert!(
        resting > 6.0,
        "at rest the fighter's head is only {resting:.1} degrees below the crosshair"
    );

    // Aim up and they come to the middle, and are most of the way faded out by
    // the time they get there. The path is not monotone by a few tenths of a
    // degree on the way -- the crosshair climbs before the camera closes in --
    // and pinning that would be pinning an accident rather than the shape.
    let (up, at) = settled(cfg.sky_full * 0.8);
    let arrived = below_centre(up, [at[0], 1.8, at[2]]);
    assert!(
        arrived < 2.0,
        "aiming up left the fighter {arrived:.1} degrees below the crosshair, from {resting:.1}"
    );
    assert!(
        up.first_person > 0.7,
        "the fighter reached the middle of the screen only {:.2} faded out",
        up.first_person
    );
}

/// How far below the middle of the screen a world point sits, in degrees.
fn below_centre(f: view::camera::Framing, point: [f32; 3]) -> f32 {
    let to = |p: [f32; 3]| {
        let v = [p[0] - f.eye[0], p[1] - f.eye[1], p[2] - f.eye[2]];
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-4);
        [v[0] / len, v[1] / len, v[2] / len]
    };
    let centre = to(f.look_at);
    let target = to(point);
    let dot: f32 = (0..3).map(|i| centre[i] * target[i]).sum();
    let angle = dot.clamp(-1.0, 1.0).acos().to_degrees();
    // Signed: below the crosshair is positive.
    if target[1] < centre[1] { angle } else { -angle }
}

#[test]
fn the_camera_never_ends_up_under_the_floor() {
    let cfg = RigConfig::default();
    for step in 0..=40 {
        let pitch = -cfg.pitch_down + step as f32 / 40.0 * (cfg.pitch_down + cfg.pitch_up);
        let f = settled(pitch).0;
        assert!(f.eye[1] > 0.0, "eye underground at pitch {pitch:.2}");
    }
}

#[test]
fn pitch_is_clamped() {
    let mut rig = CameraRig::new(RigConfig::default());
    let cfg = RigConfig::default();
    for pitch in [-10.0, 10.0] {
        let f = rig.update(
            0.016,
            [0.0, 0.0, 0.0],
            0.0,
            pitch,
            aim_at([0.0, 0.0, 0.0], 0.0, pitch),
        );
        let rise = f.look_at[1] - f.eye[1];
        let run = ((f.look_at[0] - f.eye[0]).powi(2) + (f.look_at[2] - f.eye[2]).powi(2)).sqrt();
        let got = rise.atan2(run);
        // The rendered angle is the clamped look pitch *plus the parallax* --
        // the camera is pointed at the target rather than along the axis, and
        // the eye sits above and beside that axis. A few degrees of slack for
        // that; what must not happen is a wild mouse producing a wild camera.
        const PARALLAX: f32 = 0.12;
        assert!(
            got >= -cfg.pitch_down - PARALLAX && got <= cfg.pitch_up + PARALLAX,
            "pitched to {got} past the {} / {} limits",
            cfg.pitch_down,
            cfg.pitch_up
        );
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
        rig.update(0.016, spot, 0.0, 0.0, aim_at(spot, 0.0, 0.0));
    }
    for step in 0..64 {
        let yaw = step as f32 / 64.0 * std::f32::consts::TAU;
        for pitch in [-0.8, 0.0, 0.8] {
            let framing = rig.update(0.016, spot, yaw, pitch, aim_at(spot, yaw, pitch));
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
            aim_at(spot, 0.0, 0.0),
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
            aim_at(spot, yaw, 0.0),
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
        aim_at(spot, 0.0, 0.0),
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
            aim_at(spot, 0.0, 0.0),
            view::Surroundings {
                beast: Some(&beast),
                aboard: true,
            },
        );
    }
    let arm = ((framing.eye[0] - spot[0]).powi(2) + (framing.eye[2] - spot[2]).powi(2)).sqrt();
    assert!(
        arm > 2.0,
        "the arm collapsed to {arm:.2} m while riding -- the creature is being \
         treated as something to dodge rather than something to stand on"
    );
    assert!(
        framing.eye[1] > back,
        "the eye is at {:.2} m, below the back it is looking along at {back:.2}",
        framing.eye[1]
    );
}
