//! **The near bake**: the original brush, baked once per tile of land close
//! to the eye.
//!
//! The painted looks read their strokes off tiling sheets (`look::strokes`,
//! `paint.wgsl`), which is cheap and, past a few tens of metres, the same
//! picture as working every stroke out per pixel. Close up it is not: the
//! sheets hold six angles where the brush has every angle, and sixty-four
//! texels a stroke where the bristles want more. And there is no baking the
//! whole valley at that fineness -- a square kilometre is tens of billions of
//! texels. So the near band is a window that follows the player: every tile
//! of land within [`REACH`] gets the original brush (`look::strokes::broad`,
//! every stroke at its own angle) rendered once into a texture of its own,
//! on a background thread, and the surface shader reads that texture in
//! place of the sheets, fading back to them over the band's last metres.
//! Beyond the band the sheets carry on; that is the far bake, and the middle
//! ground is a wash either way.
//!
//! What a texture costs: [`RES`] squared RGBA with mipmaps, four texels a
//! centimetre; about five megabytes a tile, a few dozen tiles in the band.
//! What a bake costs: most of a second of one core (measured below), off
//! the main thread, [`IN_FLIGHT`] at a time nearest first; a tile shows the
//! sheets until its bake lands. The browser build has no background thread
//! to give it and keeps the sheets (`platform::bakes_near`).
//!
//! The strokes are a fixed size in the world here, the size they are at
//! [`BAKE_DIST`] from the eye, where the sheets keep them a fixed size on
//! the screen; the fade hides the join.

use std::collections::HashMap;

use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, poll_once};
use sim::atlas::{self, TILE_M};

use crate::paint::Paint;
use crate::stream::Stream;

/// How far from the eye a tile is baked, in metres, measured flat to the
/// tile's middle. The shader fades to the sheets over the last eight.
pub const REACH: f32 = 20.0;
/// A baked tile is kept this much further out, so the band's edge does not
/// bake and drop the same tile as the player paces.
const SLACK: f32 = 12.0;
/// Texels across a tile: a sixteen-metre tile at four texels a centimetre.
/// At the feet that is five pixels a texel, and the edge of a stroke is that
/// much softer than the original's; 2048 halves it for four times the
/// memory and the bake.
pub const RES: u32 = 1024;
/// The distance from the eye the strokes are sized for: the ground at the
/// fighter's feet, four metres off, where the eye rests. Further out the
/// strokes come out smaller than the sheets' until the fade, which the owner
/// judged a wash either way.
pub const BAKE_DIST: f32 = 4.0;
/// How many tiles bake at once.
const IN_FLIGHT: usize = 4;

/// The near tiles: baking, or baked and painted onto their tile.
#[derive(Resource, Default)]
pub struct Near {
    tiles: HashMap<(i32, i32), NearTile>,
    /// The map origin the paints' rectangles were last written against.
    origin: Option<Vec3>,
}

enum NearTile {
    Baking(Task<Image>),
    Baked {
        paint: Handle<Paint>,
        /// The tile entity the paint is on, if it has been put on one; a
        /// tile rebuilt finer or coarser is a new entity and gets it again.
        on: Option<Entity>,
    },
}

impl Near {
    /// How many tiles are baked and how many are baking: the dev overlay.
    pub fn loaded(&self) -> (usize, usize) {
        let baking = self
            .tiles
            .values()
            .filter(|t| matches!(t, NearTile::Baking(_)))
            .count();
        (self.tiles.len() - baking, baking)
    }
}

/// Bake the tiles that have come within reach, paint the ones that have
/// landed, and let go of the ones that have passed out of it. After
/// `stream::stream`, which owns the tiles.
pub fn near(
    mut commands: Commands,
    sim: Res<crate::Sim>,
    st: Res<Stream>,
    mut near: ResMut<Near>,
    camera: Query<&Transform, With<crate::MainCamera>>,
    mut images: ResMut<Assets<Image>>,
    mut paints: ResMut<Assets<Paint>>,
) {
    let w = &sim.cur;
    let on = crate::wash::painted() && crate::platform::bakes_near() && w.valley.on;
    if !on {
        if !near.tiles.is_empty() {
            for (_, tile) in near.tiles.drain() {
                if let (NearTile::Baked { on: Some(e), .. }, Some(ground)) = (tile, st.ground()) {
                    if let Ok(mut ec) = commands.get_entity(e) {
                        ec.insert(MeshMaterial3d(ground.clone()));
                    }
                }
            }
        }
        return;
    }
    let Ok(eye) = camera.single() else {
        return;
    };
    let o = w.map_origin();
    let origin = Vec3::new(
        o.x.to_f32_for_render(),
        o.y.to_f32_for_render(),
        o.z.to_f32_for_render(),
    );
    let at = eye.translation + origin;
    let size = TILE_M as f32;
    let flat = |t: (i32, i32)| {
        let (cx, cz) = ((t.0 as f32 + 0.5) * size, (t.1 as f32 + 0.5) * size);
        ((cx - at.x).powi(2) + (cz - at.z).powi(2)).sqrt()
    };

    // The map moved under the world: every paint's rectangle, written in
    // world coordinates, moves with it.
    if near.origin != Some(origin) {
        if let Some(old) = near.origin {
            for tile in near.tiles.values() {
                if let NearTile::Baked { paint, .. } = tile {
                    if let Some(p) = paints.get_mut(paint) {
                        p.near.x += old.x - origin.x;
                        p.near.y += old.z - origin.z;
                    }
                }
            }
        }
        near.origin = Some(origin);
    }
    // Let go: past the slack, or no tile there any more.
    let gone: Vec<(i32, i32)> = near
        .tiles
        .keys()
        .copied()
        .filter(|t| flat(*t) > REACH + SLACK || !st.tiles().contains_key(t))
        .collect();
    for t in gone {
        if let Some(NearTile::Baked { on: Some(e), .. }) = near.tiles.remove(&t) {
            if let (Some(ground), Ok(mut ec)) = (st.ground(), commands.get_entity(e)) {
                ec.insert(MeshMaterial3d(ground.clone()));
            }
        }
    }
    // Land the bakes that have finished, and paint them on.
    let broad_size = crate::wash::style().broad_size;
    for (&t, tile) in near.tiles.iter_mut() {
        if let NearTile::Baking(task) = tile {
            if let Some(image) = block_on(poll_once(task)) {
                let handle = images.add(image);
                let paint = paints.add(Paint {
                    near: Vec4::new(
                        t.0 as f32 * size - origin.x,
                        t.1 as f32 * size - origin.z,
                        size,
                        REACH,
                    ),
                    near_tex: Some(handle),
                    two_sided: true,
                    ..Paint::white()
                });
                *tile = NearTile::Baked { paint, on: None };
            }
        }
        if let NearTile::Baked { paint, on } = tile {
            let e = st.tiles().get(&t).map(|(e, _, _)| *e);
            if let Some(e) = e {
                if *on != Some(e) {
                    if let Ok(mut ec) = commands.get_entity(e) {
                        ec.insert(MeshMaterial3d(paint.clone()));
                        *on = Some(e);
                    }
                }
            }
        }
    }
    // Bake what has come within reach, nearest first.
    let mut baking = near
        .tiles
        .values()
        .filter(|t| matches!(t, NearTile::Baking(_)))
        .count();
    let mut wanted: Vec<((i32, i32), f32)> = st
        .tiles()
        .keys()
        .filter(|t| !near.tiles.contains_key(t))
        .map(|t| (*t, flat(*t)))
        .filter(|(_, d)| *d < REACH)
        .collect();
    wanted.sort_by(|a, b| a.1.total_cmp(&b.1));
    for (t, _) in wanted {
        if baking >= IN_FLIGHT {
            break;
        }
        let task = AsyncComputeTaskPool::get().spawn(async move { bake(t.0, t.1, broad_size) });
        near.tiles.insert(t, NearTile::Baking(task));
        baking += 1;
    }
}

/// **One tile's bake**: the original brush over the tile, `RES` texels
/// across, with its mipmaps. The slow questions -- the ground's height,
/// normal, kind and the way the strokes run -- are asked on a lattice every
/// eighth texel and read between; the strokes themselves are worked out at
/// every texel, which is the cost that matters.
pub fn bake(tx: i32, tz: i32, broad_size: f32) -> Image {
    let atlas = atlas::valley();
    let size = TILE_M as f32;
    let (x0, z0) = (tx as f32 * size, tz as f32 * size);
    let res = RES as usize;
    // The lattice: normal (3), the way the strokes run (3), rigidity, size.
    const STEP: usize = 8;
    let n = res / STEP + 1;
    let lattice: Vec<[f32; 8]> = (0..n * n)
        .map(|k| {
            let (i, j) = (k % n, k / n);
            let x = x0 + (i * STEP) as f32 / res as f32 * size;
            let z = z0 + (j * STEP) as f32 / res as f32 * size;
            let (normal, slope) = crate::land::ground_normal(atlas, x, z);
            let (flow, kind) = crate::land::stroke_kind_at(atlas, x, z, slope);
            let h = atlas.relief_at(fx(x), fx(z)).to_f32_for_render();
            let way = look::strokes::way_on_ground([x, h, z], normal, flow, kind[0]);
            [
                normal[0], normal[1], normal[2], way[0], way[1], way[2], kind[0], kind[1],
            ]
        })
        .collect();
    let read = |u: f32, v: f32| -> [f32; 8] {
        let (u, v) = (
            (u / STEP as f32).clamp(0.0, n as f32 - 1.001),
            (v / STEP as f32).clamp(0.0, n as f32 - 1.001),
        );
        let (i, j) = (u as usize, v as usize);
        let (fu, fv) = (u - i as f32, v - j as f32);
        let (a, b, c, d) = (
            lattice[j * n + i],
            lattice[j * n + i + 1],
            lattice[(j + 1) * n + i],
            lattice[(j + 1) * n + i + 1],
        );
        let mut out = [0.0; 8];
        for k in 0..6 {
            let top = a[k] + (b[k] - a[k]) * fu;
            let bottom = c[k] + (d[k] - c[k]) * fu;
            out[k] = top + (bottom - top) * fv;
        }
        // The kind of ground is one thing or another, never halfway: read
        // from the nearest lattice point. (Read between, every texel of a
        // border band had a rigidity of its own, and the brush worked its
        // strokes out afresh for each: five times the bake.)
        let nearest = match (fu < 0.5, fv < 0.5) {
            (true, true) => a,
            (false, true) => b,
            (true, false) => c,
            (false, false) => d,
        };
        out[6] = nearest[6];
        out[7] = nearest[7];
        out
    };
    let mut level = vec![0u8; res * res * 4];
    // The rows, in bands across every core: each band with a brush of its
    // own, since the brush remembers what it has worked out.
    const BAND: usize = 64;
    let pool = bevy::tasks::ComputeTaskPool::get_or_init(bevy::tasks::TaskPool::new);
    let read = &read;
    pool.scope(|scope| {
        for (band, rows) in level.chunks_mut(BAND * res * 4).enumerate() {
            scope.spawn(async move {
                let z_lo = z0 + (band * BAND) as f32 / res as f32 * size;
                let z_hi = z0 + ((band + 1) * BAND) as f32 / res as f32 * size;
                let mut brush = look::strokes::Brush::over([x0, z_lo], [x0 + size, z_hi]);
                for (r, row) in rows.chunks_mut(res * 4).enumerate() {
                    let j = band * BAND + r;
                    for (i, px) in row.chunks_mut(4).enumerate() {
                        let (u, v) = (i as f32 + 0.5, j as f32 + 0.5);
                        let x = x0 + u / res as f32 * size;
                        let z = z0 + v / res as f32 * size;
                        let l = read(u, v);
                        let turn = l[5].atan2(l[3]);
                        let b = brush.broad([x, z], turn, l[6], l[7], BAKE_DIST, broad_size);
                        px[0] = byte(b[0]);
                        px[1] = byte(b[1]);
                        px[2] = byte(b[2]);
                        px[3] = 255;
                    }
                }
            });
        }
    });
    // The mipmaps: each level the mean of two by two of the last, down to
    // one texel, so the ground at a slant samples smoothly.
    let mut data = level.clone();
    let mut levels = 1;
    let mut side = res;
    while side > 1 {
        let half = side / 2;
        let mut next = vec![0u8; half * half * 4];
        for j in 0..half {
            for i in 0..half {
                for c in 0..4 {
                    let at = |x: usize, y: usize| level[(y * side + x) * 4 + c] as u32;
                    let sum = at(2 * i, 2 * j)
                        + at(2 * i + 1, 2 * j)
                        + at(2 * i, 2 * j + 1)
                        + at(2 * i + 1, 2 * j + 1);
                    next[(j * half + i) * 4 + c] = ((sum + 2) / 4) as u8;
                }
            }
        }
        data.extend_from_slice(&next);
        level = next;
        side = half;
        levels += 1;
    }
    // Built by hand: `Image::new` wants one level's worth of texels.
    let mut image = Image::default();
    image.texture_descriptor.size = Extent3d {
        width: RES,
        height: RES,
        depth_or_array_layers: 1,
    };
    image.texture_descriptor.dimension = TextureDimension::D2;
    image.texture_descriptor.format = TextureFormat::Rgba8Unorm;
    image.texture_descriptor.mip_level_count = levels;
    image.asset_usage = RenderAssetUsages::RENDER_WORLD;
    image.data = Some(data);
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        anisotropy_clamp: 4,
        ..default()
    });
    image
}

fn fx(v: f32) -> sim::Fx {
    sim::Fx::from_raw((v * 65536.0) as i32)
}

fn byte(v: f32) -> u8 {
    ((v.clamp(-1.0, 1.0) + 1.0) * 127.5 + 0.5) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tile bakes, with its mipmaps, in the time the module's note says.
    #[test]
    fn a_tile_bakes_with_its_mipmaps() {
        let started = std::time::Instant::now();
        let image = bake(0, 0, 0.04);
        let took = started.elapsed();
        let levels = image.texture_descriptor.mip_level_count;
        assert_eq!(levels, (RES as f32).log2() as u32 + 1);
        let data = image.data.as_ref().expect("texels");
        // Every level's texels, four bytes each: a third more than the first.
        let first = (RES * RES * 4) as usize;
        assert!(
            data.len() > first && data.len() < first * 4 / 3 + 4,
            "{}",
            data.len()
        );
        // Strokes were laid: the shade varies.
        let (lo, hi) = data[..first]
            .iter()
            .step_by(4)
            .fold((255u8, 0u8), |(lo, hi), &v| (lo.min(v), hi.max(v)));
        assert!(hi - lo > 60, "a flat bake: {lo} to {hi}");
        eprintln!("a tile baked in {took:?}");
        assert!(took.as_secs_f32() < 10.0, "too slow: {took:?}");
    }
}
