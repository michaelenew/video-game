//! The hit volume on every frame of an attack clip, in the body's own space
//! -- and the hands put on it.
//!
//! An attack clip used to be authored *against* the hit volume: the author
//! read the move's plane and reach, typed where the hands ought to be on the
//! contact frame and the last live frame, and hoped. The audit
//! (`cargo run -p anim --bin audit`) measured how that went: on the sword's
//! active frames the nearer hand was up to a quarter of a metre off the
//! volume's axis and the line through the hands turned forty degrees from
//! it, so the renderer dragged the blade out of the fists to put it where
//! the hit test was (`view::arms`), which is what read as rough.
//!
//! So the hands are no longer typed. [`lines`] throws the move in a real
//! simulation and reads the volume the hit test has out on each frame
//! (`sim::state::hitbox`, the one description of an attack's shape), and
//! [`apply`] puts the hands on it at bake time: both hands along a swing's
//! axis a grip apart, the leading hand along a thrust's. A recipe authors
//! the body -- the hips, the step, the turn of the chest, the follow-through
//! -- and the weapon goes where the weapon goes. Change a move's arc in the
//! table and the clip's arms change with it, and nothing has to be retyped.
//!
//! Only moves with a line do this: a swing or a thrust. A disc at arm's
//! length, a beam, a wing and a thing planted on the floor have no axis for
//! two hands to lie along, and their clips stay as authored.

use sim::moves::Shape;
use sim::state::Action;
use view::clips::Clip;
use view::math::{self, V3};
use view::pose::Pose;

/// The hit volume on one frame, in the body's own space: `from` is the hub
/// end, `to` the tip. The same frame the clip is drawn in (`+Z` forward,
/// the body's right at `-X`, feet at the origin).
#[derive(Clone, Copy, Debug)]
pub struct Line {
    pub from: V3,
    pub to: V3,
    pub radius: f32,
}

impl Line {
    pub fn dir(&self) -> V3 {
        math::normalize_or(math::sub(self.to, self.from), [0.0, 0.0, 1.0])
    }

    pub fn reach(&self) -> f32 {
        math::length(math::sub(self.to, self.from))
    }

    /// A point `out` metres along the axis from the hub.
    pub fn along(&self, out: f32) -> V3 {
        math::add(self.from, math::scale(self.dir(), out))
    }
}

/// Does this clip's move put out a line the hands can lie along?
pub fn has_line(clip: Clip) -> bool {
    match clip.move_slot() {
        Some((class, slot)) => matches!(
            sim::moves::get(class, slot).shape,
            Shape::Swing(_) | Shape::Thrust
        ),
        None => false,
    }
}

/// The volume out on each frame of the clip, as the simulation plays the
/// move from a standing start, aimed level. `None` where nothing is out, for
/// a clip that is not a move, and for a move that could not be thrown.
pub fn lines(clip: Clip) -> Vec<Option<Line>> {
    let length = clip.length().max(1) as usize;
    let mut out = vec![None; length];
    let Some((class, slot)) = clip.move_slot() else {
        return out;
    };
    // From the ground first; a move the ground refuses is tried from the
    // air, which is where the Champion's aerials live.
    for aerial in [false, true] {
        if let Some(found) = throw(class, slot, aerial, length) {
            out = found;
            break;
        }
    }
    out
}

fn throw(class: sim::Class, slot: u8, aerial: bool, length: usize) -> Option<Vec<Option<Line>>> {
    let mut w = sim::World::with_classes([class, sim::Class::Bulwark]);
    // The other fighter well out of reach: this is geometry, not a fight.
    w.players[1].pos = sim::V3::new(sim::Fx::from_int(11), w.players[1].pos.y, sim::Fx::ZERO);
    // A chain link the Champion can only throw mid-chain is given the chain.
    if let sim::class::Mechanic::Forms {
        chain, chain_left, ..
    } = &mut w.players[0].mechanic
    {
        if let Some(link) = sim::moves::champion::link_of(slot) {
            *chain = link;
            *chain_left = 400;
        }
    }
    let look = sim::Input::looking_at(0, 0, 0);
    let idle = sim::Input::default();
    for _ in 0..2 {
        w.advance([look, idle]);
    }
    if aerial {
        w.advance([sim::Input::looking_at(sim::Input::SPACE, 0, 0), idle]);
        for _ in 0..10 {
            w.advance([look, idle]);
        }
        if w.players[0].grounded {
            return None;
        }
    }
    w.press(0, slot, look);
    let (startup, active, recovery) = sim::moves::frames(class, slot);
    let mut out = vec![None; length];
    let mut seen = false;
    for _ in 0..600 {
        let p = w.players[0];
        let elapsed = match p.action {
            Action::Channel { kind, .. } if kind == slot => 0,
            Action::Startup { kind, left } if kind == slot => startup.saturating_sub(left),
            Action::Active { kind, left } if kind == slot => startup + active.saturating_sub(left),
            Action::Recovery { kind, left } if kind == slot => {
                startup + active + recovery.saturating_sub(left)
            }
            _ => break,
        };
        seen = true;
        if let Some(h) = sim::state::hitbox(&p) {
            let line = Line {
                from: local(h.from, p.pos, p.facing),
                to: local(h.to, p.pos, p.facing),
                radius: h.radius.to_f32_for_render(),
            };
            let i = (elapsed as usize).min(length - 1);
            // The first volume of a frame: a move that re-hits has one.
            if out[i].is_none() {
                out[i] = Some(line);
            }
        }
        w.advance([look, idle]);
    }
    seen.then_some(out)
}

/// A point in the arena, in the body's own space: `view::into_world` run
/// backwards.
fn local(v: sim::V3, pos: sim::V3, facing: sim::V3) -> V3 {
    let d = [
        (v.x.sub(pos.x)).to_f32_for_render(),
        (v.y.sub(pos.y)).to_f32_for_render(),
        (v.z.sub(pos.z)).to_f32_for_render(),
    ];
    let yaw = facing
        .x
        .to_f32_for_render()
        .atan2(facing.z.to_f32_for_render());
    let (s, c) = (-yaw).sin_cos();
    [d[0] * c + d[2] * s, d[1], -d[0] * s + d[2] * c]
}

/// Half the distance between the two hands on a two-handed grip.
const GRIP: f32 = 0.11;
/// How far out from the hub the grip sits at the start and end of a swing,
/// and how much further it goes at the middle, where the arms are longest.
const OUT: f32 = 0.28;
const EXTEND: f32 = 0.14;
/// Where the leading hand sits on a thrust, and the other hand behind it.
const LEAD: f32 = 0.50;
const REAR: f32 = 0.24;
/// Frames either side of the live window over which the hands ease onto the
/// line and off it again.
const EASE: usize = 5;

/// How far a hand may travel between two frames and still read as motion
/// rather than a cut (`crates/anim/tests/clips.rs`, the motion ceiling).
const HAND_CEILING: f32 = 0.24;

/// How far out along the axis the grip can sit on a swing whose axis turns
/// by `turn` radians this frame, so the hands keep under the motion ceiling:
/// a fast arc is held close, a slow one at arm's length. A cut that turns a
/// hundred degrees in three frames is the move; the hands cannot be half a
/// metre out on it without moving faster than anything should.
fn out_for(t: f32, turn: f32) -> f32 {
    let wanted = OUT + EXTEND * (std::f32::consts::PI * t.clamp(0.0, 1.0)).sin();
    let allowed = if turn > 1e-3 {
        HAND_CEILING / turn - GRIP
    } else {
        f32::MAX
    };
    wanted.min(allowed).max(0.16)
}

/// Where the body's two hands go on a line. `None` leaves that hand as the
/// recipe authored it.
#[derive(Clone, Copy, Debug, Default)]
pub struct Grip {
    pub left: Option<V3>,
    pub right: Option<V3>,
}

/// The grip on the line for one frame; `t` is how far through the live
/// window the frame is, 0 to 1, and `turn` how far the axis turns between
/// this frame and its neighbours, in radians.
pub fn grip_on(clip: Clip, line: &Line, t: f32, turn: f32) -> Grip {
    let Some((class, slot)) = clip.move_slot() else {
        return Grip::default();
    };
    let m = sim::moves::get(class, slot);
    match m.shape {
        Shape::Swing(_) => {
            let out = out_for(t, turn);
            // The leading hand nearer the tip, as a two-handed grip is held.
            // Which hand leads follows the side the cut comes from: the plane's
            // hand for a diagonal, and for a flat sweep whichever side the
            // axis is out on (the body's right is at `-x` as drawn), so the
            // rear hand is never asked to cross the body to a point the arm
            // cannot reach.
            let right_leads = match m.shape {
                Shape::Swing(sim::moves::Plane::Diagonal(sim::aim::Hand::Left)) => false,
                Shape::Swing(sim::moves::Plane::Flat) => line.dir()[0] < 0.0,
                _ => true,
            };
            let (lead, rear) = (line.along(out + GRIP), line.along(out - GRIP));
            if right_leads {
                Grip {
                    left: Some(rear),
                    right: Some(lead),
                }
            } else {
                Grip {
                    left: Some(lead),
                    right: Some(rear),
                }
            }
        }
        Shape::Thrust => match m.hand {
            sim::aim::Hand::Right => Grip {
                left: None,
                right: Some(line.along(LEAD)),
            },
            sim::aim::Hand::Left => Grip {
                left: Some(line.along(LEAD)),
                right: None,
            },
            sim::aim::Hand::Centre => Grip {
                left: Some(line.along(REAR)),
                right: Some(line.along(LEAD)),
            },
        },
        _ => Grip::default(),
    }
}

/// The slot a drawn pose keeps one of the **body's** hands in, as the solver
/// names it: the body's right arm is in the slot named `l`
/// (`view::hand_joint`), because the recipes are authored left-handed and
/// reflected.
fn slot_is_left(body_left: bool) -> bool {
    !body_left
}

/// Put the hands on the line on every frame that has one, easing on over the
/// last frames before it and off over the first after.
///
/// `frames` are the baked, drawn frames of `clip`. Nothing happens for a
/// clip without a line. Frame by frame, in order, each hand solved toward
/// the elbow the frame before had (`view::ik::hand_to_near`), so the arm
/// never flips over between two frames whose hands are a few centimetres
/// apart.
pub fn apply(clip: Clip, frames: &mut [Pose]) {
    if !has_line(clip) || frames.is_empty() {
        return;
    }
    let lines = lines(clip);
    let live: Vec<usize> = (0..frames.len())
        .filter(|i| lines.get(*i).copied().flatten().is_some())
        .collect();
    let (Some(&first), Some(&last)) = (live.first(), live.last()) else {
        return;
    };
    let span = (last - first).max(1) as f32;
    // How far the axis turns at each live frame: the larger of the turns to
    // the frames either side.
    let turn_at = |i: usize| -> f32 {
        let here = lines[i].unwrap().dir();
        let mut turn = 0.0f32;
        for j in [i.wrapping_sub(1), i + 1] {
            if let Some(other) = lines.get(j).copied().flatten().map(|l| l.dir()) {
                turn = turn.max(math::dot(here, other).clamp(-1.0, 1.0).acos());
            }
        }
        turn
    };
    let grip_at = |i: usize| -> Grip {
        grip_on(
            clip,
            &lines[i].unwrap(),
            (i - first) as f32 / span,
            turn_at(i),
        )
    };
    let start = grip_at(first);
    let end = grip_at(last);
    // Eased, so the frame the hands start toward the line is not a lurch.
    let weight = |k: usize| {
        let u = 1.0 - k as f32 / (EASE as f32 + 1.0);
        u * u * (3.0 - 2.0 * u)
    };
    let skeleton = view::pose::reference();
    let lo = first.saturating_sub(EASE);
    let hi = (last + EASE).min(frames.len() - 1);
    let mut elbows = [
        view::ik::elbow_of(&frames[lo.saturating_sub(1)], skeleton, slot_is_left(true)),
        view::ik::elbow_of(&frames[lo.saturating_sub(1)], skeleton, slot_is_left(false)),
    ];
    for i in lo..=hi {
        let (grip, w) = if i < first {
            (start, weight(first - i))
        } else if i > last {
            (end, weight(i - last))
        } else if lines[i].is_some() {
            (grip_at(i), 1.0)
        } else {
            // A gap inside the window, for a move that re-hits: hold the
            // line's last grip.
            (
                grip_at(live.iter().rev().copied().find(|&l| l < i).unwrap_or(first)),
                1.0,
            )
        };
        let (was_l, was_r) = hands_of(&frames[i]);
        let mut pose = frames[i];
        for (body_left, want, was) in [(true, grip.left, was_l), (false, grip.right, was_r)] {
            let Some(want) = want else { continue };
            let target = math::add(was, math::scale(math::sub(want, was), w));
            let slot = slot_is_left(body_left);
            // The elbow to prefer: the frame before's while on the line, and
            // across the ease the recipe's own, by the same weight -- so the
            // arm hands over to the authored motion at the far end of the
            // ease as smoothly as it joined the line at the near end.
            let own = view::ik::elbow_of(&frames[i], skeleton, slot);
            let carried = elbows[body_left as usize];
            let hint = math::normalize_or(
                math::add(math::scale(own, 1.0 - w), math::scale(carried, w)),
                carried,
            );
            view::ik::hand_to_near(&mut pose, skeleton, slot, target, hint);
            elbows[body_left as usize] = view::ik::elbow_of(&pose, skeleton, slot);
        }
        frames[i] = pose;
    }
}

/// Where a drawn pose has the body's left and right wrists.
fn hands_of(pose: &Pose) -> (V3, V3) {
    let skin = view::skeleton::solve(view::pose::reference(), pose);
    (
        skin.origin[view::hand_joint(true).index()],
        skin.origin[view::hand_joint(false).index()],
    )
}
