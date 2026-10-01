//! What the Siegeshell wants, and what its parasites want.
//!
//! **Most of it is not a brain** (§5). The walk is a gait with a heading, and
//! every footfall is the gait. What is left to decide is which foot does
//! something other than walk -- **the legs' channel**, run here once a frame
//! ([`legs`]) -- and what the shell does about riders and the floor around it
//! -- **the body's channel**, the shared brain's `Doing`, with its own terms
//! in the scoring ([`appetite`]). A move on one channel may not land within
//! `ChannelGap` frames of a move on the other.
//!
//! **Steering** is no pursuit: a point on the valley's centre line ahead of it
//! ([`prowl_to`]), at a turn rate that is barely there.
//!
//! **Its parasites are gnawers**: the Gnawers' own mind ([`gnawers::Mind`]),
//! handed every question first, with [`Roost`] on top -- they keep to the
//! shell until something calls them down.

use super::fight::{self, leg, word};
use super::gait;
use super::{
    BEAM, Clip, DRAG, Knob, LEG_COUNT, PLOUGH, SHED, SHIVER, SHRUG, SPECIES, STAMP, ankle_part,
};
use crate::beast::{self, Pose};
use crate::critter::{Body, Critter, CritterMove, Critters};
use crate::fixed::Fx;
use crate::math::{self, V3};
use crate::monster::{Attack, Doing, Herd, Mind, Monster};
use crate::pack::{Look, Pack, PackDecl, PackMind, Steer};
use crate::sign::{Says, Sign, Signs};
use crate::species::gnawers;
use crate::state::{MAX_PLAYERS, Player, World};

// ---------------------------------------------------------------------------
// Steering, and how it looks walking
// ---------------------------------------------------------------------------

/// **Where it walks** (`FightDecl::prowl_to`): the valley's centre line, a
/// way ahead of it. It is not after anybody.
pub fn prowl_to(m: &Monster, _mind: &Mind) -> Option<V3> {
    Some(V3::new(
        m.pos.x.add(Knob::PathAhead.fx().max(Fx::ONE)),
        Fx::ZERO,
        Fx::ZERO,
    ))
}

/// **Its posture while it walks** (`FightDecl::clip`): the walk's sway by the
/// stride while it is walking, the idle when it has halted -- read from its
/// own pace rather than the shared speed, which the shared walk it is held
/// against has braked to nothing.
pub fn posture(m: &Monster) -> Option<Pose> {
    if !matches!(m.doing, Doing::Prowl) {
        return None;
    }
    let sp = m.sp();
    let pace = super::gait::pace(m, fight::phase(m));
    if pace.raw() <= 0 {
        return Some(beast::sample(
            sp,
            Clip::Idle as usize,
            Fx::from_raw(m.beat as i32),
        ));
    }
    let clip = if fight::phase(m) >= 2 {
        Clip::Hurry
    } else {
        Clip::Walk
    };
    Some(beast::sample(
        sp,
        clip as usize,
        Fx::from_raw(m.stride as i32),
    ))
}

// ---------------------------------------------------------------------------
// The body's channel
// ---------------------------------------------------------------------------

/// Where on the shell a rider stands, by the part under them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Region {
    /// The rim or the flank, on a side: `-1` left, `+1` right.
    Side(i32),
    /// The plateau.
    Plateau,
    /// The crown and the anchors.
    Crown,
    /// A thigh splayed into a stair, or anything else of it.
    Other,
}

/// Which region a part is.
pub fn region_of(part: usize) -> Region {
    let side = super::shell_side(part);
    if side != 0 {
        return Region::Side(side);
    }
    match part {
        super::PLATEAU_FORE | super::PLATEAU_MID | super::PLATEAU_AFT => Region::Plateau,
        super::CROWN_PART => Region::Crown,
        p if super::anchor_of(p).is_some() => Region::Crown,
        _ => Region::Other,
    }
}

/// Where a rider at `at` is on the shell, from above: the part whose top is
/// under them, if any. A rider's own mount is the better answer where there
/// is one; the brain sees positions, not mounts, so it asks the shape.
pub fn region_at(m: &Monster, at: V3) -> Option<Region> {
    let rig = m.rig();
    let below = Knob::FloorBelow.fx();
    rig.surface_within(at, crate::tuning::body_radius(), below, below)
        .map(|(part, _)| region_of(part))
}

/// A floor point read across the creature: how far ahead of its middle, and
/// how far out to its right.
fn across(m: &Monster, at: V3) -> (Fx, Fx) {
    let local = gait::flat_body(m, at);
    (local.x, local.z)
}

/// Would a body move of `kind` begun now land its hit inside the channel gap
/// of the legs' move under way, or of the next footfall's ring?
fn crowds_the_legs(m: &Monster, kind: u8) -> bool {
    let a = m.sp().attack(kind);
    let gap = Knob::ChannelGap.raw().max(0);
    let lands = a.startup as i32;
    let ends = lands + a.active as i32;
    let c = leg::get(m);
    if c.kind != leg::NONE && c.phase <= leg::ACTIVE {
        let total = |p: u32| -> i32 {
            let k = if c.kind == leg::DRAG { DRAG } else { STAMP };
            let a = m.sp().attack(k);
            match p {
                leg::STARTUP => a.startup as i32,
                _ => a.active as i32,
            }
        };
        let (from, to) = if c.phase == leg::STARTUP {
            (c.left as i32, c.left as i32 + total(leg::ACTIVE))
        } else {
            (0, c.left as i32)
        };
        if !(ends + gap < from || lands > to + gap) {
            return true;
        }
    }
    if let Some(beat) = gait::frames_to_beat(m) {
        let ring = Knob::RingFrames.raw().max(0);
        // Every beat from here to the end of the hit.
        let per = (gait::frames_per_beat(m)).max(1);
        let mut at = beat as i32;
        while at <= ends + gap {
            if !(ends + gap < at || lands > at + ring + gap) {
                return true;
            }
            at += per;
        }
    }
    false
}

/// **Its own terms in the scoring** (`FightDecl::appetite`): it is not after
/// anybody, so the shared range-and-bearing score is thrown away and each
/// body move scores what it is for -- riders by region for the shrug and the
/// shiver, somebody on the flank for the shed, somebody in the lane ahead for
/// the plough, and at the siege line the beam and nothing else.
pub fn appetite(m: &Monster, kind: u8, _score: i32, mind: &Mind) -> i32 {
    let siege = fight::at_siege_line(m);
    if kind == BEAM {
        return if siege { 10_000 } else { 0 };
    }
    if siege || crowds_the_legs(m, kind) {
        return 0;
    }
    let feet = Fx::from_int(super::BUILD.foot_out).div(Fx::from_int(100));
    let mut score = 0;
    for q in mind.quarry.iter().filter(|q| q.alive) {
        let place = if q.aboard { region_at(m, q.pos) } else { None };
        score += match (kind, place) {
            (SHRUG, Some(Region::Side(_))) => Knob::RimAppetite.raw(),
            (SHIVER, Some(Region::Crown)) => Knob::CrownAppetite.raw(),
            (SHED, None) if !q.aboard => {
                let (_, out) = across(m, q.pos);
                let beyond = out.abs().sub(feet);
                if beyond.raw() >= Knob::ShedNear.fx().raw()
                    && beyond.raw() <= Knob::ShedFar.fx().raw()
                {
                    Knob::FlankAppetite.raw()
                } else {
                    0
                }
            }
            (PLOUGH, None) if !q.aboard => {
                let (ahead, out) = across(m, q.pos);
                let in_lane = out.abs().raw() <= Knob::PloughHalf.fx().raw()
                    && ahead.raw() >= Knob::PloughFrom.fx().raw()
                    && ahead.raw() <= Knob::PloughTo.fx().raw();
                if in_lane {
                    Knob::AheadAppetite.raw()
                } else {
                    0
                }
            }
            _ => 0,
        };
    }
    score
}

/// **As a body move commits** (`FightDecl::commit`): which side a shrug lifts
/// (the side with the most riders on it) and a shed rattles (the side its
/// target is on), where a shed's plates fall, and the phase's lockouts.
pub fn commit(m: &mut Monster, kind: u8, mind: &Mind) {
    let feet = Fx::from_int(super::BUILD.foot_out).div(Fx::from_int(100));
    match kind {
        SHRUG => {
            let mut lean = 0;
            for q in mind.quarry.iter().filter(|q| q.alive && q.aboard) {
                if let Some(Region::Side(s)) = region_at(m, q.pos) {
                    lean += s;
                }
            }
            // `mirror` plays the clip, which lifts its right, on its left.
            m.brain.mirror = lean < 0;
        }
        SHED => {
            // The side its nearest flanker is on, and the middle of where the
            // plates fall: abreast of them, out on the flank.
            let target = mind
                .quarry
                .iter()
                .filter(|q| q.alive && !q.aboard)
                .min_by_key(|q| crate::math::wide_flat_dist(q.pos, m.pos).raw());
            let (ahead, out) = target.map_or((Fx::ZERO, feet), |q| across(m, q.pos));
            let side = if out.raw() < 0 { -1 } else { 1 };
            m.brain.mirror = side < 0;
            let mid = math::half(Knob::ShedNear.fx().add(Knob::ShedFar.fx()));
            let at = gait::flat_world(
                m,
                V3::new(
                    ahead.clamp(Fx::from_int(-14), Fx::from_int(14)),
                    Fx::ZERO,
                    feet.add(mid).mul(Fx::from_int(side)),
                ),
            );
            m.aim_at(at);
        }
        PLOUGH => {
            let at = gait::flat_world(m, V3::new(Knob::PloughTo.fx(), Fx::ZERO, Fx::ZERO));
            m.aim_at(at);
        }
        _ => {}
    }
    // **The phase's lockouts**: walking, the shed and the plough lock out
    // long; hurried, the shiver short.
    let slot = kind as usize;
    match (kind, fight::phase(m)) {
        (SHED | PLOUGH, 0) => {
            let cd = Fx::from_int(m.brain.cooldown[slot] as i32)
                .mul(Knob::LockoutWalking.fx())
                .to_int();
            m.brain.cooldown[slot] = cd.clamp(0, u16::MAX as i32) as u16;
        }
        (SHIVER, p) if p >= 2 => {
            m.brain.cooldown[slot] = Knob::ShiverHurried.raw().clamp(0, u16::MAX as i32) as u16;
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// The legs' channel
// ---------------------------------------------------------------------------

/// Xorshift32 on the brain's generator: the same numbers the body's choices
/// draw from, advanced only inside the tick, so a rollback re-rolls them.
fn roll(m: &mut Monster) -> u32 {
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

/// A leg's stamp lockout, from the lore.
fn lockout(w: &World, l: usize) -> u16 {
    let word = w.lore.word(word::STAMP_LOCK0 + l / 2);
    if l % 2 == 0 {
        (word & 0xFFFF) as u16
    } else {
        (word >> 16) as u16
    }
}

fn set_lockout(w: &mut World, l: usize, v: u16) {
    let i = word::STAMP_LOCK0 + l / 2;
    let word = w.lore.word(i);
    let word = if l % 2 == 0 {
        (word & 0xFFFF_0000) | v as u32
    } else {
        (word & 0xFFFF) | ((v as u32) << 16)
    };
    w.lore.set_word(i, word);
}

/// A half of a word, by fighter.
fn half_of(w: &World, i: usize, who: usize) -> u16 {
    let word = w.lore.word(i);
    if who == 0 {
        (word & 0xFFFF) as u16
    } else {
        (word >> 16) as u16
    }
}

fn set_half(w: &mut World, i: usize, who: usize, v: u16) {
    let word = w.lore.word(i);
    let word = if who == 0 {
        (word & 0xFFFF_0000) | v as u32
    } else {
        (word & 0xFFFF) | ((v as u32) << 16)
    };
    w.lore.set_word(i, word);
}

/// Frames each phase of a legs' move lasts: windup, out, recovery, home.
fn frames(kind: u32, phase: u32) -> u16 {
    let a = SPECIES.attack(if kind == leg::DRAG { DRAG } else { STAMP });
    match phase {
        leg::STARTUP => a.startup,
        leg::ACTIVE => a.active,
        leg::RECOVERY => a.recovery,
        _ => Knob::FootHome.raw().clamp(1, 600) as u16,
    }
}

/// A whole legs' move, windup to the foot home.
fn length(kind: u32) -> u16 {
    (0..4).map(|p| frames(kind, p)).sum()
}

/// How far through its phase the channel is, nought to one.
fn progress(c: leg::Channel) -> Fx {
    let total = frames(c.kind, c.phase).max(1);
    Fx::from_int(total.saturating_sub(c.left.min(total)) as i32).div(Fx::from_int(total as i32))
}

/// Frames a leg's foot has been planted since it last landed.
fn planted_for(m: &Monster, l: usize) -> Option<i32> {
    if gait::swinging(m, l).is_some() {
        return None;
    }
    let per = m.sp().gait_stride();
    let walk = m.speed;
    if walk.raw() <= 0 {
        return Some(0);
    }
    let gone = gait::since_landing(m, l).mul(per).div(walk.mul(crate::DT));
    Some(gone.to_int())
}

/// Frames until a leg's foot next lifts, at the pace it walks: `None` halted.
fn frames_to_lift(m: &Monster, l: usize) -> Option<i32> {
    let walk = m.speed;
    if walk.raw() <= 0 {
        return None;
    }
    let lift_at = Fx::ONE.sub(Knob::LiftShare.fx());
    let left = lift_at.sub(gait::since_landing(m, l));
    if left.raw() <= 0 {
        return Some(0);
    }
    Some(
        left.mul(m.sp().gait_stride())
            .div(walk.mul(crate::DT))
            .to_int(),
    )
}

/// The rest of what the legs' channel needs before a move: no move under way
/// on the body that would land within the gap of this one, and the next beat
/// far enough off that the move is done with before it.
fn room_for(m: &Monster, kind: u32) -> bool {
    let gap = Knob::ChannelGap.raw().max(0);
    let lands = frames(kind, leg::STARTUP) as i32;
    let ends = lands + frames(kind, leg::ACTIVE) as i32;
    // Off the beat: the footfall's ring is out the dozen frames after a
    // landing, and this move's hit must clear it by the gap.
    if let Some(beat) = gait::frames_to_beat(m) {
        let ring = Knob::RingFrames.raw().max(0);
        let beat = beat as i32;
        let _ = ring;
        if ends + gap >= beat {
            return false;
        }
    }
    // The body's move: its hit, if it has one to come, clear of ours.
    let body = match m.doing {
        Doing::Startup { kind, left } => {
            let a = m.sp().attack(kind);
            Some((left as i32, left as i32 + a.active as i32))
        }
        Doing::Active { left, .. } => Some((0, left as i32)),
        _ => None,
    };
    if let Some((from, to)) = body {
        if !(ends + gap < from || lands > to + gap) {
            return false;
        }
    }
    true
}

/// Fighters on the floor, alive, and where.
fn on_floor(w: &World) -> [Option<V3>; MAX_PLAYERS] {
    std::array::from_fn(|i| {
        let p = &w.players[i];
        (p.health > 0 && !p.aboard()).then_some(p.pos)
    })
}

/// **The legs, once a frame**: the lockouts and the cling count, then the
/// move under way, or a new one.
pub fn legs(w: &mut World, slot: usize) {
    let Some(mut m) = w.monsters[slot] else {
        return;
    };
    for l in 0..LEG_COUNT {
        let v = lockout(w, l).saturating_sub(1);
        set_lockout(w, l, v);
    }
    cling(w, &m);
    if !m.alive()
        || !matches!(
            m.doing,
            Doing::Prowl | Doing::Startup { .. } | Doing::Active { .. } | Doing::Recovery { .. }
        )
    {
        leg::clear(&mut m);
        w.monsters[slot] = Some(m);
        return;
    }
    let c = leg::get(&m);
    if c.kind != leg::NONE {
        tick(w, &mut m, c);
    } else {
        let rest = w.lore.word(word::LEG_REST);
        if rest > 0 {
            w.lore.set_word(word::LEG_REST, rest - 1);
        } else if m.brain.grace == 0 {
            choose(w, &mut m);
        }
    }
    w.monsters[slot] = Some(m);
}

/// **cling(m)**: how long each fighter has stood within `ClingRadius` of one
/// ankle -- the same ankle; a step to another starts again.
fn cling(w: &mut World, m: &Monster) {
    let near = Knob::ClingRadius.fx().add(crate::tuning::body_radius());
    let floor = on_floor(w);
    for (who, at) in floor.iter().enumerate() {
        let found = at.and_then(|p| {
            (0..LEG_COUNT)
                .filter(|l| !m.broken(ankle_part(*l)))
                .find(|l| math::wide_flat_dist(gait::foot(m, *l), p).raw() <= near.raw())
        });
        let held = w.lore.word(word::CLING_AT);
        let was = ((held >> (4 * who)) & 0xF) as usize;
        let now = found.map_or(0, |l| l + 1);
        let frames = if now != 0 && now == was {
            half_of(w, word::CLING, who).saturating_add(1)
        } else {
            0
        };
        set_half(w, word::CLING, who, frames);
        let held = (held & !(0xF << (4 * who))) | ((now as u32) << (4 * who));
        w.lore.set_word(word::CLING_AT, held);
    }
}

/// Start a legs' move, if one is wanted and there is room for it.
fn choose(w: &mut World, m: &mut Monster) {
    let floor = on_floor(w);
    // **The drag**: somebody has clung to one ankle too long.
    for who in 0..MAX_PLAYERS {
        if half_of(w, word::CLING, who) < Knob::ClingFrames.raw().max(1) as u16 {
            continue;
        }
        let l = ((w.lore.word(word::CLING_AT) >> (4 * who)) & 0xF) as usize;
        if l == 0 || l > LEG_COUNT {
            continue;
        }
        let l = l - 1;
        if !can_move(w, m, l, leg::DRAG) {
            continue;
        }
        begin(w, m, l, leg::DRAG, inward(m, l));
        set_half(w, word::CLING, who, 0);
        return;
    }
    // **The stamp**: somebody under where a planted foot could come down,
    // on a beat it may -- `StampChance` of the beats it could.
    let reach = Knob::StampReach.fx();
    let mut best: Option<(usize, V3, Fx)> = None;
    for l in 0..LEG_COUNT {
        if !can_move(w, m, l, leg::STAMP) {
            continue;
        }
        let home = gait::foot(m, l);
        for at in floor.iter().flatten() {
            let d = math::wide_flat_dist(home, *at);
            if d.raw() <= reach.raw() && best.is_none_or(|(_, _, seen)| d.raw() < seen.raw()) {
                best = Some((l, *at, d));
            }
        }
    }
    if let Some((l, at, _)) = best {
        // One roll a beat: decided on the first frame it could, and not again
        // until the next beat lets it.
        let chance = Knob::StampChance.raw().clamp(0, 100) as u32;
        if roll(m) % 100 < chance {
            begin(w, m, l, leg::STAMP, at);
        } else {
            w.lore.set_word(
                word::LEG_REST,
                gait::frames_to_beat(m).map_or(60, |f| f as u32),
            );
        }
    }
}

/// Can leg `l` throw a legs' move of `kind` now: sound, off its lockout,
/// planted for the whole move, and room for it on both channels.
fn can_move(w: &World, m: &Monster, l: usize, kind: u32) -> bool {
    if m.broken(ankle_part(l)) || lockout(w, l) > 0 {
        return false;
    }
    if !room_for(m, kind) {
        return false;
    }
    match (planted_for(m, l), frames_to_lift(m, l)) {
        (None, _) => false,
        (Some(_), None) => true,
        (Some(_), Some(lift)) => lift > length(kind) as i32,
    }
}

/// Where a drag sweeps a foot: inward, toward the belly, `DragArc` along.
fn inward(m: &Monster, l: usize) -> V3 {
    let from = gait::foot(m, l);
    let side = Fx::from_int(super::leg_side(l));
    let across = gait::flat_world(m, V3::new(Fx::ZERO, Fx::ZERO, side.neg())).sub(m.pos);
    from.add(across.scale(Knob::DragArc.fx()))
}

fn begin(w: &mut World, m: &mut Monster, l: usize, kind: u32, aim: V3) {
    leg::set(
        m,
        leg::Channel {
            kind,
            leg: l,
            phase: leg::STARTUP,
            left: frames(kind, leg::STARTUP),
            struck: 0,
        },
    );
    leg::aim_at(m, aim);
    set_lockout(w, l, Knob::StampLockout.raw().clamp(0, 0xFFFF) as u16);
}

/// The move under way, a frame on: a stamp's aim follows whoever is under
/// it through the windup; the hit; the foot home.
fn tick(w: &mut World, m: &mut Monster, mut c: leg::Channel) {
    if c.phase == leg::STARTUP && c.kind == leg::STAMP {
        // The lifted foot follows, slowly, whoever is nearest under it.
        let aim = leg::aim(m);
        let near = on_floor(w)
            .iter()
            .flatten()
            .copied()
            .min_by_key(|p| math::wide_flat_dist(*p, aim).raw());
        if let Some(to) = near {
            let step = Knob::StampTracking.fx().mul(crate::DT);
            let gap = V3::new(to.x.sub(aim.x), Fx::ZERO, to.z.sub(aim.z));
            let d = math::wide_flat_len(gap);
            let moved = if d.raw() <= step.raw() {
                to
            } else {
                aim.add(math::wide_normalized(gap).scale(step))
            };
            // Never further from its own foot than it could reach.
            let home = gait::foot_in_gait(m, c.leg);
            let off = V3::new(moved.x.sub(home.x), Fx::ZERO, moved.z.sub(home.z));
            let reach = Knob::StampReach.fx();
            let kept = if math::wide_flat_len(off).raw() > reach.raw() {
                home.add(math::wide_normalized(off).scale(reach))
            } else {
                moved
            };
            leg::aim_at(m, V3::new(kept.x, Fx::ZERO, kept.z));
        }
    }
    if c.phase == leg::ACTIVE {
        hits(w, m, &mut c);
    }
    if c.left > 0 {
        c.left -= 1;
        leg::set(m, c);
        return;
    }
    if c.phase == leg::RETURN {
        leg::clear(m);
        w.lore
            .set_word(word::LEG_REST, Knob::LegRest.raw().max(0) as u32);
        return;
    }
    c.phase += 1;
    c.left = frames(c.kind, c.phase);
    leg::set(m, c);
}

/// What the legs' move hits this frame: the stamp's pad on whoever is under
/// it; the drag's pad on whoever its sweep passes.
fn hits(w: &mut World, m: &Monster, c: &mut leg::Channel) {
    let kind = if c.kind == leg::DRAG { DRAG } else { STAMP };
    let a = m.sp().attack(kind);
    let reach = Knob::PadRadius.fx().add(crate::tuning::body_radius());
    let now = foot_in_move(m, *c).unwrap_or(leg::aim(m));
    let before = {
        let mut was = *c;
        was.left = was.left.saturating_add(1).min(frames(c.kind, c.phase));
        foot_in_move(m, was).unwrap_or(now)
    };
    for who in 0..MAX_PLAYERS {
        if c.struck & (1 << who) != 0 {
            continue;
        }
        let p = w.players[who];
        if p.health <= 0 || p.aboard() || p.action.invulnerable() || p.action.stunned() {
            continue;
        }
        if p.pos.y.raw() > a.hit_high.raw() {
            continue;
        }
        let gap = math::flat_segment_gap(p.pos, before, now);
        if gap.raw() <= reach.raw() {
            fight::strike(w, who, now, a.damage, kind);
            c.struck |= 1 << who;
        }
    }
}

/// **Where a leg's pad is while the legs' channel has it**: lifted and carried
/// over its aim through a stamp's windup and driven down onto it, held there,
/// and taken home; swept inward along the floor through a drag.
pub fn foot_in_move(m: &Monster, c: leg::Channel) -> Option<V3> {
    let home = gait::foot_in_gait(m, c.leg);
    let aim = leg::aim(m);
    let t = progress(c);
    let ease = math::smoothstep;
    match (c.kind, c.phase) {
        (leg::STAMP, leg::STARTUP) => {
            // Up and over, then down for the last `StampDrop` of it: it arrives
            // on the first frame it is out.
            let rise = Fx::ONE
                .sub(Knob::StampDrop.fx())
                .clamp(crate::arena::SKIN, Fx::ONE);
            let lift = Knob::StampLift.fx();
            if t.raw() < rise.raw() {
                let k = ease(t.div(rise));
                let over = math::lerp3(home, aim, k);
                Some(V3::new(over.x, lift.mul(k), over.z))
            } else {
                let k = t.sub(rise).div(Fx::ONE.sub(rise));
                Some(V3::new(aim.x, lift.mul(Fx::ONE.sub(k.mul(k))), aim.z))
            }
        }
        (leg::STAMP, leg::ACTIVE | leg::RECOVERY) => Some(aim),
        (leg::DRAG, leg::STARTUP) => Some(home),
        (leg::DRAG, leg::ACTIVE) => Some(math::lerp3(home, aim, ease(t))),
        (leg::DRAG, leg::RECOVERY) => Some(aim),
        (_, leg::RETURN) => {
            let k = ease(t);
            let back = math::lerp3(aim, home, k);
            let arc = Knob::Lift.fx().mul(math::hump(k));
            Some(V3::new(back.x, arc, back.z))
        }
        _ => None,
    }
}

/// **What the legs' move draws** (§6): a stamp's pad in the stamp's own
/// shape -- a filled disc, where the beat's pads are rings -- moving while it
/// tracks; a drag's lane from the outside of the leg sweeping in.
pub fn signs(_w: &World, m: &Monster, out: &mut Signs) {
    let c = leg::get(m);
    if c.kind == leg::NONE || c.phase >= leg::RECOVERY {
        return;
    }
    let pad = Knob::PadRadius.fx();
    let says = if c.phase == leg::ACTIVE {
        Says::Live
    } else {
        Says::Coming
    };
    let fill = if c.phase == leg::ACTIVE {
        Fx::ONE
    } else {
        progress(c)
    };
    match c.kind {
        leg::STAMP => out.push(Sign::disc(says, leg::aim(m), pad.add(pad)).filled(fill)),
        leg::DRAG => {
            let from = gait::foot_in_gait(m, c.leg);
            let to = leg::aim(m);
            let d = V3::new(to.x.sub(from.x), Fx::ZERO, to.z.sub(from.z));
            let len = math::wide_flat_len(d);
            if len.raw() > 0 {
                out.push(
                    Sign::strip(says, from, math::wide_normalized(d), len, pad.add(pad))
                        .filled(fill),
                );
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// The parasites
// ---------------------------------------------------------------------------

pub const KINDS: [crate::critter::CritterKind; 1] = [gnawers::GNAWER_KIND];

pub static PACK: PackDecl = PackDecl {
    kinds: &KINDS,
    muster: &[],
    leader: None,
    mind: &Roost,
};

/// The parasites' mind: the gnawers', and the shell's roost on top.
///
/// **A parasite on the shell roosts** -- stays where it is and throws nothing
/// -- until something calls the pack down (§3, §5): somebody standing under
/// the belly for `RoostFrames`, somebody trailing more than `Straggler` behind
/// the tail for `StragglerFrames`, or, once an anchor is broken and the pack
/// is roused, anybody on the shell at all. Called, every parasite is a
/// gnawer: it goes for whoever is nearest by the pack's own rules, walks off
/// the shell's edge after them and drops. One on the floor is never recalled.
pub struct Roost;

const GNAWER: gnawers::Mind = gnawers::Mind;

/// The owner, if it stands.
fn shell<'a>(pack: &Pack, herd: &'a Herd) -> Option<&'a Monster> {
    herd.get(pack.owner as usize)
        .and_then(|m| m.as_ref())
        .filter(|m| m.alive())
}

/// Is critter `c` roosting: on the shell, and the pack not called down (its
/// frame decides that, and says so on the body: `fight::called`).
fn roosting(pack: &Pack, herd: &Herd, c: &Critter) -> bool {
    c.mounted() && !shell(pack, herd).is_some_and(fight::called)
}

impl PackMind for Roost {
    fn appetite(&self, look: &Look, i: usize, m: CritterMove, a: &Attack) -> i32 {
        if roosting(look.pack, look.herd, &look.critters[i]) {
            return 0;
        }
        GNAWER.appetite(look, i, m, a)
    }

    fn frame(&self, pack: &mut Pack, critters: &mut Critters, herd: &Herd, frame: u32) {
        GNAWER.frame(pack, critters, herd, frame);
    }

    fn steer(&self, look: &Look, i: usize, want: Steer) -> Steer {
        let c = &look.critters[i];
        if roosting(look.pack, look.herd, c) {
            return Steer {
                to: c.pos,
                speed: Fx::ZERO,
                face: None,
            };
        }
        GNAWER.steer(look, i, want)
    }

    fn landed(
        &self,
        pack: &mut Pack,
        critters: &mut Critters,
        i: usize,
        who: usize,
        victim: &mut Player,
        blocked: bool,
        parried: bool,
    ) {
        GNAWER.landed(pack, critters, i, who, victim, blocked, parried);
    }

    fn hurt(&self, pack: &mut Pack, critters: &mut Critters, i: usize, dealt: i32) {
        GNAWER.hurt(pack, critters, i, dealt);
    }

    fn died(&self, pack: &mut Pack, critters: &mut Critters, i: usize) {
        GNAWER.died(pack, critters, i);
    }

    fn body(&self, c: &Critter, plain: Body) -> Body {
        GNAWER.body(c, plain)
    }
}

/// The species, for the hooks that are handed less than a monster.
pub fn species() -> &'static crate::species::Species {
    &SPECIES
}
