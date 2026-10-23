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
    rim: None,
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
    vents: &[],
};

const SEAMS: [Seam; 5] = [
    // Back through the first pass to the Bank.
    Seam {
        zone: Zone::cm([43200, 3900, -1000], [44000, 5000, 200]),
        to: ArenaId::BANK,
        at: 1,
        marks: [
            Mark::cm(44400, -900, (48000, -2200)),
            Mark::cm(44400, -500, (48000, -2200)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Bank",
    },
    // The Mire: the foot of the stones up to it (`arena::stations`).
    Seam {
        zone: Zone::cm([51850, 4200, -5800], [52650, 4900, -5000]),
        to: ArenaId::MIREBACK,
        at: 0,
        marks: [
            Mark::cm(52100, -4700, (52100, -5400)),
            Mark::cm(52400, -4700, (52400, -5400)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Mire: the Mireback",
    },
    // The Pan: the foot of the stones down from it, east of the Mire.
    Seam {
        zone: Zone::cm([59000, 4650, -4600], [59800, 5350, -3800]),
        to: ArenaId::SANDMAW,
        at: 0,
        marks: [
            Mark::cm(59250, -3500, (59250, -4200)),
            Mark::cm(59550, -3500, (59550, -4200)),
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
    // South down the path between the mountains, to the Deep's lid.
    Seam {
        zone: Zone::cm([60000, 4000, -14000], [62000, 5600, -13400]),
        to: ArenaId::DEEP,
        at: 0,
        marks: [
            Mark::cm(60700, -9000, (60300, -1500)),
            Mark::cm(61000, -9000, (60300, -1500)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Deep",
    },
];
