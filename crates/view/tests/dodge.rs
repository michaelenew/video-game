//! The dodge is a dive roll, and every class rolls its own (`view::dodge`).
//!
//! What these hold is what would be wrong on screen and right in a table of
//! numbers: a body rolled about the wrong point sinks through the floor or
//! vaults over it, a roll that is still going when its clip has finished
//! stands up sideways, and a roll that turns away from the travel reads as the
//! character tripping.

use sim::Class;
use sim::class::ALL_CLASSES;
use sim::state::Action;
use sim::{Fx, Input, World};
use view::dodge::{
    WAYS, Way, clip, first_vulnerable, floor, length, lift, lowest, roll, turned, way,
};
use view::interp::interpolate;
use view::math::{self, Quat};
use view::skeleton::{skeleton_for, solve};

/// The lowest point of the body at a frame of the roll, turned over the way
/// the renderer turns it, and how far the renderer lifts it to hold it to the
/// floor (`view::dodge::lift`).
fn lowest_and_lift(class: Class, w: Way, frame: f32) -> (f32, f32) {
    let skeleton = skeleton_for(class);
    let pose = clip(class, w).at_fractional(frame);
    let skin = solve(&skeleton, &pose);
    let t = turned(class, w, frame);
    let low = lowest(&skin, &skeleton, t.over, t.about);
    (low, lift(floor(class, frame), low))
}

/// The frames, in quarters, while the body is properly over: neither just
/// starting nor just finishing.
fn over(class: Class) -> impl Iterator<Item = f32> {
    let r = roll(class);
    (0..length() as u32 * 4)
        .map(|q| q as f32 / 4.0)
        .filter(move |&f| (0.15..=0.85).contains(&r.through(f)))
}

/// How far round a turn is, nought to a half.
fn angle(q: Quat) -> f32 {
    2.0 * q.0[3].abs().clamp(0.0, 1.0).acos() / std::f32::consts::TAU
}

#[test]
fn a_rolled_body_keeps_its_back_on_the_floor() {
    // Drawn, with the lift: never through the floor, and -- rolling rather
    // than vaulting -- on it while it is over.
    for class in ALL_CLASSES {
        for w in WAYS {
            for f in over(class) {
                let (low, up) = lowest_and_lift(class, w, f);
                let drawn = low + up;
                assert!(
                    drawn > -0.03,
                    "{class:?} rolling {w:?} is drawn {:.2} m through the floor at frame {f}",
                    -drawn
                );
                assert!(
                    drawn < 0.03,
                    "{class:?} rolling {w:?} is drawn {drawn:.2} m off the floor at frame {f}"
                );
            }
        }
    }
}

#[test]
fn a_roll_turns_about_the_middle_of_the_ball() {
    // The lift puts a rolling body on the floor whatever it turns about, so
    // what is left to get wrong is the pivot being somewhere else entirely:
    // then the lift is large and changes fast, and the body bobs as it rolls.
    // A few centimetres either way is the arc; a third of a metre is a pivot
    // in the wrong place.
    for class in ALL_CLASSES {
        for w in WAYS {
            for f in over(class) {
                let (_, up) = lowest_and_lift(class, w, f);
                assert!(
                    up.abs() < 0.32,
                    "{class:?} rolling {w:?} is lifted {up:.2} m at frame {f}: \
                     `view::dodge::roll`'s `about` ({:?}) is not the middle of its ball",
                    roll(class).about
                );
            }
        }
    }
}

#[test]
#[ignore]
fn report_roll_lifts() {
    for class in ALL_CLASSES {
        println!("{class:?} about {:?}", roll(class).about);
        for w in WAYS {
            let row: Vec<String> = (0..length())
                .map(|f| format!("{:+.2}", lowest_and_lift(class, w, f as f32).1))
                .collect();
            println!("  {w:?}: {}", row.join(" "));
        }
    }
}

#[test]
fn a_roll_starts_and_ends_upright() {
    let last = (length() - 1) as f32;
    for class in ALL_CLASSES {
        for w in WAYS {
            let start = turned(class, w, 0.0).whole();
            let end = turned(class, w, last).whole();
            assert!(angle(start) < 0.01, "{class:?} {w:?} starts turned over");
            assert!(
                angle(end) < 0.01,
                "{class:?} {w:?} is still {:.0} degrees over on its last frame",
                angle(end) * 360.0
            );
            // And properly goes over in between.
            let most = (0..length())
                .map(|f| angle(turned(class, w, f as f32).over))
                .fold(0.0f32, f32::max);
            assert!(most > 0.4, "{class:?} {w:?} never turns over");
        }
    }
}

#[test]
fn the_turn_is_mostly_done_when_the_cover_runs_out() {
    // The silhouette changes on the frame the invulnerability ends: balled up
    // and going over while the frames are free, opening as they stop being.
    // A turn that had barely started by then would put the opening in the
    // middle of the roll, where it reads as nothing.
    let brk = first_vulnerable() as f32;
    for class in ALL_CLASSES {
        let r = roll(class);
        let k = r.through(brk);
        assert!(
            k > 0.55,
            "{class:?} is only {:.0}% round when the cover runs out",
            k * 100.0
        );
        assert!(
            r.through(1.0) < 0.05,
            "{class:?} is over before its feet have left"
        );
    }
}

#[test]
fn a_roll_goes_over_toward_the_travel() {
    // A quarter of the way round, the head is out toward where the body is
    // going. Character space as drawn has the body's right at `-X`.
    for class in ALL_CLASSES {
        let r = roll(class);
        let quarter = (0..length() * 8)
            .map(|q| q as f32 / 8.0)
            .find(|&f| r.through(f) >= 0.25)
            .unwrap();
        let toward = |w: Way| -> math::V3 {
            match w {
                Way::Forward => [0.0, 0.0, 1.0],
                Way::Back => [0.0, 0.0, -1.0],
                Way::Right => [-1.0, 0.0, 0.0],
                Way::Left => [1.0, 0.0, 0.0],
            }
        };
        for w in WAYS {
            let up = turned(class, w, quarter).whole().rotate([0.0, 1.0, 0.0]);
            assert!(
                math::dot(up, toward(w)) > 0.7,
                "{class:?} rolling {w:?} turns its head {up:?} a quarter of the way round"
            );
        }
    }
}

#[test]
fn every_class_rolls_its_own() {
    // Six sets of four that came out the same poses would be a lot of work for
    // nothing, and a class whose roll is somebody else's is the request
    // undone. Measured against the Champion's, the shared shape.
    for class in ALL_CLASSES {
        if class == Class::Champion {
            continue;
        }
        for w in WAYS {
            let (a, b) = (clip(Class::Champion, w), clip(class, w));
            assert_ne!(a, b);
            let n = a.length().min(b.length()) as u32;
            let apart: f32 = (0..n).map(|i| a.at(i).separation(&b.at(i))).sum::<f32>() / n as f32;
            assert!(
                apart > 1.0,
                "{class:?}'s roll {w:?} is the Champion's ({apart:.2} apart)"
            );
        }
        assert!(
            roll(class) != roll(Class::Champion),
            "{class:?} goes over exactly when the Champion does"
        );
    }
}

#[test]
fn the_clip_and_the_turn_read_the_same_way() {
    assert_eq!(way([0.0, 3.0]), Way::Forward);
    assert_eq!(way([0.0, -3.0]), Way::Back);
    assert_eq!(way([3.0, 1.0]), Way::Right);
    assert_eq!(way([-3.0, 1.0]), Way::Left);
    // Standing still -- a blink, which arrives rather than travels -- rolls
    // forward where it lands.
    assert_eq!(way([0.0, 0.0]), Way::Forward);
}

#[test]
fn the_dodge_does_not_change_its_frames() {
    // A dive roll is a drawing. The dodge's length and its invulnerable window
    // are the Oven's, and this is the whole of what the roll reads from them.
    assert_eq!(length(), sim::tuning::dodge_frames());
    assert_eq!(
        first_vulnerable(),
        sim::tuning::dodge_frames() - (sim::tuning::dodge_frames() - sim::tuning::dodge_iframes())
    );
}

#[test]
fn a_grounded_dodge_is_drawn_turned_over() {
    for class in ALL_CLASSES {
        let mut w = World::with_classes([class, Class::Champion]);
        for _ in 0..10 {
            w.advance([Input::default(); 2]);
        }
        let p = &mut w.players[0];
        assert!(p.grounded, "{class:?} is not standing");
        // Half way through the turn, going forward at a dodge's speed.
        let r = roll(class);
        let mid = (r.from + r.upright()) / 2;
        p.action = Action::Dodge {
            left: length() - mid,
        };
        p.vel = p.facing.scale(Fx::from_int(4));
        let frame = interpolate(&w, &w, 1.0);
        let me = frame.players[0];
        assert!(
            angle(me.roll) > 0.3,
            "{class:?} is drawn {:.0} degrees over half way through a roll",
            angle(me.roll) * 360.0
        );
        assert_eq!(me.roll_about, r.about);

        // And not in the air, where the airdodge is its own thing.
        let p = &mut w.players[0];
        p.grounded = false;
        p.pos.y = p.pos.y.add(Fx::from_int(2));
        let frame = interpolate(&w, &w, 1.0);
        assert!(
            angle(frame.players[0].roll) < 0.01,
            "{class:?}'s airdodge turns over"
        );
    }
}

#[test]
#[ignore]
fn report_ball_middles() {
    // The middle of each class's body half way round, as `about` wants it.
    for class in ALL_CLASSES {
        let skeleton = skeleton_for(class);
        let r = roll(class);
        let mid = (0..length() * 8)
            .map(|q| q as f32 / 8.0)
            .find(|&f| r.through(f) >= 0.5)
            .unwrap();
        for w in WAYS {
            let skin = solve(&skeleton, &clip(class, w).at_fractional(mid));
            let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
            for j in view::skeleton::JOINTS {
                let (c, rot) = skin.box_of(&skeleton, j);
                let h = skeleton.bone(j).half;
                for sx in [-1.0, 1.0f32] {
                    for sy in [-1.0, 1.0f32] {
                        for sz in [-1.0, 1.0f32] {
                            let p = math::add(c, rot.rotate([sx * h[0], sy * h[1], sz * h[2]]));
                            for i in 0..3 {
                                lo[i] = lo[i].min(p[i]);
                                hi[i] = hi[i].max(p[i]);
                            }
                        }
                    }
                }
            }
            println!(
                "{class:?} {w:?} frame {mid}: middle [{:.2}, {:.2}, {:.2}] size [{:.2}, {:.2}, {:.2}]",
                (lo[0] + hi[0]) / 2.0,
                (lo[1] + hi[1]) / 2.0,
                (lo[2] + hi[2]) / 2.0,
                hi[0] - lo[0],
                hi[1] - lo[1],
                hi[2] - lo[2]
            );
        }
    }
}

#[test]
fn a_roll_to_the_side_faces_its_travel_and_turns_back() {
    // Nobody rolls over their own shoulder line and comes up on their feet:
    // to the side the body turns to face the way it is going, rolls forward,
    // and turns back to the front as it rises.
    use std::f32::consts::FRAC_PI_2;
    let last = (length() - 1) as f32;
    for class in ALL_CLASSES {
        let r = roll(class);
        let mid = (r.from + r.upright()) as f32 / 2.0;
        for (w, sign) in [(Way::Right, -1.0), (Way::Left, 1.0)] {
            let t = turned(class, w, mid);
            assert!(
                (t.yaw - sign * FRAC_PI_2).abs() < 0.05,
                "{class:?} rolling {w:?} faces {:.0} degrees off the front mid-roll",
                t.yaw.to_degrees()
            );
            assert!(turned(class, w, last).yaw.abs() < 0.01);
            assert!(turned(class, w, 0.0).yaw.abs() < 0.01);
        }
        for w in [Way::Forward, Way::Back] {
            assert_eq!(turned(class, w, mid).yaw, 0.0);
        }
    }
}
