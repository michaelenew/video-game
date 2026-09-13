//! Guard test: a rollback fits inside the frame it happens on.
//!
//! This is the relationship the simulation's own budget is derived from, tested
//! from the other end. `sim/tests/budget.rs` says one `advance` must cost under
//! a thirty-second of a frame; the reason it says that is this test, which asks
//! the question a player would: when a prediction turns out to be wrong, does
//! the correction land inside the same 16.7 ms as everything else?
//!
//! A rollback is not one frame of work. It is a restore, then a re-simulation
//! of every frame since the peer and I stopped agreeing -- up to
//! [`MAX_ROLLBACK_FRAMES`] of them -- and a fresh snapshot for each, all inside
//! one rendered frame that still has to draw a picture at the end of it. That
//! is the worst moment in the game's whole loop, and it is the moment the
//! player is least able to absorb a hitch, because a rollback already means the
//! network is having a bad time.
//!
//! So the thing measured here is the *burst*, not the average: the single
//! slowest `confirm` in a match full of mispredictions. An average would hide
//! exactly the spike this exists to catch.

use net::{LocalSession, MAX_ROLLBACK_FRAMES};
use sim::class::Class;
use sim::state::MAX_PLAYERS;
use sim::{Input, World};

use std::time::Instant;

/// How long one frame lasts at 60 Hz, in nanoseconds.
const FRAME_NS: u128 = 1_000_000_000 / sim::TICK_HZ as u128;

/// What the worst rollback in a match may cost.
///
/// A quarter of the frame for the whole burst, leaving three quarters for the
/// renderer -- which is the same division `sim`'s per-advance budget is cut
/// from, stated once here and divided there. Against a measured worst burst of
/// around 0.8 ms for the heaviest class in a hunt, that is a margin of roughly
/// five times: the narrowest of the three budgets, because this one is a
/// *maximum* rather than a mean and so carries the machine's noise as well as
/// the game's work. [`REPEATS`] is what keeps it honest.
const BURST_BUDGET_NS: u128 = FRAME_NS / 4;

/// Frames of match per run. Long enough for both fighters to be mid-move with
/// things on the field when the mispredictions land.
const FRAMES: usize = 900;

/// Runs per scenario; the best one wins, because noise only ever adds.
///
/// Five rather than the three the other budget tests use. Each run reports the
/// slowest of its hundred-odd rollbacks, which is the statistic a scheduler
/// preemption is most likely to land in, so this one needs more chances at a
/// clean measurement than a mean does.
const REPEATS: usize = 5;

/// The worst rollback in a match of constant mispredictions still fits in a
/// quarter of the frame it lands on.
///
/// The Shadow Reaver in a hunt because that is the heaviest the simulation gets
/// -- a second body, a creature, and a kit that leaves things on the field --
/// so it is the case that will breach this first.
#[test]
fn the_worst_rollback_fits_inside_one_frame() {
    let script = input_script(FRAMES);
    let mut worst: Option<(String, u128)> = None;

    for class in [Class::Bulwark, Class::ShadowReaver] {
        for (scenario, with_beast) in [("versus", false), ("hunt", true)] {
            // The slowest burst of each run, then the best of those runs: the
            // first picks out the spike, the second throws away the machine.
            let mut best_of_runs = u128::MAX;
            let mut rollbacks = 0;
            for _ in 0..REPEATS {
                let (slowest, seen) = worst_burst(class, with_beast, &script);
                best_of_runs = best_of_runs.min(slowest);
                rollbacks = seen;
            }
            assert!(
                rollbacks > 0,
                "{class:?} in a {scenario} never rolled back, so this measured nothing"
            );
            if worst.as_ref().is_none_or(|(_, ns)| best_of_runs > *ns) {
                worst = Some((format!("{class:?} in a {scenario}"), best_of_runs));
            }
        }
    }

    let (where_, ns) = worst.expect("there is at least one scenario to measure");
    assert!(
        ns <= BURST_BUDGET_NS,
        "the worst rollback costs {ns} ns ({where_}), over the {BURST_BUDGET_NS} ns budget -- \
         a quarter of a {FRAME_NS} ns frame.\n\
         A rollback restores a snapshot and re-simulates up to {MAX_ROLLBACK_FRAMES} frames \
         inside one rendered frame, so this is a visible hitch at the exact moment the \
         connection is already struggling."
    );
}

/// Play the script through a real session, mispredicting the peer constantly,
/// and report the slowest single `confirm` -- which is the slowest rollback --
/// along with how many rollbacks happened at all.
fn worst_burst(class: Class, with_beast: bool, script: &[[Input; MAX_PLAYERS]]) -> (u128, u32) {
    let world = if with_beast {
        World::hunt([class; MAX_PLAYERS])
    } else {
        World::with_classes([class; MAX_PLAYERS])
    };
    let mut session = LocalSession::new(world);
    let mut slowest = 0;

    let mut i = 0;
    while i < script.len() {
        // A full window of predictions before every correction, so each
        // rollback re-simulates as far as the session will ever let it.
        let batch = (script.len() - i).min(MAX_ROLLBACK_FRAMES);
        for k in 0..batch {
            let mut guess = script[i + k];
            // Wrong every other frame, so the correction is never a no-op.
            if (i + k) % 2 == 0 {
                guess[1] = Input::default();
            }
            session.predict_and_advance(guess);
        }

        let started = Instant::now();
        let rolled = session.confirm(&script[i..i + batch]);
        let took = started.elapsed().as_nanos();
        if rolled {
            slowest = slowest.max(took);
        }
        i += batch;
    }

    (slowest, session.rollbacks)
}

/// Deterministic inputs, seeded rather than clocked. The same generator the
/// soak and the other budget tests use.
fn input_script(frames: usize) -> Vec<[Input; MAX_PLAYERS]> {
    let mut rng = 0x2545_f491_4f6c_dd1d_u64;
    let mut next = move || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    (0..frames)
        .map(|_| {
            std::array::from_fn(|_| {
                let r = next();
                Input {
                    bits: (r & 0x1ff) as u16,
                    aim: (r >> 16) as u16,
                    pitch: ((r >> 32) as i16) / 8,
                }
            })
        })
        .collect()
}
