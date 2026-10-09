//! **The Bank**: the second reach (`docs/design/valley.md`), land since
//! 2026-10-09 (`crate::valley::land`).
//!
//! | Beat | What it is |
//! | --- | --- |
//! | The upper meadow | Broad and nearly level past the Lip; the den is up a side path to the north |
//! | The crag | The old stair, as a rock: eight ledges up a face, a cairn on top that sees back down the valley |
//! | The bank | The road climbs twenty-one metres in two bends |
//! | The pass | **The first waystone**: one of the herd, the den. The road narrows to a gap between two shoulders of hill; while the stone is dark the gap is shut |

use super::{Arena, ArenaId, Bounds, Mark, Material, Solid, Spawns};
use crate::valley::land::{Point, at};
use crate::valley::{Crag, Gate, Kind, Place, Seam, Zone};

pub static ARENA: Arena = Arena {
    id: ArenaId::BANK,
    name: "Bank",
    creature: None,
    bounds: Bounds::cm((25000, 43000), (-14000, 14000)),
    floor: Material::Grass,
    regions: &[],
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [
            Mark::cm(26500, 300, (30500, 1200)),
            Mark::cm(26500, 700, (30500, 1200)),
        ],
        hunt: None,
    },
    sites: &[],
};

const SOLIDS: [Solid; 1] = [
    // The first waystone, beside the pass on its north shoulder.
    Solid::cm([41900, 4000, 400], [42000, 4720, 500], Material::Stone),
];

/// **The road**, centimetres: x, z, floor, half its width.
pub const WAY: [Point; 8] = [
    at(25200, 0, 2000, 800),
    at(27500, 800, 2000, 1800),
    at(30500, 1200, 2100, 2400),
    at(33500, 600, 2300, 1600),
    at(36000, -600, 3000, 900),
    at(38500, -1400, 3700, 800),
    at(40800, -1000, 4200, 800),
    at(42800, -400, 4400, 500),
];

pub const CRAGS: [Crag; 1] = [Crag {
    x: 34500,
    z: -2600,
    ledges: 8,
}];

pub const CAIRNS: [(i32, i32); 1] = [(28500, 1600)];

pub static PLACE: Place = Place {
    arena: ArenaId::BANK,
    kind: Kind::Reach,
    seams: &SEAMS,
    vines: &VINES,
    vents: &[],
};

const VINES: [Zone; 1] = [CRAGS[0].vine()];

const SEAMS: [Seam; 3] = [
    // Back down to the Mouth.
    Seam {
        zone: Zone::cm([25000, 1500, -500], [25800, 2600, 700]),
        to: ArenaId::MOUTH,
        at: 2,
        marks: [
            Mark::cm(26300, 300, (30500, 1200)),
            Mark::cm(26300, 700, (30500, 1200)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Mouth",
    },
    // The den, up a side path to the north: the Gnawers.
    Seam {
        zone: Zone::cm([30100, 1600, 3200], [30900, 2700, 4100]),
        to: ArenaId::GNAWERS,
        at: 0,
        marks: [
            Mark::cm(30350, 3000, (30500, 0)),
            Mark::cm(30650, 3000, (30500, 0)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the den: the Gnawers",
    },
    // On to the Shelves, through the pass under the first waystone.
    Seam {
        zone: Zone::cm([42000, 3900, -1000], [42800, 5000, 200]),
        to: ArenaId::SHELVES,
        at: 0,
        marks: [
            Mark::cm(41500, -1050, (38500, -1400)),
            Mark::cm(41500, -750, (38500, -1400)),
        ],
        gate: crate::valley::tier::ONE,
        waystone: Some(0),
        says: "the Shelves",
    },
];
