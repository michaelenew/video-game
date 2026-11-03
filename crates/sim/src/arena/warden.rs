//! **The way to the Warden** (`--hunt warden`, `docs/design/opening.md` §5):
//! the opening, from the cell to the keep. Built to the place standard
//! (`docs/design/level-design.md`): the walls that enclose are rock or thick
//! stone or the rim's land, a run-up through open ground before the fight,
//! the keep seen from the moment you step out. And within what fixed point
//! holds for an arena fought alone: 152 by 64 metres, a diagonal of 165
//! (`docs/design/fight-spaces.md`, F8).
//!
//! Along `+x`, at a run about twenty seconds end to end:
//!
//! | x (m) | What | Made of |
//! | --- | --- | --- |
//! | -16 to -6 | **The cell**, twelve metres wide, open to the sky | rock either side, the rim behind |
//! | -6 to 10 | **The cut**, twelve metres wide, out to daylight | the same rock, twenty metres tall |
//! | 10 to 64 | **The yard**: open ground, swells and a dip, a knoll to look out from, low ruins | relief ([`super::relief`]) and low stone |
//! | 63 to 84 | **The ravine**, three and a half metres deep and walkable in and out, and **the causeway** over it, ten wide | two ramps of the floor, one thick block |
//! | 84 to 136 | **The keep**: eight walls six metres thick and twenty-nine tall round a floor forty across, a gate in the west wall; inside, a spiral of ledges a held jump apart up every face to the top, four piers (6 and 10 m) and two columns (16 and 22 m); outside, **the climb**: four pillars and six ledges up its south face to the rampart, optional, rewarded | thick stone |
//!
//! Mountains rise on every side (the rim). Nothing here is a thin wall, and
//! every height in the keep is reached with the moves every class shares.

use super::hills::slab;
use super::relief::{Bump, Ramp};
use super::rim::Rim;
use super::{
    Area, Arena, ArenaId, Bounds, HuntMarks, Mark, Material, Region, Shape, Solid, Spawns,
};
use crate::fixed::Fx;
use crate::math::V3;

use Material::{Grass, Rock, Stone};

/// The middle of the keep, in centimetres.
pub const KEEP: (i32, i32) = (11000, 0);
/// The keep's floor, as a radius from its middle: inside this is the fight.
pub const KEEP_FLOOR: i32 = 2000;
/// How tall the keep's walls stand: the rampart's height.
pub const RAMPART: i32 = 2925;

pub static ARENA: Arena = Arena {
    id: ArenaId::WARDEN,
    name: "Warden",
    creature: Some(crate::species::SpeciesId::WARDEN),
    bounds: Bounds::cm((-1600, 13600), (-3200, 3200)),
    floor: Grass,
    regions: &REGIONS,
    solids: &SOLIDS,
    spawns: Spawns {
        // In the cell, side by side, facing the way out.
        versus: [
            Mark::cm(-1200, -250, (1000, -250)),
            Mark::cm(-1200, 250, (1000, 250)),
        ],
        hunt: Some(HuntMarks {
            hunters: [
                Mark::cm(-1200, -250, (1000, -250)),
                Mark::cm(-1200, 250, (1000, 250)),
            ],
            // In the middle of the keep, facing its gate.
            creatures: [
                Mark::cm(KEEP.0, KEEP.1, (9000, 0)),
                Mark::cm(KEEP.0, 1000, (9000, 1000)),
            ],
        }),
    },
    sites: &[],
    // Mountains on every side: a bank too steep to climb, rising from the
    // floor at the edge.
    rim: Some(Rim::new((-1600, 13600), (-3200, 3200), [0, 0, 0, 0], 2000)),
};

/// Bare rock in the cell and the cut, and the keep's floor flagged stone:
/// the yard is the only green.
const REGIONS: [Region; 2] = [
    Region {
        area: Area::rect_cm((-1600, 1000), (-600, 600)),
        material: Rock,
    },
    Region {
        area: Area::disc_cm(KEEP.0, KEEP.1, KEEP_FLOOR),
        material: Stone,
    },
];

/// **The yard's ground** (`super::relief`): a knoll by the south rim to look
/// out from, swells either side of the way across and a shallow dip on it.
/// Clear of the ruins: the floor may rise into a solid's foot, never fall
/// away under one.
pub const BUMPS: [Bump; 5] = [
    Bump::cm(4000, -2300, 1200, 500),
    Bump::cm(2500, 1900, 1400, 250),
    Bump::cm(5500, 2100, 1200, 300),
    Bump::cm(3500, 400, 900, -100),
    Bump::cm(1500, -2000, 1000, 200),
];

/// The ravine the causeway crosses: down three and a half metres over nine,
/// and up again, across the whole width.
pub const RAMPS: [Ramp; 2] = [Ramp::x(6400, 7300, -350), Ramp::x(7500, 8400, 350)];

/// A round pier, `r` across (a radius), `top` tall, at (x, z).
const fn pier(x: i32, z: i32, r: i32, top: i32) -> Solid {
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

/// One of the keep's four diagonal walls, centred on (x, z), lying along
/// (dx, dz): twenty metres long, six thick.
const fn diagonal(x: i32, z: i32, along: (i32, i32)) -> Solid {
    slab(x, z, (0, RAMPART), (1000, 300), along, Stone)
}

const SOLIDS: [Solid; 46] = [
    // --- the cell and the cut: rock either side, to the rim ---------------
    Solid::cm([-1600, 0, 600], [1000, 2000, 3200], Rock),
    Solid::cm([-1600, 0, -3200], [1000, 2000, -600], Rock),
    // --- the yard: low ruins, nothing the camera must keep out of --------
    // A broken enclosure.
    Solid::cm([2000, 0, 1200], [3200, 120, 1350], Stone),
    Solid::cm([2000, 0, 1200], [2150, 120, 2000], Stone),
    Solid::cm([3050, 0, 1200], [3200, 80, 1700], Stone),
    // An arch's two posts and its fallen lintel.
    Solid::cm([4500, 0, -1300], [4700, 500, -1100], Stone),
    Solid::cm([4500, 0, -700], [4700, 500, -500], Stone),
    slab(4900, -900, (0, 60), (300, 50), (1, 2), Stone),
    // A plinth, and the statue fallen off it.
    Solid::cm([5200, 0, 700], [5500, 150, 1000], Stone),
    slab(5800, 1200, (0, 150), (350, 70), (3, 1), Stone),
    // A low wall toward the ravine.
    Solid::cm([5800, 0, -2200], [6300, 100, -2050], Stone),
    // --- the causeway over the ravine ------------------------------------
    Solid::cm([6300, -450, -500], [8400, 0, 500], Stone),
    // --- the keep: eight thick walls, the west one parted by the gate ----
    // East.
    Solid::cm([13000, 0, -1000], [13600, RAMPART, 1000], Stone),
    // North and south.
    Solid::cm([10000, 0, 2000], [12000, RAMPART, 2600], Stone),
    Solid::cm([10000, 0, -2600], [12000, RAMPART, -2000], Stone),
    // West, either side of an eight-metre gate.
    Solid::cm([8400, 0, 400], [9000, RAMPART, 1000], Stone),
    Solid::cm([8400, 0, -1000], [9000, RAMPART, -400], Stone),
    // The diagonals.
    diagonal(12626, 1626, (-1, 1)),
    diagonal(9374, 1626, (1, 1)),
    diagonal(9374, -1626, (-1, 1)),
    diagonal(12626, -1626, (1, 1)),
    // **The climb to the rampart** (optional; the gate is the other way
    // in): four pillars along the south wall's foot, each a held jump above
    // the last and a jump across from it, then seven ledges stepping up
    // the wall's outside face and round the corner, 2.6 m apart, to the
    // top. Nothing overhangs anything: every ledge is climbed onto. A fall from
    // the low pillars costs nothing; from the ledges, what the fall rule
    // says. The reward is the Warden asleep below (`species::warden`).
    pier(8600, -2900, 125, 300),
    pier(9075, -2900, 125, 550),
    pier(9550, -2900, 125, 800),
    pier(10050, -2900, 125, 1050),
    Solid::cm([10100, 1210, -2850], [10500, 1310, -2600], Stone),
    Solid::cm([10500, 1470, -2850], [10900, 1570, -2600], Stone),
    Solid::cm([10900, 1730, -2850], [11300, 1830, -2600], Stone),
    Solid::cm([11300, 1990, -2850], [11700, 2090, -2600], Stone),
    Solid::cm([11700, 2250, -2850], [12060, 2350, -2600], Stone),
    // Round the corner, up the south-east wall's outside face.
    slab(12306, -2547, (2510, 2610), (200, 125), (1, 1), Stone),
    slab(12589, -2264, (2770, 2870), (200, 125), (1, 1), Stone),
    // **The spiral inside**: a ledge four and a half metres deep on every
    // wall's inner face, each a held jump (3.25 m) above the one before it
    // round the keep, meeting it at the corner, from the
    // south-west face to the one over the gate -- the way up for everybody,
    // with no tool of any class (the owner, 2026-11-03).
    slab(9745, -1255, (225, 325), (750, 225), (1, -1), Stone),
    slab(11000, -1775, (550, 650), (750, 225), (1, 0), Stone),
    slab(12255, -1255, (875, 975), (750, 225), (1, 1), Stone),
    slab(12775, 0, (1200, 1300), (750, 225), (0, 1), Stone),
    slab(12255, 1255, (1525, 1625), (750, 225), (-1, 1), Stone),
    slab(11000, 1775, (1850, 1950), (750, 225), (1, 0), Stone),
    slab(9745, 1255, (2175, 2275), (750, 225), (1, 1), Stone),
    slab(9225, 0, (2500, 2600), (750, 225), (0, 1), Stone),
    // Two columns between the piers: perches at 16 and 22 m, the other way
    // up the middle levels.
    pier(11000, 1300, 200, 1600),
    pier(11000, -1300, 200, 2200),
    // --- the piers inside, toward the walls: perches at 6 and 10 m --------
    pier(11778, 778, 250, 600),
    pier(10222, 778, 250, 1000),
    pier(10222, -778, 250, 600),
    pier(11778, -778, 250, 1000),
];
