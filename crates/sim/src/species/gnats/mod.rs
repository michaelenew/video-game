//! The gnats: a dev pack, the smallest one that uses every piece of the
//! critter machinery.
//!
//! Not a creature anybody hunts for a trophy. It is to packs what the range is
//! to arenas: what `tests/critters.rs` and `critcheck` stand up, and what
//! `--hunt gnats` puts in the proving ground so the renderer and the harness
//! have a pack to look at before any real one is built. Six gnats a little
//! shorter than a gnawer and a queen that leads them; a bite that needs a
//! token, and a leap that does not and is only thrown at somebody already in
//! trouble -- the Gnawers' pile-on, in miniature, to show a species' mind
//! overriding the pack's.
//!
//! Its numbers are first guesses, set with `bake_tuning --set` and baked into
//! `tuned.rs` beside this file.

mod tuned;

use crate::critter::{CritterKind, CritterMove};
use crate::monster::Attack;
use crate::oven::KnobDecl;
use crate::pack::{Look, PackDecl, PackMind, default_appetite};
use crate::species::{MoveDecl, Species, SpeciesId};

/// The moves, by index.
pub const BITE: u8 = 0;
pub const LEAP: u8 = 1;

pub const MOVES: [MoveDecl; 2] = [MoveDecl::new("Bite", 0), MoveDecl::new("Leap", 0)];

/// The kinds, by index.
pub const GNAT: u8 = 0;
pub const QUEEN: u8 = 1;

pub const KINDS: [CritterKind; 2] = [
    CritterKind {
        name: "Gnat",
        moves: &[CritterMove::token(BITE), CritterMove::free(LEAP)],
        role: 0,
        yields: false,
        mountable: false,
    },
    CritterKind {
        name: "Queen",
        moves: &[CritterMove::token(BITE)],
        role: 1,
        yields: true,
        mountable: false,
    },
];

/// The gnats' own mind: the generic pack, except that the leap is only for a
/// fighter who cannot get out of the way.
pub struct Mind;

impl PackMind for Mind {
    fn appetite(&self, look: &Look, i: usize, m: CritterMove, a: &Attack) -> i32 {
        if m.kind == LEAP {
            let c = &look.critters[i];
            let seen = &look.pack.seen[(c.target as usize).min(crate::state::MAX_PLAYERS - 1)];
            if !(seen.down || seen.slowed) {
                return 0;
            }
        }
        default_appetite(look, i, m, a)
    }
}

pub static PACK: PackDecl = PackDecl {
    kinds: &KINDS,
    muster: &[(GNAT, 6), (QUEEN, 1)],
    leader: Some(QUEEN),
    mind: &Mind,
};

const OWN: &[KnobDecl] = &[];

pub static SPECIES: Species = Species::pack_only(
    SpeciesId::GNATS,
    "Gnats",
    &MOVES,
    OWN,
    &tuned::KNOBS,
    "crates/sim/src/species/gnats/tuned.rs",
    &PACK,
);
