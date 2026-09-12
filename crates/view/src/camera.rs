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
#[derive(Clone, Copy, Debug)]
pub struct Zones {
    /// Straight down is a quarter turn; the rig stops short of it. Not
    /// squeamishness: at the pole the fighter's own vertical plane stops being
    /// defined and the camera has nothing left to be behind.
    pub down_limit: f32,
    pub up_limit: f32,
    /// Below this the fighter walks up the screen toward the crosshair.
    pub floor_from: f32,
    /// Above this the rig starts turning its attention from feet to head.
    pub neutral_to: f32,
    /// Above this the eye is the fighter's own.
    pub head_lock: f32,
    /// How far from the fighter's feet the eye rides, in metres.
    ///
    /// The camera has one place to be: somewhere on a sphere of this radius
    /// centred on their feet. Everything else the rig decides is *where* on
    /// that sphere, which is a single angle. Fixing the radius is what stops
    /// the camera dollying in and out while the player is only steering.
    pub sphere: f32,
    /// Where the feet sit through the neutral zone.
    pub feet_neutral: f32,
    /// Where the feet have reached at the bottom of the range.
    pub feet_floor: f32,
    /// How far under the crosshair the top of the head rides at level.
    pub head_gap: f32,
    /// Bounds on how high around the sphere the eye may climb.
    ///
    /// These carry most of the feel now, because on a fixed sphere the framing
    /// runs out of room and leaves the eye resting against one of them for much
    /// of the look-down range.
    ///
    /// The **ceiling** is how far the camera is willing to swing overhead in
    /// order to keep the fighter down at the bottom of the frame. It is a
    /// trade, not a safety limit: higher holds them low further down the range
    /// and gets more top-down about it; lower keeps a normal third-person view
    /// and hands over to the fighter walking up the screen sooner.
    ///
    /// The **floor** is how far back down the rig may come at the bottom of the
    /// look-down range, and it wants room rather than safety. Past the neutral
    /// zone the framing asks for two things at once -- the eye keeps working
    /// around the sphere *and* the view pans the fighter up toward the
    /// crosshair -- and the second is what brings the eye back down off the top
    /// of the sphere. Clamped tight the eye stops moving, which reads as the
    /// camera giving up halfway through a turn the player can feel they are
    /// still making. Only the last degree or so wants stopping, where "feet
    /// exactly on the crosshair" would ask for an eye in line with the feet.
    pub min_elevation: f32,
    pub max_elevation: f32,
}

impl Zones {
    pub fn tuned() -> Zones {
        use sim::oven::{ViewKnob as V, view};
        let deg = |k| (view(k) as f32).to_radians();
        let pct = |k| view(k) as f32 / 100.0;
        Zones {
            down_limit: deg(V::LookDownLimit),
            up_limit: deg(V::LookUpLimit),
            floor_from: deg(V::FloorZoneFrom),
            neutral_to: deg(V::NeutralZoneTo),
            head_lock: deg(V::HeadLockAt),
            sphere: sim::Fx::from_raw(view(V::Sphere)).to_f32_for_render(),
            feet_neutral: pct(V::FeetNeutral),
            feet_floor: pct(V::FeetFloor),
            head_gap: pct(V::HeadGapLevel),
            min_elevation: deg(V::MinElevation),
            max_elevation: deg(V::MaxElevation),
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

    /// What the framing asks for at this aim angle.
    ///
    /// One point on the fighter, and where on the screen it has to land. That
    /// is the whole of it: the eye is already pinned to the sphere, so a single
    /// condition is exactly enough to place it, and asking for a second one
    /// would only make the pair unsolvable.
    fn want(&self, pitch: f32, body: f32) -> Want {
        if pitch <= -self.floor_from {
            // **The floor zone.** Looking further down walks the fighter up the
            // screen, until at the limit the camera is looking at their feet.
            let t =
                ((-pitch - self.floor_from) / (self.down_limit - self.floor_from)).clamp(0.0, 1.0);
            Want {
                on_body: 0.0,
                at: lerp(self.feet_neutral, self.feet_floor, t),
                least_elevation: self.min_elevation,
                settle: t,
            }
        } else if pitch <= -self.neutral_to {
            // **The neutral zone**, where most of a match is spent. The fighter
            // sits low and the same size throughout.
            Want {
                on_body: 0.0,
                at: self.feet_neutral,
                least_elevation: 0.0,
                settle: 0.0,
            }
        } else {
            // **The turn.** Attention moves from the feet to the head, and by
            // level the crosshair rides just above the head -- which is what
            // gives a mid-range skillshot something to key off when there is no
            // ground under the aim to read it against.
            let t = ((pitch + self.neutral_to) / self.neutral_to).clamp(0.0, 1.0);
            Want {
                on_body: lerp(0.0, body, t),
                at: lerp(self.feet_neutral, 0.5 - self.head_gap, t),
                least_elevation: 0.0,
                settle: 0.0,
            }
        }
    }
}

/// Where on the fighter the rig is placing, and where that has to land.
#[derive(Clone, Copy, Debug)]
struct Want {
    /// Height up the body of the point being placed. Zero is the feet.
    on_body: f32,
    /// Where that point sits, as a fraction up the screen.
    at: f32,
    /// How far to bring the eye down to `least_elevation` regardless of the
    /// framing, from nought to one.
    ///
    /// **The floor zone's own mechanism, and it needs one.** Everywhere else a
    /// single condition places the eye and that is enough. Down here the
    /// framing stops being able to: the crosshair's mark is so close to the
    /// fighter's feet that no eye on the sphere can hold them far apart, the
    /// solve saturates, and a saturated solve does not move -- it sits against
    /// the top of the sphere answering nothing, which reads as the camera
    /// giving up halfway through a turn the player can feel they are still
    /// making.
    ///
    /// So the zone walks the eye down itself, from wherever the neutral zone
    /// handed it over to lying along the fighter's own feet at the bottom. It
    /// starts at nought, so the handover is exactly the framing's answer and
    /// there is no seam; it ends at one, so the bottom of the range is a fixed
    /// place rather than whatever the geometry happened to allow.
    ///
    /// The pan rides on top of this rather than instead of it. The camera is
    /// pointed at the mark, and the mark is itself sweeping onto the feet, so
    /// the *view* comes round to the fighter while the *eye* comes down -- two
    /// motions, which is what the zone is for.
    settle: f32,
    /// The least the eye may sit above the fighter.
    ///
    /// Zone by zone, because only one zone needs a floor. Looking almost
    /// straight down the crosshair is already at the fighter's feet, so "put
    /// the feet at the crosshair" is satisfied by any camera at all -- and
    /// asking for it *exactly* demands one in line with them, which on a fixed
    /// sphere means one swung right down behind them. Everywhere else the
    /// condition is well behaved and a floor would only stop the rig reaching
    /// the framing it was asked for: at level the eye has to come round flat to
    /// keep the head just under the mark.
    least_elevation: f32,
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

        let zones = Zones::tuned();
        let pitch = pitch.clamp(-zones.down_limit, zones.up_limit);
        let sky = zones.first_person(pitch);

        // Everything below happens in the fighter's own vertical plane: the one
        // containing them, the way they are looking, and therefore the aim
        // point, which is on a ray from their chest along that look. In that
        // plane the sphere is a circle about their feet, so placing the eye is
        // choosing one angle around it, and the one thing that has to come out
        // right is where the fighter lands on screen.
        let feet = self.focus[1] - self.cfg.look_height;
        let along = [yaw.cos(), yaw.sin()];
        let aim = [
            (aim_at[0] - self.focus[0]) * along[0] + (aim_at[2] - self.focus[2]) * along[1],
            aim_at[1] - feet,
        ];
        let body = crate::fx(sim::tuning::body_height());
        let want = zones.want(pitch, body);
        let radius = (zones.sphere * self.cfg.dolly).max(0.1);
        let [back, up] = place(&want, aim, radius, self.cfg.fov, &zones);

        // Past level the eye walks into the fighter, and it arrives at the
        // point abilities come out of rather than at the top of their head.
        // Half a metre sounds like nothing and is not: it is the difference
        // between the crosshair's line and the ability's line being the same
        // line and being six degrees apart at close range, and being the same
        // line is the entire reason this zone exists.
        let cast = crate::fx(sim::tuning::cast_height());
        let back = lerp(back, 0.0, sky);
        let up = lerp(up, cast, sky);

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

// ---------------------------------------------------------------------------
// Solving the framing
// ---------------------------------------------------------------------------

/// Where on the sphere the eye goes.
///
/// **Closed form**, and this time the geometry hands it over almost without
/// asking. The eye rides a sphere of fixed radius centred on the fighter's
/// feet, so there is exactly one number to find -- how high around it the eye
/// has climbed -- and exactly one thing to satisfy: the anchor point on the
/// fighter has to land at its mark on the screen.
///
/// One unknown, one condition. Writing the eye as `R(cos e, sin e)` and asking
/// for the angle between "eye to anchor" and "eye to aim point" to be the drop
/// below the crosshair turns into
///
/// ```text
/// A * cos(e) + B * sin(e) = C
/// ```
///
/// after the `R^2` terms cancel, which is the standard shape that collapses to
/// a single cosine: `sqrt(A^2 + B^2) * cos(e - atan2(B, A)) = C`. So the answer
/// is an `acos` either side of a lead angle, and no iteration anywhere.
///
/// The two roots are the two places on the sphere that see the pair at the same
/// angle -- one behind the fighter and one in front of them -- and the camera
/// is the flatter of the two that is genuinely behind.
///
/// **When there is no root the condition was impossible, not mistaken.** The
/// crosshair sits where the aim ray meets the ground, which at ten degrees down
/// is seven metres ahead of the fighter and at forty-five is barely one. Once
/// the mark is that close, the fighter and the crosshair are only a few degrees
/// apart *from anywhere on the sphere* -- at most `atan(mark / radius)` -- and
/// no camera can hold them half a screen apart. The lead angle is then the
/// closest the sphere comes to it, so the fighter rides up the screen toward
/// the crosshair instead of the rig snapping somewhere. That is the same walk
/// the floor zone asks for, arriving early because the geometry ran out.
fn place(want: &Want, aim: [f32; 2], radius: f32, fov: f32, zones: &Zones) -> [f32; 2] {
    let (ax, ay) = (aim[0], aim[1]);
    let c = want.on_body;
    // How far under the crosshair the anchor rides, as a tangent -- which is
    // what a screen fraction actually is under a perspective projection, the
    // screen being a flat plane a fixed distance in front of the eye rather
    // than an arc of angle. Zero drop, the anchor sitting on the mark, stays a
    // perfectly ordinary case rather than a special one.
    let drop = (1.0 - 2.0 * want.at) * (fov * 0.5).tan();

    let across = radius * ((ay - c) - drop * ax);
    let along = radius * (ax + drop * (c + ay));
    let asked = drop * (radius * radius + c * ay) + c * ax;

    let reach = (across * across + along * along).sqrt();
    let lead = along.atan2(across);
    let elevation = if reach > 1e-4 && asked.abs() <= reach {
        let spread = (asked / reach).acos();
        [lead - spread, lead + spread]
            .into_iter()
            .filter(|e| e.cos() > 0.0)
            .min_by(f32::total_cmp)
            .unwrap_or(lead)
    } else {
        lead
    };

    // The floor is taken under the ceiling first, so that two independent
    // sliders dragged past each other read as the ceiling winning rather than
    // as the game falling over -- and so that the walk down below cannot push
    // the eye back up through it. `clamp` would panic on the crossing outright.
    let floor = want.least_elevation.min(zones.max_elevation);
    let elevation = elevation.max(floor).min(zones.max_elevation);
    // Then walked down toward that floor by however much the zone asks, which
    // is nothing at all outside the bottom of the look-down range.
    let elevation = lerp(elevation, floor, want.settle);
    [radius * elevation.cos(), radius * elevation.sin()]
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
