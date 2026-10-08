//! Judge a fight somebody played.
//!
//!     cargo run -p hunt --bin replay -- <file>
//!     cargo run -p hunt --bin replay -- <file> --trace
//!
//! The file is a replay the game saved (the Esc menu's *Save replay*, or
//! `--record`). It is put back through the simulation, checked against the
//! frame and checksum the game ended on, and judged by the creature's report
//! -- the one `fight` prints for the scripted hunter -- with a table of what
//! each fighter threw beside it. `--trace` adds the play sequence.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let trace = args.iter().any(|a| a == "--trace");
    let Some(path) = args.iter().find(|a| !a.starts_with("--")) else {
        eprintln!("usage: replay <file> [--trace]");
        std::process::exit(2);
    };
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("could not read {path}: {e}");
            std::process::exit(2);
        }
    };
    let tape = match sim::replay::Tape::from_text(&text) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{path}: {e}");
            std::process::exit(2);
        }
    };
    match hunt::replay::judge(&tape) {
        Ok(judgement) => {
            print!("{}", judgement.render(&tape, trace));
            if judgement.matched == Some(false) {
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("{path}: {e}");
            std::process::exit(1);
        }
    }
}
