//! **The Mouth**: the first reach of the valley, from Hearth's east gate up to
//! the Lip (`docs/design/valley.md`). Since 2026-10-09 it is **land**, not
//! boxes (`crate::valley::land`): there are no walls in this file.
//!
//! | Beat | What it is |
//! | --- | --- |
//! | The gate road | Out of the town's east gate, a dirt road between low hills |
//! | The river meadow | The valley opens fifty metres wide; a river runs down its south side, the road along its north. The herd's meadow is up a side path to the north |
//! | The crag | A rock standing in the meadow: ledges up its west face, a vine down its east, a cairn on top |
//! | The scree | The road bends south and climbs, ten metres over sixty |
//! | The Lip | Twenty metres up, the valley narrows, and the road goes on to the Bank |
//!
//! Everything here is in the map's own coordinates: a reach's origin is the
//! map's (`valley::layout`).

use super::{Arena, ArenaId, Bounds, Mark, Material, Solid, Spawns};
use crate::valley::land::{Point, at};
use crate::valley::{Crag, Gate, Kind, Place, Seam, Zone};

pub static ARENA: Arena = Arena {
    id: ArenaId::MOUTH,
    name: "Mouth",
    creature: None,
    bounds: Bounds::cm((4500, 25000), (-14000, 14000)),
    floor: Material::Grass,
    regions: &[],
    solids: &SOLIDS,
    spawns: Spawns {
        // Just out of the town's gate, facing up the valley.
        versus: [
            Mark::cm(6000, 200, (8500, 1200)),
            Mark::cm(6000, -200, (8500, 1000)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: None,
};

const SOLIDS: [Solid; 0] = [];

/// **The road**: x, z, the floor's height and how far the floor runs either
/// side, in centimetres. Out of the gate narrow, the meadow wide, the scree
/// climbing, the Lip narrow again.
pub const WAY: [Point; 9] = [
    at(4000, 0, 0, 300),
    at(6000, 300, 0, 800),
    at(8500, 1200, 0, 2200),
    at(11500, 1400, 50, 2600),
    at(15000, 600, 150, 2400),
    at(17800, -800, 400, 1400),
    at(20500, -1800, 1000, 800),
    at(23000, -1000, 1600, 700),
    at(25200, 0, 2000, 800),
];

/// **The river**, down the meadow's south side: its bed and half its width.
/// It runs from the scree's foot west past the town.
pub const RIVER: [Point; 5] = [
    at(18500, -2600, 250, 250),
    at(15500, -1900, 20, 320),
    at(11500, -1300, -70, 380),
    at(7500, -1700, -110, 380),
    at(3000, -3600, -140, 350),
];

/// The crag in the meadow, north of the road.
pub const CRAGS: [Crag; 1] = [Crag {
    x: 16000,
    z: 2600,
    ledges: 6,
}];

/// Where the floor cairns lie: the meadow's start, and the Lip.
pub const CAIRNS: [(i32, i32); 2] = [(7000, 1600), (24200, 400)];

/// How Mouth joins the rest of the valley.
pub static PLACE: Place = Place {
    arena: ArenaId::MOUTH,
    kind: Kind::Reach,
    seams: &SEAMS,
    vines: &VINES,
    vents: &[],
};

const VINES: [Zone; 1] = [CRAGS[0].vine()];

const SEAMS: [Seam; 3] = [
    // Back to the town, at its east gate.
    Seam {
        zone: Zone::cm([5200, -300, -500], [6000, 700, 500]),
        to: ArenaId::HEARTH,
        at: 0,
        marks: [
            Mark::cm(6400, -150, (8500, 900)),
            Mark::cm(6400, 150, (8500, 1100)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "Hearth",
    },
    // Up the side path to the herd's meadow.
    Seam {
        zone: Zone::cm([11100, -300, 3800], [11900, 700, 4600]),
        to: ArenaId::HORNBACK,
        at: 0,
        marks: [
            Mark::cm(11350, 3500, (11500, 0)),
            Mark::cm(11650, 3500, (11500, 0)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the low meadow: the Hornback herd",
    },
    // On to the Bank, at the Lip.
    Seam {
        zone: Zone::cm([24200, 1500, -500], [25000, 2600, 500]),
        to: ArenaId::BANK,
        at: 0,
        marks: [
            Mark::cm(23800, -350, (20500, -1800)),
            Mark::cm(23800, -50, (20500, -1600)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Bank",
    },
];
