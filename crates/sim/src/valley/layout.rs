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

use crate::arena::{bank, hearth, mouth, pinewood, saddle, shelves};

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
    }
}

/// **The rooms**: the Ring and the Long Valley off the town, the creatures'
/// rooms off the road.
pub const ROOMS: [Room; 12] = [
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
    room(
        ArenaId::HORNBACK,
        [11500, 50, 7400],
        Side::South,
        0,
        250,
        (11500, 1400, 50),
        (ArenaId::MOUTH, 1),
    ),
    room(
        ArenaId::GNAWERS,
        [30500, 2100, 6500],
        Side::South,
        0,
        350,
        (30500, 1200, 2100),
        (ArenaId::BANK, 1),
    ),
    room(
        ArenaId::MIREBACK,
        [51500, 4500, -8600],
        Side::North,
        400,
        150,
        (51500, -3000, 4500),
        (ArenaId::SHELVES, 1),
    ),
    room(
        ArenaId::SANDMAW,
        [56800, 4650, -8400],
        Side::North,
        0,
        150,
        (56000, -2800, 4650),
        (ArenaId::SHELVES, 2),
    ),
    room(
        ArenaId::PAIR,
        [70200, 5700, -4400],
        Side::North,
        0,
        200,
        (70200, 400, 5700),
        (ArenaId::PINEWOOD, 1),
    ),
    room(
        ArenaId::BROODMOTHER,
        [73500, 5750, 5800],
        Side::South,
        0,
        200,
        (73500, 1000, 5750),
        (ArenaId::PINEWOOD, 2),
    ),
    room(
        ArenaId::HIGHLANDS,
        [76500, 6400, -4600],
        Side::North,
        0,
        150,
        (76500, -200, 6000),
        (ArenaId::PINEWOOD, 3),
    ),
    // The Cliffs sit sunk, their plateau at the road's height: the way in is
    // a bridge over the ravine round them, not a doorway (`CLIFFS_BRIDGE`).
    room(
        ArenaId::GALEWING,
        [90500, 7300, 7400],
        Side::South,
        0,
        0,
        (90500, 1000, 8500),
        (ArenaId::SADDLE, 1),
    ),
    room(
        ArenaId::VEILSTALKER,
        [93500, 8600, -6100],
        Side::North,
        0,
        200,
        (93500, -600, 8600),
        (ArenaId::SADDLE, 2),
    ),
    // The Shrine is at the top of its own path (`saddle::SHRINE_WAY`).
    room(
        ArenaId::MANTIS,
        [99500, 11000, 8000],
        Side::South,
        0,
        200,
        (100000, 400, 9000),
        (ArenaId::SADDLE, 3),
    ),
];

/// The bridge into the Cliffs: from the road's edge over the ravine to the
/// plateau's lip, at the road's height.
const CLIFFS_BRIDGE: Solid = Solid::cm([90200, 8440, 2600], [90800, 8500, 5250], Material::Wood);

/// **The doors between reaches**, and the waystones' among them: the two
/// seams, and the box across the pass (shut while its stone is dark).
/// A door between two seams, and its box: two corners in centimetres.
type Pass = ((ArenaId, u8), (ArenaId, u8), [[i32; 3]; 2]);

const PASSES: [Pass; 5] = [
    (
        (ArenaId::HEARTH, 0),
        (ArenaId::MOUTH, 0),
        [[4400, 0, -300], [5200, 600, 300]],
    ),
    (
        (ArenaId::MOUTH, 2),
        (ArenaId::BANK, 0),
        [[24800, 1900, -1200], [25200, 2600, 1200]],
    ),
    (
        (ArenaId::BANK, 2),
        (ArenaId::SHELVES, 0),
        [[42900, 4300, -1600], [43100, 5400, 800]],
    ),
    (
        (ArenaId::SHELVES, 3),
        (ArenaId::PINEWOOD, 0),
        [[65100, 5500, -1000], [65300, 6600, 1800]],
    ),
    (
        (ArenaId::PINEWOOD, 4),
        (ArenaId::SADDLE, 0),
        [[85100, 8100, -800], [85300, 9200, 2000]],
    ),
];

/// The fourth waystone's door: across the Shrine's path, above the road.
const SHRINE_GATE: [[i32; 3]; 2] = [[99600, 8900, 1600], [102400, 10200, 1800]];

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
    });
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
    out.push(road(&saddle::WAY));
    out.push(road(&saddle::SHRINE_WAY));
    for r in &TOWN_ROADS {
        out.push(road(r));
    }
    for r in &ROOMS {
        // The Shrine has its own path up, the Ring and the Long Valley the
        // town's roads, and the Cliffs a bridge at the road's height.
        if matches!(
            r.arena,
            ArenaId::MANTIS | ArenaId::RING | ArenaId::SIEGESHELL | ArenaId::GALEWING
        ) {
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
    });
    for k in 1..=c.ledges as i32 {
        let ledge_top = base.add(cm(Crag::RISE * k));
        // Alternating halves of the face, a metre apart: up and across.
        let (z0, z1) = if k % 2 == 1 {
            (c.z - h + 50, c.z - 50)
        } else {
            (c.z + 50, c.z + h - 50)
        };
        out.push(Solid {
            min: V3::new(
                cm(c.x - h - Crag::OUT),
                ledge_top.sub(Fx::ratio(1, 2)),
                cm(z0),
            ),
            max: V3::new(cm(c.x - h), ledge_top, cm(z1)),
            material: Material::Rock,
        });
    }
    out.push(Solid {
        min: V3::new(cm(c.x - 100), top, cm(c.z - 100)),
        max: V3::new(cm(c.x + 100), top.add(Fx::ratio(2, 5)), cm(c.z + 100)),
        material: Material::Snow,
    });
}

/// A cairn on the floor: a slab of snow two metres across.
fn cairn(land: &Land, x: i32, z: i32, out: &mut Vec<Solid>) {
    let (low, high) = ground_span(land, (cm(x - 100), cm(z - 100)), (cm(x + 100), cm(z + 100)));
    out.push(Solid {
        min: V3::new(cm(x - 100), low.sub(Fx::ratio(3, 10)), cm(z - 100)),
        max: V3::new(cm(x + 100), high.add(Fx::ratio(2, 5)), cm(z + 100)),
        material: Material::Snow,
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
    crags.iter().all(|c| {
        let dx = x.sub(cm(c.x)).abs();
        let dz = z.sub(cm(c.z)).abs();
        dx.raw() > cm(Crag::HALF + 500).raw() || dz.raw() > cm(Crag::HALF + 500).raw()
    })
}

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
    });
    for r in &ROOMS {
        plan.places.push(PlanPlace {
            arena: r.arena,
            at: v3(r.at),
            apron: apron_of(r),
            pad: true,
        });
    }
    for id in [
        ArenaId::MOUTH,
        ArenaId::BANK,
        ArenaId::SHELVES,
        ArenaId::PINEWOOD,
        ArenaId::SADDLE,
    ] {
        plan.places.push(PlanPlace {
            arena: id,
            at: V3::ZERO,
            apron: Fx::ZERO,
            pad: false,
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
            ArenaId::GALEWING => CLIFFS_BRIDGE,
            ArenaId::MANTIS => Solid::cm(SHRINE_GATE[0], SHRINE_GATE[1], Material::Stone),
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
    plan.extras.push(CLIFFS_BRIDGE);

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
    {
        cairn(&land, x, z, &mut plan.extras);
    }
    growth(&land, &crags, &mut plan.extras);

    plan.land = Some(land);
    plan
}

/// **The way into a room, on foot**: points on the map from the road's
/// junction to a few metres inside the room's ground -- by its side path, the
/// Shrine's switchback, the Cliffs' bridge, or the town's road. What a walk
/// through the valley follows (`tests/valley.rs`).
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
    let mut out = vec![p(r.from.0, r.from.1)];
    match r.arena {
        ArenaId::MANTIS => out.extend(saddle::SHRINE_WAY.iter().map(|q| p(q.x, q.z))),
        ArenaId::GALEWING => {
            let b = CLIFFS_BRIDGE;
            let x = Fx::from_raw(b.min.x.raw() / 2 + b.max.x.raw() / 2);
            out.push((x, b.min.z.add(Fx::ONE)));
            out.push((x, b.max.z.add(Fx::from_int(3))));
            return out;
        }
        _ => {
            for q in side_path(r).iter().skip(1) {
                out.push(p(q.x, q.z));
            }
        }
    }
    out.push(inside);
    out
}

/// **The road, on foot**: from the town's east gate to the end of the
/// Saddle, every point of it in order.
pub fn road() -> Vec<(Fx, Fx)> {
    let mut out = Vec::new();
    for way in [
        &mouth::WAY[..],
        &bank::WAY[..],
        &shelves::WAY[..],
        &shelves::CLIMB[..],
        &pinewood::WAY[..],
        &saddle::WAY[..],
    ] {
        for q in way {
            let at = (cm(q.x), cm(q.z));
            if out.last() != Some(&at) {
                out.push(at);
            }
        }
    }
    out
}
