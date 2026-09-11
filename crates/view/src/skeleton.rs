//! The frame: bones, joints, proportions, and forward kinematics.
//!
//! Before this existed, a character was six boxes each given an absolute
//! position and rotation by every pose. That works for five clips and falls
//! apart at fifty: nothing holds the elbow to the shoulder, so every pose has
//! to re-derive where the hand goes, and a pose authored for one body cannot
//! be played on another.
//!
//! A skeleton fixes both. Bones hang off their parents at fixed offsets, so a
//! pose is **joint angles only** -- and joint angles are proportion-free, which
//! is what lets one authored clip play correctly on the Bulwark's heavy frame
//! and the Dual mage's slight one without being re-authored.
//!
//! ```text
//!                   Head
//!                    |
//!   Hand ─ Forearm ─ Arm ─ Chest ─ Arm ─ Forearm ─ Hand
//!                           |
//!                         Spine
//!                           |
//!             Thigh ────── Root ────── Thigh
//!               |                        |
//!             Shin                     Shin
//!               |                        |
//!             Foot                     Foot
//! ```
//!
//! Two members per limb, as asked: a thigh and a shin, an upper arm and a
//! forearm. Hands and feet are along for the ride because they cost three
//! channels each and they are what makes a footfall land and a weapon have
//! somewhere to be. The torso is two members -- a spine that bends at the waist
//! and a chest that turns on top of it -- because one rigid torso cannot both
//! bend forward into a lunge and twist into a swing, and every melee animation
//! wants to do both at once.
//!
//! ## Rest pose and the sign conventions
//!
//! With every channel at zero the character stands straight, arms at its sides:
//! limbs point **down**, spine and head point **up**, feet point **forward**.
//! Every joint frame is axis-aligned at rest, which is what makes one uniform
//! angle decomposition work for all sixteen of them.
//!
//! Each joint carries three named angles rather than a raw Euler triple:
//!
//! - **swing** -- forward is positive, everywhere. A hip swings the leg
//!   forward; a knee "swings" by folding, heel toward backside; a spine leans
//!   forward. The sign flip that makes all of those agree lives in the bone,
//!   not in the animation.
//! - **spread** -- *away from the body's midline* is positive, on both sides.
//!   The mirror lives in the bone too, so a symmetric pose is a symmetric set
//!   of numbers and mirroring a clip is a swap plus nothing.
//! - **twist** -- rotation along the bone's own length.
//!
//! That those three read the same on the left as on the right is the whole
//! reason to name them. `rot[2] = 0.3` on one arm and `-0.3` on the other is
//! the same pose written two ways, and every hand-authored pose in the old
//! file had at least one of them wrong.

use crate::math::{self, Quat, V3};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Joint {
    /// The hips. The only joint with a position as well as a rotation: it is
    /// where the whole body's weight shift, crouch and bob are expressed.
    Root,
    /// Bends at the waist.
    Spine,
    /// Turns on top of the spine. The shoulders hang off this.
    Chest,
    Head,
    ArmL,
    ForearmL,
    HandL,
    ArmR,
    ForearmR,
    HandR,
    ThighL,
    ShinL,
    FootL,
    ThighR,
    ShinR,
    FootR,
}

pub const JOINT_COUNT: usize = 16;

/// Parents always come before their children, so kinematics is one forward
/// pass with no recursion and no sorting.
pub const JOINTS: [Joint; JOINT_COUNT] = [
    Joint::Root,
    Joint::Spine,
    Joint::Chest,
    Joint::Head,
    Joint::ArmL,
    Joint::ForearmL,
    Joint::HandL,
    Joint::ArmR,
    Joint::ForearmR,
    Joint::HandR,
    Joint::ThighL,
    Joint::ShinL,
    Joint::FootL,
    Joint::ThighR,
    Joint::ShinR,
    Joint::FootR,
];

impl Joint {
    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn name(self) -> &'static str {
        match self {
            Joint::Root => "root",
            Joint::Spine => "spine",
            Joint::Chest => "chest",
            Joint::Head => "head",
            Joint::ArmL => "arm.l",
            Joint::ForearmL => "forearm.l",
            Joint::HandL => "hand.l",
            Joint::ArmR => "arm.r",
            Joint::ForearmR => "forearm.r",
            Joint::HandR => "hand.r",
            Joint::ThighL => "thigh.l",
            Joint::ShinL => "shin.l",
            Joint::FootL => "foot.l",
            Joint::ThighR => "thigh.r",
            Joint::ShinR => "shin.r",
            Joint::FootR => "foot.r",
        }
    }

    /// The joint this one hangs from.
    pub const fn parent(self) -> Option<Joint> {
        match self {
            Joint::Root => None,
            Joint::Spine => Some(Joint::Root),
            Joint::Chest => Some(Joint::Spine),
            Joint::Head => Some(Joint::Chest),
            Joint::ArmL | Joint::ArmR => Some(Joint::Chest),
            Joint::ForearmL => Some(Joint::ArmL),
            Joint::HandL => Some(Joint::ForearmL),
            Joint::ForearmR => Some(Joint::ArmR),
            Joint::HandR => Some(Joint::ForearmR),
            Joint::ThighL | Joint::ThighR => Some(Joint::Root),
            Joint::ShinL => Some(Joint::ThighL),
            Joint::FootL => Some(Joint::ShinL),
            Joint::ShinR => Some(Joint::ThighR),
            Joint::FootR => Some(Joint::ShinR),
        }
    }

    /// Is this the left copy of a paired joint? Left joints get their spread
    /// and twist mirrored, which is what makes "away from the body" mean the
    /// same number on both sides.
    pub const fn is_left(self) -> bool {
        matches!(
            self,
            Joint::ArmL
                | Joint::ForearmL
                | Joint::HandL
                | Joint::ThighL
                | Joint::ShinL
                | Joint::FootL
        )
    }

    /// The joint on the other side, or itself for the spine.
    pub const fn opposite(self) -> Joint {
        match self {
            Joint::ArmL => Joint::ArmR,
            Joint::ArmR => Joint::ArmL,
            Joint::ForearmL => Joint::ForearmR,
            Joint::ForearmR => Joint::ForearmL,
            Joint::HandL => Joint::HandR,
            Joint::HandR => Joint::HandL,
            Joint::ThighL => Joint::ThighR,
            Joint::ThighR => Joint::ThighL,
            Joint::ShinL => Joint::ShinR,
            Joint::ShinR => Joint::ShinL,
            Joint::FootL => Joint::FootR,
            Joint::FootR => Joint::FootL,
            other => other,
        }
    }

    /// A hinge bends on one axis only -- a knee and an elbow. The editor hides
    /// the other two channels, and the limits below pin them near zero, because
    /// an elbow that can also be splayed sideways is how a rig starts producing
    /// broken dolls.
    pub const fn is_hinge(self) -> bool {
        matches!(
            self,
            Joint::ForearmL | Joint::ForearmR | Joint::ShinL | Joint::ShinR
        )
    }

    /// Which of the six body groups this belongs to, for looseness and for
    /// grouping the editor's sliders.
    pub const fn group(self) -> Group {
        match self {
            Joint::Root => Group::Root,
            Joint::Spine => Group::Spine,
            Joint::Chest => Group::Chest,
            Joint::Head => Group::Head,
            Joint::ArmL | Joint::ForearmL | Joint::HandL => Group::Arms,
            Joint::ArmR | Joint::ForearmR | Joint::HandR => Group::Arms,
            _ => Group::Legs,
        }
    }

    /// How far down its own limb a joint sits: 0 at the shoulder or hip, 2 at
    /// the hand or foot. Distal joints are given proportionally more lag, which
    /// is where follow-through comes from.
    pub const fn depth(self) -> u8 {
        match self {
            Joint::ForearmL | Joint::ForearmR | Joint::ShinL | Joint::ShinR => 1,
            Joint::HandL | Joint::HandR | Joint::FootL | Joint::FootR => 2,
            _ => 0,
        }
    }
}

/// Body groups, which is the granularity looseness is set at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Root,
    Spine,
    Chest,
    Head,
    Arms,
    Legs,
}

pub const GROUPS: [Group; 6] = [
    Group::Root,
    Group::Spine,
    Group::Chest,
    Group::Head,
    Group::Arms,
    Group::Legs,
];

impl Group {
    pub const fn name(self) -> &'static str {
        match self {
            Group::Root => "root",
            Group::Spine => "spine",
            Group::Chest => "chest",
            Group::Head => "head",
            Group::Arms => "arms",
            Group::Legs => "legs",
        }
    }
}

/// How far a joint is allowed to go, in radians.
///
/// Not decoration. Two things write angles into a pose: a person, and a spring
/// that is deliberately allowed to overshoot. Both will put a knee through the
/// front of the leg given the chance, and a single broken frame is enough to
/// read as a glitch. Clamping at the end of the solve costs nothing and means
/// overshoot can be tuned for feel without being audited for anatomy.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub swing: (f32, f32),
    pub spread: (f32, f32),
    pub twist: (f32, f32),
}

const fn d(v: f32) -> f32 {
    v * (std::f32::consts::PI / 180.0)
}

const fn limits(swing: (f32, f32), spread: (f32, f32), twist: (f32, f32)) -> Limits {
    Limits {
        swing: (d(swing.0), d(swing.1)),
        spread: (d(spread.0), d(spread.1)),
        twist: (d(twist.0), d(twist.1)),
    }
}

impl Limits {
    pub fn clamp(&self, swing: f32, spread: f32, twist: f32) -> (f32, f32, f32) {
        (
            swing.clamp(self.swing.0, self.swing.1),
            spread.clamp(self.spread.0, self.spread.1),
            twist.clamp(self.twist.0, self.twist.1),
        )
    }

    /// The range of one channel, for a slider.
    pub fn channel(&self, c: usize) -> (f32, f32) {
        match c {
            0 => self.swing,
            1 => self.spread,
            _ => self.twist,
        }
    }
}

/// One bone, in the reference body's measurements.
#[derive(Clone, Copy, Debug)]
pub struct Bone {
    /// Where the joint sits in its parent's frame, at rest.
    pub offset: V3,
    /// Which way the bone runs from its joint, at rest. Limbs point down, the
    /// spine points up, feet point forward.
    pub axis: V3,
    /// How long the bone is along that axis.
    pub length: f32,
    /// The stand-in box drawn on it: width, thickness across the bone, and the
    /// third measurement along it. Applied in the bone's own frame.
    pub half: V3,
    /// Where the box sits relative to the joint, in the bone's frame.
    pub box_at: V3,
    /// Multiplies `swing` so that positive means forward for every joint that
    /// can swing forward, and "folded" for the two that cannot.
    pub swing_sign: f32,
    pub limits: Limits,
}

/// The whole frame, at one class's proportions.
#[derive(Clone, Debug)]
pub struct Skeleton {
    pub bones: [Bone; JOINT_COUNT],
    /// Overall size against the reference body. Root translation channels are
    /// authored in reference metres and scaled by this, so a crouch authored
    /// once crouches proportionally on every class.
    pub scale: f32,
    /// Hip height at rest. The reach of the legs hangs off this.
    pub hip_height: f32,
    pub total_height: f32,
}

/// What separates one class's body from another.
///
/// Weight is the most legible thing a character can have -- you can read it
/// across the arena in the first second, before you know a single move -- and
/// `class.rs` already spends real design effort on making the classes differ
/// in the air. This is the same idea in the silhouette, and it costs six
/// numbers.
#[derive(Clone, Copy, Debug)]
pub struct Build {
    /// Overall height against the 1.8 m reference.
    pub scale: f32,
    /// How thick the limbs and torso are drawn. This is the "weight" knob.
    pub bulk: f32,
    /// Arm and leg length, at a fixed overall height: high is rangy, low is
    /// stocky.
    pub limb: f32,
    pub shoulders: f32,
    /// Hip width, which is most of how wide a stance reads.
    pub stance: f32,
    pub head: f32,
}

impl Build {
    /// The middleweight the others are read against -- the Champion, which is
    /// also the baseline everywhere else in the game.
    pub const REFERENCE: Build = Build {
        scale: 1.0,
        bulk: 1.0,
        limb: 1.0,
        shoulders: 1.0,
        stance: 1.0,
        head: 1.0,
    };
}

/// The reference body: 1.8 m tall, hips at 0.98, which is where the simulation's
/// `body_height` sits. If that number is ever retuned, this should follow it --
/// a stand-in that does not match its own hurtbox teaches players the wrong
/// spacing, which is worse than a stand-in that looks crude.
pub const REFERENCE_HEIGHT: f32 = 1.8;
pub const REFERENCE_HIP: f32 = 0.98;

impl Skeleton {
    pub fn new(build: Build) -> Skeleton {
        let Build {
            scale,
            bulk,
            limb,
            shoulders,
            stance,
            head,
        } = build;

        let thigh = 0.46 * limb * scale;
        let shin = 0.41 * limb * scale;
        let upper_arm = 0.285 * limb * scale;
        let forearm = 0.265 * limb * scale;

        // The hips sit exactly as high as the legs are long, so every build
        // stands with its soles on the floor without anyone tuning it. Derived
        // rather than typed: a typed hip height and a retuned leg length is a
        // character hovering, and it is the kind of thing nobody notices until
        // a screenshot goes out.
        let ankle_to_sole = 0.06 * scale;
        let hip_to_thigh = 0.05 * scale;
        let hip = hip_to_thigh + thigh + shin + ankle_to_sole;

        let mut bones = [ROOT_BONE; JOINT_COUNT];
        let set = |bones: &mut [Bone; JOINT_COUNT], j: Joint, b: Bone| bones[j.index()] = b;

        set(
            &mut bones,
            Joint::Root,
            Bone {
                offset: [0.0, hip, 0.0],
                axis: [0.0, 1.0, 0.0],
                length: 0.07 * scale,
                half: [0.16 * bulk * stance, 0.09, 0.11 * bulk].map(|v| v * scale),
                box_at: [0.0, -0.01 * scale, 0.0],
                swing_sign: 1.0,
                limits: limits((-40.0, 50.0), (-35.0, 35.0), (-65.0, 65.0)),
            },
        );
        set(
            &mut bones,
            Joint::Spine,
            Bone {
                offset: [0.0, 0.07 * scale, 0.0],
                axis: [0.0, 1.0, 0.0],
                length: 0.27 * scale,
                half: [0.155 * bulk, 0.135, 0.105 * bulk].map(|v| v * scale),
                box_at: [0.0, 0.135 * scale, 0.0],
                swing_sign: 1.0,
                limits: limits((-25.0, 45.0), (-25.0, 25.0), (-35.0, 35.0)),
            },
        );
        set(
            &mut bones,
            Joint::Chest,
            Bone {
                offset: [0.0, 0.27 * scale, 0.0],
                axis: [0.0, 1.0, 0.0],
                length: 0.25 * scale,
                half: [0.19 * bulk * shoulders, 0.125, 0.115 * bulk].map(|v| v * scale),
                box_at: [0.0, 0.125 * scale, 0.0],
                swing_sign: 1.0,
                limits: limits((-20.0, 40.0), (-20.0, 20.0), (-45.0, 45.0)),
            },
        );
        set(
            &mut bones,
            Joint::Head,
            Bone {
                offset: [0.0, 0.25 * scale, 0.0],
                axis: [0.0, 1.0, 0.0],
                length: 0.23 * head * scale,
                half: [0.115 * head, 0.115 * head, 0.115 * head].map(|v| v * scale),
                box_at: [0.0, 0.115 * head * scale, 0.0],
                swing_sign: 1.0,
                limits: limits((-45.0, 45.0), (-35.0, 35.0), (-70.0, 70.0)),
            },
        );

        for (arm, forearm_j, hand, side) in [
            (Joint::ArmL, Joint::ForearmL, Joint::HandL, -1.0),
            (Joint::ArmR, Joint::ForearmR, Joint::HandR, 1.0),
        ] {
            set(
                &mut bones,
                arm,
                Bone {
                    offset: [side * 0.185 * shoulders * scale, 0.15 * scale, 0.0],
                    axis: [0.0, -1.0, 0.0],
                    length: upper_arm,
                    half: [0.055 * bulk, upper_arm / (2.0 * scale), 0.055 * bulk]
                        .map(|v| v * scale),
                    box_at: [0.0, -upper_arm * 0.5, 0.0],
                    swing_sign: -1.0,
                    // A shoulder is the loosest joint on the body and the
                    // animation wants that range: 170 degrees of swing is what
                    // an overhead needs to exist at all.
                    limits: limits((-80.0, 175.0), (-30.0, 150.0), (-95.0, 95.0)),
                },
            );
            set(
                &mut bones,
                forearm_j,
                Bone {
                    offset: [0.0, -upper_arm, 0.0],
                    axis: [0.0, -1.0, 0.0],
                    length: forearm,
                    half: [0.047 * bulk, forearm / (2.0 * scale), 0.047 * bulk].map(|v| v * scale),
                    box_at: [0.0, -forearm * 0.5, 0.0],
                    swing_sign: -1.0,
                    // Flexes one way only, with two degrees of give.
                    limits: limits((-2.0, 150.0), (-4.0, 4.0), (-85.0, 85.0)),
                },
            );
            set(
                &mut bones,
                hand,
                Bone {
                    offset: [0.0, -forearm, 0.0],
                    axis: [0.0, -1.0, 0.0],
                    length: 0.10 * scale,
                    half: [0.045 * bulk, 0.05, 0.03 * bulk].map(|v| v * scale),
                    box_at: [0.0, -0.05 * scale, 0.0],
                    swing_sign: -1.0,
                    limits: limits((-70.0, 70.0), (-30.0, 30.0), (-25.0, 25.0)),
                },
            );
        }

        for (thigh_j, shin_j, foot, side) in [
            (Joint::ThighL, Joint::ShinL, Joint::FootL, -1.0),
            (Joint::ThighR, Joint::ShinR, Joint::FootR, 1.0),
        ] {
            set(
                &mut bones,
                thigh_j,
                Bone {
                    offset: [side * 0.105 * stance * scale, -0.05 * scale, 0.0],
                    axis: [0.0, -1.0, 0.0],
                    length: thigh,
                    half: [0.075 * bulk, thigh / (2.0 * scale), 0.075 * bulk].map(|v| v * scale),
                    box_at: [0.0, -thigh * 0.5, 0.0],
                    swing_sign: -1.0,
                    // Femoral rotation goes further than people expect, and
                    // a crossover run needs most of it.
                    // A hip is the least constrained joint on the body after
                    // the shoulder, and a crossover step needs most of it.
                    limits: limits((-50.0, 120.0), (-50.0, 85.0), (-70.0, 70.0)),
                },
            );
            set(
                &mut bones,
                shin_j,
                Bone {
                    offset: [0.0, -thigh, 0.0],
                    axis: [0.0, -1.0, 0.0],
                    length: shin,
                    half: [0.062 * bulk, shin / (2.0 * scale), 0.062 * bulk].map(|v| v * scale),
                    box_at: [0.0, -shin * 0.5, 0.0],
                    swing_sign: 1.0,
                    // A knee folds backward and does not do anything else.
                    limits: limits((-3.0, 150.0), (-4.0, 4.0), (-10.0, 10.0)),
                },
            );
            set(
                &mut bones,
                foot,
                Bone {
                    offset: [0.0, -shin, 0.0],
                    axis: [0.0, 0.0, 1.0],
                    length: 0.22 * scale,
                    half: [0.055 * bulk, 0.035, 0.11].map(|v| v * scale),
                    box_at: [0.0, -0.025 * scale, 0.05 * scale],
                    // Positive points the toe down.
                    swing_sign: 1.0,
                    limits: limits((-30.0, 50.0), (-18.0, 18.0), (-20.0, 20.0)),
                },
            );
        }

        let head_top = hip
            + bones[Joint::Spine.index()].offset[1]
            + bones[Joint::Chest.index()].offset[1]
            + bones[Joint::Head.index()].offset[1]
            + bones[Joint::Head.index()].length;

        Skeleton {
            bones,
            scale,
            hip_height: hip,
            total_height: head_top,
        }
    }

    pub fn bone(&self, j: Joint) -> &Bone {
        &self.bones[j.index()]
    }

    /// Hip-to-ankle reach, which is what the leg solver is allowed to ask for.
    pub fn leg_reach(&self) -> f32 {
        self.bone(Joint::ThighL).length + self.bone(Joint::ShinL).length
    }

    pub fn arm_reach(&self) -> f32 {
        self.bone(Joint::ArmL).length + self.bone(Joint::ForearmL).length
    }
}

/// A placeholder that is overwritten for every joint; `Bone` has no sensible
/// default and inventing one would be a bug waiting to be shipped.
const ROOT_BONE: Bone = Bone {
    offset: [0.0, 0.0, 0.0],
    axis: [0.0, 1.0, 0.0],
    length: 0.1,
    half: [0.05, 0.05, 0.05],
    box_at: [0.0, 0.0, 0.0],
    swing_sign: 1.0,
    limits: limits((-180.0, 180.0), (-180.0, 180.0), (-180.0, 180.0)),
};

/// Where every joint ended up, in character-local space.
#[derive(Clone, Copy, Debug)]
pub struct Skin {
    /// Joint origins -- the shoulder, the elbow, the ankle.
    pub origin: [V3; JOINT_COUNT],
    /// Joint orientations, composed down the chain.
    pub rot: [Quat; JOINT_COUNT],
}

impl Skin {
    /// Where the far end of a bone is -- the elbow for an upper arm, the
    /// fingertips for a hand. What IK aims at and what a contact sheet draws.
    pub fn tip(&self, skeleton: &Skeleton, j: Joint) -> V3 {
        let b = skeleton.bone(j);
        math::add(
            self.origin[j.index()],
            self.rot[j.index()].rotate(math::scale(b.axis, b.length)),
        )
    }

    /// Centre and orientation of the box drawn on a bone, which is all the
    /// renderer needs.
    pub fn box_of(&self, skeleton: &Skeleton, j: Joint) -> (V3, Quat) {
        let b = skeleton.bone(j);
        (
            math::add(self.origin[j.index()], self.rot[j.index()].rotate(b.box_at)),
            self.rot[j.index()],
        )
    }
}

/// Turn joint angles into positions: one pass, parents before children.
///
/// The composition order is deliberate and worth stating, because everything
/// that inverts it -- the IK solver, the editor's handles -- has to agree:
///
/// ```text
/// local = spread(Z) · swing(X) · twist(Y)
/// ```
///
/// Twist is innermost, so it spins the bone about its own length without
/// disturbing where the bone points. Swing then pitches it forward or back,
/// and spread pushes it out to the side. Reading it outside-in, that is the
/// order a person describes an arm in: out to here, forward to there, rolled
/// like so.
pub fn solve(skeleton: &Skeleton, pose: &crate::pose::Pose) -> Skin {
    let mut skin = Skin {
        origin: [math::ZERO; JOINT_COUNT],
        rot: [Quat::IDENTITY; JOINT_COUNT],
    };

    for j in JOINTS {
        let i = j.index();
        let bone = skeleton.bone(j);
        let local = local_rotation(bone, j, pose.angles(j));

        match j.parent() {
            None => {
                // The root carries the body's own offset, authored in reference
                // metres so a crouch is a crouch on every build.
                let t = pose.root_offset();
                skin.origin[i] = math::add(bone.offset, math::scale(t, skeleton.scale));
                skin.rot[i] = local;
            }
            Some(p) => {
                let pi = p.index();
                skin.origin[i] = math::add(skin.origin[pi], skin.rot[pi].rotate(bone.offset));
                skin.rot[i] = skin.rot[pi].mul(local).normalized();
            }
        }
    }
    skin
}

/// One joint's rotation from its three named angles.
pub fn local_rotation(bone: &Bone, joint: Joint, angles: [f32; 3]) -> Quat {
    let (swing, spread, twist) = bone.limits.clamp(angles[0], angles[1], angles[2]);
    let mirror = if joint.is_left() { -1.0 } else { 1.0 };
    Quat::from_z(spread * mirror)
        .mul(Quat::from_x(swing * bone.swing_sign))
        .mul(Quat::from_y(twist * mirror))
}

/// The inverse of the direction half of `local_rotation`: which swing and
/// spread point a bone's rest axis at `dir`, expressed in the parent's frame.
///
/// Closed form, not a search. Applying the composition above to a bone pointing
/// down gives `(cos·sin p, −cos s·cos p, −sin s)`, and reading `s` and `p` back
/// out of that is one `asin` and one `atan2`.
pub fn aim(bone: &Bone, joint: Joint, dir: V3) -> (f32, f32) {
    let d = math::normalize_or(dir, bone.axis);
    let mirror = if joint.is_left() { -1.0 } else { 1.0 };

    // Written for a bone whose rest axis is -Y, which is every bone IK is ever
    // pointed at. Feet and torso segments are posed directly.
    let swing = (-d[2]).clamp(-1.0, 1.0).asin();
    let spread = d[0].atan2(-d[1]);
    (swing / bone.swing_sign, spread * mirror)
}

/// The named builds. Six numbers per class, and they are meant to be read in
/// one glance against each other rather than in isolation.
pub fn build_for(class: sim::Class) -> Build {
    use sim::Class::*;
    match class {
        // Short, wide, thick, short-limbed. Holds ground for a living and
        // should look like it does.
        Bulwark => Build {
            scale: 0.98,
            bulk: 1.32,
            limb: 0.94,
            shoulders: 1.22,
            stance: 1.18,
            head: 0.94,
        },
        // The reference middleweight.
        Champion => Build::REFERENCE,
        // Lean and long-limbed: the most mobile thing in the arena reads as
        // the lightest.
        ShadowReaver => Build {
            scale: 1.00,
            bulk: 0.82,
            limb: 1.08,
            shoulders: 0.94,
            stance: 0.92,
            head: 0.96,
        },
        // Tall and narrow, with a heavy head: a caster silhouette.
        Elementalist => Build {
            scale: 1.02,
            bulk: 0.90,
            limb: 1.02,
            shoulders: 0.95,
            stance: 0.95,
            head: 1.08,
        },
        // Ordinary proportions carrying more weight than they should. The
        // class that pays with its own health does not look athletic.
        BloodMage => Build {
            scale: 0.99,
            bulk: 1.06,
            limb: 0.98,
            shoulders: 1.04,
            stance: 1.02,
            head: 1.0,
        },
        // The floatiest and the frailest.
        DualMage => Build {
            scale: 1.01,
            bulk: 0.80,
            limb: 1.06,
            shoulders: 0.90,
            stance: 0.90,
            head: 1.04,
        },
    }
}

pub fn skeleton_for(class: sim::Class) -> Skeleton {
    Skeleton::new(build_for(class))
}
