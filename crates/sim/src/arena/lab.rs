//! The lab: a dev arena for **measuring movement**, the way the range is one for
//! measuring fights. `cargo run -p sim --bin envelope` plays every class in it
//! (`crate::envelope`); nothing is hunted here and nothing is meant to look
//! good. `--arena lab` walks it.
//!
//! | What | Where | For |
//! | --- | --- | --- |
//! | The floor lane | z 0 to 60 m, x -140 to 50 m, nothing in it | Hops, long jumps, every tool on the flat |
//! | The runway | 20 m up, x -140 to 0 m, z -60 to -40 m | A real edge to run off: every jump's trajectory recorded through a 20 m fall, which is what the gap-and-rise table is read from |
//! | Eight ledges | x 60 to 90 m, 10 m wide, 2 to 20 m tall | Ledges at known heights, for the tools that go *up onto* something: a shadow sent there, a Grasp hauling her there, a shield leapt to |
//!
//! Centimetres, as every arena table is.

use super::{Arena, ArenaId, Bounds, Mark, Material, Solid, Spawns};

use Material::{Ground, Rock, Stone};

/// The runway, in centimetres: its far edge along x (where every jump off it
/// is taken from), its top -- high enough that a jump off its end is recorded
/// falling past every landing anybody asks about -- and its middle across z.
pub const RUNWAY: (i32, i32, i32) = (0, 2000, -5000);

/// Where along x a flat measurement starts from, and the floor lane's middle
/// across z.
pub const LANE: (i32, i32) = (-10000, 3000);

/// The ledges: their near and far faces along x, and each one's (middle z, height),
/// in centimetres.
pub const LEDGE_FACE: (i32, i32) = (6000, 9000);
pub const LEDGES: [(i32, i32); 8] = [
    (-4800, 200),
    (-3600, 400),
    (-2400, 600),
    (-1200, 800),
    (0, 1000),
    (1200, 1200),
    (2400, 1600),
    (3600, 2000),
];

const fn ledge(i: usize) -> Solid {
    let (z, h) = LEDGES[i];
    Solid::cm([LEDGE_FACE.0, 0, z - 500], [LEDGE_FACE.1, h, z + 500], Rock)
}

pub static ARENA: Arena = Arena {
    id: ArenaId::LAB,
    name: "Lab",
    creature: None,
    bounds: Bounds::cm((-15000, 15000), (-6500, 6500)),
    floor: Ground,
    regions: &[],
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [
            // On the runway, facing its edge.
            Mark::cm(-1000, RUNWAY.2, (RUNWAY.0, RUNWAY.2)),
            // The second fighter in the far corner of the floor, out of
            // everybody's way.
            Mark::cm(-14000, 6000, (0, 6000)),
        ],
        hunt: None,
    },
    sites: &[],
};

const SOLIDS: [Solid; 9] = [
    Solid::cm(
        [-14000, 0, RUNWAY.2 - 1000],
        [RUNWAY.0, RUNWAY.1, RUNWAY.2 + 1000],
        Stone,
    ),
    ledge(0),
    ledge(1),
    ledge(2),
    ledge(3),
    ledge(4),
    ledge(5),
    ledge(6),
    ledge(7),
];
