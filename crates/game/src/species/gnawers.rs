//! How the Gnawers are drawn: badger-grey biters with a pale stripe, and the
//! Big One darker and heavier. Their eyes and a token-holder's raised tail
//! catch the light in amber -- the two raised tails are the most important
//! thing on the screen (gnawers.md §6).

use super::{CritterPaint, Look, NO_BODY, Paint};

pub const LOOK: Look = Look {
    armour: NO_BODY,
    weak: NO_BODY,
    breakable: NO_BODY,
    broken: NO_BODY,
    no_knuckles: &[],
    head: 0,
    spikes: None,
    // In `sim::species::gnawers::KINDS` order: the gnawer, then the Big One.
    critters: &[
        CritterPaint {
            body: Paint {
                rgb: [0.36, 0.33, 0.29],
                glow: [0.0, 0.0, 0.0],
                roughness: 0.95,
            },
            mark: Paint {
                rgb: [1.0, 0.78, 0.3],
                glow: [1.2, 0.7, 0.15],
                roughness: 0.35,
            },
        },
        CritterPaint {
            body: Paint {
                rgb: [0.22, 0.19, 0.17],
                glow: [0.0, 0.0, 0.0],
                roughness: 0.9,
            },
            mark: Paint {
                rgb: [1.0, 0.45, 0.2],
                glow: [1.3, 0.4, 0.1],
                roughness: 0.35,
            },
        },
    ],
};
