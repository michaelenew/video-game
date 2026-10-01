//! **The habit memory -- the most contentious rule in the cast** (`mantis.md`
//! §5).
//!
//! A ring of the last [`DEPTH`] moves that **struck its guard** -- blocked,
//! parried, or broke it -- by move identity: class and move, which is what it
//! saw. Shared across hunters: it recognises the swing, not the swinger. An
//! entry becomes usable `HabitSettle` frames after it was written. When one
//! move holds `HabitAnticipate` of the ring (`HabitHurt` when hurt),
//! `ready(m)` scores, and at a decision point it may take **Ready** against
//! that move. Throw that move into it and it is parried and countered; throw
//! anything else, or wait out its hold, and the entry is forgotten.
//!
//! **It is written from one place**: the guard (`fight::guard`), as a blow
//! meets it -- a simulation event both peers compute, which a swing that
//! whiffs never produces -- through the body's mailbox, drained here by the
//! next frame's hook ([`drain`]). Nothing in this file is handed an input,
//! and nothing it remembers is fresher than the frame the blow landed on.
//! `tests/mantis.rs::nothing_it_remembers_came_from_an_input` holds that.
//!
//! **The fallback, behind `Mantis · habit`** (on): off, nothing is written,
//! and Ready is taken against a move [`guess`]ed from the ones that reach it
//! from where its target stands -- geometry, read by its eyes.

use crate::Class;
use crate::fixed::Fx;
use crate::lore::Lore;
use crate::math::{self, V3};
use crate::monster::{Doing, Monster};
use crate::species::{Mark, MarkLook, Marks};
use crate::state::MAX_PLAYERS;

use super::fight::{self, body, word};
use super::sight;
use super::{BLADE_L_PART, BLADE_R_PART, Knob, READY, SPECIES};

/// Entries the ring holds: the most `HabitDepth` may ask for.
pub const DEPTH: usize = 4;

/// The low twenty bits of an entry: the frame it was written on.
const STAMP: u32 = 0x000F_FFFF;
/// Where an entry's move code sits.
const CODE_SHIFT: u32 = 20;

/// **A move's identity**, as it sees it: the class carrying it and which of
/// its moves -- never zero. **The swing, not the button press**: a
/// Champion's string is one weapon swung three ways, and a sword is a sword
/// whether it opens the string or ends it -- so his nine chain moves are
/// remembered as the weapon that threw them (`moves::champion::weapon`). A
/// sword opened with every time is a sword three times, however the string
/// went on; the Champion who mixes weapons is the one it cannot read
/// (`mantis.md` §7).
pub fn code(class: Class, kind: u8) -> i32 {
    use crate::moves::champion as c;
    let kind = match (class, c::link_of(kind)) {
        (Class::Champion, Some(_)) => c::FIRST + c::weapon(kind),
        _ => kind,
    };
    ((class as i32 + 1) << 8) | kind as i32
}

/// The class and move a code names.
pub fn decode(code: i32) -> Option<(Class, u8)> {
    let c = (code >> 8) & 0xF;
    if c == 0 {
        return None;
    }
    let class = *crate::class::ALL_CLASSES.get(c as usize - 1)?;
    Some((class, (code & 0xFF) as u8))
}

/// One remembered move: its code, who threw it, the frame it struck.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Entry {
    pub code: i32,
    pub who: usize,
    pub stamp: u32,
}

fn head(lore: &Lore) -> usize {
    ((lore.word(word::HABIT_META) >> 8) & 0b11) as usize
}

/// The entries, newest first, as many as `HabitDepth` asks.
pub fn entries(lore: &Lore) -> [Option<Entry>; DEPTH] {
    let depth = (Knob::HabitDepth.raw().clamp(0, DEPTH as i32)) as usize;
    let h = head(lore);
    let meta = lore.word(word::HABIT_META);
    std::array::from_fn(|back| {
        if back >= depth {
            return None;
        }
        let slot = (h + DEPTH - back) % DEPTH;
        let w = lore.word(word::HABIT + slot);
        let code = (w >> CODE_SHIFT) as i32;
        if code == 0 {
            return None;
        }
        Some(Entry {
            code,
            who: ((meta >> slot) & 1) as usize,
            stamp: w & STAMP,
        })
    })
}

/// Write one: the newest, over the oldest.
fn push(lore: &mut Lore, code: i32, who: usize, stamp: u32) {
    let next = (head(lore) + 1) % DEPTH;
    lore.set_word(
        word::HABIT + next,
        ((code as u32 & 0xFFF) << CODE_SHIFT) | (stamp & STAMP),
    );
    let mut meta = lore.word(word::HABIT_META);
    meta = (meta & !(1 << next)) | (((who & 1) as u32) << next);
    meta = (meta & !(0b11 << 8)) | ((next as u32) << 8);
    lore.set_word(word::HABIT_META, meta);
}

/// **Forget the newest entry of a move**: a guess it got wrong costs it the
/// guess.
pub fn forget(lore: &mut Lore, code: i32) {
    let h = head(lore);
    for back in 0..DEPTH {
        let slot = (h + DEPTH - back) % DEPTH;
        let w = lore.word(word::HABIT + slot);
        if (w >> CODE_SHIFT) as i32 == code {
            lore.set_word(word::HABIT + slot, 0);
            return;
        }
    }
}

/// **The hook's half**: what struck its guard last frame, into the ring (when
/// the flag is on) -- stamped with the frame it landed on; and a Ready that
/// guessed wrong, or ran out, lowered and its memory forgotten.
pub fn drain(m: &mut Monster, lore: &mut Lore, t: u32) {
    let on = Knob::Habit.raw() != 0;
    for who in 0..MAX_PLAYERS {
        if let Some((code, _)) = fight::mail(m, who) {
            if on && code != 0 {
                push(lore, code, who, t.wrapping_sub(1));
            }
        }
    }
    m.own[body::MAILBOX] = 0;

    let waited = m.own[body::READY_FOR];
    let wrong = fight::flags(m) & fight::flag::WRONG != 0;
    if wrong {
        m.own[body::FLAGS] &= !fight::flag::WRONG;
        if matches!(
            m.doing,
            Doing::Startup { kind: READY, .. } | Doing::Active { kind: READY, .. }
        ) {
            m.doing = Doing::Recovery {
                kind: READY,
                left: SPECIES.attack(READY).recovery,
            };
        }
    }
    // Over -- wrong-footed, or its hold run out with nothing thrown: the
    // entry is forgotten.
    let over = matches!(m.doing, Doing::Recovery { kind: READY, left } if left == SPECIES.attack(READY).recovery);
    if (wrong || over) && waited != 0 {
        if on {
            forget(lore, waited);
        }
        m.own[body::READY_FOR] = 0;
    }
}

/// The move one entry holds the most of among usable entries -- older than
/// `HabitSettle` -- with how many copies, and who threw the newest.
pub fn majority(lore: &Lore, t: u32) -> Option<(i32, i32, usize)> {
    let settle = Knob::HabitSettle.raw().max(0) as u32;
    let all = entries(lore);
    let usable = |e: &Entry| (t.wrapping_sub(e.stamp) & STAMP) >= settle;
    let mut best: Option<(i32, i32, usize)> = None;
    for e in all.iter().flatten().filter(|e| usable(e)) {
        let copies = all
            .iter()
            .flatten()
            .filter(|o| usable(o) && o.code == e.code)
            .count() as i32;
        if best.is_none_or(|(_, n, _)| copies > n) {
            best = Some((e.code, copies, e.who));
        }
    }
    best
}

/// **`ready(m)`** (§5): `HabitWeight` for each copy of one move past the
/// first, once it holds `HabitAnticipate` of the ring (`HabitHurt` hurt),
/// and only while that move's thrower is inside its reach and `HabitReach`.
pub fn ready_score(m: &Monster, lore: &Lore, t: u32) -> i32 {
    let Some((code, copies, who)) = majority(lore, t) else {
        return 0;
    };
    let need = if fight::hurt(m) {
        Knob::HabitHurt.raw()
    } else {
        Knob::HabitAnticipate.raw()
    };
    if need <= 0 || copies < need {
        return 0;
    }
    let Some((class, kind)) = decode(code) else {
        return 0;
    };
    let Some(g) = sight::glimpse(lore, who, t).filter(|g| g.alive) else {
        return 0;
    };
    let mv = crate::moves::get(class, kind);
    let reach = mv
        .reach
        .add(mv.radius)
        .add(crate::tuning::body_radius())
        .add(Knob::HabitReach.fx());
    if math::wide_flat_dist(g.pos, m.pos).raw() > reach.raw() {
        return 0;
    }
    Knob::HabitWeight.raw() * (copies - 1)
}

/// **`guess(m)`**, the fallback with no memory: a blockable move of its
/// target's class that reaches it from where it sees them, drawn by its own
/// generator. Geometry, read by its eyes; never a memory.
pub fn guess(m: &Monster, lore: &Lore, t: u32, roll: u32) -> Option<i32> {
    let who = (m.brain.target as usize).min(MAX_PLAYERS - 1);
    let class = fight::class_of(lore, who)?;
    let g = sight::glimpse(lore, who, t).filter(|g| g.alive)?;
    let range = math::wide_flat_dist(g.pos, m.pos);
    let mut fits = [0u8; crate::moves::MAX_SLOTS];
    let mut n = 0;
    for kind in 0..crate::moves::slots(class) {
        let mv = crate::moves::get(class, kind as u8);
        let reach = mv.reach.add(mv.radius).add(crate::tuning::body_radius());
        if mv.unblockable || mv.damage <= 0 || range.raw() > reach.raw() {
            continue;
        }
        fits[n] = kind as u8;
        n += 1;
    }
    if n == 0 {
        return None;
    }
    Some(code(class, fits[(roll as usize) % n]))
}

/// **The memory, drawn** (§6): a notch along each blade's outer edge for
/// every move it remembers, in the thrower's class, the ones a Ready is
/// waiting for lit. What makes the rule fair: a player who looks at its
/// blades knows what it is waiting for.
pub fn notches(m: &Monster, lore: &Lore, out: &mut Marks) {
    if Knob::Habit.raw() == 0 {
        return;
    }
    let rig = m.rig();
    let waited = m.own[body::READY_FOR];
    for (i, e) in entries(lore).iter().enumerate() {
        let Some(e) = e else { continue };
        let Some((class, _)) = decode(e.code) else {
            continue;
        };
        for blade in [BLADE_L_PART, BLADE_R_PART] {
            if m.broken(blade) {
                continue;
            }
            let sh = SPECIES.shape(blade);
            let along = sh.max.x.sub(sh.min.x);
            let x = sh.min.x.add(
                along
                    .mul(Fx::from_int(i as i32 + 1))
                    .div(Fx::from_int(DEPTH as i32 + 1)),
            );
            let at = rig.part_to_world(blade, V3::new(x, sh.max.y, sh.max.z));
            out.push(Mark {
                at,
                radius: Knob::NotchSize.fx(),
                height: Knob::NotchSize.fx(),
                look: MarkLook::Notch {
                    class: class as u8,
                    lit: waited == e.code,
                },
                progress: Fx::ONE,
            });
        }
    }
}
