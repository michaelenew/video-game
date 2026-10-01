//! The Mantis's animation: its recipes, and the builders that know a mantis.
//!
//! The recipes are in [`clips`]. This file is what the factory in `super`
//! needs from the Mantis in particular -- which of its bones are body, neck,
//! "tail" and leg, the pose it stands in, where its table is baked to -- and
//! the pose builders its recipes are written with, as a trait on the shared
//! [`Pose`].
//!
//! **The silhouette is the arms.** The fight is read from four stances --
//! guard, coil, Ready, prayer (`creatures/mantis.md` §6) -- and every one of
//! them is a different place for the two raptorial arms: crossed before the
//! head, swept back over the shoulders, one cocked out wide, folded tight in
//! front. So the builders speak in arms: [`MantisPose::arm`] puts one at a
//! raise, a swing out from the body and a fold at the elbow, and the left is
//! always the right's mirror (the rig does not mirror for us: a yaw toward
//! the body's own right is a yaw toward the right whichever arm it is on).
//!
//! **The legs are planted** with the factory's two-bone solve
//! (`Pose::plant_leg`), so a body that drops into the coil bends its knees
//! rather than sinking its feet.

pub mod clips;

use super::{Authored, Group, Pose, Recipe};
use sim::species::Species;
use sim::species::mantis::{self, Clip, bones};

/// The Mantis, as the animation factory sees it.
pub struct Mantis;

pub static MANTIS: Mantis = Mantis;

impl Authored for Mantis {
    fn species(&self) -> &'static Species {
        &mantis::SPECIES
    }

    fn recipes(&self) -> Vec<Recipe> {
        clips::all()
    }

    fn group_of(&self, bone: usize) -> (Group, u8) {
        group_of(bone)
    }

    fn standing(&self) -> Pose {
        <Pose as MantisPose>::standing()
    }

    fn baked_path(&self) -> &'static str {
        "crates/sim/src/species/mantis/baked.rs"
    }
}

/// Where a point inside one phase of an attack falls in the whole clip.
pub fn mark(clip: Clip, phase: u8, u: f32) -> f32 {
    super::mark(&mantis::SPECIES, clip.index(), phase, u)
}

/// The group and depth of each bone. The neck and head are the neck; the
/// arms are the "tail" -- the loose ends whose follow-through is the read --
/// two deep, blade after arm; the abdomen and the wings are tail too, the
/// heavy and the light ends that lag the body; the legs are legs.
pub fn group_of(bone: usize) -> (Group, u8) {
    use bones::*;
    match bone {
        ROOT => (Group::Body, 0),
        THORAX => (Group::Body, 1),
        NECK => (Group::Neck, 0),
        HEAD => (Group::Neck, 1),
        ABDOMEN => (Group::Tail, 0),
        ARM_L | ARM_R => (Group::Tail, 0),
        BLADE_L | BLADE_R => (Group::Tail, 1),
        WING_L | WING_R => (Group::Tail, 1),
        FEMUR_ML | FEMUR_MR | FEMUR_HL | FEMUR_HR => (Group::Legs, 0),
        _ => (Group::Legs, 1),
    }
}

/// Which side an arm or wing is on: the right is `1`, the left `-1`.
pub const RIGHT: f32 = 1.0;
pub const LEFT: f32 = -1.0;

/// Where each leg stands, forward of its hip, in metres: the middle pair a
/// little ahead, the hind pair behind.
pub const STANCE: [f32; 4] = [0.18, 0.18, -0.28, -0.28];

/// A mantis's pose builders.
pub trait MantisPose: Sized {
    fn root(self, pitch: f32, yaw: f32, roll: f32) -> Pose;
    /// The thorax: negative pitch leans it forward over its legs.
    fn thorax(self, pitch: f32, yaw: f32, roll: f32) -> Pose;
    /// The head and neck together, the neck taking most of it.
    fn head(self, pitch: f32, yaw: f32, roll: f32) -> Pose;
    /// The abdomen's tip raised by `lift` degrees.
    fn abdomen(self, lift: f32) -> Pose;
    /// One arm: raised by `raise`, swung `out` from the body (toward its own
    /// side), and the blade folded back down along it by `fold`. `side` is
    /// [`RIGHT`] or [`LEFT`].
    fn arm(self, side: f32, raise: f32, out: f32, fold: f32) -> Pose;
    /// Both arms the same, mirrored.
    fn arms(self, raise: f32, out: f32, fold: f32) -> Pose;
    /// Both wings, opened by `open` degrees off the back and lifted by `lift`.
    fn wings(self, open: f32, lift: f32) -> Pose;
    /// Drop the body by `down` metres and plant every foot.
    fn crouch(self, down: f32) -> Pose;
    /// Plant one leg: its foot `reach` forward of its hip, `clear` off the
    /// floor.
    fn plant(self, which: usize, reach: f32, clear: f32) -> Pose;
    /// Plant every leg where it stands.
    fn planted(self) -> Pose;
    fn standing() -> Pose;
}

impl MantisPose for Pose {
    fn root(self, pitch: f32, yaw: f32, roll: f32) -> Pose {
        self.set(bones::ROOT, pitch, yaw, roll)
    }

    fn thorax(self, pitch: f32, yaw: f32, roll: f32) -> Pose {
        self.set(bones::THORAX, pitch, yaw, roll)
    }

    fn head(mut self, pitch: f32, yaw: f32, roll: f32) -> Pose {
        self.bone[bones::NECK] = [pitch * 0.6, yaw * 0.6, roll * 0.5];
        self.bone[bones::HEAD] = [pitch * 0.4, yaw * 0.4, roll * 0.5];
        self
    }

    fn abdomen(mut self, lift: f32) -> Pose {
        // The abdomen points backwards along -x, so raising its tip is a
        // nose-down pitch.
        self.bone[bones::ABDOMEN][0] = -lift;
        self
    }

    fn arm(mut self, side: f32, raise: f32, out: f32, fold: f32) -> Pose {
        let (arm, blade) = if side > 0.0 {
            (bones::ARM_R, bones::BLADE_R)
        } else {
            (bones::ARM_L, bones::BLADE_L)
        };
        self.bone[arm] = [raise, out * side, 0.0];
        self.bone[blade] = [-fold, 0.0, 0.0];
        self
    }

    fn arms(self, raise: f32, out: f32, fold: f32) -> Pose {
        self.arm(RIGHT, raise, out, fold)
            .arm(LEFT, raise, out, fold)
    }

    fn wings(mut self, open: f32, lift: f32) -> Pose {
        // Folded along the back they point back along -x; opening swings
        // each out to its own side.
        self.bone[bones::WING_R] = [-lift, -open, 0.0];
        self.bone[bones::WING_L] = [-lift, open, 0.0];
        self
    }

    fn crouch(self, down: f32) -> Pose {
        let x = self.hips[0];
        let z = self.hips[2];
        self.hips(x, -down, z).planted()
    }

    fn plant(self, which: usize, reach: f32, clear: f32) -> Pose {
        self.plant_leg(&mantis::SPECIES, which, reach, clear)
    }

    fn planted(self) -> Pose {
        (0..4).fold(self, |p, leg| p.plant(leg, STANCE[leg], 0.0))
    }

    /// How a mantis stands: upright, the arms folded before its chest, the
    /// blades hanging down in front, the wings flat along its back, the
    /// abdomen level, every foot on the floor.
    fn standing() -> Pose {
        Pose::rest(bones::COUNT)
            .thorax(-4.0, 0.0, 0.0)
            .head(-6.0, 0.0, 0.0)
            .arms(22.0, 6.0, 158.0)
            .planted()
    }
}
