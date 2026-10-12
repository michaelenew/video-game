//! Where the eye is.
//!
//! **The camera is in the simulation now, and it has to be.** Not what it
//! looks like -- what it *points at*. The crosshair is the aim: a grounded
//! ability lands exactly where the crosshair is, so the ray that decides that
//! starts at the eye, and a ray that starts at the eye cannot be worked out
//! without knowing where the eye is.
//!
//! That is a reversal, and it is worth being plain about the cost. Camera
//! numbers used to be a personal setting, deliberately kept out of the desync
//! checksum, because aiming did not go through the camera. It does now, so they
//! are shared and checked like every other number that decides what happens.
//! Two peers framing the fight differently would aim differently, and neither
//! of them would be wrong, which is exactly the failure the checksum exists to
//! make loud. See `oven::hash`.
//!
//! Two things are deliberately *not* here. The **field of view** used for the
//! framing is its own tuned number rather than the player's setting, so that
//! widening your view does not move your aim; and the **follow smoothing,
//! floor clamp and occlusion pull-in** stay in `view::camera`, because they
//! move the drawn eye off this one and must not move the aim with it. Backed
//! against a wall the crosshair is a close reference rather than an exact one.
//!
//! ## The shape
//!
//! The camera is always on the surface of a sphere, looking inward past a
//! tilt, and the mouse walks it around that sphere at its own rate. A zone
//! changes the sphere -- where it is centred, how big it is, how far the view
//! is tilted off the line to its centre -- and never how the mouse drives it.
//!
//! The tilt is what does the framing, and the reason it works is that **the
//! sphere is centred on the thing being framed**. The line from the eye to
//! that centre is the radius the eye is standing on, whichever way round it has
//! walked, so turning the view up off that line by a fixed angle puts the
//! centre at a fixed place on the screen -- at every eye position, for free. A
//! tilt is a screen position written as an angle.
//!
//! Which leaves the eye's position free to be the mouse, directly:
//!
//! ```text
//! elevation = tilt - pitch
//! ```
//!
//! One degree of mouse, one degree around the sphere, in every zone. Nothing is
//! solved, so nothing can fail to be solvable -- which is what three earlier
//! attempts at this kept running into, each of them ending with the eye parked
//! against a limit where it stopped answering the mouse at all.
//!
//! ## Off the ground, height frames as well as pitch
//!
//! The zones above are a function of **pitch**, and they were authored for a
//! fighter standing on the floor. That is most of a match and none of the air.
//!
//! Jump, and the sphere is anchored to a body that has just risen several
//! metres: the eye goes up with it while the crosshair stays nailed to the
//! middle of the screen, so the patch of floor under the crosshair leaps
//! forward and back as you climb and fall. Aiming *down* from up there stopped
//! meaning anything -- the reticle was on a spot and nobody could predict which
//! spot.
//!
//! So height is a second driver, and what it does is **stop the camera going up
//! with you**:
//!
//! > Off the ground, the sphere stays centred below your feet, on the height
//! > you left. You climb the screen toward the crosshair instead of the camera
//! > climbing beside you -- until you are just under it, and from there it
//! > follows again.
//!
//! A vertical impulse already moves you toward the reticle; this is only the
//! camera getting out of the way and letting it read that way.
//!
//! **The reason it matters is the aim, not the look.** At a fixed downward
//! pitch, the spot on the floor under the crosshair is set by how high the eye
//! is: an eye that shoots up five metres with you sweeps that spot metres
//! further out while your hand is perfectly still. Hold the eye and the spot
//! holds too, which is the whole of the complaint that a shot aimed down from
//! the air went somewhere nobody chose.
//!
//! **How far it may hold is not a knob, it is the geometry**: exactly the rise
//! that takes you from where the pitch zone puts you to
//! [`head_under_the_crosshair`], and no further. Looking steeply down that room
//! is already spent -- the floor zone has you at the reticle with your feet on
//! the ground -- so there the camera stays glued to you, which is what you want
//! when you are looking at your own shadow. See [`hold`].
//!
//! Two knobs shape the approach, and both are what keep it from being
//! disorienting:
//!
//! - **An S-curve in height**, so a hop barely moves it and a real jump takes
//!   it all the way over -- [`aloft_target`].
//! - **A rate limit in time**, and an asymmetric one: quick to take hold on the
//!   way up, slower to let go on the way down, because terminal velocity is
//!   eighteen metres a second and the last few metres of a long fall arrive in
//!   a handful of frames -- [`aloft_step`]. That makes the framing *stateful*:
//!   it is carried per fighter in the snapshot (`Player::aloft`) and folded
//!   into the checksum, because the eye decides where abilities go and a camera
//!   with memory is a camera two peers can disagree about.

use crate::curve::Curve;
use crate::fixed::{Fx, cos_turns, sin_turns};
use crate::input::Input;
use crate::math::{V3, lerp};
use crate::oven::{ViewKnob as V, view};
use crate::tuning as t;

/// The sphere the eye is riding, at one aim angle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ball {
    /// Height up the fighter the sphere is centred on. Zero is the feet.
    pub centre: Fx,
    pub radius: Fx,
    /// Where the sphere's centre sits on screen, as a fraction from the bottom.
    ///
    /// Kept as the screen position rather than as the angle, because that is
    /// what it is for and what the knob says. The angle is `atan` of it against
    /// the framing field of view, and it never has to be taken: what the
    /// placement needs is the tangent, which is this number directly.
    pub at: Fx,
}

fn turns(degrees: i32) -> Fx {
    Fx::ratio(degrees, 360)
}

fn pct(v: V) -> Fx {
    Fx::ratio(view(v), 100)
}

fn metres(v: V) -> Fx {
    Fx::from_raw(view(v))
}

/// How far down and up the aim may go, in turns.
pub fn limits() -> (Fx, Fx) {
    (turns(view(V::LookDownLimit)), turns(view(V::LookUpLimit)))
}

/// The five zones, bottom to top, and the boundary above each.
const FLOOR: usize = 0;
const NEUTRAL: usize = 1;
const TURN: usize = 2;
const HANDOVER: usize = 3;
const EYES: usize = 4;

/// Where each zone starts and ends, in turns.
fn edges() -> [(Fx, Fx); 5] {
    let (down, up) = limits();
    let floor_from = turns(view(V::FloorZoneFrom)).neg();
    let neutral_to = turns(view(V::NeutralZoneTo)).neg();
    let head_lock = turns(view(V::HeadLockAt));
    [
        (down.neg(), floor_from),
        (floor_from, neutral_to),
        (neutral_to, Fx::ZERO),
        (Fx::ZERO, head_lock),
        (head_lock, up),
    ]
}

/// How much of each zone's own span is given over to easing, at each end.
///
/// A share of the zone's own ramp, which is what makes one number mean the same
/// thing in a band four degrees wide and one seventy-five degrees wide. Half is
/// the most it can be: at half, the two ends meet and the ramp is eased the
/// whole way through, which is as smooth as a zone gets.
fn eases() -> [Fx; 5] {
    let share = [
        pct(V::SmoothFloor),
        pct(V::SmoothNeutral),
        pct(V::SmoothTurn),
        pct(V::SmoothHandover),
        pct(V::SmoothEyes),
    ];
    share.map(|s| s.clamp(Fx::ZERO, Fx::ratio(1, 2)))
}

/// A ramp that leaves and arrives at a standstill, without giving up its middle.
///
/// **The whole of the smoothing, and it is a remap of the ramp rather than
/// anything added on top.** A zone boundary with nothing done about it is
/// continuous in position and not in speed: the eye arrives moving one way and
/// leaves moving another. That is not seen so much as felt, and it reads as the
/// camera changing its mind.
///
/// A neighbouring zone at a shared boundary is holding still -- its own ramp has
/// either not started or already finished -- so all that is needed is for this
/// one to start and finish at rest too. The first and last `ease` of the ramp
/// are the cubic through `(0,0)` and `(1,1)` that leaves flat and arrives at the
/// straight line's own slope, so the two meet without a corner. Everything
/// between them is untouched, which is what keeps the waypoints exactly true
/// wherever the window does not reach.
fn eased(t: Fx, ease: Fx) -> Fx {
    if ease.raw() <= 0 {
        return t;
    }
    // `u * u * (2 - u)`: flat at nought, and at one it is climbing at exactly
    // the gradient of the line it is joining.
    let knee = |u: Fx| ease.mul(u.mul(u).mul(Fx::from_int(2).sub(u)));
    if t.raw() < ease.raw() {
        knee(t.div(ease))
    } else if t.raw() > Fx::ONE.sub(ease).raw() {
        Fx::ONE.sub(knee(Fx::ONE.sub(t).div(ease)))
    } else {
        t
    }
}

/// The sphere zone `z` asks for at this aim angle.
///
/// Every zone answers at *any* angle, not only inside its own band: past its
/// edges its ramp simply runs out, which leaves it holding the waypoint it was
/// heading for. That is what makes the handover below a plain weighted average
/// rather than a special case -- outside the window the two zones already agree,
/// so blending them changes nothing.
fn ball_of(z: usize, pitch: Fx) -> Ball {
    let (lo, hi) = edges()[z];
    let span = hi.sub(lo);
    let t = if span.raw() > 0 {
        pitch.sub(lo).div(span).clamp(Fx::ZERO, Fx::ONE)
    } else {
        Fx::ONE
    };
    let t = eased(t, eases()[z]);
    let body = t::body_height();
    let sphere = metres(V::Sphere);
    let low = pct(V::FeetNeutral);
    let level = head_under_the_crosshair();
    let centred = Fx::ratio(1, 2);

    match z {
        // **The floor zone.** The view tilts down onto the fighter's own feet,
        // so that at the bottom of the range the crosshair is on them -- the
        // shot that puts a stone underneath you.
        FLOOR => Ball {
            centre: Fx::ZERO,
            radius: sphere,
            at: lerp(pct(V::FeetFloor), low, t),
        },
        // **The neutral zone**, where most of a match is spent. Nothing about
        // the sphere changes here at all -- only where the eye is on it -- so
        // the fighter sits at exactly the same spot on screen through the whole
        // band while the camera swings around behind them.
        NEUTRAL => Ball {
            centre: Fx::ZERO,
            radius: sphere,
            at: low,
        },
        // **The turn.** The sphere slides up the body to the head and the tilt
        // comes with it, until at level the crosshair rides just above the head
        // -- which is what gives a mid-range skillshot something to key off when
        // there is no ground under the aim to read it against.
        TURN => Ball {
            centre: lerp(Fx::ZERO, body, t),
            radius: sphere,
            at: lerp(low, level, t),
        },
        // **The handover.** The sphere shrinks onto the head and the tilt goes
        // to nothing, which leaves the camera looking straight down the line the
        // crosshair draws.
        HANDOVER => Ball {
            centre: body,
            radius: lerp(sphere, metres(V::HeadSphere), t),
            at: lerp(level, centred, t),
        },
        // **The fighter's own eye**, and nothing moves again.
        _ => Ball {
            centre: body,
            radius: metres(V::HeadSphere),
            at: centred,
        },
    }
}

/// Where the head sits on screen when the crosshair is riding **just above
/// it**.
///
/// One number, read twice on purpose: it is the top waypoint of the turn zone
/// and it is the whole of the airborne framing, because those are the same
/// sentence. A second copy would be a second thing to drag apart, and then the
/// camera would mean one thing when you looked level and another when you
/// jumped.
fn head_under_the_crosshair() -> Fx {
    Fx::ratio(1, 2).sub(pct(V::HeadGapLevel))
}

/// Half the framing field of view, as a tangent.
///
/// Taken as a tangent and never as an angle, which is the trick the whole
/// module rests on: a screen fraction times this is a distance, and a distance
/// divided by it is a screen fraction, with no arctangent anywhere.
fn tan_half_fov() -> Fx {
    let half = Fx::ratio(view(V::FramingFov), 720);
    let across = cos_turns(half);
    if across.raw() > 0 {
        sin_turns(half).div(across)
    } else {
        Fx::ZERO
    }
}

/// How far the camera may hold still while a fighter climbs away from it, in
/// metres.
///
/// **Derived, not tuned.** It is the rise that takes the fighter from wherever
/// the pitch zone has put them on screen up to
/// [`head_under_the_crosshair`] -- no further, because past that they would be
/// climbing over the reticle, and the reticle is the one thing on screen a
/// player is deliberately holding still.
///
/// Which means it answers the awkward cases without being told about them.
/// Looking steeply down, the floor zone already has the fighter at the
/// crosshair with their feet on the ground, so there is no room left and the
/// camera stays glued to them -- exactly right when what you are looking at is
/// the patch of floor you are standing over. Looking level or up, the turn and
/// the handover have already taken the framing past this, and the room is zero
/// again.
///
/// The conversion from a share of the screen to metres is the screen's own
/// height *at the sphere's surface*: `2 · radius · tan(fov/2)`, which is the
/// same tangent the tilt is built from.
fn hold(standing: &Ball) -> Fx {
    let room = head_under_the_crosshair().sub(standing.at);
    if room.raw() <= 0 {
        return Fx::ZERO;
    }
    let screen = standing.radius.mul(tan_half_fov()).mul(Fx::from_int(2));
    room.mul(screen)
}

/// The S-curve: how far over to the airborne framing a fighter `height` above
/// the floor should be, from none of it to all of it.
///
/// **In height, not in time.** Time is [`aloft_step`]'s job. Keeping them apart
/// is what makes a hop and a fall from the same height frame the same way at
/// the same moment: the curve says where the camera belongs and the rate limit
/// says how fast it may get there.
///
/// The shape is the Oven's, on the same cubic the stone rise uses -- flat at
/// the bottom so a short hop barely moves the view, flat at the top so arriving
/// at full height is not a jolt.
pub fn aloft_target(height: Fx) -> Fx {
    let full = metres(V::AloftFull);
    let through = if full.raw() > 0 {
        height.max(Fx::ZERO).div(full)
    } else {
        Fx::ONE
    };
    aloft_curve().at(through)
}

/// The Oven's airborne-framing curve.
fn aloft_curve() -> Curve {
    Curve {
        x1: metres(V::AloftCurveX1),
        y1: metres(V::AloftCurveY1),
        x2: metres(V::AloftCurveX2),
        y2: metres(V::AloftCurveY2),
    }
}

/// One frame of the swing toward `target`, held to the camera's own rate limit.
///
/// **Two rates, because the two directions are not the same problem.** Going up
/// you leave the floor at a standstill and the curve is flat down there, so the
/// framing has time to take hold and wants to be quick about it -- the camera
/// should already be out of the way by the time you are climbing in earnest.
/// Coming down is the opposite: terminal velocity is eighteen metres a second,
/// so the last few metres of a long fall arrive in a handful of frames, and a
/// framing that tracked height exactly would snap the view through its whole
/// travel in those frames. Letting go slower than it takes hold turns that into
/// a glide. Set the two equal and the asymmetry is simply off.
///
/// It is also why the framing is **state**: a value that may only move so far
/// per frame is a value that remembers where it was. It lives in the snapshot
/// (`state::Player::aloft`) and is folded into the checksum, like everything
/// else that decides where an ability goes.
pub fn aloft_step(now: Fx, target: Fx) -> Fx {
    let gap = target.sub(now);
    let rate = if gap.raw() > 0 {
        Fx::from_raw(view(V::AloftRate))
    } else {
        Fx::from_raw(view(V::AloftEase))
    }
    .max(Fx::ZERO);
    let next = if gap.abs().raw() <= rate.raw() {
        target
    } else if gap.raw() > 0 {
        now.add(rate)
    } else {
        now.sub(rate)
    };
    next.clamp(Fx::ZERO, Fx::ONE)
}

/// The sphere the eye rides at this aim angle and this far off the ground.
///
/// Whichever zone the angle is in, with that zone's ramp eased at both ends so
/// that it hands over to its neighbours without a corner -- see `eased` -- and
/// then **dropped**: off the ground the sphere is centred below the fighter's
/// feet, on the height they left, so that rising carries them up the screen
/// rather than carrying the camera along with them.
///
/// `aloft` is how much of that hold is in effect, nought to one -- see
/// [`aloft_target`] for where it comes from and [`aloft_step`] for why it has
/// memory. The metres it is a share of are [`hold`]'s, which is the geometry
/// rather than a number: the fighter is never carried past the crosshair.
///
/// Nothing else about the sphere moves. The zones decide where a fighter sits
/// on screen and this does not argue with them; it only declines to follow one
/// upward for a while.
pub fn ball(pitch: Fx, aloft: Fx, clear: Fx) -> Ball {
    let (down, up) = limits();
    let pitch = pitch.clamp(down.neg(), up);
    let z = edges()
        .iter()
        .position(|(_, hi)| pitch.raw() <= hi.raw())
        .unwrap_or(EYES);
    let standing = ball_of(z, pitch);
    let dropped = hold(&standing).mul(aloft.clamp(Fx::ZERO, Fx::ONE));
    let held = Ball {
        centre: standing.centre.sub(dropped),
        ..standing
    };
    Ball {
        radius: let_out(&held, pitch, clear),
        ..held
    }
}

/// The tilt's and the elevation's sines and cosines for this sphere at this
/// pitch: `(cos_e, sin_e)`, where the elevation is how far above the sphere's
/// centre the eye stands, as an angle. Never takes an angle -- see [`eye`].
fn elevation(ball: &Ball, pitch: Fx) -> (Fx, Fx) {
    // The tilt, as a tangent, straight from the screen position it means.
    let tan_tilt = Fx::ONE
        .sub(ball.at.mul(Fx::from_int(2)))
        .mul(tan_half_fov());

    // Its sine and cosine without ever taking the angle: a tangent of `k` is
    // the right triangle with sides `k` and one, so the hypotenuse does all the
    // work. This is the only reason the whole model fits in fixed point
    // without an arctangent.
    let hyp = Fx::ONE.add(tan_tilt.mul(tan_tilt)).sqrt();
    let (cos_tilt, sin_tilt) = (Fx::ONE.div(hyp), tan_tilt.div(hyp));

    // elevation = tilt - pitch, as a sine and a cosine, by the difference
    // identities -- so the eye lands without an angle being named anywhere.
    let (cos_p, sin_p) = (cos_turns(pitch), sin_turns(pitch));
    let cos_e = cos_tilt.mul(cos_p).add(sin_tilt.mul(sin_p));
    let sin_e = sin_tilt.mul(cos_p).sub(cos_tilt.mul(sin_p));
    (cos_e, sin_e)
}

/// The room the eye has under the sphere's centre before it comes within the
/// bubble of the floor, in metres -- and **never more than the fighter's own
/// height off that floor**, so that with their feet on something it is at most
/// nothing whatever the knobs say. That is the whole of the promise that
/// standing on the ground the camera is exactly the one it always was.
fn room(centre: Fx, clear: Fx) -> Fx {
    let bubble = metres(V::EyeBubble);
    clear.min(clear.add(centre).sub(bubble))
}

/// How far into its pull-back the bubble is, nought to one: the soft edge.
///
/// Nought with no room, one once there are `Eye bubble, soft edge` metres of
/// it, and eased at both ends between -- so the camera leaves the head, and
/// arrives at the distance the floor allows, without a corner in either.
fn opened(room: Fx) -> Fx {
    let soft = metres(V::EyeBubbleSoft);
    let u = if soft.raw() > 0 {
        room.div(soft).clamp(Fx::ZERO, Fx::ONE)
    } else if room.raw() > 0 {
        Fx::ONE
    } else {
        Fx::ZERO
    };
    // `u * u * (3 - 2u)`: flat at both ends.
    u.mul(u).mul(Fx::from_int(3).sub(u.mul(Fx::from_int(2))))
}

/// **The bubble**: the radius the zones asked for, let back out toward the
/// full sphere as far as the floor under the fighter allows.
///
/// Looking up, the zones shrink the sphere onto the head -- the handover --
/// because on the ground an eye swung down behind a fighter looking up would
/// go into the floor. That is the floor's reason, and off the ground the floor
/// is somewhere else. So the camera keeps the sphere's own radius **unless
/// that would bring the eye within `Eye bubble` of the floor under the
/// fighter**, and stops exactly there if so: a hard edge, not a slope. The
/// soft edge is [`opened`], easing the pull-back in over the first two metres of
/// room so that leaving the floor is not a switch.
///
/// Three things it never does, and they are the no-regression argument:
///
/// - **It never moves a camera on the ground.** [`room`] is at most the
///   fighter's height off the floor, nought standing, and nought room is the
///   zones' radius exactly.
/// - **It never moves the crosshair.** The tilt -- where the sphere's centre
///   sits on screen -- is the zones', untouched; only the distance changes,
///   and a distance along the line to the centre does not move the centre on
///   screen. Looking up, the zones have centred the head, so the crosshair is
///   on the fighter at every distance.
/// - **It never jumps.** Continuous in room through [`opened`], room
///   continuous in height through [`clear_step`]'s rate limit, and continuous
///   in pitch because the floor's limit is (it runs to the full sphere as the
///   eye comes level with the centre, which is where the zones' own radius
///   already is).
///
/// What it does not see is anything but the floor: a wall or a ledge behind
/// the fighter is the drawn camera's to pull in from, as it always was in
/// third person (`view::camera`).
fn let_out(ball: &Ball, pitch: Fx, clear: Fx) -> Fx {
    let sphere = metres(V::Sphere);
    if ball.radius.raw() >= sphere.raw() {
        return ball.radius;
    }
    let room = room(ball.centre, clear);
    let share = opened(room);
    if share.raw() <= 0 {
        return ball.radius;
    }
    let (_, sin_e) = elevation(ball, pitch);
    let below = sin_e.neg();
    // How far back the eye may go before the floor's bubble stops it. Asked as
    // a product rather than a quotient first, so a level eye -- `below` near
    // nothing -- never divides.
    let floor_allows = if below.raw() <= 0 || below.mul(sphere).raw() <= room.raw() {
        sphere
    } else {
        room.div(below)
    };
    let wants = floor_allows.max(ball.radius);
    ball.radius.add(wants.sub(ball.radius).mul(share))
}

/// How far the bubble has let the camera back out of the head, nought to one:
/// for the drawn camera, which treats an eye inside the head and an eye in
/// the air behind it differently (its floor clamp and its arm's pull-in only
/// apply to the second).
pub fn let_out_share(pitch: Fx, aloft: Fx, clear: Fx) -> Fx {
    let (down, up) = limits();
    let pitch = pitch.clamp(down.neg(), up);
    let inside = ball(pitch, aloft, Fx::ZERO);
    let sphere = metres(V::Sphere);
    let span = sphere.sub(inside.radius);
    if span.raw() <= 0 {
        return Fx::ZERO;
    }
    let out = ball(pitch, aloft, clear).radius.sub(inside.radius);
    out.div(span).clamp(Fx::ZERO, Fx::ONE)
}

/// One frame of the fighter's height off the floor **as the camera is allowed
/// to know it** (`state::Player::clear`).
///
/// Down at once, because the bubble is a hard edge: an eye that learned of a
/// floor late would already be in it. Up by at most `Eye bubble, most it opens
/// a frame`, because stepping off a ledge puts the floor metres away in one
/// frame, and the camera would follow it out in that frame.
pub fn clear_step(now: Fx, height: Fx) -> Fx {
    let height = height.max(Fx::ZERO);
    let rate = metres(V::EyeBubbleRate).max(Fx::ZERO);
    height.min(now.add(rate))
}

/// Where the eye is, for a fighter at `pos` looking this way.
///
/// `aloft` is how far the framing has swung over to the airborne one -- zero
/// with their feet on something, one well above it, and whatever
/// [`aloft_step`] has walked it to in between. It is a fighter's own state
/// rather than a function of this position, because it has memory: see
/// `state::Player::aloft`.
///
/// `clear` is how high their feet are off the floor under them, as the camera
/// may know it -- `state::Player::clear`, nought standing -- which is how far
/// the bubble lets the eye back out of the head looking up: see [`let_out`].
pub fn eye(pos: V3, look: Input, aloft: Fx, clear: Fx) -> V3 {
    let (down, up) = limits();
    let pitch = look.pitch_turns().clamp(down.neg(), up);
    let ball = ball(pitch, aloft, clear);
    let (cos_e, sin_e) = elevation(&ball, pitch);

    let along = V3::from_turns(look.aim_turns());
    let back = ball.radius.mul(cos_e);
    V3::new(
        pos.x.sub(along.x.mul(back)),
        pos.y.add(ball.centre).add(ball.radius.mul(sin_e)),
        pos.z.sub(along.z.mul(back)),
    )
}

/// [`eye`], kept under the arena's ceiling.
///
/// **The aiming ray starts here**, so an eye that rose through a cave's vault
/// would cast every shot from inside the rock. It is held
/// `tuning::eye_under_ceiling` below the lowest ceiling over the eye or over
/// the fighter -- a solid hanging from the roof whose underside is above the
/// fighter's head -- and nowhere else does it move. With no ceiling over
/// either it is [`eye`] exactly, which is every arena but a cave or a course.
/// Near a ceiling's edge the limit slopes away (`tuning::eye_ceiling_slope`)
/// rather than stopping, so crossing an edge never jumps the eye.
///
/// The drawn camera starts from this one too (`view::camera`), so the
/// crosshair and the eye stay the same line under a vault.
pub fn eye_under(pos: V3, look: Input, aloft: Fx, clear: Fx, arena: &crate::arena::Terrain) -> V3 {
    let mut at = eye(pos, look, aloft, clear);
    let head = pos.y.add(t::body_height());
    // Sloped rather than a footprint test, so the eye moves continuously as
    // it or the body passes an edge: see `Terrain::ceiling_near`.
    let slope = t::eye_ceiling_slope();
    let over_eye = arena.ceiling_near(at.x, at.z, head, slope);
    let over_body = arena.ceiling_near(pos.x, pos.z, head, slope);
    let lowest = match (over_eye, over_body) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    };
    if let Some(ceiling) = lowest {
        let cap = ceiling.sub(t::eye_under_ceiling()).max(head);
        if at.y.raw() > cap.raw() {
            at.y = cap;
        }
    }
    at
}
