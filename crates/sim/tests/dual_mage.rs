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
use sim::class::{Class, Force, Mechanic};
use sim::moves::dual;
use sim::state::{Action, Hitbox};
use sim::{Fx, Input, V3, World};

const L: u16 = Input::LEFT;
const R: u16 = Input::RIGHT;
const Q: u16 = Input::SPECIAL;
const E: u16 = Input::MECHANIC;
/// Middle click, which is where Lance lives now: an input with no side, for the
/// one cast whose form comes from the force she is carrying rather than from
/// the button. Shift is a dodge and nothing else -- see `docs/design/controls.md`.
const M: u16 = Input::MIDDLE;

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

/// How far along one arm's own side of the body a point is, in metres.
///
/// Positive is away from the centre line on `hand`'s side. Measured against the
/// hand being asked about rather than against a fixed one, so a test says which
/// arm it means instead of carrying a sign to correct for it.
fn sideways(w: &World, hand: Hand, at: sim::V3) -> f32 {
    let p = &w.players[0];
    at.sub(p.pos)
        .dot(sim::aim::across(p.facing, hand))
        .to_f32_for_render()
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
}

#[test]
fn middle_click_throws_the_lance_the_force_she_carries_decides() {
    // **The one input in the game that is two moves.** Middle click has no
    // side, so it cannot pick a direction on the bar -- which is exactly what
    // makes it the right home for the cast whose *form* is the force in her
    // arms. Shift used to carry it and the two rules fought: a committed cast
    // on left click had a side, so it pushed her dark whatever she was
    // holding, and the light form of it had nowhere to live at all.
    for (force, want, name) in [
        (Force::Light, dual::LIGHT_LANCE, "light"),
        (Force::Dark, dual::DARK_LANCE, "dark"),
    ] {
        let mut w = mage();
        w.players[0].mechanic = meter_at(0, force);
        step(&mut w, 1, M);
        assert_eq!(
            w.players[0].action.attack_kind(),
            Some(want),
            "carrying the {name}, middle click threw the wrong form"
        );
    }
}

#[test]
fn shift_is_only_a_dodge_now() {
    // The grammar change, checked from the class it cost a binding: shift plus
    // a click is not an attack input any more, on any class, so shift plus left
    // click is the bare left click's move. See `docs/design/controls.md`.
    assert_eq!(kind_thrown(L | Input::SHIFT), dual::DARK_AUTO);
    assert_eq!(kind_thrown(R | Input::SHIFT), dual::LIGHT_AUTO);
}

#[test]
fn q_and_e_both_throw_something() {
    // Two keys, two abilities. `E` is free because the meter is steered by
    // which button attacks rather than by a key of its own, and `Q` is the
    // finisher.
    let mut w = mage();
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
fn nothing_in_the_kit_is_locked_at_the_centre() {
    // Judgement used to need the bar deep on one side before it would come out,
    // which meant `Q` did nothing at all for the opening of every match -- the
    // player pressing it had no way to tell an ability from an empty key. The
    // finisher is the class's special; a special you cannot press is not one.
    let mut w = mage();
    for (bits, name) in [
        (L, "dark auto"),
        (R, "light auto"),
        (E, "sweep"),
        (Q, "judgement"),
        (M, "lance"),
    ] {
        w.players[0].mechanic = meter_at(0, Force::Dark);
        w.players[0].action = Action::Free;
        step(&mut w, 1, bits);
        assert!(
            w.players[0].action.attack_kind().is_some(),
            "{name} cannot be thrown from centre"
        );
        w.players[0].action = Action::Free;
    }
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
        let side = sideways(&w, hand, mid.to);
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
fn the_wing_is_a_thin_band_rather_than_a_filled_section() {
    // Not a line reaching out and not a swing across the front: a chunk of a
    // torus lying flat, and a **thin** one. The hole in it is most of the ring.
    //
    // It used to reach from her own elbow out to full range on every frame,
    // which is a filled disc with a pinhole in it -- an attack with no inside,
    // catching anybody standing anywhere in the quadrant. What the shape is
    // meant to be is a curved blade travelling through the air, and a blade has
    // a near edge as well as a far one.
    //
    // Both radii are constant while it sweeps, measured from the ring's own
    // middle. A section that grew as it went would be a spiral, and a spiral
    // has no inside either.
    let reach = sim::moves::get(Class::DualMage, dual::DARK_AUTO)
        .reach
        .to_f32_for_render();
    let mut seen = Vec::new();
    for hb in swept(L) {
        // The last active frame is the tip, which is a bubble rather than a
        // section -- see `the_last_frame_is_the_tip_and_it_is_a_bubble`.
        let Some(ring) = hb.sector else { continue };
        let (inner, outer) = (
            ring.inner.to_f32_for_render(),
            ring.outer.to_f32_for_render(),
        );
        assert!(
            (outer - reach).abs() < 0.02,
            "the outer arc is at {outer:.2} m and the move reaches {reach:.2} m"
        );
        assert!(
            inner > outer * 0.6,
            "the band runs {inner:.2} m to {outer:.2} m, which is a pie slice rather than a blade"
        );
        seen.push((inner, outer));
    }
    assert!(seen.len() >= 5, "only {} frames of section", seen.len());
    let first = seen[0];
    for (inner, outer) in &seen {
        assert!(
            (inner - first.0).abs() < 0.02 && (outer - first.1).abs() < 0.02,
            "the band wanders from {:.2}-{:.2} m to {inner:.2}-{outer:.2} m, \
             so this is a spiral rather than a ring",
            first.0,
            first.1
        );
    }
}

#[test]
fn she_is_not_the_middle_of_the_ring_and_the_middle_travels_with_her() {
    // Two properties of one number, and they pull opposite ways.
    //
    // A ring centred on a fighter is the same distance from them at every
    // bearing, so however short you make the section it reads as a piece of a
    // halo rather than as something thrown. The middle is pushed off her --
    // toward the other arm and forward -- which is what makes the blade come in
    // close beside the punching fist and swing wide in front of her.
    //
    // But it is pushed off *her*, live. The autos keep sixty per cent of
    // walking speed, and a ring anchored where the punch was thrown from would
    // visibly detach from the caster over six active frames.
    let mut w = mage();
    let mut seen = Vec::new();
    for _ in 0..24 {
        if let Some(ring) = sim::state::hitbox(&w.players[0]).and_then(|hb| hb.sector) {
            let off = ring.at.sub(w.players[0].pos);
            seen.push((
                ahead(&w, ring.at),
                sideways(&w, Hand::Left, ring.at),
                V3::new(off.x, Fx::ZERO, off.z)
                    .flat_len()
                    .to_f32_for_render(),
            ));
        }
        // Held down and walking the whole time: she throws the auto, keeps
        // walking through it, and throws it again.
        step(&mut w, 1, L | Input::W);
    }
    assert!(
        seen.len() >= 5,
        "the section was out for {} frames",
        seen.len()
    );
    let (ahead_of_her, beside_her, _) = seen[0];
    assert!(
        beside_her < -0.2,
        "the ring's middle is {beside_her:.2} m toward the punching arm, so the \
         blade wraps her rather than passing her"
    );
    assert!(
        ahead_of_her > 0.0,
        "the ring's middle is {ahead_of_her:.2} m in front of her"
    );
    for (a, b, gap) in &seen {
        assert!(
            (a - ahead_of_her).abs() < 0.02 && (b - beside_her).abs() < 0.02,
            "the ring's middle slid to {a:.2} m ahead and {b:.2} m across, from \
             {ahead_of_her:.2} and {beside_her:.2}"
        );
        assert!(
            *gap < 1.0,
            "the ring's middle is {gap:.2} m from the body it hangs off"
        );
    }
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
fn the_wing_starts_behind_her_and_finishes_in_front_of_its_own_fist() {
    // The punch throws it and it overtakes the punch: it appears behind her, on
    // the arm's own side, and arrives in front of that arm's hand on the last
    // active frame. A wing that started in front would just be a swing.
    //
    // **In front of the hand, not the sternum.** The two autos are told apart by
    // which arm threw them -- that is the entire mechanic -- and one that
    // finished on her centre line put both of them in the same place at the
    // moment the player is reading which one landed.
    let hand = sim::tuning::hand_offset().to_f32_for_render();
    let reach = sim::moves::get(Class::DualMage, dual::DARK_AUTO)
        .reach
        .to_f32_for_render();
    for (bits, side, name) in [(L, Hand::Left, "dark"), (R, Hand::Right, "light")] {
        let mut w = mage();
        step(&mut w, 1, bits);
        let frames = swept(bits);
        let first = ahead(&w, frames[0].to);
        assert!(
            first < 0.0,
            "the {name} wing appears {first:.2} m in front of her, not behind"
        );
        let from_the_arm = sideways(&w, side, frames[0].to);
        assert!(
            from_the_arm > 0.5,
            "the {name} wing appears {from_the_arm:.2} m along its own arm's side"
        );
        let last = frames[frames.len() - 1];
        assert!(last.tipper, "the {name} wing does not end on its tip");
        assert!(
            ahead(&w, last.to) > reach * 0.8,
            "the {name} wing finishes {:.2} m in front of her, of a {reach:.2} m reach",
            ahead(&w, last.to)
        );
        let finish = sideways(&w, side, last.to);
        assert!(
            finish > hand * 0.5 && finish < hand + 0.5,
            "the {name} wing finishes {finish:.2} m off her centre line and her \
             hand is {hand:.2} m off it"
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
            (sideways(&w, Hand::Left, a.to) + sideways(&w, Hand::Left, b.to)).abs() < 0.01,
            "the two wings are not mirrored: {:.2} against {:.2}",
            sideways(&w, Hand::Left, a.to),
            sideways(&w, Hand::Left, b.to)
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
    // The wing's own frames. The last one is the tip arriving, which covers
    // the rest of the arc in one go on purpose.
    let frames = swept(L);
    let steps: Vec<f32> = frames[..frames.len() - 1]
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
fn a_side_is_always_declared_and_the_list_is_short() {
    // `Hand::Centre` is the default and has to stay the default: a swing that
    // quietly moved to one shoulder would change the reach of whichever class it
    // happened to. So the sided moves are **named here**, and a new one is a
    // visible one-line change to this list rather than something a reader of the
    // move table has to go looking for.
    //
    // Five, and four of them are the Dual mage. Her autos are sided because the
    // side *is* the mechanic -- left is dark, right is light, and a hitbox on
    // the centre line cannot say which one landed. Her two Lances are sided for
    // the same reason one step on: they are one input, middle click, told apart
    // by the force she is carrying, and the arm the line leaves from is the
    // second reading of which of the two is coming. The Champion's spear opener
    // is sided because it is the one attack in that class thrown with one arm:
    // a jab off the leading hand with the butt of the shaft still at the hip,
    // which is most of why it reads as a poke rather than as a short lunge.
    let declared: &[(Class, u8)] = &[
        (Class::DualMage, dual::DARK_AUTO),
        (Class::DualMage, dual::LIGHT_AUTO),
        (Class::DualMage, dual::LIGHT_LANCE),
        (Class::DualMage, dual::DARK_LANCE),
        (Class::Champion, sim::moves::champion::SPEAR_GROUND),
    ];
    for class in sim::class::ALL_CLASSES {
        for slot in 0..sim::moves::slots(class) {
            let m = sim::moves::get(class, slot as u8);
            let sided = m.hand != Hand::Centre;
            let named = declared.contains(&(class, slot as u8));
            assert_eq!(
                sided,
                named,
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
        Mechanic::Meter { value, .. } => value,
        other => panic!("not a meter: {other:?}"),
    }
}

fn colour(w: &World) -> Force {
    match w.players[0].mechanic {
        Mechanic::Meter { colour, .. } => colour,
        other => panic!("not a meter: {other:?}"),
    }
}

fn ascending(w: &World) -> u16 {
    match w.players[0].mechanic {
        Mechanic::Meter { ascending, .. } => ascending,
        other => panic!("not a meter: {other:?}"),
    }
}

fn meter_at(value: i32, colour: Force) -> Mechanic {
    Mechanic::Meter {
        value,
        colour,
        ascending: 0,
    }
}

#[test]
fn every_attack_moves_the_bar_with_nothing_in_range() {
    // **The bug this class shipped with.** The autos only steered on contact
    // and the casts took their direction from an auto that had never landed, so
    // a player standing where they spawn -- eight metres from anybody -- could
    // press every button on the class and watch the bar sit at zero. A resource
    // you cannot move without a target is one nobody can learn or tune.
    for (bits, name) in [
        (L, "dark auto"),
        (R, "light auto"),
        (M, "lance"),
        (Q, "judgement"),
        (E, "sweep"),
    ] {
        let mut w = mage();
        let gap = w.players[1]
            .pos
            .sub(w.players[0].pos)
            .flat_len()
            .to_f32_for_render();
        assert!(gap > 4.0, "the fixture is in range, so it proves nothing");
        step(&mut w, 1, bits);
        step(&mut w, 60, 0);
        assert_ne!(
            meter(&w),
            0,
            "{name} thrown at nothing moved the bar not at all"
        );
    }
}

#[test]
fn an_auto_moves_the_bar_by_exactly_one_step() {
    // The bar is calibrated in autos: the thing you throw constantly is the
    // unit everything else is measured against.
    let one = sim::tuning::meter_auto_push();
    for (bits, force, name) in [(L, Force::Dark, "dark"), (R, Force::Light, "light")] {
        let mut w = mage();
        step(&mut w, 1, bits);
        step(&mut w, 30, 0);
        assert_eq!(meter(&w), one * force.along(), "the {name} auto");
    }
}

#[test]
fn the_last_auto_she_threw_is_the_force_she_is_carrying() {
    // The mechanic in one sentence. Her casts are made of whichever force she
    // is carrying, and the autos are the only thing that sets it.
    let mut w = mage();
    step(&mut w, 1, R);
    step(&mut w, 30, 0);
    assert_eq!(colour(&w), Force::Light);
    step(&mut w, 1, L);
    step(&mut w, 30, 0);
    assert_eq!(colour(&w), Force::Dark);
}

#[test]
fn she_is_carrying_a_force_before_she_has_thrown_anything() {
    // Otherwise the first key pressed in a match has no direction to push in,
    // which is half of how the bar came to be unmovable.
    let w = mage();
    assert_eq!(colour(&w), Force::Dark);
    assert_eq!(meter(&w), 0);
}

#[test]
fn a_cast_moves_her_further_than_an_auto_does_and_in_the_force_she_carries() {
    // Committing to a move is committing harder to a side than poking is. And
    // it goes the way the *last auto* left her, not the way the button that
    // threw it faces -- a Lance thrown by a mage carrying the light is light.
    let cast = sim::tuning::meter_cast_push();
    assert!(
        cast > sim::tuning::meter_auto_push(),
        "a cast moves her no further than an auto"
    );
    for (force, bits, name) in [(Force::Dark, M, "Lance"), (Force::Light, E, "Sweep")] {
        for carrying in [Force::Dark, Force::Light] {
            let mut w = mage();
            w.players[0].mechanic = meter_at(0, carrying);
            step(&mut w, 1, bits);
            step(&mut w, 40, 0);
            assert_eq!(
                meter(&w),
                cast * carrying.along(),
                "{name} thrown while carrying the {} went the wrong way",
                carrying.name()
            );
            assert_eq!(colour(&w), carrying, "a cast changed her force");
        }
        let _ = force;
    }
}

#[test]
fn the_meter_burns_at_depth_and_stops_when_you_come_back() {
    // The containment story: relief comes from stopping, not from a reward.
    let mut w = mage();
    w.players[0].mechanic = meter_at(sim::tuning::meter_deep() + 20, Force::Light);
    let before = w.players[0].health;
    step(&mut w, 30, 0);
    assert!(w.players[0].health < before, "the edge costs nothing");

    w.players[0].mechanic = meter_at(0, Force::Dark);
    let inside = w.players[0].health;
    step(&mut w, 30, 0);
    assert_eq!(
        w.players[0].health, inside,
        "the burn followed her back to centre"
    );
}

// ---------------------------------------------------------------------------
// Ascension
// ---------------------------------------------------------------------------

#[test]
fn driving_the_bar_to_an_end_starts_ascension() {
    // There is no ascend button and there never was: you got there one cast at
    // a time. Reaching the end *is* the input.
    for force in [Force::Dark, Force::Light] {
        let mut w = mage();
        let brink = (sim::tuning::meter_max() - 1) * force.along();
        w.players[0].mechanic = meter_at(brink, force);
        step(&mut w, 1, E);
        assert!(
            ascending(&w) > 0,
            "the {} end of the bar did nothing",
            force.name()
        );
    }
}

#[test]
fn ascension_ends_on_its_own_clock() {
    // The thing it did not do before. It drained until she was nearly dead and
    // then went on draining, with nothing she could do about it and no way to
    // tell it had happened.
    let mut w = mage();
    w.players[0].mechanic = meter_at(sim::tuning::meter_max() - 1, Force::Light);
    step(&mut w, 1, E);
    let clock = ascending(&w);
    assert_eq!(meter(&w), sim::tuning::meter_max(), "she is not at the end");
    assert!(clock > 0);

    // Right up to the last frame it is still running.
    step(&mut w, clock as u32 - 1, 0);
    assert!(ascending(&w) > 0, "it ended early");
    step(&mut w, 1, 0);
    assert_eq!(ascending(&w), 0, "it did not end");
    assert_eq!(meter(&w), 0, "it did not put her back at the centre");
    assert!(
        matches!(w.players[0].action, Action::Stagger { .. }),
        "she walked out of it as though nothing had happened"
    );
}

#[test]
fn ascension_costs_between_half_and_all_of_the_health_bar() {
    // The design's own number, and the reason it can be a clock at all: **the
    // drain is the timer**. Under half and there is nothing at stake; over the
    // whole bar and it would be a suicide button rather than a gamble.
    let cost = sim::tuning::ascension_drain() * sim::tuning::ascension_frames() as i32;
    let bar = sim::tuning::max_health();
    assert!(
        cost * 2 >= bar && cost <= bar,
        "ascension costs {cost} of a {bar} health bar"
    );
}

#[test]
fn ascension_drains_faster_than_the_edge_burns() {
    // Two different things happen at depth and they have to feel different, or
    // there is no reason to fear the end of the bar over merely leaning on it.
    let mut edge = mage();
    edge.players[0].mechanic = meter_at(sim::tuning::meter_max() - 1, Force::Light);
    let before = edge.players[0].health;
    step(&mut edge, 60, 0);
    let burned = before - edge.players[0].health;

    let mut gone = mage();
    gone.players[0].mechanic = Mechanic::Meter {
        value: sim::tuning::meter_max(),
        colour: Force::Dark,
        ascending: sim::tuning::ascension_frames(),
    };
    let before = gone.players[0].health;
    step(&mut gone, 60, 0);
    let drained = before - gone.players[0].health;

    assert!(
        drained > burned,
        "ascension took {drained} where the edge took {burned}"
    );
}

#[test]
fn nothing_steers_the_bar_while_she_is_ascended() {
    // The meter is not the operative resource then -- the clock is.
    let mut w = mage();
    w.players[0].mechanic = Mechanic::Meter {
        value: sim::tuning::meter_max(),
        colour: Force::Dark,
        ascending: sim::tuning::ascension_frames(),
    };
    step(&mut w, 1, R);
    step(&mut w, 20, 0);
    assert_eq!(meter(&w), sim::tuning::meter_max());
}

#[test]
fn steering_cannot_be_pushed_past_the_ends_of_the_bar() {
    let mut w = mage();
    for _ in 0..40 {
        w.players[0].action = Action::Free;
        step(&mut w, 1, M);
    }
    assert!(meter(&w).abs() <= sim::tuning::meter_max());
    assert!(w.players[0].health > 0, "she burned herself to death");
}

#[test]
fn neither_the_burn_nor_ascension_can_be_what_kills_her() {
    // The same rule the Blood mage's costs follow: dying to your own button is
    // not a decision anybody made.
    for mechanic in [
        meter_at(sim::tuning::meter_max(), Force::Light),
        Mechanic::Meter {
            value: sim::tuning::meter_max(),
            colour: Force::Light,
            ascending: sim::tuning::ascension_frames(),
        },
    ] {
        let mut w = mage();
        w.players[0].mechanic = mechanic;
        w.players[0].health = 2;
        step(&mut w, 600, 0);
        assert_eq!(w.players[0].health, 1);
    }
}

// ---------------------------------------------------------------------------
// The numbers the kit depends on
// ---------------------------------------------------------------------------

#[test]
fn the_two_autos_keep_one_set_of_frames_and_one_shape() {
    // They are one punch mirrored, and the mirror has to hold through tuning:
    // a light auto that came out faster or reached further than the dark one
    // would make which arm you throw a *speed* decision, and the arm is
    // supposed to be the meter and the spacing and nothing else.
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
        ("hitstun", dark.hitstun as i32, light.hitstun as i32),
        ("blockstun", dark.blockstun as i32, light.blockstun as i32),
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
fn one_auto_pulls_and_the_other_pushes() {
    // **The one thing the two of them may differ about**, and the reason the
    // test above lists everything else. A fragile melee mage stays attached to
    // somebody by dragging them in; she buys herself room by shoving them out.
    // Putting those on the two buttons she presses constantly is what makes
    // which arm she punches with a spacing decision as well as a meter one.
    let dark = sim::moves::get(Class::DualMage, dual::DARK_AUTO);
    let light = sim::moves::get(Class::DualMage, dual::LIGHT_AUTO);
    assert!(
        dark.knockback.raw() < 0,
        "the dark auto shoves rather than pulls, so she has no way to stay attached"
    );
    assert!(
        light.knockback.raw() > 0,
        "the light auto pulls rather than shoves, so both arms do the same thing"
    );
    assert!(
        dark.leech > 0,
        "the dark auto returns nothing, so the arm that keeps her in range does not keep her alive"
    );
}

/// Throw one auto at somebody standing where the blade sweeps, from `at` on
/// the bar, and give back how much further away they ended up.
///
/// Negative means it brought them in. The fixture puts them out at most of the
/// reach and off to the punching arm's side, which is the shape of the wing --
/// see `the_two_autos_come_out_of_opposite_arms`.
fn gap_change(bits: u16, hand: Hand, at: i32) -> f32 {
    let mut w = mage();
    w.players[0].mechanic = meter_at(at, Force::Dark);
    let reach = sim::moves::get(Class::DualMage, dual::DARK_AUTO).reach;
    let out = w.players[0].facing.scale(reach.mul(Fx::ratio(3, 5)));
    let side = sim::aim::across(w.players[0].facing, hand).scale(reach.mul(Fx::ratio(3, 5)));
    w.players[1].pos = w.players[0].pos.add(out).add(side);
    let gap = |w: &World| {
        w.players[1]
            .pos
            .sub(w.players[0].pos)
            .flat_len()
            .to_f32_for_render()
    };
    let before = gap(&w);
    step(&mut w, 1, bits);
    step(&mut w, 24, 0);
    assert!(
        w.players[1].health < sim::state::max_health(),
        "the auto did not connect, so this proves nothing"
    );
    gap(&w) - before
}

#[test]
fn the_dark_auto_drags_them_in_and_the_light_one_sends_them_out() {
    // **The two autos are the steering wheel and they do opposite things to the
    // spacing.** Same frames, same shape, mirrored arms -- and one pulls while
    // the other pushes, which is what makes which arm you punch with a spacing
    // decision as well as a meter one. A fragile melee mage stays attached to
    // somebody with the dark hand and buys herself room with the light one.
    for (bits, hand, name, closer) in [
        (L, Hand::Left, "dark", true),
        (R, Hand::Right, "light", false),
    ] {
        let shifted = gap_change(bits, hand, 0);
        if closer {
            assert!(
                shifted < 0.0,
                "the {name} auto left them {shifted:+.2} m further off -- it is supposed to pull"
            );
        } else {
            assert!(
                shifted > 0.0,
                "the {name} auto left them {shifted:+.2} m further off -- it is supposed to push"
            );
        }
    }
}

#[test]
fn depth_is_in_the_hands_and_not_only_on_the_bar() {
    // The class's founding idea, and the thing it shipped without: the same
    // punch thrown from the edge of the bar has to be a **bigger** punch than
    // the one thrown from the centre. Checked on the two autos rather than on
    // a cast because the autos are the one thing in the kit that does not grow
    // -- their reach is pinned to the animation -- so what depth does to them
    // is purely what they *do*, which is the half of the rule that has to be
    // true everywhere.
    let deep = sim::tuning::meter_deep();
    for (bits, hand, name) in [(L, Hand::Left, "dark"), (R, Hand::Right, "light")] {
        let centre = gap_change(bits, hand, 0).abs();
        let edge = gap_change(bits, hand, -deep).abs();
        assert!(
            edge > centre * 1.2,
            "the {name} auto moved them {centre:.2} m from the centre and {edge:.2} m from \
             depth: riding the edge buys nothing you can feel"
        );
    }
}

/// Where to stand to be hit by one of her moves, as an offset from her.
///
/// Per move, because the five of them reach in five different shapes and a
/// single fixture distance would be inside the wing's hole and outside Sweep at
/// the same time. `None` means "wherever the cast lands" -- the one aimed at
/// the floor has to be measured against where it actually goes, since how far
/// it goes is itself on the depth curve.
fn stand_for(bits: u16, w: &World) -> Option<V3> {
    let reach = |kind: u8| sim::moves::get(Class::DualMage, kind).reach;
    let p = &w.players[0];
    let out = |d: Fx| p.facing.scale(d);
    Some(match bits {
        b if b == L => out(reach(dual::DARK_AUTO).mul(Fx::ratio(3, 5))).add(
            sim::aim::across(p.facing, Hand::Left)
                .scale(reach(dual::DARK_AUTO).mul(Fx::ratio(3, 5))),
        ),
        b if b == R => out(reach(dual::LIGHT_AUTO).mul(Fx::ratio(3, 5))).add(
            sim::aim::across(p.facing, Hand::Right)
                .scale(reach(dual::LIGHT_AUTO).mul(Fx::ratio(3, 5))),
        ),
        // On the line, well inside the shortest the Lance is ever thrown.
        b if b == M => out(Fx::from_int(3)),
        b if b == E => out(reach(dual::SWEEP).mul(Fx::ratio(1, 2))),
        _ => return None,
    })
}

/// What one of her moves takes off somebody, thrown from `at` on the bar.
fn dealt_from(bits: u16, at: i32) -> i32 {
    let mut w = mage();
    w.players[0].mechanic = meter_at(at, Force::Dark);
    if let Some(offset) = stand_for(bits, &w) {
        w.players[1].pos = w.players[0].pos.add(offset);
    }
    let before = w.players[1].health;
    step(&mut w, 1, bits);
    // A move aimed at the floor is measured against where it landed, which is
    // itself further out the deeper she is standing.
    if stand_for(bits, &w).is_none() {
        let lands = w.players[0].aim_at();
        w.players[1].pos = V3::new(lands.x, w.players[1].pos.y, lands.z);
    }
    step(&mut w, 150, 0);
    before - w.players[1].health
}

#[test]
fn a_cast_at_the_centre_is_a_weaker_cast_than_one_at_the_edge() {
    // **The largest hole the class shipped with, closed.** Every move she has,
    // not one at a time: at the centre everything she throws is thin, and at the
    // edge it is the most she can hold. Measured end to end -- health actually
    // taken off somebody -- rather than off the multiplier, because the point is
    // that the curve reaches all the way through to the thing that hurts.
    let deep = sim::tuning::meter_deep();
    for (bits, name) in [
        (L, "dark auto"),
        (R, "light auto"),
        (M, "lance"),
        (E, "sweep"),
        (Q, "judgement"),
    ] {
        let centre = dealt_from(bits, 0);
        let edge = dealt_from(bits, -deep);
        assert!(centre > 0, "{name} never connected at the centre");
        assert!(
            edge > centre,
            "{name} dealt {centre} from the centre and {edge} from depth, so depth is a \
             number on the HUD rather than something in your hands"
        );
    }
}

#[test]
fn the_curve_is_continuous_and_symmetric() {
    // No thresholds, no snapping between versions, and the two sides of the bar
    // worth exactly the same -- which is what stops one edge being the good one.
    // Read off `state::depth` directly, because what is being asserted is the
    // shape of the curve rather than any one move's use of it.
    let max = sim::tuning::meter_max();
    let power = |at: i32| {
        let mut w = mage();
        w.players[0].mechanic = meter_at(at, Force::Dark);
        sim::state::depth(&w.players[0]).to_f32_for_render()
    };
    assert!(
        power(0) < 1.0,
        "a cast from dead centre is not weaker than an ordinary one, so the middle of the \
         bar costs nothing"
    );
    assert!(
        power(max) > 1.0,
        "a cast from the edge is not stronger than an ordinary one"
    );
    // Out from the centre, one step of the bar at a time. Never falling, never
    // jumping: a tenth of the whole range in a single step would be a threshold
    // wearing a gradient's clothes.
    let span = power(max) - power(0);
    let mut last = power(0);
    for out in 1..=max {
        let now = power(out);
        assert!(now >= last - 0.001, "the curve goes backwards at {out}");
        let jump = now - last;
        assert!(
            jump < span * 0.1,
            "the curve jumps {jump:.3} at {out}, which is a threshold rather than a slope"
        );
        last = now;
    }
    for at in [1, max / 3, max / 2, max - 1, max] {
        assert!(
            (power(at) - power(-at)).abs() < 0.001,
            "the two sides of the bar are not worth the same at {at}"
        );
    }
}

// ---------------------------------------------------------------------------
// The wing opens, and its tip is worth waiting for
// ---------------------------------------------------------------------------

/// How wide the section is on each frame it is a section, in degrees.
///
/// The last active frame is not one: it is the tip, which is a bubble at the
/// end of the blade. See `the_last_frame_is_the_tip_and_it_is_a_bubble`.
fn spans(bits: u16) -> Vec<f32> {
    swept(bits)
        .iter()
        .filter_map(|hb| hb.sector)
        .map(|ring| ring.half_span().to_f32_for_render() * 720.0)
        .collect()
}

#[test]
fn the_wing_opens_from_nothing_to_its_whole_span() {
    // The shape *is* the opening: a wing spreading, rather than a blade
    // sweeping. It starts closed, widens every frame, and reaches the arc the
    // move is tuned for less the share the tip takes.
    let arc = sim::moves::get(Class::DualMage, dual::DARK_AUTO)
        .arc
        .to_f32_for_render()
        * 360.0;
    let wide = spans(L);
    assert!(wide.len() >= 5, "only {} frames of wing", wide.len());
    assert!(
        wide[0] < 1.0,
        "the wing is already {:.0} degrees wide on the frame it appears",
        wide[0]
    );
    for pair in wide.windows(2) {
        assert!(
            pair[1] > pair[0] + 1.0,
            "the wing stopped opening: {:.0} then {:.0} degrees",
            pair[0],
            pair[1]
        );
    }
    // The wing stops short of where it finishes and the tip covers the rest, so
    // what it opens to is the arc less the tip's own share of it.
    let tip = sim::tuning::wing_tip().to_f32_for_render();
    let widest = wide[wide.len() - 1];
    assert!(
        (widest - arc * (1.0 - tip)).abs() < 2.0,
        "the wing opens to {widest:.0} degrees of a {arc:.0} degree arc, \
         leaving {:.0} for the tip",
        arc - widest
    );
}

#[test]
fn the_wing_stops_the_tip_s_own_share_of_the_arc_short_of_the_tip() {
    // The gap is the whole reason the tip is a decision. The blade stops with
    // the last slice of the arc unswept and the bubble arrives at the end of it
    // a frame later, so the ground between the two is covered once, late, and
    // only by the part that hits for `tuning::wing_tipper`.
    let arc = sim::moves::get(Class::DualMage, dual::DARK_AUTO)
        .arc
        .to_f32_for_render();
    let share = sim::tuning::wing_tip().to_f32_for_render();
    let frames = swept(L);
    let ring = frames[frames.len() - 2]
        .sector
        .expect("the frame before the tip is a section");
    let tip = frames[frames.len() - 1].to;
    let bearing = |at: sim::V3| {
        sim::math::atan2_turns(at.z.sub(ring.at.z), at.x.sub(ring.at.x)).to_f32_for_render()
    };
    let stopped = ring.to.to_f32_for_render();
    let gap = sim::Fx::from_raw(((bearing(tip) - stopped) * 65536.0) as i32);
    let gap = sim::math::wrap_turns(gap).to_f32_for_render().abs();
    assert!(
        (gap - arc * share).abs() < 0.01,
        "the wing stops {:.0} degrees short of the tip, of a {:.0} degree arc",
        gap * 360.0,
        arc * 360.0
    );
}

#[test]
fn the_last_frame_is_the_tip_and_it_is_a_bubble() {
    // Everything behind the leading edge has already been swept, so what is
    // live on the final frame is the part that has just arrived -- and it is a
    // **ball at the end of the blade**, not the last slice of the ring.
    //
    // A slice of a ring is metres of arc. The "tip" used to be the widest thing
    // the move ever put in the world, catching the whole front of her at once,
    // which is the opposite of what a tip is for.
    let frames = swept(L);
    let last = frames.len() - 1;
    for (i, hb) in frames.iter().enumerate() {
        assert_eq!(
            hb.tipper,
            i == last,
            "frame {i} of {last} disagrees about being the tip"
        );
        assert_eq!(
            hb.sector.is_none(),
            i == last,
            "frame {i} of {last} disagrees about being a section"
        );
    }
    let tip = frames[last];
    assert!(
        !tip.is_a_beam(),
        "the tip is a line from {:?} to {:?} rather than a point",
        tip.from,
        tip.to
    );
    assert_eq!(
        tip.radius.raw(),
        sim::tuning::wing_tip_radius().raw(),
        "the tip is not the size the tip knob says"
    );
}

#[test]
fn a_body_in_front_of_her_is_caught_by_the_tip_and_by_nothing_else() {
    // Which is what makes the tip landable at all. The wing opens behind it and
    // stops short, so the point out in front of her at full extension is the
    // one place only the tip ever goes -- and standing there is the thing the
    // player is being asked to judge.
    let mut w = mage();
    w.players[0].pos = sim::V3::ZERO;
    w.players[1].pos = sim::V3::new(fx(2.3), Fx::ZERO, Fx::ZERO);
    let mut caught_on = None;
    for frame in 0..30 {
        let health = w.players[1].health;
        step(&mut w, 1, if frame == 0 { L } else { 0 });
        // Read *after* the step: a frame advances the action and then resolves
        // hits against it, so the volume that connected is the one standing
        // when `advance` returns.
        if w.players[1].health < health {
            caught_on = Some(sim::state::hitbox(&w.players[0]).map(|hb| hb.tipper));
        }
    }
    assert_eq!(
        caught_on,
        Some(Some(true)),
        "a body standing in front of her was caught by the body of the wing"
    );
}

#[test]
fn landing_the_tip_hurts_more_than_landing_the_wing() {
    // The one piece of execution in an attack that is otherwise thrown
    // constantly. Both fixtures land the same auto on the same fighter; the
    // only difference is where they were standing when it arrived.
    let damage = |place: sim::V3| {
        let mut w = mage();
        w.players[0].pos = sim::V3::ZERO;
        w.players[1].pos = place;
        let before = w.players[1].health;
        step(&mut w, 1, L);
        step(&mut w, 14, 0);
        before - w.players[1].health
    };
    // Close in on the punching arm's side, which the wing sweeps over early,
    // against the far edge of the reach directly in front, which nothing but
    // the tip gets to. **The tipper is a spacing decision**, and that is the
    // shape of it: the body of the wing is what catches somebody who is already
    // on top of you, and the tip is what catches somebody who thought they were
    // out of range.
    let out = |x: f32, z: f32| sim::V3::new(fx(x), Fx::ZERO, fx(z));
    let side = sim::aim::across(out(1.0, 0.0), Hand::Left);
    let body = damage(side.scale(fx(1.2)));
    let tip = damage(out(2.2, 0.0));
    assert!(body > 0, "the wing did not connect at its side at all");
    assert!(tip > 0, "the tip did not connect in front at all");
    assert!(
        tip > body,
        "the tip dealt {tip} and the body of the wing dealt {body}"
    );
}

fn fx(v: f32) -> Fx {
    Fx::from_raw((v * 65536.0).round() as i32)
}

// ---------------------------------------------------------------------------
// Sweep, and the two Lances
// ---------------------------------------------------------------------------

#[test]
fn sweep_reaches_behind_the_shoulders() {
    // **It is the panic button, and that is a geometric claim rather than a
    // mood.** Sweep is the answer to somebody who has already got inside the
    // punches -- which are long, thin and thrown out in front -- and somebody
    // inside them is standing beside her or past her shoulder. A cut that only
    // covered her front would miss exactly the person it exists for.
    //
    // The one move of hers that does this, which is why it is asserted here and
    // not left to the arc knob: every other volume she puts out is in front.
    let frames = swept(E);
    let mut w = mage();
    step(&mut w, 1, E);
    let behind = frames.iter().filter(|hb| ahead(&w, hb.to) < 0.0).count();
    assert!(
        behind >= 2,
        "the sweep never gets behind her shoulders: {behind} of {} frames",
        frames.len()
    );
    // And it starts behind one shoulder and ends behind the other, rather than
    // reaching back on one side only.
    let first = sideways(&w, Hand::Left, frames[0].to);
    let last = sideways(&w, Hand::Left, frames[frames.len() - 1].to);
    assert!(
        first * last < 0.0,
        "the sweep starts at {first:+.2} m and ends at {last:+.2} m -- both on one side"
    );
}

#[test]
fn sweep_is_one_move_with_two_answers() {
    // Light throws them off their feet; dark takes their legs and pays her for
    // it. Same shape, same frames -- the form is the force she is carrying, and
    // the only thing that changes is what happens to whoever it caught.
    let deep = sim::tuning::meter_deep();
    let cast = |colour: Force| {
        let mut w = mage();
        w.players[0].mechanic = meter_at(-deep * colour.along().abs(), colour);
        w.players[1].pos = w.players[0]
            .pos
            .add(w.players[0].facing.scale(Fx::ratio(6, 5)));
        w.players[0].health = sim::state::max_health() / 2;
        let health = w.players[0].health;
        step(&mut w, 1, E);
        step(&mut w, 18, 0);
        (
            !w.players[1].grounded,
            w.players[1].slowed,
            w.players[0].health - health,
        )
    };
    let (light_air, light_slow, _) = cast(Force::Light);
    let (dark_air, dark_slow, dark_gain) = cast(Force::Dark);
    assert!(light_air, "the light Sweep left them on their feet");
    assert_eq!(light_slow, 0, "the light Sweep slowed them as well");
    assert!(!dark_air, "the dark Sweep threw them off their feet");
    assert!(dark_slow > 0, "the dark Sweep did not slow them");
    assert!(
        dark_gain > 0,
        "the dark Sweep healed her by {dark_gain}, so catching somebody with it is free"
    );
}

#[test]
fn the_light_lance_is_a_thing_you_aim_past_somebody() {
    // The line is the delivery and the burst is the ability, so the damage is
    // at the **far end** of the throw. A player who aims it at somebody gets a
    // poke; one who aims it past them gets the cast.
    let mut near = 0;
    let mut far = 0;
    for (gap, out) in [(Fx::from_int(2), &mut near), (Fx::from_int(7), &mut far)] {
        let mut w = mage();
        w.players[0].mechanic = meter_at(0, Force::Light);
        w.players[1].pos = w.players[0].pos.add(w.players[0].facing.scale(gap));
        let before = w.players[1].health;
        step(&mut w, 1, M);
        assert_eq!(w.players[0].action.attack_kind(), Some(dual::LIGHT_LANCE));
        step(&mut w, 90, 0);
        *out = before - w.players[1].health;
    }
    assert!(near > 0, "the line does not connect at all on the way out");
    assert!(
        far > near * 2,
        "standing on the line took {near} and standing where it bursts took {far} -- \
         the burst is not what the move is for"
    );
}

#[test]
fn the_dark_lance_holds_on_and_the_leash_is_what_ends_it() {
    // Same input, opposite move. It catches the first thing it hits and drains
    // it for as long as the two of them stay close -- which on a fragile melee
    // mage is the cost, not the reward.
    use sim::effects::EffectKind;
    let tethered = |w: &World| {
        w.effects
            .iter()
            .flatten()
            .any(|e| e.kind == EffectKind::Tether)
    };

    // Standing still: it holds until its own clock runs out.
    let mut w = mage();
    w.players[0].mechanic = meter_at(-sim::tuning::meter_deep(), Force::Dark);
    w.players[1].pos = w.players[0]
        .pos
        .add(w.players[0].facing.scale(Fx::from_int(3)));
    let before = w.players[1].health;
    step(&mut w, 1, M);
    assert_eq!(w.players[0].action.attack_kind(), Some(dual::DARK_LANCE));
    let mut stayed = 0;
    for _ in 0..400 {
        step(&mut w, 1, 0);
        stayed += tethered(&w) as u32;
    }
    assert!(stayed > 30, "the tether never took hold");
    assert!(
        w.players[1].health < before,
        "the tether held and drained nothing"
    );

    // Walking out of the leash ends it early. Sideways, because the arena has a
    // platform behind where she spawns.
    let mut w = mage();
    w.players[0].mechanic = meter_at(-sim::tuning::meter_deep(), Force::Dark);
    w.players[1].pos = w.players[0]
        .pos
        .add(w.players[0].facing.scale(Fx::from_int(3)));
    step(&mut w, 1, M);
    let mut ran = 0;
    for _ in 0..400 {
        step(&mut w, 1, Input::D);
        ran += tethered(&w) as u32;
    }
    assert!(
        ran < stayed,
        "walking away held the tether for {ran} frames against {stayed} standing still, \
         so the leash is not what ends it"
    );
    let apart = w.players[1]
        .pos
        .sub(w.players[0].pos)
        .flat_len()
        .to_f32_for_render();
    assert!(
        apart > sim::tuning::tether_leash().to_f32_for_render(),
        "the fixture never got outside the leash"
    );
}

#[test]
fn judgement_leaves_a_field_and_the_bar_decides_how_much_of_one() {
    // The finisher's status has to come from **power** now that the depth gate
    // is gone: at the centre a puddle that is over before anybody walks through
    // it, at the edge the biggest thing in the game. Continuous, like
    // everything else on the class.
    use sim::effects::EffectKind;
    let field = |at: i32| {
        let mut w = mage();
        w.players[0].mechanic = meter_at(at, Force::Light);
        step(&mut w, 1, Q);
        let mut radius = 0.0f32;
        let mut life = 0;
        for _ in 0..400 {
            step(&mut w, 1, 0);
            for e in w.effects.iter().flatten() {
                if e.kind == EffectKind::JudgementField {
                    radius = e.field_radius().to_f32_for_render();
                    life += 1;
                }
            }
        }
        (radius, life)
    };
    let (thin, brief) = field(0);
    let (wide, long) = field(sim::tuning::meter_max());
    assert!(brief > 0, "Judgement leaves nothing behind at the centre");
    assert!(
        wide > thin * 1.5 && long > brief * 3 / 2,
        "from the centre the field is {thin:.1} m for {brief} frames and from the edge \
         {wide:.1} m for {long} -- depth is not what makes the finisher a finisher"
    );
}

#[test]
fn the_finisher_throws_the_bar_harder_than_anything_else_she_has() {
    // The tier the design has been asking for since the bar was built. What
    // makes Judgement the payoff is no longer that it is gated -- it is that
    // casting it deep is a real question about whether you survive the cast.
    let moved = |bits: u16| {
        let mut w = mage();
        step(&mut w, 1, bits);
        step(&mut w, 60, 0);
        meter(&w).abs()
    };
    let auto = moved(L);
    let cast = moved(M);
    let finisher = moved(Q);
    assert!(
        finisher > cast && cast > auto,
        "an auto moves {auto}, a cast {cast} and the finisher {finisher} -- the three \
         tiers are not three"
    );
}
