//! **The Kilnworks** (2026-11-03, `crate::versus::Kind::Front`,
//! `docs/design/versus.md`, `docs/design/fight-spaces.md`): the first
//! five-point map. A brick works in ruins, 238 by 80 metres, run west to
//! east: each side's loading dock, a clay pit, a kiln court behind the works
//! wall, and the clay yard in the middle that both sides fight over.

#![allow(unused_imports)]

use super::hills::{round, slab, turned, whole};
use super::rim::Rim;
use super::{Area, Arena, ArenaId, Bounds, Mark, Material, Region, Solid, Spawns, cm};
use crate::drift::Drift;
use crate::fixed::Fx;
use crate::math::V3;
use crate::valley::Vent;
use crate::versus::{Hill, Kind, Line, Stage, Way};

use Material::{Ash, Grass, Ground, Peat, Rock, Sand, Snow, Stone, Water, Wood};

/// **The Kilnworks**: five points in a row, forty-eight metres apart, from
/// the west side's loading dock to the east side's. Described from the
/// west; the east half is it turned half a turn about the middle.
///
/// **The clay yard** (the middle point): open ground for the big fight, the
/// point on a low slab. North and south of it stand **the drying sheds**,
/// long open-fronted roofs on brick piers seven metres up, with a back wall
/// to put a shoulder to; **the gantry**, a plank catwalk, crosses from one
/// shed roof to the other straight over the point -- the one overpass, and
/// the point's column stops under it. **A kiln chimney** sixteen metres
/// tall stands off each side of the yard, an updraft up its back: the
/// yard's top perch. Stacks of brick stand about the yard for cover.
///
/// **The works wall** (the choke): eight metres of brick across the whole
/// map twenty-six metres west of the point, taller than any jump, with
/// three ways through. **The main gate**, ten metres of open floor, its
/// lintel a timber bridge; **the gatehouse passage**, a four-metre tunnel
/// through the gatehouse beside it, with **a slot** in its roof to drop
/// through from above, and a window sill on the yard's side to perch on;
/// and **the rampart**, the wall's walkable top, reached
/// from the yard by an updraft up its face and from the court by the drying
/// house's roof. Defenders cannot hold all three at once.
///
/// **The kiln court** (the west side's second point): a courtyard ringed
/// by roofs seven metres up -- **the clay store** and **the kiln hall** in
/// an L on its west and north, **the engine house** at its south-west,
/// **the drying house** against the wall -- and over them **the great
/// kiln**, a bottle kiln fourteen and a half metres tall twenty metres from
/// the point: the strong point, aimed down at it. It is climbed by **the
/// hoist**, a cage that rises up its west face from the floor, and by a
/// draft up its side from the drying house's roof. The rampart reaches the
/// kiln hall's roof by **a plank** over the back lane.
///
/// **The clay pit** (between the court and the dock): twenty metres of open
/// ground the width of the map, drying racks and brick pillars to fight
/// through, crossed seven metres up by **the trestle** on the south -- from
/// the engine house's roof to the dock's south balcony -- and on the north
/// by **the tram**, a cart that runs the rail between the clay store's roof
/// and the dock's north balcony over a gap nobody jumps.
///
/// **The loading dock** (the west side's last point): the point on a raised
/// dock under **the great shed roof**, fourteen metres up, with
/// **balconies** along its inside seven metres up for its defenders, reached
/// by the trestle and the tram from the pit, by an updraft from the dock,
/// and by steps and an updraft up **the back wall** from the start. The back wall has
/// **three doors** from the start onto the dock, so nobody camps one.
pub static ARENA: Arena = Arena {
    id: ArenaId::KILNWORKS,
    name: "Kilnworks",
    creature: None,
    bounds: Bounds::cm((-11900, 11900), (-4000, 4000)),
    floor: Ground,
    regions: &REGIONS,
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-11300, 0, (0, 0)), Mark::cm(11300, 0, (0, 0))],
        hunt: None,
    },
    sites: &[],
    // A tall bank: the kiln tops and the chimneys are fifteen metres up and
    // more, and the walls run into it.
    rim: Some(Rim::new((-11900, 11900), (-4000, 4000), [0, 0, 0, 0], 3000)),
};

pub static LINE: Line = Line {
    points: [
        // The loading dock: on the raised dock, under the great shed roof.
        Hill::cm(-9600, 120, 0, 400, 50, 1000),
        // The kiln court.
        Hill::cm(-4800, 12, 0, 400, 50, 1000),
        // The clay yard: under the gantry.
        Hill::cm(0, 12, 0, 400, 50, 640),
        Hill::cm(4800, 12, 0, 400, 50, 1000),
        Hill::cm(9600, 120, 0, 400, 50, 1000),
    ],
    spawns: [
        // The start, behind the dock's back wall.
        Mark::cm(-11300, 0, (0, 0)),
        // In the clay pit, behind the clay store, out of the court's sight.
        Mark::cm(-7600, 0, (-4800, 0)),
        // In the lane behind the works wall, out of the yard's sight.
        Mark::cm(-3400, 2600, (0, 2600)),
        // The east half's lane behind its works wall, and its clay pit.
        Mark::cm(3400, -2600, (4800, -2600)),
        Mark::cm(7600, 0, (9600, 0)),
    ],
};

pub static STAGE: Stage = Stage {
    arena: ArenaId::KILNWORKS,
    kind: Kind::Front,
    theme: "A brick works in ruins: five points from loading dock to loading dock",
    pit: None,
    hill: None,
    front: Some(&LINE),
    vents: &VENTS,
    drifts: &DRIFTS,
    ways: &WAYS,
};

const REGIONS: [Region; 12] = [
    // The clay yard, trodden to sand round the point.
    Region {
        area: Area::disc_cm(0, 0, 1100),
        material: Sand,
    },
    // Ash under each chimney and round each great kiln.
    Region {
        area: Area::disc_cm(-1300, -1300, 450),
        material: Ash,
    },
    Region {
        area: Area::disc_cm(1300, 1300, 450),
        material: Ash,
    },
    Region {
        area: Area::disc_cm(-4800, -2200, 1000),
        material: Ash,
    },
    Region {
        area: Area::disc_cm(4800, 2200, 1000),
        material: Ash,
    },
    // The kiln courts, paved.
    Region {
        area: Area::rect_cm((-6000, -3400), (-1600, 1600)),
        material: Stone,
    },
    Region {
        area: Area::rect_cm((3400, 6000), (-1600, 1600)),
        material: Stone,
    },
    // The clay pits, wet clay.
    Region {
        area: Area::rect_cm((-8600, -6800), (-4000, 4000)),
        material: Peat,
    },
    Region {
        area: Area::rect_cm((6800, 8600), (-4000, 4000)),
        material: Peat,
    },
    // The starts, behind the docks.
    Region {
        area: Area::rect_cm((-11900, -10800), (-4000, 4000)),
        material: Sand,
    },
    Region {
        area: Area::rect_cm((10800, 11900), (-4000, 4000)),
        material: Sand,
    },
    // The gate's mouth, worn.
    Region {
        area: Area::rect_cm((-3400, 3400), (-1500, -500)),
        material: Ground,
    },
];

const SOLIDS: [Solid; 2 + 2 * HALF] = whole(
    [
        // The clay yard's point, on a slab a hand high.
        Solid::cm([-500, 0, -500], [500, 12, 500], Stone),
        // The gantry: from the north shed's roof to the south's, straight
        // over the point.
        Solid::cm([-150, 660, -1900], [150, 700, 1900], Wood),
    ],
    [
        // ---- The clay yard (the north shed; its turn is the south's).
        // The north drying shed: its roof seven metres up, its back wall,
        // and the brick piers along its open front.
        Solid::cm([-1600, 650, 1900], [1600, 700, 3100], Wood),
        Solid::cm([-1600, 0, 3000], [1600, 650, 3100], Stone),
        Solid::cm([-1600, 0, 1900], [-1500, 650, 2000], Stone),
        Solid::cm([-550, 0, 1900], [-450, 650, 2000], Stone),
        Solid::cm([450, 0, 1900], [550, 650, 2000], Stone),
        Solid::cm([1500, 0, 1900], [1600, 650, 2000], Stone),
        // Crates stacked up to its roof at its west end.
        Solid::cm([-2150, 0, 2300], [-1850, 240, 2900], Wood),
        Solid::cm([-1850, 0, 2300], [-1600, 480, 2900], Wood),
        // The chimney, sixteen metres, off the yard's south-west.
        round(-1300, -1300, (0, 1600), (250, 200), Stone),
        // Stacks of brick about the yard: cover, taller than a jump.
        Solid::cm([-725, 0, 575], [-575, 450, 725], Stone),
        Solid::cm([-1175, 0, 225], [-1025, 450, 375], Stone),
        Solid::cm([-475, 0, -1475], [-325, 450, -1325], Stone),
        // ---- The works wall, twenty-six metres west of the point.
        // Its south length, to the gate.
        Solid::cm([-3000, 0, -4100], [-2600, 800, -1500], Stone),
        // Its north length, from the gatehouse.
        Solid::cm([-3000, 0, 1500], [-2600, 800, 4100], Stone),
        // The gate's lintel: a timber bridge along the rampart.
        Solid::cm([-3000, 750, -1500], [-2600, 800, -500], Wood),
        // The gatehouse, either side of its passage, and the passage's roof
        // either side of the slot, which opens on the yard's side of the
        // rampart.
        Solid::cm([-3400, 0, -500], [-2200, 800, 300], Stone),
        Solid::cm([-3400, 0, 700], [-2200, 800, 1500], Stone),
        Solid::cm([-3400, 450, 300], [-2500, 800, 700], Stone),
        Solid::cm([-2300, 450, 300], [-2200, 800, 700], Stone),
        // The gatehouse's window sill on the yard's side, a perch over the
        // gate's mouth for whoever drops onto it from the roof.
        Solid::cm([-2200, 420, 800], [-1900, 470, 1400], Wood),
        // ---- The kiln court.
        // The clay store on its west and the kiln hall on its north, an L
        // of roofs seven metres up.
        Solid::cm([-6800, 0, -1000], [-6000, 700, 2800], Stone),
        Solid::cm([-6000, 0, 1600], [-3800, 700, 2800], Stone),
        // The engine house at its south-west.
        Solid::cm([-6800, 0, -3000], [-6000, 700, -1600], Stone),
        // The great kiln, fourteen and a half metres: the strong point.
        round(-4800, -2200, (0, 1450), (600, 500), Stone),
        // The drying house against the wall, its roof a step under the
        // rampart.
        Solid::cm([-4200, 0, -3000], [-3000, 700, -1600], Stone),
        // Steps up to the drying house's roof from the south lane.
        Solid::cm([-4200, 0, -3500], [-3700, 240, -3000], Stone),
        Solid::cm([-3700, 0, -3500], [-3200, 480, -3000], Stone),
        // Steps up to the engine house's roof from the south lane.
        Solid::cm([-6800, 0, -3500], [-6400, 240, -3000], Stone),
        Solid::cm([-6400, 0, -3500], [-6000, 480, -3000], Stone),
        // Steps up to the kiln hall's roof from the back lane.
        Solid::cm([-5400, 0, 2800], [-5000, 240, 3400], Stone),
        Solid::cm([-5000, 0, 2800], [-4600, 480, 3400], Stone),
        // The plank from the rampart to the kiln hall's roof, over the lane.
        Solid::cm([-3800, 750, 2000], [-3000, 800, 2400], Wood),
        // The court's point, on a slab a hand high, and cover round it: a
        // low stack to hop, a kiln column to hide behind.
        Solid::cm([-5300, 0, -500], [-4300, 12, 500], Stone),
        Solid::cm([-5800, 0, -800], [-5500, 180, -300], Stone),
        round(-4000, 900, (0, 450), (100, 90), Stone),
        // ---- The clay pit.
        // The trestle, from the engine house's roof to the dock's south
        // balcony, and its pier.
        Solid::cm([-8800, 650, -2400], [-6800, 700, -1800], Wood),
        Solid::cm([-7900, 0, -2300], [-7700, 650, -1900], Stone),
        // Drying racks and brick pillars to fight through.
        Solid::cm([-8200, 0, 800], [-7600, 450, 1000], Wood),
        Solid::cm([-8400, 0, -3400], [-7800, 450, -3200], Wood),
        round(-8100, -900, (0, 450), (90, 80), Stone),
        round(-7300, 1800, (0, 450), (90, 80), Stone),
        Solid::cm([-7700, 0, -1900], [-7100, 450, -1700], Wood),
        round(-7800, 3000, (0, 450), (90, 80), Stone),
        // ---- The loading dock.
        // The back wall, seven metres, and its three doors.
        Solid::cm([-10800, 0, -4100], [-10600, 700, -1900], Stone),
        Solid::cm([-10800, 0, -1300], [-10600, 700, -400], Stone),
        Solid::cm([-10800, 0, 400], [-10600, 700, 1300], Stone),
        Solid::cm([-10800, 0, 1900], [-10600, 700, 4100], Stone),
        // The dock, raised a metre, the point on it.
        Solid::cm([-10600, 0, -1300], [-8800, 120, 1300], Stone),
        // Crates left on it: cover to hop behind the point.
        Solid::cm([-10200, 120, -1000], [-9900, 270, -600], Wood),
        // The great shed roof, fourteen metres up, on its columns.
        Solid::cm([-10800, 1400, -2600], [-8600, 1450, 2600], Wood),
        Solid::cm([-10600, 0, 2400], [-10400, 1400, 2600], Stone),
        Solid::cm([-10600, 0, -2600], [-10400, 1400, -2400], Stone),
        Solid::cm([-8800, 0, 2400], [-8600, 1400, 2600], Stone),
        Solid::cm([-8800, 0, -2600], [-8600, 1400, -2400], Stone),
        // The balconies along its inside, seven metres up.
        Solid::cm([-10600, 650, 1500], [-8600, 700, 2400], Wood),
        Solid::cm([-10600, 650, -2400], [-8800, 700, -1500], Wood),
        // Steps up the back wall from the start.
        Solid::cm([-11400, 0, 2400], [-11100, 240, 3000], Stone),
        Solid::cm([-11100, 0, 2400], [-10800, 480, 3000], Stone),
    ],
);

/// How many solids one half has.
const HALF: usize = 58;

/// **The tram**: a cart on the rail across the clay pit's north side,
/// seven metres up, between the clay store's roof and the dock's north
/// balcony. Two seconds at each end, four on the way. A ride is timed, so
/// it is not one of the ways the walker in `tests/versus.rs` walks.
const CART: Solid = Solid::cm([-7100, 600, 1700], [-6800, 700, 2300], Wood);

/// **The hoist**: a cage sunk flush with the floor against the great kiln's
/// west face, rising to its top. A timber stack the height of its rise, so
/// there is never a gap under it.
const CAGE: Solid = Solid::cm([-5650, -1450, -2370], [-5300, 0, -2030], Wood);

const DRIFTS: [Drift; 4] = [
    Drift {
        solid: CART,
        travel: V3::new(cm(-1500), Fx::ZERO, Fx::ZERO),
        wait: 120,
        go: 240,
        stay: 120,
        back: 240,
        phase: 0,
    },
    Drift {
        solid: turned(CART),
        travel: V3::new(cm(1500), Fx::ZERO, Fx::ZERO),
        wait: 120,
        go: 240,
        stay: 120,
        back: 240,
        phase: 0,
    },
    Drift {
        solid: CAGE,
        travel: V3::new(Fx::ZERO, cm(1450), Fx::ZERO),
        wait: 150,
        go: 240,
        stay: 150,
        back: 240,
        phase: 0,
    },
    Drift {
        solid: turned(CAGE),
        travel: V3::new(Fx::ZERO, cm(1450), Fx::ZERO),
        wait: 150,
        go: 240,
        stay: 150,
        back: 240,
        phase: 0,
    },
];

/// A column of air rising from a roof at `from` (centimetres).
const fn draft(x: i32, z: i32, radius: i32, from: i32, top: i32) -> Vent {
    Vent {
        at: (cm(x), cm(z)),
        radius: cm(radius),
        top: cm(top),
        from: cm(from),
    }
}

const VENTS: [Vent; 14] = [
    // Up the chimney's back, onto its top: caught from the floor, or by
    // stepping off the south shed's roof into it.
    Vent::cm(-1300, -1650, 200, 1750),
    Vent::cm(1300, 1650, 200, 1750),
    // Up the north shed's front from the yard, onto its roof.
    Vent::cm(-1000, 1750, 150, 850),
    Vent::cm(1000, -1750, 150, 850),
    // Up the works wall's face from the yard, onto the rampart.
    Vent::cm(-2450, 2600, 150, 950),
    Vent::cm(2450, -2600, 150, 950),
    // Up the kiln hall's face from the court, onto its roof.
    Vent::cm(-4800, 1450, 150, 850),
    Vent::cm(4800, -1450, 150, 850),
    // From the drying house's roof up the great kiln's side.
    draft(-4150, -2200, 150, 700, 1600),
    draft(4150, 2200, 150, 700, 1600),
    // Up from the dock onto its north balcony.
    Vent::cm(-9200, 1380, 150, 850),
    Vent::cm(9200, -1380, 150, 850),
    // Up the back wall's face from the start, onto its south length.
    Vent::cm(-11000, -2700, 150, 850),
    Vent::cm(11000, 2700, 150, 850),
];

/// The west side's ways (the east's are their turn).
const WAYS: [Way; 25] = [
    // ---- Onto the loading dock (point 0).
    Way::new(
        "the open floor: across the clay pit and up onto the dock",
        (-7600, 0, -600),
        &[(-8500, 0, -300), (-9600, 120, 0)],
    ),
    Way::new(
        "from the start through the back wall's middle door onto the dock",
        (-11300, 0, 0),
        &[(-10900, 0, 0), (-9600, 120, 0)],
    ),
    Way::new(
        "the trestle: from the engine house's roof over the pit to the south balcony, down onto the dock",
        (-6400, 700, -2300),
        &[(-7800, 700, -2100), (-9600, 700, -2000), (-9600, 120, 0)],
    ),
    Way::new(
        "out of the start by the back wall's north door, round onto the dock",
        (-11300, 0, 0),
        &[(-11000, 0, 1600), (-10300, 0, 1600), (-9600, 120, 0)],
    ),
    // ---- Onto the kiln court (point 1).
    Way::new(
        "from the pit's spawn round the clay store and into the court onto the point",
        (-7600, 0, 0),
        &[(-7200, 0, -1300), (-5900, 0, -1300), (-4800, 12, 0)],
    ),
    Way::new(
        "the main gate: from the yard through the works wall onto the court's point",
        (-1600, 0, -1000),
        &[(-2800, 0, -1000), (-3800, 0, -1000), (-4800, 12, 0)],
    ),
    Way::new(
        "the gatehouse passage: from the yard through the tunnel onto the court's point",
        (-1800, 0, 500),
        &[(-2800, 0, 500), (-3800, 0, 500), (-4800, 12, 0)],
    ),
    Way::new(
        "the rampart: up the wall's updraft, along the plank to the kiln hall's roof, down onto the point",
        (-2100, 0, 2600),
        &[
            (-2800, 800, 2600),
            (-2800, 800, 2200),
            (-4300, 700, 2200),
            (-4800, 12, 0),
        ],
    ),
    // ---- Onto the clay yard (point 2).
    Way::new(
        "from the spawn behind the wall through the gatehouse passage onto the yard's point",
        (-3400, 0, 2600),
        &[(-3700, 0, 500), (-2000, 0, 500), (0, 12, 0)],
    ),
    Way::new(
        "the main gate: from the court through the works wall onto the yard's point",
        (-4000, 0, -1000),
        &[(-2000, 0, -1000), (0, 12, 0)],
    ),
    Way::new(
        "the south lane: from the court round behind the chimney onto the yard's point",
        (-4000, 0, -1000),
        &[
            (-2000, 0, -1000),
            (-1300, 0, -2600),
            (-300, 0, -1900),
            (0, 12, 0),
        ],
    ),
    Way::new(
        "the high road: along the north shed's roof, down off its front onto the yard's point",
        (-1200, 700, 2600),
        &[(1000, 700, 2500), (1000, 0, 1300), (0, 12, 0)],
    ),
    // ---- Climbs.
    Way::new(
        "the hoist: ridden up the great kiln's face onto its top",
        (-5900, 0, -1300),
        &[
            (-5475, 0, -2200),
            (-5475, 1450, -2200),
            (-4800, 1450, -2200),
        ],
    ),
    Way::new(
        "the drying house's steps, and its draft up onto the great kiln",
        (-4400, 0, -3800),
        &[
            (-3950, 240, -3250),
            (-3450, 480, -3250),
            (-3450, 700, -2600),
            (-4700, 1450, -2200),
        ],
    ),
    Way::new(
        "from the drying house's roof onto the rampart and over the gate",
        (-3450, 700, -2600),
        &[(-2800, 800, -2600), (-2800, 800, -1000), (-2800, 800, 0)],
    ),
    Way::new(
        "the court's updraft onto the kiln hall's roof, and along it to the clay store's",
        (-4800, 0, 1000),
        &[(-4800, 700, 2200), (-6400, 700, 2200), (-6400, 700, 0)],
    ),
    Way::new(
        "the kiln hall's steps from the back lane",
        (-5200, 0, 3800),
        &[(-5200, 240, 3100), (-4800, 480, 3100), (-4800, 700, 2200)],
    ),
    Way::new(
        "the engine house's steps, and along the trestle to the dock",
        (-7000, 0, -3800),
        &[
            (-6600, 240, -3250),
            (-6200, 480, -3250),
            (-6400, 700, -2300),
            (-8400, 700, -2100),
        ],
    ),
    Way::new(
        "the north shed's crates onto its roof, and over the gantry to the south shed",
        (-2000, 0, 3500),
        &[
            (-2000, 240, 2600),
            (-1730, 480, 2600),
            (-1200, 700, 2600),
            (0, 700, 1700),
            (0, 700, -1700),
            (0, 700, -2500),
        ],
    ),
    Way::new(
        "the yard's updraft onto the north shed's roof",
        (-1000, 0, 1000),
        &[(-1000, 700, 2500)],
    ),
    Way::new(
        "up the chimney's updraft onto its top",
        (-1300, 0, -2400),
        &[(-1300, 1600, -1300)],
    ),
    Way::new(
        "off the south shed's roof into the chimney's updraft, onto its top",
        (-1300, 700, -2500),
        &[(-1300, 1600, -1300)],
    ),
    Way::new(
        "the start's steps up the back wall, along it onto the north balcony",
        (-11600, 0, 3500),
        &[
            (-11250, 240, 2700),
            (-10950, 480, 2700),
            (-10700, 700, 2700),
            (-10700, 700, 2000),
            (-9600, 700, 1900),
        ],
    ),
    Way::new(
        "the start's updraft up the back wall, along it onto the south balcony",
        (-11300, 0, -2700),
        &[
            (-10700, 700, -2700),
            (-10700, 700, -2000),
            (-10000, 700, -2000),
        ],
    ),
    Way::new(
        "the dock's updraft onto the north balcony",
        (-9200, 120, 700),
        &[(-9200, 700, 2000)],
    ),
];
