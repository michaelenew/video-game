//! **The Ring**: Hearth's yard for fighting each other, and the one place in the
//! valley where the two of you can (`docs/design/valley.md`).
//!
//! It is the proving ground's square to the centimetre -- the same low walls,
//! the same two platforms, the same marks -- because versus is tuned there and a
//! ring that played differently would be a second game. Sand instead of earth.
//! Rounds run as they do in versus; the training dummy and the sparring bot
//! stand in for player two here when nobody has the second keys. Behind the
//! south wall is a small yard with the door back to the town in it: hop the
//! wall and walk into the door to leave.

use super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns};
use crate::valley::{Gate, Kind, Place, Seam, Zone};

use Material::{Ground, Sand, Stone};

pub static ARENA: Arena = Arena {
    id: ArenaId::RING,
    name: "Ring",
    creature: None,
    bounds: Bounds::cm((-1500, 1500), (-2400, 1500)),
    floor: Sand,
    regions: &REGIONS,
    solids: &SOLIDS,
    spawns: Spawns {
        // Eight metres apart, facing each other along x: the proving ground's marks.
        versus: [Mark::cm(-400, 0, (400, 0)), Mark::cm(400, 0, (-400, 0))],
        hunt: None,
    },
    sites: &[],
};

const REGIONS: [Region; 1] = [
    // The yard behind the south wall, trodden.
    Region {
        area: Area::rect_cm((-500, 500), (-2300, -1500)),
        material: Ground,
    },
];

const SOLIDS: [Solid; 9] = [
    // The proving ground's walls, a metre and a half high.
    Solid::cm([-1500, 0, -1500], [-1400, 150, 1500], Stone),
    Solid::cm([1400, 0, -1500], [1500, 150, 1500], Stone),
    Solid::cm([-1500, 0, -1500], [1500, 150, -1400], Stone),
    Solid::cm([-1500, 0, 1400], [1500, 150, 1500], Stone),
    // Its two low platforms.
    Solid::cm([-900, 0, -400], [-500, 150, 400], Stone),
    Solid::cm([500, 0, -400], [900, 150, 400], Stone),
    // The yard's walls.
    Solid::cm([-600, 0, -2300], [-500, 300, -1500], Stone),
    Solid::cm([500, 0, -2300], [600, 300, -1500], Stone),
    Solid::cm([-600, 0, -2400], [600, 300, -2300], Stone),
];

/// How Ring joins the rest of the valley.
pub static PLACE: Place = Place {
    arena: ArenaId::RING,
    kind: Kind::Ring,
    seams: &SEAMS,
    vines: &[],
    vents: &[],
};

const SEAMS: [Seam; 1] = [
    // The door back to the town, in the yard.
    Seam {
        zone: Zone::cm([-500, -100, -2300], [500, 400, -1650]),
        to: ArenaId::HEARTH,
        at: 1,
        marks: [
            Mark::cm(-200, -1100, (-200, -100)),
            Mark::cm(200, -1100, (200, -100)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "Hearth",
    },
];
