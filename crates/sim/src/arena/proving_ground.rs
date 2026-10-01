//! The proving ground: the first arena, and still the one versus, the training
//! dummy and the Ridgeback are fought in.
//!
//! 28 m square. Four low walls and two raised platforms, so the arena has an
//! edge and more than one height -- pushback toward an edge is meaningless
//! without an edge.
//!
//! The walls are deliberately low. Tall ones read as a box and put geometry
//! between the camera and the fight, and the camera's only answer is to pull in
//! toward the fighter's back -- it will not turn away, because the angle
//! belongs to the player.
//!
//! **Bit-identical to the `const` arena it replaced**: the pinned hunts in
//! `tests/ridgeback_pin.rs` and `crates/hunt/tests/pin.rs` hash every frame of
//! a fight in it, and `tests/arena.rs` checks these numbers against the old
//! ones.

use super::{Arena, ArenaId, Bounds, Mark, Material, Solid, Spawns};
use crate::species::SpeciesId;

use Material::Stone;

pub static ARENA: Arena = Arena {
    id: ArenaId::PROVING_GROUND,
    name: "Proving ground",
    creature: Some(SpeciesId::RIDGEBACK),
    bounds: Bounds::cm((-1400, 1400), (-1400, 1400)),
    floor: Material::Ground,
    regions: &[],
    solids: &SOLIDS,
    spawns: Spawns {
        // Eight metres apart, facing each other along x.
        versus: [Mark::cm(-400, 0, (400, 0)), Mark::cm(400, 0, (-400, 0))],
        // The Ridgeback's own spawn distances, which are knobs.
        hunt: None,
    },
};

const SOLIDS: [Solid; 6] = [
    // Walls, a metre thick and a metre and a half high, just outside the play
    // area.
    Solid::cm([-1500, 0, -1500], [-1400, 150, 1500], Stone),
    Solid::cm([1400, 0, -1500], [1500, 150, 1500], Stone),
    Solid::cm([-1500, 0, -1500], [1500, 150, -1400], Stone),
    Solid::cm([-1500, 0, 1400], [1500, 150, 1500], Stone),
    // Two low platforms. Low enough to jump onto, high enough that standing on
    // one changes the fight.
    Solid::cm([-900, 0, -400], [-500, 150, 400], Stone),
    Solid::cm([500, 0, -400], [900, 150, 400], Stone),
];

/// Half the floor's width: the distance from the middle to a wall. For the
/// tools and tests that place things against the proving ground's walls.
pub fn half() -> crate::fixed::Fx {
    ARENA.bounds.hi_x
}

/// How high the walls and the two platforms stand.
pub fn wall_height() -> crate::fixed::Fx {
    SOLIDS[0].max.y
}
