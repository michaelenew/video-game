//! **King of the Hill maps** (2026-10-21, `crate::versus`,
//! `docs/design/versus.md`): one capture point in the middle, a side's
//! start at each end, and three or four ways from one to the other.
//!
//! Every map is **turned half a turn about its middle** -- (x, z) to
//! (-x, -z) -- so each side has exactly what the other has: a table here is
//! the middle's own solids and one half's, and [`whole`] makes the other
//! half by turning it. A ledge west of the point for the west side is a
//! ledge east of it for the east side.
//!
//! What each map is built to have, from Team Fortress 2's King of the Hill
//! maps and nothing copied from any one of them:
//!
//! - **High ground over the point**, close enough to fight down from and
//!   reached by steps a jump each, so the point is contested from above.
//! - **A way in nobody sees**: a tunnel or a covered passage, the one kind
//!   of confined space on the map, for getting onto the point from where
//!   the defenders are not looking.
//! - **Open air everywhere else**: from either start to the point and to
//!   every high place there is a way that never goes under a roof, through
//!   at most one or two narrow places.
//! - **No buildings**: the cover is rocks, blocks, stones and banks.
//!
//! Every climb on a route is at most a jump the shortest-jumping class makes
//! with room to spare (2.7 m at the top of a held jump), so nobody is shut
//! out of high ground in a team game -- a different thing from a movement
//! section, which these are not. The edge is a rim, as every fight's is.

use super::{Material, Solid};
use crate::fixed::Fx;
use crate::math::V3;

/// A solid turned half a turn about the middle: (x, z) to (-x, -z). A round
/// solid is round still, and a turned box lies along the same line.
pub(super) const fn turned(s: Solid) -> Solid {
    Solid {
        min: V3::new(s.max.x.neg(), s.min.y, s.max.z.neg()),
        max: V3::new(s.min.x.neg(), s.max.y, s.min.z.neg()),
        material: s.material,
        shape: s.shape,
    }
}

/// **A whole map from its middle and one half**: the middle's solids, the
/// half, and the half turned. `N` must be `C + 2 H`.
pub(super) const fn whole<const C: usize, const H: usize, const N: usize>(
    centre: [Solid; C],
    half: [Solid; H],
) -> [Solid; N] {
    assert!(N == C + 2 * H, "a map is its middle and two halves");
    let mut out = [half[0]; N];
    let mut i = 0;
    while i < C {
        out[i] = centre[i];
        i += 1;
    }
    let mut k = 0;
    while k < H {
        out[C + k] = half[k];
        out[C + H + k] = turned(half[k]);
        k += 1;
    }
    out
}

/// A round solid from centimetres: centred on (x, z), from `y0` to `y1`,
/// `bottom` and `top` across (radii).
pub(super) const fn round(
    x: i32,
    z: i32,
    y: (i32, i32),
    r: (i32, i32),
    material: Material,
) -> Solid {
    let (bottom, top) = r;
    let big = if bottom > top { bottom } else { top };
    Solid {
        min: V3::new(super::cm(x - big), super::cm(y.0), super::cm(z - big)),
        max: V3::new(super::cm(x + big), super::cm(y.1), super::cm(z + big)),
        material,
        shape: super::Shape::Round {
            bottom: super::cm(bottom),
            top: super::cm(top),
            hollow: Fx::ZERO,
        },
    }
}

/// **A slab lying along a direction**: a box `half.0` long either way along
/// (dx, dz) and `half.1` either way across it, centred on (x, z), from `y.0`
/// to `y.1` (centimetres) -- a fallen block, a broken arch.
pub(super) const fn slab(
    x: i32,
    z: i32,
    y: (i32, i32),
    half: (i32, i32),
    along: (i32, i32),
    material: Material,
) -> Solid {
    let d = V3::new(Fx::from_int(along.0), Fx::ZERO, Fx::from_int(along.1)).normalized();
    let (c, s) = (d.x, d.z);
    let (long, wide) = (super::cm(half.0), super::cm(half.1));
    let ex = long.mul(c.abs()).add(wide.mul(s.abs()));
    let ez = long.mul(s.abs()).add(wide.mul(c.abs()));
    let (cx, cz) = (super::cm(x), super::cm(z));
    Solid {
        min: V3::new(cx.sub(ex), super::cm(y.0), cz.sub(ez)),
        max: V3::new(cx.add(ex), super::cm(y.1), cz.add(ez)),
        material,
        shape: super::Shape::Turned {
            half: long,
            half_wide: wide,
            cos: c,
            sin: s,
        },
    }
}

pub mod delta;
pub mod glacier;
pub mod gorge;
pub mod henge;
pub mod quarry;
pub mod sizes;

pub use delta::{DELTA, DELTA_STAGE};
pub use glacier::{GLACIER, GLACIER_STAGE};
pub use gorge::{GORGE, GORGE_STAGE};
pub use henge::{HENGE, HENGE_STAGE};
pub use quarry::{QUARRY, QUARRY_STAGE};
