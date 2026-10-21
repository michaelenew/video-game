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
//! **One roll, the kind a third-person action game has** (2026-10-21, from
//! play: the first version, with a twist, a cartwheel and a laid-out aerial,
//! "looked insane"): a hop off both feet, chin and knees to the chest, one
//! turn along the floor in a round ball, and up onto the feet. Back, it is a
//! backward roll. **To the side the body turns to face the travel, rolls
//! forward over the lead shoulder, and turns back as it rises** -- nobody
//! goes over their own shoulder line and comes up on their feet.
//!
//! Each class rolls it its own way, in timing, weight and what the arms carry:
//!
//! | Class | Its roll |
//! | --- | --- |
//! | Champion | The roll as it is; up with the weapon hand forward |
//! | Bulwark | A frame later over and up, heavier, the shield arm across the ball, up behind the shield |
//! | Shadow Reaver | Quickest over and up, blades swept back off the hop, up light and less deep |
//! | Elementalist | Slides on out of it, low, a hand dragged along the floor |
//! | Blood mage | Last over, unfurls low with the scythe arm thrown out wide |
//! | Dual mage | Slowest and lightest, up with her arms opening wide |
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
    /// Where the body turns about, in character space (`+Y` up, `+Z` ahead):
    /// about the middle of the ball the clip curls into. It decides the arc;
    /// the height is then put right by [`lift`], which keeps whatever is
    /// lowest on the floor -- so a pivot a few centimetres out is a slightly
    /// different arc rather than a body in the floor.
    pub about: V3,
}

/// The roll a class does. See the module's table.
pub const fn roll(class: Class) -> Roll {
    match class {
        Class::Champion => Roll {
            from: 2,
            past_cover: 3,
            about: [0.0, 0.47, 0.30],
        },
        Class::Bulwark => Roll {
            from: 3,
            past_cover: 4,
            about: [0.0, 0.42, 0.28],
        },
        Class::ShadowReaver => Roll {
            from: 2,
            past_cover: 1,
            about: [0.0, 0.52, 0.31],
        },
        Class::Elementalist => Roll {
            from: 2,
            past_cover: 2,
            about: [0.0, 0.50, 0.32],
        },
        Class::BloodMage => Roll {
            from: 4,
            past_cover: 3,
            about: [0.0, 0.44, 0.31],
        },
        Class::DualMage => Roll {
            from: 2,
            past_cover: 5,
            about: [0.0, 0.50, 0.33],
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

/// A roll's whole-body turn at one frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Turn {
    /// How far the body is turned to face the way it is rolling, in radians
    /// about the vertical through its feet, the engine's way round: a roll to
    /// the side turns to face the travel first, rolls forward, and turns back
    /// as it comes up. Nought going forward or back.
    pub yaw: f32,
    /// The roll itself, in the body's own frame once it has turned: over its
    /// own side, forward or back.
    pub over: Quat,
    /// The point it rolls about, in that frame.
    pub about: V3,
}

impl Turn {
    /// The whole rotation, character space to drawn: the yaw, then the roll.
    pub fn whole(&self) -> Quat {
        Quat::from_y(self.yaw).mul(self.over)
    }
}

/// The share of a quarter turn a roll to the side is turned toward its travel,
/// at a frame: in over the push, held through the roll, back out over the
/// rise.
fn facing_travel(r: &Roll, frame: f32) -> f32 {
    let ease = |t: f32| {
        let t = t.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    let last = length().saturating_sub(1) as f32;
    let up = r.upright() as f32;
    let into = ease(frame / (r.from as f32 + 1.0).max(1.0));
    let out = ease((frame - up) / (last - up).max(1.0));
    into * (1.0 - out)
}

/// The whole-body turn of a class's roll at a frame of the clip, in character
/// space as the renderer draws it.
///
/// Forward and back go over the body's own side. **To the side it turns to
/// face the travel and rolls forward**, the way anybody actually rolls
/// sideways -- a body cannot go over its own shoulder line and come up on its
/// feet -- and turns back to the front as it rises. A pose is drawn with the
/// body's right at `-X` (see `crate::hand_joint`), so facing the right is a
/// quarter turn the negative way about `+Y`.
pub fn turned(class: Class, way: Way, frame: f32) -> Turn {
    use std::f32::consts::FRAC_PI_2;
    let r = roll(class);
    let over = r.through(frame) * TAU;
    let side = match way {
        Way::Right => -FRAC_PI_2,
        Way::Left => FRAC_PI_2,
        Way::Forward | Way::Back => 0.0,
    };
    let over = match way {
        Way::Back => Quat::from_x(-over),
        _ => Quat::from_x(over),
    };
    Turn {
        yaw: side * facing_travel(&r, frame),
        over,
        about: r.about,
    }
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
}

/// How a class's roll is held to the floor at a frame of the clip.
pub fn floor(class: Class, frame: f32) -> Floor {
    let r = roll(class);
    let k = r.through(frame);
    let w = (2.0 * (k * std::f32::consts::PI).sin()).clamp(0.0, 1.0);
    if w <= 0.0 { Floor::Free } else { Floor::On(w) }
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
