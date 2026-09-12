//! The Blood mage: Rend, Black spike, Reaper's debt.
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
//! ## The three read as three distances
//!
//! An opponent has to know which one is coming, and here they are told by
//! where the caster is *pointing* in the first three frames:
//!
//! ```text
//!   Rend           one hand cocked out and up, close in    -- at you
//!   Black spike    one hand climbing, one pointing low     -- at the floor
//!   Reaper's debt  the feet square and both arms open      -- at nothing; planted
//! ```
//!
//! Reaper's debt is the odd one and deliberately so: it is the only pose in
//! the game that squares its feet. Everything else in the repository stands
//! bladed, on `locomotion::stance`, because a bladed fighter can turn. A
//! channel cannot -- `you cannot turn while it runs` is the whole cost of the
//! move -- so it plants two feet on a line, facing where it is going to fire,
//! and does not get them back until the recovery is nearly over.
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
    vec![rend(), black_spike(), reapers_debt()]
}

// ---------------------------------------------------------------------------
// The body all three are thrown from
// ---------------------------------------------------------------------------

/// The two spots the channel stands on: square, wide, and level with each
/// other. Nothing else in the game stands like this, which is the point.
///
/// Seven centimetres ahead of where the idle's weight sits, and that is not a
/// detail. The channel drops the hips more than twenty centimetres and holds
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

/// The squared base, for the channel and nothing else.
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
// Rend
// ---------------------------------------------------------------------------

/// A raking claw at chest-to-chest range, and the neutral tool.
///
/// Six frames of startup is under the reaction threshold, so none of this is a
/// telegraph -- the opponent cannot answer a Rend on sight, only learn its
/// shape. What the animation owes them instead is the *range*: this is the one
/// move in the class that happens close enough to touch, and the elbow stays
/// folded through the whole of it so the hand never gets far from the body.
/// An arm that straightens out in front is a thrust, and a thrust would teach
/// a spacing the move does not have.
///
/// It leaves the caster most of their walking speed, so the renderer blends a
/// real stride back in underneath. The legs here stay close to the idle's for
/// that reason: anything ambitious down there would be fighting a walk cycle.
fn rend() -> Recipe {
    let clip = Clip::BloodPoke;
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
                of the clip is five frames after contact rather than anywhere \
                in the wind-up. The rake itself is LINEAR: four frames is not \
                enough room to shape it, and a smooth ease across that gap puts \
                a third of a metre of hand into the single frame before the \
                hitbox appears. MARTIAL rather than CRISP because the claw \
                wants to carry past and settle -- with six frames of startup \
                the telegraph has to come from the torso anyway, and the torso \
                is the fast part of a martial body."
            .into(),
        keys: vec![
            // Out of neutral at full speed. Six frames of startup leaves no
            // room for a wind-up that waits, so the coil lands on frame two
            // and OUT is what gets it there.
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

/// A hand driven into the floor, and something comes up two and a half metres
/// in front of it.
///
/// The power goes **down**, and everything is arranged to say so before it
/// happens: on frame three the free hand drops and points along the ground at
/// the spot while the other climbs, the wind-up never goes higher than the ear
/// -- this body does not get an overhead -- and the descent is hips and spine
/// rather than shoulder. The spike erupts at `reach` on the first active frame,
/// two and a half metres from the hand that planted it, so the head comes up
/// off the floor at contact and watches the place it is going to appear. A
/// caster staring at their own knuckles while the interesting thing happens two
/// metres away is the easiest way there is to make a ranged move read as a
/// whiff.
///
/// The hand finishes at knee height rather than flat on the floor. Going the
/// last twenty-five centimetres costs a fold that twenty frames of recovery
/// cannot stand up out of, and over a spine already bent thirty degrees it
/// reads as the same gesture.
fn black_spike() -> Recipe {
    let clip = Clip::BloodCommitted;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    // Committed to inside the first sixth of the startup, at the top by not
    // quite half, a beat of hang, and then falling for the whole of the rest.
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
        notes: "Eighteen frames of startup is the second longest telegraph the \
                class has, and all of it is spent going up so that there is \
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
                still be climbing at frame eight of an eighteen-frame startup, \
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
            // Eighteen frames of startup is well over the reaction threshold,
            // and this is the part of it they get to react to.
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
/// The eyes matter more than they look. The spike appears two and a half metres
/// away, and a caster staring at their own knuckles while the interesting thing
/// happens somewhere else is the easiest way to make a ranged move read as a
/// whiff.
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
// Reaper's debt
// ---------------------------------------------------------------------------

/// The channel. Twenty-two frames of holding still in one direction, five of
/// releasing, and twenty-eight of paying for it.
///
/// The rule the whole clip exists to communicate is that **facing is locked**.
/// A player who does not know that walks into it sideways and never learns
/// why they lived; a player who does know it treats the caster as a piece of
/// terrain for half a second, which is the move. So the first thing that
/// happens is the feet: they come off the blade and square up wide, level with
/// each other, pointing at the thing that is about to be fired at. Nothing
/// else in the game stands like that, and it is legible from across the arena
/// long before the arms mean anything.
///
/// The cone widens the longer it is held, so the arms open all the way through
/// the startup rather than reaching a pose and waiting in it. What they never
/// do is come up: this is not a summoning, it is a body being opened.
fn reapers_debt() -> Recipe {
    let clip = Clip::BloodSpecial;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    // The feet are down inside the first sixth of the channel, the weight is
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
        notes: "A channel, so the startup is not a wind-up: nothing here \
                gathers and then goes. The feet square up by frame three and \
                then do not move again until the recovery is nearly over, \
                which is the pose saying out loud that the caster cannot turn \
                -- every other stance in the game is bladed, because a bladed \
                fighter can. The hips also sit back behind the ankles rather \
                than over them: a squat this deep with the weight forward asks \
                the ankles for dorsiflexion they do not have and puts both \
                soles through the floor, and sitting back is what a body being \
                emptied does anyway. The arms open steadily across the whole \
                twenty-two frames because the cone widens with the hold, and \
                they open *downward and out* rather than up -- this is a body \
                being opened, not a spell being summoned. The release is the \
                hands shutting together in front of the sternum rather than a \
                sweep, which keeps the fastest thing in the clip travelling \
                forty centimetres instead of ninety. Twenty-eight frames of \
                recovery, and the class's whole identity is in them: the knees \
                give, the hands come to the ribs, and the feet are the last \
                thing to come back. MARTIAL rather than HEAVY, which looks like \
                the wrong call for the longest move in the game and is not -- \
                HEAVY arms run three and a half frames behind the keys, and a \
                release that arrives three and a half frames after the hitbox \
                teaches the wrong frame to everybody watching. The weight is in \
                the pose and in the twenty-eight frames afterwards, which is \
                where it belongs."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(plant, squared(), Ease::SMOOTH),
            Key::eased(sink, sunk(), Ease::SMOOTH),
            Key::eased(open, opening(), Ease::SMOOTH),
            // The last frame of the channel, and the pose an opponent spends
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
/// where the idle stands. The hips have turned eleven degrees to face the line
/// and the head has stopped compensating for a turn that is no longer there.
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

/// Two thirds of the way through the channel: the arms have come out and down,
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

/// The last frame of the channel and the widest the cone gets: arms out to
/// their full span, chest hollow, head down between them.
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

/// Contact: the cone shuts. The hands drive together in front of the sternum
/// and the whole body goes forward over a base it cannot step out of.
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
