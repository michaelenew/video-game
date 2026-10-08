//! **The Pinewood**: the fourth reach, a forest floor and a wall at its end
//! (`docs/design/valley.md`). Three rooms open off it -- tier three is beat two of
//! three -- and the way on is up the trunks.
//!
//! | Beat | What it asks |
//! | --- | --- |
//! | The wood | Trunks as pillars, nothing to climb: walk |
//! | The Den | A notch in the south wall: the Pair |
//! | **The root cave** | A low roof of roots in front of a notch in the north wall: the Hollows, and the Broodmother |
//! | **The Highlands' stair** | Six blocks stepping up the south wall to a notch fourteen metres up: the Ridgeback's moor |
//! | **The chimney** | Two giant trunks four metres apart, eleven ledges alternating up the slot between them, twenty-six metres: the longest climb in the valley, and a fall from the top of it costs four hundred |
//! | **The band** | Four metres from the top of the second trunk to the band at its own height. A vine up the band's face, for anyone, slowly |
//! | The band's top | **The third waystone**: two of the Pair, the Broodmother and the Ridgeback. The passage on to the Saddle |

use super::relief::{Bump, Ramp};
use super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns};
use crate::valley::{Gate, Kind, Place, Seam, Zone};

use Material::{Grass, Ground, Peat, Rock, Snow, Stone, Wood};

pub static ARENA: Arena = Arena {
    id: ArenaId::PINEWOOD,
    name: "Pinewood",
    creature: None,
    bounds: Bounds::cm((-10000, 10000), (-3200, 3200)),
    floor: Grass,
    regions: &REGIONS,
    solids: &SOLIDS,
    spawns: Spawns {
        // Just inside the passage from the Shelves, facing into the wood.
        versus: [
            Mark::cm(-8600, -150, (-7600, -150)),
            Mark::cm(-8600, 150, (-7600, 150)),
        ],
        hunt: None,
    },
    sites: &[],
};

/// The floor between the rock: gentle swells and hollows.
pub const BUMPS: [Bump; 5] = [
    Bump::cm(-7000, -500, 900, 60),
    Bump::cm(-4000, 1200, 800, -50),
    Bump::cm(-500, -800, 700, 50),
    Bump::cm(2000, 800, 800, 60),
    Bump::cm(3500, -1400, 600, 40),
];

/// The floor climbing: what the valley rises by between its cliffs.
pub const RAMPS: [Ramp; 0] = [];

const REGIONS: [Region; 4] = [
    // Needles and peat under the trees.
    Region {
        area: Area::disc_cm(-6000, -200, 900),
        material: Peat,
    },
    // More of it.
    Region {
        area: Area::disc_cm(1500, 400, 800),
        material: Peat,
    },
    // Trodden earth at the chimney's foot.
    Region {
        area: Area::rect_cm((4000, 6200), (-600, 600)),
        material: Ground,
    },
    // Bare earth in the Den's notch.
    Region {
        area: Area::disc_cm(-5000, -2000, 400),
        material: Ground,
    },
];

const SOLIDS: [Solid; 59] = [
    // The south side, with the Den's notch and the Highlands' notch.
    Solid::cm([-9200, 0, -2400], [-5400, 3800, -2200], Rock),
    Solid::cm([-4600, 0, -2400], [0, 3800, -2200], Rock),
    Solid::cm([800, 0, -2400], [9200, 3800, -2200], Rock),
    // the notch at x -54 to -46: its sides and back
    Solid::cm([-5600, 0, -3000], [-5400, 1400, -2400], Rock),
    Solid::cm([-4600, 0, -3000], [-4400, 1400, -2400], Rock),
    Solid::cm([-5600, 0, -3200], [-4400, 1400, -3000], Rock),
    // the notch at x 0 to 8: its sides and back
    Solid::cm([-200, 0, -3000], [0, 2600, -2400], Rock),
    Solid::cm([800, 0, -3000], [1000, 2600, -2400], Rock),
    Solid::cm([-200, 0, -3200], [1000, 2600, -3000], Rock),
    // The Highlands' notch is fourteen metres up: the wall under it, and its floor.
    Solid::cm([0, 0, -2400], [800, 1400, -2200], Rock),
    Solid::cm([0, 0, -3000], [800, 1400, -2400], Rock),
    // The Highlands' stair: six blocks up the south wall.
    Solid::cm([-2600, 0, -2200], [-2300, 220, -2040], Rock),
    Solid::cm([-2200, 0, -2200], [-1900, 440, -2040], Rock),
    Solid::cm([-1800, 0, -2200], [-1500, 660, -2040], Rock),
    Solid::cm([-1400, 0, -2200], [-1100, 880, -2040], Rock),
    Solid::cm([-1000, 0, -2200], [-700, 1100, -2040], Rock),
    Solid::cm([-600, 0, -2200], [-300, 1320, -2040], Rock),
    // The north side, with the Hollows' notch.
    Solid::cm([-9200, 0, 2200], [-1400, 3800, 2400], Rock),
    Solid::cm([-600, 0, 2200], [9200, 3800, 2400], Rock),
    // the notch at x -14 to -6: its sides and back
    Solid::cm([-1600, 0, 2400], [-1400, 1400, 3000], Rock),
    Solid::cm([-600, 0, 2400], [-400, 1400, 3000], Rock),
    Solid::cm([-1600, 0, 3000], [-400, 1400, 3200], Rock),
    // The root cave: a roof of roots over the notch's mouth.
    Solid::cm([-1800, 400, 1600], [-200, 600, 2200], Wood),
    // The west end: the passage back to the Shelves.
    Solid::cm([-9200, 0, -2200], [-9000, 3800, -300], Rock),
    Solid::cm([-9200, 0, 300], [-9000, 3800, 2200], Rock),
    Solid::cm([-9800, 0, -500], [-9200, 1000, -300], Rock),
    Solid::cm([-9800, 0, 300], [-9200, 1000, 500], Rock),
    Solid::cm([-10000, 0, -500], [-9800, 1000, 500], Rock),
    // The east end, on the band: the passage on to the Saddle.
    Solid::cm([9000, 0, -2200], [9200, 3800, -300], Rock),
    Solid::cm([9000, 0, 300], [9200, 3800, 2200], Rock),
    // the passage's floor
    Solid::cm([9000, 0, -300], [9800, 2600, 300], Rock),
    Solid::cm([9200, 0, -500], [9800, 3600, -300], Rock),
    Solid::cm([9200, 0, 300], [9800, 3600, 500], Rock),
    Solid::cm([9800, 0, -500], [10000, 3600, 500], Rock),
    // The band, twenty-six metres, the whole way across.
    Solid::cm([6200, 0, -2200], [9000, 2600, 2200], Grass),
    // The chimney's two giant trunks, four metres apart.
    Solid::cm([4720, 0, -80], [4880, 2600, 80], Wood),
    Solid::cm([5520, 0, -80], [5680, 2600, 80], Wood),
    // Their ledges, alternating up the slot.
    Solid::cm([5260, 170, -200], [5400, 220, 200], Wood),
    Solid::cm([5000, 390, -200], [5140, 440, 200], Wood),
    Solid::cm([5260, 610, -200], [5400, 660, 200], Wood),
    Solid::cm([5000, 830, -200], [5140, 880, 200], Wood),
    Solid::cm([5260, 1050, -200], [5400, 1100, 200], Wood),
    Solid::cm([5000, 1270, -200], [5140, 1320, 200], Wood),
    Solid::cm([5260, 1490, -200], [5400, 1540, 200], Wood),
    Solid::cm([5000, 1710, -200], [5140, 1760, 200], Wood),
    Solid::cm([5260, 1930, -200], [5400, 1980, 200], Wood),
    Solid::cm([5000, 2150, -200], [5140, 2200, 200], Wood),
    Solid::cm([5260, 2370, -200], [5400, 2420, 200], Wood),
    // The wood: trunks.
    Solid::cm([-7080, 0, 720], [-6920, 1400, 880], Wood),
    Solid::cm([-6080, 0, -1280], [-5920, 1400, -1120], Wood),
    Solid::cm([-3580, 0, 320], [-3420, 1400, 480], Wood),
    Solid::cm([-2080, 0, -880], [-1920, 1400, -720], Wood),
    Solid::cm([920, 0, 920], [1080, 1400, 1080], Wood),
    Solid::cm([2420, 0, -680], [2580, 1400, -520], Wood),
    // Cairns: at the start, in the wood, at the chimney's foot, on the band.
    Solid::cm([-8100, 0, 700], [-7900, 80, 900], Snow),
    Solid::cm([-100, 0, 300], [100, 80, 500], Snow),
    Solid::cm([4100, 0, 700], [4300, 80, 900], Snow),
    Solid::cm([7400, 0, -1100], [7600, 2680, -900], Snow),
    // The third waystone: the heights and the woods.
    Solid::cm([8550, 0, 400], [8650, 2950, 500], Stone),
];

/// How Pinewood joins the rest of the valley.
pub static PLACE: Place = Place {
    arena: ArenaId::PINEWOOD,
    kind: Kind::Reach,
    seams: &SEAMS,
    vines: &VINES,
    vents: &[],
};

const SEAMS: [Seam; 5] = [
    // Back to the Shelves.
    Seam {
        zone: Zone::cm([-9800, -50, -300], [-9050, 600, 300]),
        to: ArenaId::SHELVES,
        at: 3,
        marks: [
            Mark::cm(-8600, -150, (-7600, -150)),
            Mark::cm(-8600, 150, (-7600, 150)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Shelves",
    },
    // The Den: the Pair.
    Seam {
        zone: Zone::cm([-5350, -100, -2950], [-4650, 600, -2250]),
        to: ArenaId::PAIR,
        at: 0,
        marks: [
            Mark::cm(-5100, -1800, (-5100, -800)),
            Mark::cm(-4900, -1800, (-4900, -800)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Den: the Pair",
    },
    // The Hollows, under the roots: the Broodmother.
    Seam {
        zone: Zone::cm([-1350, -100, 2250], [-650, 600, 2950]),
        to: ArenaId::BROODMOTHER,
        at: 0,
        marks: [
            Mark::cm(-1100, 1800, (-1100, 800)),
            Mark::cm(-900, 1800, (-900, 800)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Hollows: the Broodmother",
    },
    // The Highlands, at the top of the stair: the Ridgeback.
    Seam {
        zone: Zone::cm([50, 1350, -2950], [750, 2000, -2500]),
        to: ArenaId::HIGHLANDS,
        at: 0,
        marks: [
            Mark::cm(300, -2320, (300, -1320)),
            Mark::cm(500, -2320, (500, -1320)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Highlands: the Ridgeback",
    },
    // On to the Saddle, past the third waystone.
    Seam {
        zone: Zone::cm([9050, 2550, -300], [9800, 3200, 300]),
        to: ArenaId::SADDLE,
        at: 0,
        marks: [
            Mark::cm(8600, -150, (7600, -150)),
            Mark::cm(8600, 150, (7600, 150)),
        ],
        gate: crate::valley::tier::THREE,
        waystone: Some(58),
        says: "the Saddle",
    },
];

/// Climbable faces: hold the jump to go up, crouch to go down.
const VINES: [Zone; 1] = [
    // Up the band's face: anybody, slowly.
    Zone::cm([6080, -100, 1000], [6200, 2700, 1300]),
];
