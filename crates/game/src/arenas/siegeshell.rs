//! The Last Valley's dressing: a long cut between cliffs at the end of the
//! world's road, under a heavy grey-gold sky, ending in a walled town. The
//! town's roofs show over the wall and towers stand at its ends; scree lies at
//! the cliffs' feet. None of it collides: the cliffs, the boulders, the
//! standing stones and the wall are the simulation's (`sim::arena::siegeshell`
//! and the wall objective), and nothing here stands in the valley where a
//! fighter could mistake it for cover.

use super::{Dressing, Prop, Shape};

const ROOF: [f32; 3] = [0.42, 0.20, 0.14];
const TOWER: [f32; 3] = [0.40, 0.40, 0.42];
const SCREE: [f32; 3] = [0.30, 0.28, 0.26];
const BANNER: [f32; 3] = [0.75, 0.62, 0.18];

const fn roof(x: f32, z: f32, w: f32) -> Prop {
    Prop {
        shape: Shape::Box,
        at: [x, 6.0, z],
        size: [5.0, 6.0, w],
        yaw: 0.0,
        rgb: ROOF,
    }
}

const fn scree(x: f32, z: f32, yaw: f32) -> Prop {
    Prop {
        shape: Shape::Sphere,
        at: [x, -1.2, z],
        size: [4.0, 2.6, 4.0],
        yaw,
        rgb: SCREE,
    }
}

pub static DRESSING: Dressing = Dressing {
    // A heavy sky, the light low and grey-gold.
    sky: [0.62, 0.58, 0.50],
    below: None,
    props: &[
        // The town over the wall: roofs, and a tower at each end of it.
        roof(147.5, -18.0, 8.0),
        roof(148.0, -6.0, 10.0),
        roof(147.0, 6.0, 9.0),
        roof(148.5, 18.0, 8.0),
        Prop {
            shape: Shape::Cylinder,
            at: [145.0, 0.0, -24.0],
            size: [4.0, 14.0, 4.0],
            yaw: 0.0,
            rgb: TOWER,
        },
        Prop {
            shape: Shape::Cylinder,
            at: [145.0, 0.0, 24.0],
            size: [4.0, 14.0, 4.0],
            yaw: 0.0,
            rgb: TOWER,
        },
        // Banners on the wall's towers.
        Prop {
            shape: Shape::Box,
            at: [143.0, 9.0, -24.0],
            size: [0.1, 4.0, 1.6],
            yaw: 0.0,
            rgb: BANNER,
        },
        Prop {
            shape: Shape::Box,
            at: [143.0, 9.0, 24.0],
            size: [0.1, 4.0, 1.6],
            yaw: 0.0,
            rgb: BANNER,
        },
        // Scree at the cliffs' feet.
        scree(-110.0, -24.5, 0.1),
        scree(-70.0, 24.5, 0.3),
        scree(-25.0, -24.5, 0.6),
        scree(20.0, 24.5, 0.2),
        scree(65.0, -24.5, 0.4),
        scree(105.0, 24.5, 0.7),
    ],
};
