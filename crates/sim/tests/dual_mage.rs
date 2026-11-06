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
    // Two keys, two majors: `Q` the dark one, `E` the light one -- each with
    // its bar full, which is what it vents (2026-11-06).
    let mut w = mage();
    w.players[0].mechanic = meter_at(100, 100, Force::Dark);
    step(&mut w, 1, Q);
    assert_eq!(w.players[0].action.attack_kind(), Some(dual::ABYSS));

    let mut w = mage();
    w.players[0].mechanic = meter_at(100, 100, Force::Dark);
    step(&mut w, 1, E);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(dual::JUDGEMENT),
        "`E` does nothing, so the light major is unreachable"
    );
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
        primed: 0,
        chained: 0,
        chain_out: 0,
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
        primed: 0,
        chained: 0,
        chain_out: 0,
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
        // From full, so that the majors have a bar to spend.
        w.players[0].mechanic = meter_at(100, 100, colour(&w));
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

fn fx(v: f32) -> Fx {
    Fx::from_raw((v * 65536.0).round() as i32)
}

// ---------------------------------------------------------------------------
// The majors
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// The bars build to her finishers (2026-11-04, from play)
// ---------------------------------------------------------------------------
//
// The owner: the bars "feel like a chore rather than a part of the class" --
// "you fight to a big moment, not to always manage your bars, and building
// both gives you a bigger annihilation". They fill on a hit, never on a
// press; nothing moves them on their own and nothing burns her; only the
// majors read them, and both full is the grand annihilation.

#[test]
fn pressing_at_nothing_fills_nothing() {
    for bits in [L, R, M, L | R] {
        let mut w = mage();
        w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(-12));
        w.players[0].mechanic = meter_at(30, 30, Force::Dark);
        step(&mut w, 1, bits);
        step(&mut w, 60, 0);
        assert_eq!(
            (dark(&w).to_int(), light(&w).to_int()),
            (30, 30),
            "{bits:#x} at nothing moved a bar"
        );
    }
}

#[test]
fn a_strike_that_lands_fills_its_own_bar_and_both_fill_both() {
    for (kind, dark_by, light_by) in [
        (dual::DARK_AUTO, 1, 0),
        (dual::LIGHT_AUTO, 0, 1),
        (dual::STORM, 0, 0),
        (dual::CHAIN, 1, 1),
        (dual::ANNIHILATION, 1, 1),
        (dual::LANCE, 0, 0),
        (dual::ABYSS, 0, 0),
    ] {
        let mut w = mage();
        w.players[0].mechanic = meter_at(30, 30, Force::Dark);
        sim::dual::filled_by_a_hit(&mut w.players[0], kind);
        let step_ = t::meter_auto_push();
        assert_eq!(
            (dark(&w).to_int(), light(&w).to_int()),
            (30 + dark_by * step_, 30 + light_by * step_),
            "{} filled the wrong bars",
            sim::moves::get(Class::DualMage, kind).name
        );
    }
}

#[test]
fn b4_alternating_strikes_on_a_body_fill_both_bars_in_three_to_eight_seconds() {
    // The rhythm to the big moment: both hands landing in turn, from empty,
    // until the next annihilation would be the grand one.
    let mut w = mage();
    let stand = w.players[0].pos.add(w.players[0].facing.scale(dummy_gap()));
    w.players[1].pos = stand;
    let mut presses = 0;
    let mut brim = None;
    for frame in 0..1800u32 {
        let next = [L, R][presses % 2];
        let bits = if ready(&w, next) {
            presses += 1;
            next
        } else {
            0
        };
        let pitch = onto_them(&w);
        step_looking(&mut w, bits, pitch);
        w.players[1].health = w.players[1].full_health();
        w.players[1].pos = stand;
        if sim::dual::brimming(&w.players[0]) {
            brim = Some(frame);
            break;
        }
    }
    let secs = brim.map(|f| f as f32 / 60.0);
    assert!(
        // Five or so strikes a hand since 2026-11-06 (the owner: "maybe 5
        // autos to max bars, not the 20ish it is now").
        secs.is_some_and(|s| (3.0..=8.0).contains(&s)),
        "both bars full at {secs:?} s"
    );
}

// ---------------------------------------------------------------------------
// The bars are energy to vent (2026-11-06, from play)
// ---------------------------------------------------------------------------

#[test]
fn nothing_reads_a_bar() {
    // The owner: "the bars don't add a damage multiplier, just how much
    // energy she has to vent".
    let mut w = mage();
    w.players[0].mechanic = meter_at(high(), 0, Force::Light);
    let p = &w.players[0];
    for kind in [
        dual::DARK_AUTO,
        dual::LIGHT_AUTO,
        dual::STORM,
        dual::LANCE,
        dual::CHAIN,
        dual::ANNIHILATION,
        dual::FLURRY_DASH,
        dual::ABYSS,
        dual::JUDGEMENT,
    ] {
        assert_eq!(sim::dual::depth_at(p, Some(kind)), Fx::ONE);
    }
}

#[test]
fn the_big_spells_need_the_bar_they_spend_and_the_rest_need_nothing() {
    // The storm and the lance a third of their bar, a major the whole of a
    // full one; without it they do not come out. The strikes and the middle
    // click are free.
    let p = |d: i32, l: i32| {
        let mut w = mage();
        w.players[0].mechanic = meter_at(d, l, Force::Dark);
        w.players[0]
    };
    let third = t::lance_spends();
    for (kind, short, enough) in [
        (dual::STORM, (third - 1, 100), (third, 0)),
        (dual::LANCE, (100, third - 1), (0, third)),
        (dual::ABYSS, (99, 100), (100, 0)),
        (dual::JUDGEMENT, (100, 99), (0, 100)),
    ] {
        let name = sim::moves::get(Class::DualMage, kind).name;
        assert!(
            !sim::dual::affords(&p(short.0, short.1), kind),
            "{name} without the bar"
        );
        assert!(
            sim::dual::affords(&p(enough.0, enough.1), kind),
            "{name} with it"
        );
    }
    for kind in [
        dual::DARK_AUTO,
        dual::LIGHT_AUTO,
        dual::CHAIN,
        dual::ANNIHILATION,
    ] {
        assert!(sim::dual::affords(&p(0, 0), kind));
    }
    // And a major spends the whole of it.
    let mut w = mage();
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(-12));
    w.players[0].mechanic = meter_at(100, 100, Force::Dark);
    step(&mut w, 1, Q);
    assert_eq!(w.players[0].action.attack_kind(), Some(dual::ABYSS));
    assert_eq!((dark(&w).to_int(), light(&w).to_int()), (0, 100));
    let mut w = mage();
    w.players[0].mechanic = meter_at(90, 90, Force::Dark);
    step(&mut w, 1, Q);
    assert_ne!(w.players[0].action.attack_kind(), Some(dual::ABYSS));
}

#[test]
fn the_bars_never_move_on_their_own_and_burn_her_only_past_a_third_apart() {
    for (d, l, burns) in [
        (100, 0, true),
        (0, 100, true),
        (60, 30, false),
        (100, 100, false),
    ] {
        let mut w = mage();
        w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(-12));
        w.players[0].mechanic = meter_at(d, l, Force::Dark);
        let health = w.players[0].health;
        step(&mut w, 600, 0);
        assert_eq!((dark(&w).to_int(), light(&w).to_int()), (d, l));
        assert_eq!(w.players[0].health < health, burns, "{d}/{l}");
        assert!(w.players[0].health >= 1);
        assert_eq!(ascending(&w), 0, "{d}/{l} ascended on its own");
    }
}

#[test]
fn b3_a_judgement_is_at_most_a_fifth_of_a_health_bar() {
    // Toned down since a full bar is five strikes away (2026-11-06).
    let bar = sim::state::max_health();
    let dealt = dealt_from(E, 100);
    assert!(
        dealt > 0 && dealt * 5 <= bar,
        "a Judgement took {dealt} of {bar}"
    );
}

#[test]
fn the_majors_land_where_she_stands_and_she_dashes_out() {
    for bits in [Q, E] {
        let mut w = mage();
        w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::from_int(-12));
        w.players[0].mechanic = meter_at(100, 100, Force::Dark);
        // In the open: she lunges back from the way she faces.
        w.players[0].pos = V3::new(Fx::from_int(6), Fx::ZERO, Fx::ZERO);
        let stood = w.players[0].pos;
        step(&mut w, 1, bits);
        assert!(
            w.players[0].aim_path.to.sub(stood).len().raw() < Fx::ONE.raw(),
            "{bits:#x}: aimed at {:?} from {:?}",
            w.players[0].aim_path,
            stood
        );
        step(&mut w, 80, 0);
        assert!(
            w.players[0].pos.sub(stood).len().raw() > Fx::from_int(2).raw(),
            "{bits:#x}: she stayed in it"
        );
    }
}
