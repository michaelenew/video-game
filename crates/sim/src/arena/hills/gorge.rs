//! **The Gorge**, a King of the Hill map (`super`, `crate::versus`,
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

/// **The Gorge** (rebuilt 2026-11-02 to `docs/design/fight-spaces.md`): a
/// river canyon 128 metres long, fought up and down three levels.
///
/// - **The floor**: the gorge bottom, 28 m wide, the river down its middle
///   and the two sides' starts at its two ends. The point is on a gravel bar
///   in the river. West of it the river is stood in by **the stacks**, three
///   pillars of rock climbing 2.4, 4.2 and 5.6 m, a perch line to shoot at
///   the point from; south of it a big boulder and a smaller one are the
///   floor's cover.
/// - **The terraces**, 7 m up: a shelf 14 m deep along each cliff. **The
///   bridge** crosses from one to the other straight over the point, the
///   one thing over it; **the arch**, a span of rock at the same height,
///   crosses again 28 m out on each side's own end, out of the point's
///   reach. Each terrace is climbed from the floor by a stair of blocks at
///   one end, by **the hoist**, a timber crib that climbs out of the shingle
///   12 m from the point to the lip and sinks back, and (the north one; the
///   south one's, turned, is the east side's) by the cave's skylight.
/// - **The cliff-tops**, 15 m up and 28 m out: the high ground past a reach,
///   with a tor on each for cover. Each is climbed by **a thermal** off the
///   floor of the bay by one side's start, by a second thermal up the
///   cliff from the terrace, and by a stair of blocks up the cliff from the
///   terrace at its other end. **The cable car** crosses between the two cliff-tops at each
///   side's own end, high over the gorge.
///
/// Inside the north terrace runs **the cave**: in from the bay by the west
/// side's start, along under the terrace -- with **a skylight** in its roof
/// to drop through, and an updraft in it to ride back out -- and out of a
/// mouth in the terrace's face beside the point (the south terrace's, turned,
/// is the east side's).
pub static GORGE: Arena = Arena {
    id: ArenaId::GORGE,
    name: "Gorge",
    creature: None,
    bounds: Bounds::cm((-6400, 6400), (-4600, 4600)),
    floor: Rock,
    regions: &GORGE_REGIONS,
    solids: &GORGE_SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-5800, 0, (0, 0)), Mark::cm(5800, 0, (0, 0))],
        hunt: None,
    },
    sites: &[],
    // The bank starts at the cliff-tops' height all round: at the gorge's
    // two ends that is a wall fifteen metres up from the river, and walking
    // off a cliff-top's back is walking onto the bank.
    rim: Some(Rim::new(
        (-6400, 6400),
        (-4600, 4600),
        [1500, 1500, 1500, 1500],
        1000,
    )),
};

pub static GORGE_STAGE: Stage = Stage {
    arena: ArenaId::GORGE,
    kind: Kind::Hill,
    theme: "A river canyon in three levels: the point on a gravel bar, a bridge over it, thermals up the cliffs",
    pit: None,
    // Room up to just under the bridge's planks, 6.4 m over the bar:
    // anybody in the air under them counts, nobody standing on them.
    hill: Some(Hill::cm(0, 12, 0, 400, 50, 670)),
    front: None,
    vents: &GORGE_VENTS,
    drifts: &GORGE_DRIFTS,
    ways: &GORGE_WAYS,
};

const GORGE_REGIONS: [Region; 3] = [
    // The river, either side of the bar.
    Region {
        area: Area::rect_cm((-6400, -600), (-150, 150)),
        material: Water,
    },
    Region {
        area: Area::rect_cm((600, 6400), (-150, 150)),
        material: Water,
    },
    // The gravel bar.
    Region {
        area: Area::disc_cm(0, 0, 650),
        material: Sand,
    },
];

const GORGE_SOLIDS: [Solid; 48] = whole(
    [
        // The bar the point is on, barely proud of the river: a step a
        // hop would take under the bridge catches its edge.
        round(0, 0, (0, 12), (500, 450), Sand),
        // The bridge, straight over the point from terrace to terrace:
        // planks four metres wide, their underside 6.3 m over the floor,
        // flush with both terraces.
        Solid::cm([-200, 630, -1400], [200, 700, 1400], Wood),
    ],
    [
        // The north cliff above the terrace, to the rim: the cliff-top.
        Solid::cm([-6400, 0, 2800], [6400, 1500, 4600], Rock),
        // The north terrace west of the point, with the cave inside it: its
        // face (the cave's south wall), the cave's roof either side of the
        // skylight, the lintel over the cave's mouth, and the cave's back.
        // West of it, the bay by the west start that the cave runs in from.
        Solid::cm([-4400, 0, 1400], [-1400, 700, 1800], Rock),
        Solid::cm([-4400, 450, 1800], [-2200, 700, 2300], Rock),
        Solid::cm([-1800, 450, 1800], [-900, 700, 2300], Rock),
        Solid::cm([-1400, 450, 1400], [-900, 700, 1800], Rock),
        Solid::cm([-4400, 0, 2300], [-900, 700, 2800], Rock),
        // The north terrace from the cave's end east, to the rim: the part
        // over the point and the far part.
        Solid::cm([-900, 0, 1400], [1800, 700, 2800], Rock),
        Solid::cm([1800, 0, 1400], [6400, 700, 2800], Rock),
        // The stair up the north face at the west end: 2.5 m, 4.8 m, the
        // terrace.
        Solid::cm([-4000, 0, 900], [-3400, 250, 1400], Rock),
        Solid::cm([-4000, 0, 1150], [-3400, 480, 1400], Rock),
        // The stair up the cliff from the north terrace, east of the
        // point (the east side's; turned, the west side's up the south
        // cliff): 9.6 m, 12.3 m, the top.
        Solid::cm([2400, 0, 2000], [3000, 960, 2800], Rock),
        Solid::cm([2400, 0, 2400], [3000, 1230, 2800], Rock),
        // The arch: a span of rock across the gorge, terrace to terrace,
        // out of the point's reach.
        Solid::cm([-3000, 550, -1400], [-2600, 700, 1400], Rock),
        // The floor's cover south of the river: a big boulder and a
        // smaller one beside it.
        round(-1200, -800, (0, 480), (300, 240), Rock),
        round(-600, -1150, (0, 320), (220, 180), Rock),
        // The stacks, pillars standing out of the river west of the
        // point, each a hop over the last.
        round(-1600, 600, (0, 240), (150, 130), Rock),
        round(-2050, 950, (0, 420), (150, 130), Rock),
        round(-2350, 350, (0, 560), (150, 130), Rock),
        // A rock on the terrace over the point, to fight round.
        round(400, 2100, (700, 950), (180, 150), Rock),
        // A boulder at the bar's edge: a little cover on the point.
        round(-600, 350, (0, 140), (120, 100), Rock),
        // The tor on the cliff-top.
        round(-3000, 3700, (1500, 1850), (250, 200), Rock),
        // Boulders on the approach, out of reach of the point.
        round(-4200, -800, (0, 420), (280, 220), Rock),
        round(-3100, 650, (0, 360), (250, 200), Rock),
    ],
);

/// **The updrafts**, each with its turned copy.
const GORGE_VENTS: [Vent; 6] = [
    // The thermal in the bay, up the cliff from the floor to the top.
    Vent::cm(-5200, 2650, 150, 1650),
    Vent::cm(5200, -2650, 150, 1650),
    // The cave's skylight: from the cave's floor up through the roof.
    Vent::cm(-2000, 1950, 200, 850),
    Vent::cm(2000, -1950, 200, 850),
    // The thermal up the cliff from the terrace to the top.
    Vent {
        at: (cm(-3600), cm(2650)),
        radius: cm(150),
        top: cm(1650),
        from: cm(700),
    },
    Vent {
        at: (cm(3600), cm(-2650)),
        radius: cm(150),
        top: cm(1650),
        from: cm(700),
    },
];

/// The hoist: a timber crib four metres square, flush with the floor at
/// its foot, climbing 7 m to the terrace's lip in front of its face.
const HOIST: Solid = Solid::cm([1600, -700, 1000], [2000, 0, 1400], Wood);

/// The cable car's deck, at its north dock: flush with the cliff-top.
const CAR: Solid = Solid::cm([-4800, 1440, 2850], [-4400, 1500, 3250], Wood);

/// **What moves you**: the hoists and the cable cars, each with its turned
/// copy. (A versus stage's drifts are solid only once `World::terrain`
/// raises them off the valley as well as on it.)
const GORGE_DRIFTS: [Drift; 4] = [
    // Two seconds at the foot, two and a half up, two at the lip, two and
    // a half down.
    Drift {
        solid: HOIST,
        travel: V3::new(cm(0), cm(700), cm(0)),
        wait: 120,
        go: 150,
        stay: 120,
        back: 150,
        phase: 0,
    },
    Drift {
        solid: turned(HOIST),
        travel: V3::new(cm(0), cm(700), cm(0)),
        wait: 120,
        go: 150,
        stay: 120,
        back: 150,
        phase: 0,
    },
    // Four seconds at each dock, eight across.
    Drift {
        solid: CAR,
        travel: V3::new(cm(0), cm(0), cm(-6100)),
        wait: 240,
        go: 480,
        stay: 240,
        back: 480,
        phase: 0,
    },
    Drift {
        solid: turned(CAR),
        travel: V3::new(cm(0), cm(0), cm(6100)),
        wait: 240,
        go: 480,
        stay: 240,
        back: 480,
        phase: 0,
    },
];

/// The west side's ways; the east side's are these turned.
const GORGE_WAYS: [Way; 11] = [
    Way::new(
        "out of the core on the hoist, up to the south terrace",
        (-100, 12, 0),
        &[(-1800, 0, -700), (-1800, 0, -1200), (-1800, 700, -1700)],
    ),
    Way::new(
        "the open floor, down the river onto the point",
        (-5800, 0, 0),
        &[(-100, 12, 0)],
    ),
    Way::new(
        "the stair up the north face to the terrace",
        (-3700, 0, 500),
        &[(-3700, 700, 1600)],
    ),
    Way::new(
        "the high road: along the terrace, out on the bridge, down onto the point",
        (-3700, 700, 1600),
        &[(0, 700, 1700), (0, 700, 500), (-300, 12, 0)],
    ),
    Way::new(
        "the cave: in from the bay, out of its mouth onto the point",
        (-5000, 0, 2200),
        &[
            (-1150, 0, 2200),
            (-1150, 0, 1000),
            (-700, 0, 0),
            (-320, 12, 0),
        ],
    ),
    Way::new(
        "off the cliff-top, down to the terrace, over the bridge onto the point",
        (-600, 1500, 3200),
        &[(-300, 700, 2200), (0, 700, 500), (-300, 12, 0)],
    ),
    Way::new(
        "out of the core by the cave's mouth, up through its skylight",
        (-100, 12, 0),
        &[(-1150, 0, 1000), (-1150, 0, 2200), (-2000, 700, 1600)],
    ),
    Way::new(
        "the thermal in the bay, up the cliff to the top",
        (-5200, 0, 2200),
        &[(-5200, 1500, 3200)],
    ),
    Way::new(
        "the thermal up the cliff from the terrace to the top",
        (-3600, 700, 2500),
        &[(-3600, 1500, 3200)],
    ),
    Way::new(
        "the stair up the south cliff from the terrace to the top",
        (-2700, 700, -1700),
        &[(-2700, 1500, -3200)],
    ),
    Way::new(
        "the skylight: up out of the cave on its updraft",
        (-2400, 0, 2100),
        &[(-2000, 700, 1600)],
    ),
];

/// The floor's relief (`arena::relief`): gravel banks on the gorge floor,
/// clear of the point, the starts, the river, the hoists and the rim.
pub const BUMPS: [Bump; 6] = [
    Bump::cm(-5000, -900, 450, 120),
    Bump::cm(5000, 900, 450, 120),
    Bump::cm(-5200, 900, 400, 100),
    Bump::cm(5200, -900, 400, 100),
    Bump::cm(-2800, -1000, 350, 100),
    Bump::cm(2800, 1000, 350, 100),
];

// The tables a stretched map is made from (`super::sizes`).
pub(super) const SOLIDS: [Solid; 48] = GORGE_SOLIDS;
pub(super) const REGIONS: [Region; 3] = GORGE_REGIONS;
pub(super) const VENTS: [crate::valley::Vent; 6] = GORGE_VENTS;
pub(super) const DRIFTS: [crate::drift::Drift; 4] = GORGE_DRIFTS;
