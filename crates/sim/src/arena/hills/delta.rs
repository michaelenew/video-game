//! **The Delta**, a King of the Hill map (`super`, `crate::versus`,
//! `docs/design/versus.md` §3, `docs/design/fight-spaces.md`).

#![allow(unused_imports)]

use super::super::relief::Bump;
use super::super::rim::Rim;
use super::super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns};
use super::{round, slab, whole};
use crate::versus::{Hill, Kind, Stage, Way};

use Material::{Ash, Grass, Ground, Peat, Rock, Sand, Snow, Stone, Water, Wood};

/// **The Delta**: a river's mouth among reeds, the point on a low island
/// ringed by four standing stones. Each side's half is crossed by **a
/// dyke**, an old earth bank three metres high from rim to rim but for a
/// lane at each end and **the breach** in its middle -- the map's one
/// choke, straight at the point. Its south half hides **the sluice**, a
/// covered channel inside it from the south lane to the breach: the way
/// onto the point nobody sees. Over the north lane runs a broken
/// **aqueduct**, the side's high road, five and a half metres up from its
/// start to a broken end over the point's north flank. Steps climb the dyke
/// from either face; reeds and a fallen arch are cover.
pub static DELTA: Arena = Arena {
    id: ArenaId::DELTA,
    name: "Delta",
    creature: None,
    bounds: Bounds::cm((-4000, 4000), (-2800, 2800)),
    floor: Grass,
    regions: &DELTA_REGIONS,
    solids: &DELTA_SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-3600, 0, (0, 0)), Mark::cm(3600, 0, (0, 0))],
        hunt: None,
    },
    sites: &[],
    rim: Some(Rim::new((-4000, 4000), (-2800, 2800), [0, 0, 0, 0], 900)),
};

pub static DELTA_STAGE: Stage = Stage {
    arena: ArenaId::DELTA,
    kind: Kind::Hill,
    theme: "A river mouth among reeds: dykes, a hidden sluice, a broken aqueduct",
    pit: None,
    hill: Some(Hill::cm(0, 45, 0, 400, 50, 1000)),
    vents: &[],
    drifts: &[],
    ways: &DELTA_WAYS,
};

const DELTA_REGIONS: [Region; 7] = [
    // The river's channels, south of the west dyke and north of the east.
    Region {
        area: Area::rect_cm((-1300, 1300), (-2800, -2450)),
        material: Water,
    },
    Region {
        area: Area::rect_cm((-1300, 1300), (2450, 2800)),
        material: Water,
    },
    // Pools among the reeds.
    Region {
        area: Area::disc_cm(-2600, 700, 300),
        material: Water,
    },
    Region {
        area: Area::disc_cm(2600, -700, 300),
        material: Water,
    },
    // Peat round the point's island and the starts.
    Region {
        area: Area::disc_cm(0, 0, 650),
        material: Peat,
    },
    Region {
        area: Area::disc_cm(-3600, 0, 450),
        material: Ash,
    },
    Region {
        area: Area::disc_cm(3600, 0, 450),
        material: Ash,
    },
];

const DELTA_SOLIDS: [Solid; 33] = whole(
    [
        // The point's island: a low mound, walked up.
        round(0, 0, (0, 45), (500, 450), Grass),
    ],
    [
        // Two of the four standing stones round the point.
        round(600, 450, (0, 280), (50, 40), Stone),
        round(-600, 450, (0, 280), (50, 40), Stone),
        // The west dyke, three metres high: its north length, and its south
        // length in three -- two banks and the roof of the sluice between
        // them, two metres wide and two twenty high -- with the breach
        // between, and a lane at each end before the rim.
        Solid::cm([-1900, 0, 350], [-1300, 300, 2400], Ground),
        Solid::cm([-1900, 0, -2400], [-1700, 300, -350], Ground),
        Solid::cm([-1500, 0, -2400], [-1300, 300, -350], Ground),
        Solid::cm([-1700, 220, -2400], [-1500, 300, -350], Ground),
        // Steps up the dyke: from the start's side on the north length, and
        // from the point's side on the south.
        Solid::cm([-2200, 0, 800], [-1900, 150, 1200], Ground),
        Solid::cm([-1300, 0, -1300], [-1000, 150, -900], Ground),
        // The aqueduct's deck, five metres up to its underside, from the
        // west start to a broken end over the point's north flank, and the
        // two piers it stands on.
        Solid::cm([-3000, 500, 1300], [500, 570, 1700], Stone),
        round(-2400, 1500, (0, 500), (60, 60), Stone),
        round(-600, 1500, (0, 500), (60, 60), Stone),
        // Its stair at the start.
        Solid::cm([-3400, 0, 1300], [-3000, 180, 1700], Stone),
        Solid::cm([-3200, 0, 1300], [-3000, 360, 1700], Stone),
        // A hummock of reeds; a low wall across the start; a fallen arch.
        round(-2600, -1200, (0, 120), (150, 110), Peat),
        Solid::cm([-3300, 0, -600], [-3150, 130, 600], Rock),
        slab(-800, -1500, (0, 250), (250, 40), (10, 1), Stone),
    ],
);

const DELTA_WAYS: [Way; 5] = [
    Way::new(
        "way 0",
        (-2600, 0, 1000),
        &[(-2050, 150, 1000), (-1600, 300, 1000)],
    ),
    Way::new(
        "way 1",
        (-600, 0, -1100),
        &[(-1150, 150, -1100), (-1600, 300, -1100)],
    ),
    Way::new(
        "way 2",
        (-3700, 0, 1500),
        &[
            (-3300, 180, 1500),
            (-3100, 360, 1500),
            (-2700, 570, 1500),
            (300, 570, 1500),
        ],
    ),
    Way::new(
        "way 3",
        (-1600, 0, -2600),
        &[(-1600, 0, -500), (-1600, 0, 0), (0, 45, 0)],
    ),
    Way::new("way 4", (-2800, 0, 0), &[(0, 45, 0)]),
];

/// The floor's relief (`arena::relief`): smooth hills and hollows, no boxes.
/// Kept clear of the point, the starts and the rim.
pub const BUMPS: [Bump; 0] = [];
