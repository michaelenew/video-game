//! Print the derived frame table.
//!
//!     cargo run -p sim --bin frametable
//!
//! The raw startup/active/recovery numbers matter less than what they imply.
//! On-block advantage decides whether a move is safe to throw out, and it is
//! the number to look at when a class feels oppressive or feeble.

use sim::Fx;
use sim::class::ALL_CLASSES;
use sim::moves;
use sim::tuning as t;

fn main() {
    // Movement speeds, because how much a move takes your feet away is part of
    // the same tuning surface as its frame data.
    //
    // Formatted with integer arithmetic rather than a cast to `f32`: this file
    // lives under `crates/sim/src`, and the no-floats guard covers all of it.
    // That is the guard working, not the guard being awkward -- the rule is
    // worth more than a convenient `{:.1}`.
    println!(
        "walk {}  |  poking {}  |  crouching {}  |  guarding {}  |  committed 0",
        tenths(t::move_speed()),
        tenths(t::move_speed().mul(Fx::ratio(t::poke_mobility() as i32, 100))),
        tenths(t::crouch_move_speed()),
        tenths(t::guard_move_speed()),
    );
    println!(
        "reaction {}f  |  parry window {}f  |  dodge {}f ({} invulnerable)\n",
        t::HUMAN_REACTION_FRAMES,
        t::parry_window(),
        t::dodge_frames(),
        t::dodge_iframes()
    );

    for class in ALL_CLASSES {
        let (short_apex, short_time) = jump_shape(class, 1);
        let (full_apex, full_time) = jump_shape(class, 60);
        let mob = class.mobility();
        println!("{}  --  spends {}", class.name(), class.resource());
        println!(
            "  jump: short {}m {}f  |  full {}m {}f  |  {} body heights",
            tenths(short_apex),
            short_time,
            tenths(full_apex),
            full_time,
            tenths(full_apex.div(t::body_height())),
        );
        println!(
            "  air: jump x{}  gravity x{}  fall cap x{}  steering {}",
            tenths(mob.jump),
            tenths(mob.gravity),
            tenths(mob.fall_cap),
            tenths(mob.air_speed),
        );
        println!(
            "  {:<16}{:>4}{:>5}{:>5}{:>8}{:>10}{:>8}   notes",
            "move", "st", "act", "rec", "damage", "on block", "on hit"
        );
        for m in moves::table(class) {
            let mut notes = Vec::new();
            // A move with no hit volume has no frame advantage worth printing:
            // the columns are all about what connecting is worth, and it never
            // connects. The Champion's pole vault is the only one.
            if !m.strikes() {
                println!(
                    "  {:<16}{:>4}{:>5}{:>5}{:>8}{:>10}{:>8}   movement, no hitbox",
                    m.name, m.startup, m.active, m.recovery, "--", "--", "--"
                );
                continue;
            }
            if m.rehit > 0 {
                notes.push("re-hits");
            }
            if m.unblockable {
                notes.push("unblockable");
            }
            if !m.hits_crouching {
                notes.push("overhead");
            }
            if m.needs_mechanic {
                notes.push("needs mechanic");
            }
            if m.air_stall > 0 {
                notes.push("hangs");
            }
            if m.roots() {
                notes.push("roots you");
            }
            if m.startup < t::HUMAN_REACTION_FRAMES {
                notes.push("unreactable");
            }
            // On-hit means nothing for a move that is still swinging when it
            // lands again, so it is left blank rather than printed wrong.
            let on_hit = if m.rehit > 0 {
                "--".to_string()
            } else {
                format!("{:+}", m.on_hit())
            };
            println!(
                "  {:<16}{:>4}{:>5}{:>5}{:>8}{:>+10}{:>8}   {}",
                m.name,
                m.startup,
                m.active,
                m.recovery,
                m.damage,
                m.on_block(),
                on_hit,
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

/// One decimal place, without touching floating point.
fn tenths(v: Fx) -> String {
    // Rounded, not truncated: 4.199 should read as 4.2, not 4.1.
    let t = (v.raw() as i64 * 10 + (1 << 15)) >> 16;
    format!("{}.{}", t / 10, (t % 10).abs())
}

/// Apex and airtime for a jump held `hold` frames.
///
/// Simulated rather than derived: gravity is per class, the sustain window
/// caps the hold, and terminal velocity clips the fall, so the closed form
/// would be a lie in three different places.
fn jump_shape(class: sim::class::Class, hold: u32) -> (Fx, u32) {
    let mut w = sim::World::with_classes([class, class]);
    let mut apex = Fx::ZERO;
    for i in 0..300u32 {
        let bits = if i < hold { sim::Input::SPACE } else { 0 };
        w.advance([sim::Input::new(bits), sim::Input::default()]);
        let y = w.players[0].pos.y;
        if y.raw() > apex.raw() {
            apex = y;
        }
        if i > 0 && w.players[0].grounded {
            return (apex, i);
        }
    }
    (apex, 0)
}
