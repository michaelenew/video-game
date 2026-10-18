//! A creature stepped through time: where it stands, which way it faces,
//! what it is doing, and its body this frame.
//!
//! The motion modules say what a body looks like at a moment; this is the
//! part that **moves it through the world**. A walk carries it at its
//! gait's speed and turns it no faster than its size allows -- a giant
//! turns slowly, which is what makes getting round to its side an answer.
//! A move plays its frames from where and how the actor stood when it
//! started; when it ends, the actor is **where the move carried it**,
//! facing the way it ended, so a charge leaves it forty metres on and a
//! tail sweep turned round.
//!
//! The body is **cached**: it is rebuilt only when the drive changes, so a
//! creature standing still or holding a pose costs nothing to draw again.
//! Nothing here allocates: an actor is a value, its kit an array.

use super::build::{self, Body, MoveKind, MoveSpec, proportions};
use super::drive::Drive;
use super::locomotion::{self, FPS, Gait};
use super::posture::{self, Posture};
use super::strike::{self, Hit, Motion};
use super::{Genome, fx};
use crate::fixed::{Fx, cos_turns, sin_turns};
use crate::math::V3;

/// What the actor is doing.
#[derive(Clone, Copy, Debug)]
pub enum Act {
    Stand,
    /// Moving at a gait, turning by a share of its turn rate (`-1` to `1`,
    /// toward `+z` positive).
    Go {
        gait: Gait,
        phase: Fx,
        turn: Fx,
    },
    /// Under the floor, travelling the way it faces and turning by a share
    /// of its rate, for `frames` so far. It cannot be hit; the ground
    /// churns over it; it has to come up within
    /// [`crate::tuning::breed_under_most`].
    Under {
        frames: i32,
        turn: Fx,
    },
    /// Playing a move of its kit, turned to where it was aimed: `bearing`
    /// turns off its facing, and `distance` metres ahead for a throw.
    Strike {
        index: usize,
        frame: i32,
        bearing: Fx,
        distance: Fx,
    },
}

/// Churning ground: where a body under the floor is, or is coming up.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Mark {
    pub at: V3,
    pub radius: Fx,
    /// It is coming up here: an eruption's windup.
    pub rising: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Actor {
    pub genome: Genome,
    pub posture: Posture,
    /// Where it stands, on the floor, in the world.
    pub at: V3,
    /// Which way it faces, turns from `+x` toward `+z`.
    pub facing: Fx,
    pub act: Act,
    /// Its moves, with their motions, as the kit fills them.
    pub kit: [Option<(MoveSpec, Motion)>; build::MAX_MOVES],
    pub kit_len: usize,
    /// Its travel last frame, metres: what somebody glancing at it sees.
    pub velocity: V3,
    cache: Option<(Drive, Body)>,
    /// How many times the body has been built, for the cache's test.
    pub builds: u32,
}

/// `v` turned by `yaw` turns about the vertical, toward `+z`.
pub fn spin(v: V3, yaw: Fx) -> V3 {
    let (c, s) = (cos_turns(yaw), sin_turns(yaw));
    V3::new(v.x.mul(c).sub(v.z.mul(s)), v.y, v.x.mul(s).add(v.z.mul(c)))
}

/// A turn wrapped into `-1/2..1/2`.
pub fn wrap_half(t: Fx) -> Fx {
    let w = t.sub(Fx::from_int(t.add(fx(1, 2)).to_int()));
    let w = if w < fx(-1, 2) { w.add(Fx::ONE) } else { w };
    if w >= fx(1, 2) { w.sub(Fx::ONE) } else { w }
}

impl Actor {
    pub fn new(genome: &Genome, at: V3, facing: Fx) -> Actor {
        let q = posture::posture(genome);
        let (kit, kit_len) = strike::kit(genome, &q);
        Actor {
            genome: *genome,
            posture: q,
            at,
            facing,
            act: Act::Stand,
            kit,
            kit_len,
            velocity: V3::ZERO,
            cache: None,
            builds: 0,
        }
    }

    /// How fast it can turn, in turns a frame: slower as the square root
    /// of its length ([`crate::tuning::breed_turn`]), quicker for a quick
    /// one.
    pub fn turn_rate(&self) -> Fx {
        let len = proportions(&self.genome).len.max(fx(1, 2));
        let per_second = crate::tuning::breed_turn()
            .div(len.sqrt())
            .mul(fx(1, 2).add(self.genome.sliders.speed));
        per_second.div(Fx::from_int(FPS))
    }

    /// Whether it is free to be told something new.
    pub fn idle(&self) -> bool {
        matches!(self.act, Act::Stand | Act::Go { .. })
    }

    /// Stand still.
    pub fn stand(&mut self) {
        if self.idle() {
            self.act = Act::Stand;
        }
    }

    /// Move at `share` of its top speed, turning by `turn` of its rate.
    /// Under the floor it only steers.
    pub fn go(&mut self, share: Fx, turn: Fx) {
        let turn = turn.max(Fx::ONE.neg()).min(Fx::ONE);
        if let Act::Under { frames, .. } = self.act {
            self.act = Act::Under { frames, turn };
            return;
        }
        if !self.idle() {
            return;
        }
        let gait = locomotion::at(&self.genome, &self.posture, share);
        let phase = match self.act {
            Act::Go { phase, .. } => phase,
            _ => Fx::ZERO,
        };
        self.act = Act::Go { gait, phase, turn };
    }

    /// Start move `index` of its kit, aimed at `bearing` turns off its
    /// facing (as far as a windup turns; see [`strike::toward`]).
    pub fn strike(&mut self, index: usize, bearing: Fx) -> bool {
        self.strike_at(index, bearing, Fx::ZERO)
    }

    /// [`Actor::strike`], a thrown move aimed to come down `distance`
    /// metres ahead.
    pub fn strike_at(&mut self, index: usize, bearing: Fx, distance: Fx) -> bool {
        if index >= self.kit_len {
            return false;
        }
        let Some((spec, _)) = self.kit[index] else {
            return false;
        };
        // Under, the only move is up; above, there is no coming up.
        let under = self.under();
        let allowed = match spec.kind {
            MoveKind::Erupt => under,
            _ => self.idle(),
        };
        if !allowed {
            return false;
        }
        self.act = Act::Strike {
            index,
            frame: 0,
            bearing: wrap_half(bearing),
            distance,
        };
        true
    }

    /// Whether it is under the floor, travelling there.
    pub fn under(&self) -> bool {
        matches!(self.act, Act::Under { .. })
    }

    /// Whether anything can hit it: not under, not once a burrow has
    /// taken it down, and not until an eruption breaks the floor.
    pub fn exposed(&self) -> bool {
        match self.striking() {
            Some((spec, frame, _)) => match spec.kind {
                MoveKind::Burrow => frame < spec.startup,
                MoveKind::Erupt => frame >= spec.startup,
                _ => true,
            },
            None => !self.under(),
        }
    }

    /// The churning ground over a body that is under: where it is, and
    /// how wide. Through an eruption's windup, where it will come up --
    /// the floor marker that is its tell (bestiary §1.4) -- with `rising`.
    pub fn mark(&self) -> Option<Mark> {
        let len = proportions(&self.genome).len;
        if let Some((spec, frame, m)) = self.striking()
            && spec.kind == MoveKind::Erupt
            && frame < spec.startup + spec.active
        {
            let up = spin(V3::new(m.struck.surge, Fx::ZERO, Fx::ZERO), m.struck.yaw);
            return Some(Mark {
                at: self.to_world(up),
                radius: strike::mark_radius(len),
                rising: true,
            });
        }
        if self.under() {
            return Some(Mark {
                at: self.at,
                radius: strike::mark_radius(len).mul(fx(1, 2)),
                rising: false,
            });
        }
        None
    }

    /// The move it is playing and how far in, if it is.
    pub fn striking(&self) -> Option<(MoveSpec, i32, Motion)> {
        match self.act {
            Act::Strike {
                index,
                frame,
                bearing,
                distance,
            } => {
                self.kit[index].map(|(spec, m)| (spec, frame, strike::aimed(&m, bearing, distance)))
            }
            _ => None,
        }
    }

    /// One frame on.
    pub fn step(&mut self) {
        let before = self.root();
        let rate = self.turn_rate();
        match &mut self.act {
            Act::Stand => {}
            Act::Under { frames, turn } => {
                *frames += 1;
                self.facing = wrap_half(self.facing.add(turn.mul(rate)));
                let ahead = spin(V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO), self.facing);
                let speed = locomotion::at(&self.genome, &self.posture, Fx::ONE)
                    .speed
                    .mul(crate::tuning::breed_under_speed());
                self.at = self.at.add(ahead.scale(speed.div(Fx::from_int(FPS))));
                // Under for as long as it may be, and no longer: up where
                // it is.
                if *frames >= crate::tuning::breed_under_most()
                    && let Some(k) = self.kind_at(MoveKind::Erupt)
                {
                    self.strike_at(k, Fx::ZERO, Fx::ONE);
                }
            }
            Act::Go { gait, phase, turn } => {
                self.facing = wrap_half(self.facing.add(turn.mul(rate)));
                let ahead = spin(V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO), self.facing);
                self.at = self.at.add(ahead.scale(gait.speed.div(Fx::from_int(FPS))));
                let cycle = Fx::from_int(gait.cycle.max(1));
                let next = phase.add(Fx::ONE.div(cycle));
                *phase = if next >= Fx::ONE {
                    next.sub(Fx::ONE)
                } else {
                    next
                };
            }
            Act::Strike {
                index,
                frame,
                bearing,
                distance,
            } => {
                let entry = self.kit[*index];
                *frame += 1;
                // A burrow ends under the floor, not back on it.
                if let Some((spec, _)) = entry
                    && spec.kind == MoveKind::Burrow
                    && *frame >= spec.startup + spec.active
                {
                    self.act = Act::Under {
                        frames: 0,
                        turn: Fx::ZERO,
                    };
                } else if let Some((spec, m)) = entry
                    && *frame >= strike::frames(&spec)
                {
                    // Where the move carried it, facing the way it ended.
                    let end = strike::aimed(&m, *bearing, *distance).struck;
                    let carried = spin(
                        V3::new(end.surge, Fx::ZERO, Fx::ZERO),
                        self.facing.add(end.yaw),
                    );
                    self.at = self.at.add(carried);
                    self.facing = wrap_half(self.facing.add(end.yaw));
                    self.act = Act::Stand;
                }
            }
        }
        // What somebody glancing at it sees move: its root, which a move
        // carries before it is committed.
        self.velocity = self.root().sub(before);
    }

    /// Where its root is over the floor, in the world, counting a move's
    /// travel so far.
    pub fn root(&self) -> V3 {
        let d = self.drive();
        self.at.add(spin(
            V3::new(d.surge, Fx::ZERO, Fx::ZERO),
            self.facing.add(d.yaw),
        ))
    }

    /// This frame's drive.
    pub fn drive(&self) -> Drive {
        match self.act {
            Act::Stand => locomotion::rest(&self.genome),
            Act::Under { .. } => locomotion::rest(&self.genome)
                .plus(&strike::under_drive(&self.genome, &self.posture)),
            Act::Go { gait, phase, turn } => {
                locomotion::drive(&self.genome, &self.posture, &gait, phase, turn)
            }
            Act::Strike { .. } => match self.striking() {
                Some((spec, frame, motion)) => {
                    strike::drive_at(&self.genome, &motion, &spec, frame)
                }
                None => locomotion::rest(&self.genome),
            },
        }
    }

    /// This frame's body, in its own frame (`at` and `facing` put it in
    /// the world): rebuilt only when the drive has changed.
    pub fn body(&mut self) -> Body {
        let d = self.drive();
        if let Some((cached, body)) = &self.cache
            && *cached == d
        {
            return *body;
        }
        let body = build::body_driven(&self.genome, &self.posture, &d);
        self.builds += 1;
        self.cache = Some((d, body));
        body
    }

    /// The kit index of a move it has.
    pub fn kind_at(&self, kind: MoveKind) -> Option<usize> {
        (0..self.kit_len).find(|&k| self.kit[k].is_some_and(|(s, _)| s.kind == kind))
    }

    /// A point of its body in the world.
    pub fn to_world(&self, p: V3) -> V3 {
        self.at.add(spin(p, self.facing))
    }

    /// A point in the world in its own frame: ahead `+x`, its left `+z`.
    pub fn to_own(&self, p: V3) -> V3 {
        spin(p.sub(self.at), self.facing.neg())
    }

    /// What it hits this frame, in the world.
    pub fn hit(&mut self) -> Hit {
        let Some((spec, frame, motion)) = self.striking() else {
            return Hit::EMPTY;
        };
        if strike::beat(&spec, frame) != strike::Beat::Strike {
            return Hit::EMPTY;
        }
        let body = self.body();
        let mut hit = strike::hit_on(&self.genome, &body, &motion, &spec, frame);
        for s in hit.spheres[..hit.count].iter_mut() {
            s.at = self.to_world(s.at);
        }
        hit
    }
}
