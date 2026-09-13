//! How the time between two poses is spent.
//!
//! A key says *where* the body is; an ease says *how it gets there*. Two poses
//! eight frames apart can be a steady slide, a hold followed by a snap, or a
//! pull-back-then-go -- and those are three completely different moves to play
//! against even though the frame data is identical. This is the knob that makes
//! that choice, and it is the one a person spends the most time on.
//!
//! The shape is a **cubic Bézier from (0,0) to (1,1) with two handles**, the
//! same four numbers as a CSS easing, and the same four numbers the Oven
//! already uses for its shaped curves. Four because it is the smallest thing
//! that can express hold-then-burst *and* be edited by dragging two points,
//! which is exactly what the hub's curve widget does.
//!
//! Unlike the simulation's curve, the vertical handles are **not clamped**.
//! A handle below zero pulls the motion backwards before it goes -- anticipation
//! -- and one above one carries it past the target and back. Both are things an
//! animator reaches for constantly, and neither can hurt anything here: this
//! runs offline, and the solver clamps the result to what a joint can do.

/// `(0,0) -> (x1,y1) -> (x2,y2) -> (1,1)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ease {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl Default for Ease {
    fn default() -> Self {
        Ease::SMOOTH
    }
}

impl Ease {
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Ease {
        Ease { x1, y1, x2, y2 }
    }

    /// Constant speed. Honest, and almost always the wrong choice: nothing
    /// physical starts and stops instantaneously.
    pub const LINEAR: Ease = Ease::new(1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0);

    /// Eases in and out. The safe default, and what you get if you say nothing.
    pub const SMOOTH: Ease = Ease::new(0.42, 0.0, 0.58, 1.0);

    /// Starts slow, ends fast. A wind-up gathering itself.
    pub const IN: Ease = Ease::new(0.7, 0.0, 0.9, 0.55);

    /// Starts fast, ends slow. A limb arriving and settling.
    pub const OUT: Ease = Ease::new(0.1, 0.45, 0.2, 1.0);

    /// Almost nothing happens, and then it all happens. The telegraph: the
    /// opponent gets a long look at a pose that is not moving, then it moves.
    pub const SNAP: Ease = Ease::new(0.85, 0.02, 0.95, 0.5);

    /// All of it immediately, then a long tail coming to rest. What a strike's
    /// contact frame wants.
    pub const STRIKE: Ease = Ease::new(0.05, 0.75, 0.15, 1.0);

    /// Pulls *away* from the target before going. Anticipation, in one number
    /// instead of an extra key.
    pub const ANTICIPATE: Ease = Ease::new(0.5, -0.4, 0.25, 1.0);

    /// Carries past the target and comes back. Weight, without touching the
    /// springs.
    pub const OVERSHOOT: Ease = Ease::new(0.3, 0.0, 0.35, 1.5);

    /// Stays put until the last instant. A held pose that then cuts.
    pub const HOLD: Ease = Ease::new(0.95, 0.0, 1.0, 0.05);

    /// Every preset, for the hub's preset buttons.
    pub const PRESETS: &'static [(&'static str, Ease)] = &[
        ("linear", Ease::LINEAR),
        ("smooth", Ease::SMOOTH),
        ("in", Ease::IN),
        ("out", Ease::OUT),
        ("snap", Ease::SNAP),
        ("strike", Ease::STRIKE),
        ("anticipate", Ease::ANTICIPATE),
        ("overshoot", Ease::OVERSHOOT),
        ("hold", Ease::HOLD),
    ];

    /// The name of this shape, if it is one of the presets.
    pub fn preset_name(&self) -> Option<&'static str> {
        Ease::PRESETS
            .iter()
            .find(|(_, e)| {
                (e.x1 - self.x1).abs() < 1e-4
                    && (e.y1 - self.y1).abs() < 1e-4
                    && (e.x2 - self.x2).abs() < 1e-4
                    && (e.y2 - self.y2).abs() < 1e-4
            })
            .map(|(n, _)| *n)
    }

    /// How far through the change you are, `t` of the way through the time.
    pub fn at(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        if t <= 0.0 {
            return 0.0;
        }
        if t >= 1.0 {
            return 1.0;
        }
        // The curve is parameterised by its own parameter, not by x, so find
        // the parameter whose x is the one asked for. Bisection with a fixed
        // count: the same work and the same answer every time, which matters
        // because a baked table is checked in and diffed.
        let mut lo = 0.0f32;
        let mut hi = 1.0f32;
        for _ in 0..24 {
            let mid = 0.5 * (lo + hi);
            if axis(mid, self.x1, self.x2) < t {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        axis(0.5 * (lo + hi), self.y1, self.y2)
    }
}

/// One axis of a cubic Bézier anchored at 0 and 1.
fn axis(t: f32, a: f32, b: f32) -> f32 {
    let inv = 1.0 - t;
    3.0 * inv * inv * t * a + 3.0 * inv * t * t * b + t * t * t
}
