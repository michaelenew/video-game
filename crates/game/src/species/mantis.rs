//! How the Mantis is drawn: pale green chitin, the blades bone-white -- the
//! two things the fight is read from are the arms, so the arms are the
//! brightest thing on it -- and a broken blade a dark stump.
//!
//! Its memory is drawn by the simulation's own marks
//! (`sim::species::mantis::habit::notches`): a notch per remembered move on
//! each blade, in the thrower's class colour, lit when a Ready waits for it
//! (`crate::ground`). The guard's arc is a fan of floor signs.

use super::{Build, Foot, Look, Paint};
use crate::contour::{Contour, Kind};
use sim::species::mantis;

pub const LOOK: Look = Look {
    armour: Paint {
        rgb: [0.46, 0.58, 0.30],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.55,
    },
    weak: Paint {
        rgb: [0.70, 0.20, 0.14],
        glow: [0.20, 0.04, 0.02],
        roughness: 0.5,
    },
    // The blades: bone-white, a little lit, so a stance reads at any range.
    breakable: Paint {
        rgb: [0.90, 0.88, 0.76],
        glow: [0.10, 0.10, 0.07],
        roughness: 0.35,
    },
    broken: Paint {
        rgb: [0.22, 0.20, 0.16],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.95,
    },
    no_knuckles: &[],
    head: mantis::HEAD,
    spikes: None,
    critters: &[],
    horns: None,
    stance: None,
};

/// **How it is built** (2026-10-24, exploration 0031): the sculpted target's
/// choices (`art/mantis/build.py`) as rules -- chitin in rings, a scythe
/// serrated along its cutting edge, spines under the arm that carries it,
/// antennae, compound eyes, a tarsus along the floor.
pub const BUILD: Build = Build {
    fat: 0.2,
    muscle: 0.3,
    heavy: 0.0,
    belly: 0.0,
    girth: 1.0,
    claws: 2,
    foot: Foot::Tarsus,
    chitin: 0.19,
    grip: 0.16,
    antennae: 1.0,
    eyes: 0.15,
    // The thorax stood on end: a bulb where the legs meet, the pronotum's
    // stalk, the shield under the arms.
    contours: &[
        (mantis::THORAX_PART, Contour::of(Kind::Stalk)),
        // The abdomen tapering from the waist to its tip.
        (
            mantis::ABDOMEN_PART,
            Contour::of(Kind::Barrel).between_const(Kind::Whip, 0.55),
        ),
    ],
    ..Build::DEFAULT
};
