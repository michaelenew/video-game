//! The jump courses' dressing (`sim::arena::climb`): the Hallelujah
//! Mountains under a hazy dawn. The islands hang sixty metres and more over
//! a floor nearly black and wider than the eye reaches, so the drop under
//! every gap reads as a long fall into the dark; far
//! floating peaks on every side; far below, spires of rock crowned with
//! trees, so the eye has something to measure the drop by; a cairn of stones at each
//! checkpoint; and on the last island a nest of sticks round three pale eggs
//! -- the rookery, where the Galewing would live. None of it collides: every
//! island, bridge and overhang is the simulation's. Generated with the
//! courses, from the same hop lists, so a cairn stands on the island its
//! checkpoint is.

use super::{Dressing, Prop, Shape};

const PEAK: [f32; 3] = [0.56, 0.61, 0.65];
const MOSS: [f32; 3] = [0.30, 0.42, 0.26];
/// The floor of the world, seen from sixty metres and more: nearly black.
const DEEP: [f32; 3] = [0.05, 0.06, 0.08];
const DEEP_ROCK: [f32; 3] = [0.24, 0.24, 0.26];
const CANOPY: [f32; 3] = [0.16, 0.24, 0.14];
const CAIRN: [f32; 3] = [0.62, 0.60, 0.55];
const STICK: [f32; 3] = [0.36, 0.26, 0.16];
const EGG: [f32; 3] = [0.90, 0.88, 0.80];

/// The hazy dawn behind everything.
const SKY: [f32; 3] = [0.66, 0.74, 0.80];

const fn prop(shape: Shape, at: [f32; 3], size: [f32; 3], yaw: f32, rgb: [f32; 3]) -> Prop {
    Prop {
        shape,
        at,
        size,
        yaw,
        rgb,
    }
}

/// A far peak hanging in the haze: a rock with moss on its crown.
const fn peak(x: f32, y: f32, z: f32, d: f32) -> [Prop; 2] {
    [
        prop(Shape::Sphere, [x, y, z], [d, d * 0.8, d], 0.0, PEAK),
        prop(
            Shape::Cylinder,
            [x, y + d * 0.62, z],
            [d * 0.7, d * 0.12, d * 0.7],
            0.0,
            MOSS,
        ),
    ]
}

/// Far below: a spire of rock standing on the floor, its tip still tens of
/// metres under the islands -- something for the eye to measure the drop by.
const fn spire(x: f32, z: f32, h: f32) -> [Prop; 2] {
    [
        prop(
            Shape::Cylinder,
            [x, 0.0, z],
            [h * 0.14, h, h * 0.14],
            0.0,
            DEEP_ROCK,
        ),
        prop(
            Shape::Sphere,
            [x, h * 0.92, z],
            [h * 0.22, h * 0.18, h * 0.22],
            0.0,
            CANOPY,
        ),
    ]
}

/// A cairn: three stones stacked at the corner of a checkpoint's top.
const fn cairn(x: f32, y: f32, z: f32) -> [Prop; 3] {
    [
        prop(Shape::Sphere, [x, y - 0.15, z], [0.7, 0.5, 0.7], 0.0, CAIRN),
        prop(Shape::Sphere, [x, y + 0.2, z], [0.5, 0.4, 0.5], 0.0, CAIRN),
        prop(
            Shape::Sphere,
            [x, y + 0.5, z],
            [0.32, 0.3, 0.32],
            0.0,
            CAIRN,
        ),
    ]
}

/// The nest on the last island: a ring of sticks round three eggs.
const fn nest(x: f32, y: f32, z: f32) -> [Prop; 11] {
    let r = 1.6;
    [
        prop(Shape::Box, [x + r, y, z], [0.3, 0.45, 2.4], 0.0, STICK),
        prop(Shape::Box, [x - r, y, z], [0.3, 0.45, 2.4], 0.0, STICK),
        prop(Shape::Box, [x, y, z + r], [2.4, 0.45, 0.3], 0.0, STICK),
        prop(Shape::Box, [x, y, z - r], [2.4, 0.45, 0.3], 0.0, STICK),
        prop(
            Shape::Box,
            [x + 1.1, y, z + 1.1],
            [0.3, 0.4, 1.6],
            0.125,
            STICK,
        ),
        prop(
            Shape::Box,
            [x - 1.1, y, z - 1.1],
            [0.3, 0.4, 1.6],
            0.125,
            STICK,
        ),
        prop(
            Shape::Box,
            [x + 1.1, y, z - 1.1],
            [0.3, 0.4, 1.6],
            -0.125,
            STICK,
        ),
        prop(
            Shape::Box,
            [x - 1.1, y, z + 1.1],
            [0.3, 0.4, 1.6],
            -0.125,
            STICK,
        ),
        prop(Shape::Sphere, [x - 0.3, y, z], [0.45, 0.6, 0.45], 0.0, EGG),
        prop(
            Shape::Sphere,
            [x + 0.35, y, z + 0.2],
            [0.45, 0.6, 0.45],
            0.0,
            EGG,
        ),
        prop(
            Shape::Sphere,
            [x + 0.1, y, z - 0.4],
            [0.45, 0.6, 0.45],
            0.0,
            EGG,
        ),
    ]
}

// The Stair
const STAIR_PEAK0: [Prop; 2] = peak(-113.0, 38.4, -36.5, 30.0);
const STAIR_PEAK1: [Prop; 2] = peak(158.0, 46.4, 33.5, 36.0);
const STAIR_PEAK2: [Prop; 2] = peak(87.5, 32.4, 152.0, 40.0);
const STAIR_PEAK3: [Prop; 2] = peak(-62.5, 50.4, -115.0, 28.0);
const STAIR_PEAK4: [Prop; 2] = peak(138.0, 42.4, -105.0, 24.0);
const STAIR_PEAK5: [Prop; 2] = peak(-103.0, 34.4, 122.0, 26.0);
const STAIR_CAIRN3: [Prop; 3] = cairn(14.6, 72.5, 6.6);
const STAIR_NEST: [Prop; 11] = nest(33.5, 77.0, -0.5);
const STAIR_DEEP0: [Prop; 2] = spire(-38.3, 24.0, 10.9);
const STAIR_DEEP1: [Prop; 2] = spire(58.1, -52.7, 13.1);
const STAIR_DEEP2: [Prop; 2] = spire(28.3, 25.3, 10.1);
const STAIR_DEEP3: [Prop; 2] = spire(52.4, -43.1, 9.2);
const STAIR_DEEP4: [Prop; 2] = spire(48.6, 20.4, 13.0);
const STAIR_DEEP5: [Prop; 2] = spire(68.1, 53.6, 10.6);
const STAIR_DEEP6: [Prop; 2] = spire(72.7, -37.3, 20.8);
const STAIR_DEEP7: [Prop; 2] = spire(-37.1, 43.9, 19.3);
const STAIR_DEEP8: [Prop; 2] = spire(75.2, 21.3, 19.3);
const STAIR_DEEP9: [Prop; 2] = spire(-14.2, 38.3, 12.8);
const STAIR_DEEP10: [Prop; 2] = spire(-47.4, -37.7, 8.2);
const STAIR_DEEP11: [Prop; 2] = spire(61.9, 2.5, 18.7);
const STAIR_DEEP12: [Prop; 2] = spire(55.8, 55.8, 8.8);
const STAIR_DEEP13: [Prop; 2] = spire(-6.5, -5.0, 9.1);

pub static STAIR: Dressing = Dressing {
    sky: SKY,
    below: Some(DEEP),
    props: &[
        STAIR_DEEP0[0],
        STAIR_DEEP0[1],
        STAIR_DEEP1[0],
        STAIR_DEEP1[1],
        STAIR_DEEP2[0],
        STAIR_DEEP2[1],
        STAIR_DEEP3[0],
        STAIR_DEEP3[1],
        STAIR_DEEP4[0],
        STAIR_DEEP4[1],
        STAIR_DEEP5[0],
        STAIR_DEEP5[1],
        STAIR_DEEP6[0],
        STAIR_DEEP6[1],
        STAIR_DEEP7[0],
        STAIR_DEEP7[1],
        STAIR_DEEP8[0],
        STAIR_DEEP8[1],
        STAIR_DEEP9[0],
        STAIR_DEEP9[1],
        STAIR_DEEP10[0],
        STAIR_DEEP10[1],
        STAIR_DEEP11[0],
        STAIR_DEEP11[1],
        STAIR_DEEP12[0],
        STAIR_DEEP12[1],
        STAIR_DEEP13[0],
        STAIR_DEEP13[1],
        STAIR_PEAK0[0],
        STAIR_PEAK0[1],
        STAIR_PEAK1[0],
        STAIR_PEAK1[1],
        STAIR_PEAK2[0],
        STAIR_PEAK2[1],
        STAIR_PEAK3[0],
        STAIR_PEAK3[1],
        STAIR_PEAK4[0],
        STAIR_PEAK4[1],
        STAIR_PEAK5[0],
        STAIR_PEAK5[1],
        STAIR_CAIRN3[0],
        STAIR_CAIRN3[1],
        STAIR_CAIRN3[2],
        STAIR_NEST[0],
        STAIR_NEST[1],
        STAIR_NEST[2],
        STAIR_NEST[3],
        STAIR_NEST[4],
        STAIR_NEST[5],
        STAIR_NEST[6],
        STAIR_NEST[7],
        STAIR_NEST[8],
        STAIR_NEST[9],
        STAIR_NEST[10],
    ],
};

// The Causeway
const CAUSEWAY_PEAK0: [Prop; 2] = peak(-113.0, 44.5, -39.5, 30.0);
const CAUSEWAY_PEAK1: [Prop; 2] = peak(201.0, 52.5, 30.5, 36.0);
const CAUSEWAY_PEAK2: [Prop; 2] = peak(109.0, 38.5, 145.0, 40.0);
const CAUSEWAY_PEAK3: [Prop; 2] = peak(-41.0, 56.5, -114.0, 28.0);
const CAUSEWAY_PEAK4: [Prop; 2] = peak(181.0, 48.5, -104.0, 24.0);
const CAUSEWAY_PEAK5: [Prop; 2] = peak(-103.0, 40.5, 115.0, 26.0);
const CAUSEWAY_CAIRN4: [Prop; 3] = cairn(32.1, 79.0, -0.4);
const CAUSEWAY_NEST: [Prop; 11] = nest(76.0, 77.0, 0.0);
const CAUSEWAY_DEEP0: [Prop; 2] = spire(-37.6, -27.3, 12.5);
const CAUSEWAY_DEEP1: [Prop; 2] = spire(87.2, -17.8, 19.7);
const CAUSEWAY_DEEP2: [Prop; 2] = spire(16.1, 23.5, 16.8);
const CAUSEWAY_DEEP3: [Prop; 2] = spire(25.6, 34.9, 15.5);
const CAUSEWAY_DEEP4: [Prop; 2] = spire(117.1, -1.4, 12.2);
const CAUSEWAY_DEEP5: [Prop; 2] = spire(36.8, 39.2, 10.2);
const CAUSEWAY_DEEP6: [Prop; 2] = spire(42.2, -39.2, 18.1);
const CAUSEWAY_DEEP7: [Prop; 2] = spire(-3.7, 3.4, 19.9);
const CAUSEWAY_DEEP8: [Prop; 2] = spire(21.4, 0.6, 8.5);
const CAUSEWAY_DEEP9: [Prop; 2] = spire(126.2, -37.9, 21.3);
const CAUSEWAY_DEEP10: [Prop; 2] = spire(-7.9, -33.3, 11.9);
const CAUSEWAY_DEEP11: [Prop; 2] = spire(-33.3, 34.5, 12.6);
const CAUSEWAY_DEEP12: [Prop; 2] = spire(24.3, -48.2, 21.3);
const CAUSEWAY_DEEP13: [Prop; 2] = spire(46.7, 16.5, 18.2);

pub static CAUSEWAY: Dressing = Dressing {
    sky: SKY,
    below: Some(DEEP),
    props: &[
        CAUSEWAY_DEEP0[0],
        CAUSEWAY_DEEP0[1],
        CAUSEWAY_DEEP1[0],
        CAUSEWAY_DEEP1[1],
        CAUSEWAY_DEEP2[0],
        CAUSEWAY_DEEP2[1],
        CAUSEWAY_DEEP3[0],
        CAUSEWAY_DEEP3[1],
        CAUSEWAY_DEEP4[0],
        CAUSEWAY_DEEP4[1],
        CAUSEWAY_DEEP5[0],
        CAUSEWAY_DEEP5[1],
        CAUSEWAY_DEEP6[0],
        CAUSEWAY_DEEP6[1],
        CAUSEWAY_DEEP7[0],
        CAUSEWAY_DEEP7[1],
        CAUSEWAY_DEEP8[0],
        CAUSEWAY_DEEP8[1],
        CAUSEWAY_DEEP9[0],
        CAUSEWAY_DEEP9[1],
        CAUSEWAY_DEEP10[0],
        CAUSEWAY_DEEP10[1],
        CAUSEWAY_DEEP11[0],
        CAUSEWAY_DEEP11[1],
        CAUSEWAY_DEEP12[0],
        CAUSEWAY_DEEP12[1],
        CAUSEWAY_DEEP13[0],
        CAUSEWAY_DEEP13[1],
        CAUSEWAY_PEAK0[0],
        CAUSEWAY_PEAK0[1],
        CAUSEWAY_PEAK1[0],
        CAUSEWAY_PEAK1[1],
        CAUSEWAY_PEAK2[0],
        CAUSEWAY_PEAK2[1],
        CAUSEWAY_PEAK3[0],
        CAUSEWAY_PEAK3[1],
        CAUSEWAY_PEAK4[0],
        CAUSEWAY_PEAK4[1],
        CAUSEWAY_PEAK5[0],
        CAUSEWAY_PEAK5[1],
        CAUSEWAY_CAIRN4[0],
        CAUSEWAY_CAIRN4[1],
        CAUSEWAY_CAIRN4[2],
        CAUSEWAY_NEST[0],
        CAUSEWAY_NEST[1],
        CAUSEWAY_NEST[2],
        CAUSEWAY_NEST[3],
        CAUSEWAY_NEST[4],
        CAUSEWAY_NEST[5],
        CAUSEWAY_NEST[6],
        CAUSEWAY_NEST[7],
        CAUSEWAY_NEST[8],
        CAUSEWAY_NEST[9],
        CAUSEWAY_NEST[10],
    ],
};

// The Spiral
const SPIRAL_PEAK0: [Prop; 2] = peak(-113.0, 42.2, -40.0, 30.0);
const SPIRAL_PEAK1: [Prop; 2] = peak(141.0, 50.2, 30.0, 36.0);
const SPIRAL_PEAK2: [Prop; 2] = peak(79.0, 36.2, 148.0, 40.0);
const SPIRAL_PEAK3: [Prop; 2] = peak(-71.0, 54.2, -118.0, 28.0);
const SPIRAL_PEAK4: [Prop; 2] = peak(121.0, 46.2, -108.0, 24.0);
const SPIRAL_PEAK5: [Prop; 2] = peak(-103.0, 38.2, 118.0, 26.0);
const SPIRAL_CAIRN5: [Prop; 3] = cairn(11.9, 71.8, 6.3);
const SPIRAL_CAIRN9: [Prop; 3] = cairn(19.4, 74.8, -4.1);
const SPIRAL_CAIRN13: [Prop; 3] = cairn(6.3, 77.8, -4.6);
const SPIRAL_CAIRN17: [Prop; 3] = cairn(12.4, 81.4, 6.3);
const SPIRAL_NEST: [Prop; 11] = nest(13.0, 85.6, 0.0);
const SPIRAL_DEEP0: [Prop; 2] = spire(13.0, -5.5, 20.8);
const SPIRAL_DEEP1: [Prop; 2] = spire(-6.8, -52.1, 19.7);
const SPIRAL_DEEP2: [Prop; 2] = spire(-48.5, -35.7, 20.5);
const SPIRAL_DEEP3: [Prop; 2] = spire(-7.4, -5.7, 10.0);
const SPIRAL_DEEP4: [Prop; 2] = spire(-34.9, 23.7, 10.7);
const SPIRAL_DEEP5: [Prop; 2] = spire(50.9, -43.5, 9.2);
const SPIRAL_DEEP6: [Prop; 2] = spire(29.0, -24.3, 17.2);
const SPIRAL_DEEP7: [Prop; 2] = spire(50.1, -45.4, 13.9);
const SPIRAL_DEEP8: [Prop; 2] = spire(27.5, -3.3, 10.7);
const SPIRAL_DEEP9: [Prop; 2] = spire(-31.9, 5.0, 20.3);
const SPIRAL_DEEP10: [Prop; 2] = spire(-27.5, -15.6, 13.9);
const SPIRAL_DEEP11: [Prop; 2] = spire(3.0, 21.3, 11.4);
const SPIRAL_DEEP12: [Prop; 2] = spire(30.3, -56.9, 15.7);
const SPIRAL_DEEP13: [Prop; 2] = spire(6.5, 17.4, 12.2);

pub static SPIRAL: Dressing = Dressing {
    sky: SKY,
    below: Some(DEEP),
    props: &[
        SPIRAL_DEEP0[0],
        SPIRAL_DEEP0[1],
        SPIRAL_DEEP1[0],
        SPIRAL_DEEP1[1],
        SPIRAL_DEEP2[0],
        SPIRAL_DEEP2[1],
        SPIRAL_DEEP3[0],
        SPIRAL_DEEP3[1],
        SPIRAL_DEEP4[0],
        SPIRAL_DEEP4[1],
        SPIRAL_DEEP5[0],
        SPIRAL_DEEP5[1],
        SPIRAL_DEEP6[0],
        SPIRAL_DEEP6[1],
        SPIRAL_DEEP7[0],
        SPIRAL_DEEP7[1],
        SPIRAL_DEEP8[0],
        SPIRAL_DEEP8[1],
        SPIRAL_DEEP9[0],
        SPIRAL_DEEP9[1],
        SPIRAL_DEEP10[0],
        SPIRAL_DEEP10[1],
        SPIRAL_DEEP11[0],
        SPIRAL_DEEP11[1],
        SPIRAL_DEEP12[0],
        SPIRAL_DEEP12[1],
        SPIRAL_DEEP13[0],
        SPIRAL_DEEP13[1],
        SPIRAL_PEAK0[0],
        SPIRAL_PEAK0[1],
        SPIRAL_PEAK1[0],
        SPIRAL_PEAK1[1],
        SPIRAL_PEAK2[0],
        SPIRAL_PEAK2[1],
        SPIRAL_PEAK3[0],
        SPIRAL_PEAK3[1],
        SPIRAL_PEAK4[0],
        SPIRAL_PEAK4[1],
        SPIRAL_PEAK5[0],
        SPIRAL_PEAK5[1],
        SPIRAL_CAIRN5[0],
        SPIRAL_CAIRN5[1],
        SPIRAL_CAIRN5[2],
        SPIRAL_CAIRN9[0],
        SPIRAL_CAIRN9[1],
        SPIRAL_CAIRN9[2],
        SPIRAL_CAIRN13[0],
        SPIRAL_CAIRN13[1],
        SPIRAL_CAIRN13[2],
        SPIRAL_CAIRN17[0],
        SPIRAL_CAIRN17[1],
        SPIRAL_CAIRN17[2],
        SPIRAL_NEST[0],
        SPIRAL_NEST[1],
        SPIRAL_NEST[2],
        SPIRAL_NEST[3],
        SPIRAL_NEST[4],
        SPIRAL_NEST[5],
        SPIRAL_NEST[6],
        SPIRAL_NEST[7],
        SPIRAL_NEST[8],
        SPIRAL_NEST[9],
        SPIRAL_NEST[10],
    ],
};

// The Falls
const FALLS_PEAK0: [Prop; 2] = peak(-113.0, 38.5, -40.0, 30.0);
const FALLS_PEAK1: [Prop; 2] = peak(232.0, 46.5, 30.0, 36.0);
const FALLS_PEAK2: [Prop; 2] = peak(124.5, 32.5, 145.0, 40.0);
const FALLS_PEAK3: [Prop; 2] = peak(-25.5, 50.5, -115.0, 28.0);
const FALLS_PEAK4: [Prop; 2] = peak(212.0, 42.5, -105.0, 24.0);
const FALLS_PEAK5: [Prop; 2] = peak(-103.0, 34.5, 115.0, 26.0);
const FALLS_CAIRN6: [Prop; 3] = cairn(29.9, 79.0, -0.9);
const FALLS_CAIRN7: [Prop; 3] = cairn(57.9, 75.5, -3.9);
const FALLS_CAIRN11: [Prop; 3] = cairn(83.2, 70.3, -2.9);
const FALLS_CAIRN14: [Prop; 3] = cairn(96.7, 66.4, -2.4);
const FALLS_NEST: [Prop; 11] = nest(109.1, 67.4, 0.0);
const FALLS_DEEP0: [Prop; 2] = spire(-30.5, 19.3, 10.9);
const FALLS_DEEP1: [Prop; 2] = spire(116.4, -52.8, 13.1);
const FALLS_DEEP2: [Prop; 2] = spire(71.0, 20.5, 10.1);
const FALLS_DEEP3: [Prop; 2] = spire(107.7, -43.8, 9.2);
const FALLS_DEEP4: [Prop; 2] = spire(101.9, 15.9, 13.0);
const FALLS_DEEP5: [Prop; 2] = spire(131.6, 47.1, 10.6);
const FALLS_DEEP6: [Prop; 2] = spire(138.7, -38.3, 20.8);
const FALLS_DEEP7: [Prop; 2] = spire(-28.8, 38.0, 19.3);
const FALLS_DEEP8: [Prop; 2] = spire(142.5, 16.7, 19.3);
const FALLS_DEEP9: [Prop; 2] = spire(6.1, 32.8, 12.8);
const FALLS_DEEP10: [Prop; 2] = spire(-44.5, -38.7, 8.2);
const FALLS_DEEP11: [Prop; 2] = spire(122.1, -0.9, 18.7);
const FALLS_DEEP12: [Prop; 2] = spire(112.9, 49.2, 8.8);
const FALLS_DEEP13: [Prop; 2] = spire(18.0, -8.0, 9.1);

pub static FALLS: Dressing = Dressing {
    sky: SKY,
    below: Some(DEEP),
    props: &[
        FALLS_DEEP0[0],
        FALLS_DEEP0[1],
        FALLS_DEEP1[0],
        FALLS_DEEP1[1],
        FALLS_DEEP2[0],
        FALLS_DEEP2[1],
        FALLS_DEEP3[0],
        FALLS_DEEP3[1],
        FALLS_DEEP4[0],
        FALLS_DEEP4[1],
        FALLS_DEEP5[0],
        FALLS_DEEP5[1],
        FALLS_DEEP6[0],
        FALLS_DEEP6[1],
        FALLS_DEEP7[0],
        FALLS_DEEP7[1],
        FALLS_DEEP8[0],
        FALLS_DEEP8[1],
        FALLS_DEEP9[0],
        FALLS_DEEP9[1],
        FALLS_DEEP10[0],
        FALLS_DEEP10[1],
        FALLS_DEEP11[0],
        FALLS_DEEP11[1],
        FALLS_DEEP12[0],
        FALLS_DEEP12[1],
        FALLS_DEEP13[0],
        FALLS_DEEP13[1],
        FALLS_PEAK0[0],
        FALLS_PEAK0[1],
        FALLS_PEAK1[0],
        FALLS_PEAK1[1],
        FALLS_PEAK2[0],
        FALLS_PEAK2[1],
        FALLS_PEAK3[0],
        FALLS_PEAK3[1],
        FALLS_PEAK4[0],
        FALLS_PEAK4[1],
        FALLS_PEAK5[0],
        FALLS_PEAK5[1],
        FALLS_CAIRN6[0],
        FALLS_CAIRN6[1],
        FALLS_CAIRN6[2],
        FALLS_CAIRN7[0],
        FALLS_CAIRN7[1],
        FALLS_CAIRN7[2],
        FALLS_CAIRN11[0],
        FALLS_CAIRN11[1],
        FALLS_CAIRN11[2],
        FALLS_CAIRN14[0],
        FALLS_CAIRN14[1],
        FALLS_CAIRN14[2],
        FALLS_NEST[0],
        FALLS_NEST[1],
        FALLS_NEST[2],
        FALLS_NEST[3],
        FALLS_NEST[4],
        FALLS_NEST[5],
        FALLS_NEST[6],
        FALLS_NEST[7],
        FALLS_NEST[8],
        FALLS_NEST[9],
        FALLS_NEST[10],
    ],
};

// The Slalom
const SLALOM_PEAK0: [Prop; 2] = peak(-113.0, 36.3, -39.5, 30.0);
const SLALOM_PEAK1: [Prop; 2] = peak(187.0, 44.3, 30.5, 36.0);
const SLALOM_PEAK2: [Prop; 2] = peak(102.0, 30.3, 146.0, 40.0);
const SLALOM_PEAK3: [Prop; 2] = peak(-48.0, 48.3, -115.0, 28.0);
const SLALOM_PEAK4: [Prop; 2] = peak(167.0, 40.3, -105.0, 24.0);
const SLALOM_PEAK5: [Prop; 2] = peak(-103.0, 32.3, 116.0, 26.0);
const SLALOM_CAIRN4: [Prop; 3] = cairn(18.0, 70.0, -3.7);
const SLALOM_CAIRN8: [Prop; 3] = cairn(33.2, 70.4, -3.7);
const SLALOM_CAIRN11: [Prop; 3] = cairn(52.6, 70.0, -1.4);
const SLALOM_NEST: [Prop; 11] = nest(64.3, 71.0, 0.0);
const SLALOM_DEEP0: [Prop; 2] = spire(37.5, -4.7, 20.8);
const SLALOM_DEEP1: [Prop; 2] = spire(10.4, -49.4, 19.7);
const SLALOM_DEEP2: [Prop; 2] = spire(-46.9, -33.6, 20.5);
const SLALOM_DEEP3: [Prop; 2] = spire(9.5, -5.0, 10.0);
const SLALOM_DEEP4: [Prop; 2] = spire(-28.1, 23.2, 10.7);
const SLALOM_DEEP5: [Prop; 2] = spire(89.4, -41.2, 9.2);
const SLALOM_DEEP6: [Prop; 2] = spire(59.5, -22.8, 17.2);
const SLALOM_DEEP7: [Prop; 2] = spire(88.4, -43.0, 13.9);
const SLALOM_DEEP8: [Prop; 2] = spire(57.3, -2.7, 10.7);
const SLALOM_DEEP9: [Prop; 2] = spire(-24.0, 5.3, 20.3);
const SLALOM_DEEP10: [Prop; 2] = spire(-18.0, -14.4, 13.9);
const SLALOM_DEEP11: [Prop; 2] = spire(23.7, 20.9, 11.4);
const SLALOM_DEEP12: [Prop; 2] = spire(61.3, -54.0, 15.7);
const SLALOM_DEEP13: [Prop; 2] = spire(28.6, 17.2, 12.2);

pub static SLALOM: Dressing = Dressing {
    sky: SKY,
    below: Some(DEEP),
    props: &[
        SLALOM_DEEP0[0],
        SLALOM_DEEP0[1],
        SLALOM_DEEP1[0],
        SLALOM_DEEP1[1],
        SLALOM_DEEP2[0],
        SLALOM_DEEP2[1],
        SLALOM_DEEP3[0],
        SLALOM_DEEP3[1],
        SLALOM_DEEP4[0],
        SLALOM_DEEP4[1],
        SLALOM_DEEP5[0],
        SLALOM_DEEP5[1],
        SLALOM_DEEP6[0],
        SLALOM_DEEP6[1],
        SLALOM_DEEP7[0],
        SLALOM_DEEP7[1],
        SLALOM_DEEP8[0],
        SLALOM_DEEP8[1],
        SLALOM_DEEP9[0],
        SLALOM_DEEP9[1],
        SLALOM_DEEP10[0],
        SLALOM_DEEP10[1],
        SLALOM_DEEP11[0],
        SLALOM_DEEP11[1],
        SLALOM_DEEP12[0],
        SLALOM_DEEP12[1],
        SLALOM_DEEP13[0],
        SLALOM_DEEP13[1],
        SLALOM_PEAK0[0],
        SLALOM_PEAK0[1],
        SLALOM_PEAK1[0],
        SLALOM_PEAK1[1],
        SLALOM_PEAK2[0],
        SLALOM_PEAK2[1],
        SLALOM_PEAK3[0],
        SLALOM_PEAK3[1],
        SLALOM_PEAK4[0],
        SLALOM_PEAK4[1],
        SLALOM_PEAK5[0],
        SLALOM_PEAK5[1],
        SLALOM_CAIRN4[0],
        SLALOM_CAIRN4[1],
        SLALOM_CAIRN4[2],
        SLALOM_CAIRN8[0],
        SLALOM_CAIRN8[1],
        SLALOM_CAIRN8[2],
        SLALOM_CAIRN11[0],
        SLALOM_CAIRN11[1],
        SLALOM_CAIRN11[2],
        SLALOM_NEST[0],
        SLALOM_NEST[1],
        SLALOM_NEST[2],
        SLALOM_NEST[3],
        SLALOM_NEST[4],
        SLALOM_NEST[5],
        SLALOM_NEST[6],
        SLALOM_NEST[7],
        SLALOM_NEST[8],
        SLALOM_NEST[9],
        SLALOM_NEST[10],
    ],
};

// The Fork
const FORK_PEAK0: [Prop; 2] = peak(-113.0, 40.0, -39.5, 30.0);
const FORK_PEAK1: [Prop; 2] = peak(239.0, 48.0, 30.5, 36.0);
const FORK_PEAK2: [Prop; 2] = peak(128.0, 34.0, 146.0, 40.0);
const FORK_PEAK3: [Prop; 2] = peak(-22.0, 52.0, -115.0, 28.0);
const FORK_PEAK4: [Prop; 2] = peak(219.0, 44.0, -105.0, 24.0);
const FORK_PEAK5: [Prop; 2] = peak(-103.0, 36.0, 116.0, 26.0);
const FORK_CAIRN1: [Prop; 3] = cairn(4.6, 75.0, -3.4);
const FORK_CAIRN9: [Prop; 3] = cairn(60.3, 75.0, -3.4);
const FORK_CAIRN10: [Prop; 3] = cairn(75.3, 72.0, -4.4);
const FORK_CAIRN11: [Prop; 3] = cairn(88.3, 73.0, -1.4);
const FORK_NEST: [Prop; 11] = nest(116.2, 74.0, 0.0);
const FORK_DEEP0: [Prop; 2] = spire(116.5, -20.7, 19.3);
const FORK_DEEP1: [Prop; 2] = spire(123.3, -20.4, 19.4);
const FORK_DEEP2: [Prop; 2] = spire(25.6, -7.2, 13.3);
const FORK_DEEP3: [Prop; 2] = spire(54.7, 8.6, 13.3);
const FORK_DEEP4: [Prop; 2] = spire(80.2, 5.7, 18.2);
const FORK_DEEP5: [Prop; 2] = spire(8.1, -10.8, 19.2);
const FORK_DEEP6: [Prop; 2] = spire(142.8, -29.8, 21.5);
const FORK_DEEP7: [Prop; 2] = spire(62.7, 3.0, 13.0);
const FORK_DEEP8: [Prop; 2] = spire(31.8, -26.3, 19.2);
const FORK_DEEP9: [Prop; 2] = spire(63.6, 24.1, 14.4);
const FORK_DEEP10: [Prop; 2] = spire(34.5, 15.3, 16.8);
const FORK_DEEP11: [Prop; 2] = spire(147.8, 51.2, 15.5);
const FORK_DEEP12: [Prop; 2] = spire(62.8, 48.3, 13.8);
const FORK_DEEP13: [Prop; 2] = spire(122.2, 8.0, 8.2);

pub static FORK: Dressing = Dressing {
    sky: SKY,
    below: Some(DEEP),
    props: &[
        FORK_DEEP0[0],
        FORK_DEEP0[1],
        FORK_DEEP1[0],
        FORK_DEEP1[1],
        FORK_DEEP2[0],
        FORK_DEEP2[1],
        FORK_DEEP3[0],
        FORK_DEEP3[1],
        FORK_DEEP4[0],
        FORK_DEEP4[1],
        FORK_DEEP5[0],
        FORK_DEEP5[1],
        FORK_DEEP6[0],
        FORK_DEEP6[1],
        FORK_DEEP7[0],
        FORK_DEEP7[1],
        FORK_DEEP8[0],
        FORK_DEEP8[1],
        FORK_DEEP9[0],
        FORK_DEEP9[1],
        FORK_DEEP10[0],
        FORK_DEEP10[1],
        FORK_DEEP11[0],
        FORK_DEEP11[1],
        FORK_DEEP12[0],
        FORK_DEEP12[1],
        FORK_DEEP13[0],
        FORK_DEEP13[1],
        FORK_PEAK0[0],
        FORK_PEAK0[1],
        FORK_PEAK1[0],
        FORK_PEAK1[1],
        FORK_PEAK2[0],
        FORK_PEAK2[1],
        FORK_PEAK3[0],
        FORK_PEAK3[1],
        FORK_PEAK4[0],
        FORK_PEAK4[1],
        FORK_PEAK5[0],
        FORK_PEAK5[1],
        FORK_CAIRN1[0],
        FORK_CAIRN1[1],
        FORK_CAIRN1[2],
        FORK_CAIRN9[0],
        FORK_CAIRN9[1],
        FORK_CAIRN9[2],
        FORK_CAIRN10[0],
        FORK_CAIRN10[1],
        FORK_CAIRN10[2],
        FORK_CAIRN11[0],
        FORK_CAIRN11[1],
        FORK_CAIRN11[2],
        FORK_NEST[0],
        FORK_NEST[1],
        FORK_NEST[2],
        FORK_NEST[3],
        FORK_NEST[4],
        FORK_NEST[5],
        FORK_NEST[6],
        FORK_NEST[7],
        FORK_NEST[8],
        FORK_NEST[9],
        FORK_NEST[10],
    ],
};

// The Spire
const SPIRE_PEAK0: [Prop; 2] = peak(-113.0, 45.2, -39.5, 30.0);
const SPIRE_PEAK1: [Prop; 2] = peak(153.0, 53.2, 30.5, 36.0);
const SPIRE_PEAK2: [Prop; 2] = peak(85.0, 39.2, 145.0, 40.0);
const SPIRE_PEAK3: [Prop; 2] = peak(-65.0, 57.2, -114.0, 28.0);
const SPIRE_PEAK4: [Prop; 2] = peak(133.0, 49.2, -104.0, 24.0);
const SPIRE_PEAK5: [Prop; 2] = peak(-103.0, 41.2, 115.0, 26.0);
const SPIRE_CAIRN2: [Prop; 3] = cairn(18.6, 72.0, -2.4);
const SPIRE_NEST: [Prop; 11] = nest(28.5, 104.0, 0.0);
const SPIRE_DEEP0: [Prop; 2] = spire(-38.8, 19.6, 10.9);
const SPIRE_DEEP1: [Prop; 2] = spire(54.2, -51.8, 13.1);
const SPIRE_DEEP2: [Prop; 2] = spire(25.5, 20.8, 10.1);
const SPIRE_DEEP3: [Prop; 2] = spire(48.7, -42.9, 9.2);
const SPIRE_DEEP4: [Prop; 2] = spire(45.0, 16.3, 13.0);
const SPIRE_DEEP5: [Prop; 2] = spire(63.8, 47.1, 10.6);
const SPIRE_DEEP6: [Prop; 2] = spire(68.3, -37.5, 20.8);
const SPIRE_DEEP7: [Prop; 2] = spire(-37.7, 38.1, 19.3);
const SPIRE_DEEP8: [Prop; 2] = spire(70.7, 17.0, 19.3);
const SPIRE_DEEP9: [Prop; 2] = spire(-15.6, 33.0, 12.8);
const SPIRE_DEEP10: [Prop; 2] = spire(-47.6, -37.8, 8.2);
const SPIRE_DEEP11: [Prop; 2] = spire(57.8, -0.4, 18.7);
const SPIRE_DEEP12: [Prop; 2] = spire(51.9, 49.2, 8.8);
const SPIRE_DEEP13: [Prop; 2] = spire(-8.1, -7.4, 9.1);

pub static SPIRE: Dressing = Dressing {
    sky: SKY,
    below: Some(DEEP),
    props: &[
        SPIRE_DEEP0[0],
        SPIRE_DEEP0[1],
        SPIRE_DEEP1[0],
        SPIRE_DEEP1[1],
        SPIRE_DEEP2[0],
        SPIRE_DEEP2[1],
        SPIRE_DEEP3[0],
        SPIRE_DEEP3[1],
        SPIRE_DEEP4[0],
        SPIRE_DEEP4[1],
        SPIRE_DEEP5[0],
        SPIRE_DEEP5[1],
        SPIRE_DEEP6[0],
        SPIRE_DEEP6[1],
        SPIRE_DEEP7[0],
        SPIRE_DEEP7[1],
        SPIRE_DEEP8[0],
        SPIRE_DEEP8[1],
        SPIRE_DEEP9[0],
        SPIRE_DEEP9[1],
        SPIRE_DEEP10[0],
        SPIRE_DEEP10[1],
        SPIRE_DEEP11[0],
        SPIRE_DEEP11[1],
        SPIRE_DEEP12[0],
        SPIRE_DEEP12[1],
        SPIRE_DEEP13[0],
        SPIRE_DEEP13[1],
        SPIRE_PEAK0[0],
        SPIRE_PEAK0[1],
        SPIRE_PEAK1[0],
        SPIRE_PEAK1[1],
        SPIRE_PEAK2[0],
        SPIRE_PEAK2[1],
        SPIRE_PEAK3[0],
        SPIRE_PEAK3[1],
        SPIRE_PEAK4[0],
        SPIRE_PEAK4[1],
        SPIRE_PEAK5[0],
        SPIRE_PEAK5[1],
        SPIRE_CAIRN2[0],
        SPIRE_CAIRN2[1],
        SPIRE_CAIRN2[2],
        SPIRE_NEST[0],
        SPIRE_NEST[1],
        SPIRE_NEST[2],
        SPIRE_NEST[3],
        SPIRE_NEST[4],
        SPIRE_NEST[5],
        SPIRE_NEST[6],
        SPIRE_NEST[7],
        SPIRE_NEST[8],
        SPIRE_NEST[9],
        SPIRE_NEST[10],
    ],
};

// The Gulf
const GULF_PEAK0: [Prop; 2] = peak(-113.0, 51.6, -39.5, 30.0);
const GULF_PEAK1: [Prop; 2] = peak(192.0, 59.6, 30.5, 36.0);
const GULF_PEAK2: [Prop; 2] = peak(104.5, 45.6, 144.0, 40.0);
const GULF_PEAK3: [Prop; 2] = peak(-45.5, 63.6, -113.0, 28.0);
const GULF_PEAK4: [Prop; 2] = peak(172.0, 55.6, -103.0, 24.0);
const GULF_PEAK5: [Prop; 2] = peak(-103.0, 47.6, 114.0, 26.0);
const GULF_CAIRN2: [Prop; 3] = cairn(31.6, 85.0, -0.9);
const GULF_CAIRN5: [Prop; 3] = cairn(56.8, 86.5, 0.0);
const GULF_NEST: [Prop; 11] = nest(68.9, 84.5, 0.0);
const GULF_DEEP0: [Prop; 2] = spire(80.6, -19.9, 19.3);
const GULF_DEEP1: [Prop; 2] = spire(85.9, -19.6, 19.4);
const GULF_DEEP2: [Prop; 2] = spire(9.0, -6.9, 13.3);
const GULF_DEEP3: [Prop; 2] = spire(31.9, 8.3, 13.3);
const GULF_DEEP4: [Prop; 2] = spire(52.0, 5.5, 18.2);
const GULF_DEEP5: [Prop; 2] = spire(-4.8, -10.4, 19.2);
const GULF_DEEP6: [Prop; 2] = spire(101.3, -28.7, 21.5);
const GULF_DEEP7: [Prop; 2] = spire(38.2, 2.9, 13.0);
const GULF_DEEP8: [Prop; 2] = spire(13.8, -25.3, 19.2);
const GULF_DEEP9: [Prop; 2] = spire(38.9, 23.2, 14.4);
const GULF_DEEP10: [Prop; 2] = spire(16.0, 14.8, 16.8);
const GULF_DEEP11: [Prop; 2] = spire(105.3, 49.4, 15.5);
const GULF_DEEP12: [Prop; 2] = spire(38.3, 46.6, 13.8);
const GULF_DEEP13: [Prop; 2] = spire(85.1, 7.8, 8.2);

pub static GULF: Dressing = Dressing {
    sky: SKY,
    below: Some(DEEP),
    props: &[
        GULF_DEEP0[0],
        GULF_DEEP0[1],
        GULF_DEEP1[0],
        GULF_DEEP1[1],
        GULF_DEEP2[0],
        GULF_DEEP2[1],
        GULF_DEEP3[0],
        GULF_DEEP3[1],
        GULF_DEEP4[0],
        GULF_DEEP4[1],
        GULF_DEEP5[0],
        GULF_DEEP5[1],
        GULF_DEEP6[0],
        GULF_DEEP6[1],
        GULF_DEEP7[0],
        GULF_DEEP7[1],
        GULF_DEEP8[0],
        GULF_DEEP8[1],
        GULF_DEEP9[0],
        GULF_DEEP9[1],
        GULF_DEEP10[0],
        GULF_DEEP10[1],
        GULF_DEEP11[0],
        GULF_DEEP11[1],
        GULF_DEEP12[0],
        GULF_DEEP12[1],
        GULF_DEEP13[0],
        GULF_DEEP13[1],
        GULF_PEAK0[0],
        GULF_PEAK0[1],
        GULF_PEAK1[0],
        GULF_PEAK1[1],
        GULF_PEAK2[0],
        GULF_PEAK2[1],
        GULF_PEAK3[0],
        GULF_PEAK3[1],
        GULF_PEAK4[0],
        GULF_PEAK4[1],
        GULF_PEAK5[0],
        GULF_PEAK5[1],
        GULF_CAIRN2[0],
        GULF_CAIRN2[1],
        GULF_CAIRN2[2],
        GULF_CAIRN5[0],
        GULF_CAIRN5[1],
        GULF_CAIRN5[2],
        GULF_NEST[0],
        GULF_NEST[1],
        GULF_NEST[2],
        GULF_NEST[3],
        GULF_NEST[4],
        GULF_NEST[5],
        GULF_NEST[6],
        GULF_NEST[7],
        GULF_NEST[8],
        GULF_NEST[9],
        GULF_NEST[10],
    ],
};
