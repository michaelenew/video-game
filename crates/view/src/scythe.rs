//! The Blood mage's scythe, drawn at the reach it hits at.
//!
//! Her reach grows with the grey on her bar, and `docs/design/aiming.md`
//! refuses a reach that changes with a bar for a good reason: a reach only one
//! player can see is unlearnable. The one condition under which it is allowed
//! is the rule `CLAUDE.md` puts on the debug overlay -- **the blade is drawn at
//! the length it hits at** -- so that what both players read is a weapon, not
//! a number. This module is that rule. It is a pure function of simulation
//! state so `tests/kinematics.rs` can hold it to the hit test without a
//! renderer, the way `game::effect_piece` is held to the effects.
//!
//! A war scythe is a haft with a blade set at the head, so it is drawn as two
//! pieces: the haft from the grip to the neck, and the blade from the neck to
//! the tip. **The tip is the thing the hit test reaches with**, and it sits
//! exactly at the far end of the hit volume while she is swinging; the neck
//! bows a little off that line so the blade reads as a blade rather than as
//! the last third of a pole. At rest the same weapon stands upright beside
//! her, tilted forward, with the tip the live reach from her grip -- which is
//! the same function the hit test lengthens the sweep with, so the weapon a
//! player watches grow while she is standing still is exactly as long as the
//! one that is about to be swung.

use crate::math::{self, V3};
use sim::state::{self, Player};

/// The scythe, as two lines in the arena.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Scythe {
    /// Where the haft is held.
    pub grip: V3,
    /// Where the blade is set on the haft.
    pub neck: V3,
    /// The point of the blade: the reach, exactly.
    pub tip: V3,
    /// The hit volume's radius: how wide the blade's plane is drawn.
    pub width: f32,
}

impl Scythe {
    /// Grip to tip: the reach.
    pub fn reach(&self) -> f32 {
        math::length(math::sub(self.tip, self.grip))
    }
}

/// How much of the reach is blade rather than haft.
const BLADE: f32 = 0.3;
/// How far off the reach's line the neck bows, as a share of the reach. Small:
/// the tip is on the line, and the bow is what makes it read as a scythe.
const BOW: f32 = 0.12;
/// How far forward of upright the resting haft leans.
const LEAN: f32 = 0.34;

/// Where the scythe is, if this fighter carries one.
///
/// `grip` is where the drawn hand is in the arena, which the renderer knows
/// and the simulation does not: it is the one thing here that is not read off
/// the snapshot, and it decides where the resting weapon *starts*, never how
/// long it is.
pub fn scythe(p: &Player, grip: V3) -> Option<Scythe> {
    if p.class != sim::Class::BloodMage {
        return None;
    }
    let facing = math::normalize_or(
        [crate::fx(p.facing.x), 0.0, crate::fx(p.facing.z)],
        [1.0, 0.0, 0.0],
    );
    let swinging = p
        .action
        .attack_kind()
        .is_some_and(sim::moves::blood::scythe);
    if swinging {
        if let Some(hb) = state::hitbox(p) {
            return Some(set(fx3(hb.from), fx3(hb.to), facing, crate::fx(hb.radius)));
        }
    }
    let reach = crate::fx(state::scythe_reach(p));
    let axis = math::normalize_or(
        [
            facing[0] * LEAN,
            (1.0 - LEAN * LEAN).sqrt(),
            facing[2] * LEAN,
        ],
        [0.0, 1.0, 0.0],
    );
    let width = crate::fx(sim::moves::get(p.class, sim::moves::blood::SWEEP).radius);
    Some(set(
        grip,
        math::add(grip, math::scale(axis, reach)),
        facing,
        width,
    ))
}

/// Set the blade on a haft that runs from `grip` to `tip`.
///
/// The neck sits a little off the line, toward wherever `facing` is
/// perpendicular to it -- forward of an upright haft, above or beside a
/// swung one -- so the blade returns to the tip at an angle.
fn set(grip: V3, tip: V3, facing: V3, width: f32) -> Scythe {
    let along = math::sub(tip, grip);
    let reach = math::length(along);
    let axis = math::normalize_or(along, [0.0, 1.0, 0.0]);
    let dot = facing[0] * axis[0] + facing[1] * axis[1] + facing[2] * axis[2];
    let side = math::normalize_or(math::sub(facing, math::scale(axis, dot)), [0.0, 1.0, 0.0]);
    let neck = math::add(
        math::add(grip, math::scale(axis, reach * (1.0 - BLADE))),
        math::scale(side, reach * BOW),
    );
    Scythe {
        grip,
        neck,
        tip,
        width,
    }
}

fn fx3(v: sim::V3) -> V3 {
    [crate::fx(v.x), crate::fx(v.y), crate::fx(v.z)]
}
