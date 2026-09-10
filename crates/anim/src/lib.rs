//! Animation factory.
//!
//! **This does not run in the game.** It runs offline, generates motion, and
//! bakes the result into a static table the game reads by index. That
//! separation is not an optimisation — it is what keeps animation compatible
//! with rollback.
//!
//! ```text
//!   author intent  →  kinematics solver  →  baked frames  →  game lookup
//!   (a few poses)     (springs, chains)     (60 Hz table)    pose = f(state)
//! ```
//!
//! Hand-authoring every frame of natural motion is miserable, and the parts
//! that make motion read as *alive* — follow-through, overlap, settle — are
//! exactly the parts that are tedious to key by hand and easy to simulate.
//! So: author the intent as a few sparse poses, let a spring-damper chain
//! produce the in-betweens, then bake.
//!
//! The baked table is still a pure function of `(action, frame)`, so rollback
//! is unaffected. See `architecture.md`.

pub mod bake;
pub mod chain;
pub mod spring;

pub use bake::{Baked, bake};
pub use chain::{Chain, Segment};
pub use spring::Spring;

/// Simulation rate. Baked tables are sampled at this.
pub const HZ: f32 = 60.0;
pub const DT: f32 = 1.0 / HZ;

/// How many substeps the solver runs per baked frame.
///
/// Springs stiff enough to look snappy are stiff enough to go unstable at
/// 60 Hz. Substepping is cheaper than making the solver implicit, and this is
/// offline code so the cost does not matter.
pub const SUBSTEPS: u32 = 8;
