//! The Hornback's hunter: what a person learns in their first ten minutes
//! against the herd (hornback.md §9), played the way a person plays it --
//! seeing everything `REACTION` frames late, and aiming with the crosshair on
//! the body (`sim::aim::look_onto_closely`).
//!
//! The plan, in the document's order:
//!
//! 1. **Do not stand in open grass.** Keep the nearest solid two to four
//!    metres behind you, on the line from the bull through you.
//! 2. **Watch for the second paw.** Walk out of the lane on the head-drop,
//!    toward the side away from the lane's middle. Never before it.
//! 3. **In a stun, go to the head and hit it.** Leave with twenty frames to
//!    spare.
//! 4. **On a bellow, go to the nearest lee** -- or out of the lane, whichever
//!    is nearer across it.
//! 5. **Never stand still inside a cow's rear wedge.**
//! 6. **Poke fast from the front and swing heavy from the flank. Dodge the
//!    hook toward the flank** its head is not cocked to; hit the shoulder's
//!    lean; go round a guard.
//! 7. **After a knockdown, walk.** Do not swing.
//!
//! Before any of it, the herd has to be roused: walk at it until it is.
//! And the Elementalist's own play (§7): a charge whose lane ends in no solid
//! gets a stone raised into it after the head drops.
//!
//! **The crossing** (§11) is the same plan with one more rule over where to
//! stand: beside the cart, between it and the bull, so the cart rolls and the
//! bull's nearest threat is a hunter; past the cart's escort distance -- so it
//! stops -- whenever a lane drawn on the road is about to take it; and out of
//! line with the cart while a charge winds up, so the lane the bull locks is
//! away from the road.
//!
//! **Riding is plan v2** (§9): after a bellow, one hunt in three it catches a
//! returning cow and jumps off behind the bull, so the ride's numbers exist.
//!
//! It does not read the herd's mind. It sees the bull's head (the paw, the
//! drop, the cock, the brace, the lean, the call), the herd, and what is drawn
//! on the floor -- the lane, its lees, the ring on the rock that will stop a
//! charge -- and nothing about which move comes next.

use sim::critter::{CritterField, MAX_CRITTERS, is, stat_fx};
use sim::fixed::Fx;
use sim::math::atan2_turns;
use sim::sign::{Says, Shape, Signs};
use sim::species::hornback as h;
use sim::state::{Action, MAX_PLAYERS, Phase};
use sim::{Class, Input, V3, World};

use crate::{Intent, Plan, REACTION, steer, turns_to_aim};

pub const ROUSE: Intent = Intent("Rouse");
pub const POST: Intent = Intent("Post");
pub const HOLD: Intent = Intent("Hold");
pub const STEP_OUT: Intent = Intent("StepOut");
pub const LEE: Intent = Intent("Lee");
pub const HEAD: Intent = Intent("Head");
pub const POKE: Intent = Intent("Poke");
pub const ROUND: Intent = Intent("Round");
pub const DODGE: Intent = Intent("Dodge");
pub const WALK: Intent = Intent("Walk");
pub const WEDGE: Intent = Intent("Wedge");
pub const RAISE: Intent = Intent("Raise");
pub const RIDE: Intent = Intent("Ride");
pub const ESCORT: Intent = Intent("Escort");
pub const STOP: Intent = Intent("Stop");
pub const BAIT: Intent = Intent("Bait");

/// How far in front of the rock it stands: two to four metres (§9 step 1).
const OFF_ROCK: Fx = Fx::from_raw(3 << 16);
/// Close enough to its post to stop walking to it.
const AT_POST: Fx = Fx::ratio(8, 10);
/// It counts as in a charge's lane within this of the lane's line, past the
/// bull's own half-width.
const LANE_SLACK: Fx = Fx::ratio(15, 10);
/// How far it walks out to the side of a lane.
const OUT_OF_LANE: Fx = Fx::from_raw(4 << 16);
/// A charge's nose this near, still in the lane: dodge.
const DODGE_AT: Fx = Fx::from_raw(5 << 16);
/// Frames between presses: a person does not mash.
const SWING_GAP: u32 = 8;
/// Leave a stun this long before it ends (§9 step 3).
const SPARE: u32 = 20;
/// It walks this long after getting up (§9 step 7).
const WALK_AFTER: u32 = 24;
/// Nearer than this, a hook is dodged through toward the flank.
const HOOK_THROUGH: Fx = Fx::ratio(32, 10);
/// A hook seen winding up this near, in front: dodge it.
const HOOK_NEAR: Fx = Fx::from_raw(5 << 16);
/// A post nearer the bull than this, or a way there passing nearer, costs.
const KEEP_OFF: Fx = Fx::from_raw(7 << 16);
/// A rider jumps off this long before the second buck.
const LEAVE_BEFORE: u32 = 24;
/// A returning cow this near is one to catch.
const CATCH_FROM: Fx = Fx::from_raw(9 << 16);
/// How far ahead of a cow its path is read, in seconds.
const CATCH_LEAD: Fx = Fx::from_raw(1 << 16);
/// Standing this near its path, it is in the way.
const CATCH_OFF: Fx = Fx::ratio(5, 10);
/// Jump when it arrives within this many seconds, and not sooner than this.
const CATCH_JUMP: Fx = Fx::ratio(7, 10);
const CATCH_SOON: Fx = Fx::ratio(3, 10);
/// Frames the jump is held.
const JUMP_HOLD: u32 = 30;
/// A post this near the one the bull is wary of is the same rock.
const SPENT_NEAR: Fx = Fx::from_raw(3 << 16);
/// What standing at the bank costs, in metres of walking, over a boulder.
const BANK_COST: Fx = Fx::from_raw(6 << 16);
/// Escorting, it stands this far off the road on the bull's side...
const BESIDE_CART: Fx = Fx::from_raw(3 << 16);
/// ...and this far ahead of the cart along it: off the line from the bull
/// through the cart, by more than the cart rolls while a charge comes.
const CART_LEAD: Fx = Fx::ratio(45, 10);
/// The cart is stopped when a lane drawn on the road is this near its front.
const STOP_SHORT: Fx = Fx::from_raw(4 << 16);
/// Stopping it, it stands this far past the escort distance.
const PAST_ESCORT: Fx = Fx::ratio(15, 10);
/// How far a stone is raised in front of the Elementalist into a lane.
const STONE_AHEAD: Fx = Fx::from_raw(5 << 16);

/// The bull, as the hunter remembers seeing it.
#[derive(Clone, Copy, Default)]
struct Bull {
    alive: bool,
    pos: V3,
    facing: V3,
    middle: V3,
    state: u8,
    act: u8,
    timer: u16,
    /// Its head cocked to its right: the hook comes from there.
    right: bool,
    stunned: bool,
}

/// One frame of what it saw.
#[derive(Clone, Copy, Default)]
struct Seen {
    bull: Bull,
    /// Each cow: alive, where, its velocity, its state and move.
    cows: [(bool, V3, V3, u8, u8); MAX_CRITTERS],
    returning: bool,
    signs: Signs,
    calm: bool,
    stampede: bool,
}

pub struct Hornback {
    who: usize,
    memory: Vec<Seen>,
    at: usize,
    filled: usize,
    intent: Intent,
    cooldown: u32,
    dodge_left: u32,
    /// Frames of walking left after getting up.
    walk_left: u32,
    was_down: bool,
    /// When the stun it is spending began, if it is spending one.
    stun_from: Option<u32>,
    /// The rock it was last at when the bull was stunned: wary of it, so the
    /// next post is another (§4).
    spent: Option<V3>,
    /// The post it last stood at.
    posted: Option<V3>,
    /// When it last saw the bull stunned.
    stunned_at: Option<u32>,
    /// A stone already raised into this charge.
    raised_for: bool,
    /// Rides it means to take: one hunt in three (§9).
    rides: bool,
    /// Frames on a cow's back this ride.
    rode: u32,
    /// Frames of a jump still held.
    jumping: u32,
    rng: u32,
}

impl Hornback {
    pub fn new(who: usize, seed: u32) -> Hornback {
        let mut rng = seed | 1;
        rng ^= rng << 13;
        rng ^= rng >> 17;
        rng ^= rng << 5;
        Hornback {
            who,
            memory: vec![Seen::default(); REACTION + 1],
            at: 0,
            filled: 0,
            intent: ROUSE,
            cooldown: 0,
            dodge_left: 0,
            walk_left: 0,
            was_down: false,
            stun_from: None,
            posted: None,
            stunned_at: None,
            spent: None,
            raised_for: false,
            rides: rng % 3 == 0,
            rode: 0,
            jumping: 0,
            rng,
        }
    }

    fn recall(&self) -> Seen {
        let back = REACTION.min(self.filled.saturating_sub(1));
        let idx = (self.at + self.memory.len() - 1 - back) % self.memory.len();
        self.memory[idx]
    }

    fn roll(&mut self) -> u32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        self.rng
    }
}

fn flat(v: V3) -> V3 {
    V3::new(v.x, Fx::ZERO, v.z)
}

/// The move a left click throws on the floor: the class's auto.
fn auto_slot(class: Class) -> u8 {
    match class {
        Class::BloodMage => sim::moves::blood::SWEEP,
        _ => 0,
    }
}

/// The Dual mage's lower bar's hand, as the Gnawers' hunter throws it.
fn auto_button(me: &sim::state::Player) -> u16 {
    match sim::dual::bars(me) {
        Some((dark, light)) if dark.raw() < light.raw() => Input::LEFT,
        Some(_) => Input::RIGHT,
        None => Input::LEFT,
    }
}

/// The solids it can stand in front of: the standing boulders, and any stone
/// or shield of its own, as centres and how far across.
fn rocks(w: &World) -> Vec<Rock> {
    let mut out = Vec::new();
    let ground = w.terrain();
    let sp = sim::species::lookup(sim::species::SpeciesId::HORNBACK).unwrap();
    for p in ground.floor.iter() {
        if p.kind == h::BOULDER || p.kind == h::CRACKED {
            if let Some(s) = p.solid(sp) {
                out.push(Rock::Box(s.min, s.max, Fx::ZERO));
            }
        }
    }
    // The bank's face, and anything else the arena stands that a charge
    // stops at -- but not the edge, which it pulls up short of. Further off
    // in the choosing than a boulder: a person goes to the rock in front of
    // them first.
    let b = ground.bounds;
    for s in ground.arena.solids() {
        let edge = s.max.x.raw() <= b.lo_x.raw()
            || s.min.x.raw() >= b.hi_x.raw()
            || s.max.z.raw() <= b.lo_z.raw()
            || s.min.z.raw() >= b.hi_z.raw();
        if !edge && s.max.y.raw() > Fx::ONE.raw() {
            out.push(Rock::Box(s.min, s.max, BANK_COST));
        }
    }
    for s in sim::stones::gather(&w.players).iter().flatten() {
        if s.standing_height().raw() > Fx::ONE.raw() {
            out.push(Rock::Disc(V3::new(s.at.x, Fx::ZERO, s.at.z), s.radius()));
        }
    }
    out
}

/// A solid to stand in front of.
#[derive(Clone, Copy)]
enum Rock {
    /// A box's footprint, and what choosing it costs over a boulder.
    Box(V3, V3, Fx),
    Disc(V3, Fx),
}

impl Rock {
    /// Where to stand: `OFF_ROCK` off the face nearest the bull, on the line
    /// from the bull through you.
    fn post(&self, bull: V3, side: V3) -> V3 {
        let (near, out) = match *self {
            Rock::Box(min, max, _) => {
                let x = bull.x.clamp(min.x, max.x);
                let z = bull.z.clamp(min.z, max.z);
                (V3::new(x, Fx::ZERO, z), Fx::ZERO)
            }
            Rock::Disc(c, r) => (c, r),
        };
        let from = flat(bull.sub(near));
        let dir = if from.flat_len().raw() > 0 {
            from.normalized()
        } else {
            side
        };
        near.add(dir.scale(out.add(OFF_ROCK)))
    }

    fn cost(&self) -> Fx {
        match *self {
            Rock::Box(_, _, c) => c,
            Rock::Disc(..) => Fx::ZERO,
        }
    }
}

impl Plan for Hornback {
    fn watch(&mut self, w: &World) {
        let sp = w.critters.sp();
        let mut seen = Seen {
            signs: w.signs(),
            ..Seen::default()
        };
        if let Some(pack) = w.pack {
            seen.calm = pack.mood == sim::pack::mood::CALM;
            seen.stampede = h::herd_state(&pack) == h::HerdState::Stampede;
            seen.returning = h::herd_state(&pack) == h::HerdState::Returning;
        }
        for (i, c) in w.critters.iter().enumerate() {
            if c.kind == h::BULL && c.present() {
                seen.bull = Bull {
                    alive: c.alive(),
                    pos: c.pos,
                    facing: c.facing(),
                    middle: c.body(sp).middle(),
                    state: c.state,
                    act: c.act,
                    timer: c.timer,
                    right: c.role & (1 << 2) != 0,
                    stunned: h::stunned(c),
                };
            } else {
                seen.cows[i] = (c.alive() && c.kind == h::COW, c.pos, c.vel, c.state, c.act);
            }
        }
        self.memory[self.at] = seen;
        self.at = (self.at + 1) % self.memory.len();
        self.filled = (self.filled + 1).min(self.memory.len());
    }

    fn act(&mut self, w: &World) -> Input {
        self.cooldown = self.cooldown.saturating_sub(1);
        self.dodge_left = self.dodge_left.saturating_sub(1);
        let me = w.players[self.who];
        if me.health <= 0 || !matches!(w.phase, Phase::Fighting) {
            return Input::default();
        }
        let seen = self.recall();
        let bull = seen.bull;
        let sp = w.critters.sp();
        let half_len = stat_fx(sp, h::BULL, CritterField::Length).mul(Fx::ratio(1, 2));
        let half_wid = stat_fx(sp, h::BULL, CritterField::Width).mul(Fx::ratio(1, 2));
        let free = me.action.actionable();
        let poke = sim::moves::get(me.class, auto_slot(me.class));
        let reach = poke.reach.max(Fx::ONE);

        let look_at = |at: V3, middle: V3| {
            let to = at.sub(me.pos);
            let yaw = atan2_turns(to.z, to.x);
            let aim = turns_to_aim(yaw.sub(me.carry_yaw));
            let pitch = sim::aim::look_onto_closely(me.pos, aim, me.aloft, middle);
            (yaw, aim, pitch)
        };
        let (bull_yaw, bull_aim, bull_pitch) = look_at(bull.pos, bull.middle);
        let to_bull = flat(bull.pos.sub(me.pos));
        let gap = to_bull.flat_len();
        // Where it stands relative to the bull: along its facing, and across.
        let rel = flat(me.pos.sub(bull.pos));
        let along = rel.dot(bull.facing);
        let side = V3::new(bull.facing.z.neg(), Fx::ZERO, bull.facing.x);
        let across = rel.dot(side);
        let nose_cos = if rel.flat_len().raw() > 0 {
            bull.facing.dot(rel.normalized())
        } else {
            Fx::ONE
        };

        // **The crossing**: where to stand to roll the cart or stop it.
        let cart = h::rules::the_cart(w);
        let escort = cart.map(|o| self.escort(w, &seen, &bull, &o));

        // 7. **On the floor, nothing; up, walk.**
        let down = matches!(me.action, Action::Stagger { .. } | Action::HitStun { .. });
        if self.was_down && !down {
            self.walk_left = WALK_AFTER;
        }
        self.was_down = down;
        if down {
            self.intent = WALK;
            return Input::aimed(0, bull_aim);
        }

        // Not roused yet: walk at the herd until it is.
        if seen.calm && escort.is_none() {
            self.intent = ROUSE;
            let to = flat(bull.pos.sub(me.pos));
            return Input::aimed(steer(bull_yaw, to), bull_aim);
        }

        if self.walk_left > 0 {
            self.walk_left -= 1;
            self.intent = WALK;
            // Off the spot the trample is drawn on: across the bull's line,
            // not along it -- straight away from its middle walks a body lying
            // in front of it into the hoof's disc.
            let out = if across.raw() >= 0 {
                side
            } else {
                side.scale(Fx::ONE.neg())
            };
            let back = if along.raw() > 0 {
                bull.facing
            } else {
                bull.facing.scale(Fx::ONE.neg())
            };
            let away = out.add(out).add(back).normalized();
            return Input::aimed(steer(bull_yaw, away), bull_aim);
        }

        // **Plan v2, the ride home** (§9): aboard, brace through the first
        // buck and jump before the second -- or before the cow carries you
        // past the bull's nose.
        if let Some(_cow) = h::ride::rider_of(&me) {
            self.intent = RIDE;
            self.rode += 1;
            let first = h::knob(h::Knob::RidePatience).max(0) as u32;
            let second = first + h::knob(h::Knob::SecondBuck).max(0) as u32;
            let late = self.rode + LEAVE_BEFORE >= second;
            let hook_ahead = gap.raw() < HOOK_NEAR.add(Fx::from_int(2)).raw() && nose_cos.raw() > 0;
            if (late || hook_ahead) && free {
                return Input::aimed(Input::SPACE, bull_aim);
            }
            return Input::aimed(Input::CROUCH, bull_aim);
        }
        self.rode = 0;
        if self.rides && escort.is_none() && seen.returning && me.grounded && free {
            // A cow coming home past: stand in its way and jump as it
            // arrives, so it runs under the feet.
            let near = seen
                .cows
                .iter()
                .filter(|(alive, at, vel, state, _)| {
                    *alive
                        && *state == is::PROWL
                        && vel.flat_len().raw() > Fx::ONE.raw()
                        && flat(at.sub(me.pos)).flat_len().raw() < CATCH_FROM.raw()
                })
                .min_by_key(|(_, at, ..)| flat(at.sub(me.pos)).flat_len().raw())
                .copied();
            if let Some((_, at, vel, ..)) = near {
                let speed = vel.flat_len();
                let ahead = at.add(flat(vel).scale(CATCH_LEAD));
                let off =
                    sim::math::flat_segment_gap(me.pos, at, ahead.add(flat(vel).scale(CATCH_LEAD)));
                let arrives = flat(me.pos.sub(at)).flat_len().div(speed.max(Fx::ONE));
                self.intent = RIDE;
                if off.raw() < CATCH_OFF.raw()
                    && arrives.raw() < CATCH_JUMP.raw()
                    && arrives.raw() > CATCH_SOON.raw()
                {
                    self.jumping = JUMP_HOLD;
                    return Input::aimed(Input::SPACE, bull_aim);
                }
                return Input::aimed(steer(bull_yaw, flat(ahead.sub(me.pos))), bull_aim);
            }
        }
        if self.jumping > 0 {
            self.jumping -= 1;
            return Input::aimed(Input::SPACE, bull_aim);
        }

        // 4. **The bellow**: the lane drawn through it. To the nearest lee,
        // or out of the lane, whichever is nearer.
        let lane = seen.signs.iter().find(|s| {
            s.shape == Shape::Strip
                && matches!(s.says, Says::Coming | Says::Live)
                && s.width.raw() > Fx::from_int(6).raw()
        });
        if let Some(lane) = lane.filter(|l| l.covers(me.pos)) {
            let lees: Vec<_> = seen
                .signs
                .iter()
                .filter(|s| s.says == Says::Clear)
                .collect();
            let in_lee = lees.iter().any(|s| s.covers(me.pos));
            if !in_lee {
                self.intent = LEE;
                // Out of the lane, sideways.
                let sideways = V3::new(lane.along.z.neg(), Fx::ZERO, lane.along.x);
                let off = flat(me.pos.sub(lane.at)).dot(sideways);
                let half = lane.width.mul(Fx::ratio(1, 2)).add(Fx::ONE);
                let out_to = if off.raw() >= 0 {
                    half.sub(off)
                } else {
                    half.add(off)
                };
                let out_dir = if off.raw() >= 0 {
                    sideways
                } else {
                    sideways.scale(Fx::ONE.neg())
                };
                let mut best = (out_to, me.pos.add(out_dir.scale(out_to)));
                for lee in &lees {
                    let mid = lee.at.add(lee.along.scale(lee.length.mul(Fx::ratio(3, 4))));
                    let d = flat(mid.sub(me.pos)).flat_len();
                    if d.raw() < best.0.raw() {
                        best = (d, mid);
                    }
                }
                let to = flat(best.1.sub(me.pos));
                let live = lane.says == Says::Live;
                let dash =
                    if live && self.dodge_left == 0 && free && best.0.raw() > Fx::from_int(2).raw()
                    {
                        self.dodge_left = sim::tuning::dodge_frames() as u32 + 10;
                        Input::SHIFT
                    } else {
                        0
                    };
                return Input::aimed(steer(bull_yaw, to) | dash, bull_aim);
            }
            self.intent = LEE;
            return Input::aimed(0, bull_aim);
        }

        // 5. **A cow's rear wedge**, drawn faint under it: step out.
        let wedge = seen
            .signs
            .iter()
            .find(|s| s.says == Says::Faint && s.covers(me.pos));
        if let Some(wd) = wedge {
            self.intent = WEDGE;
            let sideways = V3::new(wd.along.z.neg(), Fx::ZERO, wd.along.x);
            let off = flat(me.pos.sub(wd.at)).dot(sideways);
            let out = if off.raw() >= 0 {
                sideways
            } else {
                sideways.scale(Fx::ONE.neg())
            };
            return Input::aimed(steer(bull_yaw, out), bull_aim);
        }

        if !bull.alive {
            self.intent = HOLD;
            return Input::aimed(0, bull_aim);
        }

        // The crossing: a lane is about to take the cart -- leave it, so it
        // stops, before anything else.
        if let (Some((at, true)), Some(o)) = (escort, cart) {
            let sp_h = sim::species::lookup(sim::species::SpeciesId::HORNBACK).unwrap();
            let near =
                sim::objective::stat_fx(sp_h, o.index, sim::objective::ObjectiveField::Escort);
            if flat(me.pos.sub(o.at)).flat_len().raw() <= near.add(Fx::ratio(1, 2)).raw() {
                self.intent = STOP;
                return Input::aimed(steer(bull_yaw, flat(at.sub(me.pos))), bull_aim);
            }
        }

        // 3. **The stun**: to the head, and hit it; leave with time to spare.
        if bull.stunned {
            let from = *self
                .stun_from
                .get_or_insert(w.frame.saturating_sub(REACTION as u32));
            self.stunned_at = Some(from);
            let stun = h::knob(h::Knob::StunFrames).max(0) as u32;
            // Its last swing ends with `SPARE` frames to go: it knows how long
            // its own swing is.
            let swing = (poke.startup + poke.active + poke.recovery) as u32;
            if w.frame + swing < from + stun.saturating_sub(SPARE) {
                self.intent = HEAD;
                // Where it was stood when the bull met the rock: the post it
                // will not use again while the bull remembers.
                if self.spent.is_none() {
                    self.spent = Some(self.posted.unwrap_or(me.pos));
                }
                let head = bull.pos.add(bull.facing.scale(half_len));
                let head_mid = V3::new(
                    head.x,
                    h::knob_fx(h::Knob::HeadLow).mul(Fx::ratio(1, 2)),
                    head.z,
                );
                let (yaw, aim, pitch) = look_at(head, head_mid);
                let d = flat(head.sub(me.pos)).flat_len();
                if d.raw() > reach.raw() {
                    // A person closes on the head, and dashes the last of it.
                    let dash = if d.raw() > Fx::from_int(6).raw() && self.dodge_left == 0 && free {
                        self.dodge_left = sim::tuning::dodge_frames() as u32 + 6;
                        Input::SHIFT
                    } else {
                        0
                    };
                    let ranged = reach.raw() > Fx::from_int(6).raw();
                    if !ranged {
                        return Input::looking_at(
                            steer(yaw, flat(head.sub(me.pos))) | dash,
                            aim,
                            pitch,
                        );
                    }
                }
                if self.cooldown == 0 && free {
                    self.cooldown = SWING_GAP + self.roll() % 4;
                    let button = if me.class == Class::DualMage {
                        auto_button(&me)
                    } else {
                        Input::LEFT
                    };
                    return Input::looking_at(button, aim, pitch);
                }
                return Input::looking_at(0, aim, pitch);
            }
        } else {
            self.stun_from = None;
        }

        // 2. **The charge**: hold through the paws; out of the lane on the
        // head-drop; dodge if it is on you.
        let lock = sim::critter::stat(sp, h::BULL, CritterField::Lock).max(0) as u16;
        let charging = bull.act == h::CHARGE && matches!(bull.state, is::STARTUP | is::ACTIVE);
        let in_lane = along.raw() > 0 && across.abs().raw() < half_wid.add(LANE_SLACK).raw();
        if !charging {
            self.raised_for = false;
        }
        // Escorting, and the cart behind it in the lane being wound up: walk
        // across the bull's line before it locks, so the lane misses the road
        // (§11).
        if charging && bull.state == is::STARTUP && bull.timer > lock {
            if let Some(o) = cart {
                let rel_cart = flat(o.at.sub(bull.pos));
                let ext = o.site.extent();
                let wide = half_wid.add(ext.x.max(ext.z)).add(LANE_SLACK);
                let behind = rel_cart.dot(bull.facing).raw() > 0
                    && rel_cart.dot(side).abs().raw() < wide.raw();
                if behind {
                    self.intent = BAIT;
                    // Across the line, and away from the cart: out of the
                    // escort's reach, so the cart stops where it is.
                    let away = if rel_cart.dot(side).raw() >= 0 {
                        side.scale(Fx::ONE.neg())
                    } else {
                        side
                    };
                    let off = flat(me.pos.sub(o.at));
                    let off = if off.flat_len().raw() > 0 {
                        off.normalized()
                    } else {
                        away
                    };
                    let way = away.add(away).add(off).normalized();
                    return Input::aimed(steer(bull_yaw, way), bull_aim);
                }
            }
        }
        if charging && in_lane {
            let dropped = bull.state == is::ACTIVE || bull.timer <= lock;
            // Which way out: away from the lane's line, or toward the side
            // with more room if it stands on it.
            let out = if across.raw() >= 0 {
                side
            } else {
                side.scale(Fx::ONE.neg())
            };
            if dropped {
                // The Elementalist raises a stone into a lane that ends in
                // nothing (§7): six metres in front of her, before she goes.
                let stops = seen.signs.iter().any(|s| s.says == Says::Stops);
                if me.class == Class::Elementalist && !stops && !self.raised_for && free {
                    self.raised_for = true;
                    self.intent = RAISE;
                    let spot = me
                        .pos
                        .add(flat(bull.pos.sub(me.pos)).normalized().scale(STONE_AHEAD));
                    let (_, aim, _) = look_at(spot, spot);
                    let pitch = sim::aim::look_onto_closely(me.pos, aim, me.aloft, spot);
                    return Input::looking_at(Input::MECHANIC, aim, pitch);
                }
                self.intent = STEP_OUT;
                // Will a walk get it out in time? The run comes at the
                // charge's speed from the nose; the walk has to clear the
                // lane's half-width. If not, the dodge -- the escape.
                let nose = bull.pos.add(bull.facing.scale(half_len));
                let run = flat(nose.sub(me.pos)).flat_len();
                let speed = sp.attack(h::CHARGE).advance.max(Fx::ONE);
                let lead = Fx::from_int(bull.timer.saturating_sub(REACTION as u16) as i32);
                let arrives = run.div(speed).mul(Fx::from_int(60)).add(lead);
                let clear = half_wid.add(LANE_SLACK).sub(across.abs()).max(Fx::ZERO);
                let walks = clear.div(sim::tuning::move_speed()).mul(Fx::from_int(60));
                let close = run.raw() < DODGE_AT.raw() || arrives.raw() < walks.raw();
                let dash = if close && self.dodge_left == 0 && free {
                    self.dodge_left = sim::tuning::dodge_frames() as u32 + 10;
                    Input::SHIFT
                } else {
                    0
                };
                // Out, and back toward the rock behind: where the bull's head
                // will be when the rock stops it.
                let stops = seen.signs.iter().any(|s| s.says == Says::Stops);
                let way = if stops && dash == 0 {
                    out.add(out).add(bull.facing).normalized()
                } else {
                    out
                };
                return Input::aimed(steer(bull_yaw, way.scale(OUT_OF_LANE)) | dash, bull_aim);
            }
            // Before the drop: stand, facing it -- leaving early is what the
            // charge teaches against -- unless the lane drawn on the floor
            // ends in nothing. Then the paws are the time to put the rock
            // behind you: it tracks you, and that is the point.
            let stops = seen.signs.iter().any(|s| s.says == Says::Stops);
            if !stops {
                if let Some(post) = self.post(w, &me, &bull, side) {
                    let to = flat(post.sub(me.pos));
                    if to.flat_len().raw() > AT_POST.raw() {
                        self.intent = POST;
                        return Input::aimed(steer(bull_yaw, to), bull_aim);
                    }
                }
            }
            self.intent = HOLD;
            return Input::aimed(0, bull_aim);
        }

        // Out of a running charge's lane: stay out of it until it is over.
        if charging && (bull.state == is::ACTIVE || bull.timer <= lock) && along.raw() > 0 {
            self.intent = STEP_OUT;
            return Input::aimed(0, bull_aim);
        }

        // 6. **The close game.**
        if gap.raw() < Fx::from_int(7).raw() {
            // The hook winding up in front: dodge toward the flank its head is
            // not cocked to.
            let hooking = bull.act == h::HOOK && bull.state == is::STARTUP;
            if hooking
                && gap.raw() < HOOK_NEAR.raw()
                && nose_cos.raw() > 0
                && free
                && self.dodge_left == 0
            {
                self.intent = DODGE;
                self.dodge_left = sim::tuning::dodge_frames() as u32 + 8;
                // Close in, through it toward the flank its head is not
                // cocked to; at the edge of its reach, out sideways on the
                // side it already stands, which leaves the volume soonest.
                let way = if gap.raw() < HOOK_THROUGH.raw() {
                    let away = if bull.right {
                        side.scale(Fx::ONE.neg())
                    } else {
                        side
                    };
                    away.add(away)
                        .add(bull.facing.scale(Fx::ONE.neg()))
                        .normalized()
                } else {
                    let out = if across.raw() >= 0 {
                        side
                    } else {
                        side.scale(Fx::ONE.neg())
                    };
                    out.add(out).add(bull.facing).normalized()
                };
                return Input::aimed(steer(bull_yaw, way) | Input::SHIFT, bull_aim);
            }
            // The shoulder's lean, at its flank: a swing already in flight
            // meets it; failing one, the dodge, away from the flank it will
            // throw.
            let leaning = bull.act == h::SHOULDER && bull.state == is::STARTUP;
            let flank_reach = h::knob_fx(h::Knob::ShoulderFlank).add(Fx::ONE);
            if leaning && gap.raw() < flank_reach.raw() && free && self.dodge_left == 0 {
                self.intent = DODGE;
                self.dodge_left = sim::tuning::dodge_frames() as u32 + 8;
                let out = if across.raw() >= 0 {
                    side
                } else {
                    side.scale(Fx::ONE.neg())
                };
                return Input::aimed(steer(bull_yaw, out) | Input::SHIFT, bull_aim);
            }
            let body_reach = reach.add(half_wid);
            let close = gap.raw() <= body_reach.add(Fx::ONE).raw();
            // Open, and staying open long enough: the recovery's phase is
            // visible, and a person who has seen it twice knows roughly where
            // its end is.
            let open = matches!(bull.state, is::RECOVERY | is::FLINCH)
                && bull.timer as usize
                    > REACTION + (poke.startup + poke.active + poke.recovery) as usize;
            // Near a bull that can act, the close game is its: back to a
            // post, round its front rather than through it. Near one that
            // cannot, punish.
            if !open && !(close && !free) {
                self.intent = ROUND;
                let post = match escort {
                    Some((at, _)) => Some(at),
                    None => self.post(w, &me, &bull, side),
                };
                if let Some(post) = post {
                    let to = flat(post.sub(me.pos));
                    // Out of its reach first, then to the post: from in front
                    // of it, sideways off the line its hook and its charge
                    // take; from beside it, straight away from the flank.
                    let out = if across.raw() >= 0 {
                        side
                    } else {
                        side.scale(Fx::ONE.neg())
                    };
                    let front = nose_cos.raw() > Fx::ratio(3, 10).raw();
                    let way = if gap.raw() >= HOOK_NEAR.raw() && !front {
                        to
                    } else if front {
                        out.add(out).add(bull.facing).normalized()
                    } else if rel.flat_len().raw() > 0 {
                        rel.normalized()
                    } else {
                        out
                    };
                    return Input::aimed(steer(bull_yaw, way), bull_aim);
                }
            }
            // In reach: heavy from the flank, poke into an opening -- a
            // rhythm, so a shoulder's lean meets a swing already in flight.
            if close && !free {
                self.intent = POKE;
                return Input::looking_at(0, bull_aim, bull_pitch);
            }
            if close && self.cooldown == 0 && free {
                self.intent = POKE;
                self.cooldown = SWING_GAP + self.roll() % 4;
                // Pokes into an opening: a heavy's commitment outlasts a
                // skid's, and what follows a skid is a shoulder or a hook.
                let button = if me.class == Class::DualMage {
                    auto_button(&me)
                } else {
                    Input::LEFT
                };
                return Input::looking_at(button, bull_aim, bull_pitch);
            }
        }

        // A ranged class pokes from where it stands at a bull in its reach.
        let ranged = reach.raw() > Fx::from_int(6).raw();
        if ranged && gap.raw() <= reach.raw() && self.cooldown == 0 && free {
            self.intent = POKE;
            self.cooldown = SWING_GAP + 4 + self.roll() % 6;
            let button = if me.class == Class::DualMage {
                auto_button(&me)
            } else {
                Input::LEFT
            };
            return Input::looking_at(button, bull_aim, bull_pitch);
        }

        // The crossing: beside the cart, or clear of it to stop it.
        if let Some((at, stopping)) = escort {
            let to = flat(at.sub(me.pos));
            self.intent = if stopping { STOP } else { ESCORT };
            if to.flat_len().raw() > AT_POST.mul(Fx::ratio(1, 2)).raw() {
                return Input::aimed(steer(bull_yaw, to), bull_aim);
            }
            return Input::aimed(0, bull_aim);
        }

        // 1. **A rock at your back**, on the line from the bull through you.
        let post = self.post(w, &me, &bull, side);
        if let Some(post) = post {
            let to = flat(post.sub(me.pos));
            self.posted = Some(post);
            if to.flat_len().raw() > AT_POST.raw() {
                self.intent = POST;
                return Input::aimed(steer(bull_yaw, to), bull_aim);
            }
        }
        self.intent = HOLD;
        let _ = (self.rides, MAX_PLAYERS);
        Input::aimed(0, bull_aim)
    }

    fn intent(&self) -> Intent {
        self.intent
    }
}

impl Hornback {
    /// **Where to stand on the crossing** (§11), and whether that is to stop
    /// the cart: beside it toward the bull while it may roll, past its escort
    /// distance while a lane drawn on the road is within `STOP_SHORT` of its
    /// front -- a stampede's or a wave's, as drawn.
    fn escort(
        &self,
        w: &World,
        seen: &Seen,
        bull: &Bull,
        cart: &sim::objective::Standing,
    ) -> (V3, bool) {
        let sp = sim::species::lookup(sim::species::SpeciesId::HORNBACK).unwrap();
        let near = sim::objective::stat_fx(sp, cart.index, sim::objective::ObjectiveField::Escort);
        let ext = cart.site.extent();
        let road = cart.dir;
        let across = V3::new(road.z.neg(), Fx::ZERO, road.x);
        let ahead = cart.at.add(road.scale(ext.x.add(STOP_SHORT)));
        let lanes = seen.signs.iter().filter(|s| {
            s.shape == Shape::Strip
                && matches!(s.says, Says::Coming | Says::Live)
                && s.width.raw() > Fx::from_int(6).raw()
        });
        let mut stop = false;
        for lane in lanes {
            for k in [-1, 0, 1] {
                let p = ahead.add(across.scale(ext.z.mul(Fx::from_int(k))));
                stop |= lane.covers(p)
                    || lane.covers(cart.at.add(across.scale(ext.z.mul(Fx::from_int(k)))));
            }
        }
        // Toward the bull and the herd -- nearer both than the cart is, so
        // the bull's threat is a hunter -- or the herd's side if it is gone.
        let mut herd = V3::ZERO;
        let mut n = 0;
        for (alive, at, ..) in seen.cows.iter().filter(|c| c.0) {
            let _ = alive;
            herd = herd.add(flat(*at));
            n += 1;
        }
        let mut to = if bull.alive {
            flat(bull.pos.sub(cart.at)).normalized()
        } else {
            V3::ZERO
        };
        if n > 0 {
            let middle = herd.scale(Fx::ONE.div(Fx::from_int(n)));
            to = to.add(flat(middle.sub(cart.at)).normalized());
        }
        let toward = if to.flat_len().raw() > 0 {
            to.normalized()
        } else {
            across
        };
        let _ = w;
        if stop {
            let back = road.scale(Fx::from_int(2).neg());
            (
                cart.at.add(toward.scale(near.add(PAST_ESCORT))).add(back),
                true,
            )
        } else {
            // Off the road on the bull's side, and ahead: never on the line
            // from the bull through the cart, so a charge at the hunter runs
            // past the cart's front rather than into it.
            let out = if toward.dot(across).raw() >= 0 {
                across
            } else {
                across.scale(Fx::ONE.neg())
            };
            (
                cart.at
                    .add(out.scale(BESIDE_CART))
                    .add(road.scale(CART_LEAD)),
                false,
            )
        }
    }

    /// **Where to stand** (§9 step 1): in front of a rock, `OFF_ROCK` off
    /// it, on the line from the bull through you -- the nearest such place,
    /// but not one beside the bull nor one whose way there passes it, and
    /// not the rock the bull is wary of. The hunter knows the rule and kept
    /// its own count since the stun.
    fn post(&mut self, w: &World, me: &sim::state::Player, bull: &Bull, side: V3) -> Option<V3> {
        let memory =
            h::knob(h::Knob::WaryFrames).max(0) as u32 + h::knob(h::Knob::StunFrames).max(0) as u32;
        if self.stunned_at.is_some_and(|at| w.frame > at + memory) {
            self.spent = None;
        }
        let wary = self.spent;
        rocks(w)
            .into_iter()
            .map(|r| (r.post(bull.pos, side), r.cost()))
            .filter(|(p, _)| {
                wary.is_none_or(|s| flat(s.sub(*p)).flat_len().raw() > SPENT_NEAR.raw())
            })
            .min_by_key(|(p, cost)| {
                let walk = flat(p.sub(me.pos)).flat_len();
                let near = flat(p.sub(bull.pos)).flat_len();
                let past = sim::math::flat_segment_gap(bull.pos, me.pos, *p);
                let crowd = KEEP_OFF
                    .sub(near)
                    .max(Fx::ZERO)
                    .add(KEEP_OFF.sub(past).max(Fx::ZERO));
                walk.add(*cost).add(crowd.mul(Fx::from_int(3))).raw()
            })
            .map(|(p, _)| p)
    }
}

/// **The Hornback's own report lines** (§9).
#[derive(Default)]
pub struct Lines {
    charges: u32,
    into_solid: u32,
    dodged: u32,
    walked: u32,
    charges_landed: u32,
    stun_frames: u32,
    stun_damage: i32,
    horns: u32,
    cracked: u32,
    shattered: u32,
    driven_off: u32,
    bellows: u32,
    touched: u32,
    in_lee: u32,
    aloft: u32,
    riding: u32,
    braced: u32,
    bounced: u32,
    broken: u32,
    kicks: u32,
    kicks_landed: u32,
    rides: u32,
    ride_frames: u32,
    bucked: u32,
    jumped: u32,
    hooked_off: u32,
    charge_dodging: bool,
    charge_hit: bool,
    lee_checked: bool,
    /// The crossing (§11): waves run, the cart's damage by what did it, and
    /// frames it stood stopped with a lane drawn ahead.
    waves: u32,
    cart_by_charge: i32,
    cart_by_herd: i32,
    held_short: u32,
    crossing: bool,
}

impl crate::report::Tally for Lines {
    fn observe(&mut self, before: &World, after: &World) {
        let (Some(was), Some(now)) = (before.pack, after.pack) else {
            return;
        };
        // The crossing: what struck the cart this frame, and the waves.
        let taken = |w: &World| {
            sim::objective::standing(&w.lore, w.arena())
                .next()
                .map(|o| (o.state.taken, o.state.along))
        };
        if let (Some((t0, a0)), Some((t1, a1))) = (taken(before), taken(after)) {
            self.crossing = true;
            if t1 > t0 {
                let by_bull = after.critters.iter().any(|c| {
                    c.kind == h::BULL && c.act == h::CHARGE && (c.attacking() || h::stunned(c))
                });
                if by_bull {
                    self.cart_by_charge += t1 - t0;
                } else {
                    self.cart_by_herd += t1 - t0;
                }
            }
            if a1 == a0 && h::rules::wave_lane(after).is_some() {
                self.held_short += 1;
            }
            if h::rules::wave_lane(before).is_some() && h::rules::wave_lane(after).is_none() {
                self.waves += 1;
            }
        }
        let sp = after.critters.sp();
        for (b, a) in before.critters.iter().zip(after.critters.iter()) {
            let began = a.state == is::STARTUP && b.state != is::STARTUP;
            let landed = a.state == is::ACTIVE
                && a.has(sim::critter::flag::HIT_USED)
                && !(b.state == is::ACTIVE && b.has(sim::critter::flag::HIT_USED));
            if a.kind == h::BULL {
                if began && a.act == h::CHARGE {
                    self.charges += 1;
                    self.charge_dodging = false;
                    self.charge_hit = false;
                }
                if a.act == h::CHARGE && a.state == is::ACTIVE {
                    if after
                        .players
                        .iter()
                        .any(|p| matches!(p.action, Action::Dodge { .. }) && p.health > 0)
                    {
                        let rel = flat(after.players[0].pos.sub(a.pos));
                        let side = V3::new(a.facing().z.neg(), Fx::ZERO, a.facing().x);
                        if rel.dot(side).abs().raw() < Fx::from_int(2).raw() {
                            self.charge_dodging = true;
                        }
                    }
                }
                if landed && a.act == h::CHARGE {
                    self.charges_landed += 1;
                    self.charge_hit = true;
                }
                if h::stunned(a) && !h::stunned(b) {
                    self.into_solid += 1;
                }
                // The end of a charge's run that met nothing.
                if b.act == h::CHARGE && b.state == is::ACTIVE && a.state == is::RECOVERY {
                    if self.charge_dodging && !self.charge_hit {
                        self.dodged += 1;
                    } else if !self.charge_hit {
                        self.walked += 1;
                    }
                }
                if h::stunned(a) {
                    self.stun_frames += 1;
                    self.stun_damage += (b.health as i32 - a.health as i32).max(0);
                }
                if began && a.act == h::BELLOW {
                    self.bellows += 1;
                    self.lee_checked = false;
                }
                if began && a.act == h::GUARD {
                    self.braced += 1;
                }
                if b.act == h::GUARD && b.state == is::ACTIVE && a.state == is::PROWL {
                    self.bounced += 1;
                }
                if b.state != is::FLINCH && a.state == is::FLINCH && a.role & (1 << 1) != 0 {
                    self.broken += 1;
                }
            } else {
                if began && a.act == h::KICK {
                    self.kicks += 1;
                }
                if landed && a.act == h::KICK {
                    self.kicks_landed += 1;
                }
                if landed && a.act == h::STAMPEDE {
                    self.touched += 1;
                }
                if b.alive() && b.role & (1 << 7) == 0 && a.role & (1 << 7) != 0 {
                    self.driven_off += 1;
                }
            }
        }
        // The hunter when the herd breaks into its run: in a lee, aloft,
        // riding -- or in the way.
        let running = |w: &World| {
            w.critters
                .iter()
                .any(|c| c.kind == h::COW && c.act == h::STAMPEDE && c.state == is::ACTIVE)
        };
        if running(after) && !running(before) && !self.lee_checked {
            self.lee_checked = true;
            let signs = after.signs();
            for p in after.players.iter().filter(|p| p.health > 0) {
                if p.aboard() {
                    self.riding += 1;
                } else if !p.grounded || p.pos.y.raw() > Fx::ONE.raw() {
                    self.aloft += 1;
                } else if signs
                    .iter()
                    .any(|s| s.says == Says::Clear && s.covers(p.pos))
                {
                    self.in_lee += 1;
                }
            }
        }
        let horns = |p: &sim::pack::Pack| h::horns_whole(p).iter().filter(|w| !**w).count() as u32;
        self.horns += horns(&now).saturating_sub(horns(&was));
        for (b, a) in sim::hazard::all(&before.lore).zip(sim::hazard::all(&after.lore)) {
            if b.1.index() == h::BOULDER && a.1.index() == h::CRACKED {
                self.cracked += 1;
            }
            if b.1.index() != h::RUBBLE && a.1.index() == h::RUBBLE && b.1.present() {
                self.shattered += 1;
            }
        }
        // Rides on a cow.
        for (pb, pa) in before.players.iter().zip(after.players.iter()) {
            let on = |p: &sim::state::Player| h::ride::rider_of(p).is_some();
            if on(pa) {
                self.ride_frames += 1;
            }
            if !on(pb) && on(pa) {
                self.rides += 1;
            }
            if on(pb) && !on(pa) {
                if matches!(pa.action, Action::HitStun { .. } | Action::Stagger { .. }) {
                    if after
                        .critters
                        .iter()
                        .any(|c| c.kind == h::BULL && c.act == h::HOOK && c.state == is::ACTIVE)
                    {
                        self.hooked_off += 1;
                    } else {
                        self.bucked += 1;
                    }
                } else {
                    self.jumped += 1;
                }
            }
        }
        let _ = sp;
    }

    fn lines(&self) -> Vec<(String, String, String)> {
        let row = |name: &str, value: String, why: &str| (name.to_string(), value, why.to_string());
        let mut out = vec![
            row(
                "charges",
                format!(
                    "{} thrown / {} into a solid / {} dodged through / {} walked out of / {} landed",
                    self.charges, self.into_solid, self.dodged, self.walked, self.charges_landed
                ),
                "at least one in three into a solid, or the idea is not landing",
            ),
            row(
                "stun frames, damage in them",
                format!("{} / {}", self.stun_frames, self.stun_damage),
                "whether the window is used",
            ),
            row(
                "horns broken, boulders cracked / shattered, cows driven off",
                format!(
                    "{} , {} / {} , {}",
                    self.horns, self.cracked, self.shattered, self.driven_off
                ),
                "what the fight spent",
            ),
            row(
                "bellows",
                format!(
                    "{} thrown / {} cows touched the hunter / {} in a lee, {} aloft, {} riding",
                    self.bellows, self.touched, self.in_lee, self.aloft, self.riding
                ),
                "the stampede's answers",
            ),
            row(
                "guard",
                format!(
                    "{} braced / {} bounced / {} broken",
                    self.braced, self.bounced, self.broken
                ),
                "the front lesson",
            ),
            row(
                "kicks",
                format!("{} thrown / {} landed", self.kicks, self.kicks_landed),
                "the rear of a cow",
            ),
            row(
                "rides",
                format!(
                    "{} / mean {:.1}s / {} bucked, {} jumped / {} ended in a hook",
                    self.rides,
                    if self.rides > 0 {
                        self.ride_frames as f32 / self.rides as f32 / 60.0
                    } else {
                        0.0
                    },
                    self.bucked,
                    self.jumped,
                    self.hooked_off
                ),
                "plan v2: a ride home",
            ),
        ];
        if self.crossing {
            out.push(row(
                "crossing",
                format!(
                    "{} waves / cart took {} from charges, {} from the herd / {:.1}s held short of a drawn lane",
                    self.waves,
                    self.cart_by_charge,
                    self.cart_by_herd,
                    self.held_short as f32 / 60.0
                ),
                "stop it short of every lane; bait charges off the road",
            ));
        }
        out
    }
}

pub static CARD: crate::plans::Card = crate::plans::Card {
    species: sim::species::SpeciesId::HORNBACK,
    plan: |who, seed, _hop| Box::new(Hornback::new(who, seed)),
    bucks: |kind| kind == h::BUCK,
    words: crate::plans::Words {
        weak_hits: "head hits",
        broken: "horns broken",
        into_breakables: "damage into the horns",
        into_breakables_why: "only the head counts",
        worst: "worst horn",
        ride_for: "a cow's ride home",
        toppled_pool: "off a pool under a stunned bull",
    },
    tally: Some(|| Box::new(Lines::default())),
};
