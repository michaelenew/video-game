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
    /// How much longer what it is doing goes on for. Visible: it is the
    /// animation's phase, and a player who has seen a recovery twice knows
    /// roughly where the end of it is.
    left: u16,
    toppled: bool,
    open: bool,
    alive: bool,
    /// The lowest surface on the animal you could stand on, right now. What a
    /// player sees when they look at it: a shoulder down on the floor during
    /// a stumble, the barrel of a creature on its side, the tail of a standing
    /// one.
    mount: Mount,
    /// Where the ridge is, in the world. A rider who came up the shoulders
    /// walks *back* to it and one who came up the tail walks forward, so it is
    /// a place rather than a direction.
    ridge: V3,
}

/// A surface on the creature and where it is.
#[derive(Clone, Copy, Default)]
struct Mount {
    /// The middle of its top face, in the world.
    spot: V3,
    /// How high that face is off the floor.
    top: Fx,
}

/// The middle of a part's top face, in the world.
fn top_of(rig: &sim::beast::Rig, part: usize) -> V3 {
    let sh = monster::shape(part);
    let mid = sh.min.add(sh.max).scale(HALF);
    rig.part_to_world(part, V3::new(mid.x, sh.max.y, mid.z))
}

/// The lowest mountable face on the animal as it stands this frame.
fn lowest_mount(beast: &Monster) -> Mount {
    let rig = beast.rig();
    let mut best = Mount {
        spot: beast.pos,
        top: Fx::MAX,
    };
    for part in 0..monster::PARTS {
        if !sim::beast::SHAPES[part].mountable {
            continue;
        }
        let spot = top_of(&rig, part);
        if spot.y.raw() < best.top.raw() {
            best = Mount { spot, top: spot.y };
        }
    }
    best
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
    /// Take it on the shield. The Bulwark's answer to a blockable blow, and
    /// how its shield gets the weight a Slam spends.
    Guard,
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
    /// Frames left in which a blow just taken on the shield is answered with
    /// Slam. Felt, not seen: the shield took the hit in the present, so there
    /// is no reaction delay on knowing the creature's move is over.
    answer_left: u16,
    /// The shield's weight last frame, to feel a blow land on it.
    weight_was: Fx,
    dodge_left: u16,
    /// Frames until the hit of a move it has seen begin will arrive, or
    /// `i32::MAX` if nothing is coming. Recomputed every frame.
    hit_due: i32,
    /// **How far off its timing is, this move.** Drawn once per move it sees
    /// begin, and added to when it dodges or jumps. A bot that dodged on the
    /// exact frame every time would never be hit by anything it could see,
    /// and then "landed" would say nothing about whether a tell can be read
    /// -- only whether a perfect player survives it. A person is a few frames
    /// early or late, and so is this.
    slop: i32,
    /// Its own generator, for the slop. Seeded per hunt so a hunt is still
    /// reproducible.
    rng: u32,
    /// Frames it has seen the creature standing between moves, in a row.
    /// **The beat before its next move is an opening a person learns**: it is
    /// the same length every time, so standing there for part of it tells
    /// you how much of it is left.
    between: u16,
    /// Frames left of holding the jump button.
    ///
    /// Jump height is variable here -- hold for higher -- so a bot that presses
    /// the key for one frame short-hops one and a half metres and bounces off
    /// the side of a tail that sits at one and three quarters. Committing to a
    /// number of frames is what a person does without thinking about it.
    leap_left: u16,
    /// How high its own full hop goes. A player knows this about themselves
    /// -- it is the first thing anybody learns in a game with a jump -- and it
    /// is what decides which of the creature's surfaces are a route and which
    /// are a wall. Measured by jumping, in `play`, rather than typed.
    hop: Fx,
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
            answer_left: 0,
            weight_was: Fx::ZERO,
            dodge_left: 0,
            hit_due: i32::MAX,
            slop: 0,
            between: 0,
            rng: 0x9E37_79B9 ^ (who as u32).wrapping_mul(0x85EB_CA6B) | 1,
            leap_left: 0,
            hop: Fx::ZERO,
        }
    }

    /// Start its timing error somewhere else. See `slop`.
    pub fn seeded(mut self, seed: u32) -> Hunter {
        self.rng ^= seed.wrapping_mul(0x27D4_EB2F);
        self.rng |= 1;
        self
    }

    /// Tell it how high it can jump. See `hop`.
    pub fn knowing_hop(mut self, hop: Fx) -> Hunter {
        self.hop = hop;
        self
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
                left: b.doing.frames_left(),
                toppled: matches!(b.doing, Doing::Toppled { .. }),
                open: b.doing.open(),
                alive: b.alive(),
                mount: lowest_mount(b),
                ridge: top_of(&b.rig(), monster::RIDGE),
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

/// Where a foot is, reconstructed the way a player eyeballs it: from where
/// the animal was and which way it was pointed.
///
/// The *rest* position rather than the live one, which is the point -- a player
/// knows roughly where an animal's leg is without tracking its gait, and a bot
/// that read the exact bone would be answering a question nobody can ask.
fn foot_at(seen: Seen, part: usize) -> V3 {
    let shape = monster::shape(part);
    let mut bone = sim::beast::SHAPES[part].bone;
    let mut at = V3::ZERO;
    while bone < sim::beast::BONES {
        at = at.add(sim::beast::rest(bone));
        bone = sim::beast::PARENTS[bone];
    }
    let mid = shape.min.add(shape.max).scale(HALF);
    let along = V3::from_turns(seen.beast_yaw);
    let side = V3::from_turns(seen.beast_yaw.add(QUARTER));
    seen.beast_pos
        .add(along.scale(at.x.add(mid.x)))
        .add(side.scale(at.z.add(mid.z)))
}

/// The foot nearest the hunter. What a person goes for in an opening: not
/// the one the plan says, the one they can reach before it closes.
fn nearest_foot(seen: Seen, from: V3) -> V3 {
    monster::BREAKABLE
        .iter()
        .map(|p| foot_at(seen, *p))
        .min_by_key(|f| {
            V3::new(f.x.sub(from.x), Fx::ZERO, f.z.sub(from.z))
                .flat_len()
                .raw()
        })
        .unwrap_or(seen.beast_pos)
}

/// Where it wants to stand, as a bearing off the creature's nose: about a
/// hundred degrees round, which is outside the cone every forward move needs
/// and inside the reach of its own.
///
/// This is the whole ground strategy, and it is the one the fight is *supposed*
/// to teach: **behind it, beside a back leg.** It is outside the cone every
/// forward move needs *and* outside the tail's, and it is the only station from
/// which a fighter on the floor can reach anything at all -- the creature's
/// barrel is three metres over their head and its feet are the one part of it
/// at ground level.
///
/// **Since the back kick, beside the hind leg rather than behind it.** About
/// a hundred and twelve degrees round: inside the sweep's cone and just
/// outside the kick's. The kick's tell is twenty frames and a poke is
/// twenty, so a hunter standing where the kick is aimed cannot both attack
/// and be sure of answering it -- which is the kick doing its job, and the
/// reason the station moved. From the flank the one threat is the tail, whose
/// tell is long enough to see over a poke. Dead astern used to be outside
/// every cone the creature had, which a player found and named a safe spot.
const FLANK: Fx = Fx::ratio(31, 100);
/// Far enough round that the tail is closer than the head. The climb only
/// starts from here, and it is deliberately just inside the station above: a
/// threshold the hunter's own resting position does not meet is a threshold
/// that fires by accident or not at all.
const BEHIND: Fx = Fx::ratio(22, 100);
/// How far past the poke's own reach a foot's centre may be and still be
/// touched: about a foot's half-width. What "close enough to swing" means,
/// and it is read off the move rather than typed because a poke thrown from
/// a metre past its reach is a poke thrown at air, which is what most of the
/// bot's swings were when this was a round number.
const FOOT_HALF: Fx = Fx::ratio(5, 10);
/// Do not bother correcting for less than this.
const SETTLED: Fx = Fx::ratio(7, 10);
/// Once it decides to climb, it keeps trying for this long.
const CLIMB_COMMIT: u16 = 150;
/// Jump for the surface from here.
const MOUNT_LEAP: Fx = Fx::ratio(24, 10);
/// Frames to hold the jump for, climbing. **All of one.** A route is a
/// surface inside a full hop's apex and often not by much, so anything less
/// bounces off the side of it -- which is the climb being a real jump rather
/// than a step, and is deliberate. See `cargo run -p sim --bin beastcheck`.
const LEAP_HOLD: u16 = 32;
/// How far inside its own apex a surface has to be before it is a route.
/// Landing snaps to a face within `mount_snap` of the feet, so this is a
/// margin against the animal moving, not against the geometry.
const ROUTE_MARGIN: Fx = Fx::ratio(15, 100);
/// Frames to hold the jump for, going over a tail sweep. Enough to put the
/// *feet* well over a hitbox a metre and a half off the floor for the whole
/// of the whip -- the sweep is eight frames wide and the feet are what it
/// measures -- and no more: every frame past that is a frame spent in the
/// air over an animal that has moved on.
const SWEEP_HOP: u16 = 16;
/// Frames to hold the jump for, going over a shake. Long enough to clear the
/// whole whip, which is the only reason jumping one works.
const BUCK_HOP: u16 = 22;
/// How far behind the ridge's middle it stands to hit it: on the barrel,
/// swinging forward at the strip.
const WORK_BACK: Fx = Fx::ratio(6, 10);
/// Stick deadzone when turning a direction into four keys.
const DEAD: Fx = Fx::ratio(3, 10);
/// Frames between attacks in a window it can see is safe -- a recovery with
/// more frames left than a poke and its own reaction put together. Nothing
/// can start inside one, so there is nothing to watch for, and a person
/// unloads.
const SWING_GAP_OPEN: u16 = 8;
/// Frames between attacks, so it does not mash into its own recovery.
///
/// **Wider than the recovery.** A poke is twenty frames of not being able to
/// jump, and the sweep's tell is thirty; a bot that pokes the moment it
/// can is committed for most of every window it has to read the tail in,
/// and got swept about one time in two. Poking, then watching, then poking
/// is what a person does at the feet of something with a tail.
const SWING_GAP: u16 = 28;
/// How long after a blow lands on the shield the Bulwark still answers it
/// with Slam: the blockstun, and a beat.
const ANSWER: u16 = 30;
/// Far enough from a foot to be worth dashing in rather than walking. Under
/// this a walk arrives first: the dodge carries a body about as far as this
/// before its speed decays, and it is twenty-two frames of not being able to
/// swing.
const DASH_FROM: Fx = Fx::ratio(3, 1);

/// Frames of an opening kept back to leave in. A swing is only started
/// with more than its own frames and this left, and a hunter still close
/// when the opening is down to this dodges out.
const EXIT: i32 = 12;
/// Close enough to the creature's centre that what comes out of a recovery
/// reaches: the stomp's volume and a little.
const CLOSE: Fx = Fx::from_raw(15 << 15);

/// How many frames early or late the hunter's dodge or jump can be. A dodge
/// is invulnerable for ten frames and most hits are out for five or six, so
/// being a few frames off either way is the difference between a read and a
/// hit -- which is what it is for a person too.
const SLOP_EARLY: i32 = 4;
const SLOP_LATE: i32 = 3;

/// Frames of dodge-lead: the dodge goes this many frames before the hit
/// arrives, so its invulnerability is running when it does.
const DODGE_LEAD: i32 = 3;
/// Frames of jump-lead over a sweep: long enough for the feet to rise over a
/// tail most of two metres off the floor.
const SWEEP_LEAD: i32 = 9;
/// How far past a move's reach counts as out of it, when running from a tell.
const ESCAPE_MARGIN: Fx = Fx::ratio(15, 10);
/// Where the hunter waits for an opening: from the creature's centre, beside
/// the hind leg, just outside the reach of what it does up close. See the
/// ground plan's third step.
const STANDOFF: Fx = Fx::ratio(85, 10);

/// How far from the creature's centre a move's hit can reach: the volume's
/// own anchor and radius, a body's width, and however far it travels while it
/// is out. **The size of the thing, not the distance it is thrown at.** The
/// hunter used to judge by the second, and a slam chosen for a target at five
/// metres flattened it at eight and a half every time.
fn reach_of(m: &monster::Attack) -> Fx {
    let moving = m.travel.add(m.advance);
    m.hit_x
        .abs()
        .add(m.hit_radius)
        .add(sim::tuning::body_radius())
        .add(moving.mul(Fx::from_int(m.active as i32)).mul(sim::DT))
}

/// Frames until a move's hit reaches somebody at `range`, **in the present**:
/// what the hunter remembers is `REACTION` frames old, so the tell it sees
/// has that much less left than it looks. A move that travels -- the charge,
/// the spray, the bite's lunge -- adds the time its hit takes to cross the gap
/// from where it starts.
fn frames_to_contact(m: &monster::Attack, seen: &Seen, range: Fx) -> i32 {
    let to_active = if seen.doing == STARTUP {
        seen.left as i32 + 1
    } else {
        -((m.active as i32) - seen.left as i32)
    } - REACTION as i32;
    let speed = if m.travel.raw() > 0 {
        m.travel
    } else {
        m.advance
    };
    let fly = if speed.raw() > 0 {
        range
            .sub(m.hit_x.abs())
            .sub(m.hit_radius)
            .max(Fx::ZERO)
            .div(speed.mul(sim::DT))
            .to_int()
    } else {
        0
    };
    to_active + fly
}

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
const PROWL: u32 = 0;
const STARTUP: u32 = 1;
const ACTIVE: u32 = 2;
const STUMBLE: u32 = 7;

impl Hunter {
    /// One frame of decision. `watch` must have been called first.
    pub fn act(&mut self, w: &World) -> Input {
        self.cooldown = self.cooldown.saturating_sub(1);
        self.climb_left = self.climb_left.saturating_sub(1);
        self.dodge_left = self.dodge_left.saturating_sub(1);
        self.answer_left = self.answer_left.saturating_sub(1);
        let me = w.players[self.who];
        let weight = sim::bulwark::weight(&me);
        if weight.raw() > self.weight_was.raw() {
            self.answer_left = ANSWER;
        }
        self.weight_was = weight;
        // A jump it has decided on is held until it happens. The button does
        // nothing while a swing is finishing, and a bot that counted the hold
        // down from the decision rather than from the takeoff spent half of
        // every sweep's hop still on the floor -- which is a person mashing
        // jump inside a recovery and getting the short hop when it comes out.
        if !me.grounded || me.aboard() || me.action.actionable() {
            self.leap_left = self.leap_left.saturating_sub(1);
        }

        let (Some(beast), true) = (w.monster, me.health > 0) else {
            return Input::default();
        };
        let mut seen = self.recall();
        // The beat between moves, read as the opening it is: how much of the
        // pause is left, as a person who has counted it would know.
        if seen.doing == PROWL && seen.alive {
            self.between = self.between.saturating_add(1);
            let beat = sim::tuning::think_frames();
            if self.between < beat {
                seen.open = true;
                seen.left = beat - self.between;
            }
        } else {
            self.between = 0;
        }
        if !seen.alive || !matches!(w.phase, Phase::Fighting) {
            return Input::default();
        }
        // Where its feet are is proprioception, not eyesight, so the ride reads
        // the live mount. Everything about the *creature* comes from the delay
        // line.
        if me.aboard() {
            // A hop decided on the ground is spent once the feet are on the
            // animal: landing on it *was* the hop. Left running, the same
            // held button is the ride's "leave", and the bot stepped straight
            // back off every surface it had just reached.
            if self.intent != Intent::Brace {
                self.leap_left = 0;
            }
            self.ride(&me, seen)
        } else {
            self.ground(&me, &beast, seen)
        }
    }

    fn ground(&mut self, me: &sim::state::Player, beast: &Monster, seen: Seen) -> Input {
        let _ = beast;
        self.hit_due = i32::MAX;
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
        // **Judged where the hunter will be when it lands, not where it is
        // standing now.** The creature aims at a lead point, so a slam begun
        // while the hunter is running in is laid across the spot they are
        // running to; read at the distance on sight, that slam is "out of
        // reach" right up to the frame it flattens them. A person who has
        // eaten one learns to stop running at a rearing animal, and what
        // they are judging is exactly this: their own speed toward it, over
        // the frames the tell has left.
        let to_land = if seen.doing == STARTUP {
            seen.left as i32
        } else {
            0
        };
        let closing = me.vel.dot(toward).max(Fx::ZERO);
        let at_landing = range.sub(closing.mul(Fx::from_int(to_land)).mul(sim::DT));
        let threatened = coming.is_some_and(|m| {
            m.damage > 0 && at_landing.raw() < reach_of(&m).add(ESCAPE_MARGIN).raw()
        });

        // 0. A Bulwark with the shield in hand takes a blockable blow on it
        //    instead: that is what loads the shield, and the loaded Slam is
        //    the punish. Facing the animal, which is where the guard's arc
        //    points. Not the sweep, which comes round from the side the
        //    guard is not covering, and not anything unblockable.
        //
        //    **Only while it can still land.** What it sees is `REACTION`
        //    frames old, so a guard held until it *sees* the move end is a
        //    guard held a quarter of a second into the creature's recovery --
        //    which is the punish, thrown away. It knows the move's frames the
        //    way a player who has seen it twice does, and drops the guard when
        //    the active frames are over in the present rather than in memory.
        let still_live = coming.is_some_and(|m| {
            let to_go = if seen.doing == STARTUP {
                seen.left as i32 + m.active as i32
            } else {
                seen.left as i32
            };
            to_go > REACTION as i32
        });
        let guards = me.shield().is_some_and(|s| s.in_hand())
            && still_live
            && coming.is_some_and(|m| !m.unblockable && seen.kind != monster::SWEEP);
        if threatened && guards {
            self.intent = Intent::Guard;
            return Input::aimed(Input::RIGHT, wire);
        }

        // 1. Get out of the way -- **at the hit, not at the tell.** The
        //    windup follows you now, so walking away from a tell you have
        //    seen is not an answer and a dodge thrown the moment it is seen
        //    is spent before anything arrives. What a person learns is when
        //    the hit lands: the move's startup, plus however long the part
        //    that travels takes to cross the gap. The dodge goes then, and its
        //    invulnerability is what carries them through.
        //
        //    The one exception is a tell long enough to run from. The slam's
        //    is, from the edge of its reach, and running is what a person does
        //    with forty-eight frames and a rearing animal.
        if threatened {
            if let Some(m) = coming {
                // A new move, a new error: drawn on the frame it is seen to
                // begin, and kept for every frame of that throw.
                if seen.doing == STARTUP && seen.left == m.startup {
                    self.rng ^= self.rng << 13;
                    self.rng ^= self.rng >> 17;
                    self.rng ^= self.rng << 5;
                    let span = (SLOP_LATE + SLOP_EARLY + 1) as u32;
                    self.slop = (self.rng % span) as i32 - SLOP_LATE;
                }
                let contact = frames_to_contact(&m, &seen, range) - self.slop;
                let run_room = reach_of(&m).add(ESCAPE_MARGIN).sub(range);
                let can_run = m.travel.raw() == 0
                    && m.advance.raw() == 0
                    && Fx::from_int(contact.max(0))
                        .mul(sim::tuning::move_speed())
                        .mul(sim::DT)
                        .raw()
                        > run_room.raw();
                if seen.kind == monster::SWEEP {
                    // **A held jump, not a tap**, and timed so the feet are
                    // over the tail when it arrives rather than on the way
                    // back down.
                    if contact <= SWEEP_LEAD && self.leap_left == 0 && me.grounded {
                        self.intent = Intent::Evade;
                        self.leap_left = SWEEP_HOP;
                        return Input::aimed(Input::SPACE, wire);
                    }
                    if self.leap_left > 0 {
                        return Input::aimed(Input::SPACE, wire);
                    }
                } else if can_run {
                    self.intent = Intent::Evade;
                    return Input::aimed(steer(aim, toward.scale(Fx::ONE.neg())), wire);
                } else if contact <= DODGE_LEAD && self.dodge_left == 0 {
                    self.intent = Intent::Evade;
                    self.dodge_left = sim::tuning::dodge_frames();
                    let out = V3::from_turns(seen.beast_yaw.add(if bearing.raw() >= 0 {
                        QUARTER
                    } else {
                        QUARTER.neg()
                    }));
                    return Input::aimed(steer(aim, out) | Input::SHIFT, wire);
                }
                // Not yet. A person keeps doing what they were doing until
                // the hit is due; what they do not do is start a swing that
                // will still be going when it arrives. See `hit_due`.
                self.hit_due = contact;
            }
        }

        // 2. Climb, if there is anything to climb. **A route is a surface
        //    inside its own jump**, and which surfaces those are is the whole
        //    ground game: standing, nothing on the animal is -- except for the
        //    floatiest class, whose hop clears the tail -- so the way up is a
        //    shoulder on the floor during a stumble, the barrel of a creature
        //    on its side, or the slam's crash, and every one of those has to be
        //    earned or read. The bot does what the design says a player does:
        //    it works a hind foot until one goes, and goes up when the animal
        //    comes down.
        //
        // The old plan jumped for the tail and nothing else, which was the
        // free route for most of the roster while the tail drooped to within
        // a hop of the floor. With the tail carried level it is a wall, and a
        // bot that kept jumping at it stood under a rearing animal for the
        // rest of every fight.
        let route = seen.mount.top.add(ROUTE_MARGIN).raw() < self.hop.raw();
        let worth_climbing = route
            && (seen.toppled
                || seen.doing == STUMBLE
                || (seen.open && bearing.abs().raw() > BEHIND.raw()))
            && range.raw() < Fx::from_int(12).raw();
        if worth_climbing && self.climb_left == 0 {
            self.climb_left = CLIMB_COMMIT;
        }
        // A window that has closed is not worth running at. A toppled animal
        // stays a route until it is up; a stumble's shoulder comes back up on
        // its own, and the bot sees that happen.
        if self.climb_left > 0 && !route {
            self.climb_left = 0;
        }

        if self.climb_left > 0 {
            self.intent = Intent::Climb;
            let spot = seen.mount.spot;
            let to_spot = V3::new(spot.x.sub(me.pos.x), Fx::ZERO, spot.z.sub(me.pos.z));
            if to_spot.flat_len().raw() < MOUNT_LEAP.raw() && me.grounded && self.leap_left == 0 {
                self.leap_left = LEAP_HOLD;
            }
            let jump = if self.leap_left > 0 { Input::SPACE } else { 0 };
            return Input::aimed(steer(aim, to_spot) | jump, wire);
        }

        // 3. Wait for an opening, and go in only for one long enough.
        //
        //    **Walking up to it is not an approach any more.** It meets a
        //    target closing on it and it chases one leaving, so the ground
        //    game is played from a standoff just outside the reach of what it
        //    does up close, beside the hind leg where the forward moves have
        //    to turn to find you. From there a person goes in on a window
        //    they can see will still be open when they arrive: the recovery
        //    left, less their own reaction, against the ground to cover and
        //    the swing to throw.
        let foot = nearest_foot(seen, me.pos);
        let to_foot = V3::new(foot.x.sub(me.pos.x), Fx::ZERO, foot.z.sub(me.pos.z));
        let at_foot = atan2_turns(to_foot.z, to_foot.x);
        let foot_wire = turns_to_aim(at_foot.sub(me.carry_yaw));
        let foot_range = to_foot.flat_len();
        let poke = sim::moves::get(me.class, sim::state::SLOT_POKE);
        let strike = poke.reach.add(FOOT_HALF);
        // The recovery left, and the beat it takes before its next move --
        // a rhythm a person has learned by the third time they have seen it
        // -- less their own reaction.
        let pause = if seen.doing == PROWL {
            0
        } else {
            sim::tuning::think_frames() as i32
        };
        let window = seen.left as i32 + pause - REACTION as i32;
        let gap = foot_range.sub(strike).max(Fx::ZERO);
        let walk_frames = gap.div(sim::tuning::move_speed().mul(sim::DT)).to_int();
        let needed = walk_frames + (poke.startup + poke.active) as i32;
        let going_in = seen.open && window > needed + EXIT;
        self.intent = if going_in {
            Intent::Punish
        } else {
            Intent::Circle
        };
        let station = if going_in {
            foot
        } else {
            seen.beast_pos
                .add(V3::from_turns(seen.beast_yaw.add(this_side)).scale(STANDOFF))
        };
        let to_station = V3::new(station.x.sub(me.pos.x), Fx::ZERO, station.z.sub(me.pos.z));
        let far = to_station.flat_len();
        let walk = if going_in && foot_range.raw() < strike.raw() {
            0
        } else if far.raw() > SETTLED.raw() {
            steer(aim, to_station)
        } else {
            0
        };
        // A dash to get in, when the window is long enough to be worth it
        // and not otherwise: the dodge is the fastest thing a fighter has,
        // and the recovery after it is the frames the next move lands in.
        let dash = if going_in
            && self.hit_due > sim::tuning::dodge_frames() as i32 + DODGE_LEAD
            && gap.raw() > DASH_FROM.raw()
            && self.dodge_left == 0
            && window > needed + sim::tuning::dodge_frames() as i32 / 2
        {
            self.dodge_left = sim::tuning::dodge_frames();
            Input::SHIFT
        } else {
            0
        };
        let wire = foot_wire;
        let range = foot_range;
        let poke_busy = (poke.startup + poke.active + poke.recovery) as i32;
        // **Out before it closes.** The stomp is the first thing out of a
        // recovery and it is faster than anybody reacts, so a punish that
        // overstays is a punish that pays for itself with a stomp. A person
        // learns to count the recovery down, swing while there is time to
        // leave after, and be gone by the end of it. The dodge is how they
        // leave, and it goes out the side the station is on.
        let close = to_beast.flat_len().raw() < CLOSE.raw();
        if close && seen.open && window <= EXIT && self.dodge_left == 0 && me.action.actionable() {
            self.intent = Intent::Evade;
            self.dodge_left = sim::tuning::dodge_frames();
            let out = V3::from_turns(seen.beast_yaw.add(this_side));
            return Input::aimed(steer(aim, out) | Input::SHIFT, wire);
        }
        let swing = if self.cooldown == 0
            && me.action.actionable()
            && range.raw() < strike.raw()
            && seen.open
            && window > (poke.startup + poke.active) as i32 + EXIT
            && self.hit_due > poke_busy + DODGE_LEAD
        {
            self.cooldown = if window > poke_busy {
                SWING_GAP_OPEN
            } else {
                SWING_GAP
            };
            let fits = window as usize > heavy_commitment(me.class);
            if fits { heavy(me.class) } else { Input::LEFT }
        } else {
            0
        };
        // **A blow just taken on the shield is answered with Slam**, on the
        // first free frame after the blockstun: what the blow put into the
        // shield comes straight back out at the foot.
        if self.answer_left > 0
            && me.action.actionable()
            && sim::bulwark::weight(me).raw() > 0
            && range.raw() < strike.add(FOOT_HALF).raw()
        {
            self.answer_left = 0;
            self.cooldown = SWING_GAP;
            self.intent = Intent::Punish;
            return Input::aimed(heavy(me.class), wire);
        }
        // **A Bulwark at the foot keeps the shield up between openings.** It
        // is the class's plan, not a reaction: the stomp is faster than anybody
        // reacts, so a guard raised on sight never meets it, and what loads
        // the shield is standing where the blows land with the guard already
        // there. Down for the swing, up again after -- the loaded Slam is then
        // the swing the next opening gets.
        let stance = me.shield().is_some_and(|s| s.in_hand())
            && !seen.open
            && swing == 0
            && range.raw() < strike.add(FOOT_HALF).raw();
        if stance {
            self.intent = Intent::Guard;
            return Input::aimed(walk | Input::RIGHT, wire);
        }
        // A hop in progress keeps its button down -- height is what it is for --
        // without that costing the swing it was going to throw.
        let hop = if self.leap_left > 0 { Input::SPACE } else { 0 };
        Input::aimed(walk | dash | swing | hop, wire)
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

        // Already committed to a hop. Holding the button is what makes it a
        // full one, and a full one is what clears a whip.
        if self.leap_left > 0 {
            self.intent = Intent::Brace;
            return Input::aimed(Input::SPACE, wire);
        }

        let coming = (seen.doing == STARTUP)
            .then_some(seen.kind)
            .filter(|k| *k != monster::NO_PART);
        if let Some(kind) = coming.filter(|k| bucks(*k)) {
            self.intent = Intent::Brace;
            match kind {
                // Nothing holds through that. Leave, and take the creature's
                // own speed with you.
                monster::SLAM => return Input::aimed(Input::SPACE, wire),
                // **Jump it.** A shake throws a loose rider and a brace holds,
                // so bracing is the safe answer -- but it costs the attack you
                // were about to throw, and the shake runs for a second. Leaving
                // the ground over the whole whip costs nothing and lands you
                // back where you were, with the animal unable to shake again
                // for nearly four seconds. It has to be committed to before the
                // whip starts, which a forty-frame windup and a fifteen-frame
                // reaction leave room for and not much more.
                monster::SHAKE => {
                    self.leap_left = BUCK_HOP;
                    return Input::aimed(Input::SPACE, wire);
                }
                _ => return Input::aimed(Input::CROUCH, wire),
            }
        }

        // To the ridge, wherever it is from here. The old version walked to a
        // spot in the barrel's own frame, which is forward from the tail and
        // *also* forward from the shoulders -- so a rider who had come up a
        // stumbling shoulder walked off the animal's front.
        let to_ridge = V3::new(
            seen.ridge.x.sub(me.pos.x),
            Fx::ZERO,
            seen.ridge.z.sub(me.pos.z),
        );
        let along_it = to_ridge.dot(along);
        let across_it = to_ridge.dot(side);
        if along_it.sub(WORK_BACK).abs().raw() > Fx::ratio(4, 10).raw()
            || across_it.abs().raw() > Fx::ratio(5, 10).raw()
        {
            self.intent = Intent::ToRidge;
            let want = along
                .scale(along_it.sub(WORK_BACK))
                .add(side.scale(across_it));
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

/// Which button throws the class's committed attack.
///
/// Everywhere but the Champion it is shift plus left click, per `controls.md`.
/// The Champion's three mouse buttons are three weapons instead, and the
/// committed one is the hammer on middle click -- so this hunter punishes an
/// opening with a hammer, and never with the class's *biggest* hits, which all
/// live behind a Rush charge it deliberately does not spend. It is here to
/// measure the creature, not to show off a kit.
///
/// The Bulwark's is on middle click too, since 2026-09-23: Slam, which spends
/// the shield's weight, so a hunter that has blocked a stomp punishes with
/// what the stomp put into it. The other classes' committed moves have had no
/// input since shift stopped modifying clicks, and for them this is the poke.
fn heavy(class: sim::Class) -> u16 {
    match class {
        sim::Class::Champion | sim::Class::Bulwark => Input::MIDDLE,
        // The Haemorrhage, on right click: a bolt that cuts and gores, and
        // the one right-click ability in the roster since shift stopped
        // modifying clicks. The creature does not bleed, so on a hunt it is
        // the ranged punish and nothing more.
        sim::Class::BloodMage => Input::RIGHT,
        _ => Input::LEFT | Input::SHIFT,
    }
}

/// How long the committed attack keeps the hunter busy: the frames until it is
/// free to move again. What an opening has to be longer than to be worth it.
fn heavy_commitment(class: sim::Class) -> usize {
    let m = sim::moves::get(class, sim::state::SLOT_COMMITTED);
    (m.startup + m.active + m.recovery) as usize
}

/// Apex of a full hop above the feet, for one class.
///
/// Measured by running the simulation, the way `beastcheck` does, because the
/// sustain window makes the closed form wrong -- and the number is what
/// decides which of the creature's surfaces the hunter treats as a route.
fn jump_apex(class: sim::Class) -> Fx {
    let mut w = World::with_classes([class; MAX_PLAYERS]);
    for _ in 0..40 {
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    let floor = w.players[0].pos.y;
    let mut best = floor;
    let held = Input::default().with(Input::SPACE);
    for _ in 0..240 {
        w.advance([held, Input::default()]);
        best = best.max(w.players[0].pos.y);
    }
    best.sub(floor)
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
    let mut bots: Vec<Hunter> = (0..count)
        .map(|who| {
            Hunter::new(who)
                .seeded(seed)
                .knowing_hop(jump_apex(classes[who]))
        })
        .collect();
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
