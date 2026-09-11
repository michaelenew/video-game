//! What the material system has to keep being true.

use art::bake::{self, Plan};
use art::materials::{self, BAKE_SIZE, Material};
use art::palette;
use art::surface::{Emission, Field, FieldKind, Surface};

/// How much a field changes as you walk a metre along an axis. Sampled a
/// millimetre at a time, which is far above the rate any frequency in the
/// table needs -- the first version of this measure walked in centimetres, and
/// undersampled stone's strata badly enough to report the material as
/// isotropic when its form field was nothing of the kind.
fn variation(s: &Surface, dir: [f32; 3]) -> f32 {
    let mut prev = s.scalar([0.0; 3]);
    let mut total = 0.0;
    for k in 1..2000 {
        let d = k as f32 * 0.001;
        let v = s.scalar([dir[0] * d, dir[1] * d, dir[2] * d]);
        total += (v - prev).abs();
        prev = v;
    }
    total
}

#[test]
fn every_field_stays_inside_its_range() {
    // Colour, roughness and relief are all indexed by this number. A field
    // that escapes 0..1 does not produce a wrong-looking material, it produces
    // a clamped one -- a flat band exactly where the interesting part was.
    for kind in [
        FieldKind::Fbm,
        FieldKind::Ridged,
        FieldKind::Cells,
        FieldKind::Seams,
        FieldKind::Stripe,
        FieldKind::Flat,
    ] {
        let f = Field::new(kind, [1.7, 1.7, 1.7], 0x1234);
        for i in 0..40 {
            for j in 0..40 {
                let v = f.at([i as f32 * 0.11, j as f32 * 0.07, (i + j) as f32 * 0.13]);
                assert!((0.0..=1.0).contains(&v), "{kind:?} returned {v}");
            }
        }
    }
}

#[test]
fn the_detail_layer_does_not_drown_the_structure() {
    // The failure mode this whole crate is arranged against: stack enough
    // isotropic detail on an anisotropic form and everything converges on the
    // same grey marble.
    //
    // It is not hypothetical. Stone's form field is 2.6-to-1 in favour of its
    // vertical, which is what makes it strata. With an isotropic crack layer at
    // 30% grain the finished surface measured 1.1-to-1 -- the strata were still
    // in the parameters and no longer on the screen. Flattening the crack cells
    // the same way the strata are flattened put it back.
    for (m, across, along) in [
        (materials::STONE, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
        (materials::FIRE, [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]),
    ] {
        let low = variation(&m.surface, across);
        let high = variation(&m.surface, along);
        assert!(
            high > low * 1.3,
            "{} varies {high:.2} along its grain and {low:.2} across it, a ratio of \
             {:.2}. Under about 1.3 the material has no direction left and reads as \
             noise. The usual cause is `grain` too high, or a detail field that is \
             isotropic where the form field is not.",
            m.name,
            high / low
        );
    }
}

#[test]
fn a_material_uses_the_range_it_is_given() {
    // A surface whose field never leaves the middle is a flat colour with extra
    // steps: the ramp's ends are never reached, the roughness never varies, and
    // the relief is a rumour. This catches a frequency set so low that the
    // whole texture samples one hill.
    for m in materials::FIXED {
        let (mut lo, mut hi) = (1.0f32, 0.0f32);
        for i in 0..32 {
            for j in 0..32 {
                let t = m.surface.scalar([i as f32 * 0.09, 0.37, j as f32 * 0.11]);
                lo = lo.min(t);
                hi = hi.max(t);
            }
        }
        assert!(
            hi - lo > 0.25,
            "{} only ever spans {lo:.2}..{hi:.2}, so most of its ramp is unreachable",
            m.name
        );
    }
}

#[test]
fn every_material_is_coarse_enough_to_be_resolved() {
    // Sampling theory's oldest result, applied to a texture. A feature that
    // lands on fewer than about four texels does not come out fine -- it comes
    // out as **moiré**, a coarse interference pattern that is not in the
    // material at all and gets coarser the finer you make the thing producing
    // it. So detail past the resolution is worse than no detail: it costs
    // evaluation time *and* adds a pattern nobody asked for.
    //
    // Both of the materials that failed this first were failing visibly and
    // neither looked like a sampling problem. Cloth's weave at 34 cycles per
    // metre resolved to 3.8 texels and came out as watered silk. Skin's pores
    // resolved to 2.9 and came out as a sheen. Pores are invisible at fighting
    // distance anyway, which is the other half of the lesson.
    for m in materials::FIXED {
        let per_cycle = m.texels_per_cycle(BAKE_SIZE);
        assert!(
            per_cycle >= 4.0,
            "{} puts its finest detail on {per_cycle:.1} texels at {BAKE_SIZE} pixels. \
             Under four it reads as interference, not as detail -- lower the frequency, \
             drop an octave, or give the material a smaller tile.",
            m.name
        );
    }
    for m in [
        materials::cloth(palette::PLAYERS[0]),
        materials::armour(palette::PLAYERS[0]),
        materials::arcane(palette::PLAYERS[0]),
    ] {
        let per_cycle = m.texels_per_cycle(BAKE_SIZE);
        assert!(
            per_cycle >= 4.0,
            "{} resolves to {per_cycle:.1} texels per cycle",
            m.name
        );
    }
}

#[test]
fn the_same_point_always_gives_the_same_answer() {
    // Not for the network -- nothing here reaches the simulation. For the bake:
    // a material that shifts between runs makes every visual comparison
    // worthless and every golden test a coin flip.
    for m in materials::FIXED {
        for i in 0..16 {
            let p = [i as f32 * 0.31, 1.7, i as f32 * -0.13];
            assert_eq!(m.surface.scalar(p).to_bits(), m.surface.scalar(p).to_bits());
        }
    }
}

#[test]
fn nothing_but_an_effect_emits_light() {
    // Emission is the game's whole spectacle budget, and a surface that emits
    // by accident is one that cannot be turned off. Stone lighting itself
    // would also quietly wreck the palette rule, because a glowing surface
    // writes its own colour past the lighting that was keeping the arena grey.
    //
    // One-directional on purpose. The first version of this asserted that
    // effects emit *and* nothing else does, and the shadow failed it -- which
    // was the test being wrong rather than the shadow. A shadow is an absence
    // of light; a shadow that glows is a lamp.
    for m in materials::FIXED {
        if matches!(m.surface.emission, Emission::None) {
            continue;
        }
        assert_eq!(
            m.role,
            materials::Role::Effect,
            "{} emits light and is not an effect",
            m.name
        );
    }
}

#[test]
fn fire_is_dark_until_it_is_lit() {
    // Flame is emitted light, not a lit surface. Give it a real base colour and
    // it picks up the arena's lamps and reads as orange plastic.
    for k in 0..=8 {
        let s = materials::FIRE.surface.shade(k as f32 / 8.0);
        let brightest = s.albedo[0].max(s.albedo[1]).max(s.albedo[2]);
        assert!(
            brightest < 0.06,
            "fire's albedo reached {brightest:.3} at {k}/8"
        );
    }
    // And it does actually glow, brighter than white at the core.
    let core = materials::FIRE.surface.shade(1.0);
    assert!(
        core.emissive[0] > 1.0,
        "fire's core is not brighter than white"
    );
}

#[test]
fn a_tileable_bake_actually_tiles() {
    // The four-corner blend is bought with contrast, so it has to deliver what
    // it was bought for: the last column has to equal the first, not merely
    // resemble it. "Nearly" is exactly what a seam is.
    let maps = bake::bake(&materials::STONE.surface, Plan::wall(64, 2.0));
    let w = maps.albedo.width;
    for map in [&maps.albedo, &maps.metallic_roughness] {
        for y in 0..map.height {
            let left = ((y * w) * 4) as usize;
            let right = ((y * w + w - 1) * 4) as usize;
            // One texel in from the far edge is one texel short of the wrap, so
            // the two differ by a single step of the field rather than by a
            // seam. Compare against the tolerance a step can produce.
            let d = (map.pixels[left] as i32 - map.pixels[right] as i32).abs();
            assert!(d <= 24, "row {y} jumps by {d} across the wrap");
        }
    }
}

#[test]
fn an_untiled_bake_keeps_more_contrast_than_a_tiled_one() {
    // The trade being made, stated as a test so it is not forgotten: tiling
    // averages the tile against three shifted copies of itself, which pulls
    // everything toward the mean. Worth it on a floor that repeats forty
    // times, pointless on a fighter's arm.
    let spread = |plan: Plan| {
        let maps = bake::bake(&materials::STONE.surface, plan);
        let (mut lo, mut hi) = (255u8, 0u8);
        for px in maps.albedo.pixels.chunks(4) {
            lo = lo.min(px[0]);
            hi = hi.max(px[0]);
        }
        hi - lo
    };
    let tiled = spread(Plan::wall(64, 2.0));
    let plain = spread(Plan::wall(64, 2.0).tiling(false));
    assert!(
        plain > tiled,
        "tiling cost no contrast ({plain} against {tiled}) -- either the blend stopped \
         happening or the material has none to lose"
    );
}

#[test]
fn relief_means_the_same_thing_at_every_resolution() {
    // `relief` is in metres and a texel is `extent / size` metres, so the slope
    // has to be scaled by their ratio. Skip the scaling and the height
    // difference between neighbouring texels shrinks with resolution while
    // nothing compensates, so a material baked at 1024 comes out visibly
    // flatter than the same material at 256 -- a thoroughly confusing bug to
    // go looking for, since nothing changed but a number that is supposed to
    // mean "more detail".
    //
    // Measured on a **single-octave** surface on purpose. A fractal one does
    // not hold still under this test and should not be expected to: a finer
    // bake genuinely resolves octaves the coarse one could not see, and those
    // octaves genuinely have steeper local slopes. Stone measures 1.78x across
    // the same range and is behaving correctly. Testing the mechanism means
    // testing it where the mechanism is the only thing moving.
    let smooth = Surface {
        form: Field::new(FieldKind::Fbm, [1.2, 1.2, 1.2], 0x9001).octaves(1),
        detail: Field::FLAT,
        warp: 0.0,
        grain: 0.0,
        contrast: 1.0,
        ramp: materials::STONE.surface.ramp,
        roughness: (0.9, 0.9),
        metallic: 0.0,
        emission: Emission::None,
        relief: 0.03,
    };

    let tilt = |size: u32| {
        let maps = bake::bake(&smooth, Plan::wall(size, 2.0));
        let mut total = 0.0f32;
        for px in maps.normal.pixels.chunks(4) {
            let x = px[0] as f32 / 255.0 - 0.5;
            let y = px[1] as f32 / 255.0 - 0.5;
            total += (x * x + y * y).sqrt();
        }
        total / (size * size) as f32
    };
    let ratio = tilt(256) / tilt(64);
    assert!(
        (0.85..1.18).contains(&ratio),
        "average surface tilt changed by {ratio:.2}x between 64 and 256 pixels, so relief \
         is being measured in texels rather than in metres"
    );
}

#[test]
fn a_bake_carries_its_emission_scale_out_separately() {
    // Fire's core is many times brighter than white and eight bits per channel
    // stops at white. Normalise the map and hand the factor back, or every
    // flame clips to a flat orange slab and loses the only part anyone looks
    // at.
    let fire = bake::bake(&materials::FIRE.surface, Plan::character(32));
    assert!(
        fire.emissive_strength > 1.5,
        "fire baked with a strength of {} -- it is not brighter than white",
        fire.emissive_strength
    );
    let hottest = fire.emissive.pixels.chunks(4).map(|p| p[0]).max().unwrap();
    assert!(
        hottest > 200,
        "the hottest texel in the fire map is only {hottest}"
    );

    let stone = bake::bake(&materials::STONE.surface, Plan::wall(32, 2.0));
    assert_eq!(
        stone.emissive_strength, 1.0,
        "stone came back with an emission scale"
    );
}

#[test]
fn every_map_comes_out_the_size_it_was_asked_for() {
    let m: Material = materials::SKIN;
    let maps = bake::bake(&m.surface, Plan::character(48));
    for (name, t) in [
        ("albedo", &maps.albedo),
        ("normal", &maps.normal),
        ("metallic_roughness", &maps.metallic_roughness),
        ("emissive", &maps.emissive),
    ] {
        assert_eq!((t.width, t.height), (48, 48), "{name} is the wrong size");
        assert_eq!(
            t.pixels.len(),
            48 * 48 * 4,
            "{name} has the wrong byte count"
        );
    }
}
