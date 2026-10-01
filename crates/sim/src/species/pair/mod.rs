//! The Pair: two big cats, bonded, that hunt as one animal with two bodies.
//! See `docs/design/creatures/the-pair.md` for what the fight is and why; this
//! file is what one cat *is* -- its skeleton, parts, moves, clips and its own
//! knobs. What the two of them do together, and to the fighters, is
//! [`fight`]; what each wants is [`mind`].
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
//! **The Ridgeback's eighteen bones, re-proportioned** (the test of
//! `docs/design/monsters.md` §6, "the rig is data"): the same parents and the
//! same sides, new rest offsets, new boxes, new legs and its own clips. A cat
//! is 1.8 m at the shoulder and 4 m nose to tail, with its head at a
//! fighter's chest; nothing about the shared machinery had to change for it
//! to stand, walk, collide and be hit -- what did change, the `sheds` flag
//! that keeps everybody off its back, is a part flag every creature has.
//!
//! **The origin is the middle of the body**, not the hips: the root sits
//! forty centimetres behind it, so a move's range and the brain's distances
//! are measured from the middle of the animal, as the Ridgeback's are.
//!
//! Two generated files sit beside this one: `baked.rs`, its pose table
//! (`cargo run -p anim --bin bake_beast -- --species pair`), and `tuned.rs`,
//! its knobs (`cargo run -p sim --bin bake_tuning`).

pub(crate) mod baked;
pub mod fight;
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

/// The cat's bones, by index: the Ridgeback's, in the Ridgeback's order.
pub mod bones {
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

    pub const COUNT: usize = 18;
}

use bones::*;

/// Where each bone sits in its parent's rest frame, in metres.
///
/// **1.8 m at the shoulder, the head at 1.5 m, 4 m nose to tail**
/// (`the-pair.md` §1). The height is in the body this time rather than the
/// legs: the back is a fighter's head height and every class's hop reaches
/// it, which is why nothing on it is a floor (`sheds`). The legs are a metre
/// long, the body eighty centimetres deep, the tail a metre and a half.
pub const BONES: [Bone; COUNT] = [
    bone("root", NO_PARENT, v((-40, 100), (120, 100), (0, 1)), 0), // the hips, 1.2 m up
    bone("spine", ROOT, v((55, 100), (5, 100), (0, 1)), 0),
    bone("chest", SPINE, v((55, 100), (8, 100), (0, 1)), 0),
    bone("neck", CHEST, v((30, 100), (12, 100), (0, 1)), 0),
    bone("neck2", NECK, v((22, 100), (6, 100), (0, 1)), 0),
    bone("head", NECK2, v((20, 100), (-1, 100), (0, 1)), 0), // the head, 1.5 m up
    bone("tail1", ROOT, v((-35, 100), (6, 100), (0, 1)), 0),
    bone("tail2", TAIL1, v((-40, 100), (-4, 100), (0, 1)), 0),
    bone("tail3", TAIL2, v((-40, 100), (-10, 100), (0, 1)), 0),
    bone("tail4", TAIL3, v((-35, 100), (-10, 100), (0, 1)), 0),
    bone("shoulder.l", CHEST, v((-5, 100), (-30, 100), (-26, 100)), -1),
    bone("forearm.l", SHOULDER_L, v((0, 1), (-52, 100), (0, 1)), -1),
    bone("shoulder.r", CHEST, v((-5, 100), (-30, 100), (26, 100)), 1),
    bone("forearm.r", SHOULDER_R, v((0, 1), (-52, 100), (0, 1)), 1),
    bone("thigh.l", ROOT, v((-12, 100), (-28, 100), (-28, 100)), -1),
    bone("shin.l", THIGH_L, v((8, 100), (-46, 100), (0, 1)), -1),
    bone("thigh.r", ROOT, v((-12, 100), (-28, 100), (28, 100)), 1),
    bone("shin.r", THIGH_R, v((8, 100), (-46, 100), (0, 1)), 1),
];

/// The left and right bones a mirrored clip swaps: the Ridgeback's.
pub const MIRROR: [(usize, usize); 4] = [
    (SHOULDER_L, SHOULDER_R),
    (FOREARM_L, FOREARM_R),
    (THIGH_L, THIGH_R),
    (SHIN_L, SHIN_R),
];

/// The bones the head's tracking is spread across, base to tip. The last is
/// where it sees from.
pub const NECK_CHAIN: [usize; 3] = [NECK, NECK2, bones::HEAD];

/// Which bone an attack's hit volume rides: the body, the head, the tail.
pub const FOLLOWS_BODY: u8 = 0;
pub const FOLLOWS_HEAD: u8 = 1;
pub const FOLLOWS_TAIL: u8 = 2;

pub const FOLLOWS: [usize; 3] = [ROOT, bones::HEAD, TAIL4];

// ---------------------------------------------------------------------------
// Parts
// ---------------------------------------------------------------------------

pub const HEAD: usize = 0;
pub const NECK_PART: usize = 1;
pub const SHOULDERS: usize = 2;
pub const BARREL: usize = 3;
pub const HAUNCH: usize = 4;
pub const TAIL_BASE: usize = 5;
pub const TAIL_MID: usize = 6;
pub const TAIL_TIP: usize = 7;
pub const FORELEG_L: usize = 8;
pub const FORELEG_R: usize = 9;
pub const HINDLEG_L: usize = 10;
pub const HINDLEG_R: usize = 11;
pub const FOREFOOT_L: usize = 12;
pub const FOREFOOT_R: usize = 13;
pub const HINDFOOT_L: usize = 14;
pub const HINDFOOT_R: usize = 15;

pub const PART_COUNT: usize = 16;

/// A part of the body: solid, and **nobody stands on it** -- the back is a hop
/// high, and a ride reachable by a hop is the Ridgeback's old tail (§3).
const fn body(name: &'static str, on: usize, min: crate::math::V3, max: crate::math::V3) -> Part {
    part(name, on, min, max, Knob::VulnBody as u16).sheds()
}

/// The cat's proportions, at scale one.
pub const PARTS: [Part; PART_COUNT] = [
    // **The head**: 1.3x, and it sits in the Holder's face. Damage into it
    // scars an eye (`fight::scar`).
    part(
        "head",
        bones::HEAD,
        v((-12, 100), (-20, 100), (-19, 100)),
        v((42, 100), (17, 100), (19, 100)),
        Knob::VulnHead as u16,
    )
    .sheds(),
    body(
        "neck",
        NECK2,
        v((-28, 100), (-20, 100), (-17, 100)),
        v((12, 100), (18, 100), (17, 100)),
    ),
    // The shoulders: the top of the scapulae, 1.8 m up.
    body(
        "shoulders",
        CHEST,
        v((-30, 100), (-40, 100), (-30, 100)),
        v((30, 100), (47, 100), (30, 100)),
    ),
    body(
        "barrel",
        SPINE,
        v((-40, 100), (-32, 100), (-31, 100)),
        v((40, 100), (30, 100), (31, 100)),
    ),
    body(
        "haunch",
        ROOT,
        v((-40, 100), (-30, 100), (-31, 100)),
        v((20, 100), (34, 100), (31, 100)),
    ),
    // The tail is **soft**: the trip's sweep is its move's volume, not the
    // tail's box, and a tail that shoved whoever it passed would take away the
    // jump that answers it.
    part(
        "tail",
        TAIL1,
        v((-42, 100), (-8, 100), (-8, 100)),
        v((4, 100), (8, 100), (8, 100)),
        Knob::VulnTail as u16,
    )
    .soft(),
    part(
        "tail, middle",
        TAIL2,
        v((-42, 100), (-7, 100), (-7, 100)),
        v((4, 100), (7, 100), (7, 100)),
        Knob::VulnTail as u16,
    )
    .soft(),
    part(
        "tail tip",
        TAIL3,
        v((-78, 100), (-7, 100), (-7, 100)),
        v((4, 100), (7, 100), (7, 100)),
        Knob::VulnTail as u16,
    )
    .soft(),
    part(
        "left foreleg",
        SHOULDER_L,
        v((-13, 100), (-52, 100), (-11, 100)),
        v((13, 100), (15, 100), (11, 100)),
        Knob::VulnLeg as u16,
    )
    .sheds(),
    part(
        "right foreleg",
        SHOULDER_R,
        v((-13, 100), (-52, 100), (-11, 100)),
        v((13, 100), (15, 100), (11, 100)),
        Knob::VulnLeg as u16,
    )
    .sheds(),
    part(
        "left hindleg",
        THIGH_L,
        v((-15, 100), (-46, 100), (-12, 100)),
        v((15, 100), (18, 100), (12, 100)),
        Knob::VulnLeg as u16,
    )
    .sheds(),
    part(
        "right hindleg",
        THIGH_R,
        v((-15, 100), (-46, 100), (-12, 100)),
        v((15, 100), (18, 100), (12, 100)),
        Knob::VulnLeg as u16,
    )
    .sheds(),
    // The paws. Not breakable: the cats' lasting consequence is the eye.
    part(
        "left forepaw",
        FOREARM_L,
        v((-10, 100), (-51, 100), (-10, 100)),
        v((16, 100), (8, 100), (10, 100)),
        Knob::VulnLeg as u16,
    )
    .sheds(),
    part(
        "right forepaw",
        FOREARM_R,
        v((-10, 100), (-51, 100), (-10, 100)),
        v((16, 100), (8, 100), (10, 100)),
        Knob::VulnLeg as u16,
    )
    .sheds(),
    part(
        "left hindpaw",
        SHIN_L,
        v((-10, 100), (-46, 100), (-10, 100)),
        v((16, 100), (8, 100), (10, 100)),
        Knob::VulnLeg as u16,
    )
    .sheds(),
    part(
        "right hindpaw",
        SHIN_R,
        v((-10, 100), (-46, 100), (-10, 100)),
        v((16, 100), (8, 100), (10, 100)),
        Knob::VulnLeg as u16,
    )
    .sheds(),
];

/// The four legs, front to back: the Ridgeback's.
pub const LEGS: [Leg; 4] = [
    Leg {
        upper: FORELEG_L,
        foot: FOREFOOT_L,
        hip: SHOULDER_L,
        knee: FOREARM_L,
        front: true,
        side: -1,
    },
    Leg {
        upper: FORELEG_R,
        foot: FOREFOOT_R,
        hip: SHOULDER_R,
        knee: FOREARM_R,
        front: true,
        side: 1,
    },
    Leg {
        upper: HINDLEG_L,
        foot: HINDFOOT_L,
        hip: THIGH_L,
        knee: SHIN_L,
        front: false,
        side: -1,
    },
    Leg {
        upper: HINDLEG_R,
        foot: HINDFOOT_R,
        hip: THIGH_R,
        knee: SHIN_R,
        front: false,
        side: 1,
    },
];

// ---------------------------------------------------------------------------
// Moves
// ---------------------------------------------------------------------------

/// Haunches drop, the tail flicks at frame ten, and it leaps at where you are
/// going; lands forepaws first and skids on, claws out. Dodged toward.
pub const POUNCE: u8 = 0;
/// A forepaw swipe, and the paw stays cocked.
pub const RAKE: u8 = 1;
/// **The paw held up**, for a beat drawn as the second hit commits. Never
/// chosen: the rake's swipe ends in it.
pub const COCK: u8 = 2;
/// The cocked paw coming down. Never chosen: the hold ends in it.
pub const RAKE2: u8 = 3;
/// A short cuff off the shoulder, below reaction: the face is the Holder's.
pub const SWAT: u8 = 4;
/// The Holder's coil with no flick, and no leap. Do nothing.
pub const FEINT: u8 = 5;
/// The Striker's lunge from behind, flat along a lane.
pub const AMBUSH: u8 = 6;
/// The tail swept low round the hips, to the side you are on. Jump it.
pub const TRIP: u8 = 7;
/// Up onto a wall top, a platform, a stone.
pub const PERCH: u8 = 8;
/// Down from the lip onto where you are standing.
pub const DIVE: u8 = 9;
/// Down from a perch the slow way, because you are under its lip. Never
/// chosen by the brain's scoring: the perch's patience runs out.
pub const DROP: u8 = 10;
/// Both cats at the same predicted point, from opposite sides. Never chosen
/// by either cat: the pair brain throws it with both at once.
pub const TWIN: u8 = 11;
/// The healthy cat runs onto the line between you and its wounded mate.
pub const INTERPOSE: u8 = 12;
/// Over the first body. Never chosen: a death starts it.
pub const HOWL: u8 = 13;

pub const MOVE_COUNT: usize = 14;

pub const MOVES: [MoveDecl; MOVE_COUNT] = [
    MoveDecl::new("Pounce", Clip::Pounce as usize).lobbed(),
    MoveDecl::new("Rake", Clip::Rake as usize),
    MoveDecl::new("Cocked paw", Clip::Cock as usize).never_chosen(),
    MoveDecl::new("Rake, again", Clip::Rake2 as usize).never_chosen(),
    MoveDecl::new("Swat", Clip::Swat as usize),
    MoveDecl::new("Feint", Clip::Feint as usize).never_chosen(),
    // Its lane runs past where it is aimed: from behind you, under you, and
    // on into your screen (`the-pair.md` §6).
    MoveDecl::new("Ambush", Clip::Ambush as usize).stops_at_aim(),
    MoveDecl::new("Tail trip", Clip::Trip as usize).mirrors_to_target_side(),
    MoveDecl::new("Perch", Clip::Perch as usize),
    MoveDecl::new("Dive", Clip::Dive as usize).lobbed(),
    MoveDecl::new("Drop", Clip::Drop as usize).never_chosen(),
    MoveDecl::new("Twin pounce", Clip::Twin as usize)
        .lobbed()
        .never_chosen(),
    MoveDecl::new("Interpose", Clip::Interpose as usize).stops_at_aim(),
    MoveDecl::new("Howl", Clip::Howl as usize).never_chosen(),
];

/// The moves that leave the ground: flown by the frame hook from where the cat
/// stood to its aim point (`fight::fly`).
pub const fn leaps(kind: u8) -> bool {
    matches!(kind, POUNCE | PERCH | DIVE | DROP | TWIN)
}

// ---------------------------------------------------------------------------
// Clips
// ---------------------------------------------------------------------------

/// Everything a cat knows how to look like, in the order its baked table lays
/// them out. Authored in `crates/anim/src/beast/pair/`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Clip {
    Idle,
    Walk,
    /// The bound: a cat at speed, gathering and throwing like the Ridgeback's
    /// gallop, with more spine in it.
    Bound,
    Pounce,
    Rake,
    Cock,
    Rake2,
    Swat,
    Feint,
    Ambush,
    Trip,
    Perch,
    Dive,
    Drop,
    Twin,
    Interpose,
    Howl,
    Flinch,
    /// Down on its chest: tripped by crowd control.
    Stumble,
    /// **On its side**: the twin pounce's crash, the big window.
    Topple,
    Dead,
}

pub const CLIP_COUNT: usize = 21;

impl Clip {
    pub const ALL: [Clip; CLIP_COUNT] = [
        Clip::Idle,
        Clip::Walk,
        Clip::Bound,
        Clip::Pounce,
        Clip::Rake,
        Clip::Cock,
        Clip::Rake2,
        Clip::Swat,
        Clip::Feint,
        Clip::Ambush,
        Clip::Trip,
        Clip::Perch,
        Clip::Dive,
        Clip::Drop,
        Clip::Twin,
        Clip::Interpose,
        Clip::Howl,
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
    cycle("bound"),
    attack("pounce"),
    attack("rake"),
    attack("cock"),
    attack("rake2"),
    attack("swat"),
    attack("feint"),
    attack("ambush"),
    attack("trip"),
    attack("perch"),
    attack("dive"),
    attack("drop"),
    attack("twin"),
    attack("interpose"),
    attack("howl"),
    once("flinch"),
    once("stumble"),
    once("topple"),
    once("dead"),
];

// ---------------------------------------------------------------------------
// Its own knobs
// ---------------------------------------------------------------------------

crate::species_knobs! {
    /// The knobs only the Pair have: the hide, what the pair brain does with
    /// two bodies, the leaps, the scar, the bond and the enrage, and the coop
    /// numbers. The design document's names (§5) are in the comment beside
    /// each, where they differ.
    pub enum Knob;
    VulnHead,        "hide",  "Head (x damage)",                         Fixed, 0, fx(3,1);
    VulnBody,        "hide",  "Body (x damage)",                         Fixed, 0, fx(3,1);
    VulnLeg,         "hide",  "Legs (x damage)",                         Fixed, 0, fx(3,1);
    VulnTail,        "hide",  "Tail (x damage)",                         Fixed, 0, fx(3,1);
    // hold_distance, strike_distance
    HoldDistance,    "pair",  "Holder stands off at",                    Fixed, 0, fx(20,1);
    StrikeDistance,  "pair",  "Striker circles at",                      Fixed, 0, fx(20,1);
    // swap_frames
    SwapMin,         "pair",  "Roles swap, at the soonest",              Frames, 1, 1200;
    SwapMax,         "pair",  "Roles swap, at the latest",               Frames, 1, 1200;
    StrainedSwapMin, "pair",  "Strained, roles swap at the soonest",     Frames, 1, 1200;
    StrainedSwapMax, "pair",  "Strained, roles swap at the latest",      Frames, 1, 1200;
    // swap_frames_crossing
    SwapCrossing,    "pair",  "The swap, frames neither attacks",        Frames, 0, 240;
    // bond_leash
    BondLeash,       "pair",  "Further apart than this, they regroup",   Fixed, 0, fx(40,1);
    // stagger_gap
    StaggerGap,      "pair",  "Two hits never land closer than",         Frames, 0, 120;
    // pincer_angle
    PincerAngle,     "pair",  "Pincer: apart round the target (turns)",  Fixed, 0, fx(1,2);
    BehindAngle,     "pair",  "Striker behind: round the target (turns)",Fixed, 0, fx(1,2);
    FeintAngle,      "pair",  "Feints with its mate this far round (turns)",Fixed, 0, fx(1,2);
    StrainedHealth,  "pair",  "Strained below health (x)",               Fixed, 0, fx(1,1);
    // feint_share
    FeintShare,      "pair",  "A coil is a feint, fresh (%)",            Percent, 0, 100;
    FeintStrained,   "pair",  "A coil is a feint, strained (%)",         Percent, 0, 100;
    // the role term
    RoleAppetite,    "pair",  "A role's own moves, more appetite",       Int, 0, 8000;
    TwinCooldown,    "pair",  "Twin pounce, at most once in",            Frames, 0, 1800;
    TwinNear,        "pair",  "Twin pounce, each cat no nearer than",    Fixed, 0, fx(20,1);
    TwinFar,         "pair",  "Twin pounce, each cat no further than",   Fixed, 0, fx(20,1);
    CrashFrames,     "pair",  "Twin pounce crashed, dazed for",          Frames, 0, 300;
    CrashReach,      "pair",  "Twin pounce crashes cats this close",     Fixed, 0, fx(10,1);
    // seen_fast_speed, fast_appetite
    SeenFastSpeed,   "mind",  "A sample this fast was a dodge",          Fixed, 0, fx(30,1);
    AmbushPast,      "mind",  "Ambush, its lane runs past you by",       Fixed, 0, fx(10,1);
    StuckAfter,      "mind",  "Getting nowhere this long, it goes round", Frames, 0, 240;
    StuckRound,      "mind",  "Goes round for",                          Frames, 0, 240;
    FastAppetite,    "mind",  "Ambush, at a fast sample",                Int, 0, 8000;
    SearchAfter,     "mind",  "Lost you, searches round where it saw you after", Frames, 0, 600;
    SearchTurn,      "mind",  "Searching, circles at (turns/s)",          Fixed, 0, fx(1,1);
    // perch_reach
    PerchReach,      "perch", "Perches on a top within",                 Fixed, 0, fx(20,1);
    PerchFar,        "perch", "Perches when the target is beyond",       Fixed, 0, fx(20,1);
    PerchWidth,      "perch", "A top this wide is somewhere to perch",   Fixed, 0, fx(5,1);
    PerchHighest,    "perch", "A top no higher than this",               Fixed, 0, fx(10,1);
    PerchAppetite,   "perch", "Perch, appetite",                         Int, 0, 8000;
    PerchPatience,   "perch", "Perched, drops down after",               Frames, 0, 1200;
    // the lip
    DiveLip,         "perch", "Dive, cannot reach inside this of the lip", Fixed, 0, fx(10,1);
    DiveAppetite,    "perch", "Dive, appetite from a perch",             Int, 0, 8000;
    // the leaps
    PounceLeave,     "leap",  "Pounce leaves the ground at startup frame", Frames, 0, 120;
    PounceTrack,     "leap",  "Pounce's mark follows you until startup frame", Frames, 0, 120;
    TwinTrack,       "leap",  "Twin's mark follows you until startup frame", Frames, 0, 120;
    DiveTrack,       "leap",  "Dive's mark follows you until startup frame", Frames, 0, 120;
    PounceLand,      "leap",  "Pounce lands at active frame",            Frames, 0, 60;
    DiveLeave,       "leap",  "Dive leaves the lip at startup frame",    Frames, 0, 120;
    TwinLeave,       "leap",  "Twin leaves the ground at startup frame", Frames, 0, 120;
    TwinLand,        "leap",  "Twin lands at active frame",              Frames, 0, 60;
    DiveLand,        "leap",  "Dive lands at active frame",              Frames, 0, 60;
    PerchLand,       "leap",  "Perch lands at active frame",             Frames, 0, 60;
    DropLand,        "leap",  "Drop lands at active frame",              Frames, 0, 60;
    LandShort,       "leap",  "A leap's body lands this short of its mark", Fixed, 0, fx(3,1);
    PerchArc,        "leap",  "The perch's leap clears its top by",      Fixed, 0, fx(3,1);
    DiveArc,         "leap",  "The dive's leap rises by",                Fixed, 0, fx(3,1);
    FallSpeed,       "leap",  "Off an edge, comes down at (m/s)",        Fixed, 0, fx(40,1);
    // the rake's hold
    HoldMin,         "rake",  "The paw stays up at least",               Frames, 0, 120;
    HoldMax,         "rake",  "The paw stays up at most",                Frames, 0, 120;
    // the scar
    ScarDamage,      "scar",  "Into one head, scars an eye",             Int, 0, 4000;
    ScarSeek,        "scar",  "Scarred, turns to find you after",        Frames, 0, 240;
    // the bond
    BondHealth,      "bond",  "Bonded, the wounded below health (x)",    Fixed, 0, fx(1,1);
    InterposeNear,   "bond",  "Interposes, you within this of its mate", Fixed, 0, fx(30,1);
    InterposeAppetite,"bond", "Interpose, appetite",                     Int, 0, 8000;
    InterposeFrom,   "bond",  "Interposes this far out from its mate",   Fixed, 0, fx(10,1);
    WoundedHang,     "bond",  "The wounded hangs back at",               Fixed, 0, fx(20,1);
    // the enrage
    EnragePace,      "enrage","Enraged, runs at (x)",                    Fixed, fx(1,1), fx(3,1);
    EnrageRecovery,  "enrage","Enraged, recoveries at (x)",              Fixed, 0, fx(1,1);
    EnrageThink,     "enrage","Enraged, the beat between moves (x)",     Fixed, 0, fx(1,1);
    ChainTell,       "enrage","Enraged, a missed pounce chains with a tell of", Frames, 0, 60;
    // coop
    CoopHealth,      "coop",  "Two hunters, each cat's health",          Int, 0, 30000;
    CoopGlance,      "coop",  "Two hunters, frames between glances",     Frames, 1, 90;
    CoopSwap,        "coop",  "Two hunters, the swap clock (x)",         Fixed, 0, fx(1,1);
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
    id: SpeciesId::PAIR,
    name: "Pair",
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
        gallop: Clip::Bound as usize,
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
    tuned_path: "crates/sim/src/species/pair/tuned.rs",
    pack: None,
    fight: &fight::FIGHT,
};
