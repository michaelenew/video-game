//! **Floating duel stages** (2026-10-21, `crate::versus`,
//! `docs/design/versus.md`): small islands hanging in open air, forty
//! metres over a floor nobody reaches. Knocked off the edge, a fighter falls
//! past the stage's **pit** and is out, as if their health had run out --
//! so spacing near an edge is the game, the way it is in Smash.
//!
//! Each is **tight** -- twenty-odd metres across, so neither fighter can run
//! from the other for long -- and has **a few changes of height**, a metre
//! or two each, so there is a high place to hold and a low one to be pushed
//! toward. Each is turned half a turn about its middle (`hills::whole`), so
//! both starts are the same start.
//!
//! Every island hangs: its bottom is above the floor, so it is a ceiling to
//! whatever is under it as well as a floor to whoever is on it, as a jump
//! course's are. The marks stand on the island's top (`Mark::up_cm`).

use super::hills::whole;
use super::{Arena, ArenaId, Bounds, Mark, Material, Solid, Spawns};
use crate::fixed::Fx;
use crate::math::V3;
use crate::versus::{Kind, Stage};

use Material::{Ash, Grass, Rock, Sand, Snow, Stone};

/// A round solid from centimetres: centred on (x, z), from `y.0` to `y.1`,
/// `r.0` across its foot and `r.1` across its top (radii).
const fn round(x: i32, z: i32, y: (i32, i32), r: (i32, i32), material: Material) -> Solid {
    let big = if r.0 > r.1 { r.0 } else { r.1 };
    Solid {
        min: V3::new(super::cm(x - big), super::cm(y.0), super::cm(z - big)),
        max: V3::new(super::cm(x + big), super::cm(y.1), super::cm(z + big)),
        material,
        shape: super::Shape::Round {
            bottom: super::cm(r.0),
            top: super::cm(r.1),
            hollow: Fx::ZERO,
        },
    }
}

// ---------------------------------------------------------------------------
// The Anvil
// ---------------------------------------------------------------------------

/// **The Anvil**: a slab of black basalt over a sea of embers at dusk,
/// twenty metres by thirteen. A low boss in its middle; two slabs of stone
/// hovering three metres thirty over it either side, to jump onto and fight
/// down from; and off each end **the horn**, a rock of its own four metres
/// out over the drop and a metre lower than the face -- a jump to reach,
/// and the place you are pushed to.
pub static ANVIL: Arena = Arena {
    id: ArenaId::ANVIL,
    name: "Anvil",
    creature: None,
    bounds: Bounds::cm((-2400, 2400), (-1800, 1800)),
    floor: Ash,
    regions: &[],
    solids: &ANVIL_SOLIDS,
    spawns: Spawns {
        versus: [
            Mark::up_cm(-600, 4000, 0, (600, 0)),
            Mark::up_cm(600, 4000, 0, (-600, 0)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: None,
};

pub static ANVIL_STAGE: Stage = Stage {
    arena: ArenaId::ANVIL,
    kind: Kind::Duel,
    theme: "A basalt anvil over a sea of embers: hovering slabs, a horn at each end",
    pit: Some(2500),
    hill: None,
};

const ANVIL_SOLIDS: [Solid; 6] = whole(
    [
        // The face: twenty by thirteen, five metres thick.
        Solid::cm([-1000, 3500, -650], [1000, 4000, 650], Rock),
        // The boss in its middle, a step up, walked.
        round(0, 0, (4000, 4045), (180, 150), Rock),
    ],
    [
        // The east horn: a rock of its own, four metres of air off the
        // face's end and a metre down.
        Solid::cm([1400, 3500, -300], [1900, 3900, 300], Rock),
        // The east slab, hovering: its underside three metres thirty over
        // the face, its top three seventy.
        Solid::cm([350, 4330, -200], [850, 4370, 200], Stone),
    ],
);

// ---------------------------------------------------------------------------
// The Ziggurat
// ---------------------------------------------------------------------------

/// **The Ziggurat**: the top of a sandstone temple risen into the sky over
/// desert haze. Three tiers, a metre each -- a broad court, a terrace, a
/// crown four metres square -- so whoever holds the crown fights downhill.
/// An obelisk stands at two corners of the court for cover, and off each
/// long side a single stepping stone hangs three and a half metres out over
/// the drop, a metre and a half down.
pub static ZIGGURAT: Arena = Arena {
    id: ArenaId::ZIGGURAT,
    name: "Ziggurat",
    creature: None,
    bounds: Bounds::cm((-2200, 2200), (-2000, 2000)),
    floor: Sand,
    regions: &[],
    solids: &ZIGGURAT_SOLIDS,
    spawns: Spawns {
        versus: [
            Mark::up_cm(-850, 4000, 0, (850, 0)),
            Mark::up_cm(850, 4000, 0, (-850, 0)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: None,
};

pub static ZIGGURAT_STAGE: Stage = Stage {
    arena: ArenaId::ZIGGURAT,
    kind: Kind::Duel,
    theme: "A temple's top in the sky: three tiers to a crown, a stone off each side",
    pit: Some(2500),
    hill: None,
};

const ZIGGURAT_SOLIDS: [Solid; 7] = whole(
    [
        // The court, the terrace, the crown.
        Solid::cm([-1100, 3500, -800], [1100, 4000, 800], Sand),
        Solid::cm([-650, 4000, -500], [650, 4100, 500], Stone),
        Solid::cm([-250, 4100, -250], [250, 4200, 250], Stone),
    ],
    [
        // An obelisk at the court's north-west corner.
        round(-900, 600, (4000, 4300), (50, 40), Stone),
        // A stepping stone off the north side, three and a half metres out
        // and a metre and a half down.
        Solid::cm([-200, 3750, 1150], [200, 3850, 1450], Stone),
    ],
);

// ---------------------------------------------------------------------------
// The Caldera
// ---------------------------------------------------------------------------

/// **The Caldera**: a volcano's crater torn loose and floating, twenty
/// metres square, an ash floor inside a broken lip of rock a metre twenty
/// high. The lip stands at the four corners and is gone at the middle of
/// each side -- six metres of nothing between you and the drop, four ways
/// out. In the middle the old vent's plug stands a metre and a quarter, a
/// drum to stand on top of or fight round.
pub static CALDERA: Arena = Arena {
    id: ArenaId::CALDERA,
    name: "Caldera",
    creature: None,
    bounds: Bounds::cm((-2000, 2000), (-2000, 2000)),
    floor: Ash,
    regions: &[],
    solids: &CALDERA_SOLIDS,
    spawns: Spawns {
        versus: [
            Mark::up_cm(-600, 4000, -600, (600, 600)),
            Mark::up_cm(600, 4000, 600, (-600, -600)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: None,
};

pub static CALDERA_STAGE: Stage = Stage {
    arena: ArenaId::CALDERA,
    kind: Kind::Duel,
    theme: "A floating crater: a lip broken four ways, a drum in the middle",
    pit: Some(2500),
    hill: None,
};

const CALDERA_SOLIDS: [Solid; 10] = whole(
    [
        // The crater's floor.
        Solid::cm([-1000, 3500, -1000], [1000, 4000, 1000], Ash),
        // The plug: a drum of rock a metre and a quarter high, three
        // metres across.
        round(0, 0, (4000, 4125), (150, 150), Rock),
    ],
    [
        // The lip at the north-west corner, an L.
        Solid::cm([-1000, 4000, 900], [-300, 4120, 1000], Rock),
        Solid::cm([-1000, 4000, 300], [-900, 4120, 900], Rock),
        // And at the north-east.
        Solid::cm([300, 4000, 900], [1000, 4120, 1000], Rock),
        Solid::cm([900, 4000, 300], [1000, 4120, 900], Rock),
    ],
);

// ---------------------------------------------------------------------------
// The Twin Isles
// ---------------------------------------------------------------------------

/// **The Twin Isles**: two snowy islands at dawn, ten metres by fourteen
/// each, seven metres apart, joined by a stone bridge three metres wide --
/// knocked sideways off it is out. A knoll a metre high on each isle, and
/// over the gap north and south two stones hang with three metres thirty of
/// air under them, a way across that is not the bridge.
pub static TWIN_ISLES: Arena = Arena {
    id: ArenaId::TWIN_ISLES,
    name: "Twin Isles",
    creature: None,
    bounds: Bounds::cm((-2600, 2600), (-1900, 1900)),
    floor: Snow,
    regions: &[],
    solids: &TWIN_ISLES_SOLIDS,
    spawns: Spawns {
        versus: [
            Mark::up_cm(-850, 4000, 300, (850, -300)),
            Mark::up_cm(850, 4000, -300, (-850, 300)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: None,
};

pub static TWIN_ISLES_STAGE: Stage = Stage {
    arena: ArenaId::TWIN_ISLES,
    kind: Kind::Duel,
    theme: "Two snowy isles at dawn: a narrow bridge, stones hung over the gap",
    pit: Some(2500),
    hill: None,
};

const TWIN_ISLES_SOLIDS: [Solid; 7] = whole(
    [
        // The bridge.
        Solid::cm([-350, 3900, -150], [350, 4000, 150], Stone),
    ],
    [
        // The west isle.
        Solid::cm([-1400, 3500, -700], [-350, 4000, 700], Snow),
        // Its knoll.
        Solid::cm([-1200, 4000, -500], [-800, 4100, -100], Rock),
        // A stone hung over the gap, north of the bridge: three metres
        // thirty of air under it.
        Solid::cm([-200, 4330, 350], [200, 4380, 750], Grass),
    ],
);
