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
    for (class, m) in every_move() {
        assert!(
            m.on_hit() >= 0,
            "{class} {}: {:+} on hit — connecting leaves you at a disadvantage",
            m.name,
            m.on_hit()
        );
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
fn every_class_has_the_same_number_of_exemplar_moves() {
    // Not a design law, just a guard against a half-finished class shipping
    // unnoticed. Relax it deliberately when a class legitimately grows.
    let counts: Vec<_> = ALL_CLASSES
        .iter()
        .map(|c| (c.name(), moves::table(*c).len()))
        .collect();
    let first = counts[0].1;
    for (name, n) in &counts {
        assert_eq!(*n, first, "{name} has {n} moves, others have {first}");
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

// ---------------------------------------------------------------------------
// Stun
// ---------------------------------------------------------------------------
//
// The rules that have to hold for damage-stuns-and-shoves to produce a fight
// with an arc rather than a stunlock or a shoving match. See
// `docs/design/stun.md`.

#[allow(clippy::assertions_on_constants)]
#[test]
fn knockback_swells_faster_than_hitstun_does() {
    // **The combo design, in one comparison.** Both grow with the damage a
    // fighter has taken. Knockback has to grow faster, or the window to follow
    // someone up grows faster than the distance they cover and combos get
    // easier for the rest of the round instead of harder -- which is a fight
    // that ends in a stunlock rather than in a kill.
    assert!(
        t::swell_knockback().raw() > t::swell_hitstun().raw(),
        "hitstun swells at least as fast as knockback ({} against {}); \
         a combo that works once works forever",
        t::swell_hitstun().to_f32_for_render(),
        t::swell_knockback().to_f32_for_render(),
    );
}

#[allow(clippy::assertions_on_constants)]
#[test]
fn a_hit_never_lands_softer_for_being_late() {
    // Both swells are added to one, so the move table's numbers are a floor:
    // what a move does to someone untouched. A negative growth would make a
    // move weaken as the round went on, which no reading of the frame table
    // would ever suggest.
    assert!(t::swell_knockback().raw() >= 0);
    assert!(t::swell_hitstun().raw() >= 0);
    assert!(t::swell_blow().raw() >= 0);
}

#[allow(clippy::assertions_on_constants)]
#[test]
fn no_source_of_damage_can_stun_for_longer_than_it_takes_to_repeat() {
    // A field ticks on a cadence. Stun it for longer than the cadence and the
    // next tick lands on someone who never got to move, which is a loop with no
    // exit -- and the fighter who laid it did not have to be there for any of
    // it. The freeze counts, because it is time the victim also cannot act in.
    let interval = t::effect_tick_frames();
    for (name, stun) in [
        ("fire pillar", (t::pillar_hitstun(), t::pillar_damage())),
        ("black spike", (t::spike_hitstun(), t::spike_drain())),
    ] {
        let (frames, damage) = stun;
        let held = frames + sim::state::hitlag_frames(damage);
        assert!(
            held < interval,
            "a {name} holds you for {held} frames and ticks every {interval}; \
             standing in one is a loop with no exit"
        );
    }
}

#[allow(clippy::assertions_on_constants)]
#[test]
fn influence_can_steer_a_launch_but_never_reverse_one() {
    // DI bends the launch by at most `atan(strength)`. At one the bend is 45
    // degrees, which is already enough to pick a quadrant; past that a victim
    // is choosing their own direction and knockback has stopped being something
    // the attacker decides.
    assert!(
        t::di_strength().raw() < Fx::ONE.raw(),
        "influence bends a launch by 45 degrees or more, which is not influence"
    );
    assert!(
        t::di_strength().raw() > 0,
        "there is no influence at all, so the freeze is presentation and nothing else"
    );
}

#[test]
fn the_freeze_is_punctuation_and_not_a_pause() {
    // Hitlag has to be long enough to read as contact and short enough that the
    // fight does not become a slideshow. The upper bound is the one that bites:
    // the heaviest move in the game is the worst case, and a freeze approaching
    // its own startup would mean two hits a second.
    let heaviest = every_move().map(|(_, m)| m.damage).max().unwrap();
    let freeze = sim::state::hitlag_frames(heaviest);
    assert!(freeze >= 2, "a hit does not freeze long enough to be felt");
    assert!(
        freeze < t::HUMAN_REACTION_FRAMES,
        "the heaviest move freezes the game for {freeze} frames, which is a pause"
    );
}

#[test]
fn classes_do_not_all_weigh_the_same() {
    // Weight decides how far a fighter travels when hit, which decides whether
    // they can be comboed at all. One shared value would throw away the
    // clearest way a roster can differ.
    let weights: Vec<f32> = ALL_CLASSES
        .iter()
        .map(|c| c.weight().to_f32_for_render())
        .collect();
    let (lo, hi) = weights
        .iter()
        .fold((f32::MAX, 0.0f32), |(l, h), x| (l.min(*x), h.max(*x)));
    assert!(lo > 0.0, "a class weighs nothing: {weights:?}");
    assert!(hi / lo > 1.2, "every class weighs the same: {weights:?}");
}

#[test]
fn the_heavy_is_the_heaviest_and_the_glass_cannon_is_the_lightest() {
    // Not arithmetic: this is the roster saying the same thing twice. A Bulwark
    // that flew further than a Dual mage would read as a bug to anyone who had
    // looked at either class for a minute.
    let heaviest = ALL_CLASSES.iter().max_by_key(|c| c.weight().raw()).unwrap();
    let lightest = ALL_CLASSES.iter().min_by_key(|c| c.weight().raw()).unwrap();
    assert_eq!(
        *heaviest,
        sim::Class::Bulwark,
        "the wall is not the heaviest"
    );
    assert_eq!(
        *lightest,
        sim::Class::DualMage,
        "the floatiest class is not the lightest"
    );
}
