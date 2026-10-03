//! **Roles**: what a bone is for, so that a pose authored on one skeleton can
//! be read onto another. Today the only thing that says what a bone is is its
//! name and its index; this is the least a transplant needs, worked out from
//! names and from the leg table, and it is a guess -- which is the finding.

#![allow(clippy::needless_range_loop)]

use crate::{Body, f, rest_pose, rig_of};
use sim::beast::NO_PARENT;
use sim::species::Species;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Root,
    Trunk,
    Neck,
    Head,
    Jaw,
    Tail,
    LegUpper,
    LegLower,
    Arm,
    Wing,
    Other,
}

#[derive(Clone, Copy, Debug)]
pub struct Role {
    pub kind: Kind,
    pub side: i32,
    /// Where along its chain, nought at the base to one at the tip; for a
    /// leg, nought at the front pair to one at the back.
    pub pos: f32,
}

fn descends(s: &Species, mut b: usize, from: usize) -> bool {
    loop {
        if b == from {
            return true;
        }
        if s.bones[b].parent == NO_PARENT {
            return false;
        }
        b = s.bones[b].parent;
    }
}

pub fn roles(s: &'static Species) -> Vec<Role> {
    let n = s.bones.len();
    let rig = rig_of(s, &rest_pose(s));
    let mut out = vec![
        Role {
            kind: Kind::Other,
            side: 0,
            pos: 0.0
        };
        n
    ];
    // Legs first: their bones are named anything (a shoulder is a leg on a
    // quadruped and a wing on a bird), and the leg table is the one place that
    // says which they are.
    let mut leg_x: Vec<(usize, f32)> = s
        .legs
        .iter()
        .enumerate()
        .map(|(i, l)| (i, f(rig.bone[l.hip].at.x)))
        .collect();
    leg_x.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let pairs = s.legs.len().div_ceil(2).max(1);
    for (rank, (i, _)) in leg_x.iter().enumerate() {
        let leg = s.legs[*i];
        let pos = if pairs > 1 {
            (rank / 2) as f32 / (pairs - 1) as f32
        } else {
            1.0
        };
        for b in 0..n {
            let kind =
                if b == leg.hip || (descends(s, leg.hip, b) && b != 0 && s.bones[b].side != 0) {
                    Some(Kind::LegUpper)
                } else if descends(s, b, leg.knee) || (descends(s, b, leg.hip) && b != leg.hip) {
                    Some(Kind::LegLower)
                } else {
                    None
                };
            if let Some(kind) = kind {
                out[b] = Role {
                    kind,
                    side: leg.side,
                    pos,
                };
            }
        }
    }
    for b in 0..n {
        if out[b].kind != Kind::Other {
            continue;
        }
        let name = s.bones[b].name;
        let base = name.split('.').next().unwrap_or(name);
        let kind = if s.bones[b].parent == NO_PARENT {
            Kind::Root
        } else if [
            "spine", "chest", "abdomen", "thorax", "pedicel", "crown", "shell",
        ]
        .contains(&base)
        {
            Kind::Trunk
        } else if base.starts_with("neck") || base == "throat" {
            Kind::Neck
        } else if base == "head" {
            Kind::Head
        } else if ["jaw", "fangs", "lip", "tongue"].contains(&base) {
            Kind::Jaw
        } else if base.starts_with("tail") {
            Kind::Tail
        } else if base.starts_with("wing") {
            Kind::Wing
        } else if ["arm", "blade", "hand", "shoulder"].contains(&base) {
            Kind::Arm
        } else {
            Kind::Other
        };
        out[b].kind = kind;
        out[b].side = s.bones[b].side;
    }
    // Position along each chain: the order of bones of that kind and side.
    for kind in [
        Kind::Trunk,
        Kind::Neck,
        Kind::Tail,
        Kind::Arm,
        Kind::Wing,
        Kind::Jaw,
    ] {
        for side in [-1, 0, 1] {
            let chain: Vec<usize> = (0..n)
                .filter(|b| out[*b].kind == kind && out[*b].side == side)
                .collect();
            for (i, b) in chain.iter().enumerate() {
                out[*b].pos = if chain.len() > 1 {
                    i as f32 / (chain.len() - 1) as f32
                } else {
                    0.0
                };
            }
        }
    }
    out
}

/// For each bone of `to`, the bone of `from` that plays the same role, if
/// any: same kind, same side, nearest place along its chain.
pub fn map(from: &'static Species, to: &'static Species) -> Vec<Option<usize>> {
    let (a, b) = (roles(from), roles(to));
    b.iter()
        .map(|r| {
            if r.kind == Kind::Other {
                return None;
            }
            a.iter()
                .enumerate()
                .filter(|(_, d)| d.kind == r.kind && d.side == r.side)
                .min_by(|x, y| {
                    (x.1.pos - r.pos)
                        .abs()
                        .partial_cmp(&(y.1.pos - r.pos).abs())
                        .unwrap()
                })
                .map(|(i, _)| i)
        })
        .collect()
}

/// **A donor's clip, read onto a recipient's skeleton by role**: each of the
/// recipient's bones takes the donor's angles for the bone that plays its
/// role, and holds its own standing angles where nothing does. The hips'
/// shove is scaled by the ratio of hip heights.
pub fn transplant_rows(
    from: &'static Species,
    clip: usize,
    to: &Body,
    to_sp: &'static Species,
) -> Vec<&'static [i32]> {
    let m = map(from, to_sp);
    let (start, count) = from.span[clip];
    let ratio = hip(to_sp) / hip(from);
    let stand = (to_sp.row)(to_sp.span[to_sp.stock.idle].0 as usize);
    (0..count as usize)
        .map(|r| {
            let src = (from.row)(start as usize + r);
            let mut row = vec![0i32; 3 + 3 * to.bones.len()];
            for c in 0..3 {
                row[c] = (src[c] as f32 * ratio).round() as i32;
            }
            for (b, d) in m.iter().enumerate() {
                for c in 0..3 {
                    row[3 + 3 * b + c] = match d {
                        Some(d) => src[3 + 3 * d + c],
                        None => stand[3 + 3 * b + c],
                    };
                }
            }
            crate::slice(row)
        })
        .collect()
}

/// The hips' height at rest, in metres at its scale.
pub fn hip(s: &'static Species) -> f32 {
    let root = s
        .bones
        .iter()
        .position(|b| b.parent == NO_PARENT)
        .unwrap_or(0);
    f(s.rest(root).y)
}
