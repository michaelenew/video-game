//! What a landscape has to keep being true about itself.

use art::terrain::{Scatter, Terrain};

#[test]
fn the_hills_have_relief_and_it_is_the_relief_asked_for() {
    let land = Terrain::HILLS;
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for i in 0..200 {
        for j in 0..200 {
            let h = land.height(i as f32 * 3.0 - 300.0, j as f32 * 3.0 - 300.0);
            lo = lo.min(h);
            hi = hi.max(h);
        }
    }
    let span = hi - lo;
    // A loose band around the asked-for relief. Not exact, because a sum of
    // octaves reaches its extremes rarely -- the number is a scale, not a
    // bound, and a test that pretended otherwise would fail whenever an octave
    // was added. Looser still because mixing ridged and rolling noise partly
    // cancels: measured, the span at a ridging of 0.62 is *lower* than at
    // either end.
    assert!(
        span > land.relief * 0.3 && span < land.relief * 2.0,
        "asked for {} m of relief and got {span:.1}",
        land.relief
    );
}

#[test]
fn the_origin_is_at_ground_level() {
    // The fighters stand at zero because that is where the simulation puts
    // them, and the simulation has no idea the terrain exists. Without the
    // datum they hover above a valley or stand buried in a hill, depending
    // entirely on what the noise happened to do at the origin -- which looks
    // like a physics bug and is a missing constant.
    let land = Terrain::HILLS;
    let ground = land.height(0.0, 0.0) + land.datum();
    assert!(
        ground.abs() < 1e-4,
        "the origin sits {ground:.3} m off the ground"
    );
}

#[test]
fn a_normal_points_up_and_is_a_unit_vector() {
    let land = Terrain::HILLS;
    for i in 0..40 {
        let (x, z) = (i as f32 * 7.3 - 140.0, i as f32 * -4.1 + 90.0);
        let n = land.normal(x, z);
        assert!(n[1] > 0.0, "the ground faces downward at ({x}, {z}): {n:?}");
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-3, "normal is not unit length: {len}");
    }
}

#[test]
fn slope_agrees_with_the_shape() {
    // Steepness is what decides where things can rest, so it has to actually
    // track the ground rather than being a number that happens to be handy.
    let land = Terrain::HILLS;
    let mut steepest = (0.0f32, 0.0f32, 0.0f32);
    let mut flattest = (1.0f32, 0.0f32, 0.0f32);
    for i in 0..80 {
        for j in 0..80 {
            let (x, z) = (i as f32 * 4.0 - 160.0, j as f32 * 4.0 - 160.0);
            let s = land.slope(x, z);
            if s > steepest.0 {
                steepest = (s, x, z);
            }
            if s < flattest.0 {
                flattest = (s, x, z);
            }
        }
    }
    assert!(
        steepest.0 > 0.1,
        "nowhere is steeper than {:.3}",
        steepest.0
    );
    assert!(
        flattest.0 < 0.02,
        "nowhere is flatter than {:.3}",
        flattest.0
    );

    // And the height actually changes faster on the steep patch.
    let climb = |x: f32, z: f32| (land.height(x + 2.0, z) - land.height(x - 2.0, z)).abs();
    assert!(climb(steepest.1, steepest.2) > climb(flattest.1, flattest.2));
}

#[test]
fn scattered_points_are_spread_rather_than_clumped() {
    // A jittered grid, not independent random points. Independent points clump
    // and leave holes -- which is what random actually looks like, and it reads
    // as a mistake.
    let land = Terrain::HILLS;
    let scatter = Scatter {
        spacing: 20.0,
        jitter: 0.9,
        seed: 7,
    };
    let spots = scatter.over(&land, 150.0, |_| true);
    assert!(spots.len() > 40, "only {} spots over 150 m", spots.len());

    let mut closest = f32::MAX;
    for (i, a) in spots.iter().enumerate() {
        for b in spots.iter().skip(i + 1) {
            let d = ((a.x - b.x).powi(2) + (a.z - b.z).powi(2)).sqrt();
            closest = closest.min(d);
        }
    }
    // With the jitter at 0.9 the tightest pair can close to about a tenth of
    // the spacing; independent points would routinely be far closer.
    assert!(
        closest > scatter.spacing * 0.08,
        "two spots are only {closest:.2} m apart at a spacing of {}",
        scatter.spacing
    );
}

#[test]
fn a_filter_actually_filters() {
    let land = Terrain::HILLS;
    let scatter = Scatter {
        spacing: 14.0,
        jitter: 0.9,
        seed: 11,
    };
    let all = scatter.over(&land, 160.0, |_| true);
    let level = scatter.over(&land, 160.0, |s| s.slope < 0.05);
    assert!(!level.is_empty(), "nowhere level enough to build on");
    assert!(level.len() < all.len(), "the slope filter kept everything");
    for s in &level {
        assert!(s.slope < 0.05);
    }
}

#[test]
fn a_spot_always_looks_the_same() {
    // The scatter is what places every ruin and boulder, so if it is not
    // stable the landscape rearranges itself between runs and no screenshot
    // can be compared with another.
    let land = Terrain::HILLS;
    let scatter = Scatter {
        spacing: 25.0,
        jitter: 0.7,
        seed: 3,
    };
    let a = scatter.over(&land, 120.0, |_| true);
    let b = scatter.over(&land, 120.0, |_| true);
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x.x.to_bits(), y.x.to_bits());
        assert_eq!(x.roll[0].to_bits(), y.roll[0].to_bits());
    }
}

#[test]
fn ridging_creases_the_surface() {
    // Summed octaves of ordinary noise make smooth mounds, which read as dunes.
    // Folding at zero turns bumps into creases, and creases are what hills
    // have -- so the measurable consequence is **curvature**: how sharply the
    // surface bends, summed over the whole landscape.
    //
    // The first version of this test asserted a *skew* instead, on the
    // reasoning that ridges are narrow high ground over broad valleys.
    // Measured, it goes the other way: folding concentrates gradient noise
    // near its zero set, so a ridged landscape is broad high ground cut by
    // narrow hollows, and it is top-heavy rather than bottom-heavy. The look
    // is right and the explanation was not. Curvature is what actually
    // distinguishes them, by about seventy per cent.
    let creasing = |ridged: f32| {
        const N: usize = 110;
        let land = Terrain {
            ridged,
            ..Terrain::HILLS
        };
        let hs: Vec<f32> = (0..N * N)
            .map(|k| land.height((k / N) as f32 * 5.0 - 275.0, (k % N) as f32 * 5.0 - 275.0))
            .collect();
        let mut total = 0.0;
        for i in 1..N - 1 {
            for j in 1..N - 1 {
                let k = i * N + j;
                total += (hs[k - 1] + hs[k + 1] + hs[k - N] + hs[k + N] - 4.0 * hs[k]).abs();
            }
        }
        total
    };
    let rolling = creasing(0.0);
    let ridged = creasing(1.0);
    assert!(
        ridged > rolling * 1.3,
        "ridging creased the surface only {:.2} times as much as rolling noise, \
         so it is not doing what it is there for",
        ridged / rolling
    );
}
