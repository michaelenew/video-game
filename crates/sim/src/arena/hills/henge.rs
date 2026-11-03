//! **The Henge**, a King of the Hill map (`super`, `crate::versus`,
//! `docs/design/versus.md` §3, `docs/design/fight-spaces.md`).
//!
//! Rebuilt to the fight-space rules (2026-11-02): 128 by 92 m, the starts 58 m
//! out, three levels -- the down, a ring road of lintels at seven metres and
//! two giant trilithons at fifteen -- with ley-line updrafts to ride, and
//! cover, high ground and a long barrow out past a reach, so the approach is
//! a choice.

#![allow(unused_imports)]

use super::super::relief::Bump;
use super::super::rim::Rim;
use super::super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns, cm};
use super::{round, slab, whole};
use crate::valley::Vent;
use crate::versus::{Hill, Kind, Stage, Way};

use Material::{Ash, Grass, Ground, Peat, Rock, Sand, Snow, Stone, Water, Wood};

/// **The Henge**: a giant's stone circle on a rolling grass down. Its stones
/// are rough rock, not dressed masonry.
///
/// **The circle** is twenty metres across the point: six great stones seven
/// metres tall -- west, north-west and south-west for one side, their turned
/// copies for the other -- carry **a ring road of lintels** at seven metres,
/// four metres wide, with eight-metre gaps under it to walk through. North
/// and south the ring is broken by **two giant trilithons**, neutral: two
/// fifteen-metre uprights a six-metre gate apart, a lintel across them, and
/// a top fourteen by six, the highest ground on the map. The ring road is
/// two arcs, one a side, each ending at the giants' uprights: at one end
/// two corbels jut from the giant's face as a stair to its top, at the other
/// a **ley-line updraft** rises off the lintel along the giant's face. **One
/// long lintel** crosses over the point from the west stone to the east, the
/// one overpass: a high road straight onto the point, to drop from or be
/// knocked off.
///
/// **Ways up**: each side's west stone is climbed by a stair of two blocks on
/// its outer face; the north-west and south-west stones by an updraft
/// against their outer faces; the giants' tops from the ring road by the
/// corbels or the ley line, or straight from the down by the **ley line on
/// each giant's outer face**, fifteen metres in a breath, in the open.
///
/// **Out past a reach**: an avenue of standing stones from each start to its
/// stair, an outer ring of smaller stones for cover, **a long barrow** on
/// each side's flank with a passage four metres tall through it -- the way
/// in nobody sees, coming out by the south-west gap -- and its roof a perch,
/// climbed off a kerb stone; and on the other flank **a dolmen**, a capstone
/// on two uprights, a chamber under it and an updraft and a step up onto it.
/// The down rolls in long swells between them.
pub static HENGE: Arena = Arena {
    id: ArenaId::HENGE,
    name: "Henge",
    creature: None,
    bounds: Bounds::cm((-6400, 6400), (-4600, 4600)),
    floor: Grass,
    regions: &HENGE_REGIONS,
    solids: &HENGE_SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-5800, 0, (0, 0)), Mark::cm(5800, 0, (0, 0))],
        hunt: None,
    },
    sites: &[],
    rim: Some(Rim::new((-6400, 6400), (-4600, 4600), [0, 0, 0, 0], 900)),
};

pub static HENGE_STAGE: Stage = Stage {
    arena: ArenaId::HENGE,
    kind: Kind::Hill,
    theme: "A giant's stone circle: a ring road of lintels, two trilithons over it",
    pit: None,
    // Room up to just under the long lintel's top, 6.7 m over the altar:
    // anybody in the air under it counts, nobody standing on it.
    hill: Some(Hill::cm(0, 12, 0, 400, 50, 670)),
    vents: &HENGE_VENTS,
    drifts: &[],
    ways: &HENGE_WAYS,
};

const HENGE_REGIONS: [Region; 5] = [
    // Chalk worn bare inside the circle.
    Region {
        area: Area::disc_cm(0, 0, 1500),
        material: Sand,
    },
    // Each start, trodden.
    Region {
        area: Area::disc_cm(-5800, 0, 450),
        material: Rock,
    },
    Region {
        area: Area::disc_cm(5800, 0, 450),
        material: Rock,
    },
    // The barrows' passages, earth underfoot.
    Region {
        area: Area::rect_cm((-4800, -1600), (-2700, -2300)),
        material: Peat,
    },
    Region {
        area: Area::rect_cm((1600, 4800), (2300, 2700)),
        material: Peat,
    },
];

/// The ring road's lintels, and the long lintel over the point: 6.3 to 7 m.
const LINTEL: (i32, i32) = (630, 700);

/// The giants' lintel tops: the top level.
const GIANT_TOP: i32 = 1500;

const HENGE_SOLIDS: [Solid; 62] = whole(
    [
        // The altar stone the point is on: a slab worn nearly flush with the
        // chalk, walked onto without a hop.
        round(0, 0, (0, 12), (500, 470), Stone),
        // The long lintel, from the west stone to the east over the point:
        // the one overpass, its underside 6.3 m up.
        Solid::cm([-1800, LINTEL.0, -200], [1800, LINTEL.1, 200], Rock),
    ],
    [
        // -- The circle: the west side's three great stones, seven metres,
        // twenty out from the point. Their tops are the ring road's.
        Solid::cm([-2200, 0, -225], [-1800, 700, 225], Rock),
        slab(-1576, 1231, (0, 700), (225, 200), (11, 14), Rock),
        slab(-1576, -1231, (0, 700), (225, 200), (11, -14), Rock),
        // The ring road's west arc: four lintels, west to north-west to the
        // north giant, west to south-west to the south giant.
        slab(-1788, 616, LINTEL, (651, 200), (10, 29), Rock),
        slab(-1038, 1616, LINTEL, (661, 200), (7, 5), Rock),
        slab(-1788, -616, LINTEL, (651, 200), (10, -29), Rock),
        slab(-1038, -1616, LINTEL, (661, 200), (7, -5), Rock),
        // The north giant: two uprights a six-metre gate apart and the
        // lintel between them, the top level. (Its turned copy is the south
        // giant.) The uprights run up to the top so no seam under the lintel
        // is a ledge to hang from.
        Solid::cm([-700, 0, 1700], [-300, GIANT_TOP, 2300], Rock),
        Solid::cm([300, 0, 1700], [700, GIANT_TOP, 2300], Rock),
        Solid::cm([-300, GIANT_TOP - 140, 1700], [300, GIANT_TOP, 2300], Rock),
        // Two corbels jutting from its west upright where the west arc ends:
        // a stair off the ring road, 2.5 m and 2.7 m, then 2.8 m to the top.
        Solid::cm([-1100, LINTEL.0, 1650], [-700, 950, 2000], Rock),
        Solid::cm([-1050, 1100, 2000], [-700, 1220, 2300], Rock),
        // The stair up the west stone's outer face.
        Solid::cm([-2600, 0, -225], [-2200, 470, 225], Rock),
        Solid::cm([-3000, 0, -225], [-2600, 230, 225], Rock),
        // A stone fallen by the point: a little cover on it.
        slab(-550, 650, (0, 120), (200, 70), (1, -1), Stone),
        // -- Out past a reach. The avenue from the start to the stair.
        Solid::cm([-4680, 0, 780], [-4520, 450, 1020], Rock),
        Solid::cm([-4680, 0, -1020], [-4520, 450, -780], Rock),
        Solid::cm([-3780, 0, 780], [-3620, 450, 1020], Rock),
        Solid::cm([-3780, 0, -1020], [-3620, 450, -780], Rock),
        // The outer ring of smaller stones, thirty-one metres out.
        round(-2685, 1550, (0, 500), (110, 90), Stone),
        round(-1310, 2810, (0, 500), (110, 90), Stone),
        round(-2810, -1310, (0, 500), (110, 90), Stone),
        // The long barrow: two banks and the roof between them over the
        // passage, four metres wide, 4.4 m tall and thirty-two long, one
        // top 5.2 m up; and the kerb stone at its west end, a step up to it.
        // (The banks run up to the roof's top: a seam under it would be a
        // ledge to hang from, as would one under the dolmen's capstone.)
        Solid::cm([-4800, 0, -2950], [-1600, 520, -2700], Ground),
        Solid::cm([-4800, 0, -2300], [-1600, 520, -2050], Ground),
        Solid::cm([-4800, 440, -2700], [-1600, 520, -2300], Ground),
        Solid::cm([-5300, 0, -2950], [-4800, 260, -2700], Stone),
        // The dolmen: two uprights, a chamber between them, the capstone,
        // and a propped stone to step up by.
        Solid::cm([-4300, 0, 2700], [-4000, 560, 3300], Rock),
        Solid::cm([-3600, 0, 2700], [-3300, 560, 3300], Rock),
        Solid::cm([-4000, 470, 2700], [-3600, 560, 3300], Rock),
        Solid::cm([-4800, 0, 2800], [-4300, 280, 3200], Stone),
    ],
);

/// **The updrafts**, both sides' (they are not turned for you): the ley
/// lines up the giants, the updraft against each of the outer great stones,
/// and the dolmen's.
const HENGE_VENTS: [Vent; 10] = [
    // The ley lines off the ring road, up the giant's face where an arc
    // ends without corbels.
    ley(-860, -1840),
    ley(860, 1840),
    // The ley lines on the giants' outer faces, from the down to the top.
    Vent::cm(-500, 2480, 170, GIANT_TOP + 50),
    Vent::cm(500, -2480, 170, GIANT_TOP + 50),
    // Against the north-west and south-west stones' outer faces, and the
    // turned copies'.
    Vent::cm(-1875, 1465, 160, 760),
    Vent::cm(1875, -1465, 160, 760),
    Vent::cm(-1875, -1465, 160, 760),
    Vent::cm(1875, 1465, 160, 760),
    // Against the dolmen's south face.
    Vent::cm(-3450, 2540, 160, 620),
    Vent::cm(3450, -2540, 160, 620),
];

/// A ley line at (x, z): rising off the ring road's lintel, let go half a
/// metre over a giant's top.
const fn ley(x: i32, z: i32) -> Vent {
    Vent {
        at: (cm(x), cm(z)),
        radius: cm(170),
        top: cm(GIANT_TOP + 50),
        from: cm(LINTEL.1),
    }
}

/// The west side's ways (the east side's are their turned copies).
const HENGE_WAYS: [Way; 9] = [
    Way::new(
        "the open floor: down the avenue, round the stair, through the west-south-west gap",
        (-5800, 0, 0),
        &[
            (-4100, 0, 0),
            (-3300, 0, -800),
            (-1890, 0, -650),
            (-100, 12, -300),
        ],
    ),
    Way::new(
        "the high road: up the west stone's stair, along the long lintel, down onto the point",
        (-5800, 0, 0),
        &[
            (-3400, 0, 0),
            (-2800, 230, 0),
            (-2400, 470, 0),
            (-2000, 700, 0),
            (-800, 700, 0),
            (-100, 12, 360),
        ],
    ),
    Way::new(
        "the hidden way: through the long barrow, in by the south-west gap",
        (-5800, 0, 0),
        &[
            (-5300, 0, -2500),
            (-1500, 0, -2500),
            (-1118, 0, -1658),
            (-100, 12, -300),
        ],
    ),
    Way::new(
        "the ring road: up the north-west stone's updraft, round to the north giant, up its corbels",
        (-2700, 0, 2100),
        &[
            (-1576, 700, 1231),
            (-1200, 700, 1500),
            (-900, 950, 1825),
            (-875, 1220, 2150),
            (-500, GIANT_TOP, 2000),
        ],
    ),
    Way::new(
        "the south giant: up the south-west stone's updraft, round, up the ley line",
        (-2500, 0, -1700),
        &[
            (-1576, 700, -1231),
            (-1038, 700, -1616),
            (-900, 700, -1820),
            (-500, GIANT_TOP, -2000),
        ],
    ),
    Way::new(
        "the north giant's outer ley line, from the down to its top",
        (-500, 0, 3100),
        &[(-500, GIANT_TOP, 2000)],
    ),
    Way::new(
        "the dolmen by its updraft",
        (-3450, 0, 2200),
        &[(-3450, 560, 3000)],
    ),
    Way::new(
        "the dolmen by its propped stone",
        (-5200, 0, 3000),
        &[(-4550, 280, 3000), (-3800, 560, 3000)],
    ),
    Way::new(
        "the barrow's roof by its kerb stone",
        (-5600, 0, -2825),
        &[(-5050, 260, -2825), (-4400, 520, -2825)],
    ),
];

/// The floor's relief (`arena::relief`): the down's long swells and a dell,
/// each listed with its turned copy. Kept clear of the point, the starts,
/// the rim, the barrows' passages, the stairs and the capstones' feet.
pub const BUMPS: [Bump; 12] = [
    // A swell north of the avenue.
    Bump::cm(-4400, 1300, 900, 200),
    Bump::cm(4400, -1300, 900, 200),
    // A low mound inside the circle, between the west and north-west stones.
    Bump::cm(-1250, 800, 550, 90),
    Bump::cm(1250, -800, 550, 90),
    // The south flank's long swell, past the barrow's mouth.
    Bump::cm(-800, -3600, 900, 250),
    Bump::cm(800, 3600, 900, 250),
    // A dell past the dolmen.
    Bump::cm(-2300, 3700, 700, -150),
    Bump::cm(2300, -3700, 700, -150),
    // A knoll in the corner behind the barrow.
    Bump::cm(-5400, -3800, 700, 250),
    Bump::cm(5400, 3800, 700, 250),
    // A swell behind the north giant.
    Bump::cm(-1500, 3500, 900, 200),
    Bump::cm(1500, -3500, 900, 200),
];

// The tables a stretched map is made from (`super::sizes`).
pub(super) const SOLIDS: [Solid; 62] = HENGE_SOLIDS;
pub(super) const REGIONS: [Region; 5] = HENGE_REGIONS;
pub(super) const VENTS: [crate::valley::Vent; 10] = HENGE_VENTS;
pub(super) const DRIFTS: [crate::drift::Drift; 0] = [];
