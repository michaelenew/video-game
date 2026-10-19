//! **The valley as one world**: the rules that make a map of places play as
//! one piece (`docs/design/atlas.md`).
//!
//! - **The world runs in the coordinates of a place**, `World::arena`: the
//!   place the fighters are in. Every position in the snapshot is in that
//!   place's own coordinates, so a creature's fight is exactly the fight it
//!   was tuned in. The rest of the map is around it, solid, through
//!   `arena::Terrain`.
//! - **The frame follows the fighters.** When the first fighter on their feet
//!   walks into another place, the world moves to that place's coordinates:
//!   every position in it shifts by the difference between the two places'
//!   origins, a whole number of centimetres, on the same frame on both
//!   machines. Nothing is created or lost by it, and the renderer adds the
//!   place's origin back to draw everything where it was.
//! - **Walking into a room starts its hunt**, when anybody does: the world
//!   moves to the room and its creature is put on its marks, noticing you as
//!   it always has. A hunt holds the world in the room while anybody is in
//!   it or near it (`tuning::hunt_held_past` past its edge, so a step out of
//!   the door mid-fight changes nothing). When nobody is, a creature after
//!   somebody follows them and the world goes on following the fighters with
//!   the hunt in it ([`super::roam`]); one after nobody is over, unwon, and
//!   gone.
//! - **A waystone's door is a wall** while the stone is dark: a box the
//!   shape of the doorway, raised on the ground like a slag mound is, unless
//!   somebody already stands on its far side.

use crate::arena::{Solid, Terrain};
use crate::atlas::{self, Atlas, Placed};
use crate::class::{Mechanic, Shield};
use crate::critter::Critters;
use crate::lore::Lore;
use crate::math::V3;
use crate::monster::MAX_MONSTERS;
use crate::species::SpeciesId;
use crate::state::{MAX_PLAYERS, Phase, Player, World};

use super::Kind;

impl World {
    /// **Where this world's coordinates sit on the valley's map**: the origin
    /// of the place it is in. Zero outside the valley. What the renderer adds
    /// to everything it draws, so the world stays still when the frame moves.
    pub fn map_origin(&self) -> V3 {
        if !self.valley.on {
            return V3::ZERO;
        }
        atlas::valley()
            .placed(self.arena)
            .map_or(V3::ZERO, |p| p.at)
    }

    /// The fighters on their feet in the seats that are played, by index.
    fn standing(&self) -> impl Iterator<Item = usize> + '_ {
        let seats = (self.seats as usize).clamp(1, MAX_PLAYERS);
        (0..seats).filter(|&i| self.players[i].health > 0)
    }

    /// The ground this frame, in the valley: the map around the place, and
    /// every waystone door that is shut raised across its doorway.
    pub(crate) fn valley_ground(&self) -> Terrain {
        let atlas = atlas::valley();
        let mut ground = Terrain::placed(atlas, self.arena.get());
        let origin = ground.origin();
        for door in &atlas.doors {
            if door.gate.lit(self.valley.beaten) {
                continue;
            }
            let far = if door.near == door.a.0 {
                door.b.0
            } else {
                door.a.0
            };
            let Some(far) = atlas.placed(far) else {
                continue;
            };
            let beyond = self.standing().any(|i| {
                let feet = self.players[i].pos.add(origin);
                far.holds(feet.x, feet.z)
            });
            if !beyond {
                ground.raise(Solid {
                    min: door.cut.min.sub(origin),
                    max: door.cut.max.sub(origin),
                    material: crate::arena::Material::Stone,
                    shape: crate::arena::Shape::Box,
                });
            }
        }
        ground
    }

    /// **Which place the world should be in**, and move it there: a room
    /// somebody has walked into, or the place the first fighter on their
    /// feet is in. A hunt keeps the world in its room while anybody is in it.
    pub(crate) fn reframe(&mut self) {
        let atlas = atlas::valley();
        let Some(here) = atlas.placed(self.arena).copied() else {
            return;
        };
        let feet = |w: &World, i: usize| w.players[i].pos.add(here.at);
        // **A hunt that ended away from home** wakes the room or range the
        // world is in now, as walking into it would have.
        if !self.hunting() && self.valley.wake {
            self.valley.wake = false;
            let inside = self.standing().any(|i| here.holds_feet(feet(self, i)));
            if let Some(Kind::Room(species)) = super::place(self.arena).map(|p| p.kind)
                && inside
                && !self.resting(self.arena)
            {
                self.begin_hunt(species);
                return;
            }
        }
        if self.hunting() {
            // Held while anybody on their feet is in its home or near it --
            // within `hunt_held_past` of its edge (2026-10-12): a step back
            // out of the door mid-fight is not giving up, and ending the hunt
            // there put the creature back on its marks at full health when
            // they stepped in again, with theirs as they had left it. And
            // held while nobody is on their feet anywhere, which is a hunt
            // being lost, and the round's end decides that.
            let margin = crate::tuning::hunt_held_past();
            let den = atlas.placed(self.den_id()).copied().unwrap_or(here);
            if self.standing().next().is_none() {
                return;
            }
            let near_home = self.standing().any(|i| {
                let f = feet(self, i);
                den.holds_within(f.x, f.z, margin) && den.in_band(f.y)
            });
            if near_home {
                if den.arena != here.arena {
                    // Back home with the hunt still on: the world goes home
                    // with them.
                    self.rebase(atlas, &here, &den);
                }
                return;
            }
            // **Everybody has left its home** (exploration 0023). A creature
            // after somebody follows them -- for ever, after its egg, and for
            // a while, roused -- and the world goes on following the fighters
            // with the hunt in it. One walking home goes on until it is home
            // or out of everyone's reach; one after nobody is over, unwon,
            // and gone.
            let follows = if self.valley.homing {
                !self.gone_home()
            } else {
                self.pursuing()
            };
            if !follows {
                self.calm();
            } else {
                // **Out, the world runs in the coordinates of the place with
                // its own origin nearest the fighters** -- a room, a range,
                // the town -- never a reach's, whose origin is the map's: a
                // hunt keeps points of its own in centimetres (`crate::lore`),
                // a few hundred metres either way of the world's origin, and
                // the Saddle is nine hundred from the map's.
                if let Some(to) = self.nearest_anchor(atlas, &here)
                    && to.arena != here.arena
                {
                    self.rebase(atlas, &here, &to);
                }
                return;
            }
        }
        let room = self.standing().find_map(|i| {
            let f = feet(self, i);
            atlas
                .place_for(f)
                .filter(|p| p.arena != self.arena && is_room(p))
                .copied()
        });
        let to = room.or_else(|| {
            let i = self.standing().next()?;
            let f = feet(self, i);
            // Still here -- and, for a range stacked over another place,
            // still at its height -- unless a stacked place has the feet.
            let over = atlas.place_for(f).filter(|p| p.band.is_some());
            if here.holds_feet(f) && over.is_none_or(|p| p.arena == here.arena) {
                return None;
            }
            atlas.place_for(f).copied()
        });
        let Some(to) = to else {
            return;
        };
        self.rebase(atlas, &here, &to);
        // **One live hunt at a time**: a chase that crosses another
        // creature's room or range does not wake it while the hunt goes on
        // (exploration 0023 §5).
        if self.hunting() {
            return;
        }
        match super::place(to.arena).map(|p| p.kind) {
            // A range beaten not long ago is resting: its pocket is empty.
            Some(Kind::Room(_)) if self.resting(to.arena) => {}
            Some(Kind::Room(species)) => self.begin_hunt(species),
            Some(Kind::Ring) => self.enter_the_ring(),
            _ => {}
        }
    }

    /// **Is this range resting**: beaten less than `tuning::range_rest` ago.
    /// Never, for a room, or a range never beaten.
    pub fn resting(&self, id: crate::arena::ArenaId) -> bool {
        let Some(k) = super::ranges::index(id) else {
            return false;
        };
        let won = self.valley.rested[k];
        won != 0 && self.frame.wrapping_sub(won) < crate::tuning::range_rest_frames()
    }

    /// **A range's temperament, kept** (`valley::ranges`), once a frame after
    /// the creatures have stepped: a creature that is not to start anything
    /// is held in the moment it takes to notice you (`Brain::grace`), and a
    /// pack in its calm, until it is roused -- by being hurt, or by somebody
    /// coming too close to a territorial one. Roused, a skittish pack runs.
    pub(crate) fn tend_range(&mut self) {
        if !self.hunting() {
            return;
        }
        let Some(range) = super::ranges::of(self.den_id()) else {
            return;
        };
        // The range's middle, where the world is: its origin, seen from here.
        let mid = self.den().off;
        use super::ranges::Temperament as T;
        // Hurt since last frame: the creatures' health, all of it, fell.
        let stock: i32 = self
            .monsters
            .iter()
            .flatten()
            .map(|m| m.health.max(0))
            .sum::<i32>()
            + self
                .critters
                .iter()
                .filter(|c| c.alive())
                .map(|c| c.health.max(0) as i32)
                .sum::<i32>();
        let was = self.valley.roused;
        if self.valley.stock != 0 && stock < self.valley.stock {
            self.valley.roused = true;
        }
        self.valley.stock = stock.max(1);
        if let T::Territorial { ring } = range.temperament {
            let ring = crate::arena::cm(ring);
            let near = self.standing().any(|i| {
                let p = self.players[i].pos.sub(mid);
                crate::math::wide_flat_len(V3::new(p.x, crate::fixed::Fx::ZERO, p.z)).raw()
                    <= ring.raw()
            });
            if near {
                self.valley.roused = true;
            }
        }
        // Roused this frame, and not skittish: it wakes now, not when the
        // moment it was held in runs out.
        if self.valley.roused && !was && range.temperament != T::Skittish {
            for m in self.monsters.iter_mut().flatten() {
                m.brain.grace = 0;
            }
            if let Some(pack) = self.pack.as_mut() {
                pack.grace = 0;
                if pack.mood == crate::pack::mood::CALM {
                    pack.mood = crate::pack::mood::HUNTING;
                }
            }
        }
        let hold = match range.temperament {
            T::Hunting => false,
            T::Territorial { .. } | T::Peaceful => !self.valley.roused,
            // A skittish creature never starts and never fights: roused, it
            // runs.
            T::Skittish => true,
        };
        // The far side of the range from the fighters, four fifths of the
        // way to its edge: where a skittish herd runs.
        let away = {
            let (mut sx, mut sz, mut n) = (0i64, 0i64, 0i64);
            for i in self.standing() {
                let p = self.players[i].pos;
                sx += p.x.raw() as i64;
                sz += p.z.raw() as i64;
                n += 1;
            }
            let centre = V3::new(
                crate::fixed::Fx::from_raw((sx / n.max(1)) as i32),
                crate::fixed::Fx::ZERO,
                crate::fixed::Fx::from_raw((sz / n.max(1)) as i32),
            )
            .sub(V3::new(mid.x, crate::fixed::Fx::ZERO, mid.z));
            let out = crate::arena::cm(range.half * 4 / 5);
            mid.add(
                crate::math::wide_normalized(centre)
                    .scale(out)
                    .scale(crate::fixed::Fx::ONE.neg()),
            )
        };
        if hold {
            for m in self.monsters.iter_mut().flatten() {
                m.brain.grace = m.brain.grace.max(u16::MAX / 2);
            }
            if let Some(pack) = self.pack.as_mut() {
                pack.grace = pack.grace.max(u16::MAX / 2);
                if !self.valley.roused {
                    pack.mood = crate::pack::mood::CALM;
                } else if range.temperament == T::Skittish {
                    // It runs for "home", so home is the far side of the
                    // range from whoever started it.
                    pack.mood = crate::pack::mood::BROKEN;
                    pack.home = away;
                }
            }
        }
    }

    /// **The place with its own origin nearest the first fighter on their
    /// feet** -- a room, a range or the town, not a reach (whose origin is the
    /// map's) nor the Ring -- if it is nearer than `here`'s by more than
    /// `tuning::roam_reframe`: where a hunt out of its home runs.
    fn nearest_anchor(&self, atlas: &'static Atlas, here: &Placed) -> Option<Placed> {
        let i = self.standing().next()?;
        let f = self.players[i].pos.add(here.at);
        let dist = |p: &Placed| crate::math::wide_len(f.sub(p.at));
        let anchor = |p: &&Placed| {
            (p.at != V3::ZERO || p.arena == super::START)
                && !matches!(super::place(p.arena).map(|q| q.kind), Some(Kind::Ring))
        };
        let best = atlas
            .places
            .iter()
            .filter(anchor)
            .min_by_key(|p| dist(p).raw())?;
        let stay = if anchor(&here) {
            dist(here).sub(crate::tuning::roam_reframe())
        } else {
            crate::fixed::Fx::MAX
        };
        (dist(best).raw() < stay.raw()).then_some(*best)
    }

    /// Move the world from `from`'s coordinates to `to`'s.
    fn rebase(&mut self, atlas: &Atlas, from: &Placed, to: &Placed) {
        let d = from.at.sub(to.at);
        self.shift(d);
        self.arena = to.arena;
        // The seam you came in by: the one of this place that leads back to
        // the last, if there is one -- what a death with no cairn, and a lost
        // hunt, stand you at.
        let back = atlas.doors.iter().find_map(|door| {
            if door.a.0 == to.arena && door.b.0 == from.arena {
                Some(door.a.1)
            } else if door.b.0 == to.arena && door.a.0 == from.arena {
                Some(door.b.1)
            } else {
                None
            }
        });
        self.valley.came = back.map_or(super::NONE, |k| k + 1);
    }

    /// **Shift everything in the world by `d`**: every position the snapshot
    /// keeps, of every body and everything they have put in the world. What a
    /// change of frame is, and nothing else; `tests/atlas.rs` checks it moves
    /// everything and changes nothing.
    pub fn shift(&mut self, d: V3) {
        for p in self.players.iter_mut() {
            shift_player(p, d);
        }
        for e in self.effects.iter_mut().flatten() {
            e.pos = e.pos.add(d);
            e.home = e.home.add(d);
        }
        crate::ground::shift(&mut self.ground, d);
        for b in self.bolts.iter_mut().flatten() {
            b.pos = b.pos.add(d);
        }
        for s in self.debris.iter_mut().flatten() {
            s.pos = s.pos.add(d);
        }
        for l in self.loose.iter_mut().flatten() {
            l.at = l.at.add(d);
        }
        for h in self.heel.iter_mut() {
            h.at = h.at.add(d);
        }
        for g in self.gusts.iter_mut().flatten() {
            g.pos = g.pos.add(d);
        }
        // **The hunt moves with it** (exploration 0023): since a creature can
        // follow somebody out of its room, the world leaves a room with a hunt
        // in it. Its bodies, what it last saw and where it aims, its critters,
        // its pack's den and sight, and every point of its lore.
        let cm = (crate::lore::cm_of(d.x), crate::lore::cm_of(d.z));
        for m in self.monsters.iter_mut().flatten() {
            m.pos = m.pos.add(d);
            m.brain.seen = m.brain.seen.add(d);
            if m.brain.aim != [0; 2] {
                m.brain.aim = [
                    crate::lore::moved_cm(m.brain.aim[0], cm.0),
                    crate::lore::moved_cm(m.brain.aim[1], cm.1),
                ];
            }
        }
        for c in self.critters.iter_mut() {
            c.pos = c.pos.add(d);
        }
        if let Some(pack) = self.pack.as_mut() {
            pack.home = pack.home.add(d);
            for s in pack.seen.iter_mut() {
                s.pos = s.pos.add(d);
            }
        }
        crate::hazard::shift(&mut self.lore, d);
        crate::noise::shift(&mut self.lore, d);
        // And what only the species knows is a point.
        let mut own = [None; 2 + MAX_MONSTERS];
        own[0] = self.lore.owner;
        own[1] = self.pack.map(|p| p.species);
        for (k, m) in self.monsters.iter().enumerate() {
            own[2 + k] = m.map(|m| m.species);
        }
        for k in 0..own.len() {
            let Some(s) = own[k] else { continue };
            if own[..k].contains(&Some(s)) {
                continue;
            }
            if let Some(hook) = s.get().fight.shift {
                hook(self, d);
            }
        }
    }

    /// **A hunt starts**: the room's creature, its pack and its lore, as
    /// `World::hunt_in` would put them, with the hunters where they stand.
    pub(crate) fn begin_hunt(&mut self, species: SpeciesId) {
        let classes = self.players.map(|p| p.class);
        let range = super::ranges::of(self.arena);
        let mut herd = crate::replay::herd_of(species);
        if let Some(r) = range.filter(|_| species.get().has_body()) {
            for (k, slot) in herd.iter_mut().enumerate() {
                *slot = (k < r.bodies as usize).then_some(species);
            }
        }
        let fresh = World::hunt_in(classes, herd, self.arena);
        self.monsters = fresh.monsters;
        self.pack = fresh.pack;
        self.critters = fresh.critters;
        self.lore = fresh.lore;
        self.phase = Phase::Fighting;
        for m in self.monsters.iter_mut().flatten() {
            m.brain.seen = self.players[0].pos;
        }
        // Its home, for as long as it lasts: where it goes back to.
        self.valley.den = Some(self.arena);
        self.valley.away = 0;
        self.valley.homing = false;
        // A range's creature: as clever as it is far from the town, and as
        // quick to start as its temperament.
        self.valley.roused = false;
        self.valley.stock = 0;
        if let Some(r) = range {
            for m in self.monsters.iter_mut().flatten() {
                m.temper = r.temper;
            }
            if let Some(p) = self.pack.as_mut() {
                p.temper = r.temper;
            }
            self.tend_range();
        }
        self.lay_the_egg(species);
        // A seat nobody plays is out of the hunt, as it always was.
        let seats = (self.seats as usize).clamp(1, MAX_PLAYERS);
        for p in self.players.iter_mut().skip(seats) {
            p.health = 0;
        }
    }

    /// **The Ring**: a fight against each other. With one seat played, the
    /// second is the dummy or the bot, and stands on its mark.
    fn enter_the_ring(&mut self) {
        let seats = (self.seats as usize).clamp(1, MAX_PLAYERS);
        let here = self.arena();
        for i in seats..MAX_PLAYERS {
            let mark = crate::arena::Mark::seat(&here.spawns.versus, i);
            let p = &mut self.players[i];
            let (wins, class) = (p.rounds_won, p.class);
            *p = Player {
                pos: V3::new(mark.at.x, here.ground_under(mark.at), mark.at.z),
                facing: mark.facing,
                rounds_won: wins,
                ..Player::new(class)
            };
            if let Mechanic::Shadow(_) = p.mechanic {
                p.mechanic = Mechanic::Shadow(crate::class::Shadow::attending(p.pos, p.facing));
            }
        }
    }

    /// **A hunt left unfinished**: the creature, its pack and whatever it put
    /// on the floor are gone; the hunters keep everything of theirs.
    pub(crate) fn calm(&mut self) {
        self.home_gone();
        self.monsters = [None; MAX_MONSTERS];
        self.pack = None;
        self.critters = Critters::NONE;
        self.lore = Lore::NONE;
        self.phase = Phase::Fighting;
    }
}

impl World {
    /// **The hunt's home is forgotten**, as a hunt ends: and if it ended
    /// away from home, in another creature's room or range, that one wakes
    /// next frame.
    pub(crate) fn home_gone(&mut self) {
        if let Some(d) = self.valley.den.take() {
            self.valley.wake = d != self.arena;
        }
        self.valley.away = 0;
        self.valley.homing = false;
    }
}

/// A creature's room: walking into one starts its hunt.
fn is_room(p: &Placed) -> bool {
    matches!(super::place(p.arena).map(|p| p.kind), Some(Kind::Room(_)))
}

/// Every position a fighter keeps, moved by `d`.
fn shift_player(p: &mut Player, d: V3) {
    p.pos = p.pos.add(d);
    p.haul_to = p.haul_to.add(d);
    // No wall is no wall wherever the world is: a frame that sets one back
    // to none would otherwise be undone into a point.
    if p.wall.is_some() {
        p.wall.at = p.wall.at.add(d);
    }
    if p.wall_jumped.is_some() {
        p.wall_jumped.at = p.wall_jumped.at.add(d);
    }
    p.rise.floor = p.rise.floor.add(d.y);
    p.ledge.edge.at = p.ledge.edge.at.add(d);
    p.ledge.from = p.ledge.from.add(d);
    p.ball_at = p.ball_at.add(d);
    p.fall_over = p.fall_over.add(d.y);
    p.aim_path.from = p.aim_path.from.add(d);
    p.aim_path.to = p.aim_path.to.add(d);
    match &mut p.mechanic {
        Mechanic::Shield(s) => match s {
            Shield::Held { .. } => {}
            Shield::Planted { pos, .. } | Shield::Flying { pos, .. } => *pos = pos.add(d),
        },
        Mechanic::Shadow(shadow) => {
            shadow.pos = shadow.pos.add(d);
            if let crate::class::Ghost::Casting { from, to, .. } = &mut shadow.doing {
                *from = from.add(d);
                *to = to.add(d);
            }
        }
        Mechanic::Structures(slots) => {
            for s in slots.iter_mut().flatten() {
                s.at = s.at.add(d);
                s.launch_from = s.launch_from.add(d);
            }
        }
        Mechanic::Forms { .. } | Mechanic::Blood | Mechanic::Meter { .. } => {}
    }
}
