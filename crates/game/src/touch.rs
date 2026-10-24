//! **A crossbreed touched by hand** (2026-10-24, exploration 0032): the
//! owner's *"take each of the intermediate steps, hand edit them to make each
//! a reasonable creature in its own right"*.
//!
//! A stage of a line (`crate::bred::lines`) is its genome's body as the
//! breeder builds it (`sim::breed::build`). A hand edit is a short list of
//! [`Edit`]s on that body -- this part thicker, that limb longer, these hips
//! brought in under the body -- applied before it is grown. Written as data
//! rather than sculpted, so the edits across every stage are a record of
//! what the breeder gets wrong, which is what the breeder is then changed by
//! (`docs/design/exploration/0032_hand_edited_crossbreeds.md`).

use bevy::prelude::*;
use sim::breed::Genome;
use sim::breed::build::{self, Body};
use sim::{Fx, V3};

use crate::contour::Contour;
use crate::grown::Anatomy;
use crate::species::Build;

/// One hand edit. A name matches every part (or bone) whose name starts
/// with it: `"hind"` is both hind legs, all three segments.
#[derive(Clone, Copy, Debug)]
pub enum Edit {
    /// Parts scaled about their own middles: along, across, up.
    Size(&'static str, f32, f32, f32),
    /// Parts moved, in the body's frame (metres).
    Move(&'static str, f32, f32, f32),
    /// A bone moved, and everything it carries with it.
    Bone(&'static str, f32, f32, f32),
    /// Parts turned to run along a new direction (body frame), about their
    /// bone's joint.
    Aim(&'static str, f32, f32, f32),
    /// A limb pair (`"fore"`, `"hind"`, `"midfore"`, `"midhind"`) longer or
    /// shorter below the hip: its joints and its segments together.
    Limb(&'static str, f32),
    /// A limb pair posed: its three segments laid along these directions
    /// (body frame, the left's; the right is its mirror), each as long as
    /// it was. A mantis's arms folded in front of it rather than hanging.
    Fold(&'static str, [f32; 3], [f32; 3], [f32; 3]),
    /// A limb pair's hips brought toward the middle line (below one) or out.
    Stance(&'static str, f32),
    /// Flat parts (a wing's membrane) made deeper from the edge at their
    /// bone to the trailing one, which moves: the wing's chord.
    Chord(&'static str, f32),
    /// Parts gone.
    Drop(&'static str),
    /// Parts grown in a contour of their own.
    Shape(&'static str, Contour),
    /// The growing's knobs changed.
    Knobs(fn(&mut Build)),
}

fn fx(x: f32) -> Fx {
    Fx::from_raw((x * 65536.0) as i32)
}

fn v(x: f32, y: f32, z: f32) -> V3 {
    V3::new(fx(x), fx(y), fx(z))
}

fn f3(p: V3) -> Vec3 {
    Vec3::new(
        p.x.to_f32_for_render(),
        p.y.to_f32_for_render(),
        p.z.to_f32_for_render(),
    )
}

/// A world vector in a part's own (along, across, up) frame.
fn local(body: &Body, i: usize, w: Vec3) -> Vec3 {
    let u = f3(body.axes[i]);
    let (a, b) = build::frame(body.axes[i]);
    Vec3::new(w.dot(u), w.dot(f3(a)), w.dot(f3(b)))
}

fn named(body: &Body, name: &str) -> Vec<usize> {
    (0..body.part_count)
        .filter(|&i| is(body.parts[i].name, name))
        .collect()
}

/// A part's name answers to a name: by its start, or `"torso"`.
fn is(part: &str, name: &str) -> bool {
    if name == "torso" {
        TORSO.contains(&part)
    } else {
        part.starts_with(name)
    }
}

fn bones_where(body: &Body, f: impl Fn(&str) -> bool) -> Vec<usize> {
    (0..body.bone_count)
        .filter(|&b| f(body.bones[b].name))
        .collect()
}

/// Apply the body's edits, and stand it back on the floor unless it flies.
pub fn edit_body(body: &mut Body, edits: &[Edit], grounded: bool) {
    let parts = body.part_count;
    for e in edits {
        match *e {
            Edit::Size(name, a, c, u) => {
                for i in named(body, name) {
                    let s = &mut body.parts[i].shape;
                    let (lo, hi) = (f3(s.min), f3(s.max));
                    let (mid, half) = ((lo + hi) * 0.5, (hi - lo) * 0.5 * Vec3::new(a, c, u));
                    s.min = v(mid.x - half.x, mid.y - half.y, mid.z - half.z);
                    s.max = v(mid.x + half.x, mid.y + half.y, mid.z + half.z);
                }
            }
            Edit::Move(name, x, y, z) => {
                for i in named(body, name) {
                    let d = local(body, i, Vec3::new(x, y, z));
                    let s = &mut body.parts[i].shape;
                    s.min = v(f3(s.min).x + d.x, f3(s.min).y + d.y, f3(s.min).z + d.z);
                    s.max = v(f3(s.max).x + d.x, f3(s.max).y + d.y, f3(s.max).z + d.z);
                }
            }
            Edit::Aim(name, x, y, z) => {
                for i in named(body, name) {
                    body.axes[i] = build::unit(v(x, y, z));
                }
            }
            Edit::Bone(name, x, y, z) => {
                for b in bones_where(body, |n| n.starts_with(name)) {
                    let r = f3(body.bones[b].rest) + Vec3::new(x, y, z);
                    body.bones[b].rest = v(r.x, r.y, r.z);
                }
            }
            Edit::Limb(station, k) => {
                let prefix = |n: &str| {
                    n.starts_with(station)
                        && n.as_bytes().get(station.len()).is_some_and(|c| *c == b'.')
                };
                // The hip stays where it is; every joint below it moves.
                for b in bones_where(body, |n| {
                    prefix(n) && (n.ends_with(".lower") || n.ends_with(".foot"))
                }) {
                    {
                        let r = f3(body.bones[b].rest) * k;
                        body.bones[b].rest = v(r.x, r.y, r.z);
                    }
                }
                let along: Vec<usize> = (0..parts)
                    .filter(|&i| {
                        prefix(body.bones[body.parts[i].shape.bone].name)
                            && !body.parts[i].name.ends_with(".foot")
                    })
                    .collect();
                for i in along {
                    let s = &mut body.parts[i].shape;
                    s.min.x = s.min.x.mul(fx(k));
                    s.max.x = s.max.x.mul(fx(k));
                }
            }
            Edit::Fold(station, upper, lower, foot) => {
                for side in [".l", ".r"] {
                    let m = if side == ".r" { -1.0 } else { 1.0 };
                    let dir = |d: [f32; 3]| Vec3::new(d[0], d[1], d[2] * m).normalize();
                    let at = |s: &str| {
                        let name = format!("{station}{side}{s}");
                        (0..body.bone_count).find(|&b| body.bones[b].name == name)
                    };
                    let (Some(hip), Some(knee), Some(ankle)) = (at(""), at(".lower"), at(".foot"))
                    else {
                        continue;
                    };
                    let len = |b: usize| f3(body.bones[b].rest).length();
                    let (lu, ll) = (len(knee), len(ankle));
                    let r = dir(upper) * lu;
                    body.bones[knee].rest = v(r.x, r.y, r.z);
                    let r = dir(lower) * ll;
                    body.bones[ankle].rest = v(r.x, r.y, r.z);
                    for (bone, d) in [(hip, upper), (knee, lower), (ankle, foot)] {
                        let d = dir(d);
                        for i in (0..parts).filter(|&i| body.parts[i].shape.bone == bone) {
                            body.axes[i] = v(d.x, d.y, d.z);
                        }
                    }
                }
            }
            Edit::Stance(station, k) => {
                for b in bones_where(body, |n| {
                    n.starts_with(station) && (n.ends_with(".l") || n.ends_with(".r"))
                }) {
                    let r = body.bones[b].rest;
                    body.bones[b].rest = V3::new(r.x, r.y, r.z.mul(fx(k)));
                }
            }
            Edit::Chord(name, k) => {
                for i in named(body, name) {
                    let s = &mut body.parts[i].shape;
                    let (lo, hi) = (f3(s.min), f3(s.max));
                    let h = hi - lo;
                    if h.z * 3.0 > h.y {
                        continue;
                    }
                    // The edge nearer the bone stays.
                    if lo.y.abs() < hi.y.abs() {
                        s.max.y = fx(lo.y + h.y * k);
                    } else {
                        s.min.y = fx(hi.y - h.y * k);
                    }
                }
            }
            Edit::Drop(name) => {
                for i in named(body, name) {
                    body.parts[i].shape.hollow = true;
                }
            }
            Edit::Shape(..) | Edit::Knobs(_) => {}
        }
    }
    if grounded {
        // Back on the floor: the lowest corner of anything solid at nought.
        let low = (0..parts)
            .filter(|&i| !body.parts[i].shape.hollow)
            .flat_map(|i| body.corners(i))
            .map(|c| c.y.to_f32_for_render())
            .fold(f32::MAX, f32::min);
        if low.is_finite() && low != f32::MAX {
            let r = f3(body.bones[0].rest);
            body.bones[0].rest = v(r.x, r.y - low, r.z);
        }
    }
}

/// **The head carried level** (exploration 0032): the breeder lays the head
/// along the neck's line, where a bite is aimed by it, so a raised neck held
/// its head up like a chimney -- the one edit every hand-edited stage made.
/// Grown, the head and what hangs off it are turned level, nose a little
/// down, and set ahead of the joint by half the head's length.
pub fn carry_head(g: &Genome, body: &mut Body) {
    if g.head == sim::breed::HeadKind::Merged {
        return;
    }
    let Some(head) = (0..body.part_count).find(|&i| body.parts[i].name == "head") else {
        return;
    };
    let s = body.parts[head].shape;
    let ahead = (s.max.x.to_f32_for_render() - s.min.x.to_f32_for_render()) * 0.25;
    let along = f3(body.axes[head]);
    let flat = Vec3::new(along.x, 0.0, along.z).normalize_or(Vec3::X);
    let level = (flat - Vec3::Y * 0.15).normalize();
    for i in 0..body.part_count {
        if ["head", "jaw", "crest", "horns"].contains(&body.parts[i].name) {
            body.axes[i] = v(level.x, level.y, level.z);
            let s = &mut body.parts[i].shape;
            s.min.x = s.min.x.add(fx(ahead));
            s.max.x = s.max.x.add(fx(ahead));
        }
    }
}

/// Apply the edits that are the growing's, not the body's.
pub fn edit_anatomy(a: &mut Anatomy, edits: &[Edit]) {
    for e in edits {
        match *e {
            Edit::Shape(name, c) => {
                for i in 0..a.parts.len() {
                    if is(a.parts[i].name, name) {
                        a.shapes[i] = Some(c);
                    }
                }
            }
            Edit::Knobs(f) => f(&mut a.build),
            _ => {}
        }
    }
}

/// A genome's anatomy with a stage's hand edits.
pub fn touched(g: &Genome, edits: &[Edit]) -> Anatomy {
    let q = sim::breed::posture::posture(g);
    let flies = g.has_wings();
    let mut body = if flies {
        build::body_flying(g, &q, Fx::ZERO)
    } else {
        build::body_in(g, &q)
    };
    carry_head(g, &mut body);
    edit_body(&mut body, edits, !flies);
    let mut a = crate::bred::anatomy_of(g, &body);
    edit_anatomy(&mut a, edits);
    a
}

/// The torso's boxes, which a name of `"torso"` means.
const TORSO: [&str; 5] = ["haunch", "barrel", "shoulders", "segment", "rump"];

// What is left of the hand edits now the breeder makes the rest of them
// (exploration 0032). The first pass, against the breeder before it was
// changed, was six groups -- the torso deepened, the wing's chord and arm,
// the head carried level, the neck a stalk -- and is in this file's history
// and the note's pictures; what the breeder and `carry_head` now do is gone
// from here, and what stays is what they still get wrong.

/// A heavy four-legged body: on legs no longer than it is deep, a leg's
/// thickness rather than a post's, under it rather than splayed.
const HEAVY: &[Edit] = &[
    Edit::Size("torso", 1.0, 0.8, 1.0),
    Edit::Limb("fore", 0.85),
    Edit::Limb("hind", 0.85),
    Edit::Stance("hind", 0.6),
    Edit::Stance("fore", 0.7),
    Edit::Size("hind.l", 1.0, 0.85, 0.85),
    Edit::Size("hind.r", 1.0, 0.85, 0.85),
    Edit::Size("fore.l", 1.0, 0.85, 0.85),
    Edit::Size("fore.r", 1.0, 0.85, 0.85),
];

/// Feathered, a bird's share of its own span in the body and head.
const FEATHERED: &[Edit] = &[
    Edit::Size("torso", 1.1, 1.1, 1.1),
    Edit::Size("head", 1.4, 1.3, 1.3),
    Edit::Size("jaw", 1.4, 1.3, 1.3),
];

/// Raptorial arms a mantis's length, held folded in front of it: up from
/// the shoulder, the forearm forward and down, the scythe folded back up
/// under it. The breeder keeps its arms long because their sweep is how its
/// strikes are read (`tests/motion.rs`); this is the look that costs.
const SCYTHES: &[Edit] = &[
    Edit::Limb("fore", 0.6),
    Edit::Fold(
        "fore",
        [0.45, 0.85, -0.2],
        [0.8, -0.6, -0.05],
        [-0.45, 0.89, 0.0],
    ),
];

/// A level body hung with stilts and scythes: something to hang them from.
const STILTS: &[Edit] = &[Edit::Size("torso", 1.3, 1.4, 1.2), Edit::Limb("hind", 0.8)];

/// Every stage's hand edits: a line's name, the stages they are true of,
/// and the edits. A stage gets every entry that names it, in order; a stage
/// none names is the breeder's body as it stands.
pub const TOUCHES: &[(&str, &[usize], &[Edit])] = &[
    ("ridgeback_to_galewing", &[0, 1], HEAVY),
    (
        "ridgeback_to_galewing",
        &[5, 6, 7, 8, 9, 10, 11, 12],
        FEATHERED,
    ),
    ("galewing_to_mantis", &[0, 1], FEATHERED),
    (
        "galewing_to_mantis",
        &[3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        SCYTHES,
    ),
    ("galewing_to_mantis", &[3, 4], STILTS),
    ("mantis_to_ridgeback", &[0, 1, 2, 3, 4], SCYTHES),
    ("mantis_to_ridgeback", &[5, 6, 7, 8, 9, 10, 11, 12], HEAVY),
    ("ridgeback_x_mantis", &[0], HEAVY),
    ("ridgeback_x_mantis", &[4, 5, 6, 7, 8], SCYTHES),
];

/// The hand edits of one stage of one line.
pub fn touches(line: &str, stage: usize) -> Vec<Edit> {
    TOUCHES
        .iter()
        .filter(|(l, k, _)| *l == line && k.contains(&stage))
        .flat_map(|(_, _, e)| e.iter().copied())
        .collect()
}
