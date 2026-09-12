//! Turning on the spot, and crouching.
//!
//! One file, two families, and they belong together: neither is something a
//! character does *instead* of walking. A turn is layered over whatever the
//! legs are already doing, and a crouch replaces the locomotion underneath it
//! while everything else -- attacks, guards, reactions -- carries on as normal.
//!
//! ## A turn clip is indexed by turn rate, not by time
//!
//! Facing follows the mouse, so there is no moment at which a turn "starts"
//! and no event to trigger a clip on. There is only a body that is currently
//! rotating quickly or slowly. So `view::play::turn_layer` reads a frame out of
//! these clips using the **turn rate** as the index, takes the difference
//! between that frame and frame zero, and adds it on top of the pose the
//! character already has.
//!
//! Three consequences, and all four turn clips are built around them:
//!
//! - **Frame zero has to be genuinely neutral.** It is the pose everything else
//!   is measured against, so anything left in it is added to every walk, run
//!   and idle in the game for as long as the player keeps moving the mouse. All
//!   four clips open on `locomotion::stance()` -- not a similar pose, the same
//!   one -- so that the difference at the bottom of the rate range is nothing
//!   at all.
//! - **The clip is a ramp, not an action.** The last frame is the most
//!   committed the turn ever gets; there is no settle at the end, because the
//!   end is "turning as fast as the game allows", not "finished turning".
//! - **Easing and looseness act on rate, not on time.** An ease decides how
//!   much of the turn's shape you get for a given speed of mouse movement, and
//!   a part's lag decides how *fast you have to be turning* before that part
//!   joins in -- it is a threshold, not follow-through. Both turn clips are
//!   given tight looseness for that reason: the trailing arms and the leading
//!   head here are authored into the poses, because the springs cannot supply
//!   them along an axis that is not time.
//!
//! The order the body commits in is the whole content of a turn, so the poses
//! are written in those terms -- how far the hips are still behind the facing,
//! how much of that the shoulders have made up, how far past them the head is
//! looking. Head first, then shoulders, then hips, then feet, and *at higher
//! rates before lower ones*: a gentle turn is a look and a lean with the feet
//! untouched, and only a whipped one moves them.
//!
//! ## The crouch is as low as the rig will go
//!
//! The simulation crouches to 55% of standing height. This gets to about 66%,
//! and the missing ten points are a property of the skeleton rather than a
//! choice: a pose stores a thigh as swing plus spread, and that decomposition
//! degenerates as the thigh approaches horizontal -- past about 70 degrees of
//! swing, the spread needed for any stance wider than the hips starts to blow
//! up, and beyond that a knee tears sideways instead of folding. So the legs
//! are taken to the edge of what they can express, the heels come up so the
//! shins can keep leaning, and the rest of the height comes out of folding the
//! torso over them. What matters for gameplay -- a silhouette that is
//! unmistakably shorter, instantly -- is met by a long way; what is missing is
//! the last few centimetres against the hurtbox.

use crate::bake::{Feel, Key, Looseness, Recipe};
use crate::clips::locomotion::stance;
use crate::ease::Ease;
use view::clips::Clip;
use view::play::CROUCH_STRIDE;
use view::pose::{ANKLE_ON_GROUND as GROUND, Pose};

/// The standing footprint, which is `locomotion::stance`'s and has to stay
/// that: a turn is layered over a body standing in it, and the crouch drops
/// between the feet it already has.
const FOOT_L: [f32; 3] = [-0.145, GROUND, 0.16];
const FOOT_R: [f32; 3] = [0.145, GROUND, -0.15];

/// Ankle to toe, in metres on the reference body. The length of the lever a
/// foot pivots on when it rolls up onto its toe.
const FOOT: f32 = 0.22;

/// How far the standing stance already has its rear toe pointed. The rear heel
/// is off the floor before any of this starts -- a bladed stance does that, and
/// so does the ankle running out of dorsiflexion at that angle.
const REAR_TOE: f32 = 8.4;

/// The ankle height that keeps a rear toe exactly where standing left it while
/// the heel lifts to `degrees`.
///
/// A foot rolling up onto its toe pivots *about the toe*, so the ankle rises as
/// the angle opens. Leaving the ankle on the floor and only opening the angle
/// drives the toe through it instead, which is a centimetre a degree and is
/// invisible in the numbers right up until a test says the foot is underground.
fn on_the_toe(degrees: f32) -> f32 {
    GROUND + FOOT * degrees.to_radians().sin()
}

pub fn clips() -> Vec<Recipe> {
    vec![
        turn(Clip::TurnLeftSlow, Side::Left, Pace::Slow),
        turn(Clip::TurnRightSlow, Side::Right, Pace::Slow),
        turn(Clip::TurnLeftFast, Side::Left, Pace::Fast),
        turn(Clip::TurnRightFast, Side::Right, Pace::Fast),
        crouch_in(),
        crouch_idle(),
        crouch_walk(),
    ]
}

// ---------------------------------------------------------------------------
// Turning
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum Side {
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq)]
enum Pace {
    Slow,
    Fast,
}

/// One rung of a turn: how far round each part of the body has come.
///
/// Written as four rotations that mean something to a person rather than as
/// eleven joint angles, because the content of a turn clip is the *order* the
/// body commits in and that order is only legible if it is what the numbers
/// say. Everything is a change from the standing stance, which is what the
/// renderer subtracts back off before layering.
#[derive(Clone, Copy)]
struct Rung {
    /// Degrees the hips are still short of the facing. The facing is already
    /// where the mouse points, so a turning body is always behind it.
    behind: f32,
    /// Degrees the shoulders have made up on the hips, split between the two
    /// torso members so the torso spirals rather than turning as one block.
    shoulders: f32,
    /// Degrees the head is looking past the shoulders.
    lead: f32,
    /// Lean into the turn. Positive is into a right turn.
    lean: f32,
    /// How far the hips drop, in metres. A body turning hard sinks a little.
    dip: f32,
    /// How far the arms are carried round with the chest.
    carry: f32,
    /// Ankle target and toe angle, per foot.
    left: ([f32; 3], f32),
    right: ([f32; 3], f32),
}

impl Rung {
    /// The stance, untouched: the neutral both turn clips open on.
    const NEUTRAL: Rung = Rung {
        behind: 0.0,
        shoulders: 0.0,
        lead: 0.0,
        lean: 0.0,
        dip: 0.0,
        carry: 0.0,
        left: (FOOT_L, 0.0),
        right: (FOOT_R, REAR_TOE),
    };

    fn pose(&self) -> Pose {
        stance()
            // The weight goes to the inside of the turn, which is what stops a
            // pivot reading as a doll spinning on a spindle.
            .hips(self.lean * 0.0022, -0.055 - self.dip, 0.0)
            .root(2.0, -self.lean * 0.5, -13.0 - self.behind)
            .spine(5.0, -self.lean * 0.45, 6.0 + self.shoulders * 0.45)
            .chest(2.0, -self.lean * 0.3, 7.0 + self.shoulders * 0.55)
            .head(-3.0, -self.lean * 0.35, 4.0 + self.lead)
            // The lead arm comes across the chest and the trailing one opens
            // out. Both are carried by the turn rather than swung by it: this
            // is a fighter keeping their guard while they pivot, not a
            // discus thrower.
            .shoulder_l(14.0 + self.carry * 0.55, 13.0 - self.carry * 0.25, 0.0)
            .elbow_l(52.0 + self.carry * 0.35)
            .shoulder_r(6.0 - self.carry * 0.45, 15.0 + self.carry * 0.5, 0.0)
            .elbow_r(44.0 - self.carry * 0.3)
            .plant_l(self.left.0)
            .plant_r(self.right.0)
            .toe_l(self.left.1)
            .toe_r(self.right.1)
    }
}

/// A turn, authored to the character's right and mirrored for the other side.
///
/// Mirroring rather than authoring twice is exact -- `mirrored` is a swap of
/// the two sides and a sign flip, and the solver is linear channel by channel,
/// so the baked left clip is the baked right clip reflected to the last
/// decimal. Two hand-written turns would differ by a degree here and there,
/// and a player who leans further one way than the other in a game decided by
/// spacing will feel it long before they can say what it is.
fn turn(clip: Clip, side: Side, pace: Pace) -> Recipe {
    let length = clip.length();
    // The committed pose lands a few frames early and is held, so the springs
    // have somewhere to settle and the last frame really is the full turn
    // rather than most of it.
    let settle = 2;
    let rungs = match pace {
        Pace::Slow => slow_turn(),
        Pace::Fast => fast_turn(),
    };
    let at: Vec<u16> = match pace {
        // Frames as fractions of the clip: the numbers are rate thresholds
        // wearing frame numbers, and they should move with the clip's length.
        Pace::Slow => vec![0, length / 3, (length * 2) / 3, length - 1 - settle],
        Pace::Fast => vec![0, (length * 2) / 5, (length * 2) / 3, length - 1 - settle],
    };
    let ease = match pace {
        // A gentle turn should already show something: most of the lean is
        // there by the time the mouse is moving at all, and the rest of the
        // rate range only refines it.
        Pace::Slow => [Ease::OUT, Ease::OUT, Ease::SMOOTH, Ease::SMOOTH],
        // A whip is the opposite. Nothing distinctive happens until the body is
        // genuinely being thrown around, and then all of it does -- otherwise
        // every ordinary turn borrows the crossed feet of a hard one.
        Pace::Fast => [Ease::IN, Ease::SMOOTH, Ease::OUT, Ease::SMOOTH],
    };

    let keys = at
        .iter()
        .zip(rungs.iter())
        .zip(ease.iter())
        .map(|((frame, rung), ease)| {
            let pose = rung.pose();
            let pose = if side == Side::Left {
                pose.mirrored()
            } else {
                pose
            };
            Key::eased(*frame, pose, *ease)
        })
        .collect();

    Recipe {
        clip,
        // Tight, because looseness on a rate-indexed clip buys lateness rather
        // than life: a part with two frames of lag simply does not appear until
        // the player is turning harder, and a turn that only shows up once you
        // are already round is worse than none.
        looseness: match pace {
            Pace::Slow => Looseness::MARTIAL,
            Pace::Fast => Looseness::CRISP,
        },
        notes: notes(side, pace),
        keys,
    }
}

/// A gentle turn: a lean, a look, and the feet barely moving.
///
/// The rear foot does what a rear foot does when you turn a few degrees at a
/// time -- swivels on the ball while the lead foot stays where it is. Anything
/// more is a step, and stepping every time the mouse moves is how a character
/// ends up looking like they are on a turntable.
fn slow_turn() -> [Rung; 4] {
    [
        Rung::NEUTRAL,
        Rung {
            behind: 2.0,
            shoulders: 3.0,
            lead: 7.0,
            lean: 2.0,
            dip: 0.004,
            carry: 3.0,
            left: ([-0.148, GROUND, 0.156], 0.0),
            right: ([0.150, on_the_toe(14.0), -0.146], 14.0),
        },
        Rung {
            behind: 5.0,
            shoulders: 8.0,
            lead: 10.0,
            lean: 4.0,
            dip: 0.009,
            carry: 7.0,
            left: ([-0.152, GROUND, 0.146], 0.0),
            right: ([0.168, on_the_toe(18.0), -0.128], 18.0),
        },
        Rung {
            behind: 8.0,
            shoulders: 13.0,
            lead: 12.0,
            lean: 6.0,
            dip: 0.014,
            carry: 11.0,
            left: ([-0.157, GROUND, 0.132], 0.0),
            right: ([0.192, on_the_toe(22.0), -0.104], 22.0),
        },
    ]
}

/// A whipped turn: the head is already there, the shoulders are on their way,
/// the hips are late and the feet cross.
///
/// The lead foot leaves the ground in the middle of the clip and is down again
/// at the end. That is deliberate on a rate-indexed clip: a foot in the air at
/// half commitment means "at a middling turn rate a foot is off the floor",
/// which is exactly true of a body being thrown round. The rear foot never
/// leaves, so there is always something under the character to layer onto.
fn fast_turn() -> [Rung; 4] {
    [
        Rung::NEUTRAL,
        Rung {
            behind: 4.0,
            shoulders: 10.0,
            lead: 20.0,
            lean: 5.0,
            dip: 0.010,
            carry: 10.0,
            // The lead foot has unweighted but has not gone anywhere. All of
            // the first half of a whip turn is above the waist.
            left: ([-0.140, GROUND + 0.028, 0.172], -6.0),
            right: ([0.158, on_the_toe(15.0), -0.150], 15.0),
        },
        Rung {
            behind: 14.0,
            shoulders: 20.0,
            lead: 22.0,
            lean: 8.0,
            dip: 0.020,
            carry: 20.0,
            // Crossing: in front of the body and past the midline, which is
            // the frame that makes this read as a turn rather than a lean.
            left: ([-0.048, GROUND + 0.082, 0.208], -10.0),
            right: ([0.208, on_the_toe(19.0), -0.098], 19.0),
        },
        Rung {
            behind: 20.0,
            shoulders: 26.0,
            lead: 22.0,
            lean: 9.0,
            dip: 0.015,
            carry: 26.0,
            left: ([0.054, GROUND, 0.202], -2.0),
            right: ([0.244, on_the_toe(22.0), -0.056], 22.0),
        },
    ]
}

fn notes(side: Side, pace: Pace) -> String {
    let way = match side {
        Side::Left => "left",
        Side::Right => "right",
    };
    match pace {
        Pace::Slow => format!(
            "A gentle pivot to the {way}, indexed by turn rate rather than by time: \
             frame zero is the standing stance exactly, and the renderer layers the \
             difference between it and whatever frame the current turn rate picks. \
             So it is a lean and a look, and the rear foot swivels on its ball while \
             the lead foot stays put -- a step here would mean stepping every time \
             the mouse moved. The easing front-loads the shape: most of the lean is \
             present the moment the body is turning at all, because the alternative \
             is a character who does not react until they are already round."
        ),
        Pace::Fast => format!(
            "A whipped turn to the {way}. Same rate indexing as the slow one, so the \
             ordering here is in *rate*: at a middling rate the head has gone round \
             and nothing below the neck has, and only at the top of the range do the \
             hips give way and the lead foot cross over and plant. The easing holds \
             the feet out of it until then -- borrowing crossed feet for an ordinary \
             turn is what makes a character look like they are skating. The rear foot \
             stays down throughout: it is the one the whole turn pivots on, and the \
             layer needs something under the character that is not moving."
        ),
    }
}

// ---------------------------------------------------------------------------
// Crouching
// ---------------------------------------------------------------------------

/// How far the hips drop below standing, in metres. See the header: this is as
/// far as the legs go before the thigh's angles stop being able to say where it
/// is pointing.
const DROP: f32 = 0.32;

/// How far back the hips sit when crouched. A squat that does not sit back
/// falls over, and one that sits back without folding the chest forward over
/// its feet looks like a chair.
const SET_BACK: f32 = 0.20;

/// The crouch, at a depth and a lateral weight.
///
/// Legs excluded: the entry, the hold and the shuffle put their feet in three
/// different places, but everything above the hips is this function in all
/// three. Three separately keyed crouches is how a character ends up an inch
/// taller the moment they start moving.
fn crouch(drop: f32, sway: f32) -> Pose {
    Pose::rest()
        .hips(sway * 0.018, -drop, -SET_BACK)
        // The pelvis tips *back* as the hips drop. Partly because that is what
        // a squat does, and partly for the rig's sake: thigh swing is measured
        // against the root, so leaning the root forward here would spend the
        // budget the legs need to fold at all.
        .root(-4.0, -sway * 3.5, -13.0)
        .spine(40.0, sway * 2.5, 7.0)
        .chest(26.0, sway * 1.5, 9.0)
        // Folded this far forward, a head at rest stares at the floor. This is
        // the angle that puts the eyes back on the opponent, which is most of
        // what stops a crouch reading as a cower.
        .head(-30.0, -sway * 2.0, 6.0)
        // The guard comes in tight. The shoulders swing back against the fold
        // so that the upper arms still hang, and the elbows do the work of
        // keeping the hands up in front of the face.
        .shoulder_l(-30.0, 15.0, 0.0)
        .elbow_l(98.0)
        .wrist_l(-8.0, 0.0, 0.0)
        .shoulder_r(-38.0, 17.0, 0.0)
        .elbow_r(90.0)
        .wrist_r(-8.0, 0.0, 0.0)
}

/// Where the rear foot ends up once the body is down, and how far its heel has
/// come up to allow it.
///
/// The lead foot does not move between standing and crouched, but the rear one
/// has to. Standing is bladed, with thirty centimetres between the feet
/// front-to-back, and a shin can only lean about twenty-four degrees forward
/// before the ankle runs out of dorsiflexion and the sole starts pointing into
/// the floor. Folding a leg that far behind the hips at this depth asks for
/// three times that. So the back foot comes in underneath the body -- which is
/// what a person dropping out of a long stance does, and the only alternative
/// the rig offers is a rear leg up on the very tip of its toe.
const CROUCH_REAR: [f32; 3] = [0.175, 0.0, -0.045];
const CROUCH_REAR_TOE: f32 = 26.0;

/// The feet of a crouch that is not going anywhere.
fn planted(pose: Pose, t: f32) -> Pose {
    let rear = [
        FOOT_R[0] + (CROUCH_REAR[0] - FOOT_R[0]) * t,
        on_the_toe(REAR_TOE + (CROUCH_REAR_TOE - REAR_TOE) * t),
        FOOT_R[2] + (CROUCH_REAR[2] - FOOT_R[2]) * t,
    ];
    pose.plant_l(FOOT_L).plant_r(rear).toe_l(0.0).toe_floor_r()
}

/// The body `t` of the way from standing into the crouch, with both feet solved
/// for where they actually are at that moment.
///
/// The blend does the torso and the arms; the feet are re-planted afterwards,
/// and that is the whole reason this exists rather than two keys and a spring.
/// A pose stores joint angles, so interpolating between standing and crouched
/// interpolates the *angles* -- and halfway between a straight leg and a folded
/// one is not the leg that reaches halfway down. The first version of this clip
/// keyed the two ends and had both feet seven centimetres into the floor for
/// the middle half of the drop.
fn descent(t: f32) -> Pose {
    planted(stance().blend(&crouch(DROP, 0.0), t), t)
}

/// Dropping into it.
fn crouch_in() -> Recipe {
    // Eight frames, and the shape of the drop is in these numbers rather than
    // in an ease, because keys this dense are what keeps the feet out of the
    // floor and an ease between planted keys only slides them. Nearly half the
    // drop is gone by the first frame; it bottoms out four per cent past the
    // held crouch and comes back, because a mass that stops dead exactly where
    // it was aimed does not read as having any.
    let profile = [
        (0, 0.0),
        (1, 0.44),
        (2, 0.79),
        (3, 0.98),
        (5, 1.04),
        (7, 1.0),
    ];

    Recipe {
        clip: Clip::CrouchIn,
        // Legs tighter than the martial default. Eight frames is not long to
        // drop a body this far, and legs that lag by a frame and a half over a
        // drop that fast put a heel through the floor before the hips have
        // finished arriving.
        looseness: Looseness {
            legs: Feel::new(0.7, 1.0),
            ..Looseness::MARTIAL
        },
        notes: "Eight frames to tell an attacker that their overhead is going to \
                whiff, so most of the drop is over within two of them and the rest \
                is the weight arriving. The lead foot never moves -- the body goes \
                down between the feet it already had -- and the rear one is drawn \
                in underneath, because a stance thirty centimetres long cannot fold \
                this far without either stepping in or standing the back leg on the \
                tip of its toe. Keyed every frame or two rather than end to end: \
                halfway between a straight leg and a folded one is not the leg that \
                reaches halfway down, and keying only the ends puts both feet \
                through the floor for the middle of the drop."
            .into(),
        keys: profile
            .iter()
            .map(|(f, t)| Key::eased(*f, descent(*t), Ease::LINEAR))
            .collect(),
    }
}

/// Held low.
fn crouch_idle() -> Recipe {
    let length = Clip::CrouchIdle.length();
    let held = planted(crouch(DROP, 0.0), 1.0);
    // A crouch breathes into its back rather than its chest, and it barely
    // moves: this is a coiled position, and a body that sways about in one is
    // not holding it.
    let breath = planted(
        crouch(DROP - 0.008, 0.0)
            .spine(38.0, 0.0, 7.0)
            .head(-32.0, 0.0, 6.0),
        1.0,
    );
    let weight_left = planted(crouch(DROP + 0.004, -1.0), 1.0);
    let weight_right = planted(crouch(DROP + 0.002, 0.8), 1.0);

    Recipe {
        clip: Clip::CrouchIdle,
        looseness: Looseness::MARTIAL,
        notes: "The held crouch. Hips a third of a metre down and the chest folded \
                over them, so the silhouette is unmistakably shorter than standing \
                from anywhere in the arena. The movement in it is deliberately \
                small -- a breath into the back and a slow shift of weight between \
                the feet, on different periods so the loop does not tick. A crouch \
                is a coiled position, and anything livelier than this reads as a \
                character who cannot hold it."
            .into(),
        keys: vec![
            Key::eased(0, held, Ease::SMOOTH),
            Key::eased(length / 4, weight_left, Ease::SMOOTH),
            Key::eased(length / 2, breath, Ease::SMOOTH),
            Key::eased((length * 3) / 4, weight_right, Ease::SMOOTH),
        ],
    }
}

/// Shuffling, crouched.
///
/// Played by distance like the rest of the locomotion, at `CROUCH_STRIDE`, so
/// the feet are placed by arithmetic from that constant rather than by eye:
///
/// ```text
/// ground per frame = CROUCH_STRIDE / cycle
/// planted foot z   = contact z − (ground per frame) × frames since contact
/// ```
///
/// A foot given that target on consecutive frames does not move, whatever the
/// hips do, which is what "no skate" means mechanically. The stance keys are
/// linear between for the same reason -- an ease on a planted foot is a slide
/// with better manners -- and they are dense, because a pose stores angles and
/// interpolating angles is not the same as interpolating the ankle position
/// they were solved from. Crouched legs are folded much further than standing
/// ones, which makes that difference worse, so the keys are closer together
/// here than in an upright walk.
fn crouch_walk() -> Recipe {
    let cycle = Clip::CrouchWalk.length() as f32;
    let shuffle = Shuffle {
        cycle,
        // Long: over half the cycle on each foot means a stretch of double
        // support at every contact, and a crouched shuffle is nothing if not
        // deliberate about keeping something on the floor.
        stance: 0.56,
        // The feet track ahead of the hips, because the hips are sitting back.
        bias: 0.08,
        lift: 0.055,
        toe_rise: 0.065,
        roll_from: 0.45,
        run_in: 0.30,
    };

    // Three shapes to a step. The hips rise a centimetre into each contact and
    // sink through mid-stance, and the fold answers the rise so the head stays
    // where it is: a crouch that bobs its head back up is a crouch that has
    // stopped being a crouch twice a second.
    let contact = crouch(DROP - 0.012, 0.0)
        .spine(42.0, 0.0, 7.0)
        .chest(27.0, 0.0, 9.0);
    let low = crouch(DROP + 0.008, 0.0)
        .spine(39.0, 0.0, 8.0)
        .chest(25.0, 0.0, 10.0);
    let push = crouch(DROP - 0.004, 0.0)
        .spine(41.0, 0.0, 6.0)
        .chest(26.0, 0.0, 8.0);
    let half = cycle * 0.5;
    let shapes = [
        (0.0, contact),
        ((half * 0.35).round(), low),
        ((half * 0.7).round(), push),
    ];

    // Both halves of the cycle use the same upper body rather than a mirrored
    // one. The crouch is bladed -- it keeps the stance's lead side -- and
    // mirroring that would swap which shoulder is forward twice a second,
    // which is a different animation from a shuffle.
    let mut keys = Vec::new();
    let mut f = 0.0;
    while f < cycle - 0.5 {
        let phase = if f >= half { f - half } else { f };
        let upper = between(&shapes, phase, half);
        keys.push(Key::eased(f as u16, shuffle.place(upper, f), Ease::LINEAR));
        f += 2.0;
    }

    Recipe {
        clip: Clip::CrouchWalk,
        looseness: Looseness::STRIDE,
        notes: "A crouched shuffle, played by distance at CROUCH_STRIDE and with \
                every planted foot placed by arithmetic from it, so it cannot skate \
                whatever speed the player crouch-walks at. Short steps, both feet \
                down for a good part of every contact, and the heel rolling up late \
                because a crouched shin already leans further forward than an ankle \
                can flatten against. The head is held at one height across the whole \
                cycle: the hips bob, and the fold gives back exactly what the bob \
                takes, because a crouch walk that rises on every step stops reading \
                as a crouch."
            .into(),
        keys,
    }
}

/// Everything about where a shuffling foot goes, kept in one place so the
/// numbers that decide skate are all visible together.
#[derive(Clone, Copy)]
struct Shuffle {
    /// Frames in a full cycle: two steps.
    cycle: f32,
    /// Fraction of the cycle a foot spends on the ground.
    stance: f32,
    /// Where the middle of a foot's travel sits, relative to the character's
    /// origin. Positive is forward of it, which is where the feet are when the
    /// hips have sat back.
    bias: f32,
    /// How high the ankle gets at the top of its swing. Low: this is a shuffle,
    /// and a crouched leg that picks its foot up has to unfold to do it.
    lift: f32,
    /// How high the ankle ends up by the end of stance, rolled onto the toe.
    toe_rise: f32,
    /// How far through stance the heel starts to lift, as a fraction.
    roll_from: f32,
    /// How much of the swing is spent holding still in the world before taking
    /// weight. Without it the last frames before contact drag the foot along
    /// the floor.
    run_in: f32,
}

impl Shuffle {
    fn per_frame(&self) -> f32 {
        CROUCH_STRIDE / self.cycle
    }

    /// Half the ground a foot covers while planted sits ahead of the middle of
    /// its travel and half behind, so the stance is centred under the body
    /// rather than all of it in front.
    fn contact(&self) -> f32 {
        CROUCH_STRIDE * self.stance * 0.5 + self.bias
    }

    fn down_for(&self) -> f32 {
        (self.cycle * self.stance).round()
    }

    /// Where a foot is, `s` frames after its own contact.
    fn foot(&self, s: f32) -> [f32; 2] {
        let down = self.down_for();
        let s = s.rem_euclid(self.cycle);
        if s <= down {
            let roll =
                ((s - down * self.roll_from) / (down * (1.0 - self.roll_from))).clamp(0.0, 1.0);
            let rise = self.toe_rise * roll * roll;
            // Rolling onto the toe is a pivot *about the toe*, so the ankle
            // travels forward over it. Place the ankle where the stride says
            // and the part actually touching the ground creeps forward a
            // couple of centimetres a frame instead.
            let pitch = (rise / FOOT).clamp(0.0, 0.95);
            let shift = FOOT * (1.0 - (1.0 - pitch * pitch).sqrt());
            return [GROUND + rise, self.contact() - self.per_frame() * s + shift];
        }

        let swing = self.cycle - down;
        let u = ((s - down) / swing).clamp(0.0, 1.0);
        // The arc peaks early: a foot clears the ground in the first frames
        // after toe-off and then reaches. A symmetric arc leaves the toe
        // scraping exactly where the body is moving fastest over it.
        let arc = (u.powf(0.7) * std::f32::consts::PI).sin();
        let height = GROUND + self.toe_rise * (1.0 - u).powi(2) + self.lift * arc;

        let run_in = swing * self.run_in;
        let z = if s >= self.cycle - run_in {
            // Still in the world, and only descending: its position relative to
            // the body keeps sliding back at the body's own speed.
            self.contact() - self.per_frame() * (s - self.cycle)
        } else {
            let from = self.contact() - self.per_frame() * down;
            let to = self.contact() + self.per_frame() * run_in;
            let t = ((s - down) / (swing - run_in)).clamp(0.0, 1.0);
            from + (to - from) * t
        };
        [height, z]
    }

    /// Put both feet where they belong at absolute frame `f`. The right foot's
    /// own contact is half a cycle after the left's.
    fn place(&self, pose: Pose, f: f32) -> Pose {
        let half = self.cycle * 0.5;
        let l = self.foot(f);
        let r = self.foot(f + half);
        let pose = pose
            .plant_l([FOOT_L[0], l[0], l[1]])
            .plant_r([FOOT_R[0], r[0], r[1]]);
        self.ankle(self.ankle(pose, f, true), f + half, false)
    }

    /// What the ankle is doing, given where that foot is in its own step.
    ///
    /// Derived rather than keyed, because the angle that keeps a toe on the
    /// floor depends on how far the ankle has risen and on how far the shin is
    /// leaning, and neither is a number anybody can hold in their head.
    fn ankle(&self, pose: Pose, s: f32, left: bool) -> Pose {
        let down = self.down_for();
        let s = s.rem_euclid(self.cycle);
        if s > down {
            // Swinging: toes up, so the foot clears instead of dragging.
            let u = (s - down) / (self.cycle - down);
            let dorsi = if u > 0.78 { -5.0 } else { -12.0 };
            return if left {
                pose.toe_l(dorsi)
            } else {
                pose.toe_r(dorsi)
            };
        }
        if s < down * self.roll_from {
            // Flat, and the only part of the cycle where the whole sole is
            // carrying: contact through mid-stance.
            if left {
                pose.toe_l(0.0)
            } else {
                pose.toe_r(0.0)
            }
        } else if left {
            pose.toe_floor_l()
        } else {
            pose.toe_floor_r()
        }
    }
}

/// Read a shape track at a point in the half cycle, wrapping round to the first
/// shape at the end of it.
fn between(shapes: &[(f32, Pose)], f: f32, half: f32) -> Pose {
    let n = shapes.len();
    for i in 0..n {
        let (a_at, a) = shapes[i];
        let (b_at, b) = if i + 1 < n {
            shapes[i + 1]
        } else {
            (half, shapes[0].1)
        };
        if f >= a_at && f <= b_at {
            let t = ((f - a_at) / (b_at - a_at).max(1e-3)).clamp(0.0, 1.0);
            return a.blend(&b, t);
        }
    }
    shapes[n - 1].1
}
