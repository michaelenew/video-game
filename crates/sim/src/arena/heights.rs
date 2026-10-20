//! **The High Plane**: a separate world over the valley (exploration 0019
//! §2c, built 2026-10-15). The owner: "sections that lead up and down to full
//! separate planes with different creatures there. The routes up need to be
//! treacherous floating stones or stones held by vines with big gaps between
//! them, assorted smaller stones for precision jumping" -- and, asked whether
//! the plane above should be a table mountain or islands in the sky, "both".
//!
//! - **The plane** is a table mountain: a place keeping its own floor 300 m up,
//!   north of the Bank behind the valley's mountains, which top out near 190.
//!   Its rim holds the land up to its edge, a lip a metre and a half high, and
//!   the land falls away three in one below it: from the valley it is a flat
//!   top over the ridges. Its creatures are its own ([`crate::valley::ranges`]).
//! - **The Sky Road** ([`sky_road`]) is the islands: off the Needle's dome,
//!   353 m up, a chain of turf islands, stones and precision stones over the
//!   town, the Mouth and the mountains, coming down gently to the plane's
//!   south-west corner. Gaps of seven to twenty-six metres, stones from a
//!   metre and a half across to twenty; a cairn on every big island.
//! - **The Hanging Stones** ([`LADDER`], [`SPAN`]) climb from the Bank's road:
//!   a column of forty hops round a square, then a span north over the
//!   mountains broken three times by **a gully** -- twelve metres of drop
//!   between two walls, crossed by jumping from wall to wall
//!   ([`super::gully`]). Until 2026-10-16 it was stones seven metres apart
//!   with vines between them, which the owner found awful to get stuck in;
//!   until 2026-10-20, fifty stones each 4.2 m over the last and a span of
//!   identical steps, sized for the Bulwark's catch.
//!
//! **Since 2026-10-20 the Hanging Stones are at the Sky Road's difficulty,
//! and two routes are harder still.** The owner, on the Sky Road: "the only
//! thing resembling challenging. I want more jump sections to be closer to
//! that difficulty, and a few that are more difficult still than that. It's
//! an actually fun challenge because there's variety in the jumps and it
//! forces you to learn class's mobility tools." And the Bulwark is no longer
//! a mover (`crate::class::MOVERS`), so nothing here is sized to it. Every
//! route but the Sky Road is now a table of [`Hop`]s, each declaring what it
//! asks ([`Need`]): a plain jump every mover makes -- most at the edge of
//! the shortest jumpers' catch -- or a class's tool, past every plain jump.
//! A rest with a cairn stands close before every tool hop of the Hanging
//! Stones. Both harder routes are optional:
//!
//! - **The Crown** ([`CROWN`]): off the plane's floor to a turf island fifty
//!   metres over it, the highest place in the valley: fifteen- to
//!   twenty-two-metre hops onto one- and two-metre stones, and two rises of
//!   ten, one rest halfway.
//! - **The High Line** ([`high_line`]): a shortcut across the Sky Road's
//!   middle, six stones a metre or two across, about twenty metres apart.
//!
//! The difficulty of every one of them is a guess, unplayed: sized against
//! `crate::envelope`, which measures three of the classes' tools.
//!
//! The Sky Road's last stretch, over the mountains, is a place of its own
//! (`ArenaId::SKY_ROAD`): the land under it is nobody's, and its cairns have
//! to belong to somewhere.

use super::rim::Rim;
use super::{Arena, ArenaId, Bounds, Mark, Material, Shape, Solid, Spawns};
use crate::fixed::Fx;
use crate::math::V3;
use crate::valley::{Kind, Place};

/// The plane's middle on the map and its floor's height (centimetres).
pub const PLANE_AT: [i32; 3] = [42000, 30000, 38000];
/// Half the plane: x and z.
pub const PLANE_HALF: (i32, i32) = (13000, 11000);

pub static PLANE: Arena = Arena {
    id: ArenaId::HIGH_PLANE,
    name: "High Plane",
    creature: None,
    bounds: Bounds::cm((-PLANE_HALF.0, PLANE_HALF.0), (-PLANE_HALF.1, PLANE_HALF.1)),
    floor: Material::Grass,
    regions: &[],
    solids: &PLANE_SOLIDS,
    spawns: Spawns {
        // On the south edge, where the Hanging Stones arrive.
        versus: [
            Mark::cm(-2300, -PLANE_HALF.1 + 600, (-2300, 0)),
            Mark::cm(-1900, -PLANE_HALF.1 + 600, (-1900, 0)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: Some(Rim::new(
        (-PLANE_HALF.0, PLANE_HALF.0),
        (-PLANE_HALF.1, PLANE_HALF.1),
        [0, 0, 0, 0],
        150,
    )),
};

/// What stands on the plane: standing stones and boulders, and a cairn
/// where each route arrives.
const PLANE_SOLIDS: [Solid; 9] = [
    Solid::cm([-12400, -30, -10400], [-12200, 40, -10200], Material::Snow),
    Solid::cm([-2200, -30, -10400], [-2000, 40, -10200], Material::Snow),
    Solid::cm([-300, 0, -300], [300, 900, 300], Material::Stone),
    Solid::cm([2500, 0, 1500], [3100, 700, 2100], Material::Stone),
    Solid::cm([-1800, 0, 2600], [-1200, 1100, 3200], Material::Stone),
    Solid::cm([6000, 0, -4000], [7600, 300, -2600], Material::Rock),
    Solid::cm([-8000, 0, 5000], [-6400, 400, 6800], Material::Rock),
    Solid::cm([9000, 0, 6000], [10200, 250, 7000], Material::Rock),
    Solid::cm([-5000, 0, -7000], [-4000, 200, -6200], Material::Rock),
];

/// How the plane joins the valley: by nothing but the routes up.
pub static PLANE_PLACE: Place = Place {
    arena: ArenaId::HIGH_PLANE,
    kind: Kind::Reach,
    seams: &[],
    vents: &[],
};

/// The Sky Road's last stretch: the mountains between the valley and the
/// plane, under the stones.
pub static SKY_ROAD: Arena = Arena {
    id: ArenaId::SKY_ROAD,
    name: "Sky Road",
    creature: None,
    bounds: Bounds::cm((0, 56000), (14000, PLANE_AT[2] - PLANE_HALF.1)),
    floor: Material::Rock,
    regions: &[],
    solids: &[],
    spawns: Spawns {
        versus: [
            Mark::cm(SPAN_FROM.0, 14500, (SPAN_FROM.0, 20000)),
            Mark::cm(SPAN_FROM.0 + 300, 14500, (SPAN_FROM.0 + 300, 20000)),
        ],
        hunt: None,
    },
    sites: &[],
    rim: None,
};

pub static SKY_ROAD_PLACE: Place = Place {
    arena: ArenaId::SKY_ROAD,
    kind: Kind::Reach,
    seams: &[],
    vents: &[],
};

// ---------------------------------------------------------------------------
// A route of stones, as a table of hops
// ---------------------------------------------------------------------------

/// **A stone of a route**: its middle on the map, its top, half its side, and
/// whether it stands on the land rather than floats.
#[derive(Clone, Copy, Debug)]
pub struct Stone {
    pub x: i32,
    pub z: i32,
    pub top: i32,
    pub half: i32,
    pub grounded: bool,
}

/// **Which way a hop goes** on the map: `x` is east and `z` north.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Way {
    N,
    E,
    S,
    W,
}

impl Way {
    /// The flat axis it goes along.
    pub const fn step(self) -> (i32, i32) {
        match self {
            Way::N => (0, 1),
            Way::E => (1, 0),
            Way::S => (0, -1),
            Way::W => (-1, 0),
        }
    }
}

/// **What a hop asks** (2026-10-20): a plain running jump, landed or caught
/// at the hands, that every class that moves makes (`crate::class::MOVERS`)
/// -- or a class's tool, because no plain jump reaches it, airdodge and all.
/// Declared in the table, so the tests hold each hop to what it was laid out
/// to be (`tests/heights.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Need {
    Jump,
    Tool,
}

/// **A hop of a route**: the next stone's half-side, which way it is, the gap
/// to it edge to edge, how far its top is above the last's (less than
/// nothing: below), and what it asks -- or a gully along a way: a drop
/// between walls ([`super::gully`]) onto a landing with a cairn, crossed by
/// kicking from wall to wall.
#[derive(Clone, Copy, Debug)]
pub enum Hop {
    Stone {
        half: i32,
        gap: i32,
        rise: i32,
        way: Way,
        need: Need,
    },
    Gully {
        way: Way,
    },
}

impl Hop {
    /// What it asks: a gully asks the wall jump, which every class has.
    pub const fn need(self) -> Option<Need> {
        match self {
            Hop::Stone { need, .. } => Some(need),
            Hop::Gully { .. } => None,
        }
    }
}

use Way::{E, N, S, W};

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

/// **A precision stone**: a metre across, room for the feet and no more, a
/// plain jump away.
const fn pin(way: Way, gap: i32, rise: i32) -> Hop {
    Hop::Stone {
        half: 50,
        gap,
        rise,
        way,
        need: Need::Jump,
    }
}

/// **A rest**: six metres across, a cairn on it, a plain jump away -- put in
/// front of a hard hop, so a miss costs that hop and no more, and wide
/// enough for any class to wind its tool up on.
const fn rest(way: Way, gap: i32, rise: i32) -> Hop {
    Hop::Stone {
        half: REST,
        gap,
        rise,
        way,
        need: Need::Jump,
    }
}

/// **A hop for a class's tool**, onto a stone `half` across: a gap or a rise
/// no plain jump reaches.
const fn tool(way: Way, half: i32, gap: i32, rise: i32) -> Hop {
    Hop::Stone {
        half,
        gap,
        rise,
        way,
        need: Need::Tool,
    }
}

/// A gully along `way`.
const fn gully(way: Way) -> Hop {
    Hop::Gully { way }
}

/// **A stone this wide carries a cairn** (half its side, centimetres): a
/// rest, a gully's landing, the Crown. The Sky Road's islands have their own
/// rule ([`build`]).
pub const REST: i32 = 300;

/// How wide a gully's landing is: room to come down from a run of kicks
/// still carrying it, and a checkpoint.
pub const LANDING_HALF: i32 = 400;

/// Does a stone of the Hanging Stones or the Crown carry a cairn?
pub const fn rests(s: &Stone) -> bool {
    s.half >= REST
}

/// **The stones a table of hops lays**, from `from`: each the hop's gap past
/// the last along its way, or a gully's landing past its drop. The last
/// `grounded` stand on the land.
pub const fn lay<const K: usize>(from: Stone, hops: &[Hop; K], grounded: usize) -> [Stone; K] {
    let mut out = [from; K];
    let mut last = from;
    let mut k = 0;
    while k < K {
        let next = match hops[k] {
            Hop::Stone {
                half,
                gap,
                rise,
                way,
                ..
            } => {
                let (dx, dz) = way.step();
                let d = last.half + gap + half;
                Stone {
                    x: last.x + dx * d,
                    z: last.z + dz * d,
                    top: last.top + rise,
                    half,
                    grounded: k + grounded >= K,
                }
            }
            Hop::Gully { way } => {
                let to = super::gully::landing(takeoff(&last, way), way.step());
                let (dx, dz) = way.step();
                Stone {
                    x: to[0] + dx * LANDING_HALF,
                    z: to[2] + dz * LANDING_HALF,
                    top: to[1],
                    half: LANDING_HALF,
                    grounded: false,
                }
            }
        };
        out[k] = next;
        last = next;
        k += 1;
    }
    out
}

/// The middle of a stone's edge on `way`, at its top: where a gully leaves.
const fn takeoff(s: &Stone, way: Way) -> [i32; 3] {
    let (dx, dz) = way.step();
    [s.x + dx * s.half, s.top, s.z + dz * s.half]
}

/// **A route's gullies**: for each, the hop it is (an index into the route's
/// hops and stones) and its two walls, from the stone before it.
pub fn gullies_of(
    from: Stone,
    hops: &'static [Hop],
    stones: &'static [Stone],
) -> impl Iterator<Item = (usize, [Solid; 2])> {
    (0..hops.len()).filter_map(move |k| match hops[k] {
        Hop::Gully { way } => {
            let last = if k == 0 { from } else { stones[k - 1] };
            Some((
                k,
                super::gully::walls(takeoff(&last, way), way.step(), Material::Rock),
            ))
        }
        Hop::Stone { .. } => None,
    })
}

/// **How far apart two stones are**, edge to edge on the flat, in
/// centimetres: what a hop between them crosses.
pub fn gap(a: &Stone, b: &Stone) -> i32 {
    let apart = |p: i32, q: i32| ((p - q).abs() - a.half - b.half).max(0) as i64;
    let (gx, gz) = (apart(a.x, b.x), apart(a.z, b.z));
    crate::math::isqrt_wide((gx * gx + gz * gz) as u128) as i32
}

// ---------------------------------------------------------------------------
// The Hanging Stones
// ---------------------------------------------------------------------------

/// **The column's foot**, by the Bank's road: a stone standing on the land.
pub const FOOT: Stone = Stone {
    x: 38000,
    z: 500,
    top: 4100,
    half: 200,
    grounded: true,
};

/// **The column** (2026-10-20, from fifty stones each 4.2 m over the last
/// round a six-metre square): forty hops climbing round a square west of the
/// foot, 41 m to 203 m. The owner, on the Sky Road: "the only thing
/// resembling challenging ... there's variety in the jumps and it forces you
/// to learn class's mobility tools." So no two stretches are alike:
///
/// - **plain catches** at the edge of the shortest jumper's reach -- three to
///   five and a half metres out and three to five up, landed by the best
///   jumpers and caught at the hands by the Champion and the Blood mage;
/// - **precision stones** a metre across among them, a run of two now and
///   then, and one drop of three metres;
/// - **a rest** six metres across with a cairn in front of every tool hop,
///   and the tool hops themselves: **high** -- nine metres up and two and a
///   half out, which no plain jump catches (the best, the Dual mage's, hangs
///   from 8.2) -- and **long**: twelve or thirteen metres, level or a little
///   up, past the longest plain jump caught with its airdodge (10.5).
///
/// Every turn of the square is one loop, and no loop is the last over again.
pub const LADDER_HOPS: [Hop; 40] = [
    // The first turn: plain, then a rest and the first high hop.
    on(W, 500, 400),
    pin(W, 450, 300),
    on(W, 550, 300),
    rest(N, 300, 400),
    tool(N, 200, 250, 900),
    on(N, 450, 450),
    pin(E, 500, 200),
    pin(E, 450, 300),
    on(E, 400, 450),
    on(S, 550, 300),
    pin(S, 400, 400),
    // The second: the first long hop, and a high one.
    rest(W, 300, 300),
    tool(W, 200, 1300, 0),
    on(W, 450, 450),
    pin(N, 400, 350),
    on(N, 300, 500),
    rest(N, 300, 400),
    tool(N, 200, 250, 900),
    on(E, 500, 400),
    pin(E, 450, 250),
    pin(E, 400, 300),
    on(S, 300, 500),
    on(S, 500, -300),
    // The third: two high hops back to back, then long and up.
    rest(S, 300, 500),
    tool(W, 200, 250, 900),
    tool(W, 150, 250, 900),
    pin(N, 450, 300),
    on(N, 450, 450),
    pin(N, 400, 300),
    rest(E, 300, 300),
    tool(E, 200, 1200, 200),
    on(E, 500, 400),
    pin(S, 450, 350),
    on(S, 450, 450),
    // The fourth: a high hop off a rest, a run of pins, the top.
    rest(W, 300, 400),
    tool(W, 150, 300, 900),
    pin(W, 450, 350),
    pin(N, 450, 300),
    on(N, 450, 450),
    rest(N, 300, 300),
];

/// The column, its foot first.
pub const LADDER: [Stone; LADDER_HOPS.len()] = lay(FOOT, &LADDER_HOPS, 0);

/// **The span** (2026-10-20, from four stretches of four stones stepping up
/// 2.5 m with 4.2 m gaps, each ending in a gully, and seven steeper ones to
/// the plane): north from the column's top over the mountains to the plane's
/// south edge, 203 m to 305 m, in three stretches.
///
/// - **Over the low ground**: a gully straight off the column's top, a long
///   hop east and up, three precision stones zig-zagging, a rest, seventeen
///   metres and four up -- the longest of the Hanging Stones -- and a second
///   gully.
/// - **To the mountains' foot**: plain catches five up, a rest, fifteen
///   metres and five up, a rest, the third gully.
/// - **Up the mountainside**, where the land rises a metre for a metre under
///   it: high hops of nine metres, a rest between them, plain catches of five
///   up and precision stones, and three plain stones onto the plane's lip,
///   the last two standing on the land the plane's rim holds up.
pub const SPAN_HOPS: [Hop; 28] = [
    // Over the low ground.
    gully(N),
    on(N, 500, 400),
    tool(E, 150, 1400, 200),
    pin(N, 500, 300),
    pin(N, 500, 200),
    pin(W, 550, 100),
    rest(N, 450, 400),
    tool(N, 200, 1700, 400),
    on(N, 400, 500),
    gully(N),
    // To the mountains' foot.
    on(W, 400, 500),
    pin(N, 450, 400),
    rest(N, 450, 500),
    tool(N, 200, 1500, 500),
    rest(E, 400, 400),
    gully(N),
    // Up the mountainside.
    tool(N, 200, 300, 900),
    on(N, 400, 500),
    pin(N, 350, 400),
    rest(E, 400, 500),
    tool(N, 200, 350, 900),
    rest(N, 300, 400),
    tool(N, 200, 350, 900),
    on(W, 350, 500),
    pin(N, 350, 400),
    on(N, 400, 300),
    on(N, 400, 200),
    on(N, 300, 100),
];

/// Where the span starts: over the column's last stone.
pub const SPAN_FROM: (i32, i32) = (LADDER[LADDER.len() - 1].x, LADDER[LADDER.len() - 1].z);

/// How many of the span's last stones stand on the land rather than float:
/// the plane's rim holds the land up to its edge, so they are over it.
const SPAN_GROUNDED: usize = 2;

/// The span's stones, in order: each laid from the last by its hop.
pub const SPAN: [Stone; SPAN_HOPS.len()] = lay(LADDER[LADDER.len() - 1], &SPAN_HOPS, SPAN_GROUNDED);

/// The span's gullies: for each, the hop it is (an index into [`SPAN`]) and
/// its walls.
pub fn gullies() -> impl Iterator<Item = (usize, [Solid; 2])> {
    gullies_of(LADDER[LADDER.len() - 1], &SPAN_HOPS, &SPAN)
}

// ---------------------------------------------------------------------------
// Harder than the Sky Road: the Crown and the High Line
// ---------------------------------------------------------------------------

/// **The Crown's foot**: a pillar four metres tall standing on the plane,
/// in its south-west quarter, clear of every range on it.
pub const CROWN_FOOT: Stone = Stone {
    x: PLANE_AT[0] - 9500,
    z: PLANE_AT[2] - 7500,
    top: PLANE_AT[1] + 400,
    half: 200,
    grounded: true,
};

/// **The Crown** (2026-10-20, the owner: "a few that are more difficult
/// still than that"): an optional climb off the High Plane's floor to a turf
/// island fifty metres over it, 354 m up -- the highest place in the valley,
/// a hand over the Needle's dome, with a cairn on it and the whole valley
/// under it. Harder than the Sky Road on every count: the Sky Road's worst,
/// by its table, is eighteen metres and two up onto an island twenty metres
/// across; here the long hops are fifteen to twenty-two metres, level to
/// four up, onto stones one and two metres across, and two high ones ten
/// up, past the Champion's vault. One rest, halfway. The plain hops between
/// are at the very edge of the shortest jumpers' catch.
pub const CROWN_HOPS: [Hop; 14] = [
    pin(E, 550, 350),
    tool(E, 50, 1700, 300),
    pin(N, 600, 200),
    tool(N, 100, 300, 1000),
    rest(W, 400, 300),
    tool(W, 100, 2000, 200),
    pin(W, 600, 200),
    pin(S, 600, 100),
    tool(S, 50, 2200, 0),
    tool(E, 100, 300, 1000),
    pin(E, 550, 300),
    tool(N, 50, 1800, 400),
    on(N, 500, 400),
    tool(W, 600, 1500, 300),
];

/// The Crown's stones, its foot first; the last the island.
pub const CROWN: [Stone; CROWN_HOPS.len()] = lay(CROWN_FOOT, &CROWN_HOPS, 0);

/// **The High Line** (2026-10-20): a harder way across the Sky Road's middle,
/// off its sixth stone (its second island with a cairn) to its fourteenth
/// (its fourth), past the seven between, bowed out thirty metres to the
/// north-west of the road it cuts short. Seven hops, every one a class's
/// tool: nineteen to twenty-two metres each, two of them rising two, onto
/// stones one and two metres across -- where the stretch it skips, by its
/// table, has one gap of twenty-one metres and three of thirteen to
/// eighteen, onto stones one and a half to sixteen metres across, and a run
/// of precision stones a plain jump apart. As the Sky Road's, the gaps are a
/// pattern, stretched or squeezed alike to fit between the two islands and
/// read off the map by the tests: for each stone, its half-side, the gap
/// onto it and how far it is up from the last (less than nothing: down);
/// and then the gap onto the island it joins, whose top is the Sky Road's.
const HIGH_LINE_HOPS: [(i32, i32, i32); 6] = [
    (50, 1750, 200),
    (100, 2050, -300),
    (50, 2300, -200),
    (100, 2150, 200),
    (50, 2200, -300),
    (100, 2150, -300),
];
const HIGH_LINE_LAST: i32 = 1950;

/// Which of the Sky Road's islands the High Line leaves and joins.
pub const HIGH_LINE_FROM: usize = 5;
pub const HIGH_LINE_TO: usize = 13;

/// How far out of the road's line the High Line bows at its middle.
const HIGH_LINE_BOW: i64 = 3000;

/// **The High Line's stones**, from the first after the island it leaves to
/// the last before the island it joins.
pub fn high_line() -> Vec<Stone> {
    let road = sky_road();
    let (a, b) = (road[HIGH_LINE_FROM], road[HIGH_LINE_TO]);
    let (dx, dz) = ((b.x - a.x) as i64, (b.z - a.z) as i64);
    let len = crate::math::isqrt_wide((dx * dx + dz * dz) as u128) as i64;
    // Along the line from middle to middle: the islands' halves, the
    // stones, and the gaps stretched alike to fill the rest.
    let stones: i64 = HIGH_LINE_HOPS.iter().map(|h| 2 * h.0 as i64).sum();
    let gaps: i64 = HIGH_LINE_HOPS.iter().map(|h| h.1 as i64).sum::<i64>() + HIGH_LINE_LAST as i64;
    let room = (len - a.half as i64 - b.half as i64 - stones).max(gaps / 2);
    let mut along = a.half as i64;
    let mut top = a.top;
    let mut out = Vec::new();
    for &(half, gap, rise) in &HIGH_LINE_HOPS {
        along += gap as i64 * room / gaps + half as i64;
        top += rise;
        // Bowed out to the left of the line (north-west), most at the
        // middle: four times t(1 - t), t in thousandths of the way.
        let t = along * 1000 / len;
        let bow = HIGH_LINE_BOW * 4 * t * (1000 - t) / 1_000_000;
        out.push(Stone {
            x: (a.x as i64 + (dx * along - dz * bow) / len) as i32,
            z: (a.z as i64 + (dz * along + dx * bow) / len) as i32,
            top,
            half,
            grounded: false,
        });
        along += half as i64;
    }
    out
}

// ---------------------------------------------------------------------------
// Built
// ---------------------------------------------------------------------------

fn cm(v: i32) -> Fx {
    super::cm(v)
}

/// A stone's solid: floating, a slab under its top; grounded, from the land
/// up. Rock for the small ones, turf for an island.
fn stone(s: &Stone, base: Fx, out: &mut Vec<Solid>) {
    let thick = if s.half >= 500 { 400 } else { 120 + s.half / 3 };
    let m = if s.half >= 500 {
        Material::Grass
    } else {
        Material::Rock
    };
    let bottom = if s.grounded {
        base.sub(Fx::ONE)
    } else {
        cm(s.top - thick)
    };
    out.push(Solid {
        min: V3::new(cm(s.x - s.half), bottom, cm(s.z - s.half)),
        max: V3::new(cm(s.x + s.half), cm(s.top), cm(s.z + s.half)),
        material: m,
        shape: Shape::Box,
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

/// **The Sky Road**: the islands and stones from the Needle's dome to the
/// plane, as the pattern of hops it is made of -- a stone's half-side and
/// the gap to the next, edge to edge (centimetres), and the drop (or, less
/// than nothing, the rise) to it.
const SKY_HOPS: [(i32, i32, i32); 22] = [
    (800, 1400, 250),
    (200, 1000, 200),
    (80, 700, 150),
    (80, 700, 150),
    (200, 2000, 300),
    (600, 2400, 250),
    (150, 900, 200),
    (70, 900, -300),
    (70, 900, 200),
    (200, 2600, 400),
    (800, 1600, 250),
    (200, 1800, 300),
    (150, 2200, -200),
    (1000, 1200, 200),
    (80, 800, 150),
    (80, 800, 150),
    (80, 900, 150),
    (200, 2000, 300),
    (600, 1800, 250),
    (150, 1400, 200),
    (200, 1600, 300),
    (1000, 1600, 0),
];

/// The Sky Road's stones, from the dome's rim outward.
pub fn sky_road() -> Vec<Stone> {
    use super::spire;
    let from = (spire::AT.0 + 2950, spire::AT.1 + 3510);
    let to = (
        PLANE_AT[0] - PLANE_HALF.0 + 1200,
        PLANE_AT[2] - PLANE_HALF.1 - 1500,
    );
    let (dx, dz) = ((to.0 - from.0) as i64, (to.1 - from.1) as i64);
    let len = crate::math::isqrt_wide((dx * dx + dz * dz) as u128) as i64;
    // The gaps are stretched or squeezed, all alike, so the last island's
    // far edge is a jump short of the plane: the table says how the hops
    // go, the map how far it is.
    let stones: i64 = SKY_HOPS.iter().map(|h| 2 * h.0 as i64).sum();
    let gaps: i64 = SKY_HOPS[..SKY_HOPS.len() - 1]
        .iter()
        .map(|h| h.1 as i64)
        .sum();
    let room = (len - stones).max(gaps / 2);
    let mut along = 0i64;
    let mut top = spire::TOP - 200;
    let mut out = Vec::new();
    let mut prev_half = 0;
    for (k, &(half, gap, drop)) in SKY_HOPS.iter().enumerate() {
        along += (prev_half + half) as i64;
        let gap = gap as i64 * room / gaps.max(1);
        // A little off the line, side to side, so it is not a ruler.
        let wobble = [0, 400, -300, 500, -500, 200, -200, 600][k % 8] as i64;
        let (ux, uz) = (dx * 1000 / len, dz * 1000 / len);
        let x = from.0 as i64 + ux * along / 1000 - uz * wobble / 1000;
        let z = from.1 as i64 + uz * along / 1000 + ux * wobble / 1000;
        out.push(Stone {
            x: x as i32,
            z: z as i32,
            top,
            half,
            grounded: false,
        });
        along += gap;
        top -= drop;
        prev_half = half;
    }
    out
}

/// **The High Plane's solids, and its routes'**: the plane's own are its
/// table; these are the stones of both routes up, the gullies' walls, the
/// Crown and the High Line, and a cairn on every rest, every gully's
/// landing, the Crown's island and every one of the Sky Road's islands.
pub fn build(height: &dyn Fn(Fx, Fx) -> Fx, out: &mut Vec<Solid>) {
    let hanging = core::iter::once(&FOOT)
        .chain(LADDER.iter())
        .chain(SPAN.iter());
    let crown = core::iter::once(&CROWN_FOOT).chain(CROWN.iter());
    for s in hanging.chain(crown) {
        stone(s, height(cm(s.x), cm(s.z)), out);
        if rests(s) {
            out.push(cairn_on(s.x, s.z, s.top));
        }
    }
    for (_, walls) in gullies() {
        out.extend(walls);
    }
    for s in sky_road() {
        stone(&s, Fx::ZERO, out);
        if s.half >= 600 {
            out.push(cairn_on(s.x, s.z, s.top));
        }
    }
    for s in high_line() {
        stone(&s, Fx::ZERO, out);
    }
}
