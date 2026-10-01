//! The Sandmaw's animation: its recipes, and the builders that know a worm.
//!
//! The recipes are in [`clips`]. This file is what the factory in `super`
//! needs from the Sandmaw in particular -- which of its bones are body, neck
//! and tail, the pose it stands in, and where its table is baked to -- and
//! the pose builders its recipes are written with, as a trait on the shared
//! [`Pose`].
//!
//! **Three places a worm can be**, and every clip is a journey between them:
//!
//! - **Under** ([`SandmawPose::under`]): lying straight a metre and a half
//!   under the sand, carried five and a half metres back by the hips so the
//!   head is at the creature's origin. Every part's box below the floor.
//! - **Standing** ([`SandmawPose::standing`]): the root at the hole half a
//!   metre down, the column leaning a third of a turn forward out of it, the
//!   body behind going down steeply so it stays under.
//! - **Beached** ([`SandmawPose::beached`]): lying on the sand, its middle line
//!   a metre up, its back at 2.2 m.

pub mod clips;

use super::{Authored, Group, Pose, Recipe};
use sim::species::Species;
use sim::species::sandmaw::{self, bones};

/// The Sandmaw, as the animation factory sees it.
pub struct Sandmaw;

pub static SANDMAW: Sandmaw = Sandmaw;

impl Authored for Sandmaw {
    fn species(&self) -> &'static Species {
        &sandmaw::SPECIES
    }

    fn recipes(&self) -> Vec<Recipe> {
        clips::all()
    }

    fn group_of(&self, bone: usize) -> (Group, u8) {
        group_of(bone)
    }

    fn standing(&self) -> Pose {
        <Pose as SandmawPose>::standing()
    }

    fn baked_path(&self) -> &'static str {
        "crates/sim/src/species/sandmaw/baked.rs"
    }
}

/// Where a point inside one phase of an attack falls in the whole clip.
pub fn mark(clip: sandmaw::Clip, phase: u8, u: f32) -> f32 {
    super::mark(&sandmaw::SPECIES, clip.index(), phase, u)
}

/// The group and depth of each bone. The column is the "neck", loosest at
/// the head; the body behind the hole is the "tail", loosest at its tip.
pub fn group_of(bone: usize) -> (Group, u8) {
    use bones::*;
    match bone {
        ROOT => (Group::Body, 0),
        NECK1 => (Group::Neck, 0),
        NECK2 => (Group::Neck, 1),
        NECK3 => (Group::Neck, 2),
        NECK4 => (Group::Neck, 3),
        NECK5 => (Group::Neck, 4),
        HEAD => (Group::Neck, 5),
        LIP_UP | LIP_DOWN => (Group::Neck, 6),
        TAIL1 => (Group::Tail, 0),
        TAIL2 => (Group::Tail, 1),
        TAIL3 => (Group::Tail, 2),
        TAIL4 => (Group::Tail, 3),
        _ => (Group::Tail, 4),
    }
}

/// The column's bones, after the root.
pub const NECK: [usize; 6] = [
    bones::NECK1,
    bones::NECK2,
    bones::NECK3,
    bones::NECK4,
    bones::NECK5,
    bones::HEAD,
];

/// How deep the middle line lies swimming: the top of every box a hand
/// under the sand.
pub const UNDER_DEPTH: f32 = -1.75;
/// How far back the hips carry the chain swimming, so the head is at the
/// origin.
pub const UNDER_BACK: f32 = -5.5;
/// How far the hole's root sits under the sand standing.
pub const STAND_DEPTH: f32 = -0.5;
/// The column's lean out of the sand, from the floor.
pub const STAND_LEAN: f32 = 60.0;
/// The middle line lying on the sand: the back at 2.2 m.
pub const BEACH_HEIGHT: f32 = 1.0;

/// The Sandmaw's pose builders.
pub trait SandmawPose: Sized {
    fn root(self, pitch: f32, yaw: f32, roll: f32) -> Pose;
    /// Bend the column: one pitch per bone, root outward.
    fn column(self, bends: [f32; 6]) -> Pose;
    /// Bend the body behind the hole, root outward.
    fn tail(self, bends: [f32; 5]) -> Pose;
    /// Swing the tail's base sideways, toward its right.
    fn tail_swing(self, yaw: f32) -> Pose;
    /// Open the mouth: each half of the tooth ring swings out by so many
    /// degrees.
    fn mouth(self, open: f32) -> Pose;
    /// A travelling wave along the whole chain, sideways: `phase` in turns,
    /// `amp` in degrees a bone.
    fn wave(self, phase: f32, amp: f32) -> Pose;
    /// The same, rolling the body about its length: a writhe.
    fn writhe(self, phase: f32, amp: f32) -> Pose;
    fn under() -> Pose;
    fn standing() -> Pose;
    fn beached() -> Pose;
}

impl SandmawPose for Pose {
    fn root(self, pitch: f32, yaw: f32, roll: f32) -> Pose {
        self.set(bones::ROOT, pitch, yaw, roll)
    }
    fn column(mut self, bends: [f32; 6]) -> Pose {
        for (b, p) in NECK.iter().zip(bends) {
            self.bone[*b][0] = p;
        }
        self
    }
    fn tail(mut self, bends: [f32; 5]) -> Pose {
        for (b, p) in bones::TAIL.iter().zip(bends) {
            self.bone[*b][0] = p;
        }
        self
    }
    fn tail_swing(mut self, yaw: f32) -> Pose {
        self.bone[bones::TAIL1][1] = yaw;
        self
    }
    fn mouth(mut self, open: f32) -> Pose {
        self.bone[bones::LIP_UP][0] = open;
        self.bone[bones::LIP_DOWN][0] = -open;
        self
    }
    fn wave(mut self, phase: f32, amp: f32) -> Pose {
        // Down the chain from the head: the column's bones in order, then
        // the tail's, each a little later in the wave than the one before.
        let chain = NECK
            .iter()
            .rev()
            .chain(std::iter::once(&bones::ROOT))
            .chain(bones::TAIL.iter());
        for (i, b) in chain.enumerate() {
            let t = (phase - i as f32 * 0.09) * std::f32::consts::TAU;
            self.bone[*b][1] += amp * t.sin();
        }
        self
    }
    fn writhe(mut self, phase: f32, amp: f32) -> Pose {
        let chain = NECK
            .iter()
            .rev()
            .chain(std::iter::once(&bones::ROOT))
            .chain(bones::TAIL.iter());
        for (i, b) in chain.enumerate() {
            let t = (phase - i as f32 * 0.07) * std::f32::consts::TAU;
            self.bone[*b][2] += amp * t.sin();
        }
        self
    }
    fn under() -> Pose {
        Pose::rest(bones::COUNT).hips(UNDER_BACK, UNDER_DEPTH, 0.0)
    }
    fn standing() -> Pose {
        Pose::rest(bones::COUNT)
            .hips(0.0, STAND_DEPTH, 0.0)
            .root(STAND_LEAN, 0.0, 0.0)
            .column([5.0, 5.0, 0.0, -10.0, -15.0, -25.0])
            .tail([-5.0, 10.0, 10.0, 10.0, 10.0])
    }
    fn beached() -> Pose {
        Pose::rest(bones::COUNT).hips(0.0, BEACH_HEIGHT, 0.0)
    }
}
