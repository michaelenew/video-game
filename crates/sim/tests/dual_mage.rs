//! The Dual mage's kit: two hands, two forces, two bars and the hill between.
//!
//! Everything here is a property the class stops working without. She holds two
//! forces apart, one in each hand, and since the 2026-10-09 rework every move
//! she has is a spell made of one of them -- left is dark, right is light,
//! middle is twilight, both at once. The button pressed has to be the thing
//! that decides which bar rises. Then the two bars: inside the band nothing
//! moves, outside it the higher rises and the lower falls and she burns, the
//! lower bar gates what her body can do, and both full is wings.
//!
//! See `docs/design/dual-mage.md` for the mechanic and
//! `docs/design/exploration/0010_dual_mage_spells.md` for the moves; what each
//! spell does on its own is `tests/dual_spells.rs`. The measured criteria in
//! `docs/design/plans/dual-mage-v2.md` are the tests from "Goading the bars"
//! down, and `cargo run -p sim --bin goad` prints the same numbers.

use sim::aim::Hand;
use sim::class::{Class, Force, Mechanic};
use sim::dual::Tier;
use sim::effects::EffectKind;
use sim::moves::dual;
use sim::moves::dual::keys;
use sim::state::Action;
use sim::tuning as t;
use sim::{Fx, Input, V3, World};

/// Left click: dark.
const L: u16 = keys::DARK;
/// Right click: light.
const R: u16 = keys::LIGHT;
/// `Q`: Abyss, the dark major.
const Q: u16 = keys::DARK_MAJOR;
/// `E`: Judgement, the light major.
const E: u16 = keys::LIGHT_MAJOR;
/// Middle click: twilight, both at once. Shift is a dodge and nothing else --
/// see `docs/design/controls.md`.
const M: u16 = keys::TWILIGHT;
const SPACE: u16 = Input::SPACE;

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

/// One frame, with the crosshair at `pitch`.
fn step_looking(w: &mut World, bits: u16, pitch: i16) {
    w.advance([Input::looking_at(bits, LOOK, pitch), Input::new(0)]);
}

/// The pitch that puts her crosshair on the other fighter's feet.
///
/// Every spell of hers is aimed through the crosshair now, and the feet are
/// the one point that serves all of them: a skillshot whose ray meets the
/// floor goes to the middle of whoever stands there (`aim::skillshot_path`),
/// and a major lands where the crosshair meets the floor.
fn onto_them(w: &World) -> i16 {
    let them = w.players[1].pos;
    sim::aim::look_onto_closely(w.players[0].pos, LOOK, w.players[0].aloft, them)
}

/// The move a button throws from the floor.
fn floor_kind(bits: u16) -> u8 {
    match bits {
        b if b == L => dual::SHADE_BOLT,
        b if b == R => dual::SUNRAY,
        b if b == M => dual::BINARY,
        b if b == Q => dual::ABYSS,
        b if b == E => dual::JUDGEMENT,
        b if b == SPACE | L => dual::NIGHTFALL,
        b if b == SPACE | R => dual::DAWN,
        b if b == SPACE | M => dual::EQUINOX,
        _ => panic!("no floor move on {bits:b}"),
    }
}

/// Free, on the floor, and the move `bits` throws is not locked out: a press
/// now is that move. A script that pressed into the repeat lockout would be
/// counting presses that threw nothing.
fn ready(w: &World, bits: u16) -> bool {
    let p = &w.players[0];
    p.action.actionable() && p.grounded && !p.locked_out(floor_kind(bits))
}

/// Up in the open, high enough that a click is the air's move for a while.
fn aloft(w: &mut World, height: i32) {
    w.players[0].pos.y = Fx::from_int(height);
    w.players[0].grounded = false;
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
    assert_eq!(kind_thrown(L), dual::SHADE_BOLT);
    assert_eq!(kind_thrown(R), dual::SUNRAY);
}

#[test]
fn middle_click_throws_binary_whatever_she_carries() {
    // Middle click has no side, so it cannot pick a direction on the bar --
    // which is what makes it the home of twilight, the one spell made of both.
    // It used to be the Lance, whose form was the force she carried; since the
    // rework it is one move, and the force she carries is left as it was.
    for force in [Force::Light, Force::Dark] {
        let mut w = mage();
        w.players[0].mechanic = meter_at(0, 0, force);
        step(&mut w, 1, M);
        assert_eq!(
            w.players[0].action.attack_kind(),
            Some(dual::BINARY),
            "carrying the {}, middle click threw something else",
            force.name()
        );
        assert_eq!(colour(&w), force, "twilight changed the force she carries");
    }
}

#[test]
fn shift_is_only_a_dodge_now() {
    // The grammar change, checked from the class it cost a binding: shift plus
    // a click is not an attack input any more, on any class, so shift plus left
    // click is the bare left click's move. See `docs/design/controls.md`.
    assert_eq!(kind_thrown(L | Input::SHIFT), dual::SHADE_BOLT);
    assert_eq!(kind_thrown(R | Input::SHIFT), dual::SUNRAY);
}

#[test]
fn q_and_e_both_throw_something() {
    // Two keys, two majors: `Q` the dark one, `E` the light one.
    let mut w = mage();
    step(&mut w, 1, Q);
    assert_eq!(w.players[0].action.attack_kind(), Some(dual::ABYSS));

    let mut w = mage();
    step(&mut w, 1, E);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(dual::JUDGEMENT),
        "`E` does nothing, so the light major is unreachable"
    );
}

#[test]
fn nothing_in_the_kit_is_locked_at_the_centre() {
    // Judgement used to need the bar deep on one side before it would come out,
    // which meant its key did nothing at all for the opening of every match --
    // the player pressing it had no way to tell an ability from an empty key.
    // A major you cannot press is not one.
    let mut w = mage();
    for (bits, name) in [
        (L, "shade bolt"),
        (R, "sunray"),
        (E, "judgement"),
        (Q, "abyss"),
        (M, "binary"),
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
// Two hands
// ---------------------------------------------------------------------------

#[test]
fn the_two_autos_come_out_of_opposite_hands() {
    // A player reads which force they just threw off which hand threw it, so
    // the table has to say so: the dark auto off the left, the light off the
    // right. (Where a skillshot actually leaves the body is
    // `aim::skillshot_path`'s question, and today that is the chest for both.)
    assert_eq!(
        sim::moves::get(Class::DualMage, dual::SHADE_BOLT).hand,
        Hand::Left
    );
    assert_eq!(
        sim::moves::get(Class::DualMage, dual::SUNRAY).hand,
        Hand::Right
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
    // Nine, and eight of them are the Dual mage. Since the 2026-10-09 rework
    // every move of hers but the twilight column is made of one force, and the
    // hand it leaves from is that force: left dark, right light. A spell off
    // the centre line could not say which one it was. The Champion's spear
    // opener is sided because it is the one attack in that class thrown with
    // one arm: a jab off the leading hand with the butt of the shaft still at
    // the hip, which is most of why it reads as a poke rather than as a short
    // lunge.
    let declared: &[(Class, u8)] = &[
        (Class::DualMage, dual::SHADE_BOLT),
        (Class::DualMage, dual::REEL),
        (Class::DualMage, dual::NIGHTFALL),
        (Class::DualMage, dual::ABYSS),
        (Class::DualMage, dual::SUNRAY),
        (Class::DualMage, dual::FLARE),
        (Class::DualMage, dual::DAWN),
        (Class::DualMage, dual::JUDGEMENT),
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

#[test]
fn every_attack_goads_a_bar_with_nothing_in_range() {
    // **The bug this class shipped with.** The autos only steered on contact
    // and the casts took their direction from an auto that had never landed, so
    // a player standing where they spawn -- eight metres from anybody -- could
    // press every button on the class and watch the bar sit at zero. A resource
    // you cannot move without a target is one nobody can learn or tune.
    for (bits, name) in [
        (L, "shade bolt"),
        (R, "sunray"),
        (M, "binary"),
        (Q, "abyss"),
        (E, "judgement"),
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
fn the_last_force_she_threw_is_the_force_she_is_carrying_and_twilight_keeps_it() {
    // Every move has a force now, so the one she carries -- the HUD border,
    // and what a question between moves reads -- is whichever she last threw.
    // Twilight is both, so it leaves the carried force where it was.
    let mut w = mage();
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(-12));
    for (bits, want) in [
        (R, Force::Light),
        (M, Force::Light),
        (L, Force::Dark),
        (M, Force::Dark),
        (E, Force::Light),
        (Q, Force::Dark),
    ] {
        for _ in 0..300 {
            if ready(&w, bits) {
                break;
            }
            step(&mut w, 1, 0);
        }
        step(&mut w, 1, bits);
        assert_eq!(
            w.players[0].action.attack_kind(),
            Some(floor_kind(bits)),
            "fixture: {bits:b} did not come out"
        );
        assert_eq!(colour(&w), want, "after {bits:b}");
        // Back down to level, so the runaway never decides anything here.
        w.players[0].mechanic = meter_at(0, 0, want);
    }
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

/// Throw the cast `bits` in the air or as a takeoff from the floor, with both
/// bars empty and `carrying` in her hands, and give back the world a frame
/// later.
fn cast_from_empty(bits: u16, in_the_air: bool, carrying: Force) -> World {
    let mut w = mage();
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(-12));
    w.players[0].mechanic = meter_at(0, 0, carrying);
    if in_the_air {
        aloft(&mut w, 6);
    }
    step(&mut w, 1, bits);
    w
}

#[test]
fn a_cast_goads_its_own_bar_and_further_than_an_auto_does() {
    // Committing to a spell is committing harder to a being than poking is.
    // And since the rework a cast is made of its own force: it feeds that
    // bar, whatever she was carrying, and leaves her carrying it. (It used to
    // feed the carried bar, when only the autos had a side.)
    let cast = t::meter_cast_push();
    assert!(
        cast > t::meter_auto_push(),
        "a cast moves her no further than an auto"
    );
    for (bits, air, kind, force) in [
        (L, true, dual::REEL, Force::Dark),
        (R, true, dual::FLARE, Force::Light),
        (SPACE | L, false, dual::NIGHTFALL, Force::Dark),
        (SPACE | R, false, dual::DAWN, Force::Light),
    ] {
        for carrying in [Force::Dark, Force::Light] {
            let w = cast_from_empty(bits, air, carrying);
            let name = sim::moves::get(Class::DualMage, kind).name;
            assert_eq!(
                w.players[0].action.attack_kind(),
                Some(kind),
                "fixture: {name} did not come out"
            );
            let (own, other) = match force {
                Force::Dark => (dark(&w), light(&w)),
                Force::Light => (light(&w), dark(&w)),
            };
            assert_eq!(
                own.to_int(),
                cast,
                "{name} thrown while carrying the {} fed its own bar by the wrong amount",
                carrying.name()
            );
            assert_eq!(other.raw(), 0, "{name} fed the other bar");
            assert_eq!(
                colour(&w),
                force,
                "{name} left her carrying the wrong force"
            );
        }
    }
}

#[test]
fn twilight_goads_both_bars_by_an_auto_and_keeps_the_force_she_carries() {
    // The middle column is both forces at once: each bar by an auto's push,
    // so it never widens the gap.
    let one = Fx::from_int(t::meter_auto_push());
    for (bits, air, kind) in [
        (M, false, dual::BINARY),
        (M, true, dual::PHASE),
        (SPACE | M, false, dual::EQUINOX),
    ] {
        for carrying in [Force::Dark, Force::Light] {
            let w = cast_from_empty(bits, air, carrying);
            let name = sim::moves::get(Class::DualMage, kind).name;
            assert_eq!(
                w.players[0].action.attack_kind(),
                Some(kind),
                "fixture: {name} did not come out"
            );
            assert_eq!(dark(&w).raw(), one.raw(), "{name}'s dark bar");
            assert_eq!(light(&w).raw(), one.raw(), "{name}'s light bar");
            assert_eq!(colour(&w), carrying, "{name} changed her force");
        }
    }
}

#[test]
fn steering_cannot_push_a_bar_past_its_top() {
    let mut w = mage();
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(-12));
    for _ in 0..40 {
        w.players[0].action = Action::Free;
        w.players[0].repeat_lock.fill(0);
        step(&mut w, 1, Q);
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
    // play. A cast is a lead you can sit on; a second one is the hill. Two
    // Reels in the air, high enough that both are thrown before she lands.
    let band = t::meter_band();
    let mut w = mage();
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(-12));
    w.players[0].mechanic = meter_at(50, 50, Force::Dark);
    aloft(&mut w, 20);
    step(&mut w, 1, L);
    assert_eq!(w.players[0].action.attack_kind(), Some(dual::REEL));
    assert!(
        sim::dual::level(&w.players[0]),
        "one cast from level left the band"
    );
    for _ in 0..200 {
        let p = &w.players[0];
        if p.action.actionable() && !p.locked_out(dual::REEL) {
            break;
        }
        step(&mut w, 1, 0);
    }
    assert!(
        !w.players[0].grounded,
        "fixture: she landed between the casts"
    );
    step(&mut w, 1, L);
    assert_eq!(w.players[0].action.attack_kind(), Some(dual::REEL));
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

/// The rhythm the tiers are climbed with: dark, light, dark, light -- the
/// bread and butter of the design (0010 §"The six combos"), the Shade bolt
/// and the Sunray in turn.
const CLIMB: [u16; 2] = [L, R];

#[test]
fn alternating_hands_climbs_without_ever_leaving_the_band() {
    // It has to stay inside the band the whole way -- if the climb itself
    // started the runaway, the class would be unclimbable -- and it has to
    // actually climb, or the calm has won.
    let mut w = mage();
    let mut presses = 0;
    let mut peak = Tier::None;
    for frame in 0..1200 {
        let next = CLIMB[presses % CLIMB.len()];
        let bits = if ready(&w, next) {
            presses += 1;
            next
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
//
// The scripts were rewritten for the 2026-10-09 spell rework: alternating is
// the Shade bolt and the Sunray, one-sided is one hand, and the finisher is
// the carried force's major.

/// How far in front of her the dummy stands: inside both autos' reach (the
/// Sunray's seven metres is the shorter) and Binary's.
fn dummy_gap() -> Fx {
    Fx::from_int(5)
}

/// Run a script of presses against a dummy standing in range, the crosshair
/// on them, healed every frame so the round never ends, and give back what
/// she dealt and what it cost her. The instrument's dummy, in a test.
///
/// A press is made only when the move it throws would come out, so a script
/// counts moves rather than presses into the repeat lockout.
fn against_a_dummy(frames: u32, mut next: impl FnMut(u32) -> u16) -> (i32, i32) {
    let mut w = mage();
    let stand = w.players[0].pos.add(w.players[0].facing.scale(dummy_gap()));
    w.players[1].pos = stand;
    let before = w.players[0].health;
    let mut dealt = 0;
    let mut presses = 0;
    let mut wanted = next(presses);
    for _ in 0..frames {
        let bits = if ready(&w, wanted) {
            let b = wanted;
            presses += 1;
            wanted = next(presses);
            b
        } else {
            0
        };
        let pitch = onto_them(&w);
        step_looking(&mut w, bits, pitch);
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
    let (dealt, _) = against_a_dummy(1800, |n| [L, R][(n % 2) as usize]);
    let bar = sim::state::max_health();
    assert!(
        dealt >= bar && dealt * 4 <= 9 * bar,
        "alternating for half a round dealt {dealt} of a {bar} health bar"
    );
}

#[test]
fn b2_one_sided_spam_on_a_dummy_deals_at_most_three_and_a_half_bars_and_costs_her_half_of_one() {
    // The burst play: the dark hand only, an Abyss every time it is up, on a
    // target that stands in all of it. It is allowed to be the biggest number
    // the class can produce, and it has to cost her.
    let (dealt, cost) = against_a_dummy(1800, |n| [L, L, Q][(n % 3) as usize]);
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
    let full = dealt_from(E, high());
    let empty = dealt_from(E, 0);
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
        let next = CLIMB[presses % CLIMB.len()];
        let bits = if ready(&w, next) {
            presses += 1;
            next
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
    // The hill, and the answer to it. An Abyss from level leaves the band.
    // Uncorrected, the lower bar is empty in eight to twelve seconds. Answered
    // -- a far-side auto, the far-side cast, and an auto or two more -- it is
    // back inside the band within one exchange of the finisher recovering.
    // Far-side autos alone hold it or claw it back, slowly.
    let mut w = mage();
    w.players[1].pos = V3::new(fx(-12.0), Fx::ZERO, fx(-12.0));
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

    // Answered with the light hand, from the frame the finisher recovers: a
    // Sunray, a Dawn (the light cast on the floor), then Sunrays.
    let answer = [R, SPACE | R, R, R, R, R];
    let mut presses = 0;
    let mut recovered_at = None;
    let mut caught_at = None;
    for frame in 1..=600u32 {
        let free = w.players[0].action.actionable() && w.players[0].grounded;
        if free && recovered_at.is_none() {
            recovered_at = Some(frame);
        }
        let next = answer[presses.min(answer.len() - 1)];
        let bits = if ready(&w, next) {
            presses += 1;
            next
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
    w.players[1].pos = V3::new(fx(-12.0), Fx::ZERO, fx(-12.0));
    w.players[0].mechanic = meter_at(50, 50, Force::Dark);
    step(&mut w, 1, Q);
    step(&mut w, 60, 0);
    let gap_before = sim::dual::gap(&w.players[0]);
    for _ in 0..300 {
        let bits = if ready(&w, R) { R } else { 0 };
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
    // The burn as a consequence: a major from level, and then nothing for
    // ten seconds, costs her between fifteen and twenty-five per cent of a
    // health bar -- roughly what a Judgement does to them. Nobody is near the
    // well, so the Abyss drains nothing back to her.
    let mut w = mage();
    w.players[1].pos = V3::new(fx(-12.0), Fx::ZERO, fx(-12.0));
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

/// Ride the whole ascension against somebody standing in range of the autos,
/// the crosshair on them, pressing `bits` every time she is free, and give
/// back the health she ended with and the stagger she came out into.
fn ride(bits: u16) -> (i32, u16) {
    let mut w = mage();
    w.players[0].mechanic = ascended(t::meter_max(), t::meter_max(), Force::Dark);
    let place = |w: &World| {
        let at = w.players[0].pos.add(w.players[0].facing.scale(dummy_gap()));
        V3::new(at.x, Fx::ZERO, at.z)
    };
    w.players[1].pos = place(&w);
    while ascending(&w) > 0 {
        let press = if bits != 0 && w.players[0].action.actionable() {
            bits
        } else {
            0
        };
        let pitch = onto_them(&w);
        step_looking(&mut w, press, pitch);
        w.players[1].health = w.players[1].full_health();
        // Held in range, so nothing walks them out of the measurement.
        w.players[1].pos = place(&w);
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
    //
    // Through Judgement, whose strike is a hitbox. See the next test for
    // the spells that are effects.
    let (idle_health, idle_stagger) = ride(0);
    let (fighting_health, fighting_stagger) = ride(E);
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

#[test]
fn landing_spells_while_ascending_counts_as_landing_hits() {
    // "Landing anything while ascended pulls some health back." Her autos are
    // what she throws every second, and they are spells in flight now.
    let (idle_health, idle_stagger) = ride(0);
    for (bits, name) in [(L, "shade bolt"), (R, "sunray")] {
        let (health, stagger) = ride(bits);
        assert!(
            health > idle_health,
            "landing the {name} refunded nothing: {health} against {idle_health} idle"
        );
        assert!(
            stagger < idle_stagger,
            "the stagger was {stagger}f after landing the {name} and {idle_stagger}f after none"
        );
    }
}

// ---------------------------------------------------------------------------
// The numbers the kit depends on
// ---------------------------------------------------------------------------

/// What one of her moves takes off somebody standing in range with the
/// crosshair on them, thrown from `at` on both bars.
fn dealt_from(bits: u16, at: i32) -> i32 {
    let mut w = mage();
    w.players[0].mechanic = meter_at(at, at, Force::Dark);
    w.players[1].pos = w.players[0].pos.add(w.players[0].facing.scale(dummy_gap()));
    let before = w.players[1].health;
    let pitch = onto_them(&w);
    step_looking(&mut w, bits, pitch);
    for _ in 0..150 {
        let pitch = onto_them(&w);
        step_looking(&mut w, 0, pitch);
    }
    before - w.players[1].health
}

#[test]
fn a_cast_at_the_centre_is_a_weaker_cast_than_one_at_the_edge() {
    // **The largest hole the class shipped with, closed.** Every move she has
    // on the floor, not one at a time: at the centre everything she throws is
    // thin, and at the edge it is the most she can hold. Measured end to end --
    // health actually taken off somebody -- rather than off the multiplier,
    // because the point is that the curve reaches all the way through to the
    // thing that hurts.
    for (bits, name) in [
        (L, "shade bolt"),
        (R, "sunray"),
        (M, "binary"),
        (Q, "abyss"),
        (E, "judgement"),
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
fn a_move_reads_its_own_bar_and_twilight_reads_the_lower() {
    // Three quantities fall out of two bars. Every move is made of its own
    // force, so it reads its own bar -- which is what lets the far-side hand
    // be thrown from strength when the far side is the high one. A twilight
    // move is both, and is worth the **lower** bar: the balance is what it
    // spends. A question between moves reads the force she is carrying.
    let mut w = mage();
    w.players[0].mechanic = meter_at(high(), 0, Force::Light);
    let p = &w.players[0];
    let at = |kind: Option<u8>| sim::dual::depth_at(p, kind).to_f32_for_render();
    for (kind, strong) in [
        (dual::SHADE_BOLT, true),
        (dual::REEL, true),
        (dual::NIGHTFALL, true),
        (dual::ABYSS, true),
        (dual::SUNRAY, false),
        (dual::FLARE, false),
        (dual::DAWN, false),
        (dual::JUDGEMENT, false),
        (dual::BINARY, false),
        (dual::PHASE, false),
        (dual::EQUINOX, false),
    ] {
        let name = sim::moves::get(Class::DualMage, kind).name;
        let power = at(Some(kind));
        assert_eq!(
            power > 1.0,
            strong,
            "dark full and light empty, {name} is worth {power:.2}"
        );
    }
    assert!(
        (at(Some(dual::SUNRAY)) - at(Some(dual::JUDGEMENT))).abs() < 0.001,
        "two light moves read different bars"
    );
    assert!(
        (at(None) - at(Some(dual::SUNRAY))).abs() < 0.001,
        "carrying the light, a question between moves did not read the light bar"
    );
}

fn fx(v: f32) -> Fx {
    Fx::from_raw((v * 65536.0).round() as i32)
}

// ---------------------------------------------------------------------------
// The majors
// ---------------------------------------------------------------------------

#[test]
fn judgement_leaves_a_field_and_the_bar_decides_how_much_of_one() {
    // The finisher's status has to come from **power** now that the depth gate
    // is gone: at the centre a puddle that is over before anybody walks through
    // it, at the edge the biggest thing in the game. Continuous, like
    // everything else on the class.
    let field = |at: i32| {
        let mut w = mage();
        w.players[0].mechanic = meter_at(at, at, Force::Light);
        step(&mut w, 1, E);
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
    // makes a major the payoff is no longer that it is gated -- it is that
    // casting it from high is a real question about what the other bar does
    // next. A cast here is Nightfall, the dark one on the floor.
    let moved = |bits: u16| {
        let mut w = mage();
        step(&mut w, 1, bits);
        goaded(&w)
    };
    let auto = moved(L);
    let cast = moved(SPACE | L);
    let finisher = moved(Q);
    assert!(
        finisher > cast && cast > auto,
        "an auto moves {auto}, a cast {cast} and the finisher {finisher} -- the three \
         tiers are not three"
    );
}

// ---------------------------------------------------------------------------
// Movement: the hang and the float
// ---------------------------------------------------------------------------

#[test]
fn alternating_clicks_in_the_air_is_a_slow_fall_and_not_a_hover() {
    // **The reason the hang has a falloff.** A hang costs nothing but the move
    // that carries it, and the repeat lockout only stops one move being thrown
    // twice -- so a class with two interchangeable air clicks can alternate
    // them, and she is that class by construction. An unlimited hang would
    // mean pressing left, right, left, right and never touching the floor
    // again. In the air her clicks are Reel and Flare, thrown here at nothing.
    //
    // Measured against falling with the hands down, which is the thing it is
    // allowed to be better than, and thrown after the apex: a hang damps
    // whatever vertical speed she has, so a spell on the way *up* cuts the
    // jump short, which is not what is being measured.
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
        "throwing spells on the way down bought no airtime at all: {floaty} against {plain}"
    );
    assert!(
        floaty < plain * 3,
        "alternating air clicks kept her up for {floaty} frames against {plain} falling -- \
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
