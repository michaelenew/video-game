//! What the Sandmaw wants: the Ridgeback's scoring and draw, with what it
//! glances at taken away and its own terms put in (§5).
//!
//! ```text
//! U(m) =  heard(m)    a fresh noise for the rise-bite, a trail for the
//!                     breach, a body it feels for the undertow
//!       + range(m)    the rise reaches so far; the breach's lane must cover
//!                     where the trail leads
//!       + posture(m)  moves from below only buried, standing moves only
//!                     standing (and those are the shared brain's range and
//!                     bearing tents, at what it attended)
//!       + island(m)   surface at an island's edge and spit, once
//!                     `island_patience` has run out on a noise on rock
//!       + variety(m), hurt(m), cooldown(m)   the shared brain's
//! ```
//!
//! **It never takes a position from a body it did not hear or feel.**
//! Everything here reads the record its frame hook wrote at its last glance
//! ([`fight::attended`]) -- a noise, or a body it felt -- and the noise ring;
//! the fighters' positions it is handed as `Quarry` it does not read at all.
//! `tests/sandmaw.rs` holds it to that.

use super::fight::{self, Attended, word};
use super::{BREACH, Knob, LASH, RISE, SOUND, SPIT, SWALLOW, UNDERTOW};
use crate::fixed::Fx;
use crate::math::{self, V3};
use crate::monster::{Mind, Monster};

/// What it attended, if it is fresh enough to act on.
fn fresh(m: &Monster, mind: &Mind) -> Option<Attended> {
    let now = fight::now(mind.lore);
    let _ = m;
    fight::attended(mind.lore)
        .filter(|a| now.wrapping_sub(a.frame) <= Knob::HeardFor.raw().max(0) as u32)
}

/// Has it been waiting at an island long enough to come up beside it?
fn island_ready(mind: &Mind, a: &Attended) -> bool {
    let since = mind.lore.word(word::ISLAND_SINCE);
    a.on_rock
        && a.heard()
        && since != 0
        && fight::now(mind.lore).wrapping_sub(since) >= Knob::IslandPatience.raw().max(0) as u32
}

/// **Where a rise would come up, if it can**: beside the island when it has
/// waited there long enough; under the noise, kept off rock, otherwise.
pub fn rise_at(m: &Monster, mind: &Mind) -> Option<V3> {
    let a = fight::attended(mind.lore)?;
    if island_ready(mind, &a) {
        return fight::island_edge(m, mind.ground, a.at);
    }
    let a = fresh(m, mind).filter(|a| a.heard() && !a.on_rock)?;
    fight::fit(m, mind.ground, a.at)
}

/// **Where the breach's lane is laid**: the trail's newest noise, led by
/// `trail_lead` of its velocity -- if what it attended is a trail.
pub fn breach_at(m: &Monster, mind: &Mind) -> Option<V3> {
    let a = fresh(m, mind).filter(|a| a.heard())?;
    let (last, vel) = fight::trail(mind.lore, a.who)?;
    let lead = Fx::from_int(Knob::TrailLead.raw()).mul(crate::DT);
    let at = last.add(vel.scale(lead));
    let b = mind.ground.bounds;
    let room = m.sp().margin();
    let inside = at.x.raw() >= b.lo_x.add(room).raw()
        && at.x.raw() <= b.hi_x.sub(room).raw()
        && at.z.raw() >= b.lo_z.add(room).raw()
        && at.z.raw() <= b.hi_z.sub(room).raw();
    (inside && fight::on_sand(mind.ground, at)).then_some(at)
}

/// A tent: one at `ideal`, nothing past `span` either side.
fn tent(d: Fx, ideal: Fx, span: Fx) -> Fx {
    if span.raw() <= 0 {
        return Fx::ZERO;
    }
    Fx::ONE
        .sub(d.sub(ideal).abs().div(span))
        .clamp(Fx::ZERO, Fx::ONE)
}

/// **Its own terms**, in place of the shared brain's for the moves from
/// below and after it for the standing ones. See the module docs.
pub fn appetite(m: &Monster, kind: u8, base: i32, mind: &Mind) -> i32 {
    // It decides the frame after a glance, once what it took has been
    // written down: a decision on the glance's own frame would act on the
    // glance before.
    if m.brain.glance_left == m.glance_frames() {
        return 0;
    }
    let sp = m.sp();
    match kind {
        RISE => {
            if !fight::under(m) {
                return 0;
            }
            let Some(at) = rise_at(m, mind) else {
                return 0;
            };
            let d = math::wide_flat_dist(at, fight::head_flat(m));
            if d.raw() > Knob::RiseReach.fx().raw() {
                return 0;
            }
            let island = fight::attended(mind.lore).is_some_and(|a| island_ready(mind, &a));
            Knob::RiseAppetite.raw()
                + if island {
                    Knob::IslandAppetite.raw()
                } else {
                    0
                }
        }
        BREACH => {
            if !fight::under(m) {
                return 0;
            }
            let Some(at) = breach_at(m, mind) else {
                return 0;
            };
            let a = sp.attack(BREACH);
            let d = math::wide_flat_dist(at, m.pos);
            Fx::from_int(Knob::BreachAppetite.raw())
                .mul(tent(d, a.ideal_range, a.range_span))
                .to_int()
        }
        UNDERTOW => {
            if !fight::under(m) {
                return 0;
            }
            let felt = fresh(m, mind).is_some_and(|a| {
                a.felt() && fight::now(mind.lore).wrapping_sub(a.frame) <= m.glance_frames() as u32
            });
            if felt {
                Knob::UndertowAppetite.raw()
            } else {
                0
            }
        }
        SPIT | LASH | SWALLOW => {
            let Some(a) = fresh(m, mind) else {
                return 0;
            };
            if !fight::standing(m) || base <= 0 {
                return 0;
            }
            let island = if kind == SPIT && a.on_rock {
                Knob::IslandAppetite.raw()
            } else {
                0
            };
            base + Knob::FrontAppetite.raw().min(base) + island
        }
        SOUND => {
            if fight::standing(m) && m.own[fight::body::UP_FOR] >= Knob::SurfaceMax.raw() {
                Knob::SoundAppetite.raw()
            } else {
                0
            }
        }
        _ => 0,
    }
}

/// **Where it swims when it is free**: standing, nowhere; buried, round the
/// last thing it attended until `silence_patience` runs out, and then a
/// widening spiral out from there -- the search.
pub fn prowl_to(m: &Monster, mind: &Mind) -> Option<V3> {
    if !fight::under(m) {
        return Some(m.pos);
    }
    let now = fight::now(mind.lore);
    let Some(a) = fight::attended(mind.lore) else {
        let home = mind.lore.word(word::GRACE_AT);
        let at = V3::new(
            crate::lore::from_cm(crate::lore::lo(home)),
            Fx::ZERO,
            crate::lore::from_cm(crate::lore::hi(home)),
        );
        return Some(orbit(m, at, Knob::CircleRadius.fx()));
    };
    let patience = if fight::hungry(m) {
        Knob::HungerPatience.raw()
    } else {
        Knob::SilencePatience.raw()
    }
    .max(0) as u32;
    let since = now.wrapping_sub(a.frame);
    if since < patience || a.on_rock {
        return Some(orbit(m, a.at, Knob::CircleRadius.fx()));
    }
    Some(spiral(m, mind, a.at))
}

/// A point a little way round a circle from where it is now: what it swims
/// at to go round it.
fn orbit(m: &Monster, centre: V3, r: Fx) -> V3 {
    let from = V3::new(m.pos.x.sub(centre.x), Fx::ZERO, m.pos.z.sub(centre.z));
    let at = if math::wide_flat_len(from).raw() > 0 {
        math::atan2_turns(from.z, from.x)
    } else {
        m.yaw
    };
    centre.add(V3::from_turns(at.add(Knob::SearchLook.fx())).scale(r))
}

/// **The search**: a spiral out from the last noise, wider by `SearchPitch`
/// a turn. Out past the arena, it starts again from the middle of it.
fn spiral(m: &Monster, mind: &Mind, centre: V3) -> V3 {
    let from = V3::new(m.pos.x.sub(centre.x), Fx::ZERO, m.pos.z.sub(centre.z));
    let r = math::wide_flat_len(from);
    let look = Knob::SearchLook.fx();
    let wider = r
        .add(Knob::SearchPitch.fx().mul(look))
        .max(Knob::CircleRadius.fx());
    let b = mind.ground.bounds;
    let reach = math::half(b.hi_x.sub(b.lo_x)).min(math::half(b.hi_z.sub(b.lo_z)));
    if wider.raw() > reach.raw() {
        let middle = V3::new(
            math::half(b.lo_x.add(b.hi_x)),
            Fx::ZERO,
            math::half(b.lo_z.add(b.hi_z)),
        );
        return orbit(m, middle, Knob::CircleRadius.fx());
    }
    let at = if r.raw() > 0 {
        math::atan2_turns(from.z, from.x)
    } else {
        m.yaw
    };
    centre.add(V3::from_turns(at.add(look)).scale(wider))
}

/// **As a move commits**: the rise is aimed at the noise, kept off rock (or
/// beside the island); the breach's lane is laid along the trail's lead and
/// it turns to it; the undertow is under the body it felt.
pub fn commit(m: &mut Monster, kind: u8, mind: &Mind) {
    match kind {
        RISE => {
            if let Some(at) = rise_at(m, mind) {
                m.aim_at(at);
            }
        }
        BREACH => {
            if let Some(at) = breach_at(m, mind) {
                m.aim_at(at);
                let to = at.sub(m.pos);
                if math::wide_flat_len(to).raw() > 0 {
                    m.yaw = math::atan2_turns(to.z, to.x);
                    m.yaw_rate = Fx::ZERO;
                }
            }
        }
        UNDERTOW => {
            if let Some(a) = fight::attended(mind.lore) {
                m.aim_at(a.at);
            }
        }
        _ => {}
    }
}
