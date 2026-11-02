//! **The Glacier**, a King of the Hill map (`super`, `crate::versus`,
//! `docs/design/versus.md` §3, `docs/design/fight-spaces.md`).

#![allow(unused_imports)]

use super::super::relief::Bump;
use super::super::rim::Rim;
use super::super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns};
use super::{round, slab, whole};
use crate::versus::{Hill, Kind, Stage, Way};

use Material::{Ash, Grass, Ground, Peat, Rock, Sand, Snow, Stone, Water, Wood};

/// **The Glacier**: a snowfield under a glacier's snout. The point is on
/// **the dome**, a hump of old ice three metres up in the middle, climbed by
/// a stair of ice at one end of each side's face and a ledge in the middle
/// of the north and south faces. Under the dome runs **the melt cave**, east
/// to west, a sneak from one side's half of the field to the other's
/// beneath the defenders' feet. Each side's own high ground is **an ice
/// shelf** five and a half metres up along the rim on its left, climbed from
/// its start, whose edge looks down on the dome across a gap you can drop
/// from -- and under each shelf another cave runs from the start out to the
/// dome's foot. Seracs, boulders and blocks of ice stand about the open
/// field for cover.
pub static GLACIER: Arena = Arena {
    id: ArenaId::GLACIER,
    name: "Glacier",
    creature: None,
    bounds: Bounds::cm((-3800, 3800), (-2800, 2800)),
    floor: Snow,
    regions: &GLACIER_REGIONS,
    solids: &GLACIER_SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-3300, -400, (0, 0)), Mark::cm(3300, 400, (0, 0))],
        hunt: None,
    },
    sites: &[],
    // The bank starts at the shelves' height on the north and south, whose
    // backs are against it: walking off a shelf's back is walking onto the
    // bank, not into a crease under it.
    rim: Some(Rim::new(
        (-3800, 3800),
        (-2800, 2800),
        [550, 550, 0, 0],
        1100,
    )),
};

pub static GLACIER_STAGE: Stage = Stage {
    arena: ArenaId::GLACIER,
    kind: Kind::Hill,
    theme: "A snowfield under a glacier: the point on an ice dome, caves beneath",
    pit: None,
    hill: Some(Hill::cm(0, 300, 0, 400, 50, 1000)),
    vents: &[],
    drifts: &[],
    ways: &GLACIER_WAYS,
};

const GLACIER_REGIONS: [Region; 4] = [
    // Wind-scoured rock where the snow has blown off.
    Region {
        area: Area::disc_cm(-2000, 800, 500),
        material: Rock,
    },
    Region {
        area: Area::disc_cm(2000, -800, 500),
        material: Rock,
    },
    // Each start's yard, trodden down to the rock.
    Region {
        area: Area::disc_cm(-3300, -400, 500),
        material: Rock,
    },
    Region {
        area: Area::disc_cm(3300, 400, 500),
        material: Rock,
    },
];

const GLACIER_SOLIDS: [Solid; 27] = whole(
    [
        // The melt cave's roof under the point: two metres twenty of
        // headroom, three wide, fourteen long.
        Solid::cm([-700, 220, -150], [700, 300, 150], Snow),
    ],
    [
        // The dome's north half, three metres up.
        Solid::cm([-700, 0, 150], [700, 300, 900], Snow),
        // Its stair at the west end of the north face, and a ledge in the
        // middle of that face.
        Solid::cm([-1100, 0, 450], [-700, 150, 900], Snow),
        Solid::cm([-250, 0, 900], [250, 150, 1150], Snow),
        // The west side's ice shelf along the north rim, five and a half
        // metres up, with a cave the length of it at its foot: the shelf's
        // front, the cave's roof, the shelf's back.
        Solid::cm([-3000, 0, 1500], [-1200, 550, 1800], Snow),
        Solid::cm([-3000, 300, 1800], [-1200, 550, 2100], Snow),
        Solid::cm([-3000, 0, 2100], [-1200, 550, 2800], Snow),
        // Its stair up from the west start.
        Solid::cm([-3500, 0, 1200], [-3000, 180, 1500], Rock),
        Solid::cm([-3300, 0, 1200], [-3000, 360, 1500], Rock),
        // A serac in the west half of the field: tall, thin, cover.
        round(-1800, -400, (0, 600), (100, 70), Snow),
        // Blocks of ice and a boulder.
        Solid::cm([-2400, 0, -1400], [-2100, 140, -900], Snow),
        Solid::cm([-1500, 0, 200], [-1300, 180, 600], Snow),
        round(-900, -1800, (0, 200), (130, 90), Rock),
        // A low wall of packed snow across the west start.
        Solid::cm([-2900, 0, -900], [-2750, 130, 100], Snow),
    ],
);

const GLACIER_WAYS: [Way; 5] = [
    Way::new(
        "way 0",
        (-1600, 0, 700),
        &[(-900, 150, 700), (-300, 300, 700)],
    ),
    Way::new("way 1", (0, 0, 1700), &[(0, 150, 1050), (0, 300, 500)]),
    Way::new(
        "way 2",
        (-3700, 0, 1350),
        &[(-3400, 180, 1350), (-3150, 360, 1350), (-2700, 550, 1650)],
    ),
    Way::new("way 3", (-1600, 0, 0), &[(1600, 0, 0)]),
    Way::new("way 4", (-3400, 0, 1950), &[(-900, 0, 1950)]),
];

/// The floor's relief (`arena::relief`): smooth hills and hollows, no boxes.
/// Kept clear of the point, the starts and the rim.
pub const BUMPS: [Bump; 0] = [];
