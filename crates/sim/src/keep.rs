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
//! Each seat has a **morph** at heel that eats what you carry -- but the
//! morph, and what you carry besides eggs, are **not here** (2026-10-19).
//! They are each player's own, kept by their game (`game::bag`), and
//! using an item is an input like a move: both machines see "fed it a
//! Mantis feather" on the frame it was sent and agree it was eaten, never
//! having had to agree it was carried. The owner: *"I want to avoid bloat
//! in the state the players need to agree on since we're on a budget for
//! that."*
//!
//! **What is here** is what the fight itself reads. What lies on the floor
//! ([`Loose`], on the `World`, in the place's coordinates). Which eggs each
//! seat carries ([`Keep`], the journey's: a bit a creature, because a
//! creature hunts whoever carries its egg, and both machines have to agree
//! who that is). And what each seat picked up this frame
//! ([`World::picked`]), which is how the game hears of a trophy to put in
//! its bag -- read by frame number, so a rollback that undoes a pickup
//! undoes it in the bag too.

use crate::breed::archetypes;
use crate::breed::cross::TrophyKind;
use crate::fixed::Fx;
use crate::math::V3;
use crate::species::SpeciesId;
use crate::state::{MAX_PLAYERS, World};
use crate::tuning as t;

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

/// Which creatures' eggs each seat carries: the journey's, so they go with
/// you through a seam and survive a won hunt, which rebuilds the fighters.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Keep {
    /// A bit a species, by its id.
    pub eggs: [u32; MAX_PLAYERS],
}

impl Keep {
    /// Whether seat `i` carries the egg of `s`.
    pub fn carries_egg(&self, i: usize, s: SpeciesId) -> bool {
        s.0 < 32 && self.eggs.get(i).is_some_and(|e| e & 1 << s.0 != 0)
    }

    /// Seat `i` has the egg of `s` now.
    pub fn carry_egg(&mut self, i: usize, s: SpeciesId) {
        if s.0 < 32
            && let Some(e) = self.eggs.get_mut(i)
        {
            *e |= 1 << s.0;
        }
    }

    /// The egg of `s` out of seat `i`'s hands; whether it had it.
    pub fn lose_eggs(&mut self, i: usize, s: SpeciesId) -> bool {
        let had = self.carries_egg(i, s);
        if had {
            self.eggs[i] &= !(1 << s.0);
        }
        had
    }

    /// Every egg seat `i` carries.
    pub fn eggs_of(&self, i: usize) -> impl Iterator<Item = SpeciesId> + '_ {
        (0..32u8)
            .map(SpeciesId)
            .filter(move |s| self.carries_egg(i, *s))
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

    /// **Once a frame, in the valley**: eggs dropped by the fallen, what is
    /// underfoot picked up, and the creature after its egg.
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
                    if self.valley.keep.lose_eggs(i, s) {
                        let at = self.players[i].pos;
                        self.lay(Item::Egg(s), at);
                    }
                }
            }
        }
        // Underfoot: one thing a seat a frame. An egg is carried; a
        // trophy is the game's to keep (`picked`).
        let reach = t::pickup_reach();
        for i in 0..seats {
            if self.players[i].health <= 0 || self.picked[i].is_some() {
                continue;
            }
            let p = self.players[i].pos;
            for k in 0..LOOSE {
                let Some(l) = self.loose[k] else { continue };
                let flat = V3::new(l.at.x.sub(p.x), Fx::ZERO, l.at.z.sub(p.z)).len();
                let up = l.at.y.sub(p.y).abs();
                if flat <= reach && up <= t::body_height() {
                    self.loose[k] = None;
                    self.picked[i] = Some(l.item);
                    if let Item::Egg(s) = l.item {
                        self.valley.keep.carry_egg(i, s);
                    }
                    break;
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
    }

    /// Seat `i` used an item (`Destination::Feed`): fed it to its morph.
    /// A trophy was the sender's own, and nothing here changes -- the
    /// games that draw the morph read the input. An egg is let go of here
    /// too, unless its creature is still hunting them for it, when it is
    /// not eaten at all (and the games, seeing it still carried, agree).
    pub(crate) fn fed(&mut self, i: usize, item: Item) {
        if let Item::Egg(s) = item
            && self.refusal(i, item).is_none()
        {
            self.valley.keep.lose_eggs(i, s);
        }
    }

    /// Seat `i` was given an item from the `--dev` panel: an egg carried,
    /// a trophy put in `picked` for its game to keep.
    pub(crate) fn granted(&mut self, i: usize, item: Item) {
        if let Item::Egg(s) = item {
            self.valley.keep.carry_egg(i, s);
        }
        if let Some(p) = self.picked.get_mut(i) {
            *p = Some(item);
        }
    }

    /// Why seat `i` cannot feed its morph `item` now, as far as the world
    /// knows: down, an egg it does not carry, or an egg whose creature is
    /// after it. Whether it has a trophy, and whether its morph is still
    /// eating, are its game's to know.
    pub fn refusal(&self, i: usize, item: Item) -> Option<Refusal> {
        if self.players.get(i).is_none_or(|p| p.health <= 0) {
            return Some(Refusal::Down);
        }
        if let Item::Egg(s) = item {
            if !self.valley.keep.carries_egg(i, s) {
                return Some(Refusal::Empty);
            }
            if self.egg_carrier().is_some_and(|(_, c)| c == s) {
                return Some(Refusal::Chased(s));
            }
        }
        None
    }
}

/// Why a morph will not eat something.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refusal {
    /// It is not carried.
    Empty,
    /// The one feeding it is down.
    Down,
    /// It is still eating the last thing.
    Eating,
    /// Its creature is hunting you for it: beat it first.
    Chased(SpeciesId),
}

impl World {
    /// Whether anything has been carried or laid down yet.
    pub(crate) fn keep_touched(&self) -> bool {
        self.loose.iter().any(|l| l.is_some()) || self.valley.keep.eggs.iter().any(|e| *e != 0)
    }

    /// The eggs carried and what lies on the floor, into the world's
    /// checksum.
    pub(crate) fn hash_keep(&self, h: &mut crate::state::Fnv) {
        for e in &self.valley.keep.eggs {
            h.write_u32(*e);
        }
        for l in &self.loose {
            match l {
                Some(l) => {
                    h.write_u32(match l.item {
                        Item::Egg(s) => 1 | (s.0 as u32) << 8,
                        Item::Trophy(k, s) => 2 | (k as u32) << 8 | (s.0 as u32) << 16,
                    });
                    for c in [l.at.x, l.at.y, l.at.z] {
                        h.write_u32(c.raw() as u32);
                    }
                }
                None => h.write_u32(0),
            }
        }
    }
}
