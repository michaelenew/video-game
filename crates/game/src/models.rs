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
//! **A class's model is loaded the first time a seat plays it** (2026-10-21,
//! exploration 0024 sec. 11): a file from `assets/`, read when it is wanted
//! rather than compiled into the program, made for that seat, and kept --
//! hidden while the seat plays something else, so changing back loads
//! nothing. Until it is there the seat is drawn as the mannequin, as it
//! always was before a model was ready. For the Reaver six more: her shadow
//! and the five copies of the Reaping, made with her. Each
//! hangs under the root the mannequin it replaces already had (`Fighter`,
//! `ShadowRoot`, `CopyRoot`), so it is placed, shown and hidden exactly as
//! that was. They wear the fighters' skin (`still::SkinMaterial`), so the
//! still world's shade falls on them as on everybody else, with the piece
//! the exporter names `team` dyed the seat's colour; the shadow and the
//! copies are in ink (`crate::ink`). For the Dual mage two more: the dark and
//! the light being behind her (`crate::being`), each a body of its own
//! ([`BEINGS`]), under one root.

use bevy::gltf::GltfMaterialName;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use sim::state::MAX_PLAYERS;
use view::skeleton::{JOINT_COUNT, Joint, Skin};

use crate::coat::{TAIL_JOINTS, TAIL_NAMES, TailJoint, Tails};
use crate::ink::{self, Ink, InkMaterial};
use crate::{REAP_COPIES, still};

/// The classes drawn by a model, and the model's file in `assets/`
/// (`platform::asset_root`): beside the program on a desktop, beside the
/// page in a browser (`crates/web/build-game.sh` copies them), so neither
/// carries a model nobody is playing.
pub const MODELS: [(sim::Class, &str); MODELLED] = [
    (sim::Class::ShadowReaver, "reaver.glb"),
    (sim::Class::Champion, "champion.glb"),
    (sim::Class::Elementalist, "elementalist.glb"),
    (sim::Class::BloodMage, "blood_mage.glb"),
    (sim::Class::DualMage, "dual_mage.glb"),
];
/// How many classes have a model.
pub const MODELLED: usize = 5;

/// The two beings behind the Dual mage, dark then light: bodies of their
/// own, built by `art/dark_being` and `art/light_being` on her skeleton.
pub const BEINGS: [&str; 2] = ["dark_being.glb", "light_being.glb"];

/// Which entry of [`MODELS`] a class is, if it has a model.
pub fn model_of(class: sim::Class) -> Option<usize> {
    MODELS.iter().position(|(c, _)| *c == class)
}

pub struct ModelsPlugin;

impl Plugin for ModelsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Rigs>()
            .init_resource::<Loaded>()
            .init_resource::<crate::squeeze::Squeeze>()
            .init_resource::<crate::coat::Coats>()
            .init_resource::<crate::slash::Traces>()
            .add_systems(PostStartup, crate::slash::spawn)
            .add_systems(Update, (spawn, crate::squeeze::squeeze))
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
    /// One of the beings behind the Dual mage: 0 the dark, 1 the light.
    Being(usize),
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
            Body::Being(k) => class == sim::Class::DualMage && crate::being::shows(k),
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
    pub being: [[Option<Joints>; 2]; MAX_PLAYERS],
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
            Body::Being(k) => &mut self.being[owner][k],
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

/// Which models have been asked for, and made for which seats.
#[derive(Resource, Default)]
pub struct Loaded {
    scenes: [Option<Handle<Scene>>; MODELLED],
    beings: [Option<Handle<Scene>>; 2],
    her: [[bool; MODELLED]; MAX_PLAYERS],
    shadow: [bool; MAX_PLAYERS],
    being: [bool; MAX_PLAYERS],
}

impl Loaded {
    /// A model's scene, asked of the file the first time it is wanted. Its
    /// textures stay readable until they are squeezed (`squeeze`), which
    /// lets their copy in memory go. Its meshes keep theirs: `warm` reads a
    /// mesh's layout to compile its pipelines before it is seen, and it is
    /// a few megabytes a character, not the textures' hundreds.
    fn scene(&mut self, m: usize, server: &AssetServer) -> Handle<Scene> {
        self.scenes[m]
            .get_or_insert_with(|| server.load(GltfAssetLabel::Scene(0).from_asset(MODELS[m].1)))
            .clone()
    }

    /// A being's scene, the same way.
    fn being(&mut self, k: usize, server: &AssetServer) -> Handle<Scene> {
        self.beings[k]
            .get_or_insert_with(|| server.load(GltfAssetLabel::Scene(0).from_asset(BEINGS[k])))
            .clone()
    }
}

/// Make the model of every class a present seat is playing, for that seat,
/// the first time it plays it -- and with the Reaver's, her shadow and her
/// copies; with the Dual mage's, the being behind her.
#[allow(clippy::too_many_arguments)] // a Bevy system: its arguments are what it reads
fn spawn(
    mut commands: Commands,
    server: Res<AssetServer>,
    sim: Res<crate::Sim>,
    mut loaded: ResMut<Loaded>,
    fighters: Query<(Entity, &crate::Fighter)>,
    shadows: Query<(Entity, &crate::ShadowRoot)>,
    copies: Query<(Entity, &crate::CopyRoot)>,
    beings: Query<(Entity, &crate::being::BeingRoot)>,
) {
    let reaver = model_of(sim::Class::ShadowReaver).expect("the Reaver has a model");
    let dual = model_of(sim::Class::DualMage).expect("the Dual mage has a model");
    let add = |commands: &mut Commands, scene: Handle<Scene>, root: Entity, owner, body| {
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
        let owner = fighter.0;
        if !sim.cur.is_present(owner) {
            continue;
        }
        let Some(m) = model_of(sim.cur.players[owner].class) else {
            continue;
        };
        if !loaded.her[owner][m] {
            loaded.her[owner][m] = true;
            let scene = loaded.scene(m, &server);
            add(&mut commands, scene, root, owner, Body::Her(m));
        }
        if m == reaver && !loaded.shadow[owner] {
            loaded.shadow[owner] = true;
            let scene = loaded.scene(reaver, &server);
            for (root, shadow) in shadows.iter().filter(|(_, s)| s.0 == owner) {
                add(&mut commands, scene.clone(), root, shadow.0, Body::Shadow);
            }
            for (root, copy) in copies.iter().filter(|(_, c)| c.owner == owner) {
                add(
                    &mut commands,
                    scene.clone(),
                    root,
                    owner,
                    Body::Copy(copy.copy),
                );
            }
        }
        if m == dual && !loaded.being[owner] {
            loaded.being[owner] = true;
            for (root, b) in beings.iter().filter(|(_, b)| b.0 == owner) {
                let scene = loaded.being(b.1, &server);
                add(&mut commands, scene, root, owner, Body::Being(b.1));
            }
        }
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
    mut squeeze: ResMut<crate::squeeze::Squeeze>,
    beings: Res<crate::being::Beings>,
    mut commands: Commands,
) {
    let root = trigger.target();
    let Ok(&Model { owner, body }) = models.get(root) else {
        return;
    };
    let mut joints = [Entity::PLACEHOLDER; JOINT_COUNT];
    let mut tails = [None; TAIL_JOINTS];
    let mut blades = [[Entity::PLACEHOLDER; 2]; 2];
    // A being's rifts: each shaft's two markers, `beam.<n>.from` and `.to`.
    let mut beams: Vec<(String, Entity, Vec3)> = Vec::new();
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
            if name.starts_with("beam.") {
                if let Ok(t) = binds.get(entity) {
                    beams.push((name.to_string(), entity, t.translation));
                }
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
        squeeze.take(&base);
        // A being is a veil of light in its force (`being::dress`), or,
        // looked at on its own, its painted self.
        if let Body::Being(k) = body {
            let painted = crate::being::whole().then(|| {
                base.reflectance = 0.12;
                // Its glow is light, in the camera's exposure: at the
                // file's factor of one it would not show.
                base.emissive = LinearRgba::rgb(1.0, 1.0, 1.0) * crate::being::GLOW[k];
                let mut skin = still::skin(base);
                skin.extension.lustre = crate::being::LUSTRE[k];
                skins.add(skin)
            });
            crate::being::dress(&mut commands, entity, &beings, owner, k, painted);
            continue;
        }
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
    if let Body::Being(_) = body {
        for (name, from, at) in beams.iter().filter(|(n, ..)| n.ends_with(".from")) {
            let to = name.replace(".from", ".to");
            if let Some((_, _, end)) = beams.iter().find(|(n, ..)| *n == to) {
                crate::being::beam(&mut commands, &beings, (*from, *at), *end);
            }
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
pub fn pose<F: bevy::ecs::query::QueryFilter>(
    rig: Option<Joints>,
    skin: Option<&Skin>,
    joints: &mut Query<&mut Transform, F>,
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
