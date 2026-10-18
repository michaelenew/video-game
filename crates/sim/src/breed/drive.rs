//! One frame of motion, as the builder reads it.
//!
//! The builder ([`super::build`]) stands a body where its posture says and
//! then bends it as a `Drive` says: how high the root rides and how far it
//! has surged, how the torso pitches, where the head is turned and how far
//! the neck reaches, whether the jaw is open, how the tail swings and
//! curls, where each foot is in its stride, how far a striking arm is
//! extended, and what the wings are doing. Nothing in it is a decision --
//! the gait ([`super::locomotion`]) and the moves ([`super::strike`]) decide, and
//! write a `Drive`; the builder only obeys. So walking, attacking and
//! flying are the same body in different drives, and a move's hit is read
//! off the very bones the drive swung.
//!
//! Units are chosen so a drive means the same on a gnawer as on a hill:
//! feet move in shares of their own standing length, angles are turns,
//! the root moves in metres (a gait or a move works those out from the
//! body's proportions).

use super::{STATIONS, fx};
use crate::fixed::Fx;

/// Where one foot is, against where it would stand.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Foot {
    /// Ahead of its standing spot, in shares of the leg's standing length.
    pub forward: Fx,
    /// Off the floor, in shares of the leg's standing length.
    pub lift: Fx,
}

/// What the wings are doing.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Wings {
    /// Folded along the body (and walked on, if the posture leans on them).
    #[default]
    Folded,
    /// Spread on the ground: a flare, a buffet.
    Flared,
    /// Spread and flying, the body lifted half its length off the floor.
    Aloft,
}

/// One frame of motion. [`Drive::STILL`] is the posture with nothing added.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Drive {
    /// The root's height over its posture's, metres.
    pub lift: Fx,
    /// The root's travel along the body's facing, metres.
    pub surge: Fx,
    /// The torso's pitch over its posture's, turns, nose up.
    pub pitch: Fx,
    /// The whole body turned about its root, turns, toward `+z`: a move
    /// that faces where it was aimed, or spins its tail round. The root's
    /// travel ([`Drive::surge`]) goes the way it faces.
    pub yaw: Fx,
    /// The neck's pitch and yaw over the posture's, turns.
    pub neck_pitch: Fx,
    pub neck_yaw: Fx,
    /// The neck's length over its own, as a share: `1/2` is half as long
    /// again, `-1/4` drawn back by a quarter.
    pub neck_reach: Fx,
    /// The jaw, nought shut to one gaping.
    pub jaw: Fx,
    /// The tail's yaw and its curl up (or down), turns per segment, added
    /// down its length so it bends rather than pivots.
    pub tail_yaw: Fx,
    pub tail_pitch: Fx,
    /// A limbless body's wave: its phase, and its sway in turns per
    /// segment. `None` lies straight.
    pub wave: Option<(Fx, Fx)>,
    /// A limbless body's front raised, turns per segment.
    pub rise: Fx,
    /// The abdomen's tilt over its own, turns.
    pub abdomen: Fx,
    /// The torso puffed out (or drawn in), as a share of its width and
    /// depth: an inflation, a bristle, a breath before a roar.
    pub swell: Fx,
    /// Each foot, by station and side (left, right).
    pub feet: [[Foot; 2]; STATIONS],
    /// A striking pair's arm, by side: `-1` cocked high and back, nought
    /// folded, one thrust out.
    pub arms: [Fx; 2],
    pub wings: Wings,
    /// The wings' beat angle when spread, turns: up positive.
    pub beat: Fx,
}

impl Drive {
    pub const STILL: Drive = Drive {
        lift: Fx::ZERO,
        surge: Fx::ZERO,
        pitch: Fx::ZERO,
        yaw: Fx::ZERO,
        neck_pitch: Fx::ZERO,
        neck_yaw: Fx::ZERO,
        neck_reach: Fx::ZERO,
        jaw: Fx::ZERO,
        tail_yaw: Fx::ZERO,
        tail_pitch: Fx::ZERO,
        wave: None,
        rise: Fx::ZERO,
        abdomen: Fx::ZERO,
        swell: Fx::ZERO,
        feet: [[Foot {
            forward: Fx::ZERO,
            lift: Fx::ZERO,
        }; 2]; STATIONS],
        arms: [Fx::ZERO; 2],
        wings: Wings::Folded,
        beat: Fx::ZERO,
    };

    /// Every number of `other` added to this one, and its wings and wave
    /// where it has them: a move laid over a gait.
    pub fn plus(&self, other: &Drive) -> Drive {
        let mut out = *self;
        out.lift = out.lift.add(other.lift);
        out.surge = out.surge.add(other.surge);
        out.pitch = out.pitch.add(other.pitch);
        out.yaw = out.yaw.add(other.yaw);
        out.neck_pitch = out.neck_pitch.add(other.neck_pitch);
        out.neck_yaw = out.neck_yaw.add(other.neck_yaw);
        out.neck_reach = out.neck_reach.add(other.neck_reach);
        out.jaw = super::unit(out.jaw.add(other.jaw));
        out.tail_yaw = out.tail_yaw.add(other.tail_yaw);
        out.tail_pitch = out.tail_pitch.add(other.tail_pitch);
        if other.wave.is_some() {
            out.wave = other.wave;
        }
        out.rise = out.rise.add(other.rise);
        out.abdomen = out.abdomen.add(other.abdomen);
        out.swell = out.swell.add(other.swell);
        for i in 0..STATIONS {
            for s in 0..2 {
                out.feet[i][s].forward = out.feet[i][s].forward.add(other.feet[i][s].forward);
                out.feet[i][s].lift = out.feet[i][s].lift.add(other.feet[i][s].lift);
            }
        }
        for s in 0..2 {
            out.arms[s] = out.arms[s].add(other.arms[s]);
        }
        if other.wings != Wings::Folded {
            out.wings = other.wings;
        }
        out.beat = out.beat.add(other.beat);
        out
    }

    /// This drive, scaled toward [`Drive::STILL`] by `k` (nought to one):
    /// how far into a move's pose a frame is. Wings and the wave are kept
    /// once `k` passes a half.
    pub fn scaled(&self, k: Fx) -> Drive {
        let s = |v: Fx| v.mul(k);
        let mut out = Drive::STILL;
        out.lift = s(self.lift);
        out.surge = s(self.surge);
        out.pitch = s(self.pitch);
        out.yaw = s(self.yaw);
        out.neck_pitch = s(self.neck_pitch);
        out.neck_yaw = s(self.neck_yaw);
        out.neck_reach = s(self.neck_reach);
        out.jaw = s(self.jaw);
        out.tail_yaw = s(self.tail_yaw);
        out.tail_pitch = s(self.tail_pitch);
        out.rise = s(self.rise);
        out.abdomen = s(self.abdomen);
        out.swell = s(self.swell);
        for i in 0..STATIONS {
            for side in 0..2 {
                out.feet[i][side].forward = s(self.feet[i][side].forward);
                out.feet[i][side].lift = s(self.feet[i][side].lift);
            }
        }
        for side in 0..2 {
            out.arms[side] = s(self.arms[side]);
        }
        if k > fx(1, 2) {
            out.wings = self.wings;
            out.wave = self.wave;
        }
        out.beat = s(self.beat);
        out
    }

    /// The straight line from `a` to `b`, `k` of the way (nought to one).
    pub fn lerp(a: &Drive, b: &Drive, k: Fx) -> Drive {
        let inv = Fx::ONE.sub(k);
        a.scaled(inv)
            .plus(&b.scaled(k))
            .with_discrete(if k > fx(1, 2) { b } else { a })
    }

    /// The wings and wave of `from`, the numbers of this.
    fn with_discrete(mut self, from: &Drive) -> Drive {
        self.wings = from.wings;
        self.wave = from.wave;
        self
    }
}

impl Default for Drive {
    fn default() -> Self {
        Drive::STILL
    }
}
