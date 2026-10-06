//! Where the accent goes: along the edges of things, and on what stands high.
//!
//! [`crate::palette`] picks one bright hue per arena that describes nothing.
//! This is the rule for where it lands, and the rule is the whole technique --
//! an accent sprayed evenly over a surface is just a different surface colour.
//!
//! **Along an edge.** A painter puts a bright rim where a form turns away,
//! partly because that is where a real light grazes it, mostly because it draws
//! the shape. Put the same hue on the edge of every object in the frame and the
//! objects start to look related, whatever colour their faces are: they are all
//! lit by the same strange light. That is the trick being borrowed -- not
//! realism, which would want the rim to be the colour of the sun, but
//! *cohesion*, which only needs it to be the same everywhere.
//!
//! **And on what stands high.** The crest of a hill, the top of a wall, the lip
//! of a platform. Partly the same reason -- high ground catches more sky -- and
//! partly a second one worth having for free: the player reads height off it. A
//! surface that gets more accent the higher it is turns the accent into an
//! altimeter, which is exactly what a game about jumping wants.
//!
//! It is a **gradient**, never a band. A hard line where the accent stops is a
//! decal; a soft falloff over a metre or two is a light, and the eye accepts a
//! light doing something physically ridiculous far more readily than it accepts
//! a sticker.

/// How much accent a point on a surface gets, as a rule.
#[derive(Clone, Copy, Debug)]
pub struct Edge {
    /// The most accent an edge gets, 0 to 1.
    pub rim: f32,
    /// How far in from an edge the accent reaches, in metres.
    ///
    /// Fixed in metres rather than as a fraction of the object, so a small
    /// prop and a thirty-metre wall get the same width of rim and read as
    /// being in the same world. A fraction would put a fifteen-metre gradient
    /// on the wall, which is not an edge, it is a colour.
    pub reach: f32,
    /// The most accent a high upward-facing surface gets.
    pub crest: f32,
    /// The height at which the crest accent starts, in metres.
    pub base: f32,
    /// The height at which it reaches its full amount.
    ///
    /// **An arena's own range, not a fixed number**, which is the one thing
    /// here that cannot be a constant. A proving ground's walls stand a metre
    /// and a half; a jump course's islands hang sixty metres up and the lowest
    /// of them is already forty. A ramp from the ground to twenty metres makes
    /// the proving ground read beautifully and paints *every* surface on a
    /// course at full strength, because every surface on a course is past the
    /// top of the ramp -- which is how the Gulf came out as one flat wash of
    /// mint. Stretched over what the arena actually contains, the same rule
    /// says the same thing in both: higher here than that, over there.
    pub top: f32,
}

/// The default rule.
///
/// Subtle, and `reach` is the number that had to come down the furthest. At
/// 1.1 m it looked like a reasonable rim on a swatch and arrived in the game as
/// *everything is purple*: an arena is full of waist-high walls and metre-thick
/// platforms, and on a surface a metre and a half across a 1.1 m rim coming in
/// from both sides is the whole surface. An edge is a band narrow enough that
/// there is plainly a middle it is not reaching.
///
/// The rest are low for a related reason: every object in frame gets them at
/// once, and twenty things each a little bit purple is a lot of purple.
pub const EDGE: Edge = Edge {
    rim: 0.26,
    reach: 0.45,
    crest: 0.20,
    base: 0.0,
    top: 20.0,
};

impl Edge {
    /// The same rule, with the crest stretched over the heights an arena
    /// actually contains.
    ///
    /// A little headroom at both ends, so the lowest thing in the arena is not
    /// flatly unaccented and the highest is not pinned at maximum -- a ramp
    /// that saturates at either end is two flat colours with a gradient in the
    /// middle.
    pub fn across(self, lo: f32, hi: f32) -> Edge {
        let span = (hi - lo).max(2.0);
        Edge {
            base: lo - span * 0.25,
            top: hi + span * 0.15,
            ..self
        }
    }

    /// How much accent at a point.
    ///
    /// `to_edge` is metres to the nearest edge of the face, `up` how far the
    /// surface faces the sky (the normal's `y`, -1 to 1), and `height` how far
    /// above the floor the point is.
    pub fn at(&self, to_edge: f32, up: f32, height: f32) -> f32 {
        let rim = self.rim * fade(1.0 - (to_edge / self.reach).clamp(0.0, 1.0));
        // Only what faces the sky catches it, and squared so a surface has to
        // be properly upward before it starts: a wall that leans back a little
        // is still a wall.
        let facing = up.max(0.0).powi(2);
        let span = (self.top - self.base).max(0.001);
        let high = fade(((height - self.base) / span).clamp(0.0, 1.0));
        (rim + self.crest * facing * high).clamp(0.0, 1.0)
    }
}

/// The line drawn round the outside of a thing.
///
/// The last piece of the hand-drawn feel, and the one that does the most for
/// how deliberate the picture looks. Flat colours with no line read as
/// *untextured* -- as a thing somebody has not finished. The same flat colours
/// with a line round them read as *drawn*, because that is what a drawing is:
/// an outline with colour inside it. Nothing about the shading changed; the
/// line changed what the eye thinks it is looking at.
#[derive(Clone, Copy, Debug)]
pub struct Line {
    /// How much darker than the surface the line is, in Oklab lightness.
    ///
    /// **There is no width here, and that is the point.** A line's width
    /// belongs to the screen, not to the world. Said in metres it comes out
    /// thick along a face turned away from you and thin along one facing you --
    /// one edge of one box going from a band to a hairline along its length --
    /// which is what sent the drawing of it out of the geometry and into
    /// `game::outline`, a pass over the finished picture where a pixel is a
    /// pixel. This crate knows nothing about a window, so it says what colour a
    /// line is and stops.
    pub ink: f32,
}

/// The default line.
///
/// `ink` is small for a reason that is easy to get backwards. A line is not
/// dark, it is *saturated*: see [`Line::colour`].
pub const LINE: Line = Line { ink: 0.24 };

impl Line {
    /// What colour the line round a surface of this colour is.
    ///
    /// **Saturated, barely darker.** This is the part that is easy to get
    /// backwards, and getting it backwards is what makes an outline look cheap.
    /// The instinct is *darker*, and a line taken a long way down in lightness
    /// is a black line whatever hue it started from -- there is no colour left
    /// at the bottom of the scale to say it was ever green. Against a palette
    /// like this one that line is the only thing in frame that did not come
    /// from the scheme, and it reads as a hard edge stuck onto a soft picture.
    ///
    /// What an illustrator does instead is push the *colour* up and the
    /// lightness down only a little. A green thing gets a line of deep, vivid
    /// green: chroma separates it from the pale surface as surely as darkness
    /// would, and it belongs to the object rather than being drawn on top of
    /// it.
    ///
    /// **Measured against the lit colour, not the albedo.** A line is ink: it
    /// is drawn unlit, so what is written here is exactly what lands on the
    /// screen, while everything round it is albedo the sun and the tonemap have
    /// carried a long way up. Darkening the albedo therefore puts the line far
    /// below its surroundings instead of a little below them -- which is a
    /// black outline on every pale surface in the game, the one thing the
    /// paragraph above says not to do.
    pub fn colour(&self, surface: [f32; 3]) -> [f32; 3] {
        let seen = crate::tint::Lch::of(crate::palette::lit(surface));
        crate::tint::Lch {
            l: (seen.l - self.ink).max(0.30),
            // A floor as well as a multiplier, so a near-grey thing -- stone,
            // snow -- still gets a line with a hue in it rather than a grey one.
            c: (seen.c * 2.6).max(0.075),
            h: seen.h,
        }
        .rgb()
    }
}

/// Smooth at both ends, so a gradient arrives and leaves without a seam.
///
/// The usual `3t^2 - 2t^3`. A linear ramp has a visible crease where it meets
/// flat colour, and a crease is exactly the hard line this is trying not to be.
fn fade(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_middle_of_a_wall_is_left_alone() {
        // If this ever stops being true the accent has become a surface colour
        // and the whole technique is a tint slider.
        assert_eq!(EDGE.at(8.0, 0.0, 1.0), 0.0);
    }

    #[test]
    fn an_edge_gets_it_and_a_metre_in_does_not() {
        let on = EDGE.at(0.0, 0.0, 1.0);
        let inside = EDGE.at(EDGE.reach, 0.0, 1.0);
        assert!(
            (on - EDGE.rim).abs() < 1e-5,
            "an edge got {on}, not {}",
            EDGE.rim
        );
        assert_eq!(inside, 0.0, "the rim reached past where it says it reaches");
    }

    #[test]
    fn it_is_a_gradient_and_not_a_band() {
        // Sampled across the falloff: every step down, no step flat, and the
        // ends smooth. A band would show up as two equal values in a row.
        let mut last = f32::MAX;
        for i in 0..=10 {
            let here = EDGE.at(EDGE.reach * i as f32 / 10.0, 0.0, 0.0);
            assert!(here < last, "the falloff flattened at step {i}");
            last = here;
        }
    }

    #[test]
    fn height_shows_on_what_faces_the_sky() {
        let floor = EDGE.at(8.0, 1.0, EDGE.base);
        let up_high = EDGE.at(8.0, 1.0, EDGE.top);
        let wall_high = EDGE.at(8.0, 0.0, EDGE.top);
        assert!(
            up_high > floor + 0.15,
            "height is not readable off a top face"
        );
        assert_eq!(wall_high, 0.0, "a vertical face picked up the crest accent");
    }

    #[test]
    fn a_line_is_darker_than_what_it_surrounds_and_still_coloured() {
        use crate::tint::Lch;
        for surface in [[0.62, 0.80, 0.54], [0.94, 0.87, 0.68], [0.54, 0.80, 0.86]] {
            let ink = Lch::of(LINE.colour(surface));
            // Compared with the surface as it will be *seen*, which is the
            // whole point of the line being chosen that way.
            let on = Lch::of(crate::palette::lit(surface));
            assert!(
                ink.l < on.l - 0.08,
                "the line is not darker than its object"
            );
            assert!(
                ink.l > 0.35,
                "the line is so dark it is a black outline with extra steps"
            );
            assert!(
                ink.c > on.c,
                "the line is duller than its object, so what separates it from \
                 the surface is darkness rather than colour"
            );
            let turn = (ink.h - on.h).abs();
            assert!(
                turn.min(1.0 - turn) < 0.02,
                "the line is not its object's colour"
            );
        }
    }

    #[test]
    fn a_courses_islands_are_not_all_at_the_top_of_the_ramp() {
        // The failure this exists for. A jump course's lowest island is already
        // forty metres up, so a ramp measured from the ground paints every
        // surface on it at full strength and the whole course comes out as one
        // flat wash. Stretched over what the arena holds, the same rule still
        // says *this is higher than that*.
        let course = EDGE.across(42.0, 78.0);
        let low = course.at(8.0, 1.0, 42.0);
        let high = course.at(8.0, 1.0, 78.0);
        assert!(
            high > low + 0.08,
            "the course reads flat: {low:.3} to {high:.3}"
        );
        assert!(
            low < course.crest * 0.5,
            "its lowest island is already maxed out"
        );
    }

    #[test]
    fn nothing_ever_goes_past_the_accent() {
        // Rim and crest add, and on the top edge of a high wall they both
        // apply. Past 1.0 the surface would stop being a surface.
        let worst = EDGE.at(0.0, 1.0, 1000.0);
        assert!(worst <= 1.0);
        assert!(worst > EDGE.rim, "the two do not add where they overlap");
    }
}
