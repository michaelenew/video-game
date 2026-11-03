//! **Big, bigger, biggest** (2026-11-02, `docs/design/fight-spaces.md` §6):
//! every King of the Hill map in three sizes, to flip through and play.
//!
//! The owner: "try the existing koth maps, aiming for big / bigger /
//! biggest on each of their scale ups so I can flip through with my
//! friends." The map as built is *big*; *bigger* and *biggest* are the same
//! map **stretched across the floor** -- every x and z times five quarters,
//! or times three halves -- with **every height kept**. So a stair is still
//! a stair, a level is still a jump and a half over the one below, a ledge
//! still catches; what grows is the ground between things: the walk from a
//! start, the gap from a perch to the point, the width of a lane. The point
//! itself stays four metres across: the objective is smaller than the fight.
//!
//! A stretched map is made at compile time from its base's own tables (the
//! functions here are `const`), so it is a table like any other -- an
//! `Arena` and a `Stage` with an id of its own, in the picker's order right
//! after its base (`versus::STAGES`). The ways its base promises are walked
//! on it stretched the same way ([`plan_of`], `tests/versus.rs`).

use super::super::relief::Bump;
use super::super::rim::Rim;
use super::super::{Area, ArenaId, Bounds, Mark, Region, Shape, Solid};
use crate::drift::Drift;
use crate::fixed::Fx;
use crate::math::V3;
use crate::valley::Vent;
use crate::versus::Hill;

/// **How much a map is stretched across the floor**: x and z times
/// `num / den`; heights never.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plan {
    pub num: i32,
    pub den: i32,
}

/// The map as built.
pub const BIG: Plan = Plan { num: 1, den: 1 };
/// A quarter more ground each way.
pub const BIGGER: Plan = Plan { num: 5, den: 4 };
/// Half as much again each way.
pub const BIGGEST: Plan = Plan { num: 3, den: 2 };

impl Plan {
    /// A length across the floor, stretched.
    pub const fn fx(self, v: Fx) -> Fx {
        Fx::from_raw((v.raw() as i64 * self.num as i64 / self.den as i64) as i32)
    }

    /// A length in centimetres, stretched.
    pub const fn cm(self, v: i32) -> i32 {
        v * self.num / self.den
    }

    /// A point stretched across the floor, its height kept.
    pub const fn v3(self, v: V3) -> V3 {
        V3::new(self.fx(v.x), v.y, self.fx(v.z))
    }
}

/// Which map an id is a stretch of, and how far: `None` for anything that
/// is not a stretched map (a map as built is its own base, at [`BIG`]).
pub const fn base_of(id: ArenaId) -> Option<(ArenaId, Plan)> {
    Some(match id {
        ArenaId::QUARRY_BIGGER => (ArenaId::QUARRY, BIGGER),
        ArenaId::QUARRY_BIGGEST => (ArenaId::QUARRY, BIGGEST),
        ArenaId::GLACIER_BIGGER => (ArenaId::GLACIER, BIGGER),
        ArenaId::GLACIER_BIGGEST => (ArenaId::GLACIER, BIGGEST),
        ArenaId::DELTA_BIGGER => (ArenaId::DELTA, BIGGER),
        ArenaId::DELTA_BIGGEST => (ArenaId::DELTA, BIGGEST),
        ArenaId::GORGE_BIGGER => (ArenaId::GORGE, BIGGER),
        ArenaId::GORGE_BIGGEST => (ArenaId::GORGE, BIGGEST),
        ArenaId::HENGE_BIGGER => (ArenaId::HENGE, BIGGER),
        ArenaId::HENGE_BIGGEST => (ArenaId::HENGE, BIGGEST),
        _ => return None,
    })
}

/// The id an arena's look, sky and dressing are taken from: its base's, for
/// a stretched map; its own for anything else.
pub const fn looks_like(id: ArenaId) -> ArenaId {
    match base_of(id) {
        Some((base, _)) => base,
        None => id,
    }
}

/// How far an arena is stretched from the table its ways were written for:
/// [`BIG`] for anything not stretched.
pub const fn plan_of(id: ArenaId) -> Plan {
    match base_of(id) {
        Some((_, p)) => p,
        None => BIG,
    }
}

/// **Where a way's point is on a stretched map** (centimetres): stretched
/// with the floor, unless it is on the base map's point -- the point is as
/// wide on every size, so a way that ends on it ends on it still.
pub fn way_point(id: ArenaId, at: (i32, i32, i32)) -> (i32, i32, i32) {
    let Some((base, p)) = base_of(id) else {
        return at;
    };
    let on = crate::versus::of(base)
        .and_then(|s| s.hill)
        .is_some_and(|h| {
            h.holds(V3::new(
                Fx::ratio(at.0, 100),
                Fx::ratio(at.1, 100),
                Fx::ratio(at.2, 100),
            ))
        });
    if on {
        at
    } else {
        (p.cm(at.0), at.1, p.cm(at.2))
    }
}

/// A solid stretched: its footprint and its shape's widths across the
/// floor, its heights kept.
pub const fn solid(s: Solid, p: Plan) -> Solid {
    Solid {
        min: p.v3(s.min),
        max: p.v3(s.max),
        material: s.material,
        shape: match s.shape {
            Shape::Box => Shape::Box,
            Shape::Round {
                bottom,
                top,
                hollow,
            } => Shape::Round {
                bottom: p.fx(bottom),
                top: p.fx(top),
                hollow: p.fx(hollow),
            },
            Shape::Turned {
                half,
                half_wide,
                cos,
                sin,
            } => Shape::Turned {
                half: p.fx(half),
                half_wide: p.fx(half_wide),
                cos,
                sin,
            },
        },
    }
}

/// How narrow a thing over the point must be to keep its width when the
/// map is stretched: as wide as the point, four metres either way of its
/// middle.
const NARROW: Fx = Fx::from_int(4);

/// A solid over the point (every map's point is at its middle) narrower
/// than [`NARROW`] across one way -- a bridge, a lintel -- keeps that width
/// and is stretched only along its length, so a walkway over the point is
/// as wide on every size and a way onto the point does not end under its
/// edge.
const fn over_the_point(s: Solid, p: Plan) -> Solid {
    let mut out = solid(s, p);
    let over = s.min.x.raw() < 0 && s.max.x.raw() > 0 && s.min.z.raw() < 0 && s.max.z.raw() > 0;
    if !over || !matches!(s.shape, Shape::Box) {
        return out;
    }
    let two = NARROW.raw() as i64 * 2;
    if (s.max.x.raw() as i64 - s.min.x.raw() as i64) <= two {
        out.min.x = s.min.x;
        out.max.x = s.max.x;
    }
    if (s.max.z.raw() as i64 - s.min.z.raw() as i64) <= two {
        out.min.z = s.min.z;
        out.max.z = s.max.z;
    }
    out
}

/// Every solid of a table, stretched.
pub const fn solids<const N: usize>(base: [Solid; N], p: Plan) -> [Solid; N] {
    let mut out = base;
    let mut i = 0;
    while i < N {
        out[i] = over_the_point(base[i], p);
        i += 1;
    }
    out
}

/// Every floor region, stretched.
pub const fn regions<const N: usize>(base: [Region; N], p: Plan) -> [Region; N] {
    let mut out = base;
    let mut i = 0;
    while i < N {
        out[i].area = match base[i].area {
            Area::Rect { lo, hi } => Area::Rect {
                lo: (p.fx(lo.0), p.fx(lo.1)),
                hi: (p.fx(hi.0), p.fx(hi.1)),
            },
            Area::Disc { at, radius } => Area::Disc {
                at: (p.fx(at.0), p.fx(at.1)),
                radius: p.fx(radius),
            },
        };
        i += 1;
    }
    out
}

/// Every relief bump, stretched: wider, and as high, so gentler.
pub const fn bumps<const N: usize>(base: [Bump; N], p: Plan) -> [Bump; N] {
    let mut out = base;
    let mut i = 0;
    while i < N {
        out[i].x = p.fx(base[i].x);
        out[i].z = p.fx(base[i].z);
        out[i].radius = p.fx(base[i].radius);
        i += 1;
    }
    out
}

/// Every updraft, moved out with the floor and as much wider, so one that
/// hugged a face still does; as tall.
pub const fn vents<const N: usize>(base: [Vent; N], p: Plan) -> [Vent; N] {
    let mut out = base;
    let mut i = 0;
    while i < N {
        out[i].at = (p.fx(base[i].at.0), p.fx(base[i].at.1));
        out[i].radius = p.fx(base[i].radius);
        i += 1;
    }
    out
}

/// Every drift, stretched, and its run across the floor with it.
pub const fn drifts<const N: usize>(base: [Drift; N], p: Plan) -> [Drift; N] {
    let mut out = base;
    let mut i = 0;
    while i < N {
        out[i].solid = solid(base[i].solid, p);
        out[i].travel = p.v3(base[i].travel);
        i += 1;
    }
    out
}

/// The bounds, stretched.
pub const fn bounds(b: Bounds, p: Plan) -> Bounds {
    Bounds {
        lo_x: p.fx(b.lo_x),
        hi_x: p.fx(b.hi_x),
        lo_z: p.fx(b.lo_z),
        hi_z: p.fx(b.hi_z),
    }
}

/// A start, moved out with the floor, facing the way it faced.
pub const fn mark(m: Mark, p: Plan) -> Mark {
    Mark {
        at: p.v3(m.at),
        facing: m.facing,
    }
}

/// The rim, round the stretched rectangle, its feet and bank as high.
pub const fn rim(r: Option<Rim>, p: Plan) -> Option<Rim> {
    match r {
        None => None,
        Some(r) => Some(Rim {
            lo: (p.cm(r.lo.0), p.cm(r.lo.1)),
            hi: (p.cm(r.hi.0), p.cm(r.hi.1)),
            ..r
        }),
    }
}

/// The point, moved with the floor: as wide, and as much room over it.
pub const fn hill(h: Option<Hill>, p: Plan) -> Option<Hill> {
    match h {
        None => None,
        Some(h) => Some(Hill {
            at: p.v3(h.at),
            ..h
        }),
    }
}

/// **A King of the Hill map stretched**: its `Arena` and `Stage`, named,
/// from its base's tables. `$base` is the base map's module; the rest name
/// its `Arena`, `Stage` and tables in it.
macro_rules! stretched {
    (
        $arena:ident, $stage:ident, $id:expr, $name:literal, $plan:expr,
        $base:ident: $b_arena:ident, $b_stage:ident,
        $solids:ident, $regions:ident, $vents:ident, $drifts:ident, $bumps:ident
    ) => {
        pub static $arena: $crate::arena::Arena = $crate::arena::Arena {
            id: $id,
            name: $name,
            creature: None,
            bounds: $crate::arena::hills::sizes::bounds($base::$b_arena.bounds, $plan),
            floor: $base::$b_arena.floor,
            regions: &$crate::arena::hills::sizes::regions($base::$regions, $plan),
            solids: &$crate::arena::hills::sizes::solids($base::$solids, $plan),
            spawns: $crate::arena::Spawns {
                versus: [
                    $crate::arena::hills::sizes::mark($base::$b_arena.spawns.versus[0], $plan),
                    $crate::arena::hills::sizes::mark($base::$b_arena.spawns.versus[1], $plan),
                ],
                hunt: None,
            },
            sites: &[],
            rim: $crate::arena::hills::sizes::rim($base::$b_arena.rim, $plan),
        };

        pub static $stage: $crate::versus::Stage = $crate::versus::Stage {
            arena: $id,
            kind: $crate::versus::Kind::Hill,
            theme: $base::$b_stage.theme,
            pit: None,
            hill: $crate::arena::hills::sizes::hill($base::$b_stage.hill, $plan),
            front: None,
            vents: &$crate::arena::hills::sizes::vents($base::$vents, $plan),
            drifts: &$crate::arena::hills::sizes::drifts($base::$drifts, $plan),
            ways: $base::$b_stage.ways,
        };
    };
}

use super::{delta, glacier, gorge, henge, quarry};

stretched!(
    QUARRY_BIGGER, QUARRY_BIGGER_STAGE, ArenaId::QUARRY_BIGGER, "Bigger Quarry", BIGGER,
    quarry: QUARRY, QUARRY_STAGE, SOLIDS, REGIONS, VENTS, DRIFTS, BUMPS
);
stretched!(
    QUARRY_BIGGEST, QUARRY_BIGGEST_STAGE, ArenaId::QUARRY_BIGGEST, "Biggest Quarry", BIGGEST,
    quarry: QUARRY, QUARRY_STAGE, SOLIDS, REGIONS, VENTS, DRIFTS, BUMPS
);
stretched!(
    GLACIER_BIGGER, GLACIER_BIGGER_STAGE, ArenaId::GLACIER_BIGGER, "Bigger Glacier", BIGGER,
    glacier: GLACIER, GLACIER_STAGE, SOLIDS, REGIONS, VENTS, DRIFTS, BUMPS
);
stretched!(
    GLACIER_BIGGEST, GLACIER_BIGGEST_STAGE, ArenaId::GLACIER_BIGGEST, "Biggest Glacier", BIGGEST,
    glacier: GLACIER, GLACIER_STAGE, SOLIDS, REGIONS, VENTS, DRIFTS, BUMPS
);
stretched!(
    DELTA_BIGGER, DELTA_BIGGER_STAGE, ArenaId::DELTA_BIGGER, "Bigger Delta", BIGGER,
    delta: DELTA, DELTA_STAGE, SOLIDS, REGIONS, VENTS, DRIFTS, BUMPS
);
stretched!(
    DELTA_BIGGEST, DELTA_BIGGEST_STAGE, ArenaId::DELTA_BIGGEST, "Biggest Delta", BIGGEST,
    delta: DELTA, DELTA_STAGE, SOLIDS, REGIONS, VENTS, DRIFTS, BUMPS
);
stretched!(
    GORGE_BIGGER, GORGE_BIGGER_STAGE, ArenaId::GORGE_BIGGER, "Bigger Gorge", BIGGER,
    gorge: GORGE, GORGE_STAGE, SOLIDS, REGIONS, VENTS, DRIFTS, BUMPS
);
stretched!(
    GORGE_BIGGEST, GORGE_BIGGEST_STAGE, ArenaId::GORGE_BIGGEST, "Biggest Gorge", BIGGEST,
    gorge: GORGE, GORGE_STAGE, SOLIDS, REGIONS, VENTS, DRIFTS, BUMPS
);
stretched!(
    HENGE_BIGGER, HENGE_BIGGER_STAGE, ArenaId::HENGE_BIGGER, "Bigger Henge", BIGGER,
    henge: HENGE, HENGE_STAGE, SOLIDS, REGIONS, VENTS, DRIFTS, BUMPS
);
stretched!(
    HENGE_BIGGEST, HENGE_BIGGEST_STAGE, ArenaId::HENGE_BIGGEST, "Biggest Henge", BIGGEST,
    henge: HENGE, HENGE_STAGE, SOLIDS, REGIONS, VENTS, DRIFTS, BUMPS
);

/// The relief of a stretched map: its base's bumps, stretched. Empty for
/// anything else.
pub fn relief_of(id: ArenaId) -> Option<&'static [Bump]> {
    static QUARRY_BIGGER_BUMPS: &[Bump] = &bumps(quarry::BUMPS, BIGGER);
    static QUARRY_BIGGEST_BUMPS: &[Bump] = &bumps(quarry::BUMPS, BIGGEST);
    static GLACIER_BIGGER_BUMPS: &[Bump] = &bumps(glacier::BUMPS, BIGGER);
    static GLACIER_BIGGEST_BUMPS: &[Bump] = &bumps(glacier::BUMPS, BIGGEST);
    static DELTA_BIGGER_BUMPS: &[Bump] = &bumps(delta::BUMPS, BIGGER);
    static DELTA_BIGGEST_BUMPS: &[Bump] = &bumps(delta::BUMPS, BIGGEST);
    static GORGE_BIGGER_BUMPS: &[Bump] = &bumps(gorge::BUMPS, BIGGER);
    static GORGE_BIGGEST_BUMPS: &[Bump] = &bumps(gorge::BUMPS, BIGGEST);
    static HENGE_BIGGER_BUMPS: &[Bump] = &bumps(henge::BUMPS, BIGGER);
    static HENGE_BIGGEST_BUMPS: &[Bump] = &bumps(henge::BUMPS, BIGGEST);
    Some(match id {
        ArenaId::QUARRY_BIGGER => QUARRY_BIGGER_BUMPS,
        ArenaId::QUARRY_BIGGEST => QUARRY_BIGGEST_BUMPS,
        ArenaId::GLACIER_BIGGER => GLACIER_BIGGER_BUMPS,
        ArenaId::GLACIER_BIGGEST => GLACIER_BIGGEST_BUMPS,
        ArenaId::DELTA_BIGGER => DELTA_BIGGER_BUMPS,
        ArenaId::DELTA_BIGGEST => DELTA_BIGGEST_BUMPS,
        ArenaId::GORGE_BIGGER => GORGE_BIGGER_BUMPS,
        ArenaId::GORGE_BIGGEST => GORGE_BIGGEST_BUMPS,
        ArenaId::HENGE_BIGGER => HENGE_BIGGER_BUMPS,
        ArenaId::HENGE_BIGGEST => HENGE_BIGGEST_BUMPS,
        _ => return None,
    })
}
