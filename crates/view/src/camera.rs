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
//! That second term is the one to keep hold of, because it is what the whole
//! rig runs into. **How far out the aim is falls away as the player looks
//! down**: the crosshair sits where the aim ray meets the ground, which is
//! seven metres ahead at ten degrees down and barely one at forty-five. So the
//! separation the camera is trying to open up is being closed by the aim
//! itself, and past a certain angle no eye on the sphere can open it again --
//! the most any of them can manage is `atan(mark / radius)`. From there the
//! fighter rides up the screen toward the crosshair whatever the rig does,
//! which is exactly the walk the floor zone asks for, arriving early because
//! the geometry ran out rather than because a zone boundary said so.
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

/// How far ahead the point the camera is aimed at is put. Any distance does --
/// it is a direction being expressed as a point, because that is what a look-at
/// transform wants -- but far enough out that single-precision rounding on it
/// cannot wobble the bearing.
const SIGHT: f32 = 100.0;

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
    /// How much of the fighter's own body to take away, 0 to 1.
    ///
    /// Two separate reasons to stop drawing it, and the stronger one wins.
    ///
    /// It goes **translucent as it comes up on the crosshair**, because a body
    /// the player is trying to aim past is worse than no body at all, and the
    /// reticle is the one thing on screen they are holding still on purpose.
    ///
    /// It goes **fully away when the eye gets close**, measured as a plain
    /// distance to the body rather than as a zone. That matters: the eye can
    /// end up against the fighter's back with the aim pointed nowhere near the
    /// sky -- a wall behind them pulls the arm in, and then the whole frame is
    /// the inside of a shoulder. Distance catches that, and a zone never could.
    pub hidden: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct RigConfig {
    /// Height above the fighter's feet that the arm pivots around.
    ///
    /// Not a framing number any more -- the framing is solved from the zones.
    /// This is the point the follow smoothing chases and the point the
    /// occlusion march starts from, and it wants to be inside the body so that
    /// standing on a platform does not read as standing inside one.
    pub look_height: f32,
    /// Fraction of the remaining position error closed per tick.
    pub smoothing: f32,
    /// The player's own distance setting, as a multiple of the tuned sphere.
    ///
    /// The rig has a real distance again -- it is the radius of the sphere the
    /// eye rides -- so "pull the camera back" means what it says. Kept as a
    /// multiplier rather than as metres so that the Oven still owns the number
    /// and this stays a preference on top of it: at the setting's own default
    /// it is one, and the camera is exactly as tuned.
    pub dolly: f32,
    /// Vertical field of view, in radians.
    ///
    /// The rig needs it because the zones are written in *screen fractions*,
    /// and turning a fraction of the screen into an angle is exactly what a
    /// field of view is. It follows that widening the view swings the eye down:
    /// the same fraction of a wider picture is a bigger angle, and the way to
    /// open an angle between the fighter and the crosshair without moving off
    /// the sphere is to come round flatter.
    pub fov: f32,
}

impl Default for RigConfig {
    fn default() -> Self {
        RigConfig {
            look_height: 1.25,
            smoothing: 0.35,
            dolly: 1.0,
            fov: 58.0_f32.to_radians(),
        }
    }
}

/// The distance setting's own default, so that a player who has never touched
/// it gets exactly the tuned camera.
const REFERENCE_DISTANCE: f32 = 10.9;

/// The camera's zones, as tuned.
///
/// Angles in radians, negative below the horizon. Screen positions as fractions
/// from the bottom, so the crosshair is at one half by definition.
///
/// Read fresh from the Oven every frame, so the sliders move the camera while
/// you are watching it. The *values* are knobs; the **shape** -- which zones
/// exist and what each one is trying to achieve -- is the code below and is
/// deliberately not configurable. A waypoint is a design decision.
/// One zone's sphere: where it sits, how big it is, and how far the view is
/// tilted off the line to its centre.
///
/// These three are the whole of what a zone decides. The eye's *position* is
/// not among them -- that comes straight from the mouse -- and neither is the
/// framing, which follows from the tilt without being asked for.
#[derive(Clone, Copy, Debug)]
struct Ball {
    /// Height up the fighter the sphere is centred on. Zero is the feet.
    centre: f32,
    radius: f32,
    /// How far the view is turned up off the line to the centre, in radians.
    ///
    /// **This is what frames the shot, and it is why nothing has to be solved.**
    /// The eye is on a sphere about the centre, so the line to that centre is
    /// the radius the eye is standing on, whichever way round the sphere it has
    /// walked. Turn the view up off that line by a fixed angle and the centre
    /// sits at a fixed place on the screen -- *always*, at every eye position,
    /// for free. A tilt is a screen position expressed as an angle.
    tilt: f32,
}

/// The camera's zones, as tuned.
///
/// Angles in radians, negative below the horizon. Screen positions as fractions
/// from the bottom, so the crosshair is at one half by definition.
///
/// Read fresh from the Oven every frame, so the sliders move the camera while
/// you are watching it. The *values* are knobs; the **shape** -- which zones
/// exist and what each one is trying to achieve -- is the code below and is
/// deliberately not configurable. A waypoint is a design decision.
#[derive(Clone, Copy, Debug)]
pub struct Zones {
    /// Straight down is a quarter turn; the rig stops short of it. Not
    /// squeamishness: at the pole the fighter's own vertical plane stops being
    /// defined and the camera has nothing left to be behind.
    pub down_limit: f32,
    pub up_limit: f32,
    /// Below this the view tilts down toward the fighter's own feet.
    pub floor_from: f32,
    /// Above this the sphere starts moving from the feet to the head.
    pub neutral_to: f32,
    /// Above this the eye is the fighter's own.
    pub head_lock: f32,
    /// How far out the eye rides below the horizon, in metres.
    pub sphere: f32,
    /// What that shrinks to once the eye is the fighter's own.
    ///
    /// Not quite nothing, because a sphere of no radius has no direction to be
    /// behind and the eye would sit exactly inside the skull.
    pub head_sphere: f32,
    /// Where the feet sit through the neutral zone.
    pub feet_neutral: f32,
    /// Where the feet have reached at the bottom of the range.
    pub feet_floor: f32,
    /// How far under the crosshair the top of the head rides at level.
    pub head_gap: f32,
    /// How close the eye may get before the fighter's own body stops being
    /// drawn at all, in metres.
    pub fade_near: f32,
}

impl Zones {
    pub fn tuned() -> Zones {
        use sim::oven::{ViewKnob as V, view};
        let deg = |k| (view(k) as f32).to_radians();
        let pct = |k| view(k) as f32 / 100.0;
        let metres = |k| sim::Fx::from_raw(view(k)).to_f32_for_render();
        Zones {
            down_limit: deg(V::LookDownLimit),
            up_limit: deg(V::LookUpLimit),
            floor_from: deg(V::FloorZoneFrom),
            neutral_to: deg(V::NeutralZoneTo),
            head_lock: deg(V::HeadLockAt),
            sphere: metres(V::Sphere),
            head_sphere: metres(V::HeadSphere),
            feet_neutral: pct(V::FeetNeutral),
            feet_floor: pct(V::FeetFloor),
            head_gap: pct(V::HeadGapLevel),
            fade_near: metres(V::FadeNear),
        }
    }

    /// Where the rig rests: the middle of the neutral zone, so there is room to
    /// steer either way without leaving it.
    pub fn neutral_pitch(&self) -> f32 {
        -(self.floor_from + self.neutral_to) * 0.5
    }

    /// How far the eye has walked into the fighter's own head.
    fn first_person(&self, pitch: f32) -> f32 {
        if pitch <= 0.0 {
            0.0
        } else {
            (pitch / self.head_lock).min(1.0)
        }
    }

    /// The sphere the eye is riding at this aim angle.
    ///
    /// Every zone is a straight interpolation of the same three numbers, and
    /// each one starts where the last one left off, so the whole range is
    /// continuous without anything being blended or clamped to make it so.
    fn ball(&self, pitch: f32, body: f32, fov: f32) -> Ball {
        // A screen position, as the angle the view has to be turned up by to
        // put it there. Through a tangent, because the screen is a flat plane a
        // fixed distance in front of the eye rather than an arc: twice the
        // angle off centre is more than twice the distance up the glass.
        let tilt_for = |at: f32| ((1.0 - 2.0 * at) * (fov * 0.5).tan()).atan();
        let low = tilt_for(self.feet_neutral);
        let level = tilt_for(0.5 - self.head_gap);

        if pitch <= -self.floor_from {
            // **The floor zone.** The view tilts down onto the fighter's own
            // feet, so that at the bottom of the range the crosshair is on them
            // -- the shot that puts a stone underneath you. The eye keeps
            // working around the sphere while it does; that is the mouse, and
            // the mouse never stops moving it.
            let t =
                ((-pitch - self.floor_from) / (self.down_limit - self.floor_from)).clamp(0.0, 1.0);
            Ball {
                centre: 0.0,
                radius: self.sphere,
                tilt: lerp(low, tilt_for(self.feet_floor), t),
            }
        } else if pitch <= -self.neutral_to {
            // **The neutral zone**, where most of a match is spent. Nothing
            // about the sphere changes here at all -- only where the eye is on
            // it -- so the fighter sits at exactly the same spot on screen
            // through the whole band while the camera swings around behind
            // them.
            Ball {
                centre: 0.0,
                radius: self.sphere,
                tilt: low,
            }
        } else if pitch <= 0.0 {
            // **The turn.** The sphere slides up the body from the feet to the
            // head, and the tilt comes with it, until at level the crosshair
            // rides just above the head -- which is what gives a mid-range
            // skillshot something to key off when there is no ground under the
            // aim to read it against.
            let t = ((pitch + self.neutral_to) / self.neutral_to).clamp(0.0, 1.0);
            Ball {
                centre: lerp(0.0, body, t),
                radius: self.sphere,
                tilt: lerp(low, level, t),
            }
        } else {
            // **The handover**, and then the fighter's own eye. The sphere
            // shrinks onto the head and the tilt goes to nothing, which leaves
            // the camera looking straight down the line the crosshair draws.
            // Past the handover none of the three moves again, so a skillshot
            // aimed at the sky follows that line rather than one parallel to
            // it.
            let t = (pitch / self.head_lock).min(1.0);
            Ball {
                centre: body,
                radius: lerp(self.sphere, self.head_sphere, t),
                tilt: lerp(level, 0.0, t),
            }
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
    /// Pull the camera back, or push it in, live.
    ///
    /// Taken as a distance in metres because that is what a player means by it,
    /// and landed as a multiple of the tuned sphere so that the tuning and the
    /// preference do not fight over the same number. Further back is a bigger
    /// sphere: a smaller fighter, and the neutral zone handing over to the
    /// floor zone a little sooner.
    pub fn set_distance(&mut self, distance: f32) {
        self.cfg.dolly = distance.max(0.1) / REFERENCE_DISTANCE;
    }

    /// Track the player's field of view, which the framing is measured against.
    pub fn set_fov(&mut self, radians: f32) {
        self.cfg.fov = radians;
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
    /// The camera is pointed straight down the look axis -- there is no aim
    /// point to hand it any more. The eye is placed by the same two angles the
    /// aim is made of, so screen centre *is* the look direction, and the
    /// crosshair sits exactly in the middle of the screen by construction
    /// rather than by correction.
    ///
    /// `dt` is real seconds, so the camera stays frame-rate independent even
    /// though the simulation is fixed-step.
    pub fn update(&mut self, dt: f32, player: [f32; 3], yaw: f32, pitch: f32) -> Framing {
        self.update_around(dt, player, yaw, pitch, Surroundings::default())
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

        let zones = Zones::tuned();
        let pitch = pitch.clamp(-zones.down_limit, zones.up_limit);
        let sky = zones.first_person(pitch);

        // Everything below happens in the fighter's own vertical plane: the one
        // containing them and the way they are looking. In that plane the
        // sphere is a circle, and the eye is one angle around it.
        let feet = self.focus[1] - self.cfg.look_height;
        let along = [yaw.cos(), yaw.sin()];
        let body = crate::fx(sim::tuning::body_height());
        let ball = zones.ball(pitch, body, self.cfg.fov);
        let radius = (ball.radius * self.cfg.dolly).max(0.02);

        // **The whole placement, and there is nothing to solve.**
        //
        // The camera looks where the mouse looks: the view runs out at `pitch`,
        // by definition, because that is what the player is pointing at. It
        // also runs `tilt` above the line to the sphere's centre, because that
        // is what puts the centre at its mark on the screen. Those two together
        // say where on the sphere the eye has to be standing, and it is a
        // subtraction:
        //
        // ```text
        // elevation = tilt - pitch
        // ```
        //
        // One degree of mouse is one degree around the sphere, always, in every
        // zone -- which is the point. Nothing here can saturate, run out of
        // room, or stop answering the aim, because nothing here is being asked
        // to satisfy a condition. The framing is a consequence of the tilt and
        // arrives for free.
        let elevation = ball.tilt - pitch;
        let back = radius * elevation.cos();
        let up = ball.centre + radius * elevation.sin();

        let mut offset = [
            -along[0] * back,
            feet + up - self.focus[1],
            -along[1] * back,
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

        // The fighter's own body, taken away for either of the two reasons it
        // has to be. Measured on the eye the camera actually ended up at, so
        // that an arm pulled in by a wall counts the same as one that walked
        // into the head on purpose.
        let middle = [self.focus[0], feet + body * 0.5, self.focus[2]];
        let reach = (0..3)
            .map(|i| (eye[i] - middle[i]).powi(2))
            .sum::<f32>()
            .sqrt();
        // Gone by the time it is this close, and back in full not long after,
        // because the distance it is guarding against is a near-plane one: a
        // body half-faded at ordinary range is a bug, not a softer version of
        // the same idea.
        let crowding = 1.0 - ((reach - zones.fade_near) / (zones.fade_near * 0.5)).clamp(0.0, 1.0);
        // And how far the top of the head has come up toward the reticle. The
        // head rather than the whole body, because the body only ever reaches
        // the middle of the screen head first.
        let to_head = [back, body - up];
        let under = pitch - to_head[1].atan2(to_head[0].max(1e-4));
        let gap = 0.5 - (0.5 - under.tan() / (2.0 * (self.cfg.fov * 0.5).tan()));
        let covering = 1.0 - (gap / (2.0 * zones.head_gap).max(0.01)).clamp(0.0, 1.0);
        // At the target, so screen centre *is* the target and the reticle never
        // has to move. Degenerate only if the two coincide, which cannot happen
        // while the eye is behind the fighter and the target is in front.
        // Straight down the line the mouse is pointing, which is the whole of
        // what "the crosshair is the aim" means once the eye is placed by the
        // same two angles. Screen centre is the look direction; the fighter
        // lands where the tilt puts them.
        let ahead = [along[0] * pitch.cos(), pitch.sin(), along[1] * pitch.cos()];
        Framing {
            eye,
            look_at: [
                eye[0] + ahead[0] * SIGHT,
                eye[1] + ahead[1] * SIGHT,
                eye[2] + ahead[2] * SIGHT,
            ],
            first_person: sky,
            hidden: crowding.max(covering),
        }
    }
}

fn lerp(from: f32, to: f32, at: f32) -> f32 {
    from + (to - from) * at
}

// ---------------------------------------------------------------------------
// Solving the framing
// ---------------------------------------------------------------------------

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
