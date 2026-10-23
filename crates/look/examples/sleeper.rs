//! **The Sleeper's land**: the valley's own heights round the beast asleep
//! in the mountains (`sim::valley::dragon`), for `scripts/sleeper/sheet.py`
//! to draw beside the posed model.
//!
//! ```text
//! cargo run --release -p look --example sleeper   # target/sleeper-land.txt
//! ```
//!
//! A grid of `Land::height`, three metres a cell, over the beast and the
//! mountain round it, in the map's metres: the first line is the grid's west
//! and south edges, its cell and its size; then one row a line, from the
//! south.

use std::fmt::Write;

fn f(v: sim::Fx) -> f32 {
    v.to_f32_for_render()
}

fn fx(v: f32) -> sim::Fx {
    sim::Fx::from_raw((v * 65536.0) as i32)
}

fn main() {
    let land = sim::atlas::valley().land.as_ref().expect("a map with land");
    let (west, south, cell, n) = (560.0f32, 200.0f32, 3.0f32, 150usize);
    let mut out = format!("{west} {south} {cell} {n} {n}\n");
    for j in 0..n {
        for i in 0..n {
            let h = f(land.height(fx(west + i as f32 * cell), fx(south + j as f32 * cell)));
            write!(out, "{h:.2} ").unwrap();
        }
        out.push('\n');
    }
    std::fs::create_dir_all("target").unwrap();
    std::fs::write("target/sleeper-land.txt", out).unwrap();
    println!("target/sleeper-land.txt");
}
