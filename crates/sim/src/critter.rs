//! Small bodies: what a pack is made of.
//!
//! A [`crate::monster::Monster`] is about two hundred bytes of a four-kilobyte
//! snapshot, most of it a skeleton people stand on and break. A knee-high biter
//! needs none of that: nothing stands on a gnawer, and its pose is presentation.
//! So a **critter** is a box on its feet, a velocity, a yaw, a health, a state,
//! a timer and an animation clock -- forty-eight bytes -- and up to
//! [`MAX_CRITTERS`] of them live in the world beside the creature slots. What
//! one *does* is decided by its pack ([`crate::pack`]); what one *is* is a row
//! of its species' table ([`CritterKind`]) and a row of knobs per kind
//! ([`CritterField`]).
//!
//! # The hit shape, decided
//!
//! **A box standing on the critter's feet, yawed with it**: its length along
//! the facing, its width across, its height from the soles to the crown. The
//! bestiary (§6, "Critter hit shape") asked for one shape, and had two
//! proposals: the Gnawers' capsule lying along the yaw, and the Hornback's box
//! that reaches the floor. The box wins, for the Hornback's reason -- a level
//! shot under a quadruped is the Ridgeback's lesson and nobody is meant to
//! shoot between a cow's legs -- and it costs the gnawer nothing: its capsule
//! touched the floor anyway, and the box is the same body with corners. It is
//! also the shape `aim::stands_at` reads the crown off, so "how tall is what
//! the crosshair passed through" and "what does a swing hit" are the same
//! number.
//!
//! The one description of it is [`Critter::body`]; the hit test, the aiming
//! ray, the debug overlay and the renderer all read it.
//!
//! # Standing on a creature
//!
//! A critter can live on a monster's mountable parts (the Siegeshell's
//! parasites), by the rule riders follow: **mounted, the position in the
//! part's own frame is authoritative**. [`Critter::mount`] names the creature
//! slot and the part, packed as a rider's is (`monster::mount_of`), and
//! [`Critter::perch`] holds the point in that part's frame, in centimetres;
//! `pos` is then worked out from them every frame, so the animal moving under
//! it moves it. See `crate::pack`'s physics.

use crate::fixed::{Fx, cos_turns, sin_turns};
use crate::math::V3;
use crate::monster::{Herd, NO_PART, mount_part, mount_slot};
use crate::oven::{Unit, species_raw};
use crate::species::{Species, SpeciesId};
use crate::state::Hitbox;

/// The most small bodies a world holds at once.
///
/// Ten, from the worst fight in the bestiary (§4): nine gnawers and the Big
/// One in coop. The herd is nine, a brood eight, the Siegeshell's parasites
/// eight. A pack brain drives all of them.
pub const MAX_CRITTERS: usize = 10;

/// Every critter slot a world has, and whose they are.
///
/// One species' critters at a time -- one pack per fight -- so the table the
/// bodies' sizes and moves are read from is a byte here rather than a byte on
/// each of them. Derefs to the array, so `critters[i]` and `critters.iter()`
/// read as they would on a plain one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Critters {
    pub species: SpeciesId,
    pub all: [Critter; MAX_CRITTERS],
}

impl Critters {
    /// No small bodies at all: every fight that is not a pack's.
    pub const NONE: Critters = Critters {
        species: SpeciesId::GNATS,
        all: [Critter::EMPTY; MAX_CRITTERS],
    };

    /// Their table.
    pub fn sp(&self) -> &'static Species {
        self.species.get()
    }

    /// Any body at all, alive or dead.
    pub fn any(&self) -> bool {
        self.all.iter().any(Critter::present)
    }
}

impl Default for Critters {
    fn default() -> Critters {
        Critters::NONE
    }
}

impl std::ops::Deref for Critters {
    type Target = [Critter; MAX_CRITTERS];
    fn deref(&self) -> &Self::Target {
        &self.all
    }
}

impl std::ops::DerefMut for Critters {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.all
    }
}

/// What a critter is doing, as the one byte the snapshot keeps.
pub mod is {
    /// The slot is free.
    pub const EMPTY: u8 = 0;
    /// Standing, walking, circling: the only state a move starts from.
    pub const PROWL: u8 = 1;
    /// Winding up a move. `act` is which, `timer` the frames left.
    pub const STARTUP: u8 = 2;
    pub const ACTIVE: u8 = 3;
    pub const RECOVERY: u8 = 4;
    /// Knocked out of whatever it was doing.
    pub const FLINCH: u8 = 5;
    /// Down. The body keeps its slot, and its clock, until the slot is needed:
    /// a corpse fading out must not pop on a rollback. `timer` counts the fade.
    pub const DEAD: u8 = 6;
    /// Left the fight: ran into its den after a rout it will not come back
    /// from. Its slot is free to reuse.
    pub const GONE: u8 = 7;
}

/// Bits of [`Critter::flags`].
pub mod flag {
    /// It holds one of the pack's attack tokens.
    pub const TOKEN: u8 = 1 << 0;
    /// The move it has out has already connected.
    pub const HIT_USED: u8 = 1 << 1;
    /// The pack's leader.
    pub const LEADER: u8 = 1 << 2;
    /// Off the floor.
    pub const AIRBORNE: u8 = 1 << 3;
    /// Already struck by fighter 0's current swing; the next bit fighter 1's.
    /// A sweep through a heap catches every body in it **once**.
    pub const STRUCK: u8 = 1 << 4;
    /// Going round something in its way, and which way: see `pack`'s
    /// physics. Kept so a body sliding along a wall does not change its mind
    /// halfway along it.
    pub const DETOUR: u8 = 1 << 6;
    pub const DETOUR_LEFT: u8 = 1 << 7;
}

/// Not in a ring slot.
pub const NO_SLOT: u8 = u8::MAX;

/// One small body. Forty-eight bytes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Critter {
    /// Its feet, in the world. Worked out from `perch` while mounted.
    pub pos: V3,
    pub vel: V3,
    /// Facing, in turns as the low sixteen bits of 16.16: zero looks down +X.
    pub yaw: u16,
    pub health: i16,
    /// Frames left in the state, or of the corpse's fade.
    pub timer: u16,
    /// **The animation clock.** Frames since it spawned, wrapping; what its
    /// pose is drawn from. In the snapshot so that a rollback re-poses the
    /// body exactly rather than popping it -- see `docs/design/critters.md`.
    pub clock: u16,
    /// Which of its pack's [`CritterKind`]s, by index.
    pub kind: u8,
    /// One of [`is`].
    pub state: u8,
    /// The move in progress, by index into its species' moves.
    pub act: u8,
    /// [`flag`] bits.
    pub flags: u8,
    /// What the species uses it for: a cow or the bull, a member or a digger.
    /// Generic code never reads it except to start it at the kind's own.
    pub role: u8,
    /// Its place on the pack's ring, or [`NO_SLOT`].
    pub slot: u8,
    /// The fighter it is after.
    pub target: u8,
    /// The creature and part it stands on, packed as a rider's mount is
    /// (`monster::mount_of`), or `monster::NO_PART`.
    pub mount: u8,
    /// Where on that part, in the part's own frame, in centimetres.
    pub perch: [i16; 3],
    /// **Which thrown things have already cut it**: one bit per fighter and
    /// move slot ([`seared_bit`]). A blade or an arm cuts a body once a pass,
    /// and an effect's own memory has no room for ten more victims; the bit is
    /// cleared when that fighter's next one of that move is thrown. Two bytes
    /// that were padding.
    pub seared: u16,
}

/// The bit of [`Critter::seared`] an effect thrown by `owner` from move slot
/// `slot` keeps. Two fighters, eight slots each.
pub const fn seared_bit(owner: u8, slot: u8) -> u16 {
    1 << ((owner as u16 & 1) * 8 + (slot as u16 & 7))
}

impl Critter {
    /// An empty slot.
    pub const EMPTY: Critter = Critter {
        pos: V3::ZERO,
        vel: V3::ZERO,
        yaw: 0,
        health: 0,
        timer: 0,
        clock: 0,
        kind: 0,
        state: is::EMPTY,
        act: 0,
        flags: 0,
        role: 0,
        slot: NO_SLOT,
        target: 0,
        mount: NO_PART,
        perch: [0; 3],
        seared: 0,
    };

    /// A fresh one of `kind`, standing at `at` and facing `yaw`.
    pub fn new(sp: &Species, kind: u8, at: V3, yaw: u16) -> Critter {
        let decl = sp.kind(kind);
        Critter {
            pos: at,
            health: stat(sp, kind, CritterField::Health).clamp(1, i16::MAX as i32) as i16,
            yaw,
            kind,
            state: is::PROWL,
            role: decl.role,
            ..Critter::EMPTY
        }
    }

    /// The slot holds a body, alive or not.
    pub const fn present(&self) -> bool {
        !matches!(self.state, is::EMPTY | is::GONE)
    }

    /// Still in the fight.
    pub const fn alive(&self) -> bool {
        matches!(
            self.state,
            is::PROWL | is::STARTUP | is::ACTIVE | is::RECOVERY | is::FLINCH
        )
    }

    /// Winding up or throwing a move.
    pub const fn attacking(&self) -> bool {
        matches!(self.state, is::STARTUP | is::ACTIVE)
    }

    pub const fn has(&self, bit: u8) -> bool {
        self.flags & bit != 0
    }

    pub fn set(&mut self, bit: u8, on: bool) {
        if on {
            self.flags |= bit;
        } else {
            self.flags &= !bit;
        }
    }

    pub const fn mounted(&self) -> bool {
        self.mount != NO_PART
    }

    /// Facing, as turns.
    pub const fn yaw_turns(&self) -> Fx {
        Fx::from_raw(self.yaw as i32)
    }

    /// Facing, as a unit vector in the floor plane.
    pub fn facing(&self) -> V3 {
        V3::from_turns(self.yaw_turns())
    }

    /// The body the hit test, the aiming ray, the overlay and the renderer all
    /// read. See the module docs for why it is a box.
    pub fn body(&self, sp: &Species) -> Body {
        let half = Fx::ratio(1, 2);
        Body {
            foot: self.pos,
            cos: cos_turns(self.yaw_turns()),
            sin: sin_turns(self.yaw_turns()),
            half_len: stat_fx(sp, self.kind, CritterField::Length).mul(half),
            half_wid: stat_fx(sp, self.kind, CritterField::Width).mul(half),
            height: stat_fx(sp, self.kind, CritterField::Height),
        }
    }

    /// The perch as fixed point, in the part's frame.
    pub fn perch_local(&self) -> V3 {
        let cm = |v: i16| Fx::ratio(v as i32, 100);
        V3::new(cm(self.perch[0]), cm(self.perch[1]), cm(self.perch[2]))
    }

    /// Store a point in the part's frame as the perch. Centimetres: a few
    /// metres of shell at a centimetre's grain, recomputed from scratch every
    /// frame, so nothing drifts.
    pub fn set_perch(&mut self, local: V3) {
        let cm = |v: Fx| v.mul(Fx::from_int(100)).to_int().clamp(-32000, 32000) as i16;
        self.perch = [cm(local.x), cm(local.y), cm(local.z)];
    }

    /// Where a mounted critter is in the world, from its perch on the creature
    /// under it -- or `None` if that creature is gone.
    pub fn perched_at(&self, herd: &Herd) -> Option<V3> {
        if !self.mounted() {
            return None;
        }
        let beast = herd.get(mount_slot(self.mount))?.as_ref()?;
        Some(beast.world_of(mount_part(self.mount), self.perch_local()))
    }

    /// The volume its move has out this frame, in the world: an upright
    /// cylinder's anchor, its radius and its height span -- the same shape a
    /// monster's move is ([`crate::monster::Monster::hit_volume`]), so the
    /// overlay and the telegraph draw both the same way.
    ///
    /// The anchor is authored in the critter's own frame (`HitX` forward,
    /// `HitZ` across) and the volume travels along the facing at `Travel`.
    pub fn hit_volume(&self, sp: &Species) -> Option<(V3, Fx, Fx, Fx)> {
        if self.state != is::ACTIVE {
            return None;
        }
        self.volume_at(sp, self.timer)
    }

    /// The volume as it will be on an active frame with `left` frames to go.
    fn volume_at(&self, sp: &Species, left: u16) -> Option<(V3, Fx, Fx, Fx)> {
        let m = sp.attack(self.act);
        if m.damage <= 0 || m.hit_radius.raw() <= 0 {
            return None;
        }
        let flown = m
            .travel
            .mul(Fx::from_int(
                m.active.saturating_sub(left).saturating_sub(1) as i32,
            ))
            .mul(crate::DT);
        let fwd = self.facing();
        let side = V3::new(fwd.z.neg(), Fx::ZERO, fwd.x);
        let anchor = self
            .pos
            .add(fwd.scale(m.hit_x.add(flown)))
            .add(side.scale(m.hit_z));
        Some((
            V3::new(anchor.x, self.pos.y, anchor.z),
            m.hit_radius,
            self.pos.y.add(m.hit_low),
            self.pos.y.add(m.hit_high),
        ))
    }

    /// **What is coming, and where it will land**: the volume its move will
    /// have when the hit comes out, for drawing through the windup -- the hit
    /// test's own answer, as the monster's telegraph is.
    pub fn telegraph(&self, sp: &Species) -> Option<crate::monster::Telegraph> {
        let m = sp.attack(self.act);
        let (live, progress, frames) = match self.state {
            is::STARTUP => (
                false,
                Fx::ONE.sub(Fx::ratio(self.timer as i32, m.startup.max(1) as i32)),
                m.active,
            ),
            is::ACTIVE => (true, Fx::ONE, self.timer),
            _ => return None,
        };
        let (anchor, radius, low, high) = if live {
            self.hit_volume(sp)?
        } else {
            self.volume_at(sp, m.active)?
        };
        Some(crate::monster::Telegraph {
            kind: self.act,
            anchor,
            radius,
            low,
            high,
            along: self.facing(),
            sweep: m
                .travel
                .add(m.advance)
                .mul(Fx::from_int(frames as i32))
                .mul(crate::DT),
            progress,
            live,
        })
    }

    /// Does the volume out this frame reach a fighter standing at `world`?
    pub fn reaches(&self, sp: &Species, world: V3, hurt_height: Fx, body_radius: Fx) -> bool {
        let Some((anchor, radius, low, high)) = self.hit_volume(sp) else {
            return false;
        };
        let flat = V3::new(world.x.sub(anchor.x), Fx::ZERO, world.z.sub(anchor.z)).flat_len();
        if flat.raw() > radius.add(body_radius).raw() {
            return false;
        }
        world.y.add(hurt_height).raw() >= low.raw() && world.y.raw() <= high.raw()
    }
}

impl Default for Critter {
    fn default() -> Critter {
        Critter::EMPTY
    }
}

/// A critter's body: a box standing on `foot`, yawed. Length along the
/// facing, width across it, height up from the soles.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Body {
    pub foot: V3,
    /// The facing's cosine and sine, so nothing below needs the angle.
    pub cos: Fx,
    pub sin: Fx,
    pub half_len: Fx,
    pub half_wid: Fx,
    pub height: Fx,
}

impl Body {
    /// A world point in the body's own frame: `x` forward, `y` up from the
    /// soles, `z` across.
    pub fn to_local(&self, world: V3) -> V3 {
        let d = world.sub(self.foot);
        V3::new(
            d.x.mul(self.cos).add(d.z.mul(self.sin)),
            d.y,
            d.z.mul(self.cos).sub(d.x.mul(self.sin)),
        )
    }

    /// A world direction in the body's frame.
    pub fn dir_to_local(&self, d: V3) -> V3 {
        V3::new(
            d.x.mul(self.cos).add(d.z.mul(self.sin)),
            d.y,
            d.z.mul(self.cos).sub(d.x.mul(self.sin)),
        )
    }

    /// A point in the body's frame, in the world.
    pub fn to_world(&self, local: V3) -> V3 {
        V3::new(
            self.foot
                .x
                .add(local.x.mul(self.cos))
                .sub(local.z.mul(self.sin)),
            self.foot.y.add(local.y),
            self.foot
                .z
                .add(local.x.mul(self.sin))
                .add(local.z.mul(self.cos)),
        )
    }

    /// The box's corners in its own frame.
    pub fn min(&self) -> V3 {
        V3::new(self.half_len.neg(), Fx::ZERO, self.half_wid.neg())
    }

    pub fn max(&self) -> V3 {
        V3::new(self.half_len, self.height, self.half_wid)
    }

    /// The crown, in the world.
    pub fn crown(&self) -> Fx {
        self.foot.y.add(self.height)
    }

    /// The middle of it.
    pub fn middle(&self) -> V3 {
        V3::new(
            self.foot.x,
            self.foot.y.add(self.height.mul(Fx::ratio(1, 2))),
            self.foot.z,
        )
    }

    /// How far the box reaches from its middle at most: half its diagonal.
    /// For throwing out what cannot possibly touch before the exact test.
    pub fn bound(&self) -> Fx {
        let h = self.height.mul(Fx::ratio(1, 2));
        crate::math::big_len(V3::new(self.half_len, h, self.half_wid))
    }

    /// Can a capsule from `a` to `b` of `radius` not possibly reach it? One
    /// closed-form distance, before the search [`Body::gap_to_segment`] does.
    fn out_of_reach(&self, a: V3, b: V3, radius: Fx) -> bool {
        let m = self.middle();
        crate::math::segment_gap(a, b, m, m).raw() > radius.add(self.bound()).raw()
    }

    /// The gap between this body and a line segment: zero when the segment
    /// passes through it.
    pub fn gap_to_segment(&self, a: V3, b: V3) -> Fx {
        crate::math::segment_box_gap(self.to_local(a), self.to_local(b), self.min(), self.max())
    }

    /// **Does a fighter's attack volume touch this body?** The one hit test,
    /// for every shape `state::hitbox` can describe: a section of a ring, a
    /// flat disc with no top or bottom, and a capsule. The debug overlay draws
    /// `state::hitbox` and [`Critter::body`], so what it shows is this.
    pub fn touched_by(&self, hb: &Hitbox) -> bool {
        if let Some(ring) = hb.sector {
            // A ring's own test, about a column standing at the middle of the
            // body, as wide as the body is across.
            return ring.touches(self.foot, self.height, hb.radius.add(self.half_wid));
        }
        if hb.flat {
            let c = self.to_local(hb.centre());
            return crate::math::flat_box_gap(c, self.min(), self.max()).raw() <= hb.radius.raw();
        }
        !self.out_of_reach(hb.from, hb.to, hb.radius)
            && self.gap_to_segment(hb.from, hb.to).raw() <= hb.radius.raw()
    }

    /// Does a level blade sweeping from `a` to `b` this frame touch it? The
    /// slab is `radius` either side of the line in the floor plane and
    /// `half_thick` above and below it: the Guillotine's blades, the same
    /// slab `state::World::sliced` cuts a fighter with.
    pub fn touched_by_slab(&self, a: V3, b: V3, radius: Fx, half_thick: Fx) -> bool {
        if self.out_of_reach(a, b, radius.add(half_thick)) {
            return false;
        }
        let mid = self.height.mul(Fx::ratio(1, 2));
        let level = |v: V3| {
            let l = self.to_local(v);
            V3::new(l.x, mid, l.z)
        };
        let wide = crate::math::segment_box_gap(level(a), level(b), self.min(), self.max());
        if wide.raw() > radius.raw() {
            return false;
        }
        let lo = b.y.sub(half_thick);
        let hi = b.y.add(half_thick);
        self.foot.y.raw() <= hi.raw() && self.crown().raw() >= lo.raw()
    }

    /// Does an upright cylinder -- a field on the floor, a burst -- overlap it?
    /// `foot` is the cylinder's base, `height` how tall it stands.
    pub fn touched_by_column(&self, foot: V3, radius: Fx, height: Fx) -> bool {
        let c = self.to_local(foot);
        if crate::math::flat_box_gap(c, self.min(), self.max()).raw() > radius.raw() {
            return false;
        }
        foot.y.add(height).raw() >= self.foot.y.raw() && foot.y.raw() <= self.crown().raw()
    }
}

// ---------------------------------------------------------------------------
// Kinds
// ---------------------------------------------------------------------------

/// One move a kind of critter can throw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CritterMove {
    /// Index into the species' moves: its numbers are that move's knobs
    /// (`oven::MonsterField`), the same row a monster's move has.
    pub kind: u8,
    /// **It needs one of the pack's attack tokens.** Declared rather than
    /// inferred, as a fighter's line of effect is: the gnawers' bite does, its
    /// pile-on does not.
    pub token: bool,
}

impl CritterMove {
    pub const fn token(kind: u8) -> CritterMove {
        CritterMove { kind, token: true }
    }

    pub const fn free(kind: u8) -> CritterMove {
        CritterMove { kind, token: false }
    }
}

/// One kind of small body in a pack: the Gnawers' gnawer and their Big One,
/// the Hornback's cow and its bull. What it can do; how big and fast it is are
/// knobs ([`CritterField`]), one row per kind in the species' store.
#[derive(Clone, Copy, Debug)]
pub struct CritterKind {
    /// As a person says it, and the family its knobs are shown under.
    pub name: &'static str,
    pub moves: &'static [CritterMove],
    /// The role it starts in. See [`Critter::role`].
    pub role: u8,
    /// **Fighters push it aside.** Off for a gnawer -- seven bodies that could
    /// wall somebody in would be an unanswerable trap -- and on for a cow.
    /// Either way a fighter is never moved by one.
    pub yields: bool,
}

/// One tuned number of one kind of critter. Every kind has a row of these in
/// its species' store, after the moves and the pack's own knobs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CritterField {
    Health,
    /// The body box: nose to tail, across, and soles to crown.
    Length,
    Width,
    Height,
    Walk,
    /// The dart: how fast it goes when it means it.
    Run,
    Accel,
    /// Turns a second.
    Turn,
    /// Frames before the hit at which the windup stops following its target.
    /// A sidestep inside this is a sidestep the move cannot answer.
    Lock,
    /// Damage that knocks it out of what it is doing. Zero: anything does --
    /// what "they're light" means.
    FlinchAt,
    FlinchFrames,
    /// How much of a blow's knockback and launch it takes. One is a fighter's.
    Shove,
    /// How long the body lies before it fades.
    Corpse,
}

/// How many fields a kind has.
pub const CRITTER_FIELDS: usize = 13;

impl CritterField {
    pub const ALL: &'static [CritterField] = &[
        CritterField::Health,
        CritterField::Length,
        CritterField::Width,
        CritterField::Height,
        CritterField::Walk,
        CritterField::Run,
        CritterField::Accel,
        CritterField::Turn,
        CritterField::Lock,
        CritterField::FlinchAt,
        CritterField::FlinchFrames,
        CritterField::Shove,
        CritterField::Corpse,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            CritterField::Health => "Health",
            CritterField::Length => "Length",
            CritterField::Width => "Width",
            CritterField::Height => "Height",
            CritterField::Walk => "Walk speed",
            CritterField::Run => "Dart speed",
            CritterField::Accel => "Acceleration",
            CritterField::Turn => "Turn rate (turns/s)",
            CritterField::Lock => "Windup stops tracking",
            CritterField::FlinchAt => "Damage that flinches it",
            CritterField::FlinchFrames => "Flinch length",
            CritterField::Shove => "Knockback taken (x)",
            CritterField::Corpse => "Body lies for",
        }
    }

    pub const fn unit(self) -> Unit {
        match self {
            CritterField::Health | CritterField::FlinchAt => Unit::Int,
            CritterField::Lock | CritterField::FlinchFrames | CritterField::Corpse => Unit::Frames,
            _ => Unit::Fixed,
        }
    }

    pub const fn range(self) -> (i32, i32) {
        const fn fx(n: i32, d: i32) -> i32 {
            Fx::ratio(n, d).raw()
        }
        match self {
            CritterField::Health => (1, 3000),
            CritterField::Length | CritterField::Width | CritterField::Height => {
                (fx(1, 10), fx(8, 1))
            }
            CritterField::Walk | CritterField::Run => (0, fx(30, 1)),
            CritterField::Accel => (fx(1, 1), fx(200, 1)),
            CritterField::Turn => (fx(1, 20), fx(8, 1)),
            CritterField::Lock => (0, 60),
            CritterField::FlinchAt => (0, 2000),
            CritterField::FlinchFrames => (0, 120),
            CritterField::Shove => (0, fx(3, 1)),
            CritterField::Corpse => (0, 1800),
        }
    }
}

/// One of a kind's numbers, live from the Oven.
pub fn stat(sp: &Species, kind: u8, field: CritterField) -> i32 {
    species_raw(sp.id, sp.critter_index(kind as usize, field))
}

/// One of a kind's numbers, as fixed point.
pub fn stat_fx(sp: &Species, kind: u8, field: CritterField) -> Fx {
    Fx::from_raw(stat(sp, kind, field))
}
