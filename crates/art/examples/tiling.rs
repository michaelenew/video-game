//! `cargo run --release -p art --example tiling` -- what tiling costs, measured.
//!
//! The tileable bake buys a seamless repeat by blending the tile against three
//! shifted copies of itself, which pulls every texel toward the mean. This
//! prints how much contrast that costs, per world material, so the decision
//! about whether tiling is good enough is made on a number.

use art::bake::{Plan, bake};
use art::materials::{self, BAKE_SIZE, Role};

fn spread(m: &materials::Material, tiling: bool) -> (u8, f32) {
    let plan = Plan {
        tileable: tiling,
        ..m.plan(BAKE_SIZE)
    };
    let maps = bake(&m.surface, plan);
    let (mut lo, mut hi) = (255u8, 0u8);
    let mut sum = 0.0f64;
    let mut sq = 0.0f64;
    let n = (maps.albedo.pixels.len() / 4) as f64;
    for px in maps.albedo.pixels.chunks(4) {
        lo = lo.min(px[0]);
        hi = hi.max(px[0]);
        sum += px[0] as f64;
        sq += (px[0] as f64).powi(2);
    }
    let mean = sum / n;
    let sd = ((sq / n) - mean * mean).max(0.0).sqrt();
    (hi - lo, sd as f32)
}

fn main() {
    println!(
        "{:>9}  {:>18}  {:>18}  {:>8}",
        "material", "tiled range / sd", "free range / sd", "sd kept"
    );
    for m in materials::FIXED {
        if m.role != Role::World {
            continue;
        }
        let (tr, tsd) = spread(m, true);
        let (fr, fsd) = spread(m, false);
        println!(
            "{:>9}  {:>8} / {:>7.1}  {:>8} / {:>7.1}  {:>7.0}%",
            m.name,
            tr,
            tsd,
            fr,
            fsd,
            tsd / fsd * 100.0
        );
    }
    println!(
        "\nThe arena floor tile is {} m and the camera sits about 11 m back, so a\n\
         repeat would be plainly visible if there were anything left to repeat.",
        materials::GROUND.extent
    );
}
