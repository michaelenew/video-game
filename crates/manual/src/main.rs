//! Print everything you can type.
//!
//!     cargo run -p manual
//!     cargo run -p manual -- --html
//!
//! No dependencies, so this answers quickly. The game's `--help` prints the
//! same text from the same tables, and `--html` prints the browser build's
//! controls panel from them as well -- see `crates/web/build-game.sh`, which
//! is the only caller of it.

fn main() {
    if std::env::args().any(|a| a == "--html") {
        print!("{}", manual::browser_help());
    } else {
        print!("{}", manual::render());
    }
}
