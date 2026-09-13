//! World state and the tick function.
//!
//! Every field here is part of the rollback snapshot. If it affects gameplay it
//! lives in `World`; if it does not, it lives in the renderer.
//!
//! **Animation state belongs here.** Rollback re-simulates past frames, so any
//! animation the engine drives independently will pop when a rollback happens.
//! Actions are frame-data state machines, which is how fighting games are built
//! anyway.
//!
//! Feel numbers live in [`crate::tuning`]; move tables live in
//! [`crate::moves`]. Neither belongs inline here — they change constantly and
//! need to be findable. See `docs/design/feel-log.md`.

use crate::DT;
use crate::aim::{self, Contact, Path, Scene};
use crate::arena;
use crate::bolt::{self, Flight, MAX_BOLTS};
pub use crate::class::Shield;
use crate::class::{self, Class, Form, Ghost, Mechanic};
use crate::effects::{Effect, EffectKind, GRASP_ARMS, LOTUS_BLADES, MAX_EFFECTS, QUARRY_VICTIM};
use crate::fixed::Fx;
use crate::input::Input;
use crate::math::V3;
use crate::monster::{self, Doing, Monster, Quarry};
use crate::moves;
use crate::shadow;
use crate::stones::{self, Field};
use crate::tuning as t;

pub const MAX_PLAYERS: usize = 2;
pub type PlayerId = usize;

const GROUND_Y: Fx = Fx::ZERO;

/// Move slots are positional and mean the same thing on every class, which is
/// what lets one control scheme drive six kits. See `controls.md`.
/// `held_by` when nobody is holding you.
pub const NOBODY: u8 = u8::MAX;

/// `Phase::RoundOver::winner` when the creature is the one still standing.
/// Player indices and `u8::MAX` for a double knockout are already spoken for.
pub const QUARRY: u8 = 200;

/// Half a turn, so a creature can be faced at the hunters without arithmetic.
/// An angle unit, not a quantity.
const HALF_TURN: Fx = Fx::from_raw(1 << 15);

pub const SLOT_POKE: u8 = 0;
pub const SLOT_COMMITTED: u8 = 1;
pub const SLOT_SPECIAL: u8 = 2;
/// `E`. An ability on some classes and a state change on the rest -- see
/// [`moves::bound`] and [`mechanic_action`].
pub const SLOT_MECHANIC: u8 = 3;
pub use crate::tuning::max_health;

// The Bulwark's shield and the Reaver's shadow used to keep their numbers here,
// as `const`s beside the code that read them. They are in the Oven now: a
// shield's speed and range decide the class's whole spacing game, which makes
// them exactly as much a feel number as any frame count.

/// What a character is currently doing. Durations are frame counts, matching
/// the `startup / active / recovery` vocabulary in `ability-spec.md`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Action {
    Free,
    Startup {
        kind: u8,
        left: u16,
    },
    Active {
        kind: u8,
        left: u16,
    },
    Recovery {
        kind: u8,
        left: u16,
    },
    /// `held` counts up. The first `t::parry_window()` frames parry.
    Guard {
        held: u16,
    },
    BlockStun {
        left: u16,
    },
    HitStun {
        left: u16,
    },
    Stagger {
        left: u16,
    },
    /// Committed evasive roll. Invulnerable for its opening frames, then
    /// recovering and vulnerable -- so it beats a read and loses to a delayed
    /// attack.
    Dodge {
        left: u16,
    },
    /// Caught by a grab. You cannot act, and you are dragged to whoever has
    /// you, which is what makes a grab different from a long stun: it moves
    /// you, so the grabber decides where the next exchange happens.
    Held {
        left: u16,
    },
}

impl Action {
    pub const fn actionable(self) -> bool {
        matches!(self, Action::Free)
    }

    pub const fn guarding(self) -> bool {
        matches!(self, Action::Guard { .. })
    }

    /// Which move is running, across all three of its phases.
    pub const fn attack_kind(self) -> Option<u8> {
        match self {
            Action::Startup { kind, .. }
            | Action::Active { kind, .. }
            | Action::Recovery { kind, .. } => Some(kind),
            _ => None,
        }
    }

    /// Invulnerable frames of a dodge. Nothing else grants invulnerability.
    pub fn invulnerable(self) -> bool {
        matches!(self, Action::Dodge { left } if left + t::dodge_iframes() > t::dodge_frames())
    }

    /// Blocking costs a vulnerable window, so it is never a safe default.
    /// See `defense.md`.
    pub const fn stunned(self) -> bool {
        matches!(
            self,
            Action::BlockStun { .. }
                | Action::HitStun { .. }
                | Action::Stagger { .. }
                | Action::Held { .. }
        )
    }

    pub const fn tag(self) -> u32 {
        match self {
            Action::Free => 0,
            Action::Startup { .. } => 1,
            Action::Active { .. } => 2,
            Action::Recovery { .. } => 3,
            Action::Guard { .. } => 4,
            Action::BlockStun { .. } => 5,
            Action::HitStun { .. } => 6,
            Action::Stagger { .. } => 7,
            Action::Held { .. } => 9,
            Action::Dodge { .. } => 8,
        }
    }

    pub const fn frames_left(self) -> u16 {
        match self {
            Action::Free => 0,
            Action::Startup { left, .. }
            | Action::Active { left, .. }
            | Action::Recovery { left, .. }
            | Action::BlockStun { left }
            | Action::HitStun { left }
            | Action::Stagger { left }
            | Action::Held { left }
            | Action::Dodge { left } => left,
            Action::Guard { held } => held,
        }
    }
}

/// Match flow.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Fighting,
    /// `winner` is a player index, or `u8::MAX` for a double knockout.
    RoundOver {
        winner: u8,
        left: u16,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Player {
    pub pos: V3,
    pub vel: V3,
    /// Unit horizontal vector. Stored as a vector rather than an angle so that
    /// turning needs no trigonometry -- one less determinism risk.
    pub facing: V3,
    pub health: i32,
    pub action: Action,
    pub grounded: bool,
    /// Frames of jump sustain left. Set on takeoff, spent while the button is
    /// held and you are rising, and zeroed the moment it is released -- so a
    /// re-press cannot extend a jump you already let go of.
    pub jump_hold: u16,
    /// Frames of suspended fall from an aerial attack. See `Move::air_stall`.
    pub air_stall: u16,
    /// An airdodge has been spent this airtime. Reset on landing.
    ///
    /// Without it, airdodging repeatedly is free flight: each one is a fresh
    /// burst of horizontal speed with invulnerability attached.
    pub air_dodged: bool,
    /// True once the current active window has connected, so a move hits once.
    pub hit_used: bool,
    pub class: Class,
    pub mechanic: Mechanic,
    pub rounds_won: u8,
    pub crouching: bool,
    /// Frames of slow left. Whatever slowed you sets it and it counts down, so
    /// the slow has a tail and does not flicker on the field's boundary.
    pub slowed: u16,
    /// How much of your speed the slow leaves you, while it lasts.
    ///
    /// Carried on the fighter rather than read from the thing that applied it,
    /// because there is more than one thing now: a drain field is a wall and a
    /// churning stone is a warning, and they cannot share a number. **The
    /// strongest wins** rather than compounding -- two slows that multiplied
    /// would freeze you, and every new source would make the last one worse.
    pub slow_mul: Fx,
    /// Frames of root left. Your feet do not carry you, you cannot dodge and
    /// you cannot jump; you can still turn, guard and swing.
    ///
    /// Separate from `slowed` rather than a slow of zero, and deliberately.
    /// A slow is a tax on movement and the strongest one wins; a root is the
    /// absence of movement and it also takes the two *buttons* that would
    /// otherwise be a way out. Overloading one on the other would mean every
    /// future slow had to be checked against "but is this one actually a
    /// root", which is the kind of question that gets answered wrong once.
    ///
    /// The design allows exactly one hard stop and only behind a hard
    /// condition (`ability-spec.md`); standing where all four arms of a Grasp
    /// converge is that condition.
    pub rooted: u16,
    /// Was the mechanic button down last frame? Part of the snapshot, so the
    /// press edge survives rollback.
    pub mechanic_held: bool,
    /// Who is holding this fighter, or `u8::MAX`. A grab has to know its owner
    /// so the victim can be kept at arm's length rather than merely stunned.
    pub held_by: u8,
    /// Was the jump button down last frame? The press edge, kept in the
    /// snapshot for the same reason `mechanic_held` is: rollback re-runs these
    /// frames, so an edge remembered outside the snapshot is an edge that
    /// disappears the first time a frame is replayed.
    ///
    /// The ordinary jump does not need it -- it is level-triggered, and
    /// deliberately, so that holding space hops the moment you land. The
    /// uppercut's extra leap does: it is worth height, and a held button that
    /// spent it every frame would make the leap automatic rather than timed.
    pub space_held: bool,
    /// The uppercut's extra leap has been spent this carry. Cleared when an
    /// uppercut starts and whenever the feet are back on the floor.
    pub leap_used: bool,
    /// Downward speed banked by a spike, cashed in when this fighter lands.
    ///
    /// The Champion's aerial hammer drives an airborne target into the ground,
    /// and the ground is meant to be the larger half of that: the hit does its
    /// damage, and then the landing does more. Carried as the speed rather
    /// than as a flag so the cost scales with how far they had to fall --
    /// bounded above by terminal velocity, which is what makes the worst case
    /// knowable.
    pub slam: Fx,

    // -- Animation clocks ---------------------------------------------------
    //
    // These four exist for the renderer and change nothing about combat. They
    // are here rather than there because *animation state belongs in the
    // snapshot*: rollback re-simulates past frames, and a walk cycle or a
    // landing that runs off the renderer's own clock slides and pops every
    // time it happens. See `docs/design/architecture.md`.
    /// Where the body is in its stride, as a fraction of one two-step cycle in
    /// 1/65536ths, wrapping.
    ///
    /// A walk cycle driven from the frame counter skates: the feet keep the
    /// same cadence whether the body is crawling or sprinting. This is driven
    /// by ground covered instead, so a footfall happens every stride's worth of
    /// metres at any speed.
    ///
    /// It is an **accumulator**, not a ratio, and that distinction is the whole
    /// reason it lives here rather than being worked out in the renderer from a
    /// distance travelled. A stride is longer at a sprint than at a walk and
    /// shorter sideways than forwards, so `distance / stride` jumps by whole
    /// cycles the moment the stride changes -- which reads as both legs
    /// teleporting. Integrating `speed / stride` cannot do that.
    pub stride: u16,
    /// Frames off the ground, saturating. On the ground it holds how long the
    /// last flight was, which is what tells a hop from a fall.
    pub air_frames: u16,
    /// Frames since touching down, saturating. Zero while airborne.
    pub since_landed: u16,
    /// Frames left of the parry flourish. A parry costs the defender nothing
    /// and is easy to miss; this is what lets them see that they got it.
    pub parried: u16,
    /// Frames spent crouching, saturating. Zero while standing.
    ///
    /// `crouching` is a bool, and a bool cannot say how long it has been true --
    /// so without this the drop into a crouch has no clock and the entry
    /// animation is skipped entirely by anyone who was already standing still.
    pub crouched_for: u16,
    /// How long the stun currently being served was when it started.
    ///
    /// `Action::HitStun { left }` counts down and never says what it counted
    /// down *from*, so the renderer cannot tell a graze from a Slam. It needs
    /// to: those get different animations, and picking between them halfway
    /// through would visibly switch clips mid-flinch.
    pub stun_total: u16,

    /// Which part of the creature this fighter is standing on, or
    /// `monster::NO_PART`.
    ///
    /// **While this is set, `local` is the authoritative position and `pos` is
    /// derived from it.** Unmounted it is the other way round. That one rule is
    /// the whole of why the creature can spin under a rider without sliding
    /// them off: a world position would have to be corrected for the rotation
    /// every frame, and a body-space one never has to be corrected at all.
    pub mount: u8,
    /// Position in the mounted part's own rest frame -- before the tail's swing
    /// or the head's reach, so a rider on the tail swings with it.
    pub local: V3,
    /// Yaw carried over from the creature's turning, added to the aim that
    /// arrives as input.
    ///
    /// Movement is camera-relative already, so carrying the look angle makes
    /// movement *surface*-relative for free and turns the camera with the
    /// animal instead of turning the animal away underneath it. One mechanism,
    /// both effects. The aim on the wire is never rewritten; this is simulation
    /// state, recomputed from the snapshot, so rollback reproduces it.
    pub carry_yaw: Fx,
    /// World velocity of the patch of creature under this fighter's feet, last
    /// frame. The buck is the change in this.
    pub grip_vel: V3,
    /// Frames left before the grip test starts. Landing gives you a moment to
    /// plant your feet; without it the first frame aboard looks like an
    /// infinite acceleration and throws you straight back off.
    pub grip_settle: u8,

    /// How far the Elementalist's beam actually reached this frame, or zero
    /// when no beam is out.
    ///
    /// The shot is a line, and where it *stopped* is as much a part of it as
    /// where it started -- a beam that met a stone two metres out is two metres
    /// long, not nine. Kept in the snapshot so the renderer draws the line the
    /// simulation tested rather than a reconstruction of it that can drift, and
    /// so a rollback redraws the same one. See [`crate::bolt`].
    pub beam_reach: Fx,
    /// The line the move currently running was aimed along: where it leaves
    /// from and where it ends.
    ///
    /// **A path rather than a point and a direction**, because those two can
    /// disagree and did: a direction taken from the look angle and a target
    /// taken from the crosshair are parallel rays that never converge, so the
    /// reticle sat on one spot and the ability went to another. One object
    /// cannot drift from itself. Solved by `crate::aim`, which is the only
    /// thing allowed to produce one.
    ///
    /// Locked when the move starts, for the same reason facing is: a target you
    /// could drag during startup would let a whiff be rescued after the fact,
    /// and an area you could slide onto someone during the wind-up would make
    /// the telegraph worth nothing. You commit to a place when you commit to
    /// the move.
    pub aim_path: Path,
}

impl Player {
    pub fn new(class: Class) -> Player {
        Player {
            class,
            mechanic: class.starting_mechanic(),
            ..Player::default()
        }
    }

    /// The Bulwark's shield, if this is one.
    pub fn shield(&self) -> Option<Shield> {
        match self.mechanic {
            Mechanic::Shield(s) => Some(s),
            _ => None,
        }
    }

    /// Can this class act right now given its mechanic? Gated moves need the
    /// mechanic in a particular state; ungated ones never do.
    pub fn mechanic_ready(&self, kind: u8) -> bool {
        if !moves::get(self.class, kind).needs_mechanic {
            return match self.mechanic {
                // The Bulwark's ordinary moves need the shield in hand.
                Mechanic::Shield(sh) => sh.in_hand(),
                _ => true,
            };
        }
        match self.mechanic {
            Mechanic::Shield(sh) => sh.in_hand(),
            // The shadow is never nowhere -- at her shoulder or out on the
            // field, but always somewhere -- so a move that needs one always
            // has one. The gate stays declared on the move (`one_aim.rs`
            // insists a move aimed at the mechanic has it) and is simply
            // always satisfied for this class.
            Mechanic::Shadow(_) => true,
            Mechanic::Structures(slots) => slots.iter().any(|s| s.is_some()),
            Mechanic::Meter { value } => value.abs() >= t::meter_deep(),
            Mechanic::Forms { .. } | Mechanic::Blood => true,
        }
    }

    /// Take a slow. The strongest one on you is the one that counts, and any
    /// of them refreshes the clock.
    pub fn slow(&mut self, frames: u16, mul: Fx) {
        if self.slowed == 0 || mul.raw() < self.slow_mul.raw() {
            self.slow_mul = mul;
        }
        self.slowed = self.slowed.max(frames);
    }

    /// Take a root. The longest one on you wins, the same way a slow works.
    pub fn root(&mut self, frames: u16) {
        self.rooted = self.rooted.max(frames);
    }

    /// Pinned. Not a stun: you can still turn, guard and attack.
    pub const fn is_rooted(&self) -> bool {
        self.rooted > 0
    }

    /// Are this fighter's options gone?
    ///
    /// Rooted, staggered, or held. **Hitstun is deliberately not in the list**,
    /// and that is the whole of the definition: hitstun happens on every hit
    /// anybody lands, so counting it would turn "increased damage to disabled
    /// enemies" into "increased damage from the second hit onward", which is a
    /// flat damage bonus wearing a costume.
    ///
    /// What is in the list is what `ability-spec.md` calls a hard stop, and the
    /// design only allows those behind a hard condition -- a parry for the
    /// stagger, a grab for the hold, every arm of a Grasp for the root. Each
    /// one had to be *earned*, which is exactly what a payoff should be waiting
    /// on. Blockstun is not one: they blocked, which was the correct decision,
    /// and rewarding the attacker for it would make guarding worse than
    /// standing still.
    pub const fn disabled(&self) -> bool {
        self.rooted > 0 || matches!(self.action, Action::Stagger { .. } | Action::Held { .. })
    }

    /// Pay for a move out of your own health, and take the return on one.
    ///
    /// Both clamp. Self-damage stops at one -- dying to your own button is not
    /// a decision anybody made, and the Dual mage's meter burn already works
    /// this way -- and healing stops at full, so a Blood mage cannot bank
    /// health above the bar by farming a field.
    pub fn spend_health(&mut self, cost: i32) {
        if cost > 0 {
            self.health = (self.health - cost).max(1);
        }
    }

    pub fn heal(&mut self, amount: i32) {
        if amount > 0 && self.health > 0 {
            self.health = (self.health + amount).min(t::max_health());
        }
    }

    /// Standing on the creature.
    pub fn aboard(&self) -> bool {
        self.mount != monster::NO_PART
    }

    /// Where this fighter is actually looking, once the creature's turning has
    /// been carried into it.
    pub fn aim(&self, input: Input) -> Fx {
        input.aim_turns().add(self.carry_yaw)
    }

    /// Where the move currently running is aimed: the end of its path.
    pub fn aim_at(&self) -> V3 {
        self.aim_path.to
    }

    /// The line it travels along, as a unit vector -- pitch included, unlike
    /// `facing`, which is flattened because a body only ever turns level.
    pub fn aim_dir(&self) -> V3 {
        self.aim_path.dir()
    }

    /// Height of the hurtbox. Crouching ducks under anything aimed high.
    pub fn hurt_height(&self) -> Fx {
        if self.crouching {
            t::body_height().mul(t::crouch_height_scale())
        } else {
            t::body_height()
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Player {
            pos: V3::ZERO,
            vel: V3::ZERO,
            facing: V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
            health: 1000,
            action: Action::Free,
            grounded: true,
            jump_hold: 0,
            air_stall: 0,
            air_dodged: false,
            hit_used: false,
            class: Class::Bulwark,
            mechanic: Mechanic::Shield(Shield::Held),
            rounds_won: 0,
            crouching: false,
            slowed: 0,
            slow_mul: Fx::ONE,
            rooted: 0,
            mechanic_held: false,
            held_by: NOBODY,
            space_held: false,
            leap_used: false,
            slam: Fx::ZERO,
            stride: 0,
            air_frames: 0,
            since_landed: 0,
            parried: 0,
            crouched_for: 0,
            stun_total: 0,
            mount: monster::NO_PART,
            local: V3::ZERO,
            carry_yaw: Fx::ZERO,
            grip_vel: V3::ZERO,
            grip_settle: 0,
            beam_reach: Fx::ZERO,
            aim_path: Path::default(),
        }
    }
}

/// The complete rollback snapshot.
#[derive(Clone, PartialEq, Debug)]
pub struct World {
    pub frame: u32,
    pub players: [Player; MAX_PLAYERS],
    pub phase: Phase,
    /// Things moves have left behind. Fixed size: see `effects`.
    pub effects: [Option<Effect>; MAX_EFFECTS],
    /// Fire bolts in flight. The only projectile a fighter throws that is not
    /// part of somebody's mechanic -- see [`crate::bolt`].
    pub bolts: Flight,
    /// The quarry, in a hunt. `None` is a versus match.
    ///
    /// One slot rather than an array: a second creature is a thing to build
    /// once there is something to learn from it, and an array of one is a
    /// promise the code has not earned. When a second arrives, the parts, the
    /// ride and the control algorithm are what it reuses; this field is what
    /// changes.
    pub monster: Option<Monster>,
}

impl World {
    pub fn new() -> World {
        World::with_classes([Class::Bulwark; MAX_PLAYERS])
    }

    /// Start a match with a chosen class per side.
    pub fn with_classes(classes: [Class; MAX_PLAYERS]) -> World {
        let mut w = World {
            frame: 0,
            players: [Player::default(); MAX_PLAYERS],
            phase: Phase::Fighting,
            effects: [None; MAX_EFFECTS],
            bolts: [None; MAX_BOLTS],
            monster: None,
        };
        for (p, class) in w.players.iter_mut().zip(classes.iter()) {
            *p = Player::new(*class);
        }
        w.reset_positions();
        w
    }

    /// Start a hunt: the same fighters, with a creature in the arena.
    ///
    /// Friendly fire is off for the duration, and it is off because the monster
    /// is there rather than because a flag says so -- one condition, in one
    /// place, that cannot get out of step with what is on screen.
    pub fn hunt(classes: [Class; MAX_PLAYERS]) -> World {
        let mut w = World::with_classes(classes);
        w.monster = Some(Monster::new());
        w.reset_positions();
        w
    }

    /// Put both fighters back on their marks. Keeps round wins and classes.
    fn reset_positions(&mut self) {
        // Nothing in the air survives a round. A bolt still flying when the
        // last one ended would land on somebody standing on their mark.
        self.bolts = [None; MAX_BOLTS];
        for (i, p) in self.players.iter_mut().enumerate() {
            let wins = p.rounds_won;
            let class = p.class;
            let side = if i == 0 { -4 } else { 4 };
            *p = Player {
                pos: V3::new(Fx::from_int(side), GROUND_Y, Fx::ZERO),
                facing: V3::new(
                    if i == 0 { Fx::ONE } else { Fx::ONE.neg() },
                    Fx::ZERO,
                    Fx::ZERO,
                ),
                rounds_won: wins,
                ..Player::new(class)
            };
            // The second body starts where it lives: at her shoulder. Built
            // after the mark is chosen rather than in `Player::new`, which does
            // not know where anybody is standing yet -- and a shadow that spent
            // the first second of a round easing in from the world origin is a
            // shadow the player watches instead of the fight.
            if let Mechanic::Shadow(_) = p.mechanic {
                p.mechanic = Mechanic::Shadow(class::Shadow::attending(p.pos, p.facing));
            }
        }
        if self.monster.is_some() {
            let mut beast = Monster::new();
            // Well back, and facing the hunters. A creature that spawns on top
            // of you has taken the opening read away from both of you.
            beast.pos = V3::new(t::monster_spawn(), Fx::ZERO, Fx::ZERO);
            beast.yaw = HALF_TURN;
            beast.brain.seen = self.players[0].pos;
            for (i, p) in self.players.iter_mut().enumerate() {
                p.pos = V3::new(
                    t::hunter_spawn().neg(),
                    GROUND_Y,
                    Fx::from_int(if i == 0 { -2 } else { 2 }),
                );
                p.facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
            }
            self.monster = Some(beast);
        }
    }

    /// Advance exactly one tick. This is the whole simulation.
    ///
    /// Must stay a pure function of `(self, inputs)`: no clocks, no randomness
    /// that is not seeded from state, no iteration over unordered collections.
    pub fn advance(&mut self, inputs: [Input; MAX_PLAYERS]) {
        self.frame = self.frame.wrapping_add(1);

        if let Phase::RoundOver { winner, left } = self.phase {
            if left > 0 {
                self.phase = Phase::RoundOver {
                    winner,
                    left: left - 1,
                };
                // Bodies still settle during the pause; nothing else acts.
                for p in self.players.iter_mut() {
                    settle(p);
                    advance_clocks(p);
                }
                return;
            }
            self.reset_positions();
            self.phase = Phase::Fighting;
            return;
        }

        // Stones move before the fighters do, so what a fighter walks into --
        // or stands on -- is where the stone is this frame rather than where it
        // was last one.
        stones::step(&mut self.players);
        let field = stones::gather(&self.players);

        // The creature decides and moves first, so that riders are carried by
        // a transform that is already final for this frame. It is handed a
        // deliberately small window on the fighters -- positions and
        // velocities, no buttons -- so "it reads your inputs" is not a thing
        // that can quietly become true. See `monster::Quarry`.
        let mut spin = Fx::ZERO;
        if let Some(mut beast) = self.monster {
            let before = beast.yaw;
            let seen: [Quarry; MAX_PLAYERS] = std::array::from_fn(|i| Quarry {
                pos: self.players[i].pos,
                vel: self.players[i].vel,
                alive: self.players[i].health > 0,
                aboard: self.players[i].aboard(),
            });
            beast.step(&seen);
            // Its *heading*, not its wobble: a shake would otherwise spin the
            // rider's camera as hard as it spins the animal, and you are about
            // to be thrown off anyway.
            spin = crate::math::wrap_turns(beast.yaw.sub(before));
            self.monster = Some(beast);
        }

        // Everything the aiming ray can meet, as it stood at the top of the
        // frame. Copied rather than borrowed because the loop below takes each
        // fighter mutably -- and snapshotting is right anyway: both players
        // aim against the same world, so neither ordering wins.
        let beast = self.monster;
        let seen = self.players;
        let effects = self.effects;
        // Who has hold of somebody, worked out before anybody moves. The
        // uppercut's leap needs it and a fighter cannot see the other one from
        // inside `step_player`, which only ever gets its own body.
        let carrying: [bool; MAX_PLAYERS] = std::array::from_fn(|i| {
            seen.iter()
                .any(|v| matches!(v.action, Action::Held { .. }) && v.held_by == i as u8)
        });
        for (i, (p, input)) in self.players.iter_mut().zip(inputs).enumerate() {
            if p.aboard() {
                p.carry_yaw = crate::math::wrap_turns(p.carry_yaw.add(spin));
            }
            let scene = Scene {
                stones: &field,
                players: &seen,
                effects: &effects,
                quarry: beast.as_ref(),
            };
            step_player(p, i, input, &field, beast.as_ref(), &scene, carrying[i]);
            advance_clocks(p);
        }

        // What a move does *as it comes out*, on its first active frame: the
        // leap of a leaping move, and whatever it leaves standing in the world.
        for i in 0..MAX_PLAYERS {
            let p = self.players[i];
            let Action::Active { kind, left } = p.action else {
                continue;
            };
            let m = moves::get(p.class, kind);
            if left != m.active {
                continue;
            }
            if m.self_lift.raw() > 0 {
                self.players[i].vel.y = m.self_lift;
                self.players[i].grounded = false;
            }
            // `E` on the Reaver: the second body goes out, or comes home
            // through whatever is in the way. On the first active frame like
            // everything else a move does, so the startup is a window somebody
            // can punish rather than a formality.
            if p.class == Class::ShadowReaver && kind == SLOT_MECHANIC {
                self.order_the_shadow(i, p.aim_at());
            }
            if let Some(leaves) = EffectKind::from_code(m.effect) {
                // Where the move was aimed when it was thrown, already solved
                // against the terrain and the move's reach. It used to be a
                // fixed distance straight ahead at floor level, which meant an
                // area ability could only ever be placed by walking.
                //
                // The two that travel start at the caster's hand instead, and
                // go *along* the aim rather than to it: a blade thrown at
                // something four metres away still flies its full distance, and
                // the crosshair picked the line, not the landing spot.
                //
                // Both come straight off the path `crate::aim` already solved,
                // which for a travelling effect is the hand through the point
                // the crosshair is on. Working the line out here from a target
                // and a look angle is how the two came to disagree.
                let (from, along) = if leaves.travels() {
                    (p.aim_path.from, p.aim_path.dir())
                } else {
                    (p.aim_at(), V3::ZERO)
                };
                spawn_effect(
                    &mut self.effects,
                    Effect::cast(leaves, i as u8, p.class, kind, from, along),
                );
            }
        }

        // The Elementalist's auto: a beam, resolved on the spot. See
        // `crate::bolt` and `docs/design/kits/elementalist.md`.
        for i in 0..MAX_PLAYERS {
            self.fire_the_beam(i);
        }

        // Hit resolution after both have stepped, so neither ordering wins.
        // In a hunt the fighters cannot hurt each other: the condition is the
        // creature's presence rather than a separate flag, so there is nothing
        // for the two to get out of step about.
        let snapshot = self.players;
        for attacker in 0..MAX_PLAYERS {
            if self.monster.is_some() {
                break;
            }
            let defender = 1 - attacker;
            if let Some(hit) = resolve_hit(&snapshot[attacker], &snapshot[defender], attacker as u8)
            {
                // What the blow is actually worth, before it lands: a killing
                // hit on someone with forty health left is worth forty, not its
                // listed damage, so leeching cannot pay out of an empty bar.
                let dealt = if hit.blocked || hit.parried {
                    0
                } else {
                    hit.damage.min(snapshot[defender].health)
                };
                apply_hit(&mut self.players[defender], hit);
                let owed = snapshot[attacker]
                    .action
                    .attack_kind()
                    .map(|kind| moves::get(snapshot[attacker].class, kind).leeched(dealt))
                    .unwrap_or(0);
                self.players[attacker].heal(owed);
                self.players[attacker].hit_used = true;
                // The autos are the steering wheel, and they only steer when
                // they land: the Dual mage's fast way back toward centre is a
                // far-side auto, which is why the class has to close distance
                // exactly when it is strongest. See `docs/design/dual-mage.md`.
                if let Some(kind) = snapshot[attacker].action.attack_kind() {
                    if steers_on_contact(snapshot[attacker].class, kind) {
                        steer_meter(&mut self.players[attacker], kind);
                    }
                }
                // The aerial spear pays its shove out on contact rather than
                // on the throw: catch somebody with the fan and it kicks you
                // the way you are holding, so it is a repositioning tool you
                // have to earn. Read from the live input rather than from a
                // direction locked at the throw, because the whole point is
                // that you choose where to go *as* it connects.
                champion_fan_boost(&mut self.players[attacker], inputs[attacker]);
                if hit.parried {
                    self.players[attacker].action = Action::Stagger {
                        left: t::parry_stagger(),
                    };
                    self.players[attacker].stun_total = t::parry_stagger();
                    self.players[defender].parried = PARRY_FLOURISH;
                }
            }
        }

        // A thrown shield is its own threat while it travels.
        let shields = [self.players[0].shield(), self.players[1].shield()];
        for (owner, shield) in shields.iter().enumerate() {
            let target = 1 - owner;
            let Some(Shield::Flying { pos, .. }) = *shield else {
                continue;
            };
            {
                let victim = self.players[target];
                let d = victim.pos.sub(pos);
                let hit_range = t::shield_radius().add(t::body_radius());
                let vertical = d.y.abs().raw() < t::body_height().raw();
                if vertical && d.flat_len().raw() < hit_range.raw() && !victim.action.invulnerable()
                {
                    let dir = V3::new(d.x, Fx::ZERO, d.z).normalized();
                    apply_hit(
                        &mut self.players[target],
                        Hit {
                            damage: t::shield_damage(),
                            hitstun: t::shield_hitstun(),
                            blockstun: t::shield_blockstun(),
                            knockback: t::shield_knockback(),
                            launch: Fx::ZERO,
                            grabs: 0,
                            by: owner as u8,
                            dir,
                            blocked: victim.action.guarding(),
                            parried: false,
                        },
                    );
                    // Contact drops it where it struck.
                    self.players[owner].mechanic = Mechanic::Shield(Shield::Planted { pos });
                }
            }
        }

        if self.monster.is_some() {
            self.trade_with_the_creature();
        }

        self.step_shadows();
        self.step_effects();
        let standing = self.effects;
        bolt::step(
            &mut self.bolts,
            &mut self.players,
            &standing,
            self.monster.is_none(),
            &mut self.monster,
        );
        stones::touch(&mut self.players);
        separate_bodies(&mut self.players);
        drag_the_held(&mut self.players);

        // Knockout check last, so the killing blow is fully applied first.
        if let (Phase::Fighting, Some(beast)) = (self.phase, self.monster) {
            let standing = self.players.iter().any(|p| p.health > 0);
            let winner = if !beast.alive() {
                0
            } else if !standing {
                QUARRY
            } else {
                return;
            };
            if winner != QUARRY {
                for p in self.players.iter_mut().filter(|p| p.health > 0) {
                    p.rounds_won += 1;
                }
            }
            self.phase = Phase::RoundOver {
                winner,
                left: t::round_over_frames(),
            };
            return;
        }
        if matches!(self.phase, Phase::Fighting) {
            let down = [self.players[0].health <= 0, self.players[1].health <= 0];
            if down[0] || down[1] {
                let winner = match down {
                    [true, true] => u8::MAX,
                    [true, false] => 1,
                    _ => 0,
                };
                if winner != u8::MAX {
                    self.players[winner as usize].rounds_won += 1;
                }
                self.phase = Phase::RoundOver {
                    winner,
                    left: t::round_over_frames(),
                };
            }
        }
    }

    /// Cheap and portable. Used for desync detection: peers exchange checksums
    /// periodically and a mismatch means the simulations diverged.
    /// Snapshot hash, including the tuning.
    ///
    /// The Oven's values are rules rather than state, so rollback never saves or
    /// restores them -- but two peers running different rules would diverge
    /// silently and look exactly like a netcode bug. Folding the tuning hash in
    /// turns that into a desync on the first frame, which is a error message
    /// rather than a mystery.
    pub fn checksum(&self) -> u64 {
        let mut h = Fnv::new();
        h.write_u64(crate::oven::hash());
        h.write_u32(self.frame);
        for b in &self.bolts {
            match b {
                Some(b) => {
                    h.write_u32(b.owner as u32 + 1);
                    hash_v3(&mut h, &b.pos);
                    hash_v3(&mut h, &b.dir);
                    h.write_i32(b.travelled.raw());
                }
                None => h.write_u32(0),
            }
        }
        for e in &self.effects {
            match e {
                Some(e) => {
                    h.write_u32(e.kind as u32 + 1);
                    h.write_u32(e.owner as u32);
                    h.write_u32(e.class as u32);
                    h.write_u32(e.slot as u32);
                    h.write_u32(e.age as u32);
                    h.write_u32(e.life as u32);
                    h.write_u32(e.struck);
                    h.write_i32(e.banked);
                    hash_v3(&mut h, &e.pos);
                    hash_v3(&mut h, &e.dir);
                }
                None => h.write_u32(0),
            }
        }
        for p in &self.players {
            for v in [p.pos, p.vel, p.facing, p.aim_path.from, p.aim_path.to] {
                h.write_i32(v.x.raw());
                h.write_i32(v.y.raw());
                h.write_i32(v.z.raw());
            }
            h.write_i32(p.health);
            h.write_u32(p.grounded as u32);
            h.write_u32(p.air_dodged as u32);
            h.write_u32(p.slowed as u32);
            h.write_i32(p.slow_mul.raw());
            h.write_u32(p.rooted as u32);
            h.write_u32(p.mechanic_held as u32);
            h.write_u32(p.space_held as u32);
            h.write_u32(p.leap_used as u32);
            h.write_i32(p.slam.raw());
            h.write_u32(p.held_by as u32);
            h.write_u32(p.jump_hold as u32);
            h.write_u32(p.air_stall as u32);
            h.write_u32(p.hit_used as u32);
            h.write_u32(p.crouching as u32);
            h.write_u32(p.stride as u32);
            h.write_u32(p.air_frames as u32);
            h.write_u32(p.since_landed as u32);
            h.write_u32(p.parried as u32);
            h.write_u32(p.crouched_for as u32);
            h.write_u32(p.stun_total as u32);
            h.write_u32(p.action.tag());
            h.write_u32(p.action.frames_left() as u32);
            let kind = match p.action {
                Action::Startup { kind, .. }
                | Action::Active { kind, .. }
                | Action::Recovery { kind, .. } => kind,
                _ => 0,
            };
            h.write_u32(kind as u32);
            h.write_u32(p.rounds_won as u32);
            h.write_u32(p.class as u32);
            h.write_u32(p.mount as u32);
            hash_v3(&mut h, &p.local);
            hash_v3(&mut h, &p.grip_vel);
            h.write_i32(p.carry_yaw.raw());
            h.write_u32(p.grip_settle as u32);
            h.write_i32(p.beam_reach.raw());
            hash_mechanic(&mut h, &p.mechanic);
        }
        match &self.monster {
            None => h.write_u32(0),
            Some(m) => {
                h.write_u32(1);
                hash_v3(&mut h, &m.pos);
                h.write_i32(m.yaw.raw());
                h.write_i32(m.yaw_rate.raw());
                h.write_i32(m.speed.raw());
                h.write_i32(m.health);
                h.write_i32(m.poise);
                for limb in &m.part_health {
                    h.write_i32(*limb);
                }
                h.write_u32(m.doing.tag());
                h.write_u32(m.doing.frames_left() as u32);
                h.write_u32(m.doing.attacking().unwrap_or(0) as u32);
                h.write_u32(m.hit_used as u32);
                hash_v3(&mut h, &m.brain.seen);
                hash_v3(&mut h, &m.brain.seen_vel);
                h.write_u32(m.brain.target as u32);
                h.write_u32(m.brain.glance_left as u32);
                h.write_u32(m.brain.think_left as u32);
                h.write_u32(m.brain.last_move as u32);
                h.write_u32(m.brain.repeat_left as u32);
                h.write_u32(m.brain.rng);
            }
        }
        match self.phase {
            Phase::Fighting => h.write_u32(0),
            Phase::RoundOver { winner, left } => {
                h.write_u32(1);
                h.write_u32(winner as u32);
                h.write_u32(left as u32);
            }
        }
        h.finish()
    }
}

impl Default for World {
    fn default() -> Self {
        World::new()
    }
}

/// One landed attack, whatever threw it.
///
/// `pub(crate)` because a fire bolt is an attack that outlives the move that
/// lit it and applies itself from [`crate::bolt`] -- and it has to arrive
/// through the same door as everything else, or guard would stop a sword and
/// not a bolt.
#[derive(Clone, Copy)]
pub(crate) struct Hit {
    pub damage: i32,
    pub hitstun: u16,
    pub blockstun: u16,
    pub knockback: Fx,
    /// Upward speed handed to the victim. This is what takes someone off the
    /// ground with an uppercut instead of shoving them along it.
    pub launch: Fx,
    /// Frames the victim is held at the attacker's arm's length. Zero is a
    /// normal hit.
    pub grabs: u16,
    /// Who threw it, so a grab knows whose arm to hang from.
    pub by: u8,
    pub dir: V3,
    pub blocked: bool,
    pub parried: bool,
}

/// The attack volume a fighter currently has out.
///
/// Exists so the debug overlay draws *the thing the hit test uses* rather than
/// its own reconstruction of it. An overlay that can drift from the rule it
/// illustrates is worse than no overlay: it is confidently wrong at exactly the
/// moment you are trying to work out why something did not connect.
///
/// A **capsule between two points**, because most of what the game throws is a
/// line -- the Elementalist's beam, and every one of the Champion's weapons --
/// and a bubble is the case where both ends are the same point. Keeping one
/// shape rather than two is what lets the overlay, the creature's hit test and
/// the web tool all stay honest about every kind of attack without each growing
/// a special case.
///
/// `flat` says which rule tests it. A disc at arm's length -- what every move
/// in the game used to be, and what everything outside the Champion's list
/// still is -- is measured in the horizontal plane only, with no top and no
/// bottom. See `moves::Shape`.
#[derive(Clone, Copy, Debug)]
pub struct Hitbox {
    /// Where the volume starts: the hand end of a weapon, and the point
    /// abilities come out of for a beam.
    pub from: V3,
    /// Where it ends. Equal to `from` for a disc.
    pub to: V3,
    /// The attack's radius. A defender is hit when their body overlaps it, so
    /// the test threshold is this plus `BODY_RADIUS`.
    pub radius: Fx,
    /// The old rule: measure flat, and ignore height entirely.
    pub flat: bool,
    /// False means an overhead: it passes over a crouching defender.
    pub hits_crouching: bool,
    pub unblockable: bool,
    /// The move has already connected this swing and cannot connect again.
    /// Still drawn, because it is still visibly out.
    pub spent: bool,
}

impl Hitbox {
    /// The middle of the volume. What a single-point overlay wants to draw at,
    /// what the browser build reports, and the same point as `from` for a disc.
    pub fn centre(&self) -> V3 {
        self.from
            .add(self.to.sub(self.from).scale(Fx::ONE.div(Fx::from_int(2))))
    }

    /// Is this volume a line rather than a bubble?
    pub fn is_a_beam(&self) -> bool {
        self.from != self.to
    }
}

/// The attack volume out this frame, if any. `None` outside active frames, and
/// `None` for a move that has no volume at all -- the pole vault is movement,
/// not an attack.
pub fn hitbox(p: &Player) -> Option<Hitbox> {
    let Action::Active { kind, left } = p.action else {
        return None;
    };
    let m = moves::get(p.class, kind);
    // Two ways to have no volume, and both answer `None` here.
    //
    // A move with no radius is a gesture that puts something into the world,
    // and the thing it put there does all the hitting -- the Blood mage's
    // thrown blade and her Grasp are both that shape. Answering `None` keeps
    // the caster's own body from quietly poking people at point blank while
    // the ability is elsewhere. A move with no *shape* puts nothing anywhere:
    // the Champion's pole vault is movement.
    if !m.strikes() {
        return None;
    }
    // Which of the three ways this move is pointed decides where its volume
    // is. Two of them are skillshots and were solved when the move started;
    // the third is a body moving, and that is where the Champion's weapons
    // live -- see `moves::Shape`.
    let disc = |at: V3| Hitbox {
        from: at,
        to: at,
        radius: m.radius,
        flat: true,
        hits_crouching: m.hits_crouching,
        unblockable: m.unblockable,
        spent: p.hit_used,
    };
    let (from, to, flat) = match m.aim() {
        // A line from her hand to the point the crosshair was on, ending
        // wherever the shot actually stopped.
        aim::Kind::Skillshot => {
            let beam = beam_of(p);
            (beam.from, beam.at(p.beam_reach), false)
        }
        // Where the thing was planted. The burst that comes with it has to be
        // there too, or the ability is two abilities pointing different ways.
        aim::Kind::Grounded => return Some(disc(p.aim_at())),
        // Not aimed at a point -- a body moving. Live rather than locked,
        // because a move that can be thrown on the move has to travel with the
        // body.
        aim::Kind::Swing => match m.shape {
            moves::Shape::None => return None,
            // The original: a disc at arm's length, along the line the
            // swing came out on -- the facing's yaw, the camera's pitch.
            moves::Shape::Cylinder => return Some(disc(p.pos.add(p.aim_dir().scale(m.reach)))),
            moves::Shape::Swing(plane) => {
                // The hand stays near the body and the head of the weapon
                // travels the long arc -- which is how a real swing works, and
                // why the volume is a line from the body rather than a ball at
                // the end of one. A fighter standing inside the arc is caught
                // by the haft.
                let hub = moves::swing_hub(p.pos, p.facing, plane, m.hand);
                let base = moves::swing_base(p.facing, p.aim_dir(), plane, p.grounded);
                let half = m.arc.mul(Fx::ratio(1, 2));
                // Where the head of the weapon is, as a fraction of the way
                // through this swing. A move that re-hits swings once per
                // interval and alternates, which is what makes Rush slash read
                // as a figure of eight cut through a crowd rather than one long
                // smear.
                let (through, back) = swing_progress(&m, left);
                let angle = if back {
                    half.neg().add(m.arc.mul(through))
                } else {
                    half.sub(m.arc.mul(through))
                };
                (
                    hub,
                    hub.add(moves::turned(base, angle, plane).scale(m.reach)),
                    false,
                )
            }
            moves::Shape::Thrust => {
                // The point is already out by the time the move is active; the
                // active frames are the last of the extension. Along the line
                // the swing was committed to rather than along the flattened
                // facing: a spear levelled at somebody below you is the whole
                // reason pitch is on the wire.
                let hub = aim::hand_origin(p.pos, p.facing, m.hand);
                let (through, _) = swing_progress(&m, left);
                let start = t::thrust_extend();
                let out = m.reach.mul(start.add(Fx::ONE.sub(start).mul(through)));
                (hub, hub.add(p.aim_dir().scale(out)), false)
            }
            // A punch that opens into a wing. Both ends move: the tip sweeps
            // outward *and* reaches further out as it goes, and the root rides
            // a little behind the fist the whole way. The two together are
            // what carve the shape -- an arc alone is a swing, and an
            // extension alone is a thrust.
            //
            // Outward is away from the body on whichever side the hand is, so
            // the two mirrored autos share one arc and one set of numbers.
            moves::Shape::Wing => {
                let hub = moves::swing_hub(p.pos, p.facing, moves::Plane::Flat, m.hand);
                let base = moves::swing_base(p.facing, p.aim_dir(), moves::Plane::Flat, p.grounded);
                let (through, _) = swing_progress(&m, left);
                let span = m.arc.mul(Fx::from_int(m.hand.outward()));
                let out = moves::turned(base, span.mul(through), moves::Plane::Flat);
                let open = t::wing_opens_at();
                let tip = m.reach.mul(open.add(Fx::ONE.sub(open).mul(through)));
                (
                    hub.sub(out.scale(m.reach.mul(t::wing_trails()))),
                    hub.add(out.scale(tip)),
                    false,
                )
            }
        },
        // At the mechanic, and *live*: the shadow can be moved while the blades
        // are out, which is the Reaver's own recall, and the volume has to go
        // with it.
        aim::Kind::AtTheMechanic => return Some(disc(p.mechanic.placed().unwrap_or(p.pos))),
    };
    Some(Hitbox {
        from,
        to,
        radius: m.radius,
        flat,
        hits_crouching: m.hits_crouching,
        unblockable: m.unblockable,
        spent: p.hit_used,
    })
}

/// How far through its swing a move is, and whether this swing is the return
/// stroke.
///
/// `left` is the active frames remaining, counting down, so the first active
/// frame is the start of the arc. A move that re-hits restarts the fraction
/// every interval and flips direction each time -- otherwise a twenty-frame
/// active window would draw one slow smear rather than three cuts.
fn swing_progress(m: &moves::Move, left: u16) -> (Fx, bool) {
    let elapsed = m.active.saturating_sub(left);
    let (span, index) = if m.rehit > 0 {
        (m.rehit, elapsed / m.rehit)
    } else {
        (m.active, 0)
    };
    let within = if m.rehit > 0 {
        elapsed % m.rehit
    } else {
        elapsed
    };
    let last = span.saturating_sub(1).max(1);
    let through = Fx::ratio(within.min(last) as i32, last as i32);
    (through, index % 2 == 1)
}

/// The line a skillshot travels this frame.
///
/// **Locked target, live origin.** The point was committed to when the move
/// started -- that is what stops a whiff being rescued by turning afterwards --
/// but it leaves her hand, and her hand moves: the auto keeps most of her
/// walking speed, so a line still anchored where she was standing two frames
/// ago would visibly detach from her.
pub fn beam_of(p: &Player) -> Path {
    Path {
        from: aim::origin(p.pos),
        to: p.aim_path.to,
    }
}

/// What a class's damage is multiplied by against a victim who cannot move.
///
/// One function, used by every path a fighter can deal damage down: a swing, a
/// blade in the air, an arm of a Grasp, a field ticking, and all of the same
/// against the creature. They used to be five separate pieces of arithmetic and
/// this is the kind of rule that is only worth having if it is true everywhere
/// -- a class trait that applies to three of a class's four abilities is not a
/// trait, it is a bug somebody will find in a match.
fn preying(class: Class, victim_disabled: bool) -> Fx {
    if victim_disabled && class.preys_on_the_disabled() {
        t::disabled_damage_mul()
    } else {
        Fx::ONE
    }
}

fn resolve_hit(attacker: &Player, defender: &Player, by: u8) -> Option<Hit> {
    let Action::Active { kind, .. } = attacker.action else {
        return None;
    };
    // The Elementalist's auto already happened. It is a ray fired the moment
    // the move comes out, and what it does depends on what it met first --
    // none of which this loop can express. See `World::fire_the_beam`.
    if bolt::throws_a_beam(attacker, kind) {
        return None;
    }
    let box_out = hitbox(attacker)?;
    if box_out.spent || defender.action.invulnerable() {
        return None;
    }
    let m = moves::get(attacker.class, kind);

    // Overheads miss a crouching defender. Expressed as a property of the move
    // rather than as hitbox geometry, because that is what players read and
    // what a frame table can state.
    if defender.crouching && !box_out.hits_crouching {
        return None;
    }

    let damage_mul = preying(attacker.class, defender.disabled());

    let reach = box_out.radius.add(t::body_radius());
    let hit_at_all = if box_out.flat {
        // The original rule, kept for every move that has not been given a
        // shape: flat distance from the middle of the disc, with no top and no
        // bottom. You cannot duck under one of these or jump over it.
        let delta = defender.pos.sub(box_out.centre());
        delta.flat_len().raw() <= reach.raw()
    } else {
        // A capsule against a standing body. Height genuinely decides this
        // one, which is the whole point of the Champion's air game: a hammer
        // aimed at the floor reaches a fighter beneath you and not one beside
        // you, and a fan thrown level goes over the head of anybody crouched.
        let spine = V3::new(
            defender.pos.x,
            defender.pos.y.add(defender.hurt_height()),
            defender.pos.z,
        );
        crate::math::segment_gap(box_out.from, box_out.to, defender.pos, spine).raw() <= reach.raw()
    };
    if !hit_at_all {
        return None;
    }

    let (guarding, parried) = guard_against(defender, attacker.pos, m.unblockable);

    // A fighter already off the ground has nothing to brace against, so the
    // same swing carries them much further -- and a launch aimed *downward*
    // only means anything to someone who has room to fall. That pair is the
    // Champion's air game stated as two lines: knock them up, follow them up,
    // and the next hit is worth double.
    let airborne = !defender.grounded;
    let air_mul = if airborne && !m.shape.flat() {
        t::air_hit_knockback()
    } else {
        Fx::ONE
    };
    // A spike is wasted on somebody standing on the floor: they are already
    // there. It reads as an ordinary heavy hit instead.
    let launch = if m.launch.raw() < 0 && !airborne {
        Fx::ZERO
    } else {
        m.launch
    };

    Some(Hit {
        damage: Fx::from_int(m.damage).mul(damage_mul).to_int(),
        hitstun: m.hitstun,
        blockstun: m.blockstun,
        knockback: m.knockback.mul(air_mul),
        launch,
        grabs: m.grabs,
        by,
        dir: attacker.facing,
        blocked: guarding,
        parried,
    })
}

/// Is this defender guarding against something arriving from `from`, and did
/// they raise it late enough to parry?
///
/// Guard covers an arc, not a bubble: turning your back on an attack is not
/// blocking it. A grapple ignores the whole question, which is what stops
/// blocking from being a solved strategy -- see defense.md.
///
/// Shared rather than inlined because there are now three kinds of attack --
/// a swing, a beam, and a bolt in flight -- and a guard that stopped two of
/// them would be worse than one that stopped none.
pub(crate) fn guard_against(defender: &Player, from: V3, unblockable: bool) -> (bool, bool) {
    if unblockable {
        return (false, false);
    }
    let toward = from.sub(defender.pos).normalized();
    let facing_it = defender.facing.dot(toward).raw() >= t::guard_arc_cos().raw();
    let guarding = defender.action.guarding() && facing_it;
    let parried =
        facing_it && matches!(defender.action, Action::Guard { held } if held < t::parry_window());
    (guarding, parried)
}

pub(crate) fn apply_hit(defender: &mut Player, hit: Hit) {
    if hit.parried {
        // The parry itself costs the defender nothing. The attacker eats the
        // stagger, which the caller applies.
        return;
    }
    if hit.blocked {
        // No chip damage. The cost of blocking is knockback plus a window
        // where you cannot act -- see defense.md.
        defender.stun_total = hit.blockstun;
        defender.action = Action::BlockStun {
            left: hit.blockstun,
        };
        defender.vel.x = hit.dir.x.mul(hit.knockback);
        defender.vel.z = hit.dir.z.mul(hit.knockback);
    } else {
        defender.health = (defender.health - hit.damage).max(0);
        defender.vel.x = hit.dir.x.mul(hit.knockback);
        defender.vel.z = hit.dir.z.mul(hit.knockback);
        if hit.grabs > 0 {
            // A grab is not knockback. The victim is pinned to the grabber and
            // goes wherever they go, which is what makes a grab a commitment
            // for *both* of them rather than a shove with a longer stun.
            defender.stun_total = hit.grabs;
            defender.action = Action::Held { left: hit.grabs };
            defender.held_by = hit.by;
            defender.vel.x = Fx::ZERO;
            defender.vel.z = Fx::ZERO;
        } else {
            defender.stun_total = hit.hitstun;
            defender.action = Action::HitStun { left: hit.hitstun };
        }
        // Anything that is not itself a spike clears one that was banked: being
        // caught on the way down and thrown somewhere else is not the landing
        // the spike was charging for.
        defender.slam = Fx::ZERO;
        if hit.launch.raw() > 0 {
            // Taken off the ground. Launch *sets* vertical speed rather than
            // adding to it, so being hit on the way down does not cancel out.
            defender.vel.y = hit.launch;
            defender.grounded = false;
        } else if hit.launch.raw() < 0 {
            // Spiked. The same rule pointed the other way, and the speed is
            // banked so that the landing can charge for it -- see
            // `Player::slam`. Terminal velocity is what the ground actually
            // sees, so banking the intent rather than the arrival would
            // overstate it.
            defender.vel.y = hit.launch;
            defender.grounded = false;
            defender.slam = hit.launch.abs();
        }
    }
    // Whatever else it did, it broke the dash. A Rush you could be hit out of
    // and keep is a Rush with invulnerability attached.
    if let Mechanic::Forms { rush, .. } = &mut defender.mechanic {
        *rush = 0;
    }
}

/// Turn a stick reading into a world direction, given where the player looks.
///
/// `W` is away from the camera, `D` is to its right. This is the whole of
/// "camera-relative movement", and it is why aim has to be simulation input:
/// the same button means a different direction depending on it.
pub fn move_dir(aim: Fx, ax: i32, az: i32) -> V3 {
    if ax == 0 && az == 0 {
        return V3::ZERO;
    }
    let forward = V3::from_turns(aim);
    let right = V3::from_turns(aim.add(QUARTER_TURN));
    forward
        .scale(Fx::from_int(az))
        .add(right.scale(Fx::from_int(ax)))
        .normalized()
}

/// A quarter turn in `Fx`, matching `Input::QUARTER_TURN`.
const QUARTER_TURN: Fx = Fx::from_raw(1 << 14);

/// Advance whatever the fighter is already doing.
///
/// `None` means they are free to choose, which is the one branch that differs
/// between standing on the floor and standing on a creature -- the countdowns
/// do not. Splitting it here is what stops the ride being a second, drifting
/// copy of the action machine.
fn countdown(p: &mut Player, want_guard: bool) -> Option<Action> {
    Some(match p.action {
        Action::Free => return None,
        Action::Dodge { left } if left > 0 => Action::Dodge { left: left - 1 },
        Action::Dodge { .. } => Action::Free,
        Action::Startup { kind, left } if left > 0 => Action::Startup {
            kind,
            left: left - 1,
        },
        Action::Startup { kind, .. } => Action::Active {
            kind,
            left: moves::get(p.class, kind).active,
        },
        Action::Active { kind, left } if left > 0 => Action::Active {
            kind,
            left: left - 1,
        },
        Action::Active { kind, .. } => Action::Recovery {
            kind,
            left: moves::get(p.class, kind).recovery,
        },
        Action::Recovery { kind, left } if left > 0 => Action::Recovery {
            kind,
            left: left - 1,
        },
        Action::Recovery { .. } => Action::Free,
        Action::BlockStun { left } if left > 0 => Action::BlockStun { left: left - 1 },
        Action::HitStun { left } if left > 0 => Action::HitStun { left: left - 1 },
        Action::Stagger { left } if left > 0 => Action::Stagger { left: left - 1 },
        Action::Held { left } if left > 0 => Action::Held { left: left - 1 },
        Action::Held { .. } => {
            p.held_by = NOBODY;
            Action::Free
        }
        Action::BlockStun { .. } | Action::HitStun { .. } | Action::Stagger { .. } => Action::Free,
        Action::Guard { held } => {
            if want_guard {
                Action::Guard {
                    held: held.saturating_add(1),
                }
            } else {
                Action::Free
            }
        }
    })
}

fn step_player(
    p: &mut Player,
    who: usize,
    input: Input,
    field: &Field,
    beast: Option<&Monster>,
    scene: &Scene,
    carrying: bool,
) {
    // Standing on the creature is a different tick: no gravity, no arena, and
    // movement that happens in the animal's frame rather than the world's.
    if p.aboard() {
        match beast {
            Some(beast) => return step_rider(p, who, input, beast, scene),
            // The creature is gone. Whatever you were standing on is not there
            // any more, so neither are you.
            None => p.mount = monster::NO_PART,
        }
    }
    // Facing comes from the mouse. Where you look is where you are pointed, and
    // where you are pointed is where your attacks go.
    //
    // Two exceptions, and both are load-bearing:
    //
    // Once a move has started, facing is **locked**. Otherwise the mouse would
    // drag a live hitbox around during its active frames, and a whiff could be
    // rescued by turning after the fact -- which would take whiff punishment,
    // most of the game, out behind the shed. Commitment is spatial here; you
    // commit to a direction when you commit to the move.
    //
    // While guarding, facing turns at a limited rate. Guard covers an arc, not
    // a bubble (see defense.md), and an arc you can flip instantly is a bubble
    // with extra steps. The camera still snaps wherever the mouse goes -- it is
    // the character who cannot reorient that fast.
    // The mechanic fires on the **press**, not while the button is down. Held,
    // it used to re-fire every frame: the Champion's form became a function of
    // how many frames you happened to hold it for, the Reaver's shadow toggled
    // itself back off, the Bulwark's shield was pinned mid-throw and never
    // planted, and the Elementalist spent all three structures on one spot in
    // three frames. A button whose meaning depends on how long you hold it is a
    // button you cannot use.
    // The mechanic fires on the **press**, not while the button is down. Held,
    // it used to re-fire every frame: the Champion's form became a function of
    // how many frames you happened to hold it, the Reaver's shadow toggled
    // itself back off, the Bulwark's shield was pinned mid-throw and never
    // planted, and the Elementalist spent all three structures on one spot in
    // three frames. A button whose meaning depends on how long you hold it is a
    // button you cannot use.
    //
    // The previous frame's state lives on the fighter rather than in a
    // renderer-side "just pressed", because rollback re-runs these frames: the
    // edge has to be recomputed from the snapshot, not remembered outside it.
    let pressed_mechanic = input.has(Input::MECHANIC) && !p.mechanic_held;
    p.mechanic_held = input.has(Input::MECHANIC);
    // The jump button's press edge, for the one thing that needs one. The
    // ordinary jump stays level-triggered -- holding space through a landing
    // should hop again -- but the uppercut's extra leap is worth height, and a
    // held button would spend it on the first frame it was available rather
    // than on the frame the player chose.
    let pressed_space = input.has(Input::SPACE) && !p.space_held;
    p.space_held = input.has(Input::SPACE);

    let look = V3::from_turns(p.aim(input));
    if p.action.actionable() || p.action.stunned() {
        p.facing = look;
    } else if p.action.guarding() {
        p.facing = p
            .facing
            .add(look.sub(p.facing).scale(t::guard_turn_rate()))
            .normalized();
    }

    let mob = p.class.mobility();
    step_mechanic(p);

    let want_guard = input.has(Input::RIGHT) && p.shield().is_some_and(|sh| sh.in_hand());
    let (ax, az) = input.move_axis();
    // Crouch is a stance, not an action: it holds while the key is down and
    // only while you are otherwise free to move.
    p.crouching = input.has(Input::CROUCH) && p.grounded && p.action.actionable();

    // **Rush is the universal cancel.** Spent from a recovery it ends the
    // recovery, which is the Champion's whole answer to having committed to the
    // wrong move -- and, more often, what turns two moves into a combo. One
    // charge means that is always a decision. See `docs/design/champion.md`.
    if pressed_mechanic && matches!(p.action, Action::Recovery { .. }) && start_rush(p, input) {
        p.action = Action::Free;
    }

    p.action = match countdown(p, want_guard) {
        Some(next) => next,
        None => {
            // Clicks are checked before the dodge, which is what disambiguates
            // shift. Shift with a click is the stronger version of that attack;
            // shift with only a direction is a dodge. See controls.md.
            //
            // The Champion has no special: its identity is the three weapons on
            // the three clicks, and `Q` is free.
            if input.has(Input::SPECIAL)
                && p.class != Class::Champion
                && p.mechanic_ready(SLOT_SPECIAL)
            {
                begin_move(p, who, SLOT_SPECIAL, input, scene, true)
            } else if pressed_mechanic {
                // `E` is the class mechanic, and on most classes that is an
                // instant change of state with no frames to it -- throw the
                // shield, Rush, place the shadow, raise a structure. Where the
                // class puts an ability there instead -- the Blood mage, whose
                // mechanic is health and so has nothing to toggle -- it is
                // thrown like any other move, with a startup you can be
                // punished during and a cost you pay on the press.
                match moves::on_e(p.class).filter(|slot| p.mechanic_ready(*slot)) {
                    Some(slot) => begin_move(p, who, slot, input, scene, true),
                    None => {
                        mechanic_action(p, who, input, scene);
                        Action::Free
                    }
                }
            }
            // Which move a click asks for. After the mechanic, so that Rush
            // can be started while a click is held down, and before the dodge,
            // because a click is what disambiguates shift.
            else if let Some(kind) = clicked_move(p, input).filter(|k| p.mechanic_ready(*k)) {
                // A no-op for anybody who is not the Champion; the weapon in
                // hand and the dash underneath it are that class's alone.
                begin_champion(p, kind);
                begin_move(p, who, kind, input, scene, true)
            } else if input.has(Input::SHIFT)
                && !input.any_click()
                && (ax != 0 || az != 0)
                && !p.is_rooted()
            {
                // Shift plus a direction dodges. It used to be space plus a
                // direction, which meant that pressing the jump button while
                // moving -- which is most of the time -- did not jump. Space is
                // now only ever a vertical takeoff.
                let dir = move_dir(p.aim(input), ax, az);
                if p.grounded {
                    // The Reaver's forward dodge, thrown with the crosshair on
                    // her shadow, is the dash to it -- the same invulnerable
                    // commitment, pointed at the one place on the map she cares
                    // about. It is not an extra input: the class's mobility and
                    // the universal defensive option are deliberately the same
                    // button, which is what keeps her from being denied either.
                    if shadow::dash_is_asked_for(p, who, input, az > 0, scene) {
                        shadow::begin_dash(p);
                    } else {
                        p.vel.x = dir.x.mul(t::dodge_speed());
                        p.vel.z = dir.z.mul(t::dodge_speed());
                    }
                    Action::Dodge {
                        left: t::dodge_frames(),
                    }
                } else if !p.air_dodged {
                    // An airdodge, once per airtime. It commits you to a
                    // direction in the air, where you otherwise have almost no
                    // say, which is why it can only be spent once: a second one
                    // would turn a jump into flight.
                    p.air_dodged = true;
                    p.vel.x = dir.x.mul(t::air_dodge_speed());
                    p.vel.z = dir.z.mul(t::air_dodge_speed());
                    // Vertical speed is wiped rather than added to, so an
                    // airdodge is a sideways commitment and never a second jump.
                    p.vel.y = Fx::ZERO;
                    Action::Dodge {
                        left: t::air_dodge_frames(),
                    }
                } else {
                    Action::Free
                }
            } else if want_guard {
                Action::Guard { held: 0 }
            } else {
                Action::Free
            }
        }
    };

    rearm_multihit(p);

    // Horizontal movement.
    //
    // How much a move hinders you is proportional to how much it commits you.
    // The heavy moves root you outright -- that is what commitment means, and
    // it is the whole basis of spatial play here. A fast poke does not: it is
    // the neutral tool, thrown constantly, and stopping dead every time made
    // neutral sticky and read as the game taking the controls away.
    let attack_speed = p
        .action
        .attack_kind()
        .map(|kind| moves::get(p.class, kind).mobility)
        .filter(|m| *m > 0)
        .map(|m| t::move_speed().mul(Fx::ratio(m as i32, 100)));
    let steering = ax != 0 || az != 0;

    if let Some(drive) = shadow::dash_drive(p) {
        // A dash has somewhere to be, so it holds its speed rather than
        // decaying like the dodge it rides on: a decaying shove covers whatever
        // distance the decay happens to be tuned for, and the shadow is at a
        // distance of its own choosing.
        p.vel.x = drive.x;
        p.vel.z = drive.z;
    } else if matches!(p.action, Action::Dodge { .. }) {
        p.vel.x = p.vel.x.mul(t::dodge_decay());
        p.vel.z = p.vel.z.mul(t::dodge_decay());
    } else if p.action.stunned() {
        // Knockback decays rather than stopping dead, in the air as on the
        // ground. Checked before the airborne branch so a hit connecting
        // mid-jump is not immediately steered out of.
        p.vel.x = p.vel.x.mul(t::stun_decay());
        p.vel.z = p.vel.z.mul(t::stun_decay());
    } else if let Some(drive) = rushing(p) {
        // A dash holds its line. Facing follows the mouse whenever you are
        // free to act, so a Rush steered by where you happen to be looking
        // would be a turn rather than a dash -- and the Rush moves are aimed
        // *across* the line you are running, which only means anything if the
        // line stays put. The attack thrown out of it does not slow it: that
        // is what "cast while rushing" has to mean for the sword's run-through
        // to be the move it is meant to be.
        p.vel.x = drive.x;
        p.vel.z = drive.z;
    } else if !p.grounded {
        // In the air, input *accelerates* rather than assigns. Momentum is
        // conserved when you let go, which is the whole difference between a
        // jump being a commitment and a jump being a hover.
        if steering {
            air_accelerate(p, move_dir(p.aim(input), ax, az), mob.air_speed);
        }
    } else if p.is_rooted() && p.grounded {
        // Pinned. Not stunned: the arms are around your legs, so you can still
        // turn, guard and swing at whoever put them there.
        p.vel.x = Fx::ZERO;
        p.vel.z = Fx::ZERO;
    } else if p.action.actionable() && steering {
        let speed = if p.crouching {
            t::crouch_move_speed()
        } else {
            t::move_speed()
        };
        let dir = move_dir(p.aim(input), ax, az);
        p.vel.x = dir.x.mul(dragged(p, speed));
        p.vel.z = dir.z.mul(dragged(p, speed));
    } else if p.action.guarding() && steering {
        let dir = move_dir(p.aim(input), ax, az);
        p.vel.x = dir.x.mul(dragged(p, t::guard_move_speed()));
        p.vel.z = dir.z.mul(dragged(p, t::guard_move_speed()));
    } else if let (Some(speed), true) = (attack_speed, steering) {
        let dir = move_dir(p.aim(input), ax, az);
        p.vel.x = dir.x.mul(dragged(p, speed));
        p.vel.z = dir.z.mul(dragged(p, speed));
    } else if p.action.attack_kind().is_some() {
        // Rooted, or steering nothing. Bleed the speed off over a few frames
        // rather than snapping to a halt: the snap was the jarring part, not
        // the rooting.
        p.vel.x = p.vel.x.mul(t::attack_root_decay());
        p.vel.z = p.vel.z.mul(t::attack_root_decay());
    } else {
        p.vel.x = Fx::ZERO;
        p.vel.z = Fx::ZERO;
    }

    // Space is a vertical takeoff, whatever your feet are doing. Holding a
    // direction while jumping carries your momentum up with you; it does not
    // turn the jump into something else.
    //
    // Added to whatever vertical speed you already have rather than replacing
    // it, for the same reason leaving the creature's back does (see
    // `step_rider`): a stone mid-eruption is already carrying you upward
    // (`stones::resolve_body`), and a jump off it should stack with that
    // rather than reset it to a flat takeoff speed. Ordinary ground has
    // nothing to stack with -- standing still on it is vertical speed zero --
    // so this changes nothing there.
    //
    // A root takes the jump away as well as the walk. That is what separates it
    // from a very heavy slow, and it is why it is gated behind landing every
    // arm of a Grasp rather than being handed out for one.
    if input.has(Input::SPACE) && p.grounded && p.action.actionable() && !p.is_rooted() {
        p.vel.y = p.vel.y.add(t::jump_speed().mul(mob.jump));
        p.grounded = false;
        p.jump_hold = t::jump_hold_frames();
    }

    // The second half of the uppercut. You are both off the ground and you
    // have hold of them; pressing jump again takes the pair of you higher,
    // once. This is the "we are settling this in the air" button, and it is the
    // only thing space does while airborne.
    if pressed_space && carrying && !p.leap_used && !p.grounded {
        p.vel.y = p.vel.y.add(t::uppercut_leap());
        p.leap_used = true;
    }

    if !p.grounded {
        if p.air_stall > 0 {
            // An aerial hangs you for a few frames: gravity is held off, and
            // whatever vertical speed you had **bleeds away** rather than being
            // deleted.
            //
            // Deleting it read as the game snatching the jump out from under
            // you mid-rise. The extra control over jump height that attacking
            // gives is worth keeping -- it just has to arrive as a slowing
            // rather than a stop, which is also what makes the hang read as
            // float rather than a pause. Gravity staying off through the window
            // is what keeps it punchy: you hang, you do not sag.
            p.air_stall -= 1;
            p.vel.y = p.vel.y.mul(t::air_stall_damp());
        } else {
            // Holding the jump button sustains the rise. Releasing ends it for
            // good -- `jump_hold` goes to zero rather than pausing, so tapping
            // again cannot resurrect a jump you already cut short.
            let sustaining = input.has(Input::SPACE) && p.jump_hold > 0 && p.vel.y.raw() > 0;
            // Letting go while still rising cuts what is left of the climb,
            // once. `jump_hold` drops to zero immediately after, so a second
            // press can neither cut again nor resurrect the jump.
            if !sustaining && p.jump_hold > 0 && p.vel.y.raw() > 0 {
                p.vel.y = p.vel.y.mul(t::jump_release_cut());
            }
            if sustaining {
                p.jump_hold -= 1;
            } else {
                p.jump_hold = 0;
            }
            let mut gravity = t::gravity().mul(mob.gravity);
            if sustaining {
                gravity = gravity.mul(t::jump_hold_gravity());
            }
            p.vel.y = p.vel.y.add(gravity.mul(DT));
            let floor = t::fall_cap().mul(mob.fall_cap);
            if p.vel.y.raw() < floor.raw() {
                p.vel.y = floor;
            }
        }
    }

    p.pos = p.pos.add(p.vel.scale(DT));

    // How fast this fighter was going when the floor arrived. Read before the
    // resolve, which is what stops it.
    let impact = p.vel.y;
    let was_grounded = p.grounded;
    let r = arena::resolve(p.pos, p.vel, was_grounded);
    let r = stones::resolve_body(field, r.pos, r.vel, r.grounded, was_grounded);
    p.pos = r.pos;
    p.vel = r.vel;
    p.grounded = r.grounded;
    if let Some(beast) = beast {
        meet_the_creature(p, beast);
    }
    if p.grounded {
        // Driven into the floor. The hit that spiked them did its damage in
        // the air; this is the ground collecting the rest, and it is most of
        // why knocking someone down is worth more than knocking them away.
        if p.slam.raw() > 0 && !was_grounded {
            let cost = Fx::from_int(t::slam_damage()).mul(impact.abs()).to_int();
            p.health = (p.health - cost).max(0);
            p.stun_total = t::slam_stagger();
            p.action = Action::Stagger {
                left: t::slam_stagger(),
            };
        }
        p.slam = Fx::ZERO;
        p.air_dodged = false;
        p.jump_hold = 0;
        p.air_stall = 0;
        p.leap_used = false;
    }
}

/// Which move a click asks for, if any.
///
/// **The one place a mouse button becomes a move.** Three classes' worth of
/// grammar meet here and they are deliberately different shapes:
///
/// ```text
///   most classes   left click is the poke, shift + left the committed version
///   the Champion   three buttons are three weapons, and the row of the grid
///                  is where your feet are -- see `moves::champion`
///   the Dual mage  left and right are two *different* autos, one per arm,
///                  because the button is which force you throw -- see
///                  `moves::dual`
///   the Reaver     right click sends the shadow, because the shadow is the
///                  thing the crosshair aims -- see `moves::on_e`
/// ```
///
/// It is a function rather than a chain of branches in `step_player` because
/// the answer is a property of the class's kit, and the three classes that
/// broke the shared rule each broke it in their own way: the Champion by
/// needing a third button, the Dual mage by needing right click to be a
/// different attack, and the Reaver by needing it to be the *aimed* one.
///
/// All three start from the same observation: **right click is dead weight on
/// a class with no shield**, and three of the six have no shield.
///
/// Right click means **guard** on every class that has a shield, and that is
/// handled by the caller: `want_guard` asks the mechanic, not the button.
fn clicked_move(p: &Player, input: Input) -> Option<u8> {
    match p.class {
        Class::Champion => champion_move(p, input),
        Class::DualMage => dual_move(input),
        // The Reaver breaks it a third way: right click sends the shadow. It is
        // the one thing in her kit the **crosshair aims**, and the mouse is
        // where aiming lives -- so the mechanic is on the mouse and the swing
        // it displaced went to `E`, which is the one key that does not care
        // where anything is pointed. See `moves::on_e`.
        Class::ShadowReaver if input.has(Input::RIGHT) => Some(SLOT_MECHANIC),
        _ => input.has(Input::LEFT).then(|| {
            if input.has(Input::SHIFT) {
                SLOT_COMMITTED
            } else {
                SLOT_POKE
            }
        }),
    }
}

/// Which of the Dual mage's five a click asks for.
///
/// Left first, so that both buttons at once throws the dark auto rather than
/// nothing. The design has a use for both-click -- a finisher with no side --
/// and does not have one yet; until it does, an accidental double press should
/// come out as an attack rather than as silence.
///
/// **Shift plus right click is the light auto, unmodified.** The kit wants a
/// light *form* of the committed cast there and there is not one built, so the
/// modifier is ignored rather than being made to mean something it does not.
/// See `docs/design/kits/dual-mage.md`.
fn dual_move(input: Input) -> Option<u8> {
    use moves::dual as d;
    if input.has(Input::LEFT) {
        return Some(if input.has(Input::SHIFT) {
            d::LANCE
        } else {
            d::DARK_AUTO
        });
    }
    input.has(Input::RIGHT).then_some(d::LIGHT_AUTO)
}

// ---------------------------------------------------------------------------
// The Champion
// ---------------------------------------------------------------------------
//
// Three weapons on three mouse buttons, and a dash that changes what all three
// of them do. The move list is a grid -- see `moves::champion` -- and what
// follows is everything that reads it: which move a click asks for, what
// throwing one does to the dash, what the dash does to your feet, how a dash
// begins, and the two consequences of swinging a shape rather than a disc.

/// Which of the Champion's ten moves this click asks for, if any.
///
/// `None` for every other class, which is what keeps the shared grammar in
/// `step_player` unchanged for the five of them. The order of the three tests
/// is the design: **rushing beats airborne beats standing**, because the Rush
/// moves are the ones you spent a charge to reach and they should not be taken
/// away by the fact that the uppercut has already left the floor.
fn champion_move(p: &Player, input: Input) -> Option<u8> {
    use moves::champion as c;
    if p.class != Class::Champion {
        return None;
    }
    let weapon = if input.has(Input::LEFT) {
        c::SWORD
    } else if input.has(Input::MIDDLE) {
        c::HAMMER
    } else if input.has(Input::RIGHT) {
        c::SPEAR
    } else {
        return None;
    };
    if rushing(p).is_some() {
        // Right click during a Rush is two moves told apart by where you are
        // pointing, which is the honest separator: a pole vault *is* a spear
        // put into the ground, and levelling it at somebody is a stab.
        if weapon == c::SPEAR && input.pitch_turns().raw() <= t::vault_pitch().neg().raw() {
            return Some(c::POLE_VAULT);
        }
        return Some(c::RUSHING + weapon);
    }
    Some(if p.grounded {
        c::ON_FOOT + weapon
    } else {
        c::IN_THE_AIR + weapon
    })
}

/// What throwing one of the Champion's moves does to the weapon in hand and to
/// the dash underneath it.
fn begin_champion(p: &mut Player, kind: u8) {
    use moves::champion as c;
    let Mechanic::Forms {
        rush,
        recharge,
        rush_vel,
        ..
    } = p.mechanic
    else {
        return;
    };
    // The weapon is whichever button was pressed. Nothing else sets it: there
    // is no mode to be in any more.
    let form = Form::of_move(kind);
    let (rush, rush_vel) = match kind {
        // Planting the spear spends the run on height. The dash is kept alive
        // for exactly as long as the plant takes so the approach does not stop
        // dead underneath it, and slowed, because a vault that kept its whole
        // run would be a jump with extra steps.
        c::POLE_VAULT => {
            let m = moves::get(p.class, kind);
            (m.startup + m.active, rush_vel.scale(t::vault_carry()))
        }
        // The stab is the one Rush move that *stops*. All of the speed goes
        // into the point, which is what buys it the damage.
        c::RUSH_STAB => (0, V3::ZERO),
        // Everything else rides the dash out.
        _ => (rush, rush_vel),
    };
    if kind == c::UPPERCUT {
        p.leap_used = false;
    }
    p.mechanic = Mechanic::Forms {
        form,
        rush,
        recharge,
        rush_vel,
    };
}

/// Which part of the creature a fighter's attack volume touches.
///
/// The creature's own test takes a point and a height above it, so each kind of
/// volume is handed over as what it actually is.
///
/// A disc has no height of its own and never had: a flat spot at arm's length,
/// a whole body tall, exactly as it was before capsules existed. A **capsule
/// is a line**, so it is asked down its length, a weapon's thickness deep at
/// each point, and the softest answer wins -- which is the same rule the
/// creature already uses when one volume covers two parts, applied one step
/// earlier.
///
/// The consequence is worth stating because it is the design rather than the
/// implementation: a sword swept across at chest height passes **over** a low
/// ridge, and a hammer brought down on it does not. Breaking the Ridgeback's
/// poise is a matter of putting the head of the right weapon on the weak point,
/// not of standing next to it and swinging.
fn part_under(beast: &Monster, attacker: &Player, box_out: &Hitbox) -> Option<usize> {
    if box_out.flat {
        return beast.part_struck(box_out.centre(), box_out.radius, attacker.hurt_height());
    }
    const SAMPLES: i32 = 5;
    let thick = box_out.radius;
    let mut best: Option<usize> = None;
    for i in 0..=SAMPLES {
        let at = crate::math::lerp3(box_out.from, box_out.to, Fx::ratio(i, SAMPLES));
        let foot = V3::new(at.x, at.y.sub(thick), at.z);
        if let Some(part) = beast.part_struck(foot, thick, thick.add(thick)) {
            // Softest wins, the creature's own rule: a swing that reaches a
            // leg with its haft and the ridge with its head has hit the ridge.
            best = Some(match best {
                Some(seen)
                    if monster::vulnerability(seen).raw() >= monster::vulnerability(part).raw() =>
                {
                    seen
                }
                _ => part,
            });
        }
    }
    best
}

/// The shove the aerial spear's fan gives the Champion when it lands.
///
/// A no-op for anything that is not that move, so the caller can apply it to
/// every hit without asking.
fn champion_fan_boost(p: &mut Player, input: Input) {
    let Some(kind) = p.action.attack_kind() else {
        return;
    };
    if p.class != Class::Champion || kind != moves::champion::AIR_SPEAR {
        return;
    }
    let (ax, az) = input.move_axis();
    // Holding nothing still gets you something -- forward, along the line the
    // fan was thrown down. A move whose reward you can fail to collect by not
    // touching a key is a move that feels broken rather than demanding.
    let dir = if ax == 0 && az == 0 {
        V3::new(p.facing.x, Fx::ZERO, p.facing.z)
    } else {
        move_dir(p.aim(input), ax, az)
    };
    let boost = t::spear_fan_boost();
    p.vel.x = p.vel.x.add(dir.x.mul(boost));
    p.vel.z = p.vel.z.add(dir.z.mul(boost));
    clamp_air_speed(p);
}

/// Let a move that keeps hitting hit again.
///
/// `hit_used` is what makes a swing land once. A move with a re-hit interval
/// puts it back every interval instead, which is the whole implementation of
/// running a sword through a crowd: one long active window, and the rule that
/// usually closes it after the first contact relaxed to a rhythm.
fn rearm_multihit(p: &mut Player) {
    let Action::Active { kind, left } = p.action else {
        return;
    };
    let m = moves::get(p.class, kind);
    if m.rehit == 0 {
        return;
    }
    if m.active.saturating_sub(left) % m.rehit == 0 {
        p.hit_used = false;
    }
}

/// The velocity a dash is driving this frame, or `None` if there is no dash.
fn rushing(p: &Player) -> Option<V3> {
    match p.mechanic {
        Mechanic::Forms { rush, rush_vel, .. } if rush > 0 => Some(rush_vel),
        _ => None,
    }
}

/// Spend the charge and start the dash. False if there was no charge to spend.
///
/// The dash goes where you are **holding**, not where you are looking, so that
/// Rush is a retreat and a sidestep as well as an approach -- and so that the
/// run-through can be aimed across an opponent rather than only at one.
fn start_rush(p: &mut Player, input: Input) -> bool {
    let Mechanic::Forms {
        form, recharge: 0, ..
    } = p.mechanic
    else {
        return false;
    };
    let (ax, az) = input.move_axis();
    let dir = if ax == 0 && az == 0 {
        p.facing
    } else {
        move_dir(p.aim(input), ax, az)
    };
    p.mechanic = Mechanic::Forms {
        form,
        rush: t::rush_frames(),
        // The charge is gone until the dash has finished *and* the recharge has
        // run, so the two never overlap and "rush ready" on the HUD means it.
        recharge: t::rush_frames() + t::rush_recharge(),
        rush_vel: V3::new(
            dir.x.mul(t::rush_speed()),
            Fx::ZERO,
            dir.z.mul(t::rush_speed()),
        ),
    };
    true
}

/// Start a move: commit to where it is aimed, pay what it costs, and enter the
/// startup frames.
///
/// One function for all four buttons, because everything here is a property of
/// *starting a move* rather than of which key started it. `aerial` is the one
/// difference: a rider has no airtime to arm.
fn begin_move(
    p: &mut Player,
    who: usize,
    kind: u8,
    input: Input,
    scene: &Scene,
    aerial: bool,
) -> Action {
    p.hit_used = false;
    lock_aim(p, who, kind, input, scene);
    // The second body throws the same thing a few frames later. A no-op for
    // every class but one, and for the two of the Reaver's four moves that are
    // already the shadow's own -- see `shadow::begin_echo`.
    shadow::begin_echo(p, kind);
    // Committing to a cast is committing to a side, for the one class where
    // that is the mechanic. The autos are the exception and steer on contact
    // instead -- a whiff steers nothing, which is what makes closing to melee
    // the fast way back toward centre. See `steer_meter`.
    if !steers_on_contact(p.class, kind) {
        steer_meter(p, kind);
    }
    if aerial {
        arm_aerial(p, kind, input);
    }
    let m = moves::get(p.class, kind);
    // Health is spent on the press, never on the hit. Missing is the
    // punishment, which is the whole of the Blood mage's economy -- see
    // `docs/design/kits/blood-mage.md`.
    p.spend_health(m.cost);
    Action::Startup {
        kind,
        left: m.startup,
    }
}

/// Work out where this move goes, and hold it there for the move's duration.
///
/// The same commitment facing is: once the move is out, the mouse moves the
/// camera and not the ability. See `Player::aim_path`.
///
/// **Three lines, and none of them decide anything.** Which of the three kinds
/// of aiming a move uses comes from the move table (`Move::aim`), and what each
/// kind means comes from `crate::aim`. A move that needs some fourth answer
/// needs `crate::aim` to grow it, not a branch here -- deciding locally is how
/// the crosshair and the ability came to disagree, three times.
fn lock_aim(p: &mut Player, who: usize, kind: u8, input: Input, scene: &Scene) {
    let m = moves::get(p.class, kind);
    p.aim_path = match m.aim() {
        aim::Kind::Grounded => aim::grounded_path(who, input, m.reach, scene),
        aim::Kind::Skillshot => aim::skillshot_path(who, input, m.reach, scene),
        // Not aimed at anything -- a body moving. What it commits to is the
        // **plane** it swings in: the yaw is the facing, which is locked
        // already, and the pitch is the rest of the same look, dead-zoned so
        // that looking slightly down at somebody does not tilt the swing into
        // the floor. The Champion's weapons read the plane; a disc-shaped
        // swing only reads the direction. The dead zone is a standing rule:
        // off the ground you are above what you are hitting, and the swing
        // follows the camera the whole way.
        aim::Kind::Swing => aim::swing_path(p.pos, p.facing, input, p.grounded, m.reach, m.hand),
        aim::Kind::AtTheMechanic => aim::mechanic_path(p.pos, &p.mechanic),
    };
}

/// Start an aerial's hang, and its shove, if this one is thrown in the air.
///
/// The shove is the basic attack's alone. A poke is the move you throw
/// constantly, so a small push in the direction you are holding is what makes
/// attacking *part of* air movement rather than a pause in it: the hang
/// supplies the float, the shove supplies the punch. The committed moves
/// deliberately get none -- they are already a commitment, and one that also
/// repositioned you would be strictly better than a poke.
fn arm_aerial(p: &mut Player, kind: u8, input: Input) {
    if p.grounded {
        return;
    }
    p.air_stall = moves::get(p.class, kind).air_stall;

    let (ax, az) = input.move_axis();
    // The class's fast button, which for the Champion in the air is the sword.
    // Its hammer is the committed one and its spear pays for the shove by
    // having to connect first -- see the fan's boost in `advance`.
    let fast =
        kind == SLOT_POKE || (p.class == Class::Champion && kind == moves::champion::AIR_SWORD);
    if !fast || (ax == 0 && az == 0) {
        return;
    }
    let dir = move_dir(p.aim(input), ax, az);
    let boost = t::air_attack_boost();
    p.vel.x = p.vel.x.add(dir.x.mul(boost));
    p.vel.z = p.vel.z.add(dir.z.mul(boost));
    clamp_air_speed(p);
}

/// Hold horizontal air speed under the ceiling. See `tuning::air_speed_cap`.
fn clamp_air_speed(p: &mut Player) {
    let cap = t::move_speed().mul(t::air_speed_cap());
    let speed = V3::new(p.vel.x, Fx::ZERO, p.vel.z).flat_len();
    if speed.raw() > cap.raw() {
        let scale = cap.div(speed);
        p.vel.x = p.vel.x.mul(scale);
        p.vel.z = p.vel.z.mul(scale);
    }
}

/// Quake-style air acceleration, which is where air control gets its skill
/// ceiling.
///
/// The whole trick is one line: `current` is the velocity **projected onto the
/// direction you asked for**. Point where you are already going and the
/// projection is large, so there is nothing left to add and holding forward does
/// almost nothing. Point perpendicular to your motion and the projection is near
/// zero, so you get the full budget -- which *turns* the velocity vector without
/// spending it.
///
/// That is why strafing is expressive and holding forward is not: the game is
/// not rewarding a faster input, it is rewarding an input aimed at the component
/// of your motion you have not already used up.
fn air_accelerate(p: &mut Player, wish: V3, wish_speed: Fx) {
    let current = p.vel.x.mul(wish.x).add(p.vel.z.mul(wish.z));
    let head_room = wish_speed.sub(current);
    if head_room.raw() <= 0 {
        return;
    }
    let mut step = t::air_accel().mul(wish_speed).mul(DT);
    if step.raw() > head_room.raw() {
        step = head_room;
    }
    p.vel.x = p.vel.x.add(wish.x.mul(step));
    p.vel.z = p.vel.z.add(wish.z.mul(step));

    // A ceiling Source does not have. See `tuning::air_speed_cap`.
    clamp_air_speed(p);
}

/// The middle-click mechanic action, per class.
///
/// One button means something different on every class, which is where the
/// identity lives -- see `controls.md`. Everything else about the control
/// scheme is shared.
fn mechanic_action(p: &mut Player, who: usize, input: Input, scene: &Scene) {
    let from = aim::origin(p.pos);
    // The mechanic fires on the press with no startup, so there is nothing to
    // lock it against -- it asks `crate::aim` the same question an ability
    // does and uses the answer immediately.
    let placed = |reach| aim::grounded_path(who, input, reach, scene).to;
    match p.mechanic {
        // Throw commits you: faster, exposed, and unable to block until it is
        // back. Recall damages along the return path; reactivating mid-flight
        // leaps you to it, which is the Bulwark's approach tool.
        Mechanic::Shield(shield) => {
            p.mechanic = Mechanic::Shield(match shield {
                // A skillshot: it flies at what the crosshair is on, not along
                // the look angle from the chest. Those are parallel lines that
                // never meet, and the shield is thrown far enough that the gap
                // between them is most of a body.
                Shield::Held => Shield::Flying {
                    pos: from,
                    vel: aim::skillshot_path(who, input, t::shield_range(), scene)
                        .dir()
                        .scale(t::shield_speed()),
                    outbound: true,
                    travelled: Fx::ZERO,
                },
                Shield::Planted { pos } => {
                    let to_owner = from.sub(pos);
                    Shield::Flying {
                        pos,
                        vel: to_owner.normalized().scale(t::shield_speed()),
                        outbound: false,
                        travelled: Fx::ZERO,
                    }
                }
                Shield::Flying {
                    pos,
                    vel,
                    outbound,
                    travelled,
                } => {
                    let to_shield = pos.sub(p.pos);
                    if to_shield.flat_len().raw() > Fx::ONE.raw() {
                        let dir = V3::new(to_shield.x, Fx::ZERO, to_shield.z).normalized();
                        p.vel.x = dir.x.mul(t::leap_speed());
                        p.vel.z = dir.z.mul(t::leap_speed());
                        p.vel.y = t::leap_rise();
                        p.grounded = false;
                    }
                    Shield::Flying {
                        pos,
                        vel,
                        outbound,
                        travelled,
                    }
                }
            });
        }

        // Rush. It used to cycle the weapon form, which was the mode this
        // rebuild deleted -- the weapon is a mouse button now, so the mechanic
        // key is free for the mechanic the class is actually built around.
        Mechanic::Forms { .. } => {
            start_rush(p, input);
        }

        // `E` on the Reaver is an ability rather than an instant -- see
        // `moves::on_e` -- so this key never reaches here for her. Throwing a
        // second body across the arena has a startup you can be punished
        // during, and getting it back has a damage number.
        Mechanic::Shadow(_) => {}

        // Spawn a structure ahead. A fourth collapses the oldest, so the cap
        // is the resource.
        Mechanic::Structures(mut slots) => {
            let raised = class::Structure::raised(placed(t::raise_reach()));
            if let Some(free) = slots.iter_mut().find(|s| s.is_none()) {
                *free = Some(raised);
            } else {
                slots.rotate_left(1);
                slots[class::MAX_STRUCTURES - 1] = Some(raised);
            }
            p.mechanic = Mechanic::Structures(slots);
        }

        // Health is the resource; there is no separate button.
        Mechanic::Blood => {}

        // The meter is steered by which button attacks, not by this one.
        Mechanic::Meter { .. } => {}
    }
}

/// Per-tick mechanic upkeep: shields in flight, shadows on a leash, the meter
/// burning at depth.
fn step_mechanic(p: &mut Player) {
    match p.mechanic {
        Mechanic::Shield(Shield::Flying {
            pos,
            vel,
            outbound,
            travelled,
        }) => {
            let step = vel.scale(DT);
            let next = pos.add(step);
            let gone = travelled.add(step.flat_len());
            p.mechanic = Mechanic::Shield(if outbound {
                if gone.raw() >= t::shield_range().raw() {
                    // Thrown downhill it would otherwise plant inside the
                    // floor, which is a shield you cannot see and cannot walk
                    // to. Lifted to the surface rather than dropped onto it, so
                    // a level throw is untouched.
                    Shield::Planted {
                        pos: V3::new(next.x, next.y.max(arena::ground_under(next)), next.z),
                    }
                } else {
                    Shield::Flying {
                        pos: next,
                        vel,
                        outbound,
                        travelled: gone,
                    }
                }
            } else {
                let hand = aim::origin(p.pos);
                if next.sub(hand).flat_len().raw() < Fx::ONE.raw() {
                    Shield::Held
                } else {
                    // Home in, so the return does not miss a moving owner.
                    let dir = hand.sub(next).normalized().scale(t::shield_speed());
                    Shield::Flying {
                        pos: next,
                        vel: dir,
                        outbound,
                        travelled: gone,
                    }
                }
            });
        }

        // The dash and its charge, both counting down. Saturating, so a dash
        // that has run out sits at zero rather than wrapping into a very long
        // one.
        Mechanic::Forms {
            form,
            rush,
            recharge,
            rush_vel,
        } => {
            p.mechanic = Mechanic::Forms {
                form,
                rush: rush.saturating_sub(1),
                recharge: recharge.saturating_sub(1),
                rush_vel,
            };
        }

        // The second body: where it is, what it is copying, and the leash.
        // See `crate::shadow`.
        Mechanic::Shadow(_) => shadow::step(p),

        // Past the deep threshold the forces burn you. Relief comes from
        // coming back inside the line, not from a reward -- see dual-mage.md.
        Mechanic::Meter { value } => {
            let depth = value.abs();
            if depth > t::meter_deep() {
                let over = depth - t::meter_deep();
                let span = (t::meter_max() - t::meter_deep()).max(1);
                let burn = (t::meter_burn() * over) / span;
                p.health = (p.health - burn.max(1)).max(1);
            }
        }

        _ => {}
    }
}

/// Attacking steers the Dual mage's meter: dark darker, light lighter, and
/// stronger moves push harder. Nothing else moves it, so every step is a
/// consequence of a decision the player made.
///
/// **Which way comes from the move, not from the buttons held down** -- see
/// `moves::dual::side`. Reading the input bits meant asking which of `shift`
/// and `left click` won, and a move already knows which force it is made of.
///
/// A move with no side pushes you **further along whichever way you were already
/// going**, which is the rule `docs/design/dual-mage.md` states for every input
/// that is neither left nor right: direction comes from side-ness, and a key
/// has none. At dead centre there is no path to push along, so it does nothing
/// -- correctly, because leaving the middle is supposed to be a decision.
/// Does this move steer the meter when it *lands* rather than when it is
/// thrown?
///
/// The Dual mage's two autos, and nothing else in the game. Everything else
/// votes the moment you commit to it; an auto has to connect, and a whiff
/// steers nothing.
fn steers_on_contact(class: Class, kind: u8) -> bool {
    class == Class::DualMage && moves::dual::is_an_auto(kind)
}

fn steer_meter(p: &mut Player, kind: u8) {
    let Mechanic::Meter { value } = p.mechanic else {
        return;
    };
    let push = 4 + (moves::get(p.class, kind).damage / 40);
    let side = match moves::dual::side(kind) {
        0 => value.signum(),
        side => side,
    };
    p.mechanic = Mechanic::Meter {
        value: (value + push * side).clamp(-t::meter_max(), t::meter_max()),
    };
}

fn hash_mechanic(h: &mut Fnv, m: &Mechanic) {
    match m {
        Mechanic::Shield(Shield::Held) => h.write_u32(0),
        Mechanic::Shield(Shield::Planted { pos }) => {
            h.write_u32(1);
            hash_v3(h, pos);
        }
        Mechanic::Shield(Shield::Flying {
            pos,
            vel,
            outbound,
            travelled,
        }) => {
            h.write_u32(2);
            hash_v3(h, pos);
            hash_v3(h, vel);
            h.write_u32(*outbound as u32);
            h.write_i32(travelled.raw());
        }
        Mechanic::Forms {
            form,
            rush,
            recharge,
            rush_vel,
        } => {
            h.write_u32(3);
            h.write_u32(*form as u32);
            h.write_u32(*rush as u32);
            h.write_u32(*recharge as u32);
            hash_v3(h, rush_vel);
        }
        Mechanic::Shadow(shadow) => {
            h.write_u32(4);
            hash_v3(h, &shadow.pos);
            hash_v3(h, &shadow.facing);
            match shadow.doing {
                Ghost::Attending => h.write_u32(0),
                Ghost::Casting { from, to, age } => {
                    h.write_u32(1);
                    hash_v3(h, &from);
                    hash_v3(h, &to);
                    h.write_u32(age as u32);
                }
                Ghost::Waiting => h.write_u32(2),
                Ghost::Returning { struck } => {
                    h.write_u32(3);
                    h.write_u32(struck as u32);
                }
            }
            h.write_u32(shadow.echo as u32);
            h.write_u32(shadow.echo_age as u32);
            h.write_u32(shadow.echo_used as u32);
            h.write_u32(shadow.dash as u32);
        }
        Mechanic::Structures(slots) => {
            h.write_u32(5);
            for slot in slots {
                match slot {
                    Some(s) => {
                        hash_v3(h, &s.at);
                        hash_v3(h, &s.vel);
                        h.write_u32(s.age as u32);
                        h.write_u32(s.struck as u32);
                        h.write_u32(s.launched as u32);
                        hash_v3(h, &s.launch_from);
                        h.write_u32(s.knock_struck as u32);
                    }
                    None => h.write_u32(0),
                }
            }
        }
        Mechanic::Blood => h.write_u32(6),
        Mechanic::Meter { value } => {
            h.write_u32(7);
            h.write_i32(*value);
        }
    }
}

fn hash_v3(h: &mut Fnv, v: &V3) {
    h.write_i32(v.x.raw());
    h.write_i32(v.y.raw());
    h.write_i32(v.z.raw());
}

/// How long the parry flourish plays for. Frames, and long enough to be seen
/// without outlasting the stagger it earned.
pub const PARRY_FLOURISH: u16 = 14;

/// Advance the four clocks the renderer needs and combat does not.
///
/// Every one of them is a fact about what just happened -- how far you have
/// walked, how long you have been in the air -- rather than a decision, which
/// is why they can live here without complicating anything. Being in the
/// snapshot is the whole point: a rollback rewinds them with everything else,
/// so a landing that gets re-simulated lands the same way twice.
fn advance_clocks(p: &mut Player) {
    if p.grounded {
        // `air_frames` deliberately keeps its value on the ground: it is how
        // long the last flight lasted, which is the only thing that can tell a
        // hop's landing from a long fall's. Clearing it on touchdown would
        // throw away the one number the landing animation needs.
        p.since_landed = p.since_landed.saturating_add(1);
    } else {
        if p.since_landed > 0 {
            p.air_frames = 0;
        }
        p.since_landed = 0;
        p.air_frames = p.air_frames.saturating_add(1);
    }
    p.parried = p.parried.saturating_sub(1);
    p.crouched_for = if p.crouching {
        p.crouched_for.saturating_add(1)
    } else {
        0
    };

    // Ground covered this frame as a fraction of one stride. The raw 16.16 bits
    // of a fraction of a turn *are* the phase, so there is no conversion.
    //
    // A rider is the exception and is left alone here: their world velocity is
    // zero while they are aboard, and `ride` has already advanced this from the
    // step they took across the creature.
    if p.aboard() {
        return;
    }
    let advance = p.vel.flat_len().mul(DT).div(stride_length(p));
    p.stride = p.stride.wrapping_add(advance.raw().clamp(0, 65535) as u16);
}

/// How much ground one full cycle of this body's current gait covers.
///
/// Blended by speed, shortened when the movement is sideways, and short again
/// when crouched. The renderer plays the clips at the phase this produces and
/// the clips were authored against the same numbers, so the three agree by
/// construction rather than by anybody remembering to keep them in step.
fn stride_length(p: &Player) -> Fx {
    if p.crouching {
        return t::CROUCH_STRIDE;
    }
    let speed = p.vel.flat_len();
    let gait = speed
        .sub(t::WALK_AT)
        .div(t::RUN_AT.sub(t::WALK_AT))
        .clamp(Fx::ZERO, Fx::ONE);
    let base = t::WALK_STRIDE.add(t::RUN_STRIDE.sub(t::WALK_STRIDE).mul(gait));

    // How much of the travel is straight ahead. A pure sidestep gets the full
    // shortening; a forward run gets none.
    if speed.raw() <= 0 {
        return base;
    }
    let along = p
        .vel
        .dot(p.facing)
        .abs()
        .div(speed)
        .clamp(Fx::ZERO, Fx::ONE);
    let scale = t::STRAFE_STRIDE.add(Fx::ONE.sub(t::STRAFE_STRIDE).mul(along));
    base.mul(scale).max(SHORTEST_STRIDE)
}

/// A floor on the stride, so the phase cannot be divided by something near
/// zero. Not a feel number: a tenth of a metre is far below any stride the
/// constants above can produce, and it exists only to stop a division blowing
/// up if they are ever retuned to nonsense.
const SHORTEST_STRIDE: Fx = Fx::ratio(1, 10);

/// Let a body come to rest without accepting input. Used during the pause
/// between rounds.
fn settle(p: &mut Player) {
    p.vel.x = p.vel.x.mul(t::settle_decay());
    p.vel.z = p.vel.z.mul(t::settle_decay());
    if !p.grounded {
        p.vel.y = p.vel.y.add(t::gravity().mul(DT));
    }
    p.pos = p.pos.add(p.vel.scale(DT));
    let r = arena::resolve(p.pos, p.vel, p.grounded);
    p.pos = r.pos;
    p.vel = r.vel;
    p.grounded = r.grounded;
}

/// Bodies are solid. Push them apart symmetrically so neither index wins.
fn separate_bodies(players: &mut [Player; MAX_PLAYERS]) {
    let delta = players[1].pos.sub(players[0].pos);
    let dist = delta.flat_len();
    let min = t::body_radius().add(t::body_radius());
    if dist.raw() == 0 || dist.raw() >= min.raw() {
        return;
    }
    let push = min.sub(dist).mul(Fx::ratio(1, 2));
    let dir = V3::new(delta.x, Fx::ZERO, delta.z).normalized();
    players[0].pos = players[0].pos.sub(dir.scale(push));
    players[1].pos = players[1].pos.add(dir.scale(push));
}

/// FNV-1a. Chosen because it is trivially portable and has no platform-varying
/// behaviour -- the hash itself must be deterministic or desync detection lies.
struct Fnv(u64);

impl Fnv {
    fn new() -> Fnv {
        Fnv(0xcbf2_9ce4_8422_2325)
    }
    fn write_u64(&mut self, v: u64) {
        self.write_u32(v as u32);
        self.write_u32((v >> 32) as u32);
    }

    fn write_u32(&mut self, v: u32) {
        for b in v.to_le_bytes() {
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(0x100_0000_01b3);
        }
    }
    fn write_i32(&mut self, v: i32) {
        self.write_u32(v as u32);
    }
    fn finish(self) -> u64 {
        self.0
    }
}

/// Frame data for a move, for debug overlays and design docs.
pub fn move_frames(class: Class, kind: u8) -> (u16, u16, u16) {
    moves::frames(class, kind)
}

/// Parry window length, exposed for debug overlays.
pub fn parry_window() -> u16 {
    t::parry_window()
}

// ---------------------------------------------------------------------------
// The Reaver's second body
// ---------------------------------------------------------------------------
//
// Where the shadow is and what it is copying lives in `crate::shadow`. What
// follows is the part that needs more than one fighter: the order `E` gives it,
// the blows its copy lands, and what its way home does to anybody standing in
// it.

impl World {
    /// Send the shadow out, or call it home -- and take the blades with it.
    ///
    /// The second half is the combination the whole kit is built around.
    /// Recalling a shadow with a Guillotine open does not cancel the lotus, it
    /// **reactivates** it: the blades stop hanging and start chasing, and since
    /// they track the shadow rather than the ground they drag the length of the
    /// arena behind it. A recall through a crowd is the Reaver's biggest turn.
    fn order_the_shadow(&mut self, who: usize, to: V3) {
        let Some(order) = shadow::order(&mut self.players[who], to) else {
            return;
        };
        if order != shadow::Order::Recalled {
            return;
        }
        for slot in self.effects.iter_mut().flatten() {
            if slot.kind == EffectKind::GuillotineLotus && slot.owner == who as u8 {
                slot.lotus_send_home();
            }
        }
    }

    /// What the second body does to the other one, this frame.
    fn step_shadows(&mut self) {
        for owner in 0..MAX_PLAYERS {
            self.echo_strikes(owner);
            self.recall_cuts(owner);
        }
    }

    /// The shadow's copy of her swing, landing a beat after hers.
    ///
    /// It goes through `resolve_hit` against a stand-in body, so the copy is
    /// blocked, ducked and spaced by exactly the rules her own swing is -- and
    /// the volume it puts in the world is `state::hitbox`'s, which is the same
    /// one the overlay draws.
    ///
    /// Two differences, and both are the shadow not being a person. It deals
    /// `shadow_echo` of what she deals, which is what makes holding the shadow
    /// worth a quarter again on every swing. And it cannot be **parried**: a
    /// parry is a stagger paid by the attacker, and there is nobody at this end
    /// of the blow to stagger. Reading it perfectly still stops it dead, which
    /// is what blocking a copy should be worth.
    fn echo_strikes(&mut self, owner: usize) {
        let Some(ghost) = shadow::echo_body(&self.players[owner]) else {
            return;
        };
        // In a hunt the fighters cannot hurt each other, so the copy has the
        // creature to swing at instead -- the same condition direct hits use,
        // and for the same reason: a mechanic that only works in versus is half
        // a mechanic.
        let landed = match self.monster {
            Some(_) => self.echo_gores_the_creature(&ghost),
            None => self.echo_cuts_the_other_fighter(owner, &ghost),
        };
        if landed {
            shadow::echo_landed(&mut self.players[owner]);
        }
    }

    fn echo_cuts_the_other_fighter(&mut self, owner: usize, ghost: &Player) -> bool {
        let target = 1 - owner;
        let defender = self.players[target];
        if defender.health <= 0 {
            return false;
        }
        let Some(mut hit) = resolve_hit(ghost, &defender, owner as u8) else {
            return false;
        };
        hit.damage = Fx::from_int(hit.damage).mul(t::shadow_echo()).to_int();
        hit.parried = false;
        apply_hit(&mut self.players[target], hit);
        true
    }

    fn echo_gores_the_creature(&mut self, ghost: &Player) -> bool {
        let (Some(mut beast), Some(box_out), Some(kind)) =
            (self.monster, hitbox(ghost), ghost.action.attack_kind())
        else {
            return false;
        };
        if box_out.spent || !beast.alive() {
            return false;
        }
        let Some(part) = part_under(&beast, ghost, &box_out) else {
            return false;
        };
        let raw = Fx::from_int(moves::get(ghost.class, kind).damage)
            .mul(preying(ghost.class, beast.disabled()))
            .mul(t::shadow_echo())
            .to_int();
        beast.take_hit(part, raw);
        self.monster = Some(beast);
        true
    }

    /// The way home. It cuts once and slows, which is the mechanic's own
    /// description of itself -- and the slow is the half that matters, because
    /// a recall is how the Reaver closes a gap she has just opened.
    fn recall_cuts(&mut self, owner: usize) {
        let Some(ghost) = shadow::of(&self.players[owner]) else {
            return;
        };
        if !ghost.is_returning() {
            return;
        }
        let m = moves::get(Class::ShadowReaver, SLOT_MECHANIC);
        let reach = t::body_radius().add(t::body_radius());

        // The creature, in a hunt. `QUARRY_VICTIM` is the slot the effects
        // already use for "the thing that is not a fighter", and the return
        // cuts it once for the same reason it cuts a fighter once.
        if let Some(mut beast) = self.monster {
            if beast.alive() && !shadow::already_cut(ghost, QUARRY_VICTIM) {
                if let Some(part) = beast.part_struck(ghost.pos, reach, t::body_height()) {
                    let raw = Fx::from_int(m.damage)
                        .mul(preying(Class::ShadowReaver, beast.disabled()))
                        .to_int();
                    beast.take_hit(part, raw);
                    self.monster = Some(beast);
                    shadow::mark_cut(&mut self.players[owner], QUARRY_VICTIM);
                }
            }
            return;
        }

        let target = 1 - owner;
        if shadow::already_cut(ghost, target) {
            return;
        }
        let victim = self.players[target];
        if victim.health <= 0 || victim.action.invulnerable() {
            return;
        }
        let apart = V3::new(
            victim.pos.x.sub(ghost.pos.x),
            Fx::ZERO,
            victim.pos.z.sub(ghost.pos.z),
        );
        if apart.flat_len().raw() > reach.raw() {
            return;
        }
        let away = apart.normalized();
        let (guarding, _) = guard_against(&victim, ghost.pos, m.unblockable);
        apply_hit(
            &mut self.players[target],
            Hit {
                damage: m.damage,
                hitstun: m.hitstun,
                blockstun: m.blockstun,
                knockback: m.knockback,
                launch: Fx::ZERO,
                grabs: 0,
                by: owner as u8,
                dir: away,
                blocked: guarding,
                parried: false,
            },
        );
        self.players[target].slow(t::slow_frames(), t::shadow_recall_slow());
        shadow::mark_cut(&mut self.players[owner], target);
    }
}

/// Age every effect, apply what it does, and drop the expired.
///
/// Effects act *after* both fighters have stepped, so standing in a field for a
/// frame costs you whether you walked in or were knocked in. The alternative --
/// checking before movement -- would let someone walk through a fire pillar
/// untouched on the frame they entered it.
///
/// A method on `World` rather than a free function over the fighters, because
/// an effect can touch the creature too. It could not, for a long time, and the
/// result was that half the Blood mage's kit did nothing at all in a hunt: the
/// spike went into the ground, drained an empty patch of arena and expired. A
/// hazard that only exists in versus is not a hazard.
impl World {
    fn step_effects(&mut self) {
        for i in 0..MAX_EFFECTS {
            let Some(mut effect) = self.effects[i] else {
                continue;
            };
            let turning = !effect.returning();
            let going_out = !effect.lotus_coming_back();
            effect.age += 1;
            // The frame the blade turns it forgets everyone it cut on the way
            // out, so the way back can cut them again. "Damage on both passes"
            // is only worth saying if the same target can eat both. The lotus
            // does the same at its own turn, with six blades instead of one.
            if effect.kind == EffectKind::Bloodletter && turning && effect.returning() {
                effect.forget_hits();
            }
            if effect.kind == EffectKind::GuillotineLotus && going_out && effect.lotus_coming_back()
            {
                effect.forget_hits();
            }
            // The one effect that does not stay where it was cast. Its centre
            // is the Reaver's shadow, live, so recalling the shadow drags the
            // blades after it -- see `crate::shadow`. Written before anything
            // is tested against it, so what the blades hit this frame is where
            // they actually are rather than where they were last frame.
            if effect.kind.follows_the_mechanic() {
                if let Some(at) = self
                    .players
                    .get(effect.owner as usize)
                    .and_then(|owner| owner.mechanic.placed())
                {
                    effect.pos = at;
                }
            }
            if effect.age >= effect.life {
                self.effects[i] = None;
                self.pay_out(&effect);
                continue;
            }
            self.apply_effect(&mut effect);
            self.effects[i] = Some(effect);
        }
        // The slow's tail, for every source of one -- a drain field here, a
        // stone churning under your feet in `stones`. It runs down before either
        // of them gets to refresh it, so standing in one holds the slow at full
        // strength and walking out of it lets the tail run. A root has no tail:
        // it is a fixed number of frames and then it is over.
        for p in self.players.iter_mut() {
            p.slowed = p.slowed.saturating_sub(1);
            if p.slowed == 0 {
                p.slow_mul = Fx::ONE;
            }
            p.rooted = p.rooted.saturating_sub(1);
        }
    }

    /// What an expiring effect still owes its caster.
    ///
    /// Only the blade owes anything: everything else paid as it went. Catching
    /// it is the payday, which is what makes the ability a small commitment
    /// rather than a free poke -- the cut lands at once and the health has to
    /// survive the flight home.
    fn pay_out(&mut self, effect: &Effect) {
        if effect.kind != EffectKind::Bloodletter || effect.banked <= 0 {
            return;
        }
        let owed = effect.leeched(effect.banked);
        if let Some(caster) = self.players.get_mut(effect.owner as usize) {
            caster.heal(owed);
        }
    }

    /// What one effect does to everything standing in it this frame.
    ///
    /// Takes the effect by `&mut` because two of them keep books: which arm has
    /// caught whom, and how much blood a blade is carrying home.
    fn apply_effect(&mut self, effect: &mut Effect) {
        match effect.kind {
            EffectKind::FirePillar => {
                if !effect.ticks_now() {
                    return;
                }
                let (base, column) = effect.pillar_volumes();
                let radius = t::body_radius();
                let height = t::body_height();
                for i in 0..MAX_PLAYERS {
                    let p = self.players[i];
                    if !self.effects_reach(i, effect.owner) {
                        continue;
                    }
                    if base.contains(effect.pos, p.pos, radius, height)
                        || column.contains(effect.pos, p.pos, radius, height)
                    {
                        self.drain(i, effect);
                    }
                }
                self.gore_the_creature(effect, 0, effect.pos, base.radius);
            }

            // Drain *and* slow. The slow is the part that matters: damage alone
            // makes a puddle you step out of, and the slow is what makes leaving
            // cost time -- which is what turns it into something you put
            // *between* yourself and someone else.
            EffectKind::BlackSpike => {
                let volume = effect.spike_volume();
                let ticking = effect.ticks_now();
                for i in 0..MAX_PLAYERS {
                    let p = self.players[i];
                    if !self.effects_reach(i, effect.owner)
                        || !volume.contains(effect.pos, p.pos, t::body_radius(), p.hurt_height())
                    {
                        continue;
                    }
                    self.players[i].slow(t::slow_frames(), t::spike_slow());
                    if ticking {
                        self.drain(i, effect);
                    }
                }
                if ticking {
                    self.gore_the_creature(effect, 0, effect.pos, volume.radius);
                }
            }

            // The blade. One pass out, one back, and it cuts each victim once
            // per pass. What it takes is banked rather than paid, and arrives
            // when the blade does -- see `pay_out`.
            EffectKind::Bloodletter => {
                let at = effect.blade_at();
                let radius = effect.field_radius();
                let pass = effect.pass();
                for i in 0..MAX_PLAYERS {
                    if !self.effects_reach(i, effect.owner)
                        || effect.already_hit(pass, i)
                        || !self.inside(i, at, radius)
                    {
                        continue;
                    }
                    effect.take_hit(pass, i);
                    effect.banked += self.cut(i, effect, at, Fx::ONE);
                }
                effect.banked += self.gore_the_creature(effect, pass, at, radius);
            }

            // Six blades out of the shadow and six back into it. Each is its
            // own part, so a blade catches a given victim once per pass and the
            // mask is cleared at the turn -- the same bookkeeping the blade
            // uses, widened.
            //
            // What the return is *worth* is a share of the way out, because the
            // two are not the same event: the eruption is the execute and the
            // drag home is the reason to recall a shadow through a crowd. It
            // also slows, which is what makes a lotus dragged across somebody
            // a setup rather than a parting shot.
            EffectKind::GuillotineLotus => {
                let radius = effect.field_radius();
                let coming_back = effect.lotus_coming_back();
                for blade in 0..LOTUS_BLADES {
                    let (was, at) = effect.lotus_span(blade, effect.pos);
                    for i in 0..MAX_PLAYERS {
                        if !self.effects_reach(i, effect.owner)
                            || effect.already_hit(blade, i)
                            || !self.swept(i, was, at, radius)
                        {
                            continue;
                        }
                        effect.take_hit(blade, i);
                        let share = if coming_back {
                            t::lotus_return_damage()
                        } else {
                            Fx::ONE
                        };
                        self.cut(i, effect, at, share);
                        self.players[i].slow(t::slow_frames(), t::lotus_slow());
                    }
                    self.gore_the_creature(effect, blade, at, radius);
                }
            }

            // Four arms, each its own skillshot, and all four of them the price
            // of the root. Every arm is tested separately and remembers who it
            // has already caught, because "hit by all four" is a question about
            // *different* arms and one shared mask could not tell them apart.
            EffectKind::Grasp => {
                let radius = effect.field_radius();
                for arm in 0..GRASP_ARMS {
                    let at = effect.arm_at(arm);
                    for i in 0..MAX_PLAYERS {
                        if !self.effects_reach(i, effect.owner)
                            || effect.already_hit(arm, i)
                            || !self.inside(i, at, radius)
                        {
                            continue;
                        }
                        effect.take_hit(arm, i);
                        let dealt = self.cut(i, effect, at, Fx::ONE);
                        let owed = effect.leeched(dealt);
                        self.players[effect.owner as usize].heal(owed);
                        // Caught by every one of them. The arms close, and for
                        // a moment you are not going anywhere.
                        if effect.parts_landed(i, GRASP_ARMS) == GRASP_ARMS {
                            self.players[i].root(t::grasp_root());
                        }
                    }
                    let dealt = self.gore_the_creature(effect, arm, at, radius);
                    let owed = effect.leeched(dealt);
                    self.players[effect.owner as usize].heal(owed);
                }
            }
        }
    }

    /// Can this effect touch this fighter at all?
    ///
    /// Never its own caster -- a fire pillar you cannot stand next to is a fire
    /// pillar you cannot use -- never a corpse, and never a partner: in a hunt
    /// the fighters cannot hurt each other, and the condition is the creature's
    /// presence rather than a separate flag, exactly as it is for direct hits in
    /// `advance`. Without this, a Blood mage's field was the one thing in the
    /// game that could kill a team-mate.
    fn effects_reach(&self, victim: usize, owner: u8) -> bool {
        victim as u8 != owner && self.players[victim].health > 0 && self.monster.is_none()
    }

    /// Did a blade sweeping from `was` to `at` cross this fighter's body?
    ///
    /// The capsule test [`resolve_hit`] already uses for a weapon that is a
    /// line, against the same standing body. It exists because a fast enough
    /// point tunnels: the thing being tested here crosses a metre in a frame,
    /// and a body is not a metre wide.
    fn swept(&self, victim: usize, was: V3, at: V3, radius: Fx) -> bool {
        let p = self.players[victim];
        let spine = V3::new(p.pos.x, p.pos.y.add(p.hurt_height()), p.pos.z);
        crate::math::segment_gap(was, at, p.pos, spine).raw() <= radius.add(t::body_radius()).raw()
    }

    /// Is this fighter's body inside a sphere?
    ///
    /// Flat distance for the width, like every other hit test in the game, and
    /// a vertical overlap so that a blade thrown over somebody's head misses.
    fn inside(&self, victim: usize, at: V3, radius: Fx) -> bool {
        let p = self.players[victim];
        let flat = V3::new(p.pos.x.sub(at.x), Fx::ZERO, p.pos.z.sub(at.z)).flat_len();
        flat.raw() <= radius.add(t::body_radius()).raw()
            && p.pos.y.add(p.hurt_height()).raw() >= at.y.sub(radius).raw()
            && p.pos.y.raw() <= at.y.add(radius).raw()
    }

    /// One tick of a field. Damage, and the caster's share of it straight back.
    ///
    /// Paid on the tick rather than banked to the end, which is the whole
    /// difference between a Blood mage who can stand in a fight trading and one
    /// who has to survive until a timer runs out to be repaid.
    fn drain(&mut self, victim: usize, effect: &Effect) {
        let damage = Fx::from_int(effect.damage())
            .mul(preying(effect.class, self.players[victim].disabled()))
            .to_int();
        let dealt = damage.min(self.players[victim].health);
        self.players[victim].health = (self.players[victim].health - damage).max(0);
        let owed = effect.leeched(dealt);
        self.players[effect.owner as usize].heal(owed);
    }

    /// A travelling effect connecting with a fighter. Returns what landed.
    ///
    /// Goes through the same `apply_hit` a swing does, so it stuns, knocks back
    /// and can be blocked exactly like one. That is the difference between a
    /// thrown blade and a puddle of fire: the blade is an attack, and guard is
    /// an answer to attacks. Knockback runs away from the *thing that hit you*
    /// rather than from the caster, who may be twenty metres behind it.
    /// `share` is what this particular contact is worth against the move's own
    /// number. One for almost everything: an effect that hits twice usually
    /// means the same blow twice. The lotus is the exception -- what it does
    /// coming home is a fraction of what the eruption did, because the two are
    /// different events and pricing them the same would make a recall through a
    /// crowd worth double an execute.
    fn cut(&mut self, victim: usize, effect: &Effect, from: V3, share: Fx) -> i32 {
        let m = effect.source();
        let p = self.players[victim];
        if p.action.invulnerable() || (p.crouching && !m.hits_crouching) {
            return 0;
        }
        let away = V3::new(p.pos.x.sub(from.x), Fx::ZERO, p.pos.z.sub(from.z)).normalized();
        let facing_it = p.facing.dot(away.scale(Fx::ONE.neg())).raw() >= t::guard_arc_cos().raw();
        let guarding = !m.unblockable && p.action.guarding() && facing_it;
        let parried = !m.unblockable
            && matches!(p.action, Action::Guard { held } if held < t::parry_window())
            && facing_it;
        let damage = Fx::from_int(m.damage)
            .mul(preying(effect.class, p.disabled()))
            .mul(share)
            .to_int();
        let dealt = if guarding || parried {
            0
        } else {
            damage.min(p.health)
        };
        apply_hit(
            &mut self.players[victim],
            Hit {
                damage,
                hitstun: m.hitstun,
                blockstun: m.blockstun,
                knockback: m.knockback,
                launch: m.launch,
                grabs: 0,
                by: effect.owner,
                dir: away,
                blocked: guarding,
                parried,
            },
        );
        if parried {
            self.players[victim].parried = PARRY_FLOURISH;
        }
        dealt
    }

    /// Land an effect on whichever part of the creature is inside it, once per
    /// part of the effect. Returns what went in after the hide, or nought.
    fn gore_the_creature(&mut self, effect: &mut Effect, part: usize, at: V3, radius: Fx) -> i32 {
        if effect.already_hit(part, QUARRY_VICTIM) {
            return 0;
        }
        let Some(mut beast) = self.monster else {
            return 0;
        };
        if !beast.alive() {
            return 0;
        }
        let Some(struck) = beast.part_struck(at, radius, t::body_height()) else {
            return 0;
        };
        let raw = Fx::from_int(effect.damage())
            .mul(preying(effect.class, beast.disabled()))
            .to_int();
        let dealt = beast.take_hit(struck, raw);
        self.monster = Some(beast);
        // A field has one part and hits over and over on its tick; a blade or an
        // arm has a pass to spend and spends it here.
        if effect.kind.travels() {
            effect.take_hit(part, QUARRY_VICTIM);
        }
        dealt
    }
}

/// Keep a grabbed fighter at their captor's arm's length.
fn drag_the_held(players: &mut [Player; MAX_PLAYERS]) {
    let snapshot = *players;
    for victim in players.iter_mut() {
        let Action::Held { .. } = victim.action else {
            continue;
        };
        let by = victim.held_by as usize;
        if by >= MAX_PLAYERS {
            continue;
        }
        let holder = &snapshot[by];
        let reach = t::body_radius().add(t::body_radius());
        victim.pos.x = holder.pos.x.add(holder.facing.x.mul(reach));
        victim.pos.z = holder.pos.z.add(holder.facing.z.mul(reach));
        victim.pos.y = holder.pos.y;
        victim.vel = V3::ZERO;
        victim.grounded = holder.grounded;
    }
}

/// Put an effect into the world, replacing the oldest if the board is full.
fn spawn_effect(effects: &mut [Option<Effect>; MAX_EFFECTS], effect: Effect) {
    let owner = effect.owner;
    if let Some(free) = effects.iter_mut().find(|s| s.is_none()) {
        *free = Some(effect);
        return;
    }
    // Full. Your oldest goes -- **yours**, never the other fighter's. Spamming
    // should cost you your own setup and nothing else; an eviction that reached
    // across owners would mean one fighter could delete the other's by holding
    // a button, which is not a decision anybody made.
    let oldest = effects
        .iter()
        .enumerate()
        .filter_map(|(i, e)| e.filter(|e| e.owner == owner).map(|e| (i, e.age)))
        .max_by_key(|(_, age)| *age)
        .map(|(i, _)| i);
    if let Some(i) = oldest {
        effects[i] = Some(effect);
    }
}

/// Walking speed after whatever is slowing you.
///
/// A slow is a positioning tool, not a damage one: the Blood mage's field hurts
/// while you stand in it and makes leaving it take longer, which is what turns
/// a puddle of damage into a wall.
fn dragged(p: &Player, speed: Fx) -> Fx {
    if p.slowed == 0 {
        return speed;
    }
    speed.mul(p.slow_mul)
}

// ---------------------------------------------------------------------------
// Riding the creature
//
// See `docs/design/monsters.md`. The rule that makes all of this work is one
// line: while mounted, the fighter's position in the creature's body frame is
// the authoritative one and the world position is derived from it.
// ---------------------------------------------------------------------------

/// One tick of a fighter standing on the creature.
///
/// The option set up here is deliberately smaller than on the ground: move,
/// brace, attack, guard, and jump off. **There is no dodge.** A seventeen
/// metre-per-second dash on a surface two metres wide is a way to fall off by
/// accident, and taking it away is what makes bracing a real answer rather than
/// a worse version of one you already had.
fn step_rider(p: &mut Player, who: usize, input: Input, beast: &Monster, scene: &Scene) {
    let part = p.mount as usize;
    let held = p.local;
    let radius = t::body_radius();

    // Where the patch of animal under their feet has got to. `p.pos` is still
    // last frame's world position of *exactly this point*, which is what makes
    // the subtraction honest: it measures the surface moving, never the rider
    // walking.
    let landing = beast.world_of(part, held);
    let rate = Fx::from_int(crate::TICK_HZ as i32);
    let surface = landing.sub(p.pos).scale(rate);
    let accel = surface.sub(p.grip_vel).scale(rate);
    p.grip_vel = surface;
    p.pos = landing;
    p.grounded = true;
    p.vel = V3::ZERO;

    let pressed_mechanic = input.has(Input::MECHANIC) && !p.mechanic_held;
    p.mechanic_held = input.has(Input::MECHANIC);
    let look = V3::from_turns(p.aim(input));
    if p.action.actionable() || p.action.stunned() {
        p.facing = look;
    } else if p.action.guarding() {
        p.facing = p
            .facing
            .add(look.sub(p.facing).scale(t::guard_turn_rate()))
            .normalized();
    }
    p.crouching = input.has(Input::CROUCH) && p.action.actionable();

    // The buck. Acceleration is read in body space, where the surface normal is
    // simply `+y`, and the part of it pressing the rider *into* the surface does
    // not count -- being shoved down onto something is not being thrown off it.
    let stance = beast.stance();
    let felt = stance.dir_to_body(accel);
    // `big_len`, not `len`: these are accelerations in the hundreds, and a
    // squared 16.16 value saturates just past 181. The first version of this
    // used `len` and reported 181 for every buck in the game, so nothing ever
    // threw anybody and nothing said why.
    let throw = crate::math::big_len(V3::new(felt.x, felt.y.max(Fx::ZERO), felt.z));
    let grip = if p.crouching {
        t::grip().mul(t::brace_grip())
    } else {
        t::grip()
    };
    if p.grip_settle > 0 {
        p.grip_settle -= 1;
    } else if throw.raw() > grip.raw() {
        thrown_off(p, &stance, surface);
        return;
    }

    step_mechanic(p);
    let want_guard = input.has(Input::RIGHT) && p.shield().is_some_and(|sh| sh.in_hand());

    p.action = match countdown(p, want_guard) {
        Some(next) => next,
        None => {
            if input.has(Input::SPECIAL)
                && p.class != Class::Champion
                && p.mechanic_ready(SLOT_SPECIAL)
            {
                begin_move(p, who, SLOT_SPECIAL, input, scene, false)
            } else if pressed_mechanic {
                match moves::on_e(p.class).filter(|slot| p.mechanic_ready(*slot)) {
                    Some(slot) => begin_move(p, who, slot, input, scene, false),
                    None => {
                        mechanic_action(p, who, input, scene);
                        Action::Free
                    }
                }
            }
            // Aboard, your feet are on something solid, so the Champion reads
            // the standing row of its grid. A Rush started up here goes
            // nowhere useful -- there is no ground under it to dash along --
            // but nothing needs to say so: `grounded` is true aboard, so the
            // standing row is what `clicked_move` picks anyway.
            else if let Some(kind) = clicked_move(p, input).filter(|k| p.mechanic_ready(*k)) {
                begin_champion(p, kind);
                begin_move(p, who, kind, input, scene, false)
            } else if want_guard {
                Action::Guard { held: 0 }
            } else {
                Action::Free
            }
        }
    };

    // Jumping is how you leave, and it carries the surface's own velocity with
    // you -- which is what makes stepping off the back of a charging animal a
    // real option rather than a mistake.
    if input.has(Input::SPACE) && p.action.actionable() {
        let mob = p.class.mobility();
        p.mount = monster::NO_PART;
        p.vel = surface.add(V3::new(Fx::ZERO, t::jump_speed().mul(mob.jump), Fx::ZERO));
        p.grounded = false;
        p.air_dodged = false;
        p.jump_hold = t::jump_hold_frames();
        return;
    }

    // Movement, in the creature's frame. The wish direction arrives
    // camera-relative as it always does; `carry_yaw` has already turned the
    // camera with the animal, so "forward" still means the same part of its
    // back that it did before it turned.
    let (ax, az) = input.move_axis();
    let speed = rider_speed(p);
    let wish = stance.dir_to_body(move_dir(p.aim(input), ax, az));
    let mut next = held;
    next.x = next.x.add(wish.x.mul(speed).mul(DT));
    next.z = next.z.add(wish.z.mul(speed).mul(DT));

    // Aboard, the ground is the animal. The stride phase runs off ground
    // covered and `vel` is zero the whole time a rider is on -- the position
    // comes from `local` -- so the walk cycle has to be advanced from the step
    // just taken across the creature's back, or a rider crossing it glides
    // there in the idle. Taken from the *intended* step rather than from where
    // they end up, because being shoved by a swinging tail is not walking.
    let walked = next.sub(held).flat_len();
    p.stride = p
        .stride
        .wrapping_add(walked.div(stride_length(p)).raw().clamp(0, 65535) as u16);

    // A step you can walk up. The creature is terrain, and the tail sits two
    // thirds of a metre below the back: without this the only way between them
    // is a jump nobody would think to try.
    let mut probe = next;
    probe.y = probe.y.add(t::step_up());
    if let Some((up, top)) = beast.surface_under(beast.world_of(part, probe), radius) {
        if up != part {
            let mut rest = beast.rest_frame(up, beast.world_of(part, probe));
            rest.y = top;
            p.mount = up as u8;
            p.local = rest;
            p.pos = beast.world_of(up, rest);
            p.grip_settle = t::mount_settle() as u8;
            return;
        }
    }

    let moved = beast.resolve(beast.world_of(part, next), radius, t::body_height());
    match beast.surface_under(moved.pos, radius) {
        Some((on, top)) => {
            // Staying on the same part keeps the body-space position that was
            // just computed rather than converting to the world and back. The
            // round trip is exact to about a millimetre, which is nothing once
            // and a crawl across the creature's back at sixty frames a second.
            let mut rest = if on == part && !moved.shoved {
                next
            } else {
                beast.rest_frame(on, moved.pos)
            };
            rest.y = top;
            if on != part {
                // Stepped from the barrel onto the tail, or the other way.
                p.grip_settle = t::mount_settle() as u8;
                p.mount = on as u8;
            }
            p.local = rest;
            p.pos = beast.world_of(on, rest);
        }
        None => {
            // Walked off the edge. You leave with whatever the surface was
            // doing, which is why stepping off a turning animal throws you
            // wide.
            p.mount = monster::NO_PART;
            p.pos = moved.pos;
            p.vel = surface;
            p.grounded = false;
        }
    }
}

/// Walking speed on the creature's back, after whatever the move you are
/// throwing costs you.
fn rider_speed(p: &Player) -> Fx {
    let base = dragged(p, t::move_speed()).mul(t::rider_speed());
    if p.crouching {
        return Fx::ZERO;
    }
    match p.action {
        Action::Free => base,
        Action::Guard { .. } => t::guard_move_speed().mul(t::rider_speed()),
        _ => match p.action.attack_kind() {
            Some(kind) => {
                let m = moves::get(p.class, kind).mobility;
                base.mul(Fx::ratio(m as i32, 100))
            }
            None => Fx::ZERO,
        },
    }
}

/// Land on the creature, or be shoved out of it.
fn meet_the_creature(p: &mut Player, beast: &Monster) {
    let radius = t::body_radius();
    let contact = beast.resolve(p.pos, radius, t::body_height());
    if contact.shoved {
        // Walked into the animal. Solid is solid.
        p.vel.x = Fx::ZERO;
        p.vel.z = Fx::ZERO;
    }
    p.pos = contact.pos;
    if p.vel.y.raw() > 0 {
        return;
    }
    let Some((part, top)) = beast.surface_under(p.pos, radius) else {
        return;
    };
    mount_on(p, beast, part, top);
}

/// Take up station on a part. Mounting is landing: there is no button, because
/// a surface is a surface.
fn mount_on(p: &mut Player, beast: &Monster, part: usize, top: Fx) {
    let mut rest = beast.rest_frame(part, p.pos);
    rest.y = top;
    p.mount = part as u8;
    p.local = rest;
    p.pos = beast.world_of(part, rest);
    p.vel = V3::ZERO;
    p.grounded = true;
    p.air_dodged = false;
    p.jump_hold = 0;
    p.air_stall = 0;
    p.grip_vel = V3::ZERO;
    p.grip_settle = t::mount_settle() as u8;
}

/// Lose your footing.
///
/// You leave with the velocity the surface had, capped, plus a push along the
/// creature's own up axis. The cap is what keeps a shake from firing someone
/// over the arena wall.
fn thrown_off(p: &mut Player, stance: &monster::Stance, surface: V3) {
    let flat = V3::new(surface.x, Fx::ZERO, surface.z);
    let speed = flat.flat_len().min(t::throw_kick());
    let up = stance.dir_to_world(V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO));
    p.mount = monster::NO_PART;
    p.vel = flat
        .normalized()
        .scale(speed)
        .add(up.scale(t::throw_lift()));
    p.grounded = false;
    p.air_dodged = false;
    p.jump_hold = 0;
    p.action = Action::HitStun {
        left: t::throw_stun(),
    };
}

/// Come off because something else decided it, rather than because you lost
/// your grip -- being hit, or the creature dying under you.
fn fall_off(p: &mut Player, beast: &Monster) {
    if !p.aboard() {
        return;
    }
    let part = p.mount as usize;
    p.pos = beast.world_of(part, p.local);
    p.mount = monster::NO_PART;
    p.grounded = false;
}

// ---------------------------------------------------------------------------
// The Elementalist's beam
// ---------------------------------------------------------------------------

impl World {
    /// Fire the auto, if this fighter has one out.
    ///
    /// Run on **every** active frame rather than only the first, for the same
    /// reason every other move's hitbox is live for its whole active window:
    /// the volume that is drawn has to be the volume that is tested, and a
    /// shot should still catch someone who steps into the line during it. The
    /// move's one hit is what keeps that from being two shots -- whatever the
    /// beam meets first ends it.
    ///
    /// Nothing here travels. The ray, the stone it kicks and the fire it
    /// lights all happen on the frame the move comes out; the fire bolt is the
    /// only part with a speed, and it starts at the pillar rather than at her.
    fn fire_the_beam(&mut self, i: usize) {
        self.players[i].beam_reach = Fx::ZERO;
        let shooter = self.players[i];
        let Action::Active { kind, .. } = shooter.action else {
            return;
        };
        if !bolt::throws_a_beam(&shooter, kind) || shooter.health <= 0 {
            return;
        }

        // The line: her hand to the point the crosshair was on when she threw
        // it. Not a direction and a range -- see `crate::aim`.
        let beam = beam_of(&shooter);
        let m = moves::get(shooter.class, kind);
        let field = stones::gather(&self.players);
        let seen = self.players;
        let effects = self.effects;
        let beast = self.monster;
        // In a hunt the two of you are on the same side, so the only thing
        // worth shooting is the creature. One condition, in one place.
        let versus = beast.is_none();
        let scene = Scene {
            stones: &field,
            players: &seen,
            effects: &effects,
            quarry: beast.as_ref(),
        };
        let met = aim::first_along(beam, m.radius, i as u8, &scene, bolt::targets(versus));

        // How long the line actually is, whether or not it still has a hit to
        // spend. A spent beam is still a beam, and it is still drawn.
        self.players[i].beam_reach = met.map_or(beam.length(), |c| c.dist());
        if shooter.hit_used {
            return;
        }

        match met {
            Some(Contact::Fighter { index, .. }) => {
                if bolt::poke(&mut self.players[index], shooter.pos, m.damage)
                    == bolt::Poked::Parried
                {
                    self.players[i].action = Action::Stagger {
                        left: t::parry_stagger(),
                    };
                    self.players[i].stun_total = t::parry_stagger();
                    self.players[index].parried = PARRY_FLOURISH;
                }
                self.players[i].hit_used = true;
            }
            // The stone goes along the line, pitch included: through the
            // ground aimed down it, up into the air aimed above it.
            Some(Contact::Stone { index, .. }) => {
                stones::kick(&mut self.players, index, beam.dir());
                self.players[i].hit_used = true;
            }
            // A hazard is not a wall. The beam does not stop at the fire, it
            // lights it, and what leaves the pillar is the poke.
            Some(Contact::Fire { dist }) => {
                bolt::light(&mut self.bolts, i as u8, beam.at(dist), beam.dir());
                self.players[i].hit_used = true;
            }
            Some(Contact::Quarry { part, .. }) => {
                if let Some(mut beast) = self.monster {
                    beast.take_hit(part, m.damage);
                    self.monster = Some(beast);
                    self.players[i].hit_used = true;
                }
            }
            None => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Trading with the creature
// ---------------------------------------------------------------------------

impl World {
    /// Both directions of the exchange, once both sides have moved.
    fn trade_with_the_creature(&mut self) {
        let Some(mut beast) = self.monster else {
            return;
        };

        // What the creature has out. Every fighter it reaches is hit on the
        // same frame -- spending the hit on whoever happened to be checked
        // first would make a coop partner a shield.
        if let (Doing::Active { kind, .. }, false) = (beast.doing, beast.hit_used) {
            let m = monster::attack(kind);
            let anchor = beast
                .hit_volume()
                .map(|(a, _, _, _)| beast.stance().to_world(a))
                .unwrap_or(beast.pos);
            let mut landed = false;
            for i in 0..MAX_PLAYERS {
                let victim = self.players[i];
                if victim.health <= 0 || victim.action.invulnerable() {
                    continue;
                }
                if !beast.reaches(victim.pos, victim.hurt_height(), t::body_radius()) {
                    continue;
                }
                let away = V3::new(
                    victim.pos.x.sub(anchor.x),
                    Fx::ZERO,
                    victim.pos.z.sub(anchor.z),
                )
                .normalized();
                let facing_it =
                    victim.facing.dot(away.scale(Fx::ONE.neg())).raw() >= t::guard_arc_cos().raw();
                let guarding = !m.unblockable && victim.action.guarding() && facing_it;
                let parried = !m.unblockable
                    && matches!(victim.action, Action::Guard { held } if held < t::parry_window())
                    && facing_it;
                apply_hit(
                    &mut self.players[i],
                    Hit {
                        damage: m.damage,
                        hitstun: m.hitstun,
                        blockstun: m.blockstun,
                        knockback: m.knockback,
                        launch: m.launch,
                        grabs: 0,
                        by: QUARRY,
                        dir: away,
                        blocked: guarding,
                        parried,
                    },
                );
                if parried {
                    // Parrying something that size staggers it. This is the one
                    // thing in the fight that interrupts an active frame, and
                    // it is the reward for the hardest read available.
                    beast.doing = Doing::Flinch {
                        left: t::parry_stagger(),
                    };
                }
                if !parried && !guarding {
                    fall_off(&mut self.players[i], &beast);
                }
                landed = true;
            }
            if landed {
                beast.hit_used = true;
            }
        }

        // What the fighters have out.
        for i in 0..MAX_PLAYERS {
            let attacker = self.players[i];
            let Some(box_out) = hitbox(&attacker) else {
                continue;
            };
            if box_out.spent || attacker.health <= 0 {
                continue;
            }
            let Some(kind) = attacker.action.attack_kind() else {
                continue;
            };
            // The beam already had its answer, on the frame it was fired.
            if bolt::throws_a_beam(&attacker, kind) {
                continue;
            }
            let Some(part) = part_under(&beast, &attacker, &box_out) else {
                continue;
            };
            let raw = Fx::from_int(moves::get(attacker.class, kind).damage)
                .mul(preying(attacker.class, beast.disabled()))
                .to_int();
            let dealt = beast.take_hit(part, raw);
            self.players[i].heal(moves::get(attacker.class, kind).leeched(dealt));
            self.players[i].hit_used = true;
        }

        // Nothing to stand on any more.
        if !beast.alive() {
            for p in self.players.iter_mut() {
                fall_off(p, &beast);
            }
        }
        self.monster = Some(beast);
    }
}
