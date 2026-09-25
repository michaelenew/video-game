//! The Ridgeback's frame: bones, parts, and the transform between them.
//!
//! The creature used to be ten boxes rigidly welded to one another, carried by
//! a body that could yaw and pitch and nothing else. Five scalars articulated
//! it. That was enough to prove the ride worked and it was never enough to look
//! like an animal: the legs did not move, the neck did not bend, and a tail
//! sweep was the whole creature rotating because the tail could not swing by
//! itself.
//!
//! So it has a skeleton now, on the same terms the fighters' one is built on
//! (see `docs/design/animation.md`): **bones hang off their parents at fixed
//! offsets, and a pose is joint angles only.** Eighteen bones, and every part
//! box is authored in the frame of the bone that carries it.
//!
//! ```text
//!                                        Neck ── Neck2 ── Head
//!                                       /
//!   Tail4 ─ Tail3 ─ Tail2 ─ Tail1 ── Root ── Spine ── Chest
//!                                     |                 |
//!                            Thigh ───┴─── Thigh   Shoulder ─┴─ Shoulder
//!                              |            |          |          |
//!                            Shin         Shin      Forearm    Forearm
//! ```
//!
//! ## Why this is in the simulation rather than the renderer
//!
//! A fighter's skeleton lives in `crates/view` because nothing about a
//! fighter's *hitboxes* depends on it -- `state::hitbox` describes those
//! separately. The creature is the other way round: its parts **are** its
//! geometry. What you collide with, what you can stand on, and what your
//! attacks land on are the boxes, so the boxes have to be where the simulation
//! can see them, and therefore so does the skeleton that moves them.
//!
//! That is also what keeps the overlay honest. The renderer draws the boxes
//! this file places; it does not rebuild them.
//!
//! ## It is still a pure function of `(action, frame)`
//!
//! Nothing here is snapshotted. `Pose` is recomputed every tick from what the
//! creature is doing and how far into it, exactly as the five scalars were, so
//! a rollback reproduces it bit for bit. The clips it reads are a baked table
//! (`beast_baked.rs`), which is a lookup and not a solver -- see
//! `crates/anim/src/beast/` for where the motion is actually authored.

use crate::fixed::Fx;
use crate::math::{Mat3, V3};
use crate::tuning as t;

// ---------------------------------------------------------------------------
// Bones
// ---------------------------------------------------------------------------

pub const ROOT: usize = 0;
pub const SPINE: usize = 1;
pub const CHEST: usize = 2;
pub const NECK: usize = 3;
pub const NECK2: usize = 4;
pub const HEAD: usize = 5;
pub const TAIL1: usize = 6;
pub const TAIL2: usize = 7;
pub const TAIL3: usize = 8;
pub const TAIL4: usize = 9;
pub const SHOULDER_L: usize = 10;
pub const FOREARM_L: usize = 11;
pub const SHOULDER_R: usize = 12;
pub const FOREARM_R: usize = 13;
pub const THIGH_L: usize = 14;
pub const SHIN_L: usize = 15;
pub const THIGH_R: usize = 16;
pub const SHIN_R: usize = 17;

pub const BONES: usize = 18;

/// A bone with no parent. `ROOT`'s, and nothing else's.
const NO_PARENT: usize = usize::MAX;

pub const BONE_NAMES: [&str; BONES] = [
    "root",
    "spine",
    "chest",
    "neck",
    "neck2",
    "head",
    "tail1",
    "tail2",
    "tail3",
    "tail4",
    "shoulder.l",
    "forearm.l",
    "shoulder.r",
    "forearm.r",
    "thigh.l",
    "shin.l",
    "thigh.r",
    "shin.r",
];

/// Parents always come before their children, so the forward pass is one loop
/// with no recursion and no sorting -- the same property the fighters'
/// skeleton relies on.
pub const PARENTS: [usize; BONES] = [
    NO_PARENT, // root
    ROOT,      // spine
    SPINE,     // chest
    CHEST,     // neck
    NECK,      // neck2
    NECK2,     // head
    ROOT,      // tail1
    TAIL1,     // tail2
    TAIL2,     // tail3
    TAIL3,     // tail4
    CHEST,     // shoulder.l
    SHOULDER_L, CHEST, // shoulder.r
    SHOULDER_R, ROOT, // thigh.l
    THIGH_L, ROOT, // thigh.r
    THIGH_R,
];

const fn v(x: (i32, i32), y: (i32, i32), z: (i32, i32)) -> V3 {
    V3::new(
        Fx::ratio(x.0, x.1),
        Fx::ratio(y.0, y.1),
        Fx::ratio(z.0, z.1),
    )
}

/// Where each bone sits in its parent's rest frame, in metres.
///
/// `ROOT`'s entry is where the hips sit above the point on the ground the
/// creature stands at: `+x` forward toward the head, `+y` up, `+z` to its own
/// right.
///
/// A constant rather than a knob for the same reason the part boxes are: this
/// is the animal's *shape*, it never changes during a fight, and the number
/// you actually reach for while playing -- how big is it -- is
/// `tuning::monster_scale`, which multiplies all of it.
///
/// The numbers make an animal about **thirteen metres nose to tail, standing
/// five and a half metres at the back on long legs**. The height is in the legs
/// rather than in the bulk on purpose, and both halves of that are load-bearing:
/// the back is above every full hop but one and the legs are the only part of
/// it a fighter on the floor can reach, which is what makes the ground game a
/// route up rather than a chore. See `docs/design/monsters.md` §1.
///
/// **A metre taller since 2026-09-23.** At 3.95 m hips the tail's middle sat at
/// 3.4 m and four of six classes walked up it from the floor, so the climb was
/// free for most of the roster. The metre went into the legs -- half into each
/// segment -- and the collapse clips went a metre deeper to match, so every
/// *earned* route (a stumble, a topple, the slam's crash) lands where it did.
pub const REST: [V3; BONES] = [
    v((-16, 10), (495, 100), (0, 1)), // root: the hips, 4.95 m up
    v((135, 100), (12, 100), (0, 1)), // spine
    v((145, 100), (5, 100), (0, 1)),  // chest
    v((100, 100), (30, 100), (0, 1)), // neck
    v((115, 100), (18, 100), (0, 1)), // neck2
    v((95, 100), (-5, 100), (0, 1)),  // head
    // The tail is **carried level** off the hips and only the last two
    // segments droop. It used to droop from the root, which put its base a
    // hand's width inside a full hop and its middle well inside one: the free
    // route up, for most of the roster. Carried level, the tail is a tread you
    // reach from the haunch or from a platform, not from the floor.
    v((-125, 100), (5, 100), (0, 1)),
    v((-130, 100), (-5, 100), (0, 1)),
    v((-125, 100), (-45, 100), (0, 1)),
    v((-115, 100), (-60, 100), (0, 1)),
    // Forelegs, off the chest. Long: the animal's height is in its legs rather
    // than in its bulk, which is what puts its back out of reach without
    // making it so long that the arena stops having room for a fight.
    v((-10, 100), (-60, 100), (-95, 100)),
    v((0, 1), (-225, 100), (0, 1)),
    v((-10, 100), (-60, 100), (95, 100)),
    v((0, 1), (-225, 100), (0, 1)),
    // Hindlegs, off the hips. Heavier and set wider: this is where the animal
    // pushes from.
    v((-25, 100), (-55, 100), (-92, 100)),
    v((15, 100), (-220, 100), (0, 1)),
    v((-25, 100), (-55, 100), (92, 100)),
    v((15, 100), (-220, 100), (0, 1)),
];

/// Which side of the body a bone is on: `-1` left, `+1` right, `0` centre.
///
/// The mirror lives in the skeleton rather than in the animation, exactly as it
/// does for the fighters. A pose that spreads both forelegs is the same number
/// twice, and mirroring a clip is a swap of the two sides and nothing else.
pub const SIDE: [i32; BONES] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, -1, 1, 1, -1, -1, 1, 1];

// ---------------------------------------------------------------------------
// Parts: the boxes, and which bone carries each
// ---------------------------------------------------------------------------

pub const P_HEAD: usize = 0;
pub const P_NECK: usize = 1;
pub const P_NAPE: usize = 2;
pub const P_CHEST: usize = 3;
pub const P_RIDGE: usize = 4;
pub const P_BARREL: usize = 5;
pub const P_HAUNCH: usize = 6;
pub const P_TAIL_BASE: usize = 7;
pub const P_TAIL_MID: usize = 8;
pub const P_TAIL_TIP: usize = 9;
pub const P_FORELEG_L: usize = 10;
pub const P_FORELEG_R: usize = 11;
pub const P_HINDLEG_L: usize = 12;
pub const P_HINDLEG_R: usize = 13;
pub const P_FOREFOOT_L: usize = 14;
pub const P_FOREFOOT_R: usize = 15;
pub const P_HINDFOOT_L: usize = 16;
pub const P_HINDFOOT_R: usize = 17;

pub const PARTS: usize = 18;

pub const PART_NAMES: [&str; PARTS] = [
    "head",
    "neck",
    "nape",
    "shoulders",
    "ridge",
    "barrel",
    "haunch",
    "tail",
    "tail, middle",
    "tail tip",
    "left foreleg",
    "right foreleg",
    "left hindleg",
    "right hindleg",
    "left forefoot",
    "right forefoot",
    "left hindfoot",
    "right hindfoot",
];

/// One box of the creature, in the frame of the bone that carries it.
#[derive(Clone, Copy, Debug)]
pub struct Shape {
    pub min: V3,
    pub max: V3,
    /// The bone this box hangs off.
    pub bone: usize,
    /// You can stand on its top face.
    pub mountable: bool,
    /// You collide with it. The two weak points are neither solid nor
    /// mountable: they are soft strips lying over the armour, not walls, so you
    /// walk over them freely and that is the point.
    pub solid: bool,
    /// Breaking it costs the creature something. The four lower legs, and only
    /// those: the ground game is about feet.
    pub breakable: bool,
}

const fn part(
    min: V3,
    max: V3,
    bone: usize,
    mountable: bool,
    solid: bool,
    breakable: bool,
) -> Shape {
    Shape {
        min,
        max,
        bone,
        mountable,
        solid,
        breakable,
    }
}

/// The creature's proportions, at scale one.
pub const SHAPES: [Shape; PARTS] = [
    // Head: armoured, and the bite's hitbox.
    part(
        v((-15, 100), (-50, 100), (-52, 100)),
        v((125, 100), (45, 100), (52, 100)),
        HEAD,
        false,
        true,
        false,
    ),
    // Neck: armoured, and between it and the head a jump-in to the back from
    // the front has nowhere to land.
    part(
        v((-5, 100), (-42, 100), (-44, 100)),
        v((100, 100), (30, 100), (44, 100)),
        NECK2,
        false,
        true,
        false,
    ),
    // **The nape.** The base of the neck, where the plates part to let the head
    // turn. Unarmoured, and reachable only from the animal's own shoulders --
    // which is what the climb buys you. Set at the same height above the
    // shoulders that the ridge sits above the back, and for the same reason.
    // See `docs/design/monsters.md` §1.
    part(
        v((5, 100), (20, 100), (-36, 100)),
        v((105, 100), (78, 100), (36, 100)),
        NECK,
        false,
        false,
        false,
    ),
    // The shoulders. Mountable, and the step between the back and the nape.
    part(
        v((-25, 100), (-80, 100), (-122, 100)),
        v((105, 100), (55, 100), (122, 100)),
        CHEST,
        true,
        true,
        false,
    ),
    // **The ridge**: the soft strip along the spine, and the thing the animal is
    // named for. It stands proud of the back rather than lying flush with it,
    // which is not decoration -- a target at a rider's ankles is a target a
    // level swing passes straight over, and the swing's pitch is dead-zoned
    // through the first forty-five degrees below the horizon on purpose (see
    // `docs/design/aiming.md`). Waist height on somebody standing beside it is
    // what makes hitting it the ordinary thing to do rather than a trick.
    part(
        v((-55, 100), (50, 100), (-38, 100)),
        v((135, 100), (118, 100), (38, 100)),
        SPINE,
        false,
        false,
        false,
    ),
    part(
        v((-115, 100), (-85, 100), (-132, 100)),
        v((140, 100), (55, 100), (132, 100)),
        SPINE,
        true,
        true,
        false,
    ),
    part(
        v((-105, 100), (-80, 100), (-124, 100)),
        v((60, 100), (58, 100), (124, 100)),
        ROOT,
        true,
        true,
        false,
    ),
    // The tail. Mountable along its first two segments, so a rider who comes up
    // the back of the animal has somewhere to walk.
    //
    // **Consecutive mountable parts overlap along `x`** -- tail into haunch,
    // haunch into barrel, barrel into shoulders. Boxes that merely met would
    // leave a seam a few centimetres wide with nothing mountable in it, and a
    // rider walking forward would step into it and fall off an animal that
    // looks continuous. The climb is a staircase, so the treads have to touch.
    part(
        v((-125, 100), (-46, 100), (-62, 100)),
        v((45, 100), (42, 100), (62, 100)),
        TAIL1,
        true,
        true,
        false,
    ),
    part(
        v((-125, 100), (-36, 100), (-46, 100)),
        v((35, 100), (34, 100), (46, 100)),
        TAIL2,
        true,
        true,
        false,
    ),
    // The tip covers the last two segments: it is the sweep's hitbox and
    // nothing stands on it.
    part(
        v((-240, 100), (-28, 100), (-34, 100)),
        v((5, 100), (26, 100), (34, 100)),
        TAIL3,
        false,
        true,
        false,
    ),
    // Upper legs. Armoured, and not what breaks.
    part(
        v((-36, 100), (-232, 100), (-38, 100)),
        v((38, 100), (30, 100), (38, 100)),
        SHOULDER_L,
        false,
        true,
        false,
    ),
    part(
        v((-36, 100), (-232, 100), (-38, 100)),
        v((38, 100), (30, 100), (38, 100)),
        SHOULDER_R,
        false,
        true,
        false,
    ),
    part(
        v((-42, 100), (-226, 100), (-42, 100)),
        v((42, 100), (34, 100), (42, 100)),
        THIGH_L,
        false,
        true,
        false,
    ),
    part(
        v((-42, 100), (-226, 100), (-42, 100)),
        v((42, 100), (34, 100), (42, 100)),
        THIGH_R,
        false,
        true,
        false,
    ),
    // **The feet.** Soft, low, and the only thing on the animal a fighter
    // standing on the floor can reach without a jump. Breaking one drops that
    // corner and puts the creature on its knee, which is the ground game's
    // whole reward -- see `monster::Monster::break_a_leg`.
    part(
        v((-32, 100), (-227, 100), (-36, 100)),
        v((52, 100), (20, 100), (36, 100)),
        FOREARM_L,
        false,
        true,
        true,
    ),
    part(
        v((-32, 100), (-227, 100), (-36, 100)),
        v((52, 100), (20, 100), (36, 100)),
        FOREARM_R,
        false,
        true,
        true,
    ),
    part(
        v((-34, 100), (-222, 100), (-38, 100)),
        v((50, 100), (20, 100), (38, 100)),
        SHIN_L,
        false,
        true,
        true,
    ),
    part(
        v((-34, 100), (-222, 100), (-38, 100)),
        v((50, 100), (20, 100), (38, 100)),
        SHIN_R,
        false,
        true,
        true,
    ),
];

/// The four legs, front to back, as `(upper part, foot part, bone chain root)`.
///
/// Named rather than derived because three separate things want to walk the
/// legs in a known order -- the break rules, the gait, and the lean a broken
/// leg produces -- and each inferring it from the part table its own way is how
/// they come to disagree.
pub const LEGS: [Leg; 4] = [
    Leg {
        upper: P_FORELEG_L,
        foot: P_FOREFOOT_L,
        hip: SHOULDER_L,
        knee: FOREARM_L,
        front: true,
        side: -1,
    },
    Leg {
        upper: P_FORELEG_R,
        foot: P_FOREFOOT_R,
        hip: SHOULDER_R,
        knee: FOREARM_R,
        front: true,
        side: 1,
    },
    Leg {
        upper: P_HINDLEG_L,
        foot: P_HINDFOOT_L,
        hip: THIGH_L,
        knee: SHIN_L,
        front: false,
        side: -1,
    },
    Leg {
        upper: P_HINDLEG_R,
        foot: P_HINDFOOT_R,
        hip: THIGH_R,
        knee: SHIN_R,
        front: false,
        side: 1,
    },
];

#[derive(Clone, Copy, Debug)]
pub struct Leg {
    pub upper: usize,
    pub foot: usize,
    pub hip: usize,
    pub knee: usize,
    pub front: bool,
    /// `-1` left, `+1` right.
    pub side: i32,
}

/// The part's box at the creature's tuned scale.
pub fn shape(index: usize) -> Shape {
    let s = SHAPES[index.min(PARTS - 1)];
    let k = t::monster_scale();
    Shape {
        min: s.min.scale(k),
        max: s.max.scale(k),
        ..s
    }
}

/// A bone's rest offset at the creature's tuned scale.
pub fn rest(bone: usize) -> V3 {
    REST[bone.min(BONES - 1)].scale(t::monster_scale())
}

// ---------------------------------------------------------------------------
// The pose
// ---------------------------------------------------------------------------

/// Three angles per bone, in turns, plus where the hips have been shoved.
///
/// The channels are ordered so that a baked row reads the same way the
/// skeleton does: hips first, then three per bone in `PARENTS` order. That is
/// the same layout the fighters' `view::Pose` uses and for the same reason --
/// the bake writes a flat row and the game reads one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pose {
    /// Where the hips are, in metres from standing.
    pub hips: V3,
    /// `(pitch, yaw, roll)` per bone, in turns. Pitch is nose-up, yaw is toward
    /// the creature's own right, roll is about the bone's own length.
    pub bone: [V3; BONES],
}

pub const CHANNELS: usize = 3 + BONES * 3;

impl Default for Pose {
    fn default() -> Pose {
        Pose::rest()
    }
}

impl Pose {
    pub const fn rest() -> Pose {
        Pose {
            hips: V3::ZERO,
            bone: [V3::ZERO; BONES],
        }
    }

    /// Read a baked row back into a pose.
    pub const fn from_channels(c: &[i32; CHANNELS]) -> Pose {
        let mut p = Pose::rest();
        p.hips = V3::new(Fx::from_raw(c[0]), Fx::from_raw(c[1]), Fx::from_raw(c[2]));
        let mut b = 0;
        while b < BONES {
            p.bone[b] = V3::new(
                Fx::from_raw(c[3 + b * 3]),
                Fx::from_raw(c[4 + b * 3]),
                Fx::from_raw(c[5 + b * 3]),
            );
            b += 1;
        }
        p
    }

    /// Blend toward another pose. Used to walk between two baked samples and to
    /// fade a layer in, and it is a plain lerp on angles because the bones are
    /// sampled densely enough that the short way round is the only way round.
    pub fn blend(&self, other: &Pose, at: Fx) -> Pose {
        let mut out = *self;
        out.hips = crate::math::lerp3(self.hips, other.hips, at);
        for b in 0..BONES {
            out.bone[b] = crate::math::lerp3(self.bone[b], other.bone[b], at);
        }
        out
    }

    /// Add another pose's angles to this one.
    ///
    /// How the procedural layers meet the baked ones: the gait, the head's
    /// tracking and the lean a broken leg produces are all *additive* over
    /// whatever clip is playing, so a creature that is biting while limping is
    /// one pose and not a special case.
    pub fn over(&self, layer: &Pose, amount: Fx) -> Pose {
        let mut out = *self;
        out.hips = self.hips.add(layer.hips.scale(amount));
        for b in 0..BONES {
            out.bone[b] = self.bone[b].add(layer.bone[b].scale(amount));
        }
        out
    }

    pub fn turn(&mut self, bone: usize, pitch: Fx, yaw: Fx, roll: Fx) {
        self.bone[bone] = V3::new(pitch, yaw, roll);
    }

    /// The same pose on the other side of the animal.
    ///
    /// Yaw and roll flip, pitch does not, and the two legs of each pair swap
    /// -- the mirror lives in the skeleton, so that is the whole of it. What
    /// it is for: the tail sweep is baked going one way, and a whip that only
    /// ever went to the creature's right left its left flank a place the
    /// sweep's own cone said it covered and its volume never reached. The
    /// creature picks the side its target is on when it commits, and plays
    /// the clip that way round.
    pub fn mirrored(&self) -> Pose {
        let mut out = *self;
        out.hips = V3::new(self.hips.x, self.hips.y, self.hips.z.neg());
        for b in 0..BONES {
            out.bone[b] = V3::new(self.bone[b].x, self.bone[b].y.neg(), self.bone[b].z.neg());
        }
        for (l, r) in [
            (SHOULDER_L, SHOULDER_R),
            (FOREARM_L, FOREARM_R),
            (THIGH_L, THIGH_R),
            (SHIN_L, SHIN_R),
        ] {
            out.bone.swap(l, r);
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Forward kinematics
// ---------------------------------------------------------------------------

/// One bone, placed in the world.
#[derive(Clone, Copy, Debug)]
pub struct Placed {
    pub at: V3,
    pub rot: Mat3,
}

impl Placed {
    pub const fn local_to_world(&self, local: V3) -> V3 {
        self.at.add(self.rot.apply(local))
    }

    pub const fn world_to_local(&self, world: V3) -> V3 {
        self.rot.unapply(world.sub(self.at))
    }
}

/// The whole creature, placed: every bone's origin and orientation in the
/// world, plus the body frame the ride is measured in.
///
/// Built once per query and handed around rather than recomputed per part. An
/// eighteen-bone forward pass is a few hundred fixed-point multiplies, which is
/// nothing once and is not nothing eighteen times.
#[derive(Clone, Copy, Debug)]
pub struct Rig {
    pub bone: [Placed; BONES],
    /// Where the creature stands, and which way it is pointed, before any pose
    /// is applied. This is the frame the design document calls *body space* --
    /// `+x` toward the head, `+y` up, `+z` to its right -- and it is stable
    /// under the pose on purpose: it is what "the creature is facing you" is
    /// measured against, and a reference that shook when the animal shook would
    /// not be one.
    pub origin: V3,
    pub yaw: Fx,
    pub facing: Mat3,
}

impl Rig {
    /// Place every bone. One forward pass, parents before children.
    pub fn build(origin: V3, yaw: Fx, pose: &Pose) -> Rig {
        let facing = Mat3::from_yaw(yaw);
        let scale = t::monster_scale();
        let mut bone = [Placed {
            at: origin,
            rot: facing,
        }; BONES];

        for b in 0..BONES {
            let a = pose.bone[b];
            let local = Mat3::from_angles(a.x, a.y, a.z);
            let (parent_at, parent_rot) = if PARENTS[b] == NO_PARENT {
                (origin, facing)
            } else {
                let p = bone[PARENTS[b]];
                (p.at, p.rot)
            };
            let mut offset = REST[b].scale(scale);
            if b == ROOT {
                offset = offset.add(pose.hips.scale(scale));
            }
            bone[b] = Placed {
                at: parent_at.add(parent_rot.apply(offset)),
                rot: parent_rot.then(local),
            };
        }

        Rig {
            bone,
            origin,
            yaw,
            facing,
        }
    }

    /// Where a bone would be with no pose on it at all: the rest skeleton,
    /// placed at this rig's origin and yaw.
    ///
    /// What a bone-carried hit volume measures its motion from. The volume is
    /// authored in body space -- "five metres in front of the hips" -- and
    /// riding a bone means moving with it, so it is the bone's displacement
    /// *from rest* that carries the anchor, not the bone's whole offset from
    /// the hips. Adding the latter put the bite four metres past the target it
    /// was thrown at and the sweep four metres behind anybody standing at the
    /// tail root, which is how the tail root came to be a safe place to stand.
    pub fn rest_at(&self, bone: usize) -> V3 {
        let scale = t::monster_scale();
        let mut sum = V3::ZERO;
        let mut b = bone.min(BONES - 1);
        loop {
            sum = sum.add(REST[b].scale(scale));
            if PARENTS[b] == NO_PARENT {
                break;
            }
            b = PARENTS[b];
        }
        self.to_world(sum)
    }

    /// The frame a part's box is authored in.
    pub fn of(&self, part: usize) -> Placed {
        self.bone[SHAPES[part.min(PARTS - 1)].bone]
    }

    /// A point in a part's own frame, put into the world.
    pub fn part_to_world(&self, part: usize, local: V3) -> V3 {
        self.of(part).local_to_world(local)
    }

    /// The inverse: a world point read in a part's own frame. This is what a
    /// rider holds on to, so that the creature moving the bone under them moves
    /// them with it.
    pub fn world_to_part(&self, part: usize, world: V3) -> V3 {
        self.of(part).world_to_local(world)
    }

    /// Body space: the creature's stable frame, ignoring the pose.
    pub fn to_body(&self, world: V3) -> V3 {
        self.facing.unapply(world.sub(self.origin))
    }

    pub fn to_world(&self, local: V3) -> V3 {
        self.origin.add(self.facing.apply(local))
    }

    /// A direction rather than a point: rotated, not translated.
    pub fn dir_to_body(&self, world: V3) -> V3 {
        self.facing.unapply(world)
    }

    pub fn dir_to_world(&self, local: V3) -> V3 {
        self.facing.apply(local)
    }

    /// A direction read in a part's own frame, where the surface normal of
    /// anything you can stand on is simply `+y`.
    ///
    /// The buck is measured here rather than in body space. It is the patch of
    /// animal under the rider's feet that is throwing them, and on a creature
    /// whose tail can swing independently of its shoulders those are two
    /// different answers.
    pub fn dir_to_part(&self, part: usize, world: V3) -> V3 {
        self.of(part).rot.unapply(world)
    }

    /// The world corners of a part's top face, in the order they wind.
    pub fn top_face(&self, part: usize) -> [V3; 4] {
        let sh = shape(part);
        let f = self.of(part);
        let c = |x: Fx, z: Fx| f.local_to_world(V3::new(x, sh.max.y, z));
        [
            c(sh.min.x, sh.min.z),
            c(sh.max.x, sh.min.z),
            c(sh.max.x, sh.max.z),
            c(sh.min.x, sh.max.z),
        ]
    }
}

// ---------------------------------------------------------------------------
// Clips
// ---------------------------------------------------------------------------

/// Everything the creature knows how to look like.
///
/// Each is a short run of baked poses in `beast_baked`. The motion itself is
/// authored in `crates/anim/src/beast/` -- keys, eases and a spring solver, the
/// same factory the fighters go through -- and baked from there. Nothing here
/// solves anything; it is a lookup, which is what keeps the pose a pure
/// function of the snapshot.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Clip {
    /// Standing. Loops, and breathes.
    Idle,
    /// One full two-beat of the walk. Loops, and is indexed by **ground
    /// covered** rather than by time -- the same rule the fighters' locomotion
    /// follows, and for the same reason: a cycle on a fixed cadence skates the
    /// moment the body moves at any other speed.
    Walk,
    /// The gallop. A separate cycle rather than a faster walk, because a
    /// quadruped at speed does not use the same footfall order.
    Gallop,
    Bite,
    Stomp,
    Sweep,
    Charge,
    Slam,
    Shake,
    /// Both hind legs, straight back. The one move aimed at whoever is standing
    /// directly behind it.
    Kick,
    /// The tail curls over the back and flings its spikes forward. The one
    /// move whose hit leaves the animal.
    Spray,
    Flinch,
    /// Down on a knee. What a broken leg and a landed crowd-control both
    /// produce, and the way onto its back from the floor.
    Stumble,
    Topple,
    Dead,
}

pub const CLIPS: usize = 15;

impl Clip {
    pub const ALL: [Clip; CLIPS] = [
        Clip::Idle,
        Clip::Walk,
        Clip::Gallop,
        Clip::Bite,
        Clip::Stomp,
        Clip::Sweep,
        Clip::Charge,
        Clip::Slam,
        Clip::Shake,
        Clip::Kick,
        Clip::Spray,
        Clip::Flinch,
        Clip::Stumble,
        Clip::Topple,
        Clip::Dead,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn name(self) -> &'static str {
        match self {
            Clip::Idle => "idle",
            Clip::Walk => "walk",
            Clip::Gallop => "gallop",
            Clip::Bite => "bite",
            Clip::Stomp => "stomp",
            Clip::Sweep => "sweep",
            Clip::Charge => "charge",
            Clip::Slam => "slam",
            Clip::Shake => "shake",
            Clip::Kick => "kick",
            Clip::Spray => "spray",
            Clip::Flinch => "flinch",
            Clip::Stumble => "stumble",
            Clip::Topple => "topple",
            Clip::Dead => "dead",
        }
    }

    /// Does it close on itself? A cycle is sampled on a wheel and a one-shot is
    /// held at its ends.
    pub const fn looping(self) -> bool {
        matches!(self, Clip::Idle | Clip::Walk | Clip::Gallop)
    }

    /// Is this an attack, baked as three phases of equal sample count so that
    /// retuning the frame data in the Oven stretches the animation with it?
    ///
    /// That is the whole reason the table is in **phase** rather than in
    /// frames. An attack clip baked at a fixed length goes out of step with its
    /// own move the first time somebody drags the startup slider, and a contact
    /// pose on the wrong frame teaches the opponent the wrong timing -- which
    /// is worse than no animation.
    pub const fn phased(self) -> bool {
        matches!(
            self,
            Clip::Bite
                | Clip::Stomp
                | Clip::Sweep
                | Clip::Charge
                | Clip::Slam
                | Clip::Shake
                | Clip::Kick
                | Clip::Spray
        )
    }

    /// The clip an attack plays, by the creature's own move numbering.
    pub const fn of_move(kind: u8) -> Clip {
        match kind {
            0 => Clip::Bite,
            1 => Clip::Stomp,
            2 => Clip::Sweep,
            3 => Clip::Charge,
            4 => Clip::Slam,
            5 => Clip::Shake,
            6 => Clip::Kick,
            _ => Clip::Spray,
        }
    }
}

/// Samples per phase of an attack, and per turn of a cycle or one-shot.
///
/// **This is a gameplay number, not a file-size one.** The samples are read
/// back with a straight lerp, so each join between two of them is a corner --
/// and a corner in position is a spike in *acceleration*, which is exactly what
/// the grip test measures. At twelve samples a thirty-four frame shake put
/// three frames between corners and threw braced riders on single frames that
/// had nothing to do with how hard the animal was actually moving.
///
/// About one sample per frame of the longest phase in the move set is the bar,
/// and thirty-two clears it.
pub const PHASE_SAMPLES: usize = 32;
pub const CYCLE_SAMPLES: usize = 32;

/// Read a clip at a fraction of its length, lerping between baked samples.
///
/// `at` runs 0 to 1. For an attack, use [`sample_phase`] instead: an attack's
/// three phases are laid out end to end and each is read on its own.
pub fn sample(clip: Clip, at: Fx) -> Pose {
    let (start, count) = crate::beast_baked::SPAN[clip.index()];
    read(start as usize, count as usize, at, clip.looping())
}

/// Read one phase of an attack: 0 startup, 1 active, 2 recovery.
pub fn sample_phase(clip: Clip, which: u8, at: Fx) -> Pose {
    let (start, count) = crate::beast_baked::SPAN[clip.index()];
    let count = count as usize;
    if !clip.phased() || count < PHASE_SAMPLES * 3 {
        return sample(clip, at);
    }
    let which = (which as usize).min(2);
    read(
        start as usize + which * PHASE_SAMPLES,
        PHASE_SAMPLES,
        at,
        false,
    )
}

fn row(index: usize) -> Pose {
    Pose::from_channels(&crate::beast_baked::FRAMES[index.min(crate::beast_baked::ROWS - 1)])
}

fn read(start: usize, count: usize, at: Fx, looping: bool) -> Pose {
    if count == 0 {
        return Pose::rest();
    }
    if count == 1 {
        return row(start);
    }
    // A cycle has `count` steps round the wheel and its last sample is the one
    // before frame zero comes back; a one-shot has `count - 1` gaps between its
    // ends. Getting that off by one is a cycle that stutters where it loops.
    let steps = if looping { count } else { count - 1 };
    let at = if looping {
        crate::math::wrap_unit(at)
    } else {
        at.clamp(Fx::ZERO, Fx::ONE)
    };
    let scaled = at.mul(Fx::from_int(steps as i32));
    let whole = scaled.to_int().clamp(0, steps as i32) as usize;
    let frac = scaled.sub(Fx::from_int(whole as i32));
    let a = start + whole.min(count - 1);
    let b = if looping {
        start + (whole + 1) % count
    } else {
        start + (whole + 1).min(count - 1)
    };
    row(a).blend(&row(b), frac)
}
