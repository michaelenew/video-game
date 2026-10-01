//! The Siegeshell's animation: its recipes, and the builders that know a
//! shell on six towers.
//!
//! The recipes are in [`clips`]. This file is what the factory in `super`
//! needs from it in particular -- which of its bones are body, head and leg,
//! the pose it stands in, and where its table is baked to.
//!
//! **The legs are never in a clip.** Every foot is placed by the simulation
//! from the stride, every frame (`sim::species::siegeshell::gait::repose`),
//! because a foot is collision geometry and the place a ring comes from. The
//! clips move the shell, its two sides, the crown, the neck and the head --
//! the parts a fighter rides and reads -- and leave every leg bone at rest.

pub mod clips;

use super::{Authored, Group, Pose, Recipe};
use sim::species::Species;
use sim::species::siegeshell::{self, bones};

/// The Siegeshell, as the animation factory sees it.
pub struct Siegeshell;

pub static SIEGESHELL: Siegeshell = Siegeshell;

impl Authored for Siegeshell {
    fn species(&self) -> &'static Species {
        &siegeshell::SPECIES
    }

    fn recipes(&self) -> Vec<Recipe> {
        clips::all()
    }

    fn group_of(&self, bone: usize) -> (Group, u8) {
        group_of(bone)
    }

    fn standing(&self) -> Pose {
        Pose::rest(bones::COUNT)
    }

    fn baked_path(&self) -> &'static str {
        "crates/sim/src/species/siegeshell/baked.rs"
    }
}

/// Where a point inside one phase of an attack falls in the whole clip.
pub fn mark(clip: siegeshell::Clip, phase: u8, u: f32) -> f32 {
    super::mark(&siegeshell::SPECIES, clip.index(), phase, u)
}

/// The group and depth of each bone. The shell's sides and the crown hang
/// off the body one deep; the neck and the head are the neck; the legs are
/// the gait's and never move in a clip.
pub fn group_of(bone: usize) -> (Group, u8) {
    use bones::*;
    match bone {
        ROOT => (Group::Body, 0),
        SHELL_L | SHELL_R | CROWN => (Group::Body, 1),
        NECK => (Group::Neck, 0),
        HEAD => (Group::Neck, 1),
        b => (Group::Legs, ((b - LEG0) % 4) as u8),
    }
}

/// Its pose builders.
pub trait ShellPose: Sized {
    /// The whole shell: pitch (nose up), yaw, roll, in degrees.
    fn body(self, pitch: f32, yaw: f32, roll: f32) -> Pose;
    /// One side of the shell rolled about its hinge at the plateau's edge:
    /// positive lifts that side's rim.
    fn side(self, right: bool, lift: f32) -> Pose;
    /// The crown: pitch and roll, in degrees.
    fn crown(self, pitch: f32, roll: f32) -> Pose;
    /// The neck and the head: pitch each (up is positive), and the head's yaw.
    fn neck(self, neck: f32, head: f32, yaw: f32) -> Pose;
}

impl ShellPose for Pose {
    fn body(self, pitch: f32, yaw: f32, roll: f32) -> Pose {
        self.set(bones::ROOT, pitch, yaw, roll)
    }
    fn side(mut self, right: bool, lift: f32) -> Pose {
        // A positive roll lowers the creature's right; the left is the
        // other way round.
        if right {
            self.bone[bones::SHELL_R][2] = -lift;
        } else {
            self.bone[bones::SHELL_L][2] = lift;
        }
        self
    }
    fn crown(mut self, pitch: f32, roll: f32) -> Pose {
        self.bone[bones::CROWN][0] = pitch;
        self.bone[bones::CROWN][2] = roll;
        self
    }
    fn neck(mut self, neck: f32, head: f32, yaw: f32) -> Pose {
        self.bone[bones::NECK][0] = neck;
        self.bone[bones::HEAD][0] = head;
        self.bone[bones::HEAD][1] = yaw;
        self
    }
}
