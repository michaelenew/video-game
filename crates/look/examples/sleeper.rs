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
//! south. Also `target/sleeper-bare.txt`: the land with no Sleeper, wide.

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
    // And the land with no Sleeper on it, wide -- the valley, the High
    // Plane and the ridges between -- four metres a cell: its height and how
    // far it has risen off the floor it rises from, for `vista.py` to lay a
    // beast on and look at from where a player stands.
    let (west, south, cell, nx, nz) = (400.0f32, -150.0f32, 4.0f32, 201usize, 226usize);
    let mut out = format!("{west} {south} {cell} {nx} {nz}\n");
    for j in 0..nz {
        for i in 0..nx {
            let s = land.bare_sample(fx(west + i as f32 * cell), fx(south + j as f32 * cell));
            write!(out, "{:.2},{:.2} ", f(s.height), f(s.rise)).unwrap();
        }
        out.push('\n');
    }
    std::fs::write("target/sleeper-bare.txt", out).unwrap();
    println!("target/sleeper-bare.txt");
}
