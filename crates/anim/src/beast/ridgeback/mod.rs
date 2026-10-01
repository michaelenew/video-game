//! The Ridgeback's animation: its recipes, and the builders that know its
//! anatomy.
//!
//! The recipes are in [`clips`]. This file is what the factory in `super`
//! needs from the Ridgeback in particular -- which of its bones are body, neck,
//! tail and leg, the pose it stands in, and where its table is baked to -- and
//! the pose builders its recipes are written with, as a trait on the shared
//! [`Pose`] so another species' builders can have the same names without
//! meeting these.

pub mod clips;

use super::{Authored, Group, Pose, Recipe};
use sim::species::Species;
use sim::species::ridgeback::{self, Clip, bones};

/// The Ridgeback, as the animation factory sees it.
pub struct Ridgeback;

pub static RIDGEBACK: Ridgeback = Ridgeback;

impl Authored for Ridgeback {
    fn species(&self) -> &'static Species {
        &ridgeback::SPECIES
    }

    fn recipes(&self) -> Vec<Recipe> {
        clips::all()
    }

    fn group_of(&self, bone: usize) -> (Group, u8) {
        group_of(bone)
    }

    fn standing(&self) -> Pose {
        <Pose as RidgebackPose>::standing()
    }

    fn baked_path(&self) -> &'static str {
        "crates/sim/src/species/ridgeback/baked.rs"
    }
}

/// Where a point inside one phase of an attack falls in the whole clip. See
/// [`super::mark`].
pub fn mark(clip: Clip, phase: u8, u: f32) -> f32 {
    super::mark(&ridgeback::SPECIES, clip.index(), phase, u)
}

/// The spine, nose to tail, as bone indices -- the chain most things bend.
pub const SPINE_CHAIN: [usize; 6] = [
    bones::ROOT,
    bones::SPINE,
    bones::CHEST,
    bones::NECK,
    bones::NECK2,
    bones::HEAD,
];

pub const TAIL_CHAIN: [usize; 4] = [bones::TAIL1, bones::TAIL2, bones::TAIL3, bones::TAIL4];

/// `(hip, knee)` per leg, in `ridgeback::LEGS` order: front left, front right,
/// hind left, hind right.
pub const LEG_BONES: [(usize, usize); 4] = [
    (bones::SHOULDER_L, bones::FOREARM_L),
    (bones::SHOULDER_R, bones::FOREARM_R),
    (bones::THIGH_L, bones::SHIN_L),
    (bones::THIGH_R, bones::SHIN_R),
];

/// The group and the depth of each bone: depth is how far down its own chain it
/// sits, and it is what makes the tip of the tail trail the base of it without
/// anyone keying that.
pub fn group_of(bone: usize) -> (Group, u8) {
    use bones::*;
    match bone {
        ROOT => (Group::Body, 0),
        SPINE => (Group::Body, 1),
        CHEST => (Group::Body, 2),
        NECK => (Group::Neck, 0),
        NECK2 => (Group::Neck, 1),
        HEAD => (Group::Neck, 2),
        TAIL1 => (Group::Tail, 0),
        TAIL2 => (Group::Tail, 1),
        TAIL3 => (Group::Tail, 2),
        TAIL4 => (Group::Tail, 3),
        SHOULDER_L | SHOULDER_R | THIGH_L | THIGH_R => (Group::Legs, 0),
        _ => (Group::Legs, 1),
    }
}

/// The Ridgeback's pose builders. The mirror lives in the skeleton, so
/// `forelegs(-20.0, 30.0)` puts *both* forelegs in the same shape.
pub trait RidgebackPose: Sized {
    fn root(self, pitch: f32, yaw: f32, roll: f32) -> Pose;
    fn spine(self, pitch: f32, yaw: f32, roll: f32) -> Pose;
    fn chest(self, pitch: f32, yaw: f32, roll: f32) -> Pose;
    fn head(self, pitch: f32, yaw: f32, roll: f32) -> Pose;
    fn neck(self, pitch: f32, yaw: f32) -> Pose;
    fn tail(self, lift: f32, yaw: f32) -> Pose;
    fn tail_from_base(self, lift: f32) -> Pose;
    fn tail_swing_base(self, yaw: f32) -> Pose;
    fn tail_roll(self, roll: f32) -> Pose;
    fn tail_lift(self, lift: f32) -> Pose;
    fn leg(self, which: usize, swing: f32, knee: f32) -> Pose;
    fn leg_spread(self, which: usize, spread: f32) -> Pose;
    fn forelegs(self, swing: f32, knee: f32) -> Pose;
    fn hindlegs(self, swing: f32, knee: f32) -> Pose;
    fn standing() -> Pose;
    fn plant(self, which: usize, reach: f32, clear: f32) -> Pose;
    fn plant_fore(self, reach: f32, clear: f32) -> Pose;
    fn plant_hind(self, reach: f32, clear: f32) -> Pose;
}

impl RidgebackPose for Pose {
    fn root(self, pitch: f32, yaw: f32, roll: f32) -> Pose {
        self.set(bones::ROOT, pitch, yaw, roll)
    }
    fn spine(self, pitch: f32, yaw: f32, roll: f32) -> Pose {
        self.set(bones::SPINE, pitch, yaw, roll)
    }
    fn chest(self, pitch: f32, yaw: f32, roll: f32) -> Pose {
        self.set(bones::CHEST, pitch, yaw, roll)
    }
    fn head(self, pitch: f32, yaw: f32, roll: f32) -> Pose {
        self.set(bones::HEAD, pitch, yaw, roll)
    }

    /// Bend the neck as a **curve** rather than a hinge: the amount is shared
    /// across both neck bones and the head, weighted toward the base.
    ///
    /// Every early version of the bite had the neck as one rigid bar swinging
    /// from the shoulder, which reads as a digger rather than as an animal.
    /// Sharing the angle is the whole fix and it belongs here rather than in
    /// each key.
    fn neck(mut self, pitch: f32, yaw: f32) -> Pose {
        let share = [0.45, 0.35, 0.20];
        for (bone, k) in [bones::NECK, bones::NECK2, bones::HEAD]
            .into_iter()
            .zip(share)
        {
            self.bone[bone][0] += pitch * k;
            self.bone[bone][1] += yaw * k;
        }
        self
    }

    /// Lift the whole tail and swing it, spread along its length and growing
    /// toward the tip -- which is what a tail does and what makes the tip carry
    /// the speed.
    ///
    /// **Positive `lift` raises the tail**, which is the opposite sign to the
    /// rig's own pitch: the tail extends *backwards* from its bones, so nose-up
    /// on a tail bone points the tail at the sky. The flip lives here rather
    /// than in every key, the same way the left-right mirror lives in the
    /// skeleton rather than in every pose.
    fn tail(mut self, lift: f32, yaw: f32) -> Pose {
        // The increments sum to one, so `tail(0.0, 60.0)` means sixty degrees
        // at the tip and the bend is spread rather than hinged at one joint.
        let share = [0.18, 0.24, 0.28, 0.30];
        for (bone, k) in TAIL_CHAIN.into_iter().zip(share) {
            self.bone[bone][0] -= lift * k;
            self.bone[bone][1] += yaw * k;
        }
        self
    }

    /// Lift or drop the tail **from its root**, so the whole thing swings as one
    /// beam rather than curling.
    ///
    /// Different from [`tail`](Pose::tail) in the place it matters: `tail`
    /// spreads the bend toward the tip, which is a whip, and this puts it all at
    /// the base, which is the tail being *carried* somewhere.
    fn tail_from_base(mut self, lift: f32) -> Pose {
        self.bone[bones::TAIL1][0] -= lift;
        self
    }

    /// Swing the whole tail about its **root**, so it travels as one beam.
    ///
    /// The difference from [`tail`](Pose::tail) is the whole of what a sweep
    /// feels like to ride. `tail` spreads the bend toward the tip, which whips
    /// the tip and leaves the base almost still -- so the tip carries the speed
    /// the hitbox needs, and somebody standing on the base feels nothing.
    /// Swinging from the root moves the base too, which is what throws them.
    fn tail_swing_base(mut self, yaw: f32) -> Pose {
        self.bone[bones::TAIL1][1] += yaw;
        self
    }

    /// Roll the tail about its own length, from the root.
    ///
    /// What tips a rider off it. A tail that only yaws carries somebody
    /// standing on it round in an arc, which they ride; one that rolls takes
    /// the floor out from under them, which they do not.
    fn tail_roll(mut self, roll: f32) -> Pose {
        self.bone[bones::TAIL1][2] += roll;
        self
    }

    /// Curl the tail up or down without swinging it. Positive is up.
    fn tail_lift(self, lift: f32) -> Pose {
        self.tail(lift, 0.0)
    }

    /// One leg, by index into [`LEG_BONES`].
    fn leg(mut self, which: usize, swing: f32, knee: f32) -> Pose {
        let (hip, shin) = LEG_BONES[which];
        self.bone[hip][0] = swing;
        self.bone[shin][0] = knee;
        self
    }

    /// Spread a leg out from under the body.
    fn leg_spread(mut self, which: usize, spread: f32) -> Pose {
        let (hip, _) = LEG_BONES[which];
        // Positive is away from the midline on both sides, the same convention
        // the fighters' skeleton uses. `SIDE` carries the mirror.
        self.bone[hip][2] = spread * ridgeback::BONES[hip].side as f32;
        self
    }

    fn forelegs(self, swing: f32, knee: f32) -> Pose {
        self.leg(0, swing, knee).leg(1, swing, knee)
    }

    fn hindlegs(self, swing: f32, knee: f32) -> Pose {
        self.leg(2, swing, knee).leg(3, swing, knee)
    }

    /// The stance an animal stands in. Not the zero pose: with every channel at
    /// zero the legs are straight poles, and nothing with a knee stands like
    /// that.
    ///
    /// The feet are **solved onto the floor** rather than angled toward it, so
    /// that every clip built on this one starts from four feet actually on the
    /// ground. The forefeet sit a little ahead of the shoulder and the hind
    /// feet a little behind the hip, which is where a standing quadruped puts
    /// them and is what gives the legs somewhere to bend.
    fn standing() -> Pose {
        Pose::rest(bones::COUNT)
            .tail_lift(4.0)
            .plant_fore(0.16, 0.0)
            .plant_hind(-0.24, 0.0)
    }

    /// **Put a foot on the floor**, and solve the leg for it: leg `which` in
    /// `ridgeback::LEGS` order. See [`Pose::plant_leg`].
    fn plant(self, which: usize, reach: f32, clear: f32) -> Pose {
        self.plant_leg(&ridgeback::SPECIES, which, reach, clear)
    }

    /// Plant both front feet, or both back ones.
    fn plant_fore(self, reach: f32, clear: f32) -> Pose {
        self.plant(0, reach, clear).plant(1, reach, clear)
    }

    fn plant_hind(self, reach: f32, clear: f32) -> Pose {
        self.plant(2, reach, clear).plant(3, reach, clear)
    }
}
