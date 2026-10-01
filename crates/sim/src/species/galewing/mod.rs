//! The Galewing: a raptor with an eighteen-metre wingspan that lives where
//! you cannot reach it. See `docs/design/creatures/galewing.md` for what the
//! fight is and why; this file is what the bird *is* -- its skeleton, parts,
//! moves, clips and its own knobs. How it flies, what its moves do to the
//! world and what the world does to its wings is [`fight`] and [`flight`];
//! what it wants is [`mind`].
//!
//! ```text
//!            HandL ── ArmL ── ShoulderL ─┐
//!                                        │
//!   Tail ───────────────────────────── Root ── Chest ── Neck ── Head
//!                                        │
//!            HandR ── ArmR ── ShoulderR ─┘
//!                              ThighL/ShinL, ThighR/ShinR hang under Root
//! ```
//!
//! **The rest pose is flight**: wings spread level, eighteen metres tip to
//! tip, three metres deep at the root and a metre and a half at the tip.
//! Standing, the clips fold them along the body. The body's middle stands
//! 2.6 m off the floor on its legs, so its back is at 3.4 m -- the
//! Ridgeback's climb question for a bird grounded for good (§4) -- and on its
//! breast after a crash the back comes down to about 2.4 m with the wings
//! lying on the floor as ramps.
//!
//! **Its bank and pitch are the root bone's roll and pitch** (put on by the
//! species' `repose` hook from its flight state), so everything built on
//! the rig -- the parts, the hit test, the riders and the grip test -- sees
//! the banked bird with no machinery of its own.
//!
//! Two generated files sit beside this one: `baked.rs`, its pose table
//! (`cargo run -p anim --bin bake_beast -- --species galewing`), and
//! `tuned.rs`, its knobs (`cargo run -p sim --bin bake_tuning`).

pub(crate) mod baked;
pub mod fight;
pub mod flight;
pub mod mind;
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

/// Its bones, by index.
pub mod bones {
    pub const ROOT: usize = 0;
    pub const CHEST: usize = 1;
    pub const NECK: usize = 2;
    pub const HEAD: usize = 3;
    pub const TAIL: usize = 4;
    pub const SHOULDER_L: usize = 5;
    pub const ARM_L: usize = 6;
    pub const HAND_L: usize = 7;
    pub const SHOULDER_R: usize = 8;
    pub const ARM_R: usize = 9;
    pub const HAND_R: usize = 10;
    pub const THIGH_L: usize = 11;
    pub const SHIN_L: usize = 12;
    pub const THIGH_R: usize = 13;
    pub const SHIN_R: usize = 14;

    pub const COUNT: usize = 15;
}

use bones::*;

/// Where each bone sits in its parent's rest frame, in metres. Left is `-z`.
pub const BONES: [Bone; COUNT] = [
    bone("root", NO_PARENT, v((0, 1), (26, 10), (0, 1)), 0),
    bone("chest", ROOT, v((16, 10), (1, 10), (0, 1)), 0),
    bone("neck", CHEST, v((10, 10), (4, 10), (0, 1)), 0),
    bone("head", NECK, v((11, 10), (2, 10), (0, 1)), 0),
    bone("tail", ROOT, v((-20, 10), (1, 10), (0, 1)), 0),
    bone("shoulder.l", ROOT, v((2, 10), (5, 10), (-85, 100)), -1),
    bone("arm.l", SHOULDER_L, v((0, 1), (0, 1), (-25, 10)), -1),
    bone("hand.l", ARM_L, v((-2, 10), (0, 1), (-35, 10)), -1),
    bone("shoulder.r", ROOT, v((2, 10), (5, 10), (85, 100)), 1),
    bone("arm.r", SHOULDER_R, v((0, 1), (0, 1), (25, 10)), 1),
    bone("hand.r", ARM_R, v((-2, 10), (0, 1), (35, 10)), 1),
    bone("thigh.l", ROOT, v((-2, 10), (-5, 10), (-6, 10)), -1),
    bone("shin.l", THIGH_L, v((1, 10), (-10, 10), (0, 1)), -1),
    bone("thigh.r", ROOT, v((-2, 10), (-5, 10), (6, 10)), 1),
    bone("shin.r", THIGH_R, v((1, 10), (-10, 10), (0, 1)), 1),
];

/// The left and right bones a mirrored clip swaps.
pub const MIRROR: [(usize, usize); 5] = [
    (SHOULDER_L, SHOULDER_R),
    (ARM_L, ARM_R),
    (HAND_L, HAND_R),
    (THIGH_L, THIGH_R),
    (SHIN_L, SHIN_R),
];

/// The bones the head's tracking is spread across. The last is where it
/// sees from.
pub const NECK_CHAIN: [usize; 2] = [NECK, bones::HEAD];

/// What a move's volume rides: the body, the head, the left talons.
pub const FOLLOWS_BODY: u8 = 0;
pub const FOLLOWS_HEAD: u8 = 1;
pub const FOLLOWS_TALONS: u8 = 2;

pub const FOLLOWS: [usize; 3] = [ROOT, bones::HEAD, SHIN_L];

// ---------------------------------------------------------------------------
// Parts
// ---------------------------------------------------------------------------

pub const BACK: usize = 0;
pub const BREAST: usize = 1;
pub const NECK_PART: usize = 2;
pub const HEAD: usize = 3;
pub const TAIL_PART: usize = 4;
pub const ROOT_L: usize = 5;
pub const BLADE_L: usize = 6;
pub const TIP_L: usize = 7;
pub const ROOT_R: usize = 8;
pub const BLADE_R: usize = 9;
pub const TIP_R: usize = 10;
pub const LEG_L: usize = 11;
pub const TALON_L: usize = 12;
pub const LEG_R: usize = 13;
pub const TALON_R: usize = 14;

pub const PART_COUNT: usize = 15;

/// Which wing a part is, if it is one: `Some(0)` left, `Some(1)` right.
pub const fn wing_of(part: usize) -> Option<usize> {
    match part {
        ROOT_L | BLADE_L | TIP_L => Some(0),
        ROOT_R | BLADE_R | TIP_R => Some(1),
        _ => None,
    }
}

/// Is this a wing root -- the two places on the back where a hit counts
/// double?
pub const fn is_root(part: usize) -> bool {
    matches!(part, ROOT_L | ROOT_R)
}

/// Is this one of its legs -- what a carried fighter swings at?
pub const fn is_leg(part: usize) -> bool {
    matches!(part, LEG_L | TALON_L | LEG_R | TALON_R)
}

/// The spine: the parts of the back a braced rider rides the roll on.
pub const fn is_spine(part: usize) -> bool {
    matches!(part, BACK | BREAST)
}

/// Its proportions, at scale one.
pub const PARTS: [Part; PART_COUNT] = [
    // The back: the spine, from the tail root to the shoulders. The thing
    // you stand on, and the axis the roll turns about.
    part(
        "back",
        ROOT,
        v((-20, 10), (-8, 10), (-85, 100)),
        v((16, 10), (8, 10), (85, 100)),
        Knob::VulnBody as u16,
    )
    .mountable(),
    part(
        "breast",
        CHEST,
        v((-2, 10), (-8, 10), (-75, 100)),
        v((10, 10), (7, 10), (75, 100)),
        Knob::VulnBody as u16,
    )
    .mountable(),
    part(
        "neck",
        NECK,
        v((-1, 10), (-35, 100), (-35, 100)),
        v((12, 10), (35, 100), (35, 100)),
        Knob::VulnHead as u16,
    )
    .sheds(),
    part(
        "head",
        bones::HEAD,
        v((-2, 10), (-4, 10), (-4, 10)),
        v((10, 10), (4, 10), (4, 10)),
        Knob::VulnHead as u16,
    )
    .sheds(),
    // The tail fan: soft, so two metres of feathers are not a wall.
    part(
        "tail",
        TAIL,
        v((-22, 10), (-15, 100), (-10, 10)),
        v((0, 1), (15, 100), (10, 10)),
        Knob::VulnBody as u16,
    )
    .soft(),
    // The wing roots: three metres deep where they meet the body, flush with
    // the back. Where the ride's jackpot is.
    part(
        "left wing root",
        SHOULDER_L,
        v((-18, 10), (-25, 100), (-25, 10)),
        v((12, 10), (25, 100), (0, 1)),
        Knob::VulnRoot as u16,
    )
    .mountable(),
    part(
        "left wing",
        ARM_L,
        v((-15, 10), (-2, 10), (-35, 10)),
        v((9, 10), (2, 10), (0, 1)),
        Knob::VulnWing as u16,
    )
    .mountable(),
    part(
        "left wingtip",
        HAND_L,
        v((-11, 10), (-15, 100), (-22, 10)),
        v((4, 10), (15, 100), (0, 1)),
        Knob::VulnWing as u16,
    )
    .mountable(),
    part(
        "right wing root",
        SHOULDER_R,
        v((-18, 10), (-25, 100), (0, 1)),
        v((12, 10), (25, 100), (25, 10)),
        Knob::VulnRoot as u16,
    )
    .mountable(),
    part(
        "right wing",
        ARM_R,
        v((-15, 10), (-2, 10), (0, 1)),
        v((9, 10), (2, 10), (35, 10)),
        Knob::VulnWing as u16,
    )
    .mountable(),
    part(
        "right wingtip",
        HAND_R,
        v((-11, 10), (-15, 100), (0, 1)),
        v((4, 10), (15, 100), (22, 10)),
        Knob::VulnWing as u16,
    )
    .mountable(),
    // The legs and talons: soft, so a fighter held in them is not shoved
    // out of them, and what a carried fighter swings at.
    part(
        "left leg",
        THIGH_L,
        v((-3, 10), (-10, 10), (-25, 100)),
        v((3, 10), (1, 10), (25, 100)),
        Knob::VulnLeg as u16,
    )
    .soft(),
    part(
        "left talons",
        SHIN_L,
        v((-3, 10), (-11, 10), (-3, 10)),
        v((6, 10), (5, 100), (3, 10)),
        Knob::VulnLeg as u16,
    )
    .soft(),
    part(
        "right leg",
        THIGH_R,
        v((-3, 10), (-10, 10), (-25, 100)),
        v((3, 10), (1, 10), (25, 100)),
        Knob::VulnLeg as u16,
    )
    .soft(),
    part(
        "right talons",
        SHIN_R,
        v((-3, 10), (-11, 10), (-3, 10)),
        v((6, 10), (5, 100), (3, 10)),
        Knob::VulnLeg as u16,
    )
    .soft(),
];

/// Two legs, both "hind": what the shared lame layer walks (nothing here is
/// ever broken by the shared ladder -- the wings are its own).
pub const LEGS: [Leg; 2] = [
    Leg {
        upper: LEG_L,
        foot: TALON_L,
        hip: THIGH_L,
        knee: SHIN_L,
        front: false,
        side: -1,
    },
    Leg {
        upper: LEG_R,
        foot: TALON_R,
        hip: THIGH_R,
        knee: SHIN_R,
        front: false,
        side: 1,
    },
];

// ---------------------------------------------------------------------------
// Moves
// ---------------------------------------------------------------------------

/// Wings fold and it drops at thirty metres a second onto a point. The
/// circle on the floor is the shared telegraph: it is `lobbed`.
pub const STOOP: u8 = 0;
/// Down out of the circle and along a lane at head height, talons forward:
/// the first fighter standing in the lane is caught. Its own hit, tested
/// against the lane its signs draw.
pub const TALON: u8 = 1;
/// The catch: climbing with somebody in its talons. Never chosen: the talon
/// pass ends in it when it catches.
pub const CARRY: u8 = 2;
/// It hangs eight metres up and beats down: everybody in the ring is blown
/// outward unless a solid shades them. No damage.
pub const DOWNWASH: u8 = 3;
/// On its side, raking a lane with blade feathers.
pub const VOLLEY: u8 = 4;
/// On the ground: a cone of noise that stuns.
pub const SCREECH: u8 = 5;
/// On the ground: the wing on your side sweeps the floor. Jumped.
pub const BUFFET: u8 = 6;
/// Grounded for good, its gap-closer: a hop onto a circle, the Stoop's
/// answer.
pub const HOP: u8 = 7;
/// Breaks its circle, calls, and flies to the tower. Never chosen: low wind
/// starts it.
pub const PERCH: u8 = 8;
/// The gather and the lift: off the floor after a crash or a Stoop, off the
/// tower after a rest. Never chosen.
pub const LIFT: u8 = 9;
/// A full roll along its spine, with somebody aboard. Never chosen: the
/// ride's lap ends in it.
pub const ROLL: u8 = 10;

pub const MOVE_COUNT: usize = 11;

pub const MOVES: [MoveDecl; MOVE_COUNT] = [
    MoveDecl::new("Stoop", Clip::Stoop as usize).lobbed(),
    MoveDecl::new("Talon pass", Clip::Talon as usize).own_hit(),
    MoveDecl::new("Carry", Clip::Carry as usize)
        .own_hit()
        .never_chosen(),
    MoveDecl::new("Downwash", Clip::Downwash as usize)
        .own_hit()
        .harmless(),
    MoveDecl::new("Feather volley", Clip::Volley as usize).own_hit(),
    MoveDecl::new("Screech", Clip::Screech as usize).own_hit(),
    MoveDecl::new("Wing buffet", Clip::Buffet as usize)
        .own_hit()
        .mirrors_to_target_side(),
    MoveDecl::new("Hop", Clip::Hop as usize).lobbed(),
    MoveDecl::new("Perch", Clip::Perch as usize).never_chosen(),
    MoveDecl::new("Lift", Clip::Lift as usize).never_chosen(),
    MoveDecl::new("Barrel roll", Clip::Roll as usize).never_chosen(),
];

/// Moves thrown from the air: scored nothing on the ground.
pub const fn aerial(kind: u8) -> bool {
    matches!(kind, STOOP | TALON | DOWNWASH | VOLLEY)
}

/// Moves thrown on the ground: scored nothing aloft.
pub const fn grounded(kind: u8) -> bool {
    matches!(kind, SCREECH | BUFFET | HOP)
}

// ---------------------------------------------------------------------------
// Clips
// ---------------------------------------------------------------------------

/// Everything it knows how to look like, in the order its baked table lays
/// them out. Authored in `crates/anim/src/beast/galewing/`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Clip {
    /// Standing, wings folded along its back.
    Idle,
    Walk,
    /// The grounded bird's run: half hop, wings half open.
    Run,
    /// One wingbeat, looping: the upstroke and the downstroke.
    Fly,
    /// Wings held spread: circling.
    Glide,
    Stoop,
    Talon,
    Carry,
    Downwash,
    Volley,
    Screech,
    Buffet,
    Hop,
    Perch,
    Lift,
    Roll,
    Flinch,
    Stumble,
    /// **On its breast, wings flat on the floor**: the crash, the big window.
    Crash,
    Dead,
}

pub const CLIP_COUNT: usize = 20;

impl Clip {
    pub const ALL: [Clip; CLIP_COUNT] = [
        Clip::Idle,
        Clip::Walk,
        Clip::Run,
        Clip::Fly,
        Clip::Glide,
        Clip::Stoop,
        Clip::Talon,
        Clip::Carry,
        Clip::Downwash,
        Clip::Volley,
        Clip::Screech,
        Clip::Buffet,
        Clip::Hop,
        Clip::Perch,
        Clip::Lift,
        Clip::Roll,
        Clip::Flinch,
        Clip::Stumble,
        Clip::Crash,
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
    cycle("run"),
    cycle("fly"),
    cycle("glide"),
    attack("stoop"),
    attack("talon"),
    attack("carry"),
    attack("downwash"),
    attack("volley"),
    attack("screech"),
    attack("buffet"),
    attack("hop"),
    attack("perch"),
    attack("lift"),
    attack("roll"),
    once("flinch"),
    once("stumble"),
    once("crash"),
    once("dead"),
];

// ---------------------------------------------------------------------------
// Its own knobs
// ---------------------------------------------------------------------------

crate::species_knobs! {
    /// The knobs only the Galewing has: its hide, its wings, its flight, its
    /// moves' shapes, its wind and perch, the ride, and the coop numbers.
    /// The design document's names are in the comment beside each where
    /// they differ.
    pub enum Knob;
    VulnBody,        "hide",    "Body (x damage)",                         Fixed, 0, fx(3,1);
    VulnHead,        "hide",    "Head and neck (x damage)",                Fixed, 0, fx(3,1);
    VulnRoot,        "hide",    "Wing roots (x damage)",                   Fixed, 0, fx(4,1);
    VulnWing,        "hide",    "Wings (x damage)",                        Fixed, 0, fx(3,1);
    VulnLeg,         "hide",    "Legs (x damage)",                         Fixed, 0, fx(3,1);
    // the crash's x1.3
    CrashHide,       "hide",    "Crashed, everything (x damage)",          Fixed, 0, fx(3,1);
    WingBar,         "wings",   "Each wing's break bar",                   Int, 100, 6000;
    // wing poise is the common Poise; this is the height it fills below
    LowBelow,        "wings",   "Wing hits fill poise below (m)",          Fixed, 0, fx(20,1);
    // one wing broken
    BrokenCeiling,   "wings",   "One wing broken, climbs no higher (m)",   Fixed, 0, fx(30,1);
    BrokenClimb,     "wings",   "One wing broken, climbs at (x)",          Fixed, 0, fx(1,1);
    BrokenTurn,      "wings",   "One wing broken, turns toward it at (x)", Fixed, 0, fx(1,1);
    BrokenList,      "wings",   "One wing broken, lists (turns)",          Fixed, 0, fx(1,8);
    // flight
    CruiseAlt,       "flight",  "Circles at (m)",                          Fixed, 0, fx(30,1);
    CircleRadius,    "flight",  "Circle radius (m)",                       Fixed, fx(4,1), fx(30,1);
    CircleLead,      "flight",  "Chases the circle this far ahead (turns)", Fixed, 0, fx(1,2);
    CruiseSpeed,     "flight",  "Cruises at (m/s)",                        Fixed, 0, fx(40,1);
    PassSpeed,       "flight",  "Passes low at (m/s)",                     Fixed, 0, fx(40,1);
    StoopSpeed,      "flight",  "Stoops at (m/s)",                         Fixed, 0, fx(60,1);
    ClimbRate,       "flight",  "Climbs at (m/s)",                         Fixed, 0, fx(30,1);
    SinkRate,        "flight",  "Sinks at (m/s)",                          Fixed, 0, fx(40,1);
    AirAccel,        "flight",  "Air speed, gains (m/s2)",                 Fixed, 0, fx(100,1);
    AirTurn,         "flight",  "Turn rate, banked fully (turns/s)",       Fixed, 0, fx(2,1);
    AirTurnAccel,    "flight",  "Turn acceleration (turns/s2)",            Fixed, 0, fx(10,1);
    BankMax,         "flight",  "Bank, at most (turns)",                   Fixed, 0, fx(1,4);
    VertAccel,       "flight",  "Climb or sink, gains (m/s2)",             Fixed, 0, fx(200,1);
    AirTurnGain,     "flight",  "Turns toward its mark at (x the error, /s)", Fixed, 0, fx(10,1);
    VertGain,        "flight",  "Climbs toward its height at (x the gap, /s)", Fixed, 0, fx(10,1);
    ArriveTurn,      "flight",  "Arriving somewhere, swings its heading at (turns/s)", Fixed, 0, fx(4,1);
    LiftClimb,       "flight",  "Taking off or carrying, climbs at (m/s)", Fixed, 0, fx(30,1);
    PerchOver,       "perch",   "Comes in over the perch this high (m)",  Fixed, 0, fx(20,1);
    PerchNear,       "perch",   "Drops onto the perch from within (m)",   Fixed, 0, fx(30,1);
    PerchRing,       "perch",   "Its ring on the tower top, across (m)",  Fixed, 0, fx(20,1);
    LeeDisc,         "downwash","Each lee drawn this wide (m)",           Fixed, 0, fx(10,1);
    // the brain
    LineUpArc,       "mind",    "Decides only with the target this near its heading (turns)", Fixed, 0, fx(1,2);
    FollowAppetite,  "mind",    "A follow-up's appetite",                  Int, 0, 8000;
    LeeAppetite,     "mind",    "Downwash, appetite per exposed target",   Int, 0, 8000;
    EdgeNear,        "mind",    "Downwash, a target this near an edge is exposed (m)", Fixed, 0, fx(20,1);
    HurtAppetite,    "mind",    "Below desperation, low moves more (x)",   Fixed, 0, fx(4,1);
    DesperateHealth, "mind",    "Desperate below health (x)",              Fixed, 0, fx(1,1);
    DesperateAlt,    "mind",    "Desperate, circles at (m)",               Fixed, 0, fx(30,1);
    LastHealth,      "mind",    "Stoops twice below health (x)",           Fixed, 0, fx(1,1);
    // the moves' shapes
    StoopLock,       "stoop",   "Its aim locks this many frames before",   Frames, 0, 60;
    StoopTrack,      "stoop",   "Its aim follows you at (m/s)",            Fixed, 0, fx(30,1);
    StoopLift,       "stoop",   "Recovery's last frames are the lift",     Frames, 0, 120;
    TalonHeight,     "talons",  "Talons clear (m)",                        Fixed, 0, fx(3,1);
    LaneLength,      "talons",  "Lane length (m)",                         Fixed, 0, fx(40,1);
    LaneWidth,       "talons",  "Lane width (m)",                          Fixed, 0, fx(8,1);
    LaneLead,        "talons",  "The target stands this far into the lane (m)", Fixed, 0, fx(20,1);
    PassHeight,      "talons",  "Body passes at (m)",                      Fixed, 0, fx(8,1);
    LaneLock,        "talons",  "The lane locks this many frames before",  Frames, 0, 120;
    LaneTrack,       "talons",  "Until then, the lane follows you at (m/s)", Fixed, 0, fx(30,1);
    CarryHeight,     "carry",   "Climbs to (m)",                           Fixed, 0, fx(30,1);
    CarryFreeDamage, "carry",   "Freed by leg damage",                     Int, 1, 2000;
    CarryFreeHits,   "carry",   "Freed by leg hits",                       Int, 1, 9;
    CarryGrab,       "carry",   "The grab's damage",                       Int, 0, 400;
    HoverHeight,     "downwash","Hangs at (m)",                            Fixed, 0, fx(20,1);
    WashRadius,      "downwash","Ring radius (m)",                         Fixed, 0, fx(30,1);
    WashEye,         "downwash","The still eye (m)",                       Fixed, 0, fx(8,1);
    WashPush,        "downwash","Pushes at (m/s)",                         Fixed, 0, fx(30,1);
    WashCrouch,      "downwash","Crouched, pushed at (x)",                 Fixed, 0, fx(1,1);
    WashBeside,      "downwash","Hangs this far beside its target (m)",    Fixed, 0, fx(20,1);
    VolleyLength,    "volley",  "Lane length (m)",                         Fixed, 0, fx(40,1);
    VolleyWidth,     "volley",  "Lane width (m)",                          Fixed, 0, fx(8,1);
    VolleyUnder,     "volley",  "Each point under feathers for",           Frames, 1, 120;
    VolleyEvery,     "volley",  "A feather every, on one fighter",         Frames, 1, 60;
    VolleyMost,      "volley",  "Feathers, at most",                       Int, 1, 9;
    VolleyHeight,    "volley",  "Rakes from (m)",                          Fixed, 0, fx(20,1);
    ScreechLength,   "screech", "Cone length (m)",                         Fixed, 0, fx(20,1);
    ScreechHalf,     "screech", "Cone half-angle (turns)",                 Fixed, 0, fx(1,4);
    BuffetReach,     "buffet",  "Sweeps out to (m)",                       Fixed, 0, fx(12,1);
    BuffetHigh,      "buffet",  "Sweeps up to (m)",                        Fixed, 0, fx(4,1);
    // wind and the perch
    WindMax,         "wind",    "Wind, full",                              Int, 1, 1000;
    WindStoop,       "wind",    "Stoop costs",                             Int, 0, 1000;
    WindTalon,       "wind",    "Talon pass costs",                        Int, 0, 1000;
    WindWash,        "wind",    "Downwash costs",                          Int, 0, 1000;
    WindVolley,      "wind",    "Volley costs",                            Int, 0, 1000;
    WindRegen,       "wind",    "Regained per second circling",            Fixed, 0, fx(50,1);
    WindRegenDesperate, "wind", "Desperate, regained per second",          Fixed, 0, fx(50,1);
    PerchWind,       "wind",    "Perches below",                           Int, 0, 1000;
    PerchRest,       "perch",   "Rests for",                               Frames, 0, 1200;
    PerchRestDesperate, "perch","Desperate, rests for",                    Frames, 0, 1200;
    // perch_rest_per_damage: frames taken off the rest per point of damage
    PerchShorten,    "perch",   "A hit takes off the rest (frames per damage)", Fixed, 0, fx(2,1);
    // the crash
    CrashFall,       "crash",   "Falls at, crashing (m/s)",                Fixed, 0, fx(40,1);
    GlideFrames,     "crash",   "Clipped, glides low for",                 Frames, 0, 300;
    GlideHeight,     "crash",   "Clipped, glides at (m)",                  Fixed, 0, fx(10,1);
    // the ride
    RideClimb,       "ride",    "With a rider, climbs to (m)",             Fixed, 0, fx(40,1);
    RideStep,        "ride",    "Each lap, higher by (m)",                 Fixed, 0, fx(10,1);
    RideSpeed,       "ride",    "Flies a lap at (m/s)",                    Fixed, 0, fx(30,1);
    RideLap,         "ride",    "A lap lasts",                             Frames, 60, 1200;
    SwoopHeight,     "ride",    "Swoops to (m)",                           Fixed, 0, fx(10,1);
    SwoopFrames,     "ride",    "Low over the far side for",               Frames, 0, 240;
    // a wingbeat is one breath of its clock: `BreathRate`
    BeatDown,        "ride",    "Its downstroke lasts",                    Frames, 1, 60;
    BeatKick,        "ride",    "A downstroke kicks the back up at (m/s)", Fixed, 0, fx(30,1);
    RollTurns,       "ride",    "The roll turns, in its active frames",    Fixed, 0, fx(2,1);
    // crash_ride_share
    CrashRideShare,  "ride",    "Riding a broken wing down, the fall's share (x)", Fixed, 0, fx(1,1);
    BrokenDrop,      "ride",    "A wing broken aloft drops it at (m/s)",   Fixed, 0, fx(40,1);
    CoopHealth,      "coop",    "Two hunters, health",                     Int, 0, 30000;
    CoopTalon,       "coop",    "Two hunters, talon lockout",              Frames, 0, 900;
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
}

// ---------------------------------------------------------------------------
// The species
// ---------------------------------------------------------------------------

pub static SPECIES: Species = Species {
    id: SpeciesId::GALEWING,
    name: "Galewing",
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
        gallop: Clip::Run as usize,
        flinch: Clip::Flinch as usize,
        stumble: Clip::Stumble as usize,
        topple: Clip::Crash as usize,
        dead: Clip::Dead as usize,
    },
    span: &baked::SPAN,
    rows: baked::ROWS,
    row: |r| &baked::FRAMES[r],
    own: OWN,
    tuned: &tuned::KNOBS,
    tuned_path: "crates/sim/src/species/galewing/tuned.rs",
    pack: None,
    fight: &fight::FIGHT,
};
