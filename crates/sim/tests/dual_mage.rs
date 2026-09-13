//! The Dual mage's kit: two arms, two forces, one meter.
//!
//! Everything here is a property the class stops working without. She holds two
//! forces apart, one in each arm, and the *only* information the player has
//! about which one they just threw is which arm it came out of and which way
//! the meter moved. So: the two autos must sweep opposite sides of her, they
//! must be mirror images of each other, and the button pressed has to be the
//! thing that decides.
//!
//! See `docs/design/dual-mage.md` for the mechanic and
//! `docs/design/kits/dual-mage.md` for the kit.

use sim::aim::Hand;
use sim::class::{Class, Mechanic};
use sim::moves::dual;
use sim::state::{Action, Hitbox};
use sim::{Fx, Input, World};

const L: u16 = Input::LEFT;
const R: u16 = Input::RIGHT;
const Q: u16 = Input::SPECIAL;
const E: u16 = Input::MECHANIC;
const SHIFT: u16 = Input::SHIFT;

/// Looking down positive X, which is where a fighter spawns facing.
const LOOK: u16 = 0;

fn mage() -> World {
    World::with_classes([Class::DualMage, Class::Bulwark])
}

fn step(w: &mut World, frames: u32, bits: u16) {
    for _ in 0..frames {
        w.advance([Input::aimed(bits, LOOK), Input::new(0)]);
    }
}

/// Throw a move and collect the volume it has out on every active frame.
fn swept(bits: u16) -> Vec<Hitbox> {
    let mut w = mage();
    step(&mut w, 1, bits);
    let mut out = Vec::new();
    for _ in 0..90 {
        if let Some(box_out) = sim::state::hitbox(&w.players[0]) {
            out.push(box_out);
        } else if !out.is_empty() {
            break;
        }
        step(&mut w, 1, 0);
    }
    assert!(!out.is_empty(), "nothing came out for {bits:b}");
    out
}

/// How far to one side of the body's centre line a point is, in metres.
/// Positive is the side `Hand::Left` swings from.
fn sideways(w: &World, at: sim::V3) -> f32 {
    let p = &w.players[0];
    let across = sim::aim::across(p.facing, Hand::Left);
    at.sub(p.pos).dot(across).to_f32_for_render()
}

/// How far from the body's own axis a point is, level.
fn radius(w: &World, at: sim::V3) -> f32 {
    at.sub(w.players[0].pos).flat_len().to_f32_for_render()
}

/// How far in front of the body a point is.
fn ahead(w: &World, at: sim::V3) -> f32 {
    let p = &w.players[0];
    at.sub(p.pos).dot(p.facing).to_f32_for_render()
}

// ---------------------------------------------------------------------------
// Which button throws what
// ---------------------------------------------------------------------------

fn kind_thrown(bits: u16) -> u8 {
    let mut w = mage();
    step(&mut w, 1, bits);
    w.players[0]
        .action
        .attack_kind()
        .unwrap_or_else(|| panic!("no move came out for {bits:b}"))
}

#[test]
fn both_clicks_are_attacks_and_they_are_different_attacks() {
    // The class has no shield, so right click is not guard -- it is the other
    // half of the mechanic. A right click that did nothing would leave the
    // player able to travel in one direction along the meter only.
    assert_eq!(kind_thrown(L), dual::DARK_AUTO);
    assert_eq!(kind_thrown(R), dual::LIGHT_AUTO);
    assert_eq!(kind_thrown(L | SHIFT), dual::LANCE);
}

#[test]
fn q_and_e_both_throw_something() {
    // Two keys, two abilities. `E` is free because the meter is steered by
    // which button attacks rather than by a key of its own, and `Q` is the
    // finisher -- which is gated, so it is thrown from depth here.
    let mut w = mage();
    w.players[0].mechanic = Mechanic::Meter {
        value: sim::tuning::meter_max(),
    };
    step(&mut w, 1, Q);
    assert_eq!(w.players[0].action.attack_kind(), Some(dual::JUDGEMENT));

    let mut w = mage();
    step(&mut w, 1, E);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(dual::SWEEP),
        "`E` does nothing, so a third of the kit is unreachable"
    );
}

#[test]
fn the_finisher_is_the_only_thing_depth_gates() {
    // Everything else has to be throwable from a standing start, or the class
    // has no way to *reach* depth.
    let mut w = mage();
    for (bits, name) in [(L, "dark auto"), (R, "light auto"), (E, "sweep")] {
        w.players[0].mechanic = Mechanic::Meter { value: 0 };
        w.players[0].action = Action::Free;
        step(&mut w, 1, bits);
        assert!(
            w.players[0].action.attack_kind().is_some(),
            "{name} cannot be thrown from centre"
        );
        w.players[0].action = Action::Free;
    }
    w.players[0].mechanic = Mechanic::Meter { value: 0 };
    step(&mut w, 1, Q);
    assert_eq!(
        w.players[0].action.attack_kind(),
        None,
        "the finisher comes out at centre, so there is no reason to leave it"
    );
}

// ---------------------------------------------------------------------------
// Two arms
// ---------------------------------------------------------------------------

#[test]
fn the_two_autos_come_out_of_opposite_arms() {
    // The whole class in one assertion. A player reads which force they just
    // threw off which arm threw it, and the volume has to agree with the arm or
    // the read is a lie. The wing wraps *around* her, so what says "left" is
    // which side it sweeps through on its way to the front.
    for (bits, hand, name) in [(L, Hand::Left, "dark"), (R, Hand::Right, "light")] {
        let mut w = mage();
        step(&mut w, 1, bits);
        let frames = swept(bits);
        let mid = frames[frames.len() / 2];
        let side = sideways(&w, mid.to) * Fx::from_int(hand.outward()).to_f32_for_render();
        assert!(
            side > 0.5,
            "halfway through, the {name} wing is {side:.2} m along its own arm's side"
        );
    }
}

// ---------------------------------------------------------------------------
// The wing
// ---------------------------------------------------------------------------

#[test]
fn the_wing_is_a_section_of_a_ring_around_her() {
    // Not a line reaching out and not a swing across the front: a chunk of a
    // torus lying flat around the caster. Which means both of its radii are
    // *constant* while it sweeps -- a section that grew as it went would be a
    // spiral, and a spiral has no inside.
    let mut w = mage();
    step(&mut w, 1, L);
    let reach = sim::moves::get(Class::DualMage, dual::DARK_AUTO)
        .reach
        .to_f32_for_render();
    let mut inner_seen = Vec::new();
    for hb in swept(L) {
        let inner = radius(&w, hb.from);
        let outer = radius(&w, hb.to);
        assert!(
            (outer - reach).abs() < 0.02,
            "the outer arc is at {outer:.2} m and the move reaches {reach:.2} m"
        );
        assert!(
            inner > 0.15 && inner < outer * 0.5,
            "the ring has no hole in it: inner {inner:.2} m against outer {outer:.2} m"
        );
        inner_seen.push(inner);
    }
    let (lo, hi) = (
        inner_seen.iter().cloned().fold(f32::MAX, f32::min),
        inner_seen.iter().cloned().fold(0.0, f32::max),
    );
    assert!(
        hi - lo < 0.02,
        "the inner arc wanders between {lo:.2} m and {hi:.2} m, so this is a spiral rather than a ring"
    );
}

#[test]
fn the_wing_lies_flat() {
    // The plane of the torus is the floor's. Both ends of the section are at
    // the height the hand punches through, so it is a ring around her rather
    // than a cut through her.
    let mut w = mage();
    step(&mut w, 1, L);
    let hand = sim::tuning::cast_height().to_f32_for_render();
    for hb in swept(L) {
        let (a, b) = (hb.from.y.to_f32_for_render(), hb.to.y.to_f32_for_render());
        assert!(
            (a - b).abs() < 0.01,
            "the section is tilted: {a:.2} m at the inside, {b:.2} m at the outside"
        );
        assert!(
            (a - hand).abs() < 0.05,
            "the section is at {a:.2} m and the hand punches through {hand:.2} m"
        );
    }
}

#[test]
fn the_wing_starts_behind_her_and_ends_directly_ahead() {
    // The punch throws it and it overtakes the punch: it appears behind her, on
    // the arm's own side, and arrives in front of the fist on the last active
    // frame. A wing that started in front would just be a swing.
    for (bits, name) in [(L, "dark"), (R, "light")] {
        let mut w = mage();
        step(&mut w, 1, bits);
        let frames = swept(bits);
        let first = ahead(&w, frames[0].to);
        let last = frames[frames.len() - 1].to;
        assert!(
            first < -0.5,
            "the {name} wing appears {first:.2} m in front of her, not behind"
        );
        assert!(
            ahead(&w, last) > 0.9 * radius(&w, last),
            "the {name} wing does not finish pointing straight ahead"
        );
        assert!(
            sideways(&w, last).abs() < 0.15,
            "the {name} wing finishes {:.2} m off her centre line",
            sideways(&w, last)
        );
    }
}

#[test]
fn the_two_wings_are_mirror_images() {
    // One ring, two halves. If they were not mirrored, one of the two forces
    // would be better than the other for reasons nobody chose.
    let mut w = mage();
    step(&mut w, 1, L);
    let (dark, light) = (swept(L), swept(R));
    assert_eq!(dark.len(), light.len());
    for (a, b) in dark.iter().zip(light.iter()) {
        assert!(
            (ahead(&w, a.to) - ahead(&w, b.to)).abs() < 0.01,
            "the two wings are at different distances in front of her"
        );
        assert!(
            (sideways(&w, a.to) + sideways(&w, b.to)).abs() < 0.01,
            "the two wings are not mirrored: {:.2} against {:.2}",
            sideways(&w, a.to),
            sideways(&w, b.to)
        );
    }
}

#[test]
fn the_wing_sweeps_the_whole_way_round_without_jumping() {
    // A hundred and fifty degrees in six frames. It has to arrive in even
    // steps: a section that covered most of its arc in one frame would pass
    // through anybody standing in the rest of it without touching them.
    let mut w = mage();
    step(&mut w, 1, L);
    let steps: Vec<f32> = swept(L)
        .windows(2)
        .map(|pair| pair[1].to.sub(pair[0].to).flat_len().to_f32_for_render())
        .filter(|d| *d > 0.001)
        .collect();
    assert!(steps.len() >= 4, "the sweep is only {} steps", steps.len());
    let biggest = steps.iter().cloned().fold(0.0, f32::max);
    let smallest = steps.iter().cloned().fold(f32::MAX, f32::min);
    assert!(
        biggest < smallest * 1.5,
        "the sweep is uneven: steps from {smallest:.2} m to {biggest:.2} m"
    );
}

#[test]
fn the_wing_travels_with_the_body_rather_than_hanging_where_it_started() {
    // A swing is a body moving, and the autos keep sixty per cent of walking
    // speed. A ring anchored where the punch was thrown from would visibly
    // detach from the caster over six active frames.
    let mut w = mage();
    step(&mut w, 1, L | Input::W);
    let mut seen = Vec::new();
    for _ in 0..24 {
        if let Some(hb) = sim::state::hitbox(&w.players[0]) {
            seen.push((hb.from, w.players[0].pos));
        }
        step(&mut w, 1, L | Input::W);
    }
    assert!(seen.len() > 1, "the volume was never out for two frames");
    for (from, pos) in &seen {
        let offset = from.sub(*pos).flat_len().to_f32_for_render();
        assert!(
            offset < 0.5,
            "the inner arc is {offset:.2} m from the body it is meant to be centred on"
        );
    }
}

#[test]
fn nothing_else_in_the_roster_has_a_side() {
    // `Hand::Centre` is the default and has to stay the default: a swing that
    // quietly moved to one shoulder would change the reach of five classes.
    for class in sim::class::ALL_CLASSES {
        for slot in 0..sim::moves::slots(class) {
            let m = sim::moves::get(class, slot as u8);
            let sided = m.hand != Hand::Centre;
            let is_an_auto = class == Class::DualMage && dual::is_an_auto(slot as u8);
            assert_eq!(
                sided,
                is_an_auto,
                "{} {} is thrown from the {} hand",
                class.name(),
                m.name,
                m.hand.name()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Steering the meter
// ---------------------------------------------------------------------------

fn meter(w: &World) -> i32 {
    match w.players[0].mechanic {
        Mechanic::Meter { value } => value,
        other => panic!("not a meter: {other:?}"),
    }
}

/// Land `bits` on the other fighter, from close enough that it connects.
fn landed(bits: u16) -> World {
    let mut w = mage();
    // Walk in. Both spawn eight metres apart and the autos are melee.
    step(&mut w, 90, Input::W);
    let gap = w.players[1]
        .pos
        .sub(w.players[0].pos)
        .flat_len()
        .to_f32_for_render();
    assert!(gap < 2.0, "fixture failed to close: {gap:.2} m apart");
    let before = w.players[1].health;
    step(&mut w, 1, bits);
    step(&mut w, 12, 0);
    assert!(
        w.players[1].health < before,
        "the fixture never connected, so it proves nothing about contact"
    );
    w
}

#[test]
fn an_auto_steers_only_when_it_lands() {
    // The autos are the steering wheel and they are melee: a whiff steers
    // nothing, which is what forces the class to close distance exactly when
    // it is at its most powerful and most fragile.
    let mut whiffed = mage();
    step(&mut whiffed, 1, L);
    step(&mut whiffed, 30, 0);
    assert_eq!(
        meter(&whiffed),
        0,
        "a dark auto swung at nothing moved the meter"
    );

    assert!(meter(&landed(L)) < 0, "landing a dark auto did not go dark");
    assert!(
        meter(&landed(R)) > 0,
        "landing a light auto did not go light"
    );
}

#[test]
fn a_cast_steers_on_the_press() {
    // Everything that is not an auto votes when you commit to it. Lance is on
    // left click, so it is dark, and it does not have to connect: you paid for
    // it in frames.
    let mut w = mage();
    step(&mut w, 1, L | SHIFT);
    step(&mut w, 40, 0);
    assert!(meter(&w) < 0, "a Lance thrown at nothing steered nothing");
}

#[test]
fn a_move_with_no_side_pushes_you_further_along_the_way_you_were_going() {
    // `Q` and `E` are keys, and a key has no side. The rule the design states
    // for every input that is neither left nor right: it pushes you further
    // along whichever path you are already on.
    for start in [-20, 20] {
        let mut w = mage();
        w.players[0].mechanic = Mechanic::Meter { value: start };
        step(&mut w, 1, E);
        step(&mut w, 40, 0);
        let moved = meter(&w) - start;
        assert!(
            moved.signum() == start.signum(),
            "from {start} a sweep moved the meter by {moved}, which is back toward centre"
        );
    }
    // And at dead centre it does nothing, because there is no path to be on.
    let mut w = mage();
    step(&mut w, 1, E);
    step(&mut w, 40, 0);
    assert_eq!(meter(&w), 0, "a sideless cast picked a side at centre");
}

#[test]
fn the_meter_burns_at_depth_and_stops_when_you_come_back() {
    // The containment story: relief comes from stopping, not from a reward.
    let mut w = mage();
    w.players[0].mechanic = Mechanic::Meter {
        value: sim::tuning::meter_max(),
    };
    let before = w.players[0].health;
    step(&mut w, 30, 0);
    assert!(w.players[0].health < before, "the edge costs nothing");

    w.players[0].mechanic = Mechanic::Meter { value: 0 };
    let inside = w.players[0].health;
    step(&mut w, 30, 0);
    assert_eq!(
        w.players[0].health, inside,
        "the burn followed her back to centre"
    );
}

#[test]
fn steering_cannot_be_pushed_past_the_ends_of_the_bar() {
    let mut w = mage();
    for _ in 0..40 {
        w.players[0].action = Action::Free;
        step(&mut w, 1, L | SHIFT);
    }
    assert_eq!(meter(&w), -sim::tuning::meter_max());
    assert!(w.players[0].health > 0, "she burned herself to death");
}

#[test]
fn the_burn_can_never_be_what_kills_her() {
    // The same rule the Blood mage's costs follow: dying to your own button is
    // not a decision anybody made.
    let mut w = mage();
    w.players[0].mechanic = Mechanic::Meter {
        value: sim::tuning::meter_max(),
    };
    w.players[0].health = 2;
    step(&mut w, 600, 0);
    assert_eq!(w.players[0].health, 1);
}

// ---------------------------------------------------------------------------
// The numbers the kit depends on
// ---------------------------------------------------------------------------

#[test]
fn the_two_autos_are_the_same_move_on_opposite_arms() {
    // They are mirrored on purpose, and the mirror has to hold through tuning:
    // a light auto that hit harder than the dark one would make the meter a
    // damage decision rather than a positional one.
    let dark = sim::moves::get(Class::DualMage, dual::DARK_AUTO);
    let light = sim::moves::get(Class::DualMage, dual::LIGHT_AUTO);
    for (what, a, b) in [
        ("startup", dark.startup as i32, light.startup as i32),
        ("active", dark.active as i32, light.active as i32),
        ("recovery", dark.recovery as i32, light.recovery as i32),
        ("damage", dark.damage, light.damage),
        ("reach", dark.reach.raw(), light.reach.raw()),
        ("radius", dark.radius.raw(), light.radius.raw()),
        ("arc", dark.arc.raw(), light.arc.raw()),
    ] {
        assert_eq!(a, b, "the two autos disagree about {what}");
    }
    assert!(
        dark.arc.raw() > 0,
        "an auto with no arc is a punch with no wing"
    );
    assert_ne!(dark.hand, light.hand);
}

#[test]
fn the_autos_reach_further_than_the_body_they_are_thrown_from() {
    // "Autos have a slight range boost, powered by the beings inside." It is
    // mechanical rather than decorative: steering depends on connecting.
    let dark = sim::moves::get(Class::DualMage, dual::DARK_AUTO);
    assert!(
        dark.reach.raw() > Fx::from_int(1).raw(),
        "the auto reaches {} m",
        dark.reach.to_f32_for_render()
    );
}
