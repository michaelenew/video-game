//! **Can this class clear that hop, and with how much to spare?** The jump
//! courses' instrument (`docs/design/courses.md` §4): a pilot that plays one
//! hop of a course -- from standing on one island to standing on the next --
//! with one technique, in the course's own arena, and the scans round it that
//! turn pass-or-fail into a **margin**.
//!
//! Two margins, because a hop can be lost two ways:
//!
//! - **Timing**: how many frames of takeoff work. The pilot presses space
//!   `lead` frames before the last frame her feet are on the island, for every
//!   lead from 0 to 15; the window is how many of those land. For the
//!   airdodge, the window is how many of the frames it could be thrown on land
//!   (at the best lead). A window of one is a frame-perfect hop; fifteen is a
//!   hop that does not care.
//! - **Distance**: how much wider the gap could be, at the same rise, and
//!   still be landed -- the running jump off the lab's runway replayed against
//!   a ledge at the hop's rise (`envelope::lands`). Plain and with the
//!   airdodge. Not asked of the tools, whose reach is the envelope's tool
//!   table.
//!
//! Then [`run`] chains the hops: the whole course, each hop with the first
//! technique that clears it, in the order a player would reach for them --
//! the plain jump, the airdodge, the class's own tool. What it reports is the
//! measured matrix: which classes finish, and on what.
//!
//! It builds worlds and clones them, so it allocates; a tool, not a frame.
//! Integer arithmetic only, like the rest of the crate.

use crate::arena::Solid;
use crate::class::{Class, Ghost};
use crate::course::Course;
use crate::envelope::{look_at, tick};
use crate::fixed::Fx;
use crate::input::Input;
use crate::math::{self, V3};
use crate::state::{Action, World};

/// How a hop is attempted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Technique {
    /// A running jump, space held while rising, space pressed `lead` frames
    /// before the last frame on the island.
    Jump { lead: u32 },
    /// The same, and the airdodge thrown toward the next island `at` frames
    /// after takeoff.
    Airdodge { lead: u32, at: u32 },
    /// The Reaver: the shadow sent onto the next island and the dash to it.
    /// `throw` is 0 from the floor, or the frame of a hop it is thrown from.
    Shadow { throw: u32 },
    /// The Reaver: the shadow onto the next island, the dash, and the dash
    /// jump off it to the island after -- two hops in one.
    ShadowJump,
    /// The Elementalist: `count` stones under her own feet at the edge, two
    /// frames apart, space from `jump_at`.
    Stones { count: u32, jump_at: u32 },
    /// The Dual mage: autos alternated on the spot until her lower bar is at
    /// the second tier (no creature to hit -- the time counts), then a
    /// running jump and the second jump `at` frames after takeoff.
    SecondJump { at: u32 },
    /// The Champion: a running jump with a weapon pressed as the feet leave,
    /// 0 sword, 1 hammer, 2 spear.
    Takeoff { weapon: u32 },
    /// The Dual mage: autos alternated on the spot until both bars are at the
    /// top and she ascends (the time counts), then space every `beat` frames,
    /// steered at the next island -- the wing beat.
    Wings { beat: u32 },
}

impl Technique {
    pub fn name(self) -> String {
        match self {
            Technique::Jump { lead } => format!("jump (-{lead}f)"),
            Technique::Airdodge { lead, at } => format!("jump (-{lead}f) + airdodge @{at}f"),
            Technique::Shadow { throw: 0 } => "shadow + dash".to_string(),
            Technique::Shadow { throw } => format!("hop, shadow @{throw}f, airdash"),
            Technique::ShadowJump => "shadow, dash, dash jump".to_string(),
            Technique::Stones { count: 1, jump_at } => format!("stone jump @{jump_at}f"),
            Technique::Stones { jump_at, .. } => format!("double stone jump @{jump_at}f"),
            Technique::SecondJump { at } => format!("tier, second jump @{at}f"),
            Technique::Takeoff { weapon } => {
                format!(
                    "takeoff ({})",
                    ["sword", "hammer", "spear"][weapon as usize % 3]
                )
            }
            Technique::Wings { beat } => format!("ascend, a beat every {beat}f"),
        }
    }

    /// The short family name, for a table.
    pub fn family(self) -> &'static str {
        match self {
            Technique::Jump { .. } => "jump",
            Technique::Airdodge { .. } => "airdodge",
            Technique::Shadow { .. } => "shadow",
            Technique::ShadowJump => "dash jump",
            Technique::Stones { count: 1, .. } => "stone",
            Technique::Stones { .. } => "2 stones",
            Technique::SecondJump { .. } => "2nd jump",
            Technique::Takeoff { .. } => "takeoff",
            Technique::Wings { .. } => "wings",
        }
    }

    /// How many hops of the route it covers.
    pub fn hops(self) -> usize {
        match self {
            Technique::ShadowJump => 2,
            _ => 1,
        }
    }
}

/// A fresh run of `course` as `class`, the other fighter an idle dummy on
/// the start beside her.
pub fn start(course: &Course, class: Class) -> World {
    let mut w = World::versus_in([class, Class::Champion], course.arena);
    w.players[1].health = i32::MAX / 2;
    w
}

/// The flat middle of a top.
fn middle(s: &Solid) -> V3 {
    let half = |a: Fx, b: Fx| Fx::from_raw(a.raw() + (b.raw() - a.raw()) / 2);
    V3::new(half(s.min.x, s.max.x), s.max.y, half(s.min.z, s.max.z))
}

/// Standing on this top: feet on it, over it, still.
fn on(w: &World, s: &Solid) -> bool {
    let p = &w.players[0];
    p.grounded
        && s.over(p.pos.x, p.pos.z, Fx::ZERO)
        && p.pos.y.sub(s.max.y).abs().raw() <= crate::arena::SKIN.raw()
}

/// Has the course stood her back on a gate -- a fall?
fn fell(before: &World, w: &World) -> bool {
    w.course[0].falls > before.course[0].falls
}

/// The yaw from her to a point, in the wire's unit.
fn yaw_to(w: &World, at: V3) -> u16 {
    let d = at.sub(w.players[0].pos);
    (math::atan2_turns(d.z, d.x).raw() & 0xFFFF) as u16
}

/// Would one more frame at her stride take her feet off `s`? The last frame
/// a running jump can be taken from.
fn leaving(w: &World, s: &Solid) -> bool {
    let p = &w.players[0];
    let next = p.pos.add(p.vel.scale(crate::DT));
    !s.over(next.x, next.z, crate::tuning::body_radius())
}

/// The frames a hop gets before it is called a miss: long enough for the
/// Dual mage to box her way to the top of both bars first.
const fn patience() -> u32 {
    2400
}

/// **One hop**, from `w` (standing on step `from`) with `how`. The world
/// standing on the step it was aiming for, and the frames it took, or `None`.
pub fn hop(w: &World, course: &Course, from: usize, how: Technique) -> Option<(World, u32)> {
    let here = course.top(from);
    let goal = from + how.hops();
    if goal >= course.route.len() {
        return None;
    }
    let next = course.top(from + 1);
    let there = course.top(goal);
    let aim = middle(&there);
    let mut w = w.clone();
    let before = w.clone();
    let mut left: Option<u32> = None;
    let mut pressed: Option<u32> = None;
    let mut sent = false;
    let mut dashed = false;
    let mut boxing_left = true;
    let mut lead_frames: Vec<World> = Vec::new();
    for f in 0..patience() {
        let p = w.players[0];
        let airborne = !p.grounded;
        if left.is_none() && airborne && pressed.is_some() {
            left = Some(f);
        }
        let after = left.map_or(0, |l| f - l);
        let toward_goal = yaw_to(&w, aim);
        // The tools are thrown from the edge facing the next island, a step
        // inside it: walk there first.
        let from_the_edge = matches!(
            how,
            Technique::Shadow { .. } | Technique::ShadowJump | Technique::Stones { .. }
        );
        if from_the_edge && pressed.is_none() {
            // A stone wants its own width on the island; a shadow only her feet.
            let inset = match how {
                Technique::Stones { .. } => {
                    crate::tuning::structure_radius().add(crate::tuning::body_radius())
                }
                _ => crate::tuning::body_radius(),
            };
            let ahead = p
                .pos
                .add(V3::from_turns(Fx::from_raw(toward_goal as i32)).scale(inset));
            if here.over(ahead.x, ahead.z, Fx::ZERO) {
                tick(&mut w, Input::aimed(Input::W, toward_goal));
                continue;
            }
            pressed = Some(f);
        }
        let mut bits = 0u16;
        let mut look = Input::aimed(0, toward_goal);
        match how {
            Technique::Jump { lead } | Technique::Airdodge { lead, .. } => {
                // Remember the last frames before the edge, so the lead can
                // reach back into them.
                if pressed.is_none() {
                    lead_frames.push(w.clone());
                    if leaving(&w, &here) {
                        let back = lead_frames.len().saturating_sub(1 + lead as usize);
                        w = lead_frames[back].clone();
                        pressed = Some(f);
                    }
                    if pressed.is_none() {
                        tick(&mut w, Input::aimed(Input::W, toward_goal));
                        continue;
                    }
                }
                bits |= Input::W;
                let p = w.players[0];
                if p.grounded || p.vel.y.raw() > 0 {
                    bits |= Input::SPACE;
                }
                if let Technique::Airdodge { at, .. } = how {
                    if left.is_some() && after == at {
                        bits |= Input::SHIFT;
                    }
                }
                look = Input::aimed(bits, toward_goal);
                if left.is_some() {
                    look = homing(&w, &there, bits & !Input::W);
                }
            }
            Technique::Takeoff { weapon } => {
                if pressed.is_none() && leaving(&w, &here) {
                    pressed = Some(f);
                }
                bits |= Input::W;
                if pressed.is_some() && (p.grounded || p.vel.y.raw() > 0) {
                    bits |= Input::SPACE;
                }
                if pressed.is_some_and(|at| f == at + 1 || f == at + 2) {
                    bits |= [Input::LEFT, Input::MIDDLE, Input::RIGHT][weapon as usize % 3];
                }
                look = if left.is_some() {
                    homing(&w, &there, bits & !Input::W)
                } else {
                    Input::aimed(bits, toward_goal)
                };
            }
            Technique::SecondJump { at } => {
                // Earn the tier first, standing still on this island.
                if pressed.is_none() && crate::dual::tier(&p) < crate::dual::Tier::Jump {
                    if p.action.actionable() {
                        bits = if boxing_left {
                            Input::LEFT
                        } else {
                            Input::RIGHT
                        };
                        boxing_left = !boxing_left;
                    }
                    tick(&mut w, Input::aimed(bits, toward_goal));
                    continue;
                }
                if pressed.is_none() && leaving(&w, &here) {
                    pressed = Some(f);
                }
                bits |= Input::W;
                let sustain = crate::tuning::jump_hold_frames() as u32;
                if pressed.is_some()
                    && (left.is_none() || after <= sustain || (after >= at && after < at + sustain))
                {
                    bits |= Input::SPACE;
                }
                look = if left.is_some() {
                    homing(&w, &there, bits & !Input::W)
                } else {
                    Input::aimed(bits, toward_goal)
                };
            }
            Technique::Shadow { throw } => {
                let target = middle(&next);
                let f = f - pressed.unwrap_or(f);
                if throw > 0 && f < throw + 2 {
                    bits |= Input::SPACE;
                }
                let shadow = crate::shadow::of(&p);
                if !sent && f >= throw {
                    if f > throw {
                        sent = true;
                    }
                    look = with_look(&w, bits | Input::RIGHT, target);
                } else if let Some(s) = shadow.filter(|s| matches!(s.doing, Ghost::Waiting)) {
                    if !dashed {
                        dashed |= matches!(p.action, Action::Dodge { .. }) && f > throw + 2;
                        look = with_look(&w, bits | Input::SHIFT | Input::W, s.pos);
                    }
                } else {
                    look = Input::aimed(bits, toward_goal);
                }
            }
            Technique::ShadowJump => {
                let stone = middle(&next);
                let f = f - pressed.unwrap_or(f);
                let shadow = crate::shadow::of(&p);
                if !sent {
                    sent = f > 0;
                    look = with_look(&w, Input::RIGHT, stone);
                } else if !dashed {
                    if let Some(s) = shadow.filter(|s| matches!(s.doing, Ghost::Waiting)) {
                        look = with_look(&w, Input::SHIFT | Input::W, s.pos);
                    }
                    dashed = matches!(p.action, Action::Dodge { .. });
                } else if crate::shadow::carrying_a_dash(&p) {
                    look = Input::aimed(Input::SPACE | Input::W, yaw_to(&w, aim));
                } else {
                    let p = w.players[0];
                    let space = if p.vel.y.raw() > 0 { Input::SPACE } else { 0 };
                    look = homing(&w, &there, space);
                }
            }
            Technique::Wings { beat } => {
                if pressed.is_none() && !crate::dual::ascending(&p) {
                    if p.action.actionable() {
                        bits = if boxing_left {
                            Input::LEFT
                        } else {
                            Input::RIGHT
                        };
                        boxing_left = !boxing_left;
                    }
                    tick(&mut w, Input::aimed(bits, toward_goal));
                    continue;
                }
                let start = *pressed.get_or_insert(f);
                if (f - start) % beat < 2 {
                    bits |= Input::SPACE;
                }
                // Up first, then across once she is above the top.
                look = if p.pos.y.raw() > there.max.y.add(Fx::ONE).raw() {
                    homing(&w, &there, bits)
                } else {
                    Input::aimed(bits, toward_goal)
                };
            }
            Technique::Stones { count, jump_at } => {
                let k = f - pressed.unwrap_or(f);
                let feet = V3::new(p.pos.x, here.max.y, p.pos.z);
                if k < count * 2 && k % 2 == 0 {
                    bits |= Input::MECHANIC;
                }
                if k >= jump_at {
                    bits |= Input::SPACE;
                }
                if bits & Input::MECHANIC != 0 {
                    look = with_look(&w, bits, feet);
                } else if p.pos.y.raw() > there.max.y.add(Fx::ONE).raw() {
                    look = homing(&w, &there, bits);
                } else {
                    look = Input::aimed(bits, toward_goal);
                }
            }
        }
        tick(&mut w, look);
        if fell(&before, &w) {
            return None;
        }
        if on(&w, &there) {
            return Some((w, f + 1));
        }
        // Down on the island she left, or on the one she skipped, after
        // leaving: a miss of its own kind -- the course does not count it as a
        // fall, but the hop did not happen.
        if left.is_some() && after > 10 && w.players[0].grounded && !on(&w, &next) {
            return None;
        }
    }
    None
}

/// Air control toward a top: the stick pointed at its middle, so the
/// strafe carves the velocity round to it -- the Quake way, which is how a
/// player lands on a stone a metre across. Nothing held once over it.
fn homing(w: &World, there: &Solid, bits: u16) -> Input {
    let p = &w.players[0];
    let mid = middle(there);
    let close = mid.sub(p.pos).flat_len().raw() < crate::tuning::body_radius().raw();
    let stick = if close { 0 } else { Input::W };
    Input::aimed(bits | stick, yaw_to(w, mid))
}

/// Buttons, the crosshair on a point.
fn with_look(w: &World, bits: u16, at: V3) -> Input {
    let (yaw, pitch) = look_at(w, at);
    Input::looking_at(bits, yaw, pitch)
}

/// Every technique a class has for a hop, in the order a player reaches for
/// them: the jump, the airdodge, the class's own.
pub fn techniques(class: Class) -> Vec<Technique> {
    let mut out: Vec<Technique> = (0..16).map(|lead| Technique::Jump { lead }).collect();
    for lead in 0..4 {
        out.extend((1..60).map(|at| Technique::Airdodge { lead, at }));
    }
    match class {
        Class::ShadowReaver => {
            out.push(Technique::Shadow { throw: 0 });
            out.extend([10, 20, 30].map(|throw| Technique::Shadow { throw }));
            out.push(Technique::ShadowJump);
        }
        Class::Elementalist => {
            out.extend((0..25).map(|jump_at| Technique::Stones { count: 1, jump_at }));
            out.extend((0..25).map(|jump_at| Technique::Stones { count: 2, jump_at }));
        }
        Class::DualMage => {
            out.extend((19..64).step_by(3).map(|at| Technique::SecondJump { at }));
            out.extend([12, 20].map(|beat| Technique::Wings { beat }));
        }
        Class::Champion => out.extend((0..3).map(|weapon| Technique::Takeoff { weapon })),
        _ => {}
    }
    out
}

/// What one class does with one hop: every technique that clears it.
#[derive(Clone, Debug, Default)]
pub struct Verdict {
    /// Takeoff frames, of sixteen, a plain running jump lands from.
    pub jump_window: u32,
    /// Airdodge frames that land, at the best lead.
    pub dodge_window: u32,
    /// Every other technique that cleared it.
    pub tools: Vec<Technique>,
}

impl Verdict {
    pub fn cleared(&self) -> bool {
        self.jump_window > 0 || self.dodge_window > 0 || !self.tools.is_empty()
    }
}

/// Every technique `class` has, tried on hop `from` -> `from + 1` from `w`.
pub fn judge(w: &World, course: &Course, from: usize, class: Class) -> Verdict {
    let mut v = Verdict::default();
    let mut dodges = [0u32; 4];
    for how in techniques(class) {
        if how.hops() > 1 {
            continue;
        }
        if hop(w, course, from, how).is_none() {
            continue;
        }
        match how {
            Technique::Jump { .. } => v.jump_window += 1,
            Technique::Airdodge { lead, .. } => dodges[lead as usize] += 1,
            other => v.tools.push(other),
        }
    }
    v.dodge_window = dodges.into_iter().max().unwrap_or(0);
    v
}

/// The whole course, each hop with the first technique that clears it --
/// the plain jump first, the class's tools last. The hops cleared, how, and
/// the frames it took; it stops at the first hop nothing clears.
///
/// **One step of looking back**: a hop nothing clears is tried again from
/// the island before, with the techniques that cover two hops at once (the
/// Reaver's dash jump off a stone she sent her shadow to), since the cheap
/// way onto the stone may be the wrong way onto the stone.
pub fn run(course: &Course, class: Class) -> (Vec<(usize, Technique)>, u32, bool) {
    let mut w = start(course, class);
    let mut from = 0;
    let mut done: Vec<(usize, Technique)> = Vec::new();
    let mut frames = 0;
    let mut behind: Option<(World, u32)> = None;
    let tries = techniques(class);
    while from + 1 < course.route.len() {
        let first = tries
            .iter()
            .copied()
            .filter(|t| t.hops() == 1)
            .find_map(|how| hop(&w, course, from, how).map(|r| (how, r)));
        if let Some((how, (next, took))) = first {
            behind = Some((w.clone(), frames));
            done.push((from, how));
            w = next;
            frames += took;
            from += 1;
            continue;
        }
        // Back one island, two hops at once.
        let Some((before, at)) = behind.take() else {
            return (done, frames, false);
        };
        let back = from - 1;
        let pair = tries
            .iter()
            .copied()
            .filter(|t| t.hops() == 2)
            .find_map(|how| hop(&before, course, back, how).map(|r| (how, r)));
        let Some((how, (next, took))) = pair else {
            return (done, frames, false);
        };
        done.pop();
        done.push((back, how));
        w = next;
        frames = at + took;
        from = back + 2;
    }
    (done, frames, true)
}

/// The world standing on step `step` of a course, from a fresh start: the
/// route played up to there with [`run`]'s choices, or stood there directly
/// when the class cannot get there itself -- so every hop can be judged for
/// every class, including the ones after a hop it fails.
pub fn standing_on(course: &Course, class: Class, step: usize) -> World {
    let mut w = start(course, class);
    let top = course.top(step);
    let p = &mut w.players[0];
    p.pos = middle(&top);
    p.vel = V3::ZERO;
    p.grounded = true;
    if let crate::class::Mechanic::Shadow(_) = p.mechanic {
        p.mechanic =
            crate::class::Mechanic::Shadow(crate::class::Shadow::attending(p.pos, p.facing));
    }
    // A frame to settle onto it.
    tick(&mut w, Input::default());
    w
}
