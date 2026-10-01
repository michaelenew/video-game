//! Play a scripted hunt and report on it.
//!
//!     cargo run -p hunt --bin fight
//!     cargo run -p hunt --bin fight -- --class bulwark --trace --repeats 5
//!     cargo run -p hunt --bin fight -- --species ridgeback
//!     cargo run -p hunt --bin fight -- --temper 3 --repeats 10
//!
//! Any species with a plan (`crates/hunt/src/plans/`); the Ridgeback by
//! default. `--temper <n>` fights it at a temper (`sim::temper`), as tuned by
//! default.
//!
//! The point is not the outcome. It is the eight or nine numbers underneath
//! it, which are what turn "the fight feels off" into a thing you can point at.

use hunt::{Outcome, play_tempered};

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

    let mut killed = 0;
    let mut total = 0u32;
    for run in 0..repeats.max(1) {
        let report = play_tempered(
            species,
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
        }
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
    }
}
