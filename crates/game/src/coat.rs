//! **Her coat tails, on springs** (`docs/design/exploration/0024_a_character_done_properly.md`).
//!
//! The four long panels of the Reaver's coat each hang from two joints of
//! their own (`tail.k.a` at the belt, `tail.k.b` halfway down), children of
//! her hips in the model (`art/reaver/outfit.py`, `coat_tails`). Nothing in
//! the simulation knows about them: this swings them, every drawn frame,
//! from what the body is doing.
//!
//! - **They lag.** Each panel is a damped pendulum driven by how fast the
//!   belt it hangs from is speeding up: she starts a dash and the front
//!   panels fall back against her legs, the back ones fly out; she stops and
//!   they swing on past her.
//! - **The legs push them.** A knee or an ankle further out along a panel's
//!   direction than the panel hangs is a floor under its swing, so a running
//!   stride lifts the cloth instead of passing through it.
//! - **The lower half whips.** The second joint swings again on top of the
//!   first, by how fast the first is turning.
//!
//! This is the one part of her that moves on the renderer's own clock, which
//! `docs/design/animation.md` allows for cosmetic smoothing: a rollback can
//! only nudge where the springs are being pulled, never make them pop, and
//! nothing that reads as gameplay is here.

use bevy::prelude::*;
use sim::state::MAX_PLAYERS;
use view::skeleton::{Joint, Skin};

/// How many panels the coat has, and its joints: two a panel.
pub const PANELS: usize = 4;
pub const TAIL_JOINTS: usize = PANELS * 2;

/// The coat's joints by name, in the model's order (`garment.TAILS`).
pub const TAIL_NAMES: [&str; TAIL_JOINTS] = [
    "tail.0.a", "tail.0.b", "tail.1.a", "tail.1.b", "tail.2.a", "tail.2.b", "tail.3.a", "tail.3.b",
];

/// How stiff the swing is (per second squared, per radian), and how damped.
/// About a third of a second a swing, settled in two.
const STIFF: f32 = 70.0;
const DAMP: f32 = 7.5;
/// The lower half: stiffer, and kicked by how fast the upper half turns.
const STIFF_LOW: f32 = 120.0;
const DAMP_LOW: f32 = 10.0;
const WHIP: f32 = 0.05;
/// How far a panel's swing answers the belt's acceleration: a pendulum of
/// this length, in metres.
const LENGTH: f32 = 0.45;
/// How thick a leg is at the knee and at the ankle, for the push.
const KNEE: f32 = 0.085;
const ANKLE: f32 = 0.07;
/// How far either side of a panel's middle a leg can be and still push it.
const HALF_WIDTH: f32 = 0.2;
/// The furthest a panel swings either way, in radians.
const MOST: f32 = 1.3;

/// One panel's swing: out from her (positive) or in, and to the side; and
/// its lower half's.
#[derive(Clone, Copy, Default, Debug)]
pub struct Swing {
    out: f32,
    out_v: f32,
    side: f32,
    side_v: f32,
    low: f32,
    low_v: f32,
}

/// Where a panel's belt joint was, in the arena, and how fast it was going.
#[derive(Clone, Copy, Default, Debug)]
struct Motion {
    at: Option<Vec3>,
    velocity: Vec3,
}

/// Every seat's coat.
#[derive(Resource, Default)]
pub struct Coats {
    swing: [[Swing; PANELS]; MAX_PLAYERS],
    motion: [[Motion; PANELS]; MAX_PLAYERS],
}

/// A coat joint as the model was bound: its entity and its place under its
/// parent, which the swing turns.
#[derive(Clone, Copy, Debug)]
pub struct TailJoint {
    pub entity: Entity,
    pub bind: Transform,
}

/// One seat's coat joints, in [`TAIL_NAMES`] order, and the hips' rotation
/// in the bind pose, which the belt joints' places are written under.
#[derive(Clone, Copy, Debug)]
pub struct Tails {
    pub joints: [TailJoint; TAIL_JOINTS],
    pub hips_bind: Quat,
}

/// Swing one seat's coat for this frame. `skin` is the body's solve, in
/// character space; `body` the fighter's place in the arena.
pub fn swing(
    coats: &mut Coats,
    owner: usize,
    tails: &Tails,
    skin: &Skin,
    body: &Transform,
    dt: f32,
    joints: &mut Query<&mut Transform, Without<crate::Fighter>>,
) {
    let dt = dt.clamp(1.0 / 240.0, 1.0 / 30.0);
    let v3 = |v: view::math::V3| Vec3::new(v[0], v[1], v[2]);
    let hips_at = v3(skin.origin[Joint::Root.index()]);
    let r = skin.rot[Joint::Root.index()].0;
    let hips = Quat::from_xyzw(r[0], r[1], r[2], r[3]);
    // How the hips have turned since the bind pose: the coat follows them.
    let follow = hips * tails.hips_bind.inverse();
    let legs = [(Joint::ShinL, Joint::FootL), (Joint::ShinR, Joint::FootR)];
    for k in 0..PANELS {
        let a = tails.joints[2 * k];
        let b = tails.joints[2 * k + 1];
        // Where the panel hangs from, which way it faces out and which way is
        // across it, in character space.
        let pivot = hips_at + hips * a.bind.translation;
        let out = Vec3::new(pivot.x - hips_at.x, 0.0, pivot.z - hips_at.z).normalize_or(Vec3::Z);
        let across = Vec3::Y.cross(out);
        let hinge = Vec3::NEG_Y.cross(out).normalize_or(Vec3::X);
        // The bind pose's lean out, from where the lower joint sits under
        // the upper (both unturned in the bind pose, so this is in
        // character space as it was bound).
        let low_at = b.bind.translation;
        let flare = low_at.with_y(0.0).length().atan2(-low_at.y);

        // The belt's acceleration, in character space.
        let motion = &mut coats.motion[owner][k];
        let world = body.transform_point(pivot);
        let velocity = motion.at.map_or(Vec3::ZERO, |at| (world - at) / dt);
        // A jump of a metre a frame is the world moving into another
        // place's coordinates, or a teleport: start again from rest.
        let accel = if motion.at.is_some_and(|at| at.distance(world) < 1.0) {
            (velocity - motion.velocity) / dt
        } else {
            Vec3::ZERO
        };
        *motion = Motion {
            at: Some(world),
            velocity: if motion.at.is_some() {
                velocity
            } else {
                Vec3::ZERO
            },
        };
        let accel = body.rotation.inverse() * accel;

        // How far out the legs make it hang, at its upper and lower half.
        let mut floor = -MOST;
        let mut floor_low = -MOST;
        for (knee, ankle) in legs {
            for (joint, thick, depth_of) in [(knee, KNEE, 0.0), (ankle, ANKLE, 1.0)] {
                let q = v3(skin.origin[joint.index()]);
                let d = q - pivot;
                if d.dot(across).abs() > HALF_WIDTH {
                    continue;
                }
                let depth = -d.y;
                if depth < 0.1 {
                    continue;
                }
                let need = (d.dot(out) + thick).atan2(depth) - flare;
                if depth_of == 0.0 {
                    floor = floor.max(need);
                } else {
                    floor_low = floor_low.max(need);
                }
            }
        }

        let s = &mut coats.swing[owner][k];
        let drive_out = -accel.dot(out) / LENGTH;
        let drive_side = -accel.dot(across) / LENGTH;
        s.out_v += (STIFF * (floor.max(0.0) - s.out) - DAMP * s.out_v + drive_out) * dt;
        s.out += s.out_v * dt;
        if s.out < floor {
            s.out = floor;
            s.out_v = s.out_v.max(0.0);
        }
        s.out = s.out.clamp(-MOST, MOST);
        s.side_v += (-STIFF * s.side - DAMP * s.side_v + drive_side) * dt;
        s.side += s.side_v * dt;
        s.side = s.side.clamp(-MOST * 0.5, MOST * 0.5);
        // The lower half trails how fast the upper turns, and is held off
        // its own legs.
        let low_floor = (floor_low - s.out).max(-MOST);
        let trail = -s.out_v * WHIP;
        s.low_v += (STIFF_LOW * (trail.max(low_floor) - s.low) - DAMP_LOW * s.low_v) * dt;
        s.low += s.low_v * dt;
        if s.low < low_floor {
            s.low = low_floor;
            s.low_v = s.low_v.max(0.0);
        }
        s.low = s.low.clamp(-MOST, MOST);

        // The belt joint: the hips' turn, then the swing, written under the
        // hips (it is their child in the model).
        let turn = Quat::from_axis_angle(hinge, s.out) * Quat::from_axis_angle(out, s.side);
        let own = turn * follow;
        if let Ok(mut tf) = joints.get_mut(a.entity) {
            tf.translation = a.bind.translation;
            tf.rotation = hips.inverse() * own;
        }
        // The lower joint: under the belt joint, turned again about the same
        // hinge, seen from the belt joint's own frame.
        let hinge_local = (own.inverse() * hinge).normalize_or(Vec3::X);
        if let Ok(mut tf) = joints.get_mut(b.entity) {
            tf.translation = b.bind.translation;
            tf.rotation = Quat::from_axis_angle(hinge_local, s.low) * b.bind.rotation;
        }
    }
}
