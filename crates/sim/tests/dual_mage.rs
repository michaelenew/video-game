//! The Dual mage's kit: two arms, two forces, two bars and the hill between.
//!
//! Everything here is a property the class stops working without. She holds two
//! forces apart, one in each arm, and the *only* information the player has
//! about which one they just threw is which arm it came out of and which bar
//! rose. So: the two autos must sweep opposite sides of her, they must be
//! mirror images of each other, and the button pressed has to be the thing
//! that decides. Then the two bars: inside the band nothing moves, outside it
//! the higher rises and the lower falls and she burns, the lower bar gates
//! what her body can do, and both full is wings.
//!
//! See `docs/design/dual-mage.md` for the mechanic and
//! `docs/design/kits/dual-mage.md` for the kit. The measured criteria in
//! `docs/design/plans/dual-mage-v2.md` are the tests from "Goading the bars"
//! down, and `cargo run -p sim --bin goad` prints the same numbers.

use sim::aim::Hand;
use sim::class::{Class, Force, Mechanic};
use sim::dual::Tier;
use sim::moves::dual;
use sim::state::{Action, Hitbox};
use sim::tuning as t;
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

/// A volume the move had out, **and where the body was on the frame it was
/// out**.
///
/// The body used to be irrelevant: she stood still for the whole of an auto, so
/// any frame of the throw answered "where is she" and the tests read it off a
/// world they had advanced once. Both autos carry her now -- the dark one in
/// after the pull, the light one out after the shove, `Move::step` -- so "in
/// front of her" is a question with a different answer every frame, and the
/// only honest one is the frame the volume belongs to.
///
/// Derefs to the hitbox, so `hb.to`, `hb.from` and `hb.sector` read as before.
struct Swing {
    hb: Hitbox,
    body: sim::state::Player,
}

impl std::ops::Deref for Swing {
    type Target = Hitbox;
    fn deref(&self) -> &Hitbox {
        &self.hb
    }
}

/// Throw a move and collect the volume it has out on every active frame.
///
/// **Thrown from the origin, facing down positive x, with the other fighter put
/// out of the way.** A stationary caster made the starting place irrelevant;
/// both autos carry her now, so the volume traces a path through the world
/// rather than sitting in one spot, and a test that wants to stand a body in
/// that path has to be able to ask where the path went.
fn swept(bits: u16) -> Vec<Swing> {
    let mut w = mage();
    w.players[0].pos = sim::V3::ZERO;
    w.players[1].pos = sim::V3::new(Fx::from_int(12), Fx::ZERO, Fx::from_int(12));
    step(&mut w, 1, bits);
    let mut out = Vec::new();
    for _ in 0..90 {
        if let Some(hb) = sim::state::hitbox(&w.players[0]) {
            out.push(Swing {
                hb,
                body: w.players[0],
            });
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
fn sideways(p: &sim::state::Player, hand: Hand, at: sim::V3) -> f32 {
    at.sub(p.pos)
        .dot(sim::aim::across(p.facing, hand))
        .to_f32_for_render()
}

/// How far in front of the body a point is.
fn ahead(p: &sim::state::Player, at: sim::V3) -> f32 {
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
        w.players[0].mechanic = meter_at(0, 0, force);
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
        w.players[0].mechanic = meter_at(0, 0, Force::Dark);
        w.players[0].action = Action::Free;
        step(&mut w, 1, bits);
        assert!(
            w.players[0].action.attack_kind().is_some(),
            "{name} cannot be thrown with both bars empty"
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
        let frames = swept(bits);
        let mid = &frames[frames.len() / 2];
        let side = sideways(&mid.body, hand, mid.to);
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
                ahead(&w.players[0], ring.at),
                sideways(&w.players[0], Hand::Left, ring.at),
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
        let frames = swept(bits);
        let first = ahead(&frames[0].body, frames[0].to);
        assert!(
            first < 0.0,
            "the {name} wing appears {first:.2} m in front of her, not behind"
        );
        let from_the_arm = sideways(&frames[0].body, side, frames[0].to);
        assert!(
            from_the_arm > 0.5,
            "the {name} wing appears {from_the_arm:.2} m along its own arm's side"
        );
        let last = &frames[frames.len() - 1];
        assert!(last.tipper, "the {name} wing does not end on its tip");
        assert!(
            ahead(&last.body, last.to) > reach * 0.8,
            "the {name} wing finishes {:.2} m in front of her, of a {reach:.2} m reach",
            ahead(&last.body, last.to)
        );
        let finish = sideways(&last.body, side, last.to);
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
    // Each wing against the body that threw it, because the two bodies are no
    // longer in the same place: the dark auto steps her in and the light one
    // steps her out, which is the mirror carried through to her feet. Mirrored
    // means the *shapes* agree, and the shape is what the volume does relative
    // to the caster.
    let (dark, light) = (swept(L), swept(R));
    assert_eq!(dark.len(), light.len());
    for (a, b) in dark.iter().zip(light.iter()) {
        assert!(
            (ahead(&a.body, a.to) - ahead(&b.body, b.to)).abs() < 0.01,
            "the two wings are at different distances in front of her"
        );
        assert!(
            (sideways(&a.body, Hand::Left, a.to) + sideways(&b.body, Hand::Left, b.to)).abs()
                < 0.01,
            "the two wings are not mirrored: {:.2} against {:.2}",
            sideways(&a.body, Hand::Left, a.to),
            sideways(&b.body, Hand::Left, b.to)
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
// Goading the bars
// ---------------------------------------------------------------------------

fn dark(w: &World) -> Fx {
    sim::dual::bar(&w.players[0], Force::Dark)
}

fn light(w: &World) -> Fx {
    sim::dual::bar(&w.players[0], Force::Light)
}

/// Both bars together, in whole units -- what one press was worth.
fn goaded(w: &World) -> i32 {
    dark(w).add(light(w)).to_int()
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

fn tier(w: &World) -> Tier {
    sim::dual::tier(&w.players[0])
}

fn meter_at(dark: i32, light: i32, colour: Force) -> Mechanic {
    Mechanic::Meter {
        dark: Fx::from_int(dark),
        light: Fx::from_int(light),
        colour,
        ascending: 0,
        jumped: false,
        landed: 0,
        singe: Fx::ZERO,
    }
}

/// Ascending, with the bars wherever they were.
fn ascended(dark: i32, light: i32, colour: Force) -> Mechanic {
    Mechanic::Meter {
        dark: Fx::from_int(dark),
        light: Fx::from_int(light),
        colour,
        ascending: t::ascension_frames(),
        jumped: false,
        landed: 0,
        singe: Fx::ZERO,
    }
}

/// The top of the bars short of the wings: the highest both can stand, level,
/// without the next frame being ascension. Where "from the edge" is measured.
fn high() -> i32 {
    t::tier_wings() - 1
}

/// A bar value that **holds** a tier through a few frames of calm. A bar set
/// exactly on the threshold is below it on the first frame, which is the rule
/// working -- a tier is held, not reached -- and not what a fixture means.
fn held(tier: Tier) -> i32 {
    tier.threshold() + 5
}

/// The frames a real exchange takes: a Judgement from its first startup frame
/// to the end of its recovery, twice. The unit the plan measures the calm and
/// the climb against.
fn exchange() -> u32 {
    let m = sim::moves::get(Class::DualMage, dual::JUDGEMENT);
    2 * (m.startup as u32 + m.active as u32 + m.recovery as u32)
}

/// Press `bits` on the next frame she is free to act, stepping until then.
fn when_free(w: &mut World, bits: u16) {
    for _ in 0..200 {
        if w.players[0].action.actionable() {
            step(w, 1, bits);
            return;
        }
        step(w, 1, 0);
    }
    panic!("never free to act");
}

#[test]
fn every_attack_goads_a_bar_with_nothing_in_range() {
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
        assert!(
            goaded(&w) > 0,
            "{name} thrown at nothing moved neither bar at all"
        );
    }
}

#[test]
fn an_auto_goads_its_own_bar_by_exactly_one_step() {
    // The bars are calibrated in autos: the thing you throw constantly is the
    // unit everything else is measured against. And an auto raises **its own**
    // bar -- the dark hand feeds the dark being -- whatever she was carrying.
    let one = Fx::from_int(t::meter_auto_push());
    for (bits, force, name) in [(L, Force::Dark, "dark"), (R, Force::Light, "light")] {
        for carrying in [Force::Dark, Force::Light] {
            let mut w = mage();
            w.players[0].mechanic = meter_at(0, 0, carrying);
            step(&mut w, 1, bits);
            let (own, other) = match force {
                Force::Dark => (dark(&w), light(&w)),
                Force::Light => (light(&w), dark(&w)),
            };
            assert_eq!(own.raw(), one.raw(), "the {name} auto's own bar");
            assert_eq!(other.raw(), 0, "the {name} auto moved the other bar");
        }
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
    // Otherwise the first key pressed in a match has no bar to push, which is
    // half of how the bar came to be unmovable.
    let w = mage();
    assert_eq!(colour(&w), Force::Dark);
    assert_eq!(goaded(&w), 0);
    assert_eq!(tier(&w), Tier::None);
}

#[test]
fn a_cast_goads_the_carried_bar_and_further_than_an_auto_does() {
    // Committing to a move is committing harder to a being than poking is. And
    // it feeds the bar of the force the *last auto* left her carrying, not the
    // one the button that threw it faces -- a Lance thrown by a mage carrying
    // the light feeds the light.
    let cast = t::meter_cast_push();
    assert!(
        cast > t::meter_auto_push(),
        "a cast moves her no further than an auto"
    );
    for (bits, name) in [(M, "Lance"), (E, "Sweep")] {
        for carrying in [Force::Dark, Force::Light] {
            let mut w = mage();
            w.players[0].mechanic = meter_at(0, 0, carrying);
            step(&mut w, 1, bits);
            let (own, other) = match carrying {
                Force::Dark => (dark(&w), light(&w)),
                Force::Light => (light(&w), dark(&w)),
            };
            assert_eq!(
                own.to_int(),
                cast,
                "{name} thrown while carrying the {} fed the wrong bar",
                carrying.name()
            );
            assert_eq!(other.raw(), 0);
            assert_eq!(colour(&w), carrying, "a cast changed her force");
        }
    }
}

#[test]
fn steering_cannot_push_a_bar_past_its_top() {
    let mut w = mage();
    for _ in 0..40 {
        w.players[0].action = Action::Free;
        step(&mut w, 1, M);
    }
    assert!(dark(&w).raw() <= Fx::from_int(t::meter_max()).raw());
    assert!(w.players[0].health > 0, "she burned herself to death");
}

// ---------------------------------------------------------------------------
// The hill
// ---------------------------------------------------------------------------

#[test]
fn inside_the_band_nothing_moves_but_the_calm_and_nothing_burns() {
    // The band is the flat top of the hill: a small lead is stable. Both bars
    // fall by exactly the calm and by nothing else, and health is untouched
    // however high both of them are.
    let band = t::meter_band();
    let calm_a_frame = t::meter_calm().mul(sim::DT);
    for (d, l) in [(60, 60), (60, 60 - band), (high(), high() - band), (5, 0)] {
        let mut w = mage();
        w.players[0].mechanic = meter_at(d, l, Force::Dark);
        let health = w.players[0].health;
        let (mut last_d, mut last_l) = (dark(&w), light(&w));
        for frame in 0..120 {
            step(&mut w, 1, 0);
            let (now_d, now_l) = (dark(&w), light(&w));
            let want_d = last_d.sub(calm_a_frame).max(Fx::ZERO);
            let want_l = last_l.sub(calm_a_frame).max(Fx::ZERO);
            assert!(
                (now_d.raw() - want_d.raw()).abs() <= 1 && (now_l.raw() - want_l.raw()).abs() <= 1,
                "from {d}/{l}, frame {frame}: the bars went {last_d:?}/{last_l:?} to \
                 {now_d:?}/{now_l:?}, which is more than the calm"
            );
            assert_eq!(
                w.players[0].health, health,
                "from {d}/{l}, level inside the band, she burned"
            );
            (last_d, last_l) = (now_d, now_l);
        }
    }
}

#[test]
fn outside_the_band_the_higher_bar_rises_and_the_lower_falls_until_it_is_empty() {
    // The hill itself. From a finisher out of level the gap is outside the band
    // and it runs away: every frame the higher bar is higher and the lower is
    // lower, until the lower one is empty. The gap can never come back inside
    // on its own -- that is what the other hand is for.
    //
    // A finisher rather than two casts, because the higher bar's rise is net
    // of the calm: just outside the band the drift is smaller than the calm
    // and the higher bar holds rather than rises, while the gap still widens.
    // A Judgement's push puts her past that.
    let finisher = 50 + t::meter_finisher_push();
    assert!(
        finisher - 50 > t::meter_band(),
        "a finisher does not leave the band"
    );
    for (d, l, name) in [(finisher, 50, "dark ahead"), (50, finisher, "light ahead")] {
        let mut w = mage();
        w.players[0].mechanic = meter_at(d, l, Force::Dark);
        let health = w.players[0].health;
        let (mut last_hi, mut last_lo) = if d > l {
            (dark(&w), light(&w))
        } else {
            (light(&w), dark(&w))
        };
        let mut emptied_at = None;
        for frame in 0..1200 {
            step(&mut w, 1, 0);
            let (hi, lo) = if d > l {
                (dark(&w), light(&w))
            } else {
                (light(&w), dark(&w))
            };
            assert!(
                !sim::dual::level(&w.players[0]),
                "{name}: the gap came back inside the band on its own at frame {frame}"
            );
            if lo.raw() == 0 {
                emptied_at.get_or_insert(frame);
                break;
            }
            assert!(
                hi.raw() > last_hi.raw(),
                "{name}, frame {frame}: the higher bar went {last_hi:?} to {hi:?} -- it did not rise"
            );
            assert!(
                lo.raw() < last_lo.raw(),
                "{name}, frame {frame}: the lower bar went {last_lo:?} to {lo:?} -- it did not fall"
            );
            (last_hi, last_lo) = (hi, lo);
        }
        let emptied = emptied_at.expect("the lower bar never emptied");
        assert!(
            emptied > 30,
            "{name}: the lower bar was gone in {emptied} frames, which is a cliff rather than a hill"
        );
        assert!(
            w.players[0].health < health,
            "{name}: she rode the runaway to the bottom and it cost no health"
        );
    }
}

#[test]
fn the_drift_alone_can_never_fill_a_bar_or_deliver_the_wings() {
    // The higher bar gains exactly what the lower one lost, so with no input
    // the two together can only fall. The only way to the top is to goad both
    // while holding them level, which is the hardest thing the class can do
    // and is meant to be.
    for (d, l) in [(90, 40), (99, 60), (94, 94), (60, 99), (100, 0)] {
        let mut w = mage();
        w.players[0].mechanic = meter_at(d, l, Force::Dark);
        let mut last = dark(&w).add(light(&w));
        for frame in 0..900 {
            step(&mut w, 1, 0);
            let sum = dark(&w).add(light(&w));
            assert!(
                sum.raw() <= last.raw(),
                "from {d}/{l}, frame {frame}: the bars summed to {last:?} and then {sum:?}"
            );
            assert_eq!(
                ascending(&w),
                0,
                "from {d}/{l}, the drift delivered the wings"
            );
            last = sum;
        }
    }
}

#[test]
fn one_cast_from_level_stays_inside_the_band_two_do_not_and_a_finisher_never_does() {
    // The cadence the band's width is chosen against, and the first thing to
    // play. A cast is a lead you can sit on; a second one is the hill.
    let band = t::meter_band();
    let mut w = mage();
    w.players[0].mechanic = meter_at(50, 50, Force::Dark);
    step(&mut w, 1, M);
    assert!(
        sim::dual::level(&w.players[0]),
        "one cast from level left the band"
    );
    when_free(&mut w, E);
    assert!(
        !sim::dual::level(&w.players[0]),
        "two casts from level are still inside the band"
    );

    let mut w = mage();
    w.players[0].mechanic = meter_at(50, 50, Force::Light);
    step(&mut w, 1, Q);
    assert!(
        !sim::dual::level(&w.players[0]),
        "a finisher from level stayed inside the band"
    );

    // And from the band's edge -- the last place it is a question.
    let mut w = mage();
    w.players[0].mechanic = meter_at(50 + band, 50, Force::Dark);
    assert!(sim::dual::level(&w.players[0]));
    step(&mut w, 1, Q);
    assert!(!sim::dual::level(&w.players[0]));
}

#[test]
fn alternating_hands_climbs_without_ever_leaving_the_band() {
    // Dark auto, dark Lance, light auto, light Sweep: the rhythm the tiers are
    // meant to be climbed with. It has to stay inside the band the whole way --
    // if the climb itself started the runaway, the class would be unclimbable
    // -- and it has to actually climb, or the calm has won.
    let mut w = mage();
    let mut presses = 0;
    let mut peak = Tier::None;
    for frame in 0..1200 {
        let bits = if w.players[0].action.actionable() {
            let b = [L, M, R, E][presses % 4];
            presses += 1;
            b
        } else {
            0
        };
        step(&mut w, 1, bits);
        assert!(
            sim::dual::level(&w.players[0]),
            "alternating left the band on frame {frame}: {:?} / {:?}",
            dark(&w),
            light(&w)
        );
        peak = peak.max(tier(&w));
    }
    assert!(
        peak >= Tier::Jump,
        "twenty seconds of alternating reached {peak:?} and never the second tier"
    );
}

#[test]
fn stopping_loses_the_second_tier_and_keeps_the_first_for_a_while() {
    // Benchmark B5. The calm: a tier is something she holds by fighting. Stop
    // goading at three quarters and the second tier is gone almost at once,
    // and the first follows -- but slowly enough that two exchanges of not
    // pressing buttons do not take the blink away, and surely inside seven.
    let mut w = mage();
    w.players[0].mechanic = meter_at(t::tier_jump(), t::tier_jump(), Force::Dark);
    assert_eq!(tier(&w), Tier::Jump);
    step(&mut w, exchange(), 0);
    assert_eq!(
        tier(&w),
        Tier::Blink,
        "one exchange of stillness and the second jump survived, or the blink went too"
    );
    step(&mut w, exchange(), 0);
    assert_eq!(
        tier(&w),
        Tier::Blink,
        "two exchanges of stillness and the blink is gone"
    );
    step(&mut w, 5 * exchange(), 0);
    assert_eq!(
        tier(&w),
        Tier::None,
        "seven exchanges of stillness and she still holds the blink"
    );
}

#[test]
fn a_bar_left_fully_one_sided_burns_most_of_a_health_bar_in_half_a_round_and_never_all_of_it() {
    // Benchmark B7. The burn is the price of ignoring the hill, and it has to
    // be one a player feels: fully one-sided and uncorrected for thirty
    // seconds costs between half and four fifths of a health bar. Over a whole
    // round it still cannot be all of it, and it stops the moment the calm
    // brings the gap back inside the band.
    let mut w = mage();
    w.players[0].mechanic = meter_at(t::meter_max(), 0, Force::Dark);
    let before = w.players[0].health;
    step(&mut w, 60, 0);
    let after_a_second = w.players[0].health;
    assert!(
        after_a_second < before,
        "a full one-sided bar burns nothing"
    );
    step(&mut w, 1800 - 60, 0);
    let half_round = before - w.players[0].health;
    let bar = sim::state::max_health();
    assert!(
        half_round * 2 >= bar && half_round * 5 <= bar * 4,
        "thirty seconds fully one-sided cost {half_round} of {bar}"
    );
    step(&mut w, 1800, 0);
    let lost = before - w.players[0].health;
    assert!(lost < bar, "the burn alone took a health bar ({lost})");
    assert!(sim::dual::level(&w.players[0]));
    let settled = w.players[0].health;
    step(&mut w, 60, 0);
    assert_eq!(
        settled, w.players[0].health,
        "the burn followed her inside the band"
    );
}

#[test]
fn neither_the_burn_nor_ascension_can_be_what_kills_her() {
    // The same rule the Blood mage's costs follow: dying to your own button is
    // not a decision anybody made.
    for mechanic in [
        meter_at(t::meter_max(), 0, Force::Light),
        ascended(t::meter_max(), t::meter_max(), Force::Light),
    ] {
        let mut w = mage();
        w.players[0].mechanic = mechanic;
        w.players[0].health = 2;
        step(&mut w, 600, 0);
        assert_eq!(w.players[0].health, 1);
    }
}

// ---------------------------------------------------------------------------
// Benchmarks -- player actions and their outcomes, 2026-09-23
// ---------------------------------------------------------------------------
//
// The first tuning came back overtuned from play: too much damage, too little
// burn, bars that drained too fast, spells that fed them too much. These pin
// the numbers it was re-tuned to, stated the way the person stated the
// complaint -- as what a player does and what happens -- and
// `cargo run -p sim --bin goad` prints the same numbers. The full list is in
// `docs/design/dual-mage.md` under "Benchmarks".

/// Run a script of presses against a dummy standing where the autos land,
/// healed every frame so the round never ends, and give back what she dealt
/// and what it cost her. The instrument's dummy, in a test.
fn against_a_dummy(frames: u32, mut next: impl FnMut(u32) -> u16) -> (i32, i32) {
    let mut w = mage();
    let reach = sim::moves::get(Class::DualMage, dual::DARK_AUTO).reach;
    let stand = w.players[0]
        .pos
        .add(w.players[0].facing.scale(reach.sub(t::body_radius())));
    w.players[1].pos = stand;
    let before = w.players[0].health;
    let mut dealt = 0;
    let mut presses = 0;
    for _ in 0..frames {
        let bits = if w.players[0].action.actionable() && w.players[0].grounded {
            let b = next(presses);
            presses += 1;
            b
        } else {
            0
        };
        step(&mut w, 1, bits);
        dealt += w.players[1].full_health() - w.players[1].health;
        w.players[1].health = w.players[1].full_health();
        w.players[1].pos = stand;
    }
    (dealt, before - w.players[0].health)
}

#[test]
fn b1_alternating_on_a_dummy_for_half_a_round_deals_about_a_health_bar_and_a_half() {
    // Everything landing on a target that never moves, for thirty seconds,
    // climbing from empty: between one and two and a quarter health bars,
    // the last quarter being the six seconds of ascension the climb reaches
    // near the end, cast at the top of the curve. Against a person a third of
    // it lands, which is one kill a round from the sustained game.
    let (dealt, _) = against_a_dummy(1800, |n| [L, M, R, E][(n % 4) as usize]);
    let bar = sim::state::max_health();
    assert!(
        dealt >= bar && dealt * 4 <= 9 * bar,
        "alternating for half a round dealt {dealt} of a {bar} health bar"
    );
}

#[test]
fn b2_one_sided_spam_on_a_dummy_deals_at_most_three_and_a_half_bars_and_costs_her_half_of_one() {
    // The burst play: one dark auto, then dark casts only, a Judgement every
    // time it is up, on a target that stands in all of it. It is allowed to be
    // the biggest number the class can produce, and it has to cost her.
    let (dealt, cost) = against_a_dummy(1800, |n| {
        if n == 0 {
            L
        } else {
            [M, E, Q][((n - 1) % 3) as usize]
        }
    });
    let bar = sim::state::max_health();
    assert!(
        dealt >= 2 * bar && dealt * 2 <= 7 * bar,
        "one-sided spam for half a round dealt {dealt} of a {bar} health bar"
    );
    assert!(
        cost * 2 >= bar,
        "one-sided spam for half a round cost her only {cost} of {bar}"
    );
}

#[test]
fn b3_a_judgement_from_a_full_bar_is_at_most_a_quarter_of_a_health_bar() {
    // The biggest single strike in the kit, and the one that was too big. A
    // quarter from a full bar; still a real hit -- not under eight per cent --
    // from an empty one. Measured over the strike and its field on somebody
    // standing where it lands.
    let bar = sim::state::max_health();
    let full = dealt_from(Q, high());
    let empty = dealt_from(Q, 0);
    assert!(
        full * 4 <= bar,
        "a Judgement from a full bar took {full} of {bar}, more than a quarter"
    );
    assert!(
        empty * 100 >= bar * 8,
        "a Judgement from an empty bar took {empty} of {bar}, under eight per cent"
    );
}

#[test]
fn b4_the_climb_takes_a_third_of_a_round_to_the_wings() {
    // Clean alternating from empty, in an empty arena: the blink in ten to
    // fourteen seconds, the second jump in sixteen to twenty-one, the wings in
    // twenty to twenty-six. So the wings are once a round, with commitment.
    let mut w = mage();
    w.players[1].pos = V3::new(fx(-12.0), Fx::ZERO, fx(-12.0));
    let mut presses = 0;
    let (mut blink, mut jump, mut wings) = (None, None, None);
    for frame in 1..=1800u32 {
        let bits = if w.players[0].action.actionable() {
            let b = [L, M, R, E][presses % 4];
            presses += 1;
            b
        } else {
            0
        };
        step(&mut w, 1, bits);
        let now = tier(&w);
        if now >= Tier::Blink {
            blink.get_or_insert(frame);
        }
        if now >= Tier::Jump {
            jump.get_or_insert(frame);
        }
        if now == Tier::Wings {
            wings.get_or_insert(frame);
            break;
        }
    }
    let seconds = |f: Option<u32>| f.map(|f| f as f32 / 60.0);
    let (blink, jump, wings) = (seconds(blink), seconds(jump), seconds(wings));
    assert!(
        blink.is_some_and(|s| (10.0..=14.0).contains(&s)),
        "the blink came at {blink:?} s"
    );
    assert!(
        jump.is_some_and(|s| (16.0..=21.0).contains(&s)),
        "the second jump came at {jump:?} s"
    );
    assert!(
        wings.is_some_and(|s| (20.0..=26.0).contains(&s)),
        "the wings came at {wings:?} s"
    );
}

#[test]
fn b6_a_finisher_from_level_is_caught_by_the_other_hand_inside_an_exchange() {
    // The hill, and the answer to it. A Judgement from level leaves the band.
    // Uncorrected, the lower bar is empty in eight to twelve seconds. Answered
    // -- a far-side auto, the far-side cast, and an auto or two more -- it is
    // back inside the band within one exchange of the finisher recovering.
    // Far-side autos alone hold it or claw it back, slowly.
    let mut w = mage();
    w.players[0].mechanic = meter_at(50, 50, Force::Dark);
    step(&mut w, 1, Q);
    assert!(!sim::dual::level(&w.players[0]));
    let mut idle = w.clone();
    let mut emptied = None;
    for frame in 1..=900u32 {
        step(&mut idle, 1, 0);
        if sim::dual::lower(&idle.players[0]).raw() == 0 {
            emptied = Some(frame);
            break;
        }
    }
    assert!(
        emptied.is_some_and(|f| (480..=720).contains(&f)),
        "uncorrected, the lower bar emptied at {emptied:?} frames"
    );

    // Answered with the light hand, from the frame the finisher recovers.
    let mut presses = 0;
    let mut recovered_at = None;
    let mut caught_at = None;
    for frame in 1..=600u32 {
        let free = w.players[0].action.actionable();
        if free && recovered_at.is_none() {
            recovered_at = Some(frame);
        }
        let bits = if free {
            let b = [R, E, R, R, R, R][presses.min(5)];
            presses += 1;
            b
        } else {
            0
        };
        step(&mut w, 1, bits);
        if sim::dual::level(&w.players[0]) {
            caught_at = Some(frame);
            break;
        }
    }
    let recovered = recovered_at.expect("the finisher never recovered");
    let caught = caught_at.expect("the other hand never caught it");
    assert!(
        caught - recovered <= exchange(),
        "caught {} frames after the finisher recovered; an exchange is {}",
        caught - recovered,
        exchange()
    );

    // Autos alone: the gap does not grow.
    let mut w = mage();
    w.players[0].mechanic = meter_at(50, 50, Force::Dark);
    step(&mut w, 1, Q);
    step(&mut w, 60, 0);
    let gap_before = sim::dual::gap(&w.players[0]);
    for _ in 0..300 {
        let bits = if w.players[0].action.actionable() {
            R
        } else {
            0
        };
        step(&mut w, 1, bits);
    }
    assert!(
        sim::dual::gap(&w.players[0]).raw() <= gap_before.raw(),
        "far-side autos alone lost ground: the gap went {gap_before:?} to {:?}",
        sim::dual::gap(&w.players[0])
    );
}

#[test]
fn b7_ignoring_a_runaway_for_ten_seconds_costs_about_a_judgement() {
    // The burn as a consequence: a Judgement from level, and then nothing for
    // ten seconds, costs her between fifteen and twenty-five per cent of a
    // health bar -- roughly what the Judgement did to them.
    let mut w = mage();
    w.players[0].mechanic = meter_at(50, 50, Force::Dark);
    let before = w.players[0].health;
    step(&mut w, 1, Q);
    step(&mut w, 600, 0);
    let lost = before - w.players[0].health;
    let bar = sim::state::max_health();
    assert!(
        lost * 100 >= bar * 15 && lost * 100 <= bar * 25,
        "ten seconds of ignoring the runaway cost {lost} of {bar}"
    );
}

// ---------------------------------------------------------------------------
// The tiers -- what frenzy does to her body
// ---------------------------------------------------------------------------

/// Shift plus forward: the dodge, or the blink.
const DODGE: u16 = Input::SHIFT | Input::W;

/// How far she moved on one frame, flat.
fn moved(before: V3, w: &World) -> f32 {
    w.players[0].pos.sub(before).flat_len().to_f32_for_render()
}

#[test]
fn the_tiers_are_on_the_lower_bar() {
    // A sum can be reached one-sided; the lower bar cannot. Both beings have
    // to be fed, which is what makes the climb a rhythm of alternating hands.
    let blink = t::tier_blink();
    let jump = t::tier_jump();
    assert!(0 < blink && blink < jump && jump < t::tier_wings());
    let held = |d: i32, l: i32| {
        let mut w = mage();
        w.players[0].mechanic = meter_at(d, l, Force::Dark);
        tier(&w)
    };
    assert_eq!(held(blink, blink), Tier::Blink);
    assert_eq!(held(t::meter_max(), blink - 1), Tier::None);
    assert_eq!(held(blink - 1, t::meter_max()), Tier::None);
    assert_eq!(held(jump, jump), Tier::Jump);
    assert_eq!(held(jump, jump - 1), Tier::Blink);
}

#[test]
fn below_the_first_tier_the_dodge_is_a_dodge() {
    let mut w = mage();
    w.players[0].mechanic = meter_at(t::tier_blink() - 1, t::tier_blink() - 1, Force::Dark);
    let before = w.players[0].pos;
    step(&mut w, 1, DODGE);
    assert!(matches!(w.players[0].action, Action::Dodge { .. }));
    let first_frame = moved(before, &w);
    let whole = sim::dual::dodge_travel(true).to_f32_for_render();
    assert!(
        first_frame < whole * 0.2,
        "below the tier she moved {first_frame:.2} m of {whole:.2} m on the first frame"
    );
}

#[test]
fn at_the_first_tier_the_dodge_is_a_blink() {
    // The whole dodge's distance on its first frame, and then the dodge's own
    // invulnerable window and vulnerable tail spent standing there. A blink
    // that kept moving would be a longer dodge; one with no tail would be an
    // escape nobody could punish.
    let mut w = mage();
    w.players[0].mechanic = meter_at(held(Tier::Blink), held(Tier::Blink), Force::Dark);
    let before = w.players[0].pos;
    step(&mut w, 1, DODGE);
    let whole = sim::dual::dodge_travel(true).to_f32_for_render();
    let jumped = moved(before, &w);
    assert!(
        (jumped - whole).abs() < 0.1,
        "at the tier she moved {jumped:.2} m on the first frame and the dodge covers {whole:.2} m"
    );
    assert!(
        w.players[0].action.invulnerable(),
        "the blink has no window"
    );
    let landed = w.players[0].pos;
    step(&mut w, (t::dodge_frames() - t::dodge_iframes()) as u32, 0);
    assert!(
        matches!(w.players[0].action, Action::Dodge { .. }) && !w.players[0].action.invulnerable(),
        "the blink has no vulnerable tail"
    );
    assert!(
        moved(landed, &w) < 0.05,
        "she kept travelling through the tail"
    );
}

#[test]
fn a_blink_carries_the_body_further_than_a_walk_would() {
    // The roster-wide relationship every class is meant to satisfy -- a move
    // that carries the body -- met here by the blink: one frame of it covers
    // more ground than a whole dodge's worth of walking.
    let walk = t::move_speed()
        .mul(sim::DT)
        .mul(Fx::from_int(t::dodge_frames() as i32))
        .to_f32_for_render();
    let mut w = mage();
    w.players[0].mechanic = meter_at(held(Tier::Blink), held(Tier::Blink), Force::Dark);
    let before = w.players[0].pos;
    step(&mut w, 1, DODGE);
    assert!(moved(before, &w) > walk);
}

#[test]
fn a_blink_stops_at_the_arena_rather_than_passing_through_it() {
    // She never passes through terrain: aimed at a platform's side, the blink
    // ends against it. The eastern platform's near face is at x = 5.
    let mut w = mage();
    w.players[0].mechanic = meter_at(held(Tier::Blink), held(Tier::Blink), Force::Dark);
    let face = 5.0f32;
    let whole = sim::dual::dodge_travel(true).to_f32_for_render();
    w.players[0].pos = V3::new(fx(face - whole * 0.5), Fx::ZERO, Fx::ZERO);
    // The other fighter spawns at x = 4, in the way: bodies are not on the
    // line, but two bodies standing in one place shove each other apart, and
    // that shove is not what is being measured.
    w.players[1].pos = V3::new(fx(face), Fx::ZERO, fx(6.0));
    step(&mut w, 1, DODGE);
    let x = w.players[0].pos.x.to_f32_for_render();
    assert!(
        x <= face - t::body_radius().to_f32_for_render() + 0.05,
        "the blink ended at x = {x:.2}, inside the platform whose face is at {face}"
    );
    assert!(
        x > face - whole * 0.5 + 0.2,
        "the blink went nowhere at all: x = {x:.2}"
    );
}

#[test]
fn a_blink_passes_through_a_body_to_the_ground_behind_it() {
    // Bodies are not on the line, as ever: a fighter is a thing standing in a
    // place, and the blink goes to the place.
    let mut w = mage();
    w.players[0].mechanic = meter_at(held(Tier::Blink), held(Tier::Blink), Force::Dark);
    let whole = sim::dual::dodge_travel(true);
    w.players[1].pos = w.players[0]
        .pos
        .add(w.players[0].facing.scale(whole.mul(Fx::ratio(1, 2))));
    let before = w.players[0].pos;
    step(&mut w, 1, DODGE);
    assert!((moved(before, &w) - whole.to_f32_for_render()).abs() < 0.1);
}

#[test]
fn in_the_air_the_blink_is_the_airdodge_and_spends_it() {
    let mut w = mage();
    w.players[0].mechanic = meter_at(held(Tier::Blink), held(Tier::Blink), Force::Dark);
    // A full hop, held: the blink and its tail both have to happen in the air.
    step(&mut w, 10, Input::SPACE);
    assert!(!w.players[0].grounded);
    let before = w.players[0].pos;
    step(&mut w, 1, DODGE);
    let whole = sim::dual::dodge_travel(false).to_f32_for_render();
    assert!((moved(before, &w) - whole).abs() < 0.1);
    assert!(
        w.players[0].air_dodged,
        "the airborne blink did not spend the airdodge"
    );
    step(&mut w, t::air_dodge_frames() as u32, 0);
    assert!(
        !w.players[0].grounded,
        "she landed before the tail ran out, so the fixture measures a walk"
    );
    let again = w.players[0].pos;
    step(&mut w, 1, DODGE);
    assert!(
        moved(again, &w) < 0.05,
        "a second airborne blink was granted"
    );
}

/// Jump, then press space again in the air and give back the change in
/// vertical speed the second press made.
fn second_press(mechanic: Mechanic) -> f32 {
    let mut w = mage();
    w.players[0].mechanic = mechanic;
    step(&mut w, 1, Input::SPACE);
    step(&mut w, 20, 0);
    assert!(!w.players[0].grounded);
    let before = w.players[0].vel.y;
    step(&mut w, 1, Input::SPACE);
    w.players[0].vel.y.sub(before).to_f32_for_render()
}

#[test]
fn space_in_the_air_does_nothing_below_the_second_tier() {
    let blink = held(Tier::Blink);
    let lift = second_press(meter_at(blink, blink, Force::Dark));
    assert!(
        lift <= 0.0,
        "at the first tier a second press of space lifted her by {lift:.2}"
    );
}

#[test]
fn at_the_second_tier_space_in_the_air_jumps_once() {
    // The first class to answer the README's open double-jump row, and once
    // per airtime, because a second one would turn a jump into flight.
    let jump = held(Tier::Jump);
    let lift = second_press(meter_at(jump, jump, Force::Dark));
    assert!(
        lift > 1.0,
        "at the second tier space in the air lifted her by only {lift:.2}"
    );

    let mut w = mage();
    w.players[0].mechanic = meter_at(jump, jump, Force::Dark);
    step(&mut w, 1, Input::SPACE);
    step(&mut w, 20, 0);
    step(&mut w, 1, Input::SPACE);
    step(&mut w, 5, 0);
    let before = w.players[0].vel.y;
    step(&mut w, 1, Input::SPACE);
    assert!(
        w.players[0].vel.y.raw() < before.raw(),
        "a third press of space jumped again in the same airtime"
    );
    // And it is back once her feet are down.
    for _ in 0..300 {
        step(&mut w, 1, 0);
        if w.players[0].grounded {
            break;
        }
    }
    assert!(w.players[0].grounded);
    assert!(
        sim::dual::may_beat_wings(&w.players[0]) || {
            // She may have fallen below the tier while airborne; top it back up
            // before asking.
            w.players[0].mechanic = meter_at(jump, jump, Force::Dark);
            sim::dual::may_beat_wings(&w.players[0])
        }
    );
}

#[test]
fn the_tier_is_lost_the_frame_the_lower_bar_drops_below_it_even_mid_air() {
    let jump = t::tier_jump();
    let mut w = mage();
    w.players[0].mechanic = meter_at(jump + 10, jump + 10, Force::Dark);
    step(&mut w, 1, Input::SPACE);
    step(&mut w, 20, 0);
    // A Judgement's worth of collapse on the light bar, mid-air.
    w.players[0].mechanic = meter_at(jump + 10, jump - 1, Force::Dark);
    assert_eq!(tier(&w), Tier::Blink);
    let before = w.players[0].vel.y;
    step(&mut w, 1, Input::SPACE);
    assert!(
        w.players[0].vel.y.raw() <= before.raw(),
        "the second jump survived losing the tier"
    );
}

#[test]
fn she_falls_slower_at_the_second_tier() {
    let fastest_fall = |mechanic: Mechanic| {
        let mut w = mage();
        w.players[0].mechanic = mechanic;
        w.players[0].pos.y = Fx::from_int(30);
        w.players[0].grounded = false;
        let mut fastest = Fx::ZERO;
        for _ in 0..90 {
            step(&mut w, 1, 0);
            if w.players[0].vel.y.raw() < fastest.raw() {
                fastest = w.players[0].vel.y;
            }
            // The bars calm while she falls; hold the tier for the measurement.
            w.players[0].mechanic = mechanic;
        }
        fastest.to_f32_for_render()
    };
    let jump = held(Tier::Jump);
    let low = fastest_fall(meter_at(
        t::tier_jump() - 1,
        t::tier_jump() - 1,
        Force::Dark,
    ));
    let high_tier = fastest_fall(meter_at(jump, jump, Force::Dark));
    assert!(low < 0.0 && high_tier < 0.0);
    assert!(
        high_tier.abs() < low.abs() * 0.9,
        "she falls at {high_tier:.1} at the tier and {low:.1} below it"
    );
}

// ---------------------------------------------------------------------------
// Ascension
// ---------------------------------------------------------------------------

#[test]
fn goading_both_bars_to_the_top_together_starts_ascension_and_nothing_else_does() {
    // There is no ascend button and there never was: you got there one press
    // at a time, with both hands. One bar at the top is a runaway, not wings.
    let top = t::meter_max();
    let wings = t::tier_wings();
    let mut w = mage();
    w.players[0].mechanic = meter_at(wings + 1, wings - 1, Force::Light);
    step(&mut w, 1, R);
    step(&mut w, 1, 0);
    assert!(
        ascending(&w) > 0,
        "both bars at the top and nothing happened"
    );

    for (d, l) in [(top, wings - 1), (wings - 1, top), (top, 0)] {
        let mut w = mage();
        w.players[0].mechanic = meter_at(d, l, Force::Dark);
        step(&mut w, 30, 0);
        assert_eq!(ascending(&w), 0, "{d}/{l} ascended");
    }
}

#[test]
fn ascension_ends_on_its_own_clock_with_both_bars_empty_and_a_stagger() {
    // The vent. Then she climbs again.
    let mut w = mage();
    w.players[0].mechanic = meter_at(t::meter_max(), t::meter_max(), Force::Light);
    step(&mut w, 1, 0);
    let clock = ascending(&w);
    assert!(clock > 0);
    step(&mut w, clock as u32 - 1, 0);
    assert!(ascending(&w) > 0, "it ended early");
    step(&mut w, 1, 0);
    assert_eq!(ascending(&w), 0, "it did not end");
    assert_eq!(dark(&w).raw(), 0, "it did not empty the dark bar");
    assert_eq!(light(&w).raw(), 0, "it did not empty the light bar");
    assert!(
        matches!(w.players[0].action, Action::Stagger { left } if left <= t::ascension_stun()),
        "she walked out of it as though nothing had happened"
    );
}

#[test]
fn ascension_costs_between_half_and_all_of_the_health_bar() {
    // The design's own number, and the reason it can be a clock at all: **the
    // drain is the timer**. Under half and there is nothing at stake; over the
    // whole bar and it would be a suicide button rather than a gamble.
    let cost = t::ascension_drain() * t::ascension_frames() as i32;
    let bar = t::max_health();
    assert!(
        cost * 2 >= bar && cost <= bar,
        "ascension costs {cost} of a {bar} health bar"
    );
}

#[test]
fn ascension_drains_faster_than_the_widest_gap_burns() {
    // Two different things happen at the top and they have to feel different,
    // or there is no reason to fear the wings over merely running away.
    let mut edge = mage();
    edge.players[0].mechanic = meter_at(t::meter_max(), 0, Force::Light);
    let before = edge.players[0].health;
    step(&mut edge, 60, 0);
    let burned = before - edge.players[0].health;

    let mut gone = mage();
    gone.players[0].mechanic = ascended(t::meter_max(), t::meter_max(), Force::Dark);
    let before = gone.players[0].health;
    step(&mut gone, 60, 0);
    let drained = before - gone.players[0].health;

    assert!(
        drained > burned,
        "ascension took {drained} where the widest gap took {burned}"
    );
}

#[test]
fn nothing_steers_the_bars_while_she_is_ascended() {
    // The bars are not the operative resource then -- the clock is.
    let mut w = mage();
    w.players[0].mechanic = ascended(t::meter_max(), t::meter_max(), Force::Dark);
    step(&mut w, 1, R);
    step(&mut w, 20, 0);
    assert_eq!(dark(&w).to_int(), t::meter_max());
    assert_eq!(light(&w).to_int(), t::meter_max());
}

#[test]
fn while_ascending_the_dodge_is_refused_and_every_press_of_space_is_a_wing_beat() {
    // No dodge: she flies instead. Loss of control is loss of the option to
    // decline.
    let mut w = mage();
    w.players[0].mechanic = ascended(t::meter_max(), t::meter_max(), Force::Dark);
    let before = w.players[0].pos;
    step(&mut w, 1, DODGE);
    assert!(
        !matches!(w.players[0].action, Action::Dodge { .. }),
        "she dodged while ascended"
    );
    // The press falls through to a walk, which is what a refused dodge is --
    // the floating walk, since ascended is off the floor. See `state::floating`.
    let walk = t::move_speed()
        .mul(t::float_move_speed())
        .mul(sim::DT)
        .to_f32_for_render();
    assert!(moved(before, &w) <= walk + 0.01);

    step(&mut w, 1, Input::SPACE);
    step(&mut w, 10, 0);
    for press in 0..4 {
        let before = w.players[0].vel.y;
        step(&mut w, 1, Input::SPACE);
        assert!(
            w.players[0].vel.y.raw() > before.raw(),
            "wing beat {press} was refused"
        );
        step(&mut w, 6, 0);
    }
}

#[test]
fn casts_thrown_while_ascending_read_the_top_of_the_curve() {
    let mut w = mage();
    w.players[0].mechanic = ascended(0, 0, Force::Dark);
    assert_eq!(
        sim::state::depth(&w.players[0]).raw(),
        t::depth_ceiling().raw()
    );
}

/// Ride the whole ascension against somebody standing where the autos land,
/// pressing `bits` every time she is free, and give back the health she ended
/// with and the stagger she came out into.
fn ride(bits: u16) -> (i32, u16) {
    let mut w = mage();
    w.players[0].mechanic = ascended(t::meter_max(), t::meter_max(), Force::Dark);
    let reach = sim::moves::get(Class::DualMage, dual::DARK_AUTO).reach;
    w.players[1].pos = w.players[0]
        .pos
        .add(w.players[0].facing.scale(reach.sub(t::body_radius())));
    while ascending(&w) > 0 {
        let press = if bits != 0 && w.players[0].action.actionable() {
            bits
        } else {
            0
        };
        step(&mut w, 1, press);
        w.players[1].health = w.players[1].full_health();
        // Held where the autos land, so the shove does not walk them out of
        // the measurement.
        w.players[1].pos = w.players[0]
            .pos
            .add(w.players[0].facing.scale(reach.sub(t::body_radius())));
    }
    assert!(
        matches!(w.players[0].action, Action::Stagger { .. }),
        "came out of ascension into {:?}",
        w.players[0].action
    );
    (w.players[0].health, w.players[0].stun_total)
}

#[test]
fn landing_hits_while_ascending_pulls_health_back_and_shortens_the_stagger() {
    // The refund the old design wrote and never built, and the graduated exit
    // it asked for: a near miss is a short stagger, and you are staying alive
    // one connection at a time.
    let (idle_health, idle_stagger) = ride(0);
    let (fighting_health, fighting_stagger) = ride(L);
    assert!(
        fighting_health > idle_health,
        "landing hits refunded nothing: {fighting_health} against {idle_health} idle"
    );
    assert!(
        fighting_stagger < idle_stagger,
        "the stagger was {fighting_stagger}f after landing hits and {idle_stagger}f after none"
    );
    assert_eq!(
        idle_stagger,
        t::ascension_stun(),
        "landing nothing is the ceiling"
    );
    assert!(fighting_stagger >= t::ascension_stun_floor());
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
    w.players[0].mechanic = meter_at(at, at, Force::Dark);
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
        w.players[1].health < w.players[1].full_health(),
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
    for (bits, hand, name) in [(L, Hand::Left, "dark"), (R, Hand::Right, "light")] {
        let centre = gap_change(bits, hand, 0).abs();
        let edge = gap_change(bits, hand, high()).abs();
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
    w.players[0].mechanic = meter_at(at, at, Force::Dark);
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
    for (bits, name) in [
        (L, "dark auto"),
        (R, "light auto"),
        (M, "lance"),
        (E, "sweep"),
        (Q, "judgement"),
    ] {
        let centre = dealt_from(bits, 0);
        let edge = dealt_from(bits, high());
        assert!(centre > 0, "{name} never connected at the centre");
        assert!(
            edge > centre,
            "{name} dealt {centre} from the centre and {edge} from depth, so depth is a \
             number on the HUD rather than something in your hands"
        );
    }
}

#[test]
fn the_curve_is_continuous_and_the_two_bars_are_worth_the_same() {
    // No thresholds, no snapping between versions, and the two bars worth
    // exactly the same -- which is what stops one force being the good one.
    // Read off `state::depth` directly, because what is being asserted is the
    // shape of the curve rather than any one move's use of it.
    let max = t::meter_max();
    let power = |at: i32, carrying: Force| {
        let mut w = mage();
        w.players[0].mechanic = meter_at(at, at, carrying);
        sim::state::depth(&w.players[0]).to_f32_for_render()
    };
    assert!(
        power(0, Force::Dark) < 1.0,
        "a cast from empty is not weaker than an ordinary one, so the bottom of the bar costs \
         nothing"
    );
    assert!(
        power(max, Force::Dark) > 1.0,
        "a cast from a full bar is not stronger than an ordinary one"
    );
    // Up the bar one unit at a time. Never falling, never jumping: a tenth of
    // the whole range in a single step would be a threshold wearing a
    // gradient's clothes.
    let span = power(max, Force::Dark) - power(0, Force::Dark);
    let mut last = power(0, Force::Dark);
    for at in 1..=max {
        let now = power(at, Force::Dark);
        assert!(now >= last - 0.001, "the curve goes backwards at {at}");
        let jump = now - last;
        assert!(
            jump < span * 0.1,
            "the curve jumps {jump:.3} at {at}, which is a threshold rather than a slope"
        );
        last = now;
    }
    for at in [1, max / 3, max / 2, max - 1, max] {
        assert!(
            (power(at, Force::Dark) - power(at, Force::Light)).abs() < 0.001,
            "the two bars are not worth the same at {at}"
        );
    }
}

#[test]
fn a_cast_reads_the_bar_she_carries_and_an_auto_reads_its_own() {
    // Three quantities fall out of two bars and this is the first: the
    // **carried** bar is what a cast is worth. An auto is made of its own
    // force whatever she is carrying, so it reads its own bar -- which is what
    // lets the far-side auto be thrown from strength when the far side is the
    // high one.
    let mut w = mage();
    w.players[0].mechanic = meter_at(high(), 0, Force::Light);
    let p = &w.players[0];
    let light_cast = sim::dual::depth_at(p, Some(dual::SWEEP)).to_f32_for_render();
    let dark_auto = sim::dual::depth_at(p, Some(dual::DARK_AUTO)).to_f32_for_render();
    let light_auto = sim::dual::depth_at(p, Some(dual::LIGHT_AUTO)).to_f32_for_render();
    assert!(
        light_cast < 1.0,
        "carrying an empty light bar, a cast is worth {light_cast:.2}"
    );
    assert!(
        dark_auto > 1.0,
        "the dark auto reads the carried bar rather than its own: {dark_auto:.2}"
    );
    assert!((light_auto - light_cast).abs() < 0.001);
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
    let tip = &frames[last];
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
    // **Stood where the tip actually lands**, read off a dry throw rather than
    // written down as a distance. The dark auto steps her forward now
    // (`Move::step`), so "two and a third metres in front of her" names a
    // different patch of floor on every frame of the throw, and a literal would
    // be a measurement of the step wearing a claim about the tip.
    let landing = {
        let frames = swept(L);
        frames[frames.len() - 1].to
    };
    let mut w = mage();
    w.players[0].pos = sim::V3::ZERO;
    w.players[1].pos = sim::V3::new(landing.x, Fx::ZERO, landing.z);
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
    // Both places read off a dry throw rather than written down, for the reason
    // `a_body_in_front_of_her_is_caught_by_the_tip_and_by_nothing_else` gives:
    // the auto carries her, so the volume travels and a fixed point in the
    // world is not a fixed point on the blade.
    //
    // Early in the sweep, where the blade is still coming round beside the
    // punching arm, against the very last frame, which is the tip. **The tipper
    // is a spacing decision**, and that is the shape of it: the body of the
    // wing is what catches somebody who is already on top of you, and the tip
    // is what catches somebody who thought they were out of range.
    let (early, landing) = {
        let frames = swept(L);
        (frames[1].to, frames[frames.len() - 1].to)
    };
    let flat = |v: sim::V3| sim::V3::new(v.x, Fx::ZERO, v.z);
    let body = damage(flat(early));
    let tip = damage(flat(landing));
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
    let behind = frames
        .iter()
        .filter(|hb| ahead(&hb.body, hb.to) < 0.0)
        .count();
    assert!(
        behind >= 2,
        "the sweep never gets behind her shoulders: {behind} of {} frames",
        frames.len()
    );
    // And it starts behind one shoulder and ends behind the other, rather than
    // reaching back on one side only.
    let first = sideways(&frames[0].body, Hand::Left, frames[0].to);
    let last = sideways(
        &frames[frames.len() - 1].body,
        Hand::Left,
        frames[frames.len() - 1].to,
    );
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
    let cast = |colour: Force| {
        let mut w = mage();
        w.players[0].mechanic = meter_at(high(), high(), colour);
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
        w.players[0].mechanic = meter_at(0, 0, Force::Light);
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
    w.players[0].mechanic = meter_at(high(), high(), Force::Dark);
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
    w.players[0].mechanic = meter_at(high(), high(), Force::Dark);
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
        w.players[0].mechanic = meter_at(at, at, Force::Light);
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
    let (wide, long) = field(high());
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
    // casting it from high is a real question about what the other bar does
    // next.
    let moved = |bits: u16| {
        let mut w = mage();
        step(&mut w, 1, bits);
        goaded(&w)
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

// ---------------------------------------------------------------------------
// Movement: the step, the hang and the float
// ---------------------------------------------------------------------------

/// How far along her own facing a throw leaves her, in metres.
///
/// Stood out past the ends of the arena's two platforms, because she spawns
/// half a metre from the side of one and a step backward walks straight into
/// it -- which is a fact about the blockout rather than about the move.
fn carried_by(bits: u16) -> f32 {
    let mut w = mage();
    w.players[0].pos = sim::V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(8));
    w.players[1].pos = sim::V3::new(Fx::from_int(10), Fx::ZERO, Fx::from_int(10));
    let from = w.players[0].pos;
    let facing = w.players[0].facing;
    step(&mut w, 1, bits);
    step(&mut w, 40, 0);
    w.players[0].pos.sub(from).dot(facing).to_f32_for_render()
}

#[test]
fn the_dash_goes_the_way_the_force_goes() {
    // **One rule, mirrored, which is how the rest of this class is built.** The
    // two autos are the same punch off opposite arms and they already disagree
    // about everything else -- one drags whoever it catches toward her and one
    // shoves them away -- so the step that carries her body is the same
    // disagreement carried down to her feet. Dark closes the gap from both
    // ends; light opens it from both ends.
    //
    // A step forward on *both* was the first version, and it made the light
    // auto's push worth nothing: she walked into the space she had just made,
    // and the net distance between the two bodies went the wrong way. Two
    // mirrored halves of one motion is also simply what the class is.
    let dark = carried_by(L);
    let light = carried_by(R);
    assert!(
        dark > 0.5,
        "the dark auto carried her {dark:.2} m -- it pulls, so it should close"
    );
    assert!(
        light < -0.5,
        "the light auto carried her {light:.2} m -- it pushes, so it should give ground"
    );
}

#[test]
fn her_autos_hang_longer_in_the_air_than_anybody_elses_poke() {
    // Her air game is the one she is supposed to want to be in: floatiest
    // gravity in the cast, and the poke she throws every second suspends her
    // for longer than anything else in the game does. Asserted against the
    // whole roster rather than against a number, so it stays true when
    // somebody tunes a hang somewhere else.
    let hers = sim::moves::get(Class::DualMage, dual::DARK_AUTO).air_stall;
    assert_eq!(
        hers,
        sim::moves::get(Class::DualMage, dual::LIGHT_AUTO).air_stall,
        "the two autos hang for different lengths -- they are one punch mirrored"
    );
    for class in sim::class::ALL_CLASSES {
        for slot in 0..sim::moves::slots(class) {
            let m = sim::moves::get(class, slot as u8);
            let mine = class == Class::DualMage && dual::is_an_auto(slot as u8);
            assert!(
                mine || m.air_stall < hers,
                "{}'s {} hangs for {} frames against her auto's {hers}",
                class.name(),
                m.name,
                m.air_stall
            );
        }
    }
}

#[test]
fn alternating_autos_in_the_air_is_a_slow_fall_and_not_a_hover() {
    // **The reason the hang has a falloff.** A hang costs nothing but the move
    // that carries it, and the repeat lockout only stops one move being thrown
    // twice -- so a class with two interchangeable pokes can alternate them,
    // and she is that class by construction. With a fourteen-frame hang against
    // a twenty-frame auto, an unlimited hang would have meant pressing left,
    // right, left, right and never touching the floor again.
    //
    // Measured against falling with the hands down, which is the thing it is
    // allowed to be better than.
    // **Thrown after the apex.** A hang damps whatever vertical speed she has,
    // so an auto on the way *up* cuts the jump short -- that is the extra
    // control over jump height that attacking has always given, and measuring
    // it here would be measuring the wrong thing. A full hop, then autos
    // alternated from the top.
    let airtime = |attacking: bool| {
        let mut w = mage();
        step(&mut w, 30, Input::SPACE);
        let mut frames = 0;
        let mut button = L;
        for i in 0..600 {
            let bits = if attacking && i % 10 == 0 {
                button = if button == L { R } else { L };
                button
            } else {
                0
            };
            step(&mut w, 1, bits);
            if w.players[0].grounded {
                break;
            }
            frames = i;
        }
        frames
    };
    let plain = airtime(false);
    let floaty = airtime(true);
    assert!(
        floaty > plain,
        "throwing autos on the way down bought no airtime at all: {floaty} against {plain}"
    );
    assert!(
        floaty < plain * 3,
        "alternating autos kept her up for {floaty} frames against {plain} falling -- \
         that is flight, and the falloff is not holding"
    );
}

#[test]
fn at_three_quarters_or_ascended_she_stops_walking_and_moves_faster() {
    // The reward for riding both bars up, and the one thing the tiers move that
    // is not a force. The float lives on the same tier as the second jump and
    // the slow fall, so "her feet leave the floor" means one thing -- see
    // `state::floating`.
    let crossed = |mechanic| {
        let mut w = mage();
        w.players[0].mechanic = mechanic;
        let from = w.players[0].pos;
        for _ in 0..40 {
            w.advance([Input::aimed(Input::W, LOOK), Input::new(0)]);
        }
        w.players[0].pos.sub(from).flat_len().to_f32_for_render()
    };
    let centre = crossed(meter_at(0, 0, Force::Dark));
    let blink = crossed(meter_at(held(Tier::Blink), held(Tier::Blink), Force::Dark));
    let jump = crossed(meter_at(held(Tier::Jump), held(Tier::Jump), Force::Dark));
    let ascending = crossed(ascended(high(), high(), Force::Light));
    assert!(
        (blink - centre).abs() < centre * 0.05,
        "at the blink tier she covered {blink:.1} m against {centre:.1} m at the centre -- \
         the float fired a tier early"
    );
    assert!(
        jump > centre * 1.1,
        "at three quarters she covered {jump:.1} m against {centre:.1} m at the centre"
    );
    assert!(
        ascending > centre * 1.1,
        "ascended she covered {ascending:.1} m against {centre:.1} m at the centre"
    );
    assert!(
        !sim::state::floating(&mage().players[0]),
        "she is floating with both bars empty"
    );
}

#[test]
fn nobody_else_floats() {
    // `state::floating` is asked of whoever is standing there, so it has to have
    // an answer for the five classes with no meter -- and the answer is no.
    for class in sim::class::ALL_CLASSES {
        if class == Class::DualMage {
            continue;
        }
        let w = World::with_classes([class, Class::Bulwark]);
        assert!(
            !sim::state::floating(&w.players[0]),
            "the {} floats, and has no bar to do it from",
            class.name()
        );
    }
}
