//! Sparse intent in, dense natural motion out.
//!
//! An author writes a handful of poses, says how the time between each pair is
//! spent, and says how loose each part of the body is. The solver drives every
//! channel toward the eased reading of those keys through a spring whose
//! looseness depends on the part -- so the torso arrives promptly and the hands
//! trail and settle without anyone keying that by hand.
//!
//! ```text
//! keys + eases + looseness  --[springs, offline]-->  one pose per frame
//! ```
//!
//! The output is a flat table indexed by frame. The game does a lookup, so
//! playback stays `pose = f(action, frame)` and rollback is unaffected.
//!
//! Two properties are worth stating because they are easy to lose:
//!
//! - **Looping clips are solved on a wheel.** A walk cycle that starts settled
//!   on its first key has a visible hitch where the loop closes. Ours pre-rolls
//!   two full cycles before recording one, so the springs are already in their
//!   steady state and frame zero continues from the last frame exactly.
//! - **The result is clamped to what a body can do.** Springs overshoot on
//!   purpose; overshoot on a knee is a broken doll. Clamping at the end means
//!   ring can be tuned for feel without being audited for anatomy.

use crate::ease::Ease;
use crate::{DT, SUBSTEPS};
use view::clips::Clip;
use view::pose::{CHANNELS, Pose, reference};
use view::skeleton::{Group, JOINTS};

use crate::spring::Spring;

/// A pose the author cares about, at a frame they care about, and how the time
/// from here to the next key is spent.
#[derive(Clone, Copy, Debug)]
pub struct Key {
    pub frame: u16,
    pub pose: Pose,
    /// Shapes the motion **out of** this key and into the next one.
    pub ease: Ease,
}

impl Key {
    /// A key with the default easing, which eases in and out.
    pub fn at(frame: u16, pose: Pose) -> Key {
        Key {
            frame,
            pose,
            ease: Ease::SMOOTH,
        }
    }

    /// A key that says how it leaves.
    pub fn eased(frame: u16, pose: Pose, ease: Ease) -> Key {
        Key { frame, pose, ease }
    }

    pub fn with(mut self, ease: Ease) -> Key {
        self.ease = ease;
        self
    }
}

/// One part's response, in units an author can picture.
///
/// The distinction between these two matters more than it looks. The first
/// version of this file expressed weight as a *low frequency*, which does not
/// mean heavy -- it means late. The Bulwark's slam has a fourteen-frame startup
/// and its silhouette had barely moved by frame five, so there was nothing on
/// screen to read while the opponent was supposed to be deciding whether to
/// block. **Weight must read as follow-through, never as delay.** Splitting lag
/// from ring makes that mistake hard to repeat, and a test pins it.
#[derive(Clone, Copy, Debug)]
pub struct Feel {
    /// Frames this part runs behind the keys. A spring chasing a moving target
    /// settles into a steady lag of about `2·ring/frequency`, so this is the
    /// honest unit for the knob: "the hand trails the shoulder by two frames"
    /// is a sentence an animator can hold in their head, and "frequency 15" is
    /// not.
    pub lag: f32,
    /// 1.0 never overshoots; lower rings more. This is where the life is.
    pub ring: f32,
}

impl Feel {
    pub const fn new(lag: f32, ring: f32) -> Self {
        Feel { lag, ring }
    }

    /// Convert to the spring's own units.
    fn spring(&self) -> (f32, f32) {
        let ring = self.ring.clamp(0.15, 1.2);
        let lag_seconds = (self.lag / 60.0).max(1.0 / 600.0);
        (2.0 * ring / lag_seconds, ring)
    }

    /// The same feel, further down a limb. A forearm trails its upper arm and a
    /// hand trails that, which is where follow-through actually comes from --
    /// and getting it for free from the skeleton beats asking every author to
    /// remember it.
    fn at_depth(&self, depth: u8) -> Feel {
        let d = depth as f32;
        Feel {
            lag: self.lag * (1.0 + 0.5 * d),
            ring: (self.ring * (1.0 - 0.12 * d)).max(0.2),
        }
    }
}

/// How loose each part of the body is. This is the whole authoring surface for
/// physicality, and six numbers of it.
#[derive(Clone, Copy, Debug)]
pub struct Looseness {
    pub root: Feel,
    pub spine: Feel,
    pub chest: Feel,
    pub head: Feel,
    pub arms: Feel,
    pub legs: Feel,
}

impl Looseness {
    /// A martial default: hips and torso lead and are firmly controlled, head
    /// lags slightly, arms trail and ring, legs stay planted.
    pub const MARTIAL: Looseness = Looseness {
        root: Feel::new(1.0, 1.0),
        spine: Feel::new(1.2, 1.0),
        chest: Feel::new(1.4, 0.9),
        head: Feel::new(1.8, 0.8),
        arms: Feel::new(2.2, 0.62),
        legs: Feel::new(1.5, 0.9),
    };

    /// Heavier and more organic. Parts still arrive on time -- they just carry
    /// further past the mark and take longer to stop wobbling.
    pub const HEAVY: Looseness = Looseness {
        root: Feel::new(1.8, 0.95),
        spine: Feel::new(2.2, 0.85),
        chest: Feel::new(2.6, 0.7),
        head: Feel::new(3.0, 0.55),
        arms: Feel::new(3.6, 0.42),
        legs: Feel::new(2.6, 0.72),
    };

    /// Quick and precise, with almost no ring. For pokes, parries and anything
    /// whose whole point is that it is faster than you expected.
    pub const CRISP: Looseness = Looseness {
        root: Feel::new(0.8, 1.0),
        spine: Feel::new(0.9, 1.0),
        chest: Feel::new(1.0, 1.0),
        head: Feel::new(1.3, 0.9),
        arms: Feel::new(1.4, 0.8),
        legs: Feel::new(1.1, 1.0),
    };

    /// Nothing here is in control. Hit reactions, staggers, being thrown: the
    /// body is being moved rather than moving.
    pub const LIMP: Looseness = Looseness {
        root: Feel::new(2.0, 0.7),
        spine: Feel::new(2.6, 0.5),
        chest: Feel::new(3.0, 0.4),
        head: Feel::new(4.0, 0.3),
        arms: Feel::new(4.4, 0.28),
        legs: Feel::new(3.4, 0.4),
    };

    /// Airborne and unhurried. Nothing is fighting gravity, so nothing snaps.
    pub const FLOATY: Looseness = Looseness {
        root: Feel::new(2.2, 0.9),
        spine: Feel::new(2.6, 0.8),
        chest: Feel::new(2.8, 0.7),
        head: Feel::new(3.2, 0.6),
        arms: Feel::new(3.8, 0.45),
        legs: Feel::new(3.0, 0.55),
    };

    /// Ordinary walking. Loose enough to breathe, tight enough to stay on the
    /// beat -- a walk cycle that lags is a walk cycle that skates.
    pub const STRIDE: Looseness = Looseness {
        root: Feel::new(1.2, 1.0),
        spine: Feel::new(1.5, 0.95),
        chest: Feel::new(1.7, 0.85),
        head: Feel::new(2.2, 0.7),
        arms: Feel::new(2.6, 0.55),
        legs: Feel::new(1.3, 0.95),
    };

    pub const ALL: &'static [(&'static str, Looseness)] = &[
        ("crisp", Looseness::CRISP),
        ("martial", Looseness::MARTIAL),
        ("stride", Looseness::STRIDE),
        ("heavy", Looseness::HEAVY),
        ("floaty", Looseness::FLOATY),
        ("limp", Looseness::LIMP),
    ];

    pub fn name(&self) -> &'static str {
        Looseness::ALL
            .iter()
            .find(|(_, l)| {
                l.root.lag == self.root.lag
                    && l.arms.lag == self.arms.lag
                    && l.legs.ring == self.legs.ring
            })
            .map(|(n, _)| *n)
            .unwrap_or("custom")
    }

    pub fn group(&self, g: Group) -> Feel {
        match g {
            Group::Root => self.root,
            Group::Spine => self.spine,
            Group::Chest => self.chest,
            Group::Head => self.head,
            Group::Arms => self.arms,
            Group::Legs => self.legs,
        }
    }

    pub fn group_mut(&mut self, g: Group) -> &mut Feel {
        match g {
            Group::Root => &mut self.root,
            Group::Spine => &mut self.spine,
            Group::Chest => &mut self.chest,
            Group::Head => &mut self.head,
            Group::Arms => &mut self.arms,
            Group::Legs => &mut self.legs,
        }
    }
}

/// One animation to bake: which clip it fills, its keys, and its feel.
#[derive(Clone, Debug)]
pub struct Recipe {
    pub clip: Clip,
    pub keys: Vec<Key>,
    pub looseness: Looseness,
    /// Why this clip is shaped the way it is. Kept as data rather than as a
    /// comment so that a round trip through the hub cannot lose it: the hub
    /// regenerates these files, and a comment would not survive.
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

    pub fn length(&self) -> u16 {
        self.clip.length()
    }
}

#[derive(Clone, Debug)]
pub struct Baked {
    pub clip: Clip,
    /// One pose per frame.
    pub frames: Vec<Pose>,
}

/// Which spring settings each channel gets.
fn channel_springs(looseness: &Looseness) -> [Spring; CHANNELS] {
    let mut springs = [Spring::new(0.0, 20.0, 1.0); CHANNELS];
    // The hip offset rides with the root's feel.
    let (f, d) = looseness.root.spring();
    for s in springs.iter_mut().take(3) {
        *s = Spring::new(0.0, f, d);
    }
    for j in JOINTS {
        let feel = looseness.group(j.group()).at_depth(j.depth());
        let (f, d) = feel.spring();
        for c in 0..3 {
            springs[3 + j.index() * 3 + c] = Spring::new(0.0, f, d);
        }
    }
    springs
}

/// Run the solver and produce the table.
pub fn bake(recipe: &Recipe) -> Baked {
    assert!(!recipe.keys.is_empty(), "{}: no keys", recipe.clip.name());

    let length = recipe.length().max(1);
    let looping = recipe.clip.looping();
    let mut springs = channel_springs(&recipe.looseness);

    // Start settled on the first key, so a one-shot does not open with a lurch
    // from an arbitrary rest pose.
    let first = sample(&recipe.keys, 0.0, length, looping);
    for (i, s) in springs.iter_mut().enumerate() {
        s.value = first.channels[i];
        s.velocity = 0.0;
    }

    // A cycle has no beginning, so give the springs two turns of the wheel to
    // reach their steady state before anything is recorded. Without it the
    // first stride of every walk is subtly different from the rest, and the
    // seam is visible exactly where the clip loops.
    let preroll = if looping { length * 2 } else { 0 };
    let dt = DT / SUBSTEPS as f32;
    let skeleton = reference();

    let mut frames = Vec::with_capacity(length as usize);
    for frame in 0..(preroll + length) {
        for sub in 0..SUBSTEPS {
            let t = frame as f32 + sub as f32 / SUBSTEPS as f32;
            let target = sample(&recipe.keys, t, length, looping);
            for (i, s) in springs.iter_mut().enumerate() {
                s.step(target.channels[i], dt);
            }
        }
        if frame >= preroll {
            let mut pose = Pose::rest();
            for (i, s) in springs.iter().enumerate() {
                pose.channels[i] = s.value;
            }
            frames.push(pose.clamped(skeleton));
        }
    }

    Baked {
        clip: recipe.clip,
        frames,
    }
}

/// Read the key track at a fractional frame.
///
/// The interpolation between two keys is *not* smoothed here beyond the ease
/// the author asked for: the springs supply the rest, and pre-smoothing the
/// target only muddies the timing that was asked for.
pub fn sample(keys: &[Key], t: f32, length: u16, looping: bool) -> Pose {
    if keys.len() == 1 {
        return keys[0].pose;
    }
    let len = length.max(1) as f32;
    let t = if looping { t.rem_euclid(len) } else { t };

    if !looping {
        if t <= keys[0].frame as f32 {
            return keys[0].pose;
        }
        if t >= keys[keys.len() - 1].frame as f32 {
            return keys[keys.len() - 1].pose;
        }
    }

    for pair in keys.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if t >= a.frame as f32 && t <= b.frame as f32 {
            return between(&a, &b, t);
        }
    }

    // Past the last key on a loop: wrap round to the first.
    if looping {
        let last = keys[keys.len() - 1];
        let mut wrapped = keys[0];
        wrapped.frame = length;
        if t >= last.frame as f32 {
            return between(&last, &wrapped, t);
        }
        // Before the first key on a loop: come in from the last one.
        let mut tail = last;
        tail.frame = 0;
        // `tail` is the wrapped copy of the final key sitting at frame zero, so
        // anything earlier than the first key interpolates out of it.
        return between(&tail, &keys[0], t);
    }
    keys[keys.len() - 1].pose
}

fn between(a: &Key, b: &Key, t: f32) -> Pose {
    let span = (b.frame.max(a.frame + 1) - a.frame) as f32;
    let k = ((t - a.frame as f32) / span).clamp(0.0, 1.0);
    a.pose.blend(&b.pose, a.ease.at(k))
}

// ---------------------------------------------------------------------------
// Emitting the table
// ---------------------------------------------------------------------------

/// Emit the baked tables as Rust source, for checking in.
///
/// Generated code rather than a data file on purpose: it needs no loader, no
/// asset path and no runtime parsing, and a diff shows exactly what changed
/// when an animation is retuned.
///
/// One line per frame. Fifty-one channels of four decimal places is a wide
/// line, but a line per frame is the only layout where a diff of a retuned
/// clip is readable -- a pose spread over twenty lines turns a two-frame change
/// into forty lines of noise.
pub fn emit(baked: &[Baked]) -> String {
    let mut out = String::new();
    out.push_str(
        "//! Baked animation tables. GENERATED -- do not edit.\n\
         //!\n\
         //! Produced by `cargo run -p anim --bin bake`, or by the animation hub's\n\
         //! save button. The recipes live in `crates/anim/src/clips/`; edit those,\n\
         //! or edit them in the hub (F9) and save.\n\
         //!\n\
         //! Each row is one frame: three numbers for the hip offset in metres,\n\
         //! then swing, spread and twist in radians for each of the sixteen\n\
         //! joints. Playback is a lookup by frame, so this stays a pure function\n\
         //! of simulation state and rollback is unaffected.\n\n\
         // Baked numbers land where the solver puts them; a value that happens to\n\
         // sit near ln(2) or 1/pi is a coincidence, not a constant.\n\
         #![allow(clippy::approx_constant)]\n\n\
         use crate::clips::CLIP_COUNT;\n\
         use crate::pose::{CHANNELS, Pose};\n\n\
         /// Shorthand, so one frame is one line rather than twenty.\n\
         const fn p(c: [f32; CHANNELS]) -> Pose {\n    Pose::from_channels(c)\n}\n\n",
    );

    for b in baked {
        let ident = b.clip.name().to_uppercase();
        out.push_str(&format!(
            "/// `{}` -- {} frames.\n#[rustfmt::skip]\nstatic {}: [Pose; {}] = [\n",
            b.clip.name(),
            b.frames.len(),
            ident,
            b.frames.len()
        ));
        for (i, pose) in b.frames.iter().enumerate() {
            out.push_str("    p([");
            for (c, v) in pose.channels.iter().enumerate() {
                if c > 0 {
                    out.push(',');
                }
                out.push_str(&number(*v));
            }
            out.push_str(&format!("]), // {i}\n"));
        }
        out.push_str("];\n\n");
    }

    out.push_str(
        "/// Indexed by `Clip as usize`.\n\
         #[rustfmt::skip]\n\
         pub static TABLE: [&[Pose]; CLIP_COUNT] = [\n",
    );
    for b in baked {
        out.push_str(&format!("    &{},\n", b.clip.name().to_uppercase()));
    }
    out.push_str("];\n");
    out
}

/// Four decimals, trimmed. Most channels of most clips are exactly zero -- a
/// clip moves a dozen joints, not all sixteen -- so trimming takes a third off
/// the generated file for nothing.
fn number(v: f32) -> String {
    if v == 0.0 {
        return "0.".into();
    }
    let s = format!("{v:.4}");
    s.trim_end_matches('0').to_string()
}
