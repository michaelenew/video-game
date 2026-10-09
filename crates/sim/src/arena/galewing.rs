//! The Cliffs: the Galewing's arena, on the edge of the world map
//! (galewing.md §11). A plateau of bare rock and short grass with the sky on
//! three sides.
//!
//! **The floor of the game is at zero**, so a drop has to be a solid standing
//! on it: the plateau is a **12 m solid, 44 × 44 m**, and the floor round
//! three sides of it is **the shelf**, four metres wide, twelve metres below
//! -- a fall the fall rule charges 75 for (free to 9 m, 25 a metre past it).
//! The fourth side is a rock face. Everything on the plateau stands on that
//! solid's top.
//!
//! | What | Size | Why |
//! | --- | --- | --- |
//! | The plateau | 44 × 44 m, 12 m up | The fight; its edges are the Downwash's teeth |
//! | The shelf | 4 m wide, round three sides | Where a push off the edge lands you; walked back up |
//! | Stairs | six, one at each end of each side, 2 m steps | About four seconds back up, a hop each for everybody |
//! | The tower | 6 m square, 12 m above the plateau | The perch; climbed by eight ledges 1.5 m apart |
//! | Four standing stones | 2 m across, 2 to 2.5 m tall | The lee from the Downwash |
//! | The rock face | 8 m above the plateau | The north edge; nothing goes off it |
//!
//! Two **sites** tell the creature where things are without the species
//! knowing the arena: `circle`, the middle of the circle it flies, and
//! `perch`, the tower it rests on.

use super::rim::Rim;
use super::{Arena, ArenaId, Bounds, HuntMarks, Mark, Material, Solid, Spawns};
use crate::objective::Site;
use crate::species::SpeciesId;

use Material::{Grass, Rock, Stone};

/// The tower's middle, in centimetres. It is 6 m square; its top is 24 m
/// over the shelf, 12 m over the plateau.
pub const TOWER: (i32, i32) = (0, 900);

pub static ARENA: Arena = Arena {
    id: ArenaId::GALEWING,
    name: "Cliffs",
    creature: Some(SpeciesId::GALEWING),
    // The shelf and its lip are inside the bounds; the rock face is the
    // north edge.
    bounds: Bounds::cm((-2700, 2700), (-2700, 2400)),
    floor: Rock,
    regions: &[],
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [
            Mark::cm(-400, -800, (400, -800)),
            Mark::cm(400, -800, (-400, -800)),
        ],
        hunt: Some(HuntMarks {
            // In from the south, across the open plateau, toward the tower.
            hunters: [
                Mark::cm(-150, -1500, (0, 900)),
                Mark::cm(150, -1500, (0, 900)),
            ],
            // It starts on the far side of its circle, over the tower.
            creatures: [Mark::cm(0, 1400, (0, 0)), Mark::cm(400, 1400, (0, 0))],
        }),
    },
    sites: &SITES,
    rim: Some(Rim::new((-2700, 2700), (-2700, 2400), [150, 2000, 150, 150], 400).falling()),
};

/// One step of a stair: a column from the floor to `k` steps up, in the
/// strip against the cliff.
const fn step(x: (i32, i32), z: (i32, i32), k: i32) -> Solid {
    Solid::cm([x.0, 0, z.0], [x.1, 200 * k, z.1], Rock)
}

/// A standing stone: two metres across, `h` centimetres above the plateau.
const fn stone(x: i32, z: i32, h: i32) -> Solid {
    Solid::cm([x - 100, 0, z - 100], [x + 100, 1200 + h, z + 100], Stone)
}

/// The tower's ledges: the range's spiral (`arena::range`), round this tower.
const fn ledge(x: (i32, i32), z: (i32, i32), up: i32) -> Solid {
    Solid::cm([x.0, 0, z.0], [x.1, 1200 + up, z.1], Stone)
}

const SOLIDS: [Solid; 54] = [
    // The plateau.
    Solid::cm([-2200, 0, -2200], [2200, 1200, 2200], Grass),
    // The rock face along the north, eight metres above the plateau.
    Solid::cm([-2700, 0, 2200], [2700, 2000, 2400], Rock),
    // The shelf's lip: a low wall at its outer edge on three sides.
    Solid::cm([-2700, 0, -2700], [-2600, 150, 2200], Rock),
    Solid::cm([2600, 0, -2700], [2700, 150, 2200], Rock),
    Solid::cm([-2600, 0, -2700], [2600, 150, -2600], Rock),
    // The tower, twelve metres above the plateau...
    Solid::cm([-300, 0, 600], [300, 2400, 1200], Stone),
    // ...and its eight ledges, each 1.5 m above the last, spiralling round
    // it twice: a hop for everybody, about six seconds of climbing. The
    // fourth is the gallery at six metres.
    ledge((-450, -300), (600, 1200), 150),
    ledge((-300, 300), (450, 600), 300),
    ledge((300, 450), (600, 1200), 450),
    ledge((-300, 300), (1200, 1350), 600),
    ledge((-600, -450), (600, 1200), 750),
    ledge((-300, 300), (300, 450), 900),
    ledge((450, 600), (600, 1200), 1050),
    ledge((-300, 300), (1350, 1500), 1200),
    // Four standing stones.
    stone(-1200, -800, 220),
    stone(1300, -300, 250),
    stone(-800, -1600, 200),
    stone(1100, 1400, 230),
    // The west stairs: the south end climbing north, the north end
    // climbing south, against the cliff.
    step((-2400, -2200), (-1900, -1750), 1),
    step((-2400, -2200), (-1750, -1600), 2),
    step((-2400, -2200), (-1600, -1450), 3),
    step((-2400, -2200), (-1450, -1300), 4),
    step((-2400, -2200), (-1300, -1150), 5),
    step((-2400, -2200), (-1150, -1000), 6),
    step((-2400, -2200), (1750, 1900), 1),
    step((-2400, -2200), (1600, 1750), 2),
    step((-2400, -2200), (1450, 1600), 3),
    step((-2400, -2200), (1300, 1450), 4),
    step((-2400, -2200), (1150, 1300), 5),
    step((-2400, -2200), (1000, 1150), 6),
    // The east stairs, mirrored.
    step((2200, 2400), (-1900, -1750), 1),
    step((2200, 2400), (-1750, -1600), 2),
    step((2200, 2400), (-1600, -1450), 3),
    step((2200, 2400), (-1450, -1300), 4),
    step((2200, 2400), (-1300, -1150), 5),
    step((2200, 2400), (-1150, -1000), 6),
    step((2200, 2400), (1750, 1900), 1),
    step((2200, 2400), (1600, 1750), 2),
    step((2200, 2400), (1450, 1600), 3),
    step((2200, 2400), (1300, 1450), 4),
    step((2200, 2400), (1150, 1300), 5),
    step((2200, 2400), (1000, 1150), 6),
    // The south stairs, from each end toward the middle.
    step((-1900, -1750), (-2400, -2200), 1),
    step((-1750, -1600), (-2400, -2200), 2),
    step((-1600, -1450), (-2400, -2200), 3),
    step((-1450, -1300), (-2400, -2200), 4),
    step((-1300, -1150), (-2400, -2200), 5),
    step((-1150, -1000), (-2400, -2200), 6),
    step((1750, 1900), (-2400, -2200), 1),
    step((1600, 1750), (-2400, -2200), 2),
    step((1450, 1600), (-2400, -2200), 3),
    step((1300, 1450), (-2400, -2200), 4),
    step((1150, 1300), (-2400, -2200), 5),
    step((1000, 1150), (-2400, -2200), 6),
];

/// Where its circle is and where it perches.
const SITES: [Site; 2] = [
    // It circles the tower, fourteen metres out: never over it, and over
    // the rock face's top on its north side. Its height is the level it
    // measures its own from: the plateau's top.
    Site {
        name: "circle",
        route: &[TOWER],
        size: (1400, 1400, 1200),
        material: Grass,
    },
    Site {
        name: "perch",
        route: &[TOWER],
        size: (300, 300, 2400),
        material: Stone,
    },
];
