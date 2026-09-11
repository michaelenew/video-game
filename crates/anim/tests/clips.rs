//! Standards every authored clip is held to.
//!
//! These run against whatever has been authored so far and skip the rest, so
//! one unfinished family never blocks another. What they check is the set of
//! failures that are invisible in a table of numbers and obvious the moment
//! somebody looks at the game: feet through the floor, feet sliding, a loop
//! that jumps, a knee that inverts, a telegraph nobody can react to.

use view::clips::{ALL, Clip, Family};
use view::play::{CROUCH_STRIDE, RUN_STRIDE, STRAFE_STRIDE, WALK_STRIDE};
use view::pose::{Pose, reference};
use view::skeleton::{JOINTS, Joint};

/// Only the clips somebody has actually written. Unauthored ones bake as a held
/// rest pose and would pass everything here for the wrong reason.
fn authored() -> Vec<anim::Baked> {
    let missing = anim::clips::missing();
    anim::bake_all()
        .0
        .into_iter()
        .filter(|b| !missing.contains(&b.clip))
        .collect()
}

fn recipes() -> Vec<anim::Recipe> {
    anim::clips::all()
}

#[test]
fn no_clip_is_authored_twice() {
    let twice = anim::clips::duplicated();
    assert!(
        twice.is_empty(),
        "authored more than once: {:?}",
        twice.iter().map(|c| c.name()).collect::<Vec<_>>()
    );
}

#[test]
fn every_recipe_fills_the_clip_it_claims() {
    for b in authored() {
        assert_eq!(
            b.frames.len(),
            b.clip.length() as usize,
            "{} baked {} frames for a {}-frame clip",
            b.clip.name(),
            b.frames.len(),
            b.clip.length()
        );
    }
}

#[test]
fn keys_are_in_order_and_inside_the_clip() {
    for r in recipes() {
        let length = r.clip.length();
        let mut last = None;
        for k in &r.keys {
            assert!(
                k.frame < length,
                "{}: a key at frame {} in a {}-frame clip",
                r.clip.name(),
                k.frame,
                length
            );
            if let Some(prev) = last {
                assert!(
                    k.frame > prev,
                    "{}: keys out of order at frame {}",
                    r.clip.name(),
                    k.frame
                );
            }
            last = Some(k.frame);
        }
    }
}

#[test]
fn no_authored_key_asks_for_a_joint_a_body_does_not_have() {
    // The solver clamps, so a broken key does not reach the screen -- but it
    // does silently change what was authored, and an author should be told
    // rather than quietly overruled.
    let skeleton = reference();
    for r in recipes() {
        for k in &r.keys {
            let bad = k.pose.violations(skeleton);
            assert!(
                bad.is_empty(),
                "{} at frame {}: {:?}",
                r.clip.name(),
                k.frame,
                bad.iter()
                    .map(|(j, c, v)| format!("{} channel {c} at {v:.0} degrees", j.name()))
                    .collect::<Vec<_>>()
            );
        }
    }
}

#[test]
fn every_baked_number_is_a_number() {
    for b in authored() {
        for (i, pose) in b.frames.iter().enumerate() {
            for (c, v) in pose.channels.iter().enumerate() {
                assert!(
                    v.is_finite(),
                    "{} frame {i} channel {c} is {v}",
                    b.clip.name()
                );
            }
        }
    }
}

#[test]
fn baked_motion_is_continuous() {
    // A jump between adjacent frames is a pop, and a pop is the thing players
    // notice before anything else.
    for b in authored() {
        for (i, pair) in b.frames.windows(2).enumerate() {
            for j in JOINTS {
                let (a, c) = (pair[0].angles(j), pair[1].angles(j));
                for k in 0..3 {
                    let jump = (c[k] - a[k]).to_degrees().abs();
                    assert!(
                        jump < 32.0,
                        "{}: {} channel {k} jumped {jump:.0} degrees at frame {i}",
                        b.clip.name(),
                        j.name()
                    );
                }
            }
        }
    }
}

#[test]
fn looping_clips_close_on_themselves() {
    for b in authored() {
        if !b.clip.looping() || b.frames.len() < 4 {
            continue;
        }
        let seam = b.frames[b.frames.len() - 1].separation(&b.frames[0]);
        let ordinary: f32 = b
            .frames
            .windows(2)
            .map(|w| w[0].separation(&w[1]))
            .sum::<f32>()
            / (b.frames.len() - 1) as f32;
        assert!(
            seam < ordinary * 4.0 + 0.05,
            "{}: the loop jumps {seam:.3} at the seam, against {ordinary:.3} between ordinary frames",
            b.clip.name()
        );
    }
}

// ---------------------------------------------------------------------------
// The floor
// ---------------------------------------------------------------------------

/// Families whose clips happen with at least one foot near the ground.
fn grounded(clip: Clip) -> bool {
    matches!(
        clip.family(),
        Family::Locomotion | Family::Turns | Family::Defence
    )
}

#[test]
fn grounded_clips_do_not_put_a_foot_through_the_floor() {
    let skeleton = reference();
    for b in authored() {
        if !grounded(b.clip) {
            continue;
        }
        for (i, pose) in b.frames.iter().enumerate() {
            let sole = pose.lowest_foot(skeleton);
            assert!(
                sole > -0.045,
                "{} frame {i}: a foot is {:.3} m below the floor",
                b.clip.name(),
                -sole
            );
        }
    }
}

#[test]
fn grounded_clips_spend_most_of_their_time_touching_the_ground() {
    // A run has a flight phase and is allowed it. A walk that floats is a bug.
    let skeleton = reference();
    for b in authored() {
        if !grounded(b.clip) {
            continue;
        }
        let touching = b
            .frames
            .iter()
            .filter(|p| p.lowest_foot(skeleton).abs() < 0.07)
            .count();
        let fraction = touching as f32 / b.frames.len() as f32;
        // A sprint has a flight phase after each step, so half the cycle with
        // nothing on the floor is correct rather than a defect.
        let floor = if b.clip.name().starts_with("run") {
            0.35
        } else {
            0.75
        };
        assert!(
            fraction >= floor,
            "{}: only {:.0}% of frames have a foot on the ground",
            b.clip.name(),
            fraction * 100.0
        );
    }
}

// ---------------------------------------------------------------------------
// Foot skate
// ---------------------------------------------------------------------------

/// How far the body travels per cycle for a clip that is played by distance,
/// and which way it goes.
fn travel(clip: Clip) -> Option<([f32; 3], f32)> {
    let name = clip.name();
    let stride = if name.starts_with("run") {
        RUN_STRIDE
    } else if name.starts_with("walk") {
        WALK_STRIDE
    } else if name == "crouch_walk" {
        CROUCH_STRIDE
    } else {
        return None;
    };
    // A sidestep is authored -- and played -- at a shorter stride, because a
    // leg swung sideways runs out of hip long before one swung forward runs out
    // of leg.
    let (dir, stride) = if name.ends_with("forward") {
        ([0.0, 0.0, 1.0], stride)
    } else if name.ends_with("back") {
        ([0.0, 0.0, -1.0], stride)
    } else if name.ends_with("left") {
        ([-1.0, 0.0, 0.0], stride * STRAFE_STRIDE)
    } else if name.ends_with("right") {
        ([1.0, 0.0, 0.0], stride * STRAFE_STRIDE)
    } else if name == "crouch_walk" {
        ([0.0, 0.0, 1.0], stride)
    } else {
        return None;
    };
    Some((dir, stride))
}

#[test]
fn a_planted_foot_does_not_slide() {
    // The test the whole stride arithmetic in `clips::locomotion` exists to
    // pass. A foot near the floor is carrying the body's weight; if its
    // position in the *world* moves while it does, the character is skating,
    // and skating is the first thing anybody notices about bad animation.
    let skeleton = reference();
    for b in authored() {
        let Some((dir, stride)) = travel(b.clip) else {
            continue;
        };
        let n = b.frames.len();
        // Flat on the floor -- heel and toe both down -- is the unambiguous
        // weight-bearing part of a stride, and the part where nothing at all
        // should move. A foot rocking from heel to toe genuinely does shift its
        // contact point, so measuring through those transitions would be
        // measuring anatomy rather than skate.
        let flat = |i: usize, left: bool| -> Option<[f32; 3]> {
            let pose = &b.frames[i % n];
            let (heel, toe) = pose.foot_contact(skeleton, left);
            if heel[1] > 0.028 || toe[1] > 0.028 {
                return None;
            }
            let along = stride * (i as f32) / n as f32;
            let mid = [
                (heel[0] + toe[0]) * 0.5 + dir[0] * along,
                0.0,
                (heel[2] + toe[2]) * 0.5 + dir[2] * along,
            ];
            Some(mid)
        };

        let mut worst: f32 = 0.0;
        let mut worst_at = (0usize, true);
        for left in [true, false] {
            let mut from: Option<(usize, [f32; 3])> = None;
            for i in 0..=n {
                match (flat(i, left), from) {
                    (Some(p), Some((start, anchor))) => {
                        let drift =
                            ((p[0] - anchor[0]).powi(2) + (p[2] - anchor[2]).powi(2)).sqrt();
                        if drift > worst {
                            worst = drift;
                            worst_at = (start, left);
                        }
                    }
                    (Some(p), None) => from = Some((i, p)),
                    (None, _) => from = None,
                }
            }
        }

        // Four centimetres across the whole flat phase. Under that, a viewer
        // reads it as a planted foot.
        assert!(
            worst < 0.04,
            "{}: the {} foot slid {:.3} m while flat on the floor, from frame {}",
            b.clip.name(),
            if worst_at.1 { "left" } else { "right" },
            worst,
            worst_at.0
        );
    }
}

// ---------------------------------------------------------------------------
// Reading the animation
// ---------------------------------------------------------------------------

#[test]
fn an_attack_has_visibly_changed_by_the_time_it_is_a_third_gone() {
    // The opponent has the startup frames to read a move and decide. A
    // silhouette that has not moved by then is asking them to react to
    // something they cannot see.
    for b in authored() {
        let Some((startup, _, _)) = b.clip.phases() else {
            continue;
        };
        if startup < 3 || b.frames.len() < 4 {
            continue;
        }
        let look_at = ((startup as f32 * 0.4) as usize).clamp(2, b.frames.len() - 1);
        let moved = b.frames[0].separation(&b.frames[look_at]);
        assert!(
            moved > 0.35,
            "{}: barely moved ({moved:.2}) by frame {look_at} of a {startup}-frame startup",
            b.clip.name()
        );
    }
}

#[test]
fn the_three_phases_of_an_attack_look_different() {
    // Reading startup from active from recovery across the arena is a gameplay
    // requirement, not an art one: the whole neutral game is built on being
    // able to tell which of them you are looking at.
    for b in authored() {
        let Some((last_startup, first_active, first_recovery)) = b.clip.phases() else {
            continue;
        };
        let at = |f: u16| b.frames[(f as usize).min(b.frames.len() - 1)];
        let windup = at(last_startup);
        let strike = at(first_active);
        let recover = at(b.frames.len() as u16 - 1);
        assert!(
            windup.separation(&strike) > 0.6,
            "{}: the wind-up and the strike look alike ({:.2})",
            b.clip.name(),
            windup.separation(&strike)
        );
        assert!(
            strike.separation(&recover) > 0.5,
            "{}: the strike and the recovery look alike ({:.2})",
            b.clip.name(),
            strike.separation(&recover)
        );
        let _ = first_recovery;
    }
}

#[test]
fn an_idle_keeps_both_feet_down() {
    // An idle that lifts a foot reads as about to do something, and it is not.
    let skeleton = reference();
    if anim::clips::missing().contains(&Clip::Idle) {
        return;
    }
    let idle = authored()
        .into_iter()
        .find(|b| b.clip == Clip::Idle)
        .expect("idle is authored");
    for (i, pose) in idle.frames.iter().enumerate() {
        let skin = view::skeleton::solve(skeleton, pose);
        for foot in [Joint::FootL, Joint::FootR] {
            let (centre, _) = skin.box_of(skeleton, foot);
            let sole = centre[1] - skeleton.bone(foot).half[1];
            assert!(
                sole < 0.03,
                "idle frame {i}: {} is {sole:.3} m off the ground",
                foot.name()
            );
        }
    }
}

#[test]
fn clips_that_are_meant_to_differ_actually_differ() {
    // Six directional walks that are the same three poses would be a lot of
    // work for nothing. This is the cheap check that they are not.
    let baked = authored();
    let find = |c: Clip| baked.iter().find(|b| b.clip == c);
    for (a, b) in [
        (Clip::WalkForward, Clip::WalkBack),
        (Clip::WalkLeft, Clip::WalkRight),
        (Clip::WalkForward, Clip::RunForward),
        (Clip::DodgeLeft, Clip::DodgeRight),
        (Clip::HitLight, Clip::HitHeavy),
    ] {
        let (Some(x), Some(y)) = (find(a), find(b)) else {
            continue;
        };
        let n = x.frames.len().min(y.frames.len());
        let apart: f32 = (0..n)
            .map(|i| x.frames[i].separation(&y.frames[i]))
            .sum::<f32>()
            / n as f32;
        assert!(
            apart > 0.25,
            "{} and {} are nearly the same clip ({apart:.2} apart)",
            a.name(),
            b.name()
        );
    }
}

#[test]
fn every_clip_the_contract_names_is_a_real_name() {
    // Guards the thing that lets a dozen people author in parallel: the name in
    // `view::clips` and the name a recipe claims have to be the same string.
    for c in ALL {
        assert_eq!(Clip::by_name(c.name()), Some(*c));
        assert!(
            !c.what().is_empty(),
            "{} says nothing about itself",
            c.name()
        );
        assert!(
            c.length() >= 2,
            "{} is {} frames long",
            c.name(),
            c.length()
        );
    }
    // And the fallback has to be a real pose, not a wall of zeroes with a
    // foot through the floor.
    assert!(Pose::rest().lowest_foot(reference()).abs() < 0.02);
}
