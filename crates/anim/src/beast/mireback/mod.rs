//! The Mireback's animation: its recipes, and the builders that know a toad.
//!
//! The recipes are in [`clips`]. This file is what the factory in `super`
//! needs from the Mireback in particular -- which of its bones are body, head
//! and leg, the pose it stands in, and where its table is baked to -- and the
//! pose builders its recipes are written with, as a trait on the shared
//! [`Pose`] so they can share names with the Ridgeback's without meeting them.

pub mod clips;

use super::{Authored, Group, Pose, Recipe};
use sim::species::Species;
use sim::species::mireback::{self, Clip, bones};

/// The Mireback, as the animation factory sees it.
pub struct Mireback;

pub static MIREBACK: Mireback = Mireback;

impl Authored for Mireback {
    fn species(&self) -> &'static Species {
        &mireback::SPECIES
    }

    fn recipes(&self) -> Vec<Recipe> {
        clips::all()
    }

    fn group_of(&self, bone: usize) -> (Group, u8) {
        group_of(bone)
    }

    fn standing(&self) -> Pose {
        <Pose as MirebackPose>::standing()
    }

    fn baked_path(&self) -> &'static str {
        "crates/sim/src/species/mireback/baked.rs"
    }
}

/// Where a point inside one phase of an attack falls in the whole clip.
pub fn mark(clip: Clip, phase: u8, u: f32) -> f32 {
    super::mark(&mireback::SPECIES, clip.index(), phase, u)
}

/// `(hip, knee)` per leg, in `mireback::LEGS` order: front left, front right,
/// hind left, hind right.
pub const LEG_BONES: [(usize, usize); 4] = [
    (bones::SHOULDER_L, bones::FOREARM_L),
    (bones::SHOULDER_R, bones::FOREARM_R),
    (bones::THIGH_L, bones::SHIN_L),
    (bones::THIGH_R, bones::SHIN_R),
];

/// The group and depth of each bone. The head and what hangs off it are the
/// "neck" -- the loose end -- and the jaw, the tongue and the sac trail it.
pub fn group_of(bone: usize) -> (Group, u8) {
    use bones::*;
    match bone {
        ROOT => (Group::Body, 0),
        HEAD => (Group::Neck, 0),
        JAW | THROAT => (Group::Neck, 1),
        TONGUE => (Group::Neck, 2),
        SHOULDER_L | SHOULDER_R | THIGH_L | THIGH_R => (Group::Legs, 0),
        _ => (Group::Legs, 1),
    }
}

/// How far ahead of its hip each foot stands, standing: the forefeet a
/// little ahead of the shoulder, the hind feet well ahead of the hip, folded
/// under the body the way a sitting toad's are.
pub const FORE_REACH: f32 = 0.25;
pub const HIND_REACH: f32 = 0.7;

/// The Mireback's pose builders.
pub trait MirebackPose: Sized {
    fn root(self, pitch: f32, yaw: f32, roll: f32) -> Pose;
    fn head(self, pitch: f32, yaw: f32, roll: f32) -> Pose;
    /// Open the jaw by so many degrees.
    fn jaw(self, open: f32) -> Pose;
    /// Swing the throat sac down and out by so many degrees: shown.
    fn sac(self, out: f32) -> Pose;
    fn tongue(self, pitch: f32) -> Pose;
    fn leg(self, which: usize, swing: f32, knee: f32) -> Pose;
    fn spread(self, which: usize, out: f32) -> Pose;
    fn spread_all(self, out: f32) -> Pose;
    fn plant(self, which: usize, reach: f32, clear: f32) -> Pose;
    fn plant_fore(self, reach: f32, clear: f32) -> Pose;
    fn plant_hind(self, reach: f32, clear: f32) -> Pose;
    /// Every foot back on the floor where it stands.
    fn planted(self) -> Pose;
    fn standing() -> Pose;
    /// On its back: rolled over, lifted so its crown rests on the floor,
    /// legs in the air. `kick` sets them going.
    fn belly_up(kick: f32) -> Pose;
}

impl MirebackPose for Pose {
    fn root(self, pitch: f32, yaw: f32, roll: f32) -> Pose {
        self.set(bones::ROOT, pitch, yaw, roll)
    }
    fn head(self, pitch: f32, yaw: f32, roll: f32) -> Pose {
        self.set(bones::HEAD, pitch, yaw, roll)
    }
    fn jaw(mut self, open: f32) -> Pose {
        self.bone[bones::JAW][0] = -open;
        self
    }
    fn sac(mut self, out: f32) -> Pose {
        self.bone[bones::THROAT][0] = -out;
        self
    }
    fn tongue(mut self, pitch: f32) -> Pose {
        self.bone[bones::TONGUE][0] = pitch;
        self
    }
    fn leg(mut self, which: usize, swing: f32, knee: f32) -> Pose {
        let (hip, knee_bone) = LEG_BONES[which];
        self.bone[hip][0] = swing;
        self.bone[knee_bone][0] = knee;
        self
    }
    fn spread(mut self, which: usize, out: f32) -> Pose {
        let (hip, _) = LEG_BONES[which];
        self.bone[hip][2] = out * mireback::BONES[hip].side as f32;
        self
    }
    fn spread_all(self, out: f32) -> Pose {
        (0..4).fold(self, |p, i| p.spread(i, out))
    }
    fn plant(self, which: usize, reach: f32, clear: f32) -> Pose {
        self.plant_leg(&mireback::SPECIES, which, reach, clear)
    }
    fn plant_fore(self, reach: f32, clear: f32) -> Pose {
        self.plant(0, reach, clear).plant(1, reach, clear)
    }
    fn plant_hind(self, reach: f32, clear: f32) -> Pose {
        self.plant(2, reach, clear).plant(3, reach, clear)
    }
    fn planted(self) -> Pose {
        self.plant_fore(FORE_REACH, 0.0).plant_hind(HIND_REACH, 0.0)
    }
    fn standing() -> Pose {
        Pose::rest(bones::COUNT).planted()
    }
    fn belly_up(kick: f32) -> Pose {
        Pose::rest(bones::COUNT)
            .hips(0.0, 2.3, 0.0)
            .root(0.0, 0.0, 180.0)
            .head(-6.0, 0.0, 0.0)
            .jaw(18.0)
            .leg(0, 30.0 + kick, -40.0 - kick)
            .leg(1, 30.0 - kick, -40.0 + kick)
            .leg(2, -20.0 - kick, 50.0 + kick)
            .leg(3, -20.0 + kick, 50.0 - kick)
            .spread_all(20.0)
    }
}
