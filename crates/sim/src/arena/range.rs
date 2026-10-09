//! The range: a dev arena with one of everything an arena can have.
//!
//! Not a creature's arena. It exists so the machinery is exercised at the
//! extremes the bestiary asks for before any creature arena is built: the
//! Siegeshell's size (240 x 50 m), the Galewing's perch tower (12 m, climbed by
//! ledges a hop apart), the Veilstalker's 5 m trunks and 3 m bank, the
//! Sandmaw's rock islands in sand, and the Broodmother's cave under a 12 m
//! vault with the vault coming down to 8 m at the walls. `--arena range` or
//! `?arena=range` puts you in it; the tests in `tests/arena.rs` and the frame
//! budget in `tests/budget.rs` ask their questions of it.
//!
//! Laid out along x, west to east, so a fighter facing the way the proving
//! ground faces them walks through it in order:
//!
//! | x (m) | What |
//! | --- | --- |
//! | -120 to -80 | Plain ground. The marks. |
//! | -80 to -40 | Sand, three rock islands 0.5 m high, a boulder on one. |
//! | -40 to 0 | Snow, with ash round two old pits; five 5 m trunks; a stream. |
//! | 0 to 40 | The tower, 6 m square and 12 m tall, eight ledges up its sides; a 3 m bank along the north wall, with a 1.5 m step. |
//! | 40 to 80 | The cave: rock floor, a vault at 12 m that drops to 8 m at the walls, two pillars. |
//! | 80 to 120 | Grass and boulders; a gate and a road, the two sites a defended thing can stand at. |

use super::{Area, Arena, ArenaId, Bounds, HuntMarks, Mark, Material, Region, Solid, Spawns};
use crate::objective::Site;

use Material::{Ash, Grass, Rock, Sand, Snow, Stone, Water, Wood};

pub static ARENA: Arena = Arena {
    id: ArenaId::RANGE,
    name: "Range",
    creature: None,
    bounds: Bounds::cm((-12000, 12000), (-2500, 2500)),
    floor: Material::Ground,
    regions: &REGIONS,
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [
            Mark::cm(-10400, 0, (-9600, 0)),
            Mark::cm(-9600, 0, (-10400, 0)),
        ],
        // Twenty metres apart on the plain ground at the west end, the
        // creature facing the hunters.
        hunt: Some(HuntMarks {
            hunters: [
                Mark::cm(-10500, -200, (-8500, -200)),
                Mark::cm(-10500, 200, (-8500, 200)),
            ],
            creatures: [
                Mark::cm(-8500, -600, (-10500, -600)),
                Mark::cm(-8500, 600, (-10500, 600)),
            ],
        }),
    },
    sites: &SITES,
    rim: None,
};

/// Two places for a defended thing, at the grass end: a gate across a gap
/// near the east wall, and a road from the cave mouth to it. Nothing stands
/// at either unless the fight's species declares an objective for it --
/// `tests/objectives.rs` puts the dev pack's there.
const SITES: [Site; 2] = [
    Site {
        name: "gate",
        route: &[(11600, 1500)],
        size: (100, 300, 400),
        material: Wood,
    },
    Site {
        name: "road",
        route: &[(8400, -1800), (10000, -1800), (11400, -400)],
        size: (100, 120, 75),
        material: Wood,
    },
];

const REGIONS: [Region; 7] = [
    Region {
        area: Area::rect_cm((-8000, -4000), (-2500, 2500)),
        material: Sand,
    },
    Region {
        area: Area::rect_cm((-4000, 0), (-2500, 2500)),
        material: Snow,
    },
    Region {
        area: Area::disc_cm(-3000, -1200, 500),
        material: Ash,
    },
    Region {
        area: Area::disc_cm(-1200, 800, 400),
        material: Ash,
    },
    // A stream three metres wide across the snow.
    Region {
        area: Area::rect_cm((-2200, -1900), (-2500, 2500)),
        material: Water,
    },
    Region {
        area: Area::rect_cm((4000, 8000), (-2500, 2500)),
        material: Rock,
    },
    Region {
        area: Area::rect_cm((8000, 12000), (-2500, 2500)),
        material: Grass,
    },
];

const SOLIDS: [Solid; 34] = [
    // The rim: 1.5 m walls at the two ends, 3 m cliffs along the long sides.
    Solid::cm([-12100, 0, -2600], [-12000, 150, 2600], Stone),
    Solid::cm([12000, 0, -2600], [12100, 150, 2600], Stone),
    Solid::cm([-12100, 0, -2600], [12100, 300, -2500], Rock),
    Solid::cm([-12100, 0, 2500], [12100, 300, 2600], Rock),
    // Sand: three rock islands 5 m across and half a metre high, in a
    // triangle, and a boulder on the first.
    Solid::cm([-7250, 0, -950], [-6750, 50, -450], Rock),
    Solid::cm([-6050, 0, 450], [-5550, 50, 950], Rock),
    Solid::cm([-4850, 0, -950], [-4350, 50, -450], Rock),
    Solid::cm([-7100, 50, -800], [-6900, 250, -600], Rock),
    // Snow: five dead trunks, 5 m tall with flat tops.
    Solid::cm([-3640, 0, -840], [-3560, 500, -760], Wood),
    Solid::cm([-2840, 0, 1160], [-2760, 500, 1240], Wood),
    Solid::cm([-1640, 0, -1640], [-1560, 500, -1560], Wood),
    Solid::cm([-940, 0, 360], [-860, 500, 440], Wood),
    Solid::cm([-540, 0, -1040], [-460, 500, -960], Wood),
    // The tower: 6 m square, 12 m to its flat top...
    Solid::cm([1700, 0, -300], [2300, 1200, 300], Stone),
    // ...climbed by eight ledges, each 1.5 m above the last, spiralling round
    // its four sides twice: a hop for everybody.
    Solid::cm([1550, 0, -300], [1700, 150, 300], Stone),
    Solid::cm([1700, 0, -450], [2300, 300, -300], Stone),
    Solid::cm([2300, 0, -300], [2450, 450, 300], Stone),
    Solid::cm([1700, 0, 300], [2300, 600, 450], Stone),
    Solid::cm([1400, 0, -300], [1550, 750, 300], Stone),
    Solid::cm([1700, 0, -600], [2300, 900, -450], Stone),
    Solid::cm([2450, 0, -300], [2600, 1050, 300], Stone),
    Solid::cm([1700, 0, 450], [2300, 1200, 600], Stone),
    // A bank along the north wall: a 1.5 m step and then 3 m, 4 m deep, with a
    // walkable top.
    Solid::cm([500, 0, 2100], [3500, 150, 2200], Rock),
    Solid::cm([500, 0, 2200], [3500, 300, 2500], Rock),
    // The cave. A vault at 12 m over its whole floor...
    Solid::cm([4000, 1200, -2500], [8000, 1400, 2500], Rock),
    // ...that comes down to 8 m along both walls and at both mouths' lintels.
    Solid::cm([4000, 800, -2500], [8000, 1200, -1900], Rock),
    Solid::cm([4000, 800, 1900], [8000, 1200, 2500], Rock),
    Solid::cm([4000, 1000, -1900], [4200, 1200, 1900], Rock),
    Solid::cm([7800, 1000, -1900], [8000, 1200, 1900], Rock),
    // Two pillars, two metres thick, floor to vault.
    Solid::cm([5400, 0, -800], [5600, 1200, -600], Rock),
    Solid::cm([6400, 0, 600], [6600, 1200, 800], Rock),
    // Grass: boulders 2.5 m across and 2 m tall, standable.
    Solid::cm([8875, 0, -1125], [9125, 200, -875], Rock),
    Solid::cm([9875, 0, 875], [10125, 200, 1125], Rock),
    Solid::cm([10875, 0, -225], [11125, 200, 25], Rock),
];
