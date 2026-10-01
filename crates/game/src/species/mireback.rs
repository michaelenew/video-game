//! How the Mireback is drawn: a hide black-brown with tar, and the warts on
//! its back lit like embers, because they are what a rider climbs up to burst.

use super::{Look, Paint};
use sim::species::mireback::{self, bones};

pub const LOOK: Look = Look {
    armour: Paint {
        rgb: [0.11, 0.09, 0.07],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.35,
    },
    // It has no weak point that is always open: what is soft on it is soft
    // because of what it is doing (the sac out, the belly up), and says so
    // by being out or up.
    weak: Paint {
        rgb: [0.86, 0.32, 0.22],
        glow: [0.55, 0.08, 0.04],
        roughness: 0.7,
    },
    // The four warts, swollen with tar and glowing from across the arena.
    breakable: Paint {
        rgb: [0.78, 0.46, 0.16],
        glow: [0.50, 0.20, 0.03],
        roughness: 0.6,
    },
    broken: Paint {
        rgb: [0.20, 0.15, 0.12],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.98,
    },
    // The root carries a body twelve metres across; a ball there would sit
    // inside it and cost a draw call for nothing. Same for the throat, inside
    // the jaw.
    no_knuckles: &[bones::ROOT, bones::THROAT, bones::TONGUE],
    head: mireback::JAW_PART,
    spikes: None,
    critters: &[],
};
