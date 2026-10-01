//! What each cat wants: its own terms in the scoring, where it walks, and
//! what it decides as a move commits (`the-pair.md` §5).
//!
//! Each cat is a full `Monster` with its own glance and its own scoring; what
//! is added here reads only its own samples and what the pair brain wrote in
//! the lore about its partner (`fight::word`). It never receives a button, a
//! look or a facing: `Mind` is positions and velocities.
//!
//! ```text
//! U(m) = the Ridgeback's terms
//!      + role(m)    the role's own moves: Holder rake, swat, pounce (and the
//!                   feint the pounce may be); Striker ambush, trip, pounce,
//!                   perch
//!      + fast(m)    the ambush, on a sample faster than a walk
//!      + perch(m)   a top in reach, and the target beyond `PerchFar`
//!      + bond(m)    the interpose, when the mate is wounded and you are near it
//!      ⟂ partner(m) nothing landing within `StaggerGap` of its partner's hit
//!      ⟂ crossing   nothing while the roles swap
//! ```

use crate::DT;
use crate::fixed::Fx;
use crate::math::{self, V3};
use crate::monster::{Mind, Monster};

use super::fight::{self, flag, state};
use super::{AMBUSH, DIVE, FEINT, INTERPOSE, Knob, PERCH, POUNCE, RAKE, SPECIES, SWAT, TRIP};

/// The cat's own terms in the scoring, after the shared ones.
pub fn appetite(m: &Monster, kind: u8, score: i32, mind: &Mind) -> i32 {
    let slot = fight::slot_of(m);
    let lore = mind.lore;
    let st = fight::state_of(lore, slot);
    if st & state::CROSSING != 0 {
        return 0;
    }
    let target = m.brain.seen;
    let range = math::wide_flat_dist(target, m.pos);

    // Up on a top, the dive is all it has -- and nothing reaches under the
    // lip.
    if fight::perched(m) {
        let dive = SPECIES.attack(DIVE);
        let most = dive.ideal_range.add(dive.range_span);
        if kind != DIVE
            || range.raw() <= Knob::DiveLip.fx().raw()
            || range.raw() > most.raw()
        {
            return 0;
        }
        return score.max(0) + Knob::DiveAppetite.raw();
    }
    if kind == DIVE {
        return 0;
    }

    let enraged = fight::enraged(m);
    if enraged && matches!(kind, FEINT | AMBUSH | INTERPOSE | PERCH) {
        return 0;
    }

    // The bond: the wounded hangs back and throws only what reaches from
    // there; the healthy one stands in front of it.
    if st & state::WOUNDED != 0 && !matches!(kind, POUNCE | PERCH) {
        return 0;
    }
    if kind == INTERPOSE {
        if st & state::GUARD == 0 {
            return 0;
        }
        let mate = fight::pos_of(lore, 1 - slot);
        let near = math::wide_flat_dist(target, mate).raw() <= Knob::InterposeNear.fx().raw();
        // Not if it is already standing on the line, or there is no room on
        // it.
        let Some(at) = interpose_point(m, mind) else {
            return 0;
        };
        let on_line = math::wide_flat_dist(at, m.pos).raw() <= Knob::LandShort.fx().raw();
        return if near && !on_line {
            Knob::InterposeAppetite.raw()
        } else {
            0
        };
    }

    // **Nothing out of its own reach.** The shared terms add a wounded
    // animal's appetite to every move whatever the range; a cat does not
    // swipe at the air five metres off.
    if !matches!(kind, PERCH) && !in_reach(m, kind) {
        return 0;
    }
    let role = fight::role(m);
    let mut score = score;
    if kind == PERCH {
        let Some((at, _)) = fight::perch_of(lore, slot) else {
            return 0;
        };
        let beyond = range.raw() > Knob::PerchFar.fx().raw();
        let reach = math::wide_flat_dist(at, m.pos).raw() <= Knob::PerchReach.fx().raw();
        if !beyond || !reach || role == flag::HOLDER {
            return 0;
        }
        score = Knob::PerchAppetite.raw();
    }
    let ours = match role {
        flag::HOLDER => matches!(kind, RAKE | SWAT | POUNCE),
        flag::STRIKER => matches!(kind, AMBUSH | TRIP | POUNCE | PERCH),
        _ => true,
    };
    if !ours {
        return 0;
    }
    // **The Striker strikes from behind.** Until it is round the far side of
    // the target from its partner, all it throws is the tail at whoever
    // comes close -- or the ambush at a body it saw move fast.
    if role == flag::STRIKER && st & state::PARTNER != 0 && !matches!(kind, TRIP | PERCH) {
        let mate = fight::pos_of(lore, 1 - slot);
        let round = fight::cos_between(
            fight::flat(m.pos.sub(target)),
            fight::flat(mate.sub(target)),
        );
        let behind = round.raw() <= fight::cos_turns(Knob::BehindAngle.fx()).raw();
        let v = m.brain.seen_vel;
        let fast = math::wide_flat_len(V3::new(v.x, Fx::ZERO, v.z)).raw()
            > Knob::SeenFastSpeed.fx().raw();
        if !behind && !(kind == AMBUSH && fast) {
            return 0;
        }
    }
    let signature = matches!(
        (role, kind),
        (flag::HOLDER, RAKE | SWAT) | (flag::STRIKER, AMBUSH | TRIP)
    );
    if score > 0 && signature {
        score += Knob::RoleAppetite.raw();
    }

    // **"It saw you dodge" is only a velocity.** A sample faster than any
    // walk raises the ambush, the way a closing target raises the forward
    // moves.
    if kind == AMBUSH && score > 0 {
        let v = m.brain.seen_vel;
        let speed = math::wide_flat_len(V3::new(v.x, Fx::ZERO, v.z));
        if speed.raw() > Knob::SeenFastSpeed.fx().raw() {
            score += Knob::FastAppetite.raw();
        }
    }

    // **Two hits never land together.** Nothing whose live window would come
    // within `StaggerGap` of the partner's.
    if score > 0 && st & state::PARTNER != 0 {
        if let Some((f1, t1)) = fight::live_of(lore, 1 - slot) {
            let a = SPECIES.attack(kind);
            let f0 = fight::now(lore).wrapping_add(a.startup as u32 + 1);
            let t0 = f0.wrapping_add(a.active as u32);
            let gap = Knob::StaggerGap.raw().max(0) as u32;
            if f0 <= t1.wrapping_add(gap) && f1 <= t0.wrapping_add(gap) {
                return 0;
            }
        }
    }
    score
}

/// Is the target, as last seen and led, inside this move's own range?
fn in_reach(m: &Monster, kind: u8) -> bool {
    let a = SPECIES.attack(kind);
    let at = m.lead_point(a.startup);
    let d = math::wide_flat_dist(at, m.pos);
    d.raw() >= a.ideal_range.sub(a.range_span).raw() && d.raw() <= a.ideal_range.add(a.range_span).raw()
}

/// Where the guard runs to: on the line from its wounded mate to the target,
/// `InterposeFrom` out from the mate -- and never so near the target that the
/// run's own reach would land on somebody standing still. The run hits what
/// walks into its line; it is not a way to hit you where you stand.
pub fn interpose_point(m: &Monster, mind: &Mind) -> Option<V3> {
    let slot = fight::slot_of(m);
    let mate = fight::pos_of(mind.lore, 1 - slot);
    let to = fight::flat(m.brain.seen.sub(mate));
    let gap = math::wide_flat_len(to);
    let a = SPECIES.attack(INTERPOSE);
    let reach = a
        .hit_x
        .add(a.hit_radius)
        .add(crate::tuning::body_radius())
        .add(Knob::LandShort.fx());
    let out = Knob::InterposeFrom.fx().min(gap.sub(reach));
    if out.raw() <= Knob::LandShort.fx().raw() || gap.raw() <= 0 {
        return None;
    }
    Some(mate.add(math::wide_normalized(to).scale(out)))
}

/// **Where it walks while it is free.**
///
/// - The Holder goes to where the target is going, `prowl_lead` ahead, and
///   stands `HoldDistance` off it: backing away walks you into its lead.
/// - The Striker goes to the far side of the target from its partner --
///   "behind you" by position, never by your camera -- round the side rather
///   than across the front.
/// - Split, the further runs to rejoin; bonded, the wounded hangs back.
/// - Scarred and blind to you, it turns toward its blind side to bring you
///   into the good eye.
pub fn prowl_to(m: &Monster, mind: &Mind) -> Option<V3> {
    let slot = fight::slot_of(m);
    let lore = mind.lore;
    let st = fight::state_of(lore, slot);
    if fight::perched(m) {
        return Some(m.pos);
    }
    let target = m.brain.seen;
    let mate = fight::pos_of(lore, 1 - slot);

    // The scar's habit.
    let side = fight::scar(m);
    if side != 0 && fight::unseen(m) > Knob::ScarSeek.raw() {
        let quarter = if side > 0 {
            math::QUARTER_TURN
        } else {
            math::QUARTER_TURN.neg()
        };
        let blind = V3::from_turns(m.yaw.add(quarter));
        return Some(m.pos.add(blind.scale(Knob::HoldDistance.fx())));
    }

    if fight::going_round(lore, slot) {
        // Round whatever it was stuck on: square off the line to the target,
        // on the side its slot says, so the two do not go the same way.
        let to = fight::flat(target.sub(m.pos));
        let dir = if to.flat_len().raw() > 0 {
            math::wide_normalized(to)
        } else {
            V3::from_turns(m.yaw)
        };
        let side = if slot == 0 {
            V3::new(dir.z.neg(), Fx::ZERO, dir.x)
        } else {
            V3::new(dir.z, Fx::ZERO, dir.x.neg())
        };
        return Some(m.pos.add(side.add(dir).scale(Knob::HoldDistance.fx())));
    }
    if st & state::REJOIN != 0 {
        return Some(mate);
    }
    // **Lost you**: it goes to where it last saw you and, if you are not
    // there, circles it until it has a line to you again. A split lasts as
    // long as it takes to walk round whatever is in the way.
    let lost = fight::unseen(m);
    if lost > Knob::SearchAfter.raw() && st & state::WOUNDED == 0 {
        let round = Knob::SearchTurn
            .fx()
            .mul(Fx::from_int(lost - Knob::SearchAfter.raw()))
            .mul(DT);
        let side = V3::from_turns(round.add(Fx::from_int(slot as i32).mul(math::QUARTER_TURN)));
        return Some(target.add(side.scale(Knob::HoldDistance.fx())));
    }
    if st & state::WOUNDED != 0 {
        // Hanging back, behind its mate from the target.
        let away = fight::flat(mate.sub(target));
        let dir = if away.flat_len().raw() > 0 {
            math::wide_normalized(away)
        } else {
            math::wide_normalized(fight::flat(m.pos.sub(target)))
        };
        return Some(target.add(dir.scale(Knob::WoundedHang.fx())));
    }
    if st & state::SOLO != 0 || fight::enraged(m) {
        return None;
    }
    match fight::role(m) {
        flag::HOLDER => {
            let ahead = target.add(
                m.brain
                    .seen_vel
                    .scale(Fx::from_int(SPECIES.prowl_lead() as i32).mul(DT)),
            );
            let back = fight::flat(m.pos.sub(ahead));
            let dir = if back.flat_len().raw() > 0 {
                math::wide_normalized(back)
            } else {
                V3::from_turns(m.yaw).scale(Fx::ONE.neg())
            };
            Some(ahead.add(dir.scale(Knob::HoldDistance.fx())))
        }
        flag::STRIKER => Some(strike_point(m.pos, target, mate)),
        _ => None,
    }
}

/// The Striker's goal: the far side of the target from its partner, reached
/// round the target at its own distance a step at a time, rather than across
/// the front of it.
pub fn strike_point(me: V3, target: V3, mate: V3) -> V3 {
    let r = Knob::StrikeDistance.fx();
    let from_mate = fight::flat(target.sub(mate));
    let goal_dir = if from_mate.flat_len().raw() > 0 {
        math::wide_normalized(from_mate)
    } else {
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO)
    };
    let mine = fight::flat(me.sub(target));
    if mine.flat_len().raw() <= 0 {
        return target.add(goal_dir.scale(r));
    }
    let a = math::atan2_turns(mine.z, mine.x);
    let g = math::atan2_turns(goal_dir.z, goal_dir.x);
    let error = math::wrap_turns(g.sub(a));
    let step = Knob::BehindAngle.fx().min(error.abs());
    if step.raw() >= error.abs().raw() {
        return target.add(goal_dir.scale(r));
    }
    let turn = if error.raw() > 0 { step } else { step.neg() };
    target.add(V3::from_turns(a.add(turn)).scale(r))
}

/// **As a move commits**: the Holder's coil may be a feint, a perch picks its
/// top, the interpose its place on the line.
pub fn commit(m: &mut Monster, kind: u8, mind: &Mind) {
    let slot = fight::slot_of(m);
    let lore = mind.lore;
    let st = fight::state_of(lore, slot);
    // A leap's mark on open floor, never inside a solid it would reach
    // through (`fight::on_open_floor`).
    if matches!(kind, POUNCE | DIVE) {
        // Stones are not in the brain's window; the frame hook puts the
        // mark on one before anything is live.
        fight::mark_at(m, m.aimed_at(), mind.ground, &[None; crate::stones::MAX_STONES]);
    }
    match kind {
        POUNCE => {
            // **The feint and the ambush are one move made by two animals.**
            // Only ever while the Striker is behind the target and free.
            if fight::role(m) == flag::HOLDER && st & state::MAY_FEINT != 0 {
                let coop = m.own[fight::body::FLAGS] & flag::COOP != 0;
                let strained = fight::below(m, Knob::StrainedHealth.fx(), coop);
                let share = if strained {
                    Knob::FeintStrained.raw()
                } else {
                    Knob::FeintShare.raw()
                };
                if (fight::draw(m) % 100) < share.clamp(0, 100) as u32 {
                    m.doing = crate::monster::Doing::Startup {
                        kind: FEINT,
                        left: SPECIES.attack(FEINT).startup,
                    };
                    m.brain.last_move = FEINT;
                }
            }
        }
        PERCH => {
            if let Some((at, _)) = fight::perch_of(lore, slot) {
                m.aim_at(at);
            }
        }
        AMBUSH => {
            // Aimed where its last look says you are going, and on past it,
            // so the lane is under you and in front of you.
            let a = SPECIES.attack(AMBUSH);
            let at = m.lead_point(a.startup);
            let to = fight::flat(at.sub(m.pos));
            let dir = if to.flat_len().raw() > 0 {
                math::wide_normalized(to)
            } else {
                V3::from_turns(m.yaw)
            };
            let far = math::wide_flat_len(to).add(Knob::AmbushPast.fx());
            m.aim_at(m.pos.add(dir.scale(far)));
        }
        INTERPOSE => {
            if let Some(at) = interpose_point(m, mind) {
                m.aim_at(at);
            }
        }
        _ => {}
    }
}
