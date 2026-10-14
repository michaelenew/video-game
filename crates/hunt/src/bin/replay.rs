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
//!
//!     cargo run -p hunt --bin replay -- <file> --motion
//!     cargo run -p hunt --bin replay -- <file> --motion 160850..160950
//!
//! `--motion` adds how the fighters moved: every frame a walk was stopped
//! dead, a foot left the floor with no jump, or a body rose faster than two
//! jumps -- with where on the map -- and, given a frame or a range, a line
//! per frame of where each fighter was, how fast, and on what.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let trace = args.iter().any(|a| a == "--trace");
    let at = args.iter().position(|a| a == "--motion");
    let moving = at.is_some();
    // The word after `--motion`, if it is a frame or a range of them.
    let window = at.and_then(|i| args.get(i + 1)).and_then(|w| {
        let (a, b) = w.split_once("..").unwrap_or((w, w));
        Some((a.parse::<u32>().ok()?, b.parse::<u32>().ok()?))
    });
    let path = args.iter().enumerate().find(|(i, a)| {
        !a.starts_with("--") && !(window.is_some() && at.is_some_and(|m| *i == m + 1))
    });
    let Some((_, path)) = path else {
        eprintln!("usage: replay <file> [--trace] [--motion [<from>[..<to>]]]");
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
    match hunt::replay::judge_moving(&tape, window) {
        Ok(judgement) => {
            print!("{}", judgement.render_moving(&tape, trace, moving));
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
