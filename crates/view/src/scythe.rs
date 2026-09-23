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
//! Two cases, and both read the same reach. While one of the two scythe moves
//! has its volume out, the blade *is* that volume: the capsule from
//! `sim::state::hitbox`, hub to head, at its live radius. The rest of the time
//! the blade hangs off the grip along the facing at `sim::state::scythe_reach`,
//! which is the same function the hit test lengthens the sweep with -- so
//! the weapon a player watches grow while she is standing still is exactly
//! as long as the one that is about to be swung.

use sim::state::{self, Player};

/// The blade, as a line in the arena with a thickness.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Blade {
    pub from: [f32; 3],
    pub to: [f32; 3],
    /// Half its thickness: the capsule's radius.
    pub width: f32,
}

impl Blade {
    pub fn length(&self) -> f32 {
        let d = [
            self.to[0] - self.from[0],
            self.to[1] - self.from[1],
            self.to[2] - self.from[2],
        ];
        (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
    }
}

/// How far above level the resting blade is carried, as a share of its
/// length. Presentation: a pole held dead level reads as a spear.
const CARRIED_UP: f32 = 0.3;

/// Where the blade is, if this fighter carries one.
///
/// `grip` is where the drawn hand is in the arena, which the renderer knows
/// and the simulation does not: it is the one thing here that is not read off
/// the snapshot, and it decides where the resting blade *starts*, never how
/// long it is.
pub fn blade(p: &Player, grip: [f32; 3]) -> Option<Blade> {
    if p.class != sim::Class::BloodMage {
        return None;
    }
    let swinging = p
        .action
        .attack_kind()
        .is_some_and(sim::moves::blood::scythe);
    if swinging {
        if let Some(hb) = state::hitbox(p) {
            return Some(Blade {
                from: fx3(hb.from),
                to: fx3(hb.to),
                width: crate::fx(hb.radius),
            });
        }
    }
    let reach = crate::fx(state::scythe_reach(p));
    let flat = [crate::fx(p.facing.x), 0.0, crate::fx(p.facing.z)];
    let along = crate::math::normalize_or([flat[0], CARRIED_UP, flat[2]], [1.0, 0.0, 0.0]);
    let width = crate::fx(sim::moves::get(p.class, sim::moves::blood::SWEEP).radius);
    Some(Blade {
        from: grip,
        to: [
            grip[0] + along[0] * reach,
            grip[1] + along[1] * reach,
            grip[2] + along[2] * reach,
        ],
        width,
    })
}

fn fx3(v: sim::V3) -> [f32; 3] {
    [crate::fx(v.x), crate::fx(v.y), crate::fx(v.z)]
}
