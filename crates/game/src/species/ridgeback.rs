//! How the Ridgeback is drawn: mossy armour, and two weak points that glow.

use super::{Build, Look, Paint, Spikes};
use crate::contour::{Contour, Kind};
use sim::species::ridgeback::{self, bones};

pub const LOOK: Look = Look {
    // A warm, weathered brown: the hide of the sculpted target, from which a
    // grown body derives its pale stone too (`look::hide::Carapace`).
    armour: Paint {
        rgb: [0.42, 0.33, 0.25],
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
    horns: None,
    stance: None,
};

/// **How it is built**: a graviportal animal -- columnar legs with the
/// muscle showing, a heavy barrel with little rib to see under its armour,
/// a whip of a tail, a wedge of a head.
pub const BUILD: Build = Build {
    fat: 0.65,
    muscle: 0.8,
    heavy: 1.0,
    // The sculpted target's choices (`art/ridgeback/build.py`): a barrel a
    // metre deeper than its box, legs a third thicker, a fifth of a metre of
    // stone over the back in metre scutes, spines up to a metre and a
    // quarter, horns a metre and a half, a dog's four toes a foot.
    belly: 0.8,
    girth: 1.3,
    armour: 0.2,
    scute: 1.05,
    spines: 1.25,
    horns: 1.6,
    claws: 4,
    contours: &[
        (ridgeback::HEAD, Contour::of(Kind::Wedge).muscle(0.6)),
        (
            ridgeback::NECK_PART,
            Contour::of(Kind::Column).between_const(Kind::Taper, 0.4),
        ),
    ],
    ..Build::DEFAULT
};
