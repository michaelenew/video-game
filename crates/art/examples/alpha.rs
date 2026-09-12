//! `cargo run -p art --example alpha` -- how much of a fading material exists.
//!
//! A fade that never reaches zero is an opaque surface with extra arithmetic,
//! and it looks exactly like the fade not being wired up at all.

use art::bake::bake;
use art::materials::{self, BAKE_SIZE};
use art::palette;

fn main() {
    let mut all = materials::FIXED.to_vec();
    all.push(materials::arcane(palette::PLAYERS[3]));
    for m in &all {
        let maps = bake(&m.surface, m.plan(BAKE_SIZE));
        let mut buckets = [0usize; 5];
        for px in maps.albedo.pixels.chunks(4) {
            buckets[(px[3] as usize * 5 / 256).min(4)] += 1;
        }
        let total: usize = buckets.iter().sum();
        let pct = |n: usize| n as f32 * 100.0 / total as f32;
        println!(
            "{:>9}  alpha<20%: {:>5.1}%  20-40: {:>5.1}%  40-60: {:>5.1}%  60-80: {:>5.1}%  >80%: {:>5.1}%   has_alpha {}",
            m.name,
            pct(buckets[0]),
            pct(buckets[1]),
            pct(buckets[2]),
            pct(buckets[3]),
            pct(buckets[4]),
            maps.has_alpha
        );
    }
}
