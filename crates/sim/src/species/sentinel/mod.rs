//! The sentinel: a dev creature for the shared machinery of bestiary P4 to P7.
//!
//! Not a creature anybody hunts for a trophy. It is to the senses, the floor
//! and the defended things what the gnats are to packs and the range is to
//! arenas: the one registered species that uses every piece, so that
//! `tests/hazards.rs`, `tests/senses.rs` and `tests/objectives.rs` stand the
//! machinery up end to end and `--hunt sentinel` puts something on the floor
//! for the renderer to draw before any creature that needs it is built.
//!
//! **The Ridgeback's body and moves, borrowed whole** -- its bones, parts,
//! clips and pose table are the Ridgeback's own statics, and its baked file
//! began as a copy of the Ridgeback's -- with everything a creature can bring
//! to a fight besides:
//!
//! - **Senses**: it sees only inside a cone off its facing, not through a
//!   solid or a cloud, and not into a scarred side (its own word 0: a side,
//!   or zero); and it hears the noise ring when it sees nobody.
//! - **A body against the arena**: it walks into solids, and a charge that
//!   does is knocked out of its stride (`bumped`).
//! - **Every generic hazard**: tar that slows and burns into slag, a snare
//!   that roots, a sinkhole that pulls, smoke that blocks sight and burns off,
//!   a vent on its back that lifts and scalds, a strand.
//! - **Two defended things**, where the arena has sites for them (the range):
//!   a gate, and a cart on a road.
//!
//! On the first frame of a hunt it lays one of each hazard round the middle
//! of the proving ground (its own word 1 says it has), so there is a floor to
//! look at. Its numbers are first guesses, set with `bake_tuning --set`.

mod tuned;

use super::ridgeback::{self as rb, baked};
use crate::arena::Material;
use crate::beast::breakables;
use crate::fixed::Fx;
use crate::hazard::{self, Hazard, HazardDecl, reach};
use crate::lore::Layout;
use crate::math::V3;
use crate::monster::{Doing, Monster};
use crate::objective::ObjectiveDecl;
use crate::perception::{self, Perceiver};
use crate::species::{FightDecl, Species, SpeciesId, Stock};
use crate::state::World;

/// The hazard kinds, by index.
pub const TAR: u8 = 0;
pub const BURNING: u8 = 1;
pub const SLAG: u8 = 2;
pub const SNARE: u8 = 3;
pub const SINKHOLE: u8 = 4;
pub const SMOKE: u8 = 5;
pub const FLASH: u8 = 6;
pub const VENT: u8 = 7;
pub const STRAND: u8 = 8;

pub const HAZARDS: [HazardDecl; 9] = [
    HazardDecl::disc("tar")
        .reaching(reach::FIGHTERS | reach::CREATURES)
        .ignites_into(BURNING)
        .made_of(Material::Peat),
    HazardDecl::disc("burning tar")
        .reaching(reach::ALL)
        .burning()
        .becomes(SLAG)
        .made_of(Material::Ash),
    HazardDecl::disc("slag").solid(Material::Rock),
    HazardDecl::disc("snare").ignites_into(FLASH),
    HazardDecl::disc("sinkhole").made_of(Material::Sand),
    HazardDecl::disc("smoke")
        .reaching(0)
        .blocks_sight()
        .ignites_into(FLASH)
        .made_of(Material::Ash),
    HazardDecl::disc("flash").reaching(reach::ALL).burning(),
    HazardDecl::disc("vent").made_of(Material::Stone),
    HazardDecl::strand("strand"),
];

/// The defended things, by index.
pub const GATE: usize = 0;
pub const CART: usize = 1;

pub const OBJECTIVES: [ObjectiveDecl; 2] = [
    ObjectiveDecl::wall("gate", 0),
    ObjectiveDecl::cart("cart", 1),
];

/// Its own words in the hunt's lore.
pub mod word {
    /// Which side is scarred: positive right, negative left, zero neither.
    pub const SCAR: usize = 0;
    /// The demo floor has been laid this round.
    pub const LAID: usize = 1;
}

/// It perceives a fighter inside its sight cone, outside its scarred side,
/// with nothing solid and no smoke between its head and them.
fn perceives(p: &Perceiver) -> bool {
    let scar = p.lore.int(word::SCAR);
    perception::in_sight_cone(p)
        && !perception::in_blind_arc(p, scar)
        && perception::in_line_of_sight(p)
}

/// A move that runs it into a solid knocks it out of its stride: the
/// Hornback's charge into a rock, in miniature.
fn bumped(m: &mut Monster, _push: V3) {
    if matches!(m.doing, Doing::Active { .. }) {
        m.doing = Doing::Flinch {
            left: m.sp().flinch_frames(),
        };
    }
}

/// On the first frame of a round, one of each hazard round the middle of the
/// proving ground, and a vent on its own back: something on the floor to
/// look at.
fn frame(w: &mut World) {
    if w.lore.word(word::LAID) != 0 {
        return;
    }
    w.lore.set_word(word::LAID, 1);
    let at = |x: i32, z: i32| V3::new(Fx::from_int(x), Fx::ZERO, Fx::from_int(z));
    let lay = |w: &mut World, h: Hazard| {
        hazard::place(&mut w.lore, h);
    };
    let r = |sp: &Species, kind: u8| hazard::stat_fx(sp, kind, hazard::HazardField::Radius);
    let sp = &SPECIES;
    lay(w, Hazard::disc(TAR, at(-6, 6), r(sp, TAR)));
    lay(w, Hazard::disc(SNARE, at(0, 9), r(sp, SNARE)));
    lay(w, Hazard::disc(SINKHOLE, at(6, -6), r(sp, SINKHOLE)));
    lay(w, Hazard::disc(SMOKE, at(-6, -8), r(sp, SMOKE)));
    lay(
        w,
        Hazard::strand(STRAND, at(2, -11), at(10, -11), r(sp, STRAND)),
    );
    lay(
        w,
        Hazard::on_part(VENT, 0, rb::RIDGE, V3::ZERO, r(sp, VENT)),
    );
}

pub static FIGHT: FightDecl = FightDecl {
    layout: Layout {
        hazards: 10,
        noises: 8,
        objectives: 2,
        own: 1,
    },
    row: true,
    hazards: &HAZARDS,
    objectives: &OBJECTIVES,
    perceives,
    hears: true,
    collides: true,
    bumped: Some(bumped),
    frame: Some(frame),
    shown: None,
};

pub static SPECIES: Species = Species {
    id: SpeciesId::SENTINEL,
    name: "Sentinel",
    bones: &rb::BONES,
    mirror: &rb::MIRROR,
    neck: &rb::NECK_CHAIN,
    follows: &rb::FOLLOWS,
    parts: &rb::PARTS,
    breakable: breakables(&rb::PARTS),
    legs: &rb::LEGS,
    moves: &rb::MOVES,
    clips: &rb::CLIPS,
    stock: Stock {
        idle: rb::Clip::Idle as usize,
        walk: rb::Clip::Walk as usize,
        gallop: rb::Clip::Gallop as usize,
        flinch: rb::Clip::Flinch as usize,
        stumble: rb::Clip::Stumble as usize,
        topple: rb::Clip::Topple as usize,
        dead: rb::Clip::Dead as usize,
    },
    span: &baked::SPAN,
    rows: baked::ROWS,
    row: |r| &baked::FRAMES[r],
    own: rb::Knob::DECLS,
    tuned: &tuned::KNOBS,
    tuned_path: "crates/sim/src/species/sentinel/tuned.rs",
    pack: None,
    fight: &FIGHT,
};
