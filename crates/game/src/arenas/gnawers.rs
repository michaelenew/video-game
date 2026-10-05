//! The Commons' dressing: the dark of the den's mouth, the carcass the pack
//! worries at, and a few tussocks so the open middle reads as a meadow and
//! not a floor. Nothing here collides; the bank, the trunk and the boulders
//! are the simulation's solids.

use super::{Dressing, Prop, Shape};

const BONE: [f32; 3] = [0.84, 0.80, 0.70];
const TUSSOCK: [f32; 3] = [0.30, 0.38, 0.18];

pub static DRESSING: Dressing = Dressing {
    sky: [0.42, 0.52, 0.60],
    below: None,
    props: &[
        // The den: a black throat at the back of the mouth.
        Prop {
            shape: Shape::Box,
            at: [0.0, 0.0, 14.35],
            size: [2.9, 1.95, 0.2],
            yaw: 0.0,
            rgb: [0.02, 0.02, 0.02],
        },
        // The carcass: ribs and a skull.
        Prop {
            shape: Shape::Box,
            at: [-5.2, 0.0, 7.0],
            size: [1.6, 0.35, 0.5],
            yaw: 0.08,
            rgb: [0.42, 0.24, 0.20],
        },
        Prop {
            shape: Shape::Box,
            at: [-4.6, 0.0, 7.3],
            size: [0.12, 0.45, 0.9],
            yaw: 0.08,
            rgb: BONE,
        },
        Prop {
            shape: Shape::Box,
            at: [-5.0, 0.0, 7.2],
            size: [0.12, 0.5, 0.95],
            yaw: 0.08,
            rgb: BONE,
        },
        Prop {
            shape: Shape::Sphere,
            at: [-6.1, 0.0, 6.7],
            size: [0.45, 0.45, 0.45],
            yaw: 0.0,
            rgb: BONE,
        },
        Prop {
            shape: Shape::Sphere,
            at: [-9.0, -0.15, -4.0],
            size: [0.7, 0.5, 0.7],
            yaw: 0.0,
            rgb: TUSSOCK,
        },
        Prop {
            shape: Shape::Sphere,
            at: [4.0, -0.15, -9.0],
            size: [0.8, 0.5, 0.8],
            yaw: 0.0,
            rgb: TUSSOCK,
        },
        Prop {
            shape: Shape::Sphere,
            at: [12.0, -0.15, 6.0],
            size: [0.6, 0.45, 0.6],
            yaw: 0.0,
            rgb: TUSSOCK,
        },
        Prop {
            shape: Shape::Sphere,
            at: [-14.0, -0.15, 9.0],
            size: [0.9, 0.5, 0.9],
            yaw: 0.0,
            rgb: TUSSOCK,
        },
    ],
};
