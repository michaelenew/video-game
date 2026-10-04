//! **The jump courses** (`docs/design/courses.md`): arenas with no
//! creature in them, where the challenge is the movement system itself --
//! world.md §2's trails, built first as measuring instruments.
//!
//! The look is the Hallelujah Mountains' climb to the banshee rookery:
//! **islands of rock hanging in open air** at staggered heights, a deep fall
//! under every gap -- sixty metres and more -- to a dark floor (the **pit**: below it you are stood back
//! on your last checkpoint, `crate::course`), stepping-stone chains of small
//! islands, and the finish on the last island, a nest -- the Galewing's rookery, where the bird would live. Every
//! island but the first hangs: its bottom is above the floor, so it is a
//! ceiling to whatever is under it as well as a floor to whoever is on it. The
//! first is **the foot**, a spire standing on the floor, because a round
//! starts on the ground under a mark.
//!
//! | Course | Tier | What it is |
//! | --- | --- | --- |
//! | The Stair (`stair`) | easy | An easy ascent: six islands, each a little higher, every gap well inside a plain jump for every class. |
//! | The Causeway (`causeway`) | easy | An easy crossing, read horizontally and drifting down: level gaps, two stepping stones and a long bridge. |
//! | The Spiral (`spiral`) | hard | Hard (a guess): a climb round a great pillar on ledges stepping up its four faces, one and a quarter turns, the ledges smaller on the second turn, to a nest on its crown. |
//! | The Falls (`falls`) | hard | Hard (a guess): a stair of stones up onto a twenty-metre arch, a run along it and a leap off its end down onto a big landing, then a waterfall of small stones stepping down left and right to a pool. |
//! | The Slalom (`slalom`) | hard | Hard (a guess): stepping stones weaving between tall pillars, then the one roof in the set -- a cave mouth over three islands -- and out into the light. |
//! | The Fork (`fork`) | hard | Hard (a guess): a hub with two ways on -- a high road up stacked ledges and along the tops, a low road of stepping stones -- that meet again; a committed leap down onto a big landing; a long runway to the nest. |
//! | The Spire (`spire`) | edge | Barely possible (a guess), the Elementalist's: two hops to the launch, and the rookery's spire 32 m above it, its face half a metre out. |
//! | The Gulf (`gulf`) | edge | Barely possible (a guess): an expert line in the open -- a runway, then eight-, seven- and six-metre leaps onto ever smaller stones, and a last nine-metre leap down to the nest. |
//!
//! **Authored by hand** (round four), as places to play rather than to
//! measure: a spiral round a pillar, an arch and a waterfall of stones, a
//! slalom between pillars with one cave mouth, a fork with a high road and a
//! low road. Path islands are grass, stepping stones sand, checkpoints snow,
//! scenery grey rock. Every tier is the author's guess, waiting on play.
//! The easy two and the Spire are older, built from lists of hops; the rest
//! from small builders in the generator.

use super::{Arena, ArenaId, Bounds, Mark, Material, Solid, Spawns};
use crate::class::Class;
use crate::course::{Course, Step, Tier};

use Material::{Grass, Peat, Rock, Sand, Snow, Wood};

/// Every course, in the order `N` steps through them: by tier.
pub static COURSES: [&Course; 8] = [
    &STAIR_COURSE,
    &CAUSEWAY_COURSE,
    &SPIRAL_COURSE,
    &FALLS_COURSE,
    &SLALOM_COURSE,
    &FORK_COURSE,
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
// The Spiral: hard
// ---------------------------------------------------------------------------

/// Hard (a guess): a climb round a great pillar on ledges stepping up its four faces, one and a quarter turns, the ledges smaller on the second turn, to a nest on its crown.
pub static SPIRAL: Arena = Arena {
    id: ArenaId::CLIMB_SPIRAL,
    name: "Spiral",
    creature: None,
    bounds: Bounds::cm((-2300, 4100), (-2800, 2800)),
    floor: Peat,
    regions: &[],
    solids: &SPIRAL_SOLIDS,
    spawns: SPAWNS,
    sites: &[],
};

const SPIRAL_SOLIDS: [Solid; 23] = [
    // the foot: a spire standing on the floor, the start
    Solid::cm([-300, 0, -300], [300, 6800, 300], Rock),
    // ledge 1 round the pillar
    Solid::cm([575, 6760, -570], [725, 6880, -330], Grass),
    // ledge 2 round the pillar
    Solid::cm([575, 6835, -170], [725, 6955, 70], Grass),
    // ledge 3 round the pillar
    Solid::cm([575, 6910, 230], [725, 7030, 470], Grass),
    // ledge 4 round the pillar
    Solid::cm([730, 6985, 575], [970, 7105, 725], Grass),
    // ledge 5 round the pillar
    Solid::cm([1130, 7060, 575], [1370, 7180, 725], Snow),
    // ledge 6 round the pillar
    Solid::cm([1530, 7135, 575], [1770, 7255, 725], Grass),
    // ledge 7 round the pillar
    Solid::cm([1875, 7210, 330], [2025, 7330, 570], Grass),
    // ledge 8 round the pillar
    Solid::cm([1875, 7285, -70], [2025, 7405, 170], Grass),
    // ledge 9 round the pillar
    Solid::cm([1875, 7360, -470], [2025, 7480, -230], Snow),
    // ledge 10 round the pillar
    Solid::cm([1630, 7435, -725], [1870, 7555, -575], Grass),
    // ledge 11 round the pillar
    Solid::cm([1230, 7510, -725], [1470, 7630, -575], Grass),
    // ledge 12 round the pillar
    Solid::cm([830, 7585, -725], [1070, 7705, -575], Grass),
    // ledge 13 round the pillar
    Solid::cm([575, 7660, -520], [725, 7780, -380], Snow),
    // ledge 14 round the pillar
    Solid::cm([575, 7750, -120], [725, 7870, 20], Sand),
    // ledge 15 round the pillar
    Solid::cm([575, 7840, 280], [725, 7960, 420], Sand),
    // ledge 16 round the pillar
    Solid::cm([780, 7930, 575], [920, 8050, 725], Sand),
    // ledge 17 round the pillar
    Solid::cm([1180, 8020, 575], [1320, 8140, 725], Snow),
    // ledge 18 round the pillar
    Solid::cm([1580, 8110, 575], [1720, 8230, 725], Sand),
    // ledge 19 round the pillar
    Solid::cm([1875, 8200, 380], [2025, 8320, 520], Sand),
    // ledge 20 round the pillar
    Solid::cm([1875, 8290, -20], [2025, 8410, 120], Sand),
    // the great pillar: the climb goes round it
    Solid::cm([800, 0, -500], [1800, 8460, 500], Rock),
    // the nest on the crown
    Solid::cm([800, 8460, -500], [1800, 8560, 500], Wood),
];

static SPIRAL_COURSE: Course = Course {
    arena: ArenaId::CLIMB_SPIRAL,
    name: "The Spiral",
    tier: Tier::Hard,
    for_class: None,
    pit: 6280,
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
            gap: 0,
            rise: 80,
            ask: "ledge 1 round the pillar",
        },
        Step {
            solid: 2,
            check: false,
            gap: 0,
            rise: 75,
            ask: "ledge 2 round the pillar",
        },
        Step {
            solid: 3,
            check: false,
            gap: 0,
            rise: 75,
            ask: "ledge 3 round the pillar",
        },
        Step {
            solid: 4,
            check: false,
            gap: 0,
            rise: 75,
            ask: "ledge 4 round the pillar",
        },
        Step {
            solid: 5,
            check: true,
            gap: 0,
            rise: 75,
            ask: "ledge 5 round the pillar",
        },
        Step {
            solid: 6,
            check: false,
            gap: 0,
            rise: 75,
            ask: "ledge 6 round the pillar",
        },
        Step {
            solid: 7,
            check: false,
            gap: 0,
            rise: 75,
            ask: "ledge 7 round the pillar",
        },
        Step {
            solid: 8,
            check: false,
            gap: 0,
            rise: 75,
            ask: "ledge 8 round the pillar",
        },
        Step {
            solid: 9,
            check: true,
            gap: 0,
            rise: 75,
            ask: "ledge 9 round the pillar",
        },
        Step {
            solid: 10,
            check: false,
            gap: 0,
            rise: 75,
            ask: "ledge 10 round the pillar",
        },
        Step {
            solid: 11,
            check: false,
            gap: 0,
            rise: 75,
            ask: "ledge 11 round the pillar",
        },
        Step {
            solid: 12,
            check: false,
            gap: 0,
            rise: 75,
            ask: "ledge 12 round the pillar",
        },
        Step {
            solid: 13,
            check: true,
            gap: 0,
            rise: 75,
            ask: "ledge 13 round the pillar",
        },
        Step {
            solid: 14,
            check: false,
            gap: 0,
            rise: 90,
            ask: "ledge 14 round the pillar",
        },
        Step {
            solid: 15,
            check: false,
            gap: 0,
            rise: 90,
            ask: "ledge 15 round the pillar",
        },
        Step {
            solid: 16,
            check: false,
            gap: 0,
            rise: 90,
            ask: "ledge 16 round the pillar",
        },
        Step {
            solid: 17,
            check: true,
            gap: 0,
            rise: 90,
            ask: "ledge 17 round the pillar",
        },
        Step {
            solid: 18,
            check: false,
            gap: 0,
            rise: 90,
            ask: "ledge 18 round the pillar",
        },
        Step {
            solid: 19,
            check: false,
            gap: 0,
            rise: 90,
            ask: "ledge 19 round the pillar",
        },
        Step {
            solid: 20,
            check: false,
            gap: 0,
            rise: 90,
            ask: "ledge 20 round the pillar",
        },
        Step {
            solid: 22,
            check: true,
            gap: 0,
            rise: 150,
            ask: "the nest on the crown",
        },
    ],
};

// ---------------------------------------------------------------------------
// The Falls: hard
// ---------------------------------------------------------------------------

/// Hard (a guess): a stair of stones up onto a twenty-metre arch, a run along it and a leap off its end down onto a big landing, then a waterfall of small stones stepping down left and right to a pool.
pub static FALLS: Arena = Arena {
    id: ArenaId::CLIMB_FALLS,
    name: "Falls",
    creature: None,
    bounds: Bounds::cm((-2300, 13200), (-2500, 2500)),
    floor: Peat,
    regions: &[],
    solids: &FALLS_SOLIDS,
    spawns: SPAWNS,
    sites: &[],
};

const FALLS_SOLIDS: [Solid; 16] = [
    // the foot: a spire standing on the floor, the start
    Solid::cm([-300, 0, -300], [300, 7200, 300], Rock),
    // the first island
    Solid::cm([450, 6960, -250], [950, 7260, 250], Grass),
    // the stair to the arch, 1
    Solid::cm([1100, 7270, -140], [1400, 7360, 140], Sand),
    // the stair to the arch, 2
    Solid::cm([1520, 7380, -140], [1820, 7470, 140], Sand),
    // the stair to the arch, 3
    Solid::cm([1940, 7490, -140], [2240, 7580, 140], Sand),
    // the stair to the arch, 4
    Solid::cm([2360, 7600, -140], [2660, 7690, 140], Sand),
    // the arch: run it
    Solid::cm([2930, 7750, -150], [5130, 7900, 150], Snow),
    // the big landing
    Solid::cm([5730, 7250, -450], [6630, 7550, 450], Snow),
    // the falls, stone 1
    Solid::cm([6910, 7330, 170], [7090, 7420, 350], Sand),
    // the falls, stone 2
    Solid::cm([7360, 7200, -350], [7540, 7290, -170], Sand),
    // the falls, stone 3
    Solid::cm([7810, 7070, 170], [7990, 7160, 350], Sand),
    // the falls, stone 4
    Solid::cm([8260, 6940, -350], [8440, 7030, -170], Snow),
    // the falls, stone 5
    Solid::cm([8710, 6810, 170], [8890, 6900, 350], Sand),
    // the falls, stone 6
    Solid::cm([9160, 6680, -350], [9340, 6770, -170], Sand),
    // the pool at the foot of the falls
    Solid::cm([9610, 6340, -300], [10210, 6640, 300], Snow),
    // the nest
    Solid::cm([10710, 6340, -200], [11110, 6740, 200], Wood),
];

static FALLS_COURSE: Course = Course {
    arena: ArenaId::CLIMB_FALLS,
    name: "The Falls",
    tier: Tier::Hard,
    for_class: None,
    pit: 6040,
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
            gap: 0,
            rise: 60,
            ask: "the first island",
        },
        Step {
            solid: 2,
            check: false,
            gap: 0,
            rise: 100,
            ask: "the stair to the arch, 1",
        },
        Step {
            solid: 3,
            check: false,
            gap: 0,
            rise: 110,
            ask: "the stair to the arch, 2",
        },
        Step {
            solid: 4,
            check: false,
            gap: 0,
            rise: 110,
            ask: "the stair to the arch, 3",
        },
        Step {
            solid: 5,
            check: false,
            gap: 0,
            rise: 110,
            ask: "the stair to the arch, 4",
        },
        Step {
            solid: 6,
            check: true,
            gap: 0,
            rise: 210,
            ask: "the arch: run it",
        },
        Step {
            solid: 7,
            check: true,
            gap: 0,
            rise: -350,
            ask: "the big landing",
        },
        Step {
            solid: 8,
            check: false,
            gap: 0,
            rise: -130,
            ask: "the falls, stone 1",
        },
        Step {
            solid: 9,
            check: false,
            gap: 0,
            rise: -130,
            ask: "the falls, stone 2",
        },
        Step {
            solid: 10,
            check: false,
            gap: 0,
            rise: -130,
            ask: "the falls, stone 3",
        },
        Step {
            solid: 11,
            check: true,
            gap: 0,
            rise: -130,
            ask: "the falls, stone 4",
        },
        Step {
            solid: 12,
            check: false,
            gap: 0,
            rise: -130,
            ask: "the falls, stone 5",
        },
        Step {
            solid: 13,
            check: false,
            gap: 0,
            rise: -130,
            ask: "the falls, stone 6",
        },
        Step {
            solid: 14,
            check: true,
            gap: 0,
            rise: -130,
            ask: "the pool at the foot of the falls",
        },
        Step {
            solid: 15,
            check: true,
            gap: 0,
            rise: 100,
            ask: "the nest",
        },
    ],
};

// ---------------------------------------------------------------------------
// The Slalom: hard
// ---------------------------------------------------------------------------

/// Hard (a guess): stepping stones weaving between tall pillars, then the one roof in the set -- a cave mouth over three islands -- and out into the light.
pub static SLALOM: Arena = Arena {
    id: ArenaId::CLIMB_SLALOM,
    name: "Slalom",
    creature: None,
    bounds: Bounds::cm((-2300, 8700), (-2500, 2600)),
    floor: Peat,
    regions: &[],
    solids: &SLALOM_SOLIDS,
    spawns: SPAWNS,
    sites: &[],
};

const SLALOM_SOLIDS: [Solid; 22] = [
    // the foot: a spire standing on the floor, the start
    Solid::cm([-300, 0, -300], [300, 7000, 300], Rock),
    // slalom stone 1
    Solid::cm([600, 6910, 210], [820, 7000, 430], Sand),
    // a slalom pillar
    Solid::cm([630, 0, -400], [790, 7900, -240], Rock),
    // slalom stone 2
    Solid::cm([980, 6950, -430], [1200, 7040, -210], Sand),
    // a slalom pillar
    Solid::cm([1010, 0, 240], [1170, 7900, 400], Rock),
    // slalom stone 3
    Solid::cm([1360, 6990, 210], [1580, 7080, 430], Sand),
    // a slalom pillar
    Solid::cm([1390, 0, -400], [1550, 7900, -240], Rock),
    // slalom stone 4
    Solid::cm([1740, 6910, -430], [1960, 7000, -210], Snow),
    // a slalom pillar
    Solid::cm([1770, 0, 240], [1930, 7900, 400], Rock),
    // slalom stone 5
    Solid::cm([2120, 6950, 210], [2340, 7040, 430], Sand),
    // a slalom pillar
    Solid::cm([2150, 0, -400], [2310, 7900, -240], Rock),
    // slalom stone 6
    Solid::cm([2500, 6990, -430], [2720, 7080, -210], Sand),
    // a slalom pillar
    Solid::cm([2530, 0, 240], [2690, 7900, 400], Rock),
    // slalom stone 7
    Solid::cm([2880, 6910, 210], [3100, 7000, 430], Sand),
    // a slalom pillar
    Solid::cm([2910, 0, -400], [3070, 7900, -240], Rock),
    // slalom stone 8
    Solid::cm([3260, 6950, -430], [3480, 7040, -210], Snow),
    // a slalom pillar
    Solid::cm([3290, 0, 240], [3450, 7900, 400], Rock),
    // under the cave mouth, 1
    Solid::cm([3840, 6700, -200], [4240, 7000, 200], Grass),
    // under the cave mouth, 2
    Solid::cm([4520, 6700, -200], [4920, 7000, 200], Grass),
    // under the cave mouth, 3
    Solid::cm([5200, 6700, -200], [5600, 7000, 200], Snow),
    // the cave mouth: a roof two metres over a head
    Solid::cm([3640, 7380, -500], [5800, 7780, 500], Rock),
    // the nest
    Solid::cm([6180, 6700, -250], [6680, 7100, 250], Wood),
];

static SLALOM_COURSE: Course = Course {
    arena: ArenaId::CLIMB_SLALOM,
    name: "The Slalom",
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
            solid: 1,
            check: false,
            gap: 0,
            rise: 0,
            ask: "slalom stone 1",
        },
        Step {
            solid: 3,
            check: false,
            gap: 0,
            rise: 40,
            ask: "slalom stone 2",
        },
        Step {
            solid: 5,
            check: false,
            gap: 0,
            rise: 40,
            ask: "slalom stone 3",
        },
        Step {
            solid: 7,
            check: true,
            gap: 0,
            rise: -80,
            ask: "slalom stone 4",
        },
        Step {
            solid: 9,
            check: false,
            gap: 0,
            rise: 40,
            ask: "slalom stone 5",
        },
        Step {
            solid: 11,
            check: false,
            gap: 0,
            rise: 40,
            ask: "slalom stone 6",
        },
        Step {
            solid: 13,
            check: false,
            gap: 0,
            rise: -80,
            ask: "slalom stone 7",
        },
        Step {
            solid: 15,
            check: true,
            gap: 0,
            rise: 40,
            ask: "slalom stone 8",
        },
        Step {
            solid: 17,
            check: false,
            gap: 0,
            rise: -40,
            ask: "under the cave mouth, 1",
        },
        Step {
            solid: 18,
            check: false,
            gap: 0,
            rise: 0,
            ask: "under the cave mouth, 2",
        },
        Step {
            solid: 19,
            check: true,
            gap: 0,
            rise: 0,
            ask: "under the cave mouth, 3",
        },
        Step {
            solid: 21,
            check: true,
            gap: 0,
            rise: 100,
            ask: "the nest",
        },
    ],
};

// ---------------------------------------------------------------------------
// The Fork: hard
// ---------------------------------------------------------------------------

/// Hard (a guess): a hub with two ways on -- a high road up stacked ledges and along the tops, a low road of stepping stones -- that meet again; a committed leap down onto a big landing; a long runway to the nest.
pub static FORK: Arena = Arena {
    id: ArenaId::CLIMB_FORK,
    name: "Fork",
    creature: None,
    bounds: Bounds::cm((-2300, 13900), (-2500, 2600)),
    floor: Peat,
    regions: &[],
    solids: &FORK_SOLIDS,
    spawns: SPAWNS,
    sites: &[],
};

const FORK_SOLIDS: [Solid; 20] = [
    // the foot: a spire standing on the floor, the start
    Solid::cm([-300, 0, -300], [300, 7500, 300], Rock),
    // the hub: two ways on
    Solid::cm([400, 7200, -400], [1200, 7500, 400], Snow),
    // the high road, ledge 1
    Solid::cm([1350, 7350, 290], [1650, 7650, 550], Grass),
    // the high road, ledge 2
    Solid::cm([1770, 7500, 290], [2070, 7800, 550], Grass),
    // the high road, ledge 3
    Solid::cm([2190, 7650, 290], [2490, 7950, 550], Grass),
    // the high road, ledge 4
    Solid::cm([2610, 7800, 290], [2910, 8100, 550], Grass),
    // the high road, top 1
    Solid::cm([3030, 7800, 290], [3530, 8100, 550], Grass),
    // the high road, top 2
    Solid::cm([3910, 7800, 290], [4410, 8100, 550], Grass),
    // the high road, top 3
    Solid::cm([4790, 7800, 290], [5290, 8100, 550], Grass),
    // the low road, stone 1
    Solid::cm([1489, 7310, -470], [1669, 7400, -290], Sand),
    // the low road, stone 2
    Solid::cm([2127, 7310, -330], [2307, 7400, -150], Sand),
    // the low road, stone 3
    Solid::cm([2765, 7310, -470], [2945, 7400, -290], Sand),
    // the low road, stone 4
    Solid::cm([3403, 7310, -330], [3583, 7400, -150], Sand),
    // the low road, stone 5
    Solid::cm([4041, 7310, -470], [4221, 7400, -290], Sand),
    // the low road, stone 6
    Solid::cm([4679, 7310, -330], [4859, 7400, -150], Sand),
    // the low road, stone 7
    Solid::cm([5317, 7310, -470], [5497, 7400, -290], Sand),
    // where the roads meet
    Solid::cm([5970, 7200, -400], [6770, 7500, 400], Snow),
    // the big landing
    Solid::cm([7470, 6900, -500], [8470, 7200, 500], Snow),
    // the runway
    Solid::cm([8770, 7150, -200], [10770, 7300, 200], Snow),
    // the nest
    Solid::cm([11370, 7000, -250], [11870, 7400, 250], Wood),
];

static FORK_COURSE: Course = Course {
    arena: ArenaId::CLIMB_FORK,
    name: "The Fork",
    tier: Tier::Hard,
    for_class: None,
    pit: 6600,
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
            check: true,
            gap: 0,
            rise: 0,
            ask: "the hub: two ways on",
        },
        Step {
            solid: 9,
            check: false,
            gap: 0,
            rise: -100,
            ask: "the low road, stone 1",
        },
        Step {
            solid: 10,
            check: false,
            gap: 0,
            rise: 0,
            ask: "the low road, stone 2",
        },
        Step {
            solid: 11,
            check: false,
            gap: 0,
            rise: 0,
            ask: "the low road, stone 3",
        },
        Step {
            solid: 12,
            check: false,
            gap: 0,
            rise: 0,
            ask: "the low road, stone 4",
        },
        Step {
            solid: 13,
            check: false,
            gap: 0,
            rise: 0,
            ask: "the low road, stone 5",
        },
        Step {
            solid: 14,
            check: false,
            gap: 0,
            rise: 0,
            ask: "the low road, stone 6",
        },
        Step {
            solid: 15,
            check: false,
            gap: 0,
            rise: 0,
            ask: "the low road, stone 7",
        },
        Step {
            solid: 16,
            check: true,
            gap: 0,
            rise: 100,
            ask: "where the roads meet",
        },
        Step {
            solid: 17,
            check: true,
            gap: 0,
            rise: -300,
            ask: "the big landing",
        },
        Step {
            solid: 18,
            check: true,
            gap: 0,
            rise: 100,
            ask: "the runway",
        },
        Step {
            solid: 19,
            check: true,
            gap: 0,
            rise: 100,
            ask: "the nest",
        },
    ],
};

// ---------------------------------------------------------------------------
// The Spire: edge
// ---------------------------------------------------------------------------

/// Barely possible (a guess), the Elementalist's: two hops to the launch, and the rookery's spire 32 m above it, its face half a metre out.
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
// The Gulf: edge
// ---------------------------------------------------------------------------

/// Barely possible (a guess): an expert line in the open -- a runway, then eight-, seven- and six-metre leaps onto ever smaller stones, and a last nine-metre leap down to the nest.
pub static GULF: Arena = Arena {
    id: ArenaId::CLIMB_GULF,
    name: "Gulf",
    creature: None,
    bounds: Bounds::cm((-2300, 9200), (-2300, 2400)),
    floor: Peat,
    regions: &[],
    solids: &GULF_SOLIDS,
    spawns: SPAWNS,
    sites: &[],
};

const GULF_SOLIDS: [Solid; 7] = [
    // the foot: a spire standing on the floor, the start
    Solid::cm([-300, 0, -300], [300, 8500, 300], Rock),
    // the runway
    Solid::cm([300, 8350, -200], [2300, 8500, 200], Grass),
    // eight metres, level, onto three
    Solid::cm([3100, 8200, -150], [3400, 8500, 150], Snow),
    // seven metres and up 1.5, onto two
    Solid::cm([4100, 8560, -100], [4300, 8650, 100], Sand),
    // six metres onto a stone 1.2 m across
    Solid::cm([4900, 8560, -60], [5020, 8650, 60], Sand),
    // and another
    Solid::cm([5620, 8560, -60], [5740, 8650, 60], Snow),
    // nine metres and down two: the nest
    Solid::cm([6640, 8050, -250], [7140, 8450, 250], Wood),
];

static GULF_COURSE: Course = Course {
    arena: ArenaId::CLIMB_GULF,
    name: "The Gulf",
    tier: Tier::Edge,
    for_class: None,
    pit: 7850,
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
            gap: 0,
            rise: 0,
            ask: "the runway",
        },
        Step {
            solid: 2,
            check: true,
            gap: 0,
            rise: 0,
            ask: "eight metres, level, onto three",
        },
        Step {
            solid: 3,
            check: false,
            gap: 0,
            rise: 150,
            ask: "seven metres and up 1.5, onto two",
        },
        Step {
            solid: 4,
            check: false,
            gap: 0,
            rise: 0,
            ask: "six metres onto a stone 1.2 m across",
        },
        Step {
            solid: 5,
            check: true,
            gap: 0,
            rise: 0,
            ask: "and another",
        },
        Step {
            solid: 6,
            check: true,
            gap: 0,
            rise: -200,
            ask: "nine metres and down two: the nest",
        },
    ],
};
