//! The Sandmaw: a worm eleven metres long that lives under a sand floor,
//! perceives the arena only as vibration, and comes up where it last heard
//! something. See `docs/design/creatures/sandmaw.md` for what the fight is and
//! why; this file is what the animal *is* -- its skeleton, parts, moves, clips
//! and its own knobs. What it does to the floor, the fighters and itself is
//! [`fight`]; what it wants is [`mind`].
//!
//! ```text
//!                                  lip.up
//!                                 /
//!   tail5 ─ tail4 ─ tail3 ─ tail2 ─ tail1 ─ ROOT ─ neck1 ─ … ─ neck5 ─ head
//!                                                                     \
//!                                                                      lip.down
//! ```
//!
//! **One chain, rooted at the hole.** Eleven segments a metre long and 2.4 m
//! thick. The root is where the body meets the sand when it stands: the six
//! segments in front of it are the column, the five behind it the body still
//! under the floor. Buried, the swim clip carries the whole chain five and a
//! half metres back and a metre and a half down (`hips`), so the creature's
//! origin is its head end and what collides with rock is the head
//! (`FightDecl::collides`), the rest following the path the head took. The
//! design document's spine of positions is that path, kept in the lore for the
//! wake ([`fight::word::SPINE`]); the body itself is clips, as every creature's
//! is.
//!
//! **What is under the sand has no body** (`FightDecl::buried`): a part whose
//! box is wholly below the floor cannot be hit, stood on or walked into, and
//! is not drawn. So buried it has no hurtbox at all, and standing, the five
//! metres still under are as untouchable as the design says.
//!
//! Two generated files sit beside this one: `baked.rs`, its pose table
//! (`cargo run -p anim --bin bake_beast -- --species sandmaw`), and
//! `tuned.rs`, its knobs (`cargo run -p sim --bin bake_tuning`).

pub(crate) mod baked;
pub mod fight;
pub mod mind;
mod tuned;

use crate::beast::{Bone, ClipDecl, NO_PARENT, Part, bone, breakables, part, v};
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

/// The Sandmaw's bones, by index.
pub mod bones {
    pub const ROOT: usize = 0;
    pub const NECK1: usize = 1;
    pub const NECK2: usize = 2;
    pub const NECK3: usize = 3;
    pub const NECK4: usize = 4;
    pub const NECK5: usize = 5;
    pub const HEAD: usize = 6;
    pub const LIP_UP: usize = 7;
    pub const LIP_DOWN: usize = 8;
    pub const TAIL1: usize = 9;
    pub const TAIL2: usize = 10;
    pub const TAIL3: usize = 11;
    pub const TAIL4: usize = 12;
    pub const TAIL5: usize = 13;

    pub const COUNT: usize = 14;

    /// The column, root to head: what stands out of the sand.
    pub const COLUMN: [usize; 7] = [ROOT, NECK1, NECK2, NECK3, NECK4, NECK5, HEAD];
    /// What stays under it, root to tip.
    pub const TAIL: [usize; 5] = [TAIL1, TAIL2, TAIL3, TAIL4, TAIL5];
}

use bones::*;

/// Where each bone sits in its parent's rest frame, in metres: a straight
/// worm lying along `+x`, its middle line at the floor, the root at the hole.
pub const BONES: [Bone; COUNT] = [
    bone("root", NO_PARENT, v((0, 1), (0, 1), (0, 1)), 0),
    bone("neck1", ROOT, v((92, 100), (0, 1), (0, 1)), 0),
    bone("neck2", NECK1, v((92, 100), (0, 1), (0, 1)), 0),
    bone("neck3", NECK2, v((92, 100), (0, 1), (0, 1)), 0),
    bone("neck4", NECK3, v((92, 100), (0, 1), (0, 1)), 0),
    bone("neck5", NECK4, v((92, 100), (0, 1), (0, 1)), 0),
    bone("head", NECK5, v((92, 100), (0, 1), (0, 1)), 0),
    // The two halves of the tooth ring's hinge: the mouth opens by swinging
    // them apart, to four metres across for the swallow.
    bone("lip.up", HEAD, v((50, 100), (60, 100), (0, 1)), 0),
    bone("lip.down", HEAD, v((50, 100), (-60, 100), (0, 1)), 0),
    bone("tail1", ROOT, v((-92, 100), (0, 1), (0, 1)), 0),
    bone("tail2", TAIL1, v((-92, 100), (0, 1), (0, 1)), 0),
    bone("tail3", TAIL2, v((-92, 100), (0, 1), (0, 1)), 0),
    bone("tail4", TAIL3, v((-92, 100), (0, 1), (0, 1)), 0),
    bone("tail5", TAIL4, v((-92, 100), (0, 1), (0, 1)), 0),
];

/// Nothing is left or right on a worm.
pub const MIRROR: [(usize, usize); 0] = [];

/// The head turns to keep what it attends in front of it, spread down the
/// top of the column. The last is the head: what it hears and feels from.
pub const NECK_CHAIN: [usize; 3] = [NECK4, NECK5, HEAD];

/// Which bone an attack's hit volume rides: the body, the head, or the tail.
pub const FOLLOWS_BODY: u8 = 0;
pub const FOLLOWS_HEAD: u8 = 1;
pub const FOLLOWS_TAIL: u8 = 2;

pub const FOLLOWS: [usize; 3] = [ROOT, HEAD, TAIL5];

// ---------------------------------------------------------------------------
// Parts
// ---------------------------------------------------------------------------

/// The body segments, root first then the column then the tail.
pub const SEG_ROOT: usize = 0;
pub const SEG_N1: usize = 1;
pub const SEG_N2: usize = 2;
pub const SEG_N3: usize = 3;
pub const SEG_N4: usize = 4;
pub const SEG_N5: usize = 5;
pub const HEAD_PART: usize = 6;
pub const SEG_T1: usize = 7;
pub const SEG_T2: usize = 8;
pub const SEG_T3: usize = 9;
pub const SEG_T4: usize = 10;
pub const SEG_T5: usize = 11;
/// **The tooth ring**: the front of the head, breakable. Reachable only when
/// the mouth is down -- the swallow's startup, the gag, a beached head.
pub const TEETH: usize = 12;
pub const LIP_UP_PART: usize = 13;
pub const LIP_DOWN_PART: usize = 14;
/// **The spiracles**: six along its back, three on the column and three on
/// the body behind it. Double damage, open only while it is out of the sand.
pub const VENT_1: usize = 15;
pub const VENT_2: usize = 16;
pub const VENT_3: usize = 17;
pub const VENT_4: usize = 18;
pub const VENT_5: usize = 19;
pub const VENT_6: usize = 20;
/// **The soft throat**: the front of the column, 1 m to 5 m up when it stands.
pub const THROAT_1: usize = 21;
pub const THROAT_2: usize = 22;
pub const THROAT_3: usize = 23;
pub const THROAT_4: usize = 24;
/// **The gullet**: a hollow in the head. The swallowed are held here.
pub const GULLET: usize = 25;

pub const PART_COUNT: usize = 26;

pub const VENTS: [usize; 6] = [VENT_1, VENT_2, VENT_3, VENT_4, VENT_5, VENT_6];
pub const THROAT: [usize; 4] = [THROAT_1, THROAT_2, THROAT_3, THROAT_4];
/// What a teammate's blow frees the swallowed by hitting: the head, its
/// mouth and the top of the throat.
pub const HEAD_PARTS: [usize; 4] = [HEAD_PART, TEETH, LIP_UP_PART, LIP_DOWN_PART];
/// The back a rider stands on, beached: every body segment.
pub const BACK: [usize; 12] = [
    SEG_ROOT, SEG_N1, SEG_N2, SEG_N3, SEG_N4, SEG_N5, HEAD_PART, SEG_T1, SEG_T2, SEG_T3, SEG_T4,
    SEG_T5,
];

/// A body segment: a box a metre long and 2.4 m thick on its bone.
const fn segment(name: &'static str, on: usize) -> Part {
    part(
        name,
        on,
        v((-50, 100), (-120, 100), (-120, 100)),
        v((50, 100), (120, 100), (120, 100)),
        Knob::VulnHide as u16,
    )
    .mountable()
}

/// A spiracle: a dark slit on top of a segment, proud of its back.
const fn vent(name: &'static str, on: usize) -> Part {
    part(
        name,
        on,
        v((-30, 100), (110, 100), (-40, 100)),
        v((30, 100), (138, 100), (40, 100)),
        Knob::VulnVent as u16,
    )
    .soft()
    .weak_point()
}

/// The soft throat under a column segment.
const fn throat(name: &'static str, on: usize) -> Part {
    part(
        name,
        on,
        v((-45, 100), (-136, 100), (-70, 100)),
        v((45, 100), (-110, 100), (70, 100)),
        Knob::VulnThroat as u16,
    )
    .soft()
    .weak_point()
}

pub const PARTS: [Part; PART_COUNT] = [
    segment("root", ROOT),
    segment("neck 1", NECK1),
    segment("neck 2", NECK2),
    segment("neck 3", NECK3),
    segment("neck 4", NECK4),
    segment("neck 5", NECK5),
    // The head: a little thicker than the body, armoured.
    part(
        "head",
        HEAD,
        v((-50, 100), (-130, 100), (-130, 100)),
        v((50, 100), (130, 100), (130, 100)),
        Knob::VulnHead as u16,
    )
    .mountable(),
    segment("tail 1", TAIL1),
    segment("tail 2", TAIL2),
    segment("tail 3", TAIL3),
    segment("tail 4", TAIL4),
    // The tip tapers.
    part(
        "tail 5",
        TAIL5,
        v((-60, 100), (-80, 100), (-80, 100)),
        v((50, 100), (80, 100), (80, 100)),
        Knob::VulnHide as u16,
    )
    .mountable(),
    // The tooth ring: the front face of the head, a hand thick.
    part(
        "tooth ring",
        HEAD,
        v((50, 100), (-120, 100), (-120, 100)),
        v((85, 100), (120, 100), (120, 100)),
        Knob::VulnTeeth as u16,
    )
    .soft()
    .breakable(),
    // The two halves of the mouth, which swing open. Soft: they are the
    // silhouette of the swallow's tell, not a wall.
    part(
        "upper lip",
        LIP_UP,
        v((0, 1), (-15, 100), (-110, 100)),
        v((110, 100), (15, 100), (110, 100)),
        Knob::VulnHead as u16,
    )
    .soft(),
    part(
        "lower lip",
        LIP_DOWN,
        v((0, 1), (-15, 100), (-110, 100)),
        v((110, 100), (15, 100), (110, 100)),
        Knob::VulnHead as u16,
    )
    .soft(),
    vent("vent 1", NECK1),
    vent("vent 2", NECK2),
    vent("vent 3", NECK3),
    vent("vent 4", TAIL1),
    vent("vent 5", TAIL2),
    vent("vent 6", TAIL3),
    throat("throat 1", NECK1),
    throat("throat 2", NECK2),
    throat("throat 3", NECK3),
    throat("throat 4", NECK4),
    // The gullet, inside the head: a hollow a body fits in, held there.
    part(
        "gullet",
        HEAD,
        v((-40, 100), (-90, 100), (-60, 100)),
        v((40, 100), (90, 100), (60, 100)),
        Knob::VulnHead as u16,
    )
    .hollow(),
];

// ---------------------------------------------------------------------------
// Moves
// ---------------------------------------------------------------------------

/// From under the sand, straight up through a circle under the noise.
pub const RISE: u8 = 0;
/// Up ahead of a moving noise and over, through a lane.
pub const BREACH: u8 = 1;
/// A sinkhole under a quiet body it felt; a rise at its centre after.
pub const UNDERTOW: u8 = 2;
/// A cone of wet sand from the mouth, standing.
pub const SPIT: u8 = 3;
/// The tail out of the sand behind the column, sweeping a half-ring.
pub const LASH: u8 = 4;
/// The head comes down and the mouth opens: whoever is in it is taken.
pub const SWALLOW: u8 = 5;
/// The dive: the vents shut, it plunges, riders thrown.
pub const SOUND: u8 = 6;
/// Somebody in its throat. Never chosen: the swallow landing puts it here.
pub const HOLD: u8 = 7;
/// The dive from a beach, off its side. Never chosen: the beach ending puts
/// it here.
pub const DIVE: u8 = 8;

pub const MOVE_COUNT: usize = 9;

pub const MOVES: [MoveDecl; MOVE_COUNT] = [
    MoveDecl::new("Rise-bite", Clip::Rise as usize).lobbed(),
    MoveDecl::new("Breach-dive", Clip::Breach as usize),
    MoveDecl::new("Undertow", Clip::Undertow as usize),
    MoveDecl::new("Sand spit", Clip::Spit as usize).own_hit(),
    MoveDecl::new("Tail lash", Clip::Lash as usize).own_hit(),
    MoveDecl::new("Swallow-grab", Clip::Grab as usize),
    MoveDecl::new("Sound", Clip::Sound as usize),
    MoveDecl::new("Swallowed", Clip::Hold as usize).never_chosen(),
    MoveDecl::new("Dive", Clip::Dive as usize).never_chosen(),
];

/// The moves it throws from under the sand.
pub const fn from_below(kind: u8) -> bool {
    matches!(kind, RISE | BREACH | UNDERTOW)
}

// ---------------------------------------------------------------------------
// Clips
// ---------------------------------------------------------------------------

/// Everything the Sandmaw knows how to look like, in the order its baked
/// table lays them out. Authored in `crates/anim/src/beast/sandmaw/`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Clip {
    /// Standing out of the sand, breathing: the column between moves.
    Stand,
    /// Under the sand, swimming: indexed by ground covered. Every part of it
    /// under the floor.
    Swim,
    Rise,
    Breach,
    Undertow,
    Spit,
    Lash,
    Grab,
    Sound,
    Hold,
    Dive,
    /// Struck, or gagged, standing.
    Flinch,
    /// **Beached**: on the sand, writhing. What the shared ladder calls a
    /// topple, and the big window.
    Beached,
    Dead,
}

pub const CLIP_COUNT: usize = 14;

impl Clip {
    pub const ALL: [Clip; CLIP_COUNT] = [
        Clip::Stand,
        Clip::Swim,
        Clip::Rise,
        Clip::Breach,
        Clip::Undertow,
        Clip::Spit,
        Clip::Lash,
        Clip::Grab,
        Clip::Sound,
        Clip::Hold,
        Clip::Dive,
        Clip::Flinch,
        Clip::Beached,
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
    cycle("stand"),
    cycle("swim"),
    attack("rise"),
    attack("breach"),
    attack("undertow"),
    attack("spit"),
    attack("lash"),
    attack("grab"),
    attack("sound"),
    attack("hold"),
    attack("dive"),
    once("flinch"),
    once("beached"),
    once("dead"),
];

// ---------------------------------------------------------------------------
// Its own knobs
// ---------------------------------------------------------------------------

crate::species_knobs! {
    /// The knobs only the Sandmaw has: its hide, what it hears and how it
    /// searches, its stand, the shapes of its spray and its tail, the
    /// swallow, the beach and its arc. The design document's names for them
    /// (§5) are in the comment beside each, where they differ.
    pub enum Knob;
    VulnHide,        "hide",    "Hide (x damage)",                      Fixed, 0, fx(3,1);
    VulnHead,        "hide",    "Head (x damage)",                      Fixed, 0, fx(3,1);
    VulnVent,        "hide",    "Spiracles, open (x damage)",           Fixed, 0, fx(4,1);
    VulnThroat,      "hide",    "Throat (x damage)",                    Fixed, 0, fx(4,1);
    VulnTeeth,       "hide",    "Tooth ring (x damage)",                Fixed, 0, fx(4,1);
    // circle_radius
    CircleRadius,    "mind",    "Circles its last noise at",            Fixed, 0, fx(20,1);
    // silence_patience
    SilencePatience, "mind",    "Searches after silence of",            Frames,0, 1200;
    SearchPitch,     "mind",    "Search spiral, wider per turn by",     Fixed, 0, fx(30,1);
    SearchLook,      "mind",    "Search spiral, looks ahead by (turns)",Fixed, 0, fx(1,2);
    // island_patience
    IslandPatience,  "mind",    "Surfaces at an island after",          Frames,0, 1200;
    IslandAppetite,  "mind",    "Island, appetite to surface and spit", Int,   0, 8000;
    // surface_max
    SurfaceMax,      "mind",    "Stays up at most",                     Frames,0, 900;
    SoundAppetite,   "mind",    "Sound, appetite once the stand is up", Int,   0, 8000;
    // breach_deafness
    BreachDeafness,  "mind",    "Deaf after a breach for",              Frames,0, 240;
    // hunger_hearing
    HungerHealth,    "mind",    "Hungry below health (x)",              Fixed, 0, fx(1,1);
    HungerHearing,   "mind",    "Hungry, hears further (x)",            Fixed, fx(1,1), fx(3,1);
    HungerPatience,  "mind",    "Hungry, searches after silence of",    Frames,0, 1200;
    FranticHealth,   "mind",    "Frantic below health (x)",             Fixed, 0, fx(1,1);
    FranticStand,    "mind",    "Frantic, stands for at most",          Frames,0, 240;
    // bite_lead, trail_lead
    TrailLead,       "mind",    "Breach leads a trail by",              Frames,0, 120;
    TrailWindow,     "mind",    "Two noises make a trail within",       Frames,1, 120;
    TrailSpeed,      "mind",    "A trail is faster than",               Fixed, 0, fx(20,1);
    HeardFor,        "mind",    "A noise is fresh for",                 Frames,0, 240;
    RiseAppetite,    "mind",    "Rise-bite, at a fresh noise",          Int,   0, 8000;
    RiseReach,       "mind",    "Rise-bite, rises at most this far",    Fixed, 0, fx(40,1);
    BreachAppetite,  "mind",    "Breach, at a trail",                   Int,   0, 8000;
    UndertowAppetite,"mind",    "Undertow, at a body it feels",         Int,   0, 8000;
    FrontAppetite,   "mind",    "Standing, at what it heard in front",  Int,   0, 8000;
    // the moves' shapes
    RiseBroken,      "teeth",   "Rise-bite radius, teeth broken",       Fixed, 0, fx(4,1);
    ShieldThrow,     "rise",    "A shield in the circle thrown by",     Fixed, 0, fx(10,1);
    SpitLength,      "spit",    "Spit reaches",                         Fixed, 0, fx(20,1);
    SpitFrom,        "spit",    "Spit skids along the sand this high",  Fixed, 0, fx(4,1);
    SpitSpread,      "spit",    "Spit, half-width at its end",          Fixed, 0, fx(10,1);
    SpitSlow,        "spit",    "Spit, slowed to (x)",                  Fixed, 0, fx(1,1);
    SpitSlowFrames,  "spit",    "Spit, slowed for",                     Frames,0, 600;
    LashInner,       "lash",    "Lash, from the hole at least",         Fixed, 0, fx(10,1);
    LashOuter,       "lash",    "Lash, from the hole at most",          Fixed, 0, fx(14,1);
    LashLow,         "lash",    "Lash, sweeps no lower than",           Fixed, 0, fx(4,1);
    LashHigh,        "lash",    "Lash, sweeps no higher than",          Fixed, 0, fx(4,1);
    LashStagger,     "lash",    "Lash, staggers for",                   Frames,0, 240;
    // the swallow (the bite is the move's own damage)
    GulpDamage,      "swallow", "Swallowed, a gulp",                    Int,   0, 400;
    GulpEvery,       "swallow", "Gulps every",                          Frames,1, 120;
    GulpWindow,      "swallow", "A gulp's window to break out",         Frames,0, 60;
    HoldFrames,      "swallow", "Held at most",                         Frames,1, 600;
    SpitOutDamage,   "swallow", "Spat out at the end, damage",          Int,   0, 400;
    SpitOutDistance, "swallow", "Spat out beside the hole at",          Fixed, 0, fx(10,1);
    GagFrames,       "swallow", "Gagged for",                           Frames,0, 240;
    GagTeeth,        "swallow", "Into the open mouth (x damage)",       Fixed, 0, fx(4,1);
    // sound
    RiderThrow,      "sound",   "Riders thrown, damage",                Int,   0, 400;
    RootPush,        "sound",   "A root holds it up for",               Frames,0, 240;
    // the beach
    StoneReach,      "beach",   "A stone this near the circle beaches", Fixed, 0, fx(4,1);
    // how it is drawn
    SegmentLength,   "wake",    "Wake, a segment every",                Fixed, 0, fx(4,1);
    WakeHeight,      "wake",    "Wake, raised sand",                    Fixed, 0, fx(2,1);
    WakeWidth,       "wake",    "Wake, half-width",                     Fixed, 0, fx(4,1);
    FinHeight,       "wake",    "Fin, height",                          Fixed, 0, fx(4,1);
    HeardShown,      "wake",    "A heard noise is drawn for",           Frames,0, 240;
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
    id: SpeciesId::SANDMAW,
    name: "Sandmaw",
    bones: &BONES,
    mirror: &MIRROR,
    neck: &NECK_CHAIN,
    follows: &FOLLOWS,
    parts: &PARTS,
    breakable: breakables(&PARTS),
    legs: &[],
    moves: &MOVES,
    clips: &CLIPS,
    stock: Stock {
        idle: Clip::Stand as usize,
        walk: Clip::Swim as usize,
        gallop: Clip::Swim as usize,
        flinch: Clip::Flinch as usize,
        stumble: Clip::Flinch as usize,
        topple: Clip::Beached as usize,
        dead: Clip::Dead as usize,
    },
    span: &baked::SPAN,
    rows: baked::ROWS,
    row: |r| &baked::FRAMES[r],
    own: OWN,
    tuned: &tuned::KNOBS,
    tuned_path: "crates/sim/src/species/sandmaw/tuned.rs",
    pack: None,
    fight: &fight::FIGHT,
};
