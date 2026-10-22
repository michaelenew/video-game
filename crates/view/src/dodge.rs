//! The grounded dodge as each class rolls it.
//!
//! **A dive roll, turned over for real.** The rig cannot roll over on its own
//! -- the root has fifty degrees of forward swing -- so the clip is only what a
//! body does *inside* the roll, and the whole body is turned over on top of
//! it, the way the Reaver's Tumble already was. The turn is a pure function of
//! the dodge's countdown, so a rollback redraws it exactly, and it is
//! **presentation only**: the hurt volume, the camera and the aim never turn
//! over, and nothing here touches a frame of the dodge's data.
//!
//! ## The roll, after the souls games' (2026-10-21)
//!
//! Twice wrong first, from play: a twist and a cartwheel ("insane"), then a
//! body that dropped straight to the floor and somersaulted ("not close").
//! What it is instead is a **dive roll**, in three parts:
//!
//! 1. **The dive.** Off both feet and *into the air*: the body long, the hands
//!    out ahead, the legs trailing, pitching forward as it flies on a low arc.
//!    The torso stays up -- its height comes down over the flight, not on the
//!    first frame.
//! 2. **The shoulder.** As the hands reach the floor the head tucks and the
//!    lead shoulder drops, and the roll goes over *diagonally*, from that
//!    shoulder across the back to the other hip, about a tilted axis.
//! 3. **The roll and the rise.** The rest of the turn is fast, on the floor,
//!    and comes up onto the feet.
//!
//! Back, it is the same thing backward: a hop back, sat down onto the back,
//! over the shoulder. **To the side the body turns to face the travel, dives
//! forward, and turns back as it rises.**
//!
//! The height is not left to the pivot: [`Turn::height`] is where the lowest
//! point of the body is meant to be -- off the floor on the arc of the dive,
//! on it through the roll -- and the renderer puts it there (`lift`).
//!
//! Each class rolls it its own way, in timing, the height of the dive, weight
//! and what the arms carry:
//!
//! | Class | Its roll |
//! | --- | --- |
//! | Champion | The roll as it is; up with the weapon hand forward |
//! | Bulwark | Lower and later to the floor, heavier, the shield arm across him, up behind the shield |
//! | Shadow Reaver | The highest, quickest dive, blades swept back off the push, up light |
//! | Elementalist | Slides on out of it, low, a hand dragged along the floor |
//! | Blood mage | The longest, flattest dive, unfurls low with the scythe arm thrown out wide |
//! | Dual mage | A high, floating dive, slowest up, arms opening wide |
//!
//! The timing is by frames, not seconds, out of `tuning::dodge_frames` and
//! `dodge_iframes`, so retuning the window in the Oven moves the whole roll
//! with it. The body stays balled until it is most of the way round, and
//! that is never before the invulnerability ends, so the silhouette still
//! opens no earlier than the cover goes.

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
/// `interp::PlayerView::travel`, strafe and forward in the character's frame.
///
/// **Its strafe is positive to the body's *left* as drawn**, whatever its own
/// comment says: `D` moves a body to its right and reads negative there
/// (2026-10-22, from play -- the first rolls turned to face away from their
/// travel and dived the wrong way). `tests/dodge.rs`
/// (`a_roll_to_the_side_dives_the_way_the_body_goes`) holds the roll to the
/// body's real motion rather than to the sign. The clip and the turn both
/// read it here, so they can never disagree about which way the roll goes.
pub fn way(travel: [f32; 2]) -> Way {
    let [strafe, forward] = travel;
    if forward.abs() >= strafe.abs() {
        if forward >= 0.0 {
            Way::Forward
        } else {
            Way::Back
        }
    } else if strafe >= 0.0 {
        Way::Left
    } else {
        Way::Right
    }
}

/// How one class dives and rolls.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Roll {
    /// The frame the hands and shoulder reach the floor: the end of the dive
    /// and the start of the roll.
    pub contact: u16,
    /// How many frames after the cover runs out it is upright again (negative:
    /// before).
    pub past_cover: i16,
    /// How high the dive carries the lowest point of the body at the top of its
    /// arc, in metres.
    pub hop: f32,
    /// How far over the body has pitched, in turns, by the time it reaches the
    /// floor: the angle of the dive.
    pub dive: f32,
    /// Where the body turns about, in character space (`+Y` up, `+Z` ahead):
    /// about the middle of the ball it curls into. It decides the arc; the
    /// height is [`Turn::height`]'s.
    pub about: V3,
}

/// The roll a class does. See the module's table.
pub const fn roll(class: Class) -> Roll {
    match class {
        Class::Champion => Roll {
            contact: 7,
            past_cover: 6,
            hop: 0.24,
            dive: 0.2,
            about: [0.0, 0.50, 0.30],
        },
        Class::Bulwark => Roll {
            contact: 8,
            past_cover: 7,
            hop: 0.14,
            dive: 0.17,
            about: [0.0, 0.46, 0.28],
        },
        Class::ShadowReaver => Roll {
            contact: 7,
            past_cover: 5,
            hop: 0.32,
            dive: 0.22,
            about: [0.0, 0.54, 0.31],
        },
        Class::Elementalist => Roll {
            contact: 7,
            past_cover: 6,
            hop: 0.24,
            dive: 0.2,
            about: [0.0, 0.52, 0.32],
        },
        Class::BloodMage => Roll {
            contact: 8,
            past_cover: 6,
            hop: 0.2,
            dive: 0.23,
            about: [0.0, 0.48, 0.31],
        },
        Class::DualMage => Roll {
            contact: 7,
            past_cover: 7,
            hop: 0.34,
            dive: 0.2,
            about: [0.0, 0.52, 0.33],
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

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl Roll {
    /// The frame the shoulder meets the floor, inside the clip and before the
    /// cover runs out.
    pub fn contact(&self) -> u16 {
        self.contact
            .clamp(2, first_vulnerable().saturating_sub(1).max(2))
    }

    /// The frame the body is upright again, inside the clip.
    pub fn upright(&self) -> u16 {
        let last = length().saturating_sub(1);
        let at = first_vulnerable() as i32 + self.past_cover as i32;
        (at.max(self.contact() as i32 + 3) as u16).min(last)
    }

    /// How far over the body is at a frame of the clip, nought to one turn.
    ///
    /// Through the dive it pitches forward faster and faster, to `dive` at the
    /// contact -- a body leaving the floor leaning, and falling further over
    /// as it flies. Then it rolls on at the speed it arrived with, gathering
    /// through the middle of the roll and easing out as it comes up onto its
    /// feet: one continuous turn, no snap where the floor is met (2026-10-22,
    /// from play: the first version rolled the rest of the way in half a dozen
    /// frames and "read too fast in motion").
    pub fn through(&self, frame: f32) -> f32 {
        let (c, u) = (self.contact() as f32, self.upright() as f32);
        if frame <= 0.0 {
            0.0
        } else if frame <= c {
            self.dive * (frame / c).powi(2)
        } else if frame < u {
            // A cubic from `dive` to a whole turn, leaving at the dive's own
            // rate and arriving still.
            let t = (frame - c) / (u - c);
            let leaving = 2.0 * self.dive / c * (u - c);
            let (t2, t3) = (t * t, t * t * t);
            (2.0 * t3 - 3.0 * t2 + 1.0) * self.dive
                + (t3 - 2.0 * t2 + t) * leaving
                + (-2.0 * t3 + 3.0 * t2)
        } else {
            1.0
        }
    }

    /// The frame the legs come out of the ball for the floor: the cover's
    /// last frame or later, once the body is most of the way round.
    pub fn opens(&self) -> u16 {
        let from = first_vulnerable();
        (from..self.upright())
            .find(|&f| self.through(f as f32) >= 0.8)
            .unwrap_or(from)
            .max(from)
    }

    /// How far the roll's axis has tipped onto the lead shoulder, nought to
    /// one: none through the dive, all of it from just after the contact.
    pub fn shoulder(&self, frame: f32) -> f32 {
        let c = self.contact() as f32;
        smooth((frame - (c - 2.0)) / 3.0)
    }

    /// Where the lowest point of the body is meant to be at a frame, in metres
    /// above the floor: up on the arc of the dive, on the floor from the
    /// contact.
    pub fn height(&self, frame: f32) -> f32 {
        let c = self.contact() as f32;
        if frame <= 0.0 || frame >= c {
            0.0
        } else {
            self.hop * (std::f32::consts::PI * frame / c).sin()
        }
    }

    /// How firmly the body is held to [`Roll::height`], nought to one: not at
    /// the push, where the feet are on the floor in the clip, nor once it is
    /// up, where they are again; fully between.
    pub fn held(&self, frame: f32) -> f32 {
        let u = self.upright() as f32;
        smooth(frame) * (1.0 - smooth((frame - (u - 2.0)) / 2.0))
    }
}

/// How far the shoulder roll's axis is tipped off the body's side, about the
/// vertical: over the lead shoulder to the other hip.
const SHOULDER: f32 = 0.6;

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
    /// the side turns to face the travel first, dives forward, and turns back
    /// as it comes up. Nought going forward or back.
    pub yaw: f32,
    /// The dive and the roll, in the body's own frame once it has turned.
    pub over: Quat,
    /// The point it turns about, in that frame.
    pub about: V3,
    /// Where the lowest point of the body is meant to be, metres above the
    /// floor, and how firmly it is held there (`Roll::height`, `Roll::held`).
    pub height: f32,
    pub held: f32,
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
    let last = length().saturating_sub(1) as f32;
    let up = r.upright() as f32;
    let into = smooth(frame / 4.0);
    let out = smooth((frame - up) / (last - up).max(1.0));
    into * (1.0 - out)
}

/// The whole-body turn of a class's roll at a frame of the clip, in character
/// space as the renderer draws it.
///
/// Forward and back go over the body's own side, tipped onto the right
/// shoulder from the contact. **To the side it turns to face the travel and
/// dives forward** over the lead shoulder -- a body cannot go over its own
/// shoulder line and come up on its feet -- and turns back to the front as it
/// rises. A pose is drawn with the body's right at `-X` (see
/// `crate::hand_joint`), so facing the right is a quarter turn the negative
/// way about `+Y`.
pub fn turned(class: Class, way: Way, frame: f32) -> Turn {
    use std::f32::consts::FRAC_PI_2;
    let r = roll(class);
    let angle = r.through(frame) * TAU;
    let (side, sign, lead) = match way {
        Way::Right => (-FRAC_PI_2, 1.0, 1.0),
        Way::Left => (FRAC_PI_2, 1.0, -1.0),
        Way::Forward => (0.0, 1.0, 1.0),
        Way::Back => (0.0, -1.0, 1.0),
    };
    // Over the body's side, with the axis tipped about the vertical onto the
    // lead shoulder: conjugating the turn by that tip is a turn about the
    // tipped axis, and still a whole turn round to upright at the end.
    let tip = Quat::from_y(lead * sign * SHOULDER * r.shoulder(frame));
    let over = tip.mul(Quat::from_x(sign * angle)).mul(tip.conjugate());
    Turn {
        yaw: side * facing_travel(&r, frame),
        over,
        about: r.about,
        height: r.height(frame),
        held: r.held(frame),
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

/// How far to raise (or lower) a rolling body so its lowest point is at
/// `height`, held by `held`: up off the floor on the dive, on it through the
/// roll.
pub fn lift(height: f32, held: f32, lowest: f32) -> f32 {
    (height - lowest) * held
}

/// How far the renderer raises a drawn body for its roll this frame, given the
/// pose it is drawing on its own skeleton. See [`lift`].
pub fn drawn_lift(p: &crate::PlayerView, skin: &Skin, skeleton: &Skeleton) -> f32 {
    if p.roll_held <= 0.0 {
        return 0.0;
    }
    lift(
        p.roll_height,
        p.roll_held,
        lowest(skin, skeleton, p.roll, p.roll_about),
    )
}
