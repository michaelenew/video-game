//! What the two cats do together.

use crate::lore::Layout;
use crate::species::FightDecl;

pub static FIGHT: FightDecl = FightDecl {
    layout: Layout {
        hazards: 0,
        noises: 0,
        objectives: 0,
        own: 8,
    },
    row: true,
    collides: true,
    keeps_height: true,
    bodies: 2,
    ..FightDecl::PLAIN
};
