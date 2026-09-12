//! The Elementalist's auto: a line out of her hand, and the fire it can light.
//!
//! Every other attack in the game is a bubble that exists for a few frames at
//! a place the move table picked. Hers is not. It is a **beam**: a short
//! wind-up, and then, on the frame the move comes out, an instantaneous ray
//! from her chest along the line the crosshair is on, out to a short-to-middle
//! distance. Nothing travels; there is no projectile to lead and nothing to
//! dodge once it has been thrown. What it meets *first* is the whole move:
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
//! ## Why this is a ray and not a circle
//!
//! It used to be neither. The auto put a flat circle at a fixed distance in
//! front of her and asked what was standing in it, which meant aiming up did
//! nothing whatsoever: the shot went the same distance along the ground it
//! always had, and the crosshair was decoration. Everything here is a real
//! three-dimensional ray for that reason, against the same upright cylinders
//! the aim resolver already traces -- see [`crate::math::ray_hits_cylinder`].
//!
//! ## What the beam does *not* stop on
//!
//! The arena. Walls and platforms are not traced against, so a shot aimed down
//! at the floor passes through it and finds nothing, which is a whiff rather
//! than a shot that stops short. Stones are the one piece of terrain the shot
//! reads, because they are the thing the move is *for*.

use crate::DT;
use crate::class::Class;
use crate::effects::{self, Effect, MAX_EFFECTS};
use crate::fixed::Fx;
use crate::math::V3;
use crate::monster::Monster;
use crate::state::{Action, Hit, MAX_PLAYERS, Player, apply_hit, guard_against};
use crate::stones::{self, Field};
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

// ---------------------------------------------------------------------------
// The beam
// ---------------------------------------------------------------------------

/// The line a shot is fired along: where it starts, where it points, how far it
/// runs and how thick it is.
///
/// A cylinder, in other words, and the renderer draws exactly this -- so what
/// you see is the volume that was tested rather than a separate reconstruction
/// of it that can drift.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Beam {
    pub from: V3,
    /// Unit vector, pitch included. This is the whole fix: `facing` is
    /// flattened to the horizontal because a body only turns level, and a shot
    /// sent along it can never leave the ground however the player aims.
    pub dir: V3,
    pub range: Fx,
    pub radius: Fx,
}

impl Beam {
    /// A point some distance along the line.
    pub fn at(&self, dist: Fx) -> V3 {
        self.from.add(self.dir.scale(dist))
    }

    /// The far end, where the shot runs out of range.
    pub fn end(&self) -> V3 {
        self.at(self.range)
    }
}

/// The first thing a beam meets, and how far along it sits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Struck {
    Fighter { index: usize, dist: Fx },
    Stone { index: usize, dist: Fx },
    Fire { dist: Fx },
}

impl Struck {
    pub fn dist(self) -> Fx {
        match self {
            Struck::Fighter { dist, .. } | Struck::Stone { dist, .. } | Struck::Fire { dist } => {
                dist
            }
        }
    }
}

/// What the shot meets first, or `None` if it reaches its whole range without
/// finding anything.
///
/// `hit_fighters` is false in a hunt, where the two of you are on the same
/// side and the creature is handled by its own exchange.
pub fn trace(
    beam: Beam,
    shooter: u8,
    players: &[Player; MAX_PLAYERS],
    hit_fighters: bool,
    stones: &Field,
    effects: &[Option<Effect>; MAX_EFFECTS],
) -> Option<Struck> {
    let mut best: Option<Struck> = None;
    let mut keep = |found: Struck| {
        if best.is_none_or(|b| found.dist().raw() < b.dist().raw()) {
            best = Some(found);
        }
    };

    if hit_fighters {
        for (index, p) in players.iter().enumerate() {
            if index as u8 == shooter || p.health <= 0 || p.action.invulnerable() {
                continue;
            }
            let Some(dist) = body_along(beam, p) else {
                continue;
            };
            keep(Struck::Fighter { index, dist });
        }
    }
    if let Some((index, dist)) =
        stones::first_along_shot(stones, beam.from, beam.dir, beam.range, beam.radius)
    {
        keep(Struck::Stone { index, dist });
    }
    if let Some(dist) =
        effects::first_fire_pillar_along(effects, beam.from, beam.dir, beam.range, beam.radius)
    {
        keep(Struck::Fire { dist });
    }
    best
}

/// Where a beam meets a fighter's body, if it does.
///
/// The body is the upright cylinder every other test in the game already uses,
/// swollen by the shot's own radius so that "do these two volumes touch" is one
/// ray against one cylinder. Crouching lowers it, which is what lets a crouch
/// duck a shot aimed over the head -- the first time height has decided a
/// fighter-on-fighter hit, and the point of the move being a line.
pub fn body_along(beam: Beam, victim: &Player) -> Option<Fx> {
    let dist = crate::math::ray_hits_cylinder(
        beam.from,
        beam.dir,
        victim.pos,
        t::body_radius().add(beam.radius),
        victim.hurt_height(),
    )?;
    (dist.raw() <= beam.range.raw()).then_some(dist)
}

/// What the beam does to a fighter it catches.
///
/// Damage, and the charge. **No stagger of any kind**: a fighter caught by the
/// auto is free again on the very next frame, and all they have lost is the
/// wind-up they were partway through. A poke that stunned would be an opener,
/// and this is not meant to be one -- it is meant to be the thing that makes
/// committing to a long telegraph in front of an Elementalist a decision.
///
/// Returns whether it connected at all, so the caller can spend the move's one
/// hit on it.
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
    /// Unit vector. Straight, and unaffected by gravity -- it is a bolt, and a
    /// bolt that dropped would stop going where the crosshair was pointing,
    /// which is the whole complaint this rework answers.
    pub dir: V3,
    pub owner: u8,
    /// How far it has come, so the range is a distance rather than a clock.
    pub travelled: Fx,
}

impl FireBolt {
    /// How far it moves in one frame.
    fn step_length(&self) -> Fx {
        t::fire_bolt_speed().mul(DT)
    }
}

/// Light a bolt at `at`, flying along `dir`.
///
/// The oldest in the air gives way when they are all busy, the same rule the
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
/// fast it is retuned to go.
pub fn step(
    flight: &mut Flight,
    players: &mut [Player; MAX_PLAYERS],
    hit_fighters: bool,
    quarry: &mut Option<Monster>,
) {
    let stones = stones::gather(players);
    for slot in flight.iter_mut() {
        let Some(mut shot) = *slot else { continue };
        let step = shot.step_length();
        let beam = Beam {
            from: shot.pos,
            dir: shot.dir,
            range: step,
            radius: t::fire_bolt_radius(),
        };

        // The creature, in a hunt. Checked first because it is the only thing
        // in the arena bigger than the step the bolt takes in a frame.
        if let Some(beast) = quarry.as_mut() {
            if let Some((part, _)) = beast.part_struck_along(beam.from, beam.dir, step, beam.radius)
            {
                beast.take_hit(part, t::fire_bolt_damage());
                *slot = None;
                continue;
            }
        }

        // A structure stops it, the same way a structure stops everything
        // else. Checked before the fighters so a body sheltering behind one is
        // actually sheltered.
        let blocked = stones::first_along_shot(&stones, beam.from, beam.dir, step, beam.radius);
        let caught = hit_fighters
            .then(|| {
                players
                    .iter()
                    .enumerate()
                    .filter(|(i, p)| {
                        *i as u8 != shot.owner && p.health > 0 && !p.action.invulnerable()
                    })
                    .filter_map(|(i, p)| body_along(beam, p).map(|d| (i, d)))
                    .min_by_key(|(_, d)| d.raw())
            })
            .flatten();

        match (caught, blocked) {
            (Some((i, hit_at)), stone) if stone.is_none_or(|(_, d)| hit_at.raw() <= d.raw()) => {
                let victim = players[i];
                let (guarding, parried) = guard_against(&victim, shot.pos, false);
                apply_hit(
                    &mut players[i],
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
                );
                *slot = None;
                continue;
            }
            (_, Some(_)) => {
                *slot = None;
                continue;
            }
            _ => {}
        }

        shot.pos = beam.end();
        shot.travelled = shot.travelled.add(step);
        // Spent, or gone off the end of the world. A bolt aimed at the sky has
        // to expire on something, and its range is the honest answer.
        *slot = (shot.travelled.raw() < t::fire_bolt_range().raw()
            && crate::arena::inside(shot.pos))
        .then_some(shot);
    }
}

/// Is this fighter the one class that throws a beam rather than a bubble?
pub fn throws_a_beam(p: &Player, kind: u8) -> bool {
    p.class == Class::Elementalist && kind == crate::state::SLOT_POKE
}
