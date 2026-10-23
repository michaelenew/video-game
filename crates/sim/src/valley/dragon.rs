//! **The Sleeper**: a great beast asleep in the mountains north of the
//! Pinewood, which is one of those mountains (the owner, 2026-10-22: "the
//! dragon itself should essentially be the mountain, and its dragon like
//! features should be just discernible only from certain angles").
//!
//! It is the land, not a creature: a height added to the mountain wherever
//! its body lies, before the land's noise and crags are laid over it, so it
//! wears the same rock, snow and broken crests as the range round it and
//! what is drawn is what a foot would stand on.
//!
//! **Its shape is a model's, posed** (2026-10-23): a CC0 low-poly dragon
//! (`assets/sleeper/`) laid down asleep by `scripts/sleeper/pose.py` --
//! curled up, its head under a forelimb, lying a little on its side, wings
//! shrunk to nubs -- and looked down on by `scripts/sleeper/terrain.py`,
//! which writes `sleeper.bin`: how far its body stands over the mountain,
//! cell by cell. Change the pose or where it lies there, not here. The
//! heights are level data (`tests/knobs.rs`), judged by looking at them.

use crate::fixed::Fx;

/// The heightfield `scripts/sleeper/terrain.py` writes: a header of six
/// i32 and a u16 a cell.
static FIELD: &[u8] = include_bytes!("sleeper.bin");

/// **How much of the Sleeper shows over the mountain**, by how high the
/// mountain has risen off its floor there: none of it until the mountain is
/// this high, so no valley or foothill is touched, all of it by
/// [`FULL_FROM`].
const SHOWS_FROM: Fx = Fx::from_int(40);
const FULL_FROM: Fx = Fx::from_int(100);

fn header(k: usize) -> i32 {
    i32::from_le_bytes([
        FIELD[k * 4],
        FIELD[k * 4 + 1],
        FIELD[k * 4 + 2],
        FIELD[k * 4 + 3],
    ])
}

/// The field's height at a cell (east, north), in its own units; zero off
/// it.
fn cell(i: i32, j: i32, nx: i32, nz: i32) -> i64 {
    if i < 0 || j < 0 || i >= nx || j >= nz {
        return 0;
    }
    let k = 24 + 2 * (j * nx + i) as usize;
    u16::from_le_bytes([FIELD[k], FIELD[k + 1]]) as i64
}

/// **How far the Sleeper raises the land** at a map point (x, z), on a
/// mountain that has risen `rise` off its floor there: zero off its body.
pub fn lift(x: Fx, z: Fx, rise: Fx) -> Fx {
    if rise.raw() <= SHOWS_FROM.raw() {
        return Fx::ZERO;
    }
    let (west, south, side, nx, nz, scale) = (
        header(0),
        header(1),
        header(2),
        header(3),
        header(4),
        header(5),
    );
    // Where the point is in the grid, in cells, 16.16.
    let one = Fx::ONE.raw() as i64;
    let at = |v: Fx, edge: i32| (v.raw() as i64 * 100 - edge as i64 * one) / side as i64;
    let (gx, gz) = (at(x, west), at(z, south));
    if gx < 0 || gz < 0 || gx >= (nx as i64 - 1) * one || gz >= (nz as i64 - 1) * one {
        return Fx::ZERO;
    }
    let (i, j) = ((gx / one) as i32, (gz / one) as i32);
    let (fx, fz) = (gx % one, gz % one);
    let row = |j: i32| cell(i, j, nx, nz) * (one - fx) + cell(i + 1, j, nx, nz) * fx;
    let h = (row(j) * (one - fz) + row(j + 1) * fz) / one;
    let h = Fx::from_raw((h / scale as i64) as i32);
    if h.raw() <= 0 {
        return Fx::ZERO;
    }
    let shows = rise
        .sub(SHOWS_FROM)
        .div(FULL_FROM.sub(SHOWS_FROM))
        .min(Fx::ONE);
    h.mul(shows)
}
