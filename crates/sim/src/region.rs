//! Regions: the overlapping hexagons the world keeps its books in.
//!
//! Every build runs the region logic on every frame, single player included,
//! so that the logic the open world will depend on is dogfooded while the game
//! is still one arena. Nothing here is read by `World::advance`: a fight plays
//! out the same whatever the grid is. What reads it is the ledger
//! (`crates/regions`), which keeps a checksum per region, runs the neighbours'
//! heartbeats and decides whether the next frame may go ahead.
//!
//! The model, from `docs/design/regions.md`:
//!
//! - The floor is tiled by pointy-topped hexagons of circumradius `size`, and a
//!   corner of three of them sits on the origin -- the middle of every arena.
//! - The **horizon** `h = reach + speed·delay` is how far anything can act
//!   before a newer outcome from a neighbour arrives.
//! - A body's **regions** are the hexagons within `h + overlap` of it: its peer
//!   sends inputs to all of them and runs all of them.
//! - A body's **checksum zones** are the hexagons within `overlap` of it, so
//!   neighbouring zones overlap and a body near a line counts in both.
//! - The geometry is sound only while `h + overlap < size / 2`: the middle of
//!   an edge is `size / 2` from the hexagons at its two ends, and past that a
//!   body there would be in four regions.
//!
//! The hexagon arithmetic itself is `crate::math::hex_*`.

use crate::TICK_HZ;
use crate::fixed::Fx;
use crate::math::{self, V3};
use crate::tuning as t;

/// One hexagon, by its axial coordinate.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash, PartialOrd, Ord)]
pub struct RegionId {
    pub q: i16,
    pub r: i16,
}

impl RegionId {
    pub const fn new(q: i16, r: i16) -> RegionId {
        RegionId { q, r }
    }

    /// The six hexagons sharing an edge with this one.
    pub fn neighbours(self) -> [RegionId; 6] {
        math::hex_neighbours(self.q as i32, self.r as i32).map(|(q, r)| id(q, r))
    }

    /// The id as one word, for a hash.
    pub const fn word(self) -> u32 {
        (self.q as u16 as u32) << 16 | self.r as u16 as u32
    }
}

fn id(q: i32, r: i32) -> RegionId {
    RegionId::new(
        q.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        r.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
    )
}

/// The most hexagons [`Grid::within`] looks at: the one a point is in and the
/// six around it. A sound grid never puts a point in more than three, and a
/// count above three is how an unsound one is noticed.
pub const MAX_AROUND: usize = 7;

/// A short list of regions, in a fixed array: a body's regions, or its zones.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Regions {
    ids: [RegionId; MAX_AROUND],
    len: u8,
}

impl Regions {
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &[RegionId] {
        &self.ids[..self.len()]
    }

    pub fn contains(&self, id: RegionId) -> bool {
        self.as_slice().contains(&id)
    }

    fn push(&mut self, id: RegionId) {
        if self.len() < MAX_AROUND {
            self.ids[self.len()] = id;
            self.len += 1;
        }
    }
}

/// The grid's numbers, read from the Oven once and carried, so one frame's
/// books are all kept against the same grid even if a knob moves mid-frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Grid {
    /// The farthest anything acts in one go, declared.
    pub reach: Fx,
    /// The fastest anything moves, in metres a second, declared.
    pub speed: Fx,
    /// How many frames old a neighbour's outcome may be.
    pub delay: u32,
    /// How far a checksum zone reaches past its hexagon.
    pub overlap: Fx,
    /// A hexagon's circumradius.
    pub size: Fx,
}

impl Grid {
    /// The grid as the Oven has it now.
    pub fn tuned() -> Grid {
        Grid {
            reach: t::region_reach(),
            speed: t::region_speed(),
            delay: t::region_delay(),
            overlap: t::region_overlap(),
            size: t::region_size(),
        }
    }

    /// `h = reach + speed·delay`: how far influence travels before a newer
    /// outcome from a neighbour arrives to account for it.
    pub fn horizon(&self) -> Fx {
        let travelled = self
            .speed
            .mul(Fx::from_int(self.delay as i32))
            .div(Fx::from_int(TICK_HZ as i32));
        self.reach.add(travelled)
    }

    /// How far from a hexagon a body may be and still be one of its members.
    pub fn membership(&self) -> Fx {
        self.horizon().add(self.overlap)
    }

    /// Whether the grid keeps every body in at most three regions.
    pub fn sound(&self) -> bool {
        self.membership().raw() < math::half(self.size).raw()
    }

    /// The hexagon a point is in.
    pub fn home(&self, p: V3) -> RegionId {
        let (x, z) = self.local(p);
        let (q, r) = math::hex_at(x, z, self.size);
        id(q, r)
    }

    /// The fight's home region: the one holding the origin, where every arena
    /// is centred. Global state -- the phase, the arena, the pack, the lore --
    /// is counted here (`docs/design/regions.md` §"A region checksum").
    pub fn fight_home(&self) -> RegionId {
        self.home(V3::ZERO)
    }

    /// How far a point is from a region's hexagon: zero inside it.
    pub fn gap(&self, region: RegionId, p: V3) -> Fx {
        let (x, z) = self.local(p);
        math::hex_gap(
            region.q as i32,
            region.r as i32,
            self.size,
            V3::new(x, Fx::ZERO, z),
        )
    }

    /// Every hexagon within `radius` of a point, its own first.
    pub fn within(&self, p: V3, radius: Fx) -> Regions {
        let home = self.home(p);
        let mut out = Regions::default();
        out.push(home);
        for n in home.neighbours() {
            if self.gap(n, p).raw() <= radius.raw() {
                out.push(n);
            }
        }
        out
    }

    /// The regions a body at `p` is a member of.
    pub fn regions_of(&self, p: V3) -> Regions {
        self.within(p, self.membership())
    }

    /// The checksum zones a body at `p` counts toward.
    pub fn zones_of(&self, p: V3) -> Regions {
        self.within(p, self.overlap)
    }

    /// Whether a body at `p` counts toward a region's checksum.
    pub fn in_zone(&self, region: RegionId, p: V3) -> bool {
        self.gap(region, p).raw() <= self.overlap.raw()
    }

    /// Whether a body at `p` is a member of a region.
    pub fn in_region(&self, region: RegionId, p: V3) -> bool {
        self.gap(region, p).raw() <= self.membership().raw()
    }

    /// World floor coordinates to the tiling's: shifted so a corner shared by
    /// three hexagons -- the `+z` corner of hexagon `(0, 0)` -- is the origin.
    fn local(&self, p: V3) -> (Fx, Fx) {
        (p.x, p.z.add(self.size))
    }
}

/// One thing in the world that stands somewhere, by kind and slot: what a
/// region checksum is made of, and what the watchdog names when two of them
/// disagree.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Body {
    Fighter(u8),
    Creature(u8),
    Critter(u8),
    Bolt(u8),
    Shard(u8),
    Gust(u8),
    Effect(u8),
}

impl Body {
    /// The body as one word, for a hash.
    pub const fn word(self) -> u32 {
        let (kind, slot) = match self {
            Body::Fighter(i) => (1, i),
            Body::Creature(i) => (2, i),
            Body::Critter(i) => (3, i),
            Body::Bolt(i) => (4, i),
            Body::Shard(i) => (5, i),
            Body::Gust(i) => (6, i),
            Body::Effect(i) => (7, i),
        };
        kind << 8 | slot as u32
    }
}
