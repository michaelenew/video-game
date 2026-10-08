//! **The Shelves**: the third reach, the longest traverse in the valley
//! (`docs/design/valley.md`). You come in on a terrace twenty metres over a
//! hollow, and the way on runs along the north face above it.
//!
//! | Beat | What it asks |
//! | --- | --- |
//! | **The traverse** | A ledge a metre and a half wide along the north wall, twenty metres over the hollow, broken twice: five metres, then six. A snow balcony between the gaps. A fall costs two hundred and seventy-five |
//! | **The hollow** | The way down, by the vine on the terrace's face -- and the slow way across: walk the hollow and climb the vine up the second terrace, which skips the traverse and takes four times as long |
//! | The Mire | A notch in the south wall at the hollow's floor: the Mireback |
//! | **The bluff** | Seven-metre walls round a sandy basin in the south-east, the Pan's notch inside: over by the vine, or by a mechanic |
//! | **The chimney** | Ten metres up from the second terrace between a block and the top's face, four ledges alternating across a four-metre slot. A vine on the face for anyone, slowly |
//! | The top | **The second waystone**, lit by the Pan or the Mire, and the passage on to the Pinewood |

use super::relief::{Bump, Ramp};
use super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns};
use crate::valley::{Gate, Kind, Place, Seam, Zone};

use Material::{Grass, Ground, Peat, Rock, Sand, Snow, Stone};

pub static ARENA: Arena = Arena {
    id: ArenaId::SHELVES,
    name: "Shelves",
    creature: None,
    bounds: Bounds::cm((-11000, 11000), (-3200, 2400)),
    floor: Grass,
    regions: &REGIONS,
    solids: &SOLIDS,
    spawns: Spawns {
        // On the first terrace, just inside the passage from the Bank.
        versus: [
            Mark::cm(-9500, -150, (-8500, -150)),
            Mark::cm(-9500, 150, (-8500, 150)),
        ],
        hunt: None,
    },
    sites: &[],
};

/// The floor between the rock: gentle swells and hollows.
pub const BUMPS: [Bump; 3] = [
    Bump::cm(-4000, 0, 800, 60),
    Bump::cm(-800, -1400, 600, -40),
    Bump::cm(2000, 1000, 700, 50),
];

/// The floor climbing: what the valley rises by between its cliffs.
pub const RAMPS: [Ramp; 0] = [];

const REGIONS: [Region; 3] = [
    // Peat round the Mire's notch.
    Region {
        area: Area::disc_cm(-2200, -1800, 600),
        material: Peat,
    },
    // Sand in the Pan's basin.
    Region {
        area: Area::rect_cm((1200, 4000), (-2200, -800)),
        material: Sand,
    },
    // The hollow's trodden floor under the traverse.
    Region {
        area: Area::rect_cm((-6000, 4000), (1400, 2200)),
        material: Ground,
    },
];

const SOLIDS: [Solid; 41] = [
    // The south side, with the notches to the Mire and the Pan.
    Solid::cm([-10200, 0, -2400], [-2600, 4400, -2200], Rock),
    Solid::cm([-1800, 0, -2400], [2200, 4400, -2200], Rock),
    Solid::cm([2800, 0, -2400], [10200, 4400, -2200], Rock),
    // the notch at x -26 to -18: its sides and back
    Solid::cm([-2800, 0, -3000], [-2600, 1400, -2400], Rock),
    Solid::cm([-1800, 0, -3000], [-1600, 1400, -2400], Rock),
    Solid::cm([-2800, 0, -3200], [-1600, 1400, -3000], Rock),
    // the notch at x 22 to 28: its sides and back
    Solid::cm([2000, 0, -3000], [2200, 1400, -2400], Rock),
    Solid::cm([2800, 0, -3000], [3000, 1400, -2400], Rock),
    Solid::cm([2000, 0, -3200], [3000, 1400, -3000], Rock),
    // The north side.
    Solid::cm([-10200, 0, 2200], [10200, 4400, 2400], Rock),
    // The west end, on the first terrace: the passage back to the Bank.
    Solid::cm([-10200, 0, -2200], [-10000, 4400, -300], Rock),
    Solid::cm([-10200, 0, 300], [-10000, 4400, 2200], Rock),
    // the passage's floor
    Solid::cm([-10800, 0, -300], [-10000, 2000, 300], Rock),
    Solid::cm([-10800, 0, -500], [-10200, 3000, -300], Rock),
    Solid::cm([-10800, 0, 300], [-10200, 3000, 500], Rock),
    Solid::cm([-11000, 0, -500], [-10800, 3000, 500], Rock),
    // The east end, on the top: the passage on to the Pinewood.
    Solid::cm([10000, 0, -2200], [10200, 4400, -300], Rock),
    Solid::cm([10000, 0, 300], [10200, 4400, 2200], Rock),
    // the passage's floor
    Solid::cm([10000, 0, -300], [10800, 3200, 300], Rock),
    Solid::cm([10200, 0, -500], [10800, 4200, -300], Rock),
    Solid::cm([10200, 0, 300], [10800, 4200, 500], Rock),
    Solid::cm([10800, 0, -500], [11000, 4200, 500], Rock),
    // The first terrace, twenty metres over the hollow.
    Solid::cm([-10000, 0, -2200], [-6000, 2000, 2200], Grass),
    // The traverse: a ledge along the north wall, broken twice.
    Solid::cm([-6000, 1900, 2040], [-3000, 2050, 2200], Ground),
    Solid::cm([-2500, 1950, 2040], [500, 2100, 2200], Ground),
    // The balcony between the gaps: a cairn.
    Solid::cm([-1200, 2020, 1700], [-600, 2100, 2040], Snow),
    Solid::cm([1100, 2000, 2040], [4000, 2150, 2200], Ground),
    // The second terrace.
    Solid::cm([4000, 0, -2200], [6800, 2200, 2200], Grass),
    // The chimney's block, on the second terrace.
    Solid::cm([6000, 0, -400], [6400, 3200, 400], Rock),
    // The top, thirty-two metres up.
    Solid::cm([6800, 0, -2200], [10000, 3200, 2200], Grass),
    // The chimney's ledges, alternating across the slot.
    Solid::cm([6660, 2370, -300], [6800, 2420, 300], Rock),
    Solid::cm([6400, 2590, -300], [6540, 2640, 300], Rock),
    Solid::cm([6660, 2810, -300], [6800, 2860, 300], Rock),
    Solid::cm([6400, 3030, -300], [6540, 3080, 300], Rock),
    // The bluff round the Pan's basin, seven metres.
    Solid::cm([1000, 0, -2200], [1200, 700, -600], Stone),
    Solid::cm([1200, 0, -800], [4000, 700, -600], Stone),
    // Cairns: on the first terrace, in the hollow, on the second terrace, on the top.
    Solid::cm([-8100, 0, 700], [-7900, 2080, 900], Snow),
    Solid::cm([-3600, 0, -1300], [-3400, 80, -1100], Snow),
    Solid::cm([4900, 0, -1100], [5100, 2280, -900], Snow),
    Solid::cm([8400, 0, 900], [8600, 3280, 1100], Snow),
    // The second waystone: the lowlands.
    Solid::cm([9550, 0, 400], [9650, 3550, 500], Stone),
];

/// How Shelves joins the rest of the valley.
pub static PLACE: Place = Place {
    arena: ArenaId::SHELVES,
    kind: Kind::Reach,
    seams: &SEAMS,
    vines: &VINES,
    vents: &[],
};

const SEAMS: [Seam; 4] = [
    // Back to the Bank.
    Seam {
        zone: Zone::cm([-10800, 1950, -300], [-10050, 2600, 300]),
        to: ArenaId::BANK,
        at: 2,
        marks: [
            Mark::cm(-9500, -150, (-8500, -150)),
            Mark::cm(-9500, 150, (-8500, 150)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Bank",
    },
    // The Mire, in the hollow's south wall: the Mireback.
    Seam {
        zone: Zone::cm([-2550, -100, -2950], [-1850, 600, -2250]),
        to: ArenaId::MIREBACK,
        at: 0,
        marks: [
            Mark::cm(-2300, -1800, (-2300, -800)),
            Mark::cm(-2100, -1800, (-2100, -800)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Mire: the Mireback",
    },
    // The Pan, inside the bluff: the Sandmaw.
    Seam {
        zone: Zone::cm([2250, -100, -2950], [2750, 600, -2250]),
        to: ArenaId::SANDMAW,
        at: 0,
        marks: [
            Mark::cm(2400, -1600, (2400, -600)),
            Mark::cm(2600, -1600, (2600, -600)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Pan: the Sandmaw",
    },
    // On to the Pinewood, past the second waystone.
    Seam {
        zone: Zone::cm([10050, 3150, -300], [10800, 3800, 300]),
        to: ArenaId::PINEWOOD,
        at: 0,
        marks: [
            Mark::cm(9500, -150, (8500, -150)),
            Mark::cm(9500, 150, (8500, 150)),
        ],
        gate: crate::valley::tier::TWO,
        waystone: Some(40),
        says: "the Pinewood",
    },
];

/// Climbable faces: hold the jump to go up, crouch to go down.
const VINES: [Zone; 5] = [
    // Down and up the first terrace's face, to the hollow.
    Zone::cm([-6000, -100, -1600], [-5880, 2100, -1200]),
    // From the hollow up the second terrace: the slow way round the traverse.
    Zone::cm([3880, -100, 0], [4000, 2300, 400]),
    // Over the bluff, from outside...
    Zone::cm([880, -100, -1600], [1000, 800, -1200]),
    // ...and out again, from inside.
    Zone::cm([2000, -100, -920], [2400, 800, -800]),
    // Up the top's face beside the chimney: anybody, slowly.
    Zone::cm([6680, 2100, 1400], [6800, 3300, 1700]),
];
