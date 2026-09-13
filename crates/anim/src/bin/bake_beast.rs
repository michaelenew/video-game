//! Bake the creature's pose table.
//!
//!     cargo run -p anim --bin bake_beast
//!
//! Writes `crates/sim/src/beast_baked.rs`. **Edit the recipes in
//! `crates/anim/src/beast/clips.rs`, not the generated file.**
//!
//! The creature's parts are simulation geometry -- they are what you collide
//! with, stand on and hit -- so unlike the fighters' table this one is read by
//! `crates/sim`, and it is therefore written as fixed point. The solve that
//! produces it is ordinary offline f32 spring work; only the table has to be
//! deterministic, and a table is.

fn main() {
    let (baked, missing) = anim::beast::bake_all();

    let path = "crates/sim/src/beast_baked.rs";
    std::fs::write(path, anim::beast::emit(&baked)).expect("write the creature's table");

    for b in &baked {
        let frames = anim::beast::clips::all()
            .iter()
            .find(|r| r.clip == b.clip)
            .map(|r| r.frames())
            .unwrap_or(0);
        println!(
            "{:<10} {:>3} samples  solved over {frames:>3} frames",
            b.clip.name(),
            b.samples.len()
        );
    }
    let rows: usize = baked.iter().map(|b| b.samples.len()).sum();
    println!("\n{path}  ({} clips, {rows} rows)", baked.len());

    if !missing.is_empty() {
        println!(
            "\n{} clips have no recipe and bake as a held standing pose:",
            missing.len()
        );
        for c in missing {
            println!("  {}", c.name());
        }
    }
}
