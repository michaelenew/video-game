//! **A trail behind a moving thing**: the arc a blade leaves, the streak
//! behind a dash. Every picture the renderer adds where the thing is drawn
//! ([`Path::push`]) -- two points, the inner and outer edge of a band, and
//! how bright it is there -- and the trail is that path joined by a curve
//! ([`ribbon`]), cut into fine columns. So a trail's newest end is the thing
//! itself, on every picture, and the way it is painted is the renderer's
//! (`game::swing`, `swing.wgsl`).
//!
//! What feeds one says only where the thing is: `arms::sample` for the
//! Champion's weapons, and the others beside the things they follow.
//! Allocation-free, so it holds to the view's frame budget.

use crate::math::{self, V3};
use sim::state::{Action, Player};

/// How many recovery frames a swing's trail keeps being drawn from the weapon
/// for: long enough for what the strike drew to wear away behind it.
pub const TRAIL_GHOST: u16 = 12;

/// How many recovery frames the weapon still lights its trail for: the end
/// of the stroke, before the return to guard begins.
pub const FOLLOW: f32 = 3.0;

/// How many recovery frames the follow-through is drawn for: [`TRAIL_GHOST`],
/// or a shorter recovery but its last frame, so it is gone before she is
/// free rather than cut off at it.
pub const fn ghost(recovery: u16) -> u16 {
    let most = recovery.saturating_sub(1);
    let ghost = if TRAIL_GHOST < most {
        TRAIL_GHOST
    } else {
        most
    };
    if ghost == 0 { 1 } else { ghost }
}

/// **How bright a swing's trail is where the weapon is now**, `alpha` of the
/// way on to the next frame, and which move it is: nothing through the
/// wind-up (the trail's lead-in, so it starts from nothing rather than a hard
/// edge), all of it while the volume is out, and out over [`FOLLOW`] frames
/// of the follow-through, squared -- bright through the end of the stroke and
/// gone by the time the weapon is lifted back to guard, so the return is not
/// drawn as a second cut. `None` once there is nothing left to draw from it.
pub fn strike(p: &Player, alpha: f32) -> Option<(u8, f32)> {
    match p.action {
        Action::Startup { kind, .. } => Some((kind, 0.0)),
        Action::Active { kind, .. } => Some((kind, 1.0)),
        Action::Recovery { kind, left } => {
            let m = sim::moves::get(p.class, kind);
            let ghost = ghost(m.recovery) as f32;
            let gone = m.recovery.saturating_sub(left) as f32 + alpha;
            if gone >= ghost {
                return None;
            }
            let left = (1.0 - gone / FOLLOW).max(0.0);
            Some((kind, left * left))
        }
        _ => None,
    }
}

/// How many places a trail remembers: a picture each, a sword's cut and its
/// tail at 240 pictures a second.
pub const TRAIL: usize = 24;

/// How many steps each stretch between two remembered places is cut into
/// for drawing, so the trail bends through them rather than turning a corner.
pub const SMOOTH: usize = 6;

/// How many columns a drawn trail has at most.
pub const RIBBON: usize = (TRAIL - 1) * SMOOTH + 1;

/// How much of the trail's length, from its oldest end, it fades in over.
pub const TAIL: f32 = 0.3;

/// One place the thing has been: the band's inner and outer edge, when (in
/// frames, with the share between them), and how bright the trail is there.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Sample {
    pub inner: V3,
    pub outer: V3,
    pub at: f32,
    pub glow: f32,
}

/// **Where the thing has been**, oldest first: one place a picture, for as
/// many frames back as the trail reaches. It is the renderer's, like a
/// cross-fade: a drawn pose runs on the renderer's clock, so it cannot be
/// asked of a frame gone by.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Path {
    samples: [Sample; TRAIL],
    count: usize,
}

impl Path {
    pub fn clear(&mut self) {
        self.count = 0;
    }

    pub fn samples(&self) -> &[Sample] {
        &self.samples[..self.count]
    }

    /// Add where the thing is now. A picture that went back in time (a
    /// rollback, a step backwards) first forgets everything after it; two
    /// pictures at the same moment are one place. What is older than
    /// `window` frames goes, but for the one just past it, so the trail's
    /// tail reaches all the way to nothing.
    pub fn push(&mut self, s: Sample, window: f32) {
        while self.count > 0 && self.samples[self.count - 1].at > s.at + 1e-4 {
            self.count -= 1;
        }
        if self.count > 0 && s.at - self.samples[self.count - 1].at < 0.02 {
            self.samples[self.count - 1] = s;
        } else {
            if self.count == TRAIL {
                self.samples.copy_within(1.., 0);
                self.count -= 1;
            }
            self.samples[self.count] = s;
            self.count += 1;
        }
        let mut stale = 0;
        while stale + 1 < self.count && s.at - self.samples[stale + 1].at >= window {
            stale += 1;
        }
        if stale > 0 {
            self.samples.copy_within(stale..self.count, 0);
            self.count -= stale;
        }
    }
}

/// **The trail as it is drawn**: the path joined by a curve, in columns from
/// the oldest end to the newest, each a point on the inner edge and one on
/// the outer; how far along it is in time (`life`: 1 now, 0 as far back as
/// it reaches), and how bright.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Ribbon {
    pub inner: [V3; RIBBON],
    pub outer: [V3; RIBBON],
    pub life: [f32; RIBBON],
    pub glow: [f32; RIBBON],
    pub count: usize,
}

/// One point of a Catmull-Rom curve through `b` and `c`, `t` of the way, with
/// `a` and `d` the points either side: the curve the thing's places lie on,
/// bending through them rather than turning a corner at each.
fn catmull(a: V3, b: V3, c: V3, d: V3, t: f32) -> V3 {
    let (t2, t3) = (t * t, t * t * t);
    std::array::from_fn(|i| {
        0.5 * (2.0 * b[i]
            + (c[i] - a[i]) * t
            + (2.0 * a[i] - 5.0 * b[i] + 4.0 * c[i] - d[i]) * t2
            + (3.0 * b[i] - a[i] - 3.0 * c[i] + d[i]) * t3)
    })
}

/// **The trail a path draws**, at the moment `now`, reaching `window` frames
/// back. Its last column is the last place pushed -- the thing itself, this
/// picture -- exactly.
pub fn ribbon(path: &Path, now: f32, window: f32) -> Ribbon {
    let s = path.samples();
    let n = s.len();
    let mut r = Ribbon {
        inner: [math::ZERO; RIBBON],
        outer: [math::ZERO; RIBBON],
        life: [0.0; RIBBON],
        glow: [0.0; RIBBON],
        count: 0,
    };
    if n < 2 {
        return r;
    }
    // The ends carried on straight, so the curve has a neighbour beyond each.
    let at = |edge: fn(&Sample) -> V3, i: isize| -> V3 {
        if i < 0 {
            math::sub(math::scale(edge(&s[0]), 2.0), edge(&s[1]))
        } else if i as usize >= n {
            math::sub(math::scale(edge(&s[n - 1]), 2.0), edge(&s[n - 2]))
        } else {
            edge(&s[i as usize])
        }
    };
    let inner: fn(&Sample) -> V3 = |x| x.inner;
    let outer: fn(&Sample) -> V3 = |x| x.outer;
    let mut k = 0;
    for seg in 0..n - 1 {
        let i = seg as isize;
        let steps = if seg == n - 2 { SMOOTH + 1 } else { SMOOTH };
        for step in 0..steps {
            let f = step as f32 / SMOOTH as f32;
            r.inner[k] = catmull(
                at(inner, i - 1),
                at(inner, i),
                at(inner, i + 1),
                at(inner, i + 2),
                f,
            );
            r.outer[k] = catmull(
                at(outer, i - 1),
                at(outer, i),
                at(outer, i + 1),
                at(outer, i + 2),
                f,
            );
            let when = s[seg].at + (s[seg + 1].at - s[seg].at) * f;
            r.life[k] = (1.0 - (now - when) / window).clamp(0.0, 1.0);
            r.glow[k] = s[seg].glow + (s[seg + 1].glow - s[seg].glow) * f;
            k += 1;
        }
    }
    r.count = k;
    // The oldest stretch fades in, so a young trail -- one whose tail is not yet
    // old enough to have worn away -- starts from nothing rather than an edge.
    let mut run = [0.0f32; RIBBON];
    for j in 1..k {
        run[j] = run[j - 1] + math::length(math::sub(r.outer[j], r.outer[j - 1]));
    }
    let total = run[k - 1].max(1e-4);
    for (glow, run) in r.glow.iter_mut().zip(run).take(k) {
        let x = (run / total / TAIL).clamp(0.0, 1.0);
        *glow *= x * x * (3.0 - 2.0 * x);
    }
    r
}
