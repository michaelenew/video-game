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

/// Flat direction the camera is pointed.
///
/// Measured eye-to-target, which is now exactly the look direction: the screen
/// looks along the axis the mouse names. The eye still sits off to one
/// shoulder, so this ray is displaced sideways from the fighter's own -- but
/// displaced, not turned, and a parallel ray aims at the same angle. The
/// crosshair is projected from the *fighter's* facing along this same ray (see
/// `crosshair::aim_point`), which is what keeps it centred when the two agree.
fn aim_dir(f: view::camera::Framing, _player: [f32; 3]) -> (f32, f32) {
    let (dx, dz) = (f.look_at[0] - f.eye[0], f.look_at[2] - f.eye[2]);
    let len = (dx * dx + dz * dz).sqrt();
    (dx / len, dz / len)
}

#[test]
fn the_camera_sits_behind_the_fighter() {
    let mut rig = CameraRig::new(RigConfig::default());
    let f = rig.update(0.016, [2.0, 0.0, -1.0], 0.0, 0.0);
    // Looking down +X means the camera is at smaller X than the fighter.
    assert!(f.eye[0] < 2.0, "camera at {} is not behind", f.eye[0]);
    // And it aims past them, not at them, so they are not standing in front of
    // whatever you are trying to look at.
    assert!(
        f.look_at[0] > 2.0,
        "aim point at {} is not ahead of the fighter",
        f.look_at[0]
    );
    // The ray runs straight down +X. It is displaced sideways by the shoulder
    // offset -- parallel, not turned -- so what must not drift is its
    // direction, which is what the player is steering.
    assert!(
        (f.look_at[2] - f.eye[2]).abs() < 0.01,
        "aim drifted sideways"
    );
    assert!(f.eye[1] > 0.0, "camera is underground");
    // Looking level means looking level -- the screen follows the mouse, not a
    // built-in tilt. What keeps the fighter out of the middle of the frame is
    // the eye sitting above their head, not the camera aiming downward.
    assert!(
        f.eye[1] > RigConfig::default().look_height,
        "the eye at {} is not above the fighter",
        f.eye[1]
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

fn settled(pitch: f32) -> (view::camera::Framing, [f32; 3]) {
    let mut rig = CameraRig::new(RigConfig::default());
    // Past the platforms in z, which reach four metres either side of the
    // middle. The arm is eleven metres now, so a fixture in the middle of the
    // arena sweeps the camera straight through one and the occlusion pull-in
    // would be what these tests were measuring.
    let at = [0.0, 0.0, 8.0];
    for _ in 0..200 {
        rig.update(0.016, at, 0.0, pitch);
    }
    (rig.update(0.016, at, 0.0, pitch), at)
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
    for distance in [5.0, 8.0, 11.0, 15.0] {
        cfg.distance = distance;
        let mut rig = CameraRig::new(cfg);
        for _ in 0..200 {
            rig.update(0.016, at, 0.0, -cfg.neutral_pitch);
        }
        let f = rig.update(0.016, at, 0.0, -cfg.neutral_pitch);
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
        [at[0], cfg.look_height, at[2]],
    );
    for step in 1..=10 {
        let pitch = -(step as f32 / 10.0) * cfg.pitch_down;
        let (f, at) = settled(pitch);
        let flat = flat_arm(f, [at[0], cfg.look_height, at[2]]);
        // Pitching down shortens the *flat* arm by cosine alone -- the camera
        // rises over the fighter. What must not happen is the arm itself
        // getting shorter.
        let arm = (flat * flat + (f.eye[1] - cfg.look_height).powi(2)).sqrt();
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
    // Reported: the feet were chopped off. The fighter has to sit below the
    // middle of the screen -- that is what the orbit centre being above their
    // head buys -- but inside the bottom of it.
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
        head > 0.02,
        "the fighter's head is on the crosshair; the orbit centre is not above them"
    );
}

#[test]
fn looking_up_walks_the_camera_down_before_it_climbs_into_the_head() {
    // The Skyrim shape. The ordinary third-person answer -- drop the arm toward
    // the ground behind the fighter -- reads well for the first forty degrees
    // or so, and this checks it still happens.
    let cfg = RigConfig::default();
    let mut last = f32::INFINITY;
    for step in 0..=10 {
        let pitch = step as f32 / 10.0 * cfg.sky_start;
        let h = settled(pitch).0.eye[1];
        assert!(h <= last + 0.001, "camera rose at pitch {pitch:.2}");
        last = h;
    }
    assert!(
        last < 0.6,
        "camera never got near the ground on the way up; lowest was {last:.2}"
    );
}

#[test]
fn looking_further_up_hands_over_to_the_fighters_own_eyes() {
    // Past the handover the third-person answer has run out: the arm is on the
    // floor, the body is in the way, and terrain shoves the view. The camera
    // goes to the fighter's eyes so the sky can simply be panned.
    let cfg = RigConfig::default();
    let (f, at) = settled(cfg.pitch_up);
    assert!(
        f.first_person > 0.9,
        "at full pitch the rig is only {:.2} of the way into the head",
        f.first_person
    );
    let flat = flat_arm(f, [at[0], cfg.look_height, at[2]]);
    assert!(
        flat < 0.5,
        "the eye is still {flat:.2}m behind the fighter, so the body is in the sky"
    );
    assert!(
        f.eye[1] > cfg.look_height - 0.1,
        "the eye dropped to {:.2}, below the fighter's own head",
        f.eye[1]
    );
}

#[test]
fn the_handover_does_not_start_early() {
    // A modest look up must still be third person, or the body vanishes while
    // the player is only glancing upward.
    let cfg = RigConfig::default();
    let (f, at) = settled(0.4);
    assert_eq!(
        f.first_person, 0.0,
        "the rig started climbing into the head at a 23-degree look up"
    );
    let focus = [at[0], cfg.look_height, at[2]];
    let level = flat_arm(settled(0.0).0, focus);
    assert!(
        flat_arm(f, focus) > level * 0.85,
        "a 23-degree look up pulled the camera in"
    );
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
        let f = rig.update(0.016, [0.0, 0.0, 0.0], 0.0, pitch);
        let rise = f.look_at[1] - f.eye[1];
        let run = ((f.look_at[0] - f.eye[0]).powi(2) + (f.look_at[2] - f.eye[2]).powi(2)).sqrt();
        let got = rise.atan2(run);
        assert!(
            got >= -cfg.pitch_down - 0.01 && got <= cfg.pitch_up + 0.01,
            "pitched to {got} past the {} / {} limits",
            cfg.pitch_down,
            cfg.pitch_up
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

// ---------------------------------------------------------------------------
// Weapon trails
// ---------------------------------------------------------------------------

mod trails {
    use sim::state::Action;
    use view::pose::PoseInput;
    use view::trail;

    fn swinging(frames_into: u16) -> PoseInput {
        PoseInput {
            clip: None,
            action: Action::Active { kind: 0, left: 3 },
            frames_into,
            frames_total: 10,
            speed: 0.0,
            grounded: true,
            crouching: false,
            sim_frame: 600,
        }
    }

    #[test]
    fn a_trail_is_recomputed_rather_than_remembered() {
        // The property the whole design rests on. Asking twice has to give the
        // same answer, because that is what makes a rollback -- which asks
        // again with corrected state -- produce a correct trail instead of a
        // smear through a swing that never happened.
        let a = trail::sweep(swinging(6), 8);
        let b = trail::sweep(swinging(6), 8);
        assert_eq!(a, b);
        assert!(!a.is_empty());
    }

    #[test]
    fn a_clip_lets_the_trail_reach_back_through_the_whole_move() {
        // The bug that made the first version invisible. `frames_into` counts
        // within a *phase*, so at the start of an active window it is nearly
        // zero -- which is exactly when the swing is fastest and the trail
        // should be longest. A clip's elapsed count runs across the whole move,
        // so walking that back crosses into the wind-up where the arc is.
        let just_became_active = PoseInput {
            clip: Some((view::pose::Clip::Move(view::baked::move_clip(0, 1)), 15)),
            frames_into: 1,
            ..swinging(1)
        };
        assert_eq!(
            trail::sweep(just_became_active, 8).len(),
            8,
            "the trail is being cut short by the phase counter instead of the clip"
        );
    }

    #[test]
    fn a_trail_stops_at_the_start_of_its_own_action() {
        // `frames_into` counts within an action and the previous action is not
        // recoverable from here, so reaching further back would be inventing
        // motion. Worse, subtracting past zero on a `u16` wraps to 65535 and
        // silently indexes the pose function with a frame from the far end of
        // the move.
        for into in 0..8u16 {
            let sweep = trail::sweep(swinging(into), 8);
            assert_eq!(
                sweep.len(),
                into as usize + 1,
                "at {into} frames in, the trail reached back {} samples",
                sweep.len()
            );
        }
        assert_eq!(
            trail::sweep(swinging(40), 8).len(),
            8,
            "a long action should cap"
        );
    }

    #[test]
    fn the_newest_sample_is_the_current_pose() {
        // A trail whose head lags the weapon reads as the weapon outrunning its
        // own arc, which looks like a bug in the animation rather than in the
        // trail.
        let sweep = trail::sweep(swinging(5), 8);
        let now = trail::sweep(swinging(5), 1);
        assert_eq!(sweep[0], now[0]);
        assert_eq!(sweep[0].age, 0.0);
    }

    #[test]
    fn age_runs_from_now_into_the_past() {
        let sweep = trail::sweep(swinging(20), 8);
        let mut last = -1.0;
        for e in &sweep {
            assert!(e.age > last, "ages are not increasing: {:?}", sweep);
            last = e.age;
        }
        assert!(last < 1.0);
    }

    #[test]
    fn the_blade_hangs_below_a_resting_arm() {
        // The direction test anyone can check by holding their own arm out. An
        // arm at rest points down, so the edge does too -- and its far end is
        // further from the shoulder than its near end.
        let idle = PoseInput {
            action: Action::Free,
            frames_into: 0,
            ..swinging(0)
        };
        let edge = trail::sweep(idle, 1)[0];
        assert!(
            edge.far[1] < edge.near[1],
            "the far end of the blade is above the near end: {edge:?}"
        );
        let reach = (edge.far[1] - edge.near[1]).abs();
        assert!(reach > 0.5, "the blade is only {reach:.2} m long");
    }

    #[test]
    fn a_baked_swing_moves_the_edge_and_standing_still_does_not() {
        // What `speed` is for: a move whose arm barely travels should not get a
        // ribbon, and the only way to know is to compare.
        let idle = PoseInput {
            action: Action::Free,
            ..swinging(30)
        };
        assert_eq!(trail::speed(&trail::sweep(idle, 8)), 0.0);

        let swung = PoseInput {
            clip: Some((view::pose::Clip::Move(view::baked::move_clip(0, 0)), 8)),
            frames_into: 8,
            ..swinging(8)
        };
        assert!(
            trail::speed(&trail::sweep(swung, 8)) > 0.05,
            "a baked swing does not out-travel standing still"
        );
    }

    #[test]
    fn a_trail_is_only_as_good_as_the_animation_under_it() {
        // Worth pinning because it is the thing that decides where trails can
        // be turned on, and it is invisible from the outside.
        //
        // **The procedural fallback poses are one pose per phase.** The whole
        // active window of a move is a single static arm position, so a trail
        // sampled across it is the same edge eight times -- a stationary
        // ribbon, which reads as a rendering fault rather than as a swing.
        // Baked clips do move, because the spring solver fills in every frame
        // between the keys.
        //
        // So a trail is drawn where there is a clip and nowhere else, and the
        // way to widen that is to give more moves clips -- which is what
        // generating them from frame data is for.
        let procedural = swinging(6);
        assert_eq!(
            trail::speed(&trail::sweep(procedural, 8)),
            0.0,
            "the procedural active pose has started moving -- good, but the renderer's \
             rule for when to draw a trail was written assuming it does not"
        );

        let baked = PoseInput {
            clip: Some((view::pose::Clip::Move(view::baked::move_clip(0, 1)), 11)),
            frames_into: 11,
            ..swinging(11)
        };
        assert!(trail::speed(&trail::sweep(baked, 8)) > 0.05);
    }
}
