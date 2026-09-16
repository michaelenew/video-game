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
    //
    // **Only moves that actually swing something.** A move with no hitbox --
    // `radius` zero, a gesture that puts something into the world and lets the
    // thing it placed do the hitting -- has no frame advantage to be wrong
    // about: `on_hit` is derived from the attacker's own active and recovery
    // frames against a victim the move never touches, which is why the frame
    // table prints `--` for it rather than a number. The rule above is about
    // hitboxes that trade stun for an interrupt, and a gesture trades nothing.
    // Asked of one anyway it reads a `damage` field that belongs to whatever
    // the gesture placed, which is how it came to have an opinion about the
    // Reaver's Send shadow -- a move whose damage is carried by the second body
    // on its way home, and which by design cannot interrupt anybody at all.
    // See `Hit::interrupts`.
    for class in ALL_CLASSES {
        let table = moves::table(class);
        let softest = table.iter().map(|m| m.damage).min().unwrap();
        for m in table.iter().filter(|m| m.hitstun == 0 && m.strikes()) {
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
        //
        // `FireTornado` never actually reaches this match: nothing casts one
        // by code, Cataclysm only ever turns an existing fire pillar into
        // one at runtime. It is grouped here anyway, for the day something
        // does ask a Move for its best case against a tornado's own numbers.
        Some(
            kind @ (EffectKind::BlackSpike
            | EffectKind::FirePillar
            | EffectKind::FireTornado
            | EffectKind::GuillotineLotus),
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
fn a_grasp_always_finishes_hauling_before_it_lets_go() {
    // The catch is a bind and then a haul (`state::drag_the_held`), and the
    // haul covers real ground at a real speed. If the hold runs out first, a
    // Grasp landed at full range drops its victim halfway home -- standing in
    // mid-air, mid-drag, suddenly able to walk. The failure is invisible at
    // short range and total at long range, which is the worst way for a number
    // to be wrong.
    use sim::state::SLOT_SPECIAL;
    let grasp = moves::get(sim::class::Class::BloodMage, SLOT_SPECIAL);
    let arm = t::body_radius().add(t::body_radius());
    let furthest = grasp.reach.sub(arm);
    let hauling = grasp.grabs.saturating_sub(t::grasp_bind());
    let covered = t::reel_speed()
        .mul(Fx::from_int(hauling as i32))
        .div(Fx::from_int(60));
    assert!(
        covered.raw() >= furthest.raw(),
        "the hold leaves {hauling} frames to haul, which covers {} m of the {} m \
         a full-range Grasp has to cross",
        covered.to_f32_for_render(),
        furthest.to_f32_for_render()
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

    // And the disable it is built around is deliberately **too short to react
    // to**. The Grasp is a way to drag somebody into something that is already
    // happening, not a free window to start one in: a mage who waits to see the
    // arms close and then presses her expensive button has already missed it.
    // That is what makes the ability high risk -- the commitment is made before
    // it is known to have landed. See `docs/design/kits/blood-mage.md`.
    use sim::state::SLOT_SPECIAL;
    let hold = moves::get(sim::class::Class::BloodMage, SLOT_SPECIAL).grabs;
    let kit = moves::table(sim::class::Class::BloodMage);
    let dearest = kit
        .iter()
        .max_by_key(|m| m.cost)
        .expect("the class has moves");
    assert!(
        hold < dearest.startup,
        "the catch lasts {hold} frames and {} comes out in {}, so it can be \
         thrown on reaction to the grab landing",
        dearest.name,
        dearest.startup
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
    // home through somebody is not an instant, the Elementalist's seven are
    // those three plus Cataclysm on right click -- otherwise dead weight on a
    // class with no shield -- and then a whole row of three more for the same
    // buttons with her feet off the floor, which is the aerials question
    // `docs/design/README.md` leaves open answered for a second class --
    // the Champion's nineteen are three weapons by six situations
    // plus the vault, three of those six being the three hits of one ground
    // chain, and the Dual mage's five are those three plus Sweep on `E` -- her
    // mechanic is a meter steered by which button attacks, so `E` is free the
    // same way -- plus a second auto on right click, because her two forces
    // are two different moves rather than one move with a modifier.
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
            Class::Champion => 19,
            Class::Elementalist => 7,
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
            committed.mobility as i32 * 100 <= poke.mobility as i32 * COMMITTED_SHARE_OF_A_POKE,
            "{class:?}: '{}' hinders you barely more than the poke '{}' does ({}% against {}%)",
            committed.name,
            poke.name,
            committed.mobility,
            poke.mobility
        );
    }
}

/// How much of a poke's mobility a committed move is allowed to keep, as a
/// percentage. Half: the two have to be *obviously* different to throw, or the
/// frame data is saying one thing and the feet another.
const COMMITTED_SHARE_OF_A_POKE: i32 = 50;

#[test]
fn a_committed_move_is_a_crawl_and_never_a_stop() {
    // The other half of `a_poke_is_a_slow_not_a_stop_and_not_free`, and the same
    // two failure modes one rung down.
    //
    // Zero is the one the design gave up on 2026-09-14: a character who ignores
    // the stick reads as the game taking the controls away, and a forty-frame
    // heavy is the worst place in the game to do that. What commitment means is
    // pinned by `a_committed_move_takes_the_jump_and_the_dodge_away` below --
    // that is where the spatial cost actually lives, and it is untouched.
    //
    // The ceiling is the other failure: a committed move you can walk out of at
    // guard speed has stopped costing you the ground, and spacing is most of
    // neutral.
    for class in ALL_CLASSES {
        let committed = moves::get(class, SLOT_COMMITTED);
        let speed = t::move_speed().mul(Fx::ratio(committed.mobility as i32, 100));
        assert!(
            committed.mobility > 0,
            "{class:?}: '{}' roots you outright",
            committed.name
        );
        assert!(
            speed.raw() < t::guard_move_speed().raw(),
            "{class:?}: '{}' leaves you faster than guarding does, so committing to it \
             costs you no ground",
            committed.name
        );
    }
}

/// Aim angle for a fighter looking the way they spawn. Movement and attacks are
/// camera-relative, so a fixture that does not say where it is looking is not
/// saying what its buttons mean.
const LOOKING: u16 = 0;

/// Where player one is, and what they are doing, four frames into a move thrown
/// with `throw` while holding `then`.
fn four_frames_into(class: sim::class::Class, throw: u16, then: u16) -> (u32, i32, i32) {
    use sim::{Input, World};
    let mut w = World::with_classes([class, class]);
    w.advance([Input::aimed(throw, LOOKING), Input::default()]);
    assert!(
        w.players[0].action.attack_kind().is_some(),
        "{}: the fixture threw nothing",
        class.name()
    );
    for _ in 0..4 {
        w.advance([Input::aimed(then, LOOKING), Input::default()]);
    }
    let p = &w.players[0];
    (p.action.tag(), p.pos.y.raw(), p.vel.y.raw())
}

#[test]
fn no_attack_lets_you_jump_or_dodge_out_of_it() {
    // **What commitment means, now that nothing roots.** The spatial cost of a
    // move is not that you stand still -- it is that for its whole length the
    // only thing you can do is finish it. If jump or dodge leaked out of one,
    // the crawl above would be all that was left of the commitment, and the
    // crawl is a feel decision rather than a cost.
    //
    // Measured against the same move with no second press rather than against
    // an absolute, because a move with `self_lift` takes you off the ground on
    // its own and an assertion that you are still standing would read that as a
    // jump.
    //
    // Both attack buttons, because shift plus left click is the committed move
    // on four of the six and the Champion and the Dual mage spend the modifier
    // differently -- whatever comes out is still an attack, which is the claim.
    use sim::Input;
    for class in ALL_CLASSES {
        for throw in [Input::LEFT, Input::SHIFT | Input::LEFT] {
            let quiet = four_frames_into(class, throw, 0);
            assert_eq!(
                four_frames_into(class, throw, Input::SPACE),
                quiet,
                "{}: jumped out of an attack",
                class.name()
            );
            assert_eq!(
                four_frames_into(class, throw, Input::SHIFT | Input::W),
                quiet,
                "{}: dodged out of an attack",
                class.name()
            );
        }
    }
}

#[test]
fn a_move_never_snaps_you_to_its_speed() {
    // The half of the 2026-09-14 change that does the work, and the half that
    // is easiest to lose: **the hindered speed is arrived at, not assigned.**
    //
    // Setting it outright is a one-frame drop of 5.6 m/s into a committed
    // move's crawl, which is the same lurch the old dead stop was with a
    // different number at the bottom of it. Both were reported as jarring and
    // both are this assertion. A regression here would not fail any other test
    // in the file: the speeds either side of the ramp would still be right.
    use sim::{Input, World};
    for class in ALL_CLASSES {
        let mut w = World::with_classes([class, class]);
        // Up to a full walk first: there is nothing to ramp down from
        // otherwise.
        for _ in 0..30 {
            w.advance([Input::aimed(Input::W, LOOKING), Input::default()]);
        }
        let walking = flat_speed(&w);

        // Throw the heaviest thing left click will give this class, and keep
        // holding the direction. Which move that is differs -- shift plus left
        // is the committed move on four of the six, and the Champion's left
        // click is the sword whatever the modifier says -- so the speed to ramp
        // to is read off whatever actually came out rather than assumed.
        let held = Input::SHIFT | Input::LEFT | Input::W;
        w.advance([Input::aimed(held, LOOKING), Input::default()]);
        let kind = w.players[0]
            .action
            .attack_kind()
            .unwrap_or_else(|| panic!("{}: the fixture threw nothing", class.name()));
        let m = moves::get(class, kind);
        let target = t::move_speed().mul(Fx::ratio(m.mobility as i32, 100)).raw();
        assert!(
            target < walking,
            "{}: '{}' does not hinder you at all, so there is no ramp to test",
            class.name(),
            m.name
        );

        let first = flat_speed(&w);
        assert!(
            first > target,
            "{}: '{}' snapped you straight to {} on its first frame",
            class.name(),
            m.name,
            target
        );

        // And it gets there, rather than gliding for the length of the move.
        //
        // **Except while the move is driving you**, which is a different thing
        // and has to be excused rather than tolerated: a move with a `step` is
        // carrying the body forward on purpose, so the speed climbs for the
        // frames of its window and then has to come back down. What the ramp is
        // about is the frames where nothing is driving you -- see
        // `state::attack_step`.
        let mut speed = first;
        let mut settled = None;
        let window = m.step.raw().signum().unsigned_abs() as usize * (t::step_lead() as usize + 1);
        for _ in 0..8 + window {
            w.advance([Input::aimed(Input::W, LOOKING), Input::default()]);
            // Read *after* the advance: the countdown runs before the movement
            // does, so the action standing at the end of a frame is the one that
            // chose that frame's velocity.
            let driving = stepping(&w, &m);
            let next = flat_speed(&w);
            if !driving {
                assert!(
                    next <= speed,
                    "{}: the ramp into '{}' went back up on a frame it was not \
                     driving the body",
                    class.name(),
                    m.name
                );
                if (next - target).abs() <= SETTLED {
                    settled = Some(next);
                }
            }
            speed = next;
        }
        assert!(
            settled.is_some(),
            "{}: '{}' is over and the walk never came back down to its own speed \
             ({speed} against {target})",
            class.name(),
            m.name
        );
    }
}

/// Is this move carrying the body forward on this frame?
///
/// The same window `state::attack_step` uses, read from the outside: `step_lead`
/// frames of the startup, and then the frame the hitbox appears on.
fn stepping(w: &sim::World, m: &Move) -> bool {
    use sim::state::Action;
    m.step.raw() != 0
        && match w.players[0].action {
            Action::Startup { left, .. } => left < t::step_lead(),
            Action::Active { left, .. } => left == m.active,
            _ => false,
        }
}

/// How close to a move's own speed counts as having arrived, in raw 16.16 bits.
///
/// Not a tolerance on the design: the ramp lands exactly on the floor, and then
/// the floor is multiplied by a *unit* direction whose components were rounded
/// to fixed point, which is worth a raw unit or two either way. Sixteen of them
/// is four ten-thousandths of a metre per second.
const SETTLED: i32 = 16;

/// Player one's horizontal speed, in raw fixed point.
fn flat_speed(w: &sim::World) -> i32 {
    let v = w.players[0].vel;
    sim::math::V3::new(v.x, Fx::ZERO, v.z).flat_len().raw()
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
// The repeat lockout
// ---------------------------------------------------------------------------
//
// The behaviour is pinned in `tests/lockout.rs`. What belongs here is the pair
// of relationships that decide whether the rule is still the one the design
// argued for after somebody has finished dragging the knob around -- because
// the same mechanism tuned twice as far stops being a charge for repetition
// and becomes a cooldown, which `docs/design/combat-kernel.md` rules out.

#[test]
fn the_lockout_always_leaves_something_faster_to_do_than_wait() {
    // **The line between this and a cooldown, as a number.**
    //
    // Being locked out of a move is only interesting while the answer is to
    // throw something else. The moment the idle frames outlast the cheapest
    // other thing in the kit, the answer becomes *stand still until it comes
    // back* -- and standing still waiting for an ability is exactly the
    // question the combat kernel refuses to ask.
    for class in ALL_CLASSES {
        let table = moves::table(class);
        for (i, m) in table.iter().enumerate() {
            let idle = m.repeat_idle();
            if idle == 0 {
                continue;
            }
            let cheapest_other = table
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, n)| n.whiff_cost())
                .min()
                .expect("every class has more than one move");
            assert!(
                idle <= cheapest_other,
                "{} {}: locked out for {idle} frames after it finishes, and the \
                 cheapest other move in the kit takes {cheapest_other}. Past that \
                 point the correct play is to wait rather than to use the kit, \
                 which is a cooldown wearing a different name.",
                class.name(),
                m.name,
            );
        }
    }
}

#[test]
fn one_press_can_never_lock_more_than_one_move() {
    // The structural half of the same claim, and the one that would catch
    // somebody making the lockout global in a hurry. A press costs you *that*
    // ability and nothing else, so the worst case is always a kit with one
    // fewer option in it -- never a kit with nothing in it.
    use sim::state::SLOT_POKE;
    for class in ALL_CLASSES {
        let mut p = sim::state::Player::new(class);
        let out = [false; moves::MAX_SLOTS];
        p.repeat_lock[SLOT_POKE as usize] = 240;
        let available = (0..moves::table(class).len())
            .filter(|slot| p.can_throw(*slot as u8, &out))
            .count();
        assert_eq!(
            available,
            moves::table(class).len() - 1,
            "{}: locking the poke left {available} of {} moves available",
            class.name(),
            moves::table(class).len(),
        );
    }
}

// ---------------------------------------------------------------------------
// The Champion's chain
// ---------------------------------------------------------------------------
//
// Three hits deep on the ground, and every hit is a free choice of all three
// weapons -- one haft, three heads, chosen a hit at a time. The numbers move
// freely; what must not move is that a string **escalates**, that it stays
// connected long enough to finish, and that each weapon still reads as itself
// all the way through.

/// How far a knockback of `speed` actually carries somebody, in metres.
///
/// The speed is applied once and decays by `knockback_decay` a frame, so what
/// they travel is `speed * dt * (1 + d + d^2 + ...)` -- a geometric series, and
/// nowhere near the speed itself. Worked out rather than measured because the
/// thing being pinned is the relationship, and a simulated hit would drag in
/// hitstun, the floor and whoever is standing where.
fn shoved(speed: Fx) -> f32 {
    let decay = t::knockback_decay().to_f32_for_render();
    speed.to_f32_for_render() * sim::DT.to_f32_for_render() / (1.0 - decay).max(1e-3)
}

/// The Champion's ground chain, as a weapon at a time: opener, connector,
/// finisher.
fn chains() -> impl Iterator<Item = (&'static str, [Move; 3])> {
    use sim::Class;
    use sim::moves::champion as c;
    [
        ("sword", c::SWORD),
        ("hammer", c::HAMMER),
        ("spear", c::SPEAR),
    ]
    .into_iter()
    .map(|(name, weapon)| {
        (
            name,
            [0u8, 1, 2].map(|link| moves::get(Class::Champion, c::link(link, weapon))),
        )
    })
}

#[test]
fn a_string_escalates() {
    // Three hits that all hit the same is three presses, not a decision. Each
    // hit has to be worth more than the one before it, and the last one has to
    // be worth the whole string -- which is what makes being interrupted on the
    // second hit a real loss rather than a rounding error.
    for (weapon, links) in chains() {
        for pair in links.windows(2) {
            assert!(
                pair[1].damage > pair[0].damage,
                "the {weapon} chain goes {} ({}) then {} ({}), which is not an escalation",
                pair[0].name,
                pair[0].damage,
                pair[1].name,
                pair[1].damage
            );
        }
    }
}

#[test]
fn a_string_is_riskier_the_deeper_you_are_in_it() {
    // Risk scales with reward, stated for the chain rather than for the kit:
    // the finisher is the move you can be punished hardest for throwing, which
    // is what stops the third hit being free once the first two landed.
    for (weapon, links) in chains() {
        assert!(
            links[2].on_block() < links[0].on_block(),
            "the {weapon} finisher {} is {:+} on block and its opener {} is {:+}",
            links[2].name,
            links[2].on_block(),
            links[0].name,
            links[0].on_block()
        );
        assert!(
            links[2].whiff_cost() > links[0].whiff_cost(),
            "the {weapon} finisher {} commits you for no longer than its opener",
            links[2].name
        );
    }
}

#[test]
fn the_first_two_hits_leave_somebody_standing_where_the_third_can_reach_them() {
    // **The rule that makes a chain a chain.** A hit that shoves them out of
    // range of the next one has ended the string whether or not the game says
    // so, which is why the openers and connectors are the softest things the
    // class throws and the finishers carry the knockback.
    for (weapon, links) in chains() {
        for early in &links[..2] {
            assert!(
                early.knockback.raw() < links[2].knockback.raw(),
                "the {weapon} chain's {} knocks them further than its own finisher {} does",
                early.name,
                links[2].name
            );
            // And the shove has to be short enough in **metres** that they are
            // still inside the next hit's reach when it arrives. Knockback is a
            // speed that decays every frame, so the distance it actually
            // carries somebody is the sum of a geometric series -- which is the
            // number this rule is about, and it is nothing like the speed.
            let travel = shoved(early.knockback);
            assert!(
                travel < links[2].reach.to_f32_for_render(),
                "the {weapon} chain's {} carries them {travel:.2} m and the finisher {} \
                 only reaches {:.2} m",
                early.name,
                links[2].name,
                links[2].reach.to_f32_for_render()
            );
        }
    }
}

#[test]
fn no_two_weapons_ever_put_out_the_same_volume() {
    // **The identity half, and the version of it that is actually a design
    // decision.** What the player reads off the screen is a shape, and the shape
    // has to name the weapon -- so the volumes one weapon uses across its chain
    // are its own, and no other weapon touches them. A player who has learnt
    // that the hammer owns the ground under it has learnt something true of
    // every hammer move there is.
    //
    // It used to say something stricter: that a weapon's three links were all
    // the *same* volume. That stopped being true on 2026-09-15 and for good
    // reasons in both directions -- the sword's cuts mirror each other, which is
    // the whole of why the string flows, and the spear's finisher spends its
    // length swept flat instead of thrust, which is the only thing a
    // three-metre pole can do that nothing else in the game can. Neither of
    // those blurs which weapon is out; a sword cut and a hammer swing reading
    // alike would. See `docs/design/feel-log.md`.
    use sim::moves::Shape;
    let sets: Vec<(&str, Vec<Shape>)> = chains()
        .map(|(weapon, links)| {
            let mut shapes: Vec<Shape> = links.iter().map(|m| m.shape).collect();
            shapes.dedup();
            (weapon, shapes)
        })
        .collect();
    for (i, (weapon, mine)) in sets.iter().enumerate() {
        for (other, theirs) in &sets[i + 1..] {
            for shape in mine {
                assert!(
                    !theirs.contains(shape),
                    "the {weapon} and the {other} both swing {shape:?}, so the \
                     volume on the screen does not say which weapon it was"
                );
            }
        }
        // And a weapon may not spend more than two of its three links on
        // different volumes: three would not be a weapon, it would be three.
        assert!(
            mine.len() <= 2,
            "the {weapon} chain is three different volumes: {mine:?}"
        );
    }
}

#[allow(clippy::assertions_on_constants)]
#[test]
fn a_cancel_is_always_a_saving_and_a_swap_is_always_the_better_one() {
    // The two knobs that decide the chain's rhythm, and the only relationship
    // between them that is a design decision rather than a taste: swapping
    // weapons flows faster than repeating one, because the weapon re-forms out
    // of the follow-through. Setting them equal turns the incentive off, which
    // is a decision somebody can make -- setting them the other way round is
    // the class arguing against its own fantasy.
    assert!(
        t::chain_cancel_swapped() < t::chain_cancel_repeated(),
        "repeating a weapon ({}%) flows no slower than swapping one ({}%)",
        t::chain_cancel_repeated(),
        t::chain_cancel_swapped()
    );
    assert!(
        t::chain_cancel_repeated() < 100,
        "a connected link pays its whole recovery, so the chain buys nothing at all"
    );
    assert!(t::chain_grace() > 0, "a chain dies the frame it is born");
}

#[test]
fn every_weapon_has_its_own_way_off_the_ground_and_they_are_three_decisions() {
    // The takeoff row. Three of them, one per weapon, and they have to differ
    // in what they are *for* rather than in how much: the sword's is the one
    // that hurts, the hammer's is the one that takes them with you, and the
    // spear's is the one that goes furthest.
    use sim::Class;
    use sim::moves::champion as c;
    let takeoffs =
        [c::RISING_CUT, c::UPPERCUT, c::POLE_DRIVE].map(|kind| moves::get(Class::Champion, kind));
    for m in &takeoffs {
        assert!(
            m.self_lift.raw() > 0,
            "{}: a takeoff that does not leave the ground",
            m.name
        );
    }
    let hardest = takeoffs.iter().max_by_key(|m| m.damage).unwrap();
    assert_eq!(
        hardest.name, "Rising cut",
        "the sword's takeoff is not the hardest-hitting of the three"
    );
    let highest = takeoffs.iter().max_by_key(|m| m.self_lift.raw()).unwrap();
    assert_eq!(
        highest.name, "Pole drive",
        "the spear's takeoff is not the highest of the three"
    );
    let holder = takeoffs.iter().max_by_key(|m| m.grabs).unwrap();
    assert_eq!(
        holder.name, "Uppercut",
        "the hammer's takeoff does not hold on to anybody"
    );
}
