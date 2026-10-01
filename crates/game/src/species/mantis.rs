//! How the Mantis is drawn: pale green chitin, the blades bone-white -- the
//! two things the fight is read from are the arms, so the arms are the
//! brightest thing on it -- and a broken blade a dark stump.
//!
//! Its memory is drawn by the simulation's own marks
//! (`sim::species::mantis::habit::notches`): a notch per remembered move on
//! each blade, in the thrower's class colour, lit when a Ready waits for it
//! (`crate::ground`). The guard's arc is a fan of floor signs.

use super::{Look, Paint};
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
