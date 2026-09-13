//! Bolt, Fissure, the fire pillar, and Cataclysm.
//!
//! The Elementalist does not fight, she *authors terrain*: she raises things
//! out of the floor and then detonates them. Nothing in her vocabulary is a
//! swing. The hands and forearms do the talking, the feet are a base rather
//! than a step, and the torso turns to aim instead of to swing. She is also not
//! frail -- tall and narrow with a heavy head, and three of her four moves put
//! her whole weight behind a gesture.
//!
//! ## The three read as three directions
//!
//! The one thing a player has to get right is which of these is coming, and
//! they are told by the first three frames:
//!
//! ```text
//!   Bolt      one hand back past the ear, feet unchanged      -- sideways
//!   Fissure   both hands sweep up, the body grows taller      -- up, then down
//!   Pillar    both hands sweep down, the body sinks           -- down, then up
//! ```
//!
//! Cataclysm is not a fourth direction in that read: it is on its own button,
//! so there is nothing to disambiguate it from. What it needs instead is the
//! one silhouette none of the three above ever makes -- **forward**. Fissure
//! and the pillar both keep their hands behind the line of her body the whole
//! way through; Bolt reaches out but only from the elbow. Cataclysm is the
//! first time both hands leave together and finish in front of her, because it
//! is the first move that is thrown rather than planted or lifted -- see
//! `heavy`.
//!
//! Fissure and the pillar end in opposite places too -- hands in the floor
//! against hands over the head -- but the opening frames are what matters,
//! because that is the half of the move an opponent still has a decision in.
//! Deliberately making them mirror images of each other is the cheapest
//! legibility there is: nothing else in the file had to change to buy it.
//!
//! ## Frames come from the move table
//!
//! Every key frame here is derived from `clip.phases()`, so retuning a startup
//! in the Oven moves the animation with it. Two boundaries are used directly
//! rather than scaled:
//!
//! - the **contact pose sits on the first active frame**, which is the frame
//!   the hitbox appears and the frame the fissure or the pillar erupts a reach
//!   in front of her;
//! - the **follow-through sits on the first recovery frame**, so the active
//!   window is the motion between the two rather than a held pose. The hands
//!   are still driving into the floor while the fissure is live, and the column
//!   is still coming up while she is still pulling on it.
//!
//! ## Looseness goes up with commitment
//!
//! `CRISP`, then `MARTIAL`, then `HEAVY`, in that order. That is not variety
//! for its own sake: a flick with follow-through is a flick that missed its
//! window, and a haul without any is a mime. The one thing looseness may not
//! buy here is delay -- Fissure has thirteen frames of startup and `HEAVY`
//! arms are still climbing at frame eight, which is why the slam is `MARTIAL`
//! and its weight lives in the pose instead.

use crate::bake::{Key, Looseness, Recipe};
use crate::clips::locomotion::stance;
use crate::ease::Ease;
use view::clips::Clip;
use view::pose::Pose;
use view::skeleton::Joint;

/// Constant speed at the *hands*, which is not constant speed in the angles.
///
/// A shoulder near the end of its range moves a hand a long way for a few
/// degrees and hardly at all for the same few in the middle, so a gap spread
/// evenly in angle space still spikes half again as fast through its middle as
/// at either end. This spends the gap's time the other way round. It is only
/// worth reaching for where a hand has more ground to cover than frames to
/// cover it in, which in this file is the second half of the Fissure's drop
/// and nowhere else.
const THROUGH: Ease = Ease::new(0.10, 0.34, 0.90, 0.66);

pub fn clips() -> Vec<Recipe> {
    vec![bolt(), fissure(), fire_pillar(), heavy()]
}

// ---------------------------------------------------------------------------
// The base she works from
// ---------------------------------------------------------------------------

/// The stance all three moves leave from and come back to.
///
/// The idle's own legs, untouched. The renderer cross-fades into an attack,
/// but a fade is for covering a change of shape, not for hiding a stamp: feet
/// that were planted a moment ago should still be planted where they were, and
/// taking them from the stance rather than inventing them means there is
/// nothing down there to fade. Only the arms are hers -- hands up and open in
/// front of the sternum, elbows soft, wrists cocked back. A fighter's guard
/// covers the head; a caster's is a pair of hands already half way into a
/// gesture.
fn ready() -> Pose {
    stance()
        .shoulder_l(24.0, 18.0, -8.0)
        .elbow_l(78.0)
        .wrist_l(-20.0, 6.0, 0.0)
        .shoulder_r(18.0, 20.0, -8.0)
        .elbow_r(70.0)
        .wrist_r(-18.0, 6.0, 0.0)
}

/// Where the idle's ankles are, read back out of the idle rather than typed
/// again. `plant_*` takes ankle positions and `joint_at` returns them, so this
/// round-trips exactly and cannot drift when the stance is retuned.
fn ankles() -> ([f32; 3], [f32; 3]) {
    let s = stance();
    (s.joint_at(Joint::FootL), s.joint_at(Joint::FootR))
}

/// Put the feet back where the idle has them, after the hips have moved.
///
/// Called at the end of every pose below, because a planted foot is solved for
/// from wherever the hips ended up -- and in two of these three clips the hips
/// travel thirty centimetres. Keying the legs by hand instead is how a caster
/// ends up skating on the spot while both her hands are in the ground.
///
/// `rise` lifts an ankle and rolls that foot onto its toe. That is the only
/// way a heel comes off the floor without the toe going through it, and both
/// heels do come off at the top of a wind-up.
fn footing(pose: Pose, rise_l: f32, rise_r: f32) -> Pose {
    let (l, r) = ankles();
    let pose = pose
        .plant_l([l[0], l[1] + rise_l, l[2]])
        .plant_r([r[0], r[1] + rise_r, r[2]]);
    let pose = if rise_l > 0.01 {
        pose.toe_floor_l()
    } else {
        pose.toe_l(0.0)
    };
    if rise_r > 0.01 {
        pose.toe_floor_r()
    } else {
        // The rear heel is light in the idle and stays light: flat at this
        // angle is more dorsiflexion than the ankle has.
        pose.toe_r(10.0)
    }
}

// ---------------------------------------------------------------------------
// Bolt
// ---------------------------------------------------------------------------

/// The neutral tool: a flicked ranged jab, thrown constantly.
///
/// Seven frames of startup is under the reaction threshold, so nothing here is
/// a telegraph -- the opponent cannot answer a bolt on sight, only learn its
/// shape and expect it. What the animation owes them instead is honesty about
/// *where the hand is*, because the bolt leaves the hand and the hand is the
/// only thing on screen that says which way it went.
///
/// It leaves her sixty per cent of her walking speed, and the renderer blends
/// the legs back toward whatever she is walking when it does, so this clip
/// keeps its legs still and out of the way. All of the motion is above the
/// waist and most of it is below the elbow.
fn bolt() -> Recipe {
    let clip = Clip::ElementalistPoke;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    // The cock lands in the first third of the startup. Any later and the
    // whole move is one frame of arm and nothing before it.
    let cock = (contact / 3).max(2);
    let drop = recover + (end - recover) / 3;
    let home = end.saturating_sub(1);

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "The bolt is thrown from the elbow and the wrist. The shoulder \
                moves about twelve degrees across the whole move and the elbow \
                moves ninety, which is what makes it read as a flick rather \
                than a punch -- and what lets it be thrown while walking \
                without the walk falling apart. The off hand comes up as a \
                sight line along the bolt's path, so a spectator can tell where \
                it went from the pose alone. The legs are deliberately \
                untouched: the move keeps sixty per cent of her walking speed, \
                and the renderer blends a real stride in underneath, so \
                anything authored down there would fight it."
            .into(),
        keys: vec![
            // Fast out of neutral, so the silhouette has already changed on
            // frame one. With seven frames to work in there is no room for a
            // wind-up that waits.
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(cock, cocked(), Ease::ANTICIPATE),
            Key::eased(contact, released(), Ease::STRIKE),
            Key::eased(recover, flicked(), Ease::OUT),
            Key::eased(drop, lowered(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// Hand back past the ear, chest wound to her right, off hand coming up.
fn cocked() -> Pose {
    footing(
        ready()
            .hips(0.015, -0.065, -0.02)
            .spine(3.0, 0.0, 13.0)
            .chest(0.0, 0.0, 22.0)
            // The head stays on the target while the chest turns away from it.
            // Winding the head with the chest is how an aim turns into a swing.
            .head(-2.0, 0.0, -20.0)
            .shoulder_r(40.0, 28.0, 18.0)
            .elbow_r(118.0)
            .wrist_r(-38.0, 0.0, 0.0)
            // The off hand goes out along the line as a sight. It is the only
            // thing in the pose that says which way the bolt went, since the
            // bolt itself is a hitbox four metres away.
            .shoulder_l(66.0, 12.0, -12.0)
            .elbow_l(40.0)
            .wrist_l(-12.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// The release, on the frame the hitbox appears: the elbow has snapped open
/// and the wrist has thrown the hand through.
fn released() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.055, 0.02)
            .spine(6.0, 0.0, -5.0)
            .chest(4.0, 0.0, -14.0)
            .head(-4.0, 0.0, 6.0)
            .shoulder_r(52.0, 16.0, 6.0)
            .elbow_r(24.0)
            .wrist_r(22.0, 0.0, 0.0)
            .shoulder_l(30.0, 20.0, -16.0)
            .elbow_l(72.0)
            .wrist_l(-22.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// The end of the active window. The hand has carried past the line and the
/// fingers have opened; nothing is being held any more.
fn flicked() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.05, 0.025)
            .spine(7.0, 0.0, -7.0)
            .chest(5.0, 0.0, -17.0)
            .head(-5.0, 0.0, 8.0)
            .shoulder_r(56.0, 20.0, 2.0)
            .elbow_r(14.0)
            .wrist_r(34.0, 0.0, 0.0)
            .shoulder_l(26.0, 22.0, -18.0)
            .elbow_l(78.0)
            .wrist_l(-26.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// Recovery: the arm falls out of the line and the chest squares back up.
fn lowered() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.06, 0.0)
            .spine(5.0, 0.0, 0.0)
            .chest(2.0, 0.0, -4.0)
            .head(-3.0, 0.0, 3.0)
            .shoulder_r(34.0, 22.0, -6.0)
            .elbow_r(48.0)
            .wrist_r(4.0, 0.0, 0.0)
            .shoulder_l(24.0, 20.0, -10.0)
            .elbow_l(76.0)
            .wrist_l(-22.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

// ---------------------------------------------------------------------------
// Fissure
// ---------------------------------------------------------------------------

/// Both hands driven into the ground; the crack races seven metres out.
///
/// The power goes **down**. Everything is arranged to say so: the arms go up
/// only so that they have somewhere to come down from, the hips fall thirty
/// centimetres, and the deepest frame of the whole clip is on the first
/// recovery frame rather than on contact -- the weight is still arriving after
/// the hands have landed, which is what collapsing onto something looks like.
fn fissure() -> Recipe {
    let clip = Clip::ElementalistCommitted;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    // Up in the first quarter of the startup, at the top by a little over
    // half, and then falling for the rest of it.
    let lift = (contact / 4).max(2);
    let top = (contact * 5 / 9).max(lift + 1);
    // Halfway down, in time as well as in the pose. The fall is the longest
    // hand travel in the file and the only gap in it that has ever been over
    // the continuity ceiling; splitting it in two is what lets the top of the
    // arc keep its anticipation without the middle of the drop turning into a
    // teleport.
    let half_way = top + (contact - top) / 2;
    let peeling = recover + (end - recover) * 2 / 5;
    let home = end.saturating_sub(3);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "A slam, but a caster's: the arc is mostly torso rather than \
                shoulder, because a body folding ninety degrees over its own \
                hands moves them further than any arm can. The hands finish \
                just off the floor rather than flat on it -- going the last \
                twenty centimetres costs a fold nobody can stand up from in \
                twenty-two frames of recovery, and it reads as the same thing. \
                MARTIAL rather than HEAVY on purpose: HEAVY's forearms are \
                still climbing at frame eight of a thirteen-frame startup, so \
                the weight would arrive as delay. It is in the pose instead, in \
                the hips and in the fact that the lowest frame is after the \
                hit. The drop is keyed in two halves rather than one: a metre \
                and a half of hand travel is what hands do at their fastest \
                even across the whole six frames, and hung on a single gap the \
                middle two frames of it took a third each."
            .into(),
        keys: vec![
            // Up fast, so the shape is on screen while the opponent still has
            // a decision. This is the frame that separates a fissure from a
            // pillar: hands rising against hands sinking.
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(lift, raised(), Ease::SMOOTH),
            // The hang is in the pose -- arched back, both heels up -- and
            // ANTICIPATE takes it a hair further before it drops, which is the
            // last thing an opponent sees before committing.
            Key::eased(top, gathered(), Ease::ANTICIPATE),
            Key::eased(half_way, falling(), THROUGH),
            Key::eased(contact, driven(), Ease::STRIKE),
            Key::eased(recover, braced(), Ease::OUT),
            Key::eased(peeling, peeled(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// The tell: both hands sweeping up and out, the body growing taller.
fn raised() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.02, -0.02)
            .root(-2.0, 0.0, -8.0)
            .spine(-4.0, 0.0, 3.0)
            .chest(-2.0, 0.0, 4.0)
            // Already looking at the ground she is about to hit.
            .head(12.0, 0.0, 4.0)
            .shoulders(66.0, 24.0, -18.0)
            .elbows(96.0)
            .wrists(-28.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// The top of the wind-up: arms long and overhead, back arched, both heels off
/// the floor. The tallest frame in the file, three frames before the shortest.
fn gathered() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.02, -0.045)
            .root(-8.0, 0.0, -6.0)
            .spine(-14.0, 0.0, 2.0)
            .chest(-8.0, 0.0, 3.0)
            .head(16.0, 0.0, 3.0)
            .shoulders(150.0, 16.0, -26.0)
            .elbows(36.0)
            .wrists(-30.0, 0.0, 0.0),
        0.030,
        0.038,
    )
}

/// Halfway down, and the pose the slam used to skip.
///
/// The hands cover a metre and a half between the top of the arc and the
/// floor, which is as fast as hands go even with six frames to do it in, and
/// with the whole of it hung on one gap the middle two frames took a third of
/// it each. This is the middle of that fall: the arms level with the
/// shoulders, the fold started in the spine rather than finished, the heels
/// back down and the knees beginning to take it.
fn falling() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.16, -0.045)
            .root(1.0, 0.0, -5.0)
            .spine(15.0, 0.0, 2.0)
            .chest(13.0, 0.0, 2.0)
            .head(-8.0, 0.0, 4.0)
            .shoulders(126.0, 15.0, -19.0)
            .elbows(26.0)
            .wrists(-35.0, 0.0, 0.0),
        0.006,
        0.008,
    )
}

/// Contact, on the frame the crack appears: both hands into the ground in
/// front, the body folded over them, the knees taking the fall.
///
/// Almost all of the fold is in the spine and the chest and almost none of it
/// in the root, which matters mechanically rather than only aesthetically: the
/// root carries the legs with it, so leaning it thirty degrees into a squat
/// this deep asks the hip for thirty degrees it has already spent and swings
/// the thigh up through horizontal, where the solver's decomposition has
/// nothing left to say. Spine and chest move the shoulders just as far and
/// leave the legs alone.
fn driven() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.34, -0.04)
            .root(10.0, 0.0, -4.0)
            .spine(44.0, 0.0, 2.0)
            .chest(34.0, 0.0, 2.0)
            // Eyes up along the line the fissure runs, not down at her hands.
            // The move happens seven metres away and she should be watching it.
            .head(-34.0, 0.0, 4.0)
            .shoulders(100.0, 14.0, -12.0)
            .elbows(16.0)
            .wrists(-40.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// The first recovery frame, and the lowest pose in the clip: the weight has
/// finished arriving, the arms are locked, the shoulders are up around the ears.
fn braced() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.385, -0.02)
            .root(12.0, 0.0, -4.0)
            .spine(45.0, 0.0, 2.0)
            .chest(32.0, 0.0, 2.0)
            .head(-34.0, 0.0, 4.0)
            .shoulders(96.0, 22.0, -8.0)
            .elbows(6.0)
            .wrists(-44.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// Peeling off the floor. Half way back up, hands still low, and slower than
/// she went down -- twenty-two frames of recovery is what the move cost.
fn peeled() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.195, -0.03)
            .root(8.0, 0.0, -8.0)
            .spine(24.0, 0.0, 3.0)
            .chest(14.0, 0.0, 4.0)
            .head(-20.0, 0.0, 4.0)
            .shoulders(60.0, 22.0, -14.0)
            .elbows(56.0)
            .wrists(-26.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

// ---------------------------------------------------------------------------
// Fire pillar
// ---------------------------------------------------------------------------

/// The column comes up out of the floor five metres in front of her, and the
/// gesture is a **lift**.
///
/// It is the mirror of the fissure and it is meant to be: down first and then
/// up, against up first and then down. She sinks, takes hold of something low,
/// and hauls it up past her own head; the column is six metres tall when it is
/// grown, so the gesture has to keep going after her hands run out of room.
/// Nothing here goes forward. A throw and a lift look nothing alike, and the
/// difference is worth more than any amount of polish on either.
fn fire_pillar() -> Recipe {
    let clip = Clip::ElementalistSpecial;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    // Down inside the first fifth of the startup, low and loaded by half way,
    // and hauling for the whole second half. Seventeen frames of startup is
    // the longest in the game; it is all telegraph, and the kit says that is
    // the point.
    let sink = (contact / 5).max(2);
    let grip = (contact * 8 / 15).max(sink + 1);
    let settling = recover + (end - recover) * 2 / 5;
    let home = end.saturating_sub(3);

    Recipe {
        clip,
        looseness: Looseness::HEAVY,
        notes: "A lift, not a throw: the hands never go in front of her. She \
                sinks onto the load in the first half of the startup and hauls \
                for the whole second half, and the arms are still rising \
                through the active window because the column is still coming \
                up -- the peak of the gesture is keyed on the first recovery \
                frame rather than on contact. HEAVY suits it and can afford to: \
                seventeen frames of startup is long enough that the arms' lag \
                reads as load rather than as lateness, and the ring at the top \
                is the weight of the thing she has just pulled out of the floor."
            .into(),
        keys: vec![
            // Down fast. This frame and the fissure's third frame are opposite
            // silhouettes, which is the whole of how the two are told apart.
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(sink, sinking(), Ease::SMOOTH),
            // The last settle before the haul: ANTICIPATE drops her a shade
            // lower still, which is what taking hold of something heavy does.
            Key::eased(grip, gripped(), Ease::ANTICIPATE),
            Key::eased(contact, hauled(), Ease::STRIKE),
            Key::eased(recover, crested(), Ease::OUT),
            Key::eased(settling, settling_pose(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// The tell: both hands sweeping down and out, palms turning to the floor, the
/// knees already going.
fn sinking() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.115, -0.03)
            .root(6.0, 0.0, -10.0)
            .spine(18.0, 0.0, 3.0)
            .chest(10.0, 0.0, 4.0)
            // Eyes stay up on the spot the column will come out of.
            .head(-16.0, 0.0, 5.0)
            .shoulders(38.0, 30.0, -34.0)
            .elbows(34.0)
            .wrists(-30.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// Loaded: as low as she gets, hands down in front of the shins, back long,
/// head up. A deadlift, held for a beat.
///
/// The fold is in the spine rather than the root, for the reason `driven`
/// gives: the root takes the legs with it and this pose has already spent most
/// of the hip.
fn gripped() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.25, -0.06)
            .root(12.0, 0.0, -10.0)
            .spine(30.0, 0.0, 3.0)
            .chest(16.0, 0.0, 4.0)
            .head(-28.0, 0.0, 5.0)
            .shoulders(52.0, 20.0, -22.0)
            .elbows(30.0)
            .wrists(-22.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// Contact, on the frame the column appears: she is half way up and still
/// pulling, hands past the chest, the body opening out of the fold.
fn hauled() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.085, -0.03)
            .root(2.0, 0.0, -8.0)
            .spine(2.0, 0.0, 3.0)
            .chest(0.0, 0.0, 4.0)
            .head(-6.0, 0.0, 5.0)
            .shoulders(104.0, 26.0, -30.0)
            .elbows(56.0)
            .wrists(-24.0, 0.0, 0.0),
        0.0,
        0.014,
    )
}

/// The first recovery frame, and the top of the lift: arms long and wide above
/// her, chest open, both heels off the floor. The tallest pose in the file.
fn crested() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.01, -0.045)
            .root(-10.0, 0.0, -6.0)
            .spine(-16.0, 0.0, 2.0)
            .chest(-10.0, 0.0, 3.0)
            .head(20.0, 0.0, 4.0)
            .shoulders(156.0, 26.0, -34.0)
            .elbows(18.0)
            .wrists(-18.0, 0.0, 0.0),
        0.034,
        0.042,
    )
}

/// Coming down off it. The arms spread and fall through the sides rather than
/// dropping in front, which keeps the column between her hands to the last.
fn settling_pose() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.075, -0.01)
            .root(0.0, 0.0, -10.0)
            .spine(-2.0, 0.0, 3.0)
            .chest(-1.0, 0.0, 4.0)
            .head(8.0, 0.0, 4.0)
            .shoulders(74.0, 54.0, -30.0)
            .elbows(50.0)
            .wrists(-14.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

// ---------------------------------------------------------------------------
// Cataclysm
// ---------------------------------------------------------------------------

/// Both hands wound up behind her and thrown forward together. The heaviest
/// single gesture in the kit, and the only one that finishes in front of her.
///
/// Twenty-four frames of startup, the longest she has: the wind-up sinks and
/// coils for the whole of it, low behind her before it is ever thrown, which
/// is what tells a Cataclysm from a Fissure or a pillar on sight -- neither of
/// those winds up *backward*. Contact is the release, and the follow-through
/// carries her weight past it rather than stopping there: HEAVY earns its
/// name here more than anywhere else in the file.
fn heavy() -> Recipe {
    let clip = Clip::ElementalistHeavy;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    // Loading inside the first fifth, fully wound by three quarters of the
    // way through the startup, and held there a beat before the throw -- the
    // same shape the pillar's own lift uses, coiled the other way.
    let coil = (contact / 5).max(2);
    let wind = (contact * 3 / 4).max(coil + 1);
    let settle = recover + (end - recover) * 2 / 5;
    let home = end.saturating_sub(2);

    Recipe {
        clip,
        looseness: Looseness::HEAVY,
        notes: "A throw, not a lift or a plant: both hands wind up behind her \
                low and to one side and finish together out in front, which is \
                the one silhouette Fissure and the pillar never make -- neither \
                of them ever puts a hand ahead of her hips. The wind-up spends \
                the whole of the startup, coiling further right up to the last \
                quarter of it, so the release on contact reads as everything \
                let go at once rather than as a swing that was already moving. \
                The follow-through overbalances her forward and the recovery \
                is spent gathering that back rather than holding a pose, which \
                is what a throw this size costs."
            .into(),
        keys: vec![
            // Down and back fast, mirroring the pillar's own tell but to one
            // side instead of straight down -- this is a throw, not a lift.
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(coil, coiled(), Ease::SMOOTH),
            // Fully wound and held: ANTICIPATE is the same held-breath beat
            // the pillar's `gripped` uses before its own haul.
            Key::eased(wind, wound(), Ease::ANTICIPATE),
            Key::eased(contact, unleashed(), Ease::STRIKE),
            Key::eased(recover, overthrown(), Ease::OUT),
            Key::eased(settle, gathering(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// The tell: both hands sweep down and back to one side, knees bending, chest
/// turning away from the target to load the throw.
fn coiled() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.09, -0.03)
            .root(-6.0, 0.0, -14.0)
            .spine(-8.0, 0.0, 10.0)
            .chest(-12.0, 0.0, 16.0)
            // Eyes stay on the target while the shoulders wind away from it.
            .head(6.0, 0.0, -14.0)
            .shoulders(70.0, 26.0, -30.0)
            .elbows(64.0)
            .wrists(-24.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// Fully wound and held for a beat: as coiled as she gets, hands drawn back
/// past her hip, weight sunk low, chest turned hard away from the throw.
fn wound() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.16, -0.06)
            .root(-14.0, 0.0, -22.0)
            .spine(-18.0, 0.0, 16.0)
            .chest(-18.0, 0.0, 26.0)
            .head(16.0, 0.0, -22.0)
            .shoulders(112.0, 20.0, -34.0)
            .elbows(104.0)
            .wrists(-32.0, 0.0, 0.0),
        0.0,
        0.02,
    )
}

/// Contact, on the frame the shot leaves: both hands thrown forward and open
/// together, the chest snapped square, the whole coil spent at once.
fn unleashed() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.045, 0.05)
            .root(16.0, 0.0, 6.0)
            .spine(22.0, 0.0, -4.0)
            .chest(28.0, 0.0, -8.0)
            .head(-10.0, 0.0, 5.0)
            .shoulders(26.0, 14.0, 14.0)
            .elbows(8.0)
            .wrists(32.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// The end of the active window: carried past the release, weight still
/// travelling forward, hands lower and further out than at contact.
fn overthrown() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.05, 0.06)
            .root(20.0, 0.0, 8.0)
            .spine(26.0, 0.0, -5.0)
            .chest(32.0, 0.0, -9.0)
            .head(-12.0, 0.0, 6.0)
            .shoulders(18.0, 12.0, 10.0)
            .elbows(4.0)
            .wrists(38.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// Gathering the overbalance back in: arms falling from the throw toward
/// neutral, chest and hips coming square again.
fn gathering() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.07, 0.02)
            .root(8.0, 0.0, 0.0)
            .spine(8.0, 0.0, -1.0)
            .chest(8.0, 0.0, -3.0)
            .head(-4.0, 0.0, 2.0)
            .shoulders(32.0, 20.0, -6.0)
            .elbows(48.0)
            .wrists(-6.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}
