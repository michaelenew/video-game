//! The jump courses' dressing (`sim::arena::climb`): the Hallelujah
//! Mountains under a hazy dawn. The islands hang sixty metres and more over
//! a cloud deck wider than the eye reaches, so the drop under every gap reads
//! as a long fall into bright air (the sky is `look::skies::ALOFT`); far
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
const DEEP_ROCK: [f32; 3] = [0.24, 0.24, 0.26];
const CANOPY: [f32; 3] = [0.16, 0.24, 0.14];
const CAIRN: [f32; 3] = [0.62, 0.60, 0.55];
const STICK: [f32; 3] = [0.36, 0.26, 0.16];
const EGG: [f32; 3] = [0.90, 0.88, 0.80];
/// A distance mark: one block per five metres at a takeoff edge.
const PIP: [f32; 3] = [0.95, 0.75, 0.20];

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

/// A distance mark on the Reach: a block at a takeoff edge, one per five
/// metres of the jump it marks.
const fn pip(x: f32, y: f32, z: f32) -> Prop {
    prop(Shape::Box, [x, y + 0.15, z], [0.3, 0.3, 0.3], 0.0, PIP)
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
    drop: true,
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
    drop: true,
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
const SPIRAL_PEAK0: [Prop; 2] = peak(-113.0, 42.7, -37.0, 30.0);
const SPIRAL_PEAK1: [Prop; 2] = peak(141.0, 50.7, 33.0, 36.0);
const SPIRAL_PEAK2: [Prop; 2] = peak(79.0, 36.7, 160.0, 40.0);
const SPIRAL_PEAK3: [Prop; 2] = peak(-71.0, 54.7, -124.0, 28.0);
const SPIRAL_PEAK4: [Prop; 2] = peak(121.0, 46.7, -114.0, 24.0);
const SPIRAL_PEAK5: [Prop; 2] = peak(-103.0, 38.7, 130.0, 26.0);
const SPIRAL_CAIRN5: [Prop; 3] = cairn(11.9, 71.8, 6.3);
const SPIRAL_CAIRN9: [Prop; 3] = cairn(19.4, 74.8, -4.1);
const SPIRAL_CAIRN13: [Prop; 3] = cairn(6.3, 77.8, -4.6);
const SPIRAL_CAIRN17: [Prop; 3] = cairn(10.1, 80.5, 13.6);
const SPIRAL_CAIRN18: [Prop; 3] = cairn(12.4, 81.4, 6.3);
const SPIRAL_CAIRN22: [Prop; 3] = cairn(4.1, 85.0, -13.4);
const SPIRAL_NEST: [Prop; 11] = nest(13.0, 85.6, 0.0);
const SPIRAL_DEEP0: [Prop; 2] = spire(13.0, -3.3, 20.8);
const SPIRAL_DEEP1: [Prop; 2] = spire(-6.8, -57.2, 19.7);
const SPIRAL_DEEP2: [Prop; 2] = spire(-48.5, -38.2, 20.5);
const SPIRAL_DEEP3: [Prop; 2] = spire(-7.4, -3.6, 10.0);
const SPIRAL_DEEP4: [Prop; 2] = spire(-34.9, 30.4, 10.7);
const SPIRAL_DEEP5: [Prop; 2] = spire(50.9, -47.3, 9.2);
const SPIRAL_DEEP6: [Prop; 2] = spire(29.0, -25.1, 17.2);
const SPIRAL_DEEP7: [Prop; 2] = spire(50.1, -49.5, 13.9);
const SPIRAL_DEEP8: [Prop; 2] = spire(27.5, -0.9, 10.7);
const SPIRAL_DEEP9: [Prop; 2] = spire(-31.9, 8.8, 20.3);
const SPIRAL_DEEP10: [Prop; 2] = spire(-27.5, -15.0, 13.9);
const SPIRAL_DEEP11: [Prop; 2] = spire(3.0, 27.6, 11.4);
const SPIRAL_DEEP12: [Prop; 2] = spire(30.3, -62.7, 15.7);
const SPIRAL_DEEP13: [Prop; 2] = spire(6.5, 23.1, 12.2);

pub static SPIRAL: Dressing = Dressing {
    drop: true,
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
        SPIRAL_CAIRN18[0],
        SPIRAL_CAIRN18[1],
        SPIRAL_CAIRN18[2],
        SPIRAL_CAIRN22[0],
        SPIRAL_CAIRN22[1],
        SPIRAL_CAIRN22[2],
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
const FALLS_PEAK0: [Prop; 2] = peak(-113.0, 37.6, -39.5, 30.0);
const FALLS_PEAK1: [Prop; 2] = peak(247.0, 45.6, 30.5, 36.0);
const FALLS_PEAK2: [Prop; 2] = peak(132.0, 31.6, 146.0, 40.0);
const FALLS_PEAK3: [Prop; 2] = peak(-18.0, 49.6, -115.0, 28.0);
const FALLS_PEAK4: [Prop; 2] = peak(227.0, 41.6, -105.0, 24.0);
const FALLS_PEAK5: [Prop; 2] = peak(-103.0, 33.6, 116.0, 26.0);
const FALLS_CAIRN6: [Prop; 3] = cairn(36.4, 79.0, -0.9);
const FALLS_CAIRN7: [Prop; 3] = cairn(72.4, 74.0, -4.4);
const FALLS_CAIRN11: [Prop; 3] = cairn(98.7, 68.8, -2.9);
const FALLS_CAIRN14: [Prop; 3] = cairn(112.2, 64.9, -2.4);
const FALLS_NEST: [Prop; 11] = nest(124.6, 65.9, 0.0);
const FALLS_DEEP0: [Prop; 2] = spire(-29.0, 20.0, 10.9);
const FALLS_DEEP1: [Prop; 2] = spire(128.3, -52.8, 13.1);
const FALLS_DEEP2: [Prop; 2] = spire(79.7, 21.2, 10.1);
const FALLS_DEEP3: [Prop; 2] = spire(118.9, -43.7, 9.2);
const FALLS_DEEP4: [Prop; 2] = spire(112.7, 16.6, 13.0);
const FALLS_DEEP5: [Prop; 2] = spire(144.5, 48.0, 10.6);
const FALLS_DEEP6: [Prop; 2] = spire(152.1, -38.2, 20.8);
const FALLS_DEEP7: [Prop; 2] = spire(-27.1, 38.8, 19.3);
const FALLS_DEEP8: [Prop; 2] = spire(156.2, 17.4, 19.3);
const FALLS_DEEP9: [Prop; 2] = spire(10.2, 33.6, 12.8);
const FALLS_DEEP10: [Prop; 2] = spire(-43.9, -38.6, 8.2);
const FALLS_DEEP11: [Prop; 2] = spire(134.4, -0.5, 18.7);
const FALLS_DEEP12: [Prop; 2] = spire(124.5, 50.1, 8.8);
const FALLS_DEEP13: [Prop; 2] = spire(22.9, -7.6, 9.1);

pub static FALLS: Dressing = Dressing {
    drop: true,
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
const SLALOM_PEAK0: [Prop; 2] = peak(-113.0, 36.0, -44.5, 30.0);
const SLALOM_PEAK1: [Prop; 2] = peak(202.0, 44.0, 25.5, 36.0);
const SLALOM_PEAK2: [Prop; 2] = peak(109.5, 30.0, 146.0, 40.0);
const SLALOM_PEAK3: [Prop; 2] = peak(-40.5, 48.0, -125.0, 28.0);
const SLALOM_PEAK4: [Prop; 2] = peak(182.0, 40.0, -115.0, 24.0);
const SLALOM_PEAK5: [Prop; 2] = peak(-103.0, 32.0, 116.0, 26.0);
const SLALOM_CAIRN4: [Prop; 3] = cairn(18.0, 70.0, -3.7);
const SLALOM_CAIRN8: [Prop; 3] = cairn(33.2, 70.4, -3.7);
const SLALOM_CAIRN11: [Prop; 3] = cairn(52.6, 70.0, -1.4);
const SLALOM_CAIRN12: [Prop; 3] = cairn(42.6, 70.0, -14.4);
const SLALOM_NEST: [Prop; 11] = nest(78.0, 67.0, 0.0);
const SLALOM_DEEP0: [Prop; 2] = spire(45.5, -10.2, 20.8);
const SLALOM_DEEP1: [Prop; 2] = spire(15.9, -58.9, 19.7);
const SLALOM_DEEP2: [Prop; 2] = spire(-46.3, -41.7, 20.5);
const SLALOM_DEEP3: [Prop; 2] = spire(15.0, -10.4, 10.0);
const SLALOM_DEEP4: [Prop; 2] = spire(-25.9, 20.3, 10.7);
const SLALOM_DEEP5: [Prop; 2] = spire(102.0, -49.9, 9.2);
const SLALOM_DEEP6: [Prop; 2] = spire(69.4, -29.9, 17.2);
const SLALOM_DEEP7: [Prop; 2] = spire(100.9, -51.9, 13.9);
const SLALOM_DEEP8: [Prop; 2] = spire(67.1, -8.0, 10.7);
const SLALOM_DEEP9: [Prop; 2] = spire(-21.5, 0.7, 20.3);
const SLALOM_DEEP10: [Prop; 2] = spire(-14.9, -20.8, 13.9);
const SLALOM_DEEP11: [Prop; 2] = spire(30.5, 17.7, 11.4);
const SLALOM_DEEP12: [Prop; 2] = spire(71.3, -63.9, 15.7);
const SLALOM_DEEP13: [Prop; 2] = spire(35.8, 13.7, 12.2);

pub static SLALOM: Dressing = Dressing {
    drop: true,
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
        SLALOM_CAIRN12[0],
        SLALOM_CAIRN12[1],
        SLALOM_CAIRN12[2],
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
const FORK_PEAK0: [Prop; 2] = peak(-113.0, 41.4, -39.0, 30.0);
const FORK_PEAK1: [Prop; 2] = peak(240.0, 49.4, 31.0, 36.0);
const FORK_PEAK2: [Prop; 2] = peak(128.5, 35.4, 147.0, 40.0);
const FORK_PEAK3: [Prop; 2] = peak(-21.5, 53.4, -115.0, 28.0);
const FORK_PEAK4: [Prop; 2] = peak(220.0, 45.4, -105.0, 24.0);
const FORK_PEAK5: [Prop; 2] = peak(-103.0, 37.4, 117.0, 26.0);
const FORK_CAIRN1: [Prop; 3] = cairn(4.6, 75.0, -3.4);
const FORK_CAIRN2: [Prop; 3] = cairn(13.6, 83.0, 2.1);
const FORK_CAIRN13: [Prop; 3] = cairn(52.6, 75.0, -3.4);
const FORK_CAIRN14: [Prop; 3] = cairn(75.6, 71.0, -4.4);
const FORK_CAIRN15: [Prop; 3] = cairn(88.6, 72.0, -1.4);
const FORK_NEST: [Prop; 11] = nest(116.5, 73.0, 0.0);
const FORK_DEEP0: [Prop; 2] = spire(117.3, -20.4, 19.3);
const FORK_DEEP1: [Prop; 2] = spire(124.1, -20.1, 19.4);
const FORK_DEEP2: [Prop; 2] = spire(25.9, -6.7, 13.3);
const FORK_DEEP3: [Prop; 2] = spire(55.2, 9.2, 13.3);
const FORK_DEEP4: [Prop; 2] = spire(80.8, 6.2, 18.2);
const FORK_DEEP5: [Prop; 2] = spire(8.4, -10.4, 19.2);
const FORK_DEEP6: [Prop; 2] = spire(143.7, -29.6, 21.5);
const FORK_DEEP7: [Prop; 2] = spire(63.2, 3.5, 13.0);
const FORK_DEEP8: [Prop; 2] = spire(32.1, -26.0, 19.2);
const FORK_DEEP9: [Prop; 2] = spire(64.1, 24.8, 14.4);
const FORK_DEEP10: [Prop; 2] = spire(34.9, 15.9, 16.8);
const FORK_DEEP11: [Prop; 2] = spire(148.7, 52.1, 15.5);
const FORK_DEEP12: [Prop; 2] = spire(63.3, 49.2, 13.8);
const FORK_DEEP13: [Prop; 2] = spire(123.0, 8.6, 8.2);

pub static FORK: Dressing = Dressing {
    drop: true,
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
        FORK_CAIRN2[0],
        FORK_CAIRN2[1],
        FORK_CAIRN2[2],
        FORK_CAIRN13[0],
        FORK_CAIRN13[1],
        FORK_CAIRN13[2],
        FORK_CAIRN14[0],
        FORK_CAIRN14[1],
        FORK_CAIRN14[2],
        FORK_CAIRN15[0],
        FORK_CAIRN15[1],
        FORK_CAIRN15[2],
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

// The Waterfall: the valley's basalt cliff hanging in the courses' air, so
// the deep and the peaks are the same family, a cairn on each shelf and the
// shore, and the nest on the top (`sim::arena::waterfall`, `COURSE_AT`).
const WATERFALL_PEAK0: [Prop; 2] = peak(-110.0, 44.0, -40.0, 30.0);
const WATERFALL_PEAK1: [Prop; 2] = peak(150.0, 52.0, 35.0, 36.0);
const WATERFALL_PEAK2: [Prop; 2] = peak(90.0, 38.0, 140.0, 40.0);
const WATERFALL_PEAK3: [Prop; 2] = peak(-60.0, 56.0, -115.0, 28.0);
const WATERFALL_PEAK4: [Prop; 2] = peak(130.0, 48.0, -105.0, 24.0);
const WATERFALL_DEEP0: [Prop; 2] = spire(-30.0, 18.0, 11.0);
const WATERFALL_DEEP1: [Prop; 2] = spire(60.0, -40.0, 13.0);
const WATERFALL_DEEP2: [Prop; 2] = spire(18.0, 32.0, 10.0);
const WATERFALL_DEEP3: [Prop; 2] = spire(75.0, 28.0, 14.0);
const WATERFALL_CAIRN_SHORE: [Prop; 3] = cairn(6.0, 60.0, -8.5);
const WATERFALL_CAIRN1: [Prop; 3] = cairn(30.5, 66.0, -15.5);
const WATERFALL_CAIRN2: [Prop; 3] = cairn(30.5, 73.5, 14.5);
const WATERFALL_NEST: [Prop; 11] = nest(49.0, 86.0, -12.0);

pub static WATERFALL: Dressing = Dressing {
    drop: true,
    props: &[
        WATERFALL_PEAK0[0],
        WATERFALL_PEAK0[1],
        WATERFALL_PEAK1[0],
        WATERFALL_PEAK1[1],
        WATERFALL_PEAK2[0],
        WATERFALL_PEAK2[1],
        WATERFALL_PEAK3[0],
        WATERFALL_PEAK3[1],
        WATERFALL_PEAK4[0],
        WATERFALL_PEAK4[1],
        WATERFALL_DEEP0[0],
        WATERFALL_DEEP0[1],
        WATERFALL_DEEP1[0],
        WATERFALL_DEEP1[1],
        WATERFALL_DEEP2[0],
        WATERFALL_DEEP2[1],
        WATERFALL_DEEP3[0],
        WATERFALL_DEEP3[1],
        WATERFALL_CAIRN_SHORE[0],
        WATERFALL_CAIRN_SHORE[1],
        WATERFALL_CAIRN_SHORE[2],
        WATERFALL_CAIRN1[0],
        WATERFALL_CAIRN1[1],
        WATERFALL_CAIRN1[2],
        WATERFALL_CAIRN2[0],
        WATERFALL_CAIRN2[1],
        WATERFALL_CAIRN2[2],
        WATERFALL_NEST[0],
        WATERFALL_NEST[1],
        WATERFALL_NEST[2],
        WATERFALL_NEST[3],
        WATERFALL_NEST[4],
        WATERFALL_NEST[5],
        WATERFALL_NEST[6],
        WATERFALL_NEST[7],
        WATERFALL_NEST[8],
        WATERFALL_NEST[9],
        WATERFALL_NEST[10],
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
    drop: true,
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
const GULF_PEAK0: [Prop; 2] = peak(-113.0, 53.0, -33.0, 30.0);
const GULF_PEAK1: [Prop; 2] = peak(218.0, 61.0, 37.0, 36.0);
const GULF_PEAK2: [Prop; 2] = peak(117.5, 47.0, 157.0, 40.0);
const GULF_PEAK3: [Prop; 2] = peak(-32.5, 65.0, -113.0, 28.0);
const GULF_PEAK4: [Prop; 2] = peak(198.0, 57.0, -103.0, 24.0);
const GULF_PEAK5: [Prop; 2] = peak(-103.0, 49.0, 127.0, 26.0);
const GULF_CAIRN2: [Prop; 3] = cairn(41.6, 85.0, -1.9);
const GULF_CAIRN3: [Prop; 3] = cairn(63.6, 88.0, -1.9);
const GULF_CAIRN4: [Prop; 3] = cairn(65.6, 95.0, 10.6);
const GULF_NEST: [Prop; 11] = nest(94.0, 84.0, 0.0);
const GULF_DEEP0: [Prop; 2] = spire(100.5, -15.9, 19.3);
const GULF_DEEP1: [Prop; 2] = spire(106.6, -15.6, 19.4);
const GULF_DEEP2: [Prop; 2] = spire(18.2, -1.3, 13.3);
const GULF_DEEP3: [Prop; 2] = spire(44.5, 15.8, 13.3);
const GULF_DEEP4: [Prop; 2] = spire(67.6, 12.6, 18.2);
const GULF_DEEP5: [Prop; 2] = spire(2.3, -5.2, 19.2);
const GULF_DEEP6: [Prop; 2] = spire(124.3, -25.8, 21.5);
const GULF_DEEP7: [Prop; 2] = spire(51.8, 9.7, 13.0);
const GULF_DEEP8: [Prop; 2] = spire(23.7, -21.9, 19.2);
const GULF_DEEP9: [Prop; 2] = spire(52.6, 32.5, 14.4);
const GULF_DEEP10: [Prop; 2] = spire(26.2, 23.0, 16.8);
const GULF_DEEP11: [Prop; 2] = spire(128.8, 61.8, 15.5);
const GULF_DEEP12: [Prop; 2] = spire(51.8, 58.7, 13.8);
const GULF_DEEP13: [Prop; 2] = spire(105.6, 15.1, 8.2);

pub static GULF: Dressing = Dressing {
    drop: true,
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
        GULF_CAIRN3[0],
        GULF_CAIRN3[1],
        GULF_CAIRN3[2],
        GULF_CAIRN4[0],
        GULF_CAIRN4[1],
        GULF_CAIRN4[2],
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

// The Reach
const REACH_PEAK0: [Prop; 2] = peak(-135.0, 26.0, -31.0, 30.0);
const REACH_PEAK1: [Prop; 2] = peak(179.0, 34.0, 39.0, 36.0);
const REACH_PEAK2: [Prop; 2] = peak(87.0, 20.0, 204.0, 40.0);
const REACH_PEAK3: [Prop; 2] = peak(-63.0, 38.0, -156.0, 28.0);
const REACH_PEAK4: [Prop; 2] = peak(159.0, 30.0, -146.0, 24.0);
const REACH_PEAK5: [Prop; 2] = peak(-125.0, 22.0, 174.0, 26.0);
const REACH_NEST: [Prop; 11] = nest(55.0, 60.0, 24.0);
const REACH_DEEP0: [Prop; 2] = spire(21.9, -58.0, 8.5);
const REACH_DEEP1: [Prop; 2] = spire(9.3, 11.2, 10.4);
const REACH_DEEP2: [Prop; 2] = spire(35.5, 108.5, 19.1);
const REACH_DEEP3: [Prop; 2] = spire(-56.8, 32.6, 17.8);
const REACH_DEEP4: [Prop; 2] = spire(81.7, -56.3, 11.6);
const REACH_DEEP5: [Prop; 2] = spire(37.1, 47.6, 8.5);
const REACH_DEEP6: [Prop; 2] = spire(62.0, -95.3, 13.2);
const REACH_DEEP7: [Prop; 2] = spire(13.5, 68.8, 20.8);
const REACH_DEEP8: [Prop; 2] = spire(25.0, 3.0, 17.1);
const REACH_DEEP9: [Prop; 2] = spire(-46.4, 63.9, 11.3);
const REACH_DEEP10: [Prop; 2] = spire(-44.9, 96.8, 9.1);
const REACH_DEEP11: [Prop; 2] = spire(-24.1, 70.8, 10.9);
const REACH_DEEP12: [Prop; 2] = spire(34.8, -47.0, 21.2);
const REACH_DEEP13: [Prop; 2] = spire(101.0, 81.6, 17.4);
const REACH_PIP0: Prop = pip(1.5, 60.0, -24.2);
const REACH_PIP1: Prop = pip(1.5, 60.0, -23.8);
const REACH_PIP2: Prop = pip(1.5, 60.0, -16.5);
const REACH_PIP3: Prop = pip(1.5, 60.0, -16.0);
const REACH_PIP4: Prop = pip(1.5, 60.0, -15.5);
const REACH_PIP5: Prop = pip(1.5, 60.0, -8.8);
const REACH_PIP6: Prop = pip(1.5, 60.0, -8.2);
const REACH_PIP7: Prop = pip(1.5, 60.0, -7.8);
const REACH_PIP8: Prop = pip(1.5, 60.0, -7.2);
const REACH_PIP9: Prop = pip(1.5, 60.0, -1.0);
const REACH_PIP10: Prop = pip(1.5, 60.0, -0.5);
const REACH_PIP11: Prop = pip(1.5, 60.0, 0.0);
const REACH_PIP12: Prop = pip(1.5, 60.0, 0.5);
const REACH_PIP13: Prop = pip(1.5, 60.0, 1.0);
const REACH_PIP14: Prop = pip(1.5, 60.0, 6.8);
const REACH_PIP15: Prop = pip(1.5, 60.0, 7.2);
const REACH_PIP16: Prop = pip(1.5, 60.0, 7.8);
const REACH_PIP17: Prop = pip(1.5, 60.0, 8.2);
const REACH_PIP18: Prop = pip(1.5, 60.0, 8.8);
const REACH_PIP19: Prop = pip(1.5, 60.0, 9.2);
const REACH_PIP20: Prop = pip(1.5, 60.0, 14.2);
const REACH_PIP21: Prop = pip(1.5, 60.0, 14.8);
const REACH_PIP22: Prop = pip(1.5, 60.0, 15.2);
const REACH_PIP23: Prop = pip(1.5, 60.0, 15.8);
const REACH_PIP24: Prop = pip(1.5, 60.0, 16.2);
const REACH_PIP25: Prop = pip(1.5, 60.0, 16.8);
const REACH_PIP26: Prop = pip(1.5, 60.0, 17.2);
const REACH_PIP27: Prop = pip(1.5, 60.0, 17.8);
const REACH_PIP28: Prop = pip(1.5, 60.0, 21.8);
const REACH_PIP29: Prop = pip(1.5, 60.0, 22.2);
const REACH_PIP30: Prop = pip(1.5, 60.0, 22.8);
const REACH_PIP31: Prop = pip(1.5, 60.0, 23.2);
const REACH_PIP32: Prop = pip(1.5, 60.0, 23.8);
const REACH_PIP33: Prop = pip(1.5, 60.0, 24.2);
const REACH_PIP34: Prop = pip(1.5, 60.0, 24.8);
const REACH_PIP35: Prop = pip(1.5, 60.0, 25.2);
const REACH_PIP36: Prop = pip(1.5, 60.0, 25.8);
const REACH_PIP37: Prop = pip(1.5, 60.0, 26.2);
const REACH_PIP38: Prop = pip(-17.5, 60.0, -20.0);
const REACH_PIP39: Prop = pip(-17.5, 60.0, -12.2);
const REACH_PIP40: Prop = pip(-17.5, 60.0, -11.8);
const REACH_PIP41: Prop = pip(-17.5, 60.0, -4.5);
const REACH_PIP42: Prop = pip(-17.5, 60.0, -4.0);
const REACH_PIP43: Prop = pip(-17.5, 60.0, -3.5);
const REACH_PIP44: Prop = pip(-17.5, 60.0, 3.2);
const REACH_PIP45: Prop = pip(-17.5, 60.0, 3.8);
const REACH_PIP46: Prop = pip(-17.5, 60.0, 4.2);
const REACH_PIP47: Prop = pip(-17.5, 60.0, 4.8);
const REACH_PIP48: Prop = pip(-17.5, 60.0, 10.8);
const REACH_PIP49: Prop = pip(-17.5, 60.0, 11.2);
const REACH_PIP50: Prop = pip(-17.5, 60.0, 11.8);
const REACH_PIP51: Prop = pip(-17.5, 60.0, 12.2);
const REACH_PIP52: Prop = pip(-17.5, 60.0, 12.8);
const REACH_PIP53: Prop = pip(-17.5, 60.0, 13.2);
const REACH_PIP54: Prop = pip(-17.5, 60.0, 18.0);
const REACH_PIP55: Prop = pip(-17.5, 60.0, 18.5);
const REACH_PIP56: Prop = pip(-17.5, 60.0, 19.0);
const REACH_PIP57: Prop = pip(-17.5, 60.0, 19.5);
const REACH_PIP58: Prop = pip(-17.5, 60.0, 20.0);
const REACH_PIP59: Prop = pip(-17.5, 60.0, 20.5);
const REACH_PIP60: Prop = pip(-17.5, 60.0, 21.0);
const REACH_PIP61: Prop = pip(-17.5, 60.0, 21.5);
const REACH_PIP62: Prop = pip(-17.5, 60.0, 22.0);
const REACH_PIP63: Prop = pip(-14.5, 60.0, 31.0);
const REACH_PIP64: Prop = pip(-14.0, 60.0, 31.0);
const REACH_PIP65: Prop = pip(-13.5, 60.0, 31.0);
const REACH_PIP66: Prop = pip(-5.0, 60.0, 31.0);
const REACH_PIP67: Prop = pip(-4.5, 60.0, 31.0);
const REACH_PIP68: Prop = pip(-4.0, 60.0, 31.0);
const REACH_PIP69: Prop = pip(-3.5, 60.0, 31.0);
const REACH_PIP70: Prop = pip(-3.0, 60.0, 31.0);
const REACH_PIP71: Prop = pip(-11.0, 60.0, -31.0);
const REACH_PIP72: Prop = pip(-3.2, 60.0, -31.0);
const REACH_PIP73: Prop = pip(-2.8, 60.0, -31.0);

pub static REACH: Dressing = Dressing {
    drop: true,
    props: &[
        REACH_DEEP0[0],
        REACH_DEEP0[1],
        REACH_DEEP1[0],
        REACH_DEEP1[1],
        REACH_DEEP2[0],
        REACH_DEEP2[1],
        REACH_DEEP3[0],
        REACH_DEEP3[1],
        REACH_DEEP4[0],
        REACH_DEEP4[1],
        REACH_DEEP5[0],
        REACH_DEEP5[1],
        REACH_DEEP6[0],
        REACH_DEEP6[1],
        REACH_DEEP7[0],
        REACH_DEEP7[1],
        REACH_DEEP8[0],
        REACH_DEEP8[1],
        REACH_DEEP9[0],
        REACH_DEEP9[1],
        REACH_DEEP10[0],
        REACH_DEEP10[1],
        REACH_DEEP11[0],
        REACH_DEEP11[1],
        REACH_DEEP12[0],
        REACH_DEEP12[1],
        REACH_DEEP13[0],
        REACH_DEEP13[1],
        REACH_PEAK0[0],
        REACH_PEAK0[1],
        REACH_PEAK1[0],
        REACH_PEAK1[1],
        REACH_PEAK2[0],
        REACH_PEAK2[1],
        REACH_PEAK3[0],
        REACH_PEAK3[1],
        REACH_PEAK4[0],
        REACH_PEAK4[1],
        REACH_PEAK5[0],
        REACH_PEAK5[1],
        REACH_PIP0,
        REACH_PIP1,
        REACH_PIP2,
        REACH_PIP3,
        REACH_PIP4,
        REACH_PIP5,
        REACH_PIP6,
        REACH_PIP7,
        REACH_PIP8,
        REACH_PIP9,
        REACH_PIP10,
        REACH_PIP11,
        REACH_PIP12,
        REACH_PIP13,
        REACH_PIP14,
        REACH_PIP15,
        REACH_PIP16,
        REACH_PIP17,
        REACH_PIP18,
        REACH_PIP19,
        REACH_PIP20,
        REACH_PIP21,
        REACH_PIP22,
        REACH_PIP23,
        REACH_PIP24,
        REACH_PIP25,
        REACH_PIP26,
        REACH_PIP27,
        REACH_PIP28,
        REACH_PIP29,
        REACH_PIP30,
        REACH_PIP31,
        REACH_PIP32,
        REACH_PIP33,
        REACH_PIP34,
        REACH_PIP35,
        REACH_PIP36,
        REACH_PIP37,
        REACH_PIP38,
        REACH_PIP39,
        REACH_PIP40,
        REACH_PIP41,
        REACH_PIP42,
        REACH_PIP43,
        REACH_PIP44,
        REACH_PIP45,
        REACH_PIP46,
        REACH_PIP47,
        REACH_PIP48,
        REACH_PIP49,
        REACH_PIP50,
        REACH_PIP51,
        REACH_PIP52,
        REACH_PIP53,
        REACH_PIP54,
        REACH_PIP55,
        REACH_PIP56,
        REACH_PIP57,
        REACH_PIP58,
        REACH_PIP59,
        REACH_PIP60,
        REACH_PIP61,
        REACH_PIP62,
        REACH_PIP63,
        REACH_PIP64,
        REACH_PIP65,
        REACH_PIP66,
        REACH_PIP67,
        REACH_PIP68,
        REACH_PIP69,
        REACH_PIP70,
        REACH_PIP71,
        REACH_PIP72,
        REACH_PIP73,
        REACH_NEST[0],
        REACH_NEST[1],
        REACH_NEST[2],
        REACH_NEST[3],
        REACH_NEST[4],
        REACH_NEST[5],
        REACH_NEST[6],
        REACH_NEST[7],
        REACH_NEST[8],
        REACH_NEST[9],
        REACH_NEST[10],
    ],
};
