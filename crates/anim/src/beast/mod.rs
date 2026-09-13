//! The Ridgeback's animation, authored.
//!
//! Same factory as the fighters go through -- sparse keys, an ease per gap, a
//! spring per channel, bake the result -- pointed at a different skeleton. The
//! rig is `sim::beast`: eighteen bones, three angles each, and a hip offset.
//!
//! ```text
//! keys + eases + looseness  --[springs, offline]-->  one pose per sample
//! ```
//!
//! Two things are different from the fighters' pipeline, and both come out of
//! the creature being simulation geometry rather than decoration.
//!
//! **The table is in phase, not in frames.** An attack bakes as three runs of
//! twelve samples -- startup, active, recovery -- each read on its own at
//! runtime. Retuning a move's frame counts in the Oven then stretches its
//! animation with it, instead of leaving the contact pose on the wrong frame.
//! The solve underneath is still continuous across the whole move at its
//! current lengths, so the phases join exactly.
//!
//! **The result is fixed point.** It is read by `crates/sim`, which has no
//! floats, so the emitter writes raw 16.16 bits. The solver here is f32 and
//! offline, which is fine: the *table* is what has to be deterministic, and a
//! table is.
//!
//! ## Making it look like an animal
//!
//! The three rules that did most of the work, in case a second creature is ever
//! built on this:
//!
//! - **Nothing starts at the same time.** The hips lead, the spine follows, the
//!   neck follows that, the tail follows last. `Looseness` does it for free by
//!   giving distal groups more lag -- the same mechanism that makes a fighter's
//!   hand trail their shoulder.
//! - **A quadruped's legs are two diagonal pairs.** A walk moves them in a
//!   four-beat sequence, a gallop gathers and throws them; using the same cycle
//!   faster is what makes a horse in a bad game look like a table sliding.
//! - **The body answers its own limbs.** A tail that heavy cannot swing without
//!   the shoulders paying for it, and the counter-rotation is most of what
//!   makes a sweep read as weight rather than as a rotating prop.

pub mod clips;
pub mod sheet;

use crate::ease::Ease;
use crate::spring::Spring;
use crate::{DT, SUBSTEPS};
use sim::beast::{BONES, CHANNELS, CLIPS, CYCLE_SAMPLES, Clip, PHASE_SAMPLES};

// ---------------------------------------------------------------------------
// Poses, in degrees
// ---------------------------------------------------------------------------

/// One pose of the creature. Angles in **degrees**, because that is how a
/// person describes a body; the emitter turns them into turns and then into
/// fixed point.
///
/// Sign conventions match the rig: `pitch` is nose-up, `yaw` is toward the
/// creature's own right, `roll` is about the bone's own length. The mirror
/// lives in the skeleton, so `forelegs(-20.0, 30.0)` puts *both* forelegs in
/// the same shape and a symmetric pose is a symmetric set of numbers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub hips: [f32; 3],
    pub bone: [[f32; 3]; BONES],
}

impl Default for Pose {
    fn default() -> Pose {
        Pose::rest()
    }
}

/// The spine, nose to tail, as bone indices -- the chain most things bend.
pub const SPINE_CHAIN: [usize; 6] = [
    sim::beast::ROOT,
    sim::beast::SPINE,
    sim::beast::CHEST,
    sim::beast::NECK,
    sim::beast::NECK2,
    sim::beast::HEAD,
];

pub const TAIL_CHAIN: [usize; 4] = [
    sim::beast::TAIL1,
    sim::beast::TAIL2,
    sim::beast::TAIL3,
    sim::beast::TAIL4,
];

/// `(hip, knee)` per leg, in `sim::beast::LEGS` order: front left, front right,
/// hind left, hind right.
/// The foot part on each leg, in the same order. What `plant` measures the
/// lower segment's length from.
pub const LEG_PARTS: [usize; 4] = [
    sim::monster::FOREFOOT_L,
    sim::monster::FOREFOOT_R,
    sim::monster::HINDFOOT_L,
    sim::monster::HINDFOOT_R,
];

pub const LEG_BONES: [(usize, usize); 4] = [
    (sim::beast::SHOULDER_L, sim::beast::FOREARM_L),
    (sim::beast::SHOULDER_R, sim::beast::FOREARM_R),
    (sim::beast::THIGH_L, sim::beast::SHIN_L),
    (sim::beast::THIGH_R, sim::beast::SHIN_R),
];

impl Pose {
    pub const fn rest() -> Pose {
        Pose {
            hips: [0.0; 3],
            bone: [[0.0; 3]; BONES],
        }
    }

    pub fn set(mut self, bone: usize, pitch: f32, yaw: f32, roll: f32) -> Pose {
        self.bone[bone] = [pitch, yaw, roll];
        self
    }

    /// Move the hips, in metres from standing.
    pub fn hips(mut self, x: f32, y: f32, z: f32) -> Pose {
        self.hips = [x, y, z];
        self
    }

    pub fn root(self, pitch: f32, yaw: f32, roll: f32) -> Pose {
        self.set(sim::beast::ROOT, pitch, yaw, roll)
    }
    pub fn spine(self, pitch: f32, yaw: f32, roll: f32) -> Pose {
        self.set(sim::beast::SPINE, pitch, yaw, roll)
    }
    pub fn chest(self, pitch: f32, yaw: f32, roll: f32) -> Pose {
        self.set(sim::beast::CHEST, pitch, yaw, roll)
    }
    pub fn head(self, pitch: f32, yaw: f32, roll: f32) -> Pose {
        self.set(sim::beast::HEAD, pitch, yaw, roll)
    }

    /// Bend the neck as a **curve** rather than a hinge: the amount is shared
    /// across both neck bones and the head, weighted toward the base.
    ///
    /// Every early version of the bite had the neck as one rigid bar swinging
    /// from the shoulder, which reads as a digger rather than as an animal.
    /// Sharing the angle is the whole fix and it belongs here rather than in
    /// each key.
    pub fn neck(mut self, pitch: f32, yaw: f32) -> Pose {
        let share = [0.45, 0.35, 0.20];
        for (bone, k) in [sim::beast::NECK, sim::beast::NECK2, sim::beast::HEAD]
            .into_iter()
            .zip(share)
        {
            self.bone[bone][0] += pitch * k;
            self.bone[bone][1] += yaw * k;
        }
        self
    }

    /// Lift the whole tail and swing it, spread along its length and growing
    /// toward the tip -- which is what a tail does and what makes the tip carry
    /// the speed.
    ///
    /// **Positive `lift` raises the tail**, which is the opposite sign to the
    /// rig's own pitch: the tail extends *backwards* from its bones, so nose-up
    /// on a tail bone points the tail at the sky. The flip lives here rather
    /// than in every key, the same way the left-right mirror lives in the
    /// skeleton rather than in every pose.
    pub fn tail(mut self, lift: f32, yaw: f32) -> Pose {
        // The increments sum to one, so `tail(0.0, 60.0)` means sixty degrees
        // at the tip and the bend is spread rather than hinged at one joint.
        let share = [0.18, 0.24, 0.28, 0.30];
        for (bone, k) in TAIL_CHAIN.into_iter().zip(share) {
            self.bone[bone][0] -= lift * k;
            self.bone[bone][1] += yaw * k;
        }
        self
    }

    /// Lift or drop the tail **from its root**, so the whole thing swings as one
    /// beam rather than curling.
    ///
    /// Different from [`tail`](Pose::tail) in the place it matters: `tail`
    /// spreads the bend toward the tip, which is a whip, and this puts it all at
    /// the base, which is the tail being *carried* somewhere.
    pub fn tail_from_base(mut self, lift: f32) -> Pose {
        self.bone[sim::beast::TAIL1][0] -= lift;
        self
    }

    /// Swing the whole tail about its **root**, so it travels as one beam.
    ///
    /// The difference from [`tail`](Pose::tail) is the whole of what a sweep
    /// feels like to ride. `tail` spreads the bend toward the tip, which whips
    /// the tip and leaves the base almost still -- so the tip carries the speed
    /// the hitbox needs, and somebody standing on the base feels nothing.
    /// Swinging from the root moves the base too, which is what throws them.
    pub fn tail_swing_base(mut self, yaw: f32) -> Pose {
        self.bone[sim::beast::TAIL1][1] += yaw;
        self
    }

    /// Roll the tail about its own length, from the root.
    ///
    /// What tips a rider off it. A tail that only yaws carries somebody
    /// standing on it round in an arc, which they ride; one that rolls takes
    /// the floor out from under them, which they do not.
    pub fn tail_roll(mut self, roll: f32) -> Pose {
        self.bone[sim::beast::TAIL1][2] += roll;
        self
    }

    /// Curl the tail up or down without swinging it. Positive is up.
    pub fn tail_lift(self, lift: f32) -> Pose {
        self.tail(lift, 0.0)
    }

    /// One leg, by index into [`LEG_BONES`].
    pub fn leg(mut self, which: usize, swing: f32, knee: f32) -> Pose {
        let (hip, shin) = LEG_BONES[which];
        self.bone[hip][0] = swing;
        self.bone[shin][0] = knee;
        self
    }

    /// Spread a leg out from under the body.
    pub fn leg_spread(mut self, which: usize, spread: f32) -> Pose {
        let (hip, _) = LEG_BONES[which];
        // Positive is away from the midline on both sides, the same convention
        // the fighters' skeleton uses. `SIDE` carries the mirror.
        self.bone[hip][2] = spread * sim::beast::SIDE[hip] as f32;
        self
    }

    pub fn forelegs(self, swing: f32, knee: f32) -> Pose {
        self.leg(0, swing, knee).leg(1, swing, knee)
    }

    pub fn hindlegs(self, swing: f32, knee: f32) -> Pose {
        self.leg(2, swing, knee).leg(3, swing, knee)
    }

    /// The stance an animal stands in. Not the zero pose: with every channel at
    /// zero the legs are straight poles, and nothing with a knee stands like
    /// that.
    ///
    /// The feet are **solved onto the floor** rather than angled toward it, so
    /// that every clip built on this one starts from four feet actually on the
    /// ground. The forefeet sit a little ahead of the shoulder and the hind
    /// feet a little behind the hip, which is where a standing quadruped puts
    /// them and is what gives the legs somewhere to bend.
    pub fn standing() -> Pose {
        Pose::rest()
            .tail_lift(4.0)
            .plant_fore(0.16, 0.0)
            .plant_hind(-0.24, 0.0)
    }

    /// The same pose on the other side.
    pub fn mirrored(&self) -> Pose {
        let mut out = *self;
        out.hips[2] = -self.hips[2];
        for b in 0..BONES {
            // Yaw and roll flip; pitch is the same on both sides, which is the
            // whole reason the angles are named rather than raw Euler.
            out.bone[b][1] = -self.bone[b][1];
            out.bone[b][2] = -self.bone[b][2];
        }
        for pair in [(0usize, 1usize), (2, 3)] {
            let (a, b) = pair;
            for (x, y) in [LEG_BONES[a], LEG_BONES[b]]
                .into_iter()
                .zip([LEG_BONES[b], LEG_BONES[a]])
            {
                out.bone[x.0] = self.bone[y.0];
                out.bone[x.1] = self.bone[y.1];
                out.bone[x.0][1] = -self.bone[y.0][1];
                out.bone[x.0][2] = -self.bone[y.0][2];
            }
        }
        out
    }

    pub fn blend(&self, other: &Pose, t: f32) -> Pose {
        let mut out = *self;
        for i in 0..3 {
            out.hips[i] = self.hips[i] + (other.hips[i] - self.hips[i]) * t;
        }
        for b in 0..BONES {
            for c in 0..3 {
                out.bone[b][c] = self.bone[b][c] + (other.bone[b][c] - self.bone[b][c]) * t;
            }
        }
        out
    }

    /// The same pose in the simulation's units, so the rig can be built from it.
    pub fn to_sim(&self) -> sim::beast::Pose {
        let turns = |d: f32| sim::Fx::from_raw((d / 360.0 * 65536.0).round() as i32);
        let metres = |v: f32| sim::Fx::from_raw((v * 65536.0).round() as i32);
        let mut out = sim::beast::Pose::rest();
        out.hips = sim::V3::new(
            metres(self.hips[0]),
            metres(self.hips[1]),
            metres(self.hips[2]),
        );
        for b in 0..BONES {
            out.bone[b] = sim::V3::new(
                turns(self.bone[b][0]),
                turns(self.bone[b][1]),
                turns(self.bone[b][2]),
            );
        }
        out
    }

    /// **Put a foot on the floor**, and solve the leg for it.
    ///
    /// `reach` is how far forward of the hip the foot lands, and `clear` is how
    /// far above the floor. Both in metres.
    ///
    /// This is the creature's answer to the fighters' `plant_l/r`, and it
    /// exists for the same reason: the things that are *arithmetic* should not
    /// be keyed by hand. Every version of these clips authored as hip and knee
    /// angles had feet through the floor somewhere -- a foreleg half a metre
    /// under on the frame a bite lands, an animal levitating at the top of a
    /// rear -- because where a foot ends up is two angles, a body pitch and a
    /// hip offset multiplied together, and nobody can hold that in their head
    /// across twelve keys.
    ///
    /// Two-bone inverse kinematics in the sagittal plane, solved against the
    /// hip's **actual** world position -- built through the simulation's own
    /// forward kinematics, so it accounts for whatever the spine and hips are
    /// doing in this pose. Which means the order matters: set the body first,
    /// then plant.
    pub fn plant(mut self, which: usize, reach: f32, clear: f32) -> Pose {
        let (hip_bone, knee_bone) = LEG_BONES[which];
        let rig = sim::beast::Rig::build(sim::V3::ZERO, sim::Fx::ZERO, &self.to_sim());
        let f = |v: sim::Fx| v.to_f32_for_render();
        let hip = rig.bone[hip_bone].at;

        // Segment lengths: hip to knee is the bone offset, knee to sole is how
        // far the foot's box hangs below its own bone.
        let knee_offset = sim::beast::rest(knee_bone);
        let l1 = (f(knee_offset.x).powi(2) + f(knee_offset.y).powi(2)).sqrt();
        let foot = sim::monster::shape(LEG_PARTS[which]);
        let l2 = -f(foot.min.y);

        let dx = reach;
        let dy = clear - f(hip.y);
        let d = (dx * dx + dy * dy)
            .sqrt()
            .clamp((l1 - l2).abs() + 0.02, l1 + l2 - 0.02);
        // Angles measured from straight down, positive forward -- the rig's own
        // convention, so no sign has to be remembered here.
        let to_target = dx.atan2(-dy);
        let spread = (((d * d + l1 * l1 - l2 * l2) / (2.0 * d * l1)).clamp(-1.0, 1.0)).acos();
        // Which way the joint folds. A foreleg's carpus and a hindleg's hock
        // both break backward, so both put the knee behind the line from the
        // hip to the foot.
        let thigh = to_target - spread;
        let knee_x = f(hip.x) + l1 * thigh.sin();
        let knee_y = f(hip.y) - l1 * thigh.cos();
        let shin = (f(hip.x) + dx - knee_x).atan2(knee_y - clear);

        // Back into each bone's *local* pitch: subtract what it is already
        // carrying from everything above it.
        let parent = rig.bone[sim::beast::PARENTS[hip_bone]].rot;
        let up = parent.r[1];
        let carried = (-f(up.x)).atan2(f(up.y));
        let deg = |r: f32| r.to_degrees();
        self.bone[hip_bone][0] = deg(thigh - carried);
        self.bone[knee_bone][0] = deg(shin - thigh);
        self
    }

    /// Plant both front feet, or both back ones.
    pub fn plant_fore(self, reach: f32, clear: f32) -> Pose {
        self.plant(0, reach, clear).plant(1, reach, clear)
    }

    pub fn plant_hind(self, reach: f32, clear: f32) -> Pose {
        self.plant(2, reach, clear).plant(3, reach, clear)
    }

    /// Flat, in the order the baked table stores: hips, then three per bone.
    pub fn channels(&self) -> [f32; CHANNELS] {
        let mut out = [0.0; CHANNELS];
        out[..3].copy_from_slice(&self.hips);
        for b in 0..BONES {
            out[3 + b * 3..6 + b * 3].copy_from_slice(&self.bone[b]);
        }
        out
    }

    pub fn from_channels(c: &[f32; CHANNELS]) -> Pose {
        let mut p = Pose::rest();
        p.hips.copy_from_slice(&c[..3]);
        for b in 0..BONES {
            p.bone[b].copy_from_slice(&c[3 + b * 3..6 + b * 3]);
        }
        p
    }
}

// ---------------------------------------------------------------------------
// Looseness
// ---------------------------------------------------------------------------

/// Which part of the animal a bone belongs to, for the purposes of how loose it
/// is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Group {
    Body,
    Neck,
    Tail,
    Legs,
}

/// The group and the depth of each bone: depth is how far down its own chain it
/// sits, and it is what makes the tip of the tail trail the base of it without
/// anyone keying that.
pub fn group_of(bone: usize) -> (Group, u8) {
    use sim::beast::*;
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

/// One part's response, in units an author can picture: how many frames it runs
/// behind the keys, and how far it carries past on arrival.
///
/// The same pair the fighters' factory uses, and it is split the same way for
/// the same reason: **weight must read as follow-through, never as delay.** A
/// creature whose silhouette has not moved by frame ten of a fifty-two frame
/// slam has nothing on screen for the player to read while they are deciding
/// whether to leave.
#[derive(Clone, Copy, Debug)]
pub struct Feel {
    pub lag: f32,
    pub ring: f32,
}

impl Feel {
    pub const fn new(lag: f32, ring: f32) -> Feel {
        Feel { lag, ring }
    }

    fn spring(&self) -> (f32, f32) {
        let ring = self.ring.clamp(0.15, 1.2);
        let lag_seconds = (self.lag / 60.0).max(1.0 / 600.0);
        (2.0 * ring / lag_seconds, ring)
    }

    fn at_depth(&self, depth: u8, group: Group) -> Feel {
        let d = depth as f32;
        // The tail is the loosest thing on the animal and the legs are the
        // least loose: a trailing tail tip is the follow-through, and a
        // trailing *foot* is a foot in the floor.
        let taper = match group {
            Group::Tail => 0.55,
            Group::Legs => 0.15,
            _ => 0.35,
        };
        Feel {
            lag: self.lag * (1.0 + taper * d),
            ring: (self.ring * (1.0 - 0.12 * d)).max(0.2),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Looseness {
    pub body: Feel,
    pub neck: Feel,
    pub tail: Feel,
    pub legs: Feel,
}

impl Looseness {
    /// How a large animal carries itself: the body is slow to start and slow to
    /// stop, the neck lags it, the tail lags everything.
    pub const BEAST: Looseness = Looseness {
        body: Feel::new(2.4, 0.85),
        neck: Feel::new(3.2, 0.60),
        tail: Feel::new(4.2, 0.42),
        legs: Feel::new(2.0, 0.85),
    };

    /// Committing to something heavy. More ring everywhere: the animal arrives
    /// and keeps arriving.
    pub const HEAVY: Looseness = Looseness {
        body: Feel::new(3.0, 0.70),
        neck: Feel::new(4.0, 0.45),
        tail: Feel::new(5.4, 0.32),
        legs: Feel::new(2.6, 0.72),
    };

    /// Snapping. The bite and the shake: the body leads hard and only the far
    /// end is allowed to ring.
    pub const SNAP: Looseness = Looseness {
        body: Feel::new(1.5, 0.95),
        neck: Feel::new(1.9, 0.70),
        tail: Feel::new(3.4, 0.45),
        legs: Feel::new(1.6, 0.92),
    };

    /// The shake, and only the shake.
    ///
    /// Its own preset because the *root's* lag is a gameplay number here rather
    /// than a look: on a spring chain the root reaches its target first and
    /// therefore accelerates hardest, so a hips-heavy preset is what stops the
    /// one patch of animal directly over the pivot from being the most violent
    /// place to stand. Which it was, and which is backwards.
    pub const BUCK: Looseness = Looseness {
        body: Feel::new(2.8, 0.88),
        neck: Feel::new(2.2, 0.60),
        tail: Feel::new(3.6, 0.45),
        legs: Feel::new(1.7, 0.92),
    };

    /// A gait. Loose enough to breathe, tight enough to stay on the beat -- a
    /// cycle that lags is a cycle that skates.
    pub const GAIT: Looseness = Looseness {
        body: Feel::new(1.8, 0.92),
        neck: Feel::new(2.6, 0.65),
        tail: Feel::new(3.8, 0.45),
        legs: Feel::new(1.3, 0.95),
    };

    /// Nothing here is in control. Going down, and getting back up.
    pub const LIMP: Looseness = Looseness {
        body: Feel::new(3.6, 0.55),
        neck: Feel::new(5.0, 0.32),
        tail: Feel::new(6.0, 0.28),
        legs: Feel::new(4.0, 0.40),
    };

    pub fn group(&self, g: Group) -> Feel {
        match g {
            Group::Body => self.body,
            Group::Neck => self.neck,
            Group::Tail => self.tail,
            Group::Legs => self.legs,
        }
    }
}

// ---------------------------------------------------------------------------
// Recipes
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Key {
    /// Where in the clip, as a fraction of its length. A fraction rather than a
    /// frame because an attack's length is read live from the Oven and a key
    /// pinned to frame eleven would mean something different every time
    /// somebody drags a slider.
    pub at: f32,
    pub pose: Pose,
    /// Shapes the motion **out of** this key and into the next one. This is
    /// where most of the character of a move lives: the same two poses the same
    /// distance apart can be a slide, a hold-then-burst, or a pull-back before
    /// a commitment.
    pub ease: Ease,
}

impl Key {
    pub fn at(at: f32, pose: Pose) -> Key {
        Key {
            at,
            pose,
            ease: Ease::SMOOTH,
        }
    }

    pub fn eased(at: f32, pose: Pose, ease: Ease) -> Key {
        Key { at, pose, ease }
    }
}

#[derive(Clone, Debug)]
pub struct Recipe {
    pub clip: Clip,
    pub keys: Vec<Key>,
    pub looseness: Looseness,
    /// Why the clip is shaped the way it is. Data rather than a comment so a
    /// round trip through a tool cannot lose it, the same as the fighters'.
    pub notes: String,
}

impl Recipe {
    pub fn new(clip: Clip, keys: Vec<Key>, looseness: Looseness) -> Recipe {
        Recipe {
            clip,
            keys,
            looseness,
            notes: String::new(),
        }
    }

    pub fn noting(mut self, notes: impl Into<String>) -> Recipe {
        self.notes = notes.into();
        self
    }

    /// How long the clip is solved over, in frames.
    ///
    /// An attack is solved across its **real** length, read live from the move
    /// table, so the springs see the timing the player will. It is then
    /// resampled into phases, which is what keeps the table tunable.
    pub fn frames(&self) -> u16 {
        match move_of(self.clip) {
            Some(kind) => sim::monster::attack(kind).total().max(3),
            None => match self.clip {
                Clip::Topple => sim::tuning::topple_frames(),
                Clip::Stumble => sim::tuning::stumble_frames(),
                Clip::Flinch => sim::tuning::flinch_frames(),
                Clip::Dead => 1,
                // A cycle's own length in frames is arbitrary -- it is indexed
                // by ground covered, not by time -- so it is solved over a
                // sensible number and sampled round the wheel.
                _ => 48,
            }
            .max(3),
        }
    }

    /// The three phase boundaries of an attack, as fractions of the clip.
    fn phase_cuts(&self) -> Option<[f32; 2]> {
        let kind = move_of(self.clip)?;
        let m = sim::monster::attack(kind);
        let total = m.total().max(1) as f32;
        Some([
            m.startup as f32 / total,
            (m.startup + m.active) as f32 / total,
        ])
    }
}

/// Where a point inside one phase of an attack falls in the whole clip.
///
/// Keys are authored against the clip, and the clip's phases are whatever the
/// move table currently says -- so authoring a contact pose means saying "at
/// the start of the active window", not "at frame thirty-five". This is what
/// turns the first into the second, and it is why retuning a move in the Oven
/// moves its animation with it instead of desynchronising it.
///
/// `phase` is 0 startup, 1 active, 2 recovery; `u` runs 0 to 1 inside it.
pub fn mark(clip: Clip, phase: u8, u: f32) -> f32 {
    let Some(kind) = move_of(clip) else {
        return u;
    };
    let m = sim::monster::attack(kind);
    let total = m.total().max(1) as f32;
    let (from, span) = match phase {
        0 => (0.0, m.startup as f32),
        1 => (m.startup as f32, m.active as f32),
        _ => ((m.startup + m.active) as f32, m.recovery as f32),
    };
    (from + span * u.clamp(0.0, 1.0)) / total
}

fn move_of(clip: Clip) -> Option<u8> {
    use sim::monster::*;
    Some(match clip {
        Clip::Bite => BITE,
        Clip::Stomp => STOMP,
        Clip::Sweep => SWEEP,
        Clip::Charge => CHARGE,
        Clip::Slam => SLAM,
        Clip::Shake => SHAKE,
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// The solver
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Baked {
    pub clip: Clip,
    /// One pose per sample, in the layout the runtime reads: for an attack,
    /// three phases of [`PHASE_SAMPLES`] laid end to end; for anything else,
    /// one run.
    pub samples: Vec<Pose>,
}

fn channel_springs(looseness: &Looseness) -> [Spring; CHANNELS] {
    let mut springs = [Spring::new(0.0, 20.0, 1.0); CHANNELS];
    let (f, d) = looseness.body.spring();
    for s in springs.iter_mut().take(3) {
        *s = Spring::new(0.0, f, d);
    }
    for bone in 0..BONES {
        let (group, depth) = group_of(bone);
        let (f, d) = looseness.group(group).at_depth(depth, group).spring();
        for c in 0..3 {
            springs[3 + bone * 3 + c] = Spring::new(0.0, f, d);
        }
    }
    springs
}

/// Read the key track at a fractional position through the clip.
fn sample_keys(keys: &[Key], at: f32, looping: bool) -> Pose {
    if keys.is_empty() {
        return Pose::rest();
    }
    if keys.len() == 1 {
        return keys[0].pose;
    }
    let at = if looping { at.rem_euclid(1.0) } else { at };
    if !looping {
        if at <= keys[0].at {
            return keys[0].pose;
        }
        if at >= keys[keys.len() - 1].at {
            return keys[keys.len() - 1].pose;
        }
    }
    let between = |a: &Key, b: &Key, at: f32| {
        let span = (b.at - a.at).max(1e-4);
        let k = ((at - a.at) / span).clamp(0.0, 1.0);
        a.pose.blend(&b.pose, a.ease.at(k))
    };
    for pair in keys.windows(2) {
        if at >= pair[0].at && at <= pair[1].at {
            return between(&pair[0], &pair[1], at);
        }
    }
    if looping {
        let last = keys[keys.len() - 1];
        if at >= last.at {
            let mut wrapped = keys[0];
            wrapped.at = 1.0;
            return between(&last, &wrapped, at);
        }
        let mut tail = last;
        tail.at = 0.0;
        return between(&tail, &keys[0], at);
    }
    keys[keys.len() - 1].pose
}

/// Run the springs and produce the samples.
pub fn bake(recipe: &Recipe) -> Baked {
    assert!(!recipe.keys.is_empty(), "{}: no keys", recipe.clip.name());
    let frames = recipe.frames();
    let looping = recipe.clip.looping();
    let mut springs = channel_springs(&recipe.looseness);

    // Start settled on the first key, so a one-shot does not open with a lurch
    // out of an arbitrary rest pose.
    let first = sample_keys(&recipe.keys, 0.0, looping).channels();
    for (i, s) in springs.iter_mut().enumerate() {
        s.value = first[i];
        s.velocity = 0.0;
    }

    // A cycle has no beginning, so give the springs two turns of the wheel to
    // reach a steady state before anything is recorded. Without it the first
    // stride of every walk is subtly different from the rest, and the seam is
    // visible exactly where the loop closes.
    let preroll = if looping { frames * 2 } else { 0 };
    let dt = DT / SUBSTEPS as f32;
    let mut solved: Vec<Pose> = Vec::with_capacity(frames as usize);
    for frame in 0..(preroll + frames) {
        for sub in 0..SUBSTEPS {
            let t = (frame as f32 + sub as f32 / SUBSTEPS as f32) / frames as f32;
            let target = sample_keys(&recipe.keys, t, looping).channels();
            for (i, s) in springs.iter_mut().enumerate() {
                s.step(target[i], dt);
            }
        }
        if frame >= preroll {
            let mut c = [0.0; CHANNELS];
            for (i, s) in springs.iter().enumerate() {
                c[i] = s.value;
            }
            solved.push(Pose::from_channels(&c));
        }
    }

    // Resample. An attack goes into three runs of equal length -- one per phase
    // -- read off the same continuous solve, so the phases join without a seam
    // and each can then be stretched independently at runtime.
    let read = |u: f32| -> Pose {
        let last = solved.len().saturating_sub(1);
        if looping {
            let scaled = u.rem_euclid(1.0) * solved.len() as f32;
            let i = scaled.floor() as usize % solved.len();
            let j = (i + 1) % solved.len();
            return solved[i].blend(&solved[j], scaled - scaled.floor());
        }
        let scaled = u.clamp(0.0, 1.0) * last as f32;
        let i = (scaled.floor() as usize).min(last);
        let j = (i + 1).min(last);
        solved[i].blend(&solved[j], scaled - scaled.floor())
    };

    let samples = match recipe.phase_cuts() {
        Some([a, b]) => {
            let mut out = Vec::with_capacity(PHASE_SAMPLES * 3);
            for (from, to) in [(0.0, a), (a, b), (b, 1.0)] {
                for i in 0..PHASE_SAMPLES {
                    let u = i as f32 / (PHASE_SAMPLES - 1) as f32;
                    out.push(read(from + (to - from) * u));
                }
            }
            out
        }
        None if recipe.clip == Clip::Dead => vec![read(1.0)],
        None => {
            let count = CYCLE_SAMPLES;
            let steps = if looping { count } else { count - 1 };
            (0..count).map(|i| read(i as f32 / steps as f32)).collect()
        }
    };

    Baked {
        clip: recipe.clip,
        samples,
    }
}

/// Bake every clip, filling in a held standing pose for anything unauthored.
pub fn bake_all() -> (Vec<Baked>, Vec<Clip>) {
    let recipes = clips::all();
    let mut out = Vec::with_capacity(CLIPS);
    let mut missing = Vec::new();
    for clip in Clip::ALL {
        match recipes.iter().find(|r| r.clip == clip) {
            Some(r) => out.push(bake(r)),
            None => {
                missing.push(clip);
                let count = if clip.phased() {
                    PHASE_SAMPLES * 3
                } else if clip == Clip::Dead {
                    1
                } else {
                    CYCLE_SAMPLES
                };
                out.push(Baked {
                    clip,
                    samples: vec![Pose::standing(); count],
                });
            }
        }
    }
    (out, missing)
}

// ---------------------------------------------------------------------------
// Emitting the table
// ---------------------------------------------------------------------------

/// Degrees to raw 16.16 turns.
fn raw(degrees: f32) -> i32 {
    (degrees / 360.0 * 65536.0).round() as i32
}

/// Metres to raw 16.16.
fn raw_m(metres: f32) -> i32 {
    (metres * 65536.0).round() as i32
}

/// Emit the baked table as Rust source, for checking in.
///
/// Generated code rather than a data file for the reasons the fighters' table
/// is: no loader, no asset path, no runtime parsing, and a diff shows exactly
/// what changed when an animation is retuned. One line per sample, because a
/// pose spread over twenty lines turns a two-frame change into forty lines of
/// noise.
pub fn emit(baked: &[Baked]) -> String {
    let mut out = String::new();
    out.push_str(
        "//! Baked creature poses. GENERATED -- do not edit by hand.\n\
         //!\n\
         //! Written by `cargo run -p anim --bin bake_beast`. The recipes live in\n\
         //! `crates/anim/src/beast/clips.rs`; edit those.\n\
         //!\n\
         //! Each row is one sample: three numbers for the hips in metres, then\n\
         //! pitch, yaw and roll for each of the eighteen bones, all as raw 16.16\n\
         //! bits. A clip is a span of rows, and an attack's rows are three equal\n\
         //! runs -- startup, active, recovery -- read by phase. Retuning a move's\n\
         //! frame counts in the Oven therefore stretches its animation with it\n\
         //! rather than leaving the contact pose on the wrong frame.\n\n\
         use crate::beast::{CHANNELS, CLIPS};\n\n",
    );

    let rows: usize = baked.iter().map(|b| b.samples.len()).sum();
    out.push_str(&format!("pub const ROWS: usize = {rows};\n\n"));
    out.push_str(
        "/// `(first row, how many)` per clip, in `beast::Clip::ALL` order.\n\
         pub const SPAN: [(u16, u16); CLIPS] = [\n",
    );
    let mut start = 0usize;
    for b in baked {
        out.push_str(&format!(
            "    ({start}, {}), // {}\n",
            b.samples.len(),
            b.clip.name()
        ));
        start += b.samples.len();
    }
    out.push_str("];\n\n#[rustfmt::skip]\npub static FRAMES: [[i32; CHANNELS]; ROWS] = [\n");
    for b in baked {
        for (i, pose) in b.samples.iter().enumerate() {
            let mut cells: Vec<String> = Vec::with_capacity(CHANNELS);
            for (c, v) in pose.channels().iter().enumerate() {
                cells.push(if c < 3 { raw_m(*v) } else { raw(*v) }.to_string());
            }
            out.push_str(&format!(
                "    [{}], // {} {i}\n",
                cells.join(","),
                b.clip.name()
            ));
        }
    }
    out.push_str("];\n");
    out
}
