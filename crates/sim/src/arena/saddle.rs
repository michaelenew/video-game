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
    // The fourth waystone, at the foot of the Shrine's path.
    Solid::cm([101200, 8800, 600], [101300, 9620, 700], Material::Stone),
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

/// **The Shrine's path**: a switchback up the north-east slope to its
/// clearing, twenty metres.
pub const SHRINE_WAY: [Point; 5] = [
    at(100000, 400, 9000, 500),
    at(101000, 2500, 9600, 400),
    at(99500, 3800, 10200, 400),
    at(99500, 5000, 10700, 400),
    at(99500, 6000, 11000, 400),
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
    vines: &VINES,
    vents: &[],
};

const VINES: [Zone; 1] = [CRAGS[0].vine()];

const SEAMS: [Seam; 4] = [
    // Back through the third pass to the Pinewood.
    Seam {
        zone: Zone::cm([85200, 7700, 0], [86000, 8800, 1000]),
        to: ArenaId::PINEWOOD,
        at: 4,
        marks: [
            Mark::cm(86500, 200, (90500, 1000)),
            Mark::cm(86500, 600, (90500, 1000)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Pinewood",
    },
    // The Cliffs, over the bridge to the north: the Galewing.
    Seam {
        zone: Zone::cm([90100, 7900, 2200], [90900, 9000, 3000]),
        to: ArenaId::GALEWING,
        at: 0,
        marks: [
            Mark::cm(90350, 2000, (90500, 0)),
            Mark::cm(90650, 2000, (90500, 0)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Cliffs: the Galewing",
    },
    // The Ashwood, south into the burnt wood: the Veilstalker.
    Seam {
        zone: Zone::cm([93100, 8100, -3700], [93900, 9200, -2800]),
        to: ArenaId::VEILSTALKER,
        at: 0,
        marks: [
            Mark::cm(93350, -2600, (93500, 0)),
            Mark::cm(93650, -2600, (93500, 0)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Ashwood: the Veilstalker",
    },
    // The Shrine, up the switchback past the fourth waystone: the Mantis.
    Seam {
        zone: Zone::cm([100100, 8500, 800], [100900, 9700, 1600]),
        to: ArenaId::MANTIS,
        at: 0,
        marks: [
            Mark::cm(99700, 200, (97000, 0)),
            Mark::cm(99700, 600, (97000, 0)),
        ],
        gate: crate::valley::tier::FOUR,
        waystone: Some(0),
        says: "the Shrine: the Mantis",
    },
];
