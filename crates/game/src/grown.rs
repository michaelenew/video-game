//! **A creature's body, grown over its skeleton** (2026-10-23,
//! exploration 0025 §14).
//!
//! A creature is drawn from its species table: a box per part, placed by the
//! simulation's own rig (`beast`). That is what lets any species -- a bred one
//! too -- be drawn at all, and it is also why every creature read as a stack
//! of boxes beside characters with sculpted bodies. This grows one body from
//! the same table instead, so it keeps working for anything the table can
//! describe:
//!
//! 1. **The shape** is the parts in their rest pose, each a rounded box, joined
//!    by a smooth union -- the way clay pressed together joins -- so a neck
//!    flows into a shoulder and a thigh into a hip. The field that describes it
//!    is turned into a mesh by *surface nets*: a grid, a vertex in each cell the
//!    surface passes through, a quad across each grid edge it crosses.
//! 2. **The skin** binds each vertex to the bones of the parts nearest it,
//!    softly, so a bending joint stretches the hide between two bones rather
//!    than opening a wedge. The rig the simulation poses drives it.
//! 3. **The hide** is painted by rule, as vertex colours: each part's paint
//!    from the species' [`Look`], darker along the back and paler underneath
//!    (countershading, which nearly every large animal has), and a slow
//!    mottle so a flank is not one flat colour. A part that breaks changes
//!    colour where it is.
//!
//! What the simulation collides and hits against is unchanged: the boxes are
//! still the truth, and the debug overlay still draws them. The weak points
//! are left out of the body and drawn as their own glowing pads on it, so
//! they say what they are from across the arena as they always did.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use sim::species::Species;

use crate::species::{Look, Paint};

/// A body grown for one species: the mesh, which parts colour each vertex,
/// and each bone's rest frame (for the skin's inverse binds).
pub struct Grown {
    pub mesh: Mesh,
    /// Per vertex, up to three parts and how much each colours it.
    pub paints: Vec<[(u16, f32); 3]>,
    /// Per vertex, where it is on the hide: how much it faces up, how deep in
    /// a band, its ridge or crease, its mottle (`look::hide::Hide::at`).
    pub shade: Vec<[f32; 5]>,
    /// Every bone's rest frame, in the order of the species' bones.
    pub rest: Vec<Mat4>,
}

/// Whether creatures are drawn as boxes, as they were before bodies were
/// grown (`--box-creatures`): to compare, and to fall back.
pub fn off() -> bool {
    crate::platform::flag("--box-creatures")
}

/// Which species are drawn with a grown body: the Ridgeback first (the owner,
/// 2026-10-23: "start with the Ridgeback").
pub fn grows(id: sim::species::SpeciesId) -> bool {
    !off() && id == sim::species::SpeciesId::RIDGEBACK
}

/// Is this part drawn as the grown body rather than as its own box?
pub fn covers(sp: &Species, index: usize) -> bool {
    grows(sp.id) && in_body(sp, index)
}

/// Which parts make the body: everything with a body of its own, but the
/// weak points, which are drawn as themselves on it, and a hollow part, which
/// is a room, not flesh.
fn in_body(sp: &Species, index: usize) -> bool {
    let p = &sp.parts[index];
    !p.weak && !p.shape.hollow
}

/// A part's box in the rest pose: its bone's frame, the box's middle and
/// half-size in that frame, and how round its edges are.
#[derive(Clone, Copy)]
struct Lump {
    to_local: Mat4,
    middle: Vec3,
    half: Vec3,
    round: f32,
    /// Its long axis (0, 1 or 2), which end of it is the joint's (+1 or -1),
    /// and how much it narrows toward the other end: none on a part not much
    /// longer than it is wide.
    long: usize,
    root: f32,
    taper: f32,
    /// Its bounds in the world, grown by the blend's reach.
    lo: Vec3,
    hi: Vec3,
    bone: usize,
}

impl Lump {
    /// Signed distance to the rounded box: negative inside.
    fn distance(&self, p: Vec3) -> f32 {
        let mut q = self.to_local.transform_point3(p) - self.middle;
        // Narrower toward the end away from its joint: across the long axis,
        // space is stretched there, so the same box is thinner.
        if self.taper > 0.0 {
            let t = (q[self.long] / self.half[self.long] * -self.root).clamp(-1.0, 1.0) * 0.5 + 0.5;
            let f = 1.0 - self.taper * t;
            for a in 0..3 {
                if a != self.long {
                    q[a] /= f;
                }
            }
        }
        let d = q.abs() - (self.half - Vec3::splat(self.round));
        d.max(Vec3::ZERO).length() + d.max_element().min(0.0) - self.round
    }

    fn near(&self, p: Vec3) -> bool {
        p.cmpge(self.lo).all() && p.cmple(self.hi).all()
    }
}

fn fx3(v: sim::V3) -> Vec3 {
    Vec3::new(
        v.x.to_f32_for_render(),
        v.y.to_f32_for_render(),
        v.z.to_f32_for_render(),
    )
}

/// A bone's placed frame as a matrix.
pub fn frame(p: &sim::beast::Placed) -> Mat4 {
    let at = fx3(p.at);
    let x = fx3(p.local_to_world(sim::V3::new(sim::Fx::ONE, sim::Fx::ZERO, sim::Fx::ZERO))) - at;
    let y = fx3(p.local_to_world(sim::V3::new(sim::Fx::ZERO, sim::Fx::ONE, sim::Fx::ZERO))) - at;
    let x = x.normalize_or(Vec3::X);
    let y = (y - x * x.dot(y)).normalize_or(Vec3::Y);
    Mat4::from_cols(
        x.extend(0.0),
        y.extend(0.0),
        x.cross(y).extend(0.0),
        at.extend(1.0),
    )
}

/// The smooth union of two distances (polynomial smooth minimum): `k` is
/// how far the join reaches.
fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = (0.5 + 0.5 * (b - a) / k).clamp(0.0, 1.0);
    b + (a - b) * h - k * h * (1.0 - h)
}

/// How many cells across the body's longest side the grid has: the detail
/// the surface can hold. About eight centimetres on the Ridgeback.
const CELLS: f32 = 200.0;

/// How far the smooth union reaches, as a share of a typical part's
/// thickness: a joint is filled about this far up each side of it.
const BLEND: f32 = 0.55;

/// How round a part's edges are, as a share of its thinnest side.
const ROUND: f32 = 0.85;

/// How much a long part narrows toward its far end: a limb toward its foot,
/// a tail toward its tip, a head toward its muzzle.
const TAPER: f32 = 0.42;

/// How many bands cross the back from nose to tail.
const BANDS: f32 = 9.0;

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Grow `sp`'s body. `None` for a species with no part that makes one.
pub fn grow(sp: &'static Species) -> Option<Grown> {
    let rig = sim::beast::Rig::build(
        sp,
        sim::V3::ZERO,
        sim::Fx::ZERO,
        &sim::beast::Pose::rest(sp.bones.len()),
    );
    let rest: Vec<Mat4> = (0..sp.bones.len()).map(|b| frame(&rig.bone[b])).collect();
    let indices: Vec<usize> = (0..sp.parts.len()).filter(|&i| in_body(sp, i)).collect();
    if indices.is_empty() {
        return None;
    }
    let mut thick: Vec<f32> = indices
        .iter()
        .map(|&i| {
            let s = sp.shape(i);
            fx3(s.max.sub(s.min)).min_element()
        })
        .collect();
    thick.sort_by(f32::total_cmp);
    let blend = thick[thick.len() / 2] * BLEND;

    let mut lumps: Vec<(usize, Lump)> = Vec::new();
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for &i in &indices {
        let s = sp.shape(i);
        let (a, b) = (fx3(s.min), fx3(s.max));
        let half = ((b - a) * 0.5).max(Vec3::splat(0.005));
        let middle = (a + b) * 0.5;
        let to_world = rest[s.bone];
        let mut plo = Vec3::splat(f32::MAX);
        let mut phi = Vec3::splat(f32::MIN);
        for c in 0..8 {
            let corner = middle
                + half
                    * Vec3::new(
                        if c & 1 == 0 { -1.0 } else { 1.0 },
                        if c & 2 == 0 { -1.0 } else { 1.0 },
                        if c & 4 == 0 { -1.0 } else { 1.0 },
                    );
            let w = to_world.transform_point3(corner);
            plo = plo.min(w);
            phi = phi.max(w);
        }
        let reach = Vec3::splat(blend * 1.5);
        lo = lo.min(plo);
        hi = hi.max(phi);
        // The long axis, and which end of it the bone's joint is nearer.
        let long = if half.x >= half.y && half.x >= half.z {
            0
        } else if half.y >= half.z {
            1
        } else {
            2
        };
        let across = (half[(long + 1) % 3] + half[(long + 2) % 3]) * 0.5;
        let taper = TAPER * ((half[long] / across - 1.4) / 1.6).clamp(0.0, 1.0);
        let root = if middle[long] > 0.0 { -1.0 } else { 1.0 };
        lumps.push((
            i,
            Lump {
                to_local: to_world.inverse(),
                middle,
                half,
                long,
                root,
                taper,
                round: half.min_element() * 2.0 * ROUND * 0.5,
                lo: plo - reach,
                hi: phi + reach,
                bone: s.bone,
            },
        ));
    }

    // The field, sampled on a grid round the whole body.
    let margin = Vec3::splat(blend * 2.0);
    let (lo, hi) = (lo - margin, hi + margin);
    let cell = (hi - lo).max_element() / CELLS;
    let n = ((hi - lo) / cell).ceil().as_uvec3() + UVec3::ONE;
    let (nx, ny, nz) = (n.x as usize, n.y as usize, n.z as usize);
    let at = |i: usize, j: usize, k: usize| lo + Vec3::new(i as f32, j as f32, k as f32) * cell;
    let field = |p: Vec3| {
        let mut d = f32::MAX;
        for (_, l) in &lumps {
            if l.near(p) {
                let e = l.distance(p);
                d = if d == f32::MAX { e } else { smin(d, e, blend) };
            }
        }
        if d == f32::MAX { cell * 4.0 } else { d }
    };
    let mut f = vec![0.0f32; nx * ny * nz];
    let id = |i: usize, j: usize, k: usize| (k * ny + j) * nx + i;
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                f[id(i, j, k)] = field(at(i, j, k));
            }
        }
    }

    // Surface nets: a vertex in each cell the surface crosses, at the mean of
    // where it crosses the cell's edges.
    const EDGES: [(usize, usize); 12] = [
        (0, 1),
        (2, 3),
        (4, 5),
        (6, 7),
        (0, 2),
        (1, 3),
        (4, 6),
        (5, 7),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];
    let mut vertex_of: HashMap<usize, u32> = HashMap::new();
    let mut positions: Vec<Vec3> = Vec::new();
    for k in 0..nz - 1 {
        for j in 0..ny - 1 {
            for i in 0..nx - 1 {
                let corner = |c: usize| (i + (c & 1), j + ((c >> 1) & 1), k + ((c >> 2) & 1));
                let v: [f32; 8] = std::array::from_fn(|c| {
                    let (a, b, cc) = corner(c);
                    f[id(a, b, cc)]
                });
                let inside = v.iter().filter(|x| **x < 0.0).count();
                if inside == 0 || inside == 8 {
                    continue;
                }
                let mut sum = Vec3::ZERO;
                let mut count = 0.0;
                for (a, b) in EDGES {
                    if (v[a] < 0.0) != (v[b] < 0.0) {
                        let t = v[a] / (v[a] - v[b]);
                        let (ai, aj, ak) = corner(a);
                        let (bi, bj, bk) = corner(b);
                        sum += at(ai, aj, ak).lerp(at(bi, bj, bk), t);
                        count += 1.0;
                    }
                }
                vertex_of.insert(id(i, j, k), positions.len() as u32);
                positions.push(sum / count);
            }
        }
    }
    // A quad across every grid edge the surface crosses, joining the four
    // cells round it, wound so it faces out of the body.
    let mut tris: Vec<u32> = Vec::new();
    for k in 1..nz - 1 {
        for j in 1..ny - 1 {
            for i in 1..nx - 1 {
                let here = f[id(i, j, k)] < 0.0;
                for axis in 0..3 {
                    let (ni, nj, nk) = match axis {
                        0 => (i + 1, j, k),
                        1 => (i, j + 1, k),
                        _ => (i, j, k + 1),
                    };
                    if ni >= nx || nj >= ny || nk >= nz {
                        continue;
                    }
                    if here == (f[id(ni, nj, nk)] < 0.0) {
                        continue;
                    }
                    // The four cells sharing this edge.
                    let cells = match axis {
                        0 => [
                            id(i, j - 1, k - 1),
                            id(i, j, k - 1),
                            id(i, j, k),
                            id(i, j - 1, k),
                        ],
                        1 => [
                            id(i - 1, j, k - 1),
                            id(i - 1, j, k),
                            id(i, j, k),
                            id(i, j, k - 1),
                        ],
                        _ => [
                            id(i - 1, j - 1, k),
                            id(i, j - 1, k),
                            id(i, j, k),
                            id(i - 1, j, k),
                        ],
                    };
                    let Some(q) = cells
                        .iter()
                        .map(|c| vertex_of.get(c).copied())
                        .collect::<Option<Vec<u32>>>()
                    else {
                        continue;
                    };
                    let (a, b, c, d) = (q[0], q[1], q[2], q[3]);
                    if here {
                        tris.extend([a, b, c, a, c, d]);
                    } else {
                        tris.extend([a, c, b, a, d, c]);
                    }
                }
            }
        }
    }

    // A little smoothing, which the grid's steps want, then every vertex
    // pulled back onto the surface and given the field's own normal.
    let mut links: Vec<Vec<u32>> = vec![Vec::new(); positions.len()];
    for t in tris.chunks_exact(3) {
        for e in 0..3 {
            links[t[e] as usize].push(t[(e + 1) % 3]);
        }
    }
    for _ in 0..2 {
        let before = positions.clone();
        for (v, near) in links.iter().enumerate() {
            if near.is_empty() {
                continue;
            }
            let mean = near.iter().map(|&u| before[u as usize]).sum::<Vec3>() / near.len() as f32;
            positions[v] = before[v].lerp(mean, 0.5);
        }
    }
    let h = cell * 0.5;
    let gradient = |p: Vec3| {
        Vec3::new(
            field(p + Vec3::X * h) - field(p - Vec3::X * h),
            field(p + Vec3::Y * h) - field(p - Vec3::Y * h),
            field(p + Vec3::Z * h) - field(p - Vec3::Z * h),
        )
        .normalize_or(Vec3::Y)
    };
    let mut normals = Vec::with_capacity(positions.len());
    for p in positions.iter_mut() {
        for _ in 0..2 {
            let g = gradient(*p);
            *p -= g * field(*p);
        }
        normals.push(gradient(*p));
    }

    // The skin and the paint: each vertex bound to the bones of the parts
    // nearest it, softly, and coloured by the parts nearest it, more sharply.
    let mut joints = Vec::with_capacity(positions.len());
    let mut weights = Vec::with_capacity(positions.len());
    let mut paints = Vec::with_capacity(positions.len());
    let mut shade = Vec::with_capacity(positions.len());
    let span = (hi - lo).max_element();
    for (p, nrm) in positions.iter().zip(&normals) {
        let mut near: Vec<(usize, usize, f32)> = lumps
            .iter()
            .filter(|(_, l)| l.near(*p))
            .map(|(i, l)| (*i, l.bone, l.distance(*p)))
            .collect();
        if near.is_empty() {
            near = lumps
                .iter()
                .map(|(i, l)| (*i, l.bone, l.distance(*p)))
                .collect();
        }
        near.sort_by(|a, b| a.2.total_cmp(&b.2));
        let best = near[0].2;
        let mut by_bone: Vec<(usize, f32)> = Vec::new();
        for &(_, bone, d) in &near {
            let w = (-(d - best) / (blend * 0.6)).exp();
            match by_bone.iter_mut().find(|(b, _)| *b == bone) {
                Some(e) => e.1 = e.1.max(w),
                None => by_bone.push((bone, w)),
            }
        }
        by_bone.sort_by(|a, b| b.1.total_cmp(&a.1));
        by_bone.truncate(4);
        let total: f32 = by_bone.iter().map(|(_, w)| w).sum();
        let mut j = [0u16; 4];
        let mut w = [0.0f32; 4];
        for (k, (b, x)) in by_bone.iter().enumerate() {
            j[k] = *b as u16;
            w[k] = x / total;
        }
        joints.push(j);
        weights.push(w);

        let mut pick = [(0u16, 0.0f32); 3];
        let mut sum = 0.0;
        for (k, &(i, _, d)) in near.iter().take(3).enumerate() {
            let x = (-(d - best) / (blend * 0.2)).exp();
            pick[k] = (i as u16, x);
            sum += x;
        }
        for e in pick.iter_mut() {
            e.1 /= sum;
        }
        paints.push(pick);

        // Where it is on the hide: how much it faces the sky, whether it
        // is in one of the bands crossing the back (wavering, not ruled),
        // and a slow mottle. Its ridge or crease is worked out below.
        let up = nrm.y * 0.5 + 0.5;
        let band =
            (p.x * std::f32::consts::TAU * BANDS / span + 1.3 * noise(*p * (3.0 / span))).sin();
        let mottle = noise(*p * (7.0 / span));
        shade.push([
            up,
            smoothstep(0.2, 0.7, band),
            0.0,
            mottle,
            scute(*p * (SCUTES / span)),
        ]);
    }

    // The light on its edges and the dark in its creases, from how the
    // surface bends round each vertex -- the painter's light on a ridge and
    // shadow in a fold, which the characters' painted textures carry too.
    for (v, near) in links.iter().enumerate() {
        if near.is_empty() {
            continue;
        }
        let n = normals[v];
        let bend = near
            .iter()
            .map(|&u| n.dot((positions[u as usize] - positions[v]).normalize_or_zero()))
            .sum::<f32>()
            / near.len() as f32;
        // Negative bends away (a ridge), positive bends in (a crease).
        shade[v][2] = (bend / 0.35).clamp(-1.0, 1.0);
    }
    let colours = vec![[1.0f32; 4]; positions.len()];
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        positions.iter().map(|p| p.to_array()).collect::<Vec<_>>(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        normals.iter().map(|n| n.to_array()).collect::<Vec<_>>(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0f32; 2]; positions.len()])
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        VertexAttributeValues::Uint16x4(joints),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, weights)
    .with_inserted_indices(Indices::U32(tris));
    Some(Grown {
        mesh,
        paints,
        shade,
        rest,
    })
}

/// The hide's colours for one body, linear: each vertex the hides of the
/// parts under it (`look::hide`, from the colour `paint` says each part
/// wears now), at its place on the hide.
pub fn colours(grown: &Grown, paint: impl Fn(usize) -> Paint) -> Vec<[f32; 4]> {
    let mut cache: HashMap<u16, look::hide::Hide> = HashMap::new();
    grown
        .paints
        .iter()
        .zip(&grown.shade)
        .map(|(pick, s)| {
            let mut c = Vec3::ZERO;
            for &(i, w) in pick {
                if w <= 0.0 {
                    continue;
                }
                let hide = *cache
                    .entry(i)
                    .or_insert_with(|| look::hide::Hide::of(paint(i as usize).rgb));
                let [r, g, b] = hide.at(s[0], s[1], s[2], s[3], s[4]);
                // The palette is written as the eye sees it (sRGB); a vertex
                // colour is linear.
                c += Vec3::from(LinearRgba::from(Color::srgb(r, g, b)).to_f32_array_no_alpha()) * w;
            }
            c.extend(1.0).to_array()
        })
        .collect()
}

/// The paint a part wears, for a body's hide: its look's paint for what it
/// is, and broken where it is.
pub fn paint_of(sp: &Species, look: &Look, broken: bool, index: usize) -> Paint {
    if broken {
        look.broken
    } else if sp.parts[index].shape.breakable {
        look.breakable
    } else {
        look.armour
    }
}

/// How many armour plates (scutes) span the body's length.
const SCUTES: f32 = 30.0;

/// How near the seam between two scutes a point is: 0 on a seam, 1 at a
/// plate's middle. Cells of a jittered grid, the nearest two of their middles
/// compared (Voronoi).
fn scute(p: Vec3) -> f32 {
    let cell = p.floor();
    let (mut a, mut b) = (f32::MAX, f32::MAX);
    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                let c = cell + Vec3::new(dx as f32, dy as f32, dz as f32);
                let h = |k: f32| {
                    ((c.x * 127.1 + c.y * 311.7 + c.z * 74.7 + k).sin() * 43_758.547)
                        .fract()
                        .abs()
                };
                let middle = c + Vec3::new(h(0.0), h(19.0), h(41.0)) * 0.8 + Vec3::splat(0.1);
                let d = middle.distance(p);
                if d < a {
                    b = a;
                    a = d;
                } else if d < b {
                    b = d;
                }
            }
        }
    }
    ((b - a) / 0.25).clamp(0.0, 1.0)
}

/// Smooth value noise, 0 to 1.
fn noise(p: Vec3) -> f32 {
    let i = p.floor();
    let f = p - i;
    let u = f * f * (Vec3::splat(3.0) - 2.0 * f);
    let h = |x: f32, y: f32, z: f32| {
        let n = (i.x + x) * 157.0 + (i.y + y) * 113.0 + (i.z + z) * 271.0;
        (n.sin() * 43_758.547).fract().abs()
    };
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let x00 = lerp(h(0.0, 0.0, 0.0), h(1.0, 0.0, 0.0), u.x);
    let x10 = lerp(h(0.0, 1.0, 0.0), h(1.0, 1.0, 0.0), u.x);
    let x01 = lerp(h(0.0, 0.0, 1.0), h(1.0, 0.0, 1.0), u.x);
    let x11 = lerp(h(0.0, 1.0, 1.0), h(1.0, 1.0, 1.0), u.x);
    lerp(lerp(x00, x10, u.y), lerp(x01, x11, u.y), u.z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ridgeback_grows_one_closed_body_bound_to_its_bones() {
        let sp = sim::species::SpeciesId::RIDGEBACK.get();
        let g = grow(sp).expect("a body");
        let verts = g.mesh.count_vertices();
        assert!(verts > 2_000 && verts < 120_000, "{verts}");
        let Some(VertexAttributeValues::Float32x4(w)) =
            g.mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT)
        else {
            panic!("weights");
        };
        assert!(w.iter().all(|w| (w.iter().sum::<f32>() - 1.0).abs() < 1e-3));
        let Some(VertexAttributeValues::Uint16x4(j)) =
            g.mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX)
        else {
            panic!("joints");
        };
        assert!(j.iter().flatten().all(|&b| (b as usize) < sp.bones.len()));
        assert_eq!(g.rest.len(), sp.bones.len());
    }
}
