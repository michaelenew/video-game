//! How the Hornback herd is drawn: tawny cows with a pale blaze, and the bull
//! darker and heavier with two pale horns. The eyes catch the light; the
//! bull's horns are drawn only while they are whole, so a broken one is gone
//! from its head (hornback.md §4) -- and its stun is drawn as its head in the
//! dirt (§1).

use super::{CritterPaint, Look, NO_BODY, Paint, StanceHint};
use sim::species::hornback as h;

pub const LOOK: Look = Look {
    armour: NO_BODY,
    weak: NO_BODY,
    breakable: NO_BODY,
    broken: NO_BODY,
    no_knuckles: &[],
    head: 0,
    spikes: None,
    // In `sim::species::hornback::KINDS` order: the cow, then the bull.
    critters: &[
        CritterPaint {
            body: Paint {
                rgb: [0.55, 0.40, 0.24],
                glow: [0.0, 0.0, 0.0],
                roughness: 0.9,
            },
            mark: Paint {
                rgb: [0.95, 0.85, 0.6],
                glow: [0.5, 0.4, 0.15],
                roughness: 0.4,
            },
            horns: None,
        },
        CritterPaint {
            body: Paint {
                rgb: [0.24, 0.15, 0.10],
                glow: [0.0, 0.0, 0.0],
                roughness: 0.85,
            },
            mark: Paint {
                rgb: [1.0, 0.55, 0.25],
                glow: [1.1, 0.45, 0.12],
                roughness: 0.35,
            },
            horns: Some(Paint {
                rgb: [0.92, 0.88, 0.78],
                glow: [0.15, 0.13, 0.1],
                roughness: 0.3,
            }),
        },
    ],
    horns: Some(horns),
    stance: Some(stance),
};

/// The bull's horns, as the pack's memo has them; a cow has none.
fn horns(w: &sim::World, slot: usize) -> Option<[bool; 2]> {
    let c = w.critters.get(slot)?;
    if c.kind != h::BULL {
        return None;
    }
    w.pack.map(|p| h::horns_whole(&p))
}

/// Stunned: nose down, horns in the dirt, sunk on its forelegs.
fn stance(c: &sim::critter::Critter) -> Option<StanceHint> {
    h::stunned(c).then_some(StanceHint {
        pitch: -0.22,
        roll: 0.06,
        head_dip: 0.9,
        drop: 0.08,
    })
}
