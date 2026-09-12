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
//!   H hunt the Ridgeback, or fight the other player
//!
//! Player two: arrows, RCtrl, Period, Comma, Slash, RShift.
//!
//! Pick classes with `--p1 <class> --p2 <class>`, or cycle player one's class
//! in-game with Tab. Names are matched loosely: bulwark, champion, reaver,
//! elementalist, blood, dual.

mod bake;
mod beast;
mod crosshair;
mod debug;
mod hub;
mod hud;
mod palette;
mod settings;

use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use sim::state::MAX_PLAYERS;
use sim::{Input as SimInput, World, arena};
use view::interp::TickClock;
use view::play::{Crossfade, PoseInput};
use view::skeleton::{JOINTS, Joint, Skeleton, skeleton_for};
use view::{CameraRig, aim_from_radians, camera::RigConfig, interpolate, pitch_from_radians};

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

/// Start in a hunt rather than a versus match.
///
/// A flag as well as a key, because the headless screenshot script takes flags
/// and not keystrokes.
fn hunting() -> bool {
    std::env::args().any(|a| a == "--hunt")
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
        .init_resource::<hub::Hub>()
        .init_resource::<Fades>()
        .init_resource::<ShieldHands>()
        .init_resource::<palette::UiFocus>()
        .init_resource::<hud::ShowClassButtons>()
        .add_plugins(bevy_egui::EguiPlugin {
            enable_multipass_for_primary_context: false,
        })
        .insert_resource(settings::Settings::load())
        .init_resource::<InsideOwnHead>()
        .init_resource::<Scripted>()
        .add_systems(Startup, (setup, beast::setup, hud::setup, crosshair::setup))
        .add_systems(
            Update,
            (
                // Who owns the mouse and keyboard this frame, before anything
                // reads them.
                palette::sample_focus,
                hud::sample_button_focus,
                // Mouse look runs next: aim is an input to the tick, not a
                // decoration applied after it.
                mouse_look,
                tick_sim,
                apply_poses,
                place_shields,
                place_effects,
                place_structures,
                beast::place,
                drive_camera,
                fade_own_body,
                hud::toggle_class_buttons,
                hud::class_buttons,
                hud::update,
                hud::update_class_buttons,
                crosshair::update,
                debug::draw,
                beast::overlay,
                palette::toggle,
                palette::draw,
            )
                .chain(),
        )
        // A second tuple only because Bevy's is full: the hub draws last, over
        // everything, and after the poses it is previewing have been placed.
        .add_systems(
            Update,
            (
                hub::toggle,
                hub::advance,
                hub::collect_bake,
                hub::onion_skin,
                hub::draw,
            )
                .chain()
                .after(palette::draw),
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
    /// Show the skeleton at rest instead of animating it.
    ///
    /// Useful rather than decorative: it is how you tell "this clip is wrong"
    /// from "this rig is wrong" while looking at the thing, in one keypress.
    bind_pose: bool,
}

/// Where each fighter's shield hand ended up this frame, in world space.
///
/// The shield is a separate object because its position is independent of the
/// character -- that is the whole mechanic -- but while it is *in hand* it
/// should be in a hand, and a hand is now a thing the skeleton has. Written by
/// the posing pass and read by the one that places shields, which runs after.
#[derive(Resource, Default)]
struct ShieldHands([(Vec3, Quat); MAX_PLAYERS]);

/// One cross-fade per fighter. Renderer-local: a rollback rewinds it to
/// whatever it was, which is wrong by a few frames of blend weight and
/// invisible. See `view::play::Crossfade`.
#[derive(Resource, Default)]
struct Fades([Crossfade; MAX_PLAYERS]);

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
        let w = if hunting() {
            World::hunt(chosen_classes())
        } else {
            World::with_classes(chosen_classes())
        };
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
            // `BIND_POSE=1` starts frozen, so the proportions of a build can be
            // captured without a keypress.
            bind_pose: std::env::var("BIND_POSE").as_deref() == Ok("1"),
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

/// The scripted hunter, when `DEMO=1` is driving a hunt.
///
/// It is the same bot `cargo run -p hunt --bin fight` measures, so what you
/// watch here and what the fight report scores are the same play sequence.
/// Renderer-side state: it produces *inputs*, and inputs are transmitted rather
/// than recomputed, so nothing about it can reach a peer's simulation.
#[derive(Resource)]
struct Scripted(hunt::Hunter);

impl Default for Scripted {
    fn default() -> Self {
        Scripted(hunt::Hunter::new(0))
    }
}

/// Start pitch override, for capturing the camera at a known angle.
fn env_f32(key: &str) -> Option<f32> {
    std::env::var(key).ok()?.parse().ok()
}

/// `--dev` turns everything on at once: hitbox and hurtbox wireframes, and the
/// Oven.
///
/// It exists because that combination *is* the working mode right now, and a
/// mode you reach for every session should not need two keypresses and a
/// reminder of which two.
pub fn dev_mode() -> bool {
    std::env::args().any(|a| a == "--dev")
}

fn env_num(key: &str) -> Option<u32> {
    std::env::var(key).ok()?.parse().ok()
}

/// Where each local player is looking. Renderer-side state: both angles are
/// quantised and handed to the simulation as input, but the floats themselves
/// never cross the wire and never enter a snapshot.
///
/// Pitch goes with the yaw now. It used to live only here, on the grounds that
/// it moved the camera and changed nothing about the fight -- which stopped
/// being true when abilities started landing where the crosshair is.
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
            yaw: env_f32("SHOT_YAW").unwrap_or(0.0),
            // Resting a little below the horizon, not level -- see
            // `Zones::neutral_pitch`.
            pitch: env_f32("SHOT_PITCH").unwrap_or(view::camera::Zones::tuned().neutral_pitch()),
            yaw_two: std::f32::consts::PI,
            grabbed: false,
        }
    }
}

impl Look {
    fn aim(&self) -> u16 {
        aim_from_radians(self.yaw)
    }

    fn tilt(&self) -> i16 {
        pitch_from_radians(self.pitch)
    }

    fn aim_two(&self) -> u16 {
        aim_from_radians(self.yaw_two)
    }
}

#[derive(Resource)]
struct Rig(CameraRig);

/// How far the camera has climbed into the local fighter's head, 0 to 1.
///
/// Lives outside the rig because the renderer needs it and the rig should not
/// know about meshes.
#[derive(Resource, Default)]
struct InsideOwnHead(f32);

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
    joint: Joint,
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct ShieldMesh(usize);

/// One drawable piece of a persistent effect.
///
/// Four per effect, which is what the widest of them needs: a Grasp is four
/// arms and each one is its own skillshot you have to be able to see coming. A
/// fire pillar uses two (a base and a column, drawn apart because they are two
/// different threats), a black spike two (the field on the floor and the spike
/// standing in it), a thrown blade one.
#[derive(Component)]
struct EffectMesh {
    slot: usize,
    part: usize,
}

/// How many pieces one effect can be drawn as.
const EFFECT_PARTS: usize = 4;

/// One of the Elementalist's structures.
#[derive(Component)]
struct StructureMesh {
    owner: usize,
    index: usize,
}

/// Materials for the persistent effects, made once. Which one an entity wears
/// changes as slots are reused, so they are kept rather than rebuilt.
#[derive(Resource)]
struct EffectLook {
    fire: Handle<StandardMaterial>,
    blood: Handle<StandardMaterial>,
    stone: Handle<StandardMaterial>,
    /// A unit cylinder, cone and sphere, scaled per frame to whatever the
    /// simulation says the volume is. Three meshes rather than one because the
    /// *shape* is the tell: a spike you can see standing in a field is what
    /// makes the field something you decide to walk around, and a flat disc on
    /// the floor is something you notice once you are in it.
    column: Handle<Mesh>,
    spike: Handle<Mesh>,
    ball: Handle<Mesh>,
}

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
                // Unit cubes, scaled every frame from whichever build the
                // fighter currently has. Baking the size into the mesh would
                // mean rebuilding sixteen meshes every time somebody presses
                // Tab to change class.
                for joint in JOINTS {
                    root.spawn((
                        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                        MeshMaterial3d(skin.clone()),
                        Transform::default(),
                        BodyPart { owner, joint },
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

    // A fixed pool, one pair of cylinders per effect slot, because the
    // simulation's effect array is itself fixed. Spawning and despawning meshes
    // as effects come and go would put allocation on the rollback path.
    let unit = meshes.add(Cylinder::new(0.5, 1.0));
    let spike = meshes.add(Cone {
        radius: 0.5,
        height: 1.0,
    });
    let ball = meshes.add(Sphere::new(0.5));
    let look = EffectLook {
        // Translucent, not solid -- it is flame, and a wall of solid orange
        // plastic reads as a structure rather than a hazard you could
        // arguably see an opponent through.
        fire: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.45, 0.12, 0.55),
            emissive: LinearRgba::rgb(2.4, 0.8, 0.15),
            perceptual_roughness: 0.9,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        blood: materials.add(StandardMaterial {
            base_color: Color::srgb(0.35, 0.03, 0.09),
            emissive: LinearRgba::rgb(0.5, 0.0, 0.12),
            perceptual_roughness: 0.95,
            ..default()
        }),
        stone: materials.add(StandardMaterial {
            base_color: Color::srgb(0.52, 0.50, 0.47),
            perceptual_roughness: 0.95,
            ..default()
        }),
        column: unit.clone(),
        spike,
        ball,
    };
    for slot in 0..sim::effects::MAX_EFFECTS {
        for part in 0..EFFECT_PARTS {
            commands.spawn((
                Mesh3d(unit.clone()),
                MeshMaterial3d(look.fire.clone()),
                Transform::default(),
                Visibility::Hidden,
                EffectMesh { slot, part },
            ));
        }
    }

    // Structures get their own pool, because they are not effects: they have no
    // clock and they belong to the Elementalist's mechanic, which is the only
    // place that knows about them.
    for owner in 0..MAX_PLAYERS {
        for index in 0..sim::class::MAX_STRUCTURES {
            commands.spawn((
                Mesh3d(unit.clone()),
                MeshMaterial3d(look.stone.clone()),
                Transform::default(),
                Visibility::Hidden,
                StructureMesh { owner, index },
            ));
        }
    }
    commands.insert_resource(look);
}

/// Stop drawing the local fighter once the camera is inside them.
///
/// Past the handover the eye is at the fighter's own eyes, so their head fills
/// the screen and there is nothing to see but the inside of a box.
///
/// **Faded, not hidden.** The rig comes in quickly once the aim crosses the
/// horizon, and a body that popped out at some threshold on the way would read
/// as a rendering fault. Fading it with the climb makes the handover one
/// continuous motion: the fighter rises toward the middle of the screen and
/// thins out as they get there.
///
/// Only ever the fighter this client is driving. The other one is what you are
/// trying to look at.
fn fade_own_body(
    sim: Res<Sim>,
    inside: Res<InsideOwnHead>,
    parts: Query<(&BodyPart, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let me = sim.local_player();
    // The rig has already worked out how much of the body to take away and
    // why -- coming up on the crosshair, or the eye simply being close to it.
    let alpha = (1.0 - inside.0).clamp(0.0, 1.0);
    for (part, material) in parts.iter() {
        if part.owner != me {
            continue;
        }
        let Some(skin) = materials.get_mut(&material.0) else {
            continue;
        };
        if (skin.base_color.alpha() - alpha).abs() < 0.001 {
            continue;
        }
        skin.base_color.set_alpha(alpha);
        // Blending only while it is actually translucent. An always-blended
        // fighter sorts against the other one and against the arena for no
        // reason the rest of the time.
        skin.alpha_mode = if alpha >= 1.0 {
            AlphaMode::Opaque
        } else {
            AlphaMode::Blend
        };
    }
}

/// Put the structure meshes where the Elementalist's mechanic says they are.
fn place_structures(
    sim: Res<Sim>,
    mut meshes: Query<(&StructureMesh, &mut Transform, &mut Visibility)>,
) {
    use sim::class::Mechanic;
    let radius = sim::tuning::structure_radius().to_f32_for_render();
    for (tag, mut tf, mut vis) in meshes.iter_mut() {
        let Mechanic::Structures(slots) = sim.cur.players[tag.owner].mechanic else {
            *vis = Visibility::Hidden;
            continue;
        };
        let Some(raised) = slots[tag.index] else {
            *vis = Visibility::Hidden;
            continue;
        };
        // It is earth: it climbs out of the floor rather than appearing in the
        // air. The whole column slides up from fully buried, so the visible
        // part grows from the ground and the silhouette is always a slab
        // standing on the floor rather than a block hanging in it.
        //
        // Along a **curve**, not a ramp: it holds near the floor -- the beat
        // where the telegraph is readable and someone can still move -- then
        // erupts. Same duration either way; completely different to play
        // against, which is the whole argument for curves over single numbers.
        //
        // The rise comes from the simulation rather than being worked out again
        // here, because it is no longer decoration: it is where the top of the
        // stone is, and the top of the stone is what you can stand on. A
        // renderer that recomputed it could disagree with the surface the game
        // is holding you up with.
        let rise = raised.risen().to_f32_for_render();
        let height = sim::tuning::structure_height().to_f32_for_render();
        *vis = Visibility::Inherited;
        tf.translation = Vec3::new(
            raised.at.x.to_f32_for_render(),
            raised.at.y.to_f32_for_render() + height * (rise - 0.5),
            raised.at.z.to_f32_for_render(),
        );
        tf.scale = Vec3::new(radius * 2.0, height, radius * 2.0);
    }
}

/// Put the effect meshes where the simulation says its effects are.
///
/// Shape comes from the same volumes the hit test uses, so what you see standing
/// in the arena is what will actually catch you. A renderer that reconstructed
/// the shape itself would drift from the rule, and a fire pillar that looks
/// bigger than it hits is worse than no fire pillar.
fn place_effects(
    sim: Res<Sim>,
    look: Res<EffectLook>,
    mut meshes: Query<(
        &EffectMesh,
        &mut Transform,
        &mut Visibility,
        &mut Mesh3d,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    for (tag, mut tf, mut vis, mut mesh, mut mat) in meshes.iter_mut() {
        let piece = sim.cur.effects[tag.slot].and_then(|e| effect_piece(&e, tag.part));
        let Some(piece) = piece else {
            *vis = Visibility::Hidden;
            continue;
        };
        let (want_mesh, want_skin) = (look.mesh(piece.shape), look.material(piece.skin));
        *vis = Visibility::Inherited;
        if mesh.0 != want_mesh {
            mesh.0 = want_mesh;
        }
        if mat.0 != want_skin {
            mat.0 = want_skin;
        }
        tf.translation = piece.at;
        tf.scale = piece.scale;
    }
}

/// One drawable piece of one effect: which shape, which material, where, and
/// how big.
///
/// Shapes and skins by name rather than by asset handle, so the geometry is a
/// pure function of simulation state that a test can check without a renderer.
/// What it is checking is that the picture agrees with the hit test, and that
/// is worth being able to assert: a spike drawn wider than it drains is a
/// promise the game does not keep.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Piece {
    shape: Shape,
    skin: Skin,
    at: Vec3,
    scale: Vec3,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Shape {
    /// An upright cylinder: a pillar, a stone, the floor of a drain field.
    Column,
    /// A cone standing on its base. The black spike, and nothing else yet.
    Spike,
    /// A sphere, which is exactly what the hit test for a travelling effect is.
    Ball,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Skin {
    Fire,
    Blood,
}

impl EffectLook {
    fn mesh(&self, shape: Shape) -> Handle<Mesh> {
        match shape {
            Shape::Column => self.column.clone(),
            Shape::Spike => self.spike.clone(),
            Shape::Ball => self.ball.clone(),
        }
    }

    fn material(&self, skin: Skin) -> Handle<StandardMaterial> {
        match skin {
            Skin::Fire => self.fire.clone(),
            Skin::Blood => self.blood.clone(),
        }
    }
}

/// An upright shape standing on the ground at `at`, between two heights.
fn standing(shape: Shape, skin: Skin, at: Vec3, radius: f32, bottom: f32, top: f32) -> Piece {
    let height = (top - bottom).max(0.01);
    Piece {
        shape,
        skin,
        at: at + Vec3::Y * (bottom + height * 0.5),
        scale: Vec3::new(radius * 2.0, height, radius * 2.0),
    }
}

/// A ball in the air, which is exactly what the hit test for a travelling
/// effect is -- see `World::inside`.
fn floating(at: Vec3, radius: f32) -> Piece {
    Piece {
        shape: Shape::Ball,
        skin: Skin::Blood,
        at,
        scale: Vec3::splat(radius * 2.0),
    }
}

fn effect_piece(effect: &sim::effects::Effect, part: usize) -> Option<Piece> {
    use sim::effects::{EffectKind, GRASP_ARMS};

    let at = fx3(effect.pos);
    match effect.kind {
        EffectKind::FirePillar if part < 2 => {
            let (base, column) = effect.pillar_volumes();
            let it = if part == 0 { base } else { column };
            Some(standing(
                Shape::Column,
                Skin::Fire,
                at,
                it.radius.to_f32_for_render(),
                it.bottom.to_f32_for_render(),
                it.top.to_f32_for_render(),
            ))
        }
        // The field it drains in, and the spike standing in the middle of it.
        // Two pieces because they say two different things: the disc is where
        // the drain reaches, and the spike is the thing you can see from across
        // the arena and decide to walk around. It was drawn as the disc alone
        // for a while, which is a hazard you find out about by standing in it.
        EffectKind::BlackSpike if part < 2 => {
            let volume = effect.spike_volume();
            let radius = volume.radius.to_f32_for_render();
            let height = volume.top.to_f32_for_render();
            Some(if part == 0 {
                standing(Shape::Column, Skin::Blood, at, radius, 0.0, 0.12)
            } else {
                standing(
                    Shape::Spike,
                    Skin::Blood,
                    at,
                    radius * SPIKE_WAIST,
                    0.0,
                    height,
                )
            })
        }
        EffectKind::Bloodletter if part == 0 => Some(floating(
            fx3(effect.blade_at()),
            effect.field_radius().to_f32_for_render(),
        )),
        EffectKind::Grasp if part < GRASP_ARMS => Some(floating(
            fx3(effect.arm_at(part)),
            effect.field_radius().to_f32_for_render(),
        )),
        _ => None,
    }
}

/// How fat the spike is against the field it stands in.
///
/// Presentation, not a rule: the field's radius is where the drain reaches, and
/// a cone that wide would be a tent rather than a spike. The disc underneath is
/// what tells you where the edge is.
const SPIKE_WAIST: f32 = 0.35;

/// Where the class mechanic sits in the world, if anywhere. A shield in hand
/// rides on the character and draws nothing; a thrown one, a placed shadow or a
/// raised structure all get a marker.
fn mechanic_world_pos(m: &sim::class::Mechanic) -> Option<sim::V3> {
    use sim::class::Mechanic;
    match m {
        Mechanic::Shield(s) => s.world_pos(),
        Mechanic::Shadow { at } => *at,
        Mechanic::Structures(slots) => slots.iter().flatten().next().map(|s| s.at),
        _ => None,
    }
}

/// A shield in hand rides on the character; a thrown one sits in the world.
fn place_shields(
    sim: Res<Sim>,
    hands: Res<ShieldHands>,
    mut shields: Query<(&ShieldMesh, &mut Transform, &mut Visibility)>,
) {
    for (tag, mut tf, mut vis) in shields.iter_mut() {
        let held = matches!(
            sim.cur.players[tag.0].mechanic,
            sim::Mechanic::Shield(sim::state::Shield::Held)
        );
        match mechanic_world_pos(&sim.cur.players[tag.0].mechanic) {
            // Thrown or planted: it is somewhere in the arena on its own.
            Some(pos) => {
                *vis = Visibility::Inherited;
                tf.rotation = Quat::IDENTITY;
                tf.translation = Vec3::new(
                    pos.x.to_f32_for_render(),
                    pos.y.to_f32_for_render(),
                    pos.z.to_f32_for_render(),
                );
            }
            // In hand, and now that the skeleton has hands it can be in one.
            // It follows the forearm, the way a strapped shield does, so
            // raising the guard raises the shield without anybody animating it.
            None if held => {
                let (at, rot) = hands.0[tag.0];
                *vis = Visibility::Inherited;
                tf.rotation = rot;
                tf.translation = at + rot * Vec3::new(0.0, -0.12, 0.06);
            }
            None => *vis = Visibility::Hidden,
        }
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Read real input, run whole fixed ticks, keep the previous snapshot.
// A Bevy system's parameter list *is* its dependency declaration: every entry
// is something the scheduler has to know this system touches. Splitting one to
// get under a count would split the system, which is the opposite of the point.
#[allow(clippy::too_many_arguments)]
fn tick_sim(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    look: Res<Look>,
    focus: Res<palette::UiFocus>,
    mut sim: ResMut<Sim>,
    mut show: ResMut<debug::ShowDebug>,
    mut scripted: ResMut<Scripted>,
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
        sim.bind_pose = !sim.bind_pose;
    }
    if keys.just_pressed(KeyCode::Tab) {
        // Cycle player one's class. Restarts the match, since a class change
        // mid-round would leave the mechanic in someone else's state.
        let next = (sim.cur.players[0].class as usize + 1) % ALL.len();
        let classes = [ALL[next], sim.cur.players[1].class];
        let w = if sim.cur.monster.is_some() {
            World::hunt(classes)
        } else {
            World::with_classes(classes)
        };
        sim.prev = w.clone();
        sim.cur = w;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        let classes = [sim.cur.players[0].class, sim.cur.players[1].class];
        let w = if sim.cur.monster.is_some() {
            World::hunt(classes)
        } else {
            World::with_classes(classes)
        };
        sim.prev = w.clone();
        sim.cur = w;
    }
    // Swap between hunting something and fighting each other. A restart either
    // way, because a creature appearing in the middle of a round would land on
    // somebody.
    if keys.just_pressed(KeyCode::KeyH) {
        let classes = [sim.cur.players[0].class, sim.cur.players[1].class];
        let w = if sim.cur.monster.is_some() {
            World::with_classes(classes)
        } else {
            World::hunt(classes)
        };
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
    .looking(look.aim(), look.tilt());
    let held_two = if focus.keyboard {
        SimInput::default()
    } else {
        read_player_two(&keys)
    }
    // Player two has no mouse, so they aim level.
    .looking(look.aim_two(), 0);

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
                let scripted = demo_mode().then(|| script(&mut scripted.0, &sim.cur));
                let pair = [
                    scripted_or(scripted, held),
                    dummy_input(sim.dummy, sim.cur.frame, held_two),
                ];
                sim.prev = sim.cur.clone();
                sim.cur.advance(pair);
            }
        }
        Driver::Online { .. } => {
            let ticks = sim.clock.advance(time.delta_secs());
            for _ in 0..ticks {
                let scripted = demo_mode().then(|| script(&mut scripted.0, &sim.cur));
                let local = scripted_or(scripted, held);
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

/// One frame of scripted play.
///
/// Against another fighter it is the fixed beat below, which exists to exercise
/// posing and framing. Against a creature it is the real hunter, because a
/// fixed beat played at a monster would be a demonstration of nothing.
fn script(hunter: &mut hunt::Hunter, w: &sim::World) -> SimInput {
    if w.monster.is_some() {
        hunter.watch(w);
        return hunter.act(w);
    }
    demo_input(w, w.frame)
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
        280..=283 => v |= SimInput::MECHANIC,               // throw the shield
        300..=303 => v |= SimInput::SPECIAL,                // the class special
        340..=343 => v |= SimInput::MECHANIC,               // recall it
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
        v |= SimInput::SPECIAL;
    }
    if keys.pressed(KeyCode::KeyL) {
        v |= SimInput::MECHANIC;
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
    // Q and E, not a chord on a click. The special and the mechanic are the
    // two things a class does that nothing else does; burying them under a
    // modifier made them feel optional.
    if keys.pressed(KeyCode::KeyQ) {
        v |= SimInput::SPECIAL;
    }
    if keys.pressed(KeyCode::KeyE) || mouse.pressed(MouseButton::Middle) {
        v |= SimInput::MECHANIC;
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

/// Place every bone from the interpolated snapshot.
///
/// Pose is a pure function of simulation state -- see `view::play`. Nothing
/// here accumulates animation time, which is what lets a rollback rewind the
/// characters without them sliding.
fn apply_poses(
    sim: Res<Sim>,
    time: Res<Time>,
    mut fades: ResMut<Fades>,
    mut hands: ResMut<ShieldHands>,
    hub: Option<Res<crate::hub::Hub>>,
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

    // One solve per fighter rather than one per bone: forward kinematics is a
    // single pass down the skeleton and there are sixteen bones hanging off it.
    let mut skeletons: [Skeleton; MAX_PLAYERS] = [
        skeleton_for(sim.cur.players[0].class),
        skeleton_for(sim.cur.players[1].class),
    ];
    let mut skins = Vec::with_capacity(MAX_PLAYERS);
    for (owner, skeleton) in skeletons.iter_mut().enumerate() {
        let p = frame.players[owner];
        let class = sim.cur.players[owner].class;
        let mut input = PoseInput::of(&p, class, &frame);
        input.bind_pose = sim.bind_pose;
        let mut pose = fades.0[owner].pose(input, time.delta_secs());
        // The hub takes over whichever fighter it is previewing, so an edit is
        // visible on a real character in the real arena rather than in a
        // separate viewer that flatters it.
        if let Some(hub) = hub.as_ref() {
            if let Some(preview) = hub.preview_for(owner) {
                pose = preview;
            }
        }
        skins.push(view::skeleton::solve(skeleton, &pose));
    }

    // The shield hand, in the arena rather than in the character's own space,
    // so whatever is holding something can be placed against it.
    for owner in 0..MAX_PLAYERS {
        let p = frame.players[owner];
        let yaw = p.facing[0].atan2(p.facing[2]);
        let turn = Quat::from_rotation_y(yaw);
        let (at, rot) = skins[owner].box_of(&skeletons[owner], Joint::HandL);
        hands.0[owner] = (
            Vec3::new(p.pos[0], p.pos[1], p.pos[2]) + turn * Vec3::new(at[0], at[1], at[2]),
            turn * Quat::from_xyzw(rot.0[0], rot.0[1], rot.0[2], rot.0[3]),
        );
    }

    for (bp, mut tf) in parts.iter_mut() {
        let skeleton = &skeletons[bp.owner];
        let (centre, rot) = skins[bp.owner].box_of(skeleton, bp.joint);
        let size = view::pose::part_size(skeleton, bp.joint);
        tf.translation = Vec3::new(centre[0], centre[1], centre[2]);
        tf.rotation = Quat::from_xyzw(rot.0[0], rot.0[1], rot.0[2], rot.0[3]);
        tf.scale = Vec3::new(size[0], size[1], size[2]);
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
///
/// The Oven and the arena share a window, so the cursor has to move between
/// them without ceremony:
///
/// - **Opening the Oven hands the cursor back**, so the first click after F7
///   lands on a widget instead of being spent getting the pointer released.
/// - **Clicking the arena takes it again**, even with the Oven open — that is
///   how you go and feel the change you just made.
/// - **Escape returns it to the Oven**, and is the way out of a captured
///   window whether the Oven is open or not.
///
/// A click *on the panel* is not a click at the arena, which is the whole
/// original bug: every drag of a slider was re-grabbing the mouse.
fn cursor_should_be_captured(
    escape: bool,
    oven_just_opened: bool,
    clicked_in_arena: bool,
    captured: bool,
) -> bool {
    if escape || oven_just_opened {
        false
    } else if clicked_in_arena {
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
    mut focus: ResMut<palette::UiFocus>,
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
    ] {
        if keys.just_pressed(key) {
            settings.nudge(knob, up);
            changed = true;
        }
    }
    if changed {
        settings.save();
    }

    if look.grabbed {
        let sensitivity = settings.radians_per_pixel();
        let (mut dx, mut dy) = (0.0, 0.0);
        for ev in motion.read() {
            dx += ev.delta.x;
            dy += ev.delta.y;
        }
        look.yaw += dx * sensitivity;
        let zones = view::camera::Zones::tuned();
        look.pitch = (look.pitch - dy * sensitivity).clamp(-zones.down_limit, zones.up_limit);
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
    let clicked = mouse.just_pressed(MouseButton::Left) || mouse.just_pressed(MouseButton::Right);
    let want = cursor_should_be_captured(
        keys.just_pressed(KeyCode::Escape),
        std::mem::take(&mut focus.just_opened),
        clicked && !focus.pointer,
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
    mut inside: ResMut<InsideOwnHead>,
    mut cam: Query<(&mut Transform, &mut Projection), With<MainCamera>>,
) {
    if settings.is_changed() {
        rig.0.set_fov(settings.fov_radians());
    }
    let frame = interpolate(&sim.prev, &sim.cur, sim.clock.alpha());
    // The camera follows whichever fighter this client is driving.
    let me = sim.local_player();
    let yaw = if me == 0 { look.yaw } else { look.yaw_two };
    let framing = rig.0.update_around(
        time.delta_secs(),
        frame.players[me].pos,
        yaw,
        look.pitch,
        view::Surroundings {
            beast: sim.cur.monster.as_ref(),
            aboard: sim.cur.players[me].aboard(),
        },
    );
    inside.0 = framing.hidden;
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

/// A simulation position, in the renderer's units.
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
    use sim::effects::{Effect, EffectKind, GRASP_ARMS};

    fn cast(kind: EffectKind, slot: u8) -> Effect {
        Effect::cast(
            kind,
            0,
            sim::Class::BloodMage,
            slot,
            sim::V3::ZERO,
            sim::V3::new(sim::Fx::ONE, sim::Fx::ZERO, sim::Fx::ZERO),
        )
    }

    #[test]
    fn a_black_spike_is_drawn_with_a_spike_in_it() {
        // It was a twelve-centimetre stain on the floor for a while, which is a
        // hazard you find out about by standing in it. The field is the disc;
        // the spike is what you can see from across the arena.
        let effect = cast(EffectKind::BlackSpike, sim::state::SLOT_MECHANIC);
        let field = effect_piece(&effect, 0).expect("the field is drawn");
        let spike = effect_piece(&effect, 1).expect("the spike is drawn");
        assert_eq!(field.shape, Shape::Column);
        assert_eq!(spike.shape, Shape::Spike);
        assert!(
            spike.scale.y > field.scale.y * 4.0,
            "the spike is no taller than the stain it stands in"
        );
        assert!(
            spike.scale.x < field.scale.x,
            "the spike is as wide as the whole field, which is a tent"
        );
    }

    #[test]
    fn the_drawn_spike_is_exactly_as_tall_as_the_volume_that_drains() {
        // The rule for every effect in the game: what you see is what catches
        // you. A field drawn shorter than it tests would be a hazard you think
        // you jumped over.
        let effect = cast(EffectKind::BlackSpike, sim::state::SLOT_MECHANIC);
        let volume = effect.spike_volume();
        let spike = effect_piece(&effect, 1).expect("the spike is drawn");
        assert!(
            (spike.scale.y - volume.top.to_f32_for_render()).abs() < 0.001,
            "drawn {} tall, drains up to {}",
            spike.scale.y,
            volume.top.to_f32_for_render()
        );
        let field = effect_piece(&effect, 0).expect("the field is drawn");
        assert!(
            (field.scale.x * 0.5 - volume.radius.to_f32_for_render()).abs() < 0.001,
            "the drawn field is not the width of the drained one"
        );
    }

    #[test]
    fn a_thrown_blade_is_drawn_where_it_actually_is() {
        // The blade is the whole threat -- the move leaves no hitbox on the
        // caster -- so a picture of it anywhere but its own position would be
        // the only thing telling the other player where the danger is, lying.
        let mut effect = cast(EffectKind::Bloodletter, sim::state::SLOT_POKE);
        let mut furthest = 0.0f32;
        for _ in 0..effect.life {
            effect.age += 1;
            let drawn = effect_piece(&effect, 0).expect("the blade is drawn");
            assert_eq!(drawn.at, fx3(effect.blade_at()));
            furthest = furthest.max(drawn.at.x);
        }
        assert!(furthest > 1.0, "the blade never went anywhere");
        assert!(
            effect_piece(&effect, 1).is_none(),
            "a blade is one object, not two"
        );
    }

    #[test]
    fn all_four_arms_of_a_grasp_are_drawn() {
        // Each one is its own skillshot and each one has to be seen coming --
        // it is being caught by *all* of them that roots you, and a player who
        // can only see two cannot tell where the fourth is going to be.
        let mut effect = cast(EffectKind::Grasp, sim::state::SLOT_SPECIAL);
        effect.age = effect.life / 2;
        let drawn: Vec<Piece> = (0..GRASP_ARMS)
            .map(|arm| effect_piece(&effect, arm).expect("every arm is drawn"))
            .collect();
        for (arm, piece) in drawn.iter().enumerate() {
            assert_eq!(piece.at, fx3(effect.arm_at(arm)));
        }
        // Halfway through, the cone is open: no two arms are in the same place.
        for a in 0..GRASP_ARMS {
            for b in a + 1..GRASP_ARMS {
                assert!(
                    drawn[a].at.distance(drawn[b].at) > 0.1,
                    "arms {a} and {b} are drawn on top of each other"
                );
            }
        }
    }

    #[test]
    fn opening_the_oven_hands_the_cursor_back() {
        // Otherwise the first click after F7 is spent releasing the pointer
        // rather than landing on the slider you were reaching for.
        assert!(!cursor_should_be_captured(false, true, true, true));
    }

    #[test]
    fn clicking_the_panel_does_not_take_the_cursor() {
        // The original bug: every drag of a slider re-grabbed the mouse, so
        // changing a number cost an Escape each time. A click the palette is
        // claiming arrives here as `clicked_in_arena == false`.
        assert!(!cursor_should_be_captured(false, false, false, false));
    }

    #[test]
    fn clicking_the_arena_takes_it_again_even_with_the_oven_open() {
        // How you go and feel the change you just made, without closing
        // anything. `oven_just_opened` is only true on the frame F7 is pressed.
        assert!(cursor_should_be_captured(false, false, true, false));
        // And a capture persists without needing the button held.
        assert!(cursor_should_be_captured(false, false, false, true));
    }

    #[test]
    fn escape_always_returns_it() {
        // Back to the Oven when it is open, and out of a captured window when
        // it is not. Without a way out the window is a trap, and this runs
        // windowed on a desktop.
        assert!(!cursor_should_be_captured(true, false, true, true));
        assert!(!cursor_should_be_captured(true, true, true, true));
    }

    #[test]
    fn the_palette_takes_input_only_when_it_is_using_it() {
        // Pointer and keyboard are claimed separately on purpose: hovering a
        // slider must not stop W and J from working, or you could not feel the
        // change you just made without closing the panel.
        let hovering = UiFocus {
            pointer: true,
            ..UiFocus::default()
        };
        assert!(hovering.pointer && !hovering.keyboard);

        let typing = UiFocus {
            keyboard: true,
            ..UiFocus::default()
        };
        assert!(typing.keyboard, "a focused text field claims the keyboard");

        let idle = UiFocus::default();
        assert!(
            !idle.pointer && !idle.keyboard,
            "a shut palette claims nothing"
        );
    }
}
