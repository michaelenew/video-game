//! Floor hazards (bestiary P4): tar, web, smoke, burning ground, a sinkhole's
//! pull, vented steam, slag.
//!
//! **One small fixed list, both drawn and hit-tested.** Each hazard is a
//! sixteen-byte cell in the hunt's [`Lore`]: a kind, a state, an anchor, a
//! radius, an age, and a centre (and, for a line, its other end). Every frame
//! the world turns the list into a [`Floor`] -- each entry placed in the world,
//! with its kind's table and knobs -- and that one value is what the fighters'
//! feet, the creatures, the aiming ray, the perception filter and the renderer
//! all read. The overlay rule, applied to the floor: a hazard you can see is
//! the hazard that slows you, because there is only one of it.
//!
//! **Kinds belong to species.** A species declares its kinds in its own file
//! ([`HazardDecl`], in its [`crate::species::FightDecl`]) and each kind gets a
//! row of knobs in the Oven ([`HazardField`]: "Mireback · tar"), so nothing
//! here names a creature. The effects are generic and any kind may use any of
//! them:
//!
//! - **slow** -- walk, dodge distance and jump, each its own multiplier; all
//!   three at zero is a **root**;
//! - **damage over time** -- so much every so many frames;
//! - **pull** toward its centre (a disc) or its line (a strand);
//! - **lift** -- a push straight up (a vent);
//! - **blocks sight** -- a creature's glance and `aim::in_view` cannot see
//!   through it (smoke);
//! - **fire** -- a kind that `ignites` into another when fire touches it (tar
//!   into burning tar), a kind that `burns` (fire: it lights what it touches,
//!   every `Spread` frames), and what a kind `becomes` when its life runs out
//!   (burning tar into slag, or nothing);
//! - **solid** -- it stands as a box of the arena's own kind, `Solid` metres
//!   per layer, which `ground_under`, every body's collision and the aiming
//!   ray meet like any wall ([`crate::arena::Terrain`]). A kind becoming a
//!   solid where one of its kind already stands adds a layer to it instead.
//!
//! **Anchored hazards** live in a creature part's own frame -- the
//! Siegeshell's vents -- and are carried by its pose exactly as a rider or a
//! perched critter is: the cell holds the point in the part's frame, and the
//! [`Floor`] places it through the creature every frame.
//!
//! What a species does *with* its kinds -- where a pool lands, whether two
//! merge, what trips on a strand -- is its own code, through [`place`] and
//! [`Lore`]. Docs: `docs/design/hazards.md`.

use crate::arena::{Material, Solid};
use crate::fixed::Fx;
use crate::lore::{self, Cell, Lore};
use crate::math::{self, V3};
use crate::monster::{self, Herd};
use crate::oven::{Unit, species_raw};
use crate::species::Species;

/// The most hazards a fight can have at once: the Mireback's sixteen pools.
pub const MAX_HAZARDS: usize = 16;

/// No anchor: the hazard lies on the floor, its centre in world space.
pub const NO_ANCHOR: u8 = monster::NO_PART;

// ---------------------------------------------------------------------------
// The kind, as a species declares it
// ---------------------------------------------------------------------------

/// What shape a kind is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shape {
    /// A disc on the floor: centre and radius.
    Disc,
    /// A line on the floor -- a web strand: its two ends, and a half-width.
    Strand,
}

/// Who a kind's effects reach.
pub mod reach {
    pub const FIGHTERS: u8 = 1;
    pub const CREATURES: u8 = 2;
    pub const CRITTERS: u8 = 4;
    pub const ALL: u8 = FIGHTERS | CREATURES | CRITTERS;
}

/// One kind of hazard, declared by the species that makes it. Every number
/// about it is a knob on its row ([`HazardField`]); this is what it *is*.
#[derive(Clone, Copy, Debug)]
pub struct HazardDecl {
    /// What the Oven and the overlay call it: "tar", "web", "smoke".
    pub name: &'static str,
    pub shape: Shape,
    /// Who it slows, hurts, pulls and lifts: [`reach`].
    pub reaches: u8,
    /// A creature's glance and `aim::in_view` cannot see through it.
    pub blocks_sight: bool,
    /// Fire touching it turns it into this kind (an index into the species'
    /// kinds), its age starting again: tar into burning tar.
    pub ignites: Option<u8>,
    /// It is fire: it lights every kind that `ignites` and touches it, every
    /// `Spread` frames, and fire put to it does nothing more.
    pub burns: bool,
    /// When its `Life` runs out it becomes this kind -- burning tar into slag
    /// -- or, `None`, is gone.
    pub becomes: Option<u8>,
    /// It stands as a solid, `Solid` metres a layer, `state` layers high.
    pub solid: bool,
    /// What its top is made of, when it is a solid; what it is drawn as.
    pub material: Material,
}

impl HazardDecl {
    /// A disc that does nothing until its row says so.
    pub const fn disc(name: &'static str) -> HazardDecl {
        HazardDecl {
            name,
            shape: Shape::Disc,
            reaches: reach::FIGHTERS,
            blocks_sight: false,
            ignites: None,
            burns: false,
            becomes: None,
            solid: false,
            material: Material::Ground,
        }
    }

    /// A line on the floor.
    pub const fn strand(name: &'static str) -> HazardDecl {
        let mut d = HazardDecl::disc(name);
        d.shape = Shape::Strand;
        d
    }

    pub const fn reaching(mut self, who: u8) -> HazardDecl {
        self.reaches = who;
        self
    }

    pub const fn blocks_sight(mut self) -> HazardDecl {
        self.blocks_sight = true;
        self
    }

    pub const fn ignites_into(mut self, kind: u8) -> HazardDecl {
        self.ignites = Some(kind);
        self
    }

    pub const fn burning(mut self) -> HazardDecl {
        self.burns = true;
        self
    }

    pub const fn becomes(mut self, kind: u8) -> HazardDecl {
        self.becomes = Some(kind);
        self
    }

    pub const fn solid(mut self, material: Material) -> HazardDecl {
        self.solid = true;
        self.material = material;
        self
    }

    pub const fn made_of(mut self, material: Material) -> HazardDecl {
        self.material = material;
        self
    }
}

/// One number about a kind of hazard. A row of these per kind, after the
/// species' pack (if any): "Mireback · tar".
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HazardField {
    /// Frames it lasts. Zero: until something ends it.
    Life,
    /// How far above the floor it reaches: a puddle's skin, a cloud's height,
    /// a vent's column.
    Height,
    /// Walk speed standing in it, times.
    Walk,
    /// Dodge distance from inside it, times. The dodge keeps its frames and its
    /// invulnerability; only how far it carries changes.
    Dodge,
    /// Takeoff speed from inside it, times.
    Jump,
    /// Damage a tick.
    Damage,
    /// Frames between ticks. Zero: it does no damage.
    Every,
    /// Speed it drags a body standing in it toward its centre or its line.
    Pull,
    /// Speed it pushes a body inside its column straight up.
    Lift,
    /// Frames between a fire spreading to what it touches. Zero: it does not.
    Spread,
    /// The radius a hazard takes when it becomes this kind. Zero keeps its own.
    Radius,
    /// Height of one layer, for a kind that stands as a solid.
    Solid,
    /// The most layers a solid stacks.
    Layers,
    /// The most of this kind at once; one more and the oldest goes. Zero: no cap.
    Cap,
}

/// How many fields a kind has.
pub const HAZARD_FIELDS: usize = 14;

impl HazardField {
    pub const ALL: &'static [HazardField] = &[
        HazardField::Life,
        HazardField::Height,
        HazardField::Walk,
        HazardField::Dodge,
        HazardField::Jump,
        HazardField::Damage,
        HazardField::Every,
        HazardField::Pull,
        HazardField::Lift,
        HazardField::Spread,
        HazardField::Radius,
        HazardField::Solid,
        HazardField::Layers,
        HazardField::Cap,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            HazardField::Life => "Lasts",
            HazardField::Height => "Reaches above the floor",
            HazardField::Walk => "Walk speed in it (x)",
            HazardField::Dodge => "Dodge distance from it (x)",
            HazardField::Jump => "Jump from it (x)",
            HazardField::Damage => "Damage a tick",
            HazardField::Every => "Frames a tick",
            HazardField::Pull => "Pull toward its middle",
            HazardField::Lift => "Lift",
            HazardField::Spread => "Fire spreads every",
            HazardField::Radius => "Radius, becoming this",
            HazardField::Solid => "Solid, height a layer",
            HazardField::Layers => "Solid, most layers",
            HazardField::Cap => "Most at once",
        }
    }

    pub const fn unit(self) -> Unit {
        match self {
            HazardField::Life | HazardField::Every | HazardField::Spread => Unit::Frames,
            HazardField::Damage | HazardField::Layers | HazardField::Cap => Unit::Int,
            _ => Unit::Fixed,
        }
    }

    /// What a kind starts at before it has ever been baked: nothing at all.
    /// The bottom of every range, except the three multipliers, whose
    /// nothing is one -- an unbaked kind that rooted everybody in it would be
    /// a trap a new creature set without meaning to.
    pub const fn neutral(self) -> i32 {
        match self {
            HazardField::Walk | HazardField::Dodge | HazardField::Jump => Fx::ONE.raw(),
            _ => self.range().0,
        }
    }

    pub const fn range(self) -> (i32, i32) {
        const fn fx(n: i32, d: i32) -> i32 {
            Fx::ratio(n, d).raw()
        }
        match self {
            HazardField::Life => (0, 7200),
            HazardField::Height => (0, fx(12, 1)),
            HazardField::Walk | HazardField::Dodge | HazardField::Jump => (0, fx(1, 1)),
            HazardField::Damage => (0, 400),
            HazardField::Every | HazardField::Spread => (0, 600),
            HazardField::Pull | HazardField::Lift => (0, fx(20, 1)),
            HazardField::Radius => (0, fx(12, 1)),
            HazardField::Solid => (0, fx(6, 1)),
            HazardField::Layers => (0, 8),
            HazardField::Cap => (0, MAX_HAZARDS as i32),
        }
    }
}

/// One of a kind's numbers, live from the Oven.
pub fn stat(sp: &Species, kind: u8, field: HazardField) -> i32 {
    species_raw(sp.id, sp.hazard_index(kind as usize, field))
}

/// One of a kind's numbers, as fixed point.
pub fn stat_fx(sp: &Species, kind: u8, field: HazardField) -> Fx {
    Fx::from_raw(stat(sp, kind, field))
}

// ---------------------------------------------------------------------------
// One hazard, as the snapshot keeps it
// ---------------------------------------------------------------------------

/// One hazard: a cell of the hunt's [`Lore`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Hazard {
    /// Which of the fight species' kinds, plus one. Zero is an empty slot.
    pub kind: u8,
    /// The kind's to mean: a solid's layers; a species' own phase.
    pub state: u8,
    /// [`NO_ANCHOR`], or the creature part it lives on, packed as a rider's
    /// mount is (`monster::mount_of`).
    pub anchor: u8,
    /// In twentieths of a metre: up to 12.75 m.
    pub radius: u8,
    /// Frames since it was made, or since it last changed kind.
    pub age: u16,
    /// Centimetres: its centre, or a strand's first end. In the part's own
    /// frame when it has an anchor.
    pub at: [i16; 3],
    /// Centimetres in the floor plane: a strand's other end.
    pub to: [i16; 2],
}

impl Hazard {
    /// A disc on the floor.
    pub fn disc(kind: u8, at: V3, radius: Fx) -> Hazard {
        Hazard {
            kind: kind + 1,
            state: 0,
            anchor: NO_ANCHOR,
            radius: radius_byte(radius),
            age: 0,
            at: lore::point_cm(at),
            to: [0; 2],
        }
    }

    /// A line on the floor from `a` to `b`, `half_width` either side.
    pub fn strand(kind: u8, a: V3, b: V3, half_width: Fx) -> Hazard {
        Hazard {
            to: [lore::to_cm(b.x), lore::to_cm(b.z)],
            ..Hazard::disc(kind, a, half_width)
        }
    }

    /// A disc that lives on a creature part, at a point in that part's own
    /// frame -- carried by the pose.
    pub fn on_part(kind: u8, slot: usize, part: usize, local: V3, radius: Fx) -> Hazard {
        Hazard {
            anchor: monster::mount_of(slot, part),
            ..Hazard::disc(kind, local, radius)
        }
    }

    pub const fn present(&self) -> bool {
        self.kind != 0
    }

    /// Its kind, as an index into the species' kinds.
    pub const fn index(&self) -> u8 {
        self.kind.saturating_sub(1)
    }

    pub fn radius(&self) -> Fx {
        Fx::ratio(self.radius as i32, 20)
    }

    pub fn centre(&self) -> V3 {
        lore::cm_point(self.at)
    }

    pub(crate) fn to_cell(self) -> Cell {
        [
            u32::from_le_bytes([self.kind, self.state, self.anchor, self.radius]),
            self.age as u32 | (self.at[0] as u16 as u32) << 16,
            lore::halves(self.at[1], self.at[2]),
            lore::halves(self.to[0], self.to[1]),
        ]
    }

    pub(crate) fn from_cell(c: Cell) -> Hazard {
        let [kind, state, anchor, radius] = c[0].to_le_bytes();
        Hazard {
            kind,
            state,
            anchor,
            radius,
            age: c[1] as u16,
            at: [lore::hi(c[1]), lore::lo(c[2]), lore::hi(c[2])],
            to: [lore::lo(c[3]), lore::hi(c[3])],
        }
    }
}

/// A radius as the cell keeps it, rounded to the nearest five centimetres.
pub fn radius_byte(r: Fx) -> u8 {
    ((r.raw() as i64 * 20 + (1 << 15)) >> 16).clamp(0, u8::MAX as i64) as u8
}

/// Hazard `i`, empty if there is no such slot.
pub fn get(lore: &Lore, i: usize) -> Hazard {
    lore.hazard_cells()
        .get(i)
        .map_or(Hazard::default(), |c| Hazard::from_cell(*c))
}

/// Write hazard `i`. A slot the layout does not have is dropped.
pub fn set(lore: &mut Lore, i: usize, h: Hazard) {
    if let Some(c) = lore.hazard_cells_mut().get_mut(i) {
        *c = h.to_cell();
    }
}

/// Every hazard in the fight, with its slot.
pub fn all(lore: &Lore) -> impl Iterator<Item = (usize, Hazard)> + '_ {
    lore.hazard_cells()
        .iter()
        .enumerate()
        .map(|(i, c)| (i, Hazard::from_cell(*c)))
        .filter(|(_, h)| h.present())
}

/// How many hazards the fight's layout has room for.
pub fn room(lore: &Lore) -> usize {
    lore.hazard_cells().len()
}

/// **Put a hazard down**: the first free slot, or -- the list full, or its
/// kind at its cap -- the oldest of its own kind. Returns the slot, or `None`
/// if there is no room and nothing of its kind to replace. What a species
/// does instead (merge a pool into the nearest, say) is its own to write.
pub fn place(lore: &mut Lore, h: Hazard) -> Option<usize> {
    let sp = lore.owner.map(|s| s.get())?;
    if h.index() as usize >= sp.fight.hazards.len() {
        return None;
    }
    let cap = stat(sp, h.index(), HazardField::Cap).max(0) as usize;
    let same = all(lore).filter(|(_, o)| o.kind == h.kind).count();
    let oldest_of_kind = all(lore)
        .filter(|(_, o)| o.kind == h.kind)
        .max_by_key(|(i, o)| (o.age, usize::MAX - i))
        .map(|(i, _)| i);
    let free = (0..room(lore)).find(|i| !get(lore, *i).present());
    let slot = if cap > 0 && same >= cap {
        oldest_of_kind
    } else {
        free.or(oldest_of_kind)
    }?;
    set(lore, slot, h);
    Some(slot)
}

/// Clear a slot.
pub fn clear(lore: &mut Lore, i: usize) {
    set(lore, i, Hazard::default());
}

// ---------------------------------------------------------------------------
// The floor this frame
// ---------------------------------------------------------------------------

/// One hazard placed in the world, with everything about its kind a reader
/// needs. What the renderer draws and what every test below reads.
#[derive(Clone, Copy, Debug)]
pub struct Placed {
    pub slot: u8,
    /// Index into the species' kinds.
    pub kind: u8,
    pub decl: &'static HazardDecl,
    pub state: u8,
    pub age: u16,
    /// Its centre, or a strand's first end, in the world.
    pub a: V3,
    /// A strand's other end; the centre again for a disc.
    pub b: V3,
    /// The disc's radius, or the strand's half-width.
    pub radius: Fx,
    /// How far above `a.y` it reaches.
    pub height: Fx,
}

impl Placed {
    /// Is the floor point under `p` inside it (in the plane only)?
    pub fn covers_flat(&self, p: V3) -> bool {
        self.flat_gap(p).raw() <= self.radius.raw()
    }

    /// Floor-plane distance from `p` to its centre, or to its line.
    pub fn flat_gap(&self, p: V3) -> Fx {
        match self.decl.shape {
            Shape::Disc => math::wide_flat_dist(p, self.a),
            Shape::Strand => math::flat_segment_gap(p, self.a, self.b),
        }
    }

    /// Is `p` inside it: over it, no lower than its floor and no higher than
    /// it reaches?
    pub fn holds(&self, p: V3) -> bool {
        let skin = crate::arena::SKIN;
        p.y.raw() >= self.a.y.sub(skin).raw()
            && p.y.raw() <= self.a.y.add(self.height).add(skin).raw()
            && self.covers_flat(p)
    }

    /// The point of its middle nearest `p`: its centre, or the nearest point
    /// of its line. What a shape is measured to.
    pub fn nearest_to(&self, p: V3) -> V3 {
        self.pull_point(p)
    }

    /// The point it pulls toward, from `p`: its centre, or the nearest point
    /// of its line.
    pub fn pull_point(&self, p: V3) -> V3 {
        match self.decl.shape {
            Shape::Disc => self.a,
            Shape::Strand => {
                let d = self.b.sub(self.a);
                let dd = d.dot(d);
                if dd.raw() <= 0 {
                    return self.a;
                }
                let s = p.sub(self.a).dot(d).div(dd).clamp(Fx::ZERO, Fx::ONE);
                self.a.add(d.scale(s))
            }
        }
    }

    /// The box it stands as, if its kind is a solid and it has a layer.
    pub fn solid(&self, sp: &Species) -> Option<Solid> {
        if !self.decl.solid || self.state == 0 {
            return None;
        }
        let tall = stat_fx(sp, self.kind, HazardField::Solid).mul(Fx::from_int(self.state as i32));
        if tall.raw() <= 0 {
            return None;
        }
        let r = self.radius;
        Some(Solid {
            min: V3::new(self.a.x.sub(r), self.a.y, self.a.z.sub(r)),
            max: V3::new(self.a.x.add(r), self.a.y.add(tall), self.a.z.add(r)),
            material: self.decl.material,
        })
    }
}

/// What standing somewhere does to a body: the hazards under it, combined.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Underfoot {
    pub walk: Fx,
    pub dodge: Fx,
    pub jump: Fx,
    /// Damage due this frame.
    pub damage: i32,
    /// Where it is being dragged, and how fast; zero speed for none.
    pub pull: (V3, Fx),
    /// The strongest lift.
    pub lift: Fx,
}

impl Underfoot {
    /// Nothing underfoot.
    pub const CLEAR: Underfoot = Underfoot {
        walk: Fx::ONE,
        dodge: Fx::ONE,
        jump: Fx::ONE,
        damage: 0,
        pull: (V3::ZERO, Fx::ZERO),
        lift: Fx::ZERO,
    };

    /// All three of walk, dodge and jump stopped.
    pub fn rooted(&self) -> bool {
        self.walk.raw() == 0 && self.dodge.raw() == 0 && self.jump.raw() == 0
    }
}

/// The hazards of a fight this frame, each placed in the world.
#[derive(Clone, Copy, Debug)]
pub struct Floor {
    items: [Option<Placed>; MAX_HAZARDS],
    /// The fight's species, for the knobs.
    species: Option<&'static Species>,
}

impl Floor {
    /// No hazards: versus, the Ridgeback, anywhere before one is placed.
    pub const NONE: Floor = Floor {
        items: [None; MAX_HAZARDS],
        species: None,
    };

    /// The list, placed: every anchored one carried through its creature's
    /// pose, which is why this is built once a frame rather than per question.
    pub fn build(lore: &Lore, herd: &Herd) -> Floor {
        let Some(sp) = lore.owner.map(|s| s.get()) else {
            return Floor::NONE;
        };
        if sp.fight.hazards.is_empty() {
            return Floor::NONE;
        }
        let mut f = Floor {
            items: [None; MAX_HAZARDS],
            species: Some(sp),
        };
        // A creature's rig, posed once for every hazard it carries: a shell
        // with half a dozen vents on its plates is one forward pass, not six.
        let mut rigs: [Option<crate::beast::Rig>; monster::MAX_MONSTERS] =
            [None; monster::MAX_MONSTERS];
        for (slot, h) in all(lore) {
            let Some(decl) = sp.fight.hazards.get(h.index() as usize) else {
                continue;
            };
            let flat_end = V3::new(
                lore::from_cm(h.to[0]),
                lore::from_cm(h.at[1]),
                lore::from_cm(h.to[1]),
            );
            let (a, b) = if h.anchor == NO_ANCHOR {
                let b = if decl.shape == Shape::Strand {
                    flat_end
                } else {
                    h.centre()
                };
                (h.centre(), b)
            } else {
                let at = monster::mount_slot(h.anchor);
                let part = monster::mount_part(h.anchor);
                let Some(beast) = herd.get(at).and_then(|m| m.as_ref()) else {
                    continue;
                };
                if part >= beast.sp().parts.len() {
                    continue;
                }
                let rig = rigs[at].get_or_insert_with(|| beast.rig());
                let a = rig.part_to_world(part, h.centre());
                let b = if decl.shape == Shape::Strand {
                    rig.part_to_world(part, flat_end)
                } else {
                    a
                };
                (a, b)
            };
            if slot < MAX_HAZARDS {
                f.items[slot] = Some(Placed {
                    slot: slot as u8,
                    kind: h.index(),
                    decl,
                    state: h.state,
                    age: h.age,
                    a,
                    b,
                    radius: h.radius(),
                    height: stat_fx(sp, h.index(), HazardField::Height),
                });
            }
        }
        f
    }

    /// Every hazard there is, placed.
    pub fn iter(&self) -> impl Iterator<Item = &Placed> {
        self.items.iter().flatten()
    }

    pub fn is_empty(&self) -> bool {
        self.items.iter().all(Option::is_none)
    }

    /// The fight's species, whose knobs these are.
    pub fn species(&self) -> Option<&'static Species> {
        self.species
    }

    /// What the hazards holding the point `p` do to a body of kind `who`
    /// ([`reach`]) standing there, on frame `frame`.
    ///
    /// `grounded` is whether the body's feet are on something: a slow, a pull
    /// and a root are things the floor does to feet, and an airborne fighter
    /// keeps the speed it took off with. Damage and lift reach whoever is
    /// inside the column.
    pub fn underfoot(&self, p: V3, who: u8, grounded: bool, frame: u32) -> Underfoot {
        let Some(sp) = self.species else {
            return Underfoot::CLEAR;
        };
        let mut u = Underfoot::CLEAR;
        for h in self.iter() {
            if h.decl.reaches & who == 0 || !h.holds(p) {
                continue;
            }
            let k = h.kind;
            if grounded {
                u.walk = u.walk.min(stat_fx(sp, k, HazardField::Walk));
                u.dodge = u.dodge.min(stat_fx(sp, k, HazardField::Dodge));
                u.jump = u.jump.min(stat_fx(sp, k, HazardField::Jump));
                let pull = stat_fx(sp, k, HazardField::Pull);
                if pull.raw() > u.pull.1.raw() {
                    u.pull = (h.pull_point(p), pull);
                }
            }
            let every = stat(sp, k, HazardField::Every);
            if every > 0 && (frame.wrapping_add(h.slot as u32)) % every as u32 == 0 {
                u.damage += stat(sp, k, HazardField::Damage).max(0);
            }
            u.lift = u.lift.max(stat_fx(sp, k, HazardField::Lift));
        }
        u
    }

    /// Does the line from `from` to `to` pass through anything that blocks
    /// sight? Asked through `aim` -- `aim::sight_clear` -- which owns every
    /// question about what a line meets.
    pub(crate) fn sight_blockers(&self) -> impl Iterator<Item = &Placed> {
        self.iter().filter(|h| h.decl.blocks_sight)
    }

    /// The boxes the solid hazards stand as, at most `N`.
    pub fn solids<const N: usize>(&self) -> ([Solid; N], usize) {
        let mut out = [Solid::new(V3::ZERO, V3::ZERO); N];
        let mut n = 0;
        if let Some(sp) = self.species {
            for h in self.iter() {
                if n >= N {
                    break;
                }
                if let Some(s) = h.solid(sp) {
                    out[n] = s;
                    n += 1;
                }
            }
        }
        (out, n)
    }
}

// ---------------------------------------------------------------------------
// The list, a frame on
// ---------------------------------------------------------------------------

/// Fire somewhere in the world this frame: a point and how far it reaches.
#[derive(Clone, Copy, Debug)]
pub struct Fire {
    pub at: V3,
    pub radius: Fx,
}

/// **One frame of the hazard list**: fire put to it, fire spreading along it,
/// and each hazard aging -- and, at the end of its life, becoming what its kind
/// becomes or going. `fires` is every source of fire outside the list
/// (`World::fires`).
pub fn step(lore: &mut Lore, herd: &Herd, fires: &[Fire]) {
    let Some(sp) = lore.owner.map(|s| s.get()) else {
        return;
    };
    let kinds = sp.fight.hazards;
    if kinds.is_empty() || room(lore) == 0 {
        return;
    }
    let floor = Floor::build(lore, herd);

    // Fire put to it, and fire spreading: decided against the list as it stood
    // at the top of the frame, so the order slots are visited in cannot carry
    // a flame further than one hop a frame.
    let mut lit = [false; MAX_HAZARDS];
    for h in floor.iter() {
        if h.decl.ignites.is_none() {
            continue;
        }
        let touched_by = |at: V3, r: Fx| h.flat_gap(at).raw() <= h.radius.add(r).raw();
        let outside = fires.iter().any(|f| touched_by(f.at, f.radius));
        // A fire spreads **once it has burned for its `Spread`**, and every
        // `Spread` after: a field of joined pools burns outward a step at a
        // time, at a rate a person can watch and outrun. (It used to spread on
        // its first frame too, so a joined field went up a pool a frame.)
        // What it touches is measured shape to shape -- a burning strand,
        // the Mireback's coals, lights tar along its whole line.
        let spread = floor.iter().any(|o| {
            if !o.decl.burns || o.slot == h.slot {
                return false;
            }
            let every = stat(sp, o.kind, HazardField::Spread);
            every > 0
                && o.age > 0
                && o.age as i32 % every == 0
                && h.flat_gap(o.nearest_to(h.a)).raw() <= h.radius.add(o.radius).raw()
        });
        if outside || spread {
            lit[h.slot as usize] = true;
        }
    }

    for (i, &lit) in lit.iter().enumerate().take(room(lore)) {
        let mut h = get(lore, i);
        if !h.present() {
            continue;
        }
        let Some(decl) = kinds.get(h.index() as usize) else {
            clear(lore, i);
            continue;
        };
        if lit {
            if let Some(into) = decl.ignites {
                h = changed(sp, h, into);
                set(lore, i, h);
                continue;
            }
        }
        h.age = h.age.saturating_add(1);
        let life = stat(sp, h.index(), HazardField::Life);
        if life > 0 && h.age as i32 >= life {
            match decl.becomes {
                Some(into) => become_kind(lore, sp, i, h, into),
                None => clear(lore, i),
            }
            continue;
        }
        set(lore, i, h);
    }
}

/// **Put fire to one hazard**, by a species' own rule rather than by a fire
/// touching it: the Mireback's belch lighting every pool it marked. A kind
/// that `ignites` becomes what it ignites into, exactly as if a flame had
/// reached it; anything else is left alone. Returns whether it caught.
pub fn ignite(lore: &mut Lore, i: usize) -> bool {
    let Some(sp) = lore.owner.map(|s| s.get()) else {
        return false;
    };
    let h = get(lore, i);
    if !h.present() {
        return false;
    }
    match sp
        .fight
        .hazards
        .get(h.index() as usize)
        .and_then(|d| d.ignites)
    {
        Some(into) => {
            set(lore, i, changed(sp, h, into));
            true
        }
        None => false,
    }
}

/// The same hazard, as another kind: its age starts again, and it takes the
/// new kind's radius if that kind has one.
fn changed(sp: &Species, h: Hazard, into: u8) -> Hazard {
    let r = stat_fx(sp, into, HazardField::Radius);
    Hazard {
        kind: into + 1,
        age: 0,
        state: 0,
        radius: if r.raw() > 0 {
            radius_byte(r)
        } else {
            h.radius
        },
        ..h
    }
}

/// Hazard `i` becomes kind `into`. A solid kind becoming where one of its kind
/// already stands adds a layer to that one instead, up to its `Layers`: tar
/// burnt on top of slag builds the step higher.
fn become_kind(lore: &mut Lore, sp: &Species, i: usize, h: Hazard, into: u8) {
    let Some(decl) = sp.fight.hazards.get(into as usize) else {
        clear(lore, i);
        return;
    };
    let mut next = changed(sp, h, into);
    if decl.solid {
        let most = stat(sp, into, HazardField::Layers).max(1) as u8;
        let here = next.centre();
        let under = all(lore).find(|(j, o)| {
            *j != i
                && o.kind == next.kind
                && o.anchor == next.anchor
                && math::wide_flat_dist(o.centre(), here).raw() <= o.radius().raw()
        });
        if let Some((j, mut o)) = under {
            o.state = o.state.saturating_add(1).min(most);
            set(lore, j, o);
            clear(lore, i);
            return;
        }
        next.state = 1;
    }
    // A kind with a cap: the oldest of it goes to make room.
    let cap = stat(sp, into, HazardField::Cap).max(0) as usize;
    if cap > 0 {
        let others: usize = all(lore)
            .filter(|(j, o)| *j != i && o.kind == next.kind)
            .count();
        if others >= cap {
            if let Some((j, _)) = all(lore)
                .filter(|(j, o)| *j != i && o.kind == next.kind)
                .max_by_key(|(j, o)| (o.age, usize::MAX - j))
            {
                clear(lore, j);
            }
        }
    }
    set(lore, i, next);
}
