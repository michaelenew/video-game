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
const STAIR_PEAK0: [Prop; 2] = peak(-48.0, 66.4, -16.5, 30.0);
const STAIR_PEAK1: [Prop; 2] = peak(88.0, 76.4, 18.5, 36.0);
const STAIR_PEAK2: [Prop; 2] = peak(17.5, 62.4, 67.0, 40.0);
const STAIR_PEAK3: [Prop; 2] = peak(-12.5, 80.4, -55.0, 28.0);
const STAIR_PEAK4: [Prop; 2] = peak(78.0, 86.4, -50.0, 24.0);
const STAIR_PEAK5: [Prop; 2] = peak(-43.0, 72.4, 52.0, 26.0);
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
const CAUSEWAY_PEAK0: [Prop; 2] = peak(-48.0, 72.5, -19.5, 30.0);
const CAUSEWAY_PEAK1: [Prop; 2] = peak(131.0, 82.5, 15.5, 36.0);
const CAUSEWAY_PEAK2: [Prop; 2] = peak(39.0, 68.5, 60.0, 40.0);
const CAUSEWAY_PEAK3: [Prop; 2] = peak(9.0, 86.5, -54.0, 28.0);
const CAUSEWAY_PEAK4: [Prop; 2] = peak(121.0, 92.5, -49.0, 24.0);
const CAUSEWAY_PEAK5: [Prop; 2] = peak(-43.0, 78.5, 45.0, 26.0);
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

// The Gallery
const GALLERY_PEAK0: [Prop; 2] = peak(-48.0, 64.2, -19.0, 30.0);
const GALLERY_PEAK1: [Prop; 2] = peak(72.0, 74.2, 16.0, 36.0);
const GALLERY_PEAK2: [Prop; 2] = peak(9.5, 60.2, 61.0, 40.0);
const GALLERY_PEAK3: [Prop; 2] = peak(-20.5, 78.2, -54.0, 28.0);
const GALLERY_PEAK4: [Prop; 2] = peak(62.0, 84.2, -49.0, 24.0);
const GALLERY_PEAK5: [Prop; 2] = peak(-43.0, 70.2, 46.0, 26.0);
const GALLERY_CAIRN3: [Prop; 3] = cairn(11.7, 70.3, 4.1);
const GALLERY_CAIRN6: [Prop; 3] = cairn(16.4, 70.3, 4.1);
const GALLERY_NEST: [Prop; 11] = nest(21.1, 70.3, 4.3);
const GALLERY_DEEP0: [Prop; 2] = spire(33.0, -49.4, 12.1);
const GALLERY_DEEP1: [Prop; 2] = spire(67.8, -34.4, 22.0);
const GALLERY_DEEP2: [Prop; 2] = spire(11.9, -22.6, 17.9);
const GALLERY_DEEP3: [Prop; 2] = spire(47.9, 54.2, 20.5);
const GALLERY_DEEP4: [Prop; 2] = spire(-1.1, -29.2, 20.3);
const GALLERY_DEEP5: [Prop; 2] = spire(20.0, -2.8, 19.2);
const GALLERY_DEEP6: [Prop; 2] = spire(59.3, -19.4, 9.0);
const GALLERY_DEEP7: [Prop; 2] = spire(28.5, 48.5, 22.0);
const GALLERY_DEEP8: [Prop; 2] = spire(-50.7, -47.6, 8.1);
const GALLERY_DEEP9: [Prop; 2] = spire(-27.4, 47.6, 13.9);
const GALLERY_DEEP10: [Prop; 2] = spire(19.0, -44.1, 9.4);
const GALLERY_DEEP11: [Prop; 2] = spire(17.2, -30.9, 15.1);
const GALLERY_DEEP12: [Prop; 2] = spire(-24.5, -52.1, 9.3);
const GALLERY_DEEP13: [Prop; 2] = spire(39.3, -38.8, 11.6);

pub static GALLERY: Dressing = Dressing {
    sky: SKY,
    below: Some(DEEP),
    props: &[
        GALLERY_DEEP0[0],
        GALLERY_DEEP0[1],
        GALLERY_DEEP1[0],
        GALLERY_DEEP1[1],
        GALLERY_DEEP2[0],
        GALLERY_DEEP2[1],
        GALLERY_DEEP3[0],
        GALLERY_DEEP3[1],
        GALLERY_DEEP4[0],
        GALLERY_DEEP4[1],
        GALLERY_DEEP5[0],
        GALLERY_DEEP5[1],
        GALLERY_DEEP6[0],
        GALLERY_DEEP6[1],
        GALLERY_DEEP7[0],
        GALLERY_DEEP7[1],
        GALLERY_DEEP8[0],
        GALLERY_DEEP8[1],
        GALLERY_DEEP9[0],
        GALLERY_DEEP9[1],
        GALLERY_DEEP10[0],
        GALLERY_DEEP10[1],
        GALLERY_DEEP11[0],
        GALLERY_DEEP11[1],
        GALLERY_DEEP12[0],
        GALLERY_DEEP12[1],
        GALLERY_DEEP13[0],
        GALLERY_DEEP13[1],
        GALLERY_PEAK0[0],
        GALLERY_PEAK0[1],
        GALLERY_PEAK1[0],
        GALLERY_PEAK1[1],
        GALLERY_PEAK2[0],
        GALLERY_PEAK2[1],
        GALLERY_PEAK3[0],
        GALLERY_PEAK3[1],
        GALLERY_PEAK4[0],
        GALLERY_PEAK4[1],
        GALLERY_PEAK5[0],
        GALLERY_PEAK5[1],
        GALLERY_CAIRN3[0],
        GALLERY_CAIRN3[1],
        GALLERY_CAIRN3[2],
        GALLERY_CAIRN6[0],
        GALLERY_CAIRN6[1],
        GALLERY_CAIRN6[2],
        GALLERY_NEST[0],
        GALLERY_NEST[1],
        GALLERY_NEST[2],
        GALLERY_NEST[3],
        GALLERY_NEST[4],
        GALLERY_NEST[5],
        GALLERY_NEST[6],
        GALLERY_NEST[7],
        GALLERY_NEST[8],
        GALLERY_NEST[9],
        GALLERY_NEST[10],
    ],
};

// The Narrows
const NARROWS_PEAK0: [Prop; 2] = peak(-48.0, 69.0, -18.5, 30.0);
const NARROWS_PEAK1: [Prop; 2] = peak(72.0, 79.0, 16.5, 36.0);
const NARROWS_PEAK2: [Prop; 2] = peak(9.5, 65.0, 62.0, 40.0);
const NARROWS_PEAK3: [Prop; 2] = peak(-20.5, 83.0, -54.0, 28.0);
const NARROWS_PEAK4: [Prop; 2] = peak(62.0, 89.0, -49.0, 24.0);
const NARROWS_PEAK5: [Prop; 2] = peak(-43.0, 75.0, 47.0, 26.0);
const NARROWS_CAIRN3: [Prop; 3] = cairn(11.4, 75.0, 4.6);
const NARROWS_CAIRN6: [Prop; 3] = cairn(17.6, 75.0, 3.4);
const NARROWS_NEST: [Prop; 11] = nest(20.9, 75.0, 4.3);
const NARROWS_DEEP0: [Prop; 2] = spire(33.0, -49.4, 12.1);
const NARROWS_DEEP1: [Prop; 2] = spire(67.8, -34.3, 22.0);
const NARROWS_DEEP2: [Prop; 2] = spire(11.9, -22.3, 17.9);
const NARROWS_DEEP3: [Prop; 2] = spire(47.9, 55.2, 20.5);
const NARROWS_DEEP4: [Prop; 2] = spire(-1.1, -28.9, 20.3);
const NARROWS_DEEP5: [Prop; 2] = spire(20.0, -2.3, 19.2);
const NARROWS_DEEP6: [Prop; 2] = spire(59.3, -19.1, 9.0);
const NARROWS_DEEP7: [Prop; 2] = spire(28.5, 49.5, 22.0);
const NARROWS_DEEP8: [Prop; 2] = spire(-50.7, -47.5, 8.1);
const NARROWS_DEEP9: [Prop; 2] = spire(-27.4, 48.6, 13.9);
const NARROWS_DEEP10: [Prop; 2] = spire(19.0, -44.0, 9.4);
const NARROWS_DEEP11: [Prop; 2] = spire(17.2, -30.6, 15.1);
const NARROWS_DEEP12: [Prop; 2] = spire(-24.5, -52.1, 9.3);
const NARROWS_DEEP13: [Prop; 2] = spire(39.3, -38.7, 11.6);

pub static NARROWS: Dressing = Dressing {
    sky: SKY,
    below: Some(DEEP),
    props: &[
        NARROWS_DEEP0[0],
        NARROWS_DEEP0[1],
        NARROWS_DEEP1[0],
        NARROWS_DEEP1[1],
        NARROWS_DEEP2[0],
        NARROWS_DEEP2[1],
        NARROWS_DEEP3[0],
        NARROWS_DEEP3[1],
        NARROWS_DEEP4[0],
        NARROWS_DEEP4[1],
        NARROWS_DEEP5[0],
        NARROWS_DEEP5[1],
        NARROWS_DEEP6[0],
        NARROWS_DEEP6[1],
        NARROWS_DEEP7[0],
        NARROWS_DEEP7[1],
        NARROWS_DEEP8[0],
        NARROWS_DEEP8[1],
        NARROWS_DEEP9[0],
        NARROWS_DEEP9[1],
        NARROWS_DEEP10[0],
        NARROWS_DEEP10[1],
        NARROWS_DEEP11[0],
        NARROWS_DEEP11[1],
        NARROWS_DEEP12[0],
        NARROWS_DEEP12[1],
        NARROWS_DEEP13[0],
        NARROWS_DEEP13[1],
        NARROWS_PEAK0[0],
        NARROWS_PEAK0[1],
        NARROWS_PEAK1[0],
        NARROWS_PEAK1[1],
        NARROWS_PEAK2[0],
        NARROWS_PEAK2[1],
        NARROWS_PEAK3[0],
        NARROWS_PEAK3[1],
        NARROWS_PEAK4[0],
        NARROWS_PEAK4[1],
        NARROWS_PEAK5[0],
        NARROWS_PEAK5[1],
        NARROWS_CAIRN3[0],
        NARROWS_CAIRN3[1],
        NARROWS_CAIRN3[2],
        NARROWS_CAIRN6[0],
        NARROWS_CAIRN6[1],
        NARROWS_CAIRN6[2],
        NARROWS_NEST[0],
        NARROWS_NEST[1],
        NARROWS_NEST[2],
        NARROWS_NEST[3],
        NARROWS_NEST[4],
        NARROWS_NEST[5],
        NARROWS_NEST[6],
        NARROWS_NEST[7],
        NARROWS_NEST[8],
        NARROWS_NEST[9],
        NARROWS_NEST[10],
    ],
};

// The Sill
const SILL_PEAK0: [Prop; 2] = peak(-48.0, 74.6, -21.5, 30.0);
const SILL_PEAK1: [Prop; 2] = peak(73.0, 84.6, 13.5, 36.0);
const SILL_PEAK2: [Prop; 2] = peak(10.0, 70.6, 59.0, 40.0);
const SILL_PEAK3: [Prop; 2] = peak(-20.0, 88.6, -57.0, 28.0);
const SILL_PEAK4: [Prop; 2] = peak(63.0, 94.6, -52.0, 24.0);
const SILL_PEAK5: [Prop; 2] = peak(-43.0, 80.6, 44.0, 26.0);
const SILL_CAIRN3: [Prop; 3] = cairn(11.9, 80.6, -4.7);
const SILL_CAIRN6: [Prop; 3] = cairn(16.6, 81.2, -5.2);
const SILL_NEST: [Prop; 11] = nest(21.2, 81.2, -5.1);
const SILL_DEEP0: [Prop; 2] = spire(43.2, -22.7, 19.3);
const SILL_DEEP1: [Prop; 2] = spire(47.0, -22.4, 19.4);
const SILL_DEEP2: [Prop; 2] = spire(-8.4, -9.2, 13.3);
const SILL_DEEP3: [Prop; 2] = spire(8.1, 6.6, 13.3);
const SILL_DEEP4: [Prop; 2] = spire(22.6, 3.7, 18.2);
const SILL_DEEP5: [Prop; 2] = spire(-18.3, -12.8, 19.2);
const SILL_DEEP6: [Prop; 2] = spire(58.1, -31.8, 21.5);
const SILL_DEEP7: [Prop; 2] = spire(12.7, 1.0, 13.0);
const SILL_DEEP8: [Prop; 2] = spire(-4.9, -28.3, 19.2);
const SILL_DEEP9: [Prop; 2] = spire(13.2, 22.1, 14.4);
const SILL_DEEP10: [Prop; 2] = spire(-3.4, 13.3, 16.8);
const SILL_DEEP11: [Prop; 2] = spire(61.0, 49.2, 15.5);
const SILL_DEEP12: [Prop; 2] = spire(12.7, 46.3, 13.8);
const SILL_DEEP13: [Prop; 2] = spire(46.4, 6.0, 8.2);

pub static SILL: Dressing = Dressing {
    sky: SKY,
    below: Some(DEEP),
    props: &[
        SILL_DEEP0[0],
        SILL_DEEP0[1],
        SILL_DEEP1[0],
        SILL_DEEP1[1],
        SILL_DEEP2[0],
        SILL_DEEP2[1],
        SILL_DEEP3[0],
        SILL_DEEP3[1],
        SILL_DEEP4[0],
        SILL_DEEP4[1],
        SILL_DEEP5[0],
        SILL_DEEP5[1],
        SILL_DEEP6[0],
        SILL_DEEP6[1],
        SILL_DEEP7[0],
        SILL_DEEP7[1],
        SILL_DEEP8[0],
        SILL_DEEP8[1],
        SILL_DEEP9[0],
        SILL_DEEP9[1],
        SILL_DEEP10[0],
        SILL_DEEP10[1],
        SILL_DEEP11[0],
        SILL_DEEP11[1],
        SILL_DEEP12[0],
        SILL_DEEP12[1],
        SILL_DEEP13[0],
        SILL_DEEP13[1],
        SILL_PEAK0[0],
        SILL_PEAK0[1],
        SILL_PEAK1[0],
        SILL_PEAK1[1],
        SILL_PEAK2[0],
        SILL_PEAK2[1],
        SILL_PEAK3[0],
        SILL_PEAK3[1],
        SILL_PEAK4[0],
        SILL_PEAK4[1],
        SILL_PEAK5[0],
        SILL_PEAK5[1],
        SILL_CAIRN3[0],
        SILL_CAIRN3[1],
        SILL_CAIRN3[2],
        SILL_CAIRN6[0],
        SILL_CAIRN6[1],
        SILL_CAIRN6[2],
        SILL_NEST[0],
        SILL_NEST[1],
        SILL_NEST[2],
        SILL_NEST[3],
        SILL_NEST[4],
        SILL_NEST[5],
        SILL_NEST[6],
        SILL_NEST[7],
        SILL_NEST[8],
        SILL_NEST[9],
        SILL_NEST[10],
    ],
};

// The Spire
const SPIRE_PEAK0: [Prop; 2] = peak(-48.0, 73.2, -19.5, 30.0);
const SPIRE_PEAK1: [Prop; 2] = peak(83.0, 83.2, 15.5, 36.0);
const SPIRE_PEAK2: [Prop; 2] = peak(15.0, 69.2, 60.0, 40.0);
const SPIRE_PEAK3: [Prop; 2] = peak(-15.0, 87.2, -54.0, 28.0);
const SPIRE_PEAK4: [Prop; 2] = peak(73.0, 93.2, -49.0, 24.0);
const SPIRE_PEAK5: [Prop; 2] = peak(-43.0, 79.2, 45.0, 26.0);
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

// The Eyrie
const EYRIE_PEAK0: [Prop; 2] = peak(-48.0, 74.8, -19.5, 30.0);
const EYRIE_PEAK1: [Prop; 2] = peak(82.0, 84.8, 15.5, 36.0);
const EYRIE_PEAK2: [Prop; 2] = peak(14.5, 70.8, 59.0, 40.0);
const EYRIE_PEAK3: [Prop; 2] = peak(-15.5, 88.8, -53.0, 28.0);
const EYRIE_PEAK4: [Prop; 2] = peak(72.0, 94.8, -48.0, 24.0);
const EYRIE_PEAK5: [Prop; 2] = peak(-43.0, 80.8, 44.0, 26.0);
const EYRIE_CAIRN2: [Prop; 3] = cairn(18.6, 72.0, -2.4);
const EYRIE_NEST: [Prop; 11] = nest(28.0, 110.0, 0.0);
const EYRIE_DEEP0: [Prop; 2] = spire(-38.9, 19.3, 10.9);
const EYRIE_DEEP1: [Prop; 2] = spire(53.4, -50.9, 13.1);
const EYRIE_DEEP2: [Prop; 2] = spire(24.9, 20.4, 10.1);
const EYRIE_DEEP3: [Prop; 2] = spire(47.9, -42.1, 9.2);
const EYRIE_DEEP4: [Prop; 2] = spire(44.2, 16.0, 13.0);
const EYRIE_DEEP5: [Prop; 2] = spire(62.9, 46.3, 10.6);
const EYRIE_DEEP6: [Prop; 2] = spire(67.4, -36.8, 20.8);
const EYRIE_DEEP7: [Prop; 2] = spire(-37.8, 37.4, 19.3);
const EYRIE_DEEP8: [Prop; 2] = spire(69.8, 16.7, 19.3);
const EYRIE_DEEP9: [Prop; 2] = spire(-15.9, 32.4, 12.8);
const EYRIE_DEEP10: [Prop; 2] = spire(-47.7, -37.1, 8.2);
const EYRIE_DEEP11: [Prop; 2] = spire(57.0, -0.4, 18.7);
const EYRIE_DEEP12: [Prop; 2] = spire(51.2, 48.3, 8.8);
const EYRIE_DEEP13: [Prop; 2] = spire(-8.4, -7.3, 9.1);

pub static EYRIE: Dressing = Dressing {
    sky: SKY,
    below: Some(DEEP),
    props: &[
        EYRIE_DEEP0[0],
        EYRIE_DEEP0[1],
        EYRIE_DEEP1[0],
        EYRIE_DEEP1[1],
        EYRIE_DEEP2[0],
        EYRIE_DEEP2[1],
        EYRIE_DEEP3[0],
        EYRIE_DEEP3[1],
        EYRIE_DEEP4[0],
        EYRIE_DEEP4[1],
        EYRIE_DEEP5[0],
        EYRIE_DEEP5[1],
        EYRIE_DEEP6[0],
        EYRIE_DEEP6[1],
        EYRIE_DEEP7[0],
        EYRIE_DEEP7[1],
        EYRIE_DEEP8[0],
        EYRIE_DEEP8[1],
        EYRIE_DEEP9[0],
        EYRIE_DEEP9[1],
        EYRIE_DEEP10[0],
        EYRIE_DEEP10[1],
        EYRIE_DEEP11[0],
        EYRIE_DEEP11[1],
        EYRIE_DEEP12[0],
        EYRIE_DEEP12[1],
        EYRIE_DEEP13[0],
        EYRIE_DEEP13[1],
        EYRIE_PEAK0[0],
        EYRIE_PEAK0[1],
        EYRIE_PEAK1[0],
        EYRIE_PEAK1[1],
        EYRIE_PEAK2[0],
        EYRIE_PEAK2[1],
        EYRIE_PEAK3[0],
        EYRIE_PEAK3[1],
        EYRIE_PEAK4[0],
        EYRIE_PEAK4[1],
        EYRIE_PEAK5[0],
        EYRIE_PEAK5[1],
        EYRIE_CAIRN2[0],
        EYRIE_CAIRN2[1],
        EYRIE_CAIRN2[2],
        EYRIE_NEST[0],
        EYRIE_NEST[1],
        EYRIE_NEST[2],
        EYRIE_NEST[3],
        EYRIE_NEST[4],
        EYRIE_NEST[5],
        EYRIE_NEST[6],
        EYRIE_NEST[7],
        EYRIE_NEST[8],
        EYRIE_NEST[9],
        EYRIE_NEST[10],
    ],
};

// The Gulf
const GULF_PEAK0: [Prop; 2] = peak(-48.0, 79.0, -20.0, 30.0);
const GULF_PEAK1: [Prop; 2] = peak(94.0, 89.0, 15.0, 36.0);
const GULF_PEAK2: [Prop; 2] = peak(20.5, 75.0, 59.0, 40.0);
const GULF_PEAK3: [Prop; 2] = peak(-9.5, 93.0, -54.0, 28.0);
const GULF_PEAK4: [Prop; 2] = peak(84.0, 99.0, -49.0, 24.0);
const GULF_PEAK5: [Prop; 2] = peak(-43.0, 85.0, 44.0, 26.0);
const GULF_CAIRN2: [Prop; 3] = cairn(26.1, 85.0, -0.9);
const GULF_NEST: [Prop; 11] = nest(42.0, 85.0, 0.0);
const GULF_DEEP0: [Prop; 2] = spire(59.2, -20.6, 19.3);
const GULF_DEEP1: [Prop; 2] = spire(63.7, -20.3, 19.4);
const GULF_DEEP2: [Prop; 2] = spire(-1.0, -7.5, 13.3);
const GULF_DEEP3: [Prop; 2] = spire(18.3, 7.9, 13.3);
const GULF_DEEP4: [Prop; 2] = spire(35.2, 5.0, 18.2);
const GULF_DEEP5: [Prop; 2] = spire(-12.5, -11.0, 19.2);
const GULF_DEEP6: [Prop; 2] = spire(76.7, -29.5, 21.5);
const GULF_DEEP7: [Prop; 2] = spire(23.6, 2.4, 13.0);
const GULF_DEEP8: [Prop; 2] = spire(3.1, -26.1, 19.2);
const GULF_DEEP9: [Prop; 2] = spire(24.2, 23.0, 14.4);
const GULF_DEEP10: [Prop; 2] = spire(4.9, 14.4, 16.8);
const GULF_DEEP11: [Prop; 2] = spire(80.0, 49.3, 15.5);
const GULF_DEEP12: [Prop; 2] = spire(23.7, 46.5, 13.8);
const GULF_DEEP13: [Prop; 2] = spire(63.0, 7.3, 8.2);

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
