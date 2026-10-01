//! The Veilstalker: a lanky lizard-cat that is not there. See
//! `docs/design/creatures/veilstalker.md` for what the fight is and why; this
//! file is what the animal *is* -- its skeleton, parts, moves, clips and its
//! own knobs. What its veil, its trail, its paint and its fire do is
//! [`fight`]; what it wants is [`mind`].
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
//! **The Pair's eighteen bones, re-proportioned again**: the same parents and
//! sides, a body six and a half metres nose to tail, three of it tail. The
//! design document guessed a lighter rig of twelve bones for the snapshot's
//! sake, but bones cost the snapshot nothing (a `Monster` is the same size
//! whatever its species) and the cat's topology comes with its animation
//! builders, so the lizard-cat is a long cat.
//!
//! **Slinking**, the shoulder is at 2.2 m and the belly at a metre: a level
//! swing from a fighter's chest meets the ribs and the forelegs, so a swing at
//! a place you *think* it is lands if you are right (§1). **The torso is two
//! halves**, left and right, because the hide is four regions -- head and
//! neck, left flank, right flank, tail -- and a region is the parts it is made
//! of ([`REGION`]); a hit on the left of the ribs is a hit on the left flank.
//!
//! Nobody stands on it (`sheds`): it is hit by aiming, not by going somewhere
//! on it (§1, "not climbable").
//!
//! Two generated files sit beside this one: `baked.rs`, its pose table
//! (`cargo run -p anim --bin bake_beast -- --species veilstalker`), and
//! `tuned.rs`, its knobs (`cargo run -p sim --bin bake_tuning`).

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

/// Its bones, by index: the Ridgeback's order, as the Pair's are.
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
/// The origin is the middle of the body: the hips sit eighty centimetres
/// behind it at 1.55 m, the chest eighty in front. The neck rises a little
/// to a head carried low at about 2.3 m, the tail runs three metres back from
/// the hips, and the legs are 1.35 m at the shoulder and 1.25 m at the hip.
pub const BONES: [Bone; COUNT] = [
    bone("root", NO_PARENT, v((-80, 100), (155, 100), (0, 1)), 0),
    bone("spine", ROOT, v((80, 100), (5, 100), (0, 1)), 0),
    bone("chest", SPINE, v((80, 100), (5, 100), (0, 1)), 0),
    bone("neck", CHEST, v((45, 100), (20, 100), (0, 1)), 0),
    bone("neck2", NECK, v((40, 100), (15, 100), (0, 1)), 0),
    bone("head", NECK2, v((35, 100), (10, 100), (0, 1)), 0),
    bone("tail1", ROOT, v((-60, 100), (5, 100), (0, 1)), 0),
    bone("tail2", TAIL1, v((-80, 100), (-8, 100), (0, 1)), 0),
    bone("tail3", TAIL2, v((-80, 100), (-12, 100), (0, 1)), 0),
    bone("tail4", TAIL3, v((-70, 100), (-12, 100), (0, 1)), 0),
    bone(
        "shoulder.l",
        CHEST,
        v((-5, 100), (-35, 100), (-34, 100)),
        -1,
    ),
    bone("forearm.l", SHOULDER_L, v((0, 1), (-62, 100), (0, 1)), -1),
    bone("shoulder.r", CHEST, v((-5, 100), (-35, 100), (34, 100)), 1),
    bone("forearm.r", SHOULDER_R, v((0, 1), (-62, 100), (0, 1)), 1),
    bone("thigh.l", ROOT, v((-10, 100), (-30, 100), (-34, 100)), -1),
    bone("shin.l", THIGH_L, v((10, 100), (-56, 100), (0, 1)), -1),
    bone("thigh.r", ROOT, v((-10, 100), (-30, 100), (34, 100)), 1),
    bone("shin.r", THIGH_R, v((10, 100), (-56, 100), (0, 1)), 1),
];

/// The left and right bones a mirrored clip swaps.
pub const MIRROR: [(usize, usize); 4] = [
    (SHOULDER_L, SHOULDER_R),
    (FOREARM_L, FOREARM_R),
    (THIGH_L, THIGH_R),
    (SHIN_L, SHIN_R),
];

/// The bones the head's tracking is spread across, base to tip. The last is
/// where it sees from, and where its breath comes out.
pub const NECK_CHAIN: [usize; 3] = [NECK, NECK2, bones::HEAD];

/// Which bone an attack's hit volume rides: the body, the head, the tail tip,
/// the left forepaw.
pub const FOLLOWS_BODY: u8 = 0;
pub const FOLLOWS_HEAD: u8 = 1;
pub const FOLLOWS_TAIL: u8 = 2;
pub const FOLLOWS_PAW: u8 = 3;

pub const FOLLOWS: [usize; 4] = [ROOT, bones::HEAD, TAIL4, FOREARM_L];

// ---------------------------------------------------------------------------
// Parts
// ---------------------------------------------------------------------------

pub const HEAD: usize = 0;
pub const NECK_PART: usize = 1;
pub const SHOULDER_PART_L: usize = 2;
pub const SHOULDER_PART_R: usize = 3;
pub const BARREL_L: usize = 4;
pub const BARREL_R: usize = 5;
pub const HAUNCH_L: usize = 6;
pub const HAUNCH_R: usize = 7;
pub const TAIL_BASE: usize = 8;
pub const TAIL_MID: usize = 9;
pub const TAIL_TIP: usize = 10;
pub const FORELEG_L: usize = 11;
pub const FORELEG_R: usize = 12;
pub const HINDLEG_L: usize = 13;
pub const HINDLEG_R: usize = 14;
pub const FOREPAW_L: usize = 15;
pub const FOREPAW_R: usize = 16;
pub const HINDPAW_L: usize = 17;
pub const HINDPAW_R: usize = 18;

pub const PART_COUNT: usize = 19;

/// The four regions of hide, each keeping its own damage total; past
/// `MottleDamage` a region stops cloaking for the rest of the fight (§4).
pub const REGION_HEAD: usize = 0;
pub const REGION_LEFT: usize = 1;
pub const REGION_RIGHT: usize = 2;
pub const REGION_TAIL: usize = 3;
pub const REGIONS: usize = 4;

/// Which region each part is hide of.
pub const REGION: [usize; PART_COUNT] = [
    REGION_HEAD,  // head
    REGION_HEAD,  // neck
    REGION_LEFT,  // shoulder, left
    REGION_RIGHT, // shoulder, right
    REGION_LEFT,  // ribs, left
    REGION_RIGHT, // ribs, right
    REGION_LEFT,  // haunch, left
    REGION_RIGHT, // haunch, right
    REGION_TAIL,  // tail
    REGION_TAIL,  // tail, middle
    REGION_TAIL,  // tail tip
    REGION_LEFT,  // foreleg, left
    REGION_RIGHT, // foreleg, right
    REGION_LEFT,  // hindleg, left
    REGION_RIGHT, // hindleg, right
    REGION_LEFT,  // forepaw, left
    REGION_RIGHT, // forepaw, right
    REGION_LEFT,  // hindpaw, left
    REGION_RIGHT, // hindpaw, right
];

/// What a person calls each region.
pub const REGION_NAMES: [&str; REGIONS] = ["head and neck", "left flank", "right flank", "tail"];

/// A part of the body: solid, and nobody stands on it.
const fn body(
    name: &'static str,
    on: usize,
    min: crate::math::V3,
    max: crate::math::V3,
    vuln: Knob,
) -> Part {
    part(name, on, min, max, vuln as u16).sheds()
}

/// Its proportions, at scale one.
pub const PARTS: [Part; PART_COUNT] = [
    // The head: a long, low skull, the jaw a metre and a half out from the
    // chest.
    body(
        "head",
        bones::HEAD,
        v((-15, 100), (-18, 100), (-22, 100)),
        v((65, 100), (20, 100), (22, 100)),
        Knob::VulnHead,
    ),
    body(
        "neck",
        NECK2,
        v((-50, 100), (-25, 100), (-22, 100)),
        v((15, 100), (22, 100), (22, 100)),
        Knob::VulnHead,
    ),
    // The shoulders: the top of the scapulae at 2.2 m; the two halves meet
    // on the spine.
    body(
        "left shoulder",
        CHEST,
        v((-45, 100), (-45, 100), (-42, 100)),
        v((45, 100), (55, 100), (0, 1)),
        Knob::VulnBody,
    ),
    body(
        "right shoulder",
        CHEST,
        v((-45, 100), (-45, 100), (0, 1)),
        v((45, 100), (55, 100), (42, 100)),
        Knob::VulnBody,
    ),
    // The ribs: the belly at a metre (the Ridgeback's lesson, §1).
    body(
        "left ribs",
        SPINE,
        v((-55, 100), (-55, 100), (-45, 100)),
        v((55, 100), (40, 100), (0, 1)),
        Knob::VulnBody,
    ),
    body(
        "right ribs",
        SPINE,
        v((-55, 100), (-55, 100), (0, 1)),
        v((55, 100), (40, 100), (45, 100)),
        Knob::VulnBody,
    ),
    body(
        "left haunch",
        ROOT,
        v((-50, 100), (-50, 100), (-45, 100)),
        v((30, 100), (45, 100), (0, 1)),
        Knob::VulnBody,
    ),
    body(
        "right haunch",
        ROOT,
        v((-50, 100), (-50, 100), (0, 1)),
        v((30, 100), (45, 100), (45, 100)),
        Knob::VulnBody,
    ),
    // The tail is **soft**: the spear is its move's volume, not the tail's
    // box, and three metres of tail that shoved whoever it passed would be a
    // wall drawn as nothing.
    part(
        "tail",
        TAIL1,
        v((-80, 100), (-14, 100), (-14, 100)),
        v((5, 100), (14, 100), (14, 100)),
        Knob::VulnTail as u16,
    )
    .soft(),
    part(
        "tail, middle",
        TAIL2,
        v((-80, 100), (-11, 100), (-11, 100)),
        v((5, 100), (11, 100), (11, 100)),
        Knob::VulnTail as u16,
    )
    .soft(),
    part(
        "tail tip",
        TAIL3,
        v((-130, 100), (-8, 100), (-8, 100)),
        v((5, 100), (8, 100), (8, 100)),
        Knob::VulnTail as u16,
    )
    .soft(),
    body(
        "left foreleg",
        SHOULDER_L,
        v((-14, 100), (-62, 100), (-12, 100)),
        v((14, 100), (12, 100), (12, 100)),
        Knob::VulnLeg,
    ),
    body(
        "right foreleg",
        SHOULDER_R,
        v((-14, 100), (-62, 100), (-12, 100)),
        v((14, 100), (12, 100), (12, 100)),
        Knob::VulnLeg,
    ),
    body(
        "left hindleg",
        THIGH_L,
        v((-16, 100), (-56, 100), (-13, 100)),
        v((16, 100), (14, 100), (13, 100)),
        Knob::VulnLeg,
    ),
    body(
        "right hindleg",
        THIGH_R,
        v((-16, 100), (-56, 100), (-13, 100)),
        v((16, 100), (14, 100), (13, 100)),
        Knob::VulnLeg,
    ),
    // The three-toed feet: what prints the snow.
    body(
        "left forepaw",
        FOREARM_L,
        v((-10, 100), (-73, 100), (-11, 100)),
        v((20, 100), (5, 100), (11, 100)),
        Knob::VulnLeg,
    ),
    body(
        "right forepaw",
        FOREARM_R,
        v((-10, 100), (-73, 100), (-11, 100)),
        v((20, 100), (5, 100), (11, 100)),
        Knob::VulnLeg,
    ),
    body(
        "left hindpaw",
        SHIN_L,
        v((-10, 100), (-69, 100), (-11, 100)),
        v((20, 100), (5, 100), (11, 100)),
        Knob::VulnLeg,
    ),
    body(
        "right hindpaw",
        SHIN_R,
        v((-10, 100), (-69, 100), (-11, 100)),
        v((20, 100), (5, 100), (11, 100)),
        Knob::VulnLeg,
    ),
];

/// The four legs, front to back.
pub const LEGS: [Leg; 4] = [
    Leg {
        upper: FORELEG_L,
        foot: FOREPAW_L,
        hip: SHOULDER_L,
        knee: FOREARM_L,
        front: true,
        side: -1,
    },
    Leg {
        upper: FORELEG_R,
        foot: FOREPAW_R,
        hip: SHOULDER_R,
        knee: FOREARM_R,
        front: true,
        side: 1,
    },
    Leg {
        upper: HINDLEG_L,
        foot: HINDPAW_L,
        hip: THIGH_L,
        knee: SHIN_L,
        front: false,
        side: -1,
    },
    Leg {
        upper: HINDLEG_R,
        foot: HINDPAW_R,
        hip: THIGH_R,
        knee: SHIN_R,
        front: false,
        side: 1,
    },
];

// ---------------------------------------------------------------------------
// Moves
// ---------------------------------------------------------------------------

/// Rears out of the cloak, gathers on its haunches, and leaps along the floor
/// jaws first. Dodged at its arrival.
pub const LUNGE: u8 = 0;
/// Stays low; the tail comes over its back and drives straight forward past
/// its head. Walked out of before its lane locks.
pub const SPEAR: u8 = 1;
/// A low swipe of the left foreleg at shin height. Jumped.
pub const RAKE: u8 = 2;
/// The right, eight frames on. Never chosen: the rake ends in it.
pub const RAKE2: u8 = 3;
/// Off a perch onto a circle four metres across. Walked out of.
pub const POUNCE: u8 = 4;
/// Rears highest, tail over its head, five quills in a flat fan. Answered
/// from behind a solid.
pub const QUILLS: u8 = 5;
/// A cloud vented from its flanks.
pub const SMOKE: u8 = 6;
/// A decloak with nothing in it, somewhere else; the real animal stands still
/// and leaves no prints.
pub const MIMIC: u8 = 7;
/// Recoil, bound away, re-cloak. Never chosen: two hits in an engagement
/// start it.
pub const RETREAT: u8 = 8;
/// Up a trunk or onto the wall's ledge, cloaked, to pounce from.
pub const CLIMB: u8 = 9;
/// Out of a panic: up and visible for half a second, then away. Never
/// chosen: a panic ends in it.
pub const GETUP: u8 = 10;

pub const MOVE_COUNT: usize = 11;

pub const MOVES: [MoveDecl; MOVE_COUNT] = [
    MoveDecl::new("Ambush lunge", Clip::Lunge as usize),
    MoveDecl::new("Tail spear", Clip::Spear as usize),
    MoveDecl::new("Rake", Clip::Rake as usize),
    MoveDecl::new("Rake, again", Clip::Rake2 as usize).never_chosen(),
    // Drawn where it lands from the first frame: the circle is positional.
    MoveDecl::new("Pounce", Clip::Pounce as usize).lobbed(),
    // Five quills in flight, tested by the species against the solids.
    MoveDecl::new("Quill fling", Clip::Quills as usize).own_hit(),
    MoveDecl::new("Smoke", Clip::Smoke as usize),
    MoveDecl::new("Mimic", Clip::Mimic as usize),
    MoveDecl::new("Retreat", Clip::Retreat as usize).never_chosen(),
    MoveDecl::new("Climb", Clip::Climb as usize),
    MoveDecl::new("Get up", Clip::Getup as usize).never_chosen(),
];

/// The moves that are strikes: each begins with a decloak, and the floor
/// test (`feel.rs`) holds each to `DecloakFloor`.
pub const STRIKES: [u8; 6] = [LUNGE, SPEAR, RAKE, POUNCE, QUILLS, MIMIC];

// ---------------------------------------------------------------------------
// Clips
// ---------------------------------------------------------------------------

/// Everything it knows how to look like, in the order its baked table lays
/// them out. Authored in `crates/anim/src/beast/veilstalker/`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Clip {
    Idle,
    /// The slink: low, the belly a metre off the snow, cloaked.
    Slink,
    /// The bound, at seventeen metres a second: it shimmers.
    Bound,
    Lunge,
    Spear,
    Rake,
    Rake2,
    Pounce,
    Quills,
    Smoke,
    /// The real animal through a mimic: flat and still.
    Mimic,
    Retreat,
    Climb,
    Getup,
    Flinch,
    Stumble,
    /// **Rolling in the snow**: the fire panic, the big window.
    Panic,
    Dead,
}

pub const CLIP_COUNT: usize = 18;

impl Clip {
    pub const ALL: [Clip; CLIP_COUNT] = [
        Clip::Idle,
        Clip::Slink,
        Clip::Bound,
        Clip::Lunge,
        Clip::Spear,
        Clip::Rake,
        Clip::Rake2,
        Clip::Pounce,
        Clip::Quills,
        Clip::Smoke,
        Clip::Mimic,
        Clip::Retreat,
        Clip::Climb,
        Clip::Getup,
        Clip::Flinch,
        Clip::Stumble,
        Clip::Panic,
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
    cycle("slink"),
    cycle("bound"),
    attack("lunge"),
    attack("spear"),
    attack("rake"),
    attack("rake2"),
    attack("pounce"),
    attack("quills"),
    attack("smoke"),
    attack("mimic"),
    attack("retreat"),
    attack("climb"),
    attack("getup"),
    once("flinch"),
    once("stumble"),
    once("panic"),
    once("dead"),
];

// ---------------------------------------------------------------------------
// Its own knobs
// ---------------------------------------------------------------------------

crate::species_knobs! {
    /// The knobs only the Veilstalker has: its hide, its veil, its trail and
    /// paint, its fire, its brain's own terms, and the coop numbers. The
    /// design document's names (`veil_<name>`) are in the comment beside each
    /// where they differ.
    pub enum Knob;
    VulnHead,        "hide",   "Head and neck (x damage)",               Fixed, 0, fx(3,1);
    VulnBody,        "hide",   "Body (x damage)",                        Fixed, 0, fx(3,1);
    VulnLeg,         "hide",   "Legs (x damage)",                        Fixed, 0, fx(3,1);
    VulnTail,        "hide",   "Tail (x damage)",                        Fixed, 0, fx(3,1);
    // veil_decloak_floor: no strike reveals itself for less.
    DecloakFloor,    "veil",   "Every decloak at least (floor)",         Frames, 18, 60;
    DecloakRamp,     "veil",   "Decloak, frames to full strength",       Frames, 1, 60;
    Recloak,         "veil",   "Re-cloak, frames of fading",             Frames, 1, 120;
    ShimmerSpeed,    "veil",   "Shimmers above (m/s)",                   Fixed, 0, fx(30,1);
    FrayedSpeed,     "veil",   "Desperate, shimmers above (m/s)",        Fixed, 0, fx(30,1);
    ShimmerShown,    "veil",   "A shimmer is drawn at",                  Fixed, 0, fx(1,1);
    OutlineShown,    "veil",   "An outline is drawn at",                 Fixed, 0, fx(1,1);
    PaintShown,      "veil",   "A painted part is drawn at",             Fixed, 0, fx(1,1);
    LungeGather,     "veil",   "Lunge, gather after its decloak",        Frames, 0, 30;
    PounceAir,       "veil",   "Pounce, frames in the air",              Frames, 1, 60;
    SpearLock,       "veil",   "Spear's lane locks at startup frame",    Frames, 0, 60;
    // veil_paint_frames
    PaintFrames,     "paint",  "Paint lit for",                          Frames, 0, 1200;
    PaintFade,       "paint",  "Paint fades over its last",              Frames, 0, 600;
    // veil_mottle_damage
    MottleDamage,    "paint",  "A region mottles at damage",             Int, 1, 5000;
    PrintsSnow,      "trail",  "Prints last, in snow",                   Frames, 0, 1200;
    PrintsAsh,       "trail",  "Prints last, in ash",                    Frames, 0, 1200;
    EmberGlow,       "trail",  "Stirred embers glow for",                Frames, 0, 240;
    RedPrints,       "trail",  "Through blood, red prints for",          Frames, 0, 600;
    QuillSpeed,      "quills", "Quills fly at (m/s)",                    Fixed, 0, fx(80,1);
    QuillFan,        "quills", "Fan, half-width (turns)",                Fixed, 0, fx(1,4);
    QuillReach,      "quills", "Quills fly for (m)",                     Fixed, 0, fx(40,1);
    QuillHeight,     "quills", "Quills leave at",                        Fixed, 0, fx(6,1);
    QuillRadius,     "quills", "A quill's width",                        Fixed, 0, fx(1,1);
    SmokeRadius,     "smoke",  "Cloud radius",                           Fixed, 0, fx(10,1);
    // the doc's 90: nothing strikes into a cloud this young
    SmokeQuiet,      "smoke",  "Nothing strikes into it until",          Frames, 0, 300;
    FireFrames,      "fire",   "In fire this long, it panics",           Frames, 1, 120;
    PanicFrames,     "fire",   "Panic lasts",                            Frames, 1, 600;
    // veil_panic_lockout
    PanicLockout,    "fire",   "No panic again for",                     Frames, 0, 6000;
    BrazierSize,     "fire",   "Brazier, height and width",              Fixed, 0, fx(4,1);
    CoalsRadius,     "fire",   "Coals, radius",                          Fixed, 0, fx(6,1);
    FireShy,         "fire",   "Walks no nearer fire than",              Fixed, 0, fx(10,1);
    // veil_glance is the common `GlanceFrames`; veil_view_cone:
    ViewCone,        "mind",   "View gate, half-angle (turns)",          Fixed, 0, fx(1,4);
    ExposedNear,     "mind",   "Exposed and stalking, runs from a target within", Fixed, 0, fx(20,1);
    EngagementStrikes,"mind",  "Strikes an engagement, at most",          Int, 1, 9;
    ViewMargin,      "mind",   "View gate, seen from this far either side too", Fixed, 0, fx(4,1);
    // veil_edge_bias
    EdgeBias,        "mind",   "Edge of the view, more appetite",        Int, 0, 8000;
    // veil_bait_appetite
    BaitAppetite,    "mind",   "Mimic, appetite after a dodge seen",     Int, 0, 8000;
    BaitSpeed,       "mind",   "A sample this fast was a dodge",         Fixed, 0, fx(30,1);
    BaitMemory,      "mind",   "A dodge seen is remembered for",         Frames, 0, 600;
    ExposureSmoke,   "mind",   "Painted or mottled, smoke appetite",     Int, 0, 8000;
    ExposureCut,     "mind",   "Painted, strikes' appetite (x)",         Fixed, 0, fx(1,1);
    // veil_mimic_quiet
    MimicQuiet,      "mind",   "No strike after a mimic for",            Frames, 0, 300;
    MimicFrom,       "mind",   "Mimic, nearest it plays to the target",  Fixed, 0, fx(20,1);
    MimicTo,         "mind",   "Mimic, furthest",                        Fixed, 0, fx(20,1);
    // veil_stalk_frames
    StalkMin,        "mind",   "Stalks at least",                        Frames, 0, 1200;
    StalkMax,        "mind",   "Stalks at most",                         Frames, 0, 1200;
    StalkRange,      "mind",   "Stalks at, from the target",             Fixed, 0, fx(30,1);
    // veil_mottled_reach
    MottledReach,    "mind",   "Each mottled region, stalks further by", Fixed, 0, fx(10,1);
    BoundFrom,       "mind",   "Bounds in from beyond",                  Fixed, 0, fx(40,1);
    BoundTo,         "mind",   "Bounds until within",                    Fixed, 0, fx(40,1);
    StalkSpeed,      "mind",   "Slinks at (m/s)",                        Fixed, 0, fx(20,1);
    // veil_watched_penalty
    WatchedPenalty,  "mind",   "A partner watching the point, less",     Int, 0, 8000;
    StrainedHealth,  "mind",   "Mimics and smokes below health (x)",     Fixed, 0, fx(1,1);
    DesperateHealth, "mind",   "Desperate below health (x)",             Fixed, 0, fx(1,1);
    LeaveHits,       "retreat","Leaves after hits",                      Int, 1, 9;
    LeaveDesperate,  "retreat","Desperate, leaves after hits",           Int, 1, 9;
    HitGap,          "retreat","Blows closer than this are one hit",     Frames, 1, 60;
    RetreatStagger,  "retreat","A burst in the recoil staggers it for",  Frames, 0, 240;
    RetreatFar,      "retreat","Bounds away this far",                   Fixed, 0, fx(30,1);
    PerchReach,      "perch",  "Climbs a perch within",                  Fixed, 0, fx(20,1);
    PerchWidth,      "perch",  "A top this wide is somewhere to perch",  Fixed, 0, fx(5,1);
    PerchHighest,    "perch",  "A top no higher than",                   Fixed, 0, fx(10,1);
    PerchLowest,     "perch",  "A top no lower than",                    Fixed, 0, fx(10,1);
    ClimbAppetite,   "perch",  "Climb, appetite",                        Int, 0, 8000;
    PerchPatience,   "perch",  "Perched, comes down after",              Frames, 0, 2400;
    ClimbArc,        "perch",  "The climb's leap clears its top by",     Fixed, 0, fx(3,1);
    PounceArc,       "perch",  "The pounce's leap rises by",             Fixed, 0, fx(3,1);
    FallSpeed,       "perch",  "Off an edge, comes down at (m/s)",       Fixed, 0, fx(40,1);
    LandShort,       "perch",  "A leap's body lands this short of its mark", Fixed, 0, fx(3,1);
    MiddleHeight,    "veil",   "A decloak is seen at its middle, this high", Fixed, 0, fx(4,1);
    OutlineHigh,     "veil",   "Outlined in water up to this high",      Fixed, 0, fx(4,1);
    FireHeight,      "fire",   "Fire reaches a part no higher than",     Fixed, 0, fx(8,1);
    MimicEdge,       "mind",   "Mimic plays this far to the view's edge (x)", Fixed, 0, fx(1,1);
    StalkEdge,       "mind",   "Stalks this far to the view's edge (x)", Fixed, 0, fx(1,1);
    StalkStep,       "mind",   "Stalks round at most (turns a step)",    Fixed, 0, fx(1,2);
    WallLook,        "retreat","Bounding away, looks ahead for a wall",  Fixed, 0, fx(10,1);
    WallTurn,        "retreat","Bounding away, turns off a wall at (turns/frame)", Fixed, 0, fx(1,8);
    PerchDrop,       "perch",  "Comes down this far out from its perch", Fixed, 0, fx(10,1);
    CoopHealth,      "coop",   "Two hunters, health",                    Int, 0, 30000;
    CoopStalk,       "coop",   "Two hunters, the stalk (x)",             Fixed, 0, fx(1,1);
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
    id: SpeciesId::VEILSTALKER,
    name: "Veilstalker",
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
        walk: Clip::Slink as usize,
        gallop: Clip::Bound as usize,
        flinch: Clip::Flinch as usize,
        stumble: Clip::Stumble as usize,
        topple: Clip::Panic as usize,
        dead: Clip::Dead as usize,
    },
    span: &baked::SPAN,
    rows: baked::ROWS,
    row: |r| &baked::FRAMES[r],
    own: OWN,
    tuned: &tuned::KNOBS,
    tuned_path: "crates/sim/src/species/veilstalker/tuned.rs",
    pack: None,
    fight: &fight::FIGHT,
};
