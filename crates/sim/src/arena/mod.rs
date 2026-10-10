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

use crate::atlas::Atlas;
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

pub mod relief;

pub mod rim;

pub mod proving_ground;

pub mod range;

pub mod gnawers;

pub mod hornback;

pub mod mireback;

pub mod sandmaw;

pub mod pair;

pub mod broodmother;

pub mod veilstalker;

pub mod mantis;

pub mod galewing;

pub mod siegeshell;

pub mod lab;

pub mod climb;

pub mod waterfall;

pub mod bench;

pub mod hearth;

pub mod ring;

pub mod mouth;

pub mod bank;

pub mod shelves;

pub mod pinewood;

pub mod saddle;

pub mod highlands;

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
    /// The Hornback's second arena: the crossing, its defend variant (P7).
    pub const HORNBACK_CROSSING: ArenaId = ArenaId(12);
    /// A dev arena for measuring movement: see [`lab`] and `crate::envelope`.
    pub const LAB: ArenaId = ArenaId(13);
    /// **The jump courses** (`docs/design/courses.md`): no creature, the
    /// movement system is the challenge. Two or three at each tier, all in [`climb`],
    /// each with its own `crate::course::Course`.
    pub const CLIMB_STAIR: ArenaId = ArenaId(14);
    pub const CLIMB_CAUSEWAY: ArenaId = ArenaId(15);
    pub const CLIMB_SPIRAL: ArenaId = ArenaId(16);
    pub const CLIMB_FALLS: ArenaId = ArenaId(17);
    pub const CLIMB_SPIRE: ArenaId = ArenaId(18);
    pub const CLIMB_GULF: ArenaId = ArenaId(19);
    pub const CLIMB_SLALOM: ArenaId = ArenaId(20);
    pub const CLIMB_FORK: ArenaId = ArenaId(21);
    /// The proving ground of the jump courses: lanes and ledges at marked
    /// distances, one per class mechanic to try.
    pub const CLIMB_REACH: ArenaId = ArenaId(22);
    /// **The valley** (`docs/design/valley.md`, `crate::valley`): the town,
    /// its ring, and the five reaches of the climb.
    pub const HEARTH: ArenaId = ArenaId(23);
    pub const RING: ArenaId = ArenaId(24);
    pub const MOUTH: ArenaId = ArenaId(25);
    pub const BANK: ArenaId = ArenaId(26);
    pub const SHELVES: ArenaId = ArenaId(27);
    pub const PINEWOOD: ArenaId = ArenaId(28);
    pub const SADDLE: ArenaId = ArenaId(29);
    /// A dev arena of single hops, for measuring a kind of hop: see [`bench`].
    pub const BENCH: ArenaId = ArenaId(30);
    /// The Ridgeback's room in the valley: the proving ground's plan on a
    /// moor. The Ridgeback is still hunted in the proving ground by
    /// [`for_species`]; this is only where the valley meets it.
    pub const HIGHLANDS: ArenaId = ArenaId(31);
    /// **The Waterfall** as a jump course of its own (`waterfall`): the
    /// same cliff the valley's road climbs at the head of the Pinewood.
    pub const CLIMB_WATERFALL: ArenaId = ArenaId(32);

    /// The table. Every registered id has one; asking for an unregistered one
    /// gets the proving ground rather than a crash in the middle of a rollback.
    pub fn get(self) -> &'static Arena {
        lookup(self).unwrap_or(&proving_ground::ARENA)
    }
}

/// How many ids there are, registered or not.
pub const COUNT: usize = 40;

/// The most solids an arena **fought on alone** may have. Alone, every query
/// walks all of them, several times a frame per body, so this is what keeps a
/// large arena inside the frame budget; `tests/arena.rs` checks every such
/// arena against it and `tests/budget.rs` measures the largest.
///
/// A place that is only ever part of a map (`crate::atlas`) -- the valley's
/// town and reaches -- is not held to it: there a query reads the few tiles
/// near it, and the map can hold as many boxes as anybody draws.
pub const MAX_SOLIDS: usize = 64;

/// The most floor regions an arena may have, for the same reason.
pub const MAX_REGIONS: usize = 32;

/// The arena registered under an id, if one is.
pub const fn lookup(id: ArenaId) -> Option<&'static Arena> {
    match id {
        ArenaId::PROVING_GROUND => Some(&proving_ground::ARENA),

        ArenaId::RANGE => Some(&range::ARENA),

        ArenaId::GNAWERS => Some(&gnawers::ARENA),

        ArenaId::HORNBACK => Some(&hornback::ARENA),

        ArenaId::HORNBACK_CROSSING => Some(&hornback::CROSSING),

        ArenaId::MIREBACK => Some(&mireback::ARENA),

        ArenaId::SANDMAW => Some(&sandmaw::ARENA),

        ArenaId::PAIR => Some(&pair::ARENA),

        ArenaId::BROODMOTHER => Some(&broodmother::ARENA),

        ArenaId::VEILSTALKER => Some(&veilstalker::ARENA),

        ArenaId::MANTIS => Some(&mantis::ARENA),

        ArenaId::GALEWING => Some(&galewing::ARENA),
        ArenaId::SIEGESHELL => Some(&siegeshell::ARENA),

        ArenaId::LAB => Some(&lab::ARENA),

        ArenaId::BENCH => Some(&bench::ARENA),

        ArenaId::CLIMB_STAIR => Some(&climb::STAIR),
        ArenaId::CLIMB_CAUSEWAY => Some(&climb::CAUSEWAY),
        ArenaId::CLIMB_SPIRAL => Some(&climb::SPIRAL),
        ArenaId::CLIMB_FALLS => Some(&climb::FALLS),
        ArenaId::CLIMB_SLALOM => Some(&climb::SLALOM),
        ArenaId::CLIMB_FORK => Some(&climb::FORK),
        ArenaId::CLIMB_SPIRE => Some(&climb::SPIRE),
        ArenaId::CLIMB_GULF => Some(&climb::GULF),
        ArenaId::CLIMB_REACH => Some(&climb::REACH),
        ArenaId::CLIMB_WATERFALL => Some(&climb::WATERFALL),

        ArenaId::HEARTH => Some(&hearth::ARENA),
        ArenaId::RING => Some(&ring::ARENA),
        ArenaId::MOUTH => Some(&mouth::ARENA),
        ArenaId::BANK => Some(&bank::ARENA),
        ArenaId::SHELVES => Some(&shelves::ARENA),
        ArenaId::PINEWOOD => Some(&pinewood::ARENA),
        ArenaId::SADDLE => Some(&saddle::ARENA),
        ArenaId::HIGHLANDS => Some(&highlands::ARENA),
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
    pub fn over(&self, x: Fx, z: Fx, pad: Fx) -> bool {
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
    /// **The ground behind its edge** ([`rim`]): a bank too steep to climb
    /// rising off the outside of its walls, and hills past it. `None` for
    /// the floor going on flat forever, which a dev arena and a jump course
    /// (whose floor is a drop) keep.
    pub rim: Option<rim::Rim>,
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
    /// The solids. An arena fought on alone is held to [`MAX_SOLIDS`] by
    /// `tests/arena.rs`, because alone every query walks all of them; one that
    /// is only ever a place on a map (`crate::atlas`) is not, because there a
    /// query reads the tiles near it.
    pub fn solids(&self) -> &'static [Solid] {
        self.solids
    }

    /// **The floor's height here**, before anything standing on it: zero on a
    /// flat floor, a hill's on a hill ([`relief`]). Every floor the
    /// simulation finds -- collision, the aiming ray, a creature's fence --
    /// starts from this.
    pub fn relief_at(&self, x: Fx, z: Fx) -> Fx {
        if let Some(h) = self.rim.as_ref().and_then(|r| r.height(x, z)) {
            return h;
        }
        relief::height_at(self.id, x, z)
    }

    /// **Is the floor one plane here**: inside the rim (or with none), the
    /// arena's relief is flat; on the rim, never. What keeps a flat fight
    /// playing exactly as it did before it had a rim.
    pub fn is_flat_at(&self, x: Fx, z: Fx) -> bool {
        relief::is_flat(self.id) && !self.on_rim(x, z)
    }

    /// Is a point on the rim's ground, outside the arena's own floor?
    pub fn on_rim(&self, x: Fx, z: Fx) -> bool {
        self.rim.as_ref().is_some_and(|r| r.outside(x, z))
    }

    /// A name for flags and URLs: lower case, underscores.
    pub fn slug(&self) -> String {
        self.name.to_lowercase().replace([' ', '-'], "_")
    }

    /// The arena alone, as ground: [`Terrain::bare`]. Every query below is
    /// the terrain's, asked of nothing but this table.
    fn alone(&'static self) -> Terrain {
        Terrain::bare(self)
    }

    /// Push a fighter out of the arena geometry.
    pub fn resolve(&'static self, pos: V3, vel: V3, was_grounded: bool) -> Resolved {
        self.alone().resolve(pos, vel, was_grounded)
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
        &'static self,
        pos: V3,
        vel: V3,
        was_grounded: bool,
        radius: Fx,
        height: Fx,
    ) -> Resolved {
        self.alone()
            .resolve_sized(pos, vel, was_grounded, radius, height)
    }

    /// Height of the highest surface under a point, for spawning, for planting
    /// things on the floor and for the renderer's shadow projection.
    ///
    /// A solid that hangs from the roof above the point is a ceiling, not
    /// ground beneath it. Every solid that stands on the floor counts however
    /// high the point is, which is what lets a spawn ask from anywhere.
    pub fn ground_under(&'static self, pos: V3) -> Fx {
        self.alone().ground_under(pos)
    }

    /// What the surface under a point is made of: the top of the solid it is
    /// standing on (within the skin), or the floor's region there.
    pub fn material_under(&'static self, pos: V3) -> Material {
        self.alone().material_under(pos)
    }

    /// **The lowest ceiling near a point, sloped**: [`Terrain::ceiling_near`].
    pub fn ceiling_near(&'static self, x: Fx, z: Fx, above: Fx, slope: Fx) -> Option<Fx> {
        self.alone().ceiling_near(x, z, above, slope)
    }

    /// **The lowest ceiling over a point**: [`Terrain::ceiling_over`].
    pub fn ceiling_over(&'static self, x: Fx, z: Fx, above: Fx) -> Option<Fx> {
        self.alone().ceiling_over(x, z, above)
    }
}

/// How far past a body's own box the resolve looks for solids. A push out of
/// one box can move a body into the next, and that one has to be a candidate
/// too; nothing a frame does pushes further than this.
const RESOLVE_REACH: Fx = Fx::from_int(2);

/// How far round a point a sloped ceiling is looked for. Further than this,
/// a ceiling sloped away by `camera_ceiling_slope` is above anything the eye
/// would be held under.
const CEILING_REACH: Fx = Fx::from_int(16);

/// The resolve, against the ground as it stands: the arena's own solids (or
/// the map's near the body), then the ones raised in this fight, in that
/// order.
fn resolve_among(
    ground: &Terrain,
    mut pos: V3,
    mut vel: V3,
    was_grounded: bool,
    radius: Fx,
    height: Fx,
) -> Resolved {
    {
        let mut grounded = false;
        let mut wall = false;

        // On land, ground too steep to walk up is a wall: the step that
        // would climb it is taken back, or slid along.
        if ground.on_land(pos.x, pos.z) {
            wall |= too_steep_to_climb(ground, &mut pos, &mut vel);
        }

        // The floor first: its relief under this point ([`relief`]), zero on
        // a flat floor.
        let floor = ground.relief_at(pos.x, pos.z);
        if pos.y.raw() <= floor.raw() {
            pos.y = floor;
            if vel.y.raw() < 0 {
                vel.y = Fx::ZERO;
            }
            grounded = true;
        }

        let reach = radius.add(RESOLVE_REACH);
        for solid in ground.around(pos, reach) {
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
                    // **Lifted onto a top is not landing on it while still
                    // rising.** A jump that clips a ledge's near corner on
                    // the way up is pushed up onto the top here -- the
                    // vertical overlap is the shallowest -- and counted as
                    // standing, it let a still-held jump fire a second
                    // takeoff stacked on the first's speed (about 27 m/s up;
                    // `tests/corner.rs`). She is put on the top and keeps
                    // rising; she lands when she stops.
                    if vel.y.raw() <= 0 {
                        grounded = true;
                    }
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

        // On land, ground too steep to stand on is no footing: the body slides
        // down it, and cannot jump from it.
        if grounded && ground.on_land(pos.x, pos.z) && pos.y.raw() <= floor.add(SKIN).raw() {
            if let Some(down) = ground.downhill(pos.x, pos.z) {
                let step = t::terrain_slide().mul(crate::DT);
                pos.x = pos.x.add(down.x.mul(step));
                pos.z = pos.z.add(down.z.mul(step));
                let under = ground.relief_at(pos.x, pos.z);
                if pos.y.raw() < under.raw() {
                    pos.y = under;
                }
                grounded = false;
            }
        }

        // Standing exactly on a surface reads as grounded even when the
        // resolver did not have to move anything this tick.
        if !grounded && was_grounded && vel.y.raw() <= 0 {
            if supported_among(ground, pos, radius) {
                grounded = true;
            } else {
                // **Feet follow a floor that falls away gently.** Walking
                // down a hill puts the feet on the floor below rather than
                // leaving them in the air for gravity to find: without this
                // a body walked down every slope in a stutter of tiny falls.
                // Only onto the open floor, only by `tuning::step_down` --
                // one frame of the steepest slope -- and only where there is
                // relief at all: a flat arena has nothing that falls away
                // gently, and plays exactly as it did before there were
                // hills (the pair's fight changed when this ran everywhere).
                // Anything further down is a drop, and a drop is a fall.
                let floor = ground.relief_at(pos.x, pos.z);
                let under = ground_among(ground, pos);
                let gap = pos.y.sub(under);
                if !ground.is_flat_at(pos.x, pos.z)
                    && under.raw() == floor.raw()
                    && gap.raw() > 0
                    && gap.raw() <= t::step_down().raw()
                {
                    pos.y = under;
                    grounded = true;
                }
            }
        }

        Resolved {
            pos,
            vel,
            grounded,
            wall,
        }
    }
}

/// **Walking up ground too steep to walk up**: a body whose step this frame
/// (`pos` less `vel` a frame) rose faster than `tuning::terrain_steepest`
/// keeps whichever half of the step does not -- along the slope rather than
/// up it -- or neither. Only for a body at or under the floor where it
/// arrived: in the air nothing is climbing anything. True if the step was
/// stopped, which is a wall.
fn too_steep_to_climb(ground: &Terrain, pos: &mut V3, vel: &mut V3) -> bool {
    let arrived = ground.relief_at(pos.x, pos.z);
    if pos.y.raw() > arrived.add(SKIN).raw() {
        return false;
    }
    let was = V3::new(
        pos.x.sub(vel.x.mul(crate::DT)),
        pos.y,
        pos.z.sub(vel.z.mul(crate::DT)),
    );
    let steepest = t::terrain_steepest();
    let start = ground.relief_at(was.x, was.z);
    let climbable = |x: Fx, z: Fx| {
        let run = crate::math::wide_len(V3::new(x.sub(was.x), Fx::ZERO, z.sub(was.z)));
        let rise = ground.relief_at(x, z).sub(start);
        // A hair of slack, so walking along a slope's contour is not stopped
        // by the rounding of one sample against the next.
        rise.raw() <= steepest.mul(run).add(Fx::ratio(1, 50)).raw()
    };
    if climbable(pos.x, pos.z) {
        return false;
    }
    if climbable(pos.x, was.z) {
        pos.z = was.z;
        vel.z = Fx::ZERO;
    } else if climbable(was.x, pos.z) {
        pos.x = was.x;
        vel.x = Fx::ZERO;
    } else {
        pos.x = was.x;
        pos.z = was.z;
        vel.x = Fx::ZERO;
        vel.z = Fx::ZERO;
    }
    true
}

fn supported_among(ground: &Terrain, pos: V3, radius: Fx) -> bool {
    if pos.y.sub(ground.relief_at(pos.x, pos.z)).abs().raw() <= SKIN.raw() {
        return true;
    }
    ground
        .around(pos, radius)
        .any(|s| s.over(pos.x, pos.z, radius) && pos.y.sub(s.max.y).abs().raw() <= SKIN.raw())
}

fn ground_among(ground: &Terrain, pos: V3) -> Fx {
    let mut best = ground.relief_at(pos.x, pos.z);
    for s in ground.around(pos, Fx::ZERO) {
        let overhead = ground.hangs(&s) && s.min.y.raw() > pos.y.raw();
        if s.over(pos.x, pos.z, Fx::ZERO) && !overhead && s.max.y.raw() > best.raw() {
            best = s.max.y;
        }
    }
    best
}

fn material_among(ground: &Terrain, pos: V3) -> Material {
    let top = ground_among(ground, pos);
    let floor = ground.relief_at(pos.x, pos.z);
    if top.raw() > floor.raw() && pos.y.sub(top).abs().raw() <= SKIN.raw() {
        // The last solid in order that tops out here: the arena's own, then
        // the raised ones, searched from the end.
        let mut found = None;
        for s in ground.around(pos, Fx::ZERO) {
            if s.over(pos.x, pos.z, Fx::ZERO) && s.max.y == top {
                found = Some(s.material);
            }
        }
        if let Some(m) = found {
            return m;
        }
    }
    ground.floor_at(pos.x, pos.z)
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
/// mounds and two objectives -- and, in the valley, the waystone doors that
/// are shut.
pub const MAX_RAISED: usize = 16;

/// **The ground as it stands this frame**: the arena's table -- or, in the
/// valley, the whole map around it -- the solids a fight has raised on it
/// (slag hardened out of burnt tar, a wall or a cart that is a solid, a shut
/// door), and the floor hazards lying on it.
///
/// What every query that used to take an `&Arena` takes now, because a solid
/// that appears in the middle of a fight has to stop bodies, hold up the
/// people standing on it and meet the aiming ray exactly as a table's own
/// solid does, or the overlay rule breaks on the first mound. It **derefs to
/// the [`Arena`]**, so its bounds, regions, name and marks read as before;
/// the queries that walk solids are its own and walk both lists, the arena's
/// first. Built once a frame by `World::terrain`; with nothing raised it
/// answers every question bit for bit as the arena alone does.
///
/// **On a map** ([`crate::atlas`]) the arena is the place the world is in,
/// everything is in that place's coordinates as always, and the solids are
/// the map's: a query reads the tiles near it, and every box comes back moved
/// into the place's coordinates. The neighbours are as solid as the place
/// itself, which is what makes the valley one piece.
#[derive(Clone, Copy, Debug)]
pub struct Terrain {
    pub arena: &'static Arena,
    /// The map the arena is a place on, if it is on one.
    atlas: Option<&'static Atlas>,
    /// Where the arena's own origin is on that map.
    origin: V3,
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

/// The solids a query reads, in order: the table's or the map's near it,
/// moved into the place's coordinates, then the raised ones.
pub struct Solids<'a> {
    own: Own<'a>,
    raised: std::slice::Iter<'a, Solid>,
}

// The map's arm carries a few hundred bytes of tile cursors. It lives on the
// stack for one query and is never boxed: a frame does not allocate.
#[allow(clippy::large_enum_variant)]
enum Own<'a> {
    Table(std::slice::Iter<'static, Solid>),
    Map {
        atlas: &'static Atlas,
        near: crate::atlas::Near<'static>,
        origin: V3,
        _ground: std::marker::PhantomData<&'a ()>,
    },
}

impl Iterator for Solids<'_> {
    type Item = Solid;

    fn next(&mut self) -> Option<Solid> {
        let own = match &mut self.own {
            Own::Table(it) => it.next().copied(),
            Own::Map {
                atlas,
                near,
                origin,
                ..
            } => near.next().map(|i| {
                let s = atlas.solids[i as usize];
                Solid {
                    min: s.min.sub(*origin),
                    max: s.max.sub(*origin),
                    material: s.material,
                }
            }),
        };
        own.or_else(|| self.raised.next().copied())
    }
}

impl Terrain {
    /// The arena alone: nothing raised, no hazards. What a test or a tool that
    /// holds no `World` stands on.
    pub const fn bare(arena: &'static Arena) -> Terrain {
        Terrain {
            arena,
            atlas: None,
            origin: V3::ZERO,
            raised: [Solid::new(V3::ZERO, V3::ZERO); MAX_RAISED],
            raised_len: 0,
            floor: crate::hazard::Floor::NONE,
        }
    }

    /// **A place on a map**: the arena as it sits on `atlas`, with every
    /// other place on the map around it. An arena the map does not have is
    /// the arena alone.
    pub fn placed(atlas: &'static Atlas, arena: &'static Arena) -> Terrain {
        match atlas.placed(arena.id) {
            Some(p) => Terrain {
                atlas: Some(atlas),
                origin: p.at,
                ..Terrain::bare(arena)
            },
            None => Terrain::bare(arena),
        }
    }

    /// The map this ground is part of, if any.
    pub fn atlas(&self) -> Option<&'static Atlas> {
        self.atlas
    }

    /// Where the place's own origin is on the map: zero off a map.
    pub fn origin(&self) -> V3 {
        self.origin
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

    /// **The solids near a box** (its x and z; every height): every solid
    /// that overlaps it and possibly some that do not, in order. On a table,
    /// all of them.
    pub fn near(&self, lo: V3, hi: V3) -> Solids<'_> {
        let own = match self.atlas {
            None => Own::Table(self.arena.solids().iter()),
            Some(atlas) => Own::Map {
                atlas,
                near: atlas.near(
                    (lo.x.add(self.origin.x), lo.z.add(self.origin.z)),
                    (hi.x.add(self.origin.x), hi.z.add(self.origin.z)),
                ),
                origin: self.origin,
                _ground: std::marker::PhantomData,
            },
        };
        Solids {
            own,
            raised: self.raised().iter(),
        }
    }

    /// The solids within `reach` of a point, sideways.
    pub fn around(&self, at: V3, reach: Fx) -> Solids<'_> {
        let r = V3::new(reach, Fx::ZERO, reach);
        self.near(at.sub(r), at.add(r))
    }

    /// The solids near a straight line from `from`, `length` along `dir`
    /// (a unit vector), grown by `pad`.
    pub fn along(&self, from: V3, dir: V3, length: Fx, pad: Fx) -> Solids<'_> {
        let to = from.add(dir.scale(length));
        let p = V3::new(pad, Fx::ZERO, pad);
        let lo = V3::new(from.x.min(to.x), Fx::ZERO, from.z.min(to.z)).sub(p);
        let hi = V3::new(from.x.max(to.x), Fx::ZERO, from.z.max(to.z)).add(p);
        self.near(lo, hi)
    }

    /// **Every solid of this place**: the arena's own table and the raised
    /// ones -- or, on a map, every solid near the place's footprint. What
    /// a question about the whole place reads (a creature's planning, a
    /// tool); anything about one body or one line asks [`Terrain::near`].
    pub fn solids(&self) -> Solids<'_> {
        match self.atlas.and_then(|a| a.placed(self.arena.id)) {
            Some(p) => {
                let pad = Fx::from_int(crate::atlas::TILE_M);
                self.near(
                    V3::new(p.lo.0.sub(pad), Fx::ZERO, p.lo.1.sub(pad)).sub(self.origin),
                    V3::new(p.hi.0.add(pad), Fx::ZERO, p.hi.1.add(pad)).sub(self.origin),
                )
            }
            None => self.near(V3::ZERO, V3::ZERO),
        }
    }

    /// **Does a solid hang** -- its bottom above the floor under it? A cave's
    /// vault, a bridge: a ceiling over whatever is under it, and not ground
    /// beneath anything below it. Alone, the floor is at zero
    /// ([`Solid::hangs`]); on a map with land a tree or a crag stands on
    /// ground metres up, and is not a ceiling for being there.
    pub fn hangs(&self, s: &Solid) -> bool {
        match self.atlas {
            None => s.hangs(),
            Some(_) => {
                let x = Fx::from_raw(s.min.x.raw() / 2 + s.max.x.raw() / 2);
                let z = Fx::from_raw(s.min.z.raw() / 2 + s.max.z.raw() / 2);
                s.min.y.raw() > self.relief_at(x, z).raw()
            }
        }
    }

    /// [`Arena::relief_at`]: the floor itself, with nothing on it -- on a
    /// map, whichever place's floor is under the point.
    pub fn relief_at(&self, x: Fx, z: Fx) -> Fx {
        match self.atlas {
            None => self.arena.relief_at(x, z),
            Some(a) => a
                .relief_at(x.add(self.origin.x), z.add(self.origin.z))
                .sub(self.origin.y),
        }
    }

    /// **The lowest a floor can be**, in this place's coordinates: zero for
    /// an arena alone, whose floor is at zero; on a map, its void, which is
    /// under every place on it. What the fall rule measures a drop down to,
    /// so a fall into a place lower than this one costs what it should.
    pub fn lowest(&self) -> Fx {
        match self.atlas {
            None => Fx::ZERO,
            Some(a) => a.void.sub(self.origin.y),
        }
    }

    /// **Is this point on land** -- a map's open ground, not a place's own
    /// floor? Where the steepness of the ground is a wall.
    pub fn on_land(&self, x: Fx, z: Fx) -> bool {
        match self.atlas {
            Some(a) if a.land.is_some() => {
                let (mx, mz) = (x.add(self.origin.x), z.add(self.origin.z));
                a.pad_at(mx, mz).is_none()
            }
            Some(_) => false,
            // Alone, an arena's rim is land.
            None => self.arena.on_rim(x, z),
        }
    }

    /// **Which way is down**, flat, if the land here is too steep to stand
    /// on: a unit vector down the slope. `None` where it is walkable.
    pub fn downhill(&self, x: Fx, z: Fx) -> Option<V3> {
        let e = Fx::ratio(1, 2);
        let h = |x: Fx, z: Fx| self.relief_at(x, z);
        let gx = h(x.add(e), z).sub(h(x.sub(e), z));
        let gz = h(x, z.add(e)).sub(h(x, z.sub(e)));
        if gx.abs().max(gz.abs()).raw() <= t::terrain_steepest().raw() {
            return None;
        }
        let down = V3::new(gx.neg(), Fx::ZERO, gz.neg());
        Some(crate::math::wide_normalized(down))
    }

    /// Is the floor one plane here? [`relief::is_flat`], for whichever place
    /// is under the point.
    pub fn is_flat_at(&self, x: Fx, z: Fx) -> bool {
        match self.atlas {
            None => self.arena.is_flat_at(x, z),
            Some(a) => a.is_flat_at(x.add(self.origin.x), z.add(self.origin.z)),
        }
    }

    /// [`Arena::floor_at`]: what the floor is made of, ignoring every solid.
    pub fn floor_at(&self, x: Fx, z: Fx) -> Material {
        match self.atlas {
            None => self.arena.floor_at(x, z),
            Some(a) => a.floor_at(x.add(self.origin.x), z.add(self.origin.z)),
        }
    }

    /// [`Arena::inside`]: is a point still in the world at all? On a map,
    /// anywhere over the map and above its void.
    pub fn inside(&self, pos: V3) -> bool {
        match self.atlas {
            None => self.arena.inside(pos),
            Some(a) => {
                let p = pos.add(self.origin);
                p.y.raw() >= a.void.raw()
                    && p.x.raw() >= a.lo.0.raw()
                    && p.x.raw() <= a.hi.0.raw()
                    && p.z.raw() >= a.lo.1.raw()
                    && p.z.raw() <= a.hi.1.raw()
            }
        }
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
        resolve_among(self, pos, vel, was_grounded, radius, height)
    }

    /// [`Arena::ground_under`], with the raised solids standing on the floor.
    pub fn ground_under(&self, pos: V3) -> Fx {
        ground_among(self, pos)
    }

    /// **The floor a point is over**: the highest top under it that is no
    /// higher than the point itself (a skin's slack), or the floor. Unlike
    /// [`Terrain::ground_under`] a tower beside you is not under a point at
    /// your feet. What a floor marker is drawn on: the Cliffs' plateau, not the
    /// shelf twelve metres under it.
    pub fn floor_below(&self, pos: V3) -> Fx {
        let mut best = self.relief_at(pos.x, pos.z);
        for s in self.around(pos, Fx::ZERO) {
            let top = s.max.y;
            if s.over(pos.x, pos.z, Fx::ZERO)
                && !self.hangs(&s)
                && top.raw() <= pos.y.add(SKIN).raw()
                && top.raw() > best.raw()
            {
                best = top;
            }
        }
        best
    }

    /// [`Arena::material_under`], with the raised solids' tops.
    pub fn material_under(&self, pos: V3) -> Material {
        material_among(self, pos)
    }

    /// **The lowest ceiling near a point, sloped**: for every solid hanging
    /// from the roof whose underside is above `above`, its underside plus
    /// `slope` for each metre the point stands outside its footprint (the
    /// larger of the two flat distances). Directly under a solid it is the
    /// underside, as [`Terrain::ceiling_over`] says; walking out from under an
    /// edge it rises away instead of vanishing. That is what keeps the eye
    /// (`camera::eye_under`) from jumping as it or the fighter crosses an
    /// edge: on a course every island hangs, and a hard footprint test popped
    /// the eye down and up again as you passed under one (2026-10-04, from
    /// play: "the camera jumped on me").
    ///
    /// The arena's own solids only, as it always was: a raised solid stands
    /// on the floor and is never a ceiling.
    pub fn ceiling_near(&self, x: Fx, z: Fx, above: Fx, slope: Fx) -> Option<Fx> {
        let mut best: Option<Fx> = None;
        let at = V3::new(x, Fx::ZERO, z);
        for s in self.without_raised(self.around(at, CEILING_REACH)) {
            if !self.hangs(&s) || s.min.y.raw() <= above.raw() {
                continue;
            }
            let dx = s.min.x.sub(x).max(x.sub(s.max.x)).max(Fx::ZERO);
            let dz = s.min.z.sub(z).max(z.sub(s.max.z)).max(Fx::ZERO);
            let at = s.min.y.add(dx.max(dz).mul(slope));
            if best.is_none_or(|b| at.raw() < b.raw()) {
                best = Some(at);
            }
        }
        best
    }

    /// **The lowest ceiling over a point**: the underside of the lowest solid
    /// hanging from the roof whose footprint covers it and whose underside is
    /// above `above`. What keeps the eye inside a cave (`camera::eye_under`).
    pub fn ceiling_over(&self, x: Fx, z: Fx, above: Fx) -> Option<Fx> {
        let mut best: Option<Fx> = None;
        let at = V3::new(x, Fx::ZERO, z);
        for s in self.without_raised(self.around(at, Fx::ZERO)) {
            if !self.hangs(&s) || s.min.y.raw() <= above.raw() || !s.over(x, z, Fx::ZERO) {
                continue;
            }
            if best.is_none_or(|b| s.min.y.raw() < b.raw()) {
                best = Some(s.min.y);
            }
        }
        best
    }

    /// A query's own solids, leaving off the raised ones.
    fn without_raised<'a>(&self, mut solids: Solids<'a>) -> Solids<'a> {
        solids.raised = [].iter();
        solids
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
        for solid in self.around(pos, radius.add(RESOLVE_REACH)) {
            if self.hangs(&solid) || solid.max.y.raw() <= pos.y.add(step).raw() {
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
