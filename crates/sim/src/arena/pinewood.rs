//! **The Pinewood**: the fourth reach (`docs/design/valley.md`), land since
//! 2026-10-09 (`crate::valley::land`): a valley of pines, three rooms off it,
//! and a long climb out.
//!
//! | Beat | What it is |
//! | --- | --- |
//! | The wood | Pines thick on both sides of the road, and up the slopes |
//! | The Den | A path south into a clearing: the Pair |
//! | The Hollows | A path north to the root caves: the Broodmother |
//! | The Highlands | A path climbing south to a moor four metres up: the Ridgeback |
//! | The crag | A tall rock among the trees, with a cairn on top |
//! | The pool | Past the Highlands' path the road ends at a pool under a cliff of basalt |
//! | **The Waterfall** | The cliff, twenty-two metres tall, with the falls down its middle: the road's one jump climb (`super::waterfall`) |
//! | The top | The road goes on from the top of the cliff to **the third waystone**: two of the Pair, the Broodmother, the Ridgeback |

use super::{Arena, ArenaId, Bounds, Mark, Material, Solid, Spawns};
use crate::valley::land::{Point, at};
use crate::valley::{Crag, Gate, Kind, Place, Seam, Zone};

pub static ARENA: Arena = Arena {
    id: ArenaId::PINEWOOD,
    name: "Pinewood",
    creature: None,
    bounds: Bounds::cm((65000, 85000), (-14000, 14000)),
    floor: Material::Grass,
    regions: &[],
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [
            Mark::cm(66500, 400, (70200, 400)),
            Mark::cm(66500, 800, (70200, 400)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: None,
};

const SOLIDS: [Solid; 1] = [
    // The third waystone, by the pass at the top of the climb.
    Solid::cm([84300, 7900, 1100], [84400, 8720, 1200], Material::Stone),
];

/// **The road**, centimetres: x, z, floor, half its width -- as far as the
/// Waterfall, where it ends at the pool's shore.
pub const WAY: [Point; 6] = [
    at(65000, 400, 5600, 500),
    at(67500, 1000, 5650, 1400),
    at(70200, 400, 5700, 2200),
    at(73500, 1000, 5750, 2200),
    at(76500, -200, 6000, 1800),
    at(78500, -300, 6000, 1000),
];

/// **The road above the Waterfall**, from the top of the cliff to the third
/// pass. Its own way, because between the two the road is a climb: it
/// starts where the land holds it up to the cliff's top
/// (`super::waterfall::PIECES`, the cliff's top runs back to here).
pub const ABOVE: [Point; 2] = [at(82800, 0, 8200, 300), at(84800, 600, 8200, 500)];

/// **The pool** under the falls.
pub const POOL: [Point; 2] = [at(79250, -900, 5850, 650), at(79250, 200, 5850, 650)];

pub const CRAGS: [Crag; 1] = [Crag {
    x: 69000,
    z: 2600,
    ledges: 6,
}];

/// The second cairn is the Waterfall's foot, on the shore.
pub const CAIRNS: [(i32, i32); 2] = [(67000, -400), (77900, -600)];

pub static PLACE: Place = Place {
    arena: ArenaId::PINEWOOD,
    kind: Kind::Reach,
    seams: &SEAMS,
    vines: &VINES,
    vents: &[],
};

const VINES: [Zone; 1] = [CRAGS[0].vine()];

const SEAMS: [Seam; 5] = [
    // Back through the second pass to the Shelves.
    Seam {
        zone: Zone::cm([65200, 5100, 0], [66000, 6200, 1000]),
        to: ArenaId::SHELVES,
        at: 3,
        marks: [
            Mark::cm(66500, 400, (70200, 400)),
            Mark::cm(66500, 800, (70200, 400)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Shelves",
    },
    // The Den, south into a clearing: the Pair.
    Seam {
        zone: Zone::cm([69800, 5200, -2300], [70600, 6300, -1400]),
        to: ArenaId::PAIR,
        at: 0,
        marks: [
            Mark::cm(70050, -1200, (70200, 0)),
            Mark::cm(70350, -1200, (70200, 0)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Den: the Pair",
    },
    // The Hollows, north to the root caves: the Broodmother.
    Seam {
        zone: Zone::cm([73100, 5200, 2800], [73900, 6300, 3750]),
        to: ArenaId::BROODMOTHER,
        at: 0,
        marks: [
            Mark::cm(73350, 2600, (73500, 0)),
            Mark::cm(73650, 2600, (73500, 0)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Hollows: the Broodmother",
    },
    // The Highlands, climbing south: the Ridgeback.
    Seam {
        zone: Zone::cm([76100, 5500, -2500], [76900, 6700, -1600]),
        to: ArenaId::HIGHLANDS,
        at: 0,
        marks: [
            Mark::cm(76350, -1400, (76500, 0)),
            Mark::cm(76650, -1400, (76500, 0)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Highlands: the Ridgeback",
    },
    // On to the Saddle, through the third pass.
    Seam {
        zone: Zone::cm([84000, 7700, 100], [84800, 8800, 1100]),
        to: ArenaId::SADDLE,
        at: 0,
        marks: [
            Mark::cm(83300, 400, (79500, 0)),
            Mark::cm(83300, 800, (79500, 0)),
        ],
        gate: crate::valley::tier::THREE,
        waystone: Some(0),
        says: "the Saddle",
    },
];
