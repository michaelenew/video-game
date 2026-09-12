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
    let level = Fx::ratio(1, 2).sub(pct(V::HeadGapLevel));
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

/// The sphere the eye rides at this aim angle, pitch in turns.
///
/// Whichever zone the angle is in, with that zone's ramp eased at both ends so
/// that it hands over to its neighbours without a corner -- see `eased`.
pub fn ball(pitch: Fx) -> Ball {
    let (down, up) = limits();
    let pitch = pitch.clamp(down.neg(), up);
    let z = edges()
        .iter()
        .position(|(_, hi)| pitch.raw() <= hi.raw())
        .unwrap_or(EYES);
    ball_of(z, pitch)
}

/// Where the eye is, for a fighter standing at `pos` looking this way.
pub fn eye(pos: V3, look: Input) -> V3 {
    let ball = ball(look.pitch_turns());

    // The tilt, as a tangent, straight from the screen position it means.
    let half = Fx::ratio(view(V::FramingFov), 720);
    let across = cos_turns(half);
    let tan_half = if across.raw() > 0 {
        sin_turns(half).div(across)
    } else {
        Fx::ZERO
    };
    let tan_tilt = Fx::ONE.sub(ball.at.mul(Fx::from_int(2))).mul(tan_half);

    // Its sine and cosine without ever taking the angle: a tangent of `k` is
    // the right triangle with sides `k` and one, so the hypotenuse does all the
    // work. This is the only reason the whole model fits in fixed point
    // without an arctangent.
    let hyp = Fx::ONE.add(tan_tilt.mul(tan_tilt)).sqrt();
    let (cos_tilt, sin_tilt) = (Fx::ONE.div(hyp), tan_tilt.div(hyp));

    // elevation = tilt - pitch, as a sine and a cosine, by the difference
    // identities -- so the eye lands without an angle being named anywhere.
    let pitch = look.pitch_turns();
    let (down, up) = limits();
    let pitch = pitch.clamp(down.neg(), up);
    let (cos_p, sin_p) = (cos_turns(pitch), sin_turns(pitch));
    let cos_e = cos_tilt.mul(cos_p).add(sin_tilt.mul(sin_p));
    let sin_e = sin_tilt.mul(cos_p).sub(cos_tilt.mul(sin_p));

    let along = V3::from_turns(look.aim_turns());
    let back = ball.radius.mul(cos_e);
    V3::new(
        pos.x.sub(along.x.mul(back)),
        pos.y.add(ball.centre).add(ball.radius.mul(sin_e)),
        pos.z.sub(along.z.mul(back)),
    )
}
