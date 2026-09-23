//! Leaving the ground, being off it, and arriving back.
//!
//! Six clips covering one action, and `view::play::airborne` picks between
//! them from **vertical speed** rather than from a timeline. That is the thing
//! to know before authoring any of them, because it decides what each one is
//! actually asked to do:
//!
//! ```text
//! air_frames 1..5   jump_takeoff, frame by frame
//! after that        jump_rise / jump_apex / jump_fall, cross-faded by rise
//!                   and all indexed by frames-since-takeoff
//! on landing        land_soft or land_heavy, by how long the flight was
//! ```
//!
//! ## The jump these are attached to
//!
//! Takeoff is 16.2 m/s against 42 m/s² of gravity, with a sustain window of 18
//! frames at 72% gravity while the button is held. So a full hop climbs for
//! something like twenty-seven frames and the whole flight runs from two thirds
//! of a second on the heaviest class to a little over a second on the
//! floatiest, while a tapped one is airborne for about half of that. The jump
//! is deliberately floaty -- verticality is meant to be part of the positioning
//! game -- and floaty means these poses are *stared at*. Nothing in the air can
//! be a pose that only works in passing.
//!
//! **Those three numbers moved on 2026-09-17** and the paragraph above is the
//! second place in the repository that states them, which is the kind of thing
//! that goes stale silently: the authoritative ones are in `sim::tuning` and
//! `cargo run -p sim --bin frametable` prints the apex and the airtime per
//! class. The nerf took about a third off every apex and shortened the sustain,
//! so the rise is a shorter and more decelerating thing than it was -- which
//! these clips are indexed against by *vertical speed* rather than by a
//! timeline, so none of them needed reauthoring for it.
//!
//! It also means the body is already travelling at better than a quarter of a
//! metre a frame by the time `jump_takeoff` gets its first look at it. The clip
//! cannot lift anybody. What it can do is look like the thing that threw them,
//! which is why the coil is a full crouch and the extension is done inside four
//! frames.
//!
//! ## Two things that are easy to miss
//!
//! **Frame zero of `jump_takeoff` is never drawn.** `air_frames` is already one
//! on the first frame off the ground, so the coil a player sees is frame one.
//! Frame zero is where the springs start settled, and the ease out of it is
//! what keeps frame one deep rather than halfway to the push.
//!
//! **The two loops are entered at an arbitrary phase.** `jump_rise` and
//! `jump_fall` are indexed by frames since takeoff, not from the moment the
//! rise or the fall began, so neither has a beginning that lines up with
//! anything. They are held shapes with a drift on them rather than cycles: a
//! loop with a beat in it would flick past at a different point on every jump,
//! and because the apex cross-fades against these same frames, a loop that
//! swings would make the hang at the top flutter.
//!
//! Feet leave the floor in nearly all of this, which is the point. The two that
//! touch it -- the first frames of `jump_takeoff` and the landings -- place
//! their feet with `plant_*` and roll them with the `toe_*` helpers, so a
//! contact is a contact rather than an angle that looks about right.

use crate::bake::{Key, Looseness, Recipe};
use crate::clips::locomotion::stance;
use crate::ease::Ease;
use view::clips::Clip;
use view::pose::{ANKLE_ON_GROUND as GROUND, Pose};

/// Where the feet sit either side of the centre line, as in the stance every
/// other clip is read against.
const L: f32 = -0.115;
const R: f32 = 0.115;

pub fn clips() -> Vec<Recipe> {
    vec![takeoff(), rise(), apex(), fall(), land_soft(), land_heavy()]
}

// ---------------------------------------------------------------------------
// Takeoff
// ---------------------------------------------------------------------------

/// The crouch. Deep enough to be a crouch rather than a dip: this is the only
/// frame of anticipation there is, and a shallow one reads as the character
/// being lifted rather than jumping.
fn coil() -> Pose {
    Pose::rest()
        .hips(0.0, -0.30, 0.05)
        .root(10.0, 0.0, -8.0)
        .spine(20.0, 0.0, 5.0)
        .chest(8.0, 0.0, 6.0)
        // Eyes stay up. A head that follows the torso down loses the face at
        // exactly the moment the opponent is trying to read the jump.
        .head(-18.0, 0.0, 3.0)
        // Arms behind the body, ready to be thrown up. Not as far behind as a
        // standing jump would take them: the whole sweep has to fit in four
        // frames without any one frame moving a shoulder further than an eye
        // can follow.
        .shoulder_l(-16.0, 12.0, 0.0)
        .elbow_l(26.0)
        .shoulder_r(-14.0, 14.0, 0.0)
        .elbow_r(22.0)
        .plant_l([L - 0.01, GROUND, 0.09])
        .plant_r([R + 0.01, GROUND, -0.07])
        .toe_l(0.0)
        .toe_r(0.0)
}

/// Mid-push: the legs are most of the way open and the body has rolled up onto
/// the balls of both feet, which is where the last of the force comes from.
fn drive() -> Pose {
    Pose::rest()
        .hips(0.0, -0.04, 0.02)
        .root(2.0, 0.0, -8.0)
        .spine(6.0, 0.0, 5.0)
        .chest(2.0, 0.0, 6.0)
        .head(-12.0, 0.0, 3.0)
        .shoulder_l(46.0, 20.0, 0.0)
        .elbow_l(34.0)
        .shoulder_r(42.0, 22.0, 0.0)
        .elbow_r(32.0)
        .plant_l([L - 0.01, GROUND + 0.12, 0.05])
        .plant_r([R + 0.01, GROUND + 0.12, -0.04])
        .toe_floor_l()
        .toe_floor_r()
}

/// Full extension: one line from the pointed toes to the hands, at the instant
/// the feet have nothing left to push against.
fn extended() -> Pose {
    Pose::rest()
        .hips(0.0, 0.08, -0.01)
        .root(-4.0, 0.0, -8.0)
        .spine(-4.0, 0.0, 5.0)
        .chest(-2.0, 0.0, 6.0)
        .head(-14.0, 0.0, 3.0)
        .shoulder_l(108.0, 26.0, 0.0)
        .elbow_l(16.0)
        .shoulder_r(98.0, 30.0, 0.0)
        .elbow_r(14.0)
        .hip_l(-12.0, 6.0, 0.0)
        .knee_l(8.0)
        .hip_r(-8.0, 5.0, 0.0)
        .knee_r(6.0)
        .ankle_l(42.0, 0.0, 0.0)
        .ankle_r(44.0, 0.0, 0.0)
}

fn takeoff() -> Recipe {
    Recipe {
        clip: Clip::JumpTakeoff,
        // Not floaty: this is the one clip in the family where the body is
        // fighting gravity rather than riding it, and a leg that lags is a leg
        // that never finishes extending inside six frames.
        looseness: Looseness::MARTIAL,
        notes: "Six frames to sell a jump, against a takeoff that is already \
                moving the body a third of a metre a frame -- so the clip has to \
                look like the thing that threw the character rather than the \
                thing lifting them. A full crouch with both heels down, then the \
                legs open and the arms come up together and the toes point by \
                frame four. The anticipation out of the first key sinks the \
                crouch a little further before it goes, because the one frame of \
                coil a player actually sees is frame one. The last key is most of \
                the way into what `jump_rise` holds: playback cuts between the \
                two with no cross-fade, so a takeoff that ends anywhere else pops."
            .into(),
        keys: vec![
            // Frame zero is a *part* coil, not a full one. This clip starts on
            // the frame the body has already left the ground, so the crouch it
            // opens on is the one the character is coming out of rather than
            // one they are still sinking into -- and two frames from a full
            // squat to a full drive moves the chest twenty centimetres a frame,
            // which is a teleport rather than a jump.
            Key::eased(0, coil().blend(&drive(), 0.35), Ease::ANTICIPATE),
            Key::eased(3, drive(), Ease::STRIKE),
            Key::eased(4, extended(), Ease::OUT),
            // Already letting the legs trail, so the handoff into the rise is a
            // continuation rather than a cut.
            Key::eased(5, extended().blend(&rising(1.0), 0.55), Ease::OUT),
        ],
    }
}

// ---------------------------------------------------------------------------
// Rise
// ---------------------------------------------------------------------------

/// Going up. `k` slides between the two ends of the drift the loop runs on:
/// `+1` is the stretched end, just off the ground, `-1` the gathered one.
///
/// A single parameterised shape rather than three keyed poses, because the loop
/// has to close on itself exactly and the amplitude has to be small enough to
/// stay invisible -- both of which are easier to guarantee with one number than
/// with three sets of angles that drift apart the moment one is retouched.
fn rising(k: f32) -> Pose {
    Pose::rest()
        .hips(0.0, 0.05 + 0.010 * k, -0.02)
        .root(-5.0 - 1.5 * k, 0.0, -9.0)
        .spine(-5.0 - 1.0 * k, 0.0, 6.0)
        .chest(-2.0, 0.0, 7.0)
        // Looking up the way you are going, which is also what tells an
        // opponent this is a jump and not a hit.
        .head(-14.0 + 2.0 * k, 0.0, 4.0)
        .shoulder_l(112.0 + 5.0 * k, 26.0, 0.0)
        .elbow_l(26.0 - 4.0 * k)
        .shoulder_r(100.0 + 4.0 * k, 32.0, 0.0)
        .elbow_r(36.0 - 4.0 * k)
        // The legs trail: they were the last thing to leave the floor and they
        // are still behind the body.
        .hip_l(-20.0 - 4.0 * k, 6.0, 0.0)
        .knee_l(44.0 - 8.0 * k)
        .hip_r(-13.0 - 3.0 * k, 5.0, 0.0)
        .knee_r(32.0 - 6.0 * k)
        .ankle_l(38.0, 0.0, 0.0)
        .ankle_r(41.0, 0.0, 0.0)
}

fn rise() -> Recipe {
    Recipe {
        clip: Clip::JumpRise,
        looseness: Looseness::FLOATY,
        notes: "A held shape with a drift on it rather than a cycle. It is \
                indexed by frames since takeoff and held for as long as the \
                climb lasts -- thirty-seven frames on a full hop -- so it loops \
                twice in front of the player and is entered at whatever phase \
                the flight happens to be at. Anything with a beat in it would \
                land on a different frame every jump, and the apex cross-fades \
                against these same frames, so a swing here would make the hang \
                flutter. The body is long, the arms are up where the takeoff \
                threw them, and the legs trail because they were the last thing \
                off the floor."
            .into(),
        keys: vec![
            Key::eased(0, rising(1.0), Ease::SMOOTH),
            Key::eased(7, rising(-1.0), Ease::SMOOTH),
            // Not halfway back: an even drift is a metronome, and a metronome
            // is exactly what a held clip must not be.
            Key::eased(12, rising(-0.2), Ease::SMOOTH),
        ],
    }
}

// ---------------------------------------------------------------------------
// Apex
// ---------------------------------------------------------------------------

/// Still going up, but out of ideas: the rise's shape with the climb gone out
/// of it.
fn topping_out() -> Pose {
    rising(0.4)
        .hips(0.0, 0.045, -0.01)
        .root(-2.0, 0.0, -9.0)
        .head(-8.0, 0.0, 4.0)
}

/// The turn itself: the arms come down out of the reach and open sideways, and
/// the knees start to come up under the body.
fn opening() -> Pose {
    Pose::rest()
        .hips(0.0, 0.03, 0.0)
        .root(0.0, 0.0, -8.0)
        .spine(0.0, 0.0, 6.0)
        .chest(0.0, 0.0, 7.0)
        .head(-4.0, 0.0, 4.0)
        .shoulder_l(78.0, 52.0, 0.0)
        .elbow_l(38.0)
        .shoulder_r(66.0, 58.0, 0.0)
        .elbow_r(46.0)
        .hip_l(14.0, 10.0, 0.0)
        .knee_l(62.0)
        .hip_r(6.0, 9.0, 0.0)
        .knee_r(50.0)
        .ankle_l(30.0, 0.0, 0.0)
        .ankle_r(33.0, 0.0, 0.0)
}

/// The hang. The widest, stillest shape in the family, and the one a player
/// reads their height off.
fn hang() -> Pose {
    Pose::rest()
        .hips(0.0, 0.005, 0.0)
        .root(-2.0, 0.0, -8.0)
        .spine(2.0, 0.0, 5.0)
        // Chest open rather than folded: the tuck is in the legs, and a body
        // that curls up as well has nothing left to read.
        .chest(-3.0, 0.0, 7.0)
        // Level. This is the frame where the player is looking around, and a
        // head still pointed at the sky says the jump is not over.
        .head(2.0, 0.0, 5.0)
        // Wide. Everything else in this family is a narrow silhouette, so the
        // arms out to the sides are most of what makes the top of the jump the
        // frame a player can pick out.
        .shoulder_l(44.0, 78.0, 0.0)
        .elbow_l(54.0)
        .shoulder_r(32.0, 82.0, 0.0)
        .elbow_r(62.0)
        .hip_l(54.0, 13.0, 0.0)
        .knee_l(94.0)
        .hip_r(42.0, 11.0, 0.0)
        .knee_r(80.0)
        .ankle_l(18.0, 0.0, 0.0)
        .ankle_r(22.0, 0.0, 0.0)
}

fn apex() -> Recipe {
    Recipe {
        clip: Clip::JumpApex,
        looseness: Looseness::FLOATY,
        notes: "The one moment in a jump a player can read their height off, so \
                it is the widest and the stillest shape in the family: arms out \
                to the sides, knees gathered up, head level and looking around \
                rather than at the sky. It is cross-faded in from both sides by \
                vertical speed, and on a full hop the hang lasts longer than the \
                clip does -- the last key is what a player actually stares at, \
                so the shape has to have arrived by then and be worth holding. \
                The hold before it is deliberate: a hang that keeps moving is a \
                float, and a float is not a jump."
            .into(),
        keys: vec![
            Key::eased(0, topping_out(), Ease::OUT),
            Key::eased(4, opening(), Ease::SMOOTH),
            Key::eased(9, hang(), Ease::HOLD),
            // The weight starting to come back. Small on purpose: the fall
            // takes over from here and this only has to lean that way.
            Key::eased(
                13,
                hang().hips(0.0, -0.01, 0.01).head(6.0, 0.0, 5.0),
                Ease::OUT,
            ),
        ],
    }
}

// ---------------------------------------------------------------------------
// Fall
// ---------------------------------------------------------------------------

/// Coming down, legs gathering under the body. `k` slides along the same kind
/// of small drift the rise has: `+1` is the open end, `-1` the gathered one.
fn falling(k: f32) -> Pose {
    Pose::rest()
        .hips(0.0, -0.03 + 0.008 * k, 0.03)
        // Barely leaning. A body dropping is upright with its legs underneath
        // it; the forward pitch belongs to the landing, not to the fall.
        .root(3.0 + 1.0 * k, 0.0, -9.0)
        .spine(5.0 + 1.5 * k, 0.0, 6.0)
        .chest(2.0, 0.0, 7.0)
        // Looking at the floor it is about to arrive on.
        .head(16.0 - 2.0 * k, 0.0, 4.0)
        // Out and low, and not the same on both sides: one hand forward over
        // the leading foot and one trailing back is a diagonal that reads from
        // any angle, where two arms doing the same thing read as a shrug.
        .shoulder_l(24.0 + 4.0 * k, 50.0, 0.0)
        .elbow_l(40.0 - 3.0 * k)
        .shoulder_r(-18.0 + 3.0 * k, 56.0, 0.0)
        .elbow_r(34.0 - 3.0 * k)
        // The legs are not doing the same thing, and that is the whole read: a
        // lead leg already reaching for the floor and a trailing knee still
        // gathered, which is a diagonal rather than a person standing in
        // mid-air. It also puts the feet where both landings start from -- lead
        // left, trail right.
        .hip_l(26.0 + 5.0 * k, 8.0, 0.0)
        .knee_l(16.0 + 6.0 * k)
        .hip_r(40.0 + 6.0 * k, 9.0, 0.0)
        .knee_r(66.0 + 8.0 * k)
        .ankle_l(30.0, 0.0, 0.0)
        .ankle_r(16.0, 0.0, 0.0)
}

fn fall() -> Recipe {
    Recipe {
        clip: Clip::JumpFall,
        looseness: Looseness::FLOATY,
        notes: "Held the same way the rise is, and for as long -- a fall from a \
                full hop runs about thirty frames and terminal velocity means it \
                can run much longer -- so it is a shape with a drift on it \
                rather than a cycle, entered at whatever phase the flight \
                reached. The legs are already gathering under the body and the \
                feet are ahead of the hips, which is where `land_soft` and \
                `land_heavy` both start: the landings cut in from here with no \
                cross-fade, and a fall that ends anywhere else arrives as a pop."
            .into(),
        keys: vec![
            Key::eased(0, falling(-1.0), Ease::SMOOTH),
            Key::eased(6, falling(1.0), Ease::SMOOTH),
            Key::eased(13, falling(-0.3), Ease::SMOOTH),
        ],
    }
}

// ---------------------------------------------------------------------------
// Landing
// ---------------------------------------------------------------------------

/// Both feet where a landing puts them: lead left, trail right, a little
/// narrower than the stance because a body coming down gathers its feet.
///
/// One pair of targets for every key of both landings, with only the ankle
/// heights changing, so the feet do not slide about underneath a body that is
/// moving a long way over them. `heel` is how far each ankle has come up off
/// its sole; `toe_floor_*` then works out the angle that keeps the toe on the
/// floor, which is not a number anybody can hold in their head and changes with
/// every centimetre of crouch.
///
/// The trailing foot stays close to underneath. Further back looks better
/// standing still and does not survive the crouch: a leg reaching back from
/// hips this low runs out of hip extension, the solver clamps it, and the foot
/// ends up somewhere nobody asked for -- through the floor, in the frames where
/// it is most visible.
fn planted(pose: Pose, heel: f32, trail_heel: f32) -> Pose {
    pose.plant_l([L - 0.04, GROUND + heel, 0.14])
        .plant_r([R + 0.04, GROUND + trail_heel, -0.04])
        .toe_floor_l()
        .toe_floor_r()
}

/// Touchdown on a short drop: the balls of both feet, knees already giving.
fn touchdown() -> Pose {
    let p = Pose::rest()
        .hips(0.0, -0.075, 0.02)
        .root(10.0, 0.0, -10.0)
        .spine(12.0, 0.0, 6.0)
        .chest(5.0, 0.0, 7.0)
        .head(-6.0, 0.0, 4.0)
        .shoulder_l(32.0, 30.0, 0.0)
        .elbow_l(42.0)
        .shoulder_r(24.0, 34.0, 0.0)
        .elbow_r(48.0);
    // Ball first, both feet: a drop this short is caught by the ankles before
    // the knees get to it.
    planted(p, 0.05, 0.05)
}

/// The bottom of a short landing. Heels down by now, hips under.
fn absorb() -> Pose {
    let p = Pose::rest()
        .hips(0.0, -0.165, 0.035)
        .root(14.0, 0.0, -10.0)
        .spine(20.0, 0.0, 6.0)
        .chest(8.0, 0.0, 7.0)
        .head(-12.0, 0.0, 4.0)
        .shoulder_l(42.0, 34.0, 0.0)
        .elbow_l(54.0)
        .shoulder_r(32.0, 38.0, 0.0)
        .elbow_r(60.0);
    planted(p, 0.0, 0.01)
}

/// On the way back up, guard rebuilding.
fn rebound() -> Pose {
    let p = Pose::rest()
        .hips(0.0, -0.085, 0.015)
        .root(6.0, 0.0, -11.0)
        .spine(10.0, 0.0, 6.0)
        .chest(4.0, 0.0, 7.0)
        .head(-5.0, 0.0, 4.0)
        .shoulder_l(20.0, 20.0, 0.0)
        .elbow_l(50.0)
        .shoulder_r(12.0, 22.0, 0.0)
        .elbow_r(46.0);
    planted(p, 0.0, 0.02)
}

fn land_soft() -> Recipe {
    Recipe {
        clip: Clip::LandSoft,
        looseness: Looseness::STRIDE,
        notes: "A short drop, absorbed and given straight back. Playback blends \
                out of this over its own length, as the square of how far \
                through it is, so the first three frames are the whole animation \
                and everything after frame six is mostly whatever the player is \
                already doing -- which is why the arrival is on frame zero and \
                the bottom of it on frame two. Both feet arrive on the ball and \
                roll flat, the knees give about a third of what the heavy \
                landing gives, and it recovers into the stance rather than into \
                a rest pose."
            .into(),
        keys: vec![
            Key::eased(0, touchdown(), Ease::STRIKE),
            Key::eased(2, absorb(), Ease::OUT),
            Key::eased(6, rebound(), Ease::SMOOTH),
            Key::eased(11, stance(), Ease::SMOOTH),
        ],
    }
}

/// A key part-way between two others, with the feet solved again afterwards.
///
/// The in-between keys of a landing are not decoration. A pose is *angles*, so
/// interpolating between two keys whose feet are both on the floor swings the
/// ankles along an arc and sinks them through it -- eight centimetres under, on
/// the frames where the character is lowest and most visible. Re-planting at a
/// key every two or three frames is the same fix the stride uses, for the same
/// reason.
fn between(a: Pose, b: Pose, t: f32, heel: f32, trail_heel: f32) -> Pose {
    planted(a.blend(&b, t), heel, trail_heel)
}

/// A real fall arriving. The legs take it, the torso folds over them, and the
/// arms are already going where they are needed.
fn slam() -> Pose {
    let p = Pose::rest()
        .hips(0.0, -0.135, 0.03)
        .root(16.0, 0.0, -8.0)
        .spine(18.0, 0.0, 5.0)
        .chest(8.0, 0.0, 6.0)
        .head(-4.0, 0.0, 3.0)
        .shoulder_l(46.0, 26.0, 0.0)
        .elbow_l(36.0)
        .shoulder_r(30.0, 34.0, 0.0)
        .elbow_r(42.0);
    planted(p, 0.06, 0.06)
}

/// The bottom: as low as the legs go, one hand on the floor.
fn bottom() -> Pose {
    let p = Pose::rest()
        // An arm is 55 cm and a shoulder is a long way off the floor, so the
        // hand only gets down there if the body takes it: hips as low as the
        // legs go, folded forward, and **leaning over the hand**. Folding
        // forward alone leaves the fingers twenty centimetres short and the
        // face out in front of the knees, which reads as a stumble rather than
        // a landing.
        .hips(-0.03, -0.38, 0.05)
        .root(21.0, 0.0, -4.0)
        .spine(35.0, 0.0, 6.0)
        .chest(8.0, 0.0, 8.0)
        // Eyes off the floor and back out at the arena, which is the difference
        // between a fighter who has landed and one who has fallen over.
        .head(-34.0, -12.0, 8.0)
        // The hand is the whole reason this clip is separate from the soft one.
        // In toward the middle rather than out wide, so it makes a tripod with
        // the two feet instead of a fourth leg.
        .reach_l([-0.26, -0.10, 0.34])
        .wrist_l(-40.0, 0.0, 0.0)
        // The other arm goes back and out as the counterweight. Both arms down
        // would be a crawl.
        .shoulder_r(-28.0, 46.0, 0.0)
        .elbow_r(30.0);
    // Both heels well up. An ankle has thirty degrees of dorsiflexion, so a
    // squat this deep with the soles flat is a squat with its toes through the
    // floor; what a real one does is come up onto the balls of the feet.
    planted(p, 0.07, 0.11)
}

/// Still down, but the weight is back over the feet and the hand has left the
/// floor.
fn pushing_up() -> Pose {
    let p = Pose::rest()
        .hips(0.0, -0.255, 0.045)
        .root(16.0, 0.0, -8.0)
        .spine(24.0, 0.0, 7.0)
        .chest(10.0, 0.0, 8.0)
        .head(-18.0, 0.0, 4.0)
        .shoulder_l(52.0, 30.0, 0.0)
        .elbow_l(66.0)
        .shoulder_r(-4.0, 38.0, 0.0)
        .elbow_r(40.0);
    planted(p, 0.01, 0.05)
}

/// Most of the way up, guard coming back before the feet finish.
fn standing_up() -> Pose {
    let p = Pose::rest()
        .hips(0.0, -0.125, 0.02)
        .root(9.0, 0.0, -10.0)
        .spine(14.0, 0.0, 6.0)
        .chest(6.0, 0.0, 7.0)
        .head(-7.0, 0.0, 4.0)
        .shoulder_l(24.0, 20.0, 0.0)
        .elbow_l(56.0)
        .shoulder_r(8.0, 24.0, 0.0)
        .elbow_r(46.0);
    planted(p, 0.0, 0.02)
}

fn land_heavy() -> Recipe {
    Recipe {
        clip: Clip::LandHeavy,
        looseness: Looseness::STRIDE,
        notes: "A real fall arriving -- anything over twenty-six frames of \
                flight, which is most jumps this game makes. The shock is spent \
                in four frames: the balls of the feet, then the legs fold as far \
                as they go and a hand goes down to the floor, which is the one \
                thing that separates this from the soft landing at a glance. \
                Recovery is slow and deliberately outlasts its own weight in the \
                blend -- playback fades this out as the square of how far \
                through it is, so by the time the character is standing the \
                player has had the controls back for a while and only sees the \
                last of it underneath what they are doing."
            .into(),
        keys: vec![
            Key::eased(0, slam(), Ease::STRIKE),
            Key::eased(2, between(slam(), bottom(), 0.55, 0.09, 0.13), Ease::OUT),
            Key::eased(4, bottom(), Ease::OUT),
            Key::eased(
                7,
                between(bottom(), pushing_up(), 0.3, 0.05, 0.09),
                Ease::SMOOTH,
            ),
            Key::eased(11, pushing_up(), Ease::SMOOTH),
            Key::eased(15, standing_up(), Ease::SMOOTH),
            Key::eased(19, stance(), Ease::SMOOTH),
        ],
    }
}
