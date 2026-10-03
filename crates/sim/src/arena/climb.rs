//! **The jump courses** (`docs/design/courses.md`): six arenas with no
//! creature in them, where the challenge is the movement system itself --
//! world.md §2's trails, built first as measuring instruments.
//!
//! The look is the Hallelujah Mountains' climb to the banshee rookery:
//! **islands of rock hanging in open air** at staggered heights, a deep fall
//! under every gap to a dark floor (the **pit**: below it you are stood back
//! on your last checkpoint, `crate::course`), stepping-stone chains of small
//! islands, a long arch, an overhang to go under, and the finish on the highest
//! island, a nest -- the Galewing's rookery, where the bird would live. Every
//! island but the first hangs: its bottom is above the floor, so it is a
//! ceiling to whatever is under it as well as a floor to whoever is on it. The
//! first is **the foot**, a spire standing on the floor, because a round
//! starts on the ground under a mark.
//!
//! | Course | Tier | What it is |
//! | --- | --- | --- |
//! | The Stair (`stair`) | easy | An easy ascent: six islands, each a little higher, every gap well inside a plain jump for every class. |
//! | The Causeway (`causeway`) | easy | An easy crossing, read horizontally and drifting down: level gaps, two stepping stones and a long bridge. |
//! | The Climb (`climb`) | hard | The hard ascent to the rookery: every hop near the edge of the shortest jumps' airdodge, stepping stones, a gap under an overhang, the arch. |
//! | The Drift (`drift`) | hard | The hard crossing: long gaps between islands at about one height, drifting down, every one past a plain jump for the shortest jumpers. |
//! | The Spire (`spire`) | edge | Barely possible, built against the Elementalist: two hops to the launch, and the rookery's spire 32 m above it, its face half a metre out. |
//! | The Gulf (`gulf`) | edge | Barely possible, built against the Reaver: gaps too wide to jump, crossed by a shadow on a stone a metre across and the dash jump off it, and a last island higher than any jump and nearer than the reach of the shadow. |
//!
//! **Generated from a list of hops** -- a direction, the gap from the last
//! island's edge, the rise from its top, the size -- so each route's numbers
//! are what was built, and are written into the route (`Step::gap`,
//! `Step::rise`) for the instrument to read. The hard courses were set against
//! `cargo run -p sim --bin envelope`: each hop just inside the airdodged jump
//! of the shortest jumpers (the Champion and the Blood mage), so that every
//! class finishes them and none has much to spare. Measured, per class and per
//! hop, by `cargo run -p sim --bin courses`.

use super::{Arena, ArenaId, Bounds, Mark, Material, Solid, Spawns};
use crate::class::Class;
use crate::course::{Course, Step, Tier};

use Material::{Ash, Grass, Rock, Wood};

/// Every course, in the order `Shift+J` steps through them: by tier.
pub static COURSES: [&Course; 6] = [
    &STAIR_COURSE,
    &CAUSEWAY_COURSE,
    &CLIMB_COURSE,
    &DRIFT_COURSE,
    &SPIRE_COURSE,
    &GULF_COURSE,
];

/// Both fighters start on the foot, side by side, facing out along `+x`.
const SPAWNS: Spawns = Spawns {
    versus: [
        Mark::cm(-100, -150, (300, -150)),
        Mark::cm(-100, 150, (300, 150)),
    ],
    hunt: None,
};

// ---------------------------------------------------------------------------
// The Stair: easy
// ---------------------------------------------------------------------------

/// An easy ascent: six islands, each a little higher, every gap well inside a plain jump for every class.
pub static STAIR: Arena = Arena {
    id: ArenaId::CLIMB_STAIR,
    name: "Stair",
    creature: None,
    bounds: Bounds::cm((-2300, 5800), (-2500, 3200)),
    floor: Ash,
    regions: &[],
    solids: &STAIR_SOLIDS,
    spawns: SPAWNS,
    sites: &[],
};

const STAIR_SOLIDS: [Solid; 7] = [
    // the foot: a spire standing on the floor, the start
    Solid::cm([-300, 0, -300], [300, 800, 300], Rock),
    // up 1 m
    Solid::cm([600, 600, -250], [1100, 900, 250], Grass),
    // up 1.5 m
    Solid::cm([600, 750, 600], [1100, 1050, 1100], Grass),
    // up 2 m
    Solid::cm([1400, 950, 600], [1900, 1250, 1100], Grass),
    // up 2.5 m
    Solid::cm([2150, 1200, 600], [2650, 1500, 1100], Grass),
    // level 4 m
    Solid::cm([2150, 1200, -300], [2650, 1500, 200], Grass),
    // up 2 m, the nest
    Solid::cm([2950, 1300, -450], [3750, 1700, 350], Wood),
];

static STAIR_COURSE: Course = Course {
    arena: ArenaId::CLIMB_STAIR,
    name: "The Stair",
    tier: Tier::Easy,
    for_class: None,
    pit: 200,
    route: &[
        Step {
            solid: 0,
            check: true,
            gap: 0,
            rise: 0,
            ask: "the start",
        },
        Step {
            solid: 1,
            check: false,
            gap: 300,
            rise: 100,
            ask: "up 1 m",
        },
        Step {
            solid: 2,
            check: false,
            gap: 350,
            rise: 150,
            ask: "up 1.5 m",
        },
        Step {
            solid: 3,
            check: true,
            gap: 300,
            rise: 200,
            ask: "up 2 m",
        },
        Step {
            solid: 4,
            check: false,
            gap: 250,
            rise: 250,
            ask: "up 2.5 m",
        },
        Step {
            solid: 5,
            check: false,
            gap: 400,
            rise: 0,
            ask: "level 4 m",
        },
        Step {
            solid: 6,
            check: true,
            gap: 300,
            rise: 200,
            ask: "up 2 m, the nest",
        },
    ],
};

// ---------------------------------------------------------------------------
// The Causeway: easy
// ---------------------------------------------------------------------------

/// An easy crossing, read horizontally and drifting down: level gaps, two stepping stones and a long bridge.
pub static CAUSEWAY: Arena = Arena {
    id: ArenaId::CLIMB_CAUSEWAY,
    name: "Causeway",
    creature: None,
    bounds: Bounds::cm((-2300, 10100), (-2400, 2500)),
    floor: Ash,
    regions: &[],
    solids: &CAUSEWAY_SOLIDS,
    spawns: SPAWNS,
    sites: &[],
};

const CAUSEWAY_SOLIDS: [Solid; 8] = [
    // the foot: a spire standing on the floor, the start
    Solid::cm([-300, 0, -300], [300, 2000, 300], Rock),
    // level 4.5 m
    Solid::cm([750, 1700, -200], [1250, 2000, 200], Grass),
    // down 1 m, 5 m
    Solid::cm([1750, 1600, -200], [2150, 1900, 200], Grass),
    // stepping stone
    Solid::cm([2550, 1820, -100], [2750, 1900, 100], Rock),
    // stepping stone
    Solid::cm([3150, 1820, -100], [3350, 1900, 100], Rock),
    // down 2 m, 5.5 m
    Solid::cm([3900, 1400, -250], [4400, 1700, 250], Grass),
    // the bridge
    Solid::cm([4700, 1580, -125], [6700, 1700, 125], Rock),
    // level 5 m, the far island
    Solid::cm([7200, 1300, -400], [8000, 1700, 400], Wood),
];

static CAUSEWAY_COURSE: Course = Course {
    arena: ArenaId::CLIMB_CAUSEWAY,
    name: "The Causeway",
    tier: Tier::Easy,
    for_class: None,
    pit: 1100,
    route: &[
        Step {
            solid: 0,
            check: true,
            gap: 0,
            rise: 0,
            ask: "the start",
        },
        Step {
            solid: 1,
            check: false,
            gap: 450,
            rise: 0,
            ask: "level 4.5 m",
        },
        Step {
            solid: 2,
            check: false,
            gap: 500,
            rise: -100,
            ask: "down 1 m, 5 m",
        },
        Step {
            solid: 3,
            check: false,
            gap: 400,
            rise: 0,
            ask: "stepping stone",
        },
        Step {
            solid: 4,
            check: true,
            gap: 400,
            rise: 0,
            ask: "stepping stone",
        },
        Step {
            solid: 5,
            check: false,
            gap: 550,
            rise: -200,
            ask: "down 2 m, 5.5 m",
        },
        Step {
            solid: 6,
            check: false,
            gap: 300,
            rise: 0,
            ask: "the bridge",
        },
        Step {
            solid: 7,
            check: true,
            gap: 500,
            rise: 0,
            ask: "level 5 m, the far island",
        },
    ],
};

// ---------------------------------------------------------------------------
// The Climb: hard
// ---------------------------------------------------------------------------

/// The hard ascent to the rookery: every hop near the edge of the shortest jumps' airdodge, stepping stones, a gap under an overhang, the arch.
pub static CLIMB: Arena = Arena {
    id: ArenaId::CLIMB_CLIMB,
    name: "Climb",
    creature: None,
    bounds: Bounds::cm((-2300, 10400), (-2300, 4300)),
    floor: Ash,
    regions: &[],
    solids: &CLIMB_SOLIDS,
    spawns: SPAWNS,
    sites: &[],
};

const CLIMB_SOLIDS: [Solid; 11] = [
    // the foot: a spire standing on the floor, the start
    Solid::cm([-300, 0, -300], [300, 1000, 300], Rock),
    // up 2 m, 6 m
    Solid::cm([900, 900, -250], [1400, 1200, 250], Grass),
    // stepping stone, up 1 m
    Solid::cm([1850, 1220, -75], [2000, 1300, 75], Rock),
    // stepping stone, up 1 m
    Solid::cm([2450, 1320, -75], [2600, 1400, 75], Rock),
    // stepping stone, up 1 m
    Solid::cm([3050, 1420, -75], [3200, 1500, 75], Rock),
    // up 4 m, 3.6 m
    Solid::cm([2875, 1600, 435], [3375, 1900, 935], Grass),
    // level 7 m
    Solid::cm([2875, 1600, 1635], [3375, 1900, 2135], Grass),
    // the overhang: a slab 3 m over the gap
    Solid::cm([3225, 2200, 1535], [3775, 2400, 2235], Rock),
    // 2.5 m under the overhang
    Solid::cm([3625, 1600, 1635], [4125, 1900, 2135], Grass),
    // down 2 m onto the arch
    Solid::cm([4625, 1580, 1760], [7025, 1700, 2010], Rock),
    // up 3 m, 5 m, the rookery
    Solid::cm([7525, 1600, 1485], [8325, 2000, 2285], Wood),
];

static CLIMB_COURSE: Course = Course {
    arena: ArenaId::CLIMB_CLIMB,
    name: "The Climb",
    tier: Tier::Hard,
    for_class: None,
    pit: 400,
    route: &[
        Step {
            solid: 0,
            check: true,
            gap: 0,
            rise: 0,
            ask: "the start",
        },
        Step {
            solid: 1,
            check: false,
            gap: 600,
            rise: 200,
            ask: "up 2 m, 6 m",
        },
        Step {
            solid: 2,
            check: false,
            gap: 450,
            rise: 100,
            ask: "stepping stone, up 1 m",
        },
        Step {
            solid: 3,
            check: false,
            gap: 450,
            rise: 100,
            ask: "stepping stone, up 1 m",
        },
        Step {
            solid: 4,
            check: false,
            gap: 450,
            rise: 100,
            ask: "stepping stone, up 1 m",
        },
        Step {
            solid: 5,
            check: true,
            gap: 360,
            rise: 400,
            ask: "up 4 m, 3.6 m",
        },
        Step {
            solid: 6,
            check: false,
            gap: 700,
            rise: 0,
            ask: "level 7 m",
        },
        Step {
            solid: 8,
            check: false,
            gap: 250,
            rise: 0,
            ask: "2.5 m under the overhang",
        },
        Step {
            solid: 9,
            check: true,
            gap: 500,
            rise: -200,
            ask: "down 2 m onto the arch",
        },
        Step {
            solid: 10,
            check: true,
            gap: 500,
            rise: 300,
            ask: "up 3 m, 5 m, the rookery",
        },
    ],
};

// ---------------------------------------------------------------------------
// The Drift: hard
// ---------------------------------------------------------------------------

/// The hard crossing: long gaps between islands at about one height, drifting down, every one past a plain jump for the shortest jumpers.
pub static DRIFT: Arena = Arena {
    id: ArenaId::CLIMB_DRIFT,
    name: "Drift",
    creature: None,
    bounds: Bounds::cm((-2300, 11000), (-2300, 3600)),
    floor: Ash,
    regions: &[],
    solids: &DRIFT_SOLIDS,
    spawns: SPAWNS,
    sites: &[],
};

const DRIFT_SOLIDS: [Solid; 10] = [
    // the foot: a spire standing on the floor, the start
    Solid::cm([-300, 0, -300], [300, 3000, 300], Rock),
    // level 7.2 m
    Solid::cm([1020, 2700, -250], [1520, 3000, 250], Grass),
    // down 2 m, 7.8 m
    Solid::cm([2300, 2500, -250], [2800, 2800, 250], Grass),
    // level 7.4 m
    Solid::cm([3540, 2500, -250], [4040, 2800, 250], Grass),
    // stepping stone
    Solid::cm([4590, 2720, -60], [4710, 2800, 60], Rock),
    // stepping stone
    Solid::cm([5260, 2720, -60], [5380, 2800, 60], Rock),
    // stepping stone
    Solid::cm([5930, 2720, -60], [6050, 2800, 60], Rock),
    // down 4 m, 8.5 m
    Solid::cm([6900, 2100, -250], [7400, 2400, 250], Grass),
    // up 2 m, 6 m
    Solid::cm([6900, 2300, 850], [7400, 2600, 1350], Grass),
    // down 1 m, 7 m, the far island
    Solid::cm([8100, 2100, 700], [8900, 2500, 1500], Wood),
];

static DRIFT_COURSE: Course = Course {
    arena: ArenaId::CLIMB_DRIFT,
    name: "The Drift",
    tier: Tier::Hard,
    for_class: None,
    pit: 1800,
    route: &[
        Step {
            solid: 0,
            check: true,
            gap: 0,
            rise: 0,
            ask: "the start",
        },
        Step {
            solid: 1,
            check: false,
            gap: 720,
            rise: 0,
            ask: "level 7.2 m",
        },
        Step {
            solid: 2,
            check: false,
            gap: 780,
            rise: -200,
            ask: "down 2 m, 7.8 m",
        },
        Step {
            solid: 3,
            check: true,
            gap: 740,
            rise: 0,
            ask: "level 7.4 m",
        },
        Step {
            solid: 4,
            check: false,
            gap: 550,
            rise: 0,
            ask: "stepping stone",
        },
        Step {
            solid: 5,
            check: false,
            gap: 550,
            rise: 0,
            ask: "stepping stone",
        },
        Step {
            solid: 6,
            check: false,
            gap: 550,
            rise: 0,
            ask: "stepping stone",
        },
        Step {
            solid: 7,
            check: true,
            gap: 850,
            rise: -400,
            ask: "down 4 m, 8.5 m",
        },
        Step {
            solid: 8,
            check: false,
            gap: 600,
            rise: 200,
            ask: "up 2 m, 6 m",
        },
        Step {
            solid: 9,
            check: true,
            gap: 700,
            rise: -100,
            ask: "down 1 m, 7 m, the far island",
        },
    ],
};

// ---------------------------------------------------------------------------
// The Spire: edge
// ---------------------------------------------------------------------------

/// Barely possible, built against the Elementalist: two hops to the launch, and the rookery's spire 32 m above it, its face half a metre out.
pub static SPIRE: Arena = Arena {
    id: ArenaId::CLIMB_SPIRE,
    name: "Spire",
    creature: None,
    bounds: Bounds::cm((-2300, 5300), (-2400, 2500)),
    floor: Ash,
    regions: &[],
    solids: &SPIRE_SOLIDS,
    spawns: SPAWNS,
    sites: &[],
};

const SPIRE_SOLIDS: [Solid; 4] = [
    // the foot: a spire standing on the floor, the start
    Solid::cm([-300, 0, -300], [300, 1000, 300], Rock),
    // up 1 m, 5 m
    Solid::cm([800, 800, -250], [1300, 1100, 250], Grass),
    // up 1 m, 5 m: the launch
    Solid::cm([1800, 900, -300], [2400, 1200, 300], Grass),
    // up 32 m, half a metre out: the spire
    Solid::cm([2450, 3900, -400], [3250, 4400, 400], Wood),
];

static SPIRE_COURSE: Course = Course {
    arena: ArenaId::CLIMB_SPIRE,
    name: "The Spire",
    tier: Tier::Edge,
    for_class: Some(Class::Elementalist),
    pit: 400,
    route: &[
        Step {
            solid: 0,
            check: true,
            gap: 0,
            rise: 0,
            ask: "the start",
        },
        Step {
            solid: 1,
            check: false,
            gap: 500,
            rise: 100,
            ask: "up 1 m, 5 m",
        },
        Step {
            solid: 2,
            check: true,
            gap: 500,
            rise: 100,
            ask: "up 1 m, 5 m: the launch",
        },
        Step {
            solid: 3,
            check: true,
            gap: 50,
            rise: 3200,
            ask: "up 32 m, half a metre out: the spire",
        },
    ],
};

// ---------------------------------------------------------------------------
// The Gulf: edge
// ---------------------------------------------------------------------------

/// Barely possible, built against the Reaver: gaps too wide to jump, crossed by a shadow on a stone a metre across and the dash jump off it, and a last island higher than any jump and nearer than the reach of the shadow.
pub static GULF: Arena = Arena {
    id: ArenaId::CLIMB_GULF,
    name: "Gulf",
    creature: None,
    bounds: Bounds::cm((-2300, 9500), (-2400, 2500)),
    floor: Ash,
    regions: &[],
    solids: &GULF_SOLIDS,
    spawns: SPAWNS,
    sites: &[],
};

const GULF_SOLIDS: [Solid; 7] = [
    // the foot: a spire standing on the floor, the start
    Solid::cm([-300, 0, -300], [300, 2500, 300], Rock),
    // level 6 m
    Solid::cm([900, 2200, -250], [1400, 2500, 250], Grass),
    // 7.5 m onto a stone a metre across
    Solid::cm([2150, 2370, -50], [2250, 2450, 50], Rock),
    // 9 m and up 1.5 m off the stone
    Solid::cm([3150, 2300, -250], [3650, 2600, 250], Grass),
    // down 3 m, 7.5 m, onto a stone
    Solid::cm([4400, 2220, -50], [4500, 2300, 50], Rock),
    // up 1 m off the stone, 8 m
    Solid::cm([5300, 2100, -250], [5800, 2400, 250], Grass),
    // up 5 m, 8.2 m: the far nest
    Solid::cm([6620, 2500, -400], [7420, 2900, 400], Wood),
];

static GULF_COURSE: Course = Course {
    arena: ArenaId::CLIMB_GULF,
    name: "The Gulf",
    tier: Tier::Edge,
    for_class: Some(Class::ShadowReaver),
    pit: 1700,
    route: &[
        Step {
            solid: 0,
            check: true,
            gap: 0,
            rise: 0,
            ask: "the start",
        },
        Step {
            solid: 1,
            check: false,
            gap: 600,
            rise: 0,
            ask: "level 6 m",
        },
        Step {
            solid: 2,
            check: false,
            gap: 750,
            rise: -50,
            ask: "7.5 m onto a stone a metre across",
        },
        Step {
            solid: 3,
            check: true,
            gap: 900,
            rise: 150,
            ask: "9 m and up 1.5 m off the stone",
        },
        Step {
            solid: 4,
            check: false,
            gap: 750,
            rise: -300,
            ask: "down 3 m, 7.5 m, onto a stone",
        },
        Step {
            solid: 5,
            check: true,
            gap: 800,
            rise: 100,
            ask: "up 1 m off the stone, 8 m",
        },
        Step {
            solid: 6,
            check: true,
            gap: 820,
            rise: 500,
            ask: "up 5 m, 8.2 m: the far nest",
        },
    ],
};
