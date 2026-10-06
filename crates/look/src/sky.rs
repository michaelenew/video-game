//! What is behind everything, and how far the eye reaches through it.
//!
//! Every arena used to set one flat colour. That is honest and it costs the
//! game two things it needs.
//!
//! **A flat sky has no horizon**, and the horizon is the only thing in view
//! that says which way is level. Without it a jump course sixty metres up looks
//! exactly like one on the ground: an island is the same size on screen either
//! way, and nothing in frame disagrees.
//!
//! **A flat sky has no distance.** Everything far is as crisp as everything
//! near, so a spire two hundred metres off reads as a small spire close by. Air
//! is not clear -- it scatters, and distant things go pale and low-contrast as
//! they recede. That one effect, **aerial perspective**, is the strongest depth
//! cue in any painting, and it is why a drop reads as a drop rather than as a
//! dark floor somebody slid under the level.
//!
//! So a sky is a gradient and a reach, and the two share their colour: things
//! fade into exactly what is behind them, which is what makes the fade read as
//! air rather than as a grey wash over the picture.

use crate::tint;

/// An arena's sky.
#[derive(Clone, Copy, Debug)]
pub struct Sky {
    /// Straight up.
    pub zenith: [f32; 3],
    /// At eye level, all the way round. Everything distant fades into this, so
    /// it is the colour the whole scene is tuned against.
    pub horizon: [f32; 3],
    /// Below the horizon: the haze a long drop disappears into.
    pub ground: [f32; 3],
    /// A warmth low in the sky toward the sun's bearing. `None` for a cave, an
    /// overcast, or anywhere the light has no direction worth pointing at.
    pub glow: Option<[f32; 3]>,
    /// How far the eye reaches, in metres: past this everything is the
    /// horizon's colour.
    ///
    /// **The altitude knob.** A course whose islands hang sixty metres up needs
    /// the floor under them visibly hazed or the height is not in the picture;
    /// a twenty-eight metre proving ground wants its far wall as crisp as its
    /// near one, so it puts this well past anything in it.
    pub reach: f32,
}

/// How far the eye reaches when nothing says otherwise. Comfortably past any
/// arena's own walls, so a sky nobody has thought about does not quietly fog
/// the fight.
pub const DEFAULT_REACH: f32 = 900.0;

impl Sky {
    /// A whole sky from one colour.
    ///
    /// Overhead is **deeper and bluer**, under the horizon is **paler and
    /// warmer**, and the horizon is what was asked for. That is less a
    /// stylistic choice than a description: the zenith is the deepest part of
    /// the sky because you are looking through the least air, the horizon is
    /// the palest because you are looking through the most, and the haze below
    /// carries the warmth of whatever the light has bounced off on the way.
    ///
    /// The derivation is the reason the twenty-third arena is cheap. An arena
    /// names one colour and has a sky; the ones that carry a mood override what
    /// they need, and whatever that teaches comes back here.
    pub const fn over(horizon: [f32; 3]) -> Sky {
        Sky {
            // Filled in by `resolved`, which cannot run in a `const`.
            zenith: [f32::NAN; 3],
            horizon,
            ground: [f32::NAN; 3],
            glow: None,
            reach: DEFAULT_REACH,
        }
    }

    /// The same, with anything left to the derivation filled in.
    ///
    /// `over` has to be `const` so a table of skies can be a `static`, and
    /// colour maths is not. So `over` leaves a mark and this reads it, once per
    /// arena rather than per frame.
    pub fn resolved(self) -> Sky {
        let h = tint::Lch::of(self.horizon);
        Sky {
            zenith: if self.zenith[0].is_nan() {
                // Deeper, more colourful, and a nudge round the wheel toward
                // blue. Small turn: a sunset's zenith is not blue, it is a
                // deeper version of the sunset.
                //
                // The depth is worth more than it looks. At -0.17 the whole
                // sheet came out as twenty-three pale washes -- pleasant, and
                // with no gradient strong enough to tell you which way is up.
                h.lighter(-0.26).vivid(1.45).turned(-0.025).rgb()
            } else {
                self.zenith
            },
            ground: if self.ground[0].is_nan() {
                // **Darker**, flatter and warmer. Below the horizon is land
                // seen through a lot of air, and land is darker than sky; the
                // haze pulls it toward the horizon's colour but never past it.
                //
                // This started out *paler* than the horizon, on the reasoning
                // that bounced light is washed out, and the result had no
                // horizon in it at all: sky and ground met at the same value
                // and the whole dome was one smooth wash. A horizon is a
                // contrast, and something has to be on the darker side of it.
                // Somewhere genuinely above the clouds says so itself.
                h.lighter(-0.10).vivid(0.80).turned(0.05).rgb()
            } else {
                self.ground
            },
            ..self
        }
    }

    /// The same sky, with every colour in it turned the same way round the hue
    /// wheel.
    ///
    /// **One knob for a related arena.** Nine jump courses want nine skies that
    /// are plainly the same world at the same hour -- not nine unrelated moods,
    /// and not one sky nine times. Turning the whole scheme together keeps the
    /// relationships inside it (how much deeper the zenith is, how warm the glow
    /// is against the horizon) and moves only where on the wheel the whole thing
    /// sits, which is exactly the freedom a set of siblings wants.
    ///
    /// `turns`, not degrees: a whole turn is 1.0, so 0.05 is a nudge and 0.5 is
    /// the opposite colour.
    pub fn turned(self, turns: f32) -> Sky {
        let turn = |c: [f32; 3]| tint::Lch::of(c).turned(turns).rgb();
        let sky = self.resolved();
        Sky {
            zenith: turn(sky.zenith),
            horizon: turn(sky.horizon),
            ground: turn(sky.ground),
            glow: sky.glow.map(turn),
            reach: sky.reach,
        }
    }

    /// Where fog begins and ends, in metres.
    ///
    /// Linear, and only one of the two numbers is worth an arena's attention,
    /// so the near one is a fixed fraction: haze that starts the moment you
    /// look away from your own feet reads as a dirty lens rather than as air.
    pub fn fog(&self) -> (f32, f32) {
        (self.reach * 0.22, self.reach)
    }

    /// The colour of the sky in a direction. `dir` and `sun` are unit vectors,
    /// `+y` up.
    pub fn looking(&self, dir: [f32; 3], sun: [f32; 3]) -> [f32; 3] {
        let sky = self.resolved();
        let up = dir[1].clamp(-1.0, 1.0);
        let base = if up >= 0.0 {
            // Shaped rather than linear, so the horizon stays a band rather
            // than becoming half the dome. A linear ramp puts the midpoint
            // forty-five degrees up, which is nothing like a sky.
            tint::mix(sky.horizon, sky.zenith, up.powf(0.55))
        } else {
            tint::mix(sky.horizon, sky.ground, (-up).powf(0.80))
        };
        let Some(glow) = sky.glow else {
            return base;
        };

        // **Added, not blended.** A glow is scattered sunlight piling up on top
        // of the sky: light arriving, not a colour replacing one. Modelling it
        // as a blend is both wrong and visibly wrong -- rotating a cool sky's
        // hue toward a warm sun takes the short way round the wheel, which from
        // blue to orange runs through purple and green, and the first version
        // came out *greener* toward the sun than away from it. Adding cannot do
        // that, because adding warm light to anything makes it warmer.
        //
        // **Broad, not a hotspot.** Scattered light is spread over most of the
        // sky near the sun -- it is the air glowing, not the sun being drawn --
        // so a tight falloff puts a bullseye on the dome and reads as a lens
        // flare somebody left on.
        let dot = dir[0] * sun[0] + dir[1] * sun[1] + dir[2] * sun[2];
        let toward = dot.max(0.0).powf(1.6);
        // Dies with height, so the warmth sits low, where you look through the
        // most air toward the sun.
        let low = (1.0 - up.max(0.0)).powf(1.8);
        let amount = toward * low * 0.75;
        let (b, g) = (tint::linear(base), tint::linear(glow));
        tint::srgb([
            (b[0] + g[0] * amount).min(1.0),
            (b[1] + g[1] * amount).min(1.0),
            (b[2] + g[2] * amount).min(1.0),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tint::Lch;

    fn lightness(c: [f32; 3]) -> f32 {
        Lch::of(c).l
    }

    #[test]
    fn a_derived_sky_is_deepest_overhead_and_palest_below() {
        // What the derivation claims to describe: least air straight up, most
        // of it at the horizon, so the sky runs deep at the top to pale at the
        // band -- and then darker again below it, where you are looking at
        // land rather than at air. Both edges of that are a horizon line.
        for horizon in [
            [0.52, 0.64, 0.74],
            [0.78, 0.62, 0.42],
            [0.30, 0.38, 0.48],
            [0.80, 0.83, 0.88],
        ] {
            let sky = Sky::over(horizon).resolved();
            assert!(
                lightness(sky.zenith) < lightness(sky.horizon),
                "zenith {:?} is not deeper than horizon {horizon:?}",
                sky.zenith
            );
            assert!(
                lightness(sky.ground) < lightness(sky.horizon),
                "the haze below {:?} is not darker than the horizon {horizon:?}, \
                 so the two meet at the same value and there is no horizon line",
                sky.ground
            );
        }
    }

    #[test]
    fn the_horizon_is_exactly_the_colour_it_was_given() {
        // Everything distant fades into the horizon and the fog fades to the
        // same number. If the derivation moved it, distance would read as a
        // grey wash laid over the picture rather than as air.
        let asked = [0.66, 0.74, 0.80];
        let sky = Sky::over(asked).resolved();
        assert_eq!(sky.horizon, asked);
        assert_eq!(sky.looking([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), asked);
    }

    #[test]
    fn the_gradient_keeps_the_horizon_a_band() {
        // Shaped, not linear: a linear ramp puts the halfway colour forty-five
        // degrees up, which looks like a wall rather than a sky.
        let sky = Sky::over([0.52, 0.64, 0.74]).resolved();
        let quarter = lightness(sky.looking([0.92, 0.38, 0.0], [0.0, 1.0, 0.0]));
        let halfway = (lightness(sky.horizon) + lightness(sky.zenith)) * 0.5;
        assert!(
            quarter < halfway,
            "twenty-two degrees up is still brighter than the midpoint, so the \
             horizon has spread over the whole dome"
        );
    }

    #[test]
    fn an_overridden_part_is_left_alone() {
        // The loop this exists for: derive, look, tweak. A tweak the derivation
        // silently overwrote would make the second half impossible.
        let mine = [0.9, 0.1, 0.4];
        let sky = Sky {
            zenith: mine,
            ..Sky::over([0.5, 0.5, 0.5])
        }
        .resolved();
        assert_eq!(sky.zenith, mine);
        assert!(
            !sky.ground[0].is_nan(),
            "the part left alone was not derived"
        );
    }

    #[test]
    fn a_glow_is_warm_toward_the_sun_and_absent_overhead() {
        let sky = Sky {
            glow: Some([0.5, 0.26, 0.10]),
            ..Sky::over([0.52, 0.60, 0.70])
        };
        let sun = [0.97, 0.24, 0.0];
        let warmth = |c: [f32; 3]| c[0] - c[2];
        let toward = warmth(sky.looking(sun, sun));
        let away = warmth(sky.looking([-0.97, 0.24, 0.0], sun));
        let above = warmth(sky.looking([0.0, 1.0, 0.0], sun));
        assert!(toward > away + 0.03, "the glow is not directional");
        assert!(above < toward, "the glow reaches the zenith");
    }

    #[test]
    fn fog_begins_well_out_and_ends_where_the_eye_does() {
        let (start, end) = Sky {
            reach: 400.0,
            ..Sky::over([0.5, 0.6, 0.7])
        }
        .fog();
        assert!(
            start > 40.0,
            "fog starts {start} m out, which is a dirty lens"
        );
        assert_eq!(end, 400.0);
    }
}
