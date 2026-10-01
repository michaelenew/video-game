//! The Mantis: a three-metre duellist with two scythes, a guard, a parry,
//! and eyes that are late. See `docs/design/creatures/mantis.md` for what the
//! fight is and why; this file is what the Mantis *is* -- its skeleton, parts,
//! moves, clips and its own knobs. What it does to the fight is [`fight`]
//! (the guard, the frame hook, the blades); what it sees is [`sight`]; what
//! it wants is [`mind`]; what it remembers is [`habit`].
//!
//! ```text
//!                     Head
//!                      |
//!     Blade ── Arm ── Thorax ── Arm ── Blade       Wing  Wing (on the thorax)
//!                      |
//!     Abdomen ──────  Root
//!                   /  |  |  \
//!               four walking legs, femur and tibia each
//! ```
//!
//! **Upright, on four walking legs**, the thorax standing from 0.7 m to
//! 2.4 m, the abdomen carried level behind at 0.6 to 1.4 m, the head at
//! 3.0 m. The scythes hinge at the shoulder, 2.2 m up, and reach four metres
//! from its middle when they are out: the arm is 1.7 m and the blade two.
//! **It is the first creature a fighter's level swing hits squarely** --
//! the thorax spans a fighter's chest height -- and nothing about aiming
//! changes for it (§10).
//!
//! **The origin is the middle of the body on the floor**, where the four legs
//! meet. Two generated files sit beside this one: `baked.rs`, its pose table
//! (`cargo run -p anim --bin bake_beast -- --species mantis`), and `tuned.rs`,
//! its knobs (`cargo run -p sim --bin bake_tuning`).

pub(crate) mod baked;
pub mod fight;
pub mod habit;
pub mod mind;
pub mod sight;
mod tuned;

use crate::beast::{Bone, ClipDecl, Leg, NO_PARENT, Part, bone, breakables, part, v};
use crate::fixed::Fx;
use crate::oven::KnobDecl;
use crate::species::{MoveDecl, Species, SpeciesId, Stock};

/// Raw helper for writing fixed-point bounds readably.
const fn fx(num: i32, den: i32) -> i32 {
    Fx::ratio(num, den).raw()
}

// ---------------------------------------------------------------------------
// Bones
// ---------------------------------------------------------------------------

/// The Mantis's bones, by index.
pub mod bones {
    pub const ROOT: usize = 0;
    pub const ABDOMEN: usize = 1;
    pub const THORAX: usize = 2;
    pub const NECK: usize = 3;
    pub const HEAD: usize = 4;
    pub const ARM_L: usize = 5;
    pub const BLADE_L: usize = 6;
    pub const ARM_R: usize = 7;
    pub const BLADE_R: usize = 8;
    pub const WING_L: usize = 9;
    pub const WING_R: usize = 10;
    pub const FEMUR_ML: usize = 11;
    pub const TIBIA_ML: usize = 12;
    pub const FEMUR_MR: usize = 13;
    pub const TIBIA_MR: usize = 14;
    pub const FEMUR_HL: usize = 15;
    pub const TIBIA_HL: usize = 16;
    pub const FEMUR_HR: usize = 17;
    pub const TIBIA_HR: usize = 18;

    pub const COUNT: usize = 19;
}

use bones::*;

/// Where each bone sits in its parent's rest frame, in metres.
///
/// **The rest pose is the scythes thrown straight out**, level at the
/// shoulder: the arm forward along the facing and the blade on along it, so
/// the tip is four metres from the middle. Every clip folds them from there
/// -- the idle carries them folded before the chest, the way a mantis does.
pub const BONES: [Bone; COUNT] = [
    bone("root", NO_PARENT, v((0, 1), (100, 100), (0, 1)), 0), // where the legs meet, 1 m up
    bone("abdomen", ROOT, v((-30, 100), (5, 100), (0, 1)), 0),
    bone("thorax", ROOT, v((15, 100), (20, 100), (0, 1)), 0), // 1.2 m up
    bone("neck", THORAX, v((12, 100), (120, 100), (0, 1)), 0), // 2.4 m
    bone("head", NECK, v((12, 100), (35, 100), (0, 1)), 0),   // 2.75 m
    bone("arm.l", THORAX, v((20, 100), (100, 100), (-28, 100)), -1), // the shoulder, 2.2 m
    bone("blade.l", ARM_L, v((170, 100), (0, 1), (0, 1)), -1), // the elbow
    bone("arm.r", THORAX, v((20, 100), (100, 100), (28, 100)), 1),
    bone("blade.r", ARM_R, v((170, 100), (0, 1), (0, 1)), 1),
    bone("wing.l", THORAX, v((-22, 100), (95, 100), (-12, 100)), -1),
    bone("wing.r", THORAX, v((-22, 100), (95, 100), (12, 100)), 1),
    bone("femur.ml", ROOT, v((25, 100), (0, 1), (-42, 100)), -1),
    bone("tibia.ml", FEMUR_ML, v((0, 1), (-55, 100), (0, 1)), -1),
    bone("femur.mr", ROOT, v((25, 100), (0, 1), (42, 100)), 1),
    bone("tibia.mr", FEMUR_MR, v((0, 1), (-55, 100), (0, 1)), 1),
    bone("femur.hl", ROOT, v((-35, 100), (0, 1), (-42, 100)), -1),
    bone("tibia.hl", FEMUR_HL, v((0, 1), (-55, 100), (0, 1)), -1),
    bone("femur.hr", ROOT, v((-35, 100), (0, 1), (42, 100)), 1),
    bone("tibia.hr", FEMUR_HR, v((0, 1), (-55, 100), (0, 1)), 1),
];

/// The left and right bones a mirrored clip swaps.
pub const MIRROR: [(usize, usize); 7] = [
    (ARM_L, ARM_R),
    (BLADE_L, BLADE_R),
    (WING_L, WING_R),
    (FEMUR_ML, FEMUR_MR),
    (TIBIA_ML, TIBIA_MR),
    (FEMUR_HL, FEMUR_HR),
    (TIBIA_HL, TIBIA_HR),
];

/// The bones the head's tracking is spread across. The head is where it
/// sees from.
pub const NECK_CHAIN: [usize; 2] = [NECK, bones::HEAD];

/// Which bone an attack's hit volume rides: every one of its moves is
/// authored in body space and rides the body.
pub const FOLLOWS: [usize; 1] = [ROOT];

// ---------------------------------------------------------------------------
// Parts
// ---------------------------------------------------------------------------

pub const HEAD: usize = 0;
pub const THORAX_PART: usize = 1;
pub const ABDOMEN_PART: usize = 2;
pub const ARM_L_PART: usize = 3;
pub const ARM_R_PART: usize = 4;
/// **The blades**: the scythe segment of each raptorial arm, and the only
/// breakable parts. Hits on a blade wear it down; a broken one leaves its
/// guard one-sided for the rest of the fight (§4).
pub const BLADE_L_PART: usize = 5;
pub const BLADE_R_PART: usize = 6;
pub const WING_L_PART: usize = 7;
pub const WING_R_PART: usize = 8;
pub const FEMUR_ML_PART: usize = 9;
pub const TIBIA_ML_PART: usize = 10;
pub const FEMUR_MR_PART: usize = 11;
pub const TIBIA_MR_PART: usize = 12;
pub const FEMUR_HL_PART: usize = 13;
pub const TIBIA_HL_PART: usize = 14;
pub const FEMUR_HR_PART: usize = 15;
pub const TIBIA_HR_PART: usize = 16;

pub const PART_COUNT: usize = 17;

/// A part of the body: solid, and **nobody stands on it**. It is a
/// fighter's height and a half, and a creature you hop onto is not this
/// fight.
const fn body(
    name: &'static str,
    on: usize,
    min: crate::math::V3,
    max: crate::math::V3,
    vuln: Knob,
) -> Part {
    part(name, on, min, max, vuln as u16).sheds()
}

/// The Mantis's proportions, at scale one.
pub const PARTS: [Part; PART_COUNT] = [
    // The head, 2.5 to 3.0 m: a hop's height for most of the roster.
    body(
        "head",
        bones::HEAD,
        v((-15, 100), (-25, 100), (-22, 100)),
        v((30, 100), (25, 100), (22, 100)),
        Knob::VulnHead,
    ),
    // **The thorax, upright from 0.7 m to 2.4 m**: what a level swing from a
    // fighter's chest meets, and a skillshot aimed at the floor under it,
    // raised to 0.9 m, meets too (§10).
    body(
        "thorax",
        THORAX,
        v((-25, 100), (-50, 100), (-28, 100)),
        v((30, 100), (120, 100), (28, 100)),
        Knob::VulnBody,
    ),
    // The abdomen, carried level behind at 0.65 to 1.45 m.
    body(
        "abdomen",
        ABDOMEN,
        v((-180, 100), (-40, 100), (-38, 100)),
        v((10, 100), (40, 100), (38, 100)),
        Knob::VulnBody,
    ),
    // The arms are **soft**: a scythe swung through you is its move's
    // volume, not a wall that shoves.
    part(
        "left arm",
        ARM_L,
        v((-10, 100), (-12, 100), (-10, 100)),
        v((170, 100), (12, 100), (10, 100)),
        Knob::VulnArm as u16,
    )
    .soft(),
    part(
        "right arm",
        ARM_R,
        v((-10, 100), (-12, 100), (-10, 100)),
        v((170, 100), (12, 100), (10, 100)),
        Knob::VulnArm as u16,
    )
    .soft(),
    part(
        "left blade",
        BLADE_L,
        v((0, 1), (-14, 100), (-7, 100)),
        v((200, 100), (10, 100), (7, 100)),
        Knob::VulnBlade as u16,
    )
    .soft()
    .breakable(),
    part(
        "right blade",
        BLADE_R,
        v((0, 1), (-14, 100), (-7, 100)),
        v((200, 100), (10, 100), (7, 100)),
        Knob::VulnBlade as u16,
    )
    .soft()
    .breakable(),
    part(
        "left wing",
        WING_L,
        v((-130, 100), (-4, 100), (-45, 100)),
        v((5, 100), (4, 100), (0, 1)),
        Knob::VulnBody as u16,
    )
    .soft(),
    part(
        "right wing",
        WING_R,
        v((-130, 100), (-4, 100), (0, 1)),
        v((5, 100), (4, 100), (45, 100)),
        Knob::VulnBody as u16,
    )
    .soft(),
    leg_part("left middle femur", FEMUR_ML, 55),
    leg_part("left middle tibia", TIBIA_ML, 55),
    leg_part("right middle femur", FEMUR_MR, 55),
    leg_part("right middle tibia", TIBIA_MR, 55),
    leg_part("left hind femur", FEMUR_HL, 55),
    leg_part("left hind tibia", TIBIA_HL, 55),
    leg_part("right hind femur", FEMUR_HR, 55),
    leg_part("right hind tibia", TIBIA_HR, 55),
];

/// One segment of a walking leg: a thin box hanging `cm` below its bone.
const fn leg_part(name: &'static str, on: usize, cm: i32) -> Part {
    part(
        name,
        on,
        v((-7, 100), (-cm, 100), (-7, 100)),
        v((7, 100), (5, 100), (7, 100)),
        Knob::VulnLeg as u16,
    )
    .sheds()
}

/// The four walking legs: the middle pair forward of the hips, the hind pair
/// behind. A mantis's front legs are its scythes.
pub const LEGS: [Leg; 4] = [
    Leg {
        upper: FEMUR_ML_PART,
        foot: TIBIA_ML_PART,
        hip: FEMUR_ML,
        knee: TIBIA_ML,
        front: true,
        side: -1,
    },
    Leg {
        upper: FEMUR_MR_PART,
        foot: TIBIA_MR_PART,
        hip: FEMUR_MR,
        knee: TIBIA_MR,
        front: true,
        side: 1,
    },
    Leg {
        upper: FEMUR_HL_PART,
        foot: TIBIA_HL_PART,
        hip: FEMUR_HL,
        knee: TIBIA_HL,
        front: false,
        side: -1,
    },
    Leg {
        upper: FEMUR_HR_PART,
        foot: TIBIA_HR_PART,
        hip: FEMUR_HR,
        knee: TIBIA_HR,
        front: false,
        side: 1,
    },
];

// ---------------------------------------------------------------------------
// Moves
// ---------------------------------------------------------------------------

/// **The scythe pair, its first slash**: the right blade across, a step in.
/// Always followed by a second (`then`): the fast one, unless the frame hook
/// swaps the held one in as it commits.
pub const SLASH: u8 = 0;
/// The second slash, after twelve frames. Never chosen.
pub const SLASH_FAST: u8 = 1;
/// The second slash, held cocked for twenty-six. Never chosen.
pub const SLASH_HELD: u8 = 2;
/// **The coil and the lunge.** Its startup is the coil, at its longest
/// (`coil_max`); the frame hook releases it early on a commitment it sees,
/// after `CoilMin`. Its lane stops at the first solid on it.
pub const LUNGE: u8 = 3;
/// **The guard**: both blades crossed, the first `ParryWindow` frames a
/// parry. Active for as long as it may be held; the frame hook lowers it
/// after the minimum hold. Zero damage: a stance, not a hit.
pub const GUARD: u8 = 4;
/// A straight thrust out of a parry, and only out of one. Never chosen: the
/// guard's parry starts it.
pub const COUNTER: u8 = 5;
/// Drops, springs six metres up cutting on the way, hangs. Always followed by
/// the dive.
pub const LEAP: u8 = 6;
/// Down where it last saw you. Never chosen: the leap ends in it.
pub const DIVE: u8 = 7;
/// Wings snap open, a shove all round, and it hops back.
pub const FLARE: u8 = 8;
/// Plants its hind legs and whirls one blade round behind it, above a
/// crouch. Played toward the side its target is on.
pub const PIVOT: u8 = 9;
/// Arms folded, head bowed, a parry-ready guard all round, then haste.
pub const PRAYER: u8 = 10;
/// **The habit stance**: one blade cocked at the side a remembered move
/// arrives from. Parries that move only.
pub const READY: u8 = 11;
/// Its guard broken: blades flung wide and low. A stagger, timed as a
/// move's recovery. Never chosen.
pub const BROKEN: u8 = 12;
/// Staggered: a lunge into a solid. Never chosen.
pub const STAGGER: u8 = 13;

pub const MOVE_COUNT: usize = 14;

pub const MOVES: [MoveDecl; MOVE_COUNT] = [
    MoveDecl::new("Scythe", Clip::Slash as usize).then(SLASH_FAST),
    MoveDecl::new("Second slash", Clip::Slash2 as usize).never_chosen(),
    MoveDecl::new("Second slash, held", Clip::Slash2 as usize).never_chosen(),
    MoveDecl::new("Lunge", Clip::Lunge as usize).stops_at_aim(),
    MoveDecl::new("Guard", Clip::Guard as usize),
    MoveDecl::new("Counter", Clip::Counter as usize).never_chosen(),
    MoveDecl::new("Leap", Clip::Leap as usize).then(DIVE),
    MoveDecl::new("Dive", Clip::Dive as usize)
        .lobbed()
        .never_chosen(),
    MoveDecl::new("Wing flare", Clip::Flare as usize),
    MoveDecl::new("Pivot cut", Clip::Pivot as usize).mirrors_to_target_side(),
    MoveDecl::new("Prayer", Clip::Prayer as usize),
    MoveDecl::new("Ready", Clip::Ready as usize),
    MoveDecl::new("Guard broken", Clip::Broken as usize).never_chosen(),
    MoveDecl::new("Staggered", Clip::Stagger as usize).never_chosen(),
];

/// The moves that are a stance rather than a hit: no volume, and what is
/// under way is the guard (or the prayer, or Ready).
pub const fn stance(kind: u8) -> bool {
    matches!(kind, GUARD | PRAYER | READY)
}

// ---------------------------------------------------------------------------
// Clips
// ---------------------------------------------------------------------------

/// Everything the Mantis knows how to look like, in the order its baked table
/// lays them out. Authored in `crates/anim/src/beast/mantis/`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Clip {
    Idle,
    Walk,
    /// The skitter: low and fast, the legs a blur, to close a gap.
    Skitter,
    Slash,
    Slash2,
    /// The coil, and the lunge out of it.
    Lunge,
    Guard,
    Counter,
    Leap,
    Dive,
    Flare,
    Pivot,
    Prayer,
    Ready,
    /// Blades flung wide and low: the guard broken.
    Broken,
    /// Rocked back on its legs: a lunge into a solid.
    Stagger,
    Flinch,
    /// Down on its legs: a blade breaking, or crowd control landing.
    Stumble,
    Topple,
    Dead,
}

pub const CLIP_COUNT: usize = 20;

impl Clip {
    pub const ALL: [Clip; CLIP_COUNT] = [
        Clip::Idle,
        Clip::Walk,
        Clip::Skitter,
        Clip::Slash,
        Clip::Slash2,
        Clip::Lunge,
        Clip::Guard,
        Clip::Counter,
        Clip::Leap,
        Clip::Dive,
        Clip::Flare,
        Clip::Pivot,
        Clip::Prayer,
        Clip::Ready,
        Clip::Broken,
        Clip::Stagger,
        Clip::Flinch,
        Clip::Stumble,
        Clip::Topple,
        Clip::Dead,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn name(self) -> &'static str {
        CLIPS[self as usize].name
    }
}

impl From<Clip> for usize {
    fn from(clip: Clip) -> usize {
        clip.index()
    }
}

const fn cycle(name: &'static str) -> ClipDecl {
    ClipDecl {
        name,
        looping: true,
        phased: false,
    }
}

const fn attack(name: &'static str) -> ClipDecl {
    ClipDecl {
        name,
        looping: false,
        phased: true,
    }
}

const fn once(name: &'static str) -> ClipDecl {
    ClipDecl {
        name,
        looping: false,
        phased: false,
    }
}

pub const CLIPS: [ClipDecl; CLIP_COUNT] = [
    cycle("idle"),
    cycle("walk"),
    cycle("skitter"),
    attack("slash"),
    attack("slash2"),
    attack("lunge"),
    attack("guard"),
    attack("counter"),
    attack("leap"),
    attack("dive"),
    attack("flare"),
    attack("pivot"),
    attack("prayer"),
    attack("ready"),
    attack("broken"),
    attack("stagger"),
    once("flinch"),
    once("stumble"),
    once("topple"),
    once("dead"),
];

// ---------------------------------------------------------------------------
// Its own knobs
// ---------------------------------------------------------------------------

crate::species_knobs! {
    /// The knobs only the Mantis has. The design document's names (§5) are in
    /// the comment beside each, where they differ.
    pub enum Knob;
    VulnHead,        "hide",   "Head (x damage)",                          Fixed, 0, fx(3,1);
    VulnBody,        "hide",   "Body (x damage)",                          Fixed, 0, fx(3,1);
    VulnArm,         "hide",   "Arms (x damage)",                          Fixed, 0, fx(3,1);
    VulnBlade,       "hide",   "Blades (x damage)",                        Fixed, 0, fx(3,1);
    VulnLeg,         "hide",   "Legs (x damage)",                          Fixed, 0, fx(3,1);
    BrokenArms,      "hide",   "Guard broken, arms and blades (x)",        Fixed, 0, fx(3,1);
    ReadyWrong,      "hide",   "Ready against another move, clean hit (x)", Fixed, 0, fx(3,1);
    // sight_min, sight_max, sight_wander
    SightMin,        "eyes",   "Sees this late, at the soonest",           Frames, 0, 60;
    SightMax,        "eyes",   "Sees this late, at the latest",            Frames, 0, 60;
    SightWander,     "eyes",   "Its lateness wanders a frame every",       Frames, 1, 240;
    SampleEvery,     "eyes",   "Remembers where you were every",           Frames, 1, 4;
    AloftAbove,      "eyes",   "Feet this far off the floor are aloft",    Fixed, 0, fx(5,1);
    // mantis_parry_window, guard_min_hold, guard_arc, guard_elevation, guard_turn
    ParryWindow,     "guard",  "Its first frames parry",                   Frames, 0, 30;
    GuardMinHold,    "guard",  "Held at least",                            Frames, 0, 120;
    GuardArc,        "guard",  "Covers either side of its facing (turns)", Fixed, 0, fx(1,2);
    GuardElevation,  "guard",  "Covers up to above level (turns)",         Fixed, 0, fx(1,4);
    GuardBelow,      "guard",  "Covers down to below level (turns)",       Fixed, 0, fx(1,4);
    GuardChest,      "guard",  "The cone's point, this high",              Fixed, 0, fx(4,1);
    GuardTurn,       "guard",  "Turns while guarding (turns/s)",           Fixed, 0, fx(2,1);
    GuardWalk,       "guard",  "Walks with its guard up at",               Fixed, 0, fx(10,1);
    GuardQuiet,      "guard",  "Lowers it, nothing seen coming for",       Frames, 0, 120;
    GuardPush,       "guard",  "A blocked hit pushes it",                  Fixed, 0, fx(3,1);
    GuardStun,       "guard",  "A blocked hit holds it",                   Frames, 0, 60;
    // guard_break_stagger
    BreakStagger,    "guard",  "Guard broken, staggered for",              Frames, 0, 300;
    PrayerStagger,   "guard",  "Prayer broken, staggered for",             Frames, 0, 300;
    OneSideWhole,    "guard",  "One blade, covers the whole side to (turns)", Fixed, 0, fx(1,2);
    OneSideAcross,   "guard",  "One blade, covers across the broken side (turns)", Fixed, 0, fx(1,4);
    GuardReach,      "guard",  "Raises it against a move that reaches within", Fixed, 0, fx(20,1);
    DropsFor,        "guard",  "Drops it before a breaker this many frames out", Frames, 0, 60;
    LateGuard,       "guard",  "Raises it on a move seen too late to parry (%)", Percent, 0, 100;
    GuardMost,       "guard",  "Holds it at most",                         Frames, 0, 600;
    GuardRest,       "guard",  "Lowered, raises it again no sooner than",  Frames, 0, 240;
    // coil_min, coil_max (the lunge's startup), lunge_lead
    CoilMin,         "coil",   "Coiled at least",                          Frames, 0, 120;
    LungeLead,       "coil",   "The lunge leads what it saw by (x)",       Fixed, 0, fx(2,1);
    LungeStagger,    "coil",   "A lunge into a solid, staggered for",      Frames, 0, 240;
    LungeTravel,     "coil",   "The lunge crosses at most",                Fixed, 0, fx(20,1);
    // the leap
    LeapHeight,      "leap",   "Springs this high",                        Fixed, 0, fx(12,1);
    LeapCalled,      "leap",   "Anyone aloft within this calls the Leap",  Fixed, 0, fx(20,1);
    // flare_brace
    FlareBrace,      "flare",  "A braced fighter slides (x of the shove)", Fixed, 0, fx(1,1);
    FlareHop,        "flare",  "Hops back",                                Fixed, 0, fx(10,1);
    FlareHopFrames,  "flare",  "Over",                                     Frames, 1, 60;
    // the pair's variants
    HeldShare,       "scythes","The second slash is the held one (%)",     Percent, 0, 100;
    // seen(m), punish(m), back(m), far(m), back_patience
    SeenAppetite,    "mind",   "seen(m): a commitment it saw, appetite",   Int, 0, 8000;
    PunishAppetite,  "mind",   "punish(m): a recovery it saw, appetite",   Int, 0, 8000;
    BackPatience,    "mind",   "back(m): outside its arc, patience",       Frames, 1, 600;
    BackNear,        "mind",   "back(m): within",                          Fixed, 0, fx(10,1);
    BackAppetite,    "mind",   "back(m): appetite, at its patience",       Int, 0, 8000;
    FarRange,        "mind",   "far(m): a hunter past",                    Fixed, 0, fx(30,1);
    FarIdle,         "mind",   "far(m): or no commitment seen for",        Frames, 0, 600;
    FarAppetite,     "mind",   "far(m): appetite",                         Int, 0, 8000;
    GuardAppetite,   "mind",   "Guards in neutral, appetite",              Int, 0, 8000;
    StandOff,        "mind",   "Holds you at",                             Fixed, 0, fx(20,1);
    // the desperate stage, and the hurt one
    HurtBelow,       "arc",    "Hurt below health (x)",                    Fixed, 0, fx(1,1);
    DesperateBelow,  "arc",    "Desperate below health (x)",               Fixed, 0, fx(1,1);
    HurtGuard,       "arc",    "Hurt, guards (x appetite)",                Fixed, 0, fx(2,1);
    HurtAttack,      "arc",    "Hurt, attacks (x appetite)",               Fixed, 0, fx(3,1);
    DesperateThink,  "arc",    "Desperate, the beat between moves (x)",    Fixed, 0, fx(1,1);
    // prayer_frames is the move's; prayer_haste
    HasteStartup,    "prayer", "Haste: startups (x)",                      Fixed, 0, fx(1,1);
    HasteFloor,      "prayer", "Haste: never a startup under",             Frames, 0, 60;
    HasteRecovery,   "prayer", "Haste: recoveries (x)",                    Fixed, 0, fx(1,1);
    HastePips,       "prayer", "Haste: moves",                             Int, 0, 7;
    PipSize,         "prayer", "Haste: a pip on its thorax, across",       Fixed, 0, fx(1,1);
    // habit_depth, habit_anticipate, habit_settle, habit_weight, ready_hold is the move's
    Habit,           "habit",  "Remembers what strikes its guard",         Flag, 0, 1;
    HabitDepth,      "habit",  "Remembers the last",                       Int, 0, 4;
    HabitAnticipate, "habit",  "Ready when one move holds",                Int, 0, 4;
    HabitHurt,       "habit",  "Hurt, Ready when one move holds",          Int, 0, 4;
    HabitSettle,     "habit",  "A memory is usable after",                 Frames, 0, 240;
    HabitWeight,     "habit",  "ready(m): per copy past one",              Int, 0, 8000;
    HabitReach,      "habit",  "ready(m): its thrower within its reach and", Fixed, 0, fx(5,1);
    GuessWeight,     "habit",  "Off: guess(m), appetite",                  Int, 0, 8000;
    GuessThink,      "habit",  "Off: the beat between moves (x)",          Fixed, 0, fx(1,1);
    NotchSize,       "habit",  "A notch on its blade, across",             Fixed, 0, fx(1,1);
    // coop
    CoopHealth,      "coop",   "Two hunters, health",                      Int, 0, 30000;
    CoopThink,       "coop",   "Two hunters, the beat between moves (x)",  Fixed, 0, fx(1,1);
    CoopPatience,    "coop",   "Two hunters, back(m) patience (x)",        Fixed, 0, fx(1,1);
    PairAngle,       "coop",   "pair(m): lines them up inside (turns)",    Fixed, 0, fx(1,2);
    PairAppetite,    "coop",   "pair(m): walks to line them up",           Int, 0, 8000;
}

const OWN: &[KnobDecl] = Knob::DECLS;

impl Knob {
    /// The live value, as fixed point.
    pub fn fx(self) -> Fx {
        SPECIES.own_fx(self as usize)
    }

    /// The live value, raw: an integer, or frames.
    pub fn raw(self) -> i32 {
        SPECIES.own_raw(self as usize)
    }

    /// The live value, as frames.
    pub fn frames(self) -> u16 {
        self.raw().clamp(0, u16::MAX as i32) as u16
    }
}

// ---------------------------------------------------------------------------
// The species
// ---------------------------------------------------------------------------

pub static SPECIES: Species = Species {
    id: SpeciesId::MANTIS,
    name: "Mantis",
    bones: &BONES,
    mirror: &MIRROR,
    neck: &NECK_CHAIN,
    follows: &FOLLOWS,
    parts: &PARTS,
    breakable: breakables(&PARTS),
    legs: &LEGS,
    moves: &MOVES,
    clips: &CLIPS,
    stock: Stock {
        idle: Clip::Idle as usize,
        walk: Clip::Walk as usize,
        gallop: Clip::Skitter as usize,
        flinch: Clip::Flinch as usize,
        stumble: Clip::Stumble as usize,
        topple: Clip::Topple as usize,
        dead: Clip::Dead as usize,
    },
    span: &baked::SPAN,
    rows: baked::ROWS,
    row: |r| &baked::FRAMES[r],
    own: OWN,
    tuned: &tuned::KNOBS,
    tuned_path: "crates/sim/src/species/mantis/tuned.rs",
    pack: None,
    fight: &fight::FIGHT,
};
