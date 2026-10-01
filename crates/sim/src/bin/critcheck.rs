//! Can every move touch something short?
//!
//!     cargo run -p sim --bin critcheck
//!     cargo run -p sim --bin critcheck -- --species gnats --kind queen
//!
//! Stands one critter at 1, 2, 3 and 5 m in front of each class, puts the
//! crosshair through its middle, presses every move, and prints where it
//! touched -- by the fight's own answer, and by the volume the overlay draws.
//! The dev pack's gnat (0.6 m) by default; any registered pack and any of its
//! kinds by name. See `sim::critcheck`, and `docs/design/critters.md`.

use sim::critcheck::{METRES, table};
use sim::critter::{CritterField, stat_fx};
use sim::oven::Unit;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let wanted = arg("--species").unwrap_or_else(|| "gnats".to_string());
    let Some(sp) = sim::species::named(&wanted).filter(|s| s.pack.is_some()) else {
        let packs: Vec<&str> = sim::species::all()
            .filter(|s| s.pack.is_some())
            .map(|s| s.name)
            .collect();
        eprintln!("no pack called {wanted}; there is {}", packs.join(", "));
        std::process::exit(2);
    };
    let kinds = sp.pack.map_or(&[][..], |p| p.kinds);
    let kind = match arg("--kind") {
        Some(name) => match kinds
            .iter()
            .position(|k| k.name.eq_ignore_ascii_case(&name))
        {
            Some(i) => i as u8,
            None => {
                eprintln!("{} has no kind called {name}", sp.name);
                std::process::exit(2);
            }
        },
        None => 0,
    };
    let show = |v: sim::Fx| Unit::Fixed.show(v.raw());
    println!(
        "{} {}: crown {} m, {} m long, {} m wide; standing, crosshair on its middle\n",
        sp.name,
        kinds[kind as usize].name,
        show(stat_fx(sp, kind, CritterField::Height)),
        show(stat_fx(sp, kind, CritterField::Length)),
        show(stat_fx(sp, kind, CritterField::Width)),
    );
    let rows = table(sp.id, kind);
    let mut misses = Vec::new();
    for row in &rows {
        let at = |pick: fn(&sim::critcheck::Trial) -> bool| {
            let hit: Vec<String> = row
                .trials
                .iter()
                .zip(METRES)
                .filter(|(t, _)| pick(t))
                .map(|(_, m)| m.to_string())
                .collect();
            if hit.is_empty() {
                "-".to_string()
            } else {
                hit.join(" ")
            }
        };
        let over: Vec<String> = row.over().map(|m| m.to_string()).collect();
        let ran: Vec<String> = row.ran_through().map(|m| m.to_string()).collect();
        let note = if !row.is_an_attack() {
            "  (no damage)".to_string()
        } else if !ran.is_empty() && over.is_empty() {
            format!("  (runs through it at {} m)", ran.join(" "))
        } else if !over.is_empty() {
            misses.push(format!(
                "{} {} ({} m)",
                row.class.name(),
                row.name,
                over.join(" ")
            ));
            format!("  << over it at {} m", over.join(" "))
        } else {
            String::new()
        };
        println!(
            "{:<14} {:<15} {:<10} fighter {:<8} critter {:<8} drawn {:<8}{}",
            row.class.name(),
            row.name,
            row.aim.name(),
            at(|t| t.fighter),
            at(|t| t.hurt),
            at(|t| t.drawn),
            note
        );
    }
    println!();
    if misses.is_empty() {
        println!("every move touches it wherever it touches a fighter: yes");
    } else {
        println!(
            "passes over it where a fighter is hit: {}",
            misses.join(", ")
        );
    }
}
