//! What a structure breaks into when Cataclysm destroys it.
//!
//! Not one instantaneous blast -- several pieces, each with a real velocity,
//! fanned out from where the structure stood in a cone around the line
//! Cataclysm was aimed. A shotgun rather than a bomb: what actually connects
//! depends on how close everyone was and where they stood relative to the
//! spread, and it takes a moment to arrive rather than landing on the same
//! frame the structure went down. See `crate::bolt`, which made the same call
//! -- a real velocity, stepped frame by frame -- for the same reason: this is
//! a thing thrown, not a place that briefly exists.
//!
//! **The cone is a cone, not an arc.** An earlier version of this fanned the
//! pieces by rotating the aimed line's own bearing -- the same angle a
//! top-down map would show -- and left its pitch untouched. That is not a
//! spread around the line of effect; it is a spread around the world's
//! vertical axis, which only happens to look right when the line of effect is
//! level. Aimed up or down it stopped meaning anything, the way a ray built
//! from the chest along the look direction stops meaning anything once
//! `crate::aim`'s own warning about that mistake is ignored. What this uses
//! instead is [`crate::math::frame_about`] -- the same construction the
//! Grasp's arms spread around their own line of effect with -- which finds
//! sideways and up **square to `dir` itself**, however `dir` is pitched.

use crate::DT;
use crate::aim::{self, Contact, Path, Scene, Targets};
use crate::effects::Effects;
use crate::fixed::{Fx, cos_turns, sin_turns};
use crate::math::{V3, frame_about};
use crate::monster::Monster;
use crate::state::{Hit, MAX_PLAYERS, Player, apply_hit, guard_against};
use crate::stones;
use crate::tuning as t;

/// How many pieces one structure breaks into: one straight down the line
/// Cataclysm was aimed, and six more evenly spaced around it at the cone's
/// own half-angle.
///
/// A count, not a feel number -- the same reasoning `bolt::MAX_BOLTS` and the
/// Grasp's four arms use. A centre piece rather than a gap in the middle of
/// the fan is worth one slot on its own; six is the fewest that reads as a
/// ring rather than a handful of stray points once it is spread out over a
/// cone's whole surface, which is a lot more room to fill than an arc is.
pub const PIECES_PER_BLAST: usize = 7;

/// The ring around the centre piece.
const RING: usize = PIECES_PER_BLAST - 1;

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

/// Break a structure at `at` into [`PIECES_PER_BLAST`] pieces, in a cone
/// around the line Cataclysm was aimed along.
///
/// One piece flies straight down `dir`. The rest sit on the rim of a cone at
/// `debris_spread`'s half-angle, evenly spaced by [`frame_about`]'s own
/// sideways and up -- true perpendiculars to `dir` in three dimensions,
/// however `dir` is pitched, rather than a bearing swept around the world's
/// vertical axis. That is what makes this a cone standing in space rather
/// than a fan lying flat in whatever horizontal slice `dir` happens to pass
/// through.
pub fn blast(shrapnel: &mut Shrapnel, owner: u8, at: V3, dir: V3) {
    light_one(shrapnel, owner, at, dir);

    let (right, up) = frame_about(dir);
    let (axial, radial) = (cos_turns(t::debris_spread()), sin_turns(t::debris_spread()));
    for k in 0..RING {
        let azimuth = Fx::ratio(k as i32, RING as i32);
        let (c, s) = (cos_turns(azimuth), sin_turns(azimuth));
        let piece_dir = dir
            .scale(axial)
            .add(right.scale(radial.mul(c)))
            .add(up.scale(radial.mul(s)))
            .normalized();
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
                        interrupts: true,
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
            // `Terrain` cannot arrive: this does not ask for it, and the
            // arena is what stops the thing rather than what it hits. The arm
            // is here because the enum is exhaustive and the alternative is a
            // wildcard that would also swallow whatever is added next.
            Some(Contact::Fire { .. }) | Some(Contact::Terrain { .. }) | None => {}
        }

        piece.pos = leg.to;
        piece.travelled = piece.travelled.add(step);
        // Spent, or gone off the end of the world.
        *slot = (piece.travelled.raw() < t::debris_range().raw()
            && crate::arena::inside(piece.pos))
        .then_some(piece);
    }
}
