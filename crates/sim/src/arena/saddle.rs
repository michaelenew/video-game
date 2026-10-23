//! **The Saddle**: the last reach (`docs/design/valley.md`), land since
//! 2026-10-09 (`crate::valley::land`): an alpine meadow under snow, high
//! between peaks.
//!
//! | Beat | What it is |
//! | --- | --- |
//! | The meadow | Wide, snowy and nearly level past the third pass |
//! | The Cliffs | A rope-bridge north over a ravine to the Galewing's plateau |
//! | The Ashwood | A path south into the burnt wood: the Veilstalker |
//! | The crag | The highest rock in the valley, with a cairn on top |
//! | The Shrine | A switchback up the north-east slope, twenty metres, past **the fourth waystone**: one of the Cliffs, the Ashwood. The Mantis waits at the top |
//!
//! The road ends here, at a view. The Long Valley is off Hearth's west gate.

use super::{Arena, ArenaId, Bounds, Mark, Material, Solid, Spawns};
use crate::valley::land::{Point, at};
use crate::valley::{Crag, Gate, Kind, Place, Seam, Zone};

pub static ARENA: Arena = Arena {
    id: ArenaId::SADDLE,
    name: "Saddle",
    creature: None,
    bounds: Bounds::cm((85000, 102500), (-14000, 14000)),
    floor: Material::Snow,
    regions: &[],
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [
            Mark::cm(86500, 200, (90500, 1000)),
            Mark::cm(86500, 600, (90500, 1000)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: None,
};

const SOLIDS: [Solid; 1] = [
    // The fourth waystone, by the foot of the stones to the Cliffs, past
    // which the way goes on to the Shrine.
    Solid::cm([91950, 8400, 1850], [92050, 9220, 1950], Material::Stone),
];

/// **The road**, centimetres: x, z, floor, half its width.
pub const WAY: [Point; 6] = [
    at(84800, 600, 8200, 500),
    at(87200, 200, 8300, 1400),
    at(90500, 1000, 8500, 2600),
    at(93500, -600, 8600, 2600),
    at(97000, 0, 8800, 1800),
    at(100000, 400, 9000, 1200),
];

pub const CRAGS: [Crag; 1] = [Crag {
    x: 95500,
    z: -2800,
    ledges: 7,
}];

pub const CAIRNS: [(i32, i32); 2] = [(87500, -600), (99000, -400)];

pub static PLACE: Place = Place {
    arena: ArenaId::SADDLE,
    kind: Kind::Reach,
    seams: &SEAMS,
    vents: &[],
};

const SEAMS: [Seam; 4] = [
    // Back through the third pass to the Pinewood.
    Seam {
        zone: Zone::cm([85200, 7700, 0], [86000, 8800, 1000]),
        to: ArenaId::PINEWOOD,
        at: 2,
        marks: [
            Mark::cm(86500, 200, (90500, 1000)),
            Mark::cm(86500, 600, (90500, 1000)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Pinewood",
    },
    // The Cliffs: the foot of the stones across the ravine to the north.
    Seam {
        zone: Zone::cm([90200, 8200, 1500], [91000, 8900, 2300]),
        to: ArenaId::GALEWING,
        at: 0,
        marks: [
            Mark::cm(90450, 1200, (90450, 1900)),
            Mark::cm(90750, 1200, (90750, 1900)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Cliffs: the Galewing",
    },
    // The Ashwood: the foot of the stones up to it, south of the road.
    Seam {
        zone: Zone::cm([93350, 8300, -4250], [94150, 9000, -3450]),
        to: ArenaId::VEILSTALKER,
        at: 0,
        marks: [
            Mark::cm(93600, -3150, (93600, -3850)),
            Mark::cm(93900, -3150, (93900, -3850)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Ashwood: the Veilstalker",
    },
    // The Shrine, on past the Cliffs and the fourth waystone: the Mantis.
    Seam {
        zone: Zone::cm([91000, 8200, 1500], [91800, 8900, 2300]),
        to: ArenaId::MANTIS,
        at: 0,
        marks: [
            Mark::cm(91250, 1200, (91250, 1900)),
            Mark::cm(91550, 1200, (91550, 1900)),
        ],
        gate: crate::valley::tier::FOUR,
        waystone: Some(0),
        says: "the Shrine: the Mantis",
    },
];
