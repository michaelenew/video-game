//! Tempers: a beaten creature fought again, cleverer rather than tougher
//! (`docs/design/world.md` §4 and §6 W2).
//!
//! A temper is **a set of overrides on the brain knobs every creature has**,
//! and nothing else. Its health, its moves, its damage and its hide are the
//! creature as tuned; what changes is how it thinks, which `monsters.md` §7
//! calls the difficulty model:
//!
//! - **the glance shortens** -- it looks more often, so a change of direction
//!   has fewer frames to hide in;
//! - **the lead lengthens** -- it trusts its extrapolation further, so running
//!   in a straight line is punished;
//! - **the decisiveness rises** -- it picks among fewer of its best-scoring
//!   moves, so it does the right thing more often and the odd thing less;
//! - **its strain thresholds stop falling as far** with its health -- the end
//!   of the hunt stays methodical instead of turning frantic and controllable.
//!
//! A pack has a glance and a lead of its own (`pack::PackKnob`), and a critter
//! decides on a cadence; the first two take the same shares, and the cadence
//! takes the glance's. So tempers are generic over a creature that is one
//! body, a body with a pack, and a pack and nothing else.
//!
//! Each override is a **share of the creature's own value**, in the Oven under
//! "Tempers". So a species gets three tempers by existing, and a person tuning
//! one creature's glance moves all four of its tempers with it.
//!
//! **Where a temper lives.** One byte on each [`crate::monster::Monster`] and
//! on the [`crate::pack::Pack`], in the snapshot: the hunt's temper is part of
//! its initial state, so both peers build the same fight and a rollback across
//! the start of one replays it. It travels the way the picker does, in the
//! [`crate::input::Travel`] byte. Versus has no creature and so no temper.

use crate::fixed::Fx;
use crate::oven::{self, Scalar};

/// How many tempers there are: the creature as tuned, and three above it.
/// A design count (world.md §4), and the two bits the travel byte keeps for
/// it -- not a magnitude.
pub const TEMPERS: u8 = 4;

/// The highest temper.
pub const HIGHEST: u8 = TEMPERS - 1;

/// One temper's overrides, as shares and points.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Temper {
    /// Frames between glances, as a percentage of its own.
    pub glance: i32,
    /// Lead on the target, as a percentage of its own.
    pub lead: i32,
    /// Points added to its decisiveness, which is itself a percentage.
    pub decisive: i32,
    /// How far its strain thresholds fall with its health, as a percentage of
    /// how far its own fall.
    pub despair: i32,
}

/// The creature as tuned: every share whole, nothing added.
pub const AS_TUNED: Temper = Temper {
    glance: 100,
    lead: 100,
    decisive: 0,
    despair: 100,
};

/// Temper `n`'s overrides. Zero is the creature as tuned; anything past the
/// highest reads as the highest, so a byte from a build that knows more
/// tempers is the hardest this one has rather than a panic mid-rollback.
pub fn of(n: u8) -> Temper {
    let rows = [
        [
            Scalar::Temper1Glance,
            Scalar::Temper1Lead,
            Scalar::Temper1Decisive,
            Scalar::Temper1Despair,
        ],
        [
            Scalar::Temper2Glance,
            Scalar::Temper2Lead,
            Scalar::Temper2Decisive,
            Scalar::Temper2Despair,
        ],
        [
            Scalar::Temper3Glance,
            Scalar::Temper3Lead,
            Scalar::Temper3Decisive,
            Scalar::Temper3Despair,
        ],
    ];
    if n == 0 {
        return AS_TUNED;
    }
    let [g, l, d, s] = rows[(n.min(HIGHEST) - 1) as usize];
    Temper {
        glance: oven::scalar(g),
        lead: oven::scalar(l),
        decisive: oven::scalar(d),
        despair: oven::scalar(s),
    }
}

/// A frame count after a share: never below one, because a glance or a
/// cadence of zero frames is no glance at all.
fn frames(base: i32, percent: i32) -> i32 {
    ((base as i64 * percent as i64) / 100).max(1) as i32
}

/// Frames between glances, tempered.
pub fn glance(base: u16, n: u8) -> u16 {
    frames(base as i32, of(n).glance).min(u16::MAX as i32) as u16
}

/// A frame count that follows the glance: a critter's thinking cadence.
pub fn cadence(base: i32, n: u8) -> i32 {
    frames(base.max(1), of(n).glance)
}

/// A lead multiplier, tempered.
pub fn lead(base: Fx, n: u8) -> Fx {
    Fx::from_raw(((base.raw() as i64 * of(n).lead as i64) / 100) as i32)
}

/// A lead in frames (a pack's), tempered.
pub fn lead_frames(base: i32, n: u8) -> i32 {
    ((base.max(0) as i64 * of(n).lead as i64) / 100) as i32
}

/// Decisiveness, tempered: points added, and never past all of it.
pub fn decisiveness(base: i32, n: u8) -> i32 {
    (base + of(n).decisive).clamp(0, 100)
}

/// How far the strain thresholds fall, tempered.
pub fn despair(base: i32, n: u8) -> i32 {
    (base * of(n).despair) / 100
}
