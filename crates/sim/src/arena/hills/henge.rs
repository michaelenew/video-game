//! **The Henge**, a King of the Hill map (`super`, `crate::versus`,
//! `docs/design/versus.md` §3, `docs/design/fight-spaces.md`).

#![allow(unused_imports)]

use super::super::relief::Bump;
use super::super::rim::Rim;
use super::super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns};
use super::{round, slab, whole};
use crate::versus::{Hill, Kind, Stage, Way};

use Material::{Ash, Grass, Ground, Peat, Rock, Sand, Snow, Stone, Water, Wood};

/// **The Henge**: a giant's stone circle on a grass down. Its great stones
/// and lintels are rough rock, not dressed masonry: drawn as masonry, block by
/// block, seven-metre stones were most of the map's cost to draw. Four standing
/// stones seven metres tall stand nine metres out from the point, north,
/// south, east and west, and **two lintels** cross between them over it in
/// a cross, four body heights up: a walkway over the point from every side,
/// to drop onto it from or to be knocked off. Each stone is climbed by a
/// stair of three blocks on its outer face; the west stone is the west
/// side's, the east the east side's, the north and south nobody's. Out by
/// each start is **a long barrow**, an earth mound with a passage through it
/// that comes out beside the south (or north) stone: the way in nobody sees.
/// A ring of smaller stones stands about the circle for cover.
pub static HENGE: Arena = Arena {
    id: ArenaId::HENGE,
    name: "Henge",
    creature: None,
    bounds: Bounds::cm((-3800, 3800), (-2800, 2800)),
    floor: Grass,
    regions: &HENGE_REGIONS,
    solids: &HENGE_SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-3400, 0, (0, 0)), Mark::cm(3400, 0, (0, 0))],
        hunt: None,
    },
    sites: &[],
    rim: Some(Rim::new((-3800, 3800), (-2800, 2800), [0, 0, 0, 0], 900)),
};

pub static HENGE_STAGE: Stage = Stage {
    arena: ArenaId::HENGE,
    kind: Kind::Hill,
    theme: "A giant's stone circle: lintels crossing high over the point",
    pit: None,
    // Room up to just under the lintels' tops, 6.55 m over the altar:
    // anybody in the air under them counts, nobody standing on them.
    hill: Some(Hill::cm(0, 45, 0, 400, 50, 640)),
    vents: &[],
    drifts: &[],
    ways: &HENGE_WAYS,
};

const HENGE_REGIONS: [Region; 3] = [
    // Chalk worn bare inside the circle.
    Region {
        area: Area::disc_cm(0, 0, 800),
        material: Sand,
    },
    // Each start, trodden.
    Region {
        area: Area::disc_cm(-3400, 0, 450),
        material: Rock,
    },
    Region {
        area: Area::disc_cm(3400, 0, 450),
        material: Rock,
    },
];

const HENGE_SOLIDS: [Solid; 32] = whole(
    [
        // The altar stone the point is on, a step up, walked.
        round(0, 0, (0, 45), (500, 450), Stone),
        // The north-south lintel, over the point: its underside 6.3 m up,
        // flush with the north and south stones' tops.
        Solid::cm([-200, 630, -900], [200, 700, 900], Rock),
    ],
    [
        // The west stone, seven metres, and its stair.
        Solid::cm([-1300, 0, -200], [-900, 700, 200], Rock),
        Solid::cm([-1700, 0, -200], [-1300, 230, 200], Rock),
        Solid::cm([-1500, 0, -200], [-1300, 470, 200], Rock),
        // The west half of the east-west lintel, from the west stone to the
        // north-south one.
        Solid::cm([-900, 630, -200], [-200, 700, 200], Rock),
        // The north stone and its stair.
        Solid::cm([-200, 0, 900], [200, 700, 1300], Rock),
        Solid::cm([-200, 0, 1300], [200, 230, 1700], Rock),
        Solid::cm([-200, 0, 1300], [200, 470, 1500], Rock),
        // The west side's long barrow: two banks and the roof of the
        // passage between them, three metres wide, twelve long.
        Solid::cm([-3000, 0, -1600], [-1800, 300, -1450], Ground),
        Solid::cm([-3000, 0, -1150], [-1800, 300, -1000], Ground),
        Solid::cm([-3000, 220, -1450], [-1800, 300, -1150], Ground),
        // The ring of smaller stones.
        round(-1600, 600, (0, 300), (60, 50), Stone),
        round(-1600, -600, (0, 300), (60, 50), Stone),
        round(-600, 1600, (0, 300), (60, 50), Stone),
        round(600, 1600, (0, 300), (60, 50), Stone),
        // A low wall across the west start.
        Solid::cm([-3000, 0, -600], [-2850, 130, 600], Rock),
    ],
);

const HENGE_WAYS: [Way; 3] = [
    Way::new(
        "way 0",
        (-2200, 0, 0),
        &[
            (-1600, 230, 0),
            (-1400, 470, 0),
            (-1100, 700, 0),
            (-500, 700, 0),
            (0, 700, 0),
            (0, 700, 1100),
        ],
    ),
    Way::new(
        "way 1",
        (0, 0, 2200),
        &[(0, 230, 1600), (0, 470, 1400), (0, 700, 1100)],
    ),
    Way::new("way 2", (-3300, 0, -1300), &[(-1500, 0, -1300), (0, 45, 0)]),
];

/// The floor's relief (`arena::relief`): smooth hills and hollows, no boxes.
/// Kept clear of the point, the starts and the rim.
pub const BUMPS: [Bump; 0] = [];
