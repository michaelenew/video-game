//! What the Mireback does to the floor and to the fighters: its hazard kinds,
//! its braziers, its swallow, its coat and its warts, and the rules that tie
//! them together, run once a frame from its `FightDecl::frame` hook.
//!
//! **One list, as P4 promises.** Every pool of tar, every burning pool, every
//! slag mound and every lane of spilled coals is one entry of the hunt's hazard
//! list (`crate::hazard`), so the slow, the burn, the climb and the drawing all
//! read the same entries. What is the Mireback's own is *where* pools land and
//! how they merge, what its moves do to the floor, the braziers, and the
//! creature's own state -- written down in [`word`] (the hunt's lore) and
//! [`body`] (the four words on the creature itself).
//!
//! See `docs/design/creatures/mireback.md` §2, §4 and §10.

use super::{
    BACKWASH, BELCH, BELLY, FLANK_R, FLOP, GAG, INFLATE, JAW_PART, Knob, SAC, SPECIES, SPEW,
    STOMACH, SWALLOW, TONGUE, WALLOW, WARTS, coated, mind,
};
use crate::arena::Material;
use crate::fixed::Fx;
use crate::hazard::{self, Hazard, HazardDecl, HazardField, reach};
use crate::lore::{self, Layout, Lore};
use crate::math::{self, V3};
use crate::monster::{Doing, Monster};
use crate::species::{FightDecl, Mark, MarkLook, Marks, SpeciesId};
use crate::state::{self, Action, Hit, MAX_PLAYERS, QUARRY, World};
use crate::{DT, aim};

// ---------------------------------------------------------------------------
// The floor's kinds
// ---------------------------------------------------------------------------

/// Tar: slows the walk, halves the dodge, shortens the jump. Fire turns it
/// into burning tar.
pub const TAR: u8 = 0;
/// Burning tar: burns whoever stands in it, lights the tar it touches, and
/// leaves slag.
pub const BURNING: u8 = 1;
/// Slag: a mound you stand on, a layer higher for tar burnt on it.
pub const SLAG: u8 = 2;
/// Coals spilled from a tipped brazier: a burning lane that lights the tar
/// it lies across and burns bare floor for a moment.
pub const COALS: u8 = 3;

pub const HAZARDS: [HazardDecl; 4] = [
    // Tar reaches fighters only: the toad stands in it by choice and is not
    // slowed by its own floor.
    HazardDecl::disc("tar")
        .reaching(reach::FIGHTERS | reach::CRITTERS)
        .ignites_into(BURNING)
        .made_of(Material::Peat),
    // Burning tar burns fighters through the shared floor; the toad's own
    // burn is counted by its footprint here (`burn_itself`), not by its
    // middle, so the shared rule does not reach creatures.
    HazardDecl::disc("burning tar")
        .reaching(reach::FIGHTERS | reach::CRITTERS)
        .burning()
        .becomes(SLAG)
        .made_of(Material::Ash),
    HazardDecl::disc("slag").solid(Material::Rock),
    HazardDecl::strand("coals")
        .reaching(reach::FIGHTERS | reach::CRITTERS)
        .burning()
        .made_of(Material::Ash),
];

// ---------------------------------------------------------------------------
// Its state
// ---------------------------------------------------------------------------

/// Its own words in the hunt's lore: eight cells, thirty-two words.
pub mod word {
    /// Bit 0: the round has been set up.
    pub const FLAGS: usize = 0;
    /// One word per brazier, by its site's order: zero standing and lit, or
    /// the frames left until it relights.
    pub const BRAZIER: usize = 1;
    pub const BRAZIERS: usize = 4;
    /// The swallow: who plus one (low byte), the phase (next byte: one
    /// pulling, two inside), a hot mouthful (bit 16), and the frames left in
    /// the phase (high half... kept in [`SWALLOW_LEFT`]).
    pub const SWALLOW: usize = 5;
    pub const SWALLOW_LEFT: usize = 6;
    /// Where a belly flop left from, in centimetres: `lore::halves`.
    pub const LEAP: usize = 7;
    /// Damage it has taken recently from somebody close: what the Inflate's
    /// `crowd` term reads. Bleeds away over about a second.
    pub const CROWD: usize = 8;
    /// Its health at the end of the last frame, to see what the frame did.
    pub const HEALTH: usize = 9;
    /// The tide's second glob has been thrown for this Spew.
    pub const SECOND: usize = 10;
    /// Which hazard slots were burning at the end of the last frame: a bit
    /// each, so a pool that has just caught can be told from one that was
    /// already alight.
    pub const BURNING: usize = 11;
    /// Which slag mounds have tar on them: a bit per slot. Burnt, they grow.
    pub const COATED: usize = 12;

    // ---- what the report counts (`hunt`'s Mireback lines) ----
    /// Fires started, a byte each: by a brazier, by the belch, by a
    /// backfire, by a fighter's own fire.
    pub const LIT: usize = 16;
    /// All the damage its own fire has done it.
    pub const SELF_BURN: usize = 17;
    /// Swallows (low half) and hot mouthfuls (high half).
    pub const SWALLOWS: usize = 18;
    /// Damage done to its stomach from inside and out while it held somebody.
    pub const STOMACH_DEALT: usize = 19;
    /// Acid the swallowed took.
    pub const ACID: usize = 20;
    /// Times gutted (low half), wallows broken (high half).
    pub const GUTTED: usize = 21;
    /// Slag mounds made, and layers added to one.
    pub const SLAG_BUILT: usize = 22;
    /// Frames it spent with its coat more than half there.
    pub const COAT_FRAMES: usize = 23;
    /// Pools it laid.
    pub const POOLS: usize = 24;
    /// Times it was winded (low half), backfires (high half).
    pub const WINDED: usize = 25;
}

/// Its four words on the body (`Monster::own`), which the hit path reads.
pub mod body {
    /// **How much of its coat is gone**, as a fraction in 16.16: zero is a
    /// whole coat. Kept as what is missing so a creature fresh from
    /// `Monster::new` -- every word zero -- has its coat.
    pub const COAT_LOST: usize = 0;
    /// The throat sac's poise, filled by hits on it while it is out.
    pub const SAC: usize = 1;
    /// Damage to its stomach during this swallow.
    pub const STOMACH: usize = 2;
    /// Things that happened to the body this frame, for the frame hook to
    /// act on with the whole world: see [`super::event`].
    pub const EVENTS: usize = 3;
}

/// The bits of [`body::EVENTS`]: flags, not quantities.
pub mod event {
    /// One bit of the word.
    pub type Bit = i32;
    /// The stumble it is in is a winding (its sac tore, or a hot mouthful):
    /// any other stumble is a knock-up it shrugs into a flinch.
    pub const WINDED: Bit = 1;
    /// The belch went off in its mouth.
    pub const BACKFIRE: Bit = 1 << 1;
    /// A wallow broken by damage.
    pub const WALLOW_BROKEN: Bit = 1 << 2;
    /// The stomach has taken enough: let them out.
    pub const RETCH: Bit = 1 << 3;
    /// A wart burst this frame.
    pub const BURST: Bit = 1 << 4;
}

/// The swallow's phases.
const PULLING: u32 = 1;
const INSIDE: u32 = 2;
const HOT: u32 = 1 << 16;

// ---------------------------------------------------------------------------
// The table
// ---------------------------------------------------------------------------

pub static FIGHT: FightDecl = FightDecl {
    layout: Layout {
        hazards: 16,
        noises: 0,
        objectives: 0,
        own: 8,
    },
    hazards: &HAZARDS,
    lands_on_bodies: true,
    rolls_over: true,
    frame: Some(frame),
    appetite: Some(mind::appetite),
    prowl_to: Some(mind::prowl_to),
    commit: Some(mind::commit),
    hide: Some(hide),
    struck: Some(struck),
    landed: Some(landed),
    marks: Some(marks),
    ..FightDecl::PLAIN
};

// ---------------------------------------------------------------------------
// Reading it
// ---------------------------------------------------------------------------

/// How much of its coat is left, nought to one.
pub fn coat(m: &Monster) -> Fx {
    Fx::ONE
        .sub(Fx::from_raw(m.own[body::COAT_LOST]))
        .clamp(Fx::ZERO, Fx::ONE)
}

fn lose_coat(m: &mut Monster, by: Fx) {
    m.own[body::COAT_LOST] = Fx::from_raw(m.own[body::COAT_LOST])
        .add(by)
        .clamp(Fx::ZERO, Fx::ONE)
        .raw();
}

/// How many of its warts are burst.
pub fn burst(m: &Monster) -> i32 {
    WARTS.iter().filter(|w| m.broken(**w)).count() as i32
}

/// Gutted: on its back, in its own fire. What the shared ladder calls a topple.
pub fn gutted(m: &Monster) -> bool {
    matches!(m.doing, Doing::Toppled { .. })
}

/// Winded: its sac torn, sagging. What the shared ladder calls a stumble.
pub fn winded(m: &Monster) -> bool {
    matches!(m.doing, Doing::Stumble { .. }) && m.own[body::EVENTS] & event::WINDED != 0
}

/// Is the throat sac out, where a blow can reach it?
pub fn sac_shown(m: &Monster) -> bool {
    matches!(m.doing.attacking(), Some(INFLATE | BELCH | GAG)) || winded(m)
}

/// The tide: under its last few tenths, it stops protecting itself.
pub fn tide(m: &Monster) -> bool {
    let full = Fx::from_int(m.sp().health());
    Fx::from_int(m.health).raw() < full.mul(Knob::TideHealth.fx()).raw()
}

/// How far out its footprint reaches from its middle: the flank's outer face.
pub fn foot_radius(m: &Monster) -> Fx {
    m.sp().shape(FLANK_R).max.z
}

/// Its mouth: the front of the lower jaw, where the tongue leaves and the
/// belch is measured from.
pub fn mouth(m: &Monster) -> V3 {
    let sh = m.sp().shape(JAW_PART);
    m.world_of(JAW_PART, V3::new(sh.max.x, sh.max.y, Fx::ZERO))
}

/// Is a disc of the floor under its footprint?
fn under_it(m: &Monster, h: &hazard::Placed) -> bool {
    math::wide_flat_dist(h.a, m.pos).raw() < h.radius.add(foot_radius(m)).raw()
}

/// The hazards of a kind under its footprint.
pub fn pools_under(m: &Monster, lore: &Lore, herd: &crate::monster::Herd, kind: u8) -> usize {
    hazard::Floor::build(lore, herd)
        .iter()
        .filter(|h| h.kind == kind && under_it(m, h))
        .count()
}

/// How much more tar a fight lays than one hunter's, by area: half again with
/// two, a quarter less for every burst wart.
fn tar_share(w: &World, m: &Monster) -> Fx {
    let hunters = w.players.iter().filter(|p| p.health > 0).count();
    let coop = if hunters > 1 {
        Knob::TarCoop.fx()
    } else {
        Fx::ONE
    };
    let left = Fx::from_int(WARTS.len() as i32 - burst(m)).div(Fx::from_int(WARTS.len() as i32));
    coop.mul(left)
}

/// A radius grown or shrunk so its *area* is `share` times as much.
fn by_area(r: Fx, share: Fx) -> Fx {
    r.mul(math::sqrt(share))
}

/// Where the ring of pools a belly flop leaves will lie, if it crashed at
/// `at` facing `yaw`: what is laid on the crash and what is drawn through the
/// crouch -- one answer, so the marker is the floor it will leave.
pub fn ring(m: &Monster, at: V3, coop: Fx) -> ([V3; 8], usize, Fx) {
    let mut out = [V3::ZERO; 8];
    let n = (Knob::RingPools.raw() - burst(m)).clamp(0, out.len() as i32) as usize;
    let r = by_area(Knob::PoolRing.fx(), coop);
    let out_to = foot_radius(m).sub(Knob::RingInside.fx()).add(r);
    for (i, slot) in out.iter_mut().enumerate().take(n) {
        // Evenly round, the first off its shoulder rather than its nose, so
        // the ring leaves a way in straight ahead and straight behind.
        let turn = m.yaw.add(
            Fx::from_int(i as i32)
                .add(math::half(Fx::ONE))
                .div(Fx::from_int(n.max(1) as i32)),
        );
        *slot = at.add(V3::from_turns(turn).scale(out_to));
    }
    (out, n, r)
}

/// The tar pools a belch from where it stands now would light: every one
/// whose disc comes within `reach` of its mouth. What is lit and what is drawn
/// glowing through the windup.
pub fn kindling(m: &Monster, lore: &Lore, herd: &crate::monster::Herd, reach: Fx) -> u32 {
    let from = mouth(m);
    let mut bits = 0u32;
    for h in hazard::Floor::build(lore, herd).iter() {
        if h.kind == TAR && h.flat_gap(from).raw() <= h.radius.add(reach).raw() {
            bits |= 1 << h.slot;
        }
    }
    bits
}

// ---------------------------------------------------------------------------
// The body's hooks
// ---------------------------------------------------------------------------

/// **What its hide is worth on a part.** Whole, the tar coat turns a blow
/// to `coat_armour` of itself; burnt off, the hide takes it in full. Gutted,
/// the whole hide is soft. The throat sac hidden is the jaw over it.
fn hide(m: &Monster, part: usize) -> Fx {
    let soft = m.sp().vulnerability(part);
    if soft.raw() <= 0 {
        return Fx::ONE;
    }
    if gutted(m) && coated(part) {
        return Knob::GuttedSoft.fx().div(soft);
    }
    let coat_mul = Fx::ONE.sub(Fx::ONE.sub(Knob::CoatArmour.fx()).mul(coat(m)));
    if part == SAC && !sac_shown(m) {
        // Tucked inside the jaw: what reaches it is the jaw's hide.
        return Knob::VulnHide.fx().mul(coat_mul).div(soft);
    }
    if coated(part) { coat_mul } else { Fx::ONE }
}

/// **A hit, after health and strain.** The swallow, the sac, the belch and
/// the wallow each take their frame here before the shared ladder.
fn struck(m: &mut Monster, part: usize, dealt: i32) -> bool {
    // Somebody inside: the stomach and the belly count toward the retch, and
    // nothing short of that interrupts it -- it is busy.
    if let Doing::Active { kind: SWALLOW, .. } = m.doing {
        if part == STOMACH || part == BELLY {
            m.own[body::STOMACH] += dealt;
            if m.own[body::STOMACH] >= Knob::SwallowRetch.raw() {
                m.own[body::EVENTS] |= event::RETCH;
            }
        }
        return true;
    }
    // A wart bursting: a quarter of its tar gone for good. A stagger, not a
    // stumble -- it is a gland, not a leg.
    if let Some(slot) = m.sp().break_slot(part) {
        if m.breaks[slot] > 0 && m.breaks[slot] <= dealt {
            m.breaks[slot] = 0;
            m.own[body::EVENTS] |= event::BURST;
            if !matches!(m.doing, Doing::Toppled { .. } | Doing::Stumble { .. }) {
                m.doing = Doing::Flinch {
                    left: Knob::StaggerFrames.raw() as u16,
                };
            }
            return true;
        }
    }
    let kind = m.doing.attacking();
    // The sac out of a swelling: its own poise, and full it tears.
    if part == SAC && kind == Some(INFLATE) {
        m.own[body::SAC] += dealt;
        if m.own[body::SAC] >= Knob::SacPoise.raw() {
            m.own[body::SAC] = 0;
            wind(m);
            return true;
        }
    }
    // The belch broken: the spark goes off in its own mouth.
    if matches!(
        m.doing,
        Doing::Startup { kind: BELCH, .. } | Doing::Active { kind: BELCH, .. }
    ) && m.strain >= m.interrupt_bar()
    {
        m.strain = 0;
        m.own[body::EVENTS] |= event::BACKFIRE;
        m.doing = Doing::Flinch {
            left: Knob::StaggerFrames.raw() as u16,
        };
        return true;
    }
    // The wallow broken: upright, staggered, the coat half made. Easier to
    // break than anything else it does, because breaking it is the point.
    if matches!(
        m.doing,
        Doing::Startup { kind: WALLOW, .. } | Doing::Active { kind: WALLOW, .. }
    ) {
        let bar = Fx::from_int(m.interrupt_bar())
            .mul(Knob::WallowInterrupt.fx())
            .to_int();
        if m.strain >= bar {
            m.strain = 0;
            m.own[body::EVENTS] |= event::WALLOW_BROKEN;
            let half = math::half(Fx::ONE);
            if coat(m).raw() < half.raw() {
                m.own[body::COAT_LOST] = half.raw();
            }
            m.doing = Doing::Flinch {
                left: Knob::StaggerFrames.raw() as u16,
            };
            return true;
        }
    }
    // Gutted, it flails without threat and takes it.
    gutted(m)
}

/// Winded: the sac torn or a hot mouthful, the body sagging.
fn wind(m: &mut Monster) {
    m.doing = Doing::Stumble {
        left: m.sp().stumble_frames(),
        front: true,
    };
    m.speed = Fx::ZERO;
    m.yaw_rate = Fx::ZERO;
    m.strain = 0;
    m.own[body::EVENTS] |= event::WINDED;
}

// ---------------------------------------------------------------------------
// What its moves do besides hurt
// ---------------------------------------------------------------------------

fn landed(w: &mut World, slot: usize, victim: usize, kind: u8, guarded: bool) {
    let Some(m) = w.monsters[slot] else { return };
    match kind {
        TONGUE if guarded => {
            // **A Bulwark blocking the tongue gives it the shield instead**:
            // it chokes on it, and spits it out planted where it lands.
            let p = &mut w.players[victim];
            if let crate::class::Mechanic::Shield(s) = p.mechanic {
                let ahead = mouth(&m).add(V3::from_turns(m.yaw).scale(Knob::SpitDistance.fx()));
                p.mechanic = crate::class::Mechanic::Shield(crate::class::Shield::Planted {
                    pos: V3::new(ahead.x, Fx::ZERO, ahead.z),
                    weight: s.weight(),
                });
            }
            if let Some(beast) = w.monsters[slot].as_mut() {
                beast.doing = Doing::Recovery {
                    kind: GAG,
                    left: beast.sp().attack(GAG).recovery,
                };
            }
        }
        TONGUE => {
            if swallowing(&w.lore).is_some() {
                return;
            }
            // Caught. Pulled in, and in.
            let ground = w.terrain();
            let hot = ground
                .floor
                .iter()
                .any(|h| h.kind == BURNING && h.holds(w.players[victim].pos));
            let mut word = (victim as u32 + 1) | PULLING << 8;
            if hot {
                word |= HOT;
            }
            w.lore.set_word(word::SWALLOW, word);
            w.lore
                .set_word(word::SWALLOW_LEFT, Knob::SwallowPull.raw() as u32);
        }
        BACKWASH => tar(&mut w.players[victim]),
        SPEW if guarded => {
            // **The shield takes the glob**, weight and all, and the pool lands
            // at its front edge rather than under her.
            let p = w.players[victim];
            let at = m.aimed_at();
            let front = p.pos.add(p.facing.scale(Knob::ShieldFront.fx()));
            let list: HazardList = hazard::all(&w.lore).collect();
            for &(i, h) in list.iter() {
                if h.index() == TAR
                    && h.age == 0
                    && math::wide_flat_dist(h.centre(), at).raw() <= h.radius().raw()
                {
                    let mut moved = h;
                    moved.at = lore::point_cm(V3::new(front.x, Fx::ZERO, front.z));
                    hazard::set(&mut w.lore, i, moved);
                }
            }
        }
        _ => {}
    }
}

/// Tarred: the tar's slow, wherever you stand, for a while.
fn tar(p: &mut state::Player) {
    p.slow(Knob::TarredFrames.raw() as u16, Knob::TarredSlow.fx());
}

/// Who it has swallowed or is pulling in, and which phase.
pub fn swallowing(lore: &Lore) -> Option<(usize, u32)> {
    let w = lore.word(word::SWALLOW);
    let who = w & 0xFF;
    (who != 0).then(|| (who as usize - 1, (w >> 8) & 0xFF))
}

// ---------------------------------------------------------------------------
// Laying tar
// ---------------------------------------------------------------------------

/// **Put a pool of tar down**: merged into a pool it lands on (areas add,
/// up to `PoolMost`), into the nearest pool if the list is full, or in a free
/// slot. A pool that lands over a slag mound coats it. Returns its slot.
pub fn lay_pool(lore: &mut Lore, at: V3, radius: Fx) -> Option<usize> {
    if radius.raw() <= 0 {
        return None;
    }
    let at = V3::new(at.x, Fx::ZERO, at.z);
    let most = Knob::PoolMost.fx();
    let tar_kind = TAR + 1;
    // Coat any slag it lands on.
    let list: HazardList = hazard::all(lore).collect();
    for &(i, h) in list.iter() {
        if h.index() == SLAG && math::wide_flat_dist(h.centre(), at).raw() <= radius.raw() {
            let coated = lore.word(word::COATED) | 1 << i;
            lore.set_word(word::COATED, coated);
        }
    }
    let onto = hazard::all(lore)
        .filter(|(_, h)| h.kind == tar_kind)
        .find(|(_, h)| math::wide_flat_dist(h.centre(), at).raw() < h.radius().raw());
    let free = (0..hazard::room(lore)).find(|i| !hazard::get(lore, *i).present());
    let merge_into = match (onto, free) {
        (Some((i, _)), _) => Some(i),
        (None, Some(_)) => None,
        (None, None) => hazard::all(lore)
            .filter(|(_, h)| h.kind == tar_kind)
            .min_by_key(|(_, h)| math::wide_flat_dist(h.centre(), at).raw())
            .map(|(i, _)| i),
    };
    let count = lore.word(word::POOLS).wrapping_add(1);
    lore.set_word(word::POOLS, count);
    match merge_into {
        Some(i) => {
            let mut h = hazard::get(lore, i);
            let r = h.radius();
            let grown = math::sqrt(r.mul(r).add(radius.mul(radius))).min(most);
            h.radius = hazard::radius_byte(grown);
            hazard::set(lore, i, h);
            Some(i)
        }
        None => {
            let slot = free?;
            hazard::set(lore, slot, Hazard::disc(TAR, at, radius.min(most)));
            Some(slot)
        }
    }
}

/// The belly flop takes your stairs as well as your floor: every slag mound
/// in its crash circle is gone.
fn shatter(lore: &mut Lore, at: V3, radius: Fx) {
    for (i, h) in hazard::all(lore).collect::<HazardList>().iter() {
        if h.index() == SLAG && math::wide_flat_dist(h.centre(), at).raw() <= radius.raw() {
            hazard::clear(lore, *i);
        }
    }
}

/// A fixed list of hazards with their slots, for walking while writing.
struct HazardList {
    items: [(usize, Hazard); hazard::MAX_HAZARDS],
    n: usize,
}

impl FromIterator<(usize, Hazard)> for HazardList {
    fn from_iter<I: IntoIterator<Item = (usize, Hazard)>>(iter: I) -> Self {
        let mut l = HazardList {
            items: [(0, Hazard::default()); hazard::MAX_HAZARDS],
            n: 0,
        };
        for x in iter {
            if l.n < l.items.len() {
                l.items[l.n] = x;
                l.n += 1;
            }
        }
        l
    }
}

impl HazardList {
    fn iter(&self) -> impl Iterator<Item = &(usize, Hazard)> {
        self.items[..self.n].iter()
    }
}

/// Light every tar pool in a set of slots; returns how many caught.
fn light(lore: &mut Lore, slots: u32) -> u32 {
    let mut n = 0;
    for i in 0..hazard::MAX_HAZARDS {
        if slots & 1 << i != 0 && hazard::ignite(lore, i) {
            n += 1;
        }
    }
    n
}

/// Add to one byte of a counter word.
fn count_byte(lore: &mut Lore, word: usize, byte: u32, by: u32) {
    let w = lore.word(word);
    let shift = byte * 8;
    let v = ((w >> shift) & 0xFF).saturating_add(by).min(0xFF);
    lore.set_word(word, (w & !(0xFF << shift)) | v << shift);
}

/// Add to one half of a counter word.
fn count_half(lore: &mut Lore, word: usize, high: bool, by: u32) {
    let w = lore.word(word);
    let shift = if high { 16 } else { 0 };
    let v = ((w >> shift) & 0xFFFF).saturating_add(by).min(0xFFFF);
    lore.set_word(word, (w & !(0xFFFF << shift)) | v << shift);
}

// ---------------------------------------------------------------------------
// Braziers
// ---------------------------------------------------------------------------

/// The arena's braziers: its sites called "brazier", in order, at most four.
/// Each stands on the plinth its site's box describes.
pub fn braziers(w: &World) -> impl Iterator<Item = (usize, V3)> + '_ {
    w.arena()
        .sites
        .iter()
        .filter(|s| s.name == "brazier")
        .take(word::BRAZIERS)
        .enumerate()
        .map(|(i, s)| {
            let (x, z) = s.route.first().copied().unwrap_or((0, 0));
            let y = lore::from_cm(s.size.1 as i16);
            (i, V3::new(Fx::ratio(x, 100), y, Fx::ratio(z, 100)))
        })
}

/// Is brazier `i` standing and lit?
pub fn brazier_lit(lore: &Lore, i: usize) -> bool {
    lore.word(word::BRAZIER + i) == 0
}

/// Tip a brazier: its coals spill in a lane away from whoever hit it, and it
/// relights after `brazier_relight`.
fn tip(w: &mut World, i: usize, at: V3, away: V3) {
    let along = V3::new(away.x, Fx::ZERO, away.z);
    let along = if along.flat_len().raw() > 0 {
        math::wide_normalized(along)
    } else {
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO)
    };
    let a = V3::new(at.x, Fx::ZERO, at.z);
    let b = a.add(along.scale(Knob::CoalLength.fx()));
    hazard::place(
        &mut w.lore,
        Hazard::strand(COALS, a, b, Knob::CoalWidth.fx()),
    );
    w.lore
        .set_word(word::BRAZIER + i, Knob::BrazierRelight.raw().max(1) as u32);
}

/// Any blow tips a standing brazier: a fighter's swing reaching it, or the
/// creature's own volume.
fn braziers_struck(w: &mut World) {
    let mut found = [(V3::ZERO, false); word::BRAZIERS];
    for (i, at) in braziers(w) {
        found[i] = (at, true);
    }
    for (i, (at, there)) in found.into_iter().enumerate() {
        if !there {
            continue;
        }
        let left = w.lore.word(word::BRAZIER + i);
        if left != 0 {
            w.lore.set_word(word::BRAZIER + i, left - 1);
            continue;
        }
        let reach = math::half(Knob::BrazierSize.fx());
        let top = at.add(V3::new(Fx::ZERO, Knob::BrazierSize.fx(), Fx::ZERO));
        let mut away = None;
        for p in w.players.iter() {
            if p.health <= 0 {
                continue;
            }
            let Some(swing) = state::hitbox(p) else {
                continue;
            };
            if swing.spent {
                continue;
            }
            let gap = math::segment_gap(swing.from, swing.to, at, top);
            if gap.raw() <= swing.radius.add(reach).raw() {
                away = Some(at.sub(p.pos));
                break;
            }
        }
        if away.is_none() {
            for m in w.monsters.iter().flatten() {
                if m.reaches(at, reach, reach) {
                    away = Some(at.sub(m.pos));
                }
            }
        }
        if let Some(dir) = away {
            tip(w, i, at, dir);
        }
    }
}

// ---------------------------------------------------------------------------
// Once a frame
// ---------------------------------------------------------------------------

fn frame(w: &mut World) {
    if w.lore.word(word::FLAGS) & 1 == 0 {
        w.lore.set_word(word::FLAGS, 1);
        if let Some(m) = w.monsters[0] {
            w.lore.set_word(word::HEALTH, m.health as u32);
        }
    }
    braziers_struck(w);
    for slot in 0..w.monsters.len() {
        if w.monsters[slot].is_some_and(|m| m.species == SpeciesId::MIREBACK) {
            creature(w, slot);
        }
    }
    tar_feet(w);
    fires_started(w);
    coated_slag(w);
}

/// The tar's slow has a tail: it lingers after the feet leave it, so the
/// slow does not flicker at a pool's edge.
fn tar_feet(w: &mut World) {
    let ground = w.terrain();
    let tail = Knob::TarTail.raw() as u16;
    let Some(sp) = ground.floor.species() else {
        return;
    };
    let walk = hazard::stat_fx(sp, TAR, HazardField::Walk);
    for p in w.players.iter_mut() {
        if p.health <= 0 || !p.footed() || p.aboard() {
            continue;
        }
        if ground.floor.iter().any(|h| h.kind == TAR && h.holds(p.pos)) {
            p.slow(tail, walk);
        }
    }
}

/// Which pools caught this frame, and from what: a belch or a backfire is
/// counted where it happens; a pool that caught touching coals was a
/// brazier; one that caught touching nothing already burning was a
/// fighter's own fire.
fn fires_started(w: &mut World) {
    let was = w.lore.word(word::BURNING);
    let ground = w.terrain();
    let mut now = 0u32;
    for h in ground.floor.iter() {
        if h.kind == BURNING || h.kind == COALS {
            now |= 1 << h.slot;
        }
    }
    for h in ground.floor.iter() {
        if h.kind != BURNING || was & 1 << h.slot != 0 || h.age > 1 {
            continue;
        }
        let touching = |kind: u8| {
            ground.floor.iter().any(|o| {
                o.slot != h.slot
                    && o.kind == kind
                    && h.flat_gap(o.nearest_to(h.a)).raw() <= h.radius.add(o.radius).raw()
            })
        };
        if touching(COALS) {
            count_byte(&mut w.lore, word::LIT, 0, 1);
        } else if !touching(BURNING) && w.lore.word(word::FLAGS) & 2 == 0 {
            count_byte(&mut w.lore, word::LIT, 3, 1);
        }
    }
    // The species' own fires (bit 1 of FLAGS) are counted where they are set.
    let flags = w.lore.word(word::FLAGS) & !2;
    w.lore.set_word(word::FLAGS, flags);
    w.lore.set_word(word::BURNING, now);
}

/// A slag mound with tar on it grows a layer when fire reaches it; new slag
/// is counted.
fn coated_slag(w: &mut World) {
    let ground = w.terrain();
    let mut coated = w.lore.word(word::COATED);
    for h in ground.floor.iter() {
        let bit = 1 << h.slot;
        if h.kind != SLAG {
            coated &= !bit;
            continue;
        }
        if h.age == 0 {
            count_half(&mut w.lore, word::SLAG_BUILT, false, 1);
        }
        if coated & bit == 0 {
            continue;
        }
        let fire = ground.floor.iter().any(|o| {
            o.decl.burns && math::wide_flat_dist(o.a, h.a).raw() <= o.radius.add(h.radius).raw()
        });
        if fire {
            coated &= !bit;
            let most = hazard::stat(&SPECIES, SLAG, HazardField::Layers).max(1) as u8;
            let mut s = hazard::get(&w.lore, h.slot as usize);
            if s.state < most {
                s.state += 1;
                hazard::set(&mut w.lore, h.slot as usize, s);
                count_half(&mut w.lore, word::SLAG_BUILT, true, 1);
            }
        }
    }
    // Slots that are empty now are not coated.
    for i in 0..hazard::MAX_HAZARDS {
        if !hazard::get(&w.lore, i).present() {
            coated &= !(1 << i);
        }
    }
    w.lore.set_word(word::COATED, coated);
}

/// One frame of the creature's own rules.
fn creature(w: &mut World, slot: usize) {
    let Some(mut m) = w.monsters[slot] else {
        return;
    };
    let herd = w.monsters;

    // What hit it since last frame, from close: the crowd term's memory.
    let was = w.lore.int(word::HEALTH);
    let took = (was - m.health).max(0);
    let close = foot_radius(&m).add(Knob::CrowdReach.fx());
    let near = w.players.iter().any(|p| {
        p.health > 0
            && !state::inside(p, &herd)
            && math::wide_flat_dist(p.pos, m.pos).raw() <= close.raw()
    });
    let mut crowd = w.lore.int(word::CROWD);
    if near {
        crowd += took;
    }
    crowd -= crowd / Knob::CrowdFade.raw().max(1) + (crowd > 0) as i32;
    w.lore.set_int(word::CROWD, crowd.max(0));

    events(w, slot, &mut m);
    act(w, slot, &mut m);
    burn_itself(w, &mut m);
    swallow(w, slot, &mut m);

    // Coat uptime, for the report.
    if coat(&m).raw() * 2 > Fx::ONE.raw() {
        let n = w.lore.word(word::COAT_FRAMES).wrapping_add(1);
        w.lore.set_word(word::COAT_FRAMES, n);
    }
    w.lore.set_int(word::HEALTH, m.health);
    w.monsters[slot] = Some(m);
}

/// What happened to the body this frame, acted on with the world.
fn events(w: &mut World, _slot: usize, m: &mut Monster) {
    let ev = m.own[body::EVENTS];
    if ev & event::BACKFIRE != 0 {
        let herd = w.monsters;
        let marked = kindling(m, &w.lore, &herd, Knob::BackfireReach.fx());
        let lit = light(&mut w.lore, marked);
        if lit > 0 {
            count_byte(&mut w.lore, word::LIT, 2, lit);
        }
        count_half(&mut w.lore, word::WINDED, true, 1);
        w.lore.set_word(word::FLAGS, w.lore.word(word::FLAGS) | 2);
    }
    if ev & event::WALLOW_BROKEN != 0 {
        count_half(&mut w.lore, word::GUTTED, true, 1);
    }
    // Any stumble that is not a winding is a knock-up it shrugged off: it
    // is already on the floor, so a launch landing on it is a flinch.
    match m.doing {
        Doing::Stumble { .. } if ev & event::WINDED == 0 => {
            m.doing = Doing::Flinch {
                left: m.sp().flinch_frames(),
            };
        }
        Doing::Stumble { .. } => {}
        _ => m.own[body::EVENTS] &= !event::WINDED,
    }
    m.own[body::EVENTS] &= event::WINDED | event::RETCH;
    // The sac's poise is for one swelling.
    if m.doing.attacking() != Some(INFLATE) {
        m.own[body::SAC] = 0;
    }
}

/// What its moves do to the floor, frame by frame.
fn act(w: &mut World, _slot: usize, m: &mut Monster) {
    let share = tar_share(w, m);
    match m.doing {
        // ---- the spew: a pool where the glob lands; two in the tide ----
        Doing::Startup { kind: SPEW, left } if left == m.sp().attack(SPEW).startup => {
            w.lore.set_word(word::SECOND, 0);
        }
        Doing::Active { kind: SPEW, left } if left == m.sp().attack(SPEW).active => {
            lay_pool(
                &mut w.lore,
                m.aimed_at(),
                by_area(Knob::PoolSpew.fx(), share),
            );
        }
        Doing::Recovery { kind: SPEW, left }
            if left == m.sp().attack(SPEW).recovery
                && tide(m)
                && w.lore.word(word::SECOND) == 0 =>
        {
            w.lore.set_word(word::SECOND, 1);
            m.doing = Doing::Startup {
                kind: SPEW,
                left: Knob::TideSecond.raw() as u16,
            };
            m.hit_used = false;
            m.lob(SPEW);
        }

        // ---- the flop: the leap, the crash, the ring ----
        Doing::Startup { kind: FLOP, left } => {
            let a = m.sp().attack(FLOP);
            if left == a.startup {
                w.lore.set_word(
                    word::LEAP,
                    lore::halves(lore::to_cm(m.pos.x), lore::to_cm(m.pos.z)),
                );
            }
            let air = Knob::FlopAir.raw().max(1) as u16;
            if left <= air {
                let from = leap_from(&w.lore);
                let t = Fx::from_int((air - left) as i32).div(Fx::from_int(air as i32));
                let to = m.aimed_at();
                let at = math::lerp3(from, to, t);
                let rise = Knob::FlopHeight.fx().mul(math::hump(t));
                m.pos = V3::new(at.x, rise, at.z);
                m.speed = Fx::ZERO;
            }
        }
        Doing::Active { kind: FLOP, left } if left == m.sp().attack(FLOP).active => {
            let at = m.aimed_at();
            m.pos = V3::new(at.x, Fx::ZERO, at.z);
            let a = m.sp().attack(FLOP);
            shatter(&mut w.lore, at, a.hit_radius);
            let (points, n, r) = ring(m, at, share.max(Fx::ZERO).max(coop_only(w)));
            for p in points.iter().take(n) {
                lay_pool(&mut w.lore, *p, r);
            }
        }

        // ---- the tongue: its line stops at the first solid ----
        Doing::Startup { kind: TONGUE, .. } | Doing::Active { kind: TONGUE, .. }
            if swallowing(&w.lore).is_none() =>
        {
            let a = m.sp().attack(TONGUE);
            let from = mouth(m);
            let from = V3::new(from.x, math::half(a.hit_low.add(a.hit_high)), from.z);
            let along = V3::from_turns(m.yaw);
            let most = a
                .hit_x
                .add(a.travel.mul(Fx::from_int(a.active as i32)).mul(DT));
            let end = m.pos.add(along.scale(most));
            let end = V3::new(end.x, from.y, end.z);
            let field = crate::stones::gather(&w.players);
            let ground = w.terrain();
            let fighters = w.players;
            let effects = w.effects;
            let critters = w.critters;
            let scene = aim::Scene {
                stones: &field,
                players: &fighters,
                effects: &effects,
                quarry: &[None; crate::monster::MAX_MONSTERS],
                critters: &critters,
                arena: &ground,
            };
            let hit = aim::first_along(
                aim::Path { from, to: end },
                a.hit_radius,
                state::NOBODY,
                &scene,
                aim::Targets::none().stones().terrain(),
            );
            let stop = match hit {
                Some(c) => from.add(along.scale(c.dist())),
                None => end.add(along.scale(a.hit_radius)),
            };
            m.aim_at(stop);
        }

        // ---- the backwash: a pool behind it ----
        Doing::Active {
            kind: BACKWASH,
            left,
        } if left == m.sp().attack(BACKWASH).active => {
            let a = m.sp().attack(BACKWASH);
            let behind = m.pos.add(V3::from_turns(m.yaw).scale(a.hit_x));
            lay_pool(&mut w.lore, behind, by_area(Knob::PoolBackwash.fx(), share));
        }

        // ---- the belch: every pool it marked ----
        Doing::Active { kind: BELCH, left } if left == m.sp().attack(BELCH).active => {
            let herd = w.monsters;
            let marked = kindling(m, &w.lore, &herd, Knob::BelchReach.fx());
            let lit = light(&mut w.lore, marked);
            if lit > 0 {
                count_byte(&mut w.lore, word::LIT, 1, lit);
                w.lore.set_word(word::FLAGS, w.lore.word(word::FLAGS) | 2);
            }
        }

        // ---- the wallow: the coat comes back; set alight, it is gutted ----
        Doing::Startup { kind: WALLOW, .. } | Doing::Active { kind: WALLOW, .. } => {
            let herd = w.monsters;
            if pools_under(m, &w.lore, &herd, BURNING) > 0 {
                m.doing = Doing::Toppled {
                    left: m.sp().topple_frames(),
                };
                m.speed = Fx::ZERO;
                m.yaw_rate = Fx::ZERO;
                m.strain = 0;
                m.own[body::COAT_LOST] = Fx::ONE.raw();
                count_half(&mut w.lore, word::GUTTED, false, 1);
            } else if let Doing::Active { .. } = m.doing {
                let frames = Fx::from_int(m.sp().attack(WALLOW).active.max(1) as i32);
                let slower = Fx::ONE
                    .sub(Knob::WartCoatCost.fx().mul(Fx::from_int(burst(m))))
                    .max(Fx::ZERO);
                lose_coat(m, slower.div(frames).neg());
            }
        }
        _ => {}
    }
}

/// Two hunters' worth of tar, ignoring the warts: a flop's ring loses pools
/// to burst warts rather than shrinking them.
fn coop_only(w: &World) -> Fx {
    if w.players.iter().filter(|p| p.health > 0).count() > 1 {
        Knob::TarCoop.fx()
    } else {
        Fx::ONE
    }
}

fn leap_from(lore: &Lore) -> V3 {
    let w = lore.word(word::LEAP);
    V3::new(
        lore::from_cm(lore::lo(w)),
        Fx::ZERO,
        lore::from_cm(lore::hi(w)),
    )
}

/// **The Mireback in its own fire**: so much a tick per burning pool under
/// its footprint, up to a few pools, double while gutted; and its coat
/// cracks while it burns.
fn burn_itself(w: &mut World, m: &mut Monster) {
    let herd = w.monsters;
    let pools = (pools_under(m, &w.lore, &herd, BURNING) as i32).min(Knob::BurnSelfPools.raw());
    if pools <= 0 || !m.alive() {
        return;
    }
    lose_coat(m, Knob::CoatCrack.fx().mul(DT));
    let every = Knob::BurnSelfEvery.raw().max(1) as u32;
    if w.frame % every != 0 {
        return;
    }
    let mut tick = Fx::from_int(Knob::BurnSelf.raw() * pools);
    if gutted(m) {
        tick = tick.mul(Knob::GuttedBurn.fx());
    }
    let dealt = m.scorch(tick.to_int());
    let total = w.lore.word(word::SELF_BURN).wrapping_add(dealt as u32);
    w.lore.set_word(word::SELF_BURN, total);
}

/// **Swallowed**: pulled in, held inside on the ride's rules, burnt by acid,
/// and spat out by time or by damage, whichever is first.
fn swallow(w: &mut World, slot: usize, m: &mut Monster) {
    let Some((who, phase)) = swallowing(&w.lore) else {
        m.own[body::STOMACH] = 0;
        m.own[body::EVENTS] &= !event::RETCH;
        return;
    };
    let hot = w.lore.word(word::SWALLOW) & HOT != 0;
    let left = w.lore.word(word::SWALLOW_LEFT);
    let p = &mut w.players[who.min(MAX_PLAYERS - 1)];
    if p.health <= 0 {
        release(w, slot, m, false);
        return;
    }
    if phase == PULLING {
        // The tongue broken mid-pull lets go.
        if m.doing.attacking() != Some(TONGUE) {
            w.lore.set_word(word::SWALLOW, 0);
            return;
        }
        let to = mouth(m);
        let gap = to.sub(p.pos);
        p.pos = p
            .pos
            .add(gap.scale(Fx::ONE.div(Fx::from_int(left.max(1) as i32))));
        p.vel = V3::ZERO;
        p.action = Action::HitStun { left: 2 };
        if left > 1 {
            w.lore.set_word(word::SWALLOW_LEFT, left - 1);
            return;
        }
        // In.
        state::put_inside(p, slot, m, STOMACH);
        p.action = Action::Free;
        let frames = if hot {
            Knob::HotRetch.raw()
        } else {
            Knob::SwallowFrames.raw()
        } as u16;
        m.doing = Doing::Active {
            kind: SWALLOW,
            left: frames,
        };
        m.hit_used = true;
        m.own[body::STOMACH] = 0;
        m.brain.cooldown[TONGUE as usize] = Knob::TongueFull.raw() as u16;
        w.lore.set_word(
            word::SWALLOW,
            (who as u32 + 1) | INSIDE << 8 | if hot { HOT } else { 0 },
        );
        count_half(&mut w.lore, word::SWALLOWS, false, 1);
        if hot {
            count_half(&mut w.lore, word::SWALLOWS, true, 1);
        }
        return;
    }
    // Inside. The acid.
    let every = Knob::SwallowAcidEvery.raw().max(1) as u32;
    if w.frame % every == 0 {
        let acid = Knob::SwallowAcid.raw();
        p.wound(acid);
        let total = w.lore.word(word::ACID).wrapping_add(acid as u32);
        w.lore.set_word(word::ACID, total);
    }
    let retch = m.own[body::EVENTS] & event::RETCH != 0;
    let still = matches!(m.doing, Doing::Active { kind: SWALLOW, .. });
    if retch || !still {
        let dealt = m.own[body::STOMACH].max(0) as u32;
        let total = w.lore.word(word::STOMACH_DEALT).wrapping_add(dealt);
        w.lore.set_word(word::STOMACH_DEALT, total);
        release(w, slot, m, true);
    }
}

/// Let the swallowed out: spat along its facing into a fresh pool, knocked
/// down -- or, a hot mouthful, retched out and the toad winded by it.
fn release(w: &mut World, _slot: usize, m: &mut Monster, spit: bool) {
    let Some((who, _)) = swallowing(&w.lore) else {
        return;
    };
    let hot = w.lore.word(word::SWALLOW) & HOT != 0;
    w.lore.set_word(word::SWALLOW, 0);
    w.lore.set_word(word::SWALLOW_LEFT, 0);
    m.own[body::STOMACH] = 0;
    m.own[body::EVENTS] &= !event::RETCH;
    let along = V3::from_turns(m.yaw);
    let from = mouth(m);
    let p = &mut w.players[who.min(MAX_PLAYERS - 1)];
    state::let_out(p, from, V3::ZERO);
    if !spit {
        return;
    }
    let far = Knob::SpitDistance.fx();
    let land = V3::new(from.x, Fx::ZERO, from.z).add(along.scale(far));
    state::apply_hit(
        p,
        Hit {
            damage: Knob::SpitDamage.raw(),
            hitstun: m.sp().attack(SWALLOW).hitstun,
            blockstun: 0,
            knockback: Knob::SpitSpeed.fx(),
            launch: m.sp().attack(SWALLOW).launch,
            grabs: 0,
            by: QUARRY,
            dir: along,
            blocked: false,
            parried: false,
            interrupts: true,
        },
    );
    tar(p);
    let share = tar_share(w, m);
    lay_pool(&mut w.lore, land, by_area(Knob::PoolSpew.fx(), share));
    if matches!(m.doing, Doing::Active { kind: SWALLOW, .. }) {
        m.doing = Doing::Recovery {
            kind: SWALLOW,
            left: m.sp().attack(SWALLOW).recovery,
        };
    }
    if hot {
        m.scorch(Knob::HotRetchDamage.raw());
        if m.alive() {
            wind(m);
        }
    }
}

// ---------------------------------------------------------------------------
// What it draws
// ---------------------------------------------------------------------------

/// **What it draws besides its hazards and its telegraph**: the ring a flop
/// will leave, every pool a belch will light, and the braziers.
fn marks(w: &World, out: &mut Marks) {
    let herd = w.monsters;
    for m in herd.iter().flatten() {
        if m.species != SpeciesId::MIREBACK {
            continue;
        }
        match m.doing {
            Doing::Startup { kind: FLOP, left } => {
                let a = m.sp().attack(FLOP);
                let progress = Fx::from_int((a.startup - left.min(a.startup)) as i32)
                    .div(Fx::from_int(a.startup.max(1) as i32));
                let (points, n, r) = ring(m, m.aimed_at(), tar_share(w, m).max(coop_only(w)));
                for p in points.iter().take(n) {
                    out.push(Mark {
                        at: *p,
                        radius: r,
                        height: Fx::ZERO,
                        look: MarkLook::Warning,
                        progress,
                    });
                }
            }
            Doing::Startup { kind: BELCH, left } => {
                let a = m.sp().attack(BELCH);
                let progress = Fx::from_int((a.startup - left.min(a.startup)) as i32)
                    .div(Fx::from_int(a.startup.max(1) as i32));
                let bits = kindling(m, &w.lore, &herd, Knob::BelchReach.fx());
                for h in hazard::Floor::build(&w.lore, &herd).iter() {
                    if bits & 1 << h.slot != 0 {
                        out.push(Mark {
                            at: h.a,
                            radius: h.radius,
                            height: Fx::ZERO,
                            look: MarkLook::Kindling,
                            progress,
                        });
                    }
                }
            }
            _ => {}
        }
    }
    let tall = Knob::BrazierSize.fx();
    for (i, at) in braziers(w) {
        out.push(Mark {
            at,
            radius: math::half(tall),
            height: tall,
            look: MarkLook::Iron,
            progress: Fx::ONE,
        });
        let left = w.lore.word(word::BRAZIER + i);
        let top = at.add(V3::new(Fx::ZERO, tall, Fx::ZERO));
        if left == 0 {
            out.push(Mark {
                at: top,
                radius: math::half(math::half(tall)),
                height: tall,
                look: MarkLook::Flame,
                progress: Fx::ONE,
            });
        } else {
            let full = Knob::BrazierRelight.raw().max(1);
            out.push(Mark {
                at: top,
                radius: math::half(math::half(tall)),
                height: Fx::ZERO,
                look: MarkLook::Embers,
                progress: Fx::ONE
                    .sub(Fx::from_int(left as i32).div(Fx::from_int(full)))
                    .clamp(Fx::ZERO, Fx::ONE),
            });
        }
    }
}
