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
    superellipsoid(size, (rounding, rounding), segments, paint, |_, _| 1.0)
}

/// A limb: round in cross-section, with ends rounded by `ends` (1 is a
/// capsule's hemisphere, 0.4 a blunt pad). The same surface as [`soft_box`]
/// with its two exponents told apart -- round about the bone, soft along it
/// -- which is what a stretched capsule is not: a capsule scaled long grows
/// pointed ends, and this keeps them blunt whatever the bone's length.
pub fn soft_cylinder(size: Vec3, ends: f32, segments: usize) -> Mesh {
    superellipsoid(size, (ends, 1.0), segments, None, |_, _| 1.0)
}

/// A rock: a rounded box roughed up. Each vertex is pulled **inward** by a
/// little noise, so the stone never pokes out of the collision box it is
/// drawn for -- a body stops a hand's breadth before a rock it cannot see
/// rather than inside one it can. `seed` makes two rocks of one size two
/// rocks.
pub fn rock(size: Vec3, seed: u32, paint: Option<(Vec3, [f32; 3], &Brush)>) -> Mesh {
    let s = seed.wrapping_mul(0x9E37_79B9) as f32 * 1e-4;
    superellipsoid(size, (0.55, 0.55), 16, paint, move |p, n| {
        // Two octaves of a smooth value noise over the direction, scaled by
        // the rock's own size so a boulder and a pebble are rough alike.
        let q = p / size.max_element().max(1e-3);
        let v = value_noise(q * 2.3 + Vec3::splat(s)) * 0.65
            + value_noise(q * 5.1 + Vec3::splat(s * 1.7)) * 0.35;
        // **The top stays where the box says.** A rock is stood on, and feet
        // stand at the box's top; a top pulled inward would leave them in
        // the air. The roughness fades out as the face turns upward.
        let top = n.y.clamp(0.0, 1.0);
        1.0 - 0.28 * (0.5 + 0.5 * v) * (1.0 - top * top)
    })
}

/// A column of raised earth: the Elementalist's stone, and the planted
/// shield's wall. A unit mesh, scaled per frame to the radius and the height
/// the simulation says (`place_structures`), so the shape is in the mesh and
/// the size is the simulation's.
///
/// A lathe rather than a superellipsoid, because the thing that made the
/// first one read as a machined drum was its **top**: a flat cap at the full
/// radius is a perfect circle however lumpy the sides under it are, and from
/// the camera's height the top is most of what you see. So the footprint is
/// what varies -- a rounded square in plan, pulled in by lumps at two scales
/// and narrowing a little toward the top -- and the cap is that same
/// irregular rim filled in, **flat at the full height**, since feet stand
/// on it. Every pull is inward, so the stone stays inside the box the hit
/// test uses. Few facets round, since a stone has facets.
pub fn rock_column(seed: u32) -> Mesh {
    let s = seed.wrapping_mul(0x9E37_79B9) as f32 * 1e-4;
    let (rings, spokes) = (8usize, 18usize);
    // The footprint at angle `a` and height `t` (0 at the foot, 1 at the top),
    // on the unit box: a superellipse pulled in by noise and a taper.
    let foot = |a: f32, t: f32| -> Vec3 {
        let (sa, ca) = a.sin_cos();
        let e = 0.6;
        let sx = ca.signum() * ca.abs().powf(e);
        let sz = sa.signum() * sa.abs().powf(e);
        let q = Vec3::new(sx * 0.5, t - 0.5, sz * 0.5);
        let v = value_noise(q * 6.5 + Vec3::splat(s)) * 0.65
            + value_noise(q * 13.0 + Vec3::splat(s * 2.3)) * 0.35;
        let k = 1.0 - (0.04 + 0.18 * (0.5 + 0.5 * v)) - 0.06 * t;
        Vec3::new(sx * 0.5 * k, t - 0.5, sz * 0.5 * k)
    };
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    // The side: a normal from the surface's own two tangents.
    for j in 0..=rings {
        let t = j as f32 / rings as f32;
        for i in 0..=spokes {
            let a = std::f32::consts::TAU * i as f32 / spokes as f32;
            let p = foot(a, t);
            let da = foot(a + 0.02, t) - foot(a - 0.02, t);
            let dt = foot(a, (t + 0.02).min(1.0)) - foot(a, (t - 0.02).max(0.0));
            let mut n = da.cross(dt).normalize_or(Vec3::X);
            if n.dot(Vec3::new(p.x, 0.0, p.z)) < 0.0 {
                n = -n;
            }
            positions.push(p.to_array());
            normals.push(n.to_array());
        }
    }
    let row = (spokes + 1) as u32;
    let mut tri = |a: u32, b: u32, c: u32, out: Vec3, ps: &[[f32; 3]]| {
        // Wound so the face points `out`: a stone is looked at from outside.
        let (pa, pb, pc) = (
            Vec3::from(ps[a as usize]),
            Vec3::from(ps[b as usize]),
            Vec3::from(ps[c as usize]),
        );
        if (pb - pa).cross(pc - pa).dot(out) >= 0.0 {
            indices.extend_from_slice(&[a, b, c]);
        } else {
            indices.extend_from_slice(&[a, c, b]);
        }
    };
    for j in 0..rings as u32 {
        for i in 0..spokes as u32 {
            let (a, b, c, d) = (
                j * row + i,
                j * row + i + 1,
                (j + 1) * row + i,
                (j + 1) * row + i + 1,
            );
            let p = Vec3::from(positions[a as usize]);
            let out = Vec3::new(p.x, 0.0, p.z);
            tri(a, b, c, out, &positions);
            tri(b, d, c, out, &positions);
        }
    }
    // The caps: the rim at each end filled to its middle, flat.
    for (t, up) in [(1.0f32, Vec3::Y), (0.0, Vec3::NEG_Y)] {
        let centre = positions.len() as u32;
        positions.push([0.0, t - 0.5, 0.0]);
        normals.push(up.to_array());
        let first = positions.len() as u32;
        for i in 0..=spokes {
            let a = std::f32::consts::TAU * i as f32 / spokes as f32;
            positions.push(foot(a, t).to_array());
            normals.push(up.to_array());
        }
        for i in 0..spokes as u32 {
            tri(centre, first + i, first + i + 1, up, &positions);
        }
    }
    let count = positions.len();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0f32, 1.0, 1.0, 1.0]; count])
    .with_inserted_indices(Indices::U32(indices))
}

/// The superellipsoid itself, with a radial scale per vertex. `rounding` is
/// the pair of exponents: along the latitude (the ends, about `y`) and the
/// longitude (the cross-section).
fn superellipsoid(
    size: Vec3,
    rounding: (f32, f32),
    segments: usize,
    paint: Option<(Vec3, [f32; 3], &Brush)>,
    radial: impl Fn(Vec3, Vec3) -> f32,
) -> Mesh {
    let (e1, e2) = (rounding.0.clamp(0.05, 1.0), rounding.1.clamp(0.05, 1.0));
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
                half.x * pow(cu, e1) * pow(cv, e2),
                half.y * pow(su, e1),
                half.z * pow(cu, e1) * pow(sv, e2),
            );
            let n = Vec3::new(
                pow(cu, 2.0 - e1) * pow(cv, 2.0 - e2) / half.x.max(1e-4),
                pow(su, 2.0 - e1) / half.y.max(1e-4),
                pow(cu, 2.0 - e1) * pow(sv, 2.0 - e2) / half.z.max(1e-4),
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

/// Break a painted surface up: each vertex's colour moved a little by a
/// smooth noise over where it is, `amount` at most (0.07 is a floor that is
/// ground rather than paint), with features about `scale` metres across.
/// For the floor, whose height is the simulation's and cannot vary; its
/// colour can.
pub fn mottle(mesh: &mut Mesh, amount: f32, scale: f32, seed: u32) {
    let Some(positions) = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|a| a.as_float3())
    else {
        return;
    };
    let positions: Vec<[f32; 3]> = positions.to_vec();
    let Some(bevy::render::mesh::VertexAttributeValues::Float32x4(colours)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
    else {
        return;
    };
    let s = Vec3::splat(seed.wrapping_mul(0x9E37_79B9) as f32 * 1e-5);
    for (c, p) in colours.iter_mut().zip(&positions) {
        let q = Vec3::from(*p) / scale.max(1e-3) + s;
        let n = value_noise(q) * 0.6 + value_noise(q * 2.7 + Vec3::splat(3.1)) * 0.4;
        let k = 1.0 + amount * n;
        c[0] *= k;
        c[1] *= k;
        c[2] *= k;
    }
}

/// A deterministic 0..1 from a few integers: where the scattered dressing
/// goes, and how big each piece is.
pub fn hash01(a: u32, b: u32, c: u32) -> f32 {
    let mut h =
        a.wrapping_mul(0x8da6_b343) ^ b.wrapping_mul(0xd816_3841) ^ c.wrapping_mul(0xcb1a_b31f);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^= h >> 15;
    (h & 0xff_ffff) as f32 / 0xff_ffff as f32
}

/// Cut stone: a box with every edge taken off at forty-five degrees by
/// `bevel` metres, faces flat. Not a rounded box -- a superellipsoid at a
/// low rounding bends its whole face a little, which on a platform six metres
/// across reads as a cushion -- but a mason's chamfer, which is what dressed
/// stone and sawn timber have. The bevel is clamped to a quarter of the
/// smallest side, so a thin slab keeps a face.
///
/// Twenty-four vertices placed, then every polygon given its own copies with
/// its own flat normal: six inset faces, twelve bevel strips, eight corner
/// triangles. Painted with the brush like [`boxy`], from where it will stand.
pub fn chamfered_box(size: Vec3, bevel: f32, paint: Option<(Vec3, [f32; 3], &Brush)>) -> Mesh {
    let half = size * 0.5;
    let b = bevel.clamp(0.0, size.min_element() * 0.25);
    // v(corner, axis): the corner with `axis` kept at the face and the other
    // two pulled in by the bevel.
    let v = |c: [f32; 3], axis: usize| -> Vec3 {
        let mut p = Vec3::ZERO;
        for i in 0..3 {
            let h = if i == axis { half[i] } else { half[i] - b };
            p[i] = c[i] * h;
        }
        p
    };
    let corners: Vec<[f32; 3]> = (0..8)
        .map(|k| {
            [
                if k & 1 == 0 { -1.0 } else { 1.0 },
                if k & 2 == 0 { -1.0 } else { 1.0 },
                if k & 4 == 0 { -1.0 } else { 1.0 },
            ]
        })
        .collect();
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colours: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    // One polygon, flat, wound to face `outward`; a quad is two triangles.
    let mut polygon = |pts: &[Vec3], outward: Vec3| {
        let n = (pts[1] - pts[0])
            .cross(pts[2] - pts[0])
            .normalize_or(outward);
        let n = if n.dot(outward) < 0.0 { -n } else { n };
        let flip = (pts[1] - pts[0]).cross(pts[2] - pts[0]).dot(outward) < 0.0;
        let base = positions.len() as u32;
        for p in pts {
            positions.push(p.to_array());
            normals.push(n.to_array());
            colours.push(match paint {
                Some((centre, rgb, brush)) => {
                    let mut to_edge = f32::MAX;
                    for axis in 0..3 {
                        if 1.0 - n[axis].abs() > 0.3 {
                            to_edge = to_edge.min((half[axis] - p[axis].abs()).max(0.0));
                        }
                    }
                    brush.at(rgb, to_edge, n.y, centre.y + p.y)
                }
                None => [1.0, 1.0, 1.0, 1.0],
            });
        }
        for t in 1..pts.len() as u32 - 1 {
            if flip {
                indices.extend_from_slice(&[base, base + t + 1, base + t]);
            } else {
                indices.extend_from_slice(&[base, base + t, base + t + 1]);
            }
        }
    };
    // The six faces: the four corners on that side, in a ring.
    for axis in 0..3usize {
        for side in [-1.0f32, 1.0] {
            let (u, w) = ((axis + 1) % 3, (axis + 2) % 3);
            let ring = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
            let pts: Vec<Vec3> = ring
                .iter()
                .map(|(a, c)| {
                    let mut corner = [0.0; 3];
                    corner[axis] = side;
                    corner[u] = *a;
                    corner[w] = *c;
                    v(corner, axis)
                })
                .collect();
            let mut outward = Vec3::ZERO;
            outward[axis] = side;
            polygon(&pts, outward);
        }
    }
    // The twelve edges: along axis `along`, between faces `a` and `c`.
    for along in 0..3usize {
        let (a, c) = ((along + 1) % 3, (along + 2) % 3);
        for sa in [-1.0f32, 1.0] {
            for sc in [-1.0f32, 1.0] {
                let mut lo = [0.0; 3];
                lo[along] = -1.0;
                lo[a] = sa;
                lo[c] = sc;
                let mut hi = lo;
                hi[along] = 1.0;
                let pts = [v(lo, a), v(hi, a), v(hi, c), v(lo, c)];
                let mut outward = Vec3::ZERO;
                outward[a] = sa;
                outward[c] = sc;
                polygon(&pts, outward);
            }
        }
    }
    // The eight corners.
    for c in &corners {
        let pts = [v(*c, 0), v(*c, 1), v(*c, 2)];
        polygon(&pts, Vec3::from(*c));
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

/// A floor with relief: a grid of `cell` metres over `size`, each vertex at
/// the height `h(x, z)` gives for its place in the arena (the mesh is centred
/// on `centre`), with normals from the slopes either side. Where the floor is
/// flat the plane is cheaper; this is for the arenas that have hills
/// (`sim::arena::relief`), and it carries no colour until it is painted.
pub fn ground_grid(size: Vec2, centre: Vec3, cell: f32, h: &dyn Fn(f32, f32) -> f32) -> Mesh {
    let nx = ((size.x / cell).ceil() as usize).clamp(1, 400);
    let nz = ((size.y / cell).ceil() as usize).clamp(1, 400);
    let (dx, dz) = (size.x / nx as f32, size.y / nz as f32);
    let at = |i: usize, j: usize| -> Vec3 {
        let x = -size.x * 0.5 + dx * i as f32;
        let z = -size.y * 0.5 + dz * j as f32;
        Vec3::new(x, h(centre.x + x, centre.z + z), z)
    };
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity((nx + 1) * (nz + 1));
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(positions.capacity());
    for j in 0..=nz {
        for i in 0..=nx {
            let p = at(i, j);
            // Central differences, one cell either way, clamped at the rim.
            let (xa, xb) = (at(i.saturating_sub(1), j), at((i + 1).min(nx), j));
            let (za, zb) = (at(i, j.saturating_sub(1)), at(i, (j + 1).min(nz)));
            let n = (xb - xa).cross(zb - za).normalize_or(Vec3::Y);
            let n = if n.y < 0.0 { -n } else { n };
            positions.push(p.to_array());
            normals.push(n.to_array());
        }
    }
    let row = (nx + 1) as u32;
    let mut indices: Vec<u32> = Vec::with_capacity(nx * nz * 6);
    for j in 0..nz as u32 {
        for i in 0..nx as u32 {
            let a = j * row + i;
            let b = a + row;
            // Facing up: +x by +z spans the floor, and up is the right-hand
            // side of z then x.
            indices.extend_from_slice(&[a, a + 1, b, a + 1, b + 1, b]);
        }
    }
    let count = positions.len();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0f32, 1.0, 1.0, 1.0]; count])
    .with_inserted_indices(Indices::U32(indices))
}

/// A disc of floor at the simulation's own heights, for a region of a floor
/// that has relief: rings of `cell` from the middle out, each vertex at
/// `h(x, z)` in arena coordinates, so a trodden patch or a pool lies *on* a
/// swell rather than floating flat through it. Normals from the height
/// function, like [`ground_grid`]'s. `centre.y` is the lift above the floor.
pub fn ground_disc(radius: f32, centre: Vec3, cell: f32, h: &dyn Fn(f32, f32) -> f32) -> Mesh {
    let rings = ((radius / cell).ceil() as usize).clamp(1, 64);
    let spokes = ((2.0 * std::f32::consts::PI * radius / cell).ceil() as usize).clamp(8, 128);
    let at = |x: f32, z: f32| Vec3::new(x, h(centre.x + x, centre.z + z) + centre.y, z);
    let normal = |x: f32, z: f32| {
        let e = cell * 0.5;
        let n = (at(x + e, z) - at(x - e, z))
            .cross(at(x, z + e) - at(x, z - e))
            .normalize_or(Vec3::Y);
        if n.y < 0.0 { -n } else { n }
    };
    let mut positions: Vec<[f32; 3]> = vec![at(0.0, 0.0).to_array()];
    let mut normals: Vec<[f32; 3]> = vec![normal(0.0, 0.0).to_array()];
    for r in 1..=rings {
        let rr = radius * r as f32 / rings as f32;
        for s in 0..spokes {
            let a = std::f32::consts::TAU * s as f32 / spokes as f32;
            let (x, z) = (rr * a.cos(), rr * a.sin());
            positions.push(at(x, z).to_array());
            normals.push(normal(x, z).to_array());
        }
    }
    let idx = |r: usize, s: usize| -> u32 {
        if r == 0 {
            0
        } else {
            (1 + (r - 1) * spokes + (s % spokes)) as u32
        }
    };
    let mut indices: Vec<u32> = Vec::with_capacity(rings * spokes * 6);
    for s in 0..spokes {
        // Facing up: with +x by +z spanning the floor, up is z then x.
        indices.extend_from_slice(&[idx(0, 0), idx(1, s + 1), idx(1, s)]);
    }
    for r in 1..rings {
        for s in 0..spokes {
            let (a, b, c, d) = (idx(r, s), idx(r, s + 1), idx(r + 1, s), idx(r + 1, s + 1));
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }
    let count = positions.len();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0f32, 1.0, 1.0, 1.0]; count])
    .with_inserted_indices(Indices::U32(indices))
}

/// Scale every vertex colour by a factor of its height, `shade(y)`: how the
/// floor's relief is lit (`look::palette::relief_shade`). The rule is the
/// look's; this only applies it.
pub fn shade_by_height(mesh: &mut Mesh, shade: &dyn Fn(f32) -> f32) {
    let Some(positions) = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|a| a.as_float3())
    else {
        return;
    };
    let heights: Vec<f32> = positions.iter().map(|p| p[1]).collect();
    let Some(bevy::render::mesh::VertexAttributeValues::Float32x4(colours)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
    else {
        return;
    };
    for (c, y) in colours.iter_mut().zip(&heights) {
        let k = shade(*y);
        c[0] *= k;
        c[1] *= k;
        c[2] *= k;
    }
}
