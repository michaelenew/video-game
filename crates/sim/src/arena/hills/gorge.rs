//! **The Gorge**, a King of the Hill map (`super`, `crate::versus`,
//! `docs/design/versus.md` §3, `docs/design/fight-spaces.md`).

#![allow(unused_imports)]

use super::super::relief::Bump;
use super::super::rim::Rim;
use super::super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns};
use super::{round, slab, whole};
use crate::versus::{Hill, Kind, Stage, Way};

use Material::{Ash, Grass, Ground, Peat, Rock, Sand, Snow, Stone, Water, Wood};

/// **The Gorge**: a river gorge twenty metres wide between cliffs seven
/// metres high, the two sides' starts at its two ends. The point is on a
/// gravel bar in the middle of the river, and **a plank bridge** crosses the
/// gorge straight over it, four body heights up, from the north cliff-top to
/// the south: a walkway to fight down onto the point from, and to be knocked
/// off. Each cliff-top is a long terrace of open high ground, climbed out of
/// the gorge by two ledges at each end of it. Inside the north cliff runs
/// **the cave**, in from a bay by the west side's start and out through a
/// mouth in the cliff beside the point: the way onto the point nobody sees
/// (the south cliff's, turned, is the east side's). The gorge floor is open,
/// with boulders for cover; the river runs down it.
pub static GORGE: Arena = Arena {
    id: ArenaId::GORGE,
    name: "Gorge",
    creature: None,
    bounds: Bounds::cm((-3800, 3800), (-2600, 2600)),
    floor: Rock,
    regions: &GORGE_REGIONS,
    solids: &GORGE_SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-3300, 0, (0, 0)), Mark::cm(3300, 0, (0, 0))],
        hunt: None,
    },
    sites: &[],
    // The bank starts at the cliff-tops' height all round: at the gorge's
    // two ends that is a wall seven metres up from the river, and walking
    // off a cliff-top's back or end is walking onto the bank.
    rim: Some(Rim::new(
        (-3800, 3800),
        (-2600, 2600),
        [700, 700, 700, 700],
        1000,
    )),
};

pub static GORGE_STAGE: Stage = Stage {
    arena: ArenaId::GORGE,
    kind: Kind::Hill,
    theme: "A river gorge: the point on a gravel bar, a plank bridge high over it",
    pit: None,
    // Room up to just under the bridge's planks, 6.55 m over the bar:
    // anybody in the air under them counts, nobody standing on them.
    hill: Some(Hill::cm(0, 45, 0, 400, 50, 640)),
    vents: &[],
    drifts: &[],
    ways: &GORGE_WAYS,
};

const GORGE_REGIONS: [Region; 3] = [
    // The river, either side of the bar.
    Region {
        area: Area::rect_cm((-3800, -600), (-150, 150)),
        material: Water,
    },
    Region {
        area: Area::rect_cm((600, 3800), (-150, 150)),
        material: Water,
    },
    // The gravel bar.
    Region {
        area: Area::disc_cm(0, 0, 650),
        material: Sand,
    },
];

const GORGE_SOLIDS: [Solid; 30] = whole(
    [
        // The bar the point is on, a step up, walked.
        round(0, 0, (0, 45), (500, 450), Sand),
        // The bridge, straight over the point: planks four metres wide,
        // their underside 6.3 m over the bar, flush with both cliff-tops.
        Solid::cm([-200, 630, -1000], [200, 700, 1000], Wood),
    ],
    [
        // The north cliff, seven metres: its face west of the cave's mouth,
        // the mouth's lintel, the cave's roof, the cliff east of the cave,
        // and its back to the rim. West of the face, a bay by the west start
        // that the cave runs in from.
        Solid::cm([-3000, 0, 1000], [-300, 700, 1300], Rock),
        Solid::cm([-300, 300, 1000], [0, 700, 1300], Rock),
        Solid::cm([-3000, 300, 1300], [0, 700, 1600], Rock),
        Solid::cm([0, 0, 1000], [3800, 700, 1600], Rock),
        Solid::cm([-3800, 0, 1600], [3800, 700, 2600], Rock),
        // Two ledges up the north cliff's face at its west end, and two at
        // its east end.
        Solid::cm([-2600, 0, 700], [-2000, 230, 1000], Rock),
        Solid::cm([-2600, 0, 850], [-2000, 470, 1000], Rock),
        Solid::cm([1200, 0, 700], [1800, 230, 1000], Rock),
        Solid::cm([1200, 0, 850], [1800, 470, 1000], Rock),
        // Boulders in the gorge.
        round(-1800, -450, (0, 160), (120, 90), Rock),
        round(-2800, 550, (0, 120), (100, 80), Rock),
        Solid::cm([-1200, 0, 300], [-900, 160, 700], Rock),
        // Boulders on the north cliff-top.
        round(-1500, 1900, (700, 900), (120, 90), Rock),
        round(800, 2100, (700, 880), (100, 80), Rock),
    ],
);

const GORGE_WAYS: [Way; 4] = [
    Way::new("way 0", (-2800, 0, 0), &[(0, 45, 0)]),
    Way::new(
        "way 1",
        (-2300, 0, 400),
        &[(-2300, 230, 780), (-2300, 700, 1400)],
    ),
    Way::new(
        "way 2",
        (1500, 0, 400),
        &[
            (1500, 230, 780),
            (1500, 700, 1400),
            (0, 700, 1200),
            (0, 700, 0),
            (0, 700, -1300),
        ],
    ),
    Way::new(
        "way 3",
        (-3400, 0, 700),
        &[
            (-3400, 0, 1450),
            (-150, 0, 1450),
            (-150, 0, 700),
            (0, 45, 200),
        ],
    ),
];

/// The floor's relief (`arena::relief`): smooth hills and hollows, no boxes.
/// Kept clear of the point, the starts and the rim.
pub const BUMPS: [Bump; 0] = [];

// The tables a stretched map is made from (`super::sizes`).
pub(super) const SOLIDS: [Solid; 30] = GORGE_SOLIDS;
pub(super) const REGIONS: [Region; 3] = GORGE_REGIONS;
pub(super) const VENTS: [crate::valley::Vent; 0] = [];
pub(super) const DRIFTS: [crate::drift::Drift; 0] = [];
