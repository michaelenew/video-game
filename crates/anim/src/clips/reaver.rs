//! The Shadow Reaver's eleven moves, and the two poses her shadow has of its own.
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
//! Which is not the same as weightless. The first three clips are deliberately
//! three different weights, because a class whose whole identity is options
//! needs its options to be told apart from across the arena:
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
//! ## The seven of the three-clicks grid (2026-10-14)
//!
//! Until 2026-10-14 every move after the send borrowed one of the four clips
//! above. Each has its own now, and each is told from the others by one thing
//! in its silhouette rather than by its timing, since five of them share a
//! three-frame startup:
//!
//! | Clip | Where | The shape |
//! | --- | --- | --- |
//! | **Plunge** | air | both blades drawn behind the head, a stoop pitched over them, a landing folded over the floor |
//! | **Fan** | air | forearms crossed at the chest flung open wide in front -- an X that becomes a V -- and the legs braking |
//! | **Whirl** | air | a tucked top: both blades level and opposite, the trunk unwinding round between them |
//! | **Gallows** | takeoff | a two-handed open-palmed heave at the sky, a hang with the knees up, the drop, the deepest landing in the kit |
//! | **Tumble** | takeoff | a throw by the ear snapped out of a body balling up; the flip itself is the renderer's |
//! | **Rising whirl** | takeoff | the Whirl stood up: legs long and together, one blade overhead, one level -- a corkscrew |
//! | **Shuriken** | floor | an overhand dart throw from behind the ear, the off arm sighting down the line |
//!
//! Three things the seven share, each with its reason at the helper:
//!
//! - **In the air the legs are angles** ([`legs`]), posed as `air.rs` poses
//!   a jump, and the clips that stay in the air begin and end on one shape
//!   ([`aloft`]). Only a clip that leaves from the floor or arrives on it puts
//!   a foot on it, through `footing`, on the stance's own marks.
//! - **The whole-body roll is not authored here.** `view::interp::roll_of`
//!   flips her over for the Tumble and spins her for nothing, so the Tumble is
//!   posed as the inside of a roll and the two whirls as the widest unwind the
//!   torso has ([`turned`]); a flip keyed here would be a second one.
//! - **The finishers' arms are the hit line's.** The Plunge and Gallows are
//!   upright cuts, so `anim::track` puts the hands on the line through their
//!   live frames; what is authored is the body going over behind them. The
//!   throws and the whirls put nothing out of her body and are posed by hand,
//!   the throws on `THROW`, which is `CRISP` with arms that do not lag the
//!   frame the blade leaves on.
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

use crate::bake::{Feel, Key, Looseness, Recipe};
use crate::clips::locomotion::stance;
use crate::ease::Ease;
use view::clips::Clip;
use view::pose::Pose;
use view::skeleton::Joint;

pub fn clips() -> Vec<Recipe> {
    vec![
        poke(),
        committed(),
        special(),
        mechanic(),
        plunge(),
        fan(),
        whirl(),
        gallows(),
        tumble(),
        rising_whirl(),
        shuriken(),
        dash(),
        ready_stance(),
    ]
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

// ---------------------------------------------------------------------------
// Send shadow -- the mechanic
// ---------------------------------------------------------------------------

/// Eight frames of startup, three active, fourteen of recovery.
///
/// The odd one out in the kit twice over: it is the only thing she throws that
/// is not a blade, and the only one where **she** is what leaves. So it is
/// authored as a release rather than a strike -- the body gathers over the rear
/// foot, both hands come in, and then the whole frame opens at once down the
/// line the second body is about to travel. The right hand shows an open palm
/// at the moment of the throw, which is the one frame in her whole vocabulary
/// where the blade hand is not holding anything.
///
/// It is deliberately not the Guillotine's shape. That one compresses, holds
/// still, and points; this one keeps moving through the release, because the
/// thing it sends is still travelling afterwards.
fn mechanic() -> Recipe {
    let clip = Clip::ReaverMechanic;
    let (_, release, follow) = phases(clip);
    let last = clip.length() - 1;
    // Four frames before the release, when there are four frames to have. At a
    // one-frame startup there are none: the release *is* the first frame she
    // does anything on, and the gather is dropped rather than crushed into the
    // same frame. See the note on the key list.
    let gather = release.saturating_sub(4);
    let settle = follow + (last - follow) / 2;

    // Gathered. Weight back over the rear foot, both hands drawn in across the
    // chest, the shoulders wound away from the line -- a body about to shed
    // something rather than about to swing at you.
    let gathered = footing(
        stance()
            .hips(0.026, -0.104, -0.052)
            .root(2.0, 3.0, 10.0)
            .spine(6.0, -2.0, 16.0)
            .chest(1.0, 0.0, 15.0)
            .head(-4.0, 3.0, -18.0)
            .shoulder_l(20.0, 26.0, -14.0)
            .elbow_l(94.0)
            .wrist_l(-14.0, 0.0, 0.0)
            .shoulder_r(-18.0, 22.0, -16.0)
            .elbow_r(88.0)
            .wrist_r(-12.0, 0.0, -6.0),
        -6.0,
        16.0,
    );

    // The release, on the first active frame. Everything opens down the line at
    // once: hips forward over the lead foot, chest square, the right arm out
    // straight with the palm turned over and the fingers spread, the left arm
    // thrown back as the counterweight. The rear heel is high and pushing.
    let thrown = footing(
        stance()
            .hips(-0.020, -0.118, 0.072)
            .root(11.0, -3.0, -8.0)
            .spine(7.0, 2.0, -4.0)
            .chest(4.0, 0.0, -6.0)
            .head(6.0, -2.0, 2.0)
            .shoulder_r(64.0, 8.0, 30.0)
            .elbow_r(10.0)
            // Palm turned over and the fingers back: the open hand is the whole
            // gesture, and it is the one pose in the kit with nothing in it.
            .wrist_r(-26.0, 0.0, 24.0)
            .shoulder_l(-42.0, 20.0, -10.0)
            .elbow_l(46.0)
            .wrist_l(-6.0, 0.0, 0.0),
        4.0,
        34.0,
    );

    // Through the release rather than stopping on it -- the arm keeps going and
    // the body follows it around, which is what says the thing that left is
    // still going.
    let following = footing(
        stance()
            .hips(-0.030, -0.098, 0.058)
            .root(9.0, -4.0, -16.0)
            .spine(6.0, 2.0, -10.0)
            .chest(3.0, 0.0, -12.0)
            .head(3.0, -2.0, 8.0)
            .shoulder_r(50.0, 6.0, 22.0)
            .elbow_r(30.0)
            .wrist_r(-18.0, 0.0, 14.0)
            .shoulder_l(-28.0, 22.0, -8.0)
            .elbow_l(58.0)
            .wrist_l(-8.0, 0.0, 0.0),
        2.0,
        26.0,
    );

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "A release, not a strike. She is the thing that leaves, so the \
                pose has to open rather than land: gather over the rear foot, \
                then the hips, chest, arm and head all arrive down the line on \
                the same frame and keep going through it. The open right palm on \
                the release frame is the tell -- it is the only frame in this \
                class's whole vocabulary with nothing in the blade hand, so an \
                opponent can read 'the shadow is going somewhere' at a glance \
                and does not have to guess whether a cut is coming. MARTIAL \
                rather than CRISP because the follow-through is wanted here: the \
                second body is still travelling for another ten frames after the \
                gesture ends, and an arm that stopped dead would say it had \
                already arrived."
            .into(),
        keys: {
            let mut keys = vec![Key::eased(0, ready(), Ease::SMOOTH)];
            // **The gather is conditional, and that is the move being honest.**
            // It is a wind-up, and a wind-up is frames the opponent gets to
            // read. Send shadow is thrown on the frame it is asked for, so
            // there are none to read and there must not be an animation
            // claiming otherwise -- a body that visibly gathers before a move
            // that cannot be reacted to is the animation lying about the frame
            // data. Drag the startup back up in the Oven and the gather comes
            // back with it.
            if gather > 0 && gather < release {
                keys.push(Key::eased(gather, gathered, Ease::OUT));
            }
            keys.push(Key::eased(release, thrown, Ease::OUT));
            keys.push(Key::eased(follow, following, Ease::SMOOTH));
            keys.push(Key::eased(settle, following, Ease::SMOOTH));
            keys.push(Key::eased(last, ready(), Ease::SMOOTH));
            keys
        },
    }
}

// ---------------------------------------------------------------------------
// Off the floor
// ---------------------------------------------------------------------------
//
// Five of the seven below happen with nothing under her feet, and two of the
// five leave the floor on their first frame. None of them is a grounded clip by
// the tests' reading -- an attack is judged on its motion, not on its footing --
// so the floor is a promise these make to the eye: in the air the legs are
// angles, posed the way `air.rs` poses a jump, and only a clip that arrives on
// the floor or leaves from it puts a foot on it, through `footing`, on exactly
// the marks the stance left.

/// The body in the air with nothing to do: the shape the jump clips hand over.
///
/// Every air move starts here and the ones that stay in the air end here, so a
/// move thrown out of a jump and a jump that carries on after it meet on one
/// pose rather than on two that happen to be close. The fall's diagonal --
/// lead leg reaching, trail knee gathered -- in the Reaver's narrow build, with
/// the blades carried low and out.
fn aloft() -> Pose {
    Pose::rest()
        .hips(0.0, -0.02, 0.02)
        .root(3.0, 0.0, -8.0)
        .spine(5.0, 0.0, 5.0)
        .chest(2.0, 0.0, 6.0)
        .head(8.0, 0.0, 4.0)
        .shoulder_l(22.0, 44.0, 0.0)
        .elbow_l(44.0)
        .wrist_l(-8.0, 0.0, 0.0)
        .shoulder_r(-6.0, 48.0, 0.0)
        .elbow_r(40.0)
        .wrist_r(-8.0, 0.0, 0.0)
        .hip_l(28.0, 8.0, 0.0)
        .knee_l(26.0)
        .hip_r(40.0, 9.0, 0.0)
        .knee_r(68.0)
        .ankle_l(28.0, 0.0, 0.0)
        .ankle_r(18.0, 0.0, 0.0)
}

/// The legs with nothing under them, as one number: `0` hanging, `1` the knees
/// drawn right up to the chest, and negative trailing out behind.
///
/// One knob rather than six angles per key, because the five air clips spend
/// their whole length moving the legs between those three shapes and a leg
/// that is posed differently on every key twitches.
fn legs(pose: Pose, tuck: f32) -> Pose {
    let (thigh, knee) = if tuck >= 0.0 {
        (12.0 + 98.0 * tuck, 30.0 + 110.0 * tuck)
    } else {
        (12.0 + 40.0 * tuck, 30.0 - 10.0 * tuck)
    };
    pose.hip_l(thigh + 8.0, 9.0, 0.0)
        .knee_l(knee - 6.0)
        .hip_r(thigh - 8.0, 9.0, 0.0)
        .knee_r(knee + 6.0)
        .ankle_l(30.0, 0.0, 0.0)
        .ankle_r(34.0, 0.0, 0.0)
}

/// Twist, shared down the torso the way a turning body shares it: the pelvis
/// first and most, then the waist, then the chest.
///
/// The three joints together turn a hundred and forty-five degrees each way and
/// no further, which is the most a spin on the spot can be in this rig (see
/// [`whirl`]). Split by their limits so that a twist of any size is in range
/// on all three at once.
fn turned(pose: Pose, lean: f32, bend: f32, degrees: f32) -> Pose {
    let share = (degrees / 145.0).clamp(-1.0, 1.0);
    pose.root(lean, 0.0, 65.0 * share)
        .spine(bend, 0.0, 35.0 * share)
        .chest(bend * 0.5, 0.0, 45.0 * share)
}

/// How a throw carries the body: `CRISP`, and tighter still in the arms.
///
/// A blade that leaves the hand leaves on the frame the move table says, and
/// under `CRISP` the hand trails the shoulder by a frame and a half -- which on
/// a five-frame startup put the Shuriken's draw on its own release frame and
/// the release two frames after the blade had gone. Nothing hangs off the hand
/// once it is thrown, so there is no follow-through for the lag to be paying
/// for. The Champion's hammer arms are the precedent.
const THROW: Looseness = Looseness {
    arms: Feel::new(0.8, 0.85),
    ..Looseness::CRISP
};

// ---------------------------------------------------------------------------
// Plunge -- the dive
// ---------------------------------------------------------------------------

/// Four frames of startup, four active, eighteen of recovery, from the air.
///
/// Left click off the floor: she stops where she is and dives along the
/// crosshair, held inside a cone round straight down, blade first, and a ring
/// of shadow runs out from where she lands. A finisher. So the clip is a stoop
/// and an arrival: both blades come up behind the head as the knees come up,
/// the body tips over them and goes down head and hands first, and then she is
/// on the floor folded over two blades driven into it.
///
/// Both hands, where every other thing she throws is one. A finisher is the
/// only time the class commits both blades to one place, and from below -- the
/// person she is diving on -- two hands over the head is the read.
fn plunge() -> Recipe {
    let clip = Clip::ReaverPlunge;
    let (_, contact, follow) = phases(clip);
    let last = clip.length() - 1;
    // The draw as early as the startup allows, so the tip-over has every frame
    // that is left: sixty degrees of pitch in two frames throws the chest past
    // the motion ceiling.
    let drawn_at = contact.saturating_sub(3).max(1);
    // Mid-dive, halfway through the live frames.
    let diving_at = contact + (follow - contact) / 2;
    // On the floor by the first recovery frame -- the dive is fast enough that
    // the body has arrived by then -- and folded over it for a third of the
    // recovery before it stands.
    let dead = follow + (last - follow) / 3;
    let standing = follow + (last - follow) * 2 / 3;

    // Drawn: both hands up behind the head, blades hanging point-down behind
    // the shoulders, the back arched and the knees drawn up. The body makes
    // itself a coil in the air -- the stall before the drop.
    let drawn = legs(
        Pose::rest()
            .hips(0.0, 0.04, -0.03)
            .root(-10.0, 0.0, -4.0)
            .spine(-10.0, 0.0, 2.0)
            .chest(-6.0, 0.0, 2.0)
            .head(10.0, 0.0, 2.0)
            .shoulder_l(146.0, 22.0, 0.0)
            .elbow_l(80.0)
            .wrist_l(-20.0, 0.0, 0.0)
            .shoulder_r(146.0, 22.0, 0.0)
            .elbow_r(80.0)
            .wrist_r(-20.0, 0.0, 0.0),
        0.6,
    );

    // The stoop, on the first active frame: the body pitched sixty degrees
    // over, both arms driven straight down ahead of the head with the blades
    // under them, and the legs swept back and up behind -- a falcon's shape,
    // and the one silhouette in the kit that is longer than it is tall.
    let stoop = legs(
        Pose::rest()
            .hips(0.0, -0.02, 0.06)
            .root(34.0, 0.0, 0.0)
            .spine(20.0, 0.0, 0.0)
            .chest(10.0, 0.0, 0.0)
            .head(-26.0, 0.0, 0.0)
            .shoulder_l(52.0, 14.0, 0.0)
            .elbow_l(14.0)
            .wrist_l(-12.0, 0.0, 0.0)
            .shoulder_r(52.0, 14.0, 0.0)
            .elbow_r(14.0)
            .wrist_r(-12.0, 0.0, 0.0),
        -0.3,
    );

    // Through the dive. Further over, and the knees coming back up under the
    // body, which is what lets the landing two frames later be a landing
    // rather than a body arriving on its face -- and keeps the feet out of
    // the floor while the hips come down onto it.
    let diving = legs(
        Pose::rest()
            .hips(0.0, 0.02, 0.06)
            .root(30.0, 0.0, 0.0)
            .spine(24.0, 0.0, 0.0)
            .chest(12.0, 0.0, 0.0)
            .head(-18.0, 0.0, 0.0)
            .shoulder_l(40.0, 16.0, 0.0)
            .elbow_l(18.0)
            .wrist_l(-14.0, 0.0, 0.0)
            .shoulder_r(40.0, 16.0, 0.0)
            .elbow_r(18.0)
            .wrist_r(-14.0, 0.0, 0.0),
        0.55,
    );

    // Arrived: on the floor, on the stance's own marks, in the deepest crouch
    // the legs allow, folded over two blades driven into the floor between the
    // feet. The ring of shadow goes out from here, so this is the frame the
    // whole move is for.
    let landed = footing(
        stance()
            .hips(0.0, -0.36, 0.04)
            .root(26.0, 0.0, -6.0)
            .spine(32.0, 0.0, 4.0)
            .chest(14.0, 0.0, 4.0)
            .head(-10.0, 0.0, 0.0)
            .shoulder_l(36.0, 20.0, 0.0)
            .elbow_l(30.0)
            .wrist_l(-16.0, 0.0, 0.0)
            .shoulder_r(34.0, 22.0, 0.0)
            .elbow_r(30.0)
            .wrist_r(-16.0, 0.0, 0.0),
        0.0,
        24.0,
    );

    // Still down: the blades pulled out of the floor, hands coming back up to
    // the knees. Nothing guards; minus seven on block should look it.
    let spent = footing(
        stance()
            .hips(0.0, -0.33, 0.03)
            .root(24.0, 0.0, -8.0)
            .spine(28.0, 0.0, 4.0)
            .chest(12.0, 0.0, 4.0)
            .head(-12.0, 0.0, 2.0)
            .shoulder_l(30.0, 24.0, -6.0)
            .elbow_l(54.0)
            .wrist_l(-12.0, 0.0, 0.0)
            .shoulder_r(26.0, 26.0, -6.0)
            .elbow_r(56.0)
            .wrist_r(-12.0, 0.0, 0.0),
        0.0,
        22.0,
    );

    // Up out of it, hips first.
    let rising = footing(
        stance()
            .hips(0.01, -0.16, 0.0)
            .root(12.0, 0.0, -10.0)
            .spine(14.0, 0.0, 4.0)
            .chest(6.0, 0.0, 5.0)
            .head(-6.0, 0.0, 3.0)
            .shoulder_l(20.0, 18.0, 0.0)
            .elbow_l(56.0)
            .wrist_l(-10.0, 0.0, 0.0)
            .shoulder_r(12.0, 20.0, 0.0)
            .elbow_r(52.0)
            .wrist_r(-10.0, 0.0, 0.0),
        0.0,
        16.0,
    );

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "The dive. Both blades, which nothing else she throws uses: a \
                finisher is the one time the class puts both hands on one place, \
                and from underneath -- where the person she is diving on is \
                standing -- two hands drawn up behind the head is the read. The \
                startup is four frames, so there is no wind-up to show, only a \
                coil: knees up, back arched, hands behind the head, two frames \
                in. Then the stoop on the first active frame, pitched sixty \
                degrees over with the arms driven down ahead of the head and the \
                legs swept back behind, the one silhouette in the kit longer than \
                it is tall. By the first recovery frame she has arrived: on the \
                stance's own marks in the deepest crouch the legs allow, folded \
                over two blades in the floor, which is where the ring goes out \
                from. The arms on the live frames are the hit line's \
                (`anim::track` puts them on the upright cut); what is authored \
                is the body going over behind them. Held down for a third of \
                the recovery because minus seven should look like it, then up \
                hips first. CRISP: a dive this short cannot afford arms that \
                arrive two frames late."
            .into(),
        keys: vec![
            Key::eased(0, aloft(), Ease::OUT),
            Key::eased(drawn_at, drawn, Ease::LINEAR),
            Key::eased(contact, stoop, Ease::LINEAR),
            Key::eased(diving_at, diving, Ease::LINEAR),
            Key::eased(follow, landed, Ease::OUT),
            Key::eased(dead, spent, Ease::IN),
            Key::eased(standing, rising, Ease::SMOOTH),
            Key::eased(last, ready(), Ease::SMOOTH),
        ],
    }
}

// ---------------------------------------------------------------------------
// Fan -- the spread
// ---------------------------------------------------------------------------

/// Three frames of startup, three active, twelve of recovery, from the air.
///
/// Middle click off the floor: five blades thrown level, spread either side of
/// her facing, and a moment's stall. Five blades from two hands is a gesture
/// that opens -- the arms crossed in front of the chest, a blade in each
/// fist, and then both flung out across the front at once, wrists flicking
/// at the end. An X that becomes a V.
///
/// It is the only one of her air moves with no turn in the trunk at all: the
/// throw is symmetric because the spread is, and the stall is a body that
/// brakes -- legs swung forward under it -- rather than one that dives or spins.
fn fan() -> Recipe {
    let clip = Clip::ReaverFan;
    let (windup, contact, follow) = phases(clip);
    let last = clip.length() - 1;
    let open_at = contact + (follow - contact) / 2 + 1;
    let lowering = follow + (last - follow) / 2;

    // Crossed: both forearms across the chest, each fist at the other
    // shoulder, the back curled over them and the knees up. Small, closed.
    // The fists are *placed*, not angled: an elbow folded on a shoulder swung
    // across the body folds the forearm up rather than across, and the X is
    // the whole read. The body's right is `-X`.
    let crossed = legs(
        Pose::rest()
            .hips(0.0, 0.02, -0.02)
            .root(6.0, 0.0, 0.0)
            .spine(10.0, 0.0, 0.0)
            .chest(8.0, 0.0, 0.0)
            .head(4.0, 0.0, 0.0)
            .reach_r([0.21, 1.40, 0.22])
            .reach_l([-0.21, 1.35, 0.27])
            .wrist_l(-24.0, 0.0, 0.0)
            .wrist_r(-24.0, 0.0, 0.0),
        0.55,
    );

    // The throw, on the first active frame: both arms swinging out across the
    // front, half open, the chest coming up out of the curl.
    let flinging = legs(
        Pose::rest()
            .hips(0.0, 0.02, 0.0)
            .root(0.0, 0.0, 0.0)
            .spine(2.0, 0.0, 0.0)
            .chest(-2.0, 0.0, 0.0)
            .head(0.0, 0.0, 0.0)
            .shoulder_l(76.0, 22.0, 0.0)
            .elbow_l(62.0)
            .wrist_l(-10.0, 0.0, 0.0)
            .shoulder_r(74.0, 22.0, 0.0)
            .elbow_r(64.0)
            .wrist_r(-10.0, 0.0, 0.0),
        0.3,
    );

    // Open: both arms straight and wide in front of her at shoulder height,
    // the wrists bent back where they let go, the chest open, and the legs
    // swung forward under the body -- the stall, a brake rather than a fall.
    let open = Pose::rest()
        .hips(0.0, 0.02, -0.02)
        .root(-6.0, 0.0, 0.0)
        .spine(-6.0, 0.0, 0.0)
        .chest(-6.0, 0.0, 0.0)
        .head(2.0, 0.0, 0.0)
        .shoulder_l(72.0, 62.0, 0.0)
        .elbow_l(8.0)
        .wrist_l(24.0, 0.0, 0.0)
        .shoulder_r(70.0, 64.0, 0.0)
        .elbow_r(8.0)
        .wrist_r(24.0, 0.0, 0.0)
        .hip_l(46.0, 12.0, 0.0)
        .knee_l(34.0)
        .hip_r(30.0, 12.0, 0.0)
        .knee_r(46.0)
        .ankle_l(26.0, 0.0, 0.0)
        .ankle_r(30.0, 0.0, 0.0);

    // Held open through the stall, arms beginning to drop.
    let held = Pose::rest()
        .hips(0.0, 0.0, -0.01)
        .root(-3.0, 0.0, -2.0)
        .spine(-2.0, 0.0, 2.0)
        .chest(-3.0, 0.0, 2.0)
        .head(4.0, 0.0, 2.0)
        .shoulder_l(52.0, 62.0, 0.0)
        .elbow_l(24.0)
        .wrist_l(6.0, 0.0, 0.0)
        .shoulder_r(46.0, 64.0, 0.0)
        .elbow_r(24.0)
        .wrist_r(6.0, 0.0, 0.0)
        .hip_l(40.0, 10.0, 0.0)
        .knee_l(36.0)
        .hip_r(34.0, 10.0, 0.0)
        .knee_r(56.0)
        .ankle_l(28.0, 0.0, 0.0)
        .ankle_r(26.0, 0.0, 0.0);

    Recipe {
        clip,
        looseness: THROW,
        notes: "Five blades from two hands, so the gesture opens: the forearms \
                cross in front of the chest with a fist at each shoulder, and \
                then both arms fling out across the front at once and finish \
                straight and wide at shoulder height with the wrists bent back \
                where the blades left them -- an X that becomes a V, which is \
                the spread the blades fly in. Symmetric, and the only air move \
                in the kit with no turn in the trunk, because the fan is \
                symmetric about her facing and a body twisting through it would \
                say the blades went one way. The legs carry the stall: they \
                come forward under the body as the arms open, a brake rather \
                than a dive or a spin, and stay there while she hangs. Three \
                frames of startup leave the cross one frame to arrive in, so \
                the contact key is the arms half open and the full spread lands \
                in the middle of the live frames. CRISP with the arms tighter \
                still (`THROW`): a flick, and the blades leave on the frame the \
                table says."
            .into(),
        keys: vec![
            Key::eased(0, aloft(), Ease::OUT),
            Key::eased(windup, crossed, Ease::LINEAR),
            Key::eased(contact, flinging, Ease::LINEAR),
            Key::eased(open_at, open, Ease::OUT),
            Key::eased(lowering, held, Ease::SMOOTH),
            Key::eased(last, aloft(), Ease::SMOOTH),
        ],
    }
}

// ---------------------------------------------------------------------------
// Whirl -- the spin
// ---------------------------------------------------------------------------

/// The flat spin: both blades out level at arm's length and the body turning
/// between them, knees tucked so it is a top and not a figure.
///
/// **It is not a full turn, and cannot be.** The pelvis, the waist and the
/// chest together turn a hundred and forty-five degrees each way and no more,
/// and nothing in `view::interp::roll_of` spins her body for these two moves
/// (it flips the Tumble, and that is all). So the spin is the widest unwind the
/// rig has -- wound hard one way on the last startup frame and carried round
/// to the stop on the other -- with the two blades held opposite each other.
/// Two arms half a turn apart swept through two hundred-odd degrees between
/// them cover the whole circle, which is what the eye reads as a spin; the
/// head spots forward the whole way, as a dancer's does, so the face is the
/// one thing that does not go round.
fn spin(tuck: f32, wind: f32, blades: f32, elbows: f32, lean: f32, hips_up: f32) -> Pose {
    // The head gives back as much of the turn as it can, so she keeps looking
    // where she is going.
    let spot = (-wind * 0.48).clamp(-68.0, 68.0);
    legs(
        turned(Pose::rest().hips(0.0, hips_up, 0.0), lean, 2.0, wind)
            .head(4.0, 0.0, spot)
            .shoulder_l(8.0, blades, 0.0)
            .elbow_l(elbows)
            .wrist_l(-6.0, 0.0, 0.0)
            .shoulder_r(8.0, blades, 0.0)
            .elbow_r(elbows)
            .wrist_r(-6.0, 0.0, 0.0),
        tuck,
    )
}

/// Three frames of startup, three active, twelve of recovery, from the air.
///
/// Right click off the floor: the shadow goes to the crosshair and whirls
/// there, cutting. The blades that spin are the shadow's; what her own body
/// does is the gesture it copies -- a flat spin on the spot, both blades out --
/// so the shape a player sees out on the field and the shape at her shoulder
/// are one shape.
fn whirl() -> Recipe {
    let clip = Clip::ReaverWhirl;
    let (windup, contact, follow) = phases(clip);
    let last = clip.length() - 1;
    // The unwind spends the live frames and most of a third of the recovery:
    // at arm's length a blade covers a third of a metre a frame at twenty-five
    // degrees of turn, so two hundred-odd degrees is eight or nine frames of
    // spin and never fewer.
    let half = follow + 1;
    let round = follow + (last - follow) / 2 + 1;
    let settle = round + (last - round) / 2;

    // Wound: turned hard to the right, arms folded in -- blades at the chest,
    // the right one across it -- and the knees drawn up into a ball.
    let wound = legs(
        turned(Pose::rest().hips(0.0, 0.03, 0.0), 4.0, 6.0, 100.0)
            .head(6.0, 0.0, -50.0)
            .shoulder_l(30.0, 46.0, 0.0)
            .elbow_l(96.0)
            .wrist_l(-14.0, 0.0, 0.0)
            .shoulder_r(66.0, -12.0, 0.0)
            .elbow_r(104.0)
            .wrist_r(-14.0, 0.0, 0.0),
        0.8,
    );

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "A flat spin on the spot: wound hard right with the blades in at \
                the chest and the knees drawn up into a ball, then the arms open \
                straight out level and the whole trunk turns round between them \
                to the stop on the other side. It is not a whole turn and \
                cannot be -- the pelvis, waist and chest give a hundred and \
                forty-five degrees each way between them, and the renderer's \
                whole-body roll flips the Tumble and spins nothing -- so it is \
                the widest unwind the rig has, with two blades held opposite \
                each other: two arms half a turn apart swept through two \
                hundred-odd degrees cover the whole circle, which is what the \
                eye reads as a spin. The head spots forward all the way round. \
                The unwind runs from the contact frame well into the recovery, \
                because at arm's length a blade turning faster than twenty-five \
                degrees a frame is over the motion ceiling; the arms open before \
                the trunk is at speed for the same reason. The knees stay tucked \
                -- a top, not a figure -- which is what separates it at a glance \
                from the Rising whirl's long corkscrew. The blades that cut are \
                the shadow's, at the crosshair; this is the shape it copies."
            .into(),
        keys: vec![
            Key::eased(0, aloft(), Ease::OUT),
            Key::eased(windup, wound, Ease::LINEAR),
            Key::eased(
                contact,
                spin(0.7, 70.0, 54.0, 60.0, 3.0, 0.03),
                Ease::LINEAR,
            ),
            Key::eased(half, spin(0.6, 0.0, 84.0, 14.0, 2.0, 0.03), Ease::LINEAR),
            Key::eased(round, spin(0.6, -110.0, 86.0, 10.0, 2.0, 0.02), Ease::OUT),
            Key::eased(
                settle,
                spin(0.45, -60.0, 60.0, 36.0, 3.0, 0.0),
                Ease::SMOOTH,
            ),
            Key::eased(last, aloft(), Ease::SMOOTH),
        ],
    }
}

// ---------------------------------------------------------------------------
// Gallows -- the hanging and the drop
// ---------------------------------------------------------------------------

/// Twelve frames of startup, four active, thirty-two of recovery, thrown as
/// the feet leave the floor.
///
/// The shadow goes up and a little ahead; the frame it arrives she blinks to
/// it, hangs a beat, and plunges as the Plunge does, heavier. So the clip has
/// four shapes the Plunge does not: a heave from the floor with both hands
/// open, the shadow thrown *up*; the hang, high and still and looking down
/// over what she is hanging above; the drop; and a landing deeper and longer
/// than the Plunge's, because thirty-two frames and minus nineteen on block is
/// the most exposed she ever is.
fn gallows() -> Recipe {
    let clip = Clip::ReaverGallows;
    let (_, contact, follow) = phases(clip);
    let last = clip.length() - 1;
    // Four marks through the startup, all fractions of it: the gather, the
    // heave, the hang, and the frame she tips over out of it.
    let gather = (contact / 4).max(1);
    let heave = (contact * 7 / 12).max(gather + 1);
    let hang = (contact * 3 / 4).max(heave + 1);
    let tip = contact.saturating_sub(1).max(hang + 1);
    let dead = follow + (last - follow) / 3;
    let standing = follow + (last - follow) * 3 / 4;

    // Gathered low: a deep crouch with both hands drawn in to the chest, palms
    // up, the head already up at where the shadow is going.
    let gathered = footing(
        stance()
            .hips(0.02, -0.21, -0.04)
            .root(12.0, 0.0, -4.0)
            .spine(12.0, 0.0, 4.0)
            .chest(4.0, 0.0, 4.0)
            .head(-24.0, 0.0, 2.0)
            .shoulder_l(30.0, 24.0, -10.0)
            .elbow_l(110.0)
            .wrist_l(-20.0, 0.0, 0.0)
            .shoulder_r(28.0, 24.0, -10.0)
            .elbow_r(112.0)
            .wrist_r(-20.0, 0.0, 0.0),
        -4.0,
        18.0,
    );

    // The heave: both arms thrown straight up and forward with the hands open
    // -- the same open palm as the send, because this is a send, aimed at the
    // sky -- and the body stretched up long behind them, legs straight down
    // and toes pointed as the floor goes.
    let heaved = Pose::rest()
        .hips(0.0, 0.04, 0.02)
        .root(-8.0, 0.0, -4.0)
        .spine(-10.0, 0.0, 0.0)
        .chest(-8.0, 0.0, 0.0)
        .head(-32.0, 0.0, 0.0)
        .shoulder_l(150.0, 14.0, 10.0)
        .elbow_l(10.0)
        .wrist_l(-24.0, 0.0, 20.0)
        .shoulder_r(150.0, 14.0, -10.0)
        .elbow_r(10.0)
        .wrist_r(-24.0, 0.0, 20.0)
        .hip_l(4.0, 5.0, 0.0)
        .knee_l(8.0)
        .hip_r(-12.0, 5.0, 0.0)
        .knee_r(22.0)
        .ankle_l(44.0, 0.0, 0.0)
        .ankle_r(46.0, 0.0, 0.0);

    // The hang, where the shadow was. The blades are drawn up behind the head
    // as the Plunge draws them, but higher and tighter, the knees right up,
    // and the chin down: she is looking at what she is hanging over. This is
    // the silhouette the move is named for.
    let hanging = legs(
        Pose::rest()
            .hips(0.0, 0.05, -0.02)
            .root(8.0, 0.0, 0.0)
            .spine(2.0, 0.0, 0.0)
            .chest(-4.0, 0.0, 0.0)
            .head(30.0, 0.0, 0.0)
            .shoulder_l(152.0, 18.0, 0.0)
            .elbow_l(70.0)
            .wrist_l(-22.0, 0.0, 0.0)
            .shoulder_r(152.0, 18.0, 0.0)
            .elbow_r(70.0)
            .wrist_r(-22.0, 0.0, 0.0),
        0.9,
    );

    // Tipping over out of the hang: the weight goes forward first and the
    // blades stay up, which is the last instant before the drop.
    let tipping = legs(
        Pose::rest()
            .hips(0.0, 0.03, 0.02)
            .root(18.0, 0.0, 0.0)
            .spine(8.0, 0.0, 0.0)
            .chest(0.0, 0.0, 0.0)
            .head(20.0, 0.0, 0.0)
            .shoulder_l(130.0, 18.0, 0.0)
            .elbow_l(50.0)
            .wrist_l(-18.0, 0.0, 0.0)
            .shoulder_r(130.0, 18.0, 0.0)
            .elbow_r(50.0)
            .wrist_r(-18.0, 0.0, 0.0),
        0.65,
    );

    // The drop, on the first active frame: pitched over the blades, both arms
    // driven down ahead of the head, the knees still under the body so the
    // feet meet the floor rather than going through it.
    let dropping = legs(
        Pose::rest()
            .hips(0.0, 0.02, 0.06)
            .root(30.0, 0.0, 0.0)
            .spine(22.0, 0.0, 0.0)
            .chest(12.0, 0.0, 0.0)
            .head(-16.0, 0.0, 0.0)
            .shoulder_l(70.0, 16.0, 0.0)
            .elbow_l(16.0)
            .wrist_l(-14.0, 0.0, 0.0)
            .shoulder_r(70.0, 16.0, 0.0)
            .elbow_r(16.0)
            .wrist_r(-14.0, 0.0, 0.0),
        0.4,
    );

    // Arrived, and buried: lower than the Plunge's landing and further over,
    // the blades in the floor at arm's length in front, head down between the
    // arms.
    let buried = footing(
        stance()
            .hips(0.0, -0.40, 0.05)
            .root(32.0, 0.0, -6.0)
            .spine(40.0, 0.0, 4.0)
            .chest(18.0, 0.0, 4.0)
            .head(6.0, 0.0, 0.0)
            .shoulder_l(40.0, 18.0, 0.0)
            .elbow_l(20.0)
            .wrist_l(-18.0, 0.0, 0.0)
            .shoulder_r(38.0, 20.0, 0.0)
            .elbow_r(20.0)
            .wrist_r(-18.0, 0.0, 0.0),
        0.0,
        26.0,
    );

    let spent = footing(
        stance()
            .hips(0.0, -0.38, 0.04)
            .root(30.0, 0.0, -8.0)
            .spine(38.0, 0.0, 4.0)
            .chest(16.0, 0.0, 4.0)
            .head(4.0, 0.0, 2.0)
            .shoulder_l(30.0, 22.0, -4.0)
            .elbow_l(44.0)
            .wrist_l(-14.0, 0.0, 0.0)
            .shoulder_r(28.0, 24.0, -4.0)
            .elbow_r(46.0)
            .wrist_r(-14.0, 0.0, 0.0),
        0.0,
        24.0,
    );

    let rising = footing(
        stance()
            .hips(0.01, -0.17, 0.0)
            .root(14.0, 0.0, -10.0)
            .spine(16.0, 0.0, 4.0)
            .chest(6.0, 0.0, 5.0)
            .head(-6.0, 0.0, 3.0)
            .shoulder_l(20.0, 18.0, 0.0)
            .elbow_l(56.0)
            .wrist_l(-10.0, 0.0, 0.0)
            .shoulder_r(12.0, 20.0, 0.0)
            .elbow_r(52.0)
            .wrist_r(-10.0, 0.0, 0.0),
        0.0,
        16.0,
    );

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "Four shapes, in the order the simulation does them. A heave from \
                a deep crouch, both arms thrown straight up with the palms open \
                -- the send's open hand, because this is a send aimed at the sky, \
                and the one frame a player has to tell Gallows from the Plunge \
                before she is gone. The hang, where the shadow was: knees right \
                up, blades drawn behind the head, chin down over what she is \
                hanging above, and still for a beat -- the gallows the move is \
                named for. A tip forward with the blades left up, the last \
                instant before the drop. Then the drop on the first active frame, \
                pitched over with both arms driven down, and a landing deeper \
                and further over than the Plunge's, held dead for a third of a \
                thirty-two-frame recovery because minus nineteen on block is the \
                most exposed she ever is. Every mark in the startup is a \
                fraction of it, so a retuned blink moves the heave and the hang \
                with it. The arms on the live frames are the hit line's \
                (`anim::track`); the body under them is what is authored here. \
                MARTIAL rather than CRISP: this is the heavy finisher, and the \
                arms trailing the body over the top of the hang is the weight."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::SMOOTH),
            Key::eased(gather, gathered, Ease::SMOOTH),
            Key::eased(heave, heaved, Ease::OUT),
            Key::eased(hang, hanging, Ease::HOLD),
            Key::eased(tip, tipping, Ease::LINEAR),
            Key::eased(contact, dropping, Ease::LINEAR),
            Key::eased(follow, buried, Ease::OUT),
            Key::eased(dead, spent, Ease::IN),
            Key::eased(standing, rising, Ease::SMOOTH),
            Key::eased(last, ready(), Ease::SMOOTH),
        ],
    }
}

// ---------------------------------------------------------------------------
// Tumble -- the roll
// ---------------------------------------------------------------------------

/// Three frames of startup, four active, twelve of recovery, thrown as the
/// feet leave the floor (or as her double jump).
///
/// **The flip is not in this clip.** `view::interp::roll_of` turns her whole
/// body over once about the hips, eased, from the first frame of the move to
/// half way through its recovery, off the simulation's clock. A flip authored
/// here as well would be two turns. So the clip is what a body does *inside* a
/// forward roll: it balls up, and the throw comes out of the ball -- the right
/// hand cocked by the ear and snapped forward and down as she goes over, the
/// three blades leaving it in a fan -- and then it opens again as the turn
/// finishes, legs reaching back under her for whatever comes next.
fn tumble() -> Recipe {
    let clip = Clip::ReaverTumble;
    let (windup, contact, follow) = phases(clip);
    let last = clip.length() - 1;
    // The arm is straight by the last live frame but one: from the ear to
    // full length in fewer than three frames is over the motion ceiling.
    let flicked = follow.saturating_sub(1).max(contact + 1);
    // The roll is done half way through the recovery, so the body opens over
    // the frames before that and hangs open after it.
    let opening = follow + (last - follow) / 3 + 1;

    // Cocked: knees coming up, the right hand drawn up by the ear with the
    // blades in it, the left arm out in front for balance.
    let cocked = legs(
        Pose::rest()
            .hips(0.0, 0.02, 0.0)
            .root(8.0, 0.0, 4.0)
            .spine(8.0, 0.0, 6.0)
            .chest(4.0, 0.0, 8.0)
            .head(-4.0, 0.0, -8.0)
            .shoulder_r(124.0, 40.0, -20.0)
            .elbow_r(118.0)
            .wrist_r(-24.0, 0.0, 0.0)
            .shoulder_l(36.0, 22.0, 0.0)
            .elbow_l(50.0)
            .wrist_l(-10.0, 0.0, 0.0),
        0.5,
    );

    // The throw, out of a body curling over: the right arm whipping forward
    // past the face and opening, chin coming down to the chest.
    let throwing = legs(
        Pose::rest()
            .hips(0.0, 0.03, 0.0)
            .root(14.0, 0.0, -2.0)
            .spine(18.0, 0.0, -4.0)
            .chest(10.0, 0.0, -6.0)
            .head(14.0, 0.0, 4.0)
            .shoulder_r(112.0, 24.0, 0.0)
            .elbow_r(84.0)
            .wrist_r(-6.0, 0.0, 6.0)
            .shoulder_l(40.0, 30.0, 0.0)
            .elbow_l(60.0)
            .wrist_l(-10.0, 0.0, 0.0),
        0.7,
    );

    // Let go: the right arm straight out ahead and down along the body's own
    // line -- the line the blades fan from -- and the rest of her already a
    // ball behind it.
    let flick = legs(
        Pose::rest()
            .hips(0.0, 0.04, 0.0)
            .root(20.0, 0.0, -4.0)
            .spine(28.0, 0.0, -6.0)
            .chest(14.0, 0.0, -6.0)
            .head(26.0, 0.0, 4.0)
            .shoulder_r(70.0, 8.0, 18.0)
            .elbow_r(12.0)
            .wrist_r(10.0, 0.0, 10.0)
            .shoulder_l(30.0, 26.0, 0.0)
            .elbow_l(90.0)
            .wrist_l(-12.0, 0.0, 0.0),
        0.9,
    );

    // The ball: everything in. Both hands at the shins, chin on the chest,
    // knees to it. This is the body the roll turns over.
    let ball = legs(
        Pose::rest()
            .hips(0.0, 0.05, 0.0)
            .root(24.0, 0.0, 0.0)
            .spine(34.0, 0.0, 0.0)
            .chest(18.0, 0.0, 0.0)
            .head(30.0, 0.0, 0.0)
            .shoulder_r(44.0, 12.0, 0.0)
            .elbow_r(96.0)
            .wrist_r(-16.0, 0.0, 0.0)
            .shoulder_l(44.0, 12.0, 0.0)
            .elbow_l(96.0)
            .wrist_l(-16.0, 0.0, 0.0),
        1.0,
    );

    // Opening out as the turn finishes: legs dropping back under, arms going
    // wide for the balance.
    let opened = legs(
        Pose::rest()
            .hips(0.0, 0.02, 0.0)
            .root(6.0, 0.0, -4.0)
            .spine(8.0, 0.0, 2.0)
            .chest(4.0, 0.0, 4.0)
            .head(4.0, 0.0, 2.0)
            .shoulder_r(30.0, 56.0, 0.0)
            .elbow_r(40.0)
            .wrist_r(-8.0, 0.0, 0.0)
            .shoulder_l(36.0, 52.0, 0.0)
            .elbow_l(40.0)
            .wrist_l(-8.0, 0.0, 0.0),
        0.4,
    );

    Recipe {
        clip,
        looseness: THROW,
        notes: "The turn itself is not here: `view::interp::roll_of` flips her \
                whole body forward once about the hips, from the move's first \
                frame to half way through its recovery, and a flip authored in \
                the clip as well would be two. So this is what a body does \
                inside a forward roll: it balls up, and the throw comes out of \
                the ball. The right hand is cocked by the ear on the last \
                startup frame and snaps forward past the face as she curls -- \
                straight out along the body's own line by the middle of the live \
                frames, which, a quarter of the way round the flip, is forward \
                and down, where the three blades go. Then everything in: chin \
                on the chest, knees to it, both hands at the shins -- the ball \
                the roll turns over, and the shape that tells it from every \
                other air move she has. It opens again as the turn finishes, \
                arms wide and legs back under her, and ends on the air pose the \
                jump clips take over from, because she is still going up. \
                `THROW`, CRISP with tighter arms: the flip is fast, an arm \
                lagging it would smear, and the blades leave on time."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(windup, cocked, Ease::LINEAR),
            Key::eased(contact, throwing, Ease::LINEAR),
            Key::eased(flicked, flick, Ease::OUT),
            Key::eased(follow + 1, ball, Ease::SMOOTH),
            Key::eased(opening + 2, opened, Ease::SMOOTH),
            Key::eased(last, aloft(), Ease::SMOOTH),
        ],
    }
}

// ---------------------------------------------------------------------------
// Rising whirl -- the corkscrew
// ---------------------------------------------------------------------------

/// Three frames of startup, three active, twelve of recovery, thrown as the
/// feet leave the floor.
///
/// The Whirl with a jump under it, so the spin is turned upright: where the
/// Whirl is a flat top with its knees tucked and both blades level, this one is
/// long -- legs straight down together, one blade driven up over the head and
/// the other out level -- and turns as it goes up, a corkscrew. Wound low in a
/// crouch on the floor, unwound in the air.
fn rising_whirl() -> Recipe {
    let clip = Clip::ReaverRisingWhirl;
    let (windup, contact, follow) = phases(clip);
    let last = clip.length() - 1;
    let half = follow + 1;
    let round = follow + (last - follow) / 2 + 1;
    let settle = round + (last - round) / 2;

    // Wound in a crouch on the stance's marks, turned hard right with the
    // right blade across the chest and the left trailing back. The head stays
    // on the line.
    let wound = footing(
        turned(stance().hips(0.02, -0.20, -0.02), 10.0, 10.0, 90.0)
            .head(-10.0, 0.0, -50.0)
            .shoulder_r(60.0, -14.0, 0.0)
            .elbow_r(108.0)
            .wrist_r(-16.0, 0.0, 0.0)
            .shoulder_l(-16.0, 44.0, 0.0)
            .elbow_l(70.0)
            .wrist_l(-10.0, 0.0, 0.0),
        -4.0,
        22.0,
    );

    // The corkscrew. `wind` turns the trunk; the right blade goes up over
    // the head (close to the axis it turns about, which is what lets it turn
    // fast) and the left goes out level.
    let screw = |wind: f32, up: f32, out: f32, lean: f32, hips_up: f32| {
        let spot = (-wind * 0.48).clamp(-68.0, 68.0);
        turned(Pose::rest().hips(0.0, hips_up, 0.0), lean, -4.0, wind)
            .head(-10.0, 0.0, spot)
            .shoulder_r(up, 14.0, 0.0)
            .elbow_r(16.0)
            .wrist_r(-8.0, 0.0, 0.0)
            .shoulder_l(10.0, out, 0.0)
            .elbow_l(14.0)
            .wrist_l(-6.0, 0.0, 0.0)
            .hip_l(4.0, 4.0, 0.0)
            .knee_l(8.0)
            .hip_r(-2.0, 4.0, 0.0)
            .knee_r(14.0)
            .ankle_l(46.0, 0.0, 0.0)
            .ankle_r(48.0, 0.0, 0.0)
    };

    // Leaving the floor: still mostly wound, the legs driving straight, the
    // right arm on its way up.
    let launching = footing(
        turned(stance().hips(0.0, -0.06, 0.0), 2.0, 0.0, 64.0)
            .head(-12.0, 0.0, -32.0)
            .shoulder_r(110.0, 10.0, 0.0)
            .elbow_r(50.0)
            .wrist_r(-10.0, 0.0, 0.0)
            .shoulder_l(0.0, 60.0, 0.0)
            .elbow_l(40.0)
            .wrist_l(-8.0, 0.0, 0.0),
        30.0,
        40.0,
    );

    // Still climbing when the move lets go: the jump's own shape, the raised
    // arm left up and the other open, the legs beginning to gather. The rise
    // takes over from here, so this is nearer it than the plain air pose.
    let risen = legs(
        Pose::rest()
            .hips(0.0, 0.04, 0.0)
            .root(-3.0, 0.0, -10.0)
            .spine(-3.0, 0.0, 4.0)
            .chest(-1.0, 0.0, 5.0)
            .head(-6.0, 0.0, 4.0)
            .shoulder_r(118.0, 30.0, 0.0)
            .elbow_r(34.0)
            .wrist_r(-8.0, 0.0, 0.0)
            .shoulder_l(30.0, 58.0, 0.0)
            .elbow_l(34.0)
            .wrist_l(-8.0, 0.0, 0.0),
        0.2,
    );

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "The Whirl with a jump under it, so the spin stands up. Wound in a \
                crouch on the stance's own marks, turned hard right with a blade \
                across the chest; then the legs drive straight and stay straight \
                and together, the right blade goes up over the head and the left \
                out level, and the trunk turns round between them as she rises \
                -- a corkscrew, long where the Whirl is a tucked top, and one \
                blade high where the Whirl's are both level, so the two are told \
                apart in silhouette before either has turned at all. The turn is \
                the rig's widest, as the Whirl's is, and for the same reason: \
                three joints give a hundred and forty-five degrees each way and \
                no renderer roll spins her. The raised arm is close to the axis \
                it turns about, which is what lets the unwind start on the \
                contact frame without the hand breaking the motion ceiling; the \
                level one opens as the trunk comes round. It ends on the air \
                pose because the jump is still going up."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(windup, wound, Ease::LINEAR),
            Key::eased(contact, launching, Ease::LINEAR),
            Key::eased(half, screw(10.0, 160.0, 70.0, -2.0, 0.08), Ease::LINEAR),
            Key::eased(round, screw(-100.0, 166.0, 84.0, -2.0, 0.08), Ease::OUT),
            Key::eased(settle, screw(-60.0, 140.0, 64.0, 2.0, 0.08), Ease::SMOOTH),
            Key::eased(last, risen, Ease::SMOOTH),
        ],
    }
}

// ---------------------------------------------------------------------------
// Shuriken -- the throw
// ---------------------------------------------------------------------------

/// Five frames of startup, three active, twelve of recovery, on foot.
///
/// One blade, thrown to the crosshair, flat and hard -- and the shadow throws
/// one too from wherever it stands. An overhand dart throw: the blade drawn
/// back behind the right ear, the left arm out straight down the line as the
/// sight, then the right arm whipped through over the top and out along that
/// same line, and the body following it round. Where Slash is a diagonal cut
/// with the arm wide, this is a straight line with the arm close to the head:
/// the difference between cutting the space in front of her and sending
/// something down it.
fn shuriken() -> Recipe {
    let clip = Clip::ReaverShuriken;
    let (_, contact, follow) = phases(clip);
    let last = clip.length() - 1;
    // Drawn three frames before the release, and the release is the hand
    // passing the face rather than the arm at full length: the hand goes most
    // of a metre over the top, and spending all of it before contact broke the
    // motion ceiling by half again. The arm reaches full length on the next
    // frame, with the blade already gone.
    let drawn_at = contact.saturating_sub(3).max(1);
    let extended_at = contact + 1;
    let through = follow + 1;
    let settle = follow + (last - follow) / 2;

    // Drawn: the blade behind the right ear, elbow high and back, the chest
    // turned away from the line and the left arm out straight along it --
    // pointing at where it is going. Weight back over the rear foot.
    let drawn = footing(
        stance()
            .hips(0.02, -0.09, -0.05)
            .root(2.0, 2.0, 10.0)
            .spine(-2.0, -2.0, 18.0)
            .chest(-4.0, 0.0, 20.0)
            .head(-2.0, 2.0, -36.0)
            // The upper arm up and a little in front of vertical with the elbow
            // folded right over, which hangs the hand behind the head. Angles
            // rather than `reach_r`: the solver found this point on the other
            // reading of the shoulder from the release's, and the frames
            // between them swung the elbow round the long way.
            .shoulder_r(150.0, 26.0, 0.0)
            .elbow_r(136.0)
            .wrist_r(-28.0, 0.0, 0.0)
            .shoulder_l(66.0, 8.0, 0.0)
            .elbow_l(26.0)
            .wrist_l(-4.0, 0.0, 0.0),
        -8.0,
        14.0,
    );

    // The release, on the first active frame: the hand has come over the top
    // and is passing in front of the face, the chest turned through square,
    // the left arm pulled back to the ribs and the hips driven forward over
    // the lead foot with the rear heel up.
    let release = footing(
        stance()
            .hips(-0.005, -0.10, 0.045)
            .root(5.0, -1.0, -4.0)
            .spine(6.0, 1.0, -4.0)
            .chest(2.0, 0.0, -6.0)
            .head(0.0, -1.0, 0.0)
            .shoulder_r(114.0, 10.0, 0.0)
            .elbow_r(56.0)
            .wrist_r(4.0, 0.0, 4.0)
            .shoulder_l(20.0, 20.0, 0.0)
            .elbow_l(76.0)
            .wrist_l(-10.0, 0.0, 0.0),
        0.0,
        26.0,
    );

    // A frame later: the arm out straight along the line at head height.
    let extended = footing(
        stance()
            .hips(-0.01, -0.10, 0.06)
            .root(6.0, -2.0, -8.0)
            .spine(8.0, 2.0, -8.0)
            .chest(4.0, 0.0, -12.0)
            .head(0.0, -2.0, 6.0)
            .shoulder_r(98.0, 6.0, 0.0)
            .elbow_r(16.0)
            .wrist_r(10.0, 0.0, 6.0)
            .shoulder_l(14.0, 20.0, 0.0)
            .elbow_l(84.0)
            .wrist_l(-10.0, 0.0, 0.0),
        0.0,
        30.0,
    );

    // Followed through: the arm carried on down and across to the left hip,
    // the chest turned past square, the head still on the line the blade went.
    let followed = footing(
        stance()
            .hips(-0.02, -0.11, 0.06)
            .root(10.0, -3.0, -18.0)
            .spine(14.0, 3.0, -14.0)
            .chest(8.0, 0.0, -20.0)
            .head(-4.0, -2.0, 24.0)
            .shoulder_r(46.0, -16.0, 10.0)
            .elbow_r(22.0)
            .wrist_r(4.0, 0.0, 8.0)
            .shoulder_l(-18.0, 26.0, 0.0)
            .elbow_l(80.0)
            .wrist_l(-8.0, 0.0, 0.0),
        0.0,
        28.0,
    );

    let gathering = footing(
        stance()
            .hips(0.0, -0.08, 0.02)
            .root(4.0, -1.0, -16.0)
            .spine(8.0, 1.0, 0.0)
            .chest(4.0, 0.0, -2.0)
            .head(-4.0, -1.0, 10.0)
            .shoulder_r(18.0, 20.0, 10.0)
            .elbow_r(60.0)
            .wrist_r(-8.0, 0.0, 0.0)
            .shoulder_l(6.0, 18.0, 0.0)
            .elbow_l(60.0)
            .wrist_l(-8.0, 0.0, 0.0),
        0.0,
        16.0,
    );

    Recipe {
        clip,
        looseness: THROW,
        notes: "An overhand dart throw, and deliberately not Slash: Slash is a \
                diagonal with the arm wide, this is a straight line with the arm \
                close to the head, the difference between cutting the space in \
                front of her and sending something down it. The draw is the read \
                -- blade behind the right ear, elbow high and back, chest turned \
                away and the left arm straight out along the line as the sight, \
                weight on the rear foot -- and it lands three frames before the \
                release so the arm has three frames to come over the top in \
                rather than one; LINEAR through them, the file's usual reason. \
                The release is the hand passing the face, already in front of \
                it, with the hips driven over the lead foot and the sighting arm \
                snapped back to the ribs; the arm is out straight along the line \
                at head height a frame later, and it follows through down \
                and across to the left hip and the head stays on the line the \
                blade went. Feet on the stance's marks throughout. `THROW`: \
                CRISP with the arms tighter, because under CRISP's arm lag the \
                draw arrived on the release frame."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(drawn_at, drawn, Ease::LINEAR),
            Key::eased(contact, release, Ease::LINEAR),
            Key::eased(extended_at, extended, Ease::LINEAR),
            Key::eased(through, followed, Ease::OUT),
            Key::eased(settle, gathering, Ease::SMOOTH),
            Key::eased(last, ready(), Ease::SMOOTH),
        ],
    }
}

// ---------------------------------------------------------------------------
// The shadow's own two poses
// ---------------------------------------------------------------------------
//
// Everything else the second body does is one of her clips replayed a few
// frames late -- that is the whole idea of a shadow, and it is why there are
// two of these rather than a second animation set. These are the two things it
// does that she never does: cross the arena on its own, and stand somewhere
// waiting for her.

/// The shadow travelling.
///
/// One pose, entered fast and held. The shadow crosses in ten frames and it is
/// a silhouette the whole way -- what the player has to read is *which way it
/// is going*, from across the arena, at speed. So the pose is built as a
/// diagonal: the body pitched forward over the line of travel, the right leg
/// thrown back behind it, the left knee up and folded under the chest, the left
/// elbow forward across the face and the right arm streaming behind. Every limb
/// is either along the line or across it; nothing is neutral.
///
/// Held rather than animated. It is the only clip in the file with no second
/// beat, and deliberately: motion on a body that is already moving at thirty
/// metres a second reads as noise, and the *arrival* is a separate clip that
/// the renderer cross-fades to.
fn dash() -> Recipe {
    let clip = Clip::ShadowDash;
    let last = clip.length() - 1;
    // Into it in two frames. The fade out of whatever it was doing covers the
    // first of them.
    let planted = 2.min(last.saturating_sub(1)).max(1);

    // The launch. Still recognisably a body pushing off, with the rear leg
    // straightening and the lead knee starting up.
    let leaving = stance()
        .hips(0.0, -0.150, 0.090)
        .root(26.0, 0.0, -6.0)
        .spine(10.0, 0.0, 4.0)
        .chest(6.0, 0.0, 6.0)
        .head(-16.0, 0.0, 0.0)
        .hip_l(48.0, 8.0, 0.0)
        .knee_l(76.0)
        .ankle_l(16.0, 0.0, 0.0)
        .hip_r(-30.0, 4.0, 0.0)
        .knee_r(26.0)
        .ankle_r(24.0, 0.0, 0.0)
        .shoulder_l(30.0, 22.0, -20.0)
        .elbow_l(96.0)
        .wrist_l(-10.0, 0.0, 0.0)
        .shoulder_r(-40.0, 14.0, -10.0)
        .elbow_r(40.0)
        .wrist_r(-8.0, 0.0, 0.0);

    // Full flight, and the pose the whole clip is for.
    //
    // The forty-five degrees is the rear leg against the body's own line, not
    // against the floor: the torso is pitched thirty-two degrees forward, so a
    // thigh swung thirteen degrees behind vertical puts the whole leg on the
    // diagonal that reads as forty-five from the side. The knee is left
    // fractionally bent -- a dead straight leg reads as a mannequin.
    let flying = stance()
        .hips(0.0, -0.205, 0.150)
        .root(32.0, 0.0, -4.0)
        .spine(12.0, 0.0, 2.0)
        .chest(7.0, 0.0, 4.0)
        // Chin tucked behind the leading elbow, eyes up the line.
        .head(-22.0, 0.0, 2.0)
        // Left knee up and fully folded, tucked under the chest.
        .hip_l(96.0, 10.0, 0.0)
        .knee_l(132.0)
        .ankle_l(22.0, 0.0, 0.0)
        // Right leg thrown back and all but straight, toe pointed.
        .hip_r(-46.0, 3.0, 0.0)
        .knee_r(10.0)
        .ankle_r(34.0, 0.0, 0.0)
        // Left elbow forward, across the face. The forearm is the leading edge
        // of the silhouette and the face is behind it.
        .shoulder_l(78.0, 4.0, -34.0)
        .elbow_l(122.0)
        .wrist_l(-16.0, 0.0, 0.0)
        // Right arm streaming behind, nearly straight, palm turned up.
        .shoulder_r(-72.0, 10.0, 16.0)
        .elbow_r(16.0)
        .wrist_r(-4.0, 0.0, 10.0);

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "A silhouette, held. The shadow crosses the arena in ten frames \
                and the only thing a player can read at that speed is the \
                outline, so every limb is placed on the diagonal of travel: body \
                pitched forward, right leg thrown back along the line, left knee \
                up and folded across it, left elbow leading in front of the face \
                and the right arm streaming behind. No second beat, because a \
                body moving that fast does not need one and any motion on top of \
                it reads as flicker. The arrival is `shadow_ready`, and the \
                renderer cross-fades between the two -- which is the transition, \
                and is why neither clip has to author it. CRISP so the pose is \
                fully arrived by frame two of ten; MARTIAL arms would still be \
                catching up when it landed."
            .into(),
        keys: vec![
            Key::eased(0, leaving, Ease::OUT),
            Key::eased(planted, flying, Ease::SMOOTH),
            Key::eased(last, flying, Ease::SMOOTH),
        ],
    }
}

/// The shadow arrived: combat ready, and waiting.
///
/// It is standing somewhere she chose, and everything in her kit is about to be
/// aimed through it, so it must not read as scenery. A low bladed guard with
/// both hands up says "this is a body and it is still in the fight", which is
/// exactly what the shadow is: it copies her swings from wherever it stands.
///
/// Loops, and breathes -- slower and shallower than hers, because it is a
/// shadow and not a person.
fn ready_stance() -> Recipe {
    let clip = Clip::ShadowReady;
    let last = clip.length() - 1;
    let half = last / 2;

    // The guard. Lower than her walking stance, weight further back, both hands
    // up and forward with the elbows in -- and squarer to the front than she
    // ever stands, because it has no line of its own to protect.
    let guard = footing(
        stance()
            .hips(0.014, -0.132, -0.028)
            .root(6.0, 0.0, -6.0)
            .spine(8.0, 0.0, 4.0)
            .chest(4.0, 0.0, 6.0)
            .head(-6.0, 0.0, 2.0)
            .shoulder_l(34.0, 20.0, -16.0)
            .elbow_l(102.0)
            .wrist_l(-12.0, 0.0, 0.0)
            .shoulder_r(24.0, 22.0, -14.0)
            .elbow_r(96.0)
            .wrist_r(-12.0, 0.0, -4.0),
        -2.0,
        18.0,
    );

    // The breath. Three centimetres at the hips and a couple of degrees
    // everywhere else -- enough to prove it is alive, not enough to be a motion
    // anybody watches.
    let breathing = footing(
        stance()
            .hips(0.014, -0.116, -0.026)
            .root(5.0, 0.0, -6.0)
            .spine(6.0, 0.0, 4.0)
            .chest(2.0, 0.0, 6.0)
            .head(-7.0, 0.0, 2.0)
            .shoulder_l(32.0, 23.0, -16.0)
            .elbow_l(99.0)
            .wrist_l(-12.0, 0.0, 0.0)
            .shoulder_r(22.0, 25.0, -14.0)
            .elbow_r(93.0)
            .wrist_r(-12.0, 0.0, -4.0),
        -2.0,
        18.0,
    );

    Recipe {
        clip,
        looseness: Looseness::STRIDE,
        notes: "What the second body does while it waits, and it waits for most \
                of a round. A low bladed guard rather than an idle: everything \
                in the kit is aimed through this thing, and it copies her swings \
                from wherever it stands, so it has to read as a fighter standing \
                there and not as a marker on the floor. Squarer to the front \
                than her own stance, because it has no line of its own to \
                defend. The breath is a centimetre and a half of hips over a \
                second and a half -- slower and shallower than hers on purpose, \
                since the pair are usually on screen together and two bodies \
                breathing in step read as one body with a ghost stuck to it."
            .into(),
        keys: vec![
            Key::eased(0, guard, Ease::SMOOTH),
            Key::eased(half, breathing, Ease::SMOOTH),
            Key::eased(last, guard, Ease::SMOOTH),
        ],
    }
}
