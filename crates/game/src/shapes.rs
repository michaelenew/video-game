//! Meshes that carry the accent in their vertices.
//!
//! Where the accent goes is `look::edge`, which is a pure function and has no
//! idea what a mesh is. This is the other half: putting that function's answer
//! somewhere the renderer will draw it.
//!
//! **In the vertices, not in a shader.** Every surface here is built once when
//! the arena is drawn and never moves, so the gradient can be worked out then
//! and stored, and the renderer needs to know nothing about any of it. That
//! costs a few hundred vertices per object and buys two things worth more than
//! them: it works unchanged in the browser build, where a custom shader is the
//! thing most likely to behave differently, and it keeps the art direction in a
//! crate with no engine in it, which is the rule in `CLAUDE.md`.
//!
//! The *line* round a thing is not here: that is `crate::outline`, a pass over
//! the finished picture, because a line wants its width in pixels and this file
//! only knows about metres.
//!
//! A vertex colour multiplies the material's `base_color`, so everything built
//! here is drawn with a white material and carries its whole colour itself.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use look::edge::Edge;
use look::{Palette, tint};

/// What paints a surface: an arena's palette, and the rule for where its accent
/// goes.
#[derive(Clone, Copy)]
pub struct Brush {
    pub palette: Palette,
    pub edge: Edge,
}

impl Brush {
    /// The colour at a point, in **linear** light, ready to be a vertex colour.
    fn at(&self, base: [f32; 3], to_edge: f32, up: f32, height: f32) -> [f32; 4] {
        let amount = self.edge.at(to_edge, up, height);
        let c = tint::linear(self.palette.accented(base, amount));
        [c[0], c[1], c[2], 1.0]
    }
}

/// A white material. Everything in this module carries its colour in its
/// vertices, so the material must not tint it.
pub fn plain() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.92,
        ..default()
    }
}

/// A box with the accent along its edges and on its top.
///
/// Not `Cuboid`: a cuboid has four vertices per face, all of them corners, so
/// a gradient stored in them would be a flat wash of accent over the whole
/// thing. These faces get a grid instead -- a line of vertices set in from each
/// edge by exactly the distance the accent reaches, and nothing in between,
/// because nothing in between is where the gradient is doing anything.
///
/// `centre` is where the box will stand, which the crest needs: how much accent
/// a top face gets depends on how high it is, and a mesh does not know where it
/// is going to be put.
pub fn boxy(size: Vec3, centre: Vec3, base: [f32; 3], brush: &Brush) -> Mesh {
    let half = size * 0.5;
    let reach = brush.edge.reach;
    let cuts = |len: f32| -> Vec<f32> {
        let h = len * 0.5;
        if len <= reach * 2.2 {
            // Too narrow for an inset ring: the whole face is edge anyway.
            vec![-h, 0.0, h]
        } else {
            vec![-h, -h + reach, h - reach, h]
        }
    };
    let (cx, cy, cz) = (cuts(size.x), cuts(size.y), cuts(size.z));

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colours: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Six faces, each named by the axis it faces and which way along it.
    for axis in 0..3usize {
        for side in [-1.0f32, 1.0] {
            let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
            let along = |i: usize| -> &Vec<f32> { [&cx, &cy, &cz][i] };
            let (us, vs) = (along(u), along(v));
            let mut normal = [0.0f32; 3];
            normal[axis] = side;
            let base_index = positions.len() as u32;

            for a in us {
                for b in vs {
                    let mut p = [0.0f32; 3];
                    p[axis] = side * half[axis];
                    p[u] = *a;
                    p[v] = *b;
                    positions.push(p);
                    normals.push(normal);
                    // How far in from this face's own boundary: the nearer of
                    // the two directions it can run out in.
                    let to_edge = (half[u] - a.abs()).min(half[v] - b.abs());
                    colours.push(brush.at(base, to_edge, normal[1], centre.y + p[1]));
                }
            }

            // Wound so the face points outward. The two spanned axes are
            // taken in cyclic order, so `u` cross `v` is the axis itself;
            // stepping v before u therefore gives a triangle facing *in*, and
            // the positive side is the one that has to be reversed. Getting
            // this backwards draws every box inside out, which looks like a
            // hole in the level rather than like a winding bug.
            let (nu, nv) = (us.len() as u32, vs.len() as u32);
            for i in 0..nu - 1 {
                for j in 0..nv - 1 {
                    let q = [
                        base_index + i * nv + j,
                        base_index + i * nv + j + 1,
                        base_index + (i + 1) * nv + j + 1,
                        base_index + (i + 1) * nv + j,
                    ];
                    if side > 0.0 {
                        indices.extend_from_slice(&[q[0], q[3], q[2], q[0], q[2], q[1]]);
                    } else {
                        indices.extend_from_slice(&[q[0], q[1], q[2], q[0], q[2], q[3]]);
                    }
                }
            }
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
    .with_inserted_indices(Indices::U32(indices))
}

/// Paint an existing mesh -- a cylinder, a sphere, a subdivided plane.
///
/// The same rule, read off the vertices that are already there. A face's
/// "edge" is where it runs out of object, so for each vertex this measures how
/// far it is from the extremes of the object's own box, in the directions the
/// surface actually runs. On a sphere that puts the accent on the cap and the
/// base and leaves the equator alone, which is what a painter does to a ball.
///
/// Whether it reads at all depends on the primitive having vertices in the
/// right places; Bevy's spheres and cylinders have plenty, a `Plane3d` has four
/// and needs `subdivisions` before it is worth painting.
pub fn paint(mesh: &mut Mesh, half: Vec3, centre: Vec3, base: [f32; 3], brush: &Brush) {
    let Some(positions) = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|a| a.as_float3())
    else {
        return;
    };
    let normals: Vec<[f32; 3]> = match mesh
        .attribute(Mesh::ATTRIBUTE_NORMAL)
        .and_then(|a| a.as_float3())
    {
        Some(n) => n.to_vec(),
        None => vec![[0.0, 1.0, 0.0]; positions.len()],
    };
    let colours: Vec<[f32; 4]> = positions
        .iter()
        .zip(&normals)
        .map(|(p, n)| {
            // Only the directions this surface runs in can reach an edge: a
            // face pointing straight up cannot run out sideways in y.
            let mut to_edge = f32::MAX;
            for axis in 0..3 {
                let flat = 1.0 - n[axis].abs();
                if flat > 0.3 && half[axis] > 0.0 {
                    to_edge = to_edge.min((half[axis] - p[axis].abs()).max(0.0));
                }
            }
            brush.at(base, to_edge, n[1], centre.y + p[1])
        })
        .collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
}

/// How many times a flat span that long should be cut up, so a gradient of the
/// accent's reach has vertices to live on.
pub fn cuts_across(span: f32, reach: f32) -> u32 {
    ((span / reach).ceil() as u32).clamp(1, 64)
}

// ---------------------------------------------------------------------------
// Forms that are not boxes
// ---------------------------------------------------------------------------

/// A box with its corners and edges rounded off: a **superellipsoid**, the
/// one family of surfaces that runs from a sphere to a box on a single
/// number. `rounding` is that number -- 1 is an ellipsoid, 0 would be the box
/// itself, and about 0.3 is a box that has been handled: flat faces, soft
/// edges, nothing that catches the light as a line.
///
/// The surface is `(a·c(u)^e·c(v)^e, b·s(u)^e, c·c(u)^e·s(v)^e)` over latitude
/// `u` and longitude `v`, with `c(w)^e` meaning `sign(cos w)·|cos w|^e`, and the
/// normal is the same expression with `2 - e` for `e`, which is why no normal
/// is estimated from neighbours here. Both come out exactly, at every vertex.
///
/// Unit-sized when `size` is one, for a mesh that is scaled per frame (a
/// fighter's parts); sized when it is not, for one that stands still (a rock).
/// `centre` and `brush` paint it the way [`boxy`] paints a box, from where it
/// will stand; `None` leaves it white for a material to colour.
pub fn soft_box(
    size: Vec3,
    rounding: f32,
    segments: usize,
    paint: Option<(Vec3, [f32; 3], &Brush)>,
) -> Mesh {
    superellipsoid(size, rounding, segments, paint, |_, _| 1.0)
}

/// A rock: a rounded box roughed up. Each vertex is pulled **inward** by a
/// little noise, so the stone never pokes out of the collision box it is
/// drawn for -- a body stops a hand's breadth before a rock it cannot see
/// rather than inside one it can. `seed` makes two rocks of one size two
/// rocks.
pub fn rock(size: Vec3, seed: u32, paint: Option<(Vec3, [f32; 3], &Brush)>) -> Mesh {
    let s = seed.wrapping_mul(0x9E37_79B9) as f32 * 1e-4;
    superellipsoid(size, 0.45, 14, paint, move |p, _| {
        // Two octaves of a smooth value noise over the direction, scaled by
        // the rock's own size so a boulder and a pebble are rough alike.
        let q = p / size.max_element().max(1e-3);
        let n = value_noise(q * 2.3 + Vec3::splat(s)) * 0.65
            + value_noise(q * 5.1 + Vec3::splat(s * 1.7)) * 0.35;
        1.0 - 0.18 * (0.5 + 0.5 * n)
    })
}

/// The superellipsoid itself, with a radial scale per vertex.
fn superellipsoid(
    size: Vec3,
    rounding: f32,
    segments: usize,
    paint: Option<(Vec3, [f32; 3], &Brush)>,
    radial: impl Fn(Vec3, Vec3) -> f32,
) -> Mesh {
    let e = rounding.clamp(0.05, 1.0);
    let half = size * 0.5;
    let (lat, lon) = (segments.max(4), segments.max(4) * 2);
    let pow = |w: f32, e: f32| w.signum() * w.abs().powf(e);
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity((lat + 1) * (lon + 1));
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(positions.capacity());
    let mut colours: Vec<[f32; 4]> = Vec::with_capacity(positions.capacity());
    for i in 0..=lat {
        let u = -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * i as f32 / lat as f32;
        let (su, cu) = u.sin_cos();
        for j in 0..=lon {
            let v = -std::f32::consts::PI + std::f32::consts::TAU * j as f32 / lon as f32;
            let (sv, cv) = v.sin_cos();
            let p = Vec3::new(
                half.x * pow(cu, e) * pow(cv, e),
                half.y * pow(su, e),
                half.z * pow(cu, e) * pow(sv, e),
            );
            let n = Vec3::new(
                pow(cu, 2.0 - e) * pow(cv, 2.0 - e) / half.x.max(1e-4),
                pow(su, 2.0 - e) / half.y.max(1e-4),
                pow(cu, 2.0 - e) * pow(sv, 2.0 - e) / half.z.max(1e-4),
            )
            .normalize_or(Vec3::Y);
            let p = p * radial(p, n);
            positions.push(p.to_array());
            normals.push(n.to_array());
            colours.push(match paint {
                Some((centre, base, brush)) => {
                    let mut to_edge = f32::MAX;
                    for axis in 0..3 {
                        if 1.0 - n[axis].abs() > 0.3 {
                            to_edge = to_edge.min((half[axis] - p[axis].abs()).max(0.0));
                        }
                    }
                    brush.at(base, to_edge, n.y, centre.y + p.y)
                }
                None => [1.0, 1.0, 1.0, 1.0],
            });
        }
    }
    let mut indices: Vec<u32> = Vec::with_capacity(lat * lon * 6);
    let row = (lon + 1) as u32;
    for i in 0..lat as u32 {
        for j in 0..lon as u32 {
            let a = i * row + j;
            let b = a + row;
            // Wound to face outward, with +y up and the longitude running
            // the way `(cos v, sin v)` runs round +y.
            indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
    .with_inserted_indices(Indices::U32(indices))
}

/// Smooth value noise in `-1..1`, from a hash of the lattice points round
/// `p`, blended with a smoothstep. Deterministic, dependency-free, and only
/// ever run when an arena is dressed.
fn value_noise(p: Vec3) -> f32 {
    let hash = |x: i32, y: i32, z: i32| -> f32 {
        let mut h = (x as u32).wrapping_mul(0x8da6_b343)
            ^ (y as u32).wrapping_mul(0xd816_3841)
            ^ (z as u32).wrapping_mul(0xcb1a_b31f);
        h ^= h >> 13;
        h = h.wrapping_mul(0x5bd1_e995);
        h ^= h >> 15;
        (h & 0xffff) as f32 / 32767.5 - 1.0
    };
    let f = p.floor();
    let (x, y, z) = (f.x as i32, f.y as i32, f.z as i32);
    let t = p - f;
    let s = t * t * (Vec3::splat(3.0) - 2.0 * t);
    let lerp = |a: f32, b: f32, w: f32| a + (b - a) * w;
    let c00 = lerp(hash(x, y, z), hash(x + 1, y, z), s.x);
    let c10 = lerp(hash(x, y + 1, z), hash(x + 1, y + 1, z), s.x);
    let c01 = lerp(hash(x, y, z + 1), hash(x + 1, y, z + 1), s.x);
    let c11 = lerp(hash(x, y + 1, z + 1), hash(x + 1, y + 1, z + 1), s.x);
    lerp(lerp(c00, c10, s.y), lerp(c01, c11, s.y), s.z)
}
