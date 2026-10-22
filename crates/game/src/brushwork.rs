//! **A character painted, in motion** (`--painted-cast`;
//! [`docs/design/exploration/0025`](../../../docs/design/exploration/0025_painted_in_motion.md)).
//!
//! GW2's concept art is broad flat strokes over a figure, and the one way to
//! keep that in motion is a stroke that is a **particle**: a point on the
//! body, carried by the skeleton, drawn as a flat card square to the eye
//! (Meier 1996). Painted onto the surface a stroke foreshortens and wraps as
//! a limb turns -- printed cloth; laid on the screen, the body slides under
//! it. A particle does neither.
//!
//! Here each model piece gets a second mesh: a few thousand strokes placed
//! over its surface once, when it loads ([`place`]). A stroke is four
//! corners at one point of the bind pose, carrying that point's normal,
//! texture place and joint weights, a direction fixed to the body, and its
//! size; `brushwork.wgsl` skins it exactly as the surface is skinned, turns
//! it square to the eye, lights it in bands and cuts it out in the shape of
//! a flat brush. The model is still drawn under it: the underpainting.

use std::collections::HashMap;

use bevy::asset::weak_handle;
use bevy::gltf::GltfMaterialName;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey, NotShadowCaster};
use bevy::prelude::*;
use bevy::render::mesh::skinning::SkinnedMesh;
use bevy::render::mesh::{
    Indices, MeshVertexBufferLayoutRef, PrimitiveTopology, VertexAttributeValues,
};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
};
use bevy::render::view::NoFrustumCulling;
use view::skeleton::Joint;

use crate::models::{Body, Model};
use crate::still::SkinMaterial;

const SHADER: Handle<Shader> = weak_handle!("7d3b2e91-5c4a-4f08-b6e1-9a2c8d5f3e47");

/// Whether the cast is painted, and which of them: the Shadow Reaver and
/// the Champion first (the owner, 2026-10-21), the Blood mage by following
/// `docs/design/cast-look.md` (2026-10-22); the rest as they were.
pub fn on() -> bool {
    crate::platform::flag("--painted-cast")
}

fn painted(class: sim::Class) -> bool {
    matches!(
        class,
        sim::Class::ShadowReaver | sim::Class::Champion | sim::Class::BloodMage
    )
}

/// How a stroke is lit and laid, as one uniform.
mod look {
    #![allow(dead_code)]
    use bevy::prelude::*;
    use bevy::render::render_resource::ShaderType;

    #[derive(ShaderType, Reflect, Debug, Clone, Copy)]
    pub struct BrushLook {
        /// The piece's own tint over its texture (a seat's dye).
        pub tint: Vec4,
        /// What the light side is multiplied by: a warm white, and in `w`
        /// how bright.
        pub light: Vec4,
        /// What the shadow side is: a cool violet, and in `w` how deep.
        pub shade: Vec4,
        /// How far a stroke is pulled toward the eye (metres); how strong
        /// the cool rim is; how much each stroke's value wanders; the
        /// brush's bristle count.
        pub knobs: Vec4,
    }
}
pub use look::BrushLook;

/// The material the strokes are drawn in: how a stroke's colour is taken
/// from the picture under it and simplified.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct Brushwork {
    #[uniform(0)]
    pub look: BrushLook,
}

impl Material for Brushwork {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Handle(SHADER)
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(SHADER)
    }

    /// Cut out, but not opaque: a stroke is thinner in its middle than at
    /// its edges, where a loaded brush leaves more paint, and the cloth
    /// shows through it (the owner, 2026-10-22). It does its own blending
    /// -- its paint over the picture at its pixel, which it already reads
    /// -- because a blended material is drawn after the copy of the
    /// picture is made only when something else on screen asks for it.
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Mask(0.5)
    }

    /// **Each stroke's colour is the picture's** (Meier 1996): drawn after
    /// everything solid, it reads the colour the lit character already has
    /// under its centre. So strokes and the gaps between them are one
    /// picture -- the same light, shadow and material, hair and steel
    /// included -- and a stroke only simplifies it.
    fn reads_view_transmission_texture(&self) -> bool {
        true
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // A card is square to the eye whichever way round it was wound.
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

pub struct BrushworkPlugin;

impl Plugin for BrushworkPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<Assets<Shader>>().insert(
            SHADER.id(),
            Shader::from_wgsl(include_str!("brushwork.wgsl"), "game/src/brushwork.wgsl"),
        );
        // Kept out of the depth prepass the outline reads, so the line is
        // round the body and not round every stroke; casting no shadow,
        // since the body under them casts it.
        app.add_plugins(MaterialPlugin::<Brushwork> {
            prepass_enabled: false,
            shadows_enabled: false,
            ..default()
        })
        .init_resource::<Placed>();
        if on() {
            app.add_systems(Update, paint);
        }
    }
}

/// The strokes of each model piece, placed once and shared by every seat
/// playing it: by model and by the piece's material name.
#[derive(Resource, Default)]
struct Placed(HashMap<(usize, String), Handle<Mesh>>);

/// A model whose pieces have their strokes.
#[derive(Component)]
struct Painted;

/// Give every model of a painted class its strokes once its pieces are in
/// the world: a stroke mesh beside each skinned piece, skinned to the same
/// joints, in the piece's own colours.
#[allow(clippy::too_many_arguments)] // a Bevy system: its arguments are what it reads
fn paint(
    mut commands: Commands,
    models: Query<(Entity, &Model), Without<Painted>>,
    children: Query<&Children>,
    pieces: Query<(
        &Mesh3d,
        &SkinnedMesh,
        &MeshMaterial3d<SkinMaterial>,
        Option<&GltfMaterialName>,
    )>,
    skins: Res<Assets<SkinMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut placed: ResMut<Placed>,
    mut brushes: ResMut<Assets<Brushwork>>,
) {
    for (root, model) in &models {
        let Body::Her(m) = model.body else {
            commands.entity(root).insert(Painted);
            continue;
        };
        if !painted(crate::models::MODELS[m].0) {
            commands.entity(root).insert(Painted);
            continue;
        }
        let found: Vec<_> = children
            .iter_descendants(root)
            .filter_map(|e| pieces.get(e).ok().map(|p| (e, p)))
            .collect();
        // Not until the model's own materials are on it (`models::ready`).
        if found.is_empty() {
            continue;
        }
        for (entity, (mesh, skin, material, name)) in found {
            let name = name.map(|n| n.0.clone()).unwrap_or_default();
            // Not the hair: its cards are already strands, and a stroke
            // laid over them is a block on the head.
            if name.starts_with("hair") {
                continue;
            }
            // Nor the skin, for now (the owner, 2026-10-22): the clothes
            // first.
            if name == "body" {
                continue;
            }
            let Some(base) = skins.get(&material.0).map(|s| &s.base) else {
                continue;
            };
            let key = (m, name.clone());
            let strokes = match placed.0.get(&key) {
                Some(h) => h.clone(),
                None => {
                    let Some(source) = meshes.get(&mesh.0) else {
                        continue;
                    };
                    let Some(made) = place(source, &name, m as u32) else {
                        continue;
                    };
                    let h = meshes.add(made);
                    placed.0.insert(key, h.clone());
                    h
                }
            };
            let brush = brushes.add(Brushwork {
                look: look_for(base, &name),
            });
            commands.entity(entity).with_child((
                Mesh3d(strokes),
                MeshMaterial3d(brush),
                skin.clone(),
                Transform::default(),
                NotShadowCaster,
                NoFrustumCulling,
            ));
        }
        commands.entity(root).insert(Painted);
    }
}

/// How a piece's strokes simplify the picture: the light warmed, the
/// shade cooled, values gathered into bands.
fn look_for(_base: &StandardMaterial, _name: &str) -> BrushLook {
    BrushLook {
        // How many value bands, and how hard they are (0 the picture's own
        // values, 1 flat bands).
        tint: Vec4::new(4.0, 0.3, 1.08, 0.0),
        light: Vec4::new(1.06, 1.0, 0.9, 0.0),
        shade: Vec4::new(0.88, 0.9, 1.12, 0.0),
        knobs: Vec4::new(0.035, 0.0, 0.05, 11.0),
    }
}

// --- Placing ----------------------------------------------------------------

/// A number from 0 to 1, the same every time for the same `k`.
fn hash(k: u32) -> f32 {
    let mut x = k.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    x = x.wrapping_mul(0x297A_2D39);
    x ^= x >> 15;
    (x & 0x00FF_FFFF) as f32 / 16_777_215.0
}

/// How wide a stroke is, in metres, on most of the body: a small flat
/// brush, the land's brush at the size a figure wants (2026-10-22, the
/// owner: "much smaller", like the terrain's).
const BROAD: f32 = 0.034;

/// How much of the surface the strokes cover, counting overlap: a little
/// over once, so the strokes are a texture over the figure rather than
/// patches on it.
const COVER: f32 = 1.5;

/// The painter's hand: every stroke is turned this far from its part's
/// long axis (radians), so a sleeve's strokes run down it at a slant.
const HAND: f32 = 0.35;

/// How freely the strokes lie, 0 to 1 (`look::strokes`' *loose*): low, so
/// they sit near-parallel as the land's built work does, and read as one
/// hand's brushwork rather than scatter.
const LOOSE: f32 = 0.3;

/// How a stroke's size follows what it is on: broad open cloth (the coat's
/// panels, the back and chest) in a bigger brush; the face and the hands,
/// where the eye goes, in a smaller one and fewer, so the model's own
/// detail shows through. `(size, how many kept)`.
fn focus(joint: usize) -> (f32, f32) {
    if joint >= view::skeleton::JOINT_COUNT {
        // A coat panel.
        (1.9, 1.0)
    } else if joint == Joint::Head.index() {
        // The face is where the eye goes: left nearly as modelled.
        (0.5, 0.2)
    } else if joint == Joint::HandL.index() || joint == Joint::HandR.index() {
        // And what they hold: a blade is read by its edge, and a stroke
        // over it smears it.
        (0.5, 0.12)
    } else if joint == Joint::Neck.index() {
        (0.7, 0.8)
    } else if [
        Joint::Chest,
        Joint::Spine,
        Joint::Root,
        Joint::ThighL,
        Joint::ThighR,
    ]
    .iter()
    .any(|j| j.index() == joint)
    {
        (1.25, 1.0)
    } else {
        (1.0, 1.0)
    }
}

/// How close to an open edge (metres) a stroke may become a wisp, and how
/// many there do: the cloth's hem going to paint, a quarter of the way
/// from fabric to flame (the owner, 2026-10-22).
const WISP_REACH: f32 = 0.03;

/// The rim of a thickened sheet (`openness`): an edge no longer than this
/// (metres; the panels are 4 mm thick, the cloth's own edges 2 cm and up)
/// whose two ends face apart by more than about seventy degrees.
const FOLD: f32 = 0.007;
const FOLD_TURN: f32 = 0.35;
/// A shell with more than this share of its corners on the rim is a band,
/// not a sheet (`openness`).
const BAND: f32 = 0.35;
const WISPS: f32 = 0.12;

/// The strokes over one model piece: placed by area, evenly (stratified
/// over the surface), each at a point of a triangle with that point's
/// normal, texture place and the joints of its nearest corner.
fn place(mesh: &Mesh, name: &str, model: u32) -> Option<Mesh> {
    let pos = match mesh.attribute(Mesh::ATTRIBUTE_POSITION)? {
        VertexAttributeValues::Float32x3(v) => v,
        _ => return None,
    };
    let nrm = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL)? {
        VertexAttributeValues::Float32x3(v) => v,
        _ => return None,
    };
    let uv = match mesh.attribute(Mesh::ATTRIBUTE_UV_0)? {
        VertexAttributeValues::Float32x2(v) => v,
        _ => return None,
    };
    let joints: Vec<[u16; 4]> = match mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX)? {
        VertexAttributeValues::Uint16x4(v) => v.clone(),
        _ => return None,
    };
    let weights = match mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT)? {
        VertexAttributeValues::Float32x4(v) => v,
        _ => return None,
    };
    let tris: Vec<[usize; 3]> = match mesh.indices()? {
        Indices::U16(i) => i
            .chunks_exact(3)
            .map(|t| [t[0] as usize, t[1] as usize, t[2] as usize])
            .collect(),
        Indices::U32(i) => i
            .chunks_exact(3)
            .map(|t| [t[0] as usize, t[1] as usize, t[2] as usize])
            .collect(),
    };
    let v3 = |i: usize| Vec3::from(pos[i]);
    // Which joint moves each corner most.
    let top = |i: usize| {
        let w = weights[i];
        let k = (0..4).fold(0, |b, k| if w[k] > w[b] { k } else { b });
        joints[i][k] as usize
    };

    // Each part's long axis: the main direction its corners spread along.
    let parts = joints
        .iter()
        .flatten()
        .map(|&j| j as usize)
        .max()
        .unwrap_or(0)
        + 1;
    let mut sum = vec![Vec3::ZERO; parts];
    let mut n = vec![0.0f32; parts];
    for i in 0..pos.len() {
        sum[top(i)] += v3(i);
        n[top(i)] += 1.0;
    }
    let mean: Vec<Vec3> = sum.iter().zip(&n).map(|(s, c)| *s / c.max(1.0)).collect();
    let mut spread = vec![Mat3::ZERO; parts];
    for i in 0..pos.len() {
        let d = v3(i) - mean[top(i)];
        spread[top(i)] += Mat3::from_cols(d * d.x, d * d.y, d * d.z);
    }
    let axis: Vec<Vec3> = spread
        .iter()
        .map(|c| {
            // Power iteration from upright: the long axis of the part.
            let mut a = Vec3::new(0.05, 1.0, 0.02);
            for _ in 0..24 {
                a = (*c * a).normalize_or(Vec3::Y);
            }
            a
        })
        .collect();

    let (open, along_open) = openness(pos, nrm, &tris);

    let area: Vec<f32> = tris
        .iter()
        .map(|t| (v3(t[1]) - v3(t[0])).cross(v3(t[2]) - v3(t[0])).length() * 0.5)
        .collect();
    let total: f32 = area.iter().sum();
    let mean_stroke = BROAD * BROAD * 3.4;
    let count = ((total * COVER / mean_stroke) as usize).clamp(16, 30_000);
    let mut running = Vec::with_capacity(area.len());
    let mut acc = 0.0;
    for a in &area {
        acc += a;
        running.push(acc);
    }

    let salt = model.wrapping_mul(7919)
        ^ name
            .bytes()
            .fold(0u32, |h, b| h.wrapping_mul(31) ^ b as u32);
    let (mut p_out, mut n_out, mut uv_out, mut c_out, mut way_out, mut brush_out) =
        (vec![], vec![], vec![], vec![], vec![], vec![]);
    let (mut j_out, mut w_out, mut idx) = (vec![], vec![], vec![]);
    for k in 0..count as u32 {
        let r = |s: u32| hash(salt ^ k.wrapping_mul(2_654_435_761) ^ s.wrapping_mul(40_503));
        // Stratified: the k-th stroke falls in the k-th equal share of the
        // area, so they spread evenly rather than clumping.
        let at = (k as f32 + r(1)) / count as f32 * total;
        let t = running.partition_point(|&c| c < at).min(tris.len() - 1);
        let [a, b, c] = tris[t];
        let (mut u, mut v) = (r(2), r(3));
        if u + v > 1.0 {
            (u, v) = (1.0 - u, 1.0 - v);
        }
        let w = 1.0 - u - v;
        let p = v3(a) * w + v3(b) * u + v3(c) * v;
        let normal = (Vec3::from(nrm[a]) * w + Vec3::from(nrm[b]) * u + Vec3::from(nrm[c]) * v)
            .normalize_or(Vec3::Z);
        let tex = Vec2::from(uv[a]) * w + Vec2::from(uv[b]) * u + Vec2::from(uv[c]) * v;
        let near = if w >= u && w >= v {
            a
        } else if u >= v {
            b
        } else {
            c
        };
        let joint = top(near);
        let (size, keep) = focus(joint);
        if r(4) > keep {
            continue;
        }
        // The stroke's direction: its part's long axis laid on the surface,
        // turned by the painter's hand and a little of its own.
        let mut way = axis[joint] - normal * axis[joint].dot(normal);
        if way.length_squared() < 1e-6 {
            way = normal.any_orthonormal_vector();
        }
        let turn = HAND + (r(5) - 0.5) * LOOSE * 1.6;
        let way = Quat::from_axis_angle(normal, turn) * way.normalize();
        // Two sizes of brush: the masses, and smaller strokes laid over them.
        let detail = r(6) < 0.3;
        let free = 1.0 + (r(7) - 0.5) * LOOSE * 1.4;
        let mut width = BROAD * size * free * if detail { 0.7 } else { 1.0 };
        let mut length = width * (3.0 + 0.8 * r(8));
        // Near an open edge -- a hem, a cuff, the hood's mouth -- the stroke
        // lies along the edge and is no wider than the cloth left beside it,
        // so the edge is found rather than frayed.
        let gap = open[a] * w + open[b] * u + open[c] * v;
        let mut hug = 0.0f32;
        let mut way = way;
        if gap < length * 0.5 {
            let e = along_open[near];
            let e = e - normal * e.dot(normal);
            if e.length_squared() > 1e-8 {
                hug = 1.0 - (gap / (length * 0.5)).clamp(0.0, 1.0);
                way = way
                    .lerp(e.normalize() * e.normalize().dot(way).signum(), hug)
                    .normalize_or(way);
                width = width.min(gap * 2.0 + width * 0.25).max(width * 0.35);
                length = length.max(width * 1.6);
            }
        }
        // At the cloth's edge, a wisp: a tongue of paint lifting off the
        // hem, its root on the cloth and its length out past the edge, the
        // way the cloth would go if it were flame (`brushwork.wgsl` lifts
        // and sways it). Its direction is out of the cloth: down the slope
        // of the distance to the edge.
        let mut kind = if detail { 1.0 } else { 0.0 };
        // Cloth only: not the hair's cards (every card is all edge), the
        // face or what the hands hold.
        let cloth = !name.starts_with("cards")
            && !name.starts_with("hair")
            && size >= 0.7
            && joint != Joint::Neck.index();
        if cloth && gap < WISP_REACH && r(10) < WISPS {
            let (pa, pb, pc) = (v3(a), v3(b), v3(c));
            let n2 = (pb - pa).cross(pc - pa);
            let area2 = n2.length();
            if area2 > 1e-10 {
                let nh = n2 / area2;
                let grad = (nh.cross(pc - pb) * open[a]
                    + nh.cross(pa - pc) * open[b]
                    + nh.cross(pb - pa) * open[c])
                    / area2;
                if let Some(out) = (-grad).try_normalize() {
                    way = out;
                    kind = 2.0;
                    hug = 0.0;
                    width = BROAD * size.min(1.3) * (1.0 + 0.4 * r(11));
                    length = width * (1.8 + 1.0 * r(12));
                }
            }
        }
        let base = p_out.len() as u32;
        for corner in [[-1.0f32, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]] {
            p_out.push(p.to_array());
            n_out.push(normal.to_array());
            uv_out.push(tex.to_array());
            c_out.push(corner);
            way_out.push([way.x, way.y, way.z, r(9)]);
            brush_out.push([width, length, hug, kind]);
            j_out.push(joints[near]);
            w_out.push(weights[near]);
        }
        idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    if idx.is_empty() {
        return None;
    }
    Some(
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, p_out)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, n_out)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv_out)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, c_out)
        .with_inserted_attribute(Mesh::ATTRIBUTE_TANGENT, way_out)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, brush_out)
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_JOINT_INDEX,
            VertexAttributeValues::Uint16x4(j_out),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, w_out)
        .with_inserted_indices(Indices::U32(idx)),
    )
}

/// How far each corner is from the piece's nearest open edge (metres, along
/// the surface), and which way that edge runs. Corners split for a texture
/// seam are welded first, so a seam is not an edge; what is left with one
/// triangle on it is the cloth's real edge.
///
/// **And the rim of a thickened sheet** (2026-10-22). A coat's panels are
/// given a few millimetres of thickness when they are built
/// (`wardrobe.coat_tails`, `g.thicken`), which closes the sheet: every edge
/// has two triangles on it, and no panel on any class had a hem to fray --
/// found painting the Blood mage, whose skirt is all panels. Its edge is
/// where the sheet folds back on itself: an edge no longer than the
/// thickness whose two ends face well apart. Cloth that only bends never
/// does both.
fn openness(pos: &[[f32; 3]], nrm: &[[f32; 3]], tris: &[[usize; 3]]) -> (Vec<f32>, Vec<Vec3>) {
    use std::cmp::Ordering;
    use std::collections::BinaryHeap;

    let mut weld: HashMap<[i32; 3], usize> = HashMap::new();
    let id: Vec<usize> = pos
        .iter()
        .map(|p| {
            let key = [
                (p[0] * 2000.0).round() as i32,
                (p[1] * 2000.0).round() as i32,
                (p[2] * 2000.0).round() as i32,
            ];
            let next = weld.len();
            *weld.entry(key).or_insert(next)
        })
        .collect();
    let count = weld.len();
    let mut at = vec![Vec3::ZERO; count];
    let mut facing = vec![Vec3::ZERO; count];
    for (i, p) in pos.iter().enumerate() {
        at[id[i]] = Vec3::from(*p);
        facing[id[i]] += Vec3::from(nrm[i]);
    }
    let facing: Vec<Vec3> = facing.iter().map(|n| n.normalize_or_zero()).collect();
    let mut edges: HashMap<(usize, usize), u32> = HashMap::new();
    let mut links: Vec<Vec<usize>> = vec![Vec::new(); count];
    for t in tris {
        for k in 0..3 {
            let (a, b) = (id[t[k]], id[t[(k + 1) % 3]]);
            if a == b {
                continue;
            }
            *edges.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            links[a].push(b);
            links[b].push(a);
        }
    }
    let mut dist = vec![f32::INFINITY; count];
    let mut dir = vec![Vec3::ZERO; count];
    // Summed with a consistent sign, so a corner between two edge pieces
    // takes their average.
    let along = |dir: &mut Vec<Vec3>, a: usize, b: usize| {
        let e = at[b] - at[a];
        let e = if e.dot(dir[a]) < 0.0 { -e } else { e };
        dir[a] += e;
        let e = if e.dot(dir[b]) < 0.0 { -e } else { e };
        dir[b] += e;
    };
    for (&(a, b), &n) in &edges {
        if n == 1 {
            dist[a] = 0.0;
            dist[b] = 0.0;
            along(&mut dir, a, b);
        }
    }
    // The folds: each end of one is on the rim. The rim runs along the
    // edges between two such corners that face the same way -- the fold
    // itself runs across the thickness, which is not the hem's direction.
    let rim: Vec<bool> = {
        let mut rim = vec![false; count];
        for &(a, b) in edges.keys() {
            if at[a].distance(at[b]) < FOLD && facing[a].dot(facing[b]) < FOLD_TURN {
                rim[a] = true;
                rim[b] = true;
            }
        }
        rim
    };
    // Only a sheet's rim is a hem. A band -- a belt, a strap, a collar, a
    // cuff's trim -- is nearly all rim, and its edge is not where cloth
    // ends but where a band is narrow: so a shell more than `BAND` rim
    // keeps no rim at all. Measured on the Blood mage: her coat, panels,
    // sleeves and boots are 8 to 20% rim, her belt and straps 55 to 75%.
    let rim = {
        let mut rim = rim;
        let mut seen = vec![false; count];
        for start in 0..count {
            if seen[start] {
                continue;
            }
            seen[start] = true;
            let mut shell = vec![start];
            let mut k = 0;
            while k < shell.len() {
                for &u in &links[shell[k]] {
                    if !seen[u] {
                        seen[u] = true;
                        shell.push(u);
                    }
                }
                k += 1;
            }
            let on = shell.iter().filter(|&&v| rim[v]).count();
            if on as f32 > shell.len() as f32 * BAND {
                for &v in &shell {
                    rim[v] = false;
                }
            }
        }
        rim
    };
    for &(a, b) in edges.keys() {
        if rim[a] && rim[b] && facing[a].dot(facing[b]) > 0.5 {
            dist[a] = 0.0;
            dist[b] = 0.0;
            along(&mut dir, a, b);
        }
    }
    #[derive(PartialEq)]
    struct Near(f32, usize);
    impl Eq for Near {}
    impl PartialOrd for Near {
        fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
            Some(self.cmp(o))
        }
    }
    impl Ord for Near {
        fn cmp(&self, o: &Self) -> Ordering {
            o.0.total_cmp(&self.0)
        }
    }
    let mut heap: BinaryHeap<Near> = (0..count)
        .filter(|&v| dist[v] == 0.0)
        .map(|v| Near(0.0, v))
        .collect();
    while let Some(Near(d, v)) = heap.pop() {
        if d > dist[v] {
            continue;
        }
        for &u in &links[v] {
            let nd = d + at[u].distance(at[v]);
            if nd < dist[u] {
                dist[u] = nd;
                dir[u] = dir[v];
                heap.push(Near(nd, u));
            }
        }
    }
    (
        id.iter().map(|&i| dist[i].min(10.0)).collect(),
        id.iter().map(|&i| dir[i]).collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A strip of cloth `w` by `h`, thickened by `t` the way the wardrobe
    /// thickens a panel: a front and a back joined round their rim, so
    /// every edge has two triangles on it. Normals as smooth shading gives
    /// them -- the rim's corners turned half out.
    type Sheet = (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[usize; 3]>);

    fn thickened(w: usize, h: usize, cell: f32, t: f32) -> Sheet {
        let (mut pos, mut nrm, mut tris) = (vec![], vec![], vec![]);
        let at = |side: usize, i: usize, j: usize| side * (w + 1) * (h + 1) + j * (w + 1) + i;
        for side in 0..2 {
            for j in 0..=h {
                for i in 0..=w {
                    let z = if side == 0 { t * 0.5 } else { -t * 0.5 };
                    pos.push([i as f32 * cell, j as f32 * cell, z]);
                    let edge = i == 0 || j == 0 || i == w || j == h;
                    let mut n = Vec3::Z * if side == 0 { 1.0 } else { -1.0 };
                    if edge {
                        let out = Vec3::new(
                            if i == 0 {
                                -1.0
                            } else if i == w {
                                1.0
                            } else {
                                0.0
                            },
                            if j == 0 {
                                -1.0
                            } else if j == h {
                                1.0
                            } else {
                                0.0
                            },
                            0.0,
                        );
                        n = (n + out.normalize()).normalize();
                    }
                    nrm.push(n.to_array());
                }
            }
            for j in 0..h {
                for i in 0..w {
                    let (a, b, c, d) = (
                        at(side, i, j),
                        at(side, i + 1, j),
                        at(side, i + 1, j + 1),
                        at(side, i, j + 1),
                    );
                    tris.push([a, b, c]);
                    tris.push([a, c, d]);
                }
            }
        }
        // The rim: front and back joined all the way round.
        let mut ring = vec![];
        ring.extend((0..w).map(|i| (i, 0)));
        ring.extend((0..h).map(|j| (w, j)));
        ring.extend((0..w).map(|i| (w - i, h)));
        ring.extend((0..h).map(|j| (0, h - j)));
        for k in 0..ring.len() {
            let (p, q) = (ring[k], ring[(k + 1) % ring.len()]);
            let (a, b) = (at(0, p.0, p.1), at(0, q.0, q.1));
            let (c, d) = (at(1, q.0, q.1), at(1, p.0, p.1));
            tris.push([a, b, c]);
            tris.push([a, c, d]);
        }
        (pos, nrm, tris)
    }

    #[test]
    fn a_thickened_panel_has_a_hem() {
        // A panel: 30 cm by 60 cm in 2 cm cells, 4 mm thick.
        let (pos, nrm, tris) = thickened(15, 30, 0.02, 0.004);
        let (open, way) = openness(&pos, &nrm, &tris);
        let corner = |i: usize, j: usize| j * 16 + i;
        assert_eq!(open[corner(7, 0)], 0.0, "its bottom edge is a hem");
        assert!(
            (open[corner(7, 15)] - 0.14).abs() < 0.01,
            "its middle is 14 cm from the side"
        );
        let along = way[corner(7, 0)].normalize();
        assert!(
            along.x.abs() > 0.9,
            "the hem runs along the bottom: {along}"
        );
    }

    #[test]
    fn a_band_has_no_hem() {
        // A strap: 40 cm by 2 cm, nearly all rim.
        let (pos, nrm, tris) = thickened(20, 1, 0.02, 0.004);
        let (open, _) = openness(&pos, &nrm, &tris);
        assert!(open.iter().all(|&d| d > 0.0));
    }
}
