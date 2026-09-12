//! The whole lighting rig, from one number.
//!
//! **How high the sun is, and everything else follows.** Its direction, its
//! colour, how bright it is, what colour the sky is, how much light comes back
//! down out of that sky, and what colour the haze is at the far wall. Six
//! answers, one parameter, and the relationships between them are physics
//! rather than taste.
//!
//! That is the same trade as fire's temperature in [`crate::color`] and it
//! pays better here, because lighting has more numbers to get wrong. The
//! previous rig was three hand-tuned magnitudes -- a key light, a fill and an
//! ambient -- with no relationship to each other at all, so moving one meant
//! re-judging the other two by eye. Worse, they had drifted into compensating
//! for placeholder materials: the key sat at 11,000 lux to make surfaces of
//! 0.02 reflectance read, and when the materials became physical the arena
//! blew out to near-white.
//!
//! ## Why the sun reddens, in one equation
//!
//! Air scatters short wavelengths far more strongly than long ones --
//! Rayleigh's result, and the reason the sky is blue at all. Scattering
//! strength goes roughly as the inverse fourth power of wavelength, so blue
//! light is removed from a beam about three times faster than red.
//!
//! Sunlight reaching the ground has therefore been filtered, and how much
//! depends on how much air it crossed. Overhead, that is one atmosphere's
//! worth and the sun is very slightly warm. At the horizon the same beam
//! travels through nearly forty times as much air, almost all the blue is
//! gone, and what is left is red.
//!
//! So: **compute the air mass from the sun's elevation, run Beer's law per
//! colour channel, and sunset comes out.** Nobody picks the colour of the
//! sunset, and nobody can pick it wrong.
//!
//! ## What this does not do
//!
//! It does not draw the sky. Bevy renders that with Hillaire's atmospheric
//! scattering model, which is a considerably better piece of work than
//! anything that belongs in this crate and is exactly the sort of thing
//! `architecture.md` says to take from the engine rather than write.
//!
//! The two are not a second implementation of one answer. The engine computes
//! **in-scattering** -- the light the air sends toward the eye, which is the
//! image of the sky. This computes **transmittance** -- the light that
//! survives the trip to the ground, which is the lamp. Different quantities
//! out of the same physics, and the renderer has no way to hand back the
//! second one in a form a `DirectionalLight` can wear.

use crate::color::Rgb;

/// Rayleigh optical depth of the whole atmosphere at sea level, looking
/// straight up, for each of the three display primaries.
///
/// From the standard `0.008735 * lambda^-4.08` fit, evaluated at roughly where
/// sRGB's red, green and blue sit -- 610, 550 and 465 nanometres. The exponent
/// is 4.08 rather than a clean 4 because real air is not quite an ideal
/// Rayleigh scatterer; the difference is small and free to keep.
///
/// The ratio is the interesting part: blue is attenuated three times as hard
/// as red. Every warm sunset in this file comes from that one fact.
pub const RAYLEIGH_ZENITH: Rgb = [0.0656, 0.1001, 0.1986];

/// Illuminance of the sun outside the atmosphere, in lux, on a surface facing
/// it. The solar constant in photometric units.
pub const SOLAR_ILLUMINANCE: f32 = 128_000.0;

/// How much air a beam crosses, relative to straight up.
///
/// The naive answer is `1 / sin(elevation)`, which is right well above the
/// horizon and diverges to infinity at it -- where the true figure is about
/// 38, because the atmosphere is a shell around a sphere rather than a flat
/// slab. This is Kasten and Young's 1989 fit, which is the standard correction
/// and behaves all the way down to the horizon.
///
/// It matters because the horizon is exactly where the interesting light is.
pub fn air_mass(elevation_deg: f32) -> f32 {
    let e = elevation_deg.max(-2.0);
    let sin = e.to_radians().sin();
    let correction = 0.50572 * (e + 6.079_95).powf(-1.6364);
    1.0 / (sin + correction).max(1e-3)
}

/// What fraction of each colour survives the trip down, at this elevation.
///
/// Beer's law: a beam crossing `m` atmospheres of optical depth `tau` keeps
/// `exp(-tau * m)` of itself. Per channel, so the answer is a colour rather
/// than a number, which is the entire point.
pub fn transmittance(elevation_deg: f32) -> Rgb {
    let m = air_mass(elevation_deg);
    [
        (-RAYLEIGH_ZENITH[0] * m).exp(),
        (-RAYLEIGH_ZENITH[1] * m).exp(),
        (-RAYLEIGH_ZENITH[2] * m).exp(),
    ]
}

/// A complete lighting setup: what the renderer needs to be told.
#[derive(Clone, Copy, Debug)]
pub struct Sky {
    /// Unit vector pointing **from** the sun **toward** the scene, so it can
    /// be handed straight to a directional light.
    pub sun_direction: [f32; 3],
    /// The sun's colour, normalised so its brightest channel is one. Warm
    /// overhead, red at the horizon.
    pub sun_color: Rgb,
    /// Illuminance on a surface facing the sun, in lux.
    pub sun_illuminance: f32,
    /// What colour the sky is, and therefore what colour the light bouncing
    /// down out of it is.
    pub sky_color: Rgb,
    /// How much light the sky alone delivers, in lux.
    pub sky_illuminance: f32,
}

impl Sky {
    /// The rig at a given sun elevation and compass bearing, both in degrees.
    ///
    /// Elevation 90 is noon, 0 is sunset, and negative is below the horizon.
    pub fn at(elevation_deg: f32, azimuth_deg: f32) -> Sky {
        let t = transmittance(elevation_deg);

        // Normalised so the renderer's colour and intensity stay separable:
        // one says what shade the light is, the other how much there is. Fold
        // them together and every change of time of day is also a change of
        // exposure.
        let peak = t[0].max(t[1]).max(t[2]).max(1e-6);
        let sun_color = [t[0] / peak, t[1] / peak, t[2] / peak];

        // Energy actually arriving. The cosine is the surface's own tilt and
        // belongs to the renderer, so what is left is the beam's own strength
        // times what survived the trip -- averaged across the channels, since
        // illuminance is one number.
        let survived = (t[0] + t[1] + t[2]) / 3.0;
        let sun_illuminance = SOLAR_ILLUMINANCE * survived * horizon_fade(elevation_deg);

        // **What the air removed from the beam is what the sky is made of.**
        // Not a separate model and not a guess: a channel scattered hard out
        // of the direct path is a channel the sky is full of, which is why the
        // sky is blue.
        //
        // But the scattering has to be measured on the way *to the sky*, not
        // on the way to the sun. The first version used the sun's own air mass
        // for both, and at sunset that runs to thirty-eight atmospheres, where
        // every channel is fully scattered and the formula saturates to a flat
        // white sky. Overhead the air is one atmosphere thick whatever the sun
        // is doing.
        //
        // So the sky's own colour is fixed Rayleigh blue, **lit by a beam that
        // has already reddened**. Only partly, because plenty of the sky is
        // lit along shorter paths than the one straight to the sun -- which is
        // exactly why dusk overhead is violet rather than orange.
        const LIT_BY_THE_BEAM: f32 = 0.6;
        let zenith = [
            1.0 - (-RAYLEIGH_ZENITH[0]).exp(),
            1.0 - (-RAYLEIGH_ZENITH[1]).exp(),
            1.0 - (-RAYLEIGH_ZENITH[2]).exp(),
        ];
        let mut scattered = [0.0f32; 3];
        for i in 0..3 {
            let incoming = 1.0 + (sun_color[i] - 1.0) * LIT_BY_THE_BEAM;
            scattered[i] = zenith[i] * incoming;
        }
        let sky_peak = scattered[0].max(scattered[1]).max(scattered[2]).max(1e-6);
        let sky_color = [
            scattered[0] / sky_peak,
            scattered[1] / sky_peak,
            scattered[2] / sky_peak,
        ];
        // Roughly a sixth of the direct beam comes back down as skylight on an
        // ordinary clear day -- around 20,000 lux of a 110,000 lux noon, which
        // is the figure daylight tables give. It is a coefficient rather than a
        // derivation because the real answer needs the whole sky integrated,
        // which is the calculation the engine is already doing for the image.
        let sky_illuminance = sun_illuminance * 0.18 + 60.0;

        let el = elevation_deg.to_radians();
        let az = azimuth_deg.to_radians();
        // Pointing from the sun into the scene, so it is already the direction
        // a directional light travels.
        let sun_direction = [-el.cos() * az.cos(), -el.sin(), -el.cos() * az.sin()];

        Sky {
            sun_direction,
            sun_color,
            sun_illuminance,
            sky_color,
            sky_illuminance,
        }
    }
}

/// How much of the sun is still above the horizon, roughly.
///
/// Without this the direct beam switches off the instant elevation crosses
/// zero, and dusk arrives as a single frame in which the key light vanishes.
/// The sun has an angular diameter of about half a degree and the atmosphere
/// refracts it a little further, so the real thing takes a couple of degrees
/// to set. Smoothstepped over that, and the lights go out the way they should.
fn horizon_fade(elevation_deg: f32) -> f32 {
    let t = ((elevation_deg + 1.5) / 3.0).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
