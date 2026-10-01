//! A pack brain: one mind, many small bodies.
//!
//! [`crate::critter`] is what a small body *is*. This is what a group of them
//! *does*, as one object -- the critters are its hands (bestiary P3, and the
//! Gnawers' §5, which is the design this was built to):
//!
//! - **One glance for the pack.** Every `GlanceFrames` it samples each fighter's
//!   position, velocity and *facing* (facing is visible, so it is something it
//!   saw), and every critter acts on that sample projected by `Lead`. A
//!   direction change between glances fools the whole ring at once.
//! - **Slots, not chasing.** At each glance the members without a move out are
//!   given places on a ring of `RingRadius` round their target's lead point,
//!   scored toward the target's back by `RearBias`. A slot with a solid
//!   between it and the target scores nothing -- which is what makes a wall at
//!   your back work. At `LineBelow` members or fewer the ring is an arc in
//!   front instead: three cannot surround anybody.
//! - **Tokens.** Only `Tokens` may hold a move that needs one at once; a token
//!   comes back when its move ends and rests `TokenRest` frames before it can be
//!   handed again. A rally ([`Pack::rally`]) lends more for a while. A monster
//!   that owns the pack holds `OwnerTokens` of them while its own move is out,
//!   so the Broodmother's slam and two bites are three of the same budget.
//! - **A leader, optionally.** It hangs back `LeaderHangback` behind the ring.
//!   If `LeaderRouts` is on, its death breaks the pack for good.
//! - **Morale, optionally.** A death scatters the others for `ScatterFrames`
//!   (`ScatterDistance` away). At `RoutShare` per cent dead the pack runs for
//!   home and regroups after `RegroupFrames` if nobody stands within
//!   `RegroupClear` of it; the count starts again against who came back. A
//!   share of zero never routs: a brood is a clock, not a crowd.
//! - **Spawning into a freed slot**, from a point, without allocating
//!   ([`spawn`]): an empty slot, or a corpse that has finished fading.
//!
//! **Thinking is staggered.** Critter `i` decides on frames where `frame %
//! ThinkEvery == i % ThinkEvery`, so a frame never holds every decision.
//!
//! What differs from one species to the next is supplied by the species' own
//! module through [`PackMind`]: how much it wants each move, where it wants to
//! be, and whatever else it keeps in [`Pack::memo`]. Every method has a default
//! that is the generic pack above, so a species overrides only what is its own.
//! The recipe is `docs/design/critters.md`.
//!
//! # The cost, bounded
//!
//! Everything here is bounded by constants, not by the fight: ten critters
//! (45 separation pairs), two fighters, at most [`MAX_RING`] ring places per
//! fighter scored once a glance against the arena's solids, and no per-frame
//! ray at all. No allocation, no floats, deterministic order throughout.

use crate::DT;
use crate::arena::Terrain;
use crate::critter::{
    Critter, CritterField, CritterMove, Critters, MAX_CRITTERS, NO_SLOT, flag, is, stat, stat_fx,
};
use crate::fixed::{Fx, cos_turns};
use crate::math::V3;
use crate::monster::{Attack, Doing, Herd, NO_PART, mount_of, mount_slot};
use crate::species::{Species, SpeciesId};
use crate::state::{MAX_PLAYERS, NOBODY, Player};
use crate::tuning as t;

/// Raw helper for writing fixed-point bounds readably.
const fn fx(num: i32, den: i32) -> i32 {
    Fx::ratio(num, den).raw()
}

crate::species_knobs! {
    /// The numbers every pack has, one set per species that brings one, in its
    /// store after its moves. Shown as "<Species> · pack" and "· morale".
    pub enum PackKnob;
    GlanceFrames,    "pack",   "Frames between glances",             Frames,  1,  90;
    Lead,            "pack",   "Lead on the target (frames)",        Frames,  0,  60;
    ThinkEvery,      "pack",   "A critter decides every (frames)",   Frames,  1,  16;
    Tokens,          "pack",   "Attack tokens",                      Int,     0,  6;
    TokenRest,       "pack",   "A returned token rests (frames)",    Frames,  0,  240;
    OwnerTokens,     "pack",   "Tokens the owner's moves hold",      Int,     0,  4;
    RingRadius,      "pack",   "Ring radius",                        Fixed,   0,  fx(20,1);
    RearBias,        "pack",   "Rear bias",                          Int,     0,  1000;
    LineBelow,       "pack",   "An arc at this many or fewer",       Int,     0,  10;
    Spacing,         "pack",   "Keep this far apart",                Fixed,   0,  fx(4,1);
    LeaderHangback,  "pack",   "Leader hangs back behind the ring",  Fixed,   0,  fx(20,1);
    Alarm,           "pack",   "Notices a fighter within (0: at once)", Fixed, 0, fx(40,1);
    Grace,           "pack",   "Holds off when a hunt begins",       Frames,  0,  600;
    Spawn,           "pack",   "Musters this far out",               Fixed,   0,  fx(30,1);
    Arrive,          "pack",   "Slows to a walk within",             Fixed,   0,  fx(8,1);
    ScatterDistance, "morale", "A death scatters them this far",     Fixed,   0,  fx(20,1);
    ScatterFrames,   "morale", "For this long (0: never)",           Frames,  0,  300;
    RoutShare,       "morale", "Routs with this share dead (0: never)", Percent, 0, 100;
    LeaderRouts,     "morale", "The leader's death breaks it",       Flag,    0,  1;
    RegroupFrames,   "morale", "A rout regroups after",              Frames,  0,  3600;
    RegroupClear,    "morale", "Only with nobody this near home",    Fixed,   0,  fx(30,1);
}

/// What a pack's mood can be, as the one byte the snapshot keeps.
pub mod mood {
    /// Not hostile yet: nobody has come near and nothing has been hit.
    pub const CALM: u8 = 0;
    pub const HUNTING: u8 = 1;
    /// A death: everybody flinches back for a moment.
    pub const SCATTERED: u8 = 2;
    /// Running for home, and coming back if left alone.
    pub const ROUTED: u8 = 3;
    /// Running for home, for good.
    pub const BROKEN: u8 = 4;
}

/// The most ring places scored per fighter: twice the members, at most this.
pub const MAX_RING: usize = 12;

/// The most tokens there can be at once, rally included.
pub const MAX_TOKENS: usize = 6;

/// Words of the pack's memory kept for its species: see [`Pack::memo`].
pub const MEMO: usize = 12;

/// No leader, no owner.
pub const NONE: u8 = NOBODY;

/// What the pack has seen of one fighter, at its last glance. A deliberately
/// small window, the monster's `Quarry` again: positions, velocities, facing --
/// all things you can see -- and nothing about buttons.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Seen {
    pub pos: V3,
    pub vel: V3,
    /// Its facing, as turns.
    pub facing: u16,
    pub alive: bool,
    /// Staggered, in hitstun, rooted: cannot act for the moment.
    pub down: bool,
    /// Slowed: what the Gnawers' pile-on waits for.
    pub slowed: bool,
    /// On the floor for a moment: staggered, knocked down or held -- not the
    /// flicker of hitstun every bite leaves, which `down` counts too. The
    /// Gnawers' pile-on waits for this or a slow.
    pub staggered: bool,
    /// The ring this fighter has: how many places it was cut into, and the
    /// bearing of place zero, in turns. Worked out at the glance.
    pub ring_places: u8,
    pub ring_base: u16,
    /// The ring is an arc in front rather than a circle round the back.
    pub arc: bool,
}

/// The pack brain, in the snapshot.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pack {
    /// Whose table: the species that brought it.
    pub species: SpeciesId,
    /// How clever it is this hunt: see [`crate::temper`]. Its glance, its lead
    /// and its critters' cadence are read through it.
    pub temper: u8,
    /// One of [`mood`].
    pub mood: u8,
    /// The creature slot of the monster that owns it, or [`NONE`].
    pub owner: u8,
    /// The critter that leads it, or [`NONE`].
    pub leader: u8,
    /// Tokens lent on top of the knob, and for how long: a howl.
    pub boost: u8,
    /// How many were in it when it last formed, and how many have died since:
    /// what the rout share is counted against.
    pub mustered: u8,
    pub lost: u8,
    pub mood_left: u16,
    pub boost_left: u16,
    pub glance_left: u16,
    pub grace: u16,
    /// Rest left on tokens that have come back, one per token.
    pub rest: [u16; MAX_TOKENS],
    pub rng: u32,
    /// Where it runs to: the den. Where it mustered, unless its species says.
    pub home: V3,
    pub seen: [Seen; MAX_PLAYERS],
    /// **The species' own words.** The pack brain never reads these; a
    /// species' [`PackMind`] keeps whatever it needs here -- the herd's state
    /// and lane, the bull's horns, a howl's lockout -- so a creature adds no
    /// field to the world. Twelve 32-bit words: the Hornback's bull, the
    /// largest ask in the bestiary, is about forty bytes.
    pub memo: [i32; MEMO],
}

impl Pack {
    /// A pack of this species, formed at `home`, with nobody in it yet.
    pub fn new(species: SpeciesId, home: V3) -> Pack {
        let sp = species.get();
        Pack {
            species,
            temper: 0,
            mood: if sp.pack_raw(PackKnob::Alarm) == 0 {
                mood::HUNTING
            } else {
                mood::CALM
            },
            owner: NONE,
            leader: NONE,
            boost: 0,
            mustered: 0,
            lost: 0,
            mood_left: 0,
            boost_left: 0,
            glance_left: 0,
            grace: sp.pack_raw(PackKnob::Grace).max(0) as u16,
            rest: [0; MAX_TOKENS],
            rng: 0x2545_F491,
            home,
            seen: [Seen::default(); MAX_PLAYERS],
            memo: [0; MEMO],
        }
    }

    pub fn sp(&self) -> &'static Species {
        self.species.get()
    }

    // The three numbers a temper overrides on a pack (`crate::temper`): read
    // here, never off the species, so a tempered pack is the same code.

    /// Frames between the pack's glances, after its temper.
    pub fn glance_frames(&self) -> u16 {
        let own = self
            .sp()
            .pack_raw(PackKnob::GlanceFrames)
            .clamp(1, u16::MAX as i32);
        crate::temper::glance(own as u16, self.temper)
    }

    /// How far ahead it leads a fighter, in frames, after its temper.
    pub fn lead_frames(&self) -> i32 {
        crate::temper::lead_frames(self.sp().pack_raw(PackKnob::Lead), self.temper)
    }

    /// How often a critter decides, after its temper: it follows the glance.
    pub fn think_every(&self) -> u32 {
        crate::temper::cadence(self.sp().pack_raw(PackKnob::ThinkEvery), self.temper) as u32
    }

    /// How many tokens there are right now, the rally included.
    pub fn token_cap(&self) -> usize {
        (self.sp().pack_raw(PackKnob::Tokens).max(0) as usize + self.boost as usize).min(MAX_TOKENS)
    }

    /// Lend `extra` tokens for `frames`: the Gnawers' howl.
    pub fn rally(&mut self, extra: u8, frames: u16) {
        self.boost = extra;
        self.boost_left = frames;
    }

    /// Advance the pack's own dice. Deterministic, in the snapshot.
    pub fn roll(&mut self) -> u32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        x
    }

    /// Still a threat: hunting, scattered, or routed and coming back.
    pub fn fighting(&self) -> bool {
        self.mood != mood::BROKEN
    }
}

/// How many tokens are out: held by critters, resting, and held by the owner.
pub fn tokens_out(pack: &Pack, critters: &Critters, herd: &Herd) -> usize {
    let held = critters.iter().filter(|c| c.has(flag::TOKEN)).count();
    let resting = pack.rest.iter().filter(|r| **r > 0).count();
    held + resting + owner_holds(pack, herd)
}

/// Tokens the owning monster is holding: `OwnerTokens` while its move is out.
fn owner_holds(pack: &Pack, herd: &Herd) -> usize {
    let Some(beast) = herd.get(pack.owner as usize).and_then(|m| m.as_ref()) else {
        return 0;
    };
    match beast.doing {
        Doing::Startup { .. } | Doing::Active { .. } | Doing::Recovery { .. } => {
            pack.sp().pack_raw(PackKnob::OwnerTokens).max(0) as usize
        }
        _ => 0,
    }
}

/// Give a token back: it rests before it can be handed again.
pub fn give_back(pack: &mut Pack, c: &mut Critter) {
    if !c.has(flag::TOKEN) {
        return;
    }
    c.set(flag::TOKEN, false);
    let rest = pack.sp().pack_raw(PackKnob::TokenRest).max(0) as u16;
    if rest == 0 {
        return;
    }
    if let Some(slot) = pack.rest.iter_mut().find(|r| **r == 0) {
        *slot = rest;
    }
}

// ---------------------------------------------------------------------------
// What a species supplies
// ---------------------------------------------------------------------------

/// A pack, as its species declares it.
pub struct PackDecl {
    /// Its kinds of critter, in index order.
    pub kinds: &'static [crate::critter::CritterKind],
    /// Who it starts with, in slot order: `(kind, how many)`.
    pub muster: &'static [(u8, u8)],
    /// Which kind leads -- the first of that kind mustered -- if any.
    pub leader: Option<u8>,
    /// What it does that the generic pack does not.
    pub mind: &'static (dyn PackMind + Sync),
}

impl std::fmt::Debug for PackDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PackDecl")
            .field("kinds", &self.kinds)
            .field("muster", &self.muster)
            .field("leader", &self.leader)
            .finish()
    }
}

/// Everything a mind may read, as things stand this frame.
pub struct Look<'a> {
    pub sp: &'static Species,
    pub pack: &'a Pack,
    pub critters: &'a Critters,
    pub herd: &'a Herd,
    pub arena: &'a Terrain,
    pub frame: u32,
}

impl Look<'_> {
    /// Where the pack thinks a fighter will be: the glance, projected by `Lead`.
    pub fn lead(&self, target: usize) -> V3 {
        lead_point(self.pack, &self.pack.seen[target.min(MAX_PLAYERS - 1)])
    }
}

/// Where a critter wants to go this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Steer {
    /// The point it is heading for.
    pub to: V3,
    /// How fast it goes about it, at most.
    pub speed: Fx,
    /// What it looks at, if not where it is going.
    pub face: Option<V3>,
}

/// What a species says about its pack. Every method has a default, and the
/// defaults are the generic pack; a species overrides what is its own.
pub trait PackMind {
    /// How much critter `i` wants to throw `m` now. Zero or less: not at all.
    /// The default is the monster's terms -- a tent on the move's range and
    /// its bearing, times its appetite -- with nothing at all for a move that
    /// needs a token while the pack is afraid.
    fn appetite(&self, look: &Look, i: usize, m: CritterMove, a: &Attack) -> i32 {
        default_appetite(look, i, m, a)
    }

    /// Where critter `i` goes this frame. `want` is the generic pack's answer.
    fn steer(&self, look: &Look, i: usize, want: Steer) -> Steer {
        let _ = (look, i);
        want
    }

    /// Once a frame, before anybody steers: the species' own pack-level rules
    /// -- spawning, a herd's states, a howl's timer -- kept in [`Pack::memo`].
    /// `herd` is every creature in the fight as it stands this frame, so a
    /// monster that owns the pack (`Pack::owner`) can be read: the
    /// Broodmother's sacs bursting are her pack's spawns ([`spawn`]).
    fn frame(&self, pack: &mut Pack, critters: &mut Critters, herd: &Herd, frame: u32) {
        let _ = (pack, critters, herd, frame);
    }

    /// **The body critter `c` stands in this frame**, given its kind's own
    /// box. The default is the box. A species whose animal changes shape with
    /// what it is doing -- the Gnawers' Big One rearing to howl -- says so
    /// here, and [`Critter::body`] is still the one description everything
    /// reads. Must be a pure function of the critter.
    fn body(&self, c: &Critter, plain: crate::critter::Body) -> crate::critter::Body {
        let _ = c;
        plain
    }

    /// **Critter `i`'s move just connected with fighter `who`** (`victim`),
    /// after the hit itself: `blocked` if it was taken on a guard, `parried`
    /// if it was parried. What a bite does besides its damage -- the
    /// Gnawers' hamstring slows and latches; three of a pile-on knock you
    /// down. Nothing, for a pack that says nothing.
    #[allow(clippy::too_many_arguments)]
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
        let _ = (pack, critters, i, who, victim, blocked, parried);
    }

    /// Critter `i` just took `dealt`.
    fn hurt(&self, pack: &mut Pack, critters: &mut Critters, i: usize, dealt: i32) {
        let _ = (pack, critters, i, dealt);
    }

    /// Critter `i` just died. After the pack's own bookkeeping.
    fn died(&self, pack: &mut Pack, critters: &mut Critters, i: usize) {
        let _ = (pack, critters, i);
    }

    /// **A critter walked into a solid**: an arena wall, a raised solid, a
    /// defended thing that is one. `push` is what the resolve did to get it
    /// out, in the floor plane. Before it goes round the face -- so a species
    /// that stuns it here (the Hornback's bull, charging a rock) stops it
    /// sliding off. Nothing, for a pack that says nothing.
    fn bumped(&self, c: &mut Critter, push: V3) {
        let _ = (c, push);
    }

    /// **Is critter `c`'s move thrown to the other side?** Its move's volume
    /// is authored with a sideways offset (`HitZ`), and a body that throws it
    /// at whichever flank its target is on -- the Hornback bull's shoulder,
    /// the hook toward the side its head is cocked -- says so here, and the
    /// offset is mirrored. Decided by the species when the move commits and
    /// kept on the body, because a swing that changed sides mid-windup would
    /// be a telegraph that lied. Must be a pure function of the critter.
    fn mirrored(&self, c: &Critter) -> bool {
        let _ = c;
        false
    }

    /// **A fighter's swing is about to land on critter `i`**: does it? The
    /// default is that it does. A body that guards -- the Hornback bull,
    /// braced -- answers [`Guarded::Bounces`] for a blow it turns, and the
    /// swing does nothing to it and recoils; or [`Guarded::Breaks`] for a
    /// guard breaker, which lands and is the species' to make something of.
    /// Asked once per body per swing, before the damage, with the swing's own
    /// volume -- the overlay's -- so a species can ask which part of it was
    /// struck (a head, a horn).
    fn guarded(&self, pack: &mut Pack, critters: &mut Critters, i: usize, blow: &Blow) -> Guarded {
        let _ = (pack, critters, i, blow);
        Guarded::Lands
    }

    /// **What critter `i`'s windup will do, drawn on the floor**, given what
    /// its move's own volume says ([`Critter::telegraph`]). The default is
    /// that. A move whose reach is decided by more than its row -- the
    /// Hornback's charge, cut short where a solid will stop it -- says so
    /// here, from what the species keeps in [`Pack::memo`]. Read by the
    /// renderer and the overlay through [`telegraph`].
    fn telegraph(
        &self,
        pack: &Pack,
        critters: &Critters,
        i: usize,
        plain: crate::monster::Telegraph,
    ) -> Option<crate::monster::Telegraph> {
        let _ = (pack, critters, i);
        Some(plain)
    }
}

/// A fighter's swing, as a critter that might guard against it sees it: see
/// [`PackMind::guarded`].
pub struct Blow<'a> {
    /// The fighter swinging.
    pub who: usize,
    /// Where they stand.
    pub from: V3,
    /// The swing's volume this frame: `state::hitbox`, the overlay's.
    pub hitbox: &'a crate::state::Hitbox,
    /// What it would deal.
    pub damage: i32,
    /// **A guard breaker**: the versus rule's own flag (`unblockable`), or a
    /// Bulwark Slam carrying weight.
    pub breaks: bool,
}

/// What a guard made of a blow: see [`PackMind::guarded`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Guarded {
    /// No guard, or not against this: it lands as it would have.
    Lands,
    /// Turned. Nothing reaches the body; the attacker recoils for `recoil`
    /// frames and is pushed `push` metres back along the line from it.
    Bounces { recoil: u16, push: Fx },
    /// A guard breaker through the guard: it lands, and the species makes of
    /// the break what it will (a stagger, a lockout).
    Breaks,
}

/// **What critter `i`'s windup or hit will do, as drawn**: its own
/// [`Critter::telegraph`], as its species' mind says it really is
/// ([`PackMind::telegraph`]). The renderer's markers and the overlay read
/// this.
pub fn telegraph(
    pack: Option<&Pack>,
    critters: &Critters,
    i: usize,
) -> Option<crate::monster::Telegraph> {
    let c = critters.get(i)?;
    let sp = critters.sp();
    let plain = c.telegraph(sp)?;
    match (pack, sp.pack) {
        (Some(pack), Some(decl)) => decl.mind.telegraph(pack, critters, i, plain),
        _ => Some(plain),
    }
}

/// The pack's own appetite for a move: see [`PackMind::appetite`].
pub fn default_appetite(look: &Look, i: usize, m: CritterMove, a: &Attack) -> i32 {
    let c = &look.critters[i];
    if m.token && look.pack.mood != mood::HUNTING {
        return 0;
    }
    let seen = &look.pack.seen[(c.target as usize).min(MAX_PLAYERS - 1)];
    if !seen.alive {
        return 0;
    }
    let to = look.lead(c.target as usize).sub(c.pos);
    let flat = V3::new(to.x, Fx::ZERO, to.z);
    let d = flat.flat_len();
    let span = a.range_span;
    let off = d.sub(a.ideal_range).abs();
    if span.raw() <= 0 || off.raw() > span.raw() {
        return 0;
    }
    let bearing = c.facing().dot(flat.normalized());
    if bearing.raw() < a.aim_cos.sub(a.aim_span).raw() {
        return 0;
    }
    Fx::from_int(a.weight.max(1))
        .mul(Fx::ONE.sub(off.div(span)))
        .to_int()
        .max(1)
}

/// The generic pack, with no opinions of its own.
pub struct Plain;

impl PackMind for Plain {}

// ---------------------------------------------------------------------------
// Forming, spawning, dying
// ---------------------------------------------------------------------------

/// Form a species' pack at `home`, its critters in a loose knot facing
/// `facing`, filling the critter slots from zero. The leader, if it has one, is
/// the first of its kind.
pub fn muster(species: SpeciesId, home: V3, facing: V3, critters: &mut Critters) -> Option<Pack> {
    let sp = species.get();
    let decl = sp.pack?;
    let mut pack = Pack::new(species, home);
    *critters = Critters {
        species,
        all: [Critter::EMPTY; MAX_CRITTERS],
    };
    let yaw = yaw_of(facing);
    let side = V3::new(facing.z.neg(), Fx::ZERO, facing.x);
    let apart = sp.pack_fx(PackKnob::Spacing).max(Fx::ONE);
    let mut n = 0usize;
    for (kind, count) in decl.muster {
        for _ in 0..*count {
            if n >= MAX_CRITTERS {
                break;
            }
            // Rows of three, a body's keep-out apart (a metre at the least),
            // the first row nearest the hunters.
            let row = Fx::from_int((n / 3) as i32).mul(apart);
            let col = Fx::from_int((n % 3) as i32 - 1).mul(apart);
            let at = home.sub(facing.scale(row)).add(side.scale(col));
            critters[n] = Critter::new(sp, *kind, at, yaw);
            if decl.leader == Some(*kind) && pack.leader == NONE {
                pack.leader = n as u8;
                critters[n].set(flag::LEADER, true);
            }
            n += 1;
        }
    }
    pack.mustered = n as u8;
    Some(pack)
}

/// **Put a new critter of `kind` into the first slot that is free** -- empty,
/// gone, or a corpse that has finished fading -- standing at `at`, facing
/// `yaw`. `None` if every slot is taken. Never allocates; the Broodmother's
/// sacs burst through this.
pub fn spawn(
    pack: &mut Pack,
    critters: &mut Critters,
    kind: u8,
    at: V3,
    yaw: u16,
) -> Option<usize> {
    let sp = pack.sp();
    let slot = critters.iter().position(|c| !c.present()).or_else(|| {
        critters
            .iter()
            .position(|c| c.state == is::DEAD && c.timer == 0)
    })?;
    if pack.leader as usize == slot {
        pack.leader = NONE;
    }
    critters[slot] = Critter::new(sp, kind, at, yaw);
    pack.mustered = pack.mustered.saturating_add(1);
    Some(slot)
}

/// **Stand critter `i` on a creature's part**: slot `slot` of the herd, part
/// `part`, at `local` in that part's own frame. From then on the perch is
/// authoritative and the animal carries it (`Critter::perch`). The
/// Siegeshell's parasites are spawned and then perched; one that walks off the
/// part's top falls, and lands on whatever is under it.
pub fn perch_on(c: &mut Critter, herd: &Herd, slot: usize, part: usize, local: V3) {
    let Some(beast) = herd.get(slot).and_then(|m| m.as_ref()) else {
        return;
    };
    c.mount = mount_of(slot, part);
    c.set_perch(local);
    c.pos = beast.world_of(part, c.perch_local());
    c.vel = V3::ZERO;
    c.set(flag::AIRBORNE, false);
}

/// Critter `i` has just been killed: the body goes down, its token comes back,
/// and the pack takes it in -- scatter, rout, a broken pack if it led.
pub fn killed(pack: &mut Pack, critters: &mut Critters, i: usize) {
    let sp = pack.sp();
    let c = &mut critters[i];
    give_back(pack, c);
    c.state = is::DEAD;
    c.health = 0;
    c.timer = stat(sp, c.kind, CritterField::Corpse).max(0) as u16;
    c.slot = NO_SLOT;
    let led = c.has(flag::LEADER);
    pack.lost = pack.lost.saturating_add(1);
    // Nothing is calm about a death.
    if pack.mood == mood::CALM {
        pack.mood = mood::HUNTING;
    }
    pack.grace = 0;
    let share = sp.pack_raw(PackKnob::RoutShare);
    if led && sp.pack_raw(PackKnob::LeaderRouts) != 0 {
        pack.mood = mood::BROKEN;
    } else if share > 0
        && pack.mood != mood::BROKEN
        && pack.lost as i32 * 100 >= share * pack.mustered.max(1) as i32
    {
        pack.mood = mood::ROUTED;
        pack.mood_left = sp.pack_raw(PackKnob::RegroupFrames).max(0) as u16;
    } else if pack.mood == mood::HUNTING || pack.mood == mood::SCATTERED {
        let frames = sp.pack_raw(PackKnob::ScatterFrames).max(0) as u16;
        if frames > 0 {
            pack.mood = mood::SCATTERED;
            pack.mood_left = frames;
        }
    }
    // Afraid, nobody finishes winding up -- but a move its kind declares
    // committed.
    if pack.mood != mood::HUNTING {
        for c in critters.iter_mut() {
            let kept = sp
                .kind(c.kind)
                .moves
                .iter()
                .any(|m| m.kind == c.act && m.committed);
            if c.state == is::STARTUP && !kept {
                c.state = is::PROWL;
                c.timer = 0;
                give_back(pack, c);
            }
        }
    }
    if let Some(decl) = sp.pack {
        decl.mind.died(pack, critters, i);
    }
}

/// Critter `i` took a blow worth `dealt`, with `shove` along `dir` and a
/// `launch` upward, as a fighter's move would have shoved a fighter. Light
/// things are knocked out of what they are doing.
pub fn struck(
    pack: &mut Pack,
    critters: &mut Critters,
    i: usize,
    dealt: i32,
    dir: V3,
    shove: Fx,
    launch: Fx,
) {
    let sp = pack.sp();
    if !critters[i].alive() {
        return;
    }
    pack.grace = 0;
    if pack.mood == mood::CALM {
        pack.mood = mood::HUNTING;
    }
    let c = &mut critters[i];
    c.health = (c.health as i32 - dealt).max(0) as i16;
    if c.health <= 0 {
        killed(pack, critters, i);
        return;
    }
    let takes = stat_fx(sp, c.kind, CritterField::Shove);
    c.vel = c.vel.add(dir.scale(shove.mul(takes)));
    if launch.raw() > 0 {
        c.vel.y = c.vel.y.add(launch.mul(takes));
        c.set(flag::AIRBORNE, true);
    }
    if dealt >= stat(sp, c.kind, CritterField::FlinchAt) {
        give_back(pack, c);
        c.state = is::FLINCH;
        c.timer = stat(sp, c.kind, CritterField::FlinchFrames).max(1) as u16;
        c.set(flag::HIT_USED, false);
    }
    if let Some(decl) = sp.pack {
        decl.mind.hurt(pack, critters, i, dealt);
    }
}

/// **Everything a shot can hit that is not a fighter**: the creatures, the
/// small bodies and the brain that drives them, lent together to the things
/// that fly about the arena -- a bolt, a shard, an air shot -- as one argument.
pub struct Prey<'a> {
    pub herd: &'a mut Herd,
    pub critters: &'a mut Critters,
    pub pack: &'a mut Option<Pack>,
}

/// **A blow lands on critter `i`** from anything that is not a fighter's own
/// swing -- a bolt, a beam, a shard, a field -- and what it dealt. The one
/// door every one of them goes through, so a critter is hurt the same way by
/// all of them: [`struck`], on the world's pack.
pub fn hurt(
    pack: &mut Option<Pack>,
    critters: &mut Critters,
    i: usize,
    damage: i32,
    dir: V3,
    shove: Fx,
    launch: Fx,
) -> i32 {
    let Some(brain) = pack.as_mut() else {
        return 0;
    };
    let Some(c) = critters.get(i) else { return 0 };
    if !c.alive() || damage <= 0 {
        return 0;
    }
    let dealt = damage.min(c.health as i32);
    struck(brain, critters, i, damage, dir, shove, launch);
    dealt
}

/// **Frames until the soonest hit the pack could land**, as things stand: the
/// pack's answer to `Monster::frames_until_free`, which the fight report
/// divides the fight by into its four windows (the Gnawers' §9: "the soonest
/// any token holder, or anybody in a pile-on, could land"). A pack is many
/// bodies, so its window is the soonest any of them could hit:
///
/// - a hit out and unspent: now;
/// - a windup: the frames left in it;
/// - a body holding a token and not yet winding up (closing in): its move's
///   whole windup;
/// - a token free: the cadence a body decides on, and the quickest windup a
///   token buys;
/// - every token out: the soonest one comes back -- a holder's recovery and
///   the rest after it, or a resting token's rest -- and then the same;
/// - the grace, or a scatter: what is left of it first. Routed or broken,
///   never.
///
/// Moves that need no token (the Gnawers' pile-on, the Big One's maul) wait
/// for something -- a slow, a fighter in reach -- that this cannot see
/// coming, so they count only once they are winding up.
pub fn frames_until_free(pack: &Pack, critters: &Critters, herd: &Herd) -> u16 {
    let sp = pack.sp();
    let hurts = |act: u8| sp.attack(act).damage > 0;
    let mut soonest = u32::MAX;
    for c in critters.iter().filter(|c| c.alive()) {
        let left = match c.state {
            is::ACTIVE if hurts(c.act) && !c.has(flag::HIT_USED) => 0,
            is::STARTUP if hurts(c.act) => c.timer as u32,
            is::PROWL if c.has(flag::TOKEN) && hurts(c.act) => sp.attack(c.act).startup as u32,
            _ => continue,
        };
        soonest = soonest.min(left);
    }
    if soonest < u32::MAX || !critters.iter().any(Critter::alive) {
        return soonest.min(u16::MAX as u32) as u16;
    }
    let before = match pack.mood {
        mood::ROUTED | mood::BROKEN | mood::CALM => return u16::MAX,
        mood::SCATTERED => pack.mood_left as u32,
        _ => 0,
    }
    .max(pack.grace as u32);
    // The quickest windup a token buys, among the bodies still standing.
    let quickest = critters
        .iter()
        .filter(|c| c.alive())
        .flat_map(|c| sp.kind(c.kind).moves.iter())
        .filter(|m| m.token && hurts(m.kind))
        .map(|m| sp.attack(m.kind).startup as u32)
        .min();
    let Some(quickest) = quickest else {
        return u16::MAX;
    };
    let then = pack.think_every() + quickest;
    let back = if tokens_out(pack, critters, herd) < pack.token_cap() {
        0
    } else {
        let rest = sp.pack_raw(PackKnob::TokenRest).max(0) as u32;
        let held = critters
            .iter()
            .filter(|c| c.alive() && c.has(flag::TOKEN))
            .map(|c| match c.state {
                is::RECOVERY | is::FLINCH => c.timer as u32 + rest,
                _ => u32::MAX,
            });
        let resting = pack
            .rest
            .iter()
            .copied()
            .filter(|r| *r > 0)
            .map(|r| r as u32);
        held.chain(resting).min().unwrap_or(0)
    };
    (before.max(back) + then).min(u16::MAX as u32) as u16
}

/// Is the pack beaten: nobody left in the fight?
pub fn beaten(critters: &Critters) -> bool {
    !critters.iter().any(Critter::alive)
}

// ---------------------------------------------------------------------------
// One frame
// ---------------------------------------------------------------------------

/// Everything outside the pack it is allowed to read.
pub struct World<'a> {
    pub players: &'a [Player; MAX_PLAYERS],
    pub herd: &'a Herd,
    pub scene: &'a crate::aim::Scene<'a>,
    pub frame: u32,
}

/// One frame of the pack: glance, mood, decisions, steering, physics.
pub fn step(pack: &mut Pack, critters: &mut Critters, w: &World) {
    let sp = pack.sp();
    let Some(decl) = sp.pack else { return };
    let arena = w.scene.arena;

    // The owner gone, the brood breaks for good.
    if pack.owner != NONE
        && !w
            .herd
            .get(pack.owner as usize)
            .and_then(|m| m.as_ref())
            .is_some_and(|m| m.alive())
    {
        pack.mood = mood::BROKEN;
    }

    // ---- clocks ----
    pack.grace = pack.grace.saturating_sub(1);
    for r in pack.rest.iter_mut() {
        *r = r.saturating_sub(1);
    }
    if pack.boost_left > 0 {
        pack.boost_left -= 1;
        if pack.boost_left == 0 {
            pack.boost = 0;
        }
    }
    for c in critters.iter_mut().filter(|c| c.present()) {
        c.clock = c.clock.wrapping_add(1);
    }

    // ---- the glance ----
    if pack.glance_left == 0 {
        glance(pack, critters, w);
        pack.glance_left = pack.glance_frames();
    }
    pack.glance_left -= 1;

    // ---- mood ----
    match pack.mood {
        mood::CALM => {
            let alarm = sp.pack_fx(PackKnob::Alarm);
            let near = critters.iter().filter(|c| c.alive()).any(|c| {
                pack.seen
                    .iter()
                    .any(|s| s.alive && s.pos.sub(c.pos).flat_len().raw() <= alarm.raw())
            });
            if near || alarm.raw() == 0 {
                pack.mood = mood::HUNTING;
            }
        }
        mood::SCATTERED => {
            pack.mood_left = pack.mood_left.saturating_sub(1);
            if pack.mood_left == 0 {
                pack.mood = mood::HUNTING;
            }
        }
        mood::ROUTED => {
            let clear = sp.pack_fx(PackKnob::RegroupClear);
            let watched = pack
                .seen
                .iter()
                .any(|s| s.alive && s.pos.sub(pack.home).flat_len().raw() < clear.raw());
            if !watched {
                pack.mood_left = pack.mood_left.saturating_sub(1);
            }
            if pack.mood_left == 0 {
                pack.mood = mood::HUNTING;
                pack.lost = 0;
                pack.mustered = critters.iter().filter(|c| c.alive()).count() as u8;
            }
        }
        _ => {}
    }

    decl.mind.frame(pack, critters, w.herd, w.frame);

    // ---- each body ----
    let every = pack.think_every();
    let mut wants = [None; MAX_CRITTERS];
    for i in 0..MAX_CRITTERS {
        if !critters[i].present() {
            continue;
        }
        tick(pack, critters, i, w);
        if critters[i].state == is::PROWL && w.frame % every == i as u32 % every {
            think(pack, critters, i, w);
        }
        let want = {
            let look = Look {
                sp,
                pack,
                critters,
                herd: w.herd,
                arena,
                frame: w.frame,
            };
            let plain = plain_steer(&look, i);
            decl.mind.steer(&look, i, plain)
        };
        drive(sp, &mut critters[i], want);
        wants[i] = Some(want);
    }

    // ---- bodies in the world ----
    let rigs: [Option<crate::beast::Rig>; crate::monster::MAX_MONSTERS] =
        std::array::from_fn(|slot| {
            let wanted = critters
                .iter()
                .any(|c| c.present() && (c.mounted() || c.has(flag::AIRBORNE)));
            match (&w.herd[slot], wanted) {
                (Some(beast), true) => Some(beast.rig()),
                _ => None,
            }
        });
    for (i, c) in critters.iter_mut().enumerate() {
        if c.present() {
            move_body(sp, c, i, wants[i], w, &rigs);
        }
    }
    keep_apart(sp, critters, w);

    // A broken pack's survivors leave by the den.
    if pack.mood == mood::BROKEN {
        for c in critters.iter_mut() {
            if c.alive() && c.pos.sub(pack.home).flat_len().raw() < Fx::ONE.raw() {
                give_back(pack, c);
                c.state = is::GONE;
            }
        }
    }
}

/// Where the pack thinks fighter `who` will be: the glance, projected by
/// `Lead`. What [`Look::lead`] answers, for a species' own rules that hold
/// the pack rather than a look at it.
pub fn lead_of(pack: &Pack, who: usize) -> V3 {
    lead_point(pack, &pack.seen[who.min(MAX_PLAYERS - 1)])
}

/// Where a fighter will be, by the pack's reckoning.
fn lead_point(pack: &Pack, seen: &Seen) -> V3 {
    let frames = pack.lead_frames();
    let ahead = seen.vel.scale(Fx::from_int(frames).mul(DT));
    V3::new(seen.pos.x.add(ahead.x), seen.pos.y, seen.pos.z.add(ahead.z))
}

/// A unit vector's bearing, as the sixteen bits a critter keeps.
pub fn yaw_of(dir: V3) -> u16 {
    if dir.flat_len().raw() == 0 {
        return 0;
    }
    (crate::math::atan2_turns(dir.z, dir.x).raw() & 0xFFFF) as u16
}

/// Sample every fighter, choose targets, and cut each fighter's ring.
fn glance(pack: &mut Pack, critters: &mut Critters, w: &World) {
    let sp = pack.sp();
    for (i, p) in w.players.iter().enumerate() {
        let s = &mut pack.seen[i];
        s.pos = p.pos;
        s.vel = p.vel;
        s.facing = yaw_of(p.facing);
        s.alive = p.health > 0;
        s.down = p.action.stunned();
        s.slowed = p.slowed > 0;
        s.staggered = matches!(
            p.action,
            crate::state::Action::Stagger { .. } | crate::state::Action::Held { .. }
        );
    }
    // Each critter is after the nearest fighter still standing.
    for c in critters.iter_mut().filter(|c| c.alive()) {
        let mut best: Option<(usize, Fx)> = None;
        for (i, s) in pack.seen.iter().enumerate() {
            if !s.alive {
                continue;
            }
            let d = crate::math::big_len(s.pos.sub(c.pos));
            if best.is_none_or(|(_, b)| d.raw() < b.raw()) {
                best = Some((i, d));
            }
        }
        c.target = best.map_or(0, |(i, _)| i as u8);
    }
    // The rings.
    let radius = sp.pack_fx(PackKnob::RingRadius);
    let line_below = sp.pack_raw(PackKnob::LineBelow).max(0) as usize;
    let bias = sp.pack_raw(PackKnob::RearBias);
    for target in 0..MAX_PLAYERS {
        let members = |c: &Critter| {
            c.alive() && c.target as usize == target && !c.has(flag::LEADER) && !c.mounted()
        };
        let n = critters.iter().filter(|c| members(c)).count();
        for c in critters.iter_mut().filter(|c| members(c)) {
            c.slot = NO_SLOT;
        }
        let seen = pack.seen[target];
        if n == 0 || !seen.alive {
            continue;
        }
        let arc = n <= line_below;
        let places = (n * 2).clamp(2, MAX_RING);
        let front = seen.facing;
        // Place zero is straight behind for a ring; an arc spreads a half turn
        // across the front.
        let base = if arc {
            front.wrapping_sub(1 << 14)
        } else {
            front.wrapping_add(1 << 15)
        };
        let step = if arc {
            ((1u32 << 15) / (places as u32 - 1).max(1)) as u16
        } else {
            ((1u32 << 16) / places as u32) as u16
        };
        let centre = if arc {
            front
        } else {
            front.wrapping_add(1 << 15)
        };
        let lead = lead_point(pack, &seen);
        let at = |k: usize| {
            let yaw = base.wrapping_add(step.wrapping_mul(k as u16));
            lead.add(V3::from_turns(Fx::from_raw(yaw as i32)).scale(radius))
        };
        // Score every place: toward the back (or the middle of the arc), and
        // nothing at all with a solid between it and the target.
        let mut score = [i32::MIN; MAX_RING];
        for (k, s) in score.iter_mut().enumerate().take(places) {
            let yaw = base.wrapping_add(step.wrapping_mul(k as u16));
            let off = Fx::from_raw(yaw.wrapping_sub(centre) as i16 as i32);
            let toward = Fx::ONE.add(cos_turns(off)).mul(Fx::ratio(1, 2));
            let p = at(k);
            let knee = |v: V3| V3::new(v.x, v.y.add(Fx::ratio(1, 2)), v.z);
            let open =
                w.scene.arena.inside(p) && crate::aim::line_clear(knee(lead), knee(p), w.scene);
            if open {
                *s = Fx::from_int(bias).mul(toward).to_int();
            }
        }
        // Greedy, in a fixed order: the best place first, to the nearest member
        // nobody has placed. Ties go to the lower index.
        let mut taken = [false; MAX_RING];
        for _ in 0..n {
            let mut best_k: Option<usize> = None;
            for k in 0..places {
                if taken[k] || score[k] == i32::MIN {
                    continue;
                }
                if best_k.is_none_or(|b| score[k] > score[b]) {
                    best_k = Some(k);
                }
            }
            let Some(k) = best_k else { break };
            taken[k] = true;
            let p = at(k);
            let mut nearest: Option<(usize, Fx)> = None;
            for (i, c) in critters.iter().enumerate() {
                if !members(c) || c.slot != NO_SLOT {
                    continue;
                }
                let d = crate::math::big_len(p.sub(c.pos));
                if nearest.is_none_or(|(_, b)| d.raw() < b.raw()) {
                    nearest = Some((i, d));
                }
            }
            if let Some((i, _)) = nearest {
                critters[i].slot = k as u8;
            }
        }
        let s = &mut pack.seen[target];
        s.ring_places = places as u8;
        s.ring_base = base;
        s.arc = arc;
    }
}

/// Where a ring place is, as things stand: the place's bearing from the
/// fighter's lead point, at the ring's radius.
pub fn ring_point(pack: &Pack, seen: &Seen, slot: u8) -> V3 {
    let sp = pack.sp();
    let places = seen.ring_places.max(2) as u32;
    let step = if seen.arc {
        ((1u32 << 15) / (places - 1).max(1)) as u16
    } else {
        ((1u32 << 16) / places) as u16
    };
    let yaw = seen.ring_base.wrapping_add(step.wrapping_mul(slot as u16));
    lead_point(pack, seen)
        .add(V3::from_turns(Fx::from_raw(yaw as i32)).scale(sp.pack_fx(PackKnob::RingRadius)))
}

/// Count a critter's state down, and move it on when it runs out.
fn tick(pack: &mut Pack, critters: &mut Critters, i: usize, w: &World) {
    let sp = pack.sp();
    let c = &mut critters[i];
    let _ = w;
    match c.state {
        is::STARTUP | is::ACTIVE | is::RECOVERY | is::FLINCH => {
            c.timer = c.timer.saturating_sub(1);
            if c.timer > 0 {
                return;
            }
            let a = sp.attack(c.act);
            match c.state {
                is::STARTUP => {
                    c.state = is::ACTIVE;
                    c.timer = a.active.max(1);
                    c.set(flag::HIT_USED, false);
                }
                is::ACTIVE => {
                    c.state = is::RECOVERY;
                    c.timer = a.recovery.max(1);
                }
                _ => {
                    c.state = is::PROWL;
                    give_back(pack, c);
                }
            }
        }
        is::DEAD => c.timer = c.timer.saturating_sub(1),
        _ => {}
    }
}

/// A decision point: throw something, or keep circling.
fn think(pack: &mut Pack, critters: &mut Critters, i: usize, w: &World) {
    let sp = pack.sp();
    let Some(decl) = sp.pack else { return };
    if pack.grace > 0 || matches!(pack.mood, mood::CALM | mood::ROUTED | mood::BROKEN) {
        return;
    }
    let free = tokens_out(pack, critters, w.herd) < pack.token_cap();
    let kind = sp.kind(critters[i].kind);
    let mut best: Option<(CritterMove, i32)> = None;
    {
        let look = Look {
            sp,
            pack,
            critters,
            herd: w.herd,
            arena: w.scene.arena,
            frame: w.frame,
        };
        for m in kind.moves {
            if m.token && !free {
                continue;
            }
            let a = sp.attack(m.kind);
            let score = decl.mind.appetite(&look, i, *m, &a);
            if score > 0 && best.is_none_or(|(_, b)| score > b) {
                best = Some((*m, score));
            }
        }
    }
    let Some((m, _)) = best else { return };
    // Individuals are noisy: a coin weighted by nothing in particular decides
    // whether this one goes now or waits for its next turn. The pack's rules
    // decide how many go; this decides which.
    if pack.roll() % 4 == 0 {
        return;
    }
    let a = sp.attack(m.kind);
    let c = &mut critters[i];
    c.state = is::STARTUP;
    c.act = m.kind;
    c.timer = a.startup.max(1);
    c.set(flag::HIT_USED, false);
    if m.token {
        c.set(flag::TOKEN, true);
    }
}

/// The generic pack's answer to "where does critter `i` go".
fn plain_steer(look: &Look, i: usize) -> Steer {
    let sp = look.sp;
    let pack = look.pack;
    let c = &look.critters[i];
    let walk = stat_fx(sp, c.kind, CritterField::Walk);
    let run = stat_fx(sp, c.kind, CritterField::Run);
    let hold = Steer {
        to: c.pos,
        speed: Fx::ZERO,
        face: None,
    };
    if !c.alive() || c.state != is::PROWL {
        // Committed or down: the move or the blow decides where it goes. The
        // windup turns to its target until it locks.
        if c.state == is::STARTUP && c.timer > stat(sp, c.kind, CritterField::Lock).max(0) as u16 {
            return Steer {
                face: Some(look.lead(c.target as usize)),
                ..hold
            };
        }
        return hold;
    }
    let target = (c.target as usize).min(MAX_PLAYERS - 1);
    let seen = &pack.seen[target];
    let lead = look.lead(target);
    match pack.mood {
        mood::ROUTED | mood::BROKEN => Steer {
            to: pack.home,
            speed: run,
            face: None,
        },
        mood::CALM => Steer {
            to: pack.home,
            speed: walk,
            face: seen.alive.then_some(seen.pos),
        },
        mood::SCATTERED => {
            let away = c.pos.sub(lead);
            let flat = V3::new(away.x, Fx::ZERO, away.z);
            let dist = sp.pack_fx(PackKnob::ScatterDistance);
            let dir = if flat.flat_len().raw() == 0 {
                c.facing().scale(Fx::ONE.neg())
            } else {
                flat.normalized()
            };
            Steer {
                to: lead.add(dir.scale(dist)),
                speed: run,
                face: Some(lead),
            }
        }
        _ if !seen.alive => hold,
        _ => {
            let to = if c.has(flag::LEADER) {
                // Behind the ring, on the side its pack is on.
                let back = pack.home.sub(lead);
                let back = V3::new(back.x, Fx::ZERO, back.z).normalized();
                let out = sp
                    .pack_fx(PackKnob::RingRadius)
                    .add(sp.pack_fx(PackKnob::LeaderHangback));
                lead.add(back.scale(out))
            } else if c.slot != NO_SLOT {
                ring_point(look.pack, seen, c.slot)
            } else {
                // Placed nowhere: hold at the ring's radius, where it is.
                let off = c.pos.sub(lead);
                let off = V3::new(off.x, Fx::ZERO, off.z).normalized();
                lead.add(off.scale(sp.pack_fx(PackKnob::RingRadius)))
            };
            // Arriving, it slows: full speed far out, walking close in.
            let gap = to.sub(c.pos).flat_len();
            let arrive = sp.pack_fx(PackKnob::Arrive);
            let speed = if gap.raw() > arrive.raw() || arrive.raw() <= 0 {
                run
            } else {
                walk.mul(gap.div(arrive).min(Fx::ONE))
            };
            Steer {
                to,
                speed,
                face: Some(lead),
            }
        }
    }
}

/// Turn the steer into a velocity and a heading.
fn drive(sp: &Species, c: &mut Critter, want: Steer) {
    if !c.present() {
        return;
    }
    let accel = stat_fx(sp, c.kind, CritterField::Accel).mul(DT);
    let flat_vel = V3::new(c.vel.x, Fx::ZERO, c.vel.z);
    let target_vel = match c.state {
        // A move that travels carries the body along the facing.
        is::ACTIVE => c.facing().scale(sp.attack(c.act).advance),
        // A windup goes where its species steers it, which by default is
        // nowhere (`plain_steer` holds it still): the Gnawers' pile-on closes
        // its ring as it crouches.
        is::PROWL | is::STARTUP if c.alive() => {
            let to = want.to.sub(c.pos);
            let to = V3::new(to.x, Fx::ZERO, to.z);
            // Close enough that one more frame at this speed would carry it
            // past: there.
            if to.flat_len().raw() <= want.speed.mul(DT).raw() {
                V3::ZERO
            } else {
                to.normalized().scale(want.speed)
            }
        }
        _ => V3::ZERO,
    };
    if c.state == is::ACTIVE && c.alive() && !c.has(flag::AIRBORNE) {
        // **A lunge is at its speed from its first frame**: the move's
        // `Advance`, not a walk accelerating up to it. Through the
        // acceleration a twelve-frame dart covered a third of its distance and
        // an eight-frame leap almost none, so the volume drawn on the floor
        // was not where the body went.
        c.vel = V3::new(target_vel.x, c.vel.y, target_vel.z);
    } else if !c.has(flag::AIRBORNE) {
        let dv = target_vel.sub(flat_vel);
        let len = dv.flat_len();
        let dv = if len.raw() > accel.raw() {
            dv.normalized().scale(accel)
        } else {
            dv
        };
        c.vel = V3::new(c.vel.x.add(dv.x), c.vel.y, c.vel.z.add(dv.z));
    }
    // Heading: what it was told to look at, or where it is going.
    let look_at = match want.face {
        Some(p) => Some(p.sub(c.pos)),
        // Where it is going, once it is going somewhere faster than a frame's
        // shuffle: standing still, it keeps the heading it had.
        None if c.state == is::PROWL && flat_vel.flat_len().raw() > accel.raw() => Some(flat_vel),
        None => None,
    };
    if let Some(dir) = look_at {
        if dir.flat_len().raw() > 0 && c.alive() && c.state != is::ACTIVE {
            let want_yaw = yaw_of(dir);
            let rate = stat_fx(sp, c.kind, CritterField::Turn).mul(DT);
            let max = rate.raw().clamp(0, 1 << 15);
            let diff = (want_yaw.wrapping_sub(c.yaw) as i16 as i32).clamp(-max, max);
            c.yaw = c.yaw.wrapping_add(diff as u16);
        }
    }
}

/// Gravity, the arena, and the creature under it.
fn move_body(
    sp: &Species,
    c: &mut Critter,
    index: usize,
    want: Option<Steer>,
    w: &World,
    rigs: &[Option<crate::beast::Rig>; crate::monster::MAX_MONSTERS],
) {
    let arena = w.scene.arena;
    let half_wid = stat_fx(sp, c.kind, CritterField::Width).mul(Fx::ratio(1, 2));
    let height = stat_fx(sp, c.kind, CritterField::Height);
    let flat_step = V3::new(c.vel.x, Fx::ZERO, c.vel.z).scale(DT);

    // **Mounted: the perch is authoritative.** Where it is in the world is
    // the part's frame carried through the creature's pose, this frame; its
    // own step is taken in the world and carried back.
    if c.mounted() {
        let slot = mount_slot(c.mount);
        let part = crate::monster::mount_part(c.mount);
        let Some(rig) = rigs.get(slot).and_then(|r| r.as_ref()) else {
            // Nothing under it any more.
            c.mount = NO_PART;
            c.set(flag::AIRBORNE, true);
            return;
        };
        let here = rig.part_to_world(part, c.perch_local());
        let next = here.add(flat_step);
        match rig.surface_under(next, half_wid) {
            Some((on, top)) => {
                let local = rig.world_to_part(on, next);
                let local = V3::new(local.x, top, local.z);
                c.mount = mount_of(slot, on);
                c.set_perch(local);
                c.pos = rig.part_to_world(on, local);
                c.vel.y = Fx::ZERO;
                c.set(flag::AIRBORNE, false);
            }
            None => {
                // Walked off the edge, or was thrown: it falls from here.
                c.mount = NO_PART;
                c.pos = next;
                c.set(flag::AIRBORNE, true);
            }
        }
        return;
    }

    // Gravity is a signed acceleration in the Oven: negative is down.
    c.vel.y = c.vel.y.add(t::gravity().mul(DT));
    let mut pos = c.pos.add(c.vel.scale(DT));
    // Landing on a creature: the same surface a rider lands on.
    if c.vel.y.raw() <= 0 {
        for (slot, rig) in rigs.iter().enumerate() {
            let Some(rig) = rig else { continue };
            if let Some((on, top)) = rig.surface_under(pos, half_wid) {
                let local = rig.world_to_part(on, pos);
                let local = V3::new(local.x, top, local.z);
                c.mount = mount_of(slot, on);
                c.set_perch(local);
                c.pos = rig.part_to_world(on, local);
                c.vel.y = Fx::ZERO;
                c.set(flag::AIRBORNE, false);
                return;
            }
        }
    }
    let r = arena.resolve_sized(pos, c.vel, !c.has(flag::AIRBORNE), half_wid, height);
    if r.wall {
        if let Some(decl) = sp.pack {
            decl.mind
                .bumped(c, V3::new(r.pos.x.sub(pos.x), Fx::ZERO, r.pos.z.sub(pos.z)));
        }
    }
    // **Blocked, it goes round.** There is no path-finding: a critter heading
    // for its place with a platform in the way slides along the face it hit,
    // whichever way it was already leaning -- or, meeting it square, the way
    // its slot number says, so a pack splits round an obstacle rather than all
    // piling to one side. The face is read off the push the arena gave it.
    if let (true, true, Some(want)) = (r.wall, c.state == is::PROWL && c.alive(), want) {
        let push = V3::new(r.pos.x.sub(pos.x), Fx::ZERO, r.pos.z.sub(pos.z));
        if push.flat_len().raw() > 0 && want.speed.raw() > 0 {
            let n = push.normalized();
            let along = V3::new(n.z.neg(), Fx::ZERO, n.x);
            let goal = want.to.sub(r.pos);
            let lean = V3::new(goal.x, Fx::ZERO, goal.z).dot(along);
            // Leaning toward its goal by more than its own width counts as
            // leaning; less is square on.
            // Once round one way, it keeps going that way until it is clear.
            let side = if c.has(flag::DETOUR) {
                if c.has(flag::DETOUR_LEFT) { -1 } else { 1 }
            } else if lean.abs().raw() > half_wid.raw() {
                lean.raw().signum()
            } else if index % 2 == 0 {
                1
            } else {
                -1
            };
            c.set(flag::DETOUR, true);
            c.set(flag::DETOUR_LEFT, side < 0);
            let slide = along.scale(want.speed.mul(Fx::from_raw(side << 16)));
            c.vel = V3::new(slide.x, r.vel.y, slide.z);
            c.set(flag::AIRBORNE, !r.grounded);
            finish(c, r.pos, half_wid, w);
            return;
        }
    }
    pos = r.pos;
    c.vel = r.vel;
    c.set(flag::AIRBORNE, !r.grounded);
    if !r.wall {
        c.set(flag::DETOUR, false);
    }
    finish(c, pos, half_wid, w);
}

/// The bounds and the stones, after the arena: where a body that is not on a
/// creature ends its frame.
fn finish(c: &mut Critter, mut pos: V3, half_wid: Fx, w: &World) {
    let arena = w.scene.arena;
    // And kept inside the arena's bounds, less its own width.
    let b = &arena.bounds;
    pos.x = pos.x.clamp(b.lo_x.add(half_wid), b.hi_x.sub(half_wid));
    pos.z = pos.z.clamp(b.lo_z.add(half_wid), b.hi_z.sub(half_wid));
    // Stones are solids too.
    for stone in w.scene.stones.iter().flatten() {
        let d = V3::new(pos.x.sub(stone.at.x), Fx::ZERO, pos.z.sub(stone.at.z));
        let reach = stone.radius().add(half_wid);
        let gap = d.flat_len();
        let above = pos.y.raw() >= stone.top().sub(crate::arena::SKIN).raw();
        if gap.raw() < reach.raw() && !above && stone.standing_height().raw() > 0 {
            let out = if gap.raw() == 0 {
                V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO)
            } else {
                d.normalized()
            };
            pos = V3::new(stone.at.x, pos.y, stone.at.z).add(out.scale(reach));
        }
    }
    c.pos = pos;
}

/// Bodies keep `Spacing` from each other, and the kinds that yield step out
/// of a fighter's way. Pairs in index order, so the answer does not depend on
/// anything but the snapshot.
fn keep_apart(sp: &Species, critters: &mut Critters, w: &World) {
    let spacing = sp.pack_fx(PackKnob::Spacing);
    let half = Fx::ratio(1, 2);
    for i in 0..MAX_CRITTERS {
        if !critters[i].alive() || critters[i].mounted() {
            continue;
        }
        for j in (i + 1)..MAX_CRITTERS {
            if !critters[j].alive() || critters[j].mounted() {
                continue;
            }
            let d = critters[j].pos.sub(critters[i].pos);
            let d = V3::new(d.x, Fx::ZERO, d.z);
            let gap = d.flat_len();
            if gap.raw() >= spacing.raw() {
                continue;
            }
            let dir = if gap.raw() == 0 {
                V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO)
            } else {
                d.normalized()
            };
            let push = dir.scale(spacing.sub(gap).mul(half));
            critters[i].pos = critters[i].pos.sub(push);
            critters[j].pos = critters[j].pos.add(push);
        }
    }
    for c in critters.iter_mut().filter(|c| c.alive() && !c.mounted()) {
        if !sp.kind(c.kind).yields {
            continue;
        }
        let half_wid = stat_fx(sp, c.kind, CritterField::Width).mul(half);
        for p in w.players.iter().filter(|p| p.health > 0) {
            let d = V3::new(c.pos.x.sub(p.pos.x), Fx::ZERO, c.pos.z.sub(p.pos.z));
            let reach = t::body_radius().add(half_wid);
            let gap = d.flat_len();
            if gap.raw() < reach.raw() {
                let out = if gap.raw() == 0 {
                    V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO)
                } else {
                    d.normalized()
                };
                c.pos = V3::new(p.pos.x, c.pos.y, p.pos.z).add(out.scale(reach));
            }
        }
    }
}
