//! The Pair's hunter: the plan a decent player follows, with a camera, and the
//! report lines the fight is measured by.
//!
//! **This hunter has a view** (`the-pair.md` §9, P8). Every other plan knows
//! where its creature is at all times; against two cats built around the
//! ninety degrees a third-person camera shows, that is a radar, and its win
//! rate would say nothing. So this one carries a camera yaw it turns at a
//! person's rate ([`TURN`]), and it perceives a cat -- where it is, what it has
//! begun -- only while the cat is inside that view (`sim::aim::in_view`) or
//! the move's floor marker is. A cat it cannot see is where it last saw it.
//! Like every plan it sees the world `REACTION` frames late and never reads
//! the creature's mind: a feint and a pounce are the same coil to it until the
//! tail flicks at frame ten.
//!
//! **The plan** (§9), what a person learns in the first ten minutes:
//!
//! 1. Strafe until both cats are in view, and keep them there; turn toward
//!    the last place the missing one was seen.
//! 2. Never dodge a coil without a tail flick; after a feint, look behind.
//! 3. Poke the Holder only in a recovery, and only while the Striker is in
//!    view.
//! 4. On an ambush lane, step sideways; on a pounce, dodge toward.
//! 5. On the twin pounce, stand still through the roar and dodge after the
//!    jump; unload into the crash.
//! 6. At the bond, hit the wounded one and swing on the interpose line.
//! 7. After the first kill, back off through the howl and fight the survivor
//!    face on.

use sim::aim::{Scene, in_view};
use sim::fixed::Fx;
use sim::math::{atan2_turns, flat_segment_gap, wide_flat_dist, wrap_turns};
use sim::monster::{Doing, MAX_MONSTERS, Monster};
use sim::species::pair::{self, Knob, fight};
use sim::state::{Action, Phase, Player};
use sim::{Input, V3, World};

use crate::report::{HALF_VIEW, Tally};
use crate::{Hands, Hunter, Intent, Plan, REACTION, heavy, steer, turns_to_aim};

/// Keeping both in view.
pub const WATCH: Intent = Intent("Watch");
/// Turning to find the one it cannot see.
pub const FIND: Intent = Intent("Find");
/// Out of a marker.
pub const EVADE: Intent = Intent("Evade");
/// The dodge toward a pounce, under the arc.
pub const UNDER: Intent = Intent("Under");
/// Still through the roar, then the late dodge.
pub const LATE: Intent = Intent("Late");
/// Over the tail.
pub const JUMP: Intent = Intent("Jump");
/// Hitting a cat that cannot answer.
pub const PUNISH: Intent = Intent("Punish");
/// Unloading into the crash.
pub const UNLOAD: Intent = Intent("Unload");
/// Under the lip of a perch.
pub const LIP: Intent = Intent("Lip");
/// Off through the howl.
pub const BACK: Intent = Intent("Back");
/// A stone in the Striker's path.
pub const STONE: Intent = Intent("Stone");

/// How fast it turns its camera: a person flicking a mouse, about a turn in a
/// second and a third.
pub const TURN: Fx = Fx::ratio(3, 4);
/// How many frames early or late its timed presses can be.
const SLOP_EARLY: i32 = 3;
const SLOP_LATE: i32 = 3;
/// Frames of jump held: a full hop.
const LEAP_HOLD: u16 = 24;
/// How far past a marker's edge counts as out of it.
const MARGIN: Fx = Fx::ratio(5, 10);
/// How near a leap's circle has to be to count as aimed at it: the circle
/// keeps following until the cat leaves the ground.
const FOLLOWS: Fx = Fx::from_int(3);
/// Where it likes to stand from the nearer cat: outside the paws.
const KEEP: Fx = Fx::ratio(50, 10);
/// Two cats wider apart than this, seen from it, is one too many to watch.
const SPREAD: Fx = Fx::ratio(18, 100);
/// A cat it has not seen for this long is one it goes looking for.
const STALE: u32 = 45;
/// Frames between swings.
const SWING_GAP: u16 = 18;
/// Frames of an opening kept back to get out in.
const EXIT: i32 = 8;
/// Frames after a dodge before another.
/// A free cat this near where you would stand to punish makes the punish
/// a walk into its rake.
const BESIDE: Fx = Fx::ratio(35, 10);
const DODGE_REST: u16 = 2;
/// The pounce's tail is up by this frame of its coil.
const FLICK: i32 = 13;
/// A cat last seen this far off is no threat to a swing at the other.
const FAR: Fx = Fx::from_int(9);
/// Frames between the Elementalist's stones.
const STONE_GAP: u16 = 240;

/// What it can see, one frame of it.
#[derive(Clone, Copy, Default)]
struct Seen {
    cats: [Option<Monster>; MAX_MONSTERS],
    frame: u32,
}

pub struct Pair {
    who: usize,
    memory: Vec<Seen>,
    at: usize,
    filled: usize,
    intent: Intent,
    /// Where its camera points, in world turns.
    look: Fx,
    looked: bool,
    /// Where each cat was when it last saw it, and on which frame.
    known: [Option<(V3, u32)>; MAX_MONSTERS],
    cooldown: u16,
    dodge_left: u16,
    leap_left: u16,
    leap_dir: V3,
    stone_left: u16,
    /// Frames to keep looking round after a feint.
    behind_left: u16,
    slop: i32,
    rng: u32,
    hop: Fx,
    /// Its class (`crate::class`), held to this camera.
    hands: Hands,
}

impl Pair {
    pub fn new(who: usize, seed: u32, hop: Fx) -> Pair {
        let mut p = Pair {
            who,
            memory: vec![Seen::default(); REACTION + 1],
            at: 0,
            filled: 0,
            intent: WATCH,
            look: Fx::ZERO,
            looked: false,
            known: [None; MAX_MONSTERS],
            cooldown: 0,
            dodge_left: 0,
            leap_left: 0,
            leap_dir: V3::ZERO,
            stone_left: 0,
            behind_left: 0,
            slop: 0,
            rng: (0x9E37_79B9
                ^ (who as u32).wrapping_mul(0x85EB_CA6B)
                ^ seed.wrapping_mul(0x27D4_EB2F))
                | 1,
            hop,
            hands: Hands::new(who, seed),
        };
        p.roll_slop();
        p
    }

    fn recall(&self) -> Seen {
        let back = REACTION.min(self.filled.saturating_sub(1));
        let idx = (self.at + self.memory.len() - 1 - back) % self.memory.len();
        self.memory[idx]
    }

    fn roll_slop(&mut self) {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        let span = (SLOP_LATE + SLOP_EARLY + 1) as u32;
        self.slop = (self.rng % span) as i32 - SLOP_EARLY;
    }
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

/// The middle of a cat, for looking at and for asking whether it is on
/// screen.
pub fn middle(m: &Monster) -> V3 {
    let sh = pair::SPECIES.shape(pair::BARREL);
    let mid = sh.min.add(sh.max).scale(crate::HALF);
    m.world_of(pair::BARREL, mid)
}

/// The scene a world stands in, for the view questions.
fn scene_of<'a>(
    w: &'a World,
    stones: &'a sim::stones::Field,
    ground: &'a sim::arena::Terrain,
) -> Scene<'a> {
    Scene {
        stones,
        players: &w.players,
        effects: &w.effects,
        quarry: &w.monsters,
        critters: &w.critters,
        arena: ground,
    }
}

/// The points of a cat's floor marker worth asking about: where it lands,
/// where its lane ends, and the nearest point of it to a body.
pub fn marker_points(m: &Monster, near: V3) -> Option<[V3; 3]> {
    let t = m.telegraph()?;
    let end = t.anchor.add(t.along.scale(t.sweep));
    let ab = flat(end.sub(t.anchor));
    let len = ab.flat_len();
    let closest = if len.raw() > 0 {
        let dir = ab.normalized();
        let along = flat(near.sub(t.anchor)).dot(dir).clamp(Fx::ZERO, len);
        t.anchor.add(dir.scale(along))
    } else {
        t.anchor
    };
    let toward =
        unit(near.sub(closest), V3::ZERO).scale(t.radius.min(wide_flat_dist(near, closest)));
    // On whatever the circle is drawn on: the floor, or a top.
    let up = |p: V3| V3::new(p.x, t.anchor.y, p.z);
    Some([up(t.anchor), up(end), up(closest.add(toward))])
}

/// Does this cat's floor marker cover a body standing here?
pub fn marker_covers(m: &Monster, at: V3, margin: Fx) -> bool {
    let Some(t) = m.telegraph() else {
        return false;
    };
    let end = t.anchor.add(t.along.scale(t.sweep));
    let gap = flat_segment_gap(at, t.anchor, end);
    gap.raw() <= t.radius.add(sim::tuning::body_radius()).add(margin).raw()
}

/// A look as the wire carries it: level while it is watching -- the floor
/// round its own feet on the screen, where the markers are -- and the
/// crosshair put on a point when it is throwing something at it.
fn wire(me: &Player, yaw: Fx, at: V3, bits: u16) -> Input {
    let aim = turns_to_aim(yaw.sub(me.carry_yaw));
    let throwing = Input::LEFT | Input::RIGHT | Input::MIDDLE | Input::MECHANIC | Input::SPECIAL;
    let pitch = if bits & throwing != 0 {
        sim::aim::look_onto(me.pos, aim, me.aloft, at)
    } else {
        0
    };
    Input::looking_at(bits, aim, pitch)
}

/// Keep a direction inside the walls: along them rather than into them.
fn keep_in(w: &World, at: V3, dir: V3) -> V3 {
    let b = w.arena().bounds;
    let room = Fx::from_int(4);
    let ahead = at.add(dir.scale(Fx::from_int(2)));
    let out = ahead.x.raw() < b.lo_x.add(room).raw()
        || ahead.x.raw() > b.hi_x.sub(room).raw()
        || ahead.z.raw() < b.lo_z.add(room).raw()
        || ahead.z.raw() > b.hi_z.sub(room).raw();
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
    // And a little in, so a corner is not walked into either.
    unit(side.add(unit(middle, side).scale(crate::HALF)), side)
}

/// Is there floor to go to that way: nothing standing in the next three
/// metres, and inside the walls?
fn clear(w: &World, ground: &sim::arena::Terrain, at: V3, dir: V3) -> bool {
    let b = w.arena().bounds;
    let room = Fx::from_int(2);
    (1..=3).all(|m| {
        let p = at.add(dir.scale(Fx::from_int(m)));
        let inside = p.x.raw() > b.lo_x.add(room).raw()
            && p.x.raw() < b.hi_x.sub(room).raw()
            && p.z.raw() > b.lo_z.add(room).raw()
            && p.z.raw() < b.hi_z.sub(room).raw();
        inside && ground.ground_under(p).raw() <= at.y.add(Fx::ratio(3, 10)).raw()
    })
}

/// The first of these ways that is clear, or the first if none is.
fn first_clear(w: &World, at: V3, ways: &[V3]) -> V3 {
    let ground = w.terrain();
    ways.iter()
        .copied()
        .find(|d| d.flat_len().raw() > 0 && clear(w, &ground, at, *d))
        .unwrap_or(ways[0])
}

/// Frames since a cat's move began, through the startup.
fn elapsed(m: &Monster) -> Option<(u8, i32, bool)> {
    let kind = m.doing.attacking()?;
    let a = pair::SPECIES.attack(kind);
    match m.doing {
        Doing::Startup { left, .. } => Some((kind, a.startup as i32 - left as i32, false)),
        Doing::Active { left, .. } => {
            Some((kind, a.startup as i32 + a.active as i32 - left as i32, true))
        }
        _ => None,
    }
}

impl Plan for Pair {
    fn watch(&mut self, w: &World) {
        let seen = Seen {
            cats: w.monsters,
            frame: w.frame,
        };
        self.memory[self.at] = seen;
        self.at = (self.at + 1) % self.memory.len();
        self.filled = (self.filled + 1).min(self.memory.len());
    }

    fn act(&mut self, w: &World) -> Input {
        self.cooldown = self.cooldown.saturating_sub(1);
        self.dodge_left = self.dodge_left.saturating_sub(1);
        self.stone_left = self.stone_left.saturating_sub(1);
        self.behind_left = self.behind_left.saturating_sub(1);
        let me = w.players[self.who];
        if !self.looked {
            self.look = yaw_of(me.facing);
            self.looked = true;
        }
        if !me.grounded || me.action.actionable() {
            self.leap_left = self.leap_left.saturating_sub(1);
        }
        if me.health <= 0 || !matches!(w.phase, Phase::Fighting) {
            return Input::default();
        }
        let seen = self.recall();
        let stones = sim::stones::gather(&w.players);
        let ground = w.terrain();
        let scene = scene_of(w, &stones, &ground);

        // **What it can see.** A cat whose body is in view, or whose marker
        // is, is perceived: where it is and what it is doing. Anything else
        // is where it was last seen.
        let view = Input::aimed(0, turns_to_aim(self.look.sub(me.carry_yaw)));
        let mut cats: [Option<Monster>; MAX_MONSTERS] = [None; MAX_MONSTERS];
        for (s, slot) in seen.cats.iter().enumerate() {
            let Some(m) = slot.filter(|m| m.alive()) else {
                if slot.is_some_and(|m| !m.alive()) {
                    self.known[s] = None;
                }
                continue;
            };
            let body = in_view(self.who, view, middle(&m), HALF_VIEW, &scene);
            let marked = marker_points(&m, me.pos).is_some_and(|pts| {
                pts.iter()
                    .any(|p| in_view(self.who, view, *p, HALF_VIEW, &scene))
            });
            if body || marked {
                cats[s] = Some(m);
                self.known[s] = Some((flat(m.pos), seen.frame));
            } else if let Some((at, f)) = self.known[s] {
                // Looking right at where it was, and it is not there: it is
                // somewhere else, and it has to be looked for.
                let there = at.add(V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO));
                if seen.frame.saturating_sub(f) > 10
                    && in_view(self.who, view, there, HALF_VIEW, &scene)
                {
                    self.known[s] = None;
                }
            }
        }
        let living = seen.cats.iter().flatten().filter(|m| m.alive()).count();
        if living == 0 {
            return Input::default();
        }

        if std::env::var_os("PAIR_DEBUG").is_some() {
            let show = |m: &Option<Monster>| {
                m.map(|m| {
                    format!(
                        "{:?} r{} ({:.1},{:.1}) d{:.1} aim ({:.1},{:.1}) cov {} seen ({:.1},{:.1}) t{}",
                        m.doing,
                        fight::role(&m),
                        m.pos.x.to_f32_for_render(),
                        m.pos.z.to_f32_for_render(),
                        wide_flat_dist(m.pos, me.pos).to_f32_for_render(),
                        m.aimed_at().x.to_f32_for_render(),
                        m.aimed_at().z.to_f32_for_render(),
                        marker_covers(&m, me.pos, MARGIN),
                        m.brain.seen.x.to_f32_for_render(),
                        m.brain.seen.z.to_f32_for_render(),
                        m.brain.target
                    )
                })
            };
            eprintln!(
                "{} me ({:.1},{:.1}) hp {} {:?} look {:.2} | {:?} | {:?} | intent {:?} st {:b} {:b}",
                w.frame,
                me.pos.x.to_f32_for_render(),
                me.pos.z.to_f32_for_render(),
                me.health,
                me.action,
                self.look.to_f32_for_render(),
                show(&seen.cats[0]),
                show(&seen.cats[1]),
                self.intent,
                fight::state_of(&w.lore, 0),
                fight::state_of(&w.lore, 1)
            );
        }
        if self.leap_left > 0 {
            let dir = keep_in(w, me.pos, self.leap_dir);
            return self.turn_and(w, &me, None, dir, Input::SPACE);
        }

        // 1. The answer to anything coming at it that it can see.
        if let Some(input) = self.answer(w, &me, &cats) {
            return input;
        }
        // 2. A cat that cannot answer, while the other is in view or far.
        if let Some(input) = self.punish(w, &me, &cats, &seen) {
            return input;
        }
        // 3. Keep both in view, and the range.
        self.watch_both(w, &me, &seen)
    }

    fn intent(&self) -> Intent {
        self.intent
    }

    fn hands(&mut self) -> Option<&mut Hands> {
        Some(&mut self.hands)
    }

    fn hands_ref(&self) -> Option<&Hands> {
        Some(&self.hands)
    }
}

impl Pair {
    /// Turn the camera toward `toward` at a person's rate, walk along `dir`,
    /// press `bits` -- the crosshair on `toward` as far as the turn has got.
    fn turn_and(
        &mut self,
        w: &World,
        me: &Player,
        toward: Option<V3>,
        dir: V3,
        bits: u16,
    ) -> Input {
        if let Some(at) = toward {
            let want = yaw_of(flat(at.sub(me.pos)));
            let error = wrap_turns(want.sub(self.look));
            let step = TURN.mul(sim::DT);
            self.look = self.look.add(error.clamp(step.neg(), step));
        }
        self.hands.camera(Some(self.look));
        let at = toward.unwrap_or(me.pos.add(V3::from_turns(self.look).scale(Fx::from_int(8))));
        let mut input = wire(me, self.look, at, bits);
        if dir.flat_len().raw() > 0 {
            input.bits |= steer(self.look, dir);
        }
        if bits & Input::SHIFT != 0 && dir.flat_len().raw() > 0 {
            return self.hands.leave(w, me, dir, input);
        }
        input
    }

    /// Is the camera on this point now -- turned that far already?
    fn facing(&self, me: &Player, at: V3, within: Fx) -> bool {
        let want = yaw_of(flat(at.sub(me.pos)));
        wrap_turns(want.sub(self.look)).abs().raw() <= within.raw()
    }

    /// **The answers**, to what it can see coming.
    fn answer(
        &mut self,
        w: &World,
        me: &Player,
        cats: &[Option<Monster>; MAX_MONSTERS],
    ) -> Option<Input> {
        let body = sim::tuning::body_radius();
        for m in cats.iter().flatten() {
            let Some((kind, e, live)) = elapsed(m) else {
                continue;
            };
            let a = pair::SPECIES.attack(kind);
            if e == 0 || e == 1 {
                self.roll_slop();
            }
            // Frames until it is live, in the present.
            let to_live = a.startup as i32 - e - REACTION as i32 - self.slop;
            match kind {
                // The coil. Until the tail is seen it may be a feint, and a
                // dodge into a feint is what the other one is waiting for;
                // with the flick, toward it, under the arc.
                pair::POUNCE | pair::FEINT => {
                    if kind == pair::FEINT {
                        if e >= FLICK {
                            // Do nothing -- and then look behind.
                            self.behind_left = 60;
                        }
                        continue;
                    }
                    // **Aimed at it, near enough**: the circle follows it
                    // until the cat leaves the ground, so a circle a few
                    // metres off now is on it by then.
                    if !marker_covers(m, me.pos, FOLLOWS) || e < FLICK {
                        continue;
                    }
                    if self.dodge_left == 0 && me.action.actionable() {
                        self.intent = UNDER;
                        self.dodge_left = sim::tuning::dodge_frames() + DODGE_REST;
                        let toward = unit(m.pos.sub(me.pos), V3::from_turns(self.look));
                        return Some(self.turn_and(w, me, Some(middle(m)), toward, Input::SHIFT));
                    }
                }
                // The twin: still through the roar, the dodge after they
                // have left the ground -- sideways, off the line they come
                // in on.
                pair::TWIN => {
                    if !marker_covers(m, me.pos, FOLLOWS) {
                        continue;
                    }
                    let leave = Knob::TwinLeave.raw();
                    let go = leave + 3 - REACTION as i32 + self.slop.max(0);
                    if e < go {
                        self.intent = LATE;
                        return Some(self.turn_and(w, me, Some(middle(m)), V3::ZERO, 0));
                    }
                    if self.dodge_left == 0 && me.action.actionable() && !live {
                        self.intent = LATE;
                        self.dodge_left = sim::tuning::dodge_frames() + DODGE_REST;
                        let along = unit(m.pos.sub(me.pos), V3::from_turns(self.look));
                        let side = V3::new(along.z.neg(), Fx::ZERO, along.x);
                        let side = first_clear(w, me.pos, &[side, side.scale(Fx::ONE.neg())]);
                        return Some(self.turn_and(w, me, Some(middle(m)), side, Input::SHIFT));
                    }
                }
                // The lane: step out of it sideways, dodging only if there is
                // no time left to walk.
                pair::AMBUSH | pair::INTERPOSE => {
                    if !marker_covers(m, me.pos, MARGIN) {
                        continue;
                    }
                    let t = m.telegraph()?;
                    let side = V3::new(t.along.z.neg(), Fx::ZERO, t.along.x);
                    let rel = flat(me.pos.sub(t.anchor));
                    let out = if side.dot(rel).raw() >= 0 {
                        side
                    } else {
                        side.scale(Fx::ONE.neg())
                    };
                    let out = first_clear(w, me.pos, &[out, out.scale(Fx::ONE.neg())]);
                    self.intent = EVADE;
                    let bits =
                        if (live || to_live <= 3) && self.dodge_left == 0 && me.action.actionable()
                        {
                            self.dodge_left = sim::tuning::dodge_frames() + DODGE_REST;
                            Input::SHIFT
                        } else {
                            0
                        };
                    return Some(self.turn_and(w, me, Some(middle(m)), out, bits));
                }
                // The tail: over it.
                pair::TRIP => {
                    if !marker_covers(m, me.pos, MARGIN) {
                        continue;
                    }
                    self.intent = JUMP;
                    if (live || to_live <= 2) && me.grounded && self.leap_left == 0 {
                        self.leap_left = LEAP_HOLD;
                        self.leap_dir = V3::ZERO;
                        return Some(self.turn_and(w, me, Some(middle(m)), V3::ZERO, Input::SPACE));
                    }
                    return Some(self.turn_and(w, me, Some(middle(m)), V3::ZERO, 0));
                }
                // The paws: out of reach, backwards; the dodge if it is too
                // late to walk.
                pair::RAKE | pair::RAKE2 | pair::SWAT | pair::COCK => {
                    let near = wide_flat_dist(m.pos, me.pos).raw()
                        <= a.hit_x
                            .add(a.hit_radius)
                            .add(body)
                            .add(MARGIN)
                            .raw()
                            .max(Fx::from_int(3).raw());
                    if !near && !marker_covers(m, me.pos, MARGIN) {
                        continue;
                    }
                    self.intent = EVADE;
                    let back = unit(me.pos.sub(m.pos), V3::from_turns(self.look));
                    let side = V3::new(back.z.neg(), Fx::ZERO, back.x);
                    let out = first_clear(
                        w,
                        me.pos,
                        &[back, unit(back.add(side), back), unit(back.sub(side), back)],
                    );
                    let bits = if kind != pair::COCK
                        && (live || to_live <= 2)
                        && self.dodge_left == 0
                        && me.action.actionable()
                    {
                        self.dodge_left = sim::tuning::dodge_frames() + DODGE_REST;
                        Input::SHIFT
                    } else {
                        0
                    };
                    return Some(self.turn_and(w, me, Some(middle(m)), out, bits));
                }
                // The dive: under the lip if it is near; otherwise out of
                // the circle, dodged once it has left the lip and can no
                // longer follow.
                pair::DIVE => {
                    if !marker_covers(m, me.pos, FOLLOWS) {
                        continue;
                    }
                    let to = flat(m.pos.sub(me.pos));
                    let lip = Knob::DiveLip.fx().add(Fx::from_int(2));
                    if to.flat_len().raw() < lip.raw() {
                        self.intent = LIP;
                        return Some(self.turn_and(w, me, Some(middle(m)), unit(to, V3::ZERO), 0));
                    }
                    self.intent = EVADE;
                    let go = Knob::DiveLeave.raw() + 3 - REACTION as i32 + self.slop.max(0);
                    let side = unit(V3::new(to.z.neg(), Fx::ZERO, to.x), V3::ZERO);
                    let away = unit(to.scale(Fx::ONE.neg()), V3::ZERO);
                    let side = first_clear(w, me.pos, &[side, side.scale(Fx::ONE.neg()), away]);
                    let bits = if self.dodge_left == 0 && me.action.actionable() && e >= go {
                        self.dodge_left = sim::tuning::dodge_frames() + DODGE_REST;
                        Input::SHIFT
                    } else {
                        0
                    };
                    return Some(self.turn_and(w, me, Some(middle(m)), side, bits));
                }
                _ => {}
            }
        }
        None
    }

    /// **A cat that cannot answer**, hit while the other is in view, far, or
    /// down: in a recovery, on its side, howling is not one (§9.7).
    fn punish(
        &mut self,
        w: &World,
        me: &Player,
        cats: &[Option<Monster>; MAX_MONSTERS],
        seen: &Seen,
    ) -> Option<Input> {
        let poke = sim::moves::get(me.class, sim::state::SLOT_POKE);
        let poke_busy = (poke.startup + poke.active + poke.recovery) as i32;
        let heavy_busy = crate::heavy_commitment(me.class) as i32;
        let reach = self.hands.reach(me).add(Fx::ONE);
        let mut best: Option<(usize, Monster, i32, i32)> = None;
        for (s, m) in cats.iter().enumerate() {
            let Some(m) = m else { continue };
            let open = matches!(
                m.doing,
                Doing::Recovery { .. }
                    | Doing::Toppled { .. }
                    | Doing::Stumble { .. }
                    | Doing::Flinch { .. }
            );
            let howling = m.doing.attacking() == Some(pair::HOWL);
            if !open || howling {
                continue;
            }
            // The other one: dead, in view, or far and not coming.
            let other = 1 - s;
            let safe = match seen.cats[other].filter(|o| o.alive()) {
                None => true,
                Some(o) => {
                    let down = matches!(o.doing, Doing::Toppled { .. } | Doing::Stumble { .. });
                    let in_sight = cats[other].is_some();
                    // Out of sight, it is where it was last seen: far enough,
                    // recently enough, is safe enough.
                    let far = self.known[other].is_some_and(|(at, f)| {
                        seen.frame.saturating_sub(f) < STALE
                            && wide_flat_dist(at, me.pos).raw() > FAR.raw()
                    });
                    // In sight and free beside the one you would hit, it
                    // is a rake waiting for you to walk up.
                    let to_me = sim::math::wide_normalized(flat(me.pos.sub(m.pos)));
                    let stand = m.pos.add(to_me.scale(reach));
                    let beside = !down
                        && o.doing.free()
                        && wide_flat_dist(o.pos, stand).raw() < BESIDE.raw();
                    down || in_sight && !beside && !marker_covers(&o, me.pos, MARGIN)
                        || !in_sight && far
                }
            };
            if !safe {
                continue;
            }
            let window = m.frames_until_free() as i32 - REACTION as i32;
            if window <= poke_busy + EXIT {
                continue;
            }
            // The other one busy for long enough that a big swing is over
            // before it could start anything: dead, down, or in a recovery.
            let other_busy = match seen.cats[other].filter(|o| o.alive()) {
                None => i32::MAX,
                Some(o) if cats[other].is_some() => match o.doing {
                    Doing::Recovery { .. } | Doing::Toppled { .. } | Doing::Stumble { .. } => {
                        o.frames_until_free() as i32
                    }
                    _ => 0,
                },
                Some(_) => 0,
            };
            // The wounded one first, at the bond.
            let wounded = fight::below(m, Knob::BondHealth.fx(), false);
            let score = window + if wounded { 200 } else { 0 };
            if best.is_none_or(|(_, _, b, _)| score > b) {
                best = Some((s, *m, score, other_busy));
            }
        }
        let (_, m, _, other_busy) = best?;
        let target = middle(&m);
        let d = wide_flat_dist(target, me.pos);
        let window = m.frames_until_free() as i32 - REACTION as i32;
        // Not worth crossing the arena for: what is reachable before it is
        // free.
        let walk_frames = d
            .sub(reach)
            .max(Fx::ZERO)
            .div(sim::tuning::move_speed().mul(sim::DT))
            .to_int();
        if walk_frames + poke_busy + EXIT > window {
            return None;
        }
        self.intent = if matches!(m.doing, Doing::Toppled { .. }) {
            UNLOAD
        } else {
            PUNISH
        };
        let dir = if d.raw() > reach.raw() {
            unit(target.sub(me.pos), V3::ZERO)
        } else {
            V3::ZERO
        };
        let ready = self.cooldown == 0
            && me.action.actionable()
            && d.raw() <= reach.raw()
            && self.facing(me, target, Fx::ratio(3, 100));
        let swing = if ready {
            self.cooldown = SWING_GAP;
            if window - walk_frames > heavy_busy + EXIT && other_busy > heavy_busy - REACTION as i32
            {
                heavy(me.class)
            } else {
                Input::LEFT
            }
        } else {
            0
        };
        let input = self.turn_and(w, me, Some(target), dir, swing);
        let left = window - walk_frames;
        if swing != 0 {
            return Some(self.hands.hit(w, me, target, input, Some(left)));
        }
        if d.raw() > reach.raw()
            && let Some(go) = self.hands.close_in(w, me, target, left)
        {
            return Some(go);
        }
        Some(input)
    }

    /// **Keep both in view**, and stand where they can be: outside the paws,
    /// backing off when they are spread round it, turning toward the one it
    /// has lost.
    fn watch_both(&mut self, w: &World, me: &Player, seen: &Seen) -> Input {
        let now = seen.frame;
        let known: Vec<(usize, V3, u32)> = (0..MAX_MONSTERS)
            .filter(|s| seen.cats[*s].is_some_and(|m| m.alive()))
            .filter_map(|s| self.known[s].map(|(at, f)| (s, at, f)))
            .collect();
        // The one it has not seen for longest, if it is stale: find it.
        let stale = known
            .iter()
            .filter(|(_, _, f)| now.saturating_sub(*f) > STALE)
            .min_by_key(|(_, _, f)| *f)
            .copied();
        let howling = seen
            .cats
            .iter()
            .flatten()
            .any(|m| m.alive() && m.doing.attacking() == Some(pair::HOWL));
        let ahead = V3::from_turns(self.look);
        let lost = (0..MAX_MONSTERS)
            .any(|s| seen.cats[s].is_some_and(|m| m.alive()) && self.known[s].is_none());
        let toward = if lost {
            // One it has no idea of: turn, and keep turning, until it is
            // on the screen.
            self.intent = FIND;
            Some(
                me.pos
                    .add(V3::from_turns(self.look.add(Fx::ratio(1, 8))).scale(Fx::from_int(6))),
            )
        } else if let Some((_, at, _)) = stale {
            self.intent = FIND;
            Some(at)
        } else if self.behind_left > 0 {
            // After a feint: look round for the other one.
            self.intent = FIND;
            Some(me.pos.sub(ahead.scale(Fx::from_int(6))))
        } else {
            self.intent = WATCH;
            match known.as_slice() {
                [] => None,
                [(_, a, _)] => Some(*a),
                [(_, a, _), (_, b, _), ..] => {
                    let ua = unit(a.sub(me.pos), ahead);
                    let ub = unit(b.sub(me.pos), ahead);
                    Some(me.pos.add(unit(ua.add(ub), ua).scale(Fx::from_int(6))))
                }
            }
        };
        // Where to stand.
        let mut dir = V3::ZERO;
        let nearest = known
            .iter()
            .min_by_key(|(_, at, _)| wide_flat_dist(*at, me.pos).raw())
            .copied();
        if let [(_, a, _), (_, b, _), ..] = known.as_slice() {
            let ua = unit(a.sub(me.pos), ahead);
            let ub = unit(b.sub(me.pos), ahead);
            let apart = yaw_of(ua).sub(yaw_of(ub));
            if wrap_turns(apart).abs().raw() > SPREAD.raw() {
                // Spread round it: back away from both, so they come round
                // onto one side.
                dir = unit(ua.add(ub).scale(Fx::ONE.neg()), V3::ZERO);
                if dir.flat_len().raw() == 0 {
                    dir = V3::new(ua.z.neg(), Fx::ZERO, ua.x);
                }
            }
        }
        if let Some((_, at, _)) = nearest {
            let d = wide_flat_dist(at, me.pos);
            let keep = if howling {
                KEEP.add(Fx::from_int(3))
            } else {
                KEEP
            };
            if d.raw() < keep.raw() {
                if howling {
                    self.intent = BACK;
                }
                dir = unit(dir.add(unit(me.pos.sub(at), ahead)), ahead);
            }
        }
        // Looking for one it cannot see, it stands still unless the one it
        // can see is too close: backing blind into the other is the
        // mistake the fight is teaching.
        if (lost || stale.is_some()) && self.intent == FIND {
            let close =
                nearest.is_some_and(|(_, at, _)| wide_flat_dist(at, me.pos).raw() < KEEP.raw());
            if !close {
                dir = V3::ZERO;
            }
        }
        let dir = if dir.flat_len().raw() > 0 {
            keep_in(w, me.pos, dir)
        } else {
            dir
        };
        // The Elementalist builds her split: a stone toward a Striker she
        // cannot see, when she is looking that way.
        if me.class == sim::Class::Elementalist && self.stone_left == 0 {
            if let Some((_, at, _)) = stale {
                if self.facing(me, at, Fx::ratio(4, 100))
                    && wide_flat_dist(at, me.pos).raw() < Fx::from_int(12).raw()
                {
                    self.stone_left = STONE_GAP;
                    self.intent = STONE;
                    let mid = me.pos.add(flat(at.sub(me.pos)).scale(crate::HALF));
                    return self.turn_and(w, me, Some(mid), dir, Input::MECHANIC);
                }
            }
        }
        let input = self.turn_and(w, me, toward, dir, 0);
        // Both in view and nothing coming: the class's own business, at the
        // nearer.
        if self.intent == WATCH
            && let Some((_, at, _)) = nearest
            && let Some(own) = self.hands.idle(
                w,
                me,
                at,
                at.add(V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO)),
                IDLE_SAFE,
            )
        {
            return own;
        }
        input
    }
}

/// How long the hunter believes it has, watching both cats with nothing
/// coming: long enough for a shot or a sent shadow, not for a pillar.
const IDLE_SAFE: i32 = 32;

/// **The jump height**, for the plan that wants it.
pub fn hop_of(p: &Pair) -> Fx {
    p.hop
}

// ---------------------------------------------------------------------------
// The report's lines
// ---------------------------------------------------------------------------

/// What the report counts about the Pair (`the-pair.md` §9).
pub struct PairTally {
    /// Frames both living cats were in the hunter's view, and the frames
    /// there were two to watch.
    both_in_view: u32,
    two_alive: u32,
    /// Each cat's move in progress: was the cat itself in view as it began,
    /// and how many frames its marker has been on its target's screen.
    began_seen: [bool; MAX_MONSTERS],
    marker_seen: [u32; MAX_MONSTERS],
    /// Hits landed, those from a cat off screen as it began, and those whose
    /// marker had been on screen at least fifteen frames (for every move
    /// whose tell is that long).
    hits: u32,
    off_screen: u32,
    marker_hits: u32,
    marker_ok: u32,
    /// Hits whose marker was on screen under fifteen frames: the second
    /// clause of unanswerable.
    unseen_markers: u32,
    feints: u32,
    bitten: u32,
    feint_at: [Option<u32>; MAX_MONSTERS],
    after_feint_thrown: u32,
    after_feint_landed: u32,
    after_feint: [bool; MAX_MONSTERS],
    twins: u32,
    crashed: u32,
    twin_landed: u32,
    twin_hit: bool,
    /// Frames either cat lacked sight of its target, of the frames both
    /// lived.
    split: u32,
    /// Damage into a scarred cat from its blind side; scars.
    blind_hits: u32,
    scars: u32,
    interposes: u32,
    interposes_punished: u32,
    death_at: [Option<u32>; MAX_MONSTERS],
    /// Where each fighter was over the last `REACTION` frames, oldest first:
    /// whether a hit was walked into.
    trail: [[V3; REACTION + 1]; sim::state::MAX_PLAYERS],
    /// Hits whose marker was on screen too briefly, taken by a fighter who
    /// walked into it in that time: theirs, as the shared rule says of a
    /// stomp run into.
    walked_in: u32,
    enraged: u32,
    fought: u32,
    perches: u32,
    dives: u32,
    frame: u32,
}

impl Tally for PairTally {
    fn observe(&mut self, before: &World, after: &World) {
        self.observe_with(before, after, &[]);
    }

    fn observe_with(&mut self, before: &World, after: &World, bots: &[Hunter]) {
        self.frame = after.frame;
        let stones = sim::stones::gather(&after.players);
        let ground = after.terrain();
        let scene = scene_of(after, &stones, &ground);
        // Where each hunter's camera points: its yaw, level. A swing thrown
        // with the crosshair a few degrees up is not a camera turned to the
        // sky, and the floor round a fighter's feet is on the screen.
        let look = |who: usize| {
            bots.iter()
                .find(|h| h.who == who)
                .map(|h| Input::aimed(0, h.last.aim))
        };
        for (i, p) in after.players.iter().enumerate() {
            self.trail[i].rotate_left(1);
            self.trail[i][REACTION] = p.pos;
        }
        let alive: Vec<usize> = (0..MAX_MONSTERS)
            .filter(|s| after.monsters[*s].is_some_and(|m| m.alive()))
            .collect();
        if after.monsters.iter().flatten().all(|m| m.brain.grace == 0) {
            self.fought += 1;
        }
        if alive.len() == 2 {
            self.two_alive += 1;
            if let Some(view) = look(0) {
                let both = alive.iter().all(|s| {
                    let m = after.monsters[*s].expect("alive");
                    in_view(0, view, middle(&m), HALF_VIEW, &scene)
                });
                if both {
                    self.both_in_view += 1;
                }
            }
            if alive
                .iter()
                .any(|s| fight::unseen(&after.monsters[*s].expect("alive")) > 0)
            {
                self.split += 1;
            }
        }
        for s in 0..MAX_MONSTERS {
            let (Some(was), Some(now)) = (before.monsters[s], after.monsters[s]) else {
                continue;
            };
            if was.alive() && !now.alive() && self.death_at[s].is_none() {
                self.death_at[s] = Some(after.frame);
            }
            if fight::enraged(&now) && now.alive() {
                self.enraged += 1;
            }
            if fight::scar(&now) != 0 && fight::scar(&was) == 0 {
                self.scars += 1;
            }
            let who = (now.brain.target as usize).min(after.players.len() - 1);
            let victim = after.players[who];
            // A move beginning: is the cat on screen as it does?
            let began = |kind: u8| {
                now.doing.attacking() == Some(kind) && was.doing.attacking() != Some(kind)
                    || matches!(now.doing, Doing::Startup { kind: k, .. } if k == kind)
                        && !matches!(was.doing, Doing::Startup { kind: k, .. } if k == kind)
            };
            if let Doing::Startup { kind, .. } = now.doing {
                if began(kind) {
                    self.marker_seen[s] = 0;
                    self.began_seen[s] =
                        look(who).is_none_or(|v| in_view(who, v, middle(&now), HALF_VIEW, &scene));
                    match kind {
                        pair::FEINT => {
                            self.feints += 1;
                            self.feint_at[s] = Some(after.frame);
                        }
                        pair::TWIN if s == 0 => {
                            self.twins += 1;
                            self.twin_hit = false;
                        }
                        pair::AMBUSH => {
                            let other = 1 - s;
                            let recent = self.feint_at[other]
                                .is_some_and(|f| after.frame.saturating_sub(f) <= 90);
                            self.after_feint[s] = recent;
                            if recent {
                                self.after_feint_thrown += 1;
                            }
                        }
                        pair::PERCH => self.perches += 1,
                        pair::DIVE => self.dives += 1,
                        pair::INTERPOSE => self.interposes += 1,
                        _ => {}
                    }
                }
            }
            // The feint bitten: a dodge begun during its coil.
            if let Doing::Startup {
                kind: pair::FEINT, ..
            } = now.doing
            {
                let dodged = matches!(victim.action, Action::Dodge { .. })
                    && !matches!(before.players[who].action, Action::Dodge { .. });
                if dodged {
                    self.bitten += 1;
                }
            }
            // Its marker on its target's screen.
            if now.doing.attacking().is_some() {
                if let (Some(v), Some(pts)) = (look(who), marker_points(&now, victim.pos)) {
                    if pts.iter().any(|p| in_view(who, v, *p, HALF_VIEW, &scene)) {
                        self.marker_seen[s] += 1;
                    }
                } else if look(who).is_none() {
                    self.marker_seen[s] += 1;
                }
            }
            // The interpose run broken by a hit.
            if matches!(
                was.doing,
                Doing::Active {
                    kind: pair::INTERPOSE,
                    ..
                }
            ) && matches!(now.doing, Doing::Flinch { .. })
            {
                self.interposes_punished += 1;
            }
            // The twin pounce crashing.
            if s == 0
                && matches!(
                    was.doing,
                    Doing::Active {
                        kind: pair::TWIN,
                        ..
                    }
                )
                && matches!(now.doing, Doing::Toppled { .. })
            {
                self.crashed += 1;
            }
            // Damage into a scarred cat from its blind side.
            let side = fight::scar(&now);
            if side != 0 && now.health < was.health {
                let right = V3::from_turns(now.yaw.add(sim::math::QUARTER_TURN));
                let hitter = after
                    .players
                    .iter()
                    .filter(|p| p.health > 0)
                    .min_by_key(|p| wide_flat_dist(p.pos, now.pos).raw());
                if let Some(p) = hitter {
                    let off = right.dot(flat(p.pos.sub(now.pos)));
                    let on_blind = (off.raw() >= 0) == (side > 0);
                    if on_blind {
                        self.blind_hits += 1;
                    }
                }
            }
            // A hit landing.
            let landed = !was.hit_used && now.hit_used;
            if landed {
                let kind = now.doing.attacking().unwrap_or(0);
                let hurt = (0..after.players.len())
                    .any(|i| after.players[i].health < before.players[i].health);
                if hurt && pair::SPECIES.attack(kind).damage > 0 {
                    self.hits += 1;
                    if !self.began_seen[s] {
                        self.off_screen += 1;
                    }
                    if std::env::var_os("PAIR_DEBUG").is_some() {
                        eprintln!(
                            "HIT {} {} marker seen {} began seen {}",
                            after.frame,
                            pair::MOVES[kind as usize].name,
                            self.marker_seen[s],
                            self.began_seen[s]
                        );
                    }
                    let tell = pair::SPECIES.attack(kind).startup as usize;
                    if tell >= REACTION || kind == pair::RAKE2 {
                        self.marker_hits += 1;
                        // Where it was a reaction ago, against the mark as
                        // it landed: outside it, it walked in since.
                        let then = self.trail[who][0];
                        let walked = !marker_covers(&now, then, Fx::ZERO);
                        if self.marker_seen[s] as usize >= REACTION {
                            self.marker_ok += 1;
                        } else if walked {
                            self.walked_in += 1;
                        } else {
                            self.unseen_markers += 1;
                        }
                    }
                    if kind == pair::TWIN && !self.twin_hit {
                        self.twin_hit = true;
                        self.twin_landed += 1;
                    }
                    if kind == pair::AMBUSH && self.after_feint[s] {
                        self.after_feint_landed += 1;
                    }
                }
            }
        }
    }

    fn lines(&self) -> Vec<(String, String, String)> {
        let pct = |a: u32, b: u32| {
            if b == 0 {
                "--".to_string()
            } else {
                format!("{:.0}%", 100.0 * a as f32 / b as f32)
            }
        };
        let gap = match (self.death_at[0], self.death_at[1]) {
            (Some(a), Some(b)) => format!("{:.1} s", a.abs_diff(b) as f32 / 60.0),
            (Some(_), None) | (None, Some(_)) => "one died".to_string(),
            _ => "--".to_string(),
        };
        vec![
            (
                "both in view".into(),
                pct(self.both_in_view, self.two_alive),
                "whether the fight is played as the lesson says".into(),
            ),
            (
                "hits from off screen".into(),
                format!("{} of {}", self.off_screen, self.hits),
                "the cat itself out of view as it began".into(),
            ),
            (
                "markers seen >= 15f".into(),
                format!(
                    "{} of {} ({} walked in)",
                    self.marker_ok, self.marker_hits, self.walked_in
                ),
                "the rest are the second clause of unanswerable".into(),
            ),
            (
                "feints bitten".into(),
                format!("{} of {}", self.bitten, self.feints),
                "dodges during a feint's coil: whether the tail reads".into(),
            ),
            (
                "ambush after a feint".into(),
                format!("{} / {}", self.after_feint_landed, self.after_feint_thrown),
                "landed / thrown: the trap, and whether it is escapable".into(),
            ),
            (
                "twin pounces".into(),
                format!(
                    "{} thrown, {} crashed, {} landed",
                    self.twins, self.crashed, self.twin_landed
                ),
                "the window's frequency, and whether late is learnable".into(),
            ),
            (
                "split time".into(),
                pct(self.split, self.two_alive),
                "frames either cat lacked sight of its target".into(),
            ),
            (
                "scars, blind-side hits".into(),
                format!("{}, {}", self.scars, self.blind_hits),
                "whether the scar changes anything".into(),
            ),
            (
                "interposes punished".into(),
                format!("{} of {}", self.interposes_punished, self.interposes),
                "whether the bond's exploit is found".into(),
            ),
            (
                "perches, dives".into(),
                format!("{}, {}", self.perches, self.dives),
                "the long-range threat".into(),
            ),
            (
                "death gap, enraged".into(),
                format!("{gap}, {:.1} s", self.enraged as f32 / 60.0),
                "who is killed first, and whether the order mattered".into(),
            ),
        ]
    }

    fn unanswerable(&self) -> u32 {
        self.unseen_markers
    }
}

impl Default for PairTally {
    fn default() -> PairTally {
        PairTally {
            both_in_view: 0,
            two_alive: 0,
            began_seen: [true; MAX_MONSTERS],
            marker_seen: [0; MAX_MONSTERS],
            hits: 0,
            off_screen: 0,
            marker_hits: 0,
            marker_ok: 0,
            unseen_markers: 0,
            feints: 0,
            bitten: 0,
            feint_at: [None; MAX_MONSTERS],
            after_feint_thrown: 0,
            after_feint_landed: 0,
            after_feint: [false; MAX_MONSTERS],
            twins: 0,
            crashed: 0,
            twin_landed: 0,
            twin_hit: false,
            split: 0,
            blind_hits: 0,
            scars: 0,
            interposes: 0,
            interposes_punished: 0,
            death_at: [None; MAX_MONSTERS],
            trail: [[V3::ZERO; REACTION + 1]; sim::state::MAX_PLAYERS],
            walked_in: 0,
            enraged: 0,
            fought: 0,
            perches: 0,
            dives: 0,
            frame: 0,
        }
    }
}

impl PairTally {
    pub fn both_in_view_share(&self) -> f32 {
        self.both_in_view as f32 / self.two_alive.max(1) as f32
    }
}

fn make_tally() -> Box<dyn Tally> {
    Box::new(PairTally::default())
}

fn make(who: usize, seed: u32, hop: Fx) -> Box<dyn Plan + Send + Sync> {
    Box::new(Pair::new(who, seed, hop))
}

/// Nothing a cat does throws a rider: nobody rides a cat.
fn bucks(_kind: u8) -> bool {
    false
}

pub static CARD: crate::plans::Card = crate::plans::Card {
    species: sim::species::SpeciesId::PAIR,
    plan: make,
    bucks,
    words: crate::plans::Words {
        weak_hits: "weak-point hits (none: the head is 1.3x, not a weak point)",
        broken: "parts broken (none)",
        into_breakables: "damage into breakable parts",
        into_breakables_why: "none: the cats' lasting consequence is the scar",
        worst: "worst part",
        ride_for: "rides (nobody rides a cat)",
        toppled_pool: "while crashed",
    },
    tally: Some(make_tally),
    gamble: None,
};
