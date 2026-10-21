//! The versus stages' dressing (`sim::arena::stages`, `sim::arena::hills`,
//! `docs/design/versus.md`). None of it collides: every island, bench, dyke
//! and shelf is the simulation's.
//!
//! The four floating duel stages hang over a drop (`drop: true`): far peaks
//! in the haze on every side give the eye something to measure the fall
//! by. The three King of the Hill maps stand on the ground inside their
//! rims, and what is added is small: reeds in the Delta, ice in the
//! Glacier, spoil in the Quarry -- low enough that nobody mistakes it for
//! cover.

use super::{Dressing, Prop, Shape};

const PEAK: [f32; 3] = [0.40, 0.36, 0.36];
const EMBER: [f32; 3] = [0.88, 0.42, 0.18];
const SANDSTONE: [f32; 3] = [0.80, 0.66, 0.46];
const ASH_ROCK: [f32; 3] = [0.30, 0.28, 0.28];
const ICE_PEAK: [f32; 3] = [0.74, 0.80, 0.86];
const REED: [f32; 3] = [0.56, 0.58, 0.30];
const ICE: [f32; 3] = [0.78, 0.88, 0.94];
const SPOIL: [f32; 3] = [0.62, 0.58, 0.52];

const fn prop(shape: Shape, at: [f32; 3], size: [f32; 3], yaw: f32, rgb: [f32; 3]) -> Prop {
    Prop {
        shape,
        at,
        size,
        yaw,
        rgb,
    }
}

/// A far peak hanging in the haze: a lump of rock, wider than six metres so
/// it is drawn as one.
const fn peak(x: f32, y: f32, z: f32, d: f32, rgb: [f32; 3]) -> Prop {
    prop(Shape::Sphere, [x, y, z], [d, d * 0.7, d], 0.0, rgb)
}

/// A clump of reeds: three thin stems.
const fn reeds(x: f32, z: f32) -> [Prop; 3] {
    [
        prop(Shape::Cylinder, [x, 0.0, z], [0.12, 1.6, 0.12], 0.0, REED),
        prop(
            Shape::Cylinder,
            [x + 0.3, 0.0, z + 0.2],
            [0.1, 1.3, 0.1],
            0.0,
            REED,
        ),
        prop(
            Shape::Cylinder,
            [x - 0.2, 0.0, z + 0.3],
            [0.1, 1.45, 0.1],
            0.0,
            REED,
        ),
    ]
}

pub static ANVIL: Dressing = Dressing {
    drop: true,
    props: &[
        peak(-60.0, 22.0, 30.0, 14.0, PEAK),
        peak(55.0, 30.0, -40.0, 18.0, PEAK),
        peak(10.0, 18.0, 70.0, 12.0, PEAK),
        peak(-30.0, 34.0, -65.0, 16.0, PEAK),
        // Embers glowing on the far rocks' tops.
        prop(
            Shape::Sphere,
            [-60.0, 31.0, 30.0],
            [2.0, 1.0, 2.0],
            0.0,
            EMBER,
        ),
        prop(
            Shape::Sphere,
            [55.0, 42.0, -40.0],
            [2.5, 1.2, 2.5],
            0.0,
            EMBER,
        ),
    ],
};

pub static ZIGGURAT: Dressing = Dressing {
    drop: true,
    props: &[
        peak(-50.0, 26.0, -45.0, 16.0, SANDSTONE),
        peak(65.0, 32.0, 20.0, 20.0, SANDSTONE),
        peak(-20.0, 20.0, 70.0, 12.0, SANDSTONE),
        peak(30.0, 38.0, -70.0, 14.0, SANDSTONE),
    ],
};

pub static CALDERA: Dressing = Dressing {
    drop: true,
    props: &[
        peak(-55.0, 28.0, 45.0, 18.0, ASH_ROCK),
        peak(60.0, 24.0, -35.0, 14.0, ASH_ROCK),
        peak(-40.0, 36.0, -60.0, 12.0, ASH_ROCK),
        peak(35.0, 30.0, 65.0, 16.0, ASH_ROCK),
        prop(
            Shape::Sphere,
            [60.0, 33.0, -35.0],
            [2.0, 1.0, 2.0],
            0.0,
            EMBER,
        ),
    ],
};

pub static TWIN_ISLES: Dressing = Dressing {
    drop: true,
    props: &[
        peak(-60.0, 30.0, -30.0, 16.0, ICE_PEAK),
        peak(55.0, 24.0, 45.0, 18.0, ICE_PEAK),
        peak(0.0, 36.0, -75.0, 14.0, ICE_PEAK),
        peak(-35.0, 20.0, 65.0, 12.0, ICE_PEAK),
    ],
};

const QUARRY_PROPS: [Prop; 4] = [
    // Spoil heaps in the lanes behind the benches.
    prop(
        Shape::Sphere,
        [-8.0, 0.0, 26.5],
        [3.0, 0.8, 2.0],
        0.0,
        SPOIL,
    ),
    prop(
        Shape::Sphere,
        [8.0, 0.0, -26.5],
        [3.0, 0.8, 2.0],
        0.0,
        SPOIL,
    ),
    prop(
        Shape::Sphere,
        [20.0, 0.0, 26.0],
        [2.0, 0.6, 1.5],
        0.0,
        SPOIL,
    ),
    prop(
        Shape::Sphere,
        [-20.0, 0.0, -26.0],
        [2.0, 0.6, 1.5],
        0.0,
        SPOIL,
    ),
];

pub static QUARRY: Dressing = Dressing {
    drop: false,
    props: &QUARRY_PROPS,
};

const GLACIER_PROPS: [Prop; 4] = [
    // Shards of ice standing in the snow.
    prop(Shape::Box, [-24.0, 0.0, -4.0], [0.4, 1.1, 0.3], 0.07, ICE),
    prop(Shape::Box, [24.0, 0.0, 4.0], [0.4, 1.1, 0.3], 0.57, ICE),
    prop(Shape::Box, [-12.0, 0.0, -20.0], [0.3, 0.8, 0.3], 0.21, ICE),
    prop(Shape::Box, [12.0, 0.0, 20.0], [0.3, 0.8, 0.3], 0.71, ICE),
];

pub static GLACIER: Dressing = Dressing {
    drop: false,
    props: &GLACIER_PROPS,
};

const DELTA_REEDS: [[Prop; 3]; 6] = [
    reeds(-28.0, 9.0),
    reeds(28.0, -9.0),
    reeds(-23.0, 4.5),
    reeds(23.0, -4.5),
    reeds(-8.0, -23.0),
    reeds(8.0, 23.0),
];

const DELTA_PROPS: [Prop; 18] = {
    let mut out = [DELTA_REEDS[0][0]; 18];
    let mut i = 0;
    while i < 18 {
        out[i] = DELTA_REEDS[i / 3][i % 3];
        i += 1;
    }
    out
};

pub static DELTA: Dressing = Dressing {
    drop: false,
    props: &DELTA_PROPS,
};
