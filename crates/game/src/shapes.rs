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
//! The one thing it cannot do is a view-dependent effect -- a rim that follows
//! the silhouette as you walk round. That is a real loss and it is not this
//! one: the edges being drawn here are the object's own, which is what a
//! painter outlines anyway.
//!
//! A vertex colour multiplies the material's `base_color`, so everything built
//! here is drawn with a white material and carries its whole colour itself.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use look::edge::{Edge, Line};
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

/// The material a silhouette line is drawn with.
///
/// **Front faces culled**, which is the whole trick. The line is the same shape
/// as the thing, swollen by a few centimetres, drawn inside out: its near side
/// is thrown away, so what is left is its *far* side, which the thing itself
/// covers everywhere except round the outside. What shows is a rim, exactly as
/// wide as the swelling, which is a silhouette by construction rather than by
/// edge detection -- no second pass, no depth buffer to read, nothing that
/// behaves differently in a browser.
///
/// Unlit, because a line is ink and ink is not a surface the sun falls on. It
/// still fogs with distance like everything else, or a far-off island would be
/// drawn in sharp outline against air it should be fading into.
pub fn ink() -> StandardMaterial {
    ink_in([1.0, 1.0, 1.0])
}

/// The same, in one colour, for a thing whose line is not in its vertices --
/// a fighter's limbs, which are unit cubes scaled every frame.
pub fn ink_in(rgb: [f32; 3]) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgb(rgb[0], rgb[1], rgb[2]),
        unlit: true,
        cull_mode: Some(bevy::render::render_resource::Face::Front),
        ..default()
    }
}

/// The shell that draws the line round a box.
///
/// The same box, bigger by the line's width in every direction, with every
/// vertex the line's colour. Built at the larger size rather than pushed out
/// along the vertex normals: a box's normals point three different ways at
/// every corner, so pushing along them tears the corners open, and a silhouette
/// with holes at its corners is worse than none. Every shape here is convex, so
/// growing it is exact.
pub fn boxy_line(size: Vec3, base: [f32; 3], line: Line) -> Mesh {
    let mut mesh = boxy(
        size + Vec3::splat(line.swell * 2.0),
        Vec3::ZERO,
        base,
        &Brush {
            palette: FLAT,
            edge: NONE,
        },
    );
    paint_flat(&mut mesh, line.colour(base));
    mesh
}

/// The same, for a mesh that already exists: a cylinder or a sphere grown by
/// pushing every vertex out from the middle, which is exact for a round thing.
pub fn round_line(mesh: &mut Mesh, line: Line, base: [f32; 3]) {
    if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(p)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    {
        for v in p.iter_mut() {
            let at = Vec3::from_array(*v);
            *v = (at + at.normalize_or_zero() * line.swell).to_array();
        }
    }
    paint_flat(mesh, line.colour(base));
}

/// Make every vertex one colour.
fn paint_flat(mesh: &mut Mesh, rgb: [f32; 3]) {
    let n = mesh.count_vertices();
    let c = tint::linear(rgb);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[c[0], c[1], c[2], 1.0]; n]);
}

/// A palette that changes nothing, and a rule that accents nothing: for
/// building the line's shell, whose colour is set afterwards anyway.
const FLAT: Palette = Palette {
    light: [1.0, 1.0, 1.0],
    accent: [1.0, 1.0, 1.0],
    unify: 0.0,
};
const NONE: Edge = Edge {
    rim: 0.0,
    reach: 1.0,
    crest: 0.0,
    climb: 1.0,
};

/// How many times a flat span that long should be cut up, so a gradient of the
/// accent's reach has vertices to live on.
pub fn cuts_across(span: f32, reach: f32) -> u32 {
    ((span / reach).ceil() as u32).clamp(1, 64)
}
