//! The tests that matter most.
//!
//! Rollback is only correct if the simulation is a pure function. These are
//! cheap to run and catch the class of bug that is otherwise diagnosed by two
//! people on different continents noticing their games disagree.

use sim::state::MAX_PLAYERS;
use sim::{Fx, Input, World};

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
                Input((next() & 0x1ff) as u16),
                Input((next() & 0x1ff) as u16),
            ]
        })
        .collect()
}

fn run(s: &[[Input; MAX_PLAYERS]]) -> World {
    let mut w = World::new();
    for i in s {
        w.advance(*i);
    }
    w
}

#[test]
fn identical_inputs_produce_identical_state() {
    let s = script(1200, 0x1234_5678_9abc_def0);
    assert_eq!(run(&s).checksum(), run(&s).checksum());
    assert_eq!(run(&s), run(&s));
}

#[test]
fn different_inputs_produce_different_state() {
    // Guards against a checksum that ignores the fields it should cover.
    let a = run(&script(600, 1));
    let b = run(&script(600, 2));
    assert_ne!(a.checksum(), b.checksum());
}

#[test]
fn resimulating_from_a_snapshot_matches() {
    // The core rollback property: restore an old state, replay the same
    // inputs, arrive at exactly the same place.
    let s = script(600, 99);
    let mut w = World::new();
    for i in &s[..300] {
        w.advance(*i);
    }
    let snapshot = w.clone();
    for i in &s[300..] {
        w.advance(*i);
    }
    let ground_truth = w.checksum();

    let mut replayed = snapshot;
    for i in &s[300..] {
        replayed.advance(*i);
    }
    assert_eq!(replayed.checksum(), ground_truth);
}

#[test]
fn state_is_frame_indexed() {
    let s = script(100, 7);
    let w = run(&s);
    assert_eq!(w.frame, 100);
}

// ---------------------------------------------------------------------------
// Fixed point
// ---------------------------------------------------------------------------

#[test]
fn fixed_point_arithmetic() {
    let two = Fx::from_int(2);
    let three = Fx::from_int(3);
    assert_eq!(two.add(three), Fx::from_int(5));
    assert_eq!(three.sub(two), Fx::ONE);
    assert_eq!(two.mul(three), Fx::from_int(6));
    assert_eq!(Fx::from_int(6).div(three), two);
    assert_eq!(Fx::ratio(1, 2).mul(Fx::from_int(8)), Fx::from_int(4));
}

#[test]
fn division_by_zero_saturates_rather_than_panicking() {
    // A panic during rollback re-simulation is worse than a wrong number.
    assert_eq!(Fx::ONE.div(Fx::ZERO), Fx::MAX);
    assert_eq!(Fx::ONE.neg().div(Fx::ZERO), Fx::MIN);
}

#[test]
fn sqrt_is_accurate_enough() {
    for n in [1i32, 2, 4, 9, 16, 100, 1000] {
        let r = Fx::from_int(n).sqrt();
        let sq = r.mul(r);
        let err = sq.sub(Fx::from_int(n)).abs();
        assert!(err.raw() < 64, "sqrt({n}) = {r:?}, squares to {sq:?}");
    }
}

#[test]
fn trig_hits_the_cardinal_points() {
    let tol = 200; // raw 16.16 units, about 0.003
    let checks = [
        (Fx::ZERO, 0),
        (Fx::ratio(1, 4), 65536),
        (Fx::ratio(1, 2), 0),
        (Fx::ratio(3, 4), -65536),
    ];
    for (turns, expected) in checks {
        let got = sim::fixed::sin_turns(turns).raw();
        assert!(
            (got - expected).abs() < tol,
            "sin({turns:?}) = {got}, want {expected}"
        );
    }
}

#[test]
fn trig_identity_holds() {
    // sin^2 + cos^2 == 1, within fixed-point tolerance.
    let mut t = Fx::ZERO;
    while t.raw() < Fx::ONE.raw() {
        let s = sim::fixed::sin_turns(t);
        let c = sim::fixed::cos_turns(t);
        let sum = s.mul(s).add(c.mul(c));
        let err = sum.sub(Fx::ONE).abs();
        assert!(err.raw() < 400, "at {t:?}: sin^2+cos^2 = {sum:?}");
        t = t.add(Fx::ratio(1, 64));
    }
}
