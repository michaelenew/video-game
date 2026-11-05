//! **The way to the Warden** (`--hunt warden`, `docs/design/opening.md` §5):
//! the opening, from the cell to the keep, **on a floating island**. Built to
//! the place standard (`docs/design/level-design.md`): walls of rock and
//! thick stone, a run-up through open ground before the fight, the keep seen
//! from the moment you step out, every height climbed with the moves every
//! class shares. Within what fixed point holds for an arena fought alone: a
//! diagonal under 170 m (`docs/design/fight-spaces.md`, F8).
//!
//! The island is round masses of rock ([`lobe`]) overlapping, their tops
//! thirty metres up ([`TOP`]), narrowing below: its edge curves, and past it
//! is the drop. **A fall off it is not a death** (`species::warden::frame`):
//! you are stood back on the nearest safe spot ([`SAFE`]), a little hurt.
//!
//! Along `+x`, at a run about twenty-five seconds end to end:
//!
//! | x (m) | What |
//! | --- | --- |
//! | -16 to 10 | **The cell** and **the cut**, between rock |
//! | 10 to 66 | **The yard**, on four lobes of the island: a curtain wall twelve metres tall across it, from the drop to the drop, crossed only at its gap to the north; a roofless chapel nine metres tall with its doors at different places, gone through or round, never over; an arch's posts, a fallen statue |
//! | 66 to 84 | **The causeway**, over the gap to the keep's island |
//! | 84 to 136 | **The keep**: eight walls six metres thick and twenty-nine tall round a floor forty across, a gate in the west wall; inside, a spiral of ledges up every face to the top, piers and columns; outside, **the climb**, pillars rising out of the drop and ledges up the south wall, optional, rewarded |

use super::hills::slab;
use super::{Arena, ArenaId, Bounds, HuntMarks, Mark, Material, Shape, Solid, Spawns};
use crate::fixed::Fx;
use crate::math::V3;

use Material::{Grass, Peat, Rock, Stone};

/// **The island's top**, in centimetres: thirty metres over the floor far
/// below, which is the drop.
pub const TOP: i32 = 3000;
/// The middle of the keep, in centimetres.
pub const KEEP: (i32, i32) = (11000, 0);
/// The keep's floor, as a radius from its middle: inside this is the fight.
pub const KEEP_FLOOR: i32 = 2000;
/// How tall the keep's walls stand over the island: the rampart's height.
pub const RAMPART: i32 = 2925;
/// **How far below the island's top a body has fallen off it**, in
/// centimetres.
pub const FALLEN: i32 = 1200;

/// **The curtain wall's line**, in centimetres: its west face, its east
/// face, and its gap's two edges along `z`. Everything else of it is wall,
/// twelve metres tall, from over the drop to over the drop. A fall is stood
/// back on the side of it the faller was on.
pub const CURTAIN: (i32, i32, (i32, i32)) = (2900, 3100, (1000, 1600));

/// **Where a fall off the island stands you**: the nearest of these, on the
/// island's top -- the cell, the yard either side of the curtain wall, the
/// causeway's two ends, and the foot of the climb, on the keep's island's
/// lip by its south-west wall.
pub const SAFE: [(i32, i32); 6] = [
    (-1000, 0),
    (2000, 0),
    (4000, 1000),
    (6400, 0),
    (8600, 0),
    (9450, -2400),
];

/// **The way to the keep**, for anything that walks it (the scripted
/// hunter, `tests/opening.rs`): out of the cut, north through the curtain
/// wall's gap, through the chapel, over the causeway, in at the gate.
pub const WAY: [(i32, i32); 8] = [
    (1000, 0),
    (2600, 1300),
    (3400, 1300),
    (4000, -900),
    (4800, -900),
    (5700, -1100),
    (6600, 0),
    (8800, 0),
];

pub static ARENA: Arena = Arena {
    id: ArenaId::WARDEN,
    name: "Warden",
    creature: Some(crate::species::SpeciesId::WARDEN),
    bounds: Bounds::cm((-1700, 14000), (-3500, 3000)),
    floor: Peat,
    regions: &[],
    solids: &SOLIDS,
    spawns: Spawns {
        // In the cell, side by side, facing the way out.
        versus: [
            on_top(Mark::cm(-1200, -250, (1000, -250))),
            on_top(Mark::cm(-1200, 250, (1000, 250))),
        ],
        hunt: Some(HuntMarks {
            hunters: [
                on_top(Mark::cm(-1200, -250, (1000, -250))),
                on_top(Mark::cm(-1200, 250, (1000, 250))),
            ],
            // In the middle of the keep, facing its gate.
            creatures: [
                on_top(Mark::cm(KEEP.0, KEEP.1, (9000, 0))),
                on_top(Mark::cm(KEEP.0, 1000, (9000, 1000))),
            ],
        }),
    },
    sites: &[],
    // No rim: past the island's edge is the drop.
    rim: None,
};

/// **A mark on the island**: a fight stands a body on what is under its
/// mark, so a mark at the floor's height would stand it on the floor far
/// below. Two metres over the island's top.
const fn on_top(m: Mark) -> Mark {
    Mark {
        at: V3::new(m.at.x, super::cm(TOP + 200), m.at.z),
        facing: m.facing,
    }
}

/// **A mass of the island**: round, `r` across (a radius) at its top, which
/// is the island's top, and a third of that at its foot twenty metres down.
const fn lobe(x: i32, z: i32, r: i32, material: Material) -> Solid {
    Solid {
        min: V3::new(super::cm(x - r), super::cm(TOP - 2000), super::cm(z - r)),
        max: V3::new(super::cm(x + r), super::cm(TOP), super::cm(z + r)),
        material,
        shape: Shape::Round {
            bottom: super::cm(r / 3),
            top: super::cm(r),
            hollow: Fx::ZERO,
        },
    }
}

/// A round pier on the keep's floor, `r` across (a radius), `top` over it,
/// at (x, z).
const fn pier(x: i32, z: i32, r: i32, top: i32) -> Solid {
    Solid {
        min: V3::new(super::cm(x - r), super::cm(TOP), super::cm(z - r)),
        max: V3::new(super::cm(x + r), super::cm(TOP + top), super::cm(z + r)),
        material: Stone,
        shape: Shape::Round {
            bottom: super::cm(r),
            top: super::cm(r),
            hollow: Fx::ZERO,
        },
    }
}

/// A pillar of the climb, rising out of the drop to `top` (centimetres,
/// over the floor far below).
const fn pillar(x: i32, z: i32, top: i32) -> Solid {
    Solid {
        min: V3::new(super::cm(x - 125), Fx::ZERO, super::cm(z - 125)),
        max: V3::new(super::cm(x + 125), super::cm(top), super::cm(z + 125)),
        material: Rock,
        shape: Shape::Round {
            bottom: super::cm(160),
            top: super::cm(125),
            hollow: Fx::ZERO,
        },
    }
}

/// One of the keep's four diagonal walls, centred on (x, z), lying along
/// (dx, dz): twenty metres long, six thick.
const fn diagonal(x: i32, z: i32, along: (i32, i32)) -> Solid {
    slab(x, z, (TOP, TOP + RAMPART), (1000, 300), along, Stone)
}

const SOLIDS: [Solid; 58] = [
    // --- the island: round masses of rock, their tops at TOP, narrowing
    // below. Overlapping, they make its curving edge. -------------------
    lobe(-500, 0, 1500, Rock),
    lobe(1700, 700, 1700, Grass),
    lobe(3300, -900, 2000, Grass),
    lobe(4700, 1300, 1600, Grass),
    lobe(5600, -700, 1500, Grass),
    lobe(11000, 0, 3000, Stone),
    // --- the cell and the cut: rock either side, and behind -------------
    Solid::cm([-1600, 3000, 600], [1000, 5000, 1300], Rock),
    Solid::cm([-1600, 3000, -1300], [1000, 5000, -600], Rock),
    Solid::cm([-1700, 3000, -600], [-1400, 5000, 600], Rock),
    // --- the curtain wall across the yard (`CURTAIN`): **impassable but at
    // its gap** to the moves every class shares -- twelve metres tall, past
    // a held jump and a ledge's catch, and run out over the drop at both
    // ends, a dozen metres past the edge and down the island's face below
    // where a fall is caught (`FALLEN`), so there is no way round an end,
    // and no drifting under it on the way down. Its gap, six metres, is at
    // the north end, against the island's edge, and frames the keep. A
    // class's own way over the top -- a stone launch, a dash -- is the other
    // way, and a skill's to take (L12).
    Solid::cm([2900, 1500, -4500], [3100, 4200, 1000], Stone),
    Solid::cm([2900, 1500, 1600], [3100, 4200, 4000], Stone),
    Solid::cm([2900, 3550, 1000], [3100, 4200, 1600], Stone),
    // --- the chapel: roofless, nine metres to its walls' tops, **through it
    // or round it, never over** with the shared moves. Its doors, west and
    // east, are at different places; through it the keep is hidden, round
    // it is the island's southern edge.
    Solid::cm([4200, 3000, -1600], [5400, 3900, -1500], Stone),
    Solid::cm([4200, 3000, -300], [5400, 3900, -200], Stone),
    Solid::cm([4200, 3000, -1600], [4300, 3900, -1100], Stone),
    Solid::cm([4200, 3000, -700], [4300, 3900, -200], Stone),
    Solid::cm([5300, 3000, -1600], [5400, 3900, -1300], Stone),
    Solid::cm([5300, 3000, -900], [5400, 3900, -200], Stone),
    // An arch's two posts and its fallen lintel; a plinth and its statue.
    Solid::cm([1800, 3000, -1100], [2000, 3500, -900], Stone),
    Solid::cm([1800, 3000, -500], [2000, 3500, -300], Stone),
    slab(2200, -700, (3000, 3060), (300, 50), (1, 2), Stone),
    Solid::cm([3900, 3000, 1300], [4200, 3150, 1600], Stone),
    slab(4500, 1900, (3000, 3150), (350, 70), (3, 1), Stone),
    // --- the causeway over the gap to the keep's island ------------------
    Solid::cm([6600, 2700, -500], [8400, 3000, 500], Stone),
    // --- the keep: eight thick walls, the west one parted by the gate ----
    Solid::cm([13000, 3000, -1000], [13600, 5925, 1000], Stone),
    Solid::cm([10000, 3000, 2000], [12000, 5925, 2600], Stone),
    Solid::cm([10000, 3000, -2600], [12000, 5925, -2000], Stone),
    Solid::cm([8400, 3000, 400], [9000, 5925, 1000], Stone),
    Solid::cm([8400, 3000, -1000], [9000, 5925, -400], Stone),
    diagonal(12626, 1626, (-1, 1)),
    diagonal(9374, 1626, (1, 1)),
    diagonal(9374, -1626, (-1, 1)),
    diagonal(12626, -1626, (1, 1)),
    // **The climb** (optional; the gate is the other way in): four pillars
    // rising out of the drop beside the keep's island, each a held jump
    // above the last -- miss one and you are off the island -- then ledges
    // up the south wall's outside face and round its corner to the top.
    pillar(9300, -3000, 3300),
    pillar(9750, -3300, 3550),
    pillar(10250, -3350, 3800),
    pillar(10700, -3150, 4050),
    Solid::cm([10100, 4210, -2850], [10500, 4310, -2600], Stone),
    Solid::cm([10500, 4470, -2850], [10900, 4570, -2600], Stone),
    Solid::cm([10900, 4730, -2850], [11300, 4830, -2600], Stone),
    Solid::cm([11300, 4990, -2850], [11700, 5090, -2600], Stone),
    Solid::cm([11700, 5250, -2850], [12060, 5350, -2600], Stone),
    slab(12306, -2547, (5510, 5610), (200, 125), (1, 1), Stone),
    slab(12589, -2264, (5770, 5870), (200, 125), (1, 1), Stone),
    // **The spiral inside**: a ledge on every wall's inner face, each a held
    // jump (3.25 m) above the one before it round the keep, meeting it at
    // the corner, from the floor to the top -- the way up for everybody,
    // with no tool of any class (the owner, 2026-11-03).
    slab(9745, -1255, (3225, 3325), (750, 225), (1, -1), Stone),
    slab(11000, -1775, (3550, 3650), (750, 225), (1, 0), Stone),
    slab(12255, -1255, (3875, 3975), (750, 225), (1, 1), Stone),
    slab(12775, 0, (4200, 4300), (750, 225), (0, 1), Stone),
    slab(12255, 1255, (4525, 4625), (750, 225), (-1, 1), Stone),
    slab(11000, 1775, (4850, 4950), (750, 225), (1, 0), Stone),
    slab(9745, 1255, (5175, 5275), (750, 225), (1, 1), Stone),
    slab(9225, 0, (5500, 5600), (750, 225), (0, 1), Stone),
    // Piers at 6 and 10 m, columns at 16 and 22 m.
    pier(11778, 778, 250, 600),
    pier(10222, 778, 250, 1000),
    pier(10222, -778, 250, 600),
    pier(11778, -778, 250, 1000),
    pier(11000, 1300, 200, 1600),
    pier(11000, -1300, 200, 2200),
];
