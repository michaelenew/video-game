//! **The jump courses** (`docs/design/courses.md`): arenas with no
//! creature in them, where the challenge is the movement system itself --
//! world.md §2's trails, built first as measuring instruments.
//!
//! The look is the Hallelujah Mountains' climb to the banshee rookery:
//! **islands of rock hanging in open air** at staggered heights, a deep fall
//! under every gap -- sixty metres and more -- to a dark floor (the **pit**: below it you are stood back
//! on your last checkpoint, `crate::course`), stepping-stone chains of small
//! islands, tunnels of low roofs through the rock, and the finish on the last
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
//! | The Gallery (`gallery`) | hard | Hard: a tunnel of slabs through the rocks, zig-zagging, every hop under a roof a hand above the head onto a stone a metre and a half across. |
//! | The Narrows (`narrows`) | hard | Hard: ledges sixty centimetres wide under low roofs, turning at every one. |
//! | The Sill (`sill`) | hard | Hard: stones stepping up thirty centimetres at a time under roofs, turning, so no hop is the same jump twice. |
//! | The Spire (`spire`) | edge | Barely possible, built against the Elementalist: two hops to the launch, and the rookery's spire 32 m above it, its face half a metre out. |
//! | The Eyrie (`eyrie`) | edge | Barely possible, built against the Elementalist: the Spire's taller sister, the eyrie 38 m above the launch and a metre out from it. |
//! | The Gulf (`gulf`) | edge | Barely possible: two gaps of twelve metres under roofs a hand over the head, too low to jump far and too long to dash -- the dash jump, or the Rush. |
//!
//! **Generated from a list of hops** -- a direction, the gap from the last
//! island's edge, the rise from its top, the size -- so each route's numbers
//! are what was built, and are written into the route (`Step::gap`,
//! `Step::rise`) for the instrument to read. The hard courses (round three)
//! are built from **constraints rather than distance**: a roof a hand over the
//! head caps every jump, launch and hang alike, over the takeoff and the gap
//! and not the target (so the Grasp has nothing over it to haul to), onto small
//! stones or narrow ledges, turning at every hop. Each hop's gap was set by
//! measuring every class's most forgiving line on the bench
//! (`courses -- --bench`) and then on the course: `cargo run --release -p sim
//! --bin courses`.

use super::{Arena, ArenaId, Bounds, Mark, Material, Solid, Spawns};
use crate::class::Class;
use crate::course::{Course, Step, Tier};

use Material::{Grass, Peat, Rock, Wood};

/// Every course, in the order `N` steps through them: by tier.
pub static COURSES: [&Course; 8] = [
    &STAIR_COURSE,
    &CAUSEWAY_COURSE,
    &GALLERY_COURSE,
    &NARROWS_COURSE,
    &SILL_COURSE,
    &SPIRE_COURSE,
    &EYRIE_COURSE,
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
    floor: Peat,
    regions: &[],
    solids: &STAIR_SOLIDS,
    spawns: SPAWNS,
    sites: &[],
};

const STAIR_SOLIDS: [Solid; 7] = [
    // the foot: a spire standing on the floor, the start
    Solid::cm([-300, 0, -300], [300, 6800, 300], Rock),
    // up 1 m
    Solid::cm([600, 6600, -250], [1100, 6900, 250], Grass),
    // up 1.5 m
    Solid::cm([600, 6750, 600], [1100, 7050, 1100], Grass),
    // up 2 m
    Solid::cm([1400, 6950, 600], [1900, 7250, 1100], Grass),
    // up 2.5 m
    Solid::cm([2150, 7200, 600], [2650, 7500, 1100], Grass),
    // level 4 m
    Solid::cm([2150, 7200, -300], [2650, 7500, 200], Grass),
    // up 2 m, the nest
    Solid::cm([2950, 7300, -450], [3750, 7700, 350], Wood),
];

static STAIR_COURSE: Course = Course {
    arena: ArenaId::CLIMB_STAIR,
    name: "The Stair",
    tier: Tier::Easy,
    for_class: None,
    pit: 6200,
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
    floor: Peat,
    regions: &[],
    solids: &CAUSEWAY_SOLIDS,
    spawns: SPAWNS,
    sites: &[],
};

const CAUSEWAY_SOLIDS: [Solid; 8] = [
    // the foot: a spire standing on the floor, the start
    Solid::cm([-300, 0, -300], [300, 8000, 300], Rock),
    // level 4.5 m
    Solid::cm([750, 7700, -200], [1250, 8000, 200], Grass),
    // down 1 m, 5 m
    Solid::cm([1750, 7600, -200], [2150, 7900, 200], Grass),
    // stepping stone
    Solid::cm([2550, 7820, -100], [2750, 7900, 100], Rock),
    // stepping stone
    Solid::cm([3150, 7820, -100], [3350, 7900, 100], Rock),
    // down 2 m, 5.5 m
    Solid::cm([3900, 7400, -250], [4400, 7700, 250], Grass),
    // the bridge
    Solid::cm([4700, 7580, -125], [6700, 7700, 125], Rock),
    // level 5 m, the far island
    Solid::cm([7200, 7300, -400], [8000, 7700, 400], Wood),
];

static CAUSEWAY_COURSE: Course = Course {
    arena: ArenaId::CLIMB_CAUSEWAY,
    name: "The Causeway",
    tier: Tier::Easy,
    for_class: None,
    pit: 7100,
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
// The Gallery: hard
// ---------------------------------------------------------------------------

/// Hard: a tunnel of slabs through the rocks, zig-zagging, every hop under a roof a hand above the head onto a stone a metre and a half across.
pub static GALLERY: Arena = Arena {
    id: ArenaId::CLIMB_GALLERY,
    name: "Gallery",
    creature: None,
    bounds: Bounds::cm((-2300, 4200), (-2400, 2600)),
    floor: Peat,
    regions: &[],
    solids: &GALLERY_SOLIDS,
    spawns: SPAWNS,
    sites: &[],
};

const GALLERY_SOLIDS: [Solid; 15] = [
    // the foot: a spire standing on the floor, the start
    Solid::cm([-300, 0, -300], [300, 7000, 300], Rock),
    // the roof over the gap, 30 cm over a head
    Solid::cm([300, 7210, -360], [620, 7810, 360], Rock),
    // 3.2 m under a roof onto a stone
    Solid::cm([620, 6920, -75], [770, 7000, 75], Rock),
    // the roof over the gap, 25 cm over a head
    Solid::cm([560, 7205, 75], [830, 7805, 355], Rock),
    // 2.8 m, turning, under a roof
    Solid::cm([620, 6920, 355], [770, 7000, 505], Rock),
    // the roof over the gap, 30 cm over a head
    Solid::cm([770, 7240, 295], [1110, 7840, 565], Rock),
    // 3.4 m and a step up, under a roof
    Solid::cm([1110, 6950, 355], [1260, 7030, 505], Rock),
    // the roof over the gap, 30 cm over a head
    Solid::cm([1050, 7240, 55], [1320, 7840, 355], Rock),
    // 3 m, turning back
    Solid::cm([1110, 6950, -95], [1260, 7030, 55], Rock),
    // the roof over the gap, 30 cm over a head
    Solid::cm([1260, 7240, -155], [1580, 7840, 115], Rock),
    // 3.2 m under a roof
    Solid::cm([1580, 6950, -95], [1730, 7030, 55], Rock),
    // the roof over the gap, 25 cm over a head
    Solid::cm([1520, 7235, 55], [1790, 7835, 355], Rock),
    // 3 m, turning
    Solid::cm([1580, 6950, 355], [1730, 7030, 505], Rock),
    // the roof over the gap, 30 cm over a head
    Solid::cm([1730, 7240, 295], [2030, 7840, 565], Rock),
    // 3 m to the nest
    Solid::cm([2030, 6630, 355], [2180, 7030, 505], Wood),
];

static GALLERY_COURSE: Course = Course {
    arena: ArenaId::CLIMB_GALLERY,
    name: "The Gallery",
    tier: Tier::Hard,
    for_class: None,
    pit: 6400,
    route: &[
        Step {
            solid: 0,
            check: true,
            gap: 0,
            rise: 0,
            ask: "the start",
        },
        Step {
            solid: 2,
            check: false,
            gap: 320,
            rise: 0,
            ask: "3.2 m under a roof onto a stone",
        },
        Step {
            solid: 4,
            check: false,
            gap: 280,
            rise: 0,
            ask: "2.8 m, turning, under a roof",
        },
        Step {
            solid: 6,
            check: true,
            gap: 340,
            rise: 30,
            ask: "3.4 m and a step up, under a roof",
        },
        Step {
            solid: 8,
            check: false,
            gap: 300,
            rise: 0,
            ask: "3 m, turning back",
        },
        Step {
            solid: 10,
            check: false,
            gap: 320,
            rise: 0,
            ask: "3.2 m under a roof",
        },
        Step {
            solid: 12,
            check: true,
            gap: 300,
            rise: 0,
            ask: "3 m, turning",
        },
        Step {
            solid: 14,
            check: true,
            gap: 300,
            rise: 0,
            ask: "3 m to the nest",
        },
    ],
};

// ---------------------------------------------------------------------------
// The Narrows: hard
// ---------------------------------------------------------------------------

/// Hard: ledges sixty centimetres wide under low roofs, turning at every one.
pub static NARROWS: Arena = Arena {
    id: ArenaId::CLIMB_NARROWS,
    name: "Narrows",
    creature: None,
    bounds: Bounds::cm((-2300, 4200), (-2400, 2700)),
    floor: Peat,
    regions: &[],
    solids: &NARROWS_SOLIDS,
    spawns: SPAWNS,
    sites: &[],
};

const NARROWS_SOLIDS: [Solid; 15] = [
    // the foot: a spire standing on the floor, the start
    Solid::cm([-300, 0, -300], [300, 7500, 300], Rock),
    // the roof over the gap, 20 cm over a head
    Solid::cm([300, 7700, -360], [600, 8300, 360], Rock),
    // 3 m onto a ledge 60 cm wide, under a roof
    Solid::cm([600, 7400, -30], [900, 7500, 30], Rock),
    // the roof over the gap, 25 cm over a head
    Solid::cm([540, 7705, 30], [960, 8305, 280], Rock),
    // 2.5 m, turning, onto a narrow ledge
    Solid::cm([720, 7400, 280], [780, 7500, 580], Rock),
    // the roof over the gap, 25 cm over a head
    Solid::cm([780, 7705, 220], [1080, 8305, 640], Rock),
    // 3 m onto a narrow ledge
    Solid::cm([1080, 7400, 400], [1380, 7500, 460], Rock),
    // the roof over the gap, 20 cm over a head
    Solid::cm([1020, 7700, 100], [1440, 8300, 400], Rock),
    // 3 m, turning back
    Solid::cm([1200, 7400, -200], [1260, 7500, 100], Rock),
    // the roof over the gap, 25 cm over a head
    Solid::cm([1260, 7705, -260], [1580, 8305, 160], Rock),
    // 3.2 m onto a narrow ledge
    Solid::cm([1580, 7400, -80], [1880, 7500, -20], Rock),
    // the roof over the gap, 25 cm over a head
    Solid::cm([1520, 7705, -20], [1940, 8305, 280], Rock),
    // 3 m, turning
    Solid::cm([1700, 7400, 280], [1760, 7500, 580], Rock),
    // the roof over the gap, 25 cm over a head
    Solid::cm([1760, 7705, 220], [2020, 8305, 640], Rock),
    // 2.6 m to the nest
    Solid::cm([2020, 7100, 355], [2170, 7500, 505], Wood),
];

static NARROWS_COURSE: Course = Course {
    arena: ArenaId::CLIMB_NARROWS,
    name: "The Narrows",
    tier: Tier::Hard,
    for_class: None,
    pit: 6900,
    route: &[
        Step {
            solid: 0,
            check: true,
            gap: 0,
            rise: 0,
            ask: "the start",
        },
        Step {
            solid: 2,
            check: false,
            gap: 300,
            rise: 0,
            ask: "3 m onto a ledge 60 cm wide, under a roof",
        },
        Step {
            solid: 4,
            check: false,
            gap: 250,
            rise: 0,
            ask: "2.5 m, turning, onto a narrow ledge",
        },
        Step {
            solid: 6,
            check: true,
            gap: 300,
            rise: 0,
            ask: "3 m onto a narrow ledge",
        },
        Step {
            solid: 8,
            check: false,
            gap: 300,
            rise: 0,
            ask: "3 m, turning back",
        },
        Step {
            solid: 10,
            check: false,
            gap: 320,
            rise: 0,
            ask: "3.2 m onto a narrow ledge",
        },
        Step {
            solid: 12,
            check: true,
            gap: 300,
            rise: 0,
            ask: "3 m, turning",
        },
        Step {
            solid: 14,
            check: true,
            gap: 260,
            rise: 0,
            ask: "2.6 m to the nest",
        },
    ],
};

// ---------------------------------------------------------------------------
// The Sill: hard
// ---------------------------------------------------------------------------

/// Hard: stones stepping up thirty centimetres at a time under roofs, turning, so no hop is the same jump twice.
pub static SILL: Arena = Arena {
    id: ArenaId::CLIMB_SILL,
    name: "Sill",
    creature: None,
    bounds: Bounds::cm((-2300, 4300), (-2700, 2400)),
    floor: Peat,
    regions: &[],
    solids: &SILL_SOLIDS,
    spawns: SPAWNS,
    sites: &[],
};

const SILL_SOLIDS: [Solid; 15] = [
    // the foot: a spire standing on the floor, the start
    Solid::cm([-300, 0, -300], [300, 8000, 300], Rock),
    // the roof over the gap, 30 cm over a head
    Solid::cm([300, 8240, -360], [620, 8840, 360], Rock),
    // 3.2 m and up 30 cm, under a roof
    Solid::cm([620, 7950, -75], [770, 8030, 75], Rock),
    // the roof over the gap, 30 cm over a head
    Solid::cm([560, 8240, -375], [830, 8840, -75], Rock),
    // 3 m, turning
    Solid::cm([620, 7950, -525], [770, 8030, -375], Rock),
    // the roof over the gap, 30 cm over a head
    Solid::cm([770, 8270, -585], [1130, 8870, -315], Rock),
    // 3.6 m and up 30 cm
    Solid::cm([1130, 7980, -525], [1280, 8060, -375], Rock),
    // the roof over the gap, 25 cm over a head
    Solid::cm([1070, 8265, -375], [1340, 8865, -95], Rock),
    // 2.8 m, turning
    Solid::cm([1130, 7980, -95], [1280, 8060, 55], Rock),
    // the roof over the gap, 30 cm over a head
    Solid::cm([1280, 8300, -155], [1600, 8900, 115], Rock),
    // 3.2 m and up 30 cm
    Solid::cm([1600, 8010, -95], [1750, 8090, 55], Rock),
    // the roof over the gap, 30 cm over a head
    Solid::cm([1540, 8330, -435], [1810, 8930, -95], Rock),
    // 3.4 m and up, turning
    Solid::cm([1600, 8040, -585], [1750, 8120, -435], Rock),
    // the roof over the gap, 30 cm over a head
    Solid::cm([1750, 8330, -645], [2050, 8930, -375], Rock),
    // 3 m to the nest
    Solid::cm([2050, 7720, -585], [2200, 8120, -435], Wood),
];

static SILL_COURSE: Course = Course {
    arena: ArenaId::CLIMB_SILL,
    name: "The Sill",
    tier: Tier::Hard,
    for_class: None,
    pit: 7400,
    route: &[
        Step {
            solid: 0,
            check: true,
            gap: 0,
            rise: 0,
            ask: "the start",
        },
        Step {
            solid: 2,
            check: false,
            gap: 320,
            rise: 30,
            ask: "3.2 m and up 30 cm, under a roof",
        },
        Step {
            solid: 4,
            check: false,
            gap: 300,
            rise: 0,
            ask: "3 m, turning",
        },
        Step {
            solid: 6,
            check: true,
            gap: 360,
            rise: 30,
            ask: "3.6 m and up 30 cm",
        },
        Step {
            solid: 8,
            check: false,
            gap: 280,
            rise: 0,
            ask: "2.8 m, turning",
        },
        Step {
            solid: 10,
            check: false,
            gap: 320,
            rise: 30,
            ask: "3.2 m and up 30 cm",
        },
        Step {
            solid: 12,
            check: true,
            gap: 340,
            rise: 30,
            ask: "3.4 m and up, turning",
        },
        Step {
            solid: 14,
            check: true,
            gap: 300,
            rise: 0,
            ask: "3 m to the nest",
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
    floor: Peat,
    regions: &[],
    solids: &SPIRE_SOLIDS,
    spawns: SPAWNS,
    sites: &[],
};

const SPIRE_SOLIDS: [Solid; 4] = [
    // the foot: a spire standing on the floor, the start
    Solid::cm([-300, 0, -300], [300, 7000, 300], Rock),
    // up 1 m, 5 m
    Solid::cm([800, 6800, -250], [1300, 7100, 250], Grass),
    // up 1 m, 5 m: the launch
    Solid::cm([1800, 6900, -300], [2400, 7200, 300], Grass),
    // up 32 m, half a metre out: the spire
    Solid::cm([2450, 9900, -400], [3250, 10400, 400], Wood),
];

static SPIRE_COURSE: Course = Course {
    arena: ArenaId::CLIMB_SPIRE,
    name: "The Spire",
    tier: Tier::Edge,
    for_class: Some(Class::Elementalist),
    pit: 6400,
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
// The Eyrie: edge
// ---------------------------------------------------------------------------

/// Barely possible, built against the Elementalist: the Spire's taller sister, the eyrie 38 m above the launch and a metre out from it.
pub static EYRIE: Arena = Arena {
    id: ArenaId::CLIMB_EYRIE,
    name: "Eyrie",
    creature: None,
    bounds: Bounds::cm((-2300, 5200), (-2300, 2400)),
    floor: Peat,
    regions: &[],
    solids: &EYRIE_SOLIDS,
    spawns: SPAWNS,
    sites: &[],
};

const EYRIE_SOLIDS: [Solid; 4] = [
    // the foot: a spire standing on the floor, the start
    Solid::cm([-300, 0, -300], [300, 7000, 300], Rock),
    // up 1 m, 5 m
    Solid::cm([800, 6800, -250], [1300, 7100, 250], Grass),
    // up 1 m, 5 m: the launch
    Solid::cm([1800, 6900, -300], [2400, 7200, 300], Grass),
    // up 38 m, a metre out: the eyrie
    Solid::cm([2500, 10500, -300], [3100, 11000, 300], Wood),
];

static EYRIE_COURSE: Course = Course {
    arena: ArenaId::CLIMB_EYRIE,
    name: "The Eyrie",
    tier: Tier::Edge,
    for_class: Some(Class::Elementalist),
    pit: 6400,
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
            gap: 100,
            rise: 3800,
            ask: "up 38 m, a metre out: the eyrie",
        },
    ],
};

// ---------------------------------------------------------------------------
// The Gulf: edge
// ---------------------------------------------------------------------------

/// Barely possible: two gaps of twelve metres under roofs a hand over the head, too low to jump far and too long to dash -- the dash jump, or the Rush.
pub static GULF: Arena = Arena {
    id: ArenaId::CLIMB_GULF,
    name: "Gulf",
    creature: None,
    bounds: Bounds::cm((-2300, 6400), (-2400, 2400)),
    floor: Peat,
    regions: &[],
    solids: &GULF_SOLIDS,
    spawns: SPAWNS,
    sites: &[],
};

const GULF_SOLIDS: [Solid; 6] = [
    // the foot: a spire standing on the floor, the start
    Solid::cm([-300, 0, -300], [300, 8500, 300], Rock),
    // level 5 m
    Solid::cm([800, 8200, -250], [1300, 8500, 250], Grass),
    // the roof over the gap, 30 cm over a head
    Solid::cm([1300, 8710, -310], [2550, 9310, 310], Rock),
    // 12.5 m under a low roof
    Solid::cm([2550, 8200, -150], [2850, 8500, 150], Grass),
    // the roof over the gap, 30 cm over a head
    Solid::cm([2850, 8710, -210], [4050, 9310, 210], Rock),
    // 12 m under a low roof, to the nest
    Solid::cm([4050, 8100, -150], [4350, 8500, 150], Wood),
];

static GULF_COURSE: Course = Course {
    arena: ArenaId::CLIMB_GULF,
    name: "The Gulf",
    tier: Tier::Edge,
    for_class: Some(Class::ShadowReaver),
    pit: 7900,
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
            rise: 0,
            ask: "level 5 m",
        },
        Step {
            solid: 3,
            check: true,
            gap: 1250,
            rise: 0,
            ask: "12.5 m under a low roof",
        },
        Step {
            solid: 5,
            check: true,
            gap: 1200,
            rise: 0,
            ask: "12 m under a low roof, to the nest",
        },
    ],
};
