//! Colour, in a space where interpolation means something.
//!
//! Three things here, and each replaces a decision a person would otherwise
//! have to make by eye.
//!
//! ## Oklab, not RGB and not HSV
//!
//! Blending two colours by averaging their sRGB channels goes through grey.
//! Orange to blue passes through mud; red to green passes through brown. That
//! is not a taste failure, it is arithmetic: sRGB is a display encoding, and
//! the straight line between two points in it does not follow the path the eye
//! reads as "between".
//!
//! **Oklab** (Björn Ottosson, 2020) is a colour space built so that Euclidean
//! distance approximates perceived difference, and so that a straight line
//! between two colours looks like an even fade. Its polar form **OkLCh** splits
//! a colour into lightness, chroma (how colourful) and hue (which colour),
//! which are the three things worth talking about separately.
//!
//! It supersedes CIELAB for this job mainly because CIELAB's hue lines bend:
//! darkening a blue in CIELAB turns it purple, which is the single most
//! notorious artefact in the older space.
//!
//! Everything in this crate ramps and blends in Oklab and converts to sRGB
//! once, at the end.
//!
//! ## Six class colours are a computation, not a choice
//!
//! The roster needs six colours that are equally vivid, equally bright, and
//! mutually distinguishable — including to the roughly one man in twelve with
//! red-green colour blindness. Picking those by eye is hard and checking them
//! is harder. In OkLCh it is one line: hold lightness and chroma, space the
//! hues evenly around the circle. Equal L and C is exactly what "equally
//! vivid and equally bright" means in a perceptual space. See [`palette`].
//!
//! [`palette`]: crate::palette
//!
//! ## Fire's colour is physics, not a colour picker
//!
//! A luminous flame is glowing soot, and glowing soot is close enough to a
//! **blackbody** — an object that emits purely because of its temperature —
//! that the standard curve fits it well. So the colour of fire is a function
//! of one number, its temperature in kelvin, and that number is meaningful:
//! 1200 K is a dull ember, 1800 K the orange of a wood fire, 3000 K the warm
//! white of a filament bulb, 6500 K daylight.
//!
//! That is worth more than it sounds. It collapses "what colour is the fire"
//! into "how hot is the fire", which is a question the *game* already has an
//! opinion about — a spell that is meant to read as hotter simply is. And the
//! gradient from core to edge falls out of a temperature falloff rather than
//! being three colours somebody chose.
//!
//! The honest limit: the blue at the base of a gas flame is **not** thermal.
//! It is light emitted by excited molecular fragments as they burn, which no
//! temperature curve will produce. Where a design wants that blue it is added
//! deliberately as a tint, not expected to appear.

/// Linear RGB. Everything computes here; sRGB exists only at the boundary.
pub type Rgb = [f32; 3];

/// sRGB's transfer function. Not a gamma of 2.2 -- a linear toe near black and
/// a 2.4 power curve above it. The toe matters for the dark end of every ramp
/// in this crate, which is most of a stone or a shadow.
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Oklab: `[L, a, b]`. `L` runs 0 to 1, `a` and `b` are unbounded in principle
/// and about -0.4 to 0.4 for anything a screen can show.
pub type Lab = [f32; 3];

pub fn linear_to_oklab(c: Rgb) -> Lab {
    let l = 0.412_221_47 * c[0] + 0.536_332_54 * c[1] + 0.051_445_995 * c[2];
    let m = 0.211_903_5 * c[0] + 0.680_699_5 * c[1] + 0.107_396_96 * c[2];
    let s = 0.088_302_46 * c[0] + 0.281_718_84 * c[1] + 0.629_978_7 * c[2];

    let l_ = cbrt(l);
    let m_ = cbrt(m);
    let s_ = cbrt(s);

    [
        0.210_454_26 * l_ + 0.793_617_8 * m_ - 0.004_072_047 * s_,
        1.977_998_5 * l_ - 2.428_592_2 * m_ + 0.450_593_7 * s_,
        0.025_904_037 * l_ + 0.782_771_77 * m_ - 0.808_675_77 * s_,
    ]
}

pub fn oklab_to_linear(c: Lab) -> Rgb {
    let l_ = c[0] + 0.396_337_78 * c[1] + 0.215_803_76 * c[2];
    let m_ = c[0] - 0.105_561_346 * c[1] - 0.063_854_17 * c[2];
    let s_ = c[0] - 0.089_484_18 * c[1] - 1.291_485_5 * c[2];

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    [
        4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s,
        -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s,
        -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s,
    ]
}

/// Cube root that keeps the sign. The Oklab transform feeds it values that are
/// non-negative in theory and very slightly negative in practice, from rounding
/// in the matrix above; `powf(1/3)` on a negative returns NaN and the pixel
/// turns black.
fn cbrt(x: f32) -> f32 {
    if x < 0.0 {
        -(-x).powf(1.0 / 3.0)
    } else {
        x.powf(1.0 / 3.0)
    }
}

/// OkLCh: lightness, chroma, hue in turns.
///
/// Hue in **turns** rather than degrees or radians to match the convention the
/// simulation already uses for angles, and because "six classes, a sixth of a
/// turn apart" is the sentence you actually want to write.
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

    pub fn to_oklab(self) -> Lab {
        let a = self.h * std::f32::consts::TAU;
        [self.l, self.c * a.cos(), self.c * a.sin()]
    }

    pub fn from_oklab(lab: Lab) -> Lch {
        let c = (lab[1] * lab[1] + lab[2] * lab[2]).sqrt();
        let mut h = lab[2].atan2(lab[1]) / std::f32::consts::TAU;
        if h < 0.0 {
            h += 1.0;
        }
        Lch { l: lab[0], c, h }
    }

    /// Linear RGB, with chroma reduced until the colour is one a screen can
    /// actually show.
    ///
    /// Most of OkLCh is outside sRGB -- a vivid yellow at the lightness of a
    /// vivid blue does not exist on a monitor. Clamping the channels instead
    /// would shift the hue, which is the one property the palette is built on
    /// holding constant, so chroma gives way and hue and lightness survive.
    pub fn to_linear(self) -> Rgb {
        let mut lo = 0.0;
        let mut hi = self.c;
        if in_gamut(oklab_to_linear(self.to_oklab())) {
            return oklab_to_linear(self.to_oklab());
        }
        // Twenty halvings puts the bracket well below a display's precision, at
        // the same cost every time -- the same reasoning as the curve solver in
        // `sim::curve`.
        for _ in 0..20 {
            let mid = 0.5 * (lo + hi);
            if in_gamut(oklab_to_linear(Lch::new(self.l, mid, self.h).to_oklab())) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let rgb = oklab_to_linear(Lch::new(self.l, lo, self.h).to_oklab());
        [
            rgb[0].clamp(0.0, 1.0),
            rgb[1].clamp(0.0, 1.0),
            rgb[2].clamp(0.0, 1.0),
        ]
    }
}

fn in_gamut(c: Rgb) -> bool {
    const SLACK: f32 = 1e-4;
    c.iter().all(|&v| (-SLACK..=1.0 + SLACK).contains(&v))
}

/// A colour ramp: up to four stops, blended in Oklab.
///
/// Four because three cannot express a highlight *and* a shadow around a base,
/// and five has never been needed. Stops are evenly spaced; an uneven ramp is
/// what the field's own shaping is for.
#[derive(Clone, Copy, Debug)]
pub struct Ramp {
    pub stops: [Lch; 4],
    pub used: u8,
}

impl Ramp {
    pub const fn two(a: Lch, b: Lch) -> Ramp {
        Ramp {
            stops: [a, b, b, b],
            used: 2,
        }
    }

    pub const fn three(a: Lch, b: Lch, c: Lch) -> Ramp {
        Ramp {
            stops: [a, b, c, c],
            used: 3,
        }
    }

    pub const fn four(a: Lch, b: Lch, c: Lch, d: Lch) -> Ramp {
        Ramp {
            stops: [a, b, c, d],
            used: 4,
        }
    }

    /// The colour at `t`, clamped to 0..1, in linear RGB.
    pub fn at(&self, t: f32) -> Rgb {
        let used = self.used.max(2) as usize;
        let t = t.clamp(0.0, 1.0) * (used - 1) as f32;
        let i = (t.floor() as usize).min(used - 2);
        let f = t - i as f32;

        // Hue is interpolated the short way round the circle. Without that, a
        // ramp from a hue just under 1.0 to one just over 0.0 travels the long
        // way and sweeps the entire spectrum -- the classic rainbow-smear bug.
        let a = self.stops[i];
        let b = self.stops[i + 1];
        let mut dh = b.h - a.h;
        if dh > 0.5 {
            dh -= 1.0;
        } else if dh < -0.5 {
            dh += 1.0;
        }
        Lch::new(
            a.l + (b.l - a.l) * f,
            a.c + (b.c - a.c) * f,
            (a.h + dh * f).rem_euclid(1.0),
        )
        .to_linear()
    }
}

/// The colour of a blackbody at `kelvin`, normalised so the brightest channel
/// is 1. Multiply by whatever intensity the effect wants.
///
/// Uses the standard cubic fit to the Planckian locus -- the path the colour of
/// a heated object traces as it warms -- which covers embers through to a star.
///
/// **Clamped at 1667 K at the bottom, and that is not conservatism.** The fit
/// is published for 1667 K upward and it *turns around* below that: it hands
/// back the same chromaticity for 1100 K as for 1500 K, so a cooling ember
/// would stop changing colour partway down and nothing would say why. The
/// clamp makes the floor visible instead of leaving a silently wrong answer in
/// the range a dying flame actually lives in.
///
/// The practical consequence for the game is small and worth knowing: the
/// coolest fire available is a deep orange-red, and an ember that fades past
/// that fades by getting *dimmer*, not redder. Which is what embers do.
pub const COOLEST: f32 = 1667.0;

// The coefficients below are the published fit and are kept at their published
// precision, even though `f32` cannot hold all of it. Trimming them to what
// fits would make the constants un-greppable against the source they came
// from, and the rounded value is identical either way.
#[allow(clippy::excessive_precision)]
pub fn blackbody(kelvin: f32) -> Rgb {
    let t = kelvin.clamp(COOLEST, 25_000.0);
    let inv = 1000.0 / t;

    // Chromaticity, as x and y on the CIE 1931 diagram: where on the horseshoe
    // of visible colours this temperature sits.
    let x = if t < 4000.0 {
        -0.266_123_9 * inv.powi(3) - 0.234_358_9 * inv.powi(2) + 0.877_695_6 * inv + 0.179_910
    } else {
        -3.025_846_9 * inv.powi(3) + 2.107_037_9 * inv.powi(2) + 0.222_634_7 * inv + 0.240_390
    };
    let y = if t < 2222.0 {
        -1.106_381_4 * x.powi(3) - 1.348_110_2 * x.powi(2) + 2.185_558_3 * x - 0.202_196_83
    } else if t < 4000.0 {
        -0.954_947_6 * x.powi(3) - 1.374_185_9 * x.powi(2) + 2.091_370_1 * x - 0.167_488_67
    } else {
        3.081_758 * x.powi(3) - 5.873_386_7 * x.powi(2) + 3.751_130 * x - 0.370_014_83
    };

    // CIE xyY at unit luminance to XYZ, then XYZ to linear sRGB.
    let big_y = 1.0;
    let big_x = x * big_y / y;
    let big_z = (1.0 - x - y) * big_y / y;

    let rgb = [
        3.240_454_2 * big_x - 1.537_138_5 * big_y - 0.498_531_4 * big_z,
        -0.969_266 * big_x + 1.876_010_8 * big_y + 0.041_556 * big_z,
        0.055_643_4 * big_x - 0.204_025_9 * big_y + 1.057_225_2 * big_z,
    ];
    let peak = rgb[0].max(rgb[1]).max(rgb[2]).max(1e-6);
    [
        (rgb[0] / peak).clamp(0.0, 1.0),
        (rgb[1] / peak).clamp(0.0, 1.0),
        (rgb[2] / peak).clamp(0.0, 1.0),
    ]
}

/// Perceived difference between two colours, as a distance in Oklab.
///
/// Roughly: below 0.02 is a shade of the same colour, above 0.10 reads as a
/// different colour at a glance. Used by the palette tests rather than at
/// runtime.
pub fn difference(a: Rgb, b: Rgb) -> f32 {
    let (a, b) = (linear_to_oklab(a), linear_to_oklab(b));
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// What a colour looks like to someone with the common form of red-green
/// colour blindness.
///
/// Deuteranomaly -- a shifted green response -- affects around 6% of men, so a
/// roster whose colours collapse under it is a roster a meaningful fraction of
/// players cannot read. This is the Machado, Oliveira and Fernandes (2009)
/// matrix at full severity, applied in linear RGB. A simulation rather than a
/// diagnosis: it says whether two colours stay apart, which is the only
/// question being asked.
pub fn deuteranope(c: Rgb) -> Rgb {
    [
        0.367_322 * c[0] + 0.860_646 * c[1] - 0.227_968 * c[2],
        0.280_085 * c[0] + 0.672_501 * c[1] + 0.047_413 * c[2],
        -0.011_820 * c[0] + 0.042_940 * c[1] + 0.968_881 * c[2],
    ]
}
