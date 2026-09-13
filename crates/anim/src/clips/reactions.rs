//! Being hit, held, and finished.
//!
//! The one family where nothing the character does is their own idea. Every
//! other clip is a decision; these are all consequences, and they are read by
//! the *other* player as feedback that their hit landed. That is what they are
//! for, and it is why they are worth authoring loudly: a hit that does not move
//! the body reads as a hit that did not connect, however much the health bar
//! moved.
//!
//! ## The stun clips are authored backwards
//!
//! `hit_light`, `hit_heavy` and `stagger` are indexed **from the end**:
//! `HitStun { left }` counts down, and `play::from_the_end` lines the clip's
//! last frame up with the frame control returns. Two things follow, and they
//! decide the shape of all three clips:
//!
//! - **Frame zero is the impact and the last frame is recovered.** The last key
//!   is `locomotion::stance()` itself, a few frames before the end so the
//!   springs are settled by the time anyone can act again. Ending on anything
//!   else puts a pop on the frame the player gets their controls back, which is
//!   the one frame they are guaranteed to be looking at.
//! - **A stun longer than the clip holds frame zero.** So frame zero has to be
//!   a pose that survives being stared at for a third of a second: hurt and
//!   off balance, not a pose that only makes sense in motion.
//!
//! A short stun is the other half of that: it starts part way in. A 14-frame
//! poke enters `hit_light` around frame two, so the recoil has to be legible
//! from the first few frames rather than building to something.
//!
//! ## The other three
//!
//! `launched` and `grabbed` loop on their own clocks -- airborne frames and
//! frames held -- so neither has an ending to aim at and both have to close on
//! themselves. `defeat` runs forward from the round-over pause and holds its
//! last pose, which is the only clip in the game that is *meant* to stop
//! moving.
//!
//! ## Looseness
//!
//! `LIMP` nearly everywhere, because that is what this family is: the body is
//! being moved rather than moving, and the lag and ring do the work of
//! whipping a head and dragging a hand that nobody wants to key. The two
//! exceptions are argued for where they are set -- a light hit never loses
//! control of the feet, and a struggle is the one thing here the character is
//! actually doing.

use crate::bake::{Key, Looseness, Recipe};
use crate::clips::locomotion::stance;
use crate::ease::Ease;
use view::clips::Clip;
use view::pose::{ANKLE_ON_GROUND as GROUND, Pose};

/// Where the stance's feet are, so a reaction can put them back exactly where
/// it found them. The same numbers `locomotion::stance` plants at: left foot
/// forward, right foot back and bladed.
const L: f32 = -0.145;
const R: f32 = 0.145;
const LEAD_Z: f32 = 0.16;
const REAR_Z: f32 = -0.15;

pub fn clips() -> Vec<Recipe> {
    vec![
        hit_light(),
        hit_heavy(),
        stagger(),
        launched(),
        grabbed(),
        defeat(),
    ]
}

// ---------------------------------------------------------------------------
// Taking a hit
// ---------------------------------------------------------------------------

/// A poke landing: sixteen frames, and the body keeps its feet.
///
/// Picked by `stun_total`, so this is what every 14 to 16 frame poke in the
/// game looks like -- the most frequently seen clip in the family by a wide
/// margin, and the one that has to survive being watched a hundred times a
/// match.
fn hit_light() -> Recipe {
    // The impact. Everything goes backwards at once -- the head furthest,
    // because it weighs little and sits on the end of the longest lever on the
    // body -- and the guard is knocked open on the way.
    let snap = stance()
        .hips(0.0, -0.050, -0.085)
        .root(-14.0, 3.0, -10.0)
        .spine(-11.0, 3.0, 13.0)
        .chest(-9.0, 4.0, 16.0)
        .head(-34.0, -10.0, 20.0)
        .shoulder_l(-8.0, 32.0, -10.0)
        .elbow_l(30.0)
        .wrist_l(-18.0, 0.0, 0.0)
        .shoulder_r(-22.0, 26.0, 0.0)
        .elbow_r(52.0)
        .wrist_r(-14.0, 0.0, 0.0)
        .plant_l([L, GROUND, LEAD_Z])
        .plant_r([R, GROUND, REAR_Z])
        .toe_l(-10.0)
        .toe_r(16.0);

    // Four frames on, and still going: the weight has arrived on the back foot
    // and the hips are at the end of their travel. This is the frame a short
    // stun spends most of its time near, so it has to read as hit on its own.
    let carried = stance()
        .hips(0.0, -0.085, -0.105)
        .root(-9.0, 2.0, -12.0)
        .spine(-2.0, 2.0, 10.0)
        .chest(-3.0, 3.0, 13.0)
        .head(-20.0, -8.0, 16.0)
        .shoulder_l(-2.0, 26.0, -6.0)
        .elbow_l(44.0)
        .shoulder_r(-14.0, 24.0, 0.0)
        .elbow_r(58.0)
        .wrists(-12.0, 0.0, 0.0)
        .plant_l([L, GROUND, LEAD_Z])
        .plant_r([R, GROUND, REAR_Z])
        .toe_l(-6.0)
        .toe_r(18.0);

    // The fold: what the head started, the gut finishes. The recoil turns into
    // a crumple over the blow, which is the half of a flinch that says it hurt
    // rather than that it was loud.
    let fold = stance()
        .hips(0.0, -0.105, -0.045)
        .root(8.0, 1.0, -12.0)
        .spine(14.0, 2.0, 8.0)
        .chest(9.0, 3.0, 10.0)
        .head(12.0, -5.0, 11.0)
        .shoulder_l(10.0, 17.0, 0.0)
        .elbow_l(64.0)
        .shoulder_r(-3.0, 19.0, 0.0)
        .elbow_r(58.0)
        .plant_l([L, GROUND, LEAD_Z - 0.01])
        .plant_r([R, GROUND, REAR_Z - 0.01])
        .toe_l(-2.0)
        .toe_r(16.0)
        .wrists(-6.0, 0.0, 0.0);

    // Gathering: hands on their way back up, head last. The guard closing is
    // the tell that the character is about to be able to act, and it wants to
    // arrive a beat before they can.
    let gather = stance()
        .hips(0.0, -0.065, -0.012)
        .root(3.0, 0.0, -13.0)
        .spine(8.0, 1.0, 7.0)
        .chest(4.0, 1.0, 8.0)
        .head(3.0, -2.0, 7.0)
        .shoulder_l(11.0, 14.0, 0.0)
        .elbow_l(48.0)
        .shoulder_r(2.0, 16.0, 0.0)
        .elbow_r(40.0)
        .plant_l([L, GROUND, LEAD_Z])
        .plant_r([R, GROUND, REAR_Z])
        .toe_l(-1.0)
        .toe_r(12.0);

    Recipe {
        clip: Clip::HitLight,
        // Not LIMP. Sixteen frames is not long enough to lose your feet, and a
        // poke that made someone go slack would read as a knockdown. What is
        // loose here is the head and the hands; the legs stay under the body,
        // and MARTIAL is exactly that split.
        looseness: Looseness::MARTIAL,
        notes: "A poke landing. Frame zero is the impact because the clip is \
                indexed from the end, and a short stun starts part way in, so \
                the recoil has to read immediately rather than build: the head \
                is already thrown on the first frame. The fold three frames \
                later is what makes it hurt -- a head snap on its own reads as \
                a flinch at a noise. The feet never leave the floor, because \
                sixteen frames of stun is not a knockdown and the character has \
                to be able to walk out of it."
            .into(),
        keys: vec![
            // An impact is instantaneous; everything after it is the body
            // catching up. OUT leaves the contact fast and arrives slowly,
            // which is a body being carried by a blow rather than leaning over.
            Key::eased(0, snap, Ease::OUT),
            Key::eased(3, carried, Ease::SMOOTH),
            Key::eased(7, fold, Ease::SMOOTH),
            Key::eased(10, gather, Ease::OUT),
            // Settled before control returns, so the frame the player can act
            // on is the stance and not a spring still moving.
            Key::eased(13, stance(), Ease::OUT),
        ],
    }
}

/// A committed hit landing: twenty-six frames, and nothing about it is
/// controlled.
///
/// Chosen whenever `stun_total` is longer than `hit_light`, which is every
/// heavy and every special in the game -- 22 to 36 frames. The long ones hold
/// frame zero first, so the blast pose has to be worth holding.
fn hit_heavy() -> Recipe {
    // Everything at once and in the same direction: hips driven back, spine
    // arched over them, head last and furthest, both arms thrown open. The
    // guard is not knocked aside here -- it is gone.
    let blast = stance()
        .hips(-0.02, -0.075, -0.145)
        .root(-15.0, 5.0, -6.0)
        .spine(-14.0, 5.0, 16.0)
        .chest(-12.0, 6.0, 20.0)
        .head(-40.0, -11.0, 24.0)
        .shoulder_l(-26.0, 48.0, -18.0)
        .elbow_l(18.0)
        .wrist_l(-24.0, 0.0, 0.0)
        .shoulder_r(-42.0, 30.0, 0.0)
        .elbow_r(26.0)
        .wrist_r(-20.0, 0.0, 0.0)
        .plant_l([L, GROUND + 0.02, LEAD_Z + 0.02])
        .plant_r([R, GROUND, REAR_Z])
        .toe_l(-14.0)
        .toe_r(18.0);

    // The whip: the hips have stopped going back and the torso has not. The
    // rear leg is already swinging under to find the floor somewhere behind.
    let through = stance()
        .hips(-0.03, -0.105, -0.185)
        .root(-4.0, 6.0, -4.0)
        .spine(2.0, 6.0, 14.0)
        .chest(-2.0, 7.0, 18.0)
        .head(-24.0, -12.0, 22.0)
        .shoulder_l(-12.0, 44.0, -14.0)
        .elbow_l(34.0)
        .shoulder_r(-30.0, 28.0, 0.0)
        .elbow_r(44.0)
        .plant_l([L, GROUND + 0.05, LEAD_Z + 0.03])
        .plant_r([R + 0.03, GROUND + 0.09, REAR_Z - 0.13])
        .toe_l(-18.0)
        .toe_r(24.0);

    // The catch: the stumble step lands a long way back and the body folds
    // over it. Head hanging, arms swinging loose and low -- this is the frame
    // that says the hit was heavy rather than merely long.
    let catch = stance()
        .hips(-0.015, -0.175, -0.120)
        .root(20.0, 3.0, -8.0)
        .spine(24.0, 3.0, 8.0)
        .chest(13.0, 4.0, 10.0)
        .head(20.0, -6.0, 12.0)
        .shoulder_l(6.0, 20.0, 0.0)
        .elbow_l(40.0)
        .shoulder_r(-10.0, 18.0, 0.0)
        .elbow_r(34.0)
        .wrists(-10.0, 0.0, 0.0)
        .plant_l([L, GROUND + 0.01, LEAD_Z - 0.02])
        .plant_r([R + 0.04, GROUND, REAR_Z - 0.26])
        .toe_l(-6.0)
        .toe_r(4.0);

    // Gathering the feet back under: the rear foot comes in light, the head
    // comes up last, and the guard is still not closed.
    let gather = stance()
        .hips(0.0, -0.105, -0.035)
        .root(8.0, 1.0, -11.0)
        .spine(11.0, 2.0, 8.0)
        .chest(6.0, 2.0, 9.0)
        .head(9.0, -3.0, 8.0)
        .shoulder_l(10.0, 16.0, 0.0)
        .elbow_l(50.0)
        .shoulder_r(-2.0, 17.0, 0.0)
        .elbow_r(44.0)
        .plant_l([L, GROUND, LEAD_Z])
        .plant_r([R + 0.01, GROUND + 0.03, REAR_Z - 0.07])
        .toe_l(-2.0)
        .toe_r(18.0);

    Recipe {
        clip: Clip::HitHeavy,
        looseness: Looseness::LIMP,
        notes: "A heavy landing, and a whole-body event: the hips go first, the \
                torso arches after them and the head arrives last and furthest. \
                The stumble step is the point of the clip -- a character who \
                keeps their feet through a thirty-frame stun looks like they \
                were not hit, and the step is what makes the knockback read as \
                distance covered rather than as sliding. Nothing is keyed to \
                arrive anywhere; LIMP's lag does the whipping, which is why the \
                arms are authored as thrown open and then simply left."
            .into(),
        keys: vec![
            Key::eased(0, blast, Ease::STRIKE),
            // The body carrying on past the blow, and losing the floor.
            Key::eased(5, through, Ease::OUT),
            // Landing on the stumble step. OUT again: it arrives and sags.
            Key::eased(11, catch, Ease::OUT),
            Key::eased(18, gather, Ease::SMOOTH),
            Key::eased(23, stance(), Ease::OUT),
        ],
    }
}

/// What a parried attacker eats. The biggest punish window in the game, and it
/// has to look like an invitation.
///
/// Thirty-four frames, which is `defence.parry_stagger` -- long enough to land
/// the slowest startup in the game, so every frame of it is a frame somebody is
/// choosing what to hit this body with.
fn stagger() -> Recipe {
    // The deflection. They were mid-swing: the weapon arm has been knocked
    // across the body, the torso has over-rotated after it, and the weight is
    // already past the lead foot with nothing left to stop it.
    //
    // The torso twist is kept under about forty degrees, spread across the
    // spine and the chest. Past that the shoulder swings round to the wrong
    // side of the body and takes the arm's whole frame with it: "spread" turns
    // into "backwards", and both arms end up sticking out horizontally like
    // scarecrow poles. Over-rotation is the point of the pose, but it belongs
    // in the hips and the head, which have room for it.
    let deflected = stance()
        .hips(0.03, -0.065, 0.070)
        .root(13.0, -5.0, -6.0)
        .spine(6.0, -4.0, -14.0)
        .chest(2.0, -5.0, -22.0)
        .head(-16.0, 11.0, -28.0)
        .shoulder_l(20.0, 16.0, 0.0)
        .elbow_l(72.0)
        .shoulder_r(40.0, 10.0, -34.0)
        .elbow_r(38.0)
        .wrist_r(-30.0, 0.0, 0.0)
        .plant_l([L + 0.02, GROUND, LEAD_Z + 0.06])
        .plant_r([R, GROUND + 0.02, REAR_Z + 0.02])
        .toe_l(-4.0)
        .toe_r(26.0);

    // Overbalanced: falling forward onto a foot that is not there yet, and
    // sideways off the line of the parry. Both hands have dropped below the
    // chest and the centre line is wide open.
    let tipping = stance()
        .hips(0.055, -0.100, 0.135)
        .root(20.0, -7.0, 4.0)
        .spine(9.0, -5.0, -12.0)
        .chest(3.0, -5.0, -18.0)
        .head(-8.0, 14.0, -22.0)
        .shoulder_l(4.0, 18.0, 0.0)
        .elbow_l(58.0)
        .shoulder_r(16.0, 34.0, -14.0)
        .elbow_r(44.0)
        .wrists(-16.0, 0.0, 0.0)
        .plant_l([L + 0.03, GROUND, LEAD_Z + 0.10])
        .plant_r([R + 0.04, GROUND + 0.14, REAR_Z + 0.12])
        .toe_l(4.0)
        .toe_r(20.0);

    // The catch: the rear foot slams down wide and crossed in front, which is
    // what stops a fall and is also the most open the body gets -- feet too far
    // apart to move, chest square to the front, both hands at hip height.
    let caught = stance()
        .hips(0.075, -0.150, 0.090)
        .root(16.0, -8.0, 8.0)
        .spine(5.0, -5.0, -10.0)
        .chest(0.0, -5.0, -14.0)
        .head(-12.0, 12.0, -14.0)
        .shoulder_l(-6.0, 24.0, 0.0)
        .elbow_l(44.0)
        .shoulder_r(2.0, 44.0, -10.0)
        .elbow_r(38.0)
        .plant_l([L + 0.01, GROUND, LEAD_Z + 0.02])
        .plant_r([R + 0.18, GROUND, 0.21])
        .toe_l(-6.0)
        .toe_r(-6.0);

    // Hanging there. The punish window proper: nothing moving fast, the guard
    // still down, and the head coming up to watch it arrive.
    let hanging = stance()
        .hips(0.055, -0.135, 0.055)
        .root(13.0, -6.0, 6.0)
        .spine(4.0, -4.0, -8.0)
        .chest(1.0, -4.0, -10.0)
        .head(-8.0, 8.0, -10.0)
        .shoulder_l(0.0, 22.0, 0.0)
        .elbow_l(54.0)
        .shoulder_r(6.0, 36.0, -6.0)
        .elbow_r(46.0)
        .plant_l([L, GROUND, LEAD_Z])
        .plant_r([R + 0.17, GROUND, 0.19])
        .toe_l(-2.0)
        .toe_r(-4.0);

    // Pulling the feet back under, guard on its way up. Still late, still
    // behind: this is the last third of the window, not the end of it.
    let gather = stance()
        .hips(0.02, -0.095, 0.015)
        .root(6.0, -3.0, -6.0)
        .spine(6.0, -2.0, 0.0)
        .chest(3.0, -2.0, -2.0)
        .head(0.0, 2.0, 0.0)
        .shoulder_l(4.0, 20.0, 0.0)
        .elbow_l(46.0)
        .shoulder_r(-2.0, 24.0, 0.0)
        .elbow_r(40.0)
        .plant_l([L, GROUND, LEAD_Z])
        .plant_r([R + 0.06, GROUND + 0.05, 0.0])
        .toe_l(0.0)
        .toe_r(12.0);

    Recipe {
        clip: Clip::Stagger,
        looseness: Looseness::LIMP,
        notes: "The parried attacker. They were already committed forward, so \
                this is a fall they are chasing rather than a push they are \
                absorbing: the weapon arm is knocked across the body, the torso \
                over-rotates after it, and the rear foot has to be thrown out \
                in front to stop the fall. That step is what makes the pose \
                open -- feet wide and crossed up, hands below the waist, chest \
                square to the front -- and it is held through the middle of the \
                clip on purpose, because this is a thirty-four frame punish \
                window and the attacker choosing what to throw should be able \
                to see that it is one."
            .into(),
        keys: vec![
            // Parried: the deflection happens to them in one frame.
            Key::eased(0, deflected, Ease::STRIKE),
            // Falling. IN, because a body going over accelerates.
            Key::eased(6, tipping, Ease::IN),
            // The foot slams down and the body sags onto it.
            Key::eased(13, caught, Ease::OUT),
            // HOLD: the window stays put until the last instant, which is the
            // whole point of it.
            Key::eased(20, hanging, Ease::HOLD),
            Key::eased(27, gather, Ease::SMOOTH),
            Key::eased(32, stance(), Ease::OUT),
        ],
    }
}

// ---------------------------------------------------------------------------
// Off the ground, and in someone else's hands
// ---------------------------------------------------------------------------

/// Airborne in hitstun, after an uppercut. Loops on airborne frames, so it has
/// no beginning and no end -- it is what the body is doing while it is up
/// there, for as long as that lasts.
fn launched() -> Recipe {
    // Arched over backwards with everything trailing: arms thrown back past the
    // head because the hips left without them, knees folded and feet well off
    // the floor because nothing is holding the legs up. The feet are the read
    // here -- a launched body whose soles stay at ankle height looks like a
    // standing one, whatever the spine is doing.
    let thrown = Pose::rest()
        .hips(0.0, -0.02, -0.03)
        .root(-24.0, 4.0, -8.0)
        .spine(-13.0, 4.0, 10.0)
        .chest(-10.0, 5.0, 14.0)
        .head(-38.0, -8.0, 16.0)
        .shoulder_l(-70.0, 40.0, -22.0)
        .elbow_l(24.0)
        .wrist_l(-30.0, 0.0, 0.0)
        .shoulder_r(-78.0, 30.0, -16.0)
        .elbow_r(36.0)
        .wrist_r(-24.0, 0.0, 0.0)
        .hip_l(30.0, 10.0, 0.0)
        .knee_l(30.0)
        .ankle_l(40.0, 0.0, 0.0)
        .hip_r(18.0, 14.0, 0.0)
        .knee_r(58.0)
        .ankle_r(44.0, 0.0, 0.0);

    // A quarter turn of the wallow: the legs scissor past each other and the
    // arms drift down. Nothing here is a decision, which is why every part
    // moves a different amount.
    let wallow = thrown
        .root(-17.0, -6.0, -16.0)
        .spine(-7.0, -5.0, 4.0)
        .chest(-5.0, -6.0, 8.0)
        .head(-24.0, 8.0, 6.0)
        .shoulder_l(-54.0, 48.0, -30.0)
        .elbow_l(46.0)
        .shoulder_r(-70.0, 24.0, -8.0)
        .elbow_r(18.0)
        .hip_l(38.0, 12.0, 0.0)
        .knee_l(64.0)
        .ankle_l(32.0, 0.0, 0.0)
        .hip_r(10.0, 8.0, 0.0)
        .knee_r(24.0);

    // Half way: the arch has bled off into a sag, the head has rolled to the
    // other side, and the legs are at the far end of their swing.
    let sag = thrown
        .hips(0.0, -0.04, -0.01)
        .root(-9.0, 2.0, -2.0)
        .spine(-3.0, 3.0, 14.0)
        .chest(-1.0, 4.0, 18.0)
        .head(-14.0, -10.0, 22.0)
        .shoulder_l(-40.0, 34.0, -16.0)
        .elbow_l(62.0)
        .shoulder_r(-46.0, 40.0, -24.0)
        .elbow_r(52.0)
        .hip_l(14.0, 6.0, 0.0)
        .knee_l(42.0)
        .ankle_l(28.0, 0.0, 0.0)
        .hip_r(-2.0, 16.0, 0.0)
        .knee_r(72.0)
        .ankle_r(30.0, 0.0, 0.0);

    // Coming back round toward the arch, with the limbs on their own schedule
    // so the loop does not tick.
    let roll = thrown
        .root(-21.0, 8.0, -12.0)
        .spine(-10.0, 7.0, 6.0)
        .chest(-8.0, 8.0, 10.0)
        .head(-30.0, 4.0, 4.0)
        .shoulder_l(-62.0, 26.0, -12.0)
        .elbow_l(34.0)
        .shoulder_r(-68.0, 38.0, -26.0)
        .elbow_r(26.0)
        .hip_l(25.0, 14.0, 0.0)
        .knee_l(50.0)
        .hip_r(13.0, 10.0, 0.0)
        .knee_r(38.0)
        .ankle_r(36.0, 0.0, 0.0);

    Recipe {
        clip: Clip::Launched,
        looseness: Looseness::LIMP,
        notes: "Taken into the air by somebody else's uppercut. The read is the \
                arch: the hips are the highest thing on the body and everything \
                else hangs off them behind, which is the shape of a person \
                who was lifted rather than one who jumped. It loops on airborne \
                frames because a launch lasts as long as the launch lasts, so \
                there is nothing to time it against -- what the loop carries is \
                a slow wallow with the arms, legs and head all on different \
                phases, which is what stops a tumble reading as a spin cycle. \
                No part of it reaches for the ground: gathering the legs under \
                you is a thing you do when you are in control, and the whole \
                point of this clip is that you are not."
            .into(),
        keys: vec![
            Key::eased(0, thrown, Ease::SMOOTH),
            Key::eased(6, wallow, Ease::SMOOTH),
            Key::eased(12, sag, Ease::SMOOTH),
            Key::eased(18, roll, Ease::SMOOTH),
        ],
    }
}

/// Held at arm's length by somebody with a fistful of you. Loops for as long as
/// the grab lasts.
fn grabbed() -> Recipe {
    // Where the captor's fist is. The grip is the one thing in this clip that
    // does not move: the hands are solved onto it on every key and everything
    // else thrashes around it. Keying the arms as angles instead would swing
    // the hands about with the shoulders, which reads as flailing at nothing --
    // and the whole point is that there is something there, and it is not
    // letting go.
    let grip_l = |slip: f32| [-0.105 + slip, 1.255, 0.335];
    let grip_r = |slip: f32| [0.090 + slip, 1.215, 0.325];

    // Hauled up onto the front foot, leaning away from the grip, weight
    // nowhere useful.
    let held = stance()
        .hips(0.0, -0.040, 0.040)
        .root(-12.0, 0.0, -6.0)
        .spine(-7.0, 0.0, 4.0)
        .chest(-3.0, 0.0, 6.0)
        .head(-10.0, 0.0, 2.0)
        .plant_l([L - 0.01, GROUND, LEAD_Z - 0.02])
        .plant_r([R + 0.01, GROUND + 0.04, REAR_Z + 0.04])
        .toe_l(6.0)
        .toe_r(24.0)
        .reach_l(grip_l(0.0))
        .reach_r(grip_r(0.0));

    // Hauling the grip one way, hips thrown after it, rear foot dragged round.
    let heave = held
        .hips(-0.045, -0.055, 0.030)
        .root(-14.0, 9.0, -24.0)
        .spine(-4.0, 10.0, -15.0)
        .chest(3.0, 11.0, -22.0)
        .head(0.0, -13.0, -24.0)
        .plant_l([L - 0.05, GROUND, LEAD_Z + 0.04])
        .plant_r([R + 0.05, GROUND + 0.09, REAR_Z - 0.02])
        .toe_l(2.0)
        .toe_r(30.0)
        .reach_l(grip_l(-0.02))
        .reach_r(grip_r(-0.015));

    // Wrenching back the other way, both hands twisting at the wrist that is
    // holding them. The grip does not care.
    let wrench = held
        .hips(0.050, -0.070, 0.020)
        .root(-4.0, -11.0, 19.0)
        .spine(-10.0, -9.0, 13.0)
        .chest(-5.0, -10.0, 21.0)
        .head(-16.0, 12.0, 22.0)
        .plant_l([L - 0.02, GROUND + 0.07, LEAD_Z - 0.07])
        .plant_r([R + 0.03, GROUND, REAR_Z + 0.01])
        .toe_l(22.0)
        .toe_r(6.0)
        .reach_l(grip_l(0.025))
        .reach_r(grip_r(0.03));

    // Both feet finally down and everything driven downward: the one honest
    // attempt to break the grip by getting some weight under it.
    let brace = held
        .hips(0.010, -0.130, -0.020)
        .root(9.0, 2.0, -3.0)
        .spine(6.0, 1.0, 2.0)
        .chest(3.0, 1.0, 4.0)
        .head(9.0, -3.0, 5.0)
        .plant_l([L - 0.03, GROUND, LEAD_Z + 0.08])
        .plant_r([R + 0.03, GROUND, REAR_Z - 0.07])
        .toe_l(-8.0)
        .toe_r(10.0)
        .reach_l(grip_l(-0.005))
        .reach_r(grip_r(0.01));

    // And hauled straight back up onto the toes, which is where it started and
    // is why the loop is a loop.
    let lifted = held
        .hips(-0.010, -0.020, 0.055)
        .root(-17.0, -4.0, -10.0)
        .spine(-11.0, -3.0, 6.0)
        .chest(-5.0, -3.0, 8.0)
        .head(-15.0, 4.0, 6.0)
        .plant_l([L - 0.01, GROUND + 0.03, LEAD_Z - 0.01])
        .plant_r([R + 0.01, GROUND + 0.07, REAR_Z + 0.06])
        .toe_l(16.0)
        .toe_r(28.0)
        .reach_l(grip_l(-0.01))
        .reach_r(grip_r(0.0));

    Recipe {
        clip: Clip::Grabbed,
        // Not LIMP. A grab is the one thing in this file the character is
        // fighting, and LIMP's four frames of arm lag turn every heave into a
        // slow waft -- which reads as hanging there. HEAVY still carries past
        // the mark and rings, but it arrives on the frame it was asked to.
        looseness: Looseness::HEAVY,
        notes: "Held at arm's length, and losing. This is the difference \
                between being grabbed and being stunned: a stun is something \
                you wait out, so it is allowed to settle, and a grab is \
                something that is happening to you continuously, so no frame of \
                this may. Four different fights against the same grip -- haul \
                sideways, wrench back, get both feet down and drive, get pulled \
                up onto the toes again -- with the feet scrabbling for a new \
                spot every time and never finding one. The hands are solved \
                onto a fixed point in front of the chest rather than keyed as \
                angles: the grip is the thing that does not move, and a body \
                twisting away from hands that stay put is what being held looks \
                like."
            .into(),
        keys: vec![
            Key::eased(0, held, Ease::OUT),
            Key::eased(6, heave, Ease::SMOOTH),
            Key::eased(12, wrench, Ease::OUT),
            // Into the brace: the character gathers and drives, so the time is
            // spent gathering.
            Key::eased(18, brace, Ease::IN),
            // And out of it in one yank, because that half is not their doing.
            Key::eased(24, lifted, Ease::STRIKE),
        ],
    }
}

// ---------------------------------------------------------------------------
// The end of the round
// ---------------------------------------------------------------------------

/// Out of health. Fifty-two frames, clocked from the round-over pause, and it
/// holds its last pose because the round is over and nothing else is coming.
fn defeat() -> Recipe {
    // The blow that did it, an instant after. Still standing, but the legs have
    // already stopped being asked to hold anything up.
    let struck = stance()
        .hips(0.0, -0.105, -0.060)
        .root(-8.0, 4.0, -8.0)
        .spine(-6.0, 4.0, 12.0)
        .chest(-4.0, 5.0, 14.0)
        .head(-30.0, -8.0, 16.0)
        .shoulder_l(-14.0, 28.0, -10.0)
        .elbow_l(26.0)
        .shoulder_r(-22.0, 24.0, 0.0)
        .elbow_r(30.0)
        .wrists(-18.0, 0.0, 0.0)
        .plant_l([L, GROUND, LEAD_Z - 0.02])
        .plant_r([R, GROUND, REAR_Z])
        .toe_l(-6.0)
        .toe_r(16.0);

    // The knees go. Not a lowering -- a drop, with the torso pitching forward
    // as the hips fall out from under it. The feet have not moved, so the legs
    // are solved for a body that has come down onto them.
    let buckle = stance()
        .hips(0.01, -0.175, -0.010)
        .root(18.0, 3.0, -6.0)
        .spine(13.0, 3.0, 8.0)
        .chest(6.0, 4.0, 10.0)
        .head(-4.0, -6.0, 12.0)
        .shoulder_l(-4.0, 22.0, 0.0)
        .elbow_l(22.0)
        .shoulder_r(-10.0, 20.0, 0.0)
        .elbow_r(26.0)
        .plant_l([L - 0.01, GROUND, LEAD_Z - 0.06])
        .plant_r([R + 0.01, GROUND, REAR_Z - 0.02])
        .toe_l(2.0)
        .toe_r(20.0);

    // Half way down, and the only reason this key exists: a pose is stored as
    // angles, so between two keys the feet go wherever the angles happen to
    // send them. Across a forty-centimetre drop that is a foot through the
    // floor for six frames, and the fix is to solve the legs again on the way.
    let dropping = buckle
        .hips(0.015, -0.330, -0.020)
        .root(22.0, 3.0, -6.0)
        .spine(15.0, 4.0, 8.0)
        .chest(7.0, 4.0, 10.0)
        .head(-1.0, -6.0, 11.0)
        .plant_l([L - 0.02, GROUND, LEAD_Z - 0.14])
        .plant_r([R + 0.02, GROUND, REAR_Z - 0.06])
        .toe_l(10.0)
        .toe_r(26.0);

    // Down on the knees: the last frame anybody could mistake for a character
    // who is going to get up. The ankles are placed behind the body rather than
    // the knee keyed to a number, because a kneel is entirely a question of
    // where the shins ended up and the solver knows that better than I do.
    let kneeling = Pose::rest()
        .hips(0.02, -0.520, -0.030)
        .root(24.0, 4.0, -6.0)
        .spine(16.0, 4.0, 8.0)
        .chest(8.0, 4.0, 10.0)
        .head(2.0, -5.0, 10.0)
        .shoulder_l(2.0, 16.0, 0.0)
        .elbow_l(26.0)
        .shoulder_r(-4.0, 14.0, 0.0)
        .elbow_r(30.0)
        .wrists(-12.0, 0.0, 0.0)
        .plant_l([-0.17, GROUND + 0.05, -0.44])
        .plant_r([0.17, GROUND + 0.04, -0.46])
        .toe_l(34.0)
        .toe_r(36.0);

    // Folding: the hands go out to the floor, because that is what a body does
    // on the way down whether it means to or not.
    let folding = kneeling
        .hips(0.03, -0.570, 0.030)
        .root(42.0, 6.0, -4.0)
        .spine(30.0, 5.0, 6.0)
        .chest(15.0, 5.0, 8.0)
        .head(14.0, -4.0, 8.0)
        .plant_l([-0.18, GROUND + 0.05, -0.46])
        .plant_r([0.18, GROUND + 0.04, -0.48])
        .toe_l(34.0)
        .toe_r(36.0)
        .reach_l([-0.27, GROUND + 0.05, 0.40])
        .reach_r([0.27, GROUND + 0.04, 0.38])
        .wrists(24.0, 0.0, 0.0);

    // The elbows go. Same reason as `dropping`: the chest travels thirty
    // centimetres here and the hands are supposed to stay on the floor while it
    // does, which only happens if they are solved again part way.
    let giving = folding
        .hips(0.04, -0.680, 0.030)
        .root(46.0, 10.0, 0.0)
        .spine(36.0, 8.0, 0.0)
        .chest(19.0, 7.0, -2.0)
        .head(10.0, 4.0, -6.0)
        .plant_l([-0.19, GROUND + 0.02, -0.62])
        .plant_r([0.19, GROUND + 0.02, -0.60])
        .toe_l(28.0)
        .toe_r(30.0)
        .reach_l([-0.31, GROUND + 0.02, 0.36])
        .reach_r([0.29, GROUND + 0.02, 0.30])
        .wrists(18.0, 0.0, 0.0);

    // Down. The legs slide out straight behind, the arms sprawl where they
    // landed, and the head is the last thing to reach the floor.
    let collapse = folding
        .hips(0.05, -0.790, 0.020)
        .root(46.0, 15.0, 4.0)
        .spine(34.0, 12.0, -6.0)
        .chest(16.0, 9.0, -8.0)
        .head(-6.0, 14.0, -14.0)
        .plant_l([-0.20, GROUND, -0.84])
        .plant_r([0.21, GROUND, -0.80])
        .toe_l(14.0)
        .toe_r(16.0)
        .reach_l([-0.36, GROUND, 0.30])
        .reach_r([0.30, GROUND, 0.18])
        .wrists(10.0, 0.0, 0.0);

    // Settled, and that is all. Almost nothing changes here on purpose: the
    // arms are the loosest thing in a LIMP body and they ring for a good ten
    // frames, so the last real movement has to happen early enough that what
    // is left at the end is a body which has stopped rather than one still
    // swinging.
    let still = collapse
        .hips(0.055, -0.805, 0.015)
        .root(47.0, 16.0, 5.0)
        .spine(35.0, 13.0, -7.0)
        .chest(17.0, 10.0, -9.0)
        .head(-9.0, 16.0, -17.0)
        .plant_l([-0.20, GROUND, -0.85])
        .plant_r([0.21, GROUND, -0.81])
        .toe_l(12.0)
        .toe_r(14.0)
        .reach_l([-0.37, GROUND, 0.28])
        .reach_r([0.30, GROUND, 0.16])
        .wrists(8.0, 0.0, 0.0);

    Recipe {
        clip: Clip::Defeat,
        looseness: Looseness::LIMP,
        notes: "The collapse, clocked from the round-over pause so it starts on \
                the frame the round is decided. It runs the other way round from \
                the stun clips -- forwards from the hit, and it holds its last \
                pose rather than recovering, because nothing is coming after it. \
                The legs fail first and the head last: a body that folds from \
                the top reads as lying down on purpose. Every contact with the \
                floor is placed by IK rather than keyed as an angle, and the two \
                extra keys in the middle of the drops are there because \
                interpolating angles between two solved poses does not keep a \
                foot where either of them put it. The eases are the argument for \
                the rest: IN into each drop because a falling body accelerates, \
                OUT onto the knees and onto the floor because an arriving one \
                does not bounce, and the last movement lands early enough that \
                the springs are done ringing before the clip ends -- a corpse \
                whose hands are still swinging is worse than no animation."
            .into(),
        keys: vec![
            Key::eased(0, struck, Ease::STRIKE),
            Key::eased(6, buckle, Ease::IN),
            Key::eased(12, dropping, Ease::IN),
            Key::eased(18, kneeling, Ease::OUT),
            Key::eased(24, folding, Ease::IN),
            Key::eased(30, giving, Ease::IN),
            Key::eased(36, collapse, Ease::OUT),
            Key::eased(44, still, Ease::OUT),
        ],
    }
}

#[cfg(test)]
mod scratch {
    #[test]
    fn dump() {
        use view::skeleton::Joint::*;
        let r = super::clips();
        for name in ["defeat"] {
            let rec = r.iter().find(|r| r.clip.name() == name).unwrap();
            let baked = crate::bake::bake(rec);
            let sk = view::pose::reference();
            for (f, pose) in baked.frames.iter().enumerate() {
                if f % 4 != 0 && f + 1 != baked.frames.len() {
                    continue;
                }
                let skin = view::skeleton::solve(sk, pose);
                let y = |j: view::skeleton::Joint| skin.origin[j.index()][1];
                let z = |j: view::skeleton::Joint| skin.origin[j.index()][2];
                println!(
                    "{name} {f:3}  hip {:+.2}  knee.r {:+.2}  ankle.r {:+.2}  head {:+.2}/{:+.2}  hand.r {:+.2}  sole {:+.3}",
                    y(Root),
                    y(ShinR),
                    y(FootR),
                    y(Head),
                    z(Head),
                    y(HandR),
                    pose.lowest_foot(sk)
                );
            }
        }
    }
}
