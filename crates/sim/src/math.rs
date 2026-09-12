//! Vector math over fixed point.

use crate::fixed::{Fx, cos_turns, sin_turns};

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Hash)]
pub struct V3 {
    pub x: Fx,
    pub y: Fx,
    pub z: Fx,
}

impl V3 {
    pub const ZERO: V3 = V3 {
        x: Fx::ZERO,
        y: Fx::ZERO,
        z: Fx::ZERO,
    };

    pub const fn new(x: Fx, y: Fx, z: Fx) -> V3 {
        V3 { x, y, z }
    }

    pub const fn add(self, o: V3) -> V3 {
        V3::new(self.x.add(o.x), self.y.add(o.y), self.z.add(o.z))
    }

    pub const fn sub(self, o: V3) -> V3 {
        V3::new(self.x.sub(o.x), self.y.sub(o.y), self.z.sub(o.z))
    }

    pub const fn scale(self, s: Fx) -> V3 {
        V3::new(self.x.mul(s), self.y.mul(s), self.z.mul(s))
    }

    pub const fn dot(self, o: V3) -> Fx {
        self.x.mul(o.x).add(self.y.mul(o.y)).add(self.z.mul(o.z))
    }

    pub const fn len_sq(self) -> Fx {
        self.dot(self)
    }

    pub const fn len(self) -> Fx {
        self.len_sq().sqrt()
    }

    /// Zero-length input returns zero rather than NaN-equivalent nonsense.
    pub const fn normalized(self) -> V3 {
        let l = self.len();
        if l.raw() == 0 {
            V3::ZERO
        } else {
            V3::new(self.x.div(l), self.y.div(l), self.z.div(l))
        }
    }

    /// Unit vector in the horizontal plane for an angle in turns.
    ///
    /// The convention for the whole game: an angle of zero looks down positive
    /// X, and increasing angle swings toward positive Z. The renderer places
    /// the camera with the same convention, which is what makes "forward" mean
    /// the same thing to the simulation and to the player's eyes.
    pub fn from_turns(turns: Fx) -> V3 {
        V3::new(cos_turns(turns), Fx::ZERO, sin_turns(turns))
    }

    /// Horizontal plane only. Most gameplay distance checks want this.
    pub const fn flat_len(self) -> Fx {
        self.x.mul(self.x).add(self.z.mul(self.z)).sqrt()
    }
}

/// Where a shot from `from` toward `to` passes closest to `centre`, and
/// whether that approach comes within `radius` of it -- flat, like every
/// other hit test in the game.
///
/// Returns the distance along the segment to that closest point, clamped to
/// the segment itself, so a caller comparing two different things a shot
/// might be aimed through can tell which one it reaches first.
pub fn ray_hits_flat(from: V3, to: V3, centre: V3, radius: Fx) -> Option<Fx> {
    let seg = V3::new(to.x.sub(from.x), Fx::ZERO, to.z.sub(from.z));
    let len_sq = seg.dot(seg);
    if len_sq.raw() <= 0 {
        return None;
    }
    let to_centre = V3::new(centre.x.sub(from.x), Fx::ZERO, centre.z.sub(from.z));
    let t = to_centre.dot(seg).div(len_sq).clamp(Fx::ZERO, Fx::ONE);
    let closest = V3::new(from.x.add(seg.x.mul(t)), Fx::ZERO, from.z.add(seg.z.mul(t)));
    let dist = V3::new(centre.x.sub(closest.x), Fx::ZERO, centre.z.sub(closest.z)).flat_len();
    if dist.raw() > radius.raw() {
        return None;
    }
    Some(seg.flat_len().mul(t))
}

/// The closest distance between two line segments.
///
/// This is what a capsule hit test is: an attack is a line with a thickness and
/// a body is a line with a thickness, and they touch when the gap between the
/// two lines is less than the two thicknesses added together. Everything the
/// Champion swings is tested with it, which is what lets a hammer brought down
/// on somebody miss a fighter standing behind them and catch one crouching in
/// front.
///
/// The standard clamped solve, and worth reading once because the clamping is
/// the whole of it. Two infinite lines have exactly one closest pair, found by
/// solving a two-by-two system; two *segments* may have that pair off either
/// end, in which case the answer is on an endpoint and the remaining parameter
/// has to be re-solved against it. Degenerate cases -- a zero-length segment,
/// two parallel ones -- fall out of the same branches rather than needing their
/// own.
///
/// Fixed point throughout. The products here are bounded by a four-metre weapon
/// and a twenty-eight metre arena, which is comfortably inside what 16.16 holds,
/// and a division by a near-zero denominator saturates into a parameter that is
/// then clamped -- so the parallel case is safe rather than special.
pub fn segment_gap(a0: V3, a1: V3, b0: V3, b1: V3) -> Fx {
    let d1 = a1.sub(a0);
    let d2 = b1.sub(b0);
    let r = a0.sub(b0);
    let a = d1.dot(d1);
    let e = d2.dot(d2);
    let f = d2.dot(r);
    let tiny = Fx::from_raw(4);
    let unit = |v: Fx| v.clamp(Fx::ZERO, Fx::ONE);

    let (s, t) = if a.raw() <= tiny.raw() && e.raw() <= tiny.raw() {
        (Fx::ZERO, Fx::ZERO)
    } else if a.raw() <= tiny.raw() {
        (Fx::ZERO, unit(f.div(e)))
    } else {
        let c = d1.dot(r);
        if e.raw() <= tiny.raw() {
            (unit(c.neg().div(a)), Fx::ZERO)
        } else {
            let b = d1.dot(d2);
            let denom = a.mul(e).sub(b.mul(b));
            let s = if denom.raw() != 0 {
                unit(b.mul(f).sub(c.mul(e)).div(denom))
            } else {
                Fx::ZERO
            };
            let t = b.mul(s).add(f).div(e);
            if t.raw() < 0 {
                (unit(c.neg().div(a)), Fx::ZERO)
            } else if t.raw() > Fx::ONE.raw() {
                (unit(b.sub(c).div(a)), Fx::ONE)
            } else {
                (s, t)
            }
        }
    };
    big_len(a0.add(d1.scale(s)).sub(b0.add(d2.scale(t))))
}

/// Linear blend of two points.
pub const fn lerp3(from: V3, to: V3, at: Fx) -> V3 {
    V3::new(
        lerp(from.x, to.x, at),
        lerp(from.y, to.y, at),
        lerp(from.z, to.z, at),
    )
}

/// Linear blend. `at` outside 0..1 extrapolates, which is occasionally what you
/// want and never a surprise.
pub const fn lerp(from: Fx, to: Fx, at: Fx) -> Fx {
    from.add(to.sub(from).mul(at))
}

/// Smoothstep: `3t^2 - 2t^3`, clamped to 0..1.
///
/// Arithmetic rather than a knob. The 3 and the 2 are the definition of the
/// curve in the same way the 3 in a cubic Bezier is -- changing them would not
/// change how anything feels, it would stop it being a smoothstep.
pub fn smoothstep(t: Fx) -> Fx {
    let t = t.clamp(Fx::ZERO, Fx::ONE);
    let sq = t.mul(t);
    let two = Fx::ONE.add(Fx::ONE);
    let three = two.add(Fx::ONE);
    three.mul(sq).sub(two.mul(sq).mul(t))
}

/// A triangle that rises 0 -> 1 -> 0 across 0..1. The shape of a windup
/// followed by a return.
pub fn arch(t: Fx) -> Fx {
    let t = t.clamp(Fx::ZERO, Fx::ONE);
    let two = Fx::ONE.add(Fx::ONE);
    if t.raw() * 2 <= Fx::ONE.raw() {
        smoothstep(t.mul(two))
    } else {
        smoothstep(Fx::ONE.sub(t).mul(two))
    }
}

/// Wrap an angle in turns into the half-open range (-1/2, 1/2].
///
/// Exact, and free: `Fx` is 16.16, so the fractional bits of the raw integer
/// *are* the fraction of a turn. This is the arithmetic that makes integer
/// angles worth having.
pub fn wrap_turns(turns: Fx) -> Fx {
    let frac = (turns.raw() as u32) & 0xFFFF;
    if frac >= 0x8000 {
        Fx::from_raw(frac as i32 - 0x1_0000)
    } else {
        Fx::from_raw(frac as i32)
    }
}

/// The angle of a direction, in turns, matching `V3::from_turns`: zero looks
/// down positive X and the angle increases toward positive Z.
///
/// A rational approximation rather than a lookup table, because the table would
/// need to be much finer than the sine table to be worth having.
///
/// The shape is the standard octant reduction. Fold the input into one eighth
/// of the circle by forming `r = (x - |z|) / (x + |z|)`; that `r` is exactly
/// `tan(pi/4 - phi)`, so recovering the angle is `1/8 - atan(r)/2pi` in turns,
/// and the negative-x branch is the same thing offset to `3/8`. What is left is
/// `atan` on `[-1, 1]`, fitted by an odd cubic-in-`r-squared`.
///
/// Three terms rather than two: two is the version that circulates, and it is
/// half a degree out at its worst, which is enough to make a creature's aim
/// visibly miss. Three costs one more multiply and brings the worst case to
/// about **0.0002 of a turn** -- a thirteenth of a degree.
///
/// Every coefficient is an exact 16.16 constant and every step is integer
/// arithmetic, so there is no rounding for two machines to disagree about.
pub fn atan2_turns(z: Fx, x: Fx) -> Fx {
    if z.raw() == 0 && x.raw() == 0 {
        return Fx::ZERO;
    }
    let az = z.abs();
    let eighth = Fx::from_raw(1 << 13);
    let three_eighths = Fx::from_raw(3 << 13);
    let (r, base) = if x.raw() >= 0 {
        (x.sub(az).div(x.add(az)), eighth)
    } else {
        (x.add(az).div(az.sub(x)), three_eighths)
    };
    // atan(r) / 2pi, as a1*r + a3*r^3 + a5*r^5, in Horner form.
    let a1 = Fx::from_raw(10388);
    let a3 = Fx::from_raw(-3048);
    let a5 = Fx::from_raw(866);
    let sq = r.mul(r);
    let angle = base.sub(r.mul(a1.add(sq.mul(a3.add(sq.mul(a5))))));
    if z.raw() < 0 { angle.neg() } else { angle }
}

/// Length of a vector whose components are too large to square in 16.16.
///
/// `V3::len` squares its components first, and a squared 16.16 value saturates
/// just past 181. That is ample for a position in a twenty-eight metre arena
/// and useless for an **acceleration**, which is where the creature's buck
/// lives: the shake produces several hundred metres per second squared, and
/// `len` reports 181 for all of them. Silently, because saturation is not an
/// error.
///
/// Squaring in `i64` instead costs one integer square root and is exact over
/// the whole range 16.16 can hold.
pub fn big_len(v: V3) -> Fx {
    let (x, y, z) = (v.x.raw() as i64, v.y.raw() as i64, v.z.raw() as i64);
    Fx::from_raw(isqrt(x * x + y * y + z * z))
}

/// Integer square root of a 64-bit value, saturating into `i32`.
///
/// A fixed iteration count rather than "until it converges": a value that
/// depends on how hard it was to compute is not something two machines can be
/// relied on to agree about.
fn isqrt(n: i64) -> i32 {
    if n <= 0 {
        return 0;
    }
    let mut guess: i64 = 1 << 31;
    let mut i = 0;
    while i < 40 {
        guess = (guess + n / guess) >> 1;
        i += 1;
    }
    if guess > i32::MAX as i64 {
        i32::MAX
    } else {
        guess as i32
    }
}
