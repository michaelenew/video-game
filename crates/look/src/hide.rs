//! **A creature's hide, from one colour** (2026-10-23).
//!
//! A creature's look names a colour per kind of part (`game::species`), and
//! the first grown bodies multiplied that colour up and down for the back and
//! the belly. On a near-black armour like the Ridgeback's, that is near-black
//! made a little darker or a little less dark: the hide read as one flat
//! shape. A hide is a palette, as an arena's sky is: the colour a creature is
//! named for is its **flank**, and the rest is derived from it here, in
//! perceptual terms (`tint::Lch`), so any species -- a bred one too -- gets a
//! whole hide from the one colour it names.
//!
//! What every one is made of, from how large animals are coloured:
//!
//! - **Countershading**: a darker, cooler back and a paler, warmer belly. Sun
//!   from above lights a back and shades a belly; a hide painted the other
//!   way cancels it, and nearly every large animal is.
//! - **Bands** across the back, a little darker than it, as a pattern is.
//! - **Light on its ridges, dark in its creases** -- what a painter puts on a
//!   form so it reads, and what the cast's painted textures carry too.
//!
//! The flank is lifted to a middle lightness first: a colour near black has no
//! room below it for a back and the eye cannot read a pattern in it.

use crate::tint::{Lch, Rgb, mix};

/// One hide, sRGB.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hide {
    pub back: Rgb,
    pub flank: Rgb,
    pub belly: Rgb,
    pub band: Rgb,
    pub ridge: Rgb,
    pub crease: Rgb,
}

/// The lightness (Oklab, 0 to 1) a flank is lifted to at least.
const FLANK_AT_LEAST: f32 = 0.5;
/// The belly's own colour, before the flank's hue is mixed into it: a warm
/// pale buff.
const BUFF: Rgb = [0.86, 0.78, 0.62];

impl Hide {
    /// The hide a creature named for `flank` (sRGB) wears.
    pub fn of(flank: Rgb) -> Hide {
        let named = Lch::of(flank);
        let flank = Lch {
            l: named.l.max(FLANK_AT_LEAST),
            ..named
        };
        let back = flank.lighter(-0.17).vivid(1.15).turned(0.02);
        let band = back.lighter(-0.09).vivid(1.2);
        let flank_rgb = flank.rgb();
        Hide {
            back: back.rgb(),
            flank: flank_rgb,
            belly: mix(flank_rgb, BUFF, 0.6),
            band: band.rgb(),
            ridge: flank.lighter(0.12).vivid(0.8).toward_hue(0.22, 0.25).rgb(),
            crease: back.lighter(-0.12).rgb(),
        }
    }

    /// The colour at a point of the hide, sRGB: `up` how much its surface
    /// faces the sky (0 straight down, 1 straight up), `band` how deep in a
    /// band it is (0 to 1), `bend` its ridge (-1) or crease (+1), `mottle` a
    /// slow noise (0 to 1), `plate` how far from the seam between two
    /// armour plates (0 on a seam, 1 at a plate's middle).
    pub fn at(&self, up: f32, band: f32, bend: f32, mottle: f32, plate: f32) -> Rgb {
        let belly = smooth(0.42, 0.05, up);
        let back = smooth(0.55, 0.95, up);
        let mut c = mix(self.flank, self.belly, belly);
        c = mix(c, self.back, back);
        c = mix(c, self.band, band * smooth(0.3, 0.7, up) * 0.85);
        if bend < 0.0 {
            c = mix(c, self.ridge, (-bend).min(1.0) * 0.6);
        } else {
            c = mix(c, self.crease, bend.min(1.0) * 0.55);
        }
        // Armour plates on what faces the sky: dark seams between them, and
        // each plate's middle a little lighter, as a worn plate is.
        let armour = smooth(0.35, 0.7, up);
        c = mix(
            c,
            self.crease,
            (1.0 - smooth(0.0, 0.35, plate)) * armour * 0.8,
        );
        c = mix(c, self.ridge, smooth(0.6, 1.0, plate) * armour * 0.18);
        let k = 0.9 + 0.2 * mottle;
        [c[0] * k, c[1] * k, c[2] * k]
    }
}

/// Smoothstep from `a` to `b` (either way round).
fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn l(c: Rgb) -> f32 {
        Lch::of(c).l
    }

    #[test]
    fn a_dark_creature_still_gets_a_hide_with_room_in_it() {
        // The Ridgeback's armour.
        let h = Hide::of([0.24, 0.27, 0.23]);
        assert!(l(h.back) < l(h.flank) && l(h.flank) < l(h.belly));
        assert!(l(h.band) < l(h.back));
        assert!(l(h.belly) - l(h.back) > 0.2, "{h:?}");
        assert!(l(h.ridge) > l(h.flank) && l(h.crease) < l(h.back));
    }

    #[test]
    fn the_back_is_darker_than_the_belly_wherever_it_is_painted() {
        let h = Hide::of([0.5, 0.3, 0.2]);
        assert!(l(h.at(1.0, 0.0, 0.0, 0.5, 0.5)) < l(h.at(0.0, 0.0, 0.0, 0.5, 0.5)));
    }
}
