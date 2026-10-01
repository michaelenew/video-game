//! Write every baked tuning file from whatever the Oven currently holds:
//! `crates/sim/src/tuned.rs`, and each species' own
//! (`crates/sim/src/species/<species>/tuned.rs`).
//!
//!     cargo run -p sim --bin bake_tuning
//!
//! The Oven's bake button calls the same emitters. This binary exists so the
//! files can be regenerated without launching the game, and so the very first
//! one could be produced from the values that used to be hand-written.

fn main() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../");
    for (rel, text) in sim::oven::baked_files() {
        let path = format!("{root}{rel}");
        std::fs::write(&path, text).unwrap_or_else(|e| panic!("could not write {rel}: {e}"));
        println!("{rel}");
    }
}
