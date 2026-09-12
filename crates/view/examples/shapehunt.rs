//! Throwaway: hill-climbs the six class builds to maximise the *worst* pair.
//!
//! A roster is only as readable as its two most confusable fighters, which is
//! the same reasoning the palette search used. Bounds keep each build inside
//! what its class is described as being -- the search picks between plausible
//! bodies, it does not get to decide the Bulwark is willowy.

use view::build::{Build, Silhouette, distinctness};

const NAMES: [&str; 6] = [
    "Bulwark",
    "Champion",
    "Reaver",
    "Elementalist",
    "Blood mage",
    "Dual mage",
];

/// Per class, the range each parameter is allowed to move in.
/// scale, breadth, shoulders, reach, stride, head, limb
const BOUNDS: [[(f32, f32); 7]; 6] = [
    // Bulwark: broad and planted, always.
    [
        (0.98, 1.10),
        (1.20, 1.45),
        (1.05, 1.25),
        (0.85, 1.00),
        (0.78, 0.92),
        (0.82, 0.96),
        (1.20, 1.45),
    ],
    // Champion: the baseline athlete. Held near even on purpose -- something
    // has to be the reference body.
    [
        (0.98, 1.04),
        (0.96, 1.06),
        (0.98, 1.10),
        (0.98, 1.08),
        (0.96, 1.06),
        (0.95, 1.05),
        (0.96, 1.06),
    ],
    // Reaver: long, thin, longest reach.
    [
        (1.00, 1.10),
        (0.70, 0.85),
        (0.85, 0.98),
        (1.15, 1.32),
        (1.10, 1.25),
        (0.84, 0.96),
        (0.72, 0.86),
    ],
    // Elementalist: small, hooded, short reach, stands behind her structures.
    [
        (0.86, 0.96),
        (1.05, 1.25),
        (0.74, 0.88),
        (0.74, 0.88),
        (0.90, 1.06),
        (1.18, 1.42),
        (0.92, 1.14),
    ],
    // Blood mage: gaunt and small.
    [
        (0.86, 0.96),
        (0.74, 0.88),
        (0.80, 0.92),
        (1.10, 1.26),
        (0.88, 1.00),
        (1.02, 1.18),
        (0.70, 0.84),
    ],
    // Dual mage: tall, wide stance, balanced.
    [
        (1.06, 1.16),
        (0.90, 1.02),
        (1.18, 1.38),
        (0.90, 1.02),
        (1.08, 1.22),
        (0.80, 0.92),
        (1.02, 1.16),
    ],
];

fn set(b: &Build) -> [f32; 7] {
    [
        b.scale,
        b.breadth,
        b.shoulders,
        b.reach,
        b.stride,
        b.head,
        b.limb,
    ]
}

fn build_from(v: [f32; 7]) -> Build {
    Build {
        scale: v[0],
        breadth: v[1],
        shoulders: v[2],
        reach: v[3],
        stride: v[4],
        head: v[5],
        limb: v[6],
    }
}

fn worst(builds: &[Build; 6]) -> f32 {
    let sils: Vec<Silhouette> = builds.iter().map(Silhouette::of).collect();
    let mut w = 1.0f32;
    for i in 0..6 {
        for j in i + 1..6 {
            w = w.min(distinctness(&sils[i], &sils[j]));
        }
    }
    w
}

fn main() {
    let mut cur: [[f32; 7]; 6] = std::array::from_fn(|i| {
        let v = set(&view::build::CLASS_BUILDS[i]);
        std::array::from_fn(|k| v[k].clamp(BOUNDS[i][k].0, BOUNDS[i][k].1))
    });
    let mut best = worst(&std::array::from_fn(|i| build_from(cur[i])));
    println!("starting worst pair {best:.3}");

    // Coordinate descent: try each parameter of each class at each end of its
    // range and in the middle, keep whatever raises the floor. Deterministic,
    // and with 42 parameters it converges in seconds.
    for _round in 0..12 {
        let mut improved = false;
        for class in 0..6 {
            for k in 0..7 {
                let (lo, hi) = BOUNDS[class][k];
                // The winning value is remembered and written once at the end.
                // The first version reverted to the value captured *before* the
                // sweep whenever a step failed, which threw away any
                // improvement an earlier step in the same sweep had made -- so
                // it reported a better score than the numbers it printed.
                let mut keep = cur[class][k];
                for step in 0..9 {
                    cur[class][k] = lo + (hi - lo) * step as f32 / 8.0;
                    let score = worst(&std::array::from_fn(|i| build_from(cur[i])));
                    if score > best + 1e-5 {
                        best = score;
                        keep = cur[class][k];
                        improved = true;
                    }
                }
                cur[class][k] = keep;
            }
        }
        if !improved {
            break;
        }
    }

    println!("worst pair {best:.3}\n");
    for (i, v) in cur.iter().enumerate() {
        println!(
            "    // {}\n    Build {{ scale: {:.2}, breadth: {:.2}, shoulders: {:.2}, reach: {:.2}, stride: {:.2}, head: {:.2}, limb: {:.2} }},",
            NAMES[i], v[0], v[1], v[2], v[3], v[4], v[5], v[6]
        );
    }

    let builds: [Build; 6] = std::array::from_fn(|i| build_from(cur[i]));
    let sils: Vec<Silhouette> = builds.iter().map(Silhouette::of).collect();
    println!("\npairs:");
    for i in 0..6 {
        for j in i + 1..6 {
            println!(
                "  {:>13} / {:<13} {:.3}",
                NAMES[i],
                NAMES[j],
                distinctness(&sils[i], &sils[j])
            );
        }
    }
}
