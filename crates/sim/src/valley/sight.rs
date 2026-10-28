//! **Sightings**: every creature of the valley where it is, whether or not
//! anybody is near it (2026-10-22). The owner: "I want to be able to spot
//! monsters from far off and navigate there. Since they're seeded and
//! deterministic while players are far off, we shouldn't need to load them in
//! to display them in the right place."
//!
//! The snapshot keeps one live hunt, and every other creature of the valley
//! is asleep: nothing about it is in the world. But **where a sleeping
//! creature is is a function of the frame number and nothing else**, the way
//! a drift's is (`crate::drift`): a range's herd walks from one point of its
//! pocket to the next on a fixed clock of legs -- turn, walk, graze -- the
//! points drawn from a hash of the range and the leg. So the renderer draws
//! every one of them, from across the valley, by asking this module, and the
//! simulation never carries them.
//!
//! **Walking up to one wakes it where it was seen.** A hunt that starts on a
//! range puts its creatures exactly where [`stand`] has them on that frame --
//! the same function, the same frame -- so the animal you were watching from
//! the road is the one that turns to face you, not a fresh one on its marks.
//! A creature's room (`valley::rooms`) is tuned from its marks, so its
//! creature waits on them, breathing, until somebody walks in; so does a
//! range stacked inside a place -- a warren of the Deep, its Caverns, the
//! Needle's dome -- where a point drawn at random could be in a wall.

use crate::arena::{ArenaId, Terrain};
use crate::critter::Critters;
use crate::fixed::Fx;
use crate::math::V3;
use crate::monster::{MAX_MONSTERS, Monster};
use crate::pack::Pack;
use crate::species::SpeciesId;

use super::Kind;
use super::ranges::{self, Range};

/// **One leg of a herd's wander**, in frames: it turns toward its next
/// point, walks there at its own walk, and grazes out the rest.
pub const LEG: u32 = 1200;

/// How long the turn at the start of a leg takes, in frames.
const TURN: u32 = 90;

/// How far from its range's middle a herd's points are drawn, as a share of
/// the range's half-side, in thousandths: inside it with its formation, so
/// the bodies at the edge of the formation stay in their bounds.
const REACH: i64 = 330;

/// A pack's walk, centimetres a second: its critters have no walk of their
/// own to read when nobody is chasing.
const PACK_WALK: i64 = 130;

/// **What a place's creatures are**, in the place's own coordinates: its
/// bodies, its critters and its pack -- exactly what a hunt starting there
/// puts in the world, before it has moved.
#[derive(Clone, Copy, Debug)]
pub struct Sight {
    pub arena: ArenaId,
    pub monsters: [Option<Monster>; MAX_MONSTERS],
    pub critters: Critters,
    pub pack: Option<Pack>,
}

/// The creatures a hunt of `species` starts with in `arena`: its herd as a
/// range counts it, and whatever pack it brings.
pub fn herd(arena: ArenaId, species: SpeciesId) -> [Option<SpeciesId>; MAX_MONSTERS] {
    let mut herd = crate::replay::herd_of(species);
    if let Some(r) = ranges::of(arena).filter(|_| species.get().has_body()) {
        for (k, slot) in herd.iter_mut().enumerate() {
            *slot = (k < r.bodies as usize).then_some(species);
        }
    }
    herd
}

/// **A place's creatures as a hunt there starts**, before they have moved:
/// `None` for a place with none. Built as a hunt is (`World::hunt_in`), so
/// it is the hunt's own first frame; worth keeping, since it builds a world.
pub fn base(arena: ArenaId) -> Option<Sight> {
    let Some(Kind::Room(species)) = super::place(arena).map(|p| p.kind) else {
        return None;
    };
    let fresh =
        crate::state::World::hunt_in([crate::class::Class::Champion], herd(arena, species), arena);
    let mut sight = Sight {
        arena,
        monsters: fresh.monsters,
        critters: fresh.critters,
        pack: fresh.pack,
    };
    if let Some(r) = ranges::of(arena) {
        for m in sight.monsters.iter_mut().flatten() {
            m.temper = r.temper;
        }
        if let Some(p) = sight.pack.as_mut() {
            p.temper = r.temper;
        }
    }
    Some(sight)
}

/// A deterministic number in [0, 1000) from a range, a leg and a salt.
fn dice(k: u32, leg: u32, salt: u32) -> i64 {
    let mut h = k.wrapping_mul(0x9E37_79B1)
        ^ leg.wrapping_mul(0x85EB_CA77)
        ^ salt.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h % 1000) as i64
}

/// A herd's point for leg `leg` of range `r`, centimetres from its middle.
fn point(r: &Range, leg: u32) -> (i64, i64) {
    let k = r.arena.0 as u32;
    let far = r.half as i64 * REACH / 1000;
    let x = (dice(k, leg, 1) - 500) * 2 * far / 1000;
    let z = (dice(k, leg, 2) - 500) * 2 * far / 1000;
    (x, z)
}

/// The way from one point to the next, in turns, and how long it is (cm).
fn heading(a: (i64, i64), b: (i64, i64)) -> (Fx, i64) {
    let (dx, dz) = (b.0 - a.0, b.1 - a.1);
    let len = crate::math::isqrt_wide((dx * dx + dz * dz) as u128) as i64;
    if len == 0 {
        return (Fx::ZERO, 0);
    }
    let yaw = crate::math::atan2_turns(
        Fx::from_raw((dz * 65536 / 100).clamp(i32::MIN as i64, i32::MAX as i64) as i32),
        Fx::from_raw((dx * 65536 / 100).clamp(i32::MIN as i64, i32::MAX as i64) as i32),
    );
    (yaw, len)
}

/// The shorter way round from `a` to `b`, in turns, `t` of the way (0 to 1).
fn turn_toward(a: Fx, b: Fx, t: Fx) -> Fx {
    let mut d = b.sub(a);
    let half = Fx::ratio(1, 2);
    while d.raw() > half.raw() {
        d = d.sub(Fx::ONE);
    }
    while d.raw() < half.neg().raw() {
        d = d.add(Fx::ONE);
    }
    a.add(d.mul(t))
}

/// **Where a range's herd is on frame `frame`**: how far it has come from
/// its middle (cm), which way it faces (turns), whether it is walking, and
/// how far it has walked this leg (cm).
pub fn wander(r: &Range, frame: u32, walk_cm_s: i64) -> ((i64, i64), Fx, bool, i64) {
    let k = r.arena.0 as u32;
    // Each range keeps its own time, so the valley's herds do not all set
    // off together.
    let t = frame.wrapping_add(dice(k, 0, 9) as u32 * LEG / 1000);
    let leg = t / LEG;
    let into = t % LEG;
    let (from, to) = (point(r, leg), point(r, leg + 1));
    let (was, _) = heading(point(r, leg.wrapping_sub(1)), from);
    let (yaw, len) = heading(from, to);
    let walk = walk_cm_s.max(1);
    // Frames the walk takes; a herd faster than its leg cuts it short.
    let frames = (len * 60 / walk).min((LEG - TURN) as i64 - 1);
    if into < TURN {
        let t = Fx::from_raw((into as i64 * 65536 / TURN as i64) as i32);
        return (from, turn_toward(was, yaw, t), false, 0);
    }
    let walked = (into - TURN) as i64;
    if walked >= frames || frames <= 0 {
        return (to, yaw, false, len);
    }
    let along = len * walked / frames;
    let at = (
        from.0 + (to.0 - from.0) * walked / frames,
        from.1 + (to.1 - from.1) * walked / frames,
    );
    (at, yaw, true, along)
}

/// How long a roaming herd grazes at each point of its loop, in frames.
pub const GRAZE: u32 = 900;

/// **Where a roaming range's herd is on frame `frame`** ([`Range::walk`]):
/// round its loop of points, turning toward the next at each, walking there
/// at its own walk and grazing out [`GRAZE`] -- the loop's whole length one
/// cycle, so the frame alone says where on it the herd is. As [`wander`]:
/// how far from its middle (cm), its facing, whether it walks, how far it
/// has walked this leg.
pub fn roam(r: &Range, frame: u32, walk_cm_s: i64) -> ((i64, i64), Fx, bool, i64) {
    let pts = r.walk;
    let n = pts.len();
    let at = |i: usize| {
        let (x, z) = pts[i % n];
        ((x - r.at.0) as i64, (z - r.at.1) as i64)
    };
    let walk = walk_cm_s.max(1);
    let leg = |i: usize| {
        let (_, len) = heading(at(i), at(i + 1));
        ((len * 60 / walk) as u32, len)
    };
    let cycle: u32 = (0..n).map(|i| TURN + leg(i).0 + GRAZE).sum::<u32>().max(1);
    let k = r.arena.0 as u32;
    let mut t = frame.wrapping_add(dice(k, 0, 9) as u32 * 997) % cycle;
    for i in 0..n {
        let (frames, len) = leg(i);
        let (from, to) = (at(i), at(i + 1));
        let (was, _) = heading(at(i + n - 1), from);
        let (yaw, _) = heading(from, to);
        if t < TURN {
            let f = Fx::from_raw((t as i64 * 65536 / TURN as i64) as i32);
            return (from, turn_toward(was, yaw, f), false, 0);
        }
        t -= TURN;
        if t < frames {
            let w = t as i64;
            let f = frames.max(1) as i64;
            let p = (
                from.0 + (to.0 - from.0) * w / f,
                from.1 + (to.1 - from.1) * w / f,
            );
            return (p, yaw, true, len * w / f);
        }
        t -= frames;
        if t < GRAZE {
            return (to, yaw, false, len);
        }
        t -= GRAZE;
    }
    (at(0), Fx::ZERO, false, 0)
}

/// **Stand a place's creatures where they are on `frame`**: a range's herd
/// moved along its wander, facing its way, its legs where its walk has them;
/// a room's creatures on their marks, breathing. `offset` is the place's
/// origin seen from wherever the answer is wanted (zero: the place's own
/// coordinates), and `ground` the floor there, which each body is stood on.
pub fn stand(sight: &mut Sight, frame: u32, offset: V3, ground: &Terrain) {
    // Only a range out on the open land wanders: one stacked in a place --
    // a warren's maze, the Caverns, the Needle's dome -- waits on its marks,
    // where its fight was laid out clear of the walls.
    let range = ranges::of(sight.arena).filter(|r| r.stacked.is_none());
    let mover = sight.monsters.iter().flatten().next().map(|m| m.sp());
    let walk = match mover {
        Some(sp) => sp.walk().raw() as i64 * 100 / 65536,
        None => PACK_WALK,
    };
    let (moved, yaw, walking, along) = match range {
        Some(r) if r.roams() => roam(r, frame, walk),
        Some(r) => wander(r, frame, walk),
        None => ((0, 0), Fx::ZERO, false, 0),
    };
    let d = V3::new(
        crate::arena::cm(moved.0 as i32),
        Fx::ZERO,
        crate::arena::cm(moved.1 as i32),
    )
    .add(offset);
    let floor = |p: V3| {
        let up = Fx::from_int(6);
        ground.floor_below(V3::new(p.x, p.y.add(up), p.z))
    };
    for m in sight.monsters.iter_mut().flatten() {
        m.pos = m.pos.add(d);
        m.pos.y = floor(m.pos);
        if range.is_some() {
            m.yaw = yaw;
            m.speed = if walking { m.sp().walk() } else { Fx::ZERO };
            let stride = m.sp().gait_stride().max(Fx::from_raw(1));
            let covered = crate::arena::cm(along as i32).div(stride);
            m.stride = covered.raw() as u16;
        }
        m.beat = (frame as u16).wrapping_mul(m.sp().breath_rate());
    }
    if let Some(p) = sight.pack.as_mut() {
        p.home = p.home.add(d);
    }
    let facing = V3::from_turns(yaw);
    let speed = crate::arena::cm(PACK_WALK as i32);
    for c in sight.critters.iter_mut().filter(|c| c.present()) {
        c.pos = c.pos.add(d);
        c.pos.y = floor(c.pos);
        c.clock = frame as u16;
        if range.is_some() {
            c.yaw = yaw.raw() as u16;
            c.vel = if walking {
                facing.scale(speed)
            } else {
                V3::ZERO
            };
        }
    }
}

/// **The middle of a roaming range's herd on `frame`**, on the map: its
/// loop's point, moved as [`stand`] moves its bodies, from the middle of its
/// marks. What waking it by approach measures from (`valley::open`).
pub fn herd_middle(r: &Range, frame: u32) -> V3 {
    let sp = r.species.get();
    let walk = if sp.has_body() {
        sp.walk().raw() as i64 * 100 / 65536
    } else {
        PACK_WALK
    };
    let (moved, _, _, _) = if r.roams() {
        roam(r, frame, walk)
    } else {
        wander(r, frame, walk)
    };
    let cm = |v: i64| crate::arena::cm(v as i32);
    V3::new(
        cm(r.at.0 as i64 + moved.0),
        cm(r.floor as i64),
        cm(r.at.1 as i64 + moved.1 + r.half as i64 / 3),
    )
}
