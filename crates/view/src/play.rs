//! Choosing which animation to play, and blending between them.
//!
//! Everything here is a pure function of the simulation snapshot. That is not
//! a style preference: rollback re-simulates past frames, so a clip that
//! advances on its own clock plays a different frame the second time through
//! and the character visibly jumps. Blending is allowed -- a blend of two pure
//! functions is still one -- but nothing may *accumulate*.
//!
//! ```text
//! pose = f(action, frames into it, stride phase, airtime, turn rate, health)
//! ```
//!
//! There is one exception, and it is deliberate: `Crossfade`, at the bottom of
//! this file, is renderer-local state. A fade that hiccups across a rollback's
//! one to eight frames is imperceptible, and the cut it removes is not.
//!
//! ## Three tricks worth knowing about
//!
//! **Locomotion is driven by ground covered, not by time.** A walk cycle on a
//! fixed cadence slides its feet the moment the body moves at any other speed.
//! Ours indexes the cycle by a stride phase the simulation carries, so a
//! footfall happens every stride's worth of metres at any speed, and the
//! directional clips are sampled at the *same phase* before being blended --
//! which is what keeps a diagonal from producing two feet on the ground at
//! once. The phase is an accumulator rather than `distance / stride`, because a
//! stride is longer at a sprint than at a walk and dividing by a changing
//! number makes the phase jump by whole cycles.
//!
//! **Stun clips are indexed from the end.** `HitStun { left }` counts down, and
//! what matters is that the character is back on their feet on the exact frame
//! control returns. So the clip is indexed backwards from its last frame: a
//! stun longer than the clip holds the impact pose at the start, where holding
//! looks like being hurt, rather than standing around neutral at the end, where
//! it looks like the game forgot to give the controls back.
//!
//! **Turn clips are indexed by turn rate, not by time.** Facing follows the
//! mouse, so there is no "turn" event to start a clip on -- only a body that is
//! currently rotating quickly or slowly. Frame zero of a turn clip is neutral
//! and the last frame is a full committed turn; the renderer picks the frame
//! from how fast the body is actually rotating, and layers the difference over
//! whatever the legs are doing.

use crate::clips::Clip;
use crate::interp::PlayerView;
use crate::pose::Pose;
use crate::skeleton::{Group, Joint};
use sim::Class;
use sim::state::Action;

/// The stride numbers, in the renderer's units.
///
/// They live in `sim::tuning` because the **simulation** advances the stride
/// phase with them -- see `Player::stride` for why that cannot be worked out
/// here -- and the clips are authored against the same values. One copy, three
/// users, no drift.
const fn metres(v: sim::Fx) -> f32 {
    v.raw() as f32 / 65536.0
}

pub const WALK_AT: f32 = metres(sim::tuning::WALK_AT);
pub const RUN_AT: f32 = metres(sim::tuning::RUN_AT);
pub const WALK_STRIDE: f32 = metres(sim::tuning::WALK_STRIDE);
pub const RUN_STRIDE: f32 = metres(sim::tuning::RUN_STRIDE);
pub const CROUCH_STRIDE: f32 = metres(sim::tuning::CROUCH_STRIDE);
pub const STRAFE_STRIDE: f32 = metres(sim::tuning::STRAFE_STRIDE);

/// Turn rates, in turns per second, at which the two turn clips reach full
/// weight.
const TURN_SLOW: f32 = 0.22;
const TURN_FAST: f32 = 1.10;

/// A flight shorter than this lands without a landing animation. Hopping off a
/// kerb should not make the character crumple.
const LANDING_WORTH_PLAYING: u16 = 7;
/// And above this, the landing is a heavy one.
const HARD_FALL: u16 = 26;

/// Everything posing is allowed to depend on. All of it comes out of the
/// snapshot, or is derived from two consecutive snapshots.
#[derive(Clone, Copy, Debug)]
pub struct PoseInput {
    pub class: Class,
    pub action: Action,
    pub grounded: bool,
    pub crouching: bool,
    pub speed: f32,
    /// Strafe and forward velocity in the character's own frame.
    pub travel: [f32; 2],
    /// The same two, eased.
    ///
    /// Two versions on purpose. The simulation goes from a standstill to a
    /// sprint in three frames and swaps travel direction in one, which no blend
    /// between walk cycles can follow -- so the *blend weights* read the eased
    /// pair. But a **choice** must read the real one: which way a dodge went is
    /// decided on its first frame and cannot be revised, and picking it from a
    /// value that is still catching up switches clips two frames into the roll.
    pub eased_speed: f32,
    pub eased_travel: [f32; 2],
    /// Where the body is in its stride, in cycles, wrapping.
    pub stride: f32,
    pub air_frames: u16,
    pub since_landed: u16,
    pub parried: u16,
    pub crouched_for: u16,
    pub stun_total: u16,
    pub rise: f32,
    /// Turns per second, positive to the character's right.
    pub turn_rate: f32,
    /// Radians above the horizon the current move was aimed along. Only the
    /// moves that actually leave along the crosshair use it.
    pub aim_pitch: f32,
    pub health: i32,
    /// Frames left of the between-rounds pause, if a round has ended.
    pub round_left: Option<u16>,
    /// Simulation frame. In the snapshot, so cyclic motion driven from it is
    /// deterministic -- and unlike the saturating counters it never stops
    /// advancing, which is what an idle that loops for twenty minutes needs.
    pub sim_frame: u32,
    /// Show the skeleton at rest instead of animating it. The toggle exists so
    /// that "is this the clip or is this the rig?" can be answered in one
    /// keypress while looking at the thing.
    pub bind_pose: bool,
}

impl PoseInput {
    pub fn of(view: &PlayerView, class: Class, frame: &crate::Frame) -> PoseInput {
        PoseInput {
            class,
            action: view.action,
            grounded: view.grounded,
            crouching: view.crouching,
            speed: view.speed,
            travel: view.travel,
            eased_speed: view.speed,
            eased_travel: view.travel,
            stride: view.stride,
            air_frames: view.air_frames,
            since_landed: view.since_landed,
            parried: view.parried,
            crouched_for: view.crouched_for,
            stun_total: view.stun_total,
            rise: view.rise,
            turn_rate: view.turn_rate,
            aim_pitch: view.aim_pitch,
            health: view.health,
            round_left: frame.round_left,
            sim_frame: frame.sim_frame,
            bind_pose: false,
        }
    }
}

/// What produced a pose, as a number.
///
/// Only ever compared for equality. The renderer uses it to notice that the
/// character has switched to a different animation and cross-fade rather than
/// cut -- which is the one piece of animation state that is allowed to live
/// outside the snapshot, because a fade that hiccups across a rollback's one to
/// eight frames is imperceptible and a cut every time you start walking is not.
pub type Shape = u32;

/// The whole animation system, and what produced it.
pub fn pose_and_shape(input: PoseInput) -> (Pose, Shape) {
    let pose = pose_for(input);
    (pose, shape_of(input))
}

/// Which branch of the selection this state lands in.
///
/// Deliberately coarse: two frames of the same walk are the same shape, and a
/// walk and a run are too, because the blend between them is continuous and
/// fading it against itself would only soften it. What has to be caught is a
/// *discontinuity* -- walking into an attack, an attack into a hit.
fn shape_of(input: PoseInput) -> Shape {
    if input.health <= 0 {
        return 1;
    }
    match input.action {
        Action::Startup { kind, .. }
        | Action::Active { kind, .. }
        | Action::Recovery { kind, .. } => {
            // The three phases of one move are one shape: they are consecutive
            // frames of a single clip, and fading between them would blur the
            // contact frame, which is the one frame that must be sharp.
            100 + kind as u32
        }
        Action::Guard { .. } => 200,
        Action::BlockStun { .. } => 201,
        Action::HitStun { .. } => 202,
        Action::Stagger { .. } => 203,
        Action::Held { .. } => 204,
        Action::Dodge { .. } => 205,
        Action::Free => {
            if !input.grounded {
                206
            } else if input.crouching {
                207
            } else if input.speed < 0.5 {
                // Standing and moving are different animations, and the
                // simulation goes from one to the other in a single frame -- it
                // has no ground friction to speak of, on purpose. Without the
                // distinction here, stopping is a cut from a full sprint to an
                // idle with nothing in between.
                208
            } else {
                209
            }
        }
    }
}

/// Renderer-local cross-fade between animations.
///
/// Held outside the snapshot on purpose. A rollback rewinds the fade to
/// whatever it was, which is wrong by up to eight frames of blend weight and
/// invisible; cutting straight from a walk into a wind-up is visible every
/// single time.
#[derive(Clone, Copy, Debug)]
pub struct Crossfade {
    /// What was on screen when the animation last changed.
    from: Pose,
    /// What was on screen last frame, which is what the next fade starts from.
    drawn: Pose,
    shape: Shape,
    /// Frames of fade left.
    left: f32,
    /// Frames the fade runs for.
    length: f32,
    /// How fast the body is going, smoothed.
    ///
    /// Only the blend weights read this -- the stride phase comes from the
    /// simulation, so smoothing the speed cannot make the legs run on the spot.
    /// It exists because the simulation accelerates from a standstill to a
    /// sprint in three frames, and blending idle to walk to run that fast is a
    /// pop however good the clips are.
    speed: f32,
    /// Which way the body is going, smoothed.
    ///
    /// The four directional walk clips are blended by this, and the raw value
    /// changes in one frame when a key goes down -- so a step to the side
    /// arrives as a cut from one cycle to another. Smoothing the *direction*
    /// rather than fading the result is the better fix: the stride phase still
    /// comes from distance walked, so the feet stay where they belong while the
    /// body turns into the new direction over a few frames.
    travel: [f32; 2],
}

impl Default for Crossfade {
    fn default() -> Self {
        Crossfade {
            from: Pose::rest(),
            drawn: Pose::rest(),
            shape: Shape::MAX,
            left: 0.0,
            length: 6.0,
            speed: 0.0,
            travel: [0.0, 1.0],
        }
    }
}

impl Crossfade {
    /// Pose this frame, fading out of whatever was on screen when the animation
    /// last changed. `dt` is real seconds, so the fade is the same length at any
    /// refresh rate.
    pub fn pose(&mut self, input: PoseInput, dt: f32) -> Pose {
        let mut input = input;
        // Slower coming down than going up. Starting to run is a decision and
        // should look like one; stopping is momentum running out, and blending
        // a sprint into an idle in five frames looks like the character was
        // switched off rather than slowed down.
        let rate = if input.speed > self.speed {
            0.20
        } else {
            0.075
        };
        let k = (dt * 60.0 * rate).clamp(0.0, 1.0);
        self.speed += (input.speed - self.speed) * k;
        input.eased_speed = self.speed;
        input.eased_travel = self.smooth_travel(input.travel, dt);
        // The shape is decided by the real speed, so stopping still starts a
        // fade at the moment it happens; only the blending is eased.
        let shape = shape_of(input);
        let target = pose_for(input);
        if shape != self.shape {
            // Fade from the picture that was actually on screen, not from the
            // old clip's idea of this frame: the point is that nothing jumps,
            // and a fade interrupted halfway has to continue from where it got
            // to rather than snapping back to where it started.
            self.from = if self.shape == Shape::MAX {
                target
            } else {
                self.drawn
            };
            self.shape = shape;
            self.left = self.length;
        }
        let out = if self.left > 0.0 {
            // Eased, so the fade has no corner at either end.
            let t = 1.0 - (self.left / self.length).clamp(0.0, 1.0);
            let k = t * t * (3.0 - 2.0 * t);
            self.from.blend(&target, k)
        } else {
            target
        };
        self.left = (self.left - dt * 60.0).max(0.0);
        self.drawn = out;
        out
    }

    /// Ease the travel *direction*, and give it the eased speed.
    ///
    /// The direction is remembered when the body stops, rather than collapsing
    /// to nothing: the eased speed takes several frames to come down, and a
    /// direction of zero during those frames leaves the walk blend with no
    /// clips to weight and nothing to draw.
    fn smooth_travel(&mut self, want: [f32; 2], dt: f32) -> [f32; 2] {
        let speed = (want[0] * want[0] + want[1] * want[1]).sqrt();
        if speed >= 0.2 {
            let unit = [want[0] / speed, want[1] / speed];
            let k = (dt * 60.0 * 0.16).clamp(0.0, 1.0);
            let blended = [
                self.travel[0] + (unit[0] - self.travel[0]) * k,
                self.travel[1] + (unit[1] - self.travel[1]) * k,
            ];
            let len = (blended[0] * blended[0] + blended[1] * blended[1]).sqrt();
            self.travel = if len < 1e-4 {
                unit
            } else {
                [blended[0] / len, blended[1] / len]
            };
        }
        [self.travel[0] * self.speed, self.travel[1] * self.speed]
    }
}

/// The whole animation system.
pub fn pose_for(input: PoseInput) -> Pose {
    if input.bind_pose {
        return Pose::rest();
    }

    // Dead overrides everything. A body that keeps guarding after the round is
    // over reads as a bug even when every other frame is right.
    if input.health <= 0 {
        if let Some(left) = input.round_left {
            let total = sim::tuning::round_over_frames();
            return Clip::Defeat.at(total.saturating_sub(left) as u32);
        }
        return Clip::Defeat.at(0);
    }

    match input.action {
        Action::Startup { kind, .. }
        | Action::Active { kind, .. }
        | Action::Recovery { kind, .. } => attack(input, kind),
        Action::Guard { held } => guard(input, held),
        Action::BlockStun { left } => from_the_end(Clip::BlockStun, left),
        Action::HitStun { left } => {
            if !input.grounded {
                Clip::Launched.at(input.air_frames as u32)
            } else if input.stun_total > Clip::HitLight.length() {
                from_the_end(Clip::HitHeavy, left)
            } else {
                from_the_end(Clip::HitLight, left)
            }
        }
        Action::Stagger { left } => from_the_end(Clip::Stagger, left),
        // Held loops for as long as somebody is holding you, so it wants a
        // clock that keeps going rather than one that ends.
        Action::Held { .. } => Clip::Grabbed.at(input.sim_frame),
        Action::Dodge { left } => dodge(input, left),
        Action::Free => free(input),
    }
}

/// An attack, indexed by how far into the move it is.
///
/// The clip's length comes from the move table, so this is a straight
/// correspondence: clip frame *n* is move frame *n*, and the contact pose is on
/// the frame the hitbox appears. Nothing lines up by accident.
fn attack(input: PoseInput, kind: u8) -> Pose {
    let (startup, active, _) = sim::moves::frames(input.class, kind);
    let elapsed = match input.action {
        Action::Startup { left, .. } => startup.saturating_sub(left),
        Action::Active { left, .. } => startup + active.saturating_sub(left),
        Action::Recovery { left, .. } => {
            let (_, _, recovery) = sim::moves::frames(input.class, kind);
            startup + active + recovery.saturating_sub(left)
        }
        _ => 0,
    };
    let clip = move_clip(input.class, kind);
    let pose = aim_along(clip.at(elapsed as u32), input, kind);

    // A move you can walk during should walk. `mobility` is the fraction of
    // walking speed the move leaves you, and a poke thrown on the move with
    // its feet nailed down is the tell that an attack was animated in
    // isolation from the locomotion it interrupts.
    let mobility = sim::moves::get(input.class, kind).mobility as f32 / 100.0;
    if mobility > 0.05 && input.eased_speed > 0.6 && input.grounded {
        let weight = (input.eased_speed / RUN_AT).clamp(0.0, 1.0) * mobility * 0.7;
        let legs = locomotion(input);
        return pose.blend(&pose.take_group(&legs, Group::Legs), weight);
    }
    pose
}

/// Tilt a shot's pose on to the line it is actually fired along.
///
/// The clips are authored level, because a clip is a shape and not an angle.
/// One move in the game leaves the body along the crosshair rather than along
/// the flat facing -- the Elementalist's auto, which is a ray -- and drawing
/// that one level meant the character threw a horizontal flick while the shot
/// went up at forty degrees. The bolt's own author's note said as much: the
/// off hand was "the only thing in the pose that says which way the bolt
/// went".
///
/// Spread over three joints rather than put on one. A body that aims upward
/// opens at the waist, opens again at the chest, and raises the shoulders; all
/// of it on the shoulder would be a raised arm on a level torso, which reads as
/// a shrug. The shares add to a little over one so the hand clears the line the
/// eye is on.
fn aim_along(pose: Pose, input: PoseInput, kind: u8) -> Pose {
    if !aims_along_the_crosshair(input.class, kind) {
        return pose;
    }
    let degrees = input.aim_pitch.to_degrees();
    if degrees.abs() < 0.5 {
        return pose;
    }
    // Negative bend opens the torso upward, positive swing lifts an arm
    // forward and up -- the conventions `view/tests/kinematics.rs` pins.
    let mut out = pose;
    for (joint, share) in [
        (Joint::Spine, -0.25),
        (Joint::Chest, -0.35),
        (Joint::ArmL, 0.55),
        (Joint::ArmR, 0.55),
    ] {
        let c = 0; // swing for a limb, bend for a torso segment: the same channel
        out.set_degrees(joint, c, out.degrees(joint, c) + degrees * share);
    }
    // The head goes with the aim, and by the whole angle: she is looking at
    // what she is shooting at, and a head left on the horizon is the one part
    // of this that would be noticed as wrong.
    out.set_degrees(Joint::Head, 0, out.degrees(Joint::Head, 0) - degrees * 0.6);
    out.clamped(crate::pose::reference())
}

/// Does this move leave the body at an angle, rather than level?
///
/// Straight from the move table, so the animation cannot disagree with the
/// simulation about which moves are tilted. Two of the four are: a skillshot
/// flies at what the crosshair is on, and a swing comes out along the body
/// tilted by the camera's pitch outside its dead zone. The other two are not —
/// a grounded cast comes out of the floor and the body is only gesturing at it,
/// and a move aimed at the mechanic is pointed at something the player placed
/// earlier.
///
/// The angle itself arrives as `aim_pitch`, already dead-zoned: by the time the
/// pose sees it, it is the angle the attack actually came out at.
pub fn aims_along_the_crosshair(class: Class, kind: u8) -> bool {
    matches!(
        sim::moves::get(class, kind).aim(),
        sim::aim::Kind::Skillshot | sim::aim::Kind::Swing
    )
}

/// Which clip animates one class's move slot.
///
/// Clamped to the last slot the class actually binds, so a class whose mechanic
/// is a state change rather than an ability never asks for a clip that does not
/// exist -- see `sim::moves::bound`.
pub fn move_clip(class: Class, slot: u8) -> Clip {
    // Clamped per class, because they no longer all have three: the Blood mage
    // has four and the Champion ten, one per square of its weapon-by-stance
    // grid. A slot a class does not have falls back to its special.
    let slot = slot.min(sim::moves::slots(class) as u8 - 1);
    match (class, slot) {
        (Class::Bulwark, 0) => Clip::BulwarkPoke,
        (Class::Bulwark, 1) => Clip::BulwarkCommitted,
        (Class::Bulwark, _) => Clip::BulwarkSpecial,
        (Class::Champion, 0) => Clip::ChampionSword,
        (Class::Champion, 1) => Clip::ChampionHammer,
        (Class::Champion, 2) => Clip::ChampionSpear,
        (Class::Champion, 3) => Clip::ChampionAirSword,
        (Class::Champion, 4) => Clip::ChampionAirHammer,
        (Class::Champion, 5) => Clip::ChampionAirSpear,
        (Class::Champion, 6) => Clip::ChampionRushSlash,
        (Class::Champion, 7) => Clip::ChampionUppercut,
        (Class::Champion, 8) => Clip::ChampionRushStab,
        (Class::Champion, _) => Clip::ChampionVault,
        (Class::ShadowReaver, 0) => Clip::ReaverPoke,
        (Class::ShadowReaver, 1) => Clip::ReaverCommitted,
        (Class::ShadowReaver, _) => Clip::ReaverSpecial,
        (Class::Elementalist, 0) => Clip::ElementalistPoke,
        (Class::Elementalist, 1) => Clip::ElementalistCommitted,
        (Class::Elementalist, _) => Clip::ElementalistSpecial,
        (Class::BloodMage, 0) => Clip::BloodPoke,
        (Class::BloodMage, 1) => Clip::BloodCommitted,
        (Class::BloodMage, 2) => Clip::BloodSpecial,
        (Class::BloodMage, _) => Clip::BloodMechanic,
        (Class::DualMage, 0) => Clip::DualPoke,
        (Class::DualMage, 1) => Clip::DualCommitted,
        (Class::DualMage, _) => Clip::DualSpecial,
    }
}

fn guard(input: PoseInput, held: u16) -> Pose {
    // A parry that just landed takes over. It is the one piece of feedback the
    // defender gets for the hardest input in the game.
    if input.parried > 0 {
        let elapsed = sim::state::PARRY_FLOURISH.saturating_sub(input.parried);
        return Clip::Parry.at(elapsed as u32);
    }
    let entry = Clip::GuardIn.length();
    if held < entry {
        Clip::GuardIn.at(held as u32)
    } else {
        Clip::GuardIdle.at((held - entry) as u32)
    }
}

/// Index a clip backwards from its last frame, so it finishes exactly as the
/// countdown does.
fn from_the_end(clip: Clip, left: u16) -> Pose {
    let last = clip.length().saturating_sub(1);
    clip.at(last.saturating_sub(left.min(last)) as u32)
}

fn dodge(input: PoseInput, left: u16) -> Pose {
    if !input.grounded {
        let len = Clip::AirDodge.length();
        return Clip::AirDodge.at(len.saturating_sub(left) as u32);
    }
    // Which way the body is actually going, not which key was pressed: a dodge
    // is a burst of velocity and the velocity is the honest source.
    let [strafe, forward] = input.travel;
    let clip = if forward.abs() >= strafe.abs() {
        if forward >= 0.0 {
            Clip::DodgeForward
        } else {
            Clip::DodgeBack
        }
    } else if strafe >= 0.0 {
        Clip::DodgeRight
    } else {
        Clip::DodgeLeft
    };
    let len = clip.length();
    clip.at(len.saturating_sub(left) as u32)
}

/// Nobody is doing anything to this character and it is not attacking: the
/// state a player spends most of a match in, and the one worth the most care.
fn free(input: PoseInput) -> Pose {
    if !input.grounded {
        return airborne(input);
    }

    // Landing, if the flight was long enough to be worth acknowledging.
    if input.air_frames >= LANDING_WORTH_PLAYING {
        let clip = if input.air_frames >= HARD_FALL {
            Clip::LandHeavy
        } else {
            Clip::LandSoft
        };
        let elapsed = input.since_landed.saturating_sub(1);
        if elapsed < clip.length() {
            let landing = clip.at(elapsed as u32);
            // Blend out of the landing into whatever the player is already
            // doing, rather than locking the controls away for its duration.
            let out = (elapsed as f32 / clip.length() as f32).powi(2);
            return landing.blend(&grounded(input), out);
        }
    }

    grounded(input)
}

fn airborne(input: PoseInput) -> Pose {
    let takeoff = Clip::JumpTakeoff.length();
    if input.air_frames < takeoff {
        let pose = Clip::JumpTakeoff.at(input.air_frames as u32);
        // **Hand over rather than cut.** The takeoff's last frame and the
        // flight's first are two separately authored poses, and the join
        // between them was a hard switch inside one shape -- so the crossfade
        // in `Crossfade` never saw it and never softened it. How bad the seam
        // looked depended on how fast you were going when you left the floor,
        // which is exactly the kind of bug that hides: fine most of the time,
        // and a visible hitch on the jumps that happen mid-sprint.
        //
        // The last few frames of the takeoff blend into the flight pose they
        // are about to become, which costs nothing and removes the seam
        // entirely.
        const HANDOVER: u16 = 3;
        let into = takeoff.saturating_sub(HANDOVER);
        if input.air_frames >= into {
            let k = (input.air_frames - into + 1) as f32 / HANDOVER as f32;
            return pose.blend(&flight(input, 0), k.clamp(0.0, 1.0));
        }
        return pose;
    }
    flight(input, (input.air_frames - takeoff) as u32)
}

/// Rising, hanging, falling -- picked from vertical speed, and cross-faded so
/// the turn at the top is a turn rather than a cut.
fn flight(input: PoseInput, since: u32) -> Pose {
    const HANG: f32 = 3.5;
    if input.rise > HANG {
        Clip::JumpRise.at(since)
    } else if input.rise < -HANG {
        Clip::JumpFall.at(since)
    } else {
        let k = (input.rise / HANG).clamp(-1.0, 1.0);
        let apex = Clip::JumpApex.at(since);
        if k >= 0.0 {
            apex.blend(&Clip::JumpRise.at(since), k)
        } else {
            apex.blend(&Clip::JumpFall.at(since), -k)
        }
    }
}

fn grounded(input: PoseInput) -> Pose {
    let base = if input.crouching {
        crouched(input)
    } else {
        locomotion(input)
    };
    turn_layer(input, base)
}

fn crouched(input: PoseInput) -> Pose {
    let entry = Clip::CrouchIn.length();
    let settle = (input.since_landed.min(entry) as f32 / entry as f32).clamp(0.0, 1.0);
    let held = if input.speed > 0.4 {
        Clip::CrouchWalk.at_fractional(input.stride * Clip::CrouchWalk.length() as f32)
    } else {
        Clip::CrouchIdle.at(input.since_landed as u32)
    };
    Clip::CrouchIn.at(0).blend(&held, settle.max(0.35))
}

/// Idle, walk and run, blended by speed and by which way the body is going.
///
/// The four directional clips are sampled at the **same stride phase** before
/// being blended. Sampling them independently is the classic way to get a
/// diagonal walk with both feet planted at once, because the two clips drift
/// out of step with each other within a couple of seconds.
fn locomotion(input: PoseInput) -> Pose {
    let speed = input.eased_speed;
    if speed < 0.15 {
        return Clip::Idle.at(idle_phase(input));
    }

    let gait = ((speed - WALK_AT) / (RUN_AT - WALK_AT)).clamp(0.0, 1.0);
    // The phase is already in cycles: the simulation integrated `speed / stride`
    // to get it, which is the only way to keep it continuous when the stride
    // itself changes with speed and direction.
    let phase = input.stride;

    let walk = directional(input, phase, false);
    let run = directional(input, phase, true);
    let moving = walk.blend(&run, gait);

    // Below a walk, fade out of the idle rather than snapping into a stride.
    let into_stride = (speed / WALK_AT).clamp(0.0, 1.0);
    Clip::Idle.at(idle_phase(input)).blend(&moving, into_stride)
}

/// Idle is the one cyclic clip with no ground covered to run on, so it uses the
/// frame counter -- which is in the snapshot, so it rolls back cleanly, and
/// which never stops advancing, so an idle held for twenty minutes does not
/// freeze the way a saturating counter would.
fn idle_phase(input: PoseInput) -> u32 {
    input.sim_frame
}

/// The four directions, weighted by where the body is actually going.
fn directional(input: PoseInput, phase: f32, run: bool) -> Pose {
    let (fwd, back, left, right) = if run {
        (
            Clip::RunForward,
            Clip::RunBack,
            Clip::RunLeft,
            Clip::RunRight,
        )
    } else {
        (
            Clip::WalkForward,
            Clip::WalkBack,
            Clip::WalkLeft,
            Clip::WalkRight,
        )
    };

    let [strafe, forward] = input.eased_travel;
    let len = (strafe * strafe + forward * forward).sqrt();
    // Degenerate only if something upstream handed us a zero vector at speed;
    // facing forward is the honest answer rather than a pose of nothing.
    let (sx, sz) = if len < 1e-3 {
        (0.0, 1.0)
    } else {
        (strafe / len, forward / len)
    };

    let w = [
        (fwd, sz.max(0.0)),
        (back, (-sz).max(0.0)),
        (right, sx.max(0.0)),
        (left, (-sx).max(0.0)),
    ];
    let total: f32 = w.iter().map(|(_, k)| k).sum::<f32>().max(1e-4);

    let mut out = Pose::rest();
    let mut first = true;
    let mut used = 0.0;
    for (clip, weight) in w {
        let k = weight / total;
        if k < 1e-3 {
            continue;
        }
        let sampled = clip.at_fractional(phase * clip.length() as f32);
        if first {
            out = sampled;
            used = k;
            first = false;
        } else {
            used += k;
            out = out.blend(&sampled, k / used);
        }
    }
    out
}

/// Lean into a turn, on top of whatever else is happening.
///
/// Layered as a *difference* from the turn clip's own first frame, so a turn
/// authored as a full-body action can be applied over a walk without erasing
/// the walk's legs.
fn turn_layer(input: PoseInput, base: Pose) -> Pose {
    let rate = input.turn_rate.abs();
    if rate < TURN_SLOW * 0.25 {
        return base;
    }
    let right = input.turn_rate > 0.0;
    let (slow, fast) = if right {
        (Clip::TurnRightSlow, Clip::TurnRightFast)
    } else {
        (Clip::TurnLeftSlow, Clip::TurnLeftFast)
    };

    // How committed the turn is, and how fast it is: one picks the frame within
    // a clip, the other picks between the two clips.
    let commitment = (rate / TURN_FAST).clamp(0.0, 1.0);
    let hardness = ((rate - TURN_SLOW) / (TURN_FAST - TURN_SLOW)).clamp(0.0, 1.0);

    let sample = |clip: Clip| {
        let last = (clip.length().saturating_sub(1)) as f32;
        let at = clip.at_fractional(commitment * last);
        at.difference(&clip.at(0))
    };
    let delta = sample(slow).blend(&sample(fast), hardness);

    // Walking absorbs a lot of a turn already; layering the full lean on top of
    // a run bends the character in half.
    let damped = 1.0 - 0.45 * (input.eased_speed / RUN_AT).clamp(0.0, 1.0);
    base.layered(&delta, damped)
        .clamped(crate::pose::reference())
}
