//! Play a scripted hunt and report on it.
//!
//!     cargo run -p hunt --bin fight
//!     cargo run -p hunt --bin fight -- --class bulwark --trace --repeats 5
//!     cargo run -p hunt --bin fight -- --species ridgeback
//!     cargo run -p hunt --bin fight -- --temper 3 --repeats 10
//!     cargo run -p hunt --bin fight -- --species hornback --arena crossing
//!     cargo run -p hunt --bin fight -- --species mireback --gamble
//!
//! Any species with a plan (`crates/hunt/src/plans/`); the Ridgeback by
//! default. `--temper <n>` fights it at a temper (`sim::temper`), as tuned by
//! default. `--arena <name>` fights it somewhere other than its own arena:
//! the Hornback's crossing is `--arena crossing`. `--gamble` plays the card's
//! second plan, the one that takes a risk the first will not (the Mireback's
//! `swallow_greed`), where it has one.
//!
//! The point is not the outcome. It is the eight or nine numbers underneath
//! it, which are what turn "the fight feels off" into a thing you can point at.

use hunt::{Outcome, play_card_in};

fn arg(flag: &str) -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == flag {
            return args.next();
        }
    }
    None
}

fn has(flag: &str) -> bool {
    std::env::args().any(|a| a == flag)
}

fn main() {
    let class = arg("--class")
        .and_then(|n| {
            sim::class::ALL_CLASSES
                .iter()
                .copied()
                .find(|c| c.name().to_lowercase().contains(&n.to_lowercase()))
        })
        .unwrap_or(sim::Class::Champion);
    let species = match arg("--species") {
        Some(name) => match sim::species::named(&name) {
            Some(s) if hunt::plans::card(s.id).is_some() => s.id,
            _ => {
                eprintln!("no species with a hunter plan is called {name}");
                std::process::exit(2);
            }
        },
        None => sim::species::SpeciesId::RIDGEBACK,
    };
    let arena = arg("--arena").map(|name| match sim::arena::named(&name) {
        Some(a) => a.id,
        None => {
            eprintln!("no arena is called {name}");
            std::process::exit(2);
        }
    });
    let temper: u8 = arg("--temper")
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
        .min(sim::temper::HIGHEST);
    let repeats: u32 = arg("--repeats").and_then(|n| n.parse().ok()).unwrap_or(1);
    let partners: usize = arg("--hunters").and_then(|n| n.parse().ok()).unwrap_or(1);
    let limit: u32 = arg("--frames")
        .and_then(|n| n.parse().ok())
        .unwrap_or(72_000);
    let seed: u32 = arg("--seed")
        .and_then(|n| n.parse().ok())
        .unwrap_or(0x2545_F491);

    let mut card = hunt::plans::card(species).expect("checked above");
    if has("--gamble") {
        match card.gamble {
            // A card lives for the program; this one is made once.
            Some(plan) => card = Box::leak(Box::new(hunt::plans::Card { plan, ..*card })),
            None => {
                eprintln!(
                    "{} has no second plan to gamble with",
                    card.species.get().name
                );
                std::process::exit(2);
            }
        }
    }

    let mut killed = 0;
    let mut total = 0u32;
    // Across the runs: the four windows, what the hunters kept, what was
    // unanswerable, and every move's thrown and landed.
    let mut windows = [0f32; 4];
    let mut kept = 0i32;
    let mut kept_won = 0i32;
    let mut unanswerable = 0u32;
    let mut thrown = [0u32; sim::species::MAX_MOVES];
    let mut landed = [0u32; sim::species::MAX_MOVES];
    let mut names: Vec<&'static str> = Vec::new();
    for run in 0..repeats.max(1) {
        let report = play_card_in(
            card,
            arena,
            temper,
            [class; sim::state::MAX_PLAYERS],
            partners,
            limit,
            seed.wrapping_add(run.wrapping_mul(0x9E37_79B9)),
            |_| {},
        );
        if repeats == 1 {
            println!("{}", report.render());
            if has("--trace") {
                println!("{}", report.trace());
            }
        } else {
            println!(
                "  run {run}: {:?} in {:.0}s, {} rides, {} topples, {} unanswerable, {} health left",
                report.outcome,
                report.frames as f32 / 60.0,
                report.rides,
                report.topples,
                report.unanswerable,
                report.health_left
            );
        }
        if let Outcome::Killed(f) = report.outcome {
            killed += 1;
            total += f;
            kept_won += report.health_left;
        }
        for (t, w) in [
            hunt::report::Threat::Threatening,
            hunt::report::Threat::PokeOnly,
            hunt::report::Threat::Skilled,
            hunt::report::Threat::WalkUp,
        ]
        .iter()
        .zip(windows.iter_mut())
        {
            *w += report.threat_share(*t);
        }
        kept += report.health_left;
        unanswerable += report.unanswerable;
        for k in 0..report.species.moves.len() {
            thrown[k] += report.starts[k];
            landed[k] += report.landed[k];
        }
        names = report.species.moves.iter().map(|m| m.name).collect();
    }
    if repeats > 1 {
        println!(
            "\n  {killed}/{repeats} hunts won{}",
            if killed > 0 {
                format!(", mean {:.0}s", total as f32 / killed as f32 / 60.0)
            } else {
                String::new()
            }
        );
        let n = repeats.max(1) as f32;
        println!(
            "  windows: threatening {:.0}%  poke {:.0}%  way in {:.0}%  walk up {:.0}%",
            windows[0] / n * 100.0,
            windows[1] / n * 100.0,
            windows[2] / n * 100.0,
            windows[3] / n * 100.0
        );
        println!(
            "  health left: {:.0} a hunt, {:.0} a win;  unanswerable: {unanswerable}",
            kept as f32 / n,
            if killed > 0 {
                kept_won as f32 / killed as f32
            } else {
                0.0
            }
        );
        let moves: Vec<String> = names
            .iter()
            .enumerate()
            .map(|(k, name)| format!("{name} {}/{}", landed[k], thrown[k]))
            .collect();
        println!("  landed/thrown: {}", moves.join(", "));
    }
}
