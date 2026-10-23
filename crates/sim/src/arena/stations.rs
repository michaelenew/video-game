//! **The stations**: the creatures of odd ground as the middles of jump
//! sections (2026-10-22).
//!
//! The owner: "remove the very obvious little areas off the valley. I don't
//! like how everything is a semioptional little fight and super spread out.
//! For the ones that are odd terrain, make them midpoints in some mobility
//! sections." So the six creatures whose arenas are ground of their own --
//! the Mire's peat and braziers, the Pan's sand, the Hollows' cave, the
//! Cliffs' plateau over a drop, the Ashwood's snow and ash, the Shrine's
//! court of columns -- are no longer clearings at the end of side paths.
//! Each is **raised over the land beside the road**, its rim's cliff all
//! round it and a ledge at its door (`valley::land::Pad::ledge`), and a
//! route of stones climbs to that ledge from the road and another leaves it
//! for somewhere else:
//!
//! | Station | Up to it | On from it |
//! | --- | --- | --- |
//! | **The Mire** | the Shelves' road | the Pan |
//! | **The Pan** | the Mire | down to the Shelves' road by the climb |
//! | **The Hollows** | the Pinewood's road | over the Waterfall to the road above it |
//! | **The Cliffs** | across the ravine from the Saddle's road | the Shrine |
//! | **The Shrine** | the Cliffs | down to the Saddle's end |
//! | **The Ashwood** | the Saddle's road | a summit with a view |
//!
//! Every route is a table of hops (`heights::Hop`) laid from where it
//! starts, the way the Hanging Stones are, and held to the same rules by
//! `tests/stations.rs`: every plain hop inside every mover's plain catch, and
//! jumped. They are sized for the movers (`class::MOVERS`), not the Bulwark.
//! A station's fight is the arena it was tuned in, floor and all; what is
//! new is only what is round it.

use super::heights::{Hop, Need, Stone, Way, lay};
use crate::arena::{Material, Solid};
use crate::drift::Drift;
use crate::fixed::Fx;

use Way::{E, N, S, W};

// ---- Where each station is --------------------------------------------------

/// The Mire's origin on the map (centimetres), twelve metres over the
/// Shelves' road south of it.
pub const MIRE_AT: [i32; 3] = [51500, 5700, -10100];
/// The Pan's, east of the Mire and higher.
pub const PAN_AT: [i32; 3] = [56800, 6300, -9900];
/// The Hollows', south of the Pinewood's road.
pub const HOLLOWS_AT: [i32; 3] = [76500, 7150, -6800];
/// The Cliffs', sunk in their ravine north of the Saddle's road, the
/// plateau's top at the road's height.
pub const CLIFFS_AT: [i32; 3] = [90500, 7300, 7400];
/// The Ashwood's, south of the Saddle's road.
pub const ASHWOOD_AT: [i32; 3] = [93500, 9800, -8600];
/// The Shrine's, on the Saddle's north-east slope, thirty-five metres over
/// the road's end.
pub const SHRINE_AT: [i32; 3] = [99500, 12500, 8000];

/// **Low ground round a station** that the valley does not already have:
/// a pocket of meadow at the road's height, so the station stands up out
/// of it as a table of rock seen from the road, not sunk in a mountainside.
/// Ways of the land that nobody treads (`valley::land::Way`).
pub const MEADOWS: [[crate::valley::land::Point; 2]; 1] = [
    // The Hollows', south of the Pinewood's road.
    [
        crate::valley::land::at(76500, -2500, 6000, 2800),
        crate::valley::land::at(76500, -8500, 6000, 2800),
    ],
];

/// How far out from its door a station's ledge is reckoned to be (its
/// middle) and how far either side, for the hop on to it: well inside the
/// ledge the land cuts (`land::LEDGE`).
pub const LEDGE_OUT: i32 = 650;
const LEDGE_HALF: i32 = 400;

/// A ledge as a stone of a route: its middle [`LEDGE_OUT`] from a door on a
/// side, `across` along the wall (to the right, looking out).
const fn ledge(door: (i32, i32), top: i32, way: Way, across: i32) -> Stone {
    let (dx, dz) = way.step();
    let (rx, rz) = (dz, -dx);
    Stone {
        x: door.0 + dx * LEDGE_OUT + rx * across,
        z: door.1 + dz * LEDGE_OUT + rz * across,
        top,
        half: LEDGE_HALF,
        grounded: true,
    }
}

/// How far either side of the middle of a ledge the route up arrives and
/// the route on leaves, so the two do not meet.
const SIDE: i32 = 500;

/// **Each station's ledge**, as the stone a route arrives on and leaves
/// from: outside its door, at its floor -- its middle, and the two places
/// either side of it the routes use. The doors are the middles of the
/// rims' sides the layout cuts them in (`valley::layout::ROOMS`).
const MIRE_DOOR: (i32, i32) = (MIRE_AT[0] + 400, MIRE_AT[2] + 1900);
pub const MIRE_LEDGE: Stone = ledge(MIRE_DOOR, MIRE_AT[1], N, 0);
const PAN_DOOR: (i32, i32) = (PAN_AT[0], PAN_AT[2] + 1900);
pub const PAN_LEDGE: Stone = ledge(PAN_DOOR, PAN_AT[1], N, 0);
const HOLLOWS_DOOR: (i32, i32) = (HOLLOWS_AT[0], HOLLOWS_AT[2] + 1450);
pub const HOLLOWS_LEDGE: Stone = ledge(HOLLOWS_DOOR, HOLLOWS_AT[1], N, 0);
const ASHWOOD_DOOR: (i32, i32) = (ASHWOOD_AT[0], ASHWOOD_AT[2] + 1800);
pub const ASHWOOD_LEDGE: Stone = ledge(ASHWOOD_DOOR, ASHWOOD_AT[1], N, 0);
const SHRINE_DOOR: (i32, i32) = (SHRINE_AT[0] - 1650, SHRINE_AT[2]);
pub const SHRINE_LEDGE: Stone = ledge(SHRINE_DOOR, SHRINE_AT[1], W, 0);
/// The Cliffs' is the south edge of the plateau's top, twelve metres over
/// the arena's floor.
pub const CLIFFS_LEDGE: Stone = Stone {
    x: CLIFFS_AT[0],
    z: CLIFFS_AT[2] - 2200 + LEDGE_HALF,
    top: CLIFFS_AT[1] + 1200,
    half: LEDGE_HALF,
    grounded: true,
};

/// The ledge of a station, by its arena.
pub const fn ledge_of(id: crate::arena::ArenaId) -> Stone {
    use crate::arena::ArenaId as A;
    match id {
        A::MIREBACK => MIRE_LEDGE,
        A::SANDMAW => PAN_LEDGE,
        A::BROODMOTHER => HOLLOWS_LEDGE,
        A::GALEWING => CLIFFS_LEDGE,
        A::VEILSTALKER => ASHWOOD_LEDGE,
        _ => SHRINE_LEDGE,
    }
}

/// **The Cliffs' door**: where the stones across the ravine arrive, a strip
/// along the plateau's south lip at its top. Open; it is a doorway only to
/// say where the place begins.
pub const CLIFFS_LIP: Solid = Solid::cm(
    [CLIFFS_AT[0] - 300, CLIFFS_AT[1] + 1140, CLIFFS_AT[2] - 2250],
    [CLIFFS_AT[0] + 300, CLIFFS_AT[1] + 1200, CLIFFS_AT[2] - 2150],
    Material::Wood,
);

/// **The fourth waystone's door**: across the Shrine's whole ledge, so that
/// while it is dark nobody steps over the court's low wall beside the
/// doorway either. `cut` is the doorway the layout cuts through its west
/// wall; the gate stands just outside it, along the wall.
pub fn shrine_gate(cut: &Solid) -> Solid {
    let wide = crate::valley::land::LEDGE;
    let z = crate::math::midpoint(cut.min.z, cut.max.z);
    Solid {
        min: crate::math::V3::new(cut.min.x.sub(Fx::ratio(3, 2)), cut.min.y, z.sub(wide)),
        max: crate::math::V3::new(cut.min.x, cut.max.y, z.add(wide)),
        material: Material::Stone,
        shape: crate::arena::Shape::Box,
    }
}

// ---- The routes --------------------------------------------------------------

/// A stone of the ordinary size, three metres across, a plain jump away.
const fn on(way: Way, gap: i32, rise: i32) -> Hop {
    Hop::Stone {
        half: 150,
        gap,
        rise,
        way,
        need: Need::Jump,
    }
}

/// A precision stone, a metre across.
const fn pin(way: Way, gap: i32, rise: i32) -> Hop {
    Hop::Stone {
        half: 50,
        gap,
        rise,
        way,
        need: Need::Jump,
    }
}

/// A rest, six metres across, with a cairn.
const fn rest(way: Way, gap: i32, rise: i32) -> Hop {
    Hop::Stone {
        half: super::heights::REST,
        gap,
        rise,
        way,
        need: Need::Jump,
    }
}

/// A ledge as the last hop of a route: its half-side, a plain jump away.
const fn onto(way: Way, gap: i32, rise: i32) -> Hop {
    Hop::Stone {
        half: LEDGE_HALF,
        gap,
        rise,
        way,
        need: Need::Jump,
    }
}

/// **Where a route has to start** to end on `to`: laid from nothing, and
/// moved so its last stone is `to`.
pub const fn ending_on<const K: usize>(to: Stone, half: i32, hops: &[Hop; K]) -> Stone {
    let zero = Stone {
        x: 0,
        z: 0,
        top: 0,
        half,
        grounded: true,
    };
    let laid = lay(zero, hops, 0);
    let last = laid[K - 1];
    Stone {
        x: to.x - last.x,
        z: to.z - last.z,
        top: to.top - last.top,
        half,
        grounded: true,
    }
}

/// **A route of a station**: its name, the stone it starts from, its hops,
/// and what it ends on -- a station's ledge, or the land.
pub struct Route {
    pub name: &'static str,
    pub from: Stone,
    pub hops: &'static [Hop],
    pub stones: &'static [Stone],
    /// It ends on something already there -- a ledge, the plateau, the
    /// Waterfall's buttress -- rather than a stone of its own.
    pub lands: bool,
}

/// The top of the Waterfall's south buttress (`waterfall::PIECES`): where
/// the Hollows' route on arrives, a step over the top of the falls.
pub const BUTTRESS: Stone = Stone {
    x: super::waterfall::VALLEY_AT[0] + 350,
    z: -3200,
    top: super::waterfall::VALLEY_AT[1] + 2600,
    half: 350,
    grounded: true,
};

/// A hop on to the Waterfall's buttress, its half-side.
const fn onto_buttress(way: Way, gap: i32, rise: i32) -> Hop {
    Hop::Stone {
        half: 350,
        gap,
        rise,
        way,
        need: Need::Jump,
    }
}

/// A lift along `way`, `rise` up (`heights::Hop::Lift`).
const fn lift(way: Way, rise: i32) -> Hop {
    Hop::Lift { way, rise }
}

// ---- The Shelves: up to the Mire, across to the Pan, and down -------------

/// **Up to the Mire**, from the Shelves' road: six hops, twelve metres, to
/// the west of its ledge.
pub const MIRE_UP_HOPS: [Hop; 6] = [
    on(W, 450, 300),
    pin(S, 400, 250),
    on(W, 450, 200),
    pin(S, 450, 150),
    on(E, 400, 0),
    onto(S, 350, 200),
];
pub const MIRE_UP_FROM: Stone =
    ending_on(ledge(MIRE_DOOR, MIRE_AT[1], N, -SIDE), 200, &MIRE_UP_HOPS);
pub const MIRE_UP: [Stone; 6] = lay(MIRE_UP_FROM, &MIRE_UP_HOPS, 0);

/// **From the Mire to the Pan**: north off the Mire's ledge, east along the
/// cliffs' feet over the low ground with a rest halfway, and south on to the
/// Pan's ledge, six metres higher.
pub const MIRE_TO_PAN_HOPS: [Hop; 10] = [
    on(N, 400, 100),
    pin(E, 450, 250),
    on(E, 500, 150),
    pin(N, 450, -100),
    rest(E, 400, 200),
    pin(E, 450, 200),
    pin(E, 400, 100),
    on(E, 300, -100),
    pin(S, 400, -50),
    onto(S, 350, -150),
];
pub const MIRE_TO_PAN_FROM: Stone = ledge(MIRE_DOOR, MIRE_AT[1], N, SIDE);
pub const MIRE_TO_PAN: [Stone; 10] = lay(MIRE_TO_PAN_FROM, &MIRE_TO_PAN_HOPS, 0);

/// **Down from the Pan** to the Shelves' road, east of it: or up to it,
/// the other way.
pub const PAN_DOWN_HOPS: [Hop; 7] = [
    on(N, 400, 100),
    pin(N, 450, -200),
    on(E, 550, -250),
    pin(E, 500, -150),
    on(N, 500, -250),
    pin(E, 450, -200),
    rest(N, 500, -300),
];
pub const PAN_DOWN_FROM: Stone = ledge(PAN_DOOR, PAN_AT[1], N, SIDE);
pub const PAN_DOWN: [Stone; 7] = lay(PAN_DOWN_FROM, &PAN_DOWN_HOPS, 1);

// ---- The Pinewood: up to the Hollows, and on over the Waterfall ------------

/// **Up to the Hollows**, from the Pinewood's road across the meadow.
pub const HOLLOWS_UP_HOPS: [Hop; 6] = [
    on(S, 450, 300),
    pin(W, 400, 250),
    on(S, 450, 200),
    pin(E, 450, 150),
    on(E, 400, 0),
    onto(S, 500, 150),
];
pub const HOLLOWS_UP_FROM: Stone = ending_on(
    ledge(HOLLOWS_DOOR, HOLLOWS_AT[1], N, -SIDE),
    200,
    &HOLLOWS_UP_HOPS,
);
pub const HOLLOWS_UP: [Stone; 6] = lay(HOLLOWS_UP_FROM, &HOLLOWS_UP_HOPS, 0);

/// **On from the Hollows, over the Waterfall**: north off the ledge and
/// east, rising fourteen metres, to the top of the falls' south buttress --
/// a way past the Waterfall's climb, to the road above it.
pub const HOLLOWS_ON_HOPS: [Hop; 7] = [
    pin(N, 450, 100),
    on(E, 550, 250),
    pin(N, 400, 250),
    on(E, 500, 300),
    pin(E, 500, 250),
    on(E, 350, 0),
    onto_buttress(E, 50, 300),
];
pub const HOLLOWS_ON_FROM: Stone = ledge(HOLLOWS_DOOR, HOLLOWS_AT[1], N, SIDE);
pub const HOLLOWS_ON: [Stone; 7] = lay(HOLLOWS_ON_FROM, &HOLLOWS_ON_HOPS, 0);

// ---- The Saddle: the Cliffs, the Shrine, the Ashwood -----------------------

/// **Across the ravine to the Cliffs**: precision stones over a twelve-metre
/// drop, at the road's height, from the road to the plateau's lip.
pub const CLIFFS_UP_HOPS: [Hop; 7] = [
    on(N, 450, 0),
    on(N, 450, 0),
    pin(N, 450, 0),
    pin(E, 500, 0),
    on(N, 450, -50),
    pin(W, 500, 0),
    onto(N, 350, 0),
];
pub const CLIFFS_UP_FROM: Stone = ending_on(CLIFFS_LEDGE, 200, &CLIFFS_UP_HOPS);
pub const CLIFFS_UP: [Stone; 7] = lay(CLIFFS_UP_FROM, &CLIFFS_UP_HOPS, 0);

/// Where the route on leaves the Cliffs: the plateau's east edge.
pub const CLIFFS_EAST: Stone = Stone {
    x: CLIFFS_AT[0] + 2200 - LEDGE_HALF,
    z: CLIFFS_AT[2] + 100,
    top: CLIFFS_AT[1] + 1200,
    half: LEDGE_HALF,
    grounded: true,
};

/// **From the Cliffs to the Shrine**: off the plateau's east edge over the
/// ravine to a rest, a lift thirty metres up, and three hops on to the
/// Shrine's ledge -- forty metres over the plateau.
pub const CLIFFS_TO_SHRINE_HOPS: [Hop; 6] = [
    pin(E, 450, 100),
    rest(E, 400, 100),
    lift(E, 3000),
    pin(E, 450, 250),
    on(E, 450, 250),
    onto(E, 330, 300),
];
pub const CLIFFS_TO_SHRINE: [Stone; 6] = lay(CLIFFS_EAST, &CLIFFS_TO_SHRINE_HOPS, 0);

/// **Up to the Ashwood**, from the Saddle's road.
pub const ASHWOOD_UP_HOPS: [Hop; 6] = [
    on(W, 450, 250),
    pin(S, 450, 250),
    on(E, 500, 200),
    pin(S, 400, 250),
    on(W, 450, 0),
    onto(S, 500, 150),
];
pub const ASHWOOD_UP_FROM: Stone = ending_on(
    ledge(ASHWOOD_DOOR, ASHWOOD_AT[1], N, -SIDE),
    200,
    &ASHWOOD_UP_HOPS,
);
pub const ASHWOOD_UP: [Stone; 6] = lay(ASHWOOD_UP_FROM, &ASHWOOD_UP_HOPS, 0);

/// **On from the Ashwood**, east and down to the road where it bends north.
pub const ASHWOOD_ON_HOPS: [Hop; 5] = [
    pin(N, 500, 100),
    on(E, 450, -100),
    pin(N, 450, -150),
    on(E, 500, -200),
    rest(N, 450, -250),
];
pub const ASHWOOD_ON_FROM: Stone = ledge(ASHWOOD_DOOR, ASHWOOD_AT[1], N, SIDE);
pub const ASHWOOD_ON: [Stone; 5] = lay(ASHWOOD_ON_FROM, &ASHWOOD_ON_HOPS, 1);

/// **Every route**, in the order the road meets them.
pub const ROUTES: [Route; 9] = [
    Route {
        name: "up to the Mire",
        from: MIRE_UP_FROM,
        hops: &MIRE_UP_HOPS,
        stones: &MIRE_UP,
        lands: true,
    },
    Route {
        name: "from the Mire to the Pan",
        from: MIRE_TO_PAN_FROM,
        hops: &MIRE_TO_PAN_HOPS,
        stones: &MIRE_TO_PAN,
        lands: true,
    },
    Route {
        name: "down from the Pan",
        from: PAN_DOWN_FROM,
        hops: &PAN_DOWN_HOPS,
        stones: &PAN_DOWN,
        lands: false,
    },
    Route {
        name: "up to the Hollows",
        from: HOLLOWS_UP_FROM,
        hops: &HOLLOWS_UP_HOPS,
        stones: &HOLLOWS_UP,
        lands: true,
    },
    Route {
        name: "on from the Hollows",
        from: HOLLOWS_ON_FROM,
        hops: &HOLLOWS_ON_HOPS,
        stones: &HOLLOWS_ON,
        lands: true,
    },
    Route {
        name: "across to the Cliffs",
        from: CLIFFS_UP_FROM,
        hops: &CLIFFS_UP_HOPS,
        stones: &CLIFFS_UP,
        lands: true,
    },
    Route {
        name: "from the Cliffs to the Shrine",
        from: CLIFFS_EAST,
        hops: &CLIFFS_TO_SHRINE_HOPS,
        stones: &CLIFFS_TO_SHRINE,
        lands: true,
    },
    Route {
        name: "up to the Ashwood",
        from: ASHWOOD_UP_FROM,
        hops: &ASHWOOD_UP_HOPS,
        stones: &ASHWOOD_UP,
        lands: true,
    },
    Route {
        name: "on from the Ashwood",
        from: ASHWOOD_ON_FROM,
        hops: &ASHWOOD_ON_HOPS,
        stones: &ASHWOOD_ON,
        lands: false,
    },
];

/// **Where each station's way up starts**, for the layout (centimetres: x,
/// z, and the road's floor there): the foot of its route, or the route it
/// is reached by.
pub const MIRE_FOOT: (i32, i32, i32) = foot(&MIRE_UP_FROM);
pub const PAN_FOOT: (i32, i32, i32) = foot(&PAN_DOWN[PAN_DOWN.len() - 1]);
pub const HOLLOWS_FOOT: (i32, i32, i32) = foot(&HOLLOWS_UP_FROM);
pub const CLIFFS_FOOT: (i32, i32, i32) = foot(&CLIFFS_UP_FROM);
pub const ASHWOOD_FOOT: (i32, i32, i32) = foot(&ASHWOOD_UP_FROM);
pub const SHRINE_FOOT: (i32, i32, i32) = foot(&CLIFFS_UP_FROM);

const fn foot(s: &Stone) -> (i32, i32, i32) {
    (s.x, s.z, s.top - 100)
}

fn cm(v: i32) -> Fx {
    super::cm(v)
}

/// **A stone's solid**: from the land up where the land is close under its
/// top, a slab where it is not. Rock.
fn stone(s: &Stone, ground: Fx, out: &mut Vec<Solid>) {
    let thick = 120 + s.half / 3;
    let top = cm(s.top);
    let slab = top.sub(cm(thick));
    let bottom = if s.grounded || ground.raw() > slab.sub(Fx::ratio(1, 2)).raw() {
        ground.sub(Fx::ONE).min(slab)
    } else {
        slab
    };
    out.push(Solid {
        min: crate::math::V3::new(cm(s.x - s.half), bottom, cm(s.z - s.half)),
        max: crate::math::V3::new(cm(s.x + s.half), top, cm(s.z + s.half)),
        material: Material::Rock,
        shape: crate::arena::Shape::Box,
    });
}

/// A cairn on a top at `y`.
fn cairn_on(x: i32, z: i32, y: i32) -> Solid {
    Solid::cm(
        [x - 100, y - 30, z - 100],
        [x + 100, y + 40, z + 100],
        Material::Snow,
    )
}

/// **The stations' routes on the map**: every stone a route lays but the
/// ledge or the buttress it ends on, which is already there; a cairn on
/// every rest and on the stone each route starts from by the road; and the
/// lifts. `height` is the land's.
pub fn build(height: &dyn Fn(Fx, Fx) -> Fx, out: &mut Vec<Solid>, moving: &mut Vec<Drift>) {
    let mut seen: Vec<(i32, i32, i32)> = Vec::new();
    for r in &ROUTES {
        let built = if r.lands {
            &r.stones[..r.stones.len() - 1]
        } else {
            r.stones
        };
        // The stone it starts from, if it is not a ledge or the plateau.
        let first = (r.from.half < LEDGE_HALF).then_some(&r.from);
        for (k, s) in first.into_iter().chain(built.iter()).enumerate() {
            // Two routes sharing a stone build it once.
            if seen.contains(&(s.x, s.z, s.top)) {
                continue;
            }
            seen.push((s.x, s.z, s.top));
            stone(s, height(cm(s.x), cm(s.z)), out);
            if s.half >= super::heights::REST || (k == 0 && first.is_some()) {
                out.push(cairn_on(s.x, s.z, s.top));
            }
        }
        // Every lift's cars; the rest at its head is a stone of the route.
        moving.extend(super::heights::drifts_of(r.from, r.hops, r.stones));
    }
}
