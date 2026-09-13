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

/// `t(2 - t)`: leaves at full speed and arrives at rest.
///
/// The shape of anything thrown rather than accelerated -- a shadow shot out to
/// the spot it was sent to, a blade erupting. Its mirror, `1 - ease_out(t)`, is
/// `(1 - t)` squared, which is the same motion run backwards: away at speed,
/// settling as it arrives.
///
/// Arithmetic rather than a knob, in the same sense [`smoothstep`] is: the 2 is
/// what makes the gradient at zero equal to one and at one equal to zero, and
/// any other value would stop it being this curve.
pub fn ease_out(t: Fx) -> Fx {
    let t = t.clamp(Fx::ZERO, Fx::ONE);
    let two = Fx::ONE.add(Fx::ONE);
    t.mul(two.sub(t))
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

/// A section of a ring lying flat: a chunk of a torus, in the plane of the
/// floor.
///
/// The one volume in the game that is not a capsule. It exists because a wing
/// is not shaped like a weapon -- it wraps around the thing that threw it -- and
/// approximating one with a straight line either misses the inside of the curve
/// or claims the outside of it. See `moves::Shape::Wing`.
///
/// Angles are in turns, measured the way [`atan2_turns`] and `V3::from_turns`
/// measure: zero looks down positive X. `from` and `to` are the two edges and
/// the section is the shorter way between them, so which is which does not
/// matter.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sector {
    /// The middle of the ring, at the height the section lies at.
    pub at: V3,
    pub inner: Fx,
    pub outer: Fx,
    pub from: Fx,
    pub to: Fx,
}

impl Sector {
    /// The angle halfway between the two edges.
    pub fn mid(&self) -> Fx {
        self.from
            .add(wrap_turns(self.to.sub(self.from)).div(Fx::from_int(2)))
    }

    /// Half of how wide the section is, in turns. Never negative.
    pub fn half_span(&self) -> Fx {
        wrap_turns(self.to.sub(self.from))
            .abs()
            .div(Fx::from_int(2))
    }

    /// A point on the section: `out` of the way from the inner arc to the outer
    /// one, `along` of the way from one edge to the other.
    pub fn point(&self, out: Fx, along: Fx) -> V3 {
        let angle = self.from.add(wrap_turns(self.to.sub(self.from)).mul(along));
        let r = self.inner.add(self.outer.sub(self.inner).mul(out));
        V3::new(
            self.at.x.add(crate::fixed::cos_turns(angle).mul(r)),
            self.at.y,
            self.at.z.add(crate::fixed::sin_turns(angle).mul(r)),
        )
    }

    /// Does an upright body touch it?
    ///
    /// `foot` is where the body stands, `height` how tall, and `slack` how far
    /// outside the section's own surface it still reaches -- the body's radius
    /// plus the attack's, which is the same threshold every other volume in the
    /// game is tested with.
    ///
    /// Three tests, cheapest first: the height, then the radius, then the
    /// bearing. The bearing is the only one that needs an angle, and it is
    /// widened by however much of a turn `slack` covers at that distance --
    /// which is what stops a body being missed by standing just off the end of
    /// a section that plainly reaches it.
    pub fn touches(&self, foot: V3, height: Fx, slack: Fx) -> bool {
        if self.at.y.raw() < foot.y.sub(slack).raw()
            || self.at.y.raw() > foot.y.add(height).add(slack).raw()
        {
            return false;
        }
        let delta = V3::new(foot.x.sub(self.at.x), Fx::ZERO, foot.z.sub(self.at.z));
        let d = delta.flat_len();
        if d.raw() > self.outer.add(slack).raw() || d.add(slack).raw() < self.inner.raw() {
            return false;
        }
        // Standing on the middle of the ring: every bearing is within reach, so
        // there is no angle to compare and the arithmetic below would divide by
        // nearly nothing.
        if d.raw() <= slack.raw() {
            return true;
        }
        let bearing = atan2_turns(delta.z, delta.x);
        // How much of a turn `slack` is worth this far out: the half-angle a
        // tolerance of `slack` subtends at radius `d`, which is `asin(slack/d)`
        // and not `slack/d` -- the small-angle version is half again too
        // generous by the time the ratio is two thirds, and this volume is
        // routinely tested against bodies standing near its own middle.
        //
        // `asin(x)` as `atan2(x, sqrt(1 - x*x))`, which is exact here because
        // the early return above guarantees `x < 1`.
        let ratio = slack.div(d);
        let widen = atan2_turns(ratio, Fx::ONE.sub(ratio.mul(ratio)).sqrt());
        wrap_turns(bearing.sub(self.mid())).abs().raw() <= self.half_span().add(widen).raw()
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

// ---------------------------------------------------------------------------
// Rotations
// ---------------------------------------------------------------------------

/// A 3x3 rotation, in fixed point.
///
/// The creature is a skeleton now rather than one rigid box with a yaw and a
/// pitch, and a chain of bones cannot be composed out of two angles: a neck
/// that is bent *and* turned is a rotation whose axis is neither. So the
/// creature's bones carry a matrix, built from the same sine table everything
/// else uses.
///
/// Rows are the rotated basis vectors, so `apply` is three dot products and
/// `inverse` is the transpose. Composition accumulates a little error down a
/// chain -- a 16.16 product loses half a bit -- but it accumulates *identically*
/// on every machine, which is the only property determinism asks for. Over the
/// six bones of the longest chain here it is well under a millimetre.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Mat3 {
    /// `r[i]` is the image of body axis `i`.
    pub r: [V3; 3],
}

impl Mat3 {
    pub const IDENTITY: Mat3 = Mat3 {
        r: [
            V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
            V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO),
            V3::new(Fx::ZERO, Fx::ZERO, Fx::ONE),
        ],
    };

    /// The rotation a joint angle triple names.
    ///
    /// **Order is pitch, then yaw, then roll**, each about the *parent's* axes,
    /// which is the order a limb reads in: swing it forward, carry it out to
    /// the side, then roll along its own length. Written out rather than
    /// composed from three matrices because the zeros are most of it.
    ///
    /// `pitch` is nose-up about `+z`, `yaw` is toward `+z` about `+y`, `roll`
    /// is about `+x`. All three in turns.
    pub fn from_angles(pitch: Fx, yaw: Fx, roll: Fx) -> Mat3 {
        let (cp, sp) = (cos_turns(pitch), sin_turns(pitch));
        let (cy, sy) = (cos_turns(yaw), sin_turns(yaw));
        let (cr, sr) = (cos_turns(roll), sin_turns(roll));

        // Pitch about z: x -> (cp, sp, 0), y -> (-sp, cp, 0), z -> z.
        // Yaw about y:   x -> (cy, 0, sy), z -> (-sy, 0, cy), y -> y.
        // Roll about x:  y -> (0, cr, sr), z -> (0, -sr, cr), x -> x.
        let x = V3::new(cp.mul(cy), sp, cp.mul(sy));
        let y0 = V3::new(sp.neg().mul(cy), cp, sp.neg().mul(sy));
        let z0 = V3::new(sy.neg(), Fx::ZERO, cy);
        Mat3 {
            r: [
                x,
                y0.scale(cr).add(z0.scale(sr)),
                y0.scale(sr.neg()).add(z0.scale(cr)),
            ],
        }
    }

    /// Yaw alone, which is what a creature standing on flat ground is doing.
    pub fn from_yaw(yaw: Fx) -> Mat3 {
        Mat3::from_angles(Fx::ZERO, yaw, Fx::ZERO)
    }

    /// A local vector, in the parent's frame.
    pub const fn apply(&self, v: V3) -> V3 {
        V3::new(
            self.r[0]
                .x
                .mul(v.x)
                .add(self.r[1].x.mul(v.y))
                .add(self.r[2].x.mul(v.z)),
            self.r[0]
                .y
                .mul(v.x)
                .add(self.r[1].y.mul(v.y))
                .add(self.r[2].y.mul(v.z)),
            self.r[0]
                .z
                .mul(v.x)
                .add(self.r[1].z.mul(v.y))
                .add(self.r[2].z.mul(v.z)),
        )
    }

    /// The inverse, which for a rotation is the transpose -- so a world vector
    /// comes back into the bone's own frame as three dot products.
    pub const fn unapply(&self, v: V3) -> V3 {
        V3::new(self.r[0].dot(v), self.r[1].dot(v), self.r[2].dot(v))
    }

    /// `self` then `child`: the child's local rotation carried by this one.
    pub const fn then(&self, child: Mat3) -> Mat3 {
        Mat3 {
            r: [
                self.apply(child.r[0]),
                self.apply(child.r[1]),
                self.apply(child.r[2]),
            ],
        }
    }
}

/// Wrap a fraction into `0..1`. What a looping clip's phase needs: the wheel
/// has no ends, so an index past the last sample comes back round to the first.
pub fn wrap_unit(v: Fx) -> Fx {
    let raw = v.raw();
    let one = Fx::ONE.raw();
    Fx::from_raw(raw.rem_euclid(one))
}
