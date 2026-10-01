//! The creatures' animation, authored.
//!
//! Same factory as the fighters go through -- sparse keys, an ease per gap, a
//! spring per channel, bake the result -- pointed at a different skeleton: a
//! species' own, from `sim::species`, with three angles per bone and a hip
//! offset.
//!
//! **One folder per species.** This file is the factory; what a creature looks
//! like is its own module -- `ridgeback/` for the Ridgeback -- which implements
//! [`Authored`]: its recipes, how loose each of its bones is, the pose it
//! stands in, and where its baked table goes (`crates/sim/src/species/<name>/
//! baked.rs`). Two species never bake into the same file.
//!
//! ```text
//! keys + eases + looseness  --[springs, offline]-->  one pose per sample
//! ```
//!
//! Two things are different from the fighters' pipeline, and both come out of
//! the creature being simulation geometry rather than decoration.
//!
//! **The table is in phase, not in frames.** An attack bakes as three runs of
//! thirty-two samples -- startup, active, recovery -- each read on its own at
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

pub mod sheet;

// One line per species with an animation module, each followed by a blank
// line: see `authored` below.

pub mod ridgeback;

// pub mod gnawers;

// pub mod hornback;

// pub mod mireback;

// pub mod sandmaw;

// pub mod pair;

// pub mod broodmother;

// pub mod veilstalker;

// pub mod mantis;

// pub mod galewing;

// pub mod siegeshell;

use crate::ease::Ease;
use crate::spring::Spring;
use crate::{DT, SUBSTEPS};
use sim::beast::{CYCLE_SAMPLES, MAX_BONES, PHASE_SAMPLES, channels};
use sim::species::{Species, SpeciesId};

// ---------------------------------------------------------------------------
// A species, as authored
// ---------------------------------------------------------------------------

/// What a species' animation module supplies to the factory.
pub trait Authored: Sync {
    /// The species it animates.
    fn species(&self) -> &'static Species;
    /// Every clip it has a recipe for.
    fn recipes(&self) -> Vec<Recipe>;
    /// Which group a bone belongs to, and how far down its own chain it sits:
    /// depth is what makes the tip of a tail trail the base of it without
    /// anyone keying that.
    fn group_of(&self, bone: usize) -> (Group, u8);
    /// The pose it stands in, which is also what an unauthored clip holds.
    fn standing(&self) -> Pose;
    /// Where its baked table goes, from the repository root.
    fn baked_path(&self) -> &'static str;
}

/// The animation module registered for a species, if it has one.
///
/// Every planned creature already has a line here, commented out and one blank
/// line from the next, for the reason `sim::species` gives.
pub fn authored(id: SpeciesId) -> Option<&'static dyn Authored> {
    match id {
        SpeciesId::RIDGEBACK => Some(&ridgeback::RIDGEBACK),

        // SpeciesId::GNAWERS => Some(&gnawers::GNAWERS),

        // SpeciesId::HORNBACK => Some(&hornback::HORNBACK),

        // SpeciesId::MIREBACK => Some(&mireback::MIREBACK),

        // SpeciesId::SANDMAW => Some(&sandmaw::SANDMAW),

        // SpeciesId::PAIR => Some(&pair::PAIR),

        // SpeciesId::BROODMOTHER => Some(&broodmother::BROODMOTHER),

        // SpeciesId::VEILSTALKER => Some(&veilstalker::VEILSTALKER),

        // SpeciesId::MANTIS => Some(&mantis::MANTIS),

        // SpeciesId::GALEWING => Some(&galewing::GALEWING),

        // SpeciesId::SIEGESHELL => Some(&siegeshell::SIEGESHELL),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Poses, in degrees
// ---------------------------------------------------------------------------

/// One pose of a creature. Angles in **degrees**, because that is how a person
/// describes a body; the emitter turns them into turns and then into fixed
/// point.
///
/// Sign conventions match the rig: `pitch` is nose-up, `yaw` is toward the
/// creature's own right, `roll` is about the bone's own length. The mirror
/// lives in the skeleton, so a symmetric pose is a symmetric set of numbers.
///
/// The builders that know a particular animal's anatomy -- the Ridgeback's
/// neck, tail and legs -- live in that species' module, as a trait on this.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub hips: [f32; 3],
    pub bone: [[f32; 3]; MAX_BONES],
    /// How many of `bone` the species has.
    pub bones: usize,
}

impl Pose {
    pub const fn rest(bones: usize) -> Pose {
        Pose {
            hips: [0.0; 3],
            bone: [[0.0; 3]; MAX_BONES],
            bones,
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

    pub fn blend(&self, other: &Pose, t: f32) -> Pose {
        let mut out = *self;
        for i in 0..3 {
            out.hips[i] = self.hips[i] + (other.hips[i] - self.hips[i]) * t;
        }
        for b in 0..self.bones {
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
        let mut out = sim::beast::Pose::rest(self.bones);
        out.hips = sim::V3::new(
            metres(self.hips[0]),
            metres(self.hips[1]),
            metres(self.hips[2]),
        );
        for b in 0..self.bones {
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
    /// `which` is the leg, by its index in the species' `legs`. `reach` is how
    /// far forward of the hip the foot lands, and `clear` is how far above the
    /// floor. Both in metres.
    ///
    /// This is the creature's answer to the fighters' `plant_l/r`, and it
    /// exists for the same reason: the things that are *arithmetic* should not
    /// be keyed by hand. Every version of the Ridgeback's clips authored as hip
    /// and knee angles had feet through the floor somewhere -- a foreleg half a
    /// metre under on the frame a bite lands, an animal levitating at the top
    /// of a rear -- because where a foot ends up is two angles, a body pitch
    /// and a hip offset multiplied together, and nobody can hold that in their
    /// head across twelve keys.
    ///
    /// Two-bone inverse kinematics in the sagittal plane, solved against the
    /// hip's **actual** world position -- built through the simulation's own
    /// forward kinematics, so it accounts for whatever the spine and hips are
    /// doing in this pose. Which means the order matters: set the body first,
    /// then plant.
    pub fn plant_leg(
        mut self,
        species: &'static Species,
        which: usize,
        reach: f32,
        clear: f32,
    ) -> Pose {
        let leg = species.legs[which];
        let (hip_bone, knee_bone) = (leg.hip, leg.knee);
        let rig = sim::beast::Rig::build(species, sim::V3::ZERO, sim::Fx::ZERO, &self.to_sim());
        let f = |v: sim::Fx| v.to_f32_for_render();
        let hip = rig.bone[hip_bone].at;

        // Segment lengths: hip to knee is the bone offset, knee to sole is how
        // far the foot's box hangs below its own bone.
        let knee_offset = species.rest(knee_bone);
        let l1 = (f(knee_offset.x).powi(2) + f(knee_offset.y).powi(2)).sqrt();
        let foot = species.shape(leg.foot);
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
        let parent = rig.bone[species.bones[hip_bone].parent].rot;
        let up = parent.r[1];
        let carried = (-f(up.x)).atan2(f(up.y));
        let deg = |r: f32| r.to_degrees();
        self.bone[hip_bone][0] = deg(thigh - carried);
        self.bone[knee_bone][0] = deg(shin - thigh);
        self
    }

    /// Flat, in the order the baked table stores: hips, then three per bone.
    pub fn channels(&self) -> Vec<f32> {
        let mut out = vec![0.0; channels(self.bones)];
        out[..3].copy_from_slice(&self.hips);
        for b in 0..self.bones {
            out[3 + b * 3..6 + b * 3].copy_from_slice(&self.bone[b]);
        }
        out
    }

    /// The inverse of [`Pose::channels`]: the row's length says how many
    /// bones it has.
    pub fn from_channels(c: &[f32]) -> Pose {
        let mut p = Pose::rest((c.len().saturating_sub(3) / 3).min(MAX_BONES));
        p.hips.copy_from_slice(&c[..3]);
        for b in 0..p.bones {
            p.bone[b].copy_from_slice(&c[3 + b * 3..6 + b * 3]);
        }
        p
    }
}

// ---------------------------------------------------------------------------
// Looseness
// ---------------------------------------------------------------------------

/// Which part of the animal a bone belongs to, for the purposes of how loose it
/// is. A species says which group each of its bones is in (see
/// [`Authored::group_of`]).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Group {
    Body,
    Neck,
    Tail,
    Legs,
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

    /// Going down and getting back up, with the legs kept honest.
    ///
    /// The body, neck and tail are as loose as `LIMP`. The legs are not: on a
    /// collapse the hips fall further than a leg is long, and a leg that lags
    /// and rings behind a fall of that size is a leg through the floor for
    /// a dozen frames. `Feel`'s own note says it -- a trailing foot is a foot
    /// in the ground -- and a metre and a half of drop is where it bites.
    pub const COLLAPSE: Looseness = Looseness {
        body: Feel::new(3.6, 0.55),
        neck: Feel::new(5.0, 0.32),
        tail: Feel::new(6.0, 0.28),
        legs: Feel::new(2.0, 0.80),
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
    /// Which of the species' clips, by index.
    pub clip: usize,
    pub keys: Vec<Key>,
    pub looseness: Looseness,
    /// Why the clip is shaped the way it is. Data rather than a comment so a
    /// round trip through a tool cannot lose it, the same as the fighters'.
    pub notes: String,
}

impl Recipe {
    pub fn new(clip: impl Into<usize>, keys: Vec<Key>, looseness: Looseness) -> Recipe {
        Recipe {
            clip: clip.into(),
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
    pub fn frames(&self, species: &Species) -> u16 {
        let stock = species.stock;
        match move_of(species, self.clip) {
            Some(kind) => species.attack(kind).total().max(3),
            None => if self.clip == stock.topple {
                species.topple_frames()
            } else if self.clip == stock.stumble {
                species.stumble_frames()
            } else if self.clip == stock.flinch {
                species.flinch_frames()
            } else if self.clip == stock.dead {
                1
            } else {
                // A cycle's own length in frames is arbitrary -- it is indexed
                // by ground covered, not by time -- so it is solved over a
                // sensible number and sampled round the wheel.
                48
            }
            .max(3),
        }
    }

    /// The three phase boundaries of an attack, as fractions of the clip.
    fn phase_cuts(&self, species: &Species) -> Option<[f32; 2]> {
        let kind = move_of(species, self.clip)?;
        let m = species.attack(kind);
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
pub fn mark(species: &Species, clip: usize, phase: u8, u: f32) -> f32 {
    let Some(kind) = move_of(species, clip) else {
        return u;
    };
    let m = species.attack(kind);
    let total = m.total().max(1) as f32;
    let (from, span) = match phase {
        0 => (0.0, m.startup as f32),
        1 => (m.startup as f32, m.active as f32),
        _ => ((m.startup + m.active) as f32, m.recovery as f32),
    };
    (from + span * u.clamp(0.0, 1.0)) / total
}

/// The move that plays a clip, if one does: read off the species' move table,
/// so a clip is an attack exactly when a move says it is.
fn move_of(species: &Species, clip: usize) -> Option<u8> {
    species
        .moves
        .iter()
        .position(|m| m.clip == clip)
        .map(|k| k as u8)
}

// ---------------------------------------------------------------------------
// The solver
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Baked {
    pub clip: usize,
    /// One pose per sample, in the layout the runtime reads: for an attack,
    /// three phases of [`PHASE_SAMPLES`] laid end to end; for anything else,
    /// one run.
    pub samples: Vec<Pose>,
}

fn channel_springs(beast: &dyn Authored, looseness: &Looseness) -> Vec<Spring> {
    let bones = beast.species().bones.len();
    let mut springs = vec![Spring::new(0.0, 20.0, 1.0); channels(bones)];
    let (f, d) = looseness.body.spring();
    for s in springs.iter_mut().take(3) {
        *s = Spring::new(0.0, f, d);
    }
    for bone in 0..bones {
        let (group, depth) = beast.group_of(bone);
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
        return Pose::rest(0);
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
pub fn bake(beast: &dyn Authored, recipe: &Recipe) -> Baked {
    let species = beast.species();
    let clip = species.clips[recipe.clip];
    assert!(!recipe.keys.is_empty(), "{}: no keys", clip.name);
    let frames = recipe.frames(species);
    let looping = clip.looping;
    let mut springs = channel_springs(beast, &recipe.looseness);

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
            let c: Vec<f32> = springs.iter().map(|s| s.value).collect();
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

    let samples = match recipe.phase_cuts(species) {
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
        None if recipe.clip == species.stock.dead => vec![read(1.0)],
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

/// Bake every clip of a species, filling in a held standing pose for anything
/// unauthored. The second list is the clips that had no recipe, by index.
pub fn bake_all(beast: &dyn Authored) -> (Vec<Baked>, Vec<usize>) {
    let species = beast.species();
    let recipes = beast.recipes();
    let mut out = Vec::with_capacity(species.clips.len());
    let mut missing = Vec::new();
    for (clip, decl) in species.clips.iter().enumerate() {
        match recipes.iter().find(|r| r.clip == clip) {
            Some(r) => out.push(bake(beast, r)),
            None => {
                missing.push(clip);
                let count = if decl.phased {
                    PHASE_SAMPLES * 3
                } else if clip == species.stock.dead {
                    1
                } else {
                    CYCLE_SAMPLES
                };
                out.push(Baked {
                    clip,
                    samples: vec![beast.standing(); count],
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
pub fn emit(beast: &dyn Authored, baked: &[Baked]) -> String {
    let species = beast.species();
    let slug = species.slug();
    let width = channels(species.bones.len());
    let mut out = String::new();
    out.push_str(&format!(
        "//! The {name}'s baked poses. GENERATED -- do not edit by hand.\n\
         //!\n\
         //! Written by `cargo run -p anim --bin bake_beast`. The recipes live in\n\
         //! `crates/anim/src/beast/{slug}/`; edit those.\n\
         //!\n\
         //! Each row is one sample: three numbers for the hips in metres, then\n\
         //! pitch, yaw and roll for each of the {bones} bones, all as raw 16.16\n\
         //! bits. A clip is a span of rows, and an attack's rows are three equal\n\
         //! runs -- startup, active, recovery -- read by phase. Retuning a move's\n\
         //! frame counts in the Oven therefore stretches its animation with it\n\
         //! rather than leaving the contact pose on the wrong frame.\n\n\
         use super::{{CLIP_COUNT, bones}};\n\n\
         pub const CHANNELS: usize = crate::beast::channels(bones::COUNT);\n\n",
        name = species.name,
        bones = species.bones.len(),
    ));

    let rows: usize = baked.iter().map(|b| b.samples.len()).sum();
    out.push_str(&format!("pub const ROWS: usize = {rows};\n\n"));
    out.push_str(
        "/// `(first row, how many)` per clip, in `Clip::ALL` order.\n\
         pub const SPAN: [(u16, u16); CLIP_COUNT] = [\n",
    );
    let mut start = 0usize;
    for b in baked {
        out.push_str(&format!(
            "    ({start}, {}), // {}\n",
            b.samples.len(),
            species.clips[b.clip].name
        ));
        start += b.samples.len();
    }
    out.push_str("];\n\n#[rustfmt::skip]\npub static FRAMES: [[i32; CHANNELS]; ROWS] = [\n");
    for b in baked {
        for (i, pose) in b.samples.iter().enumerate() {
            let mut cells: Vec<String> = Vec::with_capacity(width);
            for (c, v) in pose.channels().iter().enumerate() {
                cells.push(if c < 3 { raw_m(*v) } else { raw(*v) }.to_string());
            }
            out.push_str(&format!(
                "    [{}], // {} {i}\n",
                cells.join(","),
                species.clips[b.clip].name
            ));
        }
    }
    out.push_str("];\n");
    out
}
