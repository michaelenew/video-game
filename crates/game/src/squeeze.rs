//! **The characters' textures, squeezed** (2026-10-21, exploration 0024
//! sec. 11): each one the models load is put into one of the GPU's own
//! compressed formats, and its copy in memory let go.
//!
//! A texture comes out of its file as raw pixels, four bytes each -- a
//! 2048-square one is seventeen megabytes -- and Bevy keeps that copy in
//! memory as well as the one it uploads. A character has five of them. The
//! block formats every desktop GPU reads (BC1 for colour, BC3 where there is
//! alpha, BC5 for a normal map's two channels) hold the same picture in a
//! half or an eighth of that, and are read straight from memory by the GPU
//! with no unpacking, so the saving is in what the GPU holds, not only in
//! what is on disk.
//!
//! Squeezed here, as each texture arrives, rather than in the file: a JPEG
//! is smaller on disk than any block format, and the browser download is
//! the cost that matters most there. The fast fit (`RangeFit`), one texture
//! a frame. On a GPU without the block formats (a phone's browser) the
//! texture keeps its pixels, and only its copy in memory goes.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{TextureFormat, WgpuFeatures};
use bevy::render::renderer::RenderDevice;

use crate::ink::InkMaterial;
use crate::models::Rigs;
use crate::still::SkinMaterial;

/// What a texture is for, which decides its format.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Use {
    /// Colour, with or without alpha: base colour, glow.
    Colour,
    /// A normal map: two channels are all a unit normal needs.
    Normal,
    /// Numbers that are not a colour: metal and roughness.
    Data,
}

/// The textures waiting to be squeezed, the ones already done, and how much
/// they came to before and after (for `--profile` and the report).
#[derive(Resource, Default)]
pub struct Squeeze {
    pending: Vec<(Handle<Image>, Use)>,
    done: Vec<AssetId<Image>>,
    pub before: usize,
    pub after: usize,
}

impl Squeeze {
    /// Every texture a model's material reads, to be squeezed once.
    pub fn take(&mut self, m: &StandardMaterial) {
        let uses = [
            (&m.base_color_texture, Use::Colour),
            (&m.emissive_texture, Use::Colour),
            (&m.normal_map_texture, Use::Normal),
            (&m.metallic_roughness_texture, Use::Data),
        ];
        for (handle, used) in uses {
            let Some(handle) = handle else {
                continue;
            };
            let id = handle.id();
            if self.done.contains(&id) || self.pending.iter().any(|(h, _)| h.id() == id) {
                continue;
            }
            self.pending.push((handle.clone(), used));
        }
    }
}

/// Squeeze one waiting texture, if one has arrived; then have every model
/// material made again, since a material reads its normal map's format.
pub fn squeeze(
    mut squeeze: ResMut<Squeeze>,
    device: Option<Res<RenderDevice>>,
    mut images: ResMut<Assets<Image>>,
    rigs: Res<Rigs>,
    mut skins: ResMut<Assets<SkinMaterial>>,
    mut inks: ResMut<Assets<InkMaterial>>,
) {
    let blocks =
        device.is_some_and(|d| d.features().contains(WgpuFeatures::TEXTURE_COMPRESSION_BC));
    let Some(k) = squeeze
        .pending
        .iter()
        .position(|(h, _)| images.get(h).is_some_and(|i| i.data.is_some()))
    else {
        return;
    };
    let (handle, used) = squeeze.pending.swap_remove(k);
    let Some(image) = images.get_mut(&handle) else {
        return;
    };
    let before = image.data.as_ref().map_or(0, Vec::len);
    if blocks {
        if let Some((data, format)) = squeezed(image, used) {
            image.data = Some(data);
            image.texture_descriptor.format = format;
        }
    }
    // Uploaded once more, and from then on only the GPU's copy is kept.
    image.asset_usage = RenderAssetUsages::RENDER_WORLD;
    let after = image.data.as_ref().map_or(0, Vec::len);
    squeeze.before += before;
    squeeze.after += after;
    squeeze.done.push(handle.id());
    if squeeze.pending.is_empty() && crate::platform::flag("--profile") {
        eprintln!(
            "squeezed {} textures: {:.1} MB to {:.1} MB ({})",
            squeeze.done.len(),
            squeeze.before as f32 / 1e6,
            squeeze.after as f32 / 1e6,
            if blocks {
                "block formats"
            } else {
                "no block formats on this GPU"
            },
        );
    }
    for look in rigs.looks.iter().flatten() {
        skins.get_mut(look);
    }
    for ink in rigs.inks.iter().flatten().flatten() {
        inks.get_mut(ink);
    }
}

/// An image in its block format: BC5 for a normal map, BC3 for colour with
/// any alpha in it, BC1 for the rest. `None` for anything that is not plain
/// eight-bit RGBA in whole blocks.
fn squeezed(image: &Image, used: Use) -> Option<(Vec<u8>, TextureFormat)> {
    let size = image.texture_descriptor.size;
    let (w, h) = (size.width as usize, size.height as usize);
    if w % 4 != 0 || h % 4 != 0 || size.depth_or_array_layers != 1 {
        return None;
    }
    let srgb = match image.texture_descriptor.format {
        TextureFormat::Rgba8UnormSrgb => true,
        TextureFormat::Rgba8Unorm => false,
        _ => return None,
    };
    let pixels = image.data.as_ref()?;
    let alpha = pixels.chunks_exact(4).any(|p| p[3] < 255);
    let (block, format) = match used {
        Use::Normal => (texpresso::Format::Bc5, TextureFormat::Bc5RgUnorm),
        _ if alpha && srgb => (texpresso::Format::Bc3, TextureFormat::Bc3RgbaUnormSrgb),
        _ if alpha => (texpresso::Format::Bc3, TextureFormat::Bc3RgbaUnorm),
        Use::Colour | Use::Data if srgb => {
            (texpresso::Format::Bc1, TextureFormat::Bc1RgbaUnormSrgb)
        }
        Use::Colour | Use::Data => (texpresso::Format::Bc1, TextureFormat::Bc1RgbaUnorm),
    };
    let mut out = vec![0; block.compressed_size(w, h)];
    let params = texpresso::Params {
        algorithm: texpresso::Algorithm::RangeFit,
        ..Default::default()
    };
    block.compress(pixels, w, h, params, &mut out);
    Some((out, format))
}
