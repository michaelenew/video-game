//! **The elements as things, not primitives** (2026-10-21, the owner's word:
//! every class's effects "in the correct style"; exploration 0024 §10).
//!
//! An effect is drawn as pieces -- a shape and a skin at the size its hit
//! test reads (`effect_piece`). Until now a shape was a stock cylinder, ball
//! or cone and a skin a flat colour, so a fire pillar was an orange tin and a
//! raised stone a grey drum. This module is what they are drawn as instead:
//!
//! - **Meshes** made here, each filling the same unit volume as the stock
//!   shape it stands in for (a unit cylinder, a unit ball), so a piece
//!   scaled to its hit test is still exactly as big as it hits: a column of
//!   flame tongues, a ball of fire, a patch of broken ground, a rock, a twist
//!   of wind, a ball of wind, a disc of wind, a comet.
//!   Each carries a `u` round it and a `v` along it for its texture.
//! - **Textures** painted here, procedurally, wrapping so they can scroll:
//!   flame (tongues rising, white-hot at the root), wind (pale streaks),
//!   stone (mottled and cracked), embers in the cracks of a lit stone.
//! - **Flow**: the fire's, the wind's and the beam's textures scroll each
//!   frame, on the renderer's clock -- texture on a surface whose every
//!   place and size comes from the snapshot, so a rollback cannot make it
//!   pop (`docs/design/animation.md`).
//!
//! Which mesh a piece is drawn with is `dressed`: the shape the effect asked
//! for, read with the skin it is in -- a cylinder of fire is a flame, a
//! ball of air is a ball of wind -- so every effect in an element, whatever
//! class makes it, is drawn as that element in one place.

use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use std::f32::consts::TAU;

// --- Meshes -------------------------------------------------------------------

struct Builder {
    at: Vec<[f32; 3]>,
    uv: Vec<[f32; 2]>,
    colour: Vec<[f32; 4]>,
    index: Vec<u32>,
}

impl Builder {
    fn new() -> Builder {
        Builder {
            at: Vec::new(),
            uv: Vec::new(),
            colour: Vec::new(),
            index: Vec::new(),
        }
    }

    fn vert(&mut self, p: Vec3, uv: [f32; 2], c: [f32; 4]) -> u32 {
        self.at.push(p.to_array());
        self.uv.push(uv);
        self.colour.push(c);
        (self.at.len() - 1) as u32
    }

    fn quad(&mut self, a: u32, b: u32, c: u32, d: u32) {
        self.index.extend_from_slice(&[a, b, c, a, c, d]);
    }

    /// Pull every point in to a unit cylinder's room (radius half, height
    /// one, centred), so the drawing is never bigger than what it stands for.
    fn fit_column(&mut self) {
        for p in &mut self.at {
            let r = (p[0] * p[0] + p[2] * p[2]).sqrt();
            if r > 0.5 {
                p[0] *= 0.5 / r;
                p[2] *= 0.5 / r;
            }
            p[1] = p[1].clamp(-0.5, 0.5);
        }
    }

    /// The same, into a unit ball's room.
    fn fit_ball(&mut self) {
        for p in &mut self.at {
            let r = Vec3::from(*p).length();
            if r > 0.5 {
                *p = (Vec3::from(*p) * (0.5 / r)).to_array();
            }
        }
    }

    /// Smooth normals from the faces, unless `flat`, which gives every
    /// triangle its own corners (a cut stone).
    fn mesh(self, flat: bool) -> Mesh {
        let (at, uv, colour, index) = if flat {
            let mut a = Vec::new();
            let mut u = Vec::new();
            let mut c = Vec::new();
            for &i in &self.index {
                a.push(self.at[i as usize]);
                u.push(self.uv[i as usize]);
                c.push(self.colour[i as usize]);
            }
            let n = a.len() as u32;
            (a, u, c, (0..n).collect())
        } else {
            (self.at, self.uv, self.colour, self.index)
        };
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, at)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colour)
        .with_inserted_indices(Indices::U32(index));
        if flat {
            mesh.duplicate_vertices();
            mesh.compute_flat_normals();
        } else {
            mesh.compute_smooth_normals();
        }
        mesh
    }
}

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

/// Fire's heat, root to tip, carried in the vertices so the texture can flow
/// through it without the gradient moving: white-yellow at the root, orange,
/// a red going out at the tips.
fn heat(up: f32) -> [f32; 4] {
    let up = up.clamp(0.0, 1.0);
    [
        1.0,
        1.0 - 0.55 * up,
        0.75 * (1.0 - up).powi(2) + 0.1,
        1.0 - up.powf(2.5),
    ]
}

/// A number from 0 to 1, the same every time for the same `k`.
fn hash(k: u32) -> f32 {
    let mut x = k.wrapping_mul(0x9E37_79B9);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x & 0xFFFF) as f32 / 65535.0
}

/// A ring of `sides` points round the y axis at height `y` and radius `r`.
fn ring(b: &mut Builder, sides: u32, y: f32, r: impl Fn(f32) -> f32, v: f32, c: [f32; 4]) -> u32 {
    let first = b.at.len() as u32;
    for i in 0..=sides {
        let a = i as f32 / sides as f32 * TAU;
        let rr = r(a);
        b.vert(
            Vec3::new(a.cos() * rr, y, a.sin() * rr),
            [i as f32 / sides as f32, v],
            c,
        );
    }
    first
}

fn join_rings(b: &mut Builder, lo: u32, hi: u32, sides: u32) {
    for i in 0..sides {
        b.quad(lo + i, hi + i, hi + i + 1, lo + i + 1);
    }
}

/// **A column of flame**, in a unit cylinder's room (radius half, height
/// one, centred): wide at the root, drawing in as it rises, its rim wavering
/// round, its top broken into tongues of uneven height. The texture's `v`
/// runs up it, so flame scrolled along `v` rises.
pub fn flame() -> Mesh {
    let mut b = Builder::new();
    let (sides, rows) = (24u32, 14u32);
    let mut prev = None;
    for j in 0..=rows {
        let t = j as f32 / rows as f32;
        // Room left at the top for the tongues' tips.
        let y = t * 0.74 - 0.5;
        let taper = 0.5 * (1.0 - 0.55 * t.powf(1.4));
        let first = ring(
            &mut b,
            sides,
            y,
            |a| {
                let waver =
                    1.0 + 0.10 * (5.0 * a + t * 7.0).sin() + 0.06 * (11.0 * a - t * 5.0).sin();
                // The top ring is tongues: some reach the top, some fall short.
                let tongue = if j == rows {
                    0.25 + 0.75 * (3.0 * a).sin().abs()
                } else {
                    1.0
                };
                taper * waver * tongue
            },
            1.0 - t,
            heat(t * 0.8),
        );
        if let Some(p) = prev {
            join_rings(&mut b, p, first, sides);
        }
        prev = Some(first);
    }
    // The tongues' tips drawn up past the top in points, uneven.
    let top = prev.expect("rows");
    for i in 0..=sides {
        let k = (top + i) as usize;
        let n = i % sides;
        let lift =
            0.08 + 0.18 * hash(n * 7 + 1) * (3.0 * n as f32 / sides as f32 * TAU).sin().abs();
        b.at[k][1] += lift;
        b.colour[k] = heat(1.0);
    }
    b.fit_column();
    b.mesh(false)
}

/// **A ball of fire**, in a unit ball's room: a sphere whose surface is
/// thrown out into tongues, longer above than below, so it reads as burning
/// rather than as a glowing bead.
pub fn fireball() -> Mesh {
    let mut b = Builder::new();
    let (sides, rows) = (20u32, 12u32);
    let mut prev = None;
    for j in 0..=rows {
        let t = j as f32 / rows as f32;
        let phi = t * std::f32::consts::PI;
        let y = -phi.cos() * 0.42;
        let base = phi.sin() * 0.42;
        let first = ring(
            &mut b,
            sides,
            y,
            |a| base * (1.0 + 0.14 * (6.0 * a + t * 9.0).sin()),
            1.0 - t,
            heat(t * t * 0.9),
        );
        if let Some(p) = prev {
            join_rings(&mut b, p, first, sides);
        }
        prev = Some(first);
    }
    // Tongues: the upper half's vertices pushed out and up, unevenly.
    for (k, p) in b.at.iter_mut().enumerate() {
        let v = Vec3::from(*p);
        let up = (v.y / 0.42).max(0.0);
        let n = hash(k as u32 * 13 + 5);
        let out = v.normalize_or_zero() * (0.06 * n) + Vec3::Y * (0.14 * up * up * n);
        *p = (v + out).to_array();
    }
    b.fit_ball();
    b.mesh(false)
}

/// **A rock**, in a unit ball's room: an icosahedron-like lump, faceted and
/// uneven.
pub fn rock(seed: u32) -> Mesh {
    let mut b = Builder::new();
    let (sides, rows) = (7u32, 5u32);
    let mut prev = None;
    for j in 0..=rows {
        let t = j as f32 / rows as f32;
        let phi = t * std::f32::consts::PI;
        let y = -phi.cos() * 0.45;
        let base = phi.sin() * 0.5;
        let first = ring(
            &mut b,
            sides,
            y,
            |a| base * (0.75 + 0.25 * hash(seed + ((a * 3.0) as u32) * 11 + j * 3)),
            t,
            WHITE,
        );
        if let Some(p) = prev {
            join_rings(&mut b, p, first, sides);
        }
        prev = Some(first);
    }
    b.fit_ball();
    b.mesh(true)
}

/// **Broken ground**, in a unit cylinder's room: a cluster of slabs
/// heaved up at angles out of the floor, the tallest reaching the top, as
/// earth looks when something has come up through it.
pub fn rubble() -> Mesh {
    let mut b = Builder::new();
    for k in 0..9u32 {
        let a = hash(k * 17 + 3) * TAU;
        let d = 0.12 + 0.30 * hash(k * 29 + 1);
        let centre = Vec3::new(a.cos() * d, -0.5, a.sin() * d);
        let size = Vec3::new(
            0.12 + 0.12 * hash(k * 5),
            0.4 + 0.6 * hash(k * 7 + 2),
            0.10 + 0.10 * hash(k * 3 + 9),
        );
        let tilt = Quat::from_rotation_y(hash(k * 41) * TAU)
            * Quat::from_rotation_z(0.3 * (hash(k * 43) - 0.5));
        // A slab: a box, its top corners knocked unevenly.
        let corners: Vec<Vec3> = (0..8u32)
            .map(|c| {
                let x = if c & 1 == 0 { -0.5 } else { 0.5 };
                let y = if c & 2 == 0 {
                    0.0
                } else {
                    1.0 - 0.3 * hash(k * 13 + c)
                };
                let z = if c & 4 == 0 { -0.5 } else { 0.5 };
                centre + tilt * (Vec3::new(x, y, z) * size)
            })
            .collect();
        let v: Vec<u32> = corners
            .iter()
            .enumerate()
            .map(|(c, p)| b.vert(*p, [(c & 1) as f32, (c >> 1 & 1) as f32], WHITE))
            .collect();
        for &[a, bb, c, d] in &[
            [0, 2, 3, 1],
            [4, 5, 7, 6],
            [0, 1, 5, 4],
            [2, 6, 7, 3],
            [0, 4, 6, 2],
            [1, 3, 7, 5],
        ] {
            b.quad(v[a], v[bb], v[c], v[d]);
        }
    }
    b.fit_column();
    b.mesh(true)
}

/// A ribbon along `path` (points), `width` across, turned so its flat
/// faces `side(i)`; `u` across it and `v` along it.
fn ribbon(
    b: &mut Builder,
    path: &[Vec3],
    width: impl Fn(f32) -> f32,
    side: impl Fn(usize) -> Vec3,
) {
    let n = path.len();
    let first = b.at.len() as u32;
    for (i, p) in path.iter().enumerate() {
        let t = i as f32 / (n - 1) as f32;
        let s = side(i) * width(t) * 0.5;
        let fade = [
            1.0,
            1.0,
            1.0,
            (std::f32::consts::PI * t).sin().max(0.0).sqrt(),
        ];
        b.vert(*p - s, [0.0, t * 3.0], fade);
        b.vert(*p + s, [1.0, t * 3.0], fade);
    }
    for i in 0..(n - 1) as u32 {
        let k = first + 2 * i;
        b.quad(k, k + 2, k + 3, k + 1);
    }
}

/// **A twist of wind**, in a unit cylinder's room: three ribbons spiralling
/// up round the axis, each its own pitch, thinning at their ends.
pub fn whirl() -> Mesh {
    let mut b = Builder::new();
    for k in 0..3u32 {
        let phase = k as f32 / 3.0 * TAU;
        let turns = 1.3 + 0.4 * k as f32;
        let path: Vec<Vec3> = (0..=40)
            .map(|i| {
                let t = i as f32 / 40.0;
                let a = phase + t * turns * TAU;
                let r = 0.42 + 0.06 * (t * 9.0 + k as f32).sin();
                Vec3::new(a.cos() * r, t - 0.5, a.sin() * r)
            })
            .collect();
        let pts = path.clone();
        ribbon(
            &mut b,
            &path,
            |t| 0.12 * (std::f32::consts::PI * t).sin() + 0.01,
            |i| {
                let p = pts[i];
                Vec3::new(p.x, 0.0, p.z)
                    .normalize_or_zero()
                    .cross(Vec3::Y)
                    .normalize_or_zero()
                    * 0.3
                    + Vec3::Y
            },
        );
    }
    b.fit_column();
    b.mesh(false)
}

/// **A ball of wind**, in a unit ball's room: four rings of streaming air at
/// different tilts round a centre.
pub fn windball() -> Mesh {
    let mut b = Builder::new();
    for k in 0..4u32 {
        let tilt =
            Quat::from_rotation_x(0.9 * k as f32 + 0.3) * Quat::from_rotation_z(0.7 * k as f32);
        let r = 0.42 + 0.06 * k as f32 / 3.0;
        let path: Vec<Vec3> = (0..=36)
            .map(|i| {
                let a = i as f32 / 36.0 * TAU * 0.85;
                tilt * Vec3::new(a.cos() * r, 0.0, a.sin() * r)
            })
            .collect();
        let up = tilt * Vec3::Y;
        ribbon(
            &mut b,
            &path,
            |t| 0.16 * (std::f32::consts::PI * t).sin() + 0.02,
            |_| up,
        );
    }
    b.fit_ball();
    b.mesh(false)
}

/// **A disc of wind**, in a unit cylinder's room squashed flat: spiral arms
/// of air from the rim in to the middle, lying in the disc's plane, over a
/// faint ring at the rim.
pub fn winddisc() -> Mesh {
    let mut b = Builder::new();
    for k in 0..5u32 {
        let phase = k as f32 / 5.0 * TAU;
        let path: Vec<Vec3> = (0..=24)
            .map(|i| {
                let t = i as f32 / 24.0;
                let a = phase + t * 2.2;
                let r = 0.46 * (1.0 - 0.8 * t);
                Vec3::new(a.cos() * r, 0.0, a.sin() * r)
            })
            .collect();
        // Flat in the plane: across the arm is along the radius.
        let pts = path.clone();
        ribbon(
            &mut b,
            &path,
            |t| 0.14 * (1.0 - t) + 0.02,
            |i| Vec3::new(pts[i].x, 0.0, pts[i].z).normalize_or(Vec3::X),
        );
    }
    let rim: Vec<Vec3> = (0..=48)
        .map(|i| {
            let a = i as f32 / 48.0 * TAU;
            Vec3::new(a.cos() * 0.45, 0.0, a.sin() * 0.45)
        })
        .collect();
    ribbon(
        &mut b,
        &rim,
        |_| 0.08,
        |i| Vec3::new(rim[i].x, 0.0, rim[i].z).normalize_or(Vec3::X),
    );
    b.fit_column();
    b.mesh(false)
}

/// **A comet**, in a unit cylinder's room along `+y`: a round head at the
/// front and a tail streaming back from it, thinning to nothing -- what a
/// bolt in flight looks like.
pub fn comet() -> Mesh {
    let mut b = Builder::new();
    let (sides, rows) = (16u32, 14u32);
    let mut prev = None;
    for j in 0..=rows {
        let t = j as f32 / rows as f32;
        // t runs from the tail (0) to the front of the head (1).
        let y = t - 0.5;
        let r = if t > 0.7 {
            let h = (t - 0.7) / 0.3;
            0.5 * (1.0 - h * h).max(0.0).sqrt()
        } else {
            0.5 * (t / 0.7).powf(1.8)
        };
        let first = ring(
            &mut b,
            sides,
            y,
            |a| r * (1.0 + 0.1 * (4.0 * a + t * 6.0).sin()),
            1.0 - t,
            heat(1.0 - t),
        );
        if let Some(p) = prev {
            join_rings(&mut b, p, first, sides);
        }
        prev = Some(first);
    }
    b.fit_column();
    b.mesh(false)
}

/// **A thorn**, in a unit cone's room (base radius half at the bottom, a
/// point at the top): five ridges running up it, twisting a little, so it
/// reads as something grown rather than turned on a lathe.
pub fn thorn() -> Mesh {
    let mut b = Builder::new();
    let (sides, rows) = (20u32, 10u32);
    let mut prev = None;
    for j in 0..=rows {
        let t = j as f32 / rows as f32;
        let y = t - 0.5;
        let first = ring(
            &mut b,
            sides,
            y,
            |a| {
                let ridge = 0.72 + 0.28 * (5.0 * (a + t * 0.9)).cos().max(0.0);
                0.5 * (1.0 - t).powf(1.15) * ridge
            },
            1.0 - t,
            WHITE,
        );
        if let Some(p) = prev {
            join_rings(&mut b, p, first, sides);
        }
        prev = Some(first);
    }
    b.mesh(false)
}

/// **A vine**, in a unit cylinder's room along `y`: three strands twisting
/// round each other, with a knot here and there.
pub fn vine() -> Mesh {
    let mut b = Builder::new();
    let (sides, rows) = (8u32, 16u32);
    for k in 0..3u32 {
        let phase = k as f32 / 3.0 * TAU;
        let mut prev = None;
        for j in 0..=rows {
            let t = j as f32 / rows as f32;
            let a = phase + t * TAU * 0.8;
            let centre = Vec3::new(a.cos() * 0.22, t - 0.5, a.sin() * 0.22);
            let knot = 1.0 + 0.5 * (-((t - 0.3 - 0.2 * k as f32) * 9.0).powi(2)).exp();
            let thick = 0.2 * knot;
            let first = b.at.len() as u32;
            for i in 0..=sides {
                let q = i as f32 / sides as f32 * TAU;
                let p = centre + Vec3::new(q.cos(), 0.0, q.sin()) * thick;
                b.vert(p, [i as f32 / sides as f32, t * 2.0], WHITE);
            }
            if let Some(p) = prev {
                join_rings(&mut b, p, first, sides);
            }
            prev = Some(first);
        }
    }
    b.fit_column();
    b.mesh(false)
}

/// **A drop of liquid**, in a unit ball's room: a ball with soft lumps
/// that read as surface tension rather than as a bead.
pub fn droplet() -> Mesh {
    let mut b = Builder::new();
    let (sides, rows) = (20u32, 12u32);
    let mut prev = None;
    for j in 0..=rows {
        let t = j as f32 / rows as f32;
        let phi = t * std::f32::consts::PI;
        let y = -phi.cos() * 0.47;
        let base = phi.sin() * 0.47;
        let first = ring(
            &mut b,
            sides,
            y,
            |a| base * (1.0 + 0.06 * (3.0 * a + 2.0 * phi).sin() + 0.04 * (5.0 * a - phi).cos()),
            t,
            WHITE,
        );
        if let Some(p) = prev {
            join_rings(&mut b, p, first, sides);
        }
        prev = Some(first);
    }
    b.fit_ball();
    b.mesh(false)
}

/// **A splash on the floor**, in a unit ball's room flattened: a ragged,
/// lobed edge and a low dome, so a pool or a stain is a spill rather than a
/// coin.
pub fn puddle(seed: u32) -> Mesh {
    let mut b = Builder::new();
    let sides = 48u32;
    let edge = |a: f32| {
        let mut r = 0.40;
        for k in 1..6u32 {
            let f = (k * 2 + 1) as f32;
            r += 0.06 / k as f32 * (f * a + hash(seed + k) * TAU).sin();
        }
        r.min(0.5)
    };
    let centre = b.vert(Vec3::new(0.0, 0.35, 0.0), [0.5, 0.5], WHITE);
    let rings = [(0.5f32, 0.25f32), (0.85, 0.12), (1.0, 0.0)];
    let mut prev: Option<u32> = None;
    for (k, &(f, y)) in rings.iter().enumerate() {
        let first = b.at.len() as u32;
        for i in 0..=sides {
            let a = i as f32 / sides as f32 * TAU;
            let r = edge(a) * f;
            b.vert(
                Vec3::new(a.cos() * r, y, a.sin() * r),
                [0.5 + 0.5 * a.cos() * f, 0.5 + 0.5 * a.sin() * f],
                WHITE,
            );
        }
        match prev {
            None => {
                for i in 0..sides {
                    b.index
                        .extend_from_slice(&[centre, first + i + 1, first + i]);
                }
            }
            Some(p) => {
                for i in 0..sides {
                    b.quad(p + i, p + i + 1, first + i + 1, first + i);
                }
            }
        }
        let _ = k;
        prev = Some(first);
    }
    b.fit_ball();
    b.mesh(false)
}

/// **A tongue of liquid fire**, in a unit cone's room: the flame drawn to a
/// point -- what the Blood mage's standing essence rises as.
pub fn tongue() -> Mesh {
    let mut b = Builder::new();
    let (sides, rows) = (20u32, 14u32);
    let mut prev = None;
    for j in 0..=rows {
        let t = j as f32 / rows as f32;
        let y = t - 0.5;
        let first = ring(
            &mut b,
            sides,
            y,
            |a| {
                let waver = 1.0 + 0.12 * (4.0 * a + t * 8.0).sin();
                0.5 * (1.0 - t) * waver.min(1.0)
            },
            1.0 - t,
            [1.0, 1.0, 1.0, 1.0 - t.powi(3)],
        );
        if let Some(p) = prev {
            join_rings(&mut b, p, first, sides);
        }
        prev = Some(first);
    }
    b.mesh(false)
}

/// The cross-section of a forged blade, in the unit square: sharp at the
/// edges (`z` = +-0.5), a bevel to the flat, thickest down the middle.
const BLADE_SECTION: [(f32, f32); 8] = [
    (0.0, 0.5),
    (0.3, 0.32),
    (0.5, 0.08),
    (0.5, -0.08),
    (0.0, -0.5),
    (-0.5, -0.08),
    (-0.5, 0.08),
    (-0.3, 0.32),
];

/// **A forged blade**, in a unit cube's room (`x` its thickness, `y` its
/// length, `z` its breadth): edges ground to a bevel, the bevels catching
/// the light in their own plane, narrowing a little toward the point.
pub fn blade() -> Mesh {
    let mut b = Builder::new();
    let rows = 4u32;
    let n = BLADE_SECTION.len() as u32;
    let mut prev = None;
    for j in 0..=rows {
        let t = j as f32 / rows as f32;
        let narrow = 1.0 - 0.12 * t;
        let first = b.at.len() as u32;
        for (i, (x, z)) in BLADE_SECTION
            .iter()
            .chain(BLADE_SECTION.iter().take(1))
            .enumerate()
        {
            b.vert(
                Vec3::new(*x, t - 0.5, z * narrow),
                [i as f32 / n as f32, t],
                WHITE,
            );
        }
        if let Some(p) = prev {
            join_rings(&mut b, p, first, n);
        }
        prev = Some(first);
    }
    b.mesh(true)
}

/// **A blade's point**, in a unit cone's room: the blade's own section
/// drawn to a point, so the edge runs on unbroken to the tip.
pub fn point() -> Mesh {
    let mut b = Builder::new();
    let n = BLADE_SECTION.len() as u32;
    let base = b.at.len() as u32;
    for (i, (x, z)) in BLADE_SECTION
        .iter()
        .chain(BLADE_SECTION.iter().take(1))
        .enumerate()
    {
        b.vert(
            Vec3::new(*x * 0.9, -0.5, *z * 0.9),
            [i as f32 / n as f32, 0.0],
            WHITE,
        );
    }
    let tip = b.vert(Vec3::new(0.0, 0.5, 0.0), [0.5, 1.0], WHITE);
    for i in 0..n {
        b.index.extend_from_slice(&[base + i, tip, base + i + 1]);
    }
    b.mesh(true)
}

/// **A crescent of light**, in a unit cylinder's room squashed flat: a moon
/// thrown edge-first, its back toward `+x` and its horns reaching round to
/// the full width, thinning to nothing at their tips.
pub fn crescent() -> Mesh {
    let mut b = Builder::new();
    let steps = 32u32;
    let first = b.at.len() as u32;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let a = (t - 0.5) * std::f32::consts::PI * 1.1;
        let outer = Vec3::new(a.cos() * 0.5, 0.0, a.sin() * 0.5);
        let width = 0.22 * (std::f32::consts::PI * t).sin();
        let inner = outer - Vec3::new(a.cos(), 0.0, a.sin()) * width - Vec3::X * width * 0.6;
        let c = [1.0, 1.0, 1.0, (std::f32::consts::PI * t).sin().sqrt()];
        b.vert(inner, [0.0, t * 2.0], c);
        b.vert(outer, [1.0, t * 2.0], c);
    }
    for i in 0..steps {
        let k = first + 2 * i;
        b.quad(k, k + 2, k + 3, k + 1);
    }
    b.fit_column();
    b.mesh(false)
}

/// **A star of light**, in a unit ball's room: a bright core with rays
/// thrown out of it in every direction to the ball's edge -- light, not a
/// lamp-glass bead.
pub fn star() -> Mesh {
    let mut b = Builder::new();
    // The core.
    let (sides, rows) = (16u32, 10u32);
    let mut prev = None;
    for j in 0..=rows {
        let t = j as f32 / rows as f32;
        let phi = t * std::f32::consts::PI;
        let first = ring(
            &mut b,
            sides,
            -phi.cos() * 0.3,
            |_| phi.sin() * 0.3,
            t,
            WHITE,
        );
        if let Some(p) = prev {
            join_rings(&mut b, p, first, sides);
        }
        prev = Some(first);
    }
    // The rays: thin four-sided spikes out from the core, on a spiral that
    // covers the ball evenly, of uneven length.
    let rays = 18u32;
    for k in 0..rays {
        let y = 1.0 - 2.0 * (k as f32 + 0.5) / rays as f32;
        let r = (1.0 - y * y).sqrt();
        let a = k as f32 * 2.399_963;
        let dir = Vec3::new(a.cos() * r, y, a.sin() * r);
        let side = dir.any_orthonormal_vector();
        let up = dir.cross(side);
        let long = 0.38 + 0.12 * hash(k * 19 + 3);
        let tip = b.vert(dir * long, [0.5, 1.0], [1.0, 1.0, 1.0, 0.0]);
        let base: Vec<u32> = (0..4)
            .map(|q| {
                let q = q as f32 * TAU / 4.0;
                b.vert(
                    dir * 0.22 + (side * q.cos() + up * q.sin()) * 0.05,
                    [q / TAU, 0.0],
                    WHITE,
                )
            })
            .collect();
        for q in 0..4 {
            b.index
                .extend_from_slice(&[base[q], base[(q + 1) % 4], tip]);
        }
    }
    b.fit_ball();
    b.mesh(false)
}

/// **A cloud**, in a unit ball's room: billows -- balls of different sizes
/// clustered and overlapping, wider than tall, flatter underneath.
pub fn cloud() -> Mesh {
    let mut b = Builder::new();
    for k in 0..7u32 {
        let a = hash(k * 7 + 1) * TAU;
        let d = if k == 0 {
            0.0
        } else {
            0.18 + 0.12 * hash(k * 11)
        };
        let size = if k == 0 {
            0.3
        } else {
            0.14 + 0.1 * hash(k * 13 + 2)
        };
        let centre = Vec3::new(a.cos() * d, 0.05 * hash(k * 17), a.sin() * d);
        let (sides, rows) = (12u32, 8u32);
        let mut prev = None;
        for j in 0..=rows {
            let t = j as f32 / rows as f32;
            let phi = t * std::f32::consts::PI;
            let y = -phi.cos() * size * if t < 0.5 { 0.5 } else { 1.0 };
            let first = ring(&mut b, sides, 0.0, |_| phi.sin() * size, t, WHITE);
            for i in 0..=sides {
                let v = &mut b.at[(first + i) as usize];
                v[0] += centre.x;
                v[1] = centre.y + y;
                v[2] += centre.z;
            }
            if let Some(p) = prev {
                join_rings(&mut b, p, first, sides);
            }
            prev = Some(first);
        }
    }
    b.fit_ball();
    b.mesh(false)
}

/// **A flat disc with its texture laid flat on it**, in a unit cylinder's
/// room squashed: what a sigil or a mark on the floor is drawn on, so the
/// texture's rings are round and the right way up.
pub fn disc() -> Mesh {
    let mut b = Builder::new();
    let sides = 48u32;
    let centre = b.vert(Vec3::ZERO, [0.5, 0.5], WHITE);
    let first = b.at.len() as u32;
    for i in 0..=sides {
        let a = i as f32 / sides as f32 * TAU;
        b.vert(
            Vec3::new(a.cos() * 0.5, 0.0, a.sin() * 0.5),
            [0.5 + 0.5 * a.cos(), 0.5 + 0.5 * a.sin()],
            WHITE,
        );
    }
    for i in 0..sides {
        b.index
            .extend_from_slice(&[centre, first + i + 1, first + i]);
    }
    b.mesh(false)
}

/// **A sunburst**, in a unit cylinder's room squashed flat: a disc and a
/// crown of rays round it, alternately long and short, reaching the rim.
pub fn sunburst() -> Mesh {
    let mut b = Builder::new();
    let sides = 48u32;
    let centre = b.vert(Vec3::ZERO, [0.5, 0.5], WHITE);
    let first = b.at.len() as u32;
    for i in 0..=sides {
        let a = i as f32 / sides as f32 * TAU;
        let r = 0.3;
        b.vert(
            Vec3::new(a.cos() * r, 0.0, a.sin() * r),
            [0.5 + a.cos() * 0.3, 0.5 + a.sin() * 0.3],
            WHITE,
        );
    }
    for i in 0..sides {
        b.index
            .extend_from_slice(&[centre, first + i + 1, first + i]);
    }
    let rays = 16u32;
    for k in 0..rays {
        let a = k as f32 / rays as f32 * TAU;
        let half = TAU / rays as f32 * 0.32;
        let long = if k % 2 == 0 { 0.5 } else { 0.4 };
        let l = b.vert(
            Vec3::new((a - half).cos() * 0.28, 0.0, (a - half).sin() * 0.28),
            [0.0, 0.0],
            WHITE,
        );
        let r = b.vert(
            Vec3::new((a + half).cos() * 0.28, 0.0, (a + half).sin() * 0.28),
            [1.0, 0.0],
            WHITE,
        );
        let tip = b.vert(
            Vec3::new(a.cos() * long, 0.0, a.sin() * long),
            [0.5, 1.0],
            [1.0, 1.0, 1.0, 0.3],
        );
        b.index.extend_from_slice(&[l, tip, r, l, r, tip]);
    }
    b.mesh(false)
}

/// A texture's `u` round and `v` up a mesh that came without one -- the
/// stones' own lathe (`shapes::rock_column`) -- so a texture can lie on it.
/// What the brush wanted of `UV_0` is only asked of a paint, and the
/// stones are not drawn in one.
pub fn wrap_uv(mesh: &mut Mesh) {
    let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(at)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return;
    };
    let uv: Vec<[f32; 2]> = at
        .iter()
        .map(|p| [(p[2].atan2(p[0]) / TAU + 0.5) * 3.0, (0.5 - p[1]) * 2.0])
        .collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
}

// --- Textures -----------------------------------------------------------------

const SIZE: u32 = 256;

/// Value noise that wraps every `period` cells, so a texture made of it tiles.
fn noise(x: f32, y: f32, period: u32, seed: u32) -> f32 {
    let (xi, yi) = (x.floor(), y.floor());
    let (fx, fy) = (x - xi, y - yi);
    let corner = |i: i32, j: i32| {
        let i = (i.rem_euclid(period as i32)) as u32;
        let j = (j.rem_euclid(period as i32)) as u32;
        hash(i.wrapping_mul(73_856_093) ^ j.wrapping_mul(19_349_663) ^ seed)
    };
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let (u, v) = (s(fx), s(fy));
    let (xi, yi) = (xi as i32, yi as i32);
    let a = corner(xi, yi) + (corner(xi + 1, yi) - corner(xi, yi)) * u;
    let c = corner(xi, yi + 1) + (corner(xi + 1, yi + 1) - corner(xi, yi + 1)) * u;
    a + (c - a) * v
}

/// Layers of `noise`, each finer and fainter; wraps at `period` cells.
fn fbm(x: f32, y: f32, period: u32, seed: u32) -> f32 {
    let mut sum = 0.0;
    let mut amp = 0.5;
    let mut f = 1.0;
    for o in 0..4u32 {
        sum += amp * noise(x * f, y * f, period * f as u32, seed + o * 101);
        amp *= 0.5;
        f *= 2.0;
    }
    sum
}

fn image(pixels: Vec<u8>) -> Image {
    let mut img = Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    });
    img
}

fn paint(f: impl Fn(f32, f32) -> [f32; 4]) -> Image {
    let mut px = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let c = f(x as f32 / SIZE as f32, y as f32 / SIZE as f32);
            for k in c {
                px.push((k.clamp(0.0, 1.0) * 255.0).round() as u8);
            }
        }
    }
    image(px)
}

/// **Flame**: tongues -- streaks stretched along `v`, yellow where they
/// are thick and orange where they thin, with gaps between them the alpha
/// lets the fight through. No gradient root to tip: the mesh carries that
/// (`heat`), so this can flow up through it.
pub fn flame_texture() -> Image {
    paint(|u, v| {
        let streak = fbm(u * 10.0, v * 2.0, 10, 11);
        let lick = fbm(u * 4.0 + 0.3 * streak, v * 4.0, 4, 7);
        let body = ((streak * 0.6 + lick * 0.6) * 1.9 - 0.55).clamp(0.0, 1.0);
        [
            1.0,
            0.55 + 0.45 * body,
            0.25 + 0.6 * body * body,
            (0.25 + body * 1.2).clamp(0.0, 1.0),
        ]
    })
}

/// **Wind**: pale streaks along the texture's `v`, broken and of uneven
/// brightness, mostly clear between.
pub fn wind_texture() -> Image {
    paint(|u, v| {
        let streak = fbm(u * 3.0, v * 12.0, 12, 21);
        let gust = fbm(u * 1.5, v * 3.0, 3, 23);
        let a = ((streak - 0.42) * 3.0).clamp(0.0, 1.0) * (0.4 + 0.6 * gust);
        let edge = (1.0 - (2.0 * u - 1.0).abs()).powf(0.5);
        [0.92, 0.95, 1.0, a * edge]
    })
}

/// **Stone**: mottled grey with warmer and cooler patches, and dark cracks.
pub fn stone_texture() -> Image {
    paint(|u, v| {
        let m = fbm(u * 6.0, v * 6.0, 6, 31);
        let fine = fbm(u * 24.0, v * 24.0, 24, 33);
        let crack = (1.0 - ((fbm(u * 3.0, v * 3.0, 3, 35) - 0.5).abs() * 40.0)).clamp(0.0, 1.0);
        let g = 0.45 + 0.35 * m + 0.12 * fine - 0.35 * crack;
        let warm = fbm(u * 3.0, v * 3.0, 3, 37) - 0.5;
        [g + 0.05 * warm, g, g - 0.04 * warm, 1.0]
    })
}

/// **A lit stone's glow**: only its cracks, burning orange.
pub fn ember_texture() -> Image {
    paint(|u, v| {
        let crack = (1.0 - ((fbm(u * 3.0, v * 3.0, 3, 35) - 0.5).abs() * 40.0)).clamp(0.0, 1.0);
        let pulse = 0.6 + 0.4 * fbm(u * 10.0, v * 10.0, 10, 39);
        let c = crack * pulse;
        [c, c * 0.42, c * 0.08, 1.0]
    })
}

/// Domain-warped `fbm`: marbling, the way two liquids fold into each other.
fn marble(u: f32, v: f32, period: u32, seed: u32) -> f32 {
    let wu = fbm(u * period as f32, v * period as f32, period, seed);
    let wv = fbm(
        u * period as f32 + 5.2,
        v * period as f32 + 1.3,
        period,
        seed + 3,
    );
    fbm(
        u * period as f32 + 2.0 * wu * period as f32 / 4.0,
        v * period as f32 + 2.0 * wv * period as f32 / 4.0,
        period,
        seed + 7,
    )
}

/// **Essence**: crimson gone to violet, folded through itself, brighter in
/// its folds -- something drawn out of a body, not what a wound leaves.
pub fn essence_texture() -> Image {
    paint(|u, v| {
        let m = marble(u, v, 4, 41);
        let fold = (1.0 - ((m - 0.5).abs() * 6.0)).clamp(0.0, 1.0);
        [
            0.55 + 0.45 * fold,
            0.15 + 0.35 * fold * fold,
            0.45 + 0.45 * fold,
            0.55 + 0.45 * (m + fold * 0.5).min(1.0),
        ]
    })
}

/// **Blood**: dark and wet, clotting darker in patches, with a sheen where
/// it is thin.
pub fn blood_texture() -> Image {
    paint(|u, v| {
        let m = marble(u, v, 4, 51);
        let clot = fbm(u * 8.0, v * 8.0, 8, 53);
        let g = 0.55 + 0.45 * m - 0.35 * (clot - 0.5).max(0.0) * 2.0;
        [g, g * 0.55, g * 0.6, 1.0]
    })
}

/// **Bark of a black vine**: fibres along `v`, splits between them.
pub fn vine_texture() -> Image {
    paint(|u, v| {
        let fibre = fbm(u * 24.0, v * 3.0, 24, 61);
        let split = (1.0 - ((fbm(u * 12.0, v * 2.0, 12, 63) - 0.5).abs() * 14.0)).clamp(0.0, 1.0);
        let g = 0.55 + 0.45 * fibre - 0.5 * split;
        [g, g * 0.85, g * 0.9, 1.0]
    })
}

/// **A heart's veins**: a branching dark lacing over crimson flesh, and the
/// veins alone, for what burns.
pub fn heart_texture() -> Image {
    paint(|u, v| {
        let vein = (1.0 - ((marble(u, v, 3, 71) - 0.5).abs() * 12.0)).clamp(0.0, 1.0);
        let flesh = 0.7 + 0.3 * fbm(u * 10.0, v * 10.0, 10, 73);
        let g = flesh * (1.0 - 0.55 * vein);
        [g, g * (0.6 + 0.4 * vein), g * (0.6 + 0.5 * vein), 1.0]
    })
}

/// **Chased gold**: hammered, the dents catching the light unevenly, with a
/// running scroll of engraving -- what makes a gold weapon read as worked
/// metal rather than as yellow plastic.
pub fn gilt_texture() -> Image {
    paint(|u, v| {
        let dent = fbm(u * 20.0, v * 20.0, 20, 81);
        let scroll = ((u * TAU * 4.0 + (v * TAU * 8.0).sin() * 1.2).sin()).abs();
        let groove = (1.0 - scroll * 9.0).clamp(0.0, 1.0) * 0.5;
        let g = 0.78 + 0.3 * dent - groove;
        [g, g * 0.97, g * 0.92, 1.0]
    })
}

/// **Light**: bright filaments drawn along `v` through a paler glow, the way
/// a flare is brighter in its rays than between them.
pub fn light_texture() -> Image {
    paint(|u, v| {
        let ray = fbm(u * 16.0, v * 2.0, 16, 91);
        let glow = fbm(u * 4.0, v * 4.0, 4, 93);
        let bright = ((ray - 0.35) * 2.5).clamp(0.0, 1.0);
        let g = 0.75 + 0.25 * bright;
        [g, g, g, (0.45 + 0.4 * bright + 0.2 * glow).min(1.0)]
    })
}

/// **The dark**: a void folding on itself, with thin violet filaments in
/// its folds -- light being drunk.
pub fn dark_texture() -> Image {
    paint(|u, v| {
        let m = marble(u, v, 3, 101);
        let thread = (1.0 - ((m - 0.5).abs() * 14.0)).clamp(0.0, 1.0);
        let g = 0.35 + 0.65 * thread;
        [g, g, g, 0.75 + 0.25 * thread]
    })
}

/// **Smoke**: billows, soft at their edges, darker in their hollows.
pub fn smoke_texture() -> Image {
    paint(|u, v| {
        let b = fbm(u * 5.0, v * 5.0, 5, 111);
        let fine = fbm(u * 15.0, v * 15.0, 15, 113);
        let g = 0.6 + 0.4 * b;
        [
            g,
            g,
            g,
            (0.4 + 0.8 * (b * 0.7 + fine * 0.3)).clamp(0.0, 1.0),
        ]
    })
}

/// **A sigil**: rings round the middle, runes standing between two of
/// them, spokes, a star at the heart -- the texture of a disc laid flat
/// (`disc`), `u` and `v` across it.
pub fn sigil_texture() -> Image {
    paint(|u, v| {
        let (x, y) = (u * 2.0 - 1.0, v * 2.0 - 1.0);
        let r = (x * x + y * y).sqrt();
        let a = y.atan2(x);
        let line = |at: f32, w: f32| (1.0 - ((r - at).abs() / w)).clamp(0.0, 1.0);
        let rings = line(0.96, 0.025) + line(0.86, 0.015) + line(0.6, 0.02) + line(0.52, 0.012);
        // Runes: short strokes between the two outer rings, a glyph every
        // twelfth of the way round, each its own.
        let cell = (a / TAU * 24.0).rem_euclid(24.0);
        let k = cell.floor() as u32;
        let f = cell.fract();
        let runes = if r > 0.88 || r < 0.62 {
            0.0
        } else {
            let h = (r - 0.62) / 0.26;
            let bar = |p: f32| (1.0 - ((f - p).abs() / 0.06)).clamp(0.0, 1.0);
            let s1 = bar(0.3 + 0.4 * hash(k * 3)) * if h > 0.15 && h < 0.85 { 1.0 } else { 0.0 };
            let s2 = (1.0 - ((h - (0.2 + 0.6 * hash(k * 5))).abs() / 0.08)).clamp(0.0, 1.0)
                * if f > 0.2 && f < 0.8 { 1.0 } else { 0.0 };
            s1.max(s2)
        };
        let spokes = if r < 0.52 {
            (1.0 - ((a * 4.0).sin().abs() / 0.08)).clamp(0.0, 1.0) * (r / 0.52)
        } else {
            0.0
        };
        let star = (1.0 - r / 0.25).clamp(0.0, 1.0);
        let ink = (rings + runes + spokes + star).min(1.0);
        let fill = if r < 1.0 { 0.12 } else { 0.0 };
        [1.0, 1.0, 1.0, ink.max(fill)]
    })
}

/// The materials whose textures flow, and how: `(handle, units a second
/// along u, along v)`.
#[derive(Resource, Default)]
pub struct Flowing(pub Vec<(Handle<StandardMaterial>, Vec2)>);

/// Scroll every flowing material's texture on.
pub fn flow(
    time: Res<Time>,
    flowing: Res<Flowing>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let t = time.elapsed_secs_wrapped();
    for (handle, speed) in &flowing.0 {
        if let Some(m) = materials.get_mut(handle) {
            m.uv_transform.translation = Vec2::new((speed.x * t).fract(), (speed.y * t).fract());
        }
    }
}
