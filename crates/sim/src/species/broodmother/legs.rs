//! Her legs, placed: what the baked clips cannot know.
//!
//! A clip is baked once, for every leg at once. Two things about her legs are
//! decided in the fight instead, and both are here, as her species' say on
//! the pose (`FightDecl::repose`, [`repose`]):
//!
//! - **Which leg stabs, and where.** The stab is one move; the leg is chosen
//!   as it commits (`mind::commit`), and its foot is lifted past the knee
//!   through the windup and driven down onto the disc the hit lands on (the
//!   brain's aim point) -- the same point the telegraph draws and the hit test
//!   uses, so the foot you watch come down is the disc you step off.
//! - **The list.** Two middle legs broken on one side and that side sits low
//!   for the rest of the fight: the body dropped and rolled toward it, with
//!   every sound foot kept where it stood.
//!
//! Both put a foot somewhere by the same two-link solve the animation factory
//! uses (`anim::beast::broodmother::BroodPose::plant`), here in fixed point:
//! the coxa turned to the foot, the femur rising, knee up, the tibia down onto
//! it, in the leg's own vertical plane in the thorax's frame.

use super::fight::body;
use super::{FLURRY, Knob, LEG_COUNT, STAB, bones, middle_leg, shin_part};
use crate::beast::{Pose, Rig};
use crate::fixed::{Fx, cos_turns, sin_turns};
use crate::math::{self, V3};
use crate::monster::{Doing, Monster};

/// Where leg `leg`'s foot is, in the world, on a built rig.
pub fn foot(m: &Monster, rig: &Rig, leg: usize) -> V3 {
    let tip = m.sp().shape(shin_part(leg)).max.x;
    rig.bone[bones::tibia(leg)].local_to_world(V3::new(tip, Fx::ZERO, Fx::ZERO))
}

/// Where leg `leg`'s foot stands when she is not stabbing with it: the foot
/// of her stock pose, its stab's layer left out.
pub fn home(m: &Monster, leg: usize) -> V3 {
    let mut still = *m;
    still.own[body::STAB] &= !0xFF;
    let rig = still.rig();
    foot(&still, &rig, leg)
}

/// `acos`, in turns, from the two arguments of `atan2`.
fn acos_turns(c: Fx) -> Fx {
    let c = c.clamp(Fx::ONE.neg(), Fx::ONE);
    let s = Fx::ONE.sub(c.mul(c)).max(Fx::ZERO).sqrt();
    math::atan2_turns(s, c)
}

/// **Put leg `leg`'s foot at `at`**, in the world, on the creature as `pose`
/// has her -- or as near as the leg reaches.
pub fn plant(m: &Monster, pose: Pose, leg: usize, at: V3) -> Pose {
    let sp = m.sp();
    let rig = Rig::build(sp, m.pos, m.yaw, &pose);
    let local = rig.bone[bones::ROOT].world_to_local(at);
    let hip = sp.rest(bones::coxa(leg));
    let (dx, dy, dz) = (local.x.sub(hip.x), local.y.sub(hip.y), local.z.sub(hip.z));
    let yaw = math::atan2_turns(dz, dx);
    let l1 = super::femur_length().mul(sp.scale());
    let l2 = sp.shape(shin_part(leg)).max.x;
    let out = sp.rest(bones::femur(leg)).x;
    let dh = V3::new(dx, Fx::ZERO, dz).flat_len().sub(out);
    // Never quite straight nor quite folded: a skin short of either.
    let lo = l1.sub(l2).abs().add(crate::arena::SKIN);
    let hi = l1.add(l2).sub(crate::arena::SKIN);
    let d = V3::new(dh, dy, Fx::ZERO).len().clamp(lo, hi);
    let phi = math::atan2_turns(dy, dh);
    let c = l1
        .mul(l1)
        .add(d.mul(d))
        .sub(l2.mul(l2))
        .div(l1.mul(d).add(l1.mul(d)));
    // Knee up: the femur rises above the line to the foot.
    let thigh = phi.add(acos_turns(c));
    let (kx, ky) = (l1.mul(cos_turns(thigh)), l1.mul(sin_turns(thigh)));
    let shin = math::atan2_turns(dy.sub(ky), dh.sub(kx));
    let mut p = pose;
    p.bone[bones::coxa(leg)] = V3::new(Fx::ZERO, yaw, Fx::ZERO);
    p.bone[bones::femur(leg)] = V3::new(thigh, Fx::ZERO, Fx::ZERO);
    p.bone[bones::tibia(leg)] = V3::new(shin.sub(thigh), Fx::ZERO, Fx::ZERO);
    p
}

/// **The side she lists to**, if two of its middle legs are broken: `-1` her
/// left, `+1` her right. Both sides broken, the worse wins, and a tie lists
/// nowhere -- she is down on both.
pub fn list(m: &Monster) -> i32 {
    let at = Knob::ListLegs.raw().max(1);
    let broken = |side: i32| {
        (0..LEG_COUNT)
            .filter(|leg| middle_leg(*leg) && super::leg_side(*leg) == side)
            .filter(|leg| m.broken(shin_part(*leg)))
            .count() as i32
    };
    let (l, r) = (broken(-1), broken(1));
    match (l >= at, r >= at) {
        (true, false) => -1,
        (false, true) => 1,
        (true, true) if l > r => -1,
        (true, true) if r > l => 1,
        _ => 0,
    }
}

/// How far through a countdown, nought to one.
fn through(left: u16, total: u16) -> Fx {
    let total = total.max(1);
    Fx::from_int(total.saturating_sub(left.min(total)) as i32).div(Fx::from_int(total as i32))
}

/// **Her say on the pose** (`FightDecl::repose`): the list, then the stab.
pub fn repose(m: &Monster, pose: Pose) -> Pose {
    let mut p = pose;
    let side = list(m);
    if side != 0 {
        let rig = Rig::build(m.sp(), m.pos, m.yaw, &p);
        let feet: [V3; LEG_COUNT] = std::array::from_fn(|leg| foot(m, &rig, leg));
        p.hips.y = p.hips.y.sub(Knob::ListDrop.fx());
        p.bone[bones::ROOT].z = p.bone[bones::ROOT]
            .z
            .add(Knob::ListRoll.fx().mul(Fx::from_int(side)));
        for (leg, at) in feet.iter().enumerate() {
            if !m.broken(shin_part(leg)) {
                p = plant(m, p, leg, *at);
            }
        }
    }
    // A broken leg is held up off the floor, tucked in under her: it bears
    // nothing, and a foot left where it stood goes through the floor as the
    // body comes down on that side.
    for leg in 0..LEG_COUNT {
        if m.broken(shin_part(leg)) {
            let rig = Rig::build(m.sp(), m.pos, m.yaw, &p);
            let hip = rig.bone[bones::femur(leg)].at;
            let tuck = V3::new(hip.x, Fx::ZERO, hip.z).add(
                math::wide_normalized(V3::new(hip.x.sub(m.pos.x), Fx::ZERO, hip.z.sub(m.pos.z)))
                    .scale(super::femur_length()),
            );
            p = plant(
                m,
                p,
                leg,
                V3::new(tuck.x, hip.y.mul(Fx::ratio(1, 2)), tuck.z),
            );
        }
    }
    stab(m, p)
}

/// The stabbing leg: up past the knee through the windup, toward the disc;
/// down onto it on the hit; home again through the recovery.
fn stab(m: &Monster, p: Pose) -> Pose {
    let (kind, phase, u) = match m.doing {
        Doing::Startup { kind, left } if kind == STAB || kind == FLURRY => {
            (kind, 0, through(left, m.sp().attack(kind).startup))
        }
        Doing::Active { kind, left } if kind == STAB || kind == FLURRY => {
            (kind, 1, through(left, m.sp().attack(kind).active))
        }
        Doing::Recovery { kind, left } if kind == STAB || kind == FLURRY => {
            (kind, 2, through(left, m.sp().attack(kind).recovery))
        }
        _ => return p,
    };
    let _ = kind;
    let leg = (m.own[body::STAB] & 0xFF) as usize;
    if leg == 0 || leg > LEG_COUNT || m.broken(shin_part(leg - 1)) {
        return p;
    }
    let leg = leg - 1;
    let rig = Rig::build(m.sp(), m.pos, m.yaw, &p);
    let home = foot(m, &rig, leg);
    let disc = m.aimed_at();
    let lift = Knob::StabLift.fx();
    let ease = math::smoothstep;
    // The windup in two: up past the knee, drawing over toward the disc, for
    // the first three quarters; then driven down, to arrive on the hit's
    // first frame. On the disc through the hit; home through the recovery.
    let half = Fx::ratio(1, 2);
    let rise = Fx::ONE.sub(half.mul(half));
    let at = match phase {
        0 if u.raw() < rise.raw() => {
            let k = ease(u.div(rise));
            let over = math::lerp3(home, disc, k);
            V3::new(over.x, lift.mul(k), over.z)
        }
        0 => {
            let k = u.sub(rise).div(Fx::ONE.sub(rise));
            V3::new(disc.x, lift.mul(Fx::ONE.sub(k.mul(k))), disc.z)
        }
        1 => disc,
        _ => {
            let k = ease(u);
            let back = math::lerp3(disc, home, k);
            let arc = sin_turns(k.mul(half)).mul(lift).mul(half).mul(half);
            V3::new(back.x, arc, back.z)
        }
    };
    plant(m, p, leg, at)
}
