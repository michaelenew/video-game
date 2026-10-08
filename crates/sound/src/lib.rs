//! The game's voice, derived.
//!
//! The game had no sound, and three creatures were built around the hole
//! (`docs/design/exploration/0005_sound.md`). This crate is the art
//! direction's answer, in the same shape `look` gives colour: **a sound is a
//! function from a few numbers the simulation already has**, callable without
//! starting a game, so the whole voice can be rendered and listened to in a
//! second (`cargo run -p sound --example sheet`).
//!
//! Three layers:
//!
//! - [`synth`] -- the kit: noise, oscillators, resonators, envelopes.
//! - [`patch`] -- the instruments: a blow of some weight on some material of
//!   some size; a telegraph as long as its startup; a growl pitched by a
//!   throat; stone, fire, blood, wind. [`patch::Patch::render`] is the one
//!   place a waveform is made.
//! - [`cue`] -- what the simulation's transitions *mean*: a diff of the world
//!   before and after a tick into a fixed list of cues, each a patch and a
//!   place. The game plays them; the report could count them.
//!
//! Floats throughout: nothing here feeds the simulation. See
//! `docs/design/sound.md`.

pub mod cue;
pub mod patch;
pub mod spectrum;
pub mod synth;
pub mod wav;

pub use cue::{Cue, Cues, cues};
pub use patch::{Material, Patch};
