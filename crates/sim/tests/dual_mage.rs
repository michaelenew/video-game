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
    sim::aim::look_onto_closely(
        w.players[0].pos,
        LOOK,
        w.players[0].aloft,
        w.players[0].clear,
        them,
    )
}

/// The move a button throws from the floor.
fn floor_kind(bits: u16) -> u8 {
    match bits {
        b if b == L => dual::DARK_AUTO,
        b if b == R => dual::LIGHT_AUTO,
        b if b == M => dual::CHAIN,
        b if b == Q => dual::ABYSS,
        b if b == E => dual::JUDGEMENT,
        b if b == SPACE | L => dual::VEIL,
        b if b == SPACE | R => dual::SANCTUARY,
        b if b == SPACE | M => dual::EQUINOX,
        _ => panic!("no floor move on {bits:b}"),
    }
}

/// Free, on the floor, and the move `bits` throws is not locked out: a press
/// now is that move. A script that pressed into the repeat lockout would be
/// counting presses that threw nothing.
fn ready(w: &World, bits: u16) -> bool {
    let p = &w.players[0];
    let kind = floor_kind(bits);
    p.action.actionable() && p.grounded && !p.locked_out(kind)
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
    tap(&mut w, bits);
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
fn middle_click_throws_binary_whatever_she_carries() {
    // Middle click has no side, so it cannot pick a direction on the bar --
    // which is what makes it the home of twilight, the one spell made of both.
    // It used to be the Lance, whose form was the force she carried; since the
    // rework it is one move, and the force she carries is left as it was.
    for force in [Force::Light, Force::Dark] {
        let mut w = mage();
        w.players[0].mechanic = meter_at(0, 0, force);
        // A tap: pressed and let go before the Flurry dash's hold fills.
        step(&mut w, 1, M);
        step(&mut w, 1, 0);
        assert_eq!(
            w.players[0].action.attack_kind(),
            Some(dual::CHAIN),
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
    assert_eq!(kind_thrown(L | Input::SHIFT), dual::DARK_AUTO);
    assert_eq!(kind_thrown(R | Input::SHIFT), dual::LIGHT_AUTO);
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
        (L, "dark auto"),
        (R, "light auto"),
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
        sim::moves::get(Class::DualMage, dual::DARK_AUTO).hand,
        Hand::Left
    );
    assert_eq!(
        sim::moves::get(Class::DualMage, dual::LIGHT_AUTO).hand,
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
    // Fifteen, and fourteen of them are the Dual mage. Since the 2026-10-09
    // rework every move of hers but the twilight column is made of one force,
    // and the hand it leaves from is that force: left dark, right light (the
    // spellbook of 2026-10-12 kept the rule, and so do the wing autos). A spell off
    // the centre line could not say which one it was. The Champion's spear
    // opener is sided because it is the one attack in that class thrown with
    // one arm: a jab off the leading hand with the butt of the shaft still at
    // the hip, which is most of why it reads as a poke rather than as a short
    // lunge.
    let declared: &[(Class, u8)] = &[
        (Class::DualMage, dual::DARK_AUTO),
        (Class::DualMage, dual::VEIL),
        (Class::DualMage, dual::ABYSS),
        (Class::DualMage, dual::LIGHT_AUTO),
        (Class::DualMage, dual::DESCENT),
        (Class::DualMage, dual::SANCTUARY),
        (Class::DualMage, dual::JUDGEMENT),
        (Class::DualMage, dual::GRASPING_SHADE),
        (Class::DualMage, dual::DEVOUR),
        (Class::DualMage, dual::HALO),
        (Class::DualMage, dual::SMITE),
        (Class::DualMage, dual::STORM),
        (Class::DualMage, dual::LANCE),
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
        hopped: false,
        dash_jump: false,
        wave: Fx::ZERO,
        springs: 0,
        storm: 0,
        storm_at: sim::math::V3::ZERO,
        strike: 0,
        strike_at: sim::math::V3::ZERO,
        singe: Fx::ZERO,
        veil: 0,
        warded: false,
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
        hopped: false,
        dash_jump: false,
        wave: Fx::ZERO,
        springs: 0,
        storm: 0,
        storm_at: sim::math::V3::ZERO,
        strike: 0,
        strike_at: sim::math::V3::ZERO,
        singe: Fx::ZERO,
        veil: 0,
        warded: false,
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
fn every_move_moves_a_bar_with_nothing_in_range() {
    // **The bug this class shipped with.** The autos only steered on contact
    // and the casts took their direction from an auto that had never landed, so
    // a player standing where they spawn -- eight metres from anybody -- could
    // press every button on the class and watch the bar sit at zero. A resource
    // you cannot move without a target is one nobody can learn or tune. Since
    // 2026-10-12 the autos and twilight fill from empty and the majors spend,
    // so the majors are thrown from half.
    for (bits, from, name) in [
        (L, 0, "dark auto"),
        (R, 0, "light auto"),
        (M, 0, "binary"),
        (Q, 50, "abyss"),
        (E, 50, "judgement"),
    ] {
        let mut w = mage();
        let gap = w.players[1]
            .pos
            .sub(w.players[0].pos)
            .flat_len()
            .to_f32_for_render();
        assert!(gap > 4.0, "the fixture is in range, so it proves nothing");
        w.players[0].mechanic = meter_at(from, from, Force::Dark);
        let before = goaded(&w);
        tap(&mut w, bits);
        assert!(
            goaded(&w) != before,
            "{name} thrown at nothing moved neither bar at all"
        );
    }
}

/// Press `bits` for a frame -- and, for a bare click, let it go the next,
/// which is what makes it a tap rather than its long hold (2026-10-13: every
/// click is undecided until it is let go or held full).
fn tap(w: &mut World, bits: u16) {
    step(w, 1, bits);
    if bits & (L | M | R) != 0 && bits & SPACE == 0 {
        step(w, 1, 0);
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
            tap(&mut w, bits);
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
        // From half, so that the majors have a bar to spend.
        w.players[0].mechanic = meter_at(50, 50, colour(&w));
        tap(&mut w, bits);
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
    cast_from(bits, in_the_air, carrying, 0)
}

/// [`cast_from_empty`], from both bars at `from`.
fn cast_from(bits: u16, in_the_air: bool, carrying: Force, from: i32) -> World {
    let mut w = mage();
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(-12));
    w.players[0].mechanic = meter_at(from, from, carrying);
    if in_the_air {
        aloft(&mut w, 6);
    }
    tap(&mut w, bits);
    w
}

#[test]
fn twilight_goads_both_bars_by_an_auto_and_keeps_the_force_she_carries() {
    // The middle column is both forces at once: each bar by an auto's push,
    // so it never widens the gap.
    let one = Fx::from_int(t::meter_auto_push());
    for (bits, air, kind) in [
        (M, false, dual::CHAIN),
        (M, true, dual::CHAIN),
        (L | R, false, dual::ANNIHILATION),
        (L | R, true, dual::ANNIHILATION),
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
        step(&mut w, 1, L);
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
    // A gap of twenty rather than just past the band, because the higher
    // bar's rise is net of the calm: just outside the band the drift is
    // smaller than the calm and the higher bar holds rather than rises, while
    // the gap still widens. (It was a finisher's push from level, until the
    // majors spent rather than pushed, 2026-10-12.)
    let finisher = 70;
    assert!(
        finisher - 50 > t::meter_band(),
        "the fixture does not leave the band"
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
            // Full is as high as it goes: with no calm under it, the higher
            // bar reaches the top and holds there (2026-10-14).
            let full = hi.raw() >= Fx::from_int(t::meter_max()).raw();
            assert!(
                hi.raw() > last_hi.raw() || full,
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

/// The rhythm the tiers are climbed with: dark, light, dark, light -- the
/// two wing autos in turn (since the second pass of 2026-10-12; the lash and
/// the palm that morning, the Shade bolt and the Sunray before).
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
fn stopping_keeps_the_tiers_she_built() {
    // Benchmark B5, reversed on 2026-10-14 (the owner: no passive drain). It
    // was the calm -- stop goading and the second tier went almost at once,
    // the first inside seven exchanges. Now the bars stay where she put them:
    // seven exchanges of stillness at two thirds and she still has the
    // second jump. The way down short of overflowing is to cash out (`Q`, `E`).
    let mut w = mage();
    w.players[0].mechanic = meter_at(t::tier_jump(), t::tier_jump(), Force::Dark);
    assert_eq!(tier(&w), Tier::Jump);
    step(&mut w, 7 * exchange(), 0);
    assert_eq!(
        tier(&w),
        Tier::Jump,
        "seven exchanges of stillness and the second jump went"
    );
}

#[test]
fn a_bar_left_fully_one_sided_burns_most_of_a_health_bar_in_half_a_round_and_never_all_of_it() {
    // Benchmark B7. The burn is the price of ignoring the hill, and it has to
    // be one a player feels: fully one-sided and uncorrected for thirty
    // seconds costs between half and four fifths of a health bar. Over a whole
    // round it still cannot be all of it. Until 2026-10-14 the calm brought
    // the gap back inside the band on its own and the burn stopped there; with
    // the calm off nothing does, and it runs until she cashes out.
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
// The scripts were rewritten for the 2026-10-09 spell rework, and again for
// the spellbook of 2026-10-12: alternating is the two wing autos in turn,
// one-sided is the dark wing over and over and the Abyss, and the finisher is the carried force's major. The dummy stands in sword
// range.

/// How far in front of her the dummy stands: **sword range** since the
/// spellbook (2026-10-12) -- inside the wings' reach, the Halo's ring and the
/// orbit of Binary's stars, where every spell of hers is cast.
fn dummy_gap() -> Fx {
    Fx::from_int(2)
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
    let (dealt, _) = against_a_dummy(1800, |n| [L, L, L, R, R, R][(n % 6) as usize]);
    let bar = sim::state::max_health();
    assert!(
        dealt >= bar && dealt * 4 <= 9 * bar,
        "alternating for half a round dealt {dealt} of a {bar} health bar"
    );
}

#[test]
fn b2_one_sided_wings_burn_her_and_the_major_is_the_valve() {
    // The burst play, since the majors spend (2026-10-12): the dark wing
    // only, on a target that stands in all of it, is the most the wings make
    // -- one to two bars in half a round (one and a quarter until the curse
    // went with its ticks, 2026-11-04) -- and the dark runs
    // away and burns her for two fifths to four fifths of her own. The same
    // hand with the Abyss every fourth throw spends the runaway as it builds:
    // it costs her nothing.
    let bar = sim::state::max_health();
    let (dealt, cost) = against_a_dummy(1800, |_| L);
    assert!(
        dealt >= bar && dealt <= 2 * bar,
        "the dark wing alone for half a round dealt {dealt} of a {bar} health bar"
    );
    assert!(
        cost * 5 >= 2 * bar && cost * 5 <= 4 * bar,
        "the dark wing alone for half a round cost her {cost} of {bar}"
    );
    let (_, vented) = against_a_dummy(1800, |n| [L, L, L, Q][(n % 4) as usize]);
    assert!(
        vented * 20 <= bar,
        "with the Abyss spending it, the dark hand still cost her {vented} of {bar}"
    );
}

#[test]
fn b9_the_majors_in_turn_with_nothing_built_are_spent_bars_not_a_rotation() {
    // The black sun, then the columns through it, over and over, with nothing
    // built between: since they spend (2026-10-12) each is thrown from an
    // empty bar after the first, so it is the spectacle at its smallest --
    // three quarters of a bar to a bar and a half in half a round -- and it
    // costs her nothing. The majors are what the building is for.
    let (dealt, cost) = against_a_dummy(1800, |n| [Q, E][(n % 2) as usize]);
    let bar = sim::state::max_health();
    assert!(
        dealt * 4 >= 3 * bar && dealt * 2 <= 3 * bar,
        "the majors in turn for half a round dealt {dealt} of a {bar} health bar"
    );
    assert!(cost * 20 <= bar, "the majors in turn cost her {cost}");
}

#[test]
fn b3_a_judgement_from_a_full_bar_is_at_most_a_quarter_of_a_health_bar() {
    // The biggest single strike in the kit, and the one that was too big. A
    // quarter from a full bar; still a real hit -- not under seven per cent --
    // from an empty one. Measured over the strike and its field on somebody
    // standing where it lands. Seven since 2026-10-21: knockback between
    // fighters starts at one and a half times the table (`stun.md`), so the
    // strike shoves them out of its own field two ticks sooner -- 76 where
    // it was 80, and the floor was eight.
    let bar = sim::state::max_health();
    let full = dealt_from(E, high());
    let empty = dealt_from(E, 0);
    assert!(
        full * 4 <= bar,
        "a Judgement from a full bar took {full} of {bar}, more than a quarter"
    );
    assert!(
        empty * 100 >= bar * 7,
        "a Judgement from an empty bar took {empty} of {bar}, under seven per cent"
    );
}

#[test]
fn b4_the_climb_takes_a_third_of_a_round_to_the_wings() {
    // Clean alternating autos from empty, in an empty arena, and nothing
    // vented: the blink in four to six seconds, the second jump in eight to
    // twelve, the wings in eleven to sixteen. Since 2026-10-12 the autos are
    // what builds and the tiers are at thirds, and since 2026-10-14 nothing
    // drains the bars on their own (the calm is off), so the climb is faster
    // than it was (five to eight, eleven to fifteen, sixteen to twenty-one
    // with the calm; ten to fourteen, sixteen to twenty-one, twenty to
    // twenty-six before the thirds) -- and in play it is slowed by every
    // spell she vents.
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
        blink.is_some_and(|s| (4.0..=6.0).contains(&s)),
        "the blink came at {blink:?} s"
    );
    assert!(
        jump.is_some_and(|s| (8.0..=12.0).contains(&s)),
        "the second jump came at {jump:?} s"
    );
    assert!(
        wings.is_some_and(|s| (11.0..=16.0).contains(&s)),
        "the wings came at {wings:?} s"
    );
}

#[test]
fn b6_a_major_from_level_is_caught_by_the_other_major_or_the_spent_hand() {
    // The hill, and the answer to it, since the majors spend (2026-10-12). An
    // Abyss from level empties the dark bar: the light is half a bar ahead and
    // outside the band. Answered with Judgement, both are spent and she is
    // level again -- the spectacle in turn is also the way back. Answered
    // instead with the hand she spent -- dark wings -- she is back inside the
    // band within two exchanges of the Abyss recovering: slower, because the
    // runaway pulls from the bar she is rebuilding. A major from level is a
    // commitment to the other one.
    let spent = || {
        let mut w = mage();
        w.players[1].pos = V3::new(fx(-12.0), Fx::ZERO, fx(-12.0));
        w.players[0].mechanic = meter_at(50, 50, Force::Dark);
        step(&mut w, 1, Q);
        assert!(
            !sim::dual::level(&w.players[0]),
            "an Abyss from level stayed level"
        );
        w
    };

    let mut w = spent();
    let mut caught = None;
    for frame in 1..=600u32 {
        let bits = if ready(&w, E) { E } else { 0 };
        step(&mut w, 1, bits);
        if sim::dual::level(&w.players[0]) {
            caught = Some(frame);
            break;
        }
    }
    let m = sim::moves::get(Class::DualMage, dual::ABYSS);
    let whole = (m.startup + m.active + m.recovery) as u32;
    assert!(
        caught.is_some_and(|f| f <= whole + 6),
        "Judgement after the Abyss caught it at {caught:?} frames; the Abyss is {whole}"
    );

    let mut w = spent();
    let mut recovered_at = None;
    let mut caught_at = None;
    for frame in 1..=600u32 {
        if w.players[0].action.actionable() && recovered_at.is_none() {
            recovered_at = Some(frame);
        }
        let bits = if ready(&w, L) { L } else { 0 };
        step(&mut w, 1, bits);
        if sim::dual::level(&w.players[0]) {
            caught_at = Some(frame);
            break;
        }
    }
    let recovered = recovered_at.expect("the Abyss never recovered");
    let caught = caught_at.expect("the spent hand never caught it");
    assert!(
        caught.saturating_sub(recovered) <= 2 * exchange(),
        "dark wings caught it {} frames after the Abyss recovered; an exchange is {}",
        caught.saturating_sub(recovered),
        exchange()
    );
}

#[test]
fn b7_ignoring_a_runaway_for_ten_seconds_costs_about_a_judgement() {
    // The burn as a consequence: a major from level, and then nothing for
    // ten seconds, costs her between ten and twenty-five per cent of a
    // health bar -- roughly what a Judgement does to them. Nobody is near the
    // well, so the Abyss drains nothing back to her. Since the majors spend
    // (2026-10-12) the bar left standing is the light, and it fades rather
    // than climbs, with the dark empty to take nothing from.
    let mut w = mage();
    w.players[1].pos = V3::new(fx(-12.0), Fx::ZERO, fx(-12.0));
    w.players[0].mechanic = meter_at(50, 50, Force::Dark);
    let before = w.players[0].health;
    step(&mut w, 1, Q);
    step(&mut w, 600, 0);
    let lost = before - w.players[0].health;
    let bar = sim::state::max_health();
    assert!(
        lost * 100 >= bar * 10 && lost * 100 <= bar * 25,
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
    tap(&mut w, R);
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
fn casts_thrown_while_ascending_read_the_top_of_the_curve() {
    let mut w = mage();
    w.players[0].mechanic = ascended(0, 0, Force::Dark);
    assert_eq!(
        sim::state::depth(&w.players[0]).raw(),
        t::depth_ceiling().raw()
    );
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
        (L, "dark auto"),
        (R, "light auto"),
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
        (dual::DARK_AUTO, true),
        (dual::STORM, true),
        (dual::VEIL, true),
        (dual::ABYSS, true),
        (dual::LIGHT_AUTO, false),
        (dual::DESCENT, false),
        (dual::SANCTUARY, false),
        (dual::JUDGEMENT, false),
        (dual::CHAIN, false),
        (dual::ANNIHILATION, false),
        (dual::LANCE, false),
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
        (at(Some(dual::LIGHT_AUTO)) - at(Some(dual::JUDGEMENT))).abs() < 0.001,
        "two light moves read different bars"
    );
    assert!(
        (at(None) - at(Some(dual::LIGHT_AUTO))).abs() < 0.001,
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
fn a_major_spends_the_whole_of_its_bar_and_is_worth_what_it_spent() {
    // **The majors spend** (2026-10-12): the Abyss empties the dark bar and
    // Judgement the light, and each is worth the bar it was thrown from --
    // `Player::thrown_at` is read before the spending. The reward for having
    // built, and the way out when one bar has run away: the runaway is the
    // high bar climbing, and spending it is what brings her back down.
    for (bits, force) in [(Q, Force::Dark), (E, Force::Light)] {
        let mut w = mage();
        w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(-12));
        w.players[0].mechanic = meter_at(90, 90, force);
        step(&mut w, 1, bits);
        let (own, other) = match force {
            Force::Dark => (dark(&w), light(&w)),
            Force::Light => (light(&w), dark(&w)),
        };
        assert_eq!(own.raw(), 0, "{bits:b} left its bar at {own:?}");
        assert!(
            (other.to_int() - 90).abs() <= 1,
            "{bits:b} moved the other bar to {other:?}"
        );
        assert!(
            w.players[0].thrown_at.raw() > Fx::ONE.raw(),
            "{bits:b} was worth the bar after it was spent"
        );
    }
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
    // again. In the air her clicks move her (2026-10-12): the Shadowstep and
    // Phase are thrown here at nothing, and the Descent -- the way down, by
    // design -- is left out.
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
                button = if button == L { M } else { L };
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

// ---------------------------------------------------------------------------
// The band's cadence, on the long holds (2026-11-04)
// ---------------------------------------------------------------------------

#[test]
fn one_hold_from_level_stays_inside_the_band_two_do_not_and_a_finisher_never_does() {
    // B8, since the spells that vented went (2026-11-04): a long hold fills
    // its own bar, and one is a lead she can sit on; a second is the hill.
    let band = t::meter_band();
    let steered = |dark: i32, light: i32, kinds: &[u8]| {
        let mut w = mage();
        w.players[0].mechanic = meter_at(dark, light, Force::Dark);
        for kind in kinds {
            sim::dual::steer(&mut w.players[0], *kind);
        }
        sim::dual::level(&w.players[0])
    };
    assert!(
        steered(50, 50, &[dual::STORM]),
        "one hold from level left the band"
    );
    assert!(
        !steered(50, 50, &[dual::STORM, dual::STORM]),
        "two holds stayed in it"
    );
    assert!(
        !steered(50, 50, &[dual::ABYSS]),
        "a finisher from level stayed in it"
    );
    assert!(steered(
        50,
        50,
        &[dual::DARK_AUTO, dual::LIGHT_AUTO, dual::DARK_AUTO]
    ));
    assert!(steered(50 + band, 50, &[]));
    assert!(!steered(50 + band, 50, &[dual::ABYSS]));
}

// ---------------------------------------------------------------------------
// Ascension: staying ahead of the crest (2026-11-04)
// ---------------------------------------------------------------------------
//
// `docs/design/exploration/0037_dual_mage_the_razor_edge.md`: both bars full
// and the beings stop waiting to be asked. The wave rises on its own, every
// move she throws pushes it back, and past its break they strike at her.

fn wave(w: &World) -> f32 {
    match w.players[0].mechanic {
        Mechanic::Meter { wave, .. } => wave.to_f32_for_render(),
        _ => 0.0,
    }
}

fn alone_ascended() -> World {
    let mut w = mage();
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(-12));
    w.players[0].mechanic = ascended(t::meter_max(), t::meter_max(), Force::Light);
    w
}

#[test]
fn ascension_ends_on_its_own_clock_with_both_bars_empty_and_nothing_else() {
    let mut w = alone_ascended();
    step(&mut w, t::ascension_frames() as u32 + 1, 0);
    assert_eq!(ascending(&w), 0, "it never ended");
    assert_eq!(dark(&w), Fx::ZERO);
    assert_eq!(light(&w), Fx::ZERO);
    assert_eq!(wave(&w), 0.0, "the wave outlived it");
    assert!(
        !matches!(w.players[0].action, Action::Stagger { .. }),
        "it ended in a stagger"
    );
}

#[test]
fn the_wave_rises_on_its_own_and_every_move_she_throws_pushes_it_back() {
    let mut w = alone_ascended();
    step(&mut w, 60, 0);
    let risen = wave(&w);
    assert!(
        risen > 1.0,
        "a second of ascension and the wave is at {risen}"
    );
    step(&mut w, 1, L);
    step(&mut w, 1, 0);
    assert!(
        wave(&w) < risen - (t::wave_vent() as f32) * 0.5,
        "an auto pushed the wave from {risen} to {}",
        wave(&w)
    );
}

#[test]
fn keeping_pace_keeps_the_beings_from_striking_and_standing_still_does_not() {
    let fight = |hands: bool| {
        let mut w = alone_ascended();
        let before = w.players[0].health;
        let mut presses = 0;
        for _ in 0..t::ascension_frames() {
            let next = CLIMB[presses % CLIMB.len()];
            let bits = if hands && ready(&w, next) {
                presses += 1;
                next
            } else {
                0
            };
            step(&mut w, 1, bits);
        }
        before - w.players[0].health
    };
    assert_eq!(fight(true), 0, "she kept pace and was struck anyway");
    let idle = fight(false);
    let bar = t::max_health();
    assert!(
        idle * 5 >= bar,
        "standing still through the whole of it cost her only {idle} of {bar}"
    );
}

#[test]
fn an_unbidden_strike_lands_where_she_was_and_misses_her_if_she_left() {
    let strike_at = |w: &World, at: V3| {
        let mut w = w.clone();
        if let Mechanic::Meter {
            strike, strike_at, ..
        } = &mut w.players[0].mechanic
        {
            *strike = 2;
            *strike_at = at;
        }
        let before = w.players[0].health;
        step(&mut w, 3, 0);
        before - w.players[0].health
    };
    let w = alone_ascended();
    let p = w.players[0].pos;
    let on_her = p;
    let left = V3::new(p.x.add(Fx::from_int(4)), p.y, p.z);
    assert!(
        strike_at(&w, on_her) > 0,
        "it landed on her and did nothing"
    );
    assert_eq!(strike_at(&w, left), 0, "it struck her four metres away");
}

#[test]
fn ascending_she_moves_as_she_always_does() {
    // No wing beats and no refusal: the dodge is hers, and space in the air
    // is nothing more than it is at an empty bar.
    let mut w = alone_ascended();
    assert!(sim::dual::may_dodge(&w.players[0]));
    assert!(!sim::dual::may_beat_wings(&w.players[0]));
    w.players[0].pos.y = Fx::from_int(6);
    w.players[0].grounded = false;
    step(&mut w, 1, SPACE);
    assert!(w.players[0].vel.y.raw() <= 0, "space in the air lifted her");
}
