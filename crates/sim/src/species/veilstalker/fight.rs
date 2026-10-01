//! What the Veilstalker does to the world and what the world does to it: the
//! veil, the trail, the paint and the mottle, the smoke, the braziers and the
//! fire panic, the quills in flight, the perch and the leaps, the mimic's
//! ghost and the retreat -- everything the shared machinery calls it by. See
//! `docs/design/creatures/veilstalker.md`.
//!
//! **The simulation owns visibility** (§6). How much of each part can be seen
//! is one function, [`shown`], behind `World::shown`: the renderer draws a part
//! at that strength, and the fight report and the scripted hunter call
//! "visible" exactly what it calls visible. It is worked out, not stored: from
//! the move in progress (a decloak ramps up from nothing over its first
//! `DecloakRamp` frames), the re-cloak's fade, its speed (a shimmer), the
//! paint on a part, a mottled region, and an outline where a part stands in
//! water or fire.
//!
//! **Cloak is never a hit-test fact.** Nothing here touches the hurtboxes: a
//! cloaked body is struck exactly as a visible one is.
//!
//! **Its frame hook** ([`frame`]) runs once a frame after the creature has
//! stepped: it glances the hunters' looks (the one thing it sees that the
//! shared glance does not), counts the hits of an engagement and paints where
//! they landed, starts the retreat, flies the climb and the pounce, throws
//! the quills, grows the smoke, places the mimic's ghost, stamps the trail,
//! tips the braziers, and panics in fire. What the brain needs from all that
//! -- is the place it would decloak inside the target's glanced view, is it
//! in fire, in smoke -- it writes into the lore for [`super::mind`] to read,
//! so the brain's window is still `Quarry` and `Mind`.

use crate::DT;
use crate::aim::{self, Scene};
use crate::arena::Material;
use crate::effects::EffectKind;
use crate::fixed::Fx;
use crate::hazard::{self, Hazard, HazardDecl, reach};
use crate::lore::{self, Layout, Lore};
use crate::math::{self, V3};
use crate::monster::{Doing, MAX_MONSTERS, Monster};
use crate::species::{FightDecl, Mark, MarkLook, Marks, SpeciesId};
use crate::state::{self, Hit, MAX_PLAYERS, QUARRY, World};

use super::{
    CLIMB, GETUP, Knob, LUNGE, MIMIC, PART_COUNT, POUNCE, QUILLS, RAKE, RAKE2, REGION, REGIONS,
    RETREAT, SMOKE, SPEAR, SPECIES, STRIKES,
};

pub static FIGHT: FightDecl = FightDecl {
    layout: Layout {
        hazards: HAZARD_CELLS,
        noises: 0,
        objectives: 0,
        own: 19,
    },
    row: true,
    hazards: &HAZARDS,
    collides: true,
    keeps_height: true,
    frame: Some(frame),
    shown: Some(shown),
    apparition: Some(apparition),
    appetite: Some(super::mind::appetite),
    prowl_to: Some(super::mind::prowl_to),
    commit: Some(super::mind::commit),
    struck: Some(struck),
    marks: Some(marks),
    signs: Some(signs),
    pace: Some(pace),
    presence: Some(presence),
    lob_height: Some(lob_height),
    ..FightDecl::PLAIN
};

// ---------------------------------------------------------------------------
// The floor's kinds
// ---------------------------------------------------------------------------

/// How many hazards it keeps at once: four braziers' coals, and its smoke --
/// which becomes the flash that burns it off in its own cell.
pub const HAZARD_CELLS: u8 = 5;

/// A tipped brazier's coals: a burning patch two metres across, for eight
/// seconds, before the snow puts it out.
pub const COALS: u8 = 0;
/// Its smoke: a cloud that blocks sight, burnt off by fire.
pub const CLOUD: u8 = 1;
/// The cloud burning off: fire, for twenty frames.
pub const FLASH: u8 = 2;

pub const HAZARDS: [HazardDecl; 3] = [
    HazardDecl::disc("coals")
        .reaching(reach::FIGHTERS | reach::CRITTERS)
        .burning()
        .made_of(Material::Ash),
    HazardDecl::disc("smoke cloud")
        .reaching(0)
        .blocks_sight()
        .ignites_into(FLASH)
        .made_of(Material::Ash),
    HazardDecl::disc("flash").reaching(0).burning(),
];

// ---------------------------------------------------------------------------
// Its state
// ---------------------------------------------------------------------------

/// Its words in the hunt's lore (`Lore::word`), after its hazards.
pub mod word {
    /// **The footfall ring**: [`super::PRINTS`] words, one print each
    /// ([`super::Print`]).
    pub const PRINTS: usize = 0;
    /// **The paint ring**: four marks ([`super::Paint`]).
    pub const PAINT: usize = 32;
    /// Bit 0: set up. Bit 1: two hunters.
    pub const FLAGS: usize = 36;
    /// Low half: its stride at the end of last frame. High half: the next
    /// print's slot.
    pub const STRIDE: usize = 37;
    /// Frames of the re-cloak's fade left.
    pub const FADE: usize = 38;
    /// Low half: frames in fire, running. High half: frames of panic lockout
    /// left.
    pub const FIRE: usize = 39;
    /// Frames of stalking left before it will strike.
    pub const STALK: usize = 40;
    /// The mimic's ghost: where it stands (centimetres, `lore::halves`).
    pub const GHOST: usize = 41;
    /// Low half: the ghost's yaw, in 1/65536 of a turn. Byte 2: which rear it
    /// plays, plus one (zero: no ghost).
    pub const GHOST_LOOK: usize = 42;
    /// Frames left in which no strike follows a mimic.
    pub const QUIET: usize = 43;
    /// **The glance of each hunter's look**: where they stood (centimetres)
    /// and which way they faced, now and a glance before (`lore::halves` of
    /// the two yaws in 1/65536 of a turn). Two hunters, two words each.
    pub const GLANCED: usize = 44;
    /// Frames to the next look glance.
    pub const GLANCE_LEFT: usize = 48;
    /// The quills in flight: where they left (cm, x and z), then their
    /// height and the fan's middle bearing, then the frame they left, then
    /// which of the five are still flying and the fall per metre.
    pub const QUILL_AT: usize = 49;
    pub const QUILL_UP: usize = 50;
    pub const QUILL_FRAME: usize = 51;
    pub const QUILL_LIVE: usize = 52;
    pub const QUILL_SLOPE: usize = 53;
    /// A bit per brazier: tipped.
    pub const BRAZIERS: usize = 54;
    /// The frame its cloud began to grow, plus one, and its slot (`SMOKE_SLOT`).
    pub const SMOKE_AT: usize = 55;
    pub const SMOKE_SLOT: usize = 56;
    /// The frame red prints stop.
    pub const RED_UNTIL: usize = 57;
    /// The last frame it saw a sample at a dodge's speed, plus one.
    pub const BAIT: usize = 58;
    /// How far out it means to strike from, this engagement (cm).
    pub const PLAN: usize = 59;
    /// Panics by cause, a byte each: pillar, cloud of embers, stone, brazier.
    pub const PANICS: usize = 60;
    /// Panics by any other fire, a mimic's count, smokes burnt off.
    pub const MORE: usize = 61;
    /// **What the brain may read of what the frame hook saw**, per hunter
    /// ([`super::view`] bits and the off-look in the top half).
    pub const VIEW: usize = 62;
    /// Where it would perch (cm) and how high (cm), or zero.
    pub const PERCH_AT: usize = 64;
    pub const PERCH_TOP: usize = 65;
    /// Where a leap left from (cm), and its height (cm) with the frames it
    /// has been perched in the high half.
    pub const LEAP_FROM: usize = 66;
    pub const LEAP_UP: usize = 67;
    /// The spear's locked bearing (1/65536 turn), plus one in the high half
    /// once it is locked.
    pub const SPEAR_YAW: usize = 68;
    /// The frame of the last counted hit, plus one.
    pub const LAST_HIT: usize = 69;
    /// The ghost's candidate place, worked out each glance (cm).
    pub const GHOST_AT: usize = 70;
    /// Its decloaks by where they were on the target's screen (glanced):
    /// centre, middle, edge thirds, a byte each.
    pub const THIRDS: usize = 71;
    /// The nearest fire to it within `FireShy` and a stride, (cm), and
    /// whether there is one (bit 0 of `FIRE_NEAR`'s neighbour).
    pub const FIRE_AT: usize = 72;
    pub const FIRE_NEAR: usize = 73;
    /// The frame the hook last ran on: the brain, which steps first, thinks
    /// on the one after.
    pub const NOW: usize = 74;
    /// Strikes begun since it last cloaked.
    pub const STRIKES: usize = 75;
}

/// The bits of a hunter's [`word::VIEW`].
pub mod view {
    /// Where it would decloak now is inside that hunter's glanced view.
    pub const IN_VIEW: u32 = 1;
    /// It is standing in fire (or near enough that a decloak there would be).
    pub const IN_FIRE: u32 = 2;
    /// It is inside its own cloud.
    pub const IN_SMOKE: u32 = 4;
    /// The hunter is inside its cloud, and the cloud is younger than
    /// `SmokeQuiet`.
    pub const THEY_IN_YOUNG_SMOKE: u32 = 8;
    /// The hunter is inside its cloud.
    pub const THEY_IN_SMOKE: u32 = 16;
    /// The mimic has somewhere to play, in that hunter's view.
    pub const GHOST_OK: u32 = 32;
    /// The other hunter is looking at where it would decloak.
    pub const WATCHED: u32 = 64;
    /// Where it would decloak is out of the hunter's view behind something
    /// solid, not off the side of it.
    pub const HIDDEN: u32 = 128;
    /// It stands on floor that takes no print -- the stream, a top -- so a
    /// decloak there would set no feet, and could not be told from a mimic.
    pub const BARE: u32 = 256;
}

/// Its four words on the body (`Monster::own`), which the hit path reads.
pub mod body {
    /// [`super::flag`] bits, and the hits of this engagement from
    /// [`super::flag::HITS_SHIFT`].
    pub const FLAGS: usize = 0;
    /// Damage into the head and the left flank (`lore::halves`, unsigned).
    pub const REGIONS_A: usize = 1;
    /// Damage into the right flank and the tail.
    pub const REGIONS_B: usize = 2;
    /// The parts struck this frame, a bit each: painted by the frame hook.
    pub const STRUCK: usize = 3;
}

/// The bits of [`body::FLAGS`]: tags, not quantities.
pub mod flag {
    pub type Bits = i32;
    /// Up on a trunk or the ledge: rooted, and its only strike is the pounce.
    pub const PERCHED: Bits = 1;
    /// In a fire panic: the topple it is in is the panic.
    pub const PANICKED: Bits = 2;
    /// Two hunters.
    pub const COOP: Bits = 4;
    /// Stalking: slinking, never faster than its slink.
    pub const STALKING: Bits = 8;
    /// Exposed: a lit mark on it, or a region mottled. Slinking is no
    /// disguise, so it goes at its bound.
    pub const EXPOSED: Bits = 16;
    /// Hits this engagement, four bits from here.
    pub const HITS_SHIFT: Bits = 8;
    pub const HITS: Bits = 0xF << HITS_SHIFT;
    /// How high its mark is, in five-centimetre steps, eight bits from here.
    pub const MARK_SHIFT: Bits = 16;
}

/// The height of a mark, in centimetres a step: the word's encoding.
const MARK_STEP: flag::Bits = 5;

/// How many prints the ring keeps: eight seconds at four a second.
pub const PRINTS: usize = 32;
/// How many paint marks at once; a fifth replaces the oldest.
pub const PAINTS: usize = 4;
/// How many quills a fling throws.
pub const QUILL_COUNT: usize = 5;

/// A print's place is kept in eighths of a metre, offset so that ±32 m fits
/// in nine bits: the encoding of a word, not a magnitude.
const PRINT_UNIT: u32 = 8;
const PRINT_OFFSET: u32 = 256;
/// A print's birth is kept modulo this many frames: longer than any print
/// lives (`PrintsSnow` is at most 900), so its age is never ambiguous.
const PRINT_CLOCK: u32 = 1024;
/// A paint mark's birth, likewise.
const PAINT_CLOCK: u32 = 512;
/// A paint point is kept in sixty-fourths of its part's box on each axis.
const PAINT_STEPS: u32 = 63;

/// The smallest number 16.16 holds, guarding a division by a knob retuned to
/// zero.
const SMALLEST: Fx = Fx::from_raw(1);

/// One print, as the ring keeps it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Print {
    pub at: V3,
    /// Which foot: the leg's index.
    pub foot: u8,
    /// Laid through blood: drawn red.
    pub red: bool,
    /// Frames since it was laid.
    pub age: u32,
}

/// One paint mark, as the ring keeps it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Paint {
    pub part: usize,
    /// The point, in the part's own frame.
    pub local: V3,
    /// Frames since it was laid.
    pub age: u32,
}

fn print_word(p: V3, foot: u8, red: bool, frame: u32) -> u32 {
    let q = |v: Fx| {
        ((Fx::from_int(PRINT_UNIT as i32).mul(v).to_int() + PRINT_OFFSET as i32).clamp(0, 511))
            as u32
    };
    q(p.x)
        | q(p.z) << 9
        | (foot as u32 & 3) << 18
        | 1 << 20
        | (red as u32) << 21
        | (frame % PRINT_CLOCK) << 22
}

fn print_of(w: u32, now: u32) -> Option<Print> {
    if w & 1 << 20 == 0 {
        return None;
    }
    let d = |v: u32| Fx::ratio(v as i32 - PRINT_OFFSET as i32, PRINT_UNIT as i32);
    let born = w >> 22;
    Some(Print {
        at: V3::new(d(w & 511), Fx::ZERO, d(w >> 9 & 511)),
        foot: (w >> 18 & 3) as u8,
        red: w >> 21 & 1 != 0,
        age: (now % PRINT_CLOCK + PRINT_CLOCK - born) % PRINT_CLOCK,
    })
}

/// Every print in the ring, with how old it is.
pub fn prints(w: &World) -> impl Iterator<Item = Print> + '_ {
    let now = w.frame;
    (0..PRINTS).filter_map(move |i| print_of(w.lore.word(word::PRINTS + i), now))
}

/// How long a print lasts where it lies: half as long in ash.
pub fn print_life(w: &World, p: &Print) -> u32 {
    match w.arena().floor_at(p.at.x, p.at.z) {
        Material::Ash => Knob::PrintsAsh.raw().max(0) as u32,
        _ => Knob::PrintsSnow.raw().max(0) as u32,
    }
}

fn paint_word(part: usize, q: [i32; 3], frame: u32) -> u32 {
    (part as u32 + 1)
        | (q[0].clamp(0, PAINT_STEPS as i32) as u32) << 5
        | (q[1].clamp(0, PAINT_STEPS as i32) as u32) << 11
        | (q[2].clamp(0, PAINT_STEPS as i32) as u32) << 17
        | (frame % PAINT_CLOCK) << 23
}

fn paint_of(w: u32, now: u32) -> Option<Paint> {
    let part = (w & 31) as usize;
    if part == 0 {
        return None;
    }
    let part = part - 1;
    let shape = SPECIES.shape(part.min(PART_COUNT - 1));
    let size = shape.max.sub(shape.min);
    let at = |q: u32, lo: Fx, span: Fx| lo.add(span.mul(Fx::ratio(q as i32, PAINT_STEPS as i32)));
    let born = w >> 23;
    Some(Paint {
        part,
        local: V3::new(
            at(w >> 5 & 63, shape.min.x, size.x),
            at(w >> 11 & 63, shape.min.y, size.y),
            at(w >> 17 & 63, shape.min.z, size.z),
        ),
        age: (now % PAINT_CLOCK + PAINT_CLOCK - born) % PAINT_CLOCK,
    })
}

/// Every paint mark still lit.
pub fn paints(w: &World) -> impl Iterator<Item = Paint> + '_ {
    let now = w.frame;
    (0..PAINTS).filter_map(move |i| paint_of(w.lore.word(word::PAINT + i), now))
}

/// How bright a paint mark is, nought to one: full until its last
/// `PaintFade` frames, then fading out.
pub fn paint_strength(p: &Paint) -> Fx {
    let life = Knob::PaintFrames.raw().max(1);
    let fade = Knob::PaintFade.raw().clamp(1, life);
    let left = life - p.age as i32;
    if left <= 0 {
        Fx::ZERO
    } else if left >= fade {
        Fx::ONE
    } else {
        Fx::from_int(left).div(Fx::from_int(fade))
    }
}

/// Where a paint mark is in the world this frame: on its part, wherever the
/// body has carried it.
pub fn paint_at(m: &Monster, p: &Paint) -> V3 {
    m.world_of(p.part, p.local)
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

pub fn perched(m: &Monster) -> bool {
    flags(m) & flag::PERCHED != 0
}

pub fn panicked(m: &Monster) -> bool {
    flags(m) & flag::PANICKED != 0
}

pub fn stalking(m: &Monster) -> bool {
    flags(m) & flag::STALKING != 0
}

/// Painted or mottled: seen, whatever its veil does.
pub fn exposed(m: &Monster) -> bool {
    flags(m) & flag::EXPOSED != 0
}

/// Hits it has taken this engagement.
pub fn hits(m: &Monster) -> i32 {
    (flags(m) & flag::HITS) >> flag::HITS_SHIFT
}

fn set_hits(m: &mut Monster, n: i32) {
    m.own[body::FLAGS] = (flags(m) & !flag::HITS) | (n.clamp(0, 15) << flag::HITS_SHIFT);
}

/// The damage a region of hide has taken, all fight.
pub fn region_damage(m: &Monster, region: usize) -> i32 {
    let w = match region {
        0 | 1 => m.own[body::REGIONS_A],
        _ => m.own[body::REGIONS_B],
    } as u32;
    if region % 2 == 0 {
        (w & 0xFFFF) as i32
    } else {
        (w >> 16) as i32
    }
}

fn add_region_damage(m: &mut Monster, region: usize, dealt: i32) {
    let now = (region_damage(m, region) + dealt.max(0)).min(0xFFFF) as u32;
    let i = if region < 2 {
        body::REGIONS_A
    } else {
        body::REGIONS_B
    };
    let w = m.own[i] as u32;
    let w = if region % 2 == 0 {
        (w & 0xFFFF_0000) | now
    } else {
        (w & 0xFFFF) | now << 16
    };
    m.own[i] = w as i32;
}

/// **Has this region taken enough never to cloak again?**
pub fn mottled(m: &Monster, region: usize) -> bool {
    region_damage(m, region) >= Knob::MottleDamage.raw().max(1)
}

/// How many regions are mottled.
pub fn mottled_count(m: &Monster) -> i32 {
    (0..REGIONS).filter(|r| mottled(m, *r)).count() as i32
}

/// Below `DesperateHealth`: its cloak frays, it leaves later, its strain
/// thresholds have fallen, its panic lockout halves (§4).
pub fn desperate(m: &Monster) -> bool {
    below(m, Knob::DesperateHealth.fx())
}

/// Below a share of its full health.
pub fn below(m: &Monster, share: Fx) -> bool {
    let full = if flags(m) & flag::COOP != 0 {
        Knob::CoopHealth.raw()
    } else {
        SPECIES.health()
    };
    m.health < Fx::from_int(full).mul(share).to_int()
}

/// The hits that make it leave: two, or three when desperate.
pub fn leave_after(m: &Monster) -> i32 {
    if desperate(m) {
        Knob::LeaveDesperate.raw()
    } else {
        Knob::LeaveHits.raw()
    }
    .max(1)
}

/// Frames since its move began: through the startup, then the active window.
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

/// **How long a strike's decloak is**: its startup, less what follows the
/// reveal -- the lunge's gather on its haunches, the pounce's flight. What
/// `feel.rs` holds to `DecloakFloor`, and what the report measures a hit's
/// warning by.
pub fn decloak(kind: u8) -> i32 {
    let a = SPECIES.attack(kind);
    let after = match kind {
        LUNGE => Knob::LungeGather.raw(),
        POUNCE => Knob::PounceAir.raw(),
        _ => 0,
    };
    (a.startup as i32 - after).max(0)
}

/// Is this one of its strikes -- a move that begins with a decloak?
pub const fn strikes(kind: u8) -> bool {
    let mut i = 0;
    while i < STRIKES.len() {
        if STRIKES[i] == kind {
            return true;
        }
        i += 1;
    }
    false
}

/// **The body as the move in progress shows it**, before shimmer, paint,
/// mottle and outline: nought cloaked, one in plain view. A strike ramps up
/// from nothing over its decloak's first `DecloakRamp` frames and is in full
/// view through its hit and its recovery; the recoil, a flinch, a stagger, a
/// panic and death are in full view; the climb, the smoke, the bound away
/// and the real animal through a mimic are cloaked.
pub fn veil(m: &Monster) -> Fx {
    match m.doing {
        Doing::Dead | Doing::Flinch { .. } | Doing::Stumble { .. } | Doing::Toppled { .. } => {
            Fx::ONE
        }
        Doing::Prowl => Fx::ZERO,
        Doing::Startup { kind, left } => match kind {
            MIMIC | CLIMB | SMOKE => Fx::ZERO,
            RETREAT | GETUP | RAKE2 => Fx::ONE,
            _ => {
                let e = SPECIES.attack(kind).startup as i32 - left as i32 + 1;
                let ramp = Knob::DecloakRamp.raw().max(1);
                Fx::from_int(e.min(ramp)).div(Fx::from_int(ramp))
            }
        },
        Doing::Active { kind, .. } | Doing::Recovery { kind, .. } => match kind {
            MIMIC | CLIMB | SMOKE | RETREAT => Fx::ZERO,
            _ => Fx::ONE,
        },
    }
}

/// The share of the re-cloak's fade left, from the lore.
fn fade(lore: &Lore) -> Fx {
    let full = Knob::Recloak.raw().max(1);
    Fx::from_int((lore.int(word::FADE)).clamp(0, full)).div(Fx::from_int(full))
}

/// The speed above which it shimmers: lower once its cloak has frayed.
pub fn shimmer_speed(m: &Monster) -> Fx {
    if desperate(m) {
        Knob::FrayedSpeed.fx()
    } else {
        Knob::ShimmerSpeed.fx()
    }
}

/// Is it moving fast enough to shimmer?
pub fn shimmering(m: &Monster) -> bool {
    m.alive() && m.speed.abs().raw() > shimmer_speed(m).raw()
}

/// **How much of one part can be seen**, nought to one: the largest of the
/// decloak (or the re-cloak's fade), the shimmer, a lit paint mark on it, a
/// mottled region, and an outline where it stands in water or fire. What
/// `World::shown` answers for this species; see the module docs.
pub fn shown(w: &World, slot: usize, part: usize) -> Fx {
    let Some(m) = w.monsters.get(slot).copied().flatten() else {
        return Fx::ZERO;
    };
    let part = part.min(PART_COUNT - 1);
    let mut s = veil(&m).max(fade(&w.lore));
    if s.raw() >= Fx::ONE.raw() {
        return Fx::ONE;
    }
    if mottled(&m, REGION[part]) {
        return Fx::ONE;
    }
    if shimmering(&m) {
        s = s.max(Knob::ShimmerShown.fx());
    }
    for p in paints(w).filter(|p| p.part == part) {
        s = s.max(Knob::PaintShown.fx().mul(paint_strength(&p)));
    }
    if outlined(w, &m, part) {
        s = s.max(Knob::OutlineShown.fx());
    }
    s
}

/// Is any of the body shown at all? What the brain calls being seen, and the
/// exposure term's "painted or mottled".
pub fn any_shown(w: &World, slot: usize) -> bool {
    (0..PART_COUNT).any(|p| shown(w, slot, p).raw() > 0)
}

/// **A part standing in something that draws it**: the stream (a ripple ring
/// round each leg), or fire. Measured at the part's place at rest, carried by
/// the body's position and facing -- cheap enough to ask for every part every
/// frame, and a leg in the stream is a leg in the stream whatever it is
/// doing.
fn outlined(w: &World, m: &Monster, part: usize) -> bool {
    let at = part_rest_world(m, part);
    let floor = w.arena().ground_under(at);
    if at.y.sub(floor).raw() < Knob::OutlineHigh.fx().raw()
        && w.arena().floor_at(at.x, at.z) == Material::Water
        && floor.raw() <= Fx::ZERO.raw()
    {
        return true;
    }
    in_fire_at(w, at, Fx::ZERO).is_some()
}

/// Where a part's middle stands with the body at rest: its box's middle in
/// its bone's rest frame, carried by the body's position and facing. Not the
/// posed part -- the rig is too dear to build per part per frame -- but
/// within a reach of it.
pub fn part_rest_world(m: &Monster, part: usize) -> V3 {
    let sp = &SPECIES;
    let shape = sp.shape(part);
    let mid = shape.min.add(shape.max).scale(math::half(Fx::ONE));
    let mut sum = mid;
    let mut b = shape.bone;
    loop {
        sum = sum.add(sp.rest(b));
        let parent = sp.bones[b].parent;
        if parent == crate::beast::NO_PARENT {
            break;
        }
        b = parent;
    }
    let f = V3::from_turns(m.yaw);
    let side = V3::new(f.z.neg(), Fx::ZERO, f.x);
    m.pos
        .add(f.scale(sum.x))
        .add(side.scale(sum.z))
        .add(V3::new(Fx::ZERO, sum.y, Fx::ZERO))
}

// ---------------------------------------------------------------------------
// Fire
// ---------------------------------------------------------------------------

/// What set a fire: the report's causes of a panic.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FireFrom {
    Pillar,
    Embers,
    Stone,
    Brazier,
    Other,
}

impl FireFrom {
    pub const ALL: [FireFrom; 5] = [
        FireFrom::Pillar,
        FireFrom::Embers,
        FireFrom::Stone,
        FireFrom::Brazier,
        FireFrom::Other,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            FireFrom::Pillar => "pillar",
            FireFrom::Embers => "cloud of embers",
            FireFrom::Stone => "lit stone",
            FireFrom::Brazier => "brazier",
            FireFrom::Other => "other fire",
        }
    }
}

/// **Is there fire at this point**, flat, within `pad` of it -- and from
/// what? Every fire the world has (`World::fires`: pillars, tornados, embers,
/// the fire ring, lit stones, bolts) and every burning hazard on the floor:
/// the coals, the flash of a cloud burning off.
pub fn in_fire_at(w: &World, at: V3, pad: Fx) -> Option<FireFrom> {
    for e in w.effects.iter().flatten() {
        if !e.kind.ignites() {
            continue;
        }
        let (c, r) = match e.kind {
            EffectKind::FireTornado => (e.tornado_pos(), e.field_radius()),
            EffectKind::FireRing => (e.pos, e.ring_radius()),
            _ => (e.pos, e.field_radius()),
        };
        if math::wide_flat_dist(c, at).raw() <= r.add(pad).raw() {
            return Some(match e.kind {
                EffectKind::FirePillar => FireFrom::Pillar,
                EffectKind::Embers => FireFrom::Embers,
                _ => FireFrom::Other,
            });
        }
    }
    for stone in crate::stones::gather(&w.players).iter().flatten() {
        if stone.lit > 0
            && math::wide_flat_dist(stone.at, at).raw() <= stone.radius().add(pad).raw()
        {
            return Some(FireFrom::Stone);
        }
    }
    for (_, h) in hazard::all(&w.lore) {
        let kind = h.index();
        if kind != COALS && kind != FLASH {
            continue;
        }
        if math::wide_flat_dist(h.centre(), at).raw() <= h.radius().add(pad).raw() {
            return Some(if kind == COALS {
                FireFrom::Brazier
            } else {
                FireFrom::Other
            });
        }
    }
    None
}

/// **The nearest fire to a point**, within `within` of its edge: where its
/// middle is. What the creature keeps its distance from -- it sees fire and
/// will not walk into it (§4).
pub fn nearest_fire(w: &World, at: V3, within: Fx) -> Option<V3> {
    let mut best: Option<(V3, Fx)> = None;
    let mut consider = |c: V3, r: Fx| {
        let gap = math::wide_flat_dist(c, at).sub(r);
        if gap.raw() <= within.raw() && best.is_none_or(|(_, b)| gap.raw() < b.raw()) {
            best = Some((c, gap));
        }
    };
    for e in w.effects.iter().flatten() {
        if !e.kind.ignites() {
            continue;
        }
        match e.kind {
            EffectKind::FireTornado => consider(e.tornado_pos(), e.field_radius()),
            EffectKind::FireRing => consider(e.pos, e.ring_radius()),
            _ => consider(e.pos, e.field_radius()),
        }
    }
    for stone in crate::stones::gather(&w.players).iter().flatten() {
        if stone.lit > 0 {
            consider(stone.at, stone.radius());
        }
    }
    for (_, h) in hazard::all(&w.lore) {
        if h.index() == COALS || h.index() == FLASH {
            consider(h.centre(), h.radius());
        }
    }
    best.map(|(c, _)| c)
}

/// The fire it is keeping clear of, as the frame hook last saw it.
pub fn fire_near(lore: &Lore) -> Option<V3> {
    (lore.word(word::FIRE_NEAR) != 0).then(|| point(lore.word(word::FIRE_AT)))
}

// ---------------------------------------------------------------------------
// Hooks the shared machinery calls
// ---------------------------------------------------------------------------

/// **It goes at its slink while it stalks**: never faster than
/// `StalkSpeed`, which a fighter walking away outpaces. Out of the stalk it
/// bounds at its gallop, and shimmers doing it.
pub fn pace(m: &Monster) -> Fx {
    if stalking(m) && flags(m) & flag::EXPOSED == 0 {
        let gallop = SPECIES.gallop().max(SMALLEST);
        Knob::StalkSpeed.fx().div(gallop).min(Fx::ONE)
    } else {
        Fx::ONE
    }
}

/// **A body in the air is passed under**: there to hit, nothing to walk into,
/// through the climb's leap and the pounce's flight.
fn presence(m: &Monster, _rig: &crate::beast::Rig) -> crate::beast::Presence {
    crate::beast::Presence {
        passable: if airborne(m) { u64::MAX } else { 0 },
        ..Default::default()
    }
}

/// How high the pounce's circle is drawn: the floor it lands on.
pub fn lob_height(m: &Monster) -> Fx {
    let steps = (m.own[body::FLAGS] >> flag::MARK_SHIFT) & 0xFF;
    lore::from_cm((steps * MARK_STEP) as i16)
}

fn set_mark_height(m: &mut Monster, y: Fx) {
    let steps = (lore::to_cm(y) as i32 / MARK_STEP).clamp(0, 0xFF);
    m.own[body::FLAGS] =
        (m.own[body::FLAGS] & !(0xFF << flag::MARK_SHIFT)) | (steps << flag::MARK_SHIFT);
}

/// **A hit has landed.** Its region keeps the damage (past `MottleDamage` it
/// never cloaks again); the part is remembered for the frame hook to paint
/// where it landed. A burst past the interrupt bar in the retreat's recoil
/// breaks the retreat: it stays, staggered and in full view, and the
/// engagement goes on (§4).
pub fn struck(m: &mut Monster, part: usize, dealt: i32) -> bool {
    if dealt <= 0 {
        return false;
    }
    let part = part.min(PART_COUNT - 1);
    add_region_damage(m, REGION[part], dealt);
    m.own[body::STRUCK] |= 1 << part;
    if matches!(m.doing, Doing::Startup { kind: RETREAT, .. }) && m.strain >= m.interrupt_bar() {
        m.strain = 0;
        m.doing = Doing::Flinch {
            left: Knob::RetreatStagger.raw().clamp(1, u16::MAX as i32) as u16,
        };
        m.speed = Fx::ZERO;
        set_hits(m, 0);
        return true;
    }
    false
}

// ---------------------------------------------------------------------------
// Once a frame
// ---------------------------------------------------------------------------

/// The frame hook. See the module docs.
pub fn frame(w: &mut World) {
    let Some(slot) = (0..MAX_MONSTERS)
        .find(|s| w.monsters[*s].is_some_and(|m| m.species == SpeciesId::VEILSTALKER))
    else {
        return;
    };
    setup(w, slot);
    glance_looks(w, slot);
    let mut m = w.monsters[slot].expect("found above");
    let now = w.frame;

    hits_and_paint(w, &mut m, now);
    note_bait(w, &m);
    if m.alive() {
        leave(w, &mut m);
        chains(w, &mut m);
        spear_lock(w, &mut m);
        perch_spots(w, &m);
        fly(w, &mut m);
        quills(w, &mut m, slot);
        smoke(w, &m, now);
        mimic(w, &mut m, now);
        panic(w, &mut m, now);
    }
    let exposed = paints(w).next().is_some() || mottled_count(&m) > 0;
    set_flag(&mut m, flag::EXPOSED, exposed);
    trail(w, &m, now);
    braziers(w);
    veil_clock(w, &mut m);
    sense(w, &m, slot);
    w.lore.set_word(word::NOW, now);
    w.monsters[slot] = Some(m);
}

/// On the first frame: the coop numbers, and the first stalk.
fn setup(w: &mut World, slot: usize) {
    if w.lore.word(word::FLAGS) & 1 != 0 {
        return;
    }
    let mut flags_now = 1;
    let hunters = w.players.iter().filter(|p| p.health > 0).count();
    let Some(m) = w.monsters[slot].as_mut() else {
        return;
    };
    if hunters >= 2 {
        flags_now |= 2;
        m.health = Knob::CoopHealth.raw().max(1);
        m.own[body::FLAGS] |= flag::COOP;
    }
    m.own[body::FLAGS] |= flag::STALKING;
    // The first stalk begins when the hunt's grace ends, not under it: the
    // grace is the hunter's moment to get their bearings, and a stalk run
    // down inside it is an opening look the hunter never had to sit through.
    let stalk = stalk_frames(m) + m.sp().hunt_grace() as u32;
    w.lore.set_word(word::FLAGS, flags_now);
    w.lore.set_word(word::STALK, stalk);
}

/// A stalk's length, drawn from the creature's own generator: shortened by a
/// quarter with two hunters.
fn stalk_frames(m: &mut Monster) -> u32 {
    let lo = Knob::StalkMin.raw().max(0);
    let hi = Knob::StalkMax.raw().max(lo);
    let frames = lo + (draw(m) % (hi - lo + 1) as u32) as i32;
    if flags(m) & flag::COOP != 0 {
        Fx::from_int(frames)
            .mul(Knob::CoopStalk.fx())
            .to_int()
            .max(0) as u32
    } else {
        frames as u32
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

/// **It glances your look**: every glance, where each hunter stands and which
/// way they face -- the character's head and shoulders, which is the camera's
/// yaw whenever they can act -- with the glance before it, so the brain can
/// tell a camera that is turning (§5). It acts on this, at least one glance
/// old.
fn glance_looks(w: &mut World, slot: usize) {
    let left = w.lore.word(word::GLANCE_LEFT);
    if left > 0 {
        w.lore.set_word(word::GLANCE_LEFT, left - 1);
        return;
    }
    let every = w.monsters[slot].map_or(1, |m| m.glance_frames().max(1));
    w.lore.set_word(word::GLANCE_LEFT, every as u32 - 1);
    for i in 0..MAX_PLAYERS.min(2) {
        let p = w.players[i];
        if p.health <= 0 {
            continue;
        }
        let yaw = math::atan2_turns(p.facing.z, p.facing.x);
        let was = w.lore.word(word::GLANCED + 2 + i);
        let then = lore::lo(was);
        let now = (yaw.raw() & 0xFFFF) as u16 as i16;
        // The first glance has nothing before it: the look then is the look.
        let then = if w.lore.word(word::GLANCED + i) == 0 && was == 0 {
            now
        } else {
            then
        };
        w.lore.set_word(word::GLANCED + i, word_of(p.pos));
        w.lore
            .set_word(word::GLANCED + 2 + i, lore::halves(now, then));
    }
}

/// Where hunter `i` stood at the last glance, and which way they looked then
/// and the glance before (turns).
pub fn glanced(lore: &Lore, i: usize) -> (V3, Fx, Fx) {
    let w = lore.word(word::GLANCED + 2 + i.min(1));
    let turns = |h: i16| Fx::from_raw(h as u16 as i32);
    (
        point(lore.word(word::GLANCED + i.min(1))),
        turns(lore::lo(w)),
        turns(lore::hi(w)),
    )
}

/// **The look it acts on**: the last glance, led by how far the camera turned
/// between the last two -- the same lead it takes on a position, applied to
/// a look. A camera sweeping past is met ahead of where it was.
pub fn led_look(lore: &Lore, i: usize) -> (V3, Fx) {
    let (at, now, then) = glanced(lore, i);
    let turned = math::wrap_turns(now.sub(then));
    (at, now.add(math::half(turned)))
}

/// The hits of this engagement, and the paint where each one landed.
fn hits_and_paint(w: &mut World, m: &mut Monster, now: u32) {
    let struck = m.own[body::STRUCK];
    if struck == 0 {
        return;
    }
    m.own[body::STRUCK] = 0;
    // **One hit, not one frame of a beam**: a hit that lands within a few
    // frames of the last is the same hit, so a channel ticking on it is one.
    let last = w.lore.word(word::LAST_HIT);
    let gap = Knob::HitGap.raw().max(1) as u32;
    // Leaving, it is not engaged: a blow on its way out paints it and is not
    // the first of the next engagement.
    let leaving = matches!(m.doing.attacking(), Some(RETREAT | GETUP));
    if !leaving && (last == 0 || now.wrapping_sub(last.wrapping_sub(1)) >= gap) {
        set_hits(m, hits(m) + 1);
    }
    w.lore.set_word(word::LAST_HIT, now.wrapping_add(1));
    // Where it landed: the point of the struck part nearest whoever is
    // nearest -- the blade, or the shot's end, came from that side.
    let Some(from) = w
        .players
        .iter()
        .filter(|p| p.health > 0)
        .min_by_key(|p| math::wide_flat_dist(p.pos, m.pos).raw())
        .map(|p| aim::origin(p.pos))
    else {
        return;
    };
    let rig = m.rig();
    for part in 0..PART_COUNT {
        if struck & 1 << part == 0 {
            continue;
        }
        let shape = SPECIES.shape(part);
        let local = rig.world_to_part(part, from);
        let clamp = |v: Fx, lo: Fx, hi: Fx| v.clamp(lo, hi);
        let near = V3::new(
            clamp(local.x, shape.min.x, shape.max.x),
            clamp(local.y, shape.min.y, shape.max.y),
            clamp(local.z, shape.min.z, shape.max.z),
        );
        let size = shape.max.sub(shape.min);
        let q = |v: Fx, lo: Fx, span: Fx| {
            if span.raw() <= 0 {
                0
            } else {
                v.sub(lo)
                    .div(span)
                    .mul(Fx::from_int(PAINT_STEPS as i32))
                    .to_int()
            }
        };
        let cell = paint_word(
            part,
            [
                q(near.x, shape.min.x, size.x),
                q(near.y, shape.min.y, size.y),
                q(near.z, shape.min.z, size.z),
            ],
            now,
        );
        // A fifth replaces the oldest: an empty slot first, then the oldest.
        let slot = (0..PAINTS)
            .max_by_key(|i| paint_of(w.lore.word(word::PAINT + i), now).map_or(u32::MAX, |p| p.age))
            .unwrap_or(0);
        w.lore.set_word(word::PAINT + slot, cell);
    }
}

/// **After two hits it leaves** (three, desperate): a recoil in full view, a
/// bound away, a re-cloak. Not from the air, a panic, or a retreat already.
fn leave(w: &mut World, m: &mut Monster) {
    if hits(m) < leave_after(m) {
        return;
    }
    let busy = matches!(
        m.doing,
        Doing::Toppled { .. } | Doing::Dead | Doing::Stumble { .. }
    ) || m
        .doing
        .attacking()
        .is_some_and(|k| matches!(k, RETREAT | GETUP | POUNCE | CLIMB))
        || perched(m);
    if busy {
        return;
    }
    let _ = w;
    set_hits(m, 0);
    m.doing = Doing::Startup {
        kind: RETREAT,
        left: SPECIES.attack(RETREAT).startup,
    };
    m.hit_used = false;
    m.speed = Fx::ZERO;
    m.brain.last_move = RETREAT;
}

/// The moves that run into each other: the rake's second swipe, the getting
/// up after a panic and the bound that follows it, and the retreat turning
/// away as its recoil ends.
fn chains(w: &mut World, m: &mut Monster) {
    let bounds = w.arena().bounds;
    let away = |m: &Monster| open_way(m.pos, m.brain.seen, Knob::RetreatFar.fx(), &bounds);
    match m.doing {
        Doing::Recovery {
            kind: RAKE,
            left: 0,
        } => {
            m.doing = Doing::Startup {
                kind: RAKE2,
                left: SPECIES.attack(RAKE2).startup,
            };
            m.hit_used = false;
            m.brain.last_move = RAKE2;
        }
        // Out of a panic: a moment up and in view, then away.
        Doing::Toppled { left: 0 } if panicked(m) => {
            set_flag(m, flag::PANICKED, false);
            m.doing = Doing::Startup {
                kind: GETUP,
                left: SPECIES.attack(GETUP).startup,
            };
            m.hit_used = false;
            m.brain.last_move = GETUP;
        }
        Doing::Startup {
            kind: GETUP,
            left: 0,
        } => {
            m.yaw = away(m);
            m.yaw_rate = Fx::ZERO;
            m.doing = Doing::Active {
                kind: RETREAT,
                left: SPECIES.attack(RETREAT).active,
            };
            m.brain.last_move = RETREAT;
        }
        Doing::Startup {
            kind: RETREAT,
            left: 0,
        } => {
            m.yaw = away(m);
            m.yaw_rate = Fx::ZERO;
        }
        // The lunge lands where its flight ends, and stops: its recovery is
        // spent where you can walk up to it.
        Doing::Recovery { kind: LUNGE, .. } | Doing::Recovery { kind: RETREAT, .. } => {
            m.speed = Fx::ZERO;
        }
        _ => {}
    }
    // The bound away is aimed off the arena's walls: along them rather than
    // into them.
    if let Doing::Active { kind: RETREAT, .. } = m.doing {
        let b = w.arena().bounds;
        let f = V3::from_turns(m.yaw);
        let ahead = m.pos.add(f.scale(Knob::WallLook.fx()));
        let room = SPECIES.margin().add(Fx::ONE);
        let out = ahead.x.raw() < b.lo_x.add(room).raw()
            || ahead.x.raw() > b.hi_x.sub(room).raw()
            || ahead.z.raw() < b.lo_z.add(room).raw()
            || ahead.z.raw() > b.hi_z.sub(room).raw();
        if out {
            let middle = flat(V3::ZERO.sub(m.pos));
            if middle.flat_len().raw() > 0 {
                let want = math::atan2_turns(middle.z, middle.x);
                let error = math::wrap_turns(want.sub(m.yaw));
                let step = Knob::WallTurn.fx();
                m.yaw = m.yaw.add(error.clamp(step.neg(), step));
            }
        }
    }
}

/// **The way away from `from` with the most floor in it**: straight away if
/// a run of `far` that way stays inside the walls, otherwise turned toward
/// the middle of the arena an eighth at a time until it does -- a retreat
/// into a corner is a retreat to be cornered in.
pub fn open_way(at: V3, from: V3, far: Fx, b: &crate::arena::Bounds) -> Fx {
    let off = flat(at.sub(from));
    let straight = if off.flat_len().raw() > 0 {
        math::atan2_turns(off.z, off.x)
    } else {
        Fx::ZERO
    };
    let room = SPECIES.margin().add(Knob::WallLook.fx());
    let fits = |yaw: Fx| {
        let end = at.add(V3::from_turns(yaw).scale(far));
        end.x.raw() > b.lo_x.add(room).raw()
            && end.x.raw() < b.hi_x.sub(room).raw()
            && end.z.raw() > b.lo_z.add(room).raw()
            && end.z.raw() < b.hi_z.sub(room).raw()
    };
    let middle = flat(V3::ZERO.sub(at));
    let toward = if middle.flat_len().raw() > 0 {
        math::wrap_turns(math::atan2_turns(middle.z, middle.x).sub(straight))
    } else {
        Fx::ZERO
    };
    let turn = Knob::WayStep.fx();
    let step = if toward.raw() >= 0 { turn } else { turn.neg() };
    let mut yaw = straight;
    for _ in 0..4 {
        if fits(yaw) {
            return yaw;
        }
        yaw = yaw.add(step);
    }
    // Nowhere fits: toward the middle.
    straight.add(toward)
}

/// **The spear's lane locks** at `SpearLock` frames into its tell: it follows
/// you for half of it, then stops, so a walk out of the line before then is
/// the answer (§2).
fn spear_lock(w: &mut World, m: &mut Monster) {
    match elapsed(m) {
        Some((SPEAR, e)) if matches!(m.doing, Doing::Startup { .. } | Doing::Active { .. }) => {
            let locked = w.lore.word(word::SPEAR_YAW);
            if locked >> 16 != 0 {
                m.yaw = Fx::from_raw((locked & 0xFFFF) as i32);
                m.yaw_rate = Fx::ZERO;
            } else if e >= Knob::SpearLock.raw() {
                let yaw = math::wrap_turns(m.yaw);
                let raw = (yaw.raw() & 0xFFFF) as u32;
                w.lore.set_word(word::SPEAR_YAW, raw | 1 << 16);
                m.yaw = Fx::from_raw(raw as i32);
                m.yaw_rate = Fx::ZERO;
            }
        }
        _ => w.lore.set_word(word::SPEAR_YAW, 0),
    }
}

// ---------------------------------------------------------------------------
// The perch, and the leaps
// ---------------------------------------------------------------------------

/// **Where it would perch**: a top within its reach that is wide enough and
/// between `PerchLowest` and `PerchHighest` -- a trunk's broken top, the
/// wall's ledge -- the one nearest the target, so the pounce from it reaches.
fn perch_spots(w: &mut World, m: &Monster) {
    w.lore.set_word(word::PERCH_TOP, 0);
    if perched(m) {
        return;
    }
    let ground = w.terrain();
    let wide = Knob::PerchWidth.fx();
    let inset = math::half(wide);
    let reach = Knob::PerchReach.fx();
    let target = m.brain.seen;
    let mut best: Option<(V3, Fx, Fx)> = None;
    for solid in ground.solids() {
        if solid.hangs() {
            continue;
        }
        let top = solid.max.y;
        if top.raw() < Knob::PerchLowest.fx().raw() || top.raw() > Knob::PerchHighest.fx().raw() {
            continue;
        }
        let wx = solid.max.x.sub(solid.min.x);
        let wz = solid.max.z.sub(solid.min.z);
        if wx.raw() < wide.raw() || wz.raw() < wide.raw() {
            continue;
        }
        let x = target
            .x
            .clamp(solid.min.x.add(inset), solid.max.x.sub(inset));
        let z = target
            .z
            .clamp(solid.min.z.add(inset), solid.max.z.sub(inset));
        // The nearest point of the top to the creature is where it gets on.
        let at = V3::new(x, Fx::ZERO, z);
        if math::wide_flat_dist(at, m.pos).raw() > reach.raw() {
            continue;
        }
        let score = math::wide_flat_dist(at, target);
        if best.is_none_or(|(_, _, b)| score.raw() < b.raw()) {
            best = Some((at, top, score));
        }
    }
    if let Some((at, top, _)) = best {
        w.lore.set_word(word::PERCH_AT, word_of(at));
        w.lore.set_int(word::PERCH_TOP, lore::to_cm(top) as i32);
    }
}

/// The top it would perch on, if there is one in reach.
pub fn perch_of(lore: &Lore) -> Option<(V3, Fx)> {
    let top = lore.int(word::PERCH_TOP);
    (top > 0).then(|| (point(lore.word(word::PERCH_AT)), lore::from_cm(top as i16)))
}

/// Where a leap leaves the ground, in frames since its move began, and how
/// many frames it flies.
pub fn flight(kind: u8) -> (i32, i32) {
    let a = SPECIES.attack(kind);
    match kind {
        // The pounce: off the perch as its decloak ends, down as its hit
        // begins.
        POUNCE => {
            let leave = decloak(POUNCE);
            (leave, (a.startup as i32 - leave).max(1))
        }
        // The climb: up through the whole of its startup and active.
        _ => (0, (a.startup as i32 + a.active as i32).max(1)),
    }
}

/// Is it in the air: past the frame its leap left, and not yet down?
pub fn airborne(m: &Monster) -> bool {
    let Some((kind, e)) = elapsed(m) else {
        return false;
    };
    if !matches!(kind, POUNCE | CLIMB) || matches!(m.doing, Doing::Recovery { .. }) {
        return false;
    }
    let (leave, air) = flight(kind);
    e > leave && e <= leave + air
}

/// Aim the pounce at where the target stands now, on the floor -- positional,
/// not followed (§2) -- and remember how high the floor there is.
pub fn mark_pounce(m: &mut Monster, at: V3, ground: &crate::arena::Terrain) {
    let at = V3::new(at.x, Fx::ZERO, at.z);
    m.aim_at(at);
    set_mark_height(m, ground.ground_under(at));
}

/// **Fly a leap**: the climb from where it stood onto its perch's top, the
/// pounce from the perch to its circle -- a straight line, its height
/// carried from where it left to where it lands with an arc over it. Out of
/// a leap it stands on whatever is under it, perched or on the floor, and
/// comes down off an edge.
fn fly(w: &mut World, m: &mut Monster) {
    let ground = w.terrain();
    let under = ground.ground_under(m.pos);
    let in_leap = elapsed(m).filter(|(k, _)| {
        matches!(*k, POUNCE | CLIMB) && !matches!(m.doing, Doing::Recovery { .. })
    });
    let Some((kind, e)) = in_leap else {
        // Perched: rooted on its top, and its patience.
        if perched(m) {
            m.rooted = m.rooted.max(2);
            m.speed = Fx::ZERO;
            let up = lore::hi(w.lore.word(word::LEAP_UP)) as i32 + 1;
            let from = lore::lo(w.lore.word(word::LEAP_UP));
            w.lore.set_word(
                word::LEAP_UP,
                lore::halves(from, up.min(i16::MAX as i32) as i16),
            );
            if m.doing.free() && up > Knob::PerchPatience.raw() {
                // Down the cloaked way: off the edge toward the target.
                let to = flat(m.brain.seen.sub(m.pos));
                let dir = if to.flat_len().raw() > 0 {
                    math::wide_normalized(to)
                } else {
                    V3::from_turns(m.yaw)
                };
                let out = m.pos.add(dir.scale(Knob::PerchDrop.fx()));
                m.aim_at(out);
                m.doing = Doing::Startup {
                    kind: CLIMB,
                    left: SPECIES.attack(CLIMB).startup,
                };
                m.brain.last_move = CLIMB;
                m.hit_used = false;
            }
            if under.raw() < m.pos.y.sub(Fx::ratio(1, 10)).raw() {
                set_flag(m, flag::PERCHED, false);
            }
        }
        if !perched(m) {
            m.pos.y = if m.pos.y.raw() > under.raw() {
                m.pos.y.sub(Knob::FallSpeed.fx().mul(DT)).max(under)
            } else {
                under
            };
        }
        return;
    };
    let (leave, air) = flight(kind);
    if e == leave || (kind == CLIMB && e == 1) {
        // Off: where from, how high, facing the mark.
        w.lore.set_word(word::LEAP_FROM, word_of(m.pos));
        w.lore
            .set_word(word::LEAP_UP, lore::halves(lore::to_cm(m.pos.y), 0));
        let to = flat(m.aimed_at().sub(m.pos));
        if to.flat_len().raw() > 0 {
            m.yaw = math::atan2_turns(to.z, to.x);
            m.yaw_rate = Fx::ZERO;
        }
        set_flag(m, flag::PERCHED, false);
        m.speed = Fx::ZERO;
        return;
    }
    if e < leave {
        // On the perch through the decloak: still, in full view.
        m.rooted = m.rooted.max(2);
        m.speed = Fx::ZERO;
        return;
    }
    let from = point(w.lore.word(word::LEAP_FROM));
    let from_y = lore::from_cm(lore::lo(w.lore.word(word::LEAP_UP)));
    let mark = m.aimed_at();
    let to = flat(mark.sub(from));
    let dir = if to.flat_len().raw() > 0 {
        math::wide_normalized(to)
    } else {
        V3::from_turns(m.yaw)
    };
    let short = if kind == POUNCE {
        Knob::LandShort.fx().min(math::wide_flat_len(to))
    } else {
        Fx::ZERO
    };
    let end = mark.sub(dir.scale(short));
    let t = (e - leave).min(air);
    let u = Fx::from_int(t).div(Fx::from_int(air));
    let land_y = ground.ground_under(end);
    let arc = if kind == POUNCE {
        Knob::PounceArc.fx()
    } else {
        Knob::ClimbArc.fx()
    };
    let at = math::lerp3(from, end, u);
    m.pos = V3::new(
        at.x,
        math::lerp(from_y, land_y, u).add(arc.mul(math::hump(u))),
        at.z,
    );
    m.speed = Fx::ZERO;
    if kind == CLIMB && t >= air {
        // Up: perched if it landed on a top, on the floor if it came down.
        let top = land_y.raw() > Fx::ONE.raw();
        set_flag(m, flag::PERCHED, top);
        w.lore
            .set_word(word::LEAP_UP, lore::halves(lore::to_cm(land_y), 0));
        m.doing = Doing::Recovery {
            kind: CLIMB,
            left: SPECIES.attack(CLIMB).recovery,
        };
    }
}

// ---------------------------------------------------------------------------
// The quills
// ---------------------------------------------------------------------------

/// One quill in flight: where it is, and where it was a frame ago.
#[derive(Clone, Copy, Debug)]
pub struct Quill {
    pub at: V3,
    pub was: V3,
    pub index: usize,
}

/// The bearing of quill `i` off the fan's middle.
fn quill_bearing(i: usize) -> Fx {
    let half = Knob::QuillFan.fx();
    let span = (QUILL_COUNT - 1) as i32;
    half.neg().add(
        half.add(half)
            .mul(Fx::from_int(i as i32))
            .div(Fx::from_int(span.max(1))),
    )
}

/// **Where a fling goes**: from the tail, at the target's chest where it is
/// led to -- the place it leaves, the fan's middle bearing, and the fall (or
/// rise) per metre. What the throw uses and what the floor shows.
pub fn quill_fan(m: &Monster) -> (V3, Fx, Fx) {
    let a = SPECIES.attack(QUILLS);
    let tip = m.rig().bone[super::bones::TAIL4].at;
    let from = V3::new(tip.x, Fx::ZERO, tip.z);
    let height = Knob::QuillHeight.fx();
    let target = m.lead_point(a.startup);
    let chest = aim::standing_middle(target, crate::tuning::body_height());
    let to = flat(chest.sub(from));
    let far = math::wide_flat_len(to).max(SMALLEST);
    let middle = math::atan2_turns(to.z, to.x);
    (from, middle, chest.y.sub(height).div(far))
}

/// **The fan on the floor**: through the decloak, a strip down each quill's
/// line as it would leave now, filling as the rear rises; in flight, the
/// lines they are flying along. The quills are the species' own hit, with
/// no cylinder for `Monster::telegraph` to draw, so this is their marker --
/// built from [`quill_fan`] and [`quill_bearing`], what the throw itself uses.
fn signs(w: &World, out: &mut crate::sign::Signs) {
    use crate::sign::{Says, Sign};
    let Some(m) = w
        .monsters
        .iter()
        .flatten()
        .find(|m| m.species == SpeciesId::VEILSTALKER && m.alive())
    else {
        return;
    };
    // Across: a quill's width and a body either side of its line.
    let half = Knob::QuillRadius.fx().add(crate::tuning::body_radius());
    let width = half.add(half);
    let reach = Knob::QuillReach.fx();
    match m.doing {
        Doing::Startup { kind: QUILLS, left } => {
            let a = SPECIES.attack(QUILLS);
            let progress = Fx::from_int((a.startup - left.min(a.startup)) as i32)
                .div(Fx::from_int(a.startup.max(1) as i32));
            let (from, middle, _) = quill_fan(m);
            for i in 0..QUILL_COUNT {
                let along = V3::from_turns(middle.add(quill_bearing(i)));
                out.push(Sign::strip(Says::Coming, from, along, reach, width).filled(progress));
            }
        }
        _ => {
            let from = point(w.lore.word(word::QUILL_AT));
            let up = w.lore.word(word::QUILL_UP);
            let middle = Fx::from_raw(lore::hi(up) as u16 as i32);
            let live = w.lore.word(word::QUILL_LIVE);
            for i in 0..QUILL_COUNT {
                if live & 1 << i == 0 {
                    continue;
                }
                let along = V3::from_turns(middle.add(quill_bearing(i)));
                out.push(Sign::strip(Says::Live, from, along, reach, width));
            }
        }
    }
}

/// **Every quill still flying**, this frame and last.
pub fn quills_in_flight(w: &World) -> impl Iterator<Item = Quill> + '_ {
    let live = w.lore.word(word::QUILL_LIVE);
    let from = point(w.lore.word(word::QUILL_AT));
    let up = w.lore.word(word::QUILL_UP);
    let height = lore::from_cm(lore::lo(up));
    let middle = Fx::from_raw(lore::hi(up) as u16 as i32);
    let slope = Fx::from_raw(w.lore.int(word::QUILL_SLOPE));
    let flown = w.frame.wrapping_sub(w.lore.word(word::QUILL_FRAME)) as i32;
    let speed = Knob::QuillSpeed.fx().mul(DT);
    let along = move |t: i32, i: usize| {
        let d = speed.mul(Fx::from_int(t.max(0)));
        let dir = V3::from_turns(middle.add(quill_bearing(i)));
        V3::new(
            from.x.add(dir.x.mul(d)),
            height.add(slope.mul(d)),
            from.z.add(dir.z.mul(d)),
        )
    };
    (0..QUILL_COUNT)
        .filter(move |i| live & 1 << i != 0)
        .map(move |i| Quill {
            at: along(flown, i),
            was: along(flown - 1, i),
            index: i,
        })
}

/// **The fling**: on its first active frame five quills leave in a flat fan at
/// the target's chest, then fly on at `QuillSpeed`; each stops at the first
/// solid on its line -- a trunk, a stone, a planted shield -- or the first
/// body it meets, or at `QuillReach`.
fn quills(w: &mut World, m: &mut Monster, slot: usize) {
    if let Doing::Active { kind: QUILLS, left } = m.doing {
        let a = SPECIES.attack(QUILLS);
        if left == a.active {
            // Thrown: from its tail, raised over its head, at where it led
            // you to.
            let (from, middle, slope) = quill_fan(m);
            let height = Knob::QuillHeight.fx();
            w.lore.set_word(word::QUILL_AT, word_of(from));
            w.lore.set_word(
                word::QUILL_UP,
                lore::halves(lore::to_cm(height), (middle.raw() & 0xFFFF) as u16 as i16),
            );
            w.lore.set_int(word::QUILL_SLOPE, slope.raw());
            w.lore.set_word(word::QUILL_FRAME, w.frame);
            w.lore.set_word(word::QUILL_LIVE, (1 << QUILL_COUNT) - 1);
            let _ = slot;
        }
    }
    let live = w.lore.word(word::QUILL_LIVE);
    if live == 0 {
        return;
    }
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
    let reach = Knob::QuillReach.fx();
    let from = point(w.lore.word(word::QUILL_AT));
    let a = SPECIES.attack(QUILLS);
    let mut still = live;
    let flying: [Option<Quill>; QUILL_COUNT] = {
        let mut out = [None; QUILL_COUNT];
        for q in quills_in_flight(w) {
            out[q.index] = Some(q);
        }
        out
    };
    for q in flying.iter().flatten() {
        let bit = 1 << q.index;
        if math::wide_flat_dist(q.at, from).raw() > reach.raw() || !ground.inside(q.at) {
            still &= !bit;
            continue;
        }
        if !aim::line_clear(q.was, q.at, &scene) || q.at.y.raw() <= ground.ground_under(q.at).raw()
        {
            still &= !bit;
            continue;
        }
        for (i, p) in fighters.iter().enumerate() {
            if p.health <= 0 || p.action.invulnerable() {
                continue;
            }
            let feet = p.pos;
            let head = p.pos.add(V3::new(Fx::ZERO, p.hurt_height(), Fx::ZERO));
            let gap = math::segment_gap(q.was, q.at, feet, head);
            if gap.raw()
                > Knob::QuillRadius
                    .fx()
                    .add(crate::tuning::body_radius())
                    .raw()
            {
                continue;
            }
            still &= !bit;
            let dir = math::wide_normalized(flat(q.at.sub(q.was)));
            let facing_it = p.facing.dot(dir.scale(Fx::ONE.neg())).raw()
                >= crate::tuning::guard_arc_cos().raw();
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
            m.hit_used = true;
            break;
        }
    }
    w.lore.set_word(word::QUILL_LIVE, still);
}

// ---------------------------------------------------------------------------
// The smoke
// ---------------------------------------------------------------------------

/// **The cloud**: placed as the vent begins, grown from a point to
/// `SmokeRadius` over the vent's startup, and left standing its life. Fire
/// burns it off (the shared floor: it ignites into the flash).
fn smoke(w: &mut World, m: &Monster, now: u32) {
    if let Doing::Startup { kind: SMOKE, left } = m.doing {
        let a = SPECIES.attack(SMOKE);
        let at = V3::new(m.pos.x, Fx::ZERO, m.pos.z);
        if left == a.startup {
            // A fresh cloud replaces its last.
            let old = w.lore.word(word::SMOKE_SLOT);
            if w.lore.word(word::SMOKE_AT) != 0 {
                let h = hazard::get(&w.lore, old as usize);
                if h.present() && h.index() == CLOUD {
                    hazard::clear(&mut w.lore, old as usize);
                }
            }
            let first = Knob::SmokeRadius
                .fx()
                .div(Fx::from_int(a.startup.max(1) as i32));
            if let Some(slot) = hazard::place(&mut w.lore, Hazard::disc(CLOUD, at, first)) {
                w.lore.set_word(word::SMOKE_SLOT, slot as u32);
                w.lore.set_word(word::SMOKE_AT, now.wrapping_add(1));
            }
        }
    }
    // Growing.
    let began = w.lore.word(word::SMOKE_AT);
    if began == 0 {
        return;
    }
    let slot = w.lore.word(word::SMOKE_SLOT) as usize;
    let mut h = hazard::get(&w.lore, slot);
    if !h.present() || h.index() != CLOUD {
        // Burnt off, or gone.
        if h.present() && h.index() == FLASH {
            count_byte(&mut w.lore, word::MORE, 2, 1);
        }
        w.lore.set_word(word::SMOKE_AT, 0);
        return;
    }
    let age = now.wrapping_sub(began.wrapping_sub(1)) as i32;
    let grow = SPECIES.attack(SMOKE).startup.max(1) as i32;
    let r = Knob::SmokeRadius
        .fx()
        .mul(Fx::from_int(age.clamp(1, grow)))
        .div(Fx::from_int(grow));
    h.radius = hazard::radius_byte(r);
    hazard::set(&mut w.lore, slot, h);
}

/// Its own cloud, if one is standing: where, how wide, and how old (frames).
pub fn cloud(w: &World) -> Option<(V3, Fx, u32)> {
    let began = w.lore.word(word::SMOKE_AT);
    if began == 0 {
        return None;
    }
    let h = hazard::get(&w.lore, w.lore.word(word::SMOKE_SLOT) as usize);
    (h.present() && h.index() == CLOUD).then(|| {
        (
            h.centre(),
            h.radius(),
            w.frame.wrapping_sub(began.wrapping_sub(1)),
        )
    })
}

// ---------------------------------------------------------------------------
// The mimic
// ---------------------------------------------------------------------------

/// **The mimic's ghost**: placed as the mimic commits, at the candidate the
/// frame hook last worked out -- `MimicFrom` to `MimicTo` from the target,
/// in its glanced view -- facing the target, playing the lunge's rear or the
/// spear's. While it plays, the real animal stands still where it is; when it
/// ends, nothing strikes for `MimicQuiet` frames.
fn mimic(w: &mut World, m: &mut Monster, now: u32) {
    let _ = now;
    let quiet = w.lore.word(word::QUIET);
    if quiet > 0 {
        w.lore.set_word(word::QUIET, quiet - 1);
    }
    match m.doing {
        Doing::Startup { kind: MIMIC, left } => {
            m.speed = Fx::ZERO;
            m.rooted = m.rooted.max(2);
            if left == SPECIES.attack(MIMIC).startup {
                let at = point(w.lore.word(word::GHOST_AT));
                let to = flat(m.brain.seen.sub(at));
                let yaw = math::atan2_turns(to.z, to.x);
                let plays = if draw(m) % 2 == 0 { LUNGE } else { SPEAR };
                w.lore.set_word(word::GHOST, word_of(at));
                w.lore.set_word(
                    word::GHOST_LOOK,
                    (yaw.raw() & 0xFFFF) as u32 | (plays as u32 + 1) << 16,
                );
                count_byte(&mut w.lore, word::MORE, 1, 1);
            }
        }
        Doing::Active { kind: MIMIC, .. } | Doing::Recovery { kind: MIMIC, .. } => {
            m.speed = Fx::ZERO;
        }
        _ => {
            // Over: nothing strikes for the quiet, from now.
            if w.lore.word(word::GHOST_LOOK) != 0 {
                w.lore.set_word(word::GHOST_LOOK, 0);
                w.lore
                    .set_word(word::QUIET, Knob::MimicQuiet.raw().max(0) as u32);
            }
        }
    }
}

/// Is a strike held off by a mimic just played?
pub fn quiet(lore: &Lore) -> bool {
    lore.word(word::QUIET) > 0 || lore.word(word::GHOST_LOOK) != 0
}

/// **What a decloak looks like, wherever one is drawn**: where it stands,
/// which way it faces, which rear it is (the move whose silhouette it shows),
/// and how many frames in it is. A real one is the body; a mimic is its
/// ghost, and the two are drawn alike -- the floor under them is the only
/// difference (§2). The scripted hunter reads this, and the prints, and not
/// which one it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Apparition {
    pub at: V3,
    pub yaw: Fx,
    pub rear: u8,
    pub frame: i32,
}

/// Every decloak being drawn this frame: the body's, if it is in a strike's
/// startup, and a ghost's, if a mimic is playing.
pub fn apparitions(w: &World) -> [Option<Apparition>; 2] {
    let mut out = [None; 2];
    let Some(m) = w
        .monsters
        .iter()
        .flatten()
        .find(|m| m.species == SpeciesId::VEILSTALKER)
    else {
        return out;
    };
    if let Doing::Startup { kind, left } = m.doing {
        let e = SPECIES.attack(kind).startup as i32 - left as i32;
        if strikes(kind) && kind != MIMIC {
            out[0] = Some(Apparition {
                at: m.pos,
                yaw: m.yaw,
                rear: kind,
                frame: e,
            });
        }
        if kind == MIMIC {
            if let Some(g) = ghost(w) {
                out[1] = Some(Apparition {
                    at: g.pos,
                    yaw: g.yaw,
                    rear: g.doing.attacking().unwrap_or(LUNGE),
                    frame: e,
                });
            }
        }
    }
    out
}

/// The ghost, and the strength it is drawn at: the decloak's own ramp, as a
/// real one's would be.
fn apparition(w: &World) -> Option<(Monster, Fx)> {
    let g = ghost(w)?;
    Some((g, veil(&g)))
}

/// **The ghost as a body to draw**: the creature, moved to where the mimic
/// plays and posed in the rear it pretends. Never in the world -- nothing
/// hits it and nothing stands on it -- only drawn, at the decloak's
/// strength, with the floor marker its rear would have.
pub fn ghost(w: &World) -> Option<Monster> {
    let look = w.lore.word(word::GHOST_LOOK);
    if look == 0 {
        return None;
    }
    let m = w
        .monsters
        .iter()
        .flatten()
        .find(|m| m.species == SpeciesId::VEILSTALKER)?;
    let Doing::Startup { kind: MIMIC, left } = m.doing else {
        return None;
    };
    let plays = ((look >> 16) as u8).saturating_sub(1);
    let e = SPECIES.attack(MIMIC).startup as i32 - left as i32;
    let total = SPECIES.attack(plays).startup as i32;
    let mut g = *m;
    g.pos = point(w.lore.word(word::GHOST));
    g.pos.y = w.arena().ground_under(g.pos);
    g.yaw = Fx::from_raw((look & 0xFFFF) as i32);
    g.yaw_rate = Fx::ZERO;
    g.speed = Fx::ZERO;
    g.doing = Doing::Startup {
        kind: plays,
        left: (total - e).clamp(0, u16::MAX as i32) as u16,
    };
    Some(g)
}

// ---------------------------------------------------------------------------
// Fire
// ---------------------------------------------------------------------------

/// **Twenty frames in fire and it panics** (§4): any part of it, any fire.
/// Fully visible, down in the snow for `PanicFrames`, throwing nothing; then
/// up, visible a moment more, and away. A panic locks the next one out for
/// `PanicLockout`, halved when it is desperate.
fn panic(w: &mut World, m: &mut Monster, now: u32) {
    let _ = now;
    let fire = w.lore.word(word::FIRE);
    let (mut burning, mut lock) = (fire & 0xFFFF, fire >> 16);
    lock = lock.saturating_sub(1);
    let mut cause = None;
    if !airborne(m) && !perched(m) {
        for part in 0..PART_COUNT {
            let at = part_rest_world(m, part);
            if at.y.raw() > Knob::FireHeight.fx().raw() {
                continue;
            }
            if let Some(from) = in_fire_at(w, at, Fx::ZERO) {
                cause = Some(from);
                break;
            }
        }
    }
    burning = if cause.is_some() { burning + 1 } else { 0 };
    let already = matches!(m.doing, Doing::Toppled { .. } | Doing::Dead);
    if let Some(from) = cause {
        if burning >= Knob::FireFrames.raw().max(1) as u32 && lock == 0 && !already {
            m.doing = Doing::Toppled {
                left: Knob::PanicFrames.raw().clamp(1, u16::MAX as i32) as u16,
            };
            m.speed = Fx::ZERO;
            m.yaw_rate = Fx::ZERO;
            m.strain = 0;
            set_flag(m, flag::PANICKED, true);
            let full = Knob::PanicLockout.raw().max(0) as u32;
            lock = if desperate(m) { full / 2 } else { full };
            burning = 0;
            match from {
                FireFrom::Pillar => count_byte(&mut w.lore, word::PANICS, 0, 1),
                FireFrom::Embers => count_byte(&mut w.lore, word::PANICS, 1, 1),
                FireFrom::Stone => count_byte(&mut w.lore, word::PANICS, 2, 1),
                FireFrom::Brazier => count_byte(&mut w.lore, word::PANICS, 3, 1),
                FireFrom::Other => count_byte(&mut w.lore, word::MORE, 0, 1),
            }
        }
    }
    w.lore
        .set_word(word::FIRE, burning.min(0xFFFF) | lock.min(0xFFFF) << 16);
}

/// Panics so far, by cause.
pub fn panics(lore: &Lore) -> [u32; 5] {
    let p = lore.word(word::PANICS);
    let more = lore.word(word::MORE);
    [
        p & 0xFF,
        p >> 8 & 0xFF,
        p >> 16 & 0xFF,
        p >> 24 & 0xFF,
        more & 0xFF,
    ]
}

/// Mimics played so far.
pub fn mimics(lore: &Lore) -> u32 {
    lore.word(word::MORE) >> 8 & 0xFF
}

/// Its clouds burnt off by fire.
pub fn smokes_burnt(lore: &Lore) -> u32 {
    lore.word(word::MORE) >> 16 & 0xFF
}

/// Add to one byte of a counter word.
fn count_byte(lore: &mut Lore, word: usize, byte: u32, by: u32) {
    let w = lore.word(word);
    let shift = byte * 8;
    let v = ((w >> shift) & 0xFF).saturating_add(by).min(0xFF);
    lore.set_word(word, (w & !(0xFF << shift)) | v << shift);
}

// ---------------------------------------------------------------------------
// The trail
// ---------------------------------------------------------------------------

/// **The trail**: a print at each forefoot as it comes down -- the hind foot
/// steps into it, as a cat's does, so two prints a stride -- on any floor that
/// takes a print (snow, ash), and four at once, every foot, on a real
/// decloak's first frame. A mimic stamps nothing, and the real animal stands
/// still through it. Prints past their life are cleared, so the ring's clock
/// never reads an old print as new.
fn trail(w: &mut World, m: &Monster, now: u32) {
    let ground = w.terrain();
    let stride_word = w.lore.word(word::STRIDE);
    let was = (stride_word & 0xFFFF) as u16;
    let mut next = (stride_word >> 16) as usize % PRINTS;
    let red = now < w.lore.word(word::RED_UNTIL);
    let mut stamp = |w: &mut World, foot: usize| {
        let rig = m.rig();
        let part = SPECIES.legs[foot].foot;
        let at = rig.part_to_world(part, V3::ZERO);
        let floor = ground.ground_under(at);
        let skin = crate::arena::SKIN;
        let on_floor = floor.raw() <= skin.raw() && m.pos.y.raw() <= floor.add(skin).raw();
        if !on_floor || !ground.floor_at(at.x, at.z).takes_prints() {
            return;
        }
        w.lore
            .set_word(word::PRINTS + next, print_word(at, foot as u8, red, now));
        next = (next + 1) % PRINTS;
    };
    // The decloak's first frame: every foot sets.
    if let Doing::Startup { kind, left } = m.doing {
        if strikes(kind) && kind != MIMIC && kind != POUNCE && left == SPECIES.attack(kind).startup
        {
            for foot in 0..SPECIES.legs.len().min(4) {
                stamp(w, foot);
            }
        }
    }
    // The stride: a forefoot down at each half cycle.
    let now_stride = m.stride;
    let walking = m.alive() && !airborne(m) && !perched(m) && m.speed.abs().raw() > 0;
    if walking {
        let half = 1u32 << 15;
        let a = was as u32 / half;
        let b = now_stride as u32 / half;
        if a != b {
            stamp(w, (b % 2) as usize);
        }
    }
    w.lore
        .set_word(word::STRIDE, now_stride as u32 | (next as u32) << 16);
    // Clear the dead.
    for i in 0..PRINTS {
        if let Some(p) = print_of(w.lore.word(word::PRINTS + i), now) {
            if p.age >= print_life(w, &p) {
                w.lore.set_word(word::PRINTS + i, 0);
            }
        }
    }
    for i in 0..PAINTS {
        if let Some(p) = paint_of(w.lore.word(word::PAINT + i), now) {
            if p.age as i32 >= Knob::PaintFrames.raw() {
                w.lore.set_word(word::PAINT + i, 0);
            }
        }
    }
    // **Through blood**: a foot in one of the Blood mage's pools, and the
    // prints come out red for a while after.
    if m.alive() {
        let rig = m.rig();
        for leg in SPECIES.legs.iter() {
            let at = rig.part_to_world(leg.foot, V3::ZERO);
            let wet = w.effects.iter().flatten().any(|e| {
                e.is_a_pool() && math::wide_flat_dist(e.pos, at).raw() <= e.field_radius().raw()
            });
            if wet {
                w.lore.set_word(
                    word::RED_UNTIL,
                    now.wrapping_add(Knob::RedPrints.raw().max(0) as u32),
                );
                break;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The braziers
// ---------------------------------------------------------------------------

/// The arena's braziers: its sites called "brazier", in order, at most four.
pub fn braziers_of(w: &World) -> impl Iterator<Item = (usize, V3)> + '_ {
    w.arena()
        .sites
        .iter()
        .filter(|s| s.name == "brazier")
        .take(4)
        .enumerate()
        .map(|(i, s)| {
            let (x, z) = s.route.first().copied().unwrap_or((0, 0));
            (i, V3::new(Fx::ratio(x, 100), Fx::ZERO, Fx::ratio(z, 100)))
        })
}

/// Is brazier `i` still standing?
pub fn brazier_standing(lore: &Lore, i: usize) -> bool {
    lore.word(word::BRAZIERS) & 1 << i == 0
}

/// **Any blow tips a standing brazier**, once: a fighter's swing reaching it,
/// or the creature's own body going through it -- which is both a trace and
/// a waste of a fire somebody was saving (§4). Its coals spill on the floor
/// beside it, away from what tipped it: a burning patch for the coals' life.
fn braziers(w: &mut World) {
    let mut found = [(V3::ZERO, false); 4];
    for (i, at) in braziers_of(w) {
        found[i] = (at, brazier_standing(&w.lore, i));
    }
    let size = Knob::BrazierSize.fx();
    let reach = math::half(size);
    for (i, (at, standing)) in found.into_iter().enumerate() {
        if !standing {
            continue;
        }
        let top = at.add(V3::new(Fx::ZERO, size, Fx::ZERO));
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
                if !m.alive() {
                    continue;
                }
                let body = SPECIES.fight_fx(crate::species::FightField::BodyRadius);
                let near = math::wide_flat_dist(m.pos, at).raw() <= body.add(reach).raw();
                if m.reaches(at, size, reach) || near {
                    away = Some(at.sub(m.pos));
                }
            }
        }
        if let Some(dir) = away {
            let dir = flat(dir);
            let dir = if dir.flat_len().raw() > 0 {
                math::wide_normalized(dir)
            } else {
                V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO)
            };
            let r = Knob::CoalsRadius.fx();
            let spill = at.add(dir.scale(r.add(reach)));
            hazard::place(&mut w.lore, Hazard::disc(COALS, spill, r));
            let tipped = w.lore.word(word::BRAZIERS) | 1 << i;
            w.lore.set_word(word::BRAZIERS, tipped);
        }
    }
}

// ---------------------------------------------------------------------------
// The veil's clock, the stalk, and what the brain may read
// ---------------------------------------------------------------------------

/// **The re-cloak's fade**, the end of an engagement, and the stalk: in full
/// view the fade is full; out of it, it runs down over `Recloak` frames, and
/// when it is gone the engagement is over -- its hits forgotten, and a stalk
/// begins before the next.
fn veil_clock(w: &mut World, m: &mut Monster) {
    let full = Knob::Recloak.raw().max(1) as u32;
    let was = w.lore.word(word::FADE);
    let now = if veil(m).raw() >= Fx::ONE.raw() {
        full
    } else {
        was.saturating_sub(1)
    };
    w.lore.set_word(word::FADE, now);
    let stalk = w.lore.word(word::STALK);
    if was > 0 && now == 0 && m.alive() {
        // Cloaked again: a new engagement, after a stalk.
        set_hits(m, 0);
        w.lore.set_word(word::STRIKES, 0);
        let frames = stalk_frames(m);
        w.lore.set_word(word::STALK, frames);
        set_flag(m, flag::STALKING, true);
        // How far out it means to strike from next: drawn, so every range
        // gets its turn.
        let ranges = [RAKE, SPEAR, LUNGE, QUILLS];
        let kind = ranges[(draw(m) % ranges.len() as u32) as usize];
        let pick = SPECIES.attack(kind).ideal_range;
        w.lore.set_word(word::PLAN, lore::to_cm(pick) as u16 as u32);
    } else if stalk > 0 && m.doing.free() {
        // **Cornered**: seen, the target close, and a wall at its back --
        // the stalk is over, and it fights.
        let b = w.arena().bounds;
        let room = Knob::WallLook.fx().add(SPECIES.margin());
        let walled = m.pos.x.raw() < b.lo_x.add(room).raw()
            || m.pos.x.raw() > b.hi_x.sub(room).raw()
            || m.pos.z.raw() < b.lo_z.add(room).raw()
            || m.pos.z.raw() > b.hi_z.sub(room).raw();
        let near = math::wide_flat_dist(m.pos, m.brain.seen).raw() < Knob::ExposedNear.fx().raw();
        let left = if exposed(m) && walled && near {
            1
        } else {
            stalk
        };
        w.lore.set_word(word::STALK, left - 1);
        if left == 1 {
            set_flag(m, flag::STALKING, false);
        }
    }
    // Anything that is not a prowl ends a stalk's slink.
    if !m.doing.free() && !matches!(m.doing.attacking(), Some(MIMIC | SMOKE | CLIMB)) {
        set_flag(m, flag::STALKING, false);
    } else if w.lore.word(word::STALK) > 0 && m.doing.free() {
        set_flag(m, flag::STALKING, true);
    }
}

/// Strikes begun since it last cloaked.
pub fn strikes_this_engagement(lore: &Lore) -> u32 {
    lore.word(word::STRIKES)
}

/// Frames of stalk left.
pub fn stalk_left(lore: &Lore) -> u32 {
    lore.word(word::STALK)
}

/// The range it means to strike from this engagement.
pub fn plan_range(lore: &Lore) -> Fx {
    let cm = lore.word(word::PLAN) as u16 as i16;
    if cm <= 0 {
        SPECIES.attack(LUNGE).ideal_range
    } else {
        lore::from_cm(cm)
    }
}

/// **Where it would decloak**, seen from each hunter's glanced look, and
/// what stands there -- worked out here, where the whole world is, and
/// written for the brain to read next frame (`word::VIEW`). One frame stale,
/// against a look a glance stale.
fn sense(w: &mut World, m: &Monster, slot: usize) {
    let _ = slot;
    let ground = w.terrain();
    let field = crate::stones::gather(&w.players);
    let fighters = w.players;
    let effects = w.effects;
    let critters = w.critters;
    let herd = w.monsters;
    let scene = Scene {
        stones: &field,
        players: &fighters,
        effects: &effects,
        quarry: &herd,
        critters: &critters,
        arena: &ground,
    };
    let cone = Knob::ViewCone.fx();
    let point_of = middle_of(m);
    // **And where it will stand when it has stopped.** A strike thrown out
    // of a bound is decloaked while it skids, and a decloak that slides
    // behind a trunk halfway through is one nobody saw: the gate asks of the
    // whole of it, from here to where the braking leaves it -- speed squared
    // over twice the braking rate, along its heading.
    let stopping = m
        .speed
        .mul(m.speed)
        .div(m.sp().brake().add(m.sp().brake()).max(Fx::ONE));
    let stops_at = point_of.add(V3::from_turns(m.yaw).scale(stopping));
    let fire = in_fire_at(w, m.pos, Knob::FireShy.fx().min(Fx::ONE)).is_some();
    let cloud_now = cloud(w);
    let inside =
        |p: V3| cloud_now.is_some_and(|(c, r, _)| math::wide_flat_dist(c, p).raw() <= r.raw());
    let young = cloud_now.is_some_and(|(_, _, age)| age < Knob::SmokeQuiet.raw().max(0) as u32);
    let mut ghost_at = V3::ZERO;
    for i in 0..MAX_PLAYERS.min(2) {
        let mut bits = 0u32;
        if fighters[i].health <= 0 {
            w.lore.set_word(word::VIEW + i, 0);
            continue;
        }
        let (at, yaw) = led_look(&w.lore, i);
        let aloft = Fx::ZERO;
        if plainly_in_view(at, yaw, point_of, cone, &scene)
            && plainly_in_view(at, yaw, stops_at, cone, &scene)
        {
            bits |= view::IN_VIEW;
        } else if aim::off_look(at, aloft, yaw, point_of, &scene).raw() <= cone.raw() {
            bits |= view::HIDDEN;
        }
        if fire {
            bits |= view::IN_FIRE;
        }
        if inside(m.pos) {
            bits |= view::IN_SMOKE;
        }
        if !ground.floor_at(m.pos.x, m.pos.z).takes_prints()
            || ground.ground_under(m.pos).raw() > crate::arena::SKIN.raw()
        {
            bits |= view::BARE;
        }
        if inside(fighters[i].pos) {
            bits |= view::THEY_IN_SMOKE;
            if young {
                bits |= view::THEY_IN_YOUNG_SMOKE;
            }
        }
        // Somebody else looking at where it would decloak.
        let other = 1 - i;
        if fighters[other].health > 0 {
            let (oat, oyaw) = led_look(&w.lore, other);
            if aim::in_view_from(oat, Fx::ZERO, oyaw, point_of, cone, &scene) {
                bits |= view::WATCHED;
            }
        }
        // Where a mimic would play, for the target it has: in view, out in
        // front at the mimic's range, off to the side it is on.
        if i == m.brain.target as usize {
            if let Some(g) = ghost_place(m, at, yaw, &scene) {
                bits |= view::GHOST_OK;
                ghost_at = g;
            }
        }
        let off = aim::off_look(at, aloft, yaw, point_of, &scene);
        let share = off.div(cone.max(SMALLEST)).min(Fx::ONE);
        bits |= ((share.raw().clamp(0, 0xFFFF)) as u32) << 16;
        w.lore.set_word(word::VIEW + i, bits);
    }
    w.lore.set_word(word::GHOST_AT, word_of(ghost_at));
    let shy = Knob::FireShy.fx().add(SPECIES.gait_stride());
    match nearest_fire(w, m.pos, shy) {
        Some(c) => {
            w.lore.set_word(word::FIRE_AT, word_of(c));
            w.lore.set_word(word::FIRE_NEAR, 1);
        }
        None => w.lore.set_word(word::FIRE_NEAR, 0),
    }
    // Decloaks by where they were on the screen, counted as each begins.
    if let Doing::Startup { kind, left } = m.doing {
        if strikes(kind) && left == SPECIES.attack(kind).startup {
            // A mimic is no strike of the engagement's: nothing about it
            // shows the animal, so nothing would end the engagement it
            // counted toward.
            if kind != MIMIC {
                let n = w.lore.word(word::STRIKES);
                w.lore.set_word(word::STRIKES, n + 1);
            }
            let target = (m.brain.target as usize).min(1);
            let share = w.lore.word(word::VIEW + target) >> 16;
            let third = (share * 3 / 0x10000).min(2);
            count_byte(&mut w.lore, word::THIRDS, third, 1);
        }
    }
}

/// **In view, and not by a hair**: the point inside the glanced view from
/// where the hunter stood, and from `ViewMargin` either side of it -- a
/// hunter who has stepped a pace since the glance still sees it, and a
/// decloak just behind a trunk's edge is not one the creature trusts. The
/// glance is stale by design; the margin is what keeps a stale glance from
/// becoming a blind hit.
fn plainly_in_view(at: V3, yaw: Fx, point: V3, cone: Fx, scene: &Scene) -> bool {
    let to = flat(point.sub(at));
    let across = if to.flat_len().raw() > 0 {
        let d = math::wide_normalized(to);
        V3::new(d.z.neg(), Fx::ZERO, d.x)
    } else {
        V3::ZERO
    };
    let side = across.scale(Knob::ViewMargin.fx());
    // **And through a little turn of the look**: the eye sits behind the
    // shoulder, so a camera that sweeps a few degrees swings it sideways, and
    // a decloak beside a trunk that one glance saw clear is behind it at the
    // next. A person's look is never still; the gate asks of the whole of
    // its wander, not of one sample of it.
    let sweep = Knob::ViewSweep.fx();
    [yaw, yaw.add(sweep), yaw.sub(sweep)].iter().all(|yaw| {
        [at, at.add(side), at.sub(side)]
            .iter()
            .all(|from| aim::in_view_from(*from, Fx::ZERO, *yaw, point, cone, scene))
    })
}

/// Where it would play a mimic for a hunter who stood at `at` looking along
/// `yaw`: at `MimicFrom`..`MimicTo` out, two thirds of the way to the edge
/// of the view on the side the real animal is, if that is in view and clear.
fn ghost_place(m: &Monster, at: V3, yaw: Fx, scene: &Scene) -> Option<V3> {
    let cone = Knob::ViewCone.fx();
    let to_me = flat(m.pos.sub(at));
    let right = V3::from_turns(yaw.add(math::QUARTER_TURN));
    let side = if right.dot(to_me).raw() >= 0 {
        Fx::ONE
    } else {
        Fx::ONE.neg()
    };
    let off = cone.mul(Knob::MimicEdge.fx()).mul(side);
    let range = math::half(Knob::MimicFrom.fx().add(Knob::MimicTo.fx()));
    let spot = at.add(V3::from_turns(yaw.add(off)).scale(range));
    let spot = V3::new(spot.x, Fx::ZERO, spot.z);
    let mid = spot.add(V3::new(Fx::ZERO, Knob::MiddleHeight.fx(), Fx::ZERO));
    let inside = scene.arena.inside(spot) && scene.arena.ground_under(spot).raw() <= Fx::ZERO.raw();
    (inside && aim::in_view_from(at, Fx::ZERO, yaw, mid, cone, scene)).then_some(spot)
}

/// The middle of its body, where a decloak is seen.
pub fn middle_of(m: &Monster) -> V3 {
    m.pos
        .add(V3::new(Fx::ZERO, Knob::MiddleHeight.fx(), Fx::ZERO))
}

/// What the frame hook saw for hunter `i`: [`view`] bits.
pub fn view_of(lore: &Lore, i: usize) -> u32 {
    lore.word(word::VIEW + i.min(1)) & 0xFFFF
}

/// How far toward the edge of hunter `i`'s glanced view it stands, nought
/// (dead ahead) to one (at the cone's edge or beyond).
pub fn edge_of(lore: &Lore, i: usize) -> Fx {
    Fx::from_raw((lore.word(word::VIEW + i.min(1)) >> 16) as i32)
}

/// Its decloaks by thirds of the screen: centre, middle, edge.
pub fn thirds(lore: &Lore) -> [u32; 3] {
    let t = lore.word(word::THIRDS);
    [t & 0xFF, t >> 8 & 0xFF, t >> 16 & 0xFF]
}

/// The frame the brain is thinking on: one after the last the hook wrote.
pub fn now(lore: &Lore) -> u32 {
    lore.word(word::NOW).wrapping_add(1)
}

/// A dodge-speed sample seen, how many frames ago.
pub fn bait_seen(lore: &Lore, now: u32) -> Option<u32> {
    let at = lore.word(word::BAIT);
    (at != 0).then(|| now.wrapping_sub(at.wrapping_sub(1)))
}

// ---------------------------------------------------------------------------
// What it draws
// ---------------------------------------------------------------------------

/// **What it draws besides its telegraph**: the braziers, standing and lit or
/// tipped and cold.
fn marks(w: &World, out: &mut Marks) {
    let tall = Knob::BrazierSize.fx();
    for (i, at) in braziers_of(w) {
        let standing = brazier_standing(&w.lore, i);
        out.push(Mark {
            at,
            radius: math::half(math::half(tall)),
            height: if standing {
                tall
            } else {
                math::half(math::half(tall))
            },
            look: MarkLook::Iron,
            progress: Fx::ONE,
        });
        if standing {
            out.push(Mark {
                at: at.add(V3::new(Fx::ZERO, tall, Fx::ZERO)),
                radius: math::half(math::half(tall)),
                height: math::half(tall),
                look: MarkLook::Flame,
                progress: Fx::ONE,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Small things
// ---------------------------------------------------------------------------

pub fn flat(v: V3) -> V3 {
    V3::new(v.x, Fx::ZERO, v.z)
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

/// The glance's own bookkeeping that the shared glance cannot do: a sample
/// at a dodge's speed is remembered, for the mimic's bait. Called by the
/// brain's appetite through the lore -- see [`super::mind`] -- so it is
/// written here, where the world is.
pub fn note_bait(w: &mut World, m: &Monster) {
    let v = flat(m.brain.seen_vel);
    if math::wide_flat_len(v).raw() > Knob::BaitSpeed.fx().raw() {
        w.lore.set_word(word::BAIT, w.frame.wrapping_add(1));
    }
}
