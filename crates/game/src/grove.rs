//! **The grown giants, drawn** (`sim::valley::trees`): every segment of a
//! tree's skeleton a tapered tube as thick as the pipe model says, and every
//! leaf clump a few rough blobs -- one mesh of wood and one of leaves a tree.
//!
//! What a body stands on is the simulation's boxes, built from the same
//! skeleton (`trees::solids`) and hidden from the renderer
//! (`Atlas::hidden`); what is drawn here is the skeleton itself. So the two
//! agree because they are one thing, not because a table was kept in step.
//!
//! Colours are `look::Palette`'s ([`look::Palette::grown_bark`],
//! [`look::Palette::grown_leaves`]); nothing here decides one.

use bevy::prelude::*;
use look::Palette;
use sim::valley::trees::{Kind, NONE, Part, Tree};

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

/// **A tree's two meshes**, about its foot: its wood, and its leaves.
pub fn meshes(tree: &Tree, palette: &Palette) -> (Mesh, Mesh) {
    let at = |p: [i32; 3]| {
        Vec3::new(
            m(p[0] - tree.base[0]),
            m(p[1] - tree.base[1]),
            m(p[2] - tree.base[2]),
        )
    };
    let bark = palette.grown_bark(tree.kind);
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
            Part::Twig => vary(bark, k, 0.2),
            _ => vary(bark, k, 0.1),
        };
        wood.tube(at(p.at), at(n.at), ra, rb, sides(ra.max(rb)), rgb, None);
    }
    // Dark at the foot, where the light does not reach.
    wood.settle(0.0, 8.0, 0.35);

    let green = palette.grown_leaves(tree.kind);
    let mut leaves = Kit::new();
    for (i, l) in tree.leaves.iter().enumerate() {
        let c = at([l[0], l[1], l[2]]);
        let r = m(l[3]);
        let seed = i as u32 ^ (tree.base[2] as u32).wrapping_mul(7919);
        let rgb = vary(green, hash01(seed, 1, 5), 0.14);
        match tree.kind {
            // Needles: flat sprays, layered.
            Kind::Sentinel => {
                leaves.nugget(c, Vec3::new(r * 2.2, r * 0.7, r * 2.2), seed, rgb);
                leaves.nugget(
                    c + Vec3::new(0.0, r * 0.5, 0.0),
                    Vec3::new(r * 1.5, r * 0.5, r * 1.5),
                    seed ^ 1,
                    vary(rgb, 0.7, 0.1),
                );
            }
            // A broadleaf's clump: a few lumps round its end.
            _ => {
                for j in 0..3u32 {
                    let a = hash01(seed, j + 10, 5) * std::f32::consts::TAU;
                    let off = Vec3::new(a.cos(), 0.3 * j as f32 - 0.2, a.sin()) * r * 0.55;
                    let s = r * (1.3 + 0.5 * hash01(seed, j + 20, 5));
                    leaves.nugget(
                        c + off,
                        Vec3::new(s * 1.6, s * 1.1, s * 1.6),
                        seed ^ j,
                        vary(rgb, hash01(seed, j + 30, 5), 0.1),
                    );
                }
            }
        }
    }
    (wood.build(), leaves.build())
}
