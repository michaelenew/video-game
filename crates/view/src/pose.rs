//! What a character looks like at one instant, and how a person writes one
//! down.
//!
//! **The rule everything else depends on: pose is a pure function of simulation
//! state.** No accumulated animation time, no independently ticking player.
//! Rollback re-simulates past frames, so anything animating on its own clock
//! pops and slides every time a rollback happens.
//!
//! ```text
//! pose = f(action, frames into it, speed, stride, airtime, sim_frame)
//! ```
//!
//! All of that comes out of the snapshot. Cosmetic smoothing may live in the
//! renderer and is allowed to pop across a rollback -- one to eight frames of
//! visual discontinuity is imperceptible. Nothing that reads as gameplay may.
//!
//! ## A pose is angles
//!
//! Fifty-one numbers: three for where the hips are, and three for each of the
//! sixteen joints. No positions anywhere else -- an elbow cannot be anywhere
//! except at the end of its upper arm, and saying so in the data structure is
//! what makes one clip play on six differently-proportioned bodies.
//!
//! Authoring is in **degrees**, through named methods, because that is how a
//! person describes a body:
//!
//! ```ignore
//! // Weight back, sword arm cocked, front foot light.
//! Pose::rest()
//!     .hips(0.0, -0.05, -0.10)
//!     .spine(-8.0, 0.0, -22.0)
//!     .chest(4.0, 0.0, -18.0)
//!     .head(0.0, 0.0, 20.0)
//!     .shoulder_r(-40.0, 28.0, 0.0)
//!     .elbow_r(95.0)
//!     .hip_l(18.0, 6.0, 0.0)
//!     .knee_l(22.0)
//! ```
//!
//! Every angle means the same thing on the left as on the right: positive
//! swing is forward, positive spread is away from the body, on both sides. The
//! mirroring lives in the skeleton, so mirroring a clip is a swap of the two
//! sides and nothing else.

use crate::ik;
use crate::math::{self, V3};
use crate::skeleton::{Group, JOINT_COUNT, JOINTS, Joint, Skeleton};
use std::sync::LazyLock;

/// Three for the hips' offset, then swing/spread/twist for each joint.
pub const CHANNELS: usize = 3 + JOINT_COUNT * 3;

/// The height an ankle sits at when its foot is flat on the floor. Foot targets
/// for `plant_l` and `plant_r` are ankles, so this is the `y` a grounded step
/// asks for.
pub const ANKLE_ON_GROUND: f32 = 0.06;

/// The body every pose is authored against: the middleweight, 1.8 m. Angles
/// retarget to any build for free; only the hip offset is in metres, and that
/// is scaled by the build when the skeleton is solved.
pub fn reference() -> &'static Skeleton {
    static REFERENCE: LazyLock<Skeleton> =
        LazyLock::new(|| Skeleton::new(crate::skeleton::Build::REFERENCE));
    &REFERENCE
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    /// `[0..3]` is the hip offset in reference metres; after that, three
    /// radians per joint in `JOINTS` order.
    pub channels: [f32; CHANNELS],
}

const RAD: f32 = std::f32::consts::PI / 180.0;

impl Default for Pose {
    fn default() -> Self {
        Pose::rest()
    }
}

impl Pose {
    /// Standing straight, arms at the sides, every channel zero.
    pub const fn rest() -> Pose {
        Pose {
            channels: [0.0; CHANNELS],
        }
    }

    pub const fn from_channels(channels: [f32; CHANNELS]) -> Pose {
        Pose { channels }
    }

    /// Where the hips sit, in reference metres from standing.
    pub fn root_offset(&self) -> V3 {
        [self.channels[0], self.channels[1], self.channels[2]]
    }

    pub fn angles(&self, j: Joint) -> [f32; 3] {
        let i = 3 + j.index() * 3;
        [self.channels[i], self.channels[i + 1], self.channels[i + 2]]
    }

    pub fn set_angles(&mut self, j: Joint, a: [f32; 3]) {
        let i = 3 + j.index() * 3;
        self.channels[i] = a[0];
        self.channels[i + 1] = a[1];
        self.channels[i + 2] = a[2];
    }

    /// One channel of one joint, in degrees -- what a slider edits.
    pub fn degrees(&self, j: Joint, c: usize) -> f32 {
        self.angles(j)[c] / RAD
    }

    pub fn set_degrees(&mut self, j: Joint, c: usize, v: f32) {
        self.channels[3 + j.index() * 3 + c] = v * RAD;
    }

    /// Clamp every joint to what a body can actually do. The solver runs this
    /// on its output, so overshoot can be tuned for feel without anyone having
    /// to check it against anatomy.
    pub fn clamped(mut self, skeleton: &Skeleton) -> Pose {
        for j in JOINTS {
            let a = self.angles(j);
            let (s, p, t) = skeleton.bone(j).limits.clamp(a[0], a[1], a[2]);
            self.set_angles(j, [s, p, t]);
        }
        self
    }

    /// Does this pose ask for anything a body cannot do? Used by a test that
    /// holds every authored key to the same standard.
    pub fn violations(&self, skeleton: &Skeleton) -> Vec<(Joint, usize, f32)> {
        let mut out = Vec::new();
        for j in JOINTS {
            let a = self.angles(j);
            for (c, v) in a.iter().enumerate() {
                let (lo, hi) = skeleton.bone(j).limits.channel(c);
                if *v < lo - 1e-4 || *v > hi + 1e-4 {
                    out.push((j, c, v / RAD));
                }
            }
        }
        out
    }

    // -----------------------------------------------------------------------
    // Authoring
    // -----------------------------------------------------------------------

    /// Move the hips, in metres. Down is where most of a pose's weight lives:
    /// a crouch, a coil before a jump, the dip in a walk cycle.
    pub fn hips(mut self, x: f32, y: f32, z: f32) -> Pose {
        self.channels[0] = x;
        self.channels[1] = y;
        self.channels[2] = z;
        self
    }

    /// Rotate the whole body at the hips: lean forward, tilt sideways, turn.
    pub fn root(self, lean: f32, tilt: f32, turn: f32) -> Pose {
        self.set(Joint::Root, lean, tilt, turn)
    }

    /// Bend at the waist, lean sideways, twist.
    pub fn spine(self, bend: f32, side: f32, twist: f32) -> Pose {
        self.set(Joint::Spine, bend, side, twist)
    }

    /// The upper torso, on top of the spine. Most of a swing's rotation should
    /// be here and in the spine together -- a torso that turns as one block is
    /// the fastest way to make a character look like a mannequin.
    pub fn chest(self, bend: f32, side: f32, twist: f32) -> Pose {
        self.set(Joint::Chest, bend, side, twist)
    }

    /// Nod, tilt, look left or right. Positive turn looks to the character's
    /// own right.
    pub fn head(self, nod: f32, tilt: f32, turn: f32) -> Pose {
        self.set(Joint::Head, nod, tilt, turn)
    }

    /// Shoulder: forward, out from the body, rolled.
    pub fn shoulder_l(self, swing: f32, spread: f32, twist: f32) -> Pose {
        self.set(Joint::ArmL, swing, spread, twist)
    }

    pub fn shoulder_r(self, swing: f32, spread: f32, twist: f32) -> Pose {
        self.set(Joint::ArmR, swing, spread, twist)
    }

    /// Both shoulders at once -- a symmetric guard, a two-handed grip.
    pub fn shoulders(self, swing: f32, spread: f32, twist: f32) -> Pose {
        self.shoulder_l(swing, spread, twist)
            .shoulder_r(swing, spread, twist)
    }

    /// Elbow flexion. Zero is a straight arm; 150 is folded shut.
    pub fn elbow_l(self, bend: f32) -> Pose {
        self.set(Joint::ForearmL, bend, 0.0, 0.0)
    }

    pub fn elbow_r(self, bend: f32) -> Pose {
        self.set(Joint::ForearmR, bend, 0.0, 0.0)
    }

    pub fn elbows(self, bend: f32) -> Pose {
        self.elbow_l(bend).elbow_r(bend)
    }

    /// Elbow flexion plus forearm roll, which is what turns a palm over.
    pub fn forearm_l(self, bend: f32, twist: f32) -> Pose {
        self.set(Joint::ForearmL, bend, 0.0, twist)
    }

    pub fn forearm_r(self, bend: f32, twist: f32) -> Pose {
        self.set(Joint::ForearmR, bend, 0.0, twist)
    }

    pub fn wrist_l(self, bend: f32, spread: f32, twist: f32) -> Pose {
        self.set(Joint::HandL, bend, spread, twist)
    }

    pub fn wrist_r(self, bend: f32, spread: f32, twist: f32) -> Pose {
        self.set(Joint::HandR, bend, spread, twist)
    }

    pub fn wrists(self, bend: f32, spread: f32, twist: f32) -> Pose {
        self.wrist_l(bend, spread, twist)
            .wrist_r(bend, spread, twist)
    }

    /// Hip: leg forward, leg out to the side, leg rolled.
    pub fn hip_l(self, swing: f32, spread: f32, twist: f32) -> Pose {
        self.set(Joint::ThighL, swing, spread, twist)
    }

    pub fn hip_r(self, swing: f32, spread: f32, twist: f32) -> Pose {
        self.set(Joint::ThighR, swing, spread, twist)
    }

    pub fn hips_both(self, swing: f32, spread: f32, twist: f32) -> Pose {
        self.hip_l(swing, spread, twist).hip_r(swing, spread, twist)
    }

    /// Knee flexion. Zero is a straight leg; positive folds the heel back.
    pub fn knee_l(self, bend: f32) -> Pose {
        self.set(Joint::ShinL, bend, 0.0, 0.0)
    }

    pub fn knee_r(self, bend: f32) -> Pose {
        self.set(Joint::ShinR, bend, 0.0, 0.0)
    }

    pub fn knees(self, bend: f32) -> Pose {
        self.knee_l(bend).knee_r(bend)
    }

    /// Ankle: positive `point` drops the toe, which is what sells a push-off.
    pub fn ankle_l(self, point: f32, roll: f32, turn: f32) -> Pose {
        self.set(Joint::FootL, point, roll, turn)
    }

    pub fn ankle_r(self, point: f32, roll: f32, turn: f32) -> Pose {
        self.set(Joint::FootR, point, roll, turn)
    }

    pub fn ankles(self, point: f32, roll: f32, turn: f32) -> Pose {
        self.ankle_l(point, roll, turn).ankle_r(point, roll, turn)
    }

    /// Any joint, by name, in degrees.
    pub fn set(mut self, j: Joint, swing: f32, spread: f32, twist: f32) -> Pose {
        self.set_angles(j, [swing * RAD, spread * RAD, twist * RAD]);
        self
    }

    // -----------------------------------------------------------------------
    // Inverse kinematics
    // -----------------------------------------------------------------------

    /// Put the left ankle at a point in character space and let the leg work
    /// out how. `[x, y, z]` in metres: +Z is forward, +Y up, ground at zero.
    ///
    /// Use this for anything with a planted foot. A foot given the same target
    /// on consecutive frames does not move, however much the hips do, which is
    /// what makes a walk cycle stop sliding.
    pub fn plant_l(mut self, target: V3) -> Pose {
        ik::foot_to(&mut self, reference(), true, target);
        self
    }

    pub fn plant_r(mut self, target: V3) -> Pose {
        ik::foot_to(&mut self, reference(), false, target);
        self
    }

    /// Put the left wrist at a point in character space.
    pub fn reach_l(mut self, target: V3) -> Pose {
        ik::hand_to(&mut self, reference(), true, target);
        self
    }

    pub fn reach_r(mut self, target: V3) -> Pose {
        ik::hand_to(&mut self, reference(), false, target);
        self
    }

    /// Level a foot with the floor.
    ///
    /// A planted foot solved by IK inherits whatever angle the shin ended up
    /// at, which points the sole into the ground or up at the ceiling. This
    /// works out the ankle angle that makes the sole flat -- and if the joint
    /// cannot reach that angle, the limits clamp it and the heel lifts, which
    /// is what a real ankle does at the end of a stride.
    ///
    /// Call it *after* planting the foot.
    pub fn level_l(self) -> Pose {
        self.level(Joint::FootL)
    }

    pub fn level_r(self) -> Pose {
        self.level(Joint::FootR)
    }

    pub fn level_feet(self) -> Pose {
        self.level_l().level_r()
    }

    /// Roll the foot onto the ball of its toe, whatever height the ankle is at.
    ///
    /// The other half of `level_l`. At the end of a stride the ankle is already
    /// rising while the toe is still carrying weight, and the ankle angle that
    /// keeps the toe on the floor depends on exactly how high the ankle got --
    /// which is not a number a person can hold in their head, and is exactly
    /// the kind of thing the kinematics should work out.
    ///
    /// If the ankle is too high to reach the floor, the joint goes as far as it
    /// can and the foot leaves the ground, which is what feet do.
    pub fn toe_floor_l(self) -> Pose {
        self.pivot(Joint::FootL)
    }

    pub fn toe_floor_r(self) -> Pose {
        self.pivot(Joint::FootR)
    }

    fn pivot(mut self, foot: Joint) -> Pose {
        let skeleton = reference();
        let bone = skeleton.bone(foot);
        let parent = foot.parent().expect("a foot hangs off a shin");
        let skin = crate::skeleton::solve(skeleton, &self);
        let shin = skin.rot[parent.index()];
        let ankle_y = skin.origin[foot.index()][1];

        // The toe's height is `length * (Q cos a - P sin a)` above the ankle,
        // where P and Q are how much the shin tilts the foot's own up and
        // forward axes. Setting that equal to what is wanted is one `acos`.
        let p = shin.rotate([0.0, 1.0, 0.0])[1];
        let q = shin.rotate([0.0, 0.0, 1.0])[1];
        // A little above the floor rather than exactly on it: the tip is the
        // middle of the end of the box, and a pitched box's front-bottom corner
        // hangs below that. Cheaper than solving for the corner, and the error
        // is in the safe direction.
        let drop = bone.half[1] + 0.025 * skeleton.scale + 0.022;
        let k = (drop - ankle_y) / bone.length;
        let r = (p * p + q * q).sqrt().max(1e-5);
        let phi = (-p).atan2(q);
        // Two solutions, mirrored about the shin's own tilt. Take the one with
        // the toe down: that is the half of the stride this is for.
        let angle = phi + (k / r).clamp(-1.0, 1.0).acos();

        let (lo, hi) = bone.limits.swing;
        let mut a = self.angles(foot);
        a[0] = (angle / bone.swing_sign).clamp(lo, hi);
        self.set_angles(foot, a);
        self
    }

    /// Level the foot, then tip it: positive drops the toe, negative lifts it.
    ///
    /// This is the authoring unit for a stride. A heel strike is `toe_l(-12)`,
    /// a flat plant is `toe_l(0)`, and pushing off the ball of the foot is
    /// `toe_l(35)` -- none of which a person can express as an absolute ankle
    /// angle, because the answer depends on what the shin is doing.
    pub fn toe_l(self, degrees: f32) -> Pose {
        self.tip(Joint::FootL, degrees)
    }

    pub fn toe_r(self, degrees: f32) -> Pose {
        self.tip(Joint::FootR, degrees)
    }

    fn tip(self, foot: Joint, degrees: f32) -> Pose {
        let mut p = self.level(foot);
        let mut a = p.angles(foot);
        let (lo, hi) = reference().bone(foot).limits.swing;
        a[0] = (a[0] + degrees * RAD).clamp(lo, hi);
        p.set_angles(foot, a);
        p
    }

    fn level(mut self, foot: Joint) -> Pose {
        let skeleton = reference();
        let parent = foot.parent().expect("a foot hangs off a shin");
        let shin = crate::skeleton::solve(skeleton, &self).rot[parent.index()];
        // The foot points along its own +Z. Under the shin's rotation, the
        // ankle angle that flattens it is one atan2 -- see `skeleton::solve`
        // for why the composition is what it is.
        let up_y = shin.rotate([0.0, 1.0, 0.0])[1];
        let fwd_y = shin.rotate([0.0, 0.0, 1.0])[1];
        let angle = fwd_y.atan2(up_y);
        let mut a = self.angles(foot);
        // Clamped, because an ankle cannot always reach flat -- a leg stretched
        // out behind the body runs out of joint first, and what a real ankle
        // does then is lift the heel. Levelling is derived rather than
        // authored, so clamping it here is doing what was asked rather than
        // overruling it.
        let (lo, hi) = skeleton.bone(foot).limits.swing;
        a[0] = (angle / skeleton.bone(foot).swing_sign).clamp(lo, hi);
        self.set_angles(foot, a);
        self
    }

    // -----------------------------------------------------------------------
    // Combining
    // -----------------------------------------------------------------------

    /// The same pose on the other side. Left becomes right, sideways lean and
    /// twist flip, everything else stays -- which is the whole point of naming
    /// the channels rather than storing raw Euler angles.
    pub fn mirrored(&self) -> Pose {
        let mut out = *self;
        out.channels[0] = -self.channels[0];
        for j in JOINTS {
            let a = self.angles(j.opposite());
            if j.opposite() == j {
                out.set_angles(j, [a[0], -a[1], -a[2]]);
            } else {
                out.set_angles(j, a);
            }
        }
        out
    }

    /// Straight blend, channel by channel. Angles, so there is no shortening
    /// artefact to worry about, and the ranges involved are far from any wrap.
    pub fn blend(&self, other: &Pose, t: f32) -> Pose {
        let mut out = *self;
        for i in 0..CHANNELS {
            out.channels[i] = self.channels[i] + (other.channels[i] - self.channels[i]) * t;
        }
        out
    }

    /// Add a second pose's channels on top, scaled. Used for layering: a turn's
    /// lean on top of a walk, a hit's flinch on top of a guard.
    pub fn layered(&self, delta: &Pose, weight: f32) -> Pose {
        let mut out = *self;
        for i in 0..CHANNELS {
            out.channels[i] += delta.channels[i] * weight;
        }
        out
    }

    /// Difference between two poses, for layering one over another later.
    pub fn difference(&self, base: &Pose) -> Pose {
        let mut out = *self;
        for i in 0..CHANNELS {
            out.channels[i] = self.channels[i] - base.channels[i];
        }
        out
    }

    /// Replace one body group's channels from another pose. This is how an
    /// upper-body attack is played over whatever the legs are already doing.
    pub fn take_group(&self, other: &Pose, group: Group) -> Pose {
        let mut out = *self;
        for j in JOINTS {
            if j.group() == group {
                out.set_angles(j, other.angles(j));
            }
        }
        out
    }

    /// How far apart two poses are, summed over every channel. A crude but
    /// useful measure: tests use it to assert that two poses actually read
    /// differently at gameplay distance.
    pub fn separation(&self, other: &Pose) -> f32 {
        (0..CHANNELS)
            .map(|i| (self.channels[i] - other.channels[i]).abs())
            .sum()
    }

    /// Where a joint ends up on the reference body. Convenience for tests and
    /// for the contact sheets.
    pub fn joint_at(&self, j: Joint) -> V3 {
        crate::skeleton::solve(reference(), self).origin[j.index()]
    }

    /// Lowest corner of either foot, which is what a clip has to keep on the
    /// floor unless it means not to.
    ///
    /// The corners, not the centre: a foot is 22 cm long, so at a 40-degree
    /// push-off the toe is 7 cm below where the middle of the box is, and a
    /// check that ignores that passes a clip whose toes are through the floor.
    pub fn lowest_foot(&self, skeleton: &Skeleton) -> f32 {
        let skin = crate::skeleton::solve(skeleton, self);
        let mut lowest = f32::MAX;
        for j in [Joint::FootL, Joint::FootR] {
            let (centre, rot) = skin.box_of(skeleton, j);
            let h = skeleton.bone(j).half;
            for sx in [-1.0, 1.0f32] {
                for sy in [-1.0, 1.0f32] {
                    for sz in [-1.0, 1.0f32] {
                        let corner = rot.rotate([sx * h[0], sy * h[1], sz * h[2]]);
                        lowest = lowest.min(centre[1] + corner[1]);
                    }
                }
            }
        }
        lowest
    }

    /// The two places a foot can touch the ground: the back and front of the
    /// sole, in character space.
    ///
    /// Which of them is carrying the weight is the whole question when you are
    /// asking whether a foot is sliding: flat on the floor both are, at a heel
    /// strike it is the heel, and rolling off the end of a stride it is the toe
    /// -- and the ankle is moving in that last case however planted the foot is.
    ///
    /// Taken from the actual corners of the box rather than from the ankle with
    /// a fudge subtracted, because a foot pitched thirty degrees puts its toe
    /// two centimetres from where the fudge says, and two centimetres is the
    /// difference between a planted foot and one through the floor.
    pub fn foot_contact(&self, skeleton: &Skeleton, left: bool) -> (V3, V3) {
        let j = if left { Joint::FootL } else { Joint::FootR };
        let skin = crate::skeleton::solve(skeleton, self);
        let (centre, rot) = skin.box_of(skeleton, j);
        let h = skeleton.bone(j).half;
        let at = |z: f32| math::add(centre, rot.rotate([0.0, -h[1], z]));
        (at(-h[2]), at(h[2]))
    }
}

/// Every joint's box, ready for the renderer.
pub fn boxes(skeleton: &Skeleton, pose: &Pose) -> [(V3, math::Quat); JOINT_COUNT] {
    let skin = crate::skeleton::solve(skeleton, pose);
    let mut out = [(math::ZERO, math::Quat::IDENTITY); JOINT_COUNT];
    for j in JOINTS {
        out[j.index()] = skin.box_of(skeleton, j);
    }
    out
}

/// Full box dimensions for a joint on a given build.
pub fn part_size(skeleton: &Skeleton, j: Joint) -> [f32; 3] {
    let h = skeleton.bone(j).half;
    [h[0] * 2.0, h[1] * 2.0, h[2] * 2.0]
}
