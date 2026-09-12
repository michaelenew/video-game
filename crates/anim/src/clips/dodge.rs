//! Dodging: the universal defensive verb, on the ground and in the air.
//!
//! A dodge beats a read and loses to a delayed attack. Both halves of that
//! sentence have to be on screen, because a player who cannot see where the
//! invulnerability ended cannot learn when to hold an attack back.
//!
//! ## The one thing these clips exist to communicate
//!
//! `Action::invulnerable` is true for the first `dodge_iframes()` frames and
//! false afterwards, and **the silhouette has to change on that frame**. So
//! every recipe here is built around one boundary: low and tucked and
//! unchanging while the frames are free, then rising, open and armless the
//! instant they are not. The frame it happens on is derived from the Oven
//! rather than typed, so retuning the window moves the break with it.
//!
//! ## Which clip plays is chosen from the velocity
//!
//! `view::play::dodge` picks the direction from the body's actual travel, not
//! from the key that was pressed -- so a dodge input against a wall, or during
//! a turn, plays the clip matching where the character really went. That is
//! also why the four grounded clips have to be four different *movements*
//! rather than one movement rotated: they are seen from the same camera angle.
//!
//! ## Nothing is planted during the burst
//!
//! A dodge covers about three metres in twenty-two frames, two thirds of it
//! while invulnerable. That is a quarter of a metre per frame at the start --
//! further than a walking stride -- so there is no such thing as a planted foot
//! through the middle of one of these. The feet leave the ground, and the only
//! contacts worth arithmetic are the push-off and the catch at the end, which
//! `planted` below places against the ground the body actually covers.
//!
//! ## The rig cannot roll over
//!
//! The root joint has fifty degrees of forward swing and thirty-five of tilt,
//! which is a body that folds, not one that inverts. So the sideways clips are
//! not literal three-hundred-and-sixty-degree rolls; they sell a roll by
//! getting very low, curling hard toward the direction of travel and hiding the
//! head behind the lead shoulder. Trying to fake the rotation with the hips
//! offset instead reads as a character being dragged sideways through the floor.

use crate::bake::{Key, Looseness, Recipe};
use crate::clips::locomotion::stance;
use crate::ease::Ease;
use view::clips::Clip;
use view::pose::{ANKLE_ON_GROUND as GROUND, Pose};

/// Where the feet sit either side of the centre line, as in `locomotion`.
const L: f32 = -0.115;
const R: f32 = 0.115;

pub fn clips() -> Vec<Recipe> {
    let right = roll(Clip::DodgeRight);
    // `forward()` is written and is *not* in this list. The dive it produces
    // whips the shoulders and the trailing leg faster, relative to the hips,
    // than a body can move them -- it fails `baked_motion_is_continuous` at
    // every threshold that the other four clear comfortably, which is the
    // signature of a clip that needs its middle re-timed rather than a test
    // that needs relaxing. It bakes as a held rest pose until somebody does
    // that, and the bake says so by name.
    vec![back(), mirrored(&right, Clip::DodgeLeft), right, air()]
}

// ---------------------------------------------------------------------------
// What the simulation says
// ---------------------------------------------------------------------------

/// The first frame of a dodge clip on which the character can be hit.
///
/// `Action::invulnerable` compares the countdown against the **grounded**
/// dodge's numbers whichever kind of dodge is running, and an airdodge starts
/// its countdown lower. So the airdodge loses its cover four frames into
/// sixteen where a grounded dodge keeps it for ten out of twenty-two, and the
/// two clips have to break in completely different places. Deriving that here
/// is the only way the airdodge's break lands anywhere near honest.
fn first_vulnerable(length: u16) -> u16 {
    let spent = sim::tuning::dodge_frames().saturating_sub(sim::tuning::dodge_iframes());
    length.saturating_sub(spent)
}

/// Metres of ground the body has covered by the start of frame `f`.
///
/// The burst is one lump of velocity that decays every tick, so this is a
/// geometric sum and not speed times time. The difference is the whole shape of
/// the clip: the first frame covers a quarter of a metre and the last covers
/// six centimetres, which is why the fast half has nothing on the floor and the
/// slow half can afford a step.
fn travelled(f: f32, speed: f32) -> f32 {
    let decay = sim::tuning::dodge_decay().to_f32_for_render();
    (speed / crate::HZ) * decay * (1.0 - decay.powf(f)) / (1.0 - decay)
}

/// Where a foot that took the ground at frame `landed`, at `contact` in
/// character space, has slid to by frame `f`.
///
/// A foot on the floor does not move in the world; the body moves past it. At
/// the end of a dodge that is still eight centimetres a frame, and a recovery
/// step keyed without it is a moonwalk -- the one failure in this family that
/// is obvious from the back row.
fn planted(contact: [f32; 3], landed: f32, f: f32, away: [f32; 2], speed: f32) -> [f32; 3] {
    let moved = travelled(f, speed) - travelled(landed, speed);
    [
        contact[0] + away[0] * moved,
        contact[1],
        contact[2] + away[1] * moved,
    ]
}

fn ground_speed() -> f32 {
    sim::tuning::dodge_speed().to_f32_for_render()
}

/// The same recipe on the other side. The two rolls are one movement, and
/// `mirrored` makes the second one exactly rather than approximately -- which
/// is the entire reason poses store named channels instead of Euler triples.
fn mirrored(from: &Recipe, clip: Clip) -> Recipe {
    Recipe {
        clip,
        looseness: from.looseness,
        notes: from.notes.clone(),
        keys: from
            .keys
            .iter()
            .map(|k| Key::eased(k.frame, k.pose.mirrored(), k.ease))
            .collect(),
    }
}

// ---------------------------------------------------------------------------
// Forward: a dive
// ---------------------------------------------------------------------------

/// Held back: see the note in `clips()`. Kept rather than deleted, because
/// the shapes are right and it is the timing between two of them that is not.
#[allow(dead_code)]
fn forward() -> Recipe {
    let clip = Clip::DodgeForward;
    let len = clip.length();
    let brk = first_vulnerable(len);
    let speed = ground_speed();
    // A planted foot slides the opposite way to the body.
    let away = [0.0, -1.0];

    let (f_off, f_out) = (1, 4);
    let f_ball = brk - 3;
    let f_catch = brk + (len - brk) / 3;
    let f_step = brk + (len - brk) * 2 / 3;
    let f_last = len - 1;

    // Frame zero is already going -- the simulation hands out the whole
    // velocity on the first frame of the action, so an anticipation pose here
    // would be the body winding up for something it has already done. It is the
    // only grounded frame at this end of the clip: by frame one the body has
    // covered a quarter of a metre, which is most of what a leg has behind it.
    //
    // The next key is one frame away, and gently eased, because the solver
    // records frame zero *after* a frame's worth of stepping. A snap out of the
    // first key throws that key away -- the clip opens most of the way to the
    // second one, and whatever was standing on the floor is not any more.
    let set = stance()
        .hips(0.0, -0.12, 0.03)
        .root(16.0, 0.0, -8.0)
        .spine(14.0, 0.0, 5.0)
        .chest(8.0, 0.0, 6.0)
        .head(-22.0, 0.0, 2.0)
        .shoulder_l(10.0, 22.0, 0.0)
        .elbow_l(64.0)
        .shoulder_r(4.0, 20.0, 0.0)
        .elbow_r(60.0)
        .plant_l([L - 0.02, GROUND, 0.12])
        .plant_r([R + 0.02, GROUND, -0.24])
        .toe_l(-2.0)
        .toe_r(10.0);

    // One frame later, and both feet have already left. The back leg has run
    // out of extension and its toe is the last thing to go; the front foot is
    // barely clear. Both are placed with `plant` at a raised ankle rather than
    // by hand, because a foot keyed as angles this close to the floor is a foot
    // that ends up through it two frames later when the springs undershoot.
    let off = stance()
        .hips(0.0, -0.17, 0.07)
        .root(32.0, 0.0, -5.0)
        .spine(24.0, 0.0, 4.0)
        .chest(14.0, 0.0, 5.0)
        .head(-34.0, 0.0, 1.0)
        .shoulder_l(48.0, 20.0, 0.0)
        .elbow_l(48.0)
        .shoulder_r(40.0, 18.0, 0.0)
        .elbow_r(44.0)
        .plant_l([L - 0.02, GROUND + 0.12, 0.04])
        .plant_r([R + 0.02, GROUND + 0.06, -0.32])
        .toe_l(-10.0)
        .toe_r(16.0);

    // Flat and long, arms thrown out in front, heels up behind. This is the
    // shape that makes it a dive rather than a running step.
    //
    // A hip's swing is measured against the root, and the root is leaning
    // forty-four degrees -- so a trailing leg is a *small positive* number
    // here, not a negative one. Written as if the angles were against the
    // world, the legs come out pointing thirty degrees into the floor.
    let dive = Pose::rest()
        .hips(0.0, -0.22, 0.14)
        .root(44.0, 0.0, -4.0)
        .spine(30.0, 0.0, 3.0)
        .chest(18.0, 0.0, 4.0)
        .head(-40.0, 0.0, 0.0)
        .shoulder_l(96.0, 20.0, 0.0)
        .elbow_l(28.0)
        .shoulder_r(104.0, 18.0, 0.0)
        .elbow_r(22.0)
        .hip_l(8.0, 8.0, 0.0)
        .knee_l(80.0)
        .hip_r(2.0, 6.0, 0.0)
        .knee_r(92.0)
        .toe_l(34.0)
        .toe_r(38.0);

    // Balled up: knees under the chest, elbows in, chin down. The smallest the
    // silhouette gets, and it holds from here to the last invulnerable frame,
    // because a shape that stops changing is the clearest way to say the frames
    // are free.
    //
    // The fold stops at about a hundred degrees on purpose. Past that the head
    // is lower than the hips, and a body whose head is under its own hips does
    // not read as tucked -- it reads as having tripped.
    let ball = Pose::rest()
        .hips(0.0, -0.30, 0.18)
        .root(42.0, 0.0, 0.0)
        .spine(34.0, 0.0, 0.0)
        .chest(24.0, 0.0, 0.0)
        .head(22.0, 0.0, 0.0)
        .shoulder_l(72.0, 34.0, 0.0)
        .elbow_l(124.0)
        .shoulder_r(76.0, 32.0, 0.0)
        .elbow_r(128.0)
        .hip_l(52.0, 14.0, 0.0)
        .knee_l(104.0)
        .hip_r(58.0, 12.0, 0.0)
        .knee_r(110.0)
        .toe_l(20.0)
        .toe_r(22.0);

    // The last frame that cannot be hit. Still balled and still at the same
    // height, but the lead leg has swung out under the body, so the break has
    // somewhere to go and does not have to invent it.
    let gather = Pose::rest()
        .hips(0.0, -0.30, 0.12)
        .root(42.0, 0.0, -6.0)
        .spine(34.0, 0.0, 4.0)
        .chest(24.0, 0.0, 6.0)
        .head(12.0, 0.0, 0.0)
        .shoulder_l(50.0, 30.0, 0.0)
        .elbow_l(108.0)
        .shoulder_r(62.0, 28.0, 0.0)
        .elbow_r(100.0)
        .hip_l(34.0, 12.0, 0.0)
        .knee_l(112.0)
        .hip_r(68.0, 14.0, 0.0)
        .knee_r(100.0)
        .toe_l(16.0)
        .toe_r(-12.0);

    // Vulnerable. The foot arrives, the body comes up off it, and the arms go
    // out and back -- nowhere near a guard, which is the point.
    //
    // The lead foot is placed under the body rather than out in front of it.
    // Reaching further forward from hips this low asks the solver for a thigh
    // that is horizontal, which is exactly where the swing-and-spread
    // decomposition has its pole: the answer comes back as a hundred and
    // fifteen degrees of abduction and a leg on backwards.
    let catch_at = [R + 0.05, GROUND, 0.23];
    let catch = Pose::rest()
        .hips(0.0, -0.21, 0.0)
        .root(21.0, 0.0, -5.0)
        .spine(16.0, 0.0, 7.0)
        .chest(8.0, 0.0, 9.0)
        .head(-26.0, 0.0, -5.0)
        .shoulder_l(-38.0, 48.0, 0.0)
        .elbow_l(54.0)
        .shoulder_r(-22.0, 56.0, 0.0)
        .elbow_r(46.0)
        .hip_l(-20.0, 12.0, 0.0)
        .knee_l(98.0)
        .plant_r(catch_at)
        .toe_l(-14.0)
        .toe_r(-8.0);

    // The trailing foot passes and takes over while the first one slides back
    // under the body at the speed the body is still carrying.
    let step_at = [L - 0.05, GROUND, 0.24];
    let step = Pose::rest()
        .hips(0.0, -0.15, 0.0)
        .root(16.0, 0.0, 5.0)
        .spine(8.0, 0.0, -5.0)
        .chest(4.0, 0.0, -7.0)
        .head(-15.0, 0.0, 3.0)
        .shoulder_l(-16.0, 42.0, 0.0)
        .elbow_l(48.0)
        .shoulder_r(8.0, 46.0, 0.0)
        .elbow_r(40.0)
        .plant_r(planted(
            catch_at,
            f_catch as f32,
            f_step as f32,
            away,
            speed,
        ))
        .plant_l(step_at)
        .toe_floor_r()
        .toe_l(-10.0);

    // Back on the feet, not back on guard. The hips and legs are close to the
    // fighting stance so the hand-off to idle does not pop; the arms are not,
    // because the recovery frames are the ones you get punished in.
    let settle = stance()
        .hips(0.0, -0.09, 0.0)
        .root(7.0, 0.0, -10.0)
        .spine(7.0, 0.0, 5.0)
        .chest(3.0, 0.0, 6.0)
        .head(-5.0, 0.0, 3.0)
        .shoulder_l(0.0, 30.0, 0.0)
        .elbow_l(42.0)
        .shoulder_r(-6.0, 32.0, 0.0)
        .elbow_r(36.0)
        .plant_l(planted(step_at, f_step as f32, f_last as f32, away, speed))
        .plant_r([R + 0.05, GROUND, -0.20])
        .toe_l(0.0)
        .toe_r(10.0);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "A dive, not a running step: the body goes flat and long, the \
                arms are thrown out in front and the legs trail. It balls up \
                for the invulnerable frames and stops changing, then breaks on \
                the exact frame the cover runs out -- the lead foot takes the \
                ground, the chest comes up and the arms end wide and low, \
                nowhere near a guard. The catching feet are placed from the \
                ground the body actually covers, which by then is still eight \
                centimetres a frame."
            .into(),
        keys: vec![
            Key::eased(0, set, Ease::LINEAR),
            Key::eased(f_off, off, Ease::OUT),
            Key::eased(f_out, dive, Ease::OUT),
            Key::eased(f_ball, ball, Ease::IN),
            Key::eased(brk, gather, Ease::STRIKE),
            Key::eased(f_catch, catch, Ease::OUT),
            Key::eased(f_step, step, Ease::OUT),
            Key::eased(f_last, settle, Ease::SMOOTH),
        ],
    }
}

// ---------------------------------------------------------------------------
// Back: a hop out of range
// ---------------------------------------------------------------------------

fn back() -> Recipe {
    let clip = Clip::DodgeBack;
    let len = clip.length();
    let brk = first_vulnerable(len);

    let (f_out, f_air) = (2, (brk * 3) / 5);
    let f_land = brk + (len - brk) / 3;
    let f_low = brk + (len - brk) * 2 / 3;
    let f_last = len - 1;

    // Both feet drive at once -- a hop, not a step. The dip is tiny because the
    // velocity is already applied; a real compression here would be the
    // character crouching after it had left.
    let launch = stance()
        .hips(0.0, -0.22, -0.04)
        .root(14.0, 0.0, -10.0)
        .spine(16.0, 0.0, 6.0)
        .chest(10.0, 0.0, 7.0)
        .head(-20.0, 0.0, 4.0)
        .shoulder_l(-26.0, 20.0, 0.0)
        .elbow_l(58.0)
        .shoulder_r(-32.0, 18.0, 0.0)
        .elbow_r(52.0)
        .plant_l([L - 0.03, GROUND, 0.10])
        .plant_r([R + 0.03, GROUND, -0.04])
        .toe_l(6.0)
        .toe_r(8.0);

    // Off the floor, but going backwards rather than upwards: the knees come up
    // in front and the chest folds down over them, so the whole body is under
    // where a swing at chest height would be.
    let leave = Pose::rest()
        .hips(0.0, -0.16, -0.06)
        .root(26.0, 0.0, -8.0)
        .spine(24.0, 0.0, 5.0)
        .chest(16.0, 0.0, 6.0)
        .head(-24.0, 0.0, 3.0)
        .shoulder_l(-16.0, 26.0, 0.0)
        .elbow_l(78.0)
        .shoulder_r(-20.0, 24.0, 0.0)
        .elbow_r(72.0)
        .hip_l(58.0, 10.0, 0.0)
        .knee_l(78.0)
        .hip_r(48.0, 8.0, 0.0)
        .knee_r(88.0)
        .toe_l(28.0)
        .toe_r(30.0);

    // The tuck. Knees to the chest, elbows in, head down between them -- a
    // fighting-game hop, and the smallest a body gets without lying down.
    let tuck = Pose::rest()
        .hips(0.0, -0.12, -0.02)
        .root(38.0, 0.0, -4.0)
        .spine(38.0, 0.0, 2.0)
        .chest(28.0, 0.0, 3.0)
        .head(18.0, 0.0, 0.0)
        .shoulder_l(28.0, 36.0, 0.0)
        .elbow_l(132.0)
        .shoulder_r(24.0, 34.0, 0.0)
        .elbow_r(136.0)
        .hip_l(96.0, 14.0, 0.0)
        .knee_l(122.0)
        .hip_r(92.0, 12.0, 0.0)
        .knee_r(126.0)
        .toe_l(16.0)
        .toe_r(18.0);

    // Last invulnerable frame: the legs start reaching down for the floor while
    // everything else is still tucked, so the break has a direction.
    let reach = Pose::rest()
        .hips(0.0, -0.18, 0.02)
        .root(34.0, 0.0, -3.0)
        .spine(32.0, 0.0, 2.0)
        .chest(22.0, 0.0, 3.0)
        .head(6.0, 0.0, 0.0)
        .shoulder_l(14.0, 40.0, 0.0)
        .elbow_l(118.0)
        .shoulder_r(10.0, 38.0, 0.0)
        .elbow_r(122.0)
        .hip_l(70.0, 14.0, 0.0)
        .knee_l(104.0)
        .hip_r(66.0, 12.0, 0.0)
        .knee_r(108.0)
        .toe_l(-4.0)
        .toe_r(-2.0);

    // Vulnerable, and it should look it: the feet arrive out in front of a body
    // still travelling backwards, so the landing is a sprawl rather than a
    // stance. Arms out for balance, guard gone.
    //
    // These feet **skid**, and are placed by hand rather than by `planted`.
    // From here to the end the body covers half a metre, and a foot nailed to
    // the ground for that would have to end up further in front of the hips
    // than the leg is long -- the solver answers with a straight leg pointing
    // at nothing. What a real landing off four metres a second does is put the
    // heels down and slide, so that is what these do: a few centimetres of
    // creep, and then the trailing foot picks up and steps back underneath.
    let land = Pose::rest()
        .hips(0.0, -0.30, -0.06)
        .root(-6.0, 0.0, 0.0)
        .spine(18.0, 0.0, 0.0)
        .chest(10.0, 0.0, 0.0)
        .head(-16.0, 0.0, 0.0)
        .shoulder_l(-34.0, 62.0, 0.0)
        .elbow_l(38.0)
        .shoulder_r(-36.0, 64.0, 0.0)
        .elbow_r(34.0)
        .plant_l([L - 0.07, GROUND, 0.20])
        .plant_r([R + 0.07, GROUND, 0.14])
        .toe_l(-14.0)
        .toe_r(-12.0);

    // The absorption. Deepest here, and both heels are still creeping forward
    // under a body that has not stopped.
    let absorb = Pose::rest()
        .hips(0.0, -0.36, -0.02)
        .root(6.0, 0.0, -2.0)
        .spine(14.0, 0.0, 2.0)
        .chest(6.0, 0.0, 3.0)
        .head(-20.0, 0.0, 2.0)
        .shoulder_l(-20.0, 54.0, 0.0)
        .elbow_l(50.0)
        .shoulder_r(-24.0, 56.0, 0.0)
        .elbow_r(46.0)
        .plant_l([L - 0.06, GROUND, 0.26])
        .plant_r([R + 0.06, GROUND, 0.20])
        .toe_l(-6.0)
        .toe_r(-4.0);

    // The rear foot picks up and steps back under the hips, which is how a
    // skid actually ends and how the clip hands a plausible stance to whatever
    // plays next.
    let settle = stance()
        .hips(0.0, -0.11, 0.0)
        .root(4.0, 0.0, -11.0)
        .spine(8.0, 0.0, 5.0)
        .chest(4.0, 0.0, 6.0)
        .head(-6.0, 0.0, 4.0)
        .shoulder_l(-4.0, 32.0, 0.0)
        .elbow_l(44.0)
        .shoulder_r(-10.0, 34.0, 0.0)
        .elbow_r(38.0)
        .plant_l([L - 0.05, GROUND, 0.28])
        .plant_r([R + 0.05, GROUND, -0.10])
        .toe_l(-2.0)
        .toe_r(6.0);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "Both feet drive at once, which is what separates a hop from a \
                step. It goes backwards rather than upwards on purpose: the \
                knees come up in front and the chest folds over them, so the \
                invulnerable frames are spent underneath where a swing at chest \
                height would be. The break is the landing -- feet out in front \
                of a body still travelling, arms wide for balance, and a deep \
                absorption that reads as sprawled rather than set."
            .into(),
        keys: vec![
            Key::eased(0, launch, Ease::STRIKE),
            Key::eased(f_out, leave, Ease::OUT),
            Key::eased(f_air, tuck, Ease::IN),
            Key::eased(brk, reach, Ease::STRIKE),
            Key::eased(f_land, land, Ease::OUT),
            Key::eased(f_low, absorb, Ease::OUT),
            Key::eased(f_last, settle, Ease::SMOOTH),
        ],
    }
}

// ---------------------------------------------------------------------------
// Sideways: a roll
// ---------------------------------------------------------------------------

/// The roll to the character's right. `dodge_left` is this mirrored.
fn roll(clip: Clip) -> Recipe {
    let len = clip.length();
    let brk = first_vulnerable(len);
    let speed = ground_speed();
    let away = [-1.0, 0.0];

    let (f_off, f_out) = (1, 4);
    let f_curl = brk - 3;
    let f_catch = brk + (len - brk) / 3;
    let f_step = brk + (len - brk) * 2 / 3;
    let f_last = len - 1;

    // You roll right by driving off the left foot, so the left leg is the one
    // that extends and the right is already folding out of the way. The only
    // grounded frame at this end: half a metre of ground goes past in the next
    // two, and the next key is a single frame away because the solver records
    // frame zero a frame's worth of stepping past the first key.
    let set = stance()
        .hips(0.03, -0.12, 0.0)
        .root(8.0, 12.0, -8.0)
        .spine(8.0, 10.0, 6.0)
        .chest(4.0, 8.0, 8.0)
        .head(-8.0, -6.0, 12.0)
        .shoulder_l(-14.0, 24.0, 0.0)
        .elbow_l(60.0)
        .shoulder_r(18.0, 38.0, 0.0)
        .elbow_r(52.0)
        .plant_l([L - 0.07, GROUND, 0.06])
        .plant_r([R + 0.07, GROUND, -0.06])
        .toe_l(8.0)
        .toe_r(2.0);

    // Driving off, one frame later. The left leg has finished extending and is
    // just clear, the right is already up, and the lead arm is on its way down
    // and out toward the ground it is about to pass over -- the gesture that
    // says roll on a body that cannot actually turn over.
    let off = stance()
        .hips(0.06, -0.16, 0.0)
        .root(12.0, 20.0, -7.0)
        .spine(12.0, 16.0, 5.0)
        .chest(6.0, 12.0, 7.0)
        .head(0.0, -10.0, 16.0)
        .shoulder_l(-18.0, 28.0, 0.0)
        .elbow_l(62.0)
        .shoulder_r(32.0, 72.0, 0.0)
        .elbow_r(40.0)
        .plant_l([L - 0.11, GROUND + 0.16, 0.04])
        .plant_r([R + 0.13, GROUND + 0.18, -0.02])
        .toe_l(2.0)
        .toe_r(0.0);

    // Off the floor, curled toward the travel, the lead arm reaching for the
    // ground under it.
    let throw = Pose::rest()
        .hips(0.13, -0.26, 0.02)
        .root(18.0, 32.0, -4.0)
        .spine(18.0, 22.0, 4.0)
        .chest(12.0, 18.0, 6.0)
        .head(14.0, -18.0, 24.0)
        .shoulder_l(-14.0, 40.0, 0.0)
        .elbow_l(78.0)
        .shoulder_r(46.0, 104.0, 0.0)
        .elbow_r(26.0)
        .hip_l(54.0, 30.0, 0.0)
        .knee_l(122.0)
        .hip_r(62.0, 20.0, 0.0)
        .knee_r(116.0)
        .toe_l(-16.0)
        .toe_r(10.0);

    // Curled hard, head hidden behind the lead shoulder, both knees in. As
    // small and as sideways as the rig goes, and it stays here until the frames
    // run out. Thirty-four degrees of root tilt on top of twenty-two of spine
    // and eighteen of chest is seventy-odd degrees of lateral fold, which is
    // where the sideways read has to come from: the hips can only be thrown
    // fifteen centimetres before the legs stop reaching the floor at all.
    let curl = Pose::rest()
        .hips(0.15, -0.32, 0.03)
        .root(20.0, 34.0, 0.0)
        .spine(22.0, 24.0, 0.0)
        .chest(14.0, 18.0, 0.0)
        .head(22.0, -22.0, 30.0)
        .shoulder_l(44.0, 48.0, 0.0)
        .elbow_l(122.0)
        .shoulder_r(34.0, 116.0, 0.0)
        .elbow_r(94.0)
        .hip_l(68.0, 26.0, 0.0)
        .knee_l(132.0)
        .hip_r(66.0, 18.0, 0.0)
        .knee_r(122.0)
        .toe_l(-16.0)
        .toe_r(14.0);

    // Last invulnerable frame: the lead leg reaches out sideways for the floor
    // while the curl is still on, at the same height it has held all along.
    let gather = Pose::rest()
        .hips(0.13, -0.32, 0.02)
        .root(18.0, 30.0, -4.0)
        .spine(20.0, 22.0, 4.0)
        .chest(14.0, 18.0, 6.0)
        .head(12.0, -18.0, 26.0)
        .shoulder_l(32.0, 52.0, 0.0)
        .elbow_l(110.0)
        .shoulder_r(18.0, 106.0, 0.0)
        .elbow_r(76.0)
        .hip_l(62.0, 24.0, 0.0)
        .knee_l(128.0)
        .hip_r(40.0, 54.0, 0.0)
        .knee_r(88.0)
        .toe_l(-14.0)
        .toe_r(-6.0);

    // Vulnerable: the lead foot takes the ground out to the side and the body
    // uncurls up off it. The trailing arm swings wide, which is both what
    // balance does and what says the guard is not back yet.
    let catch_at = [R + 0.34, GROUND, 0.0];
    let catch = Pose::rest()
        .hips(0.08, -0.28, 0.0)
        .root(16.0, 16.0, -6.0)
        .spine(14.0, 10.0, 6.0)
        .chest(8.0, 8.0, 8.0)
        .head(-14.0, -6.0, 16.0)
        .shoulder_l(-28.0, 66.0, 0.0)
        .elbow_l(46.0)
        .shoulder_r(-10.0, 52.0, 0.0)
        .elbow_r(40.0)
        .hip_l(34.0, 34.0, 0.0)
        .knee_l(108.0)
        .plant_r(catch_at)
        .toe_l(-14.0)
        .toe_r(-6.0);

    // The trailing foot comes under and the lead one keeps sliding away, which
    // is what a body that has not finished moving does to the foot holding it.
    let step_at = [L + 0.16, GROUND, 0.02];
    let step = Pose::rest()
        .hips(0.03, -0.16, 0.0)
        .root(8.0, 6.0, -8.0)
        .spine(8.0, 4.0, 6.0)
        .chest(4.0, 3.0, 7.0)
        .head(-10.0, -2.0, 8.0)
        .shoulder_l(-12.0, 50.0, 0.0)
        .elbow_l(44.0)
        .shoulder_r(-2.0, 44.0, 0.0)
        .elbow_r(38.0)
        .plant_r(planted(
            catch_at,
            f_catch as f32,
            f_step as f32,
            away,
            speed,
        ))
        .plant_l(step_at)
        .toe_floor_r()
        .toe_l(-6.0);

    let settle = stance()
        .hips(0.0, -0.10, 0.0)
        .root(5.0, 0.0, -11.0)
        .spine(7.0, 0.0, 5.0)
        .chest(3.0, 0.0, 6.0)
        .head(-5.0, 0.0, 4.0)
        .shoulder_l(-2.0, 32.0, 0.0)
        .elbow_l(44.0)
        .shoulder_r(-8.0, 34.0, 0.0)
        .elbow_r(38.0)
        .plant_l(planted(step_at, f_step as f32, f_last as f32, away, speed))
        .plant_r([R + 0.10, GROUND, -0.10])
        .toe_l(2.0)
        .toe_r(8.0);

    Recipe {
        clip,
        looseness: Looseness::MARTIAL,
        notes: "A roll sold without rolling. The root has fifty degrees of \
                swing and thirty-five of tilt, so the body cannot go over -- \
                instead it curls hard toward the direction of travel, hides the \
                head behind the lead shoulder and puts that arm down toward the \
                floor it is passing over. The curl holds unchanged while the \
                frames are free, then the lead foot takes the ground out to the \
                side and the body uncurls up off it with both arms wide. \
                `dodge_left` is this mirrored exactly."
            .into(),
        keys: vec![
            Key::eased(0, set, Ease::LINEAR),
            Key::eased(f_off, off, Ease::OUT),
            Key::eased(f_out, throw, Ease::OUT),
            Key::eased(f_curl, curl, Ease::IN),
            Key::eased(brk, gather, Ease::STRIKE),
            Key::eased(f_catch, catch, Ease::OUT),
            Key::eased(f_step, step, Ease::OUT),
            Key::eased(f_last, settle, Ease::SMOOTH),
        ],
    }
}

// ---------------------------------------------------------------------------
// The airdodge
// ---------------------------------------------------------------------------

fn air() -> Recipe {
    let clip = Clip::AirDodge;
    let len = clip.length();
    let brk = first_vulnerable(len);

    let f_open = brk + (len - brk) / 3;
    let f_drop = brk + (len - brk) * 2 / 3;
    let f_last = len - 1;

    // One clip serves every direction an airdodge can go, so it commits in
    // shape rather than in a side: a lean toward the travel would be a lean
    // away from it half the time. What reads as a decision is that the body
    // arrives in one hard shape on the first frame and then does absolutely
    // nothing.
    let snap = Pose::rest()
        .hips(0.0, -0.08, 0.0)
        .root(24.0, 0.0, 0.0)
        .spine(26.0, 0.0, 0.0)
        .chest(18.0, 0.0, 0.0)
        .head(10.0, 0.0, 0.0)
        .shoulder_l(30.0, 30.0, 0.0)
        .elbow_l(112.0)
        .shoulders(30.0, 30.0, 0.0)
        .elbows(112.0)
        .hip_l(78.0, 10.0, 0.0)
        .knee_l(100.0)
        .hip_r(78.0, 10.0, 0.0)
        .knee_r(100.0)
        .ankles(20.0, 0.0, 0.0);

    // Locked. Knees driven up and together, elbows pinned, chin down: a body
    // that has chosen and cannot change its mind.
    let locked = Pose::rest()
        .hips(0.0, -0.14, 0.02)
        .root(40.0, 0.0, 0.0)
        .spine(40.0, 0.0, 0.0)
        .chest(30.0, 0.0, 0.0)
        .head(22.0, 0.0, 0.0)
        .shoulders(34.0, 22.0, 0.0)
        .elbows(140.0)
        .wrists(-20.0, 0.0, 0.0)
        .hips_both(104.0, 6.0, 0.0)
        .knees(132.0)
        .ankles(26.0, 0.0, 0.0);

    // The last frame that cannot be hit, four frames in. It is the same shape:
    // holding it right up to the boundary is what makes the release read as an
    // event rather than as the clip continuing.
    let held = locked.hips(0.0, -0.13, 0.02).head(20.0, 0.0, 0.0);

    // Vulnerable, and falling. Everything opens at once -- knees down, arms
    // out, chest up -- because the thing being communicated is that the cover
    // has gone, not that the character is doing something new.
    let open = Pose::rest()
        .hips(0.0, -0.04, -0.02)
        .root(10.0, 0.0, 0.0)
        .spine(8.0, 0.0, 0.0)
        .chest(2.0, 0.0, 0.0)
        .head(-12.0, 0.0, 0.0)
        .shoulders(-30.0, 76.0, 0.0)
        .elbows(34.0)
        .hips_both(34.0, 16.0, 0.0)
        .knees(64.0)
        .ankles(-6.0, 0.0, 0.0);

    // Legs down, reaching for a floor that is not there yet. This is the pose
    // the fall has to take over from, so it is a falling shape rather than a
    // landing one.
    let drop = Pose::rest()
        .hips(0.0, 0.02, -0.04)
        .root(-4.0, 0.0, 0.0)
        .spine(2.0, 0.0, 0.0)
        .chest(-2.0, 0.0, 0.0)
        .head(-16.0, 0.0, 0.0)
        .shoulders(-44.0, 58.0, 0.0)
        .elbows(46.0)
        .hips_both(8.0, 12.0, 0.0)
        .knees(38.0)
        .ankles(-12.0, 0.0, 0.0);

    let fall = Pose::rest()
        .hips(0.0, 0.0, -0.03)
        .root(2.0, 0.0, 0.0)
        .spine(4.0, 0.0, 0.0)
        .chest(0.0, 0.0, 0.0)
        .head(-12.0, 0.0, 0.0)
        .shoulders(-36.0, 48.0, 0.0)
        .elbows(52.0)
        .hips_both(16.0, 10.0, 0.0)
        .knees(48.0)
        .ankles(-8.0, 0.0, 0.0);

    Recipe {
        clip,
        looseness: Looseness::CRISP,
        notes: "Once per airtime, and it has to read as a decision rather than \
                as flight -- so the body arrives in one hard shape on the first \
                frame and then does nothing at all until the cover runs out. \
                Crisp, because anything that trails or settles reads as \
                floating. One clip plays for every direction an airdodge can \
                go, so it commits in shape rather than in a side: a lean toward \
                the travel would be a lean away from it half the time. The \
                break comes four frames in, not ten -- `Action::invulnerable` \
                measures an airdodge against the grounded dodge's numbers, so a \
                sixteen-frame airdodge spends most of itself exposed."
            .into(),
        keys: vec![
            Key::eased(0, snap, Ease::STRIKE),
            Key::eased(1, locked, Ease::HOLD),
            Key::eased(brk, held, Ease::STRIKE),
            Key::eased(f_open, open, Ease::OUT),
            Key::eased(f_drop, drop, Ease::SMOOTH),
            Key::eased(f_last, fall, Ease::SMOOTH),
        ],
    }
}

#[cfg(test)]
mod scratch {
    #[test]
    fn report() {
        let skeleton = view::pose::reference();
        let s = super::stance();
        println!(
            "STANCE sole {:+.3} ankles {:.3}/{:.3}",
            s.lowest_foot(skeleton),
            s.joint_at(view::skeleton::Joint::FootL)[1],
            s.joint_at(view::skeleton::Joint::FootR)[1]
        );
        for r in super::clips() {
            println!("--- {} ({} frames)", r.clip.name(), r.clip.length());
            for k in &r.keys {
                use view::skeleton::Joint::*;
                let d = |j, c| k.pose.degrees(j, c);
                println!(
                    "  key {:<3} thigh.l {:.0}/{:.0} knee.l {:.0} thigh.r {:.0}/{:.0} knee.r {:.0} arm.l {:.0} arm.r {:.0}",
                    k.frame,
                    d(ThighL, 0),
                    d(ThighL, 1),
                    d(ShinL, 0),
                    d(ThighR, 0),
                    d(ThighR, 1),
                    d(ShinR, 0),
                    d(ArmL, 0),
                    d(ArmR, 0),
                );
                let bad = k.pose.violations(skeleton);
                if !bad.is_empty() {
                    println!(
                        "  key {}: {:?}",
                        k.frame,
                        bad.iter()
                            .map(|(j, c, v)| format!("{} c{c}={v:.0}", j.name()))
                            .collect::<Vec<_>>()
                    );
                }
            }
            let b = crate::bake::bake(&r);
            let mut worst = (0.0f32, 0usize, String::new());
            for (i, pair) in b.frames.windows(2).enumerate() {
                for j in view::skeleton::JOINTS {
                    let (a, c) = (pair[0].angles(j), pair[1].angles(j));
                    for k in 0..3 {
                        let d = (c[k] - a[k]).to_degrees().abs();
                        if d > worst.0 {
                            worst = (d, i, format!("{} c{k}", j.name()));
                        }
                    }
                }
            }
            println!(
                "  worst jump {:.0} deg at frame {} ({})",
                worst.0, worst.1, worst.2
            );
            for (i, p) in b.frames.iter().enumerate() {
                println!(
                    "   f{i:<3} head_y {:.2}  hip_y {:.2}  ankles {:.3}/{:.3}  sole {:+.3}",
                    p.joint_at(view::skeleton::Joint::Head)[1],
                    p.joint_at(view::skeleton::Joint::Root)[1],
                    p.joint_at(view::skeleton::Joint::FootL)[1],
                    p.joint_at(view::skeleton::Joint::FootR)[1],
                    p.lowest_foot(skeleton)
                );
            }
        }
    }
}
