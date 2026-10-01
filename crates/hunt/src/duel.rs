//! A sparring partner: a bot that fights a **fighter** rather than the
//! creature.
//!
//! The [`crate::Hunter`] next door plays the hunt, and it is a measuring
//! instrument. This one is an opponent, and the difference decides almost
//! everything about it. An instrument should be the same every time; an
//! opponent that is the same every time is solved in three rounds and then it
//! is furniture. So this bot is built on three rules, and each one is somewhere
//! you can point at.
//!
//! **It sees late, and not always the same amount late.** What it knows about
//! the other fighter comes out of a delay line, like the hunter's, but the
//! delay wanders between two bounds from frame to frame -- attention, not a
//! constant. A move whose startup is shorter than its delay cannot be answered
//! on sight, and it will not be. It also sometimes does not notice at all.
//!
//! **Its hands are not perfect.** The mouse turns at a finite rate, every shot
//! is thrown with an aim error drawn fresh for that shot, and every timed press
//! -- a dodge, a jump, a punish -- lands a few frames early or late.
//!
//! **It decides like a person, which is to say not the same way twice.** It
//! commits to a plan for a while -- press forward, dance at the edge of your
//! reach, circle, back off and zone, dash in, jump in, bait -- and chooses the
//! next one by weighted chance rather than by rule. The weights come from the
//! situation (distance, who is winning, what you have been doing), from a
//! **personality** drawn from the seed, and from a penalty on whatever it did
//! recently. Its mood moves with the fight: hit it and it gets careful, or --
//! if it is the aggressive sort -- angry.
//!
//! What it does **not** get to do is read the simulation's mind. It knows the
//! roster the way a player who has played knows it: which moves your class
//! has, how far they reach and how long they take once it has seen them start.
//! It never sees an input, and it never looks at the present.
//!
//! Integer and fixed point throughout, like the rest of this crate: a bot
//! fight from one seed is the same fight every time, which is what lets a test
//! or the `duel` binary say anything about it.

mod mechanics;

use mechanics::{ClassOut, Cues};
use sim::aim::{self, Kind};
use sim::bolt::Flight;
use sim::fixed::Fx;
use sim::math::{atan2_turns, wrap_turns};
use sim::moves;
use sim::state::{self, Action, MAX_PLAYERS, Phase, Player};
use sim::{Class, Input, V3, World};

/// How good its eyes and hands are.
///
/// Three presets, and every field is public so a harness can make a fourth.
/// What the fields do **not** change is how it thinks: an easy bot and a hard
/// one choose plans the same way, and differ in how well they see and carry
/// them out. That is the difference between a beginner and an expert, too.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Skill {
    /// The shortest and longest its perception runs behind the present, in
    /// frames. It wanders between the two.
    pub react_min: u16,
    pub react_max: u16,
    /// Percent chance it notices a move starting at all.
    pub notice: u32,
    /// Percent chance it takes an opening it has seen and can reach.
    pub punish: u32,
    /// The largest aim error on a shot, in turns.
    pub aim_error: Fx,
    /// How fast its mouse can turn, in turns per frame.
    pub turn: Fx,
    /// Frames between decisions, least and most.
    pub think_min: u16,
    pub think_max: u16,
    /// Frames early or late a timed press can land, either way.
    pub slop: i32,
    /// Percent of shots it leads a moving target with.
    pub lead: u32,
}

/// The three presets, by name. See [`Skill`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Easy,
    Normal,
    Hard,
}

impl Level {
    pub const ALL: [Level; 3] = [Level::Easy, Level::Normal, Level::Hard];

    pub const fn name(self) -> &'static str {
        match self {
            Level::Easy => "easy",
            Level::Normal => "normal",
            Level::Hard => "hard",
        }
    }

    pub fn skill(self) -> Skill {
        match self {
            // Someone who has not played a fighting game: sees things half a
            // second late, misses a good share of them, and swings at where
            // you were.
            Level::Easy => Skill {
                react_min: 24,
                react_max: 36,
                notice: 45,
                punish: 30,
                aim_error: Fx::ratio(3, 100),
                turn: Fx::ratio(2, 100),
                think_min: 10,
                think_max: 22,
                slop: 6,
                lead: 15,
            },
            // A regular. Reacts in about a third of a second.
            Level::Normal => Skill {
                react_min: 18,
                react_max: 27,
                notice: 70,
                punish: 60,
                aim_error: Fx::ratio(15, 1000),
                turn: Fx::ratio(35, 1000),
                think_min: 6,
                think_max: 13,
                slop: 4,
                lead: 50,
            },
            // A good player. **Not below fifteen frames**: that is about as
            // fast as a person reacts to something they are waiting for, and
            // a bot quicker than that would be answering moves nobody can.
            Level::Hard => Skill {
                react_min: 15,
                react_max: 21,
                notice: 88,
                punish: 85,
                aim_error: Fx::ratio(7, 1000),
                turn: Fx::ratio(6, 100),
                think_min: 3,
                think_max: 8,
                slop: 2,
                lead: 85,
            },
        }
    }
}

/// Who it is, as opposed to how good it is. Each is a percentage, drawn from
/// the seed, and it drifts a little every round -- nobody plays the third round
/// the way they played the first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    /// Walks in and swings, rather than waiting.
    pub aggression: i32,
    /// Waits for you to commit, and punishes it.
    pub patience: i32,
    /// Leaves early, dodges more, keeps its distance when hurt.
    pub caution: i32,
    /// Jumps: over things, and at you.
    pub air: i32,
    /// Throws things from range, if it has anything to throw.
    pub zoning: i32,
}

/// How long the delay line is. Longer than any preset's slowest reaction.
const MEMORY: usize = 40;
/// The five buttons that can throw a move, in the order a kit is probed.
const BUTTONS: [u16; 5] = [
    Input::LEFT,
    Input::RIGHT,
    Input::MIDDLE,
    Input::SPECIAL,
    Input::MECHANIC,
];
/// A quarter turn, as `Fx` turns.
const QUARTER: Fx = Fx::from_raw(1 << 14);
/// Stick deadzone when turning a direction into four keys.
const DEAD: Fx = Fx::ratio(3, 10);
/// Frames a click is held for. Long enough to be a press on any frame the
/// fighter comes free in; short enough not to throw it twice.
const PRESS: u16 = 3;
/// Frames the dodge key is held.
const DODGE_HOLD: u16 = 3;
/// Frames of dodge-lead: it goes this far ahead of the hit so that its
/// invulnerability is running when the hit arrives.
const DODGE_LEAD: i32 = 3;
/// How close its yaw must be to where it wants to throw before it will, in
/// turns. About eleven degrees.
const ON_TARGET: Fx = Fx::ratio(3, 100);
/// Frames it will wait for its mouse to come round before throwing anyway.
const TURN_PATIENCE: u16 = 8;
/// Frames an attempt to attack survives while the fighter is still busy.
const ATTEMPT_LIFE: u16 = 24;
/// The widest a swing can be aimed off the victim's line and still be a
/// threat, in turns: about sixty degrees either side.
const FACING_CONE: Fx = Fx::ratio(17, 100);
/// Extra range it treats as dangerous, past a move's own reach, for the step
/// the attacker takes and the width of a body.
const THREAT_MARGIN: Fx = Fx::ratio(12, 10);
/// A bolt passing this close, side to side, is one to get out of the way of.
const BOLT_MISS: Fx = Fx::ratio(12, 10);
/// How close to the arena's edge before backing off turns into circling.
const EDGE: Fx = Fx::from_int(2);
/// How far inside the walls it keeps its jumps.
const JUMP_CLEAR: Fx = Fx::from_int(4);
/// How close to a wall it jumps from, to get back over it.
/// Measured from the inside face; the wall is a metre thick, so from outside
/// this is under a metre and a half from it.
const HOP_THE_WALL: Fx = Fx::ratio(25, 10);
/// Frames of walking without getting anywhere before it jumps whatever it is
/// walking into -- a platform's side, usually.
const STUCK: u16 = 12;
/// Slower than this, walking, is not getting anywhere.
const CRAWL: Fx = Fx::ratio(5, 10);
/// The number of plans it remembers, to avoid doing one of them again.
const RECENT: usize = 4;

/// One of its moves: the button that throws it and which move that is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tool {
    pub bits: u16,
    pub kind: u8,
}

/// Every move a class throws from the floor, by the button that throws it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kit {
    pub class: Class,
    pub tools: [Option<Tool>; BUTTONS.len()],
}

impl Kit {
    /// **Learned, not typed.** Which button throws which move depends on the
    /// class, on the mechanic, and on grammar that has moved three times this
    /// month (see `docs/design/controls.md`). Rather than keep a second copy
    /// of it here that would be wrong the next time it moves, the bot does
    /// what a player does on picking a class up: it presses each button in an
    /// empty arena and watches what comes out.
    pub fn learn(class: Class) -> Kit {
        let mut tools = [None; BUTTONS.len()];
        for (slot, &bits) in BUTTONS.iter().enumerate() {
            let Some(kind) = probe(class, bits) else {
                continue;
            };
            if tools.iter().flatten().any(|t: &Tool| t.kind == kind) {
                continue;
            }
            tools[slot] = Some(Tool { bits, kind });
        }
        Kit { class, tools }
    }

    pub fn iter(&self) -> impl Iterator<Item = Tool> + '_ {
        self.tools.iter().flatten().copied()
    }

    /// The furthest any of its moves reaches, and how soon it gets there.
    /// What a player means by "their range": the distance to respect.
    fn threat(&self) -> Fx {
        self.iter()
            .map(|t| reach(&moves::get(self.class, t.kind), None))
            .filter(|r| r.raw() < Fx::from_int(9).raw())
            .max_by_key(|r| r.raw())
            .unwrap_or(Fx::from_int(3))
    }

    fn ranged(&self) -> Option<Tool> {
        self.iter()
            .filter(|t| moves::get(self.class, t.kind).aim().is_a_skillshot())
            .max_by_key(|t| moves::get(self.class, t.kind).reach.raw())
    }
}

/// Press one button on a fresh fighter and see which move starts.
fn probe(class: Class, bits: u16) -> Option<u8> {
    let mut w = World::with_classes([class; MAX_PLAYERS]);
    for frame in 0..4 {
        let mut wire = [Input::default(); MAX_PLAYERS];
        if frame == 1 {
            wire[0] = Input::new(bits);
        }
        w.advance(wire);
        if let Some(kind) = w.players[0].action.attack_kind() {
            return Some(kind);
        }
    }
    None
}

/// How far a move reaches from the body throwing it, to the middle of a body
/// it would hit: the move's own length, the step it carries you, and the width
/// of whoever is on the end of it.
fn reach(m: &sim::Move, who: Option<&Player>) -> Fx {
    let own = match who {
        Some(p) => state::live_reach(p, m),
        None => m.reach,
    };
    match m.aim() {
        Kind::Swing => own.add(m.step).add(sim::tuning::body_radius()),
        // The crack runs from a stone the bot raised rather than from the
        // body, and the bot does not track where that is: its own length is
        // the honest guess.
        Kind::Grounded | Kind::Skillshot | Kind::Racing => own,
        // Pointed by where the mechanic stands, which the bot does not track.
        Kind::AtTheMechanic => Fx::ZERO,
    }
}

/// What it has decided to do for a while.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plan {
    /// Walk in and swing.
    Press,
    /// Stand at the edge of your reach, stepping in and out of it, waiting
    /// for you to throw something at air.
    Footsies,
    /// Strafe round you.
    Circle,
    /// Back off.
    Retreat,
    /// Keep a distance and throw things across it.
    Zone,
    /// Dodge in and swing out of it.
    DashIn,
    /// Jump at you and swing on the way down.
    JumpIn,
    /// Walk into your range and straight back out of it.
    Bait,
    /// Stand and watch.
    Wait,
}

const PLANS: [Plan; 9] = [
    Plan::Press,
    Plan::Footsies,
    Plan::Circle,
    Plan::Retreat,
    Plan::Zone,
    Plan::DashIn,
    Plan::JumpIn,
    Plan::Bait,
    Plan::Wait,
];

/// A timed answer to something it has seen coming.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Answer {
    /// Dodge this way, as a world direction.
    Dodge(V3),
    Crouch,
    Guard,
    Jump,
    /// Walk this way until it has passed.
    Back(V3),
    /// Swing first.
    Counter(Tool),
}

#[derive(Clone, Copy, Debug)]
struct Pending {
    answer: Answer,
    /// Frames until it starts.
    start_in: i32,
    /// Frames it lasts once it has.
    hold: u16,
}

/// A swing it has decided to throw and has not thrown yet.
#[derive(Clone, Copy, Debug)]
struct Attempt {
    tool: Tool,
    /// Frames spent waiting for the mouse to come round.
    turning: u16,
    /// Frames of the press left, once it has begun.
    pressing: u16,
    /// Frames it has been waiting for the fighter to be free.
    age: u16,
    /// Aim error for this shot, in turns.
    error: Fx,
    /// Lead this shot, or not.
    lead: bool,
}

/// An aimed press of something that is not an attack -- `E`, the shadow's
/// right click, the lotus, a blink -- made by the same hands as an attack: the
/// mouse comes round to `at` first, at the same speed, off by the same error.
#[derive(Clone, Copy, Debug)]
struct Gesture {
    bits: u16,
    /// Where the crosshair goes.
    at: V3,
    /// Where to walk while it is pressed, for anything that reads the stick:
    /// the Rush's direction, a dodge's.
    walk: Option<V3>,
    /// Wait for the fighter to be free before pressing.
    free: bool,
    /// Keep the look level rather than on `at`. For a press that does not
    /// care where the crosshair is and a button that does: the Champion's
    /// spear thrown looking at the floor out of a Rush is the Pole vault,
    /// which clears the arena's walls.
    level: bool,
    turning: u16,
    pressing: u16,
    age: u16,
    error: Fx,
    then: After,
}

/// What follows a gesture once it is pressed.
#[derive(Clone, Copy, Debug)]
enum After {
    Nothing,
    /// Swing this, this many frames later.
    Swing(Tool, u16),
    /// Hold the jump this long -- riding a stone up.
    Jump(u16),
}

impl Gesture {
    fn press(bits: u16, at: V3) -> Gesture {
        Gesture {
            bits,
            at,
            walk: None,
            free: false,
            level: false,
            turning: 0,
            pressing: 0,
            age: 0,
            error: Fx::ZERO,
            then: After::Nothing,
        }
    }

    fn walking(self, dir: V3) -> Gesture {
        Gesture {
            walk: Some(dir),
            ..self
        }
    }

    fn level(self) -> Gesture {
        Gesture {
            level: true,
            ..self
        }
    }

    fn free(self) -> Gesture {
        Gesture { free: true, ..self }
    }

    fn then(self, then: After) -> Gesture {
        Gesture { then, ..self }
    }
}

/// What it saw, one frame of it.
#[derive(Clone, Copy)]
struct Seen {
    /// The world's frame this was.
    frame: u32,
    them: Player,
    bolts: Flight,
    fighting: bool,
}

/// One bot, driving one fighter against the other.
pub struct Duelist {
    who: usize,
    pub skill: Skill,
    level: Level,
    /// Who it is this round. See [`Style`].
    pub style: Style,
    base: Style,
    rng: u32,
    mine: Kit,
    theirs: Kit,
    memory: [Seen; MEMORY],
    at: usize,
    filled: usize,
    /// How far behind the present it is seeing, right now.
    lag: u16,
    /// What it saw last frame, to tell a move starting from one going on.
    last_seen: Action,
    pub plan: Plan,
    plan_left: u16,
    recent: [Plan; RECENT],
    think_in: u16,
    /// Which way it strafes, and how long until it thinks about changing.
    strafe: i32,
    strafe_left: u16,
    /// Whether a footsie dance is stepping in or out.
    dance_in: bool,
    dance_left: u16,
    pending: Option<Pending>,
    /// The answer being held, and for how much longer.
    holding: Option<(Answer, u16)>,
    attempt: Option<Attempt>,
    /// A string press queued behind a swing already thrown.
    string_in: Option<(u16, Tool)>,
    jump_left: u16,
    dodge_left: u16,
    dodge_dir: V3,
    /// Its mouse, in turns.
    yaw: Fx,
    /// Negative is rattled, positive is confident.
    pub mood: i32,
    health_was: [i32; MAX_PLAYERS],
    /// Bolts it has already decided about, by slot.
    bolts_seen: u8,
    rounds_seen: u32,
    frame_was: u32,
    /// How many moves it has seen you start lately, decaying. A player who
    /// throws a lot is one to bait; one who throws nothing is one to press.
    their_tempo: i32,
    tempo_clock: u16,
    /// Did it bait already this plan, and is it retreating from one.
    bait_out: u16,
    dashed: bool,
    /// A press of the class's own, being aimed or made.
    gesture: Option<Gesture>,
    /// Frames before the class's play starts another gesture.
    mech_cool: u16,
    /// How many times it has used its mechanic. For the harness.
    pub mechanic_uses: u32,
    cues: Cues,
    /// The Champion holding the next weapon through a recovery: which
    /// button, for how much longer, and the move it is waiting out.
    chain_hold: Option<(u16, u16, u8)>,
    /// Frames the Bulwark's shield has been out of his hand.
    shield_out: u16,
    /// Frames the Reaver's shadow has been out, and a recall on a timer.
    shadow_out: u16,
    recall_in: Option<u16>,
    /// The Dual mage's second jump: a released frame, then a press.
    second_jump: u8,
    /// Buttons it held last frame, so a press that needs an edge gets one.
    last_bits: u16,
    /// Frames it has been walking into something.
    stuck: u16,
    /// The Dual mage's gap between her bars last frame, to count a mend.
    gap_was: Fx,
}

impl Duelist {
    pub fn new(who: usize, level: Level, seed: u32) -> Duelist {
        let mut rng = 0x9E37_79B9 ^ seed.wrapping_mul(0x85EB_CA6B) ^ (who as u32) << 7 | 1;
        let mut draw = || {
            rng = xorshift(rng);
            20 + (rng % 61) as i32
        };
        let base = Style {
            aggression: draw(),
            patience: draw(),
            caution: draw(),
            air: draw(),
            zoning: draw(),
        };
        let skill = level.skill();
        Duelist {
            who,
            skill,
            level,
            style: base,
            base,
            rng,
            mine: Kit::learn(Class::Champion),
            theirs: Kit::learn(Class::Champion),
            memory: [Seen {
                frame: 0,
                them: Player::default(),
                bolts: [None; sim::bolt::MAX_BOLTS],
                fighting: false,
            }; MEMORY],
            at: 0,
            filled: 0,
            lag: (skill.react_min + skill.react_max) / 2,
            last_seen: Action::Free,
            plan: Plan::Wait,
            plan_left: 0,
            recent: [Plan::Wait; RECENT],
            think_in: 0,
            strafe: 1,
            strafe_left: 0,
            dance_in: true,
            dance_left: 0,
            pending: None,
            holding: None,
            attempt: None,
            string_in: None,
            jump_left: 0,
            dodge_left: 0,
            dodge_dir: V3::ZERO,
            yaw: Fx::ZERO,
            mood: 0,
            health_was: [0; MAX_PLAYERS],
            bolts_seen: 0,
            rounds_seen: u32::MAX,
            frame_was: u32::MAX,
            their_tempo: 0,
            tempo_clock: 0,
            bait_out: 0,
            dashed: false,
            gesture: None,
            mech_cool: 0,
            mechanic_uses: 0,
            cues: Cues::default(),
            chain_hold: None,
            shield_out: 0,
            shadow_out: 0,
            recall_in: None,
            second_jump: 0,
            last_bits: 0,
            stuck: 0,
            gap_was: Fx::ZERO,
        }
    }

    /// Replace the personality the seed drew. For tests and harnesses that
    /// want to hold one still.
    pub fn with_style(mut self, style: Style) -> Duelist {
        self.style = style;
        self.base = style;
        self
    }

    /// Which fighter it drives.
    pub fn who(&self) -> usize {
        self.who
    }

    /// The frame of the world it is acting on, if it is seeing anything yet.
    /// Never the present: that is the promise, and `tests/duel.rs` holds it.
    pub fn seeing(&self) -> Option<u32> {
        let seen = self.recall();
        seen.fighting.then_some(seen.frame)
    }

    /// Which preset it was built at.
    pub fn level(&self) -> Level {
        self.level
    }

    fn roll(&mut self, below: u32) -> u32 {
        self.rng = xorshift(self.rng);
        if below == 0 { 0 } else { self.rng % below }
    }

    fn chance(&mut self, percent: i32) -> bool {
        (self.roll(100) as i32) < percent
    }

    /// A number in `lo..=hi`.
    fn between(&mut self, lo: i32, hi: i32) -> i32 {
        lo + self.roll((hi - lo + 1).max(1) as u32) as i32
    }

    /// Record this frame. Call before [`Duelist::act`], every tick.
    ///
    /// A world whose clock went backwards is a different fight -- a reset, a
    /// class change, a step back through the rewind -- and what it remembers
    /// of the old one is thrown away rather than acted on.
    pub fn watch(&mut self, w: &World) {
        if w.frame <= self.frame_was {
            self.filled = 0;
            self.rounds_seen = u32::MAX;
            self.health_was = [0; MAX_PLAYERS];
        }
        self.frame_was = w.frame;
        let them = w.players[1 - self.who];
        let mut bolts = w.bolts;
        for b in bolts.iter_mut() {
            if b.is_some_and(|b| b.owner as usize == self.who) {
                *b = None;
            }
        }
        self.memory[self.at] = Seen {
            frame: w.frame,
            them,
            bolts,
            fighting: matches!(w.phase, Phase::Fighting),
        };
        self.at = (self.at + 1) % MEMORY;
        self.filled = (self.filled + 1).min(MEMORY);
    }

    /// What it saw `lag` frames ago. Until it has been watching that long it
    /// has seen nothing yet -- not the present, which is what "the oldest it
    /// has" would be on the first frame of a round.
    fn recall(&self) -> Seen {
        let back = self.lag as usize;
        if back >= self.filled {
            return Seen {
                fighting: false,
                ..self.memory[(self.at + MEMORY - 1) % MEMORY]
            };
        }
        self.memory[(self.at + MEMORY - 1 - back) % MEMORY]
    }

    /// One frame of play. [`Duelist::watch`] must have been called first.
    pub fn act(&mut self, w: &World) -> Input {
        let me = w.players[self.who];
        if me.class != self.mine.class {
            self.mine = Kit::learn(me.class);
        }
        let them_class = w.players[1 - self.who].class;
        if them_class != self.theirs.class {
            self.theirs = Kit::learn(them_class);
        }
        if !matches!(w.phase, Phase::Fighting) || me.health <= 0 {
            self.stand_down();
            return Input::aimed(0, turns_to_aim(self.yaw.sub(me.carry_yaw)));
        }
        let round = w.players.iter().map(|p| p.rounds_won as u32).sum::<u32>();
        if round != self.rounds_seen {
            self.new_round(round, &me);
        }

        self.wander_lag();
        let seen = self.recall();
        if !seen.fighting {
            // Its eyes are still on the last round.
            return Input::aimed(0, turns_to_aim(self.yaw.sub(me.carry_yaw)));
        }
        self.feel(w);
        self.read_cues(&me, &seen, w);
        self.notice(&me, &seen);
        self.dodge_bolts(&me, &seen);

        self.think_in = self.think_in.saturating_sub(1);
        if self.think_in == 0 {
            self.think(&me, &seen);
            let (lo, hi) = (self.skill.think_min as i32, self.skill.think_max as i32);
            self.think_in = self.between(lo, hi) as u16;
        }
        let class = self.class_play(&me, &seen, w);
        let input = self.steer_plan(&me, &seen, class);
        self.last_bits = input.bits;
        input
    }

    fn stand_down(&mut self) {
        self.pending = None;
        self.holding = None;
        self.attempt = None;
        self.string_in = None;
        self.jump_left = 0;
        self.dodge_left = 0;
        self.gesture = None;
        self.chain_hold = None;
        self.recall_in = None;
        self.second_jump = 0;
    }

    fn new_round(&mut self, round: u32, me: &Player) {
        self.rounds_seen = round;
        self.filled = 0;
        self.stand_down();
        self.plan_left = 0;
        self.yaw = atan2_turns(me.facing.z, me.facing.x);
        // Nobody plays the next round quite like the last.
        let base = self.base;
        let drift = |v: i32, d: &mut Self| (v + d.between(-15, 15)).clamp(5, 95);
        self.style = Style {
            aggression: drift(base.aggression, self),
            patience: drift(base.patience, self),
            caution: drift(base.caution, self),
            air: drift(base.air, self),
            zoning: drift(base.zoning, self),
        };
        self.mood = 0;
    }

    /// Attention wanders: the delay moves a frame at a time between its
    /// bounds, so it is quick for a while and then slow for a while.
    fn wander_lag(&mut self) {
        if self.chance(15) {
            let step = if self.chance(50) { 1 } else { -1 };
            self.lag = (self.lag as i32 + step)
                .clamp(self.skill.react_min as i32, self.skill.react_max as i32)
                as u16;
        }
    }

    /// Health changing is felt, not seen: a hit you take you know about at
    /// once. It moves the mood, which moves the plans.
    fn feel(&mut self, w: &World) {
        let now = [w.players[0].health, w.players[1].health];
        if self.health_was[self.who] != 0 {
            let lost_mine = self.health_was[self.who] - now[self.who];
            let lost_theirs = self.health_was[1 - self.who] - now[1 - self.who];
            if lost_mine > 0 {
                // The aggressive sort gets angry; the rest get careful.
                let tilt = if self.style.aggression > 60 { 4 } else { -8 };
                self.mood = (self.mood + tilt).clamp(-40, 40);
                self.plan_left = self.plan_left.min(6);
            }
            if lost_theirs > 0 {
                self.mood = (self.mood + 6).clamp(-40, 40);
            }
        }
        self.health_was = now;
        self.tempo_clock += 1;
        if self.tempo_clock >= 60 {
            self.tempo_clock = 0;
            self.their_tempo -= self.their_tempo.signum();
            self.mood -= self.mood.signum();
        }
    }

    /// See a move start, maybe, and decide what to do about it.
    fn notice(&mut self, me: &Player, seen: &Seen) {
        let now = seen.them.action;
        let started = match now {
            Action::Startup { kind, .. } | Action::Channel { kind, .. } => {
                self.last_seen.attack_kind() != Some(kind)
                    || !matches!(
                        self.last_seen,
                        Action::Startup { .. } | Action::Channel { .. }
                    )
            }
            _ => false,
        };
        self.last_seen = now;
        if !started {
            return;
        }
        self.their_tempo = (self.their_tempo + 1).min(12);
        let notice = self.skill.notice as i32;
        if !self.chance(notice) {
            return;
        }
        let kind = now.attack_kind().unwrap_or(0);
        let m = moves::get(seen.them.class, kind);
        let to_me = flat(me.pos.sub(seen.them.pos));
        let dist = to_me.flat_len();
        let range = reach(&m, None).add(THREAT_MARGIN);
        if dist.raw() > range.raw() || m.aim() == Kind::AtTheMechanic {
            return;
        }
        if m.aim() == Kind::Swing {
            let bearing = atan2_turns(to_me.z, to_me.x);
            let facing = atan2_turns(seen.them.facing.z, seen.them.facing.x);
            if wrap_turns(bearing.sub(facing)).abs().raw() > FACING_CONE.raw() {
                return;
            }
        }
        // Frames until it is out, in the present rather than in what it saw.
        let startup_left = match now {
            Action::Startup { left, .. } => left as i32,
            _ => m.startup as i32,
        };
        let due = startup_left + 1 - self.lag as i32;
        if due < -(m.active as i32) {
            return;
        }
        let away = to_me.normalized();
        let side = V3::new(away.z.neg(), Fx::ZERO, away.x).scale(Fx::from_int(self.strafe));

        // Weighed, not ruled: the same move can be dodged, ducked, walked
        // away from or swung through, and which one it gets is the bot.
        let caution = self.style.caution;
        let aggression = self.style.aggression + self.mood / 2;
        let mut opts: [(Answer, i32); 7] = [
            (Answer::Dodge(side), 20 + caution / 2),
            (Answer::Dodge(away), 10 + caution / 3),
            (Answer::Crouch, if m.hits_crouching { 0 } else { 70 }),
            (
                Answer::Guard,
                if me.shield().is_some_and(|s| s.in_hand()) && !m.unblockable {
                    60 + caution / 2
                } else {
                    0
                },
            ),
            (Answer::Jump, self.style.air / 4),
            (Answer::Back(away), 0),
            (Answer::Crouch, 0),
        ];
        // Walking out works when there is time to walk out of it.
        let walk = sim::tuning::move_speed()
            .mul(sim::DT)
            .mul(Fx::from_int(due.max(0)));
        if walk.raw() > range.sub(dist).raw() && m.aim() == Kind::Swing {
            opts[5].1 = 25 + caution / 2;
        }
        // Swinging first works when yours is out before theirs.
        if let Some(t) = self.fastest_reaching(me, dist, due - 2) {
            opts[6] = (Answer::Counter(t), aggression / 2);
        }
        // Freezing is always on the table. It is what a person does most.
        let freeze = 30 - self.skill.notice as i32 / 4;
        let total: i32 = opts.iter().map(|o| o.1.max(0)).sum::<i32>() + freeze.max(0);
        let mut pick = self.roll(total.max(1) as u32) as i32;
        let mut chosen = None;
        for (answer, weight) in opts {
            if pick < weight.max(0) {
                chosen = Some(answer);
                break;
            }
            pick -= weight.max(0);
        }
        let Some(answer) = chosen else {
            return;
        };
        let slop = self.skill.slop;
        let slop = self.between(-slop, slop);
        let lasts = (due.max(0) + m.active as i32 + 4) as u16;
        let (start_in, hold) = match answer {
            Answer::Dodge(_) => (due - DODGE_LEAD + slop, DODGE_HOLD),
            Answer::Jump => (due - 8 + slop, 18),
            Answer::Counter(_) => (0, 0),
            Answer::Crouch | Answer::Guard | Answer::Back(_) => (slop.max(0), lasts),
        };
        self.pending = Some(Pending {
            answer,
            start_in,
            hold,
        });
    }

    /// Bolts are seen like anything else: late. One heading its way gets a
    /// sidestep, sometimes.
    fn dodge_bolts(&mut self, me: &Player, seen: &Seen) {
        for (slot, b) in seen.bolts.iter().enumerate() {
            let bit = 1u8 << slot;
            let Some(b) = b else {
                self.bolts_seen &= !bit;
                continue;
            };
            if self.bolts_seen & bit != 0 {
                continue;
            }
            self.bolts_seen |= bit;
            let rel = flat(me.pos.sub(b.pos));
            let dir = flat(b.dir).normalized();
            let along = rel.dot(dir);
            if along.raw() <= 0 {
                continue;
            }
            let miss = rel.sub(dir.scale(along)).flat_len();
            if miss.raw() > BOLT_MISS.raw() || self.pending.is_some() {
                continue;
            }
            let notice = self.skill.notice as i32;
            if !self.chance(notice) {
                continue;
            }
            let speed = sim::tuning::fire_bolt_speed().mul(sim::DT);
            let due = along.div(speed.max(Fx::from_raw(1))).to_int() - self.lag as i32;
            let side = V3::new(dir.z.neg(), Fx::ZERO, dir.x).scale(Fx::from_int(self.strafe));
            let slop = self.skill.slop;
            let slop = self.between(-slop, slop);
            self.pending = Some(Pending {
                answer: Answer::Dodge(side),
                start_in: due - DODGE_LEAD + slop,
                hold: DODGE_HOLD,
            });
        }
    }

    /// The quickest move it has that reaches `dist` and is out within
    /// `within` frames.
    fn fastest_reaching(&self, me: &Player, dist: Fx, within: i32) -> Option<Tool> {
        self.mine
            .iter()
            .filter(|t| self.usable(me, *t))
            .filter(|t| {
                let m = moves::get(me.class, t.kind);
                (m.startup as i32) <= within && reach(&m, Some(me)).raw() >= dist.raw()
            })
            .min_by_key(|t| moves::get(me.class, t.kind).startup)
    }

    fn usable(&self, me: &Player, t: Tool) -> bool {
        let m = moves::get(me.class, t.kind);
        m.aim() != Kind::AtTheMechanic
            && me.mechanic_ready(t.kind)
            && !me.locked_out(t.kind)
            && self.tool_bias(me, t) > 0
    }

    /// Something in reach worth throwing, chosen by chance weighted by what it
    /// is worth: damage for the frames it costs.
    fn pick_tool(&mut self, me: &Player, dist: Fx, window: Option<i32>) -> Option<Tool> {
        let mut options = [(None, 0i32); BUTTONS.len()];
        for (i, t) in self.mine.iter().enumerate() {
            if !self.usable(me, t) {
                continue;
            }
            let m = moves::get(me.class, t.kind);
            if reach(&m, Some(me)).raw() < dist.raw() {
                continue;
            }
            if window.is_some_and(|w| m.startup as i32 > w) {
                continue;
            }
            let cost = (m.startup + m.active + m.recovery).max(1) as i32;
            // Worth, and a floor under it so a slow move is still sometimes
            // the one it goes for.
            let worth = (m.damage.max(1) * 100 / cost).clamp(5, 200) + 10;
            options[i] = (Some(t), worth * self.tool_bias(me, t) / 100);
        }
        let total: i32 = options.iter().map(|o| o.1).sum();
        if total == 0 {
            return None;
        }
        let mut pick = self.roll(total as u32) as i32;
        for (t, weight) in options {
            if pick < weight {
                return t;
            }
            pick -= weight;
        }
        None
    }

    /// Decide to throw something.
    fn attempt(&mut self, tool: Tool) {
        if self.attempt.is_some() || self.gesture.is_some() {
            return;
        }
        let spread = self.skill.aim_error.raw();
        let error = Fx::from_raw(self.between(-spread, spread));
        let lead_chance = self.skill.lead as i32;
        let lead = self.chance(lead_chance);
        self.attempt = Some(Attempt {
            tool,
            turning: 0,
            pressing: 0,
            age: 0,
            error,
            lead,
        });
    }

    /// The periodic decision: take an opening if there is one, otherwise get
    /// on with the plan, and pick a new plan when this one runs out.
    fn think(&mut self, me: &Player, seen: &Seen) {
        let them = &seen.them;
        let dist = flat(them.pos.sub(me.pos)).flat_len();

        // An opening: a recovery, the tail of a dodge, a stagger -- how much
        // of it is left in the present, not in the memory.
        let open = match them.action {
            Action::Recovery { left, .. } | Action::Stagger { left } => Some(left as i32),
            Action::HitStun { left } => Some(left as i32),
            Action::Dodge { .. } if !them.action.invulnerable() => {
                Some(them.action.frames_left() as i32)
            }
            _ => None,
        }
        .map(|left| left - self.lag as i32);
        if let Some(window) = open.filter(|w| *w > 0)
            && me.action.actionable()
            && self.attempt.is_none()
        {
            // Walking in is part of the punish, when there is time for it.
            let walk = sim::tuning::move_speed()
                .mul(sim::DT)
                .mul(Fx::from_int(window / 2));
            let punish = self.skill.punish as i32 + self.style.patience / 5;
            if let Some(t) = self.pick_tool(me, dist.sub(walk).max(Fx::ZERO), Some(window))
                && self.chance(punish)
            {
                self.attempt(t);
                self.plan = Plan::Press;
                self.plan_left = self.plan_left.max(20);
                return;
            }
        }

        if self.plan_left == 0 {
            self.choose_plan(me, seen, dist);
        }

        // Swinging, per plan. Each plan swings at a different distance and
        // with a different appetite; none of them swings every time it could.
        if !me.action.actionable() || self.attempt.is_some() {
            return;
        }
        let appetite = match self.plan {
            Plan::Press => 45 + self.style.aggression / 2 + self.mood / 2,
            // Somebody who has stepped into your reach is the whole point of
            // standing at the edge of it.
            Plan::Footsies => 20 + self.style.aggression / 4,
            Plan::Zone => 30 + self.style.zoning / 3,
            Plan::Circle | Plan::Bait => 15,
            Plan::Retreat | Plan::Wait => 6,
            Plan::DashIn | Plan::JumpIn => 0,
        };
        if !self.chance(appetite) {
            return;
        }
        if self.plan == Plan::Zone
            && let Some(t) = self.mine.ranged().filter(|t| self.usable(me, *t))
            && reach(&moves::get(me.class, t.kind), Some(me)).raw() >= dist.raw()
        {
            self.attempt(t);
            return;
        }
        if let Some(t) = self.pick_tool(me, dist, None) {
            self.attempt(t);
        }
    }

    fn choose_plan(&mut self, me: &Player, seen: &Seen, dist: Fx) {
        let st = self.style;
        let mood = self.mood;
        let theirs = self.theirs.threat();
        let mine = self
            .mine
            .iter()
            .map(|t| reach(&moves::get(me.class, t.kind), Some(me)))
            .max_by_key(|r| r.raw())
            .unwrap_or(Fx::from_int(3));
        let ranged = self.mine.ranged().is_some();
        let hurt = me.health * 100 / me.full_health().max(1);
        let their_hurt = seen.them.health * 100 / seen.them.full_health().max(1);
        let ahead = hurt - their_hurt;
        let far = dist.raw() > theirs.add(Fx::from_int(3)).raw();
        let tempo = self.their_tempo;

        let weight = |p: Plan| -> i32 {
            match p {
                Plan::Press => st.aggression * 3 / 2 + mood + if far { 0 } else { 20 } - tempo,
                Plan::Footsies => st.patience + tempo * 3 + 10,
                Plan::Circle => 10 + st.caution / 4,
                Plan::Retreat => {
                    if ahead > 20 {
                        st.caution / 2 + 10
                    } else if hurt < 30 {
                        st.caution / 2
                    } else {
                        st.caution / 5
                    }
                }
                Plan::Zone if ranged => st.zoning + if far { 20 } else { 0 },
                Plan::Zone => 0,
                Plan::DashIn if far && !ranged || dist.raw() < mine.add(Fx::from_int(5)).raw() => {
                    st.aggression / 3 + mood / 2
                }
                Plan::DashIn => 0,
                Plan::JumpIn => st.air / 2 + if far { 0 } else { 10 },
                Plan::Bait => st.patience / 2 + tempo * 2,
                Plan::Wait => st.patience / 4 - mood / 2,
            }
        };
        let mut weights = [0i32; PLANS.len()];
        for (i, p) in PLANS.iter().enumerate() {
            let mut w = weight(*p).max(0);
            // Doing the same thing again is how you get read.
            let repeats = self.recent.iter().filter(|r| *r == p).count() as i32;
            w = w * 3 / (3 + repeats * 2);
            weights[i] = w.max(1);
        }
        let total: i32 = weights.iter().sum();
        let mut pick = self.roll(total as u32) as i32;
        let mut plan = Plan::Wait;
        for (p, w) in PLANS.iter().zip(weights) {
            if pick < w {
                plan = *p;
                break;
            }
            pick -= w;
        }
        self.recent.rotate_right(1);
        self.recent[0] = plan;
        self.plan = plan;
        self.plan_left = match plan {
            Plan::DashIn | Plan::JumpIn => 50,
            Plan::Wait => self.between(15, 45) as u16,
            _ => self.between(40, 110) as u16,
        };
        self.dashed = false;
        self.bait_out = 0;
        if self.chance(35) {
            self.strafe = -self.strafe;
        }
    }

    /// Turn the plan, the pending answers and the attempt into this frame's
    /// buttons and look.
    fn steer_plan(&mut self, me: &Player, seen: &Seen, class: ClassOut) -> Input {
        self.plan_left = self.plan_left.saturating_sub(1);
        self.jump_left = self.jump_left.saturating_sub(1);
        self.dodge_left = self.dodge_left.saturating_sub(1);
        self.strafe_left = self.strafe_left.saturating_sub(1);
        if self.strafe_left == 0 {
            self.strafe_left = self.between(30, 120) as u16;
            if self.chance(40) {
                self.strafe = -self.strafe;
            }
        }

        let them = &seen.them;
        let to = flat(them.pos.sub(me.pos));
        let dist = to.flat_len();
        let toward = to.normalized();
        let side = V3::new(toward.z.neg(), Fx::ZERO, toward.x).scale(Fx::from_int(self.strafe));
        let theirs = self.theirs.threat();
        let mut bits = class.hold;

        // Where it wants to go, by plan.
        let mut want = match self.plan {
            Plan::Press => toward,
            Plan::Footsies => {
                self.dance_left = self.dance_left.saturating_sub(1);
                if self.dance_left == 0 {
                    self.dance_in = !self.dance_in;
                    self.dance_left = self.between(8, 26) as u16;
                }
                let edge = theirs.add(Fx::ratio(4, 10));
                if dist.raw() > edge.add(Fx::ONE).raw() {
                    toward
                } else if dist.raw() < edge.sub(Fx::ONE).raw() {
                    back(toward)
                } else if self.dance_in {
                    toward.add(side).normalized()
                } else {
                    back(toward).add(side).normalized()
                }
            }
            Plan::Circle => {
                let keep = theirs.add(Fx::ONE);
                let nudge = if dist.raw() > keep.raw() {
                    toward.scale(Fx::ratio(1, 2))
                } else {
                    back(toward).scale(Fx::ratio(1, 2))
                };
                side.add(nudge).normalized()
            }
            Plan::Retreat => back(toward).add(side.scale(Fx::ratio(1, 2))).normalized(),
            Plan::Zone => {
                let keep = self
                    .mine
                    .ranged()
                    .map(|t| moves::get(me.class, t.kind).reach.mul(Fx::ratio(6, 10)))
                    .unwrap_or(theirs.add(Fx::from_int(2)));
                if dist.raw() < keep.sub(Fx::ONE).raw() {
                    back(toward).add(side.scale(Fx::ratio(1, 2))).normalized()
                } else if dist.raw() > keep.add(Fx::from_int(2)).raw() {
                    toward
                } else {
                    side
                }
            }
            Plan::DashIn => {
                // Walk to dodging distance, dodge in, swing out of it.
                let dash = sim::tuning::dodge_speed()
                    .mul(sim::DT)
                    .mul(Fx::from_int(sim::tuning::dodge_frames() as i32 / 2));
                if !self.dashed
                    && dist.raw() < dash.add(theirs).raw()
                    && me.action.actionable()
                    && me.grounded
                {
                    self.dashed = true;
                    self.dodge_left = DODGE_HOLD;
                    self.dodge_dir = toward;
                }
                if self.dashed
                    && me.action.actionable()
                    && self.attempt.is_none()
                    && let Some(t) = self.pick_tool(me, dist, None)
                {
                    self.attempt(t);
                    self.plan_left = 0;
                }
                toward
            }
            Plan::JumpIn => {
                let hop = theirs.add(Fx::from_int(3));
                let walled = by_the_wall(me.pos, JUMP_CLEAR) || by_the_wall(them.pos, JUMP_CLEAR);
                if walled {
                    self.plan_left = 0;
                }
                if me.grounded
                    && me.action.actionable()
                    && dist.raw() < hop.raw()
                    && !self.dashed
                    && !walled
                {
                    self.dashed = true;
                    self.jump_left = 24;
                }
                if !me.grounded && me.vel.y.raw() < 0 && self.attempt.is_none() {
                    if let Some(t) = self.pick_tool(me, dist.sub(Fx::ONE), None) {
                        self.attempt(t);
                    }
                    self.plan_left = self.plan_left.min(20);
                }
                toward
            }
            Plan::Bait => {
                let edge = theirs.add(Fx::ratio(3, 10));
                if self.bait_out > 0 {
                    self.bait_out -= 1;
                    back(toward)
                } else if dist.raw() <= edge.raw() {
                    self.bait_out = self.between(12, 24) as u16;
                    back(toward)
                } else {
                    toward
                }
            }
            Plan::Wait => V3::ZERO,
        };
        // Backed up to a wall, it goes round rather than into it.
        let next = me.pos.add(want.scale(Fx::ONE));
        if by_the_wall(next, EDGE) && !by_the_wall(them.pos, Fx::ZERO) {
            want = side;
        }
        // On the wrong side of a wall from the fight -- it went over, or they
        // did -- it heads for the wall and hops it.
        let out_me = by_the_wall(me.pos, Fx::ZERO);
        let out_them = by_the_wall(them.pos, Fx::ZERO);
        if out_me != out_them {
            let head = if out_me {
                flat(back(me.pos)).normalized()
            } else {
                toward
            };
            want = head;
            let wall = sim::arena::proving_ground::half();
            let gap = me
                .pos
                .x
                .abs()
                .sub(wall)
                .abs()
                .min(me.pos.z.abs().sub(wall).abs());
            if gap.raw() < HOP_THE_WALL.raw() && me.grounded && me.action.actionable() {
                self.jump_left = 24;
            }
        }

        if let Some(w) = class.want {
            want = w;
        }

        // Walking into something it could jump onto -- a platform -- it
        // jumps; into the arena's wall, it goes round.
        let walking = want.flat_len().raw() > 0 && me.grounded && me.action.actionable();
        if walking && flat(me.vel).flat_len().raw() < CRAWL.raw() {
            self.stuck += 1;
        } else {
            self.stuck = 0;
        }
        if self.stuck > STUCK {
            self.stuck = 0;
            if by_the_wall(me.pos, JUMP_CLEAR) && out_me == out_them {
                self.strafe = -self.strafe;
                self.plan_left = self.plan_left.min(10);
            } else {
                self.jump_left = 24;
            }
        }

        // A timed answer outranks the plan while it runs.
        if let Some(p) = self.pending.as_mut() {
            p.start_in -= 1;
            if p.start_in <= 0 {
                let p = *p;
                self.pending = None;
                match p.answer {
                    Answer::Counter(t) => {
                        self.attempt = None;
                        self.attempt(t);
                    }
                    Answer::Dodge(dir) => {
                        self.dodge_left = p.hold;
                        self.dodge_dir = dir;
                    }
                    Answer::Jump if by_the_wall(me.pos, JUMP_CLEAR) => {
                        self.dodge_left = DODGE_HOLD;
                        self.dodge_dir = side;
                    }
                    Answer::Jump => self.jump_left = p.hold,
                    a => self.holding = Some((a, p.hold)),
                }
            }
        }
        if let Some((answer, left)) = self.holding {
            match answer {
                Answer::Crouch => bits |= Input::CROUCH,
                Answer::Guard => bits |= Input::RIGHT,
                Answer::Back(dir) => want = dir,
                _ => {}
            }
            self.holding = (left > 1).then_some((answer, left - 1));
            if !matches!(answer, Answer::Back(_)) {
                self.attempt = None;
            }
        }

        // The look. Toward where it saw them, led a little if it thinks to,
        // off by this shot's error, and never faster than a hand turns a
        // mouse.
        let mut target = them.pos;
        let mut error = Fx::ZERO;
        let mut kind = Kind::Swing;
        if let Some(g) = self.gesture {
            target = g.at;
            error = g.error;
            kind = if g.level {
                Kind::Swing
            } else {
                Kind::Skillshot
            };
        } else if let Some(a) = self.attempt {
            let m = moves::get(me.class, a.tool.kind);
            kind = m.aim();
            error = a.error;
            if a.lead {
                // What it saw was `lag` frames ago; where it will be is the
                // lag and the startup further on.
                let ahead = Fx::from_int(self.lag as i32 + m.startup as i32).mul(sim::DT);
                target = target.add(flat(them.vel).scale(ahead));
            }
        }
        let to_target = flat(target.sub(me.pos));
        let wanted = if to_target.flat_len().raw() > Fx::ratio(3, 10).raw() {
            atan2_turns(to_target.z, to_target.x).add(error)
        } else {
            self.yaw
        };
        let off = wrap_turns(wanted.sub(self.yaw));
        let turn = self.skill.turn;
        self.yaw = self.yaw.add(off.clamp(turn.neg(), turn));
        let wire = turns_to_aim(self.yaw.sub(me.carry_yaw));
        let pitch = match kind {
            Kind::Grounded | Kind::Skillshot => aim::look_onto(me.pos, wire, me.aloft, target),
            // A swing takes its pitch from the camera outside a dead zone, and
            // level is the standard arc. Only look up at somebody above you.
            _ if them.pos.y.sub(me.pos.y).raw() > Fx::ONE.raw() => {
                let middle = aim::standing_middle(them.pos);
                aim::look_onto(me.pos, wire, me.aloft, middle).max(0)
            }
            _ => 0,
        };

        // A gesture: wait for the mouse, then press, then whatever follows.
        let mut walk = None;
        if let Some(mut g) = self.gesture {
            let mut done = false;
            if g.pressing > 0 {
                bits |= g.bits;
                walk = g.walk;
                g.pressing -= 1;
                if g.pressing == 0 {
                    done = true;
                    match g.then {
                        After::Nothing => {}
                        After::Swing(t, after) => self.string_in = Some((after, t)),
                        After::Jump(frames) => self.jump_left = frames,
                    }
                }
            } else if !g.free || me.action.actionable() {
                if g.turning == 0 {
                    let spread = self.skill.aim_error.raw();
                    g.error = Fx::from_raw(self.between(-spread, spread));
                }
                let settled = wrap_turns(wanted.sub(self.yaw)).abs().raw() <= ON_TARGET.raw();
                g.turning += 1;
                // A press that needs an edge needs the button up first.
                let edge = self.last_bits & g.bits == 0;
                if edge && (settled || g.turning >= TURN_PATIENCE) {
                    g.pressing = PRESS - 1;
                    bits |= g.bits;
                    walk = g.walk;
                    if g.pressing == 0 {
                        done = true;
                    }
                }
            } else {
                g.age += 1;
                done = g.age > ATTEMPT_LIFE;
            }
            self.gesture = (!done).then_some(g);
        }

        // The attempt: wait for the mouse, then press.
        if self.gesture.is_some() {
            // Its hands are busy.
        } else if let Some(mut a) = self.attempt {
            let mut done = false;
            if a.pressing > 0 {
                bits |= a.tool.bits;
                a.pressing -= 1;
                done = a.pressing == 0;
            } else if me.action.actionable() {
                let settled = wrap_turns(wanted.sub(self.yaw)).abs().raw() <= ON_TARGET.raw();
                a.turning += 1;
                if settled || a.turning >= TURN_PATIENCE {
                    a.pressing = PRESS - 1;
                    bits |= a.tool.bits;
                    // Strings are practised, not reacted to: some of the time
                    // the next press is already on its way.
                    let m = moves::get(me.class, a.tool.kind);
                    let string = 15 + self.style.aggression / 3;
                    if self.chance(string) {
                        let after = m.startup + m.active + self.between(0, 6) as u16;
                        let nth = self.roll(3) as usize;
                        let next = self.mine.iter().nth(nth);
                        self.string_in = next.map(|t| (after, t));
                    }
                }
            } else {
                a.age += 1;
                done = a.age > ATTEMPT_LIFE;
            }
            self.attempt = (!done).then_some(a);
        } else if let Some((left, t)) = self.string_in {
            if left == 0 {
                self.string_in = None;
                self.attempt(t);
            } else {
                self.string_in = Some((left - 1, t));
            }
        }

        if let Some(dir) = walk {
            bits |= steer(self.yaw, dir);
        } else if self.dodge_left > 0 {
            bits |= Input::SHIFT | steer(self.yaw, self.dodge_dir);
        } else {
            bits |= steer(self.yaw, want);
        }
        if self.jump_left > 0 {
            bits |= Input::SPACE;
        }
        // The Dual mage's second jump wants the button up for a frame, then
        // down again.
        match self.second_jump {
            2 => {
                bits &= !Input::SPACE;
                self.second_jump = 1;
            }
            1 => {
                bits |= Input::SPACE;
                self.jump_left = 10;
                self.second_jump = 0;
            }
            _ => {}
        }
        Input::looking_at(bits, wire, pitch)
    }
}

fn xorshift(mut x: u32) -> u32 {
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    x
}

fn flat(v: V3) -> V3 {
    V3::new(v.x, Fx::ZERO, v.z)
}

/// Within `margin` of the arena's walls, or past them.
///
/// **The walls are lower than a jump.** They are a metre and a half and the
/// shortest full hop in the roster is nearly twice that, so a fighter who jumps
/// beside one can land on the far side of it -- the first long run of this bot
/// spent two minutes of a three-minute fight walking round the outside. That
/// is the arena's question to answer, not this bot's; what the bot does is
/// what a person who wants to fight does, and does not jump there.
fn by_the_wall(pos: V3, margin: Fx) -> bool {
    let half = sim::arena::proving_ground::half().sub(margin);
    pos.x.abs().raw() > half.raw() || pos.z.abs().raw() > half.raw()
}

/// The other way.
fn back(v: V3) -> V3 {
    v.scale(Fx::ONE.neg())
}

fn turns_to_aim(turns: Fx) -> u16 {
    (turns.raw() as u32 & 0xFFFF) as u16
}

/// Four keys from a direction, given where the fighter is looking. Eight
/// directions, as a keyboard has.
fn steer(aim: Fx, want: V3) -> u16 {
    let want = flat(want).normalized();
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

/// The result of one bot fight. See [`spar`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bout {
    /// Rounds each side won.
    pub rounds: [u8; MAX_PLAYERS],
    /// Frames played.
    pub frames: u32,
    /// Moves each side threw.
    pub thrown: [u32; MAX_PLAYERS],
    /// Hits each side landed: the other side going into hitstun, a stagger or
    /// a hold, or having one of those restarted.
    pub landed: [u32; MAX_PLAYERS],
    /// Dodges each side made.
    pub dodges: [u32; MAX_PLAYERS],
    /// Times each side used its class mechanic on purpose.
    pub mechanic: [u32; MAX_PLAYERS],
}

/// Play two bots against each other for `frames`, and count.
pub fn spar(
    classes: [Class; MAX_PLAYERS],
    levels: [Level; MAX_PLAYERS],
    frames: u32,
    seed: u32,
) -> Bout {
    spar_traced(classes, levels, frames, seed, |_, _| {})
}

/// [`spar`], calling `trace` with the world and the bots after every frame.
pub fn spar_traced(
    classes: [Class; MAX_PLAYERS],
    levels: [Level; MAX_PLAYERS],
    frames: u32,
    seed: u32,
    mut trace: impl FnMut(&World, &[Duelist; MAX_PLAYERS]),
) -> Bout {
    let mut w = World::with_classes(classes);
    let mut bots = [
        Duelist::new(0, levels[0], seed),
        Duelist::new(1, levels[1], seed.wrapping_mul(0x9E37_79B9) ^ 0x5bd1_e995),
    ];
    let mut bout = Bout {
        rounds: [0; MAX_PLAYERS],
        frames: 0,
        thrown: [0; MAX_PLAYERS],
        landed: [0; MAX_PLAYERS],
        dodges: [0; MAX_PLAYERS],
        mechanic: [0; MAX_PLAYERS],
    };
    while w.frame < frames {
        for b in bots.iter_mut() {
            b.watch(&w);
        }
        let wire = [bots[0].act(&w), bots[1].act(&w)];
        let before = w.clone();
        w.advance(wire);
        for i in 0..MAX_PLAYERS {
            let (was, now) = (before.players[i].action, w.players[i].action);
            if matches!(now, Action::Startup { .. } | Action::Channel { .. })
                && !matches!(was, Action::Startup { .. } | Action::Channel { .. })
            {
                bout.thrown[i] += 1;
            }
            if matches!(now, Action::Dodge { .. }) && !matches!(was, Action::Dodge { .. }) {
                bout.dodges[i] += 1;
            }
            let other = 1 - i;
            let (was, now) = (before.players[other].action, w.players[other].action);
            let struck = |a: Action| {
                matches!(
                    a,
                    Action::HitStun { .. } | Action::Stagger { .. } | Action::Held { .. }
                )
            };
            if struck(now) && (!struck(was) || now.frames_left() > was.frames_left()) {
                bout.landed[i] += 1;
            }
        }
        trace(&w, &bots);
    }
    bout.rounds = [w.players[0].rounds_won, w.players[1].rounds_won];
    bout.mechanic = [bots[0].mechanic_uses, bots[1].mechanic_uses];
    bout.frames = w.frame;
    bout
}
