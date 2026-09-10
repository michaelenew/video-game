//! The prototype.
//!
//! Bevy is the renderer and nothing else. It owns no gameplay state: every
//! frame this reads the simulation, interpolates, and writes transforms. That
//! separation is what keeps rollback possible, and it is why the engine choice
//! stays reversible.
//!
//! Controls, per `docs/design/controls.md`:
//!   WASD move · Space jump · J or Left click attack · Shift+J slam
//!   K or Right click guard · R reset · P pause · ] step one frame

use bevy::prelude::*;
use sim::state::MAX_PLAYERS;
use sim::{Input as SimInput, World, arena};
use view::interp::TickClock;
use view::pose::{PARTS, Part, PoseInput, part_size, pose_for};
use view::{CameraRig, camera::RigConfig, interpolate};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Arena — prototype".into(),
                resolution: (1280.0, 760.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.06, 0.08)))
        .init_resource::<Sim>()
        .init_resource::<Rig>()
        .add_systems(Startup, setup)
        .add_systems(Update, (tick_sim, apply_poses, drive_camera).chain())
        .run();
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// The simulation, plus the previous snapshot so the renderer can interpolate.
#[derive(Resource)]
struct Sim {
    prev: World,
    cur: World,
    clock: TickClock,
    paused: bool,
    step_once: bool,
    dummy_guards: bool,
}

impl Default for Sim {
    fn default() -> Self {
        let w = World::new();
        Sim {
            prev: w.clone(),
            cur: w,
            clock: TickClock::new(),
            paused: false,
            step_once: false,
            dummy_guards: false,
        }
    }
}

#[derive(Resource)]
struct Rig(CameraRig);

impl Default for Rig {
    fn default() -> Self {
        Rig(CameraRig::new(RigConfig::default()))
    }
}

#[derive(Component)]
struct Fighter(usize);

#[derive(Component)]
struct BodyPart {
    owner: usize,
    part: Part,
}

#[derive(Component)]
struct MainCamera;

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 6.0, 14.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        MainCamera,
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 11_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(6.0, 14.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.65, 0.72, 0.85),
        brightness: 240.0,
        ..default()
    });

    // Floor.
    let floor = materials.add(StandardMaterial {
        base_color: Color::srgb(0.13, 0.15, 0.18),
        perceptual_roughness: 0.95,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(40.0, 40.0))),
        MeshMaterial3d(floor),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Arena geometry, straight from the simulation's own collision data. One
    // source of truth: if you can see it, you collide with it.
    let stone = materials.add(StandardMaterial {
        base_color: Color::srgb(0.21, 0.24, 0.29),
        perceptual_roughness: 0.9,
        ..default()
    });
    for solid in arena::SOLIDS.iter() {
        let min = fx3(solid.min);
        let max = fx3(solid.max);
        let size = max - min;
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(stone.clone()),
            Transform::from_translation((min + max) * 0.5),
        ));
    }

    // Fighters: a root per player, six primitive parts parented to it.
    let colors = [Color::srgb(0.29, 0.66, 1.0), Color::srgb(1.0, 0.54, 0.30)];
    for (owner, colour) in colors.iter().enumerate().take(MAX_PLAYERS) {
        let skin = materials.add(StandardMaterial {
            base_color: *colour,
            perceptual_roughness: 0.65,
            ..default()
        });
        commands
            .spawn((Fighter(owner), Transform::default(), Visibility::default()))
            .with_children(|root| {
                for part in PARTS {
                    let s = part_size(part);
                    root.spawn((
                        Mesh3d(meshes.add(Cuboid::new(s[0], s[1], s[2]))),
                        MeshMaterial3d(skin.clone()),
                        Transform::default(),
                        BodyPart { owner, part },
                    ));
                }
            });
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Read real input, run whole fixed ticks, keep the previous snapshot.
fn tick_sim(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut sim: ResMut<Sim>,
) {
    if keys.just_pressed(KeyCode::KeyP) {
        sim.paused = !sim.paused;
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        sim.step_once = true;
        sim.paused = true;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        let w = World::new();
        sim.prev = w.clone();
        sim.cur = w;
    }
    if keys.just_pressed(KeyCode::KeyB) {
        sim.dummy_guards = !sim.dummy_guards;
    }

    let local = if demo_mode() {
        demo_input(sim.cur.frame)
    } else {
        read_input(&keys, &mouse)
    };
    let dummy = if sim.dummy_guards {
        SimInput(SimInput::RIGHT)
    } else {
        SimInput::default()
    };

    let ticks = if sim.paused {
        u32::from(std::mem::take(&mut sim.step_once))
    } else {
        sim.clock.advance(time.delta_secs())
    };

    for _ in 0..ticks {
        sim.prev = sim.cur.clone();
        let inputs = [local, dummy];
        sim.cur.advance(inputs);
    }
}

/// `DEMO=1` drives player one from a script instead of the keyboard. Used to
/// verify posing and framing without a human at the controls.
fn demo_mode() -> bool {
    std::env::var("DEMO").is_ok_and(|v| v == "1")
}

fn demo_input(frame: u32) -> SimInput {
    let beat = frame % 180;
    let mut v = 0u16;
    match beat {
        0..=54 => v |= SimInput::D,                         // walk in
        55..=70 => v |= SimInput::LEFT,                     // bash
        90..=120 => v |= SimInput::A,                       // back off
        130..=150 => v |= SimInput::SHIFT | SimInput::LEFT, // slam
        _ => {}
    }
    SimInput(v)
}

fn read_input(keys: &ButtonInput<KeyCode>, mouse: &ButtonInput<MouseButton>) -> SimInput {
    let mut v = 0u16;
    // Keyboard stand-ins for the clicks, so the prototype is playable without
    // fighting the browser-style context menu on right click.
    if keys.pressed(KeyCode::KeyJ) || mouse.pressed(MouseButton::Left) {
        v |= SimInput::LEFT;
    }
    if keys.pressed(KeyCode::KeyK) || mouse.pressed(MouseButton::Right) {
        v |= SimInput::RIGHT;
    }
    if keys.pressed(KeyCode::KeyL) || mouse.pressed(MouseButton::Middle) {
        v |= SimInput::MIDDLE;
    }
    if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
        v |= SimInput::SHIFT;
    }
    if keys.pressed(KeyCode::KeyW) {
        v |= SimInput::W;
    }
    if keys.pressed(KeyCode::KeyA) {
        v |= SimInput::A;
    }
    if keys.pressed(KeyCode::KeyS) {
        v |= SimInput::S;
    }
    if keys.pressed(KeyCode::KeyD) {
        v |= SimInput::D;
    }
    if keys.pressed(KeyCode::Space) {
        v |= SimInput::SPACE;
    }
    SimInput(v)
}

/// Place every character part from the interpolated snapshot.
///
/// Pose is a pure function of simulation state -- see `view::pose`. Nothing
/// here accumulates animation time, which is what lets a rollback rewind the
/// characters without them sliding.
fn apply_poses(
    sim: Res<Sim>,
    mut roots: Query<(&Fighter, &mut Transform), Without<BodyPart>>,
    mut parts: Query<(&BodyPart, &mut Transform), Without<Fighter>>,
) {
    let frame = interpolate(&sim.prev, &sim.cur, sim.clock.alpha());

    for (fighter, mut tf) in roots.iter_mut() {
        let p = frame.players[fighter.0];
        tf.translation = Vec3::new(p.pos[0], p.pos[1], p.pos[2]);
        let yaw = p.facing[0].atan2(p.facing[2]);
        tf.rotation = Quat::from_rotation_y(yaw);
    }

    for (bp, mut tf) in parts.iter_mut() {
        let p = frame.players[bp.owner];
        let (into, total) = phase_frames(&p);
        let pose = pose_for(PoseInput {
            action: p.action,
            frames_into: into,
            frames_total: total,
            speed: p.speed,
            grounded: p.grounded,
            sim_frame: frame.sim_frame,
        });
        let t = pose.get(bp.part);
        tf.translation = Vec3::new(t.pos[0], t.pos[1], t.pos[2]);
        tf.rotation = Quat::from_euler(EulerRot::XYZ, t.rot[0], t.rot[1], t.rot[2]);
    }
}

/// How far into the current phase, and how long that phase runs.
fn phase_frames(p: &view::PlayerView) -> (u16, u16) {
    use sim::state::{Action, move_frames};
    match p.action {
        Action::Startup { kind, left } => {
            let total = move_frames(kind).0;
            (total.saturating_sub(left), total)
        }
        Action::Active { kind, left } => {
            let total = move_frames(kind).1;
            (total.saturating_sub(left), total)
        }
        Action::Recovery { kind, left } => {
            let total = move_frames(kind).2;
            (total.saturating_sub(left), total)
        }
        _ => (0, 0),
    }
}

fn drive_camera(
    sim: Res<Sim>,
    time: Res<Time>,
    mut rig: ResMut<Rig>,
    mut cam: Query<&mut Transform, With<MainCamera>>,
) {
    let frame = interpolate(&sim.prev, &sim.cur, sim.clock.alpha());
    let framing = rig.0.update(
        time.delta_secs(),
        frame.players[0].pos,
        frame.players[1].pos,
    );
    if let Ok(mut tf) = cam.single_mut() {
        tf.translation = Vec3::from_array(framing.eye);
        tf.look_at(Vec3::from_array(framing.look_at), Vec3::Y);
    }
}

fn fx3(v: sim::V3) -> Vec3 {
    Vec3::new(
        v.x.to_f32_for_render(),
        v.y.to_f32_for_render(),
        v.z.to_f32_for_render(),
    )
}
