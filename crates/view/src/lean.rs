//! **Leaning into the going** (2026-10-22): a body carried a little ahead of
//! its feet while it travels, and further into a change of speed.
//!
//! The physics is one line. A body speeding up is pushed forward by the floor
//! at its feet, and to keep that push from tipping it over backwards it puts
//! its weight ahead of them: it leans by the angle whose tangent is its
//! acceleration over gravity. Braking is the same backwards, and a cut to the
//! side leans into the cut. A sprinter's first steps are the extreme of it, a
//! forty-five degree line from the ankle to the ear.
//!
//! The simulation goes from a standstill to a run in three frames, which
//! taken literally is a lean past horizontal. So the velocity the lean reads
//! is eased first, and the angle saturates: what the eye needs is *that* the
//! body went into the start and sat back into the stop, for about the third
//! of a second a real one takes, not the literal angle.
//!
//! The lean is from the ankles rather than the waist -- the whole body tips
//! as one line, the hips carried forward over the feet -- and the feet are
//! put back exactly where the clip had them, so nothing slides. Renderer-local
//! like the rest of [`crate::play::Crossfade`]: a lean that hiccups across a
//! rollback is invisible.

use crate::pose::{Pose, reference};
use crate::skeleton::{Joint, solve};

/// How long the velocity the lean reads takes to catch up with the real one,
/// in seconds. It is also how long a lean into a start lasts: the eased
/// velocity's rate of change is what is leaned into, and it dies away on this
/// clock.
const EASE_VELOCITY: f32 = 0.18;
/// How long the lean itself takes to arrive, in seconds, so the first frame
/// of a start is not a pop.
const EASE_LEAN: f32 = 0.07;
/// How long the lean takes to come and go when the body starts or stops being
/// free to lean -- into an attack, a dodge, the air.
const EASE_GATE: f32 = 0.1;

/// The furthest a start tips the body forward, a stop back and a cut sideways,
/// in degrees.
const MOST_FORWARD: f32 = 14.0;
const MOST_BACK: f32 = 9.0;
const MOST_SIDE: f32 = 9.0;
/// The acceleration, in metres a second a second, at which the lean is about
/// three quarters of the way to its most. Above it the angle saturates.
const HARD: f32 = 12.0;
/// The lean of a body carried at a run, forward and back, in degrees, over
/// whatever the clip's own trunk does.
const CARRIED_FORWARD: f32 = 4.0;
const CARRIED_BACK: f32 = -2.0;
/// How much of the lean is the whole body tipping at the ankles rather than
/// the trunk at the hips: the share of the hips' own height they are carried
/// ahead by. One would be a straight line from the ankle to the ear; a little
/// less keeps a rear leg at the end of its push inside its reach.
const AT_THE_ANKLES: f32 = 0.6;
/// The share of the lean the pelvis takes; the waist bends the rest.
const PELVIS: f32 = 0.35;
/// The share of the lean the head takes back, so the eyes stay on where the
/// body is going rather than the floor.
const HEAD_UP: f32 = 0.5;

/// The lean's own memory: the eased velocity and the lean on screen.
#[derive(Clone, Copy, Debug, Default)]
pub struct Lean {
    /// Strafe and forward velocity, eased, in the body's own frame.
    velocity: [f32; 2],
    /// Forward and sideways lean on screen, in degrees.
    lean: [f32; 2],
    /// How free the body is to lean, nought to one.
    gate: f32,
}

impl Lean {
    /// The lean this frame, forward and to the body's right, in degrees.
    ///
    /// `travel` is the body's strafe and forward velocity in metres a second
    /// (`PoseInput::travel`), `free` whether it is on its feet and doing
    /// nothing but moving -- the only time the legs are a walk or a run.
    pub fn step(&mut self, travel: [f32; 2], free: bool, dt: f32) -> [f32; 2] {
        // The eased velocity's rate of change is how far it is behind over how
        // long it takes to catch up.
        let accel = [
            (travel[0] - self.velocity[0]) / EASE_VELOCITY,
            (travel[1] - self.velocity[1]) / EASE_VELOCITY,
        ];
        let k = 1.0 - (-dt / EASE_VELOCITY).exp();
        for (v, t) in self.velocity.iter_mut().zip(travel) {
            *v += (t - *v) * k;
        }

        let carried = (self.velocity[1] / crate::play::RUN_AT).clamp(-1.0, 1.0) * CARRIED_FORWARD;
        let carried = carried.max(CARRIED_BACK);
        let into = |a: f32, most_ahead: f32, most_behind: f32| {
            let t = (a / HARD).tanh();
            t * if t > 0.0 { most_ahead } else { most_behind }
        };
        let want = [
            carried + into(accel[1], MOST_FORWARD, MOST_BACK),
            into(accel[0], MOST_SIDE, MOST_SIDE),
        ];
        let k = 1.0 - (-dt / EASE_LEAN).exp();
        for (l, w) in self.lean.iter_mut().zip(want) {
            *l += (w - *l) * k;
        }
        let k = 1.0 - (-dt / EASE_GATE).exp();
        self.gate += (if free { 1.0 } else { 0.0 } - self.gate) * k;
        [self.lean[0] * self.gate, self.lean[1] * self.gate]
    }
}

/// Tip a pose forward by `forward` degrees and to its right by `right`, from
/// the ankles, with its feet left where they were.
pub fn lean_over(pose: Pose, forward: f32, right: f32) -> Pose {
    if forward.abs() < 0.05 && right.abs() < 0.05 {
        return pose;
    }
    let skeleton = reference();
    let before = solve(skeleton, &pose);
    let feet = [
        before.origin[Joint::FootL.index()],
        before.origin[Joint::FootR.index()],
    ];
    let soles = [
        sole(&pose, &before, Joint::FootL),
        sole(&pose, &before, Joint::FootR),
    ];

    // The hips' own height over the floor this frame.
    let height = before.origin[Joint::Root.index()][1];
    // A leg at the end of its reach -- a front leg landing straight while
    // the body sits back, a rear one finishing its push -- cannot follow hips
    // carried away from it. First the hips sink by what it is short, as a
    // body bends its knees to brake; where that is not enough (a hip at the
    // end of its swing), the lean backs off until both feet are where they
    // were. Almost always the first try is the one.
    let mut out = pose;
    let (mut share, mut sink) = (1.0, 0.0);
    for _ in 0..5 {
        out = tipped(pose, forward * share, right * share, height);
        out.channels[1] -= sink;
        let short = crate::ik::foot_to(&mut out, skeleton, true, feet[0])
            .max(crate::ik::foot_to(&mut out, skeleton, false, feet[1]));
        if short < 0.002 {
            break;
        }
        if sink == 0.0 {
            sink = short;
        } else {
            share *= 0.6;
        }
    }
    // The leg under a sole has turned, so the ankle turns back by as much: a
    // flat foot stays flat, a heel lifted stays lifted.
    let after = solve(skeleton, &out);
    for (foot, was) in [Joint::FootL, Joint::FootR].into_iter().zip(soles) {
        let bone = skeleton.bone(foot);
        let now = sole(&out, &after, foot);
        let mut a = out.angles(foot);
        let (lo, hi) = bone.limits.swing;
        a[0] = (a[0] + (was - now) / bone.swing_sign).clamp(lo, hi);
        out.set_angles(foot, a);
    }
    out
}

/// The pose leaned, before its feet are put back: the hips carried over the
/// floor, the pelvis and the trunk tipped, the head lifted back toward level.
fn tipped(pose: Pose, forward: f32, right: f32, height: f32) -> Pose {
    let mut out = pose;
    out.channels[0] += height * right.to_radians().sin() * AT_THE_ANKLES;
    out.channels[2] += height * forward.to_radians().sin() * AT_THE_ANKLES;
    // The pelvis takes a share and the trunk the rest: a pelvis tipped the
    // whole way would ask the rear hip for more extension than it has. The
    // root's spread leans the body to its left when positive.
    let (forward, right) = (forward.to_radians(), right.to_radians());
    let root = out.angles(Joint::Root);
    out.set_angles(
        Joint::Root,
        [
            root[0] + forward * PELVIS,
            root[1] - right * PELVIS,
            root[2],
        ],
    );
    let spine = out.angles(Joint::Spine);
    out.set_angles(
        Joint::Spine,
        [
            spine[0] + forward * (1.0 - PELVIS),
            spine[1] - right * (1.0 - PELVIS),
            spine[2],
        ],
    );
    let head = out.angles(Joint::Head);
    out.set_angles(Joint::Head, [head[0] - forward * HEAD_UP, head[1], head[2]]);
    out
}

/// How far a sole is pitched off flat, radians: the ankle's own swing against
/// the one that would level it under its shin (`Pose::level_l`).
fn sole(pose: &Pose, skin: &crate::skeleton::Skin, foot: Joint) -> f32 {
    let skeleton = reference();
    let shin = skin.rot[foot.parent().expect("a foot hangs off a shin").index()];
    let flat = shin.rotate([0.0, 0.0, 1.0])[1].atan2(shin.rotate([0.0, 1.0, 0.0])[1]);
    pose.angles(foot)[0] * skeleton.bone(foot).swing_sign - flat
}
