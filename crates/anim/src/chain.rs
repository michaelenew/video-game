//! Chains: a driven root with segments that follow it.
//!
//! This is where "flows naturally" actually comes from. A limb does not arrive
//! at its pose at the same instant the torso does — it lags, overshoots, and
//! settles. Animators call it overlap and follow-through, and it is the single
//! biggest difference between motion that reads as alive and motion that reads
//! as a slideshow.
//!
//! Rather than key that by hand for every joint on every move, the factory
//! drives the root and lets the chain produce it.

use crate::spring::Spring;

/// One link. Each follows its parent with its own lag.
#[derive(Clone, Copy, Debug)]
pub struct Segment {
    pub angle: Spring,
    /// How much of the parent's motion is inherited before the spring acts.
    /// Near 1.0 follows rigidly; near 0.0 hangs behind.
    pub follow: f32,
}

impl Segment {
    pub fn new(frequency: f32, damping: f32, follow: f32) -> Segment {
        Segment {
            angle: Spring::new(0.0, frequency, damping),
            follow,
        }
    }
}

/// A driven chain. Element zero is the root, which follows the target
/// directly; each later element follows the one before it.
#[derive(Clone, Debug)]
pub struct Chain {
    pub segments: Vec<Segment>,
}

impl Chain {
    /// A chain that gets progressively looser toward its tip, which is how
    /// real limbs behave: the shoulder leads, the hand trails.
    pub fn tapered(links: usize, root_frequency: f32) -> Chain {
        let segments = (0..links)
            .map(|i| {
                let t = i as f32 / (links.max(2) - 1) as f32;
                Segment::new(
                    // Softer and slower toward the tip.
                    root_frequency * (1.0 - 0.45 * t),
                    // And less damped, so the tip rings a little.
                    1.0 - 0.4 * t,
                    1.0 - 0.35 * t,
                )
            })
            .collect();
        Chain { segments }
    }

    pub fn set(&mut self, angle: f32) {
        for s in self.segments.iter_mut() {
            s.angle.value = angle;
            s.angle.velocity = 0.0;
        }
    }

    /// Advance the whole chain toward a root target.
    pub fn step(&mut self, root_target: f32, dt: f32) {
        let mut parent = root_target;
        for s in self.segments.iter_mut() {
            // Each link chases a blend of its parent's angle and its own, so a
            // low `follow` leaves it hanging behind the parent's swing.
            let target = parent * s.follow + s.angle.value * (1.0 - s.follow);
            s.angle.step(target, dt);
            parent = s.angle.value;
        }
    }

    pub fn angles(&self) -> impl Iterator<Item = f32> + '_ {
        self.segments.iter().map(|s| s.angle.value)
    }

    pub fn tip(&self) -> f32 {
        self.segments.last().map(|s| s.angle.value).unwrap_or(0.0)
    }
}
