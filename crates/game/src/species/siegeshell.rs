//! How the Siegeshell is drawn: a shell the colour of the valley's stone,
//! weathered dark at the rim and pale on the plateau, the legs a darker horn,
//! the ankles -- the boxes the ground game breaks -- a lighter, banded bone so
//! they read from across the valley, and the three anchors on the crown the
//! brightest things in the fight: cold crystal, glowing, and a hot white when
//! one stands open (siegeshell.md §6). A broken ankle is a cracked dark grey;
//! a broken anchor is a stump and is not drawn.

use super::{CritterPaint, Look, Paint, Tint};
use sim::species::siegeshell::{self as ss, bones, fight};

pub const LOOK: Look = Look {
    armour: Paint {
        rgb: [0.36, 0.33, 0.28],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.85,
    },
    weak: Paint {
        rgb: [0.5, 0.42, 0.3],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.7,
    },
    // The ankles, whole; the anchors are painted by `TINT`.
    breakable: Paint {
        rgb: [0.72, 0.66, 0.54],
        glow: [0.06, 0.05, 0.03],
        roughness: 0.6,
    },
    broken: Paint {
        rgb: [0.18, 0.17, 0.16],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.95,
    },
    // The root and the shell's sides sit inside boxes far wider than a ball.
    no_knuckles: &[bones::ROOT, bones::SHELL_L, bones::SHELL_R, bones::CROWN],
    head: ss::HEAD_PART,
    spikes: None,
    // The parasites are gnawers, the shell's own grey-green.
    critters: &[CritterPaint {
        body: Paint {
            rgb: [0.30, 0.34, 0.26],
            glow: [0.0, 0.0, 0.0],
            roughness: 0.9,
        },
        mark: Paint {
            rgb: [0.85, 0.95, 0.6],
            glow: [0.8, 1.0, 0.4],
            roughness: 0.35,
        },
        horns: None,
    }],
    horns: None,
    stance: None,
};

/// The anchors: cold crystal, and white-hot while one stands open.
pub const TINT: Tint = Tint {
    stages: &[
        Paint {
            rgb: [0.45, 0.75, 0.95],
            glow: [0.35, 0.75, 1.3],
            roughness: 0.2,
        },
        Paint {
            rgb: [1.0, 0.95, 0.8],
            glow: [2.2, 1.9, 1.2],
            roughness: 0.15,
        },
    ],
    stage: anchor_stage,
};

fn anchor_stage(w: &sim::World, part: usize) -> Option<usize> {
    let a = ss::anchor_of(part)?;
    let m = w
        .monsters
        .iter()
        .flatten()
        .find(|m| m.species == ss::SPECIES.id)?;
    if fight::open_anchor(m) == Some(a) {
        Some(1)
    } else {
        Some(0)
    }
}
