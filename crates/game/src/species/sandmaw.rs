//! How the Sandmaw is drawn: a hide the colour of the dunes it swims in, a
//! tooth ring bone-white, and the spiracles and throat dark red, because the
//! only places worth hitting should be seen from across the Pan.
//!
//! Under the sand it is not drawn at all -- the simulation shows a buried
//! part at nothing (`World::shown`) -- and what you see is its wake: the
//! raised ridge, the fin, the ring where it heard you, all marks
//! (`sim::species::sandmaw::fight`), drawn where the simulation keeps them.

use super::{Look, Paint};
use sim::species::sandmaw::{self, bones};

pub const LOOK: Look = Look {
    armour: Paint {
        rgb: [0.55, 0.43, 0.28],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.85,
    },
    // The spiracles and the throat: wet, dark red, and faintly lit, so a
    // fighter behind a standing worm can pick the slits out at dusk.
    weak: Paint {
        rgb: [0.62, 0.12, 0.10],
        glow: [0.30, 0.04, 0.02],
        roughness: 0.4,
    },
    // The tooth ring: bone, catching the light.
    breakable: Paint {
        rgb: [0.93, 0.89, 0.78],
        glow: [0.12, 0.10, 0.06],
        roughness: 0.3,
    },
    broken: Paint {
        rgb: [0.32, 0.28, 0.22],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.95,
    },
    // The lips sit inside the head's box; a ball at their hinge only shows
    // through the open mouth, where it would be a tooth that is not there.
    no_knuckles: &[bones::LIP_UP, bones::LIP_DOWN],
    head: sandmaw::HEAD_PART,
    spikes: None,
    critters: &[],
    horns: None,
    stance: None,
};
