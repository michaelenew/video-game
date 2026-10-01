//! What the Mantis does to the fight: its guard, its frame hook, its blades.
//! See `docs/design/creatures/mantis.md`.
//!
//! **The guard is a seam, not a special case** (`FightDecl::guard`): every
//! place a blow reaches a creature asks it first (`Monster::take_blow`), from
//! the point the blow comes from -- the same point the fighters' own guard is
//! handed (`state::guard_against`), through the same cone test
//! (`state::in_guard_arc`), with one more comparison for the elevation. A
//! blocked blow pushes it and holds it; a parried one is its counter; an
//! unblockable through a raised guard breaks it.
//!
//! **The frame hook** ([`frame`]) is its reflexes and its body: it writes its
//! eyes ([`super::sight::record`]), drains what struck its guard into its
//! memory ([`super::habit`]), raises its guard on a blockable commitment it
//! saw -- inside the beat, which nothing else may do -- lowers it on a guard
//! breaker it saw, releases the coil on a commitment, chooses the second
//! slash, flies the leap and the dive, hops after the flare, walks and turns
//! with its guard up, and keeps the prayer's haste. It never reads an input:
//! everything it decides from is the delay line, and the delay line is the
//! snapshot, late.
//!
//! **Its state on the body** (`Monster::own`, [`body`]) is what the hit path
//! reads without a world: the haste, the stage, the Ready it has taken, and a
//! mailbox of what struck its guard this frame for the hook to remember.
//! Everything else is in the hunt's lore, after its eyes ([`word`]).

use crate::DT;
use crate::fixed::Fx;
use crate::lore::{self, Layout, Lore};
use crate::math::{self, V3};
use crate::monster::{Blow, Doing, Guarded, Monster};
use crate::species::{FightDecl, FightField, Mark, MarkLook, Marks};
use crate::state::{self, MAX_PLAYERS, World};

use super::habit;
use super::sight::{self, Deed};
use super::{
    ARM_L_PART, ARM_R_PART, BLADE_L_PART, BLADE_R_PART, BROKEN, COUNTER, DIVE, FLARE, GUARD, Knob,
    LEAP, LUNGE, PRAYER, READY, SLASH, SLASH_FAST, SLASH_HELD, SPECIES, STAGGER, stance,
};

pub static FIGHT: FightDecl = FightDecl {
    layout: Layout {
        hazards: 0,
        noises: 0,
        objectives: 0,
        own: OWN_CELLS,
    },
    row: true,
    collides: true,
    keeps_height: true,
    frame: Some(frame),
    appetite: Some(super::mind::appetite),
    prowl_to: Some(super::mind::prowl_to),
    commit: Some(super::mind::commit),
    hide: Some(hide),
    struck: Some(struck),
    landed: Some(landed),
    marks: Some(marks),
    signs: Some(signs),
    bumped: Some(bumped),
    guard: Some(guard),
    covers: Some(covers),
    sight: Some(sight::sight),
    ..FightDecl::PLAIN
};

/// The cells of the lore it keeps: its eyes, then [`word`].
pub const OWN_CELLS: u8 = (word::END.div_ceil(4)) as u8;

// ---------------------------------------------------------------------------
// The state
// ---------------------------------------------------------------------------

/// Its four words on the body (`Monster::own`).
pub mod body {
    /// [`super::flag`]s.
    pub const FLAGS: usize = 0;
    /// **What struck its guard this frame**, a half per fighter: a valid
    /// bit, what the guard made of it, the class and the move. Written by
    /// the guard, drained into the memory by the next frame's hook
    /// ([`super::habit::drain`]).
    pub const MAILBOX: usize = 1;
    /// Where a leap left the floor, in centimetres (`lore::halves`): the
    /// dive flies from above it.
    pub const LEAP_FROM: usize = 2;
    /// **The Ready it has taken**: the move it is waiting for, as
    /// [`super::habit::code`], or zero.
    pub const READY_FOR: usize = 3;
}

/// The bits of [`body::FLAGS`].
pub mod flag {
    /// A bit, or a field's place in the word: a tag, not a quantity.
    pub type Bits = i32;

    /// Haste pips left, three bits from the bottom.
    pub const PIPS: Bits = 0b111;
    /// The move under way was hasted: its recovery is halved, and no beat
    /// follows it.
    pub const HASTED: Bits = 1 << 3;
    /// Two hunters.
    pub const COOP: Bits = 1 << 4;
    /// Below `HurtBelow` of its health.
    pub const HURT: Bits = 1 << 5;
    /// Below `DesperateBelow`, or both blades gone.
    pub const DESPERATE: Bits = 1 << 6;
    /// A blow landed on its Ready that it was not waiting for: the guess
    /// was wrong, and the memory it came from is forgotten.
    pub const WRONG: Bits = 1 << 7;
    /// It was free last frame: the beat it is in has already been scaled.
    pub const WAS_FREE: Bits = 1 << 8;
    /// The two-hunter health has been set.
    pub const SET_UP: Bits = 1 << 9;
    /// Why the counter under way was thrown (two bits): a parry of a move
    /// it saw, of the move its Ready waited for, or in its prayer.
    pub const CAUSE_SHIFT: Bits = 10;
    pub const CAUSE: Bits = 0b11 << CAUSE_SHIFT;
    pub const BY_SIGHT: Bits = 1;
    pub const BY_READY: Bits = 2;
    pub const BY_PRAYER: Bits = 3;
    /// The coil under way released on a commitment it saw (not at its end).
    pub const ON_SIGHT: Bits = 1 << 12;
    /// The parry under way was of a move it saw coming: its guard was
    /// raised by its eyes, not by its own choice.
    pub const SAW_IT: Bits = 1 << 13;
    /// Its guard came down early, on a guard breaker it saw coming.
    pub const DROPPED: Bits = 1 << 14;
}

/// The words of its own region of the lore, after its eyes.
pub mod word {
    use crate::species::mantis::sight::word::END as EYES;

    /// Each fighter's class plus one, four bits each: what it sees them
    /// carrying.
    pub const CLASSES: usize = EYES;
    /// Frames each fighter has stood outside its arc within `BackNear`, a
    /// half each: `back(m)`.
    pub const BACK: usize = EYES + 1;
    /// Frames since it last saw anybody commit to anything: `far(m)`.
    pub const QUIET: usize = EYES + 2;
    /// **A guard it means to raise**, on a move it saw: the frame to raise
    /// it on (low 24 bits), and a valid bit at the top.
    pub const PLAN: usize = EYES + 3;
    /// Where the dive will land, in centimetres: chosen as the leap leaves
    /// the floor, and drawn from then.
    pub const DIVE_AT: usize = EYES + 4;
    /// The memory: [`super::super::habit::DEPTH`] entries, then its meta.
    pub const HABIT: usize = EYES + 5;
    pub const HABIT_META: usize = HABIT + crate::species::mantis::habit::DEPTH;
    /// Frames its guard has had nothing seen coming at it.
    pub const GUARD_QUIET: usize = HABIT_META + 1;
    /// The frame its last coil began, and whether its release was seen.
    pub const COIL: usize = HABIT_META + 2;
    /// **The frame its guard was last committed**: raised, or a blow taken
    /// on it. Its minimum hold is counted from here.
    pub const GUARD_SINCE: usize = HABIT_META + 3;
    /// **The commitment it last decided about raising late for**: the stamp
    /// its deed began on (low 24 bits), and whether it chose to (the top
    /// bit). Decided once a commitment, so a string seen is one choice.
    pub const LATE: usize = HABIT_META + 4;
    /// The frame its guard went up, and the frame it last came down: the
    /// longest it holds it, and the rest it takes before raising it again.
    pub const GUARD_UP: usize = HABIT_META + 5;
    pub const GUARD_DOWN: usize = HABIT_META + 6;
    pub const END: usize = HABIT_META + 7;
}

/// A valid bit at the top of a word.
const VALID: u32 = 1 << 31;
/// The low twenty-four bits of a word: a frame.
const FRAME: u32 = 0x00FF_FFFF;

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

/// Haste pips left.
pub fn pips(m: &Monster) -> i32 {
    flags(m) & flag::PIPS
}

fn set_pips(m: &mut Monster, n: i32) {
    m.own[body::FLAGS] = (m.own[body::FLAGS] & !flag::PIPS) | n.clamp(0, flag::PIPS);
}

pub fn desperate(m: &Monster) -> bool {
    flags(m) & flag::DESPERATE != 0
}

pub fn hurt(m: &Monster) -> bool {
    flags(m) & flag::HURT != 0
}

pub fn coop(m: &Monster) -> bool {
    flags(m) & flag::COOP != 0
}

/// Why the counter under way was thrown: [`flag::BY_SIGHT`],
/// [`flag::BY_READY`], [`flag::BY_PRAYER`], or zero.
pub fn counter_cause(m: &Monster) -> i32 {
    (flags(m) & flag::CAUSE) >> flag::CAUSE_SHIFT
}

/// Both blades whole, one, or none: how many it can guard with.
pub fn blades(m: &Monster) -> i32 {
    !m.broken(BLADE_L_PART) as i32 + !m.broken(BLADE_R_PART) as i32
}

/// The class a fighter carries, as it sees it: written by the hook.
pub fn class_of(lore: &Lore, who: usize) -> Option<crate::Class> {
    let c = (lore.word(word::CLASSES) >> (who * 4)) & 0xF;
    if c == 0 {
        return None;
    }
    crate::class::ALL_CLASSES.get(c as usize - 1).copied()
}

/// Frames fighter `who` has stood outside its arc within `BackNear`.
pub fn back_frames(lore: &Lore, who: usize) -> i32 {
    let w = lore.word(word::BACK);
    (if who == 0 { lore::lo(w) } else { lore::hi(w) }) as i32
}

/// Frames since it saw a commitment.
pub fn quiet(lore: &Lore) -> u32 {
    lore.word(word::QUIET)
}

/// Where the dive will land: chosen as the leap leaves the floor.
pub fn dive_at(lore: &Lore) -> V3 {
    let w = lore.word(word::DIVE_AT);
    V3::new(
        lore::from_cm(lore::lo(w)),
        Fx::ZERO,
        lore::from_cm(lore::hi(w)),
    )
}

/// The frames the move under way has run: through its startup, then its
/// active window, then its recovery.
pub fn elapsed(m: &Monster) -> Option<(u8, i32)> {
    let kind = m.doing.attacking()?;
    let a = SPECIES.attack(kind);
    Some(match m.doing {
        Doing::Startup { left, .. } => (kind, a.startup as i32 - left as i32),
        Doing::Active { left, .. } => (kind, a.startup as i32 + a.active as i32 - left as i32),
        Doing::Recovery { left, .. } => (kind, a.total() as i32 - left as i32),
        _ => (kind, 0),
    })
}

pub fn flat(v: V3) -> V3 {
    V3::new(v.x, Fx::ZERO, v.z)
}

/// The cosine of an angle in turns.
pub fn cos_turns(t: Fx) -> Fx {
    V3::from_turns(t).x
}

/// Its yaw toward a point, or its own if the point is where it stands.
pub fn yaw_to(m: &Monster, at: V3) -> Fx {
    let to = flat(at.sub(m.pos));
    if to.flat_len().raw() <= 0 {
        return m.yaw;
    }
    math::atan2_turns(to.z, to.x)
}

// ---------------------------------------------------------------------------
// The guard
// ---------------------------------------------------------------------------

/// What it has up against a blow, if anything.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Up {
    /// Its guard: the front cone, parrying for its first frames.
    Guard { parry: bool },
    /// The prayer: a parry all round.
    Prayer,
    /// Ready: one blade cocked, for one move.
    Ready,
}

/// What it has up, now.
pub fn up(m: &Monster) -> Option<Up> {
    match m.doing {
        Doing::Active { kind: GUARD, left } => {
            let held = SPECIES.attack(GUARD).active.saturating_sub(left);
            Some(Up::Guard {
                parry: (held as i32) < Knob::ParryWindow.raw(),
            })
        }
        Doing::Active { kind: PRAYER, .. } => Some(Up::Prayer),
        Doing::Active { kind: READY, .. } => Some(Up::Ready),
        _ => None,
    }
}

/// **The guard's arc, after its blades**: where its middle points, off its
/// facing, and how far either side it reaches, in turns. Both blades: the
/// whole front. One: from `OneSideAcross` over the broken side to
/// `OneSideWhole` on the whole one -- the broken side is open for good. None:
/// no guard at all.
pub fn arc(m: &Monster) -> Option<(Fx, Fx)> {
    let left = m.broken(BLADE_L_PART);
    let right = m.broken(BLADE_R_PART);
    match (left, right) {
        (false, false) => Some((Fx::ZERO, Knob::GuardArc.fx())),
        (true, true) => None,
        _ => {
            let whole = Knob::OneSideWhole.fx();
            let across = Knob::OneSideAcross.fx();
            let half = math::half(whole.add(across));
            let centre = math::half(whole.sub(across));
            // Toward the whole side: its right is positive.
            let centre = if left { centre } else { centre.neg() };
            Some((centre, half))
        }
    }
}

/// **Is `from` inside its guard's cone?** The fighters' cone test
/// (`state::in_guard_arc`) in the floor plane from its middle, across its
/// arc; and an elevation, from its chest to the middle of a body standing
/// where the blow comes from -- up to `GuardElevation` above level, down to
/// `GuardBelow` under it. A blow from high above lands; so does one from
/// right under it (a pillar at its feet).
pub fn inside(m: &Monster, from: V3) -> bool {
    let Some((centre, half)) = arc(m) else {
        return false;
    };
    let at = flat(m.pos);
    let facing = V3::from_turns(m.yaw.add(centre));
    if !state::in_guard_arc(at, facing, flat(from), cos_turns(half)) {
        return false;
    }
    elevated_ok(m, from)
}

fn elevated_ok(m: &Monster, from: V3) -> bool {
    let run = math::wide_flat_dist(from, m.pos);
    // From under its body -- a pillar at its feet -- is from below it,
    // whatever the angle.
    if run.raw() < SPECIES.fight_fx(FightField::BodyRadius).raw() {
        return false;
    }
    let chest = m.pos.y.add(Knob::GuardChest.fx());
    let middle = from.y.add(math::half(crate::tuning::body_height()));
    let rise = middle.sub(chest);
    let angle = math::atan2_turns(rise, run);
    angle.raw() <= Knob::GuardElevation.fx().raw()
        && angle.raw() >= Knob::GuardBelow.fx().neg().raw()
}

/// [`FightDecl::covers`]: would a blockable blow from `from` be stopped now?
/// Its guard's cone, or its prayer all round. Ready is not a guard: it
/// stops one move, and anything else is a clean hit.
pub fn covers(m: &Monster, from: V3) -> bool {
    match up(m) {
        Some(Up::Guard { .. }) => inside(m, from),
        Some(Up::Prayer) => true,
        _ => false,
    }
}

/// **[`FightDecl::guard`]: what its guard makes of a blow.** See the module
/// docs; `mantis.md` §4.
pub fn guard(m: &mut Monster, _part: usize, blow: &Blow) -> Guarded {
    let Some(what) = up(m) else {
        return Guarded::Lands;
    };
    match what {
        Up::Ready => {
            let waited = m.own[body::READY_FOR];
            let front = state::in_guard_arc(
                flat(m.pos),
                V3::from_turns(m.yaw),
                flat(blow.from),
                Fx::ZERO,
            );
            if !blow.unblockable
                && waited != 0
                && waited == habit::code(blow.class, blow.kind)
                && front
            {
                remember(m, blow, Guarded::Parried);
                counter(m, blow.from, flag::BY_READY);
                return Guarded::Parried;
            }
            // Wrong-footed: a clean hit, and the guess is forgotten.
            set_flag(m, flag::WRONG, true);
            Guarded::Lands
        }
        Up::Prayer => {
            if blow.unblockable {
                remember(m, blow, Guarded::Broke);
                stagger(m, BROKEN, Knob::PrayerStagger.frames());
                return Guarded::Broke;
            }
            remember(m, blow, Guarded::Parried);
            counter(m, blow.from, flag::BY_PRAYER);
            Guarded::Parried
        }
        Up::Guard { parry } => {
            if !inside(m, blow.from) {
                return Guarded::Lands;
            }
            if blow.unblockable {
                remember(m, blow, Guarded::Broke);
                stagger(m, BROKEN, Knob::BreakStagger.frames());
                return Guarded::Broke;
            }
            if parry {
                remember(m, blow, Guarded::Parried);
                counter(m, blow.from, flag::BY_SIGHT);
                return Guarded::Parried;
            }
            remember(m, blow, Guarded::Blocked);
            // **No chip, no guard meter, no blade damage** (§4): pushed and
            // held, and nothing else.
            if let Doing::Active { kind: GUARD, left } = m.doing {
                m.doing = Doing::Active {
                    kind: GUARD,
                    left: left.max(Knob::GuardStun.frames()),
                };
            }
            let stun = Knob::GuardStun.raw().max(1);
            let back = Knob::GuardPush
                .fx()
                .mul(Fx::from_int(crate::TICK_HZ as i32))
                .div(Fx::from_int(stun));
            m.speed = back.neg();
            Guarded::Blocked
        }
    }
}

/// **The counter**: a straight thrust along the parried line, out of the
/// parry and only out of one. A branch of a move it had already committed
/// to, not a reaction: its twelve frames are below reaction on purpose, and
/// its answer is the choice before it (§2).
fn counter(m: &mut Monster, from: V3, cause: i32) {
    m.yaw = yaw_to(m, from);
    m.yaw_rate = Fx::ZERO;
    m.speed = Fx::ZERO;
    m.doing = Doing::Startup {
        kind: COUNTER,
        left: SPECIES.attack(COUNTER).startup,
    };
    m.hit_used = false;
    m.brain.last_move = COUNTER;
    m.brain.mirror = false;
    m.own[body::FLAGS] = (m.own[body::FLAGS] & !flag::CAUSE) | (cause << flag::CAUSE_SHIFT);
    m.own[body::READY_FOR] = 0;
}

/// A stagger timed as a move's recovery: the guard broken, a lunge into a
/// solid.
fn stagger(m: &mut Monster, kind: u8, frames: u16) {
    m.doing = Doing::Recovery { kind, left: frames };
    m.speed = Fx::ZERO;
    m.yaw_rate = Fx::ZERO;
    m.own[body::READY_FOR] = 0;
}

/// Into the mailbox: what struck its guard, by whom, and what the guard
/// made of it. The only place the memory is ever written from (§5).
fn remember(m: &mut Monster, blow: &Blow, result: Guarded) {
    let who = blow.who as usize;
    if who >= MAX_PLAYERS || blow.kind == Blow::NO_MOVE {
        return;
    }
    let r = match result {
        Guarded::Blocked => 1,
        Guarded::Parried => 2,
        _ => 3,
    };
    let half = (1u32 << 15) | (r << 12) | (habit::code(blow.class, blow.kind) as u32 & 0x0FFF);
    let shift = (who * 16) as u32;
    let w = (m.own[body::MAILBOX] as u32 & !(0xFFFF << shift)) | (half << shift);
    m.own[body::MAILBOX] = w as i32;
}

/// What struck its guard this frame from fighter `who`: the move's code and
/// what the guard made of it (1 blocked, 2 parried, 3 broke).
pub fn mail(m: &Monster, who: usize) -> Option<(i32, u32)> {
    let half = (m.own[body::MAILBOX] as u32 >> (who * 16)) & 0xFFFF;
    if half & (1 << 15) == 0 {
        return None;
    }
    Some(((half & 0x0FFF) as i32, (half >> 12) & 0b11))
}

// ---------------------------------------------------------------------------
// Hits on it, and its hits on you
// ---------------------------------------------------------------------------

/// **What its hide is worth now**: arms and blades at `BrokenArms` while its
/// guard is broken -- the blades flung wide and low are what the window
/// offers -- and everything at `ReadyWrong` while it is in a Ready that was
/// the wrong guess.
pub fn hide(m: &Monster, part: usize) -> Fx {
    match m.doing {
        Doing::Recovery { kind: BROKEN, .. }
            if matches!(part, ARM_L_PART | ARM_R_PART | BLADE_L_PART | BLADE_R_PART) =>
        {
            Knob::BrokenArms.fx()
        }
        Doing::Startup { kind: READY, .. } | Doing::Active { kind: READY, .. } => {
            Knob::ReadyWrong.fx()
        }
        _ => Fx::ONE,
    }
}

/// **A hit has landed on it.** In a stagger of its own -- its guard broken,
/// a lunge into a wall -- the stagger is the window, and a flinch or an
/// interrupt is not allowed to cut it short: its blades still wear (the
/// window's other reward), and a blade that goes is the stumble, as ever.
pub fn struck(m: &mut Monster, part: usize, dealt: i32) -> bool {
    if !matches!(
        m.doing,
        Doing::Recovery {
            kind: BROKEN | STAGGER,
            ..
        }
    ) {
        return false;
    }
    if let Some(slot) = SPECIES.break_slot(part).filter(|s| m.breaks[*s] > 0) {
        m.breaks[slot] -= dealt;
        if m.breaks[slot] <= 0 {
            m.breaks[slot] = 0;
            m.doing = Doing::Stumble {
                left: SPECIES.stumble_frames(),
                front: true,
            };
            m.speed = Fx::ZERO;
            m.yaw_rate = Fx::ZERO;
            m.strain = 0;
        }
    }
    true
}

/// **One of its moves landed**: the flare's shove is a slide for a braced
/// (crouched) fighter.
pub fn landed(w: &mut World, _slot: usize, fighter: usize, kind: u8, guarded: bool) {
    if kind != FLARE || guarded {
        return;
    }
    let p = &mut w.players[fighter];
    if p.crouching {
        let k = Knob::FlareBrace.fx();
        p.vel.x = p.vel.x.mul(k);
        p.vel.z = p.vel.z.mul(k);
    }
}

/// **It ran into a solid.** A lunge stops dead and staggers.
pub fn bumped(m: &mut Monster, _push: V3) {
    if matches!(m.doing, Doing::Active { kind: LUNGE, .. }) {
        stagger(m, STAGGER, Knob::LungeStagger.frames());
    }
}

// ---------------------------------------------------------------------------
// The frame hook
// ---------------------------------------------------------------------------

/// Its reflexes and its body, once a frame. See the module docs.
pub fn frame(w: &mut World) {
    sight::record(w);
    let t = w.frame;
    let mut lore = w.lore;
    // What it sees each fighter carrying.
    let mut classes = 0u32;
    for (who, p) in w.players.iter().enumerate() {
        classes |= (p.class as u32 + 1) << (who * 4);
    }
    lore.set_word(word::CLASSES, classes);
    let ground = w.terrain();
    let field = crate::stones::gather(&w.players);
    let hunters = w.players.iter().filter(|p| p.health > 0).count();
    for slot in 0..crate::monster::MAX_MONSTERS {
        let Some(mut m) = w.monsters[slot] else {
            continue;
        };
        if m.species != SPECIES.id || !m.alive() {
            continue;
        }
        stages(&mut m, hunters);
        // A blow its guard took last frame commits it again: blockstun is
        // a commitment, and a guard being hit is a guard being held.
        if (0..MAX_PLAYERS).any(|who| mail(&m, who).is_some_and(|(_, r)| r == 1 || r == 2)) {
            lore.set_word(word::GUARD_SINCE, t.wrapping_sub(1));
        }
        habit::drain(&mut m, &mut lore, t);
        watch(&m, &mut lore, t);
        reflexes(&mut m, &mut lore, t);
        let scene_players = w.players;
        let effects = w.effects;
        let critters = w.critters;
        let scene = crate::aim::Scene {
            stones: &field,
            players: &scene_players,
            effects: &effects,
            quarry: &[None; crate::monster::MAX_MONSTERS],
            critters: &critters,
            arena: &ground,
        };
        body_moves(&mut m, &mut lore, t, &scene);
        tempo(&mut m);
        w.monsters[slot] = Some(m);
    }
    w.lore = lore;
}

/// **Its stage** (§4): hurt below `HurtBelow`, desperate below
/// `DesperateBelow` or with both blades gone; and, on the first frame of a
/// hunt with two hunters, its two-hunter health.
fn stages(m: &mut Monster, hunters: usize) {
    if flags(m) & flag::SET_UP == 0 {
        set_flag(m, flag::SET_UP, true);
        if hunters >= 2 {
            set_flag(m, flag::COOP, true);
            m.health = Knob::CoopHealth.raw().max(1);
        }
        // **Its idle life is the prayer** (§11): the hunt opens with it at
        // prayer in the middle of the court, and its moment of noticing you
        // is the prayer's length. Break it, or wait for three hasted moves.
        if m.doing.free() {
            let a = SPECIES.attack(PRAYER);
            m.doing = Doing::Active {
                kind: PRAYER,
                left: a.active,
            };
            m.brain.last_move = PRAYER;
            m.brain.cooldown[PRAYER as usize] = a.cooldown;
        }
    }
    let full = if coop(m) {
        Knob::CoopHealth.raw()
    } else {
        SPECIES.health()
    }
    .max(1);
    let share = Fx::from_int(m.health).div(Fx::from_int(full));
    set_flag(m, flag::HURT, share.raw() < Knob::HurtBelow.fx().raw());
    set_flag(
        m,
        flag::DESPERATE,
        share.raw() < Knob::DesperateBelow.fx().raw() || blades(m) == 0,
    );
}

/// **What it keeps watching**: how long each fighter has stood outside its
/// arc within `BackNear` (`back(m)`), and how long since anybody committed
/// to anything (`far(m)`). All of it from what it sees, late.
fn watch(m: &Monster, lore: &mut Lore, t: u32) {
    let mut back = [0i32; MAX_PLAYERS];
    let mut committed = false;
    for (who, b) in back.iter_mut().enumerate() {
        let Some(g) = sight::glimpse(lore, who, t).filter(|g| g.alive) else {
            continue;
        };
        let near = math::wide_flat_dist(g.pos, m.pos).raw() <= Knob::BackNear.fx().raw();
        let (centre, half) = arc(m).unwrap_or((Fx::ZERO, Knob::GuardArc.fx()));
        let front = state::in_guard_arc(
            flat(m.pos),
            V3::from_turns(m.yaw.add(centre)),
            flat(g.pos),
            cos_turns(half),
        );
        if near && !front {
            *b = (back_frames(lore, who) + 1).min(i16::MAX as i32);
        }
        if sight::deed_seen(lore, who, t).0.commits() {
            committed = true;
        }
    }
    lore.set_word(word::BACK, lore::halves(back[0] as i16, back[1] as i16));
    let q = if committed {
        0
    } else {
        quiet(lore).saturating_add(1)
    };
    lore.set_word(word::QUIET, q);
}

/// May it guard at all, in its state: not desperate, a blade left.
pub fn may_guard(m: &Monster) -> bool {
    !desperate(m) && blades(m) > 0
}

/// **A move it saw coming at it**, from fighter `who`: the move, the frame
/// its hit comes out, and whether it reaches from where they were seen.
pub struct Coming {
    pub who: usize,
    /// The stamp its deed began on.
    pub began: u32,
    pub kind: u8,
    pub hit_at: u32,
    pub ends_at: u32,
    pub unblockable: bool,
    pub startup: u16,
}

/// What it sees coming at it, now: each fighter's seen deed, if it is a move
/// that reaches it from where it saw them -- its hit still to come, or
/// already come and gone and the next link of the string still to come.
pub fn coming(m: &Monster, lore: &Lore, t: u32) -> [Option<Coming>; MAX_PLAYERS] {
    std::array::from_fn(|who| {
        let (deed, began) = sight::deed_seen(lore, who, t);
        let Deed::Move(kind) = deed else {
            return None;
        };
        let class = class_of(lore, who)?;
        let g = sight::glimpse(lore, who, t)?;
        let mv = crate::moves::get(class, kind);
        let reach = mv
            .reach
            .add(mv.radius)
            .add(Knob::GuardReach.fx())
            .add(crate::tuning::body_radius());
        if math::wide_flat_dist(g.pos, m.pos).raw() > reach.raw() {
            return None;
        }
        let hit_at = began.wrapping_add(mv.startup as u32);
        let ends_at = hit_at.wrapping_add(mv.active as u32);
        // A move it sees is a commitment it sees, whether or not its hit
        // has already come and gone: a string's next link is coming.
        Some(Coming {
            who,
            began,
            kind,
            hit_at,
            ends_at,
            unblockable: mv.unblockable,
            startup: mv.startup,
        })
    })
}

/// Raise the guard now: with its parry, or -- raised too late for the hit
/// it saw, against whatever follows it -- already past it.
fn raise(m: &mut Monster, on_sight: bool, parry: bool) {
    let active = SPECIES.attack(GUARD).active;
    let past = if parry { 0 } else { Knob::ParryWindow.frames() };
    m.doing = Doing::Active {
        kind: GUARD,
        left: active.saturating_sub(past),
    };
    m.hit_used = false;
    m.brain.last_move = GUARD;
    m.speed = Fx::ZERO;
    m.yaw_rate = Fx::ZERO;
    set_flag(m, flag::SAW_IT, on_sight);
}

/// **Its reflexes**: the guard on a move it saw (inside the beat), the guard
/// lowered on a breaker it saw, the coil's release, the second slash.
fn reflexes(m: &mut Monster, lore: &mut Lore, t: u32) {
    let seen = coming(m, lore, t);
    let blockable = seen.iter().flatten().find(|c| !c.unblockable);
    let breaker = seen.iter().flatten().find(|c| c.unblockable);

    // ---- the guard, raised on what it saw ----
    let plan = lore.word(word::PLAN);
    // A guard just lowered is not raised again at once: it has to do
    // something else first, or stand there open.
    let rested = t.wrapping_sub(lore.word(word::GUARD_DOWN)) as i32 >= Knob::GuardRest.raw();
    if m.doing.free() && m.brain.grace == 0 && may_guard(m) && rested {
        let half = (Knob::ParryWindow.raw().max(1) / 2) as u32;
        match blockable {
            // **Seen before its hit comes out** -- a move whose startup is
            // longer than its eyes are late (§2): raised so the hit arrives
            // inside the parry window -- as late as that, or now if now is
            // already later.
            Some(c) if (c.hit_at.wrapping_sub(t) as i32) > 0 => {
                let at = c.hit_at.wrapping_sub(half);
                if t.wrapping_sub(at) < u32::MAX / 2 {
                    raise(m, true, true);
                    lore.set_word(word::PLAN, 0);
                    lore.set_word(word::GUARD_SINCE, t);
                } else {
                    lore.set_word(word::PLAN, VALID | (at & FRAME));
                }
            }
            // **Seen too late for its hit**: that one has landed or is
            // landing, and a parry of it is not on. Raised anyway, past its
            // parry, against what follows -- the next link of a string.
            // Most of the time: `LateGuard` of the commitments it sees too
            // late, chosen once each.
            Some(c) => {
                let late = lore.word(word::LATE);
                let raise_it = if late & FRAME == c.began & FRAME && late != 0 {
                    late & VALID != 0
                } else {
                    let yes = (super::mind::draw(m) % 100) < Knob::LateGuard.raw() as u32;
                    lore.set_word(word::LATE, (c.began & FRAME) | if yes { VALID } else { 0 });
                    yes
                };
                if raise_it {
                    raise(m, true, false);
                    lore.set_word(word::PLAN, 0);
                    lore.set_word(word::GUARD_SINCE, t);
                }
            }
            None if plan & VALID != 0
                && ((t & FRAME).wrapping_sub(plan & FRAME) & FRAME) < FRAME / 2 =>
            {
                raise(m, true, true);
                lore.set_word(word::PLAN, 0);
                lore.set_word(word::GUARD_SINCE, t);
            }
            None => {}
        }
    } else if !m.doing.free() {
        lore.set_word(word::PLAN, 0);
    }

    // ---- the guard, held while something comes, lowered on a breaker ----
    if let Doing::Active { kind: GUARD, left } = m.doing {
        // Its first frame, however it went up -- by its own choice, or on
        // something it saw: committed from now, and the clock on its
        // longest hold starts. (`GUARD_UP` is zero while the guard is down,
        // so frame zero is written as one.)
        if lore.word(word::GUARD_UP) == 0 {
            lore.set_word(word::GUARD_SINCE, t);
            lore.set_word(word::GUARD_UP, t.max(1));
        }
        let held = t.wrapping_sub(lore.word(word::GUARD_SINCE)) as i32;
        let mut quiet = lore.word(word::GUARD_QUIET);
        // Somebody at the edge of its reach, in front of it: it holds you
        // there with its guard up (§1), until its hold runs out.
        let held_off = (0..MAX_PLAYERS).any(|who| {
            sight::glimpse(lore, who, t).is_some_and(|g| {
                g.alive
                    && math::wide_flat_dist(g.pos, m.pos).raw() <= Knob::StandOff.fx().raw()
                    && inside(m, g.pos)
            })
        });
        let worn = t.wrapping_sub(lore.word(word::GUARD_UP)) as i32 >= Knob::GuardMost.raw();
        if blockable.is_some() && !worn {
            quiet = 0;
            // Something is still coming: it keeps it up.
            m.doing = Doing::Active {
                kind: GUARD,
                left: left.max(Knob::ParryWindow.frames()),
            };
        } else if held_off {
            quiet = 0;
        } else {
            quiet = quiet.saturating_add(1);
        }
        lore.set_word(word::GUARD_QUIET, quiet);
        let committed = held >= Knob::GuardMinHold.raw();
        let drop_for = breaker.is_some_and(|c| {
            let to_hit = c.hit_at.wrapping_sub(t) as i32;
            (0..=Knob::DropsFor.raw()).contains(&to_hit)
        });
        if committed
            && (drop_for || worn || quiet as i32 >= Knob::GuardQuiet.raw() || !may_guard(m))
        {
            m.doing = Doing::Recovery {
                kind: GUARD,
                left: SPECIES.attack(GUARD).recovery,
            };
            set_flag(m, flag::DROPPED, drop_for);
        }
    }
    match m.doing {
        Doing::Active { kind: GUARD, .. } => {}
        Doing::Recovery { kind: GUARD, left } => {
            if left == SPECIES.attack(GUARD).recovery {
                lore.set_word(word::GUARD_DOWN, t);
            }
            lore.set_word(word::GUARD_QUIET, 0);
            lore.set_word(word::GUARD_UP, 0);
        }
        _ => {
            lore.set_word(word::GUARD_QUIET, 0);
            lore.set_word(word::GUARD_UP, 0);
        }
    }

    // ---- the coil: released on a commitment it saw ----
    if let Doing::Startup { kind: LUNGE, left } = m.doing {
        let a = SPECIES.attack(LUNGE);
        let coiled = a.startup.saturating_sub(left) as i32;
        let target = m.brain.target as usize;
        let saw = sight::deed_seen(lore, target.min(MAX_PLAYERS - 1), t)
            .0
            .commits();
        if left > 0 && coiled >= Knob::CoilMin.raw() && saw {
            m.doing = Doing::Startup {
                kind: LUNGE,
                left: 0,
            };
            set_flag(m, flag::ON_SIGHT, true);
        }
        if coiled == 0 {
            set_flag(m, flag::ON_SIGHT, false);
            lore.set_word(word::COIL, t);
        }
    }

    // ---- the second slash: the fast one, or held ----
    if let Doing::Startup {
        kind: SLASH_FAST,
        left,
    } = m.doing
    {
        if left == SPECIES.attack(SLASH_FAST).startup {
            let held = desperate(m)
                || (crate::species::mantis::mind::draw(m) % 100) < Knob::HeldShare.raw() as u32;
            if held {
                m.doing = Doing::Startup {
                    kind: SLASH_HELD,
                    left: SPECIES.attack(SLASH_HELD).startup,
                };
                m.brain.last_move = SLASH_HELD;
            }
        }
    }
    // A broken blade: the pair is one slash. The second is not thrown.
    if blades(m) < 2 {
        if let Doing::Startup {
            kind: SLASH_FAST | SLASH_HELD,
            ..
        } = m.doing
        {
            m.doing = Doing::Prowl;
            m.brain.think_left = SPECIES.think_frames();
        }
    }
}

/// The lunge on its way: into a solid -- one in the lane as it ran -- it
/// stops dead and staggers. A lane with nothing in it runs its length.
fn lunge_runs(m: &mut Monster, scene: &crate::aim::Scene) {
    if aim_lane(m, scene) {
        let along = V3::from_turns(m.yaw);
        let left_to_go = m.aimed_at().sub(m.pos).dot(along);
        if left_to_go.raw() <= 0 {
            stagger(m, STAGGER, Knob::LungeStagger.frames());
        }
    }
}

/// **Its body this frame**: the lunge's release and its stop, the leap and
/// the dive, the flare's hop, walking and turning with its guard up.
fn body_moves(m: &mut Monster, lore: &mut Lore, t: u32, scene: &crate::aim::Scene) {
    // ---- the lunge: aimed as it is released, and its lane kept on the
    // first solid in it ----
    match m.doing {
        Doing::Startup { kind: LUNGE, left } => {
            if left == 0 {
                // Released: at where it saw you, and where it saw you going,
                // carried over its eyes and the eight frames of the lunge.
                let a = SPECIES.attack(LUNGE);
                let d = sight::delay(lore) as i32 + a.active as i32;
                let ahead = Fx::from_int(d).mul(DT).mul(Knob::LungeLead.fx());
                let at = m.brain.seen.add(flat(m.brain.seen_vel).scale(ahead));
                m.yaw = yaw_to(m, at);
                m.yaw_rate = Fx::ZERO;
            }
            aim_lane(m, scene);
        }
        Doing::Active { kind: LUNGE, .. } => lunge_runs(m, scene),
        _ => {}
    }

    // ---- the leap and the dive ----
    let height = Knob::LeapHeight.fx();
    match m.doing {
        Doing::Active { kind: LEAP, left } => {
            let a = SPECIES.attack(LEAP);
            if left == a.active {
                // Off the floor: where from, and where the dive will land --
                // where it sees you now, led to the landing. Drawn from now.
                m.own[body::LEAP_FROM] =
                    lore::halves(lore::to_cm(m.pos.x), lore::to_cm(m.pos.z)) as i32;
                let dive = SPECIES.attack(DIVE);
                let frames =
                    left as i32 + a.recovery as i32 + dive.startup as i32 + dive.active as i32;
                let ahead = Fx::from_int(frames).mul(DT).mul(m.lead());
                let at = m.brain.seen.add(flat(m.brain.seen_vel).scale(ahead));
                let to = flat(at.sub(m.pos));
                let far = math::wide_flat_len(to).min(dive.ideal_range.add(dive.range_span));
                let dir = if to.flat_len().raw() > 0 {
                    math::wide_normalized(to)
                } else {
                    V3::from_turns(m.yaw)
                };
                let land = m.pos.add(dir.scale(far));
                lore.set_word(
                    word::DIVE_AT,
                    lore::halves(lore::to_cm(land.x), lore::to_cm(land.z)),
                );
                m.aim_at(land);
            }
            let u = Fx::from_int(a.active as i32 - left as i32 + 1)
                .div(Fx::from_int(a.active.max(1) as i32));
            m.pos.y = height.mul(math::ease_out(u.clamp(Fx::ZERO, Fx::ONE)));
            m.speed = Fx::ZERO;
        }
        Doing::Recovery { kind: LEAP, .. } | Doing::Startup { kind: DIVE, .. } => {
            m.pos.y = height;
            m.speed = Fx::ZERO;
            m.aim_at(dive_at(lore));
        }
        Doing::Active { kind: DIVE, left } => {
            let a = SPECIES.attack(DIVE);
            let from = V3::new(
                lore::from_cm(lore::lo(m.own[body::LEAP_FROM] as u32)),
                height,
                lore::from_cm(lore::hi(m.own[body::LEAP_FROM] as u32)),
            );
            let to = dive_at(lore);
            m.aim_at(to);
            let u = Fx::from_int(a.active as i32 - left as i32 + 1)
                .div(Fx::from_int(a.active.max(1) as i32));
            let at = math::lerp3(from, to, u.clamp(Fx::ZERO, Fx::ONE));
            m.pos = at;
            m.yaw = yaw_to(m, to).add(Fx::ZERO);
            m.speed = Fx::ZERO;
        }
        _ => {
            m.pos.y = Fx::ZERO;
        }
    }

    // ---- the flare's hop back ----
    if let Doing::Recovery { kind: FLARE, left } = m.doing {
        let a = SPECIES.attack(FLARE);
        let gone = a.recovery.saturating_sub(left) as i32;
        let frames = Knob::FlareHopFrames.raw().max(1);
        if gone < frames {
            let step = Knob::FlareHop.fx().div(Fx::from_int(frames));
            let back = V3::from_turns(m.yaw).scale(step.neg());
            step_to(m, m.pos.add(back), scene);
        }
    }

    // ---- with its guard up: turning to you, walking at you ----
    if matches!(
        m.doing,
        Doing::Active { kind: GUARD, .. } | Doing::Active { kind: READY, .. }
    ) {
        let seen = m.brain.seen;
        // One blade: it turns its good side to you.
        let offset = arc(m).map_or(Fx::ZERO, |(c, _)| c);
        let want = yaw_to(m, seen).sub(offset);
        let error = math::wrap_turns(want.sub(m.yaw));
        let most = Knob::GuardTurn.fx().mul(DT);
        m.yaw = m.yaw.add(error.clamp(most.neg(), most));
        m.yaw_rate = Fx::ZERO;
        if let Doing::Active { kind: GUARD, .. } = m.doing {
            let to = flat(seen.sub(m.pos));
            let range = math::wide_flat_len(to);
            let gap = range.sub(Knob::StandOff.fx());
            let most = Knob::GuardWalk.fx().mul(DT);
            let walk = gap.clamp(most.neg(), most);
            if to.flat_len().raw() > 0 && walk.raw() != 0 && m.speed.raw() >= 0 {
                let next = m.pos.add(math::wide_normalized(to).scale(walk));
                step_to(m, next, scene);
                let covered = walk.abs().div(SPECIES.gait_stride());
                m.stride = m.stride.wrapping_add(covered.raw().clamp(0, 65535) as u16);
            }
        }
    }
    let _ = t;
}

/// Move its body to `next`, kept out of the arena's solids and inside its
/// bounds.
fn step_to(m: &mut Monster, next: V3, scene: &crate::aim::Scene) {
    let radius = SPECIES.fight_fx(FightField::BodyRadius);
    let step = SPECIES.fight_fx(FightField::StepOver);
    let (at, _) = scene.arena.fence(next, radius, step);
    let b = scene.arena.bounds;
    let margin = SPECIES.margin();
    m.pos = V3::new(
        at.x.clamp(b.lo_x.add(margin), b.hi_x.sub(margin)),
        m.pos.y,
        at.z.clamp(b.lo_z.add(margin), b.hi_z.sub(margin)),
    );
}

/// **The lunge's lane stops at the first solid in it** -- a wall, a column,
/// a stone, a planted shield -- through `aim::first_along`, the aiming
/// model's question. Its aim point is kept there, so the lane drawn on the
/// floor is the lane it will run (`MoveDecl::stops_at_aim`).
///
/// True when the lane ends at a solid rather than at its length.
fn aim_lane(m: &mut Monster, scene: &crate::aim::Scene) -> bool {
    let along = V3::from_turns(m.yaw);
    let chest = Knob::GuardChest.fx();
    let from = V3::new(m.pos.x, chest, m.pos.z);
    let radius = SPECIES.fight_fx(FightField::BodyRadius);
    let most = Knob::LungeTravel.fx();
    let to = from.add(along.scale(most.add(radius)));
    let hit = crate::aim::first_along(
        crate::aim::Path { from, to },
        radius,
        state::NOBODY,
        scene,
        crate::aim::Targets::none().stones().terrain(),
    );
    let stop = match hit {
        Some(c) => c.dist().sub(radius).max(Fx::ZERO),
        None => most,
    };
    m.aim_at(flat(m.pos).add(along.scale(stop)));
    hit.is_some()
}

/// **Its tempo**: the prayer's haste (three moves with shorter startups and
/// halved recoveries, and no beat between) and the beat between moves --
/// halved when desperate, shortened with two hunters or without its memory.
fn tempo(m: &mut Monster) {
    // The prayer, finished: its haste.
    if let Doing::Recovery { kind: PRAYER, left } = m.doing {
        if left == SPECIES.attack(PRAYER).recovery {
            set_pips(m, Knob::HastePips.raw());
        }
    }
    // A hasted move's recovery, halved on its first frame.
    if flags(m) & flag::HASTED != 0 {
        if let Doing::Recovery { kind, left } = m.doing {
            if left == SPECIES.attack(kind).recovery && !stance(kind) {
                let half = Fx::from_int(left as i32)
                    .mul(Knob::HasteRecovery.fx())
                    .to_int()
                    .max(0) as u16;
                m.doing = Doing::Recovery { kind, left: half };
            }
        }
    }
    let free = m.doing.free();
    if free && flags(m) & flag::WAS_FREE == 0 {
        // The beat it has just begun, scaled once.
        let mut beat = Fx::from_int(m.brain.think_left as i32);
        if flags(m) & flag::HASTED != 0 {
            beat = Fx::ZERO;
            set_flag(m, flag::HASTED, false);
        }
        if desperate(m) {
            beat = beat.mul(Knob::DesperateThink.fx());
        }
        if coop(m) {
            beat = beat.mul(Knob::CoopThink.fx());
        }
        if Knob::Habit.raw() == 0 {
            beat = beat.mul(Knob::GuessThink.fx());
        }
        m.brain.think_left = beat.to_int().max(0) as u16;
    }
    set_flag(m, flag::WAS_FREE, free);
    if !free {
        // Not a stance: a Ready guessed wrong, done, is forgotten in
        // `habit::drain`; a Ready over is no Ready.
        if !matches!(m.doing.attacking(), Some(READY)) {
            m.own[body::READY_FOR] = 0;
        }
    }
}

/// Haste a move as it commits: its startup by `HasteStartup`, never under
/// `HasteFloor`. Called by the brain's commit hook.
pub fn haste(m: &mut Monster, kind: u8) {
    if pips(m) <= 0 || stance(kind) || SPECIES.attack(kind).damage <= 0 {
        return;
    }
    if let Doing::Startup { kind: k, left } = m.doing {
        if k == kind {
            let floor = Knob::HasteFloor.raw().max(0);
            let short = Fx::from_int(left as i32)
                .mul(Knob::HasteStartup.fx())
                .to_int()
                .max(floor)
                .min(left as i32) as u16;
            m.doing = Doing::Startup { kind, left: short };
            set_pips(m, pips(m) - 1);
            set_flag(m, flag::HASTED, true);
        }
    }
}

// ---------------------------------------------------------------------------
// What it draws
// ---------------------------------------------------------------------------

/// **What it draws besides its telegraph**: the dive's circle from the frame
/// the leap leaves the floor; the memory, as notches along its blades, lit
/// for the move a Ready waits for; the prayer's haste, as pips on its
/// thorax. See §6.
pub fn marks(w: &World, out: &mut Marks) {
    for m in w.monsters.iter().flatten() {
        if m.species != SPECIES.id || !m.alive() {
            continue;
        }
        if let Some((LEAP, _)) | Some((DIVE, _)) = elapsed(m) {
            let leaving = matches!(m.doing, Doing::Active { kind: LEAP, .. })
                || matches!(m.doing, Doing::Recovery { kind: LEAP, .. })
                || matches!(m.doing, Doing::Startup { kind: DIVE, .. });
            if leaving {
                let a = SPECIES.attack(DIVE);
                out.push(Mark {
                    at: dive_at(&w.lore),
                    radius: a.hit_radius,
                    height: Fx::ZERO,
                    look: MarkLook::Warning,
                    progress: leap_progress(m),
                });
            }
        }
        habit::notches(m, &w.lore, out);
        // The haste: a glowing pip on the thorax for each move it has left,
        // down its front.
        let rig = m.rig();
        let sh = SPECIES.shape(super::THORAX_PART);
        let tall = sh.max.y.sub(sh.min.y);
        let most = Knob::HastePips.raw().max(1);
        for i in 0..pips(m) {
            let y = sh
                .max
                .y
                .sub(tall.mul(Fx::from_int(i + 1)).div(Fx::from_int(most + 1)));
            let at = rig.part_to_world(super::THORAX_PART, V3::new(sh.max.x, y, Fx::ZERO));
            out.push(Mark {
                at,
                radius: Knob::PipSize.fx(),
                height: Knob::PipSize.fx(),
                look: MarkLook::Embers,
                progress: Fx::ONE,
            });
        }
    }
}

/// How far from the floor to the dive's landing, nought to one.
fn leap_progress(m: &Monster) -> Fx {
    let leap = SPECIES.attack(LEAP);
    let dive = SPECIES.attack(DIVE);
    let span = (leap.active + leap.recovery + dive.startup + dive.active).max(1) as i32;
    let gone = match m.doing {
        Doing::Active { kind: LEAP, left } => (leap.active - left) as i32,
        Doing::Recovery { kind: LEAP, left } => (leap.active + leap.recovery - left) as i32,
        Doing::Startup { kind: DIVE, .. } => (leap.active + leap.recovery) as i32,
        _ => 0,
    };
    Fx::from_int(gone).div(Fx::from_int(span))
}

/// **Its guard, on the floor**: a fan of faint strips across the arc it
/// covers, from its body out to its reach, while the guard is up -- the thing
/// a player must walk round, drawn, because an arc that is only in the code
/// is a guard nobody can go round (§6). A ring all round for the prayer.
pub fn signs(w: &World, out: &mut crate::sign::Signs) {
    use crate::sign::{Says, Sign};
    for m in w.monsters.iter().flatten() {
        if m.species != SPECIES.id || !m.alive() {
            continue;
        }
        let reach = SPECIES.attack(SLASH).ideal_range;
        match up(m) {
            Some(Up::Guard { .. }) => {
                let Some((centre, half)) = arc(m) else {
                    continue;
                };
                // A strip down the middle of each of `FAN` equal sectors.
                let rays = FAN as i32;
                for i in 0..rays {
                    let u = Fx::from_int(i + i + 1).div(Fx::from_int(rays + rays));
                    let off = centre.sub(half).add(half.add(half).mul(u));
                    let along = V3::from_turns(m.yaw.add(off));
                    out.push(Sign::strip(
                        Says::No,
                        flat(m.pos),
                        along,
                        reach,
                        reach.div(Fx::from_int(FAN as i32)),
                    ));
                }
            }
            Some(Up::Prayer) => {
                out.push(Sign::ring(Says::No, flat(m.pos), reach.add(reach)));
            }
            _ => {}
        }
    }
}

/// Strips in the drawn fan of its guard.
const FAN: usize = 7;
