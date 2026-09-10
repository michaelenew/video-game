//! Deterministic combat simulation.
//!
//! A pure function of `(state, inputs) -> state`. No rendering, no I/O, no
//! engine, no dependencies, no floating point. Everything that makes rollback
//! netcode possible lives here, and nothing else does.
//!
//! See `docs/design/architecture.md` for why the boundary sits where it does.

pub mod fixed;
pub mod input;
pub mod math;
pub mod state;

pub use fixed::Fx;
pub use input::Input;
pub use math::V3;
pub use state::{PlayerId, World};

/// Simulation rate. Fighting games are authored in frames, and every duration
/// in the design documents is a frame count at this rate.
pub const TICK_HZ: u32 = 60;

/// Seconds per tick, as fixed point.
pub const DT: Fx = Fx::ratio(1, TICK_HZ as i32);
