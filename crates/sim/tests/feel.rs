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
    // Two exemptions, both real rather than convenient.
    //
    // A move that hands out **no stun at all** is not buying the initiative and
    // cannot be measured as though it were. The Elementalist's auto is the one
    // of those: a beam that takes whatever the opponent was charging and gives
    // them their frames straight back. What keeps that honest is the first test
    // below, not this one.
    //
    // A move that **re-hits** breaks the arithmetic itself. `on_hit` is hitstun
    // minus the frames you are still busy for, and it assumes the exchange is
    // over once you connect. The Champion's Rush slash is still swinging when
    // the next cut lands, so what it costs is the whole active window and what
    // it pays is every cut inside it. The second test below is its guard.
    for (class, m) in every_move().filter(|(_, m)| m.hitstun > 0 && m.rehit == 0) {
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
    // The guard on the second exemption. A re-hit interval longer than the
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

/// The most damage one cast of a move can do to one target.
///
/// Every ability the Blood mage has is a *several* rather than a one: the blade
/// cuts on the way out and again on the way back, the Grasp is four arms, the
/// spike is a field that ticks for as long as somebody is standing in it, and
/// any move at all can be given a re-hit interval. A cost weighed against a
/// single connection would say all four are a losing trade, and the class would
/// be unplayable by its own numbers.
fn best_case(m: &Move) -> i32 {
    use sim::effects::{EffectKind, GRASP_ARMS};
    // A move that keeps hitting connects once per interval across its active
    // window. Zero is the normal rule -- one connection.
    let swings = m.active.checked_div(m.rehit).map_or(1, |n| n.max(1) as i32);
    match EffectKind::from_code(m.effect) {
        Some(EffectKind::Bloodletter) => EffectKind::Bloodletter.damage(m) * 2,
        Some(EffectKind::Grasp) => EffectKind::Grasp.damage(m) * GRASP_ARMS as i32,
        // A field, for as long as it stands. The move's own hit lands too.
        Some(
            kind @ (EffectKind::BlackSpike | EffectKind::FirePillar | EffectKind::GuillotineLotus),
        ) => {
            let ticks = kind.life() / t::effect_tick_frames().max(1);
            m.damage * swings + kind.damage(m) * ticks as i32
        }
        None => m.damage * swings,
    }
}

#[test]
fn the_blood_mage_pays_for_everything_and_nobody_else_pays_for_anything() {
    // The class is its economy: health out on the press, health back on the
    // hit. Both halves on every one of her abilities, and on nobody else's --
    // a second class quietly acquiring a health cost would mean the mechanic
    // had stopped being an identity and become a tax.
    //
    // Restored after a merge dropped it. It is the assertion that stops a
    // tuning pass from quietly making an ability cost more than landing it
    // perfectly can ever return, which is the one way this class breaks that
    // looks like a balance choice rather than a bug.
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
fn the_blood_mage_does_not_kill_in_two_buttons() {
    // The other side of the same tuning pass. Her placed abilities are worth
    // several connections each, so the per-hit damage in the table says very
    // little about what one cast is actually worth -- which is how the spike
    // ended up at nearly two thirds of a health bar without any single number
    // in the table looking wrong.
    use sim::class::Class;
    for m in moves::table(Class::BloodMage) {
        let cast = best_case(&m);
        assert!(
            cast * 3 < t::max_health(),
            "{}: one cast is {cast} against a {} bar, so three of them is the match",
            m.name,
            t::max_health()
        );
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
fn every_class_has_the_three_shared_slots_and_no_more_than_it_means_to() {
    // Not a design law, just a guard against a class shipping with a gap in it
    // -- or with a table somebody appended to by accident.
    //
    // The three shared slots -- poke, committed, special -- mean the same thing
    // on every class, which is what lets one control scheme drive six kits, so
    // every class has to fill all three. What a class has **past** them is a
    // decision about that class: the Blood mage's fourth is on `E`, because her
    // mechanic is health and there is nothing to toggle, the Reaver's fourth is
    // on `E` because throwing a second body across the arena and dashing it
    // home through somebody is not an instant, the Champion's ten are three
    // weapons by three stances plus the vault, and the Dual mage's five are
    // those three plus Sweep on `E` -- her mechanic is a meter steered by which
    // button attacks, so `E` is free the same way -- plus a second auto on
    // right click, because her two forces are two different moves rather than
    // one move with a modifier.
    use sim::Class;
    use sim::state::{SLOT_COMMITTED, SLOT_POKE, SLOT_SPECIAL};
    for class in ALL_CLASSES {
        for slot in [SLOT_POKE, SLOT_COMMITTED, SLOT_SPECIAL] {
            assert!(
                moves::bound(class, slot as usize),
                "{} has nothing on {}",
                class.name(),
                moves::binding(class, slot as usize)
            );
        }
        let n = moves::table(class).len();
        let expected = match class {
            Class::Champion => 10,
            Class::BloodMage | Class::ShadowReaver => 4,
            Class::DualMage => 5,
            _ => 3,
        };
        assert_eq!(
            n,
            expected,
            "{} has {n} moves and should have {expected}",
            class.name()
        );
    }
    assert!(
        ALL_CLASSES.iter().any(|c| moves::on_e(*c).is_some()),
        "nothing binds an ability to the mechanic key, so the slot is dead weight"
    );
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
    // it. The freeze counts, because it is time the victim also cannot act in,
    // and so does the swell, because the whole point of the swell is that these
    // numbers are not what they say by the end of a round.
    let interval = t::effect_tick_frames();
    let at_death = Fx::ONE.add(t::swell_hitstun());
    for (name, frames, damage) in [
        ("fire pillar", t::pillar_hitstun(), t::pillar_damage()),
        ("black spike", t::spike_hitstun(), t::spike_drain()),
    ] {
        let swelled = Fx::from_int(frames as i32).mul(at_death).to_int() as u16;
        let held = swelled + sim::state::hitlag_frames(damage);
        assert!(
            held < interval,
            "a {name} holds you for {held} frames at death's door and ticks every \
             {interval}; standing in one is a loop with no exit"
        );
    }
}

#[test]
fn the_freeze_is_punctuation_and_not_a_pause() {
    // Hitlag has to be long enough to read as contact and short enough that the
    // fight does not become a slideshow. The upper bound is the one that bites:
    // the heaviest move in the game is the worst case, and a freeze approaching
    // reaction time would mean two hits a second.
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
fn the_wall_is_the_heaviest_and_the_glass_cannon_is_the_lightest() {
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
