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

/// Distance along a ray to an upright cylinder standing on its base, or
/// `None` if the ray misses it.
///
/// `dir` is a **unit** vector, so the answer is in metres. Both end caps are
/// tested as well as the curved side: a cylinder without caps is a tube, and a
/// shot aimed down the axis of one would pass straight through it.
///
/// This is the shape almost everything in the game actually is -- a fighter, a
/// stone, a slab of a fire pillar -- so one function serves the aim resolver
/// and the Elementalist's beam alike. It replaced a flat, height-free version,
/// which is the whole reason a shot aimed above the horizon used to read the
/// terrain as though it had been fired along the floor.
///
/// A ray starting inside the cylinder reports the distance to where it leaves,
/// which is a hit -- point blank is still contact.
pub fn ray_hits_cylinder(from: V3, dir: V3, base: V3, radius: Fx, height: Fx) -> Option<Fx> {
    if height.raw() <= 0 || radius.raw() <= 0 {
        return None;
    }
    let top = base.y.add(height);
    let ox = from.x.sub(base.x);
    let oz = from.z.sub(base.z);
    let mut best: Option<Fx> = None;
    let mut keep = |d: Fx| {
        if d.raw() >= 0 && best.is_none_or(|b| d.raw() < b.raw()) {
            best = Some(d);
        }
    };

    // The curved side. `a` is zero looking straight up or down, where there is
    // no side to hit and the caps are the whole answer.
    let a = dir.x.mul(dir.x).add(dir.z.mul(dir.z));
    if a.raw() > 0 {
        let half_b = ox.mul(dir.x).add(oz.mul(dir.z));
        let c = ox.mul(ox).add(oz.mul(oz)).sub(radius.mul(radius));
        let disc = half_b.mul(half_b).sub(a.mul(c));
        if disc.raw() >= 0 {
            let root = disc.sqrt();
            for d in [half_b.neg().sub(root).div(a), half_b.neg().add(root).div(a)] {
                let y = from.y.add(dir.y.mul(d));
                if y.raw() >= base.y.raw() && y.raw() <= top.raw() {
                    keep(d);
                }
            }
        }
    }

    // The caps.
    if dir.y.raw() != 0 {
        for face in [base.y, top] {
            let d = face.sub(from.y).div(dir.y);
            if d.raw() < 0 {
                continue;
            }
            let x = ox.add(dir.x.mul(d));
            let z = oz.add(dir.z.mul(d));
            if x.mul(x).add(z.mul(z)).raw() <= radius.mul(radius).raw() {
                keep(d);
            }
        }
    }
    best
}

/// Distance along a ray to an axis-aligned box, or `None` if it misses.
///
/// Slab method: the ray is inside the box over the intersection of the three
/// per-axis intervals it is inside each slab for. `dir` is a unit vector, so
/// the answer is in metres.
pub fn ray_hits_box(from: V3, dir: V3, min: V3, max: V3) -> Option<Fx> {
    let mut near = Fx::ZERO;
    let mut far = Fx::MAX;
    for axis in 0..3 {
        let o = component(from, axis);
        let d = component(dir, axis);
        let lo = component(min, axis);
        let hi = component(max, axis);
        if d.raw() == 0 {
            // Parallel to this pair of faces: either always between them or
            // never. Checked by hand because dividing by zero saturates, which
            // would read as "always".
            if o.raw() < lo.raw() || o.raw() > hi.raw() {
                return None;
            }
            continue;
        }
        let a = lo.sub(o).div(d);
        let b = hi.sub(o).div(d);
        let (enter, leave) = if a.raw() <= b.raw() { (a, b) } else { (b, a) };
        near = near.max(enter);
        far = far.min(leave);
        if near.raw() > far.raw() {
            return None;
        }
    }
    Some(near)
}

fn component(v: V3, axis: usize) -> Fx {
    match axis {
        0 => v.x,
        1 => v.y,
        _ => v.z,
    }
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
