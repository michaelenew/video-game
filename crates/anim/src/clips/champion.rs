//! The Champion: Sweep, Drive, Uppercut.
//!
//! GENERATED-FRIENDLY: the animation hub (F9) rewrites this file when it saves,
//! in the same shape you would write by hand. Editing it by hand is fine and
//! expected; keep the prose in each recipe's `notes` field, because ordinary
//! comments do not survive a save from the hub.
//!
//! ## What the class fights like
//!
//! Range bands and flow. One weapon that changes form between hammer, sword and
//! spear, multiplying reach, damage and recovery -- so what the Champion is
//! doing at every moment is *choosing a distance*, and the three moves have to
//! occupy visibly different space or that choice is invisible to the person
//! opposite. Sweep owns the width in front of the fighter, Drive owns the line,
//! and Uppercut owns the air above it. Three shapes that cannot be mistaken for
//! each other across the arena, thrown by one body.
//!
//! It is one body. All three are the same trained fighter with both hands on
//! the same haft, standing on the two spots the idle stands on, so a move thrown
//! out of neutral does not shuffle the feet or change the grip -- it just
//! starts, and it puts them back where it found them.
//!
//! ## The grip
//!
//! Nothing hangs off the hands yet, so the weapon exists only as the line
//! between the two wrists, and keeping that line rigid is the whole job of
//! selling it. `weapon()` places both wrists a fixed distance apart along
//! whatever direction the weapon points and solves the arms for them: the hands
//! cannot drift into holding two different things, and every arm in this file
//! is a consequence of where the weapon is rather than of two shoulders keyed
//! separately.
//!
//! It also decides the choreography, in a way worth writing down because it is
//! not obvious until the arms start missing. An arm is 55 cm long and the two
//! shoulders are 37 cm apart, so **the grip has to stay near the middle of the
//! body**: push it out to one side and the far hand has to cross the chest
//! further than a shoulder can adduct, and the solver quietly gives you a
//! broken pose. Real swings work the same way. The hands travel a short arc
//! near the sternum and the head of the weapon travels a long one, so a
//! wind-up is mostly *the weapon pointing somewhere else*, plus a torso that
//! has gone with it.
//!
//! ## Frames
//!
//! Every key is derived from `clip.phases()` -- the last startup frame, the
//! first active frame, the first recovery frame -- read live from the move
//! table. The contact pose sits on the first active frame because that is the
//! frame the hitbox appears, and retuning any of these in the Oven moves the
//! animation with it rather than leaving it teaching the wrong timing.

use crate::bake::{Key, Looseness, Recipe};
use crate::ease::Ease;
use view::clips::Clip;
use view::math::V3;
use view::pose::{ANKLE_ON_GROUND as GROUND, Pose};

/// The two spots the fighter stands on: left foot leading, right foot back and
/// bladed. The same footprint the idle holds, so entering a move and leaving it
/// do not move the feet.
const LEAD: V3 = [-0.145, GROUND, 0.16];
const REAR: V3 = [0.145, GROUND, -0.15];

/// Half the distance between the hands on the haft.
const GRIP: f32 = 0.12;

/// Constant speed *at the hands*, which is not the same thing as constant
/// speed in the joint angles.
///
/// The lunge asks the hands to cross a metre of air twice inside eleven
/// frames, and a hand's speed is not proportional to a shoulder's: an arm
/// stretched out along the haft covers a long way for a few degrees near the
/// end of its range and hardly moves for the same few in the middle. Spread
/// the *angles* evenly with `LINEAR` and the hand still spikes a third faster
/// through the middle of the gap than at either end of it; `SMOOTH`, which
/// deliberately concentrates the change in the middle, makes it a quarter
/// worse again. This spends the gap's time the other way round -- quickly
/// where the hand covers little ground, slowly where it covers a lot -- and
/// what comes out is a hand travelling at one speed from the guard to the
/// chamber and from the chamber to the point.
///
/// Measured, on the Drive: `LINEAR` peaks at 1.04 of the continuity ceiling
/// and `SMOOTH` at 1.26. This peaks at 0.97.
const CARRY: Ease = Ease::new(0.05, 0.40, 0.95, 0.60);

pub fn clips() -> Vec<Recipe> {
    vec![sweep(), drive(), uppercut()]
}

// ---------------------------------------------------------------------------
// The body everything is thrown from
// ---------------------------------------------------------------------------

/// The guard: bladed, weight even, both hands on the haft with the point up and
/// toward the opponent.
///
/// Every clip in this file opens and closes on it. That is not tidiness -- a
/// move that ends somewhere other than where it started pops on the frame
/// control comes back, and the Champion spends Rush to cancel out of recoveries
/// constantly, so that frame arrives early and often.
fn ready() -> Pose {
    let body = Pose::rest()
        .hips(0.0, -0.06, 0.0)
        .root(3.0, 0.0, 10.0)
        .spine(7.0, 0.0, 8.0)
        .chest(4.0, 0.0, 12.0)
        .head(-3.0, 0.0, -24.0)
        .wrists(-8.0, 0.0, 0.0);
    stand(
        weapon(body, [0.0, 1.18, 0.26], [-0.16, 0.90, 0.40]),
        0.0,
        0.02,
    )
}

/// Both hands on the weapon.
///
/// `at` is the middle of the grip **in the character's own space**: the frame
/// the feet stand in, y measured up from the floor and z forward along the
/// facing. So `[0.0, 1.18, 0.26]` is a hand's width in front of the sternum
/// and `[0.02, 1.62, 0.20]` is overhead. Nothing here is measured from the
/// chest, and deliberately so: a pose is resolved into joint angles the moment
/// it is written, so pinning the grip to the chest buys nothing at playback
/// time and costs the author the one thing worth having, which is being able
/// to read a swing's travel straight off the numbers.
///
/// `dir` points along the haft toward the head of the weapon. The **right**
/// hand is the one nearer the head and the left is on the pommel -- an
/// ordinary right-handed two-handed grip, and, more usefully, the assignment
/// that keeps both arms inside their range: wind the weapon out to the right
/// and it is the right shoulder that follows it, instead of the left hand
/// having to cross the whole chest to a place no shoulder can adduct to.
///
/// ## What fits
///
/// A shoulder sits at `[±0.185, 1.47, 0]` and an arm is 55 cm long, so a hand
/// more than half a metre from its own shoulder comes out straight, and one
/// past that comes out straight *and wrong*, silently, because the solver has
/// nowhere else to put it. Two hands 24 cm apart on one haft make that tighter
/// than it sounds. `report_champion_grip` in the anim tests prints every key's
/// grip and what fraction of each arm's reach it spends: keep the fighting
/// poses between about 0.65 and 0.9, and save the ones that read 1.0 for the
/// frames where a straight arm is the point, which in this file is the thrust
/// and nothing else.
///
/// Call it after the torso is posed and before the feet: the arms are solved
/// against a chest the spine has already moved.
fn weapon(pose: Pose, at: V3, dir: V3) -> Pose {
    let d = unit(dir);
    pose.reach_r(along(at, d, GRIP))
        .reach_l(along(at, d, -GRIP))
}

/// Both feet on the guard's footprint. `lead_toe` tips the front foot -- a
/// negative number is weight coming off it -- and `heel` is how far the rear
/// heel has lifted, in metres.
///
/// The rear heel is where the weight actually shows. It is raised by lifting
/// the *ankle* and then rolling the foot down onto its toe with `toe_floor_r`,
/// rather than by pointing the toe at a number of degrees: pointing a toe whose
/// ankle is still on the ground drives the front of the foot straight through
/// the floor, which is what the first pass of all three of these clips did.
fn stand(pose: Pose, lead_toe: f32, heel: f32) -> Pose {
    let pose = pose
        .plant_l(LEAD)
        .plant_r([REAR[0], REAR[1] + heel, REAR[2]])
        .toe_l(lead_toe);
    if heel > 0.001 {
        pose.toe_floor_r()
    } else {
        pose.toe_r(0.0)
    }
}

fn unit(v: V3) -> V3 {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-4);
    [v[0] / len, v[1] / len, v[2] / len]
}

fn along(from: V3, dir: V3, d: f32) -> V3 {
    [
        from[0] + dir[0] * d,
        from[1] + dir[1] * d,
        from[2] + dir[2] * d,
    ]
}

// ---------------------------------------------------------------------------
// Frames, from the phase boundaries
// ---------------------------------------------------------------------------

/// The frame the telegraph lands on: two or three, whatever the startup is.
///
/// The opponent spends the startup deciding whether to block, and what they are
/// deciding from is the first change in the silhouette. Later than frame three
/// and a six-frame poke has no readable startup at all; earlier than two and
/// the springs have not moved anything yet.
fn tell(windup: u16) -> u16 {
    (windup / 3).clamp(2, 3)
}

/// A frame a fraction of the way from one phase boundary to another, so that
/// every key in this file follows the move table rather than a typed-in number.
fn part(a: u16, b: u16, k: f32) -> u16 {
    a + ((b.saturating_sub(a)) as f32 * k).round() as u16
}

/// Keys in the order they were added, dropping any that a retune has collapsed
/// onto an earlier one.
///
/// Frames here are derived, so a move tuned down to a three-frame startup can
/// land the telegraph on the same frame as the wind-up. Dropping the later of
/// the two keeps the contact key where the hitbox is, which is the one key that
/// must not move.
struct Score(Vec<Key>);

impl Score {
    fn new() -> Score {
        Score(Vec::new())
    }

    fn key(&mut self, frame: u16, pose: Pose, ease: Ease) {
        if self.0.last().is_some_and(|k| frame <= k.frame) {
            return;
        }
        self.0.push(Key::eased(frame, pose, ease));
    }
}

// ---------------------------------------------------------------------------
// Sweep -- the poke
// ---------------------------------------------------------------------------

/// A low horizontal cut, right to left, thrown from inside the guard.
///
/// What this move must not be is a forward poke. Every other class's fast
/// button goes out along the facing; the Champion's goes *across*, which is
/// what a wide arc that catches more than one person looks like, and is half of
/// why Sweep and Drive feel like different options from the same spacing. So
/// the hands stay at hip height, the hips drive the arc rather than following
/// it, and the fighter finishes facing somewhere other than where they started.
fn sweep() -> Recipe {
    let clip = Clip::ChampionPoke;
    let (windup, contact, through) = clip.phases().expect("sweep animates a move");
    let last = clip.length().saturating_sub(1);

    // Wound to the right, sunk over the rear leg, leaning into it: the weapon
    // is out at arm's length to the right, level, and everything above the
    // knees has turned to put it there.
    let cocked = {
        let body = Pose::rest()
            .hips(0.05, -0.10, -0.02)
            .root(6.0, -3.0, 22.0)
            .spine(11.0, -6.0, 15.0)
            .chest(6.0, -4.0, 15.0)
            .head(2.0, 2.0, -38.0)
            .wrists(-6.0, 0.0, 0.0);
        stand(
            weapon(body, [0.30, 1.06, 0.10], [0.92, -0.16, 0.36]),
            -6.0,
            0.03,
        )
    };

    // Through the target and past the middle of the arc. The hips have already
    // unwound through square and kept going -- they are what swings the weapon,
    // and a cut whose hips arrive with the hands is a cut thrown with the arms.
    let cut = {
        let body = Pose::rest()
            .hips(-0.05, -0.14, 0.06)
            .root(10.0, 2.0, -10.0)
            .spine(18.0, 4.0, -12.0)
            .chest(8.0, 3.0, -16.0)
            .head(4.0, 0.0, 26.0)
            .wrists(-4.0, 0.0, 0.0);
        stand(
            weapon(body, [0.02, 1.02, 0.42], [-0.52, -0.10, 0.85]),
            0.0,
            0.07,
        )
    };

    // The arc finished out past the lead hip, everything wound the other way.
    // The rear heel is high: that foot has pivoted on its toe through the whole
    // cut, which is what lets the hips turn this far without either foot
    // leaving the spot it started on.
    let follow = {
        let body = Pose::rest()
            .hips(-0.08, -0.12, 0.02)
            .root(8.0, 5.0, -26.0)
            .spine(14.0, 7.0, -22.0)
            .chest(4.0, 5.0, -24.0)
            .head(2.0, -2.0, 42.0)
            .wrists(-6.0, 0.0, 0.0);
        stand(
            weapon(body, [-0.28, 1.06, 0.20], [-0.90, -0.10, -0.42]),
            -2.0,
            0.09,
        )
    };

    // Standing up out of the cut and bringing the point back on line.
    let settle = {
        let body = Pose::rest()
            .hips(-0.04, -0.09, 0.0)
            .root(5.0, 3.0, -8.0)
            .spine(9.0, 4.0, -4.0)
            .chest(3.0, 2.0, -2.0)
            .head(-1.0, 0.0, 14.0)
            .wrists(-8.0, 0.0, 0.0);
        stand(
            weapon(body, [-0.08, 1.14, 0.26], [-0.34, 0.60, 0.72]),
            0.0,
            0.05,
        )
    };

    let mut score = Score::new();
    score.key(0, ready(), Ease::OUT);
    // Linear out of the wind-up, for the same reason a stride is linear. The
    // blade has ninety degrees to cover in four frames; an ease-in-out on top
    // of that puts a spike in the middle of the swing that the solver has to
    // absorb, and what comes out the other side is a pop rather than a fast
    // cut. The springs do the rounding.
    score.key(tell(windup), cocked, Ease::LINEAR);
    // And linear on out through the follow-through, for the same reason
    // again. Contact is not the end of the arc -- the blade has another
    // metre of hand travel to cover before the body has finished unwinding,
    // and `OUT` on either of these gaps spends sixty per cent of it in the
    // first frame, which is the teleport rather than the cut. The
    // deceleration belongs at the far end, coming back to guard.
    score.key(contact, cut, Ease::LINEAR);
    score.key(part(through, last, 0.25), follow, Ease::LINEAR);
    score.key(part(through, last, 0.8), settle, Ease::OUT);
    score.key(last, ready(), Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "The safe poke, and the one move in the set that goes sideways. \
                Thrown from inside the guard without standing up out of it: the \
                hands stay at hip height and the hips drive the arc, so the \
                weapon crosses the whole front instead of reaching out along the \
                facing. Six frames of startup is not enough to hold a pose in, \
                only enough to change one, so the telegraph is a single frame of \
                winding to the right and the blade is moving from then on. Crisp \
                rather than martial: a poke that rings is a poke you cannot \
                throw twice. The recovery is spaced by how far the far hand \
                has to travel rather than by eye: contact to follow-through \
                and follow-through to settle are a metre each, they get six \
                frames each at one speed, and only the last two frames -- \
                coming back to guard -- ease."
            .into(),
        keys: score.0,
    }
}

// ---------------------------------------------------------------------------
// Drive -- the committed move
// ---------------------------------------------------------------------------

/// A lunging thrust: chamber, step, and the whole body behind the point.
///
/// Drive roots you, so every centimetre of the lunge is in the pose -- the hips
/// travel a quarter of a metre forward over a lead foot that steps out to meet
/// them, while the rear foot stays on the spot it started on. That is what
/// commitment looks like from the other side of the arena: a body that has gone
/// somewhere it cannot easily come back from, and then twenty frames of coming
/// back from it.
fn drive() -> Recipe {
    let clip = Clip::ChampionCommitted;
    let (windup, contact, through) = clip.phases().expect("drive animates a move");
    let last = clip.length().saturating_sub(1);

    // Chambered: hands drawn back past the right hip, point already level and
    // on line. Aiming during the wind-up is the telegraph -- the opponent is
    // told exactly where this is going and given eleven frames to leave.
    let chamber = {
        let body = Pose::rest()
            .hips(0.05, -0.11, -0.08)
            .root(2.0, -3.0, 20.0)
            .spine(6.0, -5.0, 14.0)
            .chest(2.0, -4.0, 16.0)
            .head(-2.0, 0.0, -40.0)
            .wrists(-8.0, 0.0, 0.0);
        stand(
            weapon(body, [0.14, 1.12, -0.04], [-0.10, 0.10, 0.99]),
            -10.0,
            0.03,
        )
    };

    // The rear leg starts driving and the lead foot leaves the floor. Second
    // read: the first said "a thrust", this one says "and it is coming now".
    let load = {
        let body = Pose::rest()
            .hips(0.03, -0.15, 0.04)
            .root(6.0, -2.0, 16.0)
            .spine(10.0, -3.0, 12.0)
            .chest(4.0, -2.0, 14.0)
            .head(0.0, 0.0, -34.0)
            .wrists(-8.0, 0.0, 0.0);
        weapon(body, [0.14, 1.12, 0.04], [-0.06, 0.06, 1.0])
            .plant_l([-0.16, GROUND + 0.10, 0.34])
            .toe_l(-14.0)
            .plant_r([REAR[0], REAR[1] + 0.04, REAR[2]])
            .toe_floor_r()
    };

    // Contact: one line from the rear heel to the point. The rear shoulder has
    // come through behind the weapon, which is where the damage comes from and
    // is also the only way a two-handed grip reaches this far forward.
    let thrust = {
        let body = Pose::rest()
            .hips(0.0, -0.22, 0.26)
            .root(14.0, 0.0, -6.0)
            .spine(16.0, 2.0, -10.0)
            .chest(6.0, 2.0, -14.0)
            .head(-4.0, 0.0, 20.0)
            .wrists(-2.0, 0.0, 0.0);
        weapon(body, [0.02, 1.22, 1.00], [-0.02, 0.02, 1.0])
            .plant_l([-0.17, GROUND, 0.62])
            .toe_l(0.0)
            .plant_r([REAR[0], REAR[1] + 0.11, REAR[2]])
            .toe_floor_r()
    };

    // The lunge bottoming out. Still extended, but the weight has arrived and
    // the arms have given: the difference between this and contact is the
    // difference between a strike and the end of one.
    let spent = {
        let body = Pose::rest()
            .hips(-0.02, -0.27, 0.30)
            .root(18.0, 0.0, -2.0)
            .spine(18.0, 3.0, -6.0)
            .chest(8.0, 2.0, -10.0)
            .head(-6.0, 0.0, 14.0)
            .wrists(-4.0, 0.0, 0.0);
        weapon(body, [0.0, 1.14, 0.92], [-0.06, -0.10, 0.99])
            .plant_l([-0.17, GROUND, 0.62])
            .toe_l(0.0)
            .plant_r([REAR[0], REAR[1] + 0.13, REAR[2]])
            .toe_floor_r()
    };

    // Hauling back out of it: the lead foot lifts off the far spot and comes
    // home. A lunge that recovers by sliding its front foot back is a lunge on
    // ice.
    let pull = {
        let body = Pose::rest()
            .hips(0.02, -0.15, 0.08)
            .root(6.0, -2.0, 8.0)
            .spine(8.0, -2.0, 6.0)
            .chest(3.0, -1.0, 8.0)
            .head(-2.0, 0.0, -14.0)
            .wrists(-8.0, 0.0, 0.0);
        weapon(body, [0.10, 1.14, 0.16], [-0.16, 0.40, 0.90])
            .plant_l([-0.155, GROUND + 0.06, 0.30])
            .toe_l(-10.0)
            .plant_r([REAR[0], REAR[1] + 0.04, REAR[2]])
            .toe_floor_r()
    };

    let mut score = Score::new();
    score.key(0, ready(), CARRY);
    score.key(part(tell(windup), windup, 0.3), chamber, Ease::SMOOTH);
    score.key(part(tell(windup), windup, 0.45), load, CARRY);
    score.key(contact, thrust, Ease::STRIKE);
    score.key(through, spent, CARRY);
    score.key(part(through, last, 0.4), pull, CARRY);
    score.key(last, ready(), Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "The lunge. It roots you, so all of the travel is in the pose: \
                the hips go a quarter of a metre forward over a lead foot that \
                steps out to meet them, and the rear foot never leaves the spot \
                it started on. Two reads in the startup rather than one -- the \
                point comes on line through frame five, the rear leg starts \
                driving at six -- and no hold between them, because the hands \
                have two metres to cross before the point lands and eleven \
                frames to cross them in, which at a swordsman's speed is all \
                of them. The recovery takes its time on purpose: twenty \
                frames of pulling yourself back together is the price of the \
                reach, and it should look like it."
            .into(),
        keys: score.0,
    }
}

// ---------------------------------------------------------------------------
// Uppercut -- the special
// ---------------------------------------------------------------------------

/// A rising two-handed strike that leaves the ground.
///
/// The move table gives the Champion eleven metres a second of `self_lift` on
/// the first active frame and nine to whoever it catches, so this is not a
/// rising attack that happens to look airborne: both fighters are genuinely off
/// the floor afterwards and the simulation carries the body up on its own. The
/// clip's job is the push that explains it and the shape that hangs off it -- a
/// deep coil, an extension that ends with the toes leaving the floor on exactly
/// the frame the hitbox appears, legs gathered while the body rises, legs
/// reaching back down as it falls.
///
/// It ends in the air, because the move does. At a lift of eleven against this
/// arena's gravity the fighter is still most of a metre up when the recovery
/// runs out, so the last key hands over to the falling and landing clips rather
/// than pretending to arrive.
fn uppercut() -> Recipe {
    let clip = Clip::ChampionSpecial;
    let (windup, contact, through) = clip.phases().expect("uppercut animates a move");
    let last = clip.length().saturating_sub(1);

    // The weapon drops behind the right hip and the whole body sinks with it.
    // The first thing an opponent sees is the fighter getting shorter, which is
    // a silhouette change nothing else in the kit makes.
    let drop = {
        let body = Pose::rest()
            .hips(0.04, -0.13, -0.04)
            .root(8.0, -3.0, 20.0)
            .spine(12.0, -6.0, 13.0)
            .chest(6.0, -4.0, 14.0)
            .head(0.0, 0.0, -36.0)
            .wrists(-6.0, 0.0, 0.0);
        stand(
            weapon(body, [0.14, 1.00, 0.02], [0.30, -0.86, -0.41]),
            -4.0,
            0.03,
        )
    };

    // The bottom of the coil: knees folded, torso closed over the weapon, eyes
    // up. Held for a few frames, which is what makes a sixteen-frame startup
    // readable rather than merely long.
    let coil = {
        let body = Pose::rest()
            .hips(0.02, -0.25, 0.0)
            .root(16.0, -2.0, 22.0)
            .spine(22.0, -6.0, 14.0)
            .chest(10.0, -4.0, 14.0)
            .head(-8.0, 0.0, -38.0)
            .wrists(-6.0, 0.0, 0.0);
        stand(
            weapon(body, [0.12, 0.88, 0.06], [0.22, -0.90, -0.38]),
            -2.0,
            0.02,
        )
    };

    // The push: legs extending, heels up, the weapon already whipping forward
    // and up through the front of the body.
    let push = {
        let body = Pose::rest()
            .hips(0.0, -0.09, 0.06)
            .root(10.0, 0.0, 6.0)
            .spine(13.0, 0.0, 2.0)
            .chest(6.0, 0.0, 0.0)
            .head(-14.0, 0.0, -10.0)
            .wrists(-4.0, 0.0, 0.0);
        stand(
            weapon(body, [0.06, 1.16, 0.28], [0.06, 0.10, 0.99]),
            6.0,
            0.09,
        )
    };

    // Contact, and the frame the feet leave: one line from a pointed toe to the
    // weapon overhead. The ankles are the last thing on the floor and they are
    // already off it.
    let launch = {
        let body = Pose::rest()
            .hips(0.0, 0.03, 0.08)
            .root(-6.0, 0.0, -6.0)
            .spine(-10.0, 0.0, -6.0)
            .chest(-6.0, 0.0, -8.0)
            .head(-22.0, 0.0, 6.0)
            .wrists(0.0, 0.0, 0.0);
        weapon(body, [0.02, 1.62, 0.20], [-0.10, 0.94, 0.32])
            .plant_l([-0.145, GROUND + 0.06, 0.12])
            .toe_l(34.0)
            .plant_r([0.145, GROUND + 0.06, -0.12])
            .toe_r(34.0)
    };

    // Rising, legs gathered up under the body, the weapon at the top of its
    // arc. This is the shape that says the fighter left the ground on purpose
    // rather than being knocked off it.
    let rising = {
        let body = Pose::rest()
            .hips(0.0, -0.02, -0.04)
            .root(-10.0, 0.0, -4.0)
            .spine(-12.0, 0.0, -4.0)
            .chest(-8.0, 0.0, -6.0)
            .head(-24.0, 0.0, 4.0)
            .wrists(4.0, 0.0, 0.0);
        weapon(body, [0.02, 1.72, 0.02], [-0.04, 0.98, -0.20])
            .plant_l([-0.15, 0.46, 0.06])
            .toe_l(24.0)
            .plant_r([0.15, 0.38, -0.16])
            .toe_r(20.0)
    };

    // The top of the flight: the weapon coming down out of the arc and the body
    // folding back up around it.
    let hang = {
        let body = Pose::rest()
            .hips(0.0, -0.04, 0.0)
            .root(-4.0, 0.0, 0.0)
            .spine(-2.0, 0.0, 2.0)
            .chest(0.0, 0.0, 2.0)
            .head(-8.0, 0.0, 0.0)
            .wrists(-2.0, 0.0, 0.0);
        weapon(body, [0.06, 1.44, 0.20], [-0.16, 0.62, 0.77])
            .plant_l([-0.15, 0.52, 0.16])
            .toe_l(10.0)
            .plant_r([0.15, 0.44, -0.06])
            .toe_r(14.0)
    };

    // Falling: legs reaching for a floor that is not there yet, weapon back in
    // front of the chest.
    let fall = {
        let body = Pose::rest()
            .hips(0.0, -0.06, 0.0)
            .root(4.0, 0.0, 6.0)
            .spine(6.0, 0.0, 6.0)
            .chest(2.0, 0.0, 8.0)
            .head(-2.0, 0.0, -14.0)
            .wrists(-6.0, 0.0, 0.0);
        weapon(body, [0.06, 1.18, 0.24], [-0.22, 0.50, 0.84])
            .plant_l([-0.15, 0.22, 0.12])
            .toe_l(-6.0)
            .plant_r([0.15, 0.18, -0.10])
            .toe_r(-4.0)
    };

    // The handover. Guard restored, legs down and soft, so that whatever the
    // fall and landing clips do next starts from somewhere sensible.
    let ride = {
        let body = Pose::rest()
            .hips(0.0, -0.05, 0.0)
            .root(3.0, 0.0, 9.0)
            .spine(7.0, 0.0, 8.0)
            .chest(4.0, 0.0, 11.0)
            .head(-3.0, 0.0, -22.0)
            .wrists(-8.0, 0.0, 0.0);
        weapon(body, [0.0, 1.17, 0.26], [-0.16, 0.88, 0.44])
            .plant_l([-0.145, GROUND + 0.04, 0.14])
            .toe_l(-8.0)
            .plant_r([0.145, GROUND + 0.02, -0.13])
            .toe_r(-4.0)
    };

    let mut score = Score::new();
    score.key(0, ready(), Ease::OUT);
    score.key(tell(windup), drop, Ease::SMOOTH);
    score.key(part(tell(windup), windup, 0.42), coil, Ease::IN);
    score.key(part(tell(windup), windup, 0.75), push, Ease::IN);
    score.key(contact, launch, Ease::STRIKE);
    score.key(through, rising, Ease::OUT);
    score.key(part(through, last, 0.35), hang, Ease::SMOOTH);
    score.key(part(through, last, 0.7), fall, Ease::SMOOTH);
    score.key(last, ride, Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::HEAVY,
        notes: "The leap. The move table lifts the fighter eleven metres a \
                second on the first active frame and throws whoever it catches \
                up at nine, so the animation is the push that explains it: a \
                deep coil through the startup, an extension that ends with the \
                toes leaving the floor on exactly the frame the hitbox appears, \
                legs gathered while the body rises, legs reaching back down as \
                it falls. Heavy looseness, because past the launch nothing is \
                driving the weapon but the swing that put it there. It ends in \
                the air on purpose -- at that lift the fighter is still most of \
                a metre up when the recovery runs out, so the last key hands \
                over to the fall rather than pretending to land."
            .into(),
        keys: score.0,
    }
}
