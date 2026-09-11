//! Tests for the parts of presentation that must be correct rather than pretty.
//!
//! Everything here runs without a window, which is the point of `view` having
//! no engine dependency.

use sim::state::Action;
use sim::{Input, World};
use view::interp::{TickClock, interpolate};
use view::pose::{PARTS, Part, PoseInput, pose_for};
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

/// Flat direction from a fighter to the point at the centre of their screen.
///
/// This, and not the eye-to-target vector, is what "where you look" means: the
/// eye sits off to one shoulder, so the camera's own axis is deliberately not
/// the aim axis. What has to be true is that the crosshair sits straight ahead
/// of the fighter.
fn aim_dir(f: view::camera::Framing, player: [f32; 3]) -> (f32, f32) {
    let (dx, dz) = (f.look_at[0] - player[0], f.look_at[2] - player[2]);
    let len = (dx * dx + dz * dz).sqrt();
    (dx / len, dz / len)
}

#[test]
fn the_camera_sits_behind_the_fighter() {
    let mut rig = CameraRig::new(RigConfig::default());
    let cfg = RigConfig::default();
    let f = rig.update(0.016, [2.0, 0.0, -1.0], 0.0, 0.0);
    // Looking down +X means the camera is at smaller X than the fighter.
    assert!(f.eye[0] < 2.0, "camera at {} is not behind", f.eye[0]);
    // And it aims past them, not at them, so they are not standing in front of
    // whatever you are trying to look at.
    assert!(
        (f.look_at[0] - (2.0 + cfg.look_ahead)).abs() < 0.01,
        "aim point at {} is not ahead of the fighter",
        f.look_at[0]
    );
    assert!((f.look_at[2] - -1.0).abs() < 0.01, "aim drifted sideways");
    assert!(f.eye[1] > 0.0, "camera is underground");
    assert!(
        f.eye[1] > f.look_at[1],
        "camera should look down on the fight, not up at it"
    );
}

#[test]
fn the_camera_and_the_simulation_agree_on_forward() {
    // The invariant that makes camera-relative movement work at all. If the
    // renderer's yaw convention and `move_dir`'s disagree, W walks sideways
    // and nothing about the code looks wrong. Both are checked against each
    // other here rather than each against itself.
    for eighth in 0..8u32 {
        let aim = (eighth * (u16::MAX as u32 + 1) / 8) as u16;
        let radians = aim as f32 / 65536.0 * std::f32::consts::TAU;

        let mut rig = CameraRig::new(RigConfig::default());
        let (cx, cz) = aim_dir(
            rig.update(0.016, [0.0, 0.0, 0.0], radians, 0.0),
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
    // must be looking exactly where asked on the very frame it is asked.
    let mut rig = CameraRig::new(RigConfig::default());
    for _ in 0..60 {
        rig.update(0.016, [0.0, 0.0, 0.0], 0.0, 0.0);
    }
    let turned = rig.update(0.016, [0.0, 0.0, 0.0], std::f32::consts::FRAC_PI_2, 0.0);
    let (dx, dz) = aim_dir(turned, [0.0, 0.0, 0.0]);
    assert!(
        dx.abs() < 0.02 && (dz - 1.0).abs() < 0.02,
        "camera lagged the mouse: ({dx:.3}, {dz:.3})"
    );
}

/// Flat distance from the focus point to the eye.
fn flat_arm(f: view::camera::Framing, focus: [f32; 3]) -> f32 {
    let (dx, dz) = (f.eye[0] - focus[0], f.eye[2] - focus[2]);
    (dx * dx + dz * dz).sqrt()
}

#[test]
fn looking_up_walks_the_camera_down_to_the_ground() {
    // It used to hang at a fixed height and haul itself in toward the
    // fighter's head instead -- the floor clamp compared the arm against the
    // ground without counting the eye lift, so it fired about three metres
    // early and "solved" a collision that was not happening by shortening the
    // arm. It read as a pole under the camera.
    let mut rig = CameraRig::new(RigConfig::default());
    // Clear of the platforms, which start five metres out and would otherwise
    // trigger the occlusion pull-in and confuse what is being measured.
    let at = [3.0, 0.0, 0.0];
    for _ in 0..200 {
        rig.update(0.016, at, 0.0, 0.0);
    }

    let mut last = f32::INFINITY;
    let mut reached_the_ground = false;
    for step in 0..=20 {
        let pitch = step as f32 / 20.0 * RigConfig::default().pitch_limit;
        let h = rig.update(0.016, at, 0.0, pitch).eye[1];
        assert!(h <= last + 0.001, "camera rose at pitch {pitch:.2}");
        last = h;
        if (h - 0.3).abs() < 0.05 {
            reached_the_ground = true;
        }
    }
    assert!(
        reached_the_ground,
        "camera never got near the ground; lowest was {last}"
    );
}

#[test]
fn looking_up_does_not_lunge_the_camera_at_the_fighter() {
    // The camera may close on the fighter once it is *on* the ground and has
    // nowhere else to go. It may not do so on the way down.
    let mut rig = CameraRig::new(RigConfig::default());
    let at = [3.0, 0.0, 0.0];
    for _ in 0..200 {
        rig.update(0.016, at, 0.0, 0.0);
    }
    let focus = [at[0], RigConfig::default().look_height, at[2]];
    let level = flat_arm(rig.update(0.016, at, 0.0, 0.0), focus);

    // A modest look up, well before the camera can reach the floor.
    let tilted = flat_arm(rig.update(0.016, at, 0.0, 0.4), focus);
    assert!(
        tilted > level * 0.85,
        "a 23-degree look up pulled the camera from {level:.2} to {tilted:.2}"
    );
    assert!(
        rig.update(0.016, at, 0.0, 0.4).eye[1] > 0.5,
        "camera reached the ground far too early"
    );
}

#[test]
fn the_camera_rides_along_the_ground_rather_than_hovering_over_it() {
    let cfg = RigConfig::default();
    let mut rig = CameraRig::new(cfg);
    let at = [3.0, 0.0, 0.0];
    for _ in 0..200 {
        rig.update(0.016, at, 0.0, cfg.pitch_limit);
    }
    let f = rig.update(0.016, at, 0.0, cfg.pitch_limit);
    assert!(
        (f.eye[1] - 0.3).abs() < 0.02,
        "at full pitch the eye sits at {}, not on the ground",
        f.eye[1]
    );
}

#[test]
fn pitch_is_clamped() {
    let mut rig = CameraRig::new(RigConfig::default());
    let cfg = RigConfig::default();
    for pitch in [-10.0, 10.0] {
        let f = rig.update(0.016, [0.0, 0.0, 0.0], 0.0, pitch);
        let rise = f.look_at[1] - f.eye[1];
        let run = ((f.look_at[0] - f.eye[0]).powi(2) + (f.look_at[2] - f.eye[2]).powi(2)).sqrt();
        let got = rise.atan2(run);
        assert!(
            got.abs() <= cfg.pitch_limit + 0.01,
            "pitched to {got} past the {} limit",
            cfg.pitch_limit
        );
    }
}

// ---------------------------------------------------------------------------
// Posing -- the property that matters is purity
// ---------------------------------------------------------------------------

fn input_at(action: Action, into: u16, total: u16, frame: u32) -> PoseInput {
    PoseInput {
        action,
        frames_into: into,
        frames_total: total,
        speed: 0.0,
        grounded: true,
        crouching: false,
        sim_frame: frame,
        clip: None,
    }
}

#[test]
fn posing_is_a_pure_function_of_state() {
    // The whole rollback-safe animation argument rests on this.
    let cases = [
        input_at(Action::Free, 0, 0, 41),
        input_at(Action::Startup { kind: 0, left: 2 }, 2, 4, 41),
        input_at(Action::Active { kind: 1, left: 1 }, 3, 4, 41),
        input_at(Action::Guard { held: 3 }, 3, 0, 41),
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

fn pose_of(w: &World) -> view::Pose {
    let p = &w.players[0];
    pose_for(PoseInput {
        action: p.action,
        frames_into: 0,
        frames_total: 0,
        speed: 0.0,
        grounded: p.grounded,
        crouching: p.crouching,
        sim_frame: w.frame,
        clip: None,
    })
}

#[test]
fn attack_phases_are_visually_distinct() {
    // Reading startup from active from recovery across the arena is a gameplay
    // requirement, not an art one.
    let startup = pose_for(input_at(Action::Startup { kind: 0, left: 0 }, 4, 4, 0));
    let active = pose_for(input_at(Action::Active { kind: 0, left: 2 }, 0, 3, 0));
    let recovery = pose_for(input_at(Action::Recovery { kind: 0, left: 9 }, 9, 10, 0));

    let sep = |a: &view::Pose, b: &view::Pose| -> f32 {
        PARTS
            .iter()
            .map(|p| {
                let (x, y) = (a.get(*p), b.get(*p));
                (0..3)
                    .map(|k| (x.pos[k] - y.pos[k]).abs() + (x.rot[k] - y.rot[k]).abs())
                    .sum::<f32>()
            })
            .sum()
    };
    assert!(
        sep(&startup, &active) > 1.5,
        "startup and active look alike"
    );
    assert!(
        sep(&active, &recovery) > 1.5,
        "active and recovery look alike"
    );
}

#[test]
fn walking_moves_the_legs_and_idling_does_not() {
    let mut moving = input_at(Action::Free, 0, 0, 12);
    moving.speed = 7.0;
    let idle = input_at(Action::Free, 0, 0, 12);
    let leg = |p: &view::Pose| p.get(Part::LegL).rot[0].abs();
    assert!(leg(&pose_for(moving)) > 0.1, "legs did not swing");
    assert!(leg(&pose_for(idle)) < 0.01, "idle legs are swinging");
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
