//! The Blood mage's scythe, and the essence around it that is the reach.
//!
//! Her reach grows with the grey on her bar, and `docs/design/aiming.md`
//! refuses a reach that changes with a bar for a good reason: a reach only one
//! player can see is unlearnable. The one condition under which it is allowed
//! is the rule `CLAUDE.md` puts on the debug overlay -- **the volume is drawn
//! at the size it hits at** -- so that what both players read is a thing in
//! the world, not a number. This module is that rule, in two halves. The
//! **weapon** is iron and one size: it is drawn at the row's reach, never
//! longer, and rides her hands. The **volume** -- [`swing_volume`] -- is the
//! hit test's own capsule, reach and radius both grown by her grey, and the
//! renderer draws it in the same stuff as the essence pools: it is the life
//! force doing the swinging, and the more of it she has let out, the more of
//! it there is around the blade. Both are pure functions of simulation state
//! so `tests/kinematics.rs` can hold them to the hit test without a renderer,
//! the way `game::effect_piece` is held to the effects.
//!
//! A war scythe is a long haft with a curved blade set at the head. It is
//! drawn as a **butt**, a **grip**, a **neck**, a **mid** and a **tip**: the
//! haft runs from the butt through the grip to the neck, and the blade curves
//! from the neck through the mid to the tip. While she is swinging, the tip
//! points at the far end of the hit volume and the haft runs back from it
//! through her leading hand and on past her trailing one, so the weapon rides
//! the arms the clips put on it; with no grey open the tip sits exactly on the
//! volume's end, and with grey open the volume runs on past it. Through the
//! wind-up and the recovery, when nothing is out, the haft simply lies along
//! the line of her two hands. At rest the weapon **stands**: the haft upright
//! with its butt on the floor under her right hand, its head level
//! with hers, and the blade leaving the head curving forward and a little over
//! it -- the way a scythe is stood when it is not being swung.
//!
//! The blade's *flat* lies in the plane that contains the haft and the bow --
//! the plane of the cut while she swings, the plane of her facing at rest --
//! and the renderer builds each piece's rotation from that plane rather than
//! from the axis alone, which is what stops the blade rolling to some world
//! direction of its own.

use crate::math::{self, V3};
use sim::state::{self, Player};

/// The hit volume of a swing, as the renderer draws it: the same capsule the
/// hit test reads, and a fade for the moment after it.
///
/// `fade` is one on every active frame and runs down over the first
/// [`GHOST`] frames of the recovery, where the volume is where it was on the
/// last active frame: a swing that vanished on the frame it stopped hitting
/// was a flicker, and the trail of it is what shows the arc.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Volume {
    pub from: V3,
    pub to: V3,
    pub radius: f32,
    pub fade: f32,
}

/// How many recovery frames the volume lingers for, fading.
pub const GHOST: u16 = 8;

/// The volume of a scythe swing this frame, if there is one to draw.
///
/// The hit test's own capsule on an active frame, and its last shape fading
/// through the first frames of recovery. Nothing during the wind-up, because
/// nothing is out.
pub fn swing_volume(p: &Player) -> Option<Volume> {
    match p.action {
        sim::state::Action::Active { kind, .. } if sim::moves::blood::scythe(kind) => {
            let hb = state::hitbox(p)?;
            Some(Volume {
                from: fx3(hb.from),
                to: fx3(hb.to),
                radius: crate::fx(hb.radius),
                fade: 1.0,
            })
        }
        sim::state::Action::Recovery { kind, left } if sim::moves::blood::scythe(kind) => {
            let m = sim::moves::get(p.class, kind);
            let gone = m.recovery.saturating_sub(left);
            if gone >= GHOST {
                return None;
            }
            // The last active frame, re-asked of the hit test.
            let mut last = *p;
            last.action = sim::state::Action::Active { kind, left: 1 };
            let hb = state::hitbox(&last)?;
            Some(Volume {
                from: fx3(hb.from),
                to: fx3(hb.to),
                radius: crate::fx(hb.radius),
                fade: 1.0 - gone as f32 / GHOST as f32,
            })
        }
        _ => None,
    }
}

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
    /// How broad the blade is across its flat.
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
/// How broad the blade is across its flat.
const BREADTH: f32 = 0.16;
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
/// `anim::clips::blood::gripping` -- and is the one it stands under at rest,
/// so the left is free to throw and to cast without the weapon following it.
pub fn scythe(p: &Player, left: V3, right: V3) -> Option<Scythe> {
    if p.class != sim::Class::BloodMage {
        return None;
    }
    let facing = math::normalize_or(
        [crate::fx(p.facing.x), 0.0, crate::fx(p.facing.z)],
        [1.0, 0.0, 0.0],
    );
    let sweep = sim::moves::get(p.class, sim::moves::blood::SWEEP);
    // The weapon's own length, off the row: it does not grow. What grows is
    // the volume, drawn separately -- see `swing_volume`.
    let reach = crate::fx(sweep.reach);
    let live = crate::fx(state::scythe_reach(p));
    let width = crate::fx(sweep.radius);
    let breadth = BREADTH;
    let in_a_swing = p
        .action
        .attack_kind()
        .is_some_and(sim::moves::blood::scythe);
    if in_a_swing {
        // Both hands are on the haft: the pole runs from behind the trailing
        // hand through the leading one. While the volume is out the tip
        // points at its far end, the weapon's own length from the hand,
        // which is exactly the volume's end when no grey is open; otherwise
        // the tip is that length along the hands' line.
        let hands = math::normalize_or(math::sub(right, left), facing);
        let tip = match state::hitbox(p) {
            Some(hb) => {
                let toward = math::sub(fx3(hb.to), right);
                let share = if live > 0.0 { reach / live } else { 1.0 };
                math::add(right, math::scale(toward, share))
            }
            None => math::add(right, math::scale(hands, reach)),
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
    // Standing: the haft upright on the floor under the right hand -- the
    // one that leads on it in a swing, leaving the left free to throw and
    // cast -- and the blade off its head, forward and a little up.
    let floor = crate::fx(p.pos.y);
    let haft = crate::fx(sim::tuning::body_height()) * STANDS;
    let butt = [right[0], floor, right[2]];
    let neck = [right[0], floor + haft, right[2]];
    let grip = [right[0], right[1].clamp(floor, floor + haft), right[2]];
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
