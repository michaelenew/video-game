//! **The beings beside the Dual mage, and the seal they rise from**
//! (`view::being`): her two bars, drawn.
//!
//! Each being is its own model -- the obsidian man, the knight -- under a
//! root of its own ([`BeingRoot`]), standing to her side (the dark one's
//! centre line to her left, the light one's to her right), twice her size
//! and sunk below her floor so its shoulders are at hers, posed by its own animations off the simulation's
//! frame (`view::being::pose`): it strains at its binding, and her auto is
//! its blow. It is drawn as its painted self, veiled (`still::Veil`,
//! `skin.wgsl`): only its outer half, fading into its own middle; only as
//! much of that as its bar has filled, the shoulder and fist first, the head
//! last; and nothing of it below the seal it comes out of.
//!
//! The seal is a disc round her, at her waist or her feet
//! (`view::being::SEAL_AT`), split down her middle: dark half on her left,
//! light on her right, a faint ring whatever the bars, and each half's runes
//! lit round as its bar fills.
//!
//! Both are presentation and nothing else, and both are functions of the
//! snapshot: a rollback redraws them exactly.

use bevy::asset::weak_handle;
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use sim::state::MAX_PLAYERS;
use view::skeleton::{Joint, Skeleton, skeleton_for};

const SHADER: Handle<Shader> = weak_handle!("b3e1c8d4-6f2a-4a97-8c15-7d40e9a2b6f1");

pub use look::SpiritLook;

/// The uniform on its own, for the reason `ink::look` gives.
mod look {
    #![allow(dead_code)]
    use bevy::prelude::*;
    use bevy::render::render_resource::ShaderType;

    /// Everything [`super::Spirit`] -- the seal -- needs, in the arena's
    /// coordinates. (Its body fields are the veil's before the beings were
    /// drawn as their painted selves, `still::Veil`; the seal reads `fill`,
    /// `left`, `feet` and `reach`.)
    #[derive(ShaderType, Reflect, Debug, Clone, Copy, Default)]
    pub struct SpiritLook {
        /// How full the dark half is, the light half, whether she is
        /// ascending (0 or 1), and which this is: 0 the body, 1 the seal.
        pub fill: Vec4,
        /// Unit, toward her left -- the dark side -- and the being's size
        /// against hers.
        pub left: Vec4,
        /// Her feet, which are the being's hips and the seal's middle, and
        /// the seal's radius.
        pub feet: Vec4,
        /// The being's shoulders and fists, dark side then light, and the
        /// middle of its head.
        pub dark_shoulder: Vec4,
        pub dark_fist: Vec4,
        pub light_shoulder: Vec4,
        pub light_fist: Vec4,
        pub head: Vec4,
        /// `view::being::ORDER`: first, body, head, soft.
        pub order: Vec4,
        /// `view::being::REACH`; and 0, or the being drawn whole in one
        /// force, 1 dark and 2 light (`SHOT_BEING`).
        pub reach: Vec4,
    }
}

/// The seal's material: see the module.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct Spirit {
    #[uniform(0)]
    pub look: SpiritLook,
}

impl Material for Spirit {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(SHADER)
    }

    /// Blended, so it never hides the fight: she is seen through it, and
    /// so is everything behind her. Front faces only (the default), so the
    /// body is one veil and not two.
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

/// The root one of a seat's beings hangs off (the seat, then 0 dark or 1
/// light): placed behind her, scaled to its own skeleton, and shown while
/// she has a bar to show.
#[derive(Component)]
pub struct BeingRoot(pub usize, pub usize);

/// A seat's seal, on the floor round her feet.
#[derive(Component)]
pub struct Seal(pub usize);

/// Each seat's seal, and the numbers worked out once from her skeleton.
#[derive(Resource)]
pub struct Beings {
    pub seal: [Handle<Spirit>; MAX_PLAYERS],
    /// Each being's own skeleton (`view::being::skeleton`) and how many times
    /// its size it is drawn.
    skeletons: [Skeleton; 2],
    scales: [f32; 2],
    /// How far below her floor each stands, so its shoulders are at hers.
    sinks: [f32; 2],
    /// Her size against the reference body, which the being's distances
    /// (`view::being::SIDE`, ...) are written in.
    her_scale: f32,
    seal_radius: f32,
    /// How high the seal is off her floor.
    seal_height: f32,
    /// A shaft of light out of one of the dark being's rifts (`beam`): a
    /// unit frustum, narrow at the crack, and the light it is drawn in.
    pub beam_mesh: Handle<Mesh>,
    pub beam_light: Handle<StandardMaterial>,
}

/// One of a seat's beings' painted materials (the seat, then 0 dark or 1
/// light): the veil `place` sets on it each frame.
#[derive(Component)]
pub struct BeingSkin(pub usize, pub usize);

pub struct BeingPlugin;

impl Plugin for BeingPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<Assets<Shader>>().insert(
            SHADER.id(),
            Shader::from_wgsl(include_str!("being.wgsl"), "game/src/being.wgsl"),
        );
        app.add_plugins(MaterialPlugin::<Spirit>::default())
            .add_systems(PreStartup, setup)
            .add_systems(Startup, spawn);
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<Spirit>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut lights: ResMut<Assets<StandardMaterial>>,
) {
    let her = skeleton_for(sim::Class::DualMage);
    let skeletons = [0, 1].map(view::being::skeleton);
    let scales = [0, 1].map(|k| view::being::scale(&her, &skeletons[k]));
    let sinks = [0, 1].map(|k| view::being::sink(&her, &skeletons[k]));
    let seal_radius = view::being::seal_radius(&her);
    let seal = std::array::from_fn(|_| {
        materials.add(Spirit {
            look: SpiritLook {
                fill: Vec4::new(0.0, 0.0, 0.0, 1.0),
                ..default()
            },
        })
    });
    commands.insert_resource(Beings {
        seal,
        skeletons,
        scales,
        sinks,
        her_scale: her.scale,
        seal_radius,
        seal_height: view::being::seal_height(&her, seal_at()),
        beam_mesh: meshes.add(beam_mesh()),
        beam_light: lights.add(StandardMaterial {
            base_color: Color::LinearRgba(LinearRgba::rgb(0.7, 0.35, 1.0) * BEAM_LIGHT),
            unlit: true,
            alpha_mode: AlphaMode::Add,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
    });
}

/// A root and a seal per seat, hidden until a Dual mage needs them: the
/// class is picked at runtime and can change mid-match with Tab. The being's
/// model is made under its root with hers (`models::spawn`).
fn spawn(mut commands: Commands, beings: Res<Beings>, mut meshes: ResMut<Assets<Mesh>>) {
    let disc = meshes.add(Plane3d::new(Vec3::Y, Vec2::ONE).mesh().subdivisions(0));
    for owner in 0..MAX_PLAYERS {
        for k in 0..2 {
            commands.spawn((
                BeingRoot(owner, k),
                Transform::default(),
                Visibility::Hidden,
            ));
        }
        commands.spawn((
            Seal(owner),
            Mesh3d(disc.clone()),
            MeshMaterial3d(beings.seal[owner].clone()),
            Transform::default(),
            Visibility::Hidden,
            NotShadowCaster,
            NotShadowReceiver,
        ));
    }
}

/// `SHOT_BEING=dark` or `SHOT_BEING=light`: one of the two beings on its
/// own, whole, painted and standing on her floor rather than rising out of
/// the seal -- how each is looked at before they are halved. Read once.
fn looked_at() -> f32 {
    static WHOLE: std::sync::OnceLock<f32> = std::sync::OnceLock::new();
    *WHOLE.get_or_init(|| match crate::platform::env("SHOT_BEING").as_deref() {
        Some("dark") => 1.0,
        Some("light") => 2.0,
        _ => 0.0,
    })
}

/// Whether one being is being looked at on its own (`SHOT_BEING`).
pub fn whole() -> bool {
    looked_at() > 0.0
}

/// `SHOT_CLAY=1`, with `SHOT_BEING`: the being drawn in plain grey clay,
/// no colour, glow or gloss -- its form alone, to set beside a scan of a
/// statue in the same light. Read once.
pub fn clay() -> bool {
    static CLAY: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *CLAY.get_or_init(|| crate::platform::env("SHOT_CLAY").is_some())
}

/// Whether being `k` (0 dark, 1 light) is drawn: both, unless the other is
/// being looked at on its own.
pub fn shows(k: usize) -> bool {
    let w = looked_at();
    w == 0.0 || w == (k + 1) as f32
}

/// How bright each being's painted glow is, in the emissive's units, dark
/// then light: the titan's cracks and heart, the seraph's sun and the light
/// it gives off. Bright enough to read as light, short of where the
/// tonemapper takes a colour to white.
pub const GLOW: [f32; 2] = [30.0, 5.0];

/// How bright a rift's shaft of light is at the crack, in the unlit
/// colour's units.
const BEAM_LIGHT: f32 = 0.45;

/// A shaft of light: a frustum along `+y` from the origin to one, a fifth
/// as wide at the crack as at its end, fading to nothing along its length
/// (in its colour, which an additive light reads as its strength).
fn beam_mesh() -> Mesh {
    let mut mesh = Mesh::from(ConicalFrustum {
        radius_top: 1.0,
        radius_bottom: 0.2,
        height: 1.0,
    });
    if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(at)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    {
        for p in at.iter_mut() {
            p[1] += 0.5;
        }
    }
    let fades: Vec<[f32; 4]> = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(bevy::render::mesh::VertexAttributeValues::Float32x3(at)) => at
            .iter()
            .map(|p| {
                let f = (1.0 - p[1]).clamp(0.0, 1.0).powf(1.6);
                [f, f, f, f]
            })
            .collect(),
        _ => Vec::new(),
    };
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, fades);
    mesh
}

/// Put a shaft of light on a rift: a child of the marker at the crack
/// (`from`), pointing at the marker the shaft reaches (`to`, both in the
/// joint's space, as the model's file places them).
pub fn beam(commands: &mut Commands, beings: &Beings, from: (Entity, Vec3), to: Vec3) {
    let along = to - from.1;
    let length = along.length().max(0.01);
    let turn = Quat::from_rotation_arc(Vec3::Y, along / length);
    let wide = length * 0.1;
    commands.entity(from.0).with_children(|c| {
        c.spawn((
            Mesh3d(beings.beam_mesh.clone()),
            MeshMaterial3d(beings.beam_light.clone()),
            Transform::from_rotation(turn).with_scale(Vec3::new(wide, length, wide)),
            NotShadowCaster,
            NotShadowReceiver,
        ));
    });
}

/// Each being's reflections (`still::Lustre`), dark then light: the
/// obsidian a mirror-black glass with violet at its edges, the knight's
/// plate polished steel. The world has nothing for them to reflect.
pub const LUSTRE: [crate::still::Lustre; 2] = [
    crate::still::Lustre {
        gloss: Vec4::new(1700.0, 4.0, 0.0, 0.0),
        sheen: Vec4::new(70.0, 25.0, 200.0, 4.5),
    },
    crate::still::Lustre {
        gloss: Vec4::new(3800.0, 3.0, 0.0, 0.0),
        sheen: Vec4::new(110.0, 110.0, 130.0, 4.0),
    },
];

/// How high off the floor the seal is laid, so it never fights the floor
/// for the same depth.
const SEAL_LIFT: f32 = 0.025;

/// Each force's colour at a being's condensing edge, dark then light, and
/// how bright it is, in the lights' units (as `still::Lustre`'s).
const EDGE: [Vec4; 2] = [
    Vec4::new(0.62, 0.30, 1.25, 260.0),
    Vec4::new(1.35, 1.15, 0.72, 200.0),
];

/// Where the seal is: `view::being::SEAL_AT`, or `SHOT_SEAL=feet` or
/// `waist` to look at the other. Read once.
pub fn seal_at() -> view::being::SealAt {
    match crate::platform::env("SHOT_SEAL").as_deref() {
        Some("feet") => view::being::SealAt::Feet,
        Some("waist") => view::being::SealAt::Waist,
        _ => view::being::SEAL_AT,
    }
}

/// Stand each seat's beings beside her, pose each by its own animation,
/// lay her seal, and tell each being's veil how much of it there is.
///
/// Her feet and facing come off the blended frame, which is where her body
/// is drawn; the bars and her move off the current snapshot.
#[allow(clippy::type_complexity, clippy::too_many_arguments)] // a Bevy system: its arguments are what it reads
pub fn place(
    sim: Res<crate::Sim>,
    rigs: Res<crate::models::Rigs>,
    beings: Res<Beings>,
    mut materials: ResMut<Assets<Spirit>>,
    mut skins: ResMut<Assets<crate::still::SkinMaterial>>,
    painted: Query<(&BeingSkin, &MeshMaterial3d<crate::still::SkinMaterial>)>,
    mut roots: Query<(&BeingRoot, &mut Transform, &mut Visibility), Without<Seal>>,
    mut seals: Query<(&Seal, &mut Transform, &mut Visibility), Without<BeingRoot>>,
    mut bones: Query<&mut Transform, (Without<BeingRoot>, Without<Seal>, Without<crate::Fighter>)>,
) {
    use crate::still::Veil;
    let frame = view::interpolate(&sim.prev, &sim.cur, sim.clock.alpha());
    let whole = looked_at();
    let hs = beings.her_scale;
    let mut veils: [[Option<Veil>; 2]; MAX_PLAYERS] = [[None; 2]; MAX_PLAYERS];
    let mut stands: [[Option<(Vec3, Quat)>; 2]; MAX_PLAYERS] = [[None; 2]; MAX_PLAYERS];
    let mut seal_looks: [Option<SpiritLook>; MAX_PLAYERS] = [None; MAX_PLAYERS];
    for owner in 0..MAX_PLAYERS {
        let p = &sim.cur.players[owner];
        let Some(being) = view::being::being(p).filter(|_| sim.cur.is_present(owner)) else {
            continue;
        };
        let moment = view::being::moment(p, sim.cur.frame);
        let v = frame.players[owner];
        let feet = Vec3::from_array(v.pos);
        let q = view::body_turn([v.facing[0], v.facing[2]]).0;
        let turn = Quat::from_xyzw(q[0], q[1], q[2], q[3]);
        // Character space has the facing at +Z, and a drawn body's left at
        // +X (`view::hand_joint` says why): the dark side.
        let left = turn * Vec3::X;
        let seal_y = feet.y + beings.seal_height;
        let o = view::being::ORDER;
        for k in 0..2 {
            let skeleton = &beings.skeletons[k];
            let s = beings.scales[k];
            let skin = view::skeleton::solve(skeleton, &view::being::pose(k, &moment));
            crate::models::pose(rigs.being[owner][k], Some(&skin), &mut bones);
            // Toward the half it keeps: the dark being's left, the light's
            // right -- away from her, either side.
            let out = if k == 0 { 1.0 } else { -1.0 };
            let side = if whole > 0.0 {
                0.0
            } else {
                out * view::being::SIDE * hs
            };
            // Sunk so its shoulders are at hers; or, looked at whole,
            // standing on her floor.
            let sink = if whole > 0.0 { 0.0 } else { beings.sinks[k] };
            let root =
                feet + turn * Vec3::new(side, 0.0, -view::being::BEHIND * hs) - Vec3::Y * sink;
            stands[owner][k] = Some((root, turn));
            let at = |local: [f32; 3]| root + turn * (Vec3::from_array(local) * s);
            let outer = k == 0;
            let shoulder = at(skin.origin[view::drawn_joint(if outer {
                view::skeleton::Joint::ArmL
            } else {
                view::skeleton::Joint::ArmR
            })
            .index()]);
            let fist = at(skin.box_of(skeleton, view::hand_joint(outer)).0);
            let head = at(skin.box_of(skeleton, Joint::Head).0);
            veils[owner][k] = Some(Veil {
                fill: Vec4::new(
                    being.of(if k == 0 {
                        sim::class::Force::Dark
                    } else {
                        sim::class::Force::Light
                    }),
                    if being.ascending { 1.0 } else { 0.0 },
                    if whole > 0.0 { 0.0 } else { 1.0 },
                    s,
                ),
                centre: (root + Vec3::Y * sink).extend(view::being::FADE * hs),
                keep: (left * out).extend(seal_y),
                shoulder: shoulder.extend(view::being::REACH * hs),
                fist: fist.extend(0.0),
                head: head.extend(0.16 * s),
                order: Vec4::new(o.first, o.body, o.head, o.soft),
                glow: EDGE[k],
            });
        }
        seal_looks[owner] = Some(SpiritLook {
            fill: Vec4::new(
                being.dark,
                being.light,
                if being.ascending { 1.0 } else { 0.0 },
                1.0,
            ),
            left: left.extend(1.0),
            feet: (feet + Vec3::Y * beings.seal_height).extend(beings.seal_radius),
            order: Vec4::new(o.first, o.body, o.head, o.soft),
            reach: Vec4::new(view::being::REACH, whole, 0.0, 0.0),
            ..default()
        });
    }

    for (root, mut tf, mut seen) in roots.iter_mut() {
        let (owner, k) = (root.0, root.1);
        let shown =
            veils[owner][k].is_some_and(|v| v.fill.x > 0.0 || v.fill.y > 0.0 || v.fill.z < 0.5);
        let want = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *seen != want {
            *seen = want;
        }
        if let Some((at, turn)) = stands[owner][k] {
            tf.translation = at;
            tf.rotation = turn;
            tf.scale = Vec3::splat(beings.scales[k]);
        }
    }
    for (seal, mut tf, mut seen) in seals.iter_mut() {
        let look = seal_looks[seal.0].filter(|_| whole == 0.0);
        let want = if look.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *seen != want {
            *seen = want;
        }
        if let Some(look) = look {
            tf.translation = look.feet.truncate() + Vec3::Y * SEAL_LIFT;
            tf.scale = Vec3::new(beings.seal_radius, 1.0, beings.seal_radius);
            if let Some(m) = materials.get_mut(&beings.seal[seal.0]) {
                m.look = look;
            }
        }
    }
    for (tag, handle) in painted.iter() {
        let Some(veil) = veils[tag.0][tag.1] else {
            continue;
        };
        if let Some(m) = skins.get_mut(&handle.0) {
            m.extension.veil = veil;
        }
    }
}
