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
//!   a column of fifty stones round a square, each a ledge's catch above the
//!   last (4.2 m: the plainest jump hangs from it and climbs, the best
//!   jumpers land); then a span of stones north over the mountains, broken
//!   four times by **a gully** -- twelve metres of drop between two walls,
//!   crossed by jumping from wall to wall ([`super::gully`]). Until
//!   2026-10-16 it was stones seven metres apart with vines between them,
//!   which the owner found awful to get stuck in.
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
// The Hanging Stones
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

/// Where the Hanging Stones' column stands, by the Bank's road, and its
/// first top.
pub const LADDER_AT: (i32, i32, i32) = (38000, 500, 4100);
/// How far each stone of the column is above the last: past every class's
/// landing but the best jumpers', inside the Bulwark's catch (2026-10-16,
/// from 7 m with a vine between): the plainest jump hangs from each and
/// climbs.
pub const LADDER_RISE: i32 = 420;
/// The column's stones go round a square this far across.
const LADDER_STEP: i32 = 600;
const LADDER_COUNT: usize = 50;

const fn ladder(k: i32) -> Stone {
    let (dx, dz) = match k % 4 {
        0 => (0, 0),
        1 => (LADDER_STEP, 0),
        2 => (LADDER_STEP, LADDER_STEP),
        _ => (0, LADDER_STEP),
    };
    Stone {
        x: LADDER_AT.0 + dx,
        z: LADDER_AT.1 + dz,
        top: LADDER_AT.2 + LADDER_RISE * k,
        half: if k % 10 == 9 { 250 } else { 150 },
        grounded: k == 0,
    }
}

/// Every tenth stone of the column is wider, with a cairn on it: the
/// column's checkpoints, the last its top.
pub const fn ladder_rests(k: usize) -> bool {
    k % 10 == 9
}

/// The column, foot to top.
pub const LADDER: [Stone; LADDER_COUNT] = {
    let mut out = [ladder(0); LADDER_COUNT];
    let mut k = 0;
    while k < LADDER_COUNT {
        out[k] = ladder(k as i32);
        k += 1;
    }
    out
};

/// **A hop of the span**: the next stone's half-side, the gap to it edge to
/// edge along the span (north) and how far it is up from the last -- or
/// [`GULLY`], a gully's drop between walls ([`super::gully`]) onto a landing
/// with a cairn.
#[derive(Clone, Copy, Debug)]
pub enum Hop {
    Stone { half: i32, gap: i32, rise: i32 },
    Gully,
}

/// A gully hop.
pub const GULLY: Hop = Hop::Gully;

const fn up(gap: i32, rise: i32) -> Hop {
    Hop::Stone {
        half: 200,
        gap,
        rise,
    }
}

/// How wide a gully's landing is: room to come down from a run of kicks
/// still carrying it, and a checkpoint.
pub const LANDING_HALF: i32 = 400;

/// **The span**, from the column's top north over the mountains to the
/// plane's south edge (2026-10-16, from eighteen stones ten metres apart
/// and six up with a vine across each gap): four stretches of four stones
/// stepping up, each ending in a gully, then seven steeper ones to the
/// plane. Each step is inside the Bulwark's catch; each gully is crossed by
/// kicking from wall to wall, which every class can and no class's plain
/// jump does.
pub const SPAN_HOPS: [Hop; 27] = [
    up(420, 250),
    up(420, 250),
    up(420, 250),
    up(420, 250),
    GULLY,
    up(420, 250),
    up(420, 250),
    up(420, 250),
    up(420, 250),
    GULLY,
    up(420, 250),
    up(420, 250),
    up(420, 250),
    up(420, 250),
    GULLY,
    up(420, 250),
    up(420, 250),
    up(420, 250),
    up(420, 250),
    GULLY,
    up(300, 340),
    up(300, 340),
    up(300, 340),
    up(300, 340),
    up(300, 340),
    up(300, 340),
    up(300, 340),
];

/// Where the span starts: over the column's last stone.
pub const SPAN_FROM: (i32, i32) = (LADDER[LADDER_COUNT - 1].x, LADDER[LADDER_COUNT - 1].z);

/// How many of the span's last stones stand on the land rather than float:
/// the plane's rim holds the land up to its edge, so they are over it.
const SPAN_GROUNDED: usize = 2;

/// The span's stones, in order: each laid north of the last by its hop.
pub const SPAN: [Stone; SPAN_HOPS.len()] = {
    let mut out = [LADDER[LADDER_COUNT - 1]; SPAN_HOPS.len()];
    let mut last = LADDER[LADDER_COUNT - 1];
    let mut k = 0;
    while k < SPAN_HOPS.len() {
        let next = match SPAN_HOPS[k] {
            Hop::Stone { half, gap, rise } => Stone {
                x: last.x,
                z: last.z + last.half + gap + half,
                top: last.top + rise,
                half,
                grounded: k + SPAN_GROUNDED >= SPAN_HOPS.len(),
            },
            Hop::Gully => {
                let to = super::gully::landing([last.x, last.top, last.z + last.half], (0, 1));
                Stone {
                    x: to[0],
                    z: to[2] + LANDING_HALF,
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
};

/// The span's gullies: for each, the stone it is jumped from (an index of
/// [`SPAN`], or `None` for the column's top) and its walls.
pub fn gullies() -> impl Iterator<Item = (usize, [Solid; 2])> {
    (0..SPAN_HOPS.len())
        .filter(|&k| matches!(SPAN_HOPS[k], Hop::Gully))
        .map(|k| {
            let from = if k == 0 {
                LADDER[LADDER_COUNT - 1]
            } else {
                SPAN[k - 1]
            };
            let at = [from.x, from.top, from.z + from.half];
            (k, super::gully::walls(at, (0, 1), Material::Rock))
        })
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
/// table; these are the stones of both routes up, and a cairn on every
/// island.
pub fn build(height: &dyn Fn(Fx, Fx) -> Fx, out: &mut Vec<Solid>) {
    for s in LADDER.iter().chain(SPAN.iter()) {
        stone(s, height(cm(s.x), cm(s.z)), out);
    }
    for (k, walls) in gullies() {
        out.extend(walls);
        let s = SPAN[k];
        out.push(cairn_on(s.x, s.z, s.top));
    }
    for s in sky_road() {
        stone(&s, Fx::ZERO, out);
        if s.half >= 600 {
            out.push(cairn_on(s.x, s.z, s.top));
        }
    }
    // A cairn on each of the column's wide stones.
    for (k, t) in LADDER.iter().enumerate() {
        if ladder_rests(k) {
            out.push(cairn_on(t.x, t.z, t.top));
        }
    }
}
