//! The Bulwark: Bash, Slam, Grapple.
//!
//! GENERATED-FRIENDLY: the animation hub (F9) rewrites this file when it saves,
//! in the same shape you would write by hand. Editing it by hand is fine and
//! expected; keep the prose in each recipe's `notes` field, because ordinary
//! comments do not survive a save from the hub.
//!
//! ## What the class fights like
//!
//! It denies space. Not by being slow -- `bulwark.md` is emphatic that slow is
//! the failure mode of this archetype -- but by being *committed*: normal
//! movement speed, long active and recovery windows, enormous payoff. What the
//! opponent is reading, every second of a round, is whether the wall is about
//! to move and how expensive it would be if it did.
//!
//! The body says the same thing before any move does. Short, wide, thick,
//! short-limbed: an arm on this build is 51 cm and a leg is 80, so nothing here
//! can be thrown with the arms and reach anybody. Every one of these three
//! moves is driven from the hips and the legs, and the arms are along for the
//! ride -- which is also why the three read as the same fighter rather than as
//! three animations that happen to share a skeleton.
//!
//! The three occupy three different distances and three different heights,
//! because that is what an opponent has to tell apart across the arena:
//!
//! - **Bash** goes forward at chest height and barely leaves the stance. Four
//!   frames of startup, thrown constantly, and it keeps sixty per cent of
//!   walking speed -- so it is the one clip here whose legs the renderer is
//!   allowed to replace with a walk cycle, and they are authored to be worth
//!   replacing rather than to carry the move.
//! - **Slam** goes *up* and then straight down into the floor. It is the only
//!   time a short character gets tall, which is the whole telegraph.
//! - **Grapple** goes forward and low, with the guard deliberately thrown open,
//!   and ends holding somebody at arm's length.
//!
//! ## The shield is a real object
//!
//! It hangs off the left hand and follows the forearm, the way a strapped
//! shield does, so the left arm is the shield arm in all three clips and
//! wherever that hand goes the shield goes with it. Two consequences that are
//! not obvious until you look at the game rather than at the contact sheet:
//!
//! - A strapped shield's face is **perpendicular to the forearm**, and which
//!   perpendicular is decided by the forearm's roll. Keying that by hand means
//!   discovering, one pose at a time, that the shield is edge-on to the person
//!   it is meant to be covering. `shield()` below solves the roll instead: say
//!   where the hand goes and which way the face looks, and the arithmetic finds
//!   the twist.
//! - The shield sits about 17 cm beyond the wrist, along the forearm. A hand
//!   placed where the shield should be puts the shield past it, so every target
//!   in this file is a hand position with that already taken off.
//!
//! ## Frames
//!
//! Every key is derived from `clip.phases()` -- the last startup frame, the
//! first active frame, the first recovery frame -- read live from the move
//! table. The contact pose sits on the first active frame because that is the
//! frame the hitbox appears. Grapple goes one further and reads the hold length
//! out of the move table too, so the frame the arms let go follows the frame
//! the simulation stops dragging somebody around.

use crate::bake::{Feel, Key, Looseness, Recipe};
use crate::clips::locomotion::stance;
use crate::ease::Ease;
use view::clips::Clip;
use view::ik;
use view::math::{self, Quat, V3};
use view::pose::{ANKLE_ON_GROUND as GROUND, Pose, reference};
use view::skeleton::{Joint, solve};

pub fn clips() -> Vec<Recipe> {
    vec![bash(), slam(), grapple()]
}

// ---------------------------------------------------------------------------
// The two spots the fighter stands on
// ---------------------------------------------------------------------------

/// Both feet on the footprint the fighter is already standing on. `lead` and
/// `rear` move each ankle up or down from where the idle leaves it, in metres.
///
/// The two spots -- and the height of each -- are read out of
/// `locomotion::stance` rather than typed again, for the reason `defence` reads
/// them: a move thrown out of neutral that stands somewhere else has moved the
/// feet before it has done anything, and the Bulwark of all classes cannot
/// afford to look like it shuffled. Zero and zero is the idle's own footing,
/// rear heel included; `-0.032` on the rear is that heel coming down flat,
/// which is the one thing planting for a slam actually does to the feet.
///
/// An ankle above the floor is rolled down onto its toe and one on the floor is
/// levelled, and neither angle is authored. Pointing a toe whose ankle is still
/// at floor height drives the front of the foot straight through the ground,
/// and levelling a foot whose ankle has been raised leaves it hovering; letting
/// the kinematics pick between the two is the difference between a stance that
/// stands on the floor and one that stands near it.
fn stand(pose: Pose, lead: f32, rear: f32) -> Pose {
    let spot = |j: Joint, lift: f32| {
        let p = stance().joint_at(j);
        [p[0], (p[1] + lift).max(GROUND), p[2]]
    };
    let (l, r) = (spot(Joint::FootL, lead), spot(Joint::FootR, rear));
    let pose = pose.plant_l(l).plant_r(r);
    let pose = if l[1] > GROUND + 0.002 {
        pose.toe_floor_l()
    } else {
        pose.toe_l(0.0)
    };
    if r[1] > GROUND + 0.002 {
        pose.toe_floor_r()
    } else {
        pose.toe_r(0.0)
    }
}

// ---------------------------------------------------------------------------
// The shield
// ---------------------------------------------------------------------------

/// Where the shield elbow goes: out to the side and down, never back.
///
/// This is the one number in the file that decides whether the class has a
/// shield or a plank. `ik::hand_to` tucks an elbow *backwards*, which is right
/// for a punch and wrong here: an elbow behind the hand points the forearm
/// forwards, and a face perpendicular to a forward-pointing forearm is a shield
/// turned edge-on to the person it is supposed to be covering. No amount of
/// roll fixes that, because roll spins about the bone. Carrying a shield is
/// what a shoulder does *instead* -- the elbow swings out under it and the
/// forearm lies across the body -- so that is the solution this file asks the
/// solver for.
const SHIELD_ELBOW: V3 = [-1.0, -0.45, 0.0];

/// Put the shield hand at `at` and roll the forearm so the shield's face looks
/// along `look`. Both in character space, metres.
///
/// The roll is solved rather than authored because it is arithmetic, and
/// because getting it wrong is invisible in a pose and glaring in the game. The
/// twist channel is applied innermost, so it spins the forearm about its own
/// length and moves the hand not at all -- which means where the shield is and
/// where it looks are genuinely independent knobs, and the solve is one `atan2`
/// rather than a search.
///
/// A forearm can only roll so far, and the limit is honest: the face gets as
/// close to `look` as the joint allows and stops. It also cannot do the
/// impossible. The face is perpendicular to the bone, so an arm reaching
/// straight out in front cannot hold the shield facing forward at all, however
/// it rolls, and asking anyway gets the nearest legal answer rather than an
/// error. Where a pose needs the face on the line, move the hand across the
/// body until the forearm is across it too.
///
/// Call it after the torso is posed: a hand's target is measured from a
/// shoulder the spine and chest have already moved.
fn shield(pose: Pose, at: V3, look: V3) -> Pose {
    let skeleton = reference();
    let mut pose = pose;
    let _ = ik::reach(&mut pose, skeleton, Joint::ArmL, at, SHIELD_ELBOW);
    let bone = skeleton.bone(Joint::ForearmL);
    let bend = pose.angles(Joint::ForearmL)[0];

    // The forearm's frame before its own roll is applied, which is the frame
    // the roll turns in.
    let base =
        solve(skeleton, &pose).rot[Joint::ArmL.index()].mul(Quat::from_x(bend * bone.swing_sign));
    let side = base.rotate([1.0, 0.0, 0.0]);
    let face = base.rotate([0.0, 0.0, 1.0]);
    let d = math::normalize_or(look, [0.0, 0.0, 1.0]);

    // The face traces a circle as the forearm rolls; this is the point on it
    // that lies nearest `look`. The mirror on the left side is why the first
    // term is negative.
    let roll = (-math::dot(side, d)).atan2(math::dot(face, d));
    let (lo, hi) = bone.limits.twist;
    let forearm = roll.clamp(lo, hi);
    pose.set_angles(Joint::ForearmL, [bend, 0.0, forearm]);

    // A forearm runs out of roll long before a shield runs out of angles it has
    // to be carried at, and the wrist has another twenty-five degrees of the
    // same rotation. Spending them here rather than authoring them puts the
    // overflow in the one joint that can still take it, instead of leaving the
    // face short by exactly the amount nobody remembered to add.
    let (wrist_lo, wrist_hi) = skeleton.bone(Joint::HandL).limits.twist;
    let mut wrist = pose.angles(Joint::HandL);
    wrist[2] = (roll - forearm).clamp(wrist_lo, wrist_hi);
    pose.set_angles(Joint::HandL, wrist);
    pose
}

// ---------------------------------------------------------------------------
// Weight
// ---------------------------------------------------------------------------

/// The Bulwark's own weight: heavy hips, spine and legs, and a shield arm that
/// does not ring.
///
/// None of the six presets is this shape, and the reason is the shield.
/// `HEAVY` puts three and a half frames of lag on an arm and seven on a hand,
/// which is right for a blade on the end of a swing and wrong for half a metre
/// of strapped timber. The arm is not carrying the shield, it *is* the
/// mounting: it does not whip and it does not wobble, and an overhead whose
/// shield reaches the floor five frames after the hitbox -- which is what
/// `HEAVY` actually produced here -- teaches both players the wrong timing.
///
/// So the ring lives where the mass is. The hips, the spine and the legs are as
/// slow to settle as anything in the game, the head carries further than any of
/// them, and the arms are held nearly as tight as they are in the poke. That is
/// also the thesis of the whole file said in six numbers: this class moves from
/// the ground up, and the arms are the last thing doing any of the work.
const BRACED_WEIGHT: Looseness = Looseness {
    root: Feel::new(1.8, 0.95),
    spine: Feel::new(2.2, 0.85),
    chest: Feel::new(2.4, 0.75),
    head: Feel::new(3.0, 0.55),
    arms: Feel::new(1.4, 0.72),
    legs: Feel::new(1.2, 0.88),
};

// ---------------------------------------------------------------------------
// The stance everything is thrown from
// ---------------------------------------------------------------------------

/// Braced: hips low and back, knees loaded, shield across the centre line, the
/// off hand cocked at the ribs.
///
/// Every clip in this file opens and closes on it, so that a move thrown out of
/// neutral and a move ending back in neutral both start and finish somewhere
/// the player recognises.
///
/// Bladed onto the shield side, but only about half as far as the swordsman's
/// guard in `defence`. A wall presents its width. Turn this body far enough to
/// hide behind and the shield stops covering the middle of it, which is the one
/// thing the shield is for.
fn braced() -> Pose {
    let body = Pose::rest()
        .hips(0.0, -0.105, -0.015)
        .root(6.0, 0.0, 11.0)
        .spine(9.0, 0.0, 6.0)
        .chest(4.0, 0.0, 5.0)
        .head(-4.0, 0.0, -22.0)
        .wrist_l(0.0, 0.0, 0.0)
        .shoulder_r(-8.0, 14.0, 0.0)
        .elbow_r(98.0)
        .wrist_r(-12.0, 0.0, 0.0);
    stand(
        shield(body, [-0.155, 1.16, 0.30], [0.0, 0.0, 1.0]),
        0.0,
        0.0,
    )
}

// ---------------------------------------------------------------------------
// Frames, from the phase boundaries
// ---------------------------------------------------------------------------

/// The frame the telegraph lands on: two or three, whatever the startup is.
///
/// The opponent spends the startup deciding, and what they decide from is the
/// first change in the silhouette. Later than frame three and a four-frame poke
/// has no readable startup at all; earlier than two and the springs have not
/// moved anything yet.
fn tell(windup: u16) -> u16 {
    (windup / 3).clamp(2, 3)
}

/// A frame a fraction of the way from one phase boundary to another, so that
/// every key in this file follows the move table rather than a typed-in number.
fn part(a: u16, b: u16, k: f32) -> u16 {
    a + ((b.saturating_sub(a)) as f32 * k).round() as u16
}

/// Keys in the order they were added, dropping any that a retune has collapsed
/// onto an earlier one.
///
/// Frames here are derived, so a move tuned down to a three-frame startup can
/// land the telegraph on the same frame as the wind-up. Dropping the later of
/// the two keeps the contact key where the hitbox is, which is the one key that
/// must not move.
struct Score(Vec<Key>);

impl Score {
    fn new() -> Score {
        Score(Vec::new())
    }

    fn key(&mut self, frame: u16, pose: Pose, ease: Ease) {
        if self.0.last().is_some_and(|k| frame <= k.frame) {
            return;
        }
        self.0.push(Key::eased(frame, pose, ease));
    }
}

// ---------------------------------------------------------------------------
// Bash -- the poke
// ---------------------------------------------------------------------------

/// A short shield punch: the whole body behind a forearm that barely moves.
fn bash() -> Recipe {
    let clip = Clip::BulwarkPoke;
    let (windup, contact, through) = clip.phases().expect("bash animates a move");
    let last = clip.length().saturating_sub(1);

    // Loaded: the hips sink and turn away a few degrees, the shield draws back
    // onto the chest, the rear leg takes the weight and the lead heel comes
    // off. One frame of gathering, which is all a four-frame startup has room
    // for, and it is the whole telegraph.
    let load = {
        let body = Pose::rest()
            .hips(0.032, -0.128, -0.058)
            .root(3.0, 0.0, 22.0)
            .spine(11.0, 0.0, 10.0)
            .chest(5.0, 0.0, 8.0)
            .head(-3.0, 0.0, -32.0)
            .wrist_l(0.0, 0.0, 0.0)
            .shoulder_r(-16.0, 16.0, 0.0)
            .elbow_r(112.0)
            .wrist_r(-12.0, 0.0, 0.0);
        stand(
            shield(body, [-0.150, 1.14, 0.19], [0.0, 0.0, 1.0]),
            0.030,
            0.0,
        )
    };

    // Contact. The shield is 31 cm further forward than it was and the hips are
    // 18 cm further forward than *they* were: the arm has hardly extended, and
    // that is the move. A short-limbed fighter who punches with the arm reaches
    // nobody, so the shoulder is driven through by the legs and the elbow stays
    // shut behind it.
    let punch = {
        let body = Pose::rest()
            .hips(-0.022, -0.112, 0.122)
            .root(9.0, 0.0, -6.0)
            .spine(12.0, 0.0, -6.0)
            .chest(6.0, 0.0, -8.0)
            .head(-6.0, 0.0, 4.0)
            .wrist_l(0.0, 0.0, 0.0)
            .shoulder_r(-46.0, 18.0, 0.0)
            .elbow_r(134.0)
            .wrist_r(-14.0, 0.0, 0.0);
        stand(
            shield(body, [-0.100, 1.19, 0.50], [0.0, 0.0, 1.0]),
            0.0,
            0.055,
        )
    };

    // The end of the push. The shield has stopped, the arm has given a little
    // and the body is still out over the lead foot -- which is the part of a
    // poke that is minus on block, and it should be visible that it is.
    let spent = {
        let body = Pose::rest()
            .hips(-0.032, -0.128, 0.104)
            .root(11.0, 0.0, -2.0)
            .spine(13.0, 0.0, -2.0)
            .chest(6.0, 0.0, -4.0)
            .head(-8.0, 0.0, 0.0)
            .wrist_l(0.0, 0.0, 0.0)
            .shoulder_r(-40.0, 17.0, 0.0)
            .elbow_r(128.0)
            .wrist_r(-14.0, 0.0, 0.0);
        stand(
            shield(body, [-0.120, 1.15, 0.46], [0.0, 0.0, 1.0]),
            0.0,
            0.045,
        )
    };

    // Hauling the shield back onto the line before anything else recovers,
    // because the line is what the class is protecting.
    let recover = {
        let body = Pose::rest()
            .hips(0.010, -0.115, 0.020)
            .root(7.0, 0.0, 6.0)
            .spine(10.0, 0.0, 3.0)
            .chest(5.0, 0.0, 2.0)
            .head(-5.0, 0.0, -12.0)
            .wrist_l(0.0, 0.0, 0.0)
            .shoulder_r(-12.0, 15.0, 0.0)
            .elbow_r(104.0)
            .wrist_r(-12.0, 0.0, 0.0);
        stand(
            shield(body, [-0.150, 1.16, 0.33], [0.0, 0.0, 1.0]),
            0.0,
            0.010,
        )
    };

    let mut score = Score::new();
    score.key(0, braced(), Ease::OUT);
    score.key(tell(windup), load, Ease::OUT);
    score.key(contact, punch, Ease::STRIKE);
    score.key(through, spent, Ease::OUT);
    score.key(part(through, last, 0.45), recover, Ease::SMOOTH);
    score.key(last, braced(), Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "Four frames of startup, thrown all round. The arm barely \
                extends: an arm on this build is half a metre, so a shield punch \
                thrown with the elbow reaches nobody, and what actually travels \
                is the hips -- eighteen centimetres of them, through a shoulder \
                that stays shut. That is the same engine the other two moves \
                run on, at the smallest size it comes in. Both keys either side \
                of the contact leave at full speed rather than easing, because \
                four frames is not long enough to spend any of them \
                accelerating, and because a spring given a gentle target arrives \
                after the hitbox has already come and gone. It keeps sixty per \
                cent of walking speed, so the renderer swaps these legs for a \
                walk cycle whenever the player is moving: they are authored as a \
                weight transfer over the two spots the fighter already stands \
                on, which is a thing a walk can replace without the poke falling \
                apart. Crisp rather than heavy -- a poke that rings is a poke \
                you cannot throw twice."
            .into(),
        keys: score.0,
    }
}

// ---------------------------------------------------------------------------
// Slam -- the committed move
// ---------------------------------------------------------------------------

/// The overhead: the shield goes up over the head and comes down into the
/// floor, with the whole body dropped behind it.
fn slam() -> Recipe {
    let clip = Clip::BulwarkCommitted;
    let (windup, contact, through) = clip.phases().expect("slam animates a move");
    let last = clip.length().saturating_sub(1);

    // The first read, on frame three: the bladed stance unwinds, the fighter
    // squares up, the rear heel comes down and the shield starts up off the
    // line.
    // Getting wider is a silhouette change nothing else in the kit makes, and
    // it costs the feet nothing.
    let set = {
        let body = Pose::rest()
            .hips(0.0, -0.145, -0.030)
            .root(4.0, 0.0, 2.0)
            .spine(6.0, 0.0, 0.0)
            .chest(3.0, 0.0, 0.0)
            .head(-6.0, 0.0, -4.0)
            .wrist_l(0.0, 0.0, 0.0)
            .shoulder_r(46.0, 24.0, 0.0)
            .elbow_r(78.0)
            .wrist_r(-8.0, 0.0, 0.0);
        stand(
            shield(body, [-0.175, 1.36, 0.22], [0.0, 0.0, 1.0]),
            0.0,
            -0.020,
        )
    };

    // The top of the wind, and the second read. Up on both toes, spine arched
    // back, shield a clear half metre above the head with the off hand up
    // beside it. This is the only frame in the game where the Bulwark is taller
    // than the Champion, and the opponent gets three of them to look at it.
    let high = {
        let body = Pose::rest()
            .hips(0.0, 0.020, -0.050)
            .root(-10.0, 0.0, 6.0)
            .spine(-14.0, 0.0, 4.0)
            .chest(-8.0, 0.0, 4.0)
            .head(-16.0, 0.0, -6.0)
            .wrist_l(0.0, 0.0, 0.0)
            .shoulder_r(108.0, 30.0, 0.0)
            .elbow_r(50.0)
            .wrist_r(-10.0, 0.0, 0.0);
        stand(
            shield(body, [-0.150, 1.74, -0.02], [0.0, 0.15, 0.99]),
            0.050,
            0.030,
        )
    };

    // The last of the rise, three frames later. Almost the same pose, which is
    // the point: what the opponent sees is a held shape rather than a body
    // still deciding, and the two keys are what turn an ease into a wait.
    let peak = {
        let body = Pose::rest()
            .hips(0.0, 0.024, -0.056)
            .root(-12.0, 0.0, 7.0)
            .spine(-16.0, 0.0, 5.0)
            .chest(-9.0, 0.0, 5.0)
            .head(-18.0, 0.0, -6.0)
            .wrist_l(0.0, 0.0, 0.0)
            .shoulder_r(114.0, 30.0, 0.0)
            .elbow_r(46.0)
            .wrist_r(-10.0, 0.0, 0.0);
        stand(
            shield(body, [-0.140, 1.78, -0.04], [0.0, 0.18, 0.98]),
            0.056,
            0.036,
        )
    };

    // Over the top and falling. The torso passes through upright on its way
    // forward, the heels come back down, and the shield is at head height with
    // a third of the drop spent.
    let fall = {
        let body = Pose::rest()
            .hips(0.0, -0.052, -0.028)
            .root(11.0, 0.0, 5.0)
            .spine(11.0, 0.0, 4.0)
            .chest(5.0, 0.0, 3.0)
            .head(-10.0, 0.0, -6.0)
            .wrist_l(0.0, 0.0, 0.0)
            .shoulder_r(88.0, 28.0, 0.0)
            .elbow_r(50.0)
            .wrist_r(-8.0, 0.0, 0.0);
        stand(
            shield(body, [-0.195, 1.48, 0.26], [0.0, 0.10, 0.99]),
            0.014,
            0.008,
        )
    };

    // Committed and coming down, with the heels back on the floor and the body
    // folding at the waist. Two thirds of the drop is spent by here, which is
    // what keeps the last third inside what a shoulder can do in a frame.
    let drive = {
        let body = Pose::rest()
            .hips(0.0, -0.150, 0.025)
            .root(20.0, 0.0, 4.0)
            .spine(20.0, 0.0, 2.0)
            .chest(9.0, 0.0, 2.0)
            .head(-4.0, 0.0, -4.0)
            .wrist_l(0.0, 0.0, 0.0)
            .shoulder_r(76.0, 24.0, 0.0)
            .elbow_r(56.0)
            .wrist_r(-6.0, 0.0, 0.0);
        stand(
            shield(body, [-0.230, 1.05, 0.46], [0.0, 0.05, 1.0]),
            0.0,
            0.0,
        )
    };

    // Contact: the bottom edge of the shield in the dirt, the arm straight, the
    // knees folded to ninety degrees and the off hand down beside it. The face
    // stays turned at the opponent rather than at the floor, because a shield
    // driven down flat is a shield that has stopped being a wall -- and this is
    // the frame the class is named after.
    let ground = {
        let body = Pose::rest()
            .hips(0.0, -0.255, -0.010)
            .root(30.0, 0.0, 4.0)
            .spine(24.0, 0.0, 2.0)
            .chest(9.0, 0.0, 2.0)
            .head(-28.0, 0.0, -4.0)
            .wrist_l(0.0, 0.0, 0.0)
            .wrist_r(-4.0, 0.0, 0.0);
        stand(
            shield(body, [-0.245, 0.56, 0.52], [0.0, -0.15, 0.99]).reach_r([0.230, 0.54, 0.50]),
            0.020,
            0.020,
        )
    };

    // The bottom of it, four frames later. Everything has arrived and the body
    // has compressed a little further onto the shield: the ground stops the
    // shield and the fighter keeps going, which is the whole of what
    // follow-through means on a move like this.
    let impact = {
        let body = Pose::rest()
            .hips(0.0, -0.285, -0.005)
            .root(34.0, 0.0, 4.0)
            .spine(27.0, 0.0, 2.0)
            .chest(11.0, 0.0, 2.0)
            .head(-22.0, 0.0, -4.0)
            .wrist_l(0.0, 0.0, 0.0)
            .wrist_r(-2.0, 0.0, 0.0);
        stand(
            shield(body, [-0.250, 0.49, 0.54], [0.0, -0.20, 0.98]).reach_r([0.235, 0.47, 0.52]),
            0.030,
            0.030,
        )
    };

    // Levering out of it. The hips come back over the feet first and the shield
    // is still nearly on the floor -- the arm is the last thing to leave,
    // because the shield is heavy and this is the twenty-four frames the
    // opponent is supposed to be punishing.
    let heave = {
        let body = Pose::rest()
            .hips(0.0, -0.225, -0.030)
            .root(26.0, 0.0, 6.0)
            .spine(22.0, 0.0, 4.0)
            .chest(10.0, 0.0, 3.0)
            .head(-18.0, 0.0, -8.0)
            .wrist_l(0.0, 0.0, 0.0)
            .shoulder_r(-6.0, 20.0, 0.0)
            .elbow_r(56.0)
            .wrist_r(-8.0, 0.0, 0.0);
        stand(
            shield(body, [-0.240, 0.78, 0.40], [0.0, -0.20, 0.98]),
            0.020,
            0.010,
        )
    };

    // Standing back up, the shield swinging in toward the line but still low
    // and still late.
    let rise = {
        let body = Pose::rest()
            .hips(0.0, -0.150, -0.020)
            .root(14.0, 0.0, 8.0)
            .spine(15.0, 0.0, 5.0)
            .chest(7.0, 0.0, 4.0)
            .head(-6.0, 0.0, -14.0)
            .wrist_l(0.0, 0.0, 0.0)
            .shoulder_r(-10.0, 16.0, 0.0)
            .elbow_r(80.0)
            .wrist_r(-10.0, 0.0, 0.0);
        stand(
            shield(body, [-0.200, 1.00, 0.36], [0.0, -0.05, 1.0]),
            0.0,
            0.0,
        )
    };

    let mut score = Score::new();
    score.key(0, braced(), Ease::OUT);
    score.key(tell(windup), set, Ease::IN);
    score.key(part(tell(windup), windup, 0.3), high, Ease::HOLD);
    score.key(part(tell(windup), windup, 0.5), peak, Ease::IN);
    score.key(part(tell(windup), windup, 0.7), fall, Ease::LINEAR);
    score.key(part(tell(windup), windup, 0.85), drive, Ease::OUT);
    score.key(contact, ground, Ease::STRIKE);
    score.key(through, impact, Ease::OUT);
    score.key(part(through, last, 0.3), heave, Ease::SMOOTH);
    score.key(part(through, last, 0.62), rise, Ease::SMOOTH);
    score.key(last, braced(), Ease::SMOOTH);

    Recipe {
        clip,
        looseness: BRACED_WEIGHT,
        notes: "The overhead, and the only time a short character gets tall. \
                Fourteen frames of startup is long enough to hold something \
                back, so it is spent on two reads rather than on one long drift: \
                the fighter squares up and plants on frame three, then rises \
                onto both toes with the shield half a metre above the head and \
                waits there for three frames while the opponent decides. The \
                drop is keyed in three parts rather than one, because the shield \
                falls a metre and a half and a shoulder that covers that in two \
                frames is a teleport -- so the frames either side of contact \
                carry it and the pose at contact is the same pose it would have \
                been. The shield lands bottom edge first with its face \
                still turned at the opponent, because a shield driven down flat \
                has stopped being a wall and this is the frame the class is \
                named after. It \
                roots you, so nothing travels: both feet stay on the spots they \
                started on and every centimetre of the height comes out of the \
                ankles and the hips. Twenty-four frames of getting back up, with \
                the shield leaving the floor last, is the price."
            .into(),
        keys: score.0,
    }
}

// ---------------------------------------------------------------------------
// Grapple -- the special
// ---------------------------------------------------------------------------

/// A command grab: twenty frames of opening the guard, three of catching
/// somebody, and thirty of holding them there.
fn grapple() -> Recipe {
    let clip = Clip::BulwarkSpecial;
    let (windup, contact, through) = clip.phases().expect("grapple animates a move");
    let last = clip.length().saturating_sub(1);
    // How long the simulation pins the victim at arm's length, read from the
    // move table for the same reason the phases are: the arms have to let go on
    // the frame the hold ends, and that number is tuned in the Oven.
    let hold = sim::moves::get(sim::Class::Bulwark, 2).grabs;
    let lets_go = (contact + hold).clamp(through, last.saturating_sub(2));

    // The tell, and it is the guard coming off. The shield swings out and down
    // off the centre line, and everything it was covering is open. A move that beats
    // blocking outright ought to cost the person throwing it their own block,
    // and this is the frame where it does.
    let open = {
        let body = Pose::rest()
            .hips(0.020, -0.135, -0.055)
            .root(8.0, 0.0, 18.0)
            .spine(10.0, 0.0, 8.0)
            .chest(5.0, 0.0, 6.0)
            .head(-6.0, 0.0, -28.0)
            .wrist_l(0.0, 0.0, 0.0)
            .shoulder_r(-4.0, 26.0, 0.0)
            .elbow_r(74.0)
            .wrist_r(-6.0, 0.0, 0.0);
        stand(
            shield(body, [-0.290, 1.02, 0.16], [-0.55, 0.0, 0.84]),
            0.035,
            0.0,
        )
    };

    // Sunk into the rear leg with both hands low, open and wide, eyes up. Held
    // from frame ten to frame seventeen: twenty frames of startup is long
    // enough to genuinely wait in, and this is the pose the opponent is being
    // invited to dodge.
    let gather = {
        let body = Pose::rest()
            .hips(0.030, -0.195, -0.070)
            .root(16.0, 0.0, 14.0)
            .spine(14.0, 0.0, 6.0)
            .chest(4.0, 0.0, 4.0)
            .head(-16.0, 0.0, -22.0)
            .wrist_l(0.0, 0.0, 0.0)
            .wrist_r(-4.0, 0.0, 0.0);
        stand(
            shield(body, [-0.185, 0.92, 0.28], [-0.25, 0.20, 0.95]).reach_r([0.255, 0.90, 0.30]),
            0.055,
            0.0,
        )
    };

    // The rear leg starts driving and the hands begin to close from underneath.
    // Nothing has reached yet -- this is the body arriving before the arms do,
    // which is the only order that works on a build with arms this short.
    let drive = {
        let body = Pose::rest()
            .hips(0.010, -0.185, 0.040)
            .root(20.0, 0.0, 6.0)
            .spine(16.0, 0.0, 2.0)
            .chest(6.0, 0.0, 0.0)
            .head(-12.0, 0.0, -8.0)
            .wrist_l(0.0, 0.0, 0.0)
            .wrist_r(-2.0, 0.0, 0.0);
        stand(
            shield(body, [-0.130, 1.11, 0.44], [0.0, 0.05, 1.0]).reach_r([0.220, 1.11, 0.48]),
            0.0,
            0.030,
        )
    };

    // Contact, and the frame the hitbox appears. Both hands arrive together at
    // chest height with the shield's face flat against the middle of the target
    // -- the left forearm goes across the body rather than out along the reach,
    // which is the only arrangement in which a strapped shield ends up pointed
    // at the person it has just caught rather than edge-on to them.
    let seize = {
        let body = Pose::rest()
            .hips(-0.010, -0.150, 0.140)
            .root(8.0, 0.0, -2.0)
            .spine(9.0, 0.0, -4.0)
            .chest(4.0, 0.0, -4.0)
            .head(-6.0, 0.0, 2.0)
            .wrist_l(0.0, 0.0, 0.0)
            .wrist_r(4.0, 0.0, 0.0);
        stand(
            shield(body, [-0.105, 1.21, 0.52], [0.0, 0.0, 1.0]).reach_r([0.225, 1.21, 0.55]),
            0.0,
            0.055,
        )
    };

    // The last active frame, and the most committed shape in the file: both
    // arms out at the limit of a fifty-one centimetre reach with the fighter's
    // own weight still going forward behind them. The simulation pins the
    // victim at two body radii, which is exactly where these hands are.
    let caught = {
        let body = Pose::rest()
            .hips(-0.020, -0.135, 0.180)
            .root(6.0, 0.0, -4.0)
            .spine(8.0, 0.0, -5.0)
            .chest(4.0, 0.0, -5.0)
            .head(-4.0, 0.0, 4.0)
            .wrist_l(0.0, 0.0, 0.0)
            .wrist_r(6.0, 0.0, 0.0);
        stand(
            shield(body, [-0.100, 1.25, 0.58], [0.0, 0.0, 1.0]).reach_r([0.235, 1.25, 0.61]),
            0.0,
            0.070,
        )
    };

    // Taking the weight. The arms do not come back -- there is a person on the
    // end of them -- so the recovery happens underneath: the hips drop and go
    // back over the feet, the spine leans away from the load, the shoulders
    // stay out where they were.
    let bearing = {
        let body = Pose::rest()
            .hips(0.010, -0.190, 0.075)
            .root(-4.0, 0.0, -2.0)
            .spine(4.0, 0.0, -4.0)
            .chest(4.0, 0.0, -4.0)
            .head(-2.0, 0.0, 2.0)
            .wrist_l(0.0, 0.0, 0.0)
            .wrist_r(8.0, 0.0, 0.0);
        stand(
            shield(body, [-0.110, 1.27, 0.56], [0.0, 0.0, 1.0]).reach_r([0.240, 1.27, 0.58]),
            0.0,
            0.030,
        )
    };

    // Something on the end of your arms fights. The whole body answers it in
    // one direction and then the other, from the legs, and the hands stay
    // where the victim is because the victim is not going anywhere.
    let wrench = {
        let body = Pose::rest()
            .hips(-0.030, -0.205, 0.085)
            .root(-6.0, 3.0, -8.0)
            .spine(6.0, -2.0, -4.0)
            .chest(3.0, -2.0, -4.0)
            .head(0.0, -4.0, 6.0)
            .wrist_l(0.0, 0.0, 0.0)
            .wrist_r(8.0, 0.0, 0.0);
        stand(
            shield(body, [-0.095, 1.22, 0.57], [0.0, 0.0, 1.0]).reach_r([0.250, 1.32, 0.55]),
            0.0,
            0.040,
        )
    };

    // And back the other way, a third of the hold later. The two keys are not
    // mirror images of each other -- a struggle that alternates evenly reads as
    // a machine -- so this one is squarer, higher on the hips and half a beat
    // shorter in the shoulders.
    let square = {
        let body = Pose::rest()
            .hips(0.020, -0.195, 0.070)
            .root(-2.0, -3.0, 0.0)
            .spine(5.0, 2.0, -2.0)
            .chest(4.0, 2.0, -2.0)
            .head(-2.0, 3.0, 0.0)
            .wrist_l(0.0, 0.0, 0.0)
            .wrist_r(8.0, 0.0, 0.0);
        stand(
            shield(body, [-0.085, 1.27, 0.47], [0.0, 0.0, 1.0]).reach_r([0.225, 1.22, 0.50]),
            0.0,
            0.020,
        )
    };

    // Letting go, on the frame the hold ends. The hands open, the shield rolls
    // back to face front, and the arms fall in toward the line.
    let release = {
        let body = Pose::rest()
            .hips(0.020, -0.165, 0.010)
            .root(4.0, 0.0, 6.0)
            .spine(8.0, 0.0, 2.0)
            .chest(5.0, 0.0, 2.0)
            .head(-4.0, 0.0, -8.0)
            .wrist_l(0.0, 0.0, 0.0)
            .shoulder_r(-2.0, 22.0, 0.0)
            .elbow_r(52.0)
            .wrist_r(-6.0, 0.0, 0.0);
        stand(
            shield(body, [-0.190, 1.12, 0.36], [0.0, 0.05, 1.0]),
            0.0,
            0.010,
        )
    };

    let mut score = Score::new();
    score.key(0, braced(), Ease::OUT);
    score.key(tell(windup), open, Ease::SMOOTH);
    score.key(part(tell(windup), windup, 0.45), gather, Ease::HOLD);
    score.key(part(tell(windup), windup, 0.85), drive, Ease::OUT);
    score.key(contact, seize, Ease::OUT);
    score.key(through.saturating_sub(1), caught, Ease::OUT);
    score.key(part(through, lets_go, 0.2), bearing, Ease::SMOOTH);
    score.key(part(through, lets_go, 0.5), wrench, Ease::SMOOTH);
    score.key(part(through, lets_go, 0.8), square, Ease::SMOOTH);
    score.key(lets_go, release, Ease::OUT);
    score.key(last, braced(), Ease::SMOOTH);

    Recipe {
        clip,
        looseness: BRACED_WEIGHT,
        notes: "Beats guard outright and loses to a dodge, so the twenty frames \
                of startup are spent advertising it. The shield comes off the \
                centre line on frame three -- a move that beats blocking costs \
                the thrower their own block, and that is the honest tell -- and \
                then the fighter sits down into a wide open-handed crouch and \
                stays there for seven, which is the pose a dodge is supposed to \
                be read out of. The catch is on the first active \
                frame and the committed two-handed reach on the last of the \
                three, at the limit of a fifty-one centimetre arm, which is \
                exactly where the simulation pins the victim: two body radii in \
                front. After that the arms stop being an animation. There is \
                somebody on the end of them for twenty-six frames -- read out of \
                the move table, so retuning the hold moves the frame the hands \
                open -- and the recovery happens underneath instead: hips back \
                over the feet, spine leaning away from the load, the whole body \
                answering a struggle from the legs while the hands stay where \
                the victim is. Nothing here comes back to guard until it lets \
                go, because a recovery that tidies the arms away is a recovery \
                that dropped somebody."
            .into(),
        keys: score.0,
    }
}
