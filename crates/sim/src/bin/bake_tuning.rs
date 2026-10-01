//! Write every baked tuning file from whatever the Oven currently holds:
//! `crates/sim/src/tuned.rs`, and each species' own
//! (`crates/sim/src/species/<species>/tuned.rs`).
//!
//!     cargo run -p sim --bin bake_tuning
//!     cargo run -p sim --bin bake_tuning -- --set gnats.pack.ring_radius=4.5 --set ...
//!
//! The Oven's bake button calls the same emitters. This binary exists so the
//! files can be regenerated without launching the game, and so the very first
//! one could be produced from the values that used to be hand-written.
//!
//! **`--set <id>=<value>`** sets a knob before baking, by the identifier its
//! line in the baked file carries as a comment, in the unit the comment shows
//! it in (metres as a decimal, frames and counts as integers, `on`/`off` for a
//! flag). It is the palette's slider from a terminal: what a new species uses
//! to set its first numbers without a window, and the same thing as dragging
//! them in F7 and pressing Bake. Repeat it for as many knobs as you like.

use sim::oven::{Knob, Unit, all_knobs};

/// Read a value the way `Unit::show` writes it, with integer arithmetic only.
fn parse(unit: Unit, text: &str) -> Option<i32> {
    match unit {
        Unit::Flag => match text {
            "on" | "1" | "true" => Some(1),
            "off" | "0" | "false" => Some(0),
            _ => None,
        },
        Unit::Fixed => {
            let (neg, body) = match text.strip_prefix('-') {
                Some(rest) => (true, rest),
                None => (false, text),
            };
            let (whole, frac) = body.split_once('.').unwrap_or((body, ""));
            let whole: i64 = if whole.is_empty() {
                0
            } else {
                whole.parse().ok()?
            };
            let mut raw = whole * 65536;
            if !frac.is_empty() {
                let digits: i64 = frac.parse().ok()?;
                let scale = 10i64.pow(frac.len() as u32);
                raw += (digits * 65536 + scale / 2) / scale;
            }
            Some(if neg { -raw } else { raw } as i32)
        }
        _ => text.parse().ok(),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let knobs: Vec<Knob> = all_knobs();
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--set" {
            let Some((id, value)) = args.get(i + 1).and_then(|a| a.split_once('=')) else {
                eprintln!("--set wants <id>=<value>");
                std::process::exit(2);
            };
            let Some(knob) = knobs.iter().find(|k| k.id() == id) else {
                eprintln!("no knob called {id}");
                std::process::exit(2);
            };
            let Some(raw) = parse(knob.unit(), value) else {
                eprintln!("{id}: cannot read {value:?} as {:?}", knob.unit());
                std::process::exit(2);
            };
            knob.set_raw(raw);
            i += 2;
        } else {
            eprintln!("unknown argument {}", args[i]);
            std::process::exit(2);
        }
    }
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../");
    for (rel, text) in sim::oven::baked_files() {
        let path = format!("{root}{rel}");
        std::fs::write(&path, text).unwrap_or_else(|e| panic!("could not write {rel}: {e}"));
        println!("{rel}");
    }
}
