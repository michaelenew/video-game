//! **The arc a Champion's cut leaves**, drawn as a ribbon and painted by its
//! own shader.
//!
//! The arc is **the path the drawn blade took**: every picture, where the
//! weapon is drawn (`crate::drawn_arms`, the same answer the weapon itself
//! is placed by) is added to a short memory of where it has been
//! (`view::arms::Path`), and the arc is that memory joined by a curve
//! (`view::arms::ribbon`). Its head is the blade on every picture, through
//! the wind-up, the cut and the follow-through alike; an arc taken from the
//! hit volume instead was somewhere the sword was not, since the volume is
//! only out for the cut and the sword goes on moving after it.
//!
//! The strip is one mesh a fighter, a point on the inner edge and one on the
//! outer per column. Each vertex carries how long ago the blade was there
//! (`uv.x`, 1 now and 0 [`view::arms::WINDOW`] frames back), which edge it is
//! (`uv.y`), the weapon's tint and how bright the cut was there (the colour's
//! alpha: dark through the wind-up, full while the volume is out, fading
//! over the follow-through). `swing.wgsl` turns those into the look.

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
            .init_resource::<Paths>()
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

/// Where each fighter's blade has been, for the arc behind it, and the frame
/// the path's clock counts from (so its times stay small and exact however
/// long the game has run). The renderer's own, like a cross-fade: see
/// `view::arms::Path`.
#[derive(Resource, Default)]
pub struct Paths([(view::arms::Path, u32); MAX_PLAYERS]);

/// **Lay each Champion's arc behind her blade**, this picture: where the
/// blade is drawn now is added to where it has been, and the arc is that
/// path, so its head is always on the weapon.
pub fn place(
    sim: Res<Sim>,
    hands: Res<crate::Hands>,
    torsos: Res<crate::Torsos>,
    mut paths: ResMut<Paths>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut arcs: Query<(&Cut, &mut Transform, &mut Visibility)>,
) {
    let alpha = sim.clock.alpha();
    for (arc, mut tf, mut vis) in arcs.iter_mut() {
        let (path, base) = &mut paths.0[arc.owner];
        let cut = view::arms::cut(&sim.cur.players[arc.owner], alpha);
        let arms = cut.and_then(|_| crate::drawn_arms(&sim, &hands, &torsos, arc.owner));
        let (Some(cut), Some(arms)) = (cut, arms) else {
            path.clear();
            *vis = Visibility::Hidden;
            continue;
        };
        if path.samples().is_empty() || sim.cur.frame < *base {
            path.clear();
            *base = sim.cur.frame;
        }
        let now = (sim.cur.frame - *base) as f32 + alpha;
        path.push(view::arms::Sample::of(&arms, &cut, now));
        let r = view::arms::ribbon(path, now);
        if r.count < 2 {
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
                let life = r.life[k.min(head)];
                uv[2 * k] = [life, 0.0];
                uv[2 * k + 1] = [life, 1.0];
            }
        }
        if let Some(VertexAttributeValues::Float32x4(colour)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
        {
            let [red, green, blue] = tint(cut.which);
            for k in 0..RIBBON {
                let glow = r.glow[k.min(head)];
                colour[2 * k] = [red, green, blue, glow];
                colour[2 * k + 1] = [red, green, blue, glow];
            }
        }
        *vis = Visibility::Inherited;
    }
}
