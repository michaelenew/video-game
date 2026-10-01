//! The Den: the Pair's arena, in the scrub highlands (the-pair.md §11).
//!
//! **30 × 30 m of dry grass inside a 1.5 m wall a metre and a half thick**,
//! the proving ground's two 1.5 m platforms, and **three standing stones**,
//! 3.5 m tall and 1.2 m across.
//!
//! | What | Size | Why |
//! | --- | --- | --- |
//! | The wall | 1.5 m high, 1.5 m thick | The tempting corner, and a top a cat walks out onto: wide enough to perch on (`PerchWidth`), so a back to it is a lip behind you (§3) |
//! | Two platforms | 4 × 8 m, 1.5 m high | The other temptation: the cats reach a top in one leap |
//! | Three stones | 3.5 m tall, 1.2 m across | What splitting the pair uses when a class brings no wall of its own: too narrow to perch on, tall enough to break a cat's sight |
//!
//! Two of the stones stand off two corners, six metres in; the third stands
//! between the platforms on the north side, so a hunter in the middle has one
//! in reach whichever way the fight turned. The design's "no corner more than
//! eight metres from one" cannot be had with three stones and four corners --
//! two opposite corners are forty metres apart -- so the other two corners
//! are covered by the platforms and by the wall's own perch.

use super::{Arena, ArenaId, Bounds, HuntMarks, Mark, Material, Solid, Spawns};
use crate::species::SpeciesId;

use Material::{Grass, Rock, Stone};

pub static ARENA: Arena = Arena {
    id: ArenaId::PAIR,
    name: "Den",
    creature: Some(SpeciesId::PAIR),
    // The bounds take in the wall's top: a cat walks out onto it, and the
    // walk keeps a creature inside the bounds less its keep-out.
    bounds: Bounds::cm((-1500, 1500), (-1500, 1500)),
    floor: Grass,
    regions: &[],
    solids: &SOLIDS,
    spawns: Spawns {
        versus: [Mark::cm(-400, 0, (400, 0)), Mark::cm(400, 0, (-400, 0))],
        hunt: Some(HuntMarks {
            // In from the west, the Den in front of them.
            hunters: [
                Mark::cm(-1000, -150, (0, -150)),
                Mark::cm(-1000, 150, (0, 150)),
            ],
            // The pair together at the east end, side by side, watching.
            creatures: [
                Mark::cm(1000, -250, (-1000, -250)),
                Mark::cm(1000, 250, (-1000, 250)),
            ],
        }),
    },
    sites: &[],
};

/// The standing stones, by their middles in centimetres.
pub const STONES: [(i32, i32); 3] = [(950, 950), (-950, -950), (0, 1000)];

/// A standing stone: 1.2 m across and 3.5 m tall.
const fn stone(x: i32, z: i32) -> Solid {
    Solid::cm([x - 60, 0, z - 60], [x + 60, 350, z + 60], Rock)
}

const SOLIDS: [Solid; 9] = [
    // The wall, a metre and a half thick and as high, just inside the bounds.
    Solid::cm([-1500, 0, -1500], [-1350, 150, 1500], Stone),
    Solid::cm([1350, 0, -1500], [1500, 150, 1500], Stone),
    Solid::cm([-1500, 0, -1500], [1500, 150, -1350], Stone),
    Solid::cm([-1500, 0, 1350], [1500, 150, 1500], Stone),
    // The proving ground's two platforms.
    Solid::cm([-900, 0, -400], [-500, 150, 400], Stone),
    Solid::cm([500, 0, -400], [900, 150, 400], Stone),
    stone(STONES[0].0, STONES[0].1),
    stone(STONES[1].0, STONES[1].1),
    stone(STONES[2].0, STONES[2].1),
];
