//! A creature's frame: bones, parts, and the transform between them.
//!
//! **Machinery only.** Which bones an animal has, where its boxes are, how its
//! legs are laid out and what its clips look like is a species' data, declared
//! in its own file under [`crate::species`]; this file is what turns that data
//! into a skeleton standing in the world. The Ridgeback is the first species
//! (`species/ridgeback`), and its skeleton is drawn there.
//!
//! It has a skeleton on the same terms the fighters' one is built on (see
//! `docs/design/animation.md`): **bones hang off their parents at fixed
//! offsets, and a pose is joint angles only.** Every part box is authored in
//! the frame of the bone that carries it.
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
//! creature is doing and how far into it, so a rollback reproduces it bit for
//! bit. The clips it reads are a species' baked table, which is a lookup and
//! not a solver -- see `crates/anim/src/beast/` for where the motion is
//! actually authored.
//!
//! ## Sizes
//!
//! Arrays are sized to the largest body in the cast rather than per species,
//! so a pose and a rig are plain values with no allocation: up to
//! [`MAX_BONES`] bones and [`MAX_PARTS`] parts (the Siegeshell's forty-part
//! rig is the one these were chosen against). Loops run over the species' own
//! counts, never over the maximum.

use crate::fixed::Fx;
use crate::math::{Mat3, V3};
use crate::species::Species;

/// The most bones any species has. See the module docs.
pub const MAX_BONES: usize = 40;
/// The most parts any species has. Fits the six bits a rider's mount keeps
/// for it (see `state::Player::mount`).
pub const MAX_PARTS: usize = 48;
/// The most parts on one body that can be broken, and so carry a health of
/// their own in the snapshot. Health lives on the breakable parts only: a
/// forty-part rig that kept a number for every box would be spending most of
/// the snapshot on armour that never changes.
pub const MAX_BREAKABLE: usize = 12;

/// A bone with no parent: the root's, and nothing else's.
pub const NO_PARENT: usize = usize::MAX;

/// One bone of a species' skeleton.
#[derive(Clone, Copy, Debug)]
pub struct Bone {
    pub name: &'static str,
    /// Parents always come before their children, so the forward pass is one
    /// loop with no recursion and no sorting -- the same property the
    /// fighters' skeleton relies on. [`NO_PARENT`] for the root.
    pub parent: usize,
    /// Where the bone sits in its parent's rest frame, in metres at scale
    /// one. The root's is where the hips sit above the point on the ground the
    /// creature stands at: `+x` forward toward the head, `+y` up, `+z` to its
    /// own right.
    pub rest: V3,
    /// Which side of the body: `-1` left, `+1` right, `0` centre. The mirror
    /// lives in the skeleton rather than in the animation, exactly as it does
    /// for the fighters.
    pub side: i32,
}

/// Write a bone. `const` so a species' skeleton is a table.
pub const fn bone(name: &'static str, parent: usize, rest: V3, side: i32) -> Bone {
    Bone {
        name,
        parent,
        rest,
        side,
    }
}

/// Metres written as two ratios per axis, for authoring rest offsets and part
/// boxes readably: `v((-16, 10), (495, 100), (0, 1))`.
pub const fn v(x: (i32, i32), y: (i32, i32), z: (i32, i32)) -> V3 {
    V3::new(
        Fx::ratio(x.0, x.1),
        Fx::ratio(y.0, y.1),
        Fx::ratio(z.0, z.1),
    )
}

// ---------------------------------------------------------------------------
// Parts: the boxes, and which bone carries each
// ---------------------------------------------------------------------------

/// One box of the creature, in the frame of the bone that carries it.
#[derive(Clone, Copy, Debug)]
pub struct Shape {
    pub min: V3,
    pub max: V3,
    /// The bone this box hangs off.
    pub bone: usize,
    /// You can stand on its top face.
    pub mountable: bool,
    /// You collide with it. A weak point lying over the armour is usually
    /// neither solid nor mountable: a soft strip, not a wall, so you walk over
    /// it freely and that is the point.
    pub solid: bool,
    /// Breaking it costs the creature something. Only these carry a health of
    /// their own; see [`MAX_BREAKABLE`].
    pub breakable: bool,
}

/// One part of a species: its box, its name, and what hitting it means.
#[derive(Clone, Copy, Debug)]
pub struct Part {
    pub name: &'static str,
    pub shape: Shape,
    /// **A weak point.** Damage to one fills the poise pool, and a full pool
    /// is a topple. Declared here rather than known by name, so the rule reads
    /// the table and a new species' weak points need no code.
    pub weak: bool,
    /// The species knob holding this part's damage multiplier: below one is
    /// armour, above one is soft. An index into the species' own knobs (see
    /// `species::Species::own`), so several parts can share one -- the four
    /// feet are one number.
    pub vuln: u16,
}

/// Declare a part. Solid, not mountable, not breakable, not weak: the armour
/// everything starts as. The builder methods say what is different.
pub const fn part(name: &'static str, bone: usize, min: V3, max: V3, vuln: u16) -> Part {
    Part {
        name,
        shape: Shape {
            min,
            max,
            bone,
            mountable: false,
            solid: true,
            breakable: false,
        },
        weak: false,
        vuln,
    }
}

impl Part {
    /// You can stand on it.
    pub const fn mountable(mut self) -> Part {
        self.shape.mountable = true;
        self
    }
    /// You pass through it: a soft strip lying over the armour.
    pub const fn soft(mut self) -> Part {
        self.shape.solid = false;
        self
    }
    /// It has a health, and breaking it costs the creature something.
    pub const fn breakable(mut self) -> Part {
        self.shape.breakable = true;
        self
    }
    /// Damage here fills the poise pool.
    pub const fn weak_point(mut self) -> Part {
        self.weak = true;
        self
    }
}

/// Which of a species' parts are breakable, in part order: the slot a
/// breakable part's health is kept in, and nothing else. Worked out once, at
/// compile time, from the flags, so the list and the flags cannot disagree.
pub const fn breakables<const N: usize>(parts: &[Part; N]) -> ([u8; MAX_BREAKABLE], usize) {
    let mut out = [u8::MAX; MAX_BREAKABLE];
    let mut count = 0;
    let mut i = 0;
    while i < N {
        if parts[i].shape.breakable {
            assert!(
                count < MAX_BREAKABLE,
                "more breakable parts than MAX_BREAKABLE"
            );
            out[count] = i as u8;
            count += 1;
        }
        i += 1;
    }
    (out, count)
}

/// One leg, as three separate things want to walk it in a known order -- the
/// break rules, the gait, and the lean a broken leg produces -- and each
/// inferring it from the part table its own way is how they come to disagree.
#[derive(Clone, Copy, Debug)]
pub struct Leg {
    pub upper: usize,
    pub foot: usize,
    pub hip: usize,
    pub knee: usize,
    /// Which end of the body. A foreleg buckles forward at the knee and a
    /// hindleg back at the hock, and the lever a sagging leg bends against is
    /// measured from the hips to where this leg hangs.
    pub front: bool,
    /// `-1` left, `+1` right.
    pub side: i32,
}

// ---------------------------------------------------------------------------
// The pose
// ---------------------------------------------------------------------------

/// Three angles per bone, in turns, plus where the hips have been shoved.
///
/// The channels are ordered so that a baked row reads the same way the
/// skeleton does: hips first, then three per bone in parent order. That is
/// the same layout the fighters' `view::Pose` uses and for the same reason --
/// the bake writes a flat row and the game reads one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pose {
    /// Where the hips are, in metres from standing.
    pub hips: V3,
    /// `(pitch, yaw, roll)` per bone, in turns. Pitch is nose-up, yaw is toward
    /// the creature's own right, roll is about the bone's own length.
    pub bone: [V3; MAX_BONES],
    /// How many of `bone` the species has. Everything loops to here.
    pub bones: u8,
}

/// Channels in a baked row for a skeleton of `bones` bones.
pub const fn channels(bones: usize) -> usize {
    3 + bones * 3
}

impl Pose {
    pub const fn rest(bones: usize) -> Pose {
        Pose {
            hips: V3::ZERO,
            bone: [V3::ZERO; MAX_BONES],
            bones: bones as u8,
        }
    }

    /// Read a baked row back into a pose. The row's length is the channel
    /// count, which is what says how many bones it has.
    pub fn from_channels(c: &[i32]) -> Pose {
        let bones = (c.len().saturating_sub(3) / 3).min(MAX_BONES);
        let mut p = Pose::rest(bones);
        p.hips = V3::new(Fx::from_raw(c[0]), Fx::from_raw(c[1]), Fx::from_raw(c[2]));
        for b in 0..bones {
            p.bone[b] = V3::new(
                Fx::from_raw(c[3 + b * 3]),
                Fx::from_raw(c[4 + b * 3]),
                Fx::from_raw(c[5 + b * 3]),
            );
        }
        p
    }

    /// Blend toward another pose. Used to walk between two baked samples and to
    /// fade a layer in, and it is a plain lerp on angles because the bones are
    /// sampled densely enough that the short way round is the only way round.
    pub fn blend(&self, other: &Pose, at: Fx) -> Pose {
        let mut out = *self;
        out.hips = crate::math::lerp3(self.hips, other.hips, at);
        for b in 0..self.bones as usize {
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
        for b in 0..self.bones as usize {
            out.bone[b] = self.bone[b].add(layer.bone[b].scale(amount));
        }
        out
    }

    pub fn turn(&mut self, bone: usize, pitch: Fx, yaw: Fx, roll: Fx) {
        self.bone[bone] = V3::new(pitch, yaw, roll);
    }

    /// The same pose on the other side of the animal.
    ///
    /// Yaw and roll flip, pitch does not, and the two bones of each mirrored
    /// pair swap -- the mirror lives in the skeleton, so that is the whole of
    /// it. What it is for: a move baked going one way, played toward the side
    /// its target is on (see `species::MoveDecl::mirrors_to_target_side`).
    pub fn mirrored(&self, pairs: &[(usize, usize)]) -> Pose {
        let mut out = *self;
        out.hips = V3::new(self.hips.x, self.hips.y, self.hips.z.neg());
        for b in 0..self.bones as usize {
            out.bone[b] = V3::new(self.bone[b].x, self.bone[b].y.neg(), self.bone[b].z.neg());
        }
        for (l, r) in pairs {
            out.bone.swap(*l, *r);
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
    pub bone: [Placed; MAX_BONES],
    /// Where the creature stands, and which way it is pointed, before any pose
    /// is applied. This is the frame the design document calls *body space* --
    /// `+x` toward the head, `+y` up, `+z` to its right -- and it is stable
    /// under the pose on purpose: it is what "the creature is facing you" is
    /// measured against, and a reference that shook when the animal shook would
    /// not be one.
    pub origin: V3,
    pub yaw: Fx,
    pub facing: Mat3,
    /// Whose skeleton this is: the table every part and bone lookup reads.
    pub species: &'static Species,
}

impl Rig {
    /// Place every bone. One forward pass, parents before children.
    pub fn build(species: &'static Species, origin: V3, yaw: Fx, pose: &Pose) -> Rig {
        let facing = Mat3::from_yaw(yaw);
        let scale = species.scale();
        let mut bone = [Placed {
            at: origin,
            rot: facing,
        }; MAX_BONES];

        for (b, def) in species.bones.iter().enumerate() {
            let a = pose.bone[b];
            let local = Mat3::from_angles(a.x, a.y, a.z);
            let (parent_at, parent_rot) = if def.parent == NO_PARENT {
                (origin, facing)
            } else {
                let p = bone[def.parent];
                (p.at, p.rot)
            };
            let mut offset = def.rest.scale(scale);
            if def.parent == NO_PARENT {
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
            species,
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
        let bones = self.species.bones;
        let scale = self.species.scale();
        let mut sum = V3::ZERO;
        let mut b = bone.min(bones.len() - 1);
        loop {
            sum = sum.add(bones[b].rest.scale(scale));
            if bones[b].parent == NO_PARENT {
                break;
            }
            b = bones[b].parent;
        }
        self.to_world(sum)
    }

    /// The frame a part's box is authored in.
    pub fn of(&self, part: usize) -> Placed {
        let parts = self.species.parts;
        self.bone[parts[part.min(parts.len() - 1)].shape.bone]
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
        let sh = self.species.shape(part);
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

/// One of a species' clips, as the table in its baked file lays it out.
///
/// Each is a short run of baked poses. The motion itself is authored in
/// `crates/anim/src/beast/<species>/` -- keys, eases and a spring solver, the
/// same factory the fighters go through -- and baked from there. Nothing here
/// solves anything; it is a lookup, which is what keeps the pose a pure
/// function of the snapshot.
#[derive(Clone, Copy, Debug)]
pub struct ClipDecl {
    pub name: &'static str,
    /// Does it close on itself? A cycle is sampled on a wheel and a one-shot
    /// is held at its ends.
    pub looping: bool,
    /// Is this an attack, baked as three phases of equal sample count so that
    /// retuning the frame data in the Oven stretches the animation with it?
    ///
    /// That is the whole reason the table is in **phase** rather than in
    /// frames. An attack clip baked at a fixed length goes out of step with its
    /// own move the first time somebody drags the startup slider, and a contact
    /// pose on the wrong frame teaches the opponent the wrong timing -- which
    /// is worse than no animation.
    pub phased: bool,
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
///
/// A clip the species has not baked yet -- a new species before its first
/// `bake_beast` -- reads as the rest pose.
pub fn sample(species: &Species, clip: usize, at: Fx) -> Pose {
    let Some(&(start, count)) = species.span.get(clip) else {
        return Pose::rest(species.bones.len());
    };
    read(
        species,
        start as usize,
        count as usize,
        at,
        species.clips[clip].looping,
    )
}

/// Read one phase of an attack: 0 startup, 1 active, 2 recovery.
pub fn sample_phase(species: &Species, clip: usize, which: u8, at: Fx) -> Pose {
    let Some(&(start, count)) = species.span.get(clip) else {
        return Pose::rest(species.bones.len());
    };
    let count = count as usize;
    if !species.clips[clip].phased || count < PHASE_SAMPLES * 3 {
        return sample(species, clip, at);
    }
    let which = (which as usize).min(2);
    read(
        species,
        start as usize + which * PHASE_SAMPLES,
        PHASE_SAMPLES,
        at,
        false,
    )
}

fn row(species: &Species, index: usize) -> Pose {
    Pose::from_channels((species.row)(index.min(species.rows - 1)))
}

fn read(species: &Species, start: usize, count: usize, at: Fx, looping: bool) -> Pose {
    if count == 0 {
        return Pose::rest(species.bones.len());
    }
    if count == 1 {
        return row(species, start);
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
    row(species, a).blend(&row(species, b), frac)
}
