//! Play the sparring bot against itself and count what happens.
//!
//!     cargo run -p hunt --bin duel
//!     cargo run -p hunt --bin duel -- --p1 champion --p2 bulwark --level hard
//!     cargo run -p hunt --bin duel -- --level hard --against easy
//!     cargo run -p hunt --bin duel -- --kits
//!
//! Without classes it plays every pairing once. The numbers to look at are
//! whether both sides throw, land and dodge at all, and whether the rounds
//! split rather than one side winning everything -- a bot that beats its own
//! mirror nine times in ten has found a loop, not a fight.

use hunt::Level;
use hunt::duel::{Kit, spar, spar_traced};
use sim::class::ALL_CLASSES;

fn arg(flag: &str) -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == flag {
            return args.next();
        }
    }
    None
}

fn class(name: &str) -> Option<sim::Class> {
    ALL_CLASSES
        .iter()
        .copied()
        .find(|c| c.name().to_lowercase().contains(&name.to_lowercase()))
}

fn level(name: &str) -> Level {
    Level::ALL
        .iter()
        .copied()
        .find(|l| l.name() == name)
        .unwrap_or(Level::Normal)
}

fn main() {
    if std::env::args().any(|a| a == "--kits") {
        for c in ALL_CLASSES {
            let kit = Kit::learn(c);
            let tools: Vec<String> = kit
                .iter()
                .map(|t| {
                    let m = sim::moves::get(c, t.kind);
                    format!(
                        "{:#06x}->{} ({}f, {}, {:.1}m)",
                        t.bits,
                        m.name,
                        m.startup,
                        m.aim().name(),
                        m.reach.to_f32_for_render()
                    )
                })
                .collect();
            println!("{:14} {}", c.name(), tools.join("  "));
        }
        return;
    }
    let lv = level(&arg("--level").unwrap_or_default());
    // `--against easy` gives player two a different level from player one.
    let lv2 = arg("--against").map(|n| level(&n)).unwrap_or(lv);
    let frames: u32 = arg("--frames")
        .and_then(|n| n.parse().ok())
        .unwrap_or(60 * 180);
    let seed: u32 = arg("--seed")
        .and_then(|n| n.parse().ok())
        .unwrap_or(0x2545_F491);
    let pairs: Vec<(sim::Class, sim::Class)> = match (
        arg("--p1").and_then(|n| class(&n)),
        arg("--p2").and_then(|n| class(&n)),
    ) {
        (Some(a), Some(b)) => vec![(a, b)],
        (Some(a), None) => ALL_CLASSES.iter().map(|b| (a, *b)).collect(),
        _ => ALL_CLASSES
            .iter()
            .flat_map(|a| ALL_CLASSES.iter().map(move |b| (*a, *b)))
            .filter(|(a, b)| (*a as usize) <= (*b as usize))
            .collect(),
    };
    println!(
        "  {} against {}, {}s each\n\n  {:14} {:14} {:>7} {:>13} {:>13} {:>11} {:>11}",
        lv.name(),
        lv2.name(),
        frames / 60,
        "p1",
        "p2",
        "rounds",
        "thrown",
        "landed",
        "dodges",
        "mechanic"
    );
    let tracing = std::env::args().any(|a| a == "--trace");
    for (a, b) in pairs {
        let bout = if tracing {
            spar_traced([a, b], [lv, lv2], frames, seed, |w, bots| {
                if w.frame % 30 != 0 {
                    return;
                }
                let d = w.players[0].pos.sub(w.players[1].pos).flat_len();
                println!(
                    "  {:>5}  {:4.1}m ({:5.1},{:4.1},{:5.1}) ({:5.1},{:4.1},{:5.1}) {:>4} {:>4}  {:?}/{:?}  {:?} | {:?}",
                    w.frame,
                    d.to_f32_for_render(),
                    w.players[0].pos.x.to_f32_for_render(),
                    w.players[0].pos.y.to_f32_for_render(),
                    w.players[0].pos.z.to_f32_for_render(),
                    w.players[1].pos.x.to_f32_for_render(),
                    w.players[1].pos.y.to_f32_for_render(),
                    w.players[1].pos.z.to_f32_for_render(),
                    w.players[0].health,
                    w.players[1].health,
                    bots[0].plan,
                    bots[1].plan,
                    w.players[0].action,
                    w.players[1].action,
                );
            })
        } else {
            spar([a, b], [lv, lv2], frames, seed)
        };
        println!(
            "  {:14} {:14} {:>3}-{:<3} {:>6}/{:<6} {:>6}/{:<6} {:>5}/{:<5} {:>5}/{:<5}",
            a.name(),
            b.name(),
            bout.rounds[0],
            bout.rounds[1],
            bout.thrown[0],
            bout.thrown[1],
            bout.landed[0],
            bout.landed[1],
            bout.dodges[0],
            bout.dodges[1],
            bout.mechanic[0],
            bout.mechanic[1],
        );
    }
}
