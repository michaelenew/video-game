//! **The Shelves**: the third reach (`docs/design/valley.md`), land since
//! 2026-10-09 (`crate::valley::land`).
//!
//! | Beat | What it is |
//! | --- | --- |
//! | Through the pass | Out of the first waystone's gap into a long open bowl |
//! | The tarn | A lake under the north slope; the road keeps to its south shore |
//! | The Mire and the Pan | Two side paths down to the south: the Mireback's bog, the Sandmaw's pan |
//! | The crag | A rock by the tarn's shore, with a cairn on top |
//! | The second pass | The road climbs out of the bowl to **the second waystone**: one of the Pan, the Mire |

use super::{Arena, ArenaId, Bounds, Mark, Material, Solid, Spawns};
use crate::valley::land::{Point, at};
use crate::valley::{Crag, Gate, Kind, Place, Seam, Zone};

pub static ARENA: Arena = Arena {
    id: ArenaId::SHELVES,
    name: "Shelves",
    creature: None,
    bounds: Bounds::cm((43000, 65000), (-14000, 14000)),
    floor: Material::Grass,
    regions: &[],
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [
            Mark::cm(44500, -900, (48000, -2200)),
            Mark::cm(44500, -500, (48000, -2200)),
        ],
        hunt: None,
    },
    sites: &[],
};

const SOLIDS: [Solid; 1] = [
    // The second waystone, beside the pass on its north shoulder.
    Solid::cm([64100, 5100, 1000], [64200, 5920, 1100], Material::Stone),
];

/// **The road**, centimetres: x, z, floor, half its width.
pub const WAY: [Point; 7] = [
    at(42800, -400, 4400, 500),
    at(45000, -1000, 4400, 1000),
    at(48000, -2200, 4400, 2200),
    at(51500, -3000, 4500, 2600),
    at(56000, -2800, 4650, 2600),
    at(60000, -1200, 5000, 1400),
    at(62800, -200, 5400, 800),
];

/// The last stretch of the road, up to the second pass: its own way, so the
/// road is one line through the Shelves and this is the climb.
pub const CLIMB: [Point; 2] = [at(62800, -200, 5400, 800), at(65000, 400, 5600, 500)];

/// **The tarn** under the north slope: a short, wide water.
pub const TARN: [Point; 2] = [at(51000, 3200, 4220, 1400), at(56500, 3800, 4220, 1200)];

pub const CRAGS: [Crag; 1] = [Crag {
    x: 48500,
    z: 1200,
    ledges: 5,
}];

pub const CAIRNS: [(i32, i32); 1] = [(46000, -400)];

pub static PLACE: Place = Place {
    arena: ArenaId::SHELVES,
    kind: Kind::Reach,
    seams: &SEAMS,
    vines: &VINES,
    vents: &[],
};

const VINES: [Zone; 1] = [CRAGS[0].vine()];

const SEAMS: [Seam; 4] = [
    // Back through the first pass to the Bank.
    Seam {
        zone: Zone::cm([43200, 3900, -1000], [44000, 5000, 200]),
        to: ArenaId::BANK,
        at: 2,
        marks: [
            Mark::cm(44400, -900, (48000, -2200)),
            Mark::cm(44400, -500, (48000, -2200)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Bank",
    },
    // The Mire, down a side path to the south.
    Seam {
        zone: Zone::cm([51500, 4000, -6100], [52300, 5100, -5200]),
        to: ArenaId::MIREBACK,
        at: 0,
        marks: [
            Mark::cm(51750, -5000, (51900, 0)),
            Mark::cm(52050, -5000, (51900, 0)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Mire: the Mireback",
    },
    // The Pan, down another to the south.
    Seam {
        zone: Zone::cm([56400, 4100, -5900], [57200, 5200, -5000]),
        to: ArenaId::SANDMAW,
        at: 0,
        marks: [
            Mark::cm(56650, -4800, (56800, 0)),
            Mark::cm(56950, -4800, (56800, 0)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Pan: the Sandmaw",
    },
    // On to the Pinewood, through the second pass.
    Seam {
        zone: Zone::cm([64100, 5100, -200], [64900, 6200, 800]),
        to: ArenaId::PINEWOOD,
        at: 0,
        marks: [
            Mark::cm(63500, -300, (60000, -1200)),
            Mark::cm(63500, 0, (60000, -1200)),
        ],
        gate: crate::valley::tier::TWO,
        waystone: Some(0),
        says: "the Pinewood",
    },
];
