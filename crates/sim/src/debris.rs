//! What a structure breaks into when Cataclysm destroys it.
//!
//! Not one instantaneous blast -- several pieces, each with a real velocity,
//! fanned out from where the structure stood along the line Cataclysm was
//! aimed. A shotgun rather than a bomb: what actually connects depends on how
//! close everyone was and where they stood relative to the spread, and it
//! takes a moment to arrive rather than landing on the same frame the
//! structure went down. See `crate::bolt`, which made the same call -- a real
//! velocity, stepped frame by frame -- for the same reason: this is a thing
//! thrown, not a place that briefly exists.

use crate::DT;
use crate::aim::{self, Contact, Path, Scene, Targets};
use crate::effects::Effects;
use crate::fixed::Fx;
use crate::math::{V3, atan2_turns};
use crate::monster::Monster;
use crate::state::{Hit, MAX_PLAYERS, Player, apply_hit, guard_against};
use crate::stones;
use crate::tuning as t;

/// How many pieces one structure breaks into.
///
/// A count, not a feel number -- the same reasoning `bolt::MAX_BOLTS` and the
/// Grasp's four arms use. Odd, so one piece always flies straight down the
/// line Cataclysm was aimed rather than the centre of the fan being a gap
/// between two pieces.
pub const PIECES_PER_BLAST: usize = 7;

/// Enough for one full blast per player at once. A second structure broken
/// before the first blast's pieces have burned out is the rare case, and
/// [`light_one`] gives the oldest piece up rather than silently refusing the
/// new blast, the same rule a fire bolt's cap follows.
pub const MAX_DEBRIS: usize = MAX_PLAYERS * PIECES_PER_BLAST;

pub type Shrapnel = [Option<Shard>; MAX_DEBRIS];

/// One piece of a broken structure, in flight.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Shard {
    pub pos: V3,
    /// Unit vector. Straight from the fan `blast` computed -- a shard does not
    /// steer or fall, the same way a fire bolt does not.
    pub dir: V3,
    pub owner: u8,
    /// How far it has come, so its range is a distance rather than a clock.
    pub travelled: Fx,
}

/// Break a structure at `at` into [`PIECES_PER_BLAST`] pieces, fanned around
/// the line Cataclysm was aimed along.
///
/// The fan is built by rotating `dir`'s own horizontal bearing rather than by
/// offsetting it sideways: a true angular spread reads as pieces radiating
/// from the point of impact at every range, where a sideways nudge would read
/// as a spread only close up and as a set of parallel lines far out. The
/// pieces keep `dir`'s vertical angle unchanged -- the fan is a shotgun fired
/// roughly level, not a cone in every dimension.
pub fn blast(shrapnel: &mut Shrapnel, owner: u8, at: V3, dir: V3) {
    let bearing = atan2_turns(dir.z, dir.x);
    let flat = V3::new(dir.x, Fx::ZERO, dir.z).flat_len();
    let half = (PIECES_PER_BLAST as i32 - 1) / 2;
    for k in 0..PIECES_PER_BLAST as i32 {
        let offset = if half > 0 {
            t::debris_spread().mul(Fx::ratio(k - half, half))
        } else {
            Fx::ZERO
        };
        let level = V3::from_turns(bearing.add(offset)).scale(flat);
        let piece_dir = V3::new(level.x, dir.y, level.z).normalized();
        light_one(shrapnel, owner, at, piece_dir);
    }
}

/// Light one piece. The one furthest along gives way when the array is full,
/// the same rule `bolt::light` follows.
fn light_one(shrapnel: &mut Shrapnel, owner: u8, at: V3, dir: V3) {
    let fresh = Shard {
        pos: at,
        dir,
        owner,
        travelled: Fx::ZERO,
    };
    if let Some(slot) = shrapnel.iter_mut().find(|s| s.is_none()) {
        *slot = Some(fresh);
        return;
    }
    let oldest = shrapnel
        .iter()
        .enumerate()
        .max_by_key(|(_, s)| s.map_or(0, |s| s.travelled.raw()))
        .map(|(i, _)| i)
        .unwrap_or(0);
    shrapnel[oldest] = Some(fresh);
}

/// Fly every piece one frame, and let it hit whatever it reaches first.
///
/// Tested along the segment it covered this frame, the same way a fire bolt
/// is, so a fast piece cannot step over a body standing between two frames.
pub fn step(
    shrapnel: &mut Shrapnel,
    players: &mut [Player; MAX_PLAYERS],
    effects: &Effects,
    versus: bool,
    quarry: &mut Option<Monster>,
) {
    let stones = stones::gather(players);
    for slot in shrapnel.iter_mut() {
        let Some(mut piece) = *slot else { continue };
        let step = t::debris_speed().mul(DT);
        let leg = Path {
            from: piece.pos,
            to: piece.pos.add(piece.dir.scale(step)),
        };
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
                t::debris_radius(),
                piece.owner,
                &scene,
                Targets::none().fighters(versus).stones().quarry(!versus),
            )
        };

        match met {
            Some(Contact::Fighter { index, .. }) => {
                let victim = players[index];
                let (guarding, parried) = guard_against(&victim, piece.pos, false);
                apply_hit(
                    &mut players[index],
                    Hit {
                        damage: t::debris_damage(),
                        hitstun: t::debris_stagger(),
                        blockstun: t::debris_blockstun(),
                        knockback: t::debris_knockback(),
                        launch: Fx::ZERO,
                        grabs: 0,
                        by: piece.owner,
                        dir: V3::new(piece.dir.x, Fx::ZERO, piece.dir.z).normalized(),
                        blocked: guarding,
                        parried,
                    },
                );
                *slot = None;
                continue;
            }
            Some(Contact::Quarry { part, .. }) => {
                if let Some(beast) = quarry.as_mut() {
                    beast.take_hit(part, t::debris_damage());
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

        piece.pos = leg.to;
        piece.travelled = piece.travelled.add(step);
        // Spent, or gone off the end of the world.
        *slot = (piece.travelled.raw() < t::debris_range().raw()
            && crate::arena::inside(piece.pos))
        .then_some(piece);
    }
}
