//! **Every pipeline compiled before it is needed.**
//!
//! A GPU draws a mesh through a *pipeline*: its shaders, compiled for one
//! mesh layout (which attributes its vertices carry), one material and one
//! way of blending. Bevy compiles one the first frame something on screen
//! needs it, and a compile is tens of milliseconds -- hundreds in a browser.
//! On the web and on macOS the frame waits for it; elsewhere the thing is
//! simply missing until it is ready. Either way it lands at the first moment
//! a combination is *seen*: the first parry flash, the first time the camera
//! closes in and one's own body fades to translucent, the first box of a new
//! kind streaming in. That is the game seizing at random and settling down
//! the longer it is played, because every combination it has met is kept.
//!
//! So every mesh, the moment it is made -- pooled effects hidden until they
//! are used included -- is checked against what has been asked for already,
//! and a combination not met before gets a **primer**: the same mesh and
//! material, a speck at the origin that no frustum culls, for a few frames.
//! That asks for the pipeline while the game is starting (or while a place
//! is loading) rather than in the middle of a fight. A material that can be
//! faded at run time (the standard material, which `fade_own_body` turns
//! translucent) is primed both opaque and blended.
//!
//! What is drawn is unchanged; only *when* its pipelines are compiled moves.
//! `--profile` prints every pipeline as it is asked for, so one first asked
//! for minutes into a fight is one this missed.

use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::render::mesh::skinning::SkinnedMesh;
use bevy::render::view::NoFrustumCulling;

use crate::beast::MarkMaterial;
use crate::ink::InkMaterial;
use crate::paint::Paint;
use crate::still::SkinMaterial;
use crate::swing::SwingMaterial;

/// How many frames a primer stays: one to be extracted, one to be
/// specialised, and one spare.
const PRIMER_FRAMES: u8 = 3;

/// So small it covers no pixel, wherever it is.
const SPECK: f32 = 1e-4;

pub struct WarmPlugin;

impl Plugin for WarmPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Primed>().add_systems(
            PostUpdate,
            (
                prime::<StandardMaterial>,
                prime::<Paint>,
                prime::<MarkMaterial>,
                prime::<SkinMaterial>,
                prime::<InkMaterial>,
                prime::<SwingMaterial>,
                retire,
            )
                .before(bevy::render::view::VisibilitySystems::CheckVisibility),
        );
    }
}

/// The combinations already asked for, hashed.
#[derive(Resource, Default)]
struct Primed(HashSet<u64>);

/// A speck drawn to ask for a pipeline, and how many frames it has left.
#[derive(Component)]
struct Primer(u8);

/// What a material's pipeline is specialised on, and the other forms it
/// may take at run time.
pub trait Warm: Material + Clone {
    /// Everything about this material that picks a different pipeline.
    fn key(&self, h: &mut DefaultHasher);
    /// The forms it may be changed into while it is on screen, itself
    /// first.
    fn forms(&self) -> Vec<Self> {
        vec![self.clone()]
    }
    /// Drawn into the still world's shadow map as well as the picture
    /// (`crate::still`), whose camera wants pipelines of its own.
    fn still() -> bool {
        false
    }
}

impl Warm for StandardMaterial {
    fn key(&self, h: &mut DefaultHasher) {
        std::mem::discriminant(&self.alpha_mode).hash(h);
        self.cull_mode.hash(h);
        self.double_sided.hash(h);
        self.unlit.hash(h);
        self.depth_bias.to_bits().hash(h);
        self.base_color_texture.is_some().hash(h);
        self.emissive_texture.is_some().hash(h);
        self.normal_map_texture.is_some().hash(h);
        self.metallic_roughness_texture.is_some().hash(h);
        self.occlusion_texture.is_some().hash(h);
        (self.diffuse_transmission > 0.0).hash(h);
        (self.specular_transmission > 0.0).hash(h);
        (self.clearcoat > 0.0).hash(h);
    }

    /// Opaque and blended: `fade_own_body` turns a fighter's skin
    /// translucent as the camera closes in, and back.
    fn forms(&self) -> Vec<Self> {
        let other = match self.alpha_mode {
            AlphaMode::Opaque => AlphaMode::Blend,
            AlphaMode::Blend => AlphaMode::Opaque,
            _ => return vec![self.clone()],
        };
        vec![
            self.clone(),
            StandardMaterial {
                alpha_mode: other,
                ..self.clone()
            },
        ]
    }
}

impl Warm for Paint {
    fn key(&self, h: &mut DefaultHasher) {
        self.two_sided.hash(h);
        self.windowed.hash(h);
        self.ghost.hash(h);
    }

    fn still() -> bool {
        true
    }
}

impl Warm for SkinMaterial {
    fn key(&self, h: &mut DefaultHasher) {
        self.base.key(h);
    }

    /// Opaque and blended, as the standard material: it is the fighters'
    /// skin `fade_own_body` fades.
    fn forms(&self) -> Vec<Self> {
        self.base
            .forms()
            .into_iter()
            .map(|base| SkinMaterial {
                base,
                extension: self.extension.clone(),
            })
            .collect()
    }
}

impl Warm for InkMaterial {
    fn key(&self, h: &mut DefaultHasher) {
        self.base.key(h);
    }
}

/// One form: blended, both sides, nothing else to choose.
impl Warm for SwingMaterial {
    fn key(&self, _h: &mut DefaultHasher) {}
}

impl Warm for MarkMaterial {
    fn key(&self, h: &mut DefaultHasher) {
        self.base.key(h);
    }
}

/// **Prime what is new**: every mesh of material `M` made or given another
/// mesh or material this frame, hidden or not.
#[allow(clippy::type_complexity)] // A Bevy query: one filter per change it watches.
fn prime<M: Warm>(
    mut commands: Commands,
    mut primed: ResMut<Primed>,
    meshes: Res<Assets<Mesh>>,
    mut materials: ResMut<Assets<M>>,
    new: Query<
        (
            &Mesh3d,
            &MeshMaterial3d<M>,
            Has<NotShadowCaster>,
            Has<NotShadowReceiver>,
            Option<&SkinnedMesh>,
        ),
        (
            Or<(Changed<Mesh3d>, Changed<MeshMaterial3d<M>>)>,
            Without<Primer>,
        ),
    >,
) {
    for (mesh, material, no_cast, no_receive, skin) in &new {
        let (Some(m), Some(mat)) = (meshes.get(&mesh.0), materials.get(&material.0)) else {
            continue;
        };
        let mut layout = DefaultHasher::new();
        std::any::type_name::<M>().hash(&mut layout);
        m.primitive_topology().hash(&mut layout);
        for (attribute, _) in m.attributes() {
            attribute.id.hash(&mut layout);
            attribute.format.hash(&mut layout);
        }
        m.morph_targets().is_some().hash(&mut layout);
        no_cast.hash(&mut layout);
        no_receive.hash(&mut layout);
        let forms = mat.forms();
        for (k, form) in forms.into_iter().enumerate() {
            let mut h = layout.clone();
            form.key(&mut h);
            if !primed.0.insert(h.finish()) {
                continue;
            }
            // The material itself where it is the form on screen now; a
            // copy for a form it may take later.
            let handle = if k == 0 {
                material.0.clone()
            } else {
                materials.add(form)
            };
            let mut e = commands.spawn((
                Mesh3d(mesh.0.clone()),
                MeshMaterial3d(handle),
                Transform::from_scale(Vec3::splat(SPECK)),
                Visibility::Visible,
                NoFrustumCulling,
                Primer(PRIMER_FRAMES),
            ));
            if no_cast {
                e.insert(NotShadowCaster);
            }
            // A skinned mesh's pipeline is a skinned one, and drawing it
            // without the joints it was skinned to is a validation error
            // that takes the game down: the primer is skinned to the same
            // joints. Its speck scale means nothing to a skinned mesh, so for
            // its few frames it is drawn whole, where its joints are -- for
            // the models `reaver` makes, at startup, at each seat's place,
            // which is the one moment a flicker costs nothing.
            if let Some(skin) = skin {
                e.insert(skin.clone());
            }
            if no_receive {
                e.insert(NotShadowReceiver);
            }
            // Seen by the still world's shadow camera too, which asks for
            // its own pipelines the next time it draws -- and a new kind of
            // still thing streaming in is a reason it will.
            if M::still() {
                e.insert(bevy::render::view::RenderLayers::from_layers(&[
                    0,
                    crate::still::LAYER,
                ]));
            }
        }
    }
}

/// A primer's frames run out, and it goes.
fn retire(mut commands: Commands, mut primers: Query<(Entity, &mut Primer)>) {
    for (e, mut left) in &mut primers {
        if left.0 == 0 {
            commands.entity(e).despawn();
        } else {
            left.0 -= 1;
        }
    }
}
