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
        "walk {}  |  poking {}  |  crouching {}  |  guarding {}  |  committed {}",
        tenths(t::move_speed()),
        tenths(walking_at(t::poke_mobility())),
        tenths(t::crouch_move_speed()),
        tenths(t::guard_move_speed()),
        tenths(walking_at(t::committed_mobility())),
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
        // Health beside the jump line, because the two are the most legible
        // differences a body can have. See `tuning::health_of`.
        println!(
            "  health {} (x{} of {})",
            t::health_of(class),
            hundredths(Fx::ratio(t::health_of(class), t::max_health())),
            t::max_health(),
        );
        // The Reaver's tally, and what cashing a full one is worth with each of
        // her swings -- beside the rest of her numbers because the cash-in is
        // a multiplier on moves listed below. See `sim::shadow`.
        if class == sim::class::Class::ShadowReaver {
            let full = sim::shadow::cash_multiple(t::mark_cap());
            let swings: Vec<String> = sim::moves::table(class)
                .iter()
                .filter(|m| m.aim() == sim::aim::Kind::Swing && m.damage > 0)
                .map(|m| format!("{} {}", m.name, Fx::from_int(m.damage).mul(full).to_int()))
                .collect();
            println!(
                "  tally: cap {}, one fades every {}f  |  any hit of hers cashes; full x{}: {}, and staggers {}f",
                t::mark_cap(),
                t::mark_fade(),
                hundredths(full),
                swings.join(", "),
                t::cash_stagger(),
            );
            println!(
                "  copy: {}% from the field, {}% at her heel  |  turns to a body within reach + {}m",
                percent(t::shadow_echo()),
                percent(t::shadow_echo_attending()),
                tenths(t::shadow_aim_slack()),
            );
        }
        // The Dual mage's tiers: what her body gains as the lower of her two
        // bars rises, beside the jump line because two of the three are jumps.
        // See `sim::dual`.
        if class == sim::class::Class::DualMage {
            println!(
                "  tiers, on the lower bar of {}: blink at {}  |  second jump x{} and fall x{} at {}  |  wings at {}",
                t::meter_max(),
                t::tier_blink(),
                tenths(t::second_jump()),
                tenths(t::slow_fall()),
                t::tier_jump(),
                t::tier_wings(),
            );
            println!(
                "  the hill: band {}  |  drift {}/s per unit outside it, at most {}/s  |  burn {}/s per unit, at most {}/s  |  calm {}/s",
                t::meter_band(),
                tenths(t::drift_gain()),
                tenths(t::drift_cap()),
                tenths(t::burn_gain()),
                tenths(t::burn_cap()),
                tenths(t::meter_calm()),
            );
            println!(
                "  goads: an auto {}, a cast {}, the finisher {}  |  ascension {}f, drains {} a frame, {} back a hit, stagger {}-{}f",
                t::meter_auto_push(),
                t::meter_cast_push(),
                t::meter_finisher_push(),
                t::ascension_frames(),
                t::ascension_drain(),
                t::ascension_refund(),
                t::ascension_stun_floor(),
                t::ascension_stun(),
            );
        }
        if class.preys_on_the_disabled() {
            println!(
                "  x{} damage to anything staggered, held or toppled",
                tenths(t::disabled_damage_mul())
            );
        }
        println!(
            "  {:<15}{:<12}{:>4}{:>5}{:>5}{:>8}{:>10}{:>8}{:>6}  {:<10} notes",
            "key", "move", "st", "act", "rec", "damage", "on block", "on hit", "lock", "aimed"
        );
        for (slot, m) in (0..moves::slots(class)).map(|slot| (slot, moves::get(class, slot as u8)))
        {
            let mut notes = Vec::new();
            // What repeating it actually costs. The lockout column on its own
            // does not say: it is measured from the frame the move comes out,
            // so on anything that commits you for longer than it lasts the
            // answer is "nothing", and the only moves it charges are the ones
            // cheap enough to have frames left over.
            //
            // Worked out above the branch below rather than inside it, because
            // the two moves with the most to say here -- Send shadow and the
            // Guillotine, the two that are used twice -- are both moves that
            // place something and take the other path out.
            let repeat = if moves::reactivates(class, slot as u8) {
                format!(
                    "press again to bring it back; then {}f before another",
                    m.repeat_lock()
                )
            } else if moves::lingers(class, slot as u8) {
                format!("{}f after the last of it comes home", m.repeat_lock())
            } else if m.repeat_idle() > 0 {
                format!("+{}f to throw again", m.repeat_idle())
            } else {
                String::new()
            };
            // A move with no volume of its own has no frame advantage worth
            // printing: those columns are all about what connecting is worth,
            // and this one never connects. Three kinds -- the ones that leave
            // something standing where they were cast, the ones that *throw*
            // something that travels, and the Champion's pole vault, which
            // puts nothing anywhere at all.
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
                // The one column the startup does not answer for a channelled
                // move: how long the button buys, and what it buys.
                let wound = if m.channels() {
                    format!(
                        "; channel up to {}f for {}-{} m",
                        m.channel,
                        tenths(m.channel_from),
                        tenths(m.reach)
                    )
                } else {
                    String::new()
                };
                // **Does it leave something behind**, rather than what shape
                // its own body puts out. The two used to be the same question,
                // because the only way to say "no volume of my own" was a
                // radius of zero; `Shape::None` then meant pure movement. The
                // Dual mage's dark Lance says it the other way -- no shape at
                // all, and an effect that does every bit of the work -- and was
                // printed as "movement, no hitbox", which is a table being
                // confidently wrong about an ability that tethers people.
                let leaves = sim::effects::EffectKind::from_code(m.effect).is_some();
                let base = if sim::gust::Gale::thrown_by(class, slot as u8).is_some() {
                    format!("throws something; the thing it threw hits{ignored}{wound}")
                } else if leaves || m.shape.strikes() {
                    format!("places something; the thing it placed hits{ignored}{wound}")
                } else {
                    format!("movement, no hitbox{ignored}{wound}")
                };
                let what = if repeat.is_empty() {
                    base
                } else {
                    format!("{base}; {repeat}")
                };
                println!(
                    "  {:<15}{:<12}{:>4}{:>5}{:>5}{:>8}{:>10}{:>8}{:>6}  {:<10} {what}",
                    moves::binding(class, slot),
                    m.name,
                    m.startup,
                    m.active,
                    m.recovery,
                    "--",
                    "--",
                    "--",
                    // The lockout is real on these even though the advantage
                    // columns are not: a move that places something is still a
                    // move you can lean on the button for.
                    m.repeat_lock(),
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
            // What the move costs your feet, and only when it is worth
            // saying: every move hinders you, and the committed ones hinder
            // you down to a crawl.
            let feet = if m.mobility == 0 {
                "roots you".to_string()
            } else {
                format!("walks {}", tenths(walking_at(m.mobility)))
            };
            notes.push(&feet);
            // And what it does to your feet on its own account, which is a
            // different sentence: the walk above is you steering, and this is
            // the move carrying you whether you steer or not. Printed in metres
            // rather than as a speed, because the number a player feels is how
            // much ground the swing closed.
            let carried = if m.step.raw() > 0 {
                format!("steps {} m", tenths(m.step))
            } else if m.step.raw() < 0 {
                format!("gives {} m", tenths(m.step.abs()))
            } else {
                String::new()
            };
            if !carried.is_empty() {
                notes.push(&carried);
            }
            // A launch is the difference between a hit and a hit that starts an
            // air exchange, so it belongs beside the frames rather than only in
            // the kit document.
            let vertical = if m.launch.raw() > 0 {
                format!("knocks up {}", tenths(m.launch))
            } else if m.launch.raw() < 0 {
                format!("spikes {}", tenths(m.launch.abs()))
            } else {
                String::new()
            };
            if !vertical.is_empty() {
                notes.push(&vertical);
            }
            if m.startup < t::HUMAN_REACTION_FRAMES {
                notes.push("unreactable");
            }
            // The Blood mage's economy: blood out on the press, and back only
            // from a pool on the floor. The leech beside it is the Dual mage's
            // dark arm now, and is printed wherever it is set.
            let blood = if m.cost > 0 {
                format!("costs {}% of her health", m.cost)
            } else {
                String::new()
            };
            if !blood.is_empty() {
                notes.push(&blood);
            }
            let returns = if m.leech > 0 {
                format!("returns {}% of the damage", m.leech)
            } else {
                String::new()
            };
            if !returns.is_empty() {
                notes.push(&returns);
            }
            let drinks = if m.drink > 0 {
                format!("drinks {}% of a pool it lands over", m.drink)
            } else {
                String::new()
            };
            if !drinks.is_empty() {
                notes.push(&drinks);
            }
            // The one reach in the game that scales: the scythe's, with her
            // grey. Printed so the table says what the blade can become.
            let grey = if m.rides_the_grey() {
                format!(
                    "reach {} at full grey",
                    tenths(m.reach.mul(t::grey_reach()))
                )
            } else {
                String::new()
            };
            if !grey.is_empty() {
                notes.push(&grey);
            }
            // On-hit means nothing for a move that is still swinging when it
            // lands again, so it is left blank rather than printed wrong.
            let on_hit = if m.rehit > 0 {
                "--".to_string()
            } else {
                format!("{:+}", m.on_hit())
            };
            if !repeat.is_empty() {
                notes.push(&repeat);
            }
            println!(
                "  {:<15}{:<12}{:>4}{:>5}{:>5}{:>8}{:>+10}{:>8}{:>6}  {:<10} {}",
                moves::binding(class, slot),
                m.name,
                m.startup,
                m.active,
                m.recovery,
                m.damage,
                m.on_block(),
                on_hit,
                m.repeat_lock(),
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
         should be. Lock is the repeat lockout: how long after throwing a move you\n\
         may not throw *that* move again, counted from the frame it comes out. The\n\
         rest of the kit is never locked, so it charges for repeating yourself and\n\
         not for attacking. Record what you change in docs/design/feel-log.md."
    );
}

/// Walking speed at a given percentage of it. The mobility column, in metres.
fn walking_at(percent: u8) -> Fx {
    t::move_speed().mul(Fx::ratio(percent as i32, 100))
}

/// One decimal place, without touching floating point.
fn tenths(v: Fx) -> String {
    // Rounded, not truncated: 4.199 should read as 4.2, not 4.1.
    let t = (v.raw() as i64 * 10 + (1 << 15)) >> 16;
    format!("{}.{}", t / 10, (t % 10).abs())
}

fn percent(v: Fx) -> i64 {
    (v.raw() as i64 * 100 + (1 << 15)) >> 16
}

fn hundredths(v: Fx) -> String {
    let t = (v.raw() as i64 * 100 + (1 << 15)) >> 16;
    format!("{}.{:02}", t / 100, (t % 100).abs())
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
