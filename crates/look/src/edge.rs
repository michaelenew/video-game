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
    /// The height, in metres, over which the crest accent builds to its full
    /// amount.
    pub climb: f32,
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
    rim: 0.34,
    reach: 0.45,
    crest: 0.26,
    climb: 20.0,
};

impl Edge {
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
        let high = fade((height / self.climb).clamp(0.0, 1.0));
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
    /// How far the line stands out past the thing, in metres.
    pub swell: f32,
    /// How much darker than the surface the line is, in lightness.
    pub ink: f32,
}

/// The default line. Thin -- about three pixels at the distance a fight is
/// watched from -- because a thick one stops being a line and becomes a border,
/// and a border makes everything look like a sticker.
pub const LINE: Line = Line {
    swell: 0.035,
    ink: 0.30,
};

impl Line {
    /// What colour the line round a surface of this colour is.
    ///
    /// **Not black.** A black line against a pastel palette is the one thing in
    /// frame that did not come from the scheme, and it reads as a hard edge
    /// stuck onto a soft picture. A dark version of the thing's own colour
    /// reads as the same object seen in its own shadow, which is what an
    /// illustrator draws, and it keeps its chroma so a green thing is outlined
    /// in dark green.
    pub fn colour(&self, surface: [f32; 3]) -> [f32; 3] {
        crate::tint::Lch::of(surface)
            .lighter(-self.ink)
            .vivid(0.85)
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
        let floor = EDGE.at(8.0, 1.0, 0.0);
        let up_high = EDGE.at(8.0, 1.0, EDGE.climb);
        let wall_high = EDGE.at(8.0, 0.0, EDGE.climb);
        assert!(
            up_high > floor + 0.2,
            "height is not readable off a top face"
        );
        assert_eq!(wall_high, 0.0, "a vertical face picked up the crest accent");
    }

    #[test]
    fn a_line_is_darker_than_what_it_surrounds_and_still_coloured() {
        use crate::tint::Lch;
        for surface in [[0.62, 0.80, 0.54], [0.94, 0.87, 0.68], [0.54, 0.80, 0.86]] {
            let ink = Lch::of(LINE.colour(surface));
            let on = Lch::of(surface);
            assert!(ink.l < on.l - 0.15, "the line is not dark enough to read");
            assert!(
                ink.c > 0.01,
                "the line went grey, which is a black outline \
                                   with extra steps"
            );
            let turn = (ink.h - on.h).abs();
            assert!(
                turn.min(1.0 - turn) < 0.02,
                "the line is not its object's colour"
            );
        }
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
