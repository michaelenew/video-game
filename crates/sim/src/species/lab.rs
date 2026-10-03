//! **The topology lab's door**: a species installed at runtime in place of the
//! one compiled in, for the experiments in
//! `docs/design/exploration/0005_body_plans.md`.
//!
//! Only with the `lab` feature, which only `crates/lab` turns on, and that
//! crate is outside the workspace so a workspace build never sees it. Nothing
//! the game builds can reach this module. It is rules, like the Oven: install
//! before a hunt, never during one.

use super::{COUNT, Species, SpeciesId};
use std::sync::RwLock;

static INSTALLED: RwLock<[Option<&'static Species>; COUNT]> = RwLock::new([None; COUNT]);

/// The species installed for an id, if any.
pub fn installed(id: SpeciesId) -> Option<&'static Species> {
    let slots = INSTALLED.read().ok()?;
    slots.get(id.0 as usize).copied().flatten()
}

/// Answer `lookup(species.id)` with this species from now on.
pub fn install(species: &'static Species) {
    if let Ok(mut slots) = INSTALLED.write()
        && let Some(slot) = slots.get_mut(species.id.0 as usize)
    {
        *slot = Some(species);
    }
}

/// Back to the species compiled in, for every id.
pub fn clear() {
    if let Ok(mut slots) = INSTALLED.write() {
        *slots = [None; COUNT];
    }
}
