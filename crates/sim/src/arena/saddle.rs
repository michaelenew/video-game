//! **The Saddle**: the last reach, the one open traverse in the valley
//! (`docs/design/valley.md`). A ridge three metres wide, twenty metres up,
//! between two drops; wind.
//!
//! | Beat | What it asks |
//! | --- | --- |
//! | **The ridge** | Eighty metres along its top with a fall either side that costs two hundred and seventy-five. Two gaps, five and six metres: an **updraft** rises out of each, and jumping into it carries you up three metres to drift across |
//! | The floor | The way down is the vines on the west platform's face; the way back up is the floor's two updrafts, one under each end of the ridge. The Cliffs (the Galewing) open off the north wall, the Ashwood (the Veilstalker) off the south |
//! | **The last chimney** | Two spires on the east terrace, fourteen metres, six ledges alternating up the slot between them |
//! | **The Shrine** | On the second spire's top, under **the fourth waystone**: one of the Galewing and the Veilstalker |
//! | The Long Valley | The passage at the far end, under **the fifth waystone**, lit by the Mantis: the Siegeshell walks it home toward Hearth |

use super::relief::{Bump, Ramp};
use super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns};
use crate::valley::{Gate, Kind, Place, Seam, Vent, Zone};

use Material::{Ash, Grass, Rock, Snow, Stone};

pub static ARENA: Arena = Arena {
    id: ArenaId::SADDLE,
    name: "Saddle",
    creature: None,
    bounds: Bounds::cm((-8500, 8500), (-3200, 3200)),
    floor: Grass,
    regions: &REGIONS,
    solids: &SOLIDS,
    spawns: Spawns {
        // On the west platform, just inside the passage from the Pinewood.
        versus: [
            Mark::cm(-7000, -150, (-6000, -150)),
            Mark::cm(-7000, 150, (-6000, 150)),
        ],
        hunt: None,
    },
    sites: &[],
};

/// The floor between the rock: gentle swells and hollows.
pub const BUMPS: [Bump; 3] = [
    Bump::cm(-3000, -1200, 800, 60),
    Bump::cm(-1000, 1400, 700, 50),
    Bump::cm(1500, -1000, 700, -40),
];

/// The floor climbing: what the valley rises by between its cliffs.
pub const RAMPS: [Ramp; 0] = [];

const REGIONS: [Region; 2] = [
    // Old snow on the alpine floor under the ridge, where the sun never reaches.
    Region {
        area: Area::rect_cm((-5000, 3000), (-1200, 1200)),
        material: Snow,
    },
    // Snow on the platforms' tops shows nothing: they are solids. Ash by the Ashwood's notch.
    Region {
        area: Area::disc_cm(0, -1800, 500),
        material: Ash,
    },
];

const SOLIDS: [Solid; 42] = [
    // The south side, with the Ashwood's notch.
    Solid::cm([-7700, 0, -2400], [-400, 4600, -2200], Rock),
    Solid::cm([400, 0, -2400], [7700, 4600, -2200], Rock),
    // the notch at x -4 to 4: its sides and back
    Solid::cm([-600, 0, -3000], [-400, 1400, -2400], Rock),
    Solid::cm([400, 0, -3000], [600, 1400, -2400], Rock),
    Solid::cm([-600, 0, -3200], [600, 1400, -3000], Rock),
    // The north side, with the Cliffs' notch.
    Solid::cm([-7700, 0, 2200], [-2400, 4600, 2400], Rock),
    Solid::cm([-1600, 0, 2200], [7700, 4600, 2400], Rock),
    // the notch at x -24 to -16: its sides and back
    Solid::cm([-2600, 0, 2400], [-2400, 1400, 3000], Rock),
    Solid::cm([-1600, 0, 2400], [-1400, 1400, 3000], Rock),
    Solid::cm([-2600, 0, 3000], [-1400, 1400, 3200], Rock),
    // The west end, on the west platform: the passage back to the Pinewood.
    Solid::cm([-7700, 0, -2200], [-7500, 4600, -300], Rock),
    Solid::cm([-7700, 0, 300], [-7500, 4600, 2200], Rock),
    // the passage's floor
    Solid::cm([-8300, 0, -300], [-7500, 2000, 300], Rock),
    Solid::cm([-8300, 0, -500], [-7700, 3000, -300], Rock),
    Solid::cm([-8300, 0, 300], [-7700, 3000, 500], Rock),
    Solid::cm([-8500, 0, -500], [-8300, 3000, 500], Rock),
    // The east end, on the east terrace: the passage to the Long Valley.
    Solid::cm([7500, 0, -2200], [7700, 4600, -300], Rock),
    Solid::cm([7500, 0, 300], [7700, 4600, 2200], Rock),
    // the passage's floor
    Solid::cm([7500, 0, -300], [8300, 2200, 300], Rock),
    Solid::cm([7700, 0, -500], [8300, 3200, -300], Rock),
    Solid::cm([7700, 0, 300], [8300, 3200, 500], Rock),
    Solid::cm([8300, 0, -500], [8500, 3200, 500], Rock),
    // The west platform, twenty metres up: turf over rock.
    Solid::cm([-7500, 0, -2200], [-5000, 2000, 2200], Grass),
    // The ridge, three metres wide, broken twice.
    Solid::cm([-5000, 0, -150], [-3000, 2000, 150], Rock),
    Solid::cm([-2500, 0, -150], [500, 2000, 150], Rock),
    // A widening on the ridge: a cairn.
    Solid::cm([-1400, 0, -400], [-800, 2040, 400], Snow),
    Solid::cm([1100, 0, -150], [3000, 2000, 150], Rock),
    // The east terrace, two metres higher: turf over rock.
    Solid::cm([3000, 0, -2200], [7500, 2200, 2200], Grass),
    // The two spires of the last chimney, fourteen metres over the terrace.
    Solid::cm([4400, 0, -300], [4800, 3600, 300], Rock),
    Solid::cm([5200, 0, -300], [5600, 3600, 300], Rock),
    // The chimney's ledges, alternating.
    Solid::cm([5060, 2370, -300], [5200, 2420, 300], Rock),
    Solid::cm([4800, 2590, -300], [4940, 2640, 300], Rock),
    Solid::cm([5060, 2810, -300], [5200, 2860, 300], Rock),
    Solid::cm([4800, 3030, -300], [4940, 3080, 300], Rock),
    Solid::cm([5060, 3250, -300], [5200, 3300, 300], Rock),
    Solid::cm([4800, 3470, -300], [4940, 3520, 300], Rock),
    // Cairns: the west platform, the east terrace, the floor.
    Solid::cm([-6600, 0, 700], [-6400, 2080, 900], Snow),
    Solid::cm([3500, 0, 900], [3700, 2280, 1100], Snow),
    Solid::cm([-1100, 0, -1300], [-900, 80, -1100], Snow),
    // The fourth waystone, on the first spire: the edges.
    Solid::cm([4450, 0, -250], [4550, 3900, -150], Stone),
    // The fifth waystone: the Shrine.
    Solid::cm([7050, 0, 400], [7150, 2500, 500], Stone),
    // The bridge from the second spire's top to the Shrine, through the
    // north wall, thirty-six metres over the floor.
    Solid::cm([5250, 3550, 300], [5550, 3600, 3200], Rock),
];

/// How Saddle joins the rest of the valley.
pub static PLACE: Place = Place {
    arena: ArenaId::SADDLE,
    kind: Kind::Reach,
    seams: &SEAMS,
    vines: &VINES,
    vents: &VENTS,
};

const SEAMS: [Seam; 4] = [
    // Back to the Pinewood.
    Seam {
        zone: Zone::cm([-8300, 1950, -300], [-7550, 2600, 300]),
        to: ArenaId::PINEWOOD,
        at: 4,
        marks: [
            Mark::cm(-7000, -150, (-6000, -150)),
            Mark::cm(-7000, 150, (-6000, 150)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Pinewood",
    },
    // The Cliffs, off the floor in the north wall: the Galewing.
    Seam {
        zone: Zone::cm([-2350, -100, 2250], [-1650, 600, 2950]),
        to: ArenaId::GALEWING,
        at: 0,
        marks: [
            Mark::cm(-2100, 1800, (-2100, 800)),
            Mark::cm(-1900, 1800, (-1900, 800)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Cliffs: the Galewing",
    },
    // The Ashwood, off the floor in the south wall: the Veilstalker.
    Seam {
        zone: Zone::cm([-350, -100, -2950], [350, 600, -2250]),
        to: ArenaId::VEILSTALKER,
        at: 0,
        marks: [
            Mark::cm(-100, -1800, (-100, -800)),
            Mark::cm(100, -1800, (100, -800)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Ashwood: the Veilstalker",
    },
    // The Shrine, over the bridge from the second spire's top, past the
    // fourth waystone: the Mantis.
    Seam {
        zone: Zone::cm([5250, 3550, 1800], [5550, 4200, 2200]),
        to: ArenaId::MANTIS,
        at: 0,
        marks: [
            Mark::cm(5330, 1400, (5330, 0)),
            Mark::cm(5470, 1400, (5470, 0)),
        ],
        gate: crate::valley::tier::FOUR,
        waystone: Some(39),
        says: "the Shrine: the Mantis",
    },
];

/// Climbable faces: hold the jump to go up, crouch to go down.
const VINES: [Zone; 3] = [
    // Down the west platform's face to the floor, north...
    Zone::cm([-5000, -100, 1400], [-4880, 2100, 1700]),
    // ...and south.
    Zone::cm([-5000, -100, -1700], [-4880, 2100, -1400]),
    // Up the second spire's far face to the Shrine: anybody, slowly.
    Zone::cm([5600, 2100, -200], [5720, 3700, 200]),
];

/// Updrafts: anybody in the air in one is carried to its top.
const VENTS: [Vent; 4] = [
    // In the ridge's first gap.
    Vent::cm(-2750, 0, 220, 2300),
    // In its second.
    Vent::cm(800, 0, 260, 2300),
    // Off the floor, up against the west platform's face.
    Vent::cm(-4800, 1000, 180, 2250),
    // Off the floor, up against the east terrace's face.
    Vent::cm(2800, -1200, 180, 2450),
];
