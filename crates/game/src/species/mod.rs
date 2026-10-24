//! What each creature looks like: one file per species, one [`Look`] in each.
//!
//! The renderer draws any creature from its species table -- a box per part,
//! placed by the simulation's own rig -- so all a species adds here is its
//! paint and the few decorations that are its alone. Everything is declared in
//! the species' own file; this one only finds it.
//!
//! Every planned creature already has a line in [`look`], commented out and one
//! blank line from the next, so two branches each adding a creature do not
//! conflict (the same arrangement as `sim::species`).

use sim::species::SpeciesId;

pub mod ridgeback;

/// The dev pack's paint (`sim::species::gnats`).
pub mod gnats;

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

/// One material, as numbers: the renderer turns it into a Bevy material once.
#[derive(Clone, Copy, Debug)]
pub struct Paint {
    pub rgb: [f32; 3],
    /// Glow, for the parts that must read from across the arena. Black is none.
    pub glow: [f32; 3],
    pub roughness: f32,
}

/// A volley of spikes a move throws, drawn bristling along two parts through
/// its windup and flying down its lane while it is out. The Ridgeback's spray.
#[derive(Clone, Copy, Debug)]
pub struct Spikes {
    /// The move, by the species' own numbering.
    pub kind: u8,
    /// The two parts they bristle along, half on each.
    pub along: [usize; 2],
    pub paint: Paint,
}

/// How a species is drawn.
#[derive(Clone, Copy, Debug)]
pub struct Look {
    /// Everything that is not one of the below.
    pub armour: Paint,
    /// Weak points, and they should say so: the only places worth hitting,
    /// visible from across the arena without being told.
    pub weak: Paint,
    /// Breakable parts, whole.
    pub breakable: Paint,
    /// Breakable parts, broken.
    pub broken: Paint,
    /// Bones that get no joint filler: the ones buried inside a box too wide
    /// for a ball to show, where it would only cost a draw call.
    pub no_knuckles: &'static [usize],
    /// The part the debug overlay draws the creature's intent from, and the
    /// Reaver's marks float over: its head.
    pub head: usize,
    pub spikes: Option<Spikes>,
    /// **Its small bodies**, one paint per kind of critter in its pack, in the
    /// pack's kind order (`sim::pack::PackDecl::kinds`). Empty for a creature
    /// with no pack. Their shapes are not here: every critter is drawn from
    /// its own knobs -- length, width, height -- by `crate::critters`, so what
    /// you see is the box the hit test uses.
    pub critters: &'static [CritterPaint],
    /// **Which of a critter's horns are whole**, for a species whose small
    /// bodies have them: the world, and the critter's slot. `None` for every
    /// creature without horns. The Hornback's bull: a broken horn is gone
    /// from its head, and that is a consequence the player did.
    pub horns: Option<HornsFn>,
    /// **A stance of a critter's own**, beyond the stock poses: the stunned
    /// bull, head in the dirt. `None`: the stock poses only.
    pub stance: Option<Stance>,
}

/// **Parts painted by the fight's own state**, beyond whole and broken: the
/// Broodmother's sacs, pale to red on their clocks. A set of paints, and which
/// of them a part wears this frame -- the world and the part, to an index into
/// `stages`, or `None` for the usual paint. Found by [`tint`], beside the look
/// rather than in it, so a look without one says nothing.
#[derive(Clone, Copy, Debug)]
pub struct Tint {
    pub stages: &'static [Paint],
    pub stage: fn(&sim::World, usize) -> Option<usize>,
}

/// [`Look::horns`]: the world and a critter's slot, to which of its two horns
/// are whole, or `None` for a body with none.
pub type HornsFn = fn(&sim::World, usize) -> Option<[bool; 2]>;

/// A species' own stance for a critter, if it has one this frame.
pub type Stance = fn(&sim::critter::Critter) -> Option<StanceHint>;

/// How a critter is held, in the renderer's own terms (radians, and a share
/// of its length for how far it sinks).
#[derive(Clone, Copy, Debug)]
pub struct StanceHint {
    pub pitch: f32,
    pub roll: f32,
    pub head_dip: f32,
    pub drop: f32,
}

/// How one kind of critter is painted.
#[derive(Clone, Copy, Debug)]
pub struct CritterPaint {
    /// Its hide.
    pub body: Paint,
    /// Its eyes, and the tail of one holding an attack token -- the thing
    /// the Gnawers' document says is the most important on the screen.
    pub mark: Paint,
    /// Its horns, if it has any (`Look::horns` says which are whole).
    pub horns: Option<Paint>,
}

/// A body with nothing to draw but its small bodies, for a creature that is
/// only a pack: the monster paints are never used, and are visible if they are.
pub const NO_BODY: Paint = Paint {
    rgb: [1.0, 0.0, 1.0],
    glow: [0.0, 0.0, 0.0],
    roughness: 1.0,
};

/// A species' look. A creature with none is drawn in the Ridgeback's paint,
/// which is wrong but visible -- better than an invisible body that still hits.
pub fn look(id: SpeciesId) -> &'static Look {
    match id {
        SpeciesId::RIDGEBACK => &ridgeback::LOOK,

        SpeciesId::GNATS => &gnats::LOOK,

        SpeciesId::GNAWERS => &gnawers::LOOK,

        SpeciesId::HORNBACK => &hornback::LOOK,
        SpeciesId::MIREBACK => &mireback::LOOK,

        SpeciesId::SANDMAW => &sandmaw::LOOK,

        SpeciesId::PAIR => &pair::LOOK,

        SpeciesId::BROODMOTHER => &broodmother::LOOK,

        SpeciesId::VEILSTALKER => &veilstalker::LOOK,

        SpeciesId::MANTIS => &mantis::LOOK,

        SpeciesId::GALEWING => &galewing::LOOK,
        SpeciesId::SIEGESHELL => &siegeshell::LOOK,
        _ => &ridgeback::LOOK,
    }
}

/// **How a species' body is built** (`crate::contour`, exploration 0027):
/// the contours its parts are grown in, where its own say differs from what
/// `crate::grown` infers from the part's shape and bone, and the dials every
/// part shares. Found beside the look, as a tint is, so a species that says
/// nothing is grown by inference.
#[derive(Clone, Copy, Debug)]
pub struct Build {
    /// How much flesh over the structure (0 ribs and bellies show, 1 hidden).
    pub fat: f32,
    /// How pronounced the muscle is where a contour has any.
    pub muscle: f32,
    /// How much of a limb is a column rather than a taper: 1 a graviportal
    /// animal (an elephant), 0 a cursorial one (a horse).
    pub heavy: f32,
    /// Parts contoured by the species' own word, by index.
    pub contours: &'static [(usize, crate::contour::Contour)],
    /// **What the sculpted target taught** (2026-10-23, exploration 0029):
    /// the choices that made the hand-made body read as a monster, stated so
    /// a grown one makes them too. How far a barrel hangs below its box, as a
    /// share of its depth: a deep chest, the belly low between the legs.
    pub belly: f32,
    /// How much thicker a leg is than its box: a column the body comes down
    /// over.
    pub girth: f32,
    /// How thick the carapace is over the back, in metres (0: none): grown
    /// off the skin wherever it is armoured, in scutes, parting where a weak
    /// point stands.
    pub armour: f32,
    /// How big a scute is across the back, in metres.
    pub scute: f32,
    /// How tall the spines either side of a weak strip down the back stand,
    /// in metres (0: none).
    pub spines: f32,
    /// How long the horns over the eyes are, in metres (0: none); with them
    /// come brow nubs, a nose horn and tusks.
    pub horns: f32,
    /// How many claws a foot has.
    pub claws: u8,
}

impl Build {
    pub const DEFAULT: Build = Build {
        fat: 0.5,
        muscle: 0.5,
        heavy: 0.5,
        contours: &[],
        belly: 0.15,
        girth: 1.0,
        armour: 0.0,
        scute: 0.8,
        spines: 0.0,
        horns: 0.0,
        claws: 0,
    };
}

/// A species' [`Build`]: its own, or the default.
pub fn build(id: SpeciesId) -> &'static Build {
    match id {
        SpeciesId::RIDGEBACK => &ridgeback::BUILD,
        _ => &Build::DEFAULT,
    }
}

/// A species' [`Tint`], if it has one.
pub fn tint(id: SpeciesId) -> Option<&'static Tint> {
    match id {
        SpeciesId::BROODMOTHER => Some(&broodmother::TINT),
        SpeciesId::GALEWING => Some(&galewing::TINT),
        SpeciesId::SIEGESHELL => Some(&siegeshell::TINT),
        _ => None,
    }
}
