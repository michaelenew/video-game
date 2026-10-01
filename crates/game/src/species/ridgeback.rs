//! How the Ridgeback is drawn: mossy armour, and two weak points that glow.

use super::{Look, Paint, Spikes};
use sim::species::ridgeback::{self, bones};

pub const LOOK: Look = Look {
    armour: Paint {
        rgb: [0.24, 0.27, 0.23],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.92,
    },
    // The ridge and the nape, and they say so.
    weak: Paint {
        rgb: [0.86, 0.32, 0.22],
        glow: [0.55, 0.08, 0.04],
        roughness: 0.7,
    },
    // The feet.
    breakable: Paint {
        rgb: [0.32, 0.34, 0.29],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.9,
    },
    broken: Paint {
        rgb: [0.18, 0.13, 0.13],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.98,
    },
    // The root and the chest carry the barrel; a ball there would sit inside a
    // box two and a half metres wide and cost a draw call for nothing.
    no_knuckles: &[bones::ROOT, bones::SPINE, bones::CHEST],
    head: ridgeback::HEAD,
    // The spike spray: bristling along the middle of the tail and its tip,
    // then a volley down the lane.
    spikes: Some(Spikes {
        kind: ridgeback::SPRAY,
        along: [ridgeback::TAIL_MID, ridgeback::TAIL_TIP],
        paint: Paint {
            rgb: [0.93, 0.86, 0.70],
            glow: [0.9, 0.45, 0.12],
            roughness: 0.5,
        },
    }),
    critters: &[],
};
