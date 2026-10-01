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

/// A web patch: a disc of sticky floor where a glob landed. A crouch walk
/// through it; it roots nobody.
pub const PATCH: u8 = 0;
/// A strand: a line a web line left behind, a shin's height off the floor.
/// It slows nobody; what it does to a run is her rules' (`trip`).
pub const STRAND: u8 = 1;
/// The glob's catch: a root under the feet of whoever a web shot struck, for
/// `Life` frames -- less every attack they throw (`cut_free`).
pub const GLOB: u8 = 2;
/// Fire on the web: a patch or a glob burning away.
pub const FLASH: u8 = 3;
/// Fire along a strand.
pub const FLARE: u8 = 4;

pub const HAZARDS: [HazardDecl; 5] = [
    HazardDecl::disc("web patch")
        .reaching(reach::FIGHTERS)
        .ignites_into(FLASH),
    HazardDecl::strand("strand")
        .reaching(reach::FIGHTERS)
        .ignites_into(FLARE),
    HazardDecl::disc("glob")
        .reaching(reach::FIGHTERS)
        .ignites_into(FLASH),
    HazardDecl::disc("burning web").burning(),
    HazardDecl::strand("burning strand").burning(),
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
    /// The web line under way: where it set off from, in centimetres
    /// (`lore::halves`).
    pub const LINE_FROM: usize = 7;
    /// Fighters the line under way has struck, a bit each.
    pub const LINE_STRUCK: usize = 8;
    /// Each fighter's action last frame, a byte each (`Action::tag`): an
    /// attack starting is one that was not under way.
    pub const LAST_ACTION: usize = 9;
    /// Fighters a glob holds, a bit each, and the frames they were webbed
    /// down from the air for, in the high half: the glob is laid where they
    /// land.
    pub const WEBBED: usize = 10;
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
    /// flurry (next byte), legs already used in it (next). The top byte is
    /// the web anchors a line could reach from where she stands, a bit each
    /// ([`super::anchors_clear`]).
    pub const STAB: usize = 1;
    /// The Brood guard: the fighter it names plus one (low byte), and the
    /// frames it has left (high half).
    pub const GUARD: usize = 2;
    /// Fighters a web glob has rooted, a bit each (low byte); the screech's
    /// appetite above it (`mind::work_out_recall`).
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
        hazards: 12,
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
    landed: Some(landed),
    signs: Some(signs),
    appetite: Some(super::mind::appetite),
    commit: Some(super::mind::commit),
    struck: Some(struck),
    hide: Some(hide),
    presence: Some(presence),
    repose: Some(super::legs::repose),
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
    // Down in the slam, she stays down: the window runs out its length, and
    // strain past the bar is still there to cancel what she starts next (§4,
    // "popping two in one slam window buys the interrupt").
    if matches!(
        m.doing,
        Doing::Recovery {
            kind: super::SLAM,
            ..
        }
    ) {
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
        flurry(&mut m);
        take_in_pops(w, &mut m);
        run_the_clocks(w, &mut m);
        guard(w, &mut m);
        arc(w, &mut m);
        web_shot(w, &mut m);
        web_line(w, &mut m);
        anchors_clear(w, &mut m);
        super::mind::work_out_recall(&mut m, &w.critters, &w.players);
    }
    webbed(w, &mut m);
    strands(w);
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

/// **The flurry**: on the last frame of a stab's hit, if more are due,
/// another leg -- one not yet used, and sound -- winds up at once, its own
/// windup (the flurry stab's row) behind it. Out of a stab altogether, the stab's words are cleared.
fn flurry(m: &mut Monster) {
    use super::{FLURRY, STAB};
    let kind = m.doing.attacking();
    if !matches!(kind, Some(STAB | FLURRY)) {
        // Out of a stab: its words cleared, the anchors' kept.
        m.own[body::STAB] &= !0x00FF_FFFF;
        return;
    }
    let Doing::Active { left: 0, .. } = m.doing else {
        return;
    };
    let words = m.own[body::STAB] as u32;
    let left = (words >> 8) & 0xFF;
    if left == 0 {
        return;
    }
    let windup = m.sp().attack(FLURRY).startup.max(1);
    let lead = m.lead_point(windup);
    if !super::mind::stab_reaches(m, lead) {
        return;
    }
    let Some((leg, disc)) = super::mind::stab_leg(m, lead) else {
        return;
    };
    let used = (words >> 16) & 0xFF | 1 << leg;
    let keep = words & 0xFF00_0000;
    m.own[body::STAB] = (keep | (leg as u32 + 1) | (left - 1) << 8 | used << 16) as i32;
    m.aim_at(disc);
    m.doing = Doing::Startup {
        kind: FLURRY,
        left: windup,
    };
    m.hit_used = false;
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

// ---------------------------------------------------------------------------
// The web
// ---------------------------------------------------------------------------

/// Her spinnerets: the tip of the abdomen, at the middle of its height.
pub fn spinnerets(m: &Monster) -> V3 {
    let sh = m.sp().shape(super::ABDOMEN_PART);
    let mid = sh.min.y.add(sh.max.y).mul(Fx::ratio(1, 2));
    m.world_of(super::ABDOMEN_PART, V3::new(sh.min.x, mid, Fx::ZERO))
}

/// The world as a scene to ask a straight line through: the arena and what
/// is raised on it, the stones (and a planted shield, which is one). No
/// bodies.
fn scene_of<R>(w: &World, ask: impl FnOnce(&crate::aim::Scene) -> R) -> R {
    let field = crate::stones::gather(&w.players);
    let ground = w.terrain();
    let fighters = w.players;
    let effects = w.effects;
    let critters = w.critters;
    let scene = crate::aim::Scene {
        stones: &field,
        players: &fighters,
        effects: &effects,
        quarry: &[None; crate::monster::MAX_MONSTERS],
        critters: &critters,
        arena: &ground,
    };
    ask(&scene)
}

/// **Where a glob lands**: its aim, unless a solid stands between the
/// spinnerets and it -- a pillar, a stone, a planted shield -- in which case
/// it lands at the solid. The answer to the web shot, and the one the
/// telegraph and the hit both use: on the hit's first frame the aim point is
/// moved there, so the disc drawn through the windup's last frame and the
/// volume that lands are the same disc.
pub fn glob_lands(w: &World, m: &Monster) -> V3 {
    let aim = m.aimed_at();
    let a = m.sp().attack(super::WEB_SHOT);
    let from = spinnerets(m);
    let to = V3::new(aim.x, a.hit_high.mul(Fx::ratio(1, 2)), aim.z);
    let hit = scene_of(w, |scene| {
        crate::aim::first_along(
            crate::aim::Path { from, to },
            Knob::GlobRadius.fx().mul(Fx::ratio(1, 2)),
            state::NOBODY,
            scene,
            crate::aim::Targets::none().stones().terrain(),
        )
    });
    match hit {
        Some(c) => {
            let along = math::wide_normalized(to.sub(from));
            let at = from.add(along.scale(c.dist()));
            // On the near face of what stopped it, on the floor.
            let back = V3::new(along.x, Fx::ZERO, along.z).scale(Knob::GlobRadius.fx());
            V3::new(at.x.sub(back.x), Fx::ZERO, at.z.sub(back.z))
        }
        None => aim,
    }
}

/// The web shot, as its hit comes out: the glob stopped by whatever stands
/// in its way, and a patch where it lands.
fn web_shot(w: &mut World, m: &mut Monster) {
    let a = m.sp().attack(super::WEB_SHOT);
    let Doing::Active { kind, left } = m.doing else {
        return;
    };
    if kind != super::WEB_SHOT || left != a.active {
        return;
    }
    let at = glob_lands(w, m);
    m.aim_at(at);
    crate::hazard::place(
        &mut w.lore,
        crate::hazard::Hazard::disc(PATCH, at, Knob::PatchRadius.fx()),
    );
}

/// **A glob landed on somebody** (`FightDecl::landed`): rooted where they
/// stand -- a glob under their feet -- or, caught in the air, webbed down to
/// the floor and rooted where they land.
pub fn landed(w: &mut World, _slot: usize, who: usize, kind: u8, guarded: bool) {
    if kind != super::WEB_SHOT || guarded || who >= MAX_PLAYERS {
        return;
    }
    let held = w.lore.word(word::WEBBED) | 1 << who;
    w.lore.set_word(word::WEBBED, held);
}

/// The webbed: brought down out of the air, a glob laid under them once they
/// are on the floor, and let go when it is gone. Every attack they throw cuts
/// `CutFree` frames off it.
fn webbed(w: &mut World, m: &mut Monster) {
    let mut held = w.lore.word(word::WEBBED);
    let mut last = w.lore.word(word::LAST_ACTION);
    let mut caught = 0u32;
    for who in 0..MAX_PLAYERS {
        let p = w.players[who];
        let tag = p.action.tag() & 0xFF;
        let was = (last >> (8 * who)) & 0xFF;
        let started = matches!(p.action, state::Action::Startup { .. }) && tag != was;
        last = (last & !(0xFF << (8 * who))) | tag << (8 * who);
        let glob = crate::hazard::all(&w.lore)
            .find(|(_, h)| h.index() == GLOB && h.state == who as u8 + 1)
            .map(|(i, _)| i);
        if held & 1 << who != 0 {
            if p.health <= 0 {
                held &= !(1 << who);
            } else if !p.grounded {
                // Webbed down: out of the air, at the glob's own speed.
                let down = Knob::WebDown.fx().neg();
                if w.players[who].vel.y.raw() > down.raw() {
                    w.players[who].vel.y = down;
                }
            } else {
                held &= !(1 << who);
                let mut g = crate::hazard::Hazard::disc(
                    GLOB,
                    V3::new(p.pos.x, Fx::ZERO, p.pos.z),
                    crate::tuning::body_radius(),
                );
                g.state = who as u8 + 1;
                crate::hazard::place(&mut w.lore, g);
            }
        }
        if let Some(i) = glob {
            caught |= 1 << who;
            // Cutting free: each attack thrown takes its frames off the root.
            if started {
                let mut g = crate::hazard::get(&w.lore, i);
                let life = crate::hazard::stat(m.sp(), GLOB, crate::hazard::HazardField::Life);
                let aged = g.age as i32 + Knob::CutFree.raw().max(0);
                if aged >= life {
                    crate::hazard::clear(&mut w.lore, i);
                    caught &= !(1 << who);
                } else {
                    g.age = aged as u16;
                    crate::hazard::set(&mut w.lore, i, g);
                }
            }
        }
    }
    w.lore.set_word(word::WEBBED, held);
    w.lore.set_word(word::LAST_ACTION, last);
    m.own[body::WEBBED] = (m.own[body::WEBBED] & !0xFF) | (caught | held) as i32;
}

/// **Which web anchors a line could reach from where she stands**: in reach
/// and with nothing solid on the way -- a pillar is the one place a line
/// cannot cross. Kept on her body for the brain (`mind::anchor_for`).
fn anchors_clear(w: &World, m: &mut Monster) {
    let reach = Knob::LineReach.fx();
    let from = V3::new(m.pos.x, crate::tuning::body_height(), m.pos.z);
    let mut bits = 0u32;
    for (k, site) in w
        .arena()
        .sites
        .iter()
        .filter(|s| s.name == "anchor")
        .take(8)
        .enumerate()
    {
        let (at, _) = site.at(Fx::ZERO);
        let to = V3::new(at.x, from.y, at.z);
        let near = math::wide_flat_dist(at, m.pos).raw() <= reach.raw();
        if near && scene_of(w, |scene| crate::aim::line_clear(from, to, scene)) {
            bits |= 1 << k;
        }
    }
    m.own[body::STAB] = (m.own[body::STAB] & 0x00FF_FFFF) | (bits << 24) as i32;
}

/// The web line: through its windup she turns her back to the anchor; on
/// the hit she is reeled across, abdomen first, at `LineSpeed`, striking
/// whoever is in her lane, until she reaches the anchor's wall or the line's
/// reach -- and two strands stay behind her.
fn web_line(w: &mut World, m: &mut Monster) {
    let anchor = m.aimed_at();
    match m.doing {
        Doing::Startup {
            kind: super::WEB_LINE,
            left,
        } => {
            // Her back to it.
            let away = V3::new(m.pos.x.sub(anchor.x), Fx::ZERO, m.pos.z.sub(anchor.z));
            if math::wide_flat_len(away).raw() > 0 {
                let want = math::atan2_turns(away.z, away.x);
                let err = math::wrap_turns(want.sub(m.yaw));
                let step = m.sp().turn_rate_max().mul(crate::DT);
                m.yaw = m.yaw.add(err.clamp(step.neg(), step));
            }
            if left == m.sp().attack(super::WEB_LINE).startup {
                let from =
                    crate::lore::halves(crate::lore::to_cm(m.pos.x), crate::lore::to_cm(m.pos.z));
                w.lore.set_word(word::LINE_FROM, from);
                w.lore.set_word(word::LINE_STRUCK, 0);
            }
        }
        Doing::Active {
            kind: super::WEB_LINE,
            ..
        } => {
            let from = w.lore.word(word::LINE_FROM);
            let start = V3::new(
                crate::lore::from_cm(crate::lore::lo(from)),
                Fx::ZERO,
                crate::lore::from_cm(crate::lore::hi(from)),
            );
            let to = V3::new(anchor.x.sub(m.pos.x), Fx::ZERO, anchor.z.sub(m.pos.z));
            let left_to_go = math::wide_flat_len(to);
            let step = Knob::LineSpeed.fx().mul(crate::DT);
            let stop = m.sp().margin();
            let gone = math::wide_flat_dist(m.pos, start);
            let before = m.pos;
            if left_to_go.raw() > stop.add(step).raw() && gone.raw() < Knob::LineReach.fx().raw() {
                m.pos = m.pos.add(math::wide_normalized(to).scale(step));
                m.speed = Fx::ZERO;
                line_strikes(w, m, before);
            } else {
                // Arrived: the strands, and the recovery.
                lay_strands(w, start, m.pos);
                m.doing = Doing::Recovery {
                    kind: super::WEB_LINE,
                    left: m.sp().attack(super::WEB_LINE).recovery,
                };
            }
        }
        _ => {}
    }
}

/// Whoever her body passed through this frame, in the line's lane, once a
/// line: knocked down.
fn line_strikes(w: &mut World, m: &Monster, from: V3) {
    let a = m.sp().attack(super::WEB_LINE);
    let lane = Knob::LineLane.fx().add(crate::tuning::body_radius());
    let mut struck = w.lore.word(word::LINE_STRUCK);
    for i in 0..MAX_PLAYERS {
        let p = w.players[i];
        if struck & 1 << i != 0 || p.health <= 0 || p.action.invulnerable() {
            continue;
        }
        let gap = math::flat_segment_gap(p.pos, from, m.pos);
        if gap.raw() > lane.raw() || p.pos.y.raw() > a.hit_high.raw().max(m.sp().scale().raw()) {
            continue;
        }
        struck |= 1 << i;
        let away = V3::new(p.pos.x.sub(m.pos.x), Fx::ZERO, p.pos.z.sub(m.pos.z));
        let dir = if math::wide_flat_len(away).raw() > 0 {
            math::wide_normalized(away)
        } else {
            V3::from_turns(m.yaw)
        };
        let facing_it =
            p.facing.dot(dir.scale(Fx::ONE.neg())).raw() >= crate::tuning::guard_arc_cos().raw();
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
                dir,
                blocked: guarding,
                parried: false,
                interrupts: true,
            },
        );
    }
    w.lore.set_word(word::LINE_STRUCK, struck);
}

/// Two strands along the line she crossed, `StrandApart` apart, from where
/// she set off to where she stopped.
fn lay_strands(w: &mut World, from: V3, to: V3) {
    let along = V3::new(to.x.sub(from.x), Fx::ZERO, to.z.sub(from.z));
    if math::wide_flat_len(along).raw() <= 0 {
        return;
    }
    let dir = math::wide_normalized(along);
    let side =
        V3::new(dir.z.neg(), Fx::ZERO, dir.x).scale(Knob::StrandApart.fx().mul(Fx::ratio(1, 2)));
    for s in [side, side.scale(Fx::ONE.neg())] {
        crate::hazard::place(
            &mut w.lore,
            crate::hazard::Hazard::strand(STRAND, from.add(s), to.add(s), Knob::StrandHalf.fx()),
        );
    }
}

/// **The strands**: crossing one faster than `TripSpeed` on the floor, not
/// crouched, trips you; any blow cuts one.
fn strands(w: &mut World) {
    let sp = &SPECIES;
    let high = crate::hazard::stat_fx(sp, STRAND, crate::hazard::HazardField::Height);
    let lines: [Option<(usize, V3, V3)>; crate::hazard::MAX_HAZARDS] = {
        let mut out = [None; crate::hazard::MAX_HAZARDS];
        for (k, (i, h)) in crate::hazard::all(&w.lore)
            .filter(|(_, h)| h.index() == STRAND)
            .enumerate()
        {
            if k < out.len() {
                let a = h.centre();
                let b = V3::new(
                    crate::lore::from_cm(h.to[0]),
                    Fx::ZERO,
                    crate::lore::from_cm(h.to[1]),
                );
                out[k] = Some((i, V3::new(a.x, Fx::ZERO, a.z), b));
            }
        }
        out
    };
    let trip = SPECIES.attack(super::HATCH);
    for (i, a, b) in lines.iter().flatten().copied() {
        let half = crate::hazard::get(&w.lore, i).radius();
        // Any blow cuts it.
        let cut = w.players.iter().any(|p| {
            state::hitbox(p).is_some_and(|hb| {
                let mid = hb.centre();
                let low = hb.from.y.min(hb.to.y).sub(hb.radius);
                low.raw() <= high.raw()
                    && math::flat_segment_gap(mid, a, b).raw() <= hb.radius.add(half).raw()
            })
        });
        if cut {
            crate::hazard::clear(&mut w.lore, i);
            continue;
        }
        for who in 0..MAX_PLAYERS {
            let p = w.players[who];
            if p.health <= 0
                || p.crouching
                || !p.grounded
                || p.pos.y.raw() > high.raw()
                || p.action.stunned()
            {
                continue;
            }
            let speed = V3::new(p.vel.x, Fx::ZERO, p.vel.z).flat_len();
            if speed.raw() <= Knob::TripSpeed.fx().raw() {
                continue;
            }
            let was = p.pos.sub(p.vel.scale(crate::DT));
            if !crosses(was, p.pos, a, b, half) {
                continue;
            }
            let dir = math::wide_normalized(V3::new(p.vel.x, Fx::ZERO, p.vel.z));
            state::apply_hit(
                &mut w.players[who],
                Hit {
                    damage: Knob::TripDamage.raw(),
                    hitstun: Knob::TripStagger.raw().max(0) as u16,
                    blockstun: 0,
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
            let _ = trip;
        }
    }
}

/// Does a step from `p` to `q` cross the line from `a` to `b` (within `half`
/// of it, in the floor plane)?
pub fn crosses(p: V3, q: V3, a: V3, b: V3, half: Fx) -> bool {
    let side = |x: V3| {
        let ab = V3::new(b.x.sub(a.x), Fx::ZERO, b.z.sub(a.z));
        let ax = V3::new(x.x.sub(a.x), Fx::ZERO, x.z.sub(a.z));
        ab.x.mul(ax.z).sub(ab.z.mul(ax.x))
    };
    let (sp, sq) = (side(p), side(q));
    let opposite = (sp.raw() >= 0) != (sq.raw() >= 0);
    // Along the strand, not past its ends.
    let mid = math::lerp3(p, q, Fx::ratio(1, 2));
    opposite
        && math::flat_segment_gap(mid, a, b).raw() <= half.add(crate::tuning::body_radius()).raw()
}

// ---------------------------------------------------------------------------
// What is drawn on the floor
// ---------------------------------------------------------------------------

/// **What she draws on the floor besides her telegraphs** (`FightDecl::signs`,
/// `sim::sign`): a ring under every red sac where its brood will land, the
/// web shot's lane from the spinnerets to where the glob will land, the web
/// line's lane and the anchor it goes to, the slam's footprint through the
/// screech that always leads to it, and a shadow line under every strand.
pub fn signs(w: &World, out: &mut crate::sign::Signs) {
    use crate::sign::{Says, Sign};
    let Some(m) = w.monsters[0].filter(|m| m.species == SPECIES.id && m.alive()) else {
        return;
    };
    // The web, first: it is what comes from behind you.
    match m.doing {
        Doing::Startup {
            kind: super::WEB_SHOT,
            left,
        } => {
            let a = m.sp().attack(super::WEB_SHOT);
            let fill = Fx::ONE.sub(Fx::ratio(left as i32, a.startup.max(1) as i32));
            let at = glob_lands(w, &m);
            let from = spinnerets(&m);
            let from = V3::new(from.x, Fx::ZERO, from.z);
            let along = V3::new(at.x.sub(from.x), Fx::ZERO, at.z.sub(from.z));
            let len = math::wide_flat_len(along);
            if len.raw() > 0 {
                out.push(
                    Sign::strip(
                        Says::Coming,
                        from,
                        math::wide_normalized(along),
                        len,
                        a.hit_radius.add(a.hit_radius),
                    )
                    .filled(fill),
                );
            }
        }
        Doing::Startup {
            kind: super::WEB_LINE,
            left,
        }
        | Doing::Active {
            kind: super::WEB_LINE,
            left,
        } => {
            let a = m.sp().attack(super::WEB_LINE);
            let live = matches!(m.doing, Doing::Active { .. });
            let fill = if live {
                Fx::ONE
            } else {
                Fx::ONE.sub(Fx::ratio(left as i32, a.startup.max(1) as i32))
            };
            let anchor = m.aimed_at();
            let from = V3::new(m.pos.x, Fx::ZERO, m.pos.z);
            let along = V3::new(anchor.x.sub(from.x), Fx::ZERO, anchor.z.sub(from.z));
            let len = math::wide_flat_len(along)
                .sub(m.sp().margin())
                .min(Knob::LineReach.fx())
                .max(Fx::ZERO);
            if len.raw() > 0 {
                let says = if live { Says::Live } else { Says::Coming };
                let lane = Knob::LineLane.fx();
                out.push(
                    Sign::strip(
                        says,
                        from,
                        math::wide_normalized(along),
                        len,
                        lane.add(lane),
                    )
                    .filled(fill),
                );
            }
            out.push(Sign::ring(Says::Stops, anchor, Knob::StrandApart.fx()));
        }
        // The screech always leads to the slam: its footprint, faint, the
        // whole scream long -- the hit test's own answer, asked of her as the
        // slam's hit would find her.
        Doing::Startup {
            kind: super::SCREECH,
            ..
        }
        | Doing::Active {
            kind: super::SCREECH,
            ..
        } => {
            let mut crash = m;
            crash.doing = Doing::Active {
                kind: super::SLAM,
                left: m.sp().attack(super::SLAM).active,
            };
            if let Some((at, r, _, _)) = crash.hit_volume() {
                out.push(Sign::disc(Says::Faint, at, r.add(r)));
            }
        }
        _ => {}
    }
    // A ring under every red sac: where its brood will land, closing as it
    // ripens.
    if !enraged(&m) {
        let r = ripen(w);
        let red_for = Knob::SacRed.raw().max(1);
        let splash = m.sp().attack(super::HATCH).hit_radius;
        for i in 0..SAC_COUNT {
            if !red(w, i) {
                continue;
            }
            let (_, clock) = site(w, i);
            let fill = Fx::ratio((clock - (r - red_for)).clamp(0, red_for), red_for);
            out.push(Sign::ring(Says::Coming, landing(w, &m, i), splash.add(splash)).filled(fill));
        }
    }
    // A shadow line under every strand.
    for (_, h) in crate::hazard::all(&w.lore).filter(|(_, h)| h.index() == STRAND) {
        let a = h.centre();
        let a = V3::new(a.x, Fx::ZERO, a.z);
        let b = V3::new(
            crate::lore::from_cm(h.to[0]),
            Fx::ZERO,
            crate::lore::from_cm(h.to[1]),
        );
        let along = b.sub(a);
        let len = math::wide_flat_len(along);
        if len.raw() > 0 {
            out.push(Sign::strip(
                Says::Faint,
                a,
                math::wide_normalized(along),
                len,
                h.radius().add(h.radius()),
            ));
        }
    }
}
