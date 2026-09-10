//! Headless soak.
//!
//! Drives the simulation with deterministic pseudo-random inputs and checks
//! that it replays identically and survives rollback. No graphics, no network.
//!
//! The Bevy app replaces this binary's front end later; this stays useful as a
//! CI determinism check.

use net::LocalSession;
use sim::state::MAX_PLAYERS;
use sim::{Input, World};

const FRAMES: u32 = 3600; // one minute at 60Hz -- a full versus match

fn main() {
    let script = input_script(FRAMES);

    let a = run(&script);
    let b = run(&script);

    println!("replay A checksum: {a:#018x}");
    println!("replay B checksum: {b:#018x}");
    assert_eq!(a, b, "SIMULATION IS NON-DETERMINISTIC");

    let (rolled, rollbacks) = run_with_rollbacks(&script);
    println!("rollback checksum: {rolled:#018x}  ({rollbacks} rollbacks)");
    assert_eq!(a, rolled, "ROLLBACK DIVERGED FROM GROUND TRUTH");

    println!("\nok: {FRAMES} frames, deterministic, rollback-consistent");
}

fn run(script: &[[Input; MAX_PLAYERS]]) -> u64 {
    let mut w = World::new();
    for inputs in script {
        w.advance(*inputs);
    }
    w.checksum()
}

/// Replays the script through the rollback session, mispredicting the remote
/// player every few frames so the rollback path is actually exercised.
fn run_with_rollbacks(script: &[[Input; MAX_PLAYERS]]) -> (u64, u32) {
    let mut session = LocalSession::new(World::new());
    let mut i = 0;
    while i < script.len() {
        let batch = (script.len() - i).min(4);
        for k in 0..batch {
            let mut guess = script[i + k];
            // Mispredict the remote input half the time.
            if (i + k) % 2 == 0 {
                guess[1] = Input::default();
            }
            session.predict_and_advance(guess);
        }
        session.confirm(&script[i..i + batch]);
        i += batch;
    }
    (session.sim().checksum(), session.rollbacks)
}

/// Deterministic input script. Seeded PRNG, never a clock.
fn input_script(frames: u32) -> Vec<[Input; MAX_PLAYERS]> {
    let mut rng = 0x2545_f491_4f6c_dd1d_u64;
    let mut next = move || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    (0..frames)
        .map(|_| {
            let mut mk = || Input((next() & 0x1ff) as u16);
            [mk(), mk()]
        })
        .collect()
}
