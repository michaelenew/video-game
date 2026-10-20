//! **Her blades as blades** (`docs/design/exploration/0024_a_character_done_properly.md`):
//! the meshes every blade of shadow the Reaver throws is drawn with, in place
//! of the flat discs and balls they were.
//!
//! - **A shuriken**, for everything she or her shadow throws or spins -- the
//!   Shuriken and the shadow's, the Fan, the Tumble, the Whirl and the Rising
//!   whirl, the burst when the shadow comes home, the Guillotine's lotus. Four
//!   swept points round a raised hub, thick at the middle and keen at the
//!   tips, with a faint ring round it where the points blur as it spins.
//! - **A shard**, for what comes up out of the floor: the Plunge's ring of
//!   shadow. A faceted spike, wider than it is deep.
//! - **A crescent**, for a cut left hanging in the air: the Reaping's.
//!
//! Each is a unit shape made once, scaled to the hit test's size wherever it
//! is drawn (`effect_piece`), so a blade is drawn as big as it cuts. Colours
//! ride on the vertices -- dark steel at the body, her violet at the edges --
//! so one material each serves every blade.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

/// Dark steel with violet in it, and her violet at the keen edges.
const STEEL: [f32; 4] = [0.10, 0.07, 0.16, 1.0];
const EDGE: [f32; 4] = [0.75, 0.45, 1.0, 1.0];
/// The ring a spinning blade's points blur into: her violet, faint.
const BLUR: [f32; 4] = [0.55, 0.30, 1.0, 0.22];

struct Builder {
    at: Vec<[f32; 3]>,
    normal: Vec<[f32; 3]>,
    colour: Vec<[f32; 4]>,
    index: Vec<u32>,
}

impl Builder {
    fn new() -> Builder {
        Builder {
            at: Vec::new(),
            normal: Vec::new(),
            colour: Vec::new(),
            index: Vec::new(),
        }
    }

    /// One flat-shaded triangle, its normal from its winding.
    fn tri(&mut self, a: Vec3, b: Vec3, c: Vec3, ca: [f32; 4], cb: [f32; 4], cc: [f32; 4]) {
        let n = (b - a).cross(c - a).normalize_or(Vec3::Y);
        let i = self.at.len() as u32;
        for (p, col) in [(a, ca), (b, cb), (c, cc)] {
            self.at.push(p.to_array());
            self.normal.push(n.to_array());
            self.colour.push(col);
        }
        self.index.extend_from_slice(&[i, i + 1, i + 2]);
    }

    fn mesh(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.at)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normal)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colour)
        .with_inserted_indices(Indices::U32(self.index))
    }
}

/// A four-pointed throwing star, lying in the `xz` plane, half a unit to its
/// tips: each point swept back like a pinwheel's, a ridge down its middle,
/// and a faint ring round the whole where a spinning blade blurs.
pub fn shuriken() -> Mesh {
    let mut b = Builder::new();
    let points = 4;
    let hub = 0.11;
    let tip = 0.5;
    let thick = 0.05;
    // Each point is drawn as a little diamond-section blade from the hub to
    // its tip: a ridge on top and below, keen at both sides.
    for k in 0..points {
        let a = k as f32 / points as f32 * std::f32::consts::TAU;
        let along = Vec3::new(a.cos(), 0.0, a.sin());
        let across = Vec3::new(-a.sin(), 0.0, a.cos());
        // Swept: the tip turned a little back against the spin.
        let point = (along * 0.9 - across * 0.32).normalize() * tip;
        let root_l = along * hub + across * 0.13;
        let root_r = along * hub - across * 0.13;
        let up = Vec3::Y * thick * 0.5;
        let ridge = along * (hub + 0.04);
        for side in [1.0f32, -1.0] {
            let crest = ridge + up * side;
            let (l, r) = if side > 0.0 {
                (root_l, root_r)
            } else {
                (root_r, root_l)
            };
            b.tri(crest, l, point, STEEL, EDGE, EDGE);
            b.tri(crest, point, r, STEEL, EDGE, EDGE);
        }
    }
    // The hub: a raised boss, top and bottom.
    let rim = 12;
    for side in [1.0f32, -1.0] {
        let top = Vec3::Y * thick * 0.7 * side;
        for i in 0..rim {
            let a0 = i as f32 / rim as f32 * std::f32::consts::TAU;
            let a1 = (i + 1) as f32 / rim as f32 * std::f32::consts::TAU;
            let p0 = Vec3::new(a0.cos(), 0.0, a0.sin()) * hub * 1.25;
            let p1 = Vec3::new(a1.cos(), 0.0, a1.sin()) * hub * 1.25;
            if side > 0.0 {
                b.tri(top, p1, p0, EDGE, STEEL, STEEL);
            } else {
                b.tri(top, p0, p1, EDGE, STEEL, STEEL);
            }
        }
    }
    // The blur: a thin band at the tips' radius, both faces, fading inward.
    let ring = 32;
    for i in 0..ring {
        let a0 = i as f32 / ring as f32 * std::f32::consts::TAU;
        let a1 = (i + 1) as f32 / ring as f32 * std::f32::consts::TAU;
        let d0 = Vec3::new(a0.cos(), 0.0, a0.sin());
        let d1 = Vec3::new(a1.cos(), 0.0, a1.sin());
        let (inner, outer) = (0.3, 0.5);
        let faint = [BLUR[0], BLUR[1], BLUR[2], 0.0];
        for flip in [false, true] {
            let (a, b2, c) = (d0 * inner, d1 * inner, d1 * outer);
            let d = d0 * outer;
            if flip {
                b.tri(a, c, b2, faint, BLUR, faint);
                b.tri(a, d, c, faint, BLUR, BLUR);
            } else {
                b.tri(a, b2, c, faint, faint, BLUR);
                b.tri(a, c, d, faint, BLUR, BLUR);
            }
        }
    }
    b.mesh()
}

/// A faceted spike standing on the floor: unit tall, half a unit across at
/// its widest, a quarter deep, its point a little off its middle.
pub fn shard() -> Mesh {
    let mut b = Builder::new();
    let base = [
        Vec3::new(0.5, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.25),
        Vec3::new(-0.5, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -0.25),
    ];
    let waist = base.map(|p| p * 0.8 + Vec3::Y * 0.3);
    let point = Vec3::new(0.06, 1.0, 0.0);
    for i in 0..4 {
        let j = (i + 1) % 4;
        b.tri(base[i], waist[j], waist[i], STEEL, STEEL, STEEL);
        b.tri(base[i], base[j], waist[j], STEEL, STEEL, STEEL);
        b.tri(waist[i], waist[j], point, STEEL, STEEL, EDGE);
    }
    b.mesh()
}

/// A crescent cut hanging in the air: a curved sliver in the `xy` plane, a
/// unit tall and keen at both ends, widest at its middle.
pub fn crescent() -> Mesh {
    let mut b = Builder::new();
    let steps = 16;
    let curve = |t: f32| {
        let a = (t - 0.5) * 2.2;
        Vec3::new(a.sin() * 0.5 - 0.25, a.cos() * 0.0 + (t - 0.5), 0.0) + Vec3::X * 0.0
    };
    let width = |t: f32| (std::f32::consts::PI * t).sin() * 0.12;
    for i in 0..steps {
        let (t0, t1) = (i as f32 / steps as f32, (i + 1) as f32 / steps as f32);
        let (c0, c1) = (curve(t0), curve(t1));
        let (w0, w1) = (width(t0), width(t1));
        let (o0, o1) = (c0 + Vec3::X * w0, c1 + Vec3::X * w1);
        let (z0, z1) = (Vec3::Z * w0 * 0.3, Vec3::Z * w1 * 0.3);
        for side in [1.0f32, -1.0] {
            let (m0, m1) = (c0 + z0 * side, c1 + z1 * side);
            if side > 0.0 {
                b.tri(m0, o0, o1, STEEL, EDGE, EDGE);
                b.tri(m0, o1, m1, STEEL, EDGE, STEEL);
            } else {
                b.tri(m0, o1, o0, STEEL, EDGE, EDGE);
                b.tri(m0, m1, o1, STEEL, STEEL, EDGE);
            }
        }
    }
    b.mesh()
}

/// What the blades are made of: dark steel lit at the edges by her violet,
/// which the vertex colours carry, and a glow of its own so a blade crossing
/// the floor in shade still reads.
pub fn steel() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        emissive: LinearRgba::rgb(0.10, 0.04, 0.24),
        metallic: 0.6,
        perceptual_roughness: 0.35,
        reflectance: 0.4,
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        cull_mode: None,
        ..default()
    }
}
