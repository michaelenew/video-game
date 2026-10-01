//! How the Broodmother is drawn: a cave-dark spider, chitin black-brown, the
//! underside and the waist a dull bruise (her weak points say so only when
//! she is down), her legs' shins a lighter horn so a broken one reads, and the
//! sacs on her back the brightest things in the cave -- pale and milky while
//! they swell, amber past half, red in the last four and a half seconds, and
//! twitching red when held (broodmother.md §7). A popped sac is a torn husk
//! that stays; a burst one is gone until its site lays again
//! (`sim::species::broodmother::fight::shown`).

use super::{CritterPaint, Look, Paint, Tint};
use sim::species::broodmother::{self as bm, bones, fight};

pub const LOOK: Look = Look {
    armour: Paint {
        rgb: [0.11, 0.09, 0.08],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.55,
    },
    weak: Paint {
        rgb: [0.52, 0.22, 0.26],
        glow: [0.22, 0.04, 0.06],
        roughness: 0.7,
    },
    // The sacs, whole -- overridden by `TINT` while they swell -- and the
    // middle legs' shins.
    breakable: Paint {
        rgb: [0.36, 0.30, 0.24],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.6,
    },
    broken: Paint {
        rgb: [0.20, 0.16, 0.11],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.95,
    },
    // The root carries the thorax; a ball there sits inside it.
    no_knuckles: &[bones::ROOT, bones::ABDOMEN],
    head: bm::HEAD_PART,
    spikes: None,
    // The brood are gnawers, painted a shade paler -- hatched in the dark.
    critters: &[CritterPaint {
        body: Paint {
            rgb: [0.42, 0.38, 0.33],
            glow: [0.0, 0.0, 0.0],
            roughness: 0.9,
        },
        mark: Paint {
            rgb: [1.0, 0.78, 0.3],
            glow: [1.2, 0.7, 0.15],
            roughness: 0.35,
        },
        horns: None,
    }],
    horns: None,
    stance: None,
};

/// The sacs on their clocks: pale, amber, red, and the twitch's bright red.
pub const TINT: Tint = Tint {
    stages: &[
        Paint {
            rgb: [0.78, 0.76, 0.66],
            glow: [0.10, 0.10, 0.08],
            roughness: 0.35,
        },
        Paint {
            rgb: [0.92, 0.62, 0.22],
            glow: [0.45, 0.22, 0.04],
            roughness: 0.3,
        },
        Paint {
            rgb: [0.85, 0.16, 0.10],
            glow: [0.55, 0.06, 0.03],
            roughness: 0.3,
        },
        Paint {
            rgb: [1.0, 0.22, 0.12],
            glow: [1.3, 0.16, 0.06],
            roughness: 0.25,
        },
    ],
    stage: fight::sac_stage,
};
