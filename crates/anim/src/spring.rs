//! Damped springs.
//!
//! Every piece of secondary motion in the factory is one of these. A spring
//! chasing a target overshoots and settles, which is what separates motion that
//! reads as *physical* from a linear interpolation that reads as dead.

/// A second-order spring toward a target.
#[derive(Clone, Copy, Debug)]
pub struct Spring {
    pub value: f32,
    pub velocity: f32,
    /// Angular frequency: how fast it wants to get there. Higher is snappier.
    pub frequency: f32,
    /// 1.0 is critically damped — fastest approach with no overshoot.
    /// Below 1.0 overshoots and rings, which is where the life is.
    pub damping: f32,
}

impl Spring {
    pub fn new(value: f32, frequency: f32, damping: f32) -> Spring {
        Spring {
            value,
            velocity: 0.0,
            frequency,
            damping,
        }
    }

    /// Critically damped: arrives as fast as possible without overshooting.
    /// The right choice for anything that should feel *controlled*.
    pub fn tight(value: f32, frequency: f32) -> Spring {
        Spring::new(value, frequency, 1.0)
    }

    /// Underdamped: overshoots and rings down. The right choice for anything
    /// that should feel *heavy* or *loose* — a trailing limb, a cape, a head.
    pub fn loose(value: f32, frequency: f32) -> Spring {
        Spring::new(value, frequency, 0.55)
    }

    pub fn step(&mut self, target: f32, dt: f32) {
        // Standard damped harmonic oscillator, integrated semi-implicitly:
        // velocity first, then position from the *new* velocity. Explicit Euler
        // pumps energy into a stiff spring and it blows up.
        let f = self.frequency;
        let accel = f * f * (target - self.value) - 2.0 * self.damping * f * self.velocity;
        self.velocity += accel * dt;
        self.value += self.velocity * dt;
    }

    /// Has it stopped moving in any way a viewer would notice?
    pub fn settled(&self, target: f32, tolerance: f32) -> bool {
        (self.value - target).abs() < tolerance && self.velocity.abs() < tolerance * 10.0
    }
}
