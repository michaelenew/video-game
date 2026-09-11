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
//! Three rules keep it honest:
//!
//! **The screen looks where the mouse points.** Screen centre is simply the
//! look direction. It used to be a point pinned flat at the fighter's own
//! height a fixed distance ahead, so pitch swung the eye around while the
//! middle of the screen stayed put -- which meant you could not aim at the
//! ground near your feet and could not push the reticle out either. The only
//! thing the mouse changed was how obliquely you saw the same spot.
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
    /// How far the rig has climbed into the fighter's own head, 0 to 1.
    ///
    /// Past about half, the eye is inside the body and the renderer has to stop
    /// drawing it -- see `sky_start`. Reported as a fraction rather than a flag
    /// so the model can fade instead of popping.
    pub first_person: f32,
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
    /// Fraction of the remaining position error closed per tick.
    pub smoothing: f32,
    /// How far down the camera may be pitched, in radians.
    pub pitch_down: f32,
    /// How far up, in radians. Near vertical: verticality is part of the
    /// positioning game, so looking at what is above you cannot be a thing the
    /// camera refuses to do.
    pub pitch_up: f32,
    /// Pitch at which the rig starts climbing into the fighter's head.
    ///
    /// Below it, looking up walks the camera down toward the ground behind the
    /// fighter, which is the ordinary third-person answer and reads well for
    /// the first forty degrees or so. Above it that answer runs out: the arm is
    /// on the floor, the fighter's body is between you and the sky, and every
    /// bump in the terrain shoves the view. So past this angle the camera
    /// climbs to the fighter's eyes and the body stops being drawn -- you are
    /// simply panning the sky, which is what you were trying to do.
    pub sky_start: f32,
    /// How far back the camera sits once it is looking straight down.
    ///
    /// Short. Looking down is looking at the ground *near you*, and a seven
    /// metre arm puts the centre of the screen behind your own heels.
    pub overhead_distance: f32,
    /// Eye lift at full downward pitch. See `overhead_distance`.
    pub overhead_lift: f32,
}

impl Default for RigConfig {
    fn default() -> Self {
        RigConfig {
            distance: 7.0,
            look_height: 1.25,
            eye_lift: 2.7,
            shoulder: 1.15,
            smoothing: 0.35,
            pitch_down: 1.15,
            pitch_up: 1.45,
            sky_start: 0.8,
            overhead_distance: 2.0,
            overhead_lift: 1.5,
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
    /// Change how far back the camera sits, live.
    ///
    /// Distance is a setting rather than a constant because it is the number
    /// most often reached for when a camera feels wrong, and a value you have
    /// to rebuild to try is a value that gets tried once.
    pub fn set_distance(&mut self, distance: f32) {
        self.cfg.distance = distance;
    }

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

        let pitch = pitch.clamp(-self.cfg.pitch_down, self.cfg.pitch_up);
        // The direction the player is looking. The camera sits back along it,
        // and -- the part that used to be missing -- **the screen looks along
        // it**. The aim point used to be pinned flat at the fighter's own
        // height a fixed distance ahead, so pitch swung the eye around but the
        // middle of the screen stayed put: you could not aim at the ground near
        // your feet, and you could not push the reticle out either, because the
        // only thing your mouse changed was how obliquely you saw the same
        // spot. Screen centre is now simply where you are looking.
        let dir = [
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        ];
        let flat_dir = (dir[0] * dir[0] + dir[2] * dir[2]).sqrt().max(1e-4);

        // Two blends, one for each end of the pitch range.
        //
        // Looking **down** means looking at the ground near yourself, so the
        // arm shortens and the eye drops toward the fighter's own head. Left
        // long, a steep look down puts the middle of the screen behind your
        // heels -- the camera is seven metres back, so the ray reaches the
        // floor before it reaches you.
        //
        // Looking **up** past `sky_start` means the third-person answer has run
        // out; see the field's own note.
        let down = (-pitch / self.cfg.pitch_down).clamp(0.0, 1.0);
        let sky = smoothstep(self.cfg.sky_start, self.cfg.pitch_up, pitch);

        let distance = lerp(self.cfg.distance, self.cfg.overhead_distance, down) * (1.0 - sky);
        let eye_lift = lerp(self.cfg.eye_lift, self.cfg.overhead_lift, down) * (1.0 - sky);
        // The shoulder offset exists to slide the body out of the sightline.
        // Straight down it does not do that, it just swings the world; and in
        // the sky there is no body left to slide.
        let shoulder = self.cfg.shoulder * (1.0 - down) * (1.0 - sky);

        let mut offset = [
            -dir[0] * distance,
            -dir[1] * distance + eye_lift,
            -dir[2] * distance,
        ];

        // The floor is not in `SOLIDS` -- it is a plane the simulation handles
        // separately -- so it has to be handled by hand, and *how* matters.
        //
        // Look up far enough and the arm wants to swing below the ground. The
        // wrong answer, which this used to do, is to shorten the arm: that
        // hauls the camera in toward the fighter's head while its height never
        // changes, so the rig reads as a pole of fixed length with the camera
        // sliding down it. The right answer is to let the camera settle onto
        // the ground and ride along it, keeping its distance.
        //
        // It stops applying once the rig is climbing into the head: up there
        // the eye is at the fighter's eyes, which is above the floor by
        // definition, and a clamp that still fired would be fighting the climb.
        let lowest = GROUND + FLOOR_CLEARANCE;
        offset[1] = offset[1].max((lowest - self.focus[1]) * (1.0 - sky));

        // Rightward in the horizontal plane, matching the simulation's own
        // convention for strafing (`sim::state::move_dir`). Folded into the
        // offset *before* the geometry check, not after: a camera slid sideways
        // after being cleared has not been cleared.
        offset[0] += -dir[2] / flat_dir * shoulder;
        offset[2] += dir[0] / flat_dir * shoulder;

        // Arena geometry must never get between the camera and the fighter.
        // Pull the arm in rather than swinging it: the angle is the player's,
        // and a camera that takes the mouse away to dodge a wall is worse than
        // one that gets close to the character's back for a moment.
        //
        // Faded out with the climb for the same reason as the floor: an arm
        // that is already inside the fighter has nothing left to be blocked by,
        // and terrain shoving the view around is exactly what makes panning the
        // sky unpleasant in third person.
        let clear = lerp(unobstructed_fraction(self.focus, offset), 1.0, sky);
        for axis in offset.iter_mut() {
            *axis *= clear;
        }

        let eye = [
            self.focus[0] + offset[0],
            self.focus[1] + offset[1],
            self.focus[2] + offset[2],
        ];
        // Straight out along the look direction. Any distance points the camera
        // the same way; this one is far enough that floating-point noise in the
        // eye position cannot wobble the aim.
        const FAR: f32 = 64.0;
        Framing {
            eye,
            look_at: [
                eye[0] + dir[0] * FAR,
                eye[1] + dir[1] * FAR,
                eye[2] + dir[2] * FAR,
            ],
            first_person: sky,
        }
    }
}

fn lerp(from: f32, to: f32, at: f32) -> f32 {
    from + (to - from) * at
}

/// Zero below `lo`, one above `hi`, eased between.
///
/// Eased rather than linear because this blend swaps the whole rig over: a
/// linear handover makes the camera visibly change its mind at both ends of the
/// range, and the ends are where the player is holding the mouse still.
fn smoothstep(lo: f32, hi: f32, at: f32) -> f32 {
    let t = ((at - lo) / (hi - lo).max(1e-4)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
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
