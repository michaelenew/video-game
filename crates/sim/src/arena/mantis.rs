//! The Shrine: the Mantis's arena, at the end of the world's fifth tier
//! (`creatures/mantis.md` §11).
//!
//! **A round stone court thirty metres across, walled at 1.5 m, open to the
//! sky, with four columns** 1.2 m wide and 8 m tall standing ten metres from
//! its middle. No platforms: a platform is a place to be in the air from.
//!
//! | What | Size | Why |
//! | --- | --- | --- |
//! | The wall | 1.5 m high, round | The edge of a duel: somewhere to be backed into |
//! | Four columns | 1.2 m across, 8 m tall, 10 m out | The only cover, three ways: a column in the lunge's lane stops it; a column is what it backs toward to line two hunters up; a column between you and it is somewhere to stand while it prays |
//!
//! The court is round as boxes allow: a square of wall with each corner
//! filled in, which puts every point of the wall between fifteen and fifteen
//! and a half metres from the middle. The columns stand on the diagonals, so
//! the way in from the west is open and the first decision -- break its
//! prayer or wait -- is made in plain sight of it.

use super::rim::Rim;
use super::{Arena, ArenaId, Bounds, HuntMarks, Mark, Material, Solid, Spawns};
use crate::species::SpeciesId;

use Material::Stone;

pub static ARENA: Arena = Arena {
    id: ArenaId::MANTIS,
    name: "Shrine",
    creature: Some(SpeciesId::MANTIS),
    bounds: Bounds::cm((-1650, 1650), (-1650, 1650)),
    floor: Stone,
    regions: &[],
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-400, 0, (400, 0)), Mark::cm(400, 0, (-400, 0))],
        hunt: Some(HuntMarks {
            // In from the west, along the trail, the court in front of them.
            hunters: [
                Mark::cm(-1100, -150, (0, -150)),
                Mark::cm(-1100, 150, (0, 150)),
            ],
            // In the middle of the court, at prayer, facing the way in.
            creatures: [Mark::cm(0, 0, (-1000, 0)), Mark::cm(0, 300, (-1000, 300))],
        }),
    },
    sites: &[],
    rim: Some(Rim::new(
        (-1650, 1650),
        (-1650, 1650),
        [150, 150, 150, 150],
        600,
    )),
};

/// The columns, by their middles in centimetres: on the diagonals, ten
/// metres out.
pub const COLUMNS: [(i32, i32); 4] = [(707, 707), (707, -707), (-707, 707), (-707, -707)];

/// A column: 1.2 m across and 8 m tall.
const fn column(x: i32, z: i32) -> Solid {
    Solid::cm([x - 60, 0, z - 60], [x + 60, 800, z + 60], Stone)
}

/// A corner of the court, filled in from where the round wall would run.
const fn corner(sx: i32, sz: i32) -> Solid {
    let (x0, x1) = if sx > 0 { (1100, 1650) } else { (-1650, -1100) };
    let (z0, z1) = if sz > 0 { (1100, 1650) } else { (-1650, -1100) };
    Solid::cm([x0, 0, z0], [x1, 150, z1], Stone)
}

const SOLIDS: [Solid; 12] = [
    // The wall: a square at fifteen metres...
    Solid::cm([-1650, 0, -1650], [-1500, 150, 1650], Stone),
    Solid::cm([1500, 0, -1650], [1650, 150, 1650], Stone),
    Solid::cm([-1650, 0, -1650], [1650, 150, -1500], Stone),
    Solid::cm([-1650, 0, 1500], [1650, 150, 1650], Stone),
    // ...with its corners filled, so the court is round as boxes allow.
    corner(1, 1),
    corner(1, -1),
    corner(-1, 1),
    corner(-1, -1),
    column(COLUMNS[0].0, COLUMNS[0].1),
    column(COLUMNS[1].0, COLUMNS[1].1),
    column(COLUMNS[2].0, COLUMNS[2].1),
    column(COLUMNS[3].0, COLUMNS[3].1),
];
