//! What the Veilstalker wants: its own terms in the scoring, where it walks,
//! and what it decides as a move commits (`veilstalker.md` §5).
//!
//! The Ridgeback's algorithm, with three things added and one gate:
//!
//! ```text
//! U(m) =  the Ridgeback's terms
//!       + edge(m)      a decloak near the edge of the target's view
//!       + bait(m)      the mimic, when it saw the target dodge recently
//!       + exposure(m)  painted or mottled: the smoke up, strikes down
//!       ⟂ view(m)      outside the glanced view cone, or occluded: nothing
//!       ⟂ fire(m)      a decloak point inside fire: nothing
//!       ⟂ smoke(m)     out of its cloud at somebody outside it, or into a
//!                      young one: nothing
//!       ⟂ stalk, quiet nothing while it stalks, or after a mimic
//! ```
//!
//! `leave` -- "enormous for the retreat after two hits" -- is not a score
//! here: the frame hook starts the retreat on the second hit itself
//! (`fight::leave`), which is the same thing without a weighted draw that
//! could in principle not draw it.
//!
//! Everything it knows of where you are *looking* is what the frame hook
//! glanced into the lore (`fight::glanced`): the brain still receives
//! positions and velocities and nothing else, and the look is a sample at
//! least a glance old.

use crate::fixed::Fx;
use crate::math::{self, V3};
use crate::monster::{Mind, Monster};

use super::fight::{self, view};
use super::{CLIMB, Knob, LUNGE, MIMIC, POUNCE, QUILLS, RAKE, SMOKE, SPEAR, SPECIES};

/// Its own terms in the scoring, after the shared ones.
pub fn appetite(m: &Monster, kind: u8, score: i32, mind: &Mind) -> i32 {
    let lore = mind.lore;
    let target = (m.brain.target as usize).min(1);
    let v = fight::view_of(lore, target);
    let strained = fight::below(m, Knob::StrainedHealth.fx());

    // **Up on a perch**, the pounce is all it has.
    if fight::perched(m) {
        if kind != POUNCE || fight::stalk_left(lore) > 0 || fight::quiet(lore) {
            return 0;
        }
        if !gate(m, v, lore) || !in_reach(m, POUNCE) {
            return 0;
        }
        return score.max(0) + edge(lore, target);
    }
    match kind {
        POUNCE => 0,
        CLIMB => {
            // A top in reach whose pounce would reach the target, the pounce
            // ready, and the creature unseen.
            let Some((at, _)) = fight::perch_of(lore) else {
                return 0;
            };
            let p = SPECIES.attack(POUNCE);
            let d = math::wide_flat_dist(at, m.brain.seen);
            let fits = d.raw() >= p.ideal_range.sub(p.range_span).raw()
                && d.raw() <= p.ideal_range.add(p.range_span).raw();
            // A climb is a strike begun: not while it stalks, and not twice
            // inside the pounce's lockout.
            let ready = m.brain.cooldown[POUNCE as usize] == 0 && fight::stalk_left(lore) == 0;
            if !fits || !ready || fight::quiet(lore) {
                return 0;
            }
            Knob::ClimbAppetite.raw()
        }
        SMOKE => {
            let exposed = painted(lore) || fight::mottled_count(m) > 0;
            // Strained, or exposed and hunting: not while it stalks unseen.
            let hunting = fight::stalk_left(lore) == 0;
            if !strained || v & view::IN_SMOKE != 0 || !(hunting || exposed) {
                return 0;
            }
            score.max(0)
                + if exposed {
                    Knob::ExposureSmoke.raw()
                } else {
                    0
                }
        }
        MIMIC => {
            // A mimic is a strike it does not make: thrown when a strike
            // could be, not while it stalks.
            if !strained || v & view::GHOST_OK == 0 || fight::quiet(lore) {
                return 0;
            }
            if fight::stalk_left(lore) > 0
                || fight::strikes_this_engagement(lore)
                    >= Knob::EngagementStrikes.raw().max(1) as u32
            {
                return 0;
            }
            let bait = fight::bait_seen(lore, fight::now(lore))
                .is_some_and(|ago| ago <= Knob::BaitMemory.raw().max(0) as u32);
            score.max(0) + if bait { Knob::BaitAppetite.raw() } else { 0 }
        }
        LUNGE | SPEAR | RAKE | QUILLS => {
            if fight::stalk_left(lore) > 0 || fight::quiet(lore) {
                return 0;
            }
            // **An engagement is a strike and its follow-up**: then it
            // vanishes, and hunts again.
            if fight::strikes_this_engagement(lore) >= Knob::EngagementStrikes.raw().max(1) as u32 {
                return 0;
            }
            // **Only where its feet would show**: a decloak on floor that
            // takes no print could not be told from a mimic (§2).
            if v & view::BARE != 0 {
                return 0;
            }
            if !gate(m, v, lore) || !in_reach(m, kind) {
                return 0;
            }
            let mut score = score;
            if score <= 0 {
                return 0;
            }
            if painted(lore) || fight::mottled_count(m) > 0 {
                score = Fx::from_int(score).mul(Knob::ExposureCut.fx()).to_int();
            }
            score += edge(lore, target);
            if v & view::WATCHED != 0 {
                score -= Knob::WatchedPenalty.raw();
            }
            score.max(0)
        }
        _ => 0,
    }
}

/// **The gate** every strike passes: where it would decloak is inside the
/// target's glanced view and not behind a solid; not in fire; not out of its
/// cloud at somebody outside it, nor into a cloud younger than `SmokeQuiet`.
fn gate(m: &Monster, v: u32, lore: &crate::lore::Lore) -> bool {
    let _ = (m, lore);
    if v & view::IN_VIEW == 0 || v & view::IN_FIRE != 0 {
        return false;
    }
    if v & view::IN_SMOKE != 0 && v & view::THEY_IN_SMOKE == 0 {
        return false;
    }
    v & view::THEY_IN_YOUNG_SMOKE == 0
}

/// **`edge`**: more for a decloak near the edge of the view than in the
/// middle of it -- the predator's "from behind" turned into "from the edge of
/// the frame", which a player can learn to watch (§5).
fn edge(lore: &crate::lore::Lore, target: usize) -> i32 {
    Fx::from_int(Knob::EdgeBias.raw())
        .mul(fight::edge_of(lore, target))
        .to_int()
}

/// Any paint lit on it.
fn painted(lore: &crate::lore::Lore) -> bool {
    (0..fight::PAINTS).any(|i| lore.word(fight::word::PAINT + i) != 0)
}

/// Is the target, as last seen and led, inside this move's own range?
fn in_reach(m: &Monster, kind: u8) -> bool {
    let a = SPECIES.attack(kind);
    let inside = |at: V3| {
        let d = math::wide_flat_dist(at, m.pos);
        d.raw() >= a.ideal_range.sub(a.range_span).raw()
            && d.raw() <= a.ideal_range.add(a.range_span).raw()
    };
    inside(m.lead_point(a.startup)) && inside(m.brain.seen)
}

/// **Where it walks while it is free.**
///
/// - Perched, it stays.
/// - Stalking, it slinks round a circle `StalkRange` out (further, a region
///   at a time, once mottled), toward the edge of the target's glanced view
///   on the side it is already on -- round the target a step at a time, never
///   across its front.
/// - Stalked out, it closes to the range it means to strike from, still at
///   the edge of the view; from beyond `BoundFrom` it bounds straight in.
/// - It keeps `FireShy` clear of any fire it sees.
pub fn prowl_to(m: &Monster, mind: &Mind) -> Option<V3> {
    if fight::perched(m) {
        return Some(m.pos);
    }
    let lore = mind.lore;
    let target = m.brain.seen;
    let i = (m.brain.target as usize).min(1);
    let (at, yaw) = fight::led_look(lore, i);
    let at = if at == V3::ZERO { target } else { at };
    let stalking = fight::stalk_left(lore) > 0;
    let gap = math::wide_flat_dist(m.pos, target);
    // **Seen, and followed**: painted or mottled and stalking with the
    // target close, it does not slink -- it runs, straight away from them,
    // out to where it stalks from.
    if stalking && fight::exposed(m) && gap.raw() < Knob::ExposedNear.fx().raw() {
        let away = fight::flat(m.pos.sub(target));
        let dir = if away.flat_len().raw() > 0 {
            math::wide_normalized(away)
        } else {
            V3::from_turns(m.yaw)
        };
        let _ = dir;
        let yaw = fight::open_way(m.pos, target, Knob::StalkRange.fx(), &mind.ground.bounds);
        return avoid_fire(m, mind, Some(inside(m.pos.add(V3::from_turns(yaw).scale(Knob::StalkRange.fx())), mind)));
    }
    if !stalking && gap.raw() > Knob::BoundFrom.fx().raw() {
        return avoid_fire(m, mind, None);
    }
    let reach = Knob::MottledReach
        .fx()
        .mul(Fx::from_int(fight::mottled_count(m)));
    let range = if stalking {
        Knob::StalkRange.fx().add(reach)
    } else {
        fight::plan_range(lore).add(reach)
    };
    // Which side of the look it is on, and the bearing at the edge of the view
    // on that side.
    let right = V3::from_turns(yaw.add(math::QUARTER_TURN));
    let side = if right.dot(fight::flat(m.pos.sub(at))).raw() >= 0 {
        Fx::ONE
    } else {
        Fx::ONE.neg()
    };
    let edge = Knob::ViewCone.fx().mul(Knob::StalkEdge.fx()).mul(side);
    let want = yaw.add(edge);
    // Round the target a step at a time, from the bearing it is on.
    let mine = fight::flat(m.pos.sub(at));
    // Hidden from the view it wants by something solid -- a trunk between
    // them -- it keeps going round until the line is clear.
    let hidden = fight::view_of(lore, i) & fight::view::HIDDEN != 0;
    let goal = if mine.flat_len().raw() <= 0 {
        at.add(V3::from_turns(want).scale(range))
    } else {
        let a = math::atan2_turns(mine.z, mine.x);
        let error = math::wrap_turns(want.sub(a));
        let step = Knob::StalkStep.fx();
        let turn = if hidden && !stalking {
            step.mul(side).neg()
        } else {
            error.clamp(step.neg(), step)
        };
        at.add(V3::from_turns(a.add(turn)).scale(range))
    };
    avoid_fire(m, mind, Some(inside(goal, mind)))
}

/// A goal kept inside the walls, a wall-look in from them.
fn inside(p: V3, mind: &Mind) -> V3 {
    let b = mind.ground.bounds;
    let room = Knob::WallLook.fx().add(SPECIES.margin());
    V3::new(
        p.x.clamp(b.lo_x.add(room), b.hi_x.sub(room)),
        p.y,
        p.z.clamp(b.lo_z.add(room), b.hi_z.sub(room)),
    )
}

/// **It sees fire and will not walk into it**: a goal near the fire the frame
/// hook saw is pushed out to `FireShy` from it, and a way to the target that
/// would pass through it goes round.
fn avoid_fire(m: &Monster, mind: &Mind, goal: Option<V3>) -> Option<V3> {
    let Some(fire) = fight::fire_near(mind.lore) else {
        return goal;
    };
    let shy = Knob::FireShy.fx();
    let to = goal.unwrap_or(m.brain.seen);
    let from_fire = fight::flat(to.sub(fire));
    let d = math::wide_flat_len(from_fire);
    let to = if d.raw() < shy.raw() {
        let dir = if d.raw() > 0 {
            math::wide_normalized(from_fire)
        } else {
            math::wide_normalized(fight::flat(m.pos.sub(fire)))
        };
        fire.add(dir.scale(shy))
    } else {
        to
    };
    // Its own way there: if the fire is nearer the line than `FireShy`,
    // step round it, square off the line on the side it is already on.
    let gap = math::flat_segment_gap(fire, m.pos, to);
    if gap.raw() < shy.raw() {
        let line = fight::flat(to.sub(m.pos));
        let dir = if line.flat_len().raw() > 0 {
            math::wide_normalized(line)
        } else {
            V3::from_turns(m.yaw)
        };
        let across = V3::new(dir.z.neg(), Fx::ZERO, dir.x);
        let side = if across.dot(fight::flat(m.pos.sub(fire))).raw() >= 0 {
            across
        } else {
            across.scale(Fx::ONE.neg())
        };
        return Some(fire.add(side.scale(shy.add(Fx::ONE))));
    }
    Some(to)
}

/// **As a move commits**: the pounce's circle where the target stands now --
/// positional, drawn from the first frame and never followed -- and the
/// climb's top.
pub fn commit(m: &mut Monster, kind: u8, mind: &Mind) {
    match kind {
        POUNCE => fight::mark_pounce(m, m.brain.seen, mind.ground),
        CLIMB => {
            if let Some((at, _)) = fight::perch_of(mind.lore) {
                m.aim_at(at);
            }
        }
        _ => {}
    }
}
