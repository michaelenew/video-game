//! Character posing.
//!
//! **The rule that everything else depends on: pose is a pure function of
//! simulation state.** No accumulated animation time, no independently ticking
//! player. Rollback re-simulates past frames, so anything animating on its own
//! clock pops and slides every time a rollback happens.
//!
//! ```text
//! pose = f(action, frames_into_action, speed, grounded, sim_frame)
//! ```
//!
//! `sim_frame` is part of the snapshot, so driving cyclic motion from it stays
//! deterministic. Cosmetic smoothing may live in the renderer and is allowed to
//! pop across a rollback -- one to eight frames of visual discontinuity is
//! imperceptible. Nothing that reads as gameplay may.
//!
//! This module writes transforms by hand for primitive standins. Swapping in
//! skeletal glTF later changes what `pose_for` returns, not how any of this
//! works -- which is exactly why the standins come first.

use sim::state::Action;

/// Parts of the standin. Six is enough to read a silhouette at gameplay
/// distance, which is the actual requirement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Torso,
    Head,
    ArmL,
    ArmR,
    LegL,
    LegR,
}

pub const PART_COUNT: usize = 6;
pub const PARTS: [Part; PART_COUNT] = [
    Part::Torso,
    Part::Head,
    Part::ArmL,
    Part::ArmR,
    Part::LegL,
    Part::LegR,
];

/// Character-local: root at the feet, +Y up, +Z forward along facing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PartTransform {
    pub pos: [f32; 3],
    /// Euler XYZ in radians.
    pub rot: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub parts: [PartTransform; PART_COUNT],
}

impl Pose {
    pub fn get(&self, part: Part) -> PartTransform {
        self.parts[part as usize]
    }
}

/// Which baked clip, if any, this action should play.
///
/// The caller picks, because it is the side that knows what move is running --
/// whether it is an overhead, how long it lasts. `view` does not read the move
/// tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clip {
    /// One of the shared clips: guarding, rolling, being hit.
    Shared(Shared),
    /// A specific move's own clip, timed to that move's own frame data.
    ///
    /// Indexed rather than named because the table is generated wholesale from
    /// `sim::moves` -- see `anim::derive`. The caller does the lookup, because
    /// it is the side that knows the move tables; `view` stays ignorant of what
    /// an overhead is.
    Move(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shared {
    GuardIn,
    Roll,
    Recoil,
}

impl Clip {
    fn frames(self) -> &'static [Pose] {
        match self {
            Clip::Shared(Shared::GuardIn) => &crate::baked::GUARD_IN,
            Clip::Shared(Shared::Roll) => &crate::baked::ROLL,
            Clip::Shared(Shared::Recoil) => &crate::baked::RECOIL,
            Clip::Move(i) => crate::baked::MOVE_CLIPS
                .get(i)
                .copied()
                .unwrap_or(&crate::baked::POKE),
        }
    }
}

/// Everything posing is allowed to depend on. All of it comes from the
/// simulation snapshot.
#[derive(Clone, Copy, Debug)]
pub struct PoseInput {
    /// Which baked clip to play, and how far into it. `None` falls back to the
    /// procedural poses, which is the toggle: baked animation can be switched
    /// off wholesale if it ever costs more than it is worth.
    pub clip: Option<(Clip, u16)>,
    pub action: Action,
    /// Frames elapsed within the current action.
    pub frames_into: u16,
    /// Total frames of the current phase; zero when the phase is open-ended.
    pub frames_total: u16,
    pub speed: f32,
    pub grounded: bool,
    pub crouching: bool,
    /// Simulation frame. Deterministic, so cyclic motion driven from it is too.
    pub sim_frame: u32,
}

const NEUTRAL: Pose = Pose {
    parts: [
        PartTransform {
            pos: [0.0, 0.95, 0.0],
            rot: [0.0, 0.0, 0.0],
        }, // Torso
        PartTransform {
            pos: [0.0, 1.55, 0.0],
            rot: [0.0, 0.0, 0.0],
        }, // Head
        PartTransform {
            pos: [-0.42, 1.05, 0.0],
            rot: [0.0, 0.0, 0.12],
        }, // ArmL
        PartTransform {
            pos: [0.42, 1.05, 0.0],
            rot: [0.0, 0.0, -0.12],
        }, // ArmR
        PartTransform {
            pos: [-0.18, 0.38, 0.0],
            rot: [0.0, 0.0, 0.0],
        }, // LegL
        PartTransform {
            pos: [0.18, 0.38, 0.0],
            rot: [0.0, 0.0, 0.0],
        }, // LegR
    ],
};

/// Wind up: weight back, striking arm cocked. Deliberately a large silhouette
/// change -- the opponent has to read startup from across the arena.
const WINDUP: Pose = Pose {
    parts: [
        PartTransform {
            pos: [0.0, 0.90, -0.18],
            rot: [0.0, -0.45, 0.0],
        },
        PartTransform {
            pos: [0.0, 1.50, -0.16],
            rot: [0.0, -0.30, 0.0],
        },
        PartTransform {
            pos: [-0.40, 1.02, 0.10],
            rot: [0.0, 0.0, 0.30],
        },
        PartTransform {
            pos: [0.46, 1.22, -0.40],
            rot: [-1.20, 0.0, -0.30],
        },
        PartTransform {
            pos: [-0.20, 0.38, -0.10],
            rot: [-0.20, 0.0, 0.0],
        },
        PartTransform {
            pos: [0.22, 0.38, 0.14],
            rot: [0.30, 0.0, 0.0],
        },
    ],
};

/// Strike: everything committed forward. Maximum contrast with WINDUP.
const STRIKE: Pose = Pose {
    parts: [
        PartTransform {
            pos: [0.0, 0.92, 0.26],
            rot: [0.0, 0.40, 0.0],
        },
        PartTransform {
            pos: [0.0, 1.50, 0.24],
            rot: [0.0, 0.26, 0.0],
        },
        PartTransform {
            pos: [-0.44, 0.98, -0.18],
            rot: [0.0, 0.0, 0.36],
        },
        PartTransform {
            pos: [0.30, 1.10, 0.72],
            rot: [1.35, 0.0, -0.16],
        },
        PartTransform {
            pos: [-0.20, 0.38, 0.20],
            rot: [0.34, 0.0, 0.0],
        },
        PartTransform {
            pos: [0.22, 0.38, -0.16],
            rot: [-0.26, 0.0, 0.0],
        },
    ],
};

/// Guard: side-on, shield arm across. Reads as "cannot be hit from the front".
const GUARD: Pose = Pose {
    parts: [
        PartTransform {
            pos: [0.0, 0.88, 0.0],
            rot: [0.0, 0.70, 0.0],
        },
        PartTransform {
            pos: [0.0, 1.46, 0.06],
            rot: [0.10, 0.40, 0.0],
        },
        PartTransform {
            pos: [-0.16, 1.14, 0.44],
            rot: [-0.90, 0.0, 0.55],
        },
        PartTransform {
            pos: [0.40, 1.00, 0.10],
            rot: [-0.30, 0.0, -0.25],
        },
        PartTransform {
            pos: [-0.24, 0.36, 0.06],
            rot: [0.0, 0.0, 0.10],
        },
        PartTransform {
            pos: [0.24, 0.36, -0.10],
            rot: [0.0, 0.0, -0.10],
        },
    ],
};

/// Dodge roll: low and tucked. Must read as clearly evasive from across the
/// arena, because the opponent has to be able to tell a dodge from a walk.
const ROLL: Pose = Pose {
    parts: [
        PartTransform {
            pos: [0.0, 0.52, 0.10],
            rot: [-1.05, 0.0, 0.0],
        },
        PartTransform {
            pos: [0.0, 0.86, 0.34],
            rot: [-0.90, 0.0, 0.0],
        },
        PartTransform {
            pos: [-0.36, 0.62, 0.26],
            rot: [-1.30, 0.0, 0.45],
        },
        PartTransform {
            pos: [0.36, 0.62, 0.26],
            rot: [-1.30, 0.0, -0.45],
        },
        PartTransform {
            pos: [-0.18, 0.30, -0.22],
            rot: [-1.10, 0.0, 0.0],
        },
        PartTransform {
            pos: [0.18, 0.30, -0.28],
            rot: [-1.30, 0.0, 0.0],
        },
    ],
};

/// Crouch: low and compact. Has to read from across the arena, because the
/// attacker needs to know their overhead will whiff.
const CROUCH: Pose = Pose {
    parts: [
        PartTransform {
            pos: [0.0, 0.56, 0.0],
            rot: [0.30, 0.0, 0.0],
        },
        PartTransform {
            pos: [0.0, 1.00, 0.10],
            rot: [0.20, 0.0, 0.0],
        },
        PartTransform {
            pos: [-0.40, 0.66, 0.10],
            rot: [-0.55, 0.0, 0.25],
        },
        PartTransform {
            pos: [0.40, 0.66, 0.10],
            rot: [-0.55, 0.0, -0.25],
        },
        PartTransform {
            pos: [-0.20, 0.22, 0.04],
            rot: [0.80, 0.0, 0.0],
        },
        PartTransform {
            pos: [0.20, 0.22, 0.04],
            rot: [0.80, 0.0, 0.0],
        },
    ],
};

/// Recoil: knocked off balance, arms trailing.
const RECOIL: Pose = Pose {
    parts: [
        PartTransform {
            pos: [0.0, 0.86, -0.30],
            rot: [-0.36, 0.0, 0.0],
        },
        PartTransform {
            pos: [0.0, 1.42, -0.40],
            rot: [-0.55, 0.0, 0.0],
        },
        PartTransform {
            pos: [-0.52, 1.14, -0.24],
            rot: [-0.70, 0.0, 0.60],
        },
        PartTransform {
            pos: [0.52, 1.14, -0.24],
            rot: [-0.70, 0.0, -0.60],
        },
        PartTransform {
            pos: [-0.18, 0.38, 0.16],
            rot: [0.34, 0.0, 0.0],
        },
        PartTransform {
            pos: [0.20, 0.38, -0.06],
            rot: [-0.14, 0.0, 0.0],
        },
    ],
};

/// Airborne: legs tucked.
const AIRBORNE: Pose = Pose {
    parts: [
        PartTransform {
            pos: [0.0, 0.95, 0.0],
            rot: [0.10, 0.0, 0.0],
        },
        PartTransform {
            pos: [0.0, 1.55, 0.0],
            rot: [0.0, 0.0, 0.0],
        },
        PartTransform {
            pos: [-0.46, 1.20, -0.06],
            rot: [-0.50, 0.0, 0.40],
        },
        PartTransform {
            pos: [0.46, 1.20, -0.06],
            rot: [-0.50, 0.0, -0.40],
        },
        PartTransform {
            pos: [-0.18, 0.50, 0.12],
            rot: [0.60, 0.0, 0.0],
        },
        PartTransform {
            pos: [0.18, 0.46, -0.06],
            rot: [-0.30, 0.0, 0.0],
        },
    ],
};

/// The whole animation system.
pub fn pose_for(input: PoseInput) -> Pose {
    // Baked playback is a lookup by frame, so it stays a pure function of
    // simulation state and rollback is unaffected. See `anim`.
    if let Some((clip, elapsed)) = input.clip {
        let frames = clip.frames();
        if !frames.is_empty() {
            return frames[(elapsed as usize).min(frames.len() - 1)];
        }
    }

    let t = phase_progress(input);

    match input.action {
        Action::Startup { .. } => blend(&NEUTRAL, &WINDUP, ease_out(t)),
        // Active frames snap to the strike immediately. The hitbox is live now;
        // the pose must not lag it, or players learn to read the wrong thing.
        Action::Active { .. } => STRIKE,
        Action::Recovery { .. } => blend(&STRIKE, &NEUTRAL, ease_in_out(t)),
        Action::Guard { held } => {
            // The parry window gets a distinct, tighter stance so the defender
            // can see their own timing.
            let settle = (held as f32 / 6.0).clamp(0.0, 1.0);
            blend(&NEUTRAL, &GUARD, ease_out(settle))
        }
        Action::BlockStun { .. } | Action::HitStun { .. } | Action::Stagger { .. } => {
            blend(&RECOIL, &NEUTRAL, ease_in_out(t * 0.7))
        }
        // Held: the recoil, and it does not relax. Being grabbed should read as
        // continuous, not as a hit you are walking off.
        Action::Held { .. } => RECOIL,
        Action::Dodge { .. } => {
            // Snap into the roll and come out of it, so the invulnerable
            // frames and the vulnerable recovery look different.
            let out = ease_in_out((t * 1.8 - 0.8).clamp(0.0, 1.0));
            blend(&ROLL, &NEUTRAL, out)
        }
        Action::Free => {
            if !input.grounded {
                AIRBORNE
            } else if input.crouching {
                CROUCH
            } else {
                locomotion(input)
            }
        }
    }
}

/// Idle breathing and a walk cycle, both driven from `sim_frame` so they stay
/// deterministic and survive rollback.
fn locomotion(input: PoseInput) -> Pose {
    let mut pose = NEUTRAL;
    let phase = input.sim_frame as f32;

    let moving = (input.speed / 7.0).clamp(0.0, 1.0);
    if moving > 0.01 {
        let swing = (phase * 0.42).sin() * 0.62 * moving;
        let counter = (phase * 0.42).sin() * 0.34 * moving;
        pose.parts[Part::LegL as usize].rot[0] = swing;
        pose.parts[Part::LegR as usize].rot[0] = -swing;
        pose.parts[Part::LegL as usize].pos[2] = swing * 0.22;
        pose.parts[Part::LegR as usize].pos[2] = -swing * 0.22;
        pose.parts[Part::ArmL as usize].rot[0] = -counter;
        pose.parts[Part::ArmR as usize].rot[0] = counter;
        // Bob twice per stride.
        let bob = (phase * 0.84).sin() * 0.045 * moving;
        pose.parts[Part::Torso as usize].pos[1] += bob;
        pose.parts[Part::Head as usize].pos[1] += bob;
    } else {
        let breath = (phase * 0.06).sin() * 0.022;
        pose.parts[Part::Torso as usize].pos[1] += breath;
        pose.parts[Part::Head as usize].pos[1] += breath;
    }
    pose
}

fn phase_progress(input: PoseInput) -> f32 {
    if input.frames_total == 0 {
        0.0
    } else {
        (input.frames_into as f32 / input.frames_total as f32).clamp(0.0, 1.0)
    }
}

fn blend(a: &Pose, b: &Pose, t: f32) -> Pose {
    let mut out = *a;
    for i in 0..PART_COUNT {
        for k in 0..3 {
            out.parts[i].pos[k] = a.parts[i].pos[k] + (b.parts[i].pos[k] - a.parts[i].pos[k]) * t;
            out.parts[i].rot[k] = a.parts[i].rot[k] + (b.parts[i].rot[k] - a.parts[i].rot[k]) * t;
        }
    }
    out
}

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

/// Fixed part dimensions, in metres. The renderer builds boxes from these once.
pub fn part_size(part: Part) -> [f32; 3] {
    crate::build::Build::EVEN.part_size(part)
}
