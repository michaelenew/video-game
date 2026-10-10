//! **The Waterfall** (`docs/design/courses.md` §1c, `docs/design/valley.md`
//! §3): a cliff twenty-two metres tall with a waterfall down the middle of it,
//! and the way up is a jump climb -- one table of boxes, put down in two
//! places:
//!
//! - **On the main road of the valley**, at the head of the Pinewood: the road
//!   ends at the pool under the falls, and goes on from
//!   the top of the cliff to the third waystone (`crate::valley::layout`).
//!   The one stretch of the road that is not walked.
//! - **Alone, as a jump course** (`--arena waterfall`, [`super::climb`]): the
//!   same cliff hanging in the open air of the courses, with a start and a
//!   nest.
//!
//! | Beat | What it is |
//! | --- | --- |
//! | The pool | Three mossy stones stepping up out of the pool on the right of the falls, to the first shelf (a cairn) |
//! | The right side | Up the face: a ledge, a column standing out of the pool, a ledge -- two metres up each time |
//! | Behind the falls | Three level leaps of three and a half metres along ledges between the cliff and the falling water, to the second shelf (a cairn) on the left |
//! | The left side | Three hops of 2.1 m up, between ledges on the face and a column standing out from it, and 2.2 m onto the top |
//! | The top | The cliff's top, where the river goes over |
//!
//! **Every hop is inside the Bulwark's plain running jump**, because the road
//! is for everybody (`tests/waterfall.rs` reads the hops off this table and
//! checks each class against `crate::envelope`). It is hard by its height and
//! by the water, not by its gaps: a slip from the left side is a fall the
//! fall rule does not forgive, and **the falling water pushes down** anything
//! in it (`tuning::falls_push`), so nobody jumps up through the curtain -- the
//! way across is behind it.
//!
//! The table is in centimetres in the cliff's own frame: `x` out of the cliff
//! (its face is at zero and the climb stands at negative `x`), `y` up from the
//! pool's shore, `z` along the face.
//!
//! **It is drawn as basalt** (`game::forms`, `Form::Basalt`): every box of
//! it a cluster of irregular hexagonal columns standing inside the box, the
//! ledges and stones the tops of single columns -- what a waterfall cut
//! through a lava flow looks like. The collision is the boxes.

use super::{Material, Solid};
use crate::valley::Zone;

use Material::{Grass, Rock, Snow};

/// One box of the waterfall, in the cliff's own frame: two corners, in
/// centimetres.
#[derive(Clone, Copy, Debug)]
pub struct Piece {
    pub min: [i32; 3],
    pub max: [i32; 3],
    pub material: Material,
}

const fn piece(min: [i32; 3], max: [i32; 3], material: Material) -> Piece {
    Piece { min, max, material }
}

/// How tall the cliff is, from the shore to its top.
pub const HEIGHT: i32 = 2200;

/// **The boxes.** The route first, in the order it is climbed, then what it
/// is climbed on.
pub const PIECES: [Piece; 25] = [
    // --- The pool: three stones up out of it, right of the falls ---
    // 0: the first stone, 1.5 m off the shore and 1 m up
    piece([-1250, -300, -800], [-1000, 100, -550], Grass),
    // 1: the second, 2 m across and 1.5 m up
    piece([-900, -300, -1250], [-650, 250, -1000], Grass),
    // 2: the third, 2 m across and 1.7 m up
    piece([-600, -300, -800], [-350, 420, -550], Grass),
    // 3: the first shelf, a cairn: a metre on and 1.8 m up
    piece([-250, 450, -1600], [0, 600, -800], Snow),
    // --- The right side: up the face ---
    // 4: a ledge, 3 m along and 2 m up
    piece([-180, 650, -2200], [0, 800, -1900], Grass),
    // 5: the cap of a column standing out of the pool, 2.6 m back and 2 m up
    piece([-450, 950, -1650], [-250, 1000, -1430], Grass),
    // 6: a ledge over the first shelf, 2.4 m and 2 m up
    piece([-180, 1050, -1200], [0, 1200, -900], Grass),
    // --- Behind the falls: level leaps between the cliff and the water ---
    // 7: 3.5 m and half a metre up, into the dark behind the water
    piece([-280, 1100, -550], [0, 1250, -250], Grass),
    // 8: 3.5 m and half a metre up, the falls' middle
    piece([-280, 1150, 100], [0, 1300, 400], Grass),
    // 9: the second shelf, a cairn: 3.5 m and half a metre up, out on the left
    piece([-280, 1200, 750], [0, 1350, 1550], Snow),
    // --- The left side: up to the lip, 2.1 m a hop ---
    // 10: a ledge, 3 m along
    piece([-180, 1410, 1850], [0, 1560, 2150], Grass),
    // 11: a ledge, 3 m on
    piece([-180, 1620, 2450], [0, 1770, 2750], Grass),
    // 12: the cap of a column standing out from the face, 2.9 m back
    piece([-450, 1930, 1950], [-250, 1980, 2170], Grass),
    // --- The cliff: buttresses of rock side by side, its face at x = 0 ---
    // 13: the top, 2.5 m in from the column and 2.2 m up -- the end of the climb
    piece([0, -300, 1500], [700, HEIGHT, 2300], Rock),
    piece([0, -300, -4400], [700, 2600, -2300], Rock),
    piece([0, -300, -2300], [700, 2350, -1300], Rock),
    piece([0, -300, -1300], [700, HEIGHT, -450], Rock),
    // 17: behind the falls
    piece([0, -300, -450], [700, HEIGHT, 450], Rock),
    piece([0, -300, 450], [700, HEIGHT, 1500], Rock),
    piece([0, -300, 2300], [700, 2700, 4400], Rock),
    // 20: the cliff's top, behind the buttresses, back to where the land is
    // as high as it is
    piece([700, -300, -4400], [3000, HEIGHT, 4400], Rock),
    // 21: the lip, four and a half metres out over the ledges behind the
    // water: what it falls from
    piece([-450, 1950, -450], [0, HEIGHT, 450], Rock),
    // 22: the right column, under its cap
    piece([-440, -300, -1640], [-260, 950, -1440], Rock),
    // 23: the left column, under its cap
    piece([-440, -300, 1960], [-260, 1930, 2160], Rock),
    // 24: a boulder at the falls' foot, in the spray
    piece([-1100, -300, 300], [-800, 60, 650], Rock),
];

/// The route, by index into [`PIECES`]: every top climbed on, in order, the
/// cliff's top last.
pub const ROUTE: [usize; 14] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];

/// The route's cairns: the two shelves.
pub const SHELVES: [usize; 2] = [3, 9];

/// **The falling water**: from the lip to the pool, in front of the ledges
/// behind it. Feet in it are pushed down.
pub const SHEET: Piece = piece([-700, -200, -300], [-380, 1950, 300], Material::Water);

/// **The pool** under the falls: where the water lands, its surface at the
/// shore's height.
pub const POOL: Piece = piece([-1400, -150, -1600], [-100, 0, 900], Material::Water);

/// **The river over the top**: from the back of the cliff's top to the lip,
/// the width of the falls. Drawn, not a box.
pub const RIVER: Piece = piece([-450, HEIGHT, -300], [3000, HEIGHT, 300], Material::Water);

/// Where the waterfall's frame is on the valley's map (the Pinewood is a
/// reach, so its coordinates are the map's): the middle of the cliff's foot,
/// centimetres.
pub const VALLEY_AT: [i32; 3] = [79900, 6000, 0];

/// Where it is in the jump course's arena.
pub const COURSE_AT: [i32; 3] = [3100, 6000, 0];

/// A piece, moved into a frame `at` (centimetres) and made a box.
pub const fn solid(p: &Piece, at: [i32; 3]) -> Solid {
    Solid::cm(
        [p.min[0] + at[0], p.min[1] + at[1], p.min[2] + at[2]],
        [p.max[0] + at[0], p.max[1] + at[1], p.max[2] + at[2]],
        p.material,
    )
}

/// A piece moved into a frame, as a zone a body's feet can be in.
pub const fn zone(p: &Piece, at: [i32; 3]) -> Zone {
    Zone::cm(
        [p.min[0] + at[0], p.min[1] + at[1], p.min[2] + at[2]],
        [p.max[0] + at[0], p.max[1] + at[1], p.max[2] + at[2]],
    )
}

/// Every piece, moved into a frame.
pub const fn placed(at: [i32; 3]) -> [Solid; PIECES.len()] {
    let mut out = [Solid::cm([0, 0, 0], [0, 0, 0], Rock); PIECES.len()];
    let mut i = 0;
    while i < PIECES.len() {
        out[i] = solid(&PIECES[i], at);
        i += 1;
    }
    out
}

/// **Is this box part of a waterfall** -- one of its table's, in the valley
/// or in the course, or the course's shore? What the renderer draws as
/// basalt. Not asked in a frame.
pub fn part_of(s: &Solid) -> bool {
    [VALLEY_AT, COURSE_AT]
        .iter()
        .any(|&at| PIECES.iter().any(|p| solid(p, at) == *s))
        || solid(&SHORE, COURSE_AT) == *s
}

/// The sheet on the valley's map, and in the course's arena.
const VALLEY_SHEETS: [Zone; 1] = [zone(&SHEET, VALLEY_AT)];
const COURSE_SHEETS: [Zone; 1] = [zone(&SHEET, COURSE_AT)];

/// **Where a waterfall is in a place**, in that place's coordinates: the frame
/// its table is put down in. The Pinewood's and the course's.
pub const fn frame_in(id: super::ArenaId) -> Option<[i32; 3]> {
    match id {
        super::ArenaId::PINEWOOD => Some(VALLEY_AT),
        super::ArenaId::CLIMB_WATERFALL => Some(COURSE_AT),
        _ => None,
    }
}

/// The falling water in an arena, in its own coordinates.
pub fn sheets(id: super::ArenaId) -> &'static [Zone] {
    match id {
        super::ArenaId::PINEWOOD => &VALLEY_SHEETS,
        super::ArenaId::CLIMB_WATERFALL => &COURSE_SHEETS,
        _ => &[],
    }
}

/// Is a body's feet in an arena's falling water?
pub fn sheet_at(id: super::ArenaId, feet: crate::math::V3) -> bool {
    sheets(id).iter().any(|z| z.holds(feet))
}

/// **The near shore of the pool**, where the climb starts: in the course, an
/// island; in the valley, the land the road ends on.
pub const SHORE: Piece = piece([-2600, -300, -1000], [-1400, 0, 1000], Grass);

/// **The cairn on the top**, in the valley: a slab of snow back from the
/// lip, where the climb is over. The course has its nest there instead.
pub const CROWN: Piece = piece([1700, HEIGHT, -1300], [1900, HEIGHT + 40, -1100], Snow);

/// **What the waterfall takes up** on the ground, flat: no tree or boulder
/// grows in it. Centimetres in its own frame, (x, z) low and high.
pub const CLEARING: ([i32; 2], [i32; 2]) = ([-2400, -4900], [3200, 4900]);

/// What each step of [`ROUTE`] asks, in a few words, for the course panel.
pub const ASKS: [&str; ROUTE.len()] = [
    "the first stone out of the pool",
    "the second stone",
    "the third stone",
    "the first shelf, a cairn",
    "up the right: a ledge",
    "a column out of the pool",
    "a ledge over the shelf",
    "behind the falls, 1",
    "behind the falls, 2",
    "the second shelf, a cairn",
    "up the left: a ledge",
    "a ledge on",
    "a column standing out",
    "the top of the falls",
];

/// **One hop**, from the top of `a` to the top of `b`, as built: the gap
/// between their facing edges, flat, and how far `b`'s top is above `a`'s, in
/// centimetres.
pub const fn hop(a: &Piece, b: &Piece) -> (i32, i32) {
    let gx = gap_on(a, b, 0);
    let gz = gap_on(a, b, 2);
    (root(gx * gx + gz * gz), b.max[1] - a.max[1])
}

const fn gap_on(a: &Piece, b: &Piece, axis: usize) -> i32 {
    let one = b.min[axis] - a.max[axis];
    let other = a.min[axis] - b.max[axis];
    let g = if one > other { one } else { other };
    if g > 0 { g } else { 0 }
}

/// **The hops of the route**: for each step after the shore, the piece and
/// the hop to it. What the tests check every class against.
pub fn hops() -> impl Iterator<Item = (usize, i32, i32)> {
    (0..ROUTE.len()).map(|k| {
        let from = if k == 0 {
            &SHORE
        } else {
            &PIECES[ROUTE[k - 1]]
        };
        let (gap, rise) = hop(from, &PIECES[ROUTE[k]]);
        (ROUTE[k], gap, rise)
    })
}

/// The whole square root of a whole number, rounded down.
const fn root(n: i32) -> i32 {
    let mut r = 0;
    while (r + 1) * (r + 1) <= n {
        r += 1;
    }
    r
}
