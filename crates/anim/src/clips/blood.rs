//! The Blood mage: Bloodletter, Rend, Grasp, Black spike.
//!
//! ## What the class fights like
//!
//! Sustain through aggression, paid for out of its own health bar. There is no
//! meter to run dry and no resource to husband: every ability opens the caster
//! up and the good outcomes give the blood back. So the class is never
//! *spending mana*, it is spending itself, and the body has to say that on
//! every move or the mechanic is invisible to the person opposite.
//!
//! Three things carry it, and they are in every clip here:
//!
//! - **The recovery is heavier than the wind-up.** The frame data already says
//!   so -- Rend is six frames of startup against thirteen of recovery -- and
//!   the poses agree with it rather than fighting it. The lowest, slowest,
//!   most folded frame of each of these clips is *after* the hit, not before.
//! - **A hand comes back to the ribs.** Every recovery in this file ends with
//!   the off hand drawn in against the caster's own body and the chest closed
//!   over it. It is the one gesture repeated across all three, so a spectator
//!   learns it as the class's punctuation: that was not free.
//! - **Nothing is athletic.** `build_for(BloodMage)` is ordinary proportions
//!   carrying more bulk than they should, and the poses are built for that
//!   body. No overhead goes past the ear, no lunge is long, and the knees give
//!   at the end of everything.
//!
//! ## The four read as four distances
//!
//! An opponent has to know which one is coming, and here they are told by
//! where the caster is *pointing* in the first three frames:
//!
//! ```text
//!   Bloodletter  one hand drawn back low, behind the hip  -- past you
//!   Rend         one hand cocked out and up, close in     -- at you
//!   Grasp        the feet square and both arms opening    -- around you
//!   Black spike  one hand climbing, one pointing low      -- at the floor
//! ```
//!
//! Grasp is the odd one and deliberately so: it is the only pose in the game
//! that squares its feet. Everything else in the repository stands bladed, on
//! `locomotion::stance`, because a bladed stance is what lets a body turn and
//! lead with one side. This move has no lead side -- it is two arms doing the
//! same thing at the same time, mirrored, and the four arms it throws leave in
//! four symmetric directions. A body that is about to do something symmetrical
//! squares up to do it, and squaring up is legible from across the arena long
//! before the arms mean anything.
//!
//! ## Frames come from the move table
//!
//! Every key here is derived from `clip.phases()`, read live from the Oven, so
//! retuning a startup moves the animation with it. Two boundaries are used
//! directly rather than scaled: the **contact pose sits on the first active
//! frame**, which is the frame the hitbox appears and the frame the spike
//! erupts a reach in front of the caster, and the **heaviest frame sits on the
//! first recovery frame**, because a body that is still arriving after the
//! blow has landed is what weight looks like.
//!
//! The last key of each clip sits three frames short of the end rather than on
//! it. The arms run a couple of frames behind the keys, so a return keyed on
//! the final frame is a return the body never actually reaches -- and the last
//! frame is the one control comes back on, which is where a pop would show.
//!
//! ## Why the fast parts are LINEAR
//!
//! A shaped ease spends about half its travel in the middle of its span, so
//! across the four frames of Rend's rake or the eight of the spike's descent it
//! is worth half again as much motion in the worst frame as a straight line
//! through the same two poses. That is the whole difference between a hand
//! moving at the speed of a hand and a hand over the ceiling in
//! `baked_motion_is_continuous`. The expressive eases are spent where the gaps
//! are long or the pose change is small -- the hang, the hold, the recoveries
//! -- and the fast half of each move is spent evenly. Nothing here was softened
//! to get under a ceiling; what moved was the timing either side of contact.

use crate::bake::{Key, Looseness, Recipe};
use crate::clips::locomotion::stance;
use crate::ease::Ease;
use view::clips::Clip;
use view::math::V3;
use view::pose::{ANKLE_ON_GROUND as GROUND, Pose};
use view::skeleton::Joint;

pub fn clips() -> Vec<Recipe> {
    vec![bloodletter(), rend(), grasp(), black_spike()]
}

// ---------------------------------------------------------------------------
// The body all three are thrown from
// ---------------------------------------------------------------------------

/// The two spots the Grasp stands on: square, wide, and level with each
/// other. Nothing else in the game stands like this, which is the point.
///
/// Seven centimetres ahead of where the idle's weight sits, and that is not a
/// detail. The move drops the hips more than twenty centimetres and holds
/// them there; with the feet under the body the shins lean far enough forward
/// to ask both ankles for more dorsiflexion than they have, the levelling
/// clamps, and the character stands in the floor for forty frames. Feet a
/// little forward and hips a little back is what a squat actually is.
const SQUARE_L: V3 = [-0.205, GROUND, 0.070];
const SQUARE_R: V3 = [0.205, GROUND, 0.070];

/// Where the idle's ankles are, read back out of the idle rather than typed
/// again. `plant_*` takes ankle positions and `joint_at` gives them back, so
/// this round-trips exactly and cannot drift when the stance is retuned.
fn ankles() -> (V3, V3) {
    let s = stance();
    (s.joint_at(Joint::FootL), s.joint_at(Joint::FootR))
}

/// How high an ankle can be and still have the foot carrying weight.
///
/// The idle's rear heel sits three centimetres up with the toe still down,
/// which is a bladed stance. Past four and a half it is not a stance, it is a
/// foot in the air, and the two want opposite ankle angles -- so the height
/// decides, once, rather than every pose having to remember to say.
const IN_CONTACT: f32 = 0.045;

/// Solve both legs for two ankle positions and sort the feet out afterwards.
///
/// A planted foot is solved from wherever the hips ended up, and the hips in
/// this file travel thirty centimetres inside one move. Keying the legs by hand
/// instead is how a caster ends up skating on the spot with their hand in the
/// floor.
///
/// A foot still in contact and raised **rolls onto its toe**, because tipping a
/// foot about an ankle that is still at floor height only drives the toe
/// through the boards. A foot that has left the floor is levelled instead. That
/// second case is not fussiness: a pointed foot is already several centimetres
/// nearer the ground than its ankle says, and the solver runs the ankle two
/// frames behind a shin that is folding fast, so a pointed airborne foot is the
/// shape that ends up back under the floorboards.
fn stand(pose: Pose, l: V3, r: V3) -> Pose {
    let rolled = |y: f32| y > GROUND + 0.008 && y < GROUND + IN_CONTACT;
    let pose = pose.plant_l(l).plant_r(r);
    let pose = if rolled(l[1]) {
        pose.toe_floor_l()
    } else {
        pose.toe_l(0.0)
    };
    if rolled(r[1]) {
        pose.toe_floor_r()
    } else {
        pose.toe_r(0.0)
    }
}

/// Put the feet back on the idle's two spots, with either ankle allowed to
/// rise -- a little for a heel coming up, a lot for a foot leaving the ground.
fn footing(pose: Pose, lift_l: f32, lift_r: f32) -> Pose {
    let (l, r) = ankles();
    stand(
        pose,
        [l[0], l[1] + lift_l, l[2]],
        [r[0], r[1] + lift_r, r[2]],
    )
}

/// The squared base, for the Grasp and nothing else.
fn rooted(pose: Pose, lift_l: f32, lift_r: f32) -> Pose {
    stand(
        pose,
        [SQUARE_L[0], SQUARE_L[1] + lift_l, SQUARE_L[2]],
        [SQUARE_R[0], SQUARE_R[1] + lift_r, SQUARE_R[2]],
    )
}

/// Neutral: the idle's footing, and a caster's own upper body over it.
///
/// Built on `locomotion::stance` so that entering a move and leaving it do not
/// move the feet -- the renderer cross-fades into an attack, and a fade covers
/// a change of shape, not a stamp. What is the Blood mage's own is everything
/// above the waist: rounder and lower than the idle, the lead hand open with
/// the palm turned up, the rear hand already in against the ribs. A fighter's
/// guard covers the head. This one covers the body it is going to spend.
fn ready() -> Pose {
    footing(
        stance()
            .hips(0.0, -0.082, -0.008)
            .root(4.0, 0.0, -11.0)
            .spine(11.0, 0.0, 7.0)
            .chest(7.0, 0.0, 6.0)
            .head(-8.0, 0.0, 6.0)
            .shoulder_l(10.0, 20.0, -18.0)
            .forearm_l(70.0, 32.0)
            .wrist_l(-20.0, 8.0, 0.0)
            .shoulder_r(-4.0, 13.0, -20.0)
            .forearm_r(100.0, -16.0)
            .wrist_r(-16.0, -6.0, 0.0),
        0.0,
        0.0,
    )
}

// ---------------------------------------------------------------------------
// Bloodletter
// ---------------------------------------------------------------------------

/// The auto: a blade thrown underarm, out to a fixed distance and back again.
///
/// The gesture is a **release**, not a strike, and the whole clip is arranged
/// so that the caster's own body never claims the hit. The move has no hitbox
/// of its own -- the blade in the air is the threat -- so an arm that finished
/// in a fist over somebody's head would be the animation telling a lie about
/// where the danger is. It finishes open instead.
///
/// Underarm rather than overarm for two reasons, and the second one is the real
/// one. The hand never rises above the elbow, so the silhouette stays low and
/// nothing about it can be confused with the Black spike's climbing hand. And a
/// low release is what puts the blade out at chest height on a flat line, which
/// is where it actually flies.
///
/// The recovery comes home to `ready()`, whose lead hand is already open with
/// the palm turned up. That is not a coincidence being taken advantage of: the
/// blade is coming back, and the pose a Blood mage waits in is a hand held out
/// to catch it. The catch itself is forty-odd frames after the clip is over --
/// the flight is much longer than the move -- so the animation cannot show it,
/// and the most it can do is be standing in the right shape when it happens.
///
/// It leaves the caster most of their walking speed, so the renderer blends a
/// real stride back in underneath. The legs here stay close to the idle's for
/// that reason: anything ambitious down there would be fighting a walk cycle.
fn bloodletter() -> Recipe {
    let clip = Clip::BloodPoke;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    // Drawn back by the first third of the startup. Any later and there is one
    // frame of arm travelling the whole way, which reads as a twitch.
    let draw = (contact / 3).max(2);
    let paid = recover + (end - recover) / 3;
    let home = end.saturating_sub(3);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "Seven frames of startup is under the reaction threshold, so \
                none of this is a telegraph -- what it owes an opponent is the \
                *shape*, and the shape is a low underarm draw and release. The \
                arm opens rather than closing: the move leaves no hitbox on the \
                caster, and a hand that finished in a fist would be claiming a \
                threat that is ten metres away by then. The draw is LINEAR \
                because five frames is not enough room to shape anything, and \
                the same reasoning as Rend applies to the release. The recovery \
                pays the class's usual bill -- hand to the ribs, chest shut over \
                it -- and then comes back to the open palm of `ready`, which is \
                the hand the blade is going to land in."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(draw, drawn(), Ease::LINEAR),
            Key::eased(contact, flung(), Ease::STRIKE),
            Key::eased(recover, emptied(), Ease::OUT),
            Key::eased(paid, spent(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// The draw, on frame two: the throwing hand pulled back and **down**, behind
/// the hip, palm turned up under the blade. The chest winds away with it.
///
/// The head turns back onto the line while the chest turns off it, the same
/// trick Rend uses: a body winding up to throw something at a particular place
/// has to keep looking at the place.
fn drawn() -> Pose {
    footing(
        ready()
            .hips(0.014, -0.090, -0.024)
            .root(5.0, 0.0, -20.0)
            .spine(13.0, 0.0, -12.0)
            .chest(8.0, 0.0, -20.0)
            .head(-12.0, 0.0, 24.0)
            .shoulder_l(-40.0, 16.0, -16.0)
            .forearm_l(34.0, 14.0)
            .wrist_l(-30.0, 10.0, 0.0)
            // The rear hand tightens in and stays out of the silhouette. It has
            // nothing to do with this move.
            .shoulder_r(-8.0, 10.0, -24.0)
            .forearm_r(112.0, -18.0)
            .wrist_r(-20.0, -8.0, 0.0),
        0.0,
        0.006,
    )
}

/// Release, on the frame the blade leaves: the arm has swung through to
/// roughly straight along the line, the hand is open, and the chest has
/// unwound past square.
///
/// The only nearly-straight elbow in the file, and it earns it -- this is the
/// one thing the class does at range, and every other move here is a body
/// folding onto something close.
fn flung() -> Pose {
    footing(
        ready()
            .hips(-0.010, -0.100, 0.040)
            .root(8.0, 0.0, -4.0)
            .spine(12.0, 0.0, 8.0)
            .chest(8.0, 0.0, 16.0)
            .head(-4.0, 0.0, -2.0)
            .shoulder_l(52.0, 10.0, 6.0)
            .forearm_l(16.0, -8.0)
            .wrist_l(14.0, -12.0, 0.0)
            .shoulder_r(-6.0, 12.0, -22.0)
            .forearm_r(108.0, -16.0)
            .wrist_r(-18.0, -8.0, 0.0),
        0.0,
        0.016,
    )
}

/// The first recovery frame: the throwing arm has come down out of the release
/// and the hand is still open. Nothing has closed yet.
fn emptied() -> Pose {
    footing(
        ready()
            .hips(-0.004, -0.120, 0.020)
            .root(9.0, 0.0, -8.0)
            .spine(18.0, 0.0, 6.0)
            .chest(11.0, 0.0, 9.0)
            .head(-14.0, 0.0, 2.0)
            .shoulder_l(26.0, 16.0, -14.0)
            .forearm_l(64.0, 16.0)
            .wrist_l(-14.0, -6.0, 0.0)
            .shoulder_r(-8.0, 11.0, -22.0)
            .forearm_r(112.0, -16.0)
            .wrist_r(-18.0, -8.0, 0.0),
        0.0,
        0.010,
    )
}

// ---------------------------------------------------------------------------
// Rend
// ---------------------------------------------------------------------------

/// A raking claw at chest-to-chest range, and the committed melee option.
///
/// It sat on the auto slot until the class got a real auto, and moving it to
/// shift + click is a change of weight rather than a change of gesture: the
/// same rake, given fourteen frames of wind-up instead of six and eighteen of
/// bill instead of thirteen. The keys are all derived from `clip.phases()`, so
/// the pose sequence stretched into the longer frame count by itself.
///
/// Fourteen frames is over the reaction threshold, so unlike the auto this one
/// **is** a telegraph, and the coil on frame four is what an opponent gets to
/// answer. What the animation owes them after that is the *range*: this is the
/// move in the class that happens close enough to touch, and the elbow stays
/// folded through the whole of it so the hand never gets far from the body.
/// An arm that straightens out in front is a thrust, and a thrust would teach
/// a spacing the move does not have.
///
/// It roots the caster, so there is no stride to blend underneath any more.
fn rend() -> Recipe {
    let clip = Clip::BloodCommitted;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    // The claw is out by the first third of the startup. Any later and there
    // is one frame of arm and nothing in front of it.
    let cock = (contact / 3).max(2);
    let paid = recover + (end - recover) / 5;
    let home = end.saturating_sub(3);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "More animal than martial: the head drops, the shoulders round \
                and the hand hooks rather than opening out. The elbow is folded \
                past ninety degrees on the contact frame on purpose -- a Rend \
                happens at chest-to-chest range and a straightened arm would \
                read as a thrust from twice the distance. The rake does not end \
                in the air: the claw carries through and comes back into the \
                caster's own sternum, which is the cost, and the lowest frame \
                of the clip is well after contact rather than anywhere in the \
                wind-up. The rake itself is LINEAR: a smooth ease across the \
                gap before contact puts a third of a metre of hand into the \
                single frame before the hitbox appears. MARTIAL rather than \
                CRISP because the claw wants to carry past and settle, and the \
                torso is the fast part of a martial body."
            .into(),
        keys: vec![
            // Out of neutral at full speed, and then a coil that holds. The
            // startup is over the reaction threshold now, so there is room for
            // a wind-up that waits -- and the waiting is the telegraph.
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(cock, cocked(), Ease::LINEAR),
            Key::eased(contact, raked(), Ease::STRIKE),
            Key::eased(recover, hooked(), Ease::OUT),
            Key::eased(paid, spent(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// The coil, on frame two: the lead hand pulled out and up past the shoulder,
/// the chest wound away from the target and the head dropped under it.
///
/// The head turns *back* onto the target while the chest turns off it. Winding
/// the head with the chest is how an aim turns into a swing, and this is not a
/// swing -- it is a thing about to be done to somebody who is already close.
fn cocked() -> Pose {
    footing(
        ready()
            .hips(0.018, -0.092, -0.028)
            .root(7.0, 0.0, -24.0)
            .spine(15.0, 0.0, -16.0)
            .chest(10.0, 0.0, -26.0)
            .head(-17.0, 0.0, 32.0)
            .shoulder_l(-30.0, 58.0, -34.0)
            .forearm_l(52.0, 30.0)
            .wrist_l(-48.0, 24.0, 0.0)
            // The rear hand tightens in rather than helping. It has nothing to
            // do with this move and it stays out of the silhouette.
            .shoulder_r(-8.0, 10.0, -24.0)
            .forearm_r(112.0, -18.0)
            .wrist_r(-20.0, -8.0, 0.0),
        0.0,
        0.006,
    )
}

/// Contact, on the frame the hitbox appears: the claw has come through across
/// the body, the chest has unwound past square, and the weight is on the lead
/// foot.
fn raked() -> Pose {
    footing(
        ready()
            .hips(-0.014, -0.106, 0.046)
            .root(9.0, 0.0, -2.0)
            .spine(14.0, 0.0, 9.0)
            .chest(9.0, 0.0, 19.0)
            .head(-6.0, 0.0, -4.0)
            .shoulder_l(34.0, 14.0, 18.0)
            .forearm_l(92.0, -18.0)
            .wrist_l(28.0, -20.0, 0.0)
            .shoulder_r(-6.0, 12.0, -22.0)
            .forearm_r(108.0, -16.0)
            .wrist_r(-18.0, -8.0, 0.0),
        0.0,
        0.018,
    )
}

/// The first recovery frame: the hand has finished the pull and arrived on the
/// caster's own chest, fingers shut, with the body folding over it.
fn hooked() -> Pose {
    footing(
        ready()
            .hips(-0.008, -0.124, 0.026)
            .root(9.0, 0.0, -6.0)
            .spine(20.0, 0.0, 7.0)
            .chest(13.0, 0.0, 10.0)
            .head(-18.0, 0.0, 2.0)
            .shoulder_l(-6.0, 10.0, -30.0)
            .forearm_l(124.0, 6.0)
            .wrist_l(-28.0, -12.0, 0.0)
            .shoulder_r(-8.0, 11.0, -22.0)
            .forearm_r(112.0, -16.0)
            .wrist_r(-18.0, -8.0, 0.0),
        0.0,
        0.010,
    )
}

/// The bill, four frames later, and the lowest frame in the clip. Both hands
/// are in against the ribs, the chest is shut over them and the head is down.
fn spent() -> Pose {
    footing(
        ready()
            .hips(0.004, -0.148, -0.012)
            .root(10.0, 0.0, -12.0)
            .spine(25.0, 0.0, 5.0)
            .chest(15.0, 0.0, 4.0)
            .head(-24.0, 0.0, 6.0)
            .shoulder_l(-10.0, 9.0, -28.0)
            .forearm_l(122.0, 10.0)
            .wrist_l(-24.0, -10.0, 0.0)
            .shoulder_r(-10.0, 10.0, -22.0)
            .forearm_r(116.0, -16.0)
            .wrist_r(-18.0, -8.0, 0.0),
        0.0,
        0.014,
    )
}

// ---------------------------------------------------------------------------
// Black spike
// ---------------------------------------------------------------------------

/// A hand driven into the floor, and something comes up a long way in front of
/// it.
///
/// On `E`, which is where the thing only this class does belongs -- her
/// mechanic is health, so there is no state for the key to toggle and it is
/// free to be an ability instead. It is also the slowest thing she has: thirty
/// frames of wind-up, twice the reaction threshold, because the ability is a
/// placement the other player is meant to see coming and step out of.
///
/// The power goes **down**, and everything is arranged to say so before it
/// happens: early in the wind-up the free hand drops and points along the
/// ground at the spot while the other climbs, the wind-up never goes higher
/// than the ear -- this body does not get an overhead -- and the descent is
/// hips and spine rather than shoulder. The spike erupts at `reach` on the
/// first active frame, nine metres from the hand that planted it, so the head
/// comes up off the floor at contact and watches the place it is going to
/// appear. A caster staring at their own knuckles while the interesting thing
/// happens nine metres away is the easiest way there is to make a ranged move
/// read as a whiff.
///
/// The hand finishes at knee height rather than flat on the floor. Going the
/// last twenty-five centimetres costs a fold that twenty frames of recovery
/// cannot stand up out of, and over a spine already bent thirty degrees it
/// reads as the same gesture.
fn black_spike() -> Recipe {
    let clip = Clip::BloodMechanic;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    // Committed to inside the first sixth of the startup, at the top by not
    // quite half, a beat of hang, and then falling for the whole of the rest.
    // Written in sixths and ninths rather than frame numbers, so a retune moves
    // the shape with it -- which is what happened when the cast went from
    // eighteen frames to thirty and this clip needed no edit at all.
    let open = (contact / 6).max(2);
    let top = (contact * 7 / 18).max(open + 2);
    let hang = (contact * 5 / 9).max(top + 1);
    let falling = (contact * 7 / 9).max(hang + 1);
    let peel = recover + (end - recover) * 7 / 20;
    let paid = recover + (end - recover) * 11 / 20;
    let home = end.saturating_sub(3);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "Thirty frames of startup is the longest telegraph in the class \
                by a distance, and all of it is spent going up so that there is \
                somewhere to come down from. The wind-up stops at the ear \
                rather than overhead, which is characterisation and also \
                arithmetic: a hand that starts above the head has a metre and a \
                half to cross before the hitbox appears, and no honest timing \
                gets it there. The descent is hips and spine rather than \
                shoulder -- the body falls onto the hand instead of the arm \
                throwing it -- and the lowest frame is the first recovery \
                frame, where the weight finishes arriving after the spike is \
                already out. The hips finish three centimetres *behind* where \
                they started rather than out over the lead foot, which looks \
                like the wrong direction for a stab and is not: hips over the \
                toes in a squat this deep ask \
                the lead ankle for more dorsiflexion than it has, the levelling \
                clamps, and the sole ends up pitched through the floor. Sitting \
                back is both what a heavy body does and what keeps the foot on \
                the ground. MARTIAL rather than HEAVY: HEAVY's forearms would \
                still be climbing a third of the way into the startup, \
                and the weight would arrive as lateness. It is in the pose \
                instead."
            .into(),
        keys: vec![
            // The tell: one hand climbing while the other drops and points
            // down the line. Two hands going opposite ways is the cheapest
            // silhouette change there is, and it is on screen by frame three.
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(open, opened(), Ease::SMOOTH),
            // HOLD is the telegraph. The pose barely changes between here and
            // the hang three frames later, so what an opponent gets is a body
            // stopped at the top of its reach with a hand behind its ear.
            // Thirty frames of startup is twice the reaction threshold, and
            // this is the part of it they get to react to.
            Key::eased(top, gathered(), Ease::HOLD),
            Key::eased(hang, hung(), Ease::LINEAR),
            // Eight frames of constant speed, not an ease. A body falling onto
            // its own hand does not accelerate into the floor and then back
            // off, and the ease that pretends it does spends a third of a
            // metre of hand in the middle frame. The mid-descent key
            // earns its place twice over: a straight blend from a standing pose
            // to a squat leaves the legs too straight half way down, and too
            // straight half way down is both feet through the floor.
            Key::eased(falling, falling_pose(), Ease::LINEAR),
            Key::eased(contact, driven(), Ease::STRIKE),
            Key::eased(recover, planted(), Ease::OUT),
            Key::eased(peel, peeled(), Ease::SMOOTH),
            Key::eased(paid, stooped(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// Frame three, and the whole of the early read: the rear hand starts up, the
/// lead hand turns over palm-down and reaches out low over the spot.
fn opened() -> Pose {
    footing(
        ready()
            .hips(0.010, -0.062, -0.020)
            .root(1.0, 0.0, -12.0)
            .spine(4.0, 0.0, 4.0)
            .chest(2.0, 0.0, 8.0)
            .head(0.0, 0.0, 5.0)
            .shoulder_r(56.0, 26.0, -18.0)
            .forearm_r(74.0, -30.0)
            .wrist_r(-30.0, 0.0, 0.0)
            .shoulder_l(30.0, 22.0, -42.0)
            .forearm_l(30.0, 42.0)
            .wrist_l(-26.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// The top of the wind-up, and it is not overhead. The hand is beside the ear
/// with the wrist rolled over into a reversed grip, the back is arched and the
/// rear heel is off the floor.
fn gathered() -> Pose {
    footing(
        ready()
            .hips(0.018, -0.044, -0.052)
            .root(-5.0, 0.0, -14.0)
            .spine(-9.0, 0.0, 2.0)
            .chest(-5.0, 0.0, 10.0)
            .head(8.0, 0.0, 4.0)
            .shoulder_r(104.0, 22.0, -42.0)
            .forearm_r(104.0, -56.0)
            .wrist_r(-38.0, 0.0, 0.0)
            .shoulder_l(40.0, 20.0, -46.0)
            .forearm_l(26.0, 38.0)
            .wrist_l(-24.0, 0.0, 0.0),
        0.0,
        0.026,
    )
}

/// The hang, eight frames before contact: a shade higher and further back,
/// both heels light, everything about to fall.
fn hung() -> Pose {
    footing(
        ready()
            .hips(0.022, -0.036, -0.062)
            .root(-8.0, 0.0, -14.0)
            .spine(-14.0, 0.0, 2.0)
            .chest(-8.0, 0.0, 11.0)
            .head(12.0, 0.0, 4.0)
            .shoulder_r(116.0, 20.0, -44.0)
            .forearm_r(96.0, -60.0)
            .wrist_r(-40.0, 0.0, 0.0)
            .shoulder_l(44.0, 20.0, -48.0)
            .forearm_l(22.0, 38.0)
            .wrist_l(-22.0, 0.0, 0.0),
        0.010,
        0.034,
    )
}

/// Half way down, and the key that keeps the descent honest.
///
/// The hips fall twenty-four centimetres in eight frames. A blend between a
/// standing pose and a squat does not describe what the legs do on the way --
/// it leaves them too straight at the midpoint, which pushes both feet through
/// the floor -- so the middle of the fall is a key rather than an average, with
/// the legs solved for it like every other pose here.
fn falling_pose() -> Pose {
    footing(
        driving(
            ready()
                .hips(0.014, -0.148, -0.048)
                .root(4.0, 0.0, -11.0)
                .spine(12.0, 0.0, 6.0)
                .chest(6.0, 0.0, 10.0)
                .head(-8.0, 0.0, 5.0),
            [0.20, 0.86, 0.36],
            -34.0,
        )
        .shoulder_l(40.0, 24.0, -40.0)
        .forearm_l(48.0, 34.0)
        .wrist_l(-24.0, 0.0, 0.0),
        0.0,
        0.032,
    )
}

/// Put the planting hand somewhere in the world and let the arm work it out.
///
/// The alternative is keying a shoulder angle, and a shoulder angle is measured
/// against a **chest** that this move pitches sixty degrees forward: the same
/// number that reaches for the floor at the top of the wind-up tucks the arm
/// under the ribs at the bottom of it. Saying where the hand goes instead
/// survives the fold, and the fold is the move.
fn driving(pose: Pose, wrist: V3, cock: f32) -> Pose {
    pose.reach_r(wrist).wrist_r(cock, 0.0, 0.0)
}

/// Contact, on the frame the spike erupts: the hand is driven down at the floor
/// in front of the lead foot, the body has folded over it, and the eyes have
/// already left it for the place the spike is coming out of.
///
/// The eyes matter more than they look. The spike appears most of the way
/// across the arena, and a caster staring at their own knuckles while the
/// interesting thing happens somewhere else is the easiest way to make a ranged
/// move read as a whiff.
///
/// Most of the fold is spine and chest rather than root. The root takes the
/// legs with it, so leaning it thirty degrees on top of a squat this deep asks
/// the hip for an angle it has already spent and swings the thigh up through
/// horizontal, where the decomposition has nothing useful left to say.
fn driven() -> Pose {
    footing(
        driving(
            ready()
                .hips(0.010, -0.272, -0.036)
                .root(12.0, 0.0, -9.0)
                .spine(30.0, 0.0, 6.0)
                .chest(15.0, 0.0, 8.0)
                .head(-40.0, 0.0, 6.0),
            [0.21, 0.42, 0.36],
            -44.0,
        )
        // The free hand braces on the lead thigh. It is what a heavy body does
        // on the way down, and it keeps that arm out of the way of the one
        // doing the work.
        .shoulder_l(30.0, 18.0, -34.0)
        .forearm_l(84.0, 26.0)
        .wrist_l(-16.0, 0.0, 0.0),
        0.0,
        0.056,
    )
}

/// The first recovery frame, and the lowest pose in the file: the weight has
/// finished arriving, the planting arm has locked out and the shoulders have
/// come up around the ears.
fn planted() -> Pose {
    footing(
        driving(
            ready()
                .hips(0.008, -0.310, -0.030)
                .root(14.0, 0.0, -9.0)
                .spine(33.0, 0.0, 6.0)
                .chest(15.0, 0.0, 8.0)
                .head(-40.0, 0.0, 6.0),
            [0.22, 0.33, 0.35],
            -48.0,
        )
        .shoulder_l(28.0, 22.0, -32.0)
        .forearm_l(90.0, 26.0)
        .wrist_l(-14.0, 0.0, 0.0),
        0.0,
        0.056,
    )
}

/// Peeling off the floor, and slower than the way down. The planting hand comes
/// up last; the free one has already started for the ribs.
fn peeled() -> Pose {
    footing(
        driving(
            ready()
                .hips(0.002, -0.196, -0.026)
                .root(10.0, 0.0, -10.0)
                .spine(22.0, 0.0, 6.0)
                .chest(12.0, 0.0, 6.0)
                .head(-26.0, 0.0, 6.0),
            [0.20, 0.60, 0.38],
            -30.0,
        )
        .shoulder_l(6.0, 13.0, -26.0)
        .forearm_l(104.0, -6.0)
        .wrist_l(-16.0, -8.0, 0.0),
        0.0,
        0.024,
    )
}

/// Upright, and paying for it: both hands in at the ribs, the chest shut over
/// them and the head down. The same punctuation Rend ends on, held longer
/// because this one cost more.
fn stooped() -> Pose {
    footing(
        ready()
            .hips(0.004, -0.152, -0.014)
            .root(10.0, 0.0, -12.0)
            .spine(26.0, 0.0, 6.0)
            .chest(15.0, 0.0, 5.0)
            .head(-26.0, 0.0, 6.0)
            .shoulder_r(-10.0, 10.0, -24.0)
            .forearm_r(118.0, -16.0)
            .wrist_r(-20.0, -8.0, 0.0)
            .shoulder_l(-8.0, 10.0, -26.0)
            .forearm_l(116.0, 4.0)
            .wrist_l(-22.0, -10.0, 0.0),
        0.0,
        0.008,
    )
}

// ---------------------------------------------------------------------------
// Grasp
// ---------------------------------------------------------------------------

/// Four arms thrown out in a cone and pulled back in to meet.
///
/// It replaced Reaper's debt on `Q`, and it inherited this clip rather than
/// getting a new one because the gesture was already exactly right: the arms
/// open steadily to their full span through the wind-up and then **shut
/// together in front of the sternum** on the frame the ability comes out. That
/// is the ability, drawn on the caster's own body -- wide, then a point. What
/// changed is the prose and the reasons, not the poses.
///
/// The stance is the thing to read. The feet come off the blade and square up
/// wide, level with each other, pointing down the line the arms are going to
/// leave along. Nothing else in the game stands like that. It used to be
/// justified by a channel that could not turn; the justification now is
/// **symmetry** -- this is the only move in the game with no lead side, four
/// arms leaving in four mirrored directions, and a body about to do something
/// symmetrical squares up to do it.
///
/// What the arms never do is come up: this is not a summoning, it is a body
/// being opened.
fn grasp() -> Recipe {
    let clip = Clip::BloodSpecial;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    // The feet are down inside the first sixth of the wind-up, the weight is
    // in by a third, and the arms spend the rest of it opening.
    let plant = (contact / 6).max(3);
    let sink = (contact / 3).max(plant + 2);
    let open = (contact * 2 / 3).max(sink + 2);
    let charged = contact.saturating_sub(1).max(open + 1);
    let sag = recover + (end - recover) / 4;
    let paid = recover + (end - recover) / 2;
    let home = end.saturating_sub(3);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "The clip is the ability: arms wide, then a point. They open \
                steadily through the whole wind-up and shut together in front \
                of the sternum on the frame the arms leave, which is the same \
                shape the four of them fly in -- out in a cone, back in to \
                converge. The feet square up by frame three and then do not \
                move again until the recovery is nearly over. That is the only \
                squared stance in the game, and it is here because this is the \
                only move with no lead side: two arms doing the same thing at \
                once, mirrored. The hips also sit back behind the ankles rather \
                than over them: a squat this deep with the weight forward asks \
                the ankles for dorsiflexion they do not have and puts both \
                soles through the floor, and sitting back is what a body being \
                emptied does anyway. The release is the hands shutting together \
                rather than a sweep, which keeps the fastest thing in the clip \
                travelling forty centimetres instead of ninety. The recovery \
                carries the class's whole identity: the knees give, the hands \
                come to the ribs, and the feet are the last thing to come back. \
                MARTIAL rather than HEAVY -- HEAVY arms run three and a half \
                frames behind the keys, and a release that arrives three and a \
                half frames after the hitbox teaches the wrong frame to \
                everybody watching. The weight is in the pose and in the \
                recovery, which is where it belongs."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(plant, squared(), Ease::SMOOTH),
            Key::eased(sink, sunk(), Ease::SMOOTH),
            Key::eased(open, opening(), Ease::SMOOTH),
            // The last frame of the wind-up, and the pose an opponent spends
            // the tail of it looking at. One frame from here to contact, left
            // at full speed: a release is a release, and the single-frame gap
            // is what makes the difference between the wind-up and the strike
            // legible instead of smeared across the whole of the hold.
            Key::eased(charged, wide(), Ease::OUT),
            Key::eased(contact, released(), Ease::STRIKE),
            Key::eased(recover, shut(), Ease::OUT),
            Key::eased(sag, sagging(), Ease::SMOOTH),
            Key::eased(paid, drained(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// Frame three: off the blade and onto two square feet, a little forward of
/// where the idle stands. The hips have turned to face the line the arms will
/// leave along, and the head has stopped compensating for a turn that is no
/// longer there.
fn squared() -> Pose {
    rooted(
        ready()
            .hips(0.0, -0.096, -0.020)
            .root(3.0, 0.0, 0.0)
            .spine(8.0, 0.0, 0.0)
            .chest(5.0, 0.0, 0.0)
            .head(-6.0, 0.0, 0.0)
            .shoulder_l(6.0, 26.0, -22.0)
            .forearm_l(74.0, 34.0)
            .wrist_l(-18.0, 10.0, 0.0)
            .shoulder_r(6.0, 26.0, -22.0)
            .forearm_r(74.0, 34.0)
            .wrist_r(-18.0, 10.0, 0.0),
        0.0,
        0.0,
    )
}

/// The weight goes down and stays down. Knees out over the feet, hips between
/// them, nothing on either heel.
fn sunk() -> Pose {
    rooted(
        ready()
            .hips(0.0, -0.164, -0.056)
            .root(7.0, 0.0, 0.0)
            .spine(12.0, 0.0, 0.0)
            .chest(6.0, 0.0, 0.0)
            .head(-8.0, 0.0, 0.0)
            .shoulder_l(20.0, 34.0, -30.0)
            .forearm_l(66.0, 44.0)
            .wrist_l(-22.0, 12.0, 0.0)
            .shoulder_r(20.0, 34.0, -30.0)
            .forearm_r(66.0, 44.0)
            .wrist_r(-22.0, 12.0, 0.0),
        0.0,
        0.0,
    )
}

/// Two thirds of the way through the wind-up: the arms have come out and down,
/// the palms have rolled to face each other, and the chest has hollowed.
fn opening() -> Pose {
    rooted(
        ready()
            .hips(0.0, -0.196, -0.082)
            .root(9.0, 0.0, 0.0)
            .spine(16.0, 0.0, 0.0)
            .chest(4.0, 0.0, 0.0)
            .head(-14.0, 0.0, 0.0)
            .shoulder_l(26.0, 62.0, -44.0)
            .forearm_l(46.0, 58.0)
            .wrist_l(-26.0, 16.0, 0.0)
            .shoulder_r(26.0, 62.0, -44.0)
            .forearm_r(46.0, 58.0)
            .wrist_r(-26.0, 16.0, 0.0),
        0.0,
        0.0,
    )
}

/// The last frame of the wind-up and the widest the arms get: out to their
/// full span, chest hollow, head down between them. This is the cone the four
/// arms are about to leave in.
fn wide() -> Pose {
    rooted(
        ready()
            .hips(0.0, -0.214, -0.094)
            .root(10.0, 0.0, 0.0)
            .spine(18.0, 0.0, 0.0)
            .chest(2.0, 0.0, 0.0)
            .head(-18.0, 0.0, 0.0)
            .shoulder_l(30.0, 82.0, -52.0)
            .forearm_l(34.0, 64.0)
            .wrist_l(-28.0, 20.0, 0.0)
            .shoulder_r(30.0, 82.0, -52.0)
            .forearm_r(34.0, 64.0)
            .wrist_r(-28.0, 20.0, 0.0),
        0.0,
        0.0,
    )
}

/// Contact: the arms shut. The hands drive together in front of the sternum --
/// the convergence, drawn on the caster half a second before the four arms
/// out in the world do the same thing -- and the whole body goes forward over a
/// base it has not moved off.
fn released() -> Pose {
    rooted(
        ready()
            .hips(0.0, -0.176, 0.010)
            .root(16.0, 0.0, 0.0)
            .spine(10.0, 0.0, 0.0)
            .chest(-4.0, 0.0, 0.0)
            .head(-4.0, 0.0, 0.0)
            .shoulder_l(58.0, 12.0, -12.0)
            .forearm_l(58.0, 10.0)
            .wrist_l(-14.0, -14.0, 0.0)
            .shoulder_r(58.0, 12.0, -12.0)
            .forearm_r(58.0, 10.0)
            .wrist_r(-14.0, -14.0, 0.0),
        0.0,
        0.0,
    )
}

/// The first recovery frame: the hands have crossed past each other, the body
/// has folded over them, and there is nothing left holding it up.
fn shut() -> Pose {
    rooted(
        ready()
            .hips(0.0, -0.246, -0.030)
            .root(20.0, 0.0, 0.0)
            .spine(24.0, 0.0, 0.0)
            .chest(10.0, 0.0, 0.0)
            .head(-14.0, 0.0, 0.0)
            .shoulder_l(34.0, 4.0, -6.0)
            .forearm_l(112.0, -4.0)
            .wrist_l(-22.0, -20.0, 0.0)
            .shoulder_r(34.0, 4.0, -6.0)
            .forearm_r(112.0, -4.0)
            .wrist_r(-22.0, -20.0, 0.0),
        0.0,
        0.0,
    )
}

/// The knees give. The lowest frame of the clip, and it is seven frames after
/// the damage happened.
fn sagging() -> Pose {
    rooted(
        ready()
            .hips(0.0, -0.298, -0.078)
            .root(22.0, 0.0, 0.0)
            .spine(30.0, 0.0, 0.0)
            .chest(14.0, 0.0, 0.0)
            .head(-24.0, 0.0, 0.0)
            // Nothing is being held. The arms hang off the shoulders and the
            // wrists are wherever the forearms left them.
            .shoulder_l(6.0, 14.0, -8.0)
            .forearm_l(48.0, -10.0)
            .wrist_l(-8.0, -6.0, 0.0)
            .shoulder_r(6.0, 14.0, -8.0)
            .forearm_r(48.0, -10.0)
            .wrist_r(-8.0, -6.0, 0.0),
        0.0,
        0.0,
    )
}

/// Straightening, and still square. Both hands have come in to the ribs; the
/// feet have not been asked for yet.
fn drained() -> Pose {
    rooted(
        ready()
            .hips(0.0, -0.182, -0.060)
            .root(13.0, 0.0, 0.0)
            .spine(24.0, 0.0, 0.0)
            .chest(13.0, 0.0, 0.0)
            .head(-22.0, 0.0, 0.0)
            .shoulder_l(-8.0, 11.0, -24.0)
            .forearm_l(114.0, 6.0)
            .wrist_l(-20.0, -10.0, 0.0)
            .shoulder_r(-8.0, 11.0, -24.0)
            .forearm_r(114.0, -14.0)
            .wrist_r(-20.0, -8.0, 0.0),
        0.0,
        0.0,
    )
}
