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

// ---------------------------------------------------------------------------
// Movement
// ---------------------------------------------------------------------------

/// Walk speed. Everything else is spaced against this, so it is the first
/// number to get right and the most expensive one to change later.
pub const MOVE_SPEED: Fx = Fx::ratio(7, 1);

/// Guarding is a crawl. Blocking should not be a way to travel.
pub const GUARD_MOVE_SPEED: Fx = Fx::ratio(2, 1);

/// Crouching is slower than walking, so ducking an overhead costs tempo.
pub const CROUCH_MOVE_SPEED: Fx = Fx::ratio(3, 1);

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
pub const JUMP_SPEED: Fx = Fx::ratio(8, 1);
pub const GRAVITY: Fx = Fx::ratio(-24, 1);

/// Terminal velocity. Without one, a long fall arrives faster than anyone can
/// react to, and per-class fall speed stops meaning anything at the bottom.
pub const FALL_CAP: Fx = Fx::ratio(-21, 1);

/// Gravity multiplier while the jump button is still held and you are rising.
///
/// This is what makes jump height variable: hold for a taller jump, tap for a
/// short one, and everything in between. It is a *sustain* rather than a cut on
/// release, because a cut makes the short hop feel like the jump was taken away
/// from you, whereas a sustain makes the tall one feel earned.
pub const JUMP_HOLD_GRAVITY: Fx = Fx::ratio(55, 100);

/// How long the sustain can last. Beyond this, gravity is gravity.
pub const JUMP_HOLD_FRAMES: u16 = 26;

/// Air acceleration, Quake-style. See `state::air_accelerate`.
pub const AIR_ACCEL: Fx = Fx::ratio(11, 1);

/// Ceiling on horizontal air speed, as a multiple of the ground walk.
///
/// A deliberate divergence from Source, where strafing gains speed without
/// bound and that unboundedness became the genre. In a fighter built on spacing,
/// a player who can reach any part of the arena from any other has removed
/// spacing from the game. Set high enough that good strafing is rewarded.
pub const AIR_SPEED_CAP: Fx = Fx::ratio(3, 2);

/// Turn rate toward the opponent, per tick, as a fraction of remaining error.
pub const TURN_RATE: Fx = Fx::ratio(25, 100);
/// Slower while guarding. This is what makes the facing arc a real cost and
/// what lets an opponent walk around a turtle.
pub const GUARD_TURN_RATE: Fx = Fx::ratio(6, 100);

// ---------------------------------------------------------------------------
// Defence
// ---------------------------------------------------------------------------

/// Cosine of the guard arc half-angle. 0.5 is a 120-degree frontal arc.
/// Guard is an arc, not a bubble -- see `defense.md`.
pub const GUARD_ARC_COS: Fx = Fx::ratio(1, 2);

/// Frames at the start of a guard that parry instead of blocking.
///
/// **The most important open number in the game.** Human reaction is roughly
/// 15 frames at 60 Hz, so four frames makes parry a read rather than a
/// reaction, which is intended. Whether it is *findable* is the question.
pub const PARRY_WINDOW: u16 = 4;

/// What a successful parry costs the attacker. Must be long enough that the
/// punish is worth the risk of trying to parry at all.
pub const PARRY_STAGGER: u16 = 34;

/// Dodge: committed, directional, invulnerable only at the start.
pub const DODGE_FRAMES: u16 = 22;
/// Fewer than DODGE_FRAMES, so the tail is punishable. If these were equal,
/// dodge would beat everything and never be punished.
pub const DODGE_IFRAMES: u16 = 10;
pub const DODGE_SPEED: Fx = Fx::ratio(17, 1);

/// How far a crouch lowers the hurtbox. Currently only matters as a flag, since
/// overheads are decided by move property rather than geometry.
pub const CROUCH_HEIGHT_SCALE: Fx = Fx::ratio(55, 100);

// ---------------------------------------------------------------------------
// Bodies and the arena
// ---------------------------------------------------------------------------

pub const BODY_RADIUS: Fx = Fx::ratio(1, 2);
pub const BODY_HEIGHT: Fx = Fx::ratio(18, 10);

/// Knockback decay per tick while stunned. Below 1.0 or a hit sends you
/// sliding forever.
pub const KNOCKBACK_DECAY: Fx = Fx::ratio(86, 100);

// ---------------------------------------------------------------------------
// Match
// ---------------------------------------------------------------------------

/// Target time to kill in versus is about 60 seconds. With the damage numbers
/// in `moves.rs` that is roughly six committed hits or seventeen pokes --
/// untested against a real match.
pub const MAX_HEALTH: i32 = 1000;

/// Pause after a knockout before the next round starts.
pub const ROUND_OVER_FRAMES: u16 = 150;

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
pub const AIR_DODGE_FRAMES: u16 = 16;
pub const AIR_DODGE_SPEED: Fx = Fx::ratio(13, 1);

/// Percent of walking speed kept while throwing a fast poke.
///
/// Pokes are the neutral tool and get thrown constantly. Rooting you for every
/// one made neutral sticky and read as the game snatching the controls away --
/// reported as jarring, and it was. Slowing you keeps the cost without the
/// lurch: you still cannot close or escape at full speed while swinging.
///
/// Sits between the crouch walk and the free walk, so "slowed by swinging" is a
/// speed the player already has a feel for.
pub const POKE_MOBILITY: u8 = 60;

/// How much horizontal speed survives each frame of a move that roots you.
///
/// Rooting is correct for the committed moves -- that is what commitment means
/// -- but arriving at rooted in a single frame is a snap from a full walk to
/// nothing, which is the jarring part rather than the rooting itself. Over
/// about four frames this bleeds off the speed instead. The distance slid is a
/// few centimetres; it changes nothing about the spacing and everything about
/// how it reads.
pub const ATTACK_ROOT_DECAY: Fx = Fx::ratio(62, 100);
