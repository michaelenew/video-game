//! Presentation logic, with no engine dependency.
//!
//! Interpolation, the follow camera and character posing all live here so they can
//! be unit tested without a window, and so the Bevy layer in `game` stays thin
//! enough to read in one sitting.
//!
//! Floating point is fine on this side of the boundary. `sim` is integer-only
//! because determinism demands it; nothing here feeds back into the simulation.

pub mod baked;
pub mod build;
pub mod camera;
pub mod interp;
pub mod pose;
pub mod trail;

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

/// Convert a look angle in radians to the simulation's aim unit.
///
/// The simulation counts angles in 1/65536 of a turn as an integer, because an
/// integer angle cannot drift, wraps exactly, and is the same on every machine.
/// The renderer counts in radians because that is what trigonometry and Bevy
/// want. This is the one place the two meet, so the conversion is written once.
pub fn aim_from_radians(yaw: f32) -> u16 {
    let turns = yaw / std::f32::consts::TAU;
    (turns.rem_euclid(1.0) * 65536.0).round() as u32 as u16
}

pub fn radians_from_aim(aim: u16) -> f32 {
    aim as f32 / 65536.0 * std::f32::consts::TAU
}
