//! The Shadow Reaver's second body.
//!
//! One property under all of it: **the shadow is never nowhere.** It attends
//! her or it stands out on the field, and every option the class has is a
//! function of the line between the two. The old mechanic allowed a third
//! state — no shadow at all — and the class spent half a match with its whole
//! vocabulary greyed out.
//!
//! What is pinned here is what a player would notice if it broke: that holding
//! the shadow is worth a quarter again on every swing, that the copy lands a
//! beat late and from somewhere else, that the recall cuts on its way home, and
//! that the forward dodge is the dash.

use sim::class::{Ghost, Mechanic, Shadow};
use sim::state::{Action, SLOT_COMMITTED, SLOT_MECHANIC, SLOT_POKE};
use sim::tuning as t;
use sim::{Class, Fx, Input, V3, World};

const E: u16 = Input::MECHANIC;
const L: u16 = Input::LEFT;
/// Right click, which on this class sends the shadow -- it is the half of the
/// kit the crosshair aims, and the mouse is where aiming lives.
const R: u16 = Input::RIGHT;
const W: u16 = Input::W;
const SHIFT: u16 = Input::SHIFT;

fn down(degrees: i32) -> i16 {
    (-degrees * 65536 / 360) as i16
}

fn run(w: &mut World, frames: u32, bits: u16, pitch: i16) {
    for _ in 0..frames {
        w.advance([Input::looking_at(bits, 0, pitch), Input::default()]);
    }
}

fn tap(w: &mut World, bits: u16, pitch: i16, then: u32) {
    run(w, 2, bits, pitch);
    run(w, then, 0, pitch);
}

fn shadow(w: &World) -> Shadow {
    match w.players[0].mechanic {
        Mechanic::Shadow(s) => s,
        _ => panic!("player one is not the Reaver"),
    }
}

/// A Reaver facing down +X with a dummy standing two metres in front of her.
///
/// Two metres is inside her poke and inside her shadow's copy of it, which is
/// the arrangement the damage tests need: both bodies reach the same target.
fn duel() -> World {
    let mut w = World::with_classes([Class::ShadowReaver, Class::Bulwark]);
    w.players[0].pos = V3::new(Fx::from_int(-6), Fx::ZERO, Fx::ZERO);
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    // Close, because her own hit knocks him back and the copy lands four
    // frames later from a step further away again.
    w.players[1].pos = V3::new(Fx::ratio(-49, 10), Fx::ZERO, Fx::ZERO);
    w.players[1].facing = V3::new(Fx::ONE.neg(), Fx::ZERO, Fx::ZERO);
    // Let the shadow settle onto her heel before anything is measured.
    run(&mut w, 30, 0, 0);
    w
}

fn hurt(w: &World) -> i32 {
    sim::state::max_health() - w.players[1].health
}

// ---------------------------------------------------------------------------
// It is always somewhere
// ---------------------------------------------------------------------------

#[test]
fn the_shadow_is_never_nowhere() {
    // The whole mechanic in one assertion. Through a throw, a wait, a recall
    // and the journey home, `placed()` always answers -- which is what lets
    // Guillotine lotus be aimed at the mechanic without a branch for the case
    // where the mechanic does not exist.
    let mut w = duel();
    let mut seen = Vec::new();
    for frame in 0..160 {
        let bits = if frame == 10 || frame == 90 { R } else { 0 };
        run(&mut w, 1, bits, down(15));
        assert!(
            w.players[0].mechanic.placed().is_some(),
            "frame {frame}: the shadow was nowhere"
        );
        let doing = shadow(&w).doing;
        if seen.last() != Some(&std::mem::discriminant(&doing)) {
            seen.push(std::mem::discriminant(&doing));
        }
    }
    // Attending, out, standing, home, attending: every state was visited, so
    // the assertion above was not just watching one of them.
    assert!(
        seen.len() >= 4,
        "the shadow only ever did {} things, so this proves less than it looks",
        seen.len()
    );
}

#[test]
fn the_shadow_stands_behind_her_rather_than_inside_her() {
    // Presentation with teeth: the copy swings from wherever the shadow is, so
    // two bodies in the same place would be two threats a player cannot tell
    // apart -- and one of them hits for a quarter of the other.
    let w = duel();
    let gap = shadow(&w).pos.sub(w.players[0].pos);
    assert!(
        gap.flat_len().raw() > t::shadow_trail().mul(Fx::ratio(1, 2)).raw(),
        "the shadow is standing on top of her"
    );
    assert!(
        gap.dot(w.players[0].facing).raw() < 0,
        "the shadow is standing in front of her, where it will be mistaken for her"
    );
}

// ---------------------------------------------------------------------------
// The copy
// ---------------------------------------------------------------------------

#[test]
fn the_shadow_copies_her_swing_a_beat_late() {
    let mut w = duel();
    let poke = sim::moves::get(Class::ShadowReaver, SLOT_POKE);

    // Her own hit lands first, on its own.
    let mut hers = None;
    for frame in 0..(poke.whiff_cost() as u32 + t::shadow_lag() as u32 + 4) {
        let bits = if frame < 2 { L } else { 0 };
        run(&mut w, 1, bits, 0);
        if hers.is_none() && hurt(&w) > 0 {
            hers = Some(hurt(&w));
        }
    }
    let first = hers.expect("her own poke never landed");
    assert_eq!(
        first, poke.damage,
        "her poke dealt something other than its own damage"
    );
    assert!(
        hurt(&w) > first,
        "the shadow never repeated the swing -- only {first} damage in total"
    );
}

#[test]
fn the_copy_is_worth_a_quarter_of_the_swing_it_copies() {
    // The number the class is balanced around: holding the shadow is a
    // twenty-five per cent damage buff on everything her body does, paid for by
    // not having the second body anywhere useful.
    let mut w = duel();
    let poke = sim::moves::get(Class::ShadowReaver, SLOT_POKE);
    tap(
        &mut w,
        L,
        0,
        poke.whiff_cost() as u32 + t::shadow_lag() as u32 + 8,
    );

    let total = hurt(&w);
    let echoed = total - poke.damage;
    let want = Fx::from_int(poke.damage).mul(t::shadow_echo()).to_int();
    assert_eq!(
        echoed, want,
        "the copy dealt {echoed} where the swing dealt {} -- the quarter is the class",
        poke.damage
    );
}

#[test]
fn a_shadow_out_on_the_field_swings_from_out_there() {
    // The other half of what the copy is for. With the shadow sent away, her
    // swing comes out twice in two places, so a Reaver with the shadow well
    // placed threatens ground she is not standing on.
    let mut w = World::with_classes([Class::ShadowReaver, Class::Bulwark]);
    w.players[0].pos = V3::new(Fx::from_int(-6), Fx::ZERO, Fx::ZERO);
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    // Out of the way while the shadow is thrown.
    w.players[1].pos = V3::new(Fx::from_int(-6), Fx::ZERO, Fx::from_int(9));
    run(&mut w, 20, 0, 0);

    let send = sim::moves::get(Class::ShadowReaver, SLOT_MECHANIC);
    tap(
        &mut w,
        R,
        down(30),
        (send.whiff_cost() + t::shadow_send_frames()) as u32,
    );
    // Then stand him on it, a step in front of where the copy will swing from.
    let out = shadow(&w).pos;
    w.players[1].pos = V3::new(out.x.add(Fx::ONE), Fx::ZERO, out.z);
    let poke = sim::moves::get(Class::ShadowReaver, SLOT_POKE);
    assert!(
        w.players[0].pos.sub(w.players[1].pos).flat_len().raw() > poke.reach.raw() * 2,
        "the fixture left her close enough to reach him herself"
    );

    let before = hurt(&w);
    tap(
        &mut w,
        L,
        0,
        poke.whiff_cost() as u32 + t::shadow_lag() as u32 + 8,
    );
    assert!(
        hurt(&w) > before,
        "she swung at nothing and her shadow, standing on him, swung at nothing too"
    );
}

// ---------------------------------------------------------------------------
// Sending it, and getting it back
// ---------------------------------------------------------------------------

#[test]
fn the_shadow_flies_out_and_then_stops() {
    // "Quickly out, then a pause at the end of it." The pause is what the rest
    // of the kit is aimed at, so the throw has to actually finish rather than
    // drifting.
    let mut w = duel();
    let send = sim::moves::get(Class::ShadowReaver, SLOT_MECHANIC);
    run(&mut w, 2, R, down(25));
    run(&mut w, send.startup as u32, 0, down(25));

    let mut steps = Vec::new();
    let mut last = shadow(&w).pos;
    for _ in 0..(t::shadow_send_frames() as u32 + 20) {
        run(&mut w, 1, 0, down(25));
        let now = shadow(&w).pos;
        steps.push(now.sub(last).flat_len());
        last = now;
    }
    assert!(
        matches!(shadow(&w).doing, Ghost::Waiting),
        "it never settled"
    );
    let quickest = steps.iter().map(|s| s.raw()).max().unwrap_or(0);
    assert!(
        Fx::from_raw(quickest).raw() > t::move_speed().mul(sim::DT).raw() * 3,
        "the shadow ambled out at walking pace"
    );
    // And it is standing still by the end.
    assert_eq!(steps.last().map(|s| s.raw()), Some(0), "it never stopped");
}

#[test]
fn the_leash_fits_the_throw() {
    // A leash shorter than the throw would have the shadow turn round on the
    // frame it landed, which is the class's setup deleting itself.
    let send = sim::moves::get(Class::ShadowReaver, SLOT_MECHANIC);
    assert!(
        t::shadow_leash().raw() > send.reach.raw(),
        "the shadow can be thrown {} m and is leashed at {} m",
        send.reach.to_f32_for_render(),
        t::shadow_leash().to_f32_for_render()
    );
}

#[test]
fn the_recall_cuts_and_slows_what_it_comes_home_through() {
    let mut w = World::with_classes([Class::ShadowReaver, Class::Bulwark]);
    w.players[0].pos = V3::new(Fx::from_int(-6), Fx::ZERO, Fx::ZERO);
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    // Out of the way of the throw, and back in the way of the return.
    w.players[1].pos = V3::new(Fx::from_int(-3), Fx::ZERO, Fx::from_int(6));
    run(&mut w, 20, 0, 0);

    let send = sim::moves::get(Class::ShadowReaver, SLOT_MECHANIC);
    tap(
        &mut w,
        R,
        down(25),
        (send.whiff_cost() + t::shadow_send_frames()) as u32,
    );
    assert_eq!(hurt(&w), 0, "the throw itself hurt somebody");

    // Stand him on the line home and call it back.
    let out = shadow(&w).pos;
    let midway = V3::new(
        out.x.add(w.players[0].pos.x).mul(Fx::ratio(1, 2)),
        Fx::ZERO,
        out.z.add(w.players[0].pos.z).mul(Fx::ratio(1, 2)),
    );
    w.players[1].pos = midway;
    run(&mut w, 2, R, down(25));
    let mut slowed = false;
    for _ in 0..60 {
        run(&mut w, 1, 0, down(25));
        slowed |= w.players[1].slowed > 0;
    }

    assert_eq!(
        hurt(&w),
        send.damage,
        "the shadow came home through him without cutting him exactly once"
    );
    assert!(
        slowed,
        "the recall cut him but did not slow him, which is the half that matters"
    );
}

// ---------------------------------------------------------------------------
// The dash
// ---------------------------------------------------------------------------

#[test]
fn a_forward_dodge_at_the_shadow_crosses_to_it() {
    let mut w = duel();
    let send = sim::moves::get(Class::ShadowReaver, SLOT_MECHANIC);
    tap(
        &mut w,
        R,
        down(25),
        (send.whiff_cost() + t::shadow_send_frames()) as u32,
    );
    let out = shadow(&w).pos;
    let stood = w.players[0].pos;
    assert!(
        out.sub(stood).flat_len().raw() > Fx::from_int(4).raw(),
        "the fixture left the shadow within arm's reach, so arriving proves nothing"
    );

    // Shift and forward, looking at it. The camera is behind her and she is
    // facing the shadow, so the crosshair is on it.
    let mut sighted = false;
    for _ in 0..(t::dodge_frames() as u32 + 10) {
        run(&mut w, 1, SHIFT | W, down(10));
        sighted |= w.players[0].action.invulnerable();
    }
    assert!(sighted, "she was never invulnerable, so it was not a dodge");
    assert!(
        w.players[0].pos.sub(out).flat_len().raw() < Fx::from_int(1).raw(),
        "she ended {:.1} m from the shadow she dashed at",
        w.players[0].pos.sub(out).flat_len().to_f32_for_render()
    );
    // Arriving picks it up: the loop is throw, act, dash back on, throw again.
    assert!(
        !shadow(&w).is_out(),
        "she crossed to the shadow and left it standing there"
    );
}

#[test]
fn a_forward_dodge_with_the_shadow_at_her_heel_is_just_a_dodge() {
    // The dash is a decision, not a thing that happens to her. With nothing out
    // on the field there is nothing to point at, so shift and forward is the
    // universal defensive option and behaves like everyone else's.
    let mut w = duel();
    let stood = w.players[0].pos;
    run(&mut w, t::dodge_frames() as u32 + 4, SHIFT | W, 0);
    let went = w.players[0].pos.sub(stood).flat_len();
    assert!(
        went.raw() > Fx::ONE.raw(),
        "she did not move at all, so the dodge did not happen"
    );
    assert!(!shadow(&w).is_out());
}

// ---------------------------------------------------------------------------
// In a hunt
// ---------------------------------------------------------------------------

/// A Reaver standing next to a creature that is holding still, so a test about
/// the shadow is about the shadow rather than about chasing.
fn hunting() -> World {
    let mut w = World::hunt([Class::ShadowReaver, Class::Bulwark]);
    let mut beast = w.monster.expect("a hunt has a creature");
    // Close enough that the copy, which swings from a step behind her, reaches
    // it too -- the point of the test is the second body, not her spacing.
    // Placed by its *head* rather than by its centre: it is thirteen metres
    // long, so where the middle of it is says nothing about what either body
    // can reach. Its head goes between the two of them.
    let head = sim::monster::shape(sim::monster::HEAD);
    let mid = head.min.x.add(head.max.x).mul(Fx::ratio(1, 2));
    beast.pos = V3::new(
        mid.sub(t::shadow_trail().mul(Fx::ratio(1, 2))),
        Fx::ZERO,
        Fx::ZERO,
    );
    // Facing back down the arena, and not thinking about anything.
    beast.yaw = Fx::from_raw(1 << 15);
    beast.doing = sim::monster::Doing::Prowl;
    beast.brain.think_left = u16::MAX;
    w.monster = Some(beast);
    w.players[0].pos = V3::new(Fx::ZERO, Fx::ZERO, Fx::ZERO);
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    w.players[1].pos = V3::new(Fx::from_int(-12), Fx::ZERO, Fx::ZERO);
    run(&mut w, 20, 0, 0);
    w
}

fn beast_health(w: &World) -> i32 {
    w.monster.expect("a hunt has a creature").health
}

#[test]
fn the_shadow_fights_the_creature_too() {
    // A mechanic that only works in versus is half a mechanic, and this is
    // exactly the shape of bug that hid in the Blood mage's fields for months:
    // it is invisible in the mode most tests are written in.
    let mut w = hunting();
    let full = beast_health(&w);
    let poke = sim::moves::get(Class::ShadowReaver, SLOT_POKE);

    // Her own swing lands first, alone.
    let mut hers = None;
    for frame in 0..(poke.whiff_cost() as u32 + t::shadow_lag() as u32 + 8) {
        let bits = if frame < 2 { L } else { 0 };
        run(&mut w, 1, bits, 0);
        if hers.is_none() && beast_health(&w) < full {
            hers = Some(full - beast_health(&w));
        }
    }
    let first = hers.expect("she never reached the creature");
    assert!(
        full - beast_health(&w) > first,
        "the shadow never copied the swing onto the creature -- {first} damage in total"
    );
}

#[test]
fn the_recall_cuts_the_creature_on_its_way_home() {
    let mut w = hunting();
    let send = sim::moves::get(Class::ShadowReaver, SLOT_MECHANIC);
    // Out past the animal, so the way home crosses it.
    tap(
        &mut w,
        R,
        0,
        (send.whiff_cost() + t::shadow_send_frames()) as u32,
    );
    assert!(shadow(&w).is_waiting(), "the shadow never got out");
    let before = beast_health(&w);
    tap(&mut w, R, 0, 60);
    assert!(
        beast_health(&w) < before,
        "the shadow came home through the creature without touching it"
    );
}

// ---------------------------------------------------------------------------
// Which button is which
// ---------------------------------------------------------------------------
//
// The two are swapped against every other class, and the swap is the point:
// **the mouse means where.** Sending the shadow is the only thing in this kit
// the crosshair aims, so it is on the mouse; Executioner is a swing off the
// body and does not care, so it is on the key.

#[test]
fn right_click_sends_the_shadow() {
    let mut w = duel();
    run(&mut w, 2, R, 0);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(SLOT_MECHANIC),
        "right click did not throw Send shadow"
    );
}

#[test]
fn the_mechanic_key_throws_her_committed_melee() {
    // The one class where `E` carries a move that is not the mechanic. Worth a
    // test rather than a comment: `on_e` is the only thing that says so, and it
    // is one word away from being the slot everybody else puts there.
    let mut w = duel();
    run(&mut w, 2, E, 0);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(SLOT_COMMITTED),
        "`E` did not throw Executioner"
    );
}

#[test]
fn shift_and_left_click_still_throws_the_same_melee() {
    // The shared grammar is untouched: the swap gave Executioner a second home,
    // it did not move it out of the one every class has.
    let mut w = duel();
    run(&mut w, 2, SHIFT | L, 0);
    assert_eq!(
        w.players[0].action.attack_kind(),
        Some(SLOT_COMMITTED),
        "shift + left click stopped throwing the committed move"
    );
}

#[test]
fn right_click_still_guards_for_the_class_that_has_a_shield() {
    let mut w = World::with_classes([Class::Bulwark, Class::Bulwark]);
    run(&mut w, 2, R, 0);
    assert!(
        matches!(w.players[0].action, Action::Guard { .. }),
        "the Bulwark stopped guarding when the Reaver took right click"
    );
}
