//! **The ground behind a fight's edge**: a bank of earth and rock that rises
//! off the outside of the walls, and hills and mountains past it.
//!
//! A fight was a floor with boxes round it: low walls to put your back to,
//! and behind them a six-metre hedge, a thicket, a cliff -- a box standing on
//! a plane that went on forever, there to say *the world ends here*. The
//! valley stopped drawing its edges that way (`crate::valley::land`), and so
//! do the fights: what keeps you in now is **the ground itself**, the same
//! rule as a mountainside -- land too steep to walk up is a wall
//! (`arena::Terrain`, `tuning::terrain_steepest`).
//!
//! A rim is a rectangle -- the outside faces of the arena's walls -- and
//! everything **inside** it is exactly the floor it always was, relief and
//! all: nothing a fight was tuned on moves. **Outside** it the ground starts
//! at the rim's foot, the height of the wall on that side, so stepping off a
//! wall's top outward is stepping onto the bank rather than into a crack
//! between them; climbs a bank too steep to stand on, [`Rim::bank`] high;
//! rounds over at its crest; and goes on as the valley's land does
//! (`land::rise`), foothills and then a mountain. Its corners are round,
//! because the distance from a rectangle is. A little noise lies over it,
//! growing from nothing at the foot, so the line you collide with is the
//! line drawn and nothing above it is a plane.
//!
//! On the valley's map a room's rim is the land's (`land::Pad::rim`), held up
//! against the valley round it except where a path comes through.

use crate::fixed::Fx;
use crate::math::V3;
use crate::valley::land;

/// How the ground behind an arena's edge rises.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rim {
    /// The rectangle it rises from, in centimetres: x from `lo.0` to `hi.0`,
    /// z from `lo.1` to `hi.1`. Inside it, the arena's own floor.
    pub lo: (i32, i32),
    pub hi: (i32, i32),
    /// The ground's height at the foot of each side, in centimetres: south
    /// (`-z`), north (`+z`), west (`-x`), east (`+x`). The top of the wall
    /// on that side, so the bank starts where the wall's top ends.
    pub foot: [i32; 4],
    /// How high the bank climbs above its foot before it rounds over, in
    /// centimetres. All of it too steep to walk up but its crest.
    pub bank: i32,
    /// **A drop rather than a rise**: the Cliffs, with the sky on three
    /// sides. To the simulation it is a rim like any other -- nobody stands
    /// past the lip -- and the renderer draws it falling away past its crest.
    pub drop: bool,
}

/// How steep the bank is where it leaves its foot: more than twice what can
/// be walked up, easing to level at its crest.
const SLOPE: Fx = Fx::from_int(3);

/// How far out the noise has grown to its whole: nothing at the foot.
const GROW: Fx = Fx::from_int(12);

/// How far behind its crest the land a rim makes on a map is held up
/// against the valley round it (`land::Land`).
pub const HELD: Fx = Fx::from_int(4);

impl Rim {
    /// A rim round the rectangle x `x.0..x.1`, z `z.0..z.1` (centimetres),
    /// with each side's foot (south, north, west, east) and the bank's height.
    pub const fn new(x: (i32, i32), z: (i32, i32), foot: [i32; 4], bank: i32) -> Rim {
        Rim {
            lo: (x.0, z.0),
            hi: (x.1, z.1),
            foot,
            bank,
            drop: false,
        }
    }

    /// The same rim, drawn as a drop past its crest.
    pub const fn falling(self) -> Rim {
        Rim { drop: true, ..self }
    }

    /// The rectangle, in metres: lo and hi (x, z).
    pub fn rect(&self) -> ((Fx, Fx), (Fx, Fx)) {
        (
            (super::cm(self.lo.0), super::cm(self.lo.1)),
            (super::cm(self.hi.0), super::cm(self.hi.1)),
        )
    }

    /// Is a point (in the arena's coordinates) outside the rectangle -- on
    /// the rim's ground rather than the arena's floor?
    pub fn outside(&self, x: Fx, z: Fx) -> bool {
        let (lo, hi) = self.rect();
        x.raw() < lo.0.raw() || x.raw() > hi.0.raw() || z.raw() < lo.1.raw() || z.raw() > hi.1.raw()
    }

    /// How far across the bank runs before its crest, in metres, at its
    /// highest.
    pub fn run(&self) -> Fx {
        super::cm(self.bank).mul(Fx::from_int(2)).div(SLOPE)
    }

    /// **How high the bank is here**: between seven tenths of [`Rim::bank`]
    /// and the whole of it, wandering along the rim, so its crest is a
    /// skyline and not a ruled line.
    fn bank_at(&self, x: Fx, z: Fx) -> Fx {
        let v = land::value(x, z, 23, 7);
        let k = Fx::ratio(17, 20).add(v.mul(Fx::ratio(3, 20)));
        super::cm(self.bank).mul(k)
    }

    /// How far out a point is, and the height of the foot it is out from.
    fn out(&self, x: Fx, z: Fx) -> (Fx, Fx) {
        let (lo, hi) = self.rect();
        let w = lo.0.sub(x).max(Fx::ZERO);
        let e = x.sub(hi.0).max(Fx::ZERO);
        let s = lo.1.sub(z).max(Fx::ZERO);
        let n = z.sub(hi.1).max(Fx::ZERO);
        let (dx, dz) = (w.max(e), s.max(n));
        let d = crate::math::wide_len(V3::new(dx, Fx::ZERO, dz));
        let fx = super::cm(if w.raw() > 0 {
            self.foot[2]
        } else {
            self.foot[3]
        });
        let fz = super::cm(if s.raw() > 0 {
            self.foot[0]
        } else {
            self.foot[1]
        });
        // Round a corner, each side's foot by how far out along it the
        // point is: the side's own foot along the side, an even mix on the
        // diagonal.
        let foot = if dx.raw() == 0 {
            fz
        } else if dz.raw() == 0 {
            fx
        } else {
            let t = dz.div(dx.add(dz));
            crate::math::lerp(fx, fz, t)
        };
        (d, foot)
    }

    /// How far out a point is, and the height of its foot, if it is outside.
    pub fn out_of(&self, x: Fx, z: Fx) -> Option<(Fx, Fx)> {
        self.outside(x, z).then(|| self.out(x, z))
    }

    /// The height of the foot a point outside is out from: its side's, or a
    /// corner's mix of two.
    pub fn foot_at(&self, x: Fx, z: Fx) -> Fx {
        self.out(x, z).1
    }

    /// **The ground's height at a point** in the arena's coordinates, if the
    /// point is outside the rectangle: the foot, the bank, the land past it.
    pub fn height(&self, x: Fx, z: Fx) -> Option<Fx> {
        if !self.outside(x, z) {
            return None;
        }
        let (d, foot) = self.out(x, z);
        Some(foot.add(self.rise(d, x, z)))
    }

    /// How far above its foot the rim is `d` metres out, at (x, z): the
    /// bank, rounding over at its crest, then the land's own rise; with the
    /// noise that grows with distance laid over it.
    pub fn rise(&self, d: Fx, x: Fx, z: Fx) -> Fx {
        let bank = self.bank_at(x, z);
        let run = bank.mul(Fx::from_int(2)).div(SLOPE);
        let base = if d.raw() < run.raw() {
            // `bank · t(2 - t)`: as steep as `SLOPE` at the foot, level at
            // the crest.
            let t = d.div(run);
            bank.mul(t.mul(Fx::from_int(2).sub(t)))
        } else {
            bank.add(land::rise(d.sub(run)))
        };
        let grown = d.min(GROW).div(GROW);
        let amp = Fx::ratio(3, 2)
            .add(base.min(Fx::from_int(40)).mul(Fx::ratio(1, 10)))
            .mul(grown);
        // Lumps a few metres across over the face: rock that juts and
        // falls back. Grown from nothing at the foot too, and small enough
        // that the bank stays too steep to climb below its crest.
        let lumps = land::value(x, z, 5, 11)
            .abs()
            .mul(Fx::ratio(7, 10))
            .mul(grown);
        base.add(land::noise(x, z).mul(amp).abs()).add(lumps)
    }
}
