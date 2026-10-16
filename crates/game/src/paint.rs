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
/// The depth a windowed paint leaves in the prepass, with its window cut
/// ([`Paint::window`]).
const PREPASS: Handle<Shader> = weak_handle!("3e8d1f6a-2c57-4b09-a7e4-9f1b6c3d5e82");
/// The window itself, which both of the paint's passes import.
const CUT: Handle<Shader> = weak_handle!("b5c2e7d9-4a13-4f68-8d2e-6a9c1f4b7e30");
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
    /// **A hole along the line from the eye to the fighter**: where the
    /// fighter is (`xyz`, world) and how wide the hole is there (`w`,
    /// metres; zero, none). What lets a grown tree stand between the camera
    /// and the fighter without hiding them (`crate::grove`): the camera does
    /// not dodge a tree's wood, so the wood gets out of the way instead.
    /// Only a [`Paint::window`] paint cuts it all the way through.
    #[uniform(0)]
    pub cut: Vec4,
    /// The baked brush strokes ([`STROKES`]); the same for every paint.
    #[texture(1, dimension = "2d_array")]
    #[sampler(2)]
    pub strokes: Handle<Image>,
    /// Drawn from both sides: the land's tiles, whose skirts face out
    /// either way round.
    pub two_sided: bool,
    /// **Its window cut in the depth too** ([`Paint::window`]): drawn
    /// alpha-masked, so the depth prepass runs `paint_prepass.wgsl` and
    /// cuts the same hole the colour does.
    pub windowed: bool,
    /// **Only the window, see-through** ([`Paint::ghost`]): drawn blended,
    /// over the hole a [`Paint::window`] cuts.
    pub ghost: bool,
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
            cut: Vec4::ZERO,
            strokes: STROKES,
            two_sided: false,
            windowed: false,
            ghost: false,
        }
    }

    /// **The paint with a window in it**: [`Paint::white`], with [`Paint::cut`]
    /// cut through the depth prepass as well as the colour.
    ///
    /// A hole only in the colour is not a hole. The depth prepass draws
    /// every opaque surface first, and the colour pass draws only what is
    /// no further than that depth, so the fighter behind a tree cut in the
    /// colour alone still fails against the tree's depth and nothing is
    /// drawn there at all: the clear colour, the sky's. In play that was a
    /// white shape where the tree should have opened. The same goes for the
    /// inside of the far wall seen through the hole, which is why a
    /// windowed paint culls back faces in the prepass as the colour does
    /// ([`Material::specialize`]).
    pub fn window() -> Self {
        Self {
            windowed: true,
            ..Self::white()
        }
    }

    /// **The wood in the window, see-through**: the same mesh drawn a second
    /// time over a [`Paint::window`], with the same [`Paint::cut`], keeping
    /// only what the window cut away and blending it over what is behind --
    /// thinnest in the middle, solid at the rim where the window's own wood
    /// takes over (`cut.wgsl`). Without it the window was a clean hole, a
    /// bite out of the tree.
    pub fn ghost() -> Self {
        Self {
            ghost: true,
            ..Self::white()
        }
    }

    /// One colour, sRGB, for a mesh with no colours of its own.
    pub fn srgb(rgb: [f32; 3]) -> Self {
        let c = Color::srgb(rgb[0], rgb[1], rgb[2]).to_linear();
        Self {
            color: Vec4::new(c.red, c.green, c.blue, 1.0),
            cut: Vec4::ZERO,
            strokes: STROKES,
            two_sided: false,
            windowed: false,
            ghost: false,
        }
    }
}

/// What the pipeline is specialised on.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PaintKey {
    two_sided: bool,
    windowed: bool,
    ghost: bool,
}

impl From<&Paint> for PaintKey {
    fn from(p: &Paint) -> Self {
        Self {
            two_sided: p.two_sided,
            windowed: p.windowed,
            ghost: p.ghost,
        }
    }
}

impl Material for Paint {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(SHADER)
    }

    /// Run only where the material may discard: a [`Paint::window`].
    /// Everything else keeps the prepass's depth-only draw.
    fn prepass_fragment_shader() -> ShaderRef {
        ShaderRef::Handle(PREPASS)
    }

    /// Masked is what makes Bevy run a fragment shader in the depth prepass
    /// at all; the mask's alpha is never read, the window discards.
    fn alpha_mode(&self) -> AlphaMode {
        if self.ghost {
            AlphaMode::Blend
        } else if self.windowed {
            AlphaMode::Mask(0.5)
        } else {
            AlphaMode::Opaque
        }
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if key.bind_group_data.two_sided {
            descriptor.primitive.cull_mode = None;
        } else if key.bind_group_data.windowed || key.bind_group_data.ghost {
            // Bevy's prepass draws both faces unless the material says
            // otherwise, and the colour pass only the front. Through a
            // window that difference is on screen: the inside of the far
            // wall stands in the depth, the colour never draws it, and the
            // hole is the colour of the sky.
            descriptor.primitive.cull_mode = Some(bevy::render::render_resource::Face::Back);
        }
        if let Some(fragment) = descriptor.fragment.as_mut() {
            fragment.shader_defs.extend(crate::wash::surface_defs());
            if key.bind_group_data.ghost {
                fragment.shader_defs.push("PAINT_GHOST".into());
            }
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
        app.world_mut().resource_mut::<Assets<Shader>>().insert(
            PREPASS.id(),
            Shader::from_wgsl(
                include_str!("paint_prepass.wgsl"),
                "game/src/paint_prepass.wgsl",
            ),
        );
        app.world_mut().resource_mut::<Assets<Shader>>().insert(
            CUT.id(),
            Shader::from_wgsl(include_str!("cut.wgsl"), "game/src/cut.wgsl"),
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
