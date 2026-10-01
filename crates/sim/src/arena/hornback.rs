//! The low meadow, and the crossing (hornback.md §11).
//!
//! **The meadow**: 48 by 40 metres of grass with a river crossed at a ford in
//! its east edge. The herd grazes in the south half; a fighter arrives from
//! the west, and the ground between is open -- so the first decision is which
//! rock to walk toward. What is here, and what covers each (§3):
//!
//! | What | Height | What covers it |
//! | --- | --- | --- |
//! | Four boulders, 2.5 m across | 2.0 m | Out of the stampede (the herd splits round it); the hook reaches the top, and a charge into the rock you stand on shakes you off and cracks it. Two charges and it is rubble. They are hazard cells that stand as solids (`species::hornback::rules`), not this table, because they break |
//! | The bank, along the north | 1.5 m, 4 m deep | Out of the stampede; the bull will not charge somebody up there. Stepped ramps at both ends |
//! | The edge: a thicket | 6 m | A slope and a thicket the charge pulls up short of rather than meets, and nobody hops |
//! | The ford, east | the floor | Where the herd leaves: a cow that bolts, and the herd when the bull is dead |
//!
//! **The crossing** (the defend variant): a longer meadow, 80 by 36, with a
//! road along it a cart is escorted down, and the herd's migration crossing
//! the road in waves (`--hunt hornback --arena crossing`).

use super::{Area, Arena, ArenaId, Bounds, HuntMarks, Mark, Material, Region, Solid, Spawns};
use crate::objective::Site;
use crate::species::SpeciesId;

use Material::{Grass, Ground, Rock, Water, Wood};

pub static ARENA: Arena = Arena {
    id: ArenaId::HORNBACK,
    name: "Low meadow",
    creature: Some(SpeciesId::HORNBACK),
    bounds: Bounds::cm((-2400, 2400), (-2000, 2000)),
    floor: Grass,
    regions: &REGIONS,
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-500, 0, (500, 0)), Mark::cm(500, 0, (-500, 0))],
        hunt: Some(HuntMarks {
            // Walking in from the west edge.
            hunters: [
                Mark::cm(-2000, -200, (0, -200)),
                Mark::cm(-2000, 200, (0, 200)),
            ],
            // The herd's grazing ground in the south half, facing the way the
            // hunters come: where it musters is home (`pack::Pack::home`).
            creatures: [
                Mark::cm(900, -1000, (-2000, -200)),
                Mark::cm(900, -1000, (-2000, -200)),
            ],
        }),
    },
    sites: &[],
};

const REGIONS: [Region; 4] = [
    // The ford: shallow water at the east edge where the herd crosses.
    Region {
        area: Area::rect_cm((2050, 2400), (-1400, -600)),
        material: Water,
    },
    // Trodden earth where the herd grazes, and the way down to the ford.
    Region {
        area: Area::disc_cm(900, -1000, 500),
        material: Ground,
    },
    Region {
        area: Area::rect_cm((1300, 2050), (-1200, -800)),
        material: Ground,
    },
    // The bank's foot is stony.
    Region {
        area: Area::rect_cm((-2000, 2000), (1500, 1600)),
        material: Rock,
    },
];

const SOLIDS: [Solid; 10] = [
    // The bank along the north side: a metre and a half high, four deep.
    Solid::cm([-1600, 0, 1600], [1600, 150, 2000], Rock),
    // Its ramps at both ends: two steps a hop each, a metre and half a metre.
    Solid::cm([-2000, 0, 1600], [-1600, 100, 2000], Rock),
    Solid::cm([-2400, 0, 1600], [-2000, 50, 2000], Rock),
    Solid::cm([1600, 0, 1600], [2000, 100, 2000], Rock),
    Solid::cm([2000, 0, 1600], [2400, 50, 2000], Rock),
    // The edge: a slope up into thicket, six metres -- nobody hops it, and
    // the charge pulls up short of it.
    Solid::cm([-2600, 0, -2200], [-2400, 600, 2200], Grass),
    Solid::cm([2400, 0, -2200], [2600, 600, 2200], Grass),
    Solid::cm([-2600, 0, -2200], [2600, 600, -2000], Grass),
    Solid::cm([-2600, 0, 2000], [2600, 600, 2200], Grass),
    // A fallen trunk by the ford: a low step, not a wall.
    Solid::cm([1500, 0, -1700], [1900, 40, -1550], Wood),
];

/// **The crossing**: the cart's road runs west to east along the middle of a
/// longer meadow; the herd migrates across it from the north, in waves, and
/// grazes in the south. Three boulders by the road.
pub static CROSSING: Arena = Arena {
    id: ArenaId::HORNBACK_CROSSING,
    name: "Crossing",
    creature: Some(SpeciesId::HORNBACK),
    bounds: Bounds::cm((-4000, 4000), (-1800, 1800)),
    floor: Grass,
    regions: &CROSSING_REGIONS,
    solids: &CROSSING_SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-500, 0, (500, 0)), Mark::cm(500, 0, (-500, 0))],
        hunt: Some(HuntMarks {
            // Beside the cart at the west end of the road.
            hunters: [
                Mark::cm(-3400, -300, (0, -300)),
                Mark::cm(-3400, 300, (0, 300)),
            ],
            // The herd's ground, north of the road and ahead.
            creatures: [
                Mark::cm(400, 1100, (-3400, 0)),
                Mark::cm(400, 1100, (-3400, 0)),
            ],
        }),
    },
    sites: &CROSSING_SITES,
};

/// The road: the cart starts at its west end and wins at its east.
const CROSSING_SITES: [Site; 1] = [Site {
    name: "road",
    route: &[(-3600, 0), (3600, 0)],
    size: (110, 75, 160),
    material: Wood,
}];

const CROSSING_REGIONS: [Region; 2] = [
    // The road itself.
    Region {
        area: Area::rect_cm((-4000, 4000), (-150, 150)),
        material: Ground,
    },
    // The ford the road crosses at its east end.
    Region {
        area: Area::rect_cm((3200, 4000), (-400, 400)),
        material: Water,
    },
];

const CROSSING_SOLIDS: [Solid; 4] = [
    Solid::cm([-4200, 0, -2000], [-4000, 600, 2000], Grass),
    Solid::cm([4000, 0, -2000], [4200, 600, 2000], Grass),
    Solid::cm([-4200, 0, -2000], [4200, 600, -1800], Grass),
    Solid::cm([-4200, 0, 1800], [4200, 600, 2000], Grass),
];
