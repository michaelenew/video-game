//! The herd's mind: what differs from the generic pack (hornback.md §5).
//!
//! Every method of [`PackMind`] it overrides is a rule from the document:
//!
//! - **appetite**: the bull's scoring -- the range tent and the arc from each
//!   move's own row, a decaying penalty on the last move, wounded animals
//!   committing harder, and the four terms that are this fight: `downed` (the
//!   trample is only ever a follow-up), `seen_swing` (the guard is a reaction
//!   to a pose it saw), `herd_home` (the bellow is a reply to somebody in the
//!   open) and `wary` (it learns a wall once). A cow kicks only at somebody
//!   who has stood in its rear wedge for a whole glance.
//! - **frame**: the herd's five states, the clocks, the strain bleeding off.
//! - **steer**: the bull interposes; the herd grazes, bunches, runs, comes
//!   home and rallies, by separation, cohesion, a goal and avoidance -- no
//!   alignment (§5); the stampede itself is steered by the rules, which hold
//!   it to its lane.
//! - **guarded**: the bull's guard, and the horns.
//! - **landed**, **hurt**, **died**: knockdowns, strain, the nerve that sends
//!   a cow off through the ford.
//! - **mirrored** and **telegraph**: which side a move is thrown to, and the
//!   charge's lane as the rules worked it out.
//!
//! Its own state is the pack's memo ([`word`]), so the creature adds nothing
//! to the world but its words of lore.

use super::{
    BELLOW, BUCK, BULL, CHARGE, COW, GUARD, HOOK, KICK, Knob, SHOULDER, STAMPEDE, TRAMPLE, knob,
    knob_fx,
};
use crate::critter::{Critter, CritterField, CritterMove, Critters, flag, is, stat};
use crate::fixed::Fx;
use crate::lore::{from_cm, halves, hi, lo, to_cm};
use crate::math::V3;
use crate::monster::{Attack, Herd, Telegraph};
use crate::pack::{Blow, Guarded, Look, Pack, PackMind, Steer, default_appetite, mood, yaw_of};
use crate::state::{MAX_PLAYERS, Player};

/// The pack's memo, word by word. Twelve words.
pub mod word {
    /// Bytes: the herd's state ([`super::HerdState`]); the bull's last move
    /// plus one; [`super::bits`]; and per-fighter bits -- knocked down by this
    /// stampede (low two), seen in a swing's startup at the last glance
    /// (next two).
    pub const HERD: usize = 0;
    /// Two clocks: the herd's own (a stampede's run, a calm-down), and the
    /// bellow's lockout.
    pub const CLOCKS: usize = 1;
    /// The stampede's lane: where it starts, in centimetres.
    pub const LANE_AT: usize = 2;
    /// Its bearing (sixteen bits of a turn) and its length in centimetres.
    pub const LANE_DIR: usize = 3;
    /// The horns' health, left and right.
    pub const HORNS: usize = 4;
    /// The bull's strain: recent damage, bled each frame.
    pub const STRAIN: usize = 5;
    /// Two clocks: frames of wariness left, and the charge's lockout.
    pub const WARY: usize = 6;
    /// Where the solid it last hit stands, in centimetres.
    pub const WARY_AT: usize = 7;
    /// Two clocks: the guard's lockout, and frames every fighter has kept
    /// past `CalmRadius`.
    pub const GUARD: usize = 8;
    /// The charge being wound up or run: how far it will go, in centimetres;
    /// and frames left of the repeat penalty on the last move.
    pub const RUN: usize = 9;
    /// Where the charge's lane ends, in centimetres: the solid that will stop
    /// it, or the end of its run.
    pub const RUN_END: usize = 10;
    /// Two clocks: frames since the guard last turned a blow (the hook it
    /// wants after one), and the crossing's wave clock.
    pub const SINCE: usize = 11;
}

/// Bits of the third byte of [`word::HERD`].
pub mod bits {
    /// Laid: the boulders are down, the horns set, the coop numbers given.
    pub const LAID: u8 = 1;
    /// The charge being wound up ends in a solid: the lane is drawn cut short
    /// and the solid ringed.
    pub const ENDS_IN_SOLID: u8 = 2;
    /// The coop numbers have been given.
    pub const COOP_DONE: u8 = 4;
}

/// Bits of the bull's [`Critter::role`].
pub mod bull {
    /// Its flinch is the stun: a charge met a solid.
    pub const STUNNED: u8 = 1 << 0;
    /// Its flinch is a stagger: a guard broken, a charge parried.
    pub const STAGGERED: u8 = 1 << 1;
    /// The move it has out is thrown to its right -- the other side from
    /// the one its row is authored on: see [`super::Mind`]'s `mirrored`.
    pub const RIGHT: u8 = 1 << 2;
    /// This charge was chained off a miss: one paw, not two.
    pub const CHAINED: u8 = 1 << 3;
}

/// Bits of a cow's [`Critter::role`]: the low six count frames a fighter
/// has stood in its rear wedge, as its own glances saw it.
pub mod cow {
    /// It has taken its nerve's worth and is leaving by the ford.
    pub const BOLTED: u8 = 1 << 7;
    /// The buck it has out is its second, the harder one: see `ride`.
    pub const HARD: u8 = 1 << 6;
    /// The counter's bits.
    pub const WEDGE: u8 = 0x3F;
}

/// The herd's five states (§5), and gone.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HerdState {
    Grazing,
    /// Bunched up at home, behind the bull.
    Alarmed,
    /// Running the lane a bellow drew.
    Stampede,
    /// Wheeling at the lane's end and coming home.
    Returning,
    /// Below a quarter: milling round the bull.
    Rallied,
}

impl HerdState {
    pub const fn byte(self) -> u8 {
        match self {
            HerdState::Grazing => 0,
            HerdState::Alarmed => 1,
            HerdState::Stampede => 2,
            HerdState::Returning => 3,
            HerdState::Rallied => 4,
        }
    }

    pub const fn of(b: u8) -> HerdState {
        match b {
            1 => HerdState::Alarmed,
            2 => HerdState::Stampede,
            3 => HerdState::Returning,
            4 => HerdState::Rallied,
            _ => HerdState::Grazing,
        }
    }
}

pub struct Mind;

// ---------------------------------------------------------------------------
// The memo, as named things
// ---------------------------------------------------------------------------

fn byte(pack: &Pack, w: usize, k: u32) -> u8 {
    (pack.memo[w] as u32 >> (8 * k)) as u8
}

fn set_byte(pack: &mut Pack, w: usize, k: u32, v: u8) {
    let kept = pack.memo[w] as u32 & !(0xFF << (8 * k));
    pack.memo[w] = (kept | ((v as u32) << (8 * k))) as i32;
}

pub(super) fn half(pack: &Pack, w: usize, k: u32) -> i32 {
    (pack.memo[w] as u32 >> (16 * k) & 0xFFFF) as i32
}

pub(super) fn set_half(pack: &mut Pack, w: usize, k: u32, v: i32) {
    let shift = 16 * k;
    let kept = pack.memo[w] as u32 & !(0xFFFF << shift);
    pack.memo[w] = (kept | ((v.clamp(0, 0xFFFF) as u32) << shift)) as i32;
}

pub(super) fn point_of(pack: &Pack, w: usize) -> V3 {
    let v = pack.memo[w] as u32;
    V3::new(from_cm(lo(v)), Fx::ZERO, from_cm(hi(v)))
}

pub(super) fn set_point(pack: &mut Pack, w: usize, p: V3) {
    pack.memo[w] = halves(to_cm(p.x), to_cm(p.z)) as i32;
}

pub(super) fn has_bit(pack: &Pack, bit: u8) -> bool {
    byte(pack, word::HERD, 2) & bit != 0
}

pub(super) fn set_bit(pack: &mut Pack, bit: u8, on: bool) {
    let b = byte(pack, word::HERD, 2);
    set_byte(pack, word::HERD, 2, if on { b | bit } else { b & !bit });
}

/// The herd's state.
pub fn herd_state(pack: &Pack) -> HerdState {
    HerdState::of(byte(pack, word::HERD, 0))
}

pub(super) fn set_herd(pack: &mut Pack, s: HerdState) {
    set_byte(pack, word::HERD, 0, s.byte());
}

/// The bull's last move, if it has thrown one.
fn last_move(pack: &Pack) -> Option<u8> {
    byte(pack, word::HERD, 1).checked_sub(1)
}

pub(super) fn set_last_move(pack: &mut Pack, m: u8) {
    set_byte(pack, word::HERD, 1, m + 1);
}

/// Per-fighter bits in the top byte of [`word::HERD`].
fn fighter_bit(pack: &Pack, base: u32, who: usize) -> bool {
    byte(pack, word::HERD, 3) & (1 << (base + who as u32 % 2)) != 0
}

fn set_fighter_bit(pack: &mut Pack, base: u32, who: usize, on: bool) {
    let b = byte(pack, word::HERD, 3);
    let bit = 1u8 << (base + who as u32 % 2);
    set_byte(pack, word::HERD, 3, if on { b | bit } else { b & !bit });
}

/// Fighter `who` has been knocked down by the stampede under way.
pub(super) fn trampled(pack: &Pack, who: usize) -> bool {
    fighter_bit(pack, 0, who)
}

pub(super) fn set_trampled(pack: &mut Pack, who: usize, on: bool) {
    set_fighter_bit(pack, 0, who, on);
}

/// The last glance saw fighter `who` in front of the bull, near, in the
/// startup of an attack.
pub(super) fn seen_swing(pack: &Pack, who: usize) -> bool {
    fighter_bit(pack, 2, who)
}

pub(super) fn set_seen_swing(pack: &mut Pack, who: usize, on: bool) {
    set_fighter_bit(pack, 2, who, on);
}

/// The two horns' health: left, right.
pub fn horn_health(pack: &Pack) -> [i32; 2] {
    let v = pack.memo[word::HORNS] as u32;
    [lo(v) as i32, hi(v) as i32]
}

pub(super) fn set_horns(pack: &mut Pack, h: [i32; 2]) {
    pack.memo[word::HORNS] = halves(
        h[0].clamp(0, i16::MAX as i32) as i16,
        h[1].clamp(0, i16::MAX as i32) as i16,
    ) as i32;
}

/// Which horns are whole: left, right.
pub fn horns_whole(pack: &Pack) -> [bool; 2] {
    let h = horn_health(pack);
    [h[0] > 0, h[1] > 0]
}

/// Is the bull wary of a lane from `from` to `to`: wary at all, and the lane
/// passing within `WaryRadius` of the solid it last hit?
pub fn wary_of(pack: &Pack, from: V3, to: V3) -> bool {
    if half(pack, word::WARY, 0) == 0 {
        return false;
    }
    let at = point_of(pack, word::WARY_AT);
    let gap = crate::math::flat_segment_gap(at, from, to);
    gap.raw() <= knob_fx(Knob::WaryRadius).raw()
}

/// Is critter `c` the bull, stunned by a solid?
pub fn stunned(c: &Critter) -> bool {
    c.kind == BULL && c.state == is::FLINCH && c.role & bull::STUNNED != 0
}

/// The stampede's lane, if one is drawn: where it starts, which way it runs
/// (a unit vector), how long and how wide it is.
pub fn bellow_lane(pack: &Pack) -> Option<(V3, V3, Fx, Fx)> {
    let len = half(pack, word::LANE_DIR, 1);
    if len == 0 {
        return None;
    }
    let at = point_of(pack, word::LANE_AT);
    let yaw = half(pack, word::LANE_DIR, 0);
    let dir = V3::from_turns(Fx::from_raw(yaw));
    Some((at, dir, Fx::ratio(len, 100), knob_fx(Knob::LaneWidth)))
}

pub(super) fn set_lane(pack: &mut Pack, at: V3, dir: V3, len: Fx) {
    set_point(pack, word::LANE_AT, at);
    set_half(pack, word::LANE_DIR, 0, yaw_of(dir) as i32);
    let cm = to_cm(len.max(Fx::ZERO)) as i32;
    set_half(pack, word::LANE_DIR, 1, cm.max(1));
}

pub(super) fn clear_lane(pack: &mut Pack) {
    pack.memo[word::LANE_AT] = 0;
    pack.memo[word::LANE_DIR] = 0;
}

/// A point in a lane's own frame: how far along it, and how far across it
/// (positive to the left of the way it runs).
pub fn lane_frame(at: V3, dir: V3, p: V3) -> (Fx, Fx) {
    let d = V3::new(p.x.sub(at.x), Fx::ZERO, p.z.sub(at.z));
    let side = V3::new(dir.z.neg(), Fx::ZERO, dir.x);
    (d.dot(dir), d.dot(side))
}

/// Is `p` inside the stampede's lane, as drawn?
pub fn in_lane(pack: &Pack, p: V3) -> bool {
    let Some((at, dir, len, width)) = bellow_lane(pack) else {
        return false;
    };
    let (along, across) = lane_frame(at, dir, p);
    along.raw() >= 0
        && along.raw() <= len.raw()
        && across.abs().raw() <= width.mul(Fx::ratio(1, 2)).raw()
}

/// Is `p` inside a lee: the footprint of `solid` in a lane running `dir` and
/// `LeeLength` downstream of it? The solid is an axis-aligned box in the
/// floor plane, its footprint `min`..`max`; in the lane's frame its shadow
/// is the band across the lane its corners span.
pub fn in_lee(at: V3, dir: V3, min: V3, max: V3, p: V3, grow: Fx) -> bool {
    let (lo_a, hi_a, lo_c, hi_c) = shadow(at, dir, min, max);
    let (along, across) = lane_frame(at, dir, p);
    along.raw() >= lo_a.sub(grow).raw()
        && along.raw() <= hi_a.add(knob_fx(Knob::LeeLength)).raw()
        && across.raw() >= lo_c.sub(grow).raw()
        && across.raw() <= hi_c.add(grow).raw()
}

/// A box's footprint in a lane's frame: the least and most it reaches along
/// the lane and across it.
pub fn shadow(at: V3, dir: V3, min: V3, max: V3) -> (Fx, Fx, Fx, Fx) {
    let mut lo_a = Fx::MAX;
    let mut hi_a = Fx::MIN;
    let mut lo_c = Fx::MAX;
    let mut hi_c = Fx::MIN;
    for (x, z) in [
        (min.x, min.z),
        (max.x, min.z),
        (max.x, max.z),
        (min.x, max.z),
    ] {
        let (a, c) = lane_frame(at, dir, V3::new(x, Fx::ZERO, z));
        lo_a = lo_a.min(a);
        hi_a = hi_a.max(a);
        lo_c = lo_c.min(c);
        hi_c = hi_c.max(c);
    }
    (lo_a, hi_a, lo_c, hi_c)
}

/// The charge being wound up or run, as the rules worked it out: where its
/// lane ends, how long it is, and whether a solid ends it.
pub fn charge_lane(pack: &Pack) -> (V3, Fx, bool) {
    (
        point_of(pack, word::RUN_END),
        Fx::ratio(half(pack, word::RUN, 0), 100),
        has_bit(pack, bits::ENDS_IN_SOLID),
    )
}

/// The guard's arc, as drawn: how far round from the nose it covers either
/// side (a cosine), and which halves are open -- a broken horn opens its half.
pub fn guard_arc(pack: &Pack) -> (Fx, [bool; 2]) {
    let whole = horns_whole(pack);
    (knob_fx(Knob::GuardArc), whole)
}

// ---------------------------------------------------------------------------
// Bodies and where they are
// ---------------------------------------------------------------------------

/// The bull, if it is in the fight.
pub fn the_bull(critters: &Critters) -> Option<usize> {
    critters.iter().position(|c| c.kind == BULL && c.present())
}

/// The cows still with the herd: alive, not bolting.
pub fn with_herd(c: &Critter) -> bool {
    c.kind == COW && c.alive() && c.role & cow::BOLTED == 0
}

/// The middle of the herd: the cows still with it, or home if none are.
pub fn herd_centre(pack: &Pack, critters: &Critters) -> V3 {
    let mut sum = V3::ZERO;
    let mut n = 0;
    for c in critters.iter().filter(|c| with_herd(c)) {
        sum = sum.add(V3::new(c.pos.x, Fx::ZERO, c.pos.z));
        n += 1;
    }
    if n == 0 {
        return pack.home;
    }
    sum.scale(Fx::ONE.div(Fx::from_int(n)))
}

/// The bull's share of its health, nought to one.
pub fn bull_share(sp: &crate::species::Species, c: &Critter) -> Fx {
    let full = stat(sp, c.kind, CritterField::Health).max(1);
    Fx::ratio(c.health as i32, full).clamp(Fx::ZERO, Fx::ONE)
}

/// Is the bull desperate: below `DesperateBelow` -- half -- of its health?
pub fn desperate(sp: &crate::species::Species, c: &Critter) -> bool {
    bull_share(sp, c).raw() < Fx::ratio(1, 2).raw()
}

/// The fighter a critter is after, clamped to a real index.
fn target_of(c: &Critter) -> usize {
    (c.target as usize).min(MAX_PLAYERS - 1)
}

/// The horn on the side of a point, seen from the bull: 0 left, 1 right.
pub fn side_of(c: &Critter, p: V3) -> usize {
    let fwd = c.facing();
    let left = V3::new(fwd.z.neg(), Fx::ZERO, fwd.x);
    let d = V3::new(p.x.sub(c.pos.x), Fx::ZERO, p.z.sub(c.pos.z));
    if d.dot(left).raw() >= 0 { 0 } else { 1 }
}

/// How far round from the bull's nose a point is: the cosine.
pub fn off_nose(c: &Critter, p: V3) -> Fx {
    let d = V3::new(p.x.sub(c.pos.x), Fx::ZERO, p.z.sub(c.pos.z));
    if d.flat_len().raw() == 0 {
        return Fx::ONE;
    }
    c.facing().dot(d.normalized())
}

/// **The bull's head**, as a box: at the front of its body, its top at
/// `HeadHigh` standing and `HeadLow` when it is down -- stunned, charging,
/// hooking, braced. What a blow has to touch to go into the horns (§4).
pub fn head(sp: &crate::species::Species, c: &Critter) -> crate::critter::Body {
    let body = c.body(sp);
    let down = stunned(c)
        || (c.attacking() && matches!(c.act, CHARGE | HOOK | GUARD))
        || (c.state == is::RECOVERY && c.act == CHARGE);
    let top = if down {
        knob_fx(Knob::HeadLow)
    } else {
        knob_fx(Knob::HeadHigh)
    };
    let depth = knob_fx(Knob::HeadDepth);
    let half = Fx::ratio(1, 2);
    let tall = depth.max(Fx::ONE);
    let nose = body.to_world(V3::new(
        body.half_len.sub(depth.mul(half)),
        Fx::ZERO,
        Fx::ZERO,
    ));
    crate::critter::Body {
        foot: V3::new(nose.x, c.pos.y.add(top.sub(tall).max(Fx::ZERO)), nose.z),
        half_len: depth.mul(half),
        half_wid: knob_fx(Knob::HeadSpan).mul(half),
        height: tall.min(top),
        ..body
    }
}

// ---------------------------------------------------------------------------
// Deciding
// ---------------------------------------------------------------------------

/// A little noise on a score, the same on every machine: how far the bull's
/// choice strays from the best (decisiveness, §5).
fn noise(frame: u32, i: usize, m: u8) -> i32 {
    let mut x = frame
        .wrapping_mul(0x9E37_79B9)
        .wrapping_add((i as u32) << 8)
        .wrapping_add(m as u32);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x % 201) as i32 - 100
}

/// Is there a solid within `OpenWithin` of a point: is somebody standing
/// near cover?
fn near_cover(look: &Look, at: V3) -> bool {
    let reach = knob_fx(Knob::OpenWithin);
    look.arena.solids().any(|s| {
        s.max.y.raw() > Fx::ONE.raw()
            && !edge(look.arena, s)
            && super::rules::off_footprint(s, at).raw() <= reach.raw()
    })
}

/// Is a solid the arena's edge -- the slope and thicket a charge pulls up
/// short of rather than meets (§10, item 3)? One standing outside the
/// playable bounds. (The bank runs up to the north bound and is not one: a
/// charge into its face is a stun.)
pub fn edge(arena: &crate::arena::Terrain, s: &crate::arena::Solid) -> bool {
    let b = &arena.bounds;
    s.max.x.raw() <= b.lo_x.raw()
        || s.min.x.raw() >= b.hi_x.raw()
        || s.max.z.raw() <= b.lo_z.raw()
        || s.min.z.raw() >= b.hi_z.raw()
}

impl Mind {
    fn bull_appetite(&self, look: &Look, i: usize, m: CritterMove, a: &Attack) -> i32 {
        let pack = look.pack;
        let sp = look.sp;
        let c = &look.critters[i];
        let who = target_of(c);
        let seen = &pack.seen[who];
        if pack.mood != mood::HUNTING || !seen.alive {
            return 0;
        }
        let lead = look.lead(who);
        let to = V3::new(lead.x.sub(c.pos.x), Fx::ZERO, lead.z.sub(c.pos.z));
        let dist = to.flat_len();
        let whole = horns_whole(pack);
        let base = match m.kind {
            CHARGE => {
                if half(pack, word::WARY, 1) > 0 {
                    return 0;
                }
                // It will not charge somebody above it: the lane would end in
                // the face of whatever they stand on (§3).
                if seen.pos.y.raw() > Fx::ratio(1, 2).raw() {
                    return 0;
                }
                let run = if desperate(sp, c) {
                    knob_fx(Knob::DesperateRun)
                } else {
                    knob_fx(Knob::ChargeRun)
                };
                let dir = if dist.raw() > 0 {
                    to.normalized()
                } else {
                    c.facing()
                };
                let end = c.pos.add(dir.scale(run));
                if wary_of(pack, c.pos, end) {
                    return 0;
                }
                default_appetite(look, i, m, a)
            }
            TRAMPLE => {
                // `downed`: large for a fighter on the floor near it, and
                // exactly nothing otherwise.
                if !seen.staggered || dist.raw() > knob_fx(Knob::TrampleNear).raw() {
                    return 0;
                }
                knob(Knob::TrampleWant)
            }
            GUARD => {
                // `seen_swing`: a pose it saw, at its last glance. Both horns
                // gone, there is no guard at all.
                if !seen_swing(pack, who)
                    || half(pack, word::GUARD, 0) > 0
                    || !(whole[0] || whole[1])
                {
                    return 0;
                }
                knob(Knob::GuardWant)
            }
            BELLOW => {
                // `herd_home`: nothing unless the herd is home and not
                // running, and the lockout spent; more for somebody in the
                // open.
                let home = matches!(herd_state(pack), HerdState::Alarmed | HerdState::Rallied);
                if !home || half(pack, word::CLOCKS, 1) > 0 {
                    return 0;
                }
                let open = if near_cover(look, seen.pos) {
                    0
                } else {
                    knob(Knob::BellowAppetite)
                };
                default_appetite(look, i, m, a) + open
            }
            HOOK => {
                let combo = if half(pack, word::SINCE, 0) > 0 {
                    sp.combo_appetite()
                } else {
                    0
                };
                let plain = default_appetite(look, i, m, a);
                if plain <= 0 { 0 } else { plain + combo }
            }
            SHOULDER => {
                if dist.raw() > knob_fx(Knob::ShoulderFlank).raw() {
                    return 0;
                }
                default_appetite(look, i, m, a)
            }
            _ => 0,
        };
        if base <= 0 {
            return 0;
        }
        // `variety`: the repeat penalty, decaying.
        let repeat = if last_move(pack) == Some(m.kind) {
            let left = half(pack, word::RUN, 1);
            let frames = sp.variety_frames().max(1) as i32;
            sp.variety_penalty() * left / frames
        } else {
            0
        };
        // `hurt`: wounded animals commit harder.
        let wound = Fx::ONE.sub(bull_share(sp, c));
        let hurt = Fx::from_int(sp.hurt_aggression()).mul(wound).to_int();
        // Decisiveness: how far its choice strays from the best.
        let stray = (100 - sp.decisiveness()).clamp(0, 100);
        let jitter = base * stray / 100 * noise(look.frame, i, m.kind) / 100;
        (base + hurt - repeat + jitter).max(0)
    }

    fn cow_appetite(&self, look: &Look, i: usize, m: CritterMove) -> i32 {
        let c = &look.critters[i];
        match m.kind {
            // A fighter stood in its rear wedge for a whole glance of its own.
            KICK => {
                let stood = (c.role & cow::WEDGE) as i32;
                if look.pack.mood == mood::CALM
                    || c.role & cow::BOLTED != 0
                    || herd_state(look.pack) == HerdState::Stampede
                    || stood < knob(Knob::CowGlance)
                {
                    return 0;
                }
                look.sp.attack(KICK).weight.max(1)
            }
            // The buck and the stampede are the rules' to throw.
            BUCK | STAMPEDE => 0,
            _ => 0,
        }
    }

    fn bull_steer(&self, look: &Look, i: usize, want: Steer) -> Steer {
        let pack = look.pack;
        let sp = look.sp;
        let c = &look.critters[i];
        let hold = Steer {
            to: c.pos,
            speed: Fx::ZERO,
            face: None,
        };
        if !c.alive() {
            return hold;
        }
        if c.state != is::PROWL {
            // The windup turns to its target until it locks; everything else
            // stands where the move or the blow put it.
            return want;
        }
        let walk = knob_fx(Knob::BullWalk);
        let trot = crate::critter::stat_fx(sp, c.kind, CritterField::Run);
        let who = target_of(c);
        let seen = &pack.seen[who];
        if pack.mood == mood::CALM || !seen.alive {
            // **Watching**: it faces the nearest fighter inside its watch
            // radius, by its herd. The turn of the head is the tell.
            let near = pack
                .seen
                .iter()
                .filter(|s| s.alive)
                .map(|s| s.pos)
                .find(|p| {
                    V3::new(p.x.sub(c.pos.x), Fx::ZERO, p.z.sub(c.pos.z))
                        .flat_len()
                        .raw()
                        <= knob_fx(Knob::WatchRadius).raw()
                });
            let post = pack
                .home
                .add(V3::new(knob_fx(Knob::HerdBehind).neg(), Fx::ZERO, Fx::ZERO));
            return Steer {
                to: post,
                speed: walk.mul(Fx::ratio(1, 2)),
                face: near,
            };
        }
        let lead = look.lead(who);
        // A fighter on the floor within reach: stand so the trample lands on
        // them -- a hoof's reach ahead, facing.
        if seen.staggered {
            let gap = V3::new(lead.x.sub(c.pos.x), Fx::ZERO, lead.z.sub(c.pos.z));
            if gap.flat_len().raw() <= knob_fx(Knob::TrampleNear).raw() {
                let reach = sp.attack(TRAMPLE).hit_x;
                let from = if gap.flat_len().raw() > 0 {
                    gap.normalized()
                } else {
                    c.facing()
                };
                return Steer {
                    to: lead.sub(from.scale(reach)),
                    speed: walk,
                    face: Some(lead),
                };
            }
        }
        // **It interposes rather than approaching** (§5): the point on the
        // line from the herd's middle to its target, `Standoff` short of the
        // target -- the middle of the charge's range, so where it rests is a
        // threat. Closer than that, it stands its ground and faces you.
        let middle = match herd_state(pack) {
            HerdState::Stampede | HerdState::Returning => pack.home,
            _ => herd_centre(pack, look.critters),
        };
        let from = V3::new(middle.x.sub(lead.x), Fx::ZERO, middle.z.sub(lead.z));
        let from = if from.flat_len().raw() > 0 {
            from.normalized()
        } else {
            c.facing().scale(Fx::ONE.neg())
        };
        let post = lead.add(from.scale(knob_fx(Knob::Standoff)));
        let gap = V3::new(lead.x.sub(c.pos.x), Fx::ZERO, lead.z.sub(c.pos.z)).flat_len();
        if gap.raw() < knob_fx(Knob::Standoff).raw() {
            // Too near for the charge and too far for the hook: it backs off,
            // facing you, to where it rests. Inside the hook's reach it stands
            // its ground.
            let close = sp.attack(HOOK).ideal_range.add(sp.attack(HOOK).range_span);
            if gap.raw() > close.raw() {
                let away = V3::new(c.pos.x.sub(lead.x), Fx::ZERO, c.pos.z.sub(lead.z)).normalized();
                return Steer {
                    to: lead.add(away.scale(knob_fx(Knob::Standoff))),
                    speed: sp.back(),
                    face: Some(lead),
                };
            }
            return Steer {
                to: c.pos,
                speed: Fx::ZERO,
                face: Some(lead),
            };
        }
        // Past `TrotBeyond` it trots, and against somebody moving away adds
        // their speed to its own, up to its trot (`pursuit_gain`).
        let speed = if gap.raw() > knob_fx(Knob::TrotBeyond).raw() {
            let away = V3::new(seen.vel.x, Fx::ZERO, seen.vel.z)
                .dot(from.scale(Fx::ONE.neg()))
                .max(Fx::ZERO);
            walk.add(away.mul(sp.pursuit_gain()))
                .max(walk)
                .add(walk)
                .min(trot)
        } else {
            walk
        };
        Steer {
            to: post,
            speed,
            face: Some(lead),
        }
    }

    fn cow_steer(&self, look: &Look, i: usize, want: Steer) -> Steer {
        let pack = look.pack;
        let sp = look.sp;
        let c = &look.critters[i];
        let hold = Steer {
            to: c.pos,
            speed: Fx::ZERO,
            face: None,
        };
        if !c.alive() {
            return hold;
        }
        if c.state != is::PROWL {
            // A kick or a buck stands where it is; the stampede is steered by
            // the rules, which hold it to its lane.
            return match (c.state, c.act) {
                // Winding the stampede up: into the lane, and facing down it.
                (is::STARTUP, STAMPEDE) => match bellow_lane(pack) {
                    Some((at, dir, _, width)) => {
                        let slant = sp.attack(STAMPEDE).hit_x.abs().mul(Fx::ratio(1, 2));
                        let reach = sp
                            .attack(STAMPEDE)
                            .hit_radius
                            .add(crate::tuning::body_radius())
                            .add(slant.mul(Fx::ratio(1, 2)));
                        let room = width.mul(Fx::ratio(1, 2)).sub(reach).max(Fx::ZERO);
                        let (along, across) = lane_frame(at, dir, c.pos);
                        let side = V3::new(dir.z.neg(), Fx::ZERO, dir.x);
                        let to = at
                            .add(dir.scale(along))
                            .add(side.scale(across.clamp(room.neg(), room)));
                        Steer {
                            to,
                            speed: crate::critter::stat_fx(sp, c.kind, CritterField::Run),
                            face: Some(c.pos.add(dir.scale(knob_fx(Knob::GrazeWander)))),
                        }
                    }
                    None => hold,
                },
                _ => want,
            };
        }
        let walk = crate::critter::stat_fx(sp, c.kind, CritterField::Walk);
        let run = crate::critter::stat_fx(sp, c.kind, CritterField::Run);
        // Leaving: bolted, or the bull is dead -- out by the ford.
        if c.role & cow::BOLTED != 0 || pack.mood == mood::BROKEN {
            return Steer {
                to: pack.home,
                speed: run,
                face: None,
            };
        }
        let bull = the_bull(look.critters).map(|b| look.critters[b]);
        let centre = herd_centre(pack, look.critters);
        let state = herd_state(pack);
        // The goal (§5): grazing, a slow wander seeded from the snapshot;
        // bunched, home; coming back, home; rallied, a ring round the bull.
        let k = i as i32;
        let (goal, top, cohesion) = match state {
            HerdState::Grazing => {
                let turn = Fx::from_raw(
                    ((look.frame / 8) as i32)
                        .wrapping_mul(29)
                        .wrapping_add(k.wrapping_mul(7919))
                        & 0xFFFF,
                );
                let spot = pack
                    .home
                    .add(V3::from_turns(turn).scale(knob_fx(Knob::GrazeWander)));
                (spot, walk, knob_fx(Knob::Cohesion))
            }
            HerdState::Alarmed | HerdState::Stampede => {
                let turn = Fx::from_raw(k.wrapping_mul(0x1C71) & 0xFFFF);
                let spot = pack
                    .home
                    .add(V3::from_turns(turn).scale(Fx::ONE.add(Fx::ONE)));
                (spot, run, knob_fx(Knob::CohesionRun))
            }
            // Coming home at a trot: a ride, and a route.
            HerdState::Returning => (
                pack.home,
                knob_fx(Knob::ReturnSpeed),
                knob_fx(Knob::CohesionRun),
            ),
            HerdState::Rallied => {
                let at = bull.map_or(pack.home, |b| b.pos);
                let turn = Fx::from_raw(
                    (k.wrapping_mul(0x1C71)
                        .wrapping_add((look.frame as i32).wrapping_mul(40)))
                        & 0xFFFF,
                );
                let spot = at.add(V3::from_turns(turn).scale(knob_fx(Knob::RallyRadius)));
                (spot, run, knob_fx(Knob::CohesionRun))
            }
        };
        let flat = |v: V3| V3::new(v.x, Fx::ZERO, v.z);
        let toward = flat(goal.sub(c.pos));
        let gap = toward.flat_len();
        // Arriving, it slows to a walk, and to nothing.
        let speed = if gap.raw() > knob_fx(Knob::GrazeWander).raw() {
            top
        } else {
            walk.mul(
                gap.div(knob_fx(Knob::GrazeWander).max(Fx::ONE))
                    .min(Fx::ONE),
            )
        };
        let mut v = if gap.raw() > 0 {
            toward.normalized().scale(speed)
        } else {
            V3::ZERO
        };
        // Cohesion: toward the herd's middle.
        let to_middle = flat(centre.sub(c.pos));
        if to_middle.flat_len().raw() > knob_fx(Knob::GrazeWander).raw() {
            v = v.add(to_middle.normalized().scale(cohesion));
        }
        // Avoidance: a cow steps out of a fighter's way, and out of the bull's
        // lane while it charges -- cows part for the bull (§3).
        let keep = knob_fx(Knob::AvoidFighters);
        for s in pack.seen.iter().filter(|s| s.alive) {
            let away = flat(c.pos.sub(s.pos));
            let d = away.flat_len();
            if d.raw() > 0 && d.raw() < keep.raw() {
                v = v.add(
                    away.normalized()
                        .scale(walk.mul(keep.sub(d)).div(keep.max(Fx::ONE))),
                );
            }
        }
        if let Some(b) = bull.filter(|b| b.alive() && b.act == CHARGE && b.attacking()) {
            let (end, _, _) = charge_lane(pack);
            let (along, across) = lane_frame(b.pos, b.facing(), c.pos);
            let reach = b.pos.sub(end).flat_len();
            let wide = crate::critter::stat_fx(sp, BULL, CritterField::Width)
                .add(crate::critter::stat_fx(sp, COW, CritterField::Length));
            if along.raw() > 0 && along.raw() < reach.raw() && across.abs().raw() < wide.raw() {
                let side = b.facing();
                let side = V3::new(side.z.neg(), Fx::ZERO, side.x);
                let sign = if across.raw() >= 0 {
                    Fx::ONE
                } else {
                    Fx::ONE.neg()
                };
                v = v.add(side.scale(run.mul(sign)));
            }
        }
        let len = v.flat_len();
        let len_cap = top.max(walk);
        let (dir, speed) = if len.raw() == 0 {
            (V3::ZERO, Fx::ZERO)
        } else {
            (v.normalized(), len.min(len_cap))
        };
        Steer {
            to: c
                .pos
                .add(dir.scale(knob_fx(Knob::GrazeWander).add(Fx::ONE))),
            speed,
            face: None,
        }
    }
}

impl PackMind for Mind {
    fn appetite(&self, look: &Look, i: usize, m: CritterMove, a: &Attack) -> i32 {
        match look.critters[i].kind {
            BULL => self.bull_appetite(look, i, m, a),
            _ => self.cow_appetite(look, i, m),
        }
    }

    fn steer(&self, look: &Look, i: usize, want: Steer) -> Steer {
        match look.critters[i].kind {
            BULL => self.bull_steer(look, i, want),
            _ => self.cow_steer(look, i, want),
        }
    }

    fn frame(&self, pack: &mut Pack, critters: &mut Critters, herd: &Herd, frame: u32) {
        let _ = (herd, frame);
        let sp = pack.sp();
        // The clocks.
        for (w, k) in [
            (word::CLOCKS, 0),
            (word::CLOCKS, 1),
            (word::WARY, 0),
            (word::WARY, 1),
            (word::GUARD, 0),
            (word::RUN, 1),
            (word::SINCE, 0),
        ] {
            let left = half(pack, w, k);
            set_half(pack, w, k, left - 1);
        }
        // The strain bleeds, a share a frame, as the Ridgeback's does.
        let strain = pack.memo[word::STRAIN];
        let bled = strain * sp.strain_decay() / 100;
        pack.memo[word::STRAIN] = (strain - bled.max(1)).max(0);

        // The herd's states (§5). The stampede's own run and its return are
        // the rules', which hold the lane; this is the rest.
        let state = herd_state(pack);
        let bull = the_bull(critters).map(|b| critters[b]);
        let low = bull.is_some_and(|b| {
            b.alive() && bull_share(sp, &b).raw() < Fx::ratio(knob(Knob::RallyBelow), 100).raw()
        });
        // **Alarmed**, the first `Grace` frames are spent interposing, with
        // no moves thrown (§5) -- counted from the alarm, not from the start
        // of the hunt, since a herd you walk up to slowly is calm until then.
        if pack.mood != mood::CALM && state == HerdState::Grazing {
            pack.grace = sp.pack_raw(crate::pack::PackKnob::Grace).max(0) as u16;
        }
        let next = match (pack.mood, state) {
            (mood::CALM, _) => HerdState::Grazing,
            (_, HerdState::Grazing) => HerdState::Alarmed,
            (_, HerdState::Alarmed) if low => HerdState::Rallied,
            (_, s) => s,
        };
        set_herd(pack, next);
    }

    fn frames_until_free(&self, pack: &Pack, critters: &Critters) -> Option<u16> {
        // What could land next, and when: a windup's frames left, a hit out
        // now; a cow's kick winding up, a stampede running. A bull that can
        // act could throw the quickest move its target's range allows -- the
        // hook inside its reach, the charge and its run beyond it; one that
        // cannot (a skid, a stun) only once it can again.
        let sp = pack.sp();
        let mut soonest = u32::MAX;
        for c in critters.iter().filter(|c| c.alive()) {
            let a = sp.attack(c.act);
            let left = match c.state {
                is::ACTIVE if a.damage > 0 && !c.has(flag::HIT_USED) => 0,
                is::STARTUP if a.damage > 0 => c.timer as u32,
                _ => u32::MAX,
            };
            soonest = soonest.min(left);
            if c.kind != BULL || pack.mood != mood::HUNTING {
                continue;
            }
            let seen = &pack.seen[target_of(c)];
            if !seen.alive {
                continue;
            }
            let gap =
                V3::new(seen.pos.x.sub(c.pos.x), Fx::ZERO, seen.pos.z.sub(c.pos.z)).flat_len();
            let hook = sp.attack(HOOK);
            let quickest = if gap.raw() <= hook.ideal_range.add(hook.range_span).raw() {
                hook.startup as u32
            } else {
                let charge = sp.attack(CHARGE);
                let run = gap
                    .div(charge.advance.max(Fx::ONE))
                    .mul(Fx::from_int(crate::TICK_HZ as i32));
                charge.startup as u32 + run.to_int().max(0) as u32
            };
            let until = match c.state {
                is::PROWL => 0,
                is::RECOVERY | is::FLINCH => c.timer as u32,
                _ => u32::MAX,
            };
            soonest = soonest.min(until.saturating_add(quickest));
        }
        if pack.grace > 0 || pack.mood != mood::HUNTING {
            return Some(u16::MAX);
        }
        Some(soonest.min(u16::MAX as u32) as u16)
    }

    fn spares(&self, pack: &Pack, critters: &Critters, i: usize, who: usize) -> bool {
        // Cows do not tread on the fallen (§2): one stampede, one knockdown.
        critters[i].act == STAMPEDE && trampled(pack, who)
    }

    fn surface(&self, c: &Critter) -> (Fx, Fx) {
        super::ride::surface(c)
    }

    fn mirrored(&self, c: &Critter) -> bool {
        c.kind == BULL && c.role & bull::RIGHT != 0
    }

    fn guarded(&self, pack: &mut Pack, critters: &mut Critters, i: usize, blow: &Blow) -> Guarded {
        let sp = pack.sp();
        let c = critters[i];
        if c.kind != BULL || !c.alive() {
            return Guarded::Lands;
        }
        let side = side_of(&c, blow.from);
        let whole = horns_whole(pack);
        let frontal = off_nose(&c, blow.from).raw() >= knob_fx(Knob::GuardArc).raw();
        let braced = c.state == is::ACTIVE && c.act == GUARD;
        if braced && frontal && whole[side] {
            if blow.breaks {
                // Broken: a stagger, and no guard for a while.
                let body = &mut critters[i];
                body.state = is::FLINCH;
                body.timer = knob(Knob::GuardBroken).max(1) as u16;
                body.role = (body.role & !bull::STUNNED) | bull::STAGGERED;
                set_half(pack, word::GUARD, 0, knob(Knob::GuardLockout));
            } else {
                // Turned: it slides back half a metre, a blocker's pushback,
                // drops the guard, and wants a hook.
                let body = &mut critters[i];
                let back = V3::new(
                    body.pos.x.sub(blow.from.x),
                    Fx::ZERO,
                    body.pos.z.sub(blow.from.z),
                )
                .normalized();
                body.pos = body.pos.add(back.scale(knob_fx(Knob::GuardSlide)));
                body.state = is::PROWL;
                body.timer = 0;
                set_half(pack, word::SINCE, 0, knob(Knob::ComboFrames));
                return Guarded::Bounces {
                    recoil: knob(Knob::GuardRecoil).max(1) as u16,
                    push: knob_fx(Knob::GuardPush),
                };
            }
        }
        // **The horns**: a blow that reaches the head goes into the horn on
        // the side it came from as well as into the bull (§4).
        if head(sp, &c).touched_by(blow.hitbox) {
            let mut h = horn_health(pack);
            if h[side] > 0 {
                h[side] = (h[side] - blow.damage).max(0);
                set_horns(pack, h);
            }
        }
        if braced && frontal && whole[side] {
            Guarded::Breaks
        } else {
            Guarded::Lands
        }
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
        let sp = pack.sp();
        let c = critters[i];
        match c.act {
            CHARGE => {
                if parried {
                    // A parry of the charge on a held shield: half a stun.
                    let body = &mut critters[i];
                    body.state = is::FLINCH;
                    body.timer = knob(Knob::ParryStagger).max(1) as u16;
                    body.role = (body.role & !bull::STUNNED) | bull::STAGGERED;
                } else if blocked {
                    // A block does not stop it: pushed back a long way, and a
                    // long stunlock. The lesson is to plant (§7).
                    let push = knob_fx(Knob::BlockPush);
                    let frames = knob(Knob::BlockStun).max(1);
                    let dir = c.facing();
                    let rate = Fx::from_int(crate::TICK_HZ as i32).div(Fx::from_int(frames));
                    victim.vel.x = dir.x.mul(push).mul(rate);
                    victim.vel.z = dir.z.mul(push).mul(rate);
                    victim.action = crate::state::Action::BlockStun {
                        left: frames as u16,
                    };
                    victim.stun_total = frames as u16;
                } else {
                    knock_down(victim, knob(Knob::ChargeKnockdown));
                }
            }
            HOOK => {
                if blocked || parried {
                    return;
                }
                // A broken horn's hook is a shove: no launch, no knockdown,
                // so no trample follows it (§4).
                let side = if c.role & bull::RIGHT != 0 { 1 } else { 0 };
                if horns_whole(pack)[side] {
                    // Through the flight, then the knockdown: rising and
                    // falling back take the same time.
                    let up = victim.vel.y.max(Fx::ZERO);
                    let g = crate::tuning::gravity().abs().max(Fx::ONE);
                    let rise = up.div(g).mul(Fx::from_int(crate::TICK_HZ as i32));
                    let air = rise.add(rise);
                    knock_down(victim, knob(Knob::HookKnockdown) + air.to_int());
                } else {
                    victim.vel.y = Fx::ZERO;
                }
            }
            STAMPEDE => {
                if blocked || parried {
                    return;
                }
                knock_down(victim, knob(Knob::Knockdown));
                set_trampled(pack, who, true);
            }
            _ => {}
        }
        let _ = sp;
    }

    fn hurt(&self, pack: &mut Pack, critters: &mut Critters, i: usize, dealt: i32) {
        let sp = pack.sp();
        let c = &mut critters[i];
        if c.kind == COW {
            // The nerve: a cow that has taken its share bolts for the ford.
            let full = stat(sp, COW, CritterField::Health);
            if full - c.health as i32 >= knob(Knob::CowNerve) && c.alive() {
                c.role |= cow::BOLTED;
                if c.attacking() {
                    c.state = is::PROWL;
                    c.timer = 0;
                }
            }
            return;
        }
        // The bull's strain, at critter scale (§4): recent damage buys an
        // interrupt, the thresholds falling as it weakens.
        pack.memo[word::STRAIN] = pack.memo[word::STRAIN].saturating_add(dealt);
        let fall = Fx::ratio(sp.strain_desperation(), 100).mul(Fx::ONE.sub(bull_share(sp, c)));
        let interrupt = Fx::from_int(sp.interrupt_strain())
            .mul(Fx::ONE.sub(fall))
            .to_int();
        // **The shoulder is the one move answered by hitting it**: any hit in
        // its lean flinches it, whatever the strain.
        let leaning = c.state == is::STARTUP && c.act == SHOULDER;
        let committed = c.state == is::ACTIVE && c.act == CHARGE;
        let interruptible =
            matches!(c.state, is::STARTUP | is::ACTIVE | is::PROWL | is::RECOVERY) && !committed;
        if leaning || (interruptible && pack.memo[word::STRAIN] >= interrupt) {
            c.state = is::FLINCH;
            c.timer = if leaning {
                knob(Knob::ShoulderFlinch)
            } else {
                stat(sp, BULL, CritterField::FlinchFrames)
            }
            .max(1) as u16;
            c.role &= !(bull::STUNNED | bull::STAGGERED);
            c.set(flag::HIT_USED, false);
            if !leaning {
                pack.memo[word::STRAIN] = 0;
            }
        }
    }

    fn telegraph(
        &self,
        pack: &Pack,
        critters: &Critters,
        i: usize,
        plain: Telegraph,
    ) -> Option<Telegraph> {
        let c = &critters[i];
        match c.act {
            // The charge: the whole run, from the nose, cut short where a
            // solid will stop it -- the rules' answer, which the swept stop
            // reads too.
            CHARGE => {
                let (end, run, _) = charge_lane(pack);
                if run.raw() <= 0 {
                    return Some(plain);
                }
                let from = plain.anchor;
                let along = V3::new(end.x.sub(from.x), Fx::ZERO, end.z.sub(from.z));
                let sweep = along.flat_len();
                Some(Telegraph {
                    sweep,
                    along: if sweep.raw() > 0 {
                        along.normalized()
                    } else {
                        plain.along
                    },
                    ..plain
                })
            }
            // The bellow and the stampede are drawn as the lane, with its
            // lees: the species' signs, not a body's volume.
            BELLOW | STAMPEDE | GUARD => None,
            _ => Some(plain),
        }
    }
}

/// On the floor for `frames`, wherever the blow left them.
pub(super) fn knock_down(p: &mut Player, frames: i32) {
    let frames = frames.max(1) as u16;
    p.action = crate::state::Action::Stagger { left: frames };
    p.stun_total = frames;
}
