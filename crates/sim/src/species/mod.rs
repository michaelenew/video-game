//! Species: what one kind of creature *is*, as data.
//!
//! The machinery that makes a creature stand, walk, collide, get hit, carry a
//! rider and decide what to do is shared -- [`crate::beast`] for the skeleton,
//! [`crate::monster`] for the body, the moves and the mind. What differs from
//! one creature to the next is a table, and this module is where the tables
//! live: one file (or folder) per species, each exporting one [`Species`].
//!
//! A [`Species`] is:
//!
//! - **a skeleton** -- bones, their parents, their rest offsets, which side
//!   each is on, and the pairs a mirrored clip swaps;
//! - **parts** -- one box per part on a bone, with flags that say what the box
//!   is: mountable, solid, breakable, a weak point, and which knob holds its
//!   damage multiplier;
//! - **legs** -- any number, none included;
//! - **moves** -- a name, a clip, and the flags that used to be special cases
//!   by name ([`MoveDecl`]); the frame data and hit volumes are knobs;
//! - **clips** -- its own baked table, written by `bake_beast`;
//! - **knobs** -- the numbers every creature has ([`Common`]), then its own,
//!   then its move table, all baked to its own `tuned.rs`.
//!
//! `Monster` carries a [`SpeciesId`] in the snapshot and reads everything else
//! from here, so a second species is a new file rather than a change to the
//! machinery. The recipe for adding one is in `docs/design/species.md`.
//!
//! ## Why the registry has a line for every creature already
//!
//! Ten creatures are planned after the Ridgeback (`docs/design/bestiary.md`),
//! and they will be built two at a time on separate branches. Every one of
//! them has an id and a commented-out line in each of the few places a species
//! is registered, **one blank line apart**. Building a creature means
//! uncommenting its own lines, and two branches doing that for two different
//! creatures touch lines git sees as unrelated -- so they merge without
//! conflicting.

pub mod common;

pub use common::Common;

use crate::beast::{Bone, ClipDecl, Leg, MAX_BREAKABLE, MAX_PARTS, Part, Shape};
use crate::fixed::Fx;
use crate::math::V3;
use crate::oven::KnobDecl;

// ---------------------------------------------------------------------------
// The registry
// ---------------------------------------------------------------------------

// One line per species, each followed by a blank line. See the module docs for
// why every planned creature already has one.

pub mod ridgeback;

// pub mod gnawers;

// pub mod hornback;

// pub mod mireback;

// pub mod sandmaw;

// pub mod pair;

// pub mod broodmother;

// pub mod veilstalker;

// pub mod mantis;

// pub mod galewing;

// pub mod siegeshell;

/// Which kind of creature. The one byte of species a `Monster` keeps in the
/// snapshot; everything else is looked up.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub struct SpeciesId(pub u8);

impl SpeciesId {
    pub const RIDGEBACK: SpeciesId = SpeciesId(0);
    pub const GNAWERS: SpeciesId = SpeciesId(1);
    pub const HORNBACK: SpeciesId = SpeciesId(2);
    pub const MIREBACK: SpeciesId = SpeciesId(3);
    pub const SANDMAW: SpeciesId = SpeciesId(4);
    pub const PAIR: SpeciesId = SpeciesId(5);
    pub const BROODMOTHER: SpeciesId = SpeciesId(6);
    pub const VEILSTALKER: SpeciesId = SpeciesId(7);
    pub const MANTIS: SpeciesId = SpeciesId(8);
    pub const GALEWING: SpeciesId = SpeciesId(9);
    pub const SIEGESHELL: SpeciesId = SpeciesId(10);

    /// The table. Every registered id has one; asking for an unregistered one
    /// is a bug in whoever built the monster, and gets the Ridgeback rather
    /// than a crash in the middle of a rollback.
    pub fn get(self) -> &'static Species {
        lookup(self).unwrap_or(&ridgeback::SPECIES)
    }
}

/// How many ids there are, registered or not.
pub const COUNT: usize = 11;

/// The species registered under an id, if one is.
pub const fn lookup(id: SpeciesId) -> Option<&'static Species> {
    match id {
        SpeciesId::RIDGEBACK => Some(&ridgeback::SPECIES),

        // SpeciesId::GNAWERS => Some(&gnawers::SPECIES),

        // SpeciesId::HORNBACK => Some(&hornback::SPECIES),

        // SpeciesId::MIREBACK => Some(&mireback::SPECIES),

        // SpeciesId::SANDMAW => Some(&sandmaw::SPECIES),

        // SpeciesId::PAIR => Some(&pair::SPECIES),

        // SpeciesId::BROODMOTHER => Some(&broodmother::SPECIES),

        // SpeciesId::VEILSTALKER => Some(&veilstalker::SPECIES),

        // SpeciesId::MANTIS => Some(&mantis::SPECIES),

        // SpeciesId::GALEWING => Some(&galewing::SPECIES),

        // SpeciesId::SIEGESHELL => Some(&siegeshell::SPECIES),
        _ => None,
    }
}

/// Every registered species, in id order.
pub fn all() -> impl Iterator<Item = &'static Species> {
    (0..COUNT as u8).filter_map(|i| lookup(SpeciesId(i)))
}

/// Find a registered species by name, ignoring case. For the tools that take
/// `--species`.
pub fn named(name: &str) -> Option<&'static Species> {
    all().find(|s| s.name.eq_ignore_ascii_case(name) || s.slug() == name.to_lowercase())
}

/// The next registered species after this one, in id order, wrapping round:
/// what the picker's cycle steps to. Unregistered ids are skipped, so a
/// creature whose branch has not landed is never offered.
pub fn after(id: SpeciesId) -> &'static Species {
    (1..=COUNT as u8)
        .filter_map(|step| lookup(SpeciesId((id.0.wrapping_add(step)) % COUNT as u8)))
        .next()
        .unwrap_or(&ridgeback::SPECIES)
}

// ---------------------------------------------------------------------------
// The table
// ---------------------------------------------------------------------------

/// The most moves a species has. The brain keeps a lockout per move in the
/// snapshot, so this is bytes.
pub const MAX_MOVES: usize = 16;

/// One of a species' moves, as far as the code is concerned.
///
/// The move's *numbers* -- frames, damage, hit volume, what the brain wants --
/// are knobs (`oven::MonsterField`), tuned per species. What is here is the
/// shape of the move, and the **flags that used to be special cases by name**:
/// declared in the table, the way `Move::aim()` is declared for a fighter's
/// moves rather than inferred from what they leave behind.
#[derive(Clone, Copy, Debug)]
pub struct MoveDecl {
    pub name: &'static str,
    /// The clip it plays, as an index into the species' clips.
    pub clip: usize,
    /// **Played toward the side its target is on.** The clip is baked going
    /// one way; when the move commits, the creature looks at which side its
    /// target is on and plays it mirrored if that is the other one. Decided
    /// once and held for the move, because a swing that changed its mind
    /// mid-whip would be a telegraph that lied. The Ridgeback's tail sweep.
    pub mirrors_to_target_side: bool,
    /// **Its violence is a knob.** The species' own knob, by index, that the
    /// baked clip is scaled by -- the one kind of animation number that is
    /// gameplay rather than content, because it decides whether a braced
    /// rider stays on. The Ridgeback's shake, by its shake force.
    pub scaled_by: Option<u16>,
}

impl MoveDecl {
    pub const fn new(name: &'static str, clip: usize) -> MoveDecl {
        MoveDecl {
            name,
            clip,
            mirrors_to_target_side: false,
            scaled_by: None,
        }
    }

    pub const fn mirrors_to_target_side(mut self) -> MoveDecl {
        self.mirrors_to_target_side = true;
        self
    }

    pub const fn scaled_by(mut self, knob: u16) -> MoveDecl {
        self.scaled_by = Some(knob);
        self
    }
}

/// The clips every creature plays outside its moves, by index into its clip
/// table. The same clip may stand for two of them.
#[derive(Clone, Copy, Debug)]
pub struct Stock {
    /// Standing. Loops, and breathes.
    pub idle: usize,
    /// One full cycle of the walk. Indexed by **ground covered** rather than
    /// by time -- a cycle on a fixed cadence skates the moment the body moves
    /// at any other speed.
    pub walk: usize,
    /// The fast gait, blended from the walk above the walking speed.
    pub gallop: usize,
    pub flinch: usize,
    /// Down on a knee: what a broken leg and a landed piece of crowd control
    /// both produce.
    pub stumble: usize,
    pub topple: usize,
    pub dead: usize,
}

/// One kind of creature. See the module docs.
#[derive(Debug)]
pub struct Species {
    pub id: SpeciesId,
    /// As a person says it: "Ridgeback". Also the family name of its knobs in
    /// the Oven, so it is what the palette groups them under.
    pub name: &'static str,

    // ---- the skeleton ----
    pub bones: &'static [Bone],
    /// Left and right bones that swap when a clip is mirrored.
    pub mirror: &'static [(usize, usize)],
    /// The bones the head's tracking is spread across, base to tip. Empty for
    /// a creature that does not turn its head to watch you.
    pub neck: &'static [usize],
    /// Which bone a move's hit volume rides, by the move table's `Follows`
    /// number: `follows[0]` is the body's, and the rest are the species' own.
    pub follows: &'static [usize],

    // ---- the body ----
    pub parts: &'static [Part],
    /// Which parts are breakable, in part order, and how many: worked out from
    /// the flags by `beast::breakables`.
    pub breakable: ([u8; MAX_BREAKABLE], usize),
    pub legs: &'static [Leg],

    // ---- what it does ----
    pub moves: &'static [MoveDecl],
    pub clips: &'static [ClipDecl],
    pub stock: Stock,
    /// `(first row, how many)` per clip, from the species' baked file.
    pub span: &'static [(u16, u16)],
    /// How many baked rows there are.
    pub rows: usize,
    /// One baked row: `beast::channels(bones)` numbers, hips then three per
    /// bone. A function rather than a slice so the baked file can keep its
    /// table as an array of rows, one per line.
    pub row: fn(usize) -> &'static [i32],

    // ---- its numbers ----
    /// Knobs this species has that others do not: after [`Common`] in its
    /// store, before its moves.
    pub own: &'static [KnobDecl],
    /// What its `tuned.rs` holds: common, own, then moves.
    pub tuned: &'static [i32],
    /// Where that file is, from the repository root, for the bake.
    pub tuned_path: &'static str,
}

impl Species {
    /// The name as an identifier: lower case, spaces to underscores.
    pub fn slug(&self) -> String {
        self.name.to_lowercase().replace(' ', "_")
    }

    pub fn part_count(&self) -> usize {
        self.parts.len().min(MAX_PARTS)
    }

    /// The part's box at the species' tuned scale.
    pub fn shape(&self, index: usize) -> Shape {
        let s = self.parts[index.min(self.parts.len() - 1)].shape;
        let k = self.scale();
        Shape {
            min: s.min.scale(k),
            max: s.max.scale(k),
            ..s
        }
    }

    /// A bone's rest offset at the species' tuned scale.
    pub fn rest(&self, bone: usize) -> V3 {
        self.bones[bone.min(self.bones.len() - 1)]
            .rest
            .scale(self.scale())
    }

    /// How much of an attack's damage a part passes through. Below one is
    /// armour; a weak point is above it, which is the whole reason to climb.
    pub fn vulnerability(&self, part: usize) -> Fx {
        let p = self.parts[part.min(self.parts.len() - 1)];
        Fx::from_raw(self.own_raw(p.vuln as usize))
    }

    /// Damage here fills the poise pool.
    pub fn is_weak_point(&self, part: usize) -> bool {
        self.parts.get(part).is_some_and(|p| p.weak)
    }

    /// The health slot a breakable part keeps, if it is one.
    pub fn break_slot(&self, part: usize) -> Option<usize> {
        let (slots, count) = self.breakable;
        slots[..count].iter().position(|p| *p as usize == part)
    }

    /// The breakable parts, in part order.
    pub fn breakables(&self) -> impl Iterator<Item = usize> + '_ {
        let (slots, count) = &self.breakable;
        slots[..*count].iter().map(|p| *p as usize)
    }

    /// The bone a move's `Follows` number means.
    pub fn follow_bone(&self, follows: u8) -> usize {
        self.follows
            .get(follows as usize)
            .copied()
            .unwrap_or(self.follows[0])
    }

    /// Index of a part by name, for tools and tests that talk about parts.
    pub fn part_named(&self, name: &str) -> Option<usize> {
        self.parts.iter().position(|p| p.name == name)
    }

    /// Index of a move by name.
    pub fn move_named(&self, name: &str) -> Option<u8> {
        self.moves
            .iter()
            .position(|m| m.name == name)
            .map(|i| i as u8)
    }

    // ---- the knob store ----

    /// How many knobs this species keeps: [`Common`], its own, then one row
    /// of `oven::MonsterField` per move.
    pub fn knob_count(&self) -> usize {
        Common::ALL.len() + self.own.len() + self.moves.len() * crate::oven::MONSTER_FIELDS
    }

    /// One of the numbers every creature has.
    pub fn common(&self, k: Common) -> i32 {
        crate::oven::species_raw(self.id, k as usize)
    }

    /// One of this species' own knobs, by its index in [`Species::own`].
    pub fn own_raw(&self, k: usize) -> i32 {
        crate::oven::species_raw(self.id, Common::ALL.len() + k)
    }

    /// One of this species' own knobs, as fixed point.
    pub fn own_fx(&self, k: usize) -> Fx {
        Fx::from_raw(self.own_raw(k))
    }

    /// Where a move's field sits in the store.
    pub fn move_index(&self, slot: usize, field: crate::oven::MonsterField) -> usize {
        Common::ALL.len()
            + self.own.len()
            + slot.min(self.moves.len() - 1) * crate::oven::MONSTER_FIELDS
            + field as usize
    }
}
