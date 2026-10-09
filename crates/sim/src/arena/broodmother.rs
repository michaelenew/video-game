//! The Hollows: the Broodmother's cave, off Pinewood.
//!
//! A cavern about 30 by 26 m, its floor bare rock, under a vault **12 m** at
//! the middle that comes down to 10 m and then 8 m toward the walls -- high
//! enough for the camera and the Dual mage's float, low enough to be a
//! backdrop behind every sac (the aiming argument of her §10: a ranged shot at
//! a sac goes to the point on the crosshair's ray, and the nearer the solid
//! behind the sac, the nearer that point is to the sac). The walls rise into
//! the vault; the corners are cut in, so the floor is an oval of boxes.
//!
//! **Two rock pillars**, two metres thick, at mid-cave: the solids a web shot
//! is answered with, and the one place a web line cannot cross. **A ring of
//! 1.5 m shelves** along the back (east) wall gives the middle classes the
//! fore pair from a shelf, if they can fight her over to one. **Web anchors**
//! are sites (`anchor`) on the walls, where her line goes. Light is a pale
//! ambient from cracks in the vault, drawn by the dressing
//! (`game/src/arenas/broodmother.rs`). See
//! `docs/design/creatures/broodmother.md` §11.

use super::rim::Rim;
use super::{Arena, ArenaId, Bounds, HuntMarks, Mark, Material, Solid, Spawns};
use crate::objective::Site;
use crate::species::SpeciesId;

use Material::{Rock, Stone};

pub static ARENA: Arena = Arena {
    id: ArenaId::BROODMOTHER,
    name: "Hollows",
    creature: Some(SpeciesId::BROODMOTHER),
    bounds: Bounds::cm((-1500, 1500), (-1300, 1300)),
    floor: Rock,
    regions: &[],
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-400, 0, (400, 0)), Mark::cm(400, 0, (-400, 0))],
        hunt: Some(HuntMarks {
            hunters: [
                Mark::cm(-1150, -200, (0, -200)),
                Mark::cm(-1150, 200, (0, 200)),
            ],
            creatures: [
                Mark::cm(200, 0, (-1150, 0)),
                Mark::cm(200, 700, (-1150, 700)),
            ],
        }),
    },
    sites: &SITES,
    rim: Some(Rim::new(
        (-1650, 1650),
        (-1450, 1450),
        [1400, 1400, 1400, 1400],
        500,
    )),
};

/// Where her web line goes: old anchors in the walls, about a fighter's
/// reach above the floor, round the cave. A point each; nothing stands there.
const SITES: [Site; 8] = [
    Site {
        name: "anchor",
        route: &[(-1450, -700)],
        size: (0, 0, 0),
        material: Rock,
    },
    Site {
        name: "anchor",
        route: &[(-1450, 700)],
        size: (0, 0, 0),
        material: Rock,
    },
    Site {
        name: "anchor",
        route: &[(0, -1250)],
        size: (0, 0, 0),
        material: Rock,
    },
    Site {
        name: "anchor",
        route: &[(0, 1250)],
        size: (0, 0, 0),
        material: Rock,
    },
    Site {
        name: "anchor",
        route: &[(1450, -700)],
        size: (0, 0, 0),
        material: Rock,
    },
    Site {
        name: "anchor",
        route: &[(1450, 700)],
        size: (0, 0, 0),
        material: Rock,
    },
    Site {
        name: "anchor",
        route: &[(-900, 1150)],
        size: (0, 0, 0),
        material: Rock,
    },
    Site {
        name: "anchor",
        route: &[(900, -1150)],
        size: (0, 0, 0),
        material: Rock,
    },
];

const SOLIDS: [Solid; 26] = [
    // The walls, rock from the floor into the vault, a metre and a half
    // thick, just outside the floor.
    Solid::cm([-1650, 0, -1450], [-1500, 1400, 1450], Rock),
    Solid::cm([1500, 0, -1450], [1650, 1400, 1450], Rock),
    Solid::cm([-1650, 0, -1450], [1650, 1400, -1300], Rock),
    Solid::cm([-1650, 0, 1300], [1650, 1400, 1450], Rock),
    // The corners cut in, so the floor is an oval: a step of rock in each.
    Solid::cm([-1500, 0, -1300], [-1100, 1400, -1000], Rock),
    Solid::cm([-1500, 0, 1000], [-1100, 1400, 1300], Rock),
    Solid::cm([1100, 0, -1300], [1500, 1400, -1000], Rock),
    Solid::cm([1100, 0, 1000], [1500, 1400, 1300], Rock),
    Solid::cm([-1500, 0, -1000], [-1350, 1400, -800], Rock),
    Solid::cm([-1500, 0, 800], [-1350, 1400, 1000], Rock),
    Solid::cm([1350, 0, -1000], [1500, 1400, -800], Rock),
    Solid::cm([1350, 0, 800], [1500, 1400, 1000], Rock),
    // The vault: 12 m over the middle...
    Solid::cm([-1500, 1200, -1300], [1500, 1400, 1300], Rock),
    // ...10 m over a band three metres in from the walls...
    Solid::cm([-1500, 1000, -1300], [1500, 1200, -1000], Rock),
    Solid::cm([-1500, 1000, 1000], [1500, 1200, 1300], Rock),
    Solid::cm([-1500, 1000, -1000], [-1200, 1200, 1000], Rock),
    Solid::cm([1200, 1000, -1000], [1500, 1200, 1000], Rock),
    // ...and 8 m along the walls themselves.
    Solid::cm([-1500, 800, -1300], [1500, 1000, -1150], Rock),
    Solid::cm([-1500, 800, 1150], [1500, 1000, 1300], Rock),
    Solid::cm([-1500, 800, -1150], [-1350, 1000, 1150], Rock),
    Solid::cm([1350, 800, -1150], [1500, 1000, 1150], Rock),
    // Two pillars, two metres thick, floor to vault, at mid-cave.
    Solid::cm([-600, 0, -600], [-400, 1200, -400], Rock),
    Solid::cm([400, 0, 400], [600, 1200, 600], Rock),
    // The shelves along the back (east) wall: a metre and a half up, two
    // metres deep, with a gap between them to walk through.
    Solid::cm([1150, 0, -650], [1350, 150, -150], Stone),
    Solid::cm([1150, 0, 150], [1350, 150, 650], Stone),
    Solid::cm([900, 0, -1000], [1350, 150, -800], Stone),
];
