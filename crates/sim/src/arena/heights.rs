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
//!   a column of stones a rise of seven metres apart with **vines between
//!   them**, so anybody can climb it at the vine's pace and a mechanic skips
//!   the vines; then a span of stones north over the mountains, ten metres
//!   apart and six up, each with a vine hanging off its near side to catch
//!   a short jump.
//!
//! The Sky Road's last stretch, over the mountains, is a place of its own
//! (`ArenaId::SKY_ROAD`): the land under it is nobody's, and its vines and
//! cairns have to belong to somewhere.

use super::rim::Rim;
use super::{Arena, ArenaId, Bounds, Mark, Material, Shape, Solid, Spawns};
use crate::fixed::Fx;
use crate::math::V3;
use crate::valley::{Kind, Place, Zone};

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
    vines: &[],
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
    vines: &SKY_VINES,
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
/// How far each stone of the column is above the last.
pub const LADDER_RISE: i32 = 700;
/// The column's stones go round a square this far across.
const LADDER_STEP: i32 = 600;

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
        half: 150,
        grounded: k == 0,
    }
}

/// The column, foot to top.
pub const LADDER: [Stone; LADDER_COUNT] = [
    ladder(0),
    ladder(1),
    ladder(2),
    ladder(3),
    ladder(4),
    ladder(5),
    ladder(6),
    ladder(7),
    ladder(8),
    ladder(9),
    ladder(10),
    ladder(11),
    ladder(12),
    ladder(13),
    ladder(14),
    ladder(15),
    ladder(16),
    ladder(17),
    ladder(18),
    ladder(19),
    ladder(20),
    ladder(21),
    ladder(22),
    ladder(23),
    ladder(24),
];
const LADDER_COUNT: usize = 25;

/// The span: from over the column's top north to the plane's south edge.
pub const SPAN_FROM: (i32, i32) = (38300, 1500);
pub const SPAN_TO: (i32, i32, i32) = (40000, 25800, 30300);
const SPAN_COUNT: i32 = 18;

const fn span(k: i32) -> Stone {
    let n = SPAN_COUNT - 1;
    let from_top = LADDER_AT.2 + LADDER_RISE * (LADDER_COUNT as i32 - 1);
    Stone {
        x: SPAN_FROM.0 + (SPAN_TO.0 - SPAN_FROM.0) * k / n,
        z: SPAN_FROM.1 + (SPAN_TO.1 - SPAN_FROM.1) * k / n,
        top: from_top + 200 + (SPAN_TO.2 - from_top - 200) * k / n,
        half: 200,
        grounded: false,
    }
}

pub const SPAN: [Stone; 18] = [
    span(0),
    span(1),
    span(2),
    span(3),
    span(4),
    span(5),
    span(6),
    span(7),
    span(8),
    span(9),
    span(10),
    span(11),
    span(12),
    span(13),
    span(14),
    span(15),
    span(16),
    span(17),
];

/// **A vine up the column**: filling the gap between stone `k - 1` and stone
/// `k`, from under the lower's top to a little over the upper's -- step off
/// the one and you are on it, climb, and step onto the other.
const fn ladder_vine(k: i32) -> Zone {
    let (a, b) = (ladder(k - 1), ladder(k));
    let (x0, x1, z0, z1) = if a.x < b.x {
        (a.x + a.half - 20, b.x - b.half + 20, b.z - 150, b.z + 150)
    } else if a.x > b.x {
        (b.x + b.half - 20, a.x - a.half + 20, b.z - 150, b.z + 150)
    } else if a.z < b.z {
        (b.x - 150, b.x + 150, a.z + a.half - 20, b.z - b.half + 20)
    } else {
        (b.x - 150, b.x + 150, b.z + b.half - 20, a.z - a.half + 20)
    };
    Zone::cm([x0, a.top - 400, z0], [x1, b.top + 150, z1])
}

/// **A vine strung across the span's gap**: from the stone before's far
/// edge to this one's near (south) side, from twelve metres under this one's
/// top to a little over it -- step off the one onto it, or fall short into
/// it, and climb.
const fn span_vine(k: i32) -> Zone {
    let s = span(k);
    let before = if k == 0 {
        ladder(LADDER_COUNT as i32 - 1)
    } else {
        span(k - 1)
    };
    let x0 = if before.x - before.half < s.x - s.half {
        before.x - before.half
    } else {
        s.x - s.half
    };
    let x1 = if before.x + before.half > s.x + s.half {
        before.x + before.half
    } else {
        s.x + s.half
    };
    // Off the column's top it starts just under that top: lower, it would
    // hang over the column's last stones and catch a climber on them.
    let low = if k == 0 {
        before.top - 300
    } else {
        s.top - 1200
    };
    Zone::cm(
        [x0 - 50, low, before.z + before.half - 20],
        [x1 + 50, s.top + 150, s.z - s.half + 20],
    )
}

/// **The last vine**: from the span's last stone up the plane's cliff to its
/// lip.
const fn landing_vine() -> Zone {
    let s = span(SPAN_COUNT - 1);
    let edge = PLANE_AT[2] - PLANE_HALF.1;
    Zone::cm(
        [s.x - s.half - 50, s.top - 1200, s.z + s.half - 20],
        [s.x + s.half + 50, PLANE_AT[1] + 400, edge + 50],
    )
}

/// The vines over the Bank: the column's, and the span's -- every one of
/// them, because which of its stones are over the Bank and which over the
/// mountains is the map's business; a vine only counts where it is.
pub const BANK_VINES: [Zone; 42] = [
    ladder_vine(1),
    ladder_vine(2),
    ladder_vine(3),
    ladder_vine(4),
    ladder_vine(5),
    ladder_vine(6),
    ladder_vine(7),
    ladder_vine(8),
    ladder_vine(9),
    ladder_vine(10),
    ladder_vine(11),
    ladder_vine(12),
    ladder_vine(13),
    ladder_vine(14),
    ladder_vine(15),
    ladder_vine(16),
    ladder_vine(17),
    ladder_vine(18),
    ladder_vine(19),
    ladder_vine(20),
    ladder_vine(21),
    ladder_vine(22),
    ladder_vine(23),
    ladder_vine(24),
    span_vine(0),
    span_vine(1),
    span_vine(2),
    span_vine(3),
    span_vine(4),
    span_vine(5),
    span_vine(6),
    span_vine(7),
    span_vine(8),
    span_vine(9),
    span_vine(10),
    span_vine(11),
    span_vine(12),
    span_vine(13),
    span_vine(14),
    span_vine(15),
    span_vine(16),
    span_vine(17),
];

/// The vines over the mountains: the span's, and the last one up the cliff.
const SKY_VINES: [Zone; 19] = [
    span_vine(0),
    span_vine(1),
    span_vine(2),
    span_vine(3),
    span_vine(4),
    span_vine(5),
    span_vine(6),
    span_vine(7),
    span_vine(8),
    span_vine(9),
    span_vine(10),
    span_vine(11),
    span_vine(12),
    span_vine(13),
    span_vine(14),
    span_vine(15),
    span_vine(16),
    span_vine(17),
    landing_vine(),
];

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
    for s in sky_road() {
        stone(&s, Fx::ZERO, out);
        if s.half >= 600 {
            out.push(cairn_on(s.x, s.z, s.top));
        }
    }
    // A cairn on the column's top.
    let t = LADDER[LADDER_COUNT - 1];
    out.push(cairn_on(t.x, t.z, t.top));
}
