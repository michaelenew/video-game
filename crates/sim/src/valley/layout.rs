//! **The valley as one map**: where every place goes, the land between them,
//! and what grows on it.
//!
//! Since 2026-10-09 (the second time that day) the valley is laid out as
//! land rather than as rooms of boxes (`docs/design/atlas.md`,
//! `crate::valley::land`):
//!
//! - **The town and the reaches share the map's coordinates.** A reach's
//!   origin is the map's, so its road, seams and crags are written where they
//!   are. The town sits at the origin.
//! - **The road** is each reach's `WAY`, one after another from the town's
//!   east gate to the Saddle: the valley floor, winding, wide in the meadows
//!   and narrow at the passes. Off its edges the land rises into hills and
//!   mountains, too steep to walk.
//! - **A room** sits in a clearing off the road, at the end of a side path
//!   from a junction of the road to the middle of the room's near side. Its
//!   floor is its own, level ground runs [`APRON`] round it, and a doorway is
//!   cut through its wall where the path arrives. The town, the Ring and the
//!   Long Valley are the same kind of place.
//! - **A waystone's door** is a box across a pass: shut while the stone is
//!   dark (`valley::open`).
//! - **Crags, cairns, trees and boulders** are made here, standing on the
//!   land wherever it is under them: a tree's trunk is a box you cannot walk
//!   through, and the renderer draws a tree round it.

use crate::arena::{self, ArenaId, Material, Solid};
use crate::atlas::{DoorPlan, Plan, PlanPlace, footprint};
use crate::fixed::Fx;
use crate::math::V3;
use crate::valley::Crag;
use crate::valley::land::{Land, Pad, Point, Way, at};

use crate::arena::{
    bank, deep, hearth, heights, mouth, pinewood, saddle, shelves, spire, stations, waterfall, wood,
};

/// Level ground round a room, past its own footprint, in centimetres.
pub const APRON: i32 = 600;

/// How wide a doorway into a room is cut, and how high.
const DOOR: i32 = 600;
const DOOR_HIGH: i32 = 600;

/// The floor where no place is.
const VOID: i32 = -10_000;

/// Which side of a room its door is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    North,
    South,
    East,
    West,
}

/// **A room off the road**: the place, where its origin goes on the map
/// (centimetres), the side its door is on and where along it (in the room's
/// own coordinates), how far into it the doorway is cut past the ground
/// round it, and the point of the road its side path leaves from.
#[derive(Clone, Copy, Debug)]
pub struct Room {
    pub arena: ArenaId,
    pub at: [i32; 3],
    pub side: Side,
    pub along: i32,
    pub depth: i32,
    /// The road's junction, centimetres: x, z, floor.
    pub from: (i32, i32, i32),
    /// The seam of the reach the side path leaves from.
    pub seam: (ArenaId, u8),
    /// **A station on a jump route** (2026-10-22), not a clearing at the end
    /// of a side path: raised over the land round it with a ledge at its
    /// door (`land::Pad::ledge`), and reached and left by the routes of
    /// `arena::stations`. The owner: "For the ones that are odd terrain,
    /// make them midpoints in some mobility sections." `from` is the foot
    /// of the route up to it.
    pub station: bool,
}

const fn room(
    arena: ArenaId,
    at: [i32; 3],
    side: Side,
    along: i32,
    depth: i32,
    from: (i32, i32, i32),
    seam: (ArenaId, u8),
) -> Room {
    Room {
        arena,
        at,
        side,
        along,
        depth,
        from,
        seam,
        station: false,
    }
}

/// A station: a room raised on a jump route ([`Room::station`]).
const fn station(
    arena: ArenaId,
    at: [i32; 3],
    side: Side,
    along: i32,
    depth: i32,
    from: (i32, i32, i32),
    seam: (ArenaId, u8),
) -> Room {
    Room {
        station: true,
        ..room(arena, at, side, along, depth, from, seam)
    }
}

/// **The rooms**: the Ring and the Long Valley off the town, the creatures'
/// rooms off the road.
pub const ROOMS: [Room; 8] = [
    room(
        ArenaId::RING,
        [0, 0, 7600],
        Side::South,
        0,
        200,
        (0, 3600, 0),
        (ArenaId::HEARTH, 1),
    ),
    room(
        ArenaId::SIEGESHELL,
        [-21500, 0, 0],
        Side::East,
        0,
        600,
        (-4000, 0, 0),
        (ArenaId::HEARTH, 2),
    ),
    // **The stations** (2026-10-22): the creatures whose arenas are odd
    // ground -- a mire, a pan of sand, a cave, a plateau over a drop, a wood
    // of snow and ash, a court of columns -- raised on the routes of
    // `arena::stations`, each the middle of a jump section: reached by one
    // and left by another. `from` is the foot of the way up.
    station(
        ArenaId::MIREBACK,
        stations::MIRE_AT,
        Side::North,
        400,
        150,
        stations::MIRE_FOOT,
        (ArenaId::SHELVES, 1),
    ),
    station(
        ArenaId::SANDMAW,
        stations::PAN_AT,
        Side::North,
        0,
        150,
        stations::PAN_FOOT,
        (ArenaId::SHELVES, 2),
    ),
    station(
        ArenaId::BROODMOTHER,
        stations::HOLLOWS_AT,
        Side::North,
        0,
        200,
        stations::HOLLOWS_FOOT,
        (ArenaId::PINEWOOD, 1),
    ),
    // The Cliffs sit sunk, their plateau at the road's height, a ravine round
    // them: the way in is stones across it.
    station(
        ArenaId::GALEWING,
        stations::CLIFFS_AT,
        Side::South,
        0,
        0,
        stations::CLIFFS_FOOT,
        (ArenaId::SADDLE, 1),
    ),
    station(
        ArenaId::VEILSTALKER,
        stations::ASHWOOD_AT,
        Side::North,
        0,
        200,
        stations::ASHWOOD_FOOT,
        (ArenaId::SADDLE, 2),
    ),
    station(
        ArenaId::MANTIS,
        stations::SHRINE_AT,
        Side::West,
        0,
        200,
        stations::SHRINE_FOOT,
        (ArenaId::SADDLE, 3),
    ),
];

/// **The doors between reaches**, and the waystones' among them: the two
/// seams, and the box across the pass (shut while its stone is dark).
/// A door between two seams, and its box: two corners in centimetres.
pub type Pass = ((ArenaId, u8), (ArenaId, u8), [[i32; 3]; 2]);

pub const PASSES: [Pass; 8] = [
    (
        (ArenaId::HEARTH, 0),
        (ArenaId::MOUTH, 0),
        [[4400, 0, -300], [5200, 600, 300]],
    ),
    (
        (ArenaId::MOUTH, 1),
        (ArenaId::BANK, 0),
        [[24800, 1900, -1200], [25200, 2600, 1200]],
    ),
    (
        (ArenaId::BANK, 1),
        (ArenaId::SHELVES, 0),
        [[42900, 4300, -1600], [43100, 5400, 800]],
    ),
    (
        (ArenaId::SHELVES, 3),
        (ArenaId::PINEWOOD, 0),
        [[65100, 5500, -1000], [65300, 6600, 1800]],
    ),
    (
        (ArenaId::PINEWOOD, 2),
        (ArenaId::SADDLE, 0),
        [[85100, 8100, -800], [85300, 9200, 2000]],
    ),
    (
        (ArenaId::MOUTH, 2),
        (ArenaId::SPIRE, 0),
        [[4400, 0, -5900], [4600, 600, -4900]],
    ),
    (
        (ArenaId::SPIRE, 1),
        (ArenaId::WOOD, 0),
        [[800, 0, -14050], [2000, 600, -13950]],
    ),
    (
        (ArenaId::SHELVES, 4),
        (ArenaId::DEEP, 0),
        [[60000, 4400, -14950], [62000, 5200, -14850]],
    ),
];

/// The town's three roads out: east to the climb, north to the Ring, west to
/// the Long Valley.
const TOWN_ROADS: [[Point; 2]; 2] = [
    [at(0, 3600, 0, 300), at(0, 5600, 0, 300)],
    [at(-4000, 0, 0, 300), at(-6000, 0, 0, 300)],
];

/// How dense the trees are in each reach, in thousandths of a candidate spot,
/// and whether they are pines.
const WOODS: [(ArenaId, u32, bool); 5] = [
    (ArenaId::MOUTH, 90, false),
    (ArenaId::BANK, 120, false),
    (ArenaId::SHELVES, 80, true),
    (ArenaId::PINEWOOD, 520, true),
    (ArenaId::SADDLE, 70, true),
];

/// Candidate spots for trees and boulders: a jittered grid this far apart.
const SPACING: i32 = 600;

fn cm(v: i32) -> Fx {
    arena::cm(v)
}

fn v3(c: [i32; 3]) -> V3 {
    V3::new(cm(c[0]), cm(c[1]), cm(c[2]))
}

/// A deterministic number in [0, 1000) from two integers and a salt.
fn dice(a: i32, b: i32, salt: u32) -> u32 {
    let mut h = (a as u32).wrapping_mul(0x9E37_79B1)
        ^ (b as u32).wrapping_mul(0x85EB_CA77)
        ^ salt.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h % 1000
}

/// A room's rim on the map, if the land holds one up round it: every room
/// with a rim but a drop (the Cliffs sit in their ravine).
fn rim_of(r: &Room) -> Option<arena::rim::Rim> {
    r.arena.get().rim.filter(|rim| !rim.drop)
}

/// The level ground round a room past its footprint: none where a rim
/// rises straight off its walls.
fn apron_of(r: &Room) -> Fx {
    if rim_of(r).is_some() {
        Fx::ZERO
    } else {
        cm(APRON)
    }
}

/// A room's footprint on the map with the ground round it: lo and hi (x, z).
/// With a rim, the rim's rectangle.
fn pad_of(r: &Room) -> ((Fx, Fx), (Fx, Fx)) {
    let (mut lo, mut hi) = footprint(r.arena.get());
    if let Some(rim) = rim_of(r) {
        (lo, hi) = rim.rect();
    }
    let at = v3(r.at);
    let a = apron_of(r);
    (
        (lo.0.add(at.x).sub(a), lo.1.add(at.z).sub(a)),
        (hi.0.add(at.x).add(a), hi.1.add(at.z).add(a)),
    )
}

/// Where a room's side path meets its pad, and the doorway cut through its
/// wall there, on the map.
fn door_of(r: &Room) -> (V3, Solid) {
    let (lo, hi) = pad_of(r);
    let at = v3(r.at);
    let (lo_local, hi_local) = footprint(r.arena.get());
    let along = cm(r.along);
    let half = cm(DOOR / 2);
    let y0 = at.y;
    let y1 = at.y.add(cm(DOOR_HIGH));
    let (edge, cut) = match r.side {
        Side::South => {
            let x = at.x.add(along);
            let z = lo.1;
            let inner = lo_local.1.add(at.z).add(cm(r.depth));
            (
                V3::new(x, y0, z),
                (V3::new(x.sub(half), y0, z), V3::new(x.add(half), y1, inner)),
            )
        }
        Side::North => {
            let x = at.x.add(along);
            let z = hi.1;
            let inner = hi_local.1.add(at.z).sub(cm(r.depth));
            (
                V3::new(x, y0, z),
                (V3::new(x.sub(half), y0, inner), V3::new(x.add(half), y1, z)),
            )
        }
        Side::West => {
            let z = at.z.add(along);
            let x = lo.0;
            let inner = lo_local.0.add(at.x).add(cm(r.depth));
            (
                V3::new(x, y0, z),
                (V3::new(x, y0, z.sub(half)), V3::new(inner, y1, z.add(half))),
            )
        }
        Side::East => {
            let z = at.z.add(along);
            let x = hi.0;
            let inner = hi_local.0.add(at.x).sub(cm(r.depth));
            (
                V3::new(x, y0, z),
                (V3::new(inner, y0, z.sub(half)), V3::new(x, y1, z.add(half))),
            )
        }
    };
    (
        edge,
        Solid {
            min: cut.0,
            max: cut.1,
            material: Material::Ground,
            shape: crate::arena::Shape::Box,
        },
    )
}

/// The town's pad, and the Ring's and every room's.
fn pads() -> Vec<Pad> {
    let mut out = Vec::new();
    let (lo, hi) = footprint(hearth::ARENA.id.get());
    let a = cm(APRON);
    out.push(Pad {
        lo: (lo.0.sub(a), lo.1.sub(a)),
        hi: (hi.0.add(a), hi.1.add(a)),
        y: Fx::ZERO,
        rim: None,
        door: (Fx::ZERO, Fx::ZERO),
        ledge: false,
    });
    // The Deep's pit: its floor 140 m under the road, over the pit alone --
    // the lid round it is the Deep's place but the land's crater.
    let at = v3(deep::AT);
    let (hx, hz) = (cm(deep::HALF.0), cm(deep::HALF.1));
    out.push(Pad {
        lo: (at.x.sub(hx), at.z.sub(hz)),
        hi: (at.x.add(hx), at.z.add(hz)),
        y: at.y,
        rim: None,
        door: (at.x, at.z),
        ledge: false,
    });
    // The High Plane: 300 m up, its rim holding the land up to its edge.
    let at = v3(heights::PLANE_AT);
    let (hx, hz) = (cm(heights::PLANE_HALF.0), cm(heights::PLANE_HALF.1));
    out.push(Pad {
        lo: (at.x.sub(hx), at.z.sub(hz)),
        hi: (at.x.add(hx), at.z.add(hz)),
        y: at.y,
        rim: heights::PLANE.rim.map(|rim| (rim, (at.x, at.z))),
        door: (at.x, at.z),
        ledge: false,
    });
    // The halls: a floor inside a rim, and nothing cut through it.
    for (k, t) in crate::tutorial::ALL.iter().enumerate() {
        let hall = t.arena.get();
        let at = v3(hall_at(k));
        let (lo, hi) = hall.rim.map_or_else(|| footprint(hall), |rim| rim.rect());
        out.push(Pad {
            lo: (lo.0.add(at.x), lo.1.add(at.z)),
            hi: (hi.0.add(at.x), hi.1.add(at.z)),
            y: at.y,
            rim: hall.rim.map(|rim| (rim, (at.x, at.z))),
            door: (at.x, at.z),
            ledge: false,
        });
    }
    for r in &ROOMS {
        let (lo, hi) = pad_of(r);
        let at = v3(r.at);
        let (edge, _) = door_of(r);
        out.push(Pad {
            lo,
            hi,
            y: at.y,
            rim: rim_of(r).map(|rim| (rim, (at.x, at.z))),
            door: (edge.x, edge.z),
            ledge: r.station,
        });
    }
    out
}

/// A side path's points: from the road's junction to a few metres into the
/// room's pad, through its door.
fn side_path(r: &Room) -> Vec<Point> {
    let (edge, _) = door_of(r);
    let to_cm = |v: Fx| (v.raw() as i64 * 100 / Fx::ONE.raw() as i64) as i32;
    let (x, z, y) = (to_cm(edge.x), to_cm(edge.z), to_cm(edge.y));
    // A step out of the pad, square to its side, so the path arrives
    // straight on.
    let (ox, oz) = match r.side {
        Side::South => (0, -800),
        Side::North => (0, 800),
        Side::West => (-800, 0),
        Side::East => (800, 0),
    };
    let (ix, iz) = (-ox / 4, -oz / 4);
    vec![
        at(r.from.0, r.from.1, r.from.2, 350),
        at(x + ox, z + oz, y, 300),
        at(x + ix, z + iz, y, 300),
    ]
}

/// Every way of the land: the road, the town's roads, the side paths, the
/// water.
fn ways() -> Vec<Way> {
    let mut out = Vec::new();
    let road = |points: &[Point]| Way {
        points: points.to_vec(),
        path: true,
        water: None,
    };
    out.push(road(&mouth::WAY));
    out.push(road(&bank::WAY));
    out.push(road(&shelves::WAY));
    out.push(road(&shelves::CLIMB));
    out.push(road(&pinewood::WAY));
    out.push(road(&pinewood::ABOVE));
    out.push(road(&saddle::WAY));
    // The ranges' pockets of meadow beside the road (`ranges`): level
    // ground for a creature to roam, the land rising round it.
    // A herd that roams has no pocket: its ground is the valley's own.
    for r in super::ranges::RANGES
        .iter()
        .filter(|r| r.stacked.is_none() && !r.roams())
    {
        if r.arena == super::ranges::RANGES[0].arena {
            continue; // the Wood's edge is the clearing's own floor
        }
        let (x, z, half) = (r.at.0, r.at.1, r.half + 600);
        out.push(Way {
            points: vec![
                at(x - r.half / 2, z, r.floor, half),
                at(x + r.half / 2, z, r.floor, half),
            ],
            path: false,
            water: None,
        });
    }
    // The low ground round a station that stands in the open.
    for m in &stations::MEADOWS {
        out.push(Way {
            points: m.to_vec(),
            path: false,
            water: None,
        });
    }
    out.push(road(&wood::WAY));
    out.push(road(&deep::PATH));
    out.push(Way {
        points: spire::PLAZA.to_vec(),
        path: false,
        water: None,
    });
    out.push(Way {
        points: wood::CLEARING.to_vec(),
        path: false,
        water: None,
    });
    for r in &TOWN_ROADS {
        out.push(road(r));
    }
    for r in &ROOMS {
        // The Ring and the Long Valley have the town's roads, and a station
        // no path at all: it is jumped to.
        if r.station || matches!(r.arena, ArenaId::RING | ArenaId::SIEGESHELL) {
            continue;
        }
        out.push(road(&side_path(r)));
    }
    out.push(Way {
        points: mouth::RIVER.to_vec(),
        path: false,
        water: Some(80),
    });
    out.push(Way {
        points: shelves::TARN.to_vec(),
        path: false,
        water: Some(150),
    });
    out.push(Way {
        points: pinewood::POOL.to_vec(),
        path: false,
        water: Some(150),
    });
    out
}

/// The highest and lowest the land is over a box's footprint, sampled every
/// metre.
fn ground_span(land: &Land, lo: (Fx, Fx), hi: (Fx, Fx)) -> (Fx, Fx) {
    let (mut low, mut high) = (Fx::from_int(10_000), Fx::from_int(-10_000));
    let mut x = lo.0;
    while x.raw() <= hi.0.raw() {
        let mut z = lo.1;
        while z.raw() <= hi.1.raw() {
            let h = land.height(x, z);
            low = low.min(h);
            high = high.max(h);
            z = z.add(Fx::ONE);
        }
        x = x.add(Fx::ONE);
    }
    (low, high)
}

/// **A crag's boxes**: the pillar, its ledges zig-zagging up the west face a
/// rise apart, and a cairn on its top.
fn crag(land: &Land, c: &Crag, out: &mut Vec<Solid>) {
    let h = Crag::HALF;
    let (low, high) = ground_span(land, (cm(c.x - h), cm(c.z - h)), (cm(c.x + h), cm(c.z + h)));
    let base = high;
    let top = base.add(cm(Crag::RISE * (c.ledges as i32 + 1)));
    out.push(Solid {
        min: V3::new(cm(c.x - h), low.sub(Fx::ONE), cm(c.z - h)),
        max: V3::new(cm(c.x + h), top, cm(c.z + h)),
        material: Material::Rock,
        shape: crate::arena::Shape::Box,
    });
    for k in 1..=c.ledges as i32 {
        let ledge_top = base.add(cm(Crag::RISE * k));
        // Alternating halves of the face, `ACROSS` apart: up and across.
        let half = Crag::ACROSS / 2;
        let (z0, z1) = if k % 2 == 1 {
            (c.z - h + 50, c.z - half)
        } else {
            (c.z + half, c.z + h - 50)
        };
        out.push(Solid {
            min: V3::new(
                cm(c.x - h - Crag::OUT),
                ledge_top.sub(Fx::ratio(1, 2)),
                cm(z0),
            ),
            max: V3::new(cm(c.x - h), ledge_top, cm(z1)),
            material: Material::Rock,
            shape: crate::arena::Shape::Box,
        });
    }
    out.push(Solid {
        min: V3::new(cm(c.x - 100), top, cm(c.z - 100)),
        max: V3::new(cm(c.x + 100), top.add(Fx::ratio(2, 5)), cm(c.z + 100)),
        material: Material::Snow,
        shape: crate::arena::Shape::Box,
    });
}

/// A cairn on the floor: a slab of snow two metres across.
fn cairn(land: &Land, x: i32, z: i32, out: &mut Vec<Solid>) {
    let (low, high) = ground_span(land, (cm(x - 100), cm(z - 100)), (cm(x + 100), cm(z + 100)));
    out.push(Solid {
        min: V3::new(cm(x - 100), low.sub(Fx::ratio(3, 10)), cm(z - 100)),
        max: V3::new(cm(x + 100), high.add(Fx::ratio(2, 5)), cm(z + 100)),
        material: Material::Snow,
        shape: crate::arena::Shape::Box,
    });
}

/// Is a map point clear of everything a tree or a boulder must not stand on:
/// a pad, the road, water, a crag, a seam?
fn clear(land: &Land, x: Fx, z: Fx, crags: &[Crag], keep: Fx) -> bool {
    if land.on_pad(x, z).is_some() {
        return false;
    }
    let s = land.sample(x, z);
    if s.water.is_some() || s.path.raw() < keep.raw() {
        return false;
    }
    if land.steepness(x, z).raw() > Fx::ratio(7, 10).raw() {
        return false;
    }
    let ([x0, z0], [x1, z1]) = waterfall::CLEARING;
    let at = waterfall::VALLEY_AT;
    if x.raw() >= cm(at[0] + x0).raw()
        && x.raw() <= cm(at[0] + x1).raw()
        && z.raw() >= cm(at[2] + z0).raw()
        && z.raw() <= cm(at[2] + z1).raw()
    {
        return false;
    }
    // Nothing grows up through a station's stones, nor at their feet.
    let near_a_stone = stations::ROUTES.iter().any(|r| {
        core::iter::once(&r.from).chain(r.stones.iter()).any(|s| {
            let reach = cm(s.half + 400);
            x.sub(cm(s.x)).abs().raw() <= reach.raw() && z.sub(cm(s.z)).abs().raw() <= reach.raw()
        })
    });
    if near_a_stone {
        return false;
    }
    // Nor on a roaming herd's loop: it walks there, and is drawn walking.
    if on_a_herd_s_way(x, z) {
        return false;
    }
    crags.iter().all(|c| {
        let dx = x.sub(cm(c.x)).abs();
        let dz = z.sub(cm(c.z)).abs();
        dx.raw() > cm(Crag::HALF + 500).raw() || dz.raw() > cm(Crag::HALF + 500).raw()
    })
}

/// **Is a map point on a roaming herd's way** (`ranges::Range::walk`):
/// within [`HERD_WAY`] of a leg of its loop, as its bodies walk it -- the
/// loop moved as its formation is, from the middle of its marks.
fn on_a_herd_s_way(x: Fx, z: Fx) -> bool {
    let (px, pz) = (
        ((x.raw() as i64) * 100) >> 16,
        ((z.raw() as i64) * 100) >> 16,
    );
    super::ranges::RANGES.iter().filter(|r| r.roams()).any(|r| {
        let n = r.walk.len();
        let lift = r.half as i64 / 3;
        (0..n).any(|i| {
            let (a, b) = (r.walk[i], r.walk[(i + 1) % n]);
            let (ax, az) = (a.0 as i64, a.1 as i64 + lift);
            let (bx, bz) = (b.0 as i64, b.1 as i64 + lift);
            let (vx, vz) = (bx - ax, bz - az);
            let len2 = (vx * vx + vz * vz).max(1);
            let t = (((px - ax) * vx + (pz - az) * vz) * 1000 / len2).clamp(0, 1000);
            let (cx, cz) = (ax + vx * t / 1000, az + vz * t / 1000);
            let (dx, dz) = (px - cx, pz - cz);
            dx * dx + dz * dz <= HERD_WAY * HERD_WAY
        })
    })
}

/// How far either side of a roaming herd's loop nothing grows, centimetres:
/// its formation's width and a body's.
const HERD_WAY: i64 = 900;

/// **The trees and the boulders**: every [`SPACING`] a candidate, jittered,
/// kept by the reach's density where the ground is clear.
fn growth(land: &Land, crags: &[Crag], out: &mut Vec<Solid>) {
    for &(id, per_mille, pines) in &WOODS {
        let b = id.get().bounds;
        let to_cm = |v: Fx| (v.raw() as i64 * 100 / Fx::ONE.raw() as i64) as i32;
        let (x0, x1, z0, z1) = (to_cm(b.lo_x), to_cm(b.hi_x), to_cm(b.lo_z), to_cm(b.hi_z));
        let mut gx = x0;
        while gx < x1 {
            let mut gz = z0;
            while gz < z1 {
                let jx = gx + (dice(gx, gz, 1) as i32 * SPACING / 1000);
                let jz = gz + (dice(gx, gz, 2) as i32 * SPACING / 1000);
                let (x, z) = (cm(jx), cm(jz));
                let roll = dice(gx, gz, 3);
                if roll < per_mille && clear(land, x, z, crags, Fx::from_int(5)) {
                    let rise = land.sample(x, z).rise;
                    // Thinner up the slopes, none past the tree line.
                    if rise.raw() < Fx::from_int(28).raw()
                        && dice(gx, gz, 4) as i32 * 28 > rise.to_int() * 1000
                    {
                        let tall = if pines { 700 } else { 550 } + dice(gx, gz, 5) as i32 * 3 / 10;
                        let half = if pines { 30 } else { 35 };
                        let (low, _) = ground_span(
                            land,
                            (cm(jx - half), cm(jz - half)),
                            (cm(jx + half), cm(jz + half)),
                        );
                        out.push(Solid {
                            min: V3::new(cm(jx - half), low.sub(Fx::ratio(1, 2)), cm(jz - half)),
                            max: V3::new(cm(jx + half), low.add(cm(tall)), cm(jz + half)),
                            material: Material::Wood,
                            shape: crate::arena::Shape::Box,
                        });
                    }
                } else if roll > 985 && clear(land, x, z, crags, Fx::from_int(4)) {
                    // A boulder, now and then: low enough to hop.
                    let w = 80 + dice(gx, gz, 6) as i32 / 5;
                    let d = 80 + dice(gx, gz, 7) as i32 / 5;
                    let h = 60 + dice(gx, gz, 8) as i32 / 8;
                    let (low, high) = ground_span(
                        land,
                        (cm(jx - w / 2), cm(jz - d / 2)),
                        (cm(jx + w / 2), cm(jz + d / 2)),
                    );
                    out.push(Solid {
                        min: V3::new(cm(jx - w / 2), low.sub(Fx::ratio(1, 2)), cm(jz - d / 2)),
                        max: V3::new(cm(jx + w / 2), high.add(cm(h)), cm(jz + d / 2)),
                        material: Material::Rock,
                        shape: crate::arena::Shape::Box,
                    });
                }
                gz += SPACING;
            }
            gx += SPACING;
        }
    }
}

/// **The plan**: every place put down, the land between them, every doorway,
/// every crag, cairn, tree and boulder.
/// **The tutorial halls on the map** (2026-10-25): a row of them past the
/// valley's north edge, each on its own floor inside its own rim, and no door
/// -- the way in is the Esc menu's trip, and it takes only whoever asked
/// (`World::go_alone`). On the map rather than a world of their own so that
/// two people who each go to one are in it together, and one who goes is
/// not taking everybody else with them. Centimetres: x, floor, z.
pub fn hall_at(k: usize) -> [i32; 3] {
    [0, 0, HALL_ROW + k as i32 * HALL_GAP]
}

/// Where the row of halls starts, past the map's north edge (490 m), and
/// how far apart they are: a hall's rim is under thirty metres across.
const HALL_ROW: i32 = 64_000;
const HALL_GAP: i32 = 5_000;

pub fn plan() -> Plan {
    let mut plan = Plan {
        void: cm(VOID),
        ..Plan::default()
    };
    // The places with their own floors first, so a point on one is on it;
    // then the reaches, which are the land.
    plan.places.push(PlanPlace {
        arena: ArenaId::HEARTH,
        at: V3::ZERO,
        apron: cm(APRON),
        pad: true,
        band: None,
    });
    for r in &ROOMS {
        plan.places.push(PlanPlace {
            arena: r.arena,
            at: v3(r.at),
            apron: apron_of(r),
            pad: true,
            band: None,
        });
    }
    for (k, t) in crate::tutorial::ALL.iter().enumerate() {
        plan.places.push(PlanPlace {
            arena: t.arena,
            at: v3(hall_at(k)),
            apron: Fx::ZERO,
            pad: true,
            band: None,
        });
    }
    // The ranges: places on the land, so first on the map before the reaches
    // they stand in. The roost is stacked over the Needle's plaza, between
    // the dome's top and some way over it.
    for r in &super::ranges::RANGES {
        let floor = cm(r.floor);
        plan.places.push(PlanPlace {
            arena: r.arena,
            at: V3::new(cm(r.at.0), floor, cm(r.at.1)),
            apron: Fx::ZERO,
            pad: false,
            band: r
                .stacked
                .map(|up| (floor.sub(Fx::from_int(2)), floor.add(cm(up)))),
        });
    }
    // The High Plane and the Deep after the ranges in them, so a point on
    // a range is on it.
    // The High Plane: its own floor, 300 m up.
    plan.places.push(PlanPlace {
        arena: ArenaId::HIGH_PLANE,
        at: v3(heights::PLANE_AT),
        apron: Fx::ZERO,
        pad: true,
        band: None,
    });
    // The Deep: its own floor under the whole lid.
    plan.places.push(PlanPlace {
        arena: ArenaId::DEEP,
        at: v3(deep::AT),
        apron: Fx::ZERO,
        pad: true,
        band: None,
    });
    for id in [
        ArenaId::MOUTH,
        ArenaId::BANK,
        ArenaId::SHELVES,
        ArenaId::PINEWOOD,
        ArenaId::SADDLE,
        ArenaId::WOOD,
        ArenaId::SPIRE,
        ArenaId::SKY_ROAD,
    ] {
        plan.places.push(PlanPlace {
            arena: id,
            at: V3::ZERO,
            apron: Fx::ZERO,
            pad: false,
            band: None,
        });
    }

    let land = Land::new(&ways(), pads());

    // Doors: the passes between reaches, and each room's doorway.
    for (a, b, cut) in PASSES {
        plan.doors.push(DoorPlan {
            a,
            b,
            cut: Solid::cm(cut[0], cut[1], Material::Stone),
        });
    }
    for r in &ROOMS {
        let (_, cut) = door_of(r);
        let door = match r.arena {
            ArenaId::GALEWING => stations::CLIFFS_LIP,
            ArenaId::MANTIS => stations::shrine_gate(&cut),
            _ => cut,
        };
        plan.doors.push(DoorPlan {
            a: r.seam,
            b: (r.arena, 0),
            cut: door,
        });
        if r.depth > 0 {
            plan.carves.push(cut);
        }
    }

    // What stands on the land.
    let mut crags: Vec<Crag> = Vec::new();
    crags.extend(mouth::CRAGS);
    crags.extend(bank::CRAGS);
    crags.extend(shelves::CRAGS);
    crags.extend(pinewood::CRAGS);
    crags.extend(saddle::CRAGS);
    for c in &crags {
        crag(&land, c, &mut plan.extras);
    }
    for &(x, z) in mouth::CAIRNS
        .iter()
        .chain(bank::CAIRNS.iter())
        .chain(shelves::CAIRNS.iter())
        .chain(pinewood::CAIRNS.iter())
        .chain(saddle::CAIRNS.iter())
        .chain(wood::CAIRNS.iter())
    {
        cairn(&land, x, z, &mut plan.extras);
    }
    // The Waterfall, at the head of the Pinewood: the cliff the road climbs,
    // and the cairn on its top.
    plan.extras.extend(waterfall::placed(waterfall::VALLEY_AT));
    plan.extras
        .push(waterfall::solid(&waterfall::CROWN, waterfall::VALLEY_AT));
    // The Great Wood's giants, on the land under them.
    // The Great Wood's giants, grown where they stand on the land: their
    // wood is drawn from their skeletons, not as boxes.
    let to_cm = |v: Fx| (v.raw() as i64 * 100 / Fx::ONE.raw() as i64) as i32;
    let ground = |x: i32, z: i32| to_cm(land.height(cm(x), cm(z)));
    let mut grown = wood::grow(&ground);
    let mut wood_boxes = Vec::new();
    let mut wood_cairns = Vec::new();
    wood::build(&grown, &mut wood_boxes, &mut wood_cairns);
    let ways = wood::ways_up(&mut grown, &wood_boxes, &ground);
    wood_boxes.extend(ways.boxes);
    wood_cairns.extend(ways.cairns);
    plan.drifts.extend(ways.drifts);
    plan.vents.extend(ways.vents);
    let first = plan.extras.len();
    plan.extras.extend(wood_boxes);
    plan.hidden.push(first..plan.extras.len());
    plan.extras.extend(wood_cairns);
    plan.trees = grown;
    // The Spire, on its plaza under the town's south wall.
    spire::build(
        &|x, z| land.height(x, z),
        &mut plan.extras,
        &mut plan.drifts,
    );
    // The Deep: its lid, its warrens and its Caverns.
    deep::build(&mut plan.extras);
    // The stations' routes.
    stations::build(
        &|x, z| land.height(x, z),
        &mut plan.extras,
        &mut plan.drifts,
    );
    // The routes up to the High Plane.
    heights::build(
        &|x, z| land.height(x, z),
        &mut plan.extras,
        &mut plan.drifts,
    );
    growth(&land, &crags, &mut plan.extras);

    plan.land = Some(land);
    plan
}

/// **The way into a room, on foot**: points on the map from the road's
/// junction to a few metres inside the room's ground, by its side path or
/// the town's road -- or, for a station, from its ledge, which is jumped to
/// (`arena::stations`). What a walk through the valley follows
/// (`tests/valley.rs`).
pub fn approach(r: &Room) -> Vec<(Fx, Fx)> {
    let p = |x: i32, z: i32| (cm(x), cm(z));
    let (edge, _) = door_of(r);
    let inward = match r.side {
        Side::South => (Fx::ZERO, Fx::from_int(5)),
        Side::North => (Fx::ZERO, Fx::from_int(-5)),
        Side::West => (Fx::from_int(5), Fx::ZERO),
        Side::East => (Fx::from_int(-5), Fx::ZERO),
    };
    let inside = (edge.x.add(inward.0), edge.z.add(inward.1));
    if r.station {
        // From its ledge, in through its door: five metres past the door
        // (on the Cliffs, whose ledge is the plateau's lip, five metres on
        // from it).
        let ledge = stations::ledge_of(r.arena);
        let far = cm(stations::LEDGE_OUT + 500).div(Fx::from_int(5));
        let to = (
            cm(ledge.x).add(inward.0.mul(far)),
            cm(ledge.z).add(inward.1.mul(far)),
        );
        return vec![p(ledge.x, ledge.z), to];
    }
    let mut out = vec![p(r.from.0, r.from.1)];
    for q in side_path(r).iter().skip(1) {
        out.push(p(q.x, q.z));
    }
    out.push(inside);
    out
}

/// Where a room's door meets its ground on the map (centimetres): the
/// middle of its doorway at its floor, on the edge of its rim.
pub fn door_point(r: &Room) -> [i32; 3] {
    let (edge, _) = door_of(r);
    let to_cm = |v: Fx| (v.raw() as i64 * 100 / Fx::ONE.raw() as i64) as i32;
    [to_cm(edge.x), to_cm(edge.y), to_cm(edge.z)]
}

/// **The road, on foot**: from the town's east gate to the end of the
/// Saddle, every point of it in order, in **legs**. Between two legs the road
/// is a climb, not a walk: the Waterfall, at the head of the Pinewood, whose
/// foot is the end of one leg and whose top is the start of the next.
pub fn road() -> Vec<Vec<(Fx, Fx)>> {
    let mut legs = Vec::new();
    let mut out: Vec<(Fx, Fx)> = Vec::new();
    for (way, climb_after) in [
        (&mouth::WAY[..], false),
        (&bank::WAY[..], false),
        (&shelves::WAY[..], false),
        (&shelves::CLIMB[..], false),
        (&pinewood::WAY[..], true),
        (&pinewood::ABOVE[..], false),
        (&saddle::WAY[..], false),
    ] {
        for q in way {
            let at = (cm(q.x), cm(q.z));
            if out.last() != Some(&at) {
                out.push(at);
            }
        }
        if climb_after {
            legs.push(std::mem::take(&mut out));
        }
    }
    legs.push(out);
    legs
}
