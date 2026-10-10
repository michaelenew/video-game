//! **The one material everything standing still is drawn in.**
//!
//! Every box, floor, tree and slab carries its colour in its vertices, so what
//! a material has to do is light that colour: the sun through its shadow, the
//! fill, the ambient, the fog. The standard material does that and a great
//! deal more -- metal, reflectance, emission, clearcoat, transmission, a
//! texture lookup for each -- and reads all of it for every pixel on screen
//! whether or not the surface set any of it, which none of these do. Under
//! the software renderer, where a shader's cost can be read exactly, the main
//! pass was 110 ms a frame of that, the biggest pass by three times
//! (`docs/design/architecture.md`, "The valley's frame").
//!
//! So this is the standard material's lighting with everything unset taken
//! out: `paint.wgsl`, a few dozen lines. It keeps Bevy's own diffuse and
//! ambient terms, its shadow lookup, and its fog and tonemapping *by the same
//! function*, so a surface comes out the colour it did -- `look::palette::lit`
//! is a response curve measured off the screen, and this has to be the screen
//! it was measured off. It drops the specular lobe, a sheen on matte stone.
//!
//! Nothing here decides what a colour is. `look` does, and `shapes` and
//! `forms` put the answer in the vertices; this is the Bevy half, beside
//! `shapes.rs` and `sky.rs`.

use bevy::asset::weak_handle;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
};

const SHADER: Handle<Shader> = weak_handle!("7c1e5a2b-9d44-4f3e-8b6a-2e0f5c7d9a13");

/// A flat-lit material: its colour times the vertex colour, lit by the
/// scene's lights and nothing else.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
#[bind_group_data(PaintKey)]
pub struct Paint {
    /// Multiplied with the vertex colour; white where the vertices carry it
    /// all.
    #[uniform(0)]
    pub color: Vec4,
    /// Drawn from both sides: the land's tiles, whose skirts face out
    /// either way round.
    pub two_sided: bool,
}

impl Default for Paint {
    fn default() -> Self {
        Self::white()
    }
}

impl Paint {
    /// The paint for a mesh that carries its own colour.
    pub fn white() -> Self {
        Self {
            color: Vec4::ONE,
            two_sided: false,
        }
    }

    /// One colour, sRGB, for a mesh with no colours of its own.
    pub fn srgb(rgb: [f32; 3]) -> Self {
        let c = Color::srgb(rgb[0], rgb[1], rgb[2]).to_linear();
        Self {
            color: Vec4::new(c.red, c.green, c.blue, 1.0),
            two_sided: false,
        }
    }
}

/// What the pipeline is specialised on.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PaintKey {
    two_sided: bool,
}

impl From<&Paint> for PaintKey {
    fn from(p: &Paint) -> Self {
        Self {
            two_sided: p.two_sided,
        }
    }
}

impl Material for Paint {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(SHADER)
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if key.bind_group_data.two_sided {
            descriptor.primitive.cull_mode = None;
        }
        Ok(())
    }
}

/// The material stores a scene is drawn from: the paint for what carries its
/// colour, and the standard material for what does not -- the translucent,
/// the glowing, the water.
pub struct Materials<'a> {
    pub standard: &'a mut Assets<StandardMaterial>,
    pub paint: &'a mut Assets<Paint>,
}

pub struct PaintPlugin;

impl Plugin for PaintPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<Assets<Shader>>().insert(
            SHADER.id(),
            Shader::from_wgsl(include_str!("paint.wgsl"), "game/src/paint.wgsl"),
        );
        app.add_plugins(MaterialPlugin::<Paint>::default());
    }
}
