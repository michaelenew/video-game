//! Shaped curves for the feel harness.
//!
//! A single number says *how long*; a curve says *what the time is spent
//! doing*. A structure rising out of the ground over a quarter second can be a
//! steady slide or a hold followed by an eruption, and those are different
//! moves to play against even though the duration is identical. The Oven was
//! always meant to grow this -- "individual numbers first, spline curves
//! later". This is later.
//!
//! **The shape is a cubic Bézier from (0,0) to (1,1) with two handles**, the
//! same four numbers as a CSS easing. Four numbers because that is the smallest
//! thing that can express a hold-then-burst *and* still be edited by dragging
//! two points, which is what the curve editor will do when it exists. Until it
//! does they are four ordinary knobs in the Oven, baked like every other number
//! — the data model is already right, only the widget is missing.
//!
//! Fixed point throughout, so a curve can drive simulation values and not only
//! decoration.

use crate::fixed::Fx;

/// A cubic Bézier easing: `(0,0) -> (x1,y1) -> (x2,y2) -> (1,1)`.
///
/// `x` is how far through the duration you are, `y` is how far through the
/// change. Handles near `x=1, y=0` hold at the start and then snap: that is a
/// telegraph followed by an eruption.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Curve {
    pub x1: Fx,
    pub y1: Fx,
    pub x2: Fx,
    pub y2: Fx,
}

impl Curve {
    /// A straight line. What you get when nothing has been shaped yet.
    pub const LINEAR: Curve = Curve {
        x1: Fx::from_raw(65536 / 3),
        y1: Fx::from_raw(65536 / 3),
        x2: Fx::from_raw(65536 * 2 / 3),
        y2: Fx::from_raw(65536 * 2 / 3),
    };

    /// The eased value at `x`, where `x` runs 0 to 1.
    ///
    /// The Bézier is parameterised by `t`, not by `x`, so this has to find the
    /// `t` whose `x` is the one asked for. **Bisection with a fixed iteration
    /// count**, rather than Newton: Newton converges faster but its step size
    /// depends on the curve, so the number of iterations -- and therefore the
    /// answer -- would vary with the handles. A fixed count is the same work
    /// and the same result on every machine, which is the only property that
    /// matters here.
    pub fn at(&self, x: Fx) -> Fx {
        let x = clamp01(x);
        let mut lo = Fx::ZERO;
        let mut hi = Fx::ONE;
        // Twenty halvings takes the bracket below one part in a million, which
        // is finer than 16.16 can represent.
        for _ in 0..20 {
            let mid = lo.add(hi).mul(HALF);
            if bezier(mid, self.x1, self.x2).raw() < x.raw() {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        clamp01(bezier(lo.add(hi).mul(HALF), self.y1, self.y2))
    }
}

const HALF: Fx = Fx::from_raw(1 << 15);

/// One axis of a cubic Bézier anchored at 0 and 1.
///
/// `3(1-t)²t·a + 3(1-t)t²·b + t³`, written with the squares shared so it stays
/// inside 16.16 without an intermediate overflow.
fn bezier(t: Fx, a: Fx, b: Fx) -> Fx {
    let inv = Fx::ONE.sub(t);
    let three = Fx::from_int(3);
    let term_a = three.mul(inv).mul(inv).mul(t).mul(a);
    let term_b = three.mul(inv).mul(t).mul(t).mul(b);
    let term_c = t.mul(t).mul(t);
    term_a.add(term_b).add(term_c)
}

fn clamp01(v: Fx) -> Fx {
    if v.raw() < 0 {
        Fx::ZERO
    } else if v.raw() > Fx::ONE.raw() {
        Fx::ONE
    } else {
        v
    }
}
