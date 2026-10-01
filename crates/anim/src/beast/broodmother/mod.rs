//! The Broodmother's animation: her recipes, and the builders that know a
//! spider.
//!
//! The recipes are in [`clips`]. This file is what the factory in `super`
//! needs from her in particular -- which of her bones are body, head and leg,
//! the pose she stands in, and where her table is baked to -- and the pose
//! builders her recipes are written with.
//!
//! **Eight feet, placed rather than posed.** A spider's silhouette is its
//! legs, and a leg posed by angles alone puts its foot through the floor the
//! moment the body moves. So every leg in every key is *planted*: given where
//! its foot should be, in body space, [`BroodPose::plant`] turns the coxa to
//! it, lifts the femur and drops the tibia onto it -- knee up, the way a
//! spider's knee stands above its back -- by solving the leg as two links in
//! its own vertical plane, then correcting for whatever the body is doing
//! against the rig itself. The keys are positions; the angles follow.

pub mod clips;

use super::{Authored, Group, Pose, Recipe};
use sim::species::Species;
use sim::species::broodmother::{self, LEG_COUNT, bones};

/// The Broodmother, as the animation factory sees her.
pub struct Broodmother;

pub static BROODMOTHER: Broodmother = Broodmother;

impl Authored for Broodmother {
    fn species(&self) -> &'static Species {
        &broodmother::SPECIES
    }

    fn recipes(&self) -> Vec<Recipe> {
        clips::all()
    }

    fn group_of(&self, bone: usize) -> (Group, u8) {
        group_of(bone)
    }

    fn standing(&self) -> Pose {
        <Pose as BroodPose>::standing()
    }

    fn baked_path(&self) -> &'static str {
        "crates/sim/src/species/broodmother/baked.rs"
    }
}

/// Where a point inside one phase of an attack falls in the whole clip.
pub fn mark(clip: broodmother::Clip, phase: u8, u: f32) -> f32 {
    super::mark(&broodmother::SPECIES, clip.index(), phase, u)
}

/// The group and depth of each bone. The head and fangs are the "neck"; the
/// waist and the abdomen are the "tail" -- the heavy loose end that lags the
/// body, which is what makes the slam read as weight; each leg is three deep.
pub fn group_of(bone: usize) -> (Group, u8) {
    use bones::*;
    match bone {
        ROOT => (Group::Body, 0),
        HEAD => (Group::Neck, 0),
        FANGS => (Group::Neck, 1),
        PEDICEL => (Group::Tail, 0),
        ABDOMEN => (Group::Tail, 1),
        b => (Group::Legs, ((b - LEG0) % 3) as u8),
    }
}

/// Each leg's foot when she stands: its bearing round her, in degrees from
/// straight ahead (positive to her right), and how far out from her middle,
/// in metres. Front to back, left before right.
pub const STANCE: [(f32, f32); LEG_COUNT] = [
    (-38.0, 4.3),
    (38.0, 4.3),
    (-74.0, 4.4),
    (74.0, 4.4),
    (-108.0, 4.3),
    (108.0, 4.3),
    (-146.0, 4.2),
    (146.0, 4.2),
];

/// Where leg `i`'s foot stands, in body space, at its stance.
pub fn home(leg: usize) -> [f32; 3] {
    let (deg, r) = STANCE[leg];
    let a = deg.to_radians();
    [r * a.cos(), 0.0, r * a.sin()]
}

/// The two tetrapods a spider walks on: each leg's half of the cycle. Front
/// left, second right, third left and hind right move together.
pub fn tetrapod(leg: usize) -> f32 {
    let pair = leg / 2;
    let left = leg % 2 == 0;
    if (pair % 2 == 0) == left { 0.0 } else { 0.5 }
}

fn f(v: sim::Fx) -> f32 {
    v.to_f32_for_render()
}

/// Her pose builders.
pub trait BroodPose: Sized {
    /// The thorax: pitch (nose up), yaw, roll, in degrees.
    fn body(self, pitch: f32, yaw: f32, roll: f32) -> Pose;
    fn head(self, pitch: f32, yaw: f32, roll: f32) -> Pose;
    /// The fangs open by so many degrees.
    fn fangs(self, open: f32) -> Pose;
    /// The waist and the abdomen: the waist's pitch, and the abdomen's own
    /// pitch on top of it (tail up is negative: it hangs back along `-x`).
    fn abdomen(self, waist: f32, tilt: f32) -> Pose;
    /// Put leg `leg`'s foot at `at`, in body space -- or as near as the leg
    /// reaches.
    fn plant(self, leg: usize, at: [f32; 3]) -> Pose;
    /// Every foot at its stance, lifted by `lift[leg]` metres.
    fn planted_lifted(self, lift: [f32; LEG_COUNT]) -> Pose;
    /// Every foot at its stance.
    fn planted(self) -> Pose;
    fn standing() -> Pose;
}

impl BroodPose for Pose {
    fn body(self, pitch: f32, yaw: f32, roll: f32) -> Pose {
        self.set(bones::ROOT, pitch, yaw, roll)
    }
    fn head(self, pitch: f32, yaw: f32, roll: f32) -> Pose {
        self.set(bones::HEAD, pitch, yaw, roll)
    }
    fn fangs(mut self, open: f32) -> Pose {
        self.bone[bones::FANGS][0] = -open;
        self
    }
    fn abdomen(mut self, waist: f32, tilt: f32) -> Pose {
        self.bone[bones::PEDICEL][0] = waist;
        self.bone[bones::ABDOMEN][0] = tilt;
        self
    }
    fn plant(mut self, leg: usize, at: [f32; 3]) -> Pose {
        let sp = &broodmother::SPECIES;
        let l1 = f(broodmother::femur_length());
        let l2 = f(broodmother::tibia_length());
        let coxa = bones::coxa(leg);
        let femur = bones::femur(leg);
        let tibia = bones::tibia(leg);
        let out = f(sp.rest(femur).x);
        // Solve against a target that is nudged by what the last solve
        // missed: the body's own pitch and roll tilt the leg's plane, and a
        // few rounds take that out without writing the rotation down twice.
        let mut goal = at;
        for _ in 0..6 {
            let rig = sim::beast::Rig::build(sp, sim::V3::ZERO, sim::Fx::ZERO, &self.to_sim());
            let root = rig.bone[bones::ROOT];
            let to = |v: [f32; 3]| {
                sim::V3::new(
                    sim::Fx::from_raw((v[0] * 65536.0) as i32),
                    sim::Fx::from_raw((v[1] * 65536.0) as i32),
                    sim::Fx::from_raw((v[2] * 65536.0) as i32),
                )
            };
            // The goal in the thorax's own frame, from the coxa.
            let local = root.world_to_local(to(goal));
            let hip = sp.rest(coxa);
            let (dx, dy, dz) = (
                f(local.x) - f(hip.x),
                f(local.y) - f(hip.y),
                f(local.z) - f(hip.z),
            );
            let yaw = dz.atan2(dx);
            let dh = (dx * dx + dz * dz).sqrt() - out;
            let d = (dh * dh + dy * dy)
                .sqrt()
                .clamp((l1 - l2).abs() + 0.02, l1 + l2 - 0.02);
            let phi = dy.atan2(dh);
            let alpha = ((l1 * l1 + d * d - l2 * l2) / (2.0 * l1 * d))
                .clamp(-1.0, 1.0)
                .acos();
            // Knee up: the femur rises above the line to the foot.
            let thigh = phi + alpha;
            let (kx, ky) = (l1 * thigh.cos(), l1 * thigh.sin());
            let shin = (dy - ky).atan2(dh - kx);
            self.bone[coxa] = [0.0, yaw.to_degrees(), 0.0];
            self.bone[femur] = [thigh.to_degrees(), 0.0, 0.0];
            self.bone[tibia] = [(shin - thigh).to_degrees(), 0.0, 0.0];
            // Where the foot came out, and what is left to correct.
            let rig = sim::beast::Rig::build(sp, sim::V3::ZERO, sim::Fx::ZERO, &self.to_sim());
            let foot = rig.bone[tibia].local_to_world(sim::V3::new(
                broodmother::tibia_length(),
                sim::Fx::ZERO,
                sim::Fx::ZERO,
            ));
            let miss = [at[0] - f(foot.x), at[1] - f(foot.y), at[2] - f(foot.z)];
            if miss.iter().all(|m| m.abs() < 0.005) {
                break;
            }
            goal = [goal[0] + miss[0], goal[1] + miss[1], goal[2] + miss[2]];
        }
        self
    }
    fn planted_lifted(self, lift: [f32; LEG_COUNT]) -> Pose {
        (0..LEG_COUNT).fold(self, |p, leg| {
            let h = home(leg);
            p.plant(leg, [h[0], lift[leg], h[2]])
        })
    }
    fn planted(self) -> Pose {
        self.planted_lifted([0.0; LEG_COUNT])
    }
    fn standing() -> Pose {
        Pose::rest(bones::COUNT).planted()
    }
}
