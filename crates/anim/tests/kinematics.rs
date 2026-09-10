//! Tests for the animation factory.
//!
//! These check that the solver produces *motion with the properties that read
//! as natural* — overshoot, lag, settle — rather than checking exact values.
//! A spring that never overshoots is a lerp with extra steps, and the whole
//! reason the factory exists is to get the things a lerp cannot give you.

use anim::bake::{Feel, Key, Looseness, Recipe, bake};
use anim::spring::Spring;
use anim::{Chain, DT};
use view::pose::{PARTS, Part, PartTransform, Pose};

fn flat(v: f32) -> Pose {
    Pose {
        parts: [PartTransform {
            pos: [0.0, v, 0.0],
            rot: [0.0, 0.0, 0.0],
        }; 6],
    }
}

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
        s.step(1.0, DT / anim::SUBSTEPS as f32);
        assert!(
            s.value.is_finite() && s.value.abs() < 10.0,
            "diverged to {}",
            s.value
        );
    }
}

// ---------------------------------------------------------------------------
// Chains
// ---------------------------------------------------------------------------

#[test]
fn a_chain_tip_lags_behind_its_root() {
    // Follow-through: the shoulder leads and the hand trails. Without this the
    // whole body arrives at once, which is what makes hand-keyed motion read
    // as a slideshow.
    let mut chain = Chain::tapered(4, 25.0);
    chain.set(0.0);
    let mut root_first_reached = None;
    let mut tip_first_reached = None;
    for frame in 0..240 {
        chain.step(1.0, DT / 4.0);
        let root = chain.segments[0].angle.value;
        if root_first_reached.is_none() && root > 0.9 {
            root_first_reached = Some(frame);
        }
        if tip_first_reached.is_none() && chain.tip() > 0.9 {
            tip_first_reached = Some(frame);
        }
    }
    let (r, t) = (
        root_first_reached.expect("root never arrived"),
        tip_first_reached.expect("tip never arrived"),
    );
    assert!(
        t > r,
        "tip reached the target at {t}, root at {r} -- no lag"
    );
}

// ---------------------------------------------------------------------------
// Baking
// ---------------------------------------------------------------------------

fn simple_recipe(looseness: Looseness) -> Recipe {
    Recipe {
        name: "test",
        length: 40,
        keys: vec![
            Key {
                frame: 0,
                pose: flat(0.0),
            },
            Key {
                frame: 8,
                pose: flat(1.0),
            },
            Key {
                frame: 39,
                pose: flat(0.0),
            },
        ],
        looseness,
    }
}

#[test]
fn baking_produces_one_pose_per_frame() {
    let b = bake(&simple_recipe(Looseness::MARTIAL));
    assert_eq!(b.frames.len(), 40);
}

#[test]
fn baked_motion_is_continuous() {
    // A jump between adjacent frames is a pop, and a pop is the thing players
    // notice before anything else.
    let b = bake(&simple_recipe(Looseness::HEAVY));
    for (i, pair) in b.frames.windows(2).enumerate() {
        for part in PARTS {
            let (a, c) = (pair[0].get(part), pair[1].get(part));
            for k in 0..3 {
                let jump = (c.pos[k] - a.pos[k]).abs();
                assert!(jump < 0.35, "frame {i} {part:?} jumped {jump} on axis {k}");
            }
        }
    }
}

#[test]
fn baking_starts_settled_on_the_first_key() {
    // Otherwise every animation opens with a lurch from an arbitrary rest pose.
    let b = bake(&simple_recipe(Looseness::MARTIAL));
    let first = b.frames[0].get(Part::Torso);
    assert!(first.pos[1].abs() < 0.05, "opened at {}", first.pos[1]);
}

#[test]
fn loose_baking_lags_further_than_martial() {
    // The looseness setting has to actually do something, or it is a knob that
    // lies to whoever turns it.
    let martial = bake(&simple_recipe(Looseness::MARTIAL));
    let heavy = bake(&simple_recipe(Looseness::HEAVY));
    let at = |b: &anim::Baked, f: usize| b.frames[f].get(Part::ArmL).pos[1];
    // At the moment the key says "arm is up", the heavier arm should still be
    // catching up.
    assert!(
        at(&heavy, 8) < at(&martial, 8),
        "heavy {} did not lag martial {}",
        at(&heavy, 8),
        at(&martial, 8)
    );
}

#[test]
fn the_baked_tables_in_view_are_the_right_length() {
    // Guards against a stale generated file: if a recipe length changes and
    // nobody re-runs the bake, this catches it.
    assert_eq!(view::baked::POKE.len(), 17);
    assert_eq!(view::baked::OVERHEAD.len(), 42);
    assert_eq!(view::baked::GUARD_IN.len(), 10);
    assert_eq!(view::baked::ROLL.len(), 22);
    assert_eq!(view::baked::RECOIL.len(), 26);
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
    let dt = DT / anim::SUBSTEPS as f32;
    let mut t = 0.0f32;
    for _ in 0..(120 * anim::SUBSTEPS) {
        t += dt;
        s.step(t, dt); // target ramps at one unit per second
    }
    // Output trails the ramp by `lag` seconds; convert back to frames.
    (t - s.value) * 60.0
}

#[test]
fn the_lag_knob_means_what_it_says() {
    // The whole reason Feel is expressed in frames rather than in spring
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
fn heavy_parts_arrive_late_but_not_absent() {
    // The bug this pins: the first pass made "heavy" mean a low frequency, so
    // a heavy clip had barely started moving a third of the way through its own
    // startup. An opponent cannot react to a windup they cannot see. Every part
    // of every preset must be most of the way there within a few frames of the
    // key that asks for it.
    for (name, l) in [("martial", Looseness::MARTIAL), ("heavy", Looseness::HEAVY)] {
        for part in PARTS {
            let f = match part {
                Part::Torso => l.torso,
                Part::Head => l.head,
                Part::ArmL | Part::ArmR => l.arms,
                Part::LegL | Part::LegR => l.legs,
            };
            assert!(
                f.lag <= 4.5,
                "{name} {part:?} lags {} frames -- too late to read",
                f.lag
            );
        }
    }
}

#[test]
fn every_preset_is_stable_at_the_substep_rate() {
    // The stiffest preset is the one that decides whether SUBSTEPS is enough.
    for l in [Looseness::MARTIAL, Looseness::HEAVY] {
        for f in [l.torso, l.head, l.arms, l.legs] {
            let (freq, damp) = (2.0 * f.ring / (f.lag / 60.0), f.ring);
            let mut s = Spring::new(0.0, freq, damp);
            for _ in 0..600 {
                s.step(1.0, DT / anim::SUBSTEPS as f32);
            }
            assert!(
                s.settled(1.0, 0.02),
                "diverged or never settled: {}",
                s.value
            );
        }
    }
}

#[test]
fn the_overhead_reads_early() {
    // Direct check on the clip that exposed the problem: by the time a 14-frame
    // startup is a third gone, the silhouette must have visibly changed.
    let neutral = view::baked::OVERHEAD[0].get(Part::Torso);
    let early = view::baked::OVERHEAD[4].get(Part::Torso);
    let moved: f32 = (0..3)
        .map(|k| (early.pos[k] - neutral.pos[k]).abs())
        .sum::<f32>()
        + (0..3)
            .map(|k| (early.rot[k] - neutral.rot[k]).abs())
            .sum::<f32>();
    assert!(
        moved > 0.25,
        "overhead has barely moved by frame 4: {moved}"
    );
}
