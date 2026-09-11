//! The claims the colour module makes, checked.

use art::color::{self, Lch, Ramp};

#[test]
fn srgb_and_oklab_round_trip() {
    for r in 0..6 {
        for g in 0..6 {
            for b in 0..6 {
                let c = [r as f32 / 5.0, g as f32 / 5.0, b as f32 / 5.0];
                let back = color::oklab_to_linear(color::linear_to_oklab(c));
                for k in 0..3 {
                    assert!((c[k] - back[k]).abs() < 1e-4, "{c:?} came back as {back:?}");
                }
            }
        }
    }
}

#[test]
fn the_transfer_function_round_trips_too() {
    for i in 0..=255 {
        let v = i as f32 / 255.0;
        let back = color::linear_to_srgb(color::srgb_to_linear(v));
        assert!((v - back).abs() < 1e-5, "{v} came back as {back}");
    }
}

#[test]
fn a_blend_in_oklab_does_not_go_through_grey() {
    // The reason this crate does not blend in RGB. Averaging the channels of
    // two opposite colours lands near the middle of the cube, which is grey --
    // orange to blue passes through mud. Oklab's midpoint stays colourful.
    let hot = Lch::new(0.7, 0.16, 0.10);
    let cold = Lch::new(0.5, 0.16, 0.72);
    let oklab_mid = Lch::from_oklab(color::linear_to_oklab(Ramp::two(hot, cold).at(0.5))).c;

    let (a, b) = (hot.to_linear(), cold.to_linear());
    let naive = [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ];
    let naive_mid = Lch::from_oklab(color::linear_to_oklab(naive)).c;

    assert!(
        oklab_mid > naive_mid * 1.25,
        "the Oklab midpoint kept chroma {oklab_mid:.3} against the naive blend's {naive_mid:.3} \
         -- if these ever converge, blending in RGB has stopped costing anything and this \
         module's main premise is worth re-examining"
    );
}

#[test]
fn a_ramp_takes_the_short_way_round_the_hue_circle() {
    // A ramp from just under a full turn to just over zero is two neighbouring
    // reds. Interpolating the numbers directly sweeps the entire spectrum
    // backwards -- the classic rainbow smear.
    let ramp = Ramp::two(Lch::new(0.6, 0.12, 0.97), Lch::new(0.6, 0.12, 0.03));
    for k in 0..=10 {
        let h = Lch::from_oklab(color::linear_to_oklab(ramp.at(k as f32 / 10.0))).h;
        let near_wrap = h > 0.93 || h < 0.07;
        assert!(
            near_wrap,
            "at {k}/10 the ramp reached hue {h:.3}, the long way round"
        );
    }
}

#[test]
fn gamut_clipping_gives_up_chroma_and_keeps_hue() {
    // Most of OkLCh is outside what a screen can show. Clamping the channels
    // would shift the hue, which is the one property the palette holds fixed.
    for h in 0..24 {
        let hue = h as f32 / 24.0;
        let asked = Lch::new(0.75, 0.40, hue); // far outside sRGB
        let got = Lch::from_oklab(color::linear_to_oklab(asked.to_linear()));
        let mut dh = (got.h - hue).abs();
        if dh > 0.5 {
            dh = 1.0 - dh;
        }
        assert!(dh < 0.02, "hue {hue:.3} clipped to {:.3}", got.h);
        assert!(got.c < asked.c, "chroma did not give way at hue {hue:.3}");
    }
}

#[test]
fn a_blackbody_warms_from_red_to_white_and_then_past_it() {
    // The claim that makes fire's colour a physical question: one number, and
    // the progression is the one a real fire makes.
    //
    // Red-leaning holds only up to about 6000 K. Past that the Planckian locus
    // carries on **through** white and out the blue side -- that is why a very
    // hot star is blue. The first version of this test asserted red >= green >=
    // blue over the whole range and 6500 K correctly failed it: daylight is
    // already fractionally blue. Nothing the game sets fire to goes near there,
    // but a test that is wrong about the physics is worse than no test.
    let mut last_blue = -1.0;
    for t in [1667.0, 2000.0, 2600.0, 3200.0, 4500.0, 6500.0] {
        let c = color::blackbody(t);
        if t <= 5000.0 {
            assert!(
                c[0] >= c[1] && c[1] >= c[2],
                "{t}K came out as {c:?}, not red-leaning"
            );
        }
        assert!(
            c[2] > last_blue,
            "{t}K is no bluer than the step below it -- the fit has turned around"
        );
        last_blue = c[2];
    }

    // An ember is orange, not white.
    let ember = color::blackbody(color::COOLEST);
    assert!(
        ember[2] < 0.05,
        "the coolest available fire has blue in it: {ember:?}"
    );
    // Daylight is nearly neutral.
    let day = color::blackbody(6500.0);
    assert!(
        day[1] > 0.85 && day[2] > 0.85,
        "6500K is not near white: {day:?}"
    );
    // And a star is not.
    let star = color::blackbody(25_000.0);
    assert!(star[2] > star[0], "25000K is not blue-leaning: {star:?}");
}

#[test]
fn the_blackbody_floor_is_where_the_fit_stops_being_monotonic() {
    // Why `COOLEST` exists rather than clamping at some rounder number: below
    // it the published fit turns around and hands back the same colour for two
    // different temperatures, so a cooling ember would stop changing and
    // nothing would say why.
    assert_eq!(color::blackbody(800.0), color::blackbody(color::COOLEST));
    assert_ne!(color::blackbody(color::COOLEST), color::blackbody(2500.0));
}
