//! The repeat lockout: you may not throw the same ability twice in a row.
//!
//! **This is not a cooldown**, and everything here is really one argument for
//! that claim — see `docs/design/combat-kernel.md`. A cooldown asks *did I have
//! it available*, and it can ask that because it takes an ability away from you
//! while you are busy with something else. This only ever holds the ability you
//! have this moment thrown, so the answer is always "yes, everything else in
//! the kit". What it charges for is repetition, not access.
//!
//! Four properties, and the last two are the awkward ones:
//!
//! 1. The same move, thrown twice in a row, is refused the second time.
//! 2. The rest of the kit is untouched while it is refused.
//! 3. A move you are still committed to never notices the rule at all.
//! 4. An ability with a second activation is not *used* until that activation
//!    is spent, so the press that spends it gets through and the lockout does
//!    not start counting until it has.

use sim::class::{Ghost, Mechanic};
use sim::state::{Action, SLOT_MECHANIC, SLOT_POKE, SLOT_SPECIAL};
use sim::tuning as t;
use sim::{Class, Fx, Input, V3, World};

const L: u16 = Input::LEFT;
const R: u16 = Input::RIGHT;
const Q: u16 = Input::SPECIAL;
const SHIFT: u16 = Input::SHIFT;

fn run(w: &mut World, frames: u32, bits: u16) {
    for _ in 0..frames {
        w.advance([Input::new(bits), Input::default()]);
    }
}

/// Which move, if any, player one started on this exact frame.
fn started(w: &World) -> Option<u8> {
    match w.players[0].action {
        Action::Startup { kind, left }
            if left == sim::moves::get(w.players[0].class, kind).startup =>
        {
            Some(kind)
        }
        _ => None,
    }
}

/// Hold `bits` down and report how many frames pass before `kind` comes out
/// again, counting from the frame it was last thrown.
///
/// The button is held rather than tapped because that is the spammer's input:
/// the question the lockout answers is what happens to somebody leaning on one
/// key, and a held click re-throws the moment the game lets it.
fn frames_until_repeat(w: &mut World, bits: u16, kind: u8, give_up: u32) -> Option<u32> {
    for frame in 1..=give_up {
        w.advance([Input::new(bits), Input::default()]);
        if started(w) == Some(kind) {
            return Some(frame);
        }
    }
    None
}

/// Let frames pass with nothing held until player one can act again.
///
/// By the action rather than by counting the move's frames: startup, active and
/// recovery each run one frame past the number in the table -- the countdown
/// spends the last one leaving the phase -- and a test that re-derived that
/// arithmetic would be measuring its own subtraction rather than the rule.
fn run_until_free(w: &mut World, give_up: u32) -> u32 {
    for frame in 1..=give_up {
        w.advance([Input::default(); 2]);
        if w.players[0].action.actionable() {
            return frame;
        }
    }
    panic!("player one never became actionable");
}

fn duel(class: Class) -> World {
    let mut w = World::with_classes([class, Class::Bulwark]);
    w.players[0].pos = V3::new(Fx::from_int(-6), Fx::ZERO, Fx::ZERO);
    w.players[0].facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    w.players[1].pos = V3::new(Fx::from_int(6), Fx::ZERO, Fx::ZERO);
    // Let anything the class starts with settle before a frame is counted.
    run(&mut w, 30, 0);
    w
}

// ---------------------------------------------------------------------------
// The rule
// ---------------------------------------------------------------------------

#[test]
fn the_same_move_held_down_comes_out_on_the_lockout_and_not_before() {
    // The whole rule in one measurement. Lean on left click and the auto comes
    // out every `repeat_lock` frames rather than every `whiff_cost` frames,
    // which is the difference between a spam button and a rhythm.
    //
    // **Not the Champion**, and not because the rule is different there: on
    // that class a held button does not repeat a move at all, it walks the
    // three hits of a chain, so the thing this measures does not happen. The
    // rule still applies to each of the three slots, which is what
    // `a_chain_never_brings_a_locked_move_back_early` below checks instead.
    for class in [Class::Bulwark, Class::Elementalist] {
        let mut w = duel(class);
        let m = sim::moves::get(class, SLOT_POKE);
        run(&mut w, 1, L);
        assert_eq!(started(&w), Some(SLOT_POKE), "{}: the first press", m.name);

        let again = frames_until_repeat(&mut w, L, SLOT_POKE, 300)
            .unwrap_or_else(|| panic!("{}: never came out a second time", m.name));
        assert_eq!(
            again,
            m.repeat_lock() as u32,
            "{} {}: came back after {again} frames, not the {} its lockout asks for. \
             The move itself costs {} frames, so the rule is only visible at all \
             because that is the shorter of the two.",
            class.name(),
            m.name,
            m.repeat_lock(),
            m.whiff_cost(),
        );
    }
}

#[test]
fn a_chain_never_brings_a_locked_move_back_early() {
    // The Champion's version of the measurement above. Leaning on one button
    // there walks a three-hit string rather than repeating one move, so what
    // has to hold is the rule stated per slot: however the chain steps between
    // them, no single move comes back inside its own lockout.
    //
    // It is worth a test of its own rather than a widened one because a chain
    // is exactly the shape of thing that could launder a lockout -- three slots
    // taking turns is a way to keep swinging, and the question is whether any
    // one of them is being thrown more often than the rule allows.
    let mut w = duel(Class::Champion);
    let mut last: [Option<u32>; sim::moves::MAX_SLOTS] = [None; sim::moves::MAX_SLOTS];
    for frame in 0..400u32 {
        w.advance([Input::new(L), Input::default()]);
        let Some(kind) = started(&w) else { continue };
        let m = sim::moves::get(Class::Champion, kind);
        if let Some(before) = last[kind as usize] {
            assert!(
                frame - before >= m.repeat_lock() as u32,
                "{} came back {} frames after itself, inside its {}-frame lockout",
                m.name,
                frame - before,
                m.repeat_lock()
            );
        }
        last[kind as usize] = Some(frame);
    }
    assert!(
        last.iter().filter(|seen| seen.is_some()).count() >= 3,
        "holding left click on the Champion threw fewer than three different moves, \
         so the chain is not walking"
    );
}

#[test]
fn the_rest_of_the_kit_is_untouched_while_one_move_is_locked() {
    // **The property the whole feature exists for.** Being unable to throw the
    // auto again has to leave every other answer on the table, or the rule has
    // stopped charging for repetition and started gating access -- which is a
    // cooldown, and is the thing the combat kernel rules out.
    let mut w = duel(Class::Bulwark);
    run(&mut w, 1, L);
    assert_eq!(started(&w), Some(SLOT_POKE));

    // Wait out the move but not the lockout.
    let bash = sim::moves::get(Class::Bulwark, SLOT_POKE);
    assert!(
        bash.repeat_idle() > 0,
        "this test needs a move whose lockout outlasts it, and Bash no longer is one"
    );
    run_until_free(&mut w, 300);
    assert!(
        w.players[0].locked_out(SLOT_POKE),
        "Bash should still be locked the frame it finishes"
    );

    // The committed version, on the same click with shift, comes out anyway.
    run(&mut w, 1, L | SHIFT);
    assert_eq!(
        started(&w),
        Some(sim::state::SLOT_COMMITTED),
        "the heavy was refused while the poke was locked, which makes this a cooldown"
    );
}

#[test]
fn a_lockout_is_spent_only_by_frames_that_pass() {
    // It runs down while she is busy, stunned, walking or standing still --
    // there is nothing to do to make it come back sooner and nothing that
    // stops it. A clock that only ran while idle would reward doing nothing.
    let mut w = duel(Class::Bulwark);
    run(&mut w, 1, L);
    let after_press = w.players[0].repeat_lock[SLOT_POKE as usize];
    assert_eq!(
        after_press,
        sim::moves::get(Class::Bulwark, SLOT_POKE).repeat_lock()
    );

    run(&mut w, 10, 0);
    assert_eq!(
        w.players[0].repeat_lock[SLOT_POKE as usize],
        after_press - 10,
        "ten frames should cost ten frames of lockout"
    );
}

#[test]
fn the_lockout_only_taxes_a_move_that_is_cheaper_than_it_is() {
    // The calibration, and the reason 30 is defensible rather than arbitrary.
    //
    // The clock starts when the move comes out, not when its recovery ends, so
    // a move that already commits you for longer than its lockout comes out of
    // recovery with the lockout gone. Every committed heavy in the game is in
    // that group. What is left is the autos and the fast pokes -- which is
    // precisely the set anybody would spam.
    let mut taxed = 0;
    let mut free = 0;
    for class in sim::class::ALL_CLASSES {
        for m in sim::moves::table(class) {
            if m.repeat_idle() > 0 {
                taxed += 1;
                assert!(
                    m.whiff_cost() < m.repeat_lock(),
                    "{} {}: taxed without being cheap",
                    class.name(),
                    m.name
                );
            } else {
                free += 1;
            }
        }
    }
    assert!(
        taxed > 0 && free > 0,
        "the rule should bite on some moves and be invisible on others: \
         {taxed} taxed, {free} untouched. Both at once means the lockout has been \
         tuned into either a no-op or a flat tax on everything."
    );
}

// ---------------------------------------------------------------------------
// Abilities that are used more than once
// ---------------------------------------------------------------------------

#[test]
fn sending_the_shadow_does_not_lock_the_press_that_brings_it_home() {
    // **The trap this rule walks into.** Send shadow is one button with two
    // meanings -- out, and home -- so a lockout armed by the send is a lockout
    // on the recall. The press that brings the second body back is the rest of
    // the ability, not another use of it.
    let mut w = duel(Class::ShadowReaver);
    run(&mut w, 4, R);
    run(&mut w, 20, 0);
    assert!(
        matches!(
            w.players[0].mechanic,
            Mechanic::Shadow(s) if s.is_out()
        ),
        "the shadow should be out after a right click"
    );

    // Well inside the lockout the send armed.
    assert!(w.players[0].locked_out(SLOT_MECHANIC));
    run(&mut w, 4, R);
    run(&mut w, 8, 0);
    assert!(
        matches!(
            w.players[0].mechanic,
            Mechanic::Shadow(s) if matches!(s.doing, Ghost::Returning { .. } | Ghost::Attending)
        ),
        "the recall was eaten by the lockout the send armed"
    );
}

#[test]
fn the_shadow_s_lockout_starts_when_it_is_home_and_not_when_it_leaves() {
    // The other half of the same rule. An ability is not *used* until it is
    // spent, so leaving the shadow standing out on the field does not quietly
    // serve its lockout while it waits -- otherwise parking it somewhere and
    // ignoring it would be the way to pay for the next send.
    let mut w = duel(Class::ShadowReaver);
    run(&mut w, 4, R);
    let sent = w.players[0].repeat_lock[SLOT_MECHANIC as usize];
    assert!(sent > 0);

    // Sixty frames of the shadow simply standing there.
    run(&mut w, 60, 0);
    assert!(
        matches!(w.players[0].mechanic, Mechanic::Shadow(s) if s.is_out()),
        "the shadow should still be out with nothing recalling it"
    );
    assert_eq!(
        w.players[0].repeat_lock[SLOT_MECHANIC as usize], sent,
        "the lockout ran down while the ability was still out there"
    );
}

#[test]
fn a_second_lotus_waits_for_the_first_one_to_come_home() {
    // The lotus has a second activation too, but it is on *right click* -- a
    // recall drags the hanging blades home. Its own key does not reactivate it,
    // so pressing `Q` again while the blades are still out would be a fresh
    // lotus rather than the rest of this one, and the ability is not spent yet.
    let mut w = duel(Class::ShadowReaver);
    run(&mut w, 4, Q);
    assert!(w.players[0].locked_out(SLOT_SPECIAL));

    let lotus_life = t::lotus_erupt() + t::lotus_hold() + t::lotus_return();
    let again = frames_until_repeat(&mut w, Q, SLOT_SPECIAL, 600)
        .expect("the lotus never came back at all");
    assert!(
        again > lotus_life as u32,
        "a second lotus came out after {again} frames, inside the {lotus_life} the first \
         one lives for. The ability was not finished being used."
    );
}
