//! **Floating duel stages** (2026-10-21, `crate::versus`,
//! `docs/design/versus.md`): islands hanging in open air, forty metres over
//! a floor nobody reaches. Knocked off the edge, a fighter falls past the
//! stage's **pit** and is out, as if their health had run out -- so spacing
//! near an edge is the game, the way it is in Smash.
//!
//! **Rebuilt 2026-11-02 to the fight-space rules**
//! (`docs/design/fight-spaces.md` §4). The owner: the duel stages were
//! "tiny relative to each class's mobility options". So each keeps its
//! **main island under a reach across** -- the edge is the point of a
//! ring-out stage -- and gains **pieces**:
//!
//! - **islands across real gaps**, five to seven metres, so that the stage's
//!   pieces span more than 1.8 reaches end to end;
//! - **a level held above the starts**, 8.4 m up (more than two jumps),
//!   climbed by steps of 2.7 m (a jump each, under a jump and a half, so
//!   the steps are not levels of their own) and, on three of the four, a
//!   second way, by a lift or an updraft;
//! - **something that moves you**: a lift or a ferry (`crate::drift`), an
//!   updraft;
//! - **a recovery**: one narrow updraft a side (two metres round), beside an
//!   edge, rising from just over the pit to a metre and a half over the
//!   ledge -- a fighter knocked off who steers into it rides back up. Two
//!   to a stage, so a ring-out still happens.
//!
//! Each is turned half a turn about its middle (`hills::whole`), so both
//! starts are the same start; an updraft or a drift is listed twice, at
//! (x, z) and (-x, -z), because nothing turns it.
//!
//! Every island hangs: its bottom is above the floor, so it is a ceiling to
//! whatever is under it as well as a floor to whoever is on it, as a jump
//! course's are. The marks stand on the island's top (`Mark::up_cm`).
//!
//! Heights, over the starts' 40 m: a step is 2.7 m (4270), a second 5.4 m
//! (4540), the held level 8.4 m (4840).

use super::hills::{turned, whole};
use super::{Arena, ArenaId, Bounds, Mark, Material, Solid, Spawns, cm};
use crate::drift::Drift;
use crate::fixed::Fx;
use crate::math::V3;
use crate::valley::Vent;
use crate::versus::{Kind, Stage, Way};

use Material::{Ash, Crystal, Grass, Rock, Sand, Snow, Stone};

/// A round solid from centimetres: centred on (x, z), from `y.0` to `y.1`,
/// `r.0` across its foot and `r.1` across its top (radii).
const fn round(x: i32, z: i32, y: (i32, i32), r: (i32, i32), material: Material) -> Solid {
    let big = if r.0 > r.1 { r.0 } else { r.1 };
    Solid {
        min: V3::new(cm(x - big), cm(y.0), cm(z - big)),
        max: V3::new(cm(x + big), cm(y.1), cm(z + big)),
        material,
        shape: super::Shape::Round {
            bottom: cm(r.0),
            top: cm(r.1),
            hollow: Fx::ZERO,
        },
    }
}

/// An updraft from centimetres: centred on (x, z), `r` round, carrying
/// from `from` up to `top`.
const fn vent(x: i32, z: i32, r: i32, from: i32, top: i32) -> Vent {
    Vent {
        at: (cm(x), cm(z)),
        radius: cm(r),
        top: cm(top),
        from: cm(from),
    }
}

/// **A drift and its turned copy**: `solid` at its first stop, `travel`
/// (x, y, z) centimetres to its second, on a clock of (wait, go, stay,
/// back) frames. The copy is turned half a turn about the middle, as
/// [`whole`] turns a half, and runs on the same clock.
const fn pair(solid: Solid, travel: (i32, i32, i32), clock: (u16, u16, u16, u16)) -> [Drift; 2] {
    let one = Drift {
        solid,
        travel: V3::new(cm(travel.0), cm(travel.1), cm(travel.2)),
        wait: clock.0,
        go: clock.1,
        stay: clock.2,
        back: clock.3,
        phase: 0,
    };
    let other = Drift {
        solid: turned(solid),
        travel: V3::new(cm(-travel.0), cm(travel.1), cm(-travel.2)),
        ..one
    };
    [one, other]
}

/// Two pairs of drifts as one table.
const fn four(a: [Drift; 2], b: [Drift; 2]) -> [Drift; 4] {
    [a[0], a[1], b[0], b[1]]
}

/// A lift's clock: two seconds at the bottom, two and a half up, two at
/// the top, two and a half down.
const LIFT: (u16, u16, u16, u16) = (120, 150, 120, 150);

// ---------------------------------------------------------------------------
// The Anvil
// ---------------------------------------------------------------------------

/// **The Anvil**: a slab of black basalt over a sea of embers at dusk,
/// twenty-three metres by fourteen, its waist narrowing underneath. A low
/// boss in the middle and a block of basalt a side to put between you.
/// Over each half **a hammer** hangs, a slab six metres by five-and-a-half,
/// 8.4 m up: climbed by two hovering stones off the start, or ridden up to
/// on **a lift** off the face's long side. Off each end **the horn**, a rock
/// of its own across five and a half metres of air and a metre lower than
/// the face -- the place you are pushed to. Beside each long side, an
/// **updraft** rises from just over the pit to a metre and a half over the
/// face: knocked off there, steer into it.
pub static ANVIL: Arena = Arena {
    id: ArenaId::ANVIL,
    name: "Anvil",
    creature: None,
    bounds: Bounds::cm((-3000, 3000), (-2000, 2000)),
    floor: Ash,
    regions: &[],
    solids: &ANVIL_SOLIDS,
    spawns: Spawns {
        versus: [
            Mark::up_cm(-950, 4000, 450, (950, -450)),
            Mark::up_cm(950, 4000, -450, (-950, 450)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: None,
};

pub static ANVIL_STAGE: Stage = Stage {
    arena: ArenaId::ANVIL,
    kind: Kind::Duel,
    theme: "A basalt anvil over a sea of embers: hammers hung over it, a horn at each end",
    pit: Some(2500),
    hill: None,
    front: None,
    vents: &ANVIL_VENTS,
    drifts: &ANVIL_DRIFTS,
    ways: &ANVIL_WAYS,
};

const ANVIL_SOLIDS: [Solid; 15] = whole(
    [
        // The face: twenty-three by fourteen, six metres thick.
        Solid::cm([-1150, 3400, -700], [1150, 4000, 700], Rock),
        // Its waist, narrower, under it.
        Solid::cm([-700, 2900, -450], [700, 3400, 450], Rock),
        // The boss in its middle, a step up, walked.
        round(0, 0, (4000, 4060), (220, 180), Rock),
    ],
    [
        // The east horn: a rock of its own, five and a half metres of air
        // off the face's end and a metre down, off to the south so that
        // the drop straight off the end is a drop.
        Solid::cm([1700, 3500, -950], [2400, 3900, -150], Rock),
        // Its foot, narrower under it.
        Solid::cm([1850, 3200, -800], [2250, 3500, -300], Rock),
        // Two stones hovering off the east start, a jump each: 2.7 m with
        // 2.1 m of air under it, then 5.4 m.
        Solid::cm([800, 4210, -150], [1050, 4270, 100], Stone),
        Solid::cm([800, 4480, 250], [1050, 4540, 500], Stone),
        // The east hammer, 8.4 m over the face: six by five and a half.
        Solid::cm([200, 4780, 150], [800, 4840, 700], Stone),
        // A block of basalt to stand behind, or on: three metres a side.
        Solid::cm([150, 4000, -650], [450, 4300, -350], Rock),
    ],
);

/// A lift off each long side, level with the face at the bottom and the
/// hammer at the top: step on, ride, step off onto the hammer.
const ANVIL_DRIFTS: [Drift; 2] = pair(
    Solid::cm([250, 3960, 700], [650, 4000, 1150], Stone),
    (0, 840, 0),
    LIFT,
);

/// The recovery: beside the south side of the east half (and the north of
/// the west), clear of the end the horn is off.
const ANVIL_VENTS: [Vent; 2] = [
    vent(500, -950, 200, 2600, 4150),
    vent(-500, 950, 200, 2600, 4150),
];

// ---------------------------------------------------------------------------
// The Ziggurat
// ---------------------------------------------------------------------------

/// **The Ziggurat**: the top of a sandstone temple risen into the sky over
/// desert haze, its court twenty-three metres by seventeen. Three tiers of
/// 2.7 m -- a jump each, a wall to climb -- to a crown five metres square
/// 8.4 m over the court, so whoever holds the crown fights downhill; and
/// for the way up that is not a climb, **a dais** a side, a lift out of the
/// court against the first tier that rises to the crown's height, a level
/// jump from it over the tiers. An obelisk at two corners of the court for
/// cover. Off each long side, seven metres out and three down, **a stele**
/// hangs, an island of its own: jumped down to, and come back from on **a
/// ferry** that crosses the gap at the court's height. Off each end, an
/// **updraft** for the knocked-off.
pub static ZIGGURAT: Arena = Arena {
    id: ArenaId::ZIGGURAT,
    name: "Ziggurat",
    creature: None,
    bounds: Bounds::cm((-2200, 2200), (-2600, 2600)),
    floor: Sand,
    regions: &[],
    solids: &ZIGGURAT_SOLIDS,
    spawns: Spawns {
        versus: [
            Mark::up_cm(-950, 4000, 600, (950, -600)),
            Mark::up_cm(950, 4000, -600, (-950, 600)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: None,
};

pub static ZIGGURAT_STAGE: Stage = Stage {
    arena: ArenaId::ZIGGURAT,
    kind: Kind::Duel,
    theme: "A temple's top in the sky: tiers to a crown, a stele off each side and a ferry to it",
    pit: Some(2500),
    hill: None,
    front: None,
    vents: &ZIGGURAT_VENTS,
    drifts: &ZIGGURAT_DRIFTS,
    ways: &ZIGGURAT_WAYS,
};

const ZIGGURAT_SOLIDS: [Solid; 12] = whole(
    [
        // The court, and the temple going on down under it.
        Solid::cm([-1150, 3400, -850], [1150, 4000, 850], Sand),
        Solid::cm([-800, 2950, -600], [800, 3400, 600], Sand),
        Solid::cm([-400, 2650, -300], [400, 2950, 300], Sand),
        // The tiers: 2.7 m each, and the crown.
        Solid::cm([-750, 4000, -600], [750, 4270, 600], Stone),
        Solid::cm([-500, 4270, -400], [500, 4540, 400], Stone),
        Solid::cm([-250, 4540, -250], [250, 4840, 250], Stone),
    ],
    [
        // An obelisk at the court's south-west corner: four and a half
        // metres, taller than any jump.
        round(-950, -650, (4000, 4450), (70, 55), Stone),
        // The stele off the north side: seven metres of air from the court,
        // three metres down.
        Solid::cm([-900, 3200, 1550], [-100, 3700, 2400], Sand),
        Solid::cm([-750, 2900, 1700], [-250, 3200, 2250], Sand),
    ],
);

/// The ferry to the north stele (and the south): at the court's edge, then
/// across the gap to hang over the stele's near end, three metres up; and
/// the dais on the court's north side (and the south), a slab lying on the
/// court against the first tier, rising to the crown's height.
const ZIGGURAT_DRIFTS: [Drift; 4] = four(
    pair(
        Solid::cm([-650, 3960, 850], [-350, 4000, 1150], Stone),
        (0, 0, 500),
        (150, 180, 150, 180),
    ),
    pair(
        Solid::cm([-450, 4000, 600], [-50, 4010, 850], Stone),
        (0, 830, 0),
        LIFT,
    ),
);

/// The recovery: off the west end (and the east), away from the starts.
const ZIGGURAT_VENTS: [Vent; 2] = [
    vent(-1450, -300, 200, 2600, 4150),
    vent(1450, 300, 200, 2600, 4150),
];

// ---------------------------------------------------------------------------
// The Caldera
// ---------------------------------------------------------------------------

/// **The Caldera**: a volcano's crater torn loose and floating, twenty-two
/// metres square, an ash floor inside a broken lip of rock two metres high
/// and a metre and a half thick. The lip stands at the four corners and is
/// gone at the middle of each side -- eight metres of nothing, four ways
/// out. **The plug has been blown out**: it hangs over the crater in two
/// halves, slabs 8.4 m up with five metres of open sky between them over
/// the middle, and where it was **the vent still breathes**, an updraft
/// out of the crater floor that throws you up past them. Each half is
/// climbed from a corner of the lip by a hovering boulder, or ridden up to
/// on **a lift** out of the gap beside it. Off the east and west gaps, six
/// metres out and two down, **an ejecta island**; beside the corner by
/// each, the **updraft** a knocked-off fighter steers into.
pub static CALDERA: Arena = Arena {
    id: ArenaId::CALDERA,
    name: "Caldera",
    creature: None,
    bounds: Bounds::cm((-2600, 2600), (-2000, 2000)),
    floor: Ash,
    regions: &[],
    solids: &CALDERA_SOLIDS,
    spawns: Spawns {
        versus: [
            Mark::up_cm(-750, 4000, -750, (750, 750)),
            Mark::up_cm(750, 4000, 750, (-750, -750)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: None,
};

pub static CALDERA_STAGE: Stage = Stage {
    arena: ArenaId::CALDERA,
    kind: Kind::Duel,
    theme: "A floating crater: a lip broken four ways, its plug blown out and hung over it",
    pit: Some(2500),
    hill: None,
    front: None,
    vents: &CALDERA_VENTS,
    drifts: &CALDERA_DRIFTS,
    ways: &CALDERA_WAYS,
};

const CALDERA_SOLIDS: [Solid; 17] = whole(
    [
        // The crater's floor, and its cone going on down under it.
        Solid::cm([-1100, 3400, -1100], [1100, 4000, 1100], Ash),
        Solid::cm([-750, 2900, -750], [750, 3400, 750], Rock),
        Solid::cm([-350, 2600, -350], [350, 2900, 350], Rock),
    ],
    [
        // The lip at the north-west corner, an L two metres high.
        Solid::cm([-1100, 4000, 950], [-400, 4200, 1100], Rock),
        Solid::cm([-1100, 4000, 400], [-950, 4200, 950], Rock),
        // And at the north-east.
        Solid::cm([400, 4000, 950], [1100, 4200, 1100], Rock),
        Solid::cm([950, 4000, 400], [1100, 4200, 950], Rock),
        // A boulder hovering over the north-west lip, 5.4 m up: from the
        // lip, a jump; from it, a jump to the plug.
        Solid::cm([-1100, 4480, 450], [-800, 4540, 700], Rock),
        // The plug's north half, 8.4 m up, fourteen metres by eight and a
        // half, the middle left open.
        Solid::cm([-800, 4720, 250], [600, 4840, 1100], Rock),
        // The east ejecta island: six metres off the east gap, two down.
        Solid::cm([1700, 3400, -350], [2400, 3800, 350], Ash),
    ],
);

/// A lift out of the north gap (and the south): level with the floor at
/// the bottom, the plug at the top.
const CALDERA_DRIFTS: [Drift; 2] = pair(
    Solid::cm([-300, 3960, 1100], [100, 4000, 1550], Rock),
    (0, 840, 0),
    LIFT,
);

/// The old vent, under the open sky between the plug's halves, throwing
/// you a metre and a half past them; and the recovery beside the south-east
/// corner (and the north-west), up past the lip.
const CALDERA_VENTS: [Vent; 3] = [
    vent(0, 0, 150, 4010, 4990),
    vent(1400, -700, 200, 2600, 4350),
    vent(-1400, 700, 200, 2600, 4350),
];

// ---------------------------------------------------------------------------
// The Twin Isles
// ---------------------------------------------------------------------------

/// **The Twin Isles**: two snowy islands at dawn, ten and a half metres by
/// fifteen each, eight metres apart, joined by a stone bridge three metres
/// wide -- knocked sideways off it is out. **An arch of ice** crosses the
/// gap 8.4 m over the bridge, climbed by two seracs on each isle, a jump
/// each; north and south of the bridge a stone hangs with three metres
/// thirty of air under it, and under that **an ice floe** drifts across the
/// gap and back, the ways across that are not the bridge. Off each far end
/// a skerry, five metres out and a metre down; beside each isle's outer
/// side, the **updraft** a knocked-off fighter steers into.
pub static TWIN_ISLES: Arena = Arena {
    id: ArenaId::TWIN_ISLES,
    name: "Twin Isles",
    creature: None,
    bounds: Bounds::cm((-2600, 2600), (-1500, 1500)),
    floor: Snow,
    regions: &[],
    solids: &TWIN_ISLES_SOLIDS,
    spawns: Spawns {
        versus: [
            Mark::up_cm(-1000, 4000, 300, (1000, -300)),
            Mark::up_cm(1000, 4000, -300, (-1000, 300)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: None,
};

pub static TWIN_ISLES_STAGE: Stage = Stage {
    arena: ArenaId::TWIN_ISLES,
    kind: Kind::Duel,
    theme: "Two snowy isles at dawn: a narrow bridge, an ice arch over it, floes across the gap",
    pit: Some(2500),
    hill: None,
    front: None,
    vents: &TWIN_ISLES_VENTS,
    drifts: &TWIN_ISLES_DRIFTS,
    ways: &TWIN_ISLES_WAYS,
};

const TWIN_ISLES_SOLIDS: [Solid; 16] = whole(
    [
        // The bridge.
        Solid::cm([-400, 3900, -150], [400, 4000, 150], Stone),
        // The ice arch over it, 8.4 m up, thirteen metres long.
        Solid::cm([-650, 4720, -200], [650, 4840, 200], Crystal),
    ],
    [
        // The west isle, and its rock going on down under it.
        Solid::cm([-1450, 3400, -750], [-400, 4000, 750], Snow),
        Solid::cm([-1250, 2900, -500], [-600, 3400, 500], Rock),
        // A knoll of rock, low cover.
        Solid::cm([-1350, 4000, -650], [-1050, 4120, -350], Rock),
        // Two seracs toward the gap, 2.7 m and 5.4 m: the climb to the
        // arch, and something to stand behind.
        Solid::cm([-1000, 4000, -500], [-750, 4270, -250], Crystal),
        Solid::cm([-750, 4000, -500], [-500, 4540, -220], Crystal),
        // A stone hung over the gap, north of the bridge: three metres
        // thirty of air under it.
        Solid::cm([-250, 4330, 350], [250, 4380, 750], Grass),
        // The west skerry: five metres off the isle's end, a metre down.
        Solid::cm([-2400, 3500, -300], [-1950, 3900, 400], Snow),
    ],
);

/// A floe across the gap north of the bridge, west to east, under the hung
/// stone; and its copy south of it, east to west.
const TWIN_ISLES_DRIFTS: [Drift; 2] = pair(
    Solid::cm([-400, 3960, 400], [-100, 4000, 700], Crystal),
    (500, 0, 0),
    (120, 180, 120, 180),
);

/// The recovery: off the north side of the west isle (and the south of the
/// east), clear of the gap.
const TWIN_ISLES_VENTS: [Vent; 2] = [
    vent(-900, 1050, 200, 2600, 4150),
    vent(900, -1050, 200, 2600, 4150),
];

// ---------------------------------------------------------------------------
// The ways
// ---------------------------------------------------------------------------
//
// Each is walked from the first start's half (the second's is its turned
// copy, and the second fighter stands on its own mark). A way that waits on
// a drift -- a lift, the dais, a ferry -- is not among them: the walker
// does not wait.

const ANVIL_WAYS: [Way; 3] = [
    Way::new(
        "the hammer, by the lift",
        (-450, 4000, -450),
        &[(-450, 4000, -925), (-450, 4840, -925), (-500, 4840, -425)],
    ),
    Way::new(
        "the hammer, by the hovering stones",
        (-950, 4000, 450),
        &[(-925, 4270, 25), (-925, 4540, -375), (-500, 4840, -425)],
    ),
    Way::new(
        "the horn, across the gap",
        (-950, 4000, 450),
        &[(-2050, 3900, 550)],
    ),
];

const ZIGGURAT_WAYS: [Way; 3] = [
    Way::new(
        "the crown, by the dais",
        (-650, 4000, 725),
        &[(-250, 4010, 725), (-250, 4840, 725), (0, 4840, 0)],
    ),
    Way::new(
        "the crown, up the tiers",
        (-950, 4000, 600),
        &[(-650, 4270, 450), (-400, 4540, 300), (0, 4840, 0)],
    ),
    Way::new(
        "the stele, jumped down to",
        (-500, 4000, 700),
        &[(-500, 3700, 1900)],
    ),
];

const CALDERA_WAYS: [Way; 3] = [
    Way::new(
        "the plug, by the lift",
        (100, 4000, -900),
        &[(100, 4000, -1325), (100, 4840, -1325), (100, 4840, -800)],
    ),
    Way::new(
        "the plug, from the lip",
        (-750, 4000, -750),
        &[
            (600, 4000, -750),
            (1025, 4200, -800),
            (950, 4540, -575),
            (500, 4840, -600),
        ],
    ),
    Way::new(
        "the ejecta island",
        (-750, 4000, -750),
        &[(-1000, 4000, 0), (-2050, 3800, 0)],
    ),
];

const TWIN_ISLES_WAYS: [Way; 4] = [
    Way::new(
        "the arch, by the seracs",
        (-1000, 4000, 300),
        &[(-875, 4270, -375), (-625, 4540, -360), (-300, 4840, 0)],
    ),
    Way::new(
        "over the bridge",
        (-900, 4000, 0),
        &[(0, 4000, 0), (700, 4000, 0)],
    ),
    Way::new("the skerry", (-1000, 4000, 300), &[(-2150, 3900, 50)]),
    Way::new("the hung stone", (-700, 4000, 550), &[(0, 4380, 550)]),
];
