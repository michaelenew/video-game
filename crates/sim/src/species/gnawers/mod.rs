//! The Gnawers: six knee-high biters and the one they follow.
//!
//! The first creature that is a pack rather than a body
//! (`docs/design/creatures/gnawers.md`). Nothing in it can hurt you much; the
//! pack can. It is built on the critter machinery (`crate::critter`,
//! `crate::pack`, `docs/design/critters.md`), and everything here is what the
//! generic pack does not already do:
//!
//! - **The table** (this file): seven moves, two kinds -- the gnawer and the
//!   Big One -- and the pack that musters six and one.
//! - **The mind** ([`mind`]): what each body wants (the hamstring only at a
//!   back, the pile-on only at somebody slowed or down, the howl only when it
//!   is worth it), where it goes (a tightening ring for the pile-on, the Big
//!   One's hang-back and its short scatter, a wider ring under somebody out of
//!   reach), and what a bite does besides its damage (the hamstring's slow and
//!   latch, the pile-on's knockdown).
//! - **The rules** ([`rules`]): the species' own frame over the whole world,
//!   for the three things that touch more than the pack -- a latched gnawer
//!   riding a fighter's heels, the gnaw that fells a stone, the scramble up a
//!   platform's edge.
//!
//! **For the brood and the parasites.** The Broodmother's brood and the
//! Siegeshell's parasites are gnawers (§10), so [`GNAWER`] is a kind any pack
//! can muster: [`KINDS`] is public, its moves are the first block of
//! [`MOVES`], and nothing in the mind assumes a leader, a den or morale --
//! a pack with no Big One, `RoutShare` zero and an owning monster gets
//! bites, hamstrings and pile-ons out of the same code. `tests/gnawers.rs`
//! stands one up.

mod mind;
pub mod rules;
mod tuned;

pub use mind::{Mind, gnawed, howling, latched, piling, stumbling, treed_now};

use crate::critter::{CritterKind, CritterMove};
use crate::lore::Layout;
use crate::oven::KnobDecl;
use crate::pack::PackDecl;
use crate::species::{FightDecl, MoveDecl, Species, SpeciesId};

/// The moves, by index. The gnawer's first, so a pack that borrows the
/// gnawer borrows them in this order.
pub const DART: u8 = 0;
pub const HAMSTRING: u8 = 1;
pub const PILE_ON: u8 = 2;
pub const GNAW: u8 = 3;
pub const SCRAMBLE: u8 = 4;
pub const MAUL: u8 = 5;
pub const HOWL: u8 = 6;

pub const MOVES: [MoveDecl; 7] = [
    MoveDecl::new("Dart-bite", 0),
    MoveDecl::new("Hamstring", 0),
    MoveDecl::new("Pile-on", 0),
    MoveDecl::new("Gnaw", 0),
    MoveDecl::new("Scramble", 0),
    MoveDecl::new("Maul", 0),
    MoveDecl::new("Howl", 0),
];

/// The kinds, by index.
pub const GNAWER: u8 = 0;
pub const BIG_ONE: u8 = 1;

/// **The gnawer**, as any pack may muster it. A bite and a hamstring take one
/// of the pack's tokens; a scramble up a platform's edge takes one too, so
/// that at most the token count are clinging to an edge at once. The pile-on
/// and the gnaw are thrown by the pack's own rules, not by a decision of
/// the body's ([`mind`]), and need none.
pub const GNAWER_KIND: CritterKind = CritterKind {
    name: "Gnawer",
    moves: &[
        CritterMove::token(DART),
        CritterMove::token(HAMSTRING),
        CritterMove::free(PILE_ON).committed(),
        CritterMove::free(GNAW),
        CritterMove::token(SCRAMBLE),
    ],
    role: 0,
    yields: false,
};

pub const KINDS: [CritterKind; 2] = [
    GNAWER_KIND,
    CritterKind {
        name: "Big one",
        // Neither needs a token: the maul is its answer to somebody who has
        // come to it, and the howl is what hands the tokens out.
        moves: &[CritterMove::free(MAUL), CritterMove::free(HOWL)],
        role: 0,
        yields: false,
    },
];

pub static PACK: PackDecl = PackDecl {
    kinds: &KINDS,
    muster: &[(GNAWER, 6), (BIG_ONE, 1)],
    leader: Some(BIG_ONE),
    mind: &Mind,
};

crate::species_knobs! {
    /// The Gnawers' own numbers: everything §5 names that is not already a
    /// pack knob (`pack::PackKnob`) or a body's (`critter::CritterField`).
    pub enum Knob;
    HowlTokens,      "howl",      "Tokens while a howl lasts",           Int,     0,  6;
    HowlFrames,      "howl",      "A howl lasts",                        Frames,  0,  900;
    HowlLockout,     "howl",      "Lockout after a howl",                Frames,  0,  3600;
    HowlAfterLoss,   "howl",      "Howls within this of a loss",         Frames,  0,  900;
    HowlFlinch,      "howl",      "A hit cancels it and flinches it",    Frames,  0,  120;
    HowlHeight,      "howl",      "Rears to",                            Fixed,   0,  fx(4,1);
    LeaderScatter,   "big one",   "Scatters only this far",              Fixed,   0,  fx(20,1);
    LeaderComesIn,   "big one",   "Comes in to maul a fighter within",   Fixed,   0,  fx(30,1);
    Strain,          "big one",   "Strain that stumbles it",             Int,     0,  2000;
    StrainDesperate, "big one",   "The same, below half health",         Int,     0,  2000;
    StrainBleed,     "big one",   "Strain bled per frame",               Int,     0,  50;
    StumbleFrames,   "big one",   "Stumble length",                      Frames,  0,  300;
    DartFrom,        "dart-bite", "Crouches this far from its target",   Fixed,   0,  fx(10,1);
    FrontArc,        "dart-bite", "Bites and mauls only from in front (cos)", Fixed, fx(-1,1), fx(1,1);
    DartGiveUp,      "dart-bite", "Gives the token back if not there in", Frames, 0,  240;
    RearCos,         "hamstring", "Rear third, beyond (cos)",            Fixed,   fx(-1,1), fx(1,1);
    HamstringSlow,   "hamstring", "Speed while hamstrung (x)",           Fixed,   0,  fx(1,1);
    HamstringFrames, "hamstring", "The slow lasts",                      Frames,  0,  600;
    LatchDamage,     "hamstring", "Latched, bites for",                  Int,     0,  100;
    LatchEvery,      "hamstring", "Latched, every",                      Frames,  1,  60;
    LatchMost,       "hamstring", "Lets go by itself after",             Frames,  0,  240;
    PileonRadius,    "pile-on",   "Draws every gnawer within",           Fixed,   0,  fx(20,1);
    PileonRing,      "pile-on",   "The ring tightens to",                Fixed,   0,  fx(10,1);
    PileonStagger,   "pile-on",   "Leaps this far apart",                Frames,  0,  30;
    PileonKnockdown, "pile-on",   "Bites landed that knock down",        Int,     0,  10;
    KnockdownFrames, "pile-on",   "Knocked down for",                    Frames,  0,  240;
    PileonRest,      "pile-on",   "Between pile-ons",                    Frames,  0,  900;
    LeapReach,       "treed",     "Reaches feet this high",              Fixed,   0,  fx(10,1);
    TreedAfter,      "treed",     "Out of reach for this long: treed",   Frames,  0,  600;
    TreedRadius,     "treed",     "The ring under a treed fighter",      Fixed,   0,  fx(20,1);
    GnawWork,        "gnaw",      "Digger-frames that fell a stone",     Int,     0,  5000;
    GnawDiggers,     "gnaw",      "Diggers at most",                     Int,     0,  6;
    GnawReach,       "gnaw",      "Digs from this far off the stone",    Fixed,   0,  fx(3,1);
    FallStagger,     "gnaw",      "Felled, the fighter on it staggers",  Frames,  0,  240;
    ScrambleTop,     "scramble",  "Scrambles up a top no higher than",   Fixed,   0,  fx(4,1);
    ScrambleFrom,    "scramble",  "Scrambles from this near its side",   Fixed,   0,  fx(4,1);
    CorneredAt,      "morale",    "A rout turns on a fighter this near the den", Fixed, 0, fx(20,1);
    CoopGnawers,     "coop",      "More gnawers for two hunters",        Int,     0,  4;
    CoopTokens,      "coop",      "More tokens for two hunters",         Int,     0,  4;
}

/// Raw helper for the knob bounds above.
const fn fx(num: i32, den: i32) -> i32 {
    crate::fixed::Fx::ratio(num, den).raw()
}

const OWN: &[KnobDecl] = Knob::DECLS;

/// One of the Gnawers' own knobs, live.
pub fn knob(k: Knob) -> i32 {
    SPECIES.own_raw(k as usize)
}

/// The same, as fixed point.
pub fn knob_fx(k: Knob) -> crate::fixed::Fx {
    SPECIES.own_fx(k as usize)
}

/// What the Gnawers bring to the fight besides their bodies: no hazards, no
/// defended things, no lore of their own -- their state is the pack's memo --
/// and the frame hook that runs the rules touching more than the pack.
pub static FIGHT: FightDecl = FightDecl {
    layout: Layout::NONE,
    frame: Some(rules::frame),
    ..FightDecl::PLAIN
};

pub static SPECIES: Species = Species::pack_only(
    SpeciesId::GNAWERS,
    "Gnawers",
    &MOVES,
    OWN,
    &tuned::KNOBS,
    "crates/sim/src/species/gnawers/tuned.rs",
    &PACK,
)
.fighting(&FIGHT);
