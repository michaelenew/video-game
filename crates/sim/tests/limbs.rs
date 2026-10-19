//! Bodies the builder makes stand clear of themselves (2026-10-19): an
//! abdomen is carried off the floor, and one pair's legs do not pass
//! through the next pair's -- standing, or walking.

use sim::Fx;
use sim::breed::archetypes::{self, CAST};
use sim::breed::build::{self, Body};
use sim::breed::morph::Morph;
use sim::breed::{Genome, locomotion, posture};

type P = [f32; 3];

fn f(v: sim::V3) -> P {
    [
        v.x.to_f32_for_render(),
        v.y.to_f32_for_render(),
        v.z.to_f32_for_render(),
    ]
}

fn cast(name: &str) -> Genome {
    *CAST.iter().find(|c| c.1 == name).unwrap().2
}

/// How far below the floor a part's underside is, standing (negative is
/// above it).
fn under_floor(b: &Body, i: usize) -> f32 {
    let p = &b.parts[i];
    let u = b.axes[i];
    let (v, w) = build::frame(u);
    let d = p.shape.max.sub(p.shape.min);
    let h = |a: sim::V3, l: Fx| a.y.to_f32_for_render().abs() * l.to_f32_for_render() / 2.0;
    -(b.centre(i).y.to_f32_for_render() - h(u, d.x) - h(v, d.y) - h(w, d.z))
}

#[test]
fn an_abdomen_is_carried_off_the_floor() {
    // The Mantis's own trailed at its upright torso's pitch, 0.8 m into
    // the ground; and every morph fed its trophies after it.
    let mantis = cast("Mantis");
    let kinds: Vec<_> = archetypes::drops(&mantis).into_iter().flatten().collect();
    let mut bodies = vec![("Mantis".to_string(), mantis)];
    for (_, egg, eg, _) in CAST {
        let mut m = Morph::hatch(eg, 0);
        for k in 0..8 {
            m.eat(kinds[k % kinds.len()], &mantis);
            if m.genome.segments.abdomen && m.genome.stance == sim::breed::Stance::Upright {
                bodies.push((format!("{egg} + {} Mantis", k + 1), m.shown()));
            }
        }
    }
    for (name, g) in bodies {
        let b = build::body(&g);
        for i in 0..b.part_count {
            if b.parts[i].name == "abdomen" {
                let sunk = under_floor(&b, i);
                assert!(
                    sunk <= 0.0,
                    "{name}'s abdomen is {sunk:.2} m into the floor"
                );
            }
        }
    }
}

fn seg_dist(a0: P, a1: P, b0: P, b1: P) -> f32 {
    let mut best = f32::MAX;
    for i in 0..=8 {
        for j in 0..=8 {
            let (s, t) = (i as f32 / 8.0, j as f32 / 8.0);
            let d: f32 = (0..3)
                .map(|k| (a0[k] + (a1[k] - a0[k]) * s - b0[k] - (b1[k] - b0[k]) * t).powi(2))
                .sum();
            best = best.min(d.sqrt());
        }
    }
    best
}

/// Where two pairs' legs on one side pass through each other, leaving out
/// the two upper segments where they leave the body side by side.
fn crossings(b: &Body) -> Vec<String> {
    let mut segs = vec![];
    for i in 0..b.part_count {
        let n = b.parts[i].name;
        let Some(st) = ["fore.", "midfore.", "midhind.", "hind."]
            .iter()
            .position(|p| n.starts_with(p))
        else {
            continue;
        };
        if n.ends_with("foot") {
            continue;
        }
        let side = n.contains(".l");
        let (c, u) = (f(b.centre(i)), f(b.axes[i]));
        let d = b.parts[i].shape.max.sub(b.parts[i].shape.min);
        let (half, girth) = (d.x.to_f32_for_render() / 2.0, d.y.to_f32_for_render() / 2.0);
        let l = (half - girth).max(0.0);
        let end = |s: f32| {
            [
                c[0] + u[0] * l * s,
                c[1] + u[1] * l * s,
                c[2] + u[2] * l * s,
            ]
        };
        segs.push((st, side, n, end(-1.0), end(1.0), girth));
    }
    let mut out = vec![];
    for a in 0..segs.len() {
        for b in a + 1..segs.len() {
            let (x, y) = (&segs[a], &segs[b]);
            if x.0 == y.0 || x.1 != y.1 {
                continue;
            }
            let uppers = !x.2.ends_with("lower") && !y.2.ends_with("lower");
            let apart: f32 = (0..3)
                .map(|k| (x.3[k] - y.3[k]).powi(2))
                .sum::<f32>()
                .sqrt();
            if uppers && apart < (x.5 + y.5) * 3.0 {
                continue;
            }
            if seg_dist(x.3, x.4, y.3, y.4) < (x.5 + y.5) * 0.9 {
                out.push(format!("{}~{}", x.2, y.2));
            }
        }
    }
    out
}

/// Crossings standing, and the most at any point of a walk.
fn crossing(g: &Genome) -> (usize, usize) {
    let q = posture::posture(g);
    let rest = crossings(&build::body_in(g, &q)).len();
    let gait = locomotion::at(g, &q, Fx::ratio(1, 2));
    let walk = (0..8)
        .map(|k| {
            let d = locomotion::drive(g, &q, &gait, Fx::ratio(k, 8), Fx::ZERO);
            crossings(&build::body_driven(g, &q, &d)).len()
        })
        .max()
        .unwrap_or(0);
    (rest, walk)
}

#[test]
fn a_creatures_legs_stand_clear_of_each_other() {
    // The Siegeshell's front pairs stand closer together than they are
    // thick, which no bend clears; every other one stands clear.
    for (_, name, g, _) in CAST {
        if name.eq("Siegeshell") {
            continue;
        }
        let (rest, _) = crossing(g);
        assert_eq!(rest, 0, "{name}'s legs cross standing");
    }
}

#[test]
fn most_bred_bodies_walk_clear_of_themselves() {
    // Six of one creature's trophies on every other's egg: a ratchet, from
    // 56 crossing standing and 74 walking of 121 before the legs stepped
    // out of each other's way (2026-10-19) to 14 and 26.
    let (mut built, mut rest, mut walk) = (0, 0, 0);
    for (_, _, eg, _) in CAST {
        for (_, _, dg, _) in CAST {
            let kinds: Vec<_> = archetypes::drops(dg).into_iter().flatten().collect();
            if kinds.is_empty() {
                continue;
            }
            let mut m = Morph::hatch(eg, 0);
            for k in 0..6 {
                m.eat(kinds[k % kinds.len()], dg);
            }
            let (r, w) = crossing(&m.shown());
            built += 1;
            rest += (r > 0) as usize;
            walk += (w > 0) as usize;
        }
    }
    assert!(rest <= 14, "{rest} of {built} cross standing");
    assert!(walk <= 26, "{walk} of {built} cross walking");
}
