//! The Blood mage: fourteen moves, thralls, stains and grey.
//!
//! ## What the class fights like
//!
//! Her blood goes out, and theirs comes back. There is no meter to run dry and
//! no resource to husband: every cost is a stain on the floor where she aimed,
//! every hit is a stain, and her thralls are stains stood up on legs. So the
//! class is never *spending mana*, it is spending itself, and the body has to
//! say that on every move or the mechanic is invisible to the person opposite.
//! The kit is `docs/design/kits/blood-mage.md` §"Thralls and grey".
//!
//! Three things carry it, and they are in every clip here:
//!
//! - **The recovery is heavier than the wind-up.** The frame data already says
//!   so -- most of her rows are twelve frames of startup against eighteen of
//!   recovery, and the finishers thirty against thirty -- and the poses agree
//!   with it rather than fighting it. The lowest, slowest, most folded frame of
//!   each of these clips is *after* the move comes out, not before.
//! - **A hand comes back to the ribs.** Every recovery in this file ends with
//!   a hand drawn in against the caster's own body and the chest closed over
//!   it (`spent`, `stooped`). It is the one gesture repeated across all of
//!   them, so a spectator learns it as the class's punctuation: that was not
//!   free.
//! - **Nothing is athletic.** `build_for(BloodMage)` is ordinary proportions
//!   carrying more bulk than they should, and the poses are built for that
//!   body. Nothing on the floor goes past the ear, no lunge is long, and the
//!   knees give at the end of everything. The three exceptions go overhead
//!   because their contracts say so -- the Stake, thrown from the air, and the
//!   Blood golem, a finisher -- and they are the only frames in the file where
//!   a hand is above the head, which is what makes them read as the big ones.
//!
//! ## Fourteen shapes, told apart by where the hands go
//!
//! ```text
//!   Command        one arm thrown out, pointing down at a spot   -- over there
//!   Haemorrhage    one hand drawn in to the chest, then out flat -- at you
//!   Hemoplague     both hands spread palm-down over the floor    -- that floor
//!   Carpet         crouched, both palms on the boards            -- this floor
//!   Black spike    one hand climbing to the ear, then the floor  -- under you
//!   Summon         both arms swept up from low, palms up         -- stand up
//!   Coagulate      both arms reaching out, then hauled in        -- come here
//!   Blood wings    arms folded, then flung back wide             -- away, flying
//!   Upheaval       one hand low by the foot, ripped up past the face, a hop
//!   Stake          in the air, both hands overhead, hurled down
//!   Rising reap    both hands on a haft, low left to high right
//!   Black vines    fists together, driven down, wrenched apart
//!   Crimson cascade two hands clasped at the ear, stabbed down, thrown back
//!   Blood golem    both arms high, dragged down, the body bowed
//! ```
//!
//! The one-armed moves are thrown with the **left** hand, which is the hand
//! `ready` holds out open, except the two that tear something up out of the
//! floor (the spike and the Upheaval), which are the right: the rear hand is the
//! one that has room to climb.
//!
//! **The scythe is not drawn by these clips.** The Rising reap puts the hands
//! where a haft would be held, and the game hangs the blade off the grip at
//! the reach the hit test uses -- see `view::scythe`. That is the one move
//! whose reach is allowed to grow with her grey, and it is why the hands there
//! are placed rather than posed: a grip that wandered off the haft would be a
//! blade drawn from the wrong place.
//!
//! ## The two fields are held on their first frame
//!
//! The Hemoplague and the Carpet are held clicks: the tap's startup plays (the
//! Haemorrhage's, Command's), and then for as long as the button is down the
//! renderer holds **frame zero** of the field's clip (`view::play::attack`,
//! `Action::Channel`), and on the release it cuts straight to the field's
//! first recovery frame, with no fade, because a channel and its recovery are
//! one shape. So in these two clips frame zero is *the hold* -- the hands
//! spread over the floor, the palms on it -- and the first recovery frame is
//! the same pose a shade on, so the cut is not a pop. Their startups are a
//! lift and a press back down, which is the move as it would be thrown without
//! a hold and is what the tests read the three phases off. The finisher a hold
//! becomes (the vines, the golem) is a different move and fades in, and its
//! frame zero is that hold again so the fade has nothing to cover.
//!
//! ## Three of them leave the floor
//!
//! Blood wings, the Upheaval and the Rising reap are thrown with space, on the
//! takeoff window, so they start on or just off the floor; the Stake is thrown
//! from the air. The wings and the Upheaval open on the floor and finish with
//! the legs off it (`aloft`), toes pointed and then tucking, so the jump clips
//! can take over. The Stake is in the air from its first frame to its last. The
//! Rising reap keeps the floor under it -- it is the scythe's swing, and its
//! legs are the sweep's -- but finishes up on the rear toe.
//!
//! ## Frames come from the move table
//!
//! Every key here is derived from `clip.phases()`, read live from the Oven, so
//! retuning a startup moves the animation with it. Two boundaries are used
//! directly rather than scaled: the **contact pose sits on the first active
//! frame**, which is the frame the effect comes out, and the **heaviest frame
//! sits on or after the first recovery frame**, because a body that is still
//! arriving after the thing has happened is what weight looks like.
//!
//! The last key of each clip sits a few frames short of the end rather than on
//! it. The arms run a couple of frames behind the keys, so a return keyed on
//! the final frame is a return the body never actually reaches -- and the last
//! frame is the one control comes back on, which is where a pop would show.
//!
//! ## Why the fast parts are LINEAR
//!
//! A shaped ease spends about half its travel in the middle of its span, so
//! across the eight frames of the Haemorrhage's thrust or the spike's descent it
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
    vec![
        command(),
        haemorrhage(),
        hemoplague(),
        black_spike(),
        sweep(),
        carpet(),
        summon(),
        wings(),
        upheaval(),
        stake(),
        coagulate(),
        vines(),
        cascade(),
        golem(),
    ]
}

// ---------------------------------------------------------------------------
// The body every move is thrown from
// ---------------------------------------------------------------------------

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
// The scythe: a grip, and the one swing of it
// ---------------------------------------------------------------------------

/// Both hands on the haft.
///
/// `lead` is the hand nearer the blade and `rear` the one at the butt of the
/// pole; the right hand leads, because the reap is drawn from the body's
/// left across to its right -- and the right hand is the one the scythe stands
/// under at rest, so the left is free to point and to cast.
/// Character space: `+Z` forward, `+Y` up, the left hand at `-X`.
///
/// Placed with the solver rather than keyed as angles, for the reason the
/// spike's planting hand is: the torso in these moves bends thirty degrees
/// and turns fifty, and a shoulder angle that puts a hand on the haft in one
/// of those attitudes puts it in the ribs in the other. Saying where the
/// hands go survives the fold, and the fold is the move.
fn gripping(pose: Pose, lead: V3, rear: V3, cock: f32) -> Pose {
    pose.reach_r(lead)
        .reach_l(rear)
        .wrist_r(cock, 0.0, 0.0)
        .wrist_l(cock, 0.0, 0.0)
}

// ---------------------------------------------------------------------------
// Haemorrhage
// ---------------------------------------------------------------------------

/// The bolt: one hand drawn in to the chest and thrust out flat, palm forward.
///
/// A tapped left click, on the floor and in the air: a bolt that opens a
/// bleed. Twelve frames of startup is under the reaction threshold, so it owes
/// the opponent a shape rather than a telegraph -- and the shape is the hand
/// coming **in** before it goes out, to the chest with the palm turned
/// forward, then thrust out level. Command throws the same arm down at the
/// floor; same arm, two lines, and the angle is the difference between "you"
/// and "there".
///
/// The thrust is LINEAR across its eight frames for the reason the sweep's
/// crossing is: a shaped ease would put half the reach of an arm into one
/// frame. The arm finishes straight and open -- the bolt is in the air and
/// the hand has nothing in it -- and the recovery is the class's usual bill.
///
/// Held past its startup, the click becomes the Hemoplague, whose clip
/// takes over from this one's wind-up. The scythe stays standing under the
/// other hand throughout: this is a spell, not a swing.
fn haemorrhage() -> Recipe {
    let clip = Clip::BloodCommitted;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let gather = (contact * 2 / 5).max(2);
    let paid = recover + (end - recover) / 3;
    let home = end.saturating_sub(3);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "Twelve frames in, three out, eighteen back. The gather is \
                SMOOTH because it is small and has room; the thrust is LINEAR \
                because eight frames is not enough room to shape a straight \
                arm's worth of travel. The hand finishes open and forward, \
                claiming nothing at the body -- the bolt is the threat and it \
                has left -- and the recovery draws it back to the ribs."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(gather, drawn_in(), Ease::SMOOTH),
            Key::eased(contact, loosed(), Ease::LINEAR),
            Key::eased(recover, emptied(), Ease::OUT),
            Key::eased(paid, spent(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// The gather, a few frames in: the casting hand drawn in against the chest,
/// palm turned forward, the elbow folded and the chest turned a little away
/// behind it. The head stays on the line.
fn drawn_in() -> Pose {
    footing(
        ready()
            .hips(0.010, -0.088, -0.016)
            .root(6.0, 0.0, -16.0)
            .spine(12.0, 0.0, -8.0)
            .chest(8.0, 0.0, -12.0)
            .head(-10.0, 0.0, 16.0)
            .shoulder_l(20.0, 30.0, -34.0)
            .forearm_l(118.0, 24.0)
            .wrist_l(-30.0, 10.0, 0.0)
            .shoulder_r(-8.0, 10.0, -24.0)
            .forearm_r(112.0, -18.0)
            .wrist_r(-20.0, -8.0, 0.0),
        0.0,
        0.006,
    )
}

/// The thrust, on the frame the bolt leaves: the arm straight out in front at
/// chest height, the hand open with the palm forward, the chest turned in
/// behind it and the weight on the front foot.
fn loosed() -> Pose {
    footing(
        ready()
            .hips(-0.010, -0.100, 0.044)
            .root(8.0, 0.0, 2.0)
            .spine(12.0, 0.0, 10.0)
            .chest(8.0, 0.0, 18.0)
            .head(-6.0, 0.0, -4.0)
            .shoulder_l(66.0, 8.0, 2.0)
            .forearm_l(10.0, -4.0)
            .wrist_l(20.0, -10.0, 0.0)
            .shoulder_r(-6.0, 12.0, -22.0)
            .forearm_r(108.0, -16.0)
            .wrist_r(-18.0, -8.0, 0.0),
        0.0,
        0.016,
    )
}

/// The Haemorrhage's first recovery frame: the casting arm has come down out
/// of the thrust and the hand is still open. Nothing has closed yet.
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
// Rising reap
// ---------------------------------------------------------------------------

/// Space and right click: a cone thrown forward off the scythe that hurls her
/// thralls along and drinks the stains it crosses.
///
/// The scythe swung up and out: both hands gathered low off the left hip and
/// swept across the front to finish high and forward off the right shoulder.
/// The crossing happens **during the active frames and not before them**,
/// keyed twice across the window, because that is when the cone goes out --
/// the arms and the thing they send have to be on the same side of the body at
/// the same time. Left to right, low to high, which is what the reap was
/// before it rose and is the scythe's one stroke.
///
/// It is a takeoff, thrown on the jump's window, so it may start on the floor
/// or a frame off it; the legs stay close to the idle's, and the finish comes
/// up on the rear toe, which reads either way. `view::scythe` hangs the blade
/// off the two hands.
fn sweep() -> Recipe {
    let clip = Clip::BloodSweep;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let wind = (contact * 2 / 5).max(2);
    let across = contact + (recover - contact) / 2;
    let paid = recover + (end - recover) / 3;
    let home = end.saturating_sub(3);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "Both hands on the haft, gathered low off the left hip, swept \
                up and out across the front to finish high and forward off \
                the right shoulder, the rear heel coming up. The wind is \
                LINEAR because three frames is not enough room to shape \
                anything; the sweep is LINEAR too, keyed at contact, half way \
                across and on the first recovery frame, so the hands cross at \
                the rate the cone goes out. Low to high across the arc: a war \
                scythe is swung up from the hip, and the rising line is what \
                tells it from the Dual mage's level Sweep at a glance. The \
                recovery pays the class's usual bill -- hands to the ribs -- \
                and comes back to `ready`."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(wind, gathered_left(), Ease::LINEAR),
            Key::eased(contact, entering_low(), Ease::STRIKE),
            Key::eased(across, crossing(), Ease::LINEAR),
            Key::eased(recover, carried_high(), Ease::LINEAR),
            Key::eased(paid, spent(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// The wind: both hands folded in off the left hip, the chest turned that way
/// and the head kept on the line.
fn gathered_left() -> Pose {
    footing(
        gripping(
            ready()
                .hips(0.012, -0.090, -0.020)
                .root(4.0, 0.0, 18.0)
                .spine(10.0, 0.0, 14.0)
                .chest(6.0, 0.0, 16.0)
                .head(-8.0, 0.0, -26.0),
            [-0.18, 0.82, 0.28],
            [-0.36, 0.70, 0.02],
            -16.0,
        ),
        0.006,
        0.0,
    )
}

/// Contact: the blade has come round to the front-left, low, and the hips
/// have turned under the arms a frame ahead of them.
fn entering_low() -> Pose {
    footing(
        gripping(
            ready()
                .hips(0.000, -0.100, 0.020)
                .root(6.0, 0.0, 6.0)
                .spine(10.0, 0.0, 4.0)
                .chest(6.0, 0.0, 6.0)
                .head(-6.0, 0.0, -8.0),
            [-0.12, 0.76, 0.56],
            [-0.32, 0.68, 0.34],
            -18.0,
        ),
        0.0,
        0.006,
    )
}

/// Half way across: the haft square in front at chest height, the chest
/// through square and turning the other way.
fn crossing() -> Pose {
    footing(
        gripping(
            ready()
                .hips(-0.008, -0.100, 0.040)
                .root(6.0, 0.0, -8.0)
                .spine(8.0, 0.0, -6.0)
                .chest(4.0, 0.0, -8.0)
                .head(-6.0, 0.0, 4.0),
            [0.18, 1.04, 0.56],
            [-0.06, 0.88, 0.48],
            -12.0,
        ),
        0.0,
        0.012,
    )
}

/// The first recovery frame: the blade has finished high off the right
/// shoulder, the body turned after it, and the hands are about to start back.
fn carried_high() -> Pose {
    footing(
        gripping(
            ready()
                .hips(-0.010, -0.110, 0.030)
                .root(6.0, 0.0, -20.0)
                .spine(8.0, 0.0, -14.0)
                .chest(4.0, 0.0, -16.0)
                .head(-8.0, 0.0, 16.0),
            [0.40, 1.38, 0.34],
            [0.18, 1.14, 0.40],
            -8.0,
        ),
        0.0,
        0.032,
    )
}

/// The bill, a few frames into the recovery: both hands in against the ribs,
/// the chest shut over them and the head down. The class's punctuation, and
/// every clip here ends on it.
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
/// On the middle click, charged in tiers while it is held (one spike to
/// seven); held to its end it is the Crimson cascade, which is this gesture
/// with both hands. It is the slowest of her taps: thirty frames of wind-up, twice the reaction threshold, because the ability is a
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
/// them and the head down. The same punctuation the Haemorrhage ends on, held longer
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
// Small pieces every new move is built from
// ---------------------------------------------------------------------------

/// The right hand in against the ribs, the forearm across the belly. The
/// class's punctuation, as a piece to put on any pose.
fn ribs_r(pose: Pose) -> Pose {
    pose.shoulder_r(-10.0, 10.0, -22.0)
        .forearm_r(116.0, -16.0)
        .wrist_r(-18.0, -8.0, 0.0)
}

/// The left hand to the ribs, the same way.
fn ribs_l(pose: Pose) -> Pose {
    pose.shoulder_l(-10.0, 9.0, -28.0)
        .forearm_l(122.0, 10.0)
        .wrist_l(-24.0, -10.0, 0.0)
}

/// Both hands placed in character space, left then right: `+Z` forward,
/// `+Y` up, the floor at zero, the left hand at `-X`. Placed rather than posed
/// for the reason `gripping` is: these moves fold the torso thirty degrees and
/// more, and a shoulder angle that puts a hand over the floor standing puts it
/// in the knees crouched.
fn hands(pose: Pose, l: V3, r: V3) -> Pose {
    pose.reach_l(l).reach_r(r)
}

/// Legs off the floor. `tuck` at 0 trails them long and pointed behind a body
/// that has just left the ground; at 1 the knees are drawn up under it, which
/// is the shape the jump's own hang has and what the clip hands over to.
///
/// No `plant_*` here and no footing: a foot in the air has nothing to be
/// solved against, and levelling one to a floor it is not on is how a pointed
/// toe ends up pitched flat in mid-air.
fn aloft(pose: Pose, tuck: f32) -> Pose {
    pose.hips(0.0, 0.02 - 0.05 * tuck, 0.0)
        .hip_l(4.0 + 50.0 * tuck, 7.0, 0.0)
        .knee_l(24.0 + 62.0 * tuck)
        .hip_r(-4.0 + 46.0 * tuck, 6.0, 0.0)
        .knee_r(30.0 + 56.0 * tuck)
        .ankles(34.0 - 14.0 * tuck, 0.0, 0.0)
}

// ---------------------------------------------------------------------------
// Command
// ---------------------------------------------------------------------------

/// A tapped right click: every thrall she has sent to the spot on the floor
/// under the crosshair.
///
/// One frame of startup, one active, ten back. There is nothing for an
/// opponent to read in a frame, so what it owes is *where*: the arm is thrown
/// straight out and down at the floor, a long way off, the way a person
/// points a dog at something -- and the thralls going there are the rest of
/// the sentence. It is the left arm, the hand `ready` holds open, and the
/// right stays in at the ribs throughout: the only cost of this move is a
/// moment of attention, so it is the only one in the file where the
/// punctuation is there from the start rather than paid at the end.
///
/// It points *down*, where the Haemorrhage thrusts level: the same arm on two
/// lines, and the angle is the difference between "you" and "there".
fn command() -> Recipe {
    let clip = Clip::BloodPoke;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let held = (recover + (end - recover) / 5).max(recover + 1);
    let paid = (recover + (end - recover) / 2).max(held + 2);
    let home = end.saturating_sub(2).max(paid + 1);

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "A one-frame startup, so the point is the contact pose: the \
                left arm straight out and down at a spot on the floor several \
                metres off, the chest turned in behind it and the head \
                following the line. It holds there for two frames -- an order \
                is given and then seen to -- and lets the arm down heavier \
                than it threw it, knees giving, before the open hand comes \
                back to `ready`. The right hand never leaves the ribs. CRISP \
                because the whole gesture has to arrive on frame one; any lag \
                and the contact frame is a body halfway to pointing."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::LINEAR),
            Key::eased(contact, pointing(), Ease::OUT),
            Key::eased(held, pointing_on(), Ease::SMOOTH),
            Key::eased(paid, lowering(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// The point: the left arm straight out and down along a line that meets the
/// floor four or five metres ahead, the hand flat and in line with it.
fn pointing() -> Pose {
    footing(
        ribs_r(
            ready()
                .hips(-0.012, -0.100, 0.034)
                .root(9.0, 0.0, 4.0)
                .spine(12.0, 0.0, 10.0)
                .chest(8.0, 0.0, 14.0)
                .head(6.0, 0.0, -8.0),
        )
        .reach_l([-0.15, 0.98, 0.78])
        .wrist_l(8.0, 0.0, 0.0),
        0.0,
        0.014,
    )
}

/// Two frames on: still pointing, the weight settled onto the front foot.
fn pointing_on() -> Pose {
    footing(
        ribs_r(
            ready()
                .hips(-0.014, -0.112, 0.040)
                .root(10.0, 0.0, 4.0)
                .spine(14.0, 0.0, 10.0)
                .chest(9.0, 0.0, 14.0)
                .head(8.0, 0.0, -8.0),
        )
        .reach_l([-0.15, 0.94, 0.78])
        .wrist_l(10.0, 0.0, 0.0),
        0.0,
        0.016,
    )
}

/// The arm let down, the knees giving under it. Heavier than the point.
fn lowering() -> Pose {
    footing(
        ribs_r(
            ready()
                .hips(0.000, -0.128, 0.006)
                .root(9.0, 0.0, -6.0)
                .spine(18.0, 0.0, 6.0)
                .chest(11.0, 0.0, 8.0)
                .head(-14.0, 0.0, 2.0),
        )
        .reach_l([-0.24, 0.92, 0.30])
        .wrist_l(-12.0, 0.0, 0.0),
        0.0,
        0.010,
    )
}

// ---------------------------------------------------------------------------
// Hemoplague
// ---------------------------------------------------------------------------

/// Left click held: a cloud on the floor where she aimed, growing for as long
/// as she holds.
///
/// **Frame zero is the hold** (see the module notes): both hands lifted out
/// in front and spread wide, palms down over the floor ahead, as though
/// holding something down that is trying to rise. A hand over a thing is
/// what the cloud is -- she is not throwing it, she is keeping it there -- and
/// the silhouette is the widest standing shape she has, two arms forward and
/// apart, which nothing else in the kit is.
///
/// The startup, which only plays if the move is ever thrown without the hold,
/// lifts the hands to the chest and presses them back down on contact; the
/// first recovery frame is the hold again, so the cut from the channel to the
/// recovery lands on the pose it left.
fn hemoplague() -> Recipe {
    let clip = Clip::BloodSpecial;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let welling = (contact * 2 / 5).max(2);
    let lifted = contact.saturating_sub(1).max(welling + 1);
    let paid = recover + (end - recover) / 3;
    let home = end.saturating_sub(3);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "Frame zero is held for as long as the button is, so it is the \
                pose that has to say Hemoplague: both hands out and wide, palms \
                down over the floor a few metres ahead, knees soft, head down \
                at the spot. On release the clip cuts to its first recovery \
                frame, which is the same pose pressed a shade lower, and the \
                recovery lets the hands come in to the ribs, the chest closing \
                over them, before `ready`. The startup is a lift to the chest \
                and a press back down, so the move thrown untapped would still \
                read as the same pair of hands holding something down."
            .into(),
        keys: vec![
            Key::eased(0, spread_over(), Ease::SMOOTH),
            Key::eased(
                welling,
                spread_over().blend(&welling_up(), 0.7),
                Ease::SMOOTH,
            ),
            Key::eased(lifted, welling_up(), Ease::LINEAR),
            Key::eased(contact, pressed_over(), Ease::STRIKE),
            Key::eased(recover, spread_over_low(), Ease::OUT),
            Key::eased(paid, spent(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// The hold: both hands forward and spread, palms down over the floor ahead.
fn spread_over() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.000, -0.112, 0.010)
                .root(6.0, 0.0, -4.0)
                .spine(14.0, 0.0, 3.0)
                .chest(6.0, 0.0, 3.0)
                .head(6.0, 0.0, 2.0),
            [-0.50, 1.00, 0.64],
            [0.44, 1.00, 0.62],
        )
        .wrists(-6.0, 0.0, 0.0),
        0.0,
        0.008,
    )
}

/// The first recovery frame: the hold, pressed a shade lower.
fn spread_over_low() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.000, -0.128, 0.012)
                .root(7.0, 0.0, -4.0)
                .spine(16.0, 0.0, 3.0)
                .chest(7.0, 0.0, 3.0)
                .head(8.0, 0.0, 2.0),
            [-0.50, 0.95, 0.64],
            [0.44, 0.95, 0.62],
        )
        .wrists(-4.0, 0.0, 0.0),
        0.0,
        0.008,
    )
}

/// The lift, at the end of the startup: hands drawn up to the chest, palms
/// still down, the body taller.
fn welling_up() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.000, -0.060, -0.010)
                .root(2.0, 0.0, -4.0)
                .spine(4.0, 0.0, 3.0)
                .chest(0.0, 0.0, 3.0)
                .head(-4.0, 0.0, 2.0),
            [-0.30, 1.28, 0.34],
            [0.26, 1.28, 0.32],
        )
        .wrists(10.0, 0.0, 0.0),
        0.0,
        0.012,
    )
}

/// Contact: pressed down and out, lower than the hold.
fn pressed_over() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.000, -0.146, 0.016)
                .root(9.0, 0.0, -4.0)
                .spine(18.0, 0.0, 3.0)
                .chest(8.0, 0.0, 3.0)
                .head(10.0, 0.0, 2.0),
            [-0.50, 0.90, 0.64],
            [0.44, 0.90, 0.62],
        )
        .wrists(-10.0, 0.0, 0.0),
        0.0,
        0.008,
    )
}

// ---------------------------------------------------------------------------
// Carpet of blood
// ---------------------------------------------------------------------------

/// Right click held: a disc of her blood spreading from where she aimed.
///
/// The Hemoplague's hands are held *over* the floor; these are *on* it. She
/// goes all the way down into a crouch with both palms pressed flat to the
/// boards in front of her feet -- paying into the floor, which is what the
/// carpet is, her red going out of her as it spreads. It is the lowest held
/// shape anyone has, and a body that low is a body that is not about to do
/// anything else, which is the honest read of a hold that costs her as she
/// keeps it.
///
/// Frame zero is the hold, as the Hemoplague's is. The startup lifts the
/// hands to the knees and slaps them back down wider; the recovery slides them
/// out along the floor -- the pool spreading under her hands -- and then
/// stands her up out of it, slowly, a hand to the ribs.
fn carpet() -> Recipe {
    let clip = Clip::BloodCarpet;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let lift = (contact * 2 / 5).max(2);
    let up = contact.saturating_sub(1).max(lift + 1);
    let slide = recover + (end - recover) / 4;
    let paid = recover + (end - recover) * 3 / 5;
    let home = end.saturating_sub(3);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "Held on frame zero, so frame zero is the move: a deep crouch, \
                both palms flat on the floor in front of the feet, the head up \
                to watch the pool. The lowest held pose in the game, and that \
                is the point -- a body this low is committed to the floor. The \
                hips sit back rather than over the toes, for the reason the \
                spike's do: a squat this deep with the weight forward asks the \
                ankles for more than they have. The startup is a lift to the \
                knees and a slap back down; the first recovery frame is the \
                press again, so the cut from the hold lands where it left. \
                Then the hands slide outward along the boards -- the carpet \
                spreading -- and the body stands up out of it, the slowest \
                part of the clip, right hand to the ribs."
            .into(),
        keys: vec![
            Key::eased(0, palms_down(), Ease::SMOOTH),
            Key::eased(lift, lifting_off(), Ease::SMOOTH),
            Key::eased(up, lifted_off(), Ease::LINEAR),
            Key::eased(contact, slapped(), Ease::STRIKE),
            Key::eased(recover, palms_down_wide(), Ease::OUT),
            Key::eased(slide, spreading(), Ease::SMOOTH),
            Key::eased(paid, stooped(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// The crouch every carpet pose is built on: hips down and back, the body
/// folded over the knees, the head up.
fn crouched(drop: f32, fold: f32) -> Pose {
    ready()
        .hips(0.004, -drop, -0.040)
        .root(10.0 + fold * 0.3, 0.0, -6.0)
        .spine(fold, 0.0, 4.0)
        .chest(fold * 0.45, 0.0, 4.0)
        .head(-fold * 1.05, 0.0, 4.0)
}

/// The hold: both palms on the floor in front of the feet.
fn palms_down() -> Pose {
    footing(
        hands(
            crouched(0.300, 38.0),
            [-0.34, 0.40, 0.58],
            [0.30, 0.40, 0.56],
        )
        .wrists(-40.0, 0.0, 0.0),
        0.0,
        0.050,
    )
}

/// The first recovery frame: the press, the hands a little wider.
fn palms_down_wide() -> Pose {
    footing(
        hands(
            crouched(0.306, 39.0),
            [-0.42, 0.39, 0.56],
            [0.38, 0.39, 0.54],
        )
        .wrists(-42.0, 0.0, 0.0),
        0.0,
        0.050,
    )
}

/// The hands sliding out along the boards, wider still, the body starting to
/// come up behind them.
fn spreading() -> Pose {
    footing(
        hands(
            crouched(0.270, 36.0),
            [-0.56, 0.42, 0.44],
            [0.52, 0.42, 0.42],
        )
        .wrists(-34.0, 0.0, 0.0),
        0.0,
        0.044,
    )
}

/// A third of the way into the startup: the hands coming up off the floor.
fn lifting_off() -> Pose {
    footing(
        hands(
            crouched(0.256, 32.0),
            [-0.32, 0.58, 0.52],
            [0.28, 0.58, 0.50],
        )
        .wrists(-10.0, 0.0, 0.0),
        0.0,
        0.044,
    )
}

/// The top of the lift, the frame before the slap: hands at the knees.
fn lifted_off() -> Pose {
    footing(
        hands(
            crouched(0.236, 26.0),
            [-0.30, 0.70, 0.46],
            [0.26, 0.70, 0.44],
        )
        .wrists(0.0, 0.0, 0.0),
        0.0,
        0.040,
    )
}

/// Contact: the palms slapped back down, wider than the hold.
fn slapped() -> Pose {
    footing(
        hands(
            crouched(0.310, 40.0),
            [-0.40, 0.38, 0.58],
            [0.36, 0.38, 0.56],
        )
        .wrists(-44.0, 0.0, 0.0),
        0.0,
        0.052,
    )
}

// ---------------------------------------------------------------------------
// Summon
// ---------------------------------------------------------------------------

/// `E`: every stain round her stands up as a thrall.
///
/// The arms do what the stains are asked to do. Down first -- a crouch with
/// both hands low and open beside the knees, palms up, as if under something
/// lying on the floor -- and then swept up together, out in front, to the
/// height of the shoulders and no higher, the body straightening and the head
/// coming up with them: *stand*. The one move in the kit where the hands rise
/// palm-up, so it cannot be mistaken for the Hemoplague's palms-down or the
/// Golem's haul, which goes the other way.
fn summon() -> Recipe {
    let clip = Clip::BloodSummon;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let sink = (contact * 2 / 5).max(2);
    let low = (contact * 2 / 3).max(sink + 1);
    let rising = contact.saturating_sub(1).max(low + 1);
    let paid = recover + (end - recover) * 2 / 5;
    let home = end.saturating_sub(3);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "Down and then up, both arms together. The sink is the tell -- \
                a heavy body bending its knees with both hands going low and \
                open is on screen by frame four -- and it bottoms out two \
                thirds of the way through the startup, so the sweep up has \
                room. The sweep is LINEAR from the bottom through contact to \
                the top, for the reason the Haemorrhage's thrust is: a shaped \
                ease would put most of a metre of hand into one frame. The \
                hands stop at the shoulders, palms up and open, because \
                nothing on the floor goes past the ear on this body, and the \
                rear heel comes up with them. Then the bill: arms down, hands \
                to the ribs, knees giving, the deepest frame after the thralls \
                are already up."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(sink, ready().blend(&beckoning_low(), 0.75), Ease::SMOOTH),
            Key::eased(low, beckoning_low(), Ease::SMOOTH),
            Key::eased(rising, lifting_up(), Ease::LINEAR),
            Key::eased(contact, raising(), Ease::LINEAR),
            Key::eased(recover, raised(), Ease::OUT),
            Key::eased(paid, spent(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// The bottom: knees bent, both hands low and open beside the knees, palms up.
fn beckoning_low() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.000, -0.200, -0.050)
                .root(10.0, 0.0, -4.0)
                .spine(22.0, 0.0, 2.0)
                .chest(10.0, 0.0, 2.0)
                .head(-14.0, 0.0, 2.0),
            [-0.42, 0.50, 0.34],
            [0.38, 0.50, 0.30],
        )
        .forearm_l(30.0, 70.0)
        .forearm_r(30.0, 70.0)
        .wrists(-30.0, 0.0, 0.0),
        0.0,
        0.020,
    )
}

/// The frame before contact: the hands coming up past the hips.
fn lifting_up() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.000, -0.160, -0.030)
                .root(7.0, 0.0, -4.0)
                .spine(14.0, 0.0, 2.0)
                .chest(4.0, 0.0, 2.0)
                .head(-12.0, 0.0, 2.0),
            [-0.44, 0.80, 0.38],
            [0.40, 0.80, 0.34],
        )
        .wrists(-30.0, 0.0, 0.0),
        0.0,
        0.016,
    )
}

/// Contact: hands at the chest, still rising, the body straightening.
fn raising() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.000, -0.110, -0.010)
                .root(3.0, 0.0, -4.0)
                .spine(4.0, 0.0, 2.0)
                .chest(-6.0, 0.0, 2.0)
                .head(-16.0, 0.0, 2.0),
            [-0.50, 1.10, 0.48],
            [0.46, 1.10, 0.44],
        )
        .wrists(-34.0, 0.0, 0.0),
        0.0,
        0.024,
    )
}

/// The first recovery frame: arms up at the shoulders, palms up, the chest
/// open and the rear heel off the floor. As high as anything on her goes.
fn raised() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.000, -0.070, 0.000)
                .root(0.0, 0.0, -4.0)
                .spine(0.0, 0.0, 2.0)
                .chest(-10.0, 0.0, 2.0)
                .head(-18.0, 0.0, 2.0),
            [-0.52, 1.42, 0.44],
            [0.48, 1.42, 0.40],
        )
        .wrists(-36.0, 0.0, 0.0),
        0.0,
        0.032,
    )
}

// ---------------------------------------------------------------------------
// Coagulate
// ---------------------------------------------------------------------------

/// `Q`: every stain in reach slides to the spot under the crosshair and merges.
///
/// Reaching out and hauling in. Both arms go out forward and wide, the body
/// leaning in after them, open hands as far as they go -- every stain in reach
/// -- and on contact they are dragged in together to a point in front of her
/// belly with the weight thrown back onto the rear foot. A haul on a net. The
/// silhouette is forward and then back, where the Summon is down and then up
/// and the Hemoplague is out and held.
fn coagulate() -> Recipe {
    let clip = Clip::BloodCoagulate;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let out = (contact * 2 / 5).max(2);
    let far = contact.saturating_sub(1).max(out + 1);
    let paid = recover + (end - recover) * 2 / 5;
    let home = end.saturating_sub(3);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "Out, and then in. The reach is SMOOTH and takes most of the \
                startup -- it is the tell, both arms going forward and apart \
                with the body leaning in over the lead foot -- and then the \
                haul is one LINEAR step to contact, the hands coming in \
                together to a point in front of the belly and the hips going \
                back over the rear foot. The hands keep closing after contact, \
                into fists against the stomach, which is the heaviest frame; \
                then the usual bill. The weight going back is what tells it \
                from the Haemorrhage's thrust, which goes forward, at a \
                distance where the arms are too small to read."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(out, reaching_out(), Ease::SMOOTH),
            Key::eased(far, reached(), Ease::LINEAR),
            Key::eased(contact, hauled(), Ease::STRIKE),
            Key::eased(recover, gathered_in(), Ease::OUT),
            Key::eased(paid, spent(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// Both arms out forward and wide, the body leaning in after them.
fn reaching_out() -> Pose {
    footing(
        hands(
            ready()
                .hips(-0.004, -0.104, 0.040)
                .root(9.0, 0.0, -4.0)
                .spine(12.0, 0.0, 3.0)
                .chest(6.0, 0.0, 3.0)
                .head(-6.0, 0.0, 2.0),
            [-0.40, 1.16, 0.70],
            [0.36, 1.16, 0.68],
        )
        .wrists(18.0, 0.0, 0.0),
        0.0,
        0.020,
    )
}

/// The frame before contact: as far as the arms go, fingers spread.
fn reached() -> Pose {
    footing(
        hands(
            ready()
                .hips(-0.006, -0.110, 0.052)
                .root(11.0, 0.0, -4.0)
                .spine(14.0, 0.0, 3.0)
                .chest(7.0, 0.0, 3.0)
                .head(-8.0, 0.0, 2.0),
            [-0.34, 1.12, 0.80],
            [0.30, 1.12, 0.78],
        )
        .wrists(24.0, 0.0, 0.0),
        0.0,
        0.024,
    )
}

/// Contact: hauled in, the hands meeting in front of the belly, the hips gone
/// back over the rear foot.
fn hauled() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.006, -0.120, -0.070)
                .root(-2.0, 0.0, -6.0)
                .spine(4.0, 0.0, 3.0)
                .chest(-8.0, 0.0, 3.0)
                .head(-2.0, 0.0, 2.0),
            [-0.12, 1.04, 0.50],
            [0.10, 1.04, 0.48],
        )
        .wrists(-20.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// The first recovery frame: fists in against the stomach, folded over them.
fn gathered_in() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.006, -0.150, -0.080)
                .root(8.0, 0.0, -6.0)
                .spine(22.0, 0.0, 3.0)
                .chest(10.0, 0.0, 3.0)
                .head(-16.0, 0.0, 2.0),
            [-0.06, 0.98, 0.34],
            [0.05, 0.98, 0.33],
        )
        .wrists(-30.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

// ---------------------------------------------------------------------------
// Blood wings
// ---------------------------------------------------------------------------

/// Space and left click: a burst along her aim, any way at all.
///
/// The arms are the wings. Folded in for the four frames of startup --
/// fists drawn in to the collarbones, the knees bent under a body about to go
/// -- and on the frame she goes they are flung back and wide behind her, the
/// chest thrown open and the legs trailing off the floor, a body pulled along
/// by its sternum. They stay spread through the flight, sweep a little
/// forward as it slows, and the clip finishes in the air with the knees
/// coming up and a hand to the ribs, ready for the jump's own fall.
fn wings() -> Recipe {
    let clip = Clip::BloodWings;
    let (last, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let fold = (contact / 2).max(2).min(last);
    let glide = recover + (end - recover) / 3;
    let home = end.saturating_sub(3).max(glide + 2);

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "Four frames of startup, so the fold is the whole tell: \
                fists drawn in to the collarbones, knees bent, head down. On \
                contact the arms are flung back and out behind her -- the \
                widest silhouette in the kit, and the only one that opens \
                backwards -- with the chest leading, the head up and the legs \
                trailing long off the floor: the body is going where the \
                sternum is going. The wings stay open through the burst and \
                sweep forward a little as it dies; the clip ends airborne, \
                knees tucking and the right hand back at the ribs, because \
                the jump's fall takes over from here and a landing is not \
                this clip's to draw. CRISP because four frames of startup is \
                no room for an arm to lag into."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(fold, folded(), Ease::SMOOTH),
            Key::eased(last, folded_tight(), Ease::LINEAR),
            Key::eased(contact, flung_wide(), Ease::OUT),
            Key::eased(recover, spread_wide(), Ease::OUT),
            Key::eased(glide, gliding(), Ease::SMOOTH),
            Key::eased(home, landing_wings(), Ease::SMOOTH),
        ],
    }
}

/// The fold: fists drawn in to the collarbones, knees bent.
fn folded() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.000, -0.170, -0.010)
                .root(10.0, 0.0, -4.0)
                .spine(16.0, 0.0, 2.0)
                .chest(8.0, 0.0, 2.0)
                .head(-10.0, 0.0, 2.0),
            [-0.10, 1.24, 0.24],
            [0.08, 1.22, 0.24],
        )
        .wrists(-20.0, 0.0, 0.0),
        0.0,
        0.020,
    )
}

/// The last frame on the floor: tighter, lower, the heels coming up.
fn folded_tight() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.000, -0.160, 0.000)
                .root(14.0, 0.0, -4.0)
                .spine(18.0, 0.0, 2.0)
                .chest(10.0, 0.0, 2.0)
                .head(-16.0, 0.0, 2.0),
            [-0.10, 1.20, 0.22],
            [0.08, 1.18, 0.22],
        )
        .wrists(-24.0, 0.0, 0.0),
        // The rear foot already leaving: the push is from the front one.
        0.030,
        0.100,
    )
}

/// Contact: off the floor, the arms flung back and wide, chest leading.
///
/// The whole body pitched forward at the hips, so the legs trail out behind
/// it rather than hanging under it: a body that is going somewhere, not one
/// that has jumped.
fn flung_wide() -> Pose {
    trailing(
        ready()
            .root(14.0, 0.0, -4.0)
            .spine(0.0, 0.0, 2.0)
            .chest(-4.0, 0.0, 2.0)
            .head(-26.0, 0.0, 2.0)
            .shoulders(-56.0, 46.0, 0.0)
            .elbows(20.0)
            .wrists(10.0, 0.0, 0.0),
        0.0,
    )
}

/// The first recovery frame: the wings at their widest and furthest back.
fn spread_wide() -> Pose {
    trailing(
        ready()
            .root(20.0, 0.0, -4.0)
            .spine(-2.0, 0.0, 2.0)
            .chest(-6.0, 0.0, 2.0)
            .head(-30.0, 0.0, 2.0)
            .shoulders(-64.0, 56.0, 0.0)
            .elbows(14.0)
            .wrists(14.0, 0.0, 0.0),
        1.0,
    )
}

/// The legs of a body flying forward. `k` at 0 is the frame they leave the
/// floor, still bent under it; at 1 they are straight-ish and out behind, toes
/// pointed.
fn trailing(pose: Pose, k: f32) -> Pose {
    pose.hips(0.0, -0.12 + 0.14 * k, 0.0)
        .hip_l(30.0 - 22.0 * k, 7.0, 0.0)
        .knee_l(62.0 - 22.0 * k)
        .hip_r(18.0 - 14.0 * k, 6.0, 0.0)
        .knee_r(84.0 - 24.0 * k)
        .ankles(14.0 + 26.0 * k, 0.0, 0.0)
}

/// The burst dying: the body coming upright, the wings sweeping round to the
/// sides, knees starting up.
fn gliding() -> Pose {
    aloft(
        ready()
            .root(10.0, 0.0, -4.0)
            .spine(0.0, 0.0, 2.0)
            .chest(-6.0, 0.0, 2.0)
            .head(-14.0, 0.0, 2.0)
            .shoulders(-20.0, 70.0, 0.0)
            .elbows(30.0)
            .wrists(0.0, 0.0, 0.0),
        0.4,
    )
}

/// The end, still in the air: knees up, the right hand at the ribs, the left
/// arm still half out for balance.
fn landing_wings() -> Pose {
    ribs_r(aloft(
        ready()
            .root(4.0, 0.0, -6.0)
            .spine(10.0, 0.0, 4.0)
            .chest(4.0, 0.0, 4.0)
            .head(-10.0, 0.0, 4.0)
            .shoulder_l(20.0, 44.0, -10.0)
            .forearm_l(60.0, 20.0)
            .wrist_l(-10.0, 0.0, 0.0),
        0.8,
    ))
}

// ---------------------------------------------------------------------------
// Upheaval
// ---------------------------------------------------------------------------

/// Space and middle click: a geyser that launches whoever is over it, and a
/// hop for her.
///
/// One hand goes down to the floor beside the lead foot and takes hold of
/// something, and on contact rips it up -- past the hip, past the face, the
/// arm finishing straight up beside the head -- and the rest of the body
/// follows it off the floor. The right hand, as the spike's is: the two moves
/// that come up out of the ground start from the same side, and the spike's
/// hand goes up before it goes down where this one goes down before it goes
/// up. The hop is the sim's; the clip extends the legs into it and tucks them
/// after, so the jump's fall can take over.
fn upheaval() -> Recipe {
    let clip = Clip::BloodUpheaval;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let reach = (contact / 4).max(2);
    let grip = (contact * 5 / 8).max(reach + 2);
    let tear = (contact * 13 / 16).max(grip + 1).min(contact - 1);
    let thrown = (contact + 2).min(recover);
    let held = recover.max(thrown + 1);
    let tuck = (recover + (end - recover) / 2).max(held + 2);
    let home = end.saturating_sub(3).max(tuck + 2);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "Down first, then up. The reach for the floor is on screen by \
                frame four -- a heavy body bending to pick something up, the \
                free hand bracing on the lead thigh -- and it grips low and \
                holds there for most of the sixteen frames, which is the \
                telegraph. The rip is LINEAR over the last three frames of the \
                startup and the first two of the active ones, the hand coming \
                up past the face to straight overhead-but-not-past-the-ear, \
                beside the head, and the legs extending through it with the \
                heels coming up so the hop has somewhere to come from. Then it \
                is in the air: knees tucking, the arm coming down and the hand \
                to the ribs. The heaviest frame of this one is the grip, not \
                the recovery -- the hop carries the recovery off the floor, \
                and a body in the air cannot sag."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(reach, reaching_down(), Ease::SMOOTH),
            Key::eased(grip, gripped_low(), Ease::SMOOTH),
            Key::eased(tear, tearing(), Ease::LINEAR),
            Key::eased(contact, ripped(), Ease::LINEAR),
            Key::eased(thrown, ripped_up(), Ease::OUT),
            // Held up for the frames the geyser is out: the arms run a couple
            // of frames behind their keys, and an arm let down the frame it
            // arrives never arrives.
            Key::eased(held, ripped_up(), Ease::SMOOTH),
            Key::eased(tuck, hopping(), Ease::SMOOTH),
            Key::eased(home, hopped(), Ease::SMOOTH),
        ],
    }
}

/// Bending for the floor, the right hand going down beside the lead foot.
fn reaching_down() -> Pose {
    footing(
        ready()
            .hips(0.006, -0.180, -0.040)
            .root(12.0, 0.0, -8.0)
            .spine(24.0, 0.0, 6.0)
            .chest(10.0, 0.0, 6.0)
            .head(-22.0, 0.0, 4.0)
            .reach_r([0.20, 0.52, 0.32])
            .wrist_r(-20.0, 0.0, 0.0)
            .shoulder_l(30.0, 18.0, -34.0)
            .forearm_l(84.0, 26.0)
            .wrist_l(-16.0, 0.0, 0.0),
        0.0,
        0.030,
    )
}

/// Gripped, low: the hand at the shin, fist closed, the free hand braced on
/// the lead thigh. Held for most of the startup.
fn gripped_low() -> Pose {
    footing(
        ready()
            .hips(0.008, -0.260, -0.040)
            .root(13.0, 0.0, -8.0)
            .spine(30.0, 0.0, 6.0)
            .chest(14.0, 0.0, 6.0)
            .head(-34.0, 0.0, 4.0)
            .reach_r([0.20, 0.38, 0.34])
            .wrist_r(-34.0, 0.0, 0.0)
            .shoulder_l(30.0, 18.0, -34.0)
            .forearm_l(86.0, 26.0)
            .wrist_l(-16.0, 0.0, 0.0),
        0.0,
        0.044,
    )
}

/// Tearing: the fist coming up past the knee, the body starting to rise.
fn tearing() -> Pose {
    footing(
        ready()
            .hips(0.006, -0.230, -0.030)
            .root(12.0, 0.0, -8.0)
            .spine(26.0, 0.0, 6.0)
            .chest(10.0, 0.0, 6.0)
            .head(-28.0, 0.0, 4.0)
            .reach_r([0.20, 0.58, 0.32])
            .wrist_r(-30.0, 0.0, 0.0)
            .shoulder_l(26.0, 18.0, -30.0)
            .forearm_l(88.0, 24.0)
            .wrist_l(-16.0, 0.0, 0.0),
        0.0,
        0.040,
    )
}

/// Contact: the fist at the chest and still going, the body unfolded, both
/// heels coming up.
fn ripped() -> Pose {
    footing(
        ready()
            .hips(0.000, -0.080, 0.000)
            .root(4.0, 0.0, -8.0)
            .spine(6.0, 0.0, 6.0)
            .chest(-2.0, 0.0, 6.0)
            .head(-16.0, 0.0, 4.0)
            .reach_r([0.20, 1.30, 0.30])
            .wrist_r(-20.0, 0.0, 0.0)
            .shoulder_l(10.0, 24.0, -20.0)
            .forearm_l(80.0, 20.0)
            .wrist_l(-16.0, 0.0, 0.0),
        0.030,
        0.040,
    )
}

/// Two frames on, off the floor: the arm straight up beside the head, the
/// legs extended and pointed under her.
fn ripped_up() -> Pose {
    aloft(
        ready()
            .root(-2.0, 0.0, -8.0)
            .spine(-4.0, 0.0, 6.0)
            .chest(-6.0, 0.0, 6.0)
            .head(-18.0, 0.0, 4.0)
            .reach_r([0.22, 1.84, 0.12])
            .wrist_r(-10.0, 0.0, 0.0)
            .shoulder_l(-6.0, 30.0, -14.0)
            .forearm_l(60.0, 16.0)
            .wrist_l(-10.0, 0.0, 0.0),
        0.0,
    )
}

/// The hop: the knees coming up, the arm coming down.
fn hopping() -> Pose {
    aloft(
        ready()
            .root(4.0, 0.0, -8.0)
            .spine(6.0, 0.0, 6.0)
            .chest(2.0, 0.0, 6.0)
            .head(-8.0, 0.0, 4.0)
            .reach_r([0.26, 1.30, 0.20])
            .wrist_r(-10.0, 0.0, 0.0)
            .shoulder_l(-8.0, 22.0, -20.0)
            .forearm_l(90.0, 14.0)
            .wrist_l(-14.0, 0.0, 0.0),
        0.6,
    )
}

/// The end, at the top of the hop: knees up, both hands in, the right at the
/// ribs.
fn hopped() -> Pose {
    ribs_l(ribs_r(aloft(
        ready()
            .root(8.0, 0.0, -8.0)
            .spine(14.0, 0.0, 6.0)
            .chest(8.0, 0.0, 6.0)
            .head(-12.0, 0.0, 4.0),
        0.9,
    )))
}

// ---------------------------------------------------------------------------
// Stake
// ---------------------------------------------------------------------------

/// Middle click in the air: a spear of blood thrown down at the crosshair,
/// pinning anybody it meets in the air.
///
/// Both hands come up together over the head, as though a haft had been put in
/// them, the back arches and the shins fold up behind -- the whole body a bow
/// drawn -- and on contact it is hurled forward and down with both arms, the
/// body jack-knifing after it, knees coming up to meet the chest. The only
/// overhead in the kit off a finisher, and earned: it is the one throw she
/// makes from above, and a spear thrown from the air is thrown from over the
/// head or not at all.
fn stake() -> Recipe {
    let clip = Clip::BloodStake;
    let (last, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let up = (contact * 2 / 5).max(2);
    let cocked = last.saturating_sub(1).max(up + 2);
    let through = recover;
    let curl = recover + (end - recover) * 2 / 5;
    let home = end.saturating_sub(3);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "In the air from first frame to last. Both hands climb together \
                over the head and the shins fold up behind, so by frame four \
                the silhouette is a bow drawn: that is the read, and it is \
                unlike anything she does on the floor, where nothing goes past \
                the ear. The throw is LINEAR over the two frames into contact \
                and the three after it, the arms coming over the top and down \
                through the line to the crosshair, and the body jack-knifes \
                behind them -- spine and chest folding, knees coming up -- so \
                the heaviest, most closed frame is the first recovery frame, \
                with the hands already below the hips. The recovery curls in, \
                hands to the ribs, and finishes in the jump's own loose \
                in-flight shape."
            .into(),
        keys: vec![
            Key::eased(0, in_the_air(), Ease::OUT),
            Key::eased(up, raising_the_stake(), Ease::SMOOTH),
            Key::eased(cocked, stake_overhead(), Ease::LINEAR),
            Key::eased(contact, stake_thrown(), Ease::LINEAR),
            Key::eased(through, stake_through(), Ease::OUT),
            Key::eased(curl, curled(), Ease::SMOOTH),
            Key::eased(home, in_the_air(), Ease::SMOOTH),
        ],
    }
}

/// Where she is between things in the air: `ready`'s arms over loose legs.
fn in_the_air() -> Pose {
    aloft(ready().hips(0.0, 0.0, 0.0), 0.5)
}

/// Both hands coming up together in front of the face.
fn raising_the_stake() -> Pose {
    hands(
        aloft(
            ready()
                .root(-2.0, 0.0, -4.0)
                .spine(-4.0, 0.0, 2.0)
                .chest(-4.0, 0.0, 2.0)
                .head(6.0, 0.0, 2.0),
            0.4,
        ),
        [-0.06, 1.62, 0.26],
        [0.06, 1.62, 0.26],
    )
    .wrists(-20.0, 0.0, 0.0)
}

/// The top: both hands over the head and a little behind it, the back arched,
/// the shins folded up -- a bow drawn.
fn stake_overhead() -> Pose {
    hands(
        ready()
            .hips(0.0, 0.02, 0.0)
            .root(-10.0, 0.0, -4.0)
            .spine(-14.0, 0.0, 2.0)
            .chest(-10.0, 0.0, 2.0)
            .head(14.0, 0.0, 2.0)
            .hip_l(-16.0, 7.0, 0.0)
            .knee_l(96.0)
            .hip_r(-22.0, 6.0, 0.0)
            .knee_r(104.0)
            .ankles(30.0, 0.0, 0.0),
        [-0.06, 1.86, -0.04],
        [0.06, 1.86, -0.04],
    )
    .wrists(-30.0, 0.0, 0.0)
}

/// Contact: the arms come over and are on the way down, out in front at the
/// height of the face; the jack-knife has started.
fn stake_thrown() -> Pose {
    hands(
        aloft(
            ready()
                .root(4.0, 0.0, -4.0)
                .spine(10.0, 0.0, 2.0)
                .chest(6.0, 0.0, 2.0)
                .head(16.0, 0.0, 2.0),
            0.3,
        ),
        [-0.06, 1.50, 0.40],
        [0.06, 1.50, 0.40],
    )
    .wrists(10.0, 0.0, 0.0)
}

/// The first recovery frame: thrown through, the hands down past the hips in
/// front of her, the body folded over them and the knees up.
fn stake_through() -> Pose {
    hands(
        ready()
            .hips(0.0, -0.02, 0.0)
            .root(18.0, 0.0, -4.0)
            .spine(32.0, 0.0, 2.0)
            .chest(16.0, 0.0, 2.0)
            .head(-4.0, 0.0, 2.0)
            .hip_l(58.0, 8.0, 0.0)
            .knee_l(76.0)
            .hip_r(44.0, 7.0, 0.0)
            .knee_r(84.0)
            .ankles(24.0, 0.0, 0.0),
        [-0.08, 0.72, 0.50],
        [0.08, 0.72, 0.50],
    )
    .wrists(20.0, 0.0, 0.0)
}

/// Curled: both hands to the ribs, the body still closed over them.
fn curled() -> Pose {
    ribs_l(ribs_r(aloft(
        ready()
            .root(10.0, 0.0, -6.0)
            .spine(22.0, 0.0, 4.0)
            .chest(12.0, 0.0, 4.0)
            .head(-12.0, 0.0, 4.0),
        0.8,
    )))
}

// ---------------------------------------------------------------------------
// Black vines
// ---------------------------------------------------------------------------

/// The Hemoplague held to its end: barbed vines erupting outward from the
/// cloud's middle.
///
/// It starts where the hold left her, hands spread over the floor, and gathers
/// them: two fists brought together and lifted to the chin, the body rising
/// onto the rear toe -- and then driven down together into the floor in front
/// of her feet, and **wrenched apart**, each fist dragged outward along the
/// boards as though tearing the floor open between them. The spike drives one
/// hand down; this drives two and then pulls them away from each other, which
/// is the shape of the vines going out from the middle.
fn vines() -> Recipe {
    let clip = Clip::BloodVines;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let fists = (contact / 5).max(2);
    let top = (contact * 7 / 15).max(fists + 2);
    let hang = (contact * 2 / 3).max(top + 1);
    let falling = (contact * 4 / 5).max(hang + 1);
    let torn = recover + (end - recover) / 5;
    let wide = recover + (end - recover) * 2 / 5;
    let paid = recover + (end - recover) * 2 / 3;
    let home = end.saturating_sub(3);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "A finisher, so thirty frames up and thirty back, and the shape \
                is the spike's -- up, a hang, down -- done with two fists \
                together instead of one hand, which is what makes it read as \
                the bigger thing. Frame zero is the Hemoplague's hold, since \
                that is the only place this move comes from. The fists meet \
                and rise to the chin, not past it; the hang is HOLD, the fall \
                LINEAR over six frames into the floor with the hips dropping \
                under it. Then the part only this move has: the fists, still \
                on the floor, are wrenched apart and outward, the elbows \
                flaring and the head coming up to watch the vines go -- the \
                widest low shape in the file. The recovery stands up out of \
                that slowly and pays the usual bill."
            .into(),
        keys: vec![
            Key::eased(0, spread_over(), Ease::SMOOTH),
            Key::eased(fists, fists_together(), Ease::SMOOTH),
            Key::eased(top, fists_raised(), Ease::HOLD),
            Key::eased(hang, fists_hung(), Ease::LINEAR),
            Key::eased(falling, fists_falling(), Ease::LINEAR),
            Key::eased(contact, fists_driven(), Ease::STRIKE),
            Key::eased(recover, fists_planted(), Ease::OUT),
            Key::eased(torn, wrenching(), Ease::SMOOTH),
            Key::eased(wide, wrenched(), Ease::SMOOTH),
            Key::eased(paid, stooped(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// The fists come together in front of the chest.
fn fists_together() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.000, -0.090, -0.010)
                .root(4.0, 0.0, -6.0)
                .spine(8.0, 0.0, 3.0)
                .chest(4.0, 0.0, 3.0)
                .head(-4.0, 0.0, 2.0),
            [-0.07, 1.18, 0.32],
            [0.07, 1.18, 0.32],
        )
        .wrists(-30.0, 0.0, 0.0),
        0.0,
        0.012,
    )
}

/// The top: both fists together at the chin, the back straight, the rear heel
/// up.
fn fists_raised() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.010, -0.044, -0.040)
                .root(-4.0, 0.0, -6.0)
                .spine(-6.0, 0.0, 3.0)
                .chest(-4.0, 0.0, 3.0)
                .head(6.0, 0.0, 2.0),
            [-0.06, 1.48, 0.26],
            [0.06, 1.48, 0.26],
        )
        .wrists(-34.0, 0.0, 0.0),
        0.0,
        0.026,
    )
}

/// The hang: a shade higher and further back.
fn fists_hung() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.012, -0.036, -0.052)
                .root(-7.0, 0.0, -6.0)
                .spine(-10.0, 0.0, 3.0)
                .chest(-6.0, 0.0, 3.0)
                .head(8.0, 0.0, 2.0),
            [-0.06, 1.52, 0.22],
            [0.06, 1.52, 0.22],
        )
        .wrists(-36.0, 0.0, 0.0),
        0.010,
        0.034,
    )
}

/// Half way down, keyed for the reason the spike's descent is.
fn fists_falling() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.010, -0.160, -0.044)
                .root(6.0, 0.0, -6.0)
                .spine(14.0, 0.0, 3.0)
                .chest(6.0, 0.0, 3.0)
                .head(-10.0, 0.0, 2.0),
            [-0.06, 0.96, 0.46],
            [0.06, 0.96, 0.46],
        )
        .wrists(-30.0, 0.0, 0.0),
        0.0,
        0.034,
    )
}

/// Contact: both fists driven into the floor in front of the feet.
fn fists_driven() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.008, -0.280, -0.036)
                .root(12.0, 0.0, -6.0)
                .spine(31.0, 0.0, 3.0)
                .chest(15.0, 0.0, 3.0)
                .head(-36.0, 0.0, 2.0),
            [-0.08, 0.42, 0.58],
            [0.08, 0.42, 0.58],
        )
        .wrists(-36.0, 0.0, 0.0),
        0.0,
        0.054,
    )
}

/// The first recovery frame, the lowest: arms locked out into the floor.
fn fists_planted() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.008, -0.306, -0.032)
                .root(14.0, 0.0, -6.0)
                .spine(33.0, 0.0, 3.0)
                .chest(15.0, 0.0, 3.0)
                .head(-38.0, 0.0, 2.0),
            [-0.09, 0.38, 0.58],
            [0.09, 0.38, 0.58],
        )
        .wrists(-38.0, 0.0, 0.0),
        0.0,
        0.056,
    )
}

/// Wrenching: the fists dragged outward along the floor, the elbows flaring.
fn wrenching() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.008, -0.296, -0.036)
                .root(13.0, 0.0, -6.0)
                .spine(31.0, 0.0, 3.0)
                .chest(12.0, 0.0, 3.0)
                .head(-38.0, 0.0, 2.0),
            [-0.44, 0.40, 0.50],
            [0.42, 0.40, 0.48],
        )
        .wrists(-30.0, 0.0, 0.0),
        0.0,
        0.054,
    )
}

/// Wrenched apart: the fists out wide and coming up off the floor, the chest
/// opening, the head up to watch the vines go.
fn wrenched() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.006, -0.250, -0.040)
                .root(12.0, 0.0, -6.0)
                .spine(26.0, 0.0, 3.0)
                .chest(6.0, 0.0, 3.0)
                .head(-36.0, 0.0, 2.0),
            [-0.60, 0.56, 0.18],
            [0.56, 0.56, 0.16],
        )
        .wrists(-20.0, 0.0, 0.0),
        0.0,
        0.046,
    )
}

// ---------------------------------------------------------------------------
// Crimson cascade
// ---------------------------------------------------------------------------

/// The Black spike held to its end: rings of eruptions rolling out from the
/// crosshair, wave after wave.
///
/// The spike's gesture with both hands on it: the hands clasped together and
/// lifted to beside the right ear in a reversed grip -- a stake held point
/// down -- then driven into the floor in front of her, and the body **thrown
/// back** off it, arms flung up and wide, chest open, head back, as the
/// eruptions go out. It comes out of the spike's hold, which holds `ready`,
/// so it begins there.
fn cascade() -> Recipe {
    let clip = Clip::BloodCascade;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let clasp = (contact / 6).max(2);
    let top = (contact * 2 / 5).max(clasp + 2);
    let hang = (contact * 2 / 3).max(top + 1);
    let falling = (contact * 4 / 5).max(hang + 1);
    let tearing = recover + (end - recover) / 10;
    let thrown = (recover + (end - recover) / 5).max(tearing + 2);
    let reeling = recover + (end - recover) * 2 / 5;
    let paid = recover + (end - recover) * 2 / 3;
    let home = end.saturating_sub(3);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "The spike, doubled and then answered. Both hands clasp at the \
                chest by frame five and climb to beside the right ear in a \
                reversed grip -- the spike's tell with two hands on it, so it \
                reads as the spike's bigger version -- hold, and fall LINEAR \
                into the floor in front of the lead foot. The arms lock out on \
                the first recovery frame, and then the body is thrown back off \
                the floor over five frames: hands torn up and flung back \
                beside the shoulders, the spine arching, the head back, the \
                weight onto the rear foot. That recoil is the shape no other \
                move has, and it is spent over five frames because the chest \
                travels most of half a metre in it. The reel settles forward \
                into the stoop and the bill."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(clasp, clasped(), Ease::SMOOTH),
            Key::eased(top, clasped_high(), Ease::HOLD),
            Key::eased(hang, clasped_hung(), Ease::LINEAR),
            Key::eased(falling, clasped_falling(), Ease::LINEAR),
            Key::eased(contact, clasped_driven(), Ease::STRIKE),
            Key::eased(recover, clasped_planted(), Ease::SMOOTH),
            // Torn up off the floor first, then thrown: one key straight from
            // the floor to behind the shoulders takes the hands the short way
            // through the chest.
            Key::eased(tearing, torn_up(), Ease::LINEAR),
            Key::eased(thrown, thrown_back(), Ease::OUT),
            Key::eased(reeling, reeled(), Ease::SMOOTH),
            Key::eased(paid, stooped(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// Both hands clasped together at the chest.
fn clasped() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.006, -0.080, -0.016)
                .root(3.0, 0.0, -12.0)
                .spine(6.0, 0.0, 4.0)
                .chest(4.0, 0.0, 8.0)
                .head(-4.0, 0.0, 4.0),
            [0.00, 1.20, 0.28],
            [0.06, 1.20, 0.28],
        )
        .wrists(-30.0, 0.0, 0.0),
        0.0,
        0.008,
    )
}

/// The top: the clasped hands beside the right ear, reversed, the back arched.
fn clasped_high() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.018, -0.044, -0.052)
                .root(-5.0, 0.0, -14.0)
                .spine(-9.0, 0.0, 2.0)
                .chest(-5.0, 0.0, 10.0)
                .head(8.0, 0.0, 4.0),
            [0.12, 1.52, 0.08],
            [0.18, 1.52, 0.06],
        )
        .wrists(-40.0, 0.0, 0.0),
        0.0,
        0.026,
    )
}

/// The hang: a shade higher and further back, both heels light.
fn clasped_hung() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.022, -0.036, -0.062)
                .root(-8.0, 0.0, -14.0)
                .spine(-14.0, 0.0, 2.0)
                .chest(-8.0, 0.0, 11.0)
                .head(12.0, 0.0, 4.0),
            [0.12, 1.55, 0.04],
            [0.18, 1.55, 0.02],
        )
        .wrists(-42.0, 0.0, 0.0),
        0.010,
        0.034,
    )
}

/// Half way down.
fn clasped_falling() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.014, -0.150, -0.048)
                .root(4.0, 0.0, -11.0)
                .spine(12.0, 0.0, 6.0)
                .chest(6.0, 0.0, 10.0)
                .head(-8.0, 0.0, 5.0),
            [0.08, 0.96, 0.44],
            [0.14, 0.96, 0.44],
        )
        .wrists(-36.0, 0.0, 0.0),
        0.0,
        0.032,
    )
}

/// Contact: both hands driven into the floor in front of the lead foot.
fn clasped_driven() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.010, -0.272, -0.036)
                .root(12.0, 0.0, -9.0)
                .spine(30.0, 0.0, 6.0)
                .chest(15.0, 0.0, 8.0)
                .head(-40.0, 0.0, 6.0),
            [0.04, 0.42, 0.56],
            [0.12, 0.42, 0.56],
        )
        .wrists(-44.0, 0.0, 0.0),
        0.0,
        0.056,
    )
}

/// The first recovery frame: locked out into the floor, the lowest pose.
fn clasped_planted() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.008, -0.306, -0.030)
                .root(14.0, 0.0, -9.0)
                .spine(33.0, 0.0, 6.0)
                .chest(15.0, 0.0, 8.0)
                .head(-40.0, 0.0, 6.0),
            [0.04, 0.38, 0.56],
            [0.12, 0.38, 0.56],
        )
        .wrists(-48.0, 0.0, 0.0),
        0.0,
        0.056,
    )
}

/// Torn up: the body unfolding, the hands off the floor and coming up in
/// front of the chest, opening apart.
fn torn_up() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.008, -0.220, -0.050)
                .root(8.0, 0.0, -8.0)
                .spine(14.0, 0.0, 4.0)
                .chest(2.0, 0.0, 4.0)
                .head(-24.0, 0.0, 4.0),
            [-0.24, 0.98, 0.42],
            [0.24, 0.98, 0.40],
        )
        .wrists(0.0, 0.0, 0.0),
        0.0,
        0.030,
    )
}

/// Thrown back: hands torn up and flung back beside the shoulders, spine
/// arched, head back, the weight on the rear foot.
fn thrown_back() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.006, -0.130, -0.090)
                .root(-6.0, 0.0, -8.0)
                .spine(-12.0, 0.0, 4.0)
                .chest(-10.0, 0.0, 4.0)
                .head(-22.0, 0.0, 4.0),
            [-0.44, 1.36, 0.08],
            [0.42, 1.36, 0.06],
        )
        .wrists(20.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// Reeling: coming forward off the throw, the arms dropping.
fn reeled() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.006, -0.140, -0.060)
                .root(4.0, 0.0, -8.0)
                .spine(8.0, 0.0, 4.0)
                .chest(0.0, 0.0, 4.0)
                .head(-10.0, 0.0, 4.0),
            [-0.44, 1.04, 0.06],
            [0.42, 1.04, 0.04],
        )
        .wrists(0.0, 0.0, 0.0),
        0.0,
        0.006,
    )
}

// ---------------------------------------------------------------------------
// Blood golem
// ---------------------------------------------------------------------------

/// The Carpet held to its end: everything the carpet covers, and all her grey,
/// fused into one thrall.
///
/// A haul up out of the floor. She comes up out of the carpet's crouch with
/// both arms going high -- higher than anything else she does on the floor,
/// because this is the most she has -- takes hold of something over her head,
/// and drags it down, both fists coming down to her belly while the body bows
/// over them, the knees giving, the head going down. The opposite of the
/// Summon's sweep: that brings the arms up to stand a thing up; this brings
/// them down, the body under it, because what stands up here is heavier than
/// she is.
fn golem() -> Recipe {
    let clip = Clip::BloodGolem;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let rise = (contact / 4).max(2);
    let high = (contact / 2).max(rise + 2);
    let grip = (contact * 4 / 5).max(high + 1);
    let bowed = recover + (end - recover) / 3;
    let paid = recover + (end - recover) * 2 / 3;
    let home = end.saturating_sub(3);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "Starts in the Carpet's crouch, palms on the floor, because the \
                golem only comes out of a carpet held to its end. The rise is \
                slow -- up out of the crouch, hands lifting palm-up -- and the \
                arms keep going until they are over the head and wide, the one \
                floor pose in the file with the hands above the ear, kept for \
                the finisher so it reads as the biggest thing she does. The \
                hands close at the top and hang there, then the drag comes \
                down LINEAR through contact: fists to the face, to the chest, \
                to the belly, the spine bowing over them and the knees giving, \
                so the deepest frame is a third of the way into the thirty of \
                recovery. Then she straightens and pays the bill."
            .into(),
        keys: vec![
            Key::eased(0, palms_down(), Ease::SMOOTH),
            Key::eased(rise, rising_from_carpet(), Ease::SMOOTH),
            Key::eased(high, arms_high(), Ease::SMOOTH),
            Key::eased(grip, gripped_high(), Ease::LINEAR),
            Key::eased(contact, dragging(), Ease::LINEAR),
            Key::eased(recover, dragged(), Ease::OUT),
            Key::eased(bowed, bowed_over(), Ease::SMOOTH),
            Key::eased(paid, stooped(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// Coming up out of the crouch, the hands lifting palm-up off the floor.
fn rising_from_carpet() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.004, -0.190, -0.040)
                .root(9.0, 0.0, -6.0)
                .spine(18.0, 0.0, 4.0)
                .chest(6.0, 0.0, 4.0)
                .head(-16.0, 0.0, 4.0),
            [-0.34, 0.80, 0.34],
            [0.30, 0.80, 0.32],
        )
        .wrists(-30.0, 0.0, 0.0),
        0.0,
        0.030,
    )
}

/// Arms high and wide over the head, the body tall, the chest open.
fn arms_high() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.006, -0.040, -0.030)
                .root(-3.0, 0.0, -6.0)
                .spine(-6.0, 0.0, 4.0)
                .chest(-8.0, 0.0, 4.0)
                .head(-16.0, 0.0, 4.0),
            [-0.34, 1.84, 0.16],
            [0.30, 1.84, 0.14],
        )
        .wrists(-10.0, 0.0, 0.0),
        0.0,
        0.024,
    )
}

/// The top, the frame the hands close: a shade higher, up on the rear toe.
fn gripped_high() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.008, -0.030, -0.040)
                .root(-5.0, 0.0, -6.0)
                .spine(-9.0, 0.0, 4.0)
                .chest(-9.0, 0.0, 4.0)
                .head(-18.0, 0.0, 4.0),
            [-0.30, 1.90, 0.12],
            [0.26, 1.90, 0.10],
        )
        .wrists(-34.0, 0.0, 0.0),
        0.008,
        0.034,
    )
}

/// Contact: dragging down, the fists at the face, the body starting to bow.
fn dragging() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.006, -0.090, -0.030)
                .root(4.0, 0.0, -6.0)
                .spine(8.0, 0.0, 4.0)
                .chest(2.0, 0.0, 4.0)
                .head(-6.0, 0.0, 4.0),
            [-0.24, 1.50, 0.30],
            [0.20, 1.50, 0.28],
        )
        .wrists(-34.0, 0.0, 0.0),
        0.0,
        0.020,
    )
}

/// The first recovery frame: fists at the chest, bowing.
fn dragged() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.004, -0.170, -0.040)
                .root(9.0, 0.0, -6.0)
                .spine(22.0, 0.0, 4.0)
                .chest(12.0, 0.0, 4.0)
                .head(8.0, 0.0, 4.0),
            [-0.18, 1.10, 0.34],
            [0.14, 1.10, 0.32],
        )
        .wrists(-34.0, 0.0, 0.0),
        0.0,
        0.012,
    )
}

/// Bowed over: fists at the belly, the spine rounded, knees given, head down.
/// The deepest frame, ten frames after the golem is up.
fn bowed_over() -> Pose {
    footing(
        hands(
            ready()
                .hips(0.004, -0.240, -0.060)
                .root(14.0, 0.0, -6.0)
                .spine(34.0, 0.0, 4.0)
                .chest(18.0, 0.0, 4.0)
                .head(14.0, 0.0, 4.0),
            [-0.10, 0.84, 0.30],
            [0.08, 0.84, 0.28],
        )
        .wrists(-30.0, 0.0, 0.0),
        0.0,
        0.020,
    )
}
