//! What the classes leave behind.
//!
//! Three of the six are built on things that outlive the move that made them: a
//! fire pillar that grows where it was planted, a drain field that punishes
//! standing still, structures that change the shape of the arena. A fourth is
//! built on taking someone off the ground with you, and a fifth on holding on
//! to them. These are the assertions for all of it, phrased in terms of what
//! the fighters can do to each other rather than which number moved.

use sim::class::{Class, Mechanic};
use sim::effects::EffectKind;
use sim::state::{Action, MAX_PLAYERS, SLOT_SPECIAL};
use sim::{Input, World};

const Q: u16 = Input::SPECIAL;
const E: u16 = Input::MECHANIC;
const LOOK_RIGHT: u16 = 0;
const LOOK_LEFT: u16 = 1 << 15;

fn run(w: &mut World, frames: u32, a: u16, b: u16) {
    for _ in 0..frames {
        w.advance([Input::aimed(a, LOOK_RIGHT), Input::aimed(b, LOOK_LEFT)]);
    }
}

/// Press a button, let go, and let the move play out. Holding a button down
/// re-throws the move the frame you become free again, which is fine in a match
/// and useless in a test: two pillars is not the thing being asserted.
fn tap(w: &mut World, button: u16, then: u32) {
    run(w, 2, button, 0);
    run(w, then, 0, 0);
}

fn as_class(one: Class) -> World {
    World::with_classes([one, Class::Bulwark])
}

/// Walk player one into player two, so anything with reach connects.
fn engaged(one: Class) -> World {
    let mut w = as_class(one);
    run(&mut w, 90, Input::W, 0);
    w
}

fn effects_of(w: &World, kind: EffectKind) -> Vec<sim::effects::Effect> {
    w.effects
        .iter()
        .flatten()
        .filter(|e| e.kind == kind)
        .copied()
        .collect()
}

// ---------------------------------------------------------------------------
// The Elementalist
// ---------------------------------------------------------------------------

#[test]
fn the_fire_pillar_stands_after_the_move_is_over() {
    // The point of the class: the attack is over long before the thing it made
    // stops mattering. If the pillar died with the recovery frames it would
    // just be a slow poke.
    let mut w = engaged(Class::Elementalist);
    tap(&mut w, E, 4); // raise a structure -- the special needs one out
    tap(&mut w, Q, 60);
    assert!(
        w.players[0].action.actionable(),
        "still swinging 60 frames later, so this proves nothing"
    );
    assert_eq!(
        effects_of(&w, EffectKind::FirePillar).len(),
        1,
        "the pillar did not outlive the move that made it"
    );
}

#[test]
fn the_fire_pillar_spreads_at_the_base_and_climbs_at_the_top() {
    // Two volumes doing two jobs. The base gets wide, which is what catches
    // someone walking past it; the column gets tall, which is what stops them
    // jumping over. If both grew the same way the pillar would be one decision
    // instead of two.
    let mut w = engaged(Class::Elementalist);
    tap(&mut w, E, 4);
    tap(&mut w, Q, 30);
    let young = effects_of(&w, EffectKind::FirePillar)[0].pillar_volumes();
    run(&mut w, 120, 0, 0);
    let old = effects_of(&w, EffectKind::FirePillar)[0].pillar_volumes();

    assert!(
        old.0.radius.raw() > young.0.radius.raw(),
        "the base never spread, so there is nothing to walk around"
    );
    assert!(
        old.1.top.raw() > young.1.top.raw(),
        "the column never climbed, so it can be jumped for free forever"
    );
    let base_grew = old.0.radius.raw() - young.0.radius.raw();
    let column_grew = old.1.radius.raw() - young.1.radius.raw();
    assert!(
        base_grew > column_grew,
        "the column widened as much as the base did; the pillar is one threat, not two"
    );
}

#[test]
fn standing_in_a_fire_pillar_costs_you_and_standing_in_your_own_does_not() {
    // The pillar is placed at range, so both fighters have to walk into it --
    // which is the point: it is terrain, and terrain is something you choose to
    // be in. The caster can stand in her own, or she could never fight beside
    // the thing she just made.
    let mut w = as_class(Class::Elementalist);
    tap(&mut w, E, 4); // a structure, which the pillar needs
    tap(&mut w, Q, 20); // the pillar lands well ahead of her
    let caster_before = w.players[0].health;
    let victim_before = w.players[1].health;
    run(&mut w, 140, Input::W, Input::W); // both walk in

    assert!(
        w.players[1].health < victim_before,
        "walking into a fire pillar cost nothing"
    );
    assert_eq!(
        w.players[0].health, caster_before,
        "the Elementalist burned herself on her own pillar, so she cannot fight beside it"
    );
}

#[test]
fn a_bolt_aimed_through_a_fire_pillar_hits_as_a_fire_bolt() {
    // The auto reads what it is aimed through. A fire pillar is a hazard, not
    // a wall, so it charges the shot instead of stopping it -- unlike a
    // structure in the same spot. See docs/design/kits/elementalist.md.
    let mut w = as_class(Class::Elementalist);
    tap(&mut w, E, 4); // a structure, which the pillar needs
    tap(&mut w, Q, 20); // plant a pillar ahead, along the same aim as Bolt
    assert_eq!(
        effects_of(&w, EffectKind::FirePillar).len(),
        1,
        "fixture planted no pillar to aim through"
    );
    // Clear the structure the pillar needed: it still sits closer along the
    // same aim than the pillar it fed, and this test is isolating the
    // pillar's own effect on the shot rather than the structure's.
    w.players[0].mechanic = Mechanic::Structures([None; sim::class::MAX_STRUCTURES]);

    for _ in 0..60 {
        run(&mut w, 1, Input::LEFT, 0);
        if matches!(w.players[0].action, Action::Active { kind: 0, .. }) {
            break;
        }
    }
    assert!(
        matches!(w.players[0].action, Action::Active { kind: 0, .. }),
        "Bolt never became active"
    );
    assert!(
        w.players[0].bolt_fire,
        "a bolt aimed through a fire pillar was not empowered"
    );
    assert!(
        !w.players[0].bolt_blocked,
        "a fire pillar blocked the shot the way a structure does, which it should not"
    );
}

#[test]
fn a_structure_has_no_clock_and_keeps_the_special_alive() {
    // This is the regression. Structures used to live in the effects array,
    // which gave them a lifetime; the fire pillar is gated on having one out,
    // so ten seconds after raising a structure the Elementalist's own special
    // silently stopped working. A structure is a cap-of-three resource, and the
    // only thing that spends it is raising a fourth.
    let mut w = as_class(Class::Elementalist);
    tap(&mut w, E, 4);
    assert!(
        w.players[0].mechanic_ready(SLOT_SPECIAL),
        "fixture never raised a structure"
    );
    run(&mut w, 3_000, 0, 0); // fifty seconds of doing nothing
    assert!(
        w.players[0].mechanic_ready(SLOT_SPECIAL),
        "the structure went away on its own, and took the special with it"
    );
    tap(&mut w, Q, 60);
    assert_eq!(
        effects_of(&w, EffectKind::FirePillar).len(),
        1,
        "the special did not come out long after the structure was raised"
    );
}

#[test]
fn casting_until_the_board_is_full_never_costs_a_structure() {
    // The other half of the same bug: effects evicted the oldest of *anything*
    // when the board filled, and a structure was the oldest thing on it.
    let mut w = as_class(Class::Elementalist);
    tap(&mut w, E, 4);
    for _ in 0..20 {
        tap(&mut w, Q, 12);
        assert!(
            w.players[0].mechanic_ready(SLOT_SPECIAL),
            "casting the pillar ate the structure it needs"
        );
    }
}

#[test]
fn one_fighter_cannot_spam_away_the_other_fighters_field() {
    // Eviction reaches your own effects only. Otherwise holding a button would
    // delete someone else's setup, which is not a decision anybody made.
    let mut w = World::with_classes([Class::Elementalist, Class::BloodMage]);
    for _ in 0..90 {
        w.advance([
            Input::aimed(0, LOOK_RIGHT),
            Input::aimed(Input::W, LOOK_LEFT),
        ]);
    }
    for _ in 0..2 {
        w.advance([
            Input::aimed(0, LOOK_RIGHT),
            Input::aimed(Input::SHIFT | Input::LEFT, LOOK_LEFT),
        ]);
    }
    run(&mut w, 60, 0, 0);
    assert_eq!(
        effects_of(&w, EffectKind::BlackSpike).len(),
        1,
        "fixture laid no field"
    );

    tap(&mut w, E, 4);
    for _ in 0..20 {
        tap(&mut w, Q, 4);
    }
    assert_eq!(
        effects_of(&w, EffectKind::BlackSpike).len(),
        1,
        "the Elementalist deleted the Blood mage's field by holding a button"
    );
}

#[test]
fn a_fourth_structure_costs_the_first() {
    // The cap is the resource. It is enforced by the mechanic, which is the
    // only thing that owns structures.
    let mut w = as_class(Class::Elementalist);
    for _ in 0..4 {
        tap(&mut w, E, 2);
        run(&mut w, 10, Input::D, 0); // move, so each one lands somewhere new
    }
    let Mechanic::Structures(slots) = w.players[0].mechanic else {
        panic!("the Elementalist lost her mechanic");
    };
    assert_eq!(
        slots.iter().flatten().count(),
        sim::class::MAX_STRUCTURES,
        "the cap is not a cap"
    );
}

#[test]
fn the_mechanic_fires_on_the_press_not_while_the_button_is_down() {
    // Held, it used to re-fire every frame, and every class was wrong in its
    // own way: the Champion's form became a function of how many frames you
    // happened to hold it, the Reaver's shadow toggled itself back off, the
    // Bulwark's shield was pinned mid-throw and never planted, and the
    // Elementalist spent all three structures on one spot in three frames.
    for class in sim::class::ALL_CLASSES {
        let mut w = World::with_classes([class, Class::Bulwark]);
        let mut held = w.clone();
        run(&mut held, 40, E, 0);
        run(&mut w, 2, E, 0);
        run(&mut w, 38, 0, 0);
        assert_eq!(
            held.players[0].mechanic,
            w.players[0].mechanic,
            "{}: holding the mechanic did something different from tapping it",
            class.name()
        );
    }
}

#[test]
fn holding_the_mechanic_raises_exactly_one_structure() {
    let mut w = as_class(Class::Elementalist);
    run(&mut w, 40, E, 0);
    let Mechanic::Structures(slots) = w.players[0].mechanic else {
        panic!("the Elementalist lost her mechanic");
    };
    assert_eq!(
        slots.iter().flatten().count(),
        1,
        "one press should raise one structure"
    );
}

#[test]
fn a_second_press_raises_a_second_structure() {
    // The edge must not latch: letting go and pressing again has to work.
    let mut w = as_class(Class::Elementalist);
    tap(&mut w, E, 4);
    tap(&mut w, E, 4);
    let Mechanic::Structures(slots) = w.players[0].mechanic else {
        panic!("the Elementalist lost her mechanic");
    };
    assert_eq!(
        slots.iter().flatten().count(),
        2,
        "the press edge latched, so the mechanic only ever fires once"
    );
}

#[test]
fn a_structure_climbs_out_of_the_ground_and_then_stops_counting() {
    // It is earth. The age drives the rise and nothing else -- it must not
    // become a lifetime by the back door, so it saturates rather than wrapping.
    let mut w = as_class(Class::Elementalist);
    tap(&mut w, E, 0);
    let age_of = |w: &World| {
        let Mechanic::Structures(slots) = w.players[0].mechanic else {
            panic!("no mechanic")
        };
        slots.iter().flatten().next().expect("no structure").age
    };
    let fresh = age_of(&w);
    assert!(
        fresh < sim::tuning::structure_rise(),
        "the structure was already up on the frame it was raised"
    );
    run(&mut w, 60, 0, 0);
    assert!(
        age_of(&w) >= sim::tuning::structure_rise(),
        "the structure never finished rising"
    );
    run(&mut w, 3_000, 0, 0);
    assert!(
        w.players[0].mechanic_ready(SLOT_SPECIAL),
        "the age turned back into a lifetime"
    );
}

// ---------------------------------------------------------------------------
// The Blood mage
// ---------------------------------------------------------------------------

#[test]
fn the_black_spike_drains_and_slows_whoever_stands_in_it() {
    // Not a damage puddle: the slow is what makes it a wall. Leaving costs you
    // time, which is the whole reason to put one between yourself and someone.
    let mut w = engaged(Class::BloodMage);
    let before = w.players[1].health;
    tap(&mut w, Input::SHIFT | Input::LEFT, 80);
    assert_eq!(
        effects_of(&w, EffectKind::BlackSpike).len(),
        1,
        "black spike left no field"
    );
    assert!(w.players[1].health < before, "the field drained nobody");
    assert!(
        w.players[1].slowed > 0,
        "the field did not slow, so walking out of it is free"
    );
}

#[test]
fn a_slowed_fighter_covers_less_ground() {
    let mut w = engaged(Class::BloodMage);
    tap(&mut w, Input::SHIFT | Input::LEFT, 80);
    assert!(w.players[1].slowed > 0, "fixture did not slow anyone");

    let start = w.players[1].pos;
    run(&mut w, 6, 0, Input::W);
    let slowed = w.players[1].pos.sub(start).flat_len();

    // Out of the field, under the same input, for the same number of frames.
    run(&mut w, 240, 0, 0);
    let start = w.players[1].pos;
    run(&mut w, 6, 0, Input::W);
    let free = w.players[1].pos.sub(start).flat_len();

    assert!(
        free.raw() > slowed.raw(),
        "the slow did not slow: {} free vs {} in the field",
        free.to_f32_for_render(),
        slowed.to_f32_for_render()
    );
}

// ---------------------------------------------------------------------------
// The Champion
// ---------------------------------------------------------------------------

#[test]
fn the_uppercut_takes_both_fighters_off_the_ground() {
    // The move is a leap, and it is a leap you bring someone along on. Either
    // half alone is a different move: without the lift it is a launcher you
    // cannot follow up on, and without the launch it is an escape.
    let mut w = engaged(Class::Champion);
    let mut lifted = [false; MAX_PLAYERS];
    run(&mut w, 2, Q, 0);
    for _ in 0..60 {
        w.advance([Input::aimed(0, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
        for (i, seen) in lifted.iter_mut().enumerate() {
            *seen |= !w.players[i].grounded;
        }
    }
    assert!(lifted[0], "the uppercut never left the ground");
    assert!(
        lifted[1],
        "the uppercut connected but left its victim standing, so there is nothing to chase"
    );
}

// ---------------------------------------------------------------------------
// The Bulwark
// ---------------------------------------------------------------------------

#[test]
fn the_grapple_holds_on_and_its_victim_cannot_walk_out_of_it() {
    // A grab is not a shove. The victim is pinned to the grabber at arm's
    // length and stays there while they struggle, which is what makes beating
    // guard with it worth the commitment.
    let mut w = engaged(Class::Bulwark);
    run(&mut w, 2, Q, 0);
    let mut grabbed = false;
    for _ in 0..40 {
        w.advance([Input::aimed(0, LOOK_RIGHT), Input::aimed(0, LOOK_LEFT)]);
        grabbed |= matches!(w.players[1].action, Action::Held { .. });
        if grabbed {
            break;
        }
    }
    assert!(grabbed, "the grapple did not grab anyone");
    assert_eq!(
        w.players[1].held_by, 0,
        "somebody is held and nobody is holding them"
    );

    // Now let the victim try to run. Away is `S` for them -- they spawn facing
    // the other way.
    let gap_before = w.players[1].pos.sub(w.players[0].pos).flat_len();
    run(&mut w, 8, 0, Input::S);
    let gap_after = w.players[1].pos.sub(w.players[0].pos).flat_len();
    assert!(
        matches!(w.players[1].action, Action::Held { .. }),
        "the hold broke the moment its victim pressed a key"
    );
    let drift = (gap_after.raw() - gap_before.raw()).abs();
    assert!(
        drift < (sim::tuning::body_radius().raw() / 4),
        "the victim walked out of the grab: gap went {} -> {}",
        gap_before.to_f32_for_render(),
        gap_after.to_f32_for_render()
    );
}

#[test]
fn being_held_ends_and_hands_you_back_your_feet() {
    let mut w = engaged(Class::Bulwark);
    tap(&mut w, Q, 180);
    assert!(
        !matches!(w.players[1].action, Action::Held { .. }),
        "the grab never let go"
    );
    assert_eq!(
        w.players[1].held_by,
        sim::state::NOBODY,
        "let go but still tethered"
    );
}

// ---------------------------------------------------------------------------
// Rollback
// ---------------------------------------------------------------------------

#[test]
fn a_pillar_is_part_of_the_state_a_rollback_restores() {
    // If effects were not in the checksum, a peer could have a pillar the other
    // does not and neither would notice until someone died to it.
    let mut w = engaged(Class::Elementalist);
    tap(&mut w, E, 4);
    let bare = w.checksum();
    tap(&mut w, Q, 40);
    assert!(
        !effects_of(&w, EffectKind::FirePillar).is_empty(),
        "fixture made no pillar"
    );
    assert_ne!(
        bare,
        w.checksum(),
        "a world with a fire pillar in it hashes the same as one without"
    );

    let saved = w.clone();
    run(&mut w, 30, 0, 0);
    let advanced = w.checksum();
    w = saved.clone();
    assert_eq!(saved.checksum(), w.checksum(), "restore lost the effects");
    assert_ne!(advanced, w.checksum(), "the restore did not go back");
}
