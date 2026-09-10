//! Auto-framing camera.
//!
//! The control scheme frees the camera: aim comes from WASD, facing is
//! automatic, and the mouse buttons are attacks. Nothing needs mouse-look. So
//! the camera can frame the fight the way a 3D fighting game does -- keeping
//! both combatants in view and pulling back as they separate -- rather than
//! sitting behind one player's shoulder.
//!
//! That is a readability win in 1v1 and it removes a control problem entirely.
//! It is a consequence of the control scheme, not an independent choice.

/// Where to put the camera this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Framing {
    pub eye: [f32; 3],
    pub look_at: [f32; 3],
}

#[derive(Clone, Copy, Debug)]
pub struct RigConfig {
    /// Distance at minimum separation.
    pub min_distance: f32,
    pub max_distance: f32,
    /// Extra distance per unit of separation between the fighters.
    pub distance_per_separation: f32,
    /// Height of the camera above the look target.
    pub height: f32,
    /// Height above the ground that the camera aims at -- roughly chest level,
    /// so characters sit in the middle of the frame rather than the bottom.
    pub look_height: f32,
    /// Fraction of the remaining error closed per tick. Higher is snappier.
    pub smoothing: f32,
    /// Maximum yaw change per second, in radians. Prevents a whip-around when
    /// the fighters cross over.
    pub max_yaw_rate: f32,
}

impl Default for RigConfig {
    fn default() -> Self {
        RigConfig {
            min_distance: 7.5,
            max_distance: 20.0,
            distance_per_separation: 0.62,
            height: 5.0,
            look_height: 1.3,
            smoothing: 0.12,
            max_yaw_rate: 2.2,
        }
    }
}

/// Stateful camera. The state is renderer-local and deliberately *not* part of
/// the simulation snapshot -- a rollback should not rewind the camera.
#[derive(Debug)]
pub struct CameraRig {
    cfg: RigConfig,
    yaw: f32,
    distance: f32,
    focus: [f32; 3],
    initialised: bool,
}

impl CameraRig {
    pub fn new(cfg: RigConfig) -> CameraRig {
        CameraRig {
            cfg,
            yaw: 0.0,
            distance: cfg.min_distance,
            focus: [0.0, 0.0, 0.0],
            initialised: false,
        }
    }

    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    pub fn distance(&self) -> f32 {
        self.distance
    }

    /// Advance the rig and return where to put the camera.
    ///
    /// `dt` is real seconds, so the camera stays frame-rate independent even
    /// though the simulation is fixed-step.
    pub fn update(&mut self, dt: f32, a: [f32; 3], b: [f32; 3]) -> Framing {
        let mid = [
            (a[0] + b[0]) * 0.5,
            self.cfg.look_height,
            (a[2] + b[2]) * 0.5,
        ];

        let axis = [b[0] - a[0], b[2] - a[2]];
        let separation = (axis[0] * axis[0] + axis[1] * axis[1]).sqrt();

        // Sit perpendicular to the line between the fighters, so both are in
        // profile and the space between them reads clearly.
        let target_yaw = if separation > 1e-3 {
            axis[1].atan2(axis[0]) + std::f32::consts::FRAC_PI_2
        } else {
            self.yaw
        };

        let target_distance = (self.cfg.min_distance
            + separation * self.cfg.distance_per_separation)
            .clamp(self.cfg.min_distance, self.cfg.max_distance);

        if !self.initialised {
            self.yaw = target_yaw;
            self.distance = target_distance;
            self.focus = mid;
            self.initialised = true;
        }

        // Take the shorter way round, and never faster than max_yaw_rate.
        // Without both, the camera whips 180 degrees when the fighters swap
        // sides -- which happens constantly.
        let mut delta = wrap_angle(target_yaw - self.yaw);
        if delta.abs() > std::f32::consts::FRAC_PI_2 {
            // The perpendicular has two solutions; prefer the near one.
            delta = wrap_angle(delta + std::f32::consts::PI);
        }
        let smoothing = smoothing_for(self.cfg.smoothing, dt);
        let step =
            (delta * smoothing).clamp(-self.cfg.max_yaw_rate * dt, self.cfg.max_yaw_rate * dt);
        self.yaw = wrap_angle(self.yaw + step);

        self.distance += (target_distance - self.distance) * smoothing;
        for (axis, target) in self.focus.iter_mut().zip(mid.iter()) {
            *axis += (target - *axis) * smoothing;
        }

        // Arena geometry must never get between the camera and the fight.
        // An auto-framing camera cannot solve occlusion by rotating, because
        // the angle it wants is the angle that reads -- so pull in instead.
        let clear = unobstructed_distance(self.focus, self.yaw, self.cfg.height, self.distance);

        Framing {
            eye: [
                self.focus[0] + self.yaw.cos() * clear,
                self.focus[1] + self.cfg.height * (clear / self.distance).clamp(0.35, 1.0),
                self.focus[2] + self.yaw.sin() * clear,
            ],
            look_at: self.focus,
        }
    }
}

/// Longest distance along the camera arm that stays out of the level geometry.
///
/// Marches the segment rather than solving it analytically: the arena is a
/// handful of boxes and this runs once a frame on the render side, where exact
/// determinism does not matter.
fn unobstructed_distance(focus: [f32; 3], yaw: f32, height: f32, want: f32) -> f32 {
    const STEPS: usize = 24;
    const PADDING: f32 = 0.45;
    let mut clear = want;
    for step in 1..=STEPS {
        let t = want * step as f32 / STEPS as f32;
        let p = [
            focus[0] + yaw.cos() * t,
            focus[1] + height * (t / want),
            focus[2] + yaw.sin() * t,
        ];
        if inside_geometry(p, PADDING) {
            clear = (t - want / STEPS as f32).max(2.5);
            break;
        }
    }
    clear
}

fn inside_geometry(p: [f32; 3], pad: f32) -> bool {
    sim::arena::SOLIDS.iter().any(|s| {
        let lo = [
            s.min.x.to_f32_for_render() - pad,
            s.min.y.to_f32_for_render() - pad,
            s.min.z.to_f32_for_render() - pad,
        ];
        let hi = [
            s.max.x.to_f32_for_render() + pad,
            s.max.y.to_f32_for_render() + pad,
            s.max.z.to_f32_for_render() + pad,
        ];
        (0..3).all(|i| p[i] > lo[i] && p[i] < hi[i])
    })
}

/// Frame-rate independent smoothing. A raw `t` per frame converges at different
/// speeds depending on refresh rate, which makes the camera feel different on
/// different monitors for no reason the player can see.
fn smoothing_for(per_tick: f32, dt: f32) -> f32 {
    1.0 - (1.0 - per_tick).powf(dt * crate::TICK_HZ)
}

fn wrap_angle(a: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    let mut x = (a + PI) % TAU;
    if x < 0.0 {
        x += TAU;
    }
    x - PI
}
