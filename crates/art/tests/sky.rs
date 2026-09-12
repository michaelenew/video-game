//! The lighting rig has to agree with the daylight it is modelling.
//!
//! These are checkable against published figures rather than against taste,
//! which is the point of deriving the rig instead of tuning it. Every number
//! asserted here is one a meteorology table will give you.

use art::color::{self, Lch};
use art::sky::{self, Sky};

#[test]
fn air_mass_matches_the_textbook_figures() {
    // One atmosphere straight up, by definition, and about 38 at the horizon
    // -- the figure that makes the sun set red. The naive `1/sin` would say
    // infinity here, which is why the fit exists.
    assert!(
        (sky::air_mass(90.0) - 1.0).abs() < 0.01,
        "{}",
        sky::air_mass(90.0)
    );
    assert!(
        (sky::air_mass(30.0) - 2.0).abs() < 0.05,
        "{}",
        sky::air_mass(30.0)
    );
    assert!(
        (36.0..40.0).contains(&sky::air_mass(0.0)),
        "{}",
        sky::air_mass(0.0)
    );
}

#[test]
fn the_sun_reddens_as_it_sets() {
    // The one behaviour the whole module exists for, and nobody chose it: blue
    // is scattered out of the beam about three times faster than red, so a
    // longer path through air leaves a redder sun.
    let mut last_blue = 1.0;
    for e in [90.0, 45.0, 20.0, 10.0, 5.0, 2.0] {
        let s = Sky::at(e, 0.0);
        assert_eq!(
            s.sun_color[0], 1.0,
            "red should stay the brightest channel at {e} degrees"
        );
        assert!(
            s.sun_color[2] < last_blue,
            "at {e} degrees the sun is no redder than it was higher up"
        );
        last_blue = s.sun_color[2];
    }
    // And by the horizon there is essentially no blue left.
    assert!(Sky::at(0.0, 0.0).sun_color[2] < 0.05);
}

#[test]
fn noon_matches_real_daylight() {
    // Clear-sky figures: about 110,000 lux of direct sun and about 20,000 of
    // skylight. If these drift far, the scene is no longer lit like a place.
    let noon = Sky::at(90.0, 0.0);
    assert!(
        (100_000.0..125_000.0).contains(&noon.sun_illuminance),
        "noon sun at {:.0} lux",
        noon.sun_illuminance
    );
    assert!(
        (12_000.0..28_000.0).contains(&noon.sky_illuminance),
        "noon skylight at {:.0} lux",
        noon.sky_illuminance
    );
}

#[test]
fn the_sky_is_blue_at_noon_and_never_white_at_dusk() {
    // The bug this was written against: taking the sky's colour from the
    // *sun's* air mass saturates every channel at sunset and hands back a flat
    // white sky. Overhead the air is one atmosphere thick whatever the sun is
    // doing, so the sky keeps a hue all the way down.
    let noon = Sky::at(90.0, 0.0);
    assert!(
        noon.sky_color[2] > noon.sky_color[0] * 1.8,
        "the noon sky is not clearly blue: {:?}",
        noon.sky_color
    );

    for e in [20.0, 8.0, 2.0, 0.0] {
        let c = Lch::from_oklab(color::linear_to_oklab(Sky::at(e, 0.0).sky_color));
        assert!(
            c.c > 0.02,
            "the sky at {e} degrees has washed out to chroma {:.3}, which is white",
            c.c
        );
    }
}

#[test]
fn the_lights_go_out_over_a_couple_of_degrees() {
    // Without the fade the direct beam switches off in a single frame as
    // elevation crosses zero, and dusk arrives as a pop. A real sun takes a
    // little over its own diameter to set.
    assert!(Sky::at(3.0, 0.0).sun_illuminance > 20_000.0);
    assert!(Sky::at(-2.0, 0.0).sun_illuminance == 0.0);
    let dusk = Sky::at(0.0, 0.0).sun_illuminance;
    assert!(
        (100.0..20_000.0).contains(&dusk),
        "sunset sun at {dusk:.0} lux"
    );
    // Skylight outlasts the sun, which is what twilight is.
    assert!(Sky::at(-1.0, 0.0).sky_illuminance > 0.0);
}

#[test]
fn total_light_only_ever_falls_as_the_sun_drops() {
    let mut last = f32::MAX;
    for e in [90.0, 60.0, 30.0, 15.0, 8.0, 4.0, 0.0, -1.5] {
        let s = Sky::at(e, 0.0);
        let total = s.sun_illuminance + s.sky_illuminance;
        assert!(total < last, "it got brighter at {e} degrees");
        last = total;
    }
}

#[test]
fn the_sun_direction_points_where_the_sun_is_not() {
    // A directional light wants the direction light *travels*, which is away
    // from the sun. Getting this backwards lights the scene from underneath
    // and is surprisingly easy to miss, because it still looks lit.
    for e in [10.0f32, 45.0, 80.0] {
        let d = Sky::at(e, 0.0).sun_direction;
        assert!(d[1] < 0.0, "at {e} degrees the light travels upward: {d:?}");
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        assert!(
            (len - 1.0).abs() < 1e-4,
            "direction is not a unit vector: {len}"
        );
    }
}

#[test]
fn the_sky_is_always_cooler_than_the_sun() {
    // What makes lit and shadowed sides read as different surfaces rather than
    // as the same surface at two brightnesses. It falls out of the physics --
    // the sky is made of exactly the light the beam lost, which is the blue
    // end -- so it is worth pinning rather than arranging.
    for e in [90.0, 45.0, 15.0, 5.0, 0.0] {
        let s = Sky::at(e, 0.0);
        let sun_warmth = s.sun_color[0] - s.sun_color[2];
        let sky_warmth = s.sky_color[0] - s.sky_color[2];
        assert!(
            sky_warmth < sun_warmth,
            "at {e} degrees the sky ({sky_warmth:.2}) is not cooler than the sun ({sun_warmth:.2})"
        );
    }
}
