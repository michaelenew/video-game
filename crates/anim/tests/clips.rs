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
use view::skeleton::{Group, JOINTS, Joint};

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
    //
    // Measured as how far each joint *moves*, not as how far its angles change.
    // A forearm rolling ninety degrees through a sword cut is a large number in
    // the twist channel and no motion at all on screen, and a test that cannot
    // tell those apart either forbids real animation or lets real pops through.
    let skeleton = reference();
    for b in authored() {
        for (i, pair) in b.frames.windows(2).enumerate() {
            // The opening frame of a one-shot is deliberately explosive -- a
            // dodge leaves at seventeen metres a second and a takeoff at
            // eighteen -- and the game never shows it raw: `Crossfade` eases
            // into every one of these over six frames. What has to be smooth is
            // the body of the clip, and every frame of a loop, which is seen
            // exactly as baked.
            if i == 0 && !b.clip.looping() {
                continue;
            }
            let before = view::skeleton::solve(skeleton, &pair[0]);
            let after = view::skeleton::solve(skeleton, &pair[1]);
            // Measured *relative to the hips*. A body that is travelling moves
            // every joint on it, and a dive roll or a takeoff moves them all
            // very fast -- that is the character going somewhere, not the pose
            // jumping. What a viewer reads as a pop is a limb moving relative
            // to the body it is attached to.
            let root = {
                let (p, q) = (before.origin[0], after.origin[0]);
                [q[0] - p[0], q[1] - p[1], q[2] - p[2]]
            };
            for j in JOINTS {
                let (p, q) = (before.origin[j.index()], after.origin[j.index()]);
                let moved = ((q[0] - p[0] - root[0]).powi(2)
                    + (q[1] - p[1] - root[1]).powi(2)
                    + (q[2] - p[2] - root[2]).powi(2))
                .sqrt();
                // The ceiling depends on where the joint is. A hand is at the
                // end of a two-metre lever and legitimately whips -- a sprint's
                // swing foot travels at twice the body's speed. A *hip* doing
                // that is a teleport, because the body it is attached to cannot
                // go there.
                let ceiling = match (j.depth(), j.group()) {
                    (_, Group::Root | Group::Spine | Group::Chest) => 0.15,
                    (_, Group::Head) => 0.30,
                    (0, _) => 0.24,
                    // A knee or an elbow whipping through the first frames of a
                    // dive is the fastest thing on the body relative to it.
                    (1, _) => 0.30,
                    // A hand or a foot at the end of its lever. Thirty-six
                    // centimetres a frame is twenty-one metres a second
                    // relative to the hips, which is about what a sprinter's
                    // foot does at the top of its swing -- so it is the edge of
                    // human rather than a number chosen to fit the clips.
                    _ => 0.36,
                };
                assert!(
                    moved < ceiling,
                    "{}: {} moved {moved:.3} m (ceiling {ceiling}) relative to the hips between frames {i} and {}",
                    b.clip.name(),
                    j.name(),
                    i + 1
                );
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
        let mut lowest = f32::MAX;
        for foot in [Joint::FootL, Joint::FootR] {
            let (heel, toe) = pose.foot_contact(skeleton, foot == Joint::FootL);
            let sole = heel[1].min(toe[1]);
            lowest = lowest.min(sole);
            // A raised rear heel is a stance, not a step. A whole foot in the
            // air is a step, and an idle does not take one.
            assert!(
                sole < 0.055,
                "idle frame {i}: {} is {sole:.3} m off the ground",
                foot.name()
            );
        }
        assert!(
            lowest < 0.02,
            "idle frame {i}: neither foot is on the floor"
        );
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

#[test]
fn the_clips_that_mirror_a_simulation_clock_are_the_same_length_as_it() {
    // Both of these are played by counting a simulation timer, so a clip that
    // is longer than its timer gets cut off and one that is shorter holds its
    // last frame. Neither is a crash, which is exactly why it would sit there
    // for months.
    assert_eq!(
        Clip::Parry.length(),
        sim::state::PARRY_FLOURISH,
        "the parry flourish and the clip that plays it disagree"
    );
    assert!(
        Clip::Defeat.length() <= sim::tuning::round_over_frames(),
        "the collapse is {} frames and the round pause is {} -- it would be cut off",
        Clip::Defeat.length(),
        sim::tuning::round_over_frames()
    );
}

/// Where each clip's feet are, frame by frame. Not a test -- a readout, for
/// when one of the floor tests fails and the question is *which frame*.
#[test]
#[ignore]
fn report_floor_clearance() {
    let skeleton = reference();
    for b in authored() {
        let worst = b
            .frames
            .iter()
            .map(|p| p.lowest_foot(skeleton))
            .fold(f32::MAX, f32::min);
        let at = b
            .frames
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, c)| a.lowest_foot(skeleton).total_cmp(&c.lowest_foot(skeleton)))
            .map(|(i, _)| i)
            .unwrap_or(0);
        println!("{:<22} lowest {worst:+.4} at frame {at}", b.clip.name());
    }
}

/// Worst relative motion per group across everything authored. A readout for
/// choosing the ceilings in `baked_motion_is_continuous` from what bodies
/// actually do, rather than one clip at a time.
#[test]
#[ignore]
fn report_worst_motion() {
    let skeleton = reference();
    let mut worst: Vec<(String, f32, String)> = Vec::new();
    for b in authored() {
        for (i, pair) in b.frames.windows(2).enumerate() {
            if i == 0 && !b.clip.looping() {
                continue;
            }
            let before = view::skeleton::solve(skeleton, &pair[0]);
            let after = view::skeleton::solve(skeleton, &pair[1]);
            let r = {
                let (p, q) = (before.origin[0], after.origin[0]);
                [q[0] - p[0], q[1] - p[1], q[2] - p[2]]
            };
            for j in JOINTS {
                let (p, q) = (before.origin[j.index()], after.origin[j.index()]);
                let m = ((q[0] - p[0] - r[0]).powi(2)
                    + (q[1] - p[1] - r[1]).powi(2)
                    + (q[2] - p[2] - r[2]).powi(2))
                .sqrt();
                let key = format!("{:?} depth {}", j.group(), j.depth());
                match worst.iter_mut().find(|(k, _, _)| *k == key) {
                    Some(e) if m > e.1 => {
                        e.1 = m;
                        e.2 = format!("{} {} frame {i}", b.clip.name(), j.name());
                    }
                    Some(_) => {}
                    None => {
                        worst.push((key, m, format!("{} {} frame {i}", b.clip.name(), j.name())))
                    }
                }
            }
        }
    }
    worst.sort_by(|a, c| c.1.total_cmp(&a.1));
    for (k, m, who) in worst {
        println!("{k:<18} {m:.3}   {who}");
    }
}

/// Every clip that fails the continuity ceilings, in one list. Used when a
/// batch of new clips lands, so the whole set can be triaged at once instead of
/// one assertion at a time.
#[test]
#[ignore]
fn report_discontinuous_clips() {
    let skeleton = reference();
    let mut bad: Vec<String> = Vec::new();
    for b in authored() {
        let mut worst = (0.0f32, String::new());
        for (i, pair) in b.frames.windows(2).enumerate() {
            if i == 0 && !b.clip.looping() {
                continue;
            }
            let before = view::skeleton::solve(skeleton, &pair[0]);
            let after = view::skeleton::solve(skeleton, &pair[1]);
            let r = {
                let (p, q) = (before.origin[0], after.origin[0]);
                [q[0] - p[0], q[1] - p[1], q[2] - p[2]]
            };
            for j in JOINTS {
                let (p, q) = (before.origin[j.index()], after.origin[j.index()]);
                let m = ((q[0] - p[0] - r[0]).powi(2)
                    + (q[1] - p[1] - r[1]).powi(2)
                    + (q[2] - p[2] - r[2]).powi(2))
                .sqrt();
                let ceiling = match (j.depth(), j.group()) {
                    (_, Group::Root | Group::Spine | Group::Chest) => 0.15,
                    (_, Group::Head) => 0.30,
                    (0, _) => 0.24,
                    (1, _) => 0.30,
                    _ => 0.36,
                };
                let over = m / ceiling;
                if over > 1.0 && over > worst.0 {
                    worst = (over, format!("{} frame {i} at {m:.3}", j.name()));
                }
            }
        }
        if worst.0 > 1.0 {
            bad.push(format!(
                "{:<24} {:.2}x  {}",
                b.clip.name(),
                worst.0,
                worst.1
            ));
        }
    }
    for line in &bad {
        println!("{line}");
    }
    println!(
        "{} of {} authored clips are over",
        bad.len(),
        authored().len()
    );
}

/// Authored keys whose joints are sitting *on* a limit.
///
/// A clamped joint means the pose asked for something it did not get -- almost
/// always an inverse-kinematics target further away than the limb is long. The
/// pose still bakes and still looks like something, but it is not the pose that
/// was written, and moving the target a few centimetres closer usually gets it
/// back. Worth knowing about; not worth failing over, because a strained reach
/// with a straight limb is a real thing a body does.
#[test]
#[ignore]
fn report_clamped_joints() {
    let skeleton = reference();
    let mut counts: Vec<(String, usize, usize)> = Vec::new();
    for r in recipes() {
        let mut clamped = 0;
        let mut total = 0;
        for k in &r.keys {
            for j in JOINTS {
                for c in 0..3 {
                    let v = k.pose.angles(j)[c];
                    let (lo, hi) = skeleton.bone(j).limits.channel(c);
                    total += 1;
                    if (v - lo).abs() < 1e-4 || (v - hi).abs() < 1e-4 {
                        clamped += 1;
                    }
                }
            }
        }
        if clamped > 0 {
            counts.push((r.clip.name().to_string(), clamped, total));
        }
    }
    counts.sort_by_key(|(_, c, _)| std::cmp::Reverse(*c));
    for (name, clamped, total) in counts {
        println!(
            "{name:<24} {clamped:>3} of {total:>4} channels on a limit  ({:.0}%)",
            clamped as f32 / total as f32 * 100.0
        );
    }
}

#[test]
fn every_clip_the_game_can_play_has_been_authored() {
    // A clip with no recipe bakes as a held rest pose. That is deliberate while
    // a family is being written -- one unfinished file should not block the
    // rest -- but it is a character standing still in the middle of a match,
    // and the only thing that would stop it staying that way is this.
    //
    // There is nothing outstanding. If that changes, name the clip here with a
    // reason rather than deleting the assertion.
    let missing: Vec<&str> = anim::clips::missing().iter().map(|c| c.name()).collect();
    assert!(missing.is_empty(), "no recipe for: {missing:?}");
}

/// How far each arm is extended, as a fraction of its own length.
///
/// One at the end means the arm is straight and the solver has given up: the
/// target was further away than the limb is long. A grip placed a few
/// centimetres too far out shows here as every key in a clip pinned at 1.00,
/// which is a character holding a weapon at arm's length whatever pose was
/// written.
#[test]
#[ignore]
fn report_arm_extension() {
    let skeleton = reference();
    let reach = skeleton.arm_reach();
    for r in recipes() {
        let mut straight = 0;
        let mut total = 0;
        let mut worst = 0.0f32;
        for k in &r.keys {
            let skin = view::skeleton::solve(skeleton, &k.pose);
            for (shoulder, wrist) in [(Joint::ArmL, Joint::HandL), (Joint::ArmR, Joint::HandR)] {
                let a = skin.origin[shoulder.index()];
                let b = skin.origin[wrist.index()];
                let d = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2))
                    .sqrt()
                    / reach;
                total += 1;
                worst = worst.max(d);
                if d > 0.985 {
                    straight += 1;
                }
            }
        }
        if straight > 0 {
            println!(
                "{:<24} {straight:>3} of {total:>3} arms straight, worst {worst:.3}",
                r.clip.name()
            );
        }
    }
}

/// Where the Champion's hands actually end up, key by key.
///
/// The grip is authored as a point and solved for by IK, so the only way to
/// know whether a swing travels is to read the wrists back off the solved
/// skeleton. Prints each key's hand midpoint in character space and how much
/// of the arm's reach it is using.
#[test]
#[ignore]
fn report_champion_grip() {
    let skeleton = reference();
    let reach = skeleton.arm_reach();
    for r in recipes() {
        if !r.clip.name().starts_with("champion") {
            continue;
        }
        println!("{}", r.clip.name());
        for k in &r.keys {
            let skin = view::skeleton::solve(skeleton, &k.pose);
            let l = skin.origin[Joint::HandL.index()];
            let rr = skin.origin[Joint::HandR.index()];
            let mid = [
                (l[0] + rr[0]) / 2.0,
                (l[1] + rr[1]) / 2.0,
                (l[2] + rr[2]) / 2.0,
            ];
            let ext = |sh: Joint, h: [f32; 3]| {
                let a = skin.origin[sh.index()];
                ((h[0] - a[0]).powi(2) + (h[1] - a[1]).powi(2) + (h[2] - a[2]).powi(2)).sqrt()
                    / reach
            };
            println!(
                "  f{:>3}  grip [{:>6.2},{:>5.2},{:>6.2}]  L [{:>6.2},{:>5.2},{:>6.2}] {:.2}  R [{:>6.2},{:>5.2},{:>6.2}] {:.2}",
                k.frame,
                mid[0],
                mid[1],
                mid[2],
                l[0],
                l[1],
                l[2],
                ext(Joint::ArmL, l),
                rr[0],
                rr[1],
                rr[2],
                ext(Joint::ArmR, rr),
            );
        }
    }
}

/// A hand goes where it is sent.
///
/// This is the standard the whole authoring API rests on. `reach_l`/`reach_r`
/// take a point and are trusted to hit it, so every arm in every clip is only
/// as good as this: a solver that quietly puts the wrist somewhere else turns
/// a written pose into a different pose, with nothing anywhere to say so, and
/// that is exactly what happened to all three Champion clips.
///
/// The grid is every place a fighting pose puts a hand -- across the body, out
/// to the side, behind the hip, overhead -- and the tolerance is two
/// centimetres, which is a knuckle. Targets the arm genuinely cannot get to
/// are excluded by construction rather than by loosening the ceiling: nothing
/// here is further from the shoulder than the arm is long, and nothing is so
/// close that the elbow would have to fold past what an elbow folds to.
#[test]
fn a_hand_goes_where_it_is_sent() {
    let skeleton = reference();
    let reach = skeleton.arm_reach();
    let mut worst: (f32, [f32; 3]) = (0.0, [0.0; 3]);
    for left in [true, false] {
        let shoulder_j = if left { Joint::ArmL } else { Joint::ArmR };
        let shoulder =
            view::skeleton::solve(skeleton, &view::pose::Pose::rest()).origin[shoulder_j.index()];
        for xi in -6..=6 {
            for yi in -6..=6 {
                for zi in -6..=6 {
                    let target = [
                        shoulder[0] + xi as f32 * reach / 6.0,
                        shoulder[1] + yi as f32 * reach / 6.0,
                        shoulder[2] + zi as f32 * reach / 6.0,
                    ];
                    let d = ((target[0] - shoulder[0]).powi(2)
                        + (target[1] - shoulder[1]).powi(2)
                        + (target[2] - shoulder[2]).powi(2))
                    .sqrt();
                    // Inside the arm, and outside the fold an elbow cannot
                    // pass: 150 degrees of flexion leaves the wrist about a
                    // third of the arm's length from the shoulder.
                    if d > reach * 0.97 || d < reach * 0.30 {
                        continue;
                    }
                    // And in front of the shoulder, or below it. An arm goes
                    // overhead by coming up the front; it does not go up
                    // behind the back, and asking for a wrist there is asking
                    // for something a shoulder does not do. Everywhere else --
                    // across the chest, out to the side, behind the hip, over
                    // the head -- is fair game and is where the clips live.
                    let behind = target[2] - shoulder[2] < -0.1 * reach;
                    let high = target[1] - shoulder[1] > -0.1 * reach;
                    if behind && high {
                        continue;
                    }
                    let mut p = view::pose::Pose::rest();
                    let miss = view::ik::hand_to(&mut p, skeleton, left, target);
                    if miss > worst.0 {
                        worst = (miss, target);
                    }
                }
            }
        }
    }
    assert!(
        worst.0 < 0.02,
        "a wrist sent to [{:.2},{:.2},{:.2}] landed {:.3} m away",
        worst.1[0],
        worst.1[1],
        worst.1[2],
        worst.0
    );
}

/// The two readings of a joint's three angles really are the same rotation.
///
/// `unwound` swaps keys between them freely, so if they ever differed it would
/// not show up as a bug in the numbers -- it would show up as poses quietly
/// changing shape somewhere between authoring and the screen.
#[test]
fn both_readings_of_a_joint_are_the_same_rotation() {
    let skeleton = reference();
    for j in view::skeleton::JOINTS {
        let bone = skeleton.bone(j);
        for (a, b, c) in [
            (0.0f32, 0.0f32, 0.0f32),
            (0.7, -0.3, 0.2),
            (-1.1, 0.9, -1.4),
            (2.4, 1.2, 0.5),
        ] {
            let here = [a, b, c];
            let other = view::skeleton::other_reading(bone, j, here);
            // Only where the joint would actually accept both, which is the
            // same condition `unwound` swaps under: a reading the limits clamp
            // is a different rotation on purpose.
            let (cs, cp, ct) = bone.limits.clamp(other[0], other[1], other[2]);
            if (cs - other[0]).abs() > 1e-4
                || (cp - other[1]).abs() > 1e-4
                || (ct - other[2]).abs() > 1e-4
            {
                continue;
            }
            let one = view::skeleton::local_rotation(bone, j, here);
            let two = view::skeleton::local_rotation(bone, j, other);
            // Read as directions rather than as quaternions, because q and -q
            // are the same rotation and the sign is not worth arguing with.
            for axis in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]] {
                let p = one.rotate(axis);
                let q = two.rotate(axis);
                let apart =
                    ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
                assert!(
                    apart < 1e-4,
                    "{j:?} reads {here:?} and {other:?} differently, by {apart}"
                );
            }
        }
    }
}
