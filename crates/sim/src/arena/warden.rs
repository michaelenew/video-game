//! **The Warden's tower** (`--arena warden`): a blockout of the opening
//! (`docs/design/opening.md`), built to be looked at and walked round, not
//! fought in. Nothing here is tuned, and there is no creature: the Warden is
//! drawn by the game as shapes that nothing collides with
//! (`game::arenas::warden`), so it can be posed for a screenshot.
//!
//! Two parts, side by side:
//!
//! - **The shaft**, west: an octagon forty metres across and forty-five
//!   tall, open to the sky -- the Warden's floor. Six columns stand in a
//!   ring thirteen metres out, tops from seven to thirteen metres (perches),
//!   with one already broken to a stump and one fallen. Scaffolding on two
//!   walls, a stair of ledges up the north wall to **the return ledge**
//!   fifteen metres up the west wall (the dive), and **the gallery** on the
//!   east wall at thirteen metres, where the climb comes out.
//! - **The climb**, east: a well ten metres square and twenty-two deep,
//!   beside the shaft. **The cell** is at its bottom, its roof broken; the
//!   way out is up the well by ledges a jump and a catch apart, past **a
//!   window** into the shaft at seven metres, to the gallery's door.
//!
//! | Height | What |
//! | --- | --- |
//! | 0 | The cell; the shaft's floor |
//! | 3.5 | The cell's broken roof |
//! | 7.5 | The window ledge, looking into the shaft |
//! | 13 | The gallery, and its door in the shaft's east wall |
//! | 15 | The return ledge, over the floor's west side |
//! | 22 | The top of the well |
//! | 45 | The top of the shaft |

use super::hills::slab;
use super::rim::Rim;
use super::{Arena, ArenaId, Bounds, Mark, Material, Shape, Solid, Spawns};
use crate::fixed::Fx;
use crate::math::V3;

use Material::{Rock, Stone, Wood};

pub static ARENA: Arena = Arena {
    id: ArenaId::WARDEN,
    name: "Warden",
    creature: None,
    bounds: Bounds::cm((-2300, 3500), (-2300, 2300)),
    floor: Rock,
    regions: &[],
    solids: &SOLIDS,
    spawns: Spawns {
        // In the cell, looking at the bars; and on the shaft's floor.
        versus: [Mark::cm(2950, 0, (2200, 0)), Mark::cm(-400, 0, (2000, 0))],
        hunt: None,
    },
    sites: &[],
    // The tower is cut into a cliff: the land starts at the top of each
    // wall, and the well's side is lower.
    rim: Some(Rim::new(
        (-2200, 3400),
        (-2200, 2200),
        [SHAFT_TOP, SHAFT_TOP, SHAFT_TOP, WELL_TOP],
        1200,
    )),
};

/// How high the shaft's walls go, and the well's.
const SHAFT_TOP: i32 = 4500;
const WELL_TOP: i32 = 2200;

/// A column: round, `r` across (a radius), `top` tall, at (x, z).
const fn column(x: i32, z: i32, r: i32, top: i32) -> Solid {
    Solid {
        min: V3::new(super::cm(x - r), Fx::ZERO, super::cm(z - r)),
        max: V3::new(super::cm(x + r), super::cm(top), super::cm(z + r)),
        material: Stone,
        shape: Shape::Round {
            bottom: super::cm(r),
            top: super::cm(r),
            hollow: Fx::ZERO,
        },
    }
}

/// One of the shaft's four diagonal walls, centred on (x, z) and lying
/// along (dx, dz).
const fn diagonal(x: i32, z: i32, along: (i32, i32)) -> Solid {
    slab(x, z, (0, SHAFT_TOP), (880, 100), along, Stone)
}

const SOLIDS: [Solid; 43] = [
    // --- the shaft ---------------------------------------------------------
    // North, south and west walls, whole.
    Solid::cm([-830, 0, 2000], [830, SHAFT_TOP, 2200], Stone),
    Solid::cm([-830, 0, -2200], [830, SHAFT_TOP, -2000], Stone),
    Solid::cm([-2200, 0, -830], [-2000, SHAFT_TOP, 830], Stone),
    // The east wall, between the shaft and the well: whole to seven metres,
    // a window three metres wide to nine and a half, whole to thirteen, the
    // gallery's door to sixteen, and whole above.
    Solid::cm([2000, 0, -830], [2200, 700, 830], Stone),
    Solid::cm([2000, 700, -830], [2200, 950, -150], Stone),
    Solid::cm([2000, 700, 150], [2200, 950, 830], Stone),
    Solid::cm([2000, 950, -830], [2200, 1300, 830], Stone),
    Solid::cm([2000, 1300, -830], [2200, 1600, -200], Stone),
    Solid::cm([2000, 1300, 200], [2200, 1600, 830], Stone),
    Solid::cm([2000, 1600, -830], [2200, SHAFT_TOP, 830], Stone),
    // The four diagonals.
    diagonal(1485, 1485, (-1, 1)),
    diagonal(-1485, 1485, (1, 1)),
    diagonal(-1485, -1485, (-1, 1)),
    diagonal(1485, -1485, (1, 1)),
    // --- the columns: perches, thirteen metres out -------------------------
    column(1126, 650, 70, 1000),
    column(0, 1300, 70, 1300),
    column(-1126, 650, 70, 800),
    column(-1126, -650, 70, 1100),
    column(0, -1300, 70, 900),
    column(1126, -650, 70, 700),
    // A column already broken: its stump, and the rest of it lying on the
    // floor toward the middle.
    column(-700, -100, 70, 260),
    slab(-350, 300, (0, 140), (450, 70), (1, 1), Stone),
    // --- scaffolding -------------------------------------------------------
    Solid::cm([-1950, 480, 900], [-1300, 530, 1500], Wood),
    Solid::cm([-1950, 830, -1500], [-1300, 880, -900], Wood),
    // A landing under the gallery, a drop from it.
    Solid::cm([1550, 600, -900], [1950, 650, -450], Wood),
    // --- the way out: a stair of ledges up the north wall ------------------
    Solid::cm([400, 200, 1750], [800, 300, 2000], Stone),
    Solid::cm([-100, 500, 1750], [300, 600, 2000], Stone),
    Solid::cm([-600, 800, 1750], [-200, 900, 2000], Stone),
    Solid::cm([-1100, 1100, 1750], [-700, 1200, 2000], Stone),
    Solid::cm([-1500, 1350, 1400], [-1100, 1450, 1800], Stone),
    // The return ledge: fifteen metres up the west wall, over the floor.
    Solid::cm([-2000, 1400, -500], [-1650, 1500, 500], Stone),
    // The gallery: thirteen metres up, out of the door.
    Solid::cm([1600, 1200, -400], [2000, 1300, 400], Stone),
    // --- the well ----------------------------------------------------------
    Solid::cm([2200, 0, 500], [3400, WELL_TOP, 700], Stone),
    Solid::cm([2200, 0, -700], [3400, WELL_TOP, -500], Stone),
    Solid::cm([3200, 0, -500], [3400, WELL_TOP, 500], Stone),
    // The cell's roof, broken off over its west half.
    Solid::cm([2650, 300, -500], [3200, 350, 500], Stone),
    // What fell through it: a block to climb on.
    Solid::cm([2300, 0, -250], [2550, 140, 150], Rock),
    // Up the well, a jump and a catch apart.
    Solid::cm([2850, 550, -500], [3200, 600, -250], Stone),
    Solid::cm([2200, 700, -300], [2500, 750, 300], Stone),
    Solid::cm([2600, 950, 250], [3000, 1000, 500], Stone),
    Solid::cm([2950, 1100, -200], [3200, 1150, 200], Stone),
    // The door's landing, at the gallery's height.
    Solid::cm([2200, 1250, -400], [2700, 1300, 400], Stone),
    // A lintel over the window, to say it is one from inside the well.
    Solid::cm([2200, 950, -250], [2350, 1000, 250], Stone),
];
