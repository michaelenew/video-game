//! One hunter plan per creature, and what its report calls things.
//!
//! Each creature's hunter lives in its own file here and is registered on a
//! [`Card`]: how to make its plan, which of the creature's moves buck a rider,
//! and the few words its report uses for the creature's parts. The report's
//! per-move and per-part lines come from the species table itself; the card
//! only says what to call them.
//!
//! Like the species registry in `sim::species`, every planned creature already
//! has a line here, commented out and one blank line from the next, so two
//! branches each building one creature uncomment two different lines and do
//! not conflict.

use sim::fixed::Fx;
use sim::species::SpeciesId;

use crate::Plan;

pub mod ridgeback;

/// The dev pack's hunter.
pub mod gnats;

pub mod gnawers;

pub mod hornback;

// pub mod mireback;

// pub mod sandmaw;

// pub mod pair;

// pub mod broodmother;

// pub mod veilstalker;

// pub mod mantis;

// pub mod galewing;

// pub mod siegeshell;

/// A creature's entry in the harness.
pub struct Card {
    pub species: SpeciesId,
    /// Make a hunter: which fighter it drives, the hunt's seed (for its
    /// timing error), and how high its class jumps.
    pub plan: fn(who: usize, seed: u32, hop: Fx) -> Box<dyn Plan + Send + Sync>,
    /// Does this move throw a rider? A ride that ends, not by a buck, while
    /// one of these is under way is a rider who read it and left.
    pub bucks: fn(kind: u8) -> bool,
    pub words: Words,
    /// **Its own report lines**, if it has any beyond the shared ones: a
    /// tally the report feeds every frame and prints under the creature's
    /// name. The Gnawers' pile-ons, howls and windows (§9).
    pub tally: Option<fn() -> Box<dyn crate::report::Tally>>,
}

/// What the report calls the creature's parts, in its own words.
pub struct Words {
    /// Hits that filled the poise pool: damage on a weak point.
    pub weak_hits: &'static str,
    /// Breakable parts broken.
    pub broken: &'static str,
    /// Damage into the breakable parts, and why that is worth counting.
    pub into_breakables: &'static str,
    pub into_breakables_why: &'static str,
    /// The most damage any one breakable part took.
    pub worst: &'static str,
    /// What a ride has to be long enough for.
    pub ride_for: &'static str,
    /// Where the Blood mage drinks while the creature is down.
    pub toppled_pool: &'static str,
}

/// The card for a species, if it has a plan.
pub fn card(id: SpeciesId) -> Option<&'static Card> {
    match id {
        SpeciesId::RIDGEBACK => Some(&ridgeback::CARD),

        SpeciesId::GNATS => Some(&gnats::CARD),

        SpeciesId::GNAWERS => Some(&gnawers::CARD),

        SpeciesId::HORNBACK => Some(&hornback::CARD),

        // SpeciesId::MIREBACK => Some(&mireback::CARD),

        // SpeciesId::SANDMAW => Some(&sandmaw::CARD),

        // SpeciesId::PAIR => Some(&pair::CARD),

        // SpeciesId::BROODMOTHER => Some(&broodmother::CARD),

        // SpeciesId::VEILSTALKER => Some(&veilstalker::CARD),

        // SpeciesId::MANTIS => Some(&mantis::CARD),

        // SpeciesId::GALEWING => Some(&galewing::CARD),

        // SpeciesId::SIEGESHELL => Some(&siegeshell::CARD),
        _ => None,
    }
}
