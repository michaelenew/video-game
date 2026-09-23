//! Presentation logic, with no engine dependency.
//!
//! Interpolation, the follow camera and character posing all live here so they can
//! be unit tested without a window, and so the Bevy layer in `game` stays thin
//! enough to read in one sitting.
//!
//! Floating point is fine on this side of the boundary. `sim` is integer-only
//! because determinism demands it; nothing here feeds back into the simulation.

pub mod baked;
pub mod camera;
pub mod clips;
pub mod ik;
pub mod interp;
pub mod math;
pub mod play;
pub mod pose;
pub mod skeleton;
pub mod wings;

pub use camera::{CameraRig, Framing, Surroundings};
pub use clips::Clip;
pub use interp::{Frame, PlayerView, interpolate};
pub use play::{PoseInput, pose_for};
pub use pose::Pose;
pub use skeleton::{Joint, Skeleton};

/// Simulation rate, mirrored from `sim` for convenience.
pub const TICK_HZ: f32 = sim::TICK_HZ as f32;
pub const TICK_SECONDS: f32 = 1.0 / TICK_HZ;

pub(crate) const FX: f32 = 65536.0;

pub(crate) fn fx(v: sim::Fx) -> f32 {
    v.raw() as f32 / FX
}

/// How a fighter's body sits in the world: the rotation from character space
/// into the arena, given which way they face.
///
/// **The one place the two frames meet.** Character space is `+Z` along the
/// facing, `+Y` up, and the left arm at `-X` (see [`pose`]); the arena's frame
/// is the engine's. Everything drawn on a fighter goes through this -- the body
/// parts, whatever their hands are holding -- and so does the test that checks
/// the arm the simulation swings from is the arm the renderer draws. Written
/// once so those two cannot drift apart, which for a class with a different
/// force in each hand is the difference between a hitbox that comes out of the
/// visible fist and one that comes out of the other one.
///
/// `facing` is the simulation's, flattened and unit: `[x, z]`.
pub fn body_turn(facing: [f32; 2]) -> math::Quat {
    math::Quat::from_y(facing[0].atan2(facing[1]))
}

/// Where a point in character space ends up in the arena.
pub fn into_world(local: math::V3, pos: [f32; 3], facing: [f32; 2]) -> math::V3 {
    math::add(pos, body_turn(facing).rotate(local))
}

/// Which joint slot a **drawn** pose keeps one of the body's hands in.
///
/// **A reflection swaps the sides along with the body.** A pose is authored in a
/// left-handed frame -- `+Z` forward, `+Y` up, the left arm at `-X`, which is
/// how a person describes a body -- and `anim::bake::as_drawn` reflects it into
/// the arena's right-handed one. That moves what the author wrote for the right
/// arm into the slot named `HandL`, which is the slot the skeleton hangs at
/// `-X`, which [`body_turn`] puts on the body's right. So the author's right arm
/// is drawn on the body's right, which is the whole point of the reflection.
///
/// What it costs is that `Joint::HandL` is not the body's left hand once a pose
/// has been drawn. Anything pulling a hand out of a solved skeleton wants this
/// rather than the enum's own name -- the shield the Bulwark holds, and every
/// test that asks which arm a move came out of.
pub const fn hand_joint(left: bool) -> skeleton::Joint {
    drawn_joint(if left {
        skeleton::Joint::HandL
    } else {
        skeleton::Joint::HandR
    })
}

/// The slot a drawn pose keeps an authored joint in, for the rest of the body.
///
/// [`hand_joint`] said in general: the reflection swaps every sided joint, so
/// the author's `arm.l` is drawn from the slot named `arm.r`. A joint on the
/// centre line is its own answer.
pub const fn drawn_joint(authored: skeleton::Joint) -> skeleton::Joint {
    authored.opposite()
}

/// Convert a look angle in radians to the simulation's aim unit.
///
/// The simulation counts angles in 1/65536 of a turn as an integer, because an
/// integer angle cannot drift, wraps exactly, and is the same on every machine.
/// The renderer counts in radians because that is what trigonometry and Bevy
/// want. This is the one place the two meet, so the conversion is written once.
pub fn aim_from_radians(yaw: f32) -> u16 {
    let turns = yaw / std::f32::consts::TAU;
    (turns.rem_euclid(1.0) * 65536.0).round() as u32 as u16
}

pub fn radians_from_aim(aim: u16) -> f32 {
    aim as f32 / 65536.0 * std::f32::consts::TAU
}

/// The same conversion for pitch, which is signed rather than wrapped.
///
/// Pitch does not wrap -- it is clamped well short of vertical either way, and
/// an angle that wrapped past straight up would be a camera nobody could use --
/// so it is a signed count of the same 1/65536 turn.
pub fn pitch_from_radians(pitch: f32) -> i16 {
    let turns = pitch / std::f32::consts::TAU;
    (turns * 65536.0).round().clamp(-32768.0, 32767.0) as i16
}

pub fn radians_from_pitch(pitch: i16) -> f32 {
    pitch as f32 / 65536.0 * std::f32::consts::TAU
}
