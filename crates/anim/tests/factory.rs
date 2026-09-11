//! Tests for the animation factory itself -- springs, eases, the solver.
//!
//! These check that the solver produces *motion with the properties that read
//! as natural* -- overshoot, lag, settle -- rather than checking exact values.
//! A spring that never overshoots is a lerp with extra steps, and the whole
//! reason the factory exists is to get the things a lerp cannot give you.

use anim::bake::{Feel, Key, Looseness, Recipe, bake};
use anim::ease::Ease;
use anim::spring::Spring;
use anim::{DT, SUBSTEPS};
use view::clips::Clip;
use view::pose::Pose;
use view::skeleton::Joint;

// ---------------------------------------------------------------------------
// Springs
// ---------------------------------------------------------------------------

#[test]
fn a_critically_damped_spring_arrives_without_overshooting() {
    let mut s = Spring::tight(0.0, 20.0);
    let mut peak: f32 = 0.0;
    for _ in 0..240 {
        s.step(1.0, DT / 8.0);
        peak = peak.max(s.value);
    }
    assert!(s.settled(1.0, 0.01), "never arrived: {}", s.value);
    assert!(peak <= 1.02, "critically damped spring overshot to {peak}");
}

#[test]
fn a_loose_spring_overshoots_and_settles() {
    // Overshoot is not a defect here. It is the entire reason to use a spring.
    let mut s = Spring::loose(0.0, 20.0);
    let mut peak: f32 = 0.0;
    for _ in 0..480 {
        s.step(1.0, DT / 8.0);
        peak = peak.max(s.value);
    }
    assert!(peak > 1.03, "loose spring did not overshoot (peak {peak})");
    assert!(
        s.settled(1.0, 0.02),
        "loose spring never settled: {}",
        s.value
    );
}

#[test]
fn stiff_springs_stay_stable_at_the_substep_rate() {
    // Stiff enough to look snappy is stiff enough to explode under explicit
    // integration. If this ever fails, raise SUBSTEPS rather than softening
    // the animation.
    let mut s = Spring::tight(0.0, 60.0);
    for _ in 0..600 {
        s.step(1.0, DT / SUBSTEPS as f32);
        assert!(
            s.value.is_finite() && s.value.abs() < 10.0,
            "diverged to {}",
            s.value
        );
    }
}

// ---------------------------------------------------------------------------
// The lag budget
// ---------------------------------------------------------------------------

/// How many frames behind a moving target a part actually runs.
///
/// Measured, not derived: drive the spring with a ramp and find how far back
/// along that ramp its output sits once the transient has died.
fn measured_lag(f: Feel) -> f32 {
    let (freq, damp) = (2.0 * f.ring / (f.lag / 60.0), f.ring);
    let mut s = Spring::new(0.0, freq, damp);
    let dt = DT / SUBSTEPS as f32;
    let mut t = 0.0f32;
    for _ in 0..(120 * SUBSTEPS) {
        t += dt;
        s.step(t, dt); // target ramps at one unit per second
    }
    (t - s.value) * 60.0
}

#[test]
fn the_lag_knob_means_what_it_says() {
    // The whole reason `Feel` is expressed in frames rather than in spring
    // frequency is so this assertion can exist.
    for f in [
        Feel::new(1.2, 1.0),
        Feel::new(2.4, 0.9),
        Feel::new(4.0, 0.45),
    ] {
        let got = measured_lag(f);
        assert!(
            (got - f.lag).abs() < 0.25,
            "asked for {} frames of lag, measured {got}",
            f.lag
        );
    }
}

#[test]
fn no_preset_is_late_enough_to_be_unreadable() {
    // The bug this pins: the first version made "heavy" mean a low frequency,
    // so a heavy clip had barely started moving a third of the way through its
    // own startup. An opponent cannot react to a windup they cannot see.
    // Weight must read as follow-through, never as delay.
    for (name, l) in Looseness::ALL {
        for (part, f) in [
            ("root", l.root),
            ("spine", l.spine),
            ("chest", l.chest),
            ("head", l.head),
            ("arms", l.arms),
            ("legs", l.legs),
        ] {
            // A hand is the slowest thing on the body: two joints further out
            // than the shoulder, so half again as much lag each step.
            let worst = f.lag * 2.0;
            assert!(
                worst <= 9.5,
                "{name} {part} reaches {worst} frames of lag at the fingertips"
            );
        }
    }
}

#[test]
fn every_preset_is_stable_at_the_substep_rate() {
    for (name, l) in Looseness::ALL {
        for f in [l.root, l.spine, l.chest, l.head, l.arms, l.legs] {
            let (freq, damp) = (2.0 * f.ring / (f.lag / 60.0), f.ring);
            let mut s = Spring::new(0.0, freq, damp);
            // Three seconds. The limp preset rings for a long time on purpose;
            // what is being checked here is that it rings *down* rather than up.
            for _ in 0..(180 * SUBSTEPS) {
                s.step(1.0, DT / SUBSTEPS as f32);
                assert!(
                    s.value.is_finite() && s.value.abs() < 4.0,
                    "{name}: diverged"
                );
            }
            assert!(s.settled(1.0, 0.02), "{name}: never settled: {}", s.value);
        }
    }
}

// ---------------------------------------------------------------------------
// Eases
// ---------------------------------------------------------------------------

#[test]
fn every_ease_starts_at_zero_and_finishes_at_one() {
    for (name, e) in Ease::PRESETS {
        assert!(e.at(0.0).abs() < 1e-3, "{name} does not start at zero");
        assert!(
            (e.at(1.0) - 1.0).abs() < 1e-3,
            "{name} does not finish at one"
        );
    }
}

#[test]
fn a_snap_ease_holds_and_then_moves() {
    // The telegraph: the opponent gets a long look at a pose that is not
    // moving, and then it moves. If this stops being true, every heavy move in
    // the game quietly becomes less readable.
    assert!(
        Ease::SNAP.at(0.5) < 0.2,
        "snap had already travelled {} by halfway",
        Ease::SNAP.at(0.5)
    );
    assert!(Ease::SNAP.at(0.9) < 0.75);
}

#[test]
fn anticipation_goes_the_wrong_way_first() {
    // A handle below zero is not a mistake; it is the cheapest anticipation
    // there is, and it costs no extra key.
    assert!(
        Ease::ANTICIPATE.at(0.25) < -0.02,
        "anticipate never pulled back: {}",
        Ease::ANTICIPATE.at(0.25)
    );
}

#[test]
fn overshoot_goes_past_and_comes_back() {
    let peak = (0..100)
        .map(|i| Ease::OVERSHOOT.at(i as f32 / 99.0))
        .fold(0.0f32, f32::max);
    assert!(peak > 1.05, "overshoot peaked at {peak}");
}

#[test]
fn presets_can_be_named_again() {
    // The hub shows the preset's name when the handles match one, and writes
    // `Ease::SNAP` rather than four magic numbers when it saves.
    for (name, e) in Ease::PRESETS {
        assert_eq!(e.preset_name(), Some(*name));
    }
    assert_eq!(Ease::new(0.11, 0.22, 0.33, 0.44).preset_name(), None);
}

// ---------------------------------------------------------------------------
// Baking
// ---------------------------------------------------------------------------

fn simple(clip: Clip, looseness: Looseness) -> Recipe {
    let up = Pose::rest().shoulders(90.0, 10.0, 0.0);
    Recipe {
        clip,
        looseness,
        notes: String::new(),
        keys: vec![
            Key::at(0, Pose::rest()),
            Key::at(8, up),
            Key::at(clip.length() - 1, Pose::rest()),
        ],
    }
}

#[test]
fn baking_produces_one_pose_per_frame() {
    let b = bake(&simple(Clip::HitHeavy, Looseness::MARTIAL));
    assert_eq!(b.frames.len(), Clip::HitHeavy.length() as usize);
}

#[test]
fn baking_starts_settled_on_the_first_key() {
    // Otherwise every animation opens with a lurch from an arbitrary rest pose.
    let b = bake(&simple(Clip::HitHeavy, Looseness::MARTIAL));
    let first = b.frames[0];
    assert!(
        first.separation(&Pose::rest()) < 0.02,
        "opened somewhere other than its first key"
    );
}

#[test]
fn looser_baking_lags_further() {
    // The looseness setting has to actually do something, or it is a knob that
    // lies to whoever turns it.
    let crisp = bake(&simple(Clip::HitHeavy, Looseness::CRISP));
    let heavy = bake(&simple(Clip::HitHeavy, Looseness::HEAVY));
    let at = |b: &anim::Baked, f: usize| b.frames[f].degrees(Joint::ArmR, 0);
    assert!(
        at(&heavy, 8) < at(&crisp, 8) - 4.0,
        "heavy arm at {} was not behind the crisp one at {}",
        at(&heavy, 8),
        at(&crisp, 8)
    );
}

#[test]
fn a_looping_clip_closes_on_itself() {
    // The pre-roll is what makes this true. Without it the springs are still
    // settling when the recording starts, and the seam is visible every cycle.
    let recipe = Recipe {
        clip: Clip::Idle,
        looseness: Looseness::STRIDE,
        notes: String::new(),
        keys: vec![
            Key::at(0, Pose::rest()),
            Key::at(40, Pose::rest().spine(20.0, 0.0, 0.0)),
            Key::at(80, Pose::rest().spine(-10.0, 0.0, 0.0)),
        ],
    };
    let b = bake(&recipe);
    let seam = b.frames[b.frames.len() - 1].separation(&b.frames[0]);
    let typical = b.frames[10].separation(&b.frames[11]);
    assert!(
        seam < typical * 3.0 + 0.02,
        "the loop jumps {seam} at the seam against {typical} between ordinary frames"
    );
}

#[test]
fn the_solver_refuses_to_break_a_joint() {
    // Springs are allowed to overshoot; knees are not allowed to invert. The
    // clamp at the end of the solve is what lets both be true.
    let recipe = Recipe {
        clip: Clip::HitHeavy,
        looseness: Looseness::LIMP,
        notes: String::new(),
        keys: vec![
            Key::eased(0, Pose::rest().knees(140.0), Ease::STRIKE),
            Key::eased(6, Pose::rest().knees(0.0), Ease::STRIKE),
            Key::at(25, Pose::rest().knees(0.0)),
        ],
    };
    let skeleton = view::pose::reference();
    for (i, pose) in bake(&recipe).frames.iter().enumerate() {
        assert!(
            pose.violations(skeleton).is_empty(),
            "frame {i} broke {:?}",
            pose.violations(skeleton)
        );
    }
}
