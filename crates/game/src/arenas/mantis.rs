//! The Shrine's dressing: a stone court at the end of the world's fifth tier,
//! open to a pale evening sky. Dark cypresses stand outside the wall, a
//! torii-like gate frames the trail in from the west, and a lantern hangs
//! from each column's top. None of it collides: the wall and the four columns
//! are the simulation's solids (`sim::arena::mantis`), and nothing inside the
//! court stands high enough to be mistaken for the cover a column gives.

use super::{Dressing, Prop, Shape};

const CYPRESS: [f32; 3] = [0.14, 0.24, 0.16];
const LACQUER: [f32; 3] = [0.62, 0.16, 0.10];
const LANTERN: [f32; 3] = [0.95, 0.80, 0.45];
const MOSS: [f32; 3] = [0.36, 0.42, 0.30];

const fn cypress(x: f32, z: f32) -> Prop {
    Prop {
        shape: Shape::Cylinder,
        at: [x, 0.0, z],
        size: [0.9, 7.5, 0.9],
        yaw: 0.0,
        rgb: CYPRESS,
    }
}

const fn lantern(x: f32, z: f32) -> Prop {
    Prop {
        shape: Shape::Box,
        at: [x, 8.0, z],
        size: [0.6, 0.6, 0.6],
        yaw: 0.0,
        rgb: LANTERN,
    }
}

const fn moss(x: f32, z: f32, yaw: f32) -> Prop {
    Prop {
        shape: Shape::Box,
        at: [x, 0.0, z],
        size: [2.4, 0.04, 1.2],
        yaw,
        rgb: MOSS,
    }
}

pub static DRESSING: Dressing = Dressing {
    // A pale evening, the light going gold.
    drop: false,
    props: &[
        // The gate on the trail in from the west.
        Prop {
            shape: Shape::Box,
            at: [-19.5, 0.0, -2.6],
            size: [0.5, 5.5, 0.5],
            yaw: 0.0,
            rgb: LACQUER,
        },
        Prop {
            shape: Shape::Box,
            at: [-19.5, 0.0, 2.6],
            size: [0.5, 5.5, 0.5],
            yaw: 0.0,
            rgb: LACQUER,
        },
        Prop {
            shape: Shape::Box,
            at: [-19.5, 5.5, 0.0],
            size: [0.6, 0.5, 7.2],
            yaw: 0.0,
            rgb: LACQUER,
        },
        // A lantern on each column's top.
        lantern(7.07, 7.07),
        lantern(7.07, -7.07),
        lantern(-7.07, 7.07),
        lantern(-7.07, -7.07),
        // Cypresses round the outside of the wall.
        cypress(19.0, 9.0),
        cypress(20.5, -4.0),
        cypress(14.0, 19.0),
        cypress(3.0, 21.0),
        cypress(-9.0, 20.0),
        cypress(-20.0, 11.0),
        cypress(-20.5, -12.0),
        cypress(-8.0, -20.5),
        cypress(6.0, -20.0),
        cypress(17.0, -15.0),
        // Moss in the joints of the court, flat to the floor.
        moss(3.5, -9.5, 0.4),
        moss(-10.0, 3.0, 1.1),
        moss(9.5, 1.5, 2.0),
        moss(-2.5, 10.5, 0.7),
    ],
};
