//! The Siegeshell's hunters: the legs, the crown, and one hunter playing both
//! -- and the report lines the fight is measured by.
//!
//! `docs/design/creatures/siegeshell.md` §9, what a person learns in their
//! first ten minutes:
//!
//! 1. **Legs**: stand four metres outside an ankle, beside a leg that has
//!    just landed. Jump every ring. Swing between beats. Out of a stamp's
//!    pad at the drop; round the outside of a drag. Move to the next leg on
//!    the same side after a break. **With a partner, stop one hit short of
//!    the second break and wait for the call** -- the partner standing at an
//!    anchor, which it reads off the world the way a person reads their
//!    partner (no shared memory between the two plans).
//! 2. **Crown**: wait at the knee of the side being broken; climb on the
//!    stumble; walk the treads, inward and up, stepping off grates through
//!    the hiss; brace on the shrug's tell; at the crown stand at the nearest
//!    anchor and swing; jump the shiver late in its tell.
//! 3. **Both**: never stand under the belly; never trail behind; out of a
//!    shed's circles by going in under the rim; out of the plough's lane.
//!
//! **Solo** is the legs until a side goes down or it kneels, then the crown,
//! and the legs again whenever it is back on the floor.
//!
//! Like every plan it sees the world `REACTION` frames late, and reads only
//! what a person sees: the body, the floor's signs (every landing pad, every
//! ring, the stamp's disc, the drag's lane, the shed's circles, the plough's
//! lane), the grates, and the creature's visible windups.

use sim::fixed::Fx;
use sim::math::{atan2_turns, wide_flat_dist, wide_normalized};
use sim::monster::{Doing, Monster, mount_part};
use sim::sign::{Says, Shape};
use sim::species::siegeshell::{
    self as ss, ANCHOR_COUNT, Knob, LEG_COUNT, SHIVER, SHRUG, fight, gait, mind,
};
use sim::state::{MAX_PLAYERS, Phase, Player};
use sim::{Input, V3, World};

use crate::report::Tally;
use crate::{Intent, Plan, REACTION, steer, turns_to_aim};

/// At an ankle, swinging.
pub const ANKLE: Intent = Intent("Ankle");
/// Jumping a ring.
pub const BEAT: Intent = Intent("Beat");
/// Out of something drawn on the floor.
pub const EVADE: Intent = Intent("Evade");
/// Waiting at a knee for the stumble, or for the call.
pub const WAIT: Intent = Intent("Wait");
/// Climbing the stair.
pub const CLIMB: Intent = Intent("Climb");
/// Crossing the shell to an anchor.
pub const CROSS: Intent = Intent("Cross");
/// At an anchor, swinging.
pub const ANCHOR: Intent = Intent("Anchor");
/// Bracing for a shrug, or jumping a shiver.
pub const HOLD: Intent = Intent("Hold");
/// At a parasite.
pub const PARASITE: Intent = Intent("Parasite");

/// What a hunter is for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    /// The legs, and the crown whenever it gets the chance: alone.
    Solo,
    /// Breaks ankles on the beat and calls the stumble.
    Legs,
    /// Climbs on the stumble and works the anchors.
    Crown,
}

/// Frames between swings.
const SWING_GAP: u16 = 10;
/// How far outside an ankle the legs hunter stands.
const POST_OUT: Fx = Fx::ratio(27, 10);
/// A jump goes between these many frames before the ring's front reaches
/// it: a hop takes about six frames to clear a knee-high ring.
const JUMP_EARLY: i32 = 10;
const JUMP_LATE: i32 = 5;
/// Close enough to stop walking.
const SETTLED: Fx = Fx::ratio(8, 10);
/// A parasite this near is the one to hit.
const PARASITE_NEAR: Fx = Fx::ratio(30, 10);
/// With a partner, the second ankle on a side is held at this share of its
/// health until the call.
const SLIVER: i32 = 15;
/// Frames a walk may fail to move it before it sidesteps, and for how long.
const STUCK: u16 = 8;
const DETOUR: u16 = 20;

#[derive(Clone)]
struct Seen {
    world: World,
}

pub struct Siegeshell {
    who: usize,
    role: Role,
    memory: Vec<Option<Seen>>,
    at: usize,
    filled: usize,
    intent: Intent,
    cooldown: u16,
    dodge_left: u16,
    hold_jump: u16,
    /// What it was pressing when it jumped: held, with the jump, through the
    /// rise, so a hop up a tread carries on toward it.
    held: Input,
    /// The side it is breaking: `-1` the creature's left, `+1` its right.
    side: i32,
    last: V3,
    stuck: u16,
    detour_left: u16,
}

impl Siegeshell {
    pub fn new(who: usize, seed: u32, role: Role) -> Siegeshell {
        Siegeshell {
            who,
            role,
            memory: vec![None; REACTION + 1],
            at: 0,
            filled: 0,
            intent: WAIT,
            cooldown: 0,
            dodge_left: 0,
            hold_jump: 0,
            held: Input::default(),
            // The side it breaks first, from the seed: a pair of
            // hunters starts on whichever, as people would.
            side: if (seed ^ (seed >> 7)) & 1 == 0 { -1 } else { 1 },
            last: V3::ZERO,
            stuck: 0,
            detour_left: 0,
        }
    }

    fn recall(&self) -> Option<World> {
        let back = REACTION.min(self.filled.saturating_sub(1));
        let idx = (self.at + self.memory.len() - 1 - back) % self.memory.len();
        self.memory[idx].as_ref().map(|s| s.world.clone())
    }

    /// A walk that has not moved it bears off to one side for a moment.
    fn unstick(&mut self, me: &Player, input: Input) -> Input {
        const WALK: u16 = Input::W | Input::A | Input::S | Input::D;
        let walking = input.bits & WALK != 0;
        let moved = wide_flat_dist(me.pos, self.last).raw() > Fx::ratio(1, 100).raw();
        self.last = me.pos;
        self.stuck = if walking && !moved && me.action.actionable() {
            self.stuck + 1
        } else {
            0
        };
        if self.stuck >= STUCK {
            self.stuck = 0;
            self.detour_left = DETOUR;
        }
        if self.detour_left == 0 || !walking {
            return input;
        }
        self.detour_left -= 1;
        let b = input.bits;
        let turned = (if b & Input::W != 0 { Input::A } else { 0 })
            | (if b & Input::A != 0 { Input::S } else { 0 })
            | (if b & Input::S != 0 { Input::D } else { 0 })
            | (if b & Input::D != 0 { Input::W } else { 0 });
        Input {
            bits: (b & !WALK) | turned | (b & Input::W),
            ..input
        }
    }
}

/// The plan alone: the legs, then the crown when it can.
pub fn solo(who: usize, seed: u32, _hop: Fx) -> Box<dyn Plan + Send + Sync> {
    let role = if who == 0 { Role::Solo } else { Role::Crown };
    Box::new(Siegeshell::new(who, seed, role))
}

// ---------------------------------------------------------------------------
// Reading it
// ---------------------------------------------------------------------------

fn flat(v: V3) -> V3 {
    V3::new(v.x, Fx::ZERO, v.z)
}

fn unit(v: V3) -> V3 {
    let f = flat(v);
    if sim::math::wide_flat_len(f).raw() == 0 {
        V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO)
    } else {
        wide_normalized(f)
    }
}

/// Look at a point, the crosshair on it, pressing `bits`.
fn looking(me: &Player, at: V3, bits: u16) -> Input {
    let d = flat(at.sub(me.pos));
    let yaw = atan2_turns(d.z, d.x);
    let wire = turns_to_aim(yaw.sub(me.carry_yaw));
    let pitch = sim::aim::look_onto_closely(me.pos, wire, me.aloft, at);
    Input::looking_at(bits, wire, pitch)
}

/// Walk toward `to`, looking at `face`.
fn go(me: &Player, to: V3, face: V3, extra: u16) -> Input {
    let look = flat(face.sub(me.pos));
    let yaw = atan2_turns(look.z, look.x);
    let way = flat(to.sub(me.pos));
    let walk = if sim::math::wide_flat_len(way).raw() > SETTLED.raw() {
        steer(yaw, way)
    } else {
        0
    };
    looking(me, face, walk | extra)
}

/// The auto's button.
fn auto_button(me: &Player) -> u16 {
    match sim::dual::bars(me) {
        Some((dark, light)) if dark.raw() < light.raw() => Input::LEFT,
        Some(_) => Input::RIGHT,
        None => Input::LEFT,
    }
}

fn reach_of(me: &Player) -> Fx {
    let slot = match me.class {
        sim::Class::BloodMage => sim::moves::blood::SWEEP,
        _ => 0,
    };
    sim::moves::get(me.class, slot).reach.max(Fx::ONE)
}

/// The middle of a part, in the world.
fn middle(m: &Monster, part: usize) -> V3 {
    let sh = m.sp().shape(part);
    m.rig()
        .part_to_world(part, sh.min.add(sh.max).scale(Fx::ratio(1, 2)))
}

/// Where the legs hunter stands for an ankle: outside it, by `POST_OUT`.
fn post_for(m: &Monster, leg: usize) -> V3 {
    let foot = gait::foot(m, leg);
    let out = sim::math::wide_normalized(flat(foot.sub(m.pos)));
    let side = gait::flat_world(
        m,
        V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(ss::leg_side(leg))),
    )
    .sub(m.pos);
    let _ = out;
    flat(foot.add(side.scale(POST_OUT)))
}

/// The ankle it should break next on `side`: the soundest one that is not
/// swinging, fore before mid before hind.
fn next_ankle(m: &Monster, side: i32) -> Option<usize> {
    (0..LEG_COUNT)
        .filter(|l| ss::leg_side(*l) == side && !m.broken(ss::ankle_part(*l)))
        .min_by_key(|l| (ss::leg_pair(*l) as i32, m.part_health(ss::ankle_part(*l))))
}

/// Damage still to go before a broken ankle's next buckle.
fn to_buckle(m: &Monster, leg: usize) -> i32 {
    let every = Knob::BuckleHealth.raw().max(1);
    let under = (-m.part_health(ss::ankle_part(leg))).max(0);
    every - under % every
}

/// The side to break: the one with ankles already broken, else the left.
fn pick_side(m: &Monster) -> i32 {
    let (l, r) = gait::broken_sides(m);
    if r > l { 1 } else { -1 }
}

/// Is a stumble down, or a kneel, and which side.
fn down(m: &Monster) -> Option<i32> {
    match m.doing {
        Doing::Stumble { .. } => Some(fight::stumble_side(m)),
        Doing::Toppled { .. } => Some(0),
        _ => None,
    }
}

/// The partner, if it is in the fight.
fn partner(w: &World, who: usize) -> Option<Player> {
    (0..MAX_PLAYERS)
        .filter(|i| *i != who)
        .map(|i| w.players[i])
        .find(|p| p.health > 0)
}

/// The anchor a body stands within reach of, if any.
fn tended(m: &Monster, at: V3) -> bool {
    let near = Knob::OpenRadius.fx();
    (0..ANCHOR_COUNT)
        .filter(|a| !m.broken(ss::anchor_part(*a)))
        .any(|a| wide_flat_dist(middle(m, ss::anchor_part(a)), at).raw() <= near.raw())
}

/// The nearest unbroken anchor, by health first at the siege line.
fn target_anchor(m: &Monster, me: V3) -> Option<usize> {
    let siege = fight::at_siege_line(m);
    (0..ANCHOR_COUNT)
        .filter(|a| !m.broken(ss::anchor_part(*a)))
        .min_by_key(|a| {
            if siege {
                m.part_health(ss::anchor_part(*a))
            } else {
                wide_flat_dist(middle(m, ss::anchor_part(*a)), me).raw()
            }
        })
}

/// Frames until the next ring reaches a point, if a tripod is coming down
/// near it: from the landing spots of the feet in the air and the beat.
fn ring_due(m: &Monster, at: V3) -> Option<i32> {
    let beat = gait::frames_to_beat(m)? as i32 - REACTION as i32;
    let to = Knob::RingTo.fx().add(Fx::ONE);
    let mut near = None;
    for l in 0..LEG_COUNT {
        if gait::swinging(m, l).is_none() {
            continue;
        }
        let d = wide_flat_dist(gait::landing_spot(m, l), at);
        if d.raw() <= to.raw() {
            let from = Knob::RingFrom.fx();
            let span = Knob::RingTo.fx().sub(from).max(Fx::ONE);
            let frames = Knob::RingFrames.raw().max(1);
            let out = d.sub(from).max(Fx::ZERO).min(span);
            let k = out.mul(Fx::from_int(frames)).div(span).to_int();
            let t = beat + k;
            near = Some(near.map_or(t, |n: i32| n.min(t)));
        }
    }
    near
}

/// Does a sign cover a body standing at `at`: its shape grown by the body's
/// width and a step, since what it hits is a body, not a point.
fn covered(s: &sim::sign::Sign, at: V3) -> bool {
    let pad = sim::tuning::body_radius().add(Fx::ratio(5, 10));
    let d = flat(at.sub(s.at));
    match s.shape {
        Shape::Strip => {
            let side = V3::new(s.along.z.neg(), Fx::ZERO, s.along.x);
            let a = d.dot(s.along);
            let c = d.dot(side);
            a.raw() >= pad.neg().raw()
                && a.raw() <= s.length.add(pad).raw()
                && c.abs().raw() <= s.width.mul(Fx::ratio(1, 2)).add(pad).raw()
        }
        Shape::Disc | Shape::Ring => {
            sim::math::wide_flat_len(d).raw() <= s.width.mul(Fx::ratio(1, 2)).add(pad).raw()
        }
    }
}

/// The way out of whatever drawn thing is coming down on a point: away from
/// a disc's middle, square off a strip -- and how soon it lands, as far as
/// its fill says.
fn way_out(w: &World, at: V3) -> Option<(V3, Fx)> {
    let signs = w.signs();
    let mut best: Option<(V3, Fx)> = None;
    for s in signs.iter() {
        if !matches!(s.says, Says::Coming | Says::Live) || !covered(s, at) {
            continue;
        }
        let dir = match s.shape {
            Shape::Disc | Shape::Ring => {
                // A beat pad's ring is a landing spot to leave; the faint
                // ring is the shockwave, which is jumped, not left.
                unit(at.sub(s.at))
            }
            Shape::Strip => {
                let side = V3::new(s.along.z.neg(), Fx::ZERO, s.along.x);
                let d = flat(at.sub(s.at));
                if d.dot(side).raw() >= 0 {
                    side
                } else {
                    side.scale(Fx::ONE.neg())
                }
            }
        };
        let p = if s.says == Says::Live {
            Fx::ONE
        } else {
            s.progress
        };
        if best.is_none_or(|(_, q)| p.raw() > q.raw()) {
            best = Some((dir, p));
        }
    }
    best
}

// ---------------------------------------------------------------------------
// One frame
// ---------------------------------------------------------------------------

impl Plan for Siegeshell {
    fn watch(&mut self, w: &World) {
        self.memory[self.at] = Some(Seen { world: w.clone() });
        self.at = (self.at + 1) % self.memory.len();
        self.filled = (self.filled + 1).min(self.memory.len());
    }

    fn act(&mut self, w: &World) -> Input {
        let input = self.choose(w);
        let me = w.players[self.who];
        if std::env::var("SIEGE_DEBUG").is_ok() {
            if let Some(m) = w.monsters[0] {
                let due = ring_due(&m, me.pos);
                let foot = (0..LEG_COUNT)
                    .map(|l| wide_flat_dist(gait::foot(&m, l), me.pos))
                    .min_by_key(|d| d.raw())
                    .unwrap_or(Fx::ZERO);
                eprintln!(
                    "f{} pos {:.1},{:.1} hp {} y {:.2} gr {} mount {} intent {:?} due {:?} beat {:?} foot {:.1} leg {:?} bits {:x} doing {:?}",
                    w.frame,
                    me.pos.x.to_f32_for_render(),
                    me.pos.z.to_f32_for_render(),
                    me.health,
                    me.pos.y.to_f32_for_render(),
                    me.grounded,
                    if me.aboard() {
                        mount_part(me.mount) as i32
                    } else {
                        -1
                    },
                    self.intent,
                    due,
                    gait::frames_to_beat(&m),
                    foot.to_f32_for_render(),
                    fight::leg::get(&m),
                    input.bits,
                    m.doing
                );
            }
        }
        self.unstick(&me, input)
    }

    fn intent(&self) -> Intent {
        self.intent
    }
}

impl Siegeshell {
    fn choose(&mut self, w: &World) -> Input {
        self.cooldown = self.cooldown.saturating_sub(1);
        self.dodge_left = self.dodge_left.saturating_sub(1);
        let me = w.players[self.who];
        let Some(seen) = self.recall() else {
            return Input::default();
        };
        let Some(m) = seen.monsters[0] else {
            return Input::default();
        };
        if me.health <= 0 || !m.alive() || !matches!(w.phase, Phase::Fighting) {
            return Input::default();
        }
        // A jump under way is held through its rise.
        if self.hold_jump > 0 {
            self.hold_jump -= 1;
            return self.held.with(Input::SPACE);
        }
        // Aboard, or in the air off the shell between two treads: the
        // crossing goes on. Only feet on the floor are the floor's.
        let up_there = !me.grounded && me.pos.y.raw() > Fx::from_int(3).raw();
        if me.aboard() || up_there {
            return self.aboard(w, &seen, &m, &me);
        }
        self.floor(w, &seen, &m, &me)
    }

    /// **On the floor.**
    fn floor(&mut self, w: &World, seen: &World, m: &Monster, me: &Player) -> Input {
        let coop = partner(w, self.who).is_some();
        // A ring coming: jump it, late.
        if let Some(due) = ring_due(m, me.pos) {
            if (JUMP_LATE..=JUMP_EARLY).contains(&due) && me.grounded && me.action.actionable() {
                self.intent = BEAT;
                self.hold_jump = 12;
                self.held = looking(me, m.pos, 0);
                return self.held.with(Input::SPACE);
            }
        }
        // A stamp coming down on it: dodge out of the pad at the drop, the
        // dodge's invulnerable frames over the landing.
        let c = fight::leg::get(m);
        if c.kind == fight::leg::STAMP && c.phase == fight::leg::STARTUP {
            let aim = fight::leg::aim(m);
            let near = Knob::PadRadius.fx().add(Fx::from_int(2));
            let lands = c.left as i32 - REACTION as i32;
            if wide_flat_dist(aim, me.pos).raw() <= near.raw() && lands >= -2 {
                self.intent = EVADE;
                let out = unit(unit(me.pos.sub(aim)).add(unit(me.pos.sub(m.pos))));
                let d = flat(m.pos.sub(me.pos));
                let yaw = atan2_turns(d.z, d.x);
                let wire = turns_to_aim(yaw.sub(me.carry_yaw));
                let dodge = if (2..=7).contains(&lands)
                    && self.dodge_left == 0
                    && me.action.actionable()
                    && me.grounded
                {
                    self.dodge_left = sim::tuning::dodge_frames();
                    Input::SHIFT
                } else {
                    0
                };
                // Walking out, never swinging: a swing is frames a dodge
                // cannot be thrown in.
                return Input::aimed(steer(yaw, out) | dodge, wire);
            }
        }
        // Anything drawn on the floor under it: out -- away from its middle,
        // and away from the body, which is where nothing is coming down.
        if let Some((dir, progress)) = way_out(seen, me.pos) {
            let outward = unit(me.pos.sub(m.pos));
            let dir = unit(dir.add(outward));
            self.intent = EVADE;
            let face = m.pos;
            let yaw = {
                let d = flat(face.sub(me.pos));
                atan2_turns(d.z, d.x)
            };
            let late = progress.raw() > Fx::ratio(85, 100).raw();
            let dodge = if late && self.dodge_left == 0 && me.action.actionable() {
                self.dodge_left = sim::tuning::dodge_frames();
                Input::SHIFT
            } else {
                0
            };
            let wire = turns_to_aim(yaw.sub(me.carry_yaw));
            return Input::aimed(steer(yaw, dir) | dodge, wire);
        }
        // A parasite at it.
        if let Some(c) = (0..sim::critter::MAX_CRITTERS)
            .filter(|i| seen.critters[*i].alive() && !seen.critters[*i].mounted())
            .map(|i| seen.critters[i])
            .filter(|c| wide_flat_dist(c.pos, me.pos).raw() <= PARASITE_NEAR.raw())
            .min_by_key(|c| wide_flat_dist(c.pos, me.pos).raw())
        {
            self.intent = PARASITE;
            let sp = seen.critters.sp();
            let at = c.body(sp).middle();
            let swing = self.swing(me);
            return go(me, at, at, swing);
        }
        // Down: the crown's way up, for whoever is climbing -- alone, the
        // legs hunter is the climber too.
        let climbing = match self.role {
            Role::Crown => true,
            Role::Solo => !coop,
            Role::Legs => false,
        };
        if let (Some(side), true) = (down(m), climbing) {
            if let Some(input) = self.climb(m, me, side) {
                return input;
            }
        }
        match self.role {
            Role::Crown if coop => self.wait_at_knee(m, me),
            _ => self.legs(w, m, me, coop),
        }
    }

    fn swing(&mut self, me: &Player) -> u16 {
        if self.cooldown == 0 && me.action.actionable() {
            self.cooldown = SWING_GAP;
            auto_button(me)
        } else {
            0
        }
    }

    /// **The legs**: an ankle on the side being broken, from outside it --
    /// and once two are broken there, a broken one, for the buckle that is
    /// the next stumble.
    fn legs(&mut self, w: &World, m: &Monster, me: &Player, coop: bool) -> Input {
        self.side = pick_side(m);
        let (l, r) = gait::broken_sides(m);
        let broken_here = if self.side < 0 { l } else { r };
        let buckling = broken_here >= 2;
        let leg = if buckling {
            // The broken ankle on this side nearest its next buckle.
            (0..LEG_COUNT)
                .filter(|l| ss::leg_side(*l) == self.side && m.broken(ss::ankle_part(*l)))
                .min_by_key(|l| to_buckle(m, *l))
        } else {
            next_ankle(m, self.side)
        };
        let Some(leg) = leg else {
            self.side = -self.side;
            return Input::default();
        };
        let ankle = ss::ankle_part(leg);
        let post = post_for(m, leg);
        let at = middle(m, ankle);
        // With a partner up there: the blow that brings the side down held
        // until the call -- the partner at an anchor -- so the stumble is the
        // Opening. With the partner on the floor, the stumble is their way up.
        let left = if buckling {
            to_buckle(m, leg)
        } else if broken_here == 1 {
            m.part_health(ankle)
        } else {
            i32::MAX
        };
        let sliver = left * 100 <= m.sp().part_health() * SLIVER;
        let up = partner(w, self.who).is_some_and(|p| p.aboard());
        let called = partner(w, self.who).is_some_and(|p| p.aboard() && tended(m, p.pos));
        let holding = coop && up && sliver && !called;
        self.intent = if holding { WAIT } else { ANKLE };
        // Within its reach of the ankle's face, not its middle.
        let face = Fx::ratio(12, 10);
        let near =
            wide_flat_dist(at, me.pos).sub(face).raw() <= reach_of(me).add(Fx::ratio(5, 10)).raw();
        // Swing between beats: never one that will still be going when a
        // ring arrives.
        let busy = sim::moves::get(me.class, 0);
        let busy = (busy.startup + busy.active + busy.recovery) as i32 + 6;
        let clear = ring_due(m, me.pos).is_none_or(|due| due > busy || due < -12);
        let swing = if near && !holding && clear {
            self.swing(me)
        } else {
            0
        };
        go(me, post, at, swing)
    }

    /// **Waiting for the stumble** at the knee of the side being broken.
    fn wait_at_knee(&mut self, m: &Monster, me: &Player) -> Input {
        self.intent = WAIT;
        let side = pick_side(m);
        let leg = if side < 0 { 0 } else { 1 };
        let foot = gait::foot(m, leg);
        let out = gait::flat_world(m, V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(side))).sub(m.pos);
        // A few metres outside the fore foot, behind the ring's reach.
        let at = flat(foot.add(out.scale(Fx::from_int(9))));
        go(me, at, foot, 0)
    }

    /// **The climb**: to the knee of a splayed broken leg, and in up it,
    /// hopping whenever its feet are down.
    fn climb(&mut self, m: &Monster, me: &Player, side: i32) -> Option<Input> {
        let leg = (0..LEG_COUNT)
            .filter(|l| gait::splayed(m, *l).raw() >= Fx::ratio(9, 10).raw())
            .filter(|l| side == 0 || ss::leg_side(*l) == side)
            .min_by_key(|l| wide_flat_dist(gait::foot(m, *l), me.pos).raw())?;
        self.intent = CLIMB;
        let rig = m.rig();
        let part = ss::thigh_part(leg);
        let sh = m.sp().shape(part);
        let knee = rig.part_to_world(part, V3::new(sh.max.x, sh.max.y, Fx::ZERO));
        let hip = rig.part_to_world(part, V3::new(sh.min.x, sh.max.y, Fx::ZERO));
        let out = unit(knee.sub(hip));
        let start = flat(knee.add(out.scale(Fx::from_int(2))));
        let from_knee = wide_flat_dist(me.pos, knee);
        if from_knee.raw() > Fx::from_int(4).raw() {
            return Some(go(me, start, hip, 0));
        }
        let walk = go(me, flat(hip), hip, 0);
        if me.grounded {
            self.hold_jump = 18;
            self.held = walk;
            return Some(walk.with(Input::SPACE));
        }
        Some(walk)
    }

    /// **Aboard**: brace for a shrug on a side, jump a shiver at the crown,
    /// off a hissing grate, and on to an anchor -- or back down if the
    /// shell is no place for it.
    fn aboard(&mut self, w: &World, seen: &World, m: &Monster, me: &Player) -> Input {
        let region = if me.aboard() {
            mind::region_of(mount_part(me.mount))
        } else {
            mind::Region::Other
        };
        // The shrug's tell, on a side: brace.
        if let Doing::Startup { kind: SHRUG, .. } | Doing::Active { kind: SHRUG, .. } = m.doing {
            if matches!(region, mind::Region::Side(_)) {
                self.intent = HOLD;
                return looking(me, m.pos, Input::CROUCH);
            }
        }
        // The shiver's tell, at the crown: jump late in it.
        if let Doing::Startup { kind: SHIVER, left } = m.doing {
            if region == mind::Region::Crown {
                let left = left as i32 - REACTION as i32;
                self.intent = HOLD;
                if (0..=12).contains(&left) && me.grounded {
                    self.hold_jump = 20;
                    self.held = looking(me, m.pos, 0);
                    return self.held.with(Input::SPACE);
                }
            }
        }
        // The stumble's lurch at the crown: brace.
        if matches!(m.doing, Doing::Stumble { .. }) && region == mind::Region::Crown {
            if let Some(a) = target_anchor(m, me.pos) {
                let at = middle(m, ss::anchor_part(a));
                if wide_flat_dist(at, me.pos).raw() <= reach_of(me).add(Fx::ONE).raw()
                    && me.pos.y.raw() >= at.y.sub(Fx::from_int(2)).raw()
                {
                    self.intent = ANCHOR;
                    let swing = self.swing(me);
                    return looking(me, at, swing);
                }
            }
        }
        // A grate hissing or blowing under it: off.
        if let Some(dir) = on_a_grate(seen, me.pos) {
            self.intent = EVADE;
            let d = flat(m.pos.sub(me.pos));
            let yaw = atan2_turns(d.z, d.x);
            let wire = turns_to_aim(yaw.sub(me.carry_yaw));
            return Input::aimed(steer(yaw, dir), wire);
        }
        // A parasite at it.
        if let Some(c) = (0..sim::critter::MAX_CRITTERS)
            .filter(|i| seen.critters[*i].alive())
            .map(|i| seen.critters[i])
            .filter(|c| wide_flat_dist(c.pos, me.pos).raw() <= PARASITE_NEAR.raw())
            .min_by_key(|c| wide_flat_dist(c.pos, me.pos).raw())
        {
            if c.pos.y.sub(me.pos.y).abs().raw() < Fx::from_int(2).raw() {
                self.intent = PARASITE;
                let sp = seen.critters.sp();
                let at = c.body(sp).middle();
                let swing = self.swing(me);
                return looking(me, at, swing);
            }
        }
        // The anchor.
        let Some(a) = target_anchor(m, me.pos) else {
            return Input::default();
        };
        let at = middle(m, ss::anchor_part(a));
        let d = wide_flat_dist(at, me.pos);
        // Within reach, and up on the crown with it rather than under it.
        let level = me.pos.y.raw() >= at.y.sub(Fx::from_int(2)).raw();
        let face = Fx::ratio(7, 10);
        if d.sub(face).raw() <= reach_of(me).raw() && level {
            self.intent = ANCHOR;
            let swing = self.swing(me);
            // Keep pressed against it: a shrug or a lurch moves the floor.
            return go(me, flat(at), at, swing);
        }
        self.intent = CROSS;
        // Toward it, hopping up each tread as it meets it: a full hop, held.
        let walk = go(me, flat(at), at, 0);
        // Below the crown with a tread to go: hop, every so often, while
        // walking at it.
        let below = me.pos.y.raw() < at.y.sub(Fx::from_int(2)).raw();
        if me.grounded && (self.stuck > 2 || (below && w.frame % 30 == 0)) {
            self.hold_jump = 18;
            self.held = walk;
            return walk.with(Input::SPACE);
        }
        walk
    }
}

/// Is a point on a grate that is hissing or blowing: which way off it.
fn on_a_grate(w: &World, at: V3) -> Option<V3> {
    let floor = w.terrain().floor;
    for h in floor.iter() {
        let hot = h.kind == fight::BLAST || h.state != 0;
        if !hot {
            continue;
        }
        let r = h.radius.add(Fx::ratio(8, 10));
        if wide_flat_dist(h.a, at).raw() <= r.raw()
            && at.y.sub(h.a.y).abs().raw() < Fx::from_int(2).raw()
        {
            return Some(unit(at.sub(h.a)));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// The report's own lines (§9)
// ---------------------------------------------------------------------------

/// What the Siegeshell's report adds.
#[derive(Default)]
pub struct Lines {
    frames: u32,
    /// Distance from the siege line when each anchor broke, in metres.
    anchor_at: Vec<f32>,
    arrived: Option<u32>,
    beams: u32,
    cancelled: u32,
    breaches: u32,
    /// Beats a fighter on the floor stood in the reach of, and how many of
    /// those caught them.
    beats_exposed: u32,
    beats_caught: u32,
    stamps: u32,
    stamps_landed: u32,
    drags: u32,
    drags_landed: u32,
    stumbles: u32,
    buckles: u32,
    openings: u32,
    openings_used: u32,
    climbs: u32,
    reached_crown: u32,
    aboard_frames: u32,
    hunter_frames: u32,
    thrown: u32,
    fell: u32,
    parasites_killed: u32,
    /// The windows, by where the hunter is: floor and aboard.
    ground: [u32; 5],
    shell: [u32; 5],
    /// Unanswerable, by kind: a stamp with no disc drawn, a ring on a
    /// helpless body, two channels' hits inside the gap.
    no_disc: u32,
    ring_helpless: u32,
    two_channels: u32,
    /// Frames since a legs' or ring hit and a body hit, per fighter.
    since_leg: [u32; MAX_PLAYERS],
    since_body: [u32; MAX_PLAYERS],
    ring_hunters: u32,
    open_was: Option<usize>,
    open_hit: bool,
    stood_crown: [bool; MAX_PLAYERS],
    climbing: [bool; MAX_PLAYERS],
}

impl Lines {
    fn band(frames: i32) -> usize {
        crate::report::Threat::of(frames) as usize
    }
}

pub fn lines() -> Box<dyn Tally> {
    Box::new(Lines {
        since_leg: [u32::MAX / 2; MAX_PLAYERS],
        since_body: [u32::MAX / 2; MAX_PLAYERS],
        ..Lines::default()
    })
}

/// The soonest the creature can hit a fighter on the floor at `at`: the
/// legs' move, the next ring that reaches it, the body's floor moves.
fn ground_free(w: &World, m: &Monster, at: V3) -> u16 {
    let mut free = u16::MAX;
    let c = fight::leg::get(m);
    if c.kind != fight::leg::NONE && c.phase <= fight::leg::ACTIVE {
        free = 0;
    }
    if let Some(due) = ring_due(m, at) {
        free = free.min((due + REACTION as i32).clamp(0, u16::MAX as i32) as u16);
    }
    match m.doing {
        Doing::Startup { kind, .. } | Doing::Active { kind, .. }
            if kind == ss::SHED || kind == ss::PLOUGH =>
        {
            free = 0
        }
        _ => {}
    }
    let _ = w;
    free.min(
        m.frames_until_free()
            .saturating_add(gait::frames_per_beat(m).clamp(0, 600) as u16),
    )
}

/// The soonest the shell can hit a rider: its body's moves, the grates.
fn shell_free(m: &Monster) -> u16 {
    match m.doing {
        Doing::Startup { kind, .. } | Doing::Active { kind, .. }
            if kind == SHRUG || kind == SHIVER =>
        {
            0
        }
        _ => m.frames_until_free(),
    }
}

impl Tally for Lines {
    fn observe(&mut self, before: &World, after: &World) {
        let Some(slot) = fight::slot_of(after) else {
            return;
        };
        let (Some(was), Some(now)) = (before.monsters[slot], after.monsters[slot]) else {
            return;
        };
        self.frames += 1;
        // The clock: anchors and the siege.
        let gone = |m: &Monster| fight::anchors_broken(m);
        if gone(&now) > gone(&was) {
            let left = fight::to_wall(after, &now).sub(Knob::SiegeLine.fx());
            self.anchor_at.push(left.to_f32_for_render().max(0.0));
            if matches!(was.doing, Doing::Startup { kind: ss::BEAM, .. }) {
                self.cancelled += 1;
            }
        }
        if fight::at_siege_line(&now) && self.arrived.is_none() {
            self.arrived = Some(after.frame);
        }
        if let (Doing::Active { kind: ss::BEAM, .. }, false) = (
            now.doing,
            matches!(was.doing, Doing::Active { kind: ss::BEAM, .. }),
        ) {
            self.beams += 1;
        }
        self.breaches = fight::beams_fired(after);
        // Stumbles and buckles.
        if matches!(now.doing, Doing::Stumble { .. }) && !matches!(was.doing, Doing::Stumble { .. })
        {
            self.stumbles += 1;
        }
        for l in 0..LEG_COUNT {
            if fight::buckles(&now, l) > fight::buckles(&was, l) {
                self.buckles += 1;
            }
        }
        // The Opening, and whether an open anchor was struck.
        let open = fight::open_anchor(&now);
        if open.is_some() && self.open_was.is_none() {
            self.openings += 1;
            self.open_hit = false;
        }
        if let Some(a) = open {
            let part = ss::anchor_part(a);
            if now.part_health(part) < was.part_health(part) && !self.open_hit {
                self.open_hit = true;
                self.openings_used += 1;
            }
        }
        self.open_was = open;
        // The legs' channel.
        let (cw, cn) = (fight::leg::get(&was), fight::leg::get(&now));
        let begun = cn.kind != fight::leg::NONE
            && cn.phase == fight::leg::STARTUP
            && (cw.kind == fight::leg::NONE || cw.phase != fight::leg::STARTUP);
        if begun {
            if cn.kind == fight::leg::STAMP {
                self.stamps += 1;
            } else {
                self.drags += 1;
            }
        }
        // A stamp under way with no disc drawn for it: a marker bug, and an
        // unanswerable hit if it lands.
        let stamp_drawn = cn.kind != fight::leg::STAMP
            || cn.phase >= fight::leg::RECOVERY
            || after
                .signs()
                .iter()
                .any(|s| s.shape == Shape::Disc && s.covers(fight::leg::aim(&now)));
        // Who was hit by what, this frame.
        let ring = fight::ring_live(after);
        let (_, age, _) = fight::ring(after);
        for i in 0..MAX_PLAYERS {
            let (pb, pa) = (&before.players[i], &after.players[i]);
            if pa.health <= 0 && pb.health <= 0 {
                continue;
            }
            let lost = pb.health - pa.health;
            let leg_hit = lost > 0
                && cn.kind != fight::leg::NONE
                && cn.phase == fight::leg::ACTIVE
                && cn.struck & (1 << i) != 0
                && cw.struck & (1 << i) == 0;
            let ring_hit = lost > 0 && (ring || age == 0) && !pb.aboard();
            let body_hit = lost > 0 && !was.hit_used && now.hit_used;
            if leg_hit {
                if cn.kind == fight::leg::STAMP {
                    self.stamps_landed += 1;
                    if !stamp_drawn {
                        self.no_disc += 1;
                    }
                } else {
                    self.drags_landed += 1;
                }
            }
            if ring_hit {
                self.beats_caught += 1;
                if pb.action.stunned() {
                    self.ring_helpless += 1;
                }
            }
            if leg_hit || ring_hit {
                if self.since_body[i] <= Knob::ChannelGap.raw().max(0) as u32 {
                    self.two_channels += 1;
                }
                self.since_leg[i] = 0;
            } else {
                self.since_leg[i] = self.since_leg[i].saturating_add(1);
            }
            if body_hit {
                if self.since_leg[i] <= Knob::ChannelGap.raw().max(0) as u32
                    && !leg_hit
                    && !ring_hit
                {
                    self.two_channels += 1;
                }
                self.since_body[i] = 0;
            } else {
                self.since_body[i] = self.since_body[i].saturating_add(1);
            }
            if pa.health <= 0 {
                continue;
            }
            // Aboard, climbs, the crown, throws and falls.
            self.hunter_frames += 1;
            if pa.aboard() {
                self.aboard_frames += 1;
                if !pb.aboard() {
                    self.climbs += 1;
                    self.climbing[i] = true;
                    self.stood_crown[i] = false;
                }
                if !self.stood_crown[i]
                    && matches!(mind::region_of(mount_part(pa.mount)), mind::Region::Crown)
                {
                    self.stood_crown[i] = true;
                    self.reached_crown += 1;
                }
            } else if pb.aboard() {
                if matches!(pa.action, sim::state::Action::HitStun { .. }) {
                    self.thrown += 1;
                } else {
                    self.fell += 1;
                }
            }
            // The windows, by where the fighter is.
            if pa.aboard() {
                self.shell[Lines::band(shell_free(&now) as i32)] += 1;
            } else {
                self.ground[Lines::band(ground_free(after, &now, pa.pos) as i32)] += 1;
            }
        }
        // The exposure to each beat: fighters on the floor in a landing
        // tripod's ring's reach when it lands.
        if age == 0 {
            let (t, _, _) = fight::ring(after);
            let reach = Knob::RingTo.fx().add(sim::tuning::body_radius());
            for p in after.players.iter().filter(|p| p.health > 0 && !p.aboard()) {
                if fight::tripod_legs(t)
                    .any(|l| wide_flat_dist(gait::foot(&now, l), p.pos).raw() <= reach.raw())
                {
                    self.beats_exposed += 1;
                }
            }
            self.ring_hunters += 1;
        }
        // Parasites killed.
        for i in 0..sim::critter::MAX_CRITTERS {
            if before.critters[i].alive()
                && !after.critters[i].alive()
                && after.critters[i].present()
            {
                self.parasites_killed += 1;
            }
        }
    }

    fn lines(&self) -> Vec<(String, String, String)> {
        let pct = |a: u32, b: u32| {
            if b == 0 {
                0.0
            } else {
                100.0 * a as f32 / b as f32
            }
        };
        let win = |w: &[u32; 5]| {
            let t: u32 = w.iter().sum();
            format!(
                "{:.0} / {:.0} / {:.0} / {:.0} %",
                pct(w[0], t),
                pct(w[1], t),
                pct(w[2], t),
                pct(w[3], t)
            )
        };
        let mmss = |f: u32| format!("{}:{:02}", f / 3600, (f / 60) % 60);
        vec![
            (
                "distance left at each anchor".into(),
                if self.anchor_at.is_empty() {
                    "none broken".into()
                } else {
                    self.anchor_at
                        .iter()
                        .map(|d| format!("{d:.0} m"))
                        .collect::<Vec<_>>()
                        .join(" / ")
                },
                "the clock, read where each anchor fell".into(),
            ),
            (
                "arrived at the siege line".into(),
                self.arrived.map_or("never".into(), mmss),
                "untouched it arrives in about five minutes".into(),
            ),
            (
                "beams fired / cancelled".into(),
                format!("{} / {}", self.beams, self.cancelled),
                "an anchor broken in the tell cancels it".into(),
            ),
            (
                "wall breaches".into(),
                format!("{} of 2", self.breaches),
                "the second loses the hunt".into(),
            ),
            (
                "beats: exposed / caught".into(),
                format!("{} / {}", self.beats_exposed, self.beats_caught),
                "a ring that reached a fighter on the floor; jumped is the rest".into(),
            ),
            (
                "stamps landed / thrown".into(),
                format!("{} / {}", self.stamps_landed, self.stamps),
                "out of the pad at the drop".into(),
            ),
            (
                "drags landed / thrown".into(),
                format!("{} / {}", self.drags_landed, self.drags),
                "round the outside of the leg".into(),
            ),
            (
                "stumbles (buckles)".into(),
                format!("{} ({})", self.stumbles, self.buckles),
                "two ankles on a side, then every buckle there".into(),
            ),
            (
                "openings earned / used".into(),
                format!("{} / {}", self.openings, self.openings_used),
                "a stumble with somebody at an anchor".into(),
            ),
            (
                "climbs: started / reached crown".into(),
                format!("{} / {}", self.climbs, self.reached_crown),
                "the way up is the stumble's stair or the kneel".into(),
            ),
            (
                "time aboard".into(),
                format!("{:.0}%", pct(self.aboard_frames, self.hunter_frames)),
                "of the hunters' living frames".into(),
            ),
            (
                "thrown / fell".into(),
                format!("{} / {}", self.thrown, self.fell),
                "off the shell, by a move or by walking off".into(),
            ),
            (
                "parasites killed".into(),
                format!("{}", self.parasites_killed),
                "the shell's population".into(),
            ),
            (
                "windows, ground".into(),
                win(&self.ground),
                "threatening / poke / way in / walk up, on the floor".into(),
            ),
            (
                "windows, aboard".into(),
                win(&self.shell),
                "the same, on the shell".into(),
            ),
            (
                "unanswerable, by kind".into(),
                format!(
                    "{} stamp unmarked, {} ring on the helpless, {} two channels",
                    self.no_disc, self.ring_helpless, self.two_channels
                ),
                "§6: each must be zero".into(),
            ),
        ]
    }

    fn unanswerable(&self) -> u32 {
        self.no_disc + self.ring_helpless + self.two_channels
    }
}

/// The Siegeshell's card: one plan, its role by which hunter it drives --
/// the first breaks the legs (and climbs, alone), the second is the crown.
pub static CARD: crate::plans::Card = crate::plans::Card {
    species: sim::species::SpeciesId::SIEGESHELL,
    plan: solo,
    bucks: |kind| kind == SHRUG || kind == SHIVER,
    words: crate::plans::Words {
        weak_hits: "weak-point hits",
        broken: "ankles and anchors broken",
        into_breakables: "damage into ankles and anchors",
        into_breakables_why: "the legs bring it low, the anchors kill it",
        worst: "worst part",
        ride_for: "the crown",
        toppled_pool: "off a pool under its kneel",
    },
    tally: Some(lines),
    gamble: None,
};
