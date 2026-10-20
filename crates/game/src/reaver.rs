//! **The Shadow Reaver, drawn as a character** rather than as the mannequin
//! every other class still is (`docs/design/exploration/0024_a_character_done_properly.md`).
//!
//! The model is `assets/reaver.glb`, built by `art/reaver/build.py` from
//! MakeHuman's CC0 body: a skinned mesh whose sixteen joints are exactly the
//! game's (`view::skeleton`), named as `view::skeleton::Joint::name` names
//! them and bound in a pose of the same skeleton. So nothing about animation
//! changes: `apply_poses` solves the skeleton as it always has, and each joint
//! of the model is put where the solve says, in character space, under the
//! fighter's root. The inverse bind matrices in the file do the rest.
//!
//! One instance a seat, made at startup and hidden unless that seat is
//! playing the Reaver, so changing class never loads anything. Its
//! materials are swapped for the fighters' skin (`still::SkinMaterial`) so
//! the still world's shade falls on her as on everybody else; the sash is
//! its own material and is dyed the seat's colour.

use std::path::{Path, PathBuf};

use bevy::asset::io::embedded::EmbeddedAssetRegistry;
use bevy::gltf::GltfMaterialName;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use sim::state::MAX_PLAYERS;

use crate::still;

/// Where the model is read from: embedded in the binary, so the browser
/// build carries it the same way the desktop does.
const MODEL: &str = "embedded://reaver.glb";

pub struct ReaverPlugin;

impl Plugin for ReaverPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .resource_mut::<EmbeddedAssetRegistry>()
            .insert_asset(
                PathBuf::new(),
                Path::new("reaver.glb"),
                include_bytes!("../assets/reaver.glb"),
            );
        app.init_resource::<Rigs>()
            .add_systems(PostStartup, spawn)
            .add_observer(ready);
    }
}

/// The model of one seat's Reaver.
#[derive(Component)]
pub struct Model(pub usize);

/// Each seat's joints, once its model has been spawned, in
/// `view::skeleton::JOINTS` order; and its materials, for the fade.
#[derive(Resource, Default)]
pub struct Rigs {
    pub joints: [Option<[Entity; view::skeleton::JOINT_COUNT]>; MAX_PLAYERS],
    pub looks: [Vec<Handle<still::SkinMaterial>>; MAX_PLAYERS],
}

fn spawn(
    mut commands: Commands,
    server: Res<AssetServer>,
    fighters: Query<(Entity, &crate::Fighter)>,
) {
    let scene: Handle<Scene> = server.load(GltfAssetLabel::Scene(0).from_asset(MODEL));
    for (root, fighter) in fighters.iter() {
        let model = commands
            .spawn((
                Model(fighter.0),
                SceneRoot(scene.clone()),
                Transform::default(),
                Visibility::Hidden,
            ))
            .id();
        commands.entity(root).add_child(model);
    }
}

/// Once a seat's model is in the world: find its joints by name, and give
/// it the fighters' skin in place of the plain materials the file came with.
#[allow(clippy::too_many_arguments)] // a Bevy observer: its arguments are what it reads
fn ready(
    trigger: Trigger<SceneInstanceReady>,
    models: Query<&Model>,
    children: Query<&Children>,
    names: Query<&Name>,
    looks: Query<(&MeshMaterial3d<StandardMaterial>, Option<&GltfMaterialName>)>,
    standard: Res<Assets<StandardMaterial>>,
    mut skins: ResMut<Assets<still::SkinMaterial>>,
    mut rigs: ResMut<Rigs>,
    mut commands: Commands,
) {
    let root = trigger.target();
    let Ok(model) = models.get(root) else {
        return;
    };
    let owner = model.0;
    let mut joints = [Entity::PLACEHOLDER; view::skeleton::JOINT_COUNT];
    for entity in children.iter_descendants(root) {
        if let Ok(name) = names.get(entity) {
            if let Some(j) = view::skeleton::JOINTS
                .iter()
                .position(|j| j.name() == name.as_str())
            {
                joints[j] = entity;
            }
        }
        if let Ok((look, material)) = looks.get(entity) {
            let Some(base) = standard.get(&look.0) else {
                continue;
            };
            let mut base = base.clone();
            // The exporter names its materials; the sash's is dyed.
            if material.is_some_and(|m| m.0 == "team") {
                base.base_color = crate::hud::SEAT_COLOURS[owner];
            }
            // Cloth, leather and skin hardly mirror anything. At Bevy's
            // default half (a 4% mirror) a dark coat under a bright sky came
            // out the sky's pale lavender all over; the steel's own
            // reflection comes from its metalness, which this does not touch.
            base.reflectance = 0.12;
            let skin = skins.add(still::skin(base));
            rigs.looks[owner].push(skin.clone());
            commands
                .entity(entity)
                .remove::<MeshMaterial3d<StandardMaterial>>()
                .insert(MeshMaterial3d(skin));
        }
    }
    if joints.iter().all(|e| *e != Entity::PLACEHOLDER) {
        rigs.joints[owner] = Some(joints);
    }
}

/// Whether a class is drawn by its model rather than the mannequin.
pub fn drawn(class: sim::Class) -> bool {
    matches!(class, sim::Class::ShadowReaver)
}

/// Put each model's joints where this frame's solve put the skeleton, and
/// show the model only on a seat playing the class.
pub fn pose(
    rigs: &Rigs,
    owner: usize,
    class: sim::Class,
    skin: Option<&view::skeleton::Skin>,
    models: &mut Query<(&Model, &mut Visibility)>,
    joints: &mut Query<&mut Transform, Without<crate::Fighter>>,
) {
    let on = drawn(class) && skin.is_some();
    for (model, mut seen) in models.iter_mut() {
        if model.0 == owner {
            let want = if on {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            if *seen != want {
                *seen = want;
            }
        }
    }
    let (Some(skin), Some(rig), true) = (skin, rigs.joints[owner], on) else {
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
