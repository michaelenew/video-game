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
use sim::moves::{self, Move};
use sim::state::{SLOT_COMMITTED, SLOT_POKE};
use sim::tuning as t;

fn every_move() -> impl Iterator<Item = (&'static str, Move)> {
    // By value: move data is live now, so there is no `'static` table to borrow.
    //
    // Moves with no hit volume are left out. There is one -- the Champion's
    // pole vault -- and it is not an attack that happens to miss, it is a way
    // into the air on the attack grammar's button. Every assertion below is
    // about what connecting with somebody is worth, and it never connects.
    ALL_CLASSES.iter().flat_map(|c| {
        moves::table(*c)
            .into_iter()
            .filter(|m| m.strikes())
            .map(move |m| (c.name(), m))
    })
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
    // A move that **re-hits** is exempt, and the exemption is real rather than
    // a convenience. `on_hit` is hitstun minus the frames you are still busy
    // for, and it assumes the exchange is over once you connect. The
    // Champion's Rush slash is still swinging when the next cut lands, so the
    // arithmetic is measuring the wrong thing: what it costs is the whole
    // active window, and what it pays is every cut inside it.
    for (class, m) in every_move().filter(|(_, m)| m.rehit == 0) {
        assert!(
            m.on_hit() >= 0,
            "{class} {}: {:+} on hit — connecting leaves you at a disadvantage",
            m.name,
            m.on_hit()
        );
    }
}

#[test]
fn a_move_that_keeps_hitting_lands_more_than_once_inside_its_own_swing() {
    // The other half of that exemption. A re-hit interval longer than the
    // active window would be a normal move with a misleading field on it, and
    // the exemption above would be hiding a move that is simply minus on hit.
    for class in ALL_CLASSES {
        for m in moves::table(class).iter().filter(|m| m.rehit > 0) {
            assert!(
                m.active > m.rehit,
                "{}: {} re-hits every {} frames and is only active for {}",
                class.name(),
                m.name,
                m.rehit,
                m.active
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
fn no_class_is_half_finished() {
    // Not a design law, just a guard against a class shipping with a gap in it.
    //
    // This used to require every class to have *the same* number of moves, with
    // a note to relax it when one legitimately grew. The Champion legitimately
    // grew: its three mouse buttons are three weapons, each of which behaves
    // differently on foot, in the air and out of a Rush, so it carries ten
    // where the others carry the original three. What is still worth pinning is
    // the floor -- poke, committed, special -- and that the growth is
    // deliberate rather than a table somebody appended to by accident.
    use sim::Class;
    for class in ALL_CLASSES {
        let n = moves::table(class).len();
        let expected = if class == Class::Champion { 10 } else { 3 };
        assert_eq!(
            n,
            expected,
            "{} has {n} moves and should have {expected}",
            class.name()
        );
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
