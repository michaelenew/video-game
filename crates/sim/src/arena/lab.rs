//! The lab: a dev arena for **measuring movement**, the way the range is one for
//! measuring fights. `cargo run -p sim --bin envelope` plays every class in it
//! (`crate::envelope`, `crate::search`); nothing is hunted here and nothing is
//! meant to look good. `--arena lab` walks it.
//!
//! | What | Where | For |
//! | --- | --- | --- |
//! | The runway | 40 m up, x -60 to 0 m, the whole width of the lanes | One long top with a real edge to leave from, as far back as anybody wants to run |
//! | Seventeen ledges | one per lane, their faces half a metre past the edge, 8 m below the runway to 45 m above it | A landing at a known rise, for the search: how far back a class can leave the runway and still end up on that top is the gap it crosses at that rise |
//! | The free lane | past the last ledge | Trajectories with nothing in the way |
//! | The floor | beyond the ledges' far ends, x 80 m on | Hops, runs and every tool on the flat |
//!
//! Centimetres, as every arena table is.

use super::{Arena, ArenaId, Bounds, Mark, Material, Solid, Spawns};

use Material::{Ground, Rock, Stone};

/// The runway, in centimetres: its far edge along x (where every jump off it
/// is taken from), its top, and its middle across z -- the free lane, with no
/// ledge in front of it.
pub const RUNWAY: (i32, i32, i32) = (0, 4000, 11000);

/// Where along x a flat measurement starts from, and its middle across z: on
/// the floor, past the ledges' far ends.
pub const LANE: (i32, i32) = (8000, 0);

/// The ledges: each lane's middle across z and its rise above the runway, in
/// centimetres, and where every ledge's face is along x.
pub const LANES: [(i32, i32); 17] = [
    (-10000, -800),
    (-8800, -400),
    (-7600, -200),
    (-6400, 0),
    (-5200, 100),
    (-4000, 200),
    (-2800, 300),
    (-1600, 400),
    (-400, 500),
    (800, 600),
    (2000, 800),
    (3200, 1000),
    (4400, 1200),
    (5600, 1600),
    (6800, 2000),
    (8000, 3000),
    (9200, 4500),
];

/// The near and far faces of every ledge along x.
pub const LEDGE_FACE: (i32, i32) = (50, 6000);

const fn ledge(i: usize) -> Solid {
    let (z, r) = LANES[i];
    let top = RUNWAY.1 + r;
    Solid::cm(
        [LEDGE_FACE.0, top - 300, z - 500],
        [LEDGE_FACE.1, top, z + 500],
        Rock,
    )
}

pub static ARENA: Arena = Arena {
    id: ArenaId::LAB,
    name: "Lab",
    creature: None,
    bounds: Bounds::cm((-8000, 20000), (-12000, 13000)),
    floor: Ground,
    regions: &[],
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [
            // On the runway, in the free lane, facing its edge.
            Mark::cm(-1000, RUNWAY.2, (RUNWAY.0, RUNWAY.2)),
            // The second fighter on the floor in the far corner, out of
            // everybody's way.
            Mark::cm(19000, 12000, (18000, 12000)),
        ],
        hunt: None,
    },
    sites: &[],
};

const SOLIDS: [Solid; 18] = [
    Solid::cm([-6000, 0, -10700], [RUNWAY.0, RUNWAY.1, 11700], Stone),
    ledge(0),
    ledge(1),
    ledge(2),
    ledge(3),
    ledge(4),
    ledge(5),
    ledge(6),
    ledge(7),
    ledge(8),
    ledge(9),
    ledge(10),
    ledge(11),
    ledge(12),
    ledge(13),
    ledge(14),
    ledge(15),
    ledge(16),
];
