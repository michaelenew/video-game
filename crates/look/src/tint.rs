//! Colour arithmetic that behaves the way the eye does.
//!
//! Every gradient in the art direction is a blend between two colours, and
//! blending in RGB goes through mud. Averaging the channels of orange and blue
//! lands near the middle of the cube, which is grey; darkening a colour by
//! scaling its channels desaturates it as it goes, so a "darker blue" made that
//! way is a different colour. Neither is a taste failure -- sRGB is a display
//! encoding, and the straight line between two points in it is not the path the
//! eye reads as *between*.
//!
//! **Oklab** (Björn Ottosson, 2020) is built so that distance approximates
//! perceived difference and a straight line looks like an even fade. Its polar
//! form, **OkLCh**, splits a colour into lightness, chroma (how colourful) and
//! hue -- the three things worth moving separately, and the three the style
//! guide below actually talks about.
//!
//! It supersedes CIELAB here mainly because CIELAB's hue lines bend: darkening
//! a blue in CIELAB turns it purple, which is the single most notorious
//! artefact in the older space and exactly the operation a sky gradient does
//! most.

/// Linear RGB. Everything computes here; sRGB exists only at the boundary.
pub type Rgb = [f32; 3];

/// sRGB's transfer function. Not a gamma of 2.2 -- a linear toe near black and
/// a 2.4 power curve above it. The toe matters for the dark end of every
/// gradient, which on a night sky is most of it.
pub fn to_linear(c: f32) -> f32 {
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub fn to_srgb(c: f32) -> f32 {
    if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

pub fn linear(c: Rgb) -> Rgb {
    [to_linear(c[0]), to_linear(c[1]), to_linear(c[2])]
}

pub fn srgb(c: Rgb) -> Rgb {
    [to_srgb(c[0]), to_srgb(c[1]), to_srgb(c[2])]
}

/// Oklab: `[L, a, b]`.
pub fn to_oklab(c: Rgb) -> [f32; 3] {
    let l = 0.412_221_47 * c[0] + 0.536_332_54 * c[1] + 0.051_445_995 * c[2];
    let m = 0.211_903_5 * c[0] + 0.680_699_5 * c[1] + 0.107_396_96 * c[2];
    let s = 0.088_302_46 * c[0] + 0.281_718_84 * c[1] + 0.629_978_7 * c[2];
    let (l, m, s) = (cbrt(l), cbrt(m), cbrt(s));
    [
        0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    ]
}

pub fn from_oklab(c: [f32; 3]) -> Rgb {
    let l_ = c[0] + 0.396_337_78 * c[1] + 0.215_803_76 * c[2];
    let m_ = c[0] - 0.105_561_346 * c[1] - 0.063_854_17 * c[2];
    let s_ = c[0] - 0.089_484_18 * c[1] - 1.291_485_5 * c[2];
    let (l, m, s) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);
    [
        4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s,
        -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s,
        -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s,
    ]
}

/// Cube root that keeps the sign. The forward transform feeds it values that
/// are non-negative in theory and very slightly negative in practice, from
/// rounding in the matrix; `powf(1/3)` on a negative is NaN and the pixel turns
/// black.
fn cbrt(x: f32) -> f32 {
    if x < 0.0 {
        -(-x).powf(1.0 / 3.0)
    } else {
        x.powf(1.0 / 3.0)
    }
}

/// Lightness, chroma, hue in turns.
///
/// Hue in **turns** rather than degrees, because "a sixth of a turn round the
/// wheel" is the sentence the style rules actually want to write.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lch {
    pub l: f32,
    pub c: f32,
    pub h: f32,
}

impl Lch {
    pub const fn new(l: f32, c: f32, h: f32) -> Lch {
        Lch { l, c, h }
    }

    /// From a colour written the way a person writes one: sRGB, 0 to 1.
    pub fn of(rgb: Rgb) -> Lch {
        let lab = to_oklab(linear(rgb));
        let c = (lab[1] * lab[1] + lab[2] * lab[2]).sqrt();
        let h = lab[2].atan2(lab[1]) / std::f32::consts::TAU;
        Lch {
            l: lab[0],
            c,
            h: h.rem_euclid(1.0),
        }
    }

    /// Back to sRGB, with chroma given up until the colour is one a screen can
    /// show.
    ///
    /// Most of OkLCh is outside sRGB -- a vivid yellow at the lightness of a
    /// vivid blue does not exist on a monitor. Clamping the channels instead
    /// would shift the hue, which is the one property every rule here is built
    /// on holding, so chroma gives way and hue and lightness survive.
    pub fn rgb(self) -> Rgb {
        let lab = |c: f32| {
            let a = self.h * std::f32::consts::TAU;
            from_oklab([self.l, c * a.cos(), c * a.sin()])
        };
        if in_gamut(lab(self.c)) {
            return clamp(srgb(lab(self.c)));
        }
        let (mut lo, mut hi) = (0.0, self.c);
        for _ in 0..16 {
            let mid = 0.5 * (lo + hi);
            if in_gamut(lab(mid)) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        clamp(srgb(lab(lo)))
    }

    pub fn lighter(self, by: f32) -> Lch {
        Lch {
            l: (self.l + by).clamp(0.0, 1.0),
            ..self
        }
    }

    pub fn vivid(self, by: f32) -> Lch {
        Lch {
            c: (self.c * by).max(0.0),
            ..self
        }
    }

    /// Turned round the wheel, in turns.
    pub fn turned(self, by: f32) -> Lch {
        Lch {
            h: (self.h + by).rem_euclid(1.0),
            ..self
        }
    }

    /// Pulled toward another hue, without touching lightness or chroma.
    ///
    /// The accent rule's workhorse: it is how one colour is laid over a scene
    /// of unrelated ones without flattening them into it.
    pub fn toward_hue(self, hue: f32, amount: f32) -> Lch {
        let mut gap = hue - self.h;
        if gap > 0.5 {
            gap -= 1.0;
        } else if gap < -0.5 {
            gap += 1.0;
        }
        Lch {
            h: (self.h + gap * amount).rem_euclid(1.0),
            ..self
        }
    }
}

fn in_gamut(c: Rgb) -> bool {
    c.iter().all(|v| (-1e-4..=1.000_1).contains(v))
}

fn clamp(c: Rgb) -> Rgb {
    [
        c[0].clamp(0.0, 1.0),
        c[1].clamp(0.0, 1.0),
        c[2].clamp(0.0, 1.0),
    ]
}

/// Blend two sRGB colours perceptually, taking the short way round the hue
/// wheel.
///
/// The short way matters: a blend from a hue just under one to one just over
/// zero is two neighbouring reds, and interpolating the numbers directly
/// sweeps the whole spectrum backwards.
pub fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    // The ends come back untouched. A round trip through Oklab is accurate to
    // about a part in ten thousand, which is invisible and is still enough to
    // make "the horizon is the colour it was given" false -- and the fog
    // depends on that promise, since it fades to the number the arena wrote
    // rather than to whatever the gradient computed.
    if t <= 0.0 {
        return a;
    }
    if t >= 1.0 {
        return b;
    }
    let (x, y) = (Lch::of(a), Lch::of(b));
    // Through an achromatic colour, hue is meaningless and whatever number it
    // happens to carry would drag the blend somewhere arbitrary.
    let hue = if x.c < 1e-3 {
        y.h
    } else if y.c < 1e-3 {
        x.h
    } else {
        x.toward_hue(y.h, t).h
    };
    Lch::new(x.l + (y.l - x.l) * t, x.c + (y.c - x.c) * t, hue).rgb()
}
