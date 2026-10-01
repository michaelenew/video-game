//! How the Veilstalker is drawn: a pale, grey-dun lizard-cat -- the colour of
//! birch bark in snow -- and, mostly, not at all. The renderer draws each part
//! at the strength the simulation shows it (`World::shown`): nothing while it
//! is cloaked, a pale see-through shimmer while it fades, bounds or carries
//! paint, and this hide when it rears, recovers, panics or has been hurt
//! enough that a region never cloaks again. The trail, the paint, the breath
//! and the mimic's ghost are `crate::veil`'s.
//!
//! Nothing on it is a weak point or breakable; the head is hide, a little
//! harder hit (1.2x), and is painted as hide.

use super::{Look, Paint};
use sim::species::veilstalker as vs;

pub const LOOK: Look = Look {
    // Birch bark in snow: pale, cool, a little warm in the grey.
    armour: Paint {
        rgb: [0.66, 0.64, 0.58],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.85,
    },
    weak: Paint {
        rgb: [0.66, 0.64, 0.58],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.85,
    },
    // Nothing breaks; these are never used.
    breakable: Paint {
        rgb: [0.66, 0.64, 0.58],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.85,
    },
    broken: Paint {
        rgb: [0.34, 0.32, 0.30],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.95,
    },
    no_knuckles: &[],
    head: vs::HEAD,
    spikes: None,
    critters: &[],
    horns: None,
    stance: None,
};
