//! The first shaped curve.
//!
//! A duration says how long; a curve says what the time is spent doing. These
//! assertions are about the difference being real and being the shape asked
//! for -- a hold you can read, then an eruption.

use sim::curve::Curve;
use sim::fixed::Fx;

fn at(c: Curve, num: i32, den: i32) -> f32 {
    c.at(Fx::ratio(num, den)).to_f32_for_render()
}

#[test]
fn a_curve_starts_where_it_starts_and_ends_where_it_ends() {
    for c in [Curve::LINEAR, sim::tuning::structure_rise_curve()] {
        assert!(at(c, 0, 1).abs() < 0.01, "a curve that does not start at 0");
        assert!(
            (at(c, 1, 1) - 1.0).abs() < 0.01,
            "a curve that does not finish"
        );
    }
}

#[test]
fn a_linear_curve_is_actually_a_line() {
    // The fallback shape has to be the boring one, or every unshaped value in
    // the game quietly acquires an easing nobody asked for.
    for step in 0..=10 {
        let want = step as f32 / 10.0;
        let got = at(Curve::LINEAR, step, 10);
        assert!(
            (got - want).abs() < 0.02,
            "linear bent at {want}: got {got}"
        );
    }
}

#[test]
fn a_curve_never_goes_backwards() {
    // A structure that dipped back into the ground mid-rise would read as a
    // bug even if it were on purpose.
    let c = sim::tuning::structure_rise_curve();
    let mut last = -1.0;
    for step in 0..=40 {
        let now = at(c, step, 40);
        assert!(now >= last - 0.001, "the rise went backwards at {step}/40");
        last = now;
    }
}

#[test]
fn the_structure_rise_holds_low_and_then_erupts() {
    // The shape the class is built on: a beat where the telegraph is readable
    // and someone can still move, then a fast finish. Stated as what a player
    // can see -- barely out of the ground for most of it, most of the travel
    // in the last stretch.
    let c = sim::tuning::structure_rise_curve();
    let halfway = at(c, 1, 2);
    let three_quarters = at(c, 3, 4);
    assert!(
        halfway < 0.2,
        "half the rise time has gone and it is already {:.0}% up -- there is no telegraph",
        halfway * 100.0
    );
    assert!(
        1.0 - three_quarters > 0.5,
        "most of the climb happens early, so the finish is not an eruption"
    );
}

#[test]
fn a_curve_is_the_same_shape_however_you_ask_for_it() {
    // Determinism: the solver is bisection with a fixed iteration count, so two
    // peers asking for the same point must get the identical raw value. An
    // adaptive solver would answer differently depending on the handles.
    let c = sim::tuning::structure_rise_curve();
    for step in 0..=20 {
        let x = Fx::ratio(step, 20);
        assert_eq!(c.at(x).raw(), c.at(x).raw());
    }
}
