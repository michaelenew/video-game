//! The Dual mage: two autos, two Lances, Judgement, Sweep.
//!
//! A melee mage holding two forces apart. The body is the longest and the
//! slightest in the roster -- `build_for(DualMage)` is tall, thin and
//! long-limbed -- so nothing here is powered by mass. What this class has
//! instead is **reach and line**: a limb that arrives from further away than
//! you thought, and a pose that is legible because it is long rather than
//! because it is big. Every one of these clips is authored to be read from the
//! silhouette's extremities, not from its bulk, because there is no bulk.
//!
//! That gives five uses of one vocabulary:
//!
//! - **The two autos** are one punch thrown with either arm: left is dark,
//!   right is light, and which of the two just landed is how the whole class
//!   steers. It is a step *into* the strike -- the lead foot leaves the floor
//!   on the frame the hand starts, so the footwork is the attack rather than a
//!   preface to it -- and it is the only thing here that keeps any movement,
//!   sixty per cent of walking speed, so it is authored knowing its legs may be
//!   half-replaced by a walk cycle while it plays.
//!
//!   **The light one is the dark one through `Pose::other_arm`.** Not a
//!   convenience: the pair only works if the *only* difference a player can see
//!   is which arm it came out of, and two hand-authored clips would not stay
//!   that way. The stance does not mirror with it -- the feet stay where the
//!   idle put them, or the first frame of every right-hand punch would swap the
//!   character's footing.
//!
//! - **Sweep** is the one thing here thrown with both arms at once, and the one
//!   that goes round **past both shoulders**. It is on `E`, and it is the answer
//!   to somebody already inside the punches' arc -- which means it has to reach
//!   a little behind her, because that is where they are.
//! - **The light Lance** is one straight thing: rear foot, hips, shoulder,
//!   point. It coils *away* from that line first -- the point hand goes back
//!   past the hip while the free hand stays out on the target -- so what an
//!   opponent reads during the startup is the opposite of what arrives.
//! - **The dark Lance** is the same input and nothing like the same move, which
//!   is the whole job of the pair: middle click throws one of the two and the
//!   force in her arms picks which, so the only thing the person opposite has
//!   to go on is the wind-up. So one rises off the right shoulder and goes
//!   early and forward, and the other sinks onto the rear leg, drags the left
//!   hand down past the hip, and arrives holding rather than striking.
//! - **Judgement** squares up. The rest are bladed and one-sided; the finisher
//!   is symmetric front to back and side to side, because it is the only thing
//!   the class does with both forces *together*, and because symmetry is what
//!   makes it read as a verdict rather than as a large swing.
//!
//! ## Two rules every clip in this file obeys
//!
//! **The keys come from `clip.phases()`, not from typed-in frames.** The
//! contact pose sits on the first active frame because that is the frame the
//! hitbox appears on; everything else is a fraction of the startup or of the
//! recovery. Retuning any of these in the Oven moves the animation with them,
//! rather than leaving a strike that lands two frames after the hand did.
//!
//! **Every clip starts and ends on the idle's own stance.** There is no
//! cross-fade between an attack and what follows it: `play::pose_for` returns
//! one clip's frame and the renderer draws it, so on the frame a move ends the
//! idle takes over from wherever this left the body. Ending anywhere else is a
//! pop, and a pop on the last frame of every attack is the most visible thing
//! in the game.

use crate::bake::{Feel, Key, Looseness, Recipe};
use crate::clips::locomotion::stance;
use crate::ease::Ease;
use view::clips::Clip;
use view::pose::{ANKLE_ON_GROUND as GROUND, Pose};

/// Where the idle leaves the feet: left lead, right back. Taken from the
/// stance rather than invented again here, because the first and last frame of
/// every clip in this file has to agree with it exactly.
const L: f32 = -0.145;
const R: f32 = 0.145;

/// A hold that lets go gently: nothing for the first half of the gap, then a
/// gather rather than a snap.
///
/// `IN` is the right shape for a coil and the wrong slope for one this short.
/// Three frames of `IN` put nine tenths of a wind-up into the last of them,
/// which arrives at the next key with the arm still travelling -- and then the
/// contact pose lands on top of that. This keeps the pause an opponent reads
/// the coil in and spends the rest of the gap moving.
const GATHER: Ease = Ease::new(0.45, 0.1, 0.75, 0.82);

/// Half an `ANTICIPATE`: the same pull away from the target before going, over
/// a drop long enough that the full one would spike.
///
/// `ANTICIPATE` spends the first half of a gap moving backwards, which means
/// the second half has to cover a third more ground than the gap is worth. On
/// the Judgement's descent -- a metre and three quarters of hand travel, the
/// biggest thing in this file -- that is the difference between twenty-one
/// metres a second and twenty-eight. This keeps the pull-back and halves it.
const PULL_AWAY: Ease = Ease::new(0.45, -0.22, 0.3, 1.0);

pub fn clips() -> Vec<Recipe> {
    vec![
        punch(Clip::DualDark, Arm::Dark),
        light_lance(),
        judgement(),
        sweep(),
        punch(Clip::DualLight, Arm::Light),
        dark_lance(),
        float(),
        drift(),
    ]
}

// ---------------------------------------------------------------------------
// Off the floor
// ---------------------------------------------------------------------------
//
// **The one class in the game whose locomotion has two states.** With the
// lower of her two bars at three quarters, or ascended, her feet leave the
// ground -- `sim::state::floating` -- and these two replace the idle and the
// walk for as long as that holds. It is the same tier the second jump and the
// slow fall arrive on, which is why the three share one threshold.
//
// The reference is a renaissance angel rather than a superhero: **the body
// hangs**. Feet pointed and together-ish, knees softly bent rather than
// straight, hips carried a little high and a little back so the legs trail, the
// spine long, the arms low and open with the palms turned out. Nothing is
// braced against anything, because there is nothing to brace against -- which
// is also why these are the only two clips in this file on `FLOATY` rather than
// on a martial looseness.
//
// The wings are not here. They are geometry rather than bones -- a pool of
// pieces the renderer hangs off the chest and colours by the force she is
// carrying, `game::main` -- for the same reason the Reaver's blades are: a
// skeleton has sixteen joints and none of them are a wing.

/// The hanging body both clips are built on.
///
/// Authored once so the idle and the travelling version cannot drift apart at
/// the join: `play::grounded` cross-fades between them the moment she starts
/// moving, and two separately-authored hangs would pop through each other.
fn hanging() -> Pose {
    Pose::rest()
        // Up, and the toes are what is nearest the floor. A bent knee shortens
        // the leg by more than a pointed foot lengthens it, so this lands the
        // feet at about the height they would be standing -- she reads as
        // hovering rather than as sunk into the ground or stilted above it.
        .hips(0.0, 0.035, -0.02)
        .root(-3.0, 0.0, 0.0)
        .spine(-4.0, 0.0, 0.0)
        .chest(-3.0, 0.0, 0.0)
        .head(4.0, 0.0, 0.0)
        // Low and open, palms out. The idle's guard is up and bladed; this is
        // the opposite reading, and it has to be, because the point of the
        // silhouette is that she has stopped standing like a fighter.
        .shoulder_l(-8.0, 26.0, -14.0)
        .elbow_l(16.0)
        .wrist_l(-14.0, 0.0, 0.0)
        .shoulder_r(-8.0, 26.0, 14.0)
        .elbow_r(16.0)
        .wrist_r(-14.0, 0.0, 0.0)
        // Trailing, not standing. Both legs behind the hips, knees soft, toes
        // dropped hard -- the pointed foot is most of what says "not walking".
        .hips_both(-14.0, 3.0, 0.0)
        .knees(22.0)
        .ankles(42.0, 0.0, 0.0)
}

/// Holding still, off the floor.
///
/// Two clocks against each other, the way the idle does it: a long rise and
/// fall of the whole body, and a slower sway across it. A hover on one period
/// reads as a machine, and this is the pose a player will look at for whole
/// seconds at a time -- it is the reward, so it is the thing they are looking
/// at when they get it.
fn float() -> Recipe {
    let up = hanging()
        .hips(0.0, 0.075, -0.02)
        .head(6.0, 0.0, 0.0)
        .shoulder_l(-12.0, 29.0, -14.0)
        .shoulder_r(-12.0, 29.0, 14.0)
        .knees(17.0)
        .ankles(46.0, 0.0, 0.0);

    let sway_left = hanging()
        .hips(-0.028, 0.05, -0.02)
        .root(-3.0, -4.0, -6.0)
        .spine(-4.0, 3.0, 0.0)
        .head(4.0, 2.0, 5.0)
        .hip_l(-11.0, 4.0, 0.0)
        .hip_r(-18.0, 2.0, 0.0);

    let sway_right = hanging()
        .hips(0.028, 0.05, -0.02)
        .root(-3.0, 4.0, 6.0)
        .spine(-4.0, -3.0, 0.0)
        .head(4.0, -2.0, -5.0)
        .hip_l(-18.0, 2.0, 0.0)
        .hip_r(-11.0, 4.0, 0.0);

    Recipe {
        clip: Clip::DualFloat,
        looseness: Looseness::FLOATY,
        notes: "Deep or ascended and holding still. A rise and fall every \
                seventy frames against a sway every hundred and forty, so the \
                loop never lands in the same place twice -- the idle's trick, \
                for the same reason. Feet pointed throughout: the moment a \
                heel drops she is standing again."
            .into(),
        keys: vec![
            Key::eased(0, hanging(), Ease::SMOOTH),
            Key::eased(35, up, Ease::SMOOTH),
            Key::eased(70, sway_left, Ease::SMOOTH),
            Key::eased(105, up.blend(&sway_right, 0.5), Ease::SMOOTH),
        ],
    }
}

/// Travelling, off the floor.
///
/// **No cycle in the legs at all**, which is the whole difference from the walk
/// it replaces. A stride is a sequence of contacts and there are no contacts
/// here, so what carries the motion is the torso leading and the legs trailing
/// behind it -- and the only thing that repeats is the body's own slow roll.
///
/// It is sampled by the stride phase like the walk cycles are, which means it
/// speeds up as she does. That is wrong for feet and right for this: the faster
/// she goes the harder the trail, and a phase that came from a clock instead
/// would have the legs swinging at the same rate at every speed.
fn drift() -> Recipe {
    let lead = hanging()
        .hips(0.0, 0.03, -0.06)
        .root(-11.0, 0.0, 0.0)
        .spine(-6.0, 0.0, 0.0)
        .chest(-4.0, 0.0, 0.0)
        .head(9.0, 0.0, 0.0)
        // Arms swept back by the travel. Not thrown back -- trailing.
        .shoulder_l(-22.0, 22.0, -10.0)
        .shoulder_r(-22.0, 22.0, 10.0)
        .hips_both(-24.0, 4.0, 0.0)
        .knees(26.0)
        .ankles(46.0, 0.0, 0.0);

    let roll_left = lead
        .root(-11.0, -5.0, -7.0)
        .spine(-6.0, 3.0, 0.0)
        .hip_l(-20.0, 6.0, 0.0)
        .hip_r(-28.0, 2.0, 0.0)
        .knee_l(21.0)
        .knee_r(30.0);

    let roll_right = lead
        .root(-11.0, 5.0, 7.0)
        .spine(-6.0, -3.0, 0.0)
        .hip_l(-28.0, 2.0, 0.0)
        .hip_r(-20.0, 6.0, 0.0)
        .knee_l(30.0)
        .knee_r(21.0);

    Recipe {
        clip: Clip::DualDrift,
        looseness: Looseness::FLOATY,
        notes: "Travelling, off the floor. The torso leads and everything else \
                trails it; there is no stride, because a stride is a sequence \
                of contacts and nothing here touches the ground. The only \
                cycle is a slow roll, and it runs on the stride phase so it \
                hardens as she picks up speed."
            .into(),
        keys: vec![
            Key::eased(0, lead, Ease::SMOOTH),
            Key::eased(24, roll_left, Ease::SMOOTH),
            Key::eased(48, lead, Ease::SMOOTH),
            Key::eased(72, roll_right, Ease::SMOOTH),
        ],
    }
}

/// Which arm a punch is thrown with.
///
/// The two autos are the same recipe read twice. `Light` puts every key through
/// [`Pose::other_arm`], which mirrors everything above the hips and leaves the
/// feet where they were -- see the module header.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Arm {
    Dark,
    Light,
}

impl Arm {
    fn side(self, pose: Pose) -> Pose {
        match self {
            Arm::Dark => pose,
            Arm::Light => pose.other_arm(),
        }
    }
}

// ---------------------------------------------------------------------------
// The autos -- one punch, either arm
// ---------------------------------------------------------------------------

/// Two frames in, and the lead foot is already off the floor.
///
/// The cock is deliberately shallow. Five frames is not enough time to draw a
/// hand back and then put it somewhere else, and a deep cock buys a silhouette
/// the opponent cannot act on anyway -- so the arm folds instead of swinging
/// back, and the lifted lead knee carries the tell.
fn load() -> Pose {
    stance()
        .hips(-0.012, -0.085, -0.045)
        .root(0.0, -3.0, -13.0)
        .spine(10.0, 1.0, 0.0)
        .chest(3.0, 0.0, -8.0)
        .head(-2.0, 0.0, 16.0)
        .shoulder_l(16.0, 16.0, 0.0)
        .elbow_l(78.0)
        .wrist_l(-14.0, 0.0, 0.0)
        .shoulder_r(12.0, 18.0, 0.0)
        .elbow_r(58.0)
        .plant_l([L, GROUND + 0.06, 0.235])
        .plant_r([R, GROUND, -0.15])
        .toe_l(-10.0)
        .toe_r(10.0)
}

/// The last startup frame: the lead foot is down and the arm is on its way.
///
/// A step strike lands its foot one frame *before* the hand, which is what
/// makes the step read as part of the strike rather than as a hop next to it.
/// Keying it here rather than trusting the gap to the contact frame is also
/// what gets the foot on the floor on time: the legs run a frame behind the
/// keys, so a foot first asked for on the contact frame arrives after it.
fn plant() -> Pose {
    stance()
        .hips(-0.004, -0.098, 0.02)
        .root(4.0, 0.0, -12.0)
        .spine(13.0, 0.0, 4.0)
        .chest(5.0, 0.0, 6.0)
        .head(-1.0, 0.0, 4.0)
        .shoulder_l(56.0, 14.0, 0.0)
        .elbow_l(48.0)
        .wrist_l(-10.0, 0.0, 0.0)
        .shoulder_r(-6.0, 22.0, 0.0)
        .elbow_r(66.0)
        .plant_l([L - 0.01, GROUND + 0.005, 0.36])
        .plant_r([R, GROUND + 0.02, -0.15])
        .toe_l(-12.0)
        .toe_r(14.0)
}

/// The contact frame: the lead foot is down and the fist arrives on it.
///
/// The arm is nearly straight here and stays out through most of the active
/// frames, which matters more on this move than on any other in the file. The
/// hit volume is a section of a torus sweeping round her from behind
/// (`moves::Shape::Wing`), and it finishes *in front of the fist* -- so the
/// fist has to still be out there when it arrives, or the two halves of the
/// move are pointing at different places.
fn strike() -> Pose {
    stance()
        .hips(0.0, -0.10, 0.075)
        .root(6.0, 2.0, -8.0)
        .spine(16.0, 0.0, 12.0)
        .chest(6.0, 0.0, 18.0)
        .head(0.0, 0.0, -16.0)
        // Twenty-eight degrees of that is the torso's own forward pitch, which
        // the shoulder has to pay back before the arm points anywhere: the
        // world angle of a limb is its swing minus however far the chest above
        // it has leaned. Ninety-four here is an arm level with the target, not
        // an arm above it.
        .shoulder_l(94.0, 12.0, 0.0)
        .elbow_l(10.0)
        .wrist_l(-4.0, 0.0, 0.0)
        // The light hand is thrown back as hard as the dark hand goes out.
        // The class is two forces held apart, and this is the cheapest way to
        // say so inside a five-frame move.
        .shoulder_r(-36.0, 26.0, 0.0)
        .elbow_r(72.0)
        .plant_l([L - 0.01, GROUND, 0.395])
        .plant_r([R, GROUND + 0.05, -0.15])
        .toe_l(-6.0)
        .toe_floor_r()
}

/// First recovery frame: the hand is still out, but the body has caught up
/// with it and the rear foot has come off the floor.
fn follow() -> Pose {
    stance()
        .hips(0.0, -0.085, 0.06)
        .root(4.0, 1.0, -10.0)
        .spine(12.0, 0.0, 8.0)
        .chest(4.0, 0.0, 12.0)
        .head(0.0, 0.0, -10.0)
        .shoulder_l(84.0, 14.0, 0.0)
        .elbow_l(26.0)
        .wrist_l(-8.0, 0.0, 0.0)
        .shoulder_r(-22.0, 24.0, 0.0)
        .elbow_r(80.0)
        .plant_l([L - 0.01, GROUND, 0.375])
        .plant_r([R, GROUND + 0.04, -0.105])
        .toe_l(-2.0)
        .toe_r(6.0)
}

/// The rear foot has come up under the body and the guard is rebuilding.
fn regather() -> Pose {
    stance()
        .hips(0.0, -0.07, 0.02)
        .root(3.0, 0.0, -12.0)
        .spine(8.0, 0.0, 6.0)
        .chest(3.0, 0.0, 8.0)
        .head(-2.0, 0.0, -4.0)
        .shoulder_l(30.0, 14.0, 0.0)
        .elbow_l(48.0)
        .wrist_l(-8.0, 0.0, 0.0)
        .shoulder_r(-2.0, 18.0, 0.0)
        .elbow_r(58.0)
        .plant_l([L, GROUND + 0.03, 0.26])
        .plant_r([R, GROUND, -0.12])
        .toe_l(-8.0)
        .toe_r(8.0)
}

/// One punch, thrown with whichever arm is asked for.
///
/// **The two autos are this function twice.** They have to be indistinguishable
/// apart from the side, because that is the entire information the player is
/// reading off them -- dark landed, or light did -- and two hand-authored clips
/// drift the moment either is touched.
///
/// The bookend keys are the idle's own stance, *unmirrored*, on both. Every
/// clip in this file starts and ends there (see the module header), and a
/// mirrored stance is a different stance: put one on frame zero and the
/// character's feet swap sides on the first frame of every right-hand punch and
/// swap back on the last.
fn punch(clip: Clip, arm: Arm) -> Recipe {
    let (windup, contact, recovery) = phases(clip);
    let last = clip.length() - 1;
    let side = |p: Pose| arm.side(p);

    let mut track = Track::new(clip);
    track.key(0, stance(), Ease::OUT);
    // Two frames in, whatever the Oven says the startup is. That is where the
    // opponent is looking, and a fraction of a five-frame startup would put
    // the tell on frame one, where it is a pop rather than a telegraph.
    track.key(windup.saturating_sub(2).max(1), side(load()), Ease::IN);
    track.key(windup, side(plant()), Ease::STRIKE);
    track.contact(contact, side(strike()), Ease::OUT);
    track.key(recovery, side(follow()), Ease::OUT);
    track.key(frac(recovery, last, 0.45), side(regather()), Ease::SMOOTH);
    track.key(last, stance(), Ease::SMOOTH);

    let which = match arm {
        Arm::Dark => {
            "The dark auto, thrown with the left arm; the light one is \
                      this clip mirrored above the hips."
        }
        Arm::Light => {
            "The light auto: the dark punch through `Pose::other_arm`, \
                       so the two differ in the arm and in nothing else."
        }
    };
    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: format!(
            "{which} A step into the strike rather than a step and then a \
             strike: the lead foot unweights two frames in and lands on the \
             contact frame, so the footwork and the hand are one motion. It \
             reads as a punch and nothing more -- the wing is the ability and \
             not the body: it sweeps in from behind her, on this arm's side, \
             and arrives in front of the fist rather than being swung by it. \
             The cock is shallow on purpose -- five frames is not enough to \
             draw a hand back and put it somewhere else -- and the read lives \
             in the lifted lead knee and the other hand thrown back behind. \
             Crisp, because anything that rings here arrives late and this is \
             the move thrown constantly. The rear foot comes up during recovery \
             and the body settles onto the idle's own stance, which is what the \
             frame after this clip cuts to."
        ),
        keys: track.done(),
    }
}

// ---------------------------------------------------------------------------
// The light Lance -- the committed thrust
// ---------------------------------------------------------------------------
//
// Middle click throws one of two moves and the force in her arms picks which,
// so these two clips have one job before any of the rest: **they must not look
// alike.** A person on the other side of the arena gets the wind-up and nothing
// else to decide between getting out from under a burst and breaking a tether,
// and the two answers are opposites -- leave, or close. So the light one is
// high, fast and forward off the right arm, and the dark one is low, slow and
// settled back off the left. Same input, opposite silhouettes.

/// The free hand goes out onto the target and the point hand starts back.
/// Early enough to be read, and deliberately the wrong shape: what is extended
/// here is not what arrives.
fn aim() -> Pose {
    stance()
        .hips(0.0, -0.075, -0.045)
        .root(2.0, -2.0, -6.0)
        .spine(6.0, 0.0, 8.0)
        .chest(2.0, 0.0, 14.0)
        .head(-2.0, 0.0, -14.0)
        .shoulder_l(84.0, 10.0, 0.0)
        .elbow_l(34.0)
        .wrist_l(-6.0, 0.0, 0.0)
        .shoulder_r(2.0, 16.0, 0.0)
        .elbow_r(92.0)
        .wrist_r(-10.0, 0.0, 0.0)
        .plant_l([L, GROUND + 0.02, 0.19])
        .plant_r([R, GROUND, -0.17])
        .toe_l(-8.0)
        .toe_r(8.0)
}

/// The deepest coil: everything loaded onto the back leg, the point hand drawn
/// back past the hip, the lead foot up and already travelling.
///
/// The chamber is in the elbow rather than the shoulder. A hand taken back by
/// swinging the whole arm has to travel the same distance again to arrive, and
/// the shoulder cannot cover that in the four frames between here and contact
/// without moving faster than the eye will accept as a body.
fn coil() -> Pose {
    stance()
        .hips(-0.01, -0.135, -0.085)
        .root(4.0, -4.0, -2.0)
        .spine(12.0, 0.0, 12.0)
        .chest(4.0, 0.0, 22.0)
        .head(0.0, 0.0, -24.0)
        .shoulder_l(92.0, 8.0, 0.0)
        .elbow_l(26.0)
        .wrist_l(-4.0, 0.0, 0.0)
        .shoulder_r(40.0, 16.0, 0.0)
        .elbow_r(96.0)
        .wrist_r(-14.0, 0.0, 0.0)
        .plant_l([L, GROUND + 0.075, 0.255])
        .plant_r([R, GROUND, -0.17])
        .toe_l(-12.0)
        .toe_r(6.0)
}

/// The last startup frame: the lead foot is down, the body has committed, and
/// the point is most of the way out. The line is nearly there and the free arm
/// is on its way back through.
fn launch() -> Pose {
    stance()
        .hips(0.0, -0.145, 0.115)
        .root(11.0, 0.0, -8.0)
        .spine(13.0, 0.0, -2.0)
        .chest(5.0, 0.0, -10.0)
        .head(-10.0, 0.0, 10.0)
        .shoulder_r(110.0, 8.0, 0.0)
        .elbow_r(20.0)
        .wrist_r(-4.0, 0.0, 0.0)
        .shoulder_l(16.0, 16.0, 0.0)
        .elbow_l(20.0)
        .wrist_l(-6.0, 0.0, 0.0)
        .plant_l([L - 0.02, GROUND + 0.01, 0.42])
        .plant_r([R + 0.02, GROUND + 0.03, -0.18])
        .toe_l(-6.0)
        .toe_floor_r()
}

/// Mid-thrust, and the pose the strike used to skip.
///
/// The free hand has three quarters of a metre to travel between the coil and
/// the launch -- back past the hip, down and across -- and three frames is not
/// enough air time for it. This is the middle of that arc: the point half out,
/// the free hand level with the hip on its way through, the lead foot still
/// hanging. Without it the whole thrust happens in the two frames either side
/// of the launch, and a hand that covers half a metre in one frame is a
/// teleport whatever pose it lands in.
fn extend() -> Pose {
    stance()
        .hips(-0.005, -0.145, 0.015)
        .root(8.0, -2.0, -5.0)
        .spine(13.0, 0.0, 5.0)
        .chest(5.0, 0.0, 6.0)
        .head(-5.0, 0.0, -7.0)
        .shoulder_l(56.0, 12.0, 0.0)
        .elbow_l(22.0)
        .wrist_l(-5.0, 0.0, 0.0)
        .shoulder_r(76.0, 12.0, 0.0)
        .elbow_r(56.0)
        .wrist_r(-9.0, 0.0, 0.0)
        .plant_l([L - 0.01, GROUND + 0.055, 0.34])
        .plant_r([R + 0.01, GROUND + 0.015, -0.175])
        .toe_l(-10.0)
        .toe_r(7.0)
}

/// The contact frame. One line: rear foot, hips, shoulder, point -- with the
/// free hand thrown back down the rear leg so the line runs out of both ends
/// of the body.
///
/// The torso only pitches thirty degrees. Further forward and the shoulder
/// stops being on the line between the rear foot and the point, and the whole
/// thing reads as a dive rather than a thrust.
fn lanced() -> Pose {
    stance()
        .hips(0.0, -0.15, 0.20)
        .root(14.0, 0.0, -12.0)
        .spine(12.0, 0.0, -4.0)
        .chest(4.0, 0.0, -14.0)
        .head(-12.0, 0.0, 16.0)
        .shoulder_r(122.0, 6.0, 0.0)
        .elbow_r(4.0)
        .wrist_r(2.0, 0.0, 0.0)
        .shoulder_l(-16.0, 16.0, 0.0)
        .elbow_l(14.0)
        .wrist_l(-6.0, 0.0, 0.0)
        .plant_l([L - 0.02, GROUND, 0.47])
        .plant_r([R + 0.02, GROUND + 0.06, -0.19])
        .toe_l(-2.0)
        .toe_floor_r()
}

/// Coming off the line: the arm folds first, the head leaves the target last,
/// and the front knee takes all of the weight on the way back.
fn withdraw() -> Pose {
    stance()
        .hips(0.0, -0.155, 0.10)
        .root(16.0, 0.0, -14.0)
        .spine(14.0, 0.0, -4.0)
        .chest(6.0, 0.0, -10.0)
        .head(-10.0, 0.0, 10.0)
        .shoulder_r(70.0, 12.0, 0.0)
        .elbow_r(60.0)
        .wrist_r(-6.0, 0.0, 0.0)
        .shoulder_l(-12.0, 18.0, 0.0)
        .elbow_l(40.0)
        .plant_l([L - 0.02, GROUND, 0.40])
        .plant_r([R + 0.02, GROUND + 0.03, -0.18])
        .toe_l(-4.0)
        .toe_r(6.0)
}

/// The rear foot has dragged up under the hips and the guard is back, but low,
/// and still carrying the lunge.
fn haul_up() -> Pose {
    stance()
        .hips(0.0, -0.115, 0.04)
        .root(7.0, 0.0, -14.0)
        .spine(10.0, 0.0, 4.0)
        .chest(4.0, 0.0, 6.0)
        .head(-4.0, 0.0, 2.0)
        .shoulder_r(28.0, 18.0, 0.0)
        .elbow_r(76.0)
        .shoulder_l(32.0, 16.0, 0.0)
        .elbow_l(68.0)
        .plant_l([L, GROUND, 0.28])
        .plant_r([R, GROUND + 0.03, -0.13])
        .toe_l(-4.0)
        .toe_r(6.0)
}

fn light_lance() -> Recipe {
    let clip = Clip::DualLightLance;
    let (windup, contact, recovery) = phases(clip);
    let last = clip.length() - 1;

    let mut track = Track::new(clip);
    track.key(0, stance(), Ease::OUT);
    track.key(frac(0, windup, 0.35), aim(), Ease::SMOOTH);
    track.key(frac(0, windup, 0.45), coil(), GATHER);
    // The middle of the arc, and then an arrival that settles. `IN` straight
    // from the coil to the launch put the whole thrust in the last frame of
    // the startup and left the arm still travelling when the contact pose
    // arrived, so the two fastest frames in the clip were back to back.
    track.key(frac(0, windup, 0.75), extend(), Ease::OUT);
    track.key(windup, launch(), Ease::STRIKE);
    // The same pose twice, across the active frames: the hitbox is out and
    // nothing moves. A body still travelling while its hitbox is live reads as
    // a swing, and this is a thrust.
    track.contact(contact, lanced(), Ease::HOLD);
    track.key(recovery, lanced(), Ease::OUT);
    track.key(frac(recovery, last, 0.35), withdraw(), Ease::SMOOTH);
    track.key(frac(recovery, last, 0.68), haul_up(), Ease::SMOOTH);
    track.key(last, stance(), Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "One straight thing on the contact frame -- rear foot, hips, \
                shoulder, point -- with the free hand thrown back along the \
                rear leg so the line runs out of both ends of the body. The \
                startup coils away from it: the point hand travels back past \
                the hip while the free hand stays out on the target, so the \
                shape read during the startup is the opposite of the one that \
                arrives. The move roots you, so every centimetre of the lunge \
                is in the pose -- the hips carry twenty forward and the lead \
                foot lands where the reach needs it. Crisp rather than martial, \
                which is the one place this class does not get to be floaty: an \
                arm allowed to ring carries a hand's width past the end of the \
                thrust and then comes back, and a point still travelling while \
                its hitbox is live reads as a swing. The weight in this move is \
                in how long it takes to leave the pose, not in how far the arm \
                wobbles once it arrives. It is thrown off the **right** arm, \
                which is the arm light lives in: middle click is two moves and \
                the arm you see is the second reading of which one is coming."
            .into(),
        keys: track.done(),
    }
}

// ---------------------------------------------------------------------------
// The dark Lance -- the tether
// ---------------------------------------------------------------------------
//
// The other half of middle click, and authored against the light one at every
// point. That one rises, goes early and goes forward; this one **sinks, goes
// late and stays put**. What it throws is not a point but a hand, and what it
// does when it arrives is not let go.

/// The sink: weight drops onto the rear leg, the dark hand starts down and back
/// outside the hip, the free hand comes up across the chest as a counterweight.
///
/// Down rather than up is the whole tell. The light Lance's first readable
/// frame lifts the right hand past the shoulder; this drops the left one below
/// the belt, and the two are told apart at a glance from across the arena
/// before either arm has gone anywhere.
fn sink() -> Pose {
    stance()
        .hips(0.01, -0.125, -0.05)
        .root(-4.0, 6.0, 14.0)
        .spine(-2.0, 5.0, 10.0)
        .chest(2.0, 4.0, 16.0)
        .head(4.0, 0.0, -14.0)
        .shoulder_l(-34.0, 18.0, 0.0)
        .elbow_l(24.0)
        .wrist_l(14.0, 0.0, 0.0)
        .shoulder_r(38.0, 10.0, 0.0)
        .elbow_r(84.0)
        .wrist_r(-8.0, 0.0, 0.0)
        .plant_l([L, GROUND, 0.13])
        .plant_r([R, GROUND, -0.21])
        .toe_l(-4.0)
        .toe_floor_r()
}

/// The haul: the deepest the dark hand gets, dragged back past the rear hip
/// with the whole torso turned away over it. Both knees bent, and nothing has
/// gone forward yet.
///
/// The arm stays nearly straight, unlike the light Lance's chambered elbow.
/// What has to travel here is not a point that must accelerate but a hand on
/// the end of a long arm, and the extra length is the point: it is the reach
/// being wound up, not the speed.
fn haul() -> Pose {
    stance()
        .hips(0.02, -0.165, -0.10)
        .root(-8.0, 12.0, 26.0)
        .spine(-4.0, 9.0, 18.0)
        .chest(2.0, 7.0, 28.0)
        .head(8.0, 0.0, -26.0)
        .shoulder_l(-58.0, 12.0, 0.0)
        .elbow_l(10.0)
        .wrist_l(22.0, 0.0, 0.0)
        .shoulder_r(52.0, 8.0, 0.0)
        .elbow_r(96.0)
        .wrist_r(-10.0, 0.0, 0.0)
        .plant_l([L, GROUND, 0.10])
        .plant_r([R + 0.02, GROUND, -0.24])
        .toe_l(-6.0)
        .toe_floor_r()
}

/// The last startup frame: the hand has come off the bottom of its arc and is
/// travelling forward low, palm already turned up and open. The hips have
/// squared but the feet have not moved -- she is reaching, not lunging.
fn reach_low() -> Pose {
    stance()
        .hips(0.01, -0.15, 0.01)
        .root(-2.0, 4.0, 10.0)
        .spine(0.0, 3.0, 6.0)
        .chest(2.0, 2.0, 8.0)
        .head(2.0, 0.0, -6.0)
        .shoulder_l(34.0, 16.0, 0.0)
        .elbow_l(26.0)
        .wrist_l(16.0, 0.0, 0.0)
        .shoulder_r(30.0, 14.0, 0.0)
        .elbow_r(72.0)
        .wrist_r(-8.0, 0.0, 0.0)
        .plant_l([L, GROUND, 0.16])
        .plant_r([R, GROUND, -0.20])
        .toe_l(-4.0)
        .toe_floor_r()
}

/// The contact frame. The dark arm is out low and level with the hip, wrist
/// cocked back so the hand leads and the palm faces up: the shape of catching
/// something rather than of hitting it.
///
/// **The torso leans away from the arm**, which is the opposite of the light
/// Lance's dive down its own line. Something on the end of that arm is about to
/// start pulling, and a body already committed forward has nothing left to
/// brace with. This is the frame that says the move ends attached.
fn caught() -> Pose {
    stance()
        .hips(0.0, -0.145, 0.03)
        .root(-10.0, 2.0, 6.0)
        .spine(-4.0, 2.0, 2.0)
        .chest(0.0, 1.0, 4.0)
        .head(6.0, 0.0, -4.0)
        .shoulder_l(72.0, 20.0, 0.0)
        .elbow_l(6.0)
        .wrist_l(24.0, 0.0, 0.0)
        .shoulder_r(-24.0, 20.0, 0.0)
        .elbow_r(48.0)
        .wrist_r(-6.0, 0.0, 0.0)
        .plant_l([L - 0.01, GROUND, 0.21])
        .plant_r([R + 0.02, GROUND, -0.23])
        .toe_l(-2.0)
        .toe_floor_r()
}

/// Taking the strain: the free hand comes across onto the same line, both arms
/// low, weight settling further back. The recovery of a move that caught
/// something is a fighter holding on, not one rebuilding a guard.
fn strain() -> Pose {
    stance()
        .hips(0.0, -0.17, 0.0)
        .root(-14.0, 0.0, 2.0)
        .spine(-6.0, 0.0, 0.0)
        .chest(-2.0, 0.0, 2.0)
        .head(10.0, 0.0, -2.0)
        .shoulder_l(64.0, 16.0, 0.0)
        .elbow_l(18.0)
        .wrist_l(20.0, 0.0, 0.0)
        .shoulder_r(50.0, 12.0, 0.0)
        .elbow_r(34.0)
        .wrist_r(12.0, 0.0, 0.0)
        .plant_l([L - 0.01, GROUND, 0.19])
        .plant_r([R + 0.02, GROUND, -0.26])
        .toe_l(-2.0)
        .toe_floor_r()
}

/// Letting the line go slack and standing back up out of the sink, guard
/// gathering low. Still turned slightly along the tether rather than square.
fn slacken() -> Pose {
    stance()
        .hips(0.0, -0.105, -0.01)
        .root(-4.0, 2.0, 8.0)
        .spine(0.0, 2.0, 6.0)
        .chest(0.0, 2.0, 8.0)
        .head(2.0, 0.0, -6.0)
        .shoulder_l(30.0, 20.0, 0.0)
        .elbow_l(62.0)
        .wrist_l(6.0, 0.0, 0.0)
        .shoulder_r(26.0, 16.0, 0.0)
        .elbow_r(70.0)
        .wrist_r(-4.0, 0.0, 0.0)
        .plant_l([L, GROUND, 0.16])
        .plant_r([R, GROUND, -0.19])
        .toe_l(-3.0)
        .toe_floor_r()
}

fn dark_lance() -> Recipe {
    let clip = Clip::DualDarkLance;
    let (windup, contact, recovery) = phases(clip);
    let last = clip.length() - 1;

    let mut track = Track::new(clip);
    track.key(0, stance(), Ease::OUT);
    // Later than the light Lance's first tell, and slower into it. The dark
    // form is the committed one in the sense that matters here: it is the one
    // an opponent has time to answer, and it is supposed to be.
    track.key(frac(0, windup, 0.42), sink(), Ease::SMOOTH);
    track.key(frac(0, windup, 0.62), haul(), Ease::HOLD);
    track.key(frac(0, windup, 0.82), haul(), GATHER);
    track.key(windup, reach_low(), Ease::SMOOTH);
    // The active frames hold the catch, the way the light one holds the
    // thrust -- except that here the hold is what the move *is* rather than a
    // concession to the hitbox.
    track.contact(contact, caught(), Ease::HOLD);
    track.key(recovery, caught(), Ease::SMOOTH);
    track.key(frac(recovery, last, 0.4), strain(), Ease::SMOOTH);
    track.key(frac(recovery, last, 0.75), slacken(), Ease::SMOOTH);
    track.key(last, stance(), Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness {
            // Heavier everywhere than the light Lance, which is CRISP. This one
            // is allowed to trail: a hand reaching for something and then
            // taking its weight is the one move in the kit where an arm still
            // travelling after the body has stopped is the correct read.
            root: Feel::new(1.2, 0.9),
            spine: Feel::new(1.3, 0.85),
            chest: Feel::new(1.4, 0.8),
            head: Feel::new(1.8, 0.65),
            arms: Feel::new(1.6, 0.7),
            legs: Feel::new(1.1, 0.95),
        },
        notes: "The other half of middle click, and authored to be the light \
                Lance's opposite at every point. That one rises off the right \
                shoulder, goes early and dives down its own line; this one \
                sinks onto the rear leg, drags the left hand down and back past \
                the hip, and comes through low with the palm open. The contact \
                frame is the shape of catching something rather than of hitting \
                it -- arm out level with the hip, wrist cocked so the hand \
                leads, and the torso leaning **away** from the arm, because \
                something on the end of it is about to start pulling and a body \
                already committed forward has nothing to brace with. The \
                recovery does not gather a guard: the free hand comes across \
                onto the same line and both arms take the strain, which is the \
                pose that says the move ended attached. Off the left arm, \
                because dark lives in the left arm on this class and the arm is \
                the second reading of which of the two forms is coming."
            .into(),
        keys: track.done(),
    }
}

// ---------------------------------------------------------------------------
// Judgement -- the finisher
// ---------------------------------------------------------------------------

/// The first beat, and the one that has to be legible: the guard comes apart,
/// the arms open wide, the head drops. Nothing else in the class does this, so
/// it is unmistakable from the far side of the arena.
fn open() -> Pose {
    stance()
        .hips(0.0, -0.075, 0.0)
        .root(4.0, 0.0, -6.0)
        .spine(8.0, 0.0, 3.0)
        .chest(2.0, 0.0, 3.0)
        .head(12.0, 0.0, 0.0)
        .shoulders(-10.0, 48.0, 0.0)
        .elbows(26.0)
        .wrists(-16.0, 0.0, 0.0)
        .plant_l([L - 0.01, GROUND, 0.14])
        .plant_r([R + 0.01, GROUND, -0.13])
        .toe_l(-2.0)
        .toe_r(6.0)
}

/// Gathering: the arms sweep up through the sides, the body rises with them,
/// and the rear foot comes up level so the stance is square.
fn gather() -> Pose {
    stance()
        .hips(0.0, -0.03, 0.0)
        .root(0.0, 0.0, 0.0)
        .spine(0.0, 0.0, 0.0)
        .chest(0.0, 0.0, 0.0)
        .head(-6.0, 0.0, 0.0)
        .shoulders(80.0, 36.0, 0.0)
        .elbows(58.0)
        .wrists(-10.0, 0.0, 0.0)
        .plant_l([L - 0.03, GROUND + 0.015, 0.055])
        .plant_r([R + 0.03, GROUND + 0.015, -0.015])
        .toe_l(-6.0)
        .toe_r(-6.0)
}

/// The top, and the moment that is held: both hands together above the head,
/// the body at its longest, up on the balls of both feet.
fn apex() -> Pose {
    stance()
        .hips(0.0, 0.03, -0.02)
        .root(-8.0, 0.0, 0.0)
        .spine(-14.0, 0.0, 0.0)
        .chest(-4.0, 0.0, 0.0)
        .head(-24.0, 0.0, 0.0)
        .shoulders(154.0, 8.0, 0.0)
        .elbows(10.0)
        .wrists(-8.0, 0.0, 0.0)
        .plant_l([L - 0.03, GROUND + 0.03, 0.045])
        .plant_r([R + 0.03, GROUND + 0.03, -0.005])
        .toe_floor_l()
        .toe_floor_r()
}

/// Mid-descent, three frames before the contact.
///
/// The feet are already at their landing spots and flat while the hips are
/// only half of the way down. Without this the drop interpolates from straight
/// legs to folded ones in angle space, and angle space does not know the floor
/// is there -- both soles end up several centimetres through it on the way.
fn descend() -> Pose {
    stance()
        .hips(0.0, -0.075, 0.05)
        .root(10.0, 0.0, 0.0)
        .spine(16.0, 0.0, 0.0)
        .chest(2.0, 0.0, 0.0)
        .head(2.0, 0.0, 0.0)
        .shoulders(126.0, 10.0, 0.0)
        .elbows(14.0)
        .wrists(-10.0, 0.0, 0.0)
        .plant_l([L - 0.03, GROUND, 0.10])
        .plant_r([R + 0.03, GROUND, 0.02])
        .toe_l(-6.0)
        .toe_r(-6.0)
}

/// The contact frame: the whole body folded over both hands.
///
/// The descent is carried by the spine and the hips as much as by the arms. An
/// overhead that travels on the shoulder alone wants a hundred and thirty
/// degrees of it in four frames, which tears; folding the body underneath puts
/// the hands through the same arc in the world with nothing moving faster than
/// a body can move.
fn verdict() -> Pose {
    stance()
        .hips(0.0, -0.19, 0.075)
        .root(24.0, 0.0, 0.0)
        .spine(34.0, 0.0, 0.0)
        .chest(6.0, 0.0, 0.0)
        .head(16.0, 0.0, 0.0)
        .shoulders(96.0, 12.0, 0.0)
        .elbows(20.0)
        .wrists(-14.0, 0.0, 0.0)
        .plant_l([L - 0.03, GROUND, 0.11])
        .plant_r([R + 0.03, GROUND, 0.03])
        .toe_l(-4.0)
        .toe_r(-4.0)
}

/// What it cost: down, folded, hands still low, head hanging.
fn spent() -> Pose {
    stance()
        .hips(0.0, -0.225, 0.055)
        .root(30.0, 0.0, 0.0)
        .spine(38.0, 0.0, 0.0)
        .chest(10.0, 0.0, 0.0)
        .head(26.0, 0.0, 0.0)
        .shoulders(80.0, 16.0, 0.0)
        .elbows(38.0)
        .wrists(-20.0, 0.0, 0.0)
        .plant_l([L - 0.03, GROUND, 0.105])
        .plant_r([R + 0.03, GROUND, 0.025])
        .toe_l(-2.0)
        .toe_r(-2.0)
}

/// The bottom of it. The body is still going down after the arms have stopped,
/// which is the difference between a finisher and a follow-through.
fn sag() -> Pose {
    spent()
        .hips(0.0, -0.245, 0.04)
        .root(33.0, 0.0, 0.0)
        .spine(40.0, 0.0, 0.0)
        .head(30.0, 0.0, 0.0)
        .shoulders(70.0, 18.0, 0.0)
        .elbows(50.0)
        .plant_l([L - 0.03, GROUND, 0.10])
        .plant_r([R + 0.03, GROUND, 0.02])
        .toe_l(-2.0)
        .toe_r(-2.0)
}

/// Standing back up, slowly, with the guard the last thing to come back.
fn rise() -> Pose {
    stance()
        .hips(0.0, -0.155, 0.03)
        .root(14.0, 0.0, -6.0)
        .spine(20.0, 0.0, 2.0)
        .chest(6.0, 0.0, 2.0)
        .head(6.0, 0.0, 0.0)
        .shoulders(30.0, 20.0, 0.0)
        .elbows(64.0)
        .wrists(-12.0, 0.0, 0.0)
        .plant_l([L - 0.03, GROUND, 0.14])
        .plant_r([R + 0.03, GROUND, -0.08])
        .toe_l(-2.0)
        .toe_r(2.0)
}

fn judgement() -> Recipe {
    let clip = Clip::DualSpecial;
    let (windup, contact, recovery) = phases(clip);
    let last = clip.length() - 1;

    let mut track = Track::new(clip);
    track.key(0, stance(), Ease::OUT);
    // Three frames in even though the startup is twenty. A long gathering is
    // not an excuse for a long nothing: the opponent decides early and then
    // watches the rest of it arrive.
    track.key((windup / 6).clamp(2, 4), open(), Ease::SMOOTH);
    track.key(frac(0, windup, 0.42), gather(), Ease::IN);
    // The same pose twice: the moment held at the top. The pull-away into the
    // descent is the ease rather than another key, which is what an anticipate
    // is for. The hold ends seven frames out, not five: the hands cover a
    // metre and three quarters between the top of the arc and the verdict,
    // which is as fast as hands go, and in five frames it is faster than that.
    // Two of those seven are the descent's own -- the mid-drop pose sits three
    // frames before contact rather than one, so the last of the fall is spread
    // across the frames it takes instead of landing in the last of them.
    track.key(frac(0, windup, 0.60), apex(), Ease::HOLD);
    track.key(windup.saturating_sub(6), apex(), PULL_AWAY);
    track.key(windup.saturating_sub(2), descend(), Ease::LINEAR);
    track.contact(contact, verdict(), Ease::OUT);
    track.key(recovery, spent(), Ease::SMOOTH);
    track.key(frac(recovery, last, 0.25), sag(), Ease::IN);
    track.key(frac(recovery, last, 0.65), rise(), Ease::SMOOTH);
    track.key(last, stance(), Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness {
            // Heavy where the weight is allowed to show: the spine and the head
            // keep travelling after the hands have stopped, which is what makes
            // the descent look like it landed on something. Not in the arms --
            // they have to be at the bottom of the arc on the frame the hitbox
            // appears, and three frames of lag there would put the impact after
            // it. Weight reads as follow-through, never as delay.
            root: Feel::new(1.5, 0.9),
            spine: Feel::new(2.2, 0.7),
            chest: Feel::new(2.6, 0.6),
            head: Feel::new(3.2, 0.45),
            arms: Feel::new(1.2, 0.6),
            // The legs are never behind the hips. A body that drops thirty
            // centimetres while its knees are still catching up puts both feet
            // through the floor on the way down.
            legs: Feel::new(1.2, 0.85),
        },
        notes: "A verdict rather than a swing. The other two clips in this file \
                are bladed and one-sided; this one squares the stance and stays \
                symmetric the whole way through, because it is the only thing \
                the class does with both forces at once and because symmetry is \
                what separates a sentence from a hit. The gathering takes the \
                first two thirds of the startup and then stops dead at the top \
                for two frames: the held moment is the move, and it is what \
                makes the descent read as a decision rather than as momentum. \
                Two rather than three, because the fall underneath it is a \
                metre and three quarters of hand travel and it cannot be done \
                in five frames by anything with arms. \
                The drop is carried by the hips and the spine as much as by \
                the arms, both because a shoulder cannot cover that arc on its \
                own without tearing and because a body this thin has to fold \
                to look like it weighs anything. The recovery is twenty-six \
                frames and the body spends the first half of them at the bottom \
                of the drop, because a finisher that pops back up to guard was \
                never final."
            .into(),
        keys: track.done(),
    }
}

// ---------------------------------------------------------------------------
// Sweep -- both arms, across the whole front
// ---------------------------------------------------------------------------
//
// The one move here thrown with both forces at once and still moving. Its job
// in the kit is spatial: the punches are long and thin and lose to somebody who
// is already inside them, and this is the thing that moves that person. So it
// is authored wide rather than deep -- the hands travel across the front at
// hip-to-chest height and end further apart than they started, which is the
// silhouette of a shove rather than of a strike.
//
// It travels **from the character's left to their right**, which is not an
// arbitrary choice: `moves::Shape::Swing` starts a positive arc at `+arc/2` and
// runs to `-arc/2`, and the volume that comes out of that sweeps the same way.
// A clip that crossed the other way would be an animation disagreeing with its
// own hitbox about which side of you it is on.

/// The open: hands come up and apart, weight settling onto the lead foot.
/// Early, and unmistakably not a punch -- both arms leave at once.
fn spread_wide() -> Pose {
    stance()
        .hips(0.0, -0.07, -0.02)
        .root(2.0, -4.0, -16.0)
        .spine(4.0, -3.0, -2.0)
        .chest(0.0, -2.0, -6.0)
        .head(-2.0, 0.0, 8.0)
        .shoulder_l(6.0, 34.0, 0.0)
        .elbow_l(46.0)
        .wrist_l(-10.0, 0.0, 0.0)
        .shoulder_r(14.0, 30.0, 0.0)
        .elbow_r(52.0)
        .wrist_r(-10.0, 0.0, 0.0)
        .plant_l([L - 0.02, GROUND, 0.18])
        .plant_r([R + 0.02, GROUND + 0.02, -0.16])
        .toe_l(-4.0)
        .toe_floor_r()
}

/// The coil, and the frame an opponent decides on: everything gathered across
/// to the character's left, both hands folded in outside the left hip, the
/// whole torso wound the wrong way. What arrives comes from the other side.
///
/// Both elbows are shut. An arm wound back *straight* has to cover its own
/// length again before it starts crossing the front, and four frames is not
/// enough for a hand to do that without teleporting -- which is exactly what
/// the continuity test measured when it was written that way.
fn wind_across() -> Pose {
    stance()
        .hips(-0.035, -0.10, -0.035)
        .root(4.0, -8.0, -34.0)
        .spine(6.0, -8.0, -22.0)
        .chest(2.0, -6.0, -32.0)
        .head(0.0, -4.0, 26.0)
        .shoulder_l(-16.0, 58.0, 0.0)
        .elbow_l(64.0)
        .wrist_l(-14.0, 0.0, 0.0)
        .shoulder_r(44.0, 14.0, 0.0)
        .elbow_r(78.0)
        .wrist_r(-16.0, 0.0, 0.0)
        .plant_l([L - 0.04, GROUND, 0.16])
        .plant_r([R + 0.02, GROUND + 0.02, -0.17])
        .toe_l(-2.0)
        .toe_floor_r()
}

/// The last startup frame: the coil at its tightest, and the hips already
/// turning under it. A sweep is driven from the floor, and this is the frame
/// that says so.
///
/// Close to the coil on purpose. **The arms cross during the active frames,
/// not before them** -- the hit volume sweeps from one side to the other over
/// exactly those six frames (`moves::Shape::Swing`), so an animation that had
/// already thrown the arms across by the time the hitbox appeared would be
/// drawing the move in the wrong place for the whole of it.
fn unwind() -> Pose {
    stance()
        .hips(-0.025, -0.11, -0.015)
        .root(5.0, -6.0, -28.0)
        .spine(7.0, -6.0, -18.0)
        .chest(2.0, -5.0, -30.0)
        .head(0.0, -2.0, 22.0)
        .shoulder_l(-10.0, 60.0, 0.0)
        .elbow_l(58.0)
        .wrist_l(-12.0, 0.0, 0.0)
        .shoulder_r(42.0, 20.0, 0.0)
        .elbow_r(70.0)
        .wrist_r(-14.0, 0.0, 0.0)
        .plant_l([L - 0.035, GROUND, 0.17])
        .plant_r([R + 0.02, GROUND + 0.015, -0.17])
        .toe_l(-3.0)
        .toe_floor_r()
}

/// The contact frame: the hands are still **behind the left shoulder**, which
/// is where the volume starts.
///
/// Behind, and that is the change the arc bought. Sweep runs most of a
/// half-turn now -- it is the one move of hers that reaches a little past both
/// shoulders, because it is the answer to somebody who has already got inside
/// the punches and that is where they are standing. An animation whose hands
/// were level with the hip on the frame the hitbox appeared would be drawing
/// the move a shoulder's width in front of where it hits.
fn entering() -> Pose {
    stance()
        .hips(-0.01, -0.105, 0.02)
        .root(6.0, -2.0, -16.0)
        .spine(8.0, -2.0, -10.0)
        .chest(2.0, -1.0, -16.0)
        .head(0.0, 0.0, 12.0)
        .shoulder_l(14.0, 66.0, 0.0)
        .elbow_l(36.0)
        .wrist_l(-8.0, 0.0, 0.0)
        .shoulder_r(38.0, 34.0, 0.0)
        .elbow_r(44.0)
        .wrist_r(-10.0, 0.0, 0.0)
        .plant_l([L - 0.025, GROUND, 0.19])
        .plant_r([R + 0.03, GROUND + 0.01, -0.16])
        .toe_l(-3.0)
        .toe_floor_r()
}

/// A third of the way through the crossing: the hands are coming off the left
/// shoulder and the torso has squared but not yet turned past it.
///
/// **A key that exists because the arc grew.** Sweep runs most of a half-turn
/// now, so the hands have half again as far to travel in the same six frames,
/// and three keys across the window put 0.37 m of hand between two of them --
/// which `anim/tests/clips.rs` calls a teleport, correctly. Four keys is the
/// same motion at a speed a body could produce.
fn passing() -> Pose {
    stance()
        .hips(0.002, -0.10, 0.04)
        .root(6.0, 0.0, -3.0)
        .spine(8.0, 0.0, 0.0)
        .chest(2.0, 1.0, 0.0)
        .head(0.0, 0.0, 2.0)
        .shoulder_l(32.0, 58.0, 0.0)
        .elbow_l(28.0)
        .wrist_l(-7.0, 0.0, 0.0)
        .shoulder_r(32.0, 48.0, 0.0)
        .elbow_r(34.0)
        .wrist_r(-8.0, 0.0, 0.0)
        .plant_l([L - 0.018, GROUND, 0.195])
        .plant_r([R + 0.035, GROUND + 0.005, -0.15])
        .toe_l(-3.0)
        .toe_r(2.0)
}

/// Two thirds of the way through: both arms out across the front, the torso
/// square, the hands level with the chest rather than above it.
fn across() -> Pose {
    stance()
        .hips(0.015, -0.095, 0.06)
        .root(6.0, 2.0, 10.0)
        .spine(8.0, 3.0, 10.0)
        .chest(2.0, 3.0, 16.0)
        .head(0.0, 1.0, -8.0)
        .shoulder_l(52.0, 46.0, 0.0)
        .elbow_l(20.0)
        .wrist_l(-6.0, 0.0, 0.0)
        .shoulder_r(26.0, 62.0, 0.0)
        .elbow_r(26.0)
        .wrist_r(-6.0, 0.0, 0.0)
        .plant_l([L - 0.01, GROUND, 0.20])
        .plant_r([R + 0.04, GROUND, -0.14])
        .toe_l(-2.0)
        .toe_r(4.0)
}

/// The end of the arc. The hands have run out of front to cross and the body is
/// still turning under them, which is what makes it read as having moved
/// something rather than as having stopped on someone.
fn carried_through() -> Pose {
    stance()
        .hips(0.03, -0.085, 0.035)
        .root(5.0, 6.0, 36.0)
        .spine(7.0, 5.0, 28.0)
        .chest(2.0, 5.0, 40.0)
        .head(0.0, 2.0, -28.0)
        .shoulder_l(72.0, 26.0, 0.0)
        .elbow_l(28.0)
        .wrist_l(-8.0, 0.0, 0.0)
        .shoulder_r(0.0, 88.0, 0.0)
        .elbow_r(30.0)
        .wrist_r(-8.0, 0.0, 0.0)
        .plant_l([L, GROUND + 0.02, 0.19])
        .plant_r([R + 0.05, GROUND, -0.12])
        .toe_l(-2.0)
        .toe_r(2.0)
}

/// Gathering the guard back, still turned away from where the stance wants to
/// be. The recovery is long and this is most of it.
fn rebuild() -> Pose {
    stance()
        .hips(0.01, -0.07, 0.01)
        .root(3.0, 2.0, 8.0)
        .spine(5.0, 2.0, 12.0)
        .chest(2.0, 2.0, 14.0)
        .head(-2.0, 0.0, -6.0)
        .shoulder_l(34.0, 22.0, 0.0)
        .elbow_l(54.0)
        .wrist_l(-8.0, 0.0, 0.0)
        .shoulder_r(16.0, 40.0, 0.0)
        .elbow_r(56.0)
        .wrist_r(-8.0, 0.0, 0.0)
        .plant_l([L - 0.02, GROUND, 0.17])
        .plant_r([R + 0.02, GROUND + 0.02, -0.15])
        .toe_l(-2.0)
        .toe_floor_r()
}

fn sweep() -> Recipe {
    let clip = Clip::DualSweep;
    let (windup, contact, recovery) = phases(clip);
    let last = clip.length() - 1;

    let mut track = Track::new(clip);
    track.key(0, stance(), Ease::OUT);
    track.key(frac(0, windup, 0.28), spread_wide(), Ease::SMOOTH);
    // The coil is held rather than passed through: twelve frames of startup is
    // long enough to be read, and the thing being read is the wind, so it has
    // to sit still long enough to be seen sitting still.
    track.key(frac(0, windup, 0.5), wind_across(), Ease::HOLD);
    track.key(frac(0, windup, 0.72), wind_across(), GATHER);
    track.key(windup, unwind(), Ease::STRIKE);
    // The active window is the sweep. Four keys across it rather than one,
    // because the arms and the hit volume have to be crossing the front at the
    // same time and at the same rate -- the volume runs its whole arc over
    // these six frames, and a body that arrived early would be swinging at
    // nothing while the thing that hurts was still behind it. Four rather than
    // three since the arc went round past both shoulders: the same window with
    // half again as far to travel needs the travel spread, or a hand covers a
    // third of a metre in one frame.
    track.contact(contact, entering(), Ease::LINEAR);
    track.key(frac(contact, recovery, 0.34), passing(), Ease::LINEAR);
    track.key(frac(contact, recovery, 0.67), across(), Ease::LINEAR);
    track.key(recovery, carried_through(), Ease::SMOOTH);
    track.key(frac(recovery, last, 0.5), rebuild(), Ease::SMOOTH);
    track.key(last, stance(), Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness {
            // Loose in the arms, and only there. A sweep is the one thing this
            // class throws that is meant to look like it has weight behind it,
            // and weight in a body this thin has to read as the hands arriving
            // after the hips rather than as mass.
            root: Feel::new(1.0, 0.95),
            spine: Feel::new(1.2, 0.9),
            chest: Feel::new(1.4, 0.85),
            head: Feel::new(2.0, 0.6),
            arms: Feel::new(1.5, 0.75),
            legs: Feel::new(1.1, 0.95),
        },
        notes: "Both arms across the whole front, left to right, driven from \
                the hips. The wind is the telegraph and it is held: everything \
                gathers across to the left, both hands folded in outside the \
                left hip, the torso wound the wrong way -- and then the hips \
                turn under it a frame before the arms go, which is what says \
                this is thrown from the floor rather than from the shoulders. \
                The crossing itself happens during the active frames and not \
                before them, keyed three times across the window, because that \
                is exactly when the hit volume crosses: the arms and the thing \
                that hurts have to be on the same side of the body at the same \
                time. Left to right for the same reason -- an animation that \
                swept the other way would be telling the opponent the wrong \
                side to leave by. Wide rather than deep: the hands end further \
                apart than they started and level with the chest, because what \
                this move is for is moving somebody who is already inside the \
                punches. **Round past both shoulders**: the wind takes the \
                hands behind the left one and the follow-through carries them \
                behind the right, which is what the arc actually sweeps and \
                therefore what the body has to be doing while it does."
            .into(),
        keys: track.done(),
    }
}

// ---------------------------------------------------------------------------
// Placing keys against the phase boundaries
// ---------------------------------------------------------------------------

/// The clip's three boundaries -- last startup frame, first active frame,
/// first recovery frame -- with a fallback for the case the contract stops
/// calling this a move. A bake tool that panics is worse than one that is
/// merely wrong.
fn phases(clip: Clip) -> (u16, u16, u16) {
    clip.phases().unwrap_or_else(|| {
        let n = clip.length();
        (n / 3, n / 3 + 1, (n * 2) / 3)
    })
}

/// A frame a fraction of the way from `a` to `b`. Everything in this file that
/// is not a phase boundary is one of these, so retuning a move carries the
/// texture along with the timing it belongs to.
fn frac(a: u16, b: u16, t: f32) -> u16 {
    a + ((b - a) as f32 * t).round() as u16
}

/// Keys in frame order, with two guarantees the bake needs and one an author
/// needs.
///
/// Phase boundaries can end up a single frame apart -- a move retuned down to
/// a two-frame startup puts the tell on top of the contact frame -- so keys
/// that collide have to give way to something. `contact` is the key that must
/// not: it is the frame the hitbox appears on, and an animation whose contact
/// pose has been dropped teaches the wrong timing. Everything else yields to
/// it.
struct Track {
    keys: Vec<Key>,
    length: u16,
}

impl Track {
    fn new(clip: Clip) -> Track {
        Track {
            keys: Vec::new(),
            length: clip.length(),
        }
    }

    /// Place a key, unless the frame it wants is already taken or off the end.
    fn key(&mut self, frame: u16, pose: Pose, ease: Ease) {
        if frame >= self.length || self.keys.iter().any(|k| k.frame >= frame) {
            return;
        }
        self.keys.push(Key::eased(frame, pose, ease));
    }

    /// Place the contact key, evicting anything a retune has pushed onto or
    /// past its frame.
    fn contact(&mut self, frame: u16, pose: Pose, ease: Ease) {
        let frame = frame.min(self.length.saturating_sub(1));
        self.keys.retain(|k| k.frame < frame);
        self.keys.push(Key::eased(frame, pose, ease));
    }

    fn done(self) -> Vec<Key> {
        self.keys
    }
}
