//! The hunt's own room in the snapshot: one fixed region each fight lays out
//! differently.
//!
//! The cast after the Ridgeback each wants a little state of its own -- the
//! Mireback sixteen tar pools, the Sandmaw a ring of noises and a spine, the
//! Veilstalker a trail of footfalls, the Siegeshell its vents, the things it
//! sheds and the wall it is walking to. **Only one fight is ever loaded**, so
//! summing every creature's worst case into the `World` would spend the 4 KiB
//! snapshot on bytes no fight uses at once. Instead the `World` keeps one
//! region, [`Lore`], sized to the largest fight, and the fight's species says
//! how it is laid out ([`Layout`], in its [`crate::species::FightDecl`]):
//!
//! ```text
//! | hazards (P4) | noises (P5) | objectives (P7) | the species' own cells |
//! ```
//!
//! Each piece is a whole number of sixteen-byte **cells**. The three generic
//! lists are read and written through their own modules --
//! [`crate::hazard`], [`crate::noise`], [`crate::objective`] -- which encode
//! each entry into one cell; what is left is the species', as plain words
//! ([`Lore::own`], [`Lore::own_mut`], [`Lore::word`], [`Lore::set_word`]),
//! for it to lay out however its own file says.
//!
//! Plain integers rather than a union of typed structs: `crates/sim` has no
//! `unsafe` and no dependencies, a word is the same on every machine, and the
//! hash ([`Lore::is_blank`]) can walk it without knowing whose it is.
//!
//! **A fight that uses none of it hashes exactly as before there was any**:
//! the checksum writes the region only when a cell is not zero, which is what
//! keeps the pinned hunts pinned.

use crate::species::SpeciesId;

/// How many cells the region holds: the largest fight's layout, with room.
///
/// The measured worst cases, from each creature's document (bytes, then
/// cells): the Mireback's sixteen pools and its braziers and swallow (18),
/// the Siegeshell's eight vents, twelve falling things, the anchors and its
/// siege state, and the wall (19), the Veilstalker's 32 footfalls, paint,
/// veil and four hazards (15), the Sandmaw's noise ring, spine, sinkhole and
/// swallow (14), the Broodmother's six patches, four strands, six sacs and
/// her list (13). Twenty-four leaves every one of them five cells or more to
/// grow into; `tests/lore.rs` checks every registered species' layout fits.
pub const CELLS: usize = 24;

/// One cell: sixteen bytes, as four words.
pub type Cell = [u32; 4];

/// The empty cell.
pub const BLANK: Cell = [0; 4];

/// How a fight lays the region out: how many cells each generic list gets,
/// and how many the species keeps for itself. Declared in the species' table;
/// the region itself is the same size whoever owns it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Layout {
    /// Floor hazards (P4): one cell each. At most [`crate::hazard::MAX_HAZARDS`].
    pub hazards: u8,
    /// The noise ring (P5): one cell per noise remembered.
    pub noises: u8,
    /// Defended things (P7): one cell each.
    pub objectives: u8,
    /// The species' own state, in cells: what its document itemises. Declared
    /// so that `tests/lore.rs` can check the whole layout fits.
    pub own: u8,
}

impl Layout {
    /// Nothing at all: the Ridgeback, versus, the gnats' fights in the field.
    pub const NONE: Layout = Layout {
        hazards: 0,
        noises: 0,
        objectives: 0,
        own: 0,
    };

    /// Every cell it names, generic lists and own state together.
    pub const fn cells(&self) -> usize {
        self.hazards as usize + self.noises as usize + self.objectives as usize + self.own as usize
    }

    const fn noise_base(&self) -> usize {
        self.hazards as usize
    }

    const fn objective_base(&self) -> usize {
        self.noise_base() + self.noises as usize
    }

    const fn own_base(&self) -> usize {
        self.objective_base() + self.objectives as usize
    }
}

/// The region. One per `World`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Lore {
    /// Whose layout the cells are in: the fight's species, or `None` for a
    /// fight without one (versus).
    pub owner: Option<SpeciesId>,
    cells: [Cell; CELLS],
}

impl Default for Lore {
    fn default() -> Self {
        Lore::NONE
    }
}

impl Lore {
    /// Nobody's: versus, and every fight before a creature is placed.
    pub const NONE: Lore = Lore {
        owner: None,
        cells: [BLANK; CELLS],
    };

    /// An empty region laid out for this species' fight.
    pub const fn of(owner: SpeciesId) -> Lore {
        Lore {
            owner: Some(owner),
            cells: [BLANK; CELLS],
        }
    }

    /// The layout the cells are in.
    pub fn layout(&self) -> Layout {
        self.owner.map_or(Layout::NONE, |s| s.get().fight.layout)
    }

    /// Every cell zero: nothing placed, nothing remembered. The hash writes the
    /// region only when this is false.
    pub fn is_blank(&self) -> bool {
        self.cells.iter().all(|c| *c == BLANK)
    }

    /// Every cell, for the hash.
    pub fn cells(&self) -> &[Cell; CELLS] {
        &self.cells
    }

    fn span(&self, base: usize, len: usize) -> core::ops::Range<usize> {
        let lo = base.min(CELLS);
        lo..(base + len).min(CELLS)
    }

    // ---- the three generic lists, a cell at a time ----

    pub(crate) fn hazard_cells(&self) -> &[Cell] {
        let l = self.layout();
        &self.cells[self.span(0, l.hazards as usize)]
    }

    pub(crate) fn hazard_cells_mut(&mut self) -> &mut [Cell] {
        let l = self.layout();
        let r = self.span(0, l.hazards as usize);
        &mut self.cells[r]
    }

    pub(crate) fn noise_cells(&self) -> &[Cell] {
        let l = self.layout();
        &self.cells[self.span(l.noise_base(), l.noises as usize)]
    }

    pub(crate) fn noise_cells_mut(&mut self) -> &mut [Cell] {
        let l = self.layout();
        let r = self.span(l.noise_base(), l.noises as usize);
        &mut self.cells[r]
    }

    pub(crate) fn objective_cells(&self) -> &[Cell] {
        let l = self.layout();
        &self.cells[self.span(l.objective_base(), l.objectives as usize)]
    }

    pub(crate) fn objective_cells_mut(&mut self) -> &mut [Cell] {
        let l = self.layout();
        let r = self.span(l.objective_base(), l.objectives as usize);
        &mut self.cells[r]
    }

    // ---- the species' own ----

    /// The species' own cells: everything after the generic lists. Its file
    /// says what they mean.
    pub fn own(&self) -> &[Cell] {
        let l = self.layout();
        &self.cells[self.span(l.own_base(), CELLS)]
    }

    pub fn own_mut(&mut self) -> &mut [Cell] {
        let l = self.layout();
        let r = self.span(l.own_base(), CELLS);
        &mut self.cells[r]
    }

    /// The species' own state as a flat run of words: word `i` is word
    /// `i % 4` of own cell `i / 4`. Zero past the end, so a read never panics.
    pub fn word(&self, i: usize) -> u32 {
        self.own().get(i / 4).map_or(0, |c| c[i % 4])
    }

    /// Write one of the species' own words. A write past the end is dropped:
    /// `tests/lore.rs` is where a layout that does not fit is caught, not the
    /// middle of a rollback.
    pub fn set_word(&mut self, i: usize, v: u32) {
        if let Some(c) = self.own_mut().get_mut(i / 4) {
            c[i % 4] = v;
        }
    }

    /// A word as a signed number -- a fixed-point value's raw bits, a count
    /// that can go below zero.
    pub fn int(&self, i: usize) -> i32 {
        self.word(i) as i32
    }

    pub fn set_int(&mut self, i: usize, v: i32) {
        self.set_word(i, v as u32);
    }
}

/// Pack two signed sixteen-bit halves into a word, low first.
pub const fn halves(lo: i16, hi: i16) -> u32 {
    (lo as u16 as u32) | ((hi as u16 as u32) << 16)
}

/// The low half of a word, signed.
pub const fn lo(w: u32) -> i16 {
    w as u16 as i16
}

/// The high half of a word, signed.
pub const fn hi(w: u32) -> i16 {
    (w >> 16) as u16 as i16
}

/// Centimetres, saturating into sixteen bits: ±327 m, which every arena is
/// inside (the valley is 240 m long and centred).
pub fn to_cm(v: crate::fixed::Fx) -> i16 {
    let cm = (v.raw() as i64 * 100) >> 16;
    cm.clamp(i16::MIN as i64, i16::MAX as i64) as i16
}

/// Centimetres back to fixed point.
pub const fn from_cm(cm: i16) -> crate::fixed::Fx {
    crate::fixed::Fx::ratio(cm as i32, 100)
}

/// A point to three centimetre halves.
pub fn point_cm(p: crate::math::V3) -> [i16; 3] {
    [to_cm(p.x), to_cm(p.y), to_cm(p.z)]
}

/// Three centimetre halves back to a point.
pub const fn cm_point(p: [i16; 3]) -> crate::math::V3 {
    crate::math::V3::new(from_cm(p[0]), from_cm(p[1]), from_cm(p[2]))
}
