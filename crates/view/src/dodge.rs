//! The grounded dodge as each class rolls it.
//!
//! **A dive roll, turned over for real.** The rig cannot roll over on its own
//! -- the root has fifty degrees of forward swing -- so for years the dodge sold
//! a roll by curling up and leaning. Now the clip is only what a body does
//! *inside* a roll (a dive, a tuck, legs reaching for the floor) and the whole
//! body is turned over on top of it, about the middle of the ball, the way the
//! Reaver's Tumble already was. The turn is a pure function of the dodge's
//! countdown, so a rollback redraws it exactly, and it is **presentation
//! only**: the hurt volume, the camera and the aim never turn over, and nothing
//! here touches a frame of the dodge's data.
//!
//! Every class rolls, and each rolls the way it fights (2026-10-21):
//!
//! | Class | The roll |
//! | --- | --- |
//! | Champion | The soldier's roll: a dive onto the hands, one tight turn, up in a low crouch with the weapon forward |
//! | Bulwark | The shield roll: late, heavy and low, tucked behind the shield, and up behind it |
//! | Shadow Reaver | The twisting dive: long and flat, a full twist through the turn, blades held out, and the first up, light |
//! | Elementalist | The earth roll: comes out of the turn sliding on one knee with a hand dragged on the floor |
//! | Blood mage | The low tumble: the longest dive and the latest turn, sunk low, the scythe arm trailing out wide |
//! | Dual mage | The aerial: no tuck at all -- laid out, arms spread like wings, turned about her waist, slowest of all |
//!
//! Two things are the same for everyone, because they are the dodge's rather
//! than the class's. The turn is mostly done by the frame the invulnerability
//! ends, so the silhouette still changes on that frame -- tucked while the
//! frames are free, opening as they stop being -- and it is done by frames,
//! not seconds, out of `tuning::dodge_frames` and `dodge_iframes`, so
//! retuning the window in the Oven moves the whole roll with it.

use crate::clips::Clip;
use crate::math::{self, Quat, V3};
use crate::skeleton::{JOINTS, Skeleton, Skin};
use sim::Class;
use std::f32::consts::TAU;

/// Which way a dodge went, in the body's own frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Way {
    Forward,
    Back,
    Left,
    Right,
}

pub const WAYS: [Way; 4] = [Way::Forward, Way::Back, Way::Left, Way::Right];

/// Which way the body is actually going, not which key was pressed: a dodge is
/// a burst of velocity and the velocity is the honest source. `travel` is
/// strafe and forward in the character's frame, positive to its right and
/// ahead (`interp::PlayerView::travel`). The clip and the turn both read it
/// here, so they can never disagree about which way the roll goes.
pub fn way(travel: [f32; 2]) -> Way {
    let [strafe, forward] = travel;
    if forward.abs() >= strafe.abs() {
        if forward >= 0.0 {
            Way::Forward
        } else {
            Way::Back
        }
    } else if strafe >= 0.0 {
        Way::Right
    } else {
        Way::Left
    }
}

/// How one class turns over.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Roll {
    /// The frame of the dodge the body starts going over on.
    pub from: u16,
    /// How many frames after the cover runs out it is upright again (negative:
    /// before). The turn is eased in and out between the two.
    pub past_cover: i16,
    /// Turns about its own long axis on top of a forward or backward roll: the
    /// Reaver's twist. Nought for everybody else.
    pub twist: f32,
    /// Where the body turns about, in character space (`+Y` up, `+Z` ahead):
    /// about the middle of the ball the clip curls into. It decides the arc;
    /// the height is then put right by [`lift`], which keeps whatever is
    /// lowest on the floor -- so a pivot a few centimetres out is a slightly
    /// different arc rather than a body in the floor.
    pub about: V3,
    /// Over in the air rather than on the floor: kept out of it, never pulled
    /// down onto it. The Dual mage's aerial.
    pub aerial: bool,
}

/// The roll a class does. See the module's table.
pub const fn roll(class: Class) -> Roll {
    match class {
        Class::Champion => Roll {
            from: 2,
            past_cover: 3,
            twist: 0.0,
            about: [0.0, 0.47, 0.41],
            aerial: false,
        },
        Class::Bulwark => Roll {
            from: 3,
            past_cover: 4,
            twist: 0.0,
            about: [0.0, 0.39, 0.37],
            aerial: false,
        },
        Class::ShadowReaver => Roll {
            from: 2,
            past_cover: 1,
            twist: 1.0,
            about: [0.0, 0.52, 0.41],
            aerial: false,
        },
        Class::Elementalist => Roll {
            from: 2,
            past_cover: 2,
            twist: 0.0,
            about: [0.0, 0.49, 0.43],
            aerial: false,
        },
        Class::BloodMage => Roll {
            from: 4,
            past_cover: 3,
            twist: 0.0,
            about: [0.0, 0.41, 0.42],
            aerial: false,
        },
        Class::DualMage => Roll {
            from: 1,
            past_cover: 6,
            twist: 0.0,
            about: [0.0, 1.0, -0.1],
            aerial: true,
        },
    }
}

/// The frames of a grounded dodge, as the clip is indexed.
pub fn length() -> u16 {
    sim::tuning::dodge_frames().max(2)
}

/// The first frame of the clip the body can be hit on.
pub fn first_vulnerable() -> u16 {
    let spent = sim::tuning::dodge_frames().saturating_sub(sim::tuning::dodge_iframes());
    length().saturating_sub(spent)
}

impl Roll {
    /// The frame the body is upright again, inside the clip.
    pub fn upright(&self) -> u16 {
        let last = length().saturating_sub(1);
        let at = first_vulnerable() as i32 + self.past_cover as i32;
        (at.max(self.from as i32 + 2) as u16).min(last)
    }

    /// How far over the body is at a frame of the clip, nought to one, eased in
    /// and out so it starts and ends upright and still.
    pub fn through(&self, frame: f32) -> f32 {
        let (a, b) = (self.from as f32, self.upright() as f32);
        let t = ((frame - a) / (b - a).max(1.0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
}

/// The clip a class plays for a dodge that went this way.
pub const fn clip(class: Class, way: Way) -> Clip {
    use Way::*;
    match (class, way) {
        (Class::Champion, Forward) => Clip::DodgeForward,
        (Class::Champion, Back) => Clip::DodgeBack,
        (Class::Champion, Left) => Clip::DodgeLeft,
        (Class::Champion, Right) => Clip::DodgeRight,
        (Class::Bulwark, Forward) => Clip::BulwarkDodgeForward,
        (Class::Bulwark, Back) => Clip::BulwarkDodgeBack,
        (Class::Bulwark, Left) => Clip::BulwarkDodgeLeft,
        (Class::Bulwark, Right) => Clip::BulwarkDodgeRight,
        (Class::ShadowReaver, Forward) => Clip::ReaverDodgeForward,
        (Class::ShadowReaver, Back) => Clip::ReaverDodgeBack,
        (Class::ShadowReaver, Left) => Clip::ReaverDodgeLeft,
        (Class::ShadowReaver, Right) => Clip::ReaverDodgeRight,
        (Class::Elementalist, Forward) => Clip::ElementalistDodgeForward,
        (Class::Elementalist, Back) => Clip::ElementalistDodgeBack,
        (Class::Elementalist, Left) => Clip::ElementalistDodgeLeft,
        (Class::Elementalist, Right) => Clip::ElementalistDodgeRight,
        (Class::BloodMage, Forward) => Clip::BloodDodgeForward,
        (Class::BloodMage, Back) => Clip::BloodDodgeBack,
        (Class::BloodMage, Left) => Clip::BloodDodgeLeft,
        (Class::BloodMage, Right) => Clip::BloodDodgeRight,
        (Class::DualMage, Forward) => Clip::DualDodgeForward,
        (Class::DualMage, Back) => Clip::DualDodgeBack,
        (Class::DualMage, Left) => Clip::DualDodgeLeft,
        (Class::DualMage, Right) => Clip::DualDodgeRight,
    }
}

/// The whole-body turn of a class's roll at a frame of the clip, in character
/// space as the renderer draws it, and the point it turns about.
///
/// Forward and back go over the body's own side; left and right over its
/// front, toward the travel. A pose is drawn with the body's right at `-X`
/// (see `crate::hand_joint`), so a turn about `+Z` by a positive angle carries
/// the head to the right.
pub fn turned(class: Class, way: Way, frame: f32) -> (Quat, V3) {
    let r = roll(class);
    let k = r.through(frame);
    let over = k * TAU;
    let q = match way {
        Way::Forward => Quat::from_x(over).mul(Quat::from_y(k * r.twist * TAU)),
        Way::Back => Quat::from_x(-over).mul(Quat::from_y(-k * r.twist * TAU)),
        Way::Right => Quat::from_z(over),
        Way::Left => Quat::from_z(-over),
    };
    (q, r.about)
}

/// How a turned-over body is held against the floor this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Floor {
    /// Nothing holds it: a flip in the air, or not turned at all.
    Free,
    /// Rolling on it: its lowest point is put on the floor, by this much
    /// (nought to one) -- all of it while the body is over, none at the two
    /// ends where the feet are what is on the floor.
    On(f32),
    /// Going over above it: kept out of it by this much, never pulled down.
    Above(f32),
}

/// How a class's roll is held to the floor at a frame of the clip.
pub fn floor(class: Class, frame: f32) -> Floor {
    let r = roll(class);
    let k = r.through(frame);
    let w = (2.0 * (k * std::f32::consts::PI).sin()).clamp(0.0, 1.0);
    if w <= 0.0 {
        Floor::Free
    } else if r.aerial {
        Floor::Above(w)
    } else {
        Floor::On(w)
    }
}

/// The lowest point of a posed body once it is turned over `turn` about
/// `about`, in character space: the lowest corner of any part. No allocation:
/// the renderer asks it every frame of a roll.
pub fn lowest(skin: &Skin, skeleton: &Skeleton, turn: Quat, about: V3) -> f32 {
    let mut low = f32::MAX;
    for j in JOINTS {
        let (centre, rot) = skin.box_of(skeleton, j);
        let h = skeleton.bone(j).half;
        for sx in [-1.0, 1.0f32] {
            for sy in [-1.0, 1.0f32] {
                for sz in [-1.0, 1.0f32] {
                    let corner = math::add(centre, rot.rotate([sx * h[0], sy * h[1], sz * h[2]]));
                    let turned = math::add(about, turn.rotate(math::sub(corner, about)));
                    low = low.min(turned[1]);
                }
            }
        }
    }
    low
}

/// How far to raise (or lower) a rolling body so it is on the floor: what a
/// roll is, a body that keeps touching the ground as it goes over it. Nought
/// when nothing holds it.
pub fn lift(floor: Floor, lowest: f32) -> f32 {
    match floor {
        Floor::Free => 0.0,
        Floor::On(w) => -lowest * w,
        Floor::Above(w) => (-lowest).max(0.0) * w,
    }
}

/// How far the renderer raises a drawn body for its roll this frame, given the
/// pose it is drawing on its own skeleton. See [`lift`].
pub fn drawn_lift(p: &crate::PlayerView, skin: &Skin, skeleton: &Skeleton) -> f32 {
    if p.roll_floor == Floor::Free {
        return 0.0;
    }
    lift(p.roll_floor, lowest(skin, skeleton, p.roll, p.roll_about))
}
