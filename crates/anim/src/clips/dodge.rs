//! Dodging: the universal defensive verb, on the ground and in the air.
//!
//! A dodge beats a read and loses to a delayed attack. Both halves of that
//! sentence have to be on screen, because a player who cannot see where the
//! invulnerability ended cannot learn when to hold an attack back.
//!
//! ## A dive roll, and every class rolls its own (2026-10-21)
//!
//! The grounded dodge is a **dive roll**, and the body really goes over: the
//! clips here are only what a body does *inside* a roll -- the push, the dive,
//! the ball, the legs reaching for the floor, the catch -- and
//! `view::dodge::turned` turns the whole body over on top of them, about the
//! middle of the ball. Authoring the turn here as well would be two turns, and
//! the rig could not do it anyway: the root has fifty degrees of forward
//! swing, which is a body that folds, not one that inverts.
//!
//! Each class has its own set of four, built by one recipe from a [`Style`]:
//! when it goes over and how hard it comes up are `view::dodge::roll`'s, and
//! what its arms and legs do through it are here. The table of the six is in
//! `view::dodge`.
//!
//! ## The one thing these clips exist to communicate
//!
//! `Action::invulnerable` is true for the first `dodge_iframes()` frames and
//! false afterwards, and **the silhouette has to change on that frame**. So
//! every recipe is built around one boundary: balled and unchanging while the
//! frames are free, then opening -- legs out for the floor, arms off the body
//! -- the instant they are not. The frame it happens on is derived from the
//! Oven rather than typed, so retuning the window moves the break with it, and
//! the turn is timed off the same two numbers.
//!
//! ## Which clip plays is chosen from the velocity
//!
//! `view::dodge::way` picks the direction from the body's actual travel, not
//! from the key that was pressed, and the turn reads the same answer -- so a
//! roll goes over toward where the character really went.
//!
//! ## Nothing is planted during the burst
//!
//! A dodge covers about three metres in twenty-two frames, two thirds of it
//! while invulnerable, so there is no planted foot through the middle of one of
//! these. The only contacts worth arithmetic are the push-off and the catch at
//! the end, which `planted` below places against the ground the body actually
//! covers.

use crate::bake::{Key, Looseness, Recipe};
use crate::clips::locomotion::stance;
use crate::ease::Ease;
use sim::Class;
use view::clips::Clip;
use view::dodge::Way;
use view::pose::{ANKLE_ON_GROUND as GROUND, Pose};

/// Where the feet sit either side of the centre line, as in `locomotion`.
const L: f32 = -0.115;
const R: f32 = 0.115;

pub fn clips() -> Vec<Recipe> {
    let mut out = vec![air()];
    for class in sim::class::ALL_CLASSES {
        for way in view::dodge::WAYS {
            out.push(dive_roll(class, way));
        }
    }
    out
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

// ---------------------------------------------------------------------------
// The dive roll
// ---------------------------------------------------------------------------

/// The beats of a roll, in order. A [`Style`] dresses each one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Beat {
    /// Frame zero: already going, the only grounded frame at this end.
    Push,
    /// Both feet leaving.
    Off,
    /// Going over: the dive onto the hands, the sit-back, the lean to the side.
    Over,
    /// The ball, held unchanging to the last invulnerable frame.
    Ball,
    /// The break: legs out for the floor, arms off the body.
    Open,
    /// Upright again, feet on the floor and taking the weight.
    Catch,
    /// Coming up out of it.
    Rise,
    /// Back on the feet, not back on guard.
    Settle,
}

/// How one class rolls: what its body does at each beat, on top of the shared
/// roll. `view::dodge::roll` owns *when* it goes over; this owns *how* it looks
/// doing it.
struct Style {
    looseness: Looseness,
    notes: &'static str,
    /// Dress a beat. Given the shared pose, the way the roll is going and
    /// which side that is (`1` right, `-1` left, `0` forward or back), returns
    /// the class's.
    dress: fn(Pose, Beat, Way, f32) -> Pose,
}

fn style(class: Class) -> Style {
    match class {
        Class::Champion => Style {
            looseness: Looseness::MARTIAL,
            notes: "The soldier's roll, and the shape the other five vary. A \
                    dive onto the hands, one tight turn in a ball -- arms \
                    wrapped, chin down -- held unchanging while the frames are \
                    free, then the legs open for the floor on the frame the \
                    cover runs out and he comes up in a low crouch with the \
                    weapon hand forward. The turn itself is not here: \
                    `view::dodge` turns the whole body over about the middle \
                    of the ball.",
            dress: champion,
        },
        Class::Bulwark => Style {
            looseness: Looseness::HEAVY,
            notes: "The shield roll. Later and heavier than anybody's -- he \
                    starts over a frame after the Champion and is upright a \
                    frame later -- and lower, tucked in behind the shield, the \
                    shield arm across the ball the whole way round. He comes \
                    up behind it rather than out of it: the shield is the first \
                    thing back in front of him. HEAVY, because a big body \
                    carries past its marks.",
            dress: bulwark,
        },
        Class::ShadowReaver => Style {
            looseness: Looseness::CRISP,
            notes: "The twisting dive. The first to go over and the first back \
                    up, and a full twist through the turn on top of the roll \
                    (`view::dodge::roll`'s `twist`), so going forward or back \
                    she corkscrews. Long and flat into it with both blades \
                    swept back, the blades held out wide through the ball, and \
                    up light and low on the balls of her feet with them still \
                    out. CRISP: the turn is fast and nothing trails it.",
            dress: reaver,
        },
        Class::Elementalist => Style {
            looseness: Looseness::MARTIAL,
            notes: "The earth roll. An ordinary roll that ends somewhere \
                    different: out of the turn she does not stand, she slides \
                    on, low on one knee, her lead hand dragged flat along the \
                    floor she shapes, and only rises off it at the end. The \
                    hand is placed against the ground the body is still \
                    covering, so it drags rather than skates.",
            dress: elementalist,
        },
        Class::BloodMage => Style {
            looseness: Looseness::MARTIAL,
            notes: "The low tumble. The longest dive and the latest turn -- \
                    she goes long and flat before she goes over -- and sunk \
                    lower through the ball than anyone. The scythe arm never \
                    comes in: it trails out wide through the turn like the \
                    blade is dragging her round, and she unfurls low out of it \
                    with the arm still out.",
            dress: blood,
        },
        Class::DualMage => Style {
            looseness: Looseness::FLOATY,
            notes: "The aerial. She does not tuck: the body stays long, arched \
                    a little, legs together and toes pointed, arms spread like \
                    the wings she does not have yet, and `view::dodge` turns \
                    her over about her waist rather than about a ball -- a \
                    layout forward and back, a cartwheel to the side. The \
                    slowest turn of the six and the softest landing, on one \
                    foot and then the other. Even her blink, which travels \
                    nothing, turns over where it arrives.",
            dress: dual,
        },
    }
}

/// One class's roll, one way.
fn dive_roll(class: Class, way: Way) -> Recipe {
    let clip = view::dodge::clip(class, way);
    let len = clip.length();
    let brk = first_vulnerable(len);
    let roll = view::dodge::roll(class);
    let up = roll.upright();
    let last = len - 1;
    let speed = ground_speed();
    let style = style(class);
    let side = match way {
        Way::Right => 1.0,
        Way::Left => -1.0,
        Way::Forward | Way::Back => 0.0,
    };
    // The way a planted foot slides: against the travel.
    let away = match way {
        Way::Forward => [0.0, -1.0],
        Way::Back => [0.0, 1.0],
        Way::Right => [-1.0, 0.0],
        Way::Left => [1.0, 0.0],
    };

    // The beats, placed against the turn. Over a frame after it starts, the
    // ball two after that; both held clear of the break.
    let f_over = (roll.from + 1).clamp(3, brk.saturating_sub(4).max(3));
    let f_ball = (f_over + 2).min(brk - 2);
    let f_held = brk - 1;
    let f_catch = up.clamp(brk + 1, last - 4);
    let f_rise = f_catch + (last - f_catch) / 2;

    let dress = |pose: Pose, beat: Beat| (style.dress)(pose, beat, way, side);

    // The two feet the catch puts down, and where they have slid to by the
    // rise and the last frame. The rise has the lead foot still down and the
    // trailing one stepping under it.
    let (lead, trail) = catch_feet(way);
    let lead_then = planted(lead, f_catch as f32, f_rise as f32, away, speed);
    let lead_last = planted(lead, f_catch as f32, last as f32, away, speed);
    let trail_then = planted(trail, f_catch as f32, f_rise as f32, away, speed);

    let keys = vec![
        Key::eased(0, dress(push(way), Beat::Push), Ease::LINEAR),
        Key::eased(1, dress(off(way), Beat::Off), Ease::LINEAR),
        Key::eased(f_over, dress(over(way), Beat::Over), Ease::OUT),
        Key::eased(f_ball, dress(ball(), Beat::Ball), Ease::HOLD),
        Key::eased(f_held, dress(ball(), Beat::Ball), Ease::STRIKE),
        Key::eased(brk, dress(open(way), Beat::Open), Ease::LINEAR),
        // The feet go down after the dressing, so a class that sinks lower
        // into its landing bends its knees for it rather than putting its
        // feet through the floor.
        Key::eased(
            f_catch,
            feet(dress(catch(), Beat::Catch), lead, trail),
            Ease::OUT,
        ),
        Key::eased(
            f_rise,
            feet(dress(rise(), Beat::Rise), lead_then, trail_then),
            Ease::OUT,
        ),
        Key::eased(
            last,
            settle(dress(settle_body(), Beat::Settle), lead_last, way),
            Ease::SMOOTH,
        ),
    ];
    let keys = dedupe(keys);
    Recipe {
        clip,
        looseness: style.looseness,
        notes: style.notes.into(),
        keys,
    }
}

/// Keys at the same frame keep the later one: a short window can squeeze two
/// beats onto one frame, and a recipe may not have two keys there.
fn dedupe(keys: Vec<Key>) -> Vec<Key> {
    let mut out: Vec<Key> = Vec::new();
    for k in keys {
        if let Some(prev) = out.last_mut() {
            if prev.frame >= k.frame {
                *prev = Key {
                    frame: prev.frame,
                    ..k
                };
                continue;
            }
        }
        out.push(k);
    }
    out
}

/// Mirror a pose across the body for a roll to the left, the rolls being
/// authored to the right.
fn sided(pose: Pose, way: Way) -> Pose {
    if way == Way::Left {
        pose.mirrored()
    } else {
        pose
    }
}

// -- The shared beats --------------------------------------------------------

/// Frame zero. The simulation hands out the whole velocity on the first frame
/// of the action, so this is already going -- an anticipation pose here would
/// be the body winding up for something it has already done.
fn push(way: Way) -> Pose {
    match way {
        Way::Forward => stance()
            .hips(0.0, -0.14, 0.04)
            .root(20.0, 0.0, -6.0)
            .spine(16.0, 0.0, 4.0)
            .chest(8.0, 0.0, 5.0)
            .head(-22.0, 0.0, 2.0)
            .shoulder_l(30.0, 20.0, 0.0)
            .elbow_l(50.0)
            .shoulder_r(26.0, 18.0, 0.0)
            .elbow_r(48.0)
            .plant_l([L - 0.02, GROUND, 0.12])
            .plant_r([R + 0.02, GROUND, -0.24])
            .toe_l(-2.0)
            .toe_r(10.0),
        Way::Back => stance()
            .hips(0.0, -0.22, -0.04)
            .root(14.0, 0.0, -10.0)
            .spine(16.0, 0.0, 6.0)
            .chest(10.0, 0.0, 7.0)
            .head(-12.0, 0.0, 4.0)
            .shoulder_l(10.0, 20.0, 0.0)
            .elbow_l(70.0)
            .shoulder_r(6.0, 18.0, 0.0)
            .elbow_r(66.0)
            .plant_l([L - 0.03, GROUND, 0.10])
            .plant_r([R + 0.03, GROUND, -0.04])
            .toe_l(6.0)
            .toe_r(8.0),
        Way::Left | Way::Right => sided(
            stance()
                .hips(0.03, -0.14, 0.0)
                .root(10.0, 12.0, -8.0)
                .spine(8.0, 10.0, 6.0)
                .chest(4.0, 8.0, 8.0)
                .head(-8.0, -6.0, 12.0)
                .shoulder_l(-4.0, 24.0, 0.0)
                .elbow_l(60.0)
                .shoulder_r(22.0, 38.0, 0.0)
                .elbow_r(52.0)
                .plant_l([L - 0.07, GROUND, 0.06])
                .plant_r([R + 0.07, GROUND, -0.06])
                .toe_l(8.0)
                .toe_r(2.0),
            way,
        ),
    }
}

/// One frame later, and both feet have left. Placed with `plant` at a raised
/// ankle rather than by angles, because a foot keyed as angles this close to
/// the floor ends up through it when the springs undershoot.
fn off(way: Way) -> Pose {
    match way {
        Way::Forward => stance()
            .hips(0.0, -0.18, 0.08)
            .root(34.0, 0.0, -4.0)
            .spine(24.0, 0.0, 3.0)
            .chest(14.0, 0.0, 4.0)
            .head(-30.0, 0.0, 1.0)
            .shoulder_l(62.0, 18.0, 0.0)
            .elbow_l(40.0)
            .shoulder_r(56.0, 16.0, 0.0)
            .elbow_r(38.0)
            .plant_l([L - 0.02, GROUND + 0.12, 0.04])
            .plant_r([R + 0.02, GROUND + 0.06, -0.32])
            .toe_l(-10.0)
            .toe_r(16.0),
        Way::Back => Pose::rest()
            .hips(0.0, -0.24, -0.08)
            .root(24.0, 0.0, -6.0)
            .spine(24.0, 0.0, 4.0)
            .chest(16.0, 0.0, 4.0)
            .head(0.0, 0.0, 2.0)
            .shoulder_l(30.0, 22.0, 0.0)
            .elbow_l(80.0)
            .shoulder_r(26.0, 20.0, 0.0)
            .elbow_r(76.0)
            .hip_l(62.0, 10.0, 0.0)
            .knee_l(84.0)
            .hip_r(54.0, 8.0, 0.0)
            .knee_r(92.0)
            .toe_l(24.0)
            .toe_r(26.0),
        Way::Left | Way::Right => sided(
            stance()
                .hips(0.06, -0.18, 0.0)
                .root(14.0, 20.0, -6.0)
                .spine(12.0, 16.0, 4.0)
                .chest(6.0, 12.0, 6.0)
                .head(0.0, -10.0, 14.0)
                .shoulder_l(10.0, 28.0, 0.0)
                .elbow_l(70.0)
                .shoulder_r(36.0, 66.0, 0.0)
                .elbow_r(36.0)
                .plant_l([L - 0.11, GROUND + 0.16, 0.04])
                .plant_r([R + 0.13, GROUND + 0.18, -0.02])
                .toe_l(2.0)
                .toe_r(0.0),
            way,
        ),
    }
}

/// Going over. Forward it is the dive: flat, hands reaching down for the floor
/// the shoulders are about to roll across. Back it is the sit: knees up, chin
/// down, hands forward. To the side it is the lean, the lead arm reaching for
/// the floor it is about to pass over.
fn over(way: Way) -> Pose {
    match way {
        // A hip's swing is measured against the root, and the root is leaning
        // forty-six degrees -- so a trailing leg is a small positive number,
        // and arms swung a hundred and ten degrees off a chest leaning a
        // hundred point at the floor, not ahead.
        Way::Forward => Pose::rest()
            .hips(0.0, -0.24, 0.14)
            .root(46.0, 0.0, 0.0)
            .spine(32.0, 0.0, 0.0)
            .chest(20.0, 0.0, 0.0)
            .head(-6.0, 0.0, 0.0)
            .shoulder_l(108.0, 16.0, 0.0)
            .elbow_l(26.0)
            .shoulder_r(112.0, 14.0, 0.0)
            .elbow_r(24.0)
            .hip_l(30.0, 8.0, 0.0)
            .knee_l(64.0)
            .hip_r(20.0, 6.0, 0.0)
            .knee_r(84.0)
            .toe_l(30.0)
            .toe_r(32.0),
        Way::Back => Pose::rest()
            .hips(0.0, -0.30, -0.10)
            .root(30.0, 0.0, 0.0)
            .spine(36.0, 0.0, 0.0)
            .chest(26.0, 0.0, 0.0)
            .head(28.0, 0.0, 0.0)
            .shoulder_l(56.0, 20.0, 0.0)
            .elbow_l(100.0)
            .shoulder_r(52.0, 20.0, 0.0)
            .elbow_r(104.0)
            .hip_l(98.0, 12.0, 0.0)
            .knee_l(118.0)
            .hip_r(94.0, 10.0, 0.0)
            .knee_r(122.0)
            .toe_l(24.0)
            .toe_r(24.0),
        Way::Left | Way::Right => sided(
            Pose::rest()
                .hips(0.10, -0.28, 0.04)
                .root(26.0, 22.0, 0.0)
                .spine(24.0, 16.0, 0.0)
                .chest(16.0, 10.0, 0.0)
                .head(16.0, -8.0, 0.0)
                .shoulder_l(60.0, 24.0, 0.0)
                .elbow_l(100.0)
                .shoulder_r(70.0, 58.0, 0.0)
                .elbow_r(30.0)
                .hip_l(76.0, 14.0, 0.0)
                .knee_l(104.0)
                .hip_r(84.0, 16.0, 0.0)
                .knee_r(112.0)
                .toe_l(22.0)
                .toe_r(22.0),
            way,
        ),
    }
}

/// The ball: knees to the chest, arms wrapped round the shins, chin down. The
/// smallest the silhouette gets, and the same whichever way the roll goes --
/// the turn is what tells forward from sideways now, not the curl. The trunk
/// and the thighs both point ahead and lie along each other, which is what
/// makes it a ball and not a crouch.
fn ball() -> Pose {
    Pose::rest()
        .hips(0.0, -0.34, 0.10)
        .root(44.0, 0.0, 0.0)
        .spine(40.0, 0.0, 0.0)
        .chest(30.0, 0.0, 0.0)
        .head(36.0, 0.0, 0.0)
        .shoulder_l(66.0, 24.0, 0.0)
        .elbow_l(124.0)
        .shoulder_r(70.0, 22.0, 0.0)
        .elbow_r(126.0)
        .hip_l(110.0, 12.0, 0.0)
        .knee_l(138.0)
        .hip_r(112.0, 10.0, 0.0)
        .knee_r(140.0)
        .toe_l(30.0)
        .toe_r(30.0)
}

/// The break. Most of the way round, the legs come out of the ball and reach
/// along the body for the floor -- which, the body being over, is ahead of it
/// going forward, behind it going back and below it to the side -- and the
/// arms come off the shins. Everything that changes on this frame is the
/// cover going.
fn open(way: Way) -> Pose {
    let base = Pose::rest()
        .hips(0.0, -0.32, 0.06)
        .root(30.0, 0.0, 0.0)
        .spine(28.0, 0.0, 0.0)
        .chest(16.0, 0.0, 0.0)
        .head(10.0, 0.0, 0.0)
        .shoulder_l(30.0, 40.0, 0.0)
        .elbow_l(70.0)
        .shoulder_r(34.0, 38.0, 0.0)
        .elbow_r(66.0)
        .hip_l(58.0, 12.0, 0.0)
        .knee_l(96.0)
        .hip_r(74.0, 10.0, 0.0)
        .knee_r(110.0)
        .toe_l(4.0)
        .toe_r(6.0);
    match way {
        Way::Forward | Way::Back => base,
        // Sideways the far leg reaches out under the travel.
        Way::Left | Way::Right => sided(base.hip_r(64.0, 26.0, 0.0), way),
    }
}

/// Where the catch puts the two feet down: the lead under the body, the other
/// behind it, both on the floor.
fn catch_feet(way: Way) -> ([f32; 3], [f32; 3]) {
    // The ankles a few centimetres up: on the balls of the feet.
    const UP: f32 = GROUND + 0.07;
    match way {
        Way::Forward => ([R + 0.04, UP, 0.20], [L - 0.04, UP, -0.10]),
        Way::Back => ([L - 0.04, UP, -0.18], [R + 0.04, UP, 0.10]),
        Way::Right => ([R + 0.16, UP, 0.04], [L - 0.02, UP, -0.06]),
        Way::Left => ([L - 0.16, UP, 0.04], [R + 0.02, UP, -0.06]),
    }
}

/// Upright and down: deep, weight over the lead foot, hands low and out for
/// balance -- nowhere near a guard, which is the point. The feet are put down
/// by [`feet`], once the class has had its say.
fn catch() -> Pose {
    Pose::rest()
        .hips(0.0, -0.38, 0.0)
        .root(26.0, 0.0, -4.0)
        .spine(18.0, 0.0, 4.0)
        .chest(8.0, 0.0, 4.0)
        .head(-24.0, 0.0, -2.0)
        .shoulder_l(20.0, 40.0, 0.0)
        .elbow_l(56.0)
        .shoulder_r(28.0, 40.0, 0.0)
        .elbow_r(50.0)
}

/// Coming up. The lead foot still down and sliding back under a body that has
/// not stopped; the trailing one taking weight.
fn rise() -> Pose {
    Pose::rest()
        .hips(0.0, -0.24, 0.0)
        .root(16.0, 0.0, -6.0)
        .spine(10.0, 0.0, 4.0)
        .chest(4.0, 0.0, 5.0)
        .head(-14.0, 0.0, 2.0)
        .shoulder_l(12.0, 36.0, 0.0)
        .elbow_l(54.0)
        .shoulder_r(14.0, 36.0, 0.0)
        .elbow_r(48.0)
}

/// Back on the feet, not back on guard. The hips and legs are close to the
/// fighting stance so the hand-off to idle does not pop; the arms are not,
/// because the recovery frames are the ones you get punished in.
fn settle_body() -> Pose {
    stance()
        .hips(0.0, -0.10, 0.0)
        .root(6.0, 0.0, -10.0)
        .spine(7.0, 0.0, 5.0)
        .chest(3.0, 0.0, 6.0)
        .head(-5.0, 0.0, 3.0)
        .shoulder_l(4.0, 30.0, 0.0)
        .elbow_l(44.0)
        .shoulder_r(0.0, 32.0, 0.0)
        .elbow_r(38.0)
}

/// The settled feet: the lead where it slid to, the other the stance's.
fn settle(pose: Pose, lead: [f32; 3], way: Way) -> Pose {
    let pose = pose
        .plant_l([L - 0.03, GROUND, 0.16])
        .toe_l(0.0)
        .plant_r([R + 0.03, GROUND + 0.032, -0.15])
        .toe_floor_r();
    match way {
        Way::Back | Way::Left => pose.plant_l(lead).toe_l(2.0),
        Way::Forward | Way::Right => pose.plant_r(lead).toe_r(2.0),
    }
}

/// Both feet onto the floor, whichever side leads.
///
/// A planted foot takes whatever angle its shin ended at, which points the
/// sole into the floor, so each is laid back on it afterwards. Both onto the
/// ball of the foot, heels a little up: this deep, with the shin pitched this
/// far over it, an ankle has not got the range to put a heel down, and asked
/// to it drives the toe through the floor instead.
fn feet(pose: Pose, lead: [f32; 3], trail: [f32; 3]) -> Pose {
    if lead[0] > trail[0] {
        pose.plant_r(lead)
            .toe_floor_r()
            .plant_l(trail)
            .toe_floor_l()
    } else {
        pose.plant_l(lead)
            .toe_floor_l()
            .plant_r(trail)
            .toe_floor_r()
    }
}

// -- The six ------------------------------------------------------------------

/// The soldier's roll: the shared one, with the weapon hand coming up forward
/// at the end.
fn champion(pose: Pose, beat: Beat, _way: Way, _side: f32) -> Pose {
    match beat {
        Beat::Catch => pose.shoulder_r(46.0, 20.0, 0.0).elbow_r(40.0),
        Beat::Rise => pose.shoulder_r(36.0, 18.0, 0.0).elbow_r(44.0),
        Beat::Settle => pose.shoulder_r(22.0, 18.0, 0.0).elbow_r(46.0),
        _ => pose,
    }
}

/// The shield arm across the front of whatever the body is doing: the
/// forearm level across the chest, the shield on it facing out.
fn shield(pose: Pose) -> Pose {
    pose.shoulder_l(74.0, -20.0, 0.0)
        .elbow_l(104.0)
        .wrist_l(-6.0, 0.0, 0.0)
}

fn bulwark(pose: Pose, beat: Beat, _way: Way, _side: f32) -> Pose {
    match beat {
        Beat::Push | Beat::Off => pose.shoulder_l(52.0, -6.0, 0.0).elbow_l(98.0),
        Beat::Over => shield(pose),
        // Lower and broader through the ball: he does not fold as small as a
        // lighter body, so the knees sit wider and the head stays tucked in
        // behind the shield rather than down between them.
        Beat::Ball => shield(pose.hips(0.0, -0.38, 0.08).head(26.0, 0.0, 0.0))
            .hip_l(104.0, 20.0, 0.0)
            .hip_r(106.0, 18.0, 0.0),
        Beat::Open => shield(pose),
        // Up behind the shield, deeper than anyone.
        Beat::Catch => shield(pose.hips(0.0, -0.44, 0.0).root(22.0, 0.0, -6.0)),
        Beat::Rise => shield(pose.hips(0.0, -0.30, 0.0)).shoulder_l(66.0, -14.0, 0.0),
        Beat::Settle => pose.shoulder_l(44.0, -4.0, 0.0).elbow_l(96.0),
    }
}

fn reaver(pose: Pose, beat: Beat, way: Way, _side: f32) -> Pose {
    // Both blades swept back and out: the arms behind the body and spread.
    let swept = |p: Pose| {
        p.shoulder_l(-40.0, 56.0, 0.0)
            .elbow_l(20.0)
            .shoulder_r(-40.0, 56.0, 0.0)
            .elbow_r(20.0)
    };
    // Held out wide through the turn, a blade at each side of the ball.
    let wide = |p: Pose| {
        p.shoulder_l(30.0, 84.0, 0.0)
            .elbow_l(24.0)
            .shoulder_r(30.0, 84.0, 0.0)
            .elbow_r(24.0)
    };
    match beat {
        // Forward, longer and flatter than the soldier's dive -- the hips
        // further ahead, the trunk lower -- with the blades swept back rather
        // than reaching. Any other way the blades go straight out to the
        // ball's: there are not the frames to sweep them back and out again.
        Beat::Push | Beat::Off if way == Way::Forward => swept(pose),
        Beat::Over if way == Way::Forward => {
            swept(pose.hips(0.0, -0.25, 0.18).root(48.0, 0.0, 0.0))
        }
        // CRISP arrives fast, and the trunk's bend into a sideways roll in two
        // frames is a shoulder over the motion ceiling; so she is already some
        // of the way into it as her feet leave.
        Beat::Off => pose.blend(&over(way), 0.55),
        Beat::Push | Beat::Over => pose,
        Beat::Ball => wide(pose),
        Beat::Open => wide(pose),
        // Up light: not so deep, on the balls of the feet, blades out low.
        Beat::Catch => pose
            .hips(0.0, -0.30, 0.0)
            .shoulder_l(-24.0, 50.0, 0.0)
            .elbow_l(18.0)
            .shoulder_r(-24.0, 50.0, 0.0)
            .elbow_r(18.0)
            .toe_l(12.0)
            .toe_r(12.0),
        Beat::Rise => pose
            .shoulder_l(-12.0, 46.0, 0.0)
            .elbow_l(24.0)
            .shoulder_r(-12.0, 46.0, 0.0)
            .elbow_r(24.0),
        Beat::Settle => pose,
    }
}

fn elementalist(pose: Pose, beat: Beat, way: Way, side: f32) -> Pose {
    // The hand that drags: the lead side's going sideways, otherwise her left.
    let left = side <= 0.0;
    match beat {
        // Out of the turn sliding on: one knee nearly down, the trunk forward
        // over it, the hand flat on the floor beside the lead foot.
        Beat::Catch | Beat::Rise => {
            let (deep, lean) = if beat == Beat::Catch {
                (-0.50, 34.0)
            } else {
                (-0.40, 28.0)
            };
            let hand = drag_hand(way, left);
            let body = pose
                .hips(0.0, deep, 0.0)
                .root(lean, 0.0, -6.0)
                .spine(22.0, 0.0, 0.0);
            if left {
                body.reach_l(hand).shoulder_r(10.0, 64.0, 0.0).elbow_r(30.0)
            } else {
                body.reach_r(hand).shoulder_l(10.0, 64.0, 0.0).elbow_l(30.0)
            }
        }
        _ => pose,
    }
}

/// Where her dragging hand is on the floor, in character space: low beside
/// the lead foot, a little ahead of the hips.
fn drag_hand(way: Way, left: bool) -> [f32; 3] {
    let x = if left { L - 0.20 } else { R + 0.20 };
    let z = match way {
        Way::Back => -0.10,
        _ => 0.24,
    };
    [x, 0.06, z]
}

fn blood(pose: Pose, beat: Beat, way: Way, side: f32) -> Pose {
    // The scythe arm out wide and nearly straight, trailing.
    let out = |p: Pose| p.shoulder_r(-10.0, 92.0, 0.0).elbow_r(12.0);
    // Through the ball, out to the side is along the axis a roll forward or
    // back turns about, so the blade sweeps round her; a roll to the side
    // would sweep it through the floor, so going sideways it comes in for the
    // turn and goes out again as she opens.
    let trailing = move |p: Pose| if side == 0.0 { out(p) } else { p };
    match beat {
        Beat::Push | Beat::Off => pose,
        // Long and flat: she goes further before she goes over.
        Beat::Over => match way {
            Way::Forward => trailing(pose.hips(0.0, -0.30, 0.22).root(50.0, 0.0, 0.0)),
            _ => trailing(pose.hips(0.0, -0.32, 0.06)),
        },
        // Sunk lower through the ball than anyone, the scythe arm never in.
        Beat::Ball => trailing(pose.hips(0.0, -0.40, 0.12)),
        Beat::Open => trailing(pose.hips(0.0, -0.36, 0.06)),
        // Unfurling low.
        Beat::Catch => out(pose.hips(0.0, -0.44, 0.0)),
        Beat::Rise => out(pose.hips(0.0, -0.32, 0.0)),
        Beat::Settle => pose.shoulder_r(-8.0, 50.0, 0.0).elbow_r(30.0),
    }
}

fn dual(pose: Pose, beat: Beat, way: Way, side: f32) -> Pose {
    // Laid out: long, arched a little, legs together and toes pointed, arms
    // spread wide like wings. What she turns over in place of a ball.
    let laid_out = Pose::rest()
        .hips(0.0, 0.04, 0.0)
        .root(-6.0, 0.0, 0.0)
        .spine(-12.0, 0.0, 0.0)
        .chest(-8.0, 0.0, 0.0)
        .head(-10.0, 0.0, 0.0)
        .shoulder_l(10.0, 96.0, 0.0)
        .elbow_l(12.0)
        .wrist_l(10.0, 0.0, 0.0)
        .shoulder_r(10.0, 96.0, 0.0)
        .elbow_r(12.0)
        .wrist_r(10.0, 0.0, 0.0)
        .hip_l(-4.0, 3.0, 0.0)
        .knee_l(6.0)
        .hip_r(-2.0, 3.0, 0.0)
        .knee_r(8.0)
        .toe_l(44.0)
        .toe_r(44.0);
    match beat {
        Beat::Push | Beat::Off => pose.shoulder_l(10.0, 60.0, 0.0).shoulder_r(10.0, 60.0, 0.0),
        Beat::Over => laid_out.hips(0.0, 0.0, 0.0).spine(-6.0, 0.0, 0.0),
        Beat::Ball => laid_out,
        // Legs parting for the floor, one reaching down ahead of the other.
        Beat::Open => {
            let open = laid_out
                .hips(0.0, 0.0, 0.0)
                .hip_l(24.0, 6.0, 0.0)
                .knee_l(20.0)
                .hip_r(-14.0, 4.0, 0.0)
                .knee_r(10.0);
            if way == Way::Back || side < 0.0 {
                open.mirrored()
            } else {
                open
            }
        }
        // Down on one foot and then the other, the arms still out.
        Beat::Catch => pose
            .hips(0.0, -0.20, 0.0)
            .root(10.0, 0.0, 0.0)
            .head(-10.0, 0.0, 0.0)
            .shoulder_l(0.0, 84.0, 0.0)
            .elbow_l(16.0)
            .shoulder_r(0.0, 84.0, 0.0)
            .elbow_r(16.0),
        Beat::Rise => pose
            .hips(0.0, -0.12, 0.0)
            .shoulder_l(-6.0, 64.0, 0.0)
            .elbow_l(24.0)
            .shoulder_r(-6.0, 64.0, 0.0)
            .elbow_r(24.0),
        Beat::Settle => pose,
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
