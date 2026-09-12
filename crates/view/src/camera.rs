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
//! Four rules keep it honest:
//!
//! **The camera points at the thing you are aiming at**, which `sim::aim` has
//! already solved from the fighter's own cast origin. Not along the look axis:
//! *at the point*. That is what makes the middle of the screen the place an
//! area ability lands, and it is why **the crosshair is always exactly at the
//! centre of the screen** — a reticle that wandered would read as the aim
//! slipping out of the player's hands, and the reticle is the one thing on
//! screen they are deliberately holding still.
//!
//! The eye is therefore free to sit wherever frames the fight best, which is
//! **directly behind the fighter and well above their head**. Directly behind
//! matters: an eye slid to one shoulder turns the whole view sideways once the
//! camera points at the target, so `W` stops walking up the screen. Kept on the
//! centre line, the only parallax left is vertical, and it costs a few degrees
//! of pitch and nothing at all of bearing.
//!
//! Well above their head is what puts the fighter in the **lower part of the
//! frame** rather than sitting on the reticle. Both are on the ground and the
//! fighter is nearer, so a higher eye separates them: the gap between the two
//! grows with eye height and with how far out the aim is. At chest height they
//! coincide, which is what the first attempt at this looked like and why the
//! view felt cramped.
//!
//! An earlier pass tried the other arrangement -- orbit the cast origin itself,
//! so the eye sits *on* the line and the parallax is zero. It works, and it
//! looks terrible: the cast origin is chest height, so the camera ends up at
//! chest height, the horizon climbs to the top of the frame and you cannot see
//! the arena you are fighting in. Zero parallax is not worth a view from a
//! fighter's sternum.
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
//! **The camera is not in the snapshot.** Both angles reach the simulation as
//! input (see `sim::Input::aim` and `pitch`), so gameplay agrees across peers
//! without the camera itself being rolled back. What is drawn stays
//! renderer-local.
//!
//! **The camera points at the thing you are aiming at.** Not along the look
//! axis -- *at the point*, which `sim::aim` has already solved. Those would be
//! the same direction only if the eye sat exactly on the ability's line, and it
//! does not: it is lifted well above the fighter's head, because that is what
//! puts the fighter in the lower part of the frame instead of on the reticle.
//!
//! Pointing at the target absorbs that lift into the *view* rather than into
//! the reticle. **The crosshair is always exactly at screen centre**, because a
//! reticle that moved would read as the aim slipping out of the player's hands
//! -- and the reticle is the one thing in the frame they are holding still on
//! purpose. What moves instead is the world, by the parallax angle, and only as
//! the target's distance changes.

/// Arena floor height, and how far above it the eye may come.
const GROUND: f32 = 0.0;
const FLOOR_CLEARANCE: f32 = 0.3;

/// Where to put the camera this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Framing {
    pub eye: [f32; 3],
    /// The point at the centre of the screen: what the player is aiming at.
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
    /// Height of the point the camera orbits, above the fighter's feet.
    ///
    /// Pure framing now. It used to be the number that decided where the middle
    /// of the screen met the ground -- `h / tan(pitch)`, with the arm cancelling
    /// out -- which made it the most load-bearing number in the rig. That job
    /// belongs to `sim::aim` and to `sim::tuning::cast_height`, which is a
    /// different height and rightly so: a camera is not where your hands are.
    ///
    /// What is left is what it looks like, and it is the number that decides
    /// **how low in the frame the fighter sits**. The reticle is on the aim
    /// point and the fighter is nearer than it, both on the ground, so a higher
    /// eye pushes the near one further down the screen. At chest height the two
    /// coincide and the fighter stands on the crosshair.
    ///
    /// There is no sideways counterpart. An over-the-shoulder slide would turn
    /// the view once the camera points at the target, and a view that is turned
    /// is a view `W` no longer walks up.
    pub orbit_lift: f32,
    /// Fraction of the remaining position error closed per tick.
    pub smoothing: f32,
    /// How far down the camera may be pitched, in radians.
    pub pitch_down: f32,
    /// How far up, in radians. Near vertical: verticality is part of the
    /// positioning game, so looking at what is above you cannot be a thing the
    /// camera refuses to do.
    pub pitch_up: f32,
    /// Where the camera rests: a little below the horizon.
    ///
    /// Level is the wrong neutral for a game played on the ground. Resting a
    /// few degrees down puts the mark out in front of the fighter where the
    /// fight is, and leaves the whole upward range for the verticality without
    /// spending any of it getting back to level.
    ///
    /// Shallower than it was, and by exactly as much as the aim's own origin
    /// dropped. The mark lands `cast_height / tan(pitch)` ahead -- chest height
    /// now rather than a point above the fighter's head -- so the resting aim
    /// would have hauled in from ten metres to five if this had not come with
    /// it. Eight metres at rest, which is about the distance two fighters start
    /// apart.
    pub neutral_pitch: f32,
    /// Pitch at which the rig starts climbing into the fighter's head, and the
    /// pitch by which it has arrived.
    ///
    /// **The horizon, and half a radian above it.** Below the horizon the rig
    /// is third person and the fighter sits low in the frame. Aim above it and
    /// the camera comes in quickly: the fighter rises toward the middle of the
    /// screen and fades out as it goes, until at `sky_full` you are simply
    /// panning the sky from behind their eyes -- which is what you were trying
    /// to do, and the only way to do it without your own head in the way.
    ///
    /// The handover used to start forty degrees up and finish at the pitch
    /// limit. That left a wide band where the arm was dragging along the floor
    /// behind the fighter and every bump in the terrain shoved the view.
    pub sky_start: f32,
    pub sky_full: f32,
}

impl Default for RigConfig {
    fn default() -> Self {
        RigConfig {
            distance: 10.9,
            look_height: 1.25,
            orbit_lift: 4.0,
            smoothing: 0.35,
            neutral_pitch: 0.1,
            pitch_down: 1.15,
            pitch_up: 1.45,
            sky_start: 0.0,
            sky_full: 0.5,
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
    /// `aim_at` is the point the player is aiming at, from `sim::aim`. The
    /// camera is pointed at it rather than along the raw look axis, which is
    /// what keeps the crosshair exactly at screen centre however far the eye
    /// has been slid off that axis.
    ///
    /// `dt` is real seconds, so the camera stays frame-rate independent even
    /// though the simulation is fixed-step.
    pub fn update(
        &mut self,
        dt: f32,
        player: [f32; 3],
        yaw: f32,
        pitch: f32,
        aim_at: [f32; 3],
    ) -> Framing {
        self.update_around(dt, player, yaw, pitch, aim_at, Surroundings::default())
    }

    /// The same, with a creature in the arena.
    ///
    /// It is handled by the two rules the rig already has rather than by a
    /// third. Beside it, the animal is **geometry**: the arm marches back and
    /// stops short, the same as it does for a wall. Standing on it, the animal
    /// is a **surface**: the eye rests on its back and rides along, the same as
    /// it does on the floor.
    ///
    /// Which of the two applies is not a judgement call, it is whether the
    /// fighter is on it -- and getting that wrong is very visible. Treating it
    /// as geometry while riding jams the camera against the rider's back,
    /// because an arm pointing backwards from someone standing on an animal
    /// goes straight into the animal. Treating it as a surface while beside it
    /// would let the camera sit inside its ribs.
    pub fn update_around(
        &mut self,
        dt: f32,
        player: [f32; 3],
        yaw: f32,
        pitch: f32,
        aim_at: [f32; 3],
        around: Surroundings<'_>,
    ) -> Framing {
        let beast = around.beast;
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

        // Looking **up** past `sky_start` means the third-person answer has run
        // out; see the field's own note.
        let sky = smoothstep(self.cfg.sky_start, self.cfg.sky_full, pitch);

        // Neither the arm nor the orbit changes with pitch. The arm is the
        // player's sense of how much of the fight they can see, and taking it
        // away as they look down -- which an early version did -- trades the
        // view for nothing.
        //
        // The orbit used to drop as you looked down, which was how the aim was
        // brought in close: the mark landed `orbit / tan(pitch)` ahead. The aim
        // does not come from here any more, so the orbit is free to stay put
        // and simply frame the fight. Looking down still walks the mark to the
        // fighter's feet, because `sim::aim` traces from their chest.
        let distance = self.cfg.distance * (1.0 - sky);
        let orbit = self.cfg.orbit_lift * (1.0 - sky);

        let mut offset = [
            -dir[0] * distance,
            -dir[1] * distance + orbit,
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
        let underfoot = beast.map_or(GROUND, |b| {
            let eye = sim::V3::new(
                fx_of(self.focus[0] + offset[0]),
                sim::Fx::ZERO,
                fx_of(self.focus[2] + offset[2]),
            );
            b.top_under(eye).to_f32_for_render().max(GROUND)
        });
        let lowest = underfoot + FLOOR_CLEARANCE;
        offset[1] = offset[1].max((lowest - self.focus[1]) * (1.0 - sky));

        // Arena geometry must never get between the camera and the fighter.
        // Pull the arm in rather than swinging it: the angle is the player's,
        // and a camera that takes the mouse away to dodge a wall is worse than
        // one that gets close to the character's back for a moment.
        //
        // Faded out with the climb for the same reason as the floor: an arm
        // that is already inside the fighter has nothing left to be blocked by,
        // and terrain shoving the view around is exactly what makes panning the
        // sky unpleasant in third person.
        // A creature you are standing on is not in your way -- and neither is
        // one you are standing *under*. The second follows the same reasoning
        // as the tiny minimum arm length: an arm that begins inside something
        // has nothing left to be blocked by, and clamping it anyway points the
        // camera at the back of the fighter's head while a dinosaur walks over
        // them, which is the one moment they most need to see.
        let inside_it = beast.is_some_and(|b| {
            b.contains(
                sim::V3::new(
                    fx_of(self.focus[0]),
                    fx_of(self.focus[1]),
                    fx_of(self.focus[2]),
                ),
                fx_of(PADDING),
            )
        });
        let blocker = if around.aboard || inside_it {
            None
        } else {
            beast
        };
        let clear = lerp(unobstructed_fraction(self.focus, offset, blocker), 1.0, sky);
        for axis in offset.iter_mut() {
            *axis *= clear;
        }

        let eye = [
            self.focus[0] + offset[0],
            self.focus[1] + offset[1],
            self.focus[2] + offset[2],
        ];
        // At the target, so screen centre *is* the target and the reticle never
        // has to move. Degenerate only if the two coincide, which cannot happen
        // while the eye is behind the fighter and the target is in front.
        Framing {
            eye,
            look_at: aim_at,
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
/// How far outside a solid the camera is held. Shared with the rule that says
/// a solid you are already inside cannot block you, so the two cannot disagree
/// about where "inside" starts.
const PADDING: f32 = 0.45;

fn unobstructed_fraction(focus: [f32; 3], offset: [f32; 3], beast: Option<&sim::Monster>) -> f32 {
    const STEPS: usize = 24;
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
        let blocked = inside_geometry(p, PADDING)
            || beast.is_some_and(|b| {
                b.contains(
                    sim::V3::new(fx_of(p[0]), fx_of(p[1]), fx_of(p[2])),
                    fx_of(PADDING),
                )
            });
        if blocked {
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

/// What else is in the arena this frame.
///
/// A struct rather than two arguments because the pair is one fact -- there is
/// an animal, and whether you are on it changes what it is to the camera -- and
/// a bare `bool` at a call site says nothing about which way round it goes.
#[derive(Clone, Copy, Default)]
pub struct Surroundings<'a> {
    pub beast: Option<&'a sim::Monster>,
    /// The fighter the camera is following is standing on it.
    pub aboard: bool,
}

/// RENDER-ONLY. Metres to the simulation's fixed point, for asking the
/// creature about a position the renderer computed. Nothing here feeds back
/// into a simulation -- the camera is not in the snapshot.
fn fx_of(v: f32) -> sim::Fx {
    sim::Fx::from_raw((v * crate::FX) as i32)
}
