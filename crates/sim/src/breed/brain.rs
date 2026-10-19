//! Which move when: a crossbred creature's brain, built on what its moves
//! were measured to do.
//!
//! It **glances**, it does not watch (bestiary §1.5): every few frames it
//! takes a sample of where its target is and how it is moving, and it
//! decides on the newest sample that is at least
//! [`HUMAN_REACTION_FRAMES`] old, projected forward along the velocity it
//! saw. So anything that looks like a reaction to you is a reaction to
//! something it saw a reaction-time ago, and a move it starts locks where
//! it was pointed: step off the line after the tell and it misses.
//!
//! It chooses **only moves that can land**: a move is a candidate when the
//! target is in the lane and between the near and far reach that
//! [`super::verdict`] measured for it -- the same measurement the contract
//! is judged by, so a brain never throws a move the verdict says cannot
//! reach. Among candidates it picks by the move's appetite (its weight),
//! never the same move twice running when there is another, and after a
//! move it waits a beat that its aggression shortens. With nothing that
//! can land it turns to face the target, closes on it, or circles when it
//! is too close for anything it has.
//!
//! [`HUMAN_REACTION_FRAMES`]: crate::tuning::HUMAN_REACTION_FRAMES

use super::actor::{Actor, wrap_half};
use super::build::{self, MoveKind, proportions};
use super::strike::{Delivery, MOST_TURN};
use super::verdict::{self, Extent, Lane, Reading};
use super::{Rng, fx};
use crate::fixed::Fx;
use crate::math::{V3, atan2_turns};
use crate::tuning::HUMAN_REACTION_FRAMES;

/// How many glances it remembers: enough to hold one a reaction old
/// however often it glances (one a frame, at the Oven's least).
const MEMORY: usize = 32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Glance {
    pub at: V3,
    /// Metres a frame.
    pub velocity: V3,
    pub frame: i32,
}

/// What it decided this frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Choice {
    /// Nothing new: it is busy, resting, or has seen nothing yet.
    Hold,
    /// A move, aimed `bearing` turns off its facing and, if thrown, to
    /// come down `distance` metres ahead.
    Strike {
        index: usize,
        bearing: Fx,
        distance: Fx,
    },
    Go {
        share: Fx,
        turn: Fx,
    },
}

#[derive(Clone, Copy, Debug)]
pub struct Brain {
    glances: [Option<Glance>; MEMORY],
    next: usize,
    last_glance: i32,
    rng: Rng,
    readings: [Option<Reading>; build::MAX_MOVES],
    extent: Extent,
    last: Option<usize>,
    rest_until: i32,
    /// Which moves it has thrown, by kit index: its variety.
    pub thrown: [u32; build::MAX_MOVES],
}

impl Brain {
    /// A brain for this actor's body and kit, its rolls seeded by `seed`.
    pub fn new(me: &Actor, seed: u64) -> Brain {
        let g = &me.genome;
        let q = &me.posture;
        let rest = build::body_in(g, q);
        let mut readings = [None; build::MAX_MOVES];
        for (k, entry) in me.kit[..me.kit_len].iter().enumerate() {
            if let Some((spec, m)) = entry {
                readings[k] = Some(verdict::read(g, q, &rest, spec, m));
            }
        }
        Brain {
            glances: [None; MEMORY],
            next: 0,
            last_glance: i32::MIN / 2,
            rng: Rng::new(seed ^ g.fingerprint()),
            readings,
            extent: verdict::extent(&rest, g.stance == super::Stance::Limbless),
            last: None,
            rest_until: 0,
            thrown: [0; build::MAX_MOVES],
        }
    }

    /// Take a glance, if one is due.
    pub fn glance(&mut self, now: i32, at: V3, velocity: V3) {
        if now - self.last_glance < crate::tuning::breed_glance() {
            return;
        }
        self.glances[self.next] = Some(Glance {
            at,
            velocity,
            frame: now,
        });
        self.next = (self.next + 1) % MEMORY;
        self.last_glance = now;
    }

    /// Where it believes the target is now: its newest glance at least a
    /// reaction old, carried forward along the velocity it saw.
    pub fn believes(&self, now: i32) -> Option<V3> {
        let old = HUMAN_REACTION_FRAMES as i32;
        let seen = self
            .glances
            .iter()
            .flatten()
            .filter(|g| now - g.frame >= old)
            .max_by_key(|g| g.frame)?;
        Some(
            seen.at
                .add(seen.velocity.scale(Fx::from_int(now - seen.frame))),
        )
    }

    /// How far a point in its own frame is outside its body, on the floor.
    fn outside(&self, rel: V3) -> Fx {
        let e = &self.extent;
        let dx = if rel.x > e.nose {
            rel.x.sub(e.nose)
        } else if rel.x < e.rump {
            e.rump.sub(rel.x)
        } else {
            Fx::ZERO
        };
        let dz = rel.z.abs().sub(e.half).max(Fx::ZERO);
        V3::new(dx, Fx::ZERO, dz).len()
    }

    /// Whether a target at `rel` (its own frame), `bearing` turns off its
    /// nose, is in a lane: ahead within a windup's turn, behind, beside,
    /// or anywhere for a move that hits all round.
    fn in_lane(&self, lane: Lane, rel: V3, bearing: Fx) -> bool {
        let e = &self.extent;
        let slack = fx(1, 16);
        match lane {
            Lane::Front | Lane::Above => {
                rel.x > e.nose.mul(fx(1, 2)) && bearing.abs() <= MOST_TURN.add(slack)
            }
            Lane::Rear => {
                rel.x < e.rump.mul(fx(1, 2))
                    && wrap_half(bearing.sub(fx(1, 2))).abs() <= MOST_TURN.add(slack)
            }
            Lane::Side => rel.x <= e.nose && rel.x >= e.rump,
            Lane::Around => true,
            Lane::Aboard => false,
        }
    }

    /// Decide, for this frame.
    pub fn think(&mut self, me: &Actor, now: i32) -> Choice {
        if me.under() {
            return self.under(me, now);
        }
        if !me.idle() || now < self.rest_until {
            return Choice::Hold;
        }
        let Some(target) = self.believes(now) else {
            return Choice::Hold;
        };
        let rel = me.to_own(target);
        let bearing = atan2_turns(rel.z, rel.x);
        let dist = self.outside(rel);
        // Every move that can land from here, weighed.
        let mut weights = [Fx::ZERO; build::MAX_MOVES];
        let mut total = Fx::ZERO;
        let mut options = 0;
        let slack = fx(1, 2);
        let entries = self.readings.iter().zip(me.kit.iter()).take(me.kit_len);
        for (w, (reading, entry)) in weights.iter_mut().zip(entries) {
            let (Some(r), Some((spec, m))) = (reading, entry) else {
                continue;
            };
            // Going under is a way to close on anybody not already at its
            // nose -- a burrower's own way to approach -- weighed like a
            // move.
            if spec.kind == MoveKind::Burrow {
                let len = proportions(&me.genome).len;
                if dist > super::strike::mark_radius(len).mul(crate::tuning::breed_burrow_from()) {
                    *w = spec.weight.max(fx(1, 20));
                    total = total.add(*w);
                    options += 1;
                }
                continue;
            }
            if !r.damaging || r.lane == Lane::Aboard || spec.kind == MoveKind::Erupt {
                continue;
            }
            let far = if m.delivery == Delivery::Thrown {
                spec.reach
            } else {
                r.beyond
            };
            if dist.add(slack) < r.near || dist > far || !self.in_lane(r.lane, rel, bearing) {
                continue;
            }
            *w = spec.weight.max(fx(1, 20));
            total = total.add(*w);
            options += 1;
        }
        // Closing in is an option too, when it has moves that reach less
        // far than where the target stands -- and the more so the more
        // aggressive it is -- so a spider with a web and a bite does not
        // stand off spitting forever.
        let closer = (0..me.kit_len).any(|k| {
            self.readings[k]
                .is_some_and(|r| r.damaging && r.beyond < dist && r.lane != Lane::Aboard)
        });
        let approach = if closer {
            me.genome.sliders.aggression.mul(fx(1, 2)).max(fx(1, 10))
        } else {
            Fx::ZERO
        };
        // Not the same move twice running, when there is another -- and
        // when there is not, as often as not it moves to find one rather
        // than throw the same thing again.
        if let Some(last) = self.last
            && weights[last] > Fx::ZERO
            && (options > 1 || self.rng.chance(fx(1, 2)))
        {
            total = total.sub(weights[last]);
            weights[last] = Fx::ZERO;
        }
        if total > Fx::ZERO {
            let mut pick = self.rng.unit().mul(total.add(approach));
            if pick >= total {
                // Committed to it for a stride or two.
                self.rest_until = now + 30;
                return Choice::Go {
                    share: fx(2, 5),
                    turn: Fx::ZERO,
                };
            }
            for (k, w) in weights.iter().enumerate() {
                if *w == Fx::ZERO {
                    continue;
                }
                if pick < *w || k + 1 == me.kit_len {
                    return self.commit(me, k, rel, bearing, now);
                }
                pick = pick.sub(*w);
            }
            // Rounding left it past the end: the last option.
            if let Some(k) = weights.iter().rposition(|w| *w > Fx::ZERO) {
                return self.commit(me, k, rel, bearing, now);
            }
        }
        // Nothing lands from here: face it, close on it, or circle.
        let turn = if bearing.abs() < fx(1, 64) {
            Fx::ZERO
        } else if bearing > Fx::ZERO {
            Fx::ONE
        } else {
            Fx::ONE.neg()
        };
        if bearing.abs() > fx(1, 6) {
            // A burrower turns the fastest way it has: under, and up
            // facing you.
            if let Some(k) = me.kind_at(MoveKind::Burrow) {
                return self.commit(me, k, rel, Fx::ZERO, now);
            }
            return Choice::Go {
                share: fx(1, 10),
                turn,
            };
        }
        let len = proportions(&me.genome).len;
        let run = len
            .mul(crate::tuning::breed_run_lengths())
            .max(crate::tuning::breed_run_at_least());
        let share = if dist > run {
            Fx::ONE
        } else if dist > fx(1, 2) {
            fx(2, 5)
        } else {
            // Too close for anything it has ahead: circle to find a lane.
            return Choice::Go {
                share: fx(1, 4),
                turn: Fx::ONE,
            };
        };
        Choice::Go { share, turn }
    }

    /// Under the floor: steer toward where it believes the target is, and
    /// come up under them once they are within its reach -- or when it
    /// must, wherever it is.
    fn under(&mut self, me: &Actor, now: i32) -> Choice {
        let Some(k) = me.kind_at(MoveKind::Erupt) else {
            return Choice::Hold;
        };
        let Some((spec, _)) = me.kit[k] else {
            return Choice::Hold;
        };
        let frames = match me.act {
            super::actor::Act::Under { frames, .. } => frames,
            _ => 0,
        };
        let Some(target) = self.believes(now) else {
            return Choice::Go {
                share: Fx::ZERO,
                turn: Fx::ZERO,
            };
        };
        let rel = me.to_own(target);
        let bearing = atan2_turns(rel.z, rel.x);
        let flat = V3::new(rel.x, Fx::ZERO, rel.z).len();
        let late = frames + spec.startup >= crate::tuning::breed_under_most();
        if (flat <= spec.reach && bearing.abs() <= MOST_TURN) || late {
            self.last = Some(k);
            self.thrown[k] += 1;
            self.rest_until = now + super::strike::frames(&spec);
            return Choice::Strike {
                index: k,
                bearing,
                distance: flat.max(Fx::ONE),
            };
        }
        let turn = if bearing.abs() < fx(1, 64) {
            Fx::ZERO
        } else if bearing > Fx::ZERO {
            Fx::ONE
        } else {
            Fx::ONE.neg()
        };
        Choice::Go {
            share: Fx::ZERO,
            turn,
        }
    }

    fn commit(&mut self, me: &Actor, k: usize, rel: V3, bearing: Fx, now: i32) -> Choice {
        let Some((spec, _)) = me.kit[k] else {
            return Choice::Hold;
        };
        let lane = self.readings[k].map(|r| r.lane).unwrap_or(Lane::Front);
        // Aimed with the part that hits: a rear move turns its rear to it.
        let aim = match lane {
            Lane::Rear => wrap_half(bearing.sub(fx(1, 2))),
            Lane::Front | Lane::Above => bearing,
            _ => Fx::ZERO,
        };
        self.last = Some(k);
        self.thrown[k] += 1;
        // A beat between moves, which aggression shortens.
        let pause = Fx::from_int(crate::tuning::breed_pause())
            .mul(Fx::ONE.sub(me.genome.sliders.aggression))
            .to_int();
        self.rest_until = now + super::strike::frames(&spec) + pause;
        Choice::Strike {
            index: k,
            bearing: aim,
            distance: V3::new(rel.x, Fx::ZERO, rel.z).len(),
        }
    }

    /// Do what it decided.
    pub fn act(choice: Choice, me: &mut Actor) {
        match choice {
            Choice::Hold => {}
            Choice::Strike {
                index,
                bearing,
                distance,
            } => {
                me.strike_at(index, bearing, distance);
            }
            Choice::Go { share, turn } => me.go(share, turn),
        }
    }
}
