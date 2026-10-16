//! **The bench**: a dev arena of single hops, one per lane, for measuring
//! how hard a kind of hop is for each class before a course is built out of
//! it (`cargo run --release -p sim --bin courses -- --bench`). Generated,
//! like the courses. Its first lane's island stands on the floor so the
//! marks have somewhere to stand; every other hangs.

use super::{Arena, ArenaId, Bounds, Mark, Material, Solid, Spawns};

use Material::{Grass, Peat, Rock};

/// Each lane: what it is, the solid she starts on and the solid she aims for.
pub const LANES: [(&str, usize, usize); 18] = [
    ("long roof 30 gap 700", 0, 1),
    ("long roof 30 gap 900", 3, 4),
    ("long roof 30 gap 1100", 6, 7),
    ("long roof 30 gap 1300", 9, 10),
    ("long roof 30 gap 1500", 12, 13),
    ("long roof 80 gap 700", 15, 16),
    ("long roof 80 gap 900", 18, 19),
    ("long roof 80 gap 1100", 21, 22),
    ("long roof 80 gap 1300", 24, 25),
    ("long roof 80 gap 1500", 27, 28),
    ("open level 1500", 30, 31),
    ("open level 2000", 32, 33),
    ("open level 2500", 34, 35),
    ("open level 3000", 36, 37),
    ("gully 900, 3 wide, 2 down", 38, 39),
    ("gully 1200, 3 wide, 2 down", 42, 43),
    ("gully 1500, 3 wide, 2 down", 46, 47),
    ("gully 1800, 3 wide, 2 down", 50, 51),
];

/// **A gully lane** (2026-10-16): a runway at 60 m, then `long` of drop
/// between two walls `GULLY_WIDE` apart -- from 10 m under the runway to 12 m
/// over it -- and a landing 2 m down at the far end. Too long to jump; the
/// way over is off the walls, side to side (`docs/design/valley.md` §3).
pub const GULLY_WIDE: i32 = 300;

const fn gully(z: i32, long: i32) -> [Solid; 4] {
    let h = GULLY_WIDE / 2;
    [
        Solid::cm([-2500, 5700, z - h], [0, 6000, z + h], Rock),
        Solid::cm([long, 5500, z - h], [long + 600, 5800, z + h], Grass),
        Solid::cm([-200, 5000, z - h - 100], [long, 7200, z - h], Rock),
        Solid::cm([-200, 5000, z + h], [long, 7200, z + h + 100], Rock),
    ]
}

const GULLIES: [[Solid; 4]; 4] = [
    gully(16000, 900),
    gully(20000, 1200),
    gully(24000, 1500),
    gully(28000, 1800),
];

pub static ARENA: Arena = Arena {
    id: ArenaId::BENCH,
    name: "Bench",
    creature: None,
    bounds: Bounds::cm((-3000, 6000), (-42000, 42000)),
    floor: Peat,
    regions: &[],
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [
            Mark::cm(-250, -40000, (0, -40000)),
            Mark::cm(-2500, 40000, (0, 40000)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: None,
};

const OPEN: [Solid; 38] = [
    Solid::cm([-500, 0, -40250], [0, 6000, -39750], Rock),
    Solid::cm([700, 5700, -40150], [1000, 6000, -39850], Grass),
    Solid::cm([-500, 6210, -40300], [700, 6310, -39700], Rock),
    Solid::cm([-500, 5700, -36250], [0, 6000, -35750], Rock),
    Solid::cm([900, 5700, -36150], [1200, 6000, -35850], Grass),
    Solid::cm([-500, 6210, -36300], [900, 6310, -35700], Rock),
    Solid::cm([-500, 5700, -32250], [0, 6000, -31750], Rock),
    Solid::cm([1100, 5700, -32150], [1400, 6000, -31850], Grass),
    Solid::cm([-500, 6210, -32300], [1100, 6310, -31700], Rock),
    Solid::cm([-500, 5700, -28250], [0, 6000, -27750], Rock),
    Solid::cm([1300, 5700, -28150], [1600, 6000, -27850], Grass),
    Solid::cm([-500, 6210, -28300], [1300, 6310, -27700], Rock),
    Solid::cm([-500, 5700, -24250], [0, 6000, -23750], Rock),
    Solid::cm([1500, 5700, -24150], [1800, 6000, -23850], Grass),
    Solid::cm([-500, 6210, -24300], [1500, 6310, -23700], Rock),
    Solid::cm([-500, 5700, -20250], [0, 6000, -19750], Rock),
    Solid::cm([700, 5700, -20150], [1000, 6000, -19850], Grass),
    Solid::cm([-500, 6260, -20300], [700, 6360, -19700], Rock),
    Solid::cm([-500, 5700, -16250], [0, 6000, -15750], Rock),
    Solid::cm([900, 5700, -16150], [1200, 6000, -15850], Grass),
    Solid::cm([-500, 6260, -16300], [900, 6360, -15700], Rock),
    Solid::cm([-500, 5700, -12250], [0, 6000, -11750], Rock),
    Solid::cm([1100, 5700, -12150], [1400, 6000, -11850], Grass),
    Solid::cm([-500, 6260, -12300], [1100, 6360, -11700], Rock),
    Solid::cm([-500, 5700, -8250], [0, 6000, -7750], Rock),
    Solid::cm([1300, 5700, -8150], [1600, 6000, -7850], Grass),
    Solid::cm([-500, 6260, -8300], [1300, 6360, -7700], Rock),
    Solid::cm([-500, 5700, -4250], [0, 6000, -3750], Rock),
    Solid::cm([1500, 5700, -4150], [1800, 6000, -3850], Grass),
    Solid::cm([-500, 6260, -4300], [1500, 6360, -3700], Rock),
    Solid::cm([-500, 5700, -250], [0, 6000, 250], Rock),
    Solid::cm([1500, 5700, -250], [2000, 6000, 250], Grass),
    Solid::cm([-500, 5700, 3750], [0, 6000, 4250], Rock),
    Solid::cm([2000, 5700, 3750], [2500, 6000, 4250], Grass),
    Solid::cm([-500, 5700, 7750], [0, 6000, 8250], Rock),
    Solid::cm([2500, 5700, 7750], [3000, 6000, 8250], Grass),
    Solid::cm([-500, 5700, 11750], [0, 6000, 12250], Rock),
    Solid::cm([3000, 5700, 11750], [3500, 6000, 12250], Grass),
];

const SOLIDS: [Solid; 38 + 16] = {
    let mut out = [OPEN[0]; 38 + 16];
    let mut i = 0;
    while i < 38 + 16 {
        out[i] = if i < 38 {
            OPEN[i]
        } else {
            GULLIES[(i - 38) / 4][(i - 38) % 4]
        };
        i += 1;
    }
    out
};
