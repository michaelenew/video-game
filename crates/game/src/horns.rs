//! **A grown body's hard parts** (2026-10-23; exploration 0029): horns,
//! spines, claws and quills, placed by rule from the species table the way
//! the sculpted Ridgeback placed them by hand -- silhouette is geometry, and
//! geometry with rules a table can state:
//!
//! - **spines** stand in two rows either side of a weak strip down the back,
//!   tallest toward the shoulders, and carry on down the tail, shrinking;
//! - **a collar** of spikes flares where the armour ends over the part the
//!   forelegs hang from;
//! - **horns** sweep back off the skull and curl up and forward, with nubs
//!   along each brow, a horn on the nose and tusks out of the jaw;
//! - **claws** go round the front of every foot's pad;
//! - **quills** fan out of the end of the tail that throws them.
//!
//! Each is a tube with growth ridges, set onto the body where a ray meets
//! it, bound wholly to one bone so it moves as one hard thing.

use bevy::prelude::*;

/// What a hard part is, for its paint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Horn,
    Spine,
    Claw,
    Quill,
}

/// One hard part: its mesh in the world at rest, how far along it each
/// vertex is (0 at its root, 1 at its tip), and the bone and part it is on.
pub struct Hard {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub tris: Vec<u32>,
    pub along: Vec<f32>,
    pub bone: usize,
    pub part: usize,
    pub kind: Kind,
}

/// A point on a Catmull-Rom curve through `p` at `u` (0 to 1).
fn spline(p: &[Vec3], u: f32) -> Vec3 {
    let n = p.len();
    if n == 1 {
        return p[0];
    }
    let x = u.clamp(0.0, 1.0) * (n - 1) as f32;
    let i = (x.floor() as usize).min(n - 2);
    let t = x - i as f32;
    let p0 = if i == 0 { p[0] * 2.0 - p[1] } else { p[i - 1] };
    let (p1, p2) = (p[i], p[i + 1]);
    let p3 = if i + 2 < n {
        p[i + 2]
    } else {
        p[n - 1] * 2.0 - p[n - 2]
    };
    let (t2, t3) = (t * t, t * t * t);
    0.5 * (2.0 * p1
        + (p2 - p0) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (3.0 * p1 - p0 - 3.0 * p2 + p3) * t3)
}

/// A horn, spine or claw: a tube along `path` whose radius runs through
/// `radii`, banded with `ridges` growth rings, `flat` squashing it along
/// `ref_` (the world axis least along it, if none).
pub fn tube(
    path: &[Vec3],
    radii: &[f32],
    ridges: f32,
    depth: f32,
    flat: f32,
    ref_: Option<Vec3>,
) -> (Vec<Vec3>, Vec<Vec3>, Vec<u32>, Vec<f32>) {
    let length: f32 = path.windows(2).map(|w| w[0].distance(w[1])).sum();
    let rings = ((length / 0.05) as usize).clamp(8, 40);
    let biggest = radii.iter().cloned().fold(0.0, f32::max);
    let sides = ((biggest / 0.014) as usize).clamp(6, 16);
    let mean = (path[path.len() - 1] - path[0]).normalize_or(Vec3::Y);
    let reference = ref_.unwrap_or_else(|| {
        if mean.y.abs() < 0.7 {
            Vec3::Y
        } else if mean.x.abs() < mean.z.abs() {
            Vec3::X
        } else {
            Vec3::Z
        }
    });
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut along = Vec::new();
    for r in 0..rings {
        let u = r as f32 / (rings - 1) as f32;
        let c = spline(path, u);
        let tg = (spline(path, (u + 0.01).min(1.0)) - spline(path, (u - 0.01).max(0.0)))
            .normalize_or(mean);
        let up = (reference - tg * reference.dot(tg)).normalize_or(Vec3::Y);
        let side = tg.cross(up);
        let k = u * (radii.len() - 1) as f32;
        let i = (k.floor() as usize).min(radii.len() - 2);
        let rad = radii[i] + (radii[i + 1] - radii[i]) * (k - i as f32);
        let band = 1.0
            + depth
                * ((u * std::f32::consts::PI * ridges).sin().abs().sqrt() - 0.5)
                * smooth(0.0, 0.1, u)
                * smooth(1.0, 0.75, u);
        for s in 0..sides {
            let a = s as f32 / sides as f32 * std::f32::consts::TAU;
            let dir = up * a.cos() * flat + side * a.sin();
            positions.push(c + dir * rad * band);
            normals.push((up * a.cos() / flat.max(0.2) + side * a.sin()).normalize_or(up));
            along.push(u);
        }
    }
    let mut tris = Vec::new();
    for r in 0..rings - 1 {
        let (a, b) = ((r * sides) as u32, ((r + 1) * sides) as u32);
        for s in 0..sides as u32 {
            let t = (s + 1) % sides as u32;
            tris.extend([a + s, b + s, b + t, a + s, b + t, a + t]);
        }
    }
    // Close the root with a fan (the tip already meets at a point).
    let root = positions.len() as u32;
    positions.push(spline(path, 0.0));
    normals.push(-mean);
    along.push(0.0);
    for s in 0..sides as u32 {
        tris.extend([root, (s + 1) % sides as u32, s]);
    }
    (positions, normals, tris, along)
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Where a ray from `from` toward `to` meets the body (`field` negative
/// inside), by sphere tracing; and the body's outward normal there.
pub fn meet(field: &dyn Fn(Vec3) -> f32, from: Vec3, to: Vec3) -> Option<(Vec3, Vec3)> {
    let dir = (to - from).normalize_or(Vec3::NEG_Y);
    let limit = from.distance(to) + 2.0;
    let mut t = 0.0;
    for _ in 0..200 {
        let p = from + dir * t;
        let d = field(p);
        if d < 0.004 {
            let h = 0.02;
            let n = Vec3::new(
                field(p + Vec3::X * h) - field(p - Vec3::X * h),
                field(p + Vec3::Y * h) - field(p - Vec3::Y * h),
                field(p + Vec3::Z * h) - field(p - Vec3::Z * h),
            )
            .normalize_or(-dir);
            return Some((p, n));
        }
        t += d.max(0.01) * 0.8;
        if t > limit {
            return None;
        }
    }
    None
}

/// A horn-like piece rooted at `root` (on the body, sunk a little into it).
#[allow(clippy::too_many_arguments)]
fn piece(
    path: Vec<Vec3>,
    radii: &[f32],
    ridges: f32,
    depth: f32,
    flat: f32,
    ref_: Option<Vec3>,
    bone: usize,
    part: usize,
    kind: Kind,
) -> Hard {
    let (positions, normals, tris, along) = tube(&path, radii, ridges, depth, flat, ref_);
    Hard {
        positions,
        normals,
        tris,
        along,
        bone,
        part,
        kind,
    }
}

/// What a body needs to say where its hard parts go.
pub struct Plan<'a> {
    pub sp: &'static sim::species::Species,

    /// Every part's box in the world at rest, as `(lo, hi)`.
    pub boxes: Vec<(Vec3, Vec3)>,
    pub field: &'a dyn Fn(Vec3) -> f32,
    /// The bone of the body part nearest a point.
    pub bone_at: &'a dyn Fn(Vec3) -> usize,
    pub spines: f32,
    pub horns: f32,
    pub claws: u8,
    /// How far round a part's top its armour reaches, for the collar.
    pub reach: f32,
}

/// Every hard part the table calls for.
pub fn grow(plan: &Plan) -> Vec<Hard> {
    let sp = plan.sp;
    let look = crate::species::look(sp.id);
    let field = plan.field;
    let mut out = Vec::new();
    let centre_of = |i: usize| (plan.boxes[i].0 + plan.boxes[i].1) * 0.5;
    let half_of = |i: usize| (plan.boxes[i].1 - plan.boxes[i].0) * 0.5;

    // -- Spines: two rows flanking a weak strip along the back.
    if plan.spines > 0.0 {
        let mount: Vec<usize> = (0..sp.parts.len())
            .filter(|&i| sp.parts[i].shape.mountable)
            .collect();
        let top_strip = (0..sp.parts.len()).find(|&i| {
            sp.parts[i].weak && {
                let (c, h) = (centre_of(i), half_of(i));
                h.x > h.z * 1.5 && mount.iter().any(|&m| c.y > plan.boxes[m].1.y - 0.2)
            }
        });
        if let Some(strip) = top_strip {
            // From the back of the haunch to the front of the shoulders: the
            // mountable parts that are not the tail (lower than the back).
            let back: Vec<usize> = mount
                .iter()
                .copied()
                .filter(|&m| plan.boxes[m].1.y > centre_of(strip).y - 1.2)
                .collect();
            let x0 = back
                .iter()
                .map(|&m| plan.boxes[m].0.x)
                .fold(f32::MAX, f32::min)
                + 0.5;
            let x1 = back
                .iter()
                .map(|&m| plan.boxes[m].1.x)
                .fold(f32::MIN, f32::max)
                - 0.25;
            let gap = half_of(strip).z + 0.22;
            let n = 7;
            for k in 0..n {
                let u = k as f32 / (n - 1) as f32; // 0 at the back
                let x = x0 + (x1 - x0) * u;
                let h = plan.spines
                    * (0.38 + 0.62 * (std::f32::consts::PI * (0.15 + 0.7 * u)).sin().powf(0.8));
                for side in [-1.0f32, 1.0] {
                    let z = side * gap;
                    let Some((p, nrm)) = meet(field, Vec3::new(x, 12.0, z), Vec3::new(x, 0.0, z))
                    else {
                        continue;
                    };
                    let u_dir = Vec3::new(-0.45, 1.0, side * 0.28).normalize();
                    let tip = p + u_dir * h + Vec3::new(-0.3 * h, -0.05 * h, 0.0);
                    out.push(piece(
                        vec![p - nrm * 0.25, p + u_dir * h * 0.45, tip],
                        &[0.20 * h + 0.06, 0.12 * h + 0.03, 0.0],
                        6.0,
                        0.05,
                        1.5,
                        Some(Vec3::X),
                        (plan.bone_at)(p),
                        strip,
                        Kind::Spine,
                    ));
                }
            }
        }
        // Down the tail: the parts behind the body, by their bones' names
        // or by lying behind the hips, shrinking toward the end.
        let tail: Vec<usize> = (0..sp.parts.len())
            .filter(|&i| {
                sp.bones[sp.parts[i].shape.bone].name.starts_with("tail") && !sp.parts[i].weak
            })
            .collect();
        for &i in &tail {
            let (lo, hi) = plan.boxes[i];
            for k in 0..3 {
                let x = hi.x - (hi.x - lo.x) * (0.2 + 0.3 * k as f32);
                let size = (half_of(i).z / 0.6).clamp(0.3, 1.0) * (1.0 - 0.12 * k as f32);
                let h = plan.spines * 0.5 * size + 0.08;
                for side in [-1.0f32, 1.0] {
                    let z = side * half_of(i).z * 0.45;
                    let Some((p, nrm)) = meet(field, Vec3::new(x, 12.0, z), Vec3::new(x, 0.0, z))
                    else {
                        continue;
                    };
                    let u_dir = Vec3::new(-0.5, 1.0, side * 0.45).normalize();
                    out.push(piece(
                        vec![
                            p - nrm * 0.12,
                            p + u_dir * h * 0.5,
                            p + u_dir * h + Vec3::new(-0.15 * h, 0.0, 0.0),
                        ],
                        &[0.13 * size + 0.04, 0.07 * size + 0.02, 0.0],
                        4.0,
                        0.05,
                        1.0,
                        None,
                        sp.parts[i].shape.bone,
                        i,
                        Kind::Spine,
                    ));
                }
            }
        }
        // The collar: spikes where the armour ends over the part the
        // forelegs hang from, flaring out and back.
        let fore_hip = sp
            .legs
            .iter()
            .find(|l| l.front)
            .map(|l| sp.bones[l.hip].parent);
        if let Some(carrier) = fore_hip.and_then(|b| {
            (0..sp.parts.len()).find(|&i| sp.parts[i].shape.bone == b && !sp.parts[i].weak)
        }) {
            let (lo, hi) = plan.boxes[carrier];
            let c = centre_of(carrier);
            for k in 0..4 {
                let x = lo.x + (hi.x - lo.x) * (0.15 + 0.23 * k as f32);
                let length =
                    plan.spines * (0.5 + 0.2 * (std::f32::consts::PI * k as f32 / 3.0).sin());
                for side in [-1.0f32, 1.0] {
                    let ang = plan.reach - 0.12;
                    let d = Vec3::new(0.0, ang.cos(), ang.sin() * side);
                    let axis = Vec3::new(x, c.y, 0.0);
                    let Some((p, nrm)) = meet(field, axis + d * 4.0, axis) else {
                        continue;
                    };
                    let u_dir = (nrm + Vec3::new(-0.55, 0.25, 0.0)).normalize();
                    out.push(piece(
                        vec![
                            p - nrm * 0.15,
                            p + u_dir * length * 0.5,
                            p + u_dir * length + Vec3::new(-0.12, -0.06, 0.0),
                        ],
                        &[0.15, 0.09, 0.0],
                        5.0,
                        0.05,
                        1.0,
                        None,
                        sp.parts[carrier].shape.bone,
                        carrier,
                        Kind::Spine,
                    ));
                }
            }
        }
    }

    // -- Horns, brow nubs, a nose horn, tusks: off the head.
    if plan.horns > 0.0 {
        let head = look.head;
        let (lo, hi) = plan.boxes[head];
        let (c, h) = (centre_of(head), half_of(head));
        let bone = sp.parts[head].shape.bone;
        let k = plan.horns / 1.6;
        for side in [-1.0f32, 1.0] {
            let at = Vec3::new(lo.x + (hi.x - lo.x) * 0.18, hi.y - 0.05, side * h.z * 0.75);
            let root = meet(field, at + Vec3::new(0.0, 1.0, side * 0.5), c)
                .map(|(p, _)| p)
                .unwrap_or(at);
            let offs = [
                Vec3::ZERO,
                Vec3::new(-0.30, 0.40, 0.26),
                Vec3::new(-0.55, 0.85, 0.48),
                Vec3::new(-0.40, 1.30, 0.62),
                Vec3::new(0.0, 1.53, 0.60),
                Vec3::new(0.37, 1.47, 0.46),
            ];
            let path: Vec<Vec3> = offs
                .iter()
                .map(|o| root + Vec3::new(o.x, o.y, o.z * side) * k)
                .collect();
            out.push(piece(
                path,
                &[0.25 * k, 0.22 * k, 0.18 * k, 0.13 * k, 0.08 * k, 0.0],
                18.0,
                0.07,
                0.85,
                None,
                bone,
                head,
                Kind::Horn,
            ));
            // Nubs along the brow.
            for (j, (fx, len)) in [(0.62f32, 0.18f32), (0.48, 0.24), (0.34, 0.28)]
                .iter()
                .enumerate()
            {
                let at = Vec3::new(
                    lo.x + (hi.x - lo.x) * fx,
                    hi.y - 0.08 - 0.03 * j as f32,
                    side * h.z * 0.85,
                );
                let Some((p, _)) = meet(field, at + Vec3::new(0.0, 0.6, side * 0.6), c) else {
                    continue;
                };
                let d = Vec3::new(-0.35, 0.75, side * 0.55).normalize();
                out.push(piece(
                    vec![p - d * 0.06, p + d * len * 0.6 * k, p + d * len * k],
                    &[0.07 * k, 0.04 * k, 0.0],
                    3.0,
                    0.04,
                    1.0,
                    None,
                    bone,
                    head,
                    Kind::Horn,
                ));
            }
            // A tusk out of the jaw, up past the snout.
            let at = Vec3::new(
                lo.x + (hi.x - lo.x) * 0.8,
                lo.y + (hi.y - lo.y) * 0.15,
                side * h.z * 0.5,
            );
            if let Some((p, _)) = meet(field, at + Vec3::new(0.3, -0.3, side * 0.6), c) {
                out.push(piece(
                    vec![
                        p,
                        p + Vec3::new(0.15, 0.2, side * 0.06) * k,
                        p + Vec3::new(0.22, 0.43, side * 0.04) * k,
                    ],
                    &[0.075 * k, 0.05 * k, 0.0],
                    4.0,
                    0.03,
                    1.0,
                    None,
                    bone,
                    head,
                    Kind::Claw,
                ));
            }
        }
        // The nose horn.
        let at = Vec3::new(lo.x + (hi.x - lo.x) * 0.88, hi.y, 0.0);
        if let Some((p, nrm)) = meet(
            field,
            at + Vec3::new(0.3, 1.0, 0.0),
            Vec3::new(at.x, c.y, 0.0),
        ) {
            out.push(piece(
                vec![
                    p - nrm * 0.05,
                    p + Vec3::new(0.2, 0.3, 0.0) * k,
                    p + Vec3::new(0.28, 0.55, 0.0) * k,
                ],
                &[0.17 * k, 0.11 * k, 0.0],
                6.0,
                0.06,
                1.0,
                None,
                bone,
                head,
                Kind::Horn,
            ));
        }
    }

    // -- Claws round the front of every foot's pad.
    if plan.claws > 0 {
        for leg in sp.legs {
            let foot = leg.foot;
            let (lo, hi) = plan.boxes[foot];
            let c = centre_of(foot);
            let r = half_of(foot).x.max(half_of(foot).z);
            let n = plan.claws as usize;
            for j in 0..n {
                let a = if n == 1 {
                    0.0
                } else {
                    (j as f32 / (n - 1) as f32 - 0.5) * 1.7
                };
                let d = Vec3::new(a.cos(), 0.0, a.sin());
                let base = Vec3::new(c.x, lo.y + 0.3, c.z) + d * (r * 1.1);
                let Some((p, _)) = meet(field, base + d * 1.0, Vec3::new(c.x, lo.y + 0.3, c.z))
                else {
                    continue;
                };
                out.push(piece(
                    vec![
                        p - d * 0.12,
                        p + d * 0.12 + Vec3::new(0.0, -0.08, 0.0),
                        p + d * 0.24 + Vec3::new(0.0, -0.26, 0.0),
                    ],
                    &[0.15, 0.11, 0.0],
                    4.0,
                    0.04,
                    0.75,
                    None,
                    sp.parts[foot].shape.bone,
                    foot,
                    Kind::Claw,
                ));
                let _ = hi;
            }
        }
    }

    // -- Quills: fanned out of the end of the tail that throws them.
    if let Some(spikes) = &look.spikes {
        let end = spikes.along[1];
        let (lo, hi) = plan.boxes[end];
        let tip = Vec3::new(lo.x + 0.45, (lo.y + hi.y) * 0.5, 0.0);
        let bone = sp.parts[end].shape.bone;
        for k in 0..20u32 {
            let az = (k as f32 * 2.399_963) % std::f32::consts::TAU; // golden angle round it
            let el = -0.25 + 1.3 * ((k as f32 * 0.618_034) % 1.0);
            let d = Vec3::new(-0.6 * el.cos() - 0.4, el.sin(), 0.9 * az.sin()).normalize();
            let Some((p, nrm)) = meet(field, tip + d * 1.5, tip) else {
                continue;
            };
            let len = 0.45 + 0.4 * ((k as f32 * 0.37) % 1.0);
            let dd = (nrm * 0.8 + d * 0.5 + Vec3::new(-0.3, 0.1, 0.0)).normalize();
            out.push(piece(
                vec![p - dd * 0.08, p + dd * len * 0.5, p + dd * len],
                &[0.06, 0.035, 0.0],
                3.0,
                0.04,
                1.0,
                None,
                bone,
                end,
                Kind::Quill,
            ));
        }
    }
    out
}
