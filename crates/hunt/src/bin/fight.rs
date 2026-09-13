//! Play a scripted hunt and report on it.
//!
//!     cargo run -p hunt --bin fight
//!     cargo run -p hunt --bin fight -- --class bulwark --trace --repeats 5
//!
//! The point is not the outcome. It is the eight or nine numbers underneath
//! it, which are what turn "the fight feels off" into a thing you can point at.

use hunt::{Outcome, play};

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
        let report = play(
            [class; sim::state::MAX_PLAYERS],
            partners,
            limit,
            seed.wrapping_add(run.wrapping_mul(0x9E37_79B9)),
        );
        if repeats == 1 {
            println!("{}", report.render());
            if has("--trace") {
                println!("{}", report.trace());
            }
        } else {
            println!(
                "  run {run}: {:?} in {:.0}s, {} rides, {} topples, {} unanswerable",
                report.outcome,
                report.frames as f32 / 60.0,
                report.rides,
                report.topples,
                report.unanswerable
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
