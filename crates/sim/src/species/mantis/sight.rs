//! **Its eyes: a delay line, never the present** (`mantis.md` §5).
//!
//! The Ridgeback glances: a sample of position and velocity every few frames,
//! projected forward. The Mantis needs to see *moves*, so it reads each
//! fighter as they were `D` frames ago, where `D` wanders between `SightMin`
//! (15) and `SightMax` (21) a frame at a time -- the sparring bot's hard
//! level, exactly (`sparring.md`), because that is the fastest that is still
//! fair.
//!
//! What it keeps, in the hunt's lore, written by its frame hook from the
//! snapshot as the frame finds it ([`record`]):
//!
//! - **a ring of samples per fighter**, one every `SampleEvery` frames: where
//!   they stood, how high, whether they were on the floor, able to act, alive.
//!   Read at the newest sample *at or older than* `D` ([`glimpse`]) -- never
//!   fresher -- and the velocity is the difference to the sample before it.
//! - **what each fighter began, and when** ([`Deed`]): a short history of
//!   their action as the snapshot holds it -- a move and its kind, a dodge, a
//!   guard, a jump -- with the frame each began. What it *sees* a fighter
//!   doing is the newest deed that began at least `D` frames ago
//!   ([`deed_seen`]). A move that starts and ends inside `D` frames is never
//!   seen at all, which is correct.
//!
//! **Nothing here reads an input.** The record is written from `Player::action`
//! and `Player::pos` -- state both peers compute -- and the brain reads it only
//! through [`glimpse`] and [`deed_seen`], both of which refuse anything newer
//! than `D`. `tests/mantis.rs::it_never_sees_the_present` holds that.
//!
//! **"Now"**: the hook runs at frame `t` after the creatures have stepped and
//! before the fighters do, so the state it records is the one frame `t - 1`
//! left, and it is stamped `t - 1`. Anything that decides at frame `t` -- the
//! hook, or the brain stepping at `t` -- may read stamps no newer than
//! `t - D`.

use crate::fixed::Fx;
use crate::lore::{self, Lore};
use crate::math::V3;
use crate::monster::Monster;
use crate::species::Sight;
use crate::state::{Action, MAX_PLAYERS, World};

use super::Knob;

/// Samples a fighter's ring holds. With one every four frames, eight reach
/// back thirty-two frames: past the latest it ever sees, with a sample to
/// difference against.
pub const RING: usize = 8;

/// Deeds a fighter's history holds: what they are doing, and the three
/// before it. Four is room for a chain of three links and the walk it came
/// out of.
pub const DEEDS: usize = 4;

/// The words of its own region of the lore (`Lore::word`): its eyes, from
/// the start, then what [`super::fight`] keeps after them.
pub mod word {
    use super::{DEEDS, MAX_PLAYERS, RING};

    /// The frame the hook last ran on (`World::frame`).
    pub const NOW: usize = 0;
    /// `D`, how late it sees (low byte); frames until it wanders next
    /// (the next sixteen bits).
    pub const DELAY: usize = 1;
    /// Its eyes' own generator, for the wander.
    pub const RNG: usize = 2;
    /// The newest deed's slot, per fighter: two bits each.
    pub const HEADS: usize = 3;
    /// The samples: per fighter, [`RING`] of three words -- where (x and z
    /// in centimetres), height and flags, and the stamp.
    pub const SAMPLES: usize = 4;
    pub const PER_SAMPLE: usize = 3;
    /// The deeds: per fighter, [`DEEDS`] words, a deed in the top byte and
    /// the stamp it began on below it.
    pub const DEEDS_AT: usize = SAMPLES + MAX_PLAYERS * RING * PER_SAMPLE;
    /// The first word after the eyes: [`super::super::fight`]'s.
    pub const END: usize = DEEDS_AT + MAX_PLAYERS * DEEDS;
}

/// The bits of a sample's flag half.
mod flag {
    pub const ALIVE: u32 = 1 << 16;
    /// In hitstun, staggered, blockstunned or held.
    pub const STUNNED: u32 = 1 << 17;
    /// Off the floor -- higher than `AloftAbove` over whatever is under them,
    /// and not standing on anything.
    pub const ALOFT: u32 = 1 << 18;
    /// The sample has been written.
    pub const TAKEN: u32 = 1 << 19;
}

/// **What a fighter is doing, as it can be seen**: their action as the
/// snapshot holds it, coarsened to what a person watching would name.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Deed {
    /// Standing, walking, crouching: on the floor and doing nothing.
    Free,
    /// Off the floor and doing nothing else: a jump.
    Aloft,
    Dodge,
    Guard,
    /// Hit, staggered, blocking a blow, held: not acting.
    Stunned,
    Dead,
    /// A move of their class, by its kind: from its first startup frame
    /// through its recovery, the channel in front of it included.
    Move(u8),
}

impl Deed {
    /// **A commitment** (§5): anything but standing, walking, crouching,
    /// guarding -- a move, a dodge, a jump. What the coil releases on.
    pub const fn commits(self) -> bool {
        matches!(self, Deed::Aloft | Deed::Dodge | Deed::Move(_))
    }

    const fn code(self) -> u32 {
        match self {
            Deed::Free => 0,
            Deed::Aloft => 1,
            Deed::Dodge => 2,
            Deed::Guard => 3,
            Deed::Stunned => 4,
            Deed::Dead => 5,
            Deed::Move(kind) => 0x20 + (kind as u32 & 0x1F),
        }
    }

    const fn of_code(c: u32) -> Deed {
        match c {
            1 => Deed::Aloft,
            2 => Deed::Dodge,
            3 => Deed::Guard,
            4 => Deed::Stunned,
            5 => Deed::Dead,
            c if c >= 0x20 => Deed::Move((c - 0x20) as u8),
            _ => Deed::Free,
        }
    }

    /// What a fighter is doing in the snapshot.
    pub fn of(p: &crate::state::Player, aloft: bool) -> Deed {
        if p.health <= 0 {
            return Deed::Dead;
        }
        match p.action {
            Action::Startup { kind, .. }
            | Action::Active { kind, .. }
            | Action::Recovery { kind, .. }
            | Action::Channel { kind, .. } => Deed::Move(kind),
            Action::Dodge { .. } => Deed::Dodge,
            Action::Guard { .. } => Deed::Guard,
            a if a.stunned() => Deed::Stunned,
            _ if aloft => Deed::Aloft,
            _ => Deed::Free,
        }
    }
}

/// What it saw of one fighter: a sample, read late.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Glimpse {
    pub pos: V3,
    pub vel: V3,
    pub alive: bool,
    pub stunned: bool,
    pub aloft: bool,
    /// The frame the sample is of.
    pub stamp: u32,
}

/// Low twenty-four bits: the stamp a deed began on.
const STAMP: u32 = 0x00FF_FFFF;

/// The frame a decision is being made on, from the record: one after the
/// hook last ran, for the brain stepping before it this frame.
pub fn now(lore: &Lore) -> u32 {
    lore.word(word::NOW).wrapping_add(1)
}

/// **How late it sees, now**: `D`.
pub fn delay(lore: &Lore) -> u16 {
    let d = lore.word(word::DELAY) & 0xFF;
    // A record nobody has written yet sees at its latest.
    if d == 0 {
        return Knob::SightMax.frames();
    }
    d as u16
}

fn sample_word(who: usize, slot: usize, k: usize) -> usize {
    word::SAMPLES + (who * RING + slot) * word::PER_SAMPLE + k
}

fn deed_word(who: usize, slot: usize) -> usize {
    word::DEEDS_AT + who * DEEDS + slot
}

fn head(lore: &Lore, who: usize) -> usize {
    ((lore.word(word::HEADS) >> (who * 2)) & 0b11) as usize
}

fn set_head(lore: &mut Lore, who: usize, slot: usize) {
    let w = lore.word(word::HEADS) & !(0b11 << (who * 2));
    lore.set_word(word::HEADS, w | ((slot as u32 & 0b11) << (who * 2)));
}

/// **Write this frame into the record**, from the snapshot as the frame finds
/// it: the hook's first act. Also wanders `D`.
pub fn record(w: &mut World) {
    let t = w.frame;
    let stamp = t.wrapping_sub(1);
    let ground = w.terrain();
    let every = Knob::SampleEvery.raw().clamp(1, 4) as u32;
    let mut lore = w.lore;
    for who in 0..MAX_PLAYERS {
        let p = w.players[who];
        let floor = ground.ground_under(p.pos);
        let aloft = !p.grounded && p.pos.y.sub(floor).raw() > Knob::AloftAbove.fx().raw();
        // The sample, every `every` frames.
        if stamp % every == 0 {
            let slot = ((stamp / every) as usize) % RING;
            let mut flags = flag::TAKEN;
            if p.health > 0 {
                flags |= flag::ALIVE;
            }
            if p.action.stunned() {
                flags |= flag::STUNNED;
            }
            if aloft {
                flags |= flag::ALOFT;
            }
            lore.set_word(
                sample_word(who, slot, 0),
                lore::halves(lore::to_cm(p.pos.x), lore::to_cm(p.pos.z)),
            );
            lore.set_word(
                sample_word(who, slot, 1),
                (lore::to_cm(p.pos.y) as u16 as u32) | flags,
            );
            lore.set_word(sample_word(who, slot, 2), stamp);
        }
        // The deed, every frame: a new one begins when it changes.
        let deed = Deed::of(&p, aloft);
        let h = head(&lore, who);
        let newest = lore.word(deed_word(who, h));
        let fresh = lore.word(word::NOW) == 0 && newest == 0;
        if fresh || Deed::of_code(newest >> 24) != deed {
            let next = if fresh { 0 } else { (h + 1) % DEEDS };
            lore.set_word(deed_word(who, next), (deed.code() << 24) | (stamp & STAMP));
            set_head(&mut lore, who, next);
        }
    }
    wander(&mut lore);
    lore.set_word(word::NOW, t);
    w.lore = lore;
}

/// `D` wanders a frame up or down every `SightWander` frames, inside its
/// bounds: sharp for a while, then slow for a while, which is what attention
/// does.
fn wander(lore: &mut Lore) {
    let lo = Knob::SightMin.raw().max(1) as u32;
    let hi = (Knob::SightMax.raw().max(1) as u32).max(lo);
    let packed = lore.word(word::DELAY);
    let mut d = packed & 0xFF;
    let mut left = (packed >> 8) & 0xFFFF;
    if d == 0 {
        // The first frame: it starts in the middle of its range.
        d = (lo + hi) / 2;
        left = Knob::SightWander.raw().max(1) as u32;
    } else if left == 0 {
        let mut x = lore.word(word::RNG);
        if x == 0 {
            x = 0x9E37_79B9;
        }
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        lore.set_word(word::RNG, x);
        d = if x & 1 == 0 {
            d + 1
        } else {
            d.saturating_sub(1)
        };
        left = Knob::SightWander.raw().max(1) as u32;
    } else {
        left -= 1;
    }
    let d = d.clamp(lo, hi);
    lore.set_word(word::DELAY, (d & 0xFF) | ((left & 0xFFFF) << 8));
}

/// The sample of fighter `who` that is the newest at or older than `upto`,
/// with the one before it, if both were taken.
fn samples(lore: &Lore, who: usize, upto: u32) -> Option<(usize, Option<usize>)> {
    let mut best: Option<(usize, u32)> = None;
    let mut before: Option<(usize, u32)> = None;
    for slot in 0..RING {
        let flags = lore.word(sample_word(who, slot, 1));
        if flags & flag::TAKEN == 0 {
            continue;
        }
        let s = lore.word(sample_word(who, slot, 2));
        // Older than `upto`, by the wrapping distance back from it.
        if upto.wrapping_sub(s) > u32::MAX / 2 {
            continue;
        }
        if best.is_none_or(|(_, b)| upto.wrapping_sub(s) < upto.wrapping_sub(b)) {
            before = best;
            best = Some((slot, s));
        } else if before.is_none_or(|(_, b)| upto.wrapping_sub(s) < upto.wrapping_sub(b)) {
            before = Some((slot, s));
        }
    }
    best.map(|(slot, _)| (slot, before.map(|(s, _)| s)))
}

fn read(lore: &Lore, who: usize, slot: usize) -> (V3, u32, u32) {
    let at = lore.word(sample_word(who, slot, 0));
    let hf = lore.word(sample_word(who, slot, 1));
    let pos = V3::new(
        lore::from_cm(lore::lo(at)),
        lore::from_cm(hf as u16 as i16),
        lore::from_cm(lore::hi(at)),
    );
    (pos, hf, lore.word(sample_word(who, slot, 2)))
}

/// **Fighter `who` as it sees them at frame `t`**: the newest sample no
/// newer than `t - D`. `None` before it has seen them at all.
pub fn glimpse(lore: &Lore, who: usize, t: u32) -> Option<Glimpse> {
    let upto = t.wrapping_sub(delay(lore) as u32);
    let (slot, before) = samples(lore, who, upto)?;
    let (pos, flags, stamp) = read(lore, who, slot);
    let vel = match before {
        Some(b) => {
            let (was, _, then) = read(lore, who, b);
            let gap = stamp.wrapping_sub(then).max(1) as i32;
            let per = crate::TICK_HZ as i32;
            let d = pos.sub(was);
            V3::new(
                Fx::from_raw(((d.x.raw() as i64 * per as i64) / gap as i64) as i32),
                Fx::from_raw(((d.y.raw() as i64 * per as i64) / gap as i64) as i32),
                Fx::from_raw(((d.z.raw() as i64 * per as i64) / gap as i64) as i32),
            )
        }
        None => V3::ZERO,
    };
    Some(Glimpse {
        pos,
        vel,
        alive: flags & flag::ALIVE != 0,
        stunned: flags & flag::STUNNED != 0,
        aloft: flags & flag::ALOFT != 0,
        stamp,
    })
}

/// **What it sees fighter `who` doing at frame `t`**: the newest deed that
/// began at least `D` frames before, and the stamp it began on. `Free` from
/// the start of time, before anything is recorded.
pub fn deed_seen(lore: &Lore, who: usize, t: u32) -> (Deed, u32) {
    let upto = t.wrapping_sub(delay(lore) as u32) & STAMP;
    let h = head(lore, who);
    for back in 0..DEEDS {
        let slot = (h + DEEDS - back) % DEEDS;
        let w = lore.word(deed_word(who, slot));
        if w == 0 && back > 0 {
            break;
        }
        let began = w & STAMP;
        // At or before `upto`, by the wrapping distance back from it.
        if (upto.wrapping_sub(began) & STAMP) <= STAMP / 2 {
            return (Deed::of_code(w >> 24), began);
        }
    }
    (Deed::Free, 0)
}

/// **Its glance**, for `FightDecl::sight`: the nearest fighter it sees alive
/// and where they were -- with the target it has kept unless somebody else is
/// much nearer, as the glance keeps one -- read through the delay line.
pub fn sight(m: &Monster, lore: &Lore) -> Option<Sight> {
    let t = now(lore);
    let mut best: Option<(usize, Glimpse, Fx)> = None;
    let mut seen: [Option<Glimpse>; MAX_PLAYERS] = [None; MAX_PLAYERS];
    for (who, slot) in seen.iter_mut().enumerate() {
        let Some(g) = glimpse(lore, who, t).filter(|g| g.alive) else {
            continue;
        };
        *slot = Some(g);
        let d = crate::math::wide_flat_len(g.pos.sub(m.pos));
        if best.is_none_or(|(_, _, b)| d.raw() < b.raw()) {
            best = Some((who, g, d));
        }
    }
    let (mut pick, mut g, near) = best?;
    let held = m.brain.target as usize;
    if held < MAX_PLAYERS && held != pick {
        if let Some(h) = seen[held] {
            let mine = crate::math::wide_flat_len(h.pos.sub(m.pos));
            if near.raw() >= mine.mul(m.sp().target_switch()).raw() {
                pick = held;
                g = h;
            }
        }
    }
    Some(Sight {
        target: pick as u8,
        pos: g.pos,
        vel: g.vel,
        stunned: g.stunned,
    })
}
