//! Standing, walking and running.
//!
//! This is the worked example the other clip files are written against, and it
//! is the family that matters most: a player looks at the idle for more of a
//! match than at any attack, and foot skate in a walk cycle is the single most
//! legible sign of animation done badly.
//!
//! ## How a stride avoids skating
//!
//! A planted foot must move backwards, relative to the body, at exactly the
//! rate the body moves forwards. So the foot targets are not eyeballed -- they
//! are computed from the stride length the renderer plays this clip at:
//!
//! ```text
//! ground per frame = stride length / cycle length
//! planted foot z   = contact z − (ground per frame) × frames since contact
//! ```
//!
//! `plant_l` then solves the leg for that point, and the foot is exactly where
//! the arithmetic says it is on every key. The easing between stance keys is
//! deliberately **linear** for the same reason: an ease-in on a planted foot is
//! a skate with good manners. The springs supply all the smoothing that is
//! wanted, and because a spring chasing a constant-velocity target settles at a
//! constant *offset*, they add no skate of their own.
//!
//! The stride lengths are imported from `view::play` rather than typed here, so
//! there is only one of each number in the repository and it cannot drift.

use crate::bake::{Key, Looseness, Recipe};
use crate::ease::Ease;
use view::clips::Clip;
use view::play::{RUN_STRIDE, WALK_STRIDE};
use view::pose::{ANKLE_ON_GROUND as GROUND, Pose};

/// Where the feet sit either side of the centre line.
const L: f32 = -0.115;
const R: f32 = 0.115;

pub fn clips() -> Vec<Recipe> {
    vec![
        idle(),
        walk(Clip::WalkForward, Travel::Forward),
        walk(Clip::WalkBack, Travel::Back),
        walk(Clip::WalkLeft, Travel::Left),
        walk(Clip::WalkRight, Travel::Right),
        run(Clip::RunForward, Travel::Forward),
        run(Clip::RunBack, Travel::Back),
        run(Clip::RunLeft, Travel::Left),
        run(Clip::RunRight, Travel::Right),
    ]
}

// ---------------------------------------------------------------------------
// Idle
// ---------------------------------------------------------------------------

/// The stance everything else is read against: bladed, knees soft, hands up.
///
/// Not a neutral A-pose. A fighter at rest is already in a fighting stance, and
/// the difference between "standing" and "ready" is most of what tells a player
/// the match has started.
pub fn stance() -> Pose {
    Pose::rest()
        .hips(0.0, -0.055, 0.0)
        .root(2.0, 0.0, -13.0)
        .spine(5.0, 0.0, 6.0)
        .chest(2.0, 0.0, 7.0)
        .head(-3.0, 0.0, 4.0)
        .shoulder_l(14.0, 13.0, 0.0)
        .elbow_l(52.0)
        .wrist_l(-8.0, 0.0, 0.0)
        .shoulder_r(6.0, 15.0, 0.0)
        .elbow_r(44.0)
        .wrist_r(-8.0, 0.0, 0.0)
        .plant_l([L - 0.03, GROUND, 0.16])
        // The rear heel is off the floor, which is what a bladed stance does.
        // Raising it means raising the *ankle* and pivoting on the toe -- tipping
        // the foot about an ankle that is still at floor height just drives the
        // toe through the ground, which is what the first version of this did.
        .plant_r([R + 0.03, GROUND + 0.032, -0.15])
        .toe_l(0.0)
        .toe_floor_r()
}

fn idle() -> Recipe {
    // Two things happen on different clocks: a breath every two seconds, and a
    // weight shift every four. Keying them against each other is what stops an
    // idle reading as a metronome -- the body is never in the same place twice.
    let breathe_in = stance()
        .hips(0.0, -0.043, 0.0)
        .spine(3.5, 0.0, 6.0)
        .chest(0.0, 0.0, 7.0)
        .shoulder_l(14.0, 15.5, 0.0)
        .shoulder_r(6.0, 17.5, 0.0)
        .head(-4.5, 0.0, 4.0);

    let weight_left = stance()
        .hips(-0.022, -0.062, 0.0)
        .root(2.0, -3.0, -13.0)
        .spine(5.0, 2.5, 6.0)
        .head(-3.0, 1.5, 7.0)
        .plant_l([L - 0.03, GROUND, 0.16])
        .plant_r([R + 0.03, GROUND + 0.038, -0.15])
        .toe_l(0.0)
        .toe_floor_r();

    let weight_right = stance()
        .hips(0.018, -0.058, 0.0)
        .root(2.0, 2.0, -13.0)
        .spine(5.0, -2.0, 6.0)
        .head(-2.0, -1.0, 2.0)
        .plant_r([R + 0.03, GROUND + 0.012, -0.15])
        .plant_l([L - 0.03, GROUND, 0.16])
        .plant_r([R + 0.03, GROUND, -0.15])
        .toe_l(-2.0)
        .toe_floor_r();

    Recipe {
        clip: Clip::Idle,
        looseness: Looseness::STRIDE,
        notes: "Ready stance, not a rest pose. The breath and the weight shift \
                run on different periods so the loop never reads as a metronome. \
                Both feet stay planted throughout: an idle that lifts a foot \
                looks like it is about to do something, and it is not."
            .into(),
        keys: vec![
            Key::eased(0, stance(), Ease::SMOOTH),
            Key::eased(26, breathe_in, Ease::SMOOTH),
            Key::eased(52, weight_left, Ease::SMOOTH),
            Key::eased(74, breathe_in.blend(&weight_left, 0.5), Ease::SMOOTH),
            Key::eased(96, weight_right, Ease::SMOOTH),
        ],
    }
}

// ---------------------------------------------------------------------------
// Strides
// ---------------------------------------------------------------------------

/// Which way the body is going. The stride machinery is the same in all four
/// directions; only where the feet go changes.
#[derive(Clone, Copy, PartialEq)]
pub enum Travel {
    Forward,
    Back,
    Left,
    Right,
}

impl Travel {
    /// The unit vector, in character space, that a planted foot slides along
    /// -- which is the opposite of the way the body is going.
    fn drift(self) -> (f32, f32) {
        match self {
            Travel::Forward => (0.0, -1.0),
            Travel::Back => (0.0, 1.0),
            Travel::Left => (1.0, 0.0),
            Travel::Right => (-1.0, 0.0),
        }
    }

    /// A sidestep is a shorter step than a stride forward, and has to be: a leg
    /// swung sideways runs out of hip long before one swung forward runs out of
    /// leg. The renderer shortens the stride it plays these clips at by the
    /// same factor, so the feet still do not skate.
    fn stride_scale(self) -> f32 {
        match self {
            Travel::Left | Travel::Right => view::play::STRAFE_STRIDE,
            _ => 1.0,
        }
    }

    /// +1 when travelling to the character's right, -1 to its left, 0 ahead or
    /// behind.
    fn sideways(self) -> f32 {
        match self {
            Travel::Left => -1.0,
            Travel::Right => 1.0,
            _ => 0.0,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Travel::Forward => "forward",
            Travel::Back => "backwards",
            Travel::Left => "to the left",
            Travel::Right => "to the right",
        }
    }
}

/// A walk: one foot always down, a shallow bob, a small arm swing.
///
/// Stance is a little over half the cycle, so there is a moment of double
/// support at each contact -- which is the definition of a walk, and what
/// separates it from the run below.
fn walk(clip: Clip, travel: Travel) -> Recipe {
    let cycle = clip.length() as f32;
    let half = (cycle * 0.5).round();
    let step = Step {
        travel,
        cycle,
        stride: WALK_STRIDE * travel.stride_scale(),
        stance: 0.53,
        lift: 0.16,
        contact_height: GROUND,
        tuck: if matches!(travel, Travel::Left | Travel::Right) {
            0.10
        } else {
            0.0
        },
        toe_rise: 0.085,
        roll_from: 0.55,
        run_in: 0.34,
        alternate: !matches!(travel, Travel::Left | Travel::Right),
    };
    // Key frames as whole numbers, used for both the timing and the foot
    // arithmetic. Working the two out separately is how a planted foot ends up
    // a centimetre out on every key.
    let (f_down, f_pass, f_up) = (
        (cycle * 0.12).round(),
        (cycle * 0.26).round(),
        (cycle * 0.40).round(),
    );

    let lean = match travel {
        Travel::Forward => 7.0,
        Travel::Back => -5.0,
        _ => 2.0,
    };
    // A sidestep turns its hips toward where it is going and keeps its
    // shoulders square. That is not a flourish: a leg cannot abduct far enough
    // to take a real sideways step, so a body that does not turn its pelvis
    // ends up asking the hip joint for an angle it does not have.
    let open = travel.sideways() * 20.0;
    let torso = |drop: f32, twist: f32| {
        Pose::rest()
            .hips(0.0, drop, 0.0)
            .root(0.0, 0.0, twist * 0.6 + open)
            .spine(lean, 0.0, -twist * 0.5 - open * 0.45)
            .chest(lean * 0.3, 0.0, -twist * 0.8 - open * 0.5)
            .head(-lean * 0.7, 0.0, twist * 0.3)
    };

    // Four shapes, and only the shapes: the torso, the arms and the hips. Where
    // the feet go is arithmetic, and asking a person to key it is asking them
    // to be a calculator.
    //
    // Contact -- the left heel lands and the right is rolling off its toe. The
    // hips are already dropping, because the weight arrives before the pose
    // does.
    let contact = torso(-0.070, 8.0)
        .shoulder_l(-16.0, 10.0, 0.0)
        .elbow_l(28.0)
        .shoulder_r(24.0, 9.0, 0.0)
        .elbow_r(42.0);
    // Down: the weight is taken and the hips reach their lowest.
    let down = torso(-0.095, 3.0)
        .shoulder_l(-10.0, 10.0, 0.0)
        .elbow_l(26.0)
        .shoulder_r(16.0, 9.0, 0.0)
        .elbow_r(38.0);
    // Passing: the swinging leg comes under the body, the hips rise.
    let passing = torso(-0.048, -1.0)
        .shoulder_l(-2.0, 10.0, 0.0)
        .elbow_l(22.0)
        .shoulder_r(6.0, 9.0, 0.0)
        .elbow_r(30.0);
    // Up: the standing leg straightens into its push, the swing leg reaches.
    let up = torso(-0.036, -7.0)
        .shoulder_l(10.0, 10.0, 0.0)
        .elbow_l(24.0)
        .shoulder_r(-7.0, 9.0, 0.0)
        .elbow_r(26.0);

    let shapes = [
        (0.0, contact),
        (f_down, down),
        (f_pass, passing),
        (f_up, up),
    ];

    Recipe {
        clip,
        looseness: Looseness::STRIDE,
        notes: format!(
            "A measured walk {}. Four shapes a step -- contact, down, passing, up \
             -- and the feet placed by arithmetic from the stride length at every \
             key. The easing is linear, because an ease on a planted foot is a \
             skate with good manners.",
            travel.name()
        ),
        keys: step.keys(&shapes, half, cycle),
    }
}

/// A run: a short stance, a long reach, and a flight phase where neither foot
/// is down.
///
/// Stance is only a quarter of the cycle. That is not a stylistic choice -- at
/// seven metres per second a leg this long *cannot* stay on the ground any
/// longer and still be under the body, and stretching it is what turns a run
/// into a moon walk.
fn run(clip: Clip, travel: Travel) -> Recipe {
    let cycle = clip.length() as f32;
    let half = (cycle * 0.5).round();
    let step = Step {
        travel,
        cycle,
        stride: RUN_STRIDE * travel.stride_scale(),
        stance: 0.25,
        lift: 0.32,
        // A run lands on the ball of the foot, not the heel.
        // Barely above the floor. A run lands on the ball of the foot, but the
        // foot's own angle lags the leg by two and a half frames -- so a steep
        // landing pitch is still pointing down when the leg has already brought
        // the ankle to the ground, and the toe goes through it.
        contact_height: GROUND + 0.015,
        tuck: 0.0,
        toe_rise: 0.125,
        roll_from: 0.30,
        run_in: 0.16,
        alternate: !matches!(travel, Travel::Left | Travel::Right),
    };
    let (f_absorb, f_drive, f_flight) = (
        (cycle * 0.09).round(),
        (cycle * 0.25).round(),
        (cycle * 0.38).round(),
    );

    let lean = match travel {
        Travel::Forward => 15.0,
        Travel::Back => -10.0,
        _ => 5.0,
    };
    // At a sprint the hips have to open a long way toward the direction of
    // travel -- a sideways run is a crossover run, and it looks like one.
    let open = travel.sideways() * 30.0;
    let torso = |drop: f32, twist: f32| {
        Pose::rest()
            .hips(0.0, drop, 0.02)
            .root(0.0, 0.0, twist * 0.8 + open)
            .spine(lean, 0.0, -twist * 0.6 - open * 0.4)
            .chest(lean * 0.25, 0.0, -twist - open * 0.55)
            .head(-lean * 0.8, 0.0, twist * 0.35)
    };

    // Contact: the foot lands close under the hips. Reaching out in front is
    // what a fast walk does; a run lands beneath itself.
    let contact = torso(-0.040, 12.0)
        .shoulder_l(-40.0, 11.0, 0.0)
        .elbow_l(80.0)
        .shoulder_r(48.0, 10.0, 0.0)
        .elbow_r(88.0);
    let absorb = torso(-0.088, 5.0)
        .shoulder_l(-24.0, 11.0, 0.0)
        .elbow_l(72.0)
        .shoulder_r(32.0, 10.0, 0.0)
        .elbow_r(84.0);
    // Drive: the last of the stance, up on the toe and pushing back.
    let drive = torso(-0.016, -4.0)
        .shoulder_l(16.0, 11.0, 0.0)
        .elbow_l(66.0)
        .shoulder_r(-16.0, 10.0, 0.0)
        .elbow_r(74.0);
    // Flight: both feet off the ground and the body at its highest. Without
    // this shape the clip is a fast walk, and a viewer can tell at a glance.
    let flight = torso(0.028, -10.0)
        .shoulder_l(32.0, 11.0, 0.0)
        .elbow_l(60.0)
        .shoulder_r(-30.0, 10.0, 0.0)
        .elbow_r(62.0);

    let shapes = [
        (0.0, contact),
        (f_absorb, absorb),
        (f_drive, drive),
        (f_flight, flight),
    ];

    Recipe {
        clip,
        looseness: Looseness::STRIDE,
        notes: format!(
            "A committed sprint {}. The shape that matters is flight: both feet \
             off the ground, hips at their highest. The stance shapes are close \
             together on purpose -- a quarter of the cycle is all the time a leg \
             this long has on the ground at this speed.",
            travel.name()
        ),
        keys: step.keys(&shapes, half, cycle),
    }
}

/// Everything about where the feet go in one stride, so that a walk and a run
/// differ by five numbers rather than by two copies of the same arithmetic.
#[derive(Clone, Copy)]
struct Step {
    travel: Travel,
    /// Frames in a full cycle -- two steps.
    cycle: f32,
    /// Metres of ground the cycle covers.
    stride: f32,
    /// Fraction of the cycle a foot spends on the ground.
    stance: f32,
    /// How high the ankle gets at the top of its swing.
    lift: f32,
    /// Ankle height at the moment of contact: a heel strike is on the floor, a
    /// run lands on the ball of the foot.
    contact_height: f32,
    /// How far the swinging foot passes behind the standing leg. Sidesteps need
    /// it; a forward walk does not.
    tuck: f32,
    /// How high the ankle gets by the end of stance, rolled up onto the toe.
    toe_rise: f32,
    /// How far through stance the heel starts to lift, as a fraction. A walk
    /// stays flat most of the way; a run is rolling almost immediately.
    roll_from: f32,
    /// How much of the swing is spent holding still in the world before taking
    /// weight, as a fraction. A walk reaches and waits; a sprint plants
    /// abruptly, because it has no time to do anything else.
    run_in: f32,
    /// Whether the second step of the cycle is the first one mirrored. True
    /// going forwards and backwards; false sideways, where the two steps are a
    /// lead and a trail rather than a left and a right.
    alternate: bool,
}

/// Ankle to toe, in metres on the reference body.
const FOOT: f32 = 0.22;

/// Read the shape track at a point in the half cycle, wrapping round to the
/// first shape at the end.
fn blend_shapes(shapes: &[(f32, Pose)], f: f32, half: f32, alternate: bool) -> Pose {
    let n = shapes.len();
    for i in 0..n {
        let (a_f, a) = shapes[i];
        let (b_f, b) = if i + 1 < n {
            shapes[i + 1]
        } else if alternate {
            (half, shapes[0].1.mirrored())
        } else {
            (half, shapes[0].1)
        };
        if f >= a_f && f <= b_f {
            let t = ((f - a_f) / (b_f - a_f).max(1e-3)).clamp(0.0, 1.0);
            return a.blend(&b, t);
        }
    }
    shapes[n - 1].1
}

impl Step {
    /// Where a foot is, `s` frames after its own contact.
    ///
    /// Stance is the easy half: the foot is nailed to the ground, so its
    /// position relative to the body is contact minus the ground the body has
    /// covered since. The swing is the half worth explaining. It ends with a
    /// **run-in**: for the last few frames the foot holds still *in the world*
    /// and only descends, which is what a real foot does and what stops the
    /// heel arriving sideways at the moment it takes weight. Without it, the
    /// last frame before contact drags the foot across the floor.
    fn foot(&self, s: f32, left: bool) -> [f32; 3] {
        let lift = (self.cycle * self.stance).round();
        let land = self.cycle;
        // A foot's own cycle, wrapped. The two feet are half a cycle apart, so
        // the second one runs off the end of the first one's timeline and has
        // to come back round -- and a stride that forgets to wrap puts one foot
        // permanently in the wrong half of its own step.
        let s = s.rem_euclid(land);
        if s <= lift {
            // The last fifth of stance rolls up onto the toe, which lifts the
            // ankle even though the foot has not left the ground. Without it
            // the ankle teleports upward on the frame the swing starts, and the
            // push-off reads as the foot being yanked.
            let roll =
                ((s - lift * self.roll_from) / (lift * (1.0 - self.roll_from))).clamp(0.0, 1.0);
            let rise = self.toe_rise * roll;
            // The landing height decays over the first couple of frames rather
            // than vanishing after one. A run lands on the ball of its foot and
            // then lowers onto it; cutting straight to flat drops the ankle
            // four centimetres in a frame and puts the toe through the floor.
            let land_high = (self.contact_height - GROUND) * (1.0 - s / 2.5).max(0.0);
            let height = GROUND + rise + land_high;
            let mut p = self.planted(s, left, height);
            // Rolling onto the toe is a *pivot about the toe*, so the ankle
            // swings forward over it. Place the ankle where the stride says and
            // the toe -- the part actually touching the ground -- slides
            // forward a couple of centimetres a frame, which is skate by any
            // other name.
            let sin_pitch = (rise / FOOT).clamp(0.0, 0.95);
            let shift = FOOT * (1.0 - (1.0 - sin_pitch * sin_pitch).sqrt());
            let (dx, dz) = self.travel.drift();
            p[0] -= dx * shift;
            p[2] -= dz * shift;
            return p;
        }

        let u = ((s - lift) / (land - lift)).clamp(0.0, 1.0);
        // The arc peaks early rather than halfway. A foot clears the ground
        // hard in the first few frames after toe-off and then reaches; a
        // symmetric arc leaves the toe scraping along exactly where the body is
        // moving fastest over it.
        let arc = (u.powf(0.7) * std::f32::consts::PI).sin();
        // The swing begins where the stance ended -- already up on the toe --
        // rather than back down at floor level. Forgetting that leaves a
        // seven-centimetre drop on the frame the foot leaves the ground, and
        // the foot spends the first few frames of its swing scraping along.
        let height = GROUND
            + self.toe_rise * (1.0 - u).powi(2)
            + self.lift * arc
            + (self.contact_height - GROUND) * u * u;

        // Long enough that the foot is genuinely still before it takes weight.
        // Short and the last frames of the reach are a skid into the contact.
        let run_in = (land - lift) * self.run_in;
        let (x, z) = if s >= land - run_in {
            // Holding still in the world: the foot's position relative to the
            // body keeps sliding back at exactly the body's own speed.
            let p = self.planted(s - land, left, 0.0);
            (p[0], p[2])
        } else {
            let a = self.planted(lift, left, 0.0);
            let b = self.planted(-run_in, left, 0.0);
            let t = ((s - lift) / (land - run_in - lift)).clamp(0.0, 1.0);
            let (dx, dz) = self.travel.drift();
            // Pass behind the standing leg rather than around it, which keeps a
            // sidestep from reading as a stumble.
            (
                a[0] + (b[0] - a[0]) * t - dz * self.tuck * arc,
                a[2] + (b[2] - a[2]) * t + dx * self.tuck * arc,
            )
        };
        [x, height, z]
    }

    fn planted(&self, s: f32, left: bool, height: f32) -> [f32; 3] {
        let (dx, dz) = self.travel.drift();
        let base = if left { L } else { R };
        // Half the ground the foot covers while planted, so the stance is
        // centred under the body rather than all in front of it.
        let lead = self.stride * self.stance * 0.5;
        let along = self.stride * s / self.cycle - lead;
        [base + dx * along, height, dz * along]
    }

    /// Put both feet where they belong at absolute frame `f`, given that the
    /// right foot's own contact happens `half` frames into the cycle.
    fn place(&self, pose: Pose, f: f32, half: f32) -> Pose {
        let (sl, sr) = (f, f + self.cycle - half);
        let pose = pose
            .plant_l(self.foot(sl, true))
            .plant_r(self.foot(sr, false));
        self.ankle(self.ankle(pose, sl, true), sr, false)
    }

    /// What the ankle should be doing, given where that foot is in its own
    /// step. Derived rather than keyed: the angle that keeps a toe on the floor
    /// depends on how far the ankle has risen, which is not a number anybody
    /// can hold in their head.
    fn ankle(&self, pose: Pose, s: f32, left: bool) -> Pose {
        let land = self.cycle;
        let s = s.rem_euclid(land);
        let lift = (land * self.stance).round();
        let roll = ((s - lift * self.roll_from) / (lift * (1.0 - self.roll_from))).clamp(0.0, 1.0);
        if s > lift {
            // Swinging: toes up, so the foot clears rather than drags.
            let u = (s - lift) / (land - lift);
            let dorsi = if u > 0.8 { -4.0 } else { -12.0 };
            return if left {
                pose.toe_l(dorsi)
            } else {
                pose.toe_r(dorsi)
            };
        }
        if s < 2.0 {
            // The landing: a walk strikes with the heel, a run with the ball of
            // the foot.
            let heel = self.contact_height <= GROUND + 0.001;
            return match (left, heel) {
                (true, true) => pose.toe_l(-12.0),
                (true, false) => pose.toe_floor_l(),
                (false, true) => pose.toe_r(-12.0),
                (false, false) => pose.toe_floor_r(),
            };
        }
        if roll > 0.08 {
            // The heel has left the floor; the toe has not.
            if left {
                pose.toe_floor_l()
            } else {
                pose.toe_floor_r()
            }
        } else if left {
            pose.toe_l(0.0)
        } else {
            pose.toe_r(0.0)
        }
    }

    /// Turn a handful of upper-body shapes into a full set of keys.
    ///
    /// Keys every few frames rather than one per shape, because a pose is
    /// stored as joint *angles* and interpolating angles is not the same as
    /// interpolating the foot position they were solved from. Over a five-frame
    /// gap that difference is four centimetres of drift under a planted foot;
    /// over a two-frame gap it is under one.
    fn keys(&self, shapes: &[(f32, Pose)], half: f32, cycle: f32) -> Vec<Key> {
        let spacing = (half / 6.0).round().max(2.0);
        let mut keys = Vec::new();
        let mut f = 0.0;
        while f < cycle - 0.5 {
            // Which half of the cycle this is, and how far into it.
            let second = f >= half;
            let upper = blend_shapes(
                shapes,
                if second { f - half } else { f },
                half,
                self.alternate,
            );
            // The second half of the cycle is the same shapes with the arms
            // swapped -- except sideways, where it is not. A sidestep's two
            // steps are a lead and a trail rather than a left and a right, and
            // mirroring one into the other would flip the direction of travel
            // along with everything else.
            let upper = if second && self.alternate {
                upper.mirrored()
            } else {
                upper
            };
            keys.push(Key::eased(
                f as u16,
                self.place(upper, f, half),
                Ease::LINEAR,
            ));
            f += spacing;
        }
        keys
    }
}
