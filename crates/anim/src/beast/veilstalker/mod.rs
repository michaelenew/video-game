//! The Veilstalker's animation: its recipes, and the builders that know it.
//!
//! The recipes are in [`clips`]. This file is what the factory in `super`
//! needs from the animal in particular -- which of its bones are body, neck,
//! tail and leg, the pose it stands in, where its table is baked to -- and the
//! pose builders its recipes are written with, as a trait on the shared
//! [`Pose`].
//!
//! **The Pair's builders, on the Pair's bones.** The lizard-cat is the same
//! eighteen-bone topology re-proportioned again (`sim::species::veilstalker`),
//! so the builders are the cat's with the Veilstalker's table behind them.
//! Where it differs it says so in the clips: it is long and low, its tail is
//! three metres and a weapon, and its tells are four **rears** -- shapes that
//! stand up out of nothing (`veilstalker.md` §2).

pub mod clips;

use super::{Authored, Group, Pose, Recipe};
use sim::species::Species;
use sim::species::veilstalker::{self as vs, Clip, bones};

/// The Veilstalker, as the animation factory sees it.
pub struct Veilstalker;

pub static VEILSTALKER: Veilstalker = Veilstalker;

impl Authored for Veilstalker {
    fn species(&self) -> &'static Species {
        &vs::SPECIES
    }

    fn recipes(&self) -> Vec<Recipe> {
        clips::all()
    }

    fn group_of(&self, bone: usize) -> (Group, u8) {
        group_of(bone)
    }

    fn standing(&self) -> Pose {
        <Pose as StalkerPose>::standing()
    }

    fn baked_path(&self) -> &'static str {
        "crates/sim/src/species/veilstalker/baked.rs"
    }
}

/// Where a point inside one phase of an attack falls in the whole clip. See
/// [`super::mark`].
pub fn mark(clip: Clip, phase: u8, u: f32) -> f32 {
    super::mark(&vs::SPECIES, clip.index(), phase, u)
}

/// Where a frame of an attack's startup falls in the whole clip -- for the
/// keys that have to land on a particular frame however long the startup is
/// tuned to be: the silhouette at frame eight.
pub fn at_frame(clip: Clip, frame: f32) -> f32 {
    let species = &vs::SPECIES;
    let kind = species
        .moves
        .iter()
        .position(|m| m.clip == clip.index())
        .unwrap_or(0) as u8;
    let total = species.attack(kind).total().max(1) as f32;
    (frame / total).clamp(0.0, 1.0)
}

pub const TAIL_CHAIN: [usize; 4] = [bones::TAIL1, bones::TAIL2, bones::TAIL3, bones::TAIL4];

/// `(hip, knee)` per leg, in `veilstalker::LEGS` order: front left, front right,
/// hind left, hind right.
pub const LEG_BONES: [(usize, usize); 4] = [
    (bones::SHOULDER_L, bones::FOREARM_L),
    (bones::SHOULDER_R, bones::FOREARM_R),
    (bones::THIGH_L, bones::SHIN_L),
    (bones::THIGH_R, bones::SHIN_R),
];

/// The group and the depth of each bone: the Ridgeback's map, on the
/// Ridgeback's bones.
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

/// Its pose builders. The mirror lives in the skeleton, so
/// `forelegs(-20.0, 30.0)` puts both forelegs in the same shape.
pub trait StalkerPose: Sized {
    fn root(self, pitch: f32, yaw: f32, roll: f32) -> Pose;
    fn spine(self, pitch: f32, yaw: f32, roll: f32) -> Pose;
    fn chest(self, pitch: f32, yaw: f32, roll: f32) -> Pose;
    fn head(self, pitch: f32, yaw: f32, roll: f32) -> Pose;
    fn neck(self, pitch: f32, yaw: f32) -> Pose;
    fn tail(self, lift: f32, yaw: f32) -> Pose;
    fn tail_from_base(self, lift: f32) -> Pose;
    fn tail_swing_base(self, yaw: f32) -> Pose;
    fn leg(self, which: usize, swing: f32, knee: f32) -> Pose;
    fn leg_spread(self, which: usize, spread: f32) -> Pose;
    fn forelegs(self, swing: f32, knee: f32) -> Pose;
    fn hindlegs(self, swing: f32, knee: f32) -> Pose;
    fn standing() -> Pose;
    fn plant(self, which: usize, reach: f32, clear: f32) -> Pose;
    fn plant_fore(self, reach: f32, clear: f32) -> Pose;
    fn plant_hind(self, reach: f32, clear: f32) -> Pose;
}

impl StalkerPose for Pose {
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

    /// The neck as a curve rather than a hinge, weighted toward the base.
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

    /// Lift the whole tail and swing it, spread toward the tip. **Positive
    /// `lift` raises the tail**: the tail extends backwards from its bones,
    /// so the rig's nose-up pitch points it at the floor.
    fn tail(mut self, lift: f32, yaw: f32) -> Pose {
        let share = [0.18, 0.24, 0.28, 0.30];
        for (bone, k) in TAIL_CHAIN.into_iter().zip(share) {
            self.bone[bone][0] -= lift * k;
            self.bone[bone][1] += yaw * k;
        }
        self
    }

    /// Lift or drop the tail from its root, as one beam.
    fn tail_from_base(mut self, lift: f32) -> Pose {
        self.bone[bones::TAIL1][0] -= lift;
        self
    }

    /// Swing the whole tail about its root, as one beam: the trip's sweep,
    /// which has to carry its tip across the floor rather than whip it.
    fn tail_swing_base(mut self, yaw: f32) -> Pose {
        self.bone[bones::TAIL1][1] += yaw;
        self
    }

    fn leg(mut self, which: usize, swing: f32, knee: f32) -> Pose {
        let (hip, shin) = LEG_BONES[which];
        self.bone[hip][0] = swing;
        self.bone[shin][0] = knee;
        self
    }

    fn leg_spread(mut self, which: usize, spread: f32) -> Pose {
        let (hip, _) = LEG_BONES[which];
        self.bone[hip][2] = spread * vs::BONES[hip].side as f32;
        self
    }

    fn forelegs(self, swing: f32, knee: f32) -> Pose {
        self.leg(0, swing, knee).leg(1, swing, knee)
    }

    fn hindlegs(self, swing: f32, knee: f32) -> Pose {
        self.leg(2, swing, knee).leg(3, swing, knee)
    }

    /// **How it slinks**: low, the head carried forward and level with the
    /// shoulders, the tail held straight out behind as a counterweight and
    /// lifted at the tip, the forepaws under the shoulders and the hind paws
    /// a little behind the hips, every foot solved onto the floor.
    fn standing() -> Pose {
        Pose::rest(bones::COUNT)
            .neck(-6.0, 0.0)
            .head(4.0, 0.0, 0.0)
            .tail(-4.0, 0.0)
            .plant_fore(0.08, 0.0)
            .plant_hind(-0.12, 0.0)
    }

    fn plant(self, which: usize, reach: f32, clear: f32) -> Pose {
        self.plant_leg(&vs::SPECIES, which, reach, clear)
    }

    fn plant_fore(self, reach: f32, clear: f32) -> Pose {
        self.plant(0, reach, clear).plant(1, reach, clear)
    }

    fn plant_hind(self, reach: f32, clear: f32) -> Pose {
        self.plant(2, reach, clear).plant(3, reach, clear)
    }
}
