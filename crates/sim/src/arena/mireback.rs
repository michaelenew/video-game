//! The Mire: the Mireback's arena. A drowned causeway in the lowlands, where
//! the marsh people keep pitch fires burning in iron braziers to keep the
//! thing off the road -- which is why there is fire in a swamp.
//!
//! 36 m square of flat peat, bigger than the proving ground's 28 because the
//! toad alone is twelve across and its clock needs floor to eat. 1.5 m banks
//! for walls. **Four braziers**, one on a 1.5 m stone plinth at the middle of
//! each bank -- standable, so each is a platform as well as a fire -- named as
//! sites (`brazier`) so the species finds them where the arena puts them. The
//! toad starts in the middle on clean floor and the hunters twelve metres
//! west of it. Reeds are drawn (`game/src/arenas/mireback.rs`) and collide
//! with nothing. See `docs/design/creatures/mireback.md` §11.

use super::{Area, Arena, ArenaId, Bounds, HuntMarks, Mark, Material, Region, Solid, Spawns};
use crate::objective::Site;
use crate::species::SpeciesId;

use Material::{Peat, Stone, Water};

pub static ARENA: Arena = Arena {
    id: ArenaId::MIREBACK,
    name: "Mire",
    creature: Some(SpeciesId::MIREBACK),
    bounds: Bounds::cm((-1800, 1800), (-1800, 1800)),
    floor: Peat,
    regions: &REGIONS,
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-400, 0, (400, 0)), Mark::cm(400, 0, (-400, 0))],
        hunt: Some(HuntMarks {
            hunters: [
                Mark::cm(-1200, -200, (0, -200)),
                Mark::cm(-1200, 200, (0, 200)),
            ],
            creatures: [Mark::cm(0, 0, (-1200, 0)), Mark::cm(0, 800, (-1200, 800))],
        }),
    },
    sites: &SITES,
};

/// The four braziers, at the middle of each bank, on their plinths. The box
/// is the plinth: two metres square and a metre and a half tall, the brazier
/// standing on top of it.
const SITES: [Site; 4] = [
    Site {
        name: "brazier",
        route: &[(0, -1650)],
        size: (200, 150, 200),
        material: Stone,
    },
    Site {
        name: "brazier",
        route: &[(1650, 0)],
        size: (200, 150, 200),
        material: Stone,
    },
    Site {
        name: "brazier",
        route: &[(0, 1650)],
        size: (200, 150, 200),
        material: Stone,
    },
    Site {
        name: "brazier",
        route: &[(-1650, 0)],
        size: (200, 150, 200),
        material: Stone,
    },
];

/// Standing water in the corners, under the banks: drawn, and nothing more.
const REGIONS: [Region; 4] = [
    Region {
        area: Area::disc_cm(-1550, -1550, 300),
        material: Water,
    },
    Region {
        area: Area::disc_cm(1550, -1550, 300),
        material: Water,
    },
    Region {
        area: Area::disc_cm(1550, 1550, 300),
        material: Water,
    },
    Region {
        area: Area::disc_cm(-1550, 1550, 300),
        material: Water,
    },
];

const SOLIDS: [Solid; 8] = [
    // Banks, a metre thick and a metre and a half high, just outside the floor.
    Solid::cm([-1900, 0, -1900], [-1800, 150, 1900], Peat),
    Solid::cm([1800, 0, -1900], [1900, 150, 1900], Peat),
    Solid::cm([-1900, 0, -1900], [1900, 150, -1800], Peat),
    Solid::cm([-1900, 0, 1800], [1900, 150, 1900], Peat),
    // The four plinths, against the middle of each bank.
    Solid::cm([-100, 0, -1800], [100, 150, -1500], Stone),
    Solid::cm([1500, 0, -100], [1800, 150, 100], Stone),
    Solid::cm([-100, 0, 1500], [100, 150, 1800], Stone),
    Solid::cm([-1800, 0, -100], [-1500, 150, 100], Stone),
];
