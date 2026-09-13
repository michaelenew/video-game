//! Animation factory.
//!
//! **This does not run in the game's hot path.** It generates motion offline
//! and bakes the result into a static table the game reads by index. That
//! separation is not an optimisation -- it is what keeps animation compatible
//! with rollback.
//!
//! ```text
//!   author intent  →  kinematics solver  →  baked frames  →  game lookup
//!   (a few poses)     (eases, springs)      (60 Hz table)    pose = f(state)
//! ```
//!
//! Hand-authoring every frame of natural motion is miserable, and the parts
//! that make motion read as *alive* -- follow-through, overlap, settle -- are
//! exactly the parts that are tedious to key by hand and easy to simulate. So:
//! author the intent as a few sparse poses, say how the time between them is
//! spent, let a spring-damper produce the in-betweens, then bake.
//!
//! The baked table is still a pure function of `(clip, frame)`, so rollback is
//! unaffected. See `docs/design/animation.md`.
//!
//! The game *does* link this crate, for one reason: the animation hub re-runs
//! the solver on every edit so a change can be seen immediately. Nothing in a
//! shipped frame calls it.

pub mod bake;
pub mod chain;
pub mod clips;
pub mod ease;
pub mod png;
pub mod sheet;
pub mod source;
pub mod spring;

pub use bake::{Baked, Feel, Key, Looseness, Recipe, bake};
pub use chain::{Chain, Segment};
pub use ease::Ease;
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

/// Bake every recipe, filling in a held rest pose for anything unauthored.
///
/// The fallback is on purpose: a missing clip should show up as a character
/// standing still and a line in the bake's output, not as a compile error that
/// blocks everybody else while one family is being written.
pub fn bake_all() -> (Vec<Baked>, Vec<view::Clip>) {
    let recipes = clips::all();
    let missing = clips::missing();
    let mut out = Vec::with_capacity(view::clips::CLIP_COUNT);
    for clip in view::clips::ALL.iter().copied() {
        match recipes.iter().find(|r| r.clip == clip) {
            Some(r) => out.push(bake(r)),
            None => out.push(Baked {
                clip,
                frames: vec![view::Pose::rest(); clip.length().max(1) as usize],
            }),
        }
    }
    (out, missing)
}
