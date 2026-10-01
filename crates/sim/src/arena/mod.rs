//! Arenas, as data: where a fight happens and what it collides against.
//!
//! An [`Arena`] is a table, the way a [`crate::species::Species`] is: its
//! bounds, its solids (axis-aligned boxes, standable on top, any height,
//! hanging from a roof included), what its floor is made of where, and where
//! everybody stands when the fight starts. The world holds one byte of it, an
//! [`ArenaId`], in the snapshot -- so which arena is loaded changes on the same
//! frame for both peers and rolls back with everything else -- and reads the
//! rest through [`ArenaId::get`].
//!
//! The collision is not a general physics engine, and deliberately so:
//! capsule-versus-box and capsule-versus-capsule is all an arena fighter needs,
//! and writing it here is far less work than establishing cross-platform
//! determinism in someone else's solver.
//!
//! ## Why the registry has a line for every creature already
//!
//! Each creature after the Ridgeback fights in an arena of its own, and they
//! are built two at a time on separate branches. Every one of them has an id
//! below and a commented-out line in [`lookup`] and in the module list, one
//! blank line apart, exactly as `species/mod.rs` does it: a branch uncomments
//! its own two lines, and two branches doing that for two creatures merge
//! without conflicting. The recipe is `docs/design/arenas.md`.
//!
//! ## A ceiling is a solid
//!
//! A cave's vault is a box hanging from the roof: its underside is the ceiling.
//! Bodies are pushed down out from under it by the same resolve that lands them
//! on a platform, the crosshair's ray meets it the way it meets a wall, the
//! camera's arm is pulled in under it the way it is pulled in from a wall, and
//! [`Arena::ground_under`] does not count it as ground beneath anything below
//! it. There is no second mechanism to keep in step with the first.

use crate::fixed::Fx;
use crate::math::V3;
use crate::species::SpeciesId;
use crate::tuning as t;

// The body is a vertical cylinder -- capsule-ish is close enough when nothing
// rolls or ragdolls -- and its size lives in the Oven. It used to be a `const`
// here as well as a knob there, which meant tuning the body changed what
// attacks could reach but not what walls could stop.

// ---------------------------------------------------------------------------
// The registry
// ---------------------------------------------------------------------------

// One line per arena, each followed by a blank line. See the module docs for
// why every planned creature already has one.

pub mod proving_ground;

pub mod range;

pub mod gnawers;

// pub mod hornback;

// pub mod mireback;

// pub mod sandmaw;

// pub mod pair;

// pub mod broodmother;

// pub mod veilstalker;

// pub mod mantis;

// pub mod galewing;

// pub mod siegeshell;

/// Which arena. The one byte of arena the world keeps in the snapshot.
///
/// The creature arenas share their creature's number, so nobody has to choose
/// one; the dev arenas sit after them.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub struct ArenaId(pub u8);

impl ArenaId {
    /// The first arena: 28 m square, low walls, two platforms. Versus, the
    /// training dummy and the Ridgeback fight here.
    pub const PROVING_GROUND: ArenaId = ArenaId(0);
    pub const GNAWERS: ArenaId = ArenaId(1);
    pub const HORNBACK: ArenaId = ArenaId(2);
    pub const MIREBACK: ArenaId = ArenaId(3);
    pub const SANDMAW: ArenaId = ArenaId(4);
    pub const PAIR: ArenaId = ArenaId(5);
    pub const BROODMOTHER: ArenaId = ArenaId(6);
    pub const VEILSTALKER: ArenaId = ArenaId(7);
    pub const MANTIS: ArenaId = ArenaId(8);
    pub const GALEWING: ArenaId = ArenaId(9);
    pub const SIEGESHELL: ArenaId = ArenaId(10);
    /// A dev arena with one of everything an arena can have, at the largest
    /// size any creature asks for: see [`range`].
    pub const RANGE: ArenaId = ArenaId(11);

    /// The table. Every registered id has one; asking for an unregistered one
    /// gets the proving ground rather than a crash in the middle of a rollback.
    pub fn get(self) -> &'static Arena {
        lookup(self).unwrap_or(&proving_ground::ARENA)
    }
}

/// How many ids there are, registered or not.
pub const COUNT: usize = 12;

/// The most solids an arena may have. Every query walks all of them, several
/// times a frame per body, so this is what keeps a large arena inside the
/// frame budget; `tests/arena.rs` checks every registered arena against it and
/// `tests/budget.rs` measures the largest.
pub const MAX_SOLIDS: usize = 64;

/// The most floor regions an arena may have, for the same reason.
pub const MAX_REGIONS: usize = 32;

/// The arena registered under an id, if one is.
pub const fn lookup(id: ArenaId) -> Option<&'static Arena> {
    match id {
        ArenaId::PROVING_GROUND => Some(&proving_ground::ARENA),

        ArenaId::RANGE => Some(&range::ARENA),

        ArenaId::GNAWERS => Some(&gnawers::ARENA),

        // ArenaId::HORNBACK => Some(&hornback::ARENA),

        // ArenaId::MIREBACK => Some(&mireback::ARENA),

        // ArenaId::SANDMAW => Some(&sandmaw::ARENA),

        // ArenaId::PAIR => Some(&pair::ARENA),

        // ArenaId::BROODMOTHER => Some(&broodmother::ARENA),

        // ArenaId::VEILSTALKER => Some(&veilstalker::ARENA),

        // ArenaId::MANTIS => Some(&mantis::ARENA),

        // ArenaId::GALEWING => Some(&galewing::ARENA),

        // ArenaId::SIEGESHELL => Some(&siegeshell::ARENA),
        _ => None,
    }
}

/// Every registered arena, in id order.
pub fn all() -> impl Iterator<Item = &'static Arena> {
    (0..COUNT as u8).filter_map(|i| lookup(ArenaId(i)))
}

/// Find a registered arena by name or slug, ignoring case.
pub fn named(name: &str) -> Option<&'static Arena> {
    let wanted = name.to_lowercase().replace(['-', ' '], "_");
    all().find(|a| a.slug() == wanted)
}

/// Where a creature is fought: the arena that names it, or the proving ground
/// for a creature that has none yet. A creature is built before its arena is,
/// and until then it is hunted where the Ridgeback is.
pub fn for_species(species: SpeciesId) -> &'static Arena {
    all()
        .find(|a| a.creature == Some(species))
        .unwrap_or(&proving_ground::ARENA)
}

// ---------------------------------------------------------------------------
// The table
// ---------------------------------------------------------------------------

/// What a surface is made of.
///
/// Data the simulation can ask about -- the Sandmaw swims under sand and not
/// rock, the Veilstalker's floor keeps footprints -- and what the renderer
/// colours the floor by. Nothing reads it yet but the renderer; the questions
/// below are each a first guess for the creature that asks them.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Material {
    /// Packed earth. The proving ground's floor.
    #[default]
    Ground,
    Grass,
    /// Bare rock: boulders, cliffs, the Sandmaw's islands.
    Rock,
    /// Dressed stone: walls, plinths, platforms.
    Stone,
    Sand,
    Snow,
    Ash,
    /// Marsh floor.
    Peat,
    /// Shallow water: a ford, a stream.
    Water,
    /// Timber: trunks, cordwood, a fallen tree.
    Wood,
}

impl Material {
    /// Can a burrower move under it? Sand, and nothing else: a rock island is
    /// where the Sandmaw cannot reach you.
    pub const fn soft(self) -> bool {
        matches!(self, Material::Sand)
    }

    /// Does it keep a footprint? The Veilstalker's snow and ash, and sand.
    pub const fn takes_prints(self) -> bool {
        matches!(self, Material::Snow | Material::Ash | Material::Sand)
    }
}

/// An axis-aligned box of the arena's geometry.
///
/// Every face stops a body; the top is a surface you can stand on, however
/// high it is; and one that hangs from the roof (its bottom above the floor)
/// is a ceiling over whatever is under it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Solid {
    pub min: V3,
    pub max: V3,
    /// What its top is made of, for [`Arena::material_under`] and the renderer.
    pub material: Material,
}

impl Solid {
    pub const fn new(min: V3, max: V3) -> Solid {
        Solid {
            min,
            max,
            material: Material::Stone,
        }
    }

    /// A box in centimetres, its two opposite corners. Centimetres because an
    /// arena is drawn on a plan in metres to a couple of decimals, and integers
    /// keep a table of forty boxes readable.
    pub const fn cm(min: [i32; 3], max: [i32; 3], material: Material) -> Solid {
        Solid {
            min: V3::new(cm(min[0]), cm(min[1]), cm(min[2])),
            max: V3::new(cm(max[0]), cm(max[1]), cm(max[2])),
            material,
        }
    }

    /// Does its footprint cover this point, grown by `pad` all round?
    fn over(&self, x: Fx, z: Fx, pad: Fx) -> bool {
        x.raw() > self.min.x.sub(pad).raw()
            && x.raw() < self.max.x.add(pad).raw()
            && z.raw() > self.min.z.sub(pad).raw()
            && z.raw() < self.max.z.add(pad).raw()
    }

    /// Does it hang from the roof -- its bottom above the floor?
    pub const fn hangs(&self) -> bool {
        self.min.y.raw() > 0
    }
}

/// Centimetres, as fixed point.
pub const fn cm(v: i32) -> Fx {
    Fx::ratio(v, 100)
}

/// A patch of floor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Area {
    /// A rectangle, two opposite corners in the floor plane (x, z).
    Rect { lo: (Fx, Fx), hi: (Fx, Fx) },
    /// A disc: its centre (x, z) and radius.
    Disc { at: (Fx, Fx), radius: Fx },
}

impl Area {
    /// A rectangle in centimetres: x from `x.0` to `x.1`, z from `z.0` to `z.1`.
    pub const fn rect_cm(x: (i32, i32), z: (i32, i32)) -> Area {
        Area::Rect {
            lo: (cm(x.0), cm(z.0)),
            hi: (cm(x.1), cm(z.1)),
        }
    }

    /// A disc in centimetres.
    pub const fn disc_cm(x: i32, z: i32, radius: i32) -> Area {
        Area::Disc {
            at: (cm(x), cm(z)),
            radius: cm(radius),
        }
    }

    /// Is the floor point (x, z) in it? Edges count.
    ///
    /// The disc squares in 64 bits: a 240 m arena puts points further apart
    /// than 16.16 can square.
    pub fn contains(&self, x: Fx, z: Fx) -> bool {
        match *self {
            Area::Rect { lo, hi } => {
                x.raw() >= lo.0.raw()
                    && x.raw() <= hi.0.raw()
                    && z.raw() >= lo.1.raw()
                    && z.raw() <= hi.1.raw()
            }
            Area::Disc { at, radius } => {
                let dx = x.raw() as i64 - at.0.raw() as i64;
                let dz = z.raw() as i64 - at.1.raw() as i64;
                let r = radius.raw() as i64;
                dx * dx + dz * dz <= r * r
            }
        }
    }
}

/// A patch of floor made of something other than the arena's own floor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub area: Area,
    pub material: Material,
}

/// Where a body stands when a fight starts, and which way it faces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mark {
    pub at: V3,
    /// A unit vector in the floor plane.
    pub facing: V3,
}

impl Mark {
    /// Standing on the floor at (x, z) centimetres, facing the floor point
    /// (tx, tz). Worked out at compile time, so the table says what it means
    /// -- "face the creature" -- rather than an angle somebody computed.
    pub const fn cm(x: i32, z: i32, toward: (i32, i32)) -> Mark {
        let at = V3::new(cm(x), Fx::ZERO, cm(z));
        let to = V3::new(cm(toward.0), Fx::ZERO, cm(toward.1));
        Mark {
            at,
            facing: to.sub(at).normalized(),
        }
    }
}

/// Where everybody stands when a round starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spawns {
    /// The two fighters in a versus match, player one first.
    pub versus: [Mark; 2],
    /// The hunters and the creatures in a hunt, or `None` for the species'
    /// own spawn distances along the x axis -- `Common::HunterSpawn` and
    /// `Common::MonsterSpawn`, tuned in the Oven -- which is how the proving
    /// ground has always placed a hunt.
    pub hunt: Option<HuntMarks>,
}

/// A hunt's marks: two hunters, and a creature per slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HuntMarks {
    pub hunters: [Mark; 2],
    pub creatures: [Mark; 2],
}

/// The playable rectangle, in the floor plane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub lo_x: Fx,
    pub hi_x: Fx,
    pub lo_z: Fx,
    pub hi_z: Fx,
}

impl Bounds {
    /// From centimetres: x from `x.0` to `x.1`, z from `z.0` to `z.1`.
    pub const fn cm(x: (i32, i32), z: (i32, i32)) -> Bounds {
        Bounds {
            lo_x: cm(x.0),
            hi_x: cm(x.1),
            lo_z: cm(z.0),
            hi_z: cm(z.1),
        }
    }

    /// Width (x) and depth (z), in metres.
    pub fn size(&self) -> (Fx, Fx) {
        (self.hi_x.sub(self.lo_x), self.hi_z.sub(self.lo_z))
    }
}

/// One arena.
#[derive(Debug)]
pub struct Arena {
    pub id: ArenaId,
    pub name: &'static str,
    /// The creature this arena is for, if any: what [`for_species`] reads.
    pub creature: Option<SpeciesId>,
    /// Where things are allowed to be. Bodies are kept in by the solids --
    /// author walls, banks or cliffs along the edge -- and this is the bound a
    /// projectile leaves by and a creature is clamped to.
    pub bounds: Bounds,
    /// What the floor is made of where no region says otherwise.
    pub floor: Material,
    /// Patches of other floor, in order: a later one wins where two overlap.
    pub regions: &'static [Region],
    /// The geometry. Resolved one at a time in this order, which is part of
    /// what makes a resolve deterministic.
    pub solids: &'static [Solid],
    pub spawns: Spawns,
    /// Where a fight's defended things stand (bestiary P7): a wall's place, a
    /// cart's road. Geometry, so the arena's; what stands there and what it
    /// is worth is the species' (`objective::ObjectiveDecl`).
    pub sites: &'static [crate::objective::Site],
}

// ---------------------------------------------------------------------------
// The queries
// ---------------------------------------------------------------------------

/// Result of resolving a body against the arena.
#[derive(Clone, Copy, Debug)]
pub struct Resolved {
    pub pos: V3,
    pub vel: V3,
    pub grounded: bool,
    /// A solid stopped this body along a horizontal axis this call -- a wall,
    /// or the side of a platform, as opposed to landing on or under one.
    ///
    /// Only the axis driving into the solid is zeroed below, by design (see
    /// `resolve_sized`): a fighter walking diagonally into a wall keeps
    /// sliding along it, which is the feel every wall in the game has always
    /// had. A stone caller wants to know a hard contact happened at all, so it
    /// can decide what happens to the speed that is *not* zeroed -- see
    /// `stones::step`.
    pub wall: bool,
}

/// How far a body is held off a surface before it counts as standing on it.
///
/// Shared with [`crate::stones`], which needs the same tolerance: a fighter
/// resolved exactly on to a surface has nothing left to collide with next
/// frame, so without a skin they read as airborne every other frame.
pub(crate) const SKIN: Fx = Fx::ratio(1, 32);

impl Arena {
    /// The solids, at most [`MAX_SOLIDS`] of them.
    pub fn solids(&self) -> &'static [Solid] {
        &self.solids[..self.solids.len().min(MAX_SOLIDS)]
    }

    /// A name for flags and URLs: lower case, underscores.
    pub fn slug(&self) -> String {
        self.name.to_lowercase().replace([' ', '-'], "_")
    }

    /// Push a fighter out of the arena geometry.
    pub fn resolve(&self, pos: V3, vel: V3, was_grounded: bool) -> Resolved {
        self.resolve_sized(pos, vel, was_grounded, t::body_radius(), t::body_height())
    }

    /// Push a body of any size out of the arena geometry.
    ///
    /// Resolves along the axis of least penetration, one solid at a time, in a
    /// fixed order. Order matters for determinism, which is why the solids are
    /// a slice in a table rather than anything with unstable iteration.
    ///
    /// Sized rather than fixed to the fighter, because the Elementalist's
    /// stones are solids too and they have to stop at the same walls. A second
    /// copy of this with different numbers in it is a second thing to get
    /// wrong -- and the last time a body size was written twice, attacks and
    /// walls disagreed about how wide a fighter was.
    pub fn resolve_sized(
        &self,
        pos: V3,
        vel: V3,
        was_grounded: bool,
        radius: Fx,
        height: Fx,
    ) -> Resolved {
        resolve_among(&[self.solids()], pos, vel, was_grounded, radius, height)
    }

    /// Height of the highest surface under a point, for spawning, for planting
    /// things on the floor and for the renderer's shadow projection.
    ///
    /// A solid that hangs from the roof above the point is a ceiling, not
    /// ground beneath it. Every solid that stands on the floor counts however
    /// high the point is, which is what lets a spawn ask from anywhere.
    pub fn ground_under(&self, pos: V3) -> Fx {
        ground_among(&[self.solids()], pos)
    }

    /// What the surface under a point is made of: the top of the solid it is
    /// standing on (within the skin), or the floor's region there.
    pub fn material_under(&self, pos: V3) -> Material {
        material_among(self, &[self.solids()], pos)
    }

    /// **The lowest ceiling over a point**: the underside of the lowest solid
    /// hanging from the roof whose footprint covers it and whose underside is
    /// above `above`. What keeps the eye inside a cave (`camera::eye_under`).
    pub fn ceiling_over(&self, x: Fx, z: Fx, above: Fx) -> Option<Fx> {
        let mut best: Option<Fx> = None;
        for s in self.solids().iter() {
            if !s.hangs() || s.min.y.raw() <= above.raw() || !s.over(x, z, Fx::ZERO) {
                continue;
            }
            if best.is_none_or(|b| s.min.y.raw() < b.raw()) {
                best = Some(s.min.y);
            }
        }
        best
    }
}

/// Every solid in an arena: its own, then any raised this frame.
fn each<'a>(lists: &'a [&'a [Solid]]) -> impl Iterator<Item = &'a Solid> + 'a {
    lists.iter().flat_map(|l| l.iter())
}

/// The resolve, against any run of solid lists: the arena's own, then the
/// solids raised in this fight ([`Terrain`]), in that order.
fn resolve_among(
    lists: &[&[Solid]],
    mut pos: V3,
    mut vel: V3,
    was_grounded: bool,
    radius: Fx,
    height: Fx,
) -> Resolved {
    {
        let mut grounded = false;
        let mut wall = false;

        // Ground plane first.
        if pos.y.raw() <= 0 {
            pos.y = Fx::ZERO;
            if vel.y.raw() < 0 {
                vel.y = Fx::ZERO;
            }
            grounded = true;
        }

        for solid in each(lists) {
            // Expand the box by the body radius horizontally, so the body can
            // be treated as a point in X and Z.
            let min_x = solid.min.x.sub(radius);
            let max_x = solid.max.x.add(radius);
            let min_z = solid.min.z.sub(radius);
            let max_z = solid.max.z.add(radius);

            let feet = pos.y;
            let head = pos.y.add(height);

            let inside = pos.x.raw() > min_x.raw()
                && pos.x.raw() < max_x.raw()
                && pos.z.raw() > min_z.raw()
                && pos.z.raw() < max_z.raw()
                && head.raw() > solid.min.y.raw()
                && feet.raw() < solid.max.y.raw();
            if !inside {
                continue;
            }

            // Penetration depth on each axis; resolve the shallowest.
            let px = min_penetration(pos.x, min_x, max_x);
            let pz = min_penetration(pos.z, min_z, max_z);
            let py_up = solid.max.y.sub(feet); // push up onto the top
            let py_down = head.sub(solid.min.y); // push down out from under

            let ax = px.abs();
            let az = pz.abs();
            let ay = py_up.min(py_down).abs();

            if ay.raw() <= ax.raw() && ay.raw() <= az.raw() {
                if py_up.raw() <= py_down.raw() {
                    pos.y = solid.max.y;
                    if vel.y.raw() < 0 {
                        vel.y = Fx::ZERO;
                    }
                    grounded = true;
                } else {
                    pos.y = solid.min.y.sub(height);
                    if vel.y.raw() > 0 {
                        vel.y = Fx::ZERO;
                    }
                }
            } else if ax.raw() <= az.raw() {
                pos.x = pos.x.add(px);
                vel.x = Fx::ZERO;
                wall = true;
            } else {
                pos.z = pos.z.add(pz);
                vel.z = Fx::ZERO;
                wall = true;
            }
        }

        // Standing exactly on a surface reads as grounded even when the
        // resolver did not have to move anything this tick.
        if !grounded && was_grounded && vel.y.raw() <= 0 && supported_among(lists, pos, radius) {
            grounded = true;
        }

        Resolved {
            pos,
            vel,
            grounded,
            wall,
        }
    }
}

fn supported_among(lists: &[&[Solid]], pos: V3, radius: Fx) -> bool {
    if pos.y.abs().raw() <= SKIN.raw() {
        return true;
    }
    each(lists)
        .any(|s| s.over(pos.x, pos.z, radius) && pos.y.sub(s.max.y).abs().raw() <= SKIN.raw())
}

fn ground_among(lists: &[&[Solid]], pos: V3) -> Fx {
    let mut best = Fx::ZERO;
    for s in each(lists) {
        let overhead = s.hangs() && s.min.y.raw() > pos.y.raw();
        if s.over(pos.x, pos.z, Fx::ZERO) && !overhead && s.max.y.raw() > best.raw() {
            best = s.max.y;
        }
    }
    best
}

fn material_among(arena: &Arena, lists: &[&[Solid]], pos: V3) -> Material {
    let ground = ground_among(lists, pos);
    if ground.raw() > 0 && pos.y.sub(ground).abs().raw() <= SKIN.raw() {
        // The last solid in order that tops out here: the arena's own, then
        // the raised ones, searched from the end.
        let mut found = None;
        for s in each(lists) {
            if s.over(pos.x, pos.z, Fx::ZERO) && s.max.y == ground {
                found = Some(s.material);
            }
        }
        if let Some(m) = found {
            return m;
        }
    }
    arena.floor_at(pos.x, pos.z)
}

impl Arena {
    /// What the floor is made of at (x, z), ignoring every solid.
    pub fn floor_at(&self, x: Fx, z: Fx) -> Material {
        self.regions[..self.regions.len().min(MAX_REGIONS)]
            .iter()
            .rev()
            .find(|r| r.area.contains(x, z))
            .map_or(self.floor, |r| r.material)
    }

    /// Is this point still in the arena at all?
    ///
    /// Flat bounds and a ceiling of nothing: anything below the floor or
    /// outside the bounds has left, which is what a projectile needs in order
    /// to stop existing. Bodies never ask -- they are resolved against the
    /// walls instead -- so this is deliberately a *bound* rather than a
    /// collision.
    pub fn inside(&self, pos: V3) -> bool {
        let b = &self.bounds;
        pos.y.raw() >= 0
            && pos.x.raw() >= b.lo_x.raw()
            && pos.x.raw() <= b.hi_x.raw()
            && pos.z.raw() >= b.lo_z.raw()
            && pos.z.raw() <= b.hi_z.raw()
    }

    /// Where the hunt's marks are, if the arena places them itself.
    pub fn hunt_marks(&self) -> Option<&HuntMarks> {
        self.spawns.hunt.as_ref()
    }
}

/// Signed push needed to leave the span by the nearer edge.
fn min_penetration(v: Fx, lo: Fx, hi: Fx) -> Fx {
    let out_lo = lo.sub(v); // negative: push toward lo
    let out_hi = hi.sub(v); // positive: push toward hi
    if out_hi.abs().raw() < out_lo.abs().raw() {
        out_hi
    } else {
        out_lo
    }
}

// ---------------------------------------------------------------------------
// The ground as it stands this frame
// ---------------------------------------------------------------------------

/// The most solids a fight can raise at runtime: the Mireback's six slag
/// mounds and two objectives.
pub const MAX_RAISED: usize = 8;

/// **The ground as it stands this frame**: the arena's table, the solids a
/// fight has raised on it -- slag hardened out of burnt tar, a wall or a cart
/// that is a solid -- and the floor hazards lying on it.
///
/// What every query that used to take an `&Arena` takes now, because a solid
/// that appears in the middle of a fight has to stop bodies, hold up the
/// people standing on it and meet the aiming ray exactly as a table's own
/// solid does, or the overlay rule breaks on the first mound. It **derefs to
/// the [`Arena`]**, so its bounds, regions, name and marks read as before;
/// the queries that walk solids are its own and walk both lists, the arena's
/// first. Built once a frame by `World::terrain`; with nothing raised it
/// answers every question bit for bit as the arena alone does.
#[derive(Clone, Copy, Debug)]
pub struct Terrain {
    pub arena: &'static Arena,
    raised: [Solid; MAX_RAISED],
    raised_len: u8,
    /// The floor hazards, placed: `crate::hazard`.
    pub floor: crate::hazard::Floor,
}

impl std::ops::Deref for Terrain {
    type Target = Arena;
    fn deref(&self) -> &Arena {
        self.arena
    }
}

impl Terrain {
    /// The arena alone: nothing raised, no hazards. What a test or a tool that
    /// holds no `World` stands on.
    pub const fn bare(arena: &'static Arena) -> Terrain {
        Terrain {
            arena,
            raised: [Solid::new(V3::ZERO, V3::ZERO); MAX_RAISED],
            raised_len: 0,
            floor: crate::hazard::Floor::NONE,
        }
    }

    /// The same ground with one more solid raised on it, if there is room.
    pub fn raise(&mut self, s: Solid) {
        if (self.raised_len as usize) < MAX_RAISED {
            self.raised[self.raised_len as usize] = s;
            self.raised_len += 1;
        }
    }

    /// The solids raised this fight.
    pub fn raised(&self) -> &[Solid] {
        &self.raised[..self.raised_len as usize]
    }

    fn lists(&self) -> [&[Solid]; 2] {
        [self.arena.solids(), self.raised()]
    }

    /// Every solid: the arena's, then the raised ones.
    pub fn solids(&self) -> impl Iterator<Item = &Solid> + '_ {
        self.arena.solids().iter().chain(self.raised().iter())
    }

    /// [`Arena::resolve`], against the raised solids too.
    pub fn resolve(&self, pos: V3, vel: V3, was_grounded: bool) -> Resolved {
        self.resolve_sized(pos, vel, was_grounded, t::body_radius(), t::body_height())
    }

    /// [`Arena::resolve_sized`], against the raised solids too.
    pub fn resolve_sized(
        &self,
        pos: V3,
        vel: V3,
        was_grounded: bool,
        radius: Fx,
        height: Fx,
    ) -> Resolved {
        resolve_among(&self.lists(), pos, vel, was_grounded, radius, height)
    }

    /// [`Arena::ground_under`], with the raised solids standing on the floor.
    pub fn ground_under(&self, pos: V3) -> Fx {
        ground_among(&self.lists(), pos)
    }

    /// [`Arena::material_under`], with the raised solids' tops.
    pub fn material_under(&self, pos: V3) -> Material {
        material_among(self.arena, &self.lists(), pos)
    }

    /// **A creature against the solids** (for a species that `collides`): an
    /// upright cylinder of `radius`, pushed out sideways only, along the
    /// shallower axis, of every solid standing on the floor whose top is at
    /// least `step` above its feet. It never stands on a solid and is never
    /// pushed down from under one -- a creature steps over what is low, walks
    /// into what is not, and passes under a vault, whose lintels are a cave's
    /// business to make tall enough.
    ///
    /// Returns where it ends up and the push that got it there, which is zero
    /// when it touched nothing: the species' `bumped` hook reads it.
    pub fn fence(&self, mut pos: V3, radius: Fx, step: Fx) -> (V3, V3) {
        let start = pos;
        for solid in self.solids() {
            if solid.hangs() || solid.max.y.raw() <= pos.y.add(step).raw() {
                continue;
            }
            let (min_x, max_x) = (solid.min.x.sub(radius), solid.max.x.add(radius));
            let (min_z, max_z) = (solid.min.z.sub(radius), solid.max.z.add(radius));
            let inside = pos.x.raw() > min_x.raw()
                && pos.x.raw() < max_x.raw()
                && pos.z.raw() > min_z.raw()
                && pos.z.raw() < max_z.raw();
            if !inside {
                continue;
            }
            let px = min_penetration(pos.x, min_x, max_x);
            let pz = min_penetration(pos.z, min_z, max_z);
            if px.abs().raw() <= pz.abs().raw() {
                pos.x = pos.x.add(px);
            } else {
                pos.z = pos.z.add(pz);
            }
        }
        (pos, pos.sub(start))
    }
}
