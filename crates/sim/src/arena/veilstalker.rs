//! The Ashwood: the Veilstalker's arena, in a northern birch wood
//! (veilstalker.md §11). The charcoal-burners' last clearing.
//!
//! **36 × 36 m of snow**, larger than the proving ground because a hunt
//! needs room for a trail to be a trail, inside **a 3 m wall of stacked
//! cordwood** a metre and a half thick, whose top is a walkable ledge -- the
//! creature's other perch, and too high for the Ridgeback's 1.5 m wall to
//! have been.
//!
//! | What | Size | Why |
//! | --- | --- | --- |
//! | Snow | the floor | It takes prints for eight seconds: the read |
//! | Ash | two pits, 3 m across the middle of each | Prints last half as long and stir embers |
//! | The stream | 3 m wide across the north-east corner | A leg in it is a ripple ring, and prints end at it |
//! | Six dead trunks | 1.2 m across, 5 m tall, flat broken tops | Its perches; solids the quills are answered behind |
//! | Four braziers | in the open | Every class's fire: tipped once, coals for eight seconds |
//! | The wall | 3 m high, 1.5 m thick | The tempting corner, and a ledge to pounce from |
//!
//! The braziers are **sites** (`brazier`), so the species finds them where
//! the arena puts them; they stand in the open, away from the corners, so a
//! corner beside one is a choice and not a given (§3).

use super::rim::Rim;
use super::{Area, Arena, ArenaId, Bounds, HuntMarks, Mark, Material, Region, Solid, Spawns};
use crate::objective::Site;
use crate::species::SpeciesId;

use Material::{Ash, Snow, Water, Wood};

pub static ARENA: Arena = Arena {
    id: ArenaId::VEILSTALKER,
    name: "Ashwood",
    creature: Some(SpeciesId::VEILSTALKER),
    // The bounds take in the wall: a creature walks out onto its ledge.
    bounds: Bounds::cm((-1800, 1800), (-1800, 1800)),
    floor: Snow,
    regions: &REGIONS,
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-400, 0, (400, 0)), Mark::cm(400, 0, (-400, 0))],
        hunt: Some(HuntMarks {
            // In from the west, across the open snow.
            hunters: [
                Mark::cm(-1200, -150, (0, -150)),
                Mark::cm(-1200, 150, (0, 150)),
            ],
            // Somewhere in the east, already cloaked.
            creatures: [
                Mark::cm(1250, 0, (-1200, 0)),
                Mark::cm(1250, 400, (-1200, 0)),
            ],
        }),
    },
    sites: &SITES,
    rim: Some(Rim::new(
        (-1800, 1800),
        (-1800, 1800),
        [300, 300, 300, 300],
        600,
    )),
};

/// The trunks, by their middles in centimetres: spread so that wherever the
/// fight is, one is a perch over it and one is cover from a fan.
pub const TRUNKS: [(i32, i32); 6] = [
    (-850, -650),
    (-450, 1050),
    (550, 1150),
    (1100, -350),
    (250, -1050),
    (-1150, -1150),
];

/// The braziers, by where they stand in centimetres.
pub const BRAZIERS: [(i32, i32); 4] = [(-550, -550), (500, 450), (-650, 650), (650, -700)];

/// A dead trunk: 1.2 m across, 5 m tall, its broken top flat enough to stand
/// on.
const fn trunk(x: i32, z: i32) -> Solid {
    Solid::cm([x - 60, 0, z - 60], [x + 60, 500, z + 60], Wood)
}

const SOLIDS: [Solid; 10] = [
    // The cordwood wall, three metres high and a metre and a half thick,
    // just inside the bounds.
    Solid::cm([-1800, 0, -1800], [-1650, 300, 1800], Wood),
    Solid::cm([1650, 0, -1800], [1800, 300, 1800], Wood),
    Solid::cm([-1800, 0, -1800], [1800, 300, -1650], Wood),
    Solid::cm([-1800, 0, 1650], [1800, 300, 1800], Wood),
    trunk(TRUNKS[0].0, TRUNKS[0].1),
    trunk(TRUNKS[1].0, TRUNKS[1].1),
    trunk(TRUNKS[2].0, TRUNKS[2].1),
    trunk(TRUNKS[3].0, TRUNKS[3].1),
    trunk(TRUNKS[4].0, TRUNKS[4].1),
    trunk(TRUNKS[5].0, TRUNKS[5].1),
];

/// A brazier's site: where it stands. Half a metre either way and 1.2 m
/// tall, for the tools that list sites; not a solid -- the brazier is drawn
/// from the species' marks and tipped by a blow.
const fn brazier(route: &'static [(i32, i32)]) -> Site {
    Site {
        name: "brazier",
        route,
        size: (50, 50, 120),
        material: Material::Stone,
    }
}

const SITES: [Site; 4] = [
    brazier(&[BRAZIERS[0]]),
    brazier(&[BRAZIERS[1]]),
    brazier(&[BRAZIERS[2]]),
    brazier(&[BRAZIERS[3]]),
];

/// The floor: two ash pits where the burners worked, and the stream across
/// the north-east corner as a run of discs a metre and a half across --
/// three metres wide, from the north wall to the east one.
const REGIONS: [Region; 13] = [
    Region {
        area: Area::disc_cm(-800, 300, 300),
        material: Ash,
    },
    Region {
        area: Area::disc_cm(500, -500, 300),
        material: Ash,
    },
    stream(0),
    stream(1),
    stream(2),
    stream(3),
    stream(4),
    stream(5),
    stream(6),
    stream(7),
    stream(8),
    stream(9),
    stream(10),
];

/// One disc of the stream, `i` steps along it from the north wall.
const fn stream(i: i32) -> Region {
    // From (700, 1650) to (1650, 700): eleven steps of a little under a
    // metre and a half each along the diagonal.
    let x = 700 + i * 95;
    let z = 1650 - i * 95;
    Region {
        area: Area::disc_cm(x, z, 150),
        material: Water,
    }
}
