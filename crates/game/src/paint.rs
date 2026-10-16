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
/// The brush strokes, baked (`look::strokes`, drawn by `build.rs`): a sheet
/// per rigidity and angle, as a texture array every paint reads.
const STROKES: Handle<Image> = weak_handle!("a1f3c9e2-5b7d-4c80-9e41-3d6f8a2b7c15");

/// A flat-lit material: its colour times the vertex colour, lit by the
/// scene's lights and nothing else.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
#[bind_group_data(PaintKey)]
pub struct Paint {
    /// Multiplied with the vertex colour; white where the vertices carry it
    /// all.
    #[uniform(0)]
    pub color: Vec4,
    /// **A near tile's own strokes** (`crate::near`): where on the map its
    /// texture lies -- corner x, corner z, size -- and, in the last, how far
    /// from the eye it is read before the sheet takes over. All zero for a
    /// paint with none, which is every paint but a near tile's.
    #[uniform(0)]
    pub near: Vec4,
    /// The baked brush strokes ([`STROKES`]); the same for every paint.
    #[texture(1, dimension = "2d_array")]
    #[sampler(2)]
    pub strokes: Handle<Image>,
    /// The near tile's texture, if this paint is a near tile's.
    #[texture(3)]
    #[sampler(4)]
    pub near_tex: Option<Handle<Image>>,
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
            strokes: STROKES,
            near: Vec4::ZERO,
            near_tex: None,
            two_sided: false,
        }
    }

    /// One colour, sRGB, for a mesh with no colours of its own.
    pub fn srgb(rgb: [f32; 3]) -> Self {
        let c = Color::srgb(rgb[0], rgb[1], rgb[2]).to_linear();
        Self {
            color: Vec4::new(c.red, c.green, c.blue, 1.0),
            strokes: STROKES,
            near: Vec4::ZERO,
            near_tex: None,
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
        if let Some(fragment) = descriptor.fragment.as_mut() {
            fragment.shader_defs.extend(crate::wash::surface_defs());
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
        app.world_mut()
            .resource_mut::<Assets<Image>>()
            .insert(STROKES.id(), strokes_image());
        app.add_plugins(MaterialPlugin::<Paint>::default());
    }
}

/// The baked strokes as a texture array: the PNG `build.rs` drew, the sheets
/// stacked, cut into layers. Linear, not sRGB: a texel is a shade and two
/// colour shifts, -1 to 1, not a colour. On a plain picture, which never
/// reads it, a single grey texel so the binding exists.
fn strokes_image() -> Image {
    use bevy::render::render_asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    if crate::wash::painted() {
        return baked_strokes();
    }
    Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: look::strokes::LAYERS as u32,
        },
        TextureDimension::D2,
        &[128, 128, 128, 255],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// The baked sheets, as the painted looks read them.
fn baked_strokes() -> Image {
    use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
    use bevy::render::render_asset::RenderAssetUsages;
    let mut image = Image::from_buffer(
        include_bytes!(concat!(env!("OUT_DIR"), "/strokes.png")),
        bevy::image::ImageType::Extension("png"),
        bevy::image::CompressedImageFormats::NONE,
        false,
        ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            mag_filter: bevy::image::ImageFilterMode::Linear,
            min_filter: bevy::image::ImageFilterMode::Linear,
            ..default()
        }),
        RenderAssetUsages::RENDER_WORLD,
    )
    .expect("the strokes are a png the build script drew");
    image.reinterpret_stacked_2d_as_array(look::strokes::LAYERS as u32);
    image
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The sheets have to arrive as square layers, not one tall image: read
    /// as one, every stroke is stretched along the sheet.
    #[test]
    fn the_strokes_are_square_sheets() {
        let image = baked_strokes();
        let size = image.texture_descriptor.size;
        assert_eq!(size.width, look::strokes::SIZE as u32);
        assert_eq!(size.height, look::strokes::SIZE as u32);
        assert_eq!(size.depth_or_array_layers, look::strokes::LAYERS as u32);
        // And a texel of the first sheet is the generator's.
        let want = look::strokes::sheet(0.0, 0.0)[7 * look::strokes::SIZE + 11];
        let data = image.data.as_ref().expect("texels");
        let at = (7 * look::strokes::SIZE + 11) * 4;
        assert_eq!(&data[at..at + 4], &want);
    }
}
