//! Print everything you can type.
//!
//!     cargo run -p manual
//!
//! No dependencies, so this answers quickly. The game's `--help` prints the
//! same text from the same tables.

fn main() {
    print!("{}", manual::render());
}
