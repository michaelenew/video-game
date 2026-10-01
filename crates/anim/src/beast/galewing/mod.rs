//! The Galewing's animation: its recipes, and the builders that know it.
//!
//! The recipes are in [`clips`]. This file is what the factory in `super`
//! needs from the bird in particular -- which of its bones are body, neck,
//! tail, wing and leg, the pose it stands in, where its table is baked to --
//! and the pose builders its recipes are written with, as a trait on the
//! shared [`Pose`].
//!
//! **A wing is three bones on a side**: the shoulder (the root, flush with
//! the back), the arm (the blade) and the hand (the tip). Lifting a wing is
//! a roll of the shoulder about the body's length, folding it back is a yaw;
//! both are written once, for "the wing", and turned into each side's sign
//! here (`wings`, `fold`), so a symmetric pose is a symmetric call.

pub mod clips;

use super::{Authored, Group, Pose, Recipe};
use sim::species::Species;
use sim::species::galewing::{self as gw, Clip, bones};

/// The Galewing, as the animation factory sees it.
pub struct Galewing;

pub static GALEWING: Galewing = Galewing;

impl Authored for Galewing {
    fn species(&self) -> &'static Species {
        &gw::SPECIES
    }

    fn recipes(&self) -> Vec<Recipe> {
        clips::all()
    }

    fn group_of(&self, bone: usize) -> (Group, u8) {
        group_of(bone)
    }

    fn standing(&self) -> Pose {
        <Pose as BirdPose>::standing()
    }

    fn baked_path(&self) -> &'static str {
        "crates/sim/src/species/galewing/baked.rs"
    }
}

/// Where a point inside one phase of an attack falls in the whole clip. See
/// [`super::mark`].
pub fn mark(clip: Clip, phase: u8, u: f32) -> f32 {
    super::mark(&gw::SPECIES, clip.index(), phase, u)
}

/// The group and the depth of each bone. The wings are the "tail" group:
/// the loosest things on the bird, trailing the body they hang from.
pub fn group_of(bone: usize) -> (Group, u8) {
    use bones::*;
    match bone {
        ROOT => (Group::Body, 0),
        CHEST => (Group::Body, 1),
        NECK => (Group::Neck, 0),
        HEAD => (Group::Neck, 1),
        TAIL => (Group::Tail, 0),
        SHOULDER_L | SHOULDER_R => (Group::Tail, 0),
        ARM_L | ARM_R => (Group::Tail, 1),
        HAND_L | HAND_R => (Group::Tail, 2),
        THIGH_L | THIGH_R => (Group::Legs, 0),
        _ => (Group::Legs, 1),
    }
}

/// The wing bones of a side: shoulder, arm, hand. `side` 0 is the left.
pub const fn wing(side: usize) -> [usize; 3] {
    if side == 0 {
        [bones::SHOULDER_L, bones::ARM_L, bones::HAND_L]
    } else {
        [bones::SHOULDER_R, bones::ARM_R, bones::HAND_R]
    }
}

/// The sign that turns "up" or "back" into a side's own roll or yaw: the
/// left wing lies along `-z`, so lifting it is a positive roll and folding
/// it back a negative yaw; the right is the other way round.
const fn sign(side: usize) -> f32 {
    if side == 0 { 1.0 } else { -1.0 }
}

/// Its pose builders.
pub trait BirdPose: Sized {
    fn root(self, pitch: f32, yaw: f32, roll: f32) -> Pose;
    fn chest(self, pitch: f32, yaw: f32, roll: f32) -> Pose;
    fn neck(self, pitch: f32, yaw: f32) -> Pose;
    fn head(self, pitch: f32, yaw: f32, roll: f32) -> Pose;
    fn tail(self, lift: f32, yaw: f32) -> Pose;
    /// One wing: the shoulder lifted `up` and folded `back`, the blade and
    /// the tip lifted and folded on from there.
    fn wing(self, side: usize, up: [f32; 3], back: [f32; 3]) -> Pose;
    /// Both wings the same.
    fn wings(self, up: [f32; 3], back: [f32; 3]) -> Pose;
    /// Both wings twisted about their own chord: leading edge up is
    /// positive.
    fn twist(self, by: f32) -> Pose;
    /// Folded along its flanks, a Z under each other: standing.
    fn folded(self) -> Pose;
    /// Legs drawn up and back under the tail: flying.
    fn tucked(self) -> Pose;
    fn leg(self, which: usize, swing: f32, knee: f32) -> Pose;
    fn legs(self, swing: f32, knee: f32) -> Pose;
    fn plant(self, which: usize, reach: f32, clear: f32) -> Pose;
    fn plant_both(self, reach: f32, clear: f32) -> Pose;
    fn standing() -> Pose;
    fn soaring() -> Pose;
}

impl BirdPose for Pose {
    fn root(self, pitch: f32, yaw: f32, roll: f32) -> Pose {
        self.set(bones::ROOT, pitch, yaw, roll)
    }
    fn chest(self, pitch: f32, yaw: f32, roll: f32) -> Pose {
        self.set(bones::CHEST, pitch, yaw, roll)
    }

    /// The neck as a curve: most of it at the base.
    fn neck(mut self, pitch: f32, yaw: f32) -> Pose {
        for (bone, k) in [(bones::NECK, 0.65), (bones::HEAD, 0.35)] {
            self.bone[bone][0] += pitch * k;
            self.bone[bone][1] += yaw * k;
        }
        self
    }

    fn head(self, pitch: f32, yaw: f32, roll: f32) -> Pose {
        let [p, y, r] = self.bone[bones::HEAD];
        self.set(bones::HEAD, p + pitch, y + yaw, r + roll)
    }

    /// **Positive `lift` raises the tail**: it extends backwards from its
    /// bone, so the rig's nose-up pitch points it at the floor.
    fn tail(self, lift: f32, yaw: f32) -> Pose {
        self.set(bones::TAIL, -lift, yaw, 0.0)
    }

    fn wing(mut self, side: usize, up: [f32; 3], back: [f32; 3]) -> Pose {
        let s = sign(side);
        for (i, bone) in wing(side).into_iter().enumerate() {
            self.bone[bone][2] = up[i] * s;
            self.bone[bone][1] = -back[i] * s;
        }
        self
    }

    fn wings(self, up: [f32; 3], back: [f32; 3]) -> Pose {
        self.wing(0, up, back).wing(1, up, back)
    }

    fn twist(mut self, by: f32) -> Pose {
        for side in 0..2 {
            for bone in wing(side) {
                self.bone[bone][0] += by;
            }
        }
        self
    }

    /// The Z-fold: the root back along the flank, the blade forward along
    /// it, the tip back again -- lifted a little so the three lie on top of
    /// each other rather than through each other.
    fn folded(self) -> Pose {
        self.wings([12.0, 8.0, 6.0], [82.0, -160.0, 150.0])
    }

    fn tucked(self) -> Pose {
        self.legs(-75.0, 40.0)
    }

    fn leg(mut self, which: usize, swing: f32, knee: f32) -> Pose {
        let (hip, shin) = if which == 0 {
            (bones::THIGH_L, bones::SHIN_L)
        } else {
            (bones::THIGH_R, bones::SHIN_R)
        };
        self.bone[hip][0] = swing;
        self.bone[shin][0] = knee;
        self
    }

    fn legs(self, swing: f32, knee: f32) -> Pose {
        self.leg(0, swing, knee).leg(1, swing, knee)
    }

    fn plant(self, which: usize, reach: f32, clear: f32) -> Pose {
        self.plant_leg(&gw::SPECIES, which, reach, clear)
    }

    fn plant_both(self, reach: f32, clear: f32) -> Pose {
        self.plant(0, reach, clear).plant(1, reach, clear)
    }

    /// **Standing**: wings folded along its back, head up on a curved neck,
    /// tail level, both feet under the hips.
    fn standing() -> Pose {
        Pose::rest(bones::COUNT)
            .folded()
            .neck(18.0, 0.0)
            .head(-14.0, 0.0, 0.0)
            .tail(4.0, 0.0)
            .plant_both(0.05, 0.0)
    }

    /// **In the air**: wings spread with a little dihedral, legs tucked, head
    /// forward on a straight neck. The cross it is from below.
    fn soaring() -> Pose {
        Pose::rest(bones::COUNT)
            .wings([6.0, 3.0, 2.0], [0.0, 0.0, 6.0])
            .neck(-6.0, 0.0)
            .head(4.0, 0.0, 0.0)
            .tucked()
    }
}
