//! **The grown giants, drawn** (`sim::valley::trees`): every segment of a
//! tree's skeleton a tapered tube as thick as the pipe model says, every
//! leaf clump a few rough blobs, and its pads -- the shelves up a Sentinel,
//! the roof over a broadleaf -- as what they are: one mesh of wood and one of
//! leaves a tree.
//!
//! What a body stands on is the simulation's boxes, built from the same
//! skeleton (`trees::solids`) and hidden from the renderer
//! (`Atlas::hidden`); what is drawn here is the skeleton itself. So the two
//! agree because they are one thing, not because a table was kept in step.
//!
//! **What bears you looks it** (the owner, 2026-10-16: "it needs to be
//! easier to tell where you can stand ... foliage either apparently not
//! standable or has collision"). A limb that is stood on
//! ([`trees::stood_on`], the same rule the boxes are built by) is worn pale
//! on top; one that is not stays bark all round. Leaves hang -- tufts and
//! rounded clumps, nothing flat that reads as a ledge -- except where they
//! are a floor: a roof's terraces and its lip, drawn flat-topped and lighter.
//!
//! Colours are `look::Palette`'s; nothing here decides one.

use bevy::prelude::*;
use look::Palette;
use sim::valley::trees::{self, Kind, NONE, Pad, PadKind, Part, Tree};

use crate::forms::Kit;
use crate::shapes::hash01;

/// RENDER-ONLY. Centimetres to metres.
fn m(v: i32) -> f32 {
    v as f32 / 100.0
}

/// A colour a little off its own.
fn vary(rgb: [f32; 3], k: f32, by: f32) -> [f32; 3] {
    let f = 1.0 + by * (k - 0.5) * 2.0;
    [rgb[0] * f, rgb[1] * f, rgb[2] * f]
}

/// How many faces round a branch of radius `r` metres: a trunk twenty
/// metres across is round, a twig is a square stick.
fn sides(r: f32) -> usize {
    if r >= 2.0 {
        16
    } else if r >= 0.6 {
        9
    } else if r >= 0.18 {
        6
    } else {
        4
    }
}

/// How far a tree reaches from its foot, flat, in metres: what the stream
/// loads it by.
pub fn reach(tree: &Tree) -> f32 {
    tree.nodes
        .iter()
        .map(|n| {
            let (dx, dz) = (m(n.at[0] - tree.base[0]), m(n.at[2] - tree.base[2]));
            (dx * dx + dz * dz).sqrt()
        })
        .fold(0.0, f32::max)
}

/// **The box a tree fills**, about its foot, in metres: every node with its
/// thickness, every clump of leaves with its size, every pad with its reach.
/// What its level is judged by: the distance to the nearest of it.
pub fn bounds(tree: &Tree) -> (Vec3, Vec3) {
    let rel = |p: [i32; 3]| {
        Vec3::new(
            m(p[0] - tree.base[0]),
            m(p[1] - tree.base[1]),
            m(p[2] - tree.base[2]),
        )
    };
    let mut lo = Vec3::ZERO;
    let mut hi = Vec3::ZERO;
    let mut grow = |c: Vec3, r: f32| {
        lo = lo.min(c - r);
        hi = hi.max(c + r);
    };
    for n in &tree.nodes {
        grow(rel(n.at), m(n.radius));
    }
    for l in &tree.leaves {
        grow(rel([l[0], l[1], l[2]]), m(l[3]) * 1.5);
    }
    for pad in &tree.pads {
        grow(rel(pad.at), m(pad.outer));
    }
    (lo, hi)
}

/// How many faces round a branch of radius `r` drawn where nothing finer
/// than `detail` metres shows: the fewest whose flats stand no more than half
/// of it inside the true round, and never more than [`sides`] gives up close.
fn sides_at(r: f32, detail: f32) -> usize {
    let full = sides(r);
    if detail <= 0.0 {
        return full;
    }
    [4, 6, 9, 16]
        .into_iter()
        .find(|&n| n >= full || r * (1.0 - (std::f32::consts::PI / n as f32).cos()) <= detail)
        .unwrap_or(full)
        .min(full)
}

/// The same for a ring of `n` segments of radius `r` -- a shelf, a terrace --
/// never fewer than eight, so a ring stays a ring.
fn ring_at(n: usize, r: f32, detail: f32) -> usize {
    if detail <= 0.0 {
        return n;
    }
    let mut k = 8;
    while k < n && r * (1.0 - (std::f32::consts::PI / k as f32).cos()) > detail {
        k *= 2;
    }
    k.min(n)
}

/// **How far apart the levels of a giant are**: the distances, from the
/// nearest part of it, at which it is drawn with less. Up close everything;
/// past [`LEVELS`]`[0]` what is finer than [`detail_at`] that distance shows;
/// past `[1]` the same at that one. A hard switch (`docs/design/exploration/
/// 0024_distant_detail.md`, LOD-1): what goes at a step is finer than the
/// screen shows there, so the step is not seen.
pub const LEVELS: [f32; 3] = [100.0, 250.0, 450.0];

/// The vertical field of view the levels are judged at, radians: the
/// camera's own default (`view::camera`), on a screen 1080 lines tall.
const FOV: f32 = 58.0 * std::f32::consts::PI / 180.0;
const LINES: f32 = 1080.0;

/// **The finest thing worth drawing at `distance` metres**, in metres. In the
/// plain look, a pixel. In a painted one, the paint's own brush (`look::Wash`
/// `brush`, its radius in pixels at 760 lines): anything thinner is painted
/// out of the picture by the filter anyway (LOD-2: the owner, 2026-10-21).
pub fn detail_at(distance: f32, brush_px: f32) -> f32 {
    let pixel = FOV / LINES;
    let brush = brush_px * FOV / 760.0;
    distance * pixel.max(brush)
}

/// **A tree's two meshes**, about its foot: its wood (with its shelves), and
/// its leaves (with its roof), with nothing finer than `detail` metres: a
/// branch thinner is left to its parent -- which is as thick as all it bears,
/// by the pipe model the skeleton is grown by, so the silhouette keeps its
/// weight -- rounds are drawn with fewer faces, and leaf clumps closer together
/// than a few times it are one clump of the same area. Zero is everything, as
/// close up.
pub fn meshes(tree: &Tree, palette: &Palette, detail: f32) -> (Mesh, Mesh) {
    let at = |p: [i32; 3]| {
        Vec3::new(
            m(p[0] - tree.base[0]),
            m(p[1] - tree.base[1]),
            m(p[2] - tree.base[2]),
        )
    };
    let bark = palette.grown_bark(tree.kind);
    let footing = palette.grown_footing(tree.kind);
    let moss = palette.moss();
    let mut wood = Kit::new();
    let runs = runs(tree, detail);
    for (i, n) in tree.nodes.iter().enumerate() {
        let Some((from, first)) = runs[i] else {
            continue;
        };
        let p = tree.nodes[from];
        let rb = m(n.radius);
        // A branch leaves the trunk at its own thickness, a little swollen
        // where it joins -- not at the trunk's.
        let ra = if p.part == Part::Trunk && n.part != Part::Trunk {
            m(tree.nodes[first].radius) * 1.2
        } else {
            m(p.radius)
        };
        if ra.max(rb) < 0.03 || 2.0 * ra.max(rb) < detail {
            continue;
        }
        let k = hash01(i as u32, tree.base[0] as u32, 31);
        let rgb = match n.part {
            // Roots and props in the wet: mossed toward the ground.
            Part::Root => look::tint::mix(vary(bark, k, 0.12), moss, 0.35),
            Part::Prop => look::tint::mix(vary(bark, k, 0.12), moss, 0.15),
            // What does not bear you, darker: it is not a path.
            Part::Twig => vary(bark, k, 0.2),
            _ => vary(bark, k, 0.1),
        };
        // What bears you, worn pale on top; the trunk is climbed by its
        // shelves and its limbs, not stood on.
        let top = if trees::stood_on(tree, i) && n.part != Part::Trunk {
            vary(footing, k, 0.06)
        } else {
            rgb
        };
        let s = sides_at(ra.max(rb), detail);
        // Capped where it is thick: a joint that bends opens a sliver,
        // and through an open tube the sky shows.
        let caps = (ra.max(rb) > 0.25).then_some(rgb);
        wood.tube_topped(at(p.at), at(n.at), ra, rb, s, (rgb, top), caps);
    }
    // Dark at the foot, where the light does not reach.
    wood.settle(0.0, 8.0, 0.35);

    let green = palette.grown_leaves(tree.kind);
    let mut leaves = Kit::new();
    for pad in &tree.pads {
        let c = at(pad.at);
        match pad.kind {
            PadKind::Shelf => shelf(&mut wood, c, pad, palette.shelf(), bark, detail),
            _ => terrace(
                &mut leaves,
                c,
                pad,
                palette.canopy(tree.kind),
                green,
                detail,
            ),
        }
    }
    let mut clumps = Vec::new();
    for (i, l) in tree.leaves.iter().enumerate() {
        let c = at([l[0], l[1], l[2]]);
        let r = m(l[3]);
        let seed = i as u32 ^ (tree.base[2] as u32).wrapping_mul(7919);
        // Thinned: a crown you can see through to the limbs inside it --
        // the first pass's, every clump drawn, was a wall (2026-10-16).
        let keep = match tree.kind {
            Kind::Sentinel => 0.75,
            _ => 0.6,
        };
        if hash01(seed, 2, 5) > keep {
            continue;
        }
        clumps.push((c, r, seed, vary(green, hash01(seed, 1, 5), 0.14)));
    }
    for (c, r, seed, rgb) in gathered(clumps, detail * CLUMPED) {
        match tree.kind {
            // Needles: a tuft hanging from the twig's end, taller than it is
            // wide -- a spray that droops, not a plate to land on.
            Kind::Sentinel => {
                leaves.nugget(
                    c - Vec3::Y * r * 0.35,
                    Vec3::new(r * 1.2, r * 1.1, r * 1.2),
                    seed,
                    rgb,
                );
            }
            // A broadleaf's clump: two rounded lumps round its end.
            _ => {
                for j in 0..2u32 {
                    let a = hash01(seed, j + 10, 5) * std::f32::consts::TAU;
                    let off = Vec3::new(a.cos(), -0.25, a.sin()) * r * 0.45;
                    let s = r * (1.0 + 0.3 * hash01(seed, j + 20, 5));
                    leaves.nugget(
                        c + off,
                        Vec3::new(s * 1.2, s * 1.1, s * 1.2),
                        seed ^ j,
                        vary(rgb, hash01(seed, j + 30, 5), 0.1),
                    );
                }
            }
        }
    }
    (wood.build(), leaves.build())
}

/// **Where each node's piece of wood starts**: for a node drawn as the end of
/// a tube, the node the tube starts at and the first node past it; `None`
/// for a node a longer tube passes through. With no `detail` every node is
/// its own piece, from its parent. With some, a run of nodes along one limb
/// -- one child each, the same part, borne on or not alike -- is drawn as
/// one tube for as long as every node it passes stands within half of
/// `detail` of the straight line, thickness and all: a bend too small to see
/// is one piece, and one piece is a tenth of the triangles of ten.
fn runs(tree: &Tree, detail: f32) -> Vec<Option<(usize, usize)>> {
    let nodes = &tree.nodes;
    let mut out: Vec<Option<(usize, usize)>> = nodes
        .iter()
        .map(|n| (n.parent != NONE).then_some((n.parent as usize, 0)))
        .collect();
    for (i, slot) in out.iter_mut().enumerate() {
        if let Some((_, first)) = slot {
            *first = i;
        }
    }
    if detail <= 0.0 {
        return out;
    }
    let mut children = vec![Vec::new(); nodes.len()];
    for (i, n) in nodes.iter().enumerate() {
        if n.parent != NONE {
            children[n.parent as usize].push(i);
        }
    }
    // A node a tube may pass through: one child, which is the same part and
    // borne on as it is.
    let passes = |j: usize| {
        children[j].len() == 1 && nodes[j].parent != NONE && {
            let c = children[j][0];
            nodes[c].part == nodes[j].part && trees::stood_on(tree, c) == trees::stood_on(tree, j)
        }
    };
    let at = |j: usize| Vec3::new(m(nodes[j].at[0]), m(nodes[j].at[1]), m(nodes[j].at[2]));
    let r = |j: usize| m(nodes[j].radius);
    // Every node passed between `from` and `to` near enough the line.
    let straight = |from: usize, path: &[usize], to: usize| {
        let (a, b) = (at(from), at(to));
        let (ra, rb) = (r(from), r(to));
        let len = (b - a).length().max(1e-4);
        let mut along = 0.0;
        let mut last = a;
        path.iter().all(|&j| {
            let p = at(j);
            along += (p - last).length();
            last = p;
            let t = (along / len).clamp(0.0, 1.0);
            let off = (p - a) - (b - a) * ((p - a).dot(b - a) / (len * len));
            off.length() <= detail && (r(j) - (ra + (rb - ra) * t)).abs() <= detail
        })
    };
    for start in 0..nodes.len() {
        // Runs start where a tube may not pass: at a fork, a root, a change.
        if passes(start) {
            continue;
        }
        for &first in &children[start] {
            let mut end = first;
            let mut path: Vec<usize> = Vec::new();
            while passes(end) {
                let next = children[end][0];
                path.push(end);
                if !straight(start, &path, next) {
                    path.pop();
                    break;
                }
                out[end] = None;
                end = next;
            }
            // The run so far, from its start; and on, if it bent, from
            // where it bent.
            out[end] = Some((start, first));
            let mut from = end;
            while passes(from) {
                let mut to = children[from][0];
                let begin = to;
                let mut path: Vec<usize> = Vec::new();
                while passes(to) {
                    let next = children[to][0];
                    path.push(to);
                    if !straight(from, &path, next) {
                        path.pop();
                        break;
                    }
                    out[to] = None;
                    to = next;
                }
                out[to] = Some((from, begin));
                from = to;
            }
        }
    }
    out
}

/// How far apart, in [`meshes`]' `detail`, two clumps of leaves may be and
/// still be drawn as one: a gap of a few pixels between two clumps is not a
/// gap anybody sees.
const CLUMPED: f32 = 4.0;

/// A clump of leaves: where, how big, its seed and its colour.
type Clump = (Vec3, f32, u32, [f32; 3]);

/// **Clumps closer than `cell` drawn as one**: every clump in one cube of a
/// grid `cell` metres across gathered into a clump at their middle, of the
/// same area seen from afar (the radius the root of the sum of the squares),
/// in their colour. A `cell` of zero gathers nothing.
fn gathered(clumps: Vec<Clump>, cell: f32) -> Vec<Clump> {
    if cell <= 0.0 {
        return clumps;
    }
    // In order of the first clump in each cube, so the mesh is the same
    // every time it is built.
    let mut cubes: Vec<(IVec3, Vec3, f32, u32, [f32; 3])> = Vec::new();
    let mut index: std::collections::HashMap<IVec3, usize> = Default::default();
    for (c, r, seed, rgb) in clumps {
        let key = (c / cell).floor().as_ivec3();
        let a = r * r;
        let k = *index.entry(key).or_insert_with(|| {
            cubes.push((key, Vec3::ZERO, 0.0, seed, [0.0; 3]));
            cubes.len() - 1
        });
        let cube = &mut cubes[k];
        cube.1 += c * a;
        cube.2 += a;
        for (sum, v) in cube.4.iter_mut().zip(rgb) {
            *sum += v * a;
        }
    }
    cubes
        .into_iter()
        .map(|(_, at, area, seed, rgb)| (at / area, area.sqrt(), seed, rgb.map(|v| v / area)))
        .collect()
}

/// Points round a circle of radius `r` at height `y` about `c`.
fn round(c: Vec3, r: f32, y: f32, k: usize, n: usize) -> Vec3 {
    let a = std::f32::consts::TAU * k as f32 / n as f32;
    Vec3::new(c.x + a.cos() * r, y, c.z + a.sin() * r)
}

/// **A shelf fungus**: a bracket, its flat top where feet go, its edge
/// rounded down and its underside the bark's, banded the way a bracket's
/// growth rings are.
fn shelf(kit: &mut Kit, top: Vec3, pad: &Pad, cream: [f32; 3], bark: [f32; 3], detail: f32) {
    let r = m(pad.outer);
    let deep = m(pad.thick);
    let n = ring_at(24, r, detail);
    let under = look::tint::mix(cream, bark, 0.55);
    for k in 0..n {
        let band = if k % 2 == 0 { 1.0 } else { 0.93 };
        let rim = [cream[0] * band, cream[1] * band, cream[2] * band];
        let (a0, a1) = (
            round(top, r * 0.88, top.y, k, n),
            round(top, r * 0.88, top.y, k + 1, n),
        );
        let (b0, b1) = (
            round(top, r, top.y - deep * 0.4, k, n),
            round(top, r, top.y - deep * 0.4, k + 1, n),
        );
        let (c0, c1) = (
            round(top, r * 0.9, top.y - deep, k, n),
            round(top, r * 0.9, top.y - deep, k + 1, n),
        );
        kit.poly(&[top, a0, a1], Vec3::Y, cream);
        let out = ((b0 + b1) * 0.5 - top).with_y(0.3);
        kit.poly(&[a0, b0, b1, a1], out, rim);
        kit.poly(&[b0, c0, c1, b1], out.with_y(-0.6), under);
        kit.poly(&[c0, top.with_y(top.y - deep), c1], -Vec3::Y, under);
    }
}

/// **A terrace of a roof**: a flat ring of leaves to walk on, its edge a
/// fringe of leaf hanging to its underside. The lip stands up from it the
/// same way, a darker green.
fn terrace(kit: &mut Kit, top: Vec3, pad: &Pad, floor: [f32; 3], green: [f32; 3], detail: f32) {
    let (ri, ro) = (m(pad.inner), m(pad.outer));
    let deep = m(pad.thick);
    let n = ring_at(((ro * 1.2) as usize).clamp(16, 96), ro, detail);
    let face = if pad.kind == PadKind::Rim {
        look::tint::mix(floor, green, 0.5)
    } else {
        floor
    };
    for k in 0..n {
        let shade = vary(face, hash01(k as u32, pad.outer as u32, 9), 0.08);
        let (o0, o1) = (round(top, ro, top.y, k, n), round(top, ro, top.y, k + 1, n));
        if ri > 0.0 {
            let (i0, i1) = (round(top, ri, top.y, k, n), round(top, ri, top.y, k + 1, n));
            kit.poly(&[i0, o0, o1, i1], Vec3::Y, shade);
            // The step down to the terrace inside shows its side.
            let (j0, j1) = (i0.with_y(top.y - 0.5), i1.with_y(top.y - 0.5));
            kit.poly(
                &[i0, i1, j1, j0],
                (top - (i0 + i1) * 0.5).with_y(0.0),
                green,
            );
        } else {
            kit.poly(&[top, o0, o1], Vec3::Y, shade);
        }
        // Its underside, leaves seen from below: a ceiling of the crown's
        // own green, not the floor's.
        let (u0, u1) = (o0.with_y(top.y - deep), o1.with_y(top.y - deep));
        let under = vary(green, 0.3, 0.1);
        if ri > 0.0 {
            let (v0, v1) = (
                round(top, ri, top.y - deep, k, n),
                round(top, ri, top.y - deep, k + 1, n),
            );
            kit.poly(&[v0, v1, u1, u0], -Vec3::Y, under);
        } else {
            kit.poly(&[top.with_y(top.y - deep), u1, u0], -Vec3::Y, under);
        }
        // The fringe down its outside.
        let (d0, d1) = (
            round(top, ro * 0.97, top.y - deep, k, n),
            round(top, ro * 0.97, top.y - deep, k + 1, n),
        );
        let out = ((o0 + o1) * 0.5 - top).with_y(0.0);
        kit.poly(
            &[o0, d0, d1, o1],
            out,
            vary(green, hash01(k as u32, 3, 9), 0.1),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangles(m: &Mesh) -> usize {
        m.indices().map_or(0, |i| i.len() / 3)
    }

    /// **Every giant at every level**: how much each far level keeps, in the
    /// plain look and the painted ones' finest and coarsest brushes. Printed
    /// with `--nocapture`; what is asserted is that each level is a real
    /// saving and keeps the tree's frame.
    #[test]
    fn every_level_is_a_saving() {
        let atlas = sim::atlas::valley();
        let palette = look::palette::of(sim::arena::ArenaId::WOOD);
        for brush in [0.0, 2.0, 7.0] {
            let mut sum = [0usize; 4];
            for tree in &atlas.trees {
                let mut n = [0usize; 4];
                for (k, d) in [0.0, LEVELS[0], LEVELS[1], LEVELS[2]]
                    .into_iter()
                    .enumerate()
                {
                    let detail = if k == 0 { 0.0 } else { detail_at(d, brush) };
                    let (wood, leaves) = meshes(tree, &palette, detail);
                    n[k] = triangles(&wood) + triangles(&leaves);
                    // The trunk and the big limbs are always there.
                    assert!(triangles(&wood) > 100, "a far giant with no wood");
                }
                assert!(
                    n[1] < n[0] && n[2] < n[1] && n[3] < n[2],
                    "levels {n:?} are not each a saving"
                );
                for k in 0..4 {
                    sum[k] += n[k];
                }
            }
            eprintln!(
                "brush {brush} px: {} / {} / {} / {} triangles for the {} giants",
                sum[0],
                sum[1],
                sum[2],
                sum[3],
                atlas.trees.len()
            );
        }
    }
}
