//! The jump courses' dressing (`sim::arena::climb`): the Hallelujah
//! Mountains under a hazy dawn. The islands hang sixty metres and more over
//! a floor nearly black and wider than the eye reaches, so the drop under
//! every gap reads as a long fall into the dark; far
//! floating peaks on every side; a cairn of stones at each
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

pub static STAIR: Dressing = Dressing {
    sky: SKY,
    below: Some(DEEP),
    props: &[
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

pub static CAUSEWAY: Dressing = Dressing {
    sky: SKY,
    below: Some(DEEP),
    props: &[
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

// The Climb
const CLIMB_PEAK0: [Prop; 2] = peak(-48.0, 69.8, -10.0, 30.0);
const CLIMB_PEAK1: [Prop; 2] = peak(134.0, 79.8, 25.0, 36.0);
const CLIMB_PEAK2: [Prop; 2] = peak(40.5, 65.8, 78.0, 40.0);
const CLIMB_PEAK3: [Prop; 2] = peak(10.5, 83.8, -53.0, 28.0);
const CLIMB_PEAK4: [Prop; 2] = peak(124.0, 89.8, -48.0, 24.0);
const CLIMB_PEAK5: [Prop; 2] = peak(-43.0, 75.8, 63.0, 26.0);
const CLIMB_CAIRN5: [Prop; 3] = cairn(29.4, 79.0, 4.9);
const CLIMB_CAIRN8: [Prop; 3] = cairn(46.9, 77.0, 18.2);
const CLIMB_NEST: [Prop; 11] = nest(79.2, 80.0, 18.9);

pub static CLIMB: Dressing = Dressing {
    sky: SKY,
    below: Some(DEEP),
    props: &[
        CLIMB_PEAK0[0],
        CLIMB_PEAK0[1],
        CLIMB_PEAK1[0],
        CLIMB_PEAK1[1],
        CLIMB_PEAK2[0],
        CLIMB_PEAK2[1],
        CLIMB_PEAK3[0],
        CLIMB_PEAK3[1],
        CLIMB_PEAK4[0],
        CLIMB_PEAK4[1],
        CLIMB_PEAK5[0],
        CLIMB_PEAK5[1],
        CLIMB_CAIRN5[0],
        CLIMB_CAIRN5[1],
        CLIMB_CAIRN5[2],
        CLIMB_CAIRN8[0],
        CLIMB_CAIRN8[1],
        CLIMB_CAIRN8[2],
        CLIMB_NEST[0],
        CLIMB_NEST[1],
        CLIMB_NEST[2],
        CLIMB_NEST[3],
        CLIMB_NEST[4],
        CLIMB_NEST[5],
        CLIMB_NEST[6],
        CLIMB_NEST[7],
        CLIMB_NEST[8],
        CLIMB_NEST[9],
        CLIMB_NEST[10],
    ],
};

// The Drift
const DRIFT_PEAK0: [Prop; 2] = peak(-48.0, 81.5, -13.5, 30.0);
const DRIFT_PEAK1: [Prop; 2] = peak(140.0, 91.5, 21.5, 36.0);
const DRIFT_PEAK2: [Prop; 2] = peak(43.5, 77.5, 71.0, 40.0);
const DRIFT_PEAK3: [Prop; 2] = peak(13.5, 95.5, -53.0, 28.0);
const DRIFT_PEAK4: [Prop; 2] = peak(130.0, 101.5, -48.0, 24.0);
const DRIFT_PEAK5: [Prop; 2] = peak(-43.0, 87.5, 56.0, 26.0);
const DRIFT_CAIRN3: [Prop; 3] = cairn(36.0, 88.0, -1.9);
const DRIFT_CAIRN7: [Prop; 3] = cairn(69.6, 84.0, -1.9);
const DRIFT_NEST: [Prop; 11] = nest(85.0, 85.0, 11.0);

pub static DRIFT: Dressing = Dressing {
    sky: SKY,
    below: Some(DEEP),
    props: &[
        DRIFT_PEAK0[0],
        DRIFT_PEAK0[1],
        DRIFT_PEAK1[0],
        DRIFT_PEAK1[1],
        DRIFT_PEAK2[0],
        DRIFT_PEAK2[1],
        DRIFT_PEAK3[0],
        DRIFT_PEAK3[1],
        DRIFT_PEAK4[0],
        DRIFT_PEAK4[1],
        DRIFT_PEAK5[0],
        DRIFT_PEAK5[1],
        DRIFT_CAIRN3[0],
        DRIFT_CAIRN3[1],
        DRIFT_CAIRN3[2],
        DRIFT_CAIRN7[0],
        DRIFT_CAIRN7[1],
        DRIFT_CAIRN7[2],
        DRIFT_NEST[0],
        DRIFT_NEST[1],
        DRIFT_NEST[2],
        DRIFT_NEST[3],
        DRIFT_NEST[4],
        DRIFT_NEST[5],
        DRIFT_NEST[6],
        DRIFT_NEST[7],
        DRIFT_NEST[8],
        DRIFT_NEST[9],
        DRIFT_NEST[10],
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

pub static SPIRE: Dressing = Dressing {
    sky: SKY,
    below: Some(DEEP),
    props: &[
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
const GULF_PEAK0: [Prop; 2] = peak(-48.0, 79.2, -19.5, 30.0);
const GULF_PEAK1: [Prop; 2] = peak(125.0, 89.2, 15.5, 36.0);
const GULF_PEAK2: [Prop; 2] = peak(36.0, 75.2, 60.0, 40.0);
const GULF_PEAK3: [Prop; 2] = peak(6.0, 93.2, -54.0, 28.0);
const GULF_PEAK4: [Prop; 2] = peak(115.0, 99.2, -49.0, 24.0);
const GULF_PEAK5: [Prop; 2] = peak(-43.0, 85.2, 45.0, 26.0);
const GULF_CAIRN3: [Prop; 3] = cairn(32.1, 86.0, -1.9);
const GULF_CAIRN5: [Prop; 3] = cairn(53.6, 84.0, -1.9);
const GULF_NEST: [Prop; 11] = nest(70.2, 89.0, 0.0);

pub static GULF: Dressing = Dressing {
    sky: SKY,
    below: Some(DEEP),
    props: &[
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
        GULF_CAIRN3[0],
        GULF_CAIRN3[1],
        GULF_CAIRN3[2],
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
