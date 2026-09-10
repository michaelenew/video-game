//! Sparse intent in, dense natural motion out.
//!
//! An author writes a handful of poses and says how loose each body part should
//! be. The solver drives every channel toward the piecewise-linear reading of
//! those keys, through a spring whose looseness depends on the part — so the
//! torso arrives promptly and the hands trail and settle without anyone keying
//! that by hand.
//!
//! The output is a flat table indexed by frame. The game does a lookup, so
//! playback stays `pose = f(action, frame)` and rollback is unaffected.

use crate::spring::Spring;
use crate::{DT, SUBSTEPS};
use view::pose::{PART_COUNT, PARTS, Part, PartTransform, Pose};

/// A pose the author cares about, at a frame they care about.
#[derive(Clone, Copy, Debug)]
pub struct Key {
    pub frame: u16,
    pub pose: Pose,
}

/// How loose each part is. This is the whole authoring surface for feel.
///
/// Each part is described by two numbers a person can reason about:
///
/// - **lag**, in frames: how far behind the keys this part runs. A spring
///   chasing a moving target settles into a steady lag of roughly `2z/w`, so
///   this is the honest unit for the knob -- "the hand trails the shoulder by
///   two frames" is a sentence an animator can hold in their head, and
///   "frequency 15" is not.
/// - **ring**: how much it overshoots and wobbles on arrival. `1.0` is
///   critically damped and never overshoots; lower values ring more.
///
/// The distinction matters more than it looks. The first pass at this file set
/// heavy parts to a low frequency, which made "heavy" mean *late* -- the
/// overhead's silhouette had barely moved a third of the way through its own
/// startup, so there was nothing for the opponent to read and react to. Weight
/// should read as follow-through and settle, not as delay. Splitting lag from
/// ring makes that mistake hard to make again: you can now ask for a part that
/// arrives promptly and still wobbles.
#[derive(Clone, Copy, Debug)]
pub struct Looseness {
    pub torso: Feel,
    pub head: Feel,
    pub arms: Feel,
    pub legs: Feel,
}

/// One part's response, in units an author can picture.
#[derive(Clone, Copy, Debug)]
pub struct Feel {
    /// Frames this part runs behind the keys.
    pub lag: f32,
    /// 1.0 never overshoots; lower rings more.
    pub ring: f32,
}

impl Feel {
    pub const fn new(lag: f32, ring: f32) -> Self {
        Feel { lag, ring }
    }

    /// Convert to the spring's own units.
    ///
    /// A damped spring tracking a ramp settles at a constant offset behind it
    /// of `2z/w`. Solving that for the frequency is the whole conversion.
    fn spring(&self) -> (f32, f32) {
        let lag_seconds = (self.lag / 60.0).max(1.0 / 600.0);
        (2.0 * self.ring / lag_seconds, self.ring)
    }
}

impl Looseness {
    /// A martial default: torso leads and is firmly controlled, head lags
    /// slightly, arms trail and ring, legs stay planted.
    pub const MARTIAL: Looseness = Looseness {
        torso: Feel::new(1.2, 1.0),
        head: Feel::new(1.8, 0.8),
        arms: Feel::new(2.4, 0.6),
        legs: Feel::new(1.5, 0.9),
    };

    /// Heavier and more organic. Parts still arrive on time -- they just carry
    /// further past the mark and take longer to stop wobbling.
    pub const HEAVY: Looseness = Looseness {
        torso: Feel::new(2.4, 0.9),
        head: Feel::new(3.2, 0.6),
        arms: Feel::new(4.0, 0.45),
        legs: Feel::new(2.8, 0.75),
    };

    fn for_part(&self, part: Part) -> (f32, f32) {
        match part {
            Part::Torso => self.torso,
            Part::Head => self.head,
            Part::ArmL | Part::ArmR => self.arms,
            Part::LegL | Part::LegR => self.legs,
        }
        .spring()
    }
}

/// One animation to bake.
#[derive(Clone, Debug)]
pub struct Recipe {
    pub name: &'static str,
    /// Frames to produce. Should match the action's frame count in `moves.rs`.
    pub length: u16,
    pub keys: Vec<Key>,
    pub looseness: Looseness,
}

#[derive(Clone, Debug)]
pub struct Baked {
    pub name: &'static str,
    /// `length` poses, one per frame.
    pub frames: Vec<Pose>,
}

/// Run the solver and produce the table.
pub fn bake(recipe: &Recipe) -> Baked {
    assert!(!recipe.keys.is_empty(), "{}: no keys", recipe.name);

    // Six channels per part: three position, three rotation.
    let mut springs: Vec<[Spring; 6]> = PARTS
        .iter()
        .map(|p| {
            let (f, d) = recipe.looseness.for_part(*p);
            [Spring::new(0.0, f, d); 6]
        })
        .collect();

    // Start settled on the first key, so the animation does not open with a
    // lurch from an arbitrary rest pose.
    let first = recipe.keys[0].pose;
    for (i, part) in PARTS.iter().enumerate() {
        let t = first.get(*part);
        for c in 0..3 {
            springs[i][c].value = t.pos[c];
            springs[i][3 + c].value = t.rot[c];
        }
    }

    let mut frames = Vec::with_capacity(recipe.length as usize);
    for frame in 0..recipe.length {
        // Substep, because springs stiff enough to look snappy are stiff enough
        // to go unstable at 60 Hz.
        for sub in 0..SUBSTEPS {
            let t = frame as f32 + sub as f32 / SUBSTEPS as f32;
            let target = sample(&recipe.keys, t);
            for (i, part) in PARTS.iter().enumerate() {
                let want = target.get(*part);
                for c in 0..3 {
                    springs[i][c].step(want.pos[c], DT / SUBSTEPS as f32);
                    springs[i][3 + c].step(want.rot[c], DT / SUBSTEPS as f32);
                }
            }
        }

        let mut pose = first;
        for (i, _) in PARTS.iter().enumerate() {
            pose.parts[i] = PartTransform {
                pos: [
                    springs[i][0].value,
                    springs[i][1].value,
                    springs[i][2].value,
                ],
                rot: [
                    springs[i][3].value,
                    springs[i][4].value,
                    springs[i][5].value,
                ],
            };
        }
        frames.push(pose);
    }

    Baked {
        name: recipe.name,
        frames,
    }
}

/// Piecewise-linear read of the keys at a fractional frame. Deliberately not
/// smoothed: the springs supply all the smoothing, and pre-smoothing the target
/// only muddies the timing the author asked for.
fn sample(keys: &[Key], t: f32) -> Pose {
    if t <= keys[0].frame as f32 {
        return keys[0].pose;
    }
    for pair in keys.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if t <= b.frame as f32 {
            let span = (b.frame - a.frame).max(1) as f32;
            let k = ((t - a.frame as f32) / span).clamp(0.0, 1.0);
            return lerp_pose(&a.pose, &b.pose, k);
        }
    }
    keys[keys.len() - 1].pose
}

fn lerp_pose(a: &Pose, b: &Pose, k: f32) -> Pose {
    let mut out = *a;
    for i in 0..PART_COUNT {
        for c in 0..3 {
            out.parts[i].pos[c] = a.parts[i].pos[c] + (b.parts[i].pos[c] - a.parts[i].pos[c]) * k;
            out.parts[i].rot[c] = a.parts[i].rot[c] + (b.parts[i].rot[c] - a.parts[i].rot[c]) * k;
        }
    }
    out
}

/// Emit a baked table as Rust source, for checking in.
///
/// Generated code rather than a data file on purpose: it needs no loader, no
/// asset path, and no runtime parsing, and a diff shows exactly what changed
/// when an animation is retuned.
pub fn emit(baked: &[Baked]) -> String {
    let mut out = String::new();
    out.push_str(
        "//! Baked animation tables. GENERATED — do not edit.\n\
         //!\n\
         //! Produced by `cargo run -p anim --bin bake`. The recipes live in\n\
         //! `crates/anim/src/bin/bake.rs`; edit those and re-run.\n\
         //!\n\
         //! Playback is a lookup by frame, so this stays a pure function of\n\
         //! simulation state and rollback is unaffected.\n\n\
         // Baked numbers land where the solver puts them; a pose value that\n\
         // happens to sit near ln(2) or 1/pi is a coincidence, not a constant.\n\
         #![allow(clippy::approx_constant)]\n\n\
         use crate::pose::{PartTransform, Pose};\n\n",
    );

    for b in baked {
        out.push_str(&format!(
            "/// {} frames.\npub static {}: [Pose; {}] = [\n",
            b.frames.len(),
            b.name.to_uppercase(),
            b.frames.len()
        ));
        for pose in &b.frames {
            out.push_str("    Pose { parts: [\n");
            for p in &pose.parts {
                out.push_str(&format!(
                    "        PartTransform {{ pos: [{:.4}, {:.4}, {:.4}], rot: [{:.4}, {:.4}, {:.4}] }},\n",
                    p.pos[0], p.pos[1], p.pos[2], p.rot[0], p.rot[1], p.rot[2]
                ));
            }
            out.push_str("    ] },\n");
        }
        out.push_str("];\n\n");
    }
    out
}
