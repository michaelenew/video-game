//! The herd's own rules over the whole world: everything it does that
//! touches more than the pack (hornback.md §2–§4, §10).
//!
//! Run once a frame by the shared machinery (`FightDecl::frame`), after the
//! pack has stepped and before the fighters do:
//!
//! - **The boulders**, laid on the first frame as hazard cells that stand as
//!   solids; a charge cracks one and a second shatters it into rubble that is
//!   no longer a solid.
//! - **The charge's lane and its swept stop** (§10, item 3). Through the
//!   paws, the lane is worked out from the bull's nose along its facing --
//!   the whole run, pulled up short of the arena's edge, ended at the first
//!   solid a body the bull's width would meet -- and kept in the memo, which
//!   the marker draws. Running, the same sweep is asked one frame ahead: a
//!   solid within the next step stops it there, stunned. A stone raised into
//!   a running charge is a solid like any other.
//! - **The stampede** (§10, item 5): the bellow draws a lane from the herd's
//!   far end through its target to the edge; the cows wind up with the bull
//!   and run it, held inside it and out of every solid's lee as a hard
//!   constraint, and stepping round anybody already on the floor. Then home.
//! - **The trample's windup**: no sooner than a reaction after you can move.
//! - **Cows' rear wedges**, glanced at a cow at a time.
//! - **Bodies**: a cow and the bull are bodies a fighter does not walk
//!   through.
//! - **The calm-down**, the ford, and the coop numbers.
//! - **The ride**: see [`ride`].

use super::mind::{
    self, HerdState, bits, bull, cow, half, herd_centre, herd_state, horns_whole, lane_frame,
    set_bit, set_half, set_herd, set_horns, set_point, the_bull, with_herd, word as memo,
};
use super::{
    BELLOW, BOULDER, BULL, CHARGE, COW, CRACKED, HOOK, Knob, RUBBLE, SHOULDER, STAMPEDE, TRAMPLE,
    knob, knob_fx,
};
use crate::arena::{Solid, Terrain};
use crate::critter::{Critter, CritterField, MAX_CRITTERS, flag, is, stat, stat_fx};
use crate::fixed::Fx;
use crate::hazard::{self, Hazard};
use crate::math::V3;
use crate::pack::{Pack, mood, yaw_of};
use crate::state::{Action, MAX_PLAYERS, World};
use crate::stones::MAX_STONES;
use crate::tuning as t;

/// The herd's own words of the hunt's lore ([`crate::lore`]).
pub mod word {
    /// The bull last frame: its state, and its move in the next byte -- so a
    /// move beginning, or ending, is seen once.
    pub const PREV: usize = 0;
    /// Each critter's ride clock: frames a fighter has been on its back.
    /// Sixteen bits a body, two to a word, from word 1.
    pub const RIDE: usize = 1;
    /// The crossing's migration: frames to the next wave.
    pub const WAVE: usize = 6;
    /// The crossing's lane on the road: where it is and which way it runs,
    /// while it is drawn ahead of a wave.
    pub const WAVE_AT: usize = 7;
}

/// Where the herd leaves by: the ford, in the meadow's east edge -- or, in an
/// arena without one, the edge nearest home.
pub fn ford(arena: &Terrain, home: V3) -> V3 {
    let b = &arena.bounds;
    if arena.id == crate::arena::ArenaId::HORNBACK {
        return V3::new(b.hi_x, Fx::ZERO, home.z);
    }
    let to_hi_x = b.hi_x.sub(home.x);
    let to_lo_x = home.x.sub(b.lo_x);
    let to_hi_z = b.hi_z.sub(home.z);
    let to_lo_z = home.z.sub(b.lo_z);
    let least = to_hi_x.min(to_lo_x).min(to_hi_z).min(to_lo_z);
    if least.raw() == to_hi_x.raw() {
        V3::new(b.hi_x, Fx::ZERO, home.z)
    } else if least.raw() == to_lo_x.raw() {
        V3::new(b.lo_x, Fx::ZERO, home.z)
    } else if least.raw() == to_hi_z.raw() {
        V3::new(home.x, Fx::ZERO, b.hi_z)
    } else {
        V3::new(home.x, Fx::ZERO, b.lo_z)
    }
}

/// Where the boulders stand, in centimetres, by arena: four in the meadow
/// (§11), three on the crossing's road.
pub fn boulders(arena: crate::arena::ArenaId) -> &'static [(i32, i32)] {
    match arena {
        crate::arena::ArenaId::HORNBACK => &[(-900, 700), (-600, -1200), (300, 900), (1300, 300)],
        crate::arena::ArenaId::HORNBACK_CROSSING => &[(-1800, -500), (600, 600), (2200, -600)],
        _ => &[(-600, 600), (-600, -600), (600, 0)],
    }
}

/// How far a point is from a box's footprint, in the floor plane.
pub fn off_footprint(s: &Solid, at: V3) -> Fx {
    let dx = s.min.x.sub(at.x).max(at.x.sub(s.max.x)).max(Fx::ZERO);
    let dz = s.min.z.sub(at.z).max(at.z.sub(s.max.z)).max(Fx::ZERO);
    V3::new(dx, Fx::ZERO, dz).flat_len()
}

/// Something a charge can run into.
#[derive(Clone, Copy, Debug)]
pub enum Stopper {
    /// One of the arena's own solids: the bank's face.
    Arena(Solid),
    /// A boulder, by its hazard cell.
    Boulder(usize, Solid),
    /// A raised solid that is not a boulder: the cart.
    Raised(Solid),
    /// A stone or a planted shield, by its index in the stone field.
    Stone(usize, V3, Fx),
}

impl Stopper {
    /// Where it stands, for the wary memory and the marker's ring.
    pub fn centre(&self) -> V3 {
        match self {
            Stopper::Arena(s) | Stopper::Boulder(_, s) | Stopper::Raised(s) => V3::new(
                s.min.x.add(s.max.x).mul(Fx::ratio(1, 2)),
                Fx::ZERO,
                s.min.z.add(s.max.z).mul(Fx::ratio(1, 2)),
            ),
            Stopper::Stone(_, at, _) => V3::new(at.x, Fx::ZERO, at.z),
        }
    }
}

/// Is a solid tall enough to stop a running bull -- above its knee -- and not
/// the arena's edge?
fn stops(arena: &Terrain, s: &Solid) -> bool {
    s.max.y.sub(s.min.y).raw() > knob_fx(Knob::HeadLow).raw() && !mind::edge(arena, s)
}

/// **The first thing a charge from `from` along `dir` would meet within
/// `reach`**, for a body `half_width` wide: how far, and what. The one sweep
/// the marker and the stop both read.
pub fn first_stopper(
    w: &World,
    ground: &Terrain,
    from: V3,
    dir: V3,
    reach: Fx,
) -> Option<(Fx, Stopper)> {
    let sp = &super::SPECIES;
    let half_width = stat_fx(sp, BULL, CritterField::Width).mul(Fx::ratio(1, 2));
    let mut best: Option<(Fx, Stopper)> = None;
    let mut take = |d: Option<Fx>, s: Stopper| {
        if let Some(d) = d {
            if best.is_none_or(|(b, _)| d.raw() < b.raw()) {
                best = Some((d, s));
            }
        }
    };
    for s in ground.arena.solids().iter().filter(|s| stops(ground, s)) {
        take(
            crate::math::flat_sweep_box(from, dir, reach, half_width, s.min, s.max),
            Stopper::Arena(*s),
        );
    }
    let mut boulder_boxes = [None; 8];
    for (slot, h) in hazard::all(&w.lore) {
        if h.index() != BOULDER && h.index() != CRACKED {
            continue;
        }
        let Some(placed) = ground.floor.iter().find(|p| p.slot as usize == slot) else {
            continue;
        };
        if let Some(s) = placed.solid(sp) {
            if slot < boulder_boxes.len() {
                boulder_boxes[slot] = Some(s);
            }
            take(
                crate::math::flat_sweep_box(from, dir, reach, half_width, s.min, s.max),
                Stopper::Boulder(slot, s),
            );
        }
    }
    for s in ground.raised().iter().filter(|s| stops(ground, s)) {
        let a_boulder = boulder_boxes.iter().flatten().any(|b| b == s);
        if !a_boulder {
            take(
                crate::math::flat_sweep_box(from, dir, reach, half_width, s.min, s.max),
                Stopper::Raised(*s),
            );
        }
    }
    let field = crate::stones::gather(&w.players);
    for (index, stone) in field.iter().enumerate() {
        let Some(stone) = stone else { continue };
        if stone.standing_height().raw() <= knob_fx(Knob::HeadLow).raw() {
            continue;
        }
        take(
            crate::math::flat_sweep_disc(from, dir, reach, half_width, stone.at, stone.radius()),
            Stopper::Stone(index, stone.at, stone.radius()),
        );
    }
    best
}

/// How far a body may run from `from` along `dir` before the arena's edge,
/// less `EdgeMargin`: the charge pulls up short of the slope and the thicket
/// rather than meeting them (§10, item 3).
fn to_the_edge(arena: &Terrain, from: V3, dir: V3) -> Fx {
    let b = &arena.bounds;
    let m = knob_fx(Knob::EdgeMargin);
    let mut most = Fx::MAX;
    for (o, d, lo, hi) in [
        (from.x, dir.x, b.lo_x, b.hi_x),
        (from.z, dir.z, b.lo_z, b.hi_z),
    ] {
        if d.raw() > 0 {
            most = most.min(hi.sub(m).sub(o).div(d));
        } else if d.raw() < 0 {
            most = most.min(lo.add(m).sub(o).div(d));
        }
    }
    most.max(Fx::ZERO)
}

/// The bull's nose, on the floor.
fn nose(c: &Critter) -> V3 {
    let half = stat_fx(&super::SPECIES, BULL, CritterField::Length).mul(Fx::ratio(1, 2));
    let n = c.pos.add(c.facing().scale(half));
    V3::new(n.x, Fx::ZERO, n.z)
}

/// The charge's whole run from here: the run the bull's health allows,
/// pulled up short of the edge.
fn run_length(sp: &crate::species::Species, c: &Critter, ground: &Terrain) -> Fx {
    let run = if mind::desperate(sp, c) {
        knob_fx(Knob::DesperateRun)
    } else {
        knob_fx(Knob::ChargeRun)
    };
    // Short of the edge by the skid as well: it brakes at `Brake` from its
    // run's speed, and the skid must not carry it into the thicket either.
    let v = sp.attack(CHARGE).advance;
    let brake = sp.brake().max(Fx::ONE);
    let skid = v.mul(v).div(brake.add(brake));
    run.min(
        to_the_edge(ground, nose(c), c.facing())
            .sub(skid)
            .max(Fx::ZERO),
    )
}

/// Work out the lane of the charge being wound up, into the memo.
fn draw_charge(w: &World, ground: &Terrain, pack: &mut Pack, c: &Critter) {
    let sp = pack.sp();
    let from = nose(c);
    let dir = c.facing();
    let run = run_length(sp, c, ground);
    let (end, solid) = match first_stopper(w, ground, from, dir, run) {
        Some((d, _)) => (from.add(dir.scale(d)), true),
        None => (from.add(dir.scale(run)), false),
    };
    let cm = crate::lore::to_cm(end.sub(from).flat_len()) as i32;
    set_half(pack, memo::RUN, 0, cm.max(1));
    set_point(pack, memo::RUN_END, end);
    set_bit(pack, bits::ENDS_IN_SOLID, solid);
}

/// The species' frame. See the module docs.
pub fn frame(w: &mut World) {
    let Some(mut pack) = w.pack else { return };
    if pack.species != super::SPECIES.id {
        return;
    }
    let ground = w.terrain();
    if !mind::has_bit(&pack, bits::LAID) {
        lay(w, &mut pack);
    }
    coop(w, &mut pack);
    let Some(b) = the_bull(&w.critters) else {
        w.pack = Some(pack);
        return;
    };
    let prev = w.lore.word(word::PREV);
    let (was_state, was_act) = ((prev & 0xFF) as u8, ((prev >> 8) & 0xFF) as u8);
    let now = w.critters[b];
    let began = now.alive()
        && now.state == is::STARTUP
        && !(was_state == is::STARTUP && was_act == now.act);

    if began {
        bull_begins(w, &ground, &mut pack, b);
    }
    bull_runs(w, &ground, &mut pack, b, was_state, was_act);
    stampede(w, &ground, &mut pack);
    cows(w, &mut pack);
    glance(w, &mut pack, b);
    calm(w, &mut pack);
    bodies(w);
    super::ride::frame(w, &mut pack);

    // A broken herd leaves by the ford: that is its home now.
    if pack.mood == mood::BROKEN {
        pack.home = ford(&ground, pack.home);
    }
    let after = w.critters[b];
    w.lore
        .set_word(word::PREV, after.state as u32 | (after.act as u32) << 8);
    w.pack = Some(pack);
}

/// The first frame: the boulders down, the horns grown.
fn lay(w: &mut World, pack: &mut Pack) {
    set_bit(pack, bits::LAID, true);
    let sp = pack.sp();
    let horn = sp.part_health();
    set_horns(pack, [horn, horn]);
    let radius = hazard::stat_fx(sp, BOULDER, hazard::HazardField::Radius);
    for (x, z) in boulders(w.arena) {
        let at = V3::new(Fx::ratio(*x, 100), Fx::ZERO, Fx::ratio(*z, 100));
        let mut h = Hazard::disc(BOULDER, at, radius);
        h.state = 1;
        hazard::place(&mut w.lore, h);
    }
}

/// Two hunters: the bull's health times `CoopHealth`, once.
fn coop(w: &mut World, pack: &mut Pack) {
    if mind::has_bit(pack, bits::COOP_DONE) {
        return;
    }
    let two = w.players.iter().filter(|p| p.health > 0).count() >= 2
        && pack.seen.iter().filter(|s| s.alive).count() >= 2;
    if !two {
        return;
    }
    set_bit(pack, bits::COOP_DONE, true);
    for c in w
        .critters
        .iter_mut()
        .filter(|c| c.kind == BULL && c.alive())
    {
        let more = Fx::from_int(c.health as i32).mul(knob_fx(Knob::CoopHealth));
        c.health = more.to_int().clamp(1, i16::MAX as i32) as i16;
    }
}

/// The bull's maximum health this hunt: its kind's, or the coop share.
pub fn bull_full(pack: &Pack) -> i32 {
    let base = stat(&super::SPECIES, BULL, CritterField::Health);
    if mind::has_bit(pack, bits::COOP_DONE) {
        Fx::from_int(base).mul(knob_fx(Knob::CoopHealth)).to_int()
    } else {
        base
    }
}

/// The bull has just begun a move.
fn bull_begins(w: &mut World, ground: &Terrain, pack: &mut Pack, b: usize) {
    let sp = pack.sp();
    let c = w.critters[b];
    mind::set_last_move(pack, c.act);
    set_half(pack, memo::RUN, 1, sp.variety_frames() as i32);
    let who = (c.target as usize).min(MAX_PLAYERS - 1);
    let lead = crate::pack::lead_of(pack, who);
    // Which side it throws to: where its target is, decided now and held.
    let right = mind::side_of(&c, lead) == 1;
    let body = &mut w.critters[b];
    body.role &= !(bull::RIGHT | bull::STUNNED | bull::STAGGERED);
    if right && matches!(c.act, HOOK | SHOULDER) {
        body.role |= bull::RIGHT;
    }
    match c.act {
        CHARGE => draw_charge(w, ground, pack, &w.critters[b].clone()),
        TRAMPLE => {
            // **Never sooner than a reaction after you can move** (§2): it
            // rears through whatever is left of your knockdown.
            let p = &w.players[who];
            let down = match p.action {
                Action::Stagger { left } | Action::HitStun { left } => left as i32,
                _ => 0,
            };
            let startup = sp.attack(TRAMPLE).startup as i32;
            let t = startup.max(down + knob(Knob::TrampleAfter));
            w.critters[b].timer = t.clamp(1, u16::MAX as i32) as u16;
        }
        BELLOW => call_the_herd(w, ground, pack, b),
        _ => {}
    }
}

/// The charge, wound up and run.
fn bull_runs(
    w: &mut World,
    ground: &Terrain,
    pack: &mut Pack,
    b: usize,
    was_state: u8,
    was_act: u8,
) {
    let sp = pack.sp();
    let c = w.critters[b];
    if !c.alive() || c.act != CHARGE {
        return;
    }
    match c.state {
        is::STARTUP => draw_charge(w, ground, pack, &c),
        is::ACTIVE => {
            let speed = sp.attack(CHARGE).advance.max(Fx::ONE);
            if was_state == is::STARTUP {
                // Off: the run is as long as the lane, at the move's speed.
                let (_, run, _) = mind::charge_lane(pack);
                let frames = run
                    .div(speed)
                    .mul(Fx::from_int(crate::TICK_HZ as i32))
                    .to_int()
                    + 1;
                w.critters[b].timer = frames.clamp(1, u16::MAX as i32) as u16;
            }
            // **The swept stop**: a solid within the next frame's step stops
            // it there.
            let step = speed.mul(crate::DT);
            let from = nose(&c);
            if let Some((d, hit)) = first_stopper(w, ground, from, c.facing(), step) {
                let body = &mut w.critters[b];
                body.pos = body.pos.add(c.facing().scale(d));
                stun(w, ground, pack, b, hit);
            } else {
                // Keep the marker true as it runs: a stone raised into the
                // lane shortens it.
                let left = mind::charge_lane(pack).0.sub(from).flat_len();
                let ahead = first_stopper(w, ground, from, c.facing(), left);
                if let Some((d, _)) = ahead {
                    set_point(pack, memo::RUN_END, from.add(c.facing().scale(d)));
                    set_bit(pack, bits::ENDS_IN_SOLID, true);
                }
            }
        }
        is::RECOVERY if !(was_state == is::ACTIVE && was_act == CHARGE) => {
            // The skid: braking hard, at `Brake`.
            let brake = sp.brake().mul(crate::DT);
            let body = &mut w.critters[b];
            let flat = V3::new(body.vel.x, Fx::ZERO, body.vel.z);
            let speed = flat.flat_len();
            if speed.raw() > 0 {
                let left = speed.sub(brake).max(Fx::ZERO);
                let v = flat.normalized().scale(left);
                body.vel = V3::new(v.x, body.vel.y, v.z);
            }
        }
        is::RECOVERY => {
            // A miss, skidding. Desperate, it may chain: turn, and one paw.
            set_bit(pack, bits::ENDS_IN_SOLID, false);
            let chain = mind::desperate(sp, &c)
                && (pack.roll() % 100) < knob(Knob::ChainAppetite).max(0) as u32
                && pack.mood == mood::HUNTING;
            if chain {
                let who = (c.target as usize).min(MAX_PLAYERS - 1);
                let lead = crate::pack::lead_of(pack, who);
                let body = &mut w.critters[b];
                let to = V3::new(lead.x.sub(body.pos.x), Fx::ZERO, lead.z.sub(body.pos.z));
                if to.flat_len().raw() > 0 {
                    body.yaw = yaw_of(to);
                }
                body.state = is::STARTUP;
                body.act = CHARGE;
                body.timer = knob(Knob::ChainPaw).max(1) as u16;
                body.role |= bull::CHAINED;
                body.set(flag::HIT_USED, false);
                let now = w.critters[b];
                draw_charge(w, ground, pack, &now);
            }
        }
        _ => {}
    }
}

/// **A charge met a solid**: the stun, and what the solid makes of it.
fn stun(w: &mut World, ground: &Terrain, pack: &mut Pack, b: usize, hit: Stopper) {
    let sp = pack.sp();
    let _ = ground;
    // Somebody standing on top of what it hit is shaken off, and the stun is
    // shorter: not a clean hit (§3).
    let top = match hit {
        Stopper::Arena(s) | Stopper::Boulder(_, s) | Stopper::Raised(s) => Some(s),
        Stopper::Stone(..) => None,
    };
    let mut occupied = false;
    if let Some(s) = top {
        for p in w.players.iter_mut().filter(|p| p.health > 0 && !p.aboard()) {
            let on = p.pos.x.raw() >= s.min.x.raw()
                && p.pos.x.raw() <= s.max.x.raw()
                && p.pos.z.raw() >= s.min.z.raw()
                && p.pos.z.raw() <= s.max.z.raw()
                && p.pos.y.sub(s.max.y).abs().raw()
                    <= crate::arena::SKIN.add(crate::arena::SKIN).raw()
                && p.grounded;
            if on {
                occupied = true;
                p.wound(knob(Knob::ShakeOffDamage));
                p.action = Action::Stagger {
                    left: knob(Knob::ShakeOffStagger).max(1) as u16,
                };
                p.stun_total = knob(Knob::ShakeOffStagger).max(1) as u16;
                p.vel.y = t::jump_speed().mul(Fx::ratio(1, 2));
                p.grounded = false;
            }
        }
    }
    let frames = if occupied {
        knob(Knob::StunOccupied)
    } else {
        knob(Knob::StunFrames)
    };
    let body = &mut w.critters[b];
    body.state = is::FLINCH;
    body.timer = frames.max(1) as u16;
    body.vel = V3::ZERO;
    body.role = (body.role & !(bull::STAGGERED | bull::CHAINED)) | bull::STUNNED;
    body.set(flag::HIT_USED, true);
    // It learns this wall, once.
    set_point(pack, memo::WARY_AT, hit.centre());
    set_half(pack, memo::WARY, 0, knob(Knob::WaryFrames) + frames);
    set_half(pack, memo::WARY, 1, knob(Knob::ChargeLockout) + frames);
    set_bit(pack, bits::ENDS_IN_SOLID, false);
    match hit {
        Stopper::Boulder(slot, _) => {
            // Cracked by the first, shattered by the second.
            let mut h = hazard::get(&w.lore, slot);
            if h.index() == BOULDER {
                h.kind = CRACKED + 1;
            } else {
                h.kind = RUBBLE + 1;
                h.state = 0;
            }
            hazard::set(&mut w.lore, slot, h);
        }
        Stopper::Stone(index, ..) => {
            let owner = index / crate::class::MAX_STRUCTURES;
            let lit = crate::stones::is_lit(&w.players, index);
            if matches!(w.players[owner].mechanic, crate::class::Mechanic::Shield(_)) {
                // A planted shield takes the charge's whole weight.
                let damage = sp.attack(CHARGE).damage;
                crate::bulwark::load(&mut w.players[owner], damage, false);
            } else {
                // A stone stops it like a boulder, and collapses doing it.
                crate::stones::destroy(&mut w.players, index);
                if lit {
                    let burn = knob(Knob::StoneDamage);
                    crate::pack::struck(
                        pack,
                        &mut w.critters,
                        b,
                        burn,
                        V3::ZERO,
                        Fx::ZERO,
                        Fx::ZERO,
                    );
                }
            }
        }
        Stopper::Arena(_) | Stopper::Raised(_) => {}
    }
}

/// **The bellow comes**: draw the lane, and every cow with the herd winds
/// the stampede up with the bull.
fn call_the_herd(w: &mut World, ground: &Terrain, pack: &mut Pack, b: usize) {
    let sp = pack.sp();
    let c = w.critters[b];
    let who = (c.target as usize).min(MAX_PLAYERS - 1);
    let target = crate::pack::lead_of(pack, who);
    let middle = herd_centre(pack, &w.critters);
    let to = V3::new(target.x.sub(middle.x), Fx::ZERO, target.z.sub(middle.z));
    let dir = if to.flat_len().raw() > 0 {
        to.normalized()
    } else {
        c.facing()
    };
    // From the herd's far end...
    let back = w
        .critters
        .iter()
        .filter(|c| with_herd(c))
        .map(|c| lane_frame(middle, dir, c.pos).0)
        .fold(Fx::ZERO, Fx::min);
    let tail = stat_fx(sp, COW, CritterField::Length);
    let at = middle.add(dir.scale(back.sub(tail)));
    // ...to the arena's edge, or the face of something that fills the lane.
    let mut len = to_the_edge(ground, at, dir).add(knob_fx(Knob::EdgeMargin));
    let width = knob_fx(Knob::LaneWidth);
    for s in ground.solids().filter(|s| stops(ground, s)) {
        let (lo_a, _, lo_c, hi_c) = mind::shadow(at, dir, s.min, s.max);
        let half = width.mul(Fx::ratio(1, 2));
        if lo_c.raw() <= half.neg().raw() && hi_c.raw() >= half.raw() && lo_a.raw() > 0 {
            len = len.min(lo_a);
        }
    }
    mind::set_lane(pack, at, dir, len);
    for t in 0..MAX_PLAYERS {
        mind::set_trampled(pack, t, false);
    }
    set_herd(pack, HerdState::Stampede);
    set_half(pack, memo::CLOCKS, 0, knob(Knob::RunFrames));
    let windup = c.timer;
    for c in w.critters.iter_mut() {
        if !with_herd(c) || c.state != is::PROWL || ridden(w.players.as_slice(), c) {
            continue;
        }
        c.state = is::STARTUP;
        c.act = STAMPEDE;
        c.timer = windup.max(1);
        c.set(flag::HIT_USED, false);
    }
}

/// Is this cow carrying somebody?
fn ridden(players: &[crate::state::Player], c: &Critter) -> bool {
    let _ = c;
    players
        .iter()
        .any(|p| p.health > 0 && super::ride::rider_of(p).is_some())
        && players
            .iter()
            .filter_map(super::ride::rider_of)
            .any(|i| i == c.slot as usize && false)
}

/// The obstacles in a lane: every solid that stops a cow, every stone, and
/// the bull standing in it -- each as a footprint box.
fn obstacles(w: &World, ground: &Terrain) -> ([Option<(V3, V3)>; 24], usize) {
    let mut out = [None; 24];
    let mut n = 0;
    let mut push = |min: V3, max: V3| {
        if n < out.len() {
            out[n] = Some((min, max));
            n += 1;
        }
    };
    for s in ground.solids().filter(|s| stops(ground, s)) {
        push(s.min, s.max);
    }
    for stone in crate::stones::gather(&w.players).iter().flatten() {
        if stone.standing_height().raw() <= knob_fx(Knob::HeadLow).raw() {
            continue;
        }
        let r = stone.radius();
        push(
            V3::new(stone.at.x.sub(r), Fx::ZERO, stone.at.z.sub(r)),
            V3::new(stone.at.x.add(r), Fx::ZERO, stone.at.z.add(r)),
        );
    }
    let _ = MAX_STONES;
    (out, n)
}

/// **The stampede's run**: every cow running it is steered down its lane,
/// round every lee and round anybody on the floor, and then held to all
/// three -- the drawn lane is the hit test's own answer, not a forecast.
fn stampede(w: &mut World, ground: &Terrain, pack: &mut Pack) {
    let sp = pack.sp();
    let state = herd_state(pack);
    if state == HerdState::Returning {
        // Home: every cow back near it, and the lockout starts counting.
        let near = knob_fx(Knob::HomeWithin);
        let home = w
            .critters
            .iter()
            .filter(|c| with_herd(c))
            .all(|c| c.pos.sub(pack.home).flat_len().raw() <= near.raw());
        if home || half(pack, memo::CLOCKS, 0) == 0 {
            set_herd(pack, HerdState::Alarmed);
            set_half(pack, memo::CLOCKS, 1, knob(Knob::BellowLockout));
        }
        return;
    }
    if state != HerdState::Stampede {
        return;
    }
    let Some((at, dir, len, width)) = mind::bellow_lane(pack) else {
        set_herd(pack, HerdState::Returning);
        return;
    };
    let half_w = stat_fx(sp, COW, CritterField::Width).mul(Fx::ratio(1, 2));
    let half_l = stat_fx(sp, COW, CritterField::Length).mul(Fx::ratio(1, 2));
    let room = width.mul(Fx::ratio(1, 2)).sub(half_w);
    let side = V3::new(dir.z.neg(), Fx::ZERO, dir.x);
    let (blocks, n) = obstacles(w, ground);
    // Who is on the floor in the lane, to be stepped round.
    let mut fallen = [None; MAX_PLAYERS];
    for (i, p) in w.players.iter().enumerate() {
        if p.health > 0 && matches!(p.action, Action::Stagger { .. }) {
            fallen[i] = Some(p.pos);
        }
    }
    let round = knob_fx(Knob::TrampleFallen);
    let mut running = 0;
    let mut waiting = 0;
    for i in 0..MAX_CRITTERS {
        let c = w.critters[i];
        if !(c.alive() && c.kind == COW && c.act == STAMPEDE) {
            continue;
        }
        match c.state {
            is::STARTUP => {
                waiting += 1;
                continue;
            }
            is::ACTIVE => {}
            _ => continue,
        }
        running += 1;
        let (along, mut across) = lane_frame(at, dir, c.pos);
        // Off the end of its lane: the run is over for this one.
        if along.add(half_l).raw() >= len.raw() || half(pack, memo::CLOCKS, 0) == 0 {
            let body = &mut w.critters[i];
            body.state = is::RECOVERY;
            body.timer = sp.attack(STAMPEDE).recovery.max(1);
            continue;
        }
        // Where it wants to be across the lane: here, unless a lee or a
        // body on the floor is ahead of it.
        let ahead = half_l.add(half_l).add(knob_fx(Knob::LeeLength));
        let mut want = across.clamp(room.neg(), room);
        for (min, max) in blocks[..n].iter().flatten() {
            let (lo_a, hi_a, lo_c, hi_c) = mind::shadow(at, dir, *min, *max);
            let grow = half_w.add(half_l);
            if along.add(ahead).raw() < lo_a.sub(grow).raw()
                || along.raw() > hi_a.add(knob_fx(Knob::LeeLength)).raw()
            {
                continue;
            }
            if want.raw() > lo_c.sub(half_w).raw() && want.raw() < hi_c.add(half_w).raw() {
                let left = hi_c.add(half_w);
                let right = lo_c.sub(half_w);
                want = if want.sub(right).abs().raw() <= left.sub(want).abs().raw()
                    && right.raw() >= room.neg().raw()
                    || left.raw() > room.raw()
                {
                    right
                } else {
                    left
                };
            }
        }
        for p in fallen.iter().flatten() {
            let (pa, pc) = lane_frame(at, dir, *p);
            if pa.raw() < along.sub(half_l).raw() || pa.sub(along).raw() > ahead.raw() {
                continue;
            }
            if want.sub(pc).abs().raw() < round.raw() {
                want = if want.raw() >= pc.raw() {
                    pc.add(round)
                } else {
                    pc.sub(round)
                };
            }
        }
        want = want.clamp(room.neg(), room);
        // Steered: toward its place across the lane, a body length on.
        let goal = at
            .add(dir.scale(along.add(half_l.add(half_l))))
            .add(side.scale(want));
        let to = V3::new(goal.x.sub(c.pos.x), Fx::ZERO, goal.z.sub(c.pos.z));
        let body = &mut w.critters[i];
        if to.flat_len().raw() > 0 {
            body.yaw = yaw_of(to);
        }
        // **Held**: inside the lane, out of every lee, round the fallen.
        across = across.clamp(room.neg(), room);
        for (min, max) in blocks[..n].iter().flatten() {
            if mind::in_lee(at, dir, *min, *max, body.pos, half_w) {
                let (_, _, lo_c, hi_c) = mind::shadow(at, dir, *min, *max);
                let left = hi_c.add(half_w);
                let right = lo_c.sub(half_w);
                across = if across.sub(right).abs().raw() <= left.sub(across).abs().raw()
                    && right.raw() >= room.neg().raw()
                    || left.raw() > room.raw()
                {
                    right
                } else {
                    left
                };
            }
        }
        for p in fallen.iter().flatten() {
            let (pa, pc) = lane_frame(at, dir, *p);
            if pa.sub(along).abs().raw() <= half_l.add(round).raw()
                && across.sub(pc).abs().raw() < round.raw()
            {
                across = if across.raw() >= pc.raw() {
                    pc.add(round)
                } else {
                    pc.sub(round)
                };
            }
        }
        let (along_now, across_now) = lane_frame(at, dir, body.pos);
        let _ = along_now;
        let shift = across.sub(across_now);
        if shift.raw() != 0 {
            body.pos = body.pos.add(side.scale(shift));
        }
    }
    if running == 0 && waiting == 0 {
        set_herd(pack, HerdState::Returning);
        set_half(pack, memo::CLOCKS, 0, knob(Knob::RunFrames));
        mind::clear_lane(pack);
    }
}

/// The cows' own frame: their glances at their rear wedges, and leaving.
fn cows(w: &mut World, pack: &mut Pack) {
    let every = knob(Knob::CowGlance).max(1) as u32;
    let reach = knob_fx(Knob::KickReach);
    let arc = knob_fx(Knob::KickArc);
    let frame = w.frame;
    let ford = pack.home;
    for i in 0..MAX_CRITTERS {
        let c = w.critters[i];
        if c.kind != COW || !c.alive() {
            continue;
        }
        // Bolted, at the ford: gone.
        if (c.role & cow::BOLTED != 0 || pack.mood == mood::BROKEN)
            && c.pos.sub(ford_of(w, pack)).flat_len().raw() <= Fx::ONE.add(Fx::ONE).raw()
        {
            let body = &mut w.critters[i];
            body.state = is::GONE;
            continue;
        }
        // **A cow glances every `CowGlance` frames, staggered across the
        // herd**, at what stands in its rear wedge. A fighter it sees there
        // twice running has stood there a whole glance: that is a kick.
        if frame % every != i as u32 % every {
            continue;
        }
        let back = c.facing().scale(Fx::ONE.neg());
        let rump = c.pos.add(
            back.scale(stat_fx(&super::SPECIES, COW, CritterField::Length).mul(Fx::ratio(1, 2))),
        );
        let behind = w.players.iter().any(|p| {
            let d = V3::new(p.pos.x.sub(rump.x), Fx::ZERO, p.pos.z.sub(rump.z));
            let near = d.flat_len();
            p.health > 0
                && !p.aboard()
                && p.pos.y.sub(c.pos.y).raw()
                    < stat_fx(&super::SPECIES, COW, CritterField::Height).raw()
                && near.raw() <= reach.raw()
                && (near.raw() == 0 || back.dot(d.normalized()).raw() >= arc.raw())
        });
        let body = &mut w.critters[i];
        let counted = (body.role & cow::WEDGE) as u32;
        let next = if behind {
            (counted + every).min(cow::WEDGE as u32)
        } else {
            0
        };
        body.role = (body.role & !cow::WEDGE) | next as u8;
    }
    let _ = ford;
}

/// The ford for this hunt.
fn ford_of(w: &World, pack: &Pack) -> V3 {
    let ground = w.terrain();
    if pack.mood == mood::BROKEN {
        return pack.home;
    }
    ford(&ground, pack.home)
}

/// What the bull's glance saw: a fighter in front of it, near, in the startup
/// of an attack -- a pose, not a button (§2, the guard).
fn glance(w: &mut World, pack: &mut Pack, b: usize) {
    if pack.glance_left.saturating_add(1) != pack.glance_frames() {
        return;
    }
    let c = w.critters[b];
    for (who, p) in w.players.iter().enumerate() {
        let near = V3::new(p.pos.x.sub(c.pos.x), Fx::ZERO, p.pos.z.sub(c.pos.z)).flat_len();
        let reach = knob_fx(Knob::GuardSees)
            .add(stat_fx(&super::SPECIES, BULL, CritterField::Length).mul(Fx::ratio(1, 2)));
        let swinging = matches!(p.action, Action::Startup { .. });
        let seen = p.health > 0
            && swinging
            && near.raw() <= reach.raw()
            && mind::off_nose(&c, p.pos).raw() >= knob_fx(Knob::GuardArc).raw();
        mind::set_seen_swing(pack, who, seen);
    }
}

/// **Calming down** (§5): every fighter past `CalmRadius` of every animal for
/// `CalmFrames`, and the herd goes back to grazing -- the fight abandoned,
/// not lost.
fn calm(w: &mut World, pack: &mut Pack) {
    if pack.mood != mood::HUNTING {
        set_half(pack, memo::GUARD, 1, 0);
        return;
    }
    let far = knob_fx(Knob::CalmRadius);
    let away = w.players.iter().filter(|p| p.health > 0).all(|p| {
        w.critters
            .iter()
            .filter(|c| c.alive())
            .all(|c| c.pos.sub(p.pos).flat_len().raw() > far.raw())
    });
    if !away {
        set_half(pack, memo::GUARD, 1, 0);
        return;
    }
    let kept = half(pack, memo::GUARD, 1) + 1;
    set_half(pack, memo::GUARD, 1, kept);
    if kept >= knob(Knob::CalmFrames) && herd_state(pack) != HerdState::Stampede {
        pack.mood = mood::CALM;
        set_herd(pack, HerdState::Grazing);
        set_half(pack, memo::GUARD, 1, 0);
    }
}

/// **A cow and the bull are bodies** (§2: "a cow ... shoves a fighter aside
/// like any body"): a fighter on the floor is pushed out of their boxes.
/// Nothing stands on the bull; a cow's back is a surface, and somebody on it
/// is riding (`ride`).
fn bodies(w: &mut World) {
    let sp = &super::SPECIES;
    let radius = t::body_radius();
    for c in w.critters.iter().filter(|c| c.alive()) {
        let body = c.body(sp);
        for p in w.players.iter_mut().filter(|p| p.health > 0 && !p.aboard()) {
            if p.pos.y.raw() >= body.crown().raw() {
                continue;
            }
            let local = body.to_local(p.pos);
            let lx = body.half_len.add(radius);
            let lz = body.half_wid.add(radius);
            if local.x.abs().raw() >= lx.raw() || local.z.abs().raw() >= lz.raw() {
                continue;
            }
            // Out by the nearer side.
            let out_x = lx.sub(local.x.abs());
            let out_z = lz.sub(local.z.abs());
            let pushed = if out_z.raw() <= out_x.raw() {
                let z = if local.z.raw() >= 0 { lz } else { lz.neg() };
                V3::new(local.x, local.y, z)
            } else {
                let x = if local.x.raw() >= 0 { lx } else { lx.neg() };
                V3::new(x, local.y, local.z)
            };
            let world = body.to_world(pushed);
            p.pos = V3::new(world.x, p.pos.y, world.z);
        }
    }
}

/// Is a body on the floor about to be trodden by the run? For tests.
pub fn trodden(w: &World, who: usize) -> bool {
    w.pack.is_some_and(|p| mind::trampled(&p, who))
}

/// The moves a stunned bull is out of, for tests: whether its horns are
/// whole.
pub fn horns(w: &World) -> [bool; 2] {
    w.pack.map_or([true, true], |p| horns_whole(&p))
}

/// Kept so the lane's own frame reads in one place.
pub fn lane_point(at: V3, dir: V3, along: Fx, across: Fx) -> V3 {
    let side = V3::new(dir.z.neg(), Fx::ZERO, dir.x);
    at.add(dir.scale(along)).add(side.scale(across))
}

const _: u8 = BELLOW;
