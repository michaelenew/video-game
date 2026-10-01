//! The Commons, the Gnawer den (gnawers.md §11).
//!
//! The first fight place past Hearth: a grazed meadow with a bank along its
//! north side and the den's mouth dug into the middle of it -- a dark hole
//! the pack runs for when it breaks. **Open floor in the middle**, which is
//! the lesson: nothing to put your back to where you walk in. At the edges,
//! the things that answer it, each covered by something the pack does
//! (§3's table):
//!
//! | What | Height | What covers it |
//! | --- | --- | --- |
//! | The bank, either side of the den | 3 m | A back to stand against; too high to stand on for anyone but the Dual mage, who is treed |
//! | A fallen trunk, west | 1.5 m | They **scramble** up it |
//! | Two standing boulders, east | 3.5 m | A back, and too tall to climb: a corner, which does not win |
//! | Low walls, the other three sides | 1.5 m | A back; stood on, scrambled |
//!
//! 36 by 30 metres: the proving ground's 28 had the den mouth at one wall's
//! midpoint (§10), and the extra room is the open middle a retreating fighter
//! has to cross.

use super::{Area, Arena, ArenaId, Bounds, HuntMarks, Mark, Material, Region, Solid, Spawns};
use crate::species::SpeciesId;

use Material::{Grass, Ground, Rock, Stone, Wood};

pub static ARENA: Arena = Arena {
    id: ArenaId::GNAWERS,
    name: "Commons",
    creature: Some(SpeciesId::GNAWERS),
    bounds: Bounds::cm((-1800, 1800), (-1500, 1500)),
    floor: Grass,
    regions: &REGIONS,
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-400, 0, (400, 0)), Mark::cm(400, 0, (-400, 0))],
        hunt: Some(HuntMarks {
            // Walking in from the trail at the south edge, the den ahead.
            hunters: [
                Mark::cm(-150, -1150, (-150, 0)),
                Mark::cm(150, -1150, (150, 0)),
            ],
            // The pack at the den's mouth, facing out: where it musters is its
            // home, the place it runs for (`pack::Pack::home`).
            creatures: [Mark::cm(0, 1150, (0, 0)), Mark::cm(0, 1150, (0, 0))],
        }),
    },
    sites: &[],
};

const REGIONS: [Region; 3] = [
    // Bare earth trodden out of the den's mouth, and where the carcass lies.
    Region {
        area: Area::disc_cm(0, 1100, 450),
        material: Ground,
    },
    Region {
        area: Area::disc_cm(-500, 700, 200),
        material: Ground,
    },
    // The bank's foot is stony.
    Region {
        area: Area::rect_cm((-1800, 1800), (1000, 1100)),
        material: Rock,
    },
];

const SOLIDS: [Solid; 10] = [
    // Low walls on three sides, just outside the floor.
    Solid::cm([-1900, 0, -1600], [-1800, 150, 1500], Stone),
    Solid::cm([1800, 0, -1600], [1900, 150, 1500], Stone),
    Solid::cm([-1900, 0, -1600], [1900, 150, -1500], Stone),
    // The bank along the north side, 3 m high and 4 m deep, either side of
    // the den's mouth...
    Solid::cm([-1900, 0, 1100], [-150, 300, 1600], Rock),
    Solid::cm([150, 0, 1100], [1900, 300, 1600], Rock),
    // ...which is a hole three metres wide and three and a half deep, with
    // the bank over it and behind it.
    Solid::cm([-150, 0, 1450], [150, 300, 1600], Rock),
    Solid::cm([-150, 200, 1100], [150, 300, 1450], Rock),
    // A fallen trunk, west of the middle: a platform they scramble.
    Solid::cm([-1150, 0, -550], [-350, 150, -450], Wood),
    // Two standing boulders, east: backs to stand against, too tall to climb.
    Solid::cm([700, 0, 150], [950, 350, 400], Rock),
    Solid::cm([1100, 0, -800], [1350, 350, -550], Rock),
];
