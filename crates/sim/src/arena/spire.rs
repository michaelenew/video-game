//! **The Spire**: Hearth's tower (exploration 0019 §2a, built 2026-10-15).
//! The owner: "an extremely tall narrow tower with a very large inverted dome
//! at the top (like the DBZ Korin tower, but no rounded top) with some ruined
//! buildings atop it and roosting flying monsters on top of the dome".
//!
//! It stands on a plaza just outside the town's south wall -- the square is
//! too full of town for a shaft nine metres across -- on the Wood's road, so
//! the dome's rim hangs over the town's south roofs. A place of its own, so
//! the screen says where you are: **the Needle**, which is also what
//! `--arena needle` starts at (the jump course is already called the Spire).
//!
//! | Part | What it is |
//! | --- | --- |
//! | **The shaft** | Round, pale stone, nine metres across at its foot and under eight at its top, 315 m tall |
//! | **The vine** | Down its south face the whole height: anybody, at the vine's pace, about two minutes |
//! | **The corkscrew** | Stones jutting from the shaft a quarter turn and eight metres apart: the mechanic band |
//! | **The balconies** | Every forty metres, three boards round the shaft (never across the vine), a cairn on the north one every other time |
//! | **The lantern** | Four columns on the shaft's top carrying the dome, gaps between them to walk through |
//! | **The dome** | Inverted, 90 m across and 35 m deep, its point on the lantern: eight slabs of a sphere, every one a ring round **the chimney** up its stem |
//! | **The chimney** | Nine metres across, 38 m from the shaft's top to the dome's: ledges spiralling up its wall, and a vine |
//! | **The ruins** | On the dome's top, 353 m up: broken walls, column stumps, an arch, a fallen column, a cistern, a cairn by the chimney's mouth |
//!
//! The roost -- Galewings on the dome, fought there -- is a range's
//! (exploration 0019 §2e), not this table's.

use super::{Arena, ArenaId, Bounds, Mark, Material, Shape, Solid, Spawns};
use crate::fixed::Fx;
use crate::math::V3;
use crate::valley::land::{Point, at};
use crate::valley::{Gate, Kind, Place, Seam, Zone};

/// Where the shaft stands on the map, centimetres.
pub const AT: (i32, i32) = (1000, -6000);

pub static ARENA: Arena = Arena {
    id: ArenaId::SPIRE,
    name: "Needle",
    creature: None,
    bounds: Bounds::cm((-4400, 4400), (-14000, -3900)),
    floor: Material::Stone,
    regions: &[],
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [
            Mark::cm(2400, -6000, (1000, -6000)),
            Mark::cm(2400, -6400, (1000, -6000)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: None,
};

const SOLIDS: [Solid; 0] = [];

/// **The plaza** at the shaft's foot: a way short and wide, on the Wood's
/// road (`super::wood::WAY` runs through it).
pub const PLAZA: [Point; 2] = [at(500, -5800, 0, 1500), at(1500, -6200, 0, 1500)];

/// The shaft: its radius at the foot and at the top, and how tall.
pub const SHAFT: (i32, i32, i32) = (450, 380, 31500);
/// The dome: how far above the shaft's top its point is (the lantern), how
/// deep it is and how far across at its rim (a radius), and the chimney's
/// radius.
pub const LANTERN: i32 = 350;
pub const DEPTH: i32 = 3500;
pub const RIM: i32 = 4500;
pub const CHIMNEY: i32 = 450;
/// The dome's point and its top, above the plaza.
pub const DOME_FROM: i32 = SHAFT.2 + LANTERN;
pub const TOP: i32 = DOME_FROM + DEPTH;
const SLABS: i32 = 8;

/// The bearing the vines hang on (south), in thousandths of a turn, and how
/// far either side of it nothing else is put.
const VINE_BEARING: i32 = 750;
const VINE_CLEAR: i32 = 80;

/// The vine down the shaft's south face, from the plaza to the shaft's top.
const OUTSIDE: Zone = Zone::cm(
    [AT.0 - 120, -2000, AT.1 - SHAFT.0 - 100],
    [AT.0 + 120, SHAFT.2, AT.1],
);
/// The vine up the chimney's south wall, from the shaft's top to the dome's.
const INSIDE: Zone = Zone::cm(
    [AT.0 - 120, SHAFT.2, AT.1 - CHIMNEY],
    [AT.0 + 120, TOP, AT.1],
);

pub const VINES: [Zone; 2] = [OUTSIDE, INSIDE];

/// How the Spire joins the valley.
pub static PLACE: Place = Place {
    arena: ArenaId::SPIRE,
    kind: Kind::Reach,
    seams: &SEAMS,
    vines: &VINES,
    vents: &[],
};

const SEAMS: [Seam; 2] = [
    // Up the road to the Mouth, toward the town's gate.
    Seam {
        zone: Zone::cm([3900, -300, -5900], [4400, 700, -4900]),
        to: ArenaId::MOUTH,
        at: 3,
        marks: [
            Mark::cm(3500, -5500, (1000, -6000)),
            Mark::cm(3500, -5800, (1000, -6000)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Mouth",
    },
    // On down the road to the Wood.
    Seam {
        zone: Zone::cm([800, -300, -14000], [2000, 700, -13500]),
        to: ArenaId::WOOD,
        at: 0,
        marks: [
            Mark::cm(1200, -11800, (1000, -6000)),
            Mark::cm(1500, -11800, (1000, -6000)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Great Wood",
    },
];

// ---------------------------------------------------------------------------
// Built on the land
// ---------------------------------------------------------------------------

fn cm(v: i32) -> Fx {
    super::cm(v)
}

fn turns(milli: i32) -> Fx {
    Fx::ratio(milli.rem_euclid(1000), 1000)
}

/// Is a bearing clear of the vine's?
fn off_the_vine(bearing: i32) -> bool {
    let d = (bearing - VINE_BEARING).rem_euclid(1000);
    d > VINE_CLEAR && d < 1000 - VINE_CLEAR
}

/// The shaft's radius at a height above the plaza.
const fn shaft_at(y: i32) -> i32 {
    SHAFT.0 - (SHAFT.0 - SHAFT.1) * y / SHAFT.2
}

/// The dome's radius at a height above its point: a slice of a sphere
/// through the point and the rim.
fn dome_at(h: i32) -> i32 {
    let r = (RIM as i64 * RIM as i64 + DEPTH as i64 * DEPTH as i64) / (2 * DEPTH as i64);
    let k = r - h as i64;
    crate::math::isqrt_wide((r * r - k * k).max(0) as u128) as i32
}

/// A board from `inner` to `outer` out from the axis along a bearing, its top
/// at `top`, `wide` across, `thick` deep.
#[allow(clippy::too_many_arguments)]
fn board(
    base: Fx,
    bearing: i32,
    inner: i32,
    outer: i32,
    top: i32,
    wide: i32,
    thick: i32,
    m: Material,
) -> Solid {
    let d = V3::from_turns(turns(bearing));
    let mid = cm((inner + outer) / 2);
    Solid::turned(
        cm(AT.0).add(d.x.mul(mid)),
        cm(AT.1).add(d.z.mul(mid)),
        base.add(cm(top - thick)),
        base.add(cm(top)),
        cm((outer - inner) / 2),
        cm(wide / 2),
        turns(bearing),
        m,
    )
}

/// A cairn: a slab of snow two metres across on a top at `y`.
fn cairn_on(x: Fx, z: Fx, y: Fx) -> Solid {
    Solid {
        min: V3::new(x.sub(Fx::ONE), y.sub(Fx::ratio(3, 10)), z.sub(Fx::ONE)),
        max: V3::new(x.add(Fx::ONE), y.add(Fx::ratio(2, 5)), z.add(Fx::ONE)),
        material: Material::Snow,
        shape: Shape::Box,
    }
}

/// A point on the map at a bearing and a distance from the axis.
fn out_at(bearing: i32, dist: i32) -> (Fx, Fx) {
    let d = V3::from_turns(turns(bearing));
    (
        cm(AT.0).add(d.x.mul(cm(dist))),
        cm(AT.1).add(d.z.mul(cm(dist))),
    )
}

/// **The ruins on the dome's top**: what is left of a village 353 metres up.
/// Each as a bearing and a distance from the axis (centimetres, thousandths
/// of a turn), and what it is.
#[derive(Clone, Copy)]
enum Ruin {
    /// A broken wall: along a bearing for a length, so high.
    Wall { length: i32, high: i32 },
    /// A column's stump, so high.
    Stump { high: i32 },
    /// A column fallen along a bearing, so long.
    Fallen { long: i32 },
    /// Two piers and a lintel across them, along a bearing.
    Arch,
    /// A cistern's round wall.
    Cistern,
}

const RUINS: [(i32, i32, Ruin); 18] = [
    (
        50,
        2600,
        Ruin::Wall {
            length: 700,
            high: 380,
        },
    ),
    (
        120,
        3300,
        Ruin::Wall {
            length: 500,
            high: 220,
        },
    ),
    (180, 2200, Ruin::Stump { high: 420 }),
    (230, 3700, Ruin::Arch),
    (
        300,
        2900,
        Ruin::Wall {
            length: 900,
            high: 300,
        },
    ),
    (360, 1800, Ruin::Stump { high: 180 }),
    (410, 3500, Ruin::Fallen { long: 650 }),
    (480, 2500, Ruin::Cistern),
    (
        540,
        3800,
        Ruin::Wall {
            length: 600,
            high: 450,
        },
    ),
    (600, 2000, Ruin::Stump { high: 320 }),
    (
        640,
        3000,
        Ruin::Wall {
            length: 400,
            high: 150,
        },
    ),
    (700, 3600, Ruin::Stump { high: 500 }),
    (
        800,
        2700,
        Ruin::Wall {
            length: 800,
            high: 260,
        },
    ),
    (850, 3900, Ruin::Fallen { long: 500 }),
    (900, 1900, Ruin::Stump { high: 250 }),
    (950, 3300, Ruin::Arch),
    (20, 1500, Ruin::Stump { high: 140 }),
    (
        680,
        1500,
        Ruin::Wall {
            length: 300,
            high: 120,
        },
    ),
];

fn ruin(base: Fx, bearing: i32, dist: i32, r: Ruin, out: &mut Vec<Solid>) {
    let floor = base.add(cm(TOP));
    let (x, z) = out_at(bearing, dist);
    match r {
        Ruin::Wall { length, high } => {
            // Square to the bearing: a wall across it.
            out.push(Solid::turned(
                x,
                z,
                floor.sub(Fx::ratio(1, 5)),
                floor.add(cm(high)),
                cm(length / 2),
                cm(40),
                turns(bearing + 250),
                Material::Stone,
            ));
        }
        Ruin::Stump { high } => {
            out.push(Solid::round(
                x,
                z,
                floor.sub(Fx::ratio(1, 5)),
                floor.add(cm(high)),
                cm(75),
                cm(60),
                Material::Stone,
            ));
        }
        Ruin::Fallen { long } => {
            out.push(Solid::turned(
                x,
                z,
                floor,
                floor.add(cm(120)),
                cm(long / 2),
                cm(60),
                turns(bearing + 130),
                Material::Stone,
            ));
        }
        Ruin::Arch => {
            let across = V3::from_turns(turns(bearing + 250));
            for side in [-1, 1] {
                let off = cm(200 * side);
                out.push(Solid::round(
                    x.add(across.x.mul(off)),
                    z.add(across.z.mul(off)),
                    floor.sub(Fx::ratio(1, 5)),
                    floor.add(cm(520)),
                    cm(70),
                    cm(60),
                    Material::Stone,
                ));
            }
            out.push(Solid::turned(
                x,
                z,
                floor.add(cm(520)),
                floor.add(cm(620)),
                cm(290),
                cm(60),
                turns(bearing + 250),
                Material::Stone,
            ));
        }
        Ruin::Cistern => {
            out.push(Solid::ring(
                x,
                z,
                floor.sub(Fx::ratio(1, 5)),
                floor.add(cm(130)),
                (cm(330), cm(320)),
                cm(260),
                Material::Stone,
            ));
        }
    }
}

/// **The Spire's solids**, standing on the land (`height` is the ground at
/// a map point).
pub fn build(height: &dyn Fn(Fx, Fx) -> Fx, out: &mut Vec<Solid>) {
    let (x, z) = (cm(AT.0), cm(AT.1));
    let base = height(x, z);
    // The shaft.
    out.push(Solid::round(
        x,
        z,
        base.sub(Fx::from_int(2)),
        base.add(cm(SHAFT.2)),
        cm(SHAFT.0),
        cm(SHAFT.1),
        Material::Stone,
    ));
    // The corkscrew, and the balconies every forty metres.
    let mut bearing = 0;
    let mut y = 1200;
    while y < SHAFT.2 - 600 {
        let balcony = 4000;
        let near_balcony = (y % balcony) < 300 || (y % balcony) > balcony - 300;
        if off_the_vine(bearing) && !near_balcony {
            let r = shaft_at(y);
            out.push(board(
                base,
                bearing,
                r - 100,
                r + 320,
                y,
                260,
                90,
                Material::Stone,
            ));
        }
        bearing += 280;
        y += 800;
    }
    let mut k = 0;
    let mut b = 4000;
    while b < SHAFT.2 - 600 {
        let r = shaft_at(b);
        for bearing in [0, 250, 500] {
            out.push(board(
                base,
                bearing,
                r - 100,
                r + 280,
                b,
                480,
                100,
                Material::Stone,
            ));
        }
        if k % 2 == 1 {
            let (cx, cz) = out_at(250, r + 160);
            out.push(cairn_on(cx, cz, base.add(cm(b))));
        }
        k += 1;
        b += 4000;
    }

    // The lantern: four columns on the shaft's top, into the dome's stem.
    for c in 0..4 {
        let (cx, cz) = out_at(125 + 250 * c, CHIMNEY + 30);
        out.push(Solid::round(
            cx,
            cz,
            base.add(cm(SHAFT.2 - 50)),
            base.add(cm(DOME_FROM + 250)),
            cm(130),
            cm(120),
            Material::Stone,
        ));
    }

    // The dome: eight slabs of a sphere, each a ring round the chimney.
    for s in 0..SLABS {
        let (h0, h1) = (DEPTH * s / SLABS, DEPTH * (s + 1) / SLABS);
        let r0 = dome_at(h0).max(CHIMNEY + 250);
        let r1 = dome_at(h1).max(CHIMNEY + 300);
        out.push(Solid::ring(
            x,
            z,
            base.add(cm(DOME_FROM + h0)),
            base.add(cm(DOME_FROM + h1)),
            (cm(r0), cm(r1)),
            cm(CHIMNEY),
            Material::Stone,
        ));
    }

    // The chimney's ledges: up its wall from the shaft's top, clear of the
    // vine, four metres and a half apart.
    let mut bearing = 0;
    let mut y = SHAFT.2 + 400;
    while y < TOP - 150 {
        if off_the_vine(bearing) {
            out.push(board(
                base,
                bearing,
                CHIMNEY - 230,
                CHIMNEY + 60,
                y,
                300,
                80,
                Material::Stone,
            ));
        }
        bearing += 310;
        y += 420;
    }

    // The ruins on the top, and a cairn by the chimney's mouth.
    for &(bearing, dist, r) in &RUINS {
        ruin(base, bearing, dist, r, out);
    }
    let (cx, cz) = out_at(250, CHIMNEY + 250);
    out.push(cairn_on(cx, cz, base.add(cm(TOP))));
}
