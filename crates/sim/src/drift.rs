//! **Drifts: boxes that move** (2026-10-22). A lift up the Needle, a pair of
//! walls rising and sinking out of step, a stone that runs a long stretch
//! fast. The owner: "short to medium length, interesting sections that give
//! some variety, then a moving platform, then another section" -- and moving
//! stones in the challenges themselves: a jump timed onto a fast one at an
//! angle, a jump off one whose momentum carries you, a pair of moving walls
//! to kick between going up.
//!
//! **A drift is where the frame says it is.** It goes back and forth between
//! two stops on a fixed clock -- a wait at the first, the way there eased in
//! and out ([`crate::math::smoothstep`]), a stay at the second, the way back
//! -- so where it is on any frame is a function of the frame number and
//! nothing else. Nothing about it is in the snapshot: a rollback that
//! re-simulates a frame puts every drift exactly where it was, and both
//! machines agree because they agree on the frame. Since the stops are
//! fixed, two drifts on one path half a cycle apart pass each other as a
//! pair of lifts do, one up while the other comes down.
//!
//! **A drift is a solid like any other** while anybody is near it: the world
//! raises the ones within [`near_reach`] of a fighter into its terrain each frame
//! (`World::terrain`), so it stops bodies, holds them up, is a wall and a
//! ledge, and meets the aiming ray, exactly as a box of the table does.
//!
//! **It carries what is on it** ([`attached`], `World::carry`): a body
//! standing on its top, hanging from its edge or against its side is moved
//! with it, frame for frame, before the body takes its own step -- so
//! walking on a moving stone is walking, and a wall slide down a rising wall
//! is a slide down the wall, not down the world. **And a body leaving it
//! keeps its speed**: the frame it is no longer on it, the drift's velocity
//! is added to the body's own. That is what makes a jump off a fast stone
//! carry, and a jump off a rising lift go higher -- the timing is when to
//! leave.
//!
//! Where drifts are is level data: the Needle's and the High Plane's tables
//! (`arena::spire::drifts`, `arena::heights::drifts`), put on the map with
//! everything else ([`crate::atlas::Atlas::drifts`]).

use crate::arena::{SKIN, Solid};
use crate::fixed::Fx;
use crate::math::V3;
use crate::state::Player;
use crate::tuning as t;

/// **A box that moves** between two stops on a fixed clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Drift {
    /// The box at its first stop.
    pub solid: Solid,
    /// From its first stop to its second.
    pub travel: V3,
    /// Frames it waits at its first stop, the way there, frames it stays at
    /// its second, and the way back.
    pub wait: u16,
    pub go: u16,
    pub stay: u16,
    pub back: u16,
    /// How far into its cycle it is at frame zero, in frames.
    pub phase: u16,
}

/// How far from a fighter, sideways, a drift is raised into the terrain:
/// five of the map's tiles, further than a jump or a breath of running and
/// nearer than the far end of the longest run.
pub fn near_reach() -> Fx {
    Fx::from_int(crate::atlas::TILE_M * 5)
}

impl Drift {
    /// Frames in one cycle: wait, there, stay, back.
    pub const fn period(&self) -> u32 {
        self.wait as u32 + self.go as u32 + self.stay as u32 + self.back as u32
    }

    /// **How far along its travel it is** on a frame: nought at its first
    /// stop, one at its second, eased between.
    pub fn share(&self, frame: u32) -> Fx {
        let period = self.period().max(1);
        let k = (frame.wrapping_add(self.phase as u32)) % period;
        let (wait, go, stay, back) = (
            self.wait as u32,
            self.go as u32,
            self.stay as u32,
            self.back as u32,
        );
        if k < wait {
            Fx::ZERO
        } else if k < wait + go {
            crate::math::smoothstep(Fx::ratio((k - wait) as i32, go.max(1) as i32))
        } else if k < wait + go + stay {
            Fx::ONE
        } else {
            let into = k - wait - go - stay;
            Fx::ONE.sub(crate::math::smoothstep(Fx::ratio(
                into as i32,
                back.max(1) as i32,
            )))
        }
    }

    /// How far it is from its first stop on a frame.
    pub fn offset(&self, frame: u32) -> V3 {
        self.travel.scale(self.share(frame))
    }

    /// The box on a frame.
    pub fn at(&self, frame: u32) -> Solid {
        moved(self.solid, self.offset(frame))
    }

    /// How far it moved coming into a frame, from the one before.
    pub fn step(&self, frame: u32) -> V3 {
        self.offset(frame).sub(self.offset(frame.wrapping_sub(1)))
    }

    /// **Its velocity** coming into a frame, per second: what a body
    /// leaving it keeps.
    pub fn velocity(&self, frame: u32) -> V3 {
        self.step(frame).scale(Fx::from_int(crate::TICK_HZ as i32))
    }

    /// The same drift with its box moved by `d`: into another frame of
    /// coordinates.
    pub fn moved(&self, d: V3) -> Drift {
        Drift {
            solid: moved(self.solid, d),
            ..*self
        }
    }

    /// The box its whole travel sweeps: every place it ever is.
    pub fn swept(&self) -> Solid {
        let end = moved(self.solid, self.travel);
        Solid {
            min: V3::new(
                self.solid.min.x.min(end.min.x),
                self.solid.min.y.min(end.min.y),
                self.solid.min.z.min(end.min.z),
            ),
            max: V3::new(
                self.solid.max.x.max(end.max.x),
                self.solid.max.y.max(end.max.y),
                self.solid.max.z.max(end.max.z),
            ),
            ..self.solid
        }
    }

    /// Is any of its sweep within `reach` of `at`, sideways?
    pub fn near(&self, at: V3, reach: Fx) -> bool {
        let s = self.swept();
        at.x.raw() > s.min.x.sub(reach).raw()
            && at.x.raw() < s.max.x.add(reach).raw()
            && at.z.raw() > s.min.z.sub(reach).raw()
            && at.z.raw() < s.max.z.add(reach).raw()
    }
}

/// A box moved by `d`.
fn moved(s: Solid, d: V3) -> Solid {
    Solid {
        min: s.min.add(d),
        max: s.max.add(d),
        ..s
    }
}

/// **How a body holds on to a box**, if it does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hold {
    /// Standing on its top.
    On,
    /// Hanging from an edge of its top.
    Hanging,
    /// Against its side: sliding down it, or just touched.
    Against,
}

/// **Is this body holding on to `s`** -- on it, hanging from it or against
/// it, as its last resolve left it? A body on a creature, or flying free,
/// holds nothing.
pub fn attached(p: &Player, s: &Solid) -> Option<Hold> {
    if p.aboard() || p.health <= 0 {
        return None;
    }
    let r = t::body_radius();
    let skin = SKIN.add(SKIN);
    if p.ledge.is_some() {
        let e = p.ledge.edge.at;
        let on_rim = e.y.sub(s.max.y).abs().raw() <= skin.raw() && s.over(e.x, e.z, skin);
        return on_rim.then_some(Hold::Hanging);
    }
    if p.grounded {
        let feet = p.pos.y.sub(s.max.y);
        let level = feet.raw() >= skin.neg().raw() && feet.raw() <= skin.add(skin).raw();
        return (level && s.over(p.pos.x, p.pos.z, r)).then_some(Hold::On);
    }
    // Touching a wall on the last resolve, and that wall this box's side:
    // beside it, within a body's radius and a little, and level with it.
    if p.wall_since == 0 {
        let beside = s.over(p.pos.x, p.pos.z, r.add(skin)) && !s.over(p.pos.x, p.pos.z, Fx::ZERO);
        let level =
            p.pos.y.raw() < s.max.y.raw() && p.pos.y.add(t::body_height()).raw() > s.min.y.raw();
        return (beside && level).then_some(Hold::Against);
    }
    None
}

/// **Move a body with the box it holds**, by `d`: its feet, the edge it
/// hangs from, the wall it is against, and the height its fall is measured
/// from.
pub fn carry(p: &mut Player, d: V3) {
    p.pos = p.pos.add(d);
    if p.ledge.is_some() {
        p.ledge.edge.at = p.ledge.edge.at.add(d);
        p.ledge.from = p.ledge.from.add(d);
    }
    if p.wall.is_some() {
        p.wall.at = p.wall.at.add(d);
    }
    p.fall_over = p.fall_over.add(d.y);
}

// ---------------------------------------------------------------------------
// In the world
// ---------------------------------------------------------------------------

use crate::arena::Terrain;
use crate::state::{MAX_PLAYERS, World};

/// What carrying did to each body this frame: the drift it held, by its
/// index in [`World::drifts`], and that drift's velocity -- what the body
/// keeps if it lets go.
pub type Carried = [Option<(usize, V3)>; MAX_PLAYERS];

impl World {
    /// **The drifts of the place the world is in**, in its coordinates: on
    /// the valley's map, every one on the map; anywhere else, none.
    pub fn drifts(&self) -> impl Iterator<Item = Drift> + '_ {
        let (list, o): (&[Drift], V3) = if self.valley.on {
            (&crate::atlas::valley().drifts, self.map_origin())
        } else {
            (&[], V3::ZERO)
        };
        list.iter().map(move |d| d.moved(V3::ZERO.sub(o)))
    }

    /// The seats that are played, by index.
    fn drift_seats(&self) -> core::ops::Range<usize> {
        0..(self.seats as usize).clamp(1, MAX_PLAYERS)
    }

    /// **Raise the drifts near anybody** on to the ground, where they stand
    /// this frame.
    pub(crate) fn raise_drifts(&self, ground: &mut Terrain) {
        let reach = near_reach();
        for d in self.drifts() {
            if self
                .drift_seats()
                .any(|i| d.near(self.players[i].pos, reach))
            {
                let on_its_way = d.step(self.frame) != V3::ZERO;
                ground.raise_moving(d.at(self.frame), on_its_way);
            }
        }
    }

    /// **Move every body holding on to a drift with it**, from where it was
    /// last frame to where it is this one: before anybody steps, so a body's
    /// own step starts from where the drift took it.
    pub(crate) fn carry_riders(&mut self) -> Carried {
        let mut carried: Carried = [None; MAX_PLAYERS];
        let frame = self.frame;
        let reach = near_reach();
        let seats = self.drift_seats();
        let (list, o): (&[Drift], V3) = if self.valley.on {
            (&crate::atlas::valley().drifts, self.map_origin())
        } else {
            (&[], V3::ZERO)
        };
        for (k, d) in list.iter().map(|d| d.moved(V3::ZERO.sub(o))).enumerate() {
            let was = d.at(frame.wrapping_sub(1));
            let step = d.step(frame);
            for i in seats.clone() {
                let p = &mut self.players[i];
                if carried[i].is_some() || !d.near(p.pos, reach) {
                    continue;
                }
                if attached(p, &was).is_some() {
                    carry(p, step);
                    carried[i] = Some((k, d.velocity(frame)));
                }
            }
        }
        carried
    }

    /// **A body that let go keeps the drift's speed**: after the step, any
    /// body carried this frame that no longer holds on to the drift that
    /// carried it -- jumped, walked off, kicked away -- has its velocity
    /// added to its own.
    ///
    /// **And a body that has just come against the side of one takes its
    /// rise off**: from here on the drift carries it, so its own speed up or
    /// down is against the wall, as a slide's is. Without it a kick between
    /// two rising walls would keep their speed once for every kick. (On to a
    /// top, the landing takes the fall away; a hang holds the body still.)
    pub(crate) fn let_riders_go(&mut self, carried: Carried) {
        let frame = self.frame;
        let list: &[Drift] = if self.valley.on {
            &crate::atlas::valley().drifts
        } else {
            &[]
        };
        let o = V3::ZERO.sub(self.map_origin());
        let seats = self.drift_seats();
        for i in seats {
            let held = carried[i];
            if let Some((k, v)) = held
                && let Some(d) = list.get(k)
            {
                let p = &mut self.players[i];
                let now = d.moved(o).at(frame);
                if attached(p, &now).is_none() && !p.grounded && !p.aboard() {
                    p.vel = p.vel.add(v);
                }
            }
            let reach = near_reach();
            for (k, d) in list.iter().enumerate() {
                let d = d.moved(o);
                let p = &mut self.players[i];
                if held.is_some_and(|(c, _)| c == k) || !d.near(p.pos, reach) {
                    continue;
                }
                if attached(p, &d.at(frame)) == Some(Hold::Against) {
                    p.vel.y = p.vel.y.sub(d.velocity(frame).y);
                    break;
                }
            }
        }
    }
}
