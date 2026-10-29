//! **The Warden**, drawn as a blockout (`sim::arena::warden`,
//! `docs/design/opening.md`): boxes that nothing collides with, so it can be stood and posed anywhere for a screenshot.
//! It is not a creature and does nothing.
//!
//! `SHOT_WARDEN=x,z,yaw,pose` puts it: (x, z) in metres, `yaw` in degrees
//! from facing `+x`, and `pose` one of `stand`, `slam` (both arms up, about
//! to come down), `swat` (its right arm out sideways at shoulder height,
//! sweeping a perch) and `down` (felled, its head on the floor). Without it,
//! it stands in the middle of the shaft facing the gallery.
//!
//! About two Ridgebacks: the top of its back fourteen metres up, its
//! shoulders twelve, **the soft place on its nape** -- the orange -- at
//! thirteen. Its front faces local `+x`; its right is `+z`.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::{Prop, Shape};

const IRON: [f32; 3] = [0.22, 0.21, 0.22];

/// One bar of the cell's front, `z` along it.
const fn bar(z: f32, h: f32) -> Prop {
    Prop {
        shape: Shape::Cylinder,
        at: [26.5, 0.0, z],
        size: [0.12, h, 0.12],
        yaw: 0.0,
        rgb: IRON,
    }
}

/// **The cell's bars**, across the well's floor under the broken roof: a gap
/// in the middle where two were torn out (one lies on the floor), and the
/// bars either side of it short, snapped.
pub static DRESSING: super::Dressing = super::Dressing {
    drop: false,
    props: &[
        bar(-4.6, 3.0),
        bar(-4.0, 3.0),
        bar(-3.4, 3.0),
        bar(-2.8, 3.0),
        bar(-2.2, 3.0),
        bar(-1.6, 3.0),
        bar(-1.0, 1.1),
        bar(1.0, 0.8),
        bar(1.6, 3.0),
        bar(2.2, 3.0),
        bar(2.8, 3.0),
        bar(3.4, 3.0),
        bar(4.0, 3.0),
        bar(4.6, 3.0),
        Prop {
            shape: Shape::Box,
            at: [25.4, 0.0, 0.6],
            size: [0.12, 0.12, 2.8],
            yaw: 0.08,
            rgb: IRON,
        },
    ],
};

const HIDE: [f32; 3] = [0.42, 0.38, 0.40];
const PLATE: [f32; 3] = [0.72, 0.68, 0.58];
const SOFT: [f32; 3] = [1.0, 0.42, 0.10];

#[derive(Clone, Copy, PartialEq)]
enum Pose {
    Stand,
    Slam,
    Swat,
    Down,
}

/// The Warden's shapes, placed and posed. Boxes, all of it: a blockout of
/// a hunched brute, not a sculpt.
pub fn props() -> Vec<Prop> {
    let (x, z, yaw, pose) = placed();
    let turn = Quat::from_rotation_y(-yaw * TAU);
    let mut out = Vec::new();
    let mut put = |at: [f32; 3], size: [f32; 3], rgb: [f32; 3]| {
        let p = turn * Vec3::from_array(at);
        out.push(Prop {
            shape: Shape::Box,
            at: [x + p.x, p.y, z + p.z],
            size,
            yaw,
            rgb,
        });
    };
    let down = pose == Pose::Down;
    // Felled, everything above the legs is three and a half metres lower.
    let drop = if down { -3.5 } else { 0.0 };
    for side in [-1.0, 1.0] {
        // Legs and feet.
        let leg = if down { 3.0 } else { 6.5 };
        put([-2.8, 0.0, 2.6 * side], [2.6, leg, 2.6], HIDE);
        put([-2.2, 0.0, 2.6 * side], [3.4, 1.0, 3.0], HIDE);
    }
    // Pelvis, the belly, and the great chest, hunched forward over it.
    put([-2.8, 5.0 + drop, 0.0], [4.5, 3.0, 6.5], HIDE);
    put([-1.0, 6.5 + drop, 0.0], [6.0, 5.0, 7.0], HIDE);
    put([0.8, 8.5 + drop, 0.0], [6.5, 5.0, 9.5], HIDE);
    // Plates down its back, and the soft place where they part at the nape.
    put([-0.6, 13.5 + drop, 0.0], [5.5, 0.8, 7.0], PLATE);
    put([-3.4, 11.5 + drop, 0.0], [3.5, 0.8, 5.5], PLATE);
    put([3.2, 13.0 + drop, 0.0], [2.6, 1.6, 3.4], SOFT);
    // The head, jutting forward below the shoulders; on the floor felled.
    let head = if down { 0.0 } else { 9.5 };
    put([5.2, head, 0.0], [3.4, 3.0, 3.2], HIDE);
    put(
        [5.6, head - 0.9 + if down { 0.9 } else { 0.0 }, 0.0],
        [2.8, 1.0, 2.8],
        PLATE,
    );
    for side in [-1.0, 1.0] {
        put([4.6, head + 2.8, 1.2 * side], [0.6, 2.0, 0.6], PLATE);
    }
    for side in [-1.0, 1.0] {
        // Shoulders, each a boulder of its own.
        put([1.2, 10.0 + drop, 5.6 * side], [4.2, 3.8, 3.6], HIDE);
        let right = side > 0.0;
        match pose {
            // Knuckles on the floor: two pillars in front of it.
            Pose::Stand => arm_down(&mut put, side),
            // Both fists overhead.
            Pose::Slam => {
                put([1.6, 12.5, 5.8 * side], [2.8, 7.5, 2.8], HIDE);
                put([1.6, 19.5, 5.8 * side], [3.8, 3.4, 3.8], PLATE);
            }
            // The right arm out sideways at the shoulder; the left braced.
            Pose::Swat if right => {
                put([1.4, 9.4, 11.0], [2.8, 2.8, 10.0], HIDE);
                put([1.4, 9.0, 17.5], [3.8, 3.6, 3.8], PLATE);
            }
            Pose::Swat => arm_down(&mut put, side),
            // Sprawled: arms out on the floor.
            Pose::Down => {
                put([3.0, 0.0, 8.0 * side], [3.0, 2.4, 8.0], HIDE);
                put([3.4, 0.0, 12.5 * side], [3.6, 3.0, 3.6], PLATE);
            }
        }
    }
    out
}

/// One arm straight down to its knuckles, as it stands.
fn arm_down(put: &mut impl FnMut([f32; 3], [f32; 3], [f32; 3]), side: f32) {
    put([2.4, 4.0, 6.2 * side], [2.8, 6.0, 2.8], HIDE);
    put([3.0, 0.0, 6.2 * side], [3.6, 4.0, 3.6], PLATE);
}

/// Where it stands and how: `SHOT_WARDEN`, or the middle of the shaft.
fn placed() -> (f32, f32, f32, Pose) {
    let spec = crate::platform::env("SHOT_WARDEN").unwrap_or_default();
    let mut it = spec.split(',').map(str::trim);
    let mut num = |or: f32| it.next().and_then(|v| v.parse::<f32>().ok()).unwrap_or(or);
    let (x, z, yaw) = (num(0.0), num(0.0), num(0.0) / 360.0);
    let pose = match spec.split(',').nth(3).map(str::trim) {
        Some("slam") => Pose::Slam,
        Some("swat") => Pose::Swat,
        Some("down") => Pose::Down,
        _ => Pose::Stand,
    };
    (x, z, yaw, pose)
}
