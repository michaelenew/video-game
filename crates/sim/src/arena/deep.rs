//! **The Deep**: caves under the valley (exploration 0019 §2d, built
//! 2026-10-15). The owner: "the ways down need to lead to caves with
//! monsters, and I'd like those caves to twist about and form kind of a maze
//! and keep going down and down until they open into big spectacular
//! caverns with crystals and such".
//!
//! **The land is one surface**, so nothing can be under it (the note's
//! §3b). The Deep is a place that keeps its own floor -- a pad, as a room is
//! -- sunk 140 m under the Shelves' road, south of it past the mountains,
//! and **a lid of rock** laid over the whole pit at the road's height, out to
//! where the mountains round it rise through it. From the road a path runs
//! south to a stone plain between the peaks; in the plain is **the mouth**.
//! Under the lid, everything is solids:
//!
//! | Level | Floor, over the pit's floor | What |
//! | --- | --- | --- |
//! | **The mouth** | 140 m (the lid's top) | A hole in the lid, a shaft under it |
//! | **The Upper Warren** | 110 m | A maze of tunnels fourteen metres wide, generated, ten cells by eight |
//! | **The Middle Warren** | 80 m | Another, its way in under the last one's way out, at the far end |
//! | **The Lower Warren** | 50 m | A third |
//! | **The Caverns** | 0 | The pit's own floor, 160 by 140 m and widening up its walls, under a vault 47 m up: crystal columns, crystal shards hung in the air, a lake |
//!
//! Each warren's way down is a shaft through its floor in a cell at the far
//! end of the maze from its way in; the Lower Warren's opens in the Caverns'
//! vault and reaches the floor. **A shaft is walled** from the floor above to
//! a body's height over the one below (2026-10-16, from a vine down each:
//! [`shaft`]): down is a slide down a wall, or ledge to ledge; up is the
//! ledges, zig-zagging between its north and south walls a catch apart
//! ([`shaft_ledges`]). The mazes are a fixed draw (`dice`), so both machines
//! dig the same tunnels.

use super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns};
use crate::fixed::Fx;
use crate::valley::land::{Point, at};
use crate::valley::{Gate, Kind, Place, Seam, Zone};

/// Where the pit's middle is on the map, and its floor's height
/// (centimetres).
pub const AT: [i32; 3] = [61000, -9400, -33200];

/// Half the pit's floor: x and z -- the warrens' mazes, exactly.
pub const HALF: (i32, i32) = (CELLS.0 * CELL / 2, CELLS.1 * CELL / 2);

/// The lid's top, over the pit's floor: the Shelves' road's height.
pub const LID: i32 = 14000;
/// How far past the pit's edge the lid runs, to where the land round it
/// rises through it: foothills, then the mountainside (`land::rise`).
pub const LID_OUT: i32 = 11300;
/// How thick the lid and each warren's floor are.
const SLAB: i32 = 300;

/// The warrens' floors, over the pit's floor, top to bottom.
pub const LEVELS: [i32; 3] = [11000, 8000, 5000];

/// A maze's cells: how many across (x) and deep (z), and how big.
pub const CELLS: (i32, i32) = (10, 8);
pub const CELL: i32 = 1600;
/// How thick a tunnel's wall is.
const WALL: i32 = 200;
/// How wide a shaft is.
const SHAFT: i32 = 800;

/// **The ways down**: the cell of each floor's hole, as (column, row) of
/// the maze -- the lid's mouth, then each warren's way down. Each warren's
/// way in is the hole above it; its way down is at the far end.
pub const HOLES: [(i32, i32); 4] = [(2, 7), (8, 0), (1, 6), (7, 3)];

pub static ARENA: Arena = Arena {
    id: ArenaId::DEEP,
    name: "Deep",
    creature: None,
    // The whole lid is the Deep's: the stone plain over the pit, to where
    // the mountains rise through it. Its floor is the pit's, under all of it;
    // the land's crater round the pit is sealed off by the Caverns' walls.
    bounds: Bounds::cm(
        (-HALF.0 - LID_OUT, HALF.0 + LID_OUT),
        (-HALF.1 - LID_OUT, HALF.1 + LID_OUT),
    ),
    floor: Material::Rock,
    regions: &REGIONS,
    solids: &SOLIDS,
    spawns: Spawns {
        // By the Caverns' lake.
        versus: [
            Mark::cm(-500, -2000, (-500, 0)),
            Mark::cm(500, -2000, (500, 0)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: None,
};

const SOLIDS: [Solid; 0] = [];

/// The Caverns' lake, and the crystal sand round it.
const REGIONS: [Region; 2] = [
    Region {
        area: Area::disc_cm(-3000, 2500, 2600),
        material: Material::Sand,
    },
    Region {
        area: Area::disc_cm(-3000, 2500, 2000),
        material: Material::Water,
    },
];

/// **The path** from the Shelves' road south between the mountains to the
/// lid's north edge, at the road's height (map centimetres).
pub const PATH: [Point; 3] = [
    at(60300, -1500, 5000, 300),
    at(60800, -9000, 4700, 350),
    at(61000, -15500, AT[1] + LID, 500),
];

/// The middle of a maze's cell, in the pit's coordinates.
pub const fn cell_middle(c: (i32, i32)) -> (i32, i32) {
    let w = CELLS.0 * CELL;
    let d = CELLS.1 * CELL;
    (
        -w / 2 + c.0 * CELL + CELL / 2,
        -d / 2 + c.1 * CELL + CELL / 2,
    )
}

/// **The shafts**: each hole's, from the floor it is in down to the one
/// below -- the lid's mouth to the Upper Warren, each warren to the next,
/// the Lower Warren to the Caverns' floor.
pub const SHAFTS: [(i32, i32); 4] = [
    (LID, LEVELS[0]),
    (LEVELS[0], LEVELS[1]),
    (LEVELS[1], LEVELS[2]),
    (LEVELS[2], 0),
];

/// How far apart a shaft's ledges are, one over the next: inside the
/// Bulwark's catch across the shaft.
pub const LEDGE_RISE: i32 = 400;
/// How far a ledge stands out of its wall, and how wide it is along it.
const LEDGE_OUT: i32 = 220;
const LEDGE_WIDE: i32 = 300;
/// How far over the floor below a shaft's walls end: a body walks out under.
const OPEN: i32 = 300;
/// The first shard's top, over the Caverns' floor: and the sill of the slot
/// in the Caverns' shaft's west wall that steps out on to it, half a metre
/// under the ledge beside it.
const SHARDS_FROM: i32 = 3550;

/// **A shaft's ledges**, foot to top, in the pit's coordinates: every
/// [`LEDGE_RISE`] up from the floor below, the first on the south wall and
/// then turn about, the last a step under the floor it comes up through.
pub fn shaft_ledges(k: usize) -> Vec<([i32; 3], [i32; 3])> {
    let (from, to) = SHAFTS[k];
    let (x, z) = cell_middle(HOLES[k]);
    let h = SHAFT / 2;
    let mut out = Vec::new();
    let mut y = to + LEDGE_RISE;
    let mut n = 0;
    while y <= from - 150 {
        let (z0, z1) = if n % 2 == 0 {
            (z - h, z - h + LEDGE_OUT)
        } else {
            (z + h - LEDGE_OUT, z + h)
        };
        out.push((
            [x - LEDGE_WIDE / 2, y - 60, z0],
            [x + LEDGE_WIDE / 2, y, z1],
        ));
        y += LEDGE_RISE;
        n += 1;
    }
    out
}

/// **A shaft's walls and ledges**: four walls round the hole from the top
/// of the floor it is in -- through the slab, so the rim is the one edge --
/// to [`OPEN`] over the floor below, and its ledges.
fn shaft(k: usize, out: &mut Vec<Solid>) {
    let (from, to) = SHAFTS[k];
    let (x, z) = cell_middle(HOLES[k]);
    let h = SHAFT / 2;
    let (lo, hi) = (to + OPEN, from);
    for (a, b) in [
        ((x - h - WALL, z - h - WALL), (x + h + WALL, z - h)),
        ((x - h - WALL, z + h), (x + h + WALL, z + h + WALL)),
        ((x + h, z - h), (x + h + WALL, z + h)),
    ] {
        out.push(put([a.0, lo, a.1], [b.0, hi, b.1], Material::Rock));
    }
    // The west wall; the Caverns' with a slot through it onto the first
    // shard (`crystals`), its sill a step under a ledge.
    let west = ((x - h - WALL, z - h), (x - h, z + h));
    match (k == SHAFTS.len() - 1).then_some(SHARDS_FROM) {
        Some(sill) => {
            out.push(put(
                [west.0.0, lo, west.0.1],
                [west.1.0, sill, west.1.1],
                Material::Rock,
            ));
            out.push(put(
                [west.0.0, sill + 300, west.0.1],
                [west.1.0, hi, west.1.1],
                Material::Rock,
            ));
        }
        None => out.push(put(
            [west.0.0, lo, west.0.1],
            [west.1.0, hi, west.1.1],
            Material::Rock,
        )),
    }
    for (a, b) in shaft_ledges(k) {
        out.push(put(a, b, Material::Rock));
    }
}

/// How the Deep joins the valley.
pub static PLACE: Place = Place {
    arena: ArenaId::DEEP,
    kind: Kind::Reach,
    seams: &SEAMS,
    vents: &[],
};

const SEAMS: [Seam; 1] = [
    // Back up the path to the Shelves' road, from the lid's north edge.
    Seam {
        zone: Zone::cm(
            [-1000, LID - 300, HALF.1 + LID_OUT - 600],
            [1000, LID + 700, HALF.1 + LID_OUT],
        ),
        to: ArenaId::SHELVES,
        at: 4,
        marks: [
            Mark::cm(-150, HALF.1 + 3000, (-150, 0)),
            Mark::cm(150, HALF.1 + 3000, (150, 0)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Shelves",
    },
];

// ---------------------------------------------------------------------------
// Dug
// ---------------------------------------------------------------------------

fn cm(v: i32) -> Fx {
    super::cm(v)
}

/// A box in the pit's coordinates, on the map.
fn put(min: [i32; 3], max: [i32; 3], m: Material) -> Solid {
    Solid::cm(
        [AT[0] + min[0], AT[1] + min[1], AT[2] + min[2]],
        [AT[0] + max[0], AT[1] + max[1], AT[2] + max[2]],
        m,
    )
}

/// A deterministic number in [0, 1000) from two integers and a salt.
fn dice(a: i32, b: i32, salt: u32) -> i32 {
    let mut h = (a as u32).wrapping_mul(0x9E37_79B1)
        ^ (b as u32).wrapping_mul(0x85EB_CA77)
        ^ salt.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h % 1000) as i32
}

/// **A slab with a hole in it**: from `lo` to `hi` (x, z) at `top`, `SLAB`
/// thick, less the square round `hole`'s middle that its shaft's walls stand
/// in (`SHAFT` across, and a `WALL` either side) -- four boxes.
fn holed(lo: (i32, i32), hi: (i32, i32), top: i32, hole: Option<(i32, i32)>, out: &mut Vec<Solid>) {
    let (y0, y1) = (top - SLAB, top);
    let Some(c) = hole else {
        out.push(put([lo.0, y0, lo.1], [hi.0, y1, hi.1], Material::Rock));
        return;
    };
    let (x, z) = cell_middle(c);
    let h = SHAFT / 2 + WALL;
    // West and east of the hole, the slab's whole depth; north and south of
    // it, between them.
    out.push(put([lo.0, y0, lo.1], [x - h, y1, hi.1], Material::Rock));
    out.push(put([x + h, y0, lo.1], [hi.0, y1, hi.1], Material::Rock));
    out.push(put([x - h, y0, lo.1], [x + h, y1, z - h], Material::Rock));
    out.push(put([x - h, y0, z + h], [x + h, y1, hi.1], Material::Rock));
}

/// **A maze, dug**: every cell reached from every other by one way, by a
/// walk that turns at random and backs up when it is boxed in. Which walls
/// stand, as (east of each cell, north of each cell).
fn maze(salt: u32) -> (Vec<bool>, Vec<bool>) {
    let (w, d) = (CELLS.0 as usize, CELLS.1 as usize);
    let mut east = vec![true; w * d];
    let mut north = vec![true; w * d];
    let mut seen = vec![false; w * d];
    let mut stack = vec![(0usize, 0usize)];
    seen[0] = true;
    let mut step = 0;
    while let Some(&(x, z)) = stack.last() {
        step += 1;
        let mut open: Vec<(usize, usize, u8)> = Vec::new();
        if x + 1 < w && !seen[z * w + x + 1] {
            open.push((x + 1, z, 0));
        }
        if x > 0 && !seen[z * w + x - 1] {
            open.push((x - 1, z, 1));
        }
        if z + 1 < d && !seen[(z + 1) * w + x] {
            open.push((x, z + 1, 2));
        }
        if z > 0 && !seen[(z - 1) * w + x] {
            open.push((x, z - 1, 3));
        }
        if open.is_empty() {
            stack.pop();
            continue;
        }
        let pick = dice(x as i32 * 31 + step, z as i32, salt) as usize % open.len();
        let (nx, nz, way) = open[pick];
        match way {
            0 => east[z * w + x] = false,
            1 => east[z * w + nx] = false,
            2 => north[z * w + x] = false,
            _ => north[nz * w + x] = false,
        }
        seen[nz * w + nx] = true;
        stack.push((nx, nz));
    }
    (east, north)
}

/// **One warren**: its floor (with its way down), its maze's walls from
/// that floor to the floor above, and the outer wall round it.
fn warren(level: usize, out: &mut Vec<Solid>) {
    let floor = LEVELS[level];
    let ceiling = if level == 0 { LID } else { LEVELS[level - 1] } - SLAB;
    let w = CELLS.0 * CELL;
    let d = CELLS.1 * CELL;
    let (x0, z0) = (-w / 2, -d / 2);
    holed(
        (x0, z0),
        (x0 + w, z0 + d),
        floor,
        Some(HOLES[level + 1]),
        out,
    );
    let (east, north) = maze(17 + level as u32);
    let (cw, cd) = (CELLS.0 as usize, CELLS.1 as usize);
    let h = WALL / 2;
    // East walls, merged down each column line where they run on.
    for x in 0..cw - 1 {
        let wx = x0 + (x as i32 + 1) * CELL;
        let mut z = 0;
        while z < cd {
            if !east[z * cw + x] {
                z += 1;
                continue;
            }
            let start = z;
            while z < cd && east[z * cw + x] {
                z += 1;
            }
            out.push(put(
                [wx - h, floor, z0 + start as i32 * CELL - h],
                [wx + h, ceiling, z0 + z as i32 * CELL + h],
                Material::Rock,
            ));
        }
    }
    // North walls, merged along each row line.
    for z in 0..cd - 1 {
        let wz = z0 + (z as i32 + 1) * CELL;
        let mut x = 0;
        while x < cw {
            if !north[z * cw + x] {
                x += 1;
                continue;
            }
            let start = x;
            while x < cw && north[z * cw + x] {
                x += 1;
            }
            out.push(put(
                [x0 + start as i32 * CELL - h, floor, wz - h],
                [x0 + x as i32 * CELL + h, ceiling, wz + h],
                Material::Rock,
            ));
        }
    }
    // The outer wall.
    for (lo, hi) in [
        ((x0 - WALL, z0 - WALL), (x0 + w + WALL, z0)),
        ((x0 - WALL, z0 + d), (x0 + w + WALL, z0 + d + WALL)),
        ((x0 - WALL, z0), (x0, z0 + d)),
        ((x0 + w, z0), (x0 + w + WALL, z0 + d)),
    ] {
        out.push(put(
            [lo.0, floor, lo.1],
            [hi.0, ceiling, hi.1],
            Material::Rock,
        ));
    }
}

/// **The Caverns' crystals**: columns from the floor, spears down from the
/// vault, and shards hung in the air -- a way across the cavern above its
/// floor, from under the Lower Warren's shaft to the lake.
fn crystals(out: &mut Vec<Solid>) {
    let vault = LEVELS[2] - SLAB;
    let (bx, bz) = (AT[0], AT[2]);
    let c = |v: i32| cm(v);
    for k in 0..26 {
        let x = -7000 + dice(k, 1, 41) * 14;
        let z = -6200 + dice(k, 2, 41) * 12;
        // Not in the lake, nor under the shaft.
        let (lx, lz) = (x + 3000, z - 2500);
        if lx * lx / 100 + lz * lz / 100 < 2700 * 2700 / 100 {
            continue;
        }
        let (sx, sz) = cell_middle(HOLES[3]);
        if (x - sx).abs() < 1200 && (z - sz).abs() < 1200 {
            continue;
        }
        let r = 120 + dice(k, 3, 41) * 3 / 10;
        let tall = 600 + dice(k, 4, 41) * 3;
        out.push(Solid::round(
            c(bx + x),
            c(bz + z),
            c(AT[1] - 100),
            c(AT[1] + tall),
            c(r),
            c(r * 3 / 10),
            Material::Crystal,
        ));
    }
    for k in 0..14 {
        let x = -7000 + dice(k, 5, 43) * 14;
        let z = -6200 + dice(k, 6, 43) * 12;
        let r = 150 + dice(k, 7, 43) / 4;
        let long = 800 + dice(k, 8, 43) * 2;
        out.push(Solid::round(
            c(bx + x),
            c(bz + z),
            c(AT[1] + vault - long),
            c(AT[1] + vault + 50),
            c(r / 5),
            c(r),
            Material::Crystal,
        ));
    }
    // The shards: from beside the shaft, stepping down and across toward the
    // lake, each a jump from the last.
    let (sx, sz) = cell_middle(HOLES[3]);
    // Out past the shaft's west wall, its first a step from the slot in it.
    let mut at = (sx - SHAFT / 2 - WALL - 550, sz, SHARDS_FROM);
    for k in 0..9 {
        let yaw = Fx::ratio(dice(k, 9, 47) % 1000, 1000);
        out.push(Solid::turned(
            c(bx + at.0),
            c(bz + at.1),
            c(AT[1] + at.2 - 150),
            c(AT[1] + at.2),
            c(300),
            c(150),
            yaw,
            Material::Crystal,
        ));
        at = (at.0 - 900, at.1 - 300 + dice(k, 10, 47) / 2, at.2 - 450);
    }
}

/// **The Deep's solids**: the lid and its mouth, the three warrens, the
/// Caverns' crystals, and a cairn on each warren's floor by its way in and
/// one by the lake.
pub fn build(out: &mut Vec<Solid>) {
    let (hx, hz) = (HALF.0 + LID_OUT, HALF.1 + LID_OUT);
    holed((-hx, -hz), (hx, hz), LID, Some(HOLES[0]), out);
    for k in 0..SHAFTS.len() {
        shaft(k, out);
    }
    for level in 0..LEVELS.len() {
        warren(level, out);
    }
    // The Caverns' walls, round the pit's edge from its floor to the Lower
    // Warren's: past them is the land's crater, which nobody stands in.
    let vault = LEVELS[2] - SLAB;
    let (w, d) = HALF;
    for (lo, hi) in [
        ((-w - WALL, -d - WALL), (w + WALL, -d)),
        ((-w - WALL, d), (w + WALL, d + WALL)),
        ((-w - WALL, -d), (-w, d)),
        ((w, -d), (w + WALL, d)),
    ] {
        out.push(put([lo.0, -200, lo.1], [hi.0, vault, hi.1], Material::Rock));
    }
    crystals(out);
    for (k, floor) in LEVELS.iter().enumerate() {
        let (x, z) = cell_middle(HOLES[k]);
        let (x, z) = (x + 500, z - 500);
        out.push(put(
            [x - 100, floor - 30, z - 100],
            [x + 100, floor + 40, z + 100],
            Material::Snow,
        ));
    }
    out.push(put([-500, -30, -1100], [-300, 40, -900], Material::Snow));
}
