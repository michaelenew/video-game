//! The Pan: the Sandmaw's arena, in the Glass Flats (sandmaw.md §1 and §11).
//!
//! **36 × 36 m of sand inside a rim of glassy rock 1.5 m high**, with three
//! rock islands, each about five metres across and half a metre high, set in a
//! triangle fourteen metres apart, and one boulder on each. The worm cannot
//! pass under rock -- the rim included -- so the islands are where nothing
//! from below reaches, and the boulders are the cover from the spit that is
//! the islands' price (§3). The sand is the floor everywhere else: the floor's
//! material is what the worm swims in, and every solid here is rock it cannot.
//!
//! | What | Height | Why |
//! | --- | --- | --- |
//! | The rim | 1.5 m | The edge, and rock: it surfaces no closer than its own radius |
//! | Three islands | 0.5 m | A breath, not a room: footfalls ring on rock, and it spits |
//! | A boulder on each | 2.2 m above the island | Behind it the spit cannot reach you |
//!
//! Each island is two crossed boxes, so it reads as a round outcrop rather
//! than a plinth, and is still the box the collision is written for. Bones
//! half sunk in the sand are drawn (`game/src/arenas/sandmaw.rs`) and collide
//! with nothing.

use super::rim::Rim;
use super::{Arena, ArenaId, Bounds, HuntMarks, Mark, Material, Solid, Spawns};
use crate::species::SpeciesId;

use Material::{Rock, Sand};

pub static ARENA: Arena = Arena {
    id: ArenaId::SANDMAW,
    name: "Pan",
    creature: Some(SpeciesId::SANDMAW),
    bounds: Bounds::cm((-1800, 1800), (-1800, 1800)),
    floor: Sand,
    regions: &[],
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(0, -1200, (0, 0)), Mark::cm(0, 1200, (0, 0))],
        hunt: Some(HuntMarks {
            // Walking in from the west rim, the Pan in front of them.
            hunters: [
                Mark::cm(-1300, -200, (0, -200)),
                Mark::cm(-1300, 200, (0, 200)),
            ],
            // Under the middle of the Pan, swimming. Where it opens is where
            // its first circle is.
            creatures: [
                Mark::cm(300, 0, (-1300, 0)),
                Mark::cm(300, 600, (-1300, 600)),
            ],
        }),
    },
    sites: &[],
    rim: Some(Rim::new(
        (-1900, 1900),
        (-1900, 1900),
        [150, 150, 150, 150],
        600,
    )),
};

/// The middle of each island, in centimetres: a triangle fourteen metres a
/// side, one corner east of the middle and two to the west of it.
pub const ISLANDS: [(i32, i32); 3] = [(-400, -700), (-400, 700), (810, 0)];

/// An island's two crossed boxes and its boulder, round a middle.
const fn island(x: i32, z: i32) -> [Solid; 3] {
    [
        Solid::cm([x - 250, 0, z - 180], [x + 250, 50, z + 180], Rock),
        Solid::cm([x - 180, 0, z - 250], [x + 180, 50, z + 250], Rock),
        // The boulder: off the middle toward the island's own edge, so there
        // is a lee to stand in on the far side of it.
        Solid::cm([x - 70, 50, z - 70], [x + 70, 270, z + 70], Rock),
    ]
}

const A: [Solid; 3] = island(ISLANDS[0].0, ISLANDS[0].1);
const B: [Solid; 3] = island(ISLANDS[1].0, ISLANDS[1].1);
const C: [Solid; 3] = island(ISLANDS[2].0, ISLANDS[2].1);

const SOLIDS: [Solid; 13] = [
    // The rim: a metre thick and a metre and a half high, just outside the sand.
    Solid::cm([-1900, 0, -1900], [-1800, 150, 1900], Rock),
    Solid::cm([1800, 0, -1900], [1900, 150, 1900], Rock),
    Solid::cm([-1900, 0, -1900], [1900, 150, -1800], Rock),
    Solid::cm([-1900, 0, 1800], [1900, 150, 1900], Rock),
    A[0],
    A[1],
    A[2],
    B[0],
    B[1],
    B[2],
    C[0],
    C[1],
    C[2],
];
