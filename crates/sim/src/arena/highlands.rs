//! **The Highlands**: the Ridgeback's room in the valley (`docs/design/valley.md`).
//!
//! The Ridgeback is tuned in the proving ground and hunted there by
//! `arena::for_species`; this is the same floor plan -- the same walls, the same
//! two platforms, the same spawn distances -- on a moor, so meeting it in the
//! valley is the fight that was tuned. Its way out is `crate::valley::rooms`.

use super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns};

use Material::{Grass, Peat, Rock};

pub static ARENA: Arena = Arena {
    id: ArenaId::HIGHLANDS,
    name: "Highlands",
    creature: None,
    bounds: Bounds::cm((-1400, 1400), (-1400, 1400)),
    floor: Grass,
    regions: &REGIONS,
    solids: &SOLIDS,
    spawns: Spawns {
        // The proving ground's marks.
        versus: [Mark::cm(-400, 0, (400, 0)), Mark::cm(400, 0, (-400, 0))],
        hunt: None,
    },
    sites: &[],
};

const REGIONS: [Region; 1] = [
    // Heather on the moor.
    Region {
        area: Area::disc_cm(600, -800, 400),
        material: Peat,
    },
];

const SOLIDS: [Solid; 6] = [
    // A drystone wall round the moor, a metre and a half high.
    Solid::cm([-1500, 0, -1500], [-1400, 150, 1500], Rock),
    Solid::cm([1400, 0, -1500], [1500, 150, 1500], Rock),
    Solid::cm([-1500, 0, -1500], [1500, 150, -1400], Rock),
    Solid::cm([-1500, 0, 1400], [1500, 150, 1500], Rock),
    // Two tors: the proving ground's platforms.
    Solid::cm([-900, 0, -400], [-500, 150, 400], Rock),
    Solid::cm([500, 0, -400], [900, 150, 400], Rock),
];
