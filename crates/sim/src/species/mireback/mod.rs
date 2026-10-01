//! The Mireback: a toad the size of a barn, and the floor as the clock.
//!
//! Seven metres to the top of its back, twelve across, and a walk of two
//! metres a second. Everything it does leaves tar on the floor; tar is where
//! its other moves become hard to answer; fire takes the tar back, hurts it
//! most of all, and leaves the slag that is the only way up it. See
//! `docs/design/creatures/mireback.md` for what the fight is and why; this
//! file is what the animal *is* -- its skeleton, parts, legs, moves, clips and
//! its own knobs. What it does to the floor and the fighters is
//! [`fight`]; what it wants is [`mind`].
//!
//! ```text
//!            jaw ── tongue
//!           /
//!   root ── head ── throat (the sac)
//!     |
//!   shoulder ─ forearm   (each side)     thigh ─ shin   (each side)
//! ```
//!
//! **A squat dome built as terraces**: the rim of the back -- the flanks, the
//! rump and the brow -- stands at 5.2 m, the dome inside it at 5.8, the crown
//! at 6.4, and the four warts stand proud of the crown to 7.0, waist height on
//! a rider, for the reason the Ridgeback's ridge does. Each terrace is a
//! step a rider walks up (`riding.step_you_can_walk_up` is 0.75 m), so the
//! climb is getting onto the rim at all.
//!
//! Two generated files sit beside this one: `baked.rs`, its pose table
//! (`cargo run -p anim --bin bake_beast -- --species mireback`), and
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

/// The Mireback's bones, by index.
pub mod bones {
    pub const ROOT: usize = 0;
    pub const HEAD: usize = 1;
    pub const JAW: usize = 2;
    pub const TONGUE: usize = 3;
    pub const THROAT: usize = 4;
    pub const SHOULDER_L: usize = 5;
    pub const FOREARM_L: usize = 6;
    pub const SHOULDER_R: usize = 7;
    pub const FOREARM_R: usize = 8;
    pub const THIGH_L: usize = 9;
    pub const SHIN_L: usize = 10;
    pub const THIGH_R: usize = 11;
    pub const SHIN_R: usize = 12;

    pub const COUNT: usize = 13;
}

use bones::*;

/// Where each bone sits in its parent's rest frame, in metres.
///
/// The root is the middle of the body, 2.4 m up: a toad's weight is all in
/// its middle and its legs are folded under it, so the hips are low and the
/// height is in the dome rather than the legs -- the opposite of the
/// Ridgeback, whose height is all leg. Nothing reaches its back by walking
/// under it; everything reaches it by getting up beside it.
pub const BONES: [Bone; COUNT] = [
    bone("root", NO_PARENT, v((0, 1), (240, 100), (0, 1)), 0),
    // The head is the front of the dome: a toad has no neck to speak of.
    bone("head", ROOT, v((260, 100), (0, 1), (0, 1)), 0),
    // The hinge of the jaw, at the back of the mouth.
    bone("jaw", HEAD, v((20, 100), (20, 100), (0, 1)), 0),
    bone("tongue", JAW, v((120, 100), (-20, 100), (0, 1)), 0),
    // The throat sac hangs off the head under the chin.
    bone("throat", HEAD, v((60, 100), (-40, 100), (0, 1)), 0),
    // Forelegs: short arms off the front of the body.
    bone(
        "shoulder.l",
        ROOT,
        v((220, 100), (-90, 100), (-520, 100)),
        -1,
    ),
    bone(
        "forearm.l",
        SHOULDER_L,
        v((30, 100), (-80, 100), (-20, 100)),
        -1,
    ),
    bone("shoulder.r", ROOT, v((220, 100), (-90, 100), (520, 100)), 1),
    bone(
        "forearm.r",
        SHOULDER_R,
        v((30, 100), (-80, 100), (20, 100)),
        1,
    ),
    // Hind legs: the big folded haunches it leaps from.
    bone("thigh.l", ROOT, v((-340, 100), (-80, 100), (-540, 100)), -1),
    bone("shin.l", THIGH_L, v((80, 100), (-90, 100), (-20, 100)), -1),
    bone("thigh.r", ROOT, v((-340, 100), (-80, 100), (540, 100)), 1),
    bone("shin.r", THIGH_R, v((80, 100), (-90, 100), (20, 100)), 1),
];

/// The left and right bones a mirrored clip swaps.
pub const MIRROR: [(usize, usize); 4] = [
    (SHOULDER_L, SHOULDER_R),
    (FOREARM_L, FOREARM_R),
    (THIGH_L, THIGH_R),
    (SHIN_L, SHIN_R),
];

/// The head turns to keep you in view, a little: the whole front of the
/// dome is the head, so it is one bone.
pub const NECK_CHAIN: [usize; 1] = [bones::HEAD];

/// Which bone an attack's hit volume rides: the body, or the head.
pub const FOLLOWS_BODY: u8 = 0;
pub const FOLLOWS_HEAD: u8 = 1;

pub const FOLLOWS: [usize; 2] = [ROOT, bones::HEAD];

// ---------------------------------------------------------------------------
// Parts
// ---------------------------------------------------------------------------

pub const BELLY: usize = 0;
pub const FLANK_L: usize = 1;
/// A part of the rim, for the tools and tests that stand somebody on it.
pub const LEFT_RIM: usize = FLANK_L;
pub const FLANK_R: usize = 2;
pub const RUMP: usize = 3;
pub const BROW: usize = 4;
pub const DOME: usize = 5;
pub const CROWN: usize = 6;
pub const WART_1: usize = 7;
pub const WART_2: usize = 8;
pub const WART_3: usize = 9;
pub const WART_4: usize = 10;
pub const JAW_PART: usize = 11;
pub const SAC: usize = 12;
pub const STOMACH: usize = 13;
pub const ARM_L: usize = 14;
pub const ARM_R: usize = 15;
pub const HAND_L: usize = 16;
pub const HAND_R: usize = 17;
pub const HAUNCH_L: usize = 18;
pub const HAUNCH_R: usize = 19;
pub const FOOT_L: usize = 20;
pub const FOOT_R: usize = 21;

pub const PART_COUNT: usize = 22;

/// The four warts, front to back.
pub const WARTS: [usize; 4] = [WART_1, WART_2, WART_3, WART_4];

/// The parts its tar coat covers: everything but the warts, the sac and the
/// stomach. What `fight::hide` multiplies by the coat.
pub const fn coated(part: usize) -> bool {
    !matches!(part, WART_1 | WART_2 | WART_3 | WART_4 | SAC | STOMACH)
}

/// The toad's proportions, at scale one, in each bone's own frame. Heights
/// below are written as the world heights they come to standing (the root is
/// at 2.4 m, the head bone at the same height).
pub const PARTS: [Part; PART_COUNT] = [
    // The core of the body, under the dome: solid, nothing to stand on (the
    // dome covers its top), and the stomach is inside it. 0.6 m to 4.6 m.
    part(
        "belly",
        ROOT,
        v((-460, 100), (-180, 100), (-500, 100)),
        v((300, 100), (220, 100), (500, 100)),
        Knob::VulnHide as u16,
    ),
    // **The rim**: the flanks, the rump and the brow, each topping out at
    // 5.2 m. Consecutive treads overlap so the ring has no seams.
    part(
        "left flank",
        ROOT,
        v((-430, 100), (-160, 100), (-600, 100)),
        v((290, 100), (280, 100), (-410, 100)),
        Knob::VulnHide as u16,
    )
    .mountable(),
    part(
        "right flank",
        ROOT,
        v((-430, 100), (-160, 100), (410, 100)),
        v((290, 100), (280, 100), (600, 100)),
        Knob::VulnHide as u16,
    )
    .mountable(),
    part(
        "rump",
        ROOT,
        v((-540, 100), (-160, 100), (-470, 100)),
        v((-380, 100), (280, 100), (470, 100)),
        Knob::VulnHide as u16,
    )
    .mountable(),
    // The brow: the front of the rim, over the mouth. On the body rather than
    // the head, so the head nodding through a move does not take the rim
    // down with it: what opens the rim is the flop and the winding, by design.
    part(
        "brow",
        ROOT,
        v((260, 100), (20, 100), (-390, 100)),
        v((560, 100), (280, 100), (390, 100)),
        Knob::VulnHide as u16,
    )
    .mountable(),
    // The dome, a step up from the rim, and the crown a step up from it.
    part(
        "dome",
        ROOT,
        v((-400, 100), (-100, 100), (-440, 100)),
        v((280, 100), (340, 100), (440, 100)),
        Knob::VulnHide as u16,
    )
    .mountable(),
    part(
        "crown",
        ROOT,
        v((-280, 100), (0, 1), (-260, 100)),
        v((180, 100), (400, 100), (260, 100)),
        Knob::VulnHide as u16,
    )
    .mountable(),
    // **The warts**: its tar glands, along the crown, standing proud of it
    // to 7.0 m. Soft -- a rider walks over and through them -- breakable,
    // and each one burst takes a quarter of its tar for good.
    part(
        "front wart",
        ROOT,
        v((95, 100), (400, 100), (-45, 100)),
        v((185, 100), (460, 100), (45, 100)),
        Knob::VulnWart as u16,
    )
    .soft()
    .breakable(),
    part(
        "second wart",
        ROOT,
        v((-25, 100), (400, 100), (-45, 100)),
        v((65, 100), (460, 100), (45, 100)),
        Knob::VulnWart as u16,
    )
    .soft()
    .breakable(),
    part(
        "third wart",
        ROOT,
        v((-145, 100), (400, 100), (-45, 100)),
        v((-55, 100), (460, 100), (45, 100)),
        Knob::VulnWart as u16,
    )
    .soft()
    .breakable(),
    part(
        "back wart",
        ROOT,
        v((-265, 100), (400, 100), (-45, 100)),
        v((-175, 100), (460, 100), (45, 100)),
        Knob::VulnWart as u16,
    )
    .soft()
    .breakable(),
    // The lower jaw, 1.2 m to 2.6 m, the full width of the mouth. It drops
    // for the tongue and the belch.
    part(
        "jaw",
        JAW,
        v((-20, 100), (-140, 100), (-340, 100)),
        v((280, 100), (0, 1), (340, 100)),
        Knob::VulnHide as u16,
    ),
    // **The throat sac**, under the chin: tucked inside the jaw until it
    // inflates or belches, when it hangs out below it. Soft; its worth when
    // it is not shown is the jaw's (see `fight::hide`).
    part(
        "throat sac",
        THROAT,
        v((-20, 100), (-80, 100), (-120, 100)),
        v((160, 100), (40, 100), (120, 100)),
        Knob::VulnSac as u16,
    )
    .soft(),
    // **The stomach**: a hollow 2.4 m across inside the belly, 1.2 m to 3 m.
    // A swallowed fighter rides it from inside (`beast::Shape::hollow`).
    part(
        "stomach",
        ROOT,
        v((-120, 100), (-120, 100), (-120, 100)),
        v((120, 100), (60, 100), (120, 100)),
        Knob::VulnStomach as u16,
    )
    .hollow(),
    // The legs: stubby, armoured like the rest of the hide, and **soft** --
    // folded under the flanks, they are something to hit rather than a step:
    // solid, their low tops lifted a fighter walking into its side onto a
    // ledge it could neither stand on nor dodge from.
    part(
        "left arm",
        SHOULDER_L,
        v((-45, 100), (-85, 100), (-45, 100)),
        v((45, 100), (30, 100), (45, 100)),
        Knob::VulnLeg as u16,
    )
    .soft(),
    part(
        "right arm",
        SHOULDER_R,
        v((-45, 100), (-85, 100), (-45, 100)),
        v((45, 100), (30, 100), (45, 100)),
        Knob::VulnLeg as u16,
    )
    .soft(),
    part(
        "left hand",
        FOREARM_L,
        v((-50, 100), (-70, 100), (-60, 100)),
        v((90, 100), (10, 100), (60, 100)),
        Knob::VulnLeg as u16,
    )
    .soft(),
    part(
        "right hand",
        FOREARM_R,
        v((-50, 100), (-70, 100), (-60, 100)),
        v((90, 100), (10, 100), (60, 100)),
        Knob::VulnLeg as u16,
    )
    .soft(),
    part(
        "left haunch",
        THIGH_L,
        v((-120, 100), (-90, 100), (-60, 100)),
        v((80, 100), (60, 100), (60, 100)),
        Knob::VulnLeg as u16,
    )
    .soft(),
    part(
        "right haunch",
        THIGH_R,
        v((-120, 100), (-90, 100), (-60, 100)),
        v((80, 100), (60, 100), (60, 100)),
        Knob::VulnLeg as u16,
    )
    .soft(),
    part(
        "left foot",
        SHIN_L,
        v((-40, 100), (-70, 100), (-70, 100)),
        v((160, 100), (10, 100), (70, 100)),
        Knob::VulnLeg as u16,
    )
    .soft(),
    part(
        "right foot",
        SHIN_R,
        v((-40, 100), (-70, 100), (-70, 100)),
        v((160, 100), (10, 100), (70, 100)),
        Knob::VulnLeg as u16,
    )
    .soft(),
];

/// The four legs, front to back. None of its feet break: the consequence
/// this creature carries for the rest of the fight is its warts.
pub const LEGS: [Leg; 4] = [
    Leg {
        upper: ARM_L,
        foot: HAND_L,
        hip: SHOULDER_L,
        knee: FOREARM_L,
        front: true,
        side: -1,
    },
    Leg {
        upper: ARM_R,
        foot: HAND_R,
        hip: SHOULDER_R,
        knee: FOREARM_R,
        front: true,
        side: 1,
    },
    Leg {
        upper: HAUNCH_L,
        foot: FOOT_L,
        hip: THIGH_L,
        knee: SHIN_L,
        front: false,
        side: -1,
    },
    Leg {
        upper: HAUNCH_R,
        foot: FOOT_R,
        hip: THIGH_R,
        knee: SHIN_R,
        front: false,
        side: 1,
    },
];

// ---------------------------------------------------------------------------
// Moves
// ---------------------------------------------------------------------------

/// A glob of tar lobbed at where you will be: a pool where it lands.
pub const SPEW: u8 = 0;
/// A crouch, a leap to where you will be, a crash, and a ring of pools.
pub const FLOP: u8 = 1;
/// The tongue along a line; caught, you are swallowed.
pub const TONGUE: u8 = 2;
/// Somebody inside it. Never chosen: the tongue's catch puts it here.
pub const SWALLOW: u8 = 3;
/// Every pool near its mouth ignites, and a cone of flame in front.
pub const BELCH: u8 = 4;
/// A sheet of tar thrown backwards: tarred, not hurt.
pub const BACKWASH: u8 = 5;
/// It swells by half: a shove to anyone touching it, and the sac out.
pub const INFLATE: u8 = 6;
/// It rolls in its own tar and grinds the coat back.
pub const WALLOW: u8 = 7;
/// A Bulwark's shield in its throat. Never chosen.
pub const GAG: u8 = 8;

pub const MOVE_COUNT: usize = 9;

pub const MOVES: [MoveDecl; MOVE_COUNT] = [
    MoveDecl::new("Spew", Clip::Spew as usize).lobbed(),
    MoveDecl::new("Belly flop", Clip::Flop as usize).lobbed(),
    MoveDecl::new("Tongue", Clip::Tongue as usize).stops_at_aim(),
    MoveDecl::new("Swallow", Clip::Swallow as usize).never_chosen(),
    MoveDecl::new("Flint belch", Clip::Belch as usize),
    MoveDecl::new("Backwash", Clip::Backwash as usize).harmless(),
    MoveDecl::new("Inflate", Clip::Inflate as usize),
    MoveDecl::new("Wallow", Clip::Wallow as usize),
    MoveDecl::new("Gag", Clip::Gag as usize).never_chosen(),
];

// ---------------------------------------------------------------------------
// Clips
// ---------------------------------------------------------------------------

/// Everything the Mireback knows how to look like, in the order its baked
/// table lays them out. Authored in `crates/anim/src/beast/mireback/`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Clip {
    Idle,
    /// A heavy four-beat waddle, indexed by ground covered. It has no
    /// faster gait: it relocates by leaping.
    Walk,
    Spew,
    Flop,
    Tongue,
    Swallow,
    Belch,
    Backwash,
    Inflate,
    Wallow,
    Gag,
    /// Staggered: a backfire, a broken wallow, a flinch.
    Flinch,
    /// **Winded**: the sac torn, the body sagging, the rim down to 3.8 m.
    /// What the shared ladder calls a stumble.
    Winded,
    /// **Gutted**: on its back in its own fire. What the ladder calls a
    /// topple, and the big window.
    Gutted,
    Dead,
}

pub const CLIP_COUNT: usize = 15;

impl Clip {
    pub const ALL: [Clip; CLIP_COUNT] = [
        Clip::Idle,
        Clip::Walk,
        Clip::Spew,
        Clip::Flop,
        Clip::Tongue,
        Clip::Swallow,
        Clip::Belch,
        Clip::Backwash,
        Clip::Inflate,
        Clip::Wallow,
        Clip::Gag,
        Clip::Flinch,
        Clip::Winded,
        Clip::Gutted,
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
    attack("spew"),
    attack("flop"),
    attack("tongue"),
    attack("swallow"),
    attack("belch"),
    attack("backwash"),
    attack("inflate"),
    attack("wallow"),
    attack("gag"),
    once("flinch"),
    once("winded"),
    once("gutted"),
    once("dead"),
];

// ---------------------------------------------------------------------------
// Its own knobs
// ---------------------------------------------------------------------------

crate::species_knobs! {
    /// The knobs only the Mireback has: its hide, its tar and fire, its coat
    /// and sac, the swallow, the braziers, and its own terms in the brain.
    /// The design document's names for them (§5) are in the comment beside
    /// each, where they differ. A wart's health is every creature's
    /// "breakable part health".
    pub enum Knob;
    VulnHide,        "hide",    "Hide (x damage)",                      Fixed, 0, fx(3,1);
    VulnLeg,         "hide",    "Legs (x damage)",                      Fixed, 0, fx(3,1);
    VulnWart,        "hide",    "Warts (x damage)",                     Fixed, 0, fx(4,1);
    VulnSac,         "hide",    "Throat sac, shown (x damage)",         Fixed, 0, fx(4,1);
    VulnStomach,     "hide",    "Stomach (x damage)",                   Fixed, 0, fx(4,1);
    // coat_armour, coat_crack
    CoatArmour,      "coat",    "Hide with the coat whole (x)",         Fixed, 0, fx(1,1);
    CoatCrack,       "coat",    "Coat lost a second in fire",           Fixed, 0, fx(1,1);
    GuttedSoft,      "coat",    "Hide while gutted (x damage)",         Fixed, 0, fx(4,1);
    // burn_self, burn_self_pools
    BurnSelf,        "fire",    "Its own fire, a tick a pool",          Int,   0, 400;
    BurnSelfEvery,   "fire",    "Its own fire, frames a tick",          Frames,1, 120;
    BurnSelfPools,   "fire",    "Burning pools that count, at most",    Int,   0, 8;
    GuttedBurn,      "fire",    "Its own fire while gutted (x)",        Fixed, 0, fx(4,1);
    TarTail,         "tar",     "Tar's slow lingers for",               Frames,0, 120;
    TarredFrames,    "tar",     "Tarred for",                           Frames,0, 600;
    TarredSlow,      "tar",     "Tarred walk (x)",                      Fixed, 0, fx(1,1);
    PoolSpew,        "tar",     "A glob's pool, radius",                Fixed, 0, fx(6,1);
    PoolRing,        "tar",     "A flop's ring pools, radius",          Fixed, 0, fx(6,1);
    RingPools,       "tar",     "Pools in a flop's ring",               Int,   0, 8;
    RingInside,      "tar",     "Ring's edge inside its footprint",     Fixed, 0, fx(3,1);
    PoolBackwash,    "tar",     "Backwash's pool, radius",              Fixed, 0, fx(6,1);
    PoolMost,        "tar",     "Pools merge up to, radius",            Fixed, 0, fx(12,1);
    ShieldFront,     "tar",     "A blocked glob lands ahead of her",    Fixed, 0, fx(4,1);
    WartCoatCost,    "warts",   "A burst wart slows the coat by (x)",   Fixed, 0, fx(1,1);
    // tar_coop
    TarCoop,         "tar",     "More tar with two hunters (x)",        Fixed, fx(1,1), fx(3,1);
    // sac_poise
    SacPoise,        "sac",     "Sac tears at",                         Int,   0, 4000;
    // wallow_interrupt
    WallowInterrupt, "nerve",   "Wallow broken at (x interrupt)",       Fixed, 0, fx(1,1);
    StaggerFrames,   "nerve",   "Staggered for",                        Frames,0, 240;
    BackfireReach,   "belch",   "Backfire lights tar within",           Fixed, 0, fx(12,1);
    BelchReach,      "belch",   "Belch lights tar within",              Fixed, 0, fx(20,1);
    // swallow_frames, swallow_retch
    SwallowFrames,   "swallow", "Held inside for",                      Frames,0, 600;
    SwallowRetch,    "swallow", "Retches at stomach damage",            Int,   0, 4000;
    SwallowAcid,     "swallow", "Acid, a tick",                         Int,   0, 200;
    SwallowAcidEvery,"swallow", "Acid, frames a tick",                  Frames,1, 120;
    SwallowPull,     "swallow", "Pulled in over",                       Frames,1, 60;
    SpitDamage,      "swallow", "Spat out, damage",                     Int,   0, 400;
    SpitDistance,    "swallow", "Spat out, how far",                    Fixed, 0, fx(20,1);
    SpitSpeed,       "swallow", "Spat out, at",                         Fixed, 0, fx(40,1);
    HotRetch,        "swallow", "Hot mouthful, retches at",             Frames,0, 120;
    HotRetchDamage,  "swallow", "Hot mouthful, it takes",               Int,   0, 2000;
    // tongue_full
    TongueFull,      "swallow", "Tongue locked after a swallow",        Frames,0, 3600;
    FlopAir,         "flop",    "Airborne for the last",                Frames,0, 60;
    FlopHeight,      "flop",    "Leap height",                          Fixed, 0, fx(10,1);
    // brazier_relight
    BrazierRelight,  "brazier", "Relights after",                       Frames,0, 3600;
    CoalLength,      "brazier", "Coals, lane length",                   Fixed, 0, fx(12,1);
    CoalWidth,       "brazier", "Coals, lane half-width",               Fixed, 0, fx(4,1);
    BrazierSize,     "brazier", "Brazier, height and width",            Fixed, 0, fx(4,1);
    // tide_health
    TideHealth,      "mind",    "The tide below health (x)",            Fixed, 0, fx(1,1);
    TideSecond,      "mind",    "The tide's second glob, startup",      Frames,0, 60;
    TarReach,        "mind",    "Walks to its tar within, of you",      Fixed, 0, fx(30,1);
    // floor(m), kindle(m), coat(m), crowd(m), flee(m)
    FloorAppetite,   "mind",    "Floor, per clean point at the lead",   Int,   0, 400;
    FloorReach,      "mind",    "Floor, looks this far round the lead", Fixed, 0, fx(12,1);
    KindleAppetite,  "mind",    "Kindle, per target on tar",            Int,   0, 4000;
    KindleSelf,      "mind",    "Kindle, less its own tar under it",    Int,   0, 8000;
    CoatAppetite,    "mind",    "Coat, at no coat",                     Int,   0, 8000;
    CrowdAppetite,   "mind",    "Crowd, per strain from close",         Fixed, 0, fx(20,1);
    CrowdReach,      "mind",    "Crowd, close is within",               Fixed, 0, fx(12,1);
    CrowdFade,       "mind",    "Crowd, forgets over",                  Frames,1, 600;
    FleeAppetite,    "mind",    "Flee, fire near its tar",              Int,   0, 8000;
    FleeReach,       "mind",    "Flee, fire within, of its tar",        Fixed, 0, fx(20,1);
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
    id: SpeciesId::MIREBACK,
    name: "Mireback",
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
        gallop: Clip::Walk as usize,
        flinch: Clip::Flinch as usize,
        stumble: Clip::Winded as usize,
        topple: Clip::Gutted as usize,
        dead: Clip::Dead as usize,
    },
    span: &baked::SPAN,
    rows: baked::ROWS,
    row: |r| &baked::FRAMES[r],
    own: OWN,
    tuned: &tuned::KNOBS,
    tuned_path: "crates/sim/src/species/mireback/tuned.rs",
    pack: None,
    fight: &fight::FIGHT,
};
