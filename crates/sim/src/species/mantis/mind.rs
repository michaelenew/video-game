//! What the Mantis wants: its own terms in the scoring (`mantis.md` §5), where
//! it walks, and what it decides as a move commits.
//!
//! ```text
//! U(m) =  range(m) + arc(m) + variety(m) + hurt(m)      as the Ridgeback
//!       + seen(m)     a seen commitment: guard if blockable and reaches;
//!                     the Leap if the hunter is aloft
//!       + punish(m)   a seen recovery longer than m's startup plus travel
//!       + back(m)     pivot and flare: frames outside the arc within 4 m
//!       + far(m)      prayer: a hunter past 9 m, or nothing committed for 90 f
//!       + ready(m)    Ready only (habit::ready_score), or guess(m) without
//! ```
//!
//! **Everything here reads what it saw, late**: the shared brain's sample
//! (`Brain::seen`, which its eyes fill through the delay line) and the
//! record in the lore (`sight::glimpse`, `sight::deed_seen`). `Mind::quarry`
//! -- the present -- is never read.

use crate::DT;
use crate::fixed::Fx;
use crate::math::{self, V3};
use crate::monster::{Mind, Monster};
use crate::state::MAX_PLAYERS;

use super::fight::{self, body};
use super::habit;
use super::sight::{self, Deed};
use super::{DIVE, FLARE, GUARD, Knob, LEAP, LUNGE, PIVOT, PRAYER, READY, SLASH, SPECIES};

/// Its generator, advanced from inside the tick.
pub fn draw(m: &mut Monster) -> u32 {
    let mut x = m.brain.rng;
    if x == 0 {
        x = 0x2545_F491;
    }
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    m.brain.rng = x;
    x
}

/// **Its own terms**, after the shared ones. See the module docs.
pub fn appetite(m: &Monster, kind: u8, base: i32, mind: &Mind) -> i32 {
    let lore = mind.lore;
    let t = sight::now(lore);
    let target = (m.brain.target as usize).min(MAX_PLAYERS - 1);
    let range = math::wide_flat_dist(m.brain.seen, m.pos);
    let hurt = fight::hurt(m);
    let desperate = fight::desperate(m);
    let attack = |s: i32| {
        if hurt || desperate {
            Fx::from_int(s).mul(Knob::HurtAttack.fx()).to_int()
        } else {
            s
        }
    };
    match kind {
        GUARD => {
            if !fight::may_guard(m) {
                return 0;
            }
            let mut s = base;
            // In neutral, at the edge of its reach, it holds its guard up.
            if range.raw() <= Knob::StandOff.fx().add(Knob::BackNear.fx()).raw()
                && fight::inside(m, m.brain.seen)
            {
                s += Knob::GuardAppetite.raw();
            }
            // seen(m): a blockable commitment coming at it.
            if fight::coming(m, lore, t)
                .iter()
                .flatten()
                .any(|c| !c.unblockable)
            {
                s += Knob::SeenAppetite.raw();
            }
            if hurt {
                s = Fx::from_int(s).mul(Knob::HurtGuard.fx()).to_int();
            }
            s
        }
        SLASH | LUNGE => attack(base + punish(kind, lore, t, target)),
        LEAP => {
            // seen(m): anybody aloft within reach of its rising cut.
            let aloft = (0..MAX_PLAYERS).any(|who| {
                sight::glimpse(lore, who, t).is_some_and(|g| {
                    g.alive
                        && g.aloft
                        && math::wide_flat_dist(g.pos, m.pos).raw() <= Knob::LeapCalled.fx().raw()
                })
            });
            attack(base + if aloft { Knob::SeenAppetite.raw() } else { 0 })
        }
        FLARE | PIVOT => {
            let mut s = base + back(m, lore);
            if kind == FLARE && range.raw() <= Knob::BackNear.fx().raw() / 2 {
                s += Knob::BackAppetite.raw() / 2;
            }
            attack(s)
        }
        PRAYER => {
            if !fight::may_guard(m) {
                return 0;
            }
            let far = (0..MAX_PLAYERS).all(|who| {
                sight::glimpse(lore, who, t).is_none_or(|g| {
                    !g.alive || math::wide_flat_dist(g.pos, m.pos).raw() > Knob::FarRange.fx().raw()
                })
            });
            if far || fight::quiet(lore) as i32 >= Knob::FarIdle.raw() {
                base + Knob::FarAppetite.raw()
            } else {
                0
            }
        }
        READY => {
            if !fight::may_guard(m) {
                return 0;
            }
            if Knob::Habit.raw() != 0 {
                habit::ready_score(m, lore, t)
            } else {
                // guess(m): something of theirs reaches it, and it guesses.
                let reaches = habit::guess(m, lore, t, 0).is_some();
                if reaches && range.raw() <= Knob::StandOff.fx().raw() {
                    Knob::GuessWeight.raw()
                } else {
                    0
                }
            }
        }
        _ => base,
    }
}

/// **`punish(m)`**: what it saw its target doing is a recovery -- of a move,
/// or the tail of a dodge -- with more of it left than this move takes to
/// arrive.
fn punish(kind: u8, lore: &crate::lore::Lore, t: u32, who: usize) -> i32 {
    let (deed, began) = sight::deed_seen(lore, who, t);
    let ends = match deed {
        Deed::Move(k) => {
            let Some(class) = fight::class_of(lore, who) else {
                return 0;
            };
            let mv = crate::moves::get(class, k);
            began.wrapping_add((mv.startup + mv.active + mv.recovery) as u32)
        }
        Deed::Dodge => began.wrapping_add(crate::tuning::dodge_frames() as u32),
        _ => return 0,
    };
    let left = ends.wrapping_sub(t) as i32;
    if left <= 0 {
        return 0;
    }
    let a = SPECIES.attack(kind);
    let arrives = if kind == LUNGE {
        Knob::CoilMin.raw() + a.active as i32
    } else {
        a.startup as i32
    };
    if left > arrives {
        Knob::PunishAppetite.raw()
    } else {
        0
    }
}

/// **`back(m)`**: the longest any fighter has stood outside its arc within
/// `BackNear`, over its patience (halved with two hunters), times
/// `BackAppetite`, up to twice it.
fn back(m: &Monster, lore: &crate::lore::Lore) -> i32 {
    let mut patience = Knob::BackPatience.raw().max(1);
    if fight::coop(m) {
        patience = Fx::from_int(patience)
            .mul(Knob::CoopPatience.fx())
            .to_int()
            .max(1);
    }
    let most = (0..MAX_PLAYERS)
        .map(|who| fight::back_frames(lore, who))
        .max()
        .unwrap_or(0);
    let worth = (Knob::BackAppetite.raw() as i64 * most as i64 / patience as i64) as i32;
    worth.min(Knob::BackAppetite.raw() * 2)
}

/// **Where it walks**: with two hunters, `pair(m)` -- if the angle between
/// them, from it, is wider than `PairAngle`, it backs away from both, so
/// that both fall inside its arc (§8). Otherwise at its target, as the
/// shared walk does.
pub fn prowl_to(m: &Monster, mind: &Mind) -> Option<V3> {
    if !fight::coop(m) {
        return None;
    }
    let lore = mind.lore;
    let t = sight::now(lore);
    let a = sight::glimpse(lore, 0, t).filter(|g| g.alive)?;
    let b = sight::glimpse(lore, 1, t).filter(|g| g.alive)?;
    let to_a = math::wide_normalized(fight::flat(a.pos.sub(m.pos)));
    let to_b = math::wide_normalized(fight::flat(b.pos.sub(m.pos)));
    let cos = to_a.dot(to_b);
    if cos.raw() >= fight::cos_turns(Knob::PairAngle.fx()).raw() {
        return None;
    }
    let between = to_a.add(to_b);
    let away = if between.flat_len().raw() > 0 {
        math::wide_normalized(between).scale(Fx::ONE.neg())
    } else {
        V3::from_turns(m.yaw.add(math::QUARTER_TURN))
    };
    Some(m.pos.add(away.scale(Knob::StandOff.fx())))
}

/// **As a move commits**: the Ready's move and the side its blade is cocked
/// to; the prayer's haste; the dive's mark, held from the frame the leap left
/// the floor.
pub fn commit(m: &mut Monster, kind: u8, mind: &Mind) {
    let lore = mind.lore;
    let t = sight::now(lore);
    match kind {
        READY => {
            let roll = draw(m);
            let code = if Knob::Habit.raw() != 0 {
                habit::majority(lore, t).map(|(c, _, _)| c)
            } else {
                habit::guess(m, lore, t, roll)
            };
            m.own[body::READY_FOR] = code.unwrap_or(0);
            // The blade is cocked on the side the move will come from:
            // where it sees its thrower.
            let right = V3::from_turns(m.yaw.add(math::QUARTER_TURN));
            m.brain.mirror = right.dot(fight::flat(m.brain.seen.sub(m.pos))).raw() < 0;
        }
        DIVE => m.aim_at(fight::dive_at(lore)),
        GUARD => {}
        _ => fight::haste(m, kind),
    }
    let _ = DT;
}
