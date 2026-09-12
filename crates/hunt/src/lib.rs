//! A scripted hunter, and the report that says whether the fight it plays is
//! any good.
//!
//! Its own crate rather than a module of `sim`, because it is not simulation:
//! it is a **player**, standing in for a person so that a fight can be run a
//! thousand times and measured. The numbers in here are the measuring
//! instrument, not feel values, which is exactly why they should not be in the
//! Oven -- and keeping them out of `crates/sim/src` is what stops the knob
//! guard from having to be argued with.
//!
//! Two rules make the measurement worth anything.
//!
//! **The hunter reacts late.** It sees the world as it was `REACTION` frames
//! ago and never sooner. A bot that dodges on the first frame of a startup
//! proves nothing about whether that move was readable; one that cannot see the
//! present at all proves quite a lot.
//!
//! **It does not read the creature's mind.** It gets what a player gets from
//! the screen -- where the animal is, which way it is pointed, and what it has
//! visibly begun -- and nothing about which move the control algorithm is about
//! to pick.

pub mod report;

pub use report::{Outcome, Report};

use sim::fixed::Fx;
use sim::math::{atan2_turns, wrap_turns};
use sim::monster::{self, Doing, Monster};
use sim::state::{Action, MAX_PLAYERS, Phase};
use sim::{Input, V3, World};

/// Frames of delay between the world changing and the hunter knowing.
///
/// Human reaction at 60 Hz is roughly this. It is the reason the report's
/// "reactable share" means something: below this a move genuinely cannot be
/// answered on sight, and the hunter is held to the same limit a person is.
pub const REACTION: usize = 15;

/// What the hunter can see, one frame of it.
#[derive(Clone, Copy, Default)]
struct Seen {
    beast_pos: V3,
    beast_yaw: Fx,
    doing: u32,
    kind: u8,
    toppled: bool,
    open: bool,
    alive: bool,
}

/// What the hunter is trying to do. Committing to an intent for a while is what
/// stops it dithering, and it is also what a person does.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Intent {
    /// Hold at punishing distance, off the creature's nose.
    Circle,
    /// Close and hit it.
    Punish,
    /// Get behind it and up the tail.
    Climb,
    /// On the back, walking to the ridge.
    ToRidge,
    /// On the back, hitting the ridge.
    Work,
    /// On the back, holding on.
    Brace,
    /// Get out of the way.
    Evade,
}

/// One scripted fighter.
pub struct Hunter {
    who: usize,
    memory: [Seen; REACTION + 1],
    at: usize,
    filled: usize,
    pub intent: Intent,
    /// Frames left of a commitment to climbing, so it does not abandon the
    /// attempt the first time the creature glances at it.
    climb_left: u16,
    /// Frames since it last threw an attack, so it does not mash.
    cooldown: u16,
    dodge_left: u16,
    /// Frames left of holding the jump button.
    ///
    /// Jump height is variable here -- hold for higher -- so a bot that presses
    /// the key for one frame short-hops one and a half metres and bounces off
    /// the side of a tail that sits at one and three quarters. Committing to a
    /// number of frames is what a person does without thinking about it.
    leap_left: u16,
}

impl Hunter {
    pub fn new(who: usize) -> Hunter {
        Hunter {
            who,
            memory: [Seen::default(); REACTION + 1],
            at: 0,
            filled: 0,
            intent: Intent::Circle,
            climb_left: 0,
            cooldown: 0,
            dodge_left: 0,
            leap_left: 0,
        }
    }

    /// Record this frame. Called before `act`, every tick, so the delay line
    /// stays honest.
    pub fn watch(&mut self, w: &World) {
        let seen = match &w.monster {
            Some(b) => Seen {
                beast_pos: b.pos,
                beast_yaw: b.yaw,
                doing: b.doing.tag(),
                kind: b.doing.attacking().unwrap_or(monster::NO_PART),
                toppled: matches!(b.doing, Doing::Toppled { .. }),
                open: b.doing.open(),
                alive: b.alive(),
            },
            None => Seen::default(),
        };
        self.memory[self.at] = seen;
        self.at = (self.at + 1) % self.memory.len();
        self.filled = (self.filled + 1).min(self.memory.len());
    }

    /// What it saw `REACTION` frames ago, or the oldest it has.
    fn recall(&self) -> Seen {
        let back = REACTION.min(self.filled.saturating_sub(1));
        let idx = (self.at + self.memory.len() - 1 - back) % self.memory.len();
        self.memory[idx]
    }
}

/// A quarter turn, in `Fx`.
const QUARTER: Fx = Fx::from_raw(1 << 14);
/// Half of one, for splitting a range span.
const HALF: Fx = Fx::from_raw(1 << 15);

/// Where it wants to stand, as a bearing off the creature's nose: about a
/// hundred degrees round, which is outside the cone every forward move needs
/// and inside the reach of its own.
///
/// This is the whole ground strategy, and it is the one the fight is *supposed*
/// to teach: at the flank, only the tail can reach you, and the tail announces
/// itself for twenty-eight frames.
const FLANK: Fx = Fx::ratio(25, 100);
/// Far enough round that the tail is closer than the head. The climb only
/// starts from here, and it is deliberately just inside the station above: a
/// threshold the hunter's own resting position does not meet is a threshold
/// that fires by accident or not at all.
const BEHIND: Fx = Fx::ratio(22, 100);
/// How far out that station sits, from the creature's centre.
const HOLD: Fx = Fx::ratio(35, 10);
/// Close enough for a poke to touch a leg or the barrel.
const STRIKE: Fx = Fx::ratio(39, 10);
/// Do not bother correcting for less than this.
const SETTLED: Fx = Fx::ratio(7, 10);
/// Once it decides to climb, it keeps trying for this long.
const CLIMB_COMMIT: u16 = 150;
/// Jump for the tail from here.
const TAIL_LEAP: Fx = Fx::ratio(26, 10);
/// Frames to hold the jump for, climbing. Enough to clear a tail at one and
/// three quarter metres with room to come down onto it, and well short of the
/// full hop, which sails over.
const LEAP_HOLD: u16 = 13;
/// Where on the creature's back it stands to reach the ridge, in body space.
const WORK_SPOT: Fx = Fx::ratio(-12, 10);
/// Stick deadzone when turning a direction into four keys.
const DEAD: Fx = Fx::ratio(3, 10);
/// Frames between attacks, so it does not mash into its own recovery.
const SWING_GAP: u16 = 6;

fn turns_to_aim(turns: Fx) -> u16 {
    (turns.raw() as u32 & 0xFFFF) as u16
}

/// Four keys from a direction, given where the fighter is looking.
///
/// The inverse of `state::move_dir`, and deliberately lossy in the same way a
/// keyboard is: eight directions, not three hundred and sixty.
fn steer(aim: Fx, want: V3) -> u16 {
    let want = V3::new(want.x, Fx::ZERO, want.z).normalized();
    if want.flat_len().raw() == 0 {
        return 0;
    }
    let forward = V3::from_turns(aim);
    let right = V3::from_turns(aim.add(QUARTER));
    let f = want.dot(forward);
    let r = want.dot(right);
    let mut bits = 0;
    if f.raw() > DEAD.raw() {
        bits |= Input::W;
    } else if f.raw() < DEAD.neg().raw() {
        bits |= Input::S;
    }
    if r.raw() > DEAD.raw() {
        bits |= Input::D;
    } else if r.raw() < DEAD.neg().raw() {
        bits |= Input::A;
    }
    bits
}

/// Moves that move the back hard enough to be worth answering for.
fn bucks(kind: u8) -> bool {
    matches!(kind, monster::SWEEP | monster::SLAM | monster::SHAKE)
}

/// The tag `Doing::Startup` reports. Compared as a tag because that is all the
/// hunter's delay line keeps -- it remembers what it saw, not the enum.
const STARTUP: u32 = 1;
const ACTIVE: u32 = 2;

impl Hunter {
    /// One frame of decision. `watch` must have been called first.
    pub fn act(&mut self, w: &World) -> Input {
        self.cooldown = self.cooldown.saturating_sub(1);
        self.climb_left = self.climb_left.saturating_sub(1);
        self.dodge_left = self.dodge_left.saturating_sub(1);
        self.leap_left = self.leap_left.saturating_sub(1);

        let me = w.players[self.who];
        let (Some(beast), true) = (w.monster, me.health > 0) else {
            return Input::default();
        };
        let seen = self.recall();
        if !seen.alive || !matches!(w.phase, Phase::Fighting) {
            return Input::default();
        }
        // Where its feet are is proprioception, not eyesight, so the ride reads
        // the live mount. Everything about the *creature* comes from the delay
        // line.
        if me.aboard() {
            self.ride(&me, seen)
        } else {
            self.ground(&me, &beast, seen)
        }
    }

    fn ground(&mut self, me: &sim::state::Player, beast: &Monster, seen: Seen) -> Input {
        let _ = beast;
        let to_beast = V3::new(
            seen.beast_pos.x.sub(me.pos.x),
            Fx::ZERO,
            seen.beast_pos.z.sub(me.pos.z),
        );
        let range = to_beast.flat_len();
        let toward = to_beast.normalized();
        let aim = atan2_turns(toward.z, toward.x);
        let wire = turns_to_aim(aim.sub(me.carry_yaw));
        // Where it is standing, as the creature sees it: zero is dead in front,
        // half a turn is behind the tail.
        let bearing = wrap_turns(atan2_turns(toward.z.neg(), toward.x.neg()).sub(seen.beast_yaw));
        let this_side = if bearing.raw() >= 0 {
            FLANK
        } else {
            FLANK.neg()
        };

        // A move it has visibly begun, and how far that move reaches.
        let coming = (seen.doing == STARTUP || seen.doing == ACTIVE)
            .then_some(seen.kind)
            .filter(|k| *k != monster::NO_PART)
            .map(monster::attack);
        let threatened = coming.is_some_and(|m| {
            m.damage > 0 && range.raw() < m.ideal_range.add(m.range_span.mul(HALF)).raw()
        });

        // 1. Get out of the way. The answers are not the same, which is the
        //    whole point of having six moves: a sweep goes under you, and
        //    everything else goes through where you are standing.
        if threatened && self.dodge_left == 0 {
            self.intent = Intent::Evade;
            self.dodge_left = sim::tuning::dodge_frames();
            if seen.kind == monster::SWEEP {
                return Input::aimed(Input::SPACE, wire);
            }
            let out = V3::from_turns(seen.beast_yaw.add(if bearing.raw() >= 0 {
                QUARTER
            } else {
                QUARTER.neg()
            }));
            return Input::aimed(steer(aim, out) | Input::SHIFT, wire);
        }

        // 2. Take the free window if there is one worth taking. A toppled
        //    animal is worth climbing; an ordinary recovery from behind is too.
        // A creature on the ground is worth climbing whatever else is true.
        // Otherwise the moment is an opening, taken from behind it, close
        // enough to reach the tail before the window closes.
        let worth_climbing = seen.toppled
            || (seen.open
                && bearing.abs().raw() > BEHIND.raw()
                && range.raw() < Fx::from_int(8).raw());
        if worth_climbing && self.climb_left == 0 {
            self.climb_left = CLIMB_COMMIT;
        }

        if self.climb_left > 0 {
            self.intent = Intent::Climb;
            // The tail, reconstructed the way a player eyeballs it: from where
            // the animal was and which way it was pointed.
            let tail = monster::shape(monster::TAIL_BASE);
            let mid = tail.min.add(tail.max).scale(HALF);
            let along = V3::from_turns(seen.beast_yaw);
            let side = V3::from_turns(seen.beast_yaw.add(QUARTER));
            let spot = seen
                .beast_pos
                .add(along.scale(mid.x))
                .add(side.scale(mid.z));
            let to_spot = V3::new(spot.x.sub(me.pos.x), Fx::ZERO, spot.z.sub(me.pos.z));
            if to_spot.flat_len().raw() < TAIL_LEAP.raw() && me.grounded && self.leap_left == 0 {
                self.leap_left = LEAP_HOLD;
            }
            let jump = if self.leap_left > 0 { Input::SPACE } else { 0 };
            return Input::aimed(steer(aim, to_spot) | jump, wire);
        }

        // 3. Hold station at the flank, and poke whatever comes into reach.
        self.intent = if seen.open {
            Intent::Punish
        } else {
            Intent::Circle
        };
        let station = seen
            .beast_pos
            .add(V3::from_turns(seen.beast_yaw.add(this_side)).scale(HOLD));
        let to_station = V3::new(station.x.sub(me.pos.x), Fx::ZERO, station.z.sub(me.pos.z));
        let walk = if to_station.flat_len().raw() > SETTLED.raw() {
            steer(aim, to_station)
        } else {
            0
        };
        let swing = if self.cooldown == 0 && range.raw() < STRIKE.raw() && !threatened {
            self.cooldown = SWING_GAP;
            // Nothing to punish means a poke; a real opening is worth the slow
            // one, which is what makes an opening worth having.
            if seen.open {
                Input::LEFT | Input::SHIFT
            } else {
                Input::LEFT
            }
        } else {
            0
        };
        Input::aimed(walk | swing, wire)
    }

    fn ride(&mut self, me: &sim::state::Player, seen: Seen) -> Input {
        // Face along the creature's spine, toward the head. The aim that goes
        // on the wire has `carry_yaw` taken back out of it, because the
        // simulation will add it again -- the fighter's look angle is the input
        // plus what the animal has turned under them.
        let along = V3::from_turns(seen.beast_yaw);
        let side = V3::from_turns(seen.beast_yaw.add(QUARTER));
        let aim = atan2_turns(along.z, along.x);
        let wire = turns_to_aim(aim.sub(me.carry_yaw));

        let coming = (seen.doing == STARTUP)
            .then_some(seen.kind)
            .filter(|k| *k != monster::NO_PART);
        if let Some(kind) = coming.filter(|k| bucks(*k)) {
            if kind == monster::SLAM {
                // Nothing holds through that. Leave, and take the creature's
                // own speed with you.
                self.intent = Intent::Brace;
                return Input::aimed(Input::SPACE, wire);
            }
            self.intent = Intent::Brace;
            return Input::aimed(Input::CROUCH, wire);
        }

        let dx = WORK_SPOT.sub(me.local.x);
        let dz = me.local.z.neg();
        if dx.abs().raw() > Fx::ratio(4, 10).raw() || dz.abs().raw() > Fx::ratio(5, 10).raw() {
            self.intent = Intent::ToRidge;
            let want = along.scale(dx).add(side.scale(dz));
            return Input::aimed(steer(aim, want), wire);
        }

        self.intent = Intent::Work;
        let swing = if self.cooldown == 0 && matches!(me.action, Action::Free) {
            self.cooldown = SWING_GAP;
            Input::LEFT
        } else {
            0
        };
        Input::aimed(swing, wire)
    }
}

/// Run a whole hunt and hand back the report.
///
/// One hunter and one idle second fighter by default: the numbers are set for a
/// single player, and a monster tuned for two is a different monster.
///
/// `seed` starts the creature's generator somewhere else. A hunt is completely
/// deterministic, so without it every repeat is the same fight -- which tells
/// you nothing about the spread, and the spread is most of what you want to
/// know about a creature that chooses.
pub fn play(classes: [sim::Class; MAX_PLAYERS], partners: usize, limit: u32, seed: u32) -> Report {
    let mut w = World::hunt(classes);
    if let Some(beast) = w.monster.as_mut() {
        beast.brain.rng = seed | 1;
    }
    let count = partners.clamp(1, MAX_PLAYERS);
    let mut bots: Vec<Hunter> = (0..count).map(Hunter::new).collect();
    // A fighter nobody is driving is not a fighter standing very still: it is a
    // target the creature will happily charge across the arena at for twenty
    // minutes, which is what the first long run of this harness actually
    // measured. Absent hunters are out of the hunt.
    for p in w.players.iter_mut().skip(count) {
        p.health = 0;
    }
    let mut report = Report::new();
    while w.frame < limit && matches!(w.phase, Phase::Fighting) {
        let mut inputs = [Input::default(); MAX_PLAYERS];
        for bot in bots.iter_mut() {
            bot.watch(&w);
        }
        for bot in bots.iter_mut() {
            inputs[bot.who] = bot.act(&w);
        }
        let before = w.clone();
        w.advance(inputs);
        report.observe(&before, &w, &bots);
    }
    report.finish(&w);
    report
}
