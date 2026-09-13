//! The Elementalist's Cataclysm, lit at a fire pillar: a vortex that races off
//! along the line it was aimed and drags anyone it catches toward its centre
//! as it goes.
//!
//! It has a real velocity and is stepped frame by frame, the way a fire bolt
//! is and no effect in [`crate::effects`] is. That module's rule against a
//! velocity is about growth curves inside a fixed lifetime -- a pure function
//! of `age` is what lets a rollback re-simulate the middle of one exactly. A
//! tornado has nothing to grow: it moves in a straight line at a constant
//! speed until it leaves the arena or its own clock runs out, and that is just
//! as reproducible stepped as it is computed from `age`. See `crate::bolt`,
//! which made the same call for the same reason.

use crate::DT;
use crate::arena;
use crate::fixed::Fx;
use crate::math::V3;
use crate::state::{Hit, MAX_PLAYERS, Player, apply_hit};
use crate::tuning as t;

/// One per player. A second cast while one is still out replaces the caster's
/// own rather than crowding the field -- the same rule a fire bolt's cap
/// follows, and for the same reason: a hard cap turns "what happens when you
/// spam it" into a decision instead of an emergent pile-up.
pub const MAX_TORNADOES: usize = MAX_PLAYERS;

pub type Swirl = [Option<FireTornado>; MAX_TORNADOES];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FireTornado {
    pub pos: V3,
    /// Unit vector, taken from the beam that lit it -- the same rule a fire
    /// bolt's `dir` follows, so the tornado carries the crosshair's own line
    /// onward rather than working out a heading of its own.
    pub dir: V3,
    pub owner: u8,
    /// Frames alive. Its own clock rather than a distance travelled, because
    /// unlike a fire bolt it does not disappear on first contact -- it keeps
    /// moving through everyone it catches, and something has to end it that
    /// is not "ran out of people."
    pub age: u16,
}

/// Light a tornado at `at`, flying along `dir`.
///
/// Replaces the caster's own if one of hers is already out, rather than
/// filling a second slot -- a second cataclysm while the first is still
/// tearing across the arena is not a thing the kit is built to reward twice
/// over.
pub fn spawn(swirl: &mut Swirl, owner: u8, at: V3, dir: V3) {
    let fresh = FireTornado {
        pos: at,
        dir,
        owner,
        age: 0,
    };
    if let Some(slot) = swirl
        .iter_mut()
        .find(|s| s.is_none_or(|v| v.owner == owner))
    {
        *slot = Some(fresh);
    }
}

/// Fly every tornado one frame, and let it pull and burn whoever it catches.
///
/// `versus` is the same rule every other hazard in the game follows: in a hunt
/// the fighters are on the same side, and a tornado that could still catch a
/// team-mate would be the one hazard in the game that could.
pub fn step(swirl: &mut Swirl, players: &mut [Player; MAX_PLAYERS], versus: bool) {
    for slot in swirl.iter_mut() {
        let Some(mut vortex) = *slot else { continue };
        vortex.age += 1;
        vortex.pos = vortex.pos.add(vortex.dir.scale(t::tornado_speed().mul(DT)));

        if versus {
            for (i, target) in players.iter_mut().enumerate() {
                if i as u8 == vortex.owner || target.health <= 0 {
                    continue;
                }
                let p = *target;
                let apart = V3::new(
                    p.pos.x.sub(vortex.pos.x),
                    Fx::ZERO,
                    p.pos.z.sub(vortex.pos.z),
                );
                let dist = apart.flat_len();
                if dist.raw() > t::tornado_pull_radius().raw() {
                    continue;
                }

                // A tick of damage and a short stagger -- long enough that
                // holding a direction cannot simply shrug the pull off the
                // instant it starts, short enough that it is not a lock
                // between ticks. No knockback of its own, so the pull below is
                // the only thing that moves you.
                if vortex.age % t::effect_tick_frames().max(1) == 0 {
                    apply_hit(
                        target,
                        Hit {
                            damage: t::tornado_damage(),
                            hitstun: t::tornado_stagger(),
                            blockstun: 0,
                            knockback: Fx::ZERO,
                            launch: Fx::ZERO,
                            grabs: 0,
                            by: vortex.owner,
                            dir: V3::ZERO,
                            blocked: false,
                            parried: false,
                        },
                        versus,
                    );
                }

                // The suck. Every frame you are inside it, not just on the tick
                // -- last, so the tick's own zero-knockback reset above never
                // cancels the very thing that is supposed to be happening. An
                // acceleration, in the same units gravity is, so it has to be
                // paid out over time the same way -- see `tuning::gravity`.
                if dist.raw() > 0 {
                    let inward = apart.normalized();
                    let pull = t::tornado_pull().mul(DT);
                    target.vel.x = target.vel.x.sub(inward.x.mul(pull));
                    target.vel.z = target.vel.z.sub(inward.z.mul(pull));
                }
            }
        }

        let alive = vortex.age < t::tornado_life() && arena::inside(vortex.pos);
        *slot = alive.then_some(vortex);
    }
}
