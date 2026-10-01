//! The Gnawers' own rules over the whole world: the three things they do that
//! touch more than the pack (gnawers.md §2, §3).
//!
//! Run once a frame by the shared machinery (`FightDecl::frame`), after the
//! pack has stepped and before the fighters do:
//!
//! - **The latch.** A hamstring that lands holds on (`mind::role::LATCHED`):
//!   the gnawer rides the fighter's heels, bites for `LatchDamage` every
//!   `LatchEvery` frames and keeps the slow on, until a dodge sheds it, a hit
//!   knocks it off, or it lets go after `LatchMost`. It keeps its token while
//!   it holds -- a latched gnawer is one of the two that may bite.
//! - **The gnaw.** Somebody standing on a raised stone: up to `GnawDiggers`
//!   gnawers walk to its foot and dig. Each digging gnawer is a digger-frame;
//!   `GnawWork` of them fells the stone (three in 120 frames, two in 180), and
//!   whoever was on it lands among them staggered. The one path into a stone
//!   besides Cataclysm, and it goes through `stones::destroy` as that does.
//! - **The scramble.** A gnawer whose scramble comes out is put on top of the
//!   platform it clung to, just inside the edge.

use super::mind::{role, settle, word};
use super::{GNAW, HAMSTRING, Knob, SCRAMBLE, knob, knob_fx};
use crate::class::{MAX_STRUCTURES, Mechanic};
use crate::critter::{CritterField, MAX_CRITTERS, flag, is, stat, stat_fx};
use crate::fixed::Fx;
use crate::math::V3;
use crate::pack::{give_back, mood};
use crate::state::{Action, MAX_PLAYERS, World};
use crate::stones::MAX_STONES;
use crate::tuning as t;

/// The species' frame. See the module docs.
pub fn frame(w: &mut World) {
    let Some(mut pack) = w.pack else { return };
    if pack.species != super::SPECIES.id {
        return;
    }
    let sp = pack.sp();
    for c in w.critters.iter_mut() {
        settle(c);
    }

    // ---- the latch ----
    for i in 0..MAX_CRITTERS {
        let c = w.critters[i];
        if c.role & role::LATCHED == 0 {
            continue;
        }
        let holding =
            c.alive() && c.act == HAMSTRING && matches!(c.state, is::ACTIVE | is::RECOVERY);
        let who = (c.target as usize).min(MAX_PLAYERS - 1);
        let p = w.players[who];
        let steps = (c.role / role::LATCH_STEP) as i32;
        let held = steps * 16;
        let shaken = matches!(p.action, Action::Dodge { .. })
            || p.health <= 0
            || held >= knob(Knob::LatchMost)
            || pack.mood != mood::HUNTING;
        let body = &mut w.critters[i];
        if !holding {
            body.role &= !(role::LATCHED | 0xF0);
            continue;
        }
        if shaken {
            // Shed: thrown off behind the fighter, its token back.
            body.role &= !(role::LATCHED | 0xF0);
            give_back(&mut pack, body);
            body.state = is::FLINCH;
            body.timer = stat(sp, body.kind, CritterField::FlinchFrames).max(1) as u16;
            body.vel = V3::ZERO;
            continue;
        }
        // Holding: on the heels, facing the way the fighter faces.
        let behind =
            t::body_radius().add(stat_fx(sp, body.kind, CritterField::Length).mul(Fx::ratio(1, 2)));
        let heel = p.pos.sub(p.facing.scale(behind));
        body.pos = V3::new(heel.x, p.pos.y.max(Fx::ZERO), heel.z);
        body.vel = p.vel;
        body.yaw = crate::pack::yaw_of(p.facing);
        body.state = is::RECOVERY;
        body.timer = sp.attack(HAMSTRING).recovery.max(2);
        if body.clock % 16 == 0 && steps < 15 {
            body.role += role::LATCH_STEP;
        }
        let every = knob(Knob::LatchEvery).max(1) as u16;
        if body.clock % every == 0 {
            let target = &mut w.players[who];
            target.wound(knob(Knob::LatchDamage));
            target.slow(
                knob(Knob::HamstringFrames).max(0) as u16,
                knob_fx(Knob::HamstringSlow),
            );
        }
    }

    // ---- the scramble ----
    for c in w.critters.iter_mut() {
        if !(c.alive() && c.state == is::ACTIVE && c.act == SCRAMBLE) || c.has(flag::HIT_USED) {
            continue;
        }
        c.set(flag::HIT_USED, true);
        let half_wid = stat_fx(sp, c.kind, CritterField::Width).mul(Fx::ratio(1, 2));
        let inset = half_wid.add(crate::arena::SKIN);
        let top = w
            .arena
            .get()
            .solids()
            .iter()
            .filter(|s| {
                s.max.y.raw() <= knob_fx(Knob::ScrambleTop).raw()
                    && s.max.y.raw() > Fx::ratio(1, 2).raw()
                    && s.min.y.raw() <= crate::arena::SKIN.raw()
                    && s.max.x.sub(s.min.x).raw() > inset.add(inset).raw()
                    && s.max.z.sub(s.min.z).raw() > inset.add(inset).raw()
            })
            .min_by_key(|s| super::mind::off_footprint(s, c.pos).raw())
            .copied();
        let Some(s) = top else { continue };
        if super::mind::off_footprint(&s, c.pos).raw() > knob_fx(Knob::ScrambleFrom).raw() {
            continue;
        }
        let x = c.pos.x.clamp(s.min.x.add(inset), s.max.x.sub(inset));
        let z = c.pos.z.clamp(s.min.z.add(inset), s.max.z.sub(inset));
        c.pos = V3::new(x, s.max.y, z);
        c.vel = V3::ZERO;
        c.set(flag::AIRBORNE, false);
    }

    // ---- the gnaw ----
    gnaw(w, &mut pack);

    w.pack = Some(pack);
}

/// Who is standing on which stone: the fighter on top of each, if any. Only
/// the Elementalist's raised stones -- a planted shield is a wall, not a
/// floor anybody stands on.
fn stood_on(w: &World) -> [Option<usize>; MAX_STONES] {
    let field = crate::stones::gather(&w.players);
    let mut on = [None; MAX_STONES];
    for (index, stone) in field.iter().enumerate() {
        let Some(stone) = stone else { continue };
        let owner = index / MAX_STRUCTURES;
        if !matches!(w.players[owner].mechanic, Mechanic::Structures(_)) {
            continue;
        }
        if stone.standing_height().raw() <= 0 {
            continue;
        }
        for (who, p) in w.players.iter().enumerate() {
            let flat = V3::new(p.pos.x.sub(stone.at.x), Fx::ZERO, p.pos.z.sub(stone.at.z));
            let up = p.pos.y.raw()
                >= stone
                    .top()
                    .sub(crate::arena::SKIN.add(crate::arena::SKIN))
                    .raw();
            if p.health > 0 && p.grounded && up && flat.flat_len().raw() <= stone.radius().raw() {
                on[index] = Some(who);
            }
        }
    }
    on
}

fn gnaw(w: &mut World, pack: &mut crate::pack::Pack) {
    let sp = pack.sp();
    let on = stood_on(w);
    let field = crate::stones::gather(&w.players);
    let current = pack.memo[word::GNAW_STONE] - 1;
    let release = |w: &mut World, pack: &mut crate::pack::Pack| {
        for c in w.critters.iter_mut() {
            if c.role & role::DIGGER != 0 {
                c.role &= !role::DIGGER;
                if c.alive() && c.act == GNAW && c.attacking() {
                    c.state = is::RECOVERY;
                    c.timer = sp.attack(GNAW).recovery.max(1);
                }
            }
        }
        pack.memo[word::GNAW_STONE] = 0;
        pack.memo[word::GNAW_WORK] = 0;
    };

    // The stone it was at has gone, or nobody is on it, or the pack is not
    // hunting: the diggers stop.
    let still = current >= 0
        && (current as usize) < MAX_STONES
        && field[current as usize].is_some()
        && on[current as usize].is_some()
        && pack.mood == mood::HUNTING;
    if current >= 0 && !still {
        release(w, pack);
    }
    if pack.memo[word::GNAW_STONE] == 0 {
        if pack.mood != mood::HUNTING || pack.grace > 0 {
            return;
        }
        let Some(index) = (0..MAX_STONES).find(|i| on[*i].is_some()) else {
            return;
        };
        let Some(stone) = field[index] else { return };
        pack.memo[word::GNAW_STONE] = index as i32 + 1;
        pack.memo[word::GNAW_WORK] = 0;
        pack.memo[word::GNAW_X] = stone.at.x.raw();
        pack.memo[word::GNAW_Z] = stone.at.z.raw();
    }
    let index = (pack.memo[word::GNAW_STONE] - 1) as usize;
    let Some(stone) = field[index] else { return };

    // Diggers: the nearest free gnawers, up to the cap.
    let cap = knob(Knob::GnawDiggers).max(0) as usize;
    let mut have = w
        .critters
        .iter()
        .filter(|c| c.alive() && c.role & role::DIGGER != 0)
        .count();
    while have < cap {
        let pick = w
            .critters
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                c.alive()
                    && c.state == is::PROWL
                    && !c.has(flag::LEADER)
                    && !c.has(flag::TOKEN)
                    && c.role & role::DIGGER == 0
                    && sp.kind(c.kind).moves.iter().any(|m| m.kind == GNAW)
            })
            .min_by_key(|(_, c)| {
                V3::new(c.pos.x.sub(stone.at.x), Fx::ZERO, c.pos.z.sub(stone.at.z))
                    .flat_len()
                    .raw()
            })
            .map(|(i, _)| i);
        let Some(i) = pick else { break };
        w.critters[i].role |= role::DIGGER;
        have += 1;
    }

    // At the foot, they dig; digging, each is a digger-frame.
    let reach = stone.radius().add(knob_fx(Knob::GnawReach));
    let a = sp.attack(GNAW);
    let mut digging = 0;
    for c in w.critters.iter_mut() {
        if !c.alive() || c.role & role::DIGGER == 0 {
            continue;
        }
        let near = V3::new(c.pos.x.sub(stone.at.x), Fx::ZERO, c.pos.z.sub(stone.at.z))
            .flat_len()
            .raw()
            <= reach.raw();
        match c.state {
            is::PROWL if near => {
                c.state = is::STARTUP;
                c.act = GNAW;
                c.timer = a.startup.max(1);
                c.yaw = crate::pack::yaw_of(stone.at.sub(c.pos));
            }
            is::ACTIVE if c.act == GNAW => {
                // Held at it for as long as there is digging to do.
                c.timer = a.active.max(2);
                digging += 1;
            }
            _ => {}
        }
    }
    pack.memo[word::GNAW_WORK] += digging;
    if pack.memo[word::GNAW_WORK] >= knob(Knob::GnawWork).max(1) {
        // It falls. Whoever was on it comes down among them, staggered.
        let who = on[index];
        crate::stones::destroy(&mut w.players, index);
        if let Some(who) = who {
            let p = &mut w.players[who];
            let frames = knob(Knob::FallStagger).max(1) as u16;
            p.action = Action::Stagger { left: frames };
            p.stun_total = frames;
            p.grounded = false;
        }
        release(w, pack);
    }
}
