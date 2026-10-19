//! **What you carry, and the morph** (the world's first use of breeding,
//! `docs/design/exploration/0022` §12).
//!
//! Every creature has an **egg**: it lies near the creature when its hunt
//! begins, and walking into it picks it up. A creature whose egg you carry
//! does not stop: it is roused at once, it hunts the one carrying it, and
//! the hunt cannot be walked away from -- it ends when the creature is
//! beaten, or when the carrier falls and drops the egg. A beaten creature
//! drops its **trophies**, picked up the same way.
//!
//! Each seat has a **morph**, a companion that follows at heel. Crouch
//! beside it and it eats the last thing you picked up: an egg makes it
//! that creature's young (an egg whose creature still hunts you will not be
//! eaten -- beat it first), and a trophy is a generation, a change you can
//! see at once (`breed::cross::feed`).
//!
//! **What is kept where.** What a seat carries and its morph are the
//! journey's ([`Keep`], in `valley::Journey`): they go with you through a
//! seam and survive a won hunt, which rebuilds the fighters. What lies on
//! the floor and where each morph stands are the place's ([`Loose`],
//! [`Heel`], on the `World`): they are in its coordinates, shift with it,
//! and are left behind when you go through a seam (a morph catches up).

use crate::breed::archetypes;
use crate::breed::cross::TrophyKind;
use crate::breed::locomotion;
use crate::breed::morph::Morph;
use crate::breed::posture;
use crate::fixed::Fx;
use crate::math::V3;
use crate::species::SpeciesId;
use crate::state::{MAX_PLAYERS, World};
use crate::tuning as t;

/// Things one seat can carry.
pub const SATCHEL: usize = 4;

/// Things that can lie on the floor of a place at once.
pub const LOOSE: usize = 5;

/// Something carried, or lying on the floor.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Item {
    /// A creature's egg, by species.
    Egg(SpeciesId),
    /// A trophy: what it is, and whose.
    Trophy(TrophyKind, SpeciesId),
}

/// An item on the floor.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Lying {
    pub item: Item,
    /// On the floor, in the place's coordinates.
    pub at: V3,
}

pub type Loose = [Option<Lying>; LOOSE];

/// Where a seat's morph stands, and how it walks.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Heel {
    /// On the floor, in the place's coordinates.
    pub at: V3,
    /// Turns from `+x` toward `+z`.
    pub facing: Fx,
    /// Where in its stride, nought to one.
    pub phase: Fx,
    /// Its walk: metres a second, and frames a stride -- worked out once
    /// each time it changes, not every frame.
    pub speed: Fx,
    pub cycle: u16,
    /// Moving this frame.
    pub going: bool,
    /// Put in the place yet: false after a seam, until it catches up.
    pub placed: bool,
    /// Frames before it will eat again.
    pub cool: u8,
    /// The generation its walk was worked out for, plus one.
    pub walk_of: u16,
}

/// What every seat carries, and its morph: the journey's.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Keep {
    /// Each seat's satchel, in the order things were picked up.
    pub satchel: [[Option<Item>; SATCHEL]; MAX_PLAYERS],
    pub morphs: [Morph; MAX_PLAYERS],
}

impl Default for Keep {
    fn default() -> Keep {
        // A morph that has eaten no egg is a small, plain thing: the
        // smallest body there is, young.
        let base = archetypes::of_species(SpeciesId::GNAWERS)
            .copied()
            .unwrap_or(archetypes::RIDGEBACK);
        Keep {
            satchel: [[None; SATCHEL]; MAX_PLAYERS],
            morphs: core::array::from_fn(|i| Morph::hatch(&base, i as u8)),
        }
    }
}

impl Keep {
    /// Whether seat `i` carries the egg of `s`.
    pub fn carries_egg(&self, i: usize, s: SpeciesId) -> bool {
        self.satchel[i]
            .iter()
            .flatten()
            .any(|it| *it == Item::Egg(s))
    }

    fn take(&mut self, i: usize, item: Item) -> bool {
        match self.satchel[i].iter_mut().find(|s| s.is_none()) {
            Some(slot) => {
                *slot = Some(item);
                true
            }
            None => false,
        }
    }

    /// Every egg of `s` seat `i` carries, taken out.
    pub(crate) fn lose_eggs(&mut self, i: usize, s: SpeciesId) -> usize {
        let mut n = 0;
        for slot in self.satchel[i].iter_mut() {
            if *slot == Some(Item::Egg(s)) {
                *slot = None;
                n += 1;
            }
        }
        n
    }
}

impl World {
    fn seats_played(&self) -> usize {
        (self.seats as usize).clamp(1, MAX_PLAYERS)
    }

    /// Something put on the floor at `at`, if there is room.
    pub fn lay(&mut self, item: Item, at: V3) -> bool {
        match self.loose.iter_mut().find(|l| l.is_none()) {
            Some(slot) => {
                *slot = Some(Lying { item, at });
                true
            }
            None => false,
        }
    }

    /// **A hunt begins**: the creature's egg on the floor near it, unless
    /// somebody already carries it or it already lies here.
    pub(crate) fn lay_the_egg(&mut self, species: SpeciesId) {
        if !self.valley.on || archetypes::of_species(species).is_none() {
            return;
        }
        let egg = Item::Egg(species);
        let seats = self.seats_played();
        let carried = (0..seats).any(|i| self.valley.keep.carries_egg(i, species));
        let lying = self.loose.iter().flatten().any(|l| l.item == egg);
        if carried || lying {
            return;
        }
        // Behind the creature, from where the hunters came in: you have to
        // go past it to get it.
        let mother = self
            .monsters
            .iter()
            .flatten()
            .map(|m| m.pos)
            .next()
            .or_else(|| self.critters.iter().find(|c| c.alive()).map(|c| c.pos));
        let Some(mother) = mother else {
            return;
        };
        let from = self.players[0].pos;
        let away = V3::new(mother.x.sub(from.x), Fx::ZERO, mother.z.sub(from.z));
        let len = away.len();
        let dir = if len.raw() > 0 {
            away.scale(Fx::ONE.div(len))
        } else {
            V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO)
        };
        let spot = mother.add(dir.scale(t::egg_behind()));
        let at = V3::new(spot.x, self.terrain().ground_under(spot), spot.z);
        self.lay(egg, at);
    }

    /// **A hunt won**: each beaten creature's trophies, on the floor where
    /// it fell -- a few of what its parts are, a different few each time.
    pub(crate) fn drop_trophies(&mut self) {
        if !self.valley.on {
            return;
        }
        let mut fallen = [None; crate::monster::MAX_MONSTERS];
        for (k, m) in self.monsters.iter().enumerate() {
            if let Some(m) = m {
                fallen[k] = Some((m.species, m.pos));
            }
        }
        if fallen.iter().all(|f| f.is_none())
            && let Some(c) = self.critters.iter().next()
            && let Some(s) = self.hunted().into_iter().flatten().next()
        {
            fallen[0] = Some((s, c.pos));
        }
        let each = t::trophies_dropped().max(1) as usize;
        for (species, at) in fallen.into_iter().flatten() {
            let Some(g) = archetypes::of_species(species) else {
                continue;
            };
            let kinds = archetypes::drops(g);
            let n = kinds.iter().flatten().count();
            if n == 0 {
                continue;
            }
            for j in 0..each.min(n) {
                let k = (self.frame as usize + j) % n;
                let Some(kind) = kinds.iter().flatten().nth(k).copied() else {
                    continue;
                };
                // Spread round where it fell, a little apart.
                let off = crate::breed::actor::spin(
                    V3::new(t::pickup_reach(), Fx::ZERO, Fx::ZERO),
                    Fx::ratio(j as i32, each.min(n) as i32),
                );
                self.lay(Item::Trophy(kind, species), at.add(off));
            }
        }
    }

    /// The seat carrying the egg of a creature being hunted here, if any:
    /// the one it will not stop for.
    pub fn egg_carrier(&self) -> Option<(usize, SpeciesId)> {
        if !self.valley.on || !self.hunting() {
            return None;
        }
        let seats = self.seats_played();
        for s in self.hunted().into_iter().flatten() {
            for i in 0..seats {
                if self.players[i].health > 0 && self.valley.keep.carries_egg(i, s) {
                    return Some((i, s));
                }
            }
        }
        None
    }

    /// **Once a frame, in the valley**: pick up what is underfoot, the
    /// creature after its egg, the morphs at heel, and feeding.
    pub(crate) fn step_keep(&mut self) {
        if !self.valley.on {
            return;
        }
        let seats = self.seats_played();
        // A fighter down in a hunt drops the eggs they carry where they fell.
        if self.hunting() {
            for i in 0..seats {
                if self.players[i].health > 0 {
                    continue;
                }
                for s in self.hunted().into_iter().flatten() {
                    if self.valley.keep.lose_eggs(i, s) > 0 {
                        let at = self.players[i].pos;
                        self.lay(Item::Egg(s), at);
                    }
                }
            }
        }
        // Underfoot: anything in reach goes in the satchel, if there is room.
        let reach = t::pickup_reach();
        for i in 0..seats {
            if self.players[i].health <= 0 {
                continue;
            }
            let p = self.players[i].pos;
            for k in 0..LOOSE {
                let Some(l) = self.loose[k] else { continue };
                let flat = V3::new(l.at.x.sub(p.x), Fx::ZERO, l.at.z.sub(p.z)).len();
                let up = l.at.y.sub(p.y).abs();
                if flat <= reach && up <= t::body_height() && self.valley.keep.take(i, l.item) {
                    self.loose[k] = None;
                }
            }
        }
        // The creature after its egg: roused, held by nothing, on the one
        // carrying it.
        if let Some((i, _)) = self.egg_carrier() {
            self.valley.roused = true;
            // Further off than it fights from, it knows where its egg is:
            // nothing it cannot see hides that (exploration 0023).
            let carrier = self.players[i].pos;
            let engage = t::roam_engage();
            for m in self.monsters.iter_mut().flatten() {
                m.brain.grace = 0;
                m.brain.target = i as u8;
                if crate::math::wide_flat_dist(carrier, m.pos).raw() > engage.raw() {
                    m.brain.seen = carrier;
                }
            }
            if let Some(pack) = self.pack.as_mut() {
                pack.grace = 0;
                pack.mood = crate::pack::mood::HUNTING;
            }
        }
        // In a hunt: for each seat, the way away from the creature nearest
        // them, which their morph keeps to.
        let mut fight_from = [None; MAX_PLAYERS];
        if self.hunting() {
            for (i, away) in fight_from.iter_mut().enumerate().take(seats) {
                let p = self.players[i].pos;
                let near = self
                    .monsters
                    .iter()
                    .flatten()
                    .filter(|m| m.alive())
                    .map(|m| m.pos)
                    .chain(self.critters.iter().filter(|c| c.alive()).map(|c| c.pos))
                    .min_by_key(|c| V3::new(c.x.sub(p.x), Fx::ZERO, c.z.sub(p.z)).len().raw());
                if let Some(c) = near {
                    let off = V3::new(p.x.sub(c.x), Fx::ZERO, p.z.sub(c.z));
                    let len = off.len();
                    *away = Some(if len.raw() > 0 {
                        off.scale(Fx::ONE.div(len))
                    } else {
                        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO)
                    });
                }
            }
        }
        for i in 0..seats {
            self.heel(i, &fight_from);
        }
    }

    /// Seat `i`'s morph: put beside them if it is not here yet, and kept
    /// at heel -- following when they are further than its follow
    /// distance, at the gait that distance asks for.
    fn heel(&mut self, i: usize, fight_from: &[Option<V3>; MAX_PLAYERS]) {
        let p = self.players[i];
        let morph = self.valley.keep.morphs[i];
        let h = &mut self.heel[i];
        if !h.placed {
            h.at = V3::new(p.pos.x.sub(Fx::ONE), p.pos.y, p.pos.z.sub(Fx::ONE));
            h.facing = crate::math::atan2_turns(p.facing.z, p.facing.x);
            h.placed = true;
        }
        if h.walk_of != morph.generation.wrapping_add(1) || h.speed == Fx::ZERO {
            let g = morph.shown();
            let q = posture::posture(&g);
            let gait = locomotion::at(&g, &q, Fx::ratio(1, 2));
            h.speed = gait.speed.max(Fx::ONE);
            h.cycle = gait.cycle.clamp(8, 240) as u16;
            h.walk_of = morph.generation.wrapping_add(1);
        }
        h.cool = h.cool.saturating_sub(1);
        let to = V3::new(p.pos.x.sub(h.at.x), Fx::ZERO, p.pos.z.sub(h.at.z));
        let dist = to.len();
        let follow = t::morph_follow();
        // Left a long way behind (a jump, a fall, a seam): it catches up.
        if dist > t::morph_catch_up() {
            h.at = V3::new(p.pos.x.sub(Fx::ONE), p.pos.y, p.pos.z.sub(Fx::ONE));
            h.going = false;
            return;
        }
        // **Out of the way in a fight**: it keeps well back, on the far side
        // of you from the creature nearest you -- never between you, never
        // where you are swinging. At peace it comes to heel.
        let (goal, slack) = match fight_from[i] {
            Some(away) => (p.pos.add(away.scale(t::morph_follow_fight())), Fx::ONE),
            None => (p.pos, follow),
        };
        let to_goal = V3::new(goal.x.sub(h.at.x), Fx::ZERO, goal.z.sub(h.at.z));
        let gap = to_goal.len();
        h.going = gap > slack;
        if h.going {
            // Faster the further off it is, up to its hurry.
            let urge = Fx::ONE
                .add(gap.sub(slack).div(follow))
                .min(t::morph_hurry());
            let step = h
                .speed
                .mul(urge)
                .div(Fx::from_int(locomotion::FPS))
                .min(gap);
            let dir = to_goal.scale(Fx::ONE.div(gap));
            h.at = h.at.add(dir.scale(step));
            h.facing = crate::math::atan2_turns(dir.z, dir.x);
            let next = h
                .phase
                .add(Fx::ONE.div(Fx::from_int(h.cycle as i32)).mul(urge));
            h.phase = if next >= Fx::ONE {
                next.sub(Fx::ONE)
            } else {
                next
            };
        } else if dist.raw() > 0 {
            // Standing, it watches you.
            h.facing = crate::math::atan2_turns(to.z, to.x);
        }
        // On the floor.
        let at = h.at;
        self.heel[i].at.y = self.terrain().ground_under(at);
    }

    /// Whether seat `i` could feed its morph what is in slot `k` now, and
    /// if not, why: what the bag shows beside an item.
    pub fn can_feed(&self, i: usize, k: usize) -> Result<Item, Refusal> {
        let item = self
            .valley
            .keep
            .satchel
            .get(i)
            .and_then(|bag| bag.get(k))
            .copied()
            .flatten()
            .ok_or(Refusal::Empty)?;
        if self.players[i].health <= 0 {
            return Err(Refusal::Down);
        }
        if let Item::Egg(s) = item
            && self.egg_carrier().is_some_and(|(_, c)| c == s)
        {
            return Err(Refusal::Chased(s));
        }
        if self.heel[i].cool > 0 {
            return Err(Refusal::Eating);
        }
        Ok(item)
    }

    /// Seat `i` feeds its morph what is in slot `k` of its satchel, chosen
    /// from the bag ([`crate::input::Destination::Feed`]). An egg makes it
    /// that creature's young -- not while its creature still hunts them for
    /// it -- and a trophy grows it a generation.
    pub(crate) fn feed_slot(&mut self, i: usize, k: usize) {
        let Ok(item) = self.can_feed(i, k) else {
            return;
        };
        self.valley.keep.satchel[i][k] = None;
        match item {
            Item::Egg(s) => {
                if let Some(g) = archetypes::of_species(s) {
                    self.valley.keep.morphs[i].eat_egg(g, s.0);
                }
            }
            Item::Trophy(kind, s) => {
                if let Some(g) = archetypes::of_species(s) {
                    self.valley.keep.morphs[i].eat(kind, g);
                }
            }
        }
        self.heel[i].cool = t::feed_cool().clamp(1, 250) as u8;
    }
}

/// Why a morph will not eat something.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refusal {
    /// Nothing in that slot.
    Empty,
    /// The one feeding it is down.
    Down,
    /// It is still eating the last thing.
    Eating,
    /// Its creature is hunting you for it: beat it first.
    Chased(SpeciesId),
}

impl World {
    /// Whether anything has been carried, fed or laid down yet.
    pub(crate) fn keep_touched(&self) -> bool {
        let k = &self.valley.keep;
        self.loose.iter().any(|l| l.is_some())
            || k.satchel.iter().flatten().any(|s| s.is_some())
            || k.morphs
                .iter()
                .any(|m| m.generation > 0 || m.of != Morph::UNFORMED)
    }

    /// The satchels, the morphs, what lies on the floor and the morphs'
    /// places, into the world's checksum.
    pub(crate) fn hash_keep(&self, h: &mut crate::state::Fnv) {
        let item = |it: Option<Item>| -> u32 {
            match it {
                None => 0,
                Some(Item::Egg(s)) => 1 | (s.0 as u32) << 8,
                Some(Item::Trophy(k, s)) => 2 | (k as u32) << 8 | (s.0 as u32) << 16,
            }
        };
        let k = &self.valley.keep;
        for bag in &k.satchel {
            for it in bag {
                h.write_u32(item(*it));
            }
        }
        for m in &k.morphs {
            h.write_u64(m.genome.fingerprint());
            h.write_u32(m.growth.raw() as u32);
            h.write_u32(m.generation as u32 | (m.of as u32) << 16);
        }
        for l in &self.loose {
            match l {
                Some(l) => {
                    h.write_u32(item(Some(l.item)));
                    for c in [l.at.x, l.at.y, l.at.z] {
                        h.write_u32(c.raw() as u32);
                    }
                }
                None => h.write_u32(0),
            }
        }
        for heel in &self.heel {
            for c in [heel.at.x, heel.at.y, heel.at.z, heel.facing, heel.phase] {
                h.write_u32(c.raw() as u32);
            }
        }
    }
}
