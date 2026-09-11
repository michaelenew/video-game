//! Third-person follow camera.
//!
//! The camera hovers behind the fighter and the player owns it. Aim is the
//! camera: where you look is where you are pointed, and where you are pointed
//! is where your attacks go.
//!
//! This replaced an auto-framing rig that sat perpendicular to the line between
//! the two fighters and kept both in profile. That rig was a good idea under a
//! control scheme where nothing needed mouse-look -- but it was a *consequence*
//! of that scheme, not an independent choice, and once attacks are aimed the
//! premise is gone. It also only ever made sense for 1v1; framing "both
//! fighters" means nothing in coop against four monsters.
//!
//! Two rules keep it honest:
//!
//! **Yaw and pitch are never smoothed.** They are the mouse, and a smoothed
//! mouse feels broken in a way players cannot name but immediately dislike.
//! Only the focus *position* is smoothed, so the camera glides over the
//! character's steps instead of jittering with them.
//!
//! **The camera is not in the snapshot.** Aim reaches the simulation as input
//! (see `sim::Input::aim`), so gameplay agrees across peers without the camera
//! itself being rolled back. What is drawn stays renderer-local.

/// Arena floor height, and how far above it the eye may come.
const GROUND: f32 = 0.0;
const FLOOR_CLEARANCE: f32 = 0.3;

/// Where to put the camera this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Framing {
    pub eye: [f32; 3],
    pub look_at: [f32; 3],
}

#[derive(Clone, Copy, Debug)]
pub struct RigConfig {
    /// How far behind the fighter the camera sits.
    pub distance: f32,
    /// Height above the fighter's feet that the camera arm pivots around.
    pub look_height: f32,
    /// How far the eye is lifted above the pivot.
    ///
    /// Without this the camera sits at head height and the fighter's own body
    /// blots out whatever is directly ahead -- which, in a game about facing
    /// someone, is the only thing you needed to see.
    pub eye_lift: f32,
    /// How far to the side the eye sits.
    ///
    /// Not decoration. At melee range an opponent stands directly behind your
    /// own fighter from a centred camera, and raising the eye does not fix it:
    /// a body is wider than a sightline. This is the whole reason
    /// over-the-shoulder cameras exist in games where you face someone.
    ///
    /// The eye moves; the aim point does not. The centre of the screen stays on
    /// the look axis, so the offset costs nothing in aiming precision -- it
    /// only slides the character out of the way.
    pub shoulder: f32,
    /// How far ahead of the fighter the camera aims.
    ///
    /// Moves the character down and out of the middle of the frame, and puts
    /// the centre of the screen roughly where your attacks land. The screen
    /// centre stays on the aim axis, so this costs nothing in precision.
    pub look_ahead: f32,
    /// Fraction of the remaining position error closed per tick.
    pub smoothing: f32,
    /// How far up and down the camera may be pitched, in radians.
    pub pitch_limit: f32,
}

impl Default for RigConfig {
    fn default() -> Self {
        RigConfig {
            distance: 6.2,
            look_height: 1.25,
            eye_lift: 2.7,
            shoulder: 1.15,
            look_ahead: 3.5,
            smoothing: 0.35,
            pitch_limit: 1.0,
        }
    }
}

/// Stateful camera. The only state is the smoothed focus point; aim is passed
/// in fresh every frame because it belongs to the player, not to the rig.
#[derive(Debug)]
pub struct CameraRig {
    cfg: RigConfig,
    focus: [f32; 3],
    initialised: bool,
}

impl CameraRig {
    pub fn new(cfg: RigConfig) -> CameraRig {
        CameraRig {
            cfg,
            focus: [0.0, 0.0, 0.0],
            initialised: false,
        }
    }

    /// Advance the rig and return where to put the camera.
    ///
    /// `yaw` and `pitch` are radians; `yaw` uses the same convention as the
    /// simulation -- zero looks down positive X, increasing swings toward
    /// positive Z -- so the direction the player aims and the direction their
    /// attacks travel are the same number.
    ///
    /// `dt` is real seconds, so the camera stays frame-rate independent even
    /// though the simulation is fixed-step.
    pub fn update(&mut self, dt: f32, player: [f32; 3], yaw: f32, pitch: f32) -> Framing {
        let target = [player[0], player[1] + self.cfg.look_height, player[2]];

        if !self.initialised {
            self.focus = target;
            self.initialised = true;
        }
        let smoothing = smoothing_for(self.cfg.smoothing, dt);
        for (axis, want) in self.focus.iter_mut().zip(target.iter()) {
            *axis += (want - *axis) * smoothing;
        }

        let pitch = pitch.clamp(-self.cfg.pitch_limit, self.cfg.pitch_limit);
        // The direction the player is looking. The camera sits back along it.
        let dir = [
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        ];

        // Where the eye wants to be, relative to the focus: back along the look
        // direction, then lifted.
        let mut offset = [
            -dir[0] * self.cfg.distance,
            -dir[1] * self.cfg.distance + self.cfg.eye_lift,
            -dir[2] * self.cfg.distance,
        ];

        // The floor is not in `SOLIDS` -- it is a plane the simulation handles
        // separately -- so it has to be handled by hand, and *how* matters.
        //
        // Look up far enough and the arm wants to swing below the ground. The
        // wrong answer, which this used to do, is to shorten the arm: that
        // hauls the camera in toward the fighter's head while its height never
        // changes, so the rig reads as a pole of fixed length with the camera
        // sliding down it. The right answer is to let the camera settle onto
        // the ground and ride along it, keeping its distance. It still closes
        // on the fighter as you keep looking up, but only once it is actually
        // on the ground and has nowhere left to go.
        let lowest = GROUND + FLOOR_CLEARANCE;
        offset[1] = offset[1].max(lowest - self.focus[1]);

        // Rightward in the horizontal plane, matching the simulation's own
        // convention for strafing (`sim::state::move_dir`). Folded into the
        // offset *before* the geometry check, not after: a camera slid sideways
        // after being cleared has not been cleared. This was wrong from the
        // start and only stayed hidden because the broken floor clamp kept the
        // arm too short to reach a platform in the first place.
        let flat_dir = (dir[0] * dir[0] + dir[2] * dir[2]).sqrt().max(1e-4);
        offset[0] += -dir[2] / flat_dir * self.cfg.shoulder;
        offset[2] += dir[0] / flat_dir * self.cfg.shoulder;

        // Arena geometry must never get between the camera and the fighter.
        // Pull the arm in rather than swinging it: the angle is the player's,
        // and a camera that takes the mouse away to dodge a wall is worse than
        // one that gets close to the character's back for a moment.
        //
        // Measured along the *final* offset, after the floor has had its say --
        // the ground clamp moves the eye off the arm, so checking the arm would
        // be checking a line the camera is no longer on.
        let clear = unobstructed_fraction(self.focus, offset);
        for axis in offset.iter_mut() {
            *axis *= clear;
        }

        // Aim at a point ahead of the fighter along the look axis, not at the
        // fighter. Flat, so pitch tilts the camera without dragging the aim
        // point into the floor.
        let ahead = self.cfg.look_ahead;
        Framing {
            eye: [
                self.focus[0] + offset[0],
                self.focus[1] + offset[1],
                self.focus[2] + offset[2],
            ],
            look_at: [
                self.focus[0] + dir[0] / flat_dir * ahead,
                self.focus[1],
                self.focus[2] + dir[2] / flat_dir * ahead,
            ],
        }
    }
}

/// How much of the camera's offset from the focus stays out of level geometry,
/// as a fraction between `MINIMUM` and one.
///
/// Marches the segment rather than solving it analytically: the arena is a
/// handful of boxes and this runs once a frame on the render side, where exact
/// determinism does not matter.
fn unobstructed_fraction(focus: [f32; 3], offset: [f32; 3]) -> f32 {
    const STEPS: usize = 24;
    const PADDING: f32 = 0.45;
    // Deliberately tiny. An arm that refuses to shorten past a comfortable
    // distance will happily hold the camera *inside* a wall when the fighter
    // stands close to one, which is far worse than going briefly first-person.
    // The cost is that the character can clip through the near plane when
    // backed against geometry; fading them out is the usual answer and is not
    // worth building yet.
    const MINIMUM: f32 = 0.04;
    for step in 1..=STEPS {
        let t = step as f32 / STEPS as f32;
        let p = [
            focus[0] + offset[0] * t,
            focus[1] + offset[1] * t,
            focus[2] + offset[2] * t,
        ];
        if inside_geometry(p, PADDING) {
            return (t - 1.0 / STEPS as f32).max(MINIMUM);
        }
    }
    1.0
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
