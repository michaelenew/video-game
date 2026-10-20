//! **The arc a Champion's cut leaves**, drawn as a ribbon and painted by its
//! own shader.
//!
//! The simulation knows where the weapon's volume is once a frame, and a
//! sword crosses a quarter turn in three of them, so the old trail -- a flat
//! box between each two frames, at one of six fixed solidities -- read as a
//! fan of glass blades. Here the frames are joined by a curve
//! (`view::arms::ribbon`), cut into fine columns, and laid out as one strip of
//! triangles a fighter: every column a point on the inner edge and one on the
//! outer. Each vertex carries two numbers -- how far along the arc it is, tail
//! to head, and which edge it is on -- and the weapon's tint and the fade, and
//! `swing.wgsl` turns those into the look: a crescent, thick at the head and
//! drawn to a point at the tail, a hot line where the blade's edge passed,
//! fine streaks that fray at the tail rather than stopping, and a fade that
//! eats the arc from its tail up into its head through the recovery.
//!
//! Nothing here decides what the arc looks like but the shader; where it is
//! comes from the hit volume, so the arc on the screen is where the cut could
//! land.

use bevy::asset::weak_handle;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey, NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::render::mesh::{
    Indices, MeshVertexBufferLayoutRef, PrimitiveTopology, VertexAttributeValues,
};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
};
use bevy::render::view::NoFrustumCulling;
use sim::state::MAX_PLAYERS;
use view::arms::RIBBON;

use crate::Sim;

const SHADER: Handle<Shader> = weak_handle!("4d9b2f71-6a3e-4c58-9e17-b2c8d5a0f6e3");

/// The arc's material. Everything it varies by is in the vertices, so one
/// serves every fighter and every weapon.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct SwingMaterial {}

impl Material for SwingMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(SHADER)
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    /// Seen from either side: a cut across the camera shows its back.
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        // `SHOT_SWING=uv`: the strip solid, red along the arc and green
        // across it, to see where it is and which way it runs.
        // `SHOT_SWING=alpha`: the strip solid, as bright as it is opaque.
        let def = match crate::platform::env("SHOT_SWING").as_deref() {
            Some("uv") => Some("SWING_UV"),
            Some("alpha") => Some("SWING_ALPHA"),
            _ => None,
        };
        if let (Some(def), Some(fragment)) = (def, descriptor.fragment.as_mut()) {
            fragment.shader_defs.push(def.into());
        }
        Ok(())
    }
}

pub struct SwingPlugin;

impl Plugin for SwingPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<Assets<Shader>>().insert(
            SHADER.id(),
            Shader::from_wgsl(include_str!("swing.wgsl"), "game/src/swing.wgsl"),
        );
        app.add_plugins(MaterialPlugin::<SwingMaterial>::default())
            .add_systems(Startup, spawn);
    }
}

/// One fighter's arc: its own mesh, rewritten in place each picture.
#[derive(Component)]
pub struct Cut {
    owner: usize,
    mesh: Handle<Mesh>,
}

/// The weapons' tints, so the three arcs are told apart as well as the three
/// silhouettes: the sword's icy blue, white at its edge, the hammer's a hot ember, the
/// spear's a thin blue streak. In linear light, as the vertices carry them.
fn tint(which: u8) -> [f32; 3] {
    let c = match which {
        0 => Color::srgb(0.55, 0.80, 1.0),
        1 => Color::srgb(1.0, 0.55, 0.20),
        _ => Color::srgb(0.45, 0.80, 1.0),
    }
    .to_linear();
    [c.red, c.green, c.blue]
}

/// The strip, every column at the origin: two vertices a column, inner then
/// outer, and the two triangles between each column and the next.
fn strip() -> Mesh {
    let vertices = RIBBON * 2;
    let mut indices = Vec::with_capacity((RIBBON - 1) * 6);
    for k in 0..RIBBON as u32 - 1 {
        let (a, b, c, d) = (2 * k, 2 * k + 1, 2 * k + 2, 2 * k + 3);
        indices.extend_from_slice(&[a, b, c, b, d, c]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0f32; 3]; vertices])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0f32; 2]; vertices])
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0f32; 4]; vertices])
    .with_inserted_indices(Indices::U32(indices))
}

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SwingMaterial>>,
) {
    let material = materials.add(SwingMaterial {});
    for owner in 0..MAX_PLAYERS {
        let mesh = meshes.add(strip());
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::default(),
            Visibility::Hidden,
            // The strip is rewritten each picture and its bounds are not, so
            // the box it was made with would cull it.
            NoFrustumCulling,
            NotShadowCaster,
            NotShadowReceiver,
            Cut { owner, mesh },
        ));
    }
}

/// **Lay each Champion's arc where her cut has been**, this picture.
pub fn place(
    sim: Res<Sim>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut arcs: Query<(&Cut, &mut Transform, &mut Visibility)>,
) {
    let alpha = sim.clock.alpha();
    for (arc, mut tf, mut vis) in arcs.iter_mut() {
        let Some(trail) = view::arms::trail(&sim.cur.players[arc.owner]) else {
            *vis = Visibility::Hidden;
            continue;
        };
        let r = view::arms::ribbon(&trail, alpha);
        if r.fade <= 0.0 {
            *vis = Visibility::Hidden;
            continue;
        }
        let Some(mesh) = meshes.get_mut(&arc.mesh) else {
            continue;
        };
        // Placed at its middle, so it is sorted among the other see-through
        // things by where it is rather than by the origin.
        let middle = r.outer[..r.count]
            .iter()
            .fold(Vec3::ZERO, |sum, p| sum + Vec3::from(*p))
            / r.count as f32;
        tf.translation = middle;
        let head = r.count - 1;
        if let Some(VertexAttributeValues::Float32x3(at)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        {
            for k in 0..RIBBON {
                // Past the head, every column folds onto it: no area, no pixel.
                let c = k.min(head);
                at[2 * k] = (Vec3::from(r.inner[c]) - middle).into();
                at[2 * k + 1] = (Vec3::from(r.outer[c]) - middle).into();
            }
        }
        if let Some(VertexAttributeValues::Float32x2(uv)) = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
        {
            for k in 0..RIBBON {
                let u = r.along[k.min(head)];
                uv[2 * k] = [u, 0.0];
                uv[2 * k + 1] = [u, 1.0];
            }
        }
        if let Some(VertexAttributeValues::Float32x4(colour)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
        {
            let [red, green, blue] = tint(trail.which);
            colour.fill([red, green, blue, r.fade]);
        }
        *vis = Visibility::Inherited;
    }
}
