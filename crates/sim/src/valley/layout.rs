//! **The valley as one map**: where every place goes, worked out from how
//! they join.
//!
//! The places were drawn as rooms joined by teleports, each in its own
//! coordinates. Nothing here moves a box inside one of them. What this
//! table says is only *which seam meets which*, and how:
//!
//! - **A passage** meets a passage end to end: the Mouth's way on to the Bank
//!   and the Bank's way back are one corridor. The second place goes where
//!   its passage's mouth touches the first's, on the same line, with the two
//!   floors at the same height, and the caps that closed both ends are cut.
//! - **A room** sits beyond the notch that led to it, with
//!   [`APRON`] of ground all round it and a cliff round that, so a low wall
//!   hopped is a fall onto grass rather than out of the world. Its doorway is
//!   cut through the notch's back, the cliff, the apron and the room's own
//!   wall, `depth` past the apron, at the height of the notch's floor; a sill
//!   under the cut carries you across whatever is lower (the Cliffs' shelf).
//!   `along` is where on that wall the door goes, in the room's own
//!   coordinates, so it can miss what stands inside.
//!
//! Every position is derived: change a reach's length or a passage's floor
//! and the next place moves with it. `tests/atlas.rs` checks what derivation
//! cannot promise -- that no two places overlap and that every doorway's
//! floor meets on both sides.

use crate::arena::{self, ArenaId, Material, Solid};
use crate::atlas::{DoorPlan, Plan, footprint};
use crate::fixed::Fx;
use crate::math::V3;

/// How two seams meet.
#[derive(Clone, Copy, Debug)]
pub enum Join {
    Passage,
    Room {
        /// Where on the room's wall the door goes, centimetres along it in
        /// the room's own coordinates.
        along: i32,
        /// How far past the apron the doorway is cut: through the room's own
        /// wall and whatever is in the way of its floor.
        depth: i32,
    },
}

/// One joint: the seam of a place already put down, and the seam of the
/// place it leads to.
#[derive(Clone, Copy, Debug)]
pub struct Joint {
    pub a: (ArenaId, u8),
    pub b: (ArenaId, u8),
    pub join: Join,
}

const fn passage(a: ArenaId, sa: u8, b: ArenaId, sb: u8) -> Joint {
    Joint {
        a: (a, sa),
        b: (b, sb),
        join: Join::Passage,
    }
}

const fn room(a: ArenaId, sa: u8, b: ArenaId, along: i32, depth: i32) -> Joint {
    Joint {
        a: (a, sa),
        b: (b, 0),
        join: Join::Room { along, depth },
    }
}

/// Where the map starts: Hearth, at the origin.
pub const ROOT: ArenaId = ArenaId::HEARTH;

/// Ground round a room, beyond its own footprint, in centimetres.
pub const APRON: i32 = 600;

/// How high a room's cliff stands over the tallest thing in it.
const CLIFF_OVER: i32 = 400;

/// How wide a doorway is cut into a room.
const DOOR: i32 = 600;

/// How high every doorway is cut, over its floor.
const DOOR_HIGH: i32 = 600;

/// How far either side of the meeting line a passage's doorway is cut: the
/// caps that closed both ends.
const CAP: i32 = 300;

/// The floor where no place is.
const VOID: i32 = -10_000;

/// **The joints**, in the order they are put down: each one's first place is
/// already on the map.
pub const JOINTS: [Joint; 17] = [
    // The town: the Ring north, the Long Valley west through the fifth
    // waystone's gate, the climb east.
    room(ArenaId::HEARTH, 1, ArenaId::RING, 0, 200),
    room(ArenaId::HEARTH, 2, ArenaId::SIEGESHELL, 0, 600),
    passage(ArenaId::HEARTH, 0, ArenaId::MOUTH, 0),
    // The climb.
    passage(ArenaId::MOUTH, 2, ArenaId::BANK, 0),
    passage(ArenaId::BANK, 2, ArenaId::SHELVES, 0),
    passage(ArenaId::SHELVES, 3, ArenaId::PINEWOOD, 0),
    passage(ArenaId::PINEWOOD, 4, ArenaId::SADDLE, 0),
    // The rooms off it.
    room(ArenaId::MOUTH, 1, ArenaId::HORNBACK, 0, 250),
    room(ArenaId::BANK, 1, ArenaId::GNAWERS, 0, 350),
    room(ArenaId::SHELVES, 1, ArenaId::MIREBACK, 400, 150),
    room(ArenaId::SHELVES, 2, ArenaId::SANDMAW, 0, 150),
    room(ArenaId::PINEWOOD, 1, ArenaId::PAIR, 0, 200),
    room(ArenaId::PINEWOOD, 2, ArenaId::BROODMOTHER, 0, 200),
    room(ArenaId::PINEWOOD, 3, ArenaId::HIGHLANDS, 0, 150),
    room(ArenaId::SADDLE, 1, ArenaId::GALEWING, 0, 550),
    room(ArenaId::SADDLE, 2, ArenaId::VEILSTALKER, 0, 200),
    room(ArenaId::SADDLE, 3, ArenaId::MANTIS, 0, 200),
];

/// Which way a seam faces out of its place: the side of the footprint it is
/// nearest. `(axis, sign)`, axis 0 for x and 2 for z.
fn facing(id: ArenaId, seam: u8) -> (usize, i32) {
    let (lo, hi) = footprint(id.get());
    let zone = seam_zone(id, seam);
    let c = zone.middle();
    let gaps = [
        (hi.0.sub(c.x), (0, 1)),
        (c.x.sub(lo.0), (0, -1)),
        (hi.1.sub(c.z), (2, 1)),
        (c.z.sub(lo.1), (2, -1)),
    ];
    let mut best = gaps[0];
    for g in gaps {
        if g.0.raw() < best.0.raw() {
            best = g;
        }
    }
    best.1
}

fn seam_zone(id: ArenaId, seam: u8) -> super::Zone {
    super::place(id)
        .and_then(|p| p.seam(seam))
        .map(|s| s.zone)
        .unwrap_or(super::Zone {
            min: V3::ZERO,
            max: V3::ZERO,
        })
}

fn get(v: V3, axis: usize) -> Fx {
    if axis == 0 { v.x } else { v.z }
}

fn set(v: &mut V3, axis: usize, to: Fx) {
    if axis == 0 { v.x = to } else { v.z = to }
}

/// The floor under a point of an arena: its relief, or the highest top of
/// anything standing on it there. A cave's roof is not a floor.
fn floor_of(id: ArenaId, x: Fx, z: Fx) -> Fx {
    let arena = id.get();
    arena
        .solids
        .iter()
        .filter(|s| !s.hangs() && s.over(x, z, Fx::ZERO))
        .fold(arena.relief_at(x, z), |best, s| best.max(s.max.y))
}

/// The floor of a seam: the highest top under the middle of its zone that is
/// no higher than the zone's top -- a bridge's deck as much as the ground.
fn seam_floor(id: ArenaId, zone: &super::Zone) -> Fx {
    let arena = id.get();
    let mid = zone.middle();
    arena
        .solids
        .iter()
        .filter(|s| s.over(mid.x, mid.z, Fx::ZERO) && s.max.y.raw() <= zone.max.y.raw())
        .fold(arena.relief_at(mid.x, mid.z), |best, s| best.max(s.max.y))
}

/// **The plan**: every place put down, every doorway, every cliff and sill.
pub fn plan() -> Plan {
    let cm = arena::cm;
    let mut plan = Plan {
        void: cm(VOID),
        ..Plan::default()
    };
    plan.places.push((ROOT, V3::ZERO, Fx::ZERO));
    let at = |plan: &Plan, id: ArenaId| plan.places.iter().find(|p| p.0 == id).map(|p| (p.1, p.2));
    for joint in JOINTS {
        let Some((a_at, a_apron)) = at(&plan, joint.a.0) else {
            continue;
        };
        let (axis, sign) = facing(joint.a.0, joint.a.1);
        let cross = 2 - axis;
        let s = Fx::from_int(sign);
        let a_zone = seam_zone(joint.a.0, joint.a.1);
        let a_mid = a_zone.middle();
        let (a_lo, a_hi) = footprint(joint.a.0.get());
        let a_edge = if sign > 0 {
            get(V3::new(a_hi.0, Fx::ZERO, a_hi.1), axis).add(a_apron)
        } else {
            get(V3::new(a_lo.0, Fx::ZERO, a_lo.1), axis).sub(a_apron)
        };
        // The floor of the seam, on the map.
        let floor = seam_floor(joint.a.0, &a_zone).add(a_at.y);
        let a_edge = a_edge.add(get(a_at, axis));
        let a_cross = get(a_mid, cross).add(get(a_at, cross));

        let b = joint.b.0.get();
        let (b_lo, b_hi) = footprint(b);
        // The side of the second place that meets the first: its far side
        // along the same axis.
        let b_edge_local = if sign > 0 {
            get(V3::new(b_lo.0, Fx::ZERO, b_lo.1), axis)
        } else {
            get(V3::new(b_hi.0, Fx::ZERO, b_hi.1), axis)
        };
        let mut b_at = V3::ZERO;
        match joint.join {
            Join::Passage => {
                let b_mid = seam_zone(joint.b.0, joint.b.1).middle();
                set(&mut b_at, axis, a_edge.sub(b_edge_local));
                set(&mut b_at, cross, a_cross.sub(get(b_mid, cross)));
                let b_zone = seam_zone(joint.b.0, joint.b.1);
                b_at.y = floor.sub(seam_floor(joint.b.0, &b_zone));
                plan.places.push((joint.b.0, b_at, Fx::ZERO));
                // The two caps, either side of the line, the passage's
                // width and a door's height over its floor.
                let (z0, z1) = (get(a_zone.min, cross), get(a_zone.max, cross));
                let mut lo = V3::new(Fx::ZERO, floor, Fx::ZERO);
                let mut hi = V3::new(Fx::ZERO, floor.add(cm(DOOR_HIGH)), Fx::ZERO);
                set(&mut lo, axis, a_edge.sub(cm(CAP)));
                set(&mut hi, axis, a_edge.add(cm(CAP)));
                set(&mut lo, cross, z0.add(get(a_at, cross)));
                set(&mut hi, cross, z1.add(get(a_at, cross)));
                plan.doors.push(DoorPlan {
                    a: joint.a,
                    b: joint.b,
                    cut: Solid {
                        min: lo,
                        max: hi,
                        material: Material::Ground,
                    },
                });
            }
            Join::Room { along, depth } => {
                let apron = cm(APRON);
                let b_edge_local = b_edge_local.sub(s.mul(apron));
                // Where the cut ends inside the room, and the floor there.
                let inner_local = b_edge_local.add(s.mul(apron.add(cm(depth))));
                let mut inner = V3::ZERO;
                set(&mut inner, axis, inner_local);
                set(&mut inner, cross, cm(along));
                let b_floor = floor_of(joint.b.0, inner.x, inner.z);
                set(&mut b_at, axis, a_edge.sub(b_edge_local));
                set(&mut b_at, cross, a_cross.sub(cm(along)));
                b_at.y = floor.sub(b_floor);
                plan.places.push((joint.b.0, b_at, apron));

                // The doorway: from the middle of the notch to `depth` past
                // the apron, a door wide and a door high.
                let half = cm(DOOR / 2);
                let from = get(a_mid, axis).add(get(a_at, axis));
                let to = inner_local.add(get(b_at, axis));
                let mut lo = V3::new(Fx::ZERO, floor, Fx::ZERO);
                let mut hi = V3::new(Fx::ZERO, floor.add(cm(DOOR_HIGH)), Fx::ZERO);
                set(&mut lo, axis, from.min(to));
                set(&mut hi, axis, from.max(to));
                set(&mut lo, cross, a_cross.sub(half));
                set(&mut hi, cross, a_cross.add(half));
                plan.doors.push(DoorPlan {
                    a: joint.a,
                    b: joint.b,
                    cut: Solid {
                        min: lo,
                        max: hi,
                        material: Material::Ground,
                    },
                });
                // The sill: the doorway's floor, whatever is under it.
                plan.extras.push(Solid {
                    min: V3::new(lo.x, floor.sub(Fx::ratio(1, 2)), lo.z),
                    max: V3::new(hi.x, floor, hi.z),
                    material: Material::Ground,
                });

                // The cliff round the apron: four walls, a metre thick,
                // standing from the void to over the tallest thing in the
                // room. The doorway cuts the one it passes through.
                let tallest = b
                    .solids
                    .iter()
                    .fold(Fx::ZERO, |t, s| t.max(s.max.y))
                    .add(cm(CLIFF_OVER))
                    .add(b_at.y);
                let (lo_x, lo_z) = (b_lo.0.sub(apron).add(b_at.x), b_lo.1.sub(apron).add(b_at.z));
                let (hi_x, hi_z) = (b_hi.0.add(apron).add(b_at.x), b_hi.1.add(apron).add(b_at.z));
                let one = Fx::ONE;
                let bottom = cm(VOID);
                let wall = |x0: Fx, z0: Fx, x1: Fx, z1: Fx| Solid {
                    min: V3::new(x0, bottom, z0),
                    max: V3::new(x1, tallest, z1),
                    material: Material::Rock,
                };
                plan.extras
                    .push(wall(lo_x.sub(one), lo_z.sub(one), hi_x.add(one), lo_z));
                plan.extras
                    .push(wall(lo_x.sub(one), hi_z, hi_x.add(one), hi_z.add(one)));
                plan.extras.push(wall(lo_x.sub(one), lo_z, lo_x, hi_z));
                plan.extras.push(wall(hi_x, lo_z, hi_x.add(one), hi_z));
            }
        }
    }
    plan
}
