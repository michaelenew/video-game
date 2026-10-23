//! **`--pretty`: the game as pretty as it has yet been made** (the owner,
//! 2026-10-23). One switch for a player, where the look is a set of switches
//! for the people building it: the world in the brushed look
//! (`docs/design/painting.md`), the cast painted over
//! (`docs/design/cast-look.md`), and the creatures' grown bodies, which are
//! drawn whenever `--box-creatures` is not asked for.
//!
//! The switches it stands for stay what they were -- `--look <name>` and
//! `--painted-cast` are how a look is tried while it is built, and the first
//! still wins over this (`--pretty --look wash` is the painted cast in a
//! watercolour). When something new is the best the game has looked, it
//! goes on this list, so `--pretty` is always every step so far.

/// Is this run as pretty as the game can be? `?pretty` on the web.
pub fn on() -> bool {
    crate::platform::flag("--pretty")
}

/// The world's look under `--pretty`, unless `--look` names another.
pub const LOOK: &str = "brushed";
