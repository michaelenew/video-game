//! **A gully** (2026-10-16): the wall jump's piece of level. A drop too long
//! to jump, between two walls close enough to jump between -- off one, across,
//! off the other -- with a landing a little lower at its far end.
//!
//! ```text
//!           wall                                   wall
//!   ======================================================
//!   takeoff |  ->  kick  ->  kick  ->  kick  ->  | landing
//!   ======================================================
//!   (from above: the run goes left to right, side to side)
//! ```
//!
//! **The wall jump is the way over, for everybody.** Measured on the bench's
//! gully lanes (`arena::bench`, 2026-10-16): no class's plain running jump
//! carries [`LONG`] -- the Dual mage's, the longest, carries about nine
//! metres -- and five or six kicks carry every class over it, the Bulwark
//! included; the rest of the roster makes fifteen to eighteen. The walls
//! stand [`WIDE`] apart, so a kick crosses in about a third of a second: one
//! more kick before the falloff has spent the rise (`tuning::wall_jump_falloff`).
//! They stand [`ABOVE`] over the takeoff, out of any hand's reach, so a wall's
//! top is not a ledge to go along instead.
//!
//! **How it is played**: run along one side of the takeoff, jump, drift into
//! the wall beside you, and press jump as you touch it (or just before: the
//! press is kept for the wall); the push carries you to the other wall, and
//! you press again. Hold forward: since 2026-10-20 a slide keeps the run and
//! a kick goes the way the stick points along the wall.
//!
//! What is here is the shape; where a gully goes is its route's.

use super::{Material, Solid};

/// How far apart the walls are, inside: a body is a metre across, so a kick
/// crosses two.
pub const WIDE: i32 = 300;
/// How long the drop is, the takeoff's edge to the landing's.
pub const LONG: i32 = 1200;
/// How far under the takeoff the landing is: kicks lose height.
pub const DROP: i32 = 200;
/// How far the walls stand over the takeoff, and run down under it.
pub const ABOVE: i32 = 1200;
pub const BELOW: i32 = 1200;
/// How thick a wall is.
const THICK: i32 = 100;

/// **A gully's two walls**: from a takeoff edge whose middle is `at` (its
/// top, centimetres) along `dir` -- one of the four flat axes, as `(1, 0)`,
/// `(0, -1)` and so on -- for [`LONG`], [`WIDE`] apart across it.
pub const fn walls(at: [i32; 3], dir: (i32, i32), material: Material) -> [Solid; 2] {
    [side(at, dir, -1, material), side(at, dir, 1, material)]
}

/// One of a gully's walls: on the left of the run (`s` < 0) or the right.
const fn side(at: [i32; 3], dir: (i32, i32), s: i32, material: Material) -> Solid {
    let (lo, hi) = (at[1] - BELOW, at[1] + ABOVE);
    let h = WIDE / 2;
    // Across the run, the wall's two faces; along it, the drop.
    let (a, b) = if s < 0 {
        (-h - THICK, -h)
    } else {
        (h, h + THICK)
    };
    if dir.0 != 0 {
        let end = at[0] + dir.0 * LONG;
        let (x0, x1) = if dir.0 > 0 {
            (at[0], end)
        } else {
            (end, at[0])
        };
        Solid::cm([x0, lo, at[2] + a], [x1, hi, at[2] + b], material)
    } else {
        let end = at[2] + dir.1 * LONG;
        let (z0, z1) = if dir.1 > 0 {
            (at[2], end)
        } else {
            (end, at[2])
        };
        Solid::cm([at[0] + a, lo, z0], [at[0] + b, hi, z1], material)
    }
}

/// Where a gully's landing edge is: [`LONG`] along and [`DROP`] down from
/// its takeoff's.
pub const fn landing(at: [i32; 3], dir: (i32, i32)) -> [i32; 3] {
    [at[0] + dir.0 * LONG, at[1] - DROP, at[2] + dir.1 * LONG]
}
