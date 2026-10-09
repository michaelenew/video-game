//! **The atlas**: many places laid out on one map, and the tiles that let a
//! query touch only what is near it.
//!
//! An [`Arena`] is one table of boxes, walked from end to end by every query.
//! That is why an arena was capped at sixty-four of them, and why the valley
//! was a string of separate rooms joined by teleports. The atlas is the other
//! half: it takes places, each an arena drawn in its own coordinates, puts each
//! at an offset on one map, cuts the doorways where two of them meet, and
//! sorts every box into square **tiles**. A question about a point or a small
//! box reads the few tiles under it and nothing else, so what a frame costs
//! depends on what is near the bodies, not on how big the world is.
//!
//! - **The map is composed once**, the first time it is asked for, and never
//!   changes after. It is not in the snapshot: both machines build the same
//!   one from the same tables, the way both read the same arena tables.
//!   Building it allocates; asking it anything does not.
//! - **Order is kept.** A tile lists its boxes in map order, and a query over
//!   several tiles merges them in map order, each once. Collision resolves one
//!   box at a time, so the order is part of the answer, and this keeps it the
//!   same on every machine and wherever the question is asked from.
//! - **A place keeps its own coordinates.** The simulation runs in the
//!   coordinates of the place the fighters are in (`World::arena`), so a
//!   creature's fight is the fight it was tuned in, floor at zero and all. The
//!   atlas only says where that place sits on the map, which is what turns the
//!   neighbours' boxes into the same coordinates (`arena::Terrain`).
//!
//! The valley's layout is [`crate::valley::layout`]. The design is
//! `docs/design/atlas.md`.

use std::sync::OnceLock;

use crate::arena::{self, Arena, ArenaId, Material, Solid};
use crate::fixed::Fx;
use crate::math::V3;
use crate::valley::{Gate, Zone};

/// The side of a tile, in whole metres.
///
/// Small enough that a body's own box is one to four of them, large enough
/// that a long cliff is a dozen entries rather than a hundred. A world of a
/// million boxes at the valley's density is a few hundred thousand tiles, and
/// a query still reads four.
pub const TILE_M: i32 = 16;

/// **The widest a map may be**, in metres each way: 16.16 holds about
/// 32 km either side of zero, and a place at one edge of the map has to see
/// the other edge, with room for anything thrown past it.
pub const MAX_SPAN_M: i32 = 16_000;

/// The most tiles one query merges at a time. A box wider than this many
/// tiles is answered by walking every solid in order instead, which is
/// correct and slow -- and only the rare whole-place questions ask one.
pub const MAX_TILES: usize = 64;

/// A place, put down on the map.
#[derive(Clone, Copy, Debug)]
pub struct Placed {
    pub arena: ArenaId,
    /// Where its own origin is on the map.
    pub at: V3,
    /// Its footprint on the map (x, z): its bounds, and every box it has,
    /// whichever is larger. A point in it is in this place.
    pub lo: (Fx, Fx),
    pub hi: (Fx, Fx),
    /// Its boxes are `solids[first..end]` of the map, in its own order (with
    /// any cut into pieces where a doorway goes through it).
    pub first: u32,
    pub end: u32,
}

impl Placed {
    pub fn get(&self) -> &'static Arena {
        self.arena.get()
    }

    /// Is the map point (x, z) in its footprint? Edges count.
    pub fn holds(&self, x: Fx, z: Fx) -> bool {
        x.raw() >= self.lo.0.raw()
            && x.raw() <= self.hi.0.raw()
            && z.raw() >= self.lo.1.raw()
            && z.raw() <= self.hi.1.raw()
    }

    /// A point in its own coordinates, on the map.
    pub fn to_map(&self, p: V3) -> V3 {
        p.add(self.at)
    }

    /// A map point, in its own coordinates.
    pub fn from_map(&self, p: V3) -> V3 {
        p.sub(self.at)
    }
}

/// Where a box of the map came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Source {
    /// The place, by its index in [`Atlas::places`], or [`EXTRA`].
    pub place: u16,
    /// Its index in that place's own table.
    pub index: u16,
}

/// A box the layout adds that belongs to no place: a bridge, a yard's wall.
pub const EXTRA: u16 = u16::MAX;

/// **A doorway**: where two places meet, cut through whatever stood there.
#[derive(Clone, Copy, Debug)]
pub struct Door {
    /// The box that was cut, on the map. A body walks through it.
    pub cut: Zone,
    /// The two sides: a place and the seam of it that leads through.
    pub a: (ArenaId, u8),
    pub b: (ArenaId, u8),
    /// What lets you through. A waystone's door is shut, by a box the shape
    /// of the doorway, while the stone is dark and you are on its near side.
    pub gate: Gate,
    /// The near side of a gated door: the one it is shut from.
    pub near: ArenaId,
}

/// **What a map is composed from**: every place and where it goes, every
/// doorway, and the boxes that belong to no place. Worked out from a layout
/// of joints by [`crate::valley::layout::plan`]; anything else can write one
/// directly.
#[derive(Clone, Debug, Default)]
pub struct Plan {
    /// Each place, where its origin goes on the map, and how much ground it
    /// has round its own footprint. The order is the order their boxes are
    /// resolved in, and where footprints overlap the earlier place's floor is
    /// the floor.
    pub places: Vec<(ArenaId, V3, Fx)>,
    pub doors: Vec<DoorPlan>,
    /// Boxes in no place, on the map: cliffs round a room, a door's sill.
    pub extras: Vec<Solid>,
    /// The floor where no place is: far enough under every place that
    /// anything there has fallen out of the world.
    pub void: Fx,
}

/// One doorway of a [`Plan`].
#[derive(Clone, Copy, Debug)]
pub struct DoorPlan {
    pub a: (ArenaId, u8),
    pub b: (ArenaId, u8),
    /// The box cut out of everything it meets, on the map.
    pub cut: Solid,
}

/// **The map**: every place, every box, every doorway, and the tiles.
#[derive(Debug)]
pub struct Atlas {
    pub places: Vec<Placed>,
    /// Every box on the map, in map coordinates and map order: each place's
    /// in turn, then the extras.
    pub solids: Vec<Solid>,
    /// Where each box came from, index for index.
    pub sources: Vec<Source>,
    pub doors: Vec<Door>,
    /// The floor where no place is.
    pub void: Fx,
    /// Everything on the map lies inside this (x, z).
    pub lo: (Fx, Fx),
    pub hi: (Fx, Fx),
    grid: Grid,
}

/// The tiles: for each, the boxes that touch it and the places whose
/// footprint does, both ascending.
#[derive(Debug)]
struct Grid {
    /// The tile coordinate of the first column and row.
    x0: i32,
    z0: i32,
    w: i32,
    h: i32,
    starts: Vec<u32>,
    entries: Vec<u32>,
    place_starts: Vec<u32>,
    place_entries: Vec<u16>,
}

/// The tile a map coordinate is in, along one axis.
fn tile_of(v: Fx) -> i32 {
    // Floor division of the raw value by the tile's raw size: a point on a
    // tile's lower edge is in that tile, whichever side of zero it is.
    let size = TILE_M as i64 * Fx::ONE.raw() as i64;
    (v.raw() as i64).div_euclid(size) as i32
}

impl Grid {
    /// The tiles under a box, clamped to the grid; `None` if it misses the
    /// grid entirely.
    fn span(&self, lo: (Fx, Fx), hi: (Fx, Fx)) -> Option<(i32, i32, i32, i32)> {
        let (ax, az) = (tile_of(lo.0) - self.x0, tile_of(lo.1) - self.z0);
        let (bx, bz) = (tile_of(hi.0) - self.x0, tile_of(hi.1) - self.z0);
        if bx < 0 || bz < 0 || ax >= self.w || az >= self.h {
            return None;
        }
        Some((ax.max(0), az.max(0), bx.min(self.w - 1), bz.min(self.h - 1)))
    }

    fn index(&self, tx: i32, tz: i32) -> usize {
        (tz * self.w + tx) as usize
    }

    fn solids(&self, t: usize) -> &[u32] {
        &self.entries[self.starts[t] as usize..self.starts[t + 1] as usize]
    }

    fn places(&self, t: usize) -> &[u16] {
        &self.place_entries[self.place_starts[t] as usize..self.place_starts[t + 1] as usize]
    }
}

/// **An arena's footprint** in its own coordinates (x, z): its bounds, and
/// every box it has, whichever reaches further.
pub fn footprint(arena: &Arena) -> ((Fx, Fx), (Fx, Fx)) {
    let b = arena.bounds;
    let (mut lo, mut hi) = ((b.lo_x, b.lo_z), (b.hi_x, b.hi_z));
    for s in arena.solids {
        lo = (lo.0.min(s.min.x), lo.1.min(s.min.z));
        hi = (hi.0.max(s.max.x), hi.1.max(s.max.z));
    }
    (lo, hi)
}

/// A box minus another: the pieces of `s` outside `cut`, in a fixed order.
/// `s` itself if they do not overlap.
fn carve(s: &Solid, cut: &Solid, out: &mut Vec<Solid>) {
    let overlaps = s.min.x.raw() < cut.max.x.raw()
        && s.max.x.raw() > cut.min.x.raw()
        && s.min.y.raw() < cut.max.y.raw()
        && s.max.y.raw() > cut.min.y.raw()
        && s.min.z.raw() < cut.max.z.raw()
        && s.max.z.raw() > cut.min.z.raw();
    if !overlaps {
        out.push(*s);
        return;
    }
    let piece = |min: V3, max: V3| Solid {
        min,
        max,
        material: s.material,
    };
    let (x0, x1) = (s.min.x.max(cut.min.x), s.max.x.min(cut.max.x));
    let (z0, z1) = (s.min.z.max(cut.min.z), s.max.z.min(cut.max.z));
    // Either side in x, the whole box's height and depth.
    if s.min.x.raw() < cut.min.x.raw() {
        out.push(piece(s.min, V3::new(cut.min.x, s.max.y, s.max.z)));
    }
    if s.max.x.raw() > cut.max.x.raw() {
        out.push(piece(V3::new(cut.max.x, s.min.y, s.min.z), s.max));
    }
    // Either side in z, within the cut's x.
    if s.min.z.raw() < cut.min.z.raw() {
        out.push(piece(
            V3::new(x0, s.min.y, s.min.z),
            V3::new(x1, s.max.y, cut.min.z),
        ));
    }
    if s.max.z.raw() > cut.max.z.raw() {
        out.push(piece(
            V3::new(x0, s.min.y, cut.max.z),
            V3::new(x1, s.max.y, s.max.z),
        ));
    }
    // Under and over, within the cut's x and z: a floor stays a floor, and a
    // lintel stays a lintel.
    if s.min.y.raw() < cut.min.y.raw() {
        out.push(piece(V3::new(x0, s.min.y, z0), V3::new(x1, cut.min.y, z1)));
    }
    if s.max.y.raw() > cut.max.y.raw() {
        out.push(piece(V3::new(x0, cut.max.y, z0), V3::new(x1, s.max.y, z1)));
    }
}

impl Atlas {
    /// **Compose a layout**: put each place down, cut the doorways, and sort
    /// every box into its tiles.
    pub fn compose(plan: &Plan) -> Atlas {
        let cuts: Vec<Solid> = plan.doors.iter().map(|d| d.cut).collect();
        let cut_all = |s: Solid| {
            let mut pieces = vec![s];
            for cut in &cuts {
                let mut next = Vec::with_capacity(pieces.len() + 2);
                for p in &pieces {
                    carve(p, cut, &mut next);
                }
                pieces = next;
            }
            pieces
        };

        let mut places = Vec::with_capacity(plan.places.len());
        let mut solids = Vec::new();
        let mut sources = Vec::new();
        for (k, &(id, at, apron)) in plan.places.iter().enumerate() {
            let arena = id.get();
            let (lo, hi) = footprint(arena);
            let (lo, hi) = (
                (lo.0.sub(apron), lo.1.sub(apron)),
                (hi.0.add(apron), hi.1.add(apron)),
            );
            let first = solids.len() as u32;
            for (i, s) in arena.solids.iter().enumerate() {
                let moved = Solid {
                    min: s.min.add(at),
                    max: s.max.add(at),
                    material: s.material,
                };
                for piece in cut_all(moved) {
                    solids.push(piece);
                    sources.push(Source {
                        place: k as u16,
                        index: i as u16,
                    });
                }
            }
            places.push(Placed {
                arena: id,
                at,
                lo: (lo.0.add(at.x), lo.1.add(at.z)),
                hi: (hi.0.add(at.x), hi.1.add(at.z)),
                first,
                end: solids.len() as u32,
            });
        }
        for (i, s) in plan.extras.iter().enumerate() {
            for piece in cut_all(*s) {
                solids.push(piece);
                sources.push(Source {
                    place: EXTRA,
                    index: i as u16,
                });
            }
        }

        let doors = plan
            .doors
            .iter()
            .map(|d| {
                let seam_gate = |(id, k): (ArenaId, u8)| {
                    crate::valley::place(id)
                        .and_then(|p| p.seam(k))
                        .map_or(Gate::Open, |s| s.gate)
                };
                let (gate, near) = match (seam_gate(d.a), seam_gate(d.b)) {
                    (g @ Gate::Waystone { .. }, _) => (g, d.a.0),
                    (_, g @ Gate::Waystone { .. }) => (g, d.b.0),
                    _ => (Gate::Open, d.a.0),
                };
                Door {
                    cut: Zone {
                        min: d.cut.min,
                        max: d.cut.max,
                    },
                    a: d.a,
                    b: d.b,
                    gate,
                    near,
                }
            })
            .collect();

        // The extent of everything: every footprint and every box.
        let mut lo = (Fx::from_raw(i32::MAX), Fx::from_raw(i32::MAX));
        let mut hi = (Fx::from_raw(i32::MIN), Fx::from_raw(i32::MIN));
        let mut grow = |a: (Fx, Fx), b: (Fx, Fx)| {
            lo = (lo.0.min(a.0), lo.1.min(a.1));
            hi = (hi.0.max(b.0), hi.1.max(b.1));
        };
        for p in &places {
            grow(p.lo, p.hi);
        }
        for s in &solids {
            grow((s.min.x, s.min.z), (s.max.x, s.max.z));
        }
        if places.is_empty() && solids.is_empty() {
            lo = (Fx::ZERO, Fx::ZERO);
            hi = (Fx::ZERO, Fx::ZERO);
        }
        // **The map has to fit the numbers.** Positions are 16.16, about
        // 32 km either side of zero, and the world runs in the coordinates of
        // one place with the rest of the map around it -- so everything on
        // the map has to be within that of every place, which is the map no
        // wider than [`MAX_SPAN_M`] each way. Past it a sum would saturate in
        // the middle of a frame and a wall would be somewhere else; that is
        // a map to build differently (`docs/design/atlas.md`), and finding it
        // out here, as the map is built, is better than finding it there.
        let wide = |a: Fx, b: Fx| (b.raw() as i64 - a.raw() as i64) > MAX_SPAN_M as i64 * 65536;
        assert!(
            !wide(lo.0, hi.0) && !wide(lo.1, hi.1),
            "the map is wider than {MAX_SPAN_M} m: 16.16 cannot hold one place's view of the rest"
        );

        let (x0, z0) = (tile_of(lo.0), tile_of(lo.1));
        let (w, h) = (tile_of(hi.0) - x0 + 1, tile_of(hi.1) - z0 + 1);
        let tiles = (w * h) as usize;
        let span = |a: (Fx, Fx), b: (Fx, Fx)| {
            (
                tile_of(a.0) - x0,
                tile_of(a.1) - z0,
                tile_of(b.0) - x0,
                tile_of(b.1) - z0,
            )
        };
        // Two passes each: count, then fill, so every tile's list is one run
        // of one array and ascending by construction.
        let mut counts = vec![0u32; tiles];
        let mut place_counts = vec![0u32; tiles];
        let each = |(ax, az, bx, bz): (i32, i32, i32, i32), f: &mut dyn FnMut(usize)| {
            for tz in az..=bz {
                for tx in ax..=bx {
                    f((tz * w + tx) as usize);
                }
            }
        };
        for s in &solids {
            each(span((s.min.x, s.min.z), (s.max.x, s.max.z)), &mut |t| {
                counts[t] += 1
            });
        }
        for p in &places {
            each(span(p.lo, p.hi), &mut |t| place_counts[t] += 1);
        }
        let prefix = |counts: &[u32]| {
            let mut starts = Vec::with_capacity(counts.len() + 1);
            let mut sum = 0;
            starts.push(0);
            for c in counts {
                sum += c;
                starts.push(sum);
            }
            starts
        };
        let starts = prefix(&counts);
        let place_starts = prefix(&place_counts);
        let mut entries = vec![0u32; *starts.last().unwrap_or(&0) as usize];
        let mut place_entries = vec![0u16; *place_starts.last().unwrap_or(&0) as usize];
        let mut fill = starts.clone();
        for (i, s) in solids.iter().enumerate() {
            each(span((s.min.x, s.min.z), (s.max.x, s.max.z)), &mut |t| {
                entries[fill[t] as usize] = i as u32;
                fill[t] += 1;
            });
        }
        let mut fill = place_starts.clone();
        for (k, p) in places.iter().enumerate() {
            each(span(p.lo, p.hi), &mut |t| {
                place_entries[fill[t] as usize] = k as u16;
                fill[t] += 1;
            });
        }

        Atlas {
            places,
            solids,
            sources,
            doors,
            void: plan.void,
            lo,
            hi,
            grid: Grid {
                x0,
                z0,
                w,
                h,
                starts,
                entries,
                place_starts,
                place_entries,
            },
        }
    }

    /// The place an arena is on this map, by its index, if it is on it.
    pub fn index_of(&self, id: ArenaId) -> Option<usize> {
        self.places.iter().position(|p| p.arena == id)
    }

    /// The place an arena is on this map, if it is on it.
    pub fn placed(&self, id: ArenaId) -> Option<&Placed> {
        self.places.iter().find(|p| p.arena == id)
    }

    /// **The place a map point is in**: the first, in layout order, whose
    /// footprint holds it.
    pub fn place_at(&self, x: Fx, z: Fx) -> Option<&Placed> {
        let (tx, tz) = (tile_of(x) - self.grid.x0, tile_of(z) - self.grid.z0);
        if tx < 0 || tz < 0 || tx >= self.grid.w || tz >= self.grid.h {
            return None;
        }
        self.grid
            .places(self.grid.index(tx, tz))
            .iter()
            .map(|&k| &self.places[k as usize])
            .find(|p| p.holds(x, z))
    }

    /// **The floor's height at a map point**: the relief of the place it is
    /// in, raised to where that place sits; the void anywhere else.
    pub fn relief_at(&self, x: Fx, z: Fx) -> Fx {
        match self.place_at(x, z) {
            Some(p) => p.get().relief_at(x.sub(p.at.x), z.sub(p.at.z)).add(p.at.y),
            None => self.void,
        }
    }

    /// Is the floor one plane at this map point -- no hills, no ramps? The
    /// void counts as flat.
    pub fn is_flat_at(&self, x: Fx, z: Fx) -> bool {
        self.place_at(x, z)
            .is_none_or(|p| arena::relief::is_flat(p.arena))
    }

    /// What the floor is made of at a map point, ignoring every box.
    pub fn floor_at(&self, x: Fx, z: Fx) -> Material {
        match self.place_at(x, z) {
            Some(p) => p.get().floor_at(x.sub(p.at.x), z.sub(p.at.z)),
            None => Material::Rock,
        }
    }

    /// **The boxes near a map box**, by index, ascending, each once: every
    /// box in a tile the query box touches. A superset of the boxes that
    /// overlap it, never a subset.
    pub fn near(&self, lo: (Fx, Fx), hi: (Fx, Fx)) -> Near<'_> {
        let mut near = Near {
            atlas: self,
            cursors: [(0, 0); MAX_TILES],
            n: 0,
            last: None,
            all: None,
        };
        let Some((ax, az, bx, bz)) = self.grid.span(lo, hi) else {
            return near;
        };
        let count = ((bx - ax + 1) * (bz - az + 1)) as usize;
        if count > MAX_TILES {
            near.all = Some(0);
            return near;
        }
        for tz in az..=bz {
            for tx in ax..=bx {
                let t = self.grid.index(tx, tz);
                let (s, e) = (self.grid.starts[t], self.grid.starts[t + 1]);
                if s < e {
                    near.cursors[near.n as usize] = (s, e);
                    near.n += 1;
                }
            }
        }
        near
    }

    /// The most boxes any one tile holds: what a query near a body costs at
    /// worst, per tile. `tests/atlas.rs` holds it under a budget.
    pub fn busiest_tile(&self) -> usize {
        (0..(self.grid.w * self.grid.h) as usize)
            .map(|t| self.grid.solids(t).len())
            .max()
            .unwrap_or(0)
    }

    /// How many tiles there are, and how many entries they hold between them.
    pub fn tile_count(&self) -> (usize, usize) {
        (
            (self.grid.w * self.grid.h) as usize,
            self.grid.entries.len(),
        )
    }

    /// The tile (column, row) a map point is in: what the renderer keys what
    /// it has loaded by.
    pub fn tile_at(&self, x: Fx, z: Fx) -> (i32, i32) {
        (tile_of(x), tile_of(z))
    }

    /// The boxes of one tile, by its (column, row): the renderer's unit of
    /// loading. Empty off the grid.
    pub fn tile(&self, tx: i32, tz: i32) -> &[u32] {
        let (x, z) = (tx - self.grid.x0, tz - self.grid.z0);
        if x < 0 || z < 0 || x >= self.grid.w || z >= self.grid.h {
            return &[];
        }
        self.grid.solids(self.grid.index(x, z))
    }

    /// Is box `i` *homed* in tile (tx, tz) -- the tile its lower corner is
    /// in? A box spanning many tiles is drawn once, by its home.
    pub fn homed(&self, i: u32, tx: i32, tz: i32) -> bool {
        let s = &self.solids[i as usize];
        tile_of(s.min.x) == tx && tile_of(s.min.z) == tz
    }
}

/// The boxes near a query, in map order: a merge of the tiles' lists.
pub struct Near<'a> {
    atlas: &'a Atlas,
    cursors: [(u32, u32); MAX_TILES],
    n: u8,
    last: Option<u32>,
    /// Too many tiles: every box in order, from here.
    all: Option<u32>,
}

impl Iterator for Near<'_> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if let Some(i) = self.all {
            if (i as usize) < self.atlas.solids.len() {
                self.all = Some(i + 1);
                return Some(i);
            }
            return None;
        }
        loop {
            let mut best: Option<(usize, u32)> = None;
            for k in 0..self.n as usize {
                let (pos, end) = self.cursors[k];
                if pos < end {
                    let v = self.atlas.grid.entries[pos as usize];
                    if best.is_none_or(|(_, b)| v < b) {
                        best = Some((k, v));
                    }
                }
            }
            let (k, v) = best?;
            self.cursors[k].0 += 1;
            if self.last != Some(v) {
                self.last = Some(v);
                return Some(v);
            }
        }
    }
}

/// **The valley, as one map**: composed from [`crate::valley::layout`] the
/// first time anything asks, and the same map for the life of the program.
pub fn valley() -> &'static Atlas {
    static VALLEY: OnceLock<Atlas> = OnceLock::new();
    VALLEY.get_or_init(|| Atlas::compose(&crate::valley::layout::plan()))
}
