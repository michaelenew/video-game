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
        (L | SHIFT, "lance"),
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
                sideways(&w, ring.at),
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
        let out = Fx::from_int(side.outward()).to_f32_for_render();
        let first = ahead(&w, frames[0].to);
        assert!(
            first < 0.0,
            "the {name} wing appears {first:.2} m in front of her, not behind"
        );
        let from_the_arm = sideways(&w, frames[0].to) * out;
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
        let finish = sideways(&w, last.to) * out;
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
    // Three, and two of them are the same move mirrored. The Dual mage's autos
    // are sided because the side *is* the mechanic -- left is dark, right is
    // light, and a hitbox on the centre line cannot say which one landed. The
    // Champion's spear opener is sided because it is the one attack in that class
    // thrown with one arm: a jab off the leading hand with the butt of the shaft
    // still at the hip, which is most of why it reads as a poke rather than as a
    // short lunge.
    let declared: &[(Class, u8)] = &[
        (Class::DualMage, dual::DARK_AUTO),
        (Class::DualMage, dual::LIGHT_AUTO),
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
        (L | SHIFT, "lance"),
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
    for (force, bits, name) in [
        (Force::Dark, L | SHIFT, "Lance"),
        (Force::Light, E, "Sweep"),
    ] {
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
        step(&mut w, 1, L | SHIFT);
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
