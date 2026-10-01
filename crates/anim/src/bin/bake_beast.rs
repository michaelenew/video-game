//! Bake the creatures' pose tables.
//!
//!     cargo run -p anim --bin bake_beast
//!     cargo run -p anim --bin bake_beast -- --species ridgeback
//!
//! Every species with an animation module (`crates/anim/src/beast/<species>/`)
//! by default, or just the one named. Each writes its own
//! `crates/sim/src/species/<species>/baked.rs`, so two species never share a
//! file. **Edit the recipes, not the generated files.**
//!
//! The creatures' parts are simulation geometry -- they are what you collide
//! with, stand on and hit -- so unlike the fighters' table these are read by
//! `crates/sim`, and they are therefore written as fixed point. The solve that
//! produces them is ordinary offline f32 spring work; only the table has to be
//! deterministic, and a table is.

use anim::beast::{self, Authored};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let only = args
        .iter()
        .position(|a| a == "--species")
        .and_then(|i| args.get(i + 1))
        .map(|name| {
            sim::species::named(name).unwrap_or_else(|| {
                eprintln!("no species is called {name}");
                std::process::exit(2);
            })
        });
    let mut any = false;
    for species in sim::species::all() {
        if only.is_some_and(|o| o.id != species.id) {
            continue;
        }
        let Some(authored) = beast::authored(species.id) else {
            continue;
        };
        any = true;
        bake_one(authored);
    }
    if !any {
        eprintln!("nothing to bake: no species with an animation module matched");
        std::process::exit(2);
    }
}

fn bake_one(authored: &dyn Authored) {
    let species = authored.species();
    let (baked, missing) = beast::bake_all(authored);

    let path = authored.baked_path();
    std::fs::write(path, beast::emit(authored, &baked)).expect("write the creature's table");

    println!("the {}", species.name);
    let recipes = authored.recipes();
    for b in &baked {
        let frames = recipes
            .iter()
            .find(|r| r.clip == b.clip)
            .map(|r| r.frames(species))
            .unwrap_or(0);
        println!(
            "{:<10} {:>3} samples  solved over {frames:>3} frames",
            species.clips[b.clip].name,
            b.samples.len()
        );
    }
    let rows: usize = baked.iter().map(|b| b.samples.len()).sum();
    println!("\n{path}  ({} clips, {rows} rows)\n", baked.len());

    if !missing.is_empty() {
        println!(
            "{} clips have no recipe and bake as a held standing pose:",
            missing.len()
        );
        for c in missing {
            println!("  {}", species.clips[c].name);
        }
    }
}
