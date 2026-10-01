//! The Ridgeback: the first species, and the one the machinery was built for.
//!
//! Thirteen metres of armoured animal with two soft places on it, both on top.
//! The fight is about earning your way up there and staying up there. See
//! `docs/design/monsters.md` for what the fight is and why; this file is only
//! what the animal *is*: its skeleton, its parts, its legs, its moves and its
//! clips, and the knobs no other creature has.
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
//! Two generated files sit beside this one: `baked.rs`, its pose table
//! (`cargo run -p anim --bin bake_beast`), and `tuned.rs`, its knobs
//! (`cargo run -p sim --bin bake_tuning`).

pub(crate) mod baked;
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

/// The Ridgeback's bones, by index.
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
/// A constant rather than a knob for the same reason the part boxes are: this
/// is the animal's *shape*, it never changes during a fight, and the number
/// you actually reach for while playing -- how big is it -- is its `Size`
/// knob, which multiplies all of it.
///
/// The numbers make an animal about **thirteen metres nose to tail, standing
/// five and a half metres at the back on long legs**. The height is in the legs
/// rather than in the bulk on purpose, and both halves of that are load-bearing:
/// the back is above every full hop but one and the legs are the only part of
/// it a fighter on the floor can reach, which is what makes the ground game a
/// route up rather than a chore. See `docs/design/monsters.md` §1.
///
/// **A metre taller since 2026-09-23.** At 3.95 m hips the tail's middle sat at
/// 3.4 m and four of six classes walked up it from the floor, so the climb was
/// free for most of the roster. The metre went into the legs -- half into each
/// segment -- and the collapse clips went a metre deeper to match, so every
/// *earned* route (a stumble, a topple, the slam's crash) lands where it did.
///
/// Parents always come before their children. Which side each bone is on is
/// the last column: the mirror lives in the skeleton rather than in the
/// animation, exactly as it does for the fighters.
pub const BONES: [Bone; COUNT] = [
    bone("root", NO_PARENT, v((-16, 10), (495, 100), (0, 1)), 0), // the hips, 4.95 m up
    bone("spine", ROOT, v((135, 100), (12, 100), (0, 1)), 0),
    bone("chest", SPINE, v((145, 100), (5, 100), (0, 1)), 0),
    bone("neck", CHEST, v((100, 100), (30, 100), (0, 1)), 0),
    bone("neck2", NECK, v((115, 100), (18, 100), (0, 1)), 0),
    bone("head", NECK2, v((95, 100), (-5, 100), (0, 1)), 0),
    // The tail is **carried level** off the hips and only the last two
    // segments droop. It used to droop from the root, which put its base a
    // hand's width inside a full hop and its middle well inside one: the free
    // route up, for most of the roster. Carried level, the tail is a tread you
    // reach from the haunch or from a platform, not from the floor.
    bone("tail1", ROOT, v((-125, 100), (5, 100), (0, 1)), 0),
    bone("tail2", TAIL1, v((-130, 100), (-5, 100), (0, 1)), 0),
    bone("tail3", TAIL2, v((-125, 100), (-45, 100), (0, 1)), 0),
    bone("tail4", TAIL3, v((-115, 100), (-60, 100), (0, 1)), 0),
    // Forelegs, off the chest. Long: the animal's height is in its legs rather
    // than in its bulk, which is what puts its back out of reach without
    // making it so long that the arena stops having room for a fight.
    bone(
        "shoulder.l",
        CHEST,
        v((-10, 100), (-60, 100), (-95, 100)),
        -1,
    ),
    bone("forearm.l", SHOULDER_L, v((0, 1), (-225, 100), (0, 1)), -1),
    bone("shoulder.r", CHEST, v((-10, 100), (-60, 100), (95, 100)), 1),
    bone("forearm.r", SHOULDER_R, v((0, 1), (-225, 100), (0, 1)), 1),
    // Hindlegs, off the hips. Heavier and set wider: this is where the animal
    // pushes from.
    bone("thigh.l", ROOT, v((-25, 100), (-55, 100), (-92, 100)), -1),
    bone("shin.l", THIGH_L, v((15, 100), (-220, 100), (0, 1)), -1),
    bone("thigh.r", ROOT, v((-25, 100), (-55, 100), (92, 100)), 1),
    bone("shin.r", THIGH_R, v((15, 100), (-220, 100), (0, 1)), 1),
];

/// The left and right bones a mirrored clip swaps.
pub const MIRROR: [(usize, usize); 4] = [
    (SHOULDER_L, SHOULDER_R),
    (FOREARM_L, FOREARM_R),
    (THIGH_L, THIGH_R),
    (SHIN_L, SHIN_R),
];

/// The bones the head's tracking is spread across, base to tip.
pub const NECK_CHAIN: [usize; 3] = [NECK, NECK2, bones::HEAD];

/// Which bone an attack's hit volume rides. The numbering is the move table's
/// own, kept because it is edited in the Oven as an integer.
pub const FOLLOWS_BODY: u8 = 0;
pub const FOLLOWS_TAIL: u8 = 1;
pub const FOLLOWS_HEAD: u8 = 2;

/// The bone each `Follows` number means.
pub const FOLLOWS: [usize; 3] = [ROOT, TAIL3, bones::HEAD];

// ---------------------------------------------------------------------------
// Parts
// ---------------------------------------------------------------------------

pub const HEAD: usize = 0;
pub const NECK_PART: usize = 1;
pub const NAPE: usize = 2;
pub const SHOULDERS: usize = 3;
pub const RIDGE: usize = 4;
pub const BARREL: usize = 5;
pub const HAUNCH: usize = 6;
pub const TAIL_BASE: usize = 7;
pub const TAIL_MID: usize = 8;
pub const TAIL_TIP: usize = 9;
pub const FORELEG_L: usize = 10;
pub const FORELEG_R: usize = 11;
pub const HINDLEG_L: usize = 12;
pub const HINDLEG_R: usize = 13;
pub const FOREFOOT_L: usize = 14;
pub const FOREFOOT_R: usize = 15;
pub const HINDFOOT_L: usize = 16;
pub const HINDFOOT_R: usize = 17;

pub const PART_COUNT: usize = 18;

/// The creature's proportions, at scale one.
pub const PARTS: [Part; PART_COUNT] = [
    // Head: armoured, and the bite's hitbox.
    part(
        "head",
        bones::HEAD,
        v((-15, 100), (-50, 100), (-52, 100)),
        v((125, 100), (45, 100), (52, 100)),
        Knob::VulnHead as u16,
    ),
    // Neck: armoured, and between it and the head a jump-in to the back from
    // the front has nowhere to land.
    part(
        "neck",
        NECK2,
        v((-5, 100), (-42, 100), (-44, 100)),
        v((100, 100), (30, 100), (44, 100)),
        Knob::VulnNeck as u16,
    ),
    // **The nape.** The base of the neck, where the plates part to let the head
    // turn. Unarmoured, and reachable only from the animal's own shoulders --
    // which is what the climb buys you. Set at the same height above the
    // shoulders that the ridge sits above the back, and for the same reason.
    // See `docs/design/monsters.md` §1.
    part(
        "nape",
        NECK,
        v((5, 100), (20, 100), (-36, 100)),
        v((105, 100), (78, 100), (36, 100)),
        Knob::VulnNape as u16,
    )
    .soft()
    .weak_point(),
    // The shoulders. Mountable, and the step between the back and the nape.
    part(
        "shoulders",
        CHEST,
        v((-25, 100), (-80, 100), (-122, 100)),
        v((105, 100), (55, 100), (122, 100)),
        Knob::VulnShoulder as u16,
    )
    .mountable(),
    // **The ridge**: the soft strip along the spine, and the thing the animal is
    // named for. It stands proud of the back rather than lying flush with it,
    // which is not decoration -- a target at a rider's ankles is a target a
    // level swing passes straight over, and the swing's pitch is dead-zoned
    // through the first forty-five degrees below the horizon on purpose (see
    // `docs/design/aiming.md`). Waist height on somebody standing beside it is
    // what makes hitting it the ordinary thing to do rather than a trick.
    part(
        "ridge",
        SPINE,
        v((-55, 100), (50, 100), (-38, 100)),
        v((135, 100), (118, 100), (38, 100)),
        Knob::VulnRidge as u16,
    )
    .soft()
    .weak_point(),
    part(
        "barrel",
        SPINE,
        v((-115, 100), (-85, 100), (-132, 100)),
        v((140, 100), (55, 100), (132, 100)),
        Knob::VulnBarrel as u16,
    )
    .mountable(),
    part(
        "haunch",
        ROOT,
        v((-105, 100), (-80, 100), (-124, 100)),
        v((60, 100), (58, 100), (124, 100)),
        Knob::VulnHaunch as u16,
    )
    .mountable(),
    // The tail. Mountable along its first two segments, so a rider who comes up
    // the back of the animal has somewhere to walk.
    //
    // **Consecutive mountable parts overlap along `x`** -- tail into haunch,
    // haunch into barrel, barrel into shoulders. Boxes that merely met would
    // leave a seam a few centimetres wide with nothing mountable in it, and a
    // rider walking forward would step into it and fall off an animal that
    // looks continuous. The climb is a staircase, so the treads have to touch.
    part(
        "tail",
        TAIL1,
        v((-125, 100), (-46, 100), (-62, 100)),
        v((45, 100), (42, 100), (62, 100)),
        Knob::VulnTail as u16,
    )
    .mountable(),
    part(
        "tail, middle",
        TAIL2,
        v((-125, 100), (-36, 100), (-46, 100)),
        v((35, 100), (34, 100), (46, 100)),
        Knob::VulnTailMid as u16,
    )
    .mountable(),
    // The tip covers the last two segments: it is the sweep's hitbox and
    // nothing stands on it.
    part(
        "tail tip",
        TAIL3,
        v((-240, 100), (-28, 100), (-34, 100)),
        v((5, 100), (26, 100), (34, 100)),
        Knob::VulnTailTip as u16,
    ),
    // Upper legs. Armoured, and not what breaks.
    part(
        "left foreleg",
        SHOULDER_L,
        v((-36, 100), (-232, 100), (-38, 100)),
        v((38, 100), (30, 100), (38, 100)),
        Knob::VulnLeg as u16,
    ),
    part(
        "right foreleg",
        SHOULDER_R,
        v((-36, 100), (-232, 100), (-38, 100)),
        v((38, 100), (30, 100), (38, 100)),
        Knob::VulnLeg as u16,
    ),
    part(
        "left hindleg",
        THIGH_L,
        v((-42, 100), (-226, 100), (-42, 100)),
        v((42, 100), (34, 100), (42, 100)),
        Knob::VulnLeg as u16,
    ),
    part(
        "right hindleg",
        THIGH_R,
        v((-42, 100), (-226, 100), (-42, 100)),
        v((42, 100), (34, 100), (42, 100)),
        Knob::VulnLeg as u16,
    ),
    // **The feet.** Soft, low, and the only thing on the animal a fighter
    // standing on the floor can reach without a jump. Breaking one drops that
    // corner and puts the creature on its knee, which is the ground game's
    // whole reward -- see `monster::Monster::break_a_leg`.
    part(
        "left forefoot",
        FOREARM_L,
        v((-32, 100), (-227, 100), (-36, 100)),
        v((52, 100), (20, 100), (36, 100)),
        Knob::VulnFoot as u16,
    )
    .breakable(),
    part(
        "right forefoot",
        FOREARM_R,
        v((-32, 100), (-227, 100), (-36, 100)),
        v((52, 100), (20, 100), (36, 100)),
        Knob::VulnFoot as u16,
    )
    .breakable(),
    part(
        "left hindfoot",
        SHIN_L,
        v((-34, 100), (-222, 100), (-38, 100)),
        v((50, 100), (20, 100), (38, 100)),
        Knob::VulnFoot as u16,
    )
    .breakable(),
    part(
        "right hindfoot",
        SHIN_R,
        v((-34, 100), (-222, 100), (-38, 100)),
        v((50, 100), (20, 100), (38, 100)),
        Knob::VulnFoot as u16,
    )
    .breakable(),
];

/// The four feet, in the order [`LEGS`] walks them.
pub const FEET: [usize; 4] = [FOREFOOT_L, FOREFOOT_R, HINDFOOT_L, HINDFOOT_R];

/// The four legs, front to back.
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

pub const BITE: u8 = 0;
pub const STOMP: u8 = 1;
pub const SWEEP: u8 = 2;
pub const CHARGE: u8 = 3;
pub const SLAM: u8 = 4;
pub const SHAKE: u8 = 5;
/// Both hind legs, straight back. **The answer to standing at the tail root.**
/// Every forward move needs you in front of it and the sweep is a whip about
/// the hips, so the patch of floor directly behind them was outside every
/// hit volume it had -- a place to stand and wail on a hind foot, which the
/// design document had called the ground game's station and a player called a
/// safe spot. The kick is aimed at exactly that patch, and its answer is a
/// sidestep rather than the sweep's jump, so being behind it is now two
/// questions rather than none.
pub const KICK: u8 = 6;
/// The tail curls over the back and flings a spray of spikes forward that
/// **roots** whoever it catches. The long-range answer, and the one move
/// whose hit leaves the animal: it exists for the fighter who stands at the
/// edge of everything else's reach and backpedals, and what it sets up is the
/// charge. See `docs/design/monsters.md` §"Threat modes".
pub const SPRAY: u8 = 7;

pub const MOVE_COUNT: usize = 8;

pub const MOVES: [MoveDecl; MOVE_COUNT] = [
    MoveDecl::new("Bite", Clip::Bite as usize),
    MoveDecl::new("Stomp", Clip::Stomp as usize),
    // The sweep goes to whichever side the target is on. A whip that only
    // ever went to the creature's right left its left flank a place the
    // sweep's own cone said it covered and its volume never reached.
    MoveDecl::new("Tail sweep", Clip::Sweep as usize).mirrors_to_target_side(),
    MoveDecl::new("Charge", Clip::Charge as usize),
    MoveDecl::new("Rear and slam", Clip::Slam as usize),
    // The one clip whose *violence* is a gameplay number rather than a look:
    // it is what decides whether a braced rider stays on. Everything else about
    // the animation is content; this is a knob.
    MoveDecl::new("Shake", Clip::Shake as usize).scaled_by(Knob::ShakeForce as u16),
    MoveDecl::new("Back kick", Clip::Kick as usize),
    MoveDecl::new("Spike spray", Clip::Spray as usize),
];

// ---------------------------------------------------------------------------
// Clips
// ---------------------------------------------------------------------------

/// Everything the Ridgeback knows how to look like, in the order its baked
/// table lays them out.
///
/// The motion is authored in `crates/anim/src/beast/ridgeback/` and baked into
/// `baked.rs` beside this file.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Clip {
    /// Standing. Loops, and breathes.
    Idle,
    /// One full two-beat of the walk. Loops, and is indexed by **ground
    /// covered** rather than by time.
    Walk,
    /// The gallop. A separate cycle rather than a faster walk, because a
    /// quadruped at speed does not use the same footfall order.
    Gallop,
    Bite,
    Stomp,
    Sweep,
    Charge,
    Slam,
    Shake,
    /// Both hind legs, straight back.
    Kick,
    /// The tail curls over the back and flings its spikes forward.
    Spray,
    Flinch,
    /// Down on a knee. What a broken leg and a landed crowd-control both
    /// produce, and the way onto its back from the floor.
    Stumble,
    Topple,
    Dead,
}

pub const CLIP_COUNT: usize = 15;

impl Clip {
    pub const ALL: [Clip; CLIP_COUNT] = [
        Clip::Idle,
        Clip::Walk,
        Clip::Gallop,
        Clip::Bite,
        Clip::Stomp,
        Clip::Sweep,
        Clip::Charge,
        Clip::Slam,
        Clip::Shake,
        Clip::Kick,
        Clip::Spray,
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

    pub const fn looping(self) -> bool {
        CLIPS[self as usize].looping
    }

    pub const fn phased(self) -> bool {
        CLIPS[self as usize].phased
    }

    /// The clip an attack plays.
    pub const fn of_move(kind: u8) -> Clip {
        Clip::ALL[MOVES[if (kind as usize) < MOVE_COUNT {
            kind as usize
        } else {
            MOVE_COUNT - 1
        }]
        .clip]
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
    cycle("gallop"),
    attack("bite"),
    attack("stomp"),
    attack("sweep"),
    attack("charge"),
    attack("slam"),
    attack("shake"),
    attack("kick"),
    attack("spray"),
    once("flinch"),
    once("stumble"),
    once("topple"),
    once("dead"),
];

// ---------------------------------------------------------------------------
// Its own knobs
// ---------------------------------------------------------------------------

crate::species_knobs! {
    /// The knobs only the Ridgeback has.
    ///
    /// **The hide** is a damage multiplier per part: below one is armour,
    /// above one is a weak point, and the spread between them is the
    /// difference between a twenty-second fight and a two-minute one. Three of
    /// these are the design in miniature. The **nape** is the highest number
    /// on the animal and is reachable only from its own shoulders. The
    /// **ridge** is the other weak point and is what a rider walks to first.
    /// The **foot** is the softest thing a fighter standing on the floor can
    /// reach at all, which is what makes the ground game a route up rather
    /// than a chore.
    ///
    /// **The shake force** is how violent the shake is, as a multiplier on the
    /// baked clip: the one animation number that is genuinely tuning rather
    /// than content, because it decides whether a braced rider stays on.
    pub enum Knob;
    VulnHead,     "hide", "Head (x damage)",         Fixed, 0, fx(3,1);
    VulnNeck,     "hide", "Neck (x damage)",         Fixed, 0, fx(3,1);
    VulnNape,     "hide", "Nape (x damage)",         Fixed, 0, fx(4,1);
    VulnShoulder, "hide", "Shoulders (x damage)",    Fixed, 0, fx(3,1);
    VulnBarrel,   "hide", "Barrel (x damage)",       Fixed, 0, fx(3,1);
    VulnRidge,    "hide", "Ridge (x damage)",        Fixed, 0, fx(4,1);
    VulnHaunch,   "hide", "Haunch (x damage)",       Fixed, 0, fx(3,1);
    VulnTail,     "hide", "Tail (x damage)",         Fixed, 0, fx(3,1);
    VulnTailMid,  "hide", "Tail, middle (x damage)", Fixed, 0, fx(3,1);
    VulnTailTip,  "hide", "Tail tip (x damage)",     Fixed, 0, fx(3,1);
    VulnLeg,      "hide", "Upper leg (x damage)",    Fixed, 0, fx(3,1);
    VulnFoot,     "hide", "Foot (x damage)",         Fixed, 0, fx(3,1);
    ShakeForce,   "pose", "Shake force (x)",         Fixed, 0, fx(3,1);
}

const OWN: &[KnobDecl] = Knob::DECLS;

// ---------------------------------------------------------------------------
// The species
// ---------------------------------------------------------------------------

pub static SPECIES: Species = Species {
    id: SpeciesId::RIDGEBACK,
    name: "Ridgeback",
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
        gallop: Clip::Gallop as usize,
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
    tuned_path: "crates/sim/src/species/ridgeback/tuned.rs",
    pack: None,
    fight: &crate::species::FightDecl::PLAIN,
};
