//! The Den's dressing: a thorn scrub at dusk -- flat-topped trees past the
//! wall, rocks, and dry grass in a ring just inside it, low enough to read as
//! floor. None of it collides: the wall, the two platforms and the three
//! standing stones are the simulation's solids (`sim::arena::pair`), and
//! nothing inside the wall stands high enough to be mistaken for the cover a
//! stone gives.

use super::{Dressing, Prop, Shape};

const BARK: [f32; 3] = [0.30, 0.24, 0.18];
const THORN: [f32; 3] = [0.36, 0.42, 0.22];
const DRY: [f32; 3] = [0.70, 0.62, 0.38];
const ROCK: [f32; 3] = [0.48, 0.44, 0.40];

pub static DRESSING: Dressing = Dressing {
    // Late afternoon over the scrub.
    sky: [0.78, 0.62, 0.42],
    below: None,
    props: &[
        Prop {
            shape: Shape::Cylinder,
            at: [-19.00, 0.00, -8.00],
            size: [0.35, 3.20, 0.35],
            yaw: 0.00,
            rgb: BARK,
        },
        Prop {
            shape: Shape::Box,
            at: [-19.00, 3.20, -8.00],
            size: [4.20, 0.50, 3.40],
            yaw: 0.30,
            rgb: THORN,
        },
        Prop {
            shape: Shape::Cylinder,
            at: [18.00, 0.00, 10.00],
            size: [0.35, 3.20, 0.35],
            yaw: 0.00,
            rgb: BARK,
        },
        Prop {
            shape: Shape::Box,
            at: [18.00, 3.20, 10.00],
            size: [4.20, 0.50, 3.40],
            yaw: 0.30,
            rgb: THORN,
        },
        Prop {
            shape: Shape::Cylinder,
            at: [-17.00, 0.00, 14.00],
            size: [0.35, 3.20, 0.35],
            yaw: 0.00,
            rgb: BARK,
        },
        Prop {
            shape: Shape::Box,
            at: [-17.00, 3.20, 14.00],
            size: [4.20, 0.50, 3.40],
            yaw: 0.30,
            rgb: THORN,
        },
        Prop {
            shape: Shape::Cylinder,
            at: [20.00, 0.00, -15.00],
            size: [0.35, 3.20, 0.35],
            yaw: 0.00,
            rgb: BARK,
        },
        Prop {
            shape: Shape::Box,
            at: [20.00, 3.20, -15.00],
            size: [4.20, 0.50, 3.40],
            yaw: 0.30,
            rgb: THORN,
        },
        Prop {
            shape: Shape::Cylinder,
            at: [4.00, 0.00, -20.00],
            size: [0.35, 3.20, 0.35],
            yaw: 0.00,
            rgb: BARK,
        },
        Prop {
            shape: Shape::Box,
            at: [4.00, 3.20, -20.00],
            size: [4.20, 0.50, 3.40],
            yaw: 0.30,
            rgb: THORN,
        },
        Prop {
            shape: Shape::Cylinder,
            at: [-6.00, 0.00, 19.00],
            size: [0.35, 3.20, 0.35],
            yaw: 0.00,
            rgb: BARK,
        },
        Prop {
            shape: Shape::Box,
            at: [-6.00, 3.20, 19.00],
            size: [4.20, 0.50, 3.40],
            yaw: 0.30,
            rgb: THORN,
        },
        Prop {
            shape: Shape::Box,
            at: [12.60, 0.00, 0.00],
            size: [0.80, 0.25, 0.50],
            yaw: 0.00,
            rgb: DRY,
        },
        Prop {
            shape: Shape::Box,
            at: [11.35, 0.00, 5.47],
            size: [0.80, 0.25, 0.50],
            yaw: 0.45,
            rgb: DRY,
        },
        Prop {
            shape: Shape::Box,
            at: [7.86, 0.00, 9.85],
            size: [0.80, 0.25, 0.50],
            yaw: 0.90,
            rgb: DRY,
        },
        Prop {
            shape: Shape::Box,
            at: [2.80, 0.00, 12.28],
            size: [0.80, 0.25, 0.50],
            yaw: 1.35,
            rgb: DRY,
        },
        Prop {
            shape: Shape::Box,
            at: [-2.80, 0.00, 12.28],
            size: [0.80, 0.25, 0.50],
            yaw: 1.80,
            rgb: DRY,
        },
        Prop {
            shape: Shape::Box,
            at: [-7.86, 0.00, 9.85],
            size: [0.80, 0.25, 0.50],
            yaw: 2.24,
            rgb: DRY,
        },
        Prop {
            shape: Shape::Box,
            at: [-11.35, 0.00, 5.47],
            size: [0.80, 0.25, 0.50],
            yaw: 2.69,
            rgb: DRY,
        },
        Prop {
            shape: Shape::Box,
            at: [-12.60, 0.00, 0.00],
            size: [0.80, 0.25, 0.50],
            yaw: 3.1,
            rgb: DRY,
        },
        Prop {
            shape: Shape::Box,
            at: [-11.35, 0.00, -5.47],
            size: [0.80, 0.25, 0.50],
            yaw: 3.59,
            rgb: DRY,
        },
        Prop {
            shape: Shape::Box,
            at: [-7.86, 0.00, -9.85],
            size: [0.80, 0.25, 0.50],
            yaw: 4.04,
            rgb: DRY,
        },
        Prop {
            shape: Shape::Box,
            at: [-2.80, 0.00, -12.28],
            size: [0.80, 0.25, 0.50],
            yaw: 4.49,
            rgb: DRY,
        },
        Prop {
            shape: Shape::Box,
            at: [2.80, 0.00, -12.28],
            size: [0.80, 0.25, 0.50],
            yaw: 4.94,
            rgb: DRY,
        },
        Prop {
            shape: Shape::Box,
            at: [7.86, 0.00, -9.85],
            size: [0.80, 0.25, 0.50],
            yaw: 5.39,
            rgb: DRY,
        },
        Prop {
            shape: Shape::Box,
            at: [11.35, 0.00, -5.47],
            size: [0.80, 0.25, 0.50],
            yaw: 5.83,
            rgb: DRY,
        },
        Prop {
            shape: Shape::Sphere,
            at: [-21.00, 0.00, 3.00],
            size: [2.40, 1.40, 2.00],
            yaw: 0.00,
            rgb: ROCK,
        },
        Prop {
            shape: Shape::Sphere,
            at: [22.00, 0.00, -4.00],
            size: [2.40, 1.40, 2.00],
            yaw: 0.00,
            rgb: ROCK,
        },
        Prop {
            shape: Shape::Sphere,
            at: [10.00, 0.00, 21.00],
            size: [2.40, 1.40, 2.00],
            yaw: 0.00,
            rgb: ROCK,
        },
        Prop {
            shape: Shape::Sphere,
            at: [-12.00, 0.00, -21.00],
            size: [2.40, 1.40, 2.00],
            yaw: 0.00,
            rgb: ROCK,
        },
    ],
};
