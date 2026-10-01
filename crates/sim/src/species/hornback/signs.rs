//! What the herd draws on the floor besides its bodies' own telegraphs
//! (hornback.md §6), through `FightDecl::signs`:
//!
//! - **The bellow's lane**, eight metres wide from the herd's far end to the
//!   arena's edge, filling through the call and red while the herd runs it --
//!   and **the lee of every solid in it, drawn clear**: the same rectangles the
//!   rules hold the cows out of (`rules::stampede`), so what is drawn is where
//!   the herd can be.
//! - **The solid a charge will stop at**, ringed in the "this will stop it"
//!   colour, while the lane is drawn cut short at it (the lane itself is the
//!   bull's own telegraph, `Mind::telegraph`).
//! - **The guard**, in the *no* colour across the bull's front -- half of it
//!   with one horn gone.
//! - **A cow's rear wedge**, faint, under anybody standing in it.
//! - **The hook's horn**: the side the horn comes from, brighter.

use super::mind::{self, HerdState, bellow_lane, herd_state, horns_whole};
use super::rules::{self, Stopper};
use super::{BULL, COW, GUARD, HOOK, Knob, STAMPEDE, knob_fx};
use crate::critter::{CritterField, is, stat_fx};
use crate::fixed::Fx;
use crate::math::V3;
use crate::sign::{Says, Sign, Signs};
use crate::state::World;

/// The species' signs. See the module docs.
pub fn signs(w: &World, out: &mut Signs) {
    let Some(pack) = w.pack else { return };
    if pack.species != super::SPECIES.id {
        return;
    }
    let sp = &super::SPECIES;
    let ground = w.terrain();

    // ---- the stampede's lane, and its lees ----
    if herd_state(&pack) == HerdState::Stampede {
        if let Some((at, dir, len, width)) = bellow_lane(&pack) {
            // Filling through the bellow, red once the herd runs.
            let running = w
                .critters
                .iter()
                .any(|c| c.kind == COW && c.alive() && c.act == STAMPEDE && c.state == is::ACTIVE);
            let fill = mind::the_bull(&w.critters)
                .map(|b| w.critters[b])
                .filter(|b| b.state == is::STARTUP && b.act == super::BELLOW)
                .map_or(Fx::ONE, |b| {
                    let startup = sp.attack(super::BELLOW).startup.max(1) as i32;
                    Fx::ONE.sub(Fx::ratio(b.timer as i32, startup))
                });
            let says = if running { Says::Live } else { Says::Coming };
            out.push(Sign::strip(says, at, dir, len, width).filled(fill));
            let (blocks, n) = rules::obstacles(w, &ground);
            let half = width.mul(Fx::ratio(1, 2));
            for (min, max) in blocks[..n].iter().flatten() {
                let (lo_a, hi_a, lo_c, hi_c) = mind::shadow(at, dir, *min, *max);
                // Only what stands in the lane casts a lee in it.
                if hi_a.raw() < 0
                    || lo_a.raw() > len.raw()
                    || hi_c.raw() < half.neg().raw()
                    || lo_c.raw() > half.raw()
                {
                    continue;
                }
                let lo_c = lo_c.max(half.neg());
                let hi_c = hi_c.min(half);
                let mid = lo_c.add(hi_c).mul(Fx::ratio(1, 2));
                let from = rules::lane_point(at, dir, lo_a.max(Fx::ZERO), mid);
                let to = hi_a.add(knob_fx(Knob::LeeLength)).min(len);
                out.push(Sign::strip(
                    Says::Clear,
                    from,
                    dir,
                    to.sub(lo_a.max(Fx::ZERO)),
                    hi_c.sub(lo_c),
                ));
            }
        }
    }

    let Some(b) = mind::the_bull(&w.critters) else {
        return;
    };
    let bull = w.critters[b];
    if !bull.alive() {
        return;
    }

    // ---- the charge's stop ----
    if bull.act == super::CHARGE && bull.attacking() {
        let (end, _, solid) = mind::charge_lane(&pack);
        if solid {
            let from = rules::nose(&bull);
            let to = end.sub(from);
            let reach = to.flat_len().add(Fx::ONE);
            let hit = (to.flat_len().raw() > 0)
                .then(|| rules::first_stopper(w, &ground, from, to.normalized(), reach))
                .flatten();
            if let Some((_, stop)) = hit {
                let (at, across) = match stop {
                    Stopper::Arena(s) | Stopper::Boulder(_, s) | Stopper::Raised(s) => (
                        stop.centre(),
                        s.max.x.sub(s.min.x).max(s.max.z.sub(s.min.z)),
                    ),
                    Stopper::Stone(_, at, r) => (V3::new(at.x, Fx::ZERO, at.z), r.add(r)),
                };
                // A wall as long as the bank is ringed where the lane meets it.
                let at = if across.raw() > knob_fx(Knob::LaneWidth).raw() {
                    end
                } else {
                    at
                };
                let across = across.min(knob_fx(Knob::LaneWidth));
                out.push(Sign::ring(Says::Stops, at, across.add(Fx::ONE)));
            }
        }
    }

    // ---- the guard ----
    if bull.act == GUARD && bull.attacking() {
        let whole = horns_whole(&pack);
        let head = mind::head(sp, &bull);
        let fwd = bull.facing();
        let left = V3::new(fwd.z.neg(), Fx::ZERO, fwd.x);
        let span = knob_fx(Knob::HeadSpan).add(knob_fx(Knob::GuardSees));
        let depth = knob_fx(Knob::GuardSees);
        let nose = V3::new(head.foot.x, Fx::ZERO, head.foot.z);
        let quarter = span.mul(Fx::ratio(1, 2)).mul(Fx::ratio(1, 2));
        let says = if bull.state == is::ACTIVE {
            Says::No
        } else {
            Says::Coming
        };
        match whole {
            [true, true] => out.push(Sign::strip(says, nose, fwd, depth, span)),
            [true, false] => out.push(Sign::strip(
                says,
                nose.add(left.scale(quarter)),
                fwd,
                depth,
                span.mul(Fx::ratio(1, 2)),
            )),
            [false, true] => out.push(Sign::strip(
                says,
                nose.sub(left.scale(quarter)),
                fwd,
                depth,
                span.mul(Fx::ratio(1, 2)),
            )),
            [false, false] => {}
        }
    }

    // ---- the hook's horn ----
    if bull.act == HOOK && bull.state == is::STARTUP {
        if let Some(t) = crate::pack::telegraph(Some(&pack), &w.critters, b) {
            let fwd = bull.facing();
            let left = V3::new(fwd.z.neg(), Fx::ZERO, fwd.x);
            let right = bull.role & mind::bull::RIGHT != 0;
            let side = if right {
                left.scale(Fx::ONE.neg())
            } else {
                left
            };
            let at = V3::new(t.anchor.x, Fx::ZERO, t.anchor.z)
                .add(side.scale(t.radius.mul(Fx::ratio(1, 2))));
            out.push(Sign::disc(Says::Coming, at, t.radius).filled(t.progress));
        }
    }

    // ---- cows' rear wedges, under anybody standing in one ----
    let reach = knob_fx(Knob::KickReach);
    for c in w.critters.iter().filter(|c| c.kind == COW && c.alive()) {
        let back = c.facing().scale(Fx::ONE.neg());
        let half = stat_fx(sp, COW, CritterField::Length).mul(Fx::ratio(1, 2));
        let rump = c.pos.add(back.scale(half));
        let rump = V3::new(rump.x, Fx::ZERO, rump.z);
        if rules::in_wedge(c, w.players.iter().filter(|p| p.health > 0).map(|p| p.pos)) {
            let wide = reach.mul(Fx::ratio(1, 2)).add(Fx::ratio(1, 2));
            out.push(Sign::strip(Says::Faint, rump, back, reach, wide));
        }
    }
    let _ = BULL;
}
