//! **The Bank**: the second reach, a meadow and then the bank the Gnawers' den is
//! dug into (`docs/design/valley.md`). The bank **is** the climb.
//!
//! | Beat | What it asks |
//! | --- | --- |
//! | The meadow | Nothing: walk up to the foot of the bank, where the den's mouth is a dark notch at floor level -- the Gnawers' room |
//! | **The stair** | Ten shelves zig-zagging up the bank's face, 2.2 m apart, with a snow balcony a third of the way up. Twenty-four metres: a fall from the top costs three hundred. A vine at the south end for anyone, slowly |
//! | **The gully** | Six metres across, the trail at its bottom ten metres down. Fall in, and the vine on its far side brings you out |
//! | The far bank | **The first waystone**: lit when the herd or the den has been beaten. The passage on to the Shelves is past it |

use super::relief::{Bump, Ramp};
use super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns};
use crate::valley::{Gate, Kind, Place, Seam, Zone};

use Material::{Grass, Ground, Rock, Snow, Stone};

pub static ARENA: Arena = Arena {
    id: ArenaId::BANK,
    name: "Bank",
    creature: None,
    bounds: Bounds::cm((-8600, 8600), (-2400, 2400)),
    floor: Grass,
    regions: &REGIONS,
    solids: &SOLIDS,
    spawns: Spawns {
        // Just inside the passage from the Mouth, facing the bank.
        versus: [
            Mark::cm(-7200, -150, (-6200, -150)),
            Mark::cm(-7200, 150, (-6200, 150)),
        ],
        hunt: None,
    },
    sites: &[],
};

/// The floor between the rock: gentle swells and hollows.
pub const BUMPS: [Bump; 3] = [
    Bump::cm(-5000, -1000, 800, 50),
    Bump::cm(-3000, 1200, 700, 60),
    Bump::cm(-6000, 1000, 600, -40),
];

/// The floor climbing: what the valley rises by between its cliffs.
pub const RAMPS: [Ramp; 0] = [];

const REGIONS: [Region; 3] = [
    // Bare earth trodden out of the den's mouth.
    Region {
        area: Area::disc_cm(-400, 0, 500),
        material: Ground,
    },
    // Loose stone along the bank's foot.
    Region {
        area: Area::rect_cm((-400, 0), (-2200, 2200)),
        material: Rock,
    },
    // The trail along the gully's bottom.
    Region {
        area: Area::rect_cm((3000, 3600), (-2200, 2200)),
        material: Ground,
    },
];

const SOLIDS: [Solid; 32] = [
    // The valley's sides.
    Solid::cm([-7800, 0, -2400], [7800, 3600, -2200], Rock),
    Solid::cm([-7800, 0, 2200], [7800, 3600, 2400], Rock),
    // The west end: the passage back to the Mouth.
    Solid::cm([-7800, 0, -2200], [-7600, 3600, -300], Rock),
    Solid::cm([-7800, 0, 300], [-7600, 3600, 2200], Rock),
    Solid::cm([-8400, 0, -500], [-7800, 1000, -300], Rock),
    Solid::cm([-8400, 0, 300], [-7800, 1000, 500], Rock),
    Solid::cm([-8600, 0, -500], [-8400, 1000, 500], Rock),
    // The east end, on the far bank: the passage on to the Shelves.
    Solid::cm([7600, 0, -2200], [7800, 3600, -300], Rock),
    Solid::cm([7600, 0, 300], [7800, 3600, 2200], Rock),
    // the passage's floor
    Solid::cm([7600, 0, -300], [8400, 2450, 300], Rock),
    Solid::cm([7800, 0, -500], [8400, 3450, -300], Rock),
    Solid::cm([7800, 0, 300], [8400, 3450, 500], Rock),
    Solid::cm([8400, 0, -500], [8600, 3450, 500], Rock),
    // The bank, twenty-four metres, either side of the den's mouth.
    Solid::cm([0, 0, -2200], [3000, 2400, -300], Grass),
    Solid::cm([0, 0, 300], [3000, 2400, 2200], Grass),
    // Over the den's mouth, and the bank behind it.
    Solid::cm([0, 500, -300], [800, 2400, 300], Rock),
    Solid::cm([800, 0, -300], [3000, 2400, 300], Grass),
    // The gully's bottom: the trail, ten metres under the tops.
    Solid::cm([3000, 0, -2200], [3600, 1400, 2200], Rock),
    // The far bank, half a metre higher.
    Solid::cm([3600, 0, -2200], [7600, 2450, 2200], Grass),
    // The stair: ten shelves up the bank's face.
    Solid::cm([-160, 170, -1700], [0, 220, -1400], Grass),
    Solid::cm([-160, 390, -1300], [0, 440, -1000], Grass),
    Solid::cm([-160, 610, -900], [0, 660, -600], Grass),
    // The balcony, a third of the way up: a cairn.
    Solid::cm([-300, 830, -500], [0, 880, 100], Snow),
    Solid::cm([-160, 1050, 200], [0, 1100, 500], Grass),
    Solid::cm([-160, 1270, 600], [0, 1320, 900], Grass),
    Solid::cm([-160, 1490, 1000], [0, 1540, 1300], Grass),
    Solid::cm([-160, 1710, 1400], [0, 1760, 1700], Grass),
    Solid::cm([-160, 1930, 1800], [0, 1980, 2100], Grass),
    // The last, back over the eighth.
    Solid::cm([-160, 2150, 1400], [0, 2200, 1700], Grass),
    // A cairn in the meadow.
    Solid::cm([-4100, 0, 700], [-3900, 80, 900], Snow),
    // A cairn on the far bank.
    Solid::cm([4900, 0, -1100], [5100, 2530, -900], Snow),
    // The first waystone: the Commons.
    Solid::cm([7150, 0, 400], [7250, 2750, 500], Stone),
];

/// How Bank joins the rest of the valley.
pub static PLACE: Place = Place {
    arena: ArenaId::BANK,
    kind: Kind::Reach,
    seams: &SEAMS,
    vines: &VINES,
    vents: &[],
};

const SEAMS: [Seam; 3] = [
    // Back to the Mouth.
    Seam {
        zone: Zone::cm([-8400, -50, -300], [-7650, 600, 300]),
        to: ArenaId::MOUTH,
        at: 2,
        marks: [
            Mark::cm(-7200, -150, (-6200, -150)),
            Mark::cm(-7200, 150, (-6200, 150)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Mouth",
    },
    // The den's mouth: the Gnawers.
    Seam {
        zone: Zone::cm([50, -100, -250], [750, 400, 250]),
        to: ArenaId::GNAWERS,
        at: 0,
        marks: [
            Mark::cm(-500, -150, (-1500, -150)),
            Mark::cm(-500, 150, (-1500, 150)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the den: the Gnawers",
    },
    // On to the Shelves, past the first waystone.
    Seam {
        zone: Zone::cm([7650, 2400, -300], [8400, 3050, 300]),
        to: ArenaId::SHELVES,
        at: 0,
        marks: [
            Mark::cm(7200, -150, (6200, -150)),
            Mark::cm(7200, 150, (6200, 150)),
        ],
        gate: crate::valley::tier::ONE,
        waystone: Some(31),
        says: "the Shelves",
    },
];

/// Climbable faces: hold the jump to go up, crouch to go down.
const VINES: [Zone; 2] = [
    // Up the bank's face at its south end: anybody, slowly.
    Zone::cm([-120, -100, -2150], [0, 2500, -1900]),
    // Out of the gully, up the far bank.
    Zone::cm([3480, 1300, -200], [3600, 2550, 200]),
];
