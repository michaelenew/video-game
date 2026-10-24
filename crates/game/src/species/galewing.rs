//! How the Galewing is drawn: a storm-grey raptor, slate above and pale below,
//! so against the sky it reads as a dark shape and from under it as a light
//! one. The wing roots -- where a hit counts double -- are a pale band across
//! the back, the one place on it that asks to be aimed at; a wing past half its
//! bar goes rust, and a broken one hangs dark (`TINT`, from
//! `sim::species::galewing::fight::wing_stage`).
//!
//! Nothing on it is a weak point or breakable in the shared sense: its wings
//! break on their own bars.

use super::{Build, Foot, Look, Paint, Tint};
use sim::species::galewing::{self as gw, fight};

pub const LOOK: Look = Look {
    // Slate, cool and dark: the body and the wings' tops.
    armour: Paint {
        rgb: [0.27, 0.29, 0.33],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.8,
    },
    weak: Paint {
        rgb: [0.27, 0.29, 0.33],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.8,
    },
    breakable: Paint {
        rgb: [0.27, 0.29, 0.33],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.8,
    },
    broken: Paint {
        rgb: [0.12, 0.11, 0.11],
        glow: [0.0, 0.0, 0.0],
        roughness: 0.95,
    },
    // The root carries the back; a ball there sits inside it.
    no_knuckles: &[gw::bones::ROOT],
    head: gw::HEAD,
    spikes: None,
    critters: &[],
    horns: None,
    stance: None,
};

/// The wings, by `fight::wing_stage`: a whole root, a cracking wing, a broken
/// one.
pub const TINT: Tint = Tint {
    stages: &[
        Paint {
            rgb: [0.78, 0.74, 0.62],
            glow: [0.04, 0.04, 0.03],
            roughness: 0.6,
        },
        Paint {
            rgb: [0.55, 0.30, 0.18],
            glow: [0.0, 0.0, 0.0],
            roughness: 0.8,
        },
        Paint {
            rgb: [0.12, 0.11, 0.11],
            glow: [0.0, 0.0, 0.0],
            roughness: 0.95,
        },
    ],
    stage: fight::wing_stage,
};

/// **How it is built** (2026-10-24, exploration 0030): the sculpted target's
/// choices (`art/galewing/build.py`) as rules -- a deep breast, feathered
/// thighs, wings that are a slab carrying flight feathers past it, a fan of
/// a tail, a hooked beak and a crest, a raptor's talons.
pub const BUILD: Build = Build {
    fat: 0.6,
    muscle: 0.5,
    heavy: 0.3,
    belly: 0.25,
    girth: 1.2,
    claws: 4,
    foot: Foot::Talon,
    feathers: 1.0,
    beak: 0.5,
    crest: 1.0,
    ..Build::DEFAULT
};
