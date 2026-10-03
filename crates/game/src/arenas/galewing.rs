//! The Cliffs' dressing: a high plateau at the edge of the world map, under a
//! wide pale sky. Cloud banks lie below the plateau's rim on the three open
//! sides, so the drop reads as a drop; far peaks stand behind the rock face.
//! None of it collides, and none of it stands on the plateau: the plateau,
//! the shelf, the stairs, the tower and its ledges, the four stones and the
//! rock face are all the simulation's (`sim::arena::galewing`), and anything
//! on the plateau tall enough to look like a stone would be read as a lee.

use super::{Dressing, Prop, Shape};

const CLOUD: [f32; 3] = [0.90, 0.92, 0.95];
const PEAK: [f32; 3] = [0.50, 0.53, 0.60];
const SNOW: [f32; 3] = [0.80, 0.82, 0.86];
const TUFT: [f32; 3] = [0.26, 0.36, 0.19];

/// A cloud bank: a wide ball sunk below the rim, so only its top shows past
/// the shelf, a few metres down.
const fn cloud(x: f32, z: f32, d: f32) -> Prop {
    Prop {
        shape: Shape::Sphere,
        at: [x, -d * 0.8, z],
        size: [d, d, d],
        yaw: 0.0,
        rgb: CLOUD,
    }
}

/// A far peak behind the rock face, and its snow cap.
const fn peak(x: f32, z: f32, h: f32) -> [Prop; 2] {
    [
        Prop {
            shape: Shape::Cylinder,
            at: [x, 0.0, z],
            size: [h * 0.9, h, h * 0.9],
            yaw: 0.0,
            rgb: PEAK,
        },
        Prop {
            shape: Shape::Sphere,
            at: [x, h - h * 0.3, z],
            size: [h * 0.6, h * 0.6, h * 0.6],
            yaw: 0.0,
            rgb: SNOW,
        },
    ]
}

/// A low tuft of grass on the plateau: ankle height, never cover.
const fn tuft(x: f32, z: f32) -> Prop {
    Prop {
        shape: Shape::Box,
        at: [x, 12.0, z],
        size: [0.6, 0.12, 0.5],
        yaw: 0.13,
        rgb: TUFT,
    }
}

const P0: [Prop; 2] = peak(-30.0, 70.0, 40.0);
const P1: [Prop; 2] = peak(10.0, 85.0, 55.0);
const P2: [Prop; 2] = peak(45.0, 65.0, 35.0);

pub static DRESSING: Dressing = Dressing {
    // High, thin air: a pale clear blue.
    sky: [0.62, 0.76, 0.92],
    below: None,
    props: &[
        cloud(-46.0, -10.0, 24.0),
        cloud(-44.0, 14.0, 20.0),
        cloud(46.0, -6.0, 24.0),
        cloud(44.0, 18.0, 20.0),
        cloud(-14.0, -46.0, 24.0),
        cloud(16.0, -48.0, 26.0),
        cloud(-42.0, -42.0, 20.0),
        cloud(42.0, -42.0, 20.0),
        P0[0],
        P0[1],
        P1[0],
        P1[1],
        P2[0],
        P2[1],
        tuft(-6.0, -12.0),
        tuft(7.5, -9.0),
        tuft(-15.0, 3.0),
        tuft(12.0, 6.0),
        tuft(4.0, -17.0),
        tuft(-9.0, 14.0),
    ],
};
