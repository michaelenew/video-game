//! The lean into the going (`view::lean`): forward with a start, back with a
//! stop, into a cut, and never by moving a foot.

use view::clips::Clip;
use view::lean::{Lean, lean_over};
use view::skeleton::{Joint, solve};

const DT: f32 = 1.0 / 60.0;

/// The lean, frame by frame, of a body that stands, runs for a second and
/// stops dead -- the simulation's three-frame start and stop.
fn start_run_stop(travel: [f32; 2]) -> Vec<[f32; 2]> {
    let mut lean = Lean::default();
    let mut out = Vec::new();
    for i in 0..150 {
        let k = match i {
            0..=19 => 0.0,
            20..=22 => (i - 19) as f32 / 3.0,
            23..=82 => 1.0,
            83..=85 => (86 - i) as f32 / 3.0,
            _ => 0.0,
        };
        out.push(lean.step([travel[0] * k, travel[1] * k], true, DT));
    }
    out
}

#[test]
fn a_body_leans_into_a_start_carries_a_lean_and_sits_back_into_a_stop() {
    let run = view::play::RUN_AT;
    let leans = start_run_stop([0.0, run]);
    let most = leans[20..60].iter().map(|l| l[0]).fold(0.0, f32::max);
    let carried = leans[80][0];
    let stop = leans[83..120].iter().map(|l| l[0]).fold(0.0, f32::min);
    assert!(leans[19][0].abs() < 0.01, "standing still is upright");
    assert!(
        most > 10.0 && most < 20.0,
        "a start leans well forward: {most}"
    );
    assert!(
        carried > 2.0 && carried < 6.0,
        "a run is carried a little forward: {carried}"
    );
    assert!(stop < -5.0, "a stop sits back: {stop}");
    assert!(
        leans[149][0].abs() < 0.5,
        "and it settles: {}",
        leans[149][0]
    );
    // It arrives over a few frames rather than in one.
    for pair in leans.windows(2) {
        assert!((pair[1][0] - pair[0][0]).abs() < 4.0, "{pair:?}");
    }
}

#[test]
fn a_cut_to_the_side_leans_into_it() {
    let leans = start_run_stop([view::play::RUN_AT, 0.0]);
    let most = leans[20..60].iter().map(|l| l[1]).fold(0.0, f32::max);
    assert!(most > 5.0, "into a cut to the right: {most}");
}

#[test]
fn a_body_not_free_to_lean_does_not() {
    let mut lean = Lean::default();
    let mut l = [0.0; 2];
    for i in 0..40 {
        l = lean.step([0.0, if i > 5 { 7.0 } else { 0.0 }], false, DT);
    }
    assert!(l[0].abs() < 0.01, "{l:?}");
}

#[test]
fn leaning_tips_the_body_ahead_of_feet_that_stay_put() {
    let skeleton = view::pose::reference();
    for clip in [Clip::RunForward, Clip::WalkForward] {
        for f in 0..clip.length() {
            let pose = clip.at(f as u32);
            let before = solve(skeleton, &pose);
            for (forward, right) in [(14.0, 0.0), (-9.0, 0.0), (4.0, 9.0), (0.0, -9.0)] {
                let after = solve(skeleton, &lean_over(pose, forward, right));
                for foot in [Joint::FootL, Joint::FootR] {
                    let (a, b) = (before.origin[foot.index()], after.origin[foot.index()]);
                    let moved = (0..3).map(|i| (a[i] - b[i]).powi(2)).sum::<f32>().sqrt();
                    assert!(moved < 0.01, "{clip:?} frame {f}: a foot moved {moved} m");
                }
                let head = |s: &view::skeleton::Skin| s.origin[Joint::Head.index()];
                let ahead = head(&after)[2] - head(&before)[2];
                let aside = head(&after)[0] - head(&before)[0];
                assert!(
                    ahead * forward >= 0.0 && ahead.abs() >= forward.abs() * 0.01,
                    "{clip:?} {f}: {ahead}"
                );
                assert!(
                    aside * right >= 0.0 && aside.abs() >= right.abs() * 0.01,
                    "{clip:?} {f}: {aside}"
                );
            }
        }
    }
}
