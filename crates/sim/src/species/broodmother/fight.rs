//! What the Broodmother does to the floor, her sacs and the fighters: the sac
//! clock, the brood it feeds, the web, her legs and her arc, run once a frame
//! from her `FightDecl::frame` hook.
//!
//! **The sacs** are six sites on her abdomen, each a clock in the hunt's lore
//! ([`word`]): swelling for `SacRipen` frames through pale, amber and red, then
//! bursting into `BroodPerSac` broodlings dropped under it -- or, with the
//! brood at its cap, **held**, red, until there is room. A burst site lays
//! again after `SacRegrow`; a **popped** one is a scar for the rest of the
//! fight. A sac is a soft breakable part of her abdomen, so the shared hit
//! path finds it, and her [`struck`] hook decides what a hit on one does: it
//! goes to the sac and not to her, and a sac that runs out pops -- 300 to her,
//! 300 of strain, a flinch. A sac with no body (empty, or scarred) is not there
//! to hit ([`presence`]).
//!
//! **Her arc** is here too: the clutch below 30% with sacs left (every
//! swelling sac goes red together), and the collapse when the last one pops --
//! the brood break and leave, and what gets up is enraged.
//!
//! The four words on her body (`Monster::own`, [`body`]) are what the hooks
//! without a world read: which sacs have a body, what popped this frame, the
//! Brood guard, who is webbed. Everything else is the lore's.

use super::{Knob, SAC_COUNT, SPECIES, sac_part, sac_site};
use crate::fixed::Fx;
use crate::hazard::{HazardDecl, reach};
use crate::lore::Layout;
use crate::math::{self, V3};
use crate::monster::{Doing, Monster};
use crate::species::FightDecl;
use crate::state::{self, Hit, MAX_PLAYERS, QUARRY, World};

// ---------------------------------------------------------------------------
// The floor's kinds
// ---------------------------------------------------------------------------

/// A web patch: a disc of sticky floor where a glob landed.
pub const PATCH: u8 = 0;
/// A strand: a line a web line left behind, a shin's height off the floor.
pub const STRAND: u8 = 1;

pub const HAZARDS: [HazardDecl; 2] = [
    HazardDecl::disc("web patch").reaching(reach::FIGHTERS),
    HazardDecl::strand("strand").reaching(reach::FIGHTERS),
];

// ---------------------------------------------------------------------------
// Her state
// ---------------------------------------------------------------------------

/// Her own words in the hunt's lore: six cells, twenty-four words.
pub mod word {
    /// Bits: [`super::flag`].
    pub const FLAGS: usize = 0;
    /// One word per sac site, in site order: the clock in the low half, its
    /// state ([`super::sac`]) in the next byte.
    pub const SAC0: usize = 1;
}

/// The bits of [`word::FLAGS`].
pub mod flag {
    /// The round has been set up: the clocks staggered, the first brood out.
    pub const SET_UP: u32 = 1;
    /// The clutch has come: every swelling sac sent red together.
    pub const CLUTCH: u32 = 1 << 1;
    /// The last sac popped: she has collapsed, and is enraged from here on.
    pub const ENRAGED: u32 = 1 << 2;
}

/// What a sac site is doing.
pub mod sac {
    /// Swelling on its clock.
    pub const SWELLING: u32 = 0;
    /// Ripe, with the brood at its cap: red, pulsing, waiting for room.
    pub const HELD: u32 = 1;
    /// Burst: an empty site, laying again on its own clock.
    pub const EMPTY: u32 = 2;
    /// Popped: a torn husk, for the rest of the fight.
    pub const SCARRED: u32 = 3;
}

/// Her four words on the body (`Monster::own`): what the hit path, the rig
/// and her pack's mind read, without a world.
pub mod body {
    /// Bits 0-5: sacs with no body (empty or scarred). Bits 8-13: sacs popped
    /// this frame, for the frame hook to take in. Bits 16-21: sacs struck
    /// this frame (the Brood guard). Bit 24: enraged.
    pub const SACS: usize = 0;
    /// The stab under way: its leg plus one (low byte), stabs left in the
    /// flurry (next byte), legs already used in it (next).
    pub const STAB: usize = 1;
    /// The Brood guard: the fighter it names plus one (low byte), and the
    /// frames it has left (high half).
    pub const GUARD: usize = 2;
    /// Fighters a web glob has rooted, a bit each.
    pub const WEBBED: usize = 3;

    pub const POPPED_SHIFT: u32 = 8;
    pub const STRUCK_SHIFT: u32 = 16;
    pub const ENRAGED: u32 = 1 << 24;
}

// ---------------------------------------------------------------------------
// The table
// ---------------------------------------------------------------------------

pub static FIGHT: FightDecl = FightDecl {
    layout: Layout {
        hazards: 10,
        noises: 0,
        objectives: 0,
        own: 6,
    },
    hazards: &HAZARDS,
    // Nothing on her is somewhere to stand: her own knob says so.
    steepest: Some(Knob::Steepest as u16),
    // The slam lays her abdomen down on whoever is under it.
    lands_on_bodies: true,
    frame: Some(frame),
    appetite: Some(super::mind::appetite),
    commit: Some(super::mind::commit),
    struck: Some(struck),
    hide: Some(hide),
    presence: Some(presence),
    ..FightDecl::PLAIN
};

// ---------------------------------------------------------------------------
// Reading her
// ---------------------------------------------------------------------------

/// A sac site's state and clock, from the lore.
pub fn site(w: &World, i: usize) -> (u32, i32) {
    let v = w.lore.word(word::SAC0 + i);
    ((v >> 16) & 0xFF, (v & 0xFFFF) as i32)
}

fn set_site(w: &mut World, i: usize, state: u32, clock: i32) {
    w.lore.set_word(
        word::SAC0 + i,
        (state & 0xFF) << 16 | clock.clamp(0, 0xFFFF) as u32,
    );
}

/// How long a sac swells, in this fight: shorter with two hunters.
pub fn ripen(w: &World) -> i32 {
    let solo = Knob::SacRipen.raw().max(1);
    if w.players.iter().filter(|p| p.health > 0).count() > 1 {
        Fx::from_int(solo)
            .mul(Knob::SacRipenCoop.fx())
            .to_int()
            .max(1)
    } else {
        solo
    }
}

/// How ripe a swelling sac is, nought to one.
pub fn ripeness(w: &World, i: usize) -> Fx {
    match site(w, i) {
        (sac::SWELLING, clock) => Fx::from_int(clock)
            .div(Fx::from_int(ripen(w)))
            .clamp(Fx::ZERO, Fx::ONE),
        (sac::HELD, _) => Fx::ONE,
        _ => Fx::ZERO,
    }
}

/// Is a sac red -- in the last `SacRed` frames before it bursts, or held?
pub fn red(w: &World, i: usize) -> bool {
    match site(w, i) {
        (sac::SWELLING, clock) => clock >= ripen(w) - Knob::SacRed.raw(),
        (sac::HELD, _) => true,
        _ => false,
    }
}

/// Is she enraged: every sac popped, the collapse done or under way?
pub fn enraged(m: &Monster) -> bool {
    m.own[body::SACS] as u32 & body::ENRAGED != 0
}

/// How many broodlings are alive.
pub fn brood(w: &World) -> usize {
    w.critters.iter().filter(|c| c.alive()).count()
}

/// The world middle of sac `i`.
pub fn sac_middle(m: &Monster, i: usize) -> V3 {
    let part = sac_part(i);
    let sh = m.sp().shape(part);
    let mid = |a: Fx, b: Fx| a.add(b).mul(Fx::ratio(1, 2));
    m.world_of(
        part,
        V3::new(
            mid(sh.min.x, sh.max.x),
            mid(sh.min.y, sh.max.y),
            mid(sh.min.z, sh.max.z),
        ),
    )
}

/// **Where a sac's brood land**: the floor under it, kept inside the arena.
/// The ring drawn under a red sac is this point, and the burst spawns here --
/// one function, so what is drawn is where they land.
pub fn landing(w: &World, m: &Monster, i: usize) -> V3 {
    let at = sac_middle(m, i);
    let b = w.arena().bounds;
    let keep = crate::tuning::body_radius();
    V3::new(
        at.x.clamp(b.lo_x.add(keep), b.hi_x.sub(keep)),
        Fx::ZERO,
        at.z.clamp(b.lo_z.add(keep), b.hi_z.sub(keep)),
    )
}

// ---------------------------------------------------------------------------
// The body's hooks
// ---------------------------------------------------------------------------

/// For the reach instrument (`reachcheck::reach_with`): only sac `i` on her
/// back, so a swing that touches two is asked about the one being measured.
pub fn only_sac(m: &mut Monster, i: usize) {
    let others = 0x3F & !(1 << i);
    m.own[body::SACS] = (m.own[body::SACS] & !0x3F) | others;
}

/// **Which parts have no body**: a sac site that is empty or scarred.
pub fn presence(m: &Monster, _rig: &crate::beast::Rig) -> crate::beast::Presence {
    let gone = m.own[body::SACS] as u32 & 0x3F;
    let mut buried = 0u64;
    for i in 0..SAC_COUNT {
        if gone & 1 << i != 0 {
            buried |= 1u64 << sac_part(i);
        }
    }
    crate::beast::Presence {
        buried,
        unmountable: 0,
    }
}

/// **What a hit on one of her parts does**, before the shared ladder. A hit
/// on a sac goes to the sac and not to her: her health and strain are given
/// back, the sac's own health takes it, and a sac that runs out **pops** --
/// `PopDamage` to her, `PopStrain` of strain (which may be the interrupt), and
/// a flinch. Either way the ladder is skipped: a sac is not a leg, and
/// breaking one does not stumble her.
pub fn struck(m: &mut Monster, part: usize, dealt: i32) -> bool {
    let Some(i) = sac_site(part) else {
        return false;
    };
    // Give back what the shared path took off her.
    m.health += dealt;
    m.strain = (m.strain - dealt).max(0);
    let Some(slot) = m.sp().break_slot(part) else {
        return true;
    };
    if m.own[body::SACS] & (1 << i) != 0 || m.breaks[slot] <= 0 {
        return true;
    }
    m.own[body::SACS] |= 1 << (body::STRUCK_SHIFT + i as u32);
    m.breaks[slot] -= dealt;
    if m.breaks[slot] > 0 {
        return true;
    }
    // Popped.
    m.breaks[slot] = 0;
    m.own[body::SACS] |= 1 << i | 1 << (body::POPPED_SHIFT + i as u32);
    let hurt = Knob::PopDamage.raw().max(0).min(m.health);
    m.health -= hurt;
    m.strain += Knob::PopStrain.raw().max(0);
    if m.health <= 0 {
        m.doing = Doing::Dead;
        return true;
    }
    // The interrupt the shared ladder would have read, and otherwise the pop's
    // own flinch -- never out of a live hit, a stumble or a collapse.
    if m.strain >= m.interrupt_bar() && m.doing.attacking().is_some() {
        m.strain = 0;
        m.doing = Doing::Flinch {
            left: m.sp().flinch_frames(),
        };
    } else if !matches!(
        m.doing,
        Doing::Active { .. } | Doing::Toppled { .. } | Doing::Stumble { .. } | Doing::Dead
    ) {
        m.doing = Doing::Flinch {
            left: Knob::PopFlinch.raw().max(1) as u16,
        };
    }
    true
}

/// **What her hide is worth on a part this frame**: the waist is soft only
/// while she is down -- stumbling, collapsed -- or, enraged, while the slam's
/// recovery has it at a fighter's chest.
pub fn hide(m: &Monster, part: usize) -> Fx {
    if part != super::PEDICEL_PART {
        return Fx::ONE;
    }
    let low = match m.doing {
        Doing::Stumble { .. } | Doing::Toppled { .. } => true,
        Doing::Recovery { kind, .. } => kind == super::SLAM && enraged(m),
        _ => false,
    };
    if low { Knob::PedicelLow.fx() } else { Fx::ONE }
}

// ---------------------------------------------------------------------------
// The frame
// ---------------------------------------------------------------------------

/// Once a frame, after the creatures and the brood have moved: the sac
/// clock, the brood it feeds, the Brood guard, and her arc.
pub fn frame(w: &mut World) {
    let Some(mut m) = w.monsters[0].filter(|m| m.species == SPECIES.id) else {
        return;
    };
    if w.lore.word(word::FLAGS) & flag::SET_UP == 0 {
        set_up(w, &mut m);
    }
    if m.alive() {
        take_in_pops(w, &mut m);
        run_the_clocks(w, &mut m);
        guard(w, &mut m);
        arc(w, &mut m);
        super::mind::work_out_recall(&mut m, &w.critters, &w.players);
    }
    w.monsters[0] = Some(m);
}

/// The first frame of a round: the clocks staggered so a burst comes every
/// few seconds from a fresh mother -- the first already near red -- every sac
/// at full health, and the first of the brood out.
fn set_up(w: &mut World, m: &mut Monster) {
    // Ripeness at the start, in hundredths, in site order: the reddest first.
    const STAGGER: [i32; SAC_COUNT] = [80, 65, 50, 30, 15, 0];
    let r = ripen(w);
    for (i, share) in STAGGER.iter().enumerate() {
        set_site(w, i, sac::SWELLING, r * share / 100);
        reset_sac(m, i);
    }
    let at = V3::new(m.pos.x, Fx::ZERO, m.pos.z);
    let yaw = crate::pack::yaw_of(V3::from_turns(m.yaw));
    if let Some(pack) = w.pack.as_mut() {
        for k in 0..Knob::BroodAtStart.raw().max(0) {
            let quarter = 2 * k + 1;
            let side = V3::from_turns(m.yaw.add(Fx::ratio(quarter, 4)));
            let spot = at.add(side.scale(m.sp().prowl_range()));
            crate::pack::spawn(
                pack,
                &mut w.critters,
                crate::species::gnawers::GNAWER,
                spot,
                yaw,
            );
        }
    }
    let flags = w.lore.word(word::FLAGS) | flag::SET_UP;
    w.lore.set_word(word::FLAGS, flags);
}

/// A sac with a body again, at full health.
fn reset_sac(m: &mut Monster, i: usize) {
    if let Some(slot) = m.sp().break_slot(sac_part(i)) {
        m.breaks[slot] = Knob::SacHealth.raw().max(1);
    }
    m.own[body::SACS] &= !(1 << i);
}

/// The sacs her `struck` hook popped since last frame: scars, for good.
fn take_in_pops(w: &mut World, m: &mut Monster) {
    let popped = (m.own[body::SACS] >> body::POPPED_SHIFT) & 0x3F;
    for i in 0..SAC_COUNT {
        if popped & 1 << i != 0 {
            set_site(w, i, sac::SCARRED, 0);
        }
    }
    m.own[body::SACS] &= !(0x3F << body::POPPED_SHIFT);
}

/// Every site's clock: swell, hold while the brood is full, burst, lay again.
fn run_the_clocks(w: &mut World, m: &mut Monster) {
    if enraged(m) {
        return;
    }
    let r = ripen(w);
    for i in 0..SAC_COUNT {
        let (state, clock) = site(w, i);
        match state {
            sac::SWELLING if clock + 1 >= r => {
                if !burst(w, m, i) {
                    set_site(w, i, sac::HELD, clock + 1);
                }
            }
            sac::SWELLING => set_site(w, i, sac::SWELLING, clock + 1),
            sac::HELD => {
                burst(w, m, i);
            }
            sac::EMPTY if clock + 1 >= Knob::SacRegrow.raw() => {
                set_site(w, i, sac::SWELLING, 0);
                reset_sac(m, i);
            }
            sac::EMPTY => set_site(w, i, sac::EMPTY, clock + 1),
            _ => {}
        }
    }
}

/// **Sac `i` bursts**, if the brood has room for what is in it: its brood
/// dropped under it, a splash on whoever is there, and an empty site. False,
/// and nothing happens, while the brood is full: the sac holds.
fn burst(w: &mut World, m: &mut Monster, i: usize) -> bool {
    let per = Knob::BroodPerSac.raw().max(0) as usize;
    let cap = Knob::BroodCap.raw().max(0) as usize;
    let free = w
        .critters
        .iter()
        .filter(|c| !c.present() || (c.state == crate::critter::is::DEAD && c.timer == 0))
        .count();
    if brood(w) + per > cap || free < per {
        return false;
    }
    let at = landing(w, m, i);
    let Some(pack) = w.pack.as_mut() else {
        return false;
    };
    let yaw = crate::pack::yaw_of(V3::from_turns(m.yaw));
    for k in 0..per {
        // Half a body apart, so two are not born inside each other.
        let off = V3::from_turns(m.yaw.add(Fx::ratio(k as i32, per.max(1) as i32)))
            .scale(crate::tuning::body_radius());
        crate::pack::spawn(
            pack,
            &mut w.critters,
            crate::species::gnawers::GNAWER,
            at.add(off),
            yaw,
        );
    }
    splash(w, at);
    set_site(w, i, sac::EMPTY, 0);
    m.own[body::SACS] |= 1 << i;
    true
}

/// The hatch's splash: whoever is standing under a sac as it bursts.
fn splash(w: &mut World, at: V3) {
    let a = SPECIES.attack(super::HATCH);
    for i in 0..MAX_PLAYERS {
        let p = w.players[i];
        if p.health <= 0 || p.action.invulnerable() || p.aboard() {
            continue;
        }
        let away = V3::new(p.pos.x.sub(at.x), Fx::ZERO, p.pos.z.sub(at.z));
        let gap = math::wide_flat_len(away);
        if gap.raw() > a.hit_radius.add(crate::tuning::body_radius()).raw()
            || p.pos.y.raw() > a.hit_high.raw()
        {
            continue;
        }
        let dir = if gap.raw() > 0 {
            math::wide_normalized(away)
        } else {
            V3::from_turns(Fx::ZERO)
        };
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
                dir,
                blocked: false,
                parried: false,
                interrupts: true,
            },
        );
    }
}

/// **The Brood guard**: a sac struck names whoever struck it -- the nearest
/// fighter to it -- for `GuardFrames`. Her pack's mind turns every broodling
/// within `GuardRadius` of her on them, with a token more.
fn guard(w: &mut World, m: &mut Monster) {
    let struck = (m.own[body::SACS] >> body::STRUCK_SHIFT) & 0x3F;
    m.own[body::SACS] &= !(0x3F << body::STRUCK_SHIFT);
    let left = (m.own[body::GUARD] as u32 >> 16) as i32;
    if left > 0 {
        let who = m.own[body::GUARD] & 0xFF;
        m.own[body::GUARD] = who | (left - 1) << 16;
    } else {
        m.own[body::GUARD] = 0;
    }
    let Some(i) = (0..SAC_COUNT).find(|i| struck & 1 << i != 0) else {
        return;
    };
    let at = sac_middle(m, i);
    let nearest = (0..MAX_PLAYERS)
        .filter(|p| w.players[*p].health > 0)
        .min_by_key(|p| math::wide_flat_len(w.players[*p].pos.sub(at)).raw());
    if let Some(who) = nearest {
        m.own[body::GUARD] = (who as i32 + 1) | Knob::GuardFrames.raw().max(0) << 16;
    }
}

/// Who the Brood guard names, if anybody.
pub fn guarded(m: &Monster) -> Option<usize> {
    let who = m.own[body::GUARD] & 0xFF;
    (who > 0 && (m.own[body::GUARD] as u32 >> 16) > 0).then(|| who as usize - 1)
}

/// **Her arc**: the clutch below `ClutchHealth` with sacs left, and the
/// collapse when the last one pops.
fn arc(w: &mut World, m: &mut Monster) {
    let flags = w.lore.word(word::FLAGS);
    let scarred = (0..SAC_COUNT)
        .filter(|i| site(w, *i).0 == sac::SCARRED)
        .count();
    // The collapse: every sac gone. The brood break and leave by the walls,
    // and what gets up is enraged.
    if scarred == SAC_COUNT && flags & flag::ENRAGED == 0 {
        w.lore.set_word(word::FLAGS, flags | flag::ENRAGED);
        m.own[body::SACS] |= body::ENRAGED as i32;
        m.doing = Doing::Toppled {
            left: Knob::CollapseFrames.raw().max(1) as u16,
        };
        m.speed = Fx::ZERO;
        m.yaw_rate = Fx::ZERO;
        m.strain = 0;
        // They scatter for the far wall, away from her.
        let b = w.arena().bounds;
        let x = if m.pos.x.raw() > 0 { b.lo_x } else { b.hi_x };
        if let Some(pack) = w.pack.as_mut() {
            pack.mood = crate::pack::mood::BROKEN;
            pack.home = V3::new(x, Fx::ZERO, m.pos.z);
        }
        return;
    }
    // The clutch: below its health with any sac still alive, every swelling
    // sac goes red together.
    let full = Fx::from_int(m.sp().health());
    let low = Fx::from_int(m.health).raw() < full.mul(Knob::ClutchHealth.fx()).raw();
    if low && flags & flag::CLUTCH == 0 && scarred < SAC_COUNT {
        w.lore.set_word(word::FLAGS, flags | flag::CLUTCH);
        let r = ripen(w);
        let soon = (r - Knob::ClutchFrames.raw()).max(0);
        for i in 0..SAC_COUNT {
            if let (sac::SWELLING, clock) = site(w, i) {
                set_site(w, i, sac::SWELLING, clock.max(soon));
            }
        }
    }
    // Enraged: faster, and quicker to decide.
    if enraged(m) {
        let walk = m.sp().walk().max(Fx::ONE);
        if m.slowed <= 2 {
            m.slowed = 2;
            m.slow_mul = Knob::EnrageSpeed.fx().div(walk);
        }
        if m.doing.free() && m.brain.think_left == m.sp().think_frames() {
            m.brain.think_left = Fx::from_int(m.brain.think_left as i32)
                .mul(Knob::EnrageThink.fx())
                .to_int()
                .max(0) as u16;
        }
    }
}
