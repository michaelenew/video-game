//! `cargo run --release -p art --example timing` -- how long a full bake takes.
//!
//! The bake-on-the-processor argument in `art::bake` rests on a number, so the
//! number should be re-measurable rather than remembered. It has already been
//! wrong once: the first version of that doc claimed single-digit milliseconds
//! for a 512-pixel material and the real figure was closer to two thousand.

use art::{bake, materials, palette};
use std::time::Instant;
fn main() {
    for size in [256u32, 512] {
        let mut all = materials::FIXED.to_vec();
        all.push(materials::cloth(palette::PLAYERS[0]));
        all.push(materials::armour(palette::PLAYERS[0]));
        all.push(materials::arcane(palette::PLAYERS[3]));
        let start = Instant::now();
        let mut worst = (0.0f64, "");
        for m in &all {
            let t = Instant::now();
            std::hint::black_box(bake::bake(&m.surface, m.plan(size)));
            let ms = t.elapsed().as_secs_f64() * 1000.0;
            if ms > worst.0 {
                worst = (ms, m.name);
            }
        }
        println!(
            "{size}px: {} materials in {:.0} ms, slowest {} at {:.0} ms",
            all.len(),
            start.elapsed().as_secs_f64() * 1000.0,
            worst.1,
            worst.0
        );
    }
}
