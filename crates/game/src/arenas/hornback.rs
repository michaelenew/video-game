//! The low meadow's dressing, and the crossing's: the river past the ford
//! where the herd leaves, tussocks so the open grass reads as a meadow and
//! not a floor, a few grazed-flat patches. Nothing here collides: the bank,
//! the thicket and the trunk are the simulation's solids, and the boulders
//! are hazard cells it raises (they crack and break, so they are not props).

use super::{Dressing, Prop, Shape};

const RIVER: [f32; 3] = [0.12, 0.26, 0.38];
const TUSSOCK: [f32; 3] = [0.30, 0.40, 0.18];
const FLOWER: [f32; 3] = [0.78, 0.72, 0.42];
const REED: [f32; 3] = [0.42, 0.46, 0.22];

const fn tussock(x: f32, z: f32, size: f32) -> Prop {
    Prop {
        shape: Shape::Sphere,
        at: [x, -0.15, z],
        size: [size, 0.45, size],
        yaw: 0.0,
        rgb: TUSSOCK,
    }
}

pub static DRESSING: Dressing = Dressing {
    sky: [0.52, 0.64, 0.74],
    props: &[
        // The river beyond the ford, east.
        Prop {
            shape: Shape::Box,
            at: [27.5, -0.05, 0.0],
            size: [4.0, 0.05, 44.0],
            yaw: 0.0,
            rgb: RIVER,
        },
        // Reeds either side of the ford.
        Prop {
            shape: Shape::Cylinder,
            at: [22.0, 0.0, -5.0],
            size: [0.5, 1.2, 0.5],
            yaw: 0.0,
            rgb: REED,
        },
        Prop {
            shape: Shape::Cylinder,
            at: [22.5, 0.0, -15.0],
            size: [0.6, 1.0, 0.6],
            yaw: 0.0,
            rgb: REED,
        },
        tussock(-14.0, -12.0, 0.8),
        tussock(-4.0, 3.0, 0.6),
        tussock(6.0, 13.0, 0.9),
        tussock(17.0, 9.0, 0.7),
        tussock(-18.0, 6.0, 0.6),
        tussock(2.0, -16.0, 0.8),
        Prop {
            shape: Shape::Sphere,
            at: [-7.0, -0.1, -4.0],
            size: [0.35, 0.25, 0.35],
            yaw: 0.0,
            rgb: FLOWER,
        },
        Prop {
            shape: Shape::Sphere,
            at: [12.0, -0.1, -3.0],
            size: [0.3, 0.25, 0.3],
            yaw: 0.0,
            rgb: FLOWER,
        },
    ],
};

/// The crossing: the road west to east, the herd's ground north of it.
pub static CROSSING: Dressing = Dressing {
    sky: [0.55, 0.64, 0.70],
    props: &[
        // Ruts along the road.
        Prop {
            shape: Shape::Box,
            at: [0.0, -0.04, -0.6],
            size: [78.0, 0.05, 0.25],
            yaw: 0.0,
            rgb: [0.24, 0.20, 0.15],
        },
        Prop {
            shape: Shape::Box,
            at: [0.0, -0.04, 0.6],
            size: [78.0, 0.05, 0.25],
            yaw: 0.0,
            rgb: [0.24, 0.20, 0.15],
        },
        tussock(-20.0, 9.0, 0.8),
        tussock(-6.0, -9.0, 0.7),
        tussock(14.0, 11.0, 0.9),
        tussock(28.0, -8.0, 0.6),
    ],
};
