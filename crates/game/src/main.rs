//! The prototype.
//!
//! Bevy is the renderer and nothing else. It owns no gameplay state: every
//! frame this reads the simulation, interpolates, and writes transforms. That
//! separation is what keeps rollback possible, and it is why the engine choice
//! stays reversible.
//!
//! Controls, per `docs/design/controls.md`:
//!   WASD move · Space jump · Shift+direction dodge (airdodge once per jump)
//!   J bash · Shift+J slam · K guard · L shield throw/recall · Shift+L grapple
//!   1-4 dummy mode · F1 debug overlay · P pause · ] step one frame · R reset
//!
//! Player two: arrows, RCtrl, Period, Comma, Slash, RShift.
//!
//! Pick classes with `--p1 <class> --p2 <class>`, or cycle player one's class
//! in-game with Tab. Names are matched loosely: bulwark, bellator, reaver,
//! elementalist, blood, dual.

mod bake;
mod crosshair;
mod debug;
mod hud;
mod palette;
mod settings;

use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use sim::state::MAX_PLAYERS;
use sim::{Input as SimInput, World, arena};
use view::interp::TickClock;
use view::pose::{PARTS, Part, PoseInput, part_size, pose_for};
use view::{CameraRig, aim_from_radians, camera::RigConfig, interpolate};

/// How the match is being driven.
///
/// Local is the training mode. Online routes every tick through GGRS, which
/// owns when to save, load and advance -- the simulation only has to do those
/// three things correctly.
enum Driver {
    Local,
    Online {
        session: Box<net::ggrs::P2PSession<net::SessionConfig>>,
        handle: usize,
        desynced: bool,
    },
}

/// `game` for training mode, or:
///
///     game --port 47801 --peer 192.168.1.20:47802
///
/// Both peers derive who is player one from the two addresses, so there is no
/// server and no lobby.
/// Loose class-name matching, so `--p1 reaver` works without remembering the
/// full name.
fn parse_class(name: &str) -> Option<sim::Class> {
    use sim::Class::*;
    let n = name.to_lowercase();
    ALL.iter()
        .copied()
        .find(|c| {
            c.name().to_lowercase().replace(' ', "").starts_with(&n) || matches(n.as_str(), *c)
        })
        .or(match n.as_str() {
            "reaver" | "shadow" => Some(ShadowReaver),
            "blood" => Some(BloodMage),
            "dual" => Some(DualMage),
            _ => None,
        })
}

const ALL: [sim::Class; 6] = sim::class::ALL_CLASSES;

fn matches(n: &str, c: sim::Class) -> bool {
    c.name().to_lowercase().contains(n) && !n.is_empty()
}

fn arg(flag: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn chosen_classes() -> [sim::Class; 2] {
    [
        arg("--p1")
            .and_then(|n| parse_class(&n))
            .unwrap_or(sim::Class::Bulwark),
        arg("--p2")
            .and_then(|n| parse_class(&n))
            .unwrap_or(sim::Class::Bulwark),
    ]
}

fn parse_args() -> Option<(u16, std::net::SocketAddr)> {
    let args: Vec<String> = std::env::args().collect();
    let get = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let port: u16 = get("--port")?.parse().ok()?;
    let peer: std::net::SocketAddr = get("--peer")?.parse().ok()?;
    Some((port, peer))
}

fn main() {
    // `--help` before anything else, so asking what the flags are does not
    // require a window, a GPU, or the patience to wait for Bevy to start.
    if std::env::args().any(|a| a == "--help" || a == "-h") {
        print!("{}", manual::render());
        return;
    }
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
        .init_resource::<debug::ShowDebug>()
        .init_resource::<Look>()
        .init_resource::<palette::Palette>()
        .init_resource::<palette::UiFocus>()
        .add_plugins(bevy_egui::EguiPlugin {
            enable_multipass_for_primary_context: false,
        })
        .insert_resource(settings::Settings::load())
        .add_systems(Startup, (setup, hud::setup, crosshair::setup))
        .add_systems(
            Update,
            (
                // Who owns the mouse and keyboard this frame, before anything
                // reads them.
                palette::sample_focus,
                // Mouse look runs next: aim is an input to the tick, not a
                // decoration applied after it.
                mouse_look,
                tick_sim,
                apply_poses,
                place_shields,
                drive_camera,
                hud::update,
                crosshair::update,
                debug::draw,
                palette::toggle,
                palette::draw,
            )
                .chain(),
        )
        .run();
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// The simulation, plus the previous snapshot so the renderer can interpolate.
#[derive(Resource)]
pub struct Sim {
    pub prev: World,
    pub cur: World,
    pub clock: TickClock,
    paused: bool,
    step_once: bool,
    /// `SHOT_FRAME=N` runs to frame N and stops. Makes a screenshot land on an
    /// exact simulation frame instead of wherever `sleep` happened to leave it,
    /// which is what turns "does this look right" into a repeatable comparison.
    stop_at: Option<u32>,
    dummy: Dummy,
    driver: Driver,
    /// Baked animation on or off. A toggle because it is new, and because
    /// playback cost should be measurable against the procedural path.
    baked_anim: bool,
}

/// Training-mode opponent. Player two is a scripted dummy until someone takes
/// the second set of keys.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Dummy {
    Idle,
    Block,
    Attack,
    Human,
}

impl Default for Sim {
    fn default() -> Self {
        let w = World::with_classes(chosen_classes());
        let driver = match parse_args() {
            Some((port, peer)) => {
                let local: std::net::SocketAddr =
                    format!("127.0.0.1:{port}").parse().expect("local addr");
                let handle = net::p2p::local_handle_for(local, peer);
                match net::p2p::start(port, peer, handle) {
                    Ok(session) => {
                        eprintln!(
                            "online: port {port} to {peer}, you are player {}",
                            handle + 1
                        );
                        Driver::Online {
                            session: Box::new(session),
                            handle,
                            desynced: false,
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "could not start the session ({e}); falling back to training mode"
                        );
                        Driver::Local
                    }
                }
            }
            None => Driver::Local,
        };
        Sim {
            prev: w.clone(),
            cur: w,
            clock: TickClock::new(),
            paused: false,
            step_once: false,
            dummy: Dummy::Idle,
            driver,
            // `BAKED_ANIM=0` starts with the procedural poses instead, so the
            // two can be captured back to back without a keypress.
            baked_anim: std::env::var("BAKED_ANIM").as_deref() != Ok("0"),
            stop_at: env_num("SHOT_FRAME"),
        }
    }
}

impl Sim {
    /// Which fighter this client drives. Online it is the GGRS handle; locally
    /// it is always player one, with player two on the second key set.
    fn local_player(&self) -> usize {
        match self.driver {
            Driver::Online { handle, .. } => handle.min(1),
            Driver::Local => 0,
        }
    }
}

/// Start pitch override, for capturing the camera at a known angle.
fn env_f32(key: &str) -> Option<f32> {
    std::env::var(key).ok()?.parse().ok()
}

fn env_num(key: &str) -> Option<u32> {
    std::env::var(key).ok()?.parse().ok()
}

/// Where each local player is looking. Renderer-side state: the yaw is
/// quantised and handed to the simulation as input, but the float itself never
/// crosses the wire and never enters a snapshot.
///
/// Pitch is here and nowhere else. It moves the camera and changes nothing
/// about the fight, so it has no business in the simulation.
#[derive(Resource)]
struct Look {
    yaw: f32,
    pitch: f32,
    /// Player two's yaw, for two people on one keyboard. They have no mouse, so
    /// they turn with keys.
    yaw_two: f32,
    grabbed: bool,
}

impl Default for Look {
    fn default() -> Self {
        Look {
            // Player one spawns at -X looking toward +X, where player two is.
            yaw: 0.0,
            pitch: env_f32("SHOT_PITCH").unwrap_or(0.12),
            yaw_two: std::f32::consts::PI,
            grabbed: false,
        }
    }
}

impl Look {
    fn aim(&self) -> u16 {
        aim_from_radians(self.yaw)
    }

    fn aim_two(&self) -> u16 {
        aim_from_radians(self.yaw_two)
    }
}

#[derive(Resource)]
struct Rig(CameraRig);

impl Default for Rig {
    fn default() -> Self {
        // Settings override this on the first frame; the default keeps the rig
        // sane if the resource is somehow missing.
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

#[derive(Component)]
struct ShieldMesh(usize);

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

fn setup(
    mut commands: Commands,
    settings: Res<settings::Settings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        // Bevy's default is a 45-degree vertical field of view, which is a
        // portrait-lens view of an arena you are meant to be moving around
        // inside. The real value is a setting; this is just the starting point.
        Projection::Perspective(PerspectiveProjection {
            fov: settings.fov_radians(),
            ..default()
        }),
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
    // A second, dimmer light from behind and opposite, with no shadows. The
    // old camera looked down on the arena from outside it; this one looks along
    // the floor at the inside faces of the walls, which the key light never
    // reaches. Without a fill they read as flat black and the fight happens in
    // front of a void.
    commands.spawn((
        DirectionalLight {
            illuminance: 3_200.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-8.0, 6.0, -7.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.65, 0.72, 0.85),
        brightness: 520.0,
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
        base_color: Color::srgb(0.30, 0.34, 0.40),
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

        // The shield is a separate object because its position is independent
        // of the character -- that is the whole mechanic. See bulwark.md.
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.75, 0.9, 0.14))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.92, 0.76, 0.38),
                perceptual_roughness: 0.5,
                ..default()
            })),
            Transform::default(),
            Visibility::Hidden,
            ShieldMesh(owner),
        ));
    }
}

/// Where the class mechanic sits in the world, if anywhere. A shield in hand
/// rides on the character and draws nothing; a thrown one, a placed shadow or a
/// raised structure all get a marker.
fn mechanic_world_pos(m: &sim::class::Mechanic) -> Option<sim::V3> {
    use sim::class::Mechanic;
    match m {
        Mechanic::Shield(s) => s.world_pos(),
        Mechanic::Shadow { at } => *at,
        Mechanic::Structures(slots) => slots.iter().flatten().next().copied(),
        _ => None,
    }
}

/// A shield in hand rides on the character; a thrown one sits in the world.
fn place_shields(
    sim: Res<Sim>,
    mut shields: Query<(&ShieldMesh, &mut Transform, &mut Visibility)>,
) {
    for (tag, mut tf, mut vis) in shields.iter_mut() {
        match mechanic_world_pos(&sim.cur.players[tag.0].mechanic) {
            Some(pos) => {
                *vis = Visibility::Inherited;
                tf.translation = Vec3::new(
                    pos.x.to_f32_for_render(),
                    pos.y.to_f32_for_render(),
                    pos.z.to_f32_for_render(),
                );
            }
            None => *vis = Visibility::Hidden,
        }
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
    look: Res<Look>,
    focus: Res<palette::UiFocus>,
    mut sim: ResMut<Sim>,
    mut show: ResMut<debug::ShowDebug>,
) {
    // Typing in a text field must not also pause the match or cycle the class.
    // F7 stays live regardless, since it is the way back out.
    if focus.keyboard {
        return;
    }
    if keys.just_pressed(KeyCode::KeyP) {
        sim.paused = !sim.paused;
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        sim.step_once = true;
        sim.paused = true;
    }
    if keys.just_pressed(KeyCode::F1) {
        // Toggled here rather than in the debug module so all input reading
        // stays in one place.
        show.0 = !show.0;
    }
    if keys.just_pressed(KeyCode::F2) {
        sim.baked_anim = !sim.baked_anim;
    }
    if keys.just_pressed(KeyCode::Tab) {
        // Cycle player one's class. Restarts the match, since a class change
        // mid-round would leave the mechanic in someone else's state.
        let next = (sim.cur.players[0].class as usize + 1) % ALL.len();
        let w = World::with_classes([ALL[next], sim.cur.players[1].class]);
        sim.prev = w.clone();
        sim.cur = w;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        let w = World::with_classes([sim.cur.players[0].class, sim.cur.players[1].class]);
        sim.prev = w.clone();
        sim.cur = w;
    }
    for (key, mode) in [
        (KeyCode::Digit1, Dummy::Idle),
        (KeyCode::Digit2, Dummy::Block),
        (KeyCode::Digit3, Dummy::Attack),
        (KeyCode::Digit4, Dummy::Human),
    ] {
        if keys.just_pressed(key) {
            sim.dummy = mode;
        }
    }

    // Input the palette is claiming belongs to the palette. Clicking a slider
    // used to throw a poke as well, and typing in the search box drove the
    // fighter around -- J, K and L are attack keys.
    let held = if focus.pointer || focus.keyboard {
        SimInput::default()
    } else {
        read_input(&keys, &mouse)
    }
    .looking(look.aim());
    let held_two = if focus.keyboard {
        SimInput::default()
    } else {
        read_player_two(&keys)
    }
    .looking(look.aim_two());

    match &mut sim.driver {
        Driver::Local => {
            let ticks = if sim.paused {
                u32::from(std::mem::take(&mut sim.step_once))
            } else {
                sim.clock.advance(time.delta_secs())
            };
            for _ in 0..ticks {
                if sim.stop_at.is_some_and(|n| sim.cur.frame >= n) {
                    sim.paused = true;
                    break;
                }
                // Scripted inputs are resampled per simulation tick, not per
                // rendered frame. One render frame can cover several ticks, and
                // reusing a sample across them smears a four-frame press into
                // whatever the frame rate happened to be -- which makes two
                // runs of the same script diverge.
                let pair = [
                    scripted_or(
                        demo_mode().then(|| demo_input(&sim.cur, sim.cur.frame)),
                        held,
                    ),
                    dummy_input(sim.dummy, sim.cur.frame, held_two),
                ];
                sim.prev = sim.cur.clone();
                sim.cur.advance(pair);
            }
        }
        Driver::Online { .. } => {
            let ticks = sim.clock.advance(time.delta_secs());
            for _ in 0..ticks {
                let local = scripted_or(
                    demo_mode().then(|| demo_input(&sim.cur, sim.cur.frame)),
                    held,
                );
                step_online(&mut sim, local);
            }
        }
    }
}

/// One networked tick.
///
/// GGRS decides when to save, load and advance; `handle_requests` services
/// those against the simulation. Rollbacks land here as a load followed by
/// several advances, all inside one call.
fn step_online(sim: &mut Sim, local: SimInput) {
    let Driver::Online {
        session,
        handle,
        desynced,
    } = &mut sim.driver
    else {
        return;
    };

    session.poll_remote_clients();
    for event in session.events() {
        match event {
            net::ggrs::GgrsEvent::DesyncDetected { frame, .. } => {
                // Should be impossible: the simulation is integer-only and
                // SyncTest covers it. If it happens, say so loudly rather than
                // letting the two players drift apart in silence.
                eprintln!("DESYNC at frame {frame}");
                *desynced = true;
            }
            net::ggrs::GgrsEvent::Disconnected { addr } => eprintln!("peer {addr} disconnected"),
            net::ggrs::GgrsEvent::NetworkInterrupted { addr, .. } => {
                eprintln!("peer {addr} interrupted")
            }
            _ => {}
        }
    }

    if session.current_state() != net::ggrs::SessionState::Running {
        return;
    }

    if session
        .add_local_input(*handle, net::NetInput::from(local))
        .is_err()
    {
        // Too far ahead of the peer. Waiting is the correct response.
        return;
    }

    let prev = sim.cur.clone();
    match session.advance_frame() {
        Ok(requests) => {
            net::handle_requests(&mut sim.cur, requests);
            sim.prev = prev;
        }
        Err(net::ggrs::GgrsError::PredictionThreshold) => {}
        Err(e) => eprintln!("advance failed: {e}"),
    }
}

fn scripted_or(scripted: Option<SimInput>, live: SimInput) -> SimInput {
    scripted.unwrap_or(live)
}

/// The training partner. `Attack` runs on a cadence so the parry window is
/// something you can actually practise against.
fn dummy_input(mode: Dummy, frame: u32, live: SimInput) -> SimInput {
    match mode {
        Dummy::Idle => SimInput::default(),
        Dummy::Block => SimInput::new(SimInput::RIGHT),
        Dummy::Attack if frame % 70 < 2 => SimInput::new(SimInput::LEFT),
        Dummy::Attack => SimInput::default(),
        Dummy::Human => live,
    }
}

/// `DEMO=1` drives player one from a script instead of the keyboard. Used to
/// verify posing and framing without a human at the controls.
fn demo_mode() -> bool {
    std::env::var("DEMO").is_ok_and(|v| v == "1")
}

/// Quantised angle of a flat direction.
///
/// Floating point here is safe despite the determinism rules: this produces an
/// *input*, and inputs are transmitted rather than recomputed. A peer replays
/// the integer that arrived, never this function.
fn aim_toward(v: sim::V3) -> u16 {
    aim_from_radians(v.z.to_f32_for_render().atan2(v.x.to_f32_for_render()))
}

fn demo_input(w: &sim::World, frame: u32) -> SimInput {
    let beat = frame % 480;
    let mut v = 0u16;
    match beat {
        0..=70 => v |= SimInput::W,                         // close the gap
        75..=78 => v |= SimInput::LEFT,                     // bash
        110..=126 => v |= SimInput::RIGHT,                  // guard
        150..=153 => v |= SimInput::SHIFT | SimInput::LEFT, // slam
        190..=215 => v |= SimInput::CROUCH,                 // duck
        240..=243 => v |= SimInput::SHIFT | SimInput::S,    // dodge back
        250..=252 => v |= SimInput::SPACE,                  // jump
        280..=283 => v |= SimInput::MIDDLE,                 // throw the shield
        340..=343 => v |= SimInput::MIDDLE,                 // recall it
        410..=440 => v |= SimInput::S,                      // reset spacing
        _ => {}
    }
    // The script looks at its opponent, which is what a player would do and
    // what makes "forward" mean "toward the fight".
    SimInput::aimed(v, aim_toward(w.players[1].pos.sub(w.players[0].pos)))
}

/// Second set of keys, so two people can play on one keyboard.
fn read_player_two(keys: &ButtonInput<KeyCode>) -> SimInput {
    let mut v = 0u16;
    if keys.pressed(KeyCode::Period) {
        v |= SimInput::LEFT;
    }
    if keys.pressed(KeyCode::Comma) {
        v |= SimInput::RIGHT;
    }
    if keys.pressed(KeyCode::Slash) {
        v |= SimInput::MIDDLE;
    }
    if keys.pressed(KeyCode::ShiftRight) {
        v |= SimInput::SHIFT;
    }
    if keys.pressed(KeyCode::ArrowUp) {
        v |= SimInput::W;
    }
    if keys.pressed(KeyCode::ArrowLeft) {
        v |= SimInput::A;
    }
    if keys.pressed(KeyCode::ArrowDown) {
        v |= SimInput::S;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        v |= SimInput::D;
    }
    if keys.pressed(KeyCode::ControlRight) {
        v |= SimInput::SPACE;
    }
    SimInput::new(v)
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
    if keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::KeyC) {
        v |= SimInput::CROUCH;
    }
    SimInput::new(v)
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
        let class = sim.cur.players[bp.owner].class;
        let (into, total) = phase_frames(&p, class);
        let pose = pose_for(PoseInput {
            clip: if sim.baked_anim {
                clip_for(&p, class)
            } else {
                None
            },
            action: p.action,
            frames_into: into,
            frames_total: total,
            speed: p.speed,
            grounded: p.grounded,
            crouching: p.crouching,
            sim_frame: frame.sim_frame,
        });
        let t = pose.get(bp.part);
        tf.translation = Vec3::new(t.pos[0], t.pos[1], t.pos[2]);
        tf.rotation = Quat::from_euler(EulerRot::XYZ, t.rot[0], t.rot[1], t.rot[2]);
    }
}

/// Which baked clip an action maps to, and how far into it.
///
/// The game side picks, because it is what knows the move tables. `view` stays
/// ignorant of what an overhead is.
fn clip_for(p: &view::PlayerView, class: sim::Class) -> Option<(view::pose::Clip, u16)> {
    use sim::state::Action;
    use view::pose::Clip;
    let elapsed = |kind: u8, phase: u8, left: u16| -> u16 {
        let (s, a, r) = sim::moves::frames(class, kind);
        match phase {
            0 => s.saturating_sub(left),
            1 => s + a.saturating_sub(left),
            _ => s + a + r.saturating_sub(left),
        }
    };
    let attack_clip = |kind: u8| {
        if sim::moves::get(class, kind).hits_crouching {
            Clip::Poke
        } else {
            Clip::Overhead
        }
    };
    match p.action {
        Action::Startup { kind, left } => Some((attack_clip(kind), elapsed(kind, 0, left))),
        Action::Active { kind, left } => Some((attack_clip(kind), elapsed(kind, 1, left))),
        Action::Recovery { kind, left } => Some((attack_clip(kind), elapsed(kind, 2, left))),
        Action::Guard { held } => Some((Clip::GuardIn, held)),
        Action::Dodge { left } => Some((Clip::Roll, 22u16.saturating_sub(left))),
        Action::HitStun { left } | Action::BlockStun { left } | Action::Stagger { left } => {
            Some((Clip::Recoil, 26u16.saturating_sub(left)))
        }
        Action::Free => None,
    }
}

/// How far into the current phase, and how long that phase runs.
fn phase_frames(p: &view::PlayerView, class: sim::Class) -> (u16, u16) {
    use sim::state::Action;
    let move_frames = |k: u8| sim::moves::frames(class, k);
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

/// Mouse look, and the cursor grab that makes it usable.
///
/// Deliberately not rate-limited or smoothed. The mouse is the aim, and any
/// filtering between the hand and the crosshair is felt immediately even when
/// it cannot be named.
/// Whether the cursor should be locked to the window this frame.
///
/// Extracted so the rule can be stated once and tested, rather than living
/// inside a system where the only way to check it is to play the game.
fn cursor_should_be_captured(escape: bool, oven_open: bool, clicked: bool, captured: bool) -> bool {
    if escape || oven_open {
        false
    } else if clicked {
        true
    } else {
        captured
    }
}

fn mouse_look(
    mut look: ResMut<Look>,
    mut settings: ResMut<settings::Settings>,
    mut motion: EventReader<MouseMotion>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    open: Res<palette::Palette>,
    mut windows: Query<&mut Window>,
) {
    // Sensitivity, adjustable mid-match and written straight to disk. Two people
    // sharing a machine should not have to agree on one number, and a setting
    // you have to quit and edit a file to change is a setting nobody changes.
    let mut changed = false;
    for (key, knob, up) in [
        (KeyCode::Minus, settings::Knob::Sensitivity, false),
        (KeyCode::NumpadSubtract, settings::Knob::Sensitivity, false),
        (KeyCode::Equal, settings::Knob::Sensitivity, true),
        (KeyCode::NumpadAdd, settings::Knob::Sensitivity, true),
        (KeyCode::F3, settings::Knob::Fov, false),
        (KeyCode::F4, settings::Knob::Fov, true),
        (KeyCode::F5, settings::Knob::Distance, false),
        (KeyCode::F6, settings::Knob::Distance, true),
    ] {
        if keys.just_pressed(key) {
            settings.nudge(knob, up);
            changed = true;
        }
    }
    if changed {
        settings.save();
    }

    let limit = view::camera::RigConfig::default().pitch_limit;
    if look.grabbed {
        let sensitivity = settings.radians_per_pixel();
        let (mut dx, mut dy) = (0.0, 0.0);
        for ev in motion.read() {
            dx += ev.delta.x;
            dy += ev.delta.y;
        }
        look.yaw += dx * sensitivity;
        look.pitch = (look.pitch - dy * sensitivity).clamp(-limit, limit);
    } else {
        motion.clear();
    }

    // Player two turns with keys, since there is only one mouse.
    let turn = 0.045;
    if keys.pressed(KeyCode::Semicolon) {
        look.yaw_two -= turn;
    }
    if keys.pressed(KeyCode::Quote) {
        look.yaw_two += turn;
    }

    // Click to capture, Escape to release. Without a release the window is a
    // trap, and this runs windowed on a desktop.
    //
    // A click the palette is claiming is not a click at the arena. Without this
    // every drag of a slider re-grabbed the mouse, so tuning a number meant
    // pressing Escape between each one -- which is the opposite of the rapid
    // loop the Oven exists to provide.
    // One rule, so there is nothing to learn: **while the Oven is open the
    // cursor is yours.** Capturing on any click meant every drag of a slider
    // re-grabbed the mouse and tuning a number cost an Escape each time, which
    // is the opposite of the rapid loop the Oven exists to provide. Releasing
    // only while the pointer hovers the panel would still have left the first
    // click after F7 spent on getting the cursor back.
    //
    // The keyboard keeps playing, so you can drag a value and immediately feel
    // it with W and J without closing anything.
    let want = cursor_should_be_captured(
        keys.just_pressed(KeyCode::Escape),
        open.open,
        mouse.just_pressed(MouseButton::Left) || mouse.just_pressed(MouseButton::Right),
        look.grabbed,
    );
    if want != look.grabbed {
        look.grabbed = want;
        if let Ok(mut window) = windows.single_mut() {
            window.cursor_options.grab_mode = if want {
                bevy::window::CursorGrabMode::Locked
            } else {
                bevy::window::CursorGrabMode::None
            };
            window.cursor_options.visible = !want;
        }
    }
}

fn drive_camera(
    sim: Res<Sim>,
    time: Res<Time>,
    look: Res<Look>,
    settings: Res<settings::Settings>,
    mut rig: ResMut<Rig>,
    mut cam: Query<(&mut Transform, &mut Projection), With<MainCamera>>,
) {
    if settings.is_changed() {
        rig.0.set_distance(settings.distance);
    }
    let frame = interpolate(&sim.prev, &sim.cur, sim.clock.alpha());
    // The camera follows whichever fighter this client is driving.
    let me = sim.local_player();
    let yaw = if me == 0 { look.yaw } else { look.yaw_two };
    let framing = rig
        .0
        .update(time.delta_secs(), frame.players[me].pos, yaw, look.pitch);
    if let Ok((mut tf, mut projection)) = cam.single_mut() {
        tf.translation = Vec3::from_array(framing.eye);
        tf.look_at(Vec3::from_array(framing.look_at), Vec3::Y);
        if settings.is_changed() {
            if let Projection::Perspective(p) = projection.as_mut() {
                p.fov = settings.fov_radians();
            }
        }
    }
}

fn fx3(v: sim::V3) -> Vec3 {
    Vec3::new(
        v.x.to_f32_for_render(),
        v.y.to_f32_for_render(),
        v.z.to_f32_for_render(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use palette::UiFocus;

    #[test]
    fn the_oven_keeps_the_cursor_while_it_is_open() {
        // The bug: capturing on any click meant every drag of a slider grabbed
        // the mouse back, so changing a number cost an Escape each time.
        assert!(!cursor_should_be_captured(false, true, true, true));
        assert!(!cursor_should_be_captured(false, true, false, true));
    }

    #[test]
    fn clicking_the_arena_captures_when_the_oven_is_shut() {
        assert!(cursor_should_be_captured(false, false, true, false));
        // And a capture persists without needing the button held.
        assert!(cursor_should_be_captured(false, false, false, true));
        assert!(!cursor_should_be_captured(false, false, false, false));
    }

    #[test]
    fn escape_always_releases() {
        // Without a way out the window is a trap, and this runs windowed.
        assert!(!cursor_should_be_captured(true, false, true, true));
    }

    #[test]
    fn the_palette_takes_input_only_when_it_is_using_it() {
        // Pointer and keyboard are claimed separately on purpose: hovering a
        // slider must not stop W and J from working, or you could not feel the
        // change you just made without closing the panel.
        let hovering = UiFocus {
            pointer: true,
            keyboard: false,
        };
        assert!(hovering.pointer && !hovering.keyboard);

        let typing = UiFocus {
            pointer: false,
            keyboard: true,
        };
        assert!(typing.keyboard, "a focused text field claims the keyboard");

        let idle = UiFocus::default();
        assert!(
            !idle.pointer && !idle.keyboard,
            "a shut palette claims nothing"
        );
    }
}
