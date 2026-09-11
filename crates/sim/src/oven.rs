//! The Oven: every tuned number in the game, live.
//!
//! Feel work is a loop — change a number, play it, change it again — and the
//! loop is only as fast as its slowest step. With the values compiled in, that
//! step is a rebuild, so in practice you change one number, wait, and lose the
//! comparison you were trying to make. The Oven makes the numbers runtime state
//! so the loop closes in a frame, and adds a **bake** that writes them back to
//! the repository so a session of tuning ends as a commit rather than as
//! something you have to remember and retype.
//!
//! ## Every knob is an `i32`
//!
//! Fixed-point values are stored as their raw 16.16 bits, frame counts as
//! frames, health as health. One representation means one store, one editor
//! widget and one file format for three hundred parameters that would otherwise
//! each need their own. `Unit` says how to read the integer back.
//!
//! ## Determinism
//!
//! These are *rules*, not state: they never change during a frame, so rollback
//! neither saves nor restores them. But two peers running different rules would
//! desync silently and look like a netcode bug, so the tuning hash is folded
//! into `World::checksum` — a mismatched Oven shows up immediately as a desync
//! rather than as a mystery.

use crate::class::{ALL_CLASSES, Class};
use crate::fixed::Fx;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicI32, Ordering};

use crate::tuned;

/// How to read a knob's integer back, and how to show it to a person.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unit {
    /// Raw 16.16 fixed point.
    Fixed,
    /// Whole frames at 60 Hz.
    Frames,
    /// A plain count.
    Int,
    /// 0–100.
    Percent,
    /// 0 or 1.
    Flag,
}

/// Raw helper for writing fixed-point bounds readably.
const fn fx(num: i32, den: i32) -> i32 {
    Fx::ratio(num, den).raw()
}

impl Unit {
    /// Render a raw value for a human, using integer arithmetic only.
    ///
    /// No `f32` anywhere in this crate, Oven included: `sim` is float-free as a
    /// determinism guarantee, and an editor is not a good enough reason to put
    /// the first one in. Converting for display is the palette's job; this
    /// exists so a baked file can carry a readable comment.
    pub fn show(self, raw: i32) -> String {
        match self {
            Unit::Fixed => {
                // Rounded on the magnitude. A shift floors, which rounds the
                // wrong way for negatives and turned -1.0 into "-1.001".
                let sign = if raw < 0 { "-" } else { "" };
                let m = ((raw as i64).abs() * 1000 + 32768) >> 16;
                let frac = format!("{:03}", m % 1000);
                let frac = frac.trim_end_matches('0');
                if frac.is_empty() {
                    format!("{sign}{}", m / 1000)
                } else {
                    format!("{sign}{}.{frac}", m / 1000)
                }
            }
            Unit::Flag => if raw != 0 { "on" } else { "off" }.to_string(),
            _ => raw.to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Scalars: the universal rules
// ---------------------------------------------------------------------------

macro_rules! scalars {
    ($($variant:ident, $family:literal, $label:literal, $unit:ident, $lo:expr, $hi:expr;)*) => {
        /// A universal rule, applying to every fighter.
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum Scalar { $($variant,)* }

        impl Scalar {
            pub const ALL: &'static [Scalar] = &[$(Scalar::$variant,)*];

            pub const fn family(self) -> &'static str {
                match self { $(Scalar::$variant => $family,)* }
            }

            pub const fn label(self) -> &'static str {
                match self { $(Scalar::$variant => $label,)* }
            }

            pub const fn unit(self) -> Unit {
                match self { $(Scalar::$variant => Unit::$unit,)* }
            }

            /// Sensible editing bounds, in raw units. Not correctness limits --
            /// the feel tests are what stop a value being wrong. These only keep
            /// a slider usable.
            pub const fn range(self) -> (i32, i32) {
                match self { $(Scalar::$variant => ($lo, $hi),)* }
            }
        }
    };
}

scalars! {
    MoveSpeed,        "Movement", "Walk speed",             Fixed,   fx(1,1),   fx(20,1);
    GuardMoveSpeed,   "Movement", "Walk speed, guarding",   Fixed,   0,         fx(10,1);
    CrouchMoveSpeed,  "Movement", "Walk speed, crouching",  Fixed,   0,         fx(10,1);
    TurnRate,         "Movement", "Turn rate",              Fixed,   fx(1,100), fx(1,1);
    GuardTurnRate,    "Movement", "Turn rate, guarding",    Fixed,   fx(1,100), fx(1,1);
    PokeMobility,     "Movement", "Poke mobility (%)",      Percent, 0,         100;
    AttackRootDecay,  "Movement", "Root decay per frame",   Fixed,   0,         fx(1,1);
    JumpSpeed,        "Air",      "Takeoff speed",          Fixed,   fx(1,1),   fx(25,1);
    Gravity,          "Air",      "Gravity",                Fixed,   fx(-80,1), fx(-1,1);
    FallCap,          "Air",      "Terminal velocity",      Fixed,   fx(-60,1), fx(-1,1);
    JumpHoldGravity,  "Air",      "Gravity while held",     Fixed,   fx(1,10),  fx(1,1);
    JumpHoldFrames,   "Air",      "Sustain window",         Frames,  0,         60;
    AirAccel,         "Air",      "Air acceleration",       Fixed,   0,         fx(40,1);
    AirSpeedCap,      "Air",      "Air speed cap (x walk)", Fixed,   fx(1,1),   fx(4,1);
    GuardArcCos,      "Defence",  "Guard arc (cos)",        Fixed,   fx(-1,1),  fx(1,1);
    ParryWindow,      "Defence",  "Parry window",           Frames,  1,         20;
    ParryStagger,     "Defence",  "Parry stagger",          Frames,  1,         100;
    DodgeFrames,      "Defence",  "Dodge length",           Frames,  1,         60;
    DodgeIframes,     "Defence",  "Dodge invulnerability",  Frames,  0,         60;
    DodgeSpeed,       "Defence",  "Dodge speed",            Fixed,   fx(1,1),   fx(40,1);
    AirDodgeFrames,   "Defence",  "Airdodge length",        Frames,  1,         60;
    AirDodgeSpeed,    "Defence",  "Airdodge speed",         Fixed,   fx(1,1),   fx(40,1);
    KnockbackDecay,   "Defence",  "Knockback decay",        Fixed,   0,         fx(1,1);
    BodyRadius,       "Body",     "Body radius",            Fixed,   fx(1,10),  fx(2,1);
    BodyHeight,       "Body",     "Body height",            Fixed,   fx(1,2),   fx(4,1);
    CrouchHeightScale,"Body",     "Crouch height (x)",      Fixed,   fx(1,10),  fx(1,1);
    MaxHealth,        "Match",    "Max health",             Int,     100,       5000;
    RoundOverFrames,  "Match",    "Round-over pause",       Frames,  30,        600;
    AirStallDamp,     "Air",      "Aerial hang damping",    Fixed,   0,         fx(1,1);
    AirAttackBoost,   "Air",      "Aerial poke boost",      Fixed,   0,         fx(10,1);
    JumpReleaseCut,   "Air",      "Rise kept on release",   Fixed,   fx(1,10),  fx(1,1);
}

// ---------------------------------------------------------------------------
// Per-class air, and per-move frame data
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AirField {
    Jump,
    Gravity,
    FallCap,
    AirSpeed,
}

impl AirField {
    pub const ALL: &'static [AirField] = &[
        AirField::Jump,
        AirField::Gravity,
        AirField::FallCap,
        AirField::AirSpeed,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            AirField::Jump => "Jump (x)",
            AirField::Gravity => "Gravity (x)",
            AirField::FallCap => "Fall cap (x)",
            AirField::AirSpeed => "Steering",
        }
    }

    pub const fn range(self) -> (i32, i32) {
        match self {
            AirField::AirSpeed => (0, fx(5, 1)),
            _ => (fx(3, 10), fx(5, 2)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveField {
    Startup,
    Active,
    Recovery,
    Damage,
    Reach,
    Radius,
    Hitstun,
    Blockstun,
    Knockback,
    Mobility,
    AirStall,
    Unblockable,
    HitsCrouching,
    NeedsMechanic,
}

impl MoveField {
    pub const ALL: &'static [MoveField] = &[
        MoveField::Startup,
        MoveField::Active,
        MoveField::Recovery,
        MoveField::Damage,
        MoveField::Reach,
        MoveField::Radius,
        MoveField::Hitstun,
        MoveField::Blockstun,
        MoveField::Knockback,
        MoveField::Mobility,
        MoveField::AirStall,
        MoveField::Unblockable,
        MoveField::HitsCrouching,
        MoveField::NeedsMechanic,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            MoveField::Startup => "Startup",
            MoveField::Active => "Active",
            MoveField::Recovery => "Recovery",
            MoveField::Damage => "Damage",
            MoveField::Reach => "Reach",
            MoveField::Radius => "Radius",
            MoveField::Hitstun => "Hitstun",
            MoveField::Blockstun => "Blockstun",
            MoveField::Knockback => "Knockback",
            MoveField::Mobility => "Mobility (%)",
            MoveField::AirStall => "Aerial hang",
            MoveField::Unblockable => "Unblockable",
            MoveField::HitsCrouching => "Hits crouching",
            MoveField::NeedsMechanic => "Needs mechanic",
        }
    }

    pub const fn unit(self) -> Unit {
        match self {
            MoveField::Startup
            | MoveField::Active
            | MoveField::Recovery
            | MoveField::Hitstun
            | MoveField::Blockstun
            | MoveField::AirStall => Unit::Frames,
            MoveField::Damage => Unit::Int,
            MoveField::Mobility => Unit::Percent,
            MoveField::Unblockable | MoveField::HitsCrouching | MoveField::NeedsMechanic => {
                Unit::Flag
            }
            _ => Unit::Fixed,
        }
    }

    pub const fn range(self) -> (i32, i32) {
        match self.unit() {
            Unit::Frames => (0, 90),
            Unit::Int => (0, 600),
            Unit::Percent => (0, 100),
            Unit::Flag => (0, 1),
            Unit::Fixed => (0, fx(12, 1)),
        }
    }
}

pub const SLOTS: usize = 3;
pub const CLASSES: usize = 6;
pub const SCALAR_COUNT: usize = 31;
pub const AIR_COUNT: usize = CLASSES * 4;
pub const MOVE_COUNT: usize = CLASSES * SLOTS * 14;

// ---------------------------------------------------------------------------
// The live store
// ---------------------------------------------------------------------------

fn cells<const N: usize>(seed: &'static [i32; N]) -> [AtomicI32; N] {
    std::array::from_fn(|i| AtomicI32::new(seed[i]))
}

static SCALAR_CELLS: LazyLock<[AtomicI32; SCALAR_COUNT]> = LazyLock::new(|| cells(&tuned::SCALARS));
static AIR_CELLS: LazyLock<[AtomicI32; AIR_COUNT]> = LazyLock::new(|| cells(&tuned::AIR));
static MOVE_CELLS: LazyLock<[AtomicI32; MOVE_COUNT]> = LazyLock::new(|| cells(&tuned::MOVES));

pub fn scalar(s: Scalar) -> i32 {
    SCALAR_CELLS[s as usize].load(Ordering::Relaxed)
}

pub fn set_scalar(s: Scalar, raw: i32) {
    SCALAR_CELLS[s as usize].store(raw, Ordering::Relaxed);
}

fn air_index(class: Class, field: AirField) -> usize {
    class as usize * 4 + field as usize
}

pub fn air(class: Class, field: AirField) -> i32 {
    AIR_CELLS[air_index(class, field)].load(Ordering::Relaxed)
}

pub fn set_air(class: Class, field: AirField, raw: i32) {
    AIR_CELLS[air_index(class, field)].store(raw, Ordering::Relaxed);
}

fn move_index(class: Class, slot: usize, field: MoveField) -> usize {
    (class as usize * SLOTS + slot) * 14 + field as usize
}

pub fn move_field(class: Class, slot: usize, field: MoveField) -> i32 {
    MOVE_CELLS[move_index(class, slot, field)].load(Ordering::Relaxed)
}

pub fn set_move_field(class: Class, slot: usize, field: MoveField, raw: i32) {
    MOVE_CELLS[move_index(class, slot, field)].store(raw, Ordering::Relaxed);
}

/// Put everything back to what is committed in `tuned.rs`.
pub fn reset_to_baked() {
    for (i, cell) in SCALAR_CELLS.iter().enumerate() {
        cell.store(tuned::SCALARS[i], Ordering::Relaxed);
    }
    for (i, cell) in AIR_CELLS.iter().enumerate() {
        cell.store(tuned::AIR[i], Ordering::Relaxed);
    }
    for (i, cell) in MOVE_CELLS.iter().enumerate() {
        cell.store(tuned::MOVES[i], Ordering::Relaxed);
    }
}

/// True when the live values differ from what is committed -- what the bake
/// button needs to know, and what stops an empty commit.
pub fn is_dirty() -> bool {
    SCALAR_CELLS
        .iter()
        .enumerate()
        .any(|(i, c)| c.load(Ordering::Relaxed) != tuned::SCALARS[i])
        || AIR_CELLS
            .iter()
            .enumerate()
            .any(|(i, c)| c.load(Ordering::Relaxed) != tuned::AIR[i])
        || MOVE_CELLS
            .iter()
            .enumerate()
            .any(|(i, c)| c.load(Ordering::Relaxed) != tuned::MOVES[i])
}

/// A hash of every live value.
///
/// Folded into `World::checksum` so that two peers tuned differently show up as
/// a desync on the first frame, rather than as a slow divergence that reads
/// like a netcode bug.
pub fn hash() -> u64 {
    use std::hash::Hasher;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for c in SCALAR_CELLS.iter() {
        h.write_i32(c.load(Ordering::Relaxed));
    }
    for c in AIR_CELLS.iter() {
        h.write_i32(c.load(Ordering::Relaxed));
    }
    for c in MOVE_CELLS.iter() {
        h.write_i32(c.load(Ordering::Relaxed));
    }
    h.finish()
}

// ---------------------------------------------------------------------------
// The registry the palette reads
// ---------------------------------------------------------------------------

/// One editable value, with everything the palette needs to show it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Knob {
    Scalar(Scalar),
    Air(Class, AirField),
    Move(Class, usize, MoveField),
}

impl Knob {
    /// The family a knob belongs to. Families are how three hundred numbers
    /// stay navigable without search; search is how you find one when they do
    /// not.
    pub fn family(self) -> String {
        match self {
            Knob::Scalar(s) => s.family().to_string(),
            Knob::Air(c, _) => format!("Air · {}", c.name()),
            Knob::Move(c, slot, _) => {
                format!("{} · {}", c.name(), crate::moves::get(c, slot as u8).name)
            }
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Knob::Scalar(s) => s.label(),
            Knob::Air(_, f) => f.label(),
            Knob::Move(_, _, f) => f.label(),
        }
    }

    /// A stable identifier. Used for search and for the comments in `tuned.rs`,
    /// which is what makes a baked diff readable.
    pub fn id(self) -> String {
        fn slug(s: &str) -> String {
            s.to_lowercase().replace(' ', "_")
        }
        match self {
            Knob::Scalar(s) => format!("{}.{}", slug(s.family()), slug(s.label())),
            Knob::Air(c, f) => format!("air.{}.{}", slug(c.name()), slug(f.label())),
            Knob::Move(c, slot, f) => format!(
                "move.{}.{}.{}",
                slug(c.name()),
                slug(crate::moves::get(c, slot as u8).name),
                slug(f.label())
            ),
        }
    }

    pub fn unit(self) -> Unit {
        match self {
            Knob::Scalar(s) => s.unit(),
            Knob::Air(_, _) => Unit::Fixed,
            Knob::Move(_, _, f) => f.unit(),
        }
    }

    pub const fn range(self) -> (i32, i32) {
        match self {
            Knob::Scalar(s) => s.range(),
            Knob::Air(_, f) => f.range(),
            Knob::Move(_, _, f) => f.range(),
        }
    }

    pub fn raw(self) -> i32 {
        match self {
            Knob::Scalar(s) => scalar(s),
            Knob::Air(c, f) => air(c, f),
            Knob::Move(c, slot, f) => move_field(c, slot, f),
        }
    }

    pub fn set_raw(self, raw: i32) {
        match self {
            Knob::Scalar(s) => set_scalar(s, raw),
            Knob::Air(c, f) => set_air(c, f, raw),
            Knob::Move(c, slot, f) => set_move_field(c, slot, f, raw),
        }
    }

    /// What is committed in `tuned.rs`, for showing how far a knob has drifted.
    pub fn baked_raw(self) -> i32 {
        match self {
            Knob::Scalar(s) => tuned::SCALARS[s as usize],
            Knob::Air(c, f) => tuned::AIR[air_index(c, f)],
            Knob::Move(c, slot, f) => tuned::MOVES[move_index(c, slot, f)],
        }
    }

    pub fn is_dirty(self) -> bool {
        self.raw() != self.baked_raw()
    }
}

/// Knobs collected under their family, each family appearing once, in the order
/// families are first met.
///
/// Deliberately not "runs of adjacent knobs sharing a family". The registry's
/// order is a *storage* concern — appending a new knob to the end is what keeps
/// every existing index in `tuned.rs` valid, so a knob almost always arrives
/// away from its relatives. The first version of the palette grouped by
/// adjacency and drew a second "Air" header the moment two air scalars were
/// appended, which egui flagged as a duplicate widget.
pub fn grouped(knobs: &[Knob]) -> Vec<(String, Vec<Knob>)> {
    let mut out: Vec<(String, Vec<Knob>)> = Vec::new();
    for knob in knobs {
        let family = knob.family();
        match out.iter_mut().find(|(name, _)| *name == family) {
            Some((_, members)) => members.push(*knob),
            None => out.push((family, vec![*knob])),
        }
    }
    out
}

/// Every knob, in a stable order: universal rules, then air, then move data.
pub fn all_knobs() -> Vec<Knob> {
    let mut out: Vec<Knob> = Scalar::ALL.iter().map(|s| Knob::Scalar(*s)).collect();
    for class in ALL_CLASSES {
        for field in AirField::ALL {
            out.push(Knob::Air(class, *field));
        }
    }
    for class in ALL_CLASSES {
        for slot in 0..SLOTS {
            for field in MoveField::ALL {
                out.push(Knob::Move(class, slot, *field));
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Baking
// ---------------------------------------------------------------------------

/// Emit the live values as the source of `crates/sim/src/tuned.rs`.
///
/// Generated Rust rather than a data file, for the same reasons the baked
/// animations are: no loader, no asset path, no runtime parsing, and a diff
/// that shows exactly which numbers a tuning session moved. Every entry carries
/// its identifier as a comment, which is what makes that diff readable by
/// someone who was not in the session.
pub fn emit() -> String {
    let mut out = String::new();
    out.push_str(
        "//! Baked tuning values. GENERATED -- do not edit by hand.\n\
         //!\n\
         //! Written by the Oven's bake button, or by `cargo run -p sim --bin bake_tuning`.\n\
         //! Edit these in the running game (F7) and bake; the palette is the editor.\n\
         //!\n\
         //! Each entry is an integer: fixed-point values are raw 16.16 bits,\n\
         //! frame counts are frames. See `oven::Unit`.\n\n",
    );

    let knobs = all_knobs();
    let write_block = |out: &mut String, name: &str, count: usize, from: usize| {
        // `rustfmt::skip` so the file stays byte-identical to what `emit`
        // produces. Without it `cargo fmt` re-columnises the arrays, the
        // generated file stops matching its generator, and the test that
        // catches a stale bake has to be weakened to a fuzzy comparison.
        out.push_str(&format!(
            "#[rustfmt::skip]\npub const {name}: [i32; {count}] = [\n"
        ));
        for k in knobs.iter().skip(from).take(count) {
            out.push_str(&format!(
                "    {:>12}, // {} = {}\n",
                k.raw(),
                k.id(),
                k.unit().show(k.raw())
            ));
        }
        out.push_str("];\n\n");
    };

    write_block(&mut out, "SCALARS", SCALAR_COUNT, 0);
    write_block(&mut out, "AIR", AIR_COUNT, SCALAR_COUNT);
    write_block(&mut out, "MOVES", MOVE_COUNT, SCALAR_COUNT + AIR_COUNT);
    // Exactly one trailing newline: `cargo fmt` strips a blank line at the end
    // of a file, which would leave the generated file permanently one byte
    // different from its generator.
    out.truncate(out.trim_end().len());
    out.push('\n');
    out
}
