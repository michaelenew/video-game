//! The opening's dressing (`sim::arena::warden`, `docs/design/opening.md`):
//! the cell's bars, across the east end of the cell where it opens into the
//! cut, torn out in the middle by whatever came through -- the Warden's work,
//! before you have seen it. One lies on the floor. None of it collides; the
//! way out is the gap.

use super::{Dressing, Prop, Shape};

const IRON: [f32; 3] = [0.22, 0.21, 0.22];

/// One bar of the cell's front, `z` along it.
const fn bar(z: f32, h: f32) -> Prop {
    Prop {
        shape: Shape::Cylinder,
        at: [-8.0, 0.0, z],
        size: [0.14, h, 0.14],
        yaw: 0.0,
        rgb: IRON,
    }
}

pub static DRESSING: Dressing = Dressing {
    drop: false,
    props: &[
        bar(-5.6, 4.0),
        bar(-4.8, 4.0),
        bar(-4.0, 4.0),
        bar(-3.2, 4.0),
        bar(-2.4, 1.3),
        bar(2.4, 0.9),
        bar(3.2, 4.0),
        bar(4.0, 4.0),
        bar(4.8, 4.0),
        bar(5.6, 4.0),
        // The crossbar, its middle torn away.
        Prop {
            shape: Shape::Box,
            at: [-8.0, 3.9, -4.2],
            size: [0.16, 0.16, 3.4],
            yaw: 0.0,
            rgb: IRON,
        },
        Prop {
            shape: Shape::Box,
            at: [-8.0, 3.9, 4.2],
            size: [0.16, 0.16, 3.4],
            yaw: 0.0,
            rgb: IRON,
        },
        // A bar on the floor, thrown clear.
        Prop {
            shape: Shape::Box,
            at: [-5.5, 0.0, 1.2],
            size: [0.14, 0.14, 3.6],
            yaw: 0.11,
            rgb: IRON,
        },
    ],
};
