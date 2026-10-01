//! The Galewing's hunter: the plan a decent player follows in their first ten
//! minutes against it (`galewing.md` §9).
//!
//! 1. Stand in the open, away from the edges, never on the tower top.
//! 2. **Stoop**: dodge as the fill reaches the circle's edge, then swing at
//!    the nearest wing while it is down, and jump the buffet.
//! 3. **Talon lane**: in it, crouch as the front arrives; beside it, stand and
//!    swing up at the wing going over.
//! 4. **Downwash**: walk to the nearest lee -- a stone or the tower -- or to
//!    the eye if that is nearer; crouch if neither is reachable in time.
//! 5. **Volley**: step out of the lane sideways.
//! 6. **Screech**: get out of the cone, to the side.
//! 7. **Crash**: walk up a wing, hit a wing root. **Plan A** jumps off in the
//!    gather; **plan B** (`fight --gamble`) rides: stands on the spine,
//!    braces through the downstrokes and the roll, steps off at the swoop.
//! 8. **Perch**: climb the tower if the announcement began near its first
//!    ledge.
//!
//! Carried, it swings up at the legs. A class with something that reaches
//! the bird in the air throws it: the Elementalist hops and fires her air
//! bolt at the circling bird (§7, the one class that reaches it up there).
//!
//! It reads what is drawn: the creature's body, its telegraph and its floor
//! signs, as they were `REACTION` frames ago. Where its own feet are, and the
//! rhythm of the wings it is standing on, it knows.

use sim::aim::{Scene, on_screen};
use sim::fixed::Fx;
use sim::math::{atan2_turns, wide_flat_dist, wide_flat_len, wide_normalized};
use sim::monster::{Doing, Monster, mount_part};
use sim::species::galewing::{self as gw, Knob, fight, flight};
use sim::state::{Action, Phase, Player};
use sim::{Class, Input, V3, World};

use crate::report::{HALF_VIEW, Tally};
use crate::{Hands, Hunter, Intent, Plan, REACTION, heavy, steer, turns_to_aim};

pub const HOME: Intent = Intent("Home");
pub const DODGE: Intent = Intent("Dodge");
pub const CROUCH: Intent = Intent("Crouch");
pub const LEE: Intent = Intent("Lee");
pub const OUT: Intent = Intent("Out");
pub const JUMP: Intent = Intent("Jump");
pub const WING: Intent = Intent("Wing");
pub const CLIMB: Intent = Intent("Climb");
pub const RIDE: Intent = Intent("Ride");
pub const BRACE: Intent = Intent("Brace");
pub const LEAVE: Intent = Intent("Leave");
pub const LEGS: Intent = Intent("Legs");
pub const SHOOT: Intent = Intent("Shoot");

/// How many frames early or late a timed press is.
const SLOP_EARLY: i32 = 3;
const SLOP_LATE: i32 = 2;
/// A dodge is pressed this many frames before the hit, so its invulnerable
/// frames are running when it lands.
const DODGE_LEAD: i32 = 3;
/// A jump over the buffet is pressed this many frames before it.
const JUMP_LEAD: i32 = 8;
/// Frames between its own swings.
const SWING_GAP: u16 = 22;
/// Frames to hold a jump.
const LEAP_HOLD: u16 = 24;
/// A margin round a drawn shape: it does not stand on the line.
const MARGIN: Fx = Fx::ratio(1, 1);
/// Beside a lane, within this, it stands and swings at the wing.
const BESIDE: Fx = Fx::from_int(4);
/// Home: the open middle of the plateau, south of the tower and more than
/// sixteen metres from every drop.
const HOME_AT: (i32, i32) = (0, -6);
/// How far from home it lets itself be before walking back.
const HOME_SLACK: Fx = Fx::from_int(5);
/// The crouch starts when the front is this far off.
const CROUCH_FRONT: Fx = Fx::from_int(8);
/// The first ledge is this near for the climb to be worth starting.
const STAIRS_NEAR: Fx = Fx::from_int(4);
/// The look's pitch while it watches the floor and the shadow: a little
/// down, so the markers round it are on its screen.
const WATCH_PITCH: i16 = -1800;

/// Which plan it plays.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Gamble {
    /// Off in the gather.
    A,
    /// Up with it.
    B,
}

pub struct Galewing {
    who: usize,
    memory: Vec<World>,
    at: usize,
    filled: usize,
    intent: Intent,
    cooldown: u16,
    dodge_left: u16,
    leap_left: u16,
    /// The move it has answered with a timed press: (kind, frame it began).
    answered: Option<(u8, u32)>,
    slop: i32,
    rng: u32,
    hop: Fx,
    gamble: Gamble,
    /// Climbing the tower: the ledge it is going for, or none.
    climbing: Option<usize>,
    /// Its class (`crate::class`); and what this frame's choice was for it:
    /// the point a swing went at, in what window, which way a dodge went,
    /// and where it waits.
    hands: Hands,
    aimed: Option<(V3, Option<i32>)>,
    out: Option<V3>,
    waiting: Option<(V3, V3, i32)>,
}

impl Galewing {
    pub fn new(who: usize, seed: u32, hop: Fx, gamble: Gamble) -> Galewing {
        let mut p = Galewing {
            who,
            memory: Vec::with_capacity(REACTION + 1),
            at: 0,
            filled: 0,
            intent: HOME,
            cooldown: 0,
            dodge_left: 0,
            leap_left: 0,
            answered: None,
            slop: 0,
            rng: (0x9E37_79B9
                ^ (who as u32).wrapping_mul(0x85EB_CA6B)
                ^ seed.wrapping_mul(0x27D4_EB2F))
                | 1,
            hop,
            gamble,
            climbing: None,
            hands: Hands::new(who, seed),
            aimed: None,
            out: None,
            waiting: None,
        };
        p.roll_slop();
        p
    }

    fn recall(&self) -> &World {
        let len = self.memory.len();
        let back = REACTION.min(self.filled.saturating_sub(1));
        let idx = (self.at + len - 1 - back) % len;
        &self.memory[idx]
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
    if wide_flat_len(v).raw() > 0 {
        wide_normalized(v)
    } else {
        fallback
    }
}

fn yaw_of(v: V3) -> Fx {
    atan2_turns(v.z, v.x)
}

/// The bird, alive.
fn bird(w: &World) -> Option<Monster> {
    let slot = fight::slot_of(w)?;
    w.monsters[slot].filter(|m| m.alive())
}

/// The middle of a part's box, in the world.
fn part_at(m: &Monster, part: usize) -> V3 {
    let sh = gw::SPECIES.shape(part);
    m.world_of(part, sh.min.add(sh.max).scale(crate::HALF))
}

/// The middle of a part's top face, in the world.
fn top_of(m: &Monster, part: usize) -> V3 {
    let sh = gw::SPECIES.shape(part);
    let mid = sh.min.add(sh.max).scale(crate::HALF);
    m.world_of(part, V3::new(mid.x, sh.max.y, mid.z))
}

/// The wing parts, roots first.
const WINGS: [usize; 6] = [
    gw::ROOT_L,
    gw::ROOT_R,
    gw::BLADE_L,
    gw::BLADE_R,
    gw::TIP_L,
    gw::TIP_R,
];

/// The wing part nearest a point, and where it is.
fn nearest_wing(m: &Monster, from: V3, roots: bool) -> (usize, V3) {
    WINGS
        .iter()
        .filter(|p| !roots || gw::is_root(**p))
        .map(|p| (*p, part_at(m, *p)))
        .min_by_key(|(_, at)| sim::math::wide_len(at.sub(from)).raw())
        .unwrap_or((gw::BACK, m.pos))
}

/// Is a point inside a strip, with a margin?
fn in_strip(start: V3, along: V3, length: Fx, width: Fx, at: V3, margin: Fx) -> bool {
    let d = flat(at.sub(start));
    let s = d.dot(along);
    let side = V3::new(along.z.neg(), Fx::ZERO, along.x);
    let c = d.dot(side).abs();
    s.raw() >= margin.neg().raw()
        && s.raw() <= length.add(margin).raw()
        && c.raw() <= crate::HALF.mul(width).add(margin).raw()
}

/// The nearest way out of a strip, sideways -- the other way if that way is
/// a wall or a drop.
fn out_of_strip(w: &World, start: V3, along: V3, at: V3) -> V3 {
    let side = V3::new(along.z.neg(), Fx::ZERO, along.x);
    let (near, far) = if flat(at.sub(start)).dot(side).raw() >= 0 {
        (side, side.scale(Fx::ONE.neg()))
    } else {
        (side.scale(Fx::ONE.neg()), side)
    };
    if clear(w, at, near) { near } else { far }
}

/// Is there floor that way for a few metres: nothing standing up out of
/// it, and nothing dropping away?
fn clear(w: &World, at: V3, dir: V3) -> bool {
    let ground = w.terrain();
    let here = ground.ground_under(V3::new(at.x, Fx::ZERO, at.z));
    (1..=4).all(|k| {
        let p = at.add(dir.scale(Fx::from_int(k)));
        let g = ground.ground_under(V3::new(p.x, Fx::ZERO, p.z));
        g.sub(here).abs().raw() < Fx::ratio(3, 10).raw()
    })
}

/// A look along `yaw`, a little down at the floor, or onto `at` when it
/// throws something at it.
/// The steepest a hunter on the floor looks up when not shooting at the
/// sky: level. Looking any higher from beside a bird standing over you puts
/// the floor at your own feet below the bottom of the screen.
const SWING_UP: i16 = 0;

fn wire(me: &Player, yaw: Fx, at: Option<V3>, bits: u16) -> Input {
    let aim = turns_to_aim(yaw.sub(me.carry_yaw));
    let pitch = match at {
        Some(at) => sim::aim::look_onto(me.pos, aim, me.aloft, at),
        None => WATCH_PITCH,
    };
    Input::looking_at(bits, aim, pitch)
}

/// Walk toward `to`, looking at `look_at` (or along the walk).
fn walk(me: &Player, to: V3, look_at: Option<V3>, bits: u16) -> Input {
    let dir = unit(to.sub(me.pos), V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO));
    let yaw = match look_at {
        Some(at) => yaw_of(at.sub(me.pos)),
        None => yaw_of(dir),
    };
    let keys = steer(yaw, dir);
    let near = wide_flat_dist(me.pos, to).raw() < Fx::ratio(1, 2).raw();
    wire(me, yaw, None, bits | if near { 0 } else { keys })
}

/// The tower's ledges, lowest first, and its top: every solid standing
/// within six metres of the perch and below or at its top, sorted by height.
fn ledges(w: &World) -> Vec<sim::arena::Solid> {
    let Some(top) = fight::perch_top(w) else {
        return Vec::new();
    };
    let mut out: Vec<sim::arena::Solid> = w
        .arena()
        .solids()
        .iter()
        .copied()
        .filter(|s| {
            let mid = s.min.add(s.max).scale(crate::HALF);
            !s.hangs()
                && wide_flat_dist(mid, top).raw() < Fx::from_int(7).raw()
                && s.max.y.raw() <= top.y.raw()
                && s.max.y.raw() > fight::base(w).raw()
        })
        .collect();
    out.sort_by_key(|s| s.max.y.raw());
    out
}

/// The nearest point of a solid's top to `at`, a little inside its edge.
fn onto(s: &sim::arena::Solid, at: V3) -> V3 {
    let pad = Fx::ratio(4, 10);
    let x =
        at.x.clamp(s.min.x.add(pad), s.max.x.sub(pad).max(s.min.x.add(pad)));
    let z =
        at.z.clamp(s.min.z.add(pad), s.max.z.sub(pad).max(s.min.z.add(pad)));
    V3::new(x, s.max.y, z)
}

impl Plan for Galewing {
    fn watch(&mut self, w: &World) {
        if self.memory.len() < REACTION + 1 {
            self.memory.push(w.clone());
            self.at = self.memory.len() % (REACTION + 1);
        } else {
            self.memory[self.at] = w.clone();
            self.at = (self.at + 1) % self.memory.len();
        }
        self.filled = (self.filled + 1).min(REACTION + 1);
    }

    fn act(&mut self, w: &World) -> Input {
        self.cooldown = self.cooldown.saturating_sub(1);
        self.dodge_left = self.dodge_left.saturating_sub(1);
        let me = w.players[self.who];
        if !me.grounded || me.aboard() || me.action.actionable() {
            self.leap_left = self.leap_left.saturating_sub(1);
        }
        if me.health <= 0 || !matches!(w.phase, Phase::Fighting) {
            return Input::default();
        }
        let Some(now) = bird(w) else {
            return Input::default();
        };
        let seen = self.recall().clone();
        let Some(s) = bird(&seen) else {
            return Input::default();
        };
        self.aimed = None;
        self.out = None;
        self.waiting = None;
        let mut input = if fight::carried(&w.lore) == Some(self.who) {
            self.legs(&me, &now)
        } else if me.aboard() {
            self.ride(w, &seen, &me, &now, &s)
        } else {
            self.ground(w, &seen, &me, &s)
        };
        if self.leap_left > 0 {
            input = input.with(Input::SPACE);
        }
        // The class's turn: what the choice was for, in its own hands. What
        // it throws at a point it aims at that point -- a shot is not craned
        // down below it, so the clamp below does not apply to one.
        let planned = input;
        const ATTACKS: u16 =
            Input::LEFT | Input::RIGHT | Input::MIDDLE | Input::SPECIAL | Input::MECHANIC;
        if input.bits & ATTACKS != 0
            && let Some((at, window)) = self.aimed
        {
            input = self.hands.hit(w, &me, at, input, window);
        } else if input.bits & Input::SHIFT != 0
            && let Some(out) = self.out
        {
            input = self.hands.leave(w, &me, out, input);
        } else if let Some((at, Some(window))) = self.aimed
            && wide_flat_dist(at, me.pos).raw() > self.hands.reach(&me).add(Fx::ONE).raw()
            && let Some(go) = self.hands.close_in(w, &me, at, window)
        {
            input = go;
        } else if let Some((beast, at, safe)) = self.waiting
            && let Some(own) = self.hands.idle(w, &me, beast, at, safe)
        {
            input = own;
        }
        if input != planned {
            return input;
        }
        // **Never craned at the sky from the floor**, unless shooting at it:
        // under a bird standing over you, the part you swing at is metres
        // up, and a look that follows it takes the floor -- where its next
        // windup is drawn -- off the screen. A player fights it from under it
        // with the floor round their feet in view.
        if me.grounded && !me.aboard() && self.intent != SHOOT && input.pitch > SWING_UP {
            input = Input::looking_at(input.bits, input.aim, SWING_UP);
        }
        input
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

impl Galewing {
    /// Swing: if it can, at `at`, else nothing.
    fn swing(&mut self, me: &Player, at: V3, bits: u16) -> Input {
        self.aimed = Some((at, self.aimed.and_then(|(_, w)| w)));
        let yaw = yaw_of(at.sub(me.pos));
        let mut b = bits;
        if self.cooldown == 0 && me.action.actionable() {
            b |= Input::LEFT;
            self.cooldown = SWING_GAP;
        }
        wire(me, yaw, Some(at), b)
    }

    /// **Carried**: swing up at the legs.
    fn legs(&mut self, me: &Player, m: &Monster) -> Input {
        self.intent = LEGS;
        let leg = part_at(m, gw::LEG_L);
        self.swing(me, leg, 0)
    }

    /// **On its back.**
    fn ride(&mut self, _w: &World, seen: &World, me: &Player, now: &Monster, s: &Monster) -> Input {
        let part = mount_part(me.mount);
        // Down on the floor: walk to a root and hit it.
        if !fight::aloft(s) {
            // The gather: plan A is off now.
            let gathering = matches!(s.doing, Doing::Startup { kind: gw::LIFT, .. });
            if gathering && self.gamble == Gamble::A {
                self.intent = LEAVE;
                return self.leave(me, now);
            }
            if gathering {
                return self.onto_spine(me, now, true);
            }
            let (root, at) = nearest_wing(now, me.pos, true);
            self.intent = WING;
            if part == root || wide_flat_dist(me.pos, at).raw() < Fx::from_int(2).raw() {
                return self.swing(me, at, 0);
            }
            return walk(me, top_of(now, root), Some(at), 0);
        }
        // In the air.
        let (_, phase, _) = fight::ride_state(&seen.lore);
        let low = s.pos.y.sub(fight::base(seen)).raw()
            < Knob::SwoopHeight.fx().add(Fx::from_int(2)).raw();
        if phase == fight::ride::SWOOP && low {
            self.intent = LEAVE;
            return self.leave(me, now);
        }
        let rolling = matches!(s.doing.attacking(), Some(gw::ROLL));
        // **The beat is a rhythm**: what it saw a reaction ago, carried on.
        let period = flight::beat_period(s);
        let phase_now = (flight::beat_phase(s) + REACTION as i32).rem_euclid(period);
        let down = Knob::BeatDown.raw();
        let downstroke = phase_now >= period - 4 || phase_now <= down + 2;
        // A swing only where all of it fits before the next downstroke: a
        // rider mid-swing cannot brace.
        let poke = sim::moves::get(me.class, sim::state::SLOT_POKE);
        let swing = (poke.startup + poke.active + poke.recovery) as i32;
        let until = (period - 4 - phase_now).rem_euclid(period);
        let fits = me.action.actionable() && swing + 2 <= until;
        if rolling || downstroke || !gw::is_spine(part) || !fits {
            let brace = rolling || downstroke || (gw::is_spine(part) && !fits);
            return self.onto_spine(me, now, brace);
        }
        // Between beats, on the spine: hit the root beside it.
        self.intent = WING;
        let (_, at) = nearest_wing(now, me.pos, true);
        self.swing(me, at, Input::CROUCH)
    }

    /// Walk to the middle of its back, braced if `brace`.
    fn onto_spine(&mut self, me: &Player, m: &Monster, brace: bool) -> Input {
        self.intent = if brace { BRACE } else { RIDE };
        let spine = top_of(m, gw::BACK);
        let bits = if brace { Input::CROUCH } else { 0 };
        if wide_flat_dist(me.pos, spine).raw() < Fx::ratio(6, 10).raw() {
            return wire(me, m.yaw, None, bits);
        }
        walk(me, spine, None, bits)
    }

    /// Off the side: a jump away from its middle.
    fn leave(&mut self, me: &Player, m: &Monster) -> Input {
        let away = unit(me.pos.sub(m.pos), V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO));
        let yaw = yaw_of(away);
        if me.action.actionable() {
            self.leap_left = LEAP_HOLD;
        }
        wire(me, yaw, None, steer(yaw, away))
    }

    /// **On the floor.**
    fn ground(&mut self, w: &World, seen: &World, me: &Player, s: &Monster) -> Input {
        if let Some(i) = self.answer(w, seen, me, s) {
            return i;
        }
        if let Some(i) = self.climb(w, seen, me, s) {
            return i;
        }
        if let Some(i) = self.punish(me, s) {
            return i;
        }
        if let Some(i) = self.shoot(seen, me, s) {
            return i;
        }
        self.home(w, me, s)
    }

    /// **The answers** to whatever it has begun.
    fn answer(&mut self, w: &World, seen: &World, me: &Player, s: &Monster) -> Option<Input> {
        let (kind, left, active) = match s.doing {
            Doing::Startup { kind, left } => (kind, left, false),
            Doing::Active { kind, left } => (kind, left, true),
            _ => return None,
        };
        let a = gw::SPECIES.attack(kind);
        // Frames until its hit, in the present.
        let due = if active {
            -((a.active as i32) - left as i32)
        } else {
            left as i32 + 1
        } - REACTION as i32;
        let look_at = |at: V3| yaw_of(at.sub(me.pos));
        match kind {
            gw::STOOP | gw::HOP => {
                let t = s.telegraph()?;
                let r = t.radius.add(sim::tuning::body_radius()).add(MARGIN);
                if wide_flat_dist(me.pos, t.anchor).raw() > r.raw() {
                    return None;
                }
                let away = unit(me.pos.sub(t.anchor), V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO));
                let yaw = yaw_of(away);
                let began = seen
                    .frame
                    .wrapping_sub((a.startup - left.min(a.startup)) as u32);
                if self.answered != Some((kind, began))
                    && due <= DODGE_LEAD + self.slop
                    && me.action.actionable()
                {
                    self.answered = Some((kind, began));
                    self.roll_slop();
                    self.dodge_left = 20;
                    self.intent = DODGE;
                    self.out = Some(away);
                    return Some(wire(me, yaw, None, steer(yaw, away) | Input::SHIFT));
                }
                self.intent = DODGE;
                Some(wire(me, look_at(t.anchor), None, 0))
            }
            gw::TALON => {
                let (start, along, length, width) = fight::lane(&seen.lore);
                if in_strip(start, along, length, width, me.pos, MARGIN) {
                    // In the lane: crouch as the front comes.
                    let gone = if active { fight::front(left) } else { Fx::ZERO };
                    let mine = flat(me.pos.sub(start)).dot(along);
                    let ahead = mine.sub(gone);
                    if (active || due < 30)
                        && ahead.raw() < CROUCH_FRONT.raw()
                        && ahead.raw() > Fx::from_int(-3).raw()
                    {
                        self.intent = CROUCH;
                        return Some(wire(
                            me,
                            yaw_of(along.scale(Fx::ONE.neg())),
                            None,
                            Input::CROUCH,
                        ));
                    }
                    self.intent = CROUCH;
                    return Some(wire(me, look_at(start), None, Input::CROUCH));
                }
                // Beside it: the wing goes over. Swing up at it.
                let near = in_strip(start, along, length, width, me.pos, BESIDE);
                if near && active {
                    let wing = nearest_wing(s, me.pos, false).1;
                    if sim::math::wide_len(wing.sub(me.pos)).raw() < Fx::from_int(5).raw() {
                        self.intent = WING;
                        return Some(self.swing(me, wing, 0));
                    }
                }
                None
            }
            gw::DOWNWASH => {
                let under = fight::wash_point(seen);
                let r = Knob::WashRadius.fx();
                if wide_flat_dist(me.pos, under).raw() > r.add(MARGIN).raw() {
                    return None;
                }
                self.intent = LEE;
                // The eye, or a lee: behind the nearest stone or the tower,
                // from the point under it -- whichever it can reach.
                let eye = under;
                let mut best = (eye, wide_flat_dist(me.pos, eye));
                for solid in w.arena().solids().iter() {
                    let size = solid.max.sub(solid.min);
                    let tall = solid.max.y.sub(fight::base(w));
                    if solid.hangs()
                        || tall.raw() < Fx::from_int(2).raw()
                        || size.x.raw() > Fx::from_int(8).raw()
                        || size.z.raw() > Fx::from_int(8).raw()
                    {
                        continue;
                    }
                    let mid = flat(solid.min.add(solid.max).scale(crate::HALF));
                    let away = unit(mid.sub(under), V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO));
                    let half = size.x.max(size.z).mul(crate::HALF);
                    let lee = mid.add(away.scale(half.add(Fx::ONE)));
                    let d = wide_flat_dist(me.pos, lee);
                    if d.raw() < best.1.raw() {
                        best = (lee, d);
                    }
                }
                let reach = sim::tuning::move_speed().mul(Fx::ratio(50, 60));
                if best.1.raw() > reach.add(Fx::from_int(3)).raw() && !active {
                    // Too far: crouch where it stands.
                    self.intent = CROUCH;
                    return Some(wire(me, look_at(under), None, Input::CROUCH));
                }
                let at = V3::new(best.0.x, me.pos.y, best.0.z);
                let bits = if active { Input::CROUCH } else { 0 };
                Some(walk(me, at, Some(under), bits))
            }
            gw::VOLLEY => {
                let (start, along, length, width) = fight::rake_lane(&seen.lore);
                if !in_strip(start, along, length, width, me.pos, MARGIN) {
                    return None;
                }
                self.intent = OUT;
                let out = out_of_strip(w, start, along, me.pos);
                let to = me.pos.add(out.scale(width.add(Fx::from_int(3))));
                Some(walk(me, to, Some(start), 0))
            }
            gw::SCREECH => {
                let inside = (0..fight::CONE_DISCS).any(|i| {
                    let (at, r) = fight::cone_disc(s, i);
                    wide_flat_dist(me.pos, at).raw() < r.add(MARGIN).raw()
                });
                if !inside {
                    return None;
                }
                self.intent = OUT;
                let facing = V3::from_turns(s.yaw);
                let side = V3::new(facing.z.neg(), Fx::ZERO, facing.x);
                let rel = flat(me.pos.sub(s.pos));
                let (near, far) = if rel.dot(side).raw() >= 0 {
                    (side, side.scale(Fx::ONE.neg()))
                } else {
                    (side.scale(Fx::ONE.neg()), side)
                };
                let side = if clear(w, me.pos, near) { near } else { far };
                let to = me.pos.add(side.scale(Fx::from_int(6)));
                Some(walk(me, to, Some(s.pos), 0))
            }
            gw::BUFFET => {
                let inside = (0..fight::BUFFET_DISCS).any(|i| {
                    let (at, r) = fight::buffet_disc(s, i);
                    wide_flat_dist(me.pos, at).raw() < r.add(MARGIN).raw()
                });
                if !inside {
                    return None;
                }
                let began = seen
                    .frame
                    .wrapping_sub((a.startup - left.min(a.startup)) as u32);
                if self.answered != Some((kind, began)) && due <= JUMP_LEAD + self.slop {
                    self.answered = Some((kind, began));
                    self.roll_slop();
                    self.leap_left = LEAP_HOLD;
                }
                self.intent = JUMP;
                Some(wire(me, look_at(s.pos), None, 0))
            }
            _ => None,
        }
    }

    /// **Down, and in reach**: walk to the nearest wing and hit it -- up a
    /// crashed bird's wing onto its back.
    fn punish(&mut self, me: &Player, s: &Monster) -> Option<Input> {
        let gathering = matches!(s.doing, Doing::Startup { kind: gw::LIFT, .. });
        if (fight::aloft(s) && !gathering) || fight::perched(s) {
            return None;
        }
        // Not inside what it is about to throw: that was the answers' to say.
        if matches!(s.doing, Doing::Startup { .. } | Doing::Active { .. })
            && !matches!(s.doing.attacking(), Some(gw::LIFT))
        {
            return None;
        }
        let crashed = fight::crashed(s);
        let (part, at) = nearest_wing(s, me.pos, false);
        self.intent = WING;
        // The window it can see: what is left of the crash or the dwell.
        self.aimed = Some((at, Some(s.frames_until_free() as i32 - REACTION as i32)));
        if crashed {
            // Up the wing: walk at the root, along the wing.
            let (root, _) = nearest_wing(s, me.pos, true);
            let up = top_of(s, root);
            if wide_flat_dist(me.pos, up).raw() > Fx::from_int(2).raw() {
                return Some(walk(me, up, Some(up), 0));
            }
            return Some(self.swing(me, part_at(s, root), 0));
        }
        // **Plan B boards it** as it gathers itself to lift -- swinging at
        // it until then, as plan A does: up the nearest wing or the back,
        // hopping onto it, and the ride takes it from there.
        if self.gamble == Gamble::B && gathering && !fight::grounded_for_good(s) {
            // The lowest top in reach of its wings and back, nearest first.
            let wing = WINGS
                .iter()
                .chain([gw::BACK].iter())
                .map(|p| top_of(s, *p))
                .filter(|at| at.y.sub(me.pos.y).raw() < self.hop.raw())
                .min_by_key(|at| wide_flat_dist(*at, me.pos).raw())
                .unwrap_or(at);
            self.intent = RIDE;
            // Leave the floor early enough to be over the edge before it is
            // reached: a jump into the side of a wing is a jump off it.
            let near = wide_flat_dist(me.pos, wing).raw() < Fx::ratio(25, 10).raw();
            let above = wing.y.sub(me.pos.y).raw() > Fx::ratio(4, 10).raw();
            if near && above && me.grounded && self.leap_left == 0 {
                self.leap_left = LEAP_HOLD;
            }
            return Some(walk(me, wing, Some(wing), 0));
        }
        let close = sim::math::wide_len(at.sub(me.pos));
        let poke = self.hands.reach(me);
        if close.raw() <= poke.add(Fx::ONE).raw() {
            return Some(self.swing(me, at, 0));
        }
        let _ = part;
        // On the way to it: the class's own business with the window.
        let window = s.frames_until_free() as i32 - REACTION as i32;
        self.waiting = Some((s.pos, at, window));
        Some(walk(me, at, Some(at), 0))
    }

    /// **The perch**: up the tower's ledges, if it was near the first one
    /// when the announcement began -- and on top, at it.
    fn climb(&mut self, w: &World, seen: &World, me: &Player, s: &Monster) -> Option<Input> {
        let ledges = ledges(w);
        if ledges.is_empty() {
            return None;
        }
        let announcing = matches!(
            s.doing,
            Doing::Startup {
                kind: gw::PERCH,
                ..
            }
        );
        if self.climbing.is_none() && announcing {
            let first = onto(&ledges[0], me.pos);
            if wide_flat_dist(me.pos, first).raw() < STAIRS_NEAR.raw() {
                self.climbing = Some(0);
            }
        }
        let k = self.climbing?;
        if !(announcing || fight::perched(s)) {
            // It has gone: back down by walking off.
            self.climbing = None;
            return None;
        }
        self.intent = CLIMB;
        // Which ledge it is on: the highest whose top it stands on.
        let mut on = None;
        for (i, l) in ledges.iter().enumerate() {
            let over = me.pos.x.raw() >= l.min.x.raw()
                && me.pos.x.raw() <= l.max.x.raw()
                && me.pos.z.raw() >= l.min.z.raw()
                && me.pos.z.raw() <= l.max.z.raw();
            if over && me.pos.y.sub(l.max.y).abs().raw() < Fx::ratio(3, 10).raw() {
                on = Some(i);
            }
        }
        let next = on.map_or(k, |i| (i + 1).max(k));
        self.climbing = Some(next);
        if next >= ledges.len() {
            // On top: at it.
            let m = bird(seen)?;
            return Some(self.swing(me, part_at(&m, gw::BACK), 0));
        }
        let target = onto(&ledges[next], me.pos);
        let gap = wide_flat_dist(me.pos, target);
        if gap.raw() < Fx::from_int(2).raw()
            && me.grounded
            && self.leap_left == 0
            && target.y.raw() > me.pos.y.add(Fx::ratio(3, 10)).raw()
            // A ledge it can hop onto: inside its own jump.
            && target.y.sub(me.pos.y).raw() <= self.hop.raw()
        {
            self.leap_left = LEAP_HOLD;
        }
        Some(walk(me, target, None, 0))
    }

    /// **Something that reaches it from here**: the Elementalist hops and
    /// fires her air bolt at the circling bird; anybody throws what reaches.
    fn shoot(&mut self, seen: &World, me: &Player, s: &Monster) -> Option<Input> {
        if !fight::aloft(s) || s.doing.attacking().is_some_and(gw::aerial) {
            return None;
        }
        let body = part_at(s, gw::BACK);
        let from = sim::aim::origin(me.pos);
        let far = sim::math::wide_len(body.sub(from));
        match me.class {
            Class::Elementalist => {
                // Air bolt: 22 m, thrown from a hop.
                if far.raw() > Fx::from_int(24).raw() {
                    return None;
                }
                self.intent = SHOOT;
                if me.grounded && self.leap_left == 0 && self.cooldown == 0 {
                    self.leap_left = LEAP_HOLD;
                }
                let yaw = yaw_of(body.sub(me.pos));
                let bits = if !me.grounded && me.action.actionable() && self.cooldown == 0 {
                    self.cooldown = SWING_GAP;
                    Input::LEFT
                } else {
                    0
                };
                let _ = seen;
                Some(wire(me, yaw, Some(body), bits))
            }
            _ => None,
        }
    }

    /// **Home**: in the open, looking at where the bird is, a little down so
    /// the floor round it is on its screen.
    fn home(&mut self, w: &World, me: &Player, s: &Monster) -> Input {
        self.intent = HOME;
        let base = fight::base(w);
        let home = V3::new(Fx::from_int(HOME_AT.0), base, Fx::from_int(HOME_AT.1));
        let look = V3::new(s.pos.x, me.pos.y, s.pos.z);
        if wide_flat_dist(me.pos, home).raw() > HOME_SLACK.raw() {
            return walk(me, home, Some(look), 0);
        }
        // At home with the bird down and nothing begun: the class's own
        // business.
        if !fight::aloft(s) && s.doing.attacking().is_none() {
            self.waiting = Some((s.pos, part_at(s, gw::BACK), i32::MAX));
        }
        wire(me, yaw_of(look.sub(me.pos)), None, 0)
    }
}

/// Make a plan-A hunter.
fn make(who: usize, seed: u32, hop: Fx) -> Box<dyn Plan + Send + Sync> {
    Box::new(Galewing::new(who, seed, hop, Gamble::A))
}

/// Make a plan-B hunter: it rides.
fn make_rider(who: usize, seed: u32, hop: Fx) -> Box<dyn Plan + Send + Sync> {
    Box::new(Galewing::new(who, seed, hop, Gamble::B))
}

/// The roll throws riders.
pub fn bucks(kind: u8) -> bool {
    kind == gw::ROLL
}

/// The class's heavy, for whoever wants it.
pub fn heavy_of(class: Class) -> u16 {
    heavy(class)
}

// ---------------------------------------------------------------------------
// The report's lines
// ---------------------------------------------------------------------------

/// How far each class reaches the bird from where it stands, by what it can
/// throw (`galewing.md` §6): the longest skillshot it has, and its swing
/// from the top of its hop. Instrument numbers, read off the document's
/// table; the Elementalist's air bolt needs a hop.
fn ranged(class: Class) -> Fx {
    match class {
        Class::Elementalist => Fx::from_int(22),
        Class::Champion => Fx::ZERO,
        Class::Bulwark => Fx::from_int(9),
        Class::ShadowReaver => Fx::from_int(9),
        Class::BloodMage => Fx::from_int(12),
        Class::DualMage => Fx::from_int(7),
    }
}

/// **What the report counts about the Galewing** (`galewing.md` §9).
#[derive(Default)]
pub struct GaleTally {
    fought: u32,
    out_of_reach: u32,
    /// The bird out of the first hunter's reach this frame: what the four
    /// windows leave out (§9, "of the frames it is in reach").
    out_now: bool,
    /// Wing damage per wing, from the bars.
    wing_damage: [i32; 2],
    /// Passes thrown and the wing hits taken inside them.
    passes: u32,
    pass_hits: u32,
    in_pass: bool,
    /// Rides: begun, frames, and how each ended.
    rides: u32,
    ride_frames: u32,
    riding: bool,
    ended: [u32; 4],
    /// Fall damage by cause: talon drop, thrown, off the tower, off the edge,
    /// any other.
    falls: [i32; 5],
    /// Last footing per hunter: 0 floor, 1 the tower, 2 aboard, 3 carried.
    footing: [u8; 4],
    thrown: [bool; 4],
    /// Unseen tells: the move in progress, whom at, its marker's frames on
    /// their screen; and hits whose marker was under fifteen.
    watch: Option<(u8, [u32; 4])>,
    unseen: u32,
    /// The bird's lore at the end, for its own counts.
    crashes: [u32; 4],
    carries: u32,
    freed: u32,
    low_passes: u32,
    perches: u32,
    frames: u32,
    /// The first hunter's class and its hop, measured once.
    hop: Option<(Class, Fx)>,
}

/// Marker points for the move in progress, from what is drawn.
fn marker_points(w: &World, m: &Monster) -> Vec<V3> {
    let mut out = Vec::new();
    if let Some(t) = m.telegraph() {
        let r = t.radius;
        out.push(t.anchor);
        for d in [
            V3::new(r, Fx::ZERO, Fx::ZERO),
            V3::new(r.neg(), Fx::ZERO, Fx::ZERO),
            V3::new(Fx::ZERO, Fx::ZERO, r),
            V3::new(Fx::ZERO, Fx::ZERO, r.neg()),
        ] {
            out.push(t.anchor.add(d));
        }
    }
    for s in w.signs().iter() {
        out.push(s.at);
        if s.length.raw() > 0 {
            out.push(s.at.add(s.along.scale(s.length.mul(crate::HALF))));
            out.push(s.at.add(s.along.scale(s.length)));
        } else {
            // A disc's rim as well as its middle: a disc half behind a ledge
            // is still on the screen.
            let r = s.width.mul(crate::HALF);
            for d in [
                V3::new(r, Fx::ZERO, Fx::ZERO),
                V3::new(r.neg(), Fx::ZERO, Fx::ZERO),
                V3::new(Fx::ZERO, Fx::ZERO, r),
                V3::new(Fx::ZERO, Fx::ZERO, r.neg()),
            ] {
                out.push(s.at.add(d));
            }
        }
    }
    out
}

impl Tally for GaleTally {
    fn observe(&mut self, before: &World, after: &World) {
        self.observe_with(before, after, &[]);
    }

    /// The four windows are asked of the frames it is in reach (§9): a bird
    /// circling out of reach is neither offering an opening nor refusing
    /// one, and counted as threatening -- free to act -- it made the windows
    /// four fifths threatening for the whole fight.
    fn windowed(&self, w: &World) -> bool {
        let _ = w;
        !self.out_now
    }

    /// **The gather and the lift, and the flight to the perch, are not
    /// answers.** Neither does damage or moves anybody: a bird gathering
    /// itself off the floor for 45 frames, wings down and in reach, is the
    /// end of the walk-up the Stoop and the crash open, not a threat -- as
    /// the Veilstalker's retreat is not. Until it can next hit: what is left
    /// of the move, its recovery, and the pause before it decides.
    fn until_free(&self, w: &World, slot: usize, free: u16) -> u16 {
        let Some(m) = w.monsters[slot] else {
            return free;
        };
        match m.doing {
            Doing::Startup { kind, left }
            | Doing::Active { kind, left }
            | Doing::Recovery { kind, left }
                if kind == gw::LIFT || kind == gw::PERCH =>
            {
                let a = m.sp().attack(kind);
                let rest = match m.doing {
                    Doing::Startup { .. } => left + a.active + a.recovery,
                    Doing::Active { .. } => left + a.recovery,
                    _ => left,
                };
                rest.saturating_add(m.sp().think_frames())
            }
            _ => free,
        }
    }

    fn observe_with(&mut self, before: &World, after: &World, bots: &[Hunter]) {
        self.frames += 1;
        let Some(slot) = fight::slot_of(after) else {
            return;
        };
        let (Some(was), Some(now)) = (before.monsters[slot], after.monsters[slot]) else {
            return;
        };
        if now.brain.grace == 0 {
            self.fought += 1;
        }
        let stones = sim::stones::gather(&after.players);
        let ground = after.terrain();
        let scene = Scene {
            stones: &stones,
            players: &after.players,
            effects: &after.effects,
            quarry: &after.monsters,
            critters: &after.critters,
            arena: &ground,
        };
        // Out of reach: nothing the first hunter has touches it from where
        // it stands -- its swing from the top of a hop, or its longest
        // throw.
        self.out_now = false;
        if let Some(bot) = bots.first() {
            let p = after.players[bot.who];
            if self.hop.is_none_or(|(c, _)| c != p.class) {
                self.hop = Some((p.class, crate::jump_apex(p.class)));
            }
            let hop = self.hop.map_or(Fx::ZERO, |(_, h)| h);
            if p.health > 0 && now.alive() && !p.aboard() {
                let feet = p.pos;
                let swing_top = feet
                    .add(V3::new(Fx::ZERO, sim::tuning::body_height(), Fx::ZERO))
                    .add(V3::new(Fx::ZERO, hop, Fx::ZERO));
                let nearest = now.nearest_to(swing_top);
                let melee =
                    sim::math::wide_len(nearest.sub(swing_top)).raw() <= Fx::from_int(3).raw();
                let throw = sim::math::wide_len(
                    now.nearest_to(sim::aim::origin(feet))
                        .sub(sim::aim::origin(feet)),
                )
                .raw()
                    <= ranged(p.class).raw();
                if !(melee || throw) && now.brain.grace == 0 {
                    self.out_of_reach += 1;
                    self.out_now = true;
                }
            }
        }
        // Wing damage.
        for side in 0..2 {
            let lost = fight::bar(&was, side) - fight::bar(&now, side);
            if lost > 0 {
                self.wing_damage[side] += lost;
                if self.in_pass {
                    self.pass_hits += 1;
                }
            }
        }
        // Passes: a talon pass or a Stoop, from its commit to the end of its
        // recovery.
        let pass = matches!(now.doing.attacking(), Some(gw::TALON | gw::STOOP));
        if pass && !matches!(was.doing.attacking(), Some(gw::TALON | gw::STOOP)) {
            self.passes += 1;
        }
        self.in_pass = pass;
        // Rides.
        let aboard = after.players.iter().any(|p| p.health > 0 && p.aboard());
        if aboard {
            self.ride_frames += 1;
        }
        if aboard && !self.riding {
            self.rides += 1;
        }
        if self.riding && !aboard {
            let (_, phase, _) = fight::ride_state(&before.lore);
            let thrown = after
                .players
                .iter()
                .any(|p| matches!(p.action, Action::HitStun { .. }));
            let crashed_down = matches!(now.doing, Doing::Toppled { .. }) && !fight::aloft(&now);
            let i = if thrown {
                1
            } else if phase == fight::ride::SWOOP {
                0
            } else if crashed_down {
                2
            } else {
                3
            };
            self.ended[i] += 1;
        }
        self.riding = aboard;
        // Falls, by where the fall began.
        for i in 0..after.players.len().min(4) {
            let (a, b) = (&before.players[i], &after.players[i]);
            if b.health <= 0 {
                continue;
            }
            if fight::carried(&after.lore) == Some(i) {
                self.footing[i] = 3;
            } else if b.aboard() {
                self.footing[i] = 2;
            } else if b.grounded {
                let tower = fight::perch_top(after).is_some_and(|top| {
                    b.pos.y.raw() > fight::base(after).add(Fx::from_int(2)).raw()
                        && wide_flat_dist(b.pos, top).raw() < Fx::from_int(7).raw()
                });
                if !a.grounded {
                    let d = sim::state::landing_damage(a, b);
                    if d > 0 {
                        let cause = match self.footing[i] {
                            3 => 0,
                            2 if self.thrown[i] => 1,
                            1 => 2,
                            0 if b.pos.y.raw() < fight::base(after).sub(Fx::from_int(4)).raw() => 3,
                            _ => 4,
                        };
                        self.falls[cause] += d;
                    }
                }
                self.footing[i] = if tower { 1 } else { 0 };
                self.thrown[i] = false;
            }
            if a.aboard() && !b.aboard() && matches!(b.action, Action::HitStun { .. }) {
                self.thrown[i] = true;
            }
        }
        // **Unseen tells**: frames each hunter had the move's marker on
        // their screen, through its windup.
        match now.doing {
            Doing::Startup { kind, .. } => {
                let fresh = !matches!(self.watch, Some((k, _)) if k == kind)
                    || was.doing.attacking() != Some(kind);
                if fresh {
                    self.watch = Some((kind, [0; 4]));
                }
                let points = marker_points(after, &now);
                if let Some((_, frames)) = self.watch.as_mut() {
                    for bot in bots {
                        if points
                            .iter()
                            .any(|p| on_screen(bot.who, bot.last, *p, HALF_VIEW, &scene))
                        {
                            frames[bot.who.min(3)] += 1;
                        }
                    }
                }
            }
            Doing::Active { kind, .. } => {
                if let Some((k, frames)) = self.watch {
                    if k == kind {
                        for (i, seen) in frames.iter().enumerate().take(after.players.len().min(4))
                        {
                            // Struck by it: hurt and knocked into stun. A
                            // Blood mage's own price and a hard landing hurt
                            // without either.
                            let stun = |a: &Action| match *a {
                                Action::HitStun { left } => Some(left),
                                Action::Stagger { left, .. } => Some(left),
                                _ => None,
                            };
                            let fresh = match (
                                stun(&before.players[i].action),
                                stun(&after.players[i].action),
                            ) {
                                (_, None) => false,
                                (None, Some(_)) => true,
                                (Some(b), Some(a)) => a > b,
                            };
                            let hit = after.players[i].health < before.players[i].health
                                && fresh
                                && gw::SPECIES.attack(k).damage > 0
                                && !before.players[i].aboard();
                            let watched = bots.iter().any(|b| b.who == i);
                            if hit
                                && watched
                                && *seen < REACTION as u32
                                && !gw::MOVES[k as usize].harmless
                            {
                                self.unseen += 1;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        // Its own counts, read at the end.
        let lore = &after.lore;
        for c in 0..4 {
            self.crashes[c] = fight::byte_of(lore, fight::word::CRASHES, c);
        }
        let carries = lore.word(fight::word::CARRIES);
        self.carries = carries & 0xFFFF;
        self.freed = carries >> 16;
        let passes = lore.word(fight::word::PASSES);
        self.low_passes = passes & 0xFFFF;
        self.perches = passes >> 16;
    }

    fn lines(&self) -> Vec<(String, String, String)> {
        let minutes = (self.fought.max(1) as f32) / 3600.0;
        let worst = self.wing_damage[0].max(self.wing_damage[1]);
        let mean_ride = if self.rides > 0 {
            self.ride_frames as f32 / self.rides as f32 / 60.0
        } else {
            0.0
        };
        vec![
            (
                "out of reach".into(),
                format!(
                    "{:.0}%",
                    100.0 * self.out_of_reach as f32 / self.fought.max(1) as f32
                ),
                "frames nothing this class can do from where it stands touches it -- at most a third".into(),
            ),
            (
                "low passes".into(),
                format!(
                    "{} ({:.1} a minute), {:.1} wing hits a pass",
                    self.passes,
                    self.passes as f32 / minutes,
                    self.pass_hits as f32 / self.passes.max(1) as f32
                ),
                "whether the openings are openings".into(),
            ),
            (
                "wing damage".into(),
                format!(
                    "left {}, right {}, worst {}",
                    self.wing_damage[0], self.wing_damage[1], worst
                ),
                "wings broken: 0 has two causes, and this tells them apart".into(),
            ),
            (
                "crashes".into(),
                format!(
                    "{} (pass {}, Stoop {}, clipped {}, wing broke {})",
                    self.crashes.iter().sum::<u32>(),
                    self.crashes[0],
                    self.crashes[1],
                    self.crashes[2],
                    self.crashes[3]
                ),
                "what earned each".into(),
            ),
            (
                "rides".into(),
                format!(
                    "{} (mean {:.1} s): swoop {}, thrown {}, rode it down {}, other {}",
                    self.rides, mean_ride, self.ended[0], self.ended[1], self.ended[2], self.ended[3]
                ),
                "how each ended".into(),
            ),
            (
                "fall damage".into(),
                format!(
                    "talon drop {}, thrown {}, off the tower {}, off the edge {}, other {}",
                    self.falls[0], self.falls[1], self.falls[2], self.falls[3], self.falls[4]
                ),
                "what the heights cost, by cause".into(),
            ),
            (
                "carries".into(),
                format!("{} ({} freed early)", self.carries, self.freed),
                "whether the leg answer is ever used".into(),
            ),
            (
                "perches".into(),
                format!("{}", self.perches),
                "the quiet window".into(),
            ),
            (
                "unseen tells".into(),
                format!("{}", self.unseen),
                "hits whose marker was never on the victim's screen for 15 frames -- must be zero".into(),
            ),
        ]
    }

    fn unanswerable(&self) -> u32 {
        self.unseen
    }
}

fn make_tally() -> Box<dyn Tally> {
    Box::new(GaleTally::default())
}

pub static CARD: crate::plans::Card = crate::plans::Card {
    species: sim::species::SpeciesId::GALEWING,
    plan: make,
    bucks,
    words: crate::plans::Words {
        weak_hits: "wing hits while it was low (its poise)",
        broken: "parts broken (none: its wings are bars of their own)",
        into_breakables: "damage into breakable parts",
        into_breakables_why: "none: see wing damage below",
        worst: "worst part",
        ride_for: "the sky ride",
        toppled_pool: "while crashed",
    },
    tally: Some(make_tally),
    gamble: Some(make_rider),
};
