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
        w.advance([Input(Input::D), Input::default()]);
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
    prev.advance([Input(Input::LEFT), Input::default()]);
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

#[test]
fn camera_pulls_back_as_fighters_separate() {
    let mut rig = CameraRig::new(RigConfig::default());
    rig.update(0.016, [-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
    let close = rig.distance();
    for _ in 0..240 {
        rig.update(0.016, [-9.0, 0.0, 0.0], [9.0, 0.0, 0.0]);
    }
    assert!(rig.distance() > close, "{} !> {close}", rig.distance());
}

#[test]
fn camera_never_whips_around_when_fighters_cross() {
    // The perpendicular flips by 180 degrees when the fighters swap sides.
    // Without shortest-path plus a rate limit, the camera spins.
    let mut rig = CameraRig::new(RigConfig::default());
    for _ in 0..120 {
        rig.update(0.016, [-3.0, 0.0, 0.0], [3.0, 0.0, 0.0]);
    }
    let mut worst: f32 = 0.0;
    let mut last = rig.yaw();
    for step in 0..120 {
        // Slide them through each other.
        let x = 3.0 - step as f32 * 0.05;
        rig.update(0.016, [-x, 0.0, 0.0], [x, 0.0, 0.0]);
        let delta = (rig.yaw() - last)
            .abs()
            .min(std::f32::consts::TAU - (rig.yaw() - last).abs());
        worst = worst.max(delta);
        last = rig.yaw();
    }
    let limit = RigConfig::default().max_yaw_rate * 0.016 * 1.05;
    assert!(
        worst <= limit,
        "yaw jumped {worst} rad in one frame (limit {limit})"
    );
}

#[test]
fn camera_looks_between_the_fighters() {
    let mut rig = CameraRig::new(RigConfig::default());
    let f = rig.update(0.016, [-4.0, 0.0, 2.0], [6.0, 0.0, -2.0]);
    assert!((f.look_at[0] - 1.0).abs() < 0.01, "x {}", f.look_at[0]);
    assert!((f.look_at[2] - 0.0).abs() < 0.01, "z {}", f.look_at[2]);
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
                Input(if i % 9 == 0 { Input::LEFT } else { Input::D }),
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
    let mut framing = rig.update(0.016, [-7.0, 0.0, -3.0], [-7.0, 0.0, 3.0]);
    for _ in 0..200 {
        framing = rig.update(0.016, [-7.0, 0.0, -3.0], [-7.0, 0.0, 3.0]);
    }
    let inside = sim::arena::SOLIDS.iter().any(|s| {
        (0..3).all(|i| {
            let lo = [s.min.x, s.min.y, s.min.z][i].to_f32_for_render();
            let hi = [s.max.x, s.max.y, s.max.z][i].to_f32_for_render();
            framing.eye[i] > lo && framing.eye[i] < hi
        })
    });
    assert!(
        !inside,
        "camera ended up inside arena geometry at {:?}",
        framing.eye
    );
}
