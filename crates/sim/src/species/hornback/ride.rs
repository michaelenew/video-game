//! Riding a cow: see M5. A stub until it is built.

use crate::pack::Pack;
use crate::state::{Player, World};

/// The species' ride frame.
pub fn frame(w: &mut World, pack: &mut Pack) {
    let _ = (w, pack);
}

/// The critter a fighter is riding, if any.
pub fn rider_of(p: &Player) -> Option<usize> {
    let _ = p;
    None
}
