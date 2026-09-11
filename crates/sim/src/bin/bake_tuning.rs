//! Write `crates/sim/src/tuned.rs` from whatever the Oven currently holds.
//!
//!     cargo run -p sim --bin bake_tuning
//!
//! The Oven's bake button calls the same emitter. This binary exists so the
//! file can be regenerated without launching the game, and so the very first
//! one could be produced from the values that used to be hand-written.

fn main() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/tuned.rs");
    std::fs::write(path, sim::oven::emit()).expect("could not write tuned.rs");
    println!("{path}");
}
