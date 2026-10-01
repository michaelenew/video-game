//! The Mantis's hunter: the duellist's plan, its three ablations, and the
//! report lines the fight is measured by (`mantis.md` §9).
//!
//! **The plan** -- what a person learns in the first ten minutes, in the
//! order they learn it:
//!
//! 1. *Its front is a wall.* Stand at five to six metres in neutral; inside
//!    four only on a window.
//! 2. *Don't blink under the coil.* While it is coiled, walk across the lane
//!    and reverse as the coil runs long; never dodge, swing or jump.
//! 3. *Hit the recoveries.* After a missed lunge, a leap's landing, the
//!    pair's recovery, a guard lowering -- and choose arm or body: the arm
//!    until one blade is broken.
//! 4. *Break the guard it just raised.* The guard breaker only within twenty
//!    frames of the guard going up.
//! 5. *Don't repeat yourself.* Never the same move into its guard three times
//!    in four; on Ready, go round or wait.
//! 6. *Stay on the floor* within ten metres. On prayer: break it if in
//!    reach, else step out past ten metres and wait for the pips.
//! 7. After a broken blade, circle to the broken side.
//!
//! **The three ablations** (`MANTIS_PLAN=repeater|jumper|dodger`): the
//! repeater opens with one move always; the jumper jumps in from six metres;
//! the dodger dodges the coil. The duellist's plan must beat all three. With
//! `MANTIS_HABIT=off` the creature's memory is off (the Oven flag
//! `Mantis · habit`), for the fourth comparison §9 asks for.
//!
//! **What it sees** is what is drawn: the creature's body and stance, its
//! floor markers, the dive's circle and the notches on its blades -- the
//! world as it was `REACTION` frames ago. It never reads the creature's mind:
//! it does not know which move the brain will pick, only what has begun.

use sim::fixed::Fx;
use sim::math::{atan2_turns, wide_flat_dist, wrap_turns};
use sim::monster::{Doing, Monster};
use sim::species::mantis::{self, Knob, fight, habit};
use sim::state::{Action, Phase, Player};
use sim::{Class, Input, V3, World};

use crate::duel::Kit;
use crate::report::Tally;
use crate::{Hunter, Intent, Plan, REACTION, steer, turns_to_aim};

/// At its standoff, outside its reach.
pub const SPACE: Intent = Intent("Space");
/// Walking across a coil's lane.
pub const ACROSS: Intent = Intent("Across");
/// Out of a marker or a reach.
pub const OUT: Intent = Intent("Out");
/// Hitting a recovery.
pub const PUNISH: Intent = Intent("Punish");
/// Breaking the guard it just raised.
pub const BREAK: Intent = Intent("Break");
/// Feeding its guard something to raise it against.
pub const PRESS: Intent = Intent("Press");
/// Going round its guard, or its Ready.
pub const ROUND: Intent = Intent("Round");
/// Under the pivot cut.
pub const DUCK: Intent = Intent("Duck");
/// Away from the prayer, or into it with a breaker.
pub const PRAY: Intent = Intent("Prayer");
/// The ablations' own mistakes.
pub const JUMP_IN: Intent = Intent("JumpIn");
pub const DODGE: Intent = Intent("Dodge");

/// Which plan it plays.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Style {
    Duellist,
    /// Steps 1-4 and 6, one opening move always.
    Repeater,
    /// Jumps in from six metres.
    Jumper,
    /// Dodges the coil.
    Dodger,
}

impl Style {
    /// From `MANTIS_PLAN`, the duellist without it.
    pub fn from_env() -> Style {
        match std::env::var("MANTIS_PLAN").as_deref() {
            Ok("repeater") => Style::Repeater,
            Ok("jumper") => Style::Jumper,
            Ok("dodger") => Style::Dodger,
            _ => Style::Duellist,
        }
    }
}

/// Where it stands in neutral: outside its four metres.
const STANDOFF: Fx = Fx::ratio(6, 1);
/// Inside this is inside its scythes.
const INSIDE: Fx = Fx::ratio(45, 10);
/// How many frames early or late its timed presses can be.
/// One frame in this many, standing off a free Mantis, it presses in.
const PRESS_EVERY: u32 = 120;
/// Frames between its presses: it does not mash.
const PRESS_GAP: u16 = 8;
/// A guard raised this recently is one to break (§9.4).
const FRESH_GUARD: i32 = 20;
/// Further than this from its prayer is out of it (§9.6).
const PRAYER_CLEAR: Fx = Fx::ratio(105, 10);
/// How fast it turns its camera.
const TURN: Fx = Fx::ratio(3, 2);
/// How many frames before the coil runs out it changes direction, in
/// its own late view.
const LATE_TURN: i32 = 12;
/// Frames of a hop held.
const HOP_HOLD: u16 = 20;

/// What it can see, one frame of it.
#[derive(Clone, Copy)]
struct Seen {
    mantis: Option<Monster>,
    /// The memory, as the notches show it: newest first.
    notches: [Option<habit::Entry>; habit::DEPTH],
    dive: V3,
    frame: u32,
}

pub struct Duellist {
    who: usize,
    style: Style,
    memory: Vec<Seen>,
    at: usize,
    filled: usize,
    intent: Intent,
    look: Fx,
    looked: bool,
    kit: Option<Kit>,
    gap: u16,
    hop_left: u16,
    /// Which way it is circling: +1 or -1.
    circle: i32,
    circle_left: u16,
    /// The coil it is walking across: which way, and whether it has reversed.
    across: Option<(V3, bool, u32)>,
    /// The guard it saw go up, and when (its frame, seen).
    guard_seen: Option<u32>,
    rng: u32,
    slop: i32,
    hop: Fx,
}

impl Duellist {
    pub fn new(who: usize, seed: u32, hop: Fx) -> Duellist {
        Self::styled(who, seed, hop, Style::from_env())
    }

    pub fn styled(who: usize, seed: u32, hop: Fx, style: Style) -> Duellist {
        if std::env::var("MANTIS_HABIT").as_deref() == Ok("off") {
            let sp = &mantis::SPECIES;
            sim::oven::set_species_raw(
                sp.id,
                sim::species::Common::ALL.len() + Knob::Habit as usize,
                0,
            );
        }
        Duellist {
            who,
            style,
            memory: Vec::new(),
            at: 0,
            filled: 0,
            intent: SPACE,
            look: Fx::ZERO,
            looked: false,
            kit: None,
            gap: 0,
            hop_left: 0,
            circle: if seed & 2 == 0 { 1 } else { -1 },
            circle_left: 120,
            across: None,
            guard_seen: None,
            rng: (0x9E37_79B9
                ^ (who as u32).wrapping_mul(0x85EB_CA6B)
                ^ seed.wrapping_mul(0x27D4_EB2F))
                | 1,
            slop: 0,
            hop,
        }
    }

    fn roll(&mut self) -> u32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        self.rng
    }

    fn recall(&self) -> Option<Seen> {
        if self.filled == 0 {
            return None;
        }
        let back = REACTION.min(self.filled - 1);
        let len = self.memory.len();
        Some(self.memory[(self.at + len - 1 - back) % len])
    }
}

pub fn make(who: usize, seed: u32, hop: Fx) -> Box<dyn Plan + Send + Sync> {
    Box::new(Duellist::new(who, seed, hop))
}

fn flat(v: V3) -> V3 {
    V3::new(v.x, Fx::ZERO, v.z)
}

fn unit(v: V3, fallback: V3) -> V3 {
    let v = flat(v);
    if v.flat_len().raw() > 0 {
        v.normalized()
    } else {
        fallback
    }
}

fn yaw_of(v: V3) -> Fx {
    atan2_turns(v.z, v.x)
}

/// The middle of its thorax: where it is looked at, and swung at.
pub fn chest(m: &Monster) -> V3 {
    let sh = mantis::SPECIES.shape(mantis::THORAX_PART);
    let mid = sh.min.add(sh.max).scale(crate::HALF);
    m.world_of(mantis::THORAX_PART, mid)
}

/// The middle of one of its blades.
fn blade_at(m: &Monster, part: usize) -> V3 {
    let sh = mantis::SPECIES.shape(part);
    let mid = sh.min.add(sh.max).scale(crate::HALF);
    m.world_of(part, mid)
}

/// The look as the wire carries it: level while it moves, the crosshair put
/// on a point while it throws something.
fn wire(me: &Player, yaw: Fx, at: V3, bits: u16) -> Input {
    let aim = turns_to_aim(yaw.sub(me.carry_yaw));
    let throwing = Input::LEFT
        | Input::RIGHT
        | Input::MIDDLE
        | Input::MECHANIC
        | Input::SPECIAL
        | Input::KEY_F;
    let pitch = if bits & throwing != 0 {
        sim::aim::look_onto(me.pos, aim, me.aloft, at)
    } else {
        0
    };
    Input::looking_at(bits, aim, pitch)
}

/// Inside the court, and off its wall and its columns: a direction that
/// would take it into one is turned along it.
fn keep_in(w: &World, at: V3, dir: V3) -> V3 {
    let ground = w.terrain();
    let ahead = at.add(dir.scale(Fx::from_int(2)));
    let b = w.arena().bounds;
    let room = Fx::from_int(3);
    let out = ahead.x.raw() < b.lo_x.add(room).raw()
        || ahead.x.raw() > b.hi_x.sub(room).raw()
        || ahead.z.raw() < b.lo_z.add(room).raw()
        || ahead.z.raw() > b.hi_z.sub(room).raw()
        || ground.ground_under(ahead).raw() > at.y.add(Fx::ratio(3, 10)).raw();
    if !out {
        return dir;
    }
    let across = V3::new(dir.z.neg(), Fx::ZERO, dir.x);
    let middle = flat(V3::ZERO.sub(at));
    let side = if across.dot(middle).raw() >= 0 {
        across
    } else {
        across.scale(Fx::ONE.neg())
    };
    unit(side.add(unit(middle, side).scale(crate::HALF)), side)
}

/// **How much floor there is that way**, in metres up to six: nothing
/// standing in it, and inside the walls.
fn room(w: &World, at: V3, dir: V3) -> Fx {
    if dir.flat_len().raw() == 0 {
        return Fx::ZERO;
    }
    let ground = w.terrain();
    let b = w.arena().bounds;
    let edge = Fx::ratio(15, 10);
    let mut clear = Fx::ZERO;
    for half_metres in 1..=12 {
        let step = Fx::from_int(half_metres).mul(crate::HALF);
        let p = at.add(dir.scale(step));
        let inside = p.x.raw() > b.lo_x.add(edge).raw()
            && p.x.raw() < b.hi_x.sub(edge).raw()
            && p.z.raw() > b.lo_z.add(edge).raw()
            && p.z.raw() < b.hi_z.sub(edge).raw();
        if !inside || ground.ground_under(p).raw() > at.y.add(Fx::ratio(3, 10)).raw() {
            break;
        }
        clear = step;
    }
    clear
}

/// Frames into the move it is in, through its startup and active window.
fn elapsed(m: &Monster) -> Option<(u8, i32)> {
    fight::elapsed(m)
}

/// The breaker each class carries, by its button: the versus rule's own
/// flag on the move it throws. The Champion's is the third link of a chain
/// (its hammer), which is handled as a chain.
fn breaker(class: Class) -> Option<u16> {
    match class {
        Class::Bulwark => Some(Input::SPECIAL),
        Class::Elementalist => Some(Input::KEY_F),
        Class::BloodMage => Some(Input::SPECIAL),
        _ => None,
    }
}

impl Plan for Duellist {
    fn watch(&mut self, w: &World) {
        let m = w.monsters[0].filter(|m| m.species == mantis::SPECIES.id);
        let seen = Seen {
            mantis: m,
            notches: habit::entries(&w.lore),
            dive: fight::dive_at(&w.lore),
            frame: w.frame,
        };
        if self.memory.len() < REACTION + 1 {
            self.memory.push(seen);
            self.at = self.memory.len() % (REACTION + 1);
        } else {
            self.memory[self.at] = seen;
            self.at = (self.at + 1) % self.memory.len();
        }
        self.filled = (self.filled + 1).min(REACTION + 1);
    }

    fn act(&mut self, w: &World) -> Input {
        self.gap = self.gap.saturating_sub(1);
        let me = w.players[self.who];
        if self.kit.is_none() {
            self.kit = Some(Kit::learn(me.class));
        }
        if !self.looked {
            self.look = yaw_of(me.facing);
            self.looked = true;
        }
        if me.health <= 0 || !matches!(w.phase, Phase::Fighting) {
            return Input::default();
        }
        let Some(seen) = self.recall() else {
            return Input::default();
        };
        let Some(m) = seen.mantis.filter(|m| m.alive()) else {
            return Input::default();
        };
        if self.hop_left > 0 {
            self.hop_left -= 1;
            let to = unit(m.pos.sub(me.pos), V3::from_turns(self.look));
            return self.turn_and(&me, Some(chest(&m)), to, Input::SPACE);
        }
        // Track the guard going up: its first frame, seen.
        match m.doing {
            Doing::Active {
                kind: mantis::GUARD,
                left,
            } => {
                let held = mantis::SPECIES
                    .attack(mantis::GUARD)
                    .active
                    .saturating_sub(left) as u32;
                if self.guard_seen.is_none() {
                    self.guard_seen = Some(seen.frame.saturating_sub(held));
                }
            }
            _ => self.guard_seen = None,
        }
        if std::env::var_os("MANTIS_DEBUG").is_some() {
            eprintln!(
                "{} me ({:.1},{:.1}) hp {} {:?} c{} | it ({:.1},{:.1}) {:?} hp {} d {:.1} | {:?}",
                w.frame,
                me.pos.x.to_f32_for_render(),
                me.pos.z.to_f32_for_render(),
                me.health,
                me.action,
                self.chain_depth(&me),
                m.pos.x.to_f32_for_render(),
                m.pos.z.to_f32_for_render(),
                m.doing,
                m.health,
                wide_flat_dist(m.pos, me.pos).to_f32_for_render(),
                self.intent
            );
        }
        if let Some(input) = self.answer(w, &me, &m, &seen) {
            return input;
        }
        if let Some(input) = self.opening(w, &me, &m, &seen) {
            return input;
        }
        self.neutral(w, &me, &m, &seen)
    }

    fn intent(&self) -> Intent {
        self.intent
    }
}

impl Duellist {
    fn turn_and(&mut self, me: &Player, toward: Option<V3>, dir: V3, bits: u16) -> Input {
        if let Some(at) = toward {
            let want = yaw_of(flat(at.sub(me.pos)));
            let error = wrap_turns(want.sub(self.look));
            let step = TURN.mul(sim::DT);
            self.look = self.look.add(error.clamp(step.neg(), step));
        }
        let at = toward.unwrap_or(me.pos.add(V3::from_turns(self.look).scale(Fx::from_int(8))));
        let mut input = wire(me, self.look, at, bits);
        if dir.flat_len().raw() > 0 {
            input.bits |= steer(self.look, dir);
        }
        input
    }

    /// Facing it closely enough to swing at it.
    fn facing(&self, me: &Player, at: V3) -> bool {
        let want = yaw_of(flat(at.sub(me.pos)));
        wrap_turns(want.sub(self.look)).abs().raw() <= Fx::ratio(1, 24).raw()
    }

    /// **The answers** to what it can see coming. None when nothing needs
    /// one.
    fn answer(&mut self, w: &World, me: &Player, m: &Monster, seen: &Seen) -> Option<Input> {
        let d = wide_flat_dist(m.pos, me.pos);
        let away = unit(
            me.pos.sub(m.pos),
            V3::from_turns(self.look).scale(Fx::ONE.neg()),
        );
        let (kind, e) = elapsed(m).unwrap_or((u8::MAX, 0));
        // The dive's circle, from the frame it leaves the floor: out of it.
        let leaping = matches!(
            m.doing,
            Doing::Active {
                kind: mantis::LEAP,
                ..
            } | Doing::Recovery {
                kind: mantis::LEAP,
                ..
            } | Doing::Startup {
                kind: mantis::DIVE,
                ..
            } | Doing::Active {
                kind: mantis::DIVE,
                ..
            }
        );
        if leaping {
            let r = mantis::SPECIES.attack(mantis::DIVE).hit_radius;
            let gap = wide_flat_dist(seen.dive, me.pos);
            if gap.raw() <= r.add(Fx::ONE).raw() {
                self.intent = OUT;
                let out = unit(me.pos.sub(seen.dive), away);
                let out = keep_in(w, me.pos, out);
                // Too late to walk: dodge out.
                let leap = mantis::SPECIES.attack(mantis::LEAP);
                let late = match m.doing {
                    Doing::Recovery { .. }
                    | Doing::Startup {
                        kind: mantis::DIVE, ..
                    }
                    | Doing::Active {
                        kind: mantis::DIVE, ..
                    } => true,
                    Doing::Active { left, .. } => (left as i32) < leap.active as i32 / 2,
                    _ => false,
                };
                let bits = if late && me.action.actionable() && me.grounded {
                    Input::SHIFT
                } else {
                    0
                };
                return Some(self.turn_and(me, Some(chest(m)), out, bits));
            }
            return None;
        }
        match kind {
            // The coil: commit to nothing; across the lane, reversing as it
            // runs long. The dodger dodges it.
            mantis::LUNGE if matches!(m.doing, Doing::Startup { .. }) => {
                if d.raw() > Fx::from_int(11).raw() {
                    return None;
                }
                self.intent = ACROSS;
                let lane = V3::from_turns(m.yaw);
                let side = V3::new(lane.z.neg(), Fx::ZERO, lane.x);
                let start = seen.frame.saturating_sub(e as u32);
                let (mut dir, mut flipped, began) = match self.across {
                    Some((dir, f, b)) if b == start => (dir, f, b),
                    _ => {
                        let span = 7;
                        self.slop = (self.roll() % span) as i32 - 3;
                        // Across it, whichever way has the floor for it.
                        let other = side.scale(Fx::ONE.neg());
                        let s = if room(w, me.pos, side).raw() >= room(w, me.pos, other).raw() {
                            side
                        } else {
                            other
                        };
                        (s, false, start)
                    }
                };
                if self.style == Style::Dodger && me.action.actionable() && me.grounded {
                    self.intent = DODGE;
                    return Some(self.turn_and(
                        me,
                        Some(chest(m)),
                        keep_in(w, me.pos, dir),
                        Input::SHIFT,
                    ));
                }
                // As the chitter peaks -- most of its longest coil gone --
                // change direction: what it saw you doing is now wrong.
                // Late enough that its eyes, which are late too, cannot
                // see the turn before it lets go.
                let peak = mantis::SPECIES.attack(mantis::LUNGE).startup as i32
                    - REACTION as i32
                    - LATE_TURN
                    + self.slop;
                if !flipped && e >= peak.max(0) {
                    // Back the other way if there is floor for it; stopped
                    // dead if not, which is a change of direction too.
                    let back = dir.scale(Fx::ONE.neg());
                    dir = if room(w, me.pos, back).raw() >= Fx::from_int(2).raw() {
                        back
                    } else {
                        V3::ZERO
                    };
                    flipped = true;
                }
                self.across = Some((dir, flipped, began));
                // Never along the lane: a wall in the way is a stop.
                let go = if room(w, me.pos, dir).raw() > crate::HALF.raw() {
                    dir
                } else {
                    V3::ZERO
                };
                return Some(self.turn_and(me, Some(chest(m)), go, 0));
            }
            // The scythes: straight out, pressing nothing.
            mantis::SLASH | mantis::SLASH_FAST | mantis::SLASH_HELD | mantis::COUNTER
                if matches!(m.doing, Doing::Startup { .. } | Doing::Active { .. }) =>
            {
                if d.raw() > INSIDE.add(Fx::ONE).raw() {
                    return None;
                }
                self.intent = OUT;
                return Some(self.turn_and(me, Some(chest(m)), keep_in(w, me.pos, away), 0));
            }
            // The pivot cut: crouch under it if behind it and near.
            mantis::PIVOT if matches!(m.doing, Doing::Startup { .. } | Doing::Active { .. }) => {
                if d.raw() > Fx::from_int(4).raw() {
                    return None;
                }
                self.intent = DUCK;
                return Some(self.turn_and(me, Some(chest(m)), V3::ZERO, Input::CROUCH));
            }
            // The flare: braced, then after it at once.
            mantis::FLARE if matches!(m.doing, Doing::Startup { .. } | Doing::Active { .. }) => {
                if d.raw() > Fx::from_int(4).raw() {
                    return None;
                }
                self.intent = DUCK;
                return Some(self.turn_and(me, Some(chest(m)), V3::ZERO, Input::CROUCH));
            }
            // The leap's drop: on the floor, and out of its rising cut.
            mantis::LEAP if matches!(m.doing, Doing::Startup { .. }) => {
                let r = mantis::SPECIES.attack(mantis::LEAP).hit_radius.add(Fx::ONE);
                if d.raw() > r.raw() {
                    return None;
                }
                self.intent = OUT;
                return Some(self.turn_and(me, Some(chest(m)), keep_in(w, me.pos, away), 0));
            }
            _ => {}
        }
        // The prayer: break it if a breaker reaches, otherwise out past ten
        // metres to wait for the pips.
        if let Doing::Active {
            kind: mantis::PRAYER,
            left,
        } = m.doing
        {
            if let Some(input) = self.break_it(w, me, m, left as i32) {
                return Some(input);
            }
            if d.raw() < PRAYER_CLEAR.raw() {
                self.intent = PRAY;
                return Some(self.turn_and(me, Some(chest(m)), keep_in(w, me.pos, away), 0));
            }
            self.intent = PRAY;
            return Some(self.turn_and(me, Some(chest(m)), V3::ZERO, 0));
        }
        // Hasted moves still to come: stay out until they are spent.
        if fight::pips(m) > 0 && d.raw() < INSIDE.add(Fx::ONE).raw() {
            self.intent = OUT;
            return Some(self.turn_and(me, Some(chest(m)), keep_in(w, me.pos, away), 0));
        }
        None
    }

    /// Throw this class's guard breaker, if it reaches the creature and there
    /// is time to land it inside `within` frames.
    fn break_it(&mut self, w: &World, me: &Player, m: &Monster, within: i32) -> Option<Input> {
        let bits = breaker(me.class)?;
        let kit = self.kit?;
        let tool = kit.iter().find(|t| t.bits == bits)?;
        let mv = sim::moves::get(me.class, tool.kind);
        if !mv.unblockable {
            return None;
        }
        let reach = mv
            .reach
            .add(mv.step)
            .add(sim::tuning::body_radius())
            .add(Fx::ratio(6, 10));
        let d = wide_flat_dist(m.pos, me.pos);
        let to = unit(m.pos.sub(me.pos), V3::from_turns(self.look));
        if within < mv.startup as i32 + REACTION as i32 / 2 {
            return None;
        }
        self.intent = BREAK;
        if d.raw() > reach.raw() {
            // Close in on it.
            return Some(self.turn_and(me, Some(chest(m)), keep_in(w, me.pos, to), 0));
        }
        if me.action.actionable() && self.gap == 0 && self.facing(me, chest(m)) {
            self.gap = PRESS_GAP;
            return Some(self.turn_and(me, Some(chest(m)), V3::ZERO, bits));
        }
        Some(self.turn_and(me, Some(chest(m)), V3::ZERO, 0))
    }

    /// **The openings**: a guard just raised to break, a guard to go round,
    /// a recovery to hit.
    fn opening(&mut self, w: &World, me: &Player, m: &Monster, seen: &Seen) -> Option<Input> {
        // 4. Break the guard it just raised. The Champion's breaker is the
        // third link of a string: the hammer, if it is two links in.
        if let (
            Doing::Active {
                kind: mantis::GUARD,
                left,
            },
            Some(up),
        ) = (m.doing, self.guard_seen)
        {
            let since = seen.frame.saturating_sub(up) as i32;
            if since <= FRESH_GUARD {
                let hold = Knob::GuardMinHold.raw() - since + REACTION as i32;
                if let Some(input) = self.break_it(w, me, m, hold.max(left as i32)) {
                    return Some(input);
                }
                if me.class == Class::Champion && self.chain_depth(me) >= 2 {
                    if let Some(input) = self.swing(me, chest(m), Input::MIDDLE) {
                        self.intent = BREAK;
                        return Some(input);
                    }
                }
            }
        }
        // A guard up and too old to break: round it, to its flank, and hit
        // what it does not cover. The repeater presses it anyway.
        if matches!(
            m.doing,
            Doing::Active {
                kind: mantis::GUARD,
                ..
            }
        ) && self.style != Style::Jumper
        {
            // A string under way into a guard its links raised: on with it,
            // the third link the breaker.
            let fresh = self
                .guard_seen
                .is_some_and(|up| (seen.frame.saturating_sub(up) as i32) <= FRESH_GUARD);
            if self.style == Style::Repeater
                || self.intent == PRESS && fresh && self.chain_depth(me) >= 1
            {
                return Some(self.press(w, me, m, seen));
            }
            return Some(self.round(w, me, m));
        }
        // 3. Hit the recoveries.
        let open = matches!(
            m.doing,
            Doing::Recovery { .. }
                | Doing::Flinch { .. }
                | Doing::Stumble { .. }
                | Doing::Toppled { .. }
        ) && !matches!(
            m.doing,
            Doing::Recovery {
                kind: mantis::PRAYER,
                ..
            }
        );
        if !open {
            return None;
        }
        // The present, as best it can tell: what it saw, less the frames it
        // has been seeing it late.
        let window = m.frames_until_free() as i32 - REACTION as i32;
        let poke = sim::moves::get(me.class, sim::state::SLOT_POKE);
        let busy = (poke.startup + poke.active) as i32;
        let reach = poke.reach.add(poke.step).add(sim::tuning::body_radius());
        // Arm until one blade is broken: the nearer whole blade, else the
        // body.
        let target = if fight::blades(m) == 2 {
            let l = blade_at(m, mantis::BLADE_L_PART);
            let r = blade_at(m, mantis::BLADE_R_PART);
            if wide_flat_dist(l, me.pos).raw() < wide_flat_dist(r, me.pos).raw() {
                l
            } else {
                r
            }
        } else {
            chest(m)
        };
        let d = wide_flat_dist(target, me.pos);
        let walk = (d.sub(reach).max(Fx::ZERO))
            .div(sim::tuning::move_speed().mul(sim::DT))
            .to_int();
        let dodge = sim::tuning::dodge_frames() as i32;
        let dash = sim::tuning::dodge_speed()
            .mul(sim::DT)
            .mul(Fx::from_int(dodge * 2 / 3));
        let to = unit(target.sub(me.pos), V3::from_turns(self.look));
        if d.raw() > reach.raw() {
            if window <= busy + walk.min(dodge + 4) + 2 {
                return None;
            }
            self.intent = PUNISH;
            // Far: close with a dodge, if one gets there with time to
            // swing; walking otherwise.
            let far = d.sub(reach).raw() > Fx::from_int(2).raw();
            if far
                && window > dodge + busy
                && me.action.actionable()
                && me.grounded
                && d.sub(reach).raw() < dash.add(Fx::ONE).raw()
                && self.gap == 0
            {
                self.gap = PRESS_GAP;
                return Some(self.turn_and(me, Some(target), keep_in(w, me.pos, to), Input::SHIFT));
            }
            if window <= busy + walk {
                return None;
            }
            return Some(self.turn_and(me, Some(target), keep_in(w, me.pos, to), 0));
        }
        if window <= busy {
            return None;
        }
        self.intent = PUNISH;
        // A string while there is time for it; the heavy into a long one.
        let heavy_busy = (sim::moves::get(me.class, sim::state::SLOT_COMMITTED).startup + 4) as i32;
        let bits = if window > heavy_busy + 30 && self.chain_depth(me) == 0 {
            crate::heavy(me.class)
        } else {
            Input::LEFT
        };
        Some(
            self.swing(me, target, bits)
                .unwrap_or_else(|| self.turn_and(me, Some(target), V3::ZERO, 0)),
        )
    }

    /// Swing `bits` at `at` if it is free to and facing it; `None` if not
    /// yet.
    fn swing(&mut self, me: &Player, at: V3, bits: u16) -> Option<Input> {
        let ready = me.action.actionable()
            || matches!(me.action, Action::Recovery { .. }) && me.class == Class::Champion;
        if ready && self.gap == 0 && self.facing(me, at) {
            self.gap = PRESS_GAP;
            return Some(self.turn_and(me, Some(at), V3::ZERO, bits));
        }
        None
    }

    /// How many links into a Champion's string it is.
    fn chain_depth(&self, me: &Player) -> u8 {
        match me.mechanic {
            sim::Mechanic::Forms { chain, .. } => chain,
            _ => 0,
        }
    }

    /// **Press its guard**: in to a poke's length and a string into its
    /// front -- something it has not seen three times in four -- with the
    /// Champion's hammer as the third link, which is Earthbreaker.
    fn press(&mut self, w: &World, me: &Player, m: &Monster, seen: &Seen) -> Input {
        self.intent = PRESS;
        let d = wide_flat_dist(m.pos, me.pos);
        let to = unit(m.pos.sub(me.pos), V3::from_turns(self.look));
        let poke = sim::moves::get(me.class, sim::state::SLOT_POKE);
        let reach = poke.reach.add(poke.step).add(sim::tuning::body_radius());
        if d.raw() > reach.raw() {
            return self.turn_and(me, Some(chest(m)), keep_in(w, me.pos, to), 0);
        }
        let bits = if me.class == Class::Champion && self.chain_depth(me) >= 2 {
            Input::MIDDLE
        } else {
            self.pick_press(me, seen)
        };
        self.swing(me, chest(m), bits)
            .unwrap_or_else(|| self.turn_and(me, Some(chest(m)), V3::ZERO, 0))
    }

    /// **Round its guard**: to a flank -- the broken side's, with a blade
    /// gone -- and swing at what its guard does not cover.
    fn round(&mut self, w: &World, me: &Player, m: &Monster) -> Input {
        self.intent = ROUND;
        let facing = V3::from_turns(m.yaw);
        let right = V3::from_turns(m.yaw.add(sim::math::QUARTER_TURN));
        let rel = flat(me.pos.sub(m.pos));
        let mut side = if right.dot(rel).raw() >= 0 {
            Fx::ONE
        } else {
            Fx::ONE.neg()
        };
        if fight::blades(m) == 1 {
            side = if m.broken(mantis::BLADE_L_PART) {
                Fx::ONE.neg()
            } else {
                Fx::ONE
            };
        }
        let poke = sim::moves::get(me.class, sim::state::SLOT_POKE);
        let reach = poke.reach.add(poke.step).add(sim::tuning::body_radius());
        // Beside it and a little behind: the flank, a poke's length out.
        let spot = m
            .pos
            .add(right.scale(side.mul(reach)))
            .sub(facing.scale(crate::HALF));
        if !m.covers(me.pos) && wide_flat_dist(m.pos, me.pos).raw() <= reach.add(Fx::ONE).raw() {
            if let Some(input) = self.swing(me, chest(m), Input::LEFT) {
                return input;
            }
        }
        let to = unit(spot.sub(me.pos), right.scale(side));
        // Round the outside of its reach, not through its front.
        let dir = if m.covers(me.pos) && wide_flat_dist(m.pos, me.pos).raw() < INSIDE.raw() {
            unit(to.add(right.scale(side)), to)
        } else {
            to
        };
        self.turn_and(me, Some(chest(m)), keep_in(w, me.pos, dir), 0)
    }

    /// **Neutral**: at its standoff, circling; pressing now and then with a
    /// string into its front -- something it has not shown it three times --
    /// to raise a guard worth breaking.
    fn neutral(&mut self, w: &World, me: &Player, m: &Monster, seen: &Seen) -> Input {
        let d = wide_flat_dist(m.pos, me.pos);
        let to = unit(m.pos.sub(me.pos), V3::from_turns(self.look));
        let around = V3::new(to.z.neg(), Fx::ZERO, to.x).scale(Fx::from_int(self.circle));
        self.circle_left = self.circle_left.saturating_sub(1);
        if self.circle_left == 0 {
            self.circle = -self.circle;
            self.circle_left = 90 + (self.roll() % 120) as u16;
        }
        // The jumper: in from six metres, over its guard.
        if self.style == Style::Jumper
            && self.hop.raw() > Fx::ONE.raw()
            && d.raw() <= Fx::from_int(7).raw()
            && d.raw() >= Fx::from_int(5).raw()
            && me.grounded
            && me.action.actionable()
            && self.roll() % 4 == 0
        {
            self.intent = JUMP_IN;
            self.hop_left = HOP_HOLD;
            return self.turn_and(me, Some(chest(m)), to, Input::SPACE);
        }
        // Ready against a move: go round, or wait. It does not throw that
        // move into it.
        let ready = matches!(
            m.doing,
            Doing::Active {
                kind: mantis::READY,
                ..
            } | Doing::Startup {
                kind: mantis::READY,
                ..
            }
        );
        if ready {
            self.intent = ROUND;
            if self.style == Style::Repeater {
                // The repeater throws its one move anyway.
                if d.raw() < Fx::from_int(3).raw() {
                    if let Some(input) = {
                        let bits = self.pick_press(me, seen);
                        self.swing(me, chest(m), bits)
                    } {
                        return input;
                    }
                }
            }
            let dir = keep_in(w, me.pos, unit(around.sub(to), around));
            return self.turn_and(me, Some(chest(m)), dir, 0);
        }
        // **Show it something to guard**: now and then, while it stands free,
        // in to a poke's length and a string into its front. The links it
        // sees raise its guard, and the guard it raised is the one to break.
        if self.style != Style::Jumper
            && m.doing.free()
            && (self.intent == PRESS || self.roll() % PRESS_EVERY == 0)
        {
            return self.press(w, me, m, seen);
        }
        // 1. Its front is a wall: hold at the standoff, circling.
        self.intent = SPACE;
        let radial = if d.raw() < STANDOFF.sub(crate::HALF).raw() {
            to.scale(Fx::ONE.neg())
        } else if d.raw() > STANDOFF.add(crate::HALF).raw() {
            to
        } else {
            V3::ZERO
        };
        // And toward the middle of the court when it has drifted out: a
        // back to the wall is a coil with nowhere to walk across it.
        let home = if flat(me.pos).flat_len().raw() > Fx::from_int(8).raw() {
            unit(V3::ZERO.sub(me.pos), V3::ZERO).scale(crate::HALF)
        } else {
            V3::ZERO
        };
        let dir = keep_in(
            w,
            me.pos,
            unit(radial.add(around.scale(crate::HALF)).add(home), around),
        );
        self.turn_and(me, Some(chest(m)), dir, 0)
    }

    /// **What to throw into its guard**: the repeater, the same thing every
    /// time; the duellist, whatever the notches hold fewest of.
    fn pick_press(&mut self, me: &Player, seen: &Seen) -> u16 {
        let options: [u16; 3] = match me.class {
            Class::Champion => [Input::LEFT, Input::RIGHT, Input::LEFT],
            _ => [Input::LEFT, crate::heavy(me.class), Input::LEFT],
        };
        if self.style == Style::Repeater {
            return options[0];
        }
        let kit = self.kit;
        let count = |bits: u16| {
            let Some(kind) = kit
                .and_then(|k| k.iter().find(|t| t.bits == bits))
                .map(|t| t.kind)
            else {
                return 0;
            };
            let code = habit::code(me.class, kind);
            seen.notches
                .iter()
                .flatten()
                .filter(|e| e.code == code)
                .count()
        };
        let start = (self.roll() % options.len() as u32) as usize;
        let mut best = options[start];
        for i in 0..options.len() {
            let o = options[(start + i) % options.len()];
            if count(o) < count(best) {
                best = o;
            }
        }
        best
    }
}

// ---------------------------------------------------------------------------
// The report
// ---------------------------------------------------------------------------

/// **Its report lines** (§9): what went into its guard and what came of it;
/// what it parried on sight; Ready; the coil; the air; guard breaks; blades;
/// counters; the second slash.
#[derive(Default)]
pub struct MantisTally {
    /// Per hunter move code: blocked, parried, broke.
    into_guard: Vec<(i32, [u32; 3])>,
    parried_on_sight: u32,
    parried_late: u32,
    ready_taken: u32,
    ready_right: u32,
    ready_wrong: u32,
    ready_out: u32,
    longest_run: u32,
    run_code: i32,
    run_len: u32,
    coils_seen: u32,
    coils_end: u32,
    landed_seen: u32,
    landed_end: u32,
    aloft_hits: u32,
    breaks_hold: u32,
    breaks_prayer: u32,
    breaks_missed: u32,
    blades_at: Vec<f32>,
    broken_side_damage: i32,
    counters: [u32; 4],
    second: [u32; 3],
    unanswerable: u32,
    /// The guarded window: frames its guard or prayer covered a hunter.
    pub guarded: u32,
    fought: u32,
    released_on_sight: bool,
    lunge_landed: bool,
}

impl MantisTally {
    pub fn boxed() -> Box<dyn Tally> {
        Box::new(MantisTally::default())
    }
}

fn name_of(code: i32) -> String {
    match habit::decode(code) {
        Some((class, kind)) => format!("{} {}", class.name(), sim::moves::get(class, kind).name),
        None => "?".to_string(),
    }
}

impl Tally for MantisTally {
    fn observe(&mut self, before: &World, after: &World) {
        let (Some(was), Some(now)) = (before.monsters[0], after.monsters[0]) else {
            return;
        };
        if now.species != mantis::SPECIES.id {
            return;
        }
        self.fought += 1;
        // What struck its guard this frame, from its own mailbox.
        for who in 0..sim::state::MAX_PLAYERS {
            if let Some((code, r)) = fight::mail(&now, who) {
                let slot = match self.into_guard.iter().position(|(c, _)| *c == code) {
                    Some(i) => i,
                    None => {
                        self.into_guard.push((code, [0; 3]));
                        self.into_guard.len() - 1
                    }
                };
                self.into_guard[slot].1[(r as usize).clamp(1, 3) - 1] += 1;
                if code == self.run_code {
                    self.run_len += 1;
                } else {
                    self.run_code = code;
                    self.run_len = 1;
                }
                self.longest_run = self.longest_run.max(self.run_len);
                if r == 2 {
                    if fight::flags(&was) & fight::flag::SAW_IT != 0 {
                        self.parried_on_sight += 1;
                    } else {
                        self.parried_late += 1;
                    }
                }
                if r == 3 {
                    match was.doing {
                        Doing::Active {
                            kind: mantis::PRAYER,
                            ..
                        } => self.breaks_prayer += 1,
                        _ => self.breaks_hold += 1,
                    }
                }
                if let Doing::Active {
                    kind: mantis::READY,
                    ..
                } = was.doing
                {
                    if r == 2 {
                        self.ready_right += 1;
                    }
                }
            }
        }
        // Ready: taken, and how it ended.
        let began = |k: u8| {
            matches!(now.doing, Doing::Startup { kind, .. } if kind == k)
                && !matches!(was.doing, Doing::Startup { kind, .. } if kind == k)
        };
        if began(mantis::READY) {
            self.ready_taken += 1;
        }
        if fight::flags(&was) & fight::flag::WRONG == 0
            && fight::flags(&now) & fight::flag::WRONG != 0
        {
            self.ready_wrong += 1;
        }
        if matches!(
            was.doing,
            Doing::Active {
                kind: mantis::READY,
                left: 0
            }
        ) && matches!(
            now.doing,
            Doing::Recovery {
                kind: mantis::READY,
                ..
            }
        ) {
            self.ready_out += 1;
        }
        // The guard dropped before a breaker it saw.
        if matches!(
            was.doing,
            Doing::Active {
                kind: mantis::GUARD,
                ..
            }
        ) && matches!(
            now.doing,
            Doing::Recovery {
                kind: mantis::GUARD,
                ..
            }
        ) && fight::flags(&now) & fight::flag::DROPPED != 0
        {
            self.breaks_missed += 1;
        }
        // The coil.
        if matches!(
            now.doing,
            Doing::Active {
                kind: mantis::LUNGE,
                ..
            }
        ) && matches!(
            was.doing,
            Doing::Startup {
                kind: mantis::LUNGE,
                ..
            }
        ) {
            self.released_on_sight = fight::flags(&now) & fight::flag::ON_SIGHT != 0;
            self.lunge_landed = false;
            if self.released_on_sight {
                self.coils_seen += 1;
            } else {
                self.coils_end += 1;
            }
        }
        // Blades.
        for part in [mantis::BLADE_L_PART, mantis::BLADE_R_PART] {
            if !was.broken(part) && now.broken(part) {
                self.blades_at.push(after.frame as f32 / 60.0);
            }
        }
        // Hits it landed, and on what.
        let hit = !was.hit_used && now.hit_used;
        if hit {
            if let Some(kind) = now.doing.attacking() {
                for i in 0..sim::state::MAX_PLAYERS {
                    let (b, a) = (&before.players[i], &after.players[i]);
                    if a.health >= b.health {
                        continue;
                    }
                    if !b.grounded {
                        self.aloft_hits += 1;
                    }
                    match kind {
                        mantis::LUNGE => {
                            if !self.lunge_landed {
                                self.lunge_landed = true;
                                if self.released_on_sight {
                                    self.landed_seen += 1;
                                } else {
                                    self.landed_end += 1;
                                }
                            }
                        }
                        mantis::COUNTER => {
                            let cause = fight::counter_cause(&now) as usize;
                            self.counters[cause.min(3)] += 1;
                            if cause == 0 {
                                self.unanswerable += 1;
                            }
                        }
                        mantis::SLASH_FAST | mantis::SLASH_HELD => {
                            let d = wide_flat_dist(now.pos, b.pos);
                            let pressed = matches!(
                                b.action,
                                Action::Startup { .. }
                                    | Action::Active { .. }
                                    | Action::Recovery { .. }
                            );
                            if pressed {
                                self.second[0] += 1;
                            } else if d.raw() <= Fx::from_int(4).raw() {
                                self.second[1] += 1;
                            } else {
                                self.second[2] += 1;
                                // Outside its reach, the shared rule has
                                // counted it already; inside it and over
                                // four metres is this rule's alone (§6).
                                let a = mantis::SPECIES.attack(kind);
                                let reach =
                                    a.hit_x.add(a.hit_radius).add(sim::tuning::body_radius());
                                if d.raw() <= reach.raw() {
                                    self.unanswerable += 1;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        // Damage from the broken side.
        if fight::blades(&now) == 1 {
            let lost = (was.health - now.health).max(0);
            if lost > 0 {
                let broken_left = now.broken(mantis::BLADE_L_PART);
                let right = V3::from_turns(now.yaw.add(sim::math::QUARTER_TURN));
                let side = after
                    .players
                    .iter()
                    .find(|p| p.health > 0)
                    .map(|p| right.dot(flat(p.pos.sub(now.pos))).raw() > 0);
                if side.is_some_and(|r| r != broken_left) {
                    self.broken_side_damage += lost;
                }
            }
        }
        // The guarded window.
        if after
            .players
            .iter()
            .any(|p| p.health > 0 && now.covers(p.pos))
        {
            self.guarded += 1;
        }
    }

    fn lines(&self) -> Vec<(String, String, String)> {
        let mut out = Vec::new();
        let mut into = self.into_guard.clone();
        into.sort_by_key(|(_, n)| std::cmp::Reverse(n.iter().sum::<u32>()));
        let list: Vec<String> = into
            .iter()
            .take(8)
            .map(|(c, n)| format!("{} {}/{}/{}", name_of(*c), n[0], n[1], n[2]))
            .collect();
        out.push((
            "into its guard".to_string(),
            if list.is_empty() {
                "nothing".to_string()
            } else {
                list.join(", ")
            },
            "per hunter move, blocked / parried / broke".to_string(),
        ));
        out.push((
            "parried".to_string(),
            format!(
                "{} on sight, {} otherwise",
                self.parried_on_sight, self.parried_late
            ),
            "a guard its eyes raised, against one it chose".to_string(),
        ));
        out.push((
            "ready".to_string(),
            format!(
                "taken {}, right {}, wrong {}, timed out {}",
                self.ready_taken, self.ready_right, self.ready_wrong, self.ready_out
            ),
            "the habit, against what was thrown".to_string(),
        ));
        out.push((
            "longest run of one move into guard".to_string(),
            self.longest_run.to_string(),
            "three is the habit's".to_string(),
        ));
        out.push((
            "coil".to_string(),
            format!(
                "released on a commitment {} (landed {}), at its end {} (landed {})",
                self.coils_seen, self.landed_seen, self.coils_end, self.landed_end
            ),
            "commit to nothing while it is coiled".to_string(),
        ));
        out.push((
            "hits taken aloft".to_string(),
            self.aloft_hits.to_string(),
            "stay on the floor".to_string(),
        ));
        out.push((
            "guard breaks".to_string(),
            format!(
                "in the hold {}, in prayer {}, missed (it dropped it) {}",
                self.breaks_hold, self.breaks_prayer, self.breaks_missed
            ),
            "break the guard it just raised".to_string(),
        ));
        let at: Vec<String> = self.blades_at.iter().map(|s| format!("{s:.0}s")).collect();
        out.push((
            "blades".to_string(),
            format!(
                "broken at {}; {} damage from the broken side",
                if at.is_empty() {
                    "-".to_string()
                } else {
                    at.join(", ")
                },
                self.broken_side_damage
            ),
            "go round the broken side".to_string(),
        ));
        out.push((
            "counters".to_string(),
            format!(
                "after a seen move {}, after Ready {}, after prayer {}, unanswered {}",
                self.counters[1], self.counters[2], self.counters[3], self.counters[0]
            ),
            "every counter traceable to a choice".to_string(),
        ));
        out.push((
            "second slash".to_string(),
            format!(
                "on a presser {}, on a stayer {}, other {}",
                self.second[0], self.second[1], self.second[2]
            ),
            "answered by the first: walk out, press nothing".to_string(),
        ));
        out.push((
            "guarded".to_string(),
            format!(
                "{:.0}%",
                100.0 * self.guarded as f32 / self.fought.max(1) as f32
            ),
            "its guard or prayer up and you inside it: a frontal hit wasted".to_string(),
        ));
        out
    }

    fn unanswerable(&self) -> u32 {
        self.unanswerable
    }
}

fn bucks(_kind: u8) -> bool {
    false
}

pub static CARD: crate::plans::Card = crate::plans::Card {
    species: sim::species::SpeciesId::MANTIS,
    plan: make,
    bucks,
    words: crate::plans::Words {
        weak_hits: "hits on a weak point",
        broken: "blades broken",
        into_breakables: "damage into its blades",
        into_breakables_why: "every whiff punish is a choice: the body, or the arm",
        worst: "the worst-hit blade took",
        ride_for: "a ride",
        toppled_pool: "under it",
    },
    tally: Some(MantisTally::boxed),
    gamble: None,
};

#[allow(dead_code)]
fn unused(_: &Hunter) {}
