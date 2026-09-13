//! The Elementalist's auto: a line out of her hand, and the fire it can light.
//!
//! Every other attack in the game is a bubble that exists for a few frames at a
//! place the move table picked. Hers is not. It is the game's one **skillshot**
//! in the sense [`crate::aim`] means it: a short wind-up, and then an instant
//! line from her hand to *whatever the crosshair is on*. Nothing travels; there
//! is nothing to lead and nothing to dodge once it is thrown. What it meets
//! first is the whole move:
//!
//! - **a fighter** — small damage, and it takes whatever they were winding up.
//!   No stagger at all: they get their frames straight back, minus the move
//!   they had been charging. That trade is the auto's entire identity in
//!   neutral, and it is why the move is worth throwing at someone who is
//!   already committed rather than only at someone standing still;
//! - **a structure** — the stone is sent along the line, along the ground when
//!   she is aimed down it and up into the air when she is aimed above it;
//! - **fire** — the beam does not stop at a hazard, it *lights* one. A fire
//!   bolt leaves the pillar along the same line: fast, small, long range, low
//!   to middling damage and a little stagger. That is the poke, and it is the
//!   only part of the auto that has a speed.
//!
//! ## Where the line comes from
//!
//! [`crate::aim::skillshot_path`], and **only** from there. This module does no
//! aiming and no intersection arithmetic of its own: it asks for the path, asks
//! what is on it, and decides what that means. Both of the bugs this replaced
//! were the opposite -- a flat circle a fixed distance ahead, and then a ray
//! from the chest along the look direction, which is parallel to the
//! crosshair's ray and never converges with it.

use crate::DT;
use crate::aim::{self, Contact, Path, Scene, Targets};
use crate::class::Class;
use crate::fixed::Fx;
use crate::math::V3;
use crate::monster::Monster;
use crate::state::{Action, Hit, MAX_PLAYERS, Player, apply_hit, guard_against};
use crate::stones;
use crate::tuning as t;

/// How many fire bolts can be in the air at once.
///
/// A count, not a feel number. Two per fighter is already more than the auto's
/// own frame data can put in the air against any fire pillar that stands long
/// enough to be shot through twice.
pub const MAX_BOLTS: usize = MAX_PLAYERS * 2;

/// Every fire bolt in flight. Fixed size for the same reason the effects array
/// is: it is part of the rollback snapshot, and a heap allocation per frame of
/// re-simulation would be the most expensive thing in the tick.
pub type Flight = [Option<FireBolt>; MAX_BOLTS];

/// Is this the Elementalist's auto?
///
/// Three conditions rather than one, and the last two are not redundant. Being
/// a skillshot is what decides how the move is *aimed* and is a property of
/// the move table, so another class could be given one tomorrow. What the shot
/// *does* when it lands -- poke, kick a stone, light a pillar -- is this
/// class's alone, and it is specifically the poke's: Cataclysm is a second
/// skillshot on the same class, aimed the same way and resolved the same way
/// structurally, but what it does when it lands is `crate::tornado`'s answer,
/// not this one.
pub fn throws_a_beam(p: &Player, kind: u8) -> bool {
    crate::moves::get(p.class, kind).aim() == aim::Kind::Skillshot
        && p.class == Class::Elementalist
        && kind == crate::state::SLOT_POKE
}

/// What the beam can run into: bodies, stones, and fire.
///
/// Fire is on the list here and *not* on the aiming ray's, which is the whole
/// of the pillar interaction: you can see through flame, so it never steals the
/// crosshair, but a shot passing through it comes out the far side changed.
pub fn targets(versus: bool) -> Targets {
    Targets::none()
        .fighters(versus)
        .stones()
        .fire()
        .quarry(!versus)
}

/// What the beam does to a fighter it catches.
///
/// Damage, and the charge. **No stagger of any kind**: a fighter caught by the
/// auto is free again on the very next frame, and all they have lost is the
/// wind-up they were partway through. A poke that stunned would be an opener,
/// and this is not meant to be one -- it is meant to be the thing that makes
/// committing to a long telegraph in front of an Elementalist a decision.
pub fn poke(defender: &mut Player, from: V3, damage: i32) -> Poked {
    let (guarding, parried) = guard_against(defender, from, false);
    if parried {
        return Poked::Parried;
    }
    if guarding {
        // No chip damage, and nothing to interrupt: a guard is not a charge.
        return Poked::Blocked;
    }
    defender.health = (defender.health - damage).max(0);
    // The interrupt, and the whole of it. Startup is the only phase that is a
    // *charge* -- a move already out has been paid for, and taking it back
    // would make a no-stagger poke better than moves that do stagger.
    if matches!(defender.action, Action::Startup { .. }) {
        defender.action = Action::Free;
    }
    Poked::Hit
}

/// How a poke landed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Poked {
    Hit,
    Blocked,
    /// Read on the frame it came out. The attacker eats the stagger, the same
    /// as any other parry.
    Parried,
}

// ---------------------------------------------------------------------------
// The fire bolt
// ---------------------------------------------------------------------------

/// A bolt of fire in flight.
///
/// The one thing the Elementalist throws that has a speed, and the only reason
/// it does is that it was lit at a pillar rather than at her hand: it has to
/// visibly come *from the fire* or the interaction is invisible.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FireBolt {
    pub pos: V3,
    /// Unit vector, taken from the beam that lit it -- so the bolt carries the
    /// crosshair's own line onward rather than working out a direction of its
    /// own. Straight, and unaffected by gravity.
    pub dir: V3,
    pub owner: u8,
    /// How far it has come, so the range is a distance rather than a clock.
    pub travelled: Fx,
}

/// Light a bolt at `at`, flying along `dir`.
///
/// The one furthest along gives way when they are all busy, the same rule the
/// effects array uses: a cap has to decide what happens when it is reached, and
/// silently dropping the new one would make the move stop working exactly when
/// it was being used most.
pub fn light(flight: &mut Flight, owner: u8, at: V3, dir: V3) {
    let fresh = FireBolt {
        pos: at,
        dir,
        owner,
        travelled: Fx::ZERO,
    };
    if let Some(slot) = flight.iter_mut().find(|s| s.is_none()) {
        *slot = Some(fresh);
        return;
    }
    let oldest = flight
        .iter()
        .enumerate()
        .max_by_key(|(_, b)| b.map_or(0, |b| b.travelled.raw()))
        .map(|(i, _)| i)
        .unwrap_or(0);
    flight[oldest] = Some(fresh);
}

/// Fly every bolt one frame, and let it hit what it passes through.
///
/// Tested along the segment it covered this frame rather than at the point it
/// arrived at, so a bolt cannot step over a body between two frames however
/// fast it is retuned to go. The segment is a [`Path`] and the test is
/// [`aim::first_along`], the same pair the beam itself uses.
pub fn step(
    flight: &mut Flight,
    players: &mut [Player; MAX_PLAYERS],
    effects: &crate::effects::Effects,
    versus: bool,
    quarry: &mut Option<Monster>,
) {
    let stones = stones::gather(players);
    for slot in flight.iter_mut() {
        let Some(mut shot) = *slot else { continue };
        let step = t::fire_bolt_speed().mul(DT);
        let leg = Path {
            from: shot.pos,
            to: shot.pos.add(shot.dir.scale(step)),
        };
        // A structure stops it, the same way a structure stops everything
        // else; a pillar does not, or a bolt could not leave the one that lit
        // it. Its own fire is off the list for that reason.
        let met = {
            let seen = *players;
            let scene = Scene {
                stones: &stones,
                players: &seen,
                effects,
                quarry: quarry.as_ref(),
            };
            aim::first_along(
                leg,
                t::fire_bolt_radius(),
                shot.owner,
                &scene,
                Targets::none().fighters(versus).stones().quarry(!versus),
            )
        };

        match met {
            Some(Contact::Fighter { index, .. }) => {
                let victim = players[index];
                let (guarding, parried) = guard_against(&victim, shot.pos, false);
                apply_hit(
                    &mut players[index],
                    Hit {
                        damage: t::fire_bolt_damage(),
                        hitstun: t::fire_bolt_stagger(),
                        blockstun: t::fire_bolt_blockstun(),
                        knockback: t::fire_bolt_knockback(),
                        launch: Fx::ZERO,
                        grabs: 0,
                        by: shot.owner,
                        // Along the line it was flying, flattened: knockback
                        // shoves people about the arena, and lifting them is a
                        // different move's job.
                        dir: V3::new(shot.dir.x, Fx::ZERO, shot.dir.z).normalized(),
                        blocked: guarding,
                        parried,
                    },
                    versus,
                );
                *slot = None;
                continue;
            }
            Some(Contact::Quarry { part, .. }) => {
                if let Some(beast) = quarry.as_mut() {
                    beast.take_hit(part, t::fire_bolt_damage());
                }
                *slot = None;
                continue;
            }
            Some(Contact::Stone { .. }) => {
                *slot = None;
                continue;
            }
            Some(Contact::Fire { .. }) | None => {}
        }

        shot.pos = leg.to;
        shot.travelled = shot.travelled.add(step);
        // Spent, or gone off the end of the world. A bolt aimed at the sky has
        // to expire on something, and its range is the honest answer.
        *slot = (shot.travelled.raw() < t::fire_bolt_range().raw()
            && crate::arena::inside(shot.pos))
        .then_some(shot);
    }
}
