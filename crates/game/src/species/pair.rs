//! How the Pair is drawn: two big cats, one dun and one dark, so a player can
//! say "the dark one is behind me" -- the second body in a fight is drawn in
//! its species' hide darkened (`crate::beast`), and the cats are the first
//! fight with a second body.
//!
//! The head is the only place worth hitting harder, and it is not a weak point
//! (1.3x, The Pair §4), so it is painted as hide rather than lit up: the eyes
//! a scar takes are a lasting thing to see, not a target.

use super::{Look, Paint};
use sim::species::pair;

pub const LOOK: Look = Look {
    // Dun: a dry-grass cat.
    armour: Paint {
        rgb: [0.62, 0.48, 0.30],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.8,
    },
    weak: Paint {
        rgb: [0.70, 0.20, 0.14],
        glow: [0.20, 0.04, 0.02],
        roughness: 0.5,
    },
    // Nothing on a cat breaks; these are never used.
    breakable: Paint {
        rgb: [0.62, 0.48, 0.30],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.8,
    },
    broken: Paint {
        rgb: [0.30, 0.24, 0.18],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.95,
    },
    no_knuckles: &[],
    head: pair::HEAD,
    spikes: None,
    critters: &[],
    horns: None,
    stance: None,
};
