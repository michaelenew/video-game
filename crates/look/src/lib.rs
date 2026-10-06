//! The art direction, with no engine in it.
//!
//! The same arrangement `view` has for presentation logic, and for the same
//! reason: everything here is a pure function from a handful of numbers to a
//! colour, so it can be looked at, tested and iterated on without launching a
//! game.
//!
//! That matters more than it sounds. **There will not be a developer for this
//! game forever**, so the cost that counts is not these arenas -- it is the
//! next one. A sky you can only judge by starting a fight, walking somewhere
//! the sky is visible and taking a screenshot is a minute apiece and the wrong
//! minute: what you are judging is a gradient, and a gradient does not need a
//! game to exist. `cargo run -p look --example skies` draws all of them at once.
//!
//! The loop this is built for, in order: **derive, look, tweak, fold back.**
//! A new arena gets a sky from one colour. Somebody looks at the sheet. What
//! they change by hand is a tweak. What the tweak turns out to be *generally*
//! true about is moved into the derivation, so the next arena starts closer.

pub mod edge;
pub mod palette;
pub mod sheet;
pub mod skies;
pub mod sky;
pub mod tint;

pub use edge::Edge;
pub use palette::Palette;
pub use sky::Sky;
