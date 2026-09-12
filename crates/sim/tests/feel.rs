//! The feel harness.
//!
//! "Feels good" cannot be a test. The *properties that must hold for the design
//! to work* can be, and those are what live here.
//!
//! These pin relationships, not values. Every number in `tuning.rs` and
//! `moves.rs` should move freely during tuning; if one of these fails, either
//! it is a bug or a design decision changed — and then `docs/design/feel-log.md`
//! and the relevant design document need updating too, not just the assertion.
//!
//! Record what you tried in the feel log, including the things you reverted.

use sim::class::ALL_CLASSES;
use sim::fixed::Fx;
use sim::moves::{self, Move};
use sim::state::{SLOT_COMMITTED, SLOT_POKE};
use sim::tuning as t;

fn every_move() -> impl Iterator<Item = (&'static str, Move)> {
    // By value: move data is live now, so there is no `'static` table to borrow.
    ALL_CLASSES
        .iter()
        .flat_map(|c| moves::table(*c).into_iter().map(move |m| (c.name(), m)))
}

// ---------------------------------------------------------------------------
// Attacks
// ---------------------------------------------------------------------------

#[test]
fn every_attack_is_punishable_on_block() {
    // If a move were plus on block, the correct play would be to throw it out
    // forever and blocking would be pointless. This is the single property
    // that keeps offence honest.
    for (class, m) in every_move() {
        assert!(
            m.on_block() < 0,
            "{class} {}: {:+} on block. A move that is plus on block has no counterplay.",
            m.name,
            m.on_block()
        );
    }
}

#[test]
fn committed_moves_are_more_punishable_than_pokes() {
    // Risk has to scale with reward, per class.
    for class in ALL_CLASSES {
        let table = moves::table(class);
        let poke = &table[0];
        let committed = table.iter().max_by_key(|m| m.damage).unwrap();
        assert!(
            committed.on_block() < poke.on_block(),
            "{}: {} ({:+} on block) is not more punishable than the poke {} ({:+})",
            class.name(),
            committed.name,
            committed.on_block(),
            poke.name,
            poke.on_block()
        );
        assert!(
            committed.damage > poke.damage * 2,
            "{}: the committed move does not pay enough for its risk",
            class.name()
        );
    }
}

#[test]
fn landing_a_hit_keeps_the_initiative_or_resets_neutral() {
    // On hit you should be no worse off than the defender, or hitting someone
    // would be a mistake.
    //
    // Except for a move that hands out no stun at all, which is not buying the
    // initiative and cannot be measured as though it were. The Elementalist's
    // auto is the one of those: a beam that takes whatever the opponent was
    // charging and gives them their frames straight back. What keeps that
    // honest is the test below, not this one.
    for (class, m) in every_move().filter(|(_, m)| m.hitstun > 0) {
        assert!(
            m.on_hit() >= 0,
            "{class} {}: {:+} on hit — connecting leaves you at a disadvantage",
            m.name,
            m.on_hit()
        );
    }
}

#[test]
fn a_move_that_never_stuns_is_the_cheapest_thing_its_class_throws() {
    // The price of the exception above, and the reason it is not a loophole.
    //
    // A move that lands without stunning gives the defender their turn back
    // immediately, so the attacker must not also come out of it ahead -- it has
    // to be minus on hit, or it would be a button you could simply hold down.
    // And it has to be the smallest hit in the class: what it buys is an
    // interrupt, not damage, and a no-stun move that also hit hard would beat
    // the moves that pay stun for their damage at their own game.
    for class in ALL_CLASSES {
        let table = moves::table(class);
        let softest = table.iter().map(|m| m.damage).min().unwrap();
        for m in table.iter().filter(|m| m.hitstun == 0) {
            assert!(
                m.on_hit() < 0,
                "{} {}: {:+} on hit with no stun at all -- free pressure",
                class.name(),
                m.name,
                m.on_hit()
            );
            assert_eq!(
                m.damage,
                softest,
                "{} {}: hits for {} without stunning, and something in the class hits \
                 for less. A move that buys an interrupt should not also buy damage.",
                class.name(),
                m.name,
                m.damage
            );
        }
    }
}

#[test]
fn each_class_has_something_fast_and_something_slow() {
    // A class whose moves all have the same startup has no rhythm.
    for class in ALL_CLASSES {
        let table = moves::table(class);
        let fastest = table.iter().map(|m| m.startup).min().unwrap();
        let slowest = table.iter().map(|m| m.startup).max().unwrap();
        assert!(
            fastest < t::HUMAN_REACTION_FRAMES,
            "{}: nothing is faster than reaction time; neutral will feel sluggish",
            class.name()
        );
        assert!(
            slowest >= t::HUMAN_REACTION_FRAMES,
            "{}: everything is unreactable; nothing can be punished on sight",
            class.name()
        );
    }
}

#[test]
fn every_class_can_beat_a_turtle() {
    // Guard is a frontal arc with no chip damage, so without an answer to it
    // blocking would be a solved strategy. Either an unblockable, or an
    // overhead that beats crouch-blocking.
    for class in ALL_CLASSES {
        let table = moves::table(class);
        assert!(
            table.iter().any(|m| m.unblockable || !m.hits_crouching),
            "{}: no answer to a blocking opponent",
            class.name()
        );
    }
}

// ---------------------------------------------------------------------------
// Defence
// ---------------------------------------------------------------------------

// These compare constants, so clippy flags them. That is exactly what they are
// for: pinning a relationship between two tunables that must not invert.
#[allow(clippy::assertions_on_constants)]
#[test]
fn the_parry_window_is_a_read_not_a_reaction() {
    // Parry has to be anticipated. If the window were wider than reaction time
    // it would beat everything on sight and offence would stop existing.
    assert!(
        t::parry_window() < t::HUMAN_REACTION_FRAMES,
        "parry is reactable, which makes it strictly better than blocking"
    );
}

#[test]
fn a_parry_actually_pays_for_itself() {
    // The stagger has to be long enough to land the slowest punish, or nobody
    // would risk the tight window.
    let slowest_startup = every_move().map(|(_, m)| m.startup).max().unwrap();
    assert!(
        t::parry_stagger() > slowest_startup,
        "parry stagger ({}) is shorter than the slowest move's startup ({slowest_startup}); \
         a correct read cannot be cashed in",
        t::parry_stagger()
    );
}

#[allow(clippy::assertions_on_constants)]
#[test]
fn a_dodge_has_a_vulnerable_tail() {
    // Invulnerable for its whole duration would make dodge beat everything and
    // never be punished.
    assert!(
        t::dodge_iframes() < t::dodge_frames(),
        "dodge is invulnerable for its entire duration"
    );
    let tail = t::dodge_frames() - t::dodge_iframes();
    let fastest = every_move().map(|(_, m)| m.startup).min().unwrap();
    assert!(
        tail > fastest,
        "the dodge tail ({tail}f) is shorter than the fastest startup ({fastest}f), \
         so a whiffed dodge cannot be punished"
    );
}

#[test]
fn guard_is_an_arc_not_a_bubble() {
    // A full-circle guard cannot be walked around, which removes the whole
    // reason to reposition.
    assert!(
        t::guard_arc_cos().raw() > 0,
        "guard covers 180 degrees or more; flanking a turtle is impossible"
    );
}

#[test]
fn blocking_and_crouching_both_cost_mobility() {
    assert!(t::guard_move_speed().raw() < t::move_speed().raw());
    assert!(t::crouch_move_speed().raw() < t::move_speed().raw());
    assert!(
        t::guard_turn_rate().raw() < t::turn_rate().raw(),
        "guarding does not slow your turn, so the facing arc costs nothing"
    );
}

// ---------------------------------------------------------------------------
// Match shape
// ---------------------------------------------------------------------------

#[test]
fn time_to_kill_is_in_the_right_neighbourhood() {
    // Target is roughly 60 seconds in versus. This assumes a generous hit rate,
    // so it is a sanity bound rather than a prediction: it catches damage
    // numbers that are wrong by an order of magnitude, not by 20%.
    for class in ALL_CLASSES {
        let table = moves::table(class);
        let best = table.iter().max_by_key(|m| m.damage).unwrap();
        let hits_to_kill = t::max_health() / best.damage;
        assert!(
            (3..=30).contains(&hits_to_kill),
            "{}: {} kills in {hits_to_kill} hits",
            class.name(),
            best.name
        );
    }
}

#[test]
fn the_dodge_outruns_a_walk() {
    assert!(
        t::dodge_speed().raw() > t::move_speed().raw(),
        "dodging is slower than walking, so it is never worth the commitment"
    );
}

#[test]
fn every_class_has_the_three_shared_slots() {
    // Not a design law, just a guard against a half-finished class shipping
    // unnoticed. The three shared slots -- poke, committed, special -- mean the
    // same thing on every class, which is what lets one control scheme drive
    // six kits, so every class has to fill all three.
    //
    // The fourth is `E`, and it is **not** shared: it is the class mechanic,
    // which is an instant state change on most of the roster and an ability
    // only on the Blood mage. A class having one is a decision about that
    // class, not a gap in it.
    use sim::state::{SLOT_COMMITTED, SLOT_MECHANIC, SLOT_POKE, SLOT_SPECIAL};
    for class in ALL_CLASSES {
        for slot in [SLOT_POKE, SLOT_COMMITTED, SLOT_SPECIAL] {
            assert!(
                moves::bound(class, slot as usize),
                "{} has nothing on {}",
                class.name(),
                moves::binding(slot as usize)
            );
        }
    }
    assert!(
        ALL_CLASSES
            .iter()
            .any(|c| moves::bound(*c, SLOT_MECHANIC as usize)),
        "nothing binds the mechanic slot, so the fourth column is dead weight"
    );
}

/// The most damage one cast of a move can do to one target.
///
/// Every ability the Blood mage has is a *several* rather than a one: the blade
/// cuts on the way out and again on the way back, the Grasp is four arms, and
/// the spike is a field that ticks for as long as somebody is standing in it.
/// A cost weighed against a single connection would say all four are a losing
/// trade, and the class would be unplayable by its own numbers.
fn best_case(m: &Move) -> i32 {
    use sim::effects::{EffectKind, GRASP_ARMS};
    match EffectKind::from_code(m.effect) {
        Some(EffectKind::Bloodletter) => EffectKind::Bloodletter.damage(m) * 2,
        Some(EffectKind::Grasp) => EffectKind::Grasp.damage(m) * GRASP_ARMS as i32,
        // A field, for as long as it stands. The move's own hit lands too.
        Some(kind @ (EffectKind::BlackSpike | EffectKind::FirePillar)) => {
            let ticks = kind.life() / t::effect_tick_frames().max(1);
            m.damage + kind.damage(m) * ticks as i32
        }
        None => m.damage,
    }
}

#[test]
fn a_root_outlives_the_hitstun_that_delivers_it() {
    // A root is only visible in the frames after you can act again. Deliver it
    // with a move whose hitstun is longer and it is a no-op that reads, in the
    // hand, as the ability simply not working.
    use sim::state::SLOT_SPECIAL;
    let grasp = moves::get(sim::class::Class::BloodMage, SLOT_SPECIAL);
    assert!(
        t::grasp_root() > grasp.hitstun,
        "the Grasp roots for {} frames and stuns for {}, so the root is invisible",
        t::grasp_root(),
        grasp.hitstun
    );
}

#[test]
fn preying_on_the_disabled_is_worth_feeling_and_is_not_an_execution() {
    // Both ends. Below about a fifth extra it is a number nobody notices and
    // the Grasp goes back to being a root with no payoff; far above it and
    // landing one disable is the match, which is the opposite of a game built
    // on reads and whiff punishment.
    let mul = t::disabled_damage_mul();
    assert!(
        mul.raw() > Fx::ratio(6, 5).raw(),
        "the bonus is {}x, which nobody will feel",
        mul.to_f32_for_render()
    );
    assert!(
        mul.raw() < Fx::from_int(2).raw(),
        "the bonus is {}x, so one read ends the round",
        mul.to_f32_for_render()
    );

    // And the disable it is built around has to outlast the wind-up of
    // something worth spending it on, or there is nothing to follow up with.
    let root = t::grasp_root();
    let fastest = moves::table(sim::class::Class::BloodMage)
        .iter()
        .map(|m| m.startup)
        .min()
        .expect("the class has moves");
    assert!(
        root > fastest,
        "the root lasts {root} frames and her fastest move takes {fastest} to \
         come out, so nothing can be landed inside it"
    );
}

#[test]
fn the_blood_mage_pays_for_everything_and_nobody_else_pays_for_anything() {
    // The class is its economy: health out on the press, health back on the
    // hit. Both halves on every one of her abilities, and on nobody else's --
    // a second class quietly acquiring a health cost would mean the mechanic
    // had stopped being an identity and become a tax.
    for class in ALL_CLASSES {
        let blood = class == sim::class::Class::BloodMage;
        for m in moves::table(class) {
            assert_eq!(
                m.cost > 0,
                blood,
                "{} {}: health cost {} does not match the class mechanic",
                class.name(),
                m.name,
                m.cost
            );
            if blood {
                assert!(
                    m.leech > 0,
                    "{}: costs health and gives none of it back, so it is pure downside",
                    m.name
                );
                let best = m.leeched(best_case(&m));
                assert!(
                    best > m.cost,
                    "{}: thrown perfectly it returns {best} and cost {}, so playing well \
                     still loses you the fight",
                    m.name,
                    m.cost
                );
            }
        }
    }
}

#[test]
fn hindrance_is_proportional_to_commitment() {
    // The rule, stated once: the more a move commits you, the more it takes
    // your feet away. A poke that rooted you as hard as a slam would make the
    // two feel the same to throw, which is the opposite of what the frame data
    // is trying to say.
    for class in ALL_CLASSES {
        let poke = moves::get(class, SLOT_POKE);
        let committed = moves::get(class, SLOT_COMMITTED);
        assert!(
            poke.mobility > committed.mobility,
            "{class:?}: the poke '{}' hinders you no less than '{}'",
            poke.name,
            committed.name
        );
        assert!(
            committed.roots(),
            "{class:?}: '{}' is a committed move that lets you keep walking",
            committed.name
        );
    }
}

#[test]
fn a_poke_is_a_slow_not_a_stop_and_not_free() {
    // Both failure modes are real. Rooted reads as the game snatching the
    // controls; unhindered removes the spacing cost of throwing it at all, and
    // spacing is most of neutral.
    for class in ALL_CLASSES {
        let poke = moves::get(class, SLOT_POKE);
        assert!(
            poke.mobility >= 30 && poke.mobility <= 80,
            "{class:?}: '{}' keeps {}% of walking speed",
            poke.name,
            poke.mobility
        );
    }
}

// ---------------------------------------------------------------------------
// Air
// ---------------------------------------------------------------------------

#[test]
fn classes_are_not_the_same_in_the_air() {
    // Weight is the most legible difference a character can have -- you can
    // read it across the arena in the first second of a match, before you know
    // a single one of their moves. If every class shared one jump arc, that
    // whole channel would be unused.
    let jumps: Vec<f32> = ALL_CLASSES
        .iter()
        .map(|c| c.mobility().jump.to_f32_for_render())
        .collect();
    let gravities: Vec<f32> = ALL_CLASSES
        .iter()
        .map(|c| c.mobility().gravity.to_f32_for_render())
        .collect();
    let spread = |v: &[f32]| {
        let (lo, hi) = v
            .iter()
            .fold((f32::MAX, 0.0f32), |(l, h), x| (l.min(*x), h.max(*x)));
        hi / lo
    };
    assert!(
        spread(&jumps) > 1.15,
        "every class jumps the same height: {jumps:?}"
    );
    assert!(
        spread(&gravities) > 1.25,
        "every class falls at the same rate: {gravities:?}"
    );
}

#[test]
fn air_control_is_weaker_than_walking_for_everyone() {
    // The air is a commitment. A class that steers in the air as well as it
    // walks on the ground has not committed to anything by jumping.
    for class in ALL_CLASSES {
        let air = class.mobility().air_speed;
        assert!(
            air.raw() < t::move_speed().raw(),
            "{}: air control ({}) is not weaker than walking",
            class.name(),
            air.to_f32_for_render()
        );
        assert!(
            air.raw() > 0,
            "{}: cannot steer in the air at all",
            class.name()
        );
    }
}

#[test]
fn nobody_hovers() {
    // Gravity and terminal velocity must both stay positive multiples, or a
    // class stops coming down and the whole positioning game with it.
    for class in ALL_CLASSES {
        let m = class.mobility();
        assert!(m.gravity.raw() > 0, "{}: no gravity", class.name());
        assert!(
            m.fall_cap.raw() > 0,
            "{}: no terminal velocity",
            class.name()
        );
        assert!(m.jump.raw() > 0, "{}: cannot jump", class.name());
    }
}

#[test]
fn an_aerial_hang_is_shorter_than_the_move_that_carries_it() {
    // A hang longer than the move is a float with an attack attached, not an
    // attack with a float attached -- and it would let a whiffed aerial stay
    // safe by simply remaining out of reach.
    for (class, m) in every_move() {
        assert!(
            m.air_stall < m.whiff_cost(),
            "{class} {}: hangs for {} frames but only lasts {}",
            m.name,
            m.air_stall,
            m.whiff_cost()
        );
    }
}
