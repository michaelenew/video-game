//! The Shadow Reaver's three moves: Slash, Executioner, Guillotine.
//!
//! ## What the class fights like
//!
//! The lightest body in the arena and the most mobile: `build_for` gives it
//! long limbs, narrow shoulders and a narrow stance, and the Oven gives it the
//! highest jump, the lowest gravity and nearly twice anybody's air steering.
//! Where the Bulwark plants and swings, the Reaver steps and cuts. So every
//! pose here is led from the hands and the shoulders rather than driven up out
//! of the hips, the stances are low and bladed rather than square, and nothing
//! is ever settled on both feet at once for longer than it has to be.
//!
//! Which is not the same as weightless. The three clips are deliberately three
//! different weights, because a class whose whole identity is options needs its
//! options to be told apart from across the arena:
//!
//! - **Slash** is a flick. Five frames of startup, and the blade is already
//!   carried high in the stance, so the wind-up is a gather rather than a
//!   swing. `CRISP`.
//! - **Executioner** is everything the class has, spent at once. Sixteen frames
//!   of raise, a hang at the top, and then the body falls behind the blade.
//!   `MARTIAL` -- see below for why not `HEAVY`.
//! - **Guillotine** is not a swing at all. It is a command.
//!
//! ## Guillotine is a command, and that decides its whole shape
//!
//! The damage does not happen at the caster. `reach` is **zero** and `radius`
//! is 1.6 -- the hitbox is a circle at the placed shadow, somewhere else on the
//! map. Animating this as a swing at the space in front of the caster would
//! teach the opponent to respect a threat that is not there and to ignore the
//! one that is, which is a worse outcome than no animation at all.
//!
//! So: the left hand -- the empty one, not the blade hand -- is driven out and
//! down along the line to the shadow, everything else clamps, and then the
//! whole body **holds**. No follow-through, because there is nothing here to
//! follow through into. The chin drops down the line of the arm, onto the piece
//! of floor the blades are about to come out of, which is the one place in the
//! Reaver's kit where the fighter deliberately looks at something other than
//! the person in front of them.
//!
//! It also separates cleanly from the other two by which arm throws it. Slash
//! and Executioner are the right hand; Guillotine is the left.
//!
//! ## Why the contact keys sit where they do
//!
//! Every key frame in this file is derived from `clip.phases()` -- the last
//! startup frame, the first active frame, the first recovery frame -- and never
//! typed in. The contact pose is keyed on the first active frame, which is the
//! frame the hitbox appears; retuning any of these three in the Oven moves the
//! animation with it instead of leaving it teaching the old timing.
//!
//! ## Why so much of this is LINEAR
//!
//! The motion ceiling is real and it bites hardest exactly here. A three-frame
//! gap carrying sixty degrees of shoulder is twenty degrees a frame if the ease
//! is linear and thirty-one if it is `SMOOTH`, because a smooth ease puts half
//! its travel in the middle frame. At these speeds the expressive eases are
//! affordable only where the deltas are small -- the gather, the hang, the long
//! recovery -- and the fast part of a cut has to be spent evenly or it is both
//! a pop and over the ceiling. The eases below are chosen on that basis and not
//! by taste, and where one is shaped it is because the gap is long enough or
//! the pose change small enough to pay for it.

use crate::bake::{Key, Looseness, Recipe};
use crate::clips::locomotion::stance;
use crate::ease::Ease;
use view::clips::Clip;
use view::pose::Pose;
use view::skeleton::Joint;

pub fn clips() -> Vec<Recipe> {
    vec![poke(), committed(), special()]
}

// ---------------------------------------------------------------------------
// The ground
// ---------------------------------------------------------------------------

/// Ankle to the back of the sole, and ankle to the front of it, in metres on
/// the reference body.
const HEEL: f32 = 0.06;
const TOE: f32 = 0.16;

/// How far a foot's ankle rises when the foot pivots on one end of the sole.
///
/// Rolling up onto the ball of the foot is a rotation *about the toe*, so the
/// ankle goes up. Tip a foot without raising its target and the pivot happens
/// about the ankle instead, which drives the toe into the floor -- and the
/// Reaver spends most of these three clips up on its back foot, so this is not
/// a rounding error here, it is eight centimetres.
fn pivot_lift(toe: f32) -> f32 {
    let lever = if toe < 0.0 { HEEL } else { TOE };
    lever * toe.abs().to_radians().sin()
}

/// Where the fighter is standing, taken out of the locomotion stance rather
/// than typed again.
///
/// Every clip in this file starts and ends on these two points. Attacks are cut
/// into with no blend, so a frame zero standing anywhere other than where
/// locomotion left the feet is a visible pop on every single attack.
fn ankle(j: Joint) -> [f32; 3] {
    stance().joint_at(j)
}

/// Plant both feet and say what each heel is doing.
///
/// Called at the end of every pose here. A foot target is measured from the
/// body, so a pose that drops the hips fifteen centimetres without re-planting
/// has dragged both feet fifteen centimetres across the floor -- and the
/// Executioner drops twenty-five.
///
/// **None of these three moves translates the feet.** Executioner and
/// Guillotine root you outright, and Slash leaves you sixty per cent of walking
/// speed, which the renderer already answers by blending the real locomotion
/// legs in over the top. An authored step in any of them would be a step the
/// simulation never took, and the feet would pay for it by sliding. So the
/// lunge lives in the hips and in the heels, both of which are honest.
fn footing(pose: Pose, toe_l: f32, toe_r: f32) -> Pose {
    let at = |j: Joint, toe: f32| {
        let p = ankle(j);
        [p[0], p[1] + pivot_lift(toe), p[2]]
    };
    pose.plant_l(at(Joint::FootL, toe_l))
        .plant_r(at(Joint::FootR, toe_r))
        .toe_l(toe_l)
        .toe_r(toe_r)
}

/// Frame zero of all three clips: exactly what locomotion left standing there.
fn ready() -> Pose {
    footing(stance(), 0.0, 12.0)
}

/// The three frames the move table hands this clip.
fn phases(clip: Clip) -> (u16, u16, u16) {
    clip.phases()
        .expect("a move clip takes its length from the move table")
}

// ---------------------------------------------------------------------------
// Slash -- the poke
// ---------------------------------------------------------------------------

/// Five frames of startup, three active, eleven of recovery.
///
/// A descending diagonal off the right shoulder, thrown from a stance that is
/// already carrying the blade high. Five frames is a third of a second for the
/// whole wind-up; there is no room for a swing back, so the startup gathers
/// rather than winds, and the reach comes from the torso unwinding rather than
/// from the arm travelling.
fn poke() -> Recipe {
    let clip = Clip::ReaverPoke;
    let (_, contact, follow) = phases(clip);
    let last = clip.length() - 1;
    // The gather is not a wind-up: it is where the blade already is, plus a
    // sink onto the back foot. Three frames before contact, so the whole thing
    // is inside the startup with a frame to spare on both sides.
    let gather = contact.saturating_sub(3).max(1);
    // Half the recovery is the blade coming back; the rest is standing up.
    let settle = follow + (last - follow) / 3;

    // The gather. The right hand comes up outside the right shoulder and the
    // chest winds away from the line, the hips sink four centimetres and rock
    // back onto the rear foot, and the lead heel comes off the floor. The
    // shoulders carry all of it -- the pelvis barely turns -- because the feet
    // are nailed down and every degree at the pelvis is a degree the legs have
    // to answer for.
    let cocked = footing(
        stance()
            .hips(0.020, -0.088, -0.040)
            .root(1.0, 2.0, 2.0)
            .spine(2.0, -2.0, 20.0)
            .chest(-3.0, 0.0, 22.0)
            // The body has turned forty-four degrees off the line and the head
            // gives thirty-four of it back. A fighter winding up is still
            // looking at the person they are winding up at.
            .head(-4.0, 3.0, -34.0)
            // Spread past ninety is what puts the hand *above* the shoulder
            // rather than out beside it, and that is the difference between a
            // diagonal cut and a horizontal one. Sixty-one degrees of it in two
            // frames is the fastest thing in the file and the reason the whole
            // clip is CRISP: a looser arm would still be on its way up when the
            // hitbox appeared.
            .shoulder_r(-16.0, 70.0, -18.0)
            .elbow_r(90.0)
            .wrist_r(-18.0, 0.0, -10.0)
            .shoulder_l(26.0, 18.0, -12.0)
            .elbow_l(88.0)
            .wrist_l(-10.0, 0.0, 0.0),
        -8.0,
        16.0,
    );

    // Contact, on the first active frame. The blade has come over the top and
    // is crossing the line at chest height with the arm nearly straight; the
    // hips have driven nine centimetres forward over the lead foot and the rear
    // heel is up and pushing. This is the blade *entering*, not the end of the
    // arc -- the hitbox is live for three frames and the cut keeps travelling
    // through all of them.
    let cut = footing(
        stance()
            .hips(-0.010, -0.086, 0.050)
            // The torso is only half unwound here. Putting the whole unwind on
            // the contact frame swings the hand across the midline before the
            // hitbox appears -- the blade arrives having already finished its
            // cut -- and it costs forty-six centimetres of hand travel in the
            // single frame before contact. Half at contact and the rest through
            // the active window is both the slower path and the truer one.
            .root(6.0, -2.0, -8.0)
            .spine(11.0, 2.0, -4.0)
            .chest(6.0, 0.0, -10.0)
            .head(-8.0, -2.0, 2.0)
            .shoulder_r(34.0, 52.0, 18.0)
            // Still half folded. The arm finishes straightening *through* the
            // active window rather than before it, which is both how a cut
            // actually extends and eight centimetres a frame off the hand's
            // speed on the way in.
            .elbow_r(60.0)
            .wrist_r(-8.0, 0.0, 6.0)
            .shoulder_l(2.0, 30.0, -6.0)
            .elbow_l(66.0)
            .wrist_l(-8.0, 0.0, 0.0),
        0.0,
        30.0,
    );

    // The end of the arc, on the first recovery frame. The blade has finished
    // low and across to the left, the chest has kept turning under it, and the
    // left arm has swung back and open as the counterweight -- which is where
    // the recovery's length comes from and why the move is minus on block.
    let through = footing(
        stance()
            .hips(-0.030, -0.104, 0.034)
            .root(10.0, -7.0, -26.0)
            .spine(18.0, 5.0, -16.0)
            .chest(10.0, 2.0, -26.0)
            .head(-6.0, -4.0, 22.0)
            .shoulder_r(40.0, -12.0, 44.0)
            .elbow_r(28.0)
            .wrist_r(-16.0, 0.0, 12.0)
            .shoulder_l(-14.0, 34.0, 0.0)
            .elbow_l(48.0)
            .wrist_l(-6.0, 0.0, 0.0),
        2.0,
        24.0,
    );

    // Gathering the blade back. The Reaver is light again well before it is
    // square again: the hands come home first and the hips follow.
    let gathering = footing(
        stance()
            .hips(0.006, -0.070, -0.010)
            .root(4.0, -2.0, -16.0)
            .spine(8.0, 1.0, -2.0)
            .chest(4.0, 0.0, -6.0)
            .head(-4.0, -1.0, 10.0)
            .shoulder_r(20.0, 26.0, 16.0)
            .elbow_r(66.0)
            .wrist_r(-10.0, 0.0, 4.0)
            .shoulder_l(6.0, 22.0, -4.0)
            .elbow_l(58.0)
            .wrist_l(-8.0, 0.0, 0.0),
        0.0,
        16.0,
    );

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "Slash is the neutral tool and gets thrown a hundred times a \
                round, so the whole clip is built around not rooting the \
                silhouette: frame zero is exactly the locomotion stance on its \
                own footing, the feet never leave their marks, and the lunge is \
                nine centimetres of hips over a rear heel that comes up. The \
                blade is already carried high in the stance, which is what makes \
                a five-frame startup honest -- the gather on frame two is a sink \
                and a wind of the chest, not a swing back the opponent could \
                have read anyway. The cut from the gather to contact is LINEAR, \
                and that is the one deliberate ugliness in the file: over three \
                frames a smooth ease puts half the arc in the middle frame, \
                which is thirty-one degrees of shoulder in one frame and reads \
                as a pop. Contact is the blade *entering* the line, not the end \
                of the arc -- the hitbox is live for three frames and the blade \
                travels through all of them, finishing low and left on the first \
                recovery frame. CRISP throughout: the arms still trail the \
                shoulders by a frame and a half, which is all the follow-through \
                a poke this fast can afford."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::SMOOTH),
            Key::eased(gather, cocked, Ease::LINEAR),
            Key::eased(contact, cut, Ease::LINEAR),
            Key::eased(follow, through, Ease::OUT),
            Key::eased(settle, gathering, Ease::SMOOTH),
            Key::eased(last, ready(), Ease::SMOOTH),
        ],
    }
}

// ---------------------------------------------------------------------------
// Executioner -- the commitment
// ---------------------------------------------------------------------------

/// Sixteen frames of startup, four active, twenty-six of recovery.
///
/// Two hands, a raise up the right side, a hang at the top, and then the body
/// falls behind the blade all the way to the floor. Forty-six frames is the
/// longest thing the class does and it is minus twelve on block, so the whole
/// point is that it is legible from the first fifth of it and expensive
/// afterwards.
///
/// The kit describes this as a blink upward and then a slash down. The Oven
/// gives it no `self_lift`, so the body never actually leaves the ground -- the
/// rise is a rise onto the ball of the back foot and a long spinal extension,
/// and the drop afterwards is twenty-five centimetres of hips. Animating a
/// blink the simulation does not do would put the hurtbox and the silhouette in
/// different places.
fn committed() -> Recipe {
    let clip = Clip::ReaverCommitted;
    let (_, contact, follow) = phases(clip);
    let last = clip.length() - 1;
    // Two numbers drive the whole layout, and both are distances rather than
    // tastes. The raise has to spend a hundred and fifty-odd degrees of
    // shoulder against a thirty-two-a-frame ceiling, so it needs seven frames;
    // and the blade falls a metre and a half, so the descent needs nine or the
    // hand crosses a third of a metre a frame. Six frames of startup left over
    // the top and the hang gets the rest.
    let top = contact.saturating_sub(6).max(3);
    let gather = (top / 3).max(2);
    let rising = gather + (top - gather) / 3;
    let high = gather + (top - gather) * 2 / 3;
    let hang = top + 2;
    let falls = contact.saturating_sub(2).max(hang + 1);
    // The blade reaches the floor one frame past the active window, then the
    // body is folded over it for a quarter of the recovery before it can start
    // standing up.
    let floor = follow + 1;
    let dead = floor + (last - floor) / 4;
    let standing = floor + (last - floor) * 5 / 8;

    // The gather. Both hands come together low and back at the right hip, the
    // knees fold and the hips drop eight centimetres, the chest winds hard
    // right. This is the frame the opponent reads: it is on frame three of a
    // sixteen-frame startup and it changes the silhouette from a bladed stance
    // to a crouched coil.
    let coiled = footing(
        stance()
            .hips(0.030, -0.135, -0.055)
            .root(6.0, 3.0, 8.0)
            .spine(10.0, -4.0, 16.0)
            .chest(2.0, 0.0, 18.0)
            .head(-6.0, 4.0, -30.0)
            .shoulder_r(-6.0, 22.0, -20.0)
            .elbow_r(62.0)
            .wrist_r(-18.0, 0.0, -10.0)
            .shoulder_l(-2.0, 30.0, -30.0)
            .elbow_l(96.0)
            .wrist_l(-14.0, 0.0, 0.0),
        -6.0,
        20.0,
    );

    // The blade leaves the hip and starts up the outside of the right shoulder.
    // Both arms go with it, because it is two hands on one hilt.
    let lifting = footing(
        stance()
            .hips(0.024, -0.112, -0.062)
            .root(2.0, 2.0, 10.0)
            .spine(6.0, -3.0, 18.0)
            .chest(0.0, 0.0, 20.0)
            .head(-10.0, 3.0, -30.0)
            .shoulder_r(37.0, 62.0, -6.0)
            .elbow_r(58.0)
            .wrist_r(-14.0, 0.0, -6.0)
            .shoulder_l(34.0, 52.0, -22.0)
            .elbow_l(92.0)
            .wrist_l(-12.0, 0.0, 0.0),
        -2.0,
        26.0,
    );

    // Past the shoulder and still climbing. The hips are coming back up under
    // it and the spine is beginning to open out of the coil.
    let climbing = footing(
        stance()
            .hips(0.012, -0.062, -0.060)
            .root(-2.0, 0.0, 6.0)
            .spine(-2.0, -1.0, 12.0)
            .chest(-4.0, 0.0, 12.0)
            .head(-16.0, 2.0, -22.0)
            .shoulder_r(80.0, 48.0, 4.0)
            .elbow_r(44.0)
            .wrist_r(-10.0, 0.0, -4.0)
            .shoulder_l(78.0, 44.0, -18.0)
            .elbow_l(68.0)
            .wrist_l(-8.0, 0.0, 0.0),
        6.0,
        32.0,
    );

    // The top. Arms as near straight up as the shoulder goes, spine arched back
    // so the blade is genuinely behind the head rather than in front of it --
    // the shoulder cannot swing past vertical, so the arch is the only thing
    // that puts the blade where an overhead's blade belongs. Hips at their
    // highest, back heel high, chin up along the blade.
    let overhead = footing(
        stance()
            .hips(0.000, 0.018, -0.040)
            .root(-7.0, 0.0, -2.0)
            .spine(-13.0, 0.0, 4.0)
            .chest(-8.0, 0.0, 4.0)
            .head(-22.0, 0.0, -8.0)
            // Past ninety degrees of swing the sign of spread flips, so the
            // wide numbers that opened the arms out on the way up now throw
            // them apart across the top of the head. Both come in to sixteen
            // and twenty, which is what two hands on one hilt looks like from
            // the front -- the contact sheet showed a clear X before this.
            .shoulder_r(144.0, 16.0, 8.0)
            .elbow_r(22.0)
            .wrist_r(-8.0, 0.0, 0.0)
            .shoulder_l(142.0, 20.0, -12.0)
            .elbow_l(34.0)
            .wrist_l(-8.0, 0.0, 0.0),
        16.0,
        40.0,
    );

    // The hang: two keyed frames that barely move, which under MARTIAL arms
    // running two frames behind reads as four or five frames of a body stopped
    // at the top of its own swing. That pause is the whole telegraph, and it is
    // what makes a move that does a hundred and eighty-five damage fair.
    //
    // It settles *forward*, not further back. An extra inch of wind at the top
    // was the first thing tried here and it cost the descent seven centimetres
    // of travel it could not afford -- the blade already falls a metre and a
    // half in nine frames. The weight starting to go over is the better read
    // anyway: the hang should look like the last instant before a fall, not
    // like a second wind-up.
    let hung = footing(
        stance()
            .hips(0.000, 0.006, -0.030)
            .root(-5.0, 0.0, -2.0)
            .spine(-9.0, 0.0, 5.0)
            .chest(-7.0, 0.0, 5.0)
            .head(-19.0, 0.0, -6.0)
            .shoulder_r(147.0, 15.0, 8.0)
            .elbow_r(20.0)
            .wrist_r(-6.0, 0.0, 0.0)
            .shoulder_l(145.0, 19.0, -12.0)
            .elbow_l(32.0)
            .wrist_l(-6.0, 0.0, 0.0),
        18.0,
        42.0,
    );

    // Two frames before contact, and the reason the descent is not one long
    // gap. Near vertical the hand is a metre and a half above the hips, so a
    // degree of spine is worth two and a half centimetres of hand and a degree
    // of shoulder is worth another one -- the same angular rate that is
    // comfortable at waist height crosses half a metre a frame up here. This
    // key spends only a third of the arc in the first third of the fall, which
    // is what a blade coming out of a hang actually does and what keeps the
    // hand under thirty centimetres a frame.
    let falling = footing(
        stance()
            .hips(0.000, -0.055, -0.006)
            .root(1.0, 0.0, -4.0)
            .spine(4.0, 0.0, 6.0)
            .chest(0.0, 0.0, 6.0)
            .head(-4.0, 0.0, -2.0)
            .shoulder_r(112.0, 20.0, 10.0)
            .elbow_r(26.0)
            .wrist_r(-8.0, 0.0, 2.0)
            .shoulder_l(110.0, 24.0, -11.0)
            .elbow_l(38.0)
            .wrist_l(-8.0, 0.0, 0.0),
        12.0,
        34.0,
    );

    // Contact, on the first active frame. The blade is coming down through head
    // height with the arms extended and the whole torso already folding after
    // it -- the body is falling behind the blade rather than lowering itself
    // politely behind it. The heels are both back on the floor because the
    // weight has arrived.
    let descending = footing(
        stance()
            .hips(0.000, -0.115, 0.030)
            // Only a fifth of the fold, on the frame the hitbox appears. The
            // fold is what the *floor* pose is for -- a body that has already
            // finished dropping when the blade is still at head height has
            // nothing left to fall with, and folding it here costs the hand a
            // further twenty centimetres a frame at the worst possible moment.
            .root(4.0, 0.0, -6.0)
            .spine(10.0, 0.0, 4.0)
            .chest(7.0, 0.0, 6.0)
            .head(2.0, 0.0, 0.0)
            .shoulder_r(84.0, 26.0, 12.0)
            .elbow_r(30.0)
            .wrist_r(-12.0, 0.0, 4.0)
            .shoulder_l(86.0, 30.0, -10.0)
            .elbow_l(42.0)
            .wrist_l(-10.0, 0.0, 0.0),
        0.0,
        22.0,
    );

    // The blade in the floor, one frame past the end of the active window. This
    // is the deepest pose in the file: eighty-odd degrees of forward fold over
    // hips a quarter of a metre below standing, which is the shape the whole
    // move exists to arrive at.
    let buried = footing(
        stance()
            .hips(0.000, -0.255, 0.018)
            .root(30.0, 0.0, -6.0)
            .spine(42.0, 0.0, 6.0)
            .chest(18.0, 0.0, 8.0)
            .head(2.0, 0.0, 0.0)
            .shoulder_r(24.0, 20.0, 16.0)
            .elbow_r(54.0)
            .wrist_r(-18.0, 0.0, 6.0)
            .shoulder_l(28.0, 26.0, -8.0)
            .elbow_l(66.0)
            .wrist_l(-14.0, 0.0, 0.0),
        -4.0,
        10.0,
    );

    // Nothing happens here on purpose. Twenty-six frames of recovery is a long
    // punish window and the pose that sits in it has to look like one: folded
    // over the blade, head down, weight all the way forward, and no guard
    // anywhere.
    let spent = footing(
        stance()
            .hips(0.006, -0.238, 0.006)
            .root(26.0, -2.0, -8.0)
            .spine(38.0, 1.0, 4.0)
            .chest(14.0, 0.0, 6.0)
            .head(-2.0, 0.0, 2.0)
            .shoulder_r(18.0, 24.0, 12.0)
            .elbow_r(64.0)
            .wrist_r(-16.0, 0.0, 4.0)
            .shoulder_l(22.0, 28.0, -6.0)
            .elbow_l(74.0)
            .wrist_l(-12.0, 0.0, 0.0),
        -2.0,
        12.0,
    );

    // Pushing back up out of it. The hips come back over the feet before the
    // shoulders come back up, which is the order a body actually stands out of
    // a deep fold and is also what keeps the whole recovery from reading as one
    // long even fade.
    let rising_up = footing(
        stance()
            .hips(0.010, -0.120, -0.020)
            .root(12.0, -2.0, -12.0)
            .spine(18.0, 1.0, 0.0)
            .chest(8.0, 0.0, 2.0)
            .head(-6.0, 0.0, 4.0)
            .shoulder_r(14.0, 22.0, 6.0)
            .elbow_r(58.0)
            .wrist_r(-12.0, 0.0, 2.0)
            .shoulder_l(18.0, 22.0, -4.0)
            .elbow_l(62.0)
            .wrist_l(-10.0, 0.0, 0.0),
        0.0,
        14.0,
    );

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "Everything the class has, spent in one cut. Sixteen frames of \
                startup is a long time to be readable for, so the gather lands \
                on frame three and changes the silhouette from a bladed stance \
                to a two-handed crouch, and the raise then runs continuously for \
                eight frames because a hundred and seventy degrees of shoulder \
                cannot be spent in fewer without breaking the motion ceiling. \
                MARTIAL, not HEAVY, and that is the one call in this file worth \
                arguing about: HEAVY runs the arms three and a half frames \
                behind the keys, which on a sixteen-frame startup means the \
                blade is still going up when the hitbox appears. Weight has to \
                read as follow-through, never as delay, so it lives in the shape \
                instead -- a hang keyed as a single almost-still frame at the \
                top, and twenty-five centimetres of hips falling after the blade \
                on the way down. The kit calls for a blink upward; the Oven \
                gives the move no self lift, so the rise is a spinal extension \
                and a back heel, and the body stays where its hurtbox is. The \
                recovery is deliberately dead: folded over a blade in the floor \
                with no guard anywhere, because minus twelve on block should \
                look like minus twelve on block."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::SMOOTH),
            Key::eased(gather, coiled, Ease::SMOOTH),
            Key::eased(rising, lifting, Ease::LINEAR),
            Key::eased(high, climbing, Ease::LINEAR),
            Key::eased(top, overhead, Ease::OUT),
            Key::eased(hang, hung, Ease::LINEAR),
            Key::eased(falls, falling, Ease::LINEAR),
            Key::eased(contact, descending, Ease::LINEAR),
            Key::eased(floor, buried, Ease::OUT),
            Key::eased(dead, spent, Ease::IN),
            Key::eased(standing, rising_up, Ease::SMOOTH),
            Key::eased(last, ready(), Ease::SMOOTH),
        ],
    }
}

// ---------------------------------------------------------------------------
// Guillotine -- the command
// ---------------------------------------------------------------------------

/// Nine frames of startup, five active, sixteen of recovery, and none of the
/// damage happens here.
///
/// The blades come up out of the placed shadow. `reach` is zero: there is no
/// hitbox on the caster at all. So this is a pointing gesture thrown hard and
/// then held -- compress, wind, snap the empty hand out along the line, and
/// stop dead for the whole five frames the blades are live.
///
/// The stopping is the animation. A follow-through would say the caster did
/// something to the space in front of them, and the whole read this clip has to
/// teach is that they did not.
fn special() -> Recipe {
    let clip = Clip::ReaverSpecial;
    let (_, command, release) = phases(clip);
    let last = clip.length() - 1;
    // Compress on frame two, wound and still by the middle of the startup, then
    // four frames of throw. The still frames are what make the throw read: with
    // no pause before it the arm merely travels, and a hundred degrees of elbow
    // in three frames is over the ceiling anyway.
    let drop = (command / 4).max(2);
    let wound = command.saturating_sub(4).max(drop + 1);
    // Held to the last active frame, then let go over the recovery.
    let held = release.saturating_sub(1).max(command + 1);
    let opening = release + (last - release) / 3;
    let square = release + (last - release) * 2 / 3;

    // The compression. Both hands come in to the chest, the hips drop eight
    // centimetres and back, the body winds away from the line it is about to
    // point down. It reads as a fighter making themselves small, which is not
    // what either of the other two moves does, and that is the point: the
    // opponent has to be able to tell in two frames that no blade is coming
    // at them.
    let compressed = footing(
        stance()
            .hips(0.022, -0.135, -0.050)
            .root(4.0, 3.0, 14.0)
            .spine(8.0, -3.0, 20.0)
            .chest(2.0, 0.0, 18.0)
            .head(-4.0, 4.0, -26.0)
            // The pointing hand comes up to the chest with the elbow tucked
            // rather than being drawn back and down. A hand cocked at the
            // shoulder has ninety degrees less elbow to spend on the throw, and
            // the throw only has four frames.
            .shoulder_l(24.0, 34.0, -20.0)
            .elbow_l(100.0)
            .wrist_l(-16.0, 0.0, 0.0)
            .shoulder_r(-26.0, 16.0, -18.0)
            .elbow_r(74.0)
            .wrist_r(-14.0, 0.0, -8.0),
        -6.0,
        18.0,
    );

    // Wound and still. Three degrees from the frame before it, which is a hold
    // rather than a pose -- the springs settle onto it and the body genuinely
    // stops for two frames before the hand goes.
    let still = footing(
        stance()
            .hips(0.024, -0.143, -0.054)
            .root(3.0, 3.0, 16.0)
            .spine(9.0, -3.0, 21.0)
            .chest(3.0, 0.0, 19.0)
            .head(-5.0, 4.0, -25.0)
            .shoulder_l(27.0, 37.0, -21.0)
            .elbow_l(104.0)
            .wrist_l(-18.0, 0.0, 0.0)
            .shoulder_r(-29.0, 15.0, -19.0)
            .elbow_r(78.0)
            .wrist_r(-16.0, 0.0, -8.0),
        -7.0,
        19.0,
    );

    // The command, on the first active frame. The left arm is straight and
    // thrown out along the line to the shadow, twenty-five degrees below
    // horizontal -- which is the angle from a shoulder at one metre thirty to
    // the ground three metres away, and three metres is where the mechanic
    // places the shadow. The right arm is clamped back at the ribs with the
    // blade out of the way, and the hips have driven six centimetres forward
    // over the lead foot with the rear heel up behind them.
    //
    // The body barely counter-rotates. The first version turned forty-four
    // degrees away from the point the way a punch would, and the arm came out
    // pointing across the arena instead of down it, which is the one thing this
    // clip cannot afford to get wrong. A command is aimed; it is not thrown.
    let pointing = footing(
        stance()
            .hips(-0.015, -0.130, 0.060)
            .root(10.0, -4.0, -4.0)
            .spine(8.0, 3.0, 0.0)
            .chest(3.0, 0.0, -2.0)
            // Chin down the line rather than level at the opponent. The blades
            // come up out of the floor somewhere else, and the caster is
            // looking at the floor.
            .head(16.0, -2.0, 4.0)
            .shoulder_l(70.0, 4.0, 24.0)
            .elbow_l(8.0)
            .wrist_l(10.0, 0.0, 0.0)
            .shoulder_r(-34.0, 12.0, -26.0)
            .elbow_r(112.0)
            .wrist_r(-16.0, 0.0, -10.0),
        0.0,
        32.0,
    );

    // Five frames of held. Four degrees of drift in the whole pose: the
    // shoulder settles, the hips sink a centimetre, the chest keeps turning
    // very slightly under the arm. Enough that it is a body holding a pose and
    // not a paused video, and nowhere near enough to read as the caster doing
    // anything else.
    let locked = footing(
        stance()
            .hips(-0.018, -0.136, 0.058)
            .root(11.0, -4.0, -5.0)
            .spine(9.0, 3.0, -2.0)
            .chest(3.0, 0.0, -5.0)
            .head(17.0, -2.0, 5.0)
            .shoulder_l(69.0, 3.0, 24.0)
            .elbow_l(10.0)
            .wrist_l(8.0, 0.0, 0.0)
            .shoulder_r(-32.0, 13.0, -26.0)
            .elbow_r(116.0)
            .wrist_r(-14.0, 0.0, -10.0),
        0.0,
        30.0,
    );

    // Letting go. The hand comes off the line first and the body stays low
    // behind it, which is the opposite order to the throw and stops the
    // recovery reading as the clip played backwards.
    let opened = footing(
        stance()
            .hips(0.004, -0.112, 0.020)
            .root(8.0, -2.0, -10.0)
            .spine(11.0, 1.0, -4.0)
            .chest(5.0, 0.0, -6.0)
            .head(2.0, -1.0, 2.0)
            .shoulder_l(46.0, 20.0, 12.0)
            .elbow_l(50.0)
            .wrist_l(-4.0, 0.0, 0.0)
            .shoulder_r(-14.0, 16.0, -18.0)
            .elbow_r(96.0)
            .wrist_r(-14.0, 0.0, -6.0),
        0.0,
        20.0,
    );

    // Back up onto the stance's own height, hands most of the way home.
    let recovering = footing(
        stance()
            .hips(0.010, -0.076, 0.000)
            .root(4.0, -1.0, -14.0)
            .spine(7.0, 0.0, 0.0)
            .chest(3.0, 0.0, 2.0)
            .head(-3.0, 0.0, 4.0)
            .shoulder_l(18.0, 18.0, 4.0)
            .elbow_l(58.0)
            .wrist_l(-8.0, 0.0, 0.0)
            .shoulder_r(-2.0, 16.0, -8.0)
            .elbow_r(64.0)
            .wrist_r(-10.0, 0.0, -2.0),
        0.0,
        14.0,
    );

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "The damage happens at the shadow, so this is a command and not \
                a swing. Thrown with the empty left hand -- the blade hand stays \
                clamped at the ribs -- which is also what separates it at a \
                glance from the two moves that do come at you, both of which are \
                the right arm. The shape is compress, hold, snap, stop: the body \
                makes itself small on frame two, goes genuinely still two frames \
                before the throw so the throw has something to read against, and \
                then holds the pointed pose for every one of the five active \
                frames without a trace of follow-through. That stillness is the \
                animation. A body that carried through would be telling the \
                opponent that something happened in the space in front of the \
                caster, and nothing did. CRISP for the same reason: MARTIAL arms \
                run two frames behind and would still be arriving when the \
                blades come up, and a loose arm settling out of a point turns it \
                back into a swing. The head goes down the line of the arm rather \
                than staying on the opponent, which is the one moment in this \
                kit where the fighter deliberately looks somewhere else, and it \
                is the honest tell for a move whose hitbox is somewhere else \
                too."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::SMOOTH),
            Key::eased(drop, compressed, Ease::OUT),
            Key::eased(wound, still, Ease::LINEAR),
            Key::eased(command, pointing, Ease::OUT),
            Key::eased(held, locked, Ease::SMOOTH),
            Key::eased(opening, opened, Ease::SMOOTH),
            Key::eased(square, recovering, Ease::SMOOTH),
            Key::eased(last, ready(), Ease::SMOOTH),
        ],
    }
}
