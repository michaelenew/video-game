//! What the Siegeshell does to the valley, its riders and the wall, run once a
//! frame from its `FightDecl::frame` hook -- and the hooks the shared machinery
//! calls it by when a blow lands on it.
//!
//! **The walk** is here: it holds its body still against the shared walk
//! (`rooted`, as the Pair and the Veilstalker do) and moves it itself, at the
//! pace [`gait::pace`] gives, toward the siege line; the stride advances with
//! the ground covered, and a stride that crosses a tripod's landing phase is
//! **a footfall** -- the pads come down, and a shockwave ring sweeps out from
//! each of the three feet over the next dozen frames.
//!
//! **Its health is its anchors** ([`struck`]): no blow on the shell, the legs
//! or the head takes any (the hit is refunded, after the strain it buys), and
//! the health that remains is always the share of the three anchors' health
//! still standing, so the strain thresholds fall as the anchors go and the
//! last anchor's break is its death. Ankles and anchors keep their own health
//! in `Monster::breaks`; a broken ankle's goes on below zero, and every
//! `BuckleHealth` of it there is a buckle.
//!
//! The four words on its body (`Monster::own`, [`body`]) are what the hooks
//! without a world read: the stumble's side, the open anchor, the legs'
//! channel and its aim. Everything else is the lore's ([`word`]).

use super::gait;
use super::{
    ANCHOR_COUNT, FOOTFALL, Knob, LEG_COUNT, PAD0, PART_COUNT, SPECIES, anchor_of, anchor_part,
    ankle_of, ankle_part, leg_side, thigh_part, tripod,
};
use crate::DT;
use crate::fixed::Fx;
use crate::hazard::{HazardDecl, reach};
use crate::lore::{self, Layout};
use crate::math::{self, V3};
use crate::monster::{Doing, Monster};
use crate::objective::ObjectiveDecl;
use crate::sign::{Says, Sign, Signs};
use crate::species::{FightDecl, SpeciesId};
use crate::state::{self, Hit, MAX_PLAYERS, QUARRY, World};

// ---------------------------------------------------------------------------
// The floor's kinds, and the wall
// ---------------------------------------------------------------------------

/// A vent's grate between blasts: drawn, and nothing else.
pub const GRATE: u8 = 0;
/// A vent blowing: scalds and lifts whoever stands in its column.
pub const BLAST: u8 = 1;
/// The gate's rubble, after the first breach: a solid in the valley's end.
pub const RUBBLE: u8 = 2;

pub const HAZARDS: [HazardDecl; 3] = [
    HazardDecl::disc("grate").reaching(reach::FIGHTERS),
    HazardDecl::disc("vent").reaching(reach::FIGHTERS | reach::CRITTERS),
    HazardDecl::disc("rubble").solid(crate::arena::Material::Rock),
];

/// The town wall, at the valley's end: the hunt is lost when it falls.
pub const WALL: usize = 0;

pub const OBJECTIVES: [ObjectiveDecl; 1] = [ObjectiveDecl::wall("the wall", 0)];

// ---------------------------------------------------------------------------
// Its state
// ---------------------------------------------------------------------------

/// Its own words in the hunt's lore.
pub mod word {
    /// Bits: [`super::flag`].
    pub const FLAGS: usize = 0;
    /// The last footfall: frames since it (low half), which tripod (bit 16),
    /// and the fighters its ring has already struck (bits 24 up).
    pub const RING: usize = 1;
    /// The siege: beams fired (low byte).
    pub const SIEGE: usize = 2;
    /// Each leg's stamp lockout, two legs a word (low half, high half).
    pub const STAMP_LOCK0: usize = 3;
    /// Frames each fighter has clung to one ankle (a half each), and which
    /// ankle each clings to (a nibble each, plus one) in the next word.
    pub const CLING: usize = 6;
    pub const CLING_AT: usize = 7;
    /// Frames each fighter has stood under the belly (a half each).
    pub const ROOST: usize = 8;
    /// Frames each fighter has straggled behind the tail (a half each).
    pub const STRAGGLE: usize = 9;
    /// The brood's clock: frames since the last parasite crawled out.
    pub const BROOD: usize = 10;
    /// The legs' channel's pause after a move, and the frame the body's last
    /// active window ended (for the channel gap).
    pub const LEG_REST: usize = 11;
    pub const BODY_ENDED: usize = 12;
    /// The body move under way: the fighters it has struck.
    pub const BODY_STRUCK: usize = 13;
    /// The vents' clocks: where each grate is in its cycle (a half each).
    pub const VENT0: usize = 14;
}

/// The bits of [`word::FLAGS`].
pub mod flag {
    /// The round has been set up: the anchors' health, the vents, the brood.
    pub const SET_UP: u32 = 1;
    /// The gate is down: the first breach.
    pub const GATE_DOWN: u32 = 1 << 1;
}

/// Its four words on the body (`Monster::own`): what the hit path and the
/// rig read without a world.
pub mod body {
    use crate::monster::Monster;

    /// The stumble's side (bits 0-1: 0 none, 1 left, 2 right), the open
    /// anchor plus one (bits 2-3), a stumble begun and not yet looked at by
    /// the frame (bit 4), halted at the siege line (bit 5), an anchor broken
    /// and not yet looked at (bit 6).
    pub const FLAGS: usize = 0;
    /// The legs' channel: see [`super::leg`].
    pub const LEG: usize = 1;
    /// Where the legs' move is aimed, in centimetres (`lore::halves`).
    pub const LEG_AIM: usize = 2;

    pub const SIDE_MASK: u32 = 0b11;
    pub const OPEN_SHIFT: u32 = 2;
    pub const OPEN_MASK: u32 = 0b11 << 2;
    pub const STUMBLE_NEW: u32 = 1 << 4;
    pub const SIEGE: u32 = 1 << 5;
    pub const ANCHOR_NEW: u32 = 1 << 6;

    pub fn flags(m: &Monster) -> u32 {
        m.own[FLAGS] as u32
    }

    pub fn set(m: &mut Monster, bits: u32, on: bool) {
        let f = flags(m);
        m.own[FLAGS] = if on { f | bits } else { f & !bits } as i32;
    }
}

/// **The legs' channel** (§5): a stamp or a drag, on one leg, with its own
/// frames -- beside whatever the body is doing. Packed into one word on the
/// body because the pose reads it: a stamping foot is collision geometry.
pub mod leg {
    use super::body;
    use crate::fixed::Fx;
    use crate::math::V3;
    use crate::monster::Monster;

    /// Nothing.
    pub const NONE: u32 = 0;
    pub const STAMP: u32 = 1;
    pub const DRAG: u32 = 2;

    /// Winding up, out, recovering, and the foot going home.
    pub const STARTUP: u32 = 0;
    pub const ACTIVE: u32 = 1;
    pub const RECOVERY: u32 = 2;
    pub const RETURN: u32 = 3;

    /// The channel as it stands: which move, which leg, which phase, frames
    /// left.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Channel {
        pub kind: u32,
        pub leg: usize,
        pub phase: u32,
        pub left: u16,
    }

    pub fn get(m: &Monster) -> Channel {
        let w = m.own[body::LEG] as u32;
        Channel {
            kind: w & 0b11,
            leg: ((w >> 2) & 0b111) as usize,
            phase: (w >> 5) & 0b11,
            left: ((w >> 7) & 0x7FF) as u16,
        }
    }

    pub fn set(m: &mut Monster, c: Channel) {
        let w = (c.kind & 0b11)
            | ((c.leg as u32 & 0b111) << 2)
            | ((c.phase & 0b11) << 5)
            | ((c.left as u32 & 0x7FF) << 7);
        m.own[body::LEG] = w as i32;
    }

    pub fn clear(m: &mut Monster) {
        m.own[body::LEG] = 0;
        m.own[body::LEG_AIM] = 0;
    }

    /// The legs' aim point, on the floor.
    pub fn aim(m: &Monster) -> V3 {
        let w = m.own[body::LEG_AIM] as u32;
        V3::new(
            crate::lore::from_cm(crate::lore::lo(w)),
            Fx::ZERO,
            crate::lore::from_cm(crate::lore::hi(w)),
        )
    }

    pub fn aim_at(m: &mut Monster, at: V3) {
        m.own[body::LEG_AIM] =
            crate::lore::halves(crate::lore::to_cm(at.x), crate::lore::to_cm(at.z)) as i32;
    }

    /// **Where the legs' channel has leg `leg`'s pad**, if it has it: the
    /// underside's middle, in the world. `None` leaves it to the gait.
    pub fn moving(m: &Monster, leg: usize) -> Option<V3> {
        let c = get(m);
        if c.kind == NONE || c.leg != leg {
            return None;
        }
        super::super::mind::foot_in_move(m, c)
    }
}

/// The side a stumble is putting down: `-1` left, `+1` right, `0` none.
pub fn stumble_side(m: &Monster) -> i32 {
    match body::flags(m) & body::SIDE_MASK {
        1 => -1,
        2 => 1,
        _ => 0,
    }
}

/// The anchor its plates have parted from, if one is open.
pub fn open_anchor(m: &Monster) -> Option<usize> {
    match (body::flags(m) & body::OPEN_MASK) >> body::OPEN_SHIFT {
        0 => None,
        n => Some(n as usize - 1),
    }
}

/// Halted at the siege line.
pub fn at_siege_line(m: &Monster) -> bool {
    body::flags(m) & body::SIEGE != 0
}

/// How many anchors are broken.
pub fn anchors_broken(m: &Monster) -> usize {
    (0..ANCHOR_COUNT)
        .filter(|a| m.broken(anchor_part(*a)))
        .count()
}

/// Its phase (§4): 0 walking, 1 roused, 2 hurried, 3 dead.
pub fn phase(m: &Monster) -> usize {
    anchors_broken(m)
}

/// How many times a broken ankle has buckled.
pub fn buckles(m: &Monster, leg: usize) -> i32 {
    let part = ankle_part(leg);
    let h = m.part_health(part);
    let every = Knob::BuckleHealth.raw().max(1);
    if h >= 0 { 0 } else { (-h) / every }
}

// ---------------------------------------------------------------------------
// The table
// ---------------------------------------------------------------------------

pub static FIGHT: FightDecl = FightDecl {
    layout: Layout {
        hazards: 10,
        noises: 0,
        objectives: 1,
        own: 8,
    },
    hazards: &HAZARDS,
    objectives: &OBJECTIVES,
    frame: Some(frame),
    struck: Some(struck),
    hide: Some(hide),
    presence: Some(presence),
    signs: Some(signs),
    appetite: Some(super::mind::appetite),
    prowl_to: Some(super::mind::prowl_to),
    commit: Some(super::mind::commit),
    clip: Some(super::mind::posture),
    repose: Some(gait::repose),
    ..FightDecl::PLAIN
};

// ---------------------------------------------------------------------------
// The body's hooks
// ---------------------------------------------------------------------------

/// The health it has: its anchors' share of what they started with. See the
/// module docs.
pub fn health_of(m: &Monster) -> i32 {
    let most = Knob::AnchorHealth.raw().max(1) * ANCHOR_COUNT as i32;
    let left: i32 = (0..ANCHOR_COUNT)
        .map(|a| m.part_health(anchor_part(a)).max(0))
        .sum();
    ((m.sp().health() as i64 * left.min(most) as i64) / most as i64) as i32
}

/// **A blow has landed** (`FightDecl::struck`): what it did, decided here and
/// nowhere else. The shared ladder -- a flinch, a topple, the break of a leg
/// -- is the Ridgeback's, and none of it is this animal's.
pub fn struck(m: &mut Monster, part: usize, dealt: i32) -> bool {
    let sp = m.sp();
    if let Some(a) = anchor_of(part) {
        if let Some(slot) = sp.break_slot(part) {
            if m.breaks[slot] > 0 {
                m.breaks[slot] = (m.breaks[slot] - dealt).max(0);
                if m.breaks[slot] == 0 {
                    anchor_broke(m, a);
                }
            }
        }
    } else if let Some(leg) = ankle_of(part) {
        if let Some(slot) = sp.break_slot(part) {
            let was_broken = m.breaks[slot] <= 0;
            let buckled = buckles(m, leg);
            m.breaks[slot] -= dealt;
            if !was_broken && m.breaks[slot] <= 0 {
                // Broken: it drags, and the next on its side brings it down.
                m.breaks[slot] = 0;
                ankle_broke(m, leg);
            } else if was_broken && buckles(m, leg) > buckled {
                // A buckle: a broken ankle folding once more. On a side with
                // two broken, it is another stumble.
                ankle_broke(m, leg);
            }
        }
        // **The interrupt, for the legs**: enough strain stops a stamp.
        if m.strain >= m.interrupt_bar() && leg::get(m).kind != leg::NONE {
            m.strain = 0;
            leg::clear(m);
        }
    }
    m.health = health_of(m).max(1);
    if anchors_broken(m) >= ANCHOR_COUNT {
        m.health = 0;
        m.doing = Doing::Dead;
    }
    true
}

/// An ankle broke or buckled on leg `leg`: with two broken on its side, the
/// side goes down.
fn ankle_broke(m: &mut Monster, leg: usize) {
    let side = leg_side(leg);
    let (l, r) = gait::broken_sides(m);
    let broken = if side < 0 { l } else { r };
    if broken >= 2 && !matches!(m.doing, Doing::Toppled { .. } | Doing::Dead) {
        stumble(m, side);
    }
    // A leg that has gone is out of the legs' channel.
    if leg::get(m).leg == leg {
        leg::clear(m);
    }
}

/// **The side goes down**: twelve seconds of a stumble, the walk halted, the
/// broken legs splayed into a stair. The frame looks next at who is at an
/// anchor (the Opening).
pub fn stumble(m: &mut Monster, side: i32) {
    m.doing = Doing::Stumble {
        left: m.sp().stumble_frames(),
        front: true,
    };
    let f = body::flags(m) & !body::SIDE_MASK & !body::OPEN_MASK;
    let s = if side < 0 { 1 } else { 2 };
    m.own[body::FLAGS] = (f | s | body::STUMBLE_NEW) as i32;
    m.speed = Fx::ZERO;
    m.yaw_rate = Fx::ZERO;
    m.strain = 0;
    leg::clear(m);
}

/// An anchor broke: it kneels for twenty seconds, whatever it was doing --
/// a beam charging included -- and its phase is one on.
fn anchor_broke(m: &mut Monster, _anchor: usize) {
    if anchors_broken(m) >= ANCHOR_COUNT {
        return;
    }
    m.doing = Doing::Toppled {
        left: m.sp().topple_frames(),
    };
    m.speed = Fx::ZERO;
    m.yaw_rate = Fx::ZERO;
    m.strain = 0;
    let f = body::flags(m) & !body::SIDE_MASK & !body::OPEN_MASK;
    m.own[body::FLAGS] = (f | body::ANCHOR_NEW) as i32;
    leg::clear(m);
}

/// **What its hide is worth on a part** (`FightDecl::hide`): an open anchor
/// takes `OpenDamage` times.
pub fn hide(m: &Monster, part: usize) -> Fx {
    match (anchor_of(part), open_anchor(m)) {
        (Some(a), Some(open)) if a == open => Knob::OpenDamage.fx(),
        _ => Fx::ONE,
    }
}

/// **What of it can be stood on this frame** (`FightDecl::presence`): a thigh
/// only splayed -- a stair -- and a broken anchor not at all: it is a stump.
pub fn presence(m: &Monster, _rig: &crate::beast::Rig) -> crate::beast::Presence {
    let mut p = crate::beast::Presence::default();
    for leg in 0..LEG_COUNT {
        if gait::splayed(m, leg).raw() < Fx::ratio(1, 2).raw() {
            p.unmountable |= 1u64 << thigh_part(leg);
        }
    }
    for a in 0..ANCHOR_COUNT {
        if m.broken(anchor_part(a)) {
            p.buried |= 1u64 << anchor_part(a);
        }
    }
    let _ = PART_COUNT;
    p
}

// ---------------------------------------------------------------------------
// Once a frame
// ---------------------------------------------------------------------------

/// The slot it stands in, if it is in this world.
pub fn slot_of(w: &World) -> Option<usize> {
    w.monsters
        .iter()
        .position(|m| m.is_some_and(|m| m.species == SpeciesId::SIEGESHELL))
}

/// **Its rules, once a frame** (`FightDecl::frame`).
pub fn frame(w: &mut World) {
    let Some(slot) = slot_of(w) else { return };
    if w.lore.word(word::FLAGS) & flag::SET_UP == 0 {
        set_up(w, slot);
    }
    walk(w, slot);
    footfall(w, slot);
    super::mind::legs(w, slot);
}

/// The round's first frame: every anchor at its own health.
fn set_up(w: &mut World, slot: usize) {
    let Some(m) = w.monsters[slot].as_mut() else {
        return;
    };
    let sp = m.sp();
    for a in 0..ANCHOR_COUNT {
        if let Some(s) = sp.break_slot(anchor_part(a)) {
            m.breaks[s] = Knob::AnchorHealth.raw().max(1);
        }
    }
    m.health = health_of(m).max(1);
    let f = w.lore.word(word::FLAGS);
    w.lore.set_word(word::FLAGS, f | flag::SET_UP);
}

/// **The walk**: its own pace, along its own heading, the stride with it. The
/// shared walk is held (`rooted`), so this is the only thing that moves it.
fn walk(w: &mut World, slot: usize) {
    let Some(mut m) = w.monsters[slot] else {
        return;
    };
    m.rooted = m.rooted.max(2);
    // The siege line: its head this far from the wall.
    if !at_siege_line(&m) && to_wall(w, &m).raw() <= Knob::SiegeLine.fx().raw() {
        body::set(&mut m, body::SIEGE, true);
    }
    let speed = gait::pace(&m, phase(&m));
    let forward = V3::from_turns(m.yaw);
    m.pos = m.pos.add(forward.scale(speed.mul(DT)));
    m.pos.y = Fx::ZERO;
    let before = m.stride;
    let covered = speed.mul(DT).div(m.sp().gait_stride().max(Fx::ratio(1, 2)));
    m.stride = m.stride.wrapping_add(covered.raw().clamp(0, 65535) as u16);
    m.speed = speed;
    if let Some(t) = gait::landed(before, m.stride) {
        // A beat: the ring starts again from these three feet.
        w.lore.set_word(word::RING, (t as u32) << 16);
    } else {
        let r = w.lore.word(word::RING);
        let age = (r & 0xFFFF).saturating_add(1).min(0xFFFF);
        w.lore.set_word(word::RING, (r & !0xFFFF) | age);
    }
    w.monsters[slot] = Some(m);
}

/// How far its head is from the wall, along the valley.
pub fn to_wall(w: &World, m: &Monster) -> Fx {
    let Some(site) = w.arena().sites.first() else {
        return Fx::MAX;
    };
    let (wall, _) = site.at(Fx::ZERO);
    let half = site.extent().x;
    let head = gait::flat_world(m, V3::new(head_reach(), Fx::ZERO, Fx::ZERO));
    wall.x.sub(half).sub(head.x)
}

/// How far ahead of its middle its head reaches, standing.
pub fn head_reach() -> Fx {
    let sp = &SPECIES;
    let neck = sp.rest(super::bones::NECK).x;
    let head = sp.rest(super::bones::HEAD).x;
    neck.add(head).add(sp.shape(super::HEAD_PART).max.x)
}

// ---------------------------------------------------------------------------
// The footfall
// ---------------------------------------------------------------------------

/// The footfall's ring this frame: which tripod, how many frames since it
/// landed, and the fighters it has struck.
pub fn ring(w: &World) -> (usize, u16, u32) {
    let r = w.lore.word(word::RING);
    (((r >> 16) & 1) as usize, (r & 0xFFFF) as u16, r >> 24)
}

/// The ring's radius `age` frames after its feet landed: from the pad's edge
/// out to `RingTo` over `RingFrames`.
pub fn ring_radius(age: u16) -> Fx {
    let frames = Knob::RingFrames.raw().max(1);
    let k = Fx::from_int((age as i32).min(frames)).div(Fx::from_int(frames));
    math::lerp(Knob::RingFrom.fx(), Knob::RingTo.fx(), k)
}

/// Is the ring out this frame?
pub fn ring_live(w: &World) -> bool {
    let (_, age, _) = ring(w);
    age >= 1 && (age as i32) <= Knob::RingFrames.raw() && w.frame > 1
}

/// The legs of a tripod.
pub fn tripod_legs(t: usize) -> impl Iterator<Item = usize> {
    (0..LEG_COUNT).filter(move |l| tripod(*l) == t)
}

/// Is a fighter where a ring or a pad on the floor can reach: alive, on the
/// floor rather than the creature, not in a dodge's invulnerable frames, and
/// not helpless -- a body in a throw's stun is not caught by the beat
/// (`a_thrown_rider_is_not_caught_by_the_beat`).
fn reachable(p: &state::Player) -> bool {
    p.health > 0 && !p.aboard() && !p.action.invulnerable() && !p.action.stunned()
}

/// **The footfall** (§2): on the beat, the pads land on whoever is under them;
/// for the dozen frames after, the ring sweeps out from each of the three
/// feet, knee-high, and catches anyone on the floor it passes.
fn footfall(w: &mut World, slot: usize) {
    let Some(m) = w.monsters[slot] else { return };
    let (t, age, mut struck) = ring(w);
    if age == 0 && w.frame > 1 {
        // The pads, on the frame they land.
        let pad = Knob::PadRadius.fx().add(crate::tuning::body_radius());
        for i in 0..MAX_PLAYERS {
            let p = w.players[i];
            if !reachable(&p) || p.pos.y.raw() > Knob::RingHeight.fx().raw() {
                continue;
            }
            let near = tripod_legs(t)
                .find(|l| math::wide_flat_dist(gait::foot(&m, *l), p.pos).raw() <= pad.raw());
            if let Some(l) = near {
                strike(w, i, gait::foot(&m, l), Knob::PadDamage.raw(), FOOTFALL);
                struck |= 1 << i;
            }
        }
    } else if ring_live(w) {
        let r0 = ring_radius(age.saturating_sub(1));
        let r1 = ring_radius(age);
        let body = crate::tuning::body_radius();
        for i in 0..MAX_PLAYERS {
            let p = w.players[i];
            if struck & (1 << i) != 0 || !reachable(&p) {
                continue;
            }
            if p.pos.y.raw() > Knob::RingHeight.fx().raw() {
                continue;
            }
            let hit = tripod_legs(t).find(|l| {
                let d = math::wide_flat_dist(gait::foot(&m, *l), p.pos);
                d.raw() >= r0.sub(body).raw() && d.raw() <= r1.add(body).raw()
            });
            if let Some(l) = hit {
                let a = m.sp().attack(FOOTFALL);
                strike(w, i, gait::foot(&m, l), a.damage, FOOTFALL);
                struck |= 1 << i;
            }
        }
    }
    let r = w.lore.word(word::RING);
    w.lore
        .set_word(word::RING, (r & 0x00FF_FFFF) | (struck << 24));
}

/// **One of its own hits lands on a fighter**: `damage`, away from `from`,
/// with the move's own stun and knockback. Unblockable unless the move's row
/// says otherwise; a guard facing `from` takes a blockable one.
pub fn strike(w: &mut World, who: usize, from: V3, damage: i32, kind: u8) {
    let a = SPECIES.attack(kind);
    let p = w.players[who];
    let away = V3::new(p.pos.x.sub(from.x), Fx::ZERO, p.pos.z.sub(from.z));
    let away = if math::wide_flat_len(away).raw() > 0 {
        math::wide_normalized(away)
    } else {
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO)
    };
    let facing_it =
        p.facing.dot(away.scale(Fx::ONE.neg())).raw() >= crate::tuning::guard_arc_cos().raw();
    let guarding = !a.unblockable && p.action.guarding() && facing_it;
    state::apply_hit(
        &mut w.players[who],
        Hit {
            damage,
            hitstun: a.hitstun,
            blockstun: a.blockstun,
            knockback: a.knockback,
            launch: a.launch,
            grabs: 0,
            by: QUARRY,
            dir: away,
            blocked: guarding,
            parried: false,
            interrupts: true,
        },
    );
}

// ---------------------------------------------------------------------------
// What it draws
// ---------------------------------------------------------------------------

/// **What it draws on the floor** (§6): every lifted foot's landing spot, its
/// ring faintly outside it, the live ring, and the legs' move.
pub fn signs(w: &World, out: &mut Signs) {
    let Some(slot) = slot_of(w) else { return };
    let Some(m) = w.monsters[slot] else { return };
    if !m.alive() {
        return;
    }
    let width = |r: Fx| r.add(r);
    // The live ring.
    if ring_live(w) {
        let (t, age, _) = ring(w);
        for l in tripod_legs(t) {
            out.push(Sign::ring(
                Says::Live,
                gait::foot(&m, l),
                width(ring_radius(age)),
            ));
        }
    }
    // The legs' move.
    super::mind::signs(w, &m, out);
    // Every foot in the air: where it lands, filling as it comes down, and
    // the ring it will throw.
    for l in 0..LEG_COUNT {
        if leg::moving(&m, l).is_some() {
            continue;
        }
        let Some(swing) = gait::swinging(&m, l) else {
            continue;
        };
        if m.speed.raw() <= 0 {
            continue;
        }
        let at = gait::landing_spot(&m, l);
        out.push(Sign::ring(Says::Coming, at, width(Knob::PadRadius.fx())).filled(swing));
        out.push(Sign::ring(Says::Faint, at, width(Knob::RingTo.fx())).filled(swing));
    }
    let _ = PAD0;
}

/// Every fighter, as a bit set, who is alive.
pub fn living(w: &World) -> u32 {
    (0..MAX_PLAYERS)
        .filter(|i| w.players[*i].health > 0)
        .fold(0, |acc, i| acc | 1 << i)
}

/// A point in the lore's centimetres.
pub fn cm_point(w: u32) -> V3 {
    V3::new(
        lore::from_cm(lore::lo(w)),
        Fx::ZERO,
        lore::from_cm(lore::hi(w)),
    )
}
