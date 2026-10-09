//! **Hearth**: the walled town at the valley's mouth, and where the valley
//! starts (`docs/design/valley.md`).
//!
//! A market town inside a wall nine metres high, crenellated, with a tower at
//! each corner and two either side of each gate, and three ways out: **the
//! valley gate** east, to the Mouth and the climb; **the Ring's door** north,
//! the one place in the valley where the two of you may fight each other; and
//! **the west gate** to the Long Valley, under a waystone that lights when the
//! Mantis is beaten, because the Siegeshell walks that valley toward this
//! wall. A stair inside the east wall goes up onto the wall's walk, which is
//! the **lookout**.
//!
//! Nobody fights here. The main street runs gate to gate through a paved
//! square with a well and two trees; round it, houses of plaster and timber
//! under steep tiled roofs, an inn, the hall, the armoury, a barn, a row of
//! market stalls, barrels and crates, lamps along the street, and the bell
//! tower, which is the tallest thing in town.
//!
//! **A building is a table entry** ([`BUILDINGS`]), not a box somebody drew:
//! its footprint, its eaves and its ridge. Its collision is made from that --
//! its walls a box to the eaves, its roof a stack of boxes narrowing to the
//! ridge inside the roof's slopes, a chimney -- and the renderer draws the
//! house from the same entry (`game::town`), so what you see is what you
//! bump into. The same holds for the towers, the lamps, the stalls and the
//! props.

use super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns};
use crate::math::V3;
use crate::valley::{Gate, Kind, Place, Seam, Zone};

use Material::{Grass, Stone, Wood};

pub static ARENA: Arena = Arena {
    id: ArenaId::HEARTH,
    name: "Hearth",
    creature: None,
    bounds: Bounds::cm((-4500, 4500), (-3200, 4000)),
    floor: Grass,
    regions: &REGIONS,
    solids: &SOLIDS,
    spawns: Spawns {
        // The square, by the well, looking down the street to the valley
        // gate.
        versus: [
            Mark::cm(200, -150, (4000, -150)),
            Mark::cm(200, 150, (4000, 150)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: None,
};

const REGIONS: [Region; 4] = [
    // The main street, gate to gate, paved.
    Region {
        area: Area::rect_cm((-4500, 4500), (-350, 350)),
        material: Stone,
    },
    // The lane to the Ring's door.
    Region {
        area: Area::rect_cm((-350, 350), (350, 4000)),
        material: Stone,
    },
    // The square.
    Region {
        area: Area::disc_cm(0, 0, 1150),
        material: Stone,
    },
    // The bell tower's green.
    Region {
        area: Area::disc_cm(-2650, 950, 500),
        material: Grass,
    },
];

// ---------------------------------------------------------------------------
// What the town is made of, as tables
// ---------------------------------------------------------------------------

/// What a building is for: how it is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Use {
    /// Plaster and timber over a stone footing.
    House,
    /// Two storeys of it, with a sign.
    Inn,
    /// The hall: tall, stone below, timber above.
    Hall,
    /// Stone all the way up, a forge's chimney.
    Armoury,
    /// Boards, wide doors.
    Barn,
}

/// **A building**: its footprint (centimetres, x then z), its eaves and its
/// ridge above the floor, which way its ridge runs, what it is, and where
/// along the ridge its chimney stands, in thousandths from the low end.
#[derive(Clone, Copy, Debug)]
pub struct Building {
    pub lo: (i32, i32),
    pub hi: (i32, i32),
    pub eave: i32,
    pub ridge: i32,
    pub along_x: bool,
    pub kind: Use,
    pub chimney: i32,
}

const fn building(
    lo: (i32, i32),
    hi: (i32, i32),
    eave: i32,
    ridge: i32,
    along_x: bool,
    kind: Use,
    chimney: i32,
) -> Building {
    Building {
        lo,
        hi,
        eave,
        ridge,
        along_x,
        kind,
        chimney,
    }
}

/// **The buildings.** Set back from the street, none square to its
/// neighbour's line, so the street is a street and not a corridor.
pub const BUILDINGS: [Building; 16] = [
    // North of the street, east of the square.
    building((900, 1700), (2500, 2700), 650, 1050, true, Use::Inn, 800),
    building((700, 550), (1500, 1300), 400, 700, false, Use::House, 300),
    building((1900, 650), (2900, 1350), 450, 750, true, Use::House, 700),
    building((2950, 1750), (3400, 2750), 500, 800, false, Use::House, 200),
    // North of the street, west of the square.
    building(
        (-2200, 1400),
        (-800, 2750),
        700,
        1250,
        false,
        Use::Hall,
        850,
    ),
    building(
        (-3350, 1650),
        (-2550, 2350),
        400,
        700,
        true,
        Use::House,
        250,
    ),
    building(
        (-3350, 2450),
        (-2400, 2900),
        350,
        600,
        true,
        Use::House,
        600,
    ),
    building((-1900, 600), (-1000, 1100), 380, 650, true, Use::House, 200),
    // South of the street, west of the square.
    building(
        (-2950, -2550),
        (-1650, -1550),
        550,
        850,
        true,
        Use::Armoury,
        150,
    ),
    building(
        (-1300, -2750),
        (-500, -1950),
        450,
        800,
        false,
        Use::House,
        700,
    ),
    building(
        (-3350, -1250),
        (-2450, -650),
        400,
        700,
        true,
        Use::House,
        500,
    ),
    building(
        (-2050, -1250),
        (-1000, -650),
        350,
        650,
        true,
        Use::Barn,
        1000,
    ),
    // South of the street, east of the square.
    building(
        (800, -2750),
        (1700, -1950),
        500,
        850,
        false,
        Use::House,
        300,
    ),
    building(
        (2000, -2650),
        (2900, -1950),
        420,
        720,
        true,
        Use::House,
        650,
    ),
    building((1200, -1350), (2300, -750), 450, 760, true, Use::House, 250),
    building(
        (2500, -1250),
        (3000, -700),
        380,
        620,
        false,
        Use::House,
        500,
    ),
];

/// How many boxes a building's roof is stacked of.
pub const ROOF_STEPS: usize = 3;

/// A building's collision: walls, the roof's steps, the chimney.
pub const PARTS: usize = 2 + ROOF_STEPS;

/// **A tower**: its footprint, its height, and whether it has a roof (a
/// spire) or a crenellated top.
#[derive(Clone, Copy, Debug)]
pub struct Tower {
    pub lo: (i32, i32),
    pub hi: (i32, i32),
    pub high: i32,
    pub spire: bool,
}

const fn tower(lo: (i32, i32), hi: (i32, i32), high: i32, spire: bool) -> Tower {
    Tower {
        lo,
        hi,
        high,
        spire,
    }
}

/// The wall's towers, the gates' and the bell tower.
pub const TOWERS: [Tower; 9] = [
    // The corners.
    tower((3250, -3450), (3950, -2750), 1250, false),
    tower((3250, 2750), (3950, 3450), 1250, false),
    tower((-3950, -3450), (-3250, -2750), 1250, false),
    tower((-3950, 2750), (-3250, 3450), 1250, false),
    // Either side of the valley gate and the west gate.
    tower((3400, -1150), (4050, -500), 1150, false),
    tower((3400, 500), (4050, 1150), 1150, false),
    tower((-4050, -1150), (-3400, -500), 1150, false),
    tower((-4050, 500), (-3400, 1150), 1150, false),
    // The bell tower, on its green.
    tower((-2900, 700), (-2400, 1200), 1500, true),
];

/// A thing that stands about the town: what it is, and its box.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prop {
    Barrel,
    Crate,
    /// A market stall's counter: an awning is drawn over it.
    Stall,
    /// A lamp on a post.
    Lamp,
    /// The well.
    Well,
    /// A cart, its shafts down.
    Cart,
    /// A bale of hay.
    Hay,
}

/// A prop and its box, corners in centimetres.
pub type Placed = (Prop, [i32; 3], [i32; 3]);

const fn lamp(x: i32, z: i32) -> Placed {
    (Prop::Lamp, [x - 12, 0, z - 12], [x + 12, 340, z + 12])
}

const fn barrel(x: i32, z: i32) -> Placed {
    (Prop::Barrel, [x - 35, 0, z - 35], [x + 35, 95, z + 35])
}

const fn crate_(x: i32, z: i32, s: i32) -> Placed {
    (
        Prop::Crate,
        [x - s / 2, 0, z - s / 2],
        [x + s / 2, s, z + s / 2],
    )
}

/// **The props**: what makes a street lived in.
pub const PROPS: [Placed; 31] = [
    (Prop::Well, [-700, 0, -700], [-500, 100, -500]),
    // The market, by the street.
    (Prop::Stall, [1200, 0, -650], [1650, 100, -430]),
    (Prop::Stall, [1850, 0, -650], [2300, 100, -430]),
    (Prop::Stall, [2500, 0, -650], [2950, 100, -430]),
    crate_(1720, -560, 70),
    crate_(1740, -490, 50),
    barrel(2380, -560),
    barrel(3050, -560),
    // Lamps along the street.
    lamp(-3000, 420),
    lamp(-1800, -420),
    lamp(-600, 420),
    lamp(1300, 420),
    lamp(2700, 420),
    lamp(3100, -420),
    lamp(420, 2000),
    lamp(-420, 1000),
    // By the inn.
    barrel(2620, 1800),
    barrel(2700, 1880),
    barrel(2610, 1950),
    crate_(2700, 2250, 80),
    // The armoury's yard.
    crate_(-2900, -1400, 90),
    crate_(-2780, -1380, 60),
    barrel(-1500, -1450),
    (Prop::Cart, [-2350, 0, -1450], [-1950, 120, -1290]),
    // The barn.
    (Prop::Hay, [-2150, 0, -560], [-1950, 90, -420]),
    (Prop::Hay, [-1900, 0, -560], [-1700, 90, -420]),
    (Prop::Hay, [-2020, 90, -540], [-1820, 180, -440]),
    // About the houses.
    barrel(1550, 650),
    crate_(-900, -1850, 80),
    barrel(3050, 2850),
    crate_(-2400, 2350, 70),
];

/// Trees: a trunk's middle, and how tall. Drawn as trees round a trunk box,
/// as the valley's are.
pub const TREES: [(i32, i32, i32); 6] = [
    (450, 900, 850),
    (-650, -1400, 800),
    (-2650, 1500, 950),
    (-3100, 1350, 750),
    (2400, -2900, 700),
    (-400, -2900, 800),
];

/// A wall's walk carries merlons along its outer edge, this far apart.
const MERLON_EVERY: i32 = 240;

/// The wall's height.
pub const WALL: i32 = 900;

/// The walls, the gates' passages and their arches, the stair: everything
/// but the tables above.
const WALLS: [Solid; 22] = [
    // The Long Valley's waystone: lit when the Mantis is beaten. First, so
    // the west gate's seam names it as box 0.
    Solid::cm([-3350, 0, 450], [-3250, 320, 550], Stone),
    // The east wall, either side of the valley gate.
    Solid::cm([3500, 0, -3200], [3700, WALL, -300], Stone),
    Solid::cm([3500, 0, 300], [3700, WALL, 3200], Stone),
    // The valley gate's passage, open at its far end onto the land, and its
    // arch.
    Solid::cm([3700, 0, -500], [4300, WALL, -300], Stone),
    Solid::cm([3700, 0, 300], [4300, WALL, 500], Stone),
    Solid::cm([3500, 520, -300], [4300, WALL, 300], Stone),
    // The west wall, either side of the Long Valley's gate.
    Solid::cm([-3700, 0, -3200], [-3500, WALL, -300], Stone),
    Solid::cm([-3700, 0, 300], [-3500, WALL, 3200], Stone),
    // The west gate's passage and arch.
    Solid::cm([-4300, 0, -500], [-3700, WALL, -300], Stone),
    Solid::cm([-4300, 0, 300], [-3700, WALL, 500], Stone),
    Solid::cm([-4300, 520, -300], [-3500, WALL, 300], Stone),
    // The north wall, either side of the Ring's door, and the door's arch.
    Solid::cm([-3700, 0, 3000], [-300, WALL, 3200], Stone),
    Solid::cm([300, 0, 3000], [3700, WALL, 3200], Stone),
    Solid::cm([-300, 520, 3000], [300, WALL, 3200], Stone),
    // The Ring's lane, open at its far end onto the path to the Ring.
    Solid::cm([-500, 0, 3200], [-300, 600, 3800], Stone),
    Solid::cm([300, 0, 3200], [500, 600, 3800], Stone),
    // The south wall.
    Solid::cm([-3700, 0, -3200], [3700, WALL, -3000], Stone),
    // The stair up the east wall to its walk: the lookout.
    Solid::cm([3200, 0, -2800], [3500, 150, -2500], Stone),
    Solid::cm([3200, 0, -2500], [3500, 300, -2200], Stone),
    Solid::cm([3200, 0, -2200], [3500, 450, -1900], Stone),
    Solid::cm([3200, 0, -1900], [3500, 600, -1600], Stone),
    Solid::cm([3200, 0, -1600], [3500, 750, -1300], Stone),
];

/// The wall's runs that carry merlons: from, to (centimetres along the run),
/// the run's line, and which axis it runs along. On the outer edge.
const RUNS: [(i32, i32, i32, bool); 6] = [
    // East and west, outer edges.
    (-2750, -1150, 3650, false),
    (1150, 2750, 3650, false),
    (-2750, -1150, -3650, false),
    (1150, 2750, -3650, false),
    // North and south.
    (-3250, 3250, -3150, true),
    (-3250, 3250, 3150, true),
];

/// How many merlons a run carries.
const fn merlons_on(run: (i32, i32, i32, bool)) -> usize {
    ((run.1 - run.0) / MERLON_EVERY) as usize
}

const fn count_merlons() -> usize {
    let mut n = 0;
    let mut k = 0;
    while k < RUNS.len() {
        n += merlons_on(RUNS[k]);
        k += 1;
    }
    // Three along each side of each crenellated tower.
    let mut t = 0;
    while t < TOWERS.len() {
        if !TOWERS[t].spire {
            n += 12;
        }
        t += 1;
    }
    n
}

const MERLONS: usize = count_merlons();

const COUNT: usize =
    WALLS.len() + BUILDINGS.len() * PARTS + TOWERS.len() + PROPS.len() + TREES.len() + MERLONS;

/// Where the buildings' boxes start among the solids: after the walls.
pub const BUILDINGS_FROM: usize = WALLS.len();

/// The roof's steps of a building: step `k` of [`ROOF_STEPS`], inside the
/// roof's two slopes, from the eaves up.
pub const fn roof_step(b: &Building, k: usize) -> Solid {
    let rise = b.ridge - b.eave;
    let f1 = (k as i32 + 1) * 1000 / (ROOF_STEPS as i32 + 1);
    let y0 = b.eave + rise * k as i32 / (ROOF_STEPS as i32 + 1);
    let y1 = b.eave + rise * f1 / 1000;
    let (x0, x1, z0, z1) = if b.along_x {
        let mid = (b.lo.1 + b.hi.1) / 2;
        let half = (b.hi.1 - b.lo.1) / 2 * (1000 - f1) / 1000;
        (b.lo.0, b.hi.0, mid - half, mid + half)
    } else {
        let mid = (b.lo.0 + b.hi.0) / 2;
        let half = (b.hi.0 - b.lo.0) / 2 * (1000 - f1) / 1000;
        (mid - half, mid + half, b.lo.1, b.hi.1)
    };
    Solid::cm([x0, y0, z0], [x1, y1, z1], Wood)
}

/// A building's chimney: a stack through the roof by its ridge, standing a
/// little above it.
pub const fn chimney(b: &Building) -> Solid {
    let (cx, cz) = if b.along_x {
        let mid = (b.lo.1 + b.hi.1) / 2;
        (
            b.lo.0 + 60 + (b.hi.0 - b.lo.0 - 120) * b.chimney / 1000,
            mid + 70,
        )
    } else {
        let mid = (b.lo.0 + b.hi.0) / 2;
        (
            mid + 70,
            b.lo.1 + 60 + (b.hi.1 - b.lo.1 - 120) * b.chimney / 1000,
        )
    };
    Solid::cm(
        [cx - 40, b.eave, cz - 40],
        [cx + 40, b.ridge + 90, cz + 40],
        Stone,
    )
}

const fn town() -> [Solid; COUNT] {
    let mut out = [Solid::new(V3::ZERO, V3::ZERO); COUNT];
    let mut n = 0;
    let mut k = 0;
    while k < WALLS.len() {
        out[n] = WALLS[k];
        n += 1;
        k += 1;
    }
    k = 0;
    while k < BUILDINGS.len() {
        let b = &BUILDINGS[k];
        let walls = match b.kind {
            Use::Armoury | Use::Hall => Stone,
            _ => Wood,
        };
        out[n] = Solid::cm([b.lo.0, 0, b.lo.1], [b.hi.0, b.eave, b.hi.1], walls);
        n += 1;
        let mut s = 0;
        while s < ROOF_STEPS {
            out[n] = roof_step(b, s);
            n += 1;
            s += 1;
        }
        out[n] = chimney(b);
        n += 1;
        k += 1;
    }
    k = 0;
    while k < PROPS.len() {
        let (what, lo, hi) = PROPS[k];
        let m = match what {
            Prop::Lamp | Prop::Well => Stone,
            Prop::Hay => Grass,
            _ => Wood,
        };
        out[n] = Solid::cm(lo, hi, m);
        n += 1;
        k += 1;
    }
    k = 0;
    while k < TREES.len() {
        let (x, z, h) = TREES[k];
        out[n] = Solid::cm([x - 30, 0, z - 30], [x + 30, h, z + 30], Wood);
        n += 1;
        k += 1;
    }
    k = 0;
    while k < TOWERS.len() {
        let t = &TOWERS[k];
        out[n] = Solid::cm([t.lo.0, 0, t.lo.1], [t.hi.0, t.high, t.hi.1], Stone);
        n += 1;
        k += 1;
    }
    // Merlons along the wall's walk, on its outer edge...
    k = 0;
    while k < RUNS.len() {
        let (from, _, line, along_x) = RUNS[k];
        let count = merlons_on(RUNS[k]);
        let mut m = 0;
        while m < count {
            let a = from + m as i32 * MERLON_EVERY + 40;
            let (x0, x1, z0, z1) = if along_x {
                (a, a + 80, line - 50, line + 50)
            } else {
                (line - 50, line + 50, a, a + 80)
            };
            out[n] = Solid::cm([x0, WALL, z0], [x1, WALL + 90, z1], Stone);
            n += 1;
            m += 1;
        }
        k += 1;
    }
    // ...and round the tops of the crenellated towers.
    k = 0;
    while k < TOWERS.len() {
        let t = &TOWERS[k];
        if !t.spire {
            let w = t.hi.0 - t.lo.0;
            let d = t.hi.1 - t.lo.1;
            let mut m = 0;
            while m < 3 {
                let ax = t.lo.0 + w * m / 3 + w / 12;
                let az = t.lo.1 + d * m / 3 + d / 12;
                let (sx, sz) = (w / 6, d / 6);
                out[n] = Solid::cm(
                    [ax, t.high, t.lo.1],
                    [ax + sx, t.high + 90, t.lo.1 + 50],
                    Stone,
                );
                out[n + 1] = Solid::cm(
                    [ax, t.high, t.hi.1 - 50],
                    [ax + sx, t.high + 90, t.hi.1],
                    Stone,
                );
                out[n + 2] = Solid::cm(
                    [t.lo.0, t.high, az],
                    [t.lo.0 + 50, t.high + 90, az + sz],
                    Stone,
                );
                out[n + 3] = Solid::cm(
                    [t.hi.0 - 50, t.high, az],
                    [t.hi.0, t.high + 90, az + sz],
                    Stone,
                );
                n += 4;
                m += 1;
            }
        }
        k += 1;
    }
    let _ = n;
    out
}

const SOLIDS: [Solid; COUNT] = town();

/// Where the props' boxes start among the solids.
pub const PROPS_FROM: usize = BUILDINGS_FROM + BUILDINGS.len() * PARTS;

/// Where the trees' trunks start.
pub const TREES_FROM: usize = PROPS_FROM + PROPS.len();

/// Where the towers' boxes start; the merlons follow them.
pub const TOWERS_FROM: usize = TREES_FROM + TREES.len();

/// **Is this box part of something the town draws whole** -- a building, a
/// prop, a tree -- rather than a box drawn by what it is made of? By index
/// into [`ARENA`]'s solids. The walls, the towers and the merlons are drawn
/// by their stone, as every wall is.
pub fn drawn_whole(index: usize) -> bool {
    (BUILDINGS_FROM..TOWERS_FROM).contains(&index)
}

/// How Hearth joins the rest of the valley.
pub static PLACE: Place = Place {
    arena: ArenaId::HEARTH,
    kind: Kind::Town,
    seams: &SEAMS,
    vines: &[],
    vents: &[],
};

const SEAMS: [Seam; 3] = [
    // The valley gate: the Mouth, and the climb.
    Seam {
        zone: Zone::cm([3550, -100, -300], [4300, 500, 300]),
        to: ArenaId::MOUTH,
        at: 0,
        marks: [
            Mark::cm(3100, -150, (2100, -150)),
            Mark::cm(3100, 150, (2100, 150)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the valley",
    },
    // The Ring's door.
    Seam {
        zone: Zone::cm([-300, -100, 3050], [300, 500, 3800]),
        to: ArenaId::RING,
        at: 0,
        marks: [
            Mark::cm(-150, 2600, (-150, 1600)),
            Mark::cm(150, 2600, (150, 1600)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Ring",
    },
    // The west gate: the Long Valley, where the Siegeshell walks.
    Seam {
        zone: Zone::cm([-4300, -100, -300], [-3550, 500, 300]),
        to: ArenaId::SIEGESHELL,
        at: 0,
        marks: [
            Mark::cm(-3100, -150, (-2100, -150)),
            Mark::cm(-3100, 150, (-2100, 150)),
        ],
        gate: crate::valley::tier::FIVE,
        waystone: Some(0),
        says: "the Long Valley",
    },
];
