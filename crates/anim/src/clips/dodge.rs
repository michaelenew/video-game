//! Dodging: the universal defensive verb, on the ground and in the air.
//!
//! A dodge beats a read and loses to a delayed attack. Both halves of that
//! sentence have to be on screen, because a player who cannot see where the
//! invulnerability ended cannot learn when to hold an attack back.
//!
//! ## A roll, and every class rolls its own (2026-10-21)
//!
//! The grounded dodge is a **roll**, and the body really goes over: a hop off
//! both feet, chin and knees to the chest, one turn along the floor, and up.
//! The clips here are only what a body does *inside* it -- the push, the hop,
//! the tuck, the ball, the legs reaching for the floor, the catch -- and
//! `view::dodge::turned` turns the whole body over on top of them, about the
//! middle of the ball. Authoring the turn here as well would be two turns, and
//! the rig could not do it anyway: the root has fifty degrees of forward
//! swing, which is a body that folds, not one that inverts.
//!
//! Back, it is a backward roll. **To the side it is the forward roll**: the
//! body is turned to face the travel first and back as it rises, so a side
//! clip is the forward one with a lean off the shoulder it goes over.
//!
//! Each class has its own set of four, built by one recipe from a [`Style`]:
//! when it goes over and how hard it comes up are `view::dodge::roll`'s, and
//! what its arms carry and how it lands are here. The table of the six is in
//! `view::dodge`.
//!
//! ## The one thing these clips exist to communicate
//!
//! `Action::invulnerable` is true for the first `dodge_iframes()` frames and
//! false afterwards, and **the silhouette must not open while they are free**:
//! balled and unchanging through the cover, and the legs out for the floor no
//! earlier than the frame it goes. Since 2026-10-22 it opens a little later
//! than that -- when the body is most of the way round (`Roll::opens`),
//! because a roll stretched over the recovery, so that it reads at speed,
//! is only half way round when the cover goes. The frames are derived from the
//! Oven rather than typed, so retuning the window moves them with it.
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
// The roll
// ---------------------------------------------------------------------------

/// The beats of a roll, in order. A [`Style`] dresses each one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Beat {
    /// Frame zero: already going, the only grounded frame at this end.
    Push,
    /// The launch: the back leg driving, the front one lifting, the arms
    /// swinging through.
    Off,
    /// In the air: long, the hands out ahead, the legs trailing.
    Dive,
    /// The hands reach the floor: the head tucks, the lead shoulder drops,
    /// the knees start up.
    Contact,
    /// The ball, rolling, held unchanging until the body is most of the way
    /// round (`view::dodge::Roll::opens`), never before the cover goes.
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

/// How one class rolls: what its arms carry and how it lands, on top of the
/// shared roll. `view::dodge::roll` owns *when* it goes over.
struct Style {
    looseness: Looseness,
    notes: &'static str,
    /// Dress a beat. Given the shared pose and the way the roll is going,
    /// returns the class's.
    dress: fn(Pose, Beat, Way) -> Pose,
}

fn style(class: Class) -> Style {
    match class {
        Class::Champion => Style {
            looseness: Looseness::MARTIAL,
            notes: "The dive roll everyone shares, as the Champion does it. \
                    Off the back foot into the air, long, arms up in front of \
                    the chest and legs trailing, pitching forward as he flies; \
                    as the floor arrives the head tucks, the arms fold and the \
                    knees come up, and he goes over the right shoulder in a \
                    ball, held unchanging until he is most of the way round \
                    -- never before the cover runs out. Then the legs open \
                    for the floor and he comes up in a low crouch with the weapon hand forward. \
                    The turn, the shoulder and the height of the dive are not \
                    here: `view::dodge` turns and carries the whole body on \
                    top of this, and to the side turns him to face the travel \
                    first, so a roll to the side is this roll.",
            dress: champion,
        },
        Class::Bulwark => Style {
            looseness: Looseness::MARTIAL,
            notes: "The dive roll, heavier: a lower dive, to the floor a frame \
                    later and up a frame later, the shield arm across him the \
                    whole way round. He comes up behind the shield, deeper \
                    than anybody.",
            dress: bulwark,
        },
        Class::ShadowReaver => Style {
            looseness: Looseness::MARTIAL,
            notes: "The dive roll, highest and quickest: she leaves the floor \
                    highest and is first up, light and less deep, with the \
                    blades out low.",
            dress: reaver,
        },
        Class::Elementalist => Style {
            looseness: Looseness::MARTIAL,
            notes: "The dive roll, ending somewhere different: out of the \
                    turn she does not stand straight up, she slides on, low, \
                    her lead hand dragged flat along the floor, and only rises \
                    off it at the end.",
            dress: elementalist,
        },
        Class::BloodMage => Style {
            looseness: Looseness::MARTIAL,
            notes: "The dive roll, longest and flattest: she is a frame longer \
                    in the air and further over when she lands, and unfurls \
                    low out of it with the scythe arm thrown out wide behind \
                    her.",
            dress: blood,
        },
        Class::DualMage => Style {
            looseness: Looseness::MARTIAL,
            notes: "The dive roll, floating: the highest arc after the \
                    Reaver's and the slowest up, her arms opening wide like \
                    the wings she does not have yet.",
            dress: dual,
        },
    }
}

/// One class's roll, one way.
///
/// **To the side it is the forward roll.** `view::dodge::turned` turns the
/// body to face the travel before it goes over and back as it rises, so the
/// clip is the forward one, with only the head and the trunk leaning off the
/// shoulder it goes over -- the difference between rolling over the right
/// shoulder and the left.
fn dive_roll(class: Class, way: Way) -> Recipe {
    let clip = view::dodge::clip(class, way);
    let len = clip.length();
    let brk = first_vulnerable(len);
    let roll = view::dodge::roll(class);
    let up = roll.upright();
    let last = len - 1;
    let speed = ground_speed();
    let style = style(class);
    // The body's own way, once it has turned to face the travel.
    let body = if way == Way::Back {
        Way::Back
    } else {
        Way::Forward
    };
    // The way a planted foot slides: against the travel.
    let away = if body == Way::Back {
        [0.0, 1.0]
    } else {
        [0.0, -1.0]
    };

    // The beats, placed against the turn. Launched on the second frame and
    // long in the air from the third, **never still in the air**: the dive
    // keeps reaching -- legs trailing higher, hands going for the floor --
    // right up to the contact, where it tucks, balled three frames after.
    // Down on the feet as the turn finishes, then sinking deeper for two
    // frames as the knees take the weight before it rises: the follow-through
    // that says the body has mass (2026-10-22, from play: "made of feathers").
    let f_contact = roll.contact().clamp(5, brk - 1);
    let f_dive = 3;
    let f_reach = f_contact - 1;
    let f_open = roll.opens().clamp(brk, last - 6);
    let f_ball = (f_contact + 3).min(f_open - 1);
    let f_held = f_open - 1;
    let f_catch = up.clamp(f_open + 1, last - 5);
    let f_absorb = (f_catch + 2).min(last - 3);
    let f_rise = f_absorb + (last - f_absorb) / 2;

    // Off the shoulder it goes over.
    let lean = |pose: Pose| shoulder(pose, way);
    let dress = |pose: Pose, beat: Beat| (style.dress)(pose, beat, body);

    // The two feet the catch puts down, and where they have slid to by each
    // key after.
    let (lead, trail) = catch_feet(body);
    let slid = |at: [f32; 3], f: u16| planted(at, f_catch as f32, f as f32, away, speed);

    let keys = vec![
        Key::eased(0, dress(push(body), Beat::Push), Ease::LINEAR),
        Key::eased(1, dress(off(body), Beat::Off), Ease::LINEAR),
        Key::eased(f_dive, dress(dive(body), Beat::Dive), Ease::LINEAR),
        Key::eased(f_reach, dress(reach(body), Beat::Dive), Ease::IN),
        Key::eased(
            f_contact,
            lean(dress(contact(body), Beat::Contact)),
            Ease::LINEAR,
        ),
        Key::eased(f_ball, lean(dress(ball(), Beat::Ball)), Ease::HOLD),
        Key::eased(f_held, lean(dress(ball(), Beat::Ball)), Ease::STRIKE),
        Key::eased(f_open, dress(open(), Beat::Open), Ease::LINEAR),
        // The feet go down after the dressing, so a class that sinks lower
        // into its landing bends its knees for it rather than putting its
        // feet through the floor.
        Key::eased(
            f_catch,
            feet(dress(catch(), Beat::Catch), lead, trail),
            Ease::OUT,
        ),
        Key::eased(
            f_absorb,
            feet(
                sunk(dress(catch(), Beat::Catch)),
                slid(lead, f_absorb),
                slid(trail, f_absorb),
            ),
            Ease::SMOOTH,
        ),
        Key::eased(
            f_rise,
            feet(
                dress(rise(), Beat::Rise),
                slid(lead, f_rise),
                slid(trail, f_rise),
            ),
            Ease::OUT,
        ),
        Key::eased(
            last,
            settle(dress(settle_body(), Beat::Settle), slid(lead, last), body),
            Ease::SMOOTH,
        ),
    ];
    Recipe {
        clip,
        looseness: style.looseness,
        notes: style.notes.into(),
        keys: dedupe(keys),
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

/// A shoulder roll's lean: going over the right shoulder the head tucks
/// toward the left one and the trunk tips onto the right, so the round of the
/// back runs from the right shoulder to the left hip -- the line the turn's
/// tipped axis rolls it along (`view::dodge::turned`). Over the left shoulder
/// rolling left; the right one every other way.
fn shoulder(pose: Pose, way: Way) -> Pose {
    let s = match way {
        Way::Left => -1.0,
        Way::Right | Way::Forward | Way::Back => 1.0,
    };
    let tip = |j: view::skeleton::Joint, side: f32, turn: f32| {
        move |p: Pose| {
            let mut q = p;
            q.set_degrees(j, 1, p.degrees(j, 1) + side * s);
            q.set_degrees(j, 2, p.degrees(j, 2) + turn * s);
            q
        }
    };
    use view::skeleton::Joint;
    let pose = tip(Joint::Spine, 10.0, -8.0)(pose);
    let pose = tip(Joint::Chest, 8.0, -10.0)(pose);
    tip(Joint::Head, -20.0, 24.0)(pose)
}

// -- The shared beats --------------------------------------------------------

/// Frame zero. The simulation hands out the whole velocity on the first frame
/// of the action, so this is already going -- an anticipation pose here would
/// be the body winding up for something it has already done.
fn push(body: Way) -> Pose {
    match body {
        Way::Back => stance()
            .hips(0.0, -0.22, -0.04)
            .root(14.0, 0.0, -10.0)
            .spine(16.0, 0.0, 6.0)
            .chest(10.0, 0.0, 7.0)
            .head(-6.0, 0.0, 4.0)
            .shoulder_l(20.0, 20.0, 0.0)
            .elbow_l(70.0)
            .shoulder_r(16.0, 18.0, 0.0)
            .elbow_r(66.0)
            .plant_l([L - 0.03, GROUND, 0.10])
            .plant_r([R + 0.03, GROUND, -0.04])
            .toe_l(6.0)
            .toe_r(8.0),
        _ => stance()
            .hips(0.0, -0.14, 0.04)
            .root(18.0, 0.0, -6.0)
            .spine(14.0, 0.0, 4.0)
            .chest(8.0, 0.0, 5.0)
            .head(-16.0, 0.0, 2.0)
            .shoulder_l(30.0, 20.0, 0.0)
            .elbow_l(50.0)
            .shoulder_r(26.0, 18.0, 0.0)
            .elbow_r(48.0)
            .plant_l([L - 0.02, GROUND, 0.12])
            .plant_r([R + 0.02, GROUND, -0.20])
            .toe_l(-2.0)
            .toe_r(10.0),
    }
}

/// The launch. Forward, the back leg has just driven off and is straight
/// behind, the front knee coming up, the trunk going out over it and the arms
/// swinging forward and up. Backward, the hop back, sitting into it. Legs
/// keyed as angles: nothing is on the floor to plant.
fn off(body: Way) -> Pose {
    match body {
        Way::Back => Pose::rest()
            .hips(0.0, -0.26, -0.08)
            .root(24.0, 0.0, -6.0)
            .spine(26.0, 0.0, 4.0)
            .chest(18.0, 0.0, 4.0)
            .head(10.0, 0.0, 2.0)
            .shoulder_l(40.0, 22.0, 0.0)
            .elbow_l(84.0)
            .shoulder_r(36.0, 20.0, 0.0)
            .elbow_r(80.0)
            .hip_l(66.0, 10.0, 0.0)
            .knee_l(88.0)
            .hip_r(58.0, 8.0, 0.0)
            .knee_r(96.0)
            .toe_l(24.0)
            .toe_r(26.0),
        _ => Pose::rest()
            .hips(0.0, -0.12, 0.08)
            .root(28.0, 0.0, -4.0)
            .spine(14.0, 0.0, 3.0)
            .chest(8.0, 0.0, 4.0)
            .head(-24.0, 0.0, 1.0)
            .shoulder_l(76.0, 16.0, 0.0)
            .elbow_l(52.0)
            .shoulder_r(72.0, 14.0, 0.0)
            .elbow_r(52.0)
            .hip_l(64.0, 8.0, 0.0)
            .knee_l(80.0)
            .hip_r(-4.0, 6.0, 0.0)
            .knee_r(20.0)
            .toe_l(20.0)
            .toe_r(40.0),
    }
}

/// In the air. Long from the head to the toes: the arms up in front of the
/// chest -- in front of it, not stretched overhead, because it is the
/// shoulder that meets the floor, not the hands -- the head up looking where
/// it is going, the legs trailing off the push with the toes pointed. The whole-body turn
/// pitches this forward as it flies, so by the contact the hands are down and
/// the feet up behind. Backward, it is the hop back: sat in the air, knees up,
/// chin down, hands forward, about to land on the back.
fn dive(body: Way) -> Pose {
    match body {
        Way::Back => Pose::rest()
            .hips(0.0, -0.26, -0.06)
            .root(26.0, 0.0, 0.0)
            .spine(30.0, 0.0, 0.0)
            .chest(22.0, 0.0, 0.0)
            .head(26.0, 0.0, 0.0)
            .shoulder_l(62.0, 20.0, 0.0)
            .elbow_l(96.0)
            .shoulder_r(58.0, 20.0, 0.0)
            .elbow_r(100.0)
            .hip_l(92.0, 12.0, 0.0)
            .knee_l(112.0)
            .hip_r(88.0, 10.0, 0.0)
            .knee_r(118.0)
            .toe_l(26.0)
            .toe_r(26.0),
        _ => Pose::rest()
            .hips(0.0, -0.08, 0.06)
            .root(16.0, 0.0, 0.0)
            .spine(12.0, 0.0, 0.0)
            .chest(8.0, 0.0, 0.0)
            .head(-20.0, 0.0, 0.0)
            .shoulder_l(92.0, 18.0, 0.0)
            .elbow_l(64.0)
            .shoulder_r(96.0, 16.0, 0.0)
            .elbow_r(60.0)
            .hip_l(4.0, 6.0, 0.0)
            .knee_l(28.0)
            .hip_r(-10.0, 4.0, 0.0)
            .knee_r(46.0)
            .toe_l(40.0)
            .toe_r(42.0),
    }
}

/// The end of the flight: still long, but the body is falling now -- the
/// legs trailing higher behind, the hands going down for the floor, the head
/// still up, the back starting to round. What keeps the dive moving between
/// leaving the floor and meeting it, rather than hanging in one pose.
fn reach(body: Way) -> Pose {
    match body {
        Way::Back => dive(Way::Back)
            .hips(0.0, -0.30, -0.08)
            .head(30.0, 0.0, 0.0)
            .hip_l(100.0, 12.0, 0.0)
            .hip_r(96.0, 10.0, 0.0),
        _ => Pose::rest()
            .hips(0.0, -0.12, 0.08)
            .root(20.0, 0.0, 0.0)
            .spine(18.0, 0.0, 0.0)
            .chest(12.0, 0.0, 0.0)
            .head(-12.0, 0.0, 0.0)
            .shoulder_l(98.0, 20.0, 0.0)
            .elbow_l(70.0)
            .shoulder_r(100.0, 18.0, 0.0)
            .elbow_r(68.0)
            .hip_l(16.0, 6.0, 0.0)
            .knee_l(50.0)
            .hip_r(-16.0, 4.0, 0.0)
            .knee_r(40.0)
            .toe_l(40.0)
            .toe_r(42.0),
    }
}

/// The landing taken: the same body as the catch, sunk lower and further
/// over the front foot, two frames after it -- the weight arriving.
fn sunk(pose: Pose) -> Pose {
    let [x, y, z] = pose.root_offset();
    let mut p = pose.hips(x, y - 0.07, z + 0.02);
    for (j, more) in [
        (view::skeleton::Joint::Root, 6.0),
        (view::skeleton::Joint::Spine, 4.0),
        (view::skeleton::Joint::Head, -4.0),
    ] {
        p.set_degrees(j, 0, p.degrees(j, 0) + more);
    }
    p
}

/// The floor arrives. The arms fold in, the head tucks under; the back starts to round and the
/// knees come up. Backward, the back meets it, already most of a ball.
fn contact(body: Way) -> Pose {
    match body {
        Way::Back => ball().hips(0.0, -0.32, -0.04).head(34.0, 0.0, 0.0),
        _ => Pose::rest()
            .hips(0.0, -0.18, 0.06)
            .root(22.0, 0.0, 0.0)
            .spine(26.0, 0.0, 0.0)
            .chest(20.0, 0.0, 0.0)
            .head(30.0, 0.0, 0.0)
            .shoulder_l(82.0, 20.0, 0.0)
            .elbow_l(92.0)
            .shoulder_r(80.0, 18.0, 0.0)
            .elbow_r(96.0)
            .hip_l(48.0, 10.0, 0.0)
            .knee_l(76.0)
            .hip_r(30.0, 8.0, 0.0)
            .knee_r(80.0)
            .toe_l(32.0)
            .toe_r(32.0),
    }
}

/// The ball: chin on the chest, the back rounded, knees pulled up to it and the
/// hands on the shins. **Round**, because it is what the body rolls on: the
/// trunk curls down over thighs that come up to meet it, rather than both
/// pointing ahead, so it is about as deep as it is tall. The same whichever way
/// the roll goes; the turn is what tells forward from back.
fn ball() -> Pose {
    Pose::rest()
        .hips(0.0, -0.36, 0.0)
        .root(22.0, 0.0, 0.0)
        .spine(42.0, 0.0, 0.0)
        .chest(36.0, 0.0, 0.0)
        .head(42.0, 0.0, 0.0)
        .shoulder_l(84.0, 22.0, 0.0)
        .elbow_l(104.0)
        .shoulder_r(86.0, 20.0, 0.0)
        .elbow_r(106.0)
        .hip_l(118.0, 12.0, 0.0)
        .knee_l(146.0)
        .hip_r(118.0, 10.0, 0.0)
        .knee_r(146.0)
        .toe_l(30.0)
        .toe_r(30.0)
}

/// The break. Most of the way round, the legs come out of the ball and reach
/// along the body for the floor -- which, the body being over, is ahead of it
/// going forward and behind it going back -- and the arms come off the shins.
/// Everything that changes on this frame is the cover going.
fn open() -> Pose {
    Pose::rest()
        .hips(0.0, -0.32, 0.02)
        .root(26.0, 0.0, 0.0)
        .spine(30.0, 0.0, 0.0)
        .chest(18.0, 0.0, 0.0)
        .head(14.0, 0.0, 0.0)
        .shoulder_l(30.0, 40.0, 0.0)
        .elbow_l(70.0)
        .shoulder_r(34.0, 38.0, 0.0)
        .elbow_r(66.0)
        .hip_l(62.0, 12.0, 0.0)
        .knee_l(100.0)
        .hip_r(78.0, 10.0, 0.0)
        .knee_r(114.0)
        .toe_l(4.0)
        .toe_r(6.0)
}

/// Where the catch puts the two feet down: the lead under the body, the other
/// behind it, on the balls of both.
fn catch_feet(body: Way) -> ([f32; 3], [f32; 3]) {
    // The ankles a few centimetres up: on the balls of the feet.
    const UP: f32 = GROUND + 0.07;
    match body {
        Way::Back => ([L - 0.04, UP, -0.18], [R + 0.04, UP, 0.10]),
        _ => ([R + 0.04, UP, 0.20], [L - 0.04, UP, -0.04]),
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
fn settle(pose: Pose, lead: [f32; 3], body: Way) -> Pose {
    let pose = pose
        .plant_l([L - 0.03, GROUND, 0.16])
        .toe_l(0.0)
        .plant_r([R + 0.03, GROUND + 0.032, -0.15])
        .toe_floor_r();
    match body {
        Way::Back => pose.plant_l(lead).toe_l(2.0),
        _ => pose.plant_r(lead).toe_r(2.0),
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

/// The roll as everyone shares it, with the weapon hand coming up forward.
fn champion(pose: Pose, beat: Beat, _body: Way) -> Pose {
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
    pose.shoulder_l(66.0, -14.0, 0.0)
        .elbow_l(110.0)
        .wrist_l(-6.0, 0.0, 0.0)
}

fn bulwark(pose: Pose, beat: Beat, _body: Way) -> Pose {
    match beat {
        Beat::Push | Beat::Off => pose.shoulder_l(52.0, -6.0, 0.0).elbow_l(98.0),
        Beat::Dive | Beat::Contact | Beat::Ball | Beat::Open => shield(pose),
        // Up behind the shield, deeper than anyone.
        Beat::Catch => shield(pose.hips(0.0, -0.44, 0.0).root(22.0, 0.0, -6.0)),
        Beat::Rise => shield(pose.hips(0.0, -0.30, 0.0)),
        Beat::Settle => pose.shoulder_l(44.0, -4.0, 0.0).elbow_l(96.0),
    }
}

fn reaver(pose: Pose, beat: Beat, _body: Way) -> Pose {
    match beat {
        // Up light: not so deep, the blades out low either side.
        Beat::Catch => pose
            .hips(0.0, -0.30, 0.0)
            .shoulder_l(-20.0, 46.0, 0.0)
            .elbow_l(20.0)
            .shoulder_r(-20.0, 46.0, 0.0)
            .elbow_r(20.0),
        Beat::Rise => pose
            .shoulder_l(-10.0, 42.0, 0.0)
            .elbow_l(26.0)
            .shoulder_r(-10.0, 42.0, 0.0)
            .elbow_r(26.0),
        _ => pose,
    }
}

fn elementalist(pose: Pose, beat: Beat, body: Way) -> Pose {
    match beat {
        // Out of the turn sliding on: low, the trunk forward over the knees,
        // her left hand flat on the floor beside the lead foot.
        Beat::Catch | Beat::Rise => {
            let (deep, lean) = if beat == Beat::Catch {
                (-0.48, 34.0)
            } else {
                (-0.40, 28.0)
            };
            let z = if body == Way::Back { -0.10 } else { 0.24 };
            pose.hips(0.0, deep, 0.0)
                .root(lean, 0.0, -6.0)
                .spine(22.0, 0.0, 0.0)
                .reach_l([L - 0.20, 0.06, z])
                .shoulder_r(10.0, 64.0, 0.0)
                .elbow_r(30.0)
        }
        _ => pose,
    }
}

fn blood(pose: Pose, beat: Beat, _body: Way) -> Pose {
    // The scythe arm out wide and a little back, nearly straight.
    let out = |p: Pose| p.shoulder_r(-10.0, 88.0, 0.0).elbow_r(12.0);
    match beat {
        // Unfurling low.
        Beat::Catch => out(pose.hips(0.0, -0.44, 0.0)),
        Beat::Rise => out(pose.hips(0.0, -0.32, 0.0)),
        Beat::Settle => pose.shoulder_r(-8.0, 50.0, 0.0).elbow_r(30.0),
        _ => pose,
    }
}

fn dual(pose: Pose, beat: Beat, _body: Way) -> Pose {
    // Both arms opening wide.
    let wings = |p: Pose, spread: f32| {
        p.shoulder_l(-4.0, spread, 0.0)
            .elbow_l(18.0)
            .shoulder_r(-4.0, spread, 0.0)
            .elbow_r(18.0)
    };
    match beat {
        Beat::Open => wings(pose, 60.0),
        Beat::Catch => wings(pose.hips(0.0, -0.28, 0.0), 80.0),
        Beat::Rise => wings(pose.hips(0.0, -0.16, 0.0), 66.0),
        _ => pose,
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
