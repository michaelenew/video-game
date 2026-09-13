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
        "reaction {}f  |  parry window {}f  |  dodge {}f ({} invulnerable)",
        t::HUMAN_REACTION_FRAMES,
        t::parry_window(),
        t::dodge_frames(),
        t::dodge_iframes()
    );
    // The stun family, because the hitstun and knockback columns below are what
    // a move does to someone *untouched* and are multiplied by this the rest of
    // the round. A frame table that did not say so would read as a lie by the
    // end of a match.
    let damages: Vec<i32> = ALL_CLASSES
        .iter()
        .flat_map(|c| {
            moves::table(*c)
                .iter()
                .map(|m| m.damage)
                .collect::<Vec<_>>()
        })
        .collect();
    println!(
        "stun: freeze {}-{}f by damage  |  at death knockback x{} and hitstun x{}\n",
        sim::state::hitlag_frames(*damages.iter().min().unwrap_or(&0)),
        sim::state::hitlag_frames(*damages.iter().max().unwrap_or(&0)),
        tenths(Fx::ONE.add(t::swell_knockback())),
        tenths(Fx::ONE.add(t::swell_hitstun())),
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
            "  air: jump x{}  gravity x{}  fall cap x{}  steering {}  weight x{}",
            tenths(mob.jump),
            tenths(mob.gravity),
            tenths(mob.fall_cap),
            tenths(mob.air_speed),
            tenths(mob.weight),
        );
        if class.preys_on_the_disabled() {
            println!(
                "  x{} damage to anything rooted, staggered, held or toppled",
                tenths(t::disabled_damage_mul())
            );
        }
        println!(
            "  {:<15}{:<12}{:>4}{:>5}{:>5}{:>8}{:>10}{:>8}  {:<10} notes",
            "key", "move", "st", "act", "rec", "damage", "on block", "on hit", "aimed"
        );
        for (slot, m) in (0..moves::slots(class)).map(|slot| (slot, moves::get(class, slot as u8)))
        {
            let mut notes = Vec::new();
            // A move with no volume of its own has no frame advantage worth
            // printing: those columns are all about what connecting is worth,
            // and this one never connects. Two kinds -- the ones that put
            // something in the world and let it do the hitting, and the
            // Champion's pole vault, which puts nothing anywhere.
            if !m.strikes() {
                // A move with no volume of its own has its hit delivered by
                // whatever it put in the world, and not every one of those can
                // carry a grab. Saying which is the difference between a table
                // that is incomplete and a table that is wrong: the knob is in
                // the palette, it can be turned, and on most of them nothing
                // would happen.
                let ignored = match (m.grabs > 0, sim::effects::EffectKind::from_code(m.effect)) {
                    (false, _) => "",
                    (true, Some(kind)) if kind.seizes() => "; grabs once every arm lands",
                    (true, _) => "; grab ignored -- the effect delivers the hit",
                };
                let what = if m.shape.strikes() {
                    format!("places something; the thing it placed hits{ignored}")
                } else {
                    format!("movement, no hitbox{ignored}")
                };
                println!(
                    "  {:<15}{:<12}{:>4}{:>5}{:>5}{:>8}{:>10}{:>8}  {:<10} {what}",
                    moves::binding(class, slot),
                    m.name,
                    m.startup,
                    m.active,
                    m.recovery,
                    "--",
                    "--",
                    "--",
                    // Still printed, and it matters more here than anywhere:
                    // the whole question about a move that places something is
                    // *where*, and this column is the answer.
                    m.aim().name(),
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
            // The Blood mage's whole economy, and the only class it applies to.
            let blood = if m.cost > 0 {
                format!("costs {} health, returns {}%", m.cost, m.leech)
            } else {
                String::new()
            };
            if !blood.is_empty() {
                notes.push(&blood);
            }
            // On-hit means nothing for a move that is still swinging when it
            // lands again, so it is left blank rather than printed wrong.
            let on_hit = if m.rehit > 0 {
                "--".to_string()
            } else {
                format!("{:+}", m.on_hit())
            };
            println!(
                "  {:<15}{:<12}{:>4}{:>5}{:>5}{:>8}{:>+10}{:>8}  {:<10} {}",
                moves::binding(class, slot),
                m.name,
                m.startup,
                m.active,
                m.recovery,
                m.damage,
                m.on_block(),
                on_hit,
                // Which line of effect it uses, so "where does this actually
                // go" is answerable from the table rather than from the source.
                m.aim().name(),
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
