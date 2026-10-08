//! **The Mouth**: the first reach of the valley, from Hearth's gate to the Lip
//! (`docs/design/valley.md`). A hundred and ninety metres and twenty of rise.
//!
//! | Beat | What it asks |
//! | --- | --- |
//! | The river terrace | Nothing: walk out of the gate with the river on your left and the herd's meadow through a notch on your right |
//! | The scree | A slope of loose stone, five metres up over eighteen, boulders to weave between |
//! | **The Step**, an eight-metre wall | Three grass shelves up its face, two metres apart: a hop each. A vine at its north end for anyone, slowly |
//! | **The gap** | Six metres and half a metre up, from the Step to the second terrace, over a slot eight metres deep. Fall in and the vine on the far side brings you out |
//! | **The traverse** | A jump of two and a half metres onto the first of three ledges along the Lip's face, then up them two metres at a time, over a slot fifteen metres deep: a fall from the last one costs a hundred and twenty-five |
//! | The Lip | Twenty metres up, and the passage on to the Bank |
//!
//! Snow on a top is a cairn: touch it to rest, and a death anywhere in the reach
//! stands you on the last one you touched.

use super::relief::{Bump, Ramp};
use super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns};
use crate::valley::{Gate, Kind, Place, Seam, Zone};

use Material::{Grass, Ground, Rock, Snow, Water};

pub static ARENA: Arena = Arena {
    id: ArenaId::MOUTH,
    name: "Mouth",
    creature: None,
    bounds: Bounds::cm((-10600, 10600), (-2400, 3200)),
    floor: Grass,
    regions: &REGIONS,
    solids: &SOLIDS,
    spawns: Spawns {
        // Just inside the passage from Hearth, facing up the valley.
        versus: [
            Mark::cm(-9200, -150, (-8200, -150)),
            Mark::cm(-9200, 150, (-8200, 150)),
        ],
        hunt: None,
    },
    sites: &[],
};

/// The floor between the rock: gentle swells and hollows.
pub const BUMPS: [Bump; 3] = [
    Bump::cm(-8000, 1000, 800, 50),
    Bump::cm(-7000, -600, 700, -40),
    Bump::cm(-5000, 1200, 600, 60),
];

/// The floor climbing: what the valley rises by between its cliffs.
pub const RAMPS: [Ramp; 1] = [Ramp::x(-3800, -2000, 500)];

const REGIONS: [Region; 5] = [
    // The river, along the terrace.
    Region {
        area: Area::rect_cm((-9600, -4000), (-2100, -1400)),
        material: Water,
    },
    // The path out of the gate.
    Region {
        area: Area::rect_cm((-10400, -3800), (-250, 250)),
        material: Ground,
    },
    // The scree: loose stone.
    Region {
        area: Area::rect_cm((-3800, -1800), (-2200, 2200)),
        material: Rock,
    },
    // Trodden earth in the slots under the gap and the traverse.
    Region {
        area: Area::rect_cm((4000, 4600), (-2200, 2200)),
        material: Ground,
    },
    // The traverse's slot.
    Region {
        area: Area::rect_cm((7400, 7800), (-2200, 2200)),
        material: Ground,
    },
];

const SOLIDS: [Solid; 34] = [
    // The valley's sides: cliffs twelve metres over the Lip.
    Solid::cm([-9800, 0, -2400], [9800, 3200, -2200], Rock),
    // The north side, with the notch to the low meadow.
    Solid::cm([-9800, 0, 2200], [-6600, 3200, 2400], Rock),
    Solid::cm([-5600, 0, 2200], [9800, 3200, 2400], Rock),
    // the notch at x -66 to -56: its sides and back
    Solid::cm([-6800, 0, 2400], [-6600, 1400, 3000], Rock),
    Solid::cm([-5600, 0, 2400], [-5400, 1400, 3000], Rock),
    Solid::cm([-6800, 0, 3000], [-5400, 1400, 3200], Rock),
    // The west end: the passage back to Hearth.
    Solid::cm([-9800, 0, -2200], [-9600, 3200, -300], Rock),
    Solid::cm([-9800, 0, 300], [-9600, 3200, 2200], Rock),
    Solid::cm([-10400, 0, -500], [-9800, 1000, -300], Rock),
    Solid::cm([-10400, 0, 300], [-9800, 1000, 500], Rock),
    Solid::cm([-10600, 0, -500], [-10400, 1000, 500], Rock),
    // The east end, on the Lip: the passage on to the Bank.
    Solid::cm([9600, 0, -2200], [9800, 3200, -300], Rock),
    Solid::cm([9600, 0, 300], [9800, 3200, 2200], Rock),
    // the passage's floor
    Solid::cm([9600, 0, -300], [10400, 2000, 300], Rock),
    Solid::cm([9800, 0, -500], [10400, 3000, -300], Rock),
    Solid::cm([9800, 0, 300], [10400, 3000, 500], Rock),
    Solid::cm([10400, 0, -500], [10600, 3000, 500], Rock),
    // Boulders on the scree.
    Solid::cm([-3520, 0, -920], [-3280, 180, -680], Rock),
    Solid::cm([-3150, 0, 300], [-2850, 360, 500], Rock),
    Solid::cm([-2700, 0, -1600], [-2500, 500, -1400], Rock),
    Solid::cm([-2420, 0, 980], [-2180, 620, 1220], Rock),
    // A cairn at the top of the scree.
    Solid::cm([-1500, 0, 700], [-1300, 600, 900], Snow),
    // The Step: an eight-metre wall the whole way across.
    Solid::cm([1000, 0, -2200], [4000, 1300, 2200], Grass),
    // Its three shelves, two metres apart up the face.
    Solid::cm([840, 670, -800], [1000, 720, -500], Grass),
    Solid::cm([840, 880, -400], [1000, 930, -100], Grass),
    Solid::cm([840, 1090, 0], [1000, 1140, 300], Grass),
    // A cairn on the Step.
    Solid::cm([1900, 0, -900], [2100, 1380, -700], Snow),
    // The second terrace, six metres on and half a metre up.
    Solid::cm([4600, 0, -2200], [7400, 1350, 2200], Grass),
    // A cairn before the traverse.
    Solid::cm([5900, 0, 900], [6100, 1430, 1100], Snow),
    // The Lip, twenty metres up.
    Solid::cm([7800, 0, -2200], [9600, 2000, 2200], Grass),
    // The traverse: three ledges along the Lip's face.
    Solid::cm([7660, 1450, -2000], [7800, 1500, -800], Ground),
    Solid::cm([7660, 1700, -600], [7800, 1750, 600], Ground),
    Solid::cm([7660, 1860, 800], [7800, 1910, 2000], Ground),
    // A cairn on the Lip.
    Solid::cm([8700, 0, -900], [8900, 2080, -700], Snow),
];

/// How Mouth joins the rest of the valley.
pub static PLACE: Place = Place {
    arena: ArenaId::MOUTH,
    kind: Kind::Reach,
    seams: &SEAMS,
    vines: &VINES,
    vents: &[],
};

const SEAMS: [Seam; 3] = [
    // Back to Hearth.
    Seam {
        zone: Zone::cm([-10400, -50, -300], [-9650, 600, 300]),
        to: ArenaId::HEARTH,
        at: 0,
        marks: [
            Mark::cm(-9200, -150, (-8200, -150)),
            Mark::cm(-9200, 150, (-8200, 150)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "Hearth",
    },
    // The low meadow, through the notch: the Hornback herd.
    Seam {
        zone: Zone::cm([-6550, -100, 2250], [-5650, 600, 2950]),
        to: ArenaId::HORNBACK,
        at: 0,
        marks: [
            Mark::cm(-6300, 1800, (-6300, 800)),
            Mark::cm(-5900, 1800, (-5900, 800)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the low meadow: the Hornback herd",
    },
    // On to the Bank.
    Seam {
        zone: Zone::cm([9650, 1950, -300], [10400, 2600, 300]),
        to: ArenaId::BANK,
        at: 0,
        marks: [
            Mark::cm(9200, -150, (8200, -150)),
            Mark::cm(9200, 150, (8200, 150)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Bank",
    },
];

/// Climbable faces: hold the jump to go up, crouch to go down.
const VINES: [Zone; 3] = [
    // Up the Step's face, at its north end: anybody, slowly.
    Zone::cm([880, 400, 1400], [1000, 1400, 1700]),
    // Out of the gap's slot, up the second terrace.
    Zone::cm([4480, 400, -400], [4600, 1450, 400]),
    // Out of the traverse's slot, back up the second terrace.
    Zone::cm([7400, 400, -200], [7520, 1450, 200]),
];
