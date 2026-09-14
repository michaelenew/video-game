//! Air, thrown: what the Elementalist has in her hands off the ground.
//!
//! Earth is the thing she is standing on, and in the air she is not standing on
//! it. So the two shots on her clicks while airborne are **air** — the element
//! the kit lists as a later specialisation axis, borrowed for the one situation
//! where the one she has cannot reach. See
//! `docs/design/kits/elementalist.md`.
//!
//! Both **travel**, which is what separates them from everything she throws
//! standing up. Bolt and Cataclysm are instant lines: the shot is resolved on
//! the frame it comes out, and there is nothing to lead and nothing to dodge
//! once it is thrown (`crate::bolt`, `crate::state::World::fire_the_cataclysm`).
//! These have a speed, and the reason is the situation rather than the element:
//! she is falling while she throws them, so a shot that landed instantly would
//! be a free hit taken from a position she cannot hold. A flight time is what
//! makes being in the air a trade.
//!
//! ```text
//!   Air bolt    small, fast, long        the poke: it reaches further than
//!                                         anything she has on the floor
//!   Gale        large, slow, and it       the committed one: a disc that is a
//!               grows as it goes          puff at her hand and a wall at the tip
//! ```
//!
//! ## The Gale grows, and that is the whole move
//!
//! A disc of air leaves her hand at [`tuning::gale_start`] of the size its move
//! row lists and arrives at full size at the end of its travel. **Damage and
//! knockback ride the same fraction**, so what it is worth is what it has
//! become: caught at point blank it is a nudge, and caught at the far end it is
//! the heaviest shove in the kit. That inverts the spacing every other
//! projectile in the game has, and it is deliberate — it is the same sentence
//! Flame spitter is written around ("you want them at the tip"), said as a
//! thing that flies.
//!
//! The stun does **not** scale. How long a hit holds somebody is the move's
//! frame data, and frame data that changed with distance would be a move nobody
//! could learn.
//!
//! ## Where the line comes from
//!
//! [`crate::aim::skillshot_path`], and only from there — this module does no
//! aiming and no intersection arithmetic of its own, the same rule
//! `crate::bolt` and `crate::debris` follow. The shot flies **along** that line
//! rather than to the end of it: the crosshair picked a direction, and a disc
//! thrown at somebody four metres away still travels its whole range.
//!
//! [`tuning::gale_start`]: crate::tuning::gale_start

use crate::DT;
use crate::aim::{self, Contact, Path, Scene, Targets};
use crate::class::Class;
use crate::effects::Effects;
use crate::fixed::Fx;
use crate::math::V3;
use crate::monster::Monster;
use crate::moves::{self, Move, elementalist};
use crate::state::{Hit, MAX_PLAYERS, Player, apply_hit, guard_against};
use crate::stones;
use crate::tuning as t;

/// How many air shots can be in flight at once.
///
/// A count, not a feel number — the same reasoning `bolt::MAX_BOLTS` and the
/// Grasp's four arms use. Two per fighter is more than the moves' own frame
/// data plus the repeat lockout can put in the air at the ranges these travel,
/// and the cap still has to decide what happens when it is reached: the one
/// furthest along gives way, so the move never goes quiet exactly when it is
/// being used most.
pub const MAX_GUSTS: usize = MAX_PLAYERS * 2;

/// Every air shot in flight. Fixed size for the reason the effects array is:
/// it is part of the rollback snapshot, and a heap allocation per frame of
/// re-simulation would be the most expensive thing in the tick.
pub type Flight = [Option<Gust>; MAX_GUSTS];

/// Which of the two she threw.
///
/// The kind is what the shot carries rather than a copy of its numbers,
/// because every number it needs is already in the move table: one lookup
/// answers how far it goes, how thick it is, what it deals and how long it
/// stuns, out of the same row the frame table prints. A shot with its own
/// private copy of any of that is a second version of the ability to disagree
/// with the first.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gale {
    /// Left click, airborne. Small, fast, and the longest reach in the class.
    Bolt,
    /// Right click, airborne. Large, slow, and growing.
    Disc,
}

impl Gale {
    /// Which of the Elementalist's moves throws this.
    pub const fn slot(self) -> u8 {
        match self {
            Gale::Bolt => elementalist::AIR_BOLT,
            Gale::Disc => elementalist::GALE,
        }
    }

    /// Which shot a move throws, if it throws one at all.
    ///
    /// The class test is not redundant with the slot test: slot numbers are
    /// per class, and the Champion's fifth move is not an air bolt.
    pub const fn thrown_by(class: Class, kind: u8) -> Option<Gale> {
        if !matches!(class, Class::Elementalist) {
            return None;
        }
        match kind {
            elementalist::AIR_BOLT => Some(Gale::Bolt),
            elementalist::GALE => Some(Gale::Disc),
            _ => None,
        }
    }

    /// The move this came out of, live from the Oven.
    pub fn source(self) -> Move {
        moves::get(Class::Elementalist, self.slot())
    }

    pub fn speed(self) -> Fx {
        match self {
            Gale::Bolt => t::air_bolt_speed(),
            Gale::Disc => t::gale_speed(),
        }
    }

    /// How much of itself this shot has become, `through` of the way along its
    /// travel.
    ///
    /// **One number, and it is both the size and the force.** A bolt is always
    /// all of itself. A disc opens from [`tuning::gale_start`] to one, and what
    /// it deals and how hard it shoves are that same fraction of its row —
    /// because on a disc of air those are not two facts, they are how much air
    /// is in it. The stun is deliberately not on this list; see the module
    /// header.
    ///
    /// [`tuning::gale_start`]: crate::tuning::gale_start
    pub fn swell(self, through: Fx) -> Fx {
        match self {
            Gale::Bolt => Fx::ONE,
            Gale::Disc => {
                let start = t::gale_start().min(Fx::ONE);
                start.add(Fx::ONE.sub(start).mul(through))
            }
        }
    }
}

/// One shot of air in flight.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Gust {
    pub pos: V3,
    /// Unit vector, taken from the line `crate::aim` solved when the move came
    /// out. Straight, and unaffected by gravity — it is air, and it is already
    /// moving faster than the fighter who threw it.
    pub dir: V3,
    pub owner: u8,
    /// How far it has come, so its range is a distance rather than a clock —
    /// and, for the disc, so its size is a function of where it is rather than
    /// of how long it has been alive.
    pub travelled: Fx,
    pub gale: Gale,
}

impl Gust {
    /// How far along its travel it is, from none of it to all of it.
    pub fn through(&self) -> Fx {
        let reach = self.gale.source().reach;
        if reach.raw() <= 0 {
            return Fx::ONE;
        }
        let p = self.travelled.div(reach);
        if p.raw() > Fx::ONE.raw() { Fx::ONE } else { p }
    }

    /// How much of itself it is carrying, right now.
    pub fn swell(&self) -> Fx {
        self.gale.swell(self.through())
    }

    /// Its radius, right now. What the hit test uses and what the renderer
    /// draws, so a disc cannot be drawn one size and tested at another.
    pub fn girth(&self) -> Fx {
        self.gale.source().radius.mul(self.swell())
    }
}

/// Throw one along the line the aim already solved.
///
/// The one furthest along gives way when they are all busy, the same rule the
/// effects array and the fire bolts use.
pub fn throw(flight: &mut Flight, owner: u8, gale: Gale, path: Path) {
    let fresh = Gust {
        pos: path.from,
        dir: path.dir(),
        owner,
        travelled: Fx::ZERO,
        gale,
    };
    if let Some(slot) = flight.iter_mut().find(|s| s.is_none()) {
        *slot = Some(fresh);
        return;
    }
    let oldest = flight
        .iter()
        .enumerate()
        .max_by_key(|(_, g)| g.map_or(0, |g| g.travelled.raw()))
        .map(|(i, _)| i)
        .unwrap_or(0);
    flight[oldest] = Some(fresh);
}

/// Fly every shot one frame, and let it hit whatever it reaches first.
///
/// Tested along the segment it covered this frame rather than at the point it
/// arrived at, so a shot cannot step over a body between two frames however
/// fast it is retuned to go. The segment is a [`Path`] and the test is
/// [`aim::first_along`] — the same pair the beam, the fire bolt and Cataclysm's
/// debris all use.
pub fn step(
    flight: &mut Flight,
    players: &mut [Player; MAX_PLAYERS],
    effects: &Effects,
    versus: bool,
    quarry: &mut Option<Monster>,
) {
    let stones = stones::gather(players);
    for slot in flight.iter_mut() {
        let Some(mut shot) = *slot else { continue };
        let m = shot.gale.source();
        let step = shot.gale.speed().mul(DT);
        let leg = Path {
            from: shot.pos,
            to: shot.pos.add(shot.dir.scale(step)),
        };
        // Its size as it is on this leg. A disc that is still growing is
        // tested at the size it has, which is what makes standing close to her
        // the answer to it.
        let girth = shot.girth();
        let swell = shot.swell();
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
                girth,
                shot.owner,
                &scene,
                // A structure stops it, the same way a structure stops
                // everything else she throws, and the self-obstruction is the
                // real cost the kit says it is. Fire is off the list: an air
                // shot passing through a pillar is an interaction worth having
                // and is not one anybody has played yet -- see the kit's open
                // questions.
                Targets::none().fighters(versus).stones().quarry(!versus),
            )
        };

        match met {
            Some(Contact::Fighter { index, .. }) => {
                let victim = players[index];
                let (guarding, parried) = guard_against(&victim, shot.pos, m.unblockable);
                apply_hit(
                    &mut players[index],
                    Hit {
                        damage: force(m.damage, swell),
                        // Frame data, not force: how long a hit holds you is
                        // the thing a player learns, and a stun that changed
                        // with distance could not be learnt.
                        hitstun: m.hitstun,
                        blockstun: m.blockstun,
                        knockback: m.knockback.mul(swell),
                        launch: Fx::ZERO,
                        grabs: 0,
                        by: shot.owner,
                        // Along the line it was flying, flattened: knockback
                        // shoves people about the arena, and lifting them is a
                        // different move's job.
                        dir: V3::new(shot.dir.x, Fx::ZERO, shot.dir.z).normalized(),
                        blocked: guarding,
                        parried,
                        interrupts: true,
                    },
                );
                *slot = None;
                continue;
            }
            Some(Contact::Quarry { part, .. }) => {
                if let Some(beast) = quarry.as_mut() {
                    beast.take_hit(part, force(m.damage, swell));
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
        // Spent, or gone off the end of the world. A shot aimed at the sky has
        // to expire on something, and its range is the honest answer.
        *slot = (shot.travelled.raw() < m.reach.raw() && crate::arena::inside(shot.pos))
            .then_some(shot);
    }
}

/// What a shot carrying `swell` of itself deals, out of a listed `damage`.
///
/// Rounded down and floored at nothing: a disc caught the instant it leaves her
/// hand should be worth very little, and never worth less than nothing.
fn force(damage: i32, swell: Fx) -> i32 {
    Fx::from_int(damage).mul(swell).to_int().max(0)
}
