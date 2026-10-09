//! The Last Valley: the Siegeshell's arena, at the end of the world's road
//! (`creatures/siegeshell.md` §11).
//!
//! **A cut between cliffs, fifty metres wide and three hundred long, ending in
//! a town wall.** The creature walks it west to east down the middle, its feet
//! twenty-eight metres apart in two worn ruts, and the walk is the clock: from
//! where it starts to the siege line -- its head twenty metres from the wall --
//! is 233 m, which at its walking pace is a little under five minutes.
//!
//! | What | Where | Why |
//! | --- | --- | --- |
//! | Cliffs | Both long sides, 14 m | The valley's edge: eleven metres of flank between a foot and the cliff |
//! | The ruts | 2 m wide, under each line of feet | Where it is going is drawn from the first frame |
//! | Boulders | Every thirty metres or so, 2.5 m across, 2 m tall | The Plough's cover, and the only solids on the floor. Never in a rut: a pad lands where a rut is |
//! | Standing stones | Every forty metres, both sides, against the cliffs | The clock, read off the landscape |
//! | The wall | Across the east end, 8 m | The P7 objective at the arena's one site |
//!
//! **Three hundred metres, not the document's 240**: the walk is what the
//! document's numbers are about (five minutes at 0.8 m/s is 240 m of walking),
//! and a forty-metre body that stops with its head twenty metres short of the
//! wall needs a valley longer than the walk by its own length and the gap.

use super::rim::Rim;
use super::{Area, Arena, ArenaId, Bounds, HuntMarks, Mark, Material, Region, Solid, Spawns};
use crate::objective::Site;
use crate::species::SpeciesId;

use Material::{Grass, Ground, Rock, Stone};

pub static ARENA: Arena = Arena {
    id: ArenaId::SIEGESHELL,
    name: "Last Valley",
    creature: Some(SpeciesId::SIEGESHELL),
    bounds: Bounds::cm((-15000, 15000), (-2500, 2500)),
    floor: Grass,
    regions: &REGIONS,
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [
            Mark::cm(-9600, 0, (-8800, 0)),
            Mark::cm(-8800, 0, (-9600, 0)),
        ],
        hunt: Some(HuntMarks {
            // Down the valley from it, between its ruts, facing it coming.
            hunters: [
                Mark::cm(-9000, -400, (-13000, -400)),
                Mark::cm(-9000, 400, (-13000, 400)),
            ],
            // Its middle near the west end, facing the wall. The second slot
            // is never used: there is one of it.
            creatures: [Mark::cm(-12800, 0, (0, 0)), Mark::cm(-12800, 0, (0, 0))],
        }),
    },
    sites: &SITES,
    rim: Some(Rim::new(
        (-15000, 15200),
        (-2500, 2500),
        [0, 0, 0, 1400],
        1600,
    )),
};

/// The town wall: across the whole valley at its east end, three metres
/// thick and eight tall.
const SITES: [Site; 1] = [Site {
    name: "wall",
    route: &[(14500, 0)],
    size: (150, 2500, 800),
    material: Stone,
}];

/// The ruts: two metres wide, under each line of feet, the whole walk.
const REGIONS: [Region; 3] = [
    Region {
        area: Area::rect_cm((-15000, 15000), (-1500, -1300)),
        material: Ground,
    },
    Region {
        area: Area::rect_cm((-15000, 15000), (1300, 1500)),
        material: Ground,
    },
    // Packed earth before the wall, where the siege is.
    Region {
        area: Area::rect_cm((10000, 14350), (-2500, 2500)),
        material: Ground,
    },
];

/// A boulder: 2.5 m across and 2 m tall, standable.
const fn boulder(x: i32, z: i32) -> Solid {
    Solid::cm([x - 125, 0, z - 125], [x + 125, 200, z + 125], Rock)
}

/// A standing stone: a metre square and four tall, against a cliff.
const fn stone(x: i32, side: i32) -> Solid {
    let z = side * 2300;
    Solid::cm([x - 50, 0, z - 50], [x + 50, 400, z + 50], Stone)
}

/// Where the boulders are, in centimetres: every thirty metres or so, on the
/// middle line or out on the flanks -- never in a rut.
pub const BOULDERS: [(i32, i32); 8] = [
    (-9000, 0),
    (-6000, -2000),
    (-3000, 600),
    (0, 2000),
    (3000, -600),
    (6000, 0),
    (9000, -2000),
    (11500, 600),
];

/// Where the standing stones are along the valley: every forty metres.
pub const STONES: [i32; 7] = [-12000, -8000, -4000, 0, 4000, 8000, 12000];

const SOLIDS: [Solid; 23] = [
    // Behind the wall, the town: nobody goes there.
    Solid::cm([14650, 0, -2700], [15200, 1400, 2700], Stone),
    boulder(BOULDERS[0].0, BOULDERS[0].1),
    boulder(BOULDERS[1].0, BOULDERS[1].1),
    boulder(BOULDERS[2].0, BOULDERS[2].1),
    boulder(BOULDERS[3].0, BOULDERS[3].1),
    boulder(BOULDERS[4].0, BOULDERS[4].1),
    boulder(BOULDERS[5].0, BOULDERS[5].1),
    boulder(BOULDERS[6].0, BOULDERS[6].1),
    boulder(BOULDERS[7].0, BOULDERS[7].1),
    stone(STONES[0], -1),
    stone(STONES[0], 1),
    stone(STONES[1], -1),
    stone(STONES[1], 1),
    stone(STONES[2], -1),
    stone(STONES[2], 1),
    stone(STONES[3], -1),
    stone(STONES[3], 1),
    stone(STONES[4], -1),
    stone(STONES[4], 1),
    stone(STONES[5], -1),
    stone(STONES[5], 1),
    stone(STONES[6], -1),
    stone(STONES[6], 1),
];
