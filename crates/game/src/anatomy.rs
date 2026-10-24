//! **What the Galewing and the Mantis taught** (2026-10-24, exploration 0030):
//! the parts of a grown body the Ridgeback never needed, each worked out from
//! the species table the way `crate::horns` works out horns and claws -- so
//! the sculpted targets (`art/galewing/build.py`, `art/mantis/build.py`) are
//! something an algorithm can be measured against, not copied from.
//!
//! - **A part has a role** ([`role`]) read off its box and its bone: a *wing*
//!   is flat and off to one side, a *fan* is flat on the middle line (a
//!   bird's tail), a *blade* is breakable and long and thin (a scythe).
//! - **Feathers** (`Build::feathers`): a wing's flesh is a slab over the front
//!   of its box carrying flight feathers back to the box's trailing edge, a
//!   covert over each one's base; the outermost wing part fans its primaries
//!   from pointing back to pointing out, reaching the box's corner; a fan is
//!   a stub of flesh with its feathers spread to fill the box.
//! - **A beak and a crest** (`Build::beak`, `crest`): the head's flesh is its
//!   back, the beak hooks out of its front, a crest sweeps back off the crown,
//!   and a raptor's eyes sit under the brow.
//! - **Feet** (`Build::foot`): a bird's three toes forward and one back with a
//!   talon on each; an insect's tarsus beaded along the floor.
//! - **An insect** (`Build::chitin`, `grip`, `antennae`, `eyes`): a blade's
//!   cutting edge serrated and its end hooked, the arm that carries it spined
//!   beneath, antennae swept back, compound eyes on the head's top corners.
//!
//! Every length is in the box's own terms, so a bigger box grows a bigger
//! part; and nothing here touches what the simulation collides against.

use bevy::prelude::*;

use crate::horns::{Hard, Kind, meet, piece, vane_piece};
use crate::species::{Build, Foot, Look};
use sim::species::Species;

/// What a part is, beyond its contour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Body,
    /// Flat, off to one side: a wing.
    Wing,
    /// Flat, on the middle line: a bird's tail.
    Fan,
    /// Breakable, long and thin: a scythe.
    Blade,
}

/// A part's role, from its box in the world at rest and its bone.
pub fn role(sp: &Species, part: usize, lo: Vec3, hi: Vec3) -> Role {
    let half = (hi - lo) * 0.5;
    let bone = &sp.bones[sp.parts[part].shape.bone];
    let leg = sp.legs.iter().any(|l| l.upper == part || l.foot == part);
    let thin = half.min_element();
    let long = half.max_element();
    if sp.parts[part].shape.breakable && !leg && long > 6.0 * thin {
        return Role::Blade;
    }
    let flat = half.y < 0.3 * half.x.min(half.z);
    if flat && !leg {
        if bone.side != 0 {
            return Role::Wing;
        }
        if bone.parent != sim::beast::NO_PARENT {
            return Role::Fan;
        }
    }
    Role::Body
}

/// What the rules need: the species, its build and look, every part's box
/// and role, the rest frames, and the body's field to set things onto.
pub struct Plan<'a> {
    pub sp: &'static Species,
    pub build: &'a Build,
    pub look: &'a Look,
    pub boxes: &'a [(Vec3, Vec3)],
    pub roles: &'a [Role],
    pub rest: &'a [Mat4],
    pub field: &'a dyn Fn(Vec3) -> f32,
}

/// How far from `from` along `d` (in the floor's plane) a ray leaves the box
/// `(lo, hi)`.
fn exit(from: Vec3, d: Vec3, lo: Vec3, hi: Vec3) -> f32 {
    let mut t = f32::MAX;
    for (o, v, a, b) in [(from.x, d.x, lo.x, hi.x), (from.z, d.z, lo.z, hi.z)] {
        if v.abs() > 1e-4 {
            let edge = if v > 0.0 { b } else { a };
            t = t.min(((edge - o) / v).max(0.0));
        }
    }
    if t == f32::MAX { 0.0 } else { t }
}

/// One feather from `root` along `d`, `length` long and `width` across:
/// a rounded tip, a slight droop, its vane tilted about its shaft so a row
/// of them shingles.
fn feather(
    root: Vec3,
    d: Vec3,
    length: f32,
    width: f32,
    tilt: f32,
    bone: usize,
    part: usize,
) -> Hard {
    let d = d.normalize_or(Vec3::NEG_X);
    let side = d.cross(Vec3::Y).normalize_or(Vec3::Z);
    let normal = (Vec3::Y + side * tilt).normalize();
    let sag = Vec3::Y * length * 0.03;
    let path = [
        root,
        root + d * length * 0.3 + sag,
        root + d * length * 0.65 + sag,
        root + d * length * 0.9 - sag * 0.4,
        root + d * length - sag,
    ];
    let w = width * 0.5;
    vane_piece(
        &path,
        &[w * 0.55, w, w, w * 0.8, w * 0.12],
        normal,
        0.012 * width.max(0.2) / 0.4,
        0.35,
        bone,
        part,
        Kind::Feather,
    )
}

/// The parts of a side's wing, innermost first.
fn wing_chain(p: &Plan, side: f32) -> Vec<usize> {
    let mut parts: Vec<usize> = (0..p.sp.parts.len())
        .filter(|&i| p.roles[i] == Role::Wing && (p.boxes[i].0.z + p.boxes[i].1.z) * side > 0.0)
        .collect();
    parts.sort_by(|&a, &b| {
        let za = (p.boxes[a].0.z + p.boxes[a].1.z).abs();
        let zb = (p.boxes[b].0.z + p.boxes[b].1.z).abs();
        za.total_cmp(&zb)
    });
    parts
}

/// The share of a feathered wing part's chord its flesh keeps, from the
/// front: the rest is flight feathers.
pub const WING_FLESH: f32 = 0.55;
/// The share of a fan's length its flesh keeps, from the body.
pub const FAN_FLESH: f32 = 0.3;

/// Every hard part these rules call for.
pub fn grow(p: &Plan) -> Vec<Hard> {
    let mut out = Vec::new();
    let b = p.build;
    let sp = p.sp;
    let bone_of = |i: usize| sp.parts[i].shape.bone;
    if b.feathers > 0.0 {
        for side in [-1.0f32, 1.0] {
            let chain = wing_chain(p, side);
            let total: f32 = chain
                .iter()
                .map(|&i| p.boxes[i].1.z - p.boxes[i].0.z)
                .sum::<f32>()
                .max(1e-3);
            let mut gone = 0.0;
            for (k, &i) in chain.iter().enumerate() {
                let (lo, hi) = p.boxes[i];
                let depth = hi.x - lo.x;
                let span = hi.z - lo.z;
                let (zin, zout) = if side < 0.0 {
                    (hi.z, lo.z)
                } else {
                    (lo.z, hi.z)
                };
                let y = (lo.y + hi.y) * 0.5;
                let half_y = (hi.y - lo.y) * 0.5;
                let tip = k + 1 == chain.len();
                if !tip {
                    // Secondaries along its trailing edge, each with a covert
                    // over its base.
                    let spacing = 0.3 * b.feathers;
                    let n = (span / spacing).ceil().max(2.0) as usize;
                    for j in 0..n {
                        let f = (j as f32 + 0.5) / n as f32;
                        let z = zin + (zout - zin) * f;
                        let along = (gone + span * f) / total;
                        let root = Vec3::new(
                            hi.x - (WING_FLESH - 0.1) * depth,
                            y - 0.02 - 0.004 * j as f32,
                            z,
                        );
                        let d = Vec3::new(-1.0, -0.05, side * 0.12 * along);
                        let length = (root.x - lo.x) + 0.05;
                        out.push(feather(root, d, length, spacing * 1.6, 0.12, bone_of(i), i));
                        let cov = root + Vec3::new(0.3 * depth * 0.35, half_y * 0.45, 0.0);
                        out.push(feather(
                            cov,
                            d,
                            length * 0.48,
                            spacing * 1.3,
                            0.1,
                            bone_of(i),
                            i,
                        ));
                    }
                } else {
                    // Primaries off the hand, from pointing back to pointing
                    // out, each reaching the box's edge.
                    let n = 10;
                    for j in 0..n {
                        let f = j as f32 / (n - 1) as f32;
                        let z = zin + (zout - zin) * (0.05 + 0.85 * f);
                        let root = Vec3::new(hi.x - 0.5 * depth, y - 0.03 - 0.005 * j as f32, z);
                        let a = (10.0 + 72.0 * f).to_radians();
                        let d = Vec3::new(-a.cos(), -0.03, side * a.sin());
                        let length = exit(root, d, lo, hi) + 0.1;
                        out.push(feather(
                            root,
                            d,
                            length,
                            0.40 * b.feathers,
                            0.18,
                            bone_of(i),
                            i,
                        ));
                        let cov = root + Vec3::new(0.12, half_y * 0.5, 0.0) - d * 0.05;
                        out.push(feather(
                            cov,
                            d,
                            length * 0.4,
                            0.32 * b.feathers,
                            0.12,
                            bone_of(i),
                            i,
                        ));
                    }
                }
                gone += span;
            }
        }
        // A fan: its feathers spread to fill the box.
        for i in (0..sp.parts.len()).filter(|&i| p.roles[i] == Role::Fan) {
            let (lo, hi) = p.boxes[i];
            let depth = hi.x - lo.x;
            let half_z = (hi.z - lo.z) * 0.5;
            let cz = (hi.z + lo.z) * 0.5;
            let y = (lo.y + hi.y) * 0.5;
            let n = 12;
            for j in 0..n {
                let f = j as f32 / (n - 1) as f32 * 2.0 - 1.0;
                let root = Vec3::new(
                    hi.x - 0.1 * depth,
                    y - 0.004 * (f * 6.0).abs(),
                    cz + f * half_z * 0.3,
                );
                let d = Vec3::new(-1.0, 0.0, f * half_z / depth * 0.75);
                let length = exit(root, d.normalize(), lo, hi) + 0.05;
                out.push(feather(
                    root,
                    d,
                    length,
                    0.40 * b.feathers,
                    0.1 * f,
                    bone_of(i),
                    i,
                ));
            }
        }
    }
    let head = p.look.head;
    let (hlo, hhi) = p.boxes[head];
    let (l, h) = (hhi.x - hlo.x, hhi.y - hlo.y);
    let half_z = (hhi.z - hlo.z) * 0.5;
    let at = |x: f32, y: f32, z: f32| Vec3::new(hlo.x + x * l, hlo.y + y * h, z);
    let hb = bone_of(head);
    if b.beak > 0.0 {
        // The upper mandible hooks down over the lower.
        out.push(piece(
            vec![
                at(0.46, 0.66, 0.0),
                at(0.70, 0.68, 0.0),
                at(0.88, 0.54, 0.0),
                at(0.96, 0.30, 0.0),
                at(0.91, 0.12, 0.0),
            ],
            &[0.26 * h, 0.24 * h, 0.17 * h, 0.08 * h, 0.0],
            2.0,
            0.02,
            1.4,
            Some(Vec3::Y),
            hb,
            head,
            Kind::Beak,
        ));
        out.push(piece(
            vec![
                at(0.48, 0.42, 0.0),
                at(0.66, 0.34, 0.0),
                at(0.80, 0.30, 0.0),
            ],
            &[0.17 * h, 0.11 * h, 0.0],
            1.0,
            0.0,
            0.85,
            Some(Vec3::Y),
            hb,
            head,
            Kind::Beak,
        ));
        // A raptor's eyes, under the brow.
        for s in [-1.0f32, 1.0] {
            let aim = at(0.42, 0.72, 0.0);
            if let Some((q, n)) = meet(p.field, aim + Vec3::Z * s * (half_z + 1.5), aim) {
                let r = 0.12 * h;
                out.push(piece(
                    vec![q - n * r * 0.7, q - n * r * 0.1, q + n * r * 0.4],
                    &[r * 0.9, r, 0.0],
                    0.0,
                    0.0,
                    1.0,
                    None,
                    hb,
                    head,
                    Kind::Eye,
                ));
            }
        }
    }
    if b.crest > 0.0 {
        for (k, (zs, len)) in [
            (-0.4f32, 0.75f32),
            (-0.15, 0.95),
            (0.15, 0.95),
            (0.4, 0.75),
            (0.0, 1.1),
        ]
        .into_iter()
        .enumerate()
        {
            let z = zs * half_z;
            let top = at(0.30 - 0.03 * (k % 2) as f32, 1.5, z);
            let Some((q, _)) = meet(p.field, top, at(0.28, 0.5, z)) else {
                continue;
            };
            let d = Vec3::new(-1.0, 0.42, zs * 0.8);
            let length = len * b.crest * h;
            out.push(feather(
                q - Vec3::Y * 0.03,
                d,
                length,
                0.18 * h,
                0.0,
                hb,
                head,
            ));
        }
    }
    if b.antennae > 0.0 {
        for s in [-1.0f32, 1.0] {
            let root = at(0.85, 0.85, s * 0.3 * half_z);
            let a = b.antennae;
            let path = vec![
                root,
                root + Vec3::new(0.2, 0.25, s * 0.08) * a,
                root + Vec3::new(0.0, 0.62, s * 0.22) * a,
                root + Vec3::new(-0.5, 0.86, s * 0.34) * a,
                root + Vec3::new(-1.0, 0.9, s * 0.42) * a,
            ];
            out.push(piece(
                path,
                &[0.012 * a, 0.009 * a, 0.007 * a, 0.005 * a, 0.002 * a],
                20.0,
                0.15,
                1.0,
                None,
                hb,
                head,
                Kind::Antenna,
            ));
        }
    }
    if b.eyes > 0.0 {
        // Compound eyes: the head's top corners, bulging up and out.
        for s in [-1.0f32, 1.0] {
            let (r, ry) = (b.eyes * 0.6, b.eyes * 0.95);
            let c = Vec3::new(
                hlo.x + 0.45 * l,
                hhi.y - ry * 0.55,
                s * (half_z * 0.8 + r * 0.4),
            );
            out.push(piece(
                vec![
                    c - Vec3::Y * ry,
                    c - Vec3::Y * ry * 0.5,
                    c + Vec3::Z * s * r * 0.15,
                    c + Vec3::Y * ry * 0.5,
                    c + Vec3::Y * ry,
                ],
                &[0.0, r * 0.85, r, r * 0.8, 0.0],
                0.0,
                0.0,
                1.0,
                Some(Vec3::X),
                hb,
                head,
                Kind::Eye,
            ));
        }
    }
    // A blade: its cutting edge serrated, its end hooked; and the arm that
    // carries it spined beneath.
    for i in (0..sp.parts.len()).filter(|&i| p.roles[i] == Role::Blade) {
        let (lo, hi) = p.boxes[i];
        let bh = hi.y - lo.y;
        let cz = (lo.z + hi.z) * 0.5;
        let length = hi.x - lo.x;
        let n = (length / 0.14) as usize;
        for k in 0..n {
            let x = lo.x + 0.1 + k as f32 * 0.14;
            let base = Vec3::new(x, lo.y + 0.3 * bh, cz);
            out.push(piece(
                vec![
                    base,
                    base + Vec3::new(0.02, -0.25, 0.0) * bh,
                    base + Vec3::new(0.05, -0.45, 0.0) * bh,
                ],
                &[0.06 * bh, 0.04 * bh, 0.0],
                1.0,
                0.0,
                1.0,
                None,
                bone_of(i),
                i,
                Kind::Bone,
            ));
        }
        out.push(piece(
            vec![
                Vec3::new(hi.x - 0.2, lo.y + 0.6 * bh, cz),
                Vec3::new(hi.x, lo.y + 0.3 * bh, cz),
                Vec3::new(hi.x + 0.02, lo.y - 0.25 * bh, cz),
                Vec3::new(hi.x - 0.04, lo.y - 0.6 * bh, cz),
            ],
            &[0.18 * bh, 0.14 * bh, 0.07 * bh, 0.0],
            2.0,
            0.02,
            0.5,
            Some(Vec3::Z),
            bone_of(i),
            i,
            Kind::Bone,
        ));
        if b.grip > 0.0 {
            let carrier = sp.bones[bone_of(i)].parent;
            for a in (0..sp.parts.len()).filter(|&a| bone_of(a) == carrier) {
                let (alo, ahi) = p.boxes[a];
                let az = (alo.z + ahi.z) * 0.5;
                let len = ahi.x - alo.x;
                for row in [-1.0f32, 1.0] {
                    for k in 0..9 {
                        let f = k as f32 / 8.0;
                        let x = alo.x + len * (0.25 + 0.7 * f);
                        let reach = b.grip * (0.45 + 0.55 * f);
                        let base = Vec3::new(x, alo.y + 0.02, az + row * 0.05);
                        let d = Vec3::new(0.45, -1.0, row * 0.25).normalize();
                        out.push(piece(
                            vec![base - d * 0.02, base + d * reach * 0.6, base + d * reach],
                            &[0.11 * b.grip, 0.07 * b.grip, 0.0],
                            2.0,
                            0.03,
                            1.0,
                            None,
                            carrier,
                            a,
                            Kind::Bone,
                        ));
                    }
                }
            }
        }
    }
    out.extend(feet(p.sp, p.build, p.boxes, p.rest).1);
    out
}

/// A leg's foot below its second joint: where the joint is, how big the
/// foot is against the targets', and which bone it rides.
fn foot_of(
    sp: &Species,
    boxes: &[(Vec3, Vec3)],
    rest: &[Mat4],
    leg: &sim::beast::Leg,
) -> (Vec3, f32, usize, Vec3, Vec3) {
    let (lo, hi) = boxes[leg.foot];
    let half = (hi - lo) * 0.5;
    let paw = (0..sp.bones.len()).find(|&b| sp.bones[b].paw && sp.bones[b].parent == leg.knee);
    let joint = match paw {
        Some(b) => rest[b].transform_point3(Vec3::ZERO),
        None => Vec3::new((lo.x + hi.x) * 0.5, lo.y + 0.1, (lo.z + hi.z) * 0.5),
    };
    (
        joint,
        half.x.max(half.z),
        paw.unwrap_or(sp.parts[leg.foot].shape.bone),
        lo,
        hi,
    )
}

/// The flesh of a bird's or an insect's feet (capsules: `(a, b, radius)`)
/// and their claws.
pub fn feet(
    sp: &Species,
    build: &Build,
    boxes: &[(Vec3, Vec3)],
    rest: &[Mat4],
) -> (Vec<(Vec3, Vec3, f32)>, Vec<Hard>) {
    let mut flesh = Vec::new();
    let mut claws = Vec::new();
    match build.foot {
        Foot::Paw => {}
        Foot::Talon => {
            for leg in sp.legs {
                let (joint, half, bone, lo, _) = foot_of(sp, boxes, rest, leg);
                let s = half / 0.45;
                let palm = Vec3::new(joint.x, lo.y + 0.12 * s, joint.z);
                flesh.push((joint, palm, 0.14 * s));
                for (deg, len) in [
                    (-32.0f32, 0.62f32),
                    (0.0, 0.70),
                    (32.0, 0.60),
                    (180.0, 0.48),
                ] {
                    let a = deg.to_radians();
                    let d = Vec3::new(a.cos(), 0.0, a.sin());
                    let tip = palm + d * len * s - Vec3::Y * 0.04 * s;
                    flesh.push((palm, tip, 0.08 * s));
                    let big = if deg == 180.0 { 1.25 } else { 1.0 } * s;
                    claws.push(piece(
                        vec![
                            tip - d * 0.08 * s + Vec3::Y * 0.05 * s,
                            tip + d * 0.14 * big + Vec3::Y * 0.07 * s,
                            tip + d * 0.30 * big,
                            tip + d * 0.36 * big - Vec3::Y * 0.12 * big,
                            tip + d * 0.32 * big - Vec3::Y * 0.22 * big,
                        ],
                        &[0.075 * big, 0.065 * big, 0.045 * big, 0.022 * big, 0.0],
                        3.0,
                        0.02,
                        1.35,
                        Some(Vec3::Y),
                        bone,
                        leg.foot,
                        Kind::Claw,
                    ));
                }
            }
        }
        Foot::Tarsus => {
            for leg in sp.legs {
                let (joint, half, bone, _, _) = foot_of(sp, boxes, rest, leg);
                let s = half / 0.07;
                let d = Vec3::new(0.55, 0.0, leg.side as f32 * 0.84).normalize();
                let base = joint;
                // A core through the beads, so the foot is one thing.
                flesh.push((base, base + d * 0.26 * s, 0.017 * s));
                for k in 0..5 {
                    let q = base + d * (0.04 + 0.055 * k as f32) * s;
                    flesh.push((q, q, (0.03 - 0.003 * k as f32) * s));
                }
                let tip = base + d * 0.27 * s;
                let across = d.cross(Vec3::Y);
                for side in [-1.0f32, 1.0] {
                    let o = across * 0.012 * side * s;
                    claws.push(piece(
                        vec![
                            tip + o,
                            tip + o + d * 0.04 * s,
                            tip + o + (d * 0.06 - Vec3::Y * 0.03) * s,
                        ],
                        &[0.008 * s, 0.005 * s, 0.0],
                        1.0,
                        0.0,
                        1.0,
                        None,
                        bone,
                        leg.foot,
                        Kind::Claw,
                    ));
                }
            }
        }
    }
    (flesh, claws)
}
