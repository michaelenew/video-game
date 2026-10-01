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

/// The arena picker's request is on the wire, so a peer that only learns of it
/// in a rollback rebuilds the same fight on the same frame.
///
/// The remote player asks for the Ridgeback halfway through; this side always
/// predicts that it asked for nothing, so the trip is first seen while
/// re-simulating, and the frames after it are re-simulated in the new fight.
#[test]
fn a_trip_the_peer_asked_for_lands_on_the_same_frame_after_a_rollback() {
    use sim::input::Travel;
    use sim::species::SpeciesId;

    let mut s = script(600, 0x5eed);
    s[301][1] = s[301][1].travelling(Travel::hunt(SpeciesId::RIDGEBACK));
    s[450][1] = s[450][1].travelling(Travel::VERSUS);
    let mut truth = World::new();
    let mut hunting_from = None;
    for (frame, i) in s.iter().enumerate() {
        truth.advance(*i);
        if truth.hunting() && hunting_from.is_none() {
            hunting_from = Some(frame);
        }
    }
    assert_eq!(
        hunting_from,
        Some(301),
        "the trip did not land on its frame"
    );
    assert!(!truth.hunting(), "the second trip did not land");

    let mut session = LocalSession::new(World::new());
    let mut i = 0;
    while i < s.len() {
        let batch = (s.len() - i).min(4);
        for k in 0..batch {
            let mut guess = s[i + k];
            guess[1] = Input::default();
            session.predict_and_advance(guess);
        }
        session.confirm(&s[i..i + batch]);
        i += batch;
    }
    assert_eq!(session.sim().checksum(), truth.checksum());
}

/// The trip survives the trip: a `Travel` byte packed into the wire format
/// comes out the far side unchanged, and so does everything packed beside it.
#[test]
fn the_travel_byte_round_trips_through_the_wire_format() {
    use sim::input::Travel;
    let sent = Input::looking_at(0xbeef, 0x1234, -321).travelling(Travel(0x83));
    let received: Input = net::NetInput::from(sent).into();
    assert_eq!(received, sent);
    let plain = Input::looking_at(0xffff, 0xffff, -1);
    assert_eq!(Input::from(net::NetInput::from(plain)), plain);
}
