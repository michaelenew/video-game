//! **Hearth**: the walled town at the valley's mouth, and where the valley
//! starts (`docs/design/valley.md`).
//!
//! Seventy metres of paved square inside a wall nine metres high, with three
//! ways out: **the valley gate** east, to the Mouth and the climb; **the Ring's
//! door** north, the one place in the valley where the two of you may fight
//! each other; and **the west gate** to the Long Valley, under a waystone that
//! lights when the Mantis is beaten, because the Siegeshell walks that valley
//! toward this wall. A stair inside the east wall goes up onto the wall's walk,
//! which is the **lookout**: from it the reaches are drawn on the sky, one
//! behind another, as far as the Saddle.
//!
//! Nobody fights here. The houses, the hall and the armoury are solids, the
//! bell tower is the tallest thing in town, and the well marks the square.

use super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns};
use crate::valley::{Gate, Kind, Place, Seam, Zone};

use Material::{Ash, Grass, Stone, Wood};

pub static ARENA: Arena = Arena {
    id: ArenaId::HEARTH,
    name: "Hearth",
    creature: None,
    bounds: Bounds::cm((-4500, 4500), (-3200, 4000)),
    floor: Stone,
    regions: &REGIONS,
    solids: &SOLIDS,
    spawns: Spawns {
        // The square, by the well, looking down the road to the valley gate.
        versus: [
            Mark::cm(200, -150, (4000, -150)),
            Mark::cm(200, 150, (4000, 150)),
        ],
        hunt: None,
    },
    sites: &[],
};

const REGIONS: [Region; 4] = [
    // The road through both gates: cobbles.
    Region {
        area: Area::rect_cm((-4500, 4500), (-250, 250)),
        material: Ash,
    },
    // The lane to the Ring's door.
    Region {
        area: Area::rect_cm((-250, 250), (250, 4000)),
        material: Ash,
    },
    // A green by the bell tower.
    Region {
        area: Area::disc_cm(-2000, 800, 600),
        material: Grass,
    },
    // A green by the east houses.
    Region {
        area: Area::disc_cm(2200, 600, 500),
        material: Grass,
    },
];

const SOLIDS: [Solid; 27] = [
    // The east wall, either side of the valley gate.
    Solid::cm([3500, 0, -3200], [3700, 900, -300], Stone),
    Solid::cm([3500, 0, 300], [3700, 900, 3200], Stone),
    // The valley gate's passage, open at its far end onto the land.
    Solid::cm([3700, 0, -500], [4300, 900, -300], Stone),
    Solid::cm([3700, 0, 300], [4300, 900, 500], Stone),
    // The west wall, either side of the Long Valley's gate.
    Solid::cm([-3700, 0, -3200], [-3500, 900, -300], Stone),
    Solid::cm([-3700, 0, 300], [-3500, 900, 3200], Stone),
    // The west gate's passage.
    Solid::cm([-4300, 0, -500], [-3700, 900, -300], Stone),
    Solid::cm([-4300, 0, 300], [-3700, 900, 500], Stone),
    // The north wall, either side of the Ring's door.
    Solid::cm([-3700, 0, 3000], [-300, 900, 3200], Stone),
    Solid::cm([300, 0, 3000], [3700, 900, 3200], Stone),
    // The Ring's lane, open at its far end onto the path to the Ring.
    Solid::cm([-500, 0, 3200], [-300, 900, 3800], Stone),
    Solid::cm([300, 0, 3200], [500, 900, 3800], Stone),
    // The south wall.
    Solid::cm([-3700, 0, -3200], [3700, 900, -3000], Stone),
    // The stair up the east wall to its walk: the lookout.
    Solid::cm([3200, 0, -2800], [3500, 150, -2500], Stone),
    Solid::cm([3200, 0, -2500], [3500, 300, -2200], Stone),
    Solid::cm([3200, 0, -2200], [3500, 450, -1900], Stone),
    Solid::cm([3200, 0, -1900], [3500, 600, -1600], Stone),
    Solid::cm([3200, 0, -1600], [3500, 750, -1300], Stone),
    // The armoury.
    Solid::cm([-2800, 0, -2600], [-1600, 600, -1600], Stone),
    // The hall.
    Solid::cm([-1200, 0, -2600], [0, 700, -1800], Wood),
    // Houses.
    Solid::cm([800, 0, -2600], [1800, 500, -1800], Wood),
    Solid::cm([-2800, 0, 1400], [-2000, 500, 2400], Wood),
    Solid::cm([-1200, 0, 1800], [-400, 550, 2600], Wood),
    Solid::cm([1000, 0, 1600], [2000, 600, 2600], Wood),
    // The bell tower.
    Solid::cm([-2500, 0, 200], [-2100, 1400, 600], Stone),
    // The well.
    Solid::cm([-700, 0, -700], [-500, 100, -500], Stone),
    // The Long Valley's waystone: lit when the Mantis is beaten.
    Solid::cm([-3350, 0, 450], [-3250, 320, 550], Stone),
];

/// How Hearth joins the rest of the valley.
pub static PLACE: Place = Place {
    arena: ArenaId::HEARTH,
    kind: Kind::Town,
    seams: &SEAMS,
    vines: &[],
    vents: &[],
};

const SEAMS: [Seam; 3] = [
    // The valley gate: the Mouth, and the climb.
    Seam {
        zone: Zone::cm([3550, -100, -300], [4300, 500, 300]),
        to: ArenaId::MOUTH,
        at: 0,
        marks: [
            Mark::cm(3100, -150, (2100, -150)),
            Mark::cm(3100, 150, (2100, 150)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the valley",
    },
    // The Ring's door.
    Seam {
        zone: Zone::cm([-300, -100, 3050], [300, 500, 3800]),
        to: ArenaId::RING,
        at: 0,
        marks: [
            Mark::cm(-150, 2600, (-150, 1600)),
            Mark::cm(150, 2600, (150, 1600)),
        ],
        gate: Gate::Open,
        waystone: None,
        says: "the Ring",
    },
    // The west gate: the Long Valley, where the Siegeshell walks.
    Seam {
        zone: Zone::cm([-4300, -100, -300], [-3550, 500, 300]),
        to: ArenaId::SIEGESHELL,
        at: 0,
        marks: [
            Mark::cm(-3100, -150, (-2100, -150)),
            Mark::cm(-3100, 150, (-2100, 150)),
        ],
        gate: crate::valley::tier::FIVE,
        waystone: Some(26),
        says: "the Long Valley",
    },
];
