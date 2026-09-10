//! Print the derived frame table.
//!
//!     cargo run -p sim --bin frametable
//!
//! The raw startup/active/recovery numbers matter less than what they imply.
//! On-block advantage decides whether a move is safe to throw out, and it is
//! the number to look at when a class feels oppressive or feeble.

use sim::class::ALL_CLASSES;
use sim::moves;
use sim::tuning as t;

fn main() {
    println!(
        "reaction {}f  |  parry window {}f  |  dodge {}f ({} invulnerable)\n",
        t::HUMAN_REACTION_FRAMES,
        t::PARRY_WINDOW,
        t::DODGE_FRAMES,
        t::DODGE_IFRAMES
    );

    for class in ALL_CLASSES {
        println!("{}  --  spends {}", class.name(), class.resource());
        println!(
            "  {:<16}{:>4}{:>5}{:>5}{:>8}{:>10}{:>8}   notes",
            "move", "st", "act", "rec", "damage", "on block", "on hit"
        );
        for m in moves::table(class) {
            let mut notes = Vec::new();
            if m.unblockable {
                notes.push("unblockable");
            }
            if !m.hits_crouching {
                notes.push("overhead");
            }
            if m.needs_mechanic {
                notes.push("needs mechanic");
            }
            if m.startup < t::HUMAN_REACTION_FRAMES {
                notes.push("unreactable");
            }
            println!(
                "  {:<16}{:>4}{:>5}{:>5}{:>8}{:>+10}{:>+8}   {}",
                m.name,
                m.startup,
                m.active,
                m.recovery,
                m.damage,
                m.on_block(),
                m.on_hit(),
                notes.join(", ")
            );
        }
        println!();
    }

    println!(
        "On block is the safety number: negative means punishable, and every move\n\
         should be. Record what you change in docs/design/feel-log.md."
    );
}
