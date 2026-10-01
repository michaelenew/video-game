//! What the Mireback wants: the Ridgeback's glance, lead, scoring and draw,
//! with the gallop and the chase taken out and five terms of its own put in.
//!
//! ```text
//! U(m) =  range(m) + arc(m) + rider(m) + variety(m) + hurt(m)   the shared brain
//!       + floor(m)   tar-laying moves, by the clean floor at the lead point
//!       + kindle(m)  the belch, by targets on tar near its mouth, less a lot
//!                    for tar under itself (not in the tide)
//!       + coat(m)    the wallow, by the coat it has lost, on two pools or more
//!       + crowd(m)   the inflate, by what it has taken from close lately
//!       + flee(m)    the flop, aimed away, by fire near the tar it stands in
//! ```
//!
//! It does not chase: it walks to the middle of its own tar near you
//! ([`prowl_to`]), and relocates by flopping. The set-up is tar: a target
//! standing in it is pressed the way a staggered one is.
//!
//! Every term reads the fighters only as the shared brain does -- positions
//! from its last glance -- and the floor, which is a thing it can see. See
//! `docs/design/creatures/mireback.md` §5.

use super::fight::{self, BURNING, SLAG, TAR, word};
use super::{BACKWASH, BELCH, FLOP, INFLATE, Knob, SPEW, TONGUE, WALLOW};
use crate::fixed::Fx;
use crate::hazard::Placed;
use crate::math::{self, V3};
use crate::monster::{Mind, Monster};

/// Is this point standing in a hazard of this kind?
fn in_kind(mind: &Mind, at: V3, kind: u8) -> bool {
    mind.ground
        .floor
        .iter()
        .any(|h| h.kind == kind && h.covers_flat(at))
}

/// The hazards of a kind under its footprint.
fn under(m: &Monster, mind: &Mind, kind: u8) -> usize {
    let reach = fight::foot_radius(m);
    mind.ground
        .floor
        .iter()
        .filter(|h| {
            h.kind == kind && math::wide_flat_dist(h.a, m.pos).raw() < h.radius.add(reach).raw()
        })
        .count()
}

/// How many points round the lead point the floor term looks at.
const AROUND: usize = 8;

/// **Clean floor at the lead point**: of nine points -- the lead and eight
/// round it at the floor term's reach -- how many are on bare floor.
fn clean_at(mind: &Mind, at: V3) -> i32 {
    let reach = Knob::FloorReach.fx();
    let mut clean = i32::from(!covered(mind, at));
    for i in 0..AROUND {
        let dir = V3::from_turns(Fx::from_int(i as i32).div(Fx::from_int(AROUND as i32)));
        if !covered(mind, at.add(dir.scale(reach))) {
            clean += 1;
        }
    }
    clean
}

fn covered(mind: &Mind, at: V3) -> bool {
    mind.ground
        .floor
        .iter()
        .any(|h| matches!(h.kind, TAR | BURNING | SLAG) && h.covers_flat(at))
}

/// **Fire near the tar it stands in**: the nearest burning thing within
/// `flee`'s reach of itself or of a pool under it, if any.
pub fn fire_near(m: &Monster, mind: &Mind) -> Option<V3> {
    let reach = Knob::FleeReach.fx();
    let foot = fight::foot_radius(m);
    let mut best: Option<(V3, Fx)> = None;
    let mine = |f: &Placed| {
        mind.ground.floor.iter().any(|t| {
            t.kind == TAR
                && math::wide_flat_dist(t.a, m.pos).raw() < t.radius.add(foot).raw()
                && math::wide_flat_dist(f.a, t.a)
                    .sub(f.radius)
                    .sub(t.radius)
                    .raw()
                    <= reach.raw()
        })
    };
    for f in mind.ground.floor.iter().filter(|h| h.decl.burns) {
        let from_me = math::wide_flat_dist(f.a, m.pos).sub(f.radius).sub(foot);
        let near_me = from_me.raw() <= reach.raw();
        let near_tar = mine(f);
        if near_me || near_tar {
            let d = math::wide_flat_dist(f.a, m.pos);
            if best.is_none_or(|(_, b)| d.raw() < b.raw()) {
                best = Some((f.a, d));
            }
        }
    }
    best.map(|(at, _)| at)
}

/// Targets standing in tar within the belch's reach of its mouth.
fn on_tar_near_mouth(m: &Monster, mind: &Mind) -> i32 {
    let mouth = fight::mouth(m);
    let reach = Knob::BelchReach.fx();
    mind.quarry
        .iter()
        .filter(|q| q.alive && !q.aboard)
        .filter(|q| {
            mind.ground.floor.iter().any(|h| {
                h.kind == TAR
                    && h.covers_flat(q.pos)
                    && h.flat_gap(mouth).raw() <= h.radius.add(reach).raw()
            })
        })
        .count() as i32
}

/// **Its own terms**, after the shared brain's. See the module docs.
pub fn appetite(m: &Monster, kind: u8, base: i32, mind: &Mind) -> i32 {
    let sp = m.sp();
    let a = sp.attack(kind);
    let lead = m.lead_point(a.startup);
    // The set-up is tar: a target in it cannot leave, and is pressed the way
    // a staggered one is.
    let set_up = in_kind(mind, m.brain.seen, TAR) && !m.brain.seen_stunned;
    let pressed = if set_up && base > 0 && a.damage > 0 && a.aim_cos.raw() > 0 {
        sp.combo_appetite() / 2
    } else {
        0
    };
    match kind {
        SPEW | BACKWASH => {
            if base <= 0 {
                return 0;
            }
            base + pressed + Knob::FloorAppetite.raw() * clean_at(mind, lead)
        }
        FLOP => {
            // Fire near its tar: leap away from it, whatever it is facing.
            if fire_near(m, mind).is_some() {
                return base.max(0) + Knob::FleeAppetite.raw();
            }
            if base <= 0 {
                return 0;
            }
            base + pressed + Knob::FloorAppetite.raw() * clean_at(mind, lead)
        }
        TONGUE => {
            // It will not take a mouthful it saw standing in fire.
            if in_kind(mind, m.brain.seen, BURNING) {
                return 0;
            }
            base + pressed
        }
        BELCH => {
            let targets = on_tar_near_mouth(m, mind);
            if targets == 0 {
                return 0;
            }
            let mut score = base.max(0) + Knob::KindleAppetite.raw() * targets;
            if !fight::tide(m) && under(m, mind, TAR) > 0 {
                score -= Knob::KindleSelf.raw();
            }
            score
        }
        INFLATE => {
            let crowd = mind.lore.int(word::CROWD);
            base + Knob::CrowdAppetite.fx().mul(Fx::from_int(crowd)).to_int()
        }
        WALLOW => {
            if under(m, mind, TAR) < 2 {
                return 0;
            }
            let lost = Fx::ONE.sub(fight::coat(m));
            Fx::from_int(Knob::CoatAppetite.raw()).mul(lost).to_int()
        }
        _ => base,
    }
}

/// **It walks to the middle of its own tar**: the centroid of the tar pools
/// within reach of where it last saw you. With none, it closes on you as any
/// creature does.
pub fn prowl_to(m: &Monster, mind: &Mind) -> Option<V3> {
    // **Fire near it first**: it walks away from the nearest burning thing,
    // the slow way out when the flop is not ready.
    if let Some(fire) = fire_near(m, mind) {
        let away = V3::new(m.pos.x.sub(fire.x), Fx::ZERO, m.pos.z.sub(fire.z));
        if away.flat_len().raw() > 0 {
            return Some(
                m.pos
                    .add(math::wide_normalized(away).scale(Knob::FleeReach.fx())),
            );
        }
    }
    let reach = Knob::TarReach.fx();
    let mut sum = V3::ZERO;
    let mut weight = Fx::ZERO;
    for h in mind.ground.floor.iter() {
        if h.kind != TAR || math::wide_flat_dist(h.a, m.brain.seen).raw() > reach.raw() {
            continue;
        }
        let w = h.radius;
        sum = sum.add(h.a.scale(w));
        weight = weight.add(w);
    }
    if weight.raw() <= 0 {
        return None;
    }
    Some(sum.scale(Fx::ONE.div(weight)))
}

/// As a move commits: a flop fleeing fire leaps away from it, and every
/// lobbed move lands inside the arena.
pub fn commit(m: &mut Monster, kind: u8, mind: &Mind) {
    if kind == FLOP {
        if let Some(fire) = fire_near(m, mind) {
            let a = m.sp().attack(FLOP);
            let away = V3::new(m.pos.x.sub(fire.x), Fx::ZERO, m.pos.z.sub(fire.z));
            let dir = if away.flat_len().raw() > 0 {
                math::wide_normalized(away)
            } else {
                V3::from_turns(m.yaw).scale(Fx::ONE.neg())
            };
            m.aim_at(m.pos.add(dir.scale(a.ideal_range.add(a.range_span))));
        }
    }
    if matches!(kind, SPEW | FLOP) {
        let b = mind.ground.bounds;
        let keep = if kind == FLOP {
            m.sp().margin()
        } else {
            fight::foot_radius(m).min(m.sp().margin())
        };
        let at = m.aimed_at();
        m.aim_at(V3::new(
            at.x.clamp(b.lo_x.add(keep), b.hi_x.sub(keep)),
            Fx::ZERO,
            at.z.clamp(b.lo_z.add(keep), b.hi_z.sub(keep)),
        ));
    }
}
