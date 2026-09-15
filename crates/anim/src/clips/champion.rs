//! The Champion: three weapons on three buttons, ten clips.
//!
//! GENERATED-FRIENDLY: the animation hub (F9) rewrites this file when it saves,
//! in the same shape you would write by hand. Editing it by hand is fine and
//! expected; keep the prose in each recipe's `notes` field, because ordinary
//! comments do not survive a save from the hub.
//!
//! ## What the class fights like
//!
//! Range bands and flow. Three weapons on three mouse buttons -- sword, hammer,
//! spear -- and each of them behaves differently on foot, in the air and out of
//! a Rush. So what the Champion is doing at every moment is *choosing a
//! distance*, and the clips have to occupy visibly different space or that
//! choice is invisible to the person opposite. **The sword owns the width, the
//! hammer owns the line under it, and the spear owns the distance.** Three
//! shapes that cannot be mistaken for each other across the arena, thrown by
//! one body.
//!
//! The nineteen read as a grid, and the grid is the thing to keep true:
//!
//! ```text
//!               sword               hammer              spear
//!   hit one     across the front    overhead to floor   lunging thrust
//!   hit two     back the other way  torn up out of it   a second, shorter
//!   hit three   a full turning cut  the whole body      the longest lunge
//!   airborne    down across body    wind up and smash   fan around the aim
//!   rushing     cutting past        dragged along floor dash into the point
//!   taking off  up and away         launch, and hold    the floor throws you
//! ```
//!
//! Read down a column and it is one weapon in six situations, so the *grip*
//! and the weight have to stay recognisable. Read across a row and it is three
//! weapons in one situation, so the *shape* has to differ completely. A player
//! who cannot tell which of the nineteen is coming has to guess, and guessing
//! is not the game.
//!
//! ## The first three rows are one chain
//!
//! Three hits deep, and **every hit is a free choice of all three weapons** --
//! sword into spear into hammer is an ordinary thing to do. That doubles what
//! a clip has to say. It has to say which weapon, which is the column, and it
//! has to say *how deep into a string you are*, which is the row, because the
//! question the opponent is asking is "can I still block the next one" and the
//! answer is written in the body rather than on the HUD.
//!
//! So the three rows are authored to three different rules:
//!
//! * **Hit one opens from the guard and comes back to it.** It is the only row
//!   that does. A move thrown out of neutral must not shuffle the feet or
//!   change the grip -- it just starts, and it puts them back where it found
//!   them.
//! * **Hit two never opens from the guard.** Its first key is the far side of
//!   somebody else's swing: hands where a cut left them, the hammer head still
//!   on the floor, the point still out. It has no wind-up, only a
//!   continuation, and that is the whole read -- a body that did not come back
//!   to guard is a body that is not finished.
//! * **Hit three commits the feet.** The finishers are the only grounded moves
//!   that turn, step through, or leave the floor behind, because they are the
//!   only ones you cannot take back.
//!
//! It is one body. All of them are the same trained fighter with both hands on
//! the same haft, standing on the two spots the idle stands on. The exceptions
//! are the aerials and the takeoffs, which have no floor to put them back on,
//! and the Rush moves, which start from a body already travelling.
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
    vec![
        sweep(),
        slam(),
        drive(),
        backcut(),
        uproot(),
        skewer(),
        crescent(),
        earthbreaker(),
        whirl(),
        air_sword(),
        air_hammer(),
        air_spear(),
        rush_slash(),
        rush_sweep(),
        rush_stab(),
        rising_cut(),
        uppercut(),
        pole_drive(),
        vault(),
    ]
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

/// The footwork of a **step**, for the moves the simulation carries forward.
///
/// `hold` is how far the rear foot is holding *behind* the guard's footprint, in
/// metres of the body's own frame -- so the higher it is, the more that foot is
/// staying where it was while the body travels over it. `lift` is how far the
/// lead foot is off the floor on its way to the spot it will land on.
///
/// **It suggests the push rather than cancelling the translation**, and that is a
/// decision rather than a shortcut. A step is 0.55 m in four frames, which is
/// eight metres a second -- faster than this fighter can walk -- so a rear foot
/// authored to stand *exactly* still would have to slide most of a leg's length
/// back through the hips inside four frames and would read as the character being
/// dragged. What reads as a step is a rear foot that gives ground and comes back
/// and a lead foot that leaves and arrives, which is what these two numbers are
/// for. The arithmetic that matters -- how far the body actually goes -- is
/// `moves::Move::step`, and it is the simulation's, not this file's.
fn step_stance(pose: Pose, hold: f32, lift: f32) -> Pose {
    let pose = pose.plant_l([LEAD[0], GROUND + lift, LEAD[2]]).plant_r([
        REAR[0],
        REAR[1] + 0.04,
        REAR[2] - hold,
    ]);
    if lift > 0.01 {
        pose.toe_l(-16.0).toe_floor_r()
    } else {
        pose.toe_l(0.0).toe_floor_r()
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
// Sword -- the opener, and the first of three cuts
// ---------------------------------------------------------------------------

/// A descending diagonal from over the right shoulder, thrown off a step.
///
/// The simulation swings this on `Plane::Diagonal(Hand::Right)`: the head starts
/// high and out over the right shoulder, passes level and dead ahead through the
/// middle of the arc, and finishes low past the left hip. This clip's whole job
/// is to be that, frame for frame -- the contact key is the *start* of the arc
/// because that is where the hit volume is on the frame it appears, and the key
/// on the last live frame is the end of it, so the blade the player sees and the
/// capsule the hit test uses sweep together.
///
/// **It steps.** `Move::step` carries the body a little over half a metre down
/// its locked facing, beginning three frames before the blade arrives, so the
/// cut lands with the weight already travelling. The feet are authored against
/// that (`step_stance`): the rear foot holds its ground while the body crosses
/// over it, the lead foot is in the air through the cut, and it plants on the
/// frame the arc bottoms out.
fn sweep() -> Recipe {
    let clip = Clip::ChampionSword;
    let (windup, contact, through) = clip.phases().expect("sweep animates a move");
    let last = clip.length().saturating_sub(1);

    // Wound up over the right shoulder with the head tipped back, weight sunk
    // onto the rear leg about to push. One frame of telegraph is all a
    // six-frame startup has room for, so it has to be a big one.
    let cocked = {
        let body = Pose::rest()
            .hips(0.04, -0.09, -0.02)
            .root(2.0, -4.0, 26.0)
            .spine(6.0, -7.0, 18.0)
            .chest(2.0, -5.0, 18.0)
            .head(0.0, 3.0, -34.0)
            .wrists(-6.0, 0.0, 0.0);
        stand(
            weapon(body, [0.36, 1.46, 0.10], [0.56, 0.74, 0.37]),
            -8.0,
            0.05,
        )
    };

    // **The start of the arc.** Head high and out to the right, already coming
    // down. The lead foot has left the floor: the body is a third of the way
    // through its step and the cut is riding on it.
    let high = {
        let body = Pose::rest()
            .hips(0.03, -0.10, 0.04)
            .root(3.0, -3.0, 22.0)
            .spine(7.0, -5.0, 15.0)
            .chest(3.0, -4.0, 15.0)
            .head(2.0, 2.0, -26.0)
            .wrists(-4.0, 0.0, 0.0);
        step_stance(
            weapon(body, [0.38, 1.36, 0.36], [0.57, 0.57, 0.59]),
            0.26,
            0.0,
        )
    };

    // The middle of it: level, dead ahead, hips through square. This is the
    // frame the hit volume is furthest out in front, and it is the only frame
    // of the cut where the weapon is pointing where the player is looking.
    let level = {
        let body = Pose::rest()
            .hips(-0.02, -0.12, 0.06)
            .root(5.0, 0.0, -2.0)
            .spine(10.0, 2.0, -6.0)
            .chest(4.0, 1.0, -8.0)
            .head(3.0, 0.0, 12.0)
            .wrists(-3.0, 0.0, 0.0);
        step_stance(
            weapon(body, [0.04, 1.20, 0.46], [0.02, 0.04, 1.0]),
            0.20,
            0.0,
        )
    };

    // **The end of the arc**: low, out past the left hip, blade pointing down
    // and across. The lead foot has arrived, which is what stops the step being
    // a glide.
    let low = {
        let body = Pose::rest()
            .hips(-0.07, -0.14, 0.04)
            .root(8.0, 4.0, -24.0)
            .spine(13.0, 6.0, -19.0)
            .chest(5.0, 4.0, -20.0)
            .head(3.0, -2.0, 34.0)
            .wrists(-5.0, 0.0, 0.0);
        step_stance(
            weapon(body, [-0.34, 0.94, 0.36], [-0.57, -0.57, 0.59]),
            0.12,
            0.0,
        )
    };

    // The push, two frames before the blade arrives: the rear leg driving, the
    // lead foot off the floor on its way to the spot it will land on. This is
    // the frame the simulation starts carrying the body, so it is the frame the
    // feet have to agree to go.
    let push = {
        let body = Pose::rest()
            .hips(0.04, -0.12, 0.0)
            .root(2.0, -4.0, 25.0)
            .spine(6.0, -6.0, 17.0)
            .chest(2.0, -4.0, 17.0)
            .head(1.0, 3.0, -30.0)
            .wrists(-5.0, 0.0, 0.0);
        step_stance(
            weapon(body, [0.38, 1.42, 0.20], [0.58, 0.68, 0.45]),
            0.20,
            0.09,
        )
    };

    // Standing up out of the cut, bringing the point back on line, and the rear
    // foot coming up under the body from where it was left behind.
    let settle = {
        let body = Pose::rest()
            .hips(-0.03, -0.10, 0.0)
            .root(5.0, 2.0, -8.0)
            .spine(9.0, 3.0, -4.0)
            .chest(3.0, 2.0, -2.0)
            .head(-1.0, 0.0, 12.0)
            .wrists(-8.0, 0.0, 0.0);
        stand(
            weapon(body, [-0.10, 1.14, 0.26], [-0.28, 0.54, 0.79]),
            0.0,
            0.05,
        )
    };

    let mut score = Score::new();
    score.key(0, ready(), Ease::OUT);
    // Linear out of the wind-up and linear through the cut, for the reason a
    // stride is linear: the blade has a hundred degrees to cover in four frames
    // and an ease on either side of that puts a spike in the middle of the
    // swing which the solver has to absorb. What comes out the other side is a
    // pop rather than a cut. The springs do the rounding; the deceleration
    // belongs at the far end, coming back to guard.
    score.key(tell(windup), cocked, Ease::LINEAR);
    score.key(part(tell(windup), windup, 0.75), push, Ease::LINEAR);
    score.key(contact, high, Ease::LINEAR);
    score.key(part(contact, through, 0.5), level, Ease::LINEAR);
    score.key(through, low, Ease::LINEAR);
    score.key(part(through, last, 0.55), settle, Ease::OUT);
    score.key(last, ready(), Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "The opener, and the first stroke of a figure the three sword \
                hits draw together: down from the right, down from the left, \
                then up. Authored against the hit volume rather than beside it \
                -- the key on the first active frame is the *start* of the arc, \
                high and out to the right, and the key on the last one is the \
                end of it, low past the left hip, so what the player sees and \
                what the capsule does are the same sweep. Six frames of startup \
                is not enough to hold a pose in, only enough to change one, so \
                the telegraph is a single frame of winding over the shoulder and \
                the blade is moving from then on. Crisp rather than martial: a \
                poke that rings is a poke you cannot throw twice. The feet are \
                the step -- rear foot holding the ground the body crosses, lead \
                foot in the air through the cut and down on the frame it bottoms \
                out."
            .into(),
        keys: score.0,
    }
}

// ---------------------------------------------------------------------------
// Spear -- the opener, and the one attack thrown with one arm
// ---------------------------------------------------------------------------

/// A jab off the leading hand, with the butt of the shaft still tucked at the
/// hip.
///
/// **The only one-armed move in the class**, and the only one whose hit volume
/// does not leave the body's centre line: `moves::hand` puts it on `Hand::Right`,
/// so the simulation runs the line from that shoulder, and this clip has to put
/// that hand there or the two disagree about where a three-metre pole is. The
/// rear hand keeps the pommel at the hip, which is what a jab with a long weapon
/// actually is -- the front hand is a guide and the back one is the engine.
///
/// **The feet do not move.** The spear is the spacing tool: what it buys is
/// hitting somebody from where you already are, and a jab that stepped in would
/// be giving that back. All the travel is in the shoulder and the shaft sliding
/// through the front hand.
fn drive() -> Recipe {
    let clip = Clip::ChampionSpear;
    let (windup, contact, through) = clip.phases().expect("the jab animates a move");
    let last = clip.length().saturating_sub(1);

    // Both hands close, the point already on line and level. The aim *is* the
    // telegraph: the opponent is told exactly where this is going and given nine
    // frames to not be there.
    let set = {
        let body = Pose::rest()
            .hips(0.03, -0.08, -0.02)
            .root(2.0, -2.0, 16.0)
            .spine(5.0, -3.0, 11.0)
            .chest(2.0, -2.0, 12.0)
            .head(-2.0, 0.0, -26.0)
            .wrists(-6.0, 0.0, 0.0);
        stand(
            body.reach_r([0.22, 1.24, 0.36])
                .reach_l([0.16, 1.04, -0.10]),
            -4.0,
            0.03,
        )
    };

    // **The jab.** The right shoulder drives out and the shaft runs through that
    // hand; the left stays on the pommel at the hip and never goes anywhere. The
    // volume the simulation puts out starts at that shoulder, which is why the
    // hand has to be the thing that travels rather than the chest.
    let out = {
        let body = Pose::rest()
            .hips(0.01, -0.11, 0.10)
            .root(7.0, -1.0, 6.0)
            .spine(9.0, -2.0, 2.0)
            .chest(4.0, -1.0, 2.0)
            .head(-3.0, 0.0, -10.0)
            .wrists(-2.0, 0.0, 0.0);
        stand(
            body.reach_r([0.26, 1.26, 0.72]).reach_l([0.18, 1.06, 0.06]),
            -2.0,
            0.05,
        )
    };

    // The arm at full length with the point out as far as it goes. Held for the
    // last live frame rather than snapped back: what a defender reads to decide
    // whether to step in is the extension, and it has to exist for longer than
    // one frame to be read.
    let full = {
        let body = Pose::rest()
            .hips(0.0, -0.13, 0.14)
            .root(9.0, -1.0, 2.0)
            .spine(11.0, -1.0, -2.0)
            .chest(5.0, -1.0, -2.0)
            .head(-4.0, 0.0, -4.0)
            .wrists(0.0, 0.0, 0.0);
        stand(
            body.reach_r([0.28, 1.26, 0.84]).reach_l([0.18, 1.06, 0.10]),
            0.0,
            0.05,
        )
    };

    // Snapped back to the guard's grip. A poke's recovery is the hand coming
    // home, not the body standing up out of anything -- it never went anywhere.
    let home = {
        let body = Pose::rest()
            .hips(0.02, -0.08, 0.02)
            .root(4.0, 0.0, 12.0)
            .spine(6.0, -1.0, 9.0)
            .chest(3.0, -1.0, 9.0)
            .head(-2.0, 0.0, -18.0)
            .wrists(-6.0, 0.0, 0.0);
        stand(
            weapon(body, [0.12, 1.16, 0.22], [-0.10, 0.44, 0.89]),
            0.0,
            0.03,
        )
    };

    let mut score = Score::new();
    score.key(0, ready(), Ease::OUT);
    score.key(tell(windup), set, Ease::SMOOTH);
    score.key(contact, out, Ease::STRIKE);
    score.key(through, full, Ease::OUT);
    score.key(part(through, last, 0.5), home, CARRY);
    score.key(last, ready(), Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "The jab, and the one attack in the class thrown with one arm. \
                The front hand goes and the back one stays: the pommel is at the \
                hip through the whole thing, which is both what a jab with a long \
                weapon is and the reason this reads as a poke rather than as a \
                short version of a lunge. The hit volume leaves that shoulder \
                rather than the chest -- `moves::hand` says `Right` -- so the \
                hand is the thing that has to travel, and `kinematics.rs` fails \
                if the arm the renderer draws and the arm the simulation swings \
                from ever part company. The feet never move, because the spear's \
                whole job is reaching somebody from where you already are, and \
                the extension is held for the last live frame: what a defender \
                reads before deciding to step in is the arm at full length, and \
                it has to last longer than one frame to be read."
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
    let clip = Clip::ChampionUppercut;
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

// ---------------------------------------------------------------------------
// Hammer -- overhead to the floor
// ---------------------------------------------------------------------------

/// Everything above the head, and then everything on the ground.
///
/// The hammer is the one move in the set whose hit volume goes *under* the
/// fighter: an arc in the vertical plane that finishes at floor level, which is
/// what lets it reach a crouching opponent and a ridge lying along an animal's
/// back. So the pose has to end with the head of the weapon genuinely down
/// there, not merely angled at it -- a slam that stops at knee height is a
/// slam that stops at knee height in the hit test too, and the two would
/// disagree about a move whose whole job is the bottom of its arc.
///
/// It is also the slowest thing the Champion has on the ground, and the
/// telegraph is the product: fifteen frames of raising a weight over your head
/// is a sentence the opponent can read from across the arena, and they are
/// meant to.
fn slam() -> Recipe {
    let clip = Clip::ChampionHammer;
    let (windup, contact, through) = clip.phases().expect("slam animates a move");
    let last = clip.length().saturating_sub(1);

    // First read: the weight starts going up, and the body sinks under it.
    let lift = {
        let body = Pose::rest()
            .hips(0.0, -0.11, -0.04)
            .root(4.0, -2.0, 14.0)
            .spine(6.0, -4.0, 10.0)
            .chest(2.0, -3.0, 12.0)
            .head(-8.0, 0.0, -26.0)
            .wrists(-4.0, 0.0, 0.0);
        stand(
            weapon(body, [0.10, 1.34, 0.06], [0.28, 0.90, -0.34]),
            -4.0,
            0.03,
        )
    };

    // The top of the wind-up: overhead and slightly behind, body open, weight
    // on the back foot. This is the shape the opponent is deciding against.
    let over = {
        let body = Pose::rest()
            .hips(0.02, -0.06, -0.08)
            .root(-8.0, -1.0, 8.0)
            .spine(-12.0, -2.0, 6.0)
            .chest(-8.0, -2.0, 8.0)
            .head(-18.0, 0.0, -14.0)
            .wrists(6.0, 0.0, 0.0);
        stand(
            weapon(body, [0.02, 1.66, -0.06], [-0.06, 0.86, -0.51]),
            -8.0,
            0.05,
        )
    };

    // Contact: the head of the weapon at the floor in front, the whole body
    // dropped over it, the rear heel driven down. Everything is committed --
    // this is the frame that makes the twenty-two of recovery look earned.
    let strike = {
        let body = Pose::rest()
            .hips(0.0, -0.26, 0.16)
            .root(30.0, 0.0, -4.0)
            .spine(34.0, 0.0, -6.0)
            .chest(16.0, 0.0, -8.0)
            .head(6.0, 0.0, 10.0)
            .wrists(-14.0, 0.0, 0.0);
        weapon(body, [0.02, 0.58, 0.62], [0.02, -0.72, 0.69])
            .plant_l([-0.16, GROUND, 0.28])
            .toe_l(-4.0)
            .plant_r([REAR[0], REAR[1], REAR[2] - 0.06])
            .toe_r(0.0)
    };

    // The floor takes it. Knees bent, weapon flat, nothing moving -- the beat
    // where a heavy move has arrived and has not yet begun to come back.
    let settled = {
        let body = Pose::rest()
            .hips(0.0, -0.30, 0.12)
            .root(32.0, 0.0, -2.0)
            .spine(34.0, 0.0, -4.0)
            .chest(14.0, 0.0, -4.0)
            .head(8.0, 0.0, 6.0)
            .wrists(-10.0, 0.0, 0.0);
        weapon(body, [0.02, 0.48, 0.66], [0.0, -0.34, 0.94])
            .plant_l([-0.16, GROUND, 0.28])
            .toe_l(-2.0)
            .plant_r([REAR[0], REAR[1], REAR[2] - 0.06])
            .toe_r(0.0)
    };

    // Hauling it back up. The recovery is long and it should look like lifting
    // something heavy rather than like waiting.
    let haul = {
        let body = Pose::rest()
            .hips(0.0, -0.16, 0.04)
            .root(16.0, -1.0, 10.0)
            .spine(18.0, -2.0, 8.0)
            .chest(8.0, -1.0, 10.0)
            .head(2.0, 0.0, -16.0)
            .wrists(-8.0, 0.0, 0.0);
        stand(
            weapon(body, [0.04, 0.94, 0.44], [0.02, 0.46, 0.89]),
            -4.0,
            0.04,
        )
    };

    let mut score = Score::new();
    score.key(0, ready(), Ease::OUT);
    score.key(tell(windup), lift, Ease::SMOOTH);
    score.key(part(tell(windup), windup, 0.7), over, Ease::OUT);
    score.key(contact, strike, Ease::STRIKE);
    score.key(through, settled, Ease::OUT);
    score.key(part(through, last, 0.55), haul, CARRY);
    score.key(last, ready(), Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::HEAVY,
        notes: "The overhead. Fifteen frames of raising a weight over your head \
                is the most readable telegraph in the kit and it is meant to be: \
                this is the button you get hit by when you guessed wrong, not \
                the one you get surprised by. The contact key puts the head of \
                the weapon on the floor rather than pointed at it, because the \
                hit volume goes to floor level and a pose that stopped at the \
                knee would be telling the opponent a different move happened. \
                Heavy looseness -- past the top of the arc nothing is driving \
                it but its own weight -- and a held beat on the floor before \
                the recovery starts, which is what makes twenty-two frames of \
                hauling it back up read as the price of the swing."
            .into(),
        keys: score.0,
    }
}

// ---------------------------------------------------------------------------
// Air sword -- the same cut, rolled into the vertical
// ---------------------------------------------------------------------------

/// The sword's arc taken out of the horizontal and put into the plane you are
/// aiming down.
///
/// The grounded sweep crosses the front of the body; this one crosses the front
/// of the *opponent*, from high on one side to low on the other. That is the
/// whole difference and it has to be unmistakable, because the two share a
/// button: if the air version read as a horizontal cut with the feet off the
/// ground, the player would have no way to know which of the two they threw.
///
/// There is no floor here, so the legs are doing what legs do in the air --
/// trailing, gathering, counterweighting the arms. Nothing is planted.
fn air_sword() -> Recipe {
    let clip = Clip::ChampionAirSword;
    let (windup, contact, through) = clip.phases().expect("air sword animates a move");
    let last = clip.length().saturating_sub(1);

    let air = |body: Pose, gather: f32| {
        body.plant_l([-0.15, 0.30 + gather, 0.10 - gather * 0.4])
            .toe_l(14.0)
            .plant_r([0.15, 0.22 + gather * 0.8, -0.16 - gather * 0.3])
            .toe_r(10.0)
    };

    // Wound up and over the shoulder, knees drawn up: the compact shape that
    // says a cut is coming down rather than across.
    let cocked = {
        let body = Pose::rest()
            .hips(0.03, -0.04, -0.04)
            .root(-6.0, -3.0, 16.0)
            .spine(-8.0, -5.0, 12.0)
            .chest(-4.0, -4.0, 12.0)
            .head(-12.0, 0.0, -28.0)
            .wrists(2.0, 0.0, 0.0);
        air(weapon(body, [0.16, 1.58, -0.02], [0.34, 0.78, -0.53]), 0.16)
    };

    // Through the cut: the weapon has crossed the body from high right to low
    // left and the legs have swung the other way to pay for it.
    let cut = {
        let body = Pose::rest()
            .hips(-0.04, -0.08, 0.06)
            .root(16.0, 4.0, -10.0)
            .spine(20.0, 6.0, -12.0)
            .chest(10.0, 4.0, -14.0)
            .head(6.0, 0.0, 22.0)
            .wrists(-6.0, 0.0, 0.0);
        air(
            weapon(body, [-0.06, 0.86, 0.50], [-0.34, -0.80, 0.50]),
            0.02,
        )
    };

    // Past the bottom of the arc, body folded over it.
    let follow = {
        let body = Pose::rest()
            .hips(-0.05, -0.10, 0.02)
            .root(20.0, 5.0, -18.0)
            .spine(22.0, 6.0, -16.0)
            .chest(10.0, 4.0, -18.0)
            .head(8.0, 0.0, 30.0)
            .wrists(-8.0, 0.0, 0.0);
        air(
            weapon(body, [-0.22, 0.74, 0.30], [-0.56, -0.74, 0.36]),
            -0.04,
        )
    };

    // Back to a guard that can hand over to falling.
    let recover = {
        let body = Pose::rest()
            .hips(0.0, -0.05, 0.0)
            .root(4.0, 1.0, 8.0)
            .spine(7.0, 1.0, 7.0)
            .chest(4.0, 1.0, 10.0)
            .head(-2.0, 0.0, -18.0)
            .wrists(-8.0, 0.0, 0.0);
        air(weapon(body, [0.0, 1.16, 0.26], [-0.16, 0.70, 0.70]), 0.06)
    };

    let mut score = Score::new();
    score.key(0, ready(), Ease::OUT);
    score.key(tell(windup), cocked, Ease::LINEAR);
    score.key(contact, cut, Ease::LINEAR);
    score.key(part(through, last, 0.3), follow, Ease::LINEAR);
    score.key(last, recover, Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "The sword in the air: the same arc, turned ninety degrees. It \
                shares a button with the grounded sweep, so the one thing it \
                must never read as is a horizontal cut with the feet off the \
                floor -- high on one side to low on the other, and the legs \
                swinging the opposite way to pay for it. Nothing is planted \
                because there is nothing to plant on; the feet gather on the \
                wind-up and trail through the cut, which is what airborne \
                weight looks like. It ends on a guard the falling clips can \
                take over from rather than on the standing one, which would \
                snap the legs straight the frame control came back."
            .into(),
        keys: score.0,
    }
}

// ---------------------------------------------------------------------------
// Air hammer -- the smash
// ---------------------------------------------------------------------------

/// Twenty-two frames of winding up, and then everything downward.
///
/// This is the longest startup the class has, and it is the move the air game
/// is built on: it drives an airborne opponent into the floor, and the floor
/// charges them for the landing. A startup that long is only fair if it is
/// legible, so the wind-up is one held shape -- both arms straight overhead,
/// body arched back -- rather than a gradual gather. You should be able to see
/// it coming and move.
fn air_hammer() -> Recipe {
    let clip = Clip::ChampionAirHammer;
    let (windup, contact, through) = clip.phases().expect("air hammer animates a move");
    let last = clip.length().saturating_sub(1);

    let air = |body: Pose, tuck: f32| {
        body.plant_l([-0.15, 0.24 + tuck, 0.08 - tuck * 0.5])
            .toe_l(12.0)
            .plant_r([0.15, 0.18 + tuck, -0.14 - tuck * 0.5])
            .toe_r(12.0)
    };

    // Gathering: the weapon comes up in front and the knees come with it.
    let gather = {
        let body = Pose::rest()
            .hips(0.0, -0.02, -0.02)
            .root(-4.0, 0.0, 10.0)
            .spine(-6.0, 0.0, 8.0)
            .chest(-2.0, 0.0, 10.0)
            .head(-14.0, 0.0, -18.0)
            .wrists(0.0, 0.0, 0.0);
        air(weapon(body, [0.04, 1.42, 0.16], [0.04, 0.92, 0.39]), 0.12)
    };

    // The held shape: straight overhead, arched, knees up. One silhouette for
    // most of the startup, because a telegraph that keeps changing is not one.
    let wound = {
        let body = Pose::rest()
            .hips(0.0, 0.02, -0.06)
            .root(-16.0, 0.0, 4.0)
            .spine(-20.0, 0.0, 2.0)
            .chest(-12.0, 0.0, 4.0)
            .head(-26.0, 0.0, -6.0)
            .wrists(10.0, 0.0, 0.0);
        air(weapon(body, [0.0, 1.78, -0.10], [-0.04, 0.94, -0.34]), 0.26)
    };

    // The smash. Everything above the head goes below it: arms driven down in
    // front, legs kicked back behind, body folded hard over the blow. The hit
    // volume goes well under the fighter and the pose has to go with it.
    let smash = {
        let body = Pose::rest()
            .hips(0.0, -0.12, 0.10)
            .root(40.0, 0.0, -2.0)
            .spine(40.0, 0.0, -4.0)
            .chest(18.0, 0.0, -4.0)
            .head(14.0, 0.0, 8.0)
            .wrists(-16.0, 0.0, 0.0);
        air(weapon(body, [0.02, 0.42, 0.52], [0.02, -0.88, 0.47]), -0.16)
    };

    // Carried past it, still folded, the legs starting to come back under.
    let through_pose = {
        let body = Pose::rest()
            .hips(0.0, -0.14, 0.06)
            .root(34.0, 0.0, 0.0)
            .spine(32.0, 0.0, -2.0)
            .chest(14.0, 0.0, -2.0)
            .head(12.0, 0.0, 4.0)
            .wrists(-12.0, 0.0, 0.0);
        air(weapon(body, [0.02, 0.52, 0.44], [0.0, -0.62, 0.78]), -0.08)
    };

    let recover = {
        let body = Pose::rest()
            .hips(0.0, -0.05, 0.0)
            .root(6.0, 0.0, 8.0)
            .spine(8.0, 0.0, 7.0)
            .chest(4.0, 0.0, 10.0)
            .head(-2.0, 0.0, -18.0)
            .wrists(-8.0, 0.0, 0.0);
        air(weapon(body, [0.0, 1.14, 0.26], [-0.14, 0.60, 0.79]), 0.06)
    };

    let mut score = Score::new();
    score.key(0, ready(), Ease::OUT);
    score.key(tell(windup), gather, Ease::SMOOTH);
    score.key(part(tell(windup), windup, 0.45), wound, Ease::OUT);
    score.key(contact, smash, Ease::STRIKE);
    score.key(through, through_pose, Ease::OUT);
    score.key(part(through, last, 0.6), recover, Ease::SMOOTH);
    score.key(last, recover, Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::HEAVY,
        notes: "Twenty-two frames of startup, which is the longest thing the \
                class has and only fair because it is legible: the wind-up \
                settles into one held shape -- arms straight overhead, body \
                arched, knees up -- and stays there, because a telegraph that \
                keeps changing is not a telegraph. Then everything above the \
                head goes below it. The smash key drives the weapon under the \
                fighter's own feet, which is where the hit volume goes, and \
                kicks the legs back behind to pay for it. This is the move that \
                puts an airborne opponent into the floor, so the pose has to \
                look like it is aimed at the floor and not at them."
            .into(),
        keys: score.0,
    }
}

// ---------------------------------------------------------------------------
// Air spear -- the fan
// ---------------------------------------------------------------------------

/// A wide sweep around wherever you are pointing, left to right.
///
/// The lightest thing in the set, and the only one thrown with one hand on the
/// haft near the butt: it is a fan rather than a cut, it catches whatever is in
/// a wide band around the aim, and connecting kicks the Champion the way they
/// are holding. So the body reads as *turning* rather than as striking -- the
/// arms are long, the torso rotates around a still head, and the legs trail
/// through the turn like a skater's.
fn air_spear() -> Recipe {
    let clip = Clip::ChampionAirSpear;
    let (windup, contact, through) = clip.phases().expect("air spear animates a move");
    let last = clip.length().saturating_sub(1);

    let air = |body: Pose, trail: f32| {
        body.plant_l([-0.16 - trail * 0.3, 0.26, 0.06 - trail])
            .toe_l(16.0)
            .plant_r([0.16 - trail * 0.2, 0.20, -0.14 - trail * 0.6])
            .toe_r(12.0)
    };

    // Wound across to the left, the point already out at arm's length: the fan
    // starts on that side and travels, so the wind-up has to be on the far end
    // of the arc from where it finishes.
    let wound = {
        let body = Pose::rest()
            .hips(-0.04, -0.04, 0.02)
            .root(4.0, 5.0, -20.0)
            .spine(6.0, 7.0, -16.0)
            .chest(2.0, 5.0, -18.0)
            .head(-4.0, 0.0, 26.0)
            .wrists(-4.0, 0.0, 0.0);
        air(weapon(body, [-0.18, 1.24, 0.24], [-0.86, 0.10, 0.50]), 0.10)
    };

    // The middle of the fan, straight out along the aim. The torso has rotated
    // through square and the head has stayed pointed where the mouse is.
    let sweep_mid = {
        let body = Pose::rest()
            .hips(0.0, -0.06, 0.06)
            .root(4.0, 0.0, 2.0)
            .spine(6.0, 0.0, 0.0)
            .chest(2.0, 0.0, -2.0)
            .head(-2.0, 0.0, 2.0)
            .wrists(-2.0, 0.0, 0.0);
        air(weapon(body, [0.02, 1.22, 0.40], [0.04, 0.02, 1.0]), 0.0)
    };

    // Finished out to the right, everything wound the other way, the legs
    // trailing the turn.
    let finish = {
        let body = Pose::rest()
            .hips(0.04, -0.04, 0.02)
            .root(4.0, -5.0, 22.0)
            .spine(6.0, -7.0, 18.0)
            .chest(2.0, -5.0, 20.0)
            .head(-4.0, 0.0, -26.0)
            .wrists(-4.0, 0.0, 0.0);
        air(weapon(body, [0.20, 1.24, 0.22], [0.88, 0.08, 0.47]), -0.10)
    };

    let recover = {
        let body = Pose::rest()
            .hips(0.0, -0.05, 0.0)
            .root(4.0, 0.0, 8.0)
            .spine(7.0, 0.0, 7.0)
            .chest(4.0, 0.0, 10.0)
            .head(-2.0, 0.0, -18.0)
            .wrists(-8.0, 0.0, 0.0);
        air(weapon(body, [0.0, 1.16, 0.26], [-0.14, 0.62, 0.77]), 0.04)
    };

    let mut score = Score::new();
    score.key(0, ready(), Ease::OUT);
    score.key(tell(windup), wound, Ease::LINEAR);
    score.key(contact, sweep_mid, Ease::LINEAR);
    score.key(part(contact, through, 0.9), finish, Ease::LINEAR);
    score.key(last, recover, Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "The fan. It sweeps a wide band around wherever the mouse is \
                pointing, left to right, and it kicks you the way you are \
                holding if it catches anybody -- so it has to read as the body \
                *turning* rather than as a strike: long arms, torso rotating \
                around a head that stays pointed at the aim, legs trailing \
                through the turn. The wind-up is on the far side of the arc \
                from the finish, which is the only way a sweep looks like it \
                travelled rather than appeared. Light and crisp; it is the \
                fastest thing the class has in the air and the weakest per hit."
            .into(),
        keys: score.0,
    }
}

// ---------------------------------------------------------------------------
// Rush slash -- cutting as you run past
// ---------------------------------------------------------------------------

/// Twenty frames of active, three cuts, and the feet never stop.
///
/// The samurai walks through a line of people and they fall over behind him.
/// Mechanically it is one long active window that re-arms every seven frames,
/// alternating direction; the animation has to match that exactly or the cuts
/// land on frames where the body is between them.
///
/// Nothing here is planted. The dash drives the fighter at a fixed speed
/// regardless of what the legs do, so the legs are running -- and the *arms*
/// carry the whole move, which is the one place in this file where that is the
/// right answer rather than a failure.
fn rush_slash() -> Recipe {
    let clip = Clip::ChampionRushSlash;
    let (windup, contact, through) = clip.phases().expect("rush slash animates a move");
    let last = clip.length().saturating_sub(1);

    // A stride, parameterised so the same shape can be dropped at each cut with
    // the legs in the opposite phase.
    let running = |body: Pose, phase: f32| {
        body.plant_l([-0.15, 0.10 + 0.20 * phase.max(0.0), 0.34 * phase])
            .toe_l(18.0 * phase)
            .plant_r([0.15, 0.10 + 0.20 * (-phase).max(0.0), -0.34 * phase])
            .toe_r(-18.0 * phase)
    };

    let carry = {
        let body = Pose::rest()
            .hips(0.0, -0.08, 0.08)
            .root(10.0, 0.0, 12.0)
            .spine(12.0, 0.0, 10.0)
            .chest(6.0, 0.0, 12.0)
            .head(-6.0, 0.0, -20.0)
            .wrists(-6.0, 0.0, 0.0);
        running(weapon(body, [0.10, 1.20, 0.22], [0.30, 0.24, 0.92]), 0.6)
    };

    // Cut one, right to left. The torso leads and the weapon crosses in front
    // of a body that is still travelling forward.
    let cut_right = {
        let body = Pose::rest()
            .hips(-0.03, -0.10, 0.10)
            .root(12.0, 4.0, -16.0)
            .spine(16.0, 6.0, -14.0)
            .chest(8.0, 4.0, -16.0)
            .head(0.0, 0.0, 24.0)
            .wrists(-4.0, 0.0, 0.0);
        running(
            weapon(body, [-0.10, 1.06, 0.44], [-0.62, -0.10, 0.78]),
            -0.5,
        )
    };

    // Cut two, back the other way. Same body, mirrored, one stride later.
    let cut_left = {
        let body = Pose::rest()
            .hips(0.03, -0.10, 0.10)
            .root(12.0, -4.0, 18.0)
            .spine(16.0, -6.0, 15.0)
            .chest(8.0, -4.0, 17.0)
            .head(0.0, 0.0, -24.0)
            .wrists(-4.0, 0.0, 0.0);
        running(weapon(body, [0.14, 1.10, 0.42], [0.66, -0.06, 0.75]), 0.5)
    };

    let away = {
        let body = Pose::rest()
            .hips(0.0, -0.07, 0.06)
            .root(8.0, 0.0, 10.0)
            .spine(10.0, 0.0, 9.0)
            .chest(5.0, 0.0, 11.0)
            .head(-4.0, 0.0, -18.0)
            .wrists(-6.0, 0.0, 0.0);
        running(weapon(body, [0.04, 1.18, 0.28], [0.10, 0.34, 0.94]), -0.3)
    };

    let mut score = Score::new();
    score.key(0, ready(), Ease::OUT);
    score.key(tell(windup), carry, Ease::LINEAR);
    // One key per re-hit, on the frame the cut re-arms. The interval comes out
    // of the move table, so retuning it moves the keys rather than leaving the
    // animation cutting between the hits.
    let rehit = sim::moves::get(sim::Class::Champion, sim::moves::champion::RUSH_SLASH).rehit;
    let mut at = contact;
    let mut left = false;
    while at < through && rehit > 0 {
        score.key(at, if left { cut_left } else { cut_right }, Ease::LINEAR);
        left = !left;
        at += rehit;
    }
    score.key(through, away, Ease::LINEAR);
    score.key(last, ready(), Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "The run-through. One long active window that re-arms every few \
                frames and alternates direction, so the animation drops a cut \
                key on exactly the frames the move can hit again -- read out of \
                the move table rather than typed in, or a retune leaves the \
                body between cuts on the frames that land. Nothing is planted: \
                the dash drives the fighter at its own speed whatever the legs \
                do, so the legs are running and the arms carry the whole move. \
                That is normally a failure and here it is the point -- you are \
                not stopping to swing, you are swinging because you are going \
                past."
            .into(),
        keys: score.0,
    }
}

// ---------------------------------------------------------------------------
// Rush stab -- the dash put into the point
// ---------------------------------------------------------------------------

/// The one Rush move that stops.
///
/// Everything the dash had goes into the point: both feet arrive planted, the
/// body is braced against its own momentum, and the weapon is out further than
/// anything else in the game reaches. It is the Champion's biggest single hit
/// and its longest recovery, and both facts have to be in the pose -- a stab
/// that finished standing up would be asking why it costs twenty-four frames.
fn rush_stab() -> Recipe {
    let clip = Clip::ChampionRushStab;
    let (windup, contact, through) = clip.phases().expect("rush stab animates a move");
    let last = clip.length().saturating_sub(1);

    // Planting: the lead foot goes out hard to stop the dash, the weapon
    // chambers along the hip. Nine frames, and every one of them is braking.
    let plant = {
        let body = Pose::rest()
            .hips(0.03, -0.18, -0.04)
            .root(-4.0, -2.0, 22.0)
            .spine(-2.0, -5.0, 16.0)
            .chest(0.0, -4.0, 18.0)
            .head(-4.0, 0.0, -40.0)
            .wrists(-8.0, 0.0, 0.0);
        weapon(body, [0.16, 1.10, -0.10], [-0.12, 0.06, 0.99])
            .plant_l([-0.17, GROUND, 0.46])
            .toe_l(-18.0)
            .plant_r([REAR[0], REAR[1], REAR[2] - 0.04])
            .toe_r(0.0)
    };

    // Contact: one line from the back heel to the point, further out than the
    // lunge goes because the dash paid for it.
    let strike = {
        let body = Pose::rest()
            .hips(0.0, -0.26, 0.34)
            .root(16.0, 0.0, -6.0)
            .spine(18.0, 2.0, -10.0)
            .chest(7.0, 2.0, -14.0)
            .head(-4.0, 0.0, 22.0)
            .wrists(-2.0, 0.0, 0.0);
        weapon(body, [0.02, 1.24, 1.10], [-0.02, 0.0, 1.0])
            .plant_l([-0.17, GROUND, 0.70])
            .toe_l(0.0)
            .plant_r([REAR[0], REAR[1] + 0.12, REAR[2] - 0.06])
            .toe_floor_r()
    };

    // Bottomed out: the arms have given and the weight has caught up.
    let spent = {
        let body = Pose::rest()
            .hips(-0.03, -0.31, 0.36)
            .root(20.0, 0.0, -2.0)
            .spine(20.0, 3.0, -6.0)
            .chest(9.0, 2.0, -10.0)
            .head(-8.0, 0.0, 16.0)
            .wrists(-4.0, 0.0, 0.0);
        weapon(body, [0.0, 1.12, 1.00], [-0.06, -0.14, 0.99])
            .plant_l([-0.17, GROUND, 0.70])
            .toe_l(0.0)
            .plant_r([REAR[0], REAR[1] + 0.14, REAR[2] - 0.06])
            .toe_floor_r()
    };

    // Hauling out of the longest recovery in the kit.
    let pull = {
        let body = Pose::rest()
            .hips(0.02, -0.16, 0.10)
            .root(7.0, -2.0, 9.0)
            .spine(9.0, -2.0, 7.0)
            .chest(4.0, -1.0, 9.0)
            .head(-2.0, 0.0, -16.0)
            .wrists(-8.0, 0.0, 0.0);
        weapon(body, [0.10, 1.14, 0.18], [-0.16, 0.40, 0.90])
            .plant_l([-0.155, GROUND + 0.06, 0.32])
            .toe_l(-10.0)
            .plant_r([REAR[0], REAR[1] + 0.04, REAR[2]])
            .toe_floor_r()
    };

    let mut score = Score::new();
    score.key(0, ready(), CARRY);
    score.key(part(tell(windup), windup, 0.5), plant, Ease::OUT);
    score.key(contact, strike, Ease::STRIKE);
    score.key(through, spent, CARRY);
    score.key(part(through, last, 0.45), pull, CARRY);
    score.key(last, ready(), Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "The dash stopped dead and put into the point. Nine frames of \
                startup and all of them are braking: the lead foot goes out \
                hard, the weapon chambers along the hip, and then everything \
                the Rush was carrying arrives at the end of the haft. It \
                reaches further than the standing lunge because the run paid \
                for it, and it costs the longest recovery in the kit, so the \
                spent key is the arms genuinely giving rather than a shorter \
                version of contact. Martial looseness: this is a technique, not \
                a swing."
            .into(),
        keys: score.0,
    }
}

// ---------------------------------------------------------------------------
// Pole vault -- the spear in the floor
// ---------------------------------------------------------------------------

/// No strike in it at all.
///
/// The only clip in the set with no hit volume behind it, and the animation is
/// the entire explanation of the move: you are travelling, you put the butt of
/// the spear into the ground in front of you, and the run becomes height. If it
/// does not read as *planting*, the fighter looks like they jumped for no
/// reason.
///
/// Both hands stay on the haft and the haft stays pointed at the spot on the
/// floor, which is what the grip solver is for -- the hands climb the shaft as
/// the body comes up over it.
fn vault() -> Recipe {
    let clip = Clip::ChampionVault;
    let (windup, contact, through) = clip.phases().expect("vault animates a move");
    let last = clip.length().saturating_sub(1);

    // Reaching down and forward with the point, still running. The read is the
    // spear going *down*, which nothing else in the kit does.
    let reach = {
        let body = Pose::rest()
            .hips(0.0, -0.14, 0.12)
            .root(20.0, 0.0, 8.0)
            .spine(22.0, 0.0, 6.0)
            .chest(10.0, 0.0, 8.0)
            .head(-6.0, 0.0, -14.0)
            .wrists(-10.0, 0.0, 0.0);
        weapon(body, [0.06, 1.00, 0.46], [0.04, -0.72, 0.69])
            .plant_l([-0.15, GROUND + 0.14, 0.26])
            .toe_l(16.0)
            .plant_r([REAR[0], REAR[1], REAR[2] - 0.06])
            .toe_r(0.0)
    };

    // The plant. The point is in the floor ahead, the arms are straight, and
    // the whole body is about to swing around that fixed point.
    let planted = {
        let body = Pose::rest()
            .hips(0.0, -0.20, 0.20)
            .root(26.0, 0.0, 2.0)
            .spine(26.0, 0.0, 0.0)
            .chest(12.0, 0.0, 2.0)
            .head(-10.0, 0.0, -4.0)
            .wrists(-14.0, 0.0, 0.0);
        weapon(body, [0.04, 0.88, 0.60], [0.02, -0.80, 0.60])
            .plant_l([-0.15, GROUND, 0.40])
            .toe_l(-6.0)
            .plant_r([REAR[0], REAR[1] + 0.10, REAR[2]])
            .toe_floor_r()
    };

    // Over the top: the body swung up around the planted point, knees up,
    // hands climbing the haft. This is the frame the lift lands on.
    let over = {
        let body = Pose::rest()
            .hips(0.0, 0.04, 0.06)
            .root(-10.0, 0.0, -4.0)
            .spine(-14.0, 0.0, -4.0)
            .chest(-8.0, 0.0, -4.0)
            .head(-20.0, 0.0, 4.0)
            .wrists(4.0, 0.0, 0.0);
        weapon(body, [0.02, 1.30, 0.34], [0.0, -0.64, 0.77])
            .plant_l([-0.15, 0.42, 0.16])
            .toe_l(26.0)
            .plant_r([0.15, 0.34, -0.10])
            .toe_r(22.0)
    };

    // Off it, and airborne: the spear comes back up in front and the legs
    // gather. From here the falling clips take over.
    let airborne = {
        let body = Pose::rest()
            .hips(0.0, -0.02, 0.0)
            .root(-2.0, 0.0, 4.0)
            .spine(0.0, 0.0, 5.0)
            .chest(0.0, 0.0, 8.0)
            .head(-8.0, 0.0, -12.0)
            .wrists(-4.0, 0.0, 0.0);
        weapon(body, [0.02, 1.28, 0.22], [-0.08, 0.74, 0.67])
            .plant_l([-0.15, 0.36, 0.10])
            .toe_l(16.0)
            .plant_r([0.15, 0.28, -0.14])
            .toe_r(14.0)
    };

    let mut score = Score::new();
    score.key(0, ready(), CARRY);
    score.key(tell(windup), reach, Ease::LINEAR);
    score.key(contact, planted, Ease::OUT);
    score.key(part(contact, through, 0.8), over, Ease::IN);
    score.key(last, airborne, Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "The only clip in the set with nothing to hit. The animation is \
                the whole explanation of the move -- you are travelling, the \
                butt of the spear goes into the ground in front of you, and the \
                run becomes height -- so if it does not read as planting, the \
                fighter looks like they jumped for no reason. Both hands stay \
                on the haft and the haft stays pointed at the spot on the \
                floor; the hands climb it as the body swings up over the fixed \
                point, which is what the grip solver exists for. It ends \
                airborne and gathered, because the move does: the lift is \
                bigger than a jump and the falling clips take it from there."
            .into(),
        keys: score.0,
    }
}

// ---------------------------------------------------------------------------
// Backcut -- the second cut, the mirror of the first
// ---------------------------------------------------------------------------

/// The same descending diagonal, off the other shoulder.
///
/// `Plane::Diagonal(Hand::Left)`: head high over the left shoulder, level and
/// ahead through the middle, low past the right hip at the end. The first cut
/// finished low on the left, so this one starts from where that one left the
/// blade -- the hands lift up the left side and come down again, and nothing has
/// to be carried back to a chamber.
///
/// **It opens on the far side of somebody else's swing.** That is the rule for
/// every second hit in the chain and it is the whole read: a body that did not
/// come back to guard is a body that is not finished. Five frames of startup is
/// one key of lifting the blade and nothing else.
fn backcut() -> Recipe {
    let clip = Clip::ChampionBackcut;
    let (windup, contact, through) = clip.phases().expect("backcut animates a move");
    let last = clip.length().saturating_sub(1);

    // Where the first cut left everything: wound left, blade low past that hip,
    // rear heel still up from the turn that put it there.
    let carried = {
        let body = Pose::rest()
            .hips(-0.07, -0.16, 0.04)
            .root(11.0, 4.0, -22.0)
            .spine(17.0, 6.0, -18.0)
            .chest(6.0, 4.0, -20.0)
            .head(3.0, -2.0, 34.0)
            .wrists(-5.0, 0.0, 0.0);
        stand(
            weapon(body, [-0.34, 0.94, 0.36], [-0.57, -0.57, 0.59]),
            -2.0,
            0.08,
        )
    };

    // The only wind-up a chained hit gets: the blade lifted up the left side to
    // where it can come down again. The hands travel a short way and the head of
    // the weapon travels a long one, which is what a re-chamber is.
    let lifted = {
        let body = Pose::rest()
            .hips(-0.05, -0.10, -0.02)
            .root(2.0, 5.0, -26.0)
            .spine(6.0, 8.0, -18.0)
            .chest(2.0, 5.0, -18.0)
            .head(0.0, -3.0, 32.0)
            .wrists(-4.0, 0.0, 0.0);
        stand(
            weapon(body, [-0.36, 1.46, 0.10], [-0.56, 0.74, 0.37]),
            -6.0,
            0.06,
        )
    };

    // The push: rear leg driving, lead foot off the floor. Same frame the
    // simulation starts carrying the body.
    let push = {
        let body = Pose::rest()
            .hips(-0.04, -0.12, 0.0)
            .root(2.0, 4.0, -25.0)
            .spine(6.0, 6.0, -17.0)
            .chest(2.0, 4.0, -17.0)
            .head(1.0, -3.0, 30.0)
            .wrists(-5.0, 0.0, 0.0);
        step_stance(
            weapon(body, [-0.38, 1.42, 0.20], [-0.58, 0.68, 0.45]),
            0.20,
            0.09,
        )
    };

    // **The start of the arc**, mirrored: high and out to the left, coming down.
    let high = {
        let body = Pose::rest()
            .hips(-0.03, -0.10, 0.04)
            .root(3.0, 3.0, -22.0)
            .spine(7.0, 5.0, -15.0)
            .chest(3.0, 4.0, -15.0)
            .head(2.0, -2.0, 26.0)
            .wrists(-4.0, 0.0, 0.0);
        step_stance(
            weapon(body, [-0.38, 1.36, 0.36], [-0.57, 0.57, 0.59]),
            0.26,
            0.0,
        )
    };

    // Level and dead ahead, a hand higher than the opener's middle. That gap is
    // what stops the two cuts looking like the same swing played twice.
    let level = {
        let body = Pose::rest()
            .hips(0.02, -0.11, 0.06)
            .root(5.0, 0.0, 2.0)
            .spine(9.0, -2.0, 8.0)
            .chest(4.0, -1.0, 10.0)
            .head(3.0, 0.0, -12.0)
            .wrists(-3.0, 0.0, 0.0);
        step_stance(
            weapon(body, [-0.04, 1.26, 0.46], [-0.02, 0.04, 1.0]),
            0.20,
            0.0,
        )
    };

    // **The end of it**: low, out past the right hip, wound the way the third
    // cut comes out of.
    let low = {
        let body = Pose::rest()
            .hips(0.07, -0.13, 0.04)
            .root(8.0, -4.0, 24.0)
            .spine(12.0, -6.0, 19.0)
            .chest(5.0, -4.0, 20.0)
            .head(3.0, 2.0, -34.0)
            .wrists(-5.0, 0.0, 0.0);
        step_stance(
            weapon(body, [0.34, 0.96, 0.36], [0.57, -0.57, 0.59]),
            0.12,
            0.0,
        )
    };

    let mut score = Score::new();
    score.key(0, carried, Ease::LINEAR);
    score.key(tell(windup), lifted, Ease::LINEAR);
    score.key(part(tell(windup), windup, 0.75), push, Ease::LINEAR);
    score.key(contact, high, Ease::LINEAR);
    score.key(part(contact, through, 0.5), level, Ease::LINEAR);
    score.key(through, low, Ease::LINEAR);
    score.key(last, ready(), Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "The second stroke, and the clip that decides whether the chain \
                reads as a chain at all. It opens exactly where the opener \
                finished -- blade low past the left hip, hips wound that way, \
                rear heel still up -- so whatever preceded it crossfades into \
                something plainly mid-motion. Five frames of startup is one key \
                of lifting the blade up the left side, which is the shortest \
                honest re-chamber there is: the hands move a hand's width and \
                the head of the weapon crosses two metres. Then the mirror of \
                the first cut, a hand higher through the middle so the two do \
                not read as one swing played twice, and it finishes wound right \
                -- which is where the third one starts."
            .into(),
        keys: score.0,
    }
}

// ---------------------------------------------------------------------------
// Upcut -- the sword's finisher
// ---------------------------------------------------------------------------

/// A rising cut out of a short dash, up the line the second one came down.
///
/// `Plane::Diagonal(Hand::Left)` with the arc reversed: the head starts low past
/// the right hip -- exactly where Backcut left it -- and finishes high over the
/// left shoulder. Three strokes, one figure: down from the right, down from the
/// left, up the same line. The blade never has to be carried back to a chamber
/// and the player never has to be told which hit they are on.
///
/// **The feet commit**, which is the rule for every finisher in the chain, and
/// here the simulation commits them for you: `Move::step` drives the body better
/// than a metre while the cut travels, so this is a small dash with a blade
/// coming up out of it rather than a swing thrown from a standing start. It is
/// the only grounded sword move that does not put the feet back where it found
/// them, and that is the tell -- if you see the feet leave, the string is over
/// one way or the other.
fn crescent() -> Recipe {
    let clip = Clip::ChampionUpcut;
    let (windup, contact, through) = clip.phases().expect("the upcut animates a move");
    let last = clip.length().saturating_sub(1);

    // Sunk low over the right hip with the blade dropped almost to the floor and
    // the point behind. Nine frames of startup, and the whole of it is getting
    // *under* the cut: a rising strike thrown from chest height has nowhere to
    // rise from.
    let sunk = {
        let body = Pose::rest()
            .hips(0.06, -0.24, 0.06)
            .root(16.0, -5.0, 22.0)
            .spine(20.0, -7.0, 16.0)
            .chest(9.0, -5.0, 16.0)
            .head(-6.0, 3.0, -28.0)
            .wrists(-4.0, 0.0, 0.0);
        stand(
            weapon(body, [0.26, 0.82, 0.10], [0.52, -0.68, 0.52]),
            -12.0,
            0.06,
        )
    };

    // **The start of the arc**: low and out past the right hip, blade already
    // climbing, both feet leaving as the dash takes hold. The body is a third of
    // the way through a metre of ground.
    let under = {
        let body = Pose::rest()
            .hips(0.04, -0.20, 0.14)
            .root(14.0, -3.0, 16.0)
            .spine(18.0, -5.0, 12.0)
            .chest(8.0, -4.0, 12.0)
            .head(-2.0, 2.0, -20.0)
            .wrists(-2.0, 0.0, 0.0);
        step_stance(
            weapon(body, [0.32, 0.88, 0.40], [0.54, -0.54, 0.65]),
            0.34,
            0.0,
        )
    };

    // Through level and still climbing, hips driving up out of the sink. This is
    // the frame the volume is furthest out in front.
    let driving = {
        let body = Pose::rest()
            .hips(0.0, -0.10, 0.16)
            .root(3.0, 0.0, 2.0)
            .spine(5.0, 0.0, 4.0)
            .chest(1.0, 0.0, 4.0)
            .head(2.0, 0.0, -6.0)
            .wrists(0.0, 0.0, 0.0);
        step_stance(
            weapon(body, [0.06, 1.24, 0.50], [0.04, 0.10, 0.99]),
            0.22,
            0.0,
        )
    };

    // **The end of it**: the blade overhead and out to the left, the body stood
    // all the way up and then some. Everything that was sunk is now extended,
    // which is what a launcher has to look like from the other side of the arena.
    let over = {
        let body = Pose::rest()
            .hips(-0.06, 0.02, 0.10)
            .root(-8.0, 4.0, -18.0)
            .spine(-10.0, 6.0, -14.0)
            .chest(-5.0, 4.0, -16.0)
            .head(10.0, -2.0, 26.0)
            .wrists(-2.0, 0.0, 0.0);
        step_stance(
            weapon(body, [-0.24, 1.66, 0.28], [-0.54, 0.62, 0.57]),
            0.10,
            0.0,
        )
    };

    // Coming down off it. The weight arrives on the lead foot and the rear one
    // has to be gathered in from where the dash left it behind.
    let land = {
        let body = Pose::rest()
            .hips(-0.02, -0.14, 0.04)
            .root(6.0, 2.0, -10.0)
            .spine(9.0, 3.0, -8.0)
            .chest(3.0, 2.0, -6.0)
            .head(0.0, 0.0, 14.0)
            .wrists(-7.0, 0.0, 0.0);
        stand(
            weapon(body, [-0.08, 1.24, 0.24], [-0.24, 0.72, 0.65]),
            -4.0,
            0.07,
        )
    };

    let mut score = Score::new();
    score.key(0, ready(), Ease::IN);
    // A real hold at the bottom. Nine frames is enough to be read, and the sink
    // is the read: this is the one sword move worth blocking, so it is allowed
    // to say so.
    score.key(part(tell(windup), windup, 0.55), sunk, Ease::SNAP);
    score.key(contact, under, Ease::LINEAR);
    score.key(part(contact, through, 0.5), driving, Ease::LINEAR);
    score.key(through, over, Ease::LINEAR);
    score.key(part(through, last, 0.4), land, Ease::OUT);
    score.key(last, ready(), Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "The finisher, and the third stroke of the figure: down from the \
                right, down from the left, then up the line the second one came \
                down. It starts where Backcut finished -- blade low past the \
                right hip -- and ends with it overhead and out to the left. The \
                startup is spent getting *under* the cut rather than winding it \
                back, because a rising strike thrown from chest height has \
                nothing to rise from, and the sink is also the telegraph: nine \
                frames of a body dropping is the most readable thing in the \
                sword's row and this is the one hit of the three worth blocking. \
                The simulation dashes the body better than a metre while the \
                blade climbs, so both feet leave -- which is the chain's tell \
                that a finisher has been thrown."
            .into(),
        keys: score.0,
    }
}

// ---------------------------------------------------------------------------
// Uproot -- the hammer's second hit
// ---------------------------------------------------------------------------

/// The head torn back out of the floor and up through them.
///
/// **The slam played backwards, and it has to read that way.** The first key is
/// where the overhead left the body -- folded over a weapon lying on the ground
/// -- and the whole move is levering that weight back up through the space in
/// front. The sim sweeps it with a negative arc, so the hitbox climbs from the
/// floor to overhead and catches anybody who stepped in under the first swing.
///
/// It is the one chained hit that is genuinely easier than its opener, and the
/// pose is why: a weight already at the bottom of its arc wants to come up.
fn uproot() -> Recipe {
    let clip = Clip::ChampionUproot;
    let (windup, contact, through) = clip.phases().expect("uproot animates a move");
    let last = clip.length().saturating_sub(1);

    // Where a slam leaves you: knees bent, chest over the knees, the head of
    // the weapon on the ground out in front.
    let down = {
        let body = Pose::rest()
            .hips(0.0, -0.28, 0.12)
            .root(31.0, 0.0, -2.0)
            .spine(33.0, 0.0, -4.0)
            .chest(14.0, 0.0, -4.0)
            .head(8.0, 0.0, 6.0)
            .wrists(-10.0, 0.0, 0.0);
        weapon(body, [0.02, 0.50, 0.62], [0.0, -0.36, 0.93])
            .plant_l([-0.16, GROUND, 0.26])
            .toe_l(-2.0)
            .plant_r([REAR[0], REAR[1], REAR[2] - 0.04])
            .toe_r(0.0)
    };

    // Digging under it: the hands drop and the knees take another few
    // centimetres, which is what a person does to lift something rather than to
    // swing it. One key, because thirteen frames is not long enough for two.
    let dig = {
        let body = Pose::rest()
            .hips(0.02, -0.33, 0.06)
            .root(34.0, -2.0, 4.0)
            .spine(36.0, -3.0, 2.0)
            .chest(15.0, -2.0, 2.0)
            .head(4.0, 0.0, -4.0)
            .wrists(-6.0, 0.0, 0.0);
        weapon(body, [0.06, 0.40, 0.50], [0.10, -0.52, 0.85])
            .plant_l([-0.16, GROUND, 0.24])
            .toe_l(-6.0)
            .plant_r([REAR[0], REAR[1], REAR[2] - 0.02])
            .toe_r(0.0)
    };

    // The haul begins: hips driving, the head of the weapon scraping up off the
    // floor. The legs are doing this, not the arms.
    let heave = {
        let body = Pose::rest()
            .hips(0.0, -0.20, 0.10)
            .root(22.0, -1.0, 6.0)
            .spine(24.0, -2.0, 4.0)
            .chest(10.0, -1.0, 4.0)
            .head(-2.0, 0.0, -14.0)
            .wrists(-4.0, 0.0, 0.0);
        stand(
            weapon(body, [0.06, 0.74, 0.50], [0.06, 0.04, 1.0]),
            -8.0,
            0.03,
        )
    };

    // **The start of the arc**: the head still down by the floor and already
    // travelling, the body beginning to step in behind it. The simulation carries
    // you half a metre through this, which is what turns a short swing into
    // something that arrives -- see `step_stance`.
    let scrape = {
        let body = Pose::rest()
            .hips(0.02, -0.26, 0.12)
            .root(26.0, -1.0, 6.0)
            .spine(28.0, -2.0, 4.0)
            .chest(12.0, -1.0, 4.0)
            .head(0.0, 0.0, -12.0)
            .wrists(-4.0, 0.0, 0.0);
        step_stance(
            weapon(body, [0.06, 0.66, 0.44], [0.06, -0.52, 0.85]),
            0.22,
            0.0,
        )
    };

    // Through the middle of it: the head ripped up through the front of them,
    // body long, weight arriving. Level and driving forward, which is the frame
    // that says this hit shoves rather than lifts.
    let tear = {
        let body = Pose::rest()
            .hips(0.0, -0.08, 0.12)
            .root(4.0, 0.0, 2.0)
            .spine(2.0, 0.0, 0.0)
            .chest(0.0, 0.0, 0.0)
            .head(-10.0, 0.0, -6.0)
            .wrists(-2.0, 0.0, 0.0);
        step_stance(
            weapon(body, [0.04, 1.14, 0.44], [0.04, 0.24, 0.97]),
            0.16,
            0.0,
        )
    };

    // **The end of it**: the head up and still out in front, both arms high, the
    // body long under it. It finishes ahead of the shoulders rather than behind
    // them, which is the difference between a hammer that throws somebody up and
    // one that throws them back -- and this is the one that throws them back.
    let over = {
        let body = Pose::rest()
            .hips(0.0, -0.04, 0.06)
            .root(-8.0, 0.0, 4.0)
            .spine(-12.0, 0.0, 2.0)
            .chest(-8.0, 0.0, 2.0)
            .head(-20.0, 0.0, -4.0)
            .wrists(2.0, 0.0, 0.0);
        step_stance(
            weapon(body, [0.02, 1.48, 0.34], [0.0, 0.75, 0.66]),
            0.08,
            0.0,
        )
    };

    // Bringing it back down in front. Heavy, and it should look like catching
    // something rather than like lowering it.
    let catch = {
        let body = Pose::rest()
            .hips(0.0, -0.12, 0.02)
            .root(10.0, 0.0, 8.0)
            .spine(12.0, 0.0, 6.0)
            .chest(5.0, 0.0, 8.0)
            .head(-4.0, 0.0, -18.0)
            .wrists(-8.0, 0.0, 0.0);
        stand(
            weapon(body, [0.04, 1.26, 0.28], [-0.04, 0.64, 0.77]),
            -4.0,
            0.04,
        )
    };

    let mut score = Score::new();
    score.key(0, down, CARRY);
    score.key(tell(windup), dig, Ease::SMOOTH);
    score.key(part(tell(windup), windup, 0.65), heave, Ease::IN);
    score.key(contact, scrape, Ease::LINEAR);
    score.key(part(contact, through, 0.5), tear, Ease::LINEAR);
    score.key(through, over, Ease::LINEAR);
    score.key(part(through, last, 0.5), catch, CARRY);
    score.key(last, ready(), Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::HEAVY,
        notes: "The slam played backwards, and the second hit of the hammer \
                chain. It opens exactly where the overhead left the body -- \
                folded over a weapon lying on the floor -- so the two of them \
                crossfade into one continuous piece of work, and it climbs from \
                there. The startup is a dig rather than a wind-up: the hands drop \
                and the knees take a few more centimetres, which is what a person \
                does to lift something rather than to swing it. Legs, not arms, \
                and it is the one chained hit that is easier than its own opener \
                -- a weight already at the bottom of its arc wants to come up. \
                It is the short one: the shortest reach in the chain, and it \
                steps half a metre so that the reach is still enough to catch \
                whoever the slam shoved. And it finishes with the head **ahead \
                of** the shoulders rather than behind them, because this is the \
                hammer hit that shoves people back rather than up -- the one \
                that throws them up is the finisher, and it comes down."
            .into(),
        keys: score.0,
    }
}

// ---------------------------------------------------------------------------
// Earthbreaker -- the hammer's finisher
// ---------------------------------------------------------------------------

/// The whole body over the head and into the floor. The guard break.
///
/// Twenty frames of startup is the longest telegraph in the game and it is
/// supposed to be: this one goes **through a shield**, so the only defence
/// against it is not being there, and the only way not being there can be a
/// real decision is if the opponent can see it coming from across the arena.
/// So the wind-up takes the weapon all the way back and up and then *holds*.
///
/// The feet commit, as they do in every finisher: a step in on the wind-up that
/// is not taken back.
fn earthbreaker() -> Recipe {
    let clip = Clip::ChampionEarthbreaker;
    let (windup, contact, through) = clip.phases().expect("earthbreaker animates a move");
    let last = clip.length().saturating_sub(1);

    // The step in. The lead foot goes forward and the weight starts to rise --
    // the first frame of a silhouette that is about to get very tall.
    let step = {
        let body = Pose::rest()
            .hips(0.0, -0.09, 0.06)
            .root(4.0, -2.0, 12.0)
            .spine(6.0, -3.0, 8.0)
            .chest(2.0, -2.0, 10.0)
            .head(-8.0, 0.0, -22.0)
            .wrists(-4.0, 0.0, 0.0);
        weapon(body, [0.10, 1.32, 0.10], [0.24, 0.88, -0.41])
            .plant_l([-0.15, GROUND, 0.30])
            .toe_l(-4.0)
            .plant_r([REAR[0], REAR[1] + 0.03, REAR[2]])
            .toe_floor_r()
    };

    // All the way back and over, body arched, arms straight overhead and
    // behind. This is as tall as anything in the game gets.
    let wind = {
        let body = Pose::rest()
            .hips(0.0, -0.02, -0.10)
            .root(-14.0, -2.0, 6.0)
            .spine(-18.0, -3.0, 4.0)
            .chest(-11.0, -2.0, 6.0)
            .head(-26.0, 0.0, -10.0)
            .wrists(10.0, 0.0, 0.0);
        weapon(body, [0.0, 1.78, -0.18], [-0.04, 0.80, -0.60])
            .plant_l([-0.15, GROUND, 0.30])
            .toe_l(-8.0)
            .plant_r([REAR[0], REAR[1] + 0.07, REAR[2]])
            .toe_floor_r()
    };

    // Held at the top. **The whole point of the move.** A guard break the
    // opponent cannot read is a guard break that punishes having a shield
    // rather than a guard break that beats one.
    let hang = {
        let body = Pose::rest()
            .hips(0.0, -0.04, -0.12)
            .root(-16.0, -1.0, 4.0)
            .spine(-19.0, -2.0, 2.0)
            .chest(-12.0, -1.0, 4.0)
            .head(-28.0, 0.0, -8.0)
            .wrists(12.0, 0.0, 0.0);
        weapon(body, [-0.02, 1.80, -0.22], [-0.06, 0.78, -0.62])
            .plant_l([-0.15, GROUND, 0.30])
            .toe_l(-10.0)
            .plant_r([REAR[0], REAR[1] + 0.08, REAR[2]])
            .toe_floor_r()
    };

    // Halfway down, and the fastest frame in the clip: the body passing
    // through upright with the weapon already ahead of it. A weight this size
    // does not go from held to landed in one interval -- and a torso that tries
    // to reads as a pop rather than as a slam.
    let fall = {
        let body = Pose::rest()
            .hips(0.0, -0.16, 0.10)
            .root(14.0, 0.0, 0.0)
            .spine(14.0, 0.0, -2.0)
            .chest(6.0, 0.0, -2.0)
            .head(-6.0, 0.0, 2.0)
            .wrists(-4.0, 0.0, 0.0);
        weapon(body, [0.0, 1.30, 0.34], [0.0, 0.42, 0.91])
            // **Both feet off the floor.** The simulation dashes the body a metre
            // and a half in four frames, ending on the frame the head lands, and
            // nothing that crosses that much ground that fast is walking.
            .plant_l([-0.15, GROUND + 0.16, 0.44])
            .toe_l(-20.0)
            .plant_r([REAR[0], REAR[1] + 0.20, REAR[2] - 0.22])
            .toe_r(-30.0)
    };

    // Everything into the floor. Knees collapsing under it, the head of the
    // weapon through the ground line in front, both heels driven down.
    let crack = {
        let body = Pose::rest()
            .hips(0.0, -0.34, 0.18)
            .root(36.0, 0.0, -4.0)
            .spine(38.0, 0.0, -6.0)
            .chest(17.0, 0.0, -8.0)
            .head(10.0, 0.0, 10.0)
            .wrists(-16.0, 0.0, 0.0);
        weapon(body, [0.0, 0.46, 0.66], [0.0, -0.80, 0.60])
            // Landing on the frame the head lands. The dash arrives and the hammer
            // arrives together, which is the whole of what this move is: the
            // ground closed is not a separate beat from the hit.
            .plant_l([-0.16, GROUND, 0.40])
            .toe_l(-2.0)
            .plant_r([REAR[0], REAR[1] + 0.04, REAR[2] - 0.30])
            .toe_floor_r()
    };

    // The floor has it. Nothing moving -- the beat that says the arena took the
    // hit as well as whoever was standing there.
    let crater = {
        let body = Pose::rest()
            .hips(0.0, -0.37, 0.14)
            .root(40.0, 0.0, -2.0)
            .spine(41.0, 0.0, -4.0)
            .chest(16.0, 0.0, -4.0)
            .head(12.0, 0.0, 6.0)
            .wrists(-12.0, 0.0, 0.0);
        weapon(body, [0.0, 0.36, 0.70], [0.0, -0.42, 0.91])
            .plant_l([-0.16, GROUND, 0.36])
            .toe_l(0.0)
            .plant_r([REAR[0], REAR[1], REAR[2] - 0.14])
            .toe_r(0.0)
    };

    // Twenty-eight frames of getting back up, and it should look like work.
    let haul = {
        let body = Pose::rest()
            .hips(0.0, -0.18, 0.06)
            .root(18.0, -1.0, 10.0)
            .spine(20.0, -2.0, 8.0)
            .chest(9.0, -1.0, 10.0)
            .head(2.0, 0.0, -16.0)
            .wrists(-8.0, 0.0, 0.0);
        weapon(body, [0.04, 0.92, 0.46], [0.02, 0.44, 0.90])
            .plant_l([-0.16, GROUND, 0.28])
            .toe_l(-4.0)
            .plant_r([REAR[0], REAR[1] + 0.04, REAR[2]])
            .toe_floor_r()
    };

    let mut score = Score::new();
    score.key(0, ready(), Ease::OUT);
    score.key(tell(windup), step, Ease::SMOOTH);
    score.key(part(tell(windup), windup, 0.45), wind, Ease::OUT);
    score.key(part(tell(windup), windup, 0.75), hang, Ease::SMOOTH);
    // Inside the wind-up rather than between it and contact: `part(windup,
    // contact, 0.5)` rounds up onto the contact frame itself, and a key landing
    // there drops the contact pose -- which is the one key in a clip that must
    // not move.
    score.key(part(tell(windup), windup, 0.95), fall, Ease::IN);
    score.key(contact, crack, Ease::STRIKE);
    score.key(through, crater, Ease::OUT);
    score.key(part(through, last, 0.5), haul, CARRY);
    score.key(last, ready(), Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::HEAVY,
        notes: "The guard break, and the longest telegraph in the game. It goes \
                through a shield, so the only answer to it is not being there, \
                and not being there is only a decision if you can see it coming \
                -- which is what the twenty frames buy. The wind-up takes the \
                weapon all the way back and over until this is the tallest \
                silhouette anything in the game makes, and then it *holds*, \
                which is the part that turns a long startup into a readable \
                one. Contact puts the head through the ground line with the \
                knees collapsing under it, and there is a beat on the floor \
                before the recovery begins: the arena took that one too. The \
                feet leave entirely on the way in -- the simulation dashes the \
                body a metre and a half in four frames, finishing on the frame the \
                head lands, so the ground closed is not a separate beat from the \
                hit -- and they never come back to where they started, because \
                every finisher in this chain commits them. What it does *after* \
                contact is the other half of the move and is not in this clip: it \
                throws whoever it caught into the air, and if the jump button went \
                down while this was winding up, the Champion goes up with them and \
                the airborne clips take it from here."
            .into(),
        keys: score.0,
    }
}

// ---------------------------------------------------------------------------
// Skewer -- the second thrust, and the heaviest stagger in the kit
// ---------------------------------------------------------------------------

/// A long wind-up, then a dash with everything behind the point.
///
/// The one link in the chain whose second hit is **slower** than its opener, and
/// deliberately: sixteen frames of startup is past a human reaction, so this is a
/// thing you get to see coming and have to answer. What it buys is the biggest
/// on-hit advantage the class has -- land it and the finisher is guaranteed --
/// and better than a metre and a half of ground closed while it travels.
///
/// So the clip is two halves that could not be mistaken for each other. The first
/// is a coil: both hands back on the haft, the point drawn past the hip, the
/// weight all on the rear leg, and it *holds* there. The second is the dash --
/// `Move::step` carries the body and both feet leave the floor, because a body
/// crossing that much ground in six frames is not walking.
fn skewer() -> Recipe {
    let clip = Clip::ChampionSkewer;
    let (windup, contact, through) = clip.phases().expect("skewer animates a move");
    let last = clip.length().saturating_sub(1);

    // Where the jab left it: front hand out, pommel at the hip. The second hand
    // coming onto the haft is the first thing that changes, and it is the read --
    // one hand is a poke, two hands is a commitment.
    let carried = {
        let body = Pose::rest()
            .hips(0.0, -0.13, 0.14)
            .root(9.0, -1.0, 2.0)
            .spine(11.0, -1.0, -2.0)
            .chest(5.0, -1.0, -2.0)
            .head(-4.0, 0.0, -4.0)
            .wrists(0.0, 0.0, 0.0);
        stand(
            body.reach_r([0.28, 1.26, 0.84]).reach_l([0.18, 1.06, 0.10]),
            0.0,
            0.05,
        )
    };

    // The coil. Both hands drawn right back past the hip, the point still level
    // and still on line -- a wind-up that re-aims gives the defender two reads
    // and this one is only allowed to give them one. The weight is entirely on
    // the rear leg, which is the frame that says a dash is coming.
    let coiled = {
        let body = Pose::rest()
            .hips(0.07, -0.20, -0.14)
            .root(-2.0, -5.0, 30.0)
            .spine(2.0, -8.0, 22.0)
            .chest(0.0, -6.0, 22.0)
            .head(-4.0, 2.0, -46.0)
            .wrists(-10.0, 0.0, 0.0);
        weapon(body, [0.20, 1.10, -0.20], [-0.14, 0.06, 0.99])
            .plant_l([-0.13, GROUND + 0.02, 0.10])
            .toe_l(-16.0)
            .plant_r([REAR[0], REAR[1], REAR[2] - 0.06])
            .toe_r(0.0)
    };

    // The rear leg fires and the lead foot leaves the floor. Two frames before
    // the point arrives, which is what makes the dash visible as a dash rather
    // than as a thrust that happened to move.
    let fired = {
        let body = Pose::rest()
            .hips(0.04, -0.20, 0.06)
            .root(10.0, -3.0, 20.0)
            .spine(12.0, -5.0, 14.0)
            .chest(5.0, -3.0, 14.0)
            .head(-2.0, 0.0, -30.0)
            .wrists(-8.0, 0.0, 0.0);
        weapon(body, [0.16, 1.14, 0.06], [-0.08, 0.04, 1.0])
            .plant_l([-0.15, GROUND + 0.14, 0.30])
            .toe_l(-18.0)
            .plant_r([REAR[0], REAR[1] + 0.10, REAR[2] - 0.20])
            .toe_floor_r()
    };

    // Halfway out, mid-flight. The hands cross most of a metre between the coil
    // and the point landing, and the whole of that gap is authored at one speed
    // for the reason the sweep's follow-through is: a hand asked to cross forty
    // centimetres in one interval does not look fast, it looks like a cut in the
    // film. The dash supplies the acceleration; the arms only have to be honest
    // about where they are.
    let flying = {
        let body = Pose::rest()
            .hips(0.02, -0.19, 0.18)
            .root(13.0, -1.0, 12.0)
            .spine(15.0, -2.0, 6.0)
            .chest(6.0, -1.0, 4.0)
            .head(-3.0, 0.0, -12.0)
            .wrists(-5.0, 0.0, 0.0);
        weapon(body, [0.06, 1.20, 0.64], [-0.04, 0.03, 1.0])
            .plant_l([-0.16, GROUND + 0.16, 0.36])
            .toe_l(-20.0)
            .plant_r([REAR[0], REAR[1] + 0.18, REAR[2] - 0.28])
            .toe_r(-26.0)
    };

    // **Contact.** One line from the rear heel to the point, both feet off the
    // floor, the shoulders come through behind the shaft. This is the only pose
    // in the class where the arms are meant to read as straight.
    let thrust = {
        let body = Pose::rest()
            .hips(0.0, -0.18, 0.28)
            .root(15.0, 0.0, -4.0)
            .spine(17.0, 2.0, -8.0)
            .chest(7.0, 2.0, -12.0)
            .head(-4.0, 0.0, 16.0)
            .wrists(-2.0, 0.0, 0.0);
        weapon(body, [0.02, 1.22, 0.92], [-0.02, 0.02, 1.0])
            .plant_l([-0.16, GROUND + 0.06, 0.40])
            .toe_l(-8.0)
            .plant_r([REAR[0], REAR[1] + 0.16, REAR[2] - 0.34])
            .toe_floor_r()
    };

    // Arriving. The dash lands and the arms give: the difference between this and
    // contact is the difference between a strike and the end of one.
    let spent = {
        let body = Pose::rest()
            .hips(-0.02, -0.26, 0.30)
            .root(18.0, 0.0, -2.0)
            .spine(18.0, 3.0, -6.0)
            .chest(8.0, 2.0, -10.0)
            .head(-6.0, 0.0, 12.0)
            .wrists(-4.0, 0.0, 0.0);
        weapon(body, [0.0, 1.14, 0.86], [-0.06, -0.10, 0.99])
            .plant_l([-0.17, GROUND, 0.56])
            .toe_l(0.0)
            .plant_r([REAR[0], REAR[1] + 0.06, REAR[2] - 0.12])
            .toe_floor_r()
    };

    // Hauling back out of it and gathering the rear foot in from where the dash
    // left it. A dash that recovers by sliding its feet home is a dash on ice.
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
    score.key(0, carried, CARRY);
    // A real hold in the coil. Sixteen frames is past a reaction, so the pose is
    // allowed -- and required -- to sit there and be looked at.
    score.key(part(tell(windup), windup, 0.25), coiled, Ease::SMOOTH);
    score.key(part(tell(windup), windup, 0.67), fired, Ease::SNAP);
    score.key(part(tell(windup), windup, 0.9), flying, Ease::LINEAR);
    score.key(contact, thrust, Ease::LINEAR);
    score.key(through, spent, CARRY);
    score.key(part(through, last, 0.45), pull, CARRY);
    score.key(last, ready(), Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "Two halves that cannot be mistaken for each other, which is the \
                whole design of the move: sixteen frames of startup is past a \
                human reaction, so the opponent gets to see this coming and has \
                to answer it, and what it pays for that is the biggest on-hit \
                advantage in the kit. The first half is the coil -- the second \
                hand arriving on the haft, which is the read, because one hand is \
                a poke and two is a commitment -- and it *holds*, aimed, without \
                re-aiming, because a wind-up that re-aims hands the defender two \
                reads instead of one. The second half is the dash: the simulation \
                carries the body nearly two metres in six frames, so both feet \
                leave the floor, and the lead one leaves two frames before the \
                point arrives so that what you see is a dash rather than a thrust \
                that happened to travel."
            .into(),
        keys: score.0,
    }
}

// ---------------------------------------------------------------------------
// Whirl -- the spear's finisher, and the one move that clears the ring
// ---------------------------------------------------------------------------

/// The shaft swept flat all the way round the body, low.
///
/// `Shape::Swing(Plane::Flat)` with an arc of nearly three quarters of a turn:
/// the head starts behind the right shoulder, comes round the whole front at
/// about knee height, and finishes behind the left. The simulation hangs it off
/// `tuning::sweep_height`, which is a low cut, so this is a leg sweep with a
/// three-metre pole -- and it is the only thing in the class that threatens the
/// ground *behind* you.
///
/// That is what the finisher is for. The spear's first two hits are about holding
/// people at a distance; this is what you throw when one of them has closed
/// anyway, and the knockback puts them back out where the other two work.
///
/// **A body cannot turn that far.** The root's yaw runs out around sixty-five
/// degrees, and it does not need to go further, because a sweep is mostly *the
/// weapon pointing somewhere else* with the hands near the sternum. What the body
/// supplies is the drive: a hard wind to the right, both feet turning on their
/// toes, and a spine that unwinds through square and keeps going.
fn whirl() -> Recipe {
    let clip = Clip::ChampionWhirl;
    let (windup, contact, through) = clip.phases().expect("the whirl animates a move");
    let last = clip.length().saturating_sub(1);

    // Wound hard right and dropped low, shaft swung back behind that shoulder and
    // tipped down. The knees are what make this readable: a spin thrown from
    // standing height is a spin that misses everybody.
    let wound = {
        let body = Pose::rest()
            .hips(0.08, -0.26, -0.06)
            .root(10.0, -6.0, 40.0)
            .spine(14.0, -9.0, 26.0)
            .chest(6.0, -6.0, 24.0)
            .head(-2.0, 4.0, -52.0)
            .wrists(-8.0, 0.0, 0.0);
        weapon(body, [0.22, 0.98, -0.06], [0.70, -0.24, -0.67])
            .plant_l([LEAD[0] + 0.02, GROUND, LEAD[2] - 0.02])
            .toe_l(-6.0)
            .plant_r([REAR[0], REAR[1] + 0.05, REAR[2]])
            .toe_floor_r()
    };

    // **The start of the arc**: the head still behind the right shoulder and
    // already travelling. Both heels are up -- the turn is on the toes, which is
    // the only way a body turns this far without a foot leaving the spot it is
    // standing on.
    let behind = {
        let body = Pose::rest()
            .hips(0.06, -0.28, 0.0)
            .root(11.0, -5.0, 34.0)
            .spine(15.0, -8.0, 22.0)
            .chest(6.0, -5.0, 20.0)
            .head(0.0, 3.0, -44.0)
            .wrists(-6.0, 0.0, 0.0);
        weapon(body, [0.20, 0.94, 0.02], [0.78, -0.18, -0.60])
            .plant_l([LEAD[0] + 0.02, GROUND + 0.03, LEAD[2] - 0.02])
            .toe_floor_l()
            .plant_r([REAR[0], REAR[1] + 0.07, REAR[2]])
            .toe_floor_r()
    };

    // Round to the side, and the lowest the head gets. Knee height and coming
    // across, which is the frame a crouching opponent finds out about.
    let side = {
        let body = Pose::rest()
            .hips(0.0, -0.30, 0.06)
            .root(13.0, 0.0, 6.0)
            .spine(17.0, 0.0, 0.0)
            .chest(7.0, 0.0, -2.0)
            .head(0.0, 0.0, -6.0)
            .wrists(-4.0, 0.0, 0.0);
        weapon(body, [0.06, 0.92, 0.16], [0.62, -0.20, 0.76])
            .plant_l([LEAD[0] + 0.02, GROUND + 0.02, LEAD[2] - 0.02])
            .toe_floor_l()
            .plant_r([REAR[0], REAR[1] + 0.08, REAR[2]])
            .toe_floor_r()
    };

    // Through the front, dead ahead and low. Hips already past square: the hips
    // are what swing a pole this long, and a sweep whose hips arrive with the
    // hands is a sweep thrown with the arms.
    let front = {
        let body = Pose::rest()
            .hips(-0.06, -0.30, 0.08)
            .root(13.0, 4.0, -26.0)
            .spine(17.0, 5.0, -20.0)
            .chest(7.0, 3.0, -20.0)
            .head(0.0, -2.0, 30.0)
            .wrists(-4.0, 0.0, 0.0);
        weapon(body, [-0.10, 0.94, 0.20], [-0.34, -0.18, 0.92])
            .plant_l([LEAD[0] + 0.02, GROUND + 0.02, LEAD[2] - 0.02])
            .toe_floor_l()
            .plant_r([REAR[0], REAR[1] + 0.08, REAR[2]])
            .toe_floor_r()
    };

    // **The end of it**: the head away round to the left and behind, everything
    // wound the other way, still low. The pole has been most of the way round a
    // circle and the body has turned as far as a body turns.
    let past = {
        let body = Pose::rest()
            .hips(-0.09, -0.27, 0.0)
            .root(11.0, 6.0, -40.0)
            .spine(15.0, 8.0, -26.0)
            .chest(6.0, 5.0, -24.0)
            .head(-2.0, -4.0, 50.0)
            .wrists(-6.0, 0.0, 0.0);
        weapon(body, [-0.20, 0.96, 0.04], [-0.76, -0.16, -0.62])
            .plant_l([LEAD[0] + 0.02, GROUND + 0.02, LEAD[2] - 0.04])
            .toe_floor_l()
            .plant_r([REAR[0], REAR[1] + 0.06, REAR[2] - 0.02])
            .toe_floor_r()
    };

    // Standing up out of it and unwinding. Both heels come back down, which is
    // the frame the turn is over.
    let rise = {
        let body = Pose::rest()
            .hips(-0.04, -0.16, 0.0)
            .root(6.0, 3.0, -18.0)
            .spine(9.0, 4.0, -10.0)
            .chest(3.0, 2.0, -8.0)
            .head(-2.0, 0.0, 22.0)
            .wrists(-8.0, 0.0, 0.0);
        stand(
            weapon(body, [-0.08, 1.12, 0.18], [-0.40, 0.34, 0.85]),
            -2.0,
            0.06,
        )
    };

    let mut score = Score::new();
    score.key(0, ready(), Ease::IN);
    score.key(part(tell(windup), windup, 0.5), wound, Ease::SNAP);
    score.key(contact, behind, Ease::LINEAR);
    score.key(part(contact, through, 0.34), side, Ease::LINEAR);
    score.key(part(contact, through, 0.67), front, Ease::LINEAR);
    score.key(through, past, Ease::LINEAR);
    score.key(part(through, last, 0.45), rise, Ease::OUT);
    score.key(last, ready(), Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "The one move in the class that threatens behind you. Nearly three \
                quarters of a turn, hung off the low cut height, so it is a leg \
                sweep with a three-metre pole: the head starts behind the right \
                shoulder, comes round the whole front at about knee height, and \
                finishes behind the left. Four keys across the live window rather \
                than two, because a circle interpolated from its ends is a \
                straight line through the middle and the whole point of this \
                volume is that it is a circle. The knees carry the read -- a spin \
                thrown from standing height misses everybody, and the drop is \
                what a player sees from across the arena. Both heels come up and \
                the turn happens on the toes, which is the only way a body turns \
                this far without a foot leaving the spot it is standing on."
            .into(),
        keys: score.0,
    }
}

// ---------------------------------------------------------------------------
// Rush sweep -- the hammer dragged along the floor
// ---------------------------------------------------------------------------

/// The hammer taken down through the ground line as you run past, low enough to
/// take the legs.
///
/// The Rush row is three answers to the same question -- what does this weapon
/// do to somebody you are running *at* -- and the hammer's answer is the one
/// that goes underneath. The sword cuts across them at chest height, the spear
/// stops and puts everything into the point, and this one scrapes the floor in
/// front of your own feet and keeps going.
///
/// Nothing is planted, exactly as in the Rush slash and for the same reason:
/// the dash drives the body at its own speed whatever the legs do. So the legs
/// run and the arms carry the whole move. The difference is that the arms here
/// are carrying a weight *down*, which means the body goes down with it -- and
/// a runner who drops that far is the shape a player has to be able to read
/// from behind.
fn rush_sweep() -> Recipe {
    let clip = Clip::ChampionRushSweep;
    let (windup, contact, through) = clip.phases().expect("rush sweep animates a move");
    let last = clip.length().saturating_sub(1);

    let running = |body: Pose, phase: f32| {
        body.plant_l([-0.15, 0.08 + 0.18 * phase.max(0.0), 0.32 * phase])
            .toe_l(16.0 * phase)
            .plant_r([0.15, 0.08 + 0.18 * (-phase).max(0.0), -0.32 * phase])
            .toe_r(-16.0 * phase)
    };

    // Running, and the weight has come up onto the outside shoulder. One key of
    // wind-up, which is all six frames allows.
    let shoulder = {
        let body = Pose::rest()
            .hips(0.04, -0.08, 0.06)
            .root(12.0, -4.0, 20.0)
            .spine(14.0, -6.0, 16.0)
            .chest(7.0, -4.0, 18.0)
            .head(-4.0, 0.0, -30.0)
            .wrists(-2.0, 0.0, 0.0);
        running(weapon(body, [0.20, 1.34, 0.06], [0.44, 0.76, -0.48]), 0.6)
    };

    // The head arrives at the floor outside the lead foot. The body has folded
    // over it and the run has not stopped -- this is a sprinter reaching down,
    // not a fighter crouching.
    let scrape = {
        let body = Pose::rest()
            .hips(0.02, -0.26, 0.16)
            .root(36.0, -3.0, 6.0)
            .spine(34.0, -5.0, 4.0)
            .chest(15.0, -3.0, 6.0)
            .head(2.0, 0.0, -10.0)
            .wrists(-12.0, 0.0, 0.0);
        running(weapon(body, [0.16, 0.54, 0.50], [0.30, -0.66, 0.69]), 0.2)
    };

    // Dragged across the front at floor level, hips turning through it. The
    // head of the weapon is the lowest thing in the class.
    let drag = {
        let body = Pose::rest()
            .hips(-0.04, -0.30, 0.14)
            .root(38.0, 4.0, -14.0)
            .spine(35.0, 6.0, -12.0)
            .chest(15.0, 4.0, -14.0)
            .head(4.0, 0.0, 18.0)
            .wrists(-12.0, 0.0, 0.0);
        running(
            weapon(body, [-0.12, 0.44, 0.56], [-0.44, -0.44, 0.78]),
            -0.4,
        )
    };

    // Coming back up out of it while still travelling: the head swings off the
    // floor behind the lead hip and the body unfolds around it.
    let lift = {
        let body = Pose::rest()
            .hips(-0.04, -0.16, 0.10)
            .root(22.0, 4.0, -16.0)
            .spine(22.0, 6.0, -13.0)
            .chest(10.0, 4.0, -14.0)
            .head(0.0, 0.0, 22.0)
            .wrists(-8.0, 0.0, 0.0);
        running(weapon(body, [-0.18, 0.86, 0.38], [-0.46, 0.30, 0.84]), -0.6)
    };

    // Tall again, weapon back across the chest, still running. The Rush moves
    // hand back to a body in motion rather than to a guard.
    let carry = {
        let body = Pose::rest()
            .hips(0.0, -0.08, 0.06)
            .root(10.0, 0.0, 12.0)
            .spine(12.0, 0.0, 10.0)
            .chest(6.0, 0.0, 12.0)
            .head(-4.0, 0.0, -20.0)
            .wrists(-6.0, 0.0, 0.0);
        running(weapon(body, [0.06, 1.18, 0.26], [0.16, 0.46, 0.87]), 0.4)
    };

    let mut score = Score::new();
    score.key(0, ready(), Ease::OUT);
    score.key(tell(windup), shoulder, Ease::LINEAR);
    score.key(contact, scrape, Ease::LINEAR);
    score.key(part(contact, through, 0.6), drag, Ease::LINEAR);
    score.key(through, lift, Ease::LINEAR);
    score.key(part(through, last, 0.55), carry, CARRY);
    score.key(last, ready(), Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::HEAVY,
        notes: "The hammer's answer to somebody you are running at, and it is \
                the one that goes underneath: the head comes down outside the \
                lead foot, scrapes the floor across the whole front, and comes \
                back up behind the hip without the run ever stopping. Nothing \
                is planted, for the same reason nothing is planted in the Rush \
                slash -- the dash drives the body at its own speed whatever the \
                legs do -- so the legs run and the arms carry the move. What \
                makes it readable from across the arena is the drop: a runner \
                who folds that far over is a shape nothing else in the kit \
                makes, and it is the warning that this one is coming in under \
                a guard rather than across it."
            .into(),
        keys: score.0,
    }
}

// ---------------------------------------------------------------------------
// Rising cut -- the sword's takeoff
// ---------------------------------------------------------------------------

/// One line from the back toe to the point, going up and forward, and the floor
/// is gone by the time the hitbox appears.
///
/// The three takeoffs share a button with jump and have to be told apart in the
/// air, so each of them owns a different **direction**: this one goes up *and
/// forward* along a diagonal, the uppercut goes straight up and takes somebody
/// with it, and the pole drive goes up off a shaft planted at the fighter's own
/// feet. Read from behind, the three silhouettes are a diagonal, a column and a
/// vault.
///
/// It is the hardest-hitting of the three and has nothing else in it -- no
/// grab, no shove, no extra height. Missing it leaves you falling with a long
/// tail and nothing to show, which is what the damage is paying for.
fn rising_cut() -> Recipe {
    let clip = Clip::ChampionRisingCut;
    let (windup, contact, through) = clip.phases().expect("rising cut animates a move");
    let last = clip.length().saturating_sub(1);

    // A fast dip with the blade dropped behind the right hip, point down and
    // back. Nine frames of startup is two keys and no hold.
    let sink = {
        let body = Pose::rest()
            .hips(0.04, -0.14, -0.02)
            .root(9.0, -4.0, 18.0)
            .spine(13.0, -6.0, 12.0)
            .chest(6.0, -4.0, 13.0)
            .head(-2.0, 0.0, -32.0)
            .wrists(-6.0, 0.0, 0.0);
        stand(
            weapon(body, [0.16, 1.00, -0.04], [0.36, -0.80, -0.48]),
            -4.0,
            0.03,
        )
    };

    // The bottom of it: knees folded, the blade low and behind, eyes already up
    // the line it is going to travel.
    let load = {
        let body = Pose::rest()
            .hips(0.02, -0.22, 0.02)
            .root(12.0, -3.0, 20.0)
            .spine(16.0, -5.0, 13.0)
            .chest(7.0, -3.0, 13.0)
            .head(-10.0, 0.0, -34.0)
            .wrists(-4.0, 0.0, 0.0);
        stand(
            weapon(body, [0.12, 1.00, 0.06], [0.26, -0.88, -0.40]),
            -2.0,
            0.02,
        )
    };

    // Contact, and the frame the feet leave: everything on one diagonal from a
    // trailing toe through the hips to a point out at head height and forward.
    // The blade leads the body up the same line the body is about to take.
    let rise = {
        let body = Pose::rest()
            .hips(0.0, 0.02, 0.12)
            .root(-2.0, 0.0, -4.0)
            .spine(-5.0, 0.0, -4.0)
            .chest(-3.0, 0.0, -6.0)
            .head(-20.0, 0.0, 8.0)
            .wrists(0.0, 0.0, 0.0);
        weapon(body, [0.04, 1.32, 0.36], [0.02, 0.72, 0.69])
            .plant_l([-0.145, GROUND + 0.08, 0.10])
            .toe_l(32.0)
            .plant_r([0.145, GROUND + 0.05, -0.20])
            .toe_r(30.0)
    };

    // Climbing. The blade has gone on past overhead and forward and the legs
    // have trailed out behind, which is the shape that says this went somewhere
    // rather than merely swung.
    let climb = {
        let body = Pose::rest()
            .hips(0.0, 0.0, 0.06)
            .root(-10.0, 0.0, -2.0)
            .spine(-13.0, 0.0, -2.0)
            .chest(-8.0, 0.0, -4.0)
            .head(-24.0, 0.0, 4.0)
            .wrists(4.0, 0.0, 0.0);
        weapon(body, [0.02, 1.62, 0.22], [-0.02, 0.92, 0.39])
            .plant_l([-0.15, 0.40, -0.06])
            .toe_l(26.0)
            .plant_r([0.15, 0.30, -0.26])
            .toe_r(24.0)
    };

    // The top. The blade comes back down across the front and the knees gather
    // -- the beat where the move is over and the fall has not started.
    let apex = {
        let body = Pose::rest()
            .hips(0.0, -0.04, 0.0)
            .root(-2.0, 0.0, 2.0)
            .spine(-1.0, 0.0, 3.0)
            .chest(0.0, 0.0, 3.0)
            .head(-8.0, 0.0, -6.0)
            .wrists(-2.0, 0.0, 0.0);
        weapon(body, [0.06, 1.42, 0.28], [-0.14, 0.56, 0.82])
            .plant_l([-0.15, 0.48, 0.14])
            .toe_l(12.0)
            .plant_r([0.15, 0.40, -0.08])
            .toe_r(14.0)
    };

    // Falling, guard restored, legs reaching for a floor that is not there yet.
    // The landing clips take it from here.
    let ride = {
        let body = Pose::rest()
            .hips(0.0, -0.05, 0.0)
            .root(3.0, 0.0, 9.0)
            .spine(7.0, 0.0, 8.0)
            .chest(4.0, 0.0, 11.0)
            .head(-3.0, 0.0, -22.0)
            .wrists(-8.0, 0.0, 0.0);
        weapon(body, [0.0, 1.17, 0.26], [-0.16, 0.88, 0.44])
            .plant_l([-0.145, GROUND + 0.05, 0.14])
            .toe_l(-8.0)
            .plant_r([0.145, GROUND + 0.03, -0.13])
            .toe_r(-4.0)
    };

    let mut score = Score::new();
    score.key(0, ready(), Ease::OUT);
    score.key(tell(windup), sink, Ease::SMOOTH);
    score.key(part(tell(windup), windup, 0.5), load, Ease::IN);
    score.key(contact, rise, Ease::STRIKE);
    score.key(through, climb, Ease::OUT);
    score.key(part(through, last, 0.45), apex, Ease::SMOOTH);
    score.key(last, ride, Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "The sword's way off the floor, and the one of the three \
                takeoffs that is only an attack. Everything is on a diagonal: \
                the blade drops behind the hip, the knees fold, and then one \
                line runs from a trailing toe through the hips to a point out \
                at head height and forward, with the feet leaving the ground on \
                exactly the frame the hitbox appears. The three takeoffs share \
                a button with jump and have to be told apart in the air, so \
                each owns a direction -- this is the diagonal, the uppercut is \
                the column, the pole drive is the vault. It ends falling rather \
                than landing, because at this lift the fighter is still in the \
                air when the recovery runs out, and there is nothing else in \
                the move to soften that: missing it is a long way down."
            .into(),
        keys: score.0,
    }
}

// ---------------------------------------------------------------------------
// Pole drive -- the spear's takeoff
// ---------------------------------------------------------------------------

/// The butt of the spear cracked into the floor at your own feet, and the
/// ground throws you off it.
///
/// **A launch, not a swing.** It does the least damage of anything the class
/// throws and it goes the highest, and what it is actually for is the shove:
/// the move table lifts the fighter further than a jump reaches and
/// `tuning::pole_drive_boost` sends them wherever they are holding, so this is
/// how a Champion crosses ground and gains height on one press.
///
/// The cousin it must not be confused with is the vault, which plants *ahead*
/// and swings the body over a fixed point. This one plants **beside the lead
/// foot** and pushes straight off it, which is the difference between pole
/// vaulting and pogoing -- and the difference the player has to see, because
/// one costs a Rush charge and the other costs a jump.
fn pole_drive() -> Recipe {
    let clip = Clip::ChampionPoleDrive;
    let (windup, contact, through) = clip.phases().expect("pole drive animates a move");
    let last = clip.length().saturating_sub(1);

    // The weapon turned over: hands climbing the haft as the butt swings down
    // and the point comes up behind the shoulder. Seven frames, so the flip is
    // the whole of the startup.
    let flip = {
        let body = Pose::rest()
            .hips(0.0, -0.10, 0.0)
            .root(2.0, -3.0, 12.0)
            .spine(4.0, -4.0, 9.0)
            .chest(1.0, -3.0, 10.0)
            .head(-10.0, 0.0, -22.0)
            .wrists(-2.0, 0.0, 0.0);
        stand(
            weapon(body, [0.10, 1.30, 0.06], [-0.18, 0.92, -0.35]),
            -4.0,
            0.03,
        )
    };

    // Set: the butt beside the lead foot, both hands high on the shaft, knees
    // loaded hard. A pogo rather than a vault, and the shaft is vertical to
    // make the difference impossible to miss.
    let set = {
        let body = Pose::rest()
            .hips(0.0, -0.26, 0.06)
            .root(16.0, 0.0, 6.0)
            .spine(20.0, 0.0, 4.0)
            .chest(9.0, 0.0, 4.0)
            .head(-6.0, 0.0, -10.0)
            .wrists(-6.0, 0.0, 0.0);
        stand(
            weapon(body, [0.02, 1.28, 0.20], [-0.04, 0.99, -0.08]),
            -6.0,
            0.02,
        )
    };

    // The crack. The shaft is driven into the floor, the body extends off it,
    // and the feet leave -- everything straight, which is where the height
    // comes from.
    let crack = {
        let body = Pose::rest()
            .hips(0.0, 0.04, 0.02)
            .root(-6.0, 0.0, 2.0)
            .spine(-10.0, 0.0, 0.0)
            .chest(-6.0, 0.0, 0.0)
            .head(-18.0, 0.0, -2.0)
            .wrists(2.0, 0.0, 0.0);
        weapon(body, [0.02, 1.46, 0.18], [-0.02, 0.99, -0.06])
            .plant_l([-0.145, GROUND + 0.09, 0.08])
            .toe_l(34.0)
            .plant_r([0.145, GROUND + 0.07, -0.16])
            .toe_r(32.0)
    };

    // Off it: knees up and forward, the spear coming back down across the body
    // as the hands slide back to the middle of the haft.
    let off = {
        let body = Pose::rest()
            .hips(0.0, -0.02, 0.04)
            .root(-4.0, 0.0, 6.0)
            .spine(-4.0, 0.0, 6.0)
            .chest(-2.0, 0.0, 8.0)
            .head(-16.0, 0.0, -10.0)
            .wrists(0.0, 0.0, 0.0);
        weapon(body, [0.04, 1.36, 0.26], [-0.10, 0.84, 0.53])
            .plant_l([-0.15, 0.44, 0.24])
            .toe_l(22.0)
            .plant_r([0.15, 0.36, 0.04])
            .toe_r(20.0)
    };

    // Sailing, spear levelled, legs gathering. The falling clips take over.
    let sail = {
        let body = Pose::rest()
            .hips(0.0, -0.04, 0.0)
            .root(2.0, 0.0, 8.0)
            .spine(5.0, 0.0, 8.0)
            .chest(3.0, 0.0, 10.0)
            .head(-6.0, 0.0, -18.0)
            .wrists(-6.0, 0.0, 0.0);
        weapon(body, [0.02, 1.22, 0.30], [-0.12, 0.62, 0.78])
            .plant_l([-0.15, 0.30, 0.14])
            .toe_l(12.0)
            .plant_r([0.15, 0.22, -0.06])
            .toe_r(14.0)
    };

    let mut score = Score::new();
    score.key(0, ready(), Ease::OUT);
    score.key(tell(windup), flip, Ease::LINEAR);
    score.key(part(tell(windup), windup, 0.75), set, Ease::IN);
    score.key(contact, crack, Ease::STRIKE);
    score.key(through, off, Ease::OUT);
    score.key(last, sail, Ease::SMOOTH);

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "A launch, not a swing: the least damage in the class and the \
                most height. The spear is turned over in the startup -- hands \
                climbing the haft while the butt swings down -- and then driven \
                into the floor beside the lead foot, and the body extends off \
                it straight. That is the whole silhouette, and it is chosen to \
                be impossible to confuse with the vault: the vault plants \
                *ahead* and swings the body over a fixed point, this plants \
                *beside* and pushes off, which is pole vaulting against \
                pogoing. The player has to see which one they got, because one \
                costs a Rush charge and the other costs a jump. It ends \
                sailing, with the boost carrying whatever direction was held \
                when the shaft hit the ground."
            .into(),
        keys: score.0,
    }
}
