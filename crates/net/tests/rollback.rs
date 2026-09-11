//! Rollback correctness against ground truth.

use net::LocalSession;
use sim::state::MAX_PLAYERS;
use sim::{Input, World};

fn script(frames: u32, seed: u64) -> Vec<[Input; MAX_PLAYERS]> {
    let mut rng = seed;
    let mut next = move || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    (0..frames)
        .map(|_| {
            [
                Input::aimed((next() & 0x1ff) as u16, next() as u16),
                Input::aimed((next() & 0x1ff) as u16, next() as u16),
            ]
        })
        .collect()
}

fn ground_truth(s: &[[Input; MAX_PLAYERS]]) -> u64 {
    let mut w = World::new();
    for i in s {
        w.advance(*i);
    }
    w.checksum()
}

#[test]
fn mispredictions_converge_on_ground_truth() {
    let s = script(1800, 0xdead_beef);
    let expected = ground_truth(&s);

    let mut session = LocalSession::new(World::new());
    let mut i = 0;
    while i < s.len() {
        let batch = (s.len() - i).min(4);
        for k in 0..batch {
            let mut guess = s[i + k];
            guess[1] = Input::default(); // always mispredict the remote player
            session.predict_and_advance(guess);
        }
        session.confirm(&s[i..i + batch]);
        i += batch;
    }

    assert!(
        session.rollbacks > 0,
        "test did not exercise the rollback path"
    );
    assert_eq!(session.sim().checksum(), expected);
}

#[test]
fn correct_predictions_do_not_roll_back() {
    let s = script(600, 4242);
    let mut session = LocalSession::new(World::new());
    let mut i = 0;
    while i < s.len() {
        let batch = (s.len() - i).min(4);
        for k in 0..batch {
            session.predict_and_advance(s[i + k]);
        }
        let rolled = session.confirm(&s[i..i + batch]);
        assert!(!rolled, "rolled back despite a correct prediction");
        i += batch;
    }
    assert_eq!(session.sim().checksum(), ground_truth(&s));
    assert_eq!(session.rollbacks, 0);
}
