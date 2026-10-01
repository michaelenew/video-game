//! What the Broodmother does to the floor, her sacs and the fighters: the sac
//! clock, the web, her legs and her arc, run once a frame from her
//! `FightDecl::frame` hook.

use crate::hazard::{HazardDecl, reach};
use crate::lore::Layout;
use crate::species::FightDecl;

/// A web patch: a disc of sticky floor where a glob landed.
pub const PATCH: u8 = 0;
/// A strand: a line a web line left behind, a shin's height off the floor.
pub const STRAND: u8 = 1;

pub const HAZARDS: [HazardDecl; 2] = [
    HazardDecl::disc("web patch").reaching(reach::FIGHTERS),
    HazardDecl::strand("strand").reaching(reach::FIGHTERS),
];

pub static FIGHT: FightDecl = FightDecl {
    layout: Layout {
        hazards: 10,
        noises: 0,
        objectives: 0,
        own: 3,
    },
    hazards: &HAZARDS,
    ..FightDecl::PLAIN
};
