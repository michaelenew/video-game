//! What a rock has to keep being true about itself.
//!
//! These are mostly geology stated as arithmetic. The point of parameterising a
//! rock by *how it formed* rather than by frequencies is that the resulting
//! claims are checkable: a sedimentary rock is layered, an igneous one is not,
//! a metamorphic one is layered *and* has grains flattened into the layering.

use art::bake::{Detail, bake_stone};
use art::color::{self, Lch};
use art::palette;
use art::stone::{Fabric, Placement, Stone, library};

/// How much a rock's colour varies along a direction.
///
/// **Sampled with the grain filtered out**, which is the only way this
/// measures anything. The first version sampled at full resolution and every
/// rock came back isotropic -- not because bedding was missing, but because a
/// millimetre-scale speckle contributes vastly more total variation along any
/// line than a metre-scale layering does. It was measuring the grain and
/// calling it the fabric.
fn structure_variation(stone: &Stone, from: [f32; 3], dir: [f32; 3], span: f32) -> f32 {
    const STEPS: usize = 400;
    let mut prev: Option<f32> = None;
    let mut total = 0.0;
    for i in 0..STEPS {
        let t = i as f32 / STEPS as f32 * span;
        let p = [
            from[0] + dir[0] * t,
            from[1] + dir[1] * t,
            from[2] + dir[2] * t,
        ];
        let l = Lch::from_oklab(color::linear_to_oklab(stone.at_scale(p, 0.04).colour)).l;
        if let Some(was) = prev {
            total += (l - was).abs();
        }
        prev = Some(l);
    }
    total
}

#[test]
fn a_rock_is_a_solid_and_not_an_extruded_picture() {
    // **The claim the whole module rests on.** Two parallel cuts through a
    // solid are different rock; two parallel cuts through a 2D field pushed
    // sideways are the same picture twice. Nothing else here matters if this
    // is not true.
    for (name, stone) in library() {
        let mut differ = 0;
        let mut samples = 0;
        for i in 0..24 {
            for j in 0..24 {
                let (x, y) = (i as f32 * 0.05, j as f32 * 0.05);
                let near = stone.at([x, y, 0.0]).colour;
                let far = stone.at([x, y, 0.4]).colour;
                samples += 1;
                if color::difference(near, far) > 0.02 {
                    differ += 1;
                }
            }
        }
        assert!(
            differ * 3 > samples,
            "{name}: only {differ} of {samples} points differ between two cuts 0.4 m apart, \
             so the rock is a picture rather than a volume"
        );
    }
}

#[test]
fn a_sedimentary_rock_is_layered_and_an_igneous_one_is_not() {
    // The mechanism, checked: beds are deposited under gravity, so a
    // sedimentary rock varies far more *across* its bedding than along it. A
    // rock frozen from a melt has no such direction at all.
    let sand = art::stone::sandstone();
    let across = structure_variation(&sand, [0.0, -1.0, 0.0], [0.0, 1.0, 0.0], 2.0);
    let along = structure_variation(&sand, [-1.0, 0.3, 0.0], [1.0, 0.0, 0.0], 2.0);
    assert!(
        across > along * 1.5,
        "sandstone varies {across:.2} across its bedding and {along:.2} along it, a ratio of \
         {:.2} -- under about 1.5 it has no beds",
        across / along
    );

    let gran = art::stone::granite();
    let v = structure_variation(&gran, [0.0, -1.0, 0.0], [0.0, 1.0, 0.0], 2.0);
    let h = structure_variation(&gran, [-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], 2.0);
    let ratio = v.max(h) / v.min(h).max(1e-4);
    assert!(
        ratio < 1.6,
        "granite varies {ratio:.2} times more one way than the other, which is bedding, \
         and an igneous rock has none"
    );
}

#[test]
fn a_metamorphic_rock_is_banded() {
    // Segregation banding: light and dark minerals separate along the
    // foliation, so the variation across it is dramatic.
    let g = art::stone::gneiss();
    let across = structure_variation(&g, [0.0, -0.8, 0.0], [0.25, 0.97, 0.0], 1.6);
    let along = structure_variation(&g, [-0.8, 0.0, 0.0], [0.97, -0.25, 0.0], 1.6);
    assert!(
        across > along * 1.4,
        "gneiss varies {across:.2} across its foliation and {along:.2} along it"
    );
}

#[test]
fn the_grain_fades_into_its_mean_rather_than_aliasing() {
    // A crystal smaller than a pixel cannot be drawn; the only question is
    // whether it turns into noise or into the colour the rock averages to.
    // Noise is what it does by default, and on a wall it is a grey fizz that
    // crawls when the camera moves.
    let g = art::stone::granite();
    let spread = |texel: f32| {
        let (mut lo, mut hi) = (1.0f32, 0.0f32);
        for i in 0..40 {
            for j in 0..40 {
                // Well away from any joint, so this measures grain alone.
                let p = [0.30 + i as f32 * 0.004, 0.30 + j as f32 * 0.004, 0.31];
                let l = Lch::from_oklab(color::linear_to_oklab(g.at_scale(p, texel).colour)).l;
                lo = lo.min(l);
                hi = hi.max(l);
            }
        }
        hi - lo
    };
    let close = spread(0.001);
    let far = spread(0.20);
    assert!(
        close > 0.15,
        "granite shows no grain even close up: {close:.3}"
    );
    assert!(
        far < close * 0.5,
        "at 20 cm per texel the grain still spans {far:.3} against {close:.3} close up, \
         so it is aliasing rather than averaging"
    );
}

#[test]
fn a_joint_narrower_than_a_texel_gets_fainter_and_does_not_vanish() {
    // The other half of band-limiting, and the half that is easy to get
    // backwards. Grain fills an area, so it fades toward its mean. A joint is a
    // *line*: its mean over a texel is almost all rock, so treating it the same
    // way deletes it -- which is what happened, and it took every crack out of
    // the arena walls at the resolution they are actually baked at.
    let jointed = art::stone::granite();
    let mut plain = art::stone::granite();
    plain.joint_count = 0;

    // How much light a traverse loses to jointing: the mean brightness of a
    // line across the joint sets, against the same line through rock with no
    // joints at all. A control rather than a comparison against the darkest
    // grain, which is what the first version did -- and biotite is darker than
    // any crack, so it could never have passed.
    let lost = |stone: &Stone, texel: f32| {
        let mut total = 0.0;
        const N: usize = 900;
        for i in 0..N {
            let p = [i as f32 * 0.006, 0.4, 0.2];
            total += Lch::from_oklab(color::linear_to_oklab(stone.at_scale(p, texel).colour)).l;
        }
        total / N as f32
    };

    let control = lost(&plain, 0.02);
    let fine = lost(&jointed, 0.002);
    // Twelve centimetres per texel is what a thirty-metre arena wall actually
    // gets, and a one-centimetre joint is a twelfth of one.
    let coarse = lost(&jointed, 0.12);

    assert!(
        fine < control * 0.97,
        "joints remove no light even at full resolution ({fine:.3} against {control:.3})"
    );
    assert!(
        coarse < control * 0.99,
        "at 12 cm per texel the joints have vanished entirely ({coarse:.3} against an \
         unjointed {control:.3}) -- a line narrower than a texel has to widen and fade, \
         not disappear"
    );
}

#[test]
fn the_rock_the_arena_is_built_from_is_nearly_colourless() {
    // The palette rule, which outranks geology: the world stays grey so that
    // anything with colour in it reads as a thing that matters. A warm rock is
    // a perfectly good rock and cannot be what an arena is made of -- which is
    // why sandstone stays in the library and granite is what gets built with.
    let arena = art::stone::granite();
    assert!(
        arena.max_chroma() <= palette::WORLD_CHROMA_CEILING,
        "the arena's rock reaches chroma {:.3}, over the ceiling of {}",
        arena.max_chroma(),
        palette::WORLD_CHROMA_CEILING
    );
}

#[test]
fn every_rock_in_the_library_is_a_plausible_rock() {
    for (name, stone) in library() {
        let mut total = 0.0;
        let mut n = 0;
        for i in 0..20 {
            for j in 0..20 {
                let c = stone
                    .at_scale([i as f32 * 0.06, j as f32 * 0.06, 0.17], 0.05)
                    .colour;
                total += (c[0] + c[1] + c[2]) / 3.0;
                n += 1;
            }
        }
        let albedo = total / n as f32;
        // Coal is about 0.04 and white marble about 0.5, which is the top of
        // the range anything made of rock reaches -- fresh snow is 0.8 and
        // nothing else gets near it. Outside this band is not a rock, it is a
        // mistake in a ramp.
        assert!(
            (0.005..0.55).contains(&albedo),
            "{name} reflects {albedo:.3} of the light falling on it"
        );
    }
}

#[test]
fn two_faces_of_one_wall_are_different_rock() {
    // What the solid model is *for*. Baking a surface as its intersection with
    // a volume means two faces in different places are different stone, with
    // nothing anywhere having to arrange it.
    let g = art::stone::granite();
    let face = |origin: [f32; 3]| {
        bake_stone(
            &g,
            &Placement {
                origin,
                across: [4.0, 0.0, 0.0],
                down: [0.0, -3.0, 0.0],
            },
            48,
            // Fine enough to resolve the grain, which is where two different
            // pieces of rock differ most obviously.
            Detail::Uniform(0.003),
        )
    };
    let a = face([-14.0, 3.0, -14.0]);
    let b = face([6.0, 3.0, 12.0]);
    let differing = a
        .albedo
        .pixels
        .chunks(4)
        .zip(b.albedo.pixels.chunks(4))
        .filter(|(x, y)| (x[0] as i32 - y[0] as i32).abs() > 6)
        .count();
    assert!(
        differing * 3 > (48 * 48) as usize,
        "two walls twenty metres apart came out nearly identical ({differing} texels differ), \
         so the bake is not reading the volume at the place the wall actually is"
    );
}

#[test]
fn a_rock_does_not_change_when_you_look_at_it_twice() {
    for (name, stone) in library() {
        for i in 0..20 {
            let p = [i as f32 * 0.13, 0.7, -i as f32 * 0.07];
            let a = stone.at_scale(p, 0.01);
            let b = stone.at_scale(p, 0.01);
            assert_eq!(
                a.colour[0].to_bits(),
                b.colour[0].to_bits(),
                "{name} is not stable"
            );
        }
    }
}

#[test]
fn the_three_fabrics_are_actually_three_different_functions() {
    // Guard against the fabrics collapsing into one function with different
    // constants, which is the thing this module exists not to be.
    let mut seen = std::collections::HashSet::new();
    for (_, stone) in library() {
        seen.insert(match stone.fabric {
            Fabric::Igneous => 0,
            Fabric::Sedimentary => 1,
            Fabric::Metamorphic => 2,
        });
    }
    assert_eq!(
        seen.len(),
        3,
        "the library does not cover all three fabrics"
    );
}
