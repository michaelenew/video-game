//! **Creatures that leave their range** (exploration 0023, 2026-10-19).
//!
//! The owner's brief: a valley creature can go anywhere on the map, without
//! limit, but it prefers its own ground. What makes it leave is somebody it
//! is after: a player carrying its egg is pursued across the valley until
//! the creature is beaten or the egg is dropped or taken back, and a roused
//! creature follows whoever walks off -- for a while.
//!
//! - **A hunt remembers its den** (`Journey::den`): the room or range it began
//!   in. The world still runs in the coordinates of the place the fighters
//!   are in and moves when they cross, hunt and all (`World::shift` moves the
//!   creatures and their lore); everything a creature reads of its own arena
//!   it reads through [`Den`], its arena and how far its origin is from the
//!   world's.
//! - **It is kept in its bounds until it has a reason not to be**: while it is
//!   after somebody who is out of them, or is out itself, it is kept only
//!   inside the map (`Den::kept`). At home, and in every fight that is not
//!   the valley, nothing here changes anything.
//! - **Out, it travels**: further from whoever it is after than
//!   `tuning::roam_engage`, it heads for them at `roam_pace` by
//!   [`way_toward`] -- through a room's doorway when one of them is in a room
//!   and the other is not, and by the first heading whose next few metres are
//!   walkable: not land too steep to climb (`Terrain::downhill`), not a step
//!   up or a drop past `roam_step` and `roam_drop`, not a solid. Closer, it
//!   fights as it does at home.
//! - **It gives up** after `roam_give_up_time` far from home
//!   (`roam_far` past its edge) or at once `roam_give_up_distance` past it --
//!   both scaled by temperament (`tuning::roam_keeps_at_it`) -- and walks
//!   home at `roam_home_pace`, starting nothing. **Under an egg chase it never
//!   does.** Home again, it settles; walking home with nobody within
//!   `roam_reach`, or home with nobody near, its hunt calms.

use crate::arena::{Den, Terrain};
use crate::atlas;
use crate::fixed::Fx;
use crate::math::{self, V3};
use crate::state::World;
use crate::tuning as t;

use super::Kind;
use super::ranges::{self, Temperament};

impl World {
    /// **The live hunt's home**: the room or range it began in, or the
    /// place the world is in.
    pub fn den_id(&self) -> crate::arena::ArenaId {
        match self.valley.den {
            Some(d) if self.valley.on && self.hunting() => d,
            _ => self.arena,
        }
    }

    /// **The hunt's home, seen from where the world is** ([`Den`]): its arena,
    /// its offset, its bounds in the world's coordinates and where its
    /// creatures are kept this frame. The arena itself, kept in its bounds,
    /// everywhere but a valley hunt.
    pub fn den(&self) -> Den {
        let here = self.arena();
        if !self.valley.on || !self.hunting() {
            return Den::of(here);
        }
        let id = self.den_id();
        let atlas = atlas::valley();
        let (Some(home), Some(now)) = (atlas.placed(id), atlas.placed(self.arena)) else {
            return Den::of(here);
        };
        let arena = id.get();
        let off = home.at.sub(now.at);
        let bounds = arena.bounds.moved(off);
        let homing = self.valley.homing;
        let roam = !homing && self.pursuing();
        let outside = |p: V3| !bounds.holds(p.x, p.z);
        let seats = (self.seats as usize).clamp(1, crate::state::MAX_PLAYERS);
        let out = self
            .monsters
            .iter()
            .flatten()
            .filter(|m| m.alive())
            .any(|m| outside(m.pos) || roam && outside(m.brain.seen))
            || self
                .critters
                .iter()
                .filter(|c| c.alive())
                .any(|c| outside(c.pos))
            || roam
                && self.pack.is_some()
                && (0..seats).any(|i| self.players[i].health > 0 && outside(self.players[i].pos));
        let kept = if out {
            crate::arena::Bounds {
                lo_x: atlas.lo.0.sub(now.at.x),
                hi_x: atlas.hi.0.sub(now.at.x),
                lo_z: atlas.lo.1.sub(now.at.z),
                hi_z: atlas.hi.1.sub(now.at.z),
            }
        } else {
            bounds
        };
        Den {
            arena,
            off,
            home: bounds,
            kept,
            roam,
            homing,
        }
    }

    /// The live hunt's temperament: its range's, or a room's, which is a
    /// hunting one.
    fn den_temperament(&self) -> Temperament {
        ranges::of(self.den_id()).map_or(Temperament::Hunting, |r| r.temperament)
    }

    /// **Is the hunt after somebody** -- and so free to leave home after
    /// them? Always, while somebody carries its egg. Otherwise a creature
    /// that hunts, or has been roused; never a skittish one, which runs.
    pub fn pursuing(&self) -> bool {
        if !self.valley.on || !self.hunting() {
            return false;
        }
        if self.egg_carrier().is_some() {
            return true;
        }
        match self.den_temperament() {
            Temperament::Hunting => true,
            Temperament::Skittish => false,
            Temperament::Territorial { .. } | Temperament::Peaceful => self.valley.roused,
        }
    }

    /// **How far past its home's edge the furthest of the hunt's bodies is**,
    /// flat: zero while every one is inside.
    pub fn past_home(&self, den: &Den) -> Fx {
        let past = |p: V3| {
            let b = &den.home;
            let dx = b.lo_x.sub(p.x).max(p.x.sub(b.hi_x)).max(Fx::ZERO);
            let dz = b.lo_z.sub(p.z).max(p.z.sub(b.hi_z)).max(Fx::ZERO);
            dx.max(dz)
        };
        let bodies = self
            .monsters
            .iter()
            .flatten()
            .filter(|m| m.alive())
            .map(|m| past(m.pos));
        let small = self
            .critters
            .iter()
            .filter(|c| c.alive())
            .map(|c| past(c.pos));
        bodies.chain(small).max().unwrap_or(Fx::ZERO)
    }

    /// **Once a frame, in a valley hunt**: count the time far from home,
    /// give up, walk home, settle.
    pub(crate) fn step_roam(&mut self) {
        if !self.valley.on || !self.hunting() {
            return;
        }
        // Its egg is being carried off: it never gives up.
        if self.egg_carrier().is_some() {
            self.valley.away = 0;
            self.valley.homing = false;
            return;
        }
        let den = self.den();
        let past = self.past_home(&den);
        if self.valley.homing {
            // Home, with somebody near it: it settles, and its temperament has
            // it again. Home with nobody near, its hunt is over
            // (`valley::open`, next frame).
            if past == Fx::ZERO && self.near_home(&den) {
                self.valley.homing = false;
                self.valley.away = 0;
                self.valley.roused = false;
                if let Some(p) = self.pack.as_mut() {
                    p.mood = crate::pack::mood::CALM;
                }
            }
            return;
        }
        if !self.pursuing() {
            self.valley.away = 0;
            // Out, and after nobody (a skittish herd's run, a calm one
            // pushed out): it goes home.
            if past.raw() > t::roam_far().raw() {
                self.go_home(&den);
            }
            return;
        }
        if past.raw() > t::roam_far().raw() {
            self.valley.away = self.valley.away.saturating_add(1);
        } else {
            self.valley.away = 0;
        }
        let keep = t::roam_keeps_at_it(self.den_temperament()).max(0);
        let frames = (t::roam_give_up_frames() as u64 * keep as u64 / 100) as u32;
        let far = t::roam_give_up_distance().mul(Fx::ratio(keep, 100));
        if self.valley.away as u32 > frames || past.raw() > far.raw() {
            self.go_home(&den);
        }
    }

    /// Is anybody on their feet in the hunt's home, or within
    /// `hunt_held_past` of it?
    fn near_home(&self, den: &Den) -> bool {
        let m = t::hunt_held_past();
        let b = &den.home;
        let seats = (self.seats as usize).clamp(1, crate::state::MAX_PLAYERS);
        (0..seats).any(|i| {
            let p = self.players[i].pos;
            self.players[i].health > 0
                && p.x.raw() >= b.lo_x.sub(m).raw()
                && p.x.raw() <= b.hi_x.add(m).raw()
                && p.z.raw() >= b.lo_z.sub(m).raw()
                && p.z.raw() <= b.hi_z.add(m).raw()
        })
    }

    /// **It gives up**: it walks home, and starts nothing on the way.
    fn go_home(&mut self, den: &Den) {
        self.valley.homing = true;
        self.valley.away = 0;
        if let Some(p) = self.pack.as_mut() {
            p.mood = crate::pack::mood::BROKEN;
            p.home = den.middle();
        }
    }

    /// **Walking home, and nobody near it**: out of everyone's reach, its hunt
    /// is over. And home, with nobody near home, it is too.
    pub(crate) fn gone_home(&self) -> bool {
        if !self.valley.homing {
            return false;
        }
        let den = self.den();
        if self.past_home(&den) == Fx::ZERO {
            return true;
        }
        let reach = t::roam_reach();
        let seats = (self.seats as usize).clamp(1, crate::state::MAX_PLAYERS);
        let near = |p: V3| {
            (0..seats).any(|i| {
                self.players[i].health > 0
                    && math::wide_flat_dist(self.players[i].pos, p).raw() <= reach.raw()
            })
        };
        let bodies = self
            .monsters
            .iter()
            .flatten()
            .filter(|m| m.alive())
            .map(|m| m.pos);
        let small = self.critters.iter().filter(|c| c.alive()).map(|c| c.pos);
        !bodies.chain(small).any(near)
    }
}

/// **A room with walls** -- one of the creatures' own arenas, not a range,
/// which is open land -- that the map has.
fn walled(id: crate::arena::ArenaId) -> bool {
    ranges::of(id).is_none() && matches!(super::place(id).map(|p| p.kind), Some(Kind::Room(_)))
}

/// The doorway of a walled room on the map: its middle, on the map.
fn door_of(atlas: &atlas::Atlas, id: crate::arena::ArenaId) -> Option<V3> {
    let d = atlas.doors.iter().find(|d| d.a.0 == id || d.b.0 == id)?;
    Some(V3::new(
        math::midpoint(d.cut.min.x, d.cut.max.x),
        d.cut.min.y,
        math::midpoint(d.cut.min.z, d.cut.max.z),
    ))
}

/// **Where to head next, on the way from `from` to `goal`**, both in the
/// ground's coordinates: through a doorway first if one of them is in a
/// walled room and the other is not, and along the first heading -- straight
/// at it, then turning either way, the way it already faces first -- whose
/// next `roam_probe` metres are walkable for a body of `radius`. The goal
/// itself if nothing is (and the walk refuses what is not walkable).
pub fn way_toward(ground: &Terrain, from: V3, goal: V3, yaw: Fx, radius: Fx) -> V3 {
    detour(ground, from, goal, yaw, radius).unwrap_or(goal)
}

/// [`way_toward`], or `None` when the way is straight at `goal`: nothing
/// walked round and no doorway in it. What a creature close to whoever it is
/// after asks, so its own brain keeps the say unless something is in the way.
pub fn detour(ground: &Terrain, from: V3, goal: V3, yaw: Fx, radius: Fx) -> Option<V3> {
    let (goal, door) = match ground.atlas() {
        Some(atlas) => {
            let via = through_doors(atlas, ground.origin(), from, goal);
            (via, via != goal)
        }
        None => (goal, false),
    };
    let to = V3::new(goal.x.sub(from.x), Fx::ZERO, goal.z.sub(from.z));
    let dist = math::wide_flat_len(to);
    let probe = t::roam_probe();
    if dist.raw() <= radius.raw() {
        return door.then_some(goal);
    }
    let base = math::atan2_turns(to.z, to.x);
    let floor = ground.relief_at(from.x, from.z);
    let start = V3::new(from.x, floor, from.z);
    let reach = probe.min(dist.max(radius));
    for n in 0..=6 {
        let turn = Fx::ratio(n, 12);
        let both = [base.add(turn), base.sub(turn)];
        // The side it already faces first, so it does not dither.
        let order = if math::wrap_turns(both[0].sub(yaw)).abs().raw()
            <= math::wrap_turns(both[1].sub(yaw)).abs().raw()
        {
            [both[0], both[1]]
        } else {
            [both[1], both[0]]
        };
        for (k, heading) in order.into_iter().enumerate() {
            if n == 0 && k == 1 || n == 6 && k == 1 {
                continue;
            }
            let dir = V3::from_turns(heading);
            if open_along(ground, start, dir, reach, radius) {
                return if n == 0 {
                    door.then_some(goal)
                } else {
                    Some(from.add(dir.scale(probe)))
                };
            }
        }
    }
    door.then_some(goal)
}

/// Is the way `length` along `dir` from `start` (on the floor) walkable, in
/// three steps?
fn open_along(ground: &Terrain, start: V3, dir: V3, length: Fx, radius: Fx) -> bool {
    let mut at = start;
    for k in 1..=3 {
        let p = start.add(dir.scale(length.mul(Fx::ratio(k, 3))));
        match step_onto(ground, at, p, radius) {
            Some(y) => at = V3::new(p.x, y, p.z),
            None => return false,
        }
    }
    true
}

/// **Can a body on the floor at `from` walk to `to`, the ground alone** --
/// the floor's height there, or `None`: off the map, a step up past
/// `roam_step` or down past `roam_drop`, or a climb of land steeper than
/// `terrain_steepest`, rise over run. What
/// a roaming creature's walk refuses (solids it slides along, as a fighter
/// does: `Terrain::fence`). Only a climb is refused on a steep slope, so a
/// body a charge left on one can always walk off it.
pub fn ground_step(ground: &Terrain, from: V3, to: V3) -> Option<Fx> {
    let y = ground.relief_at(to.x, to.z);
    if !ground.inside(V3::new(to.x, y, to.z)) {
        return None;
    }
    let rise = y.sub(from.y);
    if rise.raw() > t::roam_step().raw() || rise.neg().raw() > t::roam_drop().raw() {
        return None;
    }
    // Up land steeper than a body can climb: the step's own slope, rise
    // over run, so walking along a steep bank or out of a dip is never a
    // climb it is refused.
    if rise.raw() > 0 && ground.on_land(to.x, to.z) && steeper(rise, to.sub(from)) {
        return None;
    }
    Some(y)
}

/// **Is a rise over a flat run steeper than `terrain_steepest`?** In whole
/// integers squared: a frame's step is millimetres, which a fixed-point length
/// squares to nothing.
fn steeper(rise: Fx, run: V3) -> bool {
    let (r, x, z) = (rise.raw() as i128, run.x.raw() as i128, run.z.raw() as i128);
    let k = t::terrain_steepest().raw() as i128;
    r * r * (1 << 32) > (x * x + z * z) * k * k
}

/// **Is `to` somewhere a body of `radius` could walk to from `from`**: the
/// ground allows it ([`ground_step`]) and no solid it could not step over
/// stands within `radius` of it. What the steering looks ahead with.
pub fn step_onto(ground: &Terrain, from: V3, to: V3, radius: Fx) -> Option<Fx> {
    let y = ground_step(ground, from, to)?;
    let at = V3::new(to.x, y, to.z);
    let step = t::roam_step();
    for s in ground.around(at, radius) {
        if s.max.y.raw() <= y.add(step).raw() || ground.hangs(&s) {
            continue;
        }
        if s.side_push(to.x, to.z, radius, y, s.max.y).is_some() {
            return None;
        }
    }
    Some(y)
}

/// The doorway in the way, if there is one: from inside a walled room to
/// anywhere else is out through its door, and from anywhere else into one is
/// in through its door. In the ground's coordinates.
fn through_doors(atlas: &'static atlas::Atlas, origin: V3, from: V3, goal: V3) -> V3 {
    let (fm, gm) = (from.add(origin), goal.add(origin));
    let here = atlas.place_for(fm).map(|p| p.arena);
    let there = atlas.place_for(gm).map(|p| p.arena);
    if here == there {
        return goal;
    }
    let near = t::roam_door();
    // Out of the room it is in.
    if let Some(room) = here.filter(|&id| walled(id))
        && let Some(door) = door_of(atlas, room)
        && let Some(p) = atlas.placed(room)
    {
        let mid = V3::new(
            math::half(p.lo.0.add(p.hi.0)),
            Fx::ZERO,
            math::half(p.lo.1.add(p.hi.1)),
        );
        let out = math::wide_normalized(V3::new(door.x.sub(mid.x), Fx::ZERO, door.z.sub(mid.z)));
        let at_door = math::wide_flat_dist(door, fm).raw() <= near.raw();
        let beyond = door.add(out.scale(near.add(near)));
        return if at_door { beyond } else { door }.sub(origin);
    }
    // Into the room its quarry is in.
    if let Some(room) = there.filter(|&id| walled(id))
        && let Some(door) = door_of(atlas, room)
        && math::wide_flat_dist(door, fm).raw() > near.raw()
    {
        return door.sub(origin);
    }
    goal
}
