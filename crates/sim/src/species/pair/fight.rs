//! What the two cats do together: the pair brain, the leaps, the scar, the
//! bond and the death -- everything the shared machinery calls the Pair by.
//! See `docs/design/creatures/the-pair.md`.
//!
//! **The pair brain is this file's frame hook** (`FightDecl::frame`), run
//! once a frame after both cats have stepped: it owns the roles, the swap
//! clock, the pincer, the bond, the first death and the enrage, and it flies
//! the leaps. It reads the two cats' own samples and never a button
//! (`the-pair.md` §5). Nothing in it sees anything a cat did not: each cat's
//! sight is its own perception filter ([`perceives`]), asked through
//! `aim::sight_clear`.
//!
//! **What each cat's brain needs of its partner** -- where it is, what role
//! it plays, when its next hit lands -- is written into the hunt's lore here
//! ([`word`]) and read back by [`super::mind`], so the shared brain's window
//! is still `Quarry` and `Mind`.
//!
//! Each cat keeps its own state on its body (`Monster::own`, [`body`]): its
//! role and enrage and scar as flags, the damage its head has taken, where a
//! leap left from, how long it has gone without seeing you.

use crate::DT;
use crate::aim::Scene;
use crate::fixed::Fx;
use crate::lore::{self, Layout, Lore};
use crate::math::{self, V3};
use crate::monster::{Doing, MAX_MONSTERS, Monster, Quarry};
use crate::perception::{self, Perceiver};
use crate::species::{FightDecl, Mark, MarkLook, Marks};
use crate::state::World;

use super::{
    AMBUSH, COCK, DIVE, DROP, FEINT, HEAD, HOWL, INTERPOSE, Knob, PERCH, POUNCE, RAKE, RAKE2,
    SPECIES, TWIN, leaps,
};

pub static FIGHT: FightDecl = FightDecl {
    layout: Layout {
        hazards: 0,
        noises: 0,
        objectives: 0,
        own: 6,
    },
    row: true,
    perceives,
    collides: true,
    keeps_height: true,
    bodies: 2,
    frame: Some(frame),
    appetite: Some(super::mind::appetite),
    prowl_to: Some(super::mind::prowl_to),
    commit: Some(super::mind::commit),
    struck: Some(struck),
    marks: Some(marks),
    pace: Some(pace),
    glance: Some(glance),
    presence: Some(presence),
    ..FightDecl::PLAIN
};

// ---------------------------------------------------------------------------
// The state
// ---------------------------------------------------------------------------

/// The words each cat keeps on its body (`Monster::own`).
pub mod body {
    /// [`super::flag`]s.
    pub const FLAGS: usize = 0;
    /// Damage its head has taken, all fight: past `ScarDamage`, an eye goes.
    pub const HEAD_DAMAGE: usize = 1;
    /// Where its leap left the ground, in centimetres (`lore::halves`).
    pub const LEAP_FROM: usize = 2;
    /// Low half: frames since it last saw its target. High half: frames it
    /// has been perched, or, while it is in the air, the height its leap
    /// left from in centimetres.
    pub const CLOCKS: usize = 3;
}

/// The bits of [`body::FLAGS`].
pub mod flag {
    /// Its role: none, Holder, or Striker.
    pub const ROLE: i32 = 0b11;
    pub const HOLDER: i32 = 1;
    pub const STRIKER: i32 = 2;
    /// The survivor, enraged.
    pub const ENRAGED: i32 = 1 << 2;
    /// Up on a top: rooted, and its only move is the dive.
    pub const PERCHED: i32 = 1 << 3;
    /// The eye scarred on its left, or its right.
    pub const SCAR_LEFT: i32 = 1 << 4;
    pub const SCAR_RIGHT: i32 = 1 << 5;
    /// The head has taken enough; which eye is read next frame.
    pub const SCAR_PENDING: i32 = 1 << 6;
    /// An enraged pounce that has already chained once.
    pub const CHAINED: i32 = 1 << 7;
    /// Which creature slot it is, two bits from here.
    pub const SLOT_SHIFT: i32 = 12;
    /// Two hunters: it glances quicker.
    pub const COOP: i32 = 1 << 14;
}

/// The pair brain's words in the hunt's lore (`Lore::word`): the shared
/// state, then a block per cat of what its partner needs to know.
pub mod word {
    /// [`super::pair`] bits.
    pub const FLAGS: usize = 0;
    /// Frames until the roles swap.
    pub const SWAP: usize = 1;
    /// Frames of the swap's crossing left: neither attacks.
    pub const CROSS: usize = 2;
    /// Which slot holds.
    pub const HOLDER: usize = 3;
    /// Frames until a twin pounce may be thrown again.
    pub const TWIN_WAIT: usize = 4;
    /// The frame of the first death, plus one.
    pub const DEATH_AT: usize = 5;
    /// The frame this was written on.
    pub const NOW: usize = 6;
    /// Each cat's block starts here, [`EACH`] words long.
    pub const CATS: usize = 7;
    pub const EACH: usize = 7;
    /// Where it is, in centimetres.
    pub const POS: usize = 0;
    /// The frames its next hit is live across, inclusive: zero is none.
    pub const LIVE_FROM: usize = 1;
    pub const LIVE_TO: usize = 2;
    /// [`super::state`] bits.
    pub const STATE: usize = 3;
    /// The top it would perch on, in centimetres, and how high it is.
    pub const PERCH_AT: usize = 4;
    pub const PERCH_TOP: usize = 5;
    /// Frames it has been walking somewhere and getting nowhere, then (in
    /// the top half) frames left of going round whatever is in the way.
    pub const STUCK: usize = 6;

    /// A word of a cat's block.
    pub const fn of(slot: usize, w: usize) -> usize {
        CATS + slot * EACH + w
    }
}

/// The shared bits of [`word::FLAGS`].
pub mod pair {
    /// The fight has been set up: roles drawn, coop numbers applied.
    pub const READY: u32 = 1;
    /// Two hunters.
    pub const COOP: u32 = 2;
    /// One of them is dead.
    pub const DEATH: u32 = 4;
}

/// The bits of a cat's [`word::STATE`].
pub mod state {
    /// It saw its target this frame.
    pub const SEES: u32 = 1;
    /// A feint is allowed: it holds, and its Striker is behind the target
    /// and free.
    pub const MAY_FEINT: u32 = 2;
    /// The swap is crossing: nothing is thrown.
    pub const CROSSING: u32 = 4;
    /// Too far from its partner: it runs to rejoin.
    pub const REJOIN: u32 = 8;
    /// It fights alone: the nearer of a split pair, or the only one left.
    pub const SOLO: u32 = 16;
    /// Bonded, and the wounded one: it hangs back.
    pub const WOUNDED: u32 = 32;
    /// Bonded, and the healthy one: it guards.
    pub const GUARD: u32 = 64;
    /// Its partner is alive.
    pub const PARTNER: u32 = 128;
    /// Its partner is free to act.
    pub const PARTNER_FREE: u32 = 256;
}

pub fn flags(m: &Monster) -> i32 {
    m.own[body::FLAGS]
}

fn set_flag(m: &mut Monster, bit: i32, on: bool) {
    if on {
        m.own[body::FLAGS] |= bit;
    } else {
        m.own[body::FLAGS] &= !bit;
    }
}

/// Which creature slot this cat is, as the pair brain last wrote it.
pub fn slot_of(m: &Monster) -> usize {
    ((m.own[body::FLAGS] >> flag::SLOT_SHIFT) & 0b11) as usize % MAX_MONSTERS
}

pub fn role(m: &Monster) -> i32 {
    m.own[body::FLAGS] & flag::ROLE
}

pub fn enraged(m: &Monster) -> bool {
    m.own[body::FLAGS] & flag::ENRAGED != 0
}

pub fn perched(m: &Monster) -> bool {
    m.own[body::FLAGS] & flag::PERCHED != 0
}

/// Which side its scarred eye is on: -1 its left, +1 its right, 0 neither.
pub fn scar(m: &Monster) -> i32 {
    let f = m.own[body::FLAGS];
    if f & flag::SCAR_LEFT != 0 {
        -1
    } else if f & flag::SCAR_RIGHT != 0 {
        1
    } else {
        0
    }
}

/// Scar an eye, for tests and tools that want a scarred cat now: -1 its
/// left, +1 its right.
pub fn scar_eye(m: &mut Monster, side: i32) {
    set_flag(m, flag::SCAR_LEFT, side < 0);
    set_flag(m, flag::SCAR_RIGHT, side > 0);
}

/// Frames since it last saw its target.
pub fn unseen(m: &Monster) -> i32 {
    lore::lo(m.own[body::CLOCKS] as u32) as u16 as i32
}

fn set_unseen(m: &mut Monster, frames: i32) {
    let hi = lore::hi(m.own[body::CLOCKS] as u32);
    m.own[body::CLOCKS] = lore::halves(frames.clamp(0, i16::MAX as i32) as i16, hi) as i32;
}

/// Frames it has been perched, or the height a leap left from (cm).
fn upper(m: &Monster) -> i32 {
    lore::hi(m.own[body::CLOCKS] as u32) as i32
}

fn set_upper(m: &mut Monster, v: i32) {
    let lo = lore::lo(m.own[body::CLOCKS] as u32);
    m.own[body::CLOCKS] =
        lore::halves(lo, v.clamp(i16::MIN as i32, i16::MAX as i32) as i16) as i32;
}

/// A cat's block of state bits, as the pair brain wrote it.
pub fn state_of(lore: &Lore, slot: usize) -> u32 {
    lore.word(word::of(slot, word::STATE))
}

/// Where a cat was, as the pair brain wrote it.
pub fn pos_of(lore: &Lore, slot: usize) -> V3 {
    point(lore.word(word::of(slot, word::POS)))
}

/// The top a cat would perch on, if there is one in reach: where, and how
/// high.
pub fn perch_of(lore: &Lore, slot: usize) -> Option<(V3, Fx)> {
    let top = lore.int(word::of(slot, word::PERCH_TOP));
    if top <= 0 {
        return None;
    }
    Some((
        point(lore.word(word::of(slot, word::PERCH_AT))),
        lore::from_cm(top as i16),
    ))
}

/// The frame the brain is thinking on: one after the last the pair brain
/// wrote, because the brain steps before the frame hook does.
pub fn now(lore: &Lore) -> u32 {
    lore.word(word::NOW).wrapping_add(1)
}

/// A cat's next live window, as the pair brain last wrote it: `None` for none.
pub fn live_of(lore: &Lore, slot: usize) -> Option<(u32, u32)> {
    let from = lore.word(word::of(slot, word::LIVE_FROM));
    let to = lore.word(word::of(slot, word::LIVE_TO));
    (from != 0).then_some((from, to))
}

fn point(w: u32) -> V3 {
    V3::new(
        lore::from_cm(lore::lo(w)),
        Fx::ZERO,
        lore::from_cm(lore::hi(w)),
    )
}

fn word_of(p: V3) -> u32 {
    lore::halves(lore::to_cm(p.x), lore::to_cm(p.z))
}

/// The leap this cat is in, if it is in one.
fn in_leap(m: &Monster) -> Option<u8> {
    m.doing.attacking().filter(|k| leaps(*k))
}

/// Frames since its move began: through the startup, then the active window.
fn elapsed(m: &Monster) -> Option<(u8, i32)> {
    let kind = m.doing.attacking()?;
    let a = SPECIES.attack(kind);
    Some(match m.doing {
        Doing::Startup { left, .. } => (kind, a.startup as i32 - left as i32),
        Doing::Active { left, .. } => (kind, a.startup as i32 + a.active as i32 - left as i32),
        _ => (kind, a.total() as i32 + 1),
    })
}

/// Where a leap leaves the ground, in frames since its move began, and how
/// many frames it is in the air.
pub fn flight(kind: u8) -> (i32, i32) {
    let a = SPECIES.attack(kind);
    let leave = match kind {
        POUNCE => Knob::PounceLeave.raw(),
        TWIN => Knob::TwinLeave.raw(),
        DIVE => Knob::DiveLeave.raw(),
        _ => a.startup as i32,
    }
    .min(a.startup as i32);
    let land = match kind {
        POUNCE => Knob::PounceLand.raw(),
        TWIN => Knob::TwinLand.raw(),
        DIVE => Knob::DiveLand.raw(),
        PERCH => Knob::PerchLand.raw(),
        _ => Knob::DropLand.raw(),
    }
    .min(a.active as i32);
    (leave, (a.startup as i32 - leave + land).max(1))
}

/// Is this cat in the air: past the frame its leap left the ground, and not
/// yet down?
pub fn airborne(m: &Monster) -> bool {
    let (Some(kind), Some((_, e))) = (in_leap(m), elapsed(m)) else {
        return false;
    };
    let (leave, air) = flight(kind);
    e > leave && e <= leave + air
}

/// **A cat in the air is passed under**: there to hit, nothing to walk
/// into, from the frame its leap leaves the ground to the frame it lands. A
/// dodge toward a pounce goes under the arc (`the-pair.md` §2), and a body
/// whose legs were solid at head height would turn it aside.
fn presence(m: &Monster, _rig: &crate::beast::Rig) -> crate::beast::Presence {
    crate::beast::Presence {
        passable: if airborne(m) { u64::MAX } else { 0 },
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// Senses
// ---------------------------------------------------------------------------

/// **What a cat perceives** (P5): a fighter it has a clear line to from its
/// head to any of its body above the waist (`aim::sight_clear`: a wall, a
/// stone, smoke), and not inside the blind
/// arc of a scarred eye. The two do not share: a Striker that cannot see you
/// cannot strike you, whatever its partner sees.
pub fn perceives(p: &Perceiver) -> bool {
    !perception::in_blind_arc(p, scar(p.monster)) && perception::in_sight_over_cover(p)
}

/// Two hunters glance quicker (`CoopGlance`).
pub fn glance(m: &Monster) -> u16 {
    if m.own[body::FLAGS] & flag::COOP != 0 {
        Knob::CoopGlance.raw().max(1) as u16
    } else {
        SPECIES.glance_frames()
    }
}

/// The survivor runs faster than its gallop.
pub fn pace(m: &Monster) -> Fx {
    if enraged(m) {
        Knob::EnragePace.fx()
    } else {
        Fx::ONE
    }
}

// ---------------------------------------------------------------------------
// A hit on a cat
// ---------------------------------------------------------------------------

/// **A hit has landed on a cat.** Its head counts toward the scar; a cat
/// running onto the interpose line is stopped by any hit on the run,
/// whatever its strain (`the-pair.md` §2).
pub fn struck(m: &mut Monster, part: usize, dealt: i32) -> bool {
    if part == HEAD && dealt > 0 && scar(m) == 0 {
        let was = m.own[body::HEAD_DAMAGE];
        m.own[body::HEAD_DAMAGE] = was.saturating_add(dealt);
        if was < Knob::ScarDamage.raw() && m.own[body::HEAD_DAMAGE] >= Knob::ScarDamage.raw() {
            set_flag(m, flag::SCAR_PENDING, true);
        }
    }
    if matches!(m.doing, Doing::Active { kind: INTERPOSE, .. }) && dealt > 0 {
        m.doing = Doing::Flinch {
            left: SPECIES.flinch_frames(),
        };
        m.speed = Fx::ZERO;
        return true;
    }
    false
}

// ---------------------------------------------------------------------------
// The pair brain
// ---------------------------------------------------------------------------

/// The pair brain, once a frame. See the module docs.
pub fn frame(w: &mut World) {
    let mut herd = w.monsters;
    let mut lore = w.lore;
    let now = w.frame;
    let ground = w.terrain();
    let field = crate::stones::gather(&w.players);
    let cats: [bool; MAX_MONSTERS] =
        std::array::from_fn(|s| herd[s].is_some_and(|m| m.species == SPECIES.id));

    // Who is who: each body learns its own slot.
    for (slot, m) in herd.iter_mut().enumerate() {
        if let Some(m) = m.as_mut().filter(|_| cats[slot]) {
            m.own[body::FLAGS] = (m.own[body::FLAGS] & !(0b11 << flag::SLOT_SHIFT))
                | ((slot as i32) << flag::SLOT_SHIFT);
        }
    }

    let mut shared = lore.word(word::FLAGS);
    if shared & pair::READY == 0 {
        shared |= pair::READY;
        let hunters = w.players.iter().filter(|p| p.health > 0).count();
        if hunters >= 2 {
            shared |= pair::COOP;
            for m in herd.iter_mut().flatten() {
                m.health = Knob::CoopHealth.raw().max(1);
                m.own[body::FLAGS] |= flag::COOP;
            }
        }
        lore.set_word(word::SWAP, swap_frames(&herd, &lore, shared, now));
        lore.set_word(word::HOLDER, nearer_to_target(&herd) as u32);
    }

    // **The first death.** The survivor howls over the body, and then it is
    // a different fight.
    let living: Vec<usize> = (0..MAX_MONSTERS)
        .filter(|s| cats[*s] && herd[*s].is_some_and(|m| m.alive()))
        .collect();
    let any_dead = (0..MAX_MONSTERS).any(|s| cats[s] && herd[s].is_some_and(|m| !m.alive()));
    if any_dead && shared & pair::DEATH == 0 {
        shared |= pair::DEATH;
        lore.set_word(word::DEATH_AT, now.wrapping_add(1));
        for &s in &living {
            let m = herd[s].as_mut().expect("living");
            set_flag(m, flag::ENRAGED, true);
            set_flag(m, flag::ROLE, false);
            set_flag(m, flag::PERCHED, false);
            m.doing = Doing::Recovery {
                kind: HOWL,
                left: SPECIES.attack(HOWL).recovery,
            };
            m.speed = Fx::ZERO;
            m.brain.last_move = HOWL;
        }
    }
    lore.set_word(word::FLAGS, shared);

    // What each sees, by its own filter -- the same question its glance asks.
    let effects = w.effects;
    let crowd = w.critters;
    let players = w.players;
    let quarry = w.monsters;
    let scene = Scene {
        stones: &field,
        players: &players,
        effects: &effects,
        quarry: &quarry,
        critters: &crowd,
        arena: &ground,
    };
    let mut sees = [false; MAX_MONSTERS];
    for &s in &living {
        let m = herd[s].expect("living");
        let who = (m.brain.target as usize).min(players.len() - 1);
        let p = &players[who];
        let q = Quarry {
            pos: p.pos,
            vel: p.vel,
            alive: p.health > 0,
            aboard: p.aboard(),
            stunned: p.action.stunned(),
        };
        sees[s] = q.alive
            && perceives(&Perceiver {
                monster: &m,
                slot: s,
                head: m.head(),
                quarry: &q,
                who,
                scene: &scene,
                lore: &lore,
            });
        let m = herd[s].as_mut().expect("living");
        let gone = if sees[s] { 0 } else { unseen(m) + 1 };
        set_unseen(m, gone);
    }

    // The scar: which eye, read from where whoever struck it is standing.
    for &s in &living {
        let m = herd[s].as_mut().expect("living");
        if flags(m) & flag::SCAR_PENDING == 0 {
            continue;
        }
        set_flag(m, flag::SCAR_PENDING, false);
        let from = players
            .iter()
            .filter(|p| p.health > 0)
            .min_by_key(|p| math::wide_flat_dist(p.pos, m.pos).raw())
            .map(|p| p.pos)
            .unwrap_or(m.brain.seen);
        let right = V3::from_turns(m.yaw.add(math::QUARTER_TURN));
        scar_eye(m, if right.dot(from.sub(m.pos)).raw() >= 0 { 1 } else { -1 });
    }

    roles(&mut herd, &mut lore, &living, &sees, shared, now);
    perch_spots(&herd, &mut lore, &living, &ground, &field);
    for &s in &living {
        let mut m = herd[s].expect("living");
        moves(&mut m, &ground, &field, now);
        herd[s] = Some(m);
    }
    stuck(&herd, &mut lore, &living, &players, &ground);
    twin(&mut herd, &mut lore, &living, &sees);
    stagger(&mut herd, &living, now);
    apart(&mut herd, &living, &field);

    // What each needs of the other, for the brains next frame.
    lore.set_word(word::NOW, now);
    for (s, m) in herd.iter().enumerate() {
        let Some(m) = m.filter(|_| cats[s]) else {
            continue;
        };
        lore.set_word(word::of(s, word::POS), word_of(m.pos));
        let (from, to) = live_window(&m, now).unwrap_or((0, 0));
        lore.set_word(word::of(s, word::LIVE_FROM), from);
        lore.set_word(word::of(s, word::LIVE_TO), to);
    }
    w.monsters = herd;
    w.lore = lore;
}

/// The frames a cat's move in progress will be live across, from `now`.
pub fn live_window(m: &Monster, now: u32) -> Option<(u32, u32)> {
    let a = SPECIES.attack(m.doing.attacking()?);
    if a.damage <= 0 {
        return None;
    }
    match m.doing {
        Doing::Startup { left, .. } => {
            let from = now.wrapping_add(left as u32 + 1);
            Some((from, from.wrapping_add(a.active as u32)))
        }
        Doing::Active { left, .. } => Some((now, now.wrapping_add(left as u32))),
        _ => None,
    }
}

/// The slot whose cat is nearer its own target.
fn nearer_to_target(herd: &[Option<Monster>; MAX_MONSTERS]) -> usize {
    let d = |s: usize| {
        herd[s].map_or(i32::MAX, |m| {
            math::wide_flat_dist(m.pos, m.brain.seen).raw()
        })
    };
    if d(1) < d(0) { 1 } else { 0 }
}

/// Health below a share of a cat's full pool.
pub fn below(m: &Monster, share: Fx, coop: bool) -> bool {
    let full = if coop {
        Knob::CoopHealth.raw()
    } else {
        SPECIES.health()
    };
    m.health < Fx::from_int(full).mul(share).to_int()
}

/// Draw the next swap clock: fresh or strained, shortened with two hunters.
fn swap_frames(herd: &[Option<Monster>; MAX_MONSTERS], lore: &Lore, shared: u32, now: u32) -> u32 {
    let coop = shared & pair::COOP != 0;
    let strained = herd
        .iter()
        .flatten()
        .any(|m| m.alive() && below(m, Knob::StrainedHealth.fx(), coop));
    let (lo, hi) = if strained {
        (Knob::StrainedSwapMin.raw(), Knob::StrainedSwapMax.raw())
    } else {
        (Knob::SwapMin.raw(), Knob::SwapMax.raw())
    };
    let span = (hi - lo).max(0) as u32 + 1;
    // Drawn from the frame and what it last was: deterministic, and a fresh
    // number each time.
    let mut x = now ^ lore.word(word::SWAP).rotate_left(7) ^ 0x9E37_79B9;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    let frames = lo.max(1) as u32 + x % span;
    if coop {
        Fx::from_int(frames as i32)
            .mul(Knob::CoopSwap.fx())
            .to_int()
            .max(1) as u32
    } else {
        frames
    }
}

/// **The roles**: who holds and who strikes, the swap and its crossing, the
/// regroup when they are split, and the bond when one is wounded.
fn roles(
    herd: &mut [Option<Monster>; MAX_MONSTERS],
    lore: &mut Lore,
    living: &[usize],
    sees: &[bool; MAX_MONSTERS],
    shared: u32,
    now: u32,
) {
    let coop = shared & pair::COOP != 0;
    let mut st = [0u32; MAX_MONSTERS];
    if living.len() < 2 {
        for &s in living {
            st[s] = state::SOLO | if sees[s] { state::SEES } else { 0 };
            if let Some(m) = herd[s].as_mut() {
                set_flag(m, flag::ROLE, false);
            }
        }
        for (s, bits) in st.iter().enumerate() {
            lore.set_word(word::of(s, word::STATE), *bits);
        }
        return;
    }
    let (a, b) = (living[0], living[1]);
    let (ma, mb) = (herd[a].expect("living"), herd[b].expect("living"));

    // The swap clock, and the crossing.
    let mut holder = lore.word(word::HOLDER) as usize % MAX_MONSTERS;
    let mut cross = lore.word(word::CROSS).saturating_sub(1);
    let mut clock = lore.word(word::SWAP);
    if clock > 0 {
        clock -= 1;
    } else {
        // The cat nearer the line of the target's travel becomes the Holder.
        let line = |m: &Monster| {
            let ahead = m.brain.seen.add(
                m.brain
                    .seen_vel
                    .scale(Fx::from_int(SPECIES.prowl_lead() as i32).mul(DT)),
            );
            math::wide_flat_dist(m.pos, ahead).raw()
        };
        let next = if line(&mb) < line(&ma) { b } else { a };
        if next != holder {
            cross = Knob::SwapCrossing.raw().max(0) as u32;
        }
        holder = next;
        clock = swap_frames(herd, lore, shared, now);
    }
    lore.set_word(word::HOLDER, holder as u32);
    lore.set_word(word::CROSS, cross);
    lore.set_word(word::SWAP, clock);

    // Split: too far apart.
    let split = math::wide_flat_dist(ma.pos, mb.pos).raw() > Knob::BondLeash.fx().raw();
    // The bond: one wounded below its share, the other not.
    let hurt = |m: &Monster| below(m, Knob::BondHealth.fx(), coop);
    let bonded = hurt(&ma) != hurt(&mb);

    for &(s, o) in &[(a, b), (b, a)] {
        let me = herd[s].expect("living");
        let mate = herd[o].expect("living");
        let mut bits = state::PARTNER;
        if sees[s] {
            bits |= state::SEES;
        }
        if mate.doing.free() {
            bits |= state::PARTNER_FREE;
        }
        let mut r = if s == holder {
            flag::HOLDER
        } else {
            flag::STRIKER
        };
        if split {
            // The nearer fights alone; the further runs to rejoin.
            let mine = math::wide_flat_dist(me.pos, me.brain.seen).raw();
            let theirs = math::wide_flat_dist(mate.pos, mate.brain.seen).raw();
            if mine < theirs || (mine == theirs && s < o) {
                bits |= state::SOLO;
            } else {
                bits |= state::REJOIN;
            }
            r = 0;
        } else if bonded {
            if hurt(&me) {
                bits |= state::WOUNDED;
                r = flag::STRIKER;
            } else {
                bits |= state::GUARD;
                r = flag::HOLDER;
            }
        }
        if cross > 0 && !split && !bonded {
            bits |= state::CROSSING;
        }
        // A feint is only ever thrown while the Striker is behind the
        // target and free: the feint is a message about the other one.
        if r == flag::HOLDER && !split && !bonded && mate.doing.free() && sees[o] {
            let t = me.brain.seen;
            let round = cos_between(flat(me.pos.sub(t)), flat(mate.pos.sub(t)));
            if round.raw() <= cos_turns(Knob::BehindAngle.fx()).raw() {
                bits |= state::MAY_FEINT;
            }
        }
        st[s] = bits;
        let m = herd[s].as_mut().expect("living");
        set_flag(m, flag::ROLE, false);
        m.own[body::FLAGS] |= r;
    }
    for (s, bits) in st.iter().enumerate() {
        lore.set_word(word::of(s, word::STATE), *bits);
    }
}

pub fn flat(v: V3) -> V3 {
    V3::new(v.x, Fx::ZERO, v.z)
}

/// The cosine of the angle between two flat directions.
pub fn cos_between(a: V3, b: V3) -> Fx {
    if a.flat_len().raw() <= 0 || b.flat_len().raw() <= 0 {
        return Fx::ONE;
    }
    math::wide_normalized(a).dot(math::wide_normalized(b))
}

/// The cosine of an angle in turns.
pub fn cos_turns(t: Fx) -> Fx {
    V3::from_turns(t).x
}

/// **Where each cat would perch**: a top within its reach that is wide enough
/// and low enough -- an arena solid's, or an Elementalist's stone's -- the
/// one nearest the target, so the dive from it reaches.
fn perch_spots(
    herd: &[Option<Monster>; MAX_MONSTERS],
    lore: &mut Lore,
    living: &[usize],
    ground: &crate::arena::Terrain,
    field: &crate::stones::Field,
) {
    for s in 0..MAX_MONSTERS {
        lore.set_word(word::of(s, word::PERCH_TOP), 0);
    }
    let wide = Knob::PerchWidth.fx();
    let highest = Knob::PerchHighest.fx();
    let reach = Knob::PerchReach.fx();
    let inset = math::half(wide);
    for &s in living {
        let m = herd[s].expect("living");
        let target = m.brain.seen;
        let mut best: Option<(V3, Fx, Fx)> = None;
        let mut consider = |at: V3, top: Fx| {
            if top.raw() <= m.pos.y.raw() || top.raw() > highest.raw() {
                return;
            }
            if math::wide_flat_dist(at, m.pos).raw() > reach.raw() {
                return;
            }
            let score = math::wide_flat_dist(at, target);
            if best.is_none_or(|(_, _, b)| score.raw() < b.raw()) {
                best = Some((at, top, score));
            }
        };
        for solid in ground.solids() {
            if solid.hangs() {
                continue;
            }
            let w = solid.max.x.sub(solid.min.x);
            let d = solid.max.z.sub(solid.min.z);
            if w.raw() < wide.raw() || d.raw() < wide.raw() {
                continue;
            }
            // The point of the top nearest the target, kept a half-width in.
            let x = target
                .x
                .clamp(solid.min.x.add(inset), solid.max.x.sub(inset));
            let z = target
                .z
                .clamp(solid.min.z.add(inset), solid.max.z.sub(inset));
            consider(V3::new(x, Fx::ZERO, z), solid.max.y);
        }
        for stone in field.iter().flatten() {
            if stone.radius().add(stone.radius()).raw() < wide.raw() {
                continue;
            }
            consider(
                V3::new(stone.at.x, Fx::ZERO, stone.at.z),
                stone.at.y.add(stone.standing_height()),
            );
        }
        if let Some((at, top, _)) = best {
            lore.set_word(word::of(s, word::PERCH_AT), word_of(at));
            lore.set_int(word::of(s, word::PERCH_TOP), lore::to_cm(top) as i32);
        }
    }
}

/// The height of what a cat is standing over: the floor, an arena top, or a
/// stone it is up on.
pub fn ground_at(pos: V3, ground: &crate::arena::Terrain, field: &crate::stones::Field) -> Fx {
    let mut y = ground.ground_under(pos);
    for stone in field.iter().flatten() {
        if math::wide_flat_dist(pos, stone.at).raw() <= stone.radius().raw() {
            y = y.max(stone.at.y.add(stone.standing_height()));
        }
    }
    y
}

/// Aim a leap at where the target will be when it lands, inside the move's
/// own range: the lob, with the horizon the frames it has left.
pub fn aim_leap(m: &mut Monster, kind: u8, horizon: u16) {
    let a = SPECIES.attack(kind);
    let aim = m.lead_point(horizon);
    let to = flat(aim.sub(m.pos));
    let far = math::wide_flat_len(to);
    let near = a.ideal_range.sub(a.range_span).max(Fx::ZERO);
    // **Nothing reaches under the lip**: a dive comes down no nearer its
    // perch than the lip, its circle and a body clear of it.
    let near = if kind == DIVE {
        near.max(
            Knob::DiveLip
                .fx()
                .add(a.hit_radius)
                .add(crate::tuning::body_radius()),
        )
    } else {
        near
    };
    let most = a.ideal_range.add(a.range_span);
    let reach = far.clamp(near, most);
    let dir = if far.raw() > 0 {
        math::wide_normalized(to)
    } else {
        V3::from_turns(m.yaw)
    };
    m.aim_at(m.pos.add(dir.scale(reach)));
}

/// A xorshift step on a cat's own generator, for the draws made for it: the
/// rake's hold, the feint.
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

/// One cat's move mechanics this frame: the chains, the leaps, the perch, the
/// interpose run and the enrage.
fn moves(m: &mut Monster, ground: &crate::arena::Terrain, field: &crate::stones::Field, now: u32) {
    // **The rake's second hit.** The swipe ends with the paw up; how long it
    // stays up is drawn now, as the second hit commits, so the first cannot
    // teach it. Then the drop.
    match m.doing {
        Doing::Recovery { kind: RAKE, left: 0 } => {
            let lo = Knob::HoldMin.raw();
            let hi = Knob::HoldMax.raw().max(lo);
            let hold = (lo as u32 + draw(m) % ((hi - lo) as u32 + 1)) as u16;
            m.doing = Doing::Startup {
                kind: COCK,
                left: hold,
            };
            m.brain.last_move = COCK;
        }
        Doing::Startup {
            kind: COCK,
            left: 0,
        } => {
            m.doing = Doing::Startup {
                kind: RAKE2,
                left: SPECIES.attack(RAKE2).startup,
            };
            m.hit_used = false;
            m.brain.last_move = RAKE2;
        }
        _ => {}
    }

    // **Enraged**: a missed pounce chains straight into another from the
    // skid, with its own tell; recoveries run quicker and the beat between
    // moves is shorter. No tell shortens below fifteen frames.
    if enraged(m) {
        let pounce = SPECIES.attack(POUNCE);
        if let Doing::Recovery { kind: POUNCE, left } = m.doing {
            if left == pounce.recovery && !m.hit_used && flags(m) & flag::CHAINED == 0 {
                set_flag(m, flag::CHAINED, true);
                let tell = (Knob::ChainTell.raw().max(15) as u16).min(pounce.startup);
                m.doing = Doing::Startup {
                    kind: POUNCE,
                    left: tell,
                };
                m.hit_used = false;
                m.brain.last_move = POUNCE;
                aim_leap(m, POUNCE, tell);
            }
        }
        if let Doing::Recovery { kind, left } = m.doing {
            // Quicker by `EnrageRecovery`: an extra frame off whenever the
            // running share of `1/x - 1` ticks over.
            let r = Knob::EnrageRecovery.fx().max(Fx::from_raw(1));
            let extra = Fx::ONE.div(r).sub(Fx::ONE).max(Fx::ZERO);
            let tick = |n: u32| Fx::from_int((n & 0x7FFF) as i32).mul(extra).to_int();
            if left > 1 && kind != HOWL && tick(now) != tick(now.wrapping_sub(1)) {
                m.doing = Doing::Recovery {
                    kind,
                    left: left - 1,
                };
            }
        }
        if m.doing.free() {
            let beat = Fx::from_int(SPECIES.think_frames() as i32)
                .mul(Knob::EnrageThink.fx())
                .to_int()
                .max(0) as u16;
            m.brain.think_left = m.brain.think_left.min(beat);
        }
        if m.doing.attacking().is_some_and(|k| k != POUNCE) {
            set_flag(m, flag::CHAINED, false);
        }
    }

    // **The windup follows you; the leap does not.** Until it leaves the
    // ground, a leap's mark is kept on where you will be when it lands.
    if let (Some((kind, e)), Doing::Startup { left, .. }) = (elapsed(m), m.doing) {
        if matches!(kind, POUNCE | DIVE) {
            let (leave, air) = flight(kind);
            if e < leave {
                let to_land = left as i32 - (SPECIES.attack(kind).startup as i32 - leave) + air;
                aim_leap(m, kind, to_land.max(0) as u16);
            }
        }
    }

    // **The interpose run**: squared to the line as the run starts, and set
    // as soon as it gets there.
    match m.doing {
        Doing::Startup {
            kind: INTERPOSE,
            left: 0,
        } => {
            let to = flat(m.aimed_at().sub(m.pos));
            if to.flat_len().raw() > 0 {
                m.yaw = math::atan2_turns(to.z, to.x);
                m.yaw_rate = Fx::ZERO;
            }
        }
        Doing::Active {
            kind: INTERPOSE, ..
        } => {
            let along = V3::from_turns(m.yaw);
            let to_go = m.aimed_at().sub(m.pos).dot(along);
            if to_go.raw() <= Knob::LandShort.fx().raw() {
                m.doing = Doing::Recovery {
                    kind: INTERPOSE,
                    left: SPECIES.attack(INTERPOSE).recovery,
                };
                m.speed = Fx::ZERO;
                let to = flat(m.brain.seen.sub(m.pos));
                if to.flat_len().raw() > 0 {
                    m.yaw = math::atan2_turns(to.z, to.x);
                }
            }
        }
        _ => {}
    }

    // **The leaps**: flown from where it left to its mark.
    fly(m, ground, field);

    // **The perch**: landed on a top, it stays there -- rooted, turning --
    // until it dives, or its patience runs out and it drops.
    if let Doing::Recovery { kind: PERCH, .. } = m.doing {
        if !perched(m) {
            set_flag(m, flag::PERCHED, true);
            set_upper(m, 0);
        }
    }
    if perched(m) {
        m.rooted = m.rooted.max(2);
        if m.doing.free() {
            set_upper(m, upper(m) + 1);
            let under_lip =
                math::wide_flat_dist(m.brain.seen, m.pos).raw() <= Knob::DiveLip.fx().raw();
            if upper(m) > Knob::PerchPatience.raw() || under_lip && m.brain.think_left == 0 {
                // Down the slow way: off the edge toward whoever it wanted.
                let to = flat(m.brain.seen.sub(m.pos));
                let dir = if to.flat_len().raw() > 0 {
                    math::wide_normalized(to)
                } else {
                    V3::from_turns(m.yaw)
                };
                m.aim_at(m.pos.add(dir.scale(Knob::DiveLip.fx())));
                m.yaw = math::atan2_turns(dir.z, dir.x);
                m.doing = Doing::Startup {
                    kind: DROP,
                    left: SPECIES.attack(DROP).startup,
                };
                m.hit_used = false;
                m.brain.last_move = DROP;
            }
        }
        // Its perch went from under it -- a stone broken or gone.
        if !matches!(m.doing.attacking(), Some(DIVE | DROP))
            && ground_at(m.pos, ground, field).raw() < m.pos.y.raw()
        {
            set_flag(m, flag::PERCHED, false);
        }
    }
}

/// **Fly a leap**: from where it left the ground to its mark, along a
/// straight line, its height carried from where it left to the top under the
/// mark, with the leap's own arc over it; then the pounce's skid. Outside a
/// leap the cat stands on whatever is under it, and comes down off an edge.
fn fly(m: &mut Monster, ground: &crate::arena::Terrain, field: &crate::stones::Field) {
    let Some(kind) = in_leap(m) else {
        let under = ground_at(m.pos, ground, field);
        m.pos.y = if m.pos.y.raw() > under.raw() {
            m.pos.y.sub(Knob::FallSpeed.fx().mul(DT)).max(under)
        } else {
            under
        };
        return;
    };
    let (_, e) = elapsed(m).expect("in a leap");
    let (leave, air) = flight(kind);
    let a = SPECIES.attack(kind);
    if e == leave {
        // Off the ground: where from, how high, and facing the mark.
        m.own[body::LEAP_FROM] = word_of(m.pos) as i32;
        set_upper(m, lore::to_cm(m.pos.y) as i32);
        let to = flat(m.aimed_at().sub(m.pos));
        if to.flat_len().raw() > 0 {
            m.yaw = math::atan2_turns(to.z, to.x);
            m.yaw_rate = Fx::ZERO;
        }
        if matches!(kind, DIVE | DROP) {
            set_flag(m, flag::PERCHED, false);
        }
        return;
    }
    if e < leave {
        // Still coiled: on whatever is under it.
        m.pos.y = ground_at(m.pos, ground, field).max(if perched(m) {
            m.pos.y
        } else {
            Fx::ZERO
        });
        return;
    }
    let from = point(m.own[body::LEAP_FROM] as u32);
    let from_y = lore::from_cm(upper(m) as i16);
    let mark = m.aimed_at();
    let to = flat(mark.sub(from));
    let dir = if to.flat_len().raw() > 0 {
        math::wide_normalized(to)
    } else {
        V3::from_turns(m.yaw)
    };
    // A body lands short of the mark its claws are on; a perch lands on it.
    let short = match kind {
        PERCH | DROP => Fx::ZERO,
        _ => Knob::LandShort.fx().min(math::wide_flat_len(to)),
    };
    let end = mark.sub(dir.scale(short));
    let t = e - leave;
    if t <= air {
        let u = Fx::from_int(t).div(Fx::from_int(air));
        let land_y = if kind == PERCH {
            ground_at(mark, ground, field)
        } else {
            ground_at(end, ground, field)
        };
        let arc = match kind {
            PERCH => Knob::PerchArc.fx(),
            DIVE => Knob::DiveArc.fx(),
            _ => Fx::ZERO,
        };
        let at = math::lerp3(from, end, u);
        m.pos = V3::new(
            at.x,
            math::lerp(from_y, land_y, u).add(arc.mul(math::hump(u))),
            at.z,
        );
        m.speed = Fx::ZERO;
        return;
    }
    // Down, and skidding on at the volume's own speed, claws out.
    if matches!(m.doing, Doing::Active { .. }) && a.travel.raw() > 0 {
        m.pos = m.pos.add(V3::from_turns(m.yaw).scale(a.travel.mul(DT)));
    }
    m.pos.y = ground_at(m.pos, ground, field);
}

/// **Stuck**: walking somewhere and getting nowhere -- a wall, a platform,
/// a stone between it and where it wants to be. Counted here; past `StuckAfter`
/// it goes round (`mind::prowl_to`) for `StuckRound` frames.
fn stuck(
    herd: &[Option<Monster>; MAX_MONSTERS],
    lore: &mut Lore,
    living: &[usize],
    players: &[crate::state::Player; crate::state::MAX_PLAYERS],
    ground: &crate::arena::Terrain,
) {
    let quarry: [Quarry; crate::state::MAX_PLAYERS] = std::array::from_fn(|i| Quarry {
        pos: players[i].pos,
        vel: players[i].vel,
        alive: players[i].health > 0,
        aboard: players[i].aboard(),
        stunned: players[i].action.stunned(),
    });
    for &s in living {
        let m = herd[s].expect("living");
        let w = lore.word(word::of(s, word::STUCK));
        let (mut count, mut round) = (w & 0xFFFF, w >> 16);
        if round > 0 {
            round -= 1;
            count = 0;
        } else {
            let mind = crate::monster::Mind {
                quarry: &quarry,
                ground,
                lore,
            };
            let goal = if m.doing.free() && !perched(&m) {
                super::mind::prowl_to(&m, &mind)
            } else {
                None
            };
            let far = goal.is_some_and(|g| {
                math::wide_flat_dist(g, m.pos).raw() > Knob::LandShort.fx().mul(Fx::from_int(2)).raw()
            });
            if far && m.speed.abs().raw() < Knob::LandShort.fx().raw() {
                count += 1;
            } else {
                count = 0;
            }
            if count as i32 > Knob::StuckAfter.raw() {
                count = 0;
                round = Knob::StuckRound.raw().max(0) as u32;
            }
        }
        lore.set_word(word::of(s, word::STUCK), (round << 16) | (count & 0xFFFF));
    }
}

/// Is this cat going round something it was stuck on?
pub fn going_round(lore: &Lore, slot: usize) -> bool {
    lore.word(word::of(slot, word::STUCK)) >> 16 > 0
}

/// **The twin pounce.** When both cats are free and see the target, and the
/// target is between them -- the angle between them, seen from it, past
/// `PincerAngle` -- both throw it at once, at one predicted point. Before
/// they leave the ground their mark follows the target; after, it cannot. If
/// it lands on nobody, they land on each other.
fn twin(
    herd: &mut [Option<Monster>; MAX_MONSTERS],
    lore: &mut Lore,
    living: &[usize],
    sees: &[bool; MAX_MONSTERS],
) {
    let wait = lore.word(word::TWIN_WAIT);
    if wait > 0 {
        lore.set_word(word::TWIN_WAIT, wait - 1);
    }
    if living.len() < 2 {
        return;
    }
    let (a, b) = (living[0], living[1]);
    let (ma, mb) = (herd[a].expect("living"), herd[b].expect("living"));
    let tw = SPECIES.attack(TWIN);
    let mid_of = |ma: &Monster, mb: &Monster, horizon: i32| {
        let h = horizon.max(0) as u16;
        math::lerp3(ma.lead_point(h), mb.lead_point(h), math::half(Fx::ONE))
    };

    if ma.doing.attacking() == Some(TWIN) && mb.doing.attacking() == Some(TWIN) {
        // Coiled: the one mark follows the target until they leave.
        if let (Some((_, e)), Doing::Startup { left, .. }) = (elapsed(&ma), ma.doing) {
            let (leave, air) = flight(TWIN);
            if e < leave {
                let to_land = left as i32 - (tw.startup as i32 - leave) + air;
                let mid = mid_of(&ma, &mb, to_land);
                for s in [a, b] {
                    herd[s].as_mut().expect("living").aim_at(mid);
                }
            }
        }
        // The last live frame: if neither has hit anybody and they are on
        // top of each other, both go over.
        if let (Doing::Active { left: 0, .. }, Doing::Active { left: 0, .. }) = (ma.doing, mb.doing)
        {
            let met = math::wide_flat_dist(ma.pos, mb.pos).raw() <= Knob::CrashReach.fx().raw();
            if !ma.hit_used && !mb.hit_used && met {
                for s in [a, b] {
                    let m = herd[s].as_mut().expect("living");
                    m.doing = Doing::Toppled {
                        left: Knob::CrashFrames.raw().max(1) as u16,
                    };
                    m.speed = Fx::ZERO;
                    m.yaw_rate = Fx::ZERO;
                }
            }
        }
        return;
    }

    // Thrown only from both free, both seeing the one target, neither
    // bonded, split or crossing, and the target between them.
    if wait > 0 || !ma.doing.free() || !mb.doing.free() || !sees[a] || !sees[b] {
        return;
    }
    if enraged(&ma) || enraged(&mb) || perched(&ma) || perched(&mb) {
        return;
    }
    let blocked = state::WOUNDED | state::GUARD | state::REJOIN | state::SOLO | state::CROSSING;
    if (state_of(lore, a) | state_of(lore, b)) & blocked != 0 {
        return;
    }
    if ma.brain.grace > 0 || mb.brain.grace > 0 || ma.brain.target != mb.brain.target {
        return;
    }
    let t = math::lerp3(ma.brain.seen, mb.brain.seen, math::half(Fx::ONE));
    let (ta, tb) = (flat(ma.pos.sub(t)), flat(mb.pos.sub(t)));
    let (near, far) = (Knob::TwinNear.fx().raw(), Knob::TwinFar.fx().raw());
    let fits = |v: V3| {
        let d = math::wide_flat_len(v).raw();
        d >= near && d <= far
    };
    if !fits(ta) || !fits(tb) {
        return;
    }
    if cos_between(ta, tb).raw() > cos_turns(Knob::PincerAngle.fx()).raw() {
        return;
    }
    lore.set_word(word::TWIN_WAIT, Knob::TwinCooldown.raw().max(0) as u32);
    let (leave, air) = flight(TWIN);
    let mid = mid_of(&ma, &mb, leave + air);
    for s in [a, b] {
        let m = herd[s].as_mut().expect("living");
        m.doing = Doing::Startup {
            kind: TWIN,
            left: tw.startup,
        };
        m.hit_used = false;
        m.brain.last_move = TWIN;
        m.brain.repeat_left = SPECIES.variety_frames();
        m.brain.think_left = 0;
        m.brain.mirror = false;
        m.aim_at(mid);
    }
}

/// **Two hits never land within `StaggerGap` of each other** on the same
/// hunter, the twin pounce excepted: a move committed this frame whose live
/// window would fall that close to its partner's is not thrown. The brain's
/// scoring keeps to this already (`mind::appetite`); this catches the two
/// committing on the same frame, which neither brain could have seen.
fn stagger(herd: &mut [Option<Monster>; MAX_MONSTERS], living: &[usize], now: u32) {
    if living.len() < 2 {
        return;
    }
    let gap = Knob::StaggerGap.raw().max(0) as u32;
    let (a, b) = (living[0], living[1]);
    // The later slot gives way: it was the second to choose.
    for (s, o) in [(b, a), (a, b)] {
        let m = herd[s].expect("living");
        let mate = herd[o].expect("living");
        let Doing::Startup { kind, left } = m.doing else {
            continue;
        };
        if kind == TWIN
            || left != SPECIES.attack(kind).startup
            || m.brain.target != mate.brain.target
        {
            continue;
        }
        let (Some((f0, t0)), Some((f1, t1))) = (live_window(&m, now), live_window(&mate, now))
        else {
            continue;
        };
        if f0 <= t1.wrapping_add(gap) && f1 <= t0.wrapping_add(gap) {
            let m = herd[s].as_mut().expect("living");
            m.doing = Doing::Prowl;
            m.brain.cooldown[kind as usize] = 0;
            m.brain.think_left = (gap / 2) as u16;
            break;
        }
    }
}

/// **Two bodies, not one**: the cats pushed apart as two fighters are, and
/// out of the Elementalist's stones, which they walk round and do not see
/// through. Not while a leap or a crash has them in the air or on each other.
fn apart(
    herd: &mut [Option<Monster>; MAX_MONSTERS],
    living: &[usize],
    field: &crate::stones::Field,
) {
    let radius = SPECIES.fight_fx(crate::species::FightField::BodyRadius);
    let grounded = |m: &Monster| {
        in_leap(m).is_none() && !matches!(m.doing, Doing::Toppled { .. }) && !perched(m)
    };
    if living.len() == 2 {
        let (a, b) = (living[0], living[1]);
        let (ma, mb) = (herd[a].expect("living"), herd[b].expect("living"));
        if grounded(&ma) && grounded(&mb) {
            let gap = flat(mb.pos.sub(ma.pos));
            let d = math::wide_flat_len(gap);
            let want = radius.add(radius);
            if d.raw() < want.raw() {
                let dir = if d.raw() > 0 {
                    math::wide_normalized(gap)
                } else {
                    V3::from_turns(ma.yaw.add(math::QUARTER_TURN))
                };
                let push = math::half(want.sub(d));
                herd[a].as_mut().expect("living").pos = ma.pos.sub(dir.scale(push));
                herd[b].as_mut().expect("living").pos = mb.pos.add(dir.scale(push));
            }
        }
    }
    for &s in living {
        let m = herd[s].as_mut().expect("living");
        if !grounded(m) {
            continue;
        }
        for stone in field.iter().flatten() {
            let gap = flat(m.pos.sub(stone.at));
            let d = math::wide_flat_len(gap);
            let want = stone.radius().add(radius);
            let top = stone.at.y.add(stone.standing_height());
            if d.raw() < want.raw() && m.pos.y.raw() < top.raw() {
                let dir = if d.raw() > 0 {
                    math::wide_normalized(gap)
                } else {
                    V3::from_turns(m.yaw)
                };
                let out = stone.at.add(dir.scale(want));
                m.pos = V3::new(out.x, m.pos.y, out.z);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Reading it
// ---------------------------------------------------------------------------

/// **What the cats draw besides their telegraphs**: the feint's coil marks
/// the floor exactly as the pounce's does -- the tail is the only difference
/// between them, by design (`the-pair.md` §2) -- so a feint draws the circle
/// its pounce would have.
pub fn marks(w: &World, out: &mut Marks) {
    for m in w.monsters.iter().flatten() {
        if m.species != SPECIES.id {
            continue;
        }
        if let Doing::Startup { kind: FEINT, left } = m.doing {
            let total = SPECIES.attack(FEINT).startup.max(1) as i32;
            out.push(Mark {
                at: m.aimed_at(),
                radius: SPECIES.attack(POUNCE).hit_radius,
                height: Fx::ZERO,
                look: MarkLook::Warning,
                progress: Fx::from_int(total - left as i32).div(Fx::from_int(total)),
            });
        }
    }
}

/// Is this the Striker's ambush? For the report's "ambush after a feint".
pub fn ambushing(m: &Monster) -> bool {
    m.doing.attacking() == Some(AMBUSH)
}

/// The frames it has been perched.
pub fn perched_for(m: &Monster) -> i32 {
    if perched(m) { upper(m) } else { 0 }
}
