//! Colours somebody else already got right.
//!
//! ## Why a borrowed palette rather than chosen numbers
//!
//! Every surface colour in this crate used to be three floats written by hand,
//! and the trouble with three floats written by hand is that there is nothing
//! stopping them being wrong. A brown picked by eye, lifted into a lightness
//! band and pushed to the chroma ceiling, arrives as olive -- a colour nobody
//! chose, that no reviewer approved, and that is hard to argue with because
//! each step that produced it was reasonable.
//!
//! A published palette is the opposite. Every entry has been looked at next to
//! every other entry by people whose job that was, the ramps are even in
//! lightness so a shade means the same thing across hues, and the chroma of
//! each is already judged -- a grey family is grey on purpose and a green
//! family is loud on purpose, rather than everything being dragged to one
//! ceiling. Choosing *which swatch* is a decision somebody can disagree with in
//! a word; choosing three floats is not.
//!
//! ## Which one, and why this one
//!
//! **Tailwind CSS v3's colour ramps**, MIT licensed, Copyright (c) Tailwind
//! Labs, Inc. Twenty families of eleven shades, perceptually spaced, and the
//! most heavily reviewed palette of its size in use anywhere -- which is the
//! real argument for it: a great many people would have complained by now if
//! any of these were ugly.
//!
//! It was picked over two near misses. **Open Color** (also MIT, also good) has
//! no earth tones at all -- it is built for interfaces, and an arena is mostly
//! ground, rock and sand. The pixel-art palettes collected on Lospec are made
//! for exactly this kind of landscape, and their licensing is per-palette and
//! often unstated, which is a bad thing to build a game's whole look on.
//!
//! The ramps are here in full rather than trimmed to what is used, because the
//! accent picks a family by hue at run time and wants the whole wheel to choose
//! from.

/// One family: its shades, lightest first.
///
/// A slice rather than a fixed array because the two palettes borrowed here do
/// not agree on how many shades a family has -- Tailwind runs 50 to 950 in
/// eleven, Material 50 to 900 in ten. The positions line up for the ten they
/// share, which is all anything here asks for.
pub struct Ramp(pub &'static [u32]);

/// Where in a ramp a shade sits, lightest first. Named the way the palettes
/// name them, which agrees up to 900.
pub const S50: usize = 0;
pub const S100: usize = 1;
pub const S200: usize = 2;
pub const S300: usize = 3;
pub const S400: usize = 4;
pub const S500: usize = 5;
pub const S600: usize = 6;
pub const S700: usize = 7;
pub const S800: usize = 8;
pub const S900: usize = 9;
pub const S950: usize = 10;

impl Ramp {
    /// One shade, as sRGB. A shade past the end of a shorter ramp gives its
    /// darkest.
    pub fn at(&self, shade: usize) -> [f32; 3] {
        hex(self.0[shade.min(self.0.len() - 1)])
    }
}

/// A packed `0xRRGGBB` as sRGB.
pub fn hex(packed: u32) -> [f32; 3] {
    [
        ((packed >> 16) & 0xff) as f32 / 255.0,
        ((packed >> 8) & 0xff) as f32 / 255.0,
        (packed & 0xff) as f32 / 255.0,
    ]
}

pub const SLATE: Ramp = Ramp(&[
    0xf8fafc, 0xf1f5f9, 0xe2e8f0, 0xcbd5e1, 0x94a3b8, 0x64748b, 0x475569, 0x334155, 0x1e293b,
    0x0f172a, 0x020617,
]);
pub const GRAY: Ramp = Ramp(&[
    0xf9fafb, 0xf3f4f6, 0xe5e7eb, 0xd1d5db, 0x9ca3af, 0x6b7280, 0x4b5563, 0x374151, 0x1f2937,
    0x111827, 0x030712,
]);
pub const STONE: Ramp = Ramp(&[
    0xfafaf9, 0xf5f5f4, 0xe7e5e4, 0xd6d3d1, 0xa8a29e, 0x78716c, 0x57534e, 0x44403c, 0x292524,
    0x1c1917, 0x0c0a09,
]);
pub const RED: Ramp = Ramp(&[
    0xfef2f2, 0xfee2e2, 0xfecaca, 0xfca5a5, 0xf87171, 0xef4444, 0xdc2626, 0xb91c1c, 0x991b1b,
    0x7f1d1d, 0x450a0a,
]);
pub const ORANGE: Ramp = Ramp(&[
    0xfff7ed, 0xffedd5, 0xfed7aa, 0xfdba74, 0xfb923c, 0xf97316, 0xea580c, 0xc2410c, 0x9a3412,
    0x7c2d12, 0x431407,
]);
pub const AMBER: Ramp = Ramp(&[
    0xfffbeb, 0xfef3c7, 0xfde68a, 0xfcd34d, 0xfbbf24, 0xf59e0b, 0xd97706, 0xb45309, 0x92400e,
    0x78350f, 0x451a03,
]);
pub const YELLOW: Ramp = Ramp(&[
    0xfefce8, 0xfef9c3, 0xfef08a, 0xfde047, 0xfacc15, 0xeab308, 0xca8a04, 0xa16207, 0x854d0e,
    0x713f12, 0x422006,
]);
pub const LIME: Ramp = Ramp(&[
    0xf7fee7, 0xecfccb, 0xd9f99d, 0xbef264, 0xa3e635, 0x84cc16, 0x65a30d, 0x4d7c0f, 0x3f6212,
    0x365314, 0x1a2e05,
]);
pub const GREEN: Ramp = Ramp(&[
    0xf0fdf4, 0xdcfce7, 0xbbf7d0, 0x86efac, 0x4ade80, 0x22c55e, 0x16a34a, 0x15803d, 0x166534,
    0x14532d, 0x052e16,
]);
pub const EMERALD: Ramp = Ramp(&[
    0xecfdf5, 0xd1fae5, 0xa7f3d0, 0x6ee7b7, 0x34d399, 0x10b981, 0x059669, 0x047857, 0x065f46,
    0x064e3b, 0x022c22,
]);
pub const TEAL: Ramp = Ramp(&[
    0xf0fdfa, 0xccfbf1, 0x99f6e4, 0x5eead4, 0x2dd4bf, 0x14b8a6, 0x0d9488, 0x0f766e, 0x115e59,
    0x134e4a, 0x042f2e,
]);
pub const CYAN: Ramp = Ramp(&[
    0xecfeff, 0xcffafe, 0xa5f3fc, 0x67e8f9, 0x22d3ee, 0x06b6d4, 0x0891b2, 0x0e7490, 0x155e75,
    0x164e63, 0x083344,
]);
pub const SKY: Ramp = Ramp(&[
    0xf0f9ff, 0xe0f2fe, 0xbae6fd, 0x7dd3fc, 0x38bdf8, 0x0ea5e9, 0x0284c7, 0x0369a1, 0x075985,
    0x0c4a6e, 0x082f49,
]);
pub const BLUE: Ramp = Ramp(&[
    0xeff6ff, 0xdbeafe, 0xbfdbfe, 0x93c5fd, 0x60a5fa, 0x3b82f6, 0x2563eb, 0x1d4ed8, 0x1e40af,
    0x1e3a8a, 0x172554,
]);
pub const INDIGO: Ramp = Ramp(&[
    0xeef2ff, 0xe0e7ff, 0xc7d2fe, 0xa5b4fc, 0x818cf8, 0x6366f1, 0x4f46e5, 0x4338ca, 0x3730a3,
    0x312e81, 0x1e1b4b,
]);
pub const VIOLET: Ramp = Ramp(&[
    0xf5f3ff, 0xede9fe, 0xddd6fe, 0xc4b5fd, 0xa78bfa, 0x8b5cf6, 0x7c3aed, 0x6d28d9, 0x5b21b6,
    0x4c1d95, 0x2e1065,
]);
pub const PURPLE: Ramp = Ramp(&[
    0xfaf5ff, 0xf3e8ff, 0xe9d5ff, 0xd8b4fe, 0xc084fc, 0xa855f7, 0x9333ea, 0x7e22ce, 0x6b21a8,
    0x581c87, 0x3b0764,
]);
pub const FUCHSIA: Ramp = Ramp(&[
    0xfdf4ff, 0xfae8ff, 0xf5d0fe, 0xf0abfc, 0xe879f9, 0xd946ef, 0xc026d3, 0xa21caf, 0x86198f,
    0x701a75, 0x4a044e,
]);
pub const PINK: Ramp = Ramp(&[
    0xfdf2f8, 0xfce7f3, 0xfbcfe8, 0xf9a8d4, 0xf472b6, 0xec4899, 0xdb2777, 0xbe185d, 0x9d174d,
    0x831843, 0x500724,
]);
pub const ROSE: Ramp = Ramp(&[
    0xfff1f2, 0xffe4e6, 0xfecdd3, 0xfda4af, 0xfb7185, 0xf43f5e, 0xe11d48, 0xbe123c, 0x9f1239,
    0x881337, 0x4c0519,
]);

// --- Material Design 2 ------------------------------------------------------
//
// Two families Tailwind simply does not have. Its warm neutrals (`stone`) are
// grey with a hint of warmth in them, which is right for concrete and wrong for
// *earth* -- and an arena is mostly ground. Material's `brown` is a real soil
// brown, and its `blue grey` is the cool counterpart, for rock and for the
// shadowed stone of a cave.
//
// Values from Material Design 2's published palette, via `shuhei/material-
// colors` (ISC, Copyright 2014 Shuhei Kagawa). Ten shades rather than eleven;
// the positions line up with Tailwind's up to 900.

pub const BROWN: Ramp = Ramp(&[
    0xefebe9, 0xd7ccc8, 0xbcaaa4, 0xa1887f, 0x8d6e63, 0x795548, 0x6d4c41, 0x5d4037, 0x4e342e,
    0x3e2723,
]);
pub const BLUE_GREY: Ramp = Ramp(&[
    0xeceff1, 0xcfd8dc, 0xb0bec5, 0x90a4ae, 0x78909c, 0x607d8b, 0x546e7a, 0x455a64, 0x37474f,
    0x263238,
]);

/// Every **coloured** family, for picking an accent by hue.
///
/// The greys are left out on purpose: an accent is the one thing in the frame
/// that is meant to be noticed, and a grey accent is a contradiction. They are
/// still there by name for the surfaces that want them.
pub const WHEEL: [&Ramp; 17] = [
    &RED, &ORANGE, &AMBER, &YELLOW, &LIME, &GREEN, &EMERALD, &TEAL, &CYAN, &SKY, &BLUE, &INDIGO,
    &VIOLET, &PURPLE, &FUCHSIA, &PINK, &ROSE,
];

/// The family on the wheel whose mid shade is nearest this hue, in turns.
pub fn nearest(hue: f32) -> &'static Ramp {
    // Wrapped first. A hue is a position on a circle and callers arrive with
    // "the light, plus most of a half turn", which runs off the end -- and an
    // unwrapped 1.2 makes the shorter-way-round arithmetic below come out
    // negative, so the family that wins is whichever one is furthest away.
    let hue = hue.rem_euclid(1.0);
    let mut best: (f32, &'static Ramp) = (f32::MAX, &RED);
    for ramp in WHEEL {
        let theirs = crate::tint::Lch::of(ramp.at(S500)).h;
        let apart = (theirs - hue).abs();
        let turn = apart.min(1.0 - apart);
        if turn < best.0 {
            best = (turn, ramp);
        }
    }
    best.1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tint::Lch;

    #[test]
    fn a_ramp_runs_light_to_dark_without_turning_round() {
        // What makes a shade number mean something: 500 is darker than 400
        // everywhere, so a material can be moved a step on any family and land
        // where it meant to. Also catches a hex typed wrong.
        for ramp in WHEEL {
            for shade in 0..10 {
                let (a, b) = (Lch::of(ramp.at(shade)).l, Lch::of(ramp.at(shade + 1)).l);
                assert!(a > b, "shade {shade} is darker than the one after it");
            }
        }
    }

    #[test]
    fn the_wheel_covers_the_wheel() {
        // The accent asks for a hue and takes the nearest family, so the worst
        // gap between neighbours is how far from its wish it can land. A sixth
        // of a turn would be a different colour; a twentieth is a shade.
        let mut hues: Vec<f32> = WHEEL.iter().map(|r| Lch::of(r.at(S500)).h).collect();
        hues.sort_by(f32::total_cmp);
        let worst = hues
            .windows(2)
            .map(|p| p[1] - p[0])
            .chain(std::iter::once(1.0 - hues[hues.len() - 1] + hues[0]))
            .fold(0.0f32, f32::max);
        assert!(
            worst < 0.14,
            "the widest gap on the wheel is {worst:.3} of a turn"
        );
    }

    #[test]
    fn a_hue_off_the_end_of_the_wheel_still_lands_where_it_should() {
        // Callers ask for "the light's hue plus 0.42", which runs past 1.0 for
        // more than half of them. Unwrapped, the shorter-way-round arithmetic
        // goes negative and picks the family on the far side of the wheel.
        for turn in [0.1f32, 0.6, 1.1, 1.9, -0.3] {
            let got = Lch::of(nearest(turn).at(S500)).h;
            let want = turn.rem_euclid(1.0);
            let apart = (got - want).abs();
            assert!(
                apart.min(1.0 - apart) < 0.08,
                "asked for {want:.2} of a turn and got {got:.2}"
            );
        }
    }

    #[test]
    fn the_nearest_family_is_actually_the_nearest() {
        for ramp in WHEEL {
            let hue = Lch::of(ramp.at(S500)).h;
            let got = Lch::of(nearest(hue).at(S500)).h;
            assert!((got - hue).abs() < 1e-6, "asked for {hue} and got {got}");
        }
    }
}
