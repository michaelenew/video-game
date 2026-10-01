//! The Hornback herd: eight grazing cows and the bull that defends them.
//!
//! The cast's lesson in **using the arena** (`docs/design/creatures/hornback.md`):
//! the bull's charge ends in a stun when it meets something solid, the herd's
//! stampede is a moving wall that splits round those same solids, and the
//! rocks are spent as the fight goes on. It is built on the critter machinery
//! (`crate::critter`, `crate::pack`, `docs/design/critters.md`) -- **the bull
//! is a critter with extra state**, not a light monster (§10) -- and
//! everything here is what the generic pack does not already do:
//!
//! - **The table** (this file): nine moves, two kinds -- the cow and the bull
//!   -- the pack that musters eight and one, the boulders as hazard kinds, the
//!   cart for the crossing, and the species' own knobs.
//! - **The mind** ([`mind`]): what the bull wants (the scoring of §5, with
//!   its wary, downed, seen-swing and herd-home terms), where every body goes
//!   (interpose, graze, bunch, run the lane, come home, rally), the guard and
//!   the horns, strain at critter scale, and what each hit does besides its
//!   damage.
//! - **The rules** ([`rules`]): the species' frame over the whole world --
//!   the boulders laid and cracked, the herd's states, the lane and its lees
//!   as a hard constraint, the charge's swept stop on solids, the stun, cows
//!   and the bull as bodies a fighter cannot walk through, the ride and its
//!   buck, the calm-down, and the coop numbers.
//!
//! Its own state is the pack's memo ([`mind::word`]) and a few words of the
//! hunt's lore ([`rules::word`]); the boulders are hazard cells.

mod mind;
pub mod ride;
pub mod rules;
mod tuned;

pub use mind::word as memo;
pub use mind::{
    HerdState, Mind, bellow_lane, charge_lane, guard_arc, herd_state, horn_health, horns_whole,
    in_lane, in_lee, stunned, wary_of,
};

use crate::arena::Material;
use crate::critter::{CritterKind, CritterMove, pose};
use crate::hazard::HazardDecl;
use crate::lore::Layout;
use crate::objective::ObjectiveDecl;
use crate::oven::KnobDecl;
use crate::pack::PackDecl;
use crate::species::{FightDecl, MoveDecl, Species, SpeciesId};

/// The moves, by index: the bull's six, then the cow's three.
pub const CHARGE: u8 = 0;
pub const HOOK: u8 = 1;
pub const TRAMPLE: u8 = 2;
pub const SHOULDER: u8 = 3;
pub const GUARD: u8 = 4;
pub const BELLOW: u8 = 5;
pub const KICK: u8 = 6;
pub const BUCK: u8 = 7;
pub const STAMPEDE: u8 = 8;

/// A critter has no clips, so each move's clip is the stock pose it is drawn
/// with (`critter::pose`): the silhouettes §6 asks for -- the head is the
/// tell. Up and turned to you watching; dropped and level for the charge,
/// after two heavy paws; dropped and cocked for the hook; reared for the
/// trample; leaning away for the shoulder; low and square in the guard;
/// thrown back for the bellow.
pub const MOVES: [MoveDecl; 9] = [
    MoveDecl::new("Paw & charge", pose::PAW),
    MoveDecl::new("Hook", pose::HOOK),
    MoveDecl::new("Trample", pose::REAR),
    MoveDecl::new("Shoulder", pose::LEAN),
    MoveDecl::new("Guard", pose::BRACE),
    MoveDecl::new("Bellow", pose::BELLOW),
    MoveDecl::new("Cow kick", pose::KICK),
    MoveDecl::new("Buck", pose::BUCK),
    MoveDecl::new("Stampede", pose::GALLOP),
];

/// The kinds, by index.
pub const COW: u8 = 0;
pub const BULL: u8 = 1;

/// None of the herd's moves takes a token: the bull is one animal deciding,
/// and a cow's kick is its own decision about what is behind it (§5). The
/// stampede and the buck are thrown by the species' rules, not chosen.
pub const KINDS: [CritterKind; 2] = [
    CritterKind {
        name: "Cow",
        moves: &[
            CritterMove::free(KICK),
            CritterMove::free(BUCK).committed(),
            CritterMove::free(STAMPEDE).committed(),
        ],
        role: 0,
        yields: false,
        mountable: true,
    },
    CritterKind {
        name: "Bull",
        moves: &[
            CritterMove::free(CHARGE).committed(),
            CritterMove::free(HOOK),
            CritterMove::free(TRAMPLE),
            CritterMove::free(SHOULDER),
            CritterMove::free(GUARD),
            CritterMove::free(BELLOW).committed(),
        ],
        role: 0,
        yields: false,
        mountable: false,
    },
];

/// The bull leads, in the pack's sense: a herd that loses it leaves by the
/// ford (§1, step 6), which is `LeaderRouts`.
pub static PACK: PackDecl = PackDecl {
    kinds: &KINDS,
    muster: &[(BULL, 1), (COW, 8)],
    leader: Some(BULL),
    mind: &Mind,
};

/// The boulders, as the hazard kinds they are: a boulder, the same boulder
/// cracked by one charge, and the rubble a second leaves -- low enough to
/// walk over, and no longer a solid.
pub const BOULDER: u8 = 0;
pub const CRACKED: u8 = 1;
pub const RUBBLE: u8 = 2;

pub const HAZARDS: [HazardDecl; 3] = [
    HazardDecl::disc("boulder")
        .reaching(0)
        .solid(Material::Rock),
    HazardDecl::disc("cracked boulder")
        .reaching(0)
        .solid(Material::Rock),
    HazardDecl::disc("rubble")
        .reaching(0)
        .made_of(Material::Stone),
];

/// The crossing's cart (§11): it stands at the crossing arena's road and
/// nowhere else -- the meadow has no sites, so a hunt there has no cart.
pub const CART: usize = 0;

pub const OBJECTIVES: [ObjectiveDecl; 1] = [ObjectiveDecl::cart("cart", 0)];

crate::species_knobs! {
    /// The Hornback's own numbers: everything §2–§8 names that is not already
    /// a pack knob (`pack::PackKnob`), a body's (`critter::CritterField`) or
    /// a move's row.
    pub enum Knob;
    WatchRadius,     "herd",     "Bull watches a fighter within",         Fixed,  0,  fx(40,1);
    CalmRadius,      "herd",     "Calms when every fighter is past",      Fixed,  0,  fx(60,1);
    CalmFrames,      "herd",     "...for this long",                      Frames, 0,  3600;
    Standoff,        "herd",     "Bull stands this short of its target",  Fixed,  0,  fx(20,1);
    TrotBeyond,      "herd",     "Bull trots to close past",              Fixed,  0,  fx(40,1);
    BullWalk,        "herd",     "Bull's walk",                           Fixed,  0,  fx(20,1);
    HerdBehind,      "herd",     "Herd grazes this far behind the bull",  Fixed,  0,  fx(20,1);
    GrazeWander,     "herd",     "Grazing wanders this far",              Fixed,  0,  fx(12,1);
    Cohesion,        "herd",     "Cohesion, grazing",                     Fixed,  0,  fx(4,1);
    CohesionRun,     "herd",     "Cohesion, running",                     Fixed,  0,  fx(4,1);
    AvoidFighters,   "herd",     "Cows keep this far from a fighter",     Fixed,  0,  fx(8,1);
    RallyRadius,     "herd",     "Rallied, mills round the bull at",      Fixed,  0,  fx(12,1);
    RallyBelow,      "herd",     "Rallies below this health",             Percent, 0, 100;
    CowNerve,        "herd",     "A cow bolts after taking",              Int,    0,  2000;
    LaneWidth,       "stampede", "Lane width",                            Fixed,  0,  fx(20,1);
    LeeLength,       "stampede", "A solid's lee runs downstream",         Fixed,  0,  fx(12,1);
    RunFrames,       "stampede", "The run lasts at most",                 Frames, 0,  600;
    BellowLockout,   "stampede", "Bellow locked after the herd is home",  Frames, 0,  3600;
    BellowAppetite,  "stampede", "Bellow wants a target in the open",     Int,    0,  4000;
    OpenWithin,      "stampede", "In the open: no solid within",          Fixed,  0,  fx(12,1);
    HomeWithin,      "stampede", "The herd is home within",               Fixed,  0,  fx(20,1);
    TrampleFallen,   "stampede", "Cows step this far round the fallen",   Fixed,  0,  fx(6,1);
    Knockdown,       "stampede", "The first cow knocks down for",         Frames, 0,  240;
    ChargeRun,       "charge",   "Runs this far",                         Fixed,  0,  fx(40,1);
    DesperateRun,    "charge",   "Runs this far, desperate",              Fixed,  0,  fx(40,1);
    EdgeMargin,      "charge",   "Pulls up this short of the edge",       Fixed,  0,  fx(10,1);
    StunFrames,      "charge",   "Stunned by a solid for",                Frames, 0,  600;
    StunOccupied,    "charge",   "...by a solid somebody stands on",      Frames, 0,  600;
    ShakeOffDamage,  "charge",   "Shakes off whoever stands on it for",   Int,    0,  400;
    ShakeOffStagger, "charge",   "...and staggers them",                  Frames, 0,  120;
    ChargeKnockdown, "charge",   "Knocks down for",                       Frames, 0,  240;
    WaryFrames,      "charge",   "Wary of the solid it hit for",          Frames, 0,  3600;
    WaryRadius,      "charge",   "Wary of a lane passing within",         Fixed,  0,  fx(8,1);
    ChargeLockout,   "charge",   "No charge after a stun for",            Frames, 0,  600;
    ChainPaw,        "charge",   "Desperate, a missed charge chains with a paw of", Frames, 0, 120;
    ChainAppetite,   "charge",   "Chains this often (%)",                 Percent, 0, 100;
    StoneDamage,     "charge",   "A lit stone burns it for",              Int,    0,  900;
    ParryStagger,    "charge",   "A parried charge staggers it",          Frames, 0,  300;
    BlockPush,       "charge",   "A blocked charge pushes back",          Fixed,  0,  fx(12,1);
    BlockStun,       "charge",   "...with a stunlock of",                 Frames, 0,  240;
    HookKnockdown,   "hook",     "Lands as a knockdown of",               Frames, 0,  240;
    TrampleAfter,    "trample",  "Lands no sooner after you can move",    Frames, 0,  120;
    TrampleNear,     "trample",  "Wants a fallen fighter within",         Fixed,  0,  fx(10,1);
    TrampleWant,     "trample",  "Appetite for a fallen fighter",         Int,    0,  4000;
    GuardSees,       "guard",    "Braces at a swing seen within",         Fixed,  0,  fx(10,1);
    GuardArc,        "guard",    "Covers this far off its nose (cos)",    Fixed,  fx(-1,1), fx(1,1);
    GuardWant,       "guard",    "Appetite for a swing it saw",           Int,    0,  4000;
    GuardRecoil,     "guard",    "The attacker recoils for",              Frames, 0,  120;
    GuardPush,       "guard",    "...pushed back",                        Fixed,  0,  fx(6,1);
    GuardSlide,      "guard",    "The bull slides back",                  Fixed,  0,  fx(4,1);
    GuardBroken,     "guard",    "A broken guard staggers it for",        Frames, 0,  240;
    GuardLockout,    "guard",    "No guard after a break for",            Frames, 0,  600;
    ComboFrames,     "guard",    "Wants a hook after a turned blow for",  Frames, 0,  240;
    HeadHigh,        "horns",    "The head's top, standing",              Fixed,  0,  fx(6,1);
    HeadLow,         "horns",    "The head's top, stunned",               Fixed,  0,  fx(6,1);
    HeadDepth,       "horns",    "The head, nose to poll",                Fixed,  0,  fx(4,1);
    HeadSpan,        "horns",    "The head, across the horns",            Fixed,  0,  fx(4,1);
    ShoulderFlank,   "shoulder", "Throws at a fighter at its flank within", Fixed, 0, fx(8,1);
    ShoulderFlinch,  "shoulder", "A hit in the lean flinches it for",     Frames, 0,  120;
    KickArc,         "kick",     "Rear wedge (cos off the tail)",         Fixed,  fx(-1,1), fx(1,1);
    KickReach,       "kick",     "Rear wedge reaches",                    Fixed,  0,  fx(8,1);
    CowGlance,       "kick",     "A cow glances every",                   Frames, 1,  120;
    RidePatience,    "ride",     "The first buck after",                  Frames, 0,  600;
    SecondBuck,      "ride",     "The second buck after the first",       Frames, 0,  600;
    BuckPitch,       "ride",     "First buck's pitch (turns)",            Fixed,  0,  fx(1,4);
    BuckHeave,       "ride",     "First buck's heave",                    Fixed,  0,  fx(4,1);
    SecondHarder,    "ride",     "The second is harder by (x)",           Fixed,  0,  fx(4,1);
    BuckDamage,      "ride",     "A buck's fall costs",                   Int,    0,  400;
    CoopHealth,      "coop",     "Bull's health for two hunters (x)",     Fixed,  fx(1,1), fx(3,1);
    CartLanePeriod,  "crossing", "A migration wave every",                Frames, 0,  3600;
    CartLaneWarn,    "crossing", "Drawn on the road this long before",    Frames, 0,  900;
}

/// Raw helper for the knob bounds above.
const fn fx(num: i32, den: i32) -> i32 {
    crate::fixed::Fx::ratio(num, den).raw()
}

const OWN: &[KnobDecl] = Knob::DECLS;

/// One of the Hornback's own knobs, live.
pub fn knob(k: Knob) -> i32 {
    SPECIES.own_raw(k as usize)
}

/// The same, as fixed point.
pub fn knob_fx(k: Knob) -> crate::fixed::Fx {
    SPECIES.own_fx(k as usize)
}

/// What the herd brings to the fight besides its bodies: four boulders (and
/// what they become), the cart where the arena has a road for it, and its own
/// words of lore ([`rules::word`]); the frame hook that runs everything that
/// touches more than the pack.
pub static FIGHT: FightDecl = FightDecl {
    layout: Layout {
        hazards: 4,
        noises: 0,
        objectives: 1,
        own: 3,
    },
    hazards: &HAZARDS,
    objectives: &OBJECTIVES,
    frame: Some(rules::frame),
    ..FightDecl::PLAIN
};

pub static SPECIES: Species = Species::pack_only(
    SpeciesId::HORNBACK,
    "Hornback",
    &MOVES,
    OWN,
    &tuned::KNOBS,
    "crates/sim/src/species/hornback/tuned.rs",
    &PACK,
)
.fighting(&FIGHT);
