//! The Broodmother's dilemma, measured: her §9's three plans on the same
//! seeds.
//!
//!     cargo run -p hunt --bin brood -- --class champion --repeats 12
//!     cargo run -p hunt --bin brood -- --all --repeats 12
//!
//! **Balanced** is the plan a decent player follows; **brood only** never
//! touches a sac or her while a broodling stands; **mother only** never
//! swings at a broodling or a sac. The fight's sentence -- every second on
//! the little ones she makes more, every second ignoring them they eat you --
//! is a claim that both extremes lose, so the balanced plan has to win
//! clearly more than either: mother-only fast, brood-only slow. Beside each
//! plan, the report's own lines summed over the runs (§9).

use hunt::plans::broodmother::{self as plan, Lines};
use hunt::report::{Tally, Threat};
use hunt::{Outcome, play_card_in};
use sim::World;

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

struct Sum {
    won: u32,
    won_frames: u32,
    lost_frames: u32,
    kept_won: i32,
    windows: [f32; 4],
    unanswerable: u32,
    lines: Lines,
    runs: u32,
}

fn run(
    card: &'static hunt::plans::Card,
    class: sim::Class,
    repeats: u32,
    seed: u32,
    hunters: usize,
) -> Sum {
    let mut sum = Sum {
        won: 0,
        won_frames: 0,
        lost_frames: 0,
        kept_won: 0,
        windows: [0.0; 4],
        unanswerable: 0,
        lines: Lines::default(),
        runs: repeats,
    };
    for r in 0..repeats {
        let mut prev: Option<World> = None;
        let mut lines = std::mem::take(&mut sum.lines);
        let report = play_card_in(
            card,
            None,
            0,
            [class; sim::state::MAX_PLAYERS],
            hunters,
            72_000,
            seed.wrapping_add(r.wrapping_mul(0x9E37_79B9)),
            |w| {
                if let Some(p) = prev.as_ref() {
                    lines.observe(p, w);
                }
                prev = Some(w.clone());
            },
        );
        sum.lines = lines;
        match report.outcome {
            Outcome::Killed(f) => {
                sum.won += 1;
                sum.won_frames += f;
                sum.kept_won += report.health_left;
            }
            _ => sum.lost_frames += report.frames,
        }
        for (t, w) in [
            Threat::Threatening,
            Threat::PokeOnly,
            Threat::Skilled,
            Threat::WalkUp,
        ]
        .iter()
        .zip(sum.windows.iter_mut())
        {
            *w += report.threat_share(*t);
        }
        sum.unanswerable += report.unanswerable;
    }
    sum
}

fn main() {
    let classes: Vec<sim::Class> = if has("--all") {
        sim::class::ALL_CLASSES.to_vec()
    } else {
        vec![
            arg("--class")
                .and_then(|n| {
                    sim::class::ALL_CLASSES
                        .iter()
                        .copied()
                        .find(|c| c.name().to_lowercase().contains(&n.to_lowercase()))
                })
                .unwrap_or(sim::Class::Champion),
        ]
    };
    let repeats: u32 = arg("--repeats").and_then(|n| n.parse().ok()).unwrap_or(12);
    let seed: u32 = arg("--seed")
        .and_then(|n| n.parse().ok())
        .unwrap_or(0x2545_F491);
    let hunters: usize = arg("--hunters").and_then(|n| n.parse().ok()).unwrap_or(1);
    let plans: Vec<(&str, &'static hunt::plans::Card)> = if has("--balanced") {
        vec![("balanced", &plan::CARD)]
    } else {
        vec![
            ("balanced", &plan::CARD),
            ("brood only", &plan::BROOD_ONLY),
            ("mother only", &plan::MOTHER_ONLY),
        ]
    };
    for class in classes {
        println!("{}", class.name());
        for (name, card) in &plans {
            let s = run(card, class, repeats, seed, hunters);
            let n = s.runs.max(1) as f32;
            let lost = s.runs - s.won;
            println!(
                "  {name:12} won {:2}/{}  win {:>4}s  loss {:>4}s  kept {:>4}  thr {:.0}/{:.0}/{:.0}/{:.0}%  unans {}",
                s.won,
                s.runs,
                if s.won > 0 {
                    format!("{:.0}", s.won_frames as f32 / s.won as f32 / 60.0)
                } else {
                    "--".into()
                },
                if lost > 0 {
                    format!("{:.0}", s.lost_frames as f32 / lost as f32 / 60.0)
                } else {
                    "--".into()
                },
                if s.won > 0 {
                    s.kept_won / s.won as i32
                } else {
                    0
                },
                s.windows[0] / n * 100.0,
                s.windows[1] / n * 100.0,
                s.windows[2] / n * 100.0,
                s.windows[3] / n * 100.0,
                s.unanswerable,
            );
            if !has("--quiet") {
                for (k, v, _) in s.lines.lines() {
                    println!("      {k:22} {v}");
                }
            }
        }
    }
}
