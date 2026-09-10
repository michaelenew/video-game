//! Presentation logic, with no engine dependency.
//!
//! Interpolation, camera framing and character posing all live here so they can
//! be unit tested without a window, and so the Bevy layer in `game` stays thin
//! enough to read in one sitting.
//!
//! Floating point is fine on this side of the boundary. `sim` is integer-only
//! because determinism demands it; nothing here feeds back into the simulation.

pub mod baked;
pub mod camera;
pub mod interp;
pub mod pose;

pub use camera::{CameraRig, Framing};
pub use interp::{Frame, PlayerView, interpolate};
pub use pose::{Part, PartTransform, Pose, pose_for};

/// Simulation rate, mirrored from `sim` for convenience.
pub const TICK_HZ: f32 = sim::TICK_HZ as f32;
pub const TICK_SECONDS: f32 = 1.0 / TICK_HZ;

pub(crate) const FX: f32 = 65536.0;

pub(crate) fn fx(v: sim::Fx) -> f32 {
    v.raw() as f32 / FX
}
