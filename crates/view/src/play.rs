//! Choosing which animation to play, and blending between them.
//!
//! Everything here is a pure function of the simulation snapshot. That is not
//! a style preference: rollback re-simulates past frames, so a clip that
//! advances on its own clock plays a different frame the second time through
//! and the character visibly jumps. Blending is allowed -- a blend of two pure
//! functions is still one -- but nothing may *accumulate*.
//!
//! ```text
//! pose = f(action, frames into it, distance walked, airtime, turn rate, health)
//! ```
//!
//! ## Three tricks worth knowing about
//!
//! **Locomotion is driven by distance, not by time.** A walk cycle on a fixed
//! cadence slides its feet the moment the body moves at any other speed. Ours
//! indexes the cycle by ground covered, so a footfall happens every stride's
//! worth of metres, at any speed, and the directional clips are sampled at the
//! *same phase* before being blended -- which is what keeps a diagonal from
//! producing two feet on the ground at once.
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
use crate::skeleton::Group;
use sim::Class;
use sim::state::Action;

/// Speed at which the walk cycle is at full weight, in metres per second.
///
/// The guarding walk speed is two, and it is the speed a fighter spends most of
/// a tense exchange at, so that is what "walking" is calibrated to. The free
/// movement speed of seven is a run by any honest measure, whatever the tuning
/// knob is called.
pub const WALK_AT: f32 = 2.2;
pub const RUN_AT: f32 = 7.0;

/// How much ground one full cycle of each clip covers.
///
/// **This is the number that decides whether the feet skate**, and the clips are
/// authored against it: the recipes in `anim::clips::locomotion` import these
/// constants and place every planted foot by arithmetic from them. Changing one
/// without re-baking makes the feet slide, which is why they live here, once.
///
/// The values are not free choices. A leg is 0.87 m long and a hip is 0.86 m off
/// the floor at contact, so a foot can be at most about 0.34 m ahead of the hip
/// before the leg runs out -- and stride length follows from that, not the other
/// way round.
pub const WALK_STRIDE: f32 = 1.10;
pub const RUN_STRIDE: f32 = 2.70;
pub const CROUCH_STRIDE: f32 = 0.80;

/// A sidestep covers less ground per cycle than a stride forward, because a leg
/// swung sideways runs out of hip long before one swung forward runs out of
/// leg. Applied in proportion to how sideways the movement actually is.
pub const STRAFE_STRIDE: f32 = 0.72;

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
    /// Metres walked, wrapping.
    pub distance: f32,
    pub air_frames: u16,
    pub since_landed: u16,
    pub parried: u16,
    pub stun_total: u16,
    pub rise: f32,
    /// Turns per second, positive to the character's right.
    pub turn_rate: f32,
    pub health: i32,
    /// Frames left of the between-rounds pause, if a round has ended.
    pub round_left: Option<u16>,
    /// Show the skeleton at rest instead of animating it. The toggle exists so
    /// that "is this the clip or is this the rig?" can be answered in one
    /// keypress while looking at the thing.
    pub bind_pose: bool,
}

impl PoseInput {
    pub fn of(view: &PlayerView, class: Class, round_left: Option<u16>) -> PoseInput {
        PoseInput {
            class,
            action: view.action,
            grounded: view.grounded,
            crouching: view.crouching,
            speed: view.speed,
            travel: view.travel,
            distance: view.distance,
            air_frames: view.air_frames,
            since_landed: view.since_landed,
            parried: view.parried,
            stun_total: view.stun_total,
            rise: view.rise,
            turn_rate: view.turn_rate,
            health: view.health,
            round_left,
            bind_pose: false,
        }
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
        Action::Held { .. } => Clip::Grabbed.at(input.since_landed as u32),
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
    let pose = clip.at(elapsed as u32);

    // A move you can walk during should walk. `mobility` is the fraction of
    // walking speed the move leaves you, and a poke thrown on the move with
    // its feet nailed down is the tell that an attack was animated in
    // isolation from the locomotion it interrupts.
    let mobility = sim::moves::get(input.class, kind).mobility as f32 / 100.0;
    if mobility > 0.05 && input.speed > 0.6 && input.grounded {
        let weight = (input.speed / RUN_AT).clamp(0.0, 1.0) * mobility * 0.7;
        let legs = locomotion(input);
        return pose.blend(&pose.take_group(&legs, Group::Legs), weight);
    }
    pose
}

/// Which clip animates one class's move slot.
pub fn move_clip(class: Class, slot: u8) -> Clip {
    let slot = slot.min(2);
    match (class, slot) {
        (Class::Bulwark, 0) => Clip::BulwarkPoke,
        (Class::Bulwark, 1) => Clip::BulwarkCommitted,
        (Class::Bulwark, _) => Clip::BulwarkSpecial,
        (Class::Champion, 0) => Clip::ChampionPoke,
        (Class::Champion, 1) => Clip::ChampionCommitted,
        (Class::Champion, _) => Clip::ChampionSpecial,
        (Class::ShadowReaver, 0) => Clip::ReaverPoke,
        (Class::ShadowReaver, 1) => Clip::ReaverCommitted,
        (Class::ShadowReaver, _) => Clip::ReaverSpecial,
        (Class::Elementalist, 0) => Clip::ElementalistPoke,
        (Class::Elementalist, 1) => Clip::ElementalistCommitted,
        (Class::Elementalist, _) => Clip::ElementalistSpecial,
        (Class::BloodMage, 0) => Clip::BloodPoke,
        (Class::BloodMage, 1) => Clip::BloodCommitted,
        (Class::BloodMage, _) => Clip::BloodSpecial,
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
        return Clip::JumpTakeoff.at(input.air_frames as u32);
    }
    let since = (input.air_frames - takeoff) as u32;

    // Rising, hanging, falling -- picked from vertical speed, and cross-faded
    // so the turn at the top is a turn rather than a cut.
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
        let phase = input.distance / CROUCH_STRIDE;
        Clip::CrouchWalk.at_fractional(phase * Clip::CrouchWalk.length() as f32)
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
    let speed = input.speed;
    if speed < 0.15 {
        return Clip::Idle.at(idle_phase(input));
    }

    let gait = ((speed - WALK_AT) / (RUN_AT - WALK_AT)).clamp(0.0, 1.0);
    let stride = WALK_STRIDE + (RUN_STRIDE - WALK_STRIDE) * gait;
    // Shortened in proportion to how sideways the travel is, matching how the
    // strafe clips were authored. Get this wrong and the sidesteps skate even
    // though the forward walk does not.
    let [strafe, forward] = input.travel;
    let sideways = strafe.abs() / (strafe.abs() + forward.abs()).max(1e-4);
    let stride = stride * (1.0 - (1.0 - STRAFE_STRIDE) * sideways);
    let phase = input.distance / stride.max(0.2);

    let walk = directional(input, phase, false);
    let run = directional(input, phase, true);
    let moving = walk.blend(&run, gait);

    // Below a walk, fade out of the idle rather than snapping into a stride.
    let into_stride = (speed / WALK_AT).clamp(0.0, 1.0);
    Clip::Idle.at(idle_phase(input)).blend(&moving, into_stride)
}

/// Idle is the one cyclic clip with no distance to run on, so it uses the
/// frame counter -- which is in the snapshot, so it rolls back cleanly.
fn idle_phase(input: PoseInput) -> u32 {
    input.since_landed as u32
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

    let [strafe, forward] = input.travel;
    let len = (strafe * strafe + forward * forward).sqrt().max(1e-4);
    let (sx, sz) = (strafe / len, forward / len);

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
    let damped = 1.0 - 0.45 * (input.speed / RUN_AT).clamp(0.0, 1.0);
    base.layered(&delta, damped)
        .clamped(crate::pose::reference())
}
