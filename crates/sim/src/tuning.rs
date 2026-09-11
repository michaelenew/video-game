//! Every number that decides how the game feels, in one place.
//!
//! These will be changed constantly and over a long period. Two rules make that
//! survivable:
//!
//! 1. **Each value carries its provenance** — why it is what it is, or an
//!    explicit note that it was a guess. A number nobody can justify is a
//!    number nobody dares change.
//! 2. **`docs/design/feel-log.md` records the experiments.** The value lives
//!    here; the reasoning lives there. Reverted experiments are the valuable
//!    ones, so they get logged too.
//!
//! `crates/sim/tests/feel.rs` pins the *relationships* between these, not the
//! values. Move a number freely; if a relationship test fails, either it is a
//! bug or a design decision changed and the documents need updating with it.

use crate::fixed::Fx;
use crate::oven::{self, Scalar};

// ---------------------------------------------------------------------------
// Movement
// ---------------------------------------------------------------------------

/// Walk speed. Everything else is spaced against this, so it is the first
/// number to get right and the most expensive one to change later.
pub fn move_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::MoveSpeed))
}

/// Guarding is a crawl. Blocking should not be a way to travel.
pub fn guard_move_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::GuardMoveSpeed))
}

/// Crouching is slower than walking, so ducking an overhead costs tempo.
pub fn crouch_move_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::CrouchMoveSpeed))
}

/// Takeoff speed, and the gravity it is chosen against.
///
/// Deliberately floatier than a first pass would pick. Verticality is meant to
/// be part of the positioning game, and a jump you are only airborne for a third
/// of a second in is one nobody has time to *do* anything with -- you commit,
/// and it is over before you have read the situation you jumped into. A beginner
/// needs long enough in the air to notice where the other player went.
///
/// Full hop apexes around 2.2 m over roughly a second; a short hop is about half
/// that. Both are relationships the feel tests pin, not numbers to trust.
pub fn jump_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::JumpSpeed))
}
pub fn gravity() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::Gravity))
}

/// Terminal velocity. Without one, a long fall arrives faster than anyone can
/// react to, and per-class fall speed stops meaning anything at the bottom.
pub fn fall_cap() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::FallCap))
}

/// Gravity multiplier while the jump button is still held and you are rising.
///
/// This is what makes jump height variable: hold for a taller jump, tap for a
/// short one, and everything in between. It is a *sustain* rather than a cut on
/// release, because a cut makes the short hop feel like the jump was taken away
/// from you, whereas a sustain makes the tall one feel earned.
pub fn jump_hold_gravity() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::JumpHoldGravity))
}

/// How long the sustain can last. Beyond this, gravity is gravity.
pub fn jump_hold_frames() -> u16 {
    oven::scalar(Scalar::JumpHoldFrames) as u16
}

/// Air acceleration, Quake-style. See `state::air_accelerate`.
pub fn air_accel() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::AirAccel))
}

/// Ceiling on horizontal air speed, as a multiple of the ground walk.
///
/// A deliberate divergence from Source, where strafing gains speed without
/// bound and that unboundedness became the genre. In a fighter built on spacing,
/// a player who can reach any part of the arena from any other has removed
/// spacing from the game. Set high enough that good strafing is rewarded.
pub fn air_speed_cap() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::AirSpeedCap))
}

/// Turn rate toward the opponent, per tick, as a fraction of remaining error.
pub fn turn_rate() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::TurnRate))
}
/// Slower while guarding. This is what makes the facing arc a real cost and
/// what lets an opponent walk around a turtle.
pub fn guard_turn_rate() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::GuardTurnRate))
}

// ---------------------------------------------------------------------------
// Defence
// ---------------------------------------------------------------------------

/// Cosine of the guard arc half-angle. 0.5 is a 120-degree frontal arc.
/// Guard is an arc, not a bubble -- see `defense.md`.
pub fn guard_arc_cos() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::GuardArcCos))
}

/// Frames at the start of a guard that parry instead of blocking.
///
/// **The most important open number in the game.** Human reaction is roughly
/// 15 frames at 60 Hz, so four frames makes parry a read rather than a
/// reaction, which is intended. Whether it is *findable* is the question.
pub fn parry_window() -> u16 {
    oven::scalar(Scalar::ParryWindow) as u16
}

/// What a successful parry costs the attacker. Must be long enough that the
/// punish is worth the risk of trying to parry at all.
pub fn parry_stagger() -> u16 {
    oven::scalar(Scalar::ParryStagger) as u16
}

/// Dodge: committed, directional, invulnerable only at the start.
pub fn dodge_frames() -> u16 {
    oven::scalar(Scalar::DodgeFrames) as u16
}
/// Fewer than DODGE_FRAMES, so the tail is punishable. If these were equal,
/// dodge would beat everything and never be punished.
pub fn dodge_iframes() -> u16 {
    oven::scalar(Scalar::DodgeIframes) as u16
}
pub fn dodge_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::DodgeSpeed))
}

/// How far a crouch lowers the hurtbox. Currently only matters as a flag, since
/// overheads are decided by move property rather than geometry.
pub fn crouch_height_scale() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::CrouchHeightScale))
}

// ---------------------------------------------------------------------------
// Bodies and the arena
// ---------------------------------------------------------------------------

pub fn body_radius() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::BodyRadius))
}
pub fn body_height() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::BodyHeight))
}

/// Knockback decay per tick while stunned. Below 1.0 or a hit sends you
/// sliding forever.
pub fn knockback_decay() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::KnockbackDecay))
}

// ---------------------------------------------------------------------------
// Match
// ---------------------------------------------------------------------------

/// Target time to kill in versus is about 60 seconds. With the damage numbers
/// in `moves.rs` that is roughly six committed hits or seventeen pokes --
/// untested against a real match.
pub fn max_health() -> i32 {
    oven::scalar(Scalar::MaxHealth)
}

/// Pause after a knockout before the next round starts.
pub fn round_over_frames() -> u16 {
    oven::scalar(Scalar::RoundOverFrames) as u16
}

/// Reaction time at 60 Hz, used by the feel tests to reason about what is and
/// is not reactable. Roughly 250 ms, which is the usual figure for a simple
/// visual reaction.
pub const HUMAN_REACTION_FRAMES: u16 = 15;

/// The airdodge: shorter and slower than the grounded one.
///
/// Shorter because you are already committed by being in the air -- the jump
/// was the commitment, and stacking a long vulnerable tail on top of it would
/// make any anti-air a guaranteed kill. Slower because a horizontal burst at
/// ground-dodge speed, from a standing jump, crosses more of the arena than a
/// dodge should.
pub fn air_dodge_frames() -> u16 {
    oven::scalar(Scalar::AirDodgeFrames) as u16
}
pub fn air_dodge_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::AirDodgeSpeed))
}

/// Percent of walking speed kept while throwing a fast poke.
///
/// Pokes are the neutral tool and get thrown constantly. Rooting you for every
/// one made neutral sticky and read as the game snatching the controls away --
/// reported as jarring, and it was. Slowing you keeps the cost without the
/// lurch: you still cannot close or escape at full speed while swinging.
///
/// Sits between the crouch walk and the free walk, so "slowed by swinging" is a
/// speed the player already has a feel for.
pub fn poke_mobility() -> u8 {
    oven::scalar(Scalar::PokeMobility) as u8
}

/// How much horizontal speed survives each frame of a move that roots you.
///
/// Rooting is correct for the committed moves -- that is what commitment means
/// -- but arriving at rooted in a single frame is a snap from a full walk to
/// nothing, which is the jarring part rather than the rooting itself. Over
/// about four frames this bleeds off the speed instead. The distance slid is a
/// few centimetres; it changes nothing about the spacing and everything about
/// how it reads.
pub fn attack_root_decay() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::AttackRootDecay))
}
