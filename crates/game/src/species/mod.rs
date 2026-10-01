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

// pub mod gnawers;

// pub mod hornback;

// pub mod mireback;

// pub mod sandmaw;

// pub mod pair;

// pub mod broodmother;

// pub mod veilstalker;

// pub mod mantis;

// pub mod galewing;

// pub mod siegeshell;

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
}

/// How one kind of critter is painted.
#[derive(Clone, Copy, Debug)]
pub struct CritterPaint {
    /// Its hide.
    pub body: Paint,
    /// Its eyes, and the tail of one holding an attack token -- the thing
    /// the Gnawers' document says is the most important on the screen.
    pub mark: Paint,
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

        // SpeciesId::GNAWERS => &gnawers::LOOK,

        // SpeciesId::HORNBACK => &hornback::LOOK,

        // SpeciesId::MIREBACK => &mireback::LOOK,

        // SpeciesId::SANDMAW => &sandmaw::LOOK,

        // SpeciesId::PAIR => &pair::LOOK,

        // SpeciesId::BROODMOTHER => &broodmother::LOOK,

        // SpeciesId::VEILSTALKER => &veilstalker::LOOK,

        // SpeciesId::MANTIS => &mantis::LOOK,

        // SpeciesId::GALEWING => &galewing::LOOK,

        // SpeciesId::SIEGESHELL => &siegeshell::LOOK,
        _ => &ridgeback::LOOK,
    }
}
