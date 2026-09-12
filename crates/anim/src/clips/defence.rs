//! Guard, parry, and the reaction to a blocked hit.
//!
//! ## Guard is an arc, so the pose has to be side-on
//!
//! `resolve_hit` only counts a guard if the attacker is inside the defender's
//! facing arc -- `guard_arc_cos()` is a 120-degree wedge, not a bubble -- and
//! the price of holding it is that you turn slowly. So the single thing this
//! family has to communicate is **which way the guard points**: a fighter
//! bladed onto one shoulder, presenting the narrow side of the body and one
//! forearm across the line the attack has to come down.
//!
//! A symmetric hands-up stance would be a lie about the mechanic. It would read
//! as "protected", when what the rules actually say is "protected *from the
//! front*", and the opponent's whole job is to walk around it.
//!
//! The blade is the left side, because the locomotion stance already stands
//! with the left foot forward. Raising a guard is then a turn and a weight
//! change rather than a change of footing.
//!
//! ## The feet do not move
//!
//! Every pose here stands on the ankle positions taken straight off
//! `locomotion::stance`, read out of the pose rather than typed again so the
//! two cannot drift apart. Two reasons:
//!
//! - The cut in and out of guard happens on a single frame, with no blend, so
//!   any foot that is somewhere else on the first frame of `guard_in` is a
//!   visible pop.
//! - A blocked hit already moves the whole character backwards -- pushback is
//!   the cost of blocking, and it is applied to the body, not to the animation.
//!   A recovery step keyed on top of that fights it, and the feet skid twice.
//!
//! So everything here is expressed in the hips, the legs and the arms over a
//! fixed base, which is also what stops any of it from skating.
//!
//! ## What plays when
//!
//! `play::guard` cuts straight into `guard_in`, holds `guard_idle` for as long
//! as the button is down, and hands over to `parry` for the fourteen frames
//! after a guard catches something. Two of those are indexed unusually and the
//! clips are authored for it:
//!
//! - **The parry window is the first four frames of `guard_in`.** Not a
//!   separate clip, not a separate input -- `held < parry_window()`. Those four
//!   frames are the hardest input in the game, so the guard is thrown up inside
//!   them and the last four only settle it. Nothing about the opening may read
//!   as a drift into a pose. Four is what the Oven says today rather than
//!   something this file can read: if that number moves, the throw has to move
//!   with it, or the clip stops teaching the timing it is there to teach.
//! - **`block_stun` is indexed from the end.** A stun longer than the clip
//!   holds frame zero, so frame zero is the impact and the last frame is back
//!   on balance, on the guard pose the player is still holding.
//!
//! `guard_out` has no caller yet -- releasing the button drops straight back
//! into locomotion -- so it is authored to land exactly on the locomotion
//! stance, which is what whatever plays it will be blending toward.

use crate::bake::{Key, Looseness, Recipe};
use crate::clips::locomotion::stance;
use crate::ease::Ease;
use view::clips::Clip;
use view::pose::Pose;
use view::skeleton::Joint;

pub fn clips() -> Vec<Recipe> {
    vec![guard_in(), guard_idle(), guard_out(), parry(), block_stun()]
}

// ---------------------------------------------------------------------------
// The stance
// ---------------------------------------------------------------------------

/// Ankle to the back of the sole, and ankle to the front of it, in metres on
/// the reference body.
const HEEL: f32 = 0.06;
const TOE: f32 = 0.16;

/// How far a foot's ankle rises when the foot pivots on one end of the sole.
///
/// Rocking back onto the heel is a rotation about the heel, so the ankle
/// **goes up**. Tip a foot without raising it and the pivot happens about the
/// ankle instead, which drives the heel two centimetres into the floor -- and
/// two centimetres is the difference between a fighter rocked onto their heels
/// and a fighter standing in the ground.
fn pivot_lift(toe: f32) -> f32 {
    let lever = if toe < 0.0 { HEEL } else { TOE };
    lever * toe.abs().to_radians().sin()
}

/// Put the feet back where the fighter is standing, and say what each one is
/// doing with its heel.
///
/// Called at the end of every pose in this file. A foot target is measured from
/// the body, so a pose that drops the hips two centimetres without re-planting
/// has just dragged both feet two centimetres across the floor -- and in a clip
/// held as long as `guard_idle` that is a slow permanent slide rather than a
/// glitch anybody would spot.
///
/// The ankle positions come out of `locomotion::stance` rather than being typed
/// again, so the cut into guard and back out of it cannot move them.
fn planted(pose: Pose, lead_toe: f32, rear_toe: f32) -> Pose {
    let ground = stance();
    let at = |j: Joint, toe: f32| {
        let p = ground.joint_at(j);
        [p[0], p[1] + pivot_lift(toe), p[2]]
    };
    pose.plant_l(at(Joint::FootL, lead_toe))
        .plant_r(at(Joint::FootR, rear_toe))
        .toe_l(lead_toe)
        .toe_r(rear_toe)
}

/// The held guard: bladed left-shoulder-on, chin behind the lead forearm, hips
/// low and back over a braced rear leg.
///
/// Forty-six degrees of blade, split twenty at the pelvis and thirteen at each
/// of the two torso members. The split is not taste. The feet are nailed down,
/// so every degree of it at the pelvis is a degree the legs have to absorb, and
/// a pelvis that turns faster than the legs can answer drags the ankles down
/// with it and puts the toes through the floor -- which is what the first
/// version of `guard_in` did. Turning the torso instead costs the legs nothing.
///
/// The head then gives back nearly all of it. Whatever the body is doing, a
/// guard is looking at the person it is guarding against.
fn guard() -> Pose {
    planted(
        stance()
            .hips(0.010, -0.078, -0.018)
            .root(4.0, 0.0, 20.0)
            .spine(8.0, -2.0, 13.0)
            .chest(4.0, 0.0, 13.0)
            .head(-7.0, 3.0, -40.0)
            .shoulder_l(30.0, 16.0, -10.0)
            .elbow_l(116.0)
            .wrist_l(-14.0, 0.0, 0.0)
            .shoulder_r(12.0, 11.0, -32.0)
            .elbow_r(130.0)
            .wrist_r(-10.0, 0.0, 8.0),
        0.0,
        0.0,
    )
}

// ---------------------------------------------------------------------------
// Coming up to guard
// ---------------------------------------------------------------------------

/// Eight frames, and the first four of them parry.
fn guard_in() -> Recipe {
    // Past the guard, not at it: the arms drive further forward and the body
    // further round than either ends up. A stance that arrives exactly on its
    // mark and stops reads as a pose being assumed; one that arrives past it
    // and settles reads as a thing the fighter did.
    let thrown = planted(
        stance()
            // The hands lead and the weight follows. Dropping the hips in the
            // same three frames the arms travel in also drops the feet: the
            // legs are solved for where the hips were, and a body that sinks
            // faster than its own legs can answer puts its toes through the
            // floor. So the guard is up first and the fighter sits down into it
            // over the back half of the clip.
            .hips(0.014, -0.062, -0.026)
            .root(6.0, 0.0, 22.0)
            .spine(11.0, -2.0, 15.0)
            .chest(6.0, 0.0, 14.0)
            .head(-4.0, 3.0, -40.0)
            .shoulder_l(37.0, 18.0, -12.0)
            .elbow_l(108.0)
            .wrist_l(-16.0, 0.0, 0.0)
            .shoulder_r(17.0, 13.0, -34.0)
            .elbow_r(124.0)
            .wrist_r(-12.0, 0.0, 8.0),
        -2.0,
        2.0,
    );

    Recipe {
        clip: Clip::GuardIn,
        looseness: Looseness::CRISP,
        notes: "The parry window is frames zero to three, so the guard is up by \
                frame three and the back half only settles it. The ease out of \
                frame zero is the whole clip: it leaves at full speed and \
                decelerates, because a guard that eases into its own first frame \
                is a guard a player cannot time a parry against. Frame zero is \
                the locomotion stance on its own footing -- the entry is a hard \
                cut with no blend, so anything standing anywhere else pops -- \
                except that the back heel comes down flat, which is the one \
                thing raising a guard actually does to the feet."
            .into(),
        keys: vec![
            Key::eased(0, planted(stance(), 0.0, 2.0), Ease::OUT),
            Key::eased(3, thrown, Ease::SMOOTH),
            Key::eased(7, guard(), Ease::SMOOTH),
        ],
    }
}

// ---------------------------------------------------------------------------
// Holding it
// ---------------------------------------------------------------------------

/// Ninety-six frames of a fighter not doing anything, which a player may be
/// looking at for ten seconds at a stretch.
fn guard_idle() -> Recipe {
    // A breath, on its own period. The chest opens, the shoulders widen by two
    // degrees and the hips rise a centimetre: enough to be alive, not enough to
    // look at.
    let breath = planted(
        guard()
            .hips(0.010, -0.068, -0.020)
            .spine(6.0, -2.0, 13.0)
            .chest(2.0, 0.0, 13.0)
            .head(-9.0, 3.0, -40.0)
            .shoulder_l(30.0, 18.5, -10.0)
            .shoulder_r(12.0, 13.5, -32.0),
        0.0,
        0.0,
    );

    // The weight drifts onto the braced rear leg and comes back, on a longer
    // period. The hips move about a centimetre and a half; any more and a held
    // guard starts to sway.
    let onto_rear = planted(
        guard()
            .hips(0.024, -0.082, -0.028)
            .root(3.0, 2.5, 20.0)
            .spine(9.0, -4.0, 13.0)
            .head(-6.0, 5.0, -42.0),
        -2.0,
        1.0,
    );

    let onto_lead = planted(
        guard()
            .hips(-0.006, -0.073, -0.008)
            .root(6.0, -2.0, 20.0)
            .spine(7.0, 0.0, 13.0)
            .head(-8.0, 1.0, -38.0),
        1.0,
        -2.0,
    );

    // The hands re-assert themselves: the lead forearm nudges back out along
    // the line it is covering and the rear elbow re-tucks. This is most of what
    // separates a guard being held from a guard left lying there, and it is
    // three degrees.
    let reset = planted(
        breath
            .shoulder_l(33.0, 15.0, -8.0)
            .elbow_l(111.0)
            .shoulder_r(10.0, 10.0, -34.0)
            .elbow_r(134.0)
            .head(-8.0, 2.0, -41.0),
        0.0,
        0.0,
    );

    Recipe {
        clip: Clip::GuardIdle,
        looseness: Looseness::MARTIAL,
        notes: "Held for as long as the button is down, which can be most of a \
                round, so a statue reads as a bug. Three things move -- a breath, \
                the weight drifting between the two legs, and the hands \
                re-asserting themselves -- and the keys are spaced so that no two \
                of them peak together twice in a loop, because a body that \
                repeats reads as a machine even when every individual movement is \
                right. None of it is more than a centimetre and a half or a few \
                degrees. The guard itself never opens: whatever else drifts, the \
                lead forearm stays on the line an attack has to come down."
            .into(),
        keys: vec![
            Key::eased(0, guard(), Ease::SMOOTH),
            Key::eased(21, breath, Ease::IN),
            Key::eased(39, onto_rear, Ease::SMOOTH),
            Key::eased(58, reset, Ease::OUT),
            Key::eased(77, onto_lead, Ease::SMOOTH),
        ],
    }
}

// ---------------------------------------------------------------------------
// Dropping it
// ---------------------------------------------------------------------------

/// The way back to neutral, and deliberately not `guard_in` played backwards.
fn guard_out() -> Recipe {
    // Halfway out: the arms have dropped and the shoulders have come square
    // ahead of the hips, which are still turned. Unwinding from the top down is
    // what makes this read as letting go rather than as another decision.
    let dropping = planted(
        stance()
            .hips(0.006, -0.068, -0.012)
            .root(3.0, 0.0, 8.0)
            .spine(7.0, -1.0, -4.0)
            .chest(4.0, 0.0, -4.0)
            .head(-6.0, 2.0, -14.0)
            .shoulder_l(22.0, 15.0, -4.0)
            .elbow_l(84.0)
            .wrist_l(-10.0, 0.0, 0.0)
            .shoulder_r(9.0, 14.0, -16.0)
            .elbow_r(88.0)
            .wrist_r(-8.0, 0.0, 4.0),
        0.0,
        4.0,
    );

    Recipe {
        clip: Clip::GuardOut,
        looseness: Looseness::MARTIAL,
        notes: "Not guard_in reversed. Coming up is a decision and leaves at \
                full speed; letting go is a release, so this gathers for a frame \
                and then goes, and arrives on the locomotion stance settling \
                rather than snapping. The shoulders unwind before the hips do, \
                which is the order a body actually gives up a blade in."
            .into(),
        keys: vec![
            Key::eased(0, guard(), Ease::IN),
            Key::eased(4, dropping, Ease::OUT),
            Key::eased(7, planted(stance(), 0.0, 8.0), Ease::SMOOTH),
        ],
    }
}

// ---------------------------------------------------------------------------
// The guard that caught something
// ---------------------------------------------------------------------------

/// Fourteen frames the defender has earned, and the only feedback they get for
/// the hardest input in the game.
fn parry() -> Recipe {
    // Frame zero is the contact. The attack is already there, so there is no
    // wind-up to play: the guard is compressed by what it just stopped.
    let caught = planted(
        guard()
            .hips(0.004, -0.092, -0.032)
            .root(7.0, 0.0, 23.0)
            .spine(11.0, -2.0, 15.0)
            .chest(7.0, 0.0, 15.0)
            .head(-3.0, 3.0, -42.0)
            .shoulder_l(24.0, 11.0, -16.0)
            .elbow_l(128.0)
            .shoulder_r(9.0, 9.0, -34.0)
            .elbow_r(138.0),
        -4.0,
        2.0,
    );

    // The deflection: the blade unwinds hard and the lead forearm sweeps off
    // the line, taking the attack with it. The body ends nearly square, which
    // is also where the punish comes from -- the stagger the attacker is now in
    // is long, and the defender should already be facing them to use it.
    let deflect = planted(
        guard()
            .hips(-0.012, -0.072, -0.008)
            .root(1.0, -3.0, -4.0)
            .spine(4.0, 3.0, -6.0)
            .chest(0.0, 0.0, -8.0)
            .head(-4.0, -5.0, 6.0)
            // The sweep is spread and roll, not swing: the forearm stays folded
            // and goes across. An arm that straightens out in front is a punch,
            // and a parry that reads as a punch teaches the wrong thing about
            // what just happened.
            .shoulder_l(33.0, 50.0, 26.0)
            .elbow_l(88.0)
            .wrist_l(-24.0, 0.0, 0.0)
            .shoulder_r(4.0, 16.0, -24.0)
            .elbow_r(112.0),
        3.0,
        1.0,
    );

    // The arm carries a little past before it comes back, and the hips have
    // already started to re-blade underneath it.
    let carry = planted(
        guard()
            .hips(-0.008, -0.076, -0.012)
            .root(3.0, -2.0, 8.0)
            .spine(6.0, 2.0, -1.0)
            .chest(2.0, 0.0, -2.0)
            .head(-6.0, -2.0, -4.0)
            .shoulder_l(29.0, 40.0, 16.0)
            .elbow_l(98.0)
            .wrist_l(-18.0, 0.0, 0.0)
            .shoulder_r(6.0, 14.0, -28.0)
            .elbow_r(118.0),
        2.0,
        0.0,
    );

    // Squared up and a touch forward, which is the attitude the punish comes
    // out of: the attacker is in a stagger and the defender is already looking
    // at them.
    let ready = planted(
        guard()
            .hips(0.000, -0.076, -0.006)
            .root(6.0, 0.0, 14.0)
            .spine(7.0, 0.0, 9.0)
            .chest(4.0, 0.0, 9.0)
            .head(-8.0, 0.0, -28.0)
            .shoulder_l(32.0, 24.0, -2.0)
            .elbow_l(108.0)
            .wrist_l(-14.0, 0.0, 0.0)
            .shoulder_r(10.0, 12.0, -30.0)
            .elbow_r(124.0),
        1.0,
        0.0,
    );

    Recipe {
        clip: Clip::Parry,
        looseness: Looseness::CRISP,
        notes: "A deflection, not a flinch. Frame zero is the contact itself -- \
                the attack has already arrived, so there is nothing to wind up \
                -- and the two frames out of it are the sharpest thing in the \
                family, because a parry costs the attacker a long stagger and \
                the defender has to be able to see that they earned it. The \
                extreme is not held by an ease: the arms are the loosest part of \
                a crisp body, so they arrive a frame or so late and linger there \
                on their own, which is a settle rather than a freeze. It ends on \
                the guard pose, because that is what playback hands back to."
            .into(),
        keys: vec![
            Key::eased(0, caught, Ease::OUT),
            Key::eased(2, deflect, Ease::SMOOTH),
            Key::eased(5, carry, Ease::OUT),
            Key::eased(9, ready, Ease::SMOOTH),
            Key::eased(13, guard(), Ease::SMOOTH),
        ],
    }
}

// ---------------------------------------------------------------------------
// Taking one on the guard
// ---------------------------------------------------------------------------

/// Fourteen frames, played backwards from the end of the stun.
fn block_stun() -> Recipe {
    // Frame zero: the impact, and the pose a long blockstun sits on. The hips
    // are thirteen centimetres behind where they stand, both feet have rocked
    // back onto their heels, and the guard is folded shut against the chest.
    // Nothing opens. There is no chip damage in this game, which means the
    // guard held, and a guard that flies apart says otherwise -- and the head
    // stays turned onto the attacker, because blockstun does not stop the arc
    // pointing at them.
    let shoved = planted(
        guard()
            .hips(0.034, -0.112, -0.130)
            .root(-16.0, 0.0, 18.0)
            .spine(16.0, -4.0, 14.0)
            .chest(10.0, 0.0, 13.0)
            .head(-6.0, 5.0, -46.0)
            .shoulder_l(4.0, 6.0, -20.0)
            .elbow_l(146.0)
            .wrist_l(-26.0, 0.0, 0.0)
            .shoulder_r(-2.0, 6.0, -38.0)
            .elbow_r(148.0)
            .wrist_r(-16.0, 0.0, 8.0),
        -20.0,
        -6.0,
    );

    // Still going backwards, but the rear leg has taken it and the body is
    // about to stop.
    let absorbed = planted(
        guard()
            .hips(0.028, -0.102, -0.078)
            .root(-7.0, 0.0, 19.0)
            .spine(13.0, -3.0, 14.0)
            .chest(7.0, 0.0, 13.0)
            .head(-4.0, 4.0, -44.0)
            .shoulder_l(16.0, 10.0, -16.0)
            .elbow_l(130.0)
            .wrist_l(-20.0, 0.0, 0.0)
            .shoulder_r(5.0, 8.0, -36.0)
            .elbow_r(140.0)
            .wrist_r(-12.0, 0.0, 8.0),
        -11.0,
        -2.0,
    );

    // Pushing back out of it: the hips come forward over the base and the lead
    // forearm goes back out to where it covers the line.
    let recovering = planted(
        guard()
            .hips(0.012, -0.074, -0.008)
            .root(7.0, 0.0, 21.0)
            .spine(7.0, -2.0, 14.0)
            .chest(3.0, 0.0, 13.0)
            .head(-9.0, 3.0, -39.0)
            .shoulder_l(33.0, 18.0, -8.0)
            .elbow_l(110.0)
            .wrist_l(-12.0, 0.0, 0.0)
            .shoulder_r(14.0, 12.0, -31.0)
            .elbow_r(126.0),
        2.0,
        1.0,
    );

    Recipe {
        clip: Clip::BlockStun,
        looseness: Looseness::STRIDE,
        notes: "Indexed from the end, so frame zero is the impact and a stun \
                longer than fourteen frames holds it -- which is why the shove \
                is the first thing in the clip rather than something it builds \
                to. The lead foot is the read: it rocks back onto its heel with \
                the toe eight centimetres off the floor, because being moved \
                backwards is what blocking costs and the feet are where a player \
                sees it. The looseness is the walking one and not the heavy one, \
                which looks like the wrong call for a body being shoved and is \
                not: heavy legs lag far enough behind hips travelling this fast \
                to lift the back foot five centimetres clear of the ground. The \
                weight lives in the shape instead, and in the head and arms \
                carrying past the body."
            .into(),
        keys: vec![
            Key::eased(0, shoved, Ease::IN),
            Key::eased(5, absorbed, Ease::OUT),
            Key::eased(9, recovering, Ease::SMOOTH),
            Key::eased(13, guard(), Ease::SMOOTH),
        ],
    }
}
