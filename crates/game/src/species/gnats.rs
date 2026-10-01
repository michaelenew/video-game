//! How the dev pack is drawn: dun gnats with amber eyes, and a queen in rust.

use super::{CritterPaint, Look, NO_BODY, Paint};

pub const LOOK: Look = Look {
    armour: NO_BODY,
    weak: NO_BODY,
    breakable: NO_BODY,
    broken: NO_BODY,
    no_knuckles: &[],
    head: 0,
    spikes: None,
    // In `sim::species::gnats::KINDS` order: the gnat, then the queen.
    critters: &[
        CritterPaint {
            body: Paint {
                rgb: [0.42, 0.36, 0.27],
                glow: [0.0, 0.0, 0.0],
                roughness: 0.95,
            },
            mark: Paint {
                rgb: [1.0, 0.72, 0.25],
                glow: [0.9, 0.5, 0.1],
                roughness: 0.4,
            },
        },
        CritterPaint {
            body: Paint {
                rgb: [0.48, 0.22, 0.16],
                glow: [0.0, 0.0, 0.0],
                roughness: 0.9,
            },
            mark: Paint {
                rgb: [1.0, 0.85, 0.4],
                glow: [1.0, 0.6, 0.15],
                roughness: 0.4,
            },
        },
    ],
};
