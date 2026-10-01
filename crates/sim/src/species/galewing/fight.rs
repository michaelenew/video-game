//! What the Galewing does to the world and what the world does to it: its
//! wings and the crash, the carry, the Downwash and its lee, the volley's
//! rake, the Screech and the buffet, wind and the perch, and the sky ride's
//! cycle -- everything the shared machinery calls it by. How it *flies* is
//! [`super::flight`]. See `docs/design/creatures/galewing.md`.
//!
//! **Its frame hook** ([`frame`]) runs once a frame after the creature has
//! stepped. The shared step has glanced, ticked its frame data, chosen a move
//! and walked it; the hook then flies the body from its own flight state in
//! the lore (`flight::step`), throwing away whatever the shared walk did to a
//! body that is in the air, and tests the moves whose hits are its own
//! against the same shapes its signs draw.
//!
//! **What the hit path and the rig read lives on the body** (`Monster::own`),
//! because the struck, hide, clip and repose hooks are handed the creature
//! and nothing else: the two wings' break bars, the carry's leg damage, the
//! bank, pitch and heave the pose is put in, and the posture flags.

use crate::DT;
use crate::aim::{self, Scene};
use crate::fixed::Fx;
use crate::lore::{self, Layout, Lore};
use crate::math::{self, V3};
use crate::monster::{Doing, MAX_MONSTERS, Monster, mount_slot};
use crate::sign::{Says, Sign, Signs};
use crate::species::{FightDecl, SpeciesId};
use crate::state::{self, Hit, MAX_PLAYERS, QUARRY, World};
use crate::tuning as t;

use super::flight;
use super::{
    BUFFET, CARRY, DOWNWASH, HOP, Knob, LIFT, PERCH, ROLL, SCREECH, SPECIES, STOOP, TALON, VOLLEY,
    bones, is_leg, wing_of,
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
    commit: Some(super::mind::commit),
    hide: Some(hide),
    struck: Some(struck),
    signs: Some(signs),
    clip: Some(flight::clip),
    presence: Some(presence),
    repose: Some(flight::repose),
    lob_height: Some(lob_height),
    glance: Some(glance),
    ..FightDecl::PLAIN
};

/// Its cells of the hunt's lore: see [`word`].
pub const OWN_CELLS: u8 = 12;

// ---------------------------------------------------------------------------
// Its state
// ---------------------------------------------------------------------------

/// Its words in the hunt's lore (`Lore::word`).
pub mod word {
    /// **The flight state** (`flight::Flight`): position (three raw fixed
    /// point words), horizontal speed, vertical speed, heading, turn rate.
    pub const POS: usize = 0;
    pub const SPEED: usize = 3;
    pub const VY: usize = 4;
    pub const YAW: usize = 5;
    pub const YAW_RATE: usize = 6;
    /// Bit 0: set up. Bit 1: two hunters. Bit 2: aloft (the body is
    /// flying, not standing). Bit 3: perched. Bit 4: it has stooped twice
    /// (the second of the pair is spent).
    pub const FLAGS: usize = 7;
    /// Wind, raw fixed point.
    pub const WIND: usize = 8;
    /// Frames of rest left on the perch.
    pub const REST: usize = 9;
    /// Frames it has stood on the floor since it became free there.
    pub const DWELL: usize = 10;
    /// The carried fighter plus one (low byte; zero is nobody), and the
    /// frame the catch was made (high half, the frame's low sixteen bits).
    pub const CARRIED: usize = 11;
    /// The talon lane: where it starts (cm), and its heading (raw turns).
    pub const LANE_AT: usize = 12;
    pub const LANE_YAW: usize = 13;
    /// The Downwash: the point under it (cm).
    pub const WASH_AT: usize = 14;
    /// The volley's lane: where it starts (cm), its heading (raw turns).
    pub const RAKE_AT: usize = 15;
    pub const RAKE_YAW: usize = 16;
    /// Feathers each fighter has taken this volley (a byte each) and the
    /// frame each last took one (high half: hunter 0's low byte, hunter
    /// 1's high byte -- frames since, capped).
    pub const RAKE_HITS: usize = 17;
    /// Who the Screech and the buffet have reached this move: a bit each.
    pub const STRUCK: usize = 18;
    /// **The ride**: the lap (low byte), the phase ([`ride`], byte 1), and
    /// frames in the phase (high half).
    pub const RIDE: usize = 19;
    /// Frames of the clipped glide left.
    pub const GLIDE: usize = 20;
    /// What the frame hook saw, per hunter, for the brain ([`seen`] bits):
    /// a byte each.
    pub const SEEN: usize = 21;
    /// The frame the hook last ran on.
    pub const NOW: usize = 22;
    /// The height it fell from when a crash began (raw fixed point), for
    /// the riders' share of the fall.
    pub const FELL_FROM: usize = 23;
    /// **Report counters**, a byte each: crashes from poise on a pass, from
    /// poise after a Stoop, from a clip, from a wing breaking.
    pub const CRASHES: usize = 24;
    /// Rides that ended: stepped off at the swoop, thrown, rode a wing
    /// down, any other way.
    pub const RIDES: usize = 25;
    /// Carries (low half) and carries freed early (high half).
    pub const CARRIES: usize = 26;
    /// Low passes (talon passes and Stoops thrown, low half) and perches
    /// (high half).
    pub const PASSES: usize = 27;
    /// The stoop's dive: where it began (cm) and its height (raw).
    pub const DIVE_AT: usize = 28;
    pub const DIVE_UP: usize = 29;
    /// The volley's and the Downwash's own aim heights, unused for now.
    pub const SPARE: usize = 30;
    /// The heave of the back's last wingbeat (raw metres) -- the integral of
    /// the beat, kept so the pose is a function of the body.
    pub const HEAVE_V: usize = 31;
    /// The bird's own clock, in frames since the hunt began.
    pub const CLOCK: usize = 32;
    /// Its health at the end of last frame: what a hit on the perch is
    /// measured by.
    pub const LAST_HEALTH: usize = 33;
    /// Bit 0: it was crashed last frame. Bit 1: it was aloft.
    pub const WAS: usize = 34;
    /// The talon lane's pivot: the point the target stood at, the lane
    /// running `LaneLead` before it (cm).
    pub const LANE_PIVOT: usize = 35;
    /// The middle of the circle it is flying now (cm): drifting toward the
    /// one over its target (`flight::drift`).
    pub const CIRCLE_AT: usize = 36;
    /// The air move this approach is for, plus one (`intend`).
    pub const INTENT: usize = 37;
}

/// The phases of the ride ([`word::RIDE`]).
pub mod ride {
    /// Nobody aboard.
    pub const NONE: u8 = 0;
    /// Climbing and lapping, wings beating.
    pub const CLIMB: u8 = 1;
    /// The roll is under way.
    pub const ROLL: u8 = 2;
    /// Down to swoop height over the far side.
    pub const SWOOP: u8 = 3;
}

/// The bits of a hunter's [`word::SEEN`] byte.
pub mod seen {
    /// Standing where the Downwash would push them: no solid between them
    /// and the point under the bird.
    pub const EXPOSED: u32 = 1;
    /// Near an edge, or on the tower top: somewhere a push costs a fall.
    pub const EDGE: u32 = 2;
    /// On the tower top.
    pub const TOWER: u32 = 4;
}

/// What lives on the body (`Monster::own`), for the hooks handed only the
/// creature.
pub mod body {
    /// The two wings' break bars: left in the low half, right in the high.
    pub const BARS: usize = 0;
    /// The carry's leg damage (low half) and leg hits (byte 2).
    pub const LEGS: usize = 1;
    /// The pitch (low half, 1/65536 turn) and the wingbeat's heave (high
    /// half, millimetres) the pose is put in.
    pub const PITCH: usize = 2;
    /// Posture flags (low half, [`super::flag`]) and the bank (high half,
    /// 1/65536 turn).
    pub const FLAGS: usize = 3;
}

/// The posture flags on the body ([`body::FLAGS`]'s low half).
pub mod flag {
    /// The bars are filled: the first frame has run.
    pub const SET_UP: u32 = 1;
    /// In the air: flying, not standing.
    pub const ALOFT: u32 = 2;
    /// Its body is below `LowBelow` over the ground: wing hits fill poise.
    pub const LOW: u32 = 4;
    /// Somebody is aboard and it is flying the ride.
    pub const RIDING: u32 = 8;
    /// Standing on the perch.
    pub const PERCHED: u32 = 16;
    /// Two hunters.
    pub const COOP: u32 = 32;
    /// A wing broke this frame (the struck hook says so; the frame hook
    /// reads it, counts it, and crashes a bird that was carrying a rider).
    pub const BROKE: u32 = 64;
}

// ---------------------------------------------------------------------------
// The body's words
// ---------------------------------------------------------------------------

pub fn flags(m: &Monster) -> u32 {
    (m.own[body::FLAGS] & 0xFFFF) as u32
}

pub fn set_flag(m: &mut Monster, f: u32, on: bool) {
    let rest = m.own[body::FLAGS] & !0xFFFF;
    let low = if on { flags(m) | f } else { flags(m) & !f };
    m.own[body::FLAGS] = rest | (low & 0xFFFF) as i32;
}

/// The bank the pose is put in, in turns: positive is its right wing down.
pub fn bank(m: &Monster) -> Fx {
    Fx::from_raw(((m.own[body::FLAGS] >> 16) as i16) as i32)
}

pub fn set_bank(m: &mut Monster, b: Fx) {
    let raw = b.raw().clamp(i16::MIN as i32, i16::MAX as i32) as i16 as u16 as i32;
    m.own[body::FLAGS] = (m.own[body::FLAGS] & 0xFFFF) | (raw << 16);
}

/// The pitch the pose is put in, in turns: positive is nose up.
pub fn pitch(m: &Monster) -> Fx {
    Fx::from_raw((m.own[body::PITCH] as i16) as i32)
}

pub fn set_pitch(m: &mut Monster, p: Fx) {
    m.own[body::PITCH] = p.raw().clamp(i16::MIN as i32, i16::MAX as i32);
}

/// What is left of a wing's break bar: `side` 0 is the left.
pub fn bar(m: &Monster, side: usize) -> i32 {
    let w = m.own[body::BARS];
    if side == 0 {
        (w as i16) as i32
    } else {
        ((w >> 16) as i16) as i32
    }
}

fn set_bar(m: &mut Monster, side: usize, v: i32) {
    let v = v.clamp(0, i16::MAX as i32) as i16 as u16 as i32;
    let w = m.own[body::BARS];
    m.own[body::BARS] = if side == 0 {
        (w & !0xFFFF) | v
    } else {
        (w & 0xFFFF) | (v << 16)
    };
}

/// Is that wing broken? Only once its bar has been filled at the start.
pub fn broken(m: &Monster, side: usize) -> bool {
    flags(m) & flag::SET_UP != 0 && bar(m, side) <= 0
}

/// How many wings are broken.
pub fn broken_wings(m: &Monster) -> u32 {
    broken(m, 0) as u32 + broken(m, 1) as u32
}

/// **Grounded for good**: both wings broken. It never leaves the floor again.
pub fn grounded_for_good(m: &Monster) -> bool {
    broken(m, 0) && broken(m, 1)
}

/// Is it in the air?
pub fn aloft(m: &Monster) -> bool {
    flags(m) & flag::ALOFT != 0
}

pub fn perched(m: &Monster) -> bool {
    flags(m) & flag::PERCHED != 0
}

pub fn riding(m: &Monster) -> bool {
    flags(m) & flag::RIDING != 0
}

/// Is it in the crash: on its breast, wings on the floor?
pub fn crashed(m: &Monster) -> bool {
    matches!(m.doing, Doing::Toppled { .. }) && !aloft(m)
}

/// Below `health` of its pool, as a share.
pub fn below(m: &Monster, share: Fx) -> bool {
    let max = m.sp().health().max(1);
    let max = if flags(m) & flag::COOP != 0 {
        Knob::CoopHealth.raw().max(max)
    } else {
        max
    };
    Fx::ratio(m.health.max(0), max).raw() < share.raw()
}

// ---------------------------------------------------------------------------
// Small helpers over the lore
// ---------------------------------------------------------------------------

pub fn word_of(p: V3) -> u32 {
    lore::halves(lore::to_cm(p.x), lore::to_cm(p.z))
}

pub fn point(w: u32) -> V3 {
    V3::new(
        lore::from_cm(lore::lo(w)),
        Fx::ZERO,
        lore::from_cm(lore::hi(w)),
    )
}

pub fn fx_word(lore: &Lore, i: usize) -> Fx {
    Fx::from_raw(lore.int(i))
}

pub fn set_fx(lore: &mut Lore, i: usize, v: Fx) {
    lore.set_int(i, v.raw());
}

fn bump_byte(lore: &mut Lore, i: usize, byte: usize) {
    let w = lore.word(i);
    let shift = byte * 8;
    let b = ((w >> shift) & 0xFF).saturating_add(1).min(0xFF);
    lore.set_word(i, (w & !(0xFF << shift)) | (b << shift));
}

fn bump_half(lore: &mut Lore, i: usize, high: bool) {
    let w = lore.word(i);
    let (lo, hi) = (w & 0xFFFF, w >> 16);
    let (lo, hi) = if high {
        (lo, (hi + 1).min(0xFFFF))
    } else {
        ((lo + 1).min(0xFFFF), hi)
    };
    lore.set_word(i, lo | (hi << 16));
}

/// A byte of a counter word.
pub fn byte_of(lore: &Lore, i: usize, byte: usize) -> u32 {
    (lore.word(i) >> (byte * 8)) & 0xFF
}

/// The ride's lap, phase and frames in the phase.
pub fn ride_state(lore: &Lore) -> (u8, u8, u16) {
    let w = lore.word(word::RIDE);
    ((w & 0xFF) as u8, ((w >> 8) & 0xFF) as u8, (w >> 16) as u16)
}

pub fn set_ride(lore: &mut Lore, lap: u8, phase: u8, frames: u16) {
    lore.set_word(
        word::RIDE,
        lap as u32 | ((phase as u32) << 8) | ((frames as u32) << 16),
    );
}

/// Who it is carrying, if anybody.
pub fn carried(lore: &Lore) -> Option<usize> {
    let w = lore.word(word::CARRIED) & 0xFF;
    (w > 0).then(|| (w - 1) as usize)
}

/// Its wind, nought to `WindMax`.
pub fn wind(lore: &Lore) -> Fx {
    fx_word(lore, word::WIND)
}

/// What the frame hook saw of hunter `i`.
pub fn seen_of(lore: &Lore, i: usize) -> u32 {
    (lore.word(word::SEEN) >> (8 * i.min(3))) & 0xFF
}

/// The bird's slot in a world, if it is in one.
pub fn slot_of(w: &World) -> Option<usize> {
    (0..MAX_MONSTERS).find(|s| w.monsters[*s].is_some_and(|m| m.species == SpeciesId::GALEWING))
}

// ---------------------------------------------------------------------------
// Where things are in the arena
// ---------------------------------------------------------------------------

/// **The middle of its circle**: the arena's `circle` site, or the middle
/// of the arena.
pub fn circle_centre(w: &World) -> V3 {
    let a = w.arena();
    if let Some(s) = a.sites.iter().find(|s| s.name == "circle") {
        let (x, z) = s.route[0];
        return V3::new(crate::arena::cm(x), Fx::ZERO, crate::arena::cm(z));
    }
    let b = a.bounds;
    V3::new(
        math::half(b.lo_x.add(b.hi_x)),
        Fx::ZERO,
        math::half(b.lo_z.add(b.hi_z)),
    )
}

/// **Its perch**: the top of the arena's `perch` site, if it has one.
pub fn perch_top(w: &World) -> Option<V3> {
    let a = w.arena();
    let s = a.sites.iter().find(|s| s.name == "perch")?;
    let (x, z) = s.route[0];
    let at = V3::new(crate::arena::cm(x), Fx::ZERO, crate::arena::cm(z));
    Some(V3::new(at.x, ground_at(w, at), at.z))
}

/// [`base`], from the ground alone: what the brain may read. The arena's
/// `circle` site says it in its height; with no such site, the ground at the
/// middle of the arena.
pub fn base_of(ground: &crate::arena::Terrain) -> Fx {
    let a: &crate::arena::Arena = ground;
    if let Some(s) = a.sites.iter().find(|s| s.name == "circle") {
        return crate::arena::cm(s.size.2);
    }
    let b = a.bounds;
    let c = V3::new(
        math::half(b.lo_x.add(b.hi_x)),
        Fx::ZERO,
        math::half(b.lo_z.add(b.hi_z)),
    );
    ground.ground_under(c)
}

/// **The level it measures its heights from**: the plateau, in the Cliffs --
/// the floor the fight is on, not the tower it circles.
pub fn base(w: &World) -> Fx {
    base_of(&w.terrain())
}

/// The ground under a point: the top of whatever stands under it, however
/// high the point is.
pub fn ground_at(w: &World, at: V3) -> Fx {
    w.terrain().ground_under(V3::new(at.x, Fx::ZERO, at.z))
}

// ---------------------------------------------------------------------------
// The frame
// ---------------------------------------------------------------------------

/// **Once a frame**, after the creature has stepped.
pub fn frame(w: &mut World) {
    let Some(slot) = slot_of(w) else {
        return;
    };
    setup(w, slot);
    let mut m = w.monsters[slot].expect("found above");
    let now = w.frame;
    let turned_by_step = m.yaw;
    let clock = w.lore.word(word::CLOCK).wrapping_add(1);
    w.lore.set_word(word::CLOCK, clock);

    sense(w, &m);
    if m.alive() {
        trouble(w, &mut m);
        committed(w, &mut m);
        track(w, &mut m);
        transitions(w, &mut m, slot);
    }
    flight::step(w, &mut m, slot);
    if m.alive() {
        talons(w, &mut m, slot);
        carry(w, &mut m, slot);
        downwash(w, &m);
        volley(w, &mut m);
        screech(w, &mut m);
        buffet(w, &mut m);
        wind_and_rest(w, &mut m);
    } else {
        drop_carried(w, &m);
    }
    // **A rider's look turns with the bird**: the shared spin saw only the
    // turn made inside the step, and this hook has just set the heading.
    let turned = math::wrap_turns(m.yaw.sub(turned_by_step));
    if turned.raw() != 0 {
        for p in w.players.iter_mut() {
            if p.aboard() && mount_slot(p.mount) == slot {
                p.carry_yaw = math::wrap_turns(p.carry_yaw.add(turned));
            }
        }
    }
    // The rest, shortened by what it took on the perch.
    let last = w.lore.int(word::LAST_HEALTH);
    if perched(&m) && last > m.health {
        shorten_rest(&mut w.lore, last - m.health);
    }
    w.lore.set_int(word::LAST_HEALTH, m.health);
    let was = (crashed(&m) as u32)
        | ((aloft(&m) as u32) << 1)
        | (matches!(m.doing, Doing::Toppled { .. }) as u32) << 2;
    w.lore.set_word(word::WAS, was);
    w.lore.set_word(word::NOW, now);
    w.monsters[slot] = Some(m);
}

/// **Trouble in the air**: a wing breaking, a piece of crowd control it
/// took, a crash beginning -- and the counting of each.
fn trouble(w: &mut World, m: &mut Monster) {
    let was = w.lore.word(word::WAS);
    let was_toppled = was & 4 != 0;
    if flags(m) & flag::BROKE != 0 {
        set_flag(m, flag::BROKE, false);
        // **Both wings, or one with somebody aboard**: it comes down.
        let falls = grounded_for_good(m) || riding(m);
        if falls && aloft(m) && !matches!(m.doing, Doing::Toppled { .. }) {
            m.doing = Doing::Toppled {
                left: m.sp().topple_frames(),
            };
            m.poise = 0;
        }
    }
    if aloft(m) {
        // **Clipped**: a knock-up while it is low crashes it; anywhere else,
        // and a grab or a slow, puts it into a low glide.
        if let Doing::Stumble { .. } = m.doing {
            if flags(m) & flag::LOW != 0 {
                m.doing = Doing::Toppled {
                    left: m.sp().topple_frames(),
                };
                bump_byte(&mut w.lore, word::CRASHES, 2);
            } else {
                m.doing = Doing::Prowl;
                w.lore
                    .set_word(word::GLIDE, Knob::GlideFrames.raw().max(0) as u32);
            }
        }
        if m.rooted > 0 || (m.slowed > 0 && m.slow_mul.raw() < Fx::ONE.raw()) {
            m.rooted = 0;
            m.slowed = 0;
            m.slow_mul = Fx::ONE;
            if m.doing.free() {
                w.lore
                    .set_word(word::GLIDE, Knob::GlideFrames.raw().max(0) as u32);
            }
        }
    }
    // **A crash beginning**: counted by what earned it, and the height it
    // falls from kept for the riders' share.
    let toppled = matches!(m.doing, Doing::Toppled { .. });
    if toppled && !was_toppled {
        set_fx(&mut w.lore, word::FELL_FROM, m.pos.y);
        let cause = if broken_wings(m) > 0 && (grounded_for_good(m) || riding(m)) {
            3
        } else if m.brain.last_move == STOOP {
            1
        } else if m.brain.last_move == TALON {
            0
        } else {
            2
        };
        // A clip's crash was counted where it was decided.
        if !(cause == 2 && byte_of(&w.lore, word::CRASHES, 2) > 0 && aloft(m)) {
            bump_byte(&mut w.lore, word::CRASHES, cause);
        }
    }
    if toppled && aloft(m) {
        let h = fx_word(&w.lore, word::FELL_FROM).max(m.pos.y);
        set_fx(&mut w.lore, word::FELL_FROM, h);
    }
}

/// **As an air move commits** -- the first frame of its startup -- the
/// frame hook keeps what the move needs and spends the wind: the talon
/// lane, the Downwash's point, the volley's lane.
fn committed(w: &mut World, m: &mut Monster) {
    let Doing::Startup { kind, left } = m.doing else {
        return;
    };
    let a = SPECIES.attack(kind);
    if left != a.startup {
        return;
    }
    spend(&mut w.lore, kind);
    w.lore.set_word(word::INTENT, 0);
    let lead = m.lead_point(a.startup);
    let lead = V3::new(lead.x, Fx::ZERO, lead.z);
    let to = V3::new(lead.x.sub(m.pos.x), Fx::ZERO, lead.z.sub(m.pos.z));
    let dir = if math::wide_flat_len(to).raw() > 0 {
        math::wide_normalized(to)
    } else {
        V3::from_turns(m.yaw)
    };
    let yaw = math::atan2_turns(dir.z, dir.x);
    match kind {
        TALON => {
            let start = lead.sub(dir.scale(Knob::LaneLead.fx()));
            w.lore.set_word(word::LANE_AT, word_of(start));
            w.lore.set_int(word::LANE_YAW, yaw.raw());
            w.lore.set_word(word::LANE_PIVOT, word_of(lead));
            bump_half(&mut w.lore, word::PASSES, false);
        }
        STOOP => {
            bump_half(&mut w.lore, word::PASSES, false);
        }
        DOWNWASH => {
            let back = dir.scale(Knob::WashBeside.fx().neg());
            w.lore.set_word(word::WASH_AT, word_of(lead.add(back)));
        }
        VOLLEY => {
            let heading = V3::from_turns(m.yaw);
            let start = lead.sub(heading.scale(math::half(Knob::VolleyLength.fx())));
            w.lore.set_word(word::RAKE_AT, word_of(start));
            w.lore.set_int(word::RAKE_YAW, m.yaw.raw());
            w.lore.set_word(word::RAKE_HITS, 0);
        }
        _ => {}
    }
}

/// **The windup follows you; the hit does not**: the Stoop's circle and the
/// talon lane track the target until they lock -- the Stoop's for its last
/// `StoopLock` frames, the lane's for its last `LaneLock`.
fn track(w: &mut World, m: &mut Monster) {
    match m.doing {
        Doing::Startup { kind: STOOP, left } if left as i32 > Knob::StoopLock.raw() => {
            let a = SPECIES.attack(STOOP);
            let want = m.lead_point(left);
            let aim = m.aimed_at();
            let gap = V3::new(want.x.sub(aim.x), Fx::ZERO, want.z.sub(aim.z));
            let far = math::wide_flat_len(gap);
            let step = Knob::StoopTrack.fx().mul(DT).min(far);
            if far.raw() > 0 {
                let next = aim.add(math::wide_normalized(gap).scale(step));
                // Inside the move's own reach of where it is.
                let from = V3::new(m.pos.x, Fx::ZERO, m.pos.z);
                let out = V3::new(next.x.sub(from.x), Fx::ZERO, next.z.sub(from.z));
                let most = a.ideal_range.add(a.range_span);
                let next = if math::wide_flat_len(out).raw() > most.raw() {
                    from.add(math::wide_normalized(out).scale(most))
                } else {
                    next
                };
                m.aim_at(next);
                set_aim_height(m, ground_at(w, next));
            }
        }
        Doing::Startup { kind: TALON, left } if left as i32 > Knob::LaneLock.raw() => {
            let want = m.lead_point(left);
            let pivot = point(w.lore.word(word::LANE_PIVOT));
            let gap = V3::new(want.x.sub(pivot.x), Fx::ZERO, want.z.sub(pivot.z));
            let far = math::wide_flat_len(gap);
            if far.raw() > 0 {
                let step = Knob::LaneTrack.fx().mul(DT).min(far);
                let next = pivot.add(math::wide_normalized(gap).scale(step));
                let yaw = Fx::from_raw(w.lore.int(word::LANE_YAW));
                let start = next.sub(V3::from_turns(yaw).scale(Knob::LaneLead.fx()));
                w.lore.set_word(word::LANE_PIVOT, word_of(next));
                w.lore.set_word(word::LANE_AT, word_of(start));
            }
        }
        _ => {}
    }
}

/// On the first frame: the bars, the coop numbers, and the bird in the air
/// on its circle.
fn setup(w: &mut World, slot: usize) {
    if w.lore.word(word::FLAGS) & 1 != 0 {
        return;
    }
    let hunters = w.players.iter().filter(|p| p.health > 0).count();
    let centre = circle_centre(w);
    let base = base(w);
    let Some(mut m) = w.monsters[slot] else {
        return;
    };
    let mut lore_flags = 1;
    set_bar(&mut m, 0, Knob::WingBar.raw());
    set_bar(&mut m, 1, Knob::WingBar.raw());
    set_flag(&mut m, flag::SET_UP, true);
    if hunters >= 2 {
        lore_flags |= 2;
        m.health = Knob::CoopHealth.raw().max(1);
        set_flag(&mut m, flag::COOP, true);
    }
    // On its circle, at its height, heading round it: on the far side from
    // where its mark faces.
    let r = Knob::CircleRadius.fx();
    let out = V3::new(m.pos.x.sub(centre.x), Fx::ZERO, m.pos.z.sub(centre.z));
    let out = if math::wide_flat_len(out).raw() > 0 {
        math::wide_normalized(out)
    } else {
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO)
    };
    let at = V3::new(
        centre.x.add(out.x.mul(r)),
        base.add(Knob::CruiseAlt.fx()),
        centre.z.add(out.z.mul(r)),
    );
    // Heading round anticlockwise seen from above: a quarter turn on from
    // the way out.
    let heading = math::wrap_turns(math::atan2_turns(out.z, out.x).add(math::QUARTER_TURN));
    m.pos = at;
    m.yaw = heading;
    m.yaw_rate = Fx::ZERO;
    m.speed = Fx::ZERO;
    set_flag(&mut m, flag::ALOFT, true);
    let f = flight::Flight {
        pos: at,
        speed: Knob::CruiseSpeed.fx(),
        vy: Fx::ZERO,
        yaw: heading,
        yaw_rate: Fx::ZERO,
    };
    f.save(&mut w.lore);
    set_fx(&mut w.lore, word::WIND, Fx::from_int(Knob::WindMax.raw()));
    w.lore.set_word(word::FLAGS, lore_flags | 4);
    w.monsters[slot] = Some(m);
}

/// **Moves the bird starts itself**: the perch when the wind runs low, the
/// lift off the floor, the roll at the end of a ride's lap, the carry when
/// the talons close. A move started here is played and timed as one, and
/// the brain never picks it.
pub fn start(m: &mut Monster, kind: u8) {
    let a = SPECIES.attack(kind);
    m.doing = Doing::Startup {
        kind,
        left: a.startup,
    };
    m.hit_used = false;
    m.brain.mirror = false;
}

/// What it decides that is not a scored choice: perching, lifting off,
/// rolling with a rider, landing for good.
fn transitions(w: &mut World, m: &mut Monster, slot: usize) {
    let riders = riders_on(w, slot);
    set_flag(m, flag::RIDING, riders > 0 && aloft(m));
    if !m.doing.free() {
        return;
    }
    let lore = &mut w.lore;
    // On the floor and free: a moment to throw a ground move, then up --
    // unless it can never fly again, or is resting.
    if !aloft(m) && !grounded_for_good(m) {
        if perched(m) {
            let rest = lore.word(word::REST);
            if rest == 0 {
                set_flag(m, flag::PERCHED, false);
                start(m, LIFT);
            }
            return;
        }
        let dwell = lore.word(word::DWELL) + 1;
        lore.set_word(word::DWELL, dwell);
        // Riders on a bird that has gathered itself go up with it: it does
        // not wait for them to decide.
        // **A while on the floor**: the walk-up after a Stoop, with the
        // buffet and the Screech as its price, before it gathers itself.
        let wait = Knob::GroundDwell.raw().max(0) as u32;
        if dwell > wait {
            lore.set_word(word::DWELL, 0);
            start(m, LIFT);
        }
        return;
    }
    lore.set_word(word::DWELL, 0);
    if !aloft(m) {
        return;
    }
    // Aloft and free.
    if riders > 0 {
        return;
    }
    // **The perch**: the wind run low. Not with nowhere to perch.
    if wind(lore).raw() < Fx::from_int(Knob::PerchWind.raw()).raw()
        && !grounded_for_good(m)
        && perch_top(w).is_some()
    {
        start(m, PERCH);
        bump_half(&mut w.lore, word::PASSES, true);
        return;
    }
    intend(w, m);
}

/// **The decision point** (§5): as its target comes into the line-up arc,
/// it decides which air move this approach is for -- a weighted draw among
/// those it can afford and has off lockout, the last one less likely -- and
/// the brain then throws that one when the range fits, or nothing. The
/// intent is dropped when the target leaves the arc, and drawn again the
/// next time round.
fn intend(w: &mut World, m: &mut Monster) {
    let target = V3::new(m.brain.seen.x, Fx::ZERO, m.brain.seen.z);
    let to = V3::new(target.x.sub(m.pos.x), Fx::ZERO, target.z.sub(m.pos.z));
    let inside = math::wide_flat_len(to).raw() > 0 && {
        let off = math::wrap_turns(math::atan2_turns(to.z, to.x).sub(m.yaw)).abs();
        off.raw() <= Knob::LineUpArc.fx().raw()
    };
    if !inside {
        w.lore.set_word(word::INTENT, 0);
        return;
    }
    if w.lore.word(word::INTENT) != 0 {
        return;
    }
    let mut weights = [0i32; super::MOVE_COUNT];
    let mut total = 0;
    for kind in [STOOP, TALON, DOWNWASH, VOLLEY] {
        let a = SPECIES.attack(kind);
        if m.brain.cooldown[kind as usize] > 0 || cost(kind) > wind(&w.lore).to_int() {
            continue;
        }
        let mut wgt = a.weight.max(0);
        if m.brain.last_move == kind {
            wgt = Fx::from_int(wgt).mul(Knob::RepeatShare.fx()).to_int();
        }
        weights[kind as usize] = wgt;
        total += wgt;
    }
    if total <= 0 {
        return;
    }
    let mut ticket = (draw(m) % total as u32) as i32;
    for (kind, wgt) in weights.iter().enumerate() {
        if *wgt <= 0 {
            continue;
        }
        ticket -= wgt;
        if ticket < 0 {
            w.lore.set_word(word::INTENT, kind as u32 + 1);
            return;
        }
    }
}

/// A xorshift step on the creature's own generator.
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

/// The air move this approach is for, if it has decided.
pub fn intent(lore: &Lore) -> Option<u8> {
    let w = lore.word(word::INTENT);
    (w > 0).then(|| (w - 1) as u8)
}

/// How many fighters are standing on it.
pub fn riders_on(w: &World, slot: usize) -> usize {
    w.players
        .iter()
        .filter(|p| p.health > 0 && p.aboard() && mount_slot(p.mount) == slot)
        .count()
}

/// **What it sees that the brain needs and cannot ask**: for each hunter,
/// whether a Downwash from where the bird would hang would reach them -- no
/// solid between them and the point under it -- and whether they stand
/// somewhere a push costs a fall. Asked through `aim::clear_between`, the
/// same question the push itself asks.
fn sense(w: &mut World, m: &Monster) {
    let ground = w.terrain();
    let field = crate::stones::gather(&w.players);
    let fighters = w.players;
    let effects = w.effects;
    let critters = w.critters;
    let scene = Scene {
        stones: &field,
        players: &fighters,
        effects: &effects,
        quarry: &[None; MAX_MONSTERS],
        critters: &critters,
        arena: &ground,
    };
    let tower = perch_top(w);
    let b = base(w);
    let edge = Knob::EdgeNear.fx();
    let mut bits = 0u32;
    for (i, p) in fighters.iter().enumerate().take(MAX_PLAYERS.min(4)) {
        let p = *p;
        if p.health <= 0 {
            continue;
        }
        let mut s = 0;
        // The point it would hang over: beside them, toward the bird.
        let toward = V3::new(m.pos.x.sub(p.pos.x), Fx::ZERO, m.pos.z.sub(p.pos.z));
        let toward = if math::wide_flat_len(toward).raw() > 0 {
            math::wide_normalized(toward)
        } else {
            V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO)
        };
        let under = p.pos.add(toward.scale(Knob::WashBeside.fx()));
        let under = V3::new(under.x, ground_at(w, under), under.z);
        if aim::clear_between(under, p.pos, &scene) {
            s |= seen::EXPOSED;
        }
        let on_tower = tower.is_some_and(|top| {
            p.pos.y.raw() >= top.y.sub(Fx::ONE).raw()
                && math::wide_flat_dist(p.pos, top).raw() <= Knob::EdgeNear.fx().raw()
        });
        if on_tower {
            s |= seen::TOWER | seen::EDGE;
        }
        // Near a drop: the ground a few metres out from them toward the
        // nearest edge of the bounds is lower than where they stand.
        if near_a_drop(w, p.pos, edge, b) {
            s |= seen::EDGE;
        }
        bits |= s << (8 * i);
    }
    w.lore.set_word(word::SEEN, bits);
}

/// Is there a drop past the free fall height within `reach` of `at`, along
/// any of the four flat directions? The edge of the plateau, the side of
/// the tower: somewhere a push costs.
fn near_a_drop(w: &World, at: V3, reach: Fx, _base: Fx) -> bool {
    let here = ground_at(w, at);
    let dirs = [
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
        V3::new(Fx::ONE.neg(), Fx::ZERO, Fx::ZERO),
        V3::new(Fx::ZERO, Fx::ZERO, Fx::ONE),
        V3::new(Fx::ZERO, Fx::ZERO, Fx::ONE.neg()),
    ];
    dirs.iter().any(|d| {
        let there = at.add(d.scale(reach));
        here.sub(ground_at(w, there)).raw() > t::fall_free().raw()
    })
}

// ---------------------------------------------------------------------------
// The hooks the body is read through
// ---------------------------------------------------------------------------

/// **A hit on its wings** fills that wing's bar (a root's hit was already
/// doubled by its hide), and -- while it is low -- the wing poise, which is
/// the shared pool: the shared ladder crashes it when it is full. A leg hit
/// while it carries somebody counts toward their freedom. Breaking a wing
/// is said to the frame hook through a flag. Returning false lets the
/// shared ladder (flinch, interrupt, the crash) run.
pub fn struck(m: &mut Monster, part: usize, dealt: i32) -> bool {
    if is_leg(part) && matches!(m.doing.attacking(), Some(CARRY)) {
        let w = m.own[body::LEGS];
        let dmg = ((w & 0xFFFF) + dealt).min(0xFFFF);
        let hits = (((w >> 16) & 0xFF) + 1).min(0xFF);
        m.own[body::LEGS] = dmg | (hits << 16);
    }
    let Some(side) = wing_of(part) else {
        // Nothing but a wing fills the poise.
        return false;
    };
    let was = bar(m, side);
    if was > 0 {
        let now = (was - dealt).max(0);
        set_bar(m, side, now);
        if now == 0 {
            set_flag(m, flag::BROKE, true);
        }
    }
    // Low -- passing, or down on the floor after a Stoop -- and not already
    // crashed or grounded for good.
    let down = matches!(m.doing, Doing::Toppled { .. }) || grounded_for_good(m);
    if flags(m) & flag::LOW != 0 && !down && !perched(m) {
        m.poise += dealt;
    }
    false
}

/// **Coming down on a point it has no body to shove with**: through a
/// Stoop's dive and its hit, and the grounded hop's, every part is passable
/// -- hit, never walked into -- so the hit is the hit, and a fighter is not
/// barged out of its circle by the body arriving a frame early.
pub fn presence(m: &Monster, _rig: &crate::beast::Rig) -> crate::beast::Presence {
    let diving = matches!(
        m.doing,
        Doing::Startup {
            kind: STOOP | HOP,
            ..
        } | Doing::Active {
            kind: STOOP | HOP,
            ..
        }
    );
    crate::beast::Presence {
        buried: 0,
        unmountable: 0,
        passable: if diving { u64::MAX } else { 0 },
    }
}

/// **What its hide is worth this frame**: everything ×`CrashHide` while it
/// lies crashed.
pub fn hide(m: &Monster, _part: usize) -> Fx {
    if crashed(m) {
        Knob::CrashHide.fx()
    } else {
        Fx::ONE
    }
}

/// A lobbed move lands on the ground at its aim point: the plateau, or the
/// tower's top if that is where it is aimed. The rig has no world, so the
/// height is the one `mind::commit` kept on the body when it aimed.
pub fn lob_height(m: &Monster) -> Fx {
    aim_height(m)
}

/// The height of the ground at its aim point, in centimetres on the body:
/// kept by `mind::commit` in `body::LEGS`, which nothing else uses outside
/// a carry.
pub fn aim_height(m: &Monster) -> Fx {
    if matches!(m.doing.attacking(), Some(STOOP) | Some(HOP)) {
        lore::from_cm(m.own[body::LEGS] as i16)
    } else {
        Fx::ZERO
    }
}

/// Keep the ground height at its aim point on the body.
pub fn set_aim_height(m: &mut Monster, h: Fx) {
    m.own[body::LEGS] = lore::to_cm(h) as i32;
}

/// Two hunters: it glances a fifth sooner.
pub fn glance(m: &Monster) -> u16 {
    let g = m.sp().glance_frames();
    if flags(m) & flag::COOP != 0 {
        (g as i32 * 4 / 5).max(1) as u16
    } else {
        g
    }
}

// ---------------------------------------------------------------------------
// The talons and the carry
// ---------------------------------------------------------------------------

/// The talon lane: where it starts, which way it runs, how long and wide.
pub fn lane(lore: &Lore) -> (V3, V3, Fx, Fx) {
    let at = point(lore.word(word::LANE_AT));
    let yaw = Fx::from_raw(lore.int(word::LANE_YAW));
    (
        at,
        V3::from_turns(yaw),
        Knob::LaneLength.fx(),
        Knob::LaneWidth.fx(),
    )
}

/// How far along its lane the front has come, `left` frames from the end of
/// the pass.
pub fn front(left: u16) -> Fx {
    let a = SPECIES.attack(TALON);
    let gone = a.active.saturating_sub(left) as i32;
    Knob::LaneLength
        .fx()
        .mul(Fx::from_int(gone))
        .div(Fx::from_int(a.active.max(1) as i32))
}

/// Is a fighter standing at `p` inside the lane's strip, and how far along?
fn in_strip(at: V3, along: V3, length: Fx, width: Fx, p: V3) -> Option<Fx> {
    let d = V3::new(p.x.sub(at.x), Fx::ZERO, p.z.sub(at.z));
    let s = d.dot(along);
    let side = V3::new(along.z.neg(), Fx::ZERO, along.x);
    let c = d.dot(side).abs();
    let half = math::half(width).add(t::body_radius());
    (s.raw() >= 0 && s.raw() <= length.raw() && c.raw() <= half.raw()).then_some(s)
}

/// **The pass**: the front sweeps the lane, and the first fighter it
/// reaches standing taller than the talons clear -- not crouched, not
/// dodging -- is caught.
fn talons(w: &mut World, m: &mut Monster, slot: usize) {
    let Doing::Active { kind: TALON, left } = m.doing else {
        return;
    };
    if m.hit_used {
        return;
    }
    let (at, along, _, width) = lane(&w.lore);
    let a = SPECIES.attack(TALON);
    let was = front(left.saturating_add(1).min(a.active));
    let now = front(left);
    let reach = Knob::TalonHeight.fx();
    let mut best: Option<(usize, Fx)> = None;
    for i in 0..MAX_PLAYERS {
        let p = w.players[i];
        if p.health <= 0 || p.aboard() || p.action.invulnerable() {
            continue;
        }
        // Somebody already held elsewhere is not caught again.
        if carried(&w.lore) == Some(i) {
            continue;
        }
        let Some(s) = in_strip(at, along, Knob::LaneLength.fx(), width, p.pos) else {
            continue;
        };
        if s.raw() < was.sub(Fx::ONE).raw() || s.raw() > now.add(Fx::ONE).raw() {
            continue;
        }
        // The talons clear `TalonHeight` over the ground under the lane.
        let floor = ground_at(w, p.pos);
        let top = p.pos.y.add(p.hurt_height());
        if top.raw() <= floor.add(reach).raw() {
            continue;
        }
        // And reach no higher than the body passing over.
        if p.pos.y.raw() > floor.add(Knob::PassHeight.fx()).raw() {
            continue;
        }
        if best.is_none_or(|(_, seen)| s.raw() < seen.raw()) {
            best = Some((i, s));
        }
    }
    let Some((i, _)) = best else {
        return;
    };
    m.hit_used = true;
    // **The catch**: the grab's damage, and the climb begins.
    let dir = along;
    state::apply_hit(
        &mut w.players[i],
        Hit {
            damage: Knob::CarryGrab.raw(),
            hitstun: a.hitstun,
            blockstun: a.blockstun,
            knockback: Fx::ZERO,
            launch: Fx::ZERO,
            grabs: 0,
            by: QUARRY,
            dir,
            blocked: false,
            parried: false,
            interrupts: true,
        },
    );
    let frame_lo = w.frame & 0xFFFF;
    w.lore
        .set_word(word::CARRIED, (i as u32 + 1) | (frame_lo << 16));
    bump_half(&mut w.lore, word::CARRIES, false);
    m.own[body::LEGS] = 0;
    start(m, CARRY);
    // The carry's climb starts at once: the startup is the grab.
    let _ = slot;
}

/// **Where a carried fighter hangs**: held round the chest in its talons --
/// their middle at the talons' tips, their feet a body's half-height and
/// more below -- and never under the ground beneath them, which they are
/// dragged along until the climb lifts them off it.
pub fn talon_point(w: &World, m: &Monster) -> V3 {
    let rig = m.rig();
    let l = rig.bone[bones::SHIN_L].at;
    let r = rig.bone[bones::SHIN_R].at;
    let mid = V3::new(
        math::half(l.x.add(r.x)),
        math::half(l.y.add(r.y)),
        math::half(l.z.add(r.z)),
    );
    let tips = mid.y.add(SPECIES.shape(super::TALON_L).min.y);
    let feet = tips.sub(math::half(t::body_height()));
    let at = V3::new(mid.x, feet, mid.z);
    V3::new(at.x, feet.max(ground_at(w, at)), at.z)
}

/// **The carry**: the fighter held under the talons for the climb, and let
/// go at the end of it -- or early, three hits or `CarryFreeDamage` on the
/// legs. Let go, they fall from where they are.
fn carry(w: &mut World, m: &mut Monster, _slot: usize) {
    let Some(i) = carried(&w.lore) else {
        return;
    };
    let holding =
        matches!(m.doing.attacking(), Some(CARRY)) && !matches!(m.doing, Doing::Recovery { .. });
    let legs = m.own[body::LEGS];
    let freed = (legs & 0xFFFF) >= Knob::CarryFreeDamage.raw()
        || ((legs >> 16) & 0xFF) >= Knob::CarryFreeHits.raw();
    let p = &mut w.players[i];
    if !holding || freed || p.health <= 0 || !m.alive() {
        if freed && holding {
            bump_half(&mut w.lore, word::CARRIES, true);
        }
        let height = p.pos.y;
        p.falling_from(height);
        p.grounded = false;
        w.lore.set_word(word::CARRIED, 0);
        m.own[body::LEGS] = 0;
        if holding {
            // Its talons are empty: the climb is over.
            let a = SPECIES.attack(CARRY);
            m.doing = Doing::Recovery {
                kind: CARRY,
                left: a.recovery,
            };
        }
        return;
    }
    let at = talon_point(w, m);
    let p = &mut w.players[i];
    p.pos = at;
    p.vel = V3::ZERO;
    p.grounded = false;
    p.mount = crate::monster::NO_PART;
}

/// Let go of whoever it holds, wherever it is: it died, or it was knocked
/// out of the climb.
fn drop_carried(w: &mut World, m: &Monster) {
    if let Some(i) = carried(&w.lore) {
        let p = &mut w.players[i];
        let h = p.pos.y;
        p.falling_from(h);
        w.lore.set_word(word::CARRIED, 0);
    }
    let _ = m;
}

// ---------------------------------------------------------------------------
// The Downwash
// ---------------------------------------------------------------------------

/// The point under the hovering bird, on the ground.
pub fn wash_point(w: &World) -> V3 {
    let at = point(w.lore.word(word::WASH_AT));
    V3::new(at.x, ground_at(w, at), at.z)
}

/// **Is a fighter at `p` in the Downwash's lee**: a solid between them and
/// the point under the bird. The push's own answer, and the sign's.
pub fn in_lee(scene: &Scene, under: V3, p: V3) -> bool {
    !aim::clear_between(under, p, scene)
}

/// The push a fighter at `p` feels from a Downwash at `under`, per second:
/// straight out from the point under it, zero in the eye, zero in the lee,
/// a quarter crouched. What the frame applies, and what the tests ask.
pub fn wash_push(scene: &Scene, under: V3, p: &crate::state::Player) -> V3 {
    let gap = V3::new(p.pos.x.sub(under.x), Fx::ZERO, p.pos.z.sub(under.z));
    let far = math::wide_flat_len(gap);
    if far.raw() > Knob::WashRadius.fx().raw() || far.raw() <= Knob::WashEye.fx().raw() {
        return V3::ZERO;
    }
    if in_lee(scene, under, p.pos) {
        return V3::ZERO;
    }
    let speed = if p.crouching {
        Knob::WashPush.fx().mul(Knob::WashCrouch.fx())
    } else {
        Knob::WashPush.fx()
    };
    math::wide_normalized(gap).scale(speed)
}

/// **The push**, every active frame: everybody in the ring not in its lee
/// or its eye slides straight outward. Applied to the position, before the
/// fighters step, as the floor's pull is -- so a wall stops it and an edge
/// does not.
fn downwash(w: &mut World, m: &Monster) {
    if !matches!(m.doing, Doing::Active { kind: DOWNWASH, .. }) {
        return;
    }
    let under = wash_point(w);
    let ground = w.terrain();
    let field = crate::stones::gather(&w.players);
    let fighters = w.players;
    let effects = w.effects;
    let critters = w.critters;
    let scene = Scene {
        stones: &field,
        players: &fighters,
        effects: &effects,
        quarry: &[None; MAX_MONSTERS],
        critters: &critters,
        arena: &ground,
    };
    for (i, p) in fighters.iter().enumerate() {
        let p = *p;
        if p.health <= 0 || p.aboard() || carried(&w.lore) == Some(i) {
            continue;
        }
        let push = wash_push(&scene, under, &p);
        if push == V3::ZERO {
            continue;
        }
        w.players[i].pos = w.players[i].pos.add(push.scale(DT));
    }
}

// ---------------------------------------------------------------------------
// The volley
// ---------------------------------------------------------------------------

/// The volley's lane: start, direction, length, width.
pub fn rake_lane(lore: &Lore) -> (V3, V3, Fx, Fx) {
    let at = point(lore.word(word::RAKE_AT));
    let yaw = Fx::from_raw(lore.int(word::RAKE_YAW));
    (
        at,
        V3::from_turns(yaw),
        Knob::VolleyLength.fx(),
        Knob::VolleyWidth.fx(),
    )
}

/// **Which stretch of the lane is under feathers**, `gone` frames into the
/// rake: from `lo` to `hi` metres along it. Each point is under them for
/// `VolleyUnder` frames, the curtain running the lane's length over the
/// rest of the active window.
pub fn raked(gone: u16) -> (Fx, Fx) {
    let a = SPECIES.attack(VOLLEY);
    let under = Knob::VolleyUnder.raw().max(1);
    let run = (a.active as i32 - under).max(1);
    let len = Knob::VolleyLength.fx();
    // A point s along is covered from t0 = s/len * run to t0 + under.
    let at = |t: i32| len.mul(Fx::from_int(t)).div(Fx::from_int(run));
    let hi = at(gone as i32).min(len);
    let lo = at(gone as i32 - under).max(Fx::ZERO);
    (lo, hi)
}

/// **The rake**: a feather every `VolleyEvery` frames on a fighter standing
/// where the curtain is, up to `VolleyMost`. A dodge's invulnerable frames
/// are shorter than the time any point stays raked.
fn volley(w: &mut World, m: &mut Monster) {
    let Doing::Active { kind: VOLLEY, left } = m.doing else {
        return;
    };
    let a = SPECIES.attack(VOLLEY);
    let gone = a.active.saturating_sub(left);
    if gone == 0 {
        w.lore.set_word(word::RAKE_HITS, 0);
    }
    let (at, along, length, width) = rake_lane(&w.lore);
    let (lo, hi) = raked(gone);
    let mut hits = w.lore.word(word::RAKE_HITS);
    for i in 0..MAX_PLAYERS.min(2) {
        let p = w.players[i];
        if p.health <= 0 || p.aboard() || p.action.invulnerable() {
            continue;
        }
        let Some(s) = in_strip(at, along, length, width, p.pos) else {
            continue;
        };
        if s.raw() < lo.raw() || s.raw() > hi.raw() {
            continue;
        }
        let taken = (hits >> (8 * i)) & 0xF;
        let since = (hits >> (8 * i + 4)) & 0xF;
        if taken >= Knob::VolleyMost.raw().max(0) as u32 {
            continue;
        }
        let every = Knob::VolleyEvery.raw().clamp(1, 15) as u32;
        if taken > 0 && since < every {
            continue;
        }
        // Feathers come down from above: a guard is a facing arc, and this
        // is not in front of it (galewing.md §12.5, kept to one answer).
        let guarding = false;
        state::apply_hit(
            &mut w.players[i],
            Hit {
                damage: a.damage,
                hitstun: a.hitstun,
                blockstun: a.blockstun,
                knockback: a.knockback,
                launch: a.launch,
                grabs: 0,
                by: QUARRY,
                dir: V3::new(along.z.neg(), Fx::ZERO, along.x),
                blocked: guarding,
                parried: false,
                interrupts: true,
            },
        );
        m.hit_used = true;
        let mask = 0xFFu32 << (8 * i);
        hits = (hits & !mask) | ((taken + 1) << (8 * i));
    }
    // Every hunter's frames since their last feather, capped at fifteen.
    for i in 0..2 {
        let since = (hits >> (8 * i + 4)) & 0xF;
        let taken = (hits >> (8 * i)) & 0xF;
        let since = (since + 1).min(15);
        let mask = 0xFFu32 << (8 * i);
        hits = (hits & !mask) | ((taken | (since << 4)) << (8 * i));
    }
    w.lore.set_word(word::RAKE_HITS, hits);
}

// ---------------------------------------------------------------------------
// The ground moves
// ---------------------------------------------------------------------------

/// How many discs the Screech's cone is drawn and tested as: a count, not
/// a size -- its length and width are the knobs.
pub const CONE_DISCS: usize = 4;

/// How many discs the buffet's sweep is drawn and tested as.
pub const BUFFET_DISCS: usize = 3;

/// **The Screech's cone, as discs**: the i-th of [`CONE_DISCS`] along its
/// facing, each wide enough to touch its neighbours and the cone's edge. The
/// one shape the hit test and the sign both read (the Sandmaw's way).
pub fn cone_disc(m: &Monster, i: usize) -> (V3, Fx) {
    let len = Knob::ScreechLength.fx();
    let half = Knob::ScreechHalf.fx();
    let n = CONE_DISCS as i32;
    // The middle of the i-th of n equal stretches of its length.
    let step = len.div(Fx::from_int(n));
    let s = step.mul(Fx::from_int(i as i32)).add(math::half(step));
    let cos = crate::fixed::cos_turns(half);
    // A cone a quarter turn wide either side is a half-plane: as wide as it
    // is long, at every disc.
    let wide = if cos.raw() > 0 {
        s.mul(crate::fixed::sin_turns(half)).div(cos)
    } else {
        len
    };
    let r = wide.max(math::half(step));
    let at = m.pos.add(V3::from_turns(m.yaw).scale(s));
    (V3::new(at.x, m.pos.y, at.z), r)
}

/// The buffet's sweep, as discs along the wing on the side it goes to:
/// out to `BuffetReach`, from the floor to `BuffetHigh`.
pub fn buffet_disc(m: &Monster, i: usize) -> (V3, Fx) {
    let reach = Knob::BuffetReach.fx();
    let n = BUFFET_DISCS as i32;
    let side = if m.brain.mirror {
        Fx::ONE
    } else {
        Fx::ONE.neg()
    };
    let out = V3::from_turns(m.yaw.add(math::QUARTER_TURN)).scale(side);
    let step = reach.div(Fx::from_int(n));
    let s = step.mul(Fx::from_int(i as i32)).add(math::half(step));
    let at = m.pos.add(out.scale(s));
    // Each touches its neighbours, and a body's width more.
    (
        V3::new(at.x, m.pos.y, at.z),
        math::half(step).add(t::body_radius()),
    )
}

/// Is a fighter at `p` (feet) inside a disc at `at` of radius `r`, between
/// the floor and `high` over it?
fn in_disc(at: V3, r: Fx, high: Fx, p: &crate::state::Player) -> bool {
    let flat = math::wide_flat_dist(p.pos, at);
    flat.raw() <= r.add(t::body_radius()).raw()
        && p.pos.y.raw() <= at.y.add(high).raw()
        && p.pos.y.add(p.hurt_height()).raw() >= at.y.raw()
}

/// A hit from one of its own volumes, guarded against from the front.
fn own_strike(w: &mut World, m: &mut Monster, i: usize, kind: u8, from: V3) {
    let a = SPECIES.attack(kind);
    let p = w.players[i];
    let away = math::wide_normalized(V3::new(p.pos.x.sub(from.x), Fx::ZERO, p.pos.z.sub(from.z)));
    let facing_it = p.facing.dot(away.scale(Fx::ONE.neg())).raw() >= t::guard_arc_cos().raw();
    let guarding = !a.unblockable && p.action.guarding() && facing_it;
    state::apply_hit(
        &mut w.players[i],
        Hit {
            damage: a.damage,
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
    m.hit_used = true;
}

/// **The Screech**: everybody in the cone, once, stunned.
fn screech(w: &mut World, m: &mut Monster) {
    own_area(
        w,
        m,
        SCREECH,
        Knob::ScreechLength.fx(),
        cone_disc,
        CONE_DISCS,
    );
}

/// **The buffet**: everybody in the sweep on its side, once -- unless they
/// are in the air over it.
fn buffet(w: &mut World, m: &mut Monster) {
    own_area(w, m, BUFFET, Knob::BuffetHigh.fx(), buffet_disc, 3);
}

fn own_area(
    w: &mut World,
    m: &mut Monster,
    kind: u8,
    high: Fx,
    disc: impl Fn(&Monster, usize) -> (V3, Fx),
    n: usize,
) {
    match m.doing {
        Doing::Startup { kind: k, .. } if k == kind => {
            w.lore.set_word(word::STRUCK, 0);
            return;
        }
        Doing::Active { kind: k, .. } if k == kind => {}
        _ => return,
    }
    let mut struck = w.lore.word(word::STRUCK);
    for i in 0..MAX_PLAYERS {
        let p = w.players[i];
        if struck & (1 << i) != 0 || p.health <= 0 || p.aboard() || p.action.invulnerable() {
            continue;
        }
        if (0..n).any(|d| {
            let (at, r) = disc(m, d);
            in_disc(at, r, high, &p)
        }) {
            struck |= 1 << i;
            own_strike(w, m, i, kind, m.pos);
        }
    }
    w.lore.set_word(word::STRUCK, struck);
}

// ---------------------------------------------------------------------------
// Wind and the perch
// ---------------------------------------------------------------------------

/// **Wind**, the budget: regained circling, spent on commit (`mind::commit`),
/// refilled on the perch. **A hit on the perched bird shortens its rest.**
fn wind_and_rest(w: &mut World, m: &mut Monster) {
    let max = Fx::from_int(Knob::WindMax.raw());
    let desperate = below(m, Knob::DesperateHealth.fx());
    let mut wind = wind(&w.lore);
    if aloft(m) && m.doing.free() && !riding(m) {
        let regen = if desperate {
            Knob::WindRegenDesperate.fx()
        } else {
            Knob::WindRegen.fx()
        };
        wind = wind.add(regen.mul(DT)).min(max);
    }
    if perched(m) {
        let rest = w.lore.word(word::REST);
        let total = if desperate {
            Knob::PerchRestDesperate.raw()
        } else {
            Knob::PerchRest.raw()
        }
        .max(1);
        // The wind refills over the rest.
        wind = wind.add(max.div(Fx::from_int(total))).min(max);
        w.lore.set_word(word::REST, rest.saturating_sub(1));
    }
    set_fx(&mut w.lore, word::WIND, wind);
}

/// Shorten the rest for damage taken on the perch: called by the frame hook
/// with what it lost since last frame.
pub fn shorten_rest(lore: &mut Lore, damage: i32) {
    let cut = Fx::from_int(damage.max(0))
        .mul(Knob::PerchShorten.fx())
        .to_int()
        .max(0) as u32;
    let rest = lore.word(word::REST);
    lore.set_word(word::REST, rest.saturating_sub(cut));
}

/// Spend wind on a move: what `mind::commit` calls.
pub fn spend(lore: &mut Lore, kind: u8) {
    let cost = cost(kind);
    let wind = wind(lore).sub(Fx::from_int(cost)).max(Fx::ZERO);
    set_fx(lore, word::WIND, wind);
}

/// A move's wind.
pub fn cost(kind: u8) -> i32 {
    match kind {
        STOOP => Knob::WindStoop.raw(),
        TALON => Knob::WindTalon.raw(),
        DOWNWASH => Knob::WindWash.raw(),
        VOLLEY => Knob::WindVolley.raw(),
        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// What it draws on the floor
// ---------------------------------------------------------------------------

/// **Its floor signs**, from the same shapes its hits are tested against:
/// the talon lane and its front, the Downwash's ring, eye and lees, the
/// volley's lane and its curtain, the Screech's cone, the buffet's sweep.
/// The Stoop's and the hop's circles are the shared telegraph.
pub fn signs(w: &World, out: &mut Signs) {
    let Some(slot) = slot_of(w) else {
        return;
    };
    let Some(m) = w.monsters[slot] else {
        return;
    };
    let lore = &w.lore;
    match m.doing {
        Doing::Startup { kind: TALON, left } => {
            let a = SPECIES.attack(TALON);
            let (at, along, length, width) = lane(lore);
            let at = V3::new(at.x, ground_at(w, at), at.z);
            out.push(
                Sign::strip(Says::Coming, at, along, length, width)
                    .filled(through(left, a.startup)),
            );
        }
        Doing::Active { kind: TALON, left } => {
            let (at, along, length, width) = lane(lore);
            let at = V3::new(at.x, ground_at(w, at), at.z);
            let f = front(left);
            out.push(Sign::strip(Says::Coming, at, along, length, width));
            out.push(Sign::strip(
                Says::Live,
                at.add(along.scale(f)),
                along,
                width.min(length.sub(f).max(Fx::ZERO)),
                width,
            ));
        }
        Doing::Startup {
            kind: DOWNWASH,
            left,
        }
        | Doing::Active {
            kind: DOWNWASH,
            left,
        } => {
            let a = SPECIES.attack(DOWNWASH);
            let live = matches!(m.doing, Doing::Active { .. });
            let under = wash_point(w);
            let says = if live { Says::Live } else { Says::Coming };
            let fill = if live {
                Fx::ONE
            } else {
                through(left, a.startup)
            };
            out.push(Sign::ring(says, under, diameter(Knob::WashRadius.fx())).filled(fill));
            out.push(Sign::disc(Says::Clear, under, diameter(Knob::WashEye.fx())));
            lees(w, under, out);
        }
        Doing::Startup { kind: VOLLEY, left } => {
            let a = SPECIES.attack(VOLLEY);
            let (at, along, length, width) = rake_lane(lore);
            let at = V3::new(at.x, ground_at(w, at), at.z);
            out.push(
                Sign::strip(Says::Coming, at, along, length, width)
                    .filled(through(left, a.startup)),
            );
        }
        Doing::Active { kind: VOLLEY, left } => {
            let a = SPECIES.attack(VOLLEY);
            let (at, along, length, width) = rake_lane(lore);
            let at = V3::new(at.x, ground_at(w, at), at.z);
            let (lo, hi) = raked(a.active.saturating_sub(left));
            out.push(Sign::strip(Says::Coming, at, along, length, width));
            out.push(Sign::strip(
                Says::Live,
                at.add(along.scale(lo)),
                along,
                hi.sub(lo).max(Fx::ZERO),
                width,
            ));
        }
        Doing::Startup {
            kind: SCREECH,
            left,
        }
        | Doing::Active {
            kind: SCREECH,
            left,
        } => {
            let a = SPECIES.attack(SCREECH);
            let live = matches!(m.doing, Doing::Active { .. });
            for i in 0..CONE_DISCS {
                let (at, r) = cone_disc(&m, i);
                let s = Sign::disc(
                    if live { Says::Live } else { Says::Coming },
                    at,
                    diameter(r),
                );
                out.push(if live {
                    s
                } else {
                    s.filled(through(left, a.startup))
                });
            }
        }
        Doing::Startup { kind: BUFFET, left } | Doing::Active { kind: BUFFET, left } => {
            let a = SPECIES.attack(BUFFET);
            let live = matches!(m.doing, Doing::Active { .. });
            for i in 0..BUFFET_DISCS {
                let (at, r) = buffet_disc(&m, i);
                let s = Sign::disc(
                    if live { Says::Live } else { Says::Coming },
                    at,
                    diameter(r),
                );
                out.push(if live {
                    s
                } else {
                    s.filled(through(left, a.startup))
                });
            }
        }
        _ => {}
    }
    // Its shadow is the renderer's; where its perch is, while it announces
    // one, is a ring on the tower top.
    if let Doing::Startup { kind: PERCH, left } = m.doing {
        if let Some(top) = perch_top(w) {
            let a = SPECIES.attack(PERCH);
            out.push(
                Sign::ring(Says::Faint, top, Knob::PerchRing.fx()).filled(through(left, a.startup)),
            );
        }
    }
    let _ = (STOOP, HOP, LIFT, ROLL);
}

/// **The lee behind every solid**, inside the ring: a disc at each stone or
/// solid that shades a patch -- drawn where the push's own test says there
/// is no push. Sampled on a grid of points round the ring and drawn where
/// a sample is shaded, so what is drawn is the push's answer.
fn lees(w: &World, under: V3, out: &mut Signs) {
    let ground = w.terrain();
    let field = crate::stones::gather(&w.players);
    let fighters = w.players;
    let effects = w.effects;
    let critters = w.critters;
    let scene = Scene {
        stones: &field,
        players: &fighters,
        effects: &effects,
        quarry: &[None; MAX_MONSTERS],
        critters: &critters,
        arena: &ground,
    };
    let r = Knob::WashRadius.fx();
    // Rings of samples at two radii, eight bearings each: the list holds
    // twenty-four signs, and the lane and ring take the first few.
    for ring in 1..=2 {
        let quarter = math::half(math::half(r));
        let at_r = if ring == 1 { quarter } else { r.sub(quarter) };
        for k in 0..8 {
            let dir = V3::from_turns(Fx::ratio(k, 8));
            let p = under.add(dir.scale(at_r));
            let p = V3::new(p.x, ground_at(w, p), p.z);
            if in_lee(&scene, under, p) {
                out.push(Sign::disc(Says::Clear, p, Knob::LeeDisc.fx()));
            }
        }
    }
}

/// How far through a countdown, nought to one.
pub fn through(left: u16, total: u16) -> Fx {
    let total = total.max(1);
    Fx::from_int(total.saturating_sub(left.min(total)) as i32).div(Fx::from_int(total as i32))
}

/// A circle's width across, for a sign: twice its radius, because that is
/// what a diameter is.
fn diameter(r: Fx) -> Fx {
    r.add(r)
}

/// **Which paint a wing part wears**, for the renderer's tint: `None` for a
/// part that is not a wing's, `Some(0)` for a whole wing root (where a hit
/// counts double -- painted so it reads as the place to aim), `Some(1)` for a
/// wing past half its bar, `Some(2)` for a broken wing. Read off the first
/// Galewing in the world; there is only ever one.
pub fn wing_stage(w: &World, part: usize) -> Option<usize> {
    let side = super::wing_of(part)?;
    let m = w
        .monsters
        .iter()
        .flatten()
        .find(|m| m.species == SpeciesId::GALEWING)?;
    if broken(m, side) {
        return Some(2);
    }
    if flags(m) & flag::SET_UP != 0 && bar(m, side).saturating_mul(2) < Knob::WingBar.raw() {
        return Some(1);
    }
    if super::is_root(part) {
        return Some(0);
    }
    None
}

/// **Put it where it would throw `kind` at player one, and start it**: for a
/// capture of a telegraph (the game's `SHOT_MOVE`), the way the Sandmaw's
/// `ready_for` is. An air move from its cruising height, a ground move from
/// the plateau, each at the move's own distance along `+x` -- the way the
/// camera starts looking -- facing them. The move starts a frame early so its
/// first frame (the lane laid, the aim taken) happens in the world.
pub fn ready_for(w: &mut World, kind: u8) {
    let me = w.players[0].pos;
    let ground = base(w);
    let Some(slot) = w
        .monsters
        .iter()
        .position(|m| m.is_some_and(|m| m.species == SpeciesId::GALEWING))
    else {
        return;
    };
    // Its first frame's set-up now, so that frame does not put it back on
    // its circle.
    setup(w, slot);
    let Some(m) = w.monsters[slot].as_mut() else {
        return;
    };
    let mv = SPECIES.attack(kind);
    let air = super::aerial(kind);
    let y = if air {
        flight::cruise_over(m, ground)
    } else {
        ground
    };
    let at = V3::new(me.x.add(mv.ideal_range), y, me.z);
    let yaw = Fx::ratio(1, 2);
    let f = flight::Flight {
        pos: at,
        speed: if air {
            Knob::CruiseSpeed.fx()
        } else {
            Fx::ZERO
        },
        vy: Fx::ZERO,
        yaw,
        yaw_rate: Fx::ZERO,
    };
    f.save(&mut w.lore);
    m.pos = at;
    m.yaw = yaw;
    set_flag(m, flag::ALOFT, air);
    m.brain.seen = me;
    m.brain.target = 0;
    m.brain.grace = 0;
    m.brain.think_left = u16::MAX;
    m.aim_at(me);
    m.doing = Doing::Startup {
        kind,
        left: mv.startup + 1,
    };
    // A lobbed move lands on the ground its aim is on, as `mind::commit` keeps.
    set_aim_height(m, me.y);
    m.hit_used = false;
    m.brain.last_move = kind;
}
