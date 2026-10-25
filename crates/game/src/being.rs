//! **The being behind the Dual mage, and the seal it rises from**
//! (`view::being`, 2026-10-25): her two bars, drawn.
//!
//! The being is her own model a second time, under a root of its own
//! ([`BeingRoot`]) that stands it behind her at [`view::being::scale`] times
//! her size with its hips at her feet, posed with her pose from
//! [`view::being::LAG_SECONDS`] ago ([`Echo`]), its fists held up in a guard
//! (`view::being::guarded`) -- so it throws what she throws, larger and a
//! beat behind. It is drawn in [`Spirit`], which is nothing of
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

/// The root a seat's being hangs off: placed behind her, scaled, and shown
/// while she has a bar to show.
#[derive(Component)]
pub struct BeingRoot(pub usize);

/// A seat's seal, on the floor round her feet.
#[derive(Component)]
pub struct Seal(pub usize);

/// Each seat's two materials, and the numbers worked out once from her
/// skeleton.
#[derive(Resource)]
pub struct Beings {
    pub body: [Handle<Spirit>; MAX_PLAYERS],
    pub seal: [Handle<Spirit>; MAX_PLAYERS],
    skeleton: Skeleton,
    scale: f32,
    seal_radius: f32,
}

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
            .add_systems(PreStartup, setup)
            .add_systems(Startup, spawn);
    }
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<Spirit>>) {
    let skeleton = skeleton_for(sim::Class::DualMage);
    let scale = view::being::scale(&skeleton);
    let seal_radius = view::being::seal_radius(&skeleton);
    let mut made = |mode: f32| {
        materials.add(Spirit {
            look: SpiritLook {
                fill: Vec4::new(0.0, 0.0, 0.0, mode),
                ..default()
            },
        })
    };
    let body = std::array::from_fn(|_| made(0.0));
    let seal = std::array::from_fn(|_| made(1.0));
    commands.insert_resource(Beings {
        body,
        seal,
        skeleton,
        scale,
        seal_radius,
    });
}

/// A root and a seal per seat, hidden until a Dual mage needs them: the
/// class is picked at runtime and can change mid-match with Tab. The being's
/// model is made under its root with hers (`models::spawn`).
fn spawn(mut commands: Commands, beings: Res<Beings>, mut meshes: ResMut<Assets<Mesh>>) {
    let disc = meshes.add(Plane3d::new(Vec3::Y, Vec2::ONE).mesh().subdivisions(0));
    for owner in 0..MAX_PLAYERS {
        commands.spawn((BeingRoot(owner), Transform::default(), Visibility::Hidden));
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

/// `SHOT_BEING=dark` or `SHOT_BEING=light`: the being drawn whole, in one
/// force, every part of it there and standing on her floor rather than
/// rising out of the seal -- how each of its two bodies is looked at on its
/// own before they are halved. Read once.
fn whole() -> f32 {
    static WHOLE: std::sync::OnceLock<f32> = std::sync::OnceLock::new();
    *WHOLE.get_or_init(|| match crate::platform::env("SHOT_BEING").as_deref() {
        Some("dark") => 1.0,
        Some("light") => 2.0,
        _ => 0.0,
    })
}

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
    mut materials: ResMut<Assets<Spirit>>,
    mut roots: Query<(&BeingRoot, &mut Transform, &mut Visibility), Without<Seal>>,
    mut seals: Query<(&Seal, &mut Transform, &mut Visibility), Without<BeingRoot>>,
    mut bones: Query<&mut Transform, (Without<BeingRoot>, Without<Seal>, Without<crate::Fighter>)>,
) {
    let now = time.elapsed_secs();
    let frame = view::interpolate(&sim.prev, &sim.cur, sim.clock.alpha());
    let s = beings.scale;
    let skeleton = &beings.skeleton;
    let mut looks: [Option<SpiritLook>; MAX_PLAYERS] = [None; MAX_PLAYERS];
    let mut stands: [Option<(Vec3, Quat)>; MAX_PLAYERS] = [None; MAX_PLAYERS];
    let whole = whole();
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
        let skin = view::skeleton::solve(skeleton, &view::being::guarded(&late));
        crate::models::pose(rigs.being[owner], Some(&skin), &mut bones);

        let v = frame.players[owner];
        let feet = Vec3::from_array(v.pos);
        let q = view::body_turn([v.facing[0], v.facing[2]]).0;
        let turn = Quat::from_xyzw(q[0], q[1], q[2], q[3]);
        // Character space has the facing at +Z, and a drawn body's left at
        // +X (`view::hand_joint` says why).
        // Hips at her feet; or, looked at whole, standing on her floor.
        let sink = if whole > 0.0 {
            0.0
        } else {
            s * skeleton.hip_height
        };
        let root = feet + turn * Vec3::new(0.0, 0.0, -view::being::BEHIND) - Vec3::Y * sink;
        stands[owner] = Some((root, turn));
        let at = |local: [f32; 3]| root + turn * (Vec3::from_array(local) * s);
        let joint = |j: Joint| at(skin.origin[j.index()]);
        let fist = |left: bool| at(skin.box_of(skeleton, view::hand_joint(left)).0);
        let left = turn * Vec3::X;
        let o = view::being::ORDER;
        looks[owner] = Some(SpiritLook {
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
            reach: Vec4::new(view::being::REACH, whole, 0.0, 0.0),
        });
    }

    for (root, mut tf, mut seen) in roots.iter_mut() {
        let shown = looks[root.0].is_some_and(|l| l.fill.x > 0.0 || l.fill.y > 0.0);
        let want = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *seen != want {
            *seen = want;
        }
        if let Some((at, turn)) = stands[root.0] {
            tf.translation = at;
            tf.rotation = turn;
            tf.scale = Vec3::splat(s);
        }
    }
    for (seal, mut tf, mut seen) in seals.iter_mut() {
        let want = if looks[seal.0].is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *seen != want {
            *seen = want;
        }
        if let Some(look) = looks[seal.0] {
            tf.translation = look.feet.truncate() + Vec3::Y * SEAL_LIFT;
            tf.scale = Vec3::new(beings.seal_radius, 1.0, beings.seal_radius);
        }
    }
    for (owner, look) in looks.iter().enumerate() {
        let Some(look) = *look else {
            continue;
        };
        for (handle, mode) in [(&beings.body[owner], 0.0), (&beings.seal[owner], 1.0)] {
            if let Some(m) = materials.get_mut(handle) {
                m.look = SpiritLook {
                    fill: look.fill.truncate().extend(mode),
                    ..look
                };
            }
        }
    }
}

/// Give a being's model its material in place of the costume's, and keep it
/// out of the shadows: it is light, not a body.
pub fn dress(commands: &mut Commands, entity: Entity, beings: &Beings, owner: usize) {
    commands
        .entity(entity)
        .remove::<MeshMaterial3d<StandardMaterial>>()
        .insert((
            MeshMaterial3d(beings.body[owner].clone()),
            NotShadowCaster,
            NotShadowReceiver,
        ));
}
