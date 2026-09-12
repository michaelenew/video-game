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
use bevy::math::UVec2;
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
        // How a see-through material combines with what is behind it, and the
        // answer differs by *why* it is see-through.
        //
        // **Something that glows adds.** Fire is emitted light: it makes what
        // is behind it brighter and never darker, and where there is no flame
        // it does nothing at all. Alpha-blending it instead multiplies the
        // background by a near-black albedo, so every dim part of the flame
        // paints a grey smear -- which is exactly what the fire pillar did,
        // rendering as a dark drum with flames on top of it. Adding also means
        // the silhouette takes care of itself: black adds nothing, so the
        // cylinder stops existing wherever the flame is out, with no alpha
        // needed to carve it.
        //
        // **Something that blocks light blends.** The Reaver's shadow is an
        // absence, so it has to be able to make what is behind it darker,
        // which is the one thing adding cannot do.
        alpha_mode: if maps.emissive_strength > 1.0 {
            AlphaMode::Add
        } else if maps.has_alpha {
            AlphaMode::Blend
        } else {
            AlphaMode::Opaque
        },
        // Deliberately *not* `unlit`, which this reached for first and which
        // turned every flame into a black drum.
        //
        // The reasoning was sound -- a flame is light, not a lit surface, so
        // taking the lighting off it should stop the sun putting a highlight
        // on it. The mistake is where Bevy adds emission: inside the lighting
        // pass. Marking a material unlit skips that pass, and the glow goes
        // with it, leaving only the near-black albedo the flame was given
        // precisely *because* it was supposed to be glowing.
        //
        // The albedo already does the job: at 0.02 reflectance there is
        // nothing for the sun to highlight.
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

// ---------------------------------------------------------------------------
// Boxes cut out of a solid
// ---------------------------------------------------------------------------

/// The six faces of a box, in the order they are packed into one texture.
const FACES: [([f32; 3], [f32; 3], [f32; 3]); 6] = [
    // (outward normal, across, down) as signs on the box's own axes.
    ([0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, -1.0, 0.0]), // +Z
    ([0.0, 0.0, -1.0], [-1.0, 0.0, 0.0], [0.0, -1.0, 0.0]), // -Z
    ([1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, -1.0, 0.0]), // +X
    ([-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, -1.0, 0.0]), // -X
    ([0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),  // +Y
    ([0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0]), // -Y
];

/// Where in the atlas each face lives: a three-by-two grid.
const ATLAS: (u32, u32) = (3, 2);

/// A box whose six faces each get their own corner of one texture.
///
/// Bevy's own cuboid gives every face the *whole* texture, which is right for a
/// tiling material and wrong for a solid one: six faces would show six copies
/// of the same cut through the rock, and the thing a solid model buys -- that
/// every face is a different piece of stone and a pattern turns the corner --
/// would be thrown away at the last step.
pub fn box_mesh(size: Vec3) -> Mesh {
    let mut positions = Vec::with_capacity(24);
    let mut normals = Vec::with_capacity(24);
    let mut uvs = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    let half = size * 0.5;
    for (f, (normal, across, down)) in FACES.iter().enumerate() {
        let n = Vec3::from_array(*normal);
        let a = Vec3::from_array(*across);
        let d = Vec3::from_array(*down);
        // The face's centre, and the two vectors that span it.
        let centre = n * half;
        let ax = a * half;
        let dn = d * half;

        let base = positions.len() as u32;
        for (du, dv) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            positions.push((centre + ax * du + dn * dv).to_array());
            normals.push(normal.to_owned());
            let (cx, cy) = ((f as u32 % ATLAS.0) as f32, (f as u32 / ATLAS.0) as f32);
            uvs.push([
                (cx + (du + 1.0) * 0.5) / ATLAS.0 as f32,
                (cy + (dv + 1.0) * 0.5) / ATLAS.1 as f32,
            ]);
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    tangented(mesh)
}

/// Bake a box's six faces out of a stone volume, into one atlas.
///
/// Each face is cut from where it actually sits in the world, so a wall at one
/// end of the arena is a different piece of rock from the one at the other --
/// with nothing to say so, because they were never the same texture.
pub fn box_from_stone(
    stone: &art::stone::Stone,
    centre: Vec3,
    size: Vec3,
    tile: u32,
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    use art::bake::{Detail, bake_stone};
    use art::stone::Placement;

    let atlas = UVec2::new(ATLAS.0 * tile, ATLAS.1 * tile);
    let mut albedo = vec![0u8; (atlas.x * atlas.y * 4) as usize];
    let mut normal = albedo.clone();
    let mut orm = albedo.clone();

    // One density for the whole box, taken from its largest face. Otherwise a
    // thirty-metre wall's end cap resolves thirty times finer than its side and
    // the two faces show visibly different rock along the edge between them.
    let uniform = Detail::Uniform(size.max_element() / tile as f32);

    let half = size * 0.5;
    for (f, (n, across, down)) in FACES.iter().enumerate() {
        let n = Vec3::from_array(*n);
        let a = Vec3::from_array(*across);
        let d = Vec3::from_array(*down);
        let ax = a * half;
        let dn = d * half;
        // The face's top-left corner, and the vectors that walk across and down
        // it -- which is exactly what the placement wants.
        let corner = centre + n * half - ax - dn;
        let maps = bake_stone(
            stone,
            &Placement {
                origin: corner.to_array(),
                across: (ax * 2.0).to_array(),
                down: (dn * 2.0).to_array(),
            },
            tile,
            uniform,
        );

        let (cx, cy) = (f as u32 % ATLAS.0, f as u32 / ATLAS.0);
        for (dst, src) in [
            (&mut albedo, &maps.albedo),
            (&mut normal, &maps.normal),
            (&mut orm, &maps.metallic_roughness),
        ] {
            for y in 0..tile {
                for x in 0..tile {
                    let s = ((y * tile + x) * 4) as usize;
                    let d = (((cy * tile + y) * atlas.x + cx * tile + x) * 4) as usize;
                    dst[d..d + 4].copy_from_slice(&src.pixels[s..s + 4]);
                }
            }
        }
    }

    let upload = |data: Vec<u8>, srgb: bool| {
        Image::new(
            Extent3d {
                width: atlas.x,
                height: atlas.y,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            if srgb {
                TextureFormat::Rgba8UnormSrgb
            } else {
                TextureFormat::Rgba8Unorm
            },
            RenderAssetUsages::RENDER_WORLD,
        )
    };

    materials.add(StandardMaterial {
        base_color_texture: Some(images.add(upload(albedo, true))),
        normal_map_texture: Some(images.add(upload(normal, false))),
        metallic_roughness_texture: Some(images.add(upload(orm, false))),
        base_color: Color::WHITE,
        perceptual_roughness: 1.0,
        metallic: 1.0,
        ..default()
    })
}
