//! **Can it move** (exploration 0033): a bred body walked through its
//! slowest gait and held to what a walking animal's legs do -- the cast's
//! own genomes, and every stage of the lines exploration 0031 breeds.
//!
//! "Balanced" was not enough. The posture put the mass over the feet by
//! dragging the feet together under it, and the walk took strides nearly a
//! leg long, so legs crossed through each other and a second rule stepped
//! the colliding ones out sideways. These are the properties that picture
//! broke, each measured off the built body's bones at sixteen points of the
//! cycle:
//!
//! - **legs do not cross**: on each side, a pair's foot stays ahead of the
//!   next pair's, and no two legs pass through each other;
//! - **a foot is under its hip**, not stepped out to the side (a sprawling
//!   build -- a spider's, a low-slung body's -- is allowed its sprawl);
//! - **only feet touch the floor**: no scythe, wing, belly or head;
//! - **it is carried**: the mass over the feet that are down;
//! - **a step is a step**: the feet travel.

use sim::breed::archetypes::{self, CAST, GALEWING, MANTIS, RIDGEBACK};
use sim::breed::build::{self, Body};
use sim::breed::cross::{self, Rates, Trophy};
use sim::breed::locomotion::{self, GaitKind};
use sim::breed::{Genome, Rng, posture};
use sim::fixed::Fx;

fn f(x: Fx) -> f32 {
    x.to_f32_for_render()
}

type P = [f32; 3];

fn at(body: &Body, name: &str) -> Option<P> {
    let b = body.bone_named(name)?;
    let p = body.at(b);
    Some([f(p.x), f(p.y), f(p.z)])
}

fn sub(a: P, b: P) -> P {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: P, b: P) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// The least distance between two segments.
fn seg_dist(p1: P, q1: P, p2: P, q2: P) -> f32 {
    let (d1, d2, r) = (sub(q1, p1), sub(q2, p2), sub(p1, p2));
    let (a, e, ff) = (dot(d1, d1), dot(d2, d2), dot(d2, r));
    let (s, t);
    if a <= 1e-6 && e <= 1e-6 {
        return dot(r, r).sqrt();
    }
    if a <= 1e-6 {
        s = 0.0;
        t = (ff / e).clamp(0.0, 1.0);
    } else {
        let c = dot(d1, r);
        if e <= 1e-6 {
            t = 0.0;
            s = (-c / a).clamp(0.0, 1.0);
        } else {
            let b = dot(d1, d2);
            let den = a * e - b * b;
            let mut s0 = if den > 1e-9 {
                ((b * ff - c * e) / den).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let mut t0 = (b * s0 + ff) / e;
            if t0 < 0.0 {
                t0 = 0.0;
                s0 = (-c / a).clamp(0.0, 1.0);
            } else if t0 > 1.0 {
                t0 = 1.0;
                s0 = ((b - c) / a).clamp(0.0, 1.0);
            }
            s = s0;
            t = t0;
        }
    }
    let c1 = [p1[0] + d1[0] * s, p1[1] + d1[1] * s, p1[2] + d1[2] * s];
    let c2 = [p2[0] + d2[0] * t, p2[1] + d2[1] * t, p2[2] + d2[2] * t];
    let d = sub(c1, c2);
    dot(d, d).sqrt()
}

const STATIONS: [&str; 4] = ["fore", "midfore", "midhind", "hind"];
const SIDES: [&str; 2] = [".l", ".r"];

/// What is wrong with a body's walk, one line each; empty is a body that
/// can move.
fn faults(g: &Genome) -> Vec<String> {
    let mut out = vec![];
    let q = posture::posture(g);
    let p = build::proportions(g);
    if g.stance == sim::breed::Stance::Limbless {
        return out;
    }
    let gaits = locomotion::repertoire(g, &q);
    let Some(gait) = gaits
        .iter()
        .flatten()
        .find(|x| x.kind != GaitKind::Stand && x.kind != GaitKind::Fly)
        .copied()
    else {
        return out;
    };
    // Which pairs stand, and which are not legs at all.
    let walks: Vec<usize> = (0..4).filter(|&i| q.stations[i].grounded).collect();
    let mut worst_cross = 0.0f32;
    let mut worst_side = 0.0f32;
    let mut worst_floor = (0.0f32, String::new());
    let mut worst_carry = 0.0f32;
    let mut travel = [0.0f32; 4];
    for k in 0..16 {
        let phase = Fx::ratio(k, 16);
        let d = locomotion::drive(g, &q, &gait, phase, Fx::ZERO);
        let body = build::body_driven(g, &q, &d);
        let root = body.at(0);
        let height = f(root.y).max(0.1);
        // Legs: hip, knee, ankle, per pair and side.
        let leg = |i: usize, s: usize| -> Option<[P; 3]> {
            let n = format!("{}{}", STATIONS[i], SIDES[s]);
            Some([
                at(&body, &n)?,
                at(&body, &format!("{n}.lower"))?,
                at(&body, &format!("{n}.foot"))?,
            ])
        };
        for s in 0..2 {
            for (a, &i) in walks.iter().enumerate() {
                for &j in &walks[a + 1..] {
                    let (Some(li), Some(lj)) = (leg(i, s), leg(j, s)) else {
                        continue;
                    };
                    let room = f(p.girth[i]) + f(p.girth[j]);
                    // The fore pair's foot ahead of the hind pair's.
                    let ahead = li[2][0] - lj[2][0];
                    worst_cross = worst_cross.max((room - ahead) / room);
                    // And no segment of one through a segment of the other.
                    for (a0, a1) in [(li[0], li[1]), (li[1], li[2])] {
                        for (b0, b1) in [(lj[0], lj[1]), (lj[1], lj[2])] {
                            let gap = seg_dist(a0, a1, b0, b1);
                            worst_cross = worst_cross.max((room * 0.8 - gap) / room);
                        }
                    }
                }
            }
            for &i in &walks {
                let Some(l) = leg(i, s) else { continue };
                let pair = g.limbs[i].unwrap();
                let standing = posture::standing_length(
                    g.stance,
                    pair.build,
                    q.stations[i].wing_walks,
                    p.limb[i],
                );
                // A sprawl is its own; and a wing walked on holds its elbow
                // out, as a pterosaur's does.
                if !posture::splayed(g.stance, pair.build) && !q.stations[i].wing_walks {
                    let out = (l[2][2].abs() - l[0][2].abs()).max(l[1][2].abs() - l[0][2].abs());
                    worst_side = worst_side.max(out / f(standing).max(0.01));
                }
                if k == 0 {
                    travel[i] = l[2][0];
                } else {
                    travel[i] = travel[i].max((l[2][0] - travel[i]).abs());
                }
            }
        }
        // Only feet on the floor.
        for i in 0..body.part_count {
            let part = &body.parts[i];
            let bone = body.bones[part.shape.bone].name;
            let station = STATIONS
                .iter()
                .position(|s| bone.starts_with(s) && bone.as_bytes().get(s.len()) == Some(&b'.'));
            let foot_end = bone.ends_with(".foot") || bone.ends_with(".lower");
            if let Some(st) = station {
                if q.stations[st].grounded && foot_end {
                    continue;
                }
            }
            if q.bellied || part.shape.hollow {
                continue;
            }
            if q.drags && matches!(part.name, "rump" | "haunch" | "abdomen") {
                continue;
            }
            if part.name.starts_with("tail") {
                continue;
            }
            let low = f(body.lowest(i));
            // A sprawling leg's upper half runs out near the belly's height:
            // it need only be off the floor.
            let sprawl = station
                .is_some_and(|st| g.limbs[st].is_some_and(|l| posture::splayed(g.stance, l.build)));
            let clear = if sprawl { 0.0 } else { height * 0.08 };
            if clear - low > worst_floor.0 {
                worst_floor = (clear - low, part.name.to_string());
            }
        }
        // Carried: the mass over the planted feet, front to back.
        if gait.kind != GaitKind::Crawl && walks.len() >= 2 && !q.bellied {
            let mut lo = f32::MAX;
            let mut hi = f32::MIN;
            for &i in &walks {
                for s in 0..2 {
                    let Some(l) = leg(i, s) else { continue };
                    let foot = d.feet[i][s];
                    if foot.lift == Fx::ZERO {
                        let fl = f(p.limb[i]) * 0.15;
                        lo = lo.min(l[2][0] - fl);
                        hi = hi.max(l[2][0] + fl);
                    }
                }
            }
            let com = f(q.com) + f(root.x);
            if lo <= hi {
                let off = (lo - com).max(com - hi).max(0.0);
                worst_carry = worst_carry.max(off / f(p.torso).max(0.1));
            }
        }
    }
    if worst_cross > 0.0 {
        out.push(format!("legs cross ({worst_cross:.2})"));
    }
    // An elbow held a little out from a deep body is an animal's; a fifth
    // of the leg out is a leg stepped round another.
    if worst_side > 0.2 {
        out.push(format!(
            "a foot out to the side ({worst_side:.2} of the leg)"
        ));
    }
    if worst_floor.0 > 0.0 {
        out.push(format!(
            "{} on the floor ({:.2} m under clear)",
            worst_floor.1, worst_floor.0
        ));
    }
    if worst_carry > 0.0 {
        out.push(format!(
            "not carried ({worst_carry:.2} of the torso off its feet)"
        ));
    }
    for &i in &walks {
        let pair = g.limbs[i].unwrap();
        let standing = f(posture::standing_length(
            g.stance,
            pair.build,
            q.stations[i].wing_walks,
            p.limb[i],
        ));
        if gait.kind != GaitKind::Crawl && travel[i] < standing * 0.08 {
            out.push(format!(
                "{} pair barely steps ({:.2} of the leg)",
                STATIONS[i],
                travel[i] / standing
            ));
        }
    }
    out
}

/// The lines of exploration 0031, bred the same way `game::bred::lines`
/// breeds them.
fn lines() -> Vec<(&'static str, Vec<Genome>)> {
    let fed = |from: &Genome, to: &Genome, n: usize, seed: u64| {
        let mut rng = Rng::new(seed);
        let drops: Vec<_> = archetypes::drops(to).into_iter().flatten().collect();
        let mut g = *from;
        let mut out = vec![g];
        for k in 0..n {
            let kind = drops[k % drops.len().max(1)];
            g = cross::feed(&g, &Trophy { kind, donor: *to }, &mut rng).genome;
            out.push(g);
        }
        out
    };
    let crossed = |from: &Genome, to: &Genome, n: usize, seed: u64| {
        let mut rng = Rng::new(seed);
        let mut g = *from;
        let mut out = vec![g];
        for _ in 0..n {
            g = cross::breed(&g, to, &Rates::DEFAULT, &mut rng).genome;
            out.push(g);
        }
        out
    };
    vec![
        ("ridgeback_to_galewing", fed(&RIDGEBACK, &GALEWING, 12, 1)),
        ("galewing_to_mantis", fed(&GALEWING, &MANTIS, 12, 2)),
        ("mantis_to_ridgeback", fed(&MANTIS, &RIDGEBACK, 12, 3)),
        ("ridgeback_x_mantis", crossed(&RIDGEBACK, &MANTIS, 8, 4)),
    ]
}

#[test]
fn every_body_can_move() {
    let mut bad = vec![];
    let mut total = 0;
    for (_, name, g, _) in CAST {
        total += 1;
        let e = faults(g);
        if !e.is_empty() {
            bad.push(format!("{name}: {}", e.join("; ")));
        }
    }
    for (line, stages) in lines() {
        for (k, g) in stages.iter().enumerate() {
            total += 1;
            let e = faults(g);
            if !e.is_empty() {
                bad.push(format!("{line} {k}: {}", e.join("; ")));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "{} of {total} bodies cannot move:\n{}",
        bad.len(),
        bad.join("\n")
    );
}
