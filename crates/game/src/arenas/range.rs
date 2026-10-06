//! The range's dressing: a few landmarks, so a long valley reads as one when
//! you are standing in it. Bones half sunk in the sand, a stump by the
//! stream, a cairn at each end.

use super::{Dressing, Prop, Shape};

const BONE: [f32; 3] = [0.86, 0.83, 0.74];
const CAIRN: [f32; 3] = [0.45, 0.43, 0.40];

pub static DRESSING: Dressing = Dressing {
    drop: false,
    props: &[
        Prop {
            shape: Shape::Box,
            at: [-64.0, 0.0, 6.0],
            size: [3.2, 0.25, 0.3],
            yaw: 0.1,
            rgb: BONE,
        },
        Prop {
            shape: Shape::Box,
            at: [-52.0, 0.0, -10.0],
            size: [2.4, 0.2, 0.25],
            yaw: 0.35,
            rgb: BONE,
        },
        Prop {
            shape: Shape::Sphere,
            at: [-58.0, -0.3, -2.0],
            size: [1.0, 1.0, 1.0],
            yaw: 0.0,
            rgb: BONE,
        },
        Prop {
            shape: Shape::Cylinder,
            at: [-23.0, 0.0, 12.0],
            size: [0.9, 0.6, 0.9],
            yaw: 0.0,
            rgb: [0.33, 0.24, 0.16],
        },
        Prop {
            shape: Shape::Box,
            at: [-114.0, 0.0, 0.0],
            size: [1.2, 1.6, 1.2],
            yaw: 0.125,
            rgb: CAIRN,
        },
        Prop {
            shape: Shape::Box,
            at: [114.0, 0.0, 0.0],
            size: [1.2, 1.6, 1.2],
            yaw: 0.125,
            rgb: CAIRN,
        },
    ],
};
