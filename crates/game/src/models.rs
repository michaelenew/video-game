//! **The classes drawn as characters** rather than as mannequins
//! (`docs/design/exploration/0024_a_character_done_properly.md`): the Shadow
//! Reaver first, and since 2026-10-20 the Champion, the Elementalist, the
//! Blood mage and the Dual mage. The Bulwark is still the mannequin.
//!
//! Each model is `assets/<class>.glb`, built by `art/<class>/build.py` from
//! MakeHuman's CC0 body: a skinned mesh whose twenty-one joints are exactly
//! the game's (`view::skeleton`), named as `view::skeleton::Joint::name`
//! names them and bound in a pose of the same skeleton. So nothing about
//! animation changes: `apply_poses` solves the skeleton as it always has, and
//! each joint of the model is put where the solve says, in character space,
//! under the fighter's root. The inverse bind matrices in the file do the rest.
//!
//! **One of each class a seat**, made at startup and hidden unless that seat
//! is playing that class, so changing class never loads anything -- and for
//! the Reaver six more: her shadow and the five copies of the Reaping. Each
//! hangs under the root the mannequin it replaces already had (`Fighter`,
//! `ShadowRoot`, `CopyRoot`), so it is placed, shown and hidden exactly as
//! that was. They wear the fighters' skin (`still::SkinMaterial`), so the
//! still world's shade falls on them as on everybody else, with the piece
//! the exporter names `team` dyed the seat's colour; the shadow and the
//! copies are in ink (`crate::ink`).

use std::path::{Path, PathBuf};

use bevy::asset::io::embedded::EmbeddedAssetRegistry;
use bevy::gltf::GltfMaterialName;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use sim::state::MAX_PLAYERS;
use view::skeleton::{JOINT_COUNT, Joint, Skin};

use crate::coat::{TAIL_JOINTS, TAIL_NAMES, TailJoint, Tails};
use crate::ink::{self, Ink, InkMaterial};
use crate::{REAP_COPIES, still};

/// The classes drawn by a model, and the model's file: embedded in the
/// binary, so the browser build carries them the same way the desktop does.
pub const MODELS: [(sim::Class, &str, &[u8]); MODELLED] = [
    (
        sim::Class::ShadowReaver,
        "reaver.glb",
        include_bytes!("../assets/reaver.glb"),
    ),
    (
        sim::Class::Champion,
        "champion.glb",
        include_bytes!("../assets/champion.glb"),
    ),
    (
        sim::Class::Elementalist,
        "elementalist.glb",
        include_bytes!("../assets/elementalist.glb"),
    ),
    (
        sim::Class::BloodMage,
        "blood_mage.glb",
        include_bytes!("../assets/blood_mage.glb"),
    ),
    (
        sim::Class::DualMage,
        "dual_mage.glb",
        include_bytes!("../assets/dual_mage.glb"),
    ),
];
/// How many classes have a model.
pub const MODELLED: usize = 5;

/// Which entry of [`MODELS`] a class is, if it has a model.
pub fn model_of(class: sim::Class) -> Option<usize> {
    MODELS.iter().position(|(c, _, _)| *c == class)
}

pub struct ModelsPlugin;

impl Plugin for ModelsPlugin {
    fn build(&self, app: &mut App) {
        let registry = app.world_mut().resource_mut::<EmbeddedAssetRegistry>();
        for (_, file, bytes) in MODELS {
            registry.insert_asset(PathBuf::new(), Path::new(file), bytes);
        }
        app.init_resource::<Rigs>()
            .init_resource::<crate::coat::Coats>()
            .init_resource::<crate::slash::Traces>()
            .add_systems(PostStartup, (spawn, crate::slash::spawn))
            .add_systems(
                PostUpdate,
                crate::slash::draw.after(bevy::transform::TransformSystem::TransformPropagate),
            )
            .add_observer(ready);
    }
}

/// Which body a model is: a fighter's own (as one of the classes with a
/// model, by its place in [`MODELS`]), or the Reaver's shadow or a copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    Her(usize),
    Shadow,
    Copy(usize),
}

/// One of a seat's models.
#[derive(Component)]
pub struct Model {
    pub owner: usize,
    pub body: Body,
}

impl Model {
    /// Whether this model is shown while its seat plays `class`.
    pub fn shown_as(&self, class: sim::Class) -> bool {
        match self.body {
            Body::Her(m) => MODELS[m].0 == class,
            Body::Shadow | Body::Copy(_) => class == sim::Class::ShadowReaver,
        }
    }
}

type Joints = [Entity; JOINT_COUNT];

/// Each seat's joints, once its models have been spawned, in
/// `view::skeleton::JOINTS` order -- one set for each class with a model;
/// its own materials, for the fade; and the copies' ink, for theirs.
#[derive(Resource, Default)]
pub struct Rigs {
    pub her: [[Option<Joints>; MODELLED]; MAX_PLAYERS],
    pub shadow: [Option<Joints>; MAX_PLAYERS],
    pub copies: [[Option<Joints>; REAP_COPIES]; MAX_PLAYERS],
    pub looks: [Vec<Handle<still::SkinMaterial>>; MAX_PLAYERS],
    pub inks: [[Vec<Handle<InkMaterial>>; REAP_COPIES]; MAX_PLAYERS],
    /// Each model's coat joints (`coat`), and the Reaver's blades' tips and
    /// roots, left hand then right (`slash`).
    pub tails: [[Option<Tails>; MODELLED]; MAX_PLAYERS],
    pub blades: [Option<[[Entity; 2]; 2]>; MAX_PLAYERS],
}

impl Rigs {
    fn slot(&mut self, owner: usize, body: Body) -> &mut Option<Joints> {
        match body {
            Body::Her(m) => &mut self.her[owner][m],
            Body::Shadow => &mut self.shadow[owner],
            Body::Copy(c) => &mut self.copies[owner][c],
        }
    }

    /// Whether a seat playing `class` can put its mannequin away: that
    /// class has a model and the seat's is there.
    pub fn ready(&self, owner: usize, class: sim::Class) -> bool {
        model_of(class).is_some_and(|m| self.her[owner][m].is_some())
    }

    /// The joints of the model a seat playing `class` is drawn with.
    pub fn her(&self, owner: usize, class: sim::Class) -> Option<Joints> {
        model_of(class).and_then(|m| self.her[owner][m])
    }

    /// The coat of the model a seat playing `class` is drawn with.
    pub fn tails(&self, owner: usize, class: sim::Class) -> Option<&Tails> {
        model_of(class).and_then(|m| self.tails[owner][m].as_ref())
    }
}

fn spawn(
    mut commands: Commands,
    server: Res<AssetServer>,
    fighters: Query<(Entity, &crate::Fighter)>,
    shadows: Query<(Entity, &crate::ShadowRoot)>,
    copies: Query<(Entity, &crate::CopyRoot)>,
) {
    let scenes: Vec<Handle<Scene>> = MODELS
        .iter()
        .map(|(_, file, _)| {
            server.load(GltfAssetLabel::Scene(0).from_asset(format!("embedded://{file}")))
        })
        .collect();
    let reaver = model_of(sim::Class::ShadowReaver).expect("the Reaver has a model");
    let mut add = |root: Entity, owner: usize, body: Body| {
        let scene = match body {
            Body::Her(m) => scenes[m].clone(),
            Body::Shadow | Body::Copy(_) => scenes[reaver].clone(),
        };
        let model = commands
            .spawn((
                Model { owner, body },
                SceneRoot(scene),
                Transform::default(),
                Visibility::Hidden,
            ))
            .id();
        commands.entity(root).add_child(model);
    };
    for (root, fighter) in fighters.iter() {
        for m in 0..MODELLED {
            add(root, fighter.0, Body::Her(m));
        }
    }
    for (root, shadow) in shadows.iter() {
        add(root, shadow.0, Body::Shadow);
    }
    for (root, copy) in copies.iter() {
        add(root, copy.owner, Body::Copy(copy.copy));
    }
}

/// Once a model is in the world: find its joints by name, and give it its
/// materials in place of the plain ones the file came with.
#[allow(clippy::too_many_arguments)] // a Bevy observer: its arguments are what it reads
fn ready(
    trigger: Trigger<SceneInstanceReady>,
    models: Query<&Model>,
    children: Query<&Children>,
    names: Query<&Name>,
    binds: Query<&Transform>,
    looks: Query<(&MeshMaterial3d<StandardMaterial>, Option<&GltfMaterialName>)>,
    standard: Res<Assets<StandardMaterial>>,
    mut skins: ResMut<Assets<still::SkinMaterial>>,
    mut inks: ResMut<Assets<InkMaterial>>,
    mut rigs: ResMut<Rigs>,
    mut commands: Commands,
) {
    let root = trigger.target();
    let Ok(&Model { owner, body }) = models.get(root) else {
        return;
    };
    let mut joints = [Entity::PLACEHOLDER; JOINT_COUNT];
    let mut tails = [None; TAIL_JOINTS];
    let mut blades = [[Entity::PLACEHOLDER; 2]; 2];
    for entity in children.iter_descendants(root) {
        if let Ok(name) = names.get(entity) {
            let name = name.as_str();
            if let Some(j) = view::skeleton::JOINTS.iter().position(|j| j.name() == name) {
                joints[j] = entity;
            }
            // Read before anything poses them: these are the bind pose.
            if let (Some(t), Ok(bind)) = (
                TAIL_NAMES.iter().position(|t| *t == name),
                binds.get(entity),
            ) {
                tails[t] = Some(TailJoint {
                    entity,
                    bind: *bind,
                });
            }
            for (hand, side) in ["l", "r"].iter().enumerate() {
                if name == format!("blade.{side}.tip") {
                    blades[hand][0] = entity;
                } else if name == format!("blade.{side}.base") {
                    blades[hand][1] = entity;
                }
            }
        }
        let Ok((look, material)) = looks.get(entity) else {
            continue;
        };
        let Some(base) = standard.get(&look.0) else {
            continue;
        };
        let mut base = base.clone();
        let mut swap = commands.entity(entity);
        swap.remove::<MeshMaterial3d<StandardMaterial>>();
        if let Body::Her(_) = body {
            // The exporter names its materials; the team piece's is dyed.
            if material.is_some_and(|m| m.0 == "team") {
                base.base_color = crate::hud::SEAT_COLOURS[owner];
            }
            // Cloth, leather and skin hardly mirror anything. At Bevy's
            // default half (a 4% mirror) a dark coat under a bright sky
            // takes the sky's colour all over; the steel's own reflection
            // comes from its metalness, which this does not touch.
            base.reflectance = 0.12;
            let skin = skins.add(still::skin(base));
            rigs.looks[owner].push(skin.clone());
            swap.insert(MeshMaterial3d(skin));
        } else {
            let made = inks.add(ink::ink(base, Ink::shadow()));
            if let Body::Copy(c) = body {
                rigs.inks[owner][c].push(made.clone());
            }
            swap.insert((MeshMaterial3d(made), NotShadowCaster));
        }
    }
    if joints.iter().all(|e| *e != Entity::PLACEHOLDER) {
        *rigs.slot(owner, body) = Some(joints);
        if let Body::Her(m) = body {
            let hips = binds.get(joints[Joint::Root.index()]).map(|t| t.rotation);
            if let (true, Ok(hips_bind)) = (tails.iter().all(Option::is_some), hips) {
                rigs.tails[owner][m] = Some(Tails {
                    joints: tails.map(|t| t.expect("every coat joint found")),
                    hips_bind,
                });
            }
            if MODELS[m].0 == sim::Class::ShadowReaver
                && blades.iter().flatten().all(|e| *e != Entity::PLACEHOLDER)
            {
                rigs.blades[owner] = Some(blades);
            }
        }
    }
}

/// Whether a class is drawn by its model rather than the mannequin.
pub fn drawn(class: sim::Class) -> bool {
    model_of(class).is_some()
}

/// Put a model's joints where a solve put the skeleton.
pub fn pose(
    rig: Option<Joints>,
    skin: Option<&Skin>,
    joints: &mut Query<&mut Transform, Without<crate::Fighter>>,
) {
    let (Some(rig), Some(skin)) = (rig, skin) else {
        return;
    };
    for (i, entity) in rig.iter().enumerate() {
        if let Ok(mut tf) = joints.get_mut(*entity) {
            let o = skin.origin[i];
            let r = skin.rot[i].0;
            tf.translation = Vec3::new(o[0], o[1], o[2]);
            tf.rotation = Quat::from_xyzw(r[0], r[1], r[2], r[3]);
        }
    }
}

/// How solid a copy is, on its ink: what its mannequin's alpha was.
pub fn fade_copy(
    rigs: &Rigs,
    owner: usize,
    copy: usize,
    solid: f32,
    inks: &mut Assets<InkMaterial>,
) {
    for handle in &rigs.inks[owner][copy] {
        // Looked at before it is taken mutably, so an unchanged copy is not
        // rebuilt on the GPU every frame (`fade_own_body` says why).
        if inks
            .get(handle)
            .is_some_and(|m| (m.extension.look.fade.x - solid).abs() > 0.01)
        {
            if let Some(m) = inks.get_mut(handle) {
                m.extension.look.fade.x = solid;
            }
        }
    }
}
