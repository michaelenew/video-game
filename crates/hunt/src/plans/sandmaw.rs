//! The Sandmaw's hunter: the plan a decent player follows, and the report
//! lines the fight is measured by.
//!
//! **Stand still until the wake is coming; make one deliberate noise and walk
//! off it; when it stands, be behind it at the low spiracles and crouch under
//! the tail; sidestep a lane; jump out of a sinkhole; climb a beached worm and
//! work its back until the vents clamp; behind a boulder when it spits at an
//! island; break out of a swallow on the first gulp.**
//! `docs/design/creatures/sandmaw.md` §9, in order.
//!
//! Like every plan it sees the world `REACTION` frames late and never reads
//! the creature's mind: it reads the body (its wake and fin while it is under,
//! its column when it is up), the floor (the sinkhole), and the markers its
//! moves draw -- the rise's circle, the breach's lane, the spray's cone, the
//! tail's half-ring -- which a person sees too. What it heard is drawn in the
//! sand, but this hunter does not need it: it knows what it did.

use sim::fixed::Fx;
use sim::hazard::Floor;
use sim::math::{atan2_turns, wide_flat_dist};
use sim::monster::{Doing, Monster};
use sim::species::sandmaw::{self, Knob, fight};
use sim::state::{Action, Phase};
use sim::{Input, V3, World};

use crate::report::{Tally, Threat};
use crate::{Intent, Plan, REACTION, heavy, steer, turns_to_aim};

/// Quiet: standing still, waiting for the wake.
pub const QUIET: Intent = Intent("Quiet");
/// Making the noise it wants the worm to come up at.
pub const BAIT: Intent = Intent("Bait");
/// Walking off the noise it made.
pub const OFF: Intent = Intent("Off");
/// Out of a marker, a lane, a cone or a ring.
pub const EVADE: Intent = Intent("Evade");
/// Out of a sinkhole: a jump.
pub const JUMP: Intent = Intent("Jump");
/// Under the tail.
pub const DUCK: Intent = Intent("Duck");
/// To its back, and hitting the low spiracles.
pub const PUNISH: Intent = Intent("Punish");
/// Raising a stone into the circle.
pub const STONE: Intent = Intent("Stone");
/// Onto the beached back.
pub const CLIMB: Intent = Intent("Climb");
/// Aboard: hitting the vents.
pub const WORK: Intent = Intent("Work");
/// Aboard: off before it dives.
pub const LEAVE: Intent = Intent("Leave");
/// In its throat: the gulp.
pub const GULP: Intent = Intent("Gulp");
/// Into the open mouth.
pub const GAG: Intent = Intent("Gag");
/// Behind a boulder from the spit.
pub const COVER: Intent = Intent("Cover");

/// How many frames early or late its timed presses can be.
const SLOP_EARLY: i32 = 3;
const SLOP_LATE: i32 = 3;
/// Frames of jump held: a full hop.
const LEAP_HOLD: u16 = 32;
/// How near the wake has to come before it baits: inside the landing's
/// hearing, outside the feel.
const BAIT_NEAR: Fx = Fx::ratio(12, 1);
/// Frames of quiet before a bait.
const QUIET_FIRST: u32 = 40;
/// Bait anyway after this long quiet: the search is coming.
const BAIT_AFTER: u32 = 420;
/// How far off its noise it walks.
const OFF_BY: Fx = Fx::ratio(32, 10);
/// How far past a marker's edge counts as out of it.
const MARGIN: Fx = Fx::ratio(6, 10);
/// Where it stands behind the column: this far from the hole.
const BEHIND: Fx = Fx::ratio(20, 10);
/// Where it stands in front of it, at the throat.
const IN_FRONT: Fx = Fx::ratio(30, 10);
/// In front, as the stand begins, is a bearing within this of its facing.
const FRONT_COS: Fx = Fx::ratio(3, 10);
/// How far from its hole to go once it dives: out of its feel.
const CLEAR_OF: Fx = Fx::ratio(85, 10);
/// Do not bother correcting for less than this.
const SETTLED: Fx = Fx::ratio(5, 10);
/// Frames between swings.
const SWING_GAP: u16 = 20;
/// Frames of an opening kept back to get out in.
const EXIT: i32 = 8;
/// Frames between the Elementalist's stones.
const STONE_GAP: u16 = 120;
/// The dodge goes this many frames before the hit.
const DODGE_LEAD: i32 = 4;
/// Frames after a dodge before another.
const DODGE_REST: u16 = 20;
/// How long to wait for the bait to be answered before baiting again.
const BAIT_WAIT: u16 = 150;

/// What the hunter can see, one frame of it.
#[derive(Clone, Copy)]
struct Seen {
    beast: Option<Monster>,
    floor: Floor,
}

impl Default for Seen {
    fn default() -> Seen {
        Seen {
            beast: None,
            floor: Floor::NONE,
        }
    }
}

pub struct Sandmaw {
    who: usize,
    memory: Vec<Seen>,
    at: usize,
    filled: usize,
    intent: Intent,
    cooldown: u16,
    leap_left: u16,
    dodge_left: u16,
    /// Which way a leap in progress goes.
    leap_dir: V3,
    stone_left: u16,
    /// Where it made its noise, and which way it is walking off it.
    bait_at: Option<V3>,
    off_dir: V3,
    /// Frames since it last baited.
    since_bait: u32,
    /// Frames it has been quiet.
    quiet_for: u32,
    /// Which side of the column it plays this stand: the front, or not.
    side: Option<bool>,
    /// Where it last went under, to keep away from while it is quiet.
    dived_at: Option<V3>,
    /// Frames it has been inside, and whether it has pressed this swallow.
    inside_for: u32,
    pressed: bool,
    slop: i32,
    rng: u32,
    hop: Fx,
}

impl Sandmaw {
    pub fn new(who: usize, seed: u32, hop: Fx) -> Sandmaw {
        let mut s = Sandmaw {
            who,
            memory: vec![Seen::default(); REACTION + 1],
            at: 0,
            filled: 0,
            intent: QUIET,
            cooldown: 0,
            leap_left: 0,
            dodge_left: 0,
            leap_dir: V3::ZERO,
            stone_left: 0,
            bait_at: None,
            off_dir: V3::ZERO,
            since_bait: 0,
            quiet_for: 0,
            side: None,
            dived_at: None,
            inside_for: 0,
            pressed: false,
            slop: 0,
            rng: (0x9E37_79B9
                ^ (who as u32).wrapping_mul(0x85EB_CA6B)
                ^ seed.wrapping_mul(0x27D4_EB2F))
                | 1,
            hop,
        };
        s.roll_slop();
        s
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

fn away_from(from: V3, me: V3, fallback: V3) -> V3 {
    let d = flat(me.sub(from));
    if d.flat_len().raw() > 0 {
        d.normalized()
    } else {
        fallback
    }
}

/// Buttons, with the crosshair put on a point.
fn looking(me: &sim::state::Player, at: V3, bits: u16) -> Input {
    let d = flat(at.sub(me.pos));
    let yaw = atan2_turns(d.z, d.x);
    let wire = turns_to_aim(yaw.sub(me.carry_yaw));
    let pitch = sim::aim::look_onto(me.pos, wire, me.aloft, at);
    Input::looking_at(bits, wire, pitch)
}

/// Walking toward a direction, looking at a point.
fn walking(me: &sim::state::Player, dir: V3, look_at: V3, bits: u16) -> Input {
    let d = flat(look_at.sub(me.pos));
    let yaw = atan2_turns(d.z, d.x);
    let wire = turns_to_aim(yaw.sub(me.carry_yaw));
    Input::aimed(steer(yaw, dir) | bits, wire)
}

/// The middle of a part, in the world.
fn part_at(beast: &Monster, part: usize) -> V3 {
    let sh = sandmaw::SPECIES.shape(part);
    let mid = sh.min.add(sh.max).scale(crate::HALF);
    beast.world_of(part, mid)
}

/// Is it standing out of the sand: the column, or a move from it?
fn up(beast: &Monster) -> bool {
    fight::standing(beast)
}

impl Sandmaw {
    fn watch(&mut self, w: &World) {
        let seen = Seen {
            beast: w.monster().copied(),
            floor: w.terrain().floor,
        };
        self.memory[self.at] = seen;
        self.at = (self.at + 1) % self.memory.len();
        self.filled = (self.filled + 1).min(self.memory.len());
    }

    fn act(&mut self, w: &World) -> Input {
        self.cooldown = self.cooldown.saturating_sub(1);
        self.stone_left = self.stone_left.saturating_sub(1);
        self.dodge_left = self.dodge_left.saturating_sub(1);
        self.since_bait = self.since_bait.saturating_add(1);
        let me = w.players[self.who];
        if !me.grounded || me.aboard() || me.action.actionable() {
            self.leap_left = self.leap_left.saturating_sub(1);
        }
        if me.health <= 0 || !matches!(w.phase, Phase::Fighting) {
            return Input::default();
        }
        // **In its throat**: one press, on the first gulp, with the slop
        // every timing has. Counted from the moment it was taken, because
        // the gulp is a rhythm: a person times it, rather than reacting to a
        // ten-frame window.
        if sim::state::inside(&me, &w.monsters) {
            self.intent = GULP;
            self.inside_for += 1;
            let at = Knob::GulpEvery.raw() + 2 + self.slop;
            if !self.pressed && self.inside_for as i32 >= at {
                self.pressed = true;
                return Input::new(Input::SPECIAL);
            }
            return Input::default();
        }
        if self.inside_for > 0 {
            self.inside_for = 0;
            self.pressed = false;
            self.roll_slop();
        }
        let seen = self.recall();
        let Some(beast) = seen.beast else {
            return Input::default();
        };
        if !beast.alive() {
            return Input::default();
        }
        if me.aboard() {
            return self.ride(&me, &beast);
        }
        // **Strayed out over the rim**: back in, over it.
        let b = w.arena().bounds;
        let outside = me.pos.x.raw() < b.lo_x.raw()
            || me.pos.x.raw() > b.hi_x.raw()
            || me.pos.z.raw() < b.lo_z.raw()
            || me.pos.z.raw() > b.hi_z.raw();
        if outside {
            self.intent = OFF;
            let home = flat(V3::ZERO.sub(me.pos)).normalized();
            if me.grounded && self.leap_left == 0 {
                self.leap_left = LEAP_HOLD;
                self.leap_dir = home;
            }
            return walking(&me, home, beast.pos, Input::SPACE);
        }
        if self.leap_left > 0 {
            let dir = keep_in(w, me.pos, self.leap_dir);
            return walking(&me, dir, beast.pos, Input::SPACE);
        }
        if let Some(input) = self.answer(w, &me, &beast, &seen) {
            return input;
        }
        if up(&beast) || matches!(beast.doing, Doing::Toppled { .. }) {
            self.dived_at = None;
            return self.punish(w, &me, &beast);
        }
        if self.side.take().is_some() {
            // It has just gone under: away from the hole, out of its feel,
            // before going quiet again.
            self.dived_at = Some(flat(beast.pos));
        }
        if let Some(hole) = self.dived_at {
            if wide_flat_dist(hole, me.pos).raw() < CLEAR_OF.raw() {
                self.intent = OFF;
                let out = away_from(hole, me.pos, flat(me.facing));
                return walking(&me, keep_in(w, me.pos, out), beast.pos, 0);
            }
            self.dived_at = None;
        }
        self.wait(w, &me, &beast)
    }

    /// **The answer to what is coming**, if anything is coming here.
    fn answer(
        &mut self,
        w: &World,
        me: &sim::state::Player,
        beast: &Monster,
        seen: &Seen,
    ) -> Option<Input> {
        let _ = w;
        // A sinkhole under its feet: out, by the air.
        let pit = seen.floor.iter().find(|h| {
            h.kind == fight::SINKHOLE && wide_flat_dist(h.a, me.pos).raw() < h.radius.raw()
        });
        let undertow_marked = matches!(
            beast.doing,
            Doing::Startup {
                kind: sandmaw::UNDERTOW,
                ..
            }
        ) && wide_flat_dist(beast.aimed_at(), me.pos).raw()
            < Knob::RiseBroken
                .fx()
                .max(sim::hazard::stat_fx(
                    &sandmaw::SPECIES,
                    fight::SINKHOLE,
                    sim::hazard::HazardField::Radius,
                ))
                .add(MARGIN)
                .raw();
        // The ring sinking: walk out of it while the sand still holds.
        if undertow_marked && pit.is_none() {
            self.intent = EVADE;
            let out = away_from(beast.aimed_at(), me.pos, flat(me.facing));
            let out = keep_in(w, me.pos, out);
            return Some(walking(me, out, me.pos.add(out), 0));
        }
        // In it: walking out, and off the sand into the air at once.
        if let Some(centre) = pit.map(|h| h.a) {
            self.intent = JUMP;
            let out = away_from(centre, me.pos, flat(me.facing));
            let out = keep_in(w, me.pos, out);
            if me.grounded && self.leap_left == 0 {
                self.leap_left = LEAP_HOLD;
                self.leap_dir = out;
            }
            return Some(walking(me, out, me.pos.add(out), Input::SPACE));
        }
        let (kind, left, live) = match beast.doing {
            Doing::Startup { kind, left } => (kind, left, false),
            Doing::Active { kind, left } => (kind, left, true),
            _ => return None,
        };
        let a = beast.sp().attack(kind);
        if !live && left == a.startup.saturating_sub(1) {
            self.roll_slop();
        }
        let body = sim::tuning::body_radius();
        match kind {
            // The circle: off it, walking -- it does not follow. The
            // Elementalist puts a stone in it on her way.
            sandmaw::RISE => {
                let t = beast.telegraph()?;
                let d = wide_flat_dist(t.anchor, me.pos);
                if me.class == sim::Class::Elementalist
                    && !live
                    && self.stone_left == 0
                    && left as i32 > 8
                    && d.raw() < Fx::from_int(12).raw()
                {
                    self.stone_left = STONE_GAP;
                    self.intent = STONE;
                    return Some(looking(me, flat(t.anchor), Input::MECHANIC));
                }
                if d.raw() <= t.radius.add(body).add(MARGIN).raw() {
                    self.intent = EVADE;
                    let out = away_from(t.anchor, me.pos, V3::from_turns(beast.yaw));
                    return Some(walking(me, out, t.anchor, 0));
                }
                // Out of it: face it and wait for the stand.
                self.intent = OFF;
                Some(looking(me, flat(t.anchor), 0))
            }
            // The lane: sidestep it.
            sandmaw::BREACH => {
                let t = beast.telegraph()?;
                let end = t.anchor.add(t.along.scale(t.sweep));
                let gap = sim::math::flat_segment_gap(me.pos, t.anchor, end);
                if gap.raw() > t.radius.add(body).add(MARGIN).raw() {
                    return None;
                }
                self.intent = EVADE;
                let side = V3::new(t.along.z.neg(), Fx::ZERO, t.along.x);
                let rel = flat(me.pos.sub(t.anchor));
                let out = if side.dot(rel).raw() >= 0 {
                    side
                } else {
                    side.scale(Fx::ONE.neg())
                };
                Some(walking(me, out, beast.pos, 0))
            }
            // The spray: out of the cone, sideways.
            sandmaw::SPIT => {
                let discs = fight::spit_discs(beast);
                let inside = discs
                    .iter()
                    .any(|(c, r)| wide_flat_dist(*c, me.pos).raw() <= r.add(body).add(MARGIN).raw());
                if !inside {
                    return None;
                }
                self.intent = if me.pos.y.raw() > Fx::ratio(1, 4).raw() {
                    COVER
                } else {
                    EVADE
                };
                // Behind a boulder if there is one near; else across.
                if let Some(lee) = lee(w, beast, me.pos) {
                    let to = flat(lee.sub(me.pos));
                    if to.flat_len().raw() > SETTLED.raw() {
                        return Some(walking(me, to.normalized(), beast.pos, 0));
                    }
                    return Some(looking(me, beast.pos, 0));
                }
                let along = V3::from_turns(beast.yaw);
                let side = V3::new(along.z.neg(), Fx::ZERO, along.x);
                let rel = flat(me.pos.sub(beast.pos));
                let out = if side.dot(rel).raw() >= 0 {
                    side
                } else {
                    side.scale(Fx::ONE.neg())
                };
                // A spray this close is not walked out of: the dodge, loud
                // as it is, on the last frames before it comes.
                Some(self.out_or_dodge(me, out, beast.pos, left, live))
            }
            // The tail: under it.
            sandmaw::LASH => {
                let discs = fight::lash_discs(beast);
                let inside = discs
                    .iter()
                    .any(|(c, r)| wide_flat_dist(*c, me.pos).raw() <= r.add(body).add(MARGIN).raw());
                if !inside {
                    return None;
                }
                self.intent = DUCK;
                Some(looking(me, part_at(beast, sandmaw::VENT_2), Input::CROUCH))
            }
            // The open mouth: hit into it if a swing lands before it shuts;
            // out of it otherwise.
            sandmaw::SWALLOW => {
                let t = beast.telegraph()?;
                if wide_flat_dist(t.anchor, me.pos).raw() > t.radius.add(body).add(MARGIN).raw() {
                    return None;
                }
                let teeth = part_at(beast, sandmaw::TEETH);
                let poke = sim::moves::get(me.class, sim::state::SLOT_POKE);
                let lands = (poke.startup as i32) + 1 < left as i32 - REACTION as i32;
                let near = wide_flat_dist(teeth, me.pos).raw() <= poke.reach.add(Fx::ONE).raw();
                if !live && lands && near && me.action.actionable() {
                    self.intent = GAG;
                    self.cooldown = SWING_GAP;
                    return Some(looking(me, teeth, Input::LEFT));
                }
                self.intent = EVADE;
                let out = away_from(t.anchor, me.pos, V3::from_turns(beast.yaw));
                Some(self.out_or_dodge(me, out, beast.pos, left, live))
            }
            // The dive: away from the hole.
            sandmaw::SOUND | sandmaw::DIVE => {
                let t = beast.telegraph()?;
                if wide_flat_dist(t.anchor, me.pos).raw() > t.radius.add(body).add(MARGIN).raw() {
                    return None;
                }
                self.intent = EVADE;
                let out = away_from(t.anchor, me.pos, V3::from_turns(beast.yaw));
                Some(walking(me, out, beast.pos, 0))
            }
            _ => None,
        }
    }

    /// Walking out, or -- in the last frames before a hit it cannot walk out
    /// of -- the dodge out.
    fn out_or_dodge(
        &mut self,
        me: &sim::state::Player,
        out: V3,
        look_at: V3,
        left: u16,
        live: bool,
    ) -> Input {
        let real_left = if live {
            0
        } else {
            left as i32 - REACTION as i32 - self.slop
        };
        if real_left <= DODGE_LEAD && self.dodge_left == 0 && me.action.actionable() {
            self.dodge_left = sim::tuning::dodge_frames() + DODGE_REST;
            return walking(me, out, look_at, Input::SHIFT);
        }
        walking(me, out, look_at, 0)
    }

    /// **It is up**: to its back at the low spiracles, or -- when it came up
    /// facing it -- in front at the throat, ready to hit into the swallow's
    /// open mouth. Beached: onto its back.
    fn punish(&mut self, w: &World, me: &sim::state::Player, beast: &Monster) -> Input {
        let _ = w;
        self.bait_at = None;
        self.quiet_for = 0;
        if let Doing::Toppled { left } = beast.doing {
            return self.climb(me, beast, left);
        }
        let ahead = V3::from_turns(beast.yaw);
        let rel = flat(me.pos.sub(beast.pos));
        // Which side it plays this stand: chosen as it comes up, from where
        // it is standing then, and kept.
        if self.side.is_none() {
            let cos = if rel.flat_len().raw() > 0 {
                ahead.dot(rel.normalized())
            } else {
                Fx::ZERO
            };
            self.side = Some(cos.raw() > FRONT_COS.raw());
        }
        let front = self.side == Some(true);
        let (station, targets): (V3, &[usize]) = if front {
            (
                flat(beast.pos).add(ahead.scale(IN_FRONT)),
                &[sandmaw::THROAT_2, sandmaw::THROAT_3, sandmaw::THROAT_1],
            )
        } else {
            (
                flat(beast.pos).sub(ahead.scale(BEHIND)),
                &[sandmaw::VENT_1, sandmaw::VENT_2],
            )
        };
        let to = flat(station.sub(me.pos));
        let target = targets
            .iter()
            .map(|v| part_at(beast, *v))
            .min_by_key(|v| wide_flat_dist(*v, me.pos).raw())
            .unwrap_or(beast.pos);
        let walk = if to.flat_len().raw() > SETTLED.raw() {
            let d = to.normalized();
            // Round it, not through it: past its flank if it is in the way.
            let near = rel.flat_len().raw() < Fx::from_int(4).raw();
            let wrong_side = (ahead.dot(rel).raw() > 0) != front;
            let d = if near && wrong_side {
                let side = V3::new(ahead.z.neg(), Fx::ZERO, ahead.x);
                let side = if side.dot(rel).raw() >= 0 {
                    side
                } else {
                    side.scale(Fx::ONE.neg())
                };
                let toward = if front { ahead } else { ahead.scale(Fx::ONE.neg()) };
                side.add(toward).normalized()
            } else {
                d
            };
            Some(d)
        } else {
            None
        };
        // **Only a swing that is over before it can act again**.
        let window = beast.frames_until_free() as i32 - REACTION as i32;
        let poke = sim::moves::get(me.class, sim::state::SLOT_POKE);
        let poke_busy = (poke.startup + poke.active + poke.recovery) as i32;
        let heavy_busy = crate::heavy_commitment(me.class) as i32;
        let reach = poke.reach.add(Fx::ONE);
        let near = wide_flat_dist(target, me.pos).raw() <= reach.raw();
        let swing = if self.cooldown == 0 && me.action.actionable() && near {
            if window > heavy_busy + EXIT {
                self.cooldown = SWING_GAP;
                heavy(me.class)
            } else if window > poke_busy + EXIT {
                self.cooldown = SWING_GAP;
                Input::LEFT
            } else {
                0
            }
        } else {
            0
        };
        self.intent = PUNISH;
        let mut input = looking(me, target, swing);
        if let Some(dir) = walk {
            // Walking: the look stays on the target, the keys go where it is
            // going, relative to the look.
            let d = flat(target.sub(me.pos));
            let yaw = atan2_turns(d.z, d.x);
            input.bits |= steer(yaw, dir);
        }
        input
    }

    /// **Beached**: up onto its back from beside the middle of it.
    fn climb(&mut self, me: &sim::state::Player, beast: &Monster, left: u16) -> Input {
        let middle = part_at(beast, sandmaw::SEG_ROOT);
        let to = flat(middle.sub(me.pos));
        self.intent = CLIMB;
        // Not worth getting on for the last of it, or out of its hop.
        let back = middle.y.add(sandmaw::SPECIES.shape(sandmaw::SEG_ROOT).max.y);
        if (left as i32) < REACTION as i32 + 20 || back.raw() >= self.hop.raw() {
            return self.punish_beached(me, beast);
        }
        let close = to.flat_len().raw() < Fx::from_int(3).raw();
        if close && me.grounded && self.leap_left == 0 {
            self.leap_left = LEAP_HOLD;
            self.leap_dir = to.normalized();
        }
        let jump = if self.leap_left > 0 { Input::SPACE } else { 0 };
        walking(me, to.normalized(), middle, jump)
    }

    /// Beached but not climbing: hit what is in reach from the sand.
    fn punish_beached(&mut self, me: &sim::state::Player, beast: &Monster) -> Input {
        let target = [sandmaw::HEAD_PART, sandmaw::SEG_N3, sandmaw::SEG_ROOT, sandmaw::SEG_T2]
            .into_iter()
            .map(|p| part_at(beast, p))
            .min_by_key(|p| wide_flat_dist(*p, me.pos).raw())
            .unwrap_or(beast.pos);
        let poke = sim::moves::get(me.class, sim::state::SLOT_POKE);
        let to = flat(target.sub(me.pos));
        self.intent = PUNISH;
        if to.flat_len().raw() > poke.reach.raw() {
            return walking(me, to.normalized(), target, 0);
        }
        let swing = if self.cooldown == 0 && me.action.actionable() {
            self.cooldown = SWING_GAP;
            heavy(me.class)
        } else {
            0
        };
        looking(me, target, swing)
    }

    /// **Aboard**: to the middle of the back, at the vents, crouched through
    /// the writhe -- and off when they clamp.
    fn ride(&mut self, me: &sim::state::Player, beast: &Monster) -> Input {
        let along = V3::from_turns(beast.yaw);
        if self.leap_left > 0 || matches!(beast.doing.attacking(), Some(sandmaw::DIVE | sandmaw::SOUND))
        {
            self.intent = LEAVE;
            let side = V3::new(along.z.neg(), Fx::ZERO, along.x);
            if self.leap_left == 0 {
                self.leap_left = LEAP_HOLD;
                self.leap_dir = side;
            }
            return walking(me, side, me.pos.add(side), Input::SPACE);
        }
        let vent = sandmaw::VENTS
            .iter()
            .map(|v| part_at(beast, *v))
            .min_by_key(|v| wide_flat_dist(*v, me.pos).raw())
            .unwrap_or(beast.pos);
        let to = flat(vent.sub(me.pos));
        self.intent = WORK;
        let walk = if to.flat_len().raw() > Fx::ratio(8, 10).raw() {
            let yaw = atan2_turns(to.z, to.x);
            steer(yaw, to)
        } else {
            0
        };
        let swing = if self.cooldown == 0 && matches!(me.action, Action::Free) {
            self.cooldown = SWING_GAP;
            Input::LEFT
        } else {
            0
        };
        let mut input = looking(me, vent, swing | walk);
        if walk == 0 {
            input.bits |= Input::CROUCH;
        }
        input
    }

    /// **Under the sand**: quiet until the wake is near, then one noise and
    /// off it.
    fn wait(&mut self, w: &World, me: &sim::state::Player, beast: &Monster) -> Input {
        let head = fight::head_flat(beast);
        let d = wide_flat_dist(head, me.pos);
        // Walking off the noise it made.
        if let Some(at) = self.bait_at {
            let gone = wide_flat_dist(at, me.pos);
            if gone.raw() < OFF_BY.raw() && me.grounded {
                self.intent = OFF;
                return walking(me, self.off_dir, head, 0);
            }
            if self.since_bait > BAIT_WAIT as u32 {
                self.bait_at = None;
            } else {
                self.intent = QUIET;
                return looking(me, head, 0);
            }
        }
        self.quiet_for += 1;
        let off_rock = !matches!(
            w.terrain().material_under(me.pos),
            sim::arena::Material::Rock | sim::arena::Material::Stone
        );
        let feel = sandmaw::SPECIES.fight_fx(sim::species::FightField::FeelRadius);
        // Quiet for a moment first: a bait straight after walking is a
        // landing on top of the footfalls, and it may come up at either.
        if (d.raw() < BAIT_NEAR.raw() || self.quiet_for > BAIT_AFTER)
            && self.quiet_for > QUIET_FIRST
            && d.raw() > feel.raw()
            && me.grounded
            && off_rock
        {
            // A jump: the landing is the noise.
            self.intent = BAIT;
            self.bait_at = Some(flat(me.pos));
            // Off it across the wake's line rather than down it, to the side
            // with room.
            let out = away_from(head, me.pos, V3::new(Fx::ZERO, Fx::ZERO, Fx::ONE));
            let across = V3::new(out.z.neg(), Fx::ZERO, out.x);
            let side = if self.roll() & 1 == 0 {
                across
            } else {
                across.scale(Fx::ONE.neg())
            };
            self.off_dir = keep_in(w, me.pos, side);
            self.since_bait = 0;
            self.quiet_for = 0;
            self.leap_left = 6;
            self.leap_dir = V3::ZERO;
            return looking(me, head, Input::SPACE);
        }
        // Too near and quiet is how the sinkhole opens: walk off, which is
        // loud, but loud and moving is never bitten.
        if d.raw() <= feel.add(Fx::ONE).raw() {
            self.intent = OFF;
            let out = away_from(head, me.pos, V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO));
            return walking(me, keep_in(w, me.pos, out), head, 0);
        }
        self.intent = QUIET;
        looking(me, head, 0)
    }
}

/// A direction kept off the rim: turned back in toward the middle of the
/// Pan when it would walk out of it.
fn keep_in(w: &World, at: V3, dir: V3) -> V3 {
    let b = w.arena().bounds;
    let room = Fx::from_int(4);
    let ahead = at.add(dir.scale(Fx::from_int(3)));
    let out = ahead.x.raw() < b.lo_x.add(room).raw()
        || ahead.x.raw() > b.hi_x.sub(room).raw()
        || ahead.z.raw() < b.lo_z.add(room).raw()
        || ahead.z.raw() > b.hi_z.sub(room).raw();
    if out {
        flat(V3::ZERO.sub(at)).normalized()
    } else {
        dir
    }
}

/// **The lee of the nearest boulder** from the worm, if one is near: where
/// a fighter on an island hides from the spray.
fn lee(w: &World, beast: &Monster, me: V3) -> Option<V3> {
    sim::arena::sandmaw::ISLANDS
        .iter()
        .map(|(x, z)| V3::new(Fx::ratio(*x, 100), Fx::ZERO, Fx::ratio(*z, 100)))
        .filter(|c| wide_flat_dist(*c, me).raw() < Fx::from_int(5).raw())
        .map(|c| {
            let away = away_from(beast.pos, c, QUARTER_DIR);
            c.add(away.scale(Fx::ratio(13, 10)))
        })
        .find(|_| w.arena().id == sim::arena::ArenaId::SANDMAW)
}

const QUARTER_DIR: V3 = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);

impl Plan for Sandmaw {
    fn watch(&mut self, w: &World) {
        Sandmaw::watch(self, w);
    }

    fn act(&mut self, w: &World) -> Input {
        Sandmaw::act(self, w)
    }

    fn intent(&self) -> Intent {
        self.intent
    }
}

/// Its writhe throws riders; the dive ends every ride.
pub fn bucks(kind: u8) -> bool {
    matches!(kind, sandmaw::SOUND | sandmaw::DIVE)
}

pub static CARD: crate::plans::Card = crate::plans::Card {
    species: sim::species::SpeciesId::SANDMAW,
    plan: |who, seed, hop| Box::new(Sandmaw::new(who, seed, hop)),
    bucks,
    words: crate::plans::Words {
        weak_hits: "spiracle and throat hits",
        broken: "tooth rings broken",
        into_breakables: "damage into the teeth",
        into_breakables_why: "the swallow, answered",
        worst: "teeth taken",
        ride_for: "long enough to work the vents?",
        toppled_pool: "off the pool beside a beached worm",
    },
    tally: Some(|| Box::new(SandTally::default())),
    gamble: None,
};

// ---------------------------------------------------------------------------
// The report's Sandmaw lines (§9)
// ---------------------------------------------------------------------------

/// What the Sandmaw's report counts.
#[derive(Default)]
pub struct SandTally {
    /// The frame each fighter last made a noise.
    last_noise: [u32; sim::state::MAX_PLAYERS],
    /// Bites (rise-bite hits) on a hunter quiet for a second, split by
    /// whether the rise came out of something it felt.
    pub quiet_felt: u32,
    pub quiet_unfelt: u32,
    /// Rises, and how many of their markers covered what they acted on.
    pub rises: u32,
    pub covered: u32,
    /// Rises at an island's edge, to spit: not at a noise.
    pub island_rises: u32,
    /// Frames the hunter was neither heard nor felt, of the fight's.
    pub unperceived: u32,
    pub frames: u32,
    /// What it attended, by noise kind (and felt).
    pub heard: [u32; 10],
    pub felt: u32,
    last_attend: u32,
    /// Stands, and beaches by route.
    pub stands: u32,
    pub beaches: [u32; 6],
    /// Frames on rock, and spits that landed there.
    pub island_frames: u32,
    pub island_spits: u32,
    /// Swallows, and how each ended: escaped on gulp n, rescued, spat.
    pub swallows: u32,
    pub escaped: [u32; 5],
    pub rescued: u32,
    pub spat: u32,
    pub slipped: u32,
    /// The frame the tooth ring broke.
    pub teeth: Option<u32>,
    /// The new windows: threatening only when it could move and had the
    /// hunter in its last glance; the rest as the shared report; and the
    /// frames it could move and had not.
    pub threat: [u32; 4],
    pub blind: u32,
    /// Hits its own rule says were unanswerable: a rise or undertow whose
    /// marker covered nothing the victim did.
    pub bad_bites: u32,
}

impl Tally for SandTally {
    fn observe(&mut self, before: &World, after: &World) {
        let (Some(was), Some(m)) = (before.monster(), after.monster()) else {
            return;
        };
        let sp = m.sp();
        // Noises made, by whom.
        for n in sim::noise::all(&after.lore) {
            let who = n.who as usize;
            if who < self.last_noise.len() && n.born == after.frame {
                self.last_noise[who] = after.frame;
            }
        }
        let fighting = m.brain.grace == 0 && m.alive();
        if fighting {
            self.frames += 1;
        }
        let me = &after.players[0];
        let ground = after.terrain();
        let head = fight::head_flat(m);
        // Perceived: felt now, or a noise of the hunter's reached it within
        // its glance.
        let felt = me.health > 0 && fight::feels(m, head, me.pos, &ground);
        let window = m.glance_frames() as u32;
        let heard = (0..after.lore.layout().noises as usize).any(|i| {
            sim::noise::nth(&after.lore, i).is_some_and(|n| {
                n.who == 0
                    && fight::heard_cell(&after.lore, i)
                    && after.frame.wrapping_sub(n.born) <= window
            })
        });
        if fighting {
            if !felt && !heard {
                self.unperceived += 1;
            }
            let t = Threat::of(m.frames_until_free() as i32);
            if t == Threat::Threatening && !felt && !heard {
                self.blind += 1;
            } else {
                self.threat[t as usize] += 1;
            }
        }
        // What it attended.
        let frame = after.lore.word(fight::word::ATTEND_FRAME);
        if frame != self.last_attend {
            self.last_attend = frame;
            if let Some(a) = fight::attended(&after.lore) {
                if a.felt() {
                    self.felt += 1;
                } else if let Some(k) = a.kind {
                    self.heard[(k as usize).min(9)] += 1;
                }
            }
        }
        // A rise committing: its marker against what it acted on.
        if let Doing::Startup { kind, left } = m.doing {
            if kind == sandmaw::RISE && left + 1 == sp.attack(sandmaw::RISE).startup {
                if let (Some(acted), Some(t)) = (fight::acted_on(&after.lore), m.telegraph()) {
                    if acted.on_rock {
                        self.island_rises += 1;
                    } else {
                        self.rises += 1;
                        if wide_flat_dist(t.anchor, acted.at).raw() <= t.radius.raw() {
                            self.covered += 1;
                        }
                    }
                }
            }
        }
        // A stand.
        if matches!(m.doing, Doing::Active { kind: sandmaw::RISE, .. })
            && !matches!(was.doing, Doing::Active { kind: sandmaw::RISE, .. })
        {
            self.stands += 1;
        }
        // A beach, by route.
        if matches!(m.doing, Doing::Toppled { .. }) && !matches!(was.doing, Doing::Toppled { .. }) {
            // The route is on its body the frame it goes over; a poise
            // broken by the shared ladder leaves none, and is a stand broken.
            let r = match m.own[fight::body::ROUTE] {
                0 => fight::route::BROKEN as usize,
                r => r as usize,
            };
            self.beaches[r.min(5)] += 1;
        }
        // Island time, and spits taken there.
        if me.health > 0
            && matches!(
                ground.material_under(me.pos),
                sim::arena::Material::Rock | sim::arena::Material::Stone
            )
        {
            self.island_frames += 1;
        }
        let lost = before.players[0].health - me.health;
        let doing_was = was.doing.attacking();
        if lost > 0 && m.doing.attacking() == Some(sandmaw::SPIT) && me.pos.y.raw() > 0 {
            self.island_spits += 1;
        }
        // A bite: on a quiet hunter? And did its marker cover them?
        let bit = lost > 0 && matches!(m.doing, Doing::Active { kind: sandmaw::RISE, .. });
        let _ = doing_was;
        if bit {
            let quiet = after.frame.saturating_sub(self.last_noise[0]) >= 60;
            let from_felt = fight::acted_on(&after.lore).is_some_and(|a| a.felt());
            if quiet {
                if from_felt {
                    self.quiet_felt += 1;
                } else {
                    self.quiet_unfelt += 1;
                }
            }
            // §6: a bite whose marker covered neither a noise the victim
            // made nor a point they were felt at.
            let covered = fight::acted_on(&after.lore).is_some_and(|a| {
                a.who == 0
                    && m.telegraph()
                        .is_some_and(|t| wide_flat_dist(t.anchor, a.at).raw() <= t.radius.raw())
            });
            if !covered {
                self.bad_bites += 1;
            }
        }
        // Swallows and how they ended.
        if fight::holding(&after.lore).is_some() && fight::holding(&before.lore).is_none() {
            self.swallows += 1;
        }
        let ended = after.lore.word(fight::word::ENDED);
        if ended != before.lore.word(fight::word::ENDED) && ended != 0 {
            match ended >> 8 {
                fight::end::ESCAPED => self.escaped[((ended & 0xFF) as usize).min(4)] += 1,
                fight::end::RESCUED => self.rescued += 1,
                fight::end::SPAT => self.spat += 1,
                fight::end::SLIPPED => self.slipped += 1,
                _ => {}
            }
        }
        if self.teeth.is_none() && m.broken(sandmaw::TEETH) {
            self.teeth = Some(after.frame);
        }
    }

    fn unanswerable(&self) -> u32 {
        self.bad_bites
    }

    fn lines(&self) -> Vec<(String, String, String)> {
        let pct = |n: u32, d: u32| {
            if d == 0 {
                "--".to_string()
            } else {
                format!("{:.0}%", n as f32 * 100.0 / d as f32)
            }
        };
        let windows: u32 = self.threat.iter().sum::<u32>() + self.blind;
        let kinds = [
            "footfall", "rock", "landing", "dodge", "hit", "rush", "stone", "shield", "quake",
        ];
        let total_heard: u32 = self.heard.iter().sum::<u32>() + self.felt;
        let heard: Vec<String> = kinds
            .iter()
            .enumerate()
            .filter(|(i, _)| self.heard[i + 1] > 0)
            .map(|(i, k)| format!("{k} {}", pct(self.heard[i + 1], total_heard)))
            .chain(std::iter::once(format!("felt {}", pct(self.felt, total_heard))))
            .collect();
        vec![
            (
                "bitten while quiet".into(),
                format!("{} felt / {} unfelt", self.quiet_felt, self.quiet_unfelt),
                "felt: the undertow working; unfelt: a bug".into(),
            ),
            (
                "marker covered the noise".into(),
                format!("{}/{}", self.covered, self.rises),
                "every rise over what it heard".into(),
            ),
            (
                "unperceived share".into(),
                pct(self.unperceived, self.frames),
                "neither heard nor felt; about a third".into(),
            ),
            (
                "heard, per kind".into(),
                heard.join(", "),
                "no kind above half".into(),
            ),
            (
                "stands, beaches".into(),
                format!(
                    "{} / {} (broken {}, stone {}, lane {}, gag {}, knocked {})",
                    self.stands,
                    self.beaches.iter().sum::<u32>(),
                    self.beaches[1],
                    self.beaches[2],
                    self.beaches[3],
                    self.beaches[4],
                    self.beaches[5]
                ),
                "the window, and the big one by route".into(),
            ),
            (
                "island time".into(),
                pct(self.island_frames, self.frames),
                format!(
                    "{} rises to spit, {} spits taken there",
                    self.island_rises, self.island_spits
                ),
            ),
            (
                "swallows".into(),
                format!(
                    "{} (out on gulp 1: {}, 2: {}, 3+: {}; rescued {}, spat {}, slipped {})",
                    self.swallows,
                    self.escaped[1],
                    self.escaped[2],
                    self.escaped[3] + self.escaped[4],
                    self.rescued,
                    self.spat,
                    self.slipped
                ),
                "most out on the first or second gulp".into(),
            ),
            (
                "tooth ring broken".into(),
                self.teeth
                    .map_or("no".into(), |f| format!("at {:.1} min", f as f32 / 3600.0)),
                "the permanent change, in a won fight".into(),
            ),
            (
                "threatening (perceived)".into(),
                pct(self.threat[Threat::Threatening as usize], windows),
                "could move, and had you; ~35%".into(),
            ),
            (
                "unperceived band".into(),
                pct(self.blind, windows),
                "could move, and had not; ~30%".into(),
            ),
            (
                "poke / way in / walk up".into(),
                format!(
                    "{} / {} / {}",
                    pct(self.threat[Threat::PokeOnly as usize], windows),
                    pct(self.threat[Threat::Skilled as usize], windows),
                    pct(self.threat[Threat::WalkUp as usize], windows)
                ),
                "walk up ~20%: the stands and the beaches".into(),
            ),
        ]
    }
}
