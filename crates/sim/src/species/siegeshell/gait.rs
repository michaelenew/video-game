//! How the Siegeshell walks, and where every foot is.
//!
//! **The walk is a gait with a heading** (§5): six legs in two tripods, each
//! tripod landing together, the two alternating -- a beat every half cycle.
//! The cycle is a distance, not a time: the stride accumulator in the snapshot
//! (`Monster::stride`) advances with ground covered, so at the walking pace a
//! beat comes every three seconds, hurried every two and a half, limping
//! later, and a body that has halted has no beat at all. A footfall is the
//! accumulator crossing a tripod's landing phase ([`landed`]); nothing chooses
//! it.
//!
//! **Every foot is placed from the stride**, never from a clip ([`repose`]):
//! planted where it landed and held there while the body walks over it, swung
//! through the air to where it will land, and the leg solved to meet it -- the
//! hip turned to the foot, the thigh and the shin a two-link solve in that
//! plane, knee out and up like a crab's, the ankle levelled so the pad is flat.
//! A foot is collision geometry and the place a ring comes from, so where it
//! is has to be the simulation's, and it is a pure function of the snapshot.
//!
//! **What the legs do besides walk**, all here as well: a stamping foot lifted
//! and driven down onto its aim; a dragged foot swept inward; a broken leg
//! hanging; a stumbling side's broken legs **splayed** out flat into the stair
//! (the knee on the floor, the thigh a ramp up to the rim); a kneel splaying
//! all six; and the side that has lost ankles sitting lower for good.

use super::fight::{self, leg};
use super::{BUILD, HIP_X, Knob, LEG_COUNT, ankle_part, bones, leg_pair, leg_side, tripod};
use crate::DT;
use crate::beast::{Placed, Pose};
use crate::fixed::{Fx, cos_turns, sin_turns};
use crate::math::{self, Mat3, V3};
use crate::monster::{Doing, Monster};

/// Centimetres as fixed point.
const fn cm(v: i32) -> Fx {
    Fx::ratio(v, 100)
}

/// Half of one: where the second tripod lands.
fn half_cycle() -> Fx {
    math::half(Fx::ONE)
}

// ---------------------------------------------------------------------------
// The cycle
// ---------------------------------------------------------------------------

/// Where it is in its gait cycle, nought to one: the stride accumulator.
pub fn cycle(m: &Monster) -> Fx {
    Fx::from_raw(m.stride as i32)
}

/// Where in the cycle tripod `t` lands: the first at nought, the second half
/// a cycle later.
pub fn lands_at(t: usize) -> Fx {
    if t == 0 { Fx::ZERO } else { half_cycle() }
}

/// How far through its own cycle leg `leg` is, nought to one, from the frame
/// its foot last landed.
pub fn since_landing(m: &Monster, leg: usize) -> Fx {
    math::wrap_unit(cycle(m).sub(lands_at(tripod(leg))))
}

/// The share of a cycle a foot spends in the air, before it lands.
fn lift_share() -> Fx {
    Knob::LiftShare.fx().clamp(crate::arena::SKIN, half_cycle())
}

/// Is leg `leg`'s foot in the air in its gait, and how far through the swing:
/// `None` while it is planted.
pub fn swinging(m: &Monster, leg: usize) -> Option<Fx> {
    let u = since_landing(m, leg);
    let lift = lift_share();
    let from = Fx::ONE.sub(lift);
    if u.raw() >= from.raw() {
        Some(u.sub(from).div(lift))
    } else {
        None
    }
}

/// **The tripod that landed between two strides**, if one did: the beat. The
/// accumulator wraps, so a crossing of nought is the first tripod's.
pub fn landed(before: u16, after: u16) -> Option<usize> {
    if before == after {
        return None;
    }
    let crossed = |at: u16| -> bool {
        // Did walking forward from `before` to `after` pass `at`? Strides are
        // short against a whole cycle, so the forward distance is the wrapped
        // difference.
        let gone = after.wrapping_sub(before);
        let to = at.wrapping_sub(before).wrapping_sub(1);
        to < gone
    };
    if crossed(0) {
        Some(0)
    } else if crossed(1 << 15) {
        Some(1)
    } else {
        None
    }
}

/// Frames until the next tripod lands at the pace it is walking, or `None`
/// halted.
pub fn frames_to_beat(m: &Monster) -> Option<u16> {
    let speed = m.speed;
    if speed.raw() <= 0 {
        return None;
    }
    let per_frame = speed.mul(DT).div(m.sp().gait_stride());
    if per_frame.raw() <= 0 {
        return None;
    }
    let c = cycle(m);
    let half = half_cycle();
    let to = if c.raw() < half.raw() {
        half.sub(c)
    } else {
        Fx::ONE.sub(c)
    };
    Some(to.div(per_frame).to_int().clamp(0, u16::MAX as i32) as u16)
}

/// Frames between beats at the pace it is walking: a large number halted.
pub fn frames_per_beat(m: &Monster) -> i32 {
    let speed = m.speed;
    if speed.raw() <= 0 {
        return i32::MAX / 2;
    }
    math::half(m.sp().gait_stride())
        .div(speed.mul(DT))
        .to_int()
        .max(1)
}

// ---------------------------------------------------------------------------
// Where the feet are
// ---------------------------------------------------------------------------

/// A foot's home in body space, on the floor: under its hip's line, out at the
/// planted width.
pub fn home(leg: usize) -> V3 {
    V3::new(
        cm(HIP_X[leg_pair(leg)]),
        Fx::ZERO,
        cm(BUILD.foot_out * leg_side(leg)),
    )
}

/// **Where leg `leg`'s foot is in the gait**, in body space: the middle of the
/// pad's underside, and how high it is lifted. Planted, it slides back under
/// the body exactly as fast as the body walks over it, so in the world it
/// does not move; in the air it swings forward to where it will land.
pub fn gait_foot(m: &Monster, leg: usize) -> V3 {
    let stride = m.sp().gait_stride();
    let lift = lift_share();
    // The body covers this much while a foot is down; the foot travels it
    // backward under the body, and forward again in the air.
    let reach = stride.mul(Fx::ONE.sub(lift));
    let front = math::half(reach);
    let base = home(leg);
    match swinging(m, leg) {
        Some(t) => {
            let x = front.neg().add(reach.mul(t));
            let up = Knob::Lift.fx().mul(math::hump(t));
            V3::new(base.x.add(x), up, base.z)
        }
        None => {
            let u = since_landing(m, leg).div(Fx::ONE.sub(lift));
            V3::new(base.x.add(front.sub(reach.mul(u))), Fx::ZERO, base.z)
        }
    }
}

/// Where leg `leg`'s foot will land next, in the world: the pad circle drawn
/// from the lift. Its place in the gait at the end of the swing.
pub fn landing_spot(m: &Monster, leg: usize) -> V3 {
    let stride = m.sp().gait_stride();
    let lift = lift_share();
    let reach = stride.mul(Fx::ONE.sub(lift));
    let base = home(leg);
    // Lands at the front of its stance, after the body has walked on by
    // what is left of the swing.
    let left = match swinging(m, leg) {
        Some(t) => stride.mul(lift).mul(Fx::ONE.sub(t)),
        None => Fx::ZERO,
    };
    let at = V3::new(base.x.add(math::half(reach)).add(left), Fx::ZERO, base.z);
    flat_world(m, at)
}

/// A body-space point put into the world on the flat: the creature's own
/// frame, its pose left out.
pub fn flat_world(m: &Monster, local: V3) -> V3 {
    m.pos.add(Mat3::from_yaw(m.yaw).apply(local))
}

/// A world point read in the creature's flat body frame.
pub fn flat_body(m: &Monster, world: V3) -> V3 {
    Mat3::from_yaw(m.yaw).unapply(world.sub(m.pos))
}

// ---------------------------------------------------------------------------
// The body over the legs
// ---------------------------------------------------------------------------

/// How far through a countdown, nought to one.
fn through(left: u16, total: u16) -> Fx {
    let total = total.max(1);
    Fx::from_int(total.saturating_sub(left.min(total)) as i32).div(Fx::from_int(total as i32))
}

/// How far a lowering of `total` frames is in, nought to one: in over
/// `settle` frames, held, and out over the last `settle`.
fn lowered(left: u16, total: u16, settle: u16) -> Fx {
    let settle = settle.max(1).min(total.max(2) / 2);
    let gone = total.saturating_sub(left);
    let k = if gone < settle {
        through(settle - gone, settle)
    } else if left < settle {
        through(settle - left, settle).neg().add(Fx::ONE)
    } else {
        Fx::ONE
    };
    math::smoothstep(k)
}

/// Broken ankles on each side: left, right.
pub fn broken_sides(m: &Monster) -> (i32, i32) {
    let mut l = 0;
    let mut r = 0;
    for leg in 0..LEG_COUNT {
        if m.broken(ankle_part(leg)) {
            if leg_side(leg) < 0 {
                l += 1;
            } else {
                r += 1;
            }
        }
    }
    (l, r)
}

/// The side a stumble is putting down, `-1` left or `+1` right, and how far
/// down it is, nought to one. `(0, 0)` standing.
pub fn stumbling(m: &Monster) -> (i32, Fx) {
    let side = fight::stumble_side(m);
    match m.doing {
        Doing::Stumble { left, .. } if side != 0 => (
            side,
            lowered(
                left,
                m.sp().stumble_frames(),
                Knob::Settle.raw().clamp(1, 600) as u16,
            ),
        ),
        _ => (0, Fx::ZERO),
    }
}

/// How far down a kneel is, nought to one.
pub fn kneeling(m: &Monster) -> Fx {
    match m.doing {
        Doing::Toppled { left } => lowered(
            left,
            m.sp().topple_frames(),
            Knob::KneelSettle.raw().clamp(1, 600) as u16,
        ),
        Doing::Dead => Fx::ONE,
        _ => Fx::ZERO,
    }
}

/// Is leg `leg` splayed out flat this frame -- a stair -- and how far into it?
pub fn splayed(m: &Monster, leg: usize) -> Fx {
    let kneel = kneeling(m);
    let (side, down) = stumbling(m);
    let stair = if side == leg_side(leg) && m.broken(ankle_part(leg)) {
        down
    } else {
        Fx::ZERO
    };
    kneel.max(stair)
}

/// **The body's lowering**, on top of the clip: the side that has lost ankles
/// sits lower for good, a stumble drops and rolls the shell toward its side,
/// and a kneel drops all of it. A drop, and a roll in turns (positive lowers
/// its right, the rig's own sense).
fn lowering(m: &Monster) -> (Fx, Fx) {
    let (l, r) = broken_sides(m);
    let sag = Knob::Sag.fx();
    let (a, b) = (sag.mul(Fx::from_int(l)), sag.mul(Fx::from_int(r)));
    // Each side's rim lower by its own share: the middle drops by the mean,
    // and the roll takes the difference across the rim's width.
    let rim = cm(BUILD.hip_out).add(cm(100));
    let mut drop = math::half(a.add(b));
    let mut roll = b
        .sub(a)
        .div(rim.add(rim))
        .div(math::turns_to_radians(Fx::ONE));
    let (side, down) = stumbling(m);
    if side != 0 {
        drop = drop.add(Knob::StumbleDrop.fx().mul(down));
        roll = roll.add(Knob::StumbleRoll.fx().mul(down).mul(Fx::from_int(side)));
    }
    let kneel = kneeling(m);
    drop = drop.add(Knob::KneelDrop.fx().mul(kneel));
    (drop, roll)
}

// ---------------------------------------------------------------------------
// The legs
// ---------------------------------------------------------------------------

/// The root, placed, as the rig will place it with this pose -- without
/// building the rest of the rig.
fn root_of(m: &Monster, pose: &Pose) -> Placed {
    let sp = m.sp();
    let facing = Mat3::from_yaw(m.yaw);
    let a = pose.bone[bones::ROOT];
    let local = Mat3::from_angles(a.x, a.y, a.z);
    let offset = sp.rest(bones::ROOT).add(pose.hips.scale(sp.scale()));
    Placed {
        at: m.pos.add(facing.apply(offset)),
        rot: facing.then(local),
    }
}

/// `acos`, in turns, of a cosine.
fn acos_turns(c: Fx) -> Fx {
    let c = c.clamp(Fx::ONE.neg(), Fx::ONE);
    let s = Fx::ONE.sub(c.mul(c)).max(Fx::ZERO).sqrt();
    math::atan2_turns(s, c)
}

/// The two lengths, at the species' scale.
fn lengths(m: &Monster) -> (Fx, Fx) {
    let k = m.sp().scale();
    (cm(BUILD.thigh).mul(k), cm(BUILD.shin).mul(k))
}

/// **Put leg `leg`'s ankle at `at`**, in the world -- the top of the ankle
/// box, three and a half metres over the pad's underside -- on the body as
/// `root` places it, or as near as the leg reaches. Knee out and up.
fn plant(m: &Monster, mut p: Pose, root: &Placed, leg: usize, at: V3) -> Pose {
    let sp = m.sp();
    let hip_at = root.local_to_world(sp.rest(bones::hip(leg)));
    let t = root.rot.unapply(at.sub(hip_at));
    let yaw = math::atan2_turns(t.z, t.x);
    let h = V3::new(t.x, Fx::ZERO, t.z).flat_len();
    let dy = t.y;
    let (l1, l2) = lengths(m);
    let lo = l1.sub(l2).abs().add(crate::arena::SKIN);
    let hi = l1.add(l2).sub(crate::arena::SKIN);
    let d = V3::new(h, dy, Fx::ZERO).len().clamp(lo, hi);
    let phi = math::atan2_turns(dy, h);
    let c = l1
        .mul(l1)
        .add(d.mul(d))
        .sub(l2.mul(l2))
        .div(l1.mul(d).add(l1.mul(d)));
    let thigh = phi.add(acos_turns(c));
    let (kx, ky) = (l1.mul(cos_turns(thigh)), l1.mul(sin_turns(thigh)));
    let shin = math::atan2_turns(dy.sub(ky), h.sub(kx));
    p.bone[bones::hip(leg)] = V3::new(Fx::ZERO, yaw, Fx::ZERO);
    p.bone[bones::thigh(leg)] = V3::new(thigh, Fx::ZERO, Fx::ZERO);
    p.bone[bones::shin(leg)] = V3::new(shin.sub(thigh), Fx::ZERO, Fx::ZERO);
    // Level in the root's frame: the pad flat under a body standing level.
    p.bone[bones::ankle(leg)] = V3::new(shin.neg(), Fx::ZERO, Fx::ZERO);
    p
}

/// **A leg splayed out flat**: the thigh straight out sideways from the hip
/// and down to a knee whose top stands at `StairKnee` -- a ramp from the floor
/// to the shell's side -- and the shin and the foot lying out beyond it.
fn splay(m: &Monster, mut p: Pose, root: &Placed, leg: usize) -> Pose {
    let sp = m.sp();
    let (l1, l2) = lengths(m);
    let hip_at = root.local_to_world(sp.rest(bones::hip(leg)));
    let out = Mat3::from_yaw(m.yaw).apply(V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(leg_side(leg))));
    // The knee's middle a thigh's half-thickness under the stair's top.
    let knee_y = Knob::StairKnee.fx().sub(cm(90));
    let fall = hip_at.y.sub(knee_y).clamp(Fx::ZERO, l1);
    let across = l1.mul(l1).sub(fall.mul(fall)).max(Fx::ZERO).sqrt();
    let knee = V3::new(hip_at.x, knee_y, hip_at.z).add(out.scale(across));
    let t = root.rot.unapply(knee.sub(hip_at));
    let yaw = math::atan2_turns(t.z, t.x);
    let thigh = math::atan2_turns(t.y, V3::new(t.x, Fx::ZERO, t.z).flat_len());
    // The shin folded back in under the thigh, along the floor, its far end
    // a shin's half-thickness up: the knee is the outermost thing, and the
    // first tread is what a fighter walking in meets first.
    let low = cm(80);
    let sink = knee_y.sub(low).clamp(Fx::ZERO, l2);
    let lie = l2.mul(l2).sub(sink.mul(sink)).max(Fx::ZERO).sqrt();
    let foot = V3::new(knee.x, low, knee.z).sub(out.scale(lie));
    let u = root.rot.unapply(foot.sub(knee));
    let along = u.x.mul(cos_turns(yaw)).add(u.z.mul(sin_turns(yaw)));
    let shin = math::atan2_turns(u.y, along);
    p.bone[bones::hip(leg)] = V3::new(Fx::ZERO, yaw, Fx::ZERO);
    p.bone[bones::thigh(leg)] = V3::new(thigh, Fx::ZERO, Fx::ZERO);
    p.bone[bones::shin(leg)] = V3::new(shin.sub(thigh), Fx::ZERO, Fx::ZERO);
    // The foot level, its pad flat under the folded shin.
    p.bone[bones::ankle(leg)] = V3::new(shin.neg(), Fx::ZERO, Fx::ZERO);
    p
}

/// Where the top of a foot's ankle box goes, for a pad whose underside is at
/// `foot`.
fn ankle_over(m: &Monster, foot: V3) -> V3 {
    let k = m.sp().scale();
    foot.add(V3::new(
        Fx::ZERO,
        cm(BUILD.ankle_drop + BUILD.pad_thick).mul(k),
        Fx::ZERO,
    ))
}

/// Where leg `leg`'s pad would be in the gait alone, in the world.
pub fn foot_in_gait(m: &Monster, leg: usize) -> V3 {
    flat_world(m, gait_foot(m, leg))
}

/// **Where leg `leg`'s pad is this frame**, its underside's middle in the
/// world: the gait's, unless the legs' channel has it (a stamp, a drag), or
/// it is broken and hangs.
pub fn foot(m: &Monster, leg: usize) -> V3 {
    let walked = flat_world(m, gait_foot(m, leg));
    match leg::moving(m, leg) {
        Some(at) => at,
        None => walked,
    }
}

/// **Its say on the pose** (`FightDecl::repose`): the body lowered over its
/// legs, then every leg put where its foot is -- or splayed into a stair.
pub fn repose(m: &Monster, pose: Pose) -> Pose {
    let mut p = pose;
    let (drop, roll) = lowering(m);
    p.hips.y = p.hips.y.sub(drop);
    p.bone[bones::ROOT].z = p.bone[bones::ROOT].z.add(roll);
    let root = root_of(m, &p);
    for leg in 0..LEG_COUNT {
        let flat = splayed(m, leg);
        if flat.raw() >= Fx::ONE.raw() {
            p = splay(m, p, &root, leg);
            continue;
        }
        let mut at = foot(m, leg);
        // A broken leg drags: its foot never quite lifts, and it trails.
        if m.broken(ankle_part(leg)) {
            at.y = at.y.min(cm(40));
        }
        let planted = plant(m, p, &root, leg, ankle_over(m, at));
        p = if flat.raw() > 0 {
            planted.blend(&splay(m, planted, &root, leg), flat)
        } else {
            planted
        };
    }
    p
}

/// **The walk this frame**: how fast it goes, from its pace and everything
/// that holds it. What the frame hook moves it by.
pub fn pace(m: &Monster, phase: usize) -> Fx {
    if !m.alive() || fight::at_siege_line(m) {
        return Fx::ZERO;
    }
    match m.doing {
        Doing::Stumble { .. } | Doing::Toppled { .. } | Doing::Dead => return Fx::ZERO,
        _ => {}
    }
    let mut speed = m.sp().walk();
    let (l, r) = broken_sides(m);
    for _ in 0..(l + r) {
        speed = speed.mul(Knob::Limp.fx());
    }
    if phase >= 2 {
        speed = speed.mul(Knob::Hurry.fx());
    }
    if m.slowed > 0 {
        speed = speed.mul(m.slow_mul);
    }
    speed
}
