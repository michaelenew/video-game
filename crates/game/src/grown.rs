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
//! 4. **Toward the sculpted target** (2026-10-24, exploration 0029): parts
//!    fuller than their boxes, a carapace grown off the skin wherever the
//!    table says armour (`crate::carapace`), hard parts by rule
//!    (`crate::horns`), paint by region, and the fine detail worked out in
//!    the skin's shader from the rest pose each vertex carries.
//!
//! What the simulation collides and hits against is unchanged: the boxes are
//! still the truth, and the debug overlay still draws them. The weak points
//! stand out of the body as crests of flesh whose fissures glow, so they say
//! what they are from across the arena as the pads did.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use sim::beast::{Bone, Leg, Part};
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
    /// How many cells of body were cut off for not being joined to the
    /// rest: nought, or the sinews have a gap.
    pub dropped: usize,
    /// A body formed by hand rather than grown ([`modelled`]): its own
    /// colours and its glowing flesh.
    pub own: Option<Own>,
    /// Per vertex, what it is (exploration 0029): how much it is stone (the
    /// carapace) and soft (a weak point's flesh), which hard part it is (0
    /// none, 1 horn or spine, 2 claw or tusk, 3 quill) and how far along it.
    pub region: Vec<[f32; 4]>,
    /// Per vertex: metres to the groove between scutes, its scute's own
    /// number, how open it is to the sky (occlusion), how near a weak point.
    pub stone: Vec<[f32; 4]>,
    /// The species' build, which its paint reads (feathered, segmented),
    /// and its body's own colour (its look's armour), for what grows off it.
    pub build: crate::species::Build,
    pub armour: [f32; 3],
}

/// What a body formed by hand brings that a grown one works out: the
/// colour each vertex is tinted (white: the texture is the paint), which
/// part each vertex belongs to (so a broken foot shows where it is), and
/// its own material -- the colour, normal, glow and roughness maps baked
/// off its sculpt, the glow being where its weak points are.
pub struct Own {
    pub base: Vec<[f32; 4]>,
    pub part: Vec<u16>,
    pub material: StandardMaterial,
}

/// **What a body is grown from** (2026-10-24, exploration 0031): a skeleton,
/// boxes on it and the legs through them, with the frame each box is written
/// in, what the species' build and look say, and anything its source knows
/// that the boxes do not. A species' table makes one ([`Anatomy::of_species`]);
/// so does a bred genome's body (`crate::bred`), whose boxes run along their
/// own axes rather than their bones'. The growing reads nothing else.
#[derive(Clone)]
pub struct Anatomy {
    pub bones: Vec<Bone>,
    /// Every part, its box already in metres.
    pub parts: Vec<Part>,
    pub legs: Vec<Leg>,
    /// Every bone's rest frame, and every part's: the frame its box is in.
    pub rest: Vec<Mat4>,
    pub frames: Vec<Mat4>,
    /// Which of a part's box axes runs along it, where the source knows
    /// (a bred part runs along its own `x`); the bones say otherwise.
    pub long: Vec<Option<usize>>,
    /// What a part is, where the source knows better than its box
    /// (`crate::anatomy::role`): a bred raptorial limb's foot is a blade
    /// though nothing breaks it.
    pub roles: Vec<Option<crate::anatomy::Role>>,
    /// A part's own contour, where the source says (a bred upright thorax
    /// is a stalk); before the build's table, which is by part index.
    pub shapes: Vec<Option<crate::contour::Contour>>,
    /// Whether a part closes past the end of its box as a hemisphere of its
    /// own end's radius, rather than flat: a bred body's limb segments meet
    /// joint to joint, and flat ends there read as pipes.
    pub capsules: bool,
    pub build: crate::species::Build,
    pub look: Look,
}

impl Anatomy {
    /// A species' table, at its tuned size.
    pub fn of_species(sp: &'static Species) -> Anatomy {
        let rest = rest_frames(sp);
        let parts: Vec<Part> = (0..sp.parts.len())
            .map(|i| Part {
                shape: sp.shape(i),
                ..sp.parts[i]
            })
            .collect();
        let frames = parts.iter().map(|p| rest[p.shape.bone]).collect();
        Anatomy {
            bones: sp.bones.to_vec(),
            legs: sp.legs.to_vec(),
            long: vec![None; parts.len()],
            roles: vec![None; parts.len()],
            shapes: vec![None; parts.len()],
            capsules: false,
            parts,
            rest,
            frames,
            build: *crate::species::build(sp.id),
            look: *crate::species::look(sp.id),
        }
    }

    /// A part's box, in metres, in its frame.
    pub fn shape(&self, i: usize) -> sim::beast::Shape {
        self.parts[i.min(self.parts.len() - 1)].shape
    }

    /// How thick a part is: the middle of its box's three half-sizes, in
    /// its own frame -- a leg's across it, however the leg is turned. Its
    /// bounds in the world would add the length of a leg that leans.
    pub fn thickness(&self, i: usize) -> f32 {
        let s = self.shape(i);
        let h = fx3(s.max.sub(s.min)) * 0.5;
        h.x + h.y + h.z - h.max_element() - h.min_element()
    }
}

/// **A body formed by hand** (the owner, 2026-10-23: "take the current model,
/// form it by hand into a creature you might find in Monster Hunter or Dark
/// Souls... we'll refine that first, then update the algorithms to get there
/// without manual effort"): the file `art/<species>/build.py` writes over the
/// species' own skeleton (`docs/design/exploration/0029_a_target_body.md`).
/// It is the target a grown body is measured against, and it is drawn in
/// its place -- unless `--grown-creatures` asks for the algorithm, to compare.
pub fn model_of(id: sim::species::SpeciesId) -> Option<&'static str> {
    if !grows(id) || crate::platform::flag("--grown-creatures") {
        return None;
    }
    MODELLED
        .iter()
        .find(|(s, _)| *s == id)
        .map(|(_, file)| *file)
}

/// The species with a formed body (`art/<species>/build.py`), and its file:
/// the Ridgeback first (2026-10-23), then the Galewing and the Mantis
/// (2026-10-24, exploration 0030).
const MODELLED: [(sim::species::SpeciesId, &str); 3] = [
    (sim::species::SpeciesId::RIDGEBACK, "ridgeback.glb"),
    (sim::species::SpeciesId::GALEWING, "galewing.glb"),
    (sim::species::SpeciesId::MANTIS, "mantis.glb"),
];

/// Whether creatures are drawn as boxes, as they were before bodies were
/// grown (`--box-creatures`): to compare, and to fall back.
pub fn off() -> bool {
    crate::platform::flag("--box-creatures")
}

/// Which species are drawn with a grown body: the Ridgeback first (the owner,
/// 2026-10-23: "start with the Ridgeback"), then the Galewing and the Mantis
/// (2026-10-24: "two other creatures hand crafted then a growing algorithm
/// to match") -- the ones with a formed target to grow toward.
pub fn grows(id: sim::species::SpeciesId) -> bool {
    !off() && MODELLED.iter().any(|(s, _)| *s == id)
}

/// Is this part drawn as the grown body rather than as its own box?
pub fn covers(sp: &Species, index: usize) -> bool {
    // Weak points too: a grown body stands them out of its skin as glowing
    // flesh (exploration 0029), and a formed one paints them so.
    grows(sp.id) && !sp.parts[index].shape.hollow
}

/// Which parts make the body: everything with a body of its own, but the
/// weak points, which are drawn as themselves on it, and a hollow part, which
/// is a room, not flesh.
fn in_body(sp: &Anatomy, index: usize) -> bool {
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
    /// Its long axis (0, 1 or 2) and which end of it is the joint's (+1 or
    /// -1): what its contour is laid along.
    long: usize,
    root: f32,
    /// The shape it is grown in (`crate::contour`), and which way is up and
    /// forward for it, in the box's frame.
    contour: crate::contour::Contour,
    up: Vec3,
    fwd: Vec3,
    /// Its bounds in the world, grown by the blend's reach.
    lo: Vec3,
    hi: Vec3,
    bone: usize,
    /// Its armour (`crate::carapace`), where it has any.
    shell: Option<crate::carapace::Shell>,
    /// A segmented body's rings along it (`Build::chitin`): how far apart,
    /// and how deep the groove between two (0: none).
    rings: (f32, f32),
    /// Closed past its ends as a capsule ([`Anatomy::capsules`]).
    capsule: bool,
}

impl Lump {
    /// Signed distance to the contoured part: negative inside.
    fn distance(&self, p: Vec3) -> f32 {
        let q = self.to_local.transform_point3(p) - self.middle;
        let at = |q: Vec3| {
            crate::contour::distance(
                &self.contour,
                q,
                self.half,
                self.long,
                self.root,
                self.up,
                self.fwd,
            )
        };
        let over = q[self.long].abs() - self.half[self.long];
        let d = if self.capsule && over > 0.0 {
            // Past the end: a hemisphere of the end's own radius.
            let mut end = q;
            end[self.long] = q[self.long].signum() * self.half[self.long];
            let mut axis = Vec3::ZERO;
            axis[self.long] = end[self.long];
            let radius = (-at(axis)).max(1e-3);
            let a = at(end);
            Vec2::new(over, (a + radius).max(0.0)).length() - radius
        } else {
            at(q)
        };
        let (gap, depth) = self.rings;
        if gap <= 0.0 {
            return d;
        }
        // Each segment rising toward its back edge, a groove between two:
        // an insect's abdomen, a millipede's.
        let t = q[self.long] / gap;
        let f = t - t.floor();
        let groove = 1.0 - smoothstep(0.0, 0.18, f.min(1.0 - f) * 2.0);
        // Not at the ends, where the part has thinned and a groove would
        // cut its tip off.
        let end = (q[self.long] / self.half[self.long]).abs();
        d + depth * smoothstep(0.92, 0.7, end) * (groove - 0.25 * f * self.root.signum())
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
/// the surface can hold. About five and a half centimetres on the Ridgeback:
/// fine enough for the carapace's grooves to read in the mesh, the rest of
/// its detail being the skin shader's (`skin.wgsl`).
const CELLS: f32 = 260.0;

/// How far the smooth union reaches, as a share of a typical part's
/// thickness: a joint is filled about this far up each side of it.
const BLEND: f32 = 0.55;

/// How many bands cross the back from nose to tail.
const BANDS: f32 = 9.0;

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// **What joins the parts** (2026-10-23; the owner: *we can't have floating
/// sections of the body plan*): a capsule along every bone from its joint to
/// its parent's, and from every part's middle to its bone's joint, so the
/// body is connected along the skeleton by construction, whatever gaps the
/// boxes leave. Its radius is a share of the thinner parts it joins.
struct Sinew {
    a: Vec3,
    b: Vec3,
    radius: f32,
    lo: Vec3,
    hi: Vec3,
}

impl Sinew {
    fn distance(&self, p: Vec3) -> f32 {
        let ab = self.b - self.a;
        let t = ((p - self.a).dot(ab) / ab.length_squared().max(1e-9)).clamp(0.0, 1.0);
        (p - (self.a + ab * t)).length() - self.radius
    }

    fn near(&self, p: Vec3) -> bool {
        p.cmpge(self.lo).all() && p.cmple(self.hi).all()
    }
}

/// How thick a sinew is, as a share of the thinner part's width.
const SINEW: f32 = 0.42;

/// A part's contour inferred from what it is: its bone's name, which way its
/// long axis points, and the species' build -- without its place in a chain.
fn inferred(sp: &Anatomy, index: usize, rest: &[Mat4]) -> crate::contour::Contour {
    use crate::contour::{Contour, Kind};
    let build = &sp.build;
    let look = &sp.look;
    let shape = sp.shape(index);
    let bone = &sp.bones[shape.bone];
    let name = bone.name;
    let half = fx3(shape.max.sub(shape.min)) * 0.5;
    let long = long_of(sp, index, rest, half);
    let mut axis = Vec3::ZERO;
    axis[long] = 1.0;
    let world_axis = sp.frames[index].transform_vector3(axis);
    let vertical = world_axis.y.abs() > 0.7;
    let (lo, hi) = world_box(sp, rest, index);
    let role = crate::anatomy::role(sp, index, lo, hi);
    let c = if role != crate::anatomy::Role::Body {
        // A wing, a fan of a tail, a scythe: flat, with a thickened edge.
        Contour::of(Kind::Wing)
    } else if index == look.head || name.starts_with("head") {
        Contour::of(Kind::Wedge)
    } else if name.starts_with("tail") {
        Contour::of(Kind::Whip)
    } else if name.starts_with("wing") {
        Contour::of(Kind::Wing)
    } else if name.starts_with("neck") {
        Contour::of(Kind::Column).between(Contour::of(Kind::Taper), 0.5)
    } else if vertical {
        Contour::of(Kind::Taper).between(Contour::of(Kind::Column), build.heavy)
    } else {
        Contour::of(Kind::Barrel)
    };
    c.fat(build.fat).muscle(build.muscle)
}

/// The contour a part is grown in: the species' own word for it, or one
/// inferred from what the part is; and its place in its chain.
fn contour_of(sp: &Anatomy, index: usize, rest: &[Mat4]) -> crate::contour::Contour {
    let build = &sp.build;
    let said = sp.shapes[index].or_else(|| {
        build
            .contours
            .iter()
            .find(|(i, _)| *i == index)
            .map(|(_, c)| *c)
    });
    let c = said.unwrap_or_else(|| inferred(sp, index, rest));
    // **A chain of parts is one contour** (a tail's three segments thinning
    // to one point, a leg's two to one foot): the parts along one line of
    // bones that are the same kind, and this part's share of their length.
    // A part's own say on its span stands.
    if said.is_some_and(|s| s.span != (0.0, 1.0)) {
        return c;
    }
    let (from, to) = chain_span(sp, index, rest);
    c.span(from, to)
}

/// Which kind a part mostly is.
fn main_kind(sp: &Anatomy, i: usize, rest: &[Mat4]) -> usize {
    let c = sp.shapes[i]
        .or_else(|| {
            sp.build
                .contours
                .iter()
                .find(|(j, _)| *j == i)
                .map(|(_, c)| *c)
        })
        .unwrap_or_else(|| inferred(sp, i, rest));
    c.kinds
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(k, _)| k)
        .unwrap_or(0)
}

/// Where `index` lies along the chain of same-kind parts its bone is in: up
/// the bones to the first ancestor whose part is another kind, then down
/// through single children of the same kind. Shares by each part's length.
fn chain_span(sp: &Anatomy, index: usize, rest: &[Mat4]) -> (f32, f32) {
    let part_on = |bone: usize| {
        (0..sp.parts.len()).find(|&i| in_body(sp, i) && sp.parts[i].shape.bone == bone)
    };
    let kind = main_kind(sp, index, rest);
    let mut bone = sp.parts[index].shape.bone;
    loop {
        let parent = sp.bones[bone].parent;
        if parent == sim::beast::NO_PARENT {
            break;
        }
        match part_on(parent) {
            Some(p) if main_kind(sp, p, rest) == kind => bone = parent,
            _ => break,
        }
    }
    let mut chain = vec![];
    loop {
        if let Some(p) = part_on(bone) {
            chain.push(p);
        }
        let kids: Vec<usize> = (0..sp.bones.len())
            .filter(|&b| sp.bones[b].parent == bone)
            .filter(|&b| part_on(b).is_some_and(|p| main_kind(sp, p, rest) == kind))
            .collect();
        if kids.len() != 1 {
            break;
        }
        bone = kids[0];
    }
    let length = |i: usize| {
        let sh = sp.shape(i);
        let h = fx3(sh.max.sub(sh.min)) * 0.5;
        h[long_of(sp, i, rest, h)] * 2.0
    };
    let total: f32 = chain.iter().map(|&i| length(i)).sum();
    let before: f32 = chain
        .iter()
        .take_while(|&&i| i != index)
        .map(|&i| length(i))
        .sum();
    if total <= 0.0 || !chain.contains(&index) {
        return (0.0, 1.0);
    }
    (before / total, (before + length(index)) / total)
}

/// A part's box in the world at rest, as its bounds.
fn world_box(sp: &Anatomy, _rest: &[Mat4], i: usize) -> (Vec3, Vec3) {
    let s = sp.shape(i);
    let (a, b) = (fx3(s.min), fx3(s.max));
    let m = sp.frames[i];
    let mut lo = Vec3::splat(f32::MAX);
    let mut hi = Vec3::splat(f32::MIN);
    for c in 0..8 {
        let k = Vec3::new(
            if c & 1 == 0 { a.x } else { b.x },
            if c & 2 == 0 { a.y } else { b.y },
            if c & 4 == 0 { a.z } else { b.z },
        );
        let w = m.transform_point3(k);
        lo = lo.min(w);
        hi = hi.max(w);
    }
    (lo, hi)
}

/// Which of a part's box axes runs along its bone: the one most along the
/// line from the bone's parent to it (or, for the root, from it to its
/// first child), so a torso part lies along the body, a leg's runs down it
/// and a tail's back -- unless that side is under half the box's longest,
/// when the box's own shape says more.
/// Which of part `i`'s box axes runs along it: what its source says, or
/// along its bone.
fn long_of(sp: &Anatomy, i: usize, rest: &[Mat4], half: Vec3) -> usize {
    sp.long[i].unwrap_or_else(|| along_bone(sp, sp.parts[i].shape.bone, rest, half))
}

fn along_bone(sp: &Anatomy, bone: usize, rest: &[Mat4], half: Vec3) -> usize {
    let at = |b: usize| rest[b].transform_point3(Vec3::ZERO);
    let parent = sp.bones[bone].parent;
    let dir = if parent != sim::beast::NO_PARENT {
        at(bone) - at(parent)
    } else {
        match (0..sp.bones.len()).find(|&c| sp.bones[c].parent == bone && sp.bones[c].side == 0) {
            Some(c) => at(c) - at(bone),
            None => return longest(half),
        }
    };
    let local = rest[bone].inverse().transform_vector3(dir).abs();
    let k = if local.x >= local.y && local.x >= local.z {
        0
    } else if local.y >= local.z {
        1
    } else {
        2
    };
    if half[k] >= half.max_element() * 0.5 {
        k
    } else {
        longest(half)
    }
}

fn longest(half: Vec3) -> usize {
    if half.x >= half.y && half.x >= half.z {
        0
    } else if half.y >= half.z {
        1
    } else {
        2
    }
}

/// Grow `sp`'s body. `None` for a species with no part that makes one.
pub fn grow(sp: &Anatomy) -> Option<Grown> {
    let rest = sp.rest.clone();
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
    let build = &sp.build;
    let look = &sp.look;
    for &i in &indices {
        let s = sp.shape(i);
        let (a, b) = (fx3(s.min), fx3(s.max));
        let mut half = ((b - a) * 0.5).max(Vec3::splat(0.005));
        let mut middle = (a + b) * 0.5;
        let to_world = sp.frames[i];
        // The long axis follows the bone, not the box's longest side: the
        // Ridgeback's barrel is wider than it is long, and contoured along its
        // longest side it was a tube lying across the body (2026-10-23). And it
        // is fixed before the part is made fuller, which must never turn it.
        let long = long_of(sp, i, &rest, half);
        // **A foot ends at its wrist** (2026-10-24): where the leg has a paw
        // (`sim::beast::Bone::paw`), the lower leg's flesh stops at the
        // joint and the paw grown below it (`horns::paws`) meets the floor.
        // The box itself still reaches the ground; this is only the body.
        // A species that grows no paw (`Build::claws` zero) keeps its legs
        // to the floor.
        if let Some(pb) = (0..sp.bones.len()).find(|&b| {
            build.claws > 0
                && sp.bones[b].paw
                && sp.bones[b].parent == s.bone
                && sp.legs.iter().any(|l| l.foot == i)
        }) {
            let joint = to_world
                .inverse()
                .transform_point3(rest[pb].transform_point3(Vec3::ZERO))[long];
            let root = if middle[long] > 0.0 { -1.0 } else { 1.0 };
            let top = middle[long] + root * half[long];
            if (top - joint) * root > 0.2 {
                middle[long] = (top + joint) * 0.5;
                half[long] = (top - joint).abs() * 0.5;
            }
        }
        // **What the part is** (`crate::anatomy`, exploration 0030): a
        // feathered wing's flesh is the front of its box, the flight
        // feathers the rest; a feathered fan's is a stub at the body, the
        // feathers filling the box; a beaked head's is its back, the beak
        // its front.
        {
            let (wlo, whi) = world_box(sp, &rest, i);
            let role = crate::anatomy::role(sp, i, wlo, whi);
            let fwd = to_world.inverse().transform_vector3(Vec3::X);
            let a = (0..3)
                .max_by(|&p, &q| fwd[p].abs().total_cmp(&fwd[q].abs()))
                .unwrap_or(0);
            let front = fwd[a].signum();
            let keep = |middle: &mut Vec3, half: &mut Vec3, share: f32, from_front: bool| {
                let sign = if from_front { front } else { -front };
                let edge = middle[a] + sign * half[a];
                half[a] *= share;
                middle[a] = edge - sign * half[a];
            };
            if build.feathers > 0.0 && role == crate::anatomy::Role::Wing {
                keep(&mut middle, &mut half, crate::anatomy::WING_FLESH, true);
                // A wing's arm is thinner than the box it is hit in.
                let up = to_world.inverse().transform_vector3(Vec3::Y).abs();
                let thin = (0..3).max_by(|&p, &q| up[p].total_cmp(&up[q])).unwrap_or(1);
                half[thin] *= 0.5;
            }
            if build.feathers > 0.0 && role == crate::anatomy::Role::Fan {
                keep(&mut middle, &mut half, crate::anatomy::FAN_FLESH, true);
                let c = (a + 2) % 3;
                half[c] *= 0.45;
            }
            if build.beak > 0.0 && i == look.head {
                keep(&mut middle, &mut half, 1.0 - build.beak * 0.85, false);
            }
        }
        // **Fuller than the boxes** (exploration 0029): a barrel hangs below
        // its box, so the chest is deep and the belly low between the legs,
        // and a leg is a column thicker than its box, which the body comes
        // down over.
        {
            let mut axis = Vec3::ZERO;
            axis[long] = 1.0;
            let up_local = to_world
                .inverse()
                .transform_vector3(Vec3::Y)
                .normalize_or(Vec3::Y);
            if to_world.transform_vector3(axis).y.abs() > 0.7 {
                let (a1, a2) = ((long + 1) % 3, (long + 2) % 3);
                half[a1] *= build.girth;
                half[a2] *= build.girth;
            } else if main_kind(sp, i, &rest) == crate::contour::Kind::Barrel as usize
                && up_local.y.abs() > 0.9
            {
                let extra = build.belly * half.y * 2.0;
                middle.y -= extra * 0.5 * up_local.y.signum();
                half.y += extra * 0.5;
            }
        }
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
        // A capsule closes as far past its box as it is thick.
        let cap = if sp.capsules {
            let (a1, a2) = ((long + 1) % 3, (long + 2) % 3);
            half[a1].max(half[a2])
        } else {
            0.0
        };
        lo = lo.min(plo - Vec3::splat(cap));
        hi = hi.max(phi + Vec3::splat(cap));
        // Which end of the long axis the bone's joint is nearer.
        // The joint end is the end nearer the bone's origin (the box is in
        // the bone's frame, and the bone's origin is its joint): a leg hangs
        // down from its joint, a tail runs back from its.
        let root = if middle[long] > 0.0 { -1.0 } else { 1.0 };
        // Which way is up for the contour, in the box's frame: the world's
        // up, pulled into the bone's frame, so a barrel's belly sags down
        // and a leg's bellies sit front and back whichever way its box is
        // written.
        let up_local = to_world
            .inverse()
            .transform_vector3(Vec3::Y)
            .normalize_or(Vec3::Y);
        let fwd_local = to_world
            .inverse()
            .transform_vector3(Vec3::X)
            .normalize_or(Vec3::X);
        let shell = {
            let mut axis = Vec3::ZERO;
            axis[long] = 1.0;
            let bone = &sp.bones[s.bone];
            let side = if bone.side < 0 { -1.0 } else { 1.0 };
            let outward = to_world
                .inverse()
                .transform_vector3(Vec3::Z * side)
                .normalize_or(Vec3::Z);
            let carries_leg = sp.legs.iter().any(|l| sp.bones[l.hip].parent == s.bone);
            let lower_leg = sp.legs.iter().any(|l| l.knee == s.bone);
            let head = i == look.head || bone.name.starts_with("head");
            let neck = bone.name.starts_with("neck");
            crate::carapace::shell_of(
                half,
                long,
                root,
                up_local,
                fwd_local,
                outward,
                to_world.transform_vector3(axis),
                carries_leg,
                lower_leg,
                head,
                neck,
                build.armour,
                build.scute,
                31 + i as u32 * 7,
            )
        };
        lumps.push((
            i,
            Lump {
                to_local: to_world.inverse(),
                middle,
                half,
                long,
                root,
                contour: contour_of(sp, i, &rest),
                up: up_local,
                fwd: fwd_local,
                lo: plo - reach - Vec3::splat(build.armour * 2.0 + cap),
                hi: phi + reach + Vec3::splat(build.armour * 2.0 + cap),
                bone: s.bone,
                shell,
                capsule: sp.capsules,
                rings: {
                    let (wlo, whi) = world_box(sp, &rest, i);
                    let flat_body =
                        crate::anatomy::role(sp, i, wlo, whi) == crate::anatomy::Role::Body;
                    let mut axis = Vec3::ZERO;
                    axis[long] = 1.0;
                    let lying = to_world.transform_vector3(axis).y.abs() < 0.7;
                    if build.chitin > 0.0
                        && flat_body
                        && lying
                        && sp.bones[s.bone].side == 0
                        && i != look.head
                    {
                        (build.chitin, half.min_element() * 0.06)
                    } else {
                        (0.0, 0.0)
                    }
                },
            },
        ));
    }
    // Where the armour parts: round every weak point, which is exactly
    // where it is not (the trench the ridge stands in, the gap in the
    // collars for the nape).
    let weak: Vec<(Vec3, Vec3)> = (0..sp.parts.len())
        .filter(|&i| sp.parts[i].weak)
        .map(|i| {
            let s = sp.shape(i);
            let (a, b) = (fx3(s.min), fx3(s.max));
            let m = sp.frames[i];
            let mut lo = Vec3::splat(f32::MAX);
            let mut hi = Vec3::splat(f32::MIN);
            for c in 0..8 {
                let k = Vec3::new(
                    if c & 1 == 0 { a.x } else { b.x },
                    if c & 2 == 0 { a.y } else { b.y },
                    if c & 4 == 0 { a.z } else { b.z },
                );
                let w = m.transform_point3(k);
                lo = lo.min(w);
                hi = hi.max(w);
            }
            // Down to the skin: a weak strip on the back opens the stone all
            // the way down to the flesh it stands out of.
            (lo - Vec3::new(0.0, 1.0, 0.0), hi)
        })
        .collect();
    let opening = |p: Vec3| {
        let mut open = 1.0f32;
        for (lo, hi) in &weak {
            let d = (*lo - p).max(p - *hi).max(Vec3::ZERO).length();
            open = open.min(smoothstep(0.08, 0.2, d));
        }
        open
    };
    // The sinews: along every bone that carries a part, and from every part
    // to its joint.
    let mut sinews: Vec<Sinew> = Vec::new();
    let thin_on = |bone: usize| {
        lumps
            .iter()
            .filter(|(_, l)| l.bone == bone)
            .map(|(_, l)| l.half.min_element() * 2.0)
            .fold(f32::MAX, f32::min)
    };
    let mut sinew = |a: Vec3, b: Vec3, radius: f32| {
        let r = Vec3::splat(radius + blend * 1.5);
        sinews.push(Sinew {
            a,
            b,
            radius,
            lo: a.min(b) - r,
            hi: a.max(b) + r,
        });
    };
    for (b, def) in sp.bones.iter().enumerate() {
        let thin = thin_on(b);
        if thin == f32::MAX {
            continue;
        }
        let joint = rest[b].transform_point3(Vec3::ZERO);
        if def.parent != sim::beast::NO_PARENT {
            let parent = rest[def.parent].transform_point3(Vec3::ZERO);
            let thin_parent = thin_on(def.parent);
            let radius = thin.min(if thin_parent == f32::MAX {
                thin
            } else {
                thin_parent
            }) * SINEW;
            sinew(joint, parent, radius.max(blend));
        }
        for (_, l) in lumps.iter().filter(|(_, l)| l.bone == b) {
            // The part's middle in its own frame -- which for a species is
            // its bone's, and for a bred part is not.
            let middle = l.to_local.inverse().transform_point3(l.middle);
            sinew(
                joint,
                middle,
                (l.half.min_element() * 2.0 * SINEW).max(blend),
            );
        }
    }

    // **What the boxes do not say** (exploration 0029): toes under each
    // claw, the club the quills fan from, a jaw under the skull, and the soft
    // places as flesh standing out of the skin -- the ridge between the
    // armour's halves, the nape between the collars. Joined more tightly
    // than the parts, so they keep their shape.
    let boxes: Vec<(Vec3, Vec3)> = (0..sp.parts.len())
        .map(|i| world_box(sp, &rest, i))
        .collect();
    let mut extras: Vec<Sinew> = Vec::new();
    let mut crests: Vec<Sinew> = Vec::new();
    // Toes, joined tightly enough that the cleft between two stays a cleft.
    let mut digits: Vec<Sinew> = Vec::new();
    let capsule = |a: Vec3, b: Vec3, radius: f32| {
        let r = Vec3::splat(radius + 0.3);
        Sinew {
            a,
            b,
            radius,
            lo: a.min(b) - r,
            hi: a.max(b) + r,
        }
    };
    // A dog's paw on every leg (`horns::paws`): the palm, and each toe a
    // knuckle that arches up and a pad that comes down to the floor.
    let paws = if build.foot == crate::species::Foot::Paw {
        crate::horns::paws(sp, &boxes, &rest, build.claws, build.girth)
    } else {
        Vec::new()
    };
    // A bird's toes, an insect's tarsus (`crate::anatomy::feet`).
    for (a, b, r) in crate::anatomy::feet(sp, build, &boxes, &rest).0 {
        lo = lo.min(a.min(b) - Vec3::splat(r));
        digits.push(capsule(a, b, r));
    }
    for paw in &paws {
        let k = paw.scale;
        lo = lo.min(boxes[paw.foot].0);
        // The pastern, from the wrist down and forward into the palm.
        extras.push(capsule(
            paw.joint,
            paw.palm + Vec3::new(-0.08, 0.06, 0.0) * k,
            0.24 * k,
        ));
        let across = Vec3::Z * 0.27 * k;
        extras.push(capsule(paw.palm - across, paw.palm + across, 0.25 * k));
        extras.push(capsule(
            paw.palm - Vec3::X * 0.2 * k,
            paw.palm + Vec3::X * 0.22 * k,
            0.25 * k,
        ));
        for toe in &paw.toes {
            let r = toe.size * k;
            digits.push(capsule(
                toe.at - toe.dir * 0.10 * r,
                toe.at + toe.dir * 0.09 * r,
                0.16 * r,
            ));
            let knuckle = toe.at - toe.dir * 0.10 * r + Vec3::Y * 0.11 * k;
            digits.push(capsule(knuckle, knuckle, 0.12 * k));
        }
        if paw.fore {
            let pad = paw.joint + Vec3::new(-0.36, -0.04, 0.0) * k;
            extras.push(capsule(pad, pad, 0.15 * k));
            let dew = paw.joint + Vec3::new(0.06, 0.26, 0.33 * paw.inside) * k;
            extras.push(capsule(dew, dew, 0.1 * k));
        }
    }
    // **The legs' muscle** (exploration 0029: "the legs are straight
    // posts"): the masses the sculpted target's legs have, placed off the
    // leg's two boxes. A foreleg: the deltoid capping it outside, the
    // triceps behind its upper half, the forearm's muscle down the outside
    // of its lower. A hindleg: the thigh's mass in front, the hamstring
    // behind down to the knee, the calf behind the top of the shin and the
    // point of the hock.
    if build.girth > 1.0 {
        let g = build.girth;
        for leg in &sp.legs {
            let (ulo, uhi) = boxes[leg.upper];
            let (flo, fhi) = boxes[leg.foot];
            let (uc, uh) = ((ulo + uhi) * 0.5, (uhi - ulo) * 0.5);
            let (fc, fh) = ((flo + fhi) * 0.5, (fhi - flo) * 0.5);
            let (side, r) = (leg.side as f32, uh.x.max(uh.z) * g);
            let (uy, fy) = (uhi.y - ulo.y, fhi.y - flo.y);
            let at = |c: Vec3, x: f32, y: f32, z: f32| Vec3::new(c.x + x, y, c.z + side * z);
            if leg.front {
                extras.push(capsule(
                    at(uc, 0.1 * r, uhi.y - 0.16 * uy, 0.62 * r),
                    at(uc, 0.05 * r, uhi.y - 0.40 * uy, 0.68 * r),
                    0.48 * r,
                ));
                extras.push(capsule(
                    at(uc, -0.78 * r, uhi.y - 0.2 * uy, 0.05 * r),
                    at(uc, -0.7 * r, uhi.y - 0.62 * uy, 0.12 * r),
                    0.48 * r,
                ));
                extras.push(capsule(
                    at(fc, 0.2 * fh.x * g, fhi.y - 0.06 * fy, 0.7 * fh.z * g),
                    at(fc, 0.1 * fh.x * g, fhi.y - 0.40 * fy, 0.6 * fh.z * g),
                    0.42 * fh.z * g,
                ));
            } else {
                extras.push(capsule(
                    at(uc, 0.7 * r, uhi.y - 0.15 * uy, 0.35 * r),
                    at(uc, 0.55 * r, uhi.y - 0.55 * uy, 0.35 * r),
                    0.5 * r,
                ));
                extras.push(capsule(
                    at(uc, -0.75 * r, uhi.y - 0.2 * uy, 0.2 * r),
                    at(uc, -0.65 * r, ulo.y + 0.15 * uy, 0.15 * r),
                    0.48 * r,
                ));
                extras.push(capsule(
                    at(fc, -0.8 * fh.x * g, fhi.y - 0.1 * fy, 0.0),
                    at(fc, -0.5 * fh.x * g, fhi.y - 0.45 * fy, 0.0),
                    0.45 * fh.x * g,
                ));
            }
        }
    }
    if let Some(spikes) = &look.spikes {
        let (lo, hi) = boxes[spikes.along[1]];
        let half = (hi - lo) * 0.5;
        let y = (lo.y + hi.y) * 0.5;
        extras.push(capsule(
            Vec3::new(lo.x + 0.5, y, 0.0),
            Vec3::new(lo.x + 0.9, y + 0.1, 0.0),
            half.z * 1.4,
        ));
    }
    if build.horns > 0.0 {
        let (lo, hi) = boxes[look.head];
        let (l, h) = (hi.x - lo.x, hi.y - lo.y);
        let half_z = (hi.z - lo.z) * 0.5;
        // A jaw hung under the skull, and the bite's muscle behind the eye.
        extras.push(capsule(
            Vec3::new(lo.x + 0.15 * l, lo.y + 0.24 * h, 0.0),
            Vec3::new(hi.x - 0.08 * l, lo.y + 0.10 * h, 0.0),
            0.22 * h,
        ));
        for side in [-1.0f32, 1.0] {
            let at = Vec3::new(lo.x + 0.18 * l, lo.y + 0.38 * h, side * half_z * 0.85);
            extras.push(capsule(at, at, 0.3 * h));
            // The skull's own shape over the wedge: a heavy brow over each
            // eye, a cheekbone flaring below it.
            extras.push(capsule(
                Vec3::new(lo.x + 0.42 * l, lo.y + 0.86 * h, side * half_z * 0.62),
                Vec3::new(lo.x + 0.68 * l, lo.y + 0.80 * h, side * half_z * 0.55),
                0.13 * h,
            ));
            extras.push(capsule(
                Vec3::new(lo.x + 0.30 * l, lo.y + 0.52 * h, side * half_z * 0.95),
                Vec3::new(lo.x + 0.62 * l, lo.y + 0.50 * h, side * half_z * 0.78),
                0.13 * h,
            ));
        }
        // A ridge down the nose to the boss the nose horn stands on.
        extras.push(capsule(
            Vec3::new(lo.x + 0.62 * l, lo.y + 0.84 * h, 0.0),
            Vec3::new(hi.x - 0.12 * l, lo.y + 0.70 * h, 0.0),
            0.12 * h,
        ));
    }
    for i in (0..sp.parts.len()).filter(|&i| sp.parts[i].weak) {
        let (lo, hi) = boxes[i];
        let half = (hi - lo) * 0.5;
        let y = lo.y + (hi.y - lo.y) * 0.3;
        let r = half.z * 0.8;
        crests.push(capsule(
            Vec3::new(lo.x + r, y, 0.0),
            Vec3::new(hi.x - r, y, 0.0),
            r,
        ));
    }

    // The field, sampled on a grid round the whole body.
    // Room round the parts for everything the field grows past them: the
    // blend, and the armour and the fuller legs.
    // Everything the field holds, not only the boxes: a sinew to a joint
    // outside every box (a bred wing folded) would otherwise run off the
    // grid and be cut open there.
    for c in sinews.iter().chain(&extras).chain(&crests).chain(&digits) {
        lo = lo.min(c.lo);
        hi = hi.max(c.hi);
    }
    let margin = Vec3::splat(blend * 2.0 + build.armour * 3.0 + 0.3);
    let (lo, hi) = (lo - margin, hi + margin);
    let cell = (hi - lo).max_element() / CELLS;
    let n = ((hi - lo) / cell).ceil().as_uvec3() + UVec3::ONE;
    let (nx, ny, nz) = (n.x as usize, n.y as usize, n.z as usize);
    let at = |i: usize, j: usize, k: usize| lo + Vec3::new(i as f32, j as f32, k as f32) * cell;
    let armour_at = |p: Vec3, nearest: usize| -> (f32, f32, f32) {
        let l = &lumps[nearest].1;
        match &l.shell {
            Some(sh) => {
                let q = l.to_local.transform_point3(p) - l.middle;
                let (h, groove, own) = crate::carapace::thickness(sh, q);
                (h * if h > 0.0 { opening(p) } else { 1.0 }, groove, own)
            }
            None => (0.0, 9.0, 0.0),
        }
    };
    // The armour at a point: each nearby part's, weighted by how near the
    // point is to that part -- never one part's alone, which would step from
    // a fifth of a metre to nothing where two parts meet and leave the
    // mesher a cliff it cannot close.
    let armour_blend = |p: Vec3, near: &[(f32, usize)]| -> (f32, f32, f32) {
        let best = near.iter().map(|n| n.0).fold(f32::MAX, f32::min);
        let (mut h, mut g, mut o, mut wsum) = (0.0, 0.0, 0.0, 0.0);
        for &(e, k) in near {
            let w = (-(e - best) / 0.12).exp();
            if w < 0.02 {
                continue;
            }
            let (hk, gk, ok) = armour_at(p, k);
            h += hk * w;
            g += gk.min(1.0) * w;
            o += ok * w;
            wsum += w;
        }
        if wsum == 0.0 {
            (0.0, 9.0, 0.0)
        } else {
            (h / wsum, g / wsum, o / wsum)
        }
    };
    let field = |p: Vec3| {
        let mut d = f32::MAX;
        let mut near: [(f32, usize); 8] = [(f32::MAX, 0); 8];
        let mut n = 0;
        for (k, (_, l)) in lumps.iter().enumerate() {
            if l.near(p) {
                let e = l.distance(p);
                if n < near.len() {
                    near[n] = (e, k);
                    n += 1;
                }
                d = if d == f32::MAX { e } else { smin(d, e, blend) };
            }
        }
        if n > 0 && d.abs() < 0.6 {
            d -= armour_blend(p, &near[..n]).0;
        }
        for s in &sinews {
            if s.near(p) {
                let e = s.distance(p);
                d = if d == f32::MAX { e } else { smin(d, e, blend) };
            }
        }
        for s in extras.iter().chain(&crests) {
            if s.near(p) {
                let e = s.distance(p);
                d = if d == f32::MAX { e } else { smin(d, e, 0.15) };
            }
        }
        for s in &digits {
            if s.near(p) {
                let e = s.distance(p);
                d = if d == f32::MAX { e } else { smin(d, e, 0.05) };
            }
        }
        if d == f32::MAX { cell * 4.0 } else { d }
    };
    // How much a point is a soft place's flesh.
    let softness = |p: Vec3| {
        crests
            .iter()
            .map(|s| smoothstep(0.12, 0.0, s.distance(p)))
            .fold(0.0f32, f32::max)
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

    // **One piece.** Whatever is not joined to the largest body is dropped:
    // a flood fill over the cells inside, six-connected. With the sinews
    // there should be nothing to drop; `dropped` says if there was.
    let mut dropped = 0usize;
    {
        let mut comp = vec![u32::MAX; f.len()];
        let mut sizes: Vec<usize> = Vec::new();
        let mut stack = Vec::new();
        for start in 0..f.len() {
            if f[start] >= 0.0 || comp[start] != u32::MAX {
                continue;
            }
            let c = sizes.len() as u32;
            let mut size = 0;
            comp[start] = c;
            stack.push(start);
            while let Some(v) = stack.pop() {
                size += 1;
                let (i, j, k) = (v % nx, (v / nx) % ny, v / (nx * ny));
                let mut step = |u: usize| {
                    if f[u] < 0.0 && comp[u] == u32::MAX {
                        comp[u] = c;
                        stack.push(u);
                    }
                };
                if i > 0 {
                    step(v - 1)
                }
                if i + 1 < nx {
                    step(v + 1)
                }
                if j > 0 {
                    step(v - nx)
                }
                if j + 1 < ny {
                    step(v + nx)
                }
                if k > 0 {
                    step(v - nx * ny)
                }
                if k + 1 < nz {
                    step(v + nx * ny)
                }
            }
            sizes.push(size);
        }
        if let Some((largest, _)) = sizes.iter().enumerate().max_by_key(|(_, s)| **s) {
            for v in 0..f.len() {
                if comp[v] != u32::MAX && comp[v] != largest as u32 {
                    f[v] = cell;
                    dropped += 1;
                }
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
        // Below a wrist or an ankle, the lower leg's hold passes to the
        // paw's bone, eased across the joint, so the paw turns flat while
        // the leg above it swings (`sim::beast::Bone::paw`).
        for paw in paws.iter().filter(|w| sp.bones[w.bone].paw) {
            let leg = sp.bones[paw.bone].parent;
            let Some(from) = j
                .iter()
                .zip(&w)
                .position(|(b, x)| *b as usize == leg && *x > 0.0)
            else {
                continue;
            };
            let band = 0.12 * paw.scale;
            let off = *p - paw.joint;
            let share = smoothstep(band, -band, off.y)
                * smoothstep(
                    1.6 * paw.scale,
                    1.1 * paw.scale,
                    Vec2::new(off.x, off.z).length(),
                );
            if share <= 0.0 {
                continue;
            }
            let moved = w[from] * share;
            let to = (0..4).min_by(|&a, &b| w[a].total_cmp(&w[b])).unwrap_or(0);
            if to == from {
                if share > 0.5 {
                    j[from] = paw.bone as u16;
                }
                continue;
            }
            w[from] -= moved;
            if w[to] > 0.0 {
                w[from] += w[to];
            }
            j[to] = paw.bone as u16;
            w[to] = moved;
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
    // **What each vertex is** (exploration 0029): stone where the armour
    // stands off the skin, soft where a weak point's flesh is; and what its
    // paint reads -- the groove between scutes, its scute's own number, how
    // open it is to the sky, how near a weak point.
    let near_lump = |p: Vec3| -> [(f32, usize); 8] {
        let mut near: [(f32, usize); 8] = [(f32::MAX, 0); 8];
        let mut n = 0;
        for (k, (_, l)) in lumps.iter().enumerate() {
            if l.near(p) && n < near.len() {
                near[n] = (l.distance(p), k);
                n += 1;
            }
        }
        near
    };
    let mut region = Vec::with_capacity(positions.len());
    let mut stone = Vec::with_capacity(positions.len());
    for (p, nrm) in positions.iter().zip(&normals) {
        let near = near_lump(*p);
        let count = near.iter().filter(|n| n.0 < f32::MAX).count();
        let (h, groove, own) = if count > 0 {
            armour_blend(*p, &near[..count])
        } else {
            (0.0, 9.0, 0.0)
        };
        let soft = softness(*p);
        region.push([smoothstep(0.02, 0.07, h) * (1.0 - soft), soft, 0.0, 0.0]);
        // Occlusion, read off the field itself: how much of the space a
        // little way out along the normal is inside the body.
        let mut occ = 0.0;
        for (step, w) in [(0.15f32, 0.4f32), (0.35, 0.3), (0.7, 0.2), (1.2, 0.1)] {
            occ += w * ((step - field(*p + *nrm * step)).max(0.0) / step);
        }
        let weakness = weak
            .iter()
            .map(|(lo, hi)| (*lo - *p).max(*p - *hi).max(Vec3::ZERO).length())
            .fold(f32::MAX, f32::min);
        stone.push([
            groove.min(1.0),
            own,
            (1.0 - occ * 1.6).clamp(0.0, 1.0),
            smoothstep(0.9, 0.15, weakness),
        ]);
    }

    // **The hard parts**, by rule (`crate::horns`), each bound wholly to
    // one bone.
    let bone_at = |p: Vec3| {
        lumps
            .iter()
            .filter(|(_, l)| l.near(p))
            .map(|(_, l)| (l.distance(p), l.bone))
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|x| x.1)
            .unwrap_or(0)
    };
    let hard = crate::horns::grow(&crate::horns::Plan {
        sp,

        boxes: boxes.clone(),
        field: &field,
        bone_at: &bone_at,
        spines: build.spines,
        horns: build.horns,
        paws: &paws,
        reach: 1.85,
    });
    let roles: Vec<crate::anatomy::Role> = (0..sp.parts.len())
        .map(|i| crate::anatomy::role(sp, i, boxes[i].0, boxes[i].1))
        .collect();
    let mut hard = hard;
    hard.extend(crate::anatomy::grow(&crate::anatomy::Plan {
        sp,
        build,
        look,
        boxes: &boxes,
        roles: &roles,
        rest: &rest,
        field: &field,
    }));
    for hp in &hard {
        let base = positions.len() as u32;
        let kind = hp.kind.code();
        for (((p, n), u), x) in hp
            .positions
            .iter()
            .zip(&hp.normals)
            .zip(&hp.along)
            .zip(&hp.across)
        {
            positions.push(*p);
            normals.push(*n);
            joints.push([hp.bone as u16, 0, 0, 0]);
            weights.push([1.0, 0.0, 0.0, 0.0]);
            paints.push([(hp.part as u16, 1.0), (0, 0.0), (0, 0.0)]);
            shade.push([0.5, 0.0, 0.0, 0.5, 1.0]);
            region.push([0.0, 0.0, kind, *u]);
            // A hard part's second number is where it is across: a vane's
            // shaft and edges.
            stone.push([9.0, *x, 1.0, 0.0]);
        }
        tris.extend(hp.tris.iter().map(|t| t + base));
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
    // Where each vertex is at rest, and what it is there, in the two sets of
    // texture coordinates (which skinning leaves alone) for the skin's
    // detail (`skin.wgsl`): rest x and y; rest z, and one number that still
    // means something after the rasteriser blends three of them -- how much
    // stone (0 to 1), soft flesh (2 to 3), or a hard part (-1). Packing four
    // values into the bits of one did not: halfway along a triangle the low
    // bits were noise, and a horn came out speckled with stone.
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_UV_0,
        positions.iter().map(|p| [p.x, p.y]).collect::<Vec<_>>(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_UV_1,
        positions
            .iter()
            .zip(&region)
            .map(|(p, r)| {
                let what = if r[2] > 0.0 {
                    -1.0
                } else if r[1] > 0.5 {
                    2.0 + r[1]
                } else {
                    r[0]
                };
                [p.z, what]
            })
            .collect::<Vec<_>>(),
    )
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
        dropped,
        own: None,
        region,
        stone,
        build: *build,
        armour: look.armour.rgb,
    })
}

/// The rest frame of every bone of a species, standing at the origin facing
/// `+x`: what a body is bound in, grown or formed.
fn rest_frames(sp: &'static Species) -> Vec<Mat4> {
    let rig = sim::beast::Rig::build(
        sp,
        sim::V3::ZERO,
        sim::Fx::ZERO,
        &sim::beast::Pose::rest(sp.bones.len()),
    );
    (0..sp.bones.len()).map(|b| frame(&rig.bone[b])).collect()
}

/// A body from a formed creature's file: its mesh, carrying a tint, its
/// part in the second set of texture coordinates and its bones' weights,
/// and its material. The part is taken off the mesh once read, so nothing
/// downstream mistakes it for the painted cast's marker (`brushwork`).
pub fn modelled(sp: &'static Species, hide: &Mesh, material: &StandardMaterial) -> Option<Grown> {
    let mut mesh = hide.clone();
    let parts: Vec<u16> = match mesh.remove_attribute(Mesh::ATTRIBUTE_UV_1) {
        Some(VertexAttributeValues::Float32x2(uv)) => {
            uv.iter().map(|p| p[0].round().max(0.0) as u16).collect()
        }
        _ => return None,
    };
    let base = match mesh.attribute(Mesh::ATTRIBUTE_COLOR)? {
        VertexAttributeValues::Float32x4(c) => c.clone(),
        _ => return None,
    };
    if parts.len() != base.len() {
        return None;
    }
    let mut material = material.clone();
    // Hide and stone hardly mirror anything (the cast's lesson: at Bevy's
    // default a dark hide under a bright sky takes the sky's colour).
    material.reflectance = 0.12;
    Some(Grown {
        mesh,
        paints: Vec::new(),
        shade: Vec::new(),
        rest: rest_frames(sp),
        dropped: 0,
        region: Vec::new(),
        stone: Vec::new(),
        build: *crate::species::build(sp.id),
        armour: crate::species::look(sp.id).armour.rgb,
        own: Some(Own {
            base,
            part: parts,
            material,
        }),
    })
}

/// A formed body's colours with what is broken shown: its own paint, gone
/// toward the species' broken colour where its part has broken.
pub fn own_colours(
    own: &Own,
    look: &Look,
    broken: impl Fn(usize) -> bool,
    tinted: impl Fn(usize) -> Option<Paint>,
) -> Vec<[f32; 4]> {
    let linear = |rgb: [f32; 3]| {
        Vec3::from(LinearRgba::from(Color::srgb(rgb[0], rgb[1], rgb[2])).to_f32_array_no_alpha())
    };
    let scar = linear(look.broken.rgb);
    own.base
        .iter()
        .zip(&own.part)
        .map(|(c, &p)| {
            let c3 = Vec3::new(c[0], c[1], c[2]);
            if broken(p as usize) {
                (c3 * 0.35 + scar * 0.65).extend(1.0).to_array()
            } else if let Some(paint) = tinted(p as usize) {
                // Over the body's own paint, so the feathers still read.
                (c3 * 0.45 + linear(paint.rgb) * 0.55)
                    .extend(1.0)
                    .to_array()
            } else {
                *c
            }
        })
        .collect()
}

/// The hide's colours for one body, linear: each vertex the hides of the
/// parts under it (`look::hide`, from the colour `paint` says each part
/// wears now), at its place on the hide; and where it is armour, a weak
/// point's flesh or a hard part, that instead (`look::hide::Carapace`, its
/// heat from `weak`) -- worn pale on its edges, dark in its grooves, moss
/// where it faces the sky, all of it shut in where the body closes round it.
pub fn colours(grown: &Grown, paint: impl Fn(usize) -> Paint, weak: [f32; 3]) -> Vec<[f32; 4]> {
    let mut cache: HashMap<u16, (look::hide::Hide, look::hide::Carapace)> = HashMap::new();
    let mix = |a: [f32; 3], b: [f32; 3], t: f32| -> [f32; 3] {
        let t = t.clamp(0.0, 1.0);
        [
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
        ]
    };
    grown
        .paints
        .iter()
        .zip(&grown.shade)
        .enumerate()
        .map(|(v, (pick, s))| {
            let region = grown.region.get(v).copied().unwrap_or([0.0; 4]);
            let st = grown.stone.get(v).copied().unwrap_or([9.0, 0.0, 1.0, 0.0]);
            let mut c = [0.0f32; 3];
            let mut wsum = 0.0;
            for &(i, w) in pick {
                if w <= 0.0 {
                    continue;
                }
                let (hide, shell) = *cache.entry(i).or_insert_with(|| {
                    let rgb = paint(i as usize).rgb;
                    (
                        look::hide::Hide::of(rgb),
                        look::hide::Carapace::of(rgb, weak),
                    )
                });
                let up = s[0];
                let skin = hide.at(up, s[1], s[2], s[3], 1.0);
                // Stone: each scute its own tint, darker where it faces
                // down, dark in its groove, worn on its edges, mossed on top,
                // charred beside a weak point.
                let mut stone = mix(shell.stone, shell.stone_warm, st[1] * 0.6);
                stone = mix(stone, shell.stone_dark, smoothstep(0.55, 0.1, up) * 0.5);
                stone = mix(
                    stone,
                    shell.groove,
                    (1.0 - smoothstep(0.01, 0.06, st[0])) * 0.85,
                );
                stone = mix(stone, [0.95, 0.92, 0.85], (-s[2]).max(0.0) * 0.35);
                stone = mix(
                    stone,
                    shell.moss,
                    smoothstep(0.7, 0.9, up) * smoothstep(0.45, 0.62, s[3]) * 0.8,
                );
                stone = mix(stone, shell.char, st[3] * 0.55);
                // Feathered: slate above and pale below, the line between
                // them wavering with the mottle -- a raptor's countershading.
                let skin = if grown.build.feathers > 0.0 {
                    let pale = mix(paint(i as usize).rgb, [0.80, 0.77, 0.70], 0.8);
                    mix(
                        skin,
                        pale,
                        smoothstep(0.5, 0.3, up + (s[3] - 0.5) * 0.15) * 0.8,
                    )
                } else {
                    skin
                };
                let flesh = mix(shell.ember_deep, shell.ember, s[3]);
                let mut here = mix(skin, stone, region[0]);
                here = mix(here, flesh, region[1]);
                let u = region[3];
                here = match region[2] as u32 {
                    1 => mix(shell.horn_root, shell.horn_tip, smoothstep(0.05, 0.75, u)),
                    2 => mix(
                        [0.30, 0.25, 0.18],
                        [0.88, 0.84, 0.72],
                        smoothstep(0.0, 0.8, u),
                    ),
                    3 => mix(shell.ember_deep, shell.horn_tip, smoothstep(0.05, 0.6, u)),
                    // A feather: barred across, darker to its tip, a pale
                    // fringe round its edge and a pale shaft.
                    4 => {
                        // The body's own colour, or the part's where that
                        // is darker -- a cracking or broken wing -- but not
                        // a pale band on the skin under them.
                        let own = paint(i as usize).rgb;
                        let body = grown.armour;
                        let lum = |c: [f32; 3]| c[0] + c[1] + c[2];
                        let rgb = if lum(own) < lum(body) { own } else { body };
                        let rgb = [rgb[0] * 0.8, rgb[1] * 0.8, rgb[2] * 0.82];
                        let x = st[1].abs();
                        let bars = smoothstep(
                            0.35,
                            0.65,
                            (u * std::f32::consts::PI * 7.0).sin() * 0.5 + 0.5,
                        );
                        let dark = [rgb[0] * 0.5, rgb[1] * 0.5, rgb[2] * 0.52];
                        let mut f = mix(rgb, dark, bars * 0.55 + smoothstep(0.6, 1.0, u) * 0.35);
                        let pale = mix(rgb, [0.80, 0.77, 0.70], 0.6);
                        f = mix(
                            f,
                            pale,
                            smoothstep(0.94, 1.0, u).max(smoothstep(0.85, 1.0, x)) * 0.35,
                        );
                        mix(f, [0.80, 0.77, 0.70], smoothstep(0.12, 0.0, x) * 0.3)
                    }
                    // A beak: the cere at its base, dark horn, a pale tip.
                    5 => mix(
                        mix(
                            [0.88, 0.70, 0.22],
                            [0.20, 0.20, 0.22],
                            smoothstep(0.0, 0.18, u),
                        ),
                        [0.62, 0.62, 0.58],
                        smoothstep(0.7, 1.0, u),
                    ),
                    // Bone-white, darkening to its point.
                    6 => mix(
                        [0.93, 0.91, 0.80],
                        [0.30, 0.27, 0.22],
                        smoothstep(0.6, 1.0, u) * 0.6,
                    ),
                    // An eye: an insect's dark and glassy; a raptor's amber
                    // round a black pupil looking out.
                    8 => {
                        if grown.build.eyes > 0.0 {
                            [0.16, 0.22, 0.10]
                        } else {
                            mix(
                                mix(
                                    [0.35, 0.18, 0.05],
                                    [1.0, 0.72, 0.18],
                                    smoothstep(0.3, 0.6, u),
                                ),
                                [0.02, 0.02, 0.02],
                                smoothstep(0.86, 0.94, u),
                            )
                        }
                    }
                    9 => [0.10, 0.12, 0.06],
                    _ => here,
                };
                for k in 0..3 {
                    c[k] += here[k] * w;
                }
                wsum += w;
            }
            let ao = 0.5 + 0.5 * st[2];
            let [r, g, b] = [
                c[0] / wsum.max(1e-6) * ao,
                c[1] / wsum.max(1e-6) * ao,
                c[2] / wsum.max(1e-6) * ao,
            ];
            // The palette is written as the eye sees it (sRGB); a vertex
            // colour is linear.
            let lin = LinearRgba::from(Color::srgb(
                r.clamp(0.0, 1.0),
                g.clamp(0.0, 1.0),
                b.clamp(0.0, 1.0),
            ));
            [lin.red, lin.green, lin.blue, 1.0]
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

    /// `GROWN_OBJ=1 cargo test -p game --bin game grown_obj`: the grown
    /// Ridgeback as `target/grown.obj`, to look at outside the game.
    #[test]
    fn grown_obj() {
        let Some(which) = crate::platform::env("GROWN_OBJ") else {
            return;
        };
        // `1` for the Ridgeback, or a species' name.
        let sp = sim::species::all()
            .find(|s| s.name.eq_ignore_ascii_case(&which))
            .unwrap_or(sim::species::SpeciesId::RIDGEBACK.get());
        let g = grow(&Anatomy::of_species(sp)).expect("a body");
        let Some(VertexAttributeValues::Float32x3(p)) = g.mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            return;
        };
        let Some(Indices::U32(t)) = g.mesh.indices() else {
            return;
        };
        let mut out = String::new();
        for v in p {
            out.push_str(&format!("v {} {} {}\n", v[0], v[1], v[2]));
        }
        for f in t.chunks_exact(3) {
            out.push_str(&format!("f {} {} {}\n", f[0] + 1, f[1] + 1, f[2] + 1));
        }
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            if sp.id == sim::species::SpeciesId::RIDGEBACK {
                "../../target/grown.obj".to_string()
            } else {
                format!("../../target/grown_{}.obj", sp.name.to_lowercase())
            },
        );
        crate::platform::write_to(&path, &out, "the grown body");
    }

    #[test]
    fn every_grown_species_is_one_closed_body_bound_to_its_bones() {
        for (id, _) in MODELLED {
            one_body(id.get());
        }
    }

    /// The new rules leave something to see: a feathered body grows its
    /// feathers, a beak and talons; an insect its scythes' serration,
    /// antennae and compound eyes (exploration 0030).
    #[test]
    fn the_galewing_and_the_mantis_grow_what_their_targets_have() {
        use crate::horns::Kind;
        let count = |id: sim::species::SpeciesId, kind: Kind| {
            let g = grow(&Anatomy::of_species(id.get())).expect("a body");
            g.region.iter().filter(|r| r[2] == kind.code()).count()
        };
        let gw = sim::species::SpeciesId::GALEWING;
        for kind in [Kind::Feather, Kind::Beak, Kind::Claw, Kind::Eye] {
            assert!(count(gw, kind) > 0, "the Galewing grows no {kind:?}");
        }
        let mt = sim::species::SpeciesId::MANTIS;
        for kind in [Kind::Bone, Kind::Antenna, Kind::Eye, Kind::Claw] {
            assert!(count(mt, kind) > 0, "the Mantis grows no {kind:?}");
        }
    }

    fn one_body(sp: &'static Species) {
        let g = grow(&Anatomy::of_species(sp)).expect("a body");
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
        assert_eq!(g.dropped, 0, "{}: the body is one piece", sp.name);
    }
}
