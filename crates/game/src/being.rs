//! **The being behind the Dual mage, and the seal it rises from**
//! (`view::being`, 2026-10-25): her two bars, drawn.
//!
//! The being is her own model a second time, under a root of its own
//! ([`BeingRoot`]) that stands it behind her at [`view::being::scale`] times
//! her size with its hips at her feet, posed with her pose from
//! [`view::being::LAG_SECONDS`] ago ([`Echo`]) while she casts, and in its
//! own stance, arms down, between her casts (`view::being::stance`) -- so it
//! throws what she throws, larger and a beat behind. It is drawn in [`Spirit`], which is nothing of
//! her costume: a translucent body, dark on her left and light on her right,
//! each half there as far as its bar has filled, in the order
//! `view::being::presence` gives (the shoulder and fist, the arm and torso,
//! the head), and nothing below its waist, where it goes into the seal.
//!
//! The seal is a disc on the floor round her feet in the same material,
//! split down the same line: dark half on her left, light on her right, a
//! faint ring whatever the bars, and each half's runes lit round from behind
//! her toward the front as its bar fills.
//!
//! Both are presentation and nothing else. The echo is the renderer's own
//! memory of what it drew, kept in her own space so a change of map frame
//! moves nothing; a rollback shows for a tenth of a second what was drawn
//! before it, which is what a thing that follows her is meant to do.

use bevy::asset::weak_handle;
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use sim::state::MAX_PLAYERS;
use view::pose::Pose;
use view::skeleton::{Joint, Skeleton, skeleton_for};

const SHADER: Handle<Shader> = weak_handle!("b3e1c8d4-6f2a-4a97-8c15-7d40e9a2b6f1");

pub use look::SpiritLook;

/// The uniform on its own, for the reason `ink::look` gives.
mod look {
    #![allow(dead_code)]
    use bevy::prelude::*;
    use bevy::render::render_resource::ShaderType;

    /// Everything [`super::Spirit`] needs, in the arena's coordinates.
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

/// The being's material, and the seal's: see the module.
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

/// Each seat's two materials, and the numbers worked out once from her
/// skeleton.
#[derive(Resource)]
pub struct Beings {
    /// Each seat's two beings' material, dark then light.
    pub body: [[Handle<Spirit>; MAX_PLAYERS]; 2],
    pub seal: [Handle<Spirit>; MAX_PLAYERS],
    /// Each being's own skeleton (`view::being::BODIES`) and how many times
    /// its size it is drawn.
    skeletons: [Skeleton; 2],
    scales: [f32; 2],
    seal_radius: f32,
    /// A shaft of light out of one of the dark being's rifts (`beam`): a
    /// unit frustum, narrow at the crack, and the light it is drawn in.
    pub beam_mesh: Handle<Mesh>,
    pub beam_light: Handle<StandardMaterial>,
}

/// How far each seat's being has taken up her pose: 0 standing as itself,
/// 1 throwing with her (`view::being::stance`). Eased, on the renderer's
/// clock: it is how a body moves, never a rule.
#[derive(Resource, Default)]
pub struct Casting(pub [f32; MAX_PLAYERS]);

/// How many of her drawn poses are kept: a tenth of a second at the
/// fastest refresh anybody has, and then some.
const ECHOES: usize = 64;

/// Her poses, as drawn, each with when: what the being is posed from.
#[derive(Resource)]
pub struct Echo {
    poses: [[Option<(f32, Pose)>; ECHOES]; MAX_PLAYERS],
    next: [usize; MAX_PLAYERS],
}

impl Default for Echo {
    fn default() -> Self {
        Echo {
            poses: [[None; ECHOES]; MAX_PLAYERS],
            next: [0; MAX_PLAYERS],
        }
    }
}

impl Echo {
    fn push(&mut self, owner: usize, at: f32, pose: Pose) {
        let i = self.next[owner];
        self.poses[owner][i] = Some((at, pose));
        self.next[owner] = (i + 1) % ECHOES;
    }

    /// The newest pose at least `lag` old, or the oldest kept if none is.
    fn behind(&self, owner: usize, now: f32, lag: f32) -> Option<Pose> {
        let mut best: Option<(f32, Pose)> = None;
        let mut oldest: Option<(f32, Pose)> = None;
        for (at, pose) in self.poses[owner].iter().flatten() {
            if now - at >= lag && best.is_none_or(|(b, _)| *at > b) {
                best = Some((*at, *pose));
            }
            if oldest.is_none_or(|(o, _)| *at < o) {
                oldest = Some((*at, *pose));
            }
        }
        best.or(oldest).map(|(_, pose)| pose)
    }
}

pub struct BeingPlugin;

impl Plugin for BeingPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<Assets<Shader>>().insert(
            SHADER.id(),
            Shader::from_wgsl(include_str!("being.wgsl"), "game/src/being.wgsl"),
        );
        app.add_plugins(MaterialPlugin::<Spirit>::default())
            .init_resource::<Echo>()
            .init_resource::<Casting>()
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
    let skeletons = view::being::BODIES.map(skeleton_for);
    let scales = [0, 1].map(|k| view::being::scale(&her, &skeletons[k]));
    let seal_radius = view::being::seal_radius(&her);
    let mut made = |mode: f32| {
        materials.add(Spirit {
            look: SpiritLook {
                fill: Vec4::new(0.0, 0.0, 0.0, mode),
                ..default()
            },
        })
    };
    let body = [
        std::array::from_fn(|_| made(0.0)),
        std::array::from_fn(|_| made(0.0)),
    ];
    let seal = std::array::from_fn(|_| made(1.0));
    commands.insert_resource(Beings {
        body,
        seal,
        skeletons,
        scales,
        seal_radius,
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
        gloss: Vec4::new(2200.0, 7.0, 0.0, 0.0),
        sheen: Vec4::new(70.0, 25.0, 200.0, 4.5),
    },
    crate::still::Lustre {
        gloss: Vec4::new(7000.0, 4.0, 0.0, 0.0),
        sheen: Vec4::new(300.0, 300.0, 340.0, 4.0),
    },
];

/// How high off the floor the seal is laid, so it never fights the floor
/// for the same depth.
const SEAL_LIFT: f32 = 0.025;

/// Stand each seat's being behind her, pose it from her echo, lay her seal,
/// and tell both how full each half is.
///
/// Her feet and facing come off the blended frame, which is where her body
/// is drawn; the bars off the current snapshot, because a meter is not a
/// thing to blend (as the wings were before it).
#[allow(clippy::type_complexity, clippy::too_many_arguments)] // a Bevy system: its arguments are what it reads
pub fn place(
    sim: Res<crate::Sim>,
    time: Res<Time>,
    joints: Res<crate::Joints>,
    rigs: Res<crate::models::Rigs>,
    beings: Res<Beings>,
    mut echo: ResMut<Echo>,
    mut casting: ResMut<Casting>,
    mut materials: ResMut<Assets<Spirit>>,
    mut roots: Query<(&BeingRoot, &mut Transform, &mut Visibility), Without<Seal>>,
    mut seals: Query<(&Seal, &mut Transform, &mut Visibility), Without<BeingRoot>>,
    mut bones: Query<&mut Transform, (Without<BeingRoot>, Without<Seal>, Without<crate::Fighter>)>,
) {
    let now = time.elapsed_secs();
    let frame = view::interpolate(&sim.prev, &sim.cur, sim.clock.alpha());
    // Per seat and per being: its look (what its veil is told) and where its
    // root stands.
    let mut looks: [[Option<SpiritLook>; 2]; MAX_PLAYERS] = [[None; 2]; MAX_PLAYERS];
    let mut stands: [[Option<(Vec3, Quat)>; 2]; MAX_PLAYERS] = [[None; 2]; MAX_PLAYERS];
    let whole = looked_at();
    for owner in 0..MAX_PLAYERS {
        let p = &sim.cur.players[owner];
        let Some(being) = view::being::being(p).filter(|_| sim.cur.is_present(owner)) else {
            continue;
        };
        let Some(drawn) = joints.pose[owner] else {
            continue;
        };
        echo.push(owner, now, drawn);
        let late = echo
            .behind(owner, now, view::being::LAG_SECONDS)
            .unwrap_or(drawn);
        // Into her pose while she throws something, back to its own after.
        let throwing = p.action.attack_kind().is_some();
        let w = &mut casting.0[owner];
        let rate = if throwing {
            view::being::CAST_IN
        } else {
            view::being::CAST_OUT
        };
        let target = if throwing { 1.0 } else { 0.0 };
        *w += (target - *w) * (1.0 - (-rate * time.delta_secs()).exp());
        let w = *w;
        let v = frame.players[owner];
        let feet = Vec3::from_array(v.pos);
        let q = view::body_turn([v.facing[0], v.facing[2]]).0;
        let turn = Quat::from_xyzw(q[0], q[1], q[2], q[3]);
        // Character space has the facing at +Z, and a drawn body's left at
        // +X (`view::hand_joint` says why).
        let left = turn * Vec3::X;
        for k in 0..2 {
            let skeleton = &beings.skeletons[k];
            let s = beings.scales[k];
            let skin = view::skeleton::solve(skeleton, &view::being::stance(k, &late, w));
            crate::models::pose(rigs.being[owner][k], Some(&skin), &mut bones);
            // Hips at her feet; or, looked at whole, standing on her floor.
            let sink = if whole > 0.0 {
                0.0
            } else {
                s * skeleton.hip_height
            };
            let root = feet + turn * Vec3::new(0.0, 0.0, -view::being::BEHIND) - Vec3::Y * sink;
            stands[owner][k] = Some((root, turn));
            let at = |local: [f32; 3]| root + turn * (Vec3::from_array(local) * s);
            let joint = |j: Joint| at(skin.origin[j.index()]);
            let fist = |left: bool| at(skin.box_of(skeleton, view::hand_joint(left)).0);
            let o = view::being::ORDER;
            looks[owner][k] = Some(SpiritLook {
                fill: Vec4::new(
                    if whole > 0.0 { 1.0 } else { being.dark },
                    if whole > 0.0 { 1.0 } else { being.light },
                    if being.ascending { 1.0 } else { 0.0 },
                    0.0,
                ),
                left: left.extend(s),
                feet: feet.extend(beings.seal_radius),
                dark_shoulder: joint(view::drawn_joint(Joint::ArmL)).extend(0.0),
                dark_fist: fist(true).extend(0.0),
                light_shoulder: joint(view::drawn_joint(Joint::ArmR)).extend(0.0),
                light_fist: fist(false).extend(0.0),
                head: at(skin.box_of(skeleton, Joint::Head).0).extend(0.0),
                order: Vec4::new(o.first, o.body, o.head, o.soft),
                // Each being keeps only its own half: 1 dark, 2 light.
                reach: Vec4::new(view::being::REACH, whole, (k + 1) as f32, 0.0),
            });
        }
    }

    for (root, mut tf, mut seen) in roots.iter_mut() {
        let (owner, k) = (root.0, root.1);
        let shown = looks[owner][k].is_some_and(|l| l.fill.x > 0.0 || l.fill.y > 0.0);
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
        let look = looks[seal.0][0];
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
        }
    }
    for (owner, pair) in looks.iter().enumerate() {
        for (k, look) in pair.iter().enumerate() {
            let Some(look) = *look else {
                continue;
            };
            if let Some(m) = materials.get_mut(&beings.body[k][owner]) {
                m.look = look;
            }
            if k == 0 {
                if let Some(m) = materials.get_mut(&beings.seal[owner]) {
                    m.look = SpiritLook {
                        fill: look.fill.truncate().extend(1.0),
                        reach: Vec4::new(look.reach.x, look.reach.y, 0.0, 0.0),
                        ..look
                    };
                }
            }
        }
    }
}

/// Give a being's model its material in place of what the file came with:
/// its veil of light, kept out of the shadows because it is light, not a
/// body -- or, looked at on its own (`SHOT_BEING`), its `painted` self.
pub fn dress(
    commands: &mut Commands,
    entity: Entity,
    beings: &Beings,
    owner: usize,
    being: usize,
    painted: Option<Handle<crate::still::SkinMaterial>>,
) {
    let mut e = commands.entity(entity);
    e.remove::<MeshMaterial3d<StandardMaterial>>();
    match painted {
        Some(skin) => {
            e.insert(MeshMaterial3d(skin));
        }
        None => {
            e.insert((
                MeshMaterial3d(beings.body[being][owner].clone()),
                NotShadowCaster,
                NotShadowReceiver,
            ));
        }
    }
}
