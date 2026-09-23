//! The Blood mage's scythe, drawn at the reach it hits at.
//!
//! Her reach grows with the grey on her bar, and `docs/design/aiming.md`
//! refuses a reach that changes with a bar for a good reason: a reach only one
//! player can see is unlearnable. The one condition under which it is allowed
//! is the rule `CLAUDE.md` puts on the debug overlay -- **the blade is drawn at
//! the length it hits at** -- so that what both players read is a weapon, not
//! a number. This module is that rule. It is a pure function of simulation
//! state so `tests/kinematics.rs` can hold it to the hit test without a
//! renderer, the way `game::effect_piece` is held to the effects.
//!
//! A war scythe is a long haft with a curved blade set at the head. It is
//! drawn as a **butt**, a **grip**, a **neck**, a **mid** and a **tip**: the
//! haft runs from the butt through the grip to the neck, and the blade curves
//! from the neck through the mid to the tip. **The tip is the thing the hit
//! test reaches with** -- while she is swinging it sits exactly at the far end
//! of the hit volume, and the haft runs back from it through her leading hand
//! and on past her trailing one, so the weapon rides the arms the clips put on
//! it. Through the wind-up and the recovery, when nothing is out, the haft
//! simply lies along the line of her two hands, the tip the live reach from
//! the leading one. At rest the weapon **stands**: the haft upright with its
//! butt on the floor under the hand that holds it, its head level with hers,
//! and the blade leaving the head curving forward and a little over it -- the
//! way a scythe is stood when it is not being swung. It grows with grey by the
//! same factor the hit test lengthens the sweep with, so the weapon a player
//! watches grow while she is standing still is the one that is about to be
//! swung.
//!
//! The blade's *flat* lies in the plane that contains the haft and the bow --
//! the plane of the cut while she swings, the plane of her facing at rest --
//! and the renderer builds each piece's rotation from that plane rather than
//! from the axis alone, which is what stops the blade rolling to some world
//! direction of its own.

use crate::math::{self, V3};
use sim::state::{self, Player};

/// The scythe, as five points in the arena and the plane its blade lies in.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Scythe {
    /// The end of the pole, behind the hands.
    pub butt: V3,
    /// Where the leading hand holds the haft. The reach is measured from here.
    pub grip: V3,
    /// Where the blade is set on the haft.
    pub neck: V3,
    /// Half way along the blade, bowed out: the curve.
    pub mid: V3,
    /// The point of the blade: the reach, exactly.
    pub tip: V3,
    /// The direction the blade bows, unit and square to the haft. With the
    /// haft's own direction it spans the plane the blade's flat lies in.
    pub bow: V3,
    /// The hit volume's radius: how far the swept volume extends either side
    /// of the blade's plane.
    pub width: f32,
    /// How broad the blade is across its flat. Grows with her grey: the blade
    /// is her blood, and the more of it she has let out the more of it there
    /// is.
    pub breadth: f32,
}

impl Scythe {
    /// Grip to tip: the reach.
    pub fn reach(&self) -> f32 {
        math::length(math::sub(self.tip, self.grip))
    }

    /// The haft's direction, butt to tip, unit.
    pub fn axis(&self) -> V3 {
        math::normalize_or(math::sub(self.tip, self.butt), [0.0, 1.0, 0.0])
    }
}

/// How much of the reach is blade rather than haft.
const BLADE: f32 = 0.34;
/// How far the neck sits off the reach's line, as a share of the reach: the
/// blade is set at an angle. Small: the tip is on the line, and the set is
/// what makes it a scythe rather than a spear.
const SET: f32 = 0.10;
/// How far the blade's middle bows past the straight line from neck to tip,
/// as a share of the blade: the curve of it.
const CURVE: f32 = 0.15;
/// How far the pole runs on behind the trailing hand while she swings it, and
/// behind the grip while she carries it.
const BUTT: f32 = 0.35;
/// How broad the blade is across its flat with no grey open. Doubles at a full
/// bar.
const BREADTH: f32 = 0.14;
/// How tall the standing haft is, as a share of her height: its head is level
/// with hers, and the blade rises past it.
const STANDS: f32 = 1.0;
/// How long the standing blade is, as a share of the haft.
const HEAD: f32 = 0.5;
/// Which way the standing blade leaves the head of the haft, as forward, up
/// and to her side: mostly forward, a little up, so its tip ends a little over
/// her head.
const LEANS: V3 = [0.94, 0.35, 0.0];

/// Where the scythe is, if this fighter carries one.
///
/// `left` and `right` are where the drawn hands are in the arena, which the
/// renderer knows and the simulation does not: they are the one thing here
/// that is not read off the snapshot, and they decide where the weapon *is
/// held*, never how long it is. The right hand leads on the haft -- see
/// `anim::clips::blood::gripping` -- and the left is the one that carries it
/// at rest.
pub fn scythe(p: &Player, left: V3, right: V3) -> Option<Scythe> {
    if p.class != sim::Class::BloodMage {
        return None;
    }
    let facing = math::normalize_or(
        [crate::fx(p.facing.x), 0.0, crate::fx(p.facing.z)],
        [1.0, 0.0, 0.0],
    );
    let reach = crate::fx(state::scythe_reach(p));
    let width = crate::fx(sim::moves::get(p.class, sim::moves::blood::SWEEP).radius);
    let breadth = BREADTH * (1.0 + crate::fx(p.grey_share()));
    let in_a_swing = p
        .action
        .attack_kind()
        .is_some_and(sim::moves::blood::scythe);
    if in_a_swing {
        // Both hands are on the haft: the pole runs from behind the trailing
        // hand through the leading one. While the volume is out the tip is
        // its far end; otherwise the tip is the reach along the hands' line.
        let hands = math::normalize_or(math::sub(right, left), facing);
        let (tip, width) = match state::hitbox(p) {
            Some(hb) => (fx3(hb.to), crate::fx(hb.radius)),
            None => (math::add(right, math::scale(hands, reach)), width),
        };
        let axis = math::normalize_or(math::sub(tip, right), hands);
        let apart = math::length(math::sub(right, left));
        let butt = math::sub(right, math::scale(axis, apart + BUTT));
        // The blade bows up out of the plane of the cut where it can, and
        // forward where the cut is already vertical.
        let bow = square(axis, [0.0, 1.0, 0.0])
            .unwrap_or_else(|| square(axis, facing).unwrap_or([0.0, 1.0, 0.0]));
        // The blade is set at an angle: the neck sits a little off the line
        // and the tip is back on it, which is what makes it a scythe rather
        // than a spear.
        let length = math::length(math::sub(tip, right));
        let neck = math::add(
            math::sub(tip, math::scale(axis, length * BLADE)),
            math::scale(bow, length * SET),
        );
        return Some(set(butt, right, neck, tip, bow, width, breadth));
    }
    // Standing: the haft upright on the floor under the holding hand, and
    // the blade off its head, forward and a little up. It grows with grey by
    // the same factor as the reach.
    let base = crate::fx(sim::moves::get(p.class, sim::moves::blood::SWEEP).reach);
    let grows = if base > 0.0 { reach / base } else { 1.0 };
    let floor = crate::fx(p.pos.y);
    let haft = crate::fx(sim::tuning::body_height()) * STANDS * grows;
    let butt = [left[0], floor, left[2]];
    let neck = [left[0], floor + haft, left[2]];
    let grip = [left[0], left[1].clamp(floor, floor + haft), left[2]];
    let side = math::normalize_or(math::cross([0.0, 1.0, 0.0], facing), [0.0, 0.0, 1.0]);
    let leans = math::normalize_or(
        math::add(
            math::add(
                math::scale(facing, LEANS[0]),
                math::scale([0.0, 1.0, 0.0], LEANS[1]),
            ),
            math::scale(side, LEANS[2]),
        ),
        facing,
    );
    let tip = math::add(neck, math::scale(leans, haft * HEAD));
    // The blade's flat lies in the plane of her facing, bowing forward.
    Some(set(butt, grip, neck, tip, facing, width, breadth))
}

/// The part of `v` square to `axis`, unit, or `None` if there is none.
fn square(axis: V3, v: V3) -> Option<V3> {
    let flat = math::sub(v, math::scale(axis, math::dot(v, axis)));
    (math::length(flat) > 0.05).then(|| math::normalize_or(flat, axis))
}

/// Set the blade: the haft runs from `butt` through `grip` to `neck`, and the
/// blade curves from the neck to `tip`, bowing toward `bow`.
fn set(butt: V3, grip: V3, neck: V3, tip: V3, bow: V3, width: f32, breadth: f32) -> Scythe {
    let blade = math::length(math::sub(tip, neck));
    let mid = math::add(
        math::scale(math::add(neck, tip), 0.5),
        math::scale(bow, blade * CURVE),
    );
    Scythe {
        butt,
        grip,
        neck,
        mid,
        tip,
        bow,
        width,
        breadth,
    }
}

fn fx3(v: sim::V3) -> V3 {
    [crate::fx(v.x), crate::fx(v.y), crate::fx(v.z)]
}
