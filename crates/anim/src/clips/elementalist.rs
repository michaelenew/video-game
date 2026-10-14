//! Every move she has: Bolt, Fissure, the fire pillar and Cataclysm; the
//! Cinder spray, the two drafts and the two quakes; her air row (the Air
//! bolt, the Gale and Landfall); and the four of the three clicks (the Air
//! ball, the Fire carpet, the Fire fountain and the earth jump).
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
//! The Cinder spray, since v2, is the one **one-armed throw** on the floor and
//! the only move there whose hand crosses the body: it scoops low across to
//! the far hip and flings up and out to the near side, scattering coals.
//! Nothing else she throws standing puts a hand across the midline, so the
//! scoop is the tell. (The Gale's coil does, at the ribs; it is only ever
//! thrown in the air, where the spray's scoop is nothing to confuse it with.)
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
//! ## Off the floor
//!
//! Her air row was played on the grounded clips it is the air version of
//! until 2026-10-14, and the four moves of the three clicks on Tremor's. Each
//! has its own clip now, and the rule for all seven is the same: **a body
//! with nothing under it has no base, so the legs become the counterweight.**
//! Every pose of hers in the air goes through `aloft`, which takes a pose's
//! arms and trunk and hangs its legs -- tucked, scissored, or long -- instead
//! of planting them. The trunk and the arms keep the class's vocabulary, so
//! each air move reads as the ground move it is a version of:
//!
//! ```text
//!   Air bolt   the Bolt's flick exactly; legs gather, then kick out behind
//!   Gale       Cataclysm's coil, laid flat: wound at the ribs, thrown level,
//!              the hands parting as the disc opens
//!   Landfall   Fissure's up-then-down: arms overhead over a tuck, then the
//!              drop, the deepest landing she has, and a lever as the slab
//!              comes out of the floor
//!   Carpet     one arm swept down from behind and out along the floor, the
//!              body tipped forward over it -- her one horizontal shape
//! ```
//!
//! The two **takeoffs** cross the floor in the middle of the clip: the
//! startup is on it (feet where the idle has them, through `footing`) and the
//! first active frame is the one the sim lifts her, from which the legs hang.
//! They are told apart by the arms, and are opposite on purpose -- the Fire
//! fountain flings them up into a V with the legs long, the earth jump pushes
//! them down past the hips and then tucks into a ball to meet the stone.
//!
//! Three clips are played against the sim in a way their frames have to
//! answer to, and their recipes say so: **Landfall's contact frame is held
//! for the whole fall** (the startup waits for the floor, so it is a plunge
//! rather than an impact), **the Air ball's frame zero is held for the whole
//! charge** (a channel shows the clip's first frame, so it is the cup, not a
//! pose passing through), and the takeoffs' first active frame is the leap.
//!
//! ## Looseness goes up with commitment
//!
//! `CRISP`, then `MARTIAL`, then `HEAVY`, in that order. That is not variety
//! for its own sake: a flick with follow-through is a flick that missed its
//! window, and a haul without any is a mime. The one thing looseness may not
//! buy here is delay -- Fissure has thirteen frames of startup and `HEAVY`
//! arms are still climbing at frame eight, which is why the slam is `MARTIAL`
//! and its weight lives in the pose instead.
//!
//! Two variations on it, both for a reason the floor gives. The quick air
//! moves (the Air bolt, the Fire carpet, the Downdraft) are `CRISP`, because
//! eight or nine frames thrown while falling leave no room for an arm that
//! trails. And the three that cross between the floor and the air (Landfall
//! and the two takeoffs) are `SPRUNG` -- `MARTIAL` with the legs held to the
//! hips -- because a leg that lags a fast-dropping hip is a foot in the floor.

use crate::bake::{Feel, Key, Looseness, Recipe};
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
    vec![
        bolt(),
        fissure(),
        fire_pillar(),
        heavy(),
        cinder(),
        updraft(),
        downdraft(),
        quake(),
        tremor(),
        air_bolt(),
        gale(),
        landfall(),
        air_ball(),
        fire_carpet(),
        fire_fountain(),
        earth_jump(),
    ]
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

// ---------------------------------------------------------------------------
// Cinder spray
// ---------------------------------------------------------------------------

/// A handful of coals scooped and flung: one arm, low across the body and up
/// and out, the off arm swinging back to balance it.
///
/// The one-armed throw is what makes it legible against the rest of the file.
/// Bolt is one arm too, but from the *ear* and in a straight line; this hand
/// starts at the far hip, which is a place no other move of hers ever visits,
/// and travels a diagonal across the whole front. The body helps: the hips dip
/// into the scoop and rise out of it, and the chest turns *with* the arm rather
/// than winding against it -- a scatter is not a wound-up throw, it is a
/// sweep, and the coals leave the hand at the top of the arc.
fn cinder() -> Recipe {
    let clip = Clip::ElementalistCinder;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    // Down to the hip inside the first two fifths of the startup, loaded by
    // three quarters, and the arc itself is the last quarter: a scatter is
    // fast at the release and slow at the gather.
    let scoop = (contact * 2 / 5).max(2);
    let loaded = (contact * 3 / 4).max(scoop + 1);
    let settle = recover + (end - recover) * 2 / 5;
    let home = end.saturating_sub(2);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "A scatter, not a shot and not a throw with weight behind it: \
                the right hand scoops low across to the left hip, palm up, and \
                flings up and out to her own right. The chest turns with the \
                hand both ways so the whole front sweeps, and the left arm \
                swings back and down as the counterweight. The release is at \
                the top of the arc rather than out in front, because coals are \
                scattered rather than thrown at somebody -- what leaves is a \
                spray, and the spray's shape is the arc the hand drew. MARTIAL \
                so the arm rings a little at the top: the fingers open and \
                the hand keeps going after the coals have left it."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(scoop, scooping(), Ease::SMOOTH),
            Key::eased(loaded, scooped(), Ease::ANTICIPATE),
            Key::eased(contact, flung(), Ease::STRIKE),
            Key::eased(recover, scattered(), Ease::OUT),
            Key::eased(settle, easing(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// The tell: the right hand dropping and crossing toward the left hip, the
/// chest turning left with it, the knees taking the dip.
fn scooping() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.07, -0.01)
            .root(4.0, 0.0, -6.0)
            .spine(8.0, -3.0, -8.0)
            .chest(6.0, -2.0, -10.0)
            // Eyes on the target while the shoulders turn away from it.
            .head(2.0, 0.0, 18.0)
            .shoulder_r(14.0, -16.0, 24.0)
            .elbow_r(84.0)
            .wrist_r(-34.0, 0.0, 0.0)
            .shoulder_l(30.0, 26.0, -10.0)
            .elbow_l(60.0)
            .wrist_l(-16.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// Loaded: as low and as far across as the hand gets, cupped at the far hip.
/// The lowest frame of the clip.
fn scooped() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.12, -0.02)
            .root(6.0, 0.0, -10.0)
            .spine(12.0, -4.0, -12.0)
            .chest(8.0, -3.0, -14.0)
            .head(0.0, 0.0, 26.0)
            .shoulder_r(8.0, -26.0, 30.0)
            .elbow_r(96.0)
            .wrist_r(-42.0, 0.0, 0.0)
            .shoulder_l(28.0, 28.0, -12.0)
            .elbow_l(52.0)
            .wrist_l(-14.0, 0.0, 0.0),
        0.0,
        0.02,
    )
}

/// The release, on the frame the coal leaves: the arm swept up and out to her
/// own right, elbow open, fingers spread; the chest turned with it, the body
/// risen out of the dip, and the off arm back and down.
fn flung() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.02, 0.02)
            .root(-3.0, 0.0, 6.0)
            .spine(-6.0, 4.0, 8.0)
            .chest(-8.0, 4.0, 10.0)
            .head(-6.0, 0.0, -14.0)
            .shoulder_r(128.0, 44.0, -10.0)
            .elbow_r(14.0)
            .wrist_r(30.0, 8.0, 0.0)
            .shoulder_l(8.0, 22.0, -6.0)
            .elbow_l(30.0)
            .wrist_l(10.0, 0.0, 0.0),
        0.02,
        0.0,
    )
}

/// The end of the active window: the hand carried on past the release, high
/// and wide, fingers open, nothing held.
fn scattered() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.02, 0.025)
            .root(-4.0, 0.0, 8.0)
            .spine(-7.0, 5.0, 9.0)
            .chest(-9.0, 5.0, 11.0)
            .head(-7.0, 0.0, -15.0)
            .shoulder_r(140.0, 52.0, -16.0)
            .elbow_r(8.0)
            .wrist_r(38.0, 12.0, 0.0)
            .shoulder_l(2.0, 20.0, -4.0)
            .elbow_l(24.0)
            .wrist_l(12.0, 0.0, 0.0),
        0.02,
        0.0,
    )
}

/// Recovery: the arm falling out of the arc, the chest coming back square.
fn easing() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.045, 0.0)
            .root(0.0, 0.0, 4.0)
            .spine(0.0, 2.0, 4.0)
            .chest(-2.0, 2.0, 6.0)
            .head(-3.0, 0.0, -5.0)
            .shoulder_r(60.0, 30.0, -8.0)
            .elbow_r(40.0)
            .wrist_r(4.0, 4.0, 0.0)
            .shoulder_l(14.0, 20.0, -8.0)
            .elbow_l(56.0)
            .wrist_l(-10.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

// ---------------------------------------------------------------------------
// Updraft and Downdraft
// ---------------------------------------------------------------------------

/// The column of air on her own body, standing: both arms spread wide and
/// low, palms up, swept up to shoulder height as the body rises on to its
/// toes -- lifting the whole room.
///
/// **Nothing goes over the head.** The pillar's haul finishes with the hands
/// overhead and the body sunk on to the load; this finishes with the arms
/// level and the body tall, which is the opposite silhouette: wide against
/// narrow, up against down. The spread is the tell, from the second frame.
fn updraft() -> Recipe {
    let clip = Clip::ElementalistUpdraft;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let spread = (contact / 3).max(2);
    let gathered_low = (contact * 3 / 4).max(spread + 1);
    let settle = recover + (end - recover) / 2;
    let home = end.saturating_sub(2);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "A lift with the whole width of her: the arms open wide and low \
                first, palms turned up, then come up level as the heels leave \
                the floor. The chest opens and the head comes up with it. It \
                is built to read against the pillar -- that haul narrows and \
                sinks, this widens and rises -- and against Cataclysm, which \
                puts both hands out in front. MARTIAL so the arms ring at the \
                top and the body settles back on to its heels over the \
                recovery rather than dropping."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(spread, opening_wide(), Ease::SMOOTH),
            Key::eased(gathered_low, cupped_low(), Ease::ANTICIPATE),
            Key::eased(contact, lifted_wide(), Ease::STRIKE),
            Key::eased(recover, held_wide(), Ease::OUT),
            Key::eased(settle, easing_down(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// The tell: arms opening out to the sides, low, palms turning up.
fn opening_wide() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.05, 0.0)
            .spine(4.0, 0.0, 0.0)
            .chest(2.0, 0.0, 0.0)
            .head(4.0, 0.0, 0.0)
            .shoulders(14.0, 58.0, 30.0)
            .elbows(40.0)
            .wrists(-30.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// Loaded: as low and wide as the hands get, cupped, knees soft.
fn cupped_low() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.09, -0.01)
            .spine(8.0, 0.0, 0.0)
            .chest(4.0, 0.0, 0.0)
            .head(6.0, 0.0, 0.0)
            .shoulders(6.0, 72.0, 40.0)
            .elbows(48.0)
            .wrists(-40.0, 0.0, 0.0),
        0.0,
        0.02,
    )
}

/// The gust, on the frame the column goes: arms level and wide, palms up,
/// chest open, up on the toes.
fn lifted_wide() -> Pose {
    footing(
        ready()
            .hips(0.0, 0.03, 0.01)
            .spine(-8.0, 0.0, 0.0)
            .chest(-10.0, 0.0, 0.0)
            .head(-8.0, 0.0, 0.0)
            .shoulders(88.0, 84.0, 30.0)
            .elbows(12.0)
            .wrists(-24.0, 0.0, 0.0),
        0.05,
        0.05,
    )
}

/// The end of the active window: a shade higher still, arms ringing.
fn held_wide() -> Pose {
    footing(
        ready()
            .hips(0.0, 0.035, 0.01)
            .spine(-9.0, 0.0, 0.0)
            .chest(-11.0, 0.0, 0.0)
            .head(-9.0, 0.0, 0.0)
            .shoulders(96.0, 86.0, 26.0)
            .elbows(8.0)
            .wrists(-20.0, 0.0, 0.0),
        0.05,
        0.05,
    )
}

/// Recovery: back on to the heels, the arms falling from level.
fn easing_down() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.04, 0.0)
            .spine(-2.0, 0.0, 0.0)
            .chest(-3.0, 0.0, 0.0)
            .head(-2.0, 0.0, 0.0)
            .shoulders(48.0, 50.0, 10.0)
            .elbows(40.0)
            .wrists(-16.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// The column drawn falling, in the air: both palms turned down and driven
/// from the chest to full reach below the hips, the knees drawn up as the
/// arms come down -- pressing the air under her.
///
/// **The torso stays upright.** Landfall is Fissure's fold, a body bent
/// double over its hands; this keeps the spine tall and moves the arms and
/// the knees instead. Same direction, opposite body.
fn downdraft() -> Recipe {
    let clip = Clip::ElementalistDowndraft;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let raise = (contact / 2).max(2);
    let settle = recover + (end - recover) / 2;
    let home = end.saturating_sub(2);

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "A press, not a slam: the hands come up to the chest, palms \
                down, and drive straight down past the hips while the knees \
                come up to meet them. The spine stays tall all the way through \
                -- the one thing the clip must not do is fold, because folding \
                is Landfall. CRISP because it is thrown falling and has eight \
                frames of startup; anything looser arrives late."
            .into(),
        keys: vec![
            Key::eased(0, in_flight(), Ease::OUT),
            Key::eased(raise, palms_up_at_the_chest(), Ease::ANTICIPATE),
            Key::eased(contact, pressed_down(), Ease::STRIKE),
            Key::eased(recover, pressed_through(), Ease::OUT),
            Key::eased(settle, in_flight(), Ease::SMOOTH),
            Key::eased(home, in_flight(), Ease::SMOOTH),
        ],
    }
}

/// Where the arms and legs are in the air, as a base to press from: legs
/// slightly split, arms loose.
fn in_flight() -> Pose {
    ready()
        .hips(0.0, 0.0, 0.0)
        .spine(2.0, 0.0, 0.0)
        .chest(0.0, 0.0, 0.0)
        .head(-2.0, 0.0, 0.0)
        .hip_l(14.0, 4.0, 0.0)
        .knee_l(22.0)
        .hip_r(-8.0, 4.0, 0.0)
        .knee_r(16.0)
        .ankles(10.0, 0.0, 0.0)
}

/// Hands drawn up to the chest, palms turned to the floor, knees starting up.
fn palms_up_at_the_chest() -> Pose {
    in_flight()
        .spine(-2.0, 0.0, 0.0)
        .chest(-4.0, 0.0, 0.0)
        .head(4.0, 0.0, 0.0)
        .shoulders(52.0, 26.0, -20.0)
        .elbows(110.0)
        .wrists(36.0, 0.0, 0.0)
        .hip_l(30.0, 8.0, 0.0)
        .knee_l(50.0)
        .hip_r(24.0, 8.0, 0.0)
        .knee_r(44.0)
}

/// The press, on the frame the column goes: arms straight down past the
/// hips, palms flat to the floor, knees drawn well up.
fn pressed_down() -> Pose {
    in_flight()
        .spine(4.0, 0.0, 0.0)
        .chest(2.0, 0.0, 0.0)
        .head(10.0, 0.0, 0.0)
        .shoulders(-18.0, 22.0, -30.0)
        .elbows(6.0)
        .wrists(58.0, 0.0, 0.0)
        .hip_l(62.0, 10.0, 0.0)
        .knee_l(92.0)
        .hip_r(58.0, 10.0, 0.0)
        .knee_r(88.0)
        .ankles(20.0, 0.0, 0.0)
}

/// The end of the active window: the arms carried a little behind the hips,
/// the knees at their highest.
fn pressed_through() -> Pose {
    in_flight()
        .spine(5.0, 0.0, 0.0)
        .chest(3.0, 0.0, 0.0)
        .head(11.0, 0.0, 0.0)
        .shoulders(-26.0, 20.0, -34.0)
        .elbows(4.0)
        .wrists(62.0, 0.0, 0.0)
        .hip_l(66.0, 10.0, 0.0)
        .knee_l(96.0)
        .hip_r(62.0, 10.0, 0.0)
        .knee_r(92.0)
        .ankles(22.0, 0.0, 0.0)
}

// ---------------------------------------------------------------------------
// Quake and Tremor
// ---------------------------------------------------------------------------

/// The stamp: the left foot lifted and driven down, and both hands pushed out
/// low toward the patch, palms down -- pressing the shake into the floor over
/// there.
///
/// The stamp is the tell, and it is the one thing nothing else in the file
/// does: every other move keeps both feet planted. A foot in the air for a
/// third of a second is readable from anywhere.
fn quake() -> Recipe {
    let clip = Clip::ElementalistQuake;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let lift = (contact / 3).max(2);
    let high = (contact * 2 / 3).max(lift + 1);
    let settle = recover + (end - recover) / 2;
    let home = end.saturating_sub(2);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "A stamp and a push. The left knee comes up high through the \
                first two thirds of the startup while the hands draw back to \
                the hips, then the foot is driven down on contact as both \
                hands push out low and forward, palms to the floor. The push \
                is what says *over there*; Tremor keeps the stamp and drops \
                the hands straight down instead."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(lift, knee_lifting(), Ease::SMOOTH),
            Key::eased(high, knee_high(), Ease::ANTICIPATE),
            Key::eased(contact, stamped_forward(), Ease::STRIKE),
            Key::eased(recover, pushed_low(), Ease::OUT),
            Key::eased(settle, standing_back(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// Weight on to the right leg, the left knee starting up, hands drawing back.
fn knee_lifting() -> Pose {
    let (_, r) = ankles();
    ready()
        .hips(0.0, -0.03, 0.0)
        .spine(4.0, 3.0, 0.0)
        .chest(2.0, 2.0, 0.0)
        .head(4.0, 0.0, 0.0)
        .shoulders(-6.0, 22.0, 0.0)
        .elbows(60.0)
        .wrists(-20.0, 0.0, 0.0)
        .hip_l(40.0, 6.0, 0.0)
        .knee_l(70.0)
        .ankle_l(20.0, 0.0, 0.0)
        .plant_r(r)
        .toe_r(10.0)
}

/// The top of the stamp: the left knee at hip height, the hands back at the
/// hips, the body a shade taller on the standing leg.
fn knee_high() -> Pose {
    let (_, r) = ankles();
    ready()
        .hips(0.0, -0.01, 0.0)
        .spine(0.0, 4.0, 0.0)
        .chest(-2.0, 3.0, 0.0)
        .head(6.0, 0.0, 0.0)
        .shoulders(-14.0, 24.0, 4.0)
        .elbows(70.0)
        .wrists(-26.0, 0.0, 0.0)
        .hip_l(78.0, 8.0, 0.0)
        .knee_l(100.0)
        .ankle_l(26.0, 0.0, 0.0)
        .plant_r(r)
        .toe_r(10.0)
}

/// Contact: the foot driven down and planted, both hands pushed out low and
/// forward, palms down, the body dropping into the stamp.
fn stamped_forward() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.12, 0.03)
            .spine(16.0, 0.0, 0.0)
            .chest(10.0, 0.0, 0.0)
            .head(0.0, 0.0, 0.0)
            .shoulders(52.0, 12.0, -10.0)
            .elbows(14.0)
            .wrists(48.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// The end of the active window: the push carried a little further and lower.
fn pushed_low() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.13, 0.035)
            .spine(18.0, 0.0, 0.0)
            .chest(12.0, 0.0, 0.0)
            .head(-2.0, 0.0, 0.0)
            .shoulders(46.0, 12.0, -12.0)
            .elbows(8.0)
            .wrists(54.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// Recovery: coming back up out of the stamp.
fn standing_back() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.06, 0.01)
            .spine(6.0, 0.0, 0.0)
            .chest(3.0, 0.0, 0.0)
            .head(-2.0, 0.0, 0.0)
            .shoulders(28.0, 18.0, -8.0)
            .elbows(50.0)
            .wrists(0.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// The same stamp, and both arms driven straight down at her sides as the body
/// drops into a deep crouch -- the shake pressed into the floor under her own
/// feet. Down and narrow, where Quake is forward.
fn tremor() -> Recipe {
    let clip = Clip::ElementalistTremor;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let lift = (contact / 3).max(2);
    let high = (contact * 2 / 3).max(lift + 1);
    let settle = recover + (end - recover) / 2;
    let home = end.saturating_sub(2);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "Quake's stamp with the hands going the other way: instead of \
                pushing out toward a patch over there they are driven straight \
                down past the hips, and the whole body drops with them into \
                the deepest crouch in the file. Everything about it points at \
                the floor she is standing on, which is where the patch is."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(lift, knee_lifting(), Ease::SMOOTH),
            Key::eased(high, knee_high_arms_up(), Ease::ANTICIPATE),
            Key::eased(contact, stamped_down(), Ease::STRIKE),
            Key::eased(recover, pressed_low(), Ease::OUT),
            Key::eased(settle, rising_back(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// The top of Tremor's stamp: the knee high, and the arms raised to the
/// shoulders ready to come down.
fn knee_high_arms_up() -> Pose {
    let (_, r) = ankles();
    ready()
        .hips(0.0, -0.01, 0.0)
        .spine(-2.0, 4.0, 0.0)
        .chest(-4.0, 3.0, 0.0)
        .head(6.0, 0.0, 0.0)
        .shoulders(70.0, 30.0, -10.0)
        .elbows(80.0)
        .wrists(20.0, 0.0, 0.0)
        .hip_l(78.0, 8.0, 0.0)
        .knee_l(100.0)
        .ankle_l(26.0, 0.0, 0.0)
        .plant_r(r)
        .toe_r(10.0)
}

/// Contact: the foot driven down, both arms straight down at her sides, palms
/// flat, the body dropped into a deep crouch.
fn stamped_down() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.28, 0.0)
            .spine(10.0, 0.0, 0.0)
            .chest(6.0, 0.0, 0.0)
            .head(8.0, 0.0, 0.0)
            .shoulders(-8.0, 14.0, -20.0)
            .elbows(6.0)
            .wrists(56.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// The end of the active window: lower still, the arms pressed a shade back.
fn pressed_low() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.30, 0.0)
            .spine(12.0, 0.0, 0.0)
            .chest(7.0, 0.0, 0.0)
            .head(9.0, 0.0, 0.0)
            .shoulders(-14.0, 14.0, -22.0)
            .elbows(4.0)
            .wrists(60.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// Recovery: standing back up out of the crouch.
fn rising_back() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.12, 0.0)
            .spine(5.0, 0.0, 0.0)
            .chest(2.0, 0.0, 0.0)
            .head(3.0, 0.0, 0.0)
            .shoulders(8.0, 18.0, -10.0)
            .elbows(50.0)
            .wrists(-4.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

// ---------------------------------------------------------------------------
// Off the floor: the shape of a body with nothing under it
// ---------------------------------------------------------------------------

/// Any pose of hers, taken off the floor: the hips back on the body's own
/// line and the legs hung rather than planted.
///
/// `tuck` draws both knees up toward the chest (`0` hangs, `1` is a ball) and
/// `split` scissors them, the left forward and the right back (negative the
/// other way round). One function rather than a set of angles per pose for
/// the reason `air::rising` gives: every air pose in this file has to read as
/// the same body in the same element, and a dozen hand-typed pairs of legs
/// drift apart the first time one is retouched.
///
/// The hips go back to zero because a sink is a thing a body does *onto its
/// legs*, and in the air there is nothing for the legs to push against -- a
/// ground pose's crouch lifted into the air as it stands is a body that has
/// shrunk.
fn aloft(pose: Pose, tuck: f32, split: f32) -> Pose {
    let (x, z) = (pose.channels[0], pose.channels[2]);
    pose.hips(x, 0.0, z)
        .hip_l(10.0 + 58.0 * tuck + 18.0 * split, 5.0, 0.0)
        .knee_l(20.0 + 76.0 * tuck)
        .hip_r(-6.0 + 56.0 * tuck - 18.0 * split, 5.0, 0.0)
        .knee_r(16.0 + 74.0 * tuck)
        .ankles(18.0 + 6.0 * tuck, 0.0, 0.0)
}

/// `MARTIAL` with the legs held to the hips: the looseness of the three clips
/// that cross between the floor and the air.
///
/// A leg that runs behind its hips is a foot in the floor whenever the hips
/// drop fast, and these three drop them fastest -- a takeoff's crouch in two
/// or three frames, and Landfall's arrival, thirty centimetres in four. On
/// `MARTIAL`'s legs the earth jump's crouch put a sole eleven centimetres
/// through the floor and Landfall's sixteen. The trunk and the arms keep
/// `MARTIAL`'s follow-through; only the legs are tightened.
const SPRUNG: Looseness = Looseness {
    legs: Feel::new(0.6, 1.0),
    ..Looseness::MARTIAL
};

/// What every air move leaves from and comes back to: the caster's hands on
/// a body hung in the air. The same base the Downdraft presses from.
fn air_ready() -> Pose {
    in_flight()
}

// ---------------------------------------------------------------------------
// Air bolt
// ---------------------------------------------------------------------------

/// Bolt's flick thrown from the air, with the body hanging behind it.
///
/// The arm is the Bolt's exactly -- the same cock past the ear, the same
/// release from the elbow, the same sight line from the off hand -- because
/// the shot is the same shape of thing and a player who has read one has read
/// the other. What changes is everything below the waist: on the ground the
/// legs are a base the renderer walks under it, and off the floor there is no
/// base, so the legs have to be the counterweight. They gather as the hand
/// goes back and kick out behind as it comes through.
fn air_bolt() -> Recipe {
    let clip = Clip::ElementalistAirBolt;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let cock = (contact / 3).max(2);
    // Drawn a shade further a beat before the release: nine frames is two
    // more than the Bolt has, and spent as a longer glide into the same cock
    // they would only make the release arrive softer.
    let drawn = contact.saturating_sub(2).max(cock + 1);
    let drop = recover + (end - recover) / 3;
    let home = end.saturating_sub(1);

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "The Bolt's arm on a body with nothing under it. Above the \
                waist it is the same flick -- cocked past the ear, released \
                from the elbow, the off hand up as a sight line -- so the two \
                bolts read as one shot from two rows. Below it the legs do \
                what the floor did: they gather under her as the hand goes \
                back, and scissor out behind as it comes through, so the body \
                hangs back from the arm the way a throw in the air has to. \
                Two frames longer to wind than the Bolt, spent as a held draw \
                just before the release rather than as a slower cock, so the \
                release is as sharp as the grounded one. CRISP, as the Bolt is: \
                a flick with follow-through is a flick that missed its window. \
                Ends on the air base so the jump clips take over without a \
                change of shape."
            .into(),
        keys: vec![
            Key::eased(0, air_ready(), Ease::OUT),
            Key::eased(cock, aloft(cocked(), 0.6, -0.25), Ease::SMOOTH),
            Key::eased(drawn, air_drawn(), Ease::SMOOTH),
            Key::eased(contact, air_released(), Ease::STRIKE),
            Key::eased(
                recover,
                aloft(flicked(), 0.0, 0.6).root(8.0, 0.0, -13.0),
                Ease::OUT,
            ),
            Key::eased(drop, aloft(lowered(), 0.2, 0.3), Ease::SMOOTH),
            Key::eased(home, air_ready(), Ease::SMOOTH),
        ],
    }
}

/// The last frame before the release: the cock held a hair deeper, knees
/// drawn up under her.
fn air_drawn() -> Pose {
    aloft(cocked(), 0.7, -0.3)
        .chest(-2.0, 0.0, 26.0)
        .shoulder_r(36.0, 30.0, 22.0)
        .elbow_r(124.0)
}

/// The release: the Bolt's, with the hips tipped forward behind the hand and
/// the legs kicked out behind.
fn air_released() -> Pose {
    aloft(released(), 0.05, 0.55).root(8.0, 0.0, -13.0)
}

// ---------------------------------------------------------------------------
// Gale
// ---------------------------------------------------------------------------

/// Cataclysm's two-handed throw, flattened into a frisbee.
///
/// Cataclysm winds low behind her and throws forward and down, because what
/// it throws hits the floor. The Gale is a disc that flies level and opens as
/// it goes, so the wind-up is at the ribs rather than the hip, the throw
/// finishes at shoulder height rather than at the knees, and the hands part
/// after the release -- the disc opening -- where Cataclysm's stay together.
/// Same coil, same both-hands release; turned on its side.
///
/// The coil is made **with the chest**: it turns hard to her right and the
/// two hands, still in front of it, end up together at her right ribs. The
/// hips turn the other way in the air, so the coil is a twist of the whole
/// body rather than a turn of it. The far hand does cross her midline to get
/// there, which on the floor is the Cinder spray's tell alone; the Gale is
/// only ever thrown in the air, where the spray's scoop is not on offer to
/// be confused with it, and the hands are at the ribs rather than the hip.
fn gale() -> Recipe {
    let clip = Clip::ElementalistGale;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let coil = (contact / 5).max(2);
    let wind = (contact * 3 / 4).max(coil + 1);
    let drift = recover + (end - recover) * 2 / 5;
    let home = end.saturating_sub(2);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "A frisbee thrown with both hands: Cataclysm's coil and release \
                laid flat. The hands gather together at her right ribs while \
                the chest winds hard to the right and the hips counter-turn \
                under it with the knees drawn up -- a body twisting on nothing \
                -- then the chest snaps through and both hands go out level at \
                shoulder height, and part as the disc opens. Cataclysm winds \
                low and finishes low, because it hits the floor; this winds \
                at the ribs and finishes level, because the disc flies flat, \
                and the hands spreading after the release are the size the \
                Gale is about to become. The hands are placed with reach_* \
                rather than angled, because where they are depends on how far \
                the chest has turned and the read is where they are. MARTIAL \
                rather than \
                Cataclysm's HEAVY: eighteen frames of startup is long enough \
                to read and short enough that HEAVY's arms would still be \
                climbing into the coil when it is time to throw."
            .into(),
        keys: vec![
            Key::eased(0, air_ready(), Ease::OUT),
            Key::eased(coil, gale_turning(), Ease::SMOOTH),
            Key::eased(wind, gale_wound(), Ease::ANTICIPATE),
            Key::eased(contact, gale_thrown(), Ease::STRIKE),
            Key::eased(recover, gale_opened(), Ease::OUT),
            Key::eased(drift, gale_drifting(), Ease::SMOOTH),
            Key::eased(home, air_ready(), Ease::SMOOTH),
        ],
    }
}

/// The tell: the chest starting to turn right, both hands drawn in together
/// at the ribs, the knees coming up.
fn gale_turning() -> Pose {
    aloft(
        ready()
            .root(0.0, 0.0, -22.0)
            .spine(2.0, 0.0, 14.0)
            .chest(0.0, 0.0, 18.0)
            .head(0.0, 0.0, -18.0)
            .shoulders(46.0, 14.0, -16.0)
            .elbows(104.0)
            .wrists(-10.0, 0.0, 0.0),
        0.35,
        -0.15,
    )
}

/// Fully wound: chest turned hard right over hips turned left, the hands
/// stacked at her right flank, knees up. A spring with nothing to stand on.
fn gale_wound() -> Pose {
    aloft(
        ready()
            .root(-4.0, 0.0, -30.0)
            .spine(-2.0, 0.0, 22.0)
            .chest(-4.0, 0.0, 30.0)
            .head(4.0, 0.0, -30.0),
        0.6,
        -0.25,
    )
    // The two hands together at her right ribs, the near one a little back:
    // where the angles would have put them depends on how far the chest has
    // turned, and what has to be true is where they are.
    .reach_r([0.22, 1.20, 0.06])
    .reach_l([0.12, 1.24, 0.20])
    .wrists(-6.0, 0.0, 0.0)
}

/// The throw, on the frame the disc leaves: the chest snapped through square
/// and past, both hands out together at shoulder height, palms flat, legs
/// scissored open behind it.
fn gale_thrown() -> Pose {
    aloft(
        ready()
            .root(4.0, 0.0, -4.0)
            .spine(4.0, 0.0, -8.0)
            .chest(2.0, 0.0, -14.0)
            .head(-4.0, 0.0, 10.0),
        0.1,
        0.55,
    )
    .reach_r([0.07, 1.38, 0.64])
    .reach_l([-0.07, 1.38, 0.64])
    .wrists(6.0, 0.0, 0.0)
}

/// The end of the active window: the hands parting level, as wide as her
/// reach -- the disc opening.
fn gale_opened() -> Pose {
    aloft(
        ready()
            .root(4.0, 0.0, -8.0)
            .spine(2.0, 0.0, -2.0)
            .chest(-2.0, 0.0, -4.0)
            .head(-4.0, 0.0, 8.0),
        0.05,
        0.65,
    )
    .reach_r([0.46, 1.40, 0.44])
    .reach_l([-0.46, 1.40, 0.44])
    .wrists(8.0, 0.0, 0.0)
}

/// Recovery: the arms falling out of the spread, the body turning square.
fn gale_drifting() -> Pose {
    aloft(
        ready()
            .root(2.0, 0.0, -8.0)
            .spine(2.0, 0.0, -2.0)
            .chest(0.0, 0.0, -4.0)
            .head(-2.0, 0.0, 4.0)
            .shoulders(48.0, 40.0, 0.0)
            .elbows(40.0)
            .wrists(-8.0, 0.0, 0.0),
        0.25,
        0.25,
    )
}

// ---------------------------------------------------------------------------
// Landfall
// ---------------------------------------------------------------------------

/// Fissure from the air: both hands up, a hang, and the whole body brought
/// down behind them into the floor.
///
/// **One frame of this clip is held for as long as she falls.** Landfall's
/// startup ends on the floor rather than on a number: the sim runs the
/// startup down to its last frame and *keeps* it there until her feet arrive
/// (`sim::state::plunging`), and the renderer reads that as the first active
/// frame, which is also the frame her feet meet the floor. So the contact key
/// is not an impact but the **drop** -- legs reaching for the ground, hands
/// driven down ahead of her, folded over them -- because that is what is on
/// screen for the whole of a four-metre plunge, and the impact itself is the
/// three active frames after it, collapsing into the deepest crouch she has.
fn landfall() -> Recipe {
    let clip = Clip::ElementalistLandfall;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let lift = (contact / 4).max(2);
    let top = (contact * 5 / 9).max(lift + 1);
    let half_way = top + (contact - top) / 2;
    let lever = recover + (end - recover) * 2 / 5;
    let home = end.saturating_sub(3);

    Recipe {
        clip,
        looseness: SPRUNG,
        notes: "Fissure's up-then-down, with the body that Fissure keeps on \
                the floor taken off it. The hands sweep up and the knees tuck \
                into the hang -- arms long overhead, a ball underneath them, \
                the tallest and the most compact she is at once -- then the \
                legs drop out of the tuck to reach for the floor as the hands \
                come down ahead of her. The contact key is the plunge rather \
                than the impact, because the sim holds the last startup frame \
                until she lands and the renderer shows the contact frame for \
                the whole fall: legs long underneath, hands driven down in \
                front, folded over them. The landing is the three active \
                frames after it, collapsing into a crouch deeper than \
                Fissure's. Then, where Fissure peels her hands off the floor, \
                this heaves them forward and up at the angle the slab comes \
                out of the ground: the lever is the payoff, and the one frame \
                that tells the two apart once they are both on the floor. \
                MARTIAL for the reason Fissure is: the weight is in the pose, \
                and HEAVY would spend it as delay."
            .into(),
        keys: vec![
            Key::eased(0, air_ready(), Ease::OUT),
            Key::eased(lift, landfall_rising(), Ease::SMOOTH),
            Key::eased(top, landfall_hung(), Ease::ANTICIPATE),
            Key::eased(half_way, landfall_dropping(), THROUGH),
            Key::eased(contact, landfall_plunging(), Ease::STRIKE),
            Key::eased(recover, landfall_landed(), Ease::OUT),
            Key::eased(lever, landfall_levered(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// The tell, and Fissure's: both hands sweeping up, here with the knees
/// starting up under her.
fn landfall_rising() -> Pose {
    aloft(
        ready()
            .root(-2.0, 0.0, -8.0)
            .spine(-4.0, 0.0, 3.0)
            .chest(-2.0, 0.0, 4.0)
            .head(14.0, 0.0, 4.0)
            .shoulders(70.0, 24.0, -18.0)
            .elbows(92.0)
            .wrists(-28.0, 0.0, 0.0),
        0.45,
        0.0,
    )
}

/// The hang: arms long overhead, knees tucked high, looking down at the
/// floor she is about to hit.
///
/// Not arched as Fissure's top is. Arched over tucked knees, the body sits
/// back in the air as if into a chair; upright, the tuck and the raised arms
/// make one tall, narrow shape, which is the one that reads as poised.
fn landfall_hung() -> Pose {
    aloft(
        ready()
            .root(2.0, 0.0, -6.0)
            .spine(-2.0, 0.0, 2.0)
            .chest(-4.0, 0.0, 3.0)
            .head(24.0, 0.0, 3.0)
            .shoulders(150.0, 26.0, -14.0)
            .elbows(30.0)
            .wrists(-30.0, 0.0, 0.0),
        0.9,
        0.0,
    )
}

/// Half way down: the arms level with the shoulders, the fold begun, the legs
/// coming out of the tuck.
fn landfall_dropping() -> Pose {
    aloft(
        ready()
            .root(4.0, 0.0, -5.0)
            .spine(14.0, 0.0, 2.0)
            .chest(12.0, 0.0, 2.0)
            .head(4.0, 0.0, 4.0)
            .shoulders(124.0, 15.0, -19.0)
            .elbows(24.0)
            .wrists(-34.0, 0.0, 0.0),
        0.45,
        0.0,
    )
}

/// The plunge, held for the whole fall and still on the frame her feet
/// arrive: legs long under her and reaching for the floor, both hands driven
/// down in front, the body folded over them, eyes on where she is landing.
fn landfall_plunging() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.10, -0.03)
            .root(8.0, 0.0, -4.0)
            .spine(30.0, 0.0, 2.0)
            .chest(24.0, 0.0, 2.0)
            .head(-24.0, 0.0, 4.0)
            .shoulders(98.0, 16.0, -12.0)
            .elbows(10.0)
            .wrists(-40.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// The first recovery frame: the landing finished arriving -- lower than
/// Fissure ever gets, hands into the floor, arms locked.
fn landfall_landed() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.40, -0.03)
            .root(12.0, 0.0, -4.0)
            .spine(44.0, 0.0, 2.0)
            .chest(32.0, 0.0, 2.0)
            .head(-36.0, 0.0, 4.0)
            .shoulders(92.0, 24.0, -8.0)
            .elbows(6.0)
            .wrists(-46.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// The lever: hands heaved forward and up at the slab's own angle as it comes
/// out of the floor, the body rising behind them.
fn landfall_levered() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.20, 0.02)
            .root(8.0, 0.0, -6.0)
            .spine(20.0, 0.0, 2.0)
            .chest(10.0, 0.0, 2.0)
            .head(-16.0, 0.0, 4.0)
            .shoulders(146.0, 20.0, -10.0)
            .elbows(10.0)
            .wrists(-40.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

// ---------------------------------------------------------------------------
// Air ball
// ---------------------------------------------------------------------------

/// Both hands cupped round a ball of air, then rolled away.
///
/// **The hold is frame zero.** The Air ball is a charge held *before* the
/// move (`sim::moves::Charge::Gather`), and `view::play` shows a channel as
/// the clip's first frame for as long as the button is down. So the cup is
/// not a wind-up passing through: it is the pose she stands in while the
/// ball grows, for up to a second, and the four frames of startup after the
/// release are only the roll. The ball itself is out where the crosshair met
/// the floor; this is the gesture of holding it, which is what says the hold
/// is hers.
fn air_ball() -> Recipe {
    let clip = Clip::ElementalistAirBall;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let dip = (contact / 2).max(1);
    let settle = recover + (end - recover) * 2 / 5;
    let home = end.saturating_sub(2);

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "The held frame is the whole tell: one palm over and one palm \
                under, a ball's width apart in front of her belly, elbows out, \
                knees soft -- a body holding something round. It is frame \
                zero because a held channel shows frame zero, so it is what \
                is on screen for as long as the ball is growing. On the \
                release the hands drop down and back past the hips and swing \
                forward \
                and up together, underhand, palms up and open, as the body \
                rises behind them: a ball bowled with both hands, sent off \
                the fingertips at waist height. Quake also puts both hands \
                out low in front of her, but palms down and pressing, from a \
                stamp; this is palms up and rising, from a cup, and the feet \
                never leave the floor. CRISP: four frames of startup is a \
                flick, and the read was made during the hold."
            .into(),
        keys: vec![
            // Straight out of the hold and down: the read was made while it
            // was held, and four frames is no time to ease out of anything.
            Key::eased(0, cupping(), Ease::STRIKE),
            Key::eased(dip, bowled_low(), Ease::SMOOTH),
            Key::eased(contact, rolled(), Ease::OUT),
            Key::eased(recover, rolled_through(), Ease::SMOOTH),
            Key::eased(settle, rolled_after(), Ease::SMOOTH),
            Key::eased(home, ready(), Ease::SMOOTH),
        ],
    }
}

/// Held for as long as the ball grows: the right palm over, the left palm
/// under, a ball's width apart in front of the belly, elbows out.
fn cupping() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.10, -0.01)
            .root(4.0, 0.0, -12.0)
            .spine(10.0, 0.0, 4.0)
            .chest(4.0, 0.0, 4.0)
            .head(0.0, 0.0, 4.0),
        0.0,
        0.0,
    )
    // Placed rather than angled: the whole read is two hands one over the
    // other on her centre line, and that is a statement about where the
    // wrists are. The ball they hold is about thirty centimetres across.
    .reach_r(CUP_OVER)
    .wrist_r(24.0, 0.0, 0.0)
    .reach_l(CUP_UNDER)
    .wrist_l(-30.0, 0.0, 0.0)
}

/// Where the wrists are while the Air ball is held, in character space: the
/// right over and the left under, a ball's width apart in front of the belly.
const CUP_OVER: [f32; 3] = [0.02, 1.30, 0.38];
const CUP_UNDER: [f32; 3] = [-0.04, 1.03, 0.40];

/// The backswing: both hands dropped down and a little back past the hips,
/// the body sinking over them -- the bottom of a two-handed underhand bowl.
fn bowled_low() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.17, -0.03)
            .root(8.0, 0.0, -10.0)
            .spine(24.0, 0.0, 3.0)
            .chest(10.0, 0.0, 3.0)
            .head(-20.0, 0.0, 4.0)
            .shoulders(-4.0, 12.0, -10.0)
            .forearm_l(14.0, 60.0)
            .forearm_r(14.0, -60.0)
            .wrists(-24.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// The release, on the frame the ball goes: both arms swung through forward
/// and up to waist height, palms up and open, the body risen behind them.
fn rolled() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.05, 0.04)
            .root(6.0, 0.0, -8.0)
            .spine(8.0, 0.0, 2.0)
            .chest(4.0, 0.0, 2.0)
            .head(-6.0, 0.0, 4.0)
            .shoulders(64.0, 10.0, -6.0)
            .forearm_l(10.0, 70.0)
            .forearm_r(10.0, -70.0)
            .wrists(-30.0, 0.0, 0.0),
        0.0,
        0.02,
    )
}

/// The end of the active window: the hands carried on up past the release,
/// open, the rear heel up.
fn rolled_through() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.035, 0.05)
            .root(4.0, 0.0, -8.0)
            .spine(4.0, 0.0, 2.0)
            .chest(0.0, 0.0, 2.0)
            .head(-6.0, 0.0, 4.0)
            .shoulders(82.0, 12.0, -4.0)
            .forearm_l(8.0, 70.0)
            .forearm_r(8.0, -70.0)
            .wrists(-26.0, 0.0, 0.0),
        0.0,
        0.03,
    )
}

/// Recovery: the arms lowering out of the throw.
fn rolled_after() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.05, 0.02)
            .spine(5.0, 0.0, 4.0)
            .chest(2.0, 0.0, 5.0)
            .head(-3.0, 0.0, 4.0)
            .shoulders(40.0, 16.0, -8.0)
            .forearm_l(40.0, 30.0)
            .forearm_r(40.0, -30.0)
            .wrists(-16.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

// ---------------------------------------------------------------------------
// Fire carpet
// ---------------------------------------------------------------------------

/// One arm swept low and out along the line, laying a strip of fire ahead.
///
/// Thrown in the air, so the arm has the whole height of her to sweep
/// through: drawn up and back behind the shoulder, then brought down past the
/// hip and out along the floor ahead, palm down, and *held there* while the
/// strip runs out from it. The body tips forward over the arm and the legs
/// trail behind, so for the active window she is long and level over the
/// fire she is laying -- the only horizontal silhouette in the file.
///
/// One-armed like the Bolt and the Cinder spray, and told from both by where
/// the hand goes: the Bolt's leaves from the ear in a straight line, the
/// spray's crosses to the far hip and flings up; this one goes back and over
/// and comes down, and finishes lower than it started.
fn fire_carpet() -> Recipe {
    let clip = Clip::ElementalistFireCarpet;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let draw = (contact / 2).max(2);
    // The bottom of the arc, keyed: the hand has a metre and a half to
    // travel from behind the shoulder to out along the floor, and hung on
    // one gap the middle of it is faster than a hand goes.
    let passing = draw + (contact - draw) / 2;
    let laid = recover + (end - recover) * 2 / 5;
    let home = end.saturating_sub(2);

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "A one-armed sweep downward: the right arm drawn up and back \
                behind the shoulder with the chest turned after it, then \
                brought down past the hip and out ahead of her along the \
                floor, palm down, fingers spread. The body tips forward over \
                the arm and the legs trail out behind, so on the active frames \
                she is long and level above the strip -- the one horizontal \
                shape she makes. The arm stays out along the line through the \
                first part of the recovery, because the fire is drawn running \
                from the near end to the far one and the hand is where it \
                comes from. The left arm goes back and wide as the \
                counterweight. Told from the Bolt (from the ear, straight) and \
                the Cinder spray (across, and up) by the hand coming *down*. \
                CRISP, as the Downdraft is and for its reason: eight frames of \
                startup thrown in the air, and on MARTIAL the sweeping arm was \
                still hanging straight down on the contact frame, two frames \
                behind the fire it was supposed to be laying."
            .into(),
        keys: vec![
            Key::eased(0, air_ready(), Ease::OUT),
            Key::eased(draw, carpet_drawn(), THROUGH),
            Key::eased(passing, carpet_passing(), THROUGH),
            Key::eased(contact, carpet_swept(), Ease::STRIKE),
            Key::eased(recover, carpet_laid(), Ease::OUT),
            Key::eased(laid, carpet_trailing(), Ease::SMOOTH),
            Key::eased(home, air_ready(), Ease::SMOOTH),
        ],
    }
}

/// The tell: the right arm up and back behind the shoulder, the chest turned
/// after it, knees drawn a little under her.
fn carpet_drawn() -> Pose {
    aloft(
        ready()
            .root(-4.0, 0.0, -10.0)
            .spine(-6.0, 0.0, 12.0)
            .chest(-6.0, 0.0, 18.0)
            .head(6.0, 0.0, -18.0)
            .shoulder_r(-40.0, 40.0, 10.0)
            .elbow_r(40.0)
            .wrist_r(-10.0, 0.0, 0.0)
            .shoulder_l(56.0, 22.0, -10.0)
            .elbow_l(40.0)
            .wrist_l(-14.0, 0.0, 0.0),
        0.4,
        0.1,
    )
}

/// The bottom of the sweep: the arm straight down past the hip, the chest
/// turning back through square, the body starting to tip over it.
fn carpet_passing() -> Pose {
    aloft(
        ready()
            .root(6.0, 0.0, -12.0)
            .spine(8.0, 0.0, 4.0)
            .chest(2.0, 0.0, 4.0)
            .head(-10.0, 0.0, -4.0)
            .shoulder_r(0.0, 22.0, 4.0)
            .elbow_r(14.0)
            .wrist_r(10.0, 0.0, 0.0)
            .shoulder_l(14.0, 36.0, -4.0)
            .elbow_l(28.0)
            .wrist_l(0.0, 0.0, 0.0),
        0.2,
        0.0,
    )
}

/// Contact, on the frame the carpet is laid: the arm swept down and out ahead
/// along the floor, palm down; the body tipped forward over it and the legs
/// trailing; the left arm back and wide.
fn carpet_swept() -> Pose {
    aloft(
        ready()
            .root(10.0, 0.0, -12.0)
            .spine(16.0, 0.0, -6.0)
            .chest(6.0, 0.0, -10.0)
            .head(-24.0, 0.0, 6.0)
            // Forward of the chest by more than it looks: the body is tipped
            // thirty degrees over the arm, and that much of the swing only
            // brings it back to hanging straight down.
            .shoulder_r(76.0, 14.0, 0.0)
            .elbow_r(6.0)
            .wrist_r(26.0, 0.0, 0.0)
            .shoulder_l(-24.0, 44.0, 0.0)
            .elbow_l(18.0)
            .wrist_l(10.0, 0.0, 0.0),
        0.0,
        -0.1,
    )
    .hip_l(-10.0, 6.0, 0.0)
    .hip_r(-20.0, 6.0, 0.0)
    .knee_l(32.0)
    .knee_r(24.0)
}

/// The end of the active window: further along and lower still, the body
/// near level.
fn carpet_laid() -> Pose {
    aloft(
        ready()
            .root(12.0, 0.0, -12.0)
            .spine(19.0, 0.0, -8.0)
            .chest(8.0, 0.0, -12.0)
            .head(-28.0, 0.0, 6.0)
            .shoulder_r(86.0, 12.0, 0.0)
            .elbow_r(4.0)
            .wrist_r(30.0, 0.0, 0.0)
            .shoulder_l(-28.0, 48.0, 0.0)
            .elbow_l(14.0)
            .wrist_l(12.0, 0.0, 0.0),
        0.0,
        -0.1,
    )
    .hip_l(-12.0, 6.0, 0.0)
    .hip_r(-24.0, 6.0, 0.0)
    .knee_l(34.0)
    .knee_r(26.0)
}

/// Recovery: the arm still out along the strip, the body coming back up
/// over it.
fn carpet_trailing() -> Pose {
    aloft(
        ready()
            .root(8.0, 0.0, -12.0)
            .spine(10.0, 0.0, -2.0)
            .chest(4.0, 0.0, -4.0)
            .head(-12.0, 0.0, 4.0)
            .shoulder_r(60.0, 20.0, -4.0)
            .elbow_r(30.0)
            .wrist_r(10.0, 0.0, 0.0)
            .shoulder_l(10.0, 36.0, -6.0)
            .elbow_l(40.0)
            .wrist_l(-6.0, 0.0, 0.0),
        0.15,
        0.0,
    )
}

// ---------------------------------------------------------------------------
// Fire fountain
// ---------------------------------------------------------------------------

/// Both arms flung up as fire bursts under her and throws her up.
///
/// A takeoff: the startup is on the floor and the first active frame is the
/// frame the sim lifts her (`sim::state`, the leap of a leaping move), so
/// everything before contact keeps its feet where the idle has them and
/// everything from contact on hangs its legs. The crouch is down onto the
/// fire -- palms flat to the floor at her sides, head bowed over the spot --
/// and the launch is everything opened at once: arms flung up and wide in a
/// V, legs long and together with the toes pointed, the head thrown back. The
/// tallest and the most open shape she has, which is what a body thrown by a
/// fountain looks like.
fn fire_fountain() -> Recipe {
    let clip = Clip::ElementalistFireFountain;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    // Down by half way, so the fling has the other half: from the floor to
    // over the head is most of two metres of hand travel, and the arms are
    // still going up through the active window -- the V is reached on the
    // first recovery frame, not on contact.
    let crouch = (contact / 2).max(2);
    // The legs open while the feet are still down: the last startup frame is
    // a push from the toes, so the straight legs of the launch are reached by
    // extending rather than by swinging a planted foot through the floor.
    let drive = contact.saturating_sub(1).max(crouch + 1);
    let riding = recover + (end - recover) * 2 / 5;
    let home = end.saturating_sub(2);

    Recipe {
        clip,
        looseness: SPRUNG,
        notes: "Down onto the fire, then thrown up by it. The crouch presses \
                both palms flat to the floor at her sides with her head bowed \
                over the spot, which is the only thing in the clip an opponent \
                has time to read; on contact, the frame the burst lifts her, \
                the arms are flung up and wide into a V, the legs go long and \
                together with the toes pointed, and the head goes back. The V \
                is held and widened through the active window while the burst \
                burns under her, and then the arms come down into the shape \
                the jump's own rise holds, so the hand-off to `jump_rise` is a \
                continuation. Told from the earth jump by the arms: these go \
                up and open, the earth jump's push down and stay low. Told \
                from the pillar's haul, which also ends overhead, by the feet: \
                the pillar never leaves the floor."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(crouch, fountain_crouch(), Ease::ANTICIPATE),
            Key::eased(drive, fountain_driving(), Ease::SMOOTH),
            Key::eased(contact, fountain_flung(), Ease::STRIKE),
            Key::eased(recover, fountain_borne(), Ease::OUT),
            Key::eased(riding, fountain_riding(), Ease::SMOOTH),
            Key::eased(home, fountain_risen(), Ease::SMOOTH),
        ],
    }
}

/// The crouch onto the fire: palms flat to the floor at her sides, head bowed.
fn fountain_crouch() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.26, -0.02)
            .root(8.0, 0.0, -10.0)
            .spine(18.0, 0.0, 4.0)
            .chest(8.0, 0.0, 4.0)
            .head(18.0, 0.0, 4.0)
            .shoulders(8.0, 32.0, -10.0)
            .elbows(16.0)
            .wrists(56.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// The push: up on both toes, legs nearly long, the arms coming up through
/// the front.
fn fountain_driving() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.05, 0.0)
            .root(2.0, 0.0, -10.0)
            .spine(4.0, 0.0, 3.0)
            .chest(0.0, 0.0, 3.0)
            .head(-6.0, 0.0, 3.0)
            .shoulders(72.0, 32.0, -6.0)
            .elbows(22.0)
            .wrists(10.0, 0.0, 0.0),
        0.11,
        0.11,
    )
}

/// The launch, on the frame the burst throws her: arms flung up past the
/// shoulders on their way to the V, legs long and together, toes pointed,
/// head back.
fn fountain_flung() -> Pose {
    aloft(
        ready()
            .root(-4.0, 0.0, -10.0)
            .spine(-8.0, 0.0, 3.0)
            .chest(-8.0, 0.0, 3.0)
            .head(-20.0, 0.0, 3.0)
            .wrists(-10.0, 0.0, 0.0),
        0.0,
        0.0,
    )
    // Up off the floor by what the push left her: the legs are long and the
    // toes pointed, and at the height of a standing body they would be through
    // it on the one frame the sim has not yet lifted her.
    .hips(0.0, 0.08, 0.0)
    .hips_both(-4.0, 3.0, 0.0)
    .knees(4.0)
    .ankles(32.0, 0.0, 0.0)
    // On their way up and already apart: placed, because overhead is where a
    // shoulder's two angles stop meaning one thing each, and a V is a
    // statement about where the hands are.
    .reach_r([0.34, 1.80, 0.38])
    .reach_l([-0.34, 1.80, 0.38])
}

/// The end of the burst: the V at its widest, the legs starting to trail.
fn fountain_borne() -> Pose {
    aloft(
        ready()
            .root(-5.0, 0.0, -10.0)
            .spine(-8.0, 0.0, 3.0)
            .chest(-6.0, 0.0, 3.0)
            .head(-16.0, 0.0, 3.0)
            .wrists(-8.0, 0.0, 0.0),
        0.0,
        0.0,
    )
    .hip_l(-12.0, 5.0, 0.0)
    .hip_r(-8.0, 5.0, 0.0)
    .knee_l(20.0)
    .knee_r(14.0)
    .ankles(40.0, 0.0, 0.0)
    .reach_r(FOUNTAIN_V)
    .reach_l([-FOUNTAIN_V[0], FOUNTAIN_V[1], FOUNTAIN_V[2]])
}

/// Where the right wrist is at the top of the fountain's V, in character
/// space: up and out over the shoulder, a little in front of the head.
const FOUNTAIN_V: [f32; 3] = [0.50, 1.90, 0.12];

/// Riding it up: the arms coming down out of the V.
fn fountain_riding() -> Pose {
    aloft(
        ready()
            .root(-4.0, 0.0, -10.0)
            .spine(-5.0, 0.0, 4.0)
            .chest(-3.0, 0.0, 5.0)
            .head(-12.0, 0.0, 4.0)
            .wrists(-10.0, 0.0, 0.0),
        0.0,
        0.0,
    )
    .hip_l(-18.0, 6.0, 0.0)
    .hip_r(-12.0, 5.0, 0.0)
    .knee_l(36.0)
    .knee_r(28.0)
    .ankles(38.0, 0.0, 0.0)
    .reach_r([0.40, 1.76, 0.22])
    .reach_l([-0.40, 1.76, 0.22])
}

/// The hand-off: close to what `jump_rise` holds, arms up and the legs
/// trailing, so the jump clips take over without a change of shape.
fn fountain_risen() -> Pose {
    aloft(
        ready()
            .root(-5.0, 0.0, -9.0)
            .spine(-5.0, 0.0, 6.0)
            .chest(-2.0, 0.0, 7.0)
            .head(-12.0, 0.0, 4.0)
            .shoulder_l(110.0, 26.0, 0.0)
            .elbow_l(26.0)
            .shoulder_r(100.0, 32.0, 0.0)
            .elbow_r(34.0)
            .wrists(-8.0, 0.0, 0.0),
        0.0,
        0.0,
    )
    .hip_l(-20.0, 6.0, 0.0)
    .hip_r(-13.0, 5.0, 0.0)
    .knee_l(42.0)
    .knee_r(32.0)
    .ankles(38.0, 0.0, 0.0)
}

// ---------------------------------------------------------------------------
// Earth jump
// ---------------------------------------------------------------------------

/// A crouch and a spring as a stone comes up under her.
///
/// Three frames of startup and the leap on the fourth, so the crouch is all
/// there is to read and it has to be deep. The hands go *down* in front of
/// the knees, palms to the floor, and on the spring they push down and back
/// past the hips -- driving the stone up and herself off it -- rather than
/// being thrown up as a jump throws them. Then the knees come up under her
/// into a tuck and the hands open out low at her sides, palms down, and she
/// looks down: she is waiting for the stone to meet her feet, which it does
/// on the way up. Low and compact, where the fountain is tall and open.
fn earth_jump() -> Recipe {
    let clip = Clip::ElementalistEarthJump;
    let (_, contact, recover) = clip.phases().expect("an attack clip has phases");
    let end = clip.length() - 1;
    let crouch = contact.saturating_sub(1).max(1);
    let tuck = recover + (end - recover) * 2 / 5;
    let home = end.saturating_sub(1);

    Recipe {
        clip,
        looseness: SPRUNG,
        notes: "A crouch, a push and a tuck. The crouch is deep and quick -- \
                three frames of startup -- with both hands reaching down in \
                front of the knees, palms to the floor; the spring on the \
                frame she leaves pushes the hands down and back past the hips \
                as the legs drive, so the arms say the force went into the \
                ground rather than into the sky. Off the floor the knees come \
                up into a tuck and the hands open low at her sides, palms \
                down, head down: she is watching for the stone, which meets \
                her feet on the way up and which she should look ready to \
                land on. The fountain is the same takeoff told the other way \
                -- arms up, legs long -- so the two are opposite silhouettes \
                from the launch frame on. Ends in the tuck rather than in the \
                jump's own rise, because the stone arriving under her is what \
                comes next."
            .into(),
        keys: vec![
            Key::eased(0, ready(), Ease::OUT),
            Key::eased(crouch, earth_crouch(), Ease::STRIKE),
            Key::eased(contact, earth_spring(), Ease::OUT),
            Key::eased(recover, earth_off(), Ease::SMOOTH),
            Key::eased(tuck, earth_tucked(), Ease::SMOOTH),
            Key::eased(home, earth_riding(), Ease::SMOOTH),
        ],
    }
}

/// The crouch: deep, both hands reaching down in front of the knees, palms to
/// the floor, eyes up.
fn earth_crouch() -> Pose {
    footing(
        ready()
            .hips(0.0, -0.28, 0.0)
            .root(10.0, 0.0, -10.0)
            .spine(22.0, 0.0, 4.0)
            .chest(10.0, 0.0, 4.0)
            .head(-16.0, 0.0, 4.0)
            .shoulders(46.0, 14.0, -10.0)
            .elbows(12.0)
            .wrists(44.0, 0.0, 0.0),
        0.0,
        0.0,
    )
}

/// The spring, on the frame she leaves: legs driving long, up on the toes,
/// both hands pushed down and back past the hips.
fn earth_spring() -> Pose {
    footing(
        ready()
            .hips(0.0, 0.0, 0.02)
            .root(4.0, 0.0, -10.0)
            .spine(6.0, 0.0, 4.0)
            .chest(2.0, 0.0, 4.0)
            .head(-6.0, 0.0, 4.0)
            .shoulders(-22.0, 20.0, -6.0)
            .elbows(8.0)
            .wrists(50.0, 0.0, 0.0),
        0.10,
        0.10,
    )
}

/// Off the floor: legs long and pointed, the hands still down and back.
fn earth_off() -> Pose {
    aloft(
        ready()
            .root(2.0, 0.0, -10.0)
            .spine(4.0, 0.0, 4.0)
            .chest(2.0, 0.0, 4.0)
            .head(4.0, 0.0, 4.0)
            .shoulders(-14.0, 30.0, -6.0)
            .elbows(10.0)
            .wrists(46.0, 0.0, 0.0),
        0.1,
        0.0,
    )
    .ankles(36.0, 0.0, 0.0)
}

/// The tuck: knees drawn up under her, the hands open low at her sides,
/// palms down, looking down for the stone.
fn earth_tucked() -> Pose {
    aloft(
        ready()
            .root(4.0, 0.0, -10.0)
            .spine(10.0, 0.0, 4.0)
            .chest(4.0, 0.0, 4.0)
            .head(22.0, 0.0, 4.0)
            .shoulders(18.0, 48.0, -10.0)
            .elbows(18.0)
            .wrists(40.0, 0.0, 0.0),
        1.0,
        0.0,
    )
}

/// The last frame: still tucked, arms out a little higher, ready to land.
fn earth_riding() -> Pose {
    aloft(
        ready()
            .root(2.0, 0.0, -10.0)
            .spine(8.0, 0.0, 4.0)
            .chest(2.0, 0.0, 4.0)
            .head(16.0, 0.0, 4.0)
            .shoulders(26.0, 52.0, -10.0)
            .elbows(24.0)
            .wrists(32.0, 0.0, 0.0),
        0.85,
        0.0,
    )
}
