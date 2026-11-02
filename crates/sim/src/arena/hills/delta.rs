//! **The Delta**, a King of the Hill map (`super`, `crate::versus`,
//! `docs/design/versus.md` §3, `docs/design/fight-spaces.md`).

#![allow(unused_imports)]

use super::super::relief::Bump;
use super::super::rim::Rim;
use super::super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns, cm};
use super::{round, slab, turned, whole};
use crate::drift::Drift;
use crate::math::V3;
use crate::valley::Vent;
use crate::versus::{Hill, Kind, Stage, Way};

use Material::{Ash, Grass, Ground, Peat, Rock, Sand, Snow, Stone, Water, Wood};

/// **The Delta** (rebuilt 2026-11-02 to `docs/design/fight-spaces.md`): a
/// river's mouth among reeds, 128 by 92 metres, its waterworks in ruins.
/// The point is on a low island in the middle, ringed by four standing
/// stones four metres tall, under open sky.
///
/// Each side's half (the west's described; the east's is it turned) is
/// crossed eighteen metres from the point by **the dyke**, an earth bank six
/// and a half metres high and six wide -- the map's middle level -- from the
/// north lane to the south, but for **the breach** in its middle, ten metres
/// of open floor straight at the point. Across the breach's start-side mouth
/// stands **the sluice gate**, a timber gate that sinks into the floor and
/// rises out of it on a clock: up, it shuts the breach and bridges the dyke's
/// two lengths; standing on it as it rises is a ride onto the dyke.
///
/// Inside the dyke's south length runs **the sluice**, a covered channel
/// four metres high, fed from the south by **the old lock** -- a trench
/// between a wall as high as the dyke (a causeway onto the dyke's top from
/// the start's flank, climbed by steps at its west end or an updraft in the
/// trench) and a broken one -- so the hidden way onto the point is a walk
/// down the lock and up the channel into the breach.
///
/// Over the north half runs **the aqueduct**, the side's high road,
/// fourteen and a half metres up: from the top of **the water tower** out of
/// reach by the start, on an arch, across **a broken span** three metres
/// wide over the dyke's foot, and on over the dyke and the point's north
/// flank on tall piers to a broken end past the point. The tower is
/// climbed by an updraft up its back and by **the water-lift**, a stone
/// float that rises out of the floor against its face to its top; the
/// aqueduct itself by steps up from the dyke's top, and by
/// **the spout**, an updraft up the face of the pier nearest the point: the
/// way off the point and up out of it. The dyke is climbed by a stair on
/// its start side, an updraft on its point side, the gate, and the lock's
/// wall. Mud banks and reed islands roll the floor; a fallen span is cover
/// in the contest, a low wall at the start.
pub static DELTA: Arena = Arena {
    id: ArenaId::DELTA,
    name: "Delta",
    creature: None,
    bounds: Bounds::cm((-6400, 6400), (-4600, 4600)),
    floor: Grass,
    regions: &DELTA_REGIONS,
    solids: &DELTA_SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-5800, 0, (0, 0)), Mark::cm(5800, 0, (0, 0))],
        hunt: None,
    },
    sites: &[],
    // A tall bank: the aqueduct and the tower are fourteen metres up.
    rim: Some(Rim::new((-6400, 6400), (-4600, 4600), [0, 0, 0, 0], 1200)),
};

pub static DELTA_STAGE: Stage = Stage {
    arena: ArenaId::DELTA,
    kind: Kind::Hill,
    theme: "A river mouth's ruined waterworks: dykes, a sluice gate, a broken aqueduct",
    pit: None,
    hill: Some(Hill::cm(0, 45, 0, 400, 50, 1000)),
    vents: &DELTA_VENTS,
    drifts: &DELTA_DRIFTS,
    ways: &DELTA_WAYS,
};

const DELTA_REGIONS: [Region; 9] = [
    // The river's channels, out past the point to the south and the north.
    Region {
        area: Area::rect_cm((-1300, 1300), (-4600, -3900)),
        material: Water,
    },
    Region {
        area: Area::rect_cm((-1300, 1300), (3900, 4600)),
        material: Water,
    },
    // The old locks' trenches, a little water standing in them.
    Region {
        area: Area::rect_cm((-5000, -2500), (-3700, -2900)),
        material: Water,
    },
    Region {
        area: Area::rect_cm((2500, 5000), (2900, 3700)),
        material: Water,
    },
    // Peat round the point's island and on the reed islands; ash at the
    // starts.
    Region {
        area: Area::disc_cm(0, 0, 650),
        material: Peat,
    },
    Region {
        area: Area::disc_cm(-800, -2600, 500),
        material: Peat,
    },
    Region {
        area: Area::disc_cm(800, 2600, 500),
        material: Peat,
    },
    Region {
        area: Area::disc_cm(-5800, 0, 450),
        material: Ash,
    },
    Region {
        area: Area::disc_cm(5800, 0, 450),
        material: Ash,
    },
];

const DELTA_SOLIDS: [Solid; 49] = whole(
    [
        // The point's island: a low mound, walked up.
        round(0, 0, (0, 45), (500, 450), Grass),
    ],
    [
        // Two of the four standing stones round the point, taller than a
        // jump.
        round(600, 450, (0, 400), (60, 50), Stone),
        round(-600, 450, (0, 400), (60, 50), Stone),
        // The west dyke, six and a half metres high and six wide: its north
        // length, and its south length in three -- two banks and the roof of
        // the sluice between them, three metres wide and four twenty high --
        // with the breach between, and a lane at each end before the rim.
        Solid::cm([-2100, 0, 500], [-1500, 650, 2800], Ground),
        Solid::cm([-2100, 0, -2500], [-1950, 650, -500], Ground),
        Solid::cm([-1650, 0, -2500], [-1500, 650, -500], Ground),
        Solid::cm([-1950, 420, -2500], [-1650, 650, -500], Ground),
        // The dyke's stair on the start side, at its north end.
        Solid::cm([-2900, 0, 2100], [-2100, 220, 2800], Ground),
        Solid::cm([-2500, 0, 2100], [-2100, 440, 2800], Ground),
        // Steps on the dyke's top, along its point-side edge, up to the
        // aqueduct: the last as high as its deck.
        Solid::cm([-1800, 650, 500], [-1500, 850, 1400], Stone),
        Solid::cm([-1800, 650, 800], [-1500, 1050, 1400], Stone),
        Solid::cm([-1800, 650, 1050], [-1500, 1200, 1400], Stone),
        Solid::cm([-1800, 650, 1250], [-1500, 1450, 1400], Stone),
        // The water tower, the aqueduct's head, out of reach by the start.
        Solid::cm([-4600, 0, 1000], [-3800, 1450, 2400], Stone),
        // The aqueduct, fourteen and a half metres up: its first span on an
        // arch off the tower, the broken gap, and the long span over the
        // dyke and the point's north flank to its broken end, on two piers.
        Solid::cm([-3800, 1350, 1400], [-2500, 1450, 2000], Stone),
        Solid::cm([-3000, 0, 1400], [-2600, 1350, 2000], Stone),
        Solid::cm([-2200, 1350, 1400], [600, 1450, 2000], Stone),
        Solid::cm([-1100, 0, 1400], [-700, 1350, 2000], Stone),
        Solid::cm([100, 0, 1400], [500, 1350, 2000], Stone),
        // The old lock: its north wall, as high as the dyke and running
        // into it, a causeway onto its top; its broken south wall; and the
        // steps up the north wall's west end.
        Solid::cm([-5000, 0, -2900], [-1950, 650, -2500], Stone),
        Solid::cm([-5000, 0, -4000], [-2500, 250, -3700], Stone),
        Solid::cm([-5600, 0, -2900], [-5000, 220, -2500], Stone),
        Solid::cm([-5300, 0, -2900], [-5000, 440, -2500], Stone),
        // A fallen span in the contest; a low wall by the start.
        slab(-1000, -1300, (0, 250), (400, 60), (1, 2), Stone),
        Solid::cm([-4900, 0, -1600], [-4700, 130, -400], Rock),
    ],
);

/// **The sluice gate** in the west breach, sunk flush with the floor: it
/// rises six and a half metres to the dyke's top, stays, and sinks.
const GATE: Solid = Solid::cm([-2100, -650, -500], [-1950, 0, 500], Wood);

/// **The water-lift**: a stone float in front of the tower's south face,
/// sunk flush with the floor, rising to the tower's top.
const FLOAT: Solid = Solid::cm([-4350, -1450, 700], [-4050, 0, 1000], Stone);

const DELTA_DRIFTS: [Drift; 4] = [
    // The gates: six seconds down, a second and a half up, three up, two
    // down -- the east's half a cycle behind the west's, so one breach is
    // open while the other is shut.
    Drift {
        solid: GATE,
        travel: V3::new(cm(0), cm(650), cm(0)),
        wait: 360,
        go: 90,
        stay: 180,
        back: 120,
        phase: 0,
    },
    Drift {
        solid: turned(GATE),
        travel: V3::new(cm(0), cm(650), cm(0)),
        wait: 360,
        go: 90,
        stay: 180,
        back: 120,
        phase: 375,
    },
    // The floats: three seconds down, four up, two at the top, three down.
    Drift {
        solid: FLOAT,
        travel: V3::new(cm(0), cm(1450), cm(0)),
        wait: 180,
        go: 240,
        stay: 120,
        back: 180,
        phase: 0,
    },
    Drift {
        solid: turned(FLOAT),
        travel: V3::new(cm(0), cm(1450), cm(0)),
        wait: 180,
        go: 240,
        stay: 120,
        back: 180,
        phase: 360,
    },
];

const DELTA_VENTS: [Vent; 8] = [
    // Up the dyke's point-side face, out over its lip.
    Vent::cm(-1380, 2400, 150, 800),
    Vent::cm(1380, -2400, 150, 800),
    // The spout: up the face of the pier nearest the point, onto the
    // aqueduct.
    Vent::cm(-900, 1220, 200, 1600),
    Vent::cm(900, -1220, 200, 1600),
    // Up the water tower's back, out over its top.
    Vent::cm(-4750, 1700, 150, 1600),
    Vent::cm(4750, -1700, 150, 1600),
    // Out of the lock's trench, up its high wall.
    Vent::cm(-4000, -3030, 150, 800),
    Vent::cm(4000, 3030, 150, 800),
];

/// The west side's ways (the east's are them turned).
const DELTA_WAYS: [Way; 13] = [
    // The rides: the gate and the water-lift, carried up.
    Way::new(
        "the sluice gate: ridden up out of the breach onto the dyke",
        (-2600, 0, 0),
        &[(-2025, 0, 0), (-2025, 650, 800)],
    ),
    Way::new(
        "the water-lift: ridden up the tower's face onto its top",
        (-5800, 0, 0),
        &[(-4200, 0, 850), (-4200, 1450, 1300), (-3000, 1450, 1700)],
    ),
    Way::new(
        "the open floor: from the start through the breach onto the point",
        (-5800, 0, 0),
        &[(-3000, 0, 0), (-1000, 0, 0), (0, 45, 0)],
    ),
    Way::new(
        "the hidden way: down the old lock and up the sluice onto the point",
        (-5800, 0, 0),
        &[
            (-5900, 0, -3300),
            (-2200, 0, -3300),
            (-1800, 0, -3300),
            (-1800, 0, -2700),
            (-1800, 0, -300),
            (0, 45, 0),
        ],
    ),
    Way::new(
        "the high road: up the tower's updraft, along the aqueduct over its broken span, down onto the point",
        (-5800, 0, 0),
        &[
            (-5200, 0, 1700),
            (-4300, 1450, 1700),
            (-2800, 1450, 1700),
            (-1900, 1450, 1700),
            (300, 1450, 1700),
            (0, 45, 0),
        ],
    ),
    Way::new(
        "the north lane: round the dyke's north end onto the point",
        (-5800, 0, 0),
        &[(-4800, 0, 3500), (-300, 0, 3500), (0, 45, 0)],
    ),
    Way::new(
        "onto the dyke by its stair on the start side",
        (-3300, 0, 2450),
        &[(-2700, 220, 2450), (-2300, 440, 2450), (-1800, 650, 2450)],
    ),
    Way::new(
        "onto the dyke from the point side, up the updraft on its face",
        (-1000, 0, 2400),
        &[(-1800, 650, 2400)],
    ),
    Way::new(
        "from the dyke's top up its steps onto the aqueduct",
        (-1950, 650, 650),
        &[
            (-1650, 850, 650),
            (-1650, 1050, 925),
            (-1650, 1200, 1150),
            (-1650, 1450, 1700),
        ],
    ),
    Way::new(
        "out of the core: up the spout and back along the aqueduct to the tower",
        (0, 45, 0),
        &[
            (-900, 0, 1000),
            (-900, 1450, 1700),
            (-1900, 1450, 1700),
            (-2800, 1450, 1700),
            (-4000, 1450, 1700),
        ],
    ),
    Way::new(
        "onto the old lock's wall by its steps",
        (-5800, 0, -1800),
        &[
            (-5450, 220, -2700),
            (-5150, 440, -2700),
            (-4700, 650, -2700),
        ],
    ),
    Way::new(
        "out of the lock's trench up the updraft on its wall",
        (-4000, 0, -3300),
        &[(-4000, 650, -2700)],
    ),
    Way::new(
        "along the lock's wall onto the dyke",
        (-4700, 650, -2700),
        &[(-2100, 650, -2700), (-1800, 650, -1500)],
    ),
];

/// The floor's relief (`arena::relief`): smooth hills and hollows, no boxes.
/// Kept clear of the point, the starts, the rim and the water: a mud bank
/// in the field before each start, a knoll in each north-or-south lane, a
/// reed island by the river, a hummock by the tower.
pub const BUMPS: [Bump; 8] = [
    Bump::cm(-3500, -1200, 900, 200),
    Bump::cm(3500, 1200, 900, 200),
    Bump::cm(-3600, 3600, 900, 250),
    Bump::cm(3600, -3600, 900, 250),
    Bump::cm(-800, -2600, 600, 150),
    Bump::cm(800, 2600, 600, 150),
    Bump::cm(-3000, 700, 500, 120),
    Bump::cm(3000, -700, 500, 120),
];

// The tables a stretched map is made from (`super::sizes`).
pub(super) const SOLIDS: [Solid; 49] = DELTA_SOLIDS;
pub(super) const REGIONS: [Region; 9] = DELTA_REGIONS;
pub(super) const VENTS: [crate::valley::Vent; 8] = DELTA_VENTS;
pub(super) const DRIFTS: [crate::drift::Drift; 4] = DELTA_DRIFTS;
