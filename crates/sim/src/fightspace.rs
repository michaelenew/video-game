//! **Fight spaces, measured in the game's own units** (2026-11-02,
//! `docs/design/fight-spaces.md`).
//!
//! The owner, on King of the Hill: "everyone just stands in the middle and
//! spams abilities and it becomes chaotic ... the koth levels are just tiny
//! relative to each class's mobility options." A metre means nothing to a
//! fight; what a fight feels is **how many of its moves a distance is**. So a
//! place is measured here in two yardsticks taken off the classes themselves:
//!
//! - [`reach`]: how far a fight acts -- the longest pointed reach (a
//!   skillshot, a grounded cast, the Dual mage's overhead ray) of the class
//!   that reaches furthest but one, among the five that move. Somewhere
//!   further than that from a fighter is somewhere their pokes do not arrive
//!   but the longest one; a place with nowhere like it has no "away".
//! - [`jump`]: how high a fight climbs -- the full held jump of the class
//!   that jumps least, among the five that move. Two levels less than a
//!   jump and a half apart are one level that you hop between.
//!
//! [`measure`] reads an arena (and its stage, if it is one) against them:
//! how big it is in reaches, how much of where you can stand is out of reach
//! of its middle, how many levels it has a jump and a half apart, and what in
//! it moves you. `tests/fight_space.rs` holds every versus stage to the
//! rules, and `cargo run -p sim --bin fightspace` prints every fight's
//! numbers. Neither runs in a frame: this allocates, and is never called by
//! the simulation.

use crate::World;
use crate::aim::Kind as Aim;
use crate::arena::{Arena, ArenaId};
use crate::class::{Class, MOVERS};
use crate::fixed::Fx;
use crate::input::Input;
use crate::math::V3;
use crate::moves;
use crate::versus;

/// The longest pointed reach a class has: its furthest skillshot, grounded
/// cast or overhead ray.
pub fn pointed_reach(class: Class) -> Fx {
    moves::table(class)
        .iter()
        .filter(|m| matches!(m.aim(), Aim::Skillshot | Aim::Grounded | Aim::Behind))
        .map(|m| m.reach)
        .max_by_key(|r| r.raw())
        .unwrap_or(Fx::ZERO)
}

/// **How far a fight acts**: the second-longest of the movers' longest
/// pointed reaches. The longest is one class's outlier (a gust, a thrown
/// field); the second is a poke more than one class can put on you.
pub fn reach() -> Fx {
    let mut all: Vec<Fx> = MOVERS.iter().map(|&c| pointed_reach(c)).collect();
    all.sort_by_key(|r| core::cmp::Reverse(r.raw()));
    all[1]
}

/// A class's full held jump: the height its feet reach off flat ground with
/// the jump held to its top.
pub fn full_jump(class: Class) -> Fx {
    let mut w = World::with_classes([class, class]);
    let start = w.players[0].pos.y;
    let mut apex = start;
    for i in 0..300u32 {
        let bits = if i < 60 { Input::SPACE } else { 0 };
        w.advance([Input::new(bits), Input::default()]);
        let y = w.players[0].pos.y;
        if y.raw() > apex.raw() {
            apex = y;
        }
        if i > 0 && w.players[0].grounded {
            break;
        }
    }
    apex.sub(start)
}

/// **How high a fight climbs**: the shortest full held jump among the movers.
pub fn jump() -> Fx {
    MOVERS
        .iter()
        .map(|&c| full_jump(c))
        .min_by_key(|j| j.raw())
        .unwrap_or(Fx::ONE)
}

/// What a place measures, against [`reach`] and [`jump`].
#[derive(Clone, Debug)]
pub struct Measure {
    pub id: ArenaId,
    /// Its bounds along x and along z.
    pub length: Fx,
    pub width: Fx,
    /// Its middle: the capture point on a hill map, the middle of its bounds
    /// anywhere else; at the starts' height.
    pub centre: V3,
    /// From the first start to the middle, flat.
    pub start_to_centre: Fx,
    /// From the first start to the second, flat.
    pub starts_apart: Fx,
    /// How far everything you can stand on spans, along x and along z:
    /// a floating stage's pieces end to end.
    pub span: (Fx, Fx),
    /// Of every square metre you can stand on, the share (percent) further
    /// than a reach from the middle.
    pub beyond_reach: u32,
    /// The furthest you can stand from the middle.
    pub farthest: Fx,
    /// The heights of its levels over the starts, each at least a jump and a
    /// half over the one below: the first is the starts' own.
    pub tiers: Vec<Fx>,
    /// **Where its structure is**: the boxes a metre or more tall (or
    /// hanging), by how far their middles are from the middle -- within
    /// two fifths of a reach (the core), out to a reach (the contest),
    /// past it (out of reach).
    pub rings: [usize; 3],
    /// Updrafts and boxes that move.
    pub vents: usize,
    pub drifts: usize,
    /// Boxes that hang: a bridge, an island, a lintel, a roof.
    pub hanging: usize,
    /// The ways it promises, and how many of them end on the point.
    pub ways: usize,
    pub ways_to_centre: usize,
}

impl Measure {
    /// The highest level over the starts.
    pub fn top(&self) -> Fx {
        self.tiers.last().copied().unwrap_or(Fx::ZERO)
    }
}

/// The smallest top worth counting as a level: two metres square, room to
/// stand and turn.
fn standable(s: &crate::arena::Solid) -> bool {
    let two = Fx::from_int(2);
    s.max.x.sub(s.min.x).raw() >= two.raw() && s.max.z.sub(s.min.z).raw() >= two.raw()
}

/// **Measure a place** against [`reach`] and [`jump`] (pass them in: each is
/// worked out by running the classes).
pub fn measure(arena: &'static Arena, reach: Fx, jump: Fx) -> Measure {
    let stage = versus::of(arena.id);
    let b = arena.bounds;
    let base = arena.spawns.versus[0].at.y;
    let centre = match stage.and_then(|s| s.hill) {
        Some(h) => V3::new(h.at.x, base, h.at.z),
        None => V3::new(
            crate::math::midpoint(b.lo_x, b.hi_x),
            base,
            crate::math::midpoint(b.lo_z, b.hi_z),
        ),
    };
    let start = arena.spawns.versus[0].at;
    let floored = stage.is_none_or(|s| s.pit.is_none());
    let solids = arena.solids();

    // Every square metre: the highest top over it (or the floor), and
    // whether anybody can stand there at all.
    let (mut area, mut far, mut farthest) = (0u32, 0u32, Fx::ZERO);
    let (mut lo, mut hi) = ((b.hi_x, b.hi_z), (b.lo_x, b.lo_z));
    let mut heights: Vec<(i32, u32)> = Vec::new();
    let mut x = b.lo_x.add(Fx::ratio(1, 2));
    while x.raw() < b.hi_x.raw() {
        let mut z = b.lo_z.add(Fx::ratio(1, 2));
        while z.raw() < b.hi_z.raw() {
            let top = solids
                .iter()
                .filter(|s| standable(s) && s.over(x, z, Fx::ZERO))
                .map(|s| s.max.y)
                .max_by_key(|y| y.raw());
            let at = match (top, floored) {
                (Some(y), _) => Some(y),
                (None, true) => Some(arena.relief_at(x, z)),
                (None, false) => None,
            };
            if let Some(y) = at {
                area += 1;
                lo = (lo.0.min(x), lo.1.min(z));
                hi = (hi.0.max(x), hi.1.max(z));
                let d = crate::math::wide_len(V3::new(x, y, z).sub(centre));
                if d.raw() > reach.raw() {
                    far += 1;
                }
                if d.raw() > farthest.raw() {
                    farthest = d;
                }
                let h = y.sub(base).raw();
                match heights.iter_mut().find(|(k, _)| *k == h) {
                    Some((_, n)) => *n += 1,
                    None => heights.push((h, 1)),
                }
            }
            z = z.add(Fx::ONE);
        }
        x = x.add(Fx::ONE);
    }

    // Levels: heights with room on them (four square metres or more),
    // climbing from the starts', each a jump and a half over the last.
    heights.sort();
    let step = jump.add(jump.div(Fx::from_int(2)));
    let mut tiers = vec![Fx::ZERO];
    for (h, n) in heights {
        let h = Fx::from_raw(h);
        if n >= 4 && h.raw() >= tiers.last().unwrap().add(step).raw() {
            tiers.push(h);
        }
    }

    let hill = stage.and_then(|s| s.hill);
    let ways = stage.map_or(&[][..], |s| s.ways);
    let to_centre = ways
        .iter()
        .filter(|w| {
            let (Some(&(x, y, z)), Some(h)) = (w.route.last(), hill) else {
                return false;
            };
            let (x, y, z) = crate::arena::hills::sizes::way_point(arena.id, (x, y, z));
            h.holds(V3::new(
                Fx::ratio(x, 100),
                Fx::ratio(y, 100),
                Fx::ratio(z, 100),
            ))
        })
        .count();
    let ground = crate::arena::Terrain::bare(arena);
    let mut rings = [0usize; 3];
    let core = reach.mul(Fx::ratio(2, 5));
    for s in solids {
        if s.max.y.sub(s.min.y).raw() < Fx::ONE.raw() && !ground.hangs(s) {
            continue;
        }
        let mid = V3::new(
            crate::math::midpoint(s.min.x, s.max.x),
            Fx::ZERO,
            crate::math::midpoint(s.min.z, s.max.z),
        );
        let d = crate::math::wide_len(mid.sub(V3::new(centre.x, Fx::ZERO, centre.z)));
        let k = if d.raw() <= core.raw() {
            0
        } else if d.raw() <= reach.raw() {
            1
        } else {
            2
        };
        rings[k] += 1;
    }
    Measure {
        id: arena.id,
        length: b.hi_x.sub(b.lo_x),
        width: b.hi_z.sub(b.lo_z),
        centre,
        start_to_centre: crate::math::wide_len(V3::new(
            start.x.sub(centre.x),
            Fx::ZERO,
            start.z.sub(centre.z),
        )),
        starts_apart: {
            let other = arena.spawns.versus[1].at;
            crate::math::wide_len(V3::new(
                other.x.sub(start.x),
                Fx::ZERO,
                other.z.sub(start.z),
            ))
        },
        span: (hi.0.sub(lo.0).max(Fx::ZERO), hi.1.sub(lo.1).max(Fx::ZERO)),
        beyond_reach: (far * 100).checked_div(area).unwrap_or(0),
        farthest,
        tiers,
        rings,
        vents: crate::valley::vents(arena.id).len(),
        drifts: versus::drifts(arena.id).len(),
        hanging: solids.iter().filter(|s| ground.hangs(s)).count(),
        ways: ways.len(),
        ways_to_centre: to_centre,
    }
}
