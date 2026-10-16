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
use sim::valley::Zone;
use sim::valley::trees::{self, Kind, NONE, Pad, PadKind, Part, Tree};

use crate::forms::Kit;
use crate::shapes::hash01;

/// RENDER-ONLY. Centimetres to metres.
fn m(v: i32) -> f32 {
    v as f32 / 100.0
}

/// RENDER-ONLY. Fixed point to metres.
fn fx(v: sim::Fx) -> f32 {
    v.to_f32_for_render()
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

/// **A tree's two meshes**, about its foot: its wood (with its shelves and
/// the lianas off it, of `vines`), and its leaves (with its roof).
pub fn meshes(tree: &Tree, palette: &Palette, vines: &[Zone]) -> (Mesh, Mesh) {
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
    for (i, n) in tree.nodes.iter().enumerate() {
        if n.parent == NONE {
            continue;
        }
        let p = tree.nodes[n.parent as usize];
        let rb = m(n.radius);
        // A branch leaves the trunk at its own thickness, a little swollen
        // where it joins -- not at the trunk's.
        let ra = if p.part == Part::Trunk && n.part != Part::Trunk {
            rb * 1.2
        } else {
            m(p.radius)
        };
        if ra.max(rb) < 0.03 {
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
        let s = sides(ra.max(rb));
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
            PadKind::Shelf => shelf(&mut wood, c, pad, palette.shelf(), bark),
            _ => terrace(&mut leaves, c, pad, palette.canopy(tree.kind), green),
        }
    }
    // The lianas hung off its roof.
    if let Some(rim) = tree.pads.iter().find(|p| p.kind == PadKind::Rim) {
        let mid = Vec2::new(m(rim.at[0]), m(rim.at[2]));
        for v in vines {
            let (lo, hi) = (
                Vec3::new(fx(v.min.x), fx(v.min.y), fx(v.min.z)),
                Vec3::new(fx(v.max.x), fx(v.max.y), fx(v.max.z)),
            );
            let foot = Vec2::new((lo.x + hi.x) * 0.5, (lo.z + hi.z) * 0.5);
            if foot.distance(mid) > m(rim.outer) + 4.0 {
                continue;
            }
            let base = Vec3::new(m(tree.base[0]), m(tree.base[1]), m(tree.base[2]));
            liana(&mut wood, lo - base, hi - base, palette.vine());
        }
    }

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
        let rgb = vary(green, hash01(seed, 1, 5), 0.14);
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

/// Points round a circle of radius `r` at height `y` about `c`.
fn round(c: Vec3, r: f32, y: f32, k: usize, n: usize) -> Vec3 {
    let a = std::f32::consts::TAU * k as f32 / n as f32;
    Vec3::new(c.x + a.cos() * r, y, c.z + a.sin() * r)
}

/// **A shelf fungus**: a bracket, its flat top where feet go, its edge
/// rounded down and its underside the bark's, banded the way a bracket's
/// growth rings are.
fn shelf(kit: &mut Kit, top: Vec3, pad: &Pad, cream: [f32; 3], bark: [f32; 3]) {
    let r = m(pad.outer);
    let deep = m(pad.thick);
    let n = 24;
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
fn terrace(kit: &mut Kit, top: Vec3, pad: &Pad, floor: [f32; 3], green: [f32; 3]) {
    let (ri, ro) = (m(pad.inner), m(pad.outer));
    let deep = m(pad.thick);
    let n = ((ro * 1.2) as usize).clamp(16, 96);
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

/// **A liana**: a few strands hung down the zone a body climbs, from its top
/// to the ground.
fn liana(kit: &mut Kit, lo: Vec3, hi: Vec3, rgb: [f32; 3]) {
    let c = (lo + hi) * 0.5;
    let foot = lo.y + 2.0;
    for k in 0..4 {
        let a = k as f32 * 1.7;
        let off = Vec3::new(a.cos(), 0.0, a.sin()) * 0.45;
        let mut y = hi.y;
        let mut was = Vec3::new(c.x, y, c.z) + off;
        while y > foot {
            let step = 6.0f32.min(y - foot);
            y -= step;
            let sway = (y * 0.21 + k as f32).sin() * 0.25;
            let p = Vec3::new(c.x + off.x + sway, y, c.z + off.z - sway);
            kit.tube(was, p, 0.09, 0.09, 4, rgb, None);
            was = p;
        }
    }
}
