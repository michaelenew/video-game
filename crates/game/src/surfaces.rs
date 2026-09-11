//! Wiring the material system into Bevy.
//!
//! Thin on purpose. `art` is a pure function from parameters to pixel buffers
//! and knows nothing about the engine; this turns those buffers into
//! `StandardMaterial` handles and does nothing else. Everything interesting is
//! on the other side of that line, where it can be tested without a window.
//!
//! Two engine-shaped details that are easy to get wrong and silent when you
//! do:
//!
//! **Colour space is per map, not per material.** Albedo and emission are
//! sRGB-encoded because they are colours a person picked; normals and the
//! roughness pack are raw numbers and must be uploaded as-is. Upload a normal
//! map as sRGB and every slope comes out wrong in a way that looks like bad
//! lighting rather than like a wrong texture.
//!
//! **A normal map needs tangents**, and Bevy's primitive meshes do not carry
//! any. Without them the map is ignored -- no warning, no error, just a
//! material that looks exactly as flat as it did before the relief was added.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::math::Affine2;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use art::bake::{Maps, Texture};
use art::materials::{self, BAKE_SIZE, Material};

/// Upload one baked map.
fn image(tex: &Texture, srgb: bool, tiling: bool) -> Image {
    let format = if srgb {
        TextureFormat::Rgba8UnormSrgb
    } else {
        TextureFormat::Rgba8Unorm
    };
    let mut img = Image::new(
        Extent3d {
            width: tex.width,
            height: tex.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        tex.pixels.clone(),
        format,
        // The textures are built once at startup and never read back, so the
        // processor copy is dead weight the moment it reaches the card.
        RenderAssetUsages::RENDER_WORLD,
    );
    if tiling {
        img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            ..ImageSamplerDescriptor::default()
        });
    }
    img
}

/// Build a Bevy material from a baked one.
///
/// `repeat` is how many times the tile covers the surface it is going on. It
/// is a property of the *object*, not of the material -- a four-metre tile on
/// a thirty-metre floor repeats seven and a half times and on a two-metre
/// crate repeats half a time -- which is why it is an argument here rather
/// than a field over in `art`.
pub fn build(
    m: &Material,
    repeat: Vec2,
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    let maps: Maps = art::bake::bake(&m.surface, m.plan(BAKE_SIZE));
    let tiling = repeat.x > 1.0 || repeat.y > 1.0;

    let emissive = if maps.emissive_strength > 1.0 {
        // The bake normalised the emission into eight bits and handed the
        // factor back. Multiply it in here, where the renderer is working in
        // floating point and a flame is allowed to be nine times brighter than
        // white -- which is what makes bloom pick it out.
        LinearRgba::rgb(
            maps.emissive_strength,
            maps.emissive_strength,
            maps.emissive_strength,
        )
    } else {
        LinearRgba::BLACK
    };

    materials.add(StandardMaterial {
        base_color_texture: Some(images.add(image(&maps.albedo, true, tiling))),
        normal_map_texture: Some(images.add(image(&maps.normal, false, tiling))),
        metallic_roughness_texture: Some(images.add(image(
            &maps.metallic_roughness,
            false,
            tiling,
        ))),
        emissive_texture: (maps.emissive_strength > 1.0)
            .then(|| images.add(image(&maps.emissive, true, tiling))),
        emissive,
        // The textures carry the variation; these are the multipliers Bevy
        // applies on top, so they have to be neutral or the maps are scaled by
        // whatever the defaults happen to be.
        base_color: Color::WHITE,
        perceptual_roughness: 1.0,
        metallic: 1.0,
        uv_transform: Affine2::from_scale(repeat),
        ..default()
    })
}

/// How many times a material's tile repeats across a surface of this size.
pub fn repeat_for(m: &Material, size: Vec2) -> Vec2 {
    (size / m.surface_extent()).max(Vec2::ONE)
}

/// Every material the arena and its fighters need, built once.
///
/// A resource rather than loose handles because the effect pool swaps which
/// material an entity wears as slots are reused, so they have to outlive
/// setup. Named for surfaces rather than for looks because `Look` is already
/// taken, by the camera.
#[derive(Resource)]
pub struct Surfaces {
    pub fire: Handle<StandardMaterial>,
    pub blood: Handle<StandardMaterial>,
    pub shadow: Handle<StandardMaterial>,
    pub stone: Handle<StandardMaterial>,
    pub ground: Handle<StandardMaterial>,
    pub leather: Handle<StandardMaterial>,
    /// Per player: skin, cloth, armour.
    pub skin: Vec<Handle<StandardMaterial>>,
    pub cloth: Vec<Handle<StandardMaterial>>,
    pub armour: Vec<Handle<StandardMaterial>>,
}

impl Surfaces {
    pub fn build(
        players: usize,
        images: &mut Assets<Image>,
        mats: &mut Assets<StandardMaterial>,
    ) -> Surfaces {
        let one = Vec2::ONE;
        let mut skin = Vec::new();
        let mut cloth = Vec::new();
        let mut armour = Vec::new();
        for p in 0..players {
            let tint = art::palette::PLAYERS[p % art::palette::PLAYERS.len()];
            // Skin is the same for everyone. Identity is carried by what a
            // fighter is *wearing*, not by the colour of their body -- see the
            // palette module. A player-tinted skin would spend the hue channel
            // twice and leave the arena with two blue things in it.
            skin.push(build(&materials::SKIN, one, images, mats));
            cloth.push(build(&materials::cloth(tint), one, images, mats));
            armour.push(build(&materials::armour(tint), one, images, mats));
        }
        Surfaces {
            fire: build(&materials::FIRE, one, images, mats),
            blood: build(&materials::BLOOD, one, images, mats),
            shadow: build(&materials::SHADOW, one, images, mats),
            stone: build(&materials::STONE, Vec2::splat(2.0), images, mats),
            // The floor is forty metres across and the tile is four, so it
            // repeats ten times. That is the point at which one triplanar
            // shader starts to earn its place -- see the header of `art::bake`
            // -- and the point at which it does not is now a thing that can be
            // looked at rather than argued about.
            ground: build(&materials::GROUND, Vec2::splat(10.0), images, mats),
            leather: build(&materials::LEATHER, one, images, mats),
            skin,
            cloth,
            armour,
        }
    }
}

/// Give a mesh the tangents a normal map needs.
///
/// Bevy's primitive meshes carry positions, normals and texture coordinates
/// and no tangents, and a `StandardMaterial` with a normal map on a mesh
/// without them **silently ignores the map**. No warning, no error: the
/// material just looks exactly as flat as it did before the relief was added,
/// which sends you looking at the bake.
///
/// Failure here means the mesh was missing something tangents are computed
/// from, which for a built-in primitive cannot happen; the mesh is returned
/// untouched rather than panicking, since a flat-looking wall is a much better
/// outcome than a game that will not start.
pub fn tangented(mut mesh: Mesh) -> Mesh {
    let _ = mesh.generate_tangents();
    mesh
}
