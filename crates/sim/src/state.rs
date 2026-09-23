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
use crate::camera;
pub use crate::class::Shield;
use crate::class::{self, Class, Force, Form, Ghost, Mechanic};
use crate::debris::{self, MAX_DEBRIS, Shrapnel};
use crate::effects::{Effect, EffectKind, GRASP_ARMS, LOTUS_BLADES, MAX_EFFECTS, QUARRY_VICTIM};
use crate::fixed::Fx;
use crate::gust::{self, Gale, MAX_GUSTS};
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

/// `Player::blinked` when she did not blink this frame.
pub const NO_POOL: u8 = u8::MAX;

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
/// Right click, on the one other class with nothing there: the Elementalist
/// has no shield to guard with either, and Cataclysm takes the button instead
/// of leaving it dead. See [`clicked_move`],
/// `crate::effects::EffectKind::FireTornado` and `crate::debris`.
pub const SLOT_HEAVY: u8 = 3;
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
    /// Winding a move up for as long as the button is held, up to a cap.
    /// `held` counts **up**.
    ///
    /// The one place in the game where aiming is still live after a button
    /// went down. Everywhere else the rule is that facing locks the moment a
    /// move starts, and it is load-bearing -- a hitbox you can drag around
    /// during its active frames takes whiff punishment out behind the shed.
    /// Nothing is out during a channel: it is the *aiming*, and the lock lands
    /// where it always does, on the first frame of startup.
    ///
    /// What a channel chooses is a number the move otherwise takes from the
    /// table. The Grasp chooses its reach; nothing else channels yet.
    Channel {
        kind: u8,
        held: u16,
    },
}

impl Action {
    pub const fn actionable(self) -> bool {
        matches!(self, Action::Free)
    }

    pub const fn guarding(self) -> bool {
        matches!(self, Action::Guard { .. })
    }

    /// Which move is running, across all three of its phases -- and the
    /// channel in front of them, which is the same commitment with no hitbox.
    pub const fn attack_kind(self) -> Option<u8> {
        match self {
            Action::Startup { kind, .. }
            | Action::Active { kind, .. }
            | Action::Recovery { kind, .. }
            | Action::Channel { kind, .. } => Some(kind),
            _ => None,
        }
    }

    /// Winding up, and still choosing. The aim is live and nothing is out.
    pub const fn channelling(self) -> Option<(u8, u16)> {
        match self {
            Action::Channel { kind, held } => Some((kind, held)),
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
            Action::Channel { .. } => 10,
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
            Action::Guard { held } | Action::Channel { held, .. } => held,
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
    /// Health that is no longer red and not yet gone.
    ///
    /// The Blood mage's, and zero on everybody else (`Class::wounds_go_grey`).
    /// Every cost she pays and every hit she takes moves red into here; it
    /// fades on its own clock (`tuning::grey_fade`), and the only thing that
    /// turns it back is a drink from an essence pool. While it is open it is
    /// reach: the scythe's length and damage ride on it, which is why a full
    /// bar is not simply the best place to be. `health + grey` never exceeds
    /// the bar. See `docs/design/blood-mage.md` §"Grey health".
    pub grey: i32,
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
    /// Frames of **haste** left, and the mirror of `slowed`.
    ///
    /// One source in the game: standing in the field a Judgement left, which is
    /// the half of that ability that is for the caster rather than against
    /// whoever she threw it at. Refreshed every frame she is in it and given
    /// the same tail a slow has, so leaving it runs down rather than snapping.
    ///
    /// There is no `haste_mul` beside it, unlike the slow, and that asymmetry
    /// is honest rather than an oversight: a slow can arrive from a spike, a
    /// lotus, a stone or a Sweep and they are not all worth the same, so the
    /// strength has to travel with it. There is exactly one thing that hastens
    /// anybody, so its strength is a knob (`tuning::judgement_field_speed`) and
    /// this is only how much longer it has.
    pub hasted: u16,
    /// What the Dual mage's bar was worth on the frame she committed to the
    /// move she is in the middle of. [`Fx::ONE`] for everybody else, and
    /// meaningless while she is not throwing anything.
    ///
    /// **A cast is worth where you were standing when you pressed the button**,
    /// not where the cast's own push has since taken you. The two are different
    /// numbers because throwing anything at all moves the bar, on the press,
    /// and a finisher moves it a long way -- so a Judgement read live would be
    /// worth its own push, and the one place in the kit that is supposed to be
    /// embarrassing at the centre would be the least embarrassing thing there.
    /// It would also mean the bar on the HUD never matched what the player got.
    ///
    /// See [`depth`], which reads this while she is busy and the live bar when
    /// she is not.
    pub thrown_at: Fx,
    /// Frames of root left. Your feet do not carry you, you cannot dodge and
    /// Frames left of a grab's **bind**: caught, and not going anywhere yet.
    ///
    /// The front of `Action::Held`, not a state beside it. While it runs the
    /// victim stays exactly where the grab closed on them; when it expires the
    /// haul in `drag_the_held` starts. A grab whose catch is already at arm's
    /// length -- the Bulwark's Grapple -- sets it to zero and has neither
    /// phase to speak of.
    ///
    /// Counted on the victim rather than read off the grabber's move because
    /// the haul does not care who started it, and because the number has to
    /// survive the grabber dying mid-drag.
    pub bound: u16,
    /// Was the mechanic button down last frame? Part of the snapshot, so the
    /// press edge survives rollback.
    pub mechanic_held: bool,
    /// Was right click down last frame? The press edge for the Reaver's
    /// mechanic, which is the one class that has its mechanic on a click and so
    /// cannot borrow `mechanic_held`. In the snapshot for the same reason that
    /// one is.
    pub right_held: bool,
    /// Frames a right click stays live, waiting for a frame she can spend it on.
    ///
    /// Zero for everybody else. The Reaver's mechanic is the only input in the
    /// game that is remembered past the frame it was pressed on, and
    /// `crate::shadow` is where the argument for that lives.
    pub shadow_queued: u16,
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
    /// Which effect slot the Blood mage blinked to this frame, or [`NO_POOL`].
    ///
    /// The dodge branch places her in the pool and cannot delete it -- it
    /// only has her body -- so it leaves the slot here for `advance` to spend
    /// on the same frame. In the snapshot for the same reason every edge is:
    /// a frame re-simulated must consume the same pool.
    pub blinked: u8,
    /// Frames of **bleed** left, from a Blood mage's Haemorrhage. Every
    /// `tuning::bleed_tick` frames it takes `tuning::bleed_damage` and spills
    /// that under you wherever you are standing, so a bleeding fighter walks
    /// a trail of her pools behind them -- which is the point: a spike on any
    /// of them chains along the trail. See `World::step_bleeds`.
    pub bleeding: u16,
    /// Whose bleed it is, so the pools are hers. `u8::MAX` for nobody.
    pub bled_by: u8,
    /// Frames left of being **hauled** somewhere by her own Grasp: all four
    /// arms landed on the creature, and the heavier body wins, so she is the
    /// one pulled across the gap to the contact point. Zero for everybody and
    /// almost always for her. See `haul_drive`.
    pub haul: u16,
    /// Where the haul is taking her.
    pub haul_to: V3,
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
    /// How far the camera's framing has swung over to the **airborne** one:
    /// zero with their feet on something, one well above it.
    ///
    /// **A camera number in the snapshot, and it has to be.** Two things put it
    /// here rather than leaving it to the renderer. The eye is where the aiming
    /// ray starts (`crate::camera`), so a framing two peers disagreed about is
    /// two peers aiming differently -- it is in the checksum for the same
    /// reason the camera's knobs are. And it has **memory**: it may only move
    /// so far a frame (`camera::aloft_step`), so a rollback that re-simulated
    /// the middle of a jump without it would come out framing the fight
    /// differently than the first pass did.
    ///
    /// Stepped at the end of the frame, after everything that aims. That is
    /// deliberate: the aim a player takes on frame *n* was taken against the
    /// camera they were looking at, which is the one frame *n-1* left behind.
    /// See [`step_aloft`].
    pub aloft: Fx,

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
    ///
    /// **It is added in exactly one place: [`World::advance`], into the input,
    /// before anything reads one.** A look is used four ways -- the eye, the
    /// ray the crosshair draws out of it, the facing, and the direction `W`
    /// walks -- and they have to be the same angle. See `Input::turned`.
    ///
    /// **It does not reset when you get off,** and must not: zeroing it would
    /// spin the view a quarter turn on the frame you landed. So it is not a
    /// rider's problem alone. Mounting is landing, so a knock that drops you
    /// across the animal for a second leaves some behind, and whatever is wrong
    /// with how it is spent is wrong for the rest of the match.
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
    /// Frames left before each of this class's moves may be thrown again.
    ///
    /// **Per move, and only ever the one you just threw** -- which is what
    /// separates it from a cooldown and is why the combat kernel allows it. The
    /// rest of the kit is always available, so the question it puts to a player
    /// is never *do I have anything* but *what else have I got*. See
    /// `tuning::repeat_lockout`.
    ///
    /// Indexed by move slot and sized to the widest class, so the Champion's
    /// ten fit and everybody else leaves the tail at zero. In the snapshot
    /// because it decides what a button does: a lockout kept outside it would
    /// let a rollback re-throw a move the original frame refused.
    pub repeat_lock: [u16; moves::MAX_SLOTS],
    /// Frames left before an ability that is **still out there** may be used
    /// again -- the recall, not the cast.
    ///
    /// A separate clock from `repeat_lock` because it answers a different
    /// question. `repeat_lock` is *when does this ability come back*; this is
    /// *how soon may I use the half of it I have already paid for*. Running
    /// both off one number is what made the first version of this eat the
    /// Reaver's recall: the send armed a thirty-frame lockout and the press
    /// that brings the shadow home is the same button.
    ///
    /// Zero everywhere until somebody plays it -- see `Move::reactivate`.
    pub reactivate_lock: [u16; moves::MAX_SLOTS],
    /// The reach a channelled move wound up to before its button came back up.
    ///
    /// Kept because the move outlives the channel: the arms of a Grasp are
    /// spawned frames later, in `Action::Active`, and have to converge where
    /// the marker was rather than at the move's full range. Zero on every other
    /// move, and on this one until the button is released.
    pub channelled: Fx,
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
            Mechanic::Meter { value, .. } => value.abs() >= t::meter_deep(),
            Mechanic::Forms { .. } | Mechanic::Blood => true,
        }
    }

    /// Which of this fighter's abilities still have something of their own out
    /// in the world, and so have not finished being used yet.
    ///
    /// **The one place that answers it**, indexed by move slot, and the reason
    /// it is one place is that the answer is stored in two completely different
    /// shapes: the second body is the class mechanic, and the hanging blades
    /// are an entry in the world's effect table. A caller that went looking for
    /// itself would find one of them.
    ///
    /// Read against the scene as it stood at the top of the frame, which is the
    /// same snapshot the aiming ray is cast against and is deterministic for
    /// the same reason.
    pub fn abilities_out(&self, who: usize, scene: &Scene) -> [bool; moves::MAX_SLOTS] {
        let mut out = [false; moves::MAX_SLOTS];
        for (slot, flag) in out.iter_mut().enumerate() {
            // Which slots are worth asking about is `moves::lingers`, so the
            // frame table and the simulation cannot come to different views of
            // which abilities the lockout waits for.
            if !moves::lingers(self.class, slot as u8) {
                continue;
            }
            *flag = match slot as u8 {
                // Send shadow: out from the frame it leaves her shoulder until
                // the frame it is back at it, flight home included.
                SLOT_MECHANIC => shadow::of(self).is_some_and(|s| s.is_out()),
                // Guillotine lotus: out for as long as a blade of it is still
                // hanging or still chasing.
                SLOT_SPECIAL => scene.effects.iter().flatten().any(|e| {
                    e.kind == crate::effects::EffectKind::GuillotineLotus && e.owner == who as u8
                }),
                _ => false,
            };
        }
        out
    }

    /// Is this move still locked out from the last time it was thrown?
    ///
    /// See [`Player::repeat_lock`]. A slot past the end of this class's list
    /// can never be locked, which keeps the answer total for any `kind` a
    /// caller can produce.
    pub fn locked_out(&self, kind: u8) -> bool {
        self.repeat_lock.get(kind as usize).is_some_and(|&f| f > 0)
    }

    /// Everything that has to be true before a press becomes a move.
    ///
    /// One predicate rather than three at each of the six call sites, because
    /// they are asked together every single time and a call site that remembered
    /// only some of them would be a move you could spam on one input path and
    /// not on another.
    ///
    /// **Three cases, and the middle one is the whole reason this takes an
    /// argument.** An ability that is not out is gated by the shared repeat
    /// lockout, which is the ordinary rule. An ability that *is* out, on a
    /// button that reactivates it, is not being used again at all -- the press
    /// is the second half of the one already paid for, so the lockout must not
    /// touch it and its own [`Move::reactivate`] gap is what gates it. An
    /// ability that is out on a button that does *not* reactivate it is simply
    /// not available: the first cast is not finished, so there is nothing for a
    /// second one to be.
    pub fn can_throw(&self, kind: u8, out: &[bool; moves::MAX_SLOTS]) -> bool {
        if !self.mechanic_ready(kind) {
            return false;
        }
        match out.get(kind as usize) {
            Some(true) if moves::reactivates(self.class, kind) => self
                .reactivate_lock
                .get(kind as usize)
                .is_none_or(|&f| f == 0),
            Some(true) => false,
            _ => !self.locked_out(kind),
        }
    }

    /// Start this move's clocks. Called as the move comes out, not as the
    /// button goes down -- a channel spends its own frames first.
    fn lock_repeat(&mut self, kind: u8) {
        let m = moves::get(self.class, kind);
        if let Some(slot) = self.repeat_lock.get_mut(kind as usize) {
            *slot = m.repeat_lock();
        }
        if let Some(slot) = self.reactivate_lock.get_mut(kind as usize) {
            *slot = m.reactivate;
        }
    }

    /// Run the clocks down by a frame.
    ///
    /// **An ability still out in the world holds its lockout at full rather
    /// than spending it**, so the countdown begins on the frame the last of it
    /// comes home. That is the rule that keeps the lockout a charge for
    /// *finishing* with an ability rather than a tax on starting one: sending
    /// the shadow and leaving it standing there for five seconds does not
    /// quietly serve the lockout while it waits.
    ///
    /// Parked rather than paused, which needs no memory of whether it was
    /// parked last frame: re-arming it to full every frame it is out leaves it
    /// at full on the frame it stops being out, which is the same thing and is
    /// one line.
    fn tick_repeat_locks(&mut self, out: &[bool; moves::MAX_SLOTS]) {
        for (kind, f) in self.repeat_lock.iter_mut().enumerate() {
            if out[kind] {
                *f = moves::get(self.class, kind as u8).repeat_lock();
            } else {
                *f = f.saturating_sub(1);
            }
        }
        for f in self.reactivate_lock.iter_mut() {
            *f = f.saturating_sub(1);
        }
    }

    /// Open a bleed, or refresh one. A second bolt on a bleeding fighter
    /// restarts the clock rather than stacking: two bleeds ticking at once
    /// would be a damage-over-time build, and this is a marker, not a build.
    pub fn bleed(&mut self, by: u8, frames: u16) {
        self.bleeding = self.bleeding.max(frames);
        self.bled_by = by;
    }

    /// Take a slow. The strongest one on you is the one that counts, and any
    /// of them refreshes the clock.
    pub fn slow(&mut self, frames: u16, mul: Fx) {
        if self.slowed == 0 || mul.raw() < self.slow_mul.raw() {
            self.slow_mul = mul;
        }
        self.slowed = self.slowed.max(frames);
    }

    /// The mirror: keep this fighter fast for at least `frames` more.
    ///
    /// No multiplier argument, because there is one source of haste in the
    /// game and its strength is a knob -- see [`Player::hasted`].
    pub fn hasten(&mut self, frames: u16) {
        self.hasted = self.hasted.max(frames);
    }

    /// Be caught and held by somebody.
    ///
    /// A grab is not knockback. The victim is pinned to the grabber and goes
    /// wherever they go -- see `drag_the_held` -- which is what makes it a
    /// commitment for *both* of them rather than a shove with a longer stun.
    ///
    /// Its own method because two different things do it now: a swing that
    /// lands, and the Blood mage's Grasp closing on somebody every one of its
    /// arms caught. Written twice, the second one would have been the version
    /// that forgot to zero the velocity.
    ///
    /// `bound` is how many of those frames are spent standing still before the
    /// haul begins -- zero for a grab that already has you at arm's length.
    pub fn seized(&mut self, by: u8, frames: u16, bound: u16) {
        self.stun_total = frames;
        self.action = Action::Held { left: frames };
        self.held_by = by;
        self.bound = bound.min(frames);
        self.vel.x = Fx::ZERO;
        self.vel.z = Fx::ZERO;
    }

    /// Are this fighter's options gone?
    ///
    /// Staggered or held. **Hitstun is deliberately not in the list**,
    /// and that is the whole of the definition: hitstun happens on every hit
    /// anybody lands, so counting it would turn "increased damage to disabled
    /// enemies" into "increased damage from the second hit onward", which is a
    /// flat damage bonus wearing a costume.
    ///
    /// What is in the list is what `ability-spec.md` calls a hard stop, and the
    /// design only allows those behind a hard condition -- a parry for the
    /// stagger, and for the hold either a grab that landed or every one of a
    /// Grasp's four arms. Each had to be *earned*, which is exactly what a
    /// payoff should be waiting on. Blockstun is not one: they blocked, which
    /// was the correct decision, and rewarding the attacker for it would make
    /// guarding worse than standing still.
    pub const fn disabled(&self) -> bool {
        matches!(self.action, Action::Stagger { .. } | Action::Held { .. })
    }

    /// Pay for a move out of your own health, and take the return on one.
    ///
    /// Both clamp. Self-damage stops at one -- dying to your own button is not
    /// a decision anybody made, and the Dual mage's meter burn already works
    /// this way -- and healing stops at full, so a Blood mage cannot bank
    /// health above the bar by farming a field.
    /// What a cast that costs `percent` of her current health would take
    /// right now, in points.
    ///
    /// **A percentage of current red, not of the bar.** Casting at full health
    /// opens a big wound -- which is how she gets power quickly, since the
    /// wound is reach -- and casting at low health opens a small one, so she
    /// is never burning herself to death trying to get back into the fight.
    /// Clamped at one: dying to your own button is not a decision anybody
    /// made.
    pub fn cost_of(&self, percent: i32) -> i32 {
        (self.health * percent.max(0) / 100)
            .min(self.health - 1)
            .max(0)
    }

    pub fn spend_health(&mut self, percent: i32) {
        // What was actually paid is what turns grey.
        let paid = self.cost_of(percent);
        self.health -= paid;
        if self.class.wounds_go_grey() {
            self.grey += paid;
        }
    }

    /// Take `damage` off the bar, and give back what actually came off.
    ///
    /// **The one place enemy damage lands**, whichever path delivered it -- a
    /// swing, a field ticking, a bolt, a stone, the floor after a spike -- so
    /// that the Blood mage's grey is bookkept once. A killing blow on somebody
    /// with forty health left is worth forty, not its listed damage, which is
    /// what the return is for: anything paid out of a hit has to be paid out
    /// of what the hit was worth.
    pub fn wound(&mut self, damage: i32) -> i32 {
        let dealt = damage.min(self.health).max(0);
        self.health -= dealt;
        if self.class.wounds_go_grey() {
            self.grey += dealt;
        }
        dealt
    }

    pub fn heal(&mut self, amount: i32) {
        // Never above the bar, and never into the grey: a heal from anywhere
        // but a pool cannot close a wound this class has open. Nothing heals
        // her that way any more, and this is what keeps that true if something
        // shared tries.
        if amount > 0 && self.health > 0 {
            self.health = (self.health + amount).min(t::max_health() - self.grey);
        }
    }

    /// Turn up to `amount` of grey back into red, and give back how much was.
    ///
    /// The Blood mage's only heal. Grey is the ceiling -- she cannot get back
    /// past where she stood a moment ago, only to it -- so a drink with no
    /// grey to convert is worth nothing, and the pool it came from is not
    /// charged for it (see `World::drink`).
    pub fn drink(&mut self, amount: i32) -> i32 {
        if self.health <= 0 {
            return 0;
        }
        let took = amount.min(self.grey).max(0);
        self.health += took;
        self.grey -= took;
        took
    }

    /// How much of the bar is grey, nought to one. What the scythe reads.
    pub fn grey_share(&self) -> Fx {
        let max = t::max_health().max(1);
        Fx::ratio(self.grey.clamp(0, max), max)
    }

    /// Standing on the creature.
    pub fn aboard(&self) -> bool {
        self.mount != monster::NO_PART
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
            grey: 0,
            action: Action::Free,
            grounded: true,
            jump_hold: 0,
            air_stall: 0,
            air_dodged: false,
            hit_used: false,
            repeat_lock: [0; moves::MAX_SLOTS],
            reactivate_lock: [0; moves::MAX_SLOTS],
            class: Class::Bulwark,
            mechanic: Mechanic::Shield(Shield::Held),
            rounds_won: 0,
            crouching: false,
            slowed: 0,
            slow_mul: Fx::ONE,
            hasted: 0,
            thrown_at: Fx::ONE,
            bound: 0,
            mechanic_held: false,
            right_held: false,
            shadow_queued: 0,
            held_by: NOBODY,
            space_held: false,
            leap_used: false,
            blinked: NO_POOL,
            bleeding: 0,
            bled_by: u8::MAX,
            haul: 0,
            haul_to: V3::ZERO,
            slam: Fx::ZERO,
            stride: 0,
            air_frames: 0,
            since_landed: 0,
            parried: 0,
            crouched_for: 0,
            stun_total: 0,
            aloft: Fx::ZERO,
            mount: monster::NO_PART,
            local: V3::ZERO,
            carry_yaw: Fx::ZERO,
            grip_vel: V3::ZERO,
            grip_settle: 0,
            beam_reach: Fx::ZERO,
            aim_path: Path::default(),
            channelled: Fx::ZERO,
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
    /// Debris thrown by Cataclysm destroying a structure. The same kind of
    /// thing a fire bolt is -- a real velocity, stepped frame by frame -- and
    /// for the same reason: see [`crate::debris`].
    pub debris: Shrapnel,
    /// The Elementalist's air shots in flight. The only thing a fighter throws
    /// that is neither part of a mechanic nor lit by something else -- she
    /// throws these out of her own hands, with her feet off the floor. See
    /// [`crate::gust`].
    pub gusts: gust::Flight,
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
            debris: [None; MAX_DEBRIS],
            gusts: [None; MAX_GUSTS],
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
        self.debris = [None; MAX_DEBRIS];
        self.gusts = [None; MAX_GUSTS];
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
    pub fn advance(&mut self, wire: [Input; MAX_PLAYERS]) {
        self.frame = self.frame.wrapping_add(1);

        if let Phase::RoundOver { winner, left } = self.phase {
            if left > 0 {
                self.phase = Phase::RoundOver {
                    winner,
                    left: left - 1,
                };
                // Bodies still settle during the pause; nothing else acts.
                // The camera keeps working through it -- a fighter knocked out
                // of the air would otherwise have the view frozen mid-swing
                // for the whole round-over pause.
                let field = stones::gather(&self.players);
                for p in self.players.iter_mut() {
                    settle(p);
                    advance_clocks(p);
                    step_aloft(p, &field);
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

        // **The creature's turn goes into the look, here and nowhere else.**
        //
        // A rider's look angle is the aim on the wire plus whatever the animal
        // has turned under them (`Player::carry_yaw`), and *everything* that
        // reads a look has to read the same one: the eye, the aiming ray out of
        // it, the facing, and the direction `W` walks. Folding it into the
        // input once, before anybody reads it, is what makes that true by
        // construction.
        //
        // It used to be added at the facing and at the movement and at neither
        // of the other two, which put the camera at the mouse's angle and the
        // fighter at the mouse's angle plus the ride. The character came out
        // drawn looking off to one side of the screen, the crosshair stopped
        // meaning what it says, and because `carry_yaw` survives coming off --
        // it has to, or dismounting would whip the view round -- so did the
        // discrepancy. A brush against the animal was enough to pick some up.
        let inputs: [Input; MAX_PLAYERS] = std::array::from_fn(|i| {
            let p = &mut self.players[i];
            if p.aboard() {
                p.carry_yaw = crate::math::wrap_turns(p.carry_yaw.add(spin));
            }
            wire[i].turned(p.carry_yaw)
        });

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
        let frame = self.frame;
        for (i, (p, input)) in self.players.iter_mut().zip(inputs).enumerate() {
            let scene = Scene {
                stones: &field,
                players: &seen,
                effects: &effects,
                quarry: beast.as_ref(),
            };
            step_player(p, i, input, &field, beast.as_ref(), &scene, carrying[i]);
            advance_clocks(p);
            step_aloft(p, &field);
            fade_grey(p, frame);
        }
        // A blink spends its pool. Done here rather than in the dodge branch
        // because the branch only has her body, and the pool is an effect.
        for i in 0..MAX_PLAYERS {
            let slot = self.players[i].blinked as usize;
            if slot < MAX_EFFECTS && self.effects[slot].is_some_and(|e| e.is_a_pool()) {
                self.effects[slot] = None;
            }
            self.players[i].blinked = NO_POOL;
        }

        // What a move does *as it comes out*, on its first active frame: the
        // leap of a leaping move, and whatever it leaves standing in the world.
        for (i, input) in inputs.into_iter().enumerate() {
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
                // The spear's takeoff spends the plant on a direction as well
                // as on height. On the same frame as the lift, because they are
                // one motion: the butt of the spear hits the floor and both
                // halves of the launch come out of it.
                if p.class == Class::Champion && kind == moves::champion::POLE_DRIVE {
                    pole_drive_boost(&mut self.players[i], input);
                }
            }
            // `E` on the Reaver: the second body goes out, or comes home
            // through whatever is in the way. On the first active frame like
            // everything else a move does, so the startup is a window somebody
            // can punish rather than a formality.
            if p.class == Class::ShadowReaver && kind == SLOT_MECHANIC {
                self.order_the_shadow(i, p.aim_at());
            }
            // Her two air shots leave her hand here, along the line
            // `crate::aim` already solved -- and *along* it rather than to the
            // end of it, the same rule the two travelling effects follow: the
            // crosshair picked a direction, and a disc thrown at somebody four
            // metres away still travels its whole range. See `crate::gust`.
            if let Some(gale) = Gale::thrown_by(p.class, kind) {
                gust::throw(&mut self.gusts, i as u8, gale, p.aim_path);
            }
            // Landfall arriving: the slab of rock she drives up in front of
            // her. **The one structure in the game the crosshair does not
            // place** -- she is landing, not aiming -- so where it goes is
            // `aim::planted_ahead` rather than either of the two lines of
            // effect, and it spends the cap of three through `stones::raise`
            // like the mechanic key does. Her own facing is the direction it
            // leans away along, locked when the move started, so a slab thrown
            // up behind her is not a thing a mouse flick can produce.
            if p.class == Class::Elementalist && kind == moves::elementalist::LANDFALL {
                let field = stones::gather(&self.players);
                let at = aim::planted_ahead(p.pos, p.facing, t::landfall_ahead(), &field);
                stones::raise(
                    &mut self.players[i],
                    class::Structure::slammed(at, p.facing),
                );
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
                // A channelled move goes as far as it was wound to, not as far
                // as its row says it could -- see `step_channel`.
                let reach = if m.channels() { p.channelled } else { m.reach };
                let mut born = Effect::cast(leaves, i as u8, p.class, kind, from, along, reach);
                born.power = depth(&p);
                // **How long a thing she leaves behind lasts is where she was
                // standing when she threw it.** The one place the depth curve
                // is allowed to move a frame count -- a move's own startup,
                // active and recovery never change, because that is the
                // contract between two players, but how long a field burns
                // afterwards is the field's business. At the centre Judgement
                // leaves a puddle that is gone before anyone walks through it.
                // A no-op for every other class. See [`depth`].
                born.life = lasting(&p, born.life);
                // The Black spike reads the floor. On bare ground it is a
                // spike: the move's own disc does the damage, the launch and
                // the slow, and this is only the thing you can see. On one
                // of her pools the whole pool erupts at the pool's radius,
                // launching and slowing everything in it, and drinks the
                // pool in one go -- the payoff placement, and the pool is
                // spent whether or not she had grey to fill.
                let mut place = true;
                if leaves == EffectKind::BlackSpike {
                    match self.pool_covering(i as u8, born.pos) {
                        Some(slot) => {
                            // The eruption delivers the hit; the disc must
                            // not land a second one. And the eruption chains
                            // -- see `erupt_from`.
                            self.players[i].hit_used = true;
                            self.erupt_from(i, kind, slot);
                            place = false;
                        }
                        None => born.reach = m.radius,
                    }
                }
                if place {
                    spawn_effect(&mut self.effects, born);
                }
            }
        }

        // The Elementalist's auto and her heavy: two different beams, the same
        // instant resolution, and never both at once for one fighter -- reset
        // once, here, rather than in each, so neither can clobber the other's
        // answer on the frame it is the one actually firing.
        for i in 0..MAX_PLAYERS {
            self.players[i].beam_reach = Fx::ZERO;
            self.fire_the_beam(i);
            self.fire_the_cataclysm(i);
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
                // The Blood mage's double duty: a hit over one of her pools
                // drinks from it, and then spills what it dealt onto the
                // floor. In that order, so a hit on bare floor returns
                // nothing -- the smear it leaves is for the next one.
                if let Some(kind) = snapshot[attacker].action.attack_kind() {
                    let m = moves::get(snapshot[attacker].class, kind);
                    // A spike on bare floor is a spike: damage, a launch, and
                    // a short slow. The slow is what makes leaving cost time.
                    if snapshot[attacker].class == Class::BloodMage
                        && kind == moves::blood::BLACK_SPIKE
                        && !hit.blocked
                        && !hit.parried
                    {
                        self.players[defender].slow(t::slow_frames(), t::spike_slow());
                    }
                    if dealt > 0 {
                        if let Some(hb) = hitbox(&snapshot[attacker]) {
                            self.drink_over(attacker, &m, hb.from, hb.to, Some(defender));
                        }
                        self.spill_under(
                            attacker as u8,
                            snapshot[attacker].class,
                            kind,
                            &snapshot[defender],
                            dealt,
                        );
                    }
                }
                self.players[attacker].hit_used = true;
                // The aerial spear pays its shove out on contact rather than
                // on the throw: catch somebody with the fan and it kicks you
                // the way you are holding, so it is a repositioning tool you
                // have to earn. Read from the live input rather than from a
                // direction locked at the throw, because the whole point is
                // that you choose where to go *as* it connects.
                champion_fan_boost(&mut self.players[attacker], inputs[attacker]);
                // The dark Sweep's half of the form split: their legs, and her
                // health, per target it caught. A no-op for everybody else.
                if snapshot[attacker]
                    .action
                    .attack_kind()
                    .is_some_and(|kind| sweeping_dark(&snapshot[attacker], kind))
                {
                    dual_sweep_dark(
                        &mut self.players,
                        attacker,
                        defender,
                        !hit.blocked && !hit.parried,
                    );
                }
                // And the other half of the hammer finisher: the knock-up went
                // out in `resolve_hit`, and this is the Champion following it
                // off the floor on the same frame.
                champion_leap_with_them(&mut self.players[attacker], hit.blocked || hit.parried);
                // A chain link that actually landed may cut its recovery short
                // for the next one. Blocked and parried links pay in full, so
                // the string ends on a defender who answered it -- see
                // `chain_cancel`.
                confirm_chain(&mut self.players[attacker], hit.blocked || hit.parried);
                if hit.parried {
                    self.players[attacker].action = Action::Stagger {
                        left: t::parry_stagger(),
                    };
                    self.players[attacker].stun_total = t::parry_stagger();
                    self.players[defender].parried = PARRY_FLOURISH;
                }
            }
        }

        // The scythe collects: any pool of hers its volume passes over on an
        // active frame is drunk, hit or no hit. The blade is the thing she
        // puts through the blood, and it does not need a body in the way.
        for i in 0..MAX_PLAYERS {
            self.collect_with_the_scythe(i);
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
                            interrupts: true,
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
        debris::step(
            &mut self.debris,
            &mut self.players,
            &standing,
            self.monster.is_none(),
            &mut self.monster,
        );
        gust::step(
            &mut self.gusts,
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
        for d in &self.debris {
            match d {
                Some(d) => {
                    h.write_u32(d.owner as u32 + 1);
                    hash_v3(&mut h, &d.pos);
                    hash_v3(&mut h, &d.dir);
                    h.write_i32(d.travelled.raw());
                }
                None => h.write_u32(0),
            }
        }
        for g in &self.gusts {
            match g {
                Some(g) => {
                    h.write_u32(g.owner as u32 + 1);
                    // Which of the two, on the wire: a bolt and a disc at the
                    // same place going the same way are not the same shot, and
                    // a checksum that could not tell them apart would let one
                    // peer's disc be the other's bolt.
                    h.write_u32(g.gale as u32);
                    hash_v3(&mut h, &g.pos);
                    hash_v3(&mut h, &g.dir);
                    h.write_i32(g.travelled.raw());
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
                    h.write_u64(e.struck);
                    h.write_i32(e.banked);
                    h.write_i32(e.power.raw());
                    h.write_i32(e.reach.raw());
                    hash_v3(&mut h, &e.pos);
                    hash_v3(&mut h, &e.dir);
                    hash_v3(&mut h, &e.home);
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
            h.write_i32(p.grey);
            h.write_u32(p.grounded as u32);
            h.write_u32(p.air_dodged as u32);
            h.write_u32(p.slowed as u32);
            h.write_i32(p.slow_mul.raw());
            h.write_u32(p.hasted as u32);
            h.write_i32(p.thrown_at.raw());
            h.write_u32(p.bound as u32);
            h.write_u32(p.mechanic_held as u32);
            h.write_u32(p.right_held as u32);
            h.write_u32(p.shadow_queued as u32);
            h.write_u32(p.space_held as u32);
            h.write_u32(p.leap_used as u32);
            h.write_u32(p.blinked as u32);
            h.write_u32(p.bleeding as u32);
            h.write_u32(p.bled_by as u32);
            h.write_u32(p.haul as u32);
            hash_v3(&mut h, &p.haul_to);
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
            h.write_i32(p.aloft.raw());
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
            h.write_i32(p.channelled.raw());
            for f in &p.repeat_lock {
                h.write_u32(*f as u32);
            }
            for f in &p.reactivate_lock {
                h.write_u32(*f as u32);
            }
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
                h.write_i32(m.strain);
                h.write_u32(m.stride as u32);
                h.write_u32(m.beat as u32);
                h.write_u32(m.slowed as u32);
                h.write_i32(m.slow_mul.raw());
                h.write_u32(m.rooted as u32);
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
                for slot in &m.brain.cooldown {
                    h.write_u32(*slot as u32);
                }
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
    /// Does landing this take the victim's frames away?
    ///
    /// **Almost everything does, and the exception is the point.** A hit
    /// normally writes `Action::HitStun` over whatever the victim was doing, so
    /// even a hit with `hitstun: 0` cancels the move they were in the middle
    /// of -- they are free again next frame, but the attack they had committed
    /// to is gone. That is a full interrupt however small the number is, and a
    /// move that can be thrown instantly and interrupts is the panic button
    /// that makes commitment optional.
    ///
    /// False means it lands without touching their action at all: the damage
    /// and the slow arrive, and the victim keeps swinging. See
    /// `World::recall_cuts`, which is the one thing in the game that wants
    /// this, and `docs/design/kits/shadow-reaver.md` for why.
    ///
    /// It is about **frames**, not about force. Knockback is still whatever the
    /// number says, because moving somebody is not the same as stopping them.
    pub interrupts: bool,
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
    /// Present when the volume is a **section of a ring** rather than a
    /// capsule, which is one shape in the game: the Dual mage's wing.
    ///
    /// When it is here it is what the hit test uses and what the overlay draws;
    /// `from` and `to` are then the section's **leading edge**, the line its tip
    /// is on, so that everything which only understands a line still has a true
    /// one to draw rather than a reconstruction of its own.
    pub sector: Option<crate::math::Sector>,
    /// The last frame of a wing: its edge alone, and it hits harder. See
    /// `tuning::wing_tipper`.
    pub tipper: bool,
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
    let mut m = moves::get(p.class, kind);
    // **The Dual mage's bar changes how big this is.** One multiply, here,
    // rather than a branch per ability. Every other class, and both of her
    // autos, get exactly one back and nothing changes. See [`depth`].
    m.radius = m.radius.mul(depth_of_size(p, kind));
    // **And the Blood mage's grey changes how far this reaches.** The same
    // shape of rule -- one multiply, here, one for everybody else -- and the
    // one reach in the game that scales. See [`live_reach`].
    m.reach = live_reach(p, &m);
    // And how wide it is: the essence around the blade. See [`live_radius`].
    m.radius = live_radius(p, &m);
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
        sector: None,
        tipper: false,
    };
    let (from, to, flat) = match m.aim() {
        // A line from her hand to the point the crosshair was on, ending
        // wherever the shot actually stopped.
        aim::Kind::Skillshot => {
            let beam = beam_of(p);
            // **The volume is the whole line it flew**, from the hand to the
            // point the crosshair picked. `beam_reach` is the exception rather
            // than the rule: it is set by the two abilities that resolve
            // themselves the instant they fire and stop at whatever they met
            // (`fire_the_beam`, `fire_the_cataclysm`), and it is zero for
            // everything else. Reading it unconditionally made every other
            // skillshot in the game a bubble at the caster's own chest -- which
            // is exactly what the Lance was until this line was written, a line
            // skillshot with a four-metre reach that could only hit somebody
            // standing on top of her.
            let stop = if p.beam_reach.raw() > 0 {
                beam.at(p.beam_reach)
            } else {
                beam.to
            };
            (beam.from, stop, false)
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
            // A section of a torus lying flat: a thin curved blade that opens
            // from nothing to its whole span over the active window, with its
            // leading edge arriving in front of the punching fist. See
            // `moves::Shape::Wing`.
            //
            // The volume is the section itself rather than a line standing in
            // for one -- `math::Sector` is the shape, and the hit test and the
            // overlay both read it. A wing curves around the thing that threw
            // it, and a straight line through that either misses the inside of
            // the curve or claims the outside of it.
            moves::Shape::Wing => {
                let ring = moves::wing(p.pos, p.facing, p.aim_dir(), p.grounded, &m);
                // Its own progression rather than `swing_progress`, and for a
                // reason that matters: that one saturates a frame early, so a
                // wing would reach the front on the frame *before* its last one
                // and a body standing there would be caught by the body of the
                // section rather than by the tip. The tip has to be the first
                // thing to arrive in front of her or it is not a tip.
                let elapsed = m.active.saturating_sub(left);
                let through = Fx::ratio(elapsed as i32, m.active.max(2) as i32 - 1);
                // **The last frame is the tip**: a bubble at the foremost point
                // of the ring, and the only thing the whole move ever puts
                // there. The wing opens behind it and stops short, so a body
                // standing in front of her is caught by the tip or by nothing
                // -- which is what makes landing it a decision about distance,
                // taken a sixth of a second earlier, rather than a damage bonus
                // attached to a frame number.
                //
                // A bubble rather than the last slice of the section, because a
                // slice of a ring is metres of arc: it caught everything across
                // the whole front of her and the "tip" was the widest part of
                // the move. A point at the end of the blade is the thing the
                // design has always described.
                if left == 0 {
                    let tip = ring.tip();
                    return Some(Hitbox {
                        from: tip,
                        to: tip,
                        radius: t::wing_tip_radius(),
                        flat: false,
                        hits_crouching: m.hits_crouching,
                        unblockable: m.unblockable,
                        spent: p.hit_used,
                        sector: None,
                        tipper: true,
                    });
                }
                let sector = ring.opened(through);
                // The leading edge, for everything that can only draw a line.
                return Some(Hitbox {
                    from: sector.point(Fx::ZERO, Fx::ONE),
                    to: sector.point(Fx::ONE, Fx::ONE),
                    radius: m.radius,
                    flat: false,
                    hits_crouching: m.hits_crouching,
                    unblockable: m.unblockable,
                    spent: p.hit_used,
                    sector: Some(sector),
                    tipper: false,
                });
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
        sector: None,
        tipper: false,
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
///
/// **And it is her hand, not her sternum**, for a move that declares an arm.
/// The Dual mage's two Lances are one input told apart by the force she is
/// carrying, and the arm it leaves from is the second reading of which one is
/// coming -- so a line that started on the centre line would throw away half of
/// what the player opposite has to go on. `Hand::Centre` is the body's own line
/// and is what every other skillshot in the game still gets.
pub fn beam_of(p: &Player) -> Path {
    let hand = p
        .action
        .attack_kind()
        .map_or(aim::Hand::Centre, |kind| moves::get(p.class, kind).hand);
    Path {
        from: aim::hand_origin(p.pos, p.facing, hand),
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
/// How far a move reaches on this fighter right now.
///
/// The move's own reach for everything but the Blood mage's scythe, which
/// lengthens with the grey on her bar: base at none, `tuning::grey_reach`
/// times it at a full bar, a straight line between. **This is the one reach
/// in the game that scales with a bar**, and `docs/design/aiming.md` refuses
/// that for good reason; it is allowed here on the condition that the blade is
/// drawn at the length it hits at, which `view::scythe` reads from the same
/// function.
pub fn live_reach(p: &Player, m: &moves::Move) -> Fx {
    if !m.rides_the_grey() {
        return m.reach;
    }
    m.reach
        .mul(crate::math::lerp(Fx::ONE, t::grey_reach(), p.grey_share()))
}

/// A move's hit radius on this fighter, right now: the row's, widened by
/// her grey for the scythe. The same line as [`live_reach`], with its own
/// number at the top (`tuning::grey_width`). The weapon is drawn at one size
/// and the volume is drawn as essence around it -- see `view::scythe`.
pub fn live_radius(p: &Player, m: &moves::Move) -> Fx {
    if !m.rides_the_grey() {
        return m.radius;
    }
    m.radius
        .mul(crate::math::lerp(Fx::ONE, t::grey_width(), p.grey_share()))
}

/// What a move's damage is multiplied by for the grey on the caster's bar.
/// The same line as [`live_reach`], with its own number at the top.
fn grey_power(p: &Player, m: &moves::Move) -> Fx {
    if !m.rides_the_grey() {
        return Fx::ONE;
    }
    crate::math::lerp(Fx::ONE, t::grey_damage(), p.grey_share())
}

/// The scythe's reach on this fighter, out of a move: how far the sweep
/// reaches right now. The weapon is drawn at the row's reach; this is what
/// the essence around it is drawn to.
pub fn scythe_reach(p: &Player) -> Fx {
    live_reach(p, &moves::get(p.class, moves::blood::SWEEP))
}

/// Grey fades: `tuning::grey_fade` points a second, spread over the frames.
///
/// Integer arithmetic on the frame counter rather than a fixed-point
/// accumulator in the snapshot: the cumulative fade after `k` frames of the
/// second is `rate * k / 60`, and this frame's share is the difference of two
/// of those, so a second of fading is exactly the rate whatever the rounding
/// did in between. **The only way grey ever goes.** Nothing else takes it off
/// the bar -- not a hit, not a cost, not a heal from anywhere but a pool.
fn fade_grey(p: &mut Player, frame: u32) {
    if p.grey <= 0 {
        return;
    }
    let rate = t::grey_fade().max(0);
    let hz = crate::TICK_HZ as i32;
    let k = (frame % crate::TICK_HZ) as i32;
    let faded = rate * (k + 1) / hz - rate * k / hz;
    p.grey = (p.grey - faded).max(0);
}

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
    // The Elementalist's auto, and her heavy, already happened. Both are a ray
    // fired the moment the move comes out, and what either does depends on
    // what it met first -- none of which this loop can express. See
    // `World::fire_the_beam` and `World::fire_the_cataclysm`.
    if bolt::throws_a_beam(attacker, kind) || throws_a_cataclysm(attacker, kind) {
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

    // The tip of a wing, on the last frame it is out. Timing it is the one
    // piece of execution in an attack that is otherwise thrown constantly.
    let tipped = box_out.tipper;
    // How far out on her own bar the Dual mage is standing, as a multiplier on
    // everything this blow is worth. One for everybody else. See [`depth`].
    let power = depth(attacker);
    // The scythe's tip: the outer part of the blade, measured from her feet,
    // hits harder. Decided per defender rather than per frame, because a
    // sweep can catch one body on the haft and another on the point.
    let on_the_point = m.rides_the_grey()
        && kind == moves::blood::SWEEP
        && defender.pos.sub(attacker.pos).flat_len().raw()
            >= box_out.to.sub(box_out.from).len().mul(t::sweep_tip()).raw();
    let damage_mul = preying(attacker.class, defender.disabled())
        .mul(if tipped { t::wing_tipper() } else { Fx::ONE })
        .mul(if on_the_point {
            t::sweep_tip_damage()
        } else {
            Fx::ONE
        })
        .mul(grey_power(attacker, &m))
        .mul(power);

    let reach = box_out.radius.add(t::body_radius());
    let hit_at_all = if let Some(ring) = box_out.sector {
        // A section of a ring. Its own test, because a body inside the curve is
        // not inside the volume and a straight line cannot say so.
        ring.touches(defender.pos, defender.hurt_height(), reach)
    } else if box_out.flat {
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
    // What the blow does to the space between the two of them, and the sign of
    // it is which way.
    //
    // **A negative knockback is a pull**, and it is measured along the line
    // between the two bodies rather than down the attacker's facing: the point
    // of dragging somebody is that they arrive at *you*, and a wing wraps
    // around her, so a body caught out at the side would otherwise be sent
    // backwards past her instead of in. A push is still the facing, because
    // what a shove does is send them the way the blow was travelling.
    //
    // The **tip** multiplies it either way. That is what makes which arm she
    // punches with a spacing decision as well as a meter one: the light auto's
    // real knockback lives at the far edge of the blade, and on the dark arm
    // the same frame hauls them that much further in. See
    // `tuning::wing_tip_shove`.
    let shove = m
        .knockback
        .mul(power)
        .mul(if tipped { t::wing_tip_shove() } else { Fx::ONE });
    let (push_dir, speed) = if shove.raw() < 0 {
        let toward = V3::new(
            attacker.pos.x.sub(defender.pos.x),
            Fx::ZERO,
            attacker.pos.z.sub(defender.pos.z),
        )
        .normalized();
        (toward, shove.neg())
    } else {
        (attacker.facing, shove)
    };
    // A spike is wasted on somebody standing on the floor: they are already
    // there. It reads as an ordinary heavy hit instead.
    let launch = if m.launch.raw() < 0 && !airborne {
        Fx::ZERO
    } else {
        m.launch.mul(power)
    };
    // And a knock-up is worth more when the attacker elected to go up with it.
    // See `bank_the_leap`: the decision was taken during the wind-up and is
    // being paid for here, on the frame it lands.
    let launch = if going_up_with_them(attacker, kind) {
        launch.mul(t::hammer_leap_launch())
    } else {
        launch
    };
    // **Sweep is one move with two answers**, and taking somebody off their
    // feet is the light one's. Both forms shove -- it is the panic button, and
    // what a panic button is for is moving whoever has got inside the punches
    // -- but light throws them off the floor while dark takes their legs and
    // pays her for it. The slow and the heal arrive in `advance` beside the
    // other things a landed blow does; what belongs here is the half of the
    // split that is a number in this row.
    let launch = if sweeping_dark(attacker, kind) {
        Fx::ZERO
    } else {
        launch
    };

    Some(Hit {
        damage: Fx::from_int(m.damage).mul(damage_mul).to_int(),
        hitstun: m.hitstun,
        blockstun: m.blockstun,
        knockback: speed.mul(air_mul),
        launch,
        grabs: m.grabs,
        by,
        dir: push_dir,
        blocked: guarding,
        parried,
        interrupts: true,
    })
}

/// Is this the Dark form of Sweep?
///
/// The form is the force she is carrying, which is the arm she last punched
/// with -- the same question every other ability on the class asks. Sweep is
/// one move rather than two because the *shape* is the same either way: both
/// arms across the whole front, driven from the hips. Only what happens to the
/// people it caught changes, which is a thing to branch on at the moment of
/// contact rather than a second move with a second animation.
fn sweeping_dark(p: &Player, kind: u8) -> bool {
    p.class == Class::DualMage && kind == moves::dual::SWEEP && carrying(p) == Force::Dark
}

/// The dark Sweep's payment, on the frame it lands: their legs, and her health.
///
/// **Per target caught**, not a share of the damage, which is what makes the
/// answer to being swarmed the same move as the answer to being cornered. It
/// is here in the hit loop rather than in `resolve_hit` because both halves of
/// it act on somebody -- one on the victim and one on the caster -- and
/// `resolve_hit` is a pure question about what a blow is worth.
fn dual_sweep_dark(
    players: &mut [Player; MAX_PLAYERS],
    attacker: usize,
    victim: usize,
    paid: bool,
) {
    // Blocked or parried buys nothing. The slow is a hit effect and the heal is
    // the reward for landing it, which is the same rule leeching already
    // follows everywhere else in the game.
    if !paid {
        return;
    }
    players[victim].slow(t::slow_frames(), t::sweep_slow());
    let owed = Fx::from_int(t::sweep_heal())
        .mul(depth(&players[attacker]))
        .to_int();
    players[attacker].heal(owed);
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
        if hit.interrupts {
            defender.stun_total = hit.blockstun;
            defender.action = Action::BlockStun {
                left: hit.blockstun,
            };
        }
        defender.vel.x = hit.dir.x.mul(hit.knockback);
        defender.vel.z = hit.dir.z.mul(hit.knockback);
    } else {
        defender.wound(hit.damage);
        defender.vel.x = hit.dir.x.mul(hit.knockback);
        defender.vel.z = hit.dir.z.mul(hit.knockback);
        if hit.grabs > 0 {
            defender.seized(hit.by, hit.grabs, 0);
        } else if hit.interrupts {
            // Writing the action is the interrupt, and it happens even when
            // `hitstun` is zero -- `HitStun { left: 0 }` is one frame of
            // nothing, but the move it replaced is gone. Anything that must
            // land without stopping its victim has to skip this branch rather
            // than tune the number to zero. See `Hit::interrupts`.
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
    // Whatever else it did, it broke the dash -- the Champion's Rush, and the
    // Reaver's dash to her shadow with the carry it leaves behind. A commitment
    // you can be hit out of and keep is a commitment with invulnerability
    // attached.
    //
    // **Unless the hit takes no frames.** A dash is frames, so a blow that
    // lands without touching the victim's action must not end one either --
    // see [`Hit::interrupts`], and the recall, which is the one hit that sets
    // it. It is about force rather than frames, and ending somebody's dash is
    // the other thing.
    if hit.interrupts {
        if let Mechanic::Forms { rush, .. } = &mut defender.mechanic {
            *rush = 0;
        }
        shadow::broken_by_a_hit(defender);
        defender.haul = 0;
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
        // A plunge that has not arrived has not finished winding up. The
        // ordinary rule below turns a spent startup into active frames; this
        // one refuses to, for the single move whose wind-up ends on the floor
        // -- see [`waits_for_the_floor`]. The active frames start in
        // `step_player`, on the frame her feet do.
        Action::Startup { kind, .. } if waits_for_the_floor(p, kind) => {
            Action::Startup { kind, left: 0 }
        }
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
        // Already resolved, by `step_channel`, on the frame it happened: a
        // channel is the one action whose next state needs the scene and the
        // buttons, which is more than this function is given. Reaching here
        // means the caller skipped it.
        Action::Channel { .. } => return None,
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
    // The repeat lockouts run down first, and above the rider branch so they
    // run exactly once a frame on either tick. Before the input is read rather
    // than after, so a lockout of `n` costs `n` frames and not `n + 1`.
    //
    // Which abilities are still out is worked out once and handed to both the
    // tick and the gate below, because they have to agree: a lockout parked by
    // one answer and consulted against another is a move that is locked on the
    // frame it was meant to come back.
    let out = p.abilities_out(who, scene);
    p.tick_repeat_locks(&out);
    // Standing on the creature is a different tick: no gravity, no arena, and
    // movement that happens in the animal's frame rather than the world's.
    if p.aboard() {
        match beast {
            Some(beast) => return step_rider(p, who, input, beast, scene, &out),
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

    let look = V3::from_turns(input.aim_turns());
    // A channel is the aiming, so the body keeps turning through it. Every
    // other action locks the facing -- see `Action::Channel`.
    if p.action.actionable() || p.action.stunned() || p.action.channelling().is_some() {
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

    // The jump button holds the takeoff window open, and a connected chain link
    // lets the next one start early. Both are Champion-only and both are no-ops
    // for everybody else; both go here, before the countdown, for the same
    // reason the Rush cancel does -- a recovery that has been cut short has to
    // reach the input below on the frame it was cut, not the frame after.
    arm_takeoff(p, input);
    bank_the_leap(p, pressed_space);
    chain_cancel(p, input);

    queue_the_shadow(p, input);

    // A channel resolves **instead of** the countdown, because it is the one
    // action whose next state depends on a button and on the scene rather than
    // on a number running down: held, it winds on and re-aims; released, it
    // throws the move it was winding. Falling through to the input below would
    // read the same press again and buy the same ability twice.
    if let Some((kind, held)) = p.action.channelling() {
        p.action = step_channel(p, who, kind, held, input, scene);
    } else {
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
                    && p.can_throw(SLOT_SPECIAL, &out)
                {
                    begin_move(p, who, SLOT_SPECIAL, input, scene, true)
                } else if pressed_mechanic {
                    // `E` is the class mechanic, and on half the roster that is
                    // an instant change of state with no frames to it -- throw
                    // the shield, Rush, raise a structure. Where the class puts
                    // an ability there instead -- the Blood mage and the Dual
                    // mage, whose mechanics have nothing to toggle, and the
                    // Reaver, whose mechanic went to the mouse because it is
                    // aimed -- it is thrown like any other move, with a startup
                    // you can be punished during and a cost you pay on the
                    // press. See `moves::on_e` and `keyed_move`.
                    match keyed_move(p).filter(|slot| p.can_throw(*slot, &out)) {
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
                else if let Some(kind) = clicked_move(p, input).filter(|k| p.can_throw(*k, &out))
                {
                    // A no-op for anybody who is not the Champion; the weapon in
                    // hand and the dash underneath it are that class's alone.
                    begin_champion(p, kind);
                    begin_move(p, who, kind, input, scene, true)
                } else if input.has(Input::SHIFT) && (ax != 0 || az != 0) {
                    // Shift plus a direction dodges. It used to be space plus a
                    // direction, which meant that pressing the jump button while
                    // moving -- which is most of the time -- did not jump. Space is
                    // now only ever a vertical takeoff.
                    //
                    // **And shift means nothing else.** It used to also be the
                    // attack modifier, which is why this branch once demanded
                    // that no click be held: the click was what disambiguated
                    // the two meanings. There is nothing to disambiguate any
                    // more -- a click is an attack whatever else is down -- so
                    // the only thing that guard still did was swallow the dodge
                    // on a frame where a click was held and threw nothing,
                    // which is a button press answered with silence.
                    let dir = move_dir(input.aim_turns(), ax, az);
                    // On the ground there is nothing to spend; in the air the
                    // commitment is spent once per airtime, because a second one
                    // would turn a jump into flight.
                    let may_commit = p.grounded || !p.air_dodged;
                    // The Reaver's forward dodge, thrown with the crosshair on
                    // her shadow, is the dash to it -- the same invulnerable
                    // commitment, pointed at the one place on the map she cares
                    // about. It is not an extra input: the class's mobility and
                    // the universal defensive option are deliberately the same
                    // button, which is what keeps her from being denied either.
                    //
                    // **In the air as much as on the ground.** Being off the floor
                    // is the commonest reason she is not standing where she wants
                    // to be, and the shadow is the answer to that question by
                    // construction -- a class whose mobility switched off the
                    // moment she jumped had the mobility in the wrong place. The
                    // airborne one is the airdodge, aimed, so it costs the
                    // airdodge; what it does not do is take the airdodge's shorter
                    // frames, because the dash has somewhere to *be* and a tail cut
                    // short would strand her halfway there.
                    if let Some(pool) = may_commit
                        .then(|| pool_under_the_crosshair(p, who, input, scene))
                        .flatten()
                    {
                        // The Blood mage's blink: a dodge thrown with the
                        // reticle on one of her pools puts her in it, and the
                        // pool is spent. The Reaver's dash pointed at the
                        // class's object instead of at a body -- and with
                        // nothing to cross, the crossing is instant. The
                        // airborne one spends the airdodge, like hers.
                        if !p.grounded {
                            p.air_dodged = true;
                        }
                        p.pos = scene.effects[pool].map_or(p.pos, |e| e.pos);
                        p.vel = V3::ZERO;
                        p.blinked = pool as u8;
                        Action::Dodge {
                            left: t::dodge_frames(),
                        }
                    } else if may_commit && shadow::dash_is_asked_for(p, who, input, az > 0, scene)
                    {
                        if !p.grounded {
                            p.air_dodged = true;
                        }
                        shadow::begin_dash(p);
                        // Whatever she was doing vertically is over: from here the
                        // dash drives all three axes along its own straight line.
                        p.vel.y = Fx::ZERO;
                        Action::Dodge {
                            left: t::dodge_frames(),
                        }
                    } else if p.grounded {
                        p.vel.x = dir.x.mul(t::dodge_speed());
                        p.vel.z = dir.z.mul(t::dodge_speed());
                        Action::Dodge {
                            left: t::dodge_frames(),
                        }
                    } else if !p.air_dodged {
                        // An airdodge, once per airtime. It commits you to a
                        // direction in the air, where you otherwise have almost no
                        // say.
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
    }

    rearm_multihit(p);

    // Horizontal movement.
    //
    // How much a move hinders you is proportional to how much it commits you.
    // A fast poke slows you; a committed move slows you to a crawl, slower than
    // guarding, which is the most your feet ever cost you.
    //
    // **Nothing stops you dead.** The committed moves used to, and it was the
    // same complaint the poke drew before it: a character who ignores the stick
    // reads as the game taking the controls away, and a heavy move is the worst
    // place to do that because it is where you are looking at your own feet. It
    // costs no spacing to give the frames back -- a crawl over a forty-frame
    // slam is under a metre -- and what commitment actually means survives
    // untouched, because it was never the standing still. It is that you cannot
    // jump, dodge, guard or throw anything else until the move is done, and that
    // is decided by `Action::actionable`, not by this number.
    //
    // Zero is still reachable from the palette and still roots, so this keeps
    // filtering it out rather than dividing the two cases twice.
    let attack_speed = p
        .action
        .attack_kind()
        .map(|kind| moves::get(p.class, kind).mobility)
        .filter(|m| *m > 0)
        .map(|m| t::move_speed().mul(Fx::ratio(m as i32, 100)));
    let steering = ax != 0 || az != 0;

    let dashing = shadow::dash_drive(p);
    let hauled = haul_drive(p);
    if let Some(drive) = dashing {
        // A dash has somewhere to be, so it holds its speed rather than
        // decaying like the dodge it rides on: a decaying shove covers whatever
        // distance the decay happens to be tuned for, and the shadow is at a
        // distance of its own choosing.
        //
        // **And it drives the vertical too**, which is what makes a shadow left
        // on a dais somewhere she can actually get to. See the resolve below:
        // for as long as this is driving, gravity and the arena are off and the
        // line is the whole of her motion.
        p.vel.x = drive.x;
        p.vel.y = drive.y;
        p.vel.z = drive.z;
    } else if let Some(drive) = hauled {
        // Her Grasp hauling *her*: a straight line to the creature's flank at
        // the reel speed every grab hauls at, with the arena and the creature
        // still solid under her -- unlike the Reaver's dash, this ends by
        // arriving against a body.
        p.vel = drive;
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
        //
        // Ahead of the step below, so a move that took you off the floor
        // partway through its own drive -- the hammer finisher, leaping with
        // whoever it just launched -- keeps the speed it had and steers with it,
        // rather than being driven along a line its feet have left.
        if steering {
            air_accelerate(p, move_dir(input.aim_turns(), ax, az), mob.air_speed);
        }
    } else if let Some(drive) = attack_step(p) {
        // **A move carrying the body.** The step is the move's own motion, down
        // the facing it locked when it started, and the stick is *added* to it
        // rather than replaced by it: a cut thrown while strafing comes out
        // diagonally instead of snapping to the front, which is the whole of
        // what "you can move while you swing it" is worth. Holding back blunts
        // the step, and that is the same answer -- you chose to give ground.
        //
        // Assigned rather than accumulated. The ramp below reads last frame's
        // speed back off the velocity, so a drive *added* to a velocity that
        // already contains it compounds into a rocket by the fourth frame.
        let steer = match (attack_speed, steering) {
            (Some(allowed), true) => move_dir(input.aim_turns(), ax, az).scale(dragged(p, allowed)),
            _ => V3::ZERO,
        };
        p.vel.x = drive.x.add(steer.x);
        p.vel.z = drive.z.add(steer.z);
    } else if p.action.actionable() && steering {
        let speed = if p.crouching {
            t::crouch_move_speed()
        } else {
            t::move_speed()
        };
        let dir = move_dir(input.aim_turns(), ax, az);
        p.vel.x = dir.x.mul(dragged(p, speed));
        p.vel.z = dir.z.mul(dragged(p, speed));
    } else if p.action.guarding() && steering {
        let dir = move_dir(input.aim_turns(), ax, az);
        p.vel.x = dir.x.mul(dragged(p, t::guard_move_speed()));
        p.vel.z = dir.z.mul(dragged(p, t::guard_move_speed()));
    } else if let (Some(allowed), true) = (attack_speed, steering) {
        // Hindered, and steering. **The hindered speed is arrived at, not
        // assigned.** Setting it outright would drop a running fighter from a
        // full walk to a crawl in one frame, which is the same lurch the dead
        // stop was -- the snap and the stopping were always two complaints
        // wearing one coat, and only one of them is design.
        //
        // Direction follows the stick from the first frame and only the
        // magnitude bleeds, which is what keeps it feeling responsive: you are
        // steering immediately, you are just not going anywhere fast yet.
        let dir = move_dir(input.aim_turns(), ax, az);
        let floor = dragged(p, allowed);
        let carried = V3::new(p.vel.x, Fx::ZERO, p.vel.z)
            .flat_len()
            .mul(t::hindrance_decay());
        let speed = if carried.raw() > floor.raw() {
            carried
        } else {
            floor
        };
        p.vel.x = dir.x.mul(speed);
        p.vel.z = dir.z.mul(speed);
    } else if p.action.attack_kind().is_some() {
        // Steering nothing -- or rooted, if somebody has tuned a move's
        // mobility to zero. The same ramp with the floor at zero: bleed the
        // speed off over a few frames rather than snapping to a halt.
        p.vel.x = p.vel.x.mul(t::hindrance_decay());
        p.vel.z = p.vel.z.mul(t::hindrance_decay());
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
    if input.has(Input::SPACE) && p.grounded && p.action.actionable() {
        p.vel.y = p.vel.y.add(t::jump_speed().mul(mob.jump));
        p.grounded = false;
        p.jump_hold = t::jump_hold_frames();
    }

    // **The dash jump.** Arriving from a dash leaves the Reaver sliding for a
    // few frames with the speed she crossed at still under her -- the carry --
    // and a jump pressed inside that window takes the slide up with her instead
    // of letting the floor have it. It cuts the dodge's tail short, which is the
    // other half of the reward: the frames she would have spent standing there
    // being punished are spent in the air going somewhere.
    //
    // A press rather than a hold, and the slide decays while the window is open,
    // so the tech has a gradient -- the earlier she finds it, the further she
    // goes. The ordinary jump above cannot fire here: the carry runs inside the
    // dodge, and a dodge is not actionable.
    if pressed_space && shadow::carrying_a_dash(p) {
        p.vel.y = p.vel.y.add(t::jump_speed().mul(mob.jump));
        p.grounded = false;
        p.jump_hold = t::jump_hold_frames();
        p.action = Action::Free;
        shadow::spend_carry(p);
    }

    // The second half of the uppercut. You are both off the ground and you
    // have hold of them; pressing jump again takes the pair of you higher,
    // once. This is the "we are settling this in the air" button, and it is the
    // only thing space does while airborne.
    if pressed_space && carrying && !p.leap_used && !p.grounded {
        p.vel.y = p.vel.y.add(t::uppercut_leap());
        p.leap_used = true;
    }

    if !p.grounded && dashing.is_none() {
        if plunging(p) {
            // **The descent.** Driven rather than fallen: gravity would make
            // the plunge take longer the lower she started, which is backwards
            // -- the thing that should scale with height is how long the
            // opponent gets to answer, not how fast she is moving when she
            // gets there. A fixed speed makes the telegraph exactly as long as
            // the distance she chose to open up.
            //
            // Vertical only. Air control is untouched above, so she still
            // steers where she is coming down, which is what makes throwing it
            // a read on where they will be rather than on where they are.
            p.vel.y = t::landfall_dive().neg();
        } else if p.air_stall > 0 {
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

    // **A dash is not resolved against the world.** Whether there was anything
    // in the way was decided on the frame it began, by `aim::clear_between`, and
    // the answer it gives is all-or-nothing: either no line reaches the shadow
    // and the press was an ordinary dodge, or one does and she takes it. Pushing
    // her out of the geometry halfway along is exactly the *partial* obstruction
    // the rule says there is no such thing as -- it is what used to catch her
    // feet on the side of a platform and end the dash at the foot of the thing
    // she was dashing on to.
    //
    // She arrives at the shadow's own spot, which is somewhere a body can stand,
    // and the frame after the dash ends resolves her there normally.
    if dashing.is_some() {
        return;
    }

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
            p.wound(cost);
            p.stun_total = t::slam_stagger();
            p.action = Action::Stagger {
                left: t::slam_stagger(),
            };
        }
        // Landfall arrives. The slam **is** the impact, so its active frames
        // begin on the frame her feet reach something rather than on a number
        // -- see [`plunging`]. Held at zero by `countdown` until this happens,
        // which is what lets the wind-up be as long as the height she chose.
        if let Action::Startup { kind, .. } = p.action {
            if p.class == Class::Elementalist && kind == moves::elementalist::LANDFALL {
                p.action = Action::Active {
                    kind,
                    left: moves::get(p.class, kind).active,
                };
            }
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
        Class::DualMage => dual_move(p, input),
        Class::BloodMage => blood_move(input),
        // The Reaver breaks it a third way: right click sends the shadow. It is
        // the one thing in her kit the **crosshair aims**, and the mouse is
        // where aiming lives -- so the mechanic is on the mouse and the swing
        // it displaced went to `E`, which is the one key that does not care
        // where anything is pointed. See `moves::on_e`.
        // The remembered press rather than the button itself: hers is the one
        // mechanic on a click, so it needs an edge of its own, and it is the
        // one input that outlives the frame it was pressed on. See
        // `crate::shadow` for both arguments.
        Class::ShadowReaver if shadow::order_queued(p) => Some(SLOT_MECHANIC),
        // The Elementalist breaks it a fourth way, and then a fifth. Right
        // click is Cataclysm, for the reason the Reaver's is the mechanic --
        // no shield, so the button is otherwise dead, and Cataclysm is aimed
        // along the crosshair like the auto. And both clicks mean something
        // different once her feet leave the floor, which is the *row* rather
        // than a fifth exception: see `elementalist_move`.
        Class::Elementalist => elementalist_move(p, input),
        // **Shift is only ever a dodge now**, so on the five classes that kept
        // the shared grammar a click is a poke and nothing modifies it. That
        // strands `SLOT_COMMITTED` on the three that have one and nowhere else
        // to put it -- the Bulwark's Slam, the Elementalist's Fissure, the
        // Blood mage's Rend -- and that is deliberate and temporary: finding
        // each of them a home is a separate job, one kit at a time, and the
        // Reaver's is already on `E`. See `docs/design/controls.md`.
        _ => input.has(Input::LEFT).then_some(SLOT_POKE),
    }
}

/// Which of the Elementalist's seven a click asks for.
///
/// **The row is where her feet are.** On the floor it is the shared grammar
/// plus Cataclysm on right click; off it, the same two buttons throw the two
/// things she can make out of air — see `moves::elementalist`, which is where
/// the grid is drawn.
///
/// Shift is spent on the ground and ignored in the air, and that is a
/// statement rather than an oversight: shift plus left click is Fissure, a
/// crack that races *along the ground*, and there is no airborne version of it
/// to reach for. A modifier that silently threw the grounded move from the air
/// would put a skillshot into the dirt below her; one that threw nothing would
/// eat the input. It throws the Air bolt, which is what left click means up
/// there.
///
/// `E` is not here. It is the only one of her three airborne moves that
/// displaces something rather than adding to it — on the floor that key is the
/// mechanic and has no frames at all — so it is answered by
/// [`keyed_move`] beside the rest of the mechanic grammar.
/// Which move the Blood mage's three clicks throw.
///
/// Left is the auto, right is the committed heavy and middle is the throw --
/// the same reading as the Elementalist's clicks, with the third button taking
/// the ranged move because the mouse means where. See `moves::blood` for why
/// the auto is the fifth row of the table rather than the first.
fn blood_move(input: Input) -> Option<u8> {
    use moves::blood as b;
    if input.has(Input::LEFT) {
        Some(b::SWEEP)
    } else if input.has(Input::RIGHT) {
        Some(b::HAEMORRHAGE)
    } else {
        input.has(Input::MIDDLE).then_some(b::BLOODLETTER)
    }
}

fn elementalist_move(p: &Player, input: Input) -> Option<u8> {
    use moves::elementalist as e;
    if !p.grounded {
        // Left before right, so both buttons at once is an attack rather than
        // silence -- the same tie-break the Dual mage and the Champion make.
        if input.has(Input::LEFT) {
            return Some(e::AIR_BOLT);
        }
        return input.has(Input::RIGHT).then_some(e::GALE);
    }
    if input.has(Input::RIGHT) {
        return Some(SLOT_HEAVY);
    }
    // Fissure is stranded by the retirement of shift as an attack modifier,
    // along with the other two committed moves in the roster. See
    // `clicked_move`.
    input.has(Input::LEFT).then_some(SLOT_POKE)
}

/// Which move the mechanic key throws, here and now.
///
/// [`moves::on_e`] answers for the *class* — which classes put an ability on
/// `E` at all, because their mechanic is not a thing you toggle. This answers
/// for the **situation**, which is one class's business: the Elementalist's
/// mechanic is an instant on the floor (raise a stone, no frames, nothing to
/// punish) and an ability off it (Landfall, a long plunge you can be knocked
/// out of). `None` means the key falls through to `mechanic_action`.
///
/// Split the same way `clicked_move` and `moves::binding` are: what the kit
/// declares lives in `moves`, and what your feet are doing is read here.
fn keyed_move(p: &Player) -> Option<u8> {
    if p.class == Class::Elementalist && !p.grounded {
        return Some(moves::elementalist::LANDFALL);
    }
    moves::on_e(p.class)
}

/// Is this fighter part-way through Landfall's plunge, with the wind-up spent
/// and the floor still to come?
///
/// **Landfall is the one move in the game whose startup ends on a place rather
/// than on a count.** The frames in its row are the part she is guaranteed to
/// owe — the hang at the top (`Move::air_stall`) and then the descent — and the
/// descent is over when her feet arrive, which from four metres up takes longer
/// than from one. So the startup runs down normally, and if it reaches zero
/// while she is still falling it *stays* at zero until the ground answers: see
/// [`countdown`], which holds it, and the landing in [`step_player`], which
/// spends it.
///
/// Everything about that is deliberately ordinary in one respect: it is a
/// startup, so being hit during it interrupts it exactly the way being hit
/// during any other wind-up does, and the slab of rock she was going to drive
/// up never appears. That is the whole of "a foe can stop her coming down",
/// and it needed no rule of its own — see
/// `tests/elementalist_air.rs::a_hit_on_the_way_down_ends_the_plunge`.
fn plunging(p: &Player) -> bool {
    let Action::Startup { kind, .. } = p.action else {
        return false;
    };
    !p.grounded
        && p.air_stall == 0
        && p.class == Class::Elementalist
        && kind == moves::elementalist::LANDFALL
}

/// Is this startup one that waits for the floor rather than for its own clock?
fn waits_for_the_floor(p: &Player, kind: u8) -> bool {
    !p.grounded && p.class == Class::Elementalist && kind == moves::elementalist::LANDFALL
}

/// Which of the Dual mage's six a click asks for.
///
/// Three buttons, and the middle one is where the whole class comes together:
///
/// ```text
///   left click    Dark auto    -- darker, and she is now dark
///   right click   Light auto   -- lighter, and she is now light
///   middle click  Lance        -- no side, so it pushes her further along
///                                 whichever way she is already going, and
///                                 which *form* comes out is the force she is
///                                 carrying
/// ```
///
/// **Middle click is the right home for Lance and shift never was.** The two
/// rules this class is made of used to fight there: shift + left click had a
/// side, so a committed cast pushed her dark whatever she was carrying, and the
/// light form of it had nowhere to live at all. Middle click has no side, so by
/// the class's own rule it pushes her further along her current path -- and
/// the form is then free to come from the force in her arms, which is the thing
/// the two autos exist to set. One input, two moves, and the arm you last
/// punched with is what decides which.
///
/// Left before right before middle, so a player mashing buttons gets an attack
/// rather than silence. The design has a use for both-click -- a finisher with
/// no side -- and does not have one yet.
fn dual_move(p: &Player, input: Input) -> Option<u8> {
    use moves::dual as d;
    if input.has(Input::LEFT) {
        return Some(d::DARK_AUTO);
    }
    if input.has(Input::RIGHT) {
        return Some(d::LIGHT_AUTO);
    }
    input.has(Input::MIDDLE).then(|| d::lance_for(carrying(p)))
}

/// Which of the two forces she is holding right now.
///
/// Dark for anybody who is not a Dual mage, which never comes up: the only
/// callers are her own moves. It is here rather than read off `Mechanic::Meter`
/// at each call site because "what is she made of this frame" is one question
/// and the number of places asking it grew the moment an ability had two forms.
pub fn carrying(p: &Player) -> Force {
    match p.mechanic {
        Mechanic::Meter { colour, .. } => colour,
        _ => Force::Dark,
    }
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

/// Which weapon a click is asking for. The column of the grid, and the one
/// thing about the Champion's controls that never changes meaning.
///
/// Left before middle before right, so a player mashing two buttons gets an
/// attack rather than silence.
const fn champion_weapon(input: Input) -> Option<u8> {
    use moves::champion as c;
    if input.has(Input::LEFT) {
        Some(c::SWORD)
    } else if input.has(Input::MIDDLE) {
        Some(c::HAMMER)
    } else if input.has(Input::RIGHT) {
        Some(c::SPEAR)
    } else {
        None
    }
}

/// Which of the Champion's nineteen moves this click asks for, if any.
///
/// `None` for every other class, which is what keeps the shared grammar in
/// `step_player` unchanged for the five of them. The order of the tests is the
/// design: **rushing beats taking off beats airborne beats standing.**
///
/// Rushing first because the Rush moves are the ones you spent a charge to
/// reach, and they should not be taken away by the fact that the uppercut has
/// already left the floor. Taking off next because the takeoff window runs a
/// few frames *into* the jump it belongs to -- see [`champion_takeoff_window`]
/// -- and for those frames "I pressed jump and a weapon" has to beat "my feet
/// are off the ground", which is the same sentence with the answer the player
/// did not mean.
///
/// What is left on the ground is the **chain**, and which of its three rows you
/// get is how many hits are already behind you rather than anything about the
/// button. See `moves::champion::link`.
fn champion_move(p: &Player, input: Input) -> Option<u8> {
    use moves::champion as c;
    if p.class != Class::Champion {
        return None;
    }
    let weapon = champion_weapon(input)?;
    if rushing(p).is_some() {
        // Right click during a Rush is two moves told apart by where you are
        // pointing, which is the honest separator: a pole vault *is* a spear
        // put into the ground, and levelling it at somebody is a stab.
        if weapon == c::SPEAR && input.pitch_turns().raw() <= t::vault_pitch().neg().raw() {
            return Some(c::POLE_VAULT);
        }
        return Some(c::RUSHING + weapon);
    }
    // No takeoffs on the creature's back: up there the jump button is how you
    // *leave*, and a move that spent it on an attack would take that away. The
    // window is armed on the floor and can still be open on the frame you land
    // aboard, which is the only way this is ever reached.
    if !p.aboard() && champion_takeoff_window(p) > 0 {
        return Some(c::TAKEOFF + weapon);
    }
    if !p.grounded {
        return Some(c::IN_THE_AIR + weapon);
    }
    Some(c::link(champion_chain(p).0, weapon))
}

/// How deep the grounded chain is, and how long it has left to live.
///
/// `(0, 0)` for anybody who is not a Champion, which makes every caller below
/// safe to write without a class test of its own.
fn champion_chain(p: &Player) -> (u8, u16) {
    match p.mechanic {
        Mechanic::Forms {
            chain, chain_left, ..
        } => (chain, chain_left),
        _ => (0, 0),
    }
}

/// Frames left in which a weapon click leaves the floor rather than swinging on
/// it.
fn champion_takeoff_window(p: &Player) -> u16 {
    match p.mechanic {
        Mechanic::Forms { takeoff, .. } => takeoff,
        _ => 0,
    }
}

/// Hold the takeoff window open while the jump button is down and the feet are
/// on something.
///
/// **The window exists because two buttons are never pressed on the same
/// frame.** "Attack as you jump" is one intention and two inputs, and a rule
/// that needed them on the same tick would be a rule nobody could hit -- so the
/// jump arms a few frames during which a weapon click is a takeoff, and the
/// click spends them.
///
/// Armed off the button being *down* rather than off its press edge, and
/// grounded rather than actionable, which between them cover the three orders
/// the player can press in. Jump then click is the window. Both at once is the
/// same frame, and the window is armed before the click is read. Click first
/// throws the grounded move, because it already has -- and the window is still
/// armed underneath it, so a weapon pressed again as the recovery ends takes
/// off out of it.
///
/// It can only ever be open around a real jump: standing on the floor with the
/// jump button down *is* jumping, on the frame you are free to.
fn arm_takeoff(p: &mut Player, input: Input) {
    if p.class != Class::Champion || !p.grounded || !input.has(Input::SPACE) {
        return;
    }
    if let Mechanic::Forms { takeoff, .. } = &mut p.mechanic {
        *takeoff = t::takeoff_window();
    }
}

/// Cut a **connected** chain link's recovery short, so the next hit can start.
///
/// The third cancel in the game and the narrowest, and the two rules around it
/// are what keep it from being a licence to swing forever.
///
/// **It is a hit confirm.** A link that was blocked, parried or thrown at
/// nothing pays its recovery in full, so every number the frame table prints
/// about a Champion move is true against a defender who did something about it.
/// `every_attack_is_punishable_on_block` stays a fact rather than an
/// approximation, and blocking one hit of a three-hit string is worth doing
/// because it ends the string.
///
/// **Swapping weapons cancels earlier than repeating one.** The class's whole
/// fantasy is one haft with three heads, and the honest reason the swap is
/// quicker is that the weapon is *re-formed out of the follow-through* while
/// swinging the same one twice has to re-chamber it. It is a few frames rather
/// than a wall: repeating a weapon stays completely viable, which is the
/// "strong when played linearly" half of the design, and mixing is the half
/// that pays a little better. The two knobs are the whole of that decision and
/// setting them equal turns it off -- see `tuning::chain_cancel_swapped`.
fn chain_cancel(p: &mut Player, input: Input) {
    use moves::champion as c;
    if p.class != Class::Champion {
        return;
    }
    let Action::Recovery { kind, left } = p.action else {
        return;
    };
    if c::link_of(kind).is_none() {
        return;
    }
    let Mechanic::Forms {
        chain,
        chain_left,
        chain_hit: true,
        ..
    } = p.mechanic
    else {
        return;
    };
    // Nothing to cancel into: the chain has run out of links, or out of time.
    if chain >= c::DEPTH || chain_left == 0 {
        return;
    }
    let Some(next) = champion_weapon(input) else {
        return;
    };
    let m = moves::get(p.class, kind);
    let share = if next == c::weapon(kind) {
        t::chain_cancel_repeated()
    } else {
        t::chain_cancel_swapped()
    };
    let owed = (m.recovery as u32 * share as u32 / 100) as u16;
    if m.recovery.saturating_sub(left) < owed {
        return;
    }
    // Free rather than straight into the next link: the ordinary input path is
    // three lines below and already knows how to throw one, including the
    // lockout, the aim and the animation. A second way to start a move is a
    // second way for one of those to be forgotten.
    p.action = Action::Free;
}

/// Record that a chain link landed, so its recovery may be cut short.
///
/// Called from the one place a hit is resolved, and a no-op for everything that
/// is not a Champion swinging one of the nine. Blocked and parried hits do not
/// count: see [`chain_cancel`].
fn confirm_chain(p: &mut Player, blocked: bool) {
    if p.class != Class::Champion || blocked {
        return;
    }
    let Some(kind) = p.action.attack_kind() else {
        return;
    };
    if moves::champion::link_of(kind).is_none() {
        return;
    }
    if let Mechanic::Forms { chain_hit, .. } = &mut p.mechanic {
        *chain_hit = true;
    }
}

/// Where the chain is after this move: one hit deeper, or back to nothing.
///
/// **Only the nine ground moves are in it.** An aerial, a Rush move or a
/// takeoff ends the chain outright rather than being ignored by it, and that is
/// the design rather than an implementation shortcut: leaving the floor is a
/// different situation, and a string you could park in the air and come back to
/// would make the grace window meaningless.
fn advance_chain(p: &mut Player, kind: u8) {
    let stage = match moves::champion::link_of(kind) {
        Some(link) => link + 1,
        None => 0,
    };
    if let Mechanic::Forms {
        chain,
        chain_left,
        chain_hit,
        ..
    } = &mut p.mechanic
    {
        *chain = stage;
        *chain_left = if stage > 0 { t::chain_grace() } else { 0 };
        *chain_hit = false;
    }
}

/// What throwing one of the Champion's moves does to the weapon in hand, to the
/// chain it is part of, and to the dash underneath it.
fn begin_champion(p: &mut Player, kind: u8) {
    use moves::champion as c;
    if p.class != Class::Champion {
        return;
    }
    let Mechanic::Forms { rush, rush_vel, .. } = p.mechanic else {
        return;
    };
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
    // The chain moves on, or ends, depending on what this move was.
    advance_chain(p, kind);
    if let Mechanic::Forms {
        form,
        rush: r,
        rush_vel: v,
        takeoff,
        ..
    } = &mut p.mechanic
    {
        // The weapon is whichever button was pressed. Nothing else sets it:
        // there is no mode to be in any more.
        *form = Form::of_move(kind);
        *r = rush;
        *v = rush_vel;
        // A takeoff spends the window it came out of, so one jump buys one of
        // them. Everything else leaves it alone: a grounded swing thrown with
        // the jump button already down is a move you can still take off out of
        // when its recovery ends.
        if c::is_takeoff(kind) {
            *takeoff = 0;
        }
    }
}

/// The horizontal shove the pole drive gives, on the frame the spear reaches
/// the floor.
///
/// **The move is a jump with a weapon in it**, so the reward is a direction
/// rather than damage: the butt of the spear cracks the ground, the fighter
/// goes up higher than a jump reaches, and the run they were holding comes with
/// them. Read from the live input on the frame it happens, for the same reason
/// the aerial fan's shove is -- the point is that you choose where to go as the
/// spear lands, not when you pressed the button.
///
/// Holding nothing still gets you something: forward, along the facing. A
/// reward you can fail to collect by not touching a key reads as broken rather
/// than as demanding.
fn pole_drive_boost(p: &mut Player, input: Input) {
    let (ax, az) = input.move_axis();
    let dir = if ax == 0 && az == 0 {
        V3::new(p.facing.x, Fx::ZERO, p.facing.z)
    } else {
        move_dir(input.aim_turns(), ax, az)
    };
    let boost = t::pole_drive_boost();
    p.vel.x = p.vel.x.add(dir.x.mul(boost));
    p.vel.z = p.vel.z.add(dir.z.mul(boost));
    clamp_air_speed(p);
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
        move_dir(input.aim_turns(), ax, az)
    };
    let boost = t::spear_fan_boost();
    p.vel.x = p.vel.x.add(dir.x.mul(boost));
    p.vel.z = p.vel.z.add(dir.z.mul(boost));
    clamp_air_speed(p);
}

/// The drive a **stepping** move is putting under the body this frame.
///
/// `None` for almost everything. A move with no [`step`] does not carry you, and
/// neither do the frames outside its window: that window ends on the last active
/// frame and runs [`tuning::step_lead`] frames back into the startup, so the feet
/// always arrive with the weapon rather than some frames before or after it.
///
/// **Constant speed across the window**, which is what makes one number enough.
/// Every stepping move in the game drives for the same shape of window, so a
/// long distance over it is a dash and a short one is a step -- the character of
/// the motion falls out of the distance instead of needing a second knob for it.
/// What happens afterwards is not this function's business: the hindrance ramp
/// the recovery falls into bleeds the speed off over a few frames, which is the
/// follow-through.
///
/// [`step`]: moves::Move::step
/// [`tuning::step_lead`]: crate::tuning::step_lead
fn attack_step(p: &Player) -> Option<V3> {
    let kind = p.action.attack_kind()?;
    let m = moves::get(p.class, kind);
    if m.step.raw() == 0 {
        return None;
    }
    let lead = t::step_lead();
    // **The step is finished by the time the weapon lands.** `left` is the
    // startup frames still to come after this one, so the frame the hitbox
    // appears on is `left + 1` away and the run-up is the last `lead` of them --
    // and then the first active frame, which is the one the step arrives on.
    //
    // It used to run for the whole active window, which is right for a cut that
    // sweeps and wrong for anything that plants: the hammer's finisher put its
    // head through the floor on the first active frame and then carried the body
    // most of another metre past it, feet skating, for the five frames the volume
    // was still out. Ending it on contact is the rule that is true of both --
    // a swing still travels afterwards, because the momentum is still there and
    // the hindrance ramp spends it, which is the follow-through rather than the
    // step.
    let driving = match p.action {
        Action::Startup { left, .. } => left < lead,
        Action::Active { left, .. } => left == m.active,
        _ => false,
    };
    if !driving {
        return None;
    }
    let frames = Fx::from_int((lead + 1) as i32);
    let speed = m.step.div(DT.mul(frames));
    Some(V3::new(
        p.facing.x.mul(speed),
        Fx::ZERO,
        p.facing.z.mul(speed),
    ))
}

/// Bank a jump pressed **during** the hammer's finisher, so it can be spent on
/// the frame the finisher lands.
///
/// The one move in the game where the jump button is read inside another move's
/// frames, and the reason is that the finisher throws somebody into the air and
/// the interesting question is whether you are going with them. Declining is a
/// knock-up and a reset; accepting is a knock-up worth more, and the pair of you
/// leaving the floor on the same frame.
///
/// A **press**, not a hold, and it has to happen while the move is still
/// deciding -- during the startup or the active frames. Holding the button from
/// before the swing would have thrown the takeoff instead (`arm_takeoff`), and a
/// press during the recovery is a decision taken after the answer was known.
///
/// The bank clears itself the moment the action is anything else, so it cannot
/// be carried from one finisher into the next.
fn bank_the_leap(p: &mut Player, pressed_space: bool) {
    if p.class != Class::Champion {
        return;
    }
    let deciding = matches!(
        p.action,
        Action::Startup { kind, .. } | Action::Active { kind, .. }
            if kind == moves::champion::EARTHBREAKER
    );
    if let Mechanic::Forms { leap_banked, .. } = &mut p.mechanic {
        *leap_banked = deciding && (*leap_banked || pressed_space);
    }
}

/// Is this fighter going up with whoever this move is about to launch?
///
/// Read by two places that must agree: the knock-up the defender gets, and the
/// lift the attacker takes. One function so a retune cannot make the launch
/// bigger without the Champion following it.
fn going_up_with_them(p: &Player, kind: u8) -> bool {
    p.class == Class::Champion
        && kind == moves::champion::EARTHBREAKER
        && matches!(
            p.mechanic,
            Mechanic::Forms {
                leap_banked: true,
                ..
            }
        )
}

/// Spend the banked jump: leave the floor on the frame the finisher connects.
///
/// **On contact rather than on the press**, which is the same rule the aerial
/// fan's shove follows and for the same reason -- a whiffed finisher that still
/// threw you into the air would be a free escape bolted to the most punishable
/// move in the kit. You go up because it landed.
fn champion_leap_with_them(p: &mut Player, blocked: bool) {
    let Some(kind) = p.action.attack_kind() else {
        return;
    };
    if blocked || !going_up_with_them(p, kind) {
        return;
    }
    p.vel.y = t::hammer_leap();
    p.grounded = false;
    // Spent. The hit is over and the bank is not a thing you keep.
    if let Mechanic::Forms { leap_banked, .. } = &mut p.mechanic {
        *leap_banked = false;
    }
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
    let Mechanic::Forms { recharge: 0, .. } = p.mechanic else {
        return false;
    };
    let (ax, az) = input.move_axis();
    let dir = if ax == 0 && az == 0 {
        p.facing
    } else {
        move_dir(input.aim_turns(), ax, az)
    };
    if let Mechanic::Forms {
        rush,
        recharge,
        rush_vel,
        ..
    } = &mut p.mechanic
    {
        *rush = t::rush_frames();
        // The charge is gone until the dash has finished *and* the recharge has
        // run, so the two never overlap and "rush ready" on the HUD means it.
        *recharge = t::rush_frames() + t::rush_recharge();
        *rush_vel = V3::new(
            dir.x.mul(t::rush_speed()),
            Fx::ZERO,
            dir.z.mul(t::rush_speed()),
        );
    }
    true
}

/// Read the Reaver's right click, and let it cut a recovery short.
///
/// **The second cancel in the game, and the same shape as the first.** The
/// Champion's Rush ends a recovery it is spent from, which is that class's
/// answer to having committed to the wrong move; this is the Reaver's, and the
/// case for it is the class rather than the convenience. Everything she has is
/// a function of the line between her two bodies, and that line is her way out
/// of both the ordinary limits on where she can stand and the ordinary limits
/// on what she can reach. A mechanic that can only be moved on the frames she
/// happens to be idle is a mechanic the rest of her kit can lock her out of.
///
/// **Recovery only, and that is not a hedge.** Cancelling a startup would let
/// her take a committed swing back after throwing it, which is whiff punishment
/// deleted; cancelling active frames would let her un-throw one that is already
/// out. Recovery is the part that is over -- the animation finishing, not the
/// decision -- and it is what "lag" means when a player says a move has some.
///
/// **It buys her tempo, not safety.** Send shadow costs twenty-five frames and
/// the longest recovery it can cut short is Executioner's twenty-six, so the
/// exchange nets her one frame: she is busy for as long either way, and what
/// changes is that the frames do something. That is why it needs no charge
/// behind it the way Rush does, and
/// `reaver::cutting_a_recovery_short_leaves_every_move_punishable` pins it as a
/// relationship rather than a hope.
///
/// Both input paths call it -- on the ground and on the creature's back -- so
/// the press edge is read exactly once a frame either way.
fn queue_the_shadow(p: &mut Player, input: Input) {
    shadow::queue_order(p, input);
    if shadow::order_queued(p) && matches!(p.action, Action::Recovery { .. }) {
        p.action = Action::Free;
    }
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
    // **Before anything else**, and before `throw_move` steers the bar: what
    // this cast is worth is where she is standing right now. See
    // [`Player::thrown_at`].
    p.thrown_at = live_depth(p);
    // Health is spent on the press, never on the hit. Missing is the
    // punishment, which is the whole of the Blood mage's economy -- see
    // `docs/design/kits/blood-mage.md`. Paid here even for a channelled move,
    // because the press is still the press: there is no way to cancel out of a
    // wind-up, so an ability you started is an ability you bought.
    p.spend_health(moves::get(p.class, kind).cost);
    // A channelled move does not start here. Pressing the button opens the
    // wind-up instead, and the move begins when the button comes back up --
    // see `step_channel`.
    if moves::get(p.class, kind).channels() {
        aim_channel(p, who, kind, 0, input, scene);
        return Action::Channel { kind, held: 0 };
    }
    lock_aim(p, who, kind, input, scene);
    throw_move(p, kind, input, aerial)
}

/// Everything a move does on the frame it actually comes out, after the aim has
/// been settled.
///
/// Its own function because there are two frames that can be that frame: the
/// press, for almost everything, and the *release*, for a channelled move. Left
/// inline in `begin_move`, a channel would quietly skip the class mechanics
/// below -- which is a no-op for the one class that channels today and a bug
/// waiting for the second one.
fn throw_move(p: &mut Player, kind: u8, input: Input, aerial: bool) -> Action {
    // The lockout starts here, on the frame the move comes out, rather than on
    // the press that asked for it. For a channel those are different frames and
    // the wind-up has already been paid for in frames of its own; charging from
    // the press would bill the hold twice.
    p.lock_repeat(kind);
    // The second body throws the same thing a few frames later. A no-op for
    // every class but one, and for the two of the Reaver's four moves that are
    // already the shadow's own -- see `shadow::begin_echo`.
    shadow::begin_echo(p, kind);
    // And the press that asked for the shadow is spent here, on the frame the
    // move it asked for actually starts. A no-op for everything else.
    shadow::spend_order(p, kind);
    // Throwing anything at all is committing to a side, for the one class
    // where that is the mechanic. **On the press, including the autos** -- see
    // `steer_meter` for why that stopped being on contact.
    steer_meter(p, kind);
    if aerial {
        arm_aerial(p, kind, input);
    }
    Action::Startup {
        kind,
        left: moves::get(p.class, kind).startup,
    }
}

/// One frame of a channel: wind on and re-aim, or release and throw.
///
/// The aim is recomputed **every frame** rather than locked, which is the
/// whole reason a channel exists here: the marker in front of the caster is
/// `aim_path.to`, so what the player is looking at is what the arms will
/// converge on, by construction rather than by two pieces of arithmetic
/// agreeing with each other.
fn step_channel(
    p: &mut Player,
    who: usize,
    kind: u8,
    held: u16,
    input: Input,
    scene: &Scene,
) -> Action {
    let cap = moves::get(p.class, kind).channel;
    // Held, and there is still room: wind on. At the cap it releases itself,
    // so a player holding the button through a fight is not quietly storing an
    // ability they have already paid for.
    if input.has(channel_button(kind)) && held < cap {
        let held = held + 1;
        aim_channel(p, who, kind, held, input, scene);
        return Action::Channel { kind, held };
    }
    // Released. The aim locks now, on the frame the wind-up becomes a move,
    // which is where every other move locks it too.
    aim_channel(p, who, kind, held, input, scene);
    // Read back off the path rather than recomputed from the hold. The two are
    // the same number -- `aim_channel` has just put the marker there -- and
    // taking it from the path is what makes the arms land on the marker rather
    // than on a second calculation that agrees with it today.
    p.channelled = p.aim_path.length();
    // `aerial` is always true here and always harmless: `arm_aerial` returns on
    // the spot for anybody whose feet are on something, and aboard the creature
    // they always are.
    throw_move(p, kind, input, true)
}

/// Which button holds a channel open. The slot's own, since a channel is the
/// front of a move rather than a thing of its own.
const fn channel_button(kind: u8) -> u16 {
    match kind {
        // Shift picks *which* click; the click is what holds the wind-up open,
        // so letting go of shift halfway through does not throw the move.
        SLOT_POKE | SLOT_COMMITTED => Input::LEFT,
        SLOT_SPECIAL => Input::SPECIAL,
        _ => Input::MECHANIC,
    }
}

/// Re-solve a channelled move's aim: the crosshair picks the line, the hold
/// picks how far along it.
///
/// **Two questions, kept apart, and that is the whole of what makes a wind-up
/// readable.** The line is solved at the move's full reach every frame, so it
/// does not move while the mouse does not. Solving it at the wound-up range
/// instead -- which is what this did first -- means the raycast's own answer
/// changes as the range grows: the far end walks off one surface and onto
/// another, and a player holding the mouse perfectly still watched the marker
/// jump between the floor, a wall and the edge of the range while they were
/// choosing a depth.
///
/// **The distance is the hold and nothing but the hold.** It does not ask what
/// the crosshair stopped on. Aiming at a wall six metres away and winding to
/// full range is a ten-metre Grasp that goes through the wall -- a wall is a
/// thing to punch an ability through, not a shorter version of the ability. The
/// solved line is only ever read for its *direction*.
///
/// Nothing is decided here: `aim_at` is the same call the finished move uses,
/// so there is no second answer to where the ability goes and the marker cannot
/// drift from it.
fn aim_channel(p: &mut Player, who: usize, kind: u8, held: u16, input: Input, scene: &Scene) {
    let m = moves::get(p.class, kind);
    aim_at(p, who, kind, m.reach, input, scene);
    p.aim_path.to = p.aim_path.at(m.reach_after(held));
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
    aim_at(p, who, kind, moves::get(p.class, kind).reach, input, scene);
}

/// The three lines themselves, at whatever reach the caller has decided on.
/// Only a channel has anything to decide: every other move is aimed at the
/// reach in its own row.
fn aim_at(p: &mut Player, who: usize, kind: u8, reach: Fx, input: Input, scene: &Scene) {
    let m = moves::get(p.class, kind);
    p.aim_path = match m.aim() {
        aim::Kind::Grounded => aim::grounded_path(who, input, reach, scene),
        aim::Kind::Skillshot => aim::skillshot_path(who, input, reach, scene),
        // Not aimed at anything -- a body moving. What it commits to is the
        // **plane** it swings in: the yaw is the facing, which is locked
        // already, and the pitch is the rest of the same look, dead-zoned so
        // that looking slightly down at somebody does not tilt the swing into
        // the floor. The Champion's weapons read the plane; a disc-shaped
        // swing only reads the direction. The dead zone is a standing rule:
        // off the ground you are above what you are hitting, and the swing
        // follows the camera the whole way.
        aim::Kind::Swing => aim::swing_path(p.pos, p.facing, input, p.grounded, reach, m.hand),
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
    let fast = kind == SLOT_POKE
        || (p.class == Class::Champion && kind == moves::champion::AIR_SWORD)
        // The Elementalist's air row has its own cheap button, and it is the
        // same button: left click. The Air bolt is the thing she throws
        // constantly up there, so it is the one that gets to be part of moving
        // rather than a pause in it.
        || (p.class == Class::Elementalist && kind == moves::elementalist::AIR_BOLT);
    if !fast || (ax == 0 && az == 0) {
        return;
    }
    let dir = move_dir(input.aim_turns(), ax, az);
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

/// The mechanic key's action, per class -- `E`, and middle click once upon a
/// time, which is where this used to say the action lived.
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
        // is the resource -- and `stones::raise` is where that rule lives,
        // because Landfall raises one too and the two have to spend the cap
        // the same way.
        //
        // **On the ground only.** Off it, `E` is Landfall instead: a move with
        // frames somebody can punish rather than an instant, which is why it
        // is picked in `keyed_move` before this is ever reached.
        Mechanic::Structures(_) => {
            stones::raise(p, class::Structure::raised(placed(t::raise_reach())));
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
        //
        // The chain's own clock runs here too, and it runs in three modes
        // rather than one, because a chain is a rhythm rather than a timer:
        //
        //   * **parked** while a link of it is actually being thrown, so a
        //     fifty-frame finisher cannot time its own chain out from under
        //     itself. The same idiom the repeat lockout uses for an ability
        //     that is still out in the world -- re-armed to full every frame it
        //     is held, so it is at full on the frame it stops being held;
        //   * **counting down** while the fighter is standing on the floor with
        //     nothing to do, which is the window the next hit has to arrive in;
        //   * **gone** for anything else. Being hit, blocking, dodging, jumping
        //     and rushing all end a string, and that is most of what makes
        //     committing to one a decision: three hits is a plan, and the
        //     opponent gets to have an opinion about it.
        Mechanic::Forms { .. } => {
            let live = p
                .action
                .attack_kind()
                .is_some_and(|kind| moves::champion::link_of(kind).is_some());
            let standing = p.action.actionable() && p.grounded;
            if let Mechanic::Forms {
                rush,
                recharge,
                chain,
                chain_left,
                chain_hit,
                takeoff,
                ..
            } = &mut p.mechanic
            {
                *rush = rush.saturating_sub(1);
                *recharge = recharge.saturating_sub(1);
                *takeoff = takeoff.saturating_sub(1);
                if live {
                    *chain_left = t::chain_grace();
                } else if standing && *chain < moves::champion::DEPTH {
                    *chain_left = chain_left.saturating_sub(1);
                } else {
                    // Either the fighter is doing something a string does not
                    // survive, or the string has spent its third hit. There is
                    // no fourth: a chain that has finished is over on the frame
                    // its finisher stops running, and the next press opens a
                    // new one.
                    *chain_left = 0;
                }
                if *chain_left == 0 {
                    *chain = 0;
                    *chain_hit = false;
                }
            }
        }

        // The second body: where it is, what it is copying, and the leash.
        // See `crate::shadow`.
        Mechanic::Shadow(_) => shadow::step(p),

        // Two different things happen at depth, and they are different on
        // purpose. Inside the bar, past the deep threshold, the forces **burn**
        // you and you stop it by coming back inside the line -- relief from
        // stopping rather than from a reward, which is the containment story.
        // Driven all the way to an end, that becomes **ascension**, which you
        // cannot stop: it runs its clock, it costs far more, and it puts you
        // back at the centre staggered. See `docs/design/dual-mage.md`.
        Mechanic::Meter {
            value,
            colour,
            ascending,
        } => {
            if ascending > 0 {
                p.health = (p.health - t::ascension_drain().max(1)).max(1);
                let left = ascending - 1;
                p.mechanic = Mechanic::Meter {
                    // Pinned while it runs: the meter is not the operative
                    // resource during ascension, so nothing steers it.
                    value,
                    colour,
                    ascending: left,
                };
                if left == 0 {
                    // Spat back out at the centre. The stun is what makes
                    // reaching the end a decision rather than a free ride --
                    // flat for now, where the design wants it graduated by how
                    // much damage was dealt.
                    p.mechanic = Mechanic::Meter {
                        value: 0,
                        colour,
                        ascending: 0,
                    };
                    p.stun_total = t::ascension_stun();
                    p.action = Action::Stagger {
                        left: t::ascension_stun(),
                    };
                }
                return;
            }
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

// ---------------------------------------------------------------------------
// The depth curve
// ---------------------------------------------------------------------------
//
// The Dual mage's founding idea, and until now the largest hole in her:
// **power scales continuously with distance from the centre of the bar.** Not
// thresholds, not two versions of a move with a snap between them -- a straight
// line from `tuning::depth_floor` at zero to `tuning::depth_ceiling` at either
// end, symmetric, with every point on it reachable.
//
// It is one function rather than a rule each ability remembers, which is the
// whole reason it is worth building at all: "ride as close to the edge as you
// can" has to be true of everything she throws, or it is a number going up on
// the HUD.
//
// **What it scales, and what it deliberately does not.** Depth moves *force*:
// damage, knockback and pull, launch, what a thing she leaves behind drains and
// how long it lasts. It never touches a move's own startup, active or recovery
// frames, and never its hitstun or blockstun. Frame data is the contract
// between two players -- it is what the frame table prints and what a punish is
// read off -- and a class whose moves changed speed with a bar nobody else can
// see would be unlearnable from the other side. So the bar changes how much it
// hurts and how big it is, never how fast it comes out.

/// What this fighter's bar is worth, as a multiplier on everything they throw.
///
/// [`Fx::ONE`] for every class but one, so it is safe to ask of anybody: five
/// of the six have no meter, and the arithmetic is a multiply by one.
///
/// Read from her position **now** rather than from the position she was in when
/// a move started, and that is deliberate for the things she leaves behind: a
/// tether drains harder if she keeps riding outward while it holds, which is
/// the class's own argument made by an ability that outlives its cast.
pub fn depth(p: &Player) -> Fx {
    // Mid-move, it is what the bar said when she committed. Throwing anything
    // moves the bar on the press and the finisher moves it a long way, so a
    // cast read live would be worth its own push -- see [`Player::thrown_at`].
    match p.action.attack_kind() {
        Some(_) => p.thrown_at,
        None => live_depth(p),
    }
}

/// The curve read off the bar as it is this instant.
///
/// The shape of the whole thing: a straight line from `depth_floor` at the
/// centre to `depth_ceiling` at either end, symmetric, with every point on it
/// reachable and nothing anywhere that snaps.
fn live_depth(p: &Player) -> Fx {
    let Mechanic::Meter { value, .. } = p.mechanic else {
        return Fx::ONE;
    };
    let max = t::meter_max().max(1);
    let out = Fx::ratio(value.abs().min(max), max);
    crate::math::lerp(t::depth_floor(), t::depth_ceiling(), out)
}

/// The same curve as it applies to the **size** of what a move puts in the
/// world, which is a separate question with a separate knob.
///
/// More damage is worse to be hit by; more radius is harder not to be hit by.
/// `tuning::depth_size` is how much of the one the other takes, so a tuning
/// session can have a deep cast hit twice as hard and be exactly as big.
///
/// **The two autos never grow.** Their reach is pinned to the punch that throws
/// them -- `view/tests/kinematics.rs` checks that the blade's near edge starts
/// where the fist stops -- and they are the one move in the kit thrown every
/// second, so a volume that drifted away from the animation would make the
/// steering wheel unreadable. What depth does to an auto is what it *does*: how
/// much it hurts, and how hard it pulls or shoves. See `moves::dual::is_an_auto`.
pub fn depth_of_size(p: &Player, kind: u8) -> Fx {
    // **Radius, never reach**, which is the line the caller has to hold: how
    // big the thing that arrives is, not how far away you can put it. Spacing
    // is what every judgement in a fight is made from, and a class whose range
    // changed continuously with a bar the other player cannot see would be
    // unlearnable from either side of it -- the same argument that keeps depth
    // off the frame data. A deep Judgement is a far bigger Judgement thrown
    // exactly as far as a feeble one.
    if p.class != Class::DualMage || moves::dual::is_an_auto(kind) {
        return Fx::ONE;
    }
    crate::math::lerp(Fx::ONE, depth(p), t::depth_size())
}

/// Attacking steers the Dual mage's meter: dark darker, light lighter, an auto
/// a little and a cast more. Nothing else moves it, so every step is a
/// consequence of a decision the player made.
///
/// **On the press, every time, including the autos.** They used to steer on
/// *contact* -- the design's own rule, and the reason for it is good: landing a
/// far-side auto is the fast way back toward centre, which is what forces this
/// class into melee exactly when it is most fragile. It was still wrong, and
/// obviously so the moment anybody played it: with nothing in reach, **no
/// button on the class moved the bar at all.** The autos hit nothing, the casts
/// took their direction from an auto that had never landed, and the whole
/// mechanic sat at zero. A resource you cannot move without a target is a
/// resource you cannot learn, cannot tune, and cannot see working.
///
/// If the melee pull is wanted back it should return as a **bonus for
/// landing** rather than as the only way to move -- see the feel log entry for
/// 2026-09-13.
///
/// **Which way comes from the force she is carrying, not from the buttons held
/// down.** Only the autos have a side of their own, and throwing one is what
/// sets which force she carries; everything else is made of that force. Reading
/// the input bits instead meant asking which of `shift` and `left click` won,
/// and a move already knows what it is made of.
fn steer_meter(p: &mut Player, kind: u8) {
    let Mechanic::Meter {
        value,
        colour,
        ascending,
    } = p.mechanic
    else {
        return;
    };
    // Nothing steers during ascension. The bar is not the resource then; the
    // clock is.
    if ascending > 0 {
        return;
    }
    // Three tiers, and the order of the arms is the order of the commitment.
    // An auto is the unit the bar is measured in, and it also *sets* which
    // force she is carrying. A cast moves her further, in whichever direction
    // the last auto left her facing. And the **finisher** very nearly throws
    // her over the edge, which is the tier `docs/design/dual-mage.md` has been
    // asking for since the bar was built: what makes Judgement the payoff is no
    // longer that it is gated, it is that casting it deep is a real question
    // about whether you survive the cast.
    let (push, colour) = match moves::dual::force(kind) {
        Some(thrown) => (t::meter_auto_push(), thrown),
        None if moves::dual::is_the_finisher(kind) => (t::meter_finisher_push(), colour),
        None => (t::meter_cast_push(), colour),
    };
    let value = (value + push * colour.along()).clamp(-t::meter_max(), t::meter_max());
    // Driven all the way to an end, and it takes her. There is no input for it
    // and there never was -- you got there one cast at a time.
    let ascending = if value.abs() >= t::meter_max() {
        t::ascension_frames()
    } else {
        0
    };
    p.mechanic = Mechanic::Meter {
        value,
        colour,
        ascending,
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
            chain,
            chain_left,
            chain_hit,
            takeoff,
            leap_banked,
        } => {
            h.write_u32(3);
            h.write_u32(*form as u32);
            h.write_u32(*rush as u32);
            h.write_u32(*recharge as u32);
            hash_v3(h, rush_vel);
            h.write_u32(*chain as u32);
            h.write_u32(*chain_left as u32);
            h.write_u32(*chain_hit as u32);
            h.write_u32(*takeoff as u32);
            h.write_u32(*leap_banked as u32);
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
            h.write_u32(shadow.carry as u32);
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
                        h.write_u32(s.rise as u32);
                        hash_v3(h, &s.erupt);
                    }
                    None => h.write_u32(0),
                }
            }
        }
        Mechanic::Blood => h.write_u32(6),
        Mechanic::Meter {
            value,
            colour,
            ascending,
        } => {
            h.write_u32(7);
            h.write_i32(*value);
            h.write_i32(colour.along());
            h.write_u32(*ascending as u32);
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

/// One frame of the camera's swing between its grounded framing and its
/// airborne one. See [`crate::camera`] for what the swing is and why.
///
/// **Last thing in the frame, on purpose.** Everything that aims reads the
/// scene as it stood at the *top* of the frame, which is the camera the player
/// was actually looking at when they pressed the button -- the renderer draws
/// frame *n-1*'s snapshot while frame *n*'s input is being made. Stepping this
/// first would aim against a framing nobody had seen yet.
fn step_aloft(p: &mut Player, field: &Field) {
    // How far the feet are above whatever they would land on. **Not `pos.y`**:
    // standing on a platform, on a stone or on the creature's back is
    // standing, and a framing that measured height from the world's origin
    // would swing the view around for somebody who has not left the ground.
    // `grounded` is exactly "my feet are on something", creature included.
    let height = if p.grounded {
        Fx::ZERO
    } else {
        p.pos.y.sub(aim::settle(p.pos, field).y)
    };
    p.aloft = camera::aloft_step(p.aloft, camera::aloft_target(height));
}

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
                    // The recall's slow, offered the same way everything else
                    // is. It is half the reason to recall through something.
                    beast.take_control(monster::Control::slowing(
                        t::slow_frames(),
                        t::shadow_recall_slow(),
                    ));
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
                // **The one hit in the game that does not take frames.** The
                // shadow can be sent on the frame it is asked for and out of
                // any recovery, so if it also interrupted, the Reaver would
                // never have to commit to anything: throw the risky move, and
                // when it goes wrong, recall through whoever is punishing you
                // and take their attack off them. A shine, and a worse one,
                // because not even her own animations gate it. It cuts and it
                // slows; what it may not do is hand her the exchange back.
                interrupts: false,
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
            // does the same at its own turn, with twelve blades instead of one.
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
            // And the one that comes back to a *person*. The blade's way home
            // is drawn to wherever its caster is standing this frame, so
            // walking while it is in the air bends its return rather than
            // leaving it to arrive at an empty patch of arena.
            //
            // **Not updated once the caster is dead.** There is nobody to catch
            // it, so it finishes its arc to the last place they stood and pays
            // out nothing -- `Player::heal` does not reach a corpse.
            if effect.kind.comes_home() {
                if let Some(owner) = self
                    .players
                    .get(effect.owner as usize)
                    .filter(|owner| owner.health > 0)
                {
                    effect.home = aim::origin(owner.pos);
                }
            }
            // And the one that is a **line between two live things**. A tether
            // runs from the caster's hand to whatever it caught, and both ends
            // walk around: `pos` is her hand this frame and `home` is the body
            // on the other end, or the far end of the throw while the line is
            // still looking for something. Written before the catch is tested,
            // so the line that hits is the line that is drawn.
            //
            // It is the same argument `state::beam_of` makes about a skillshot
            // -- locked target, live origin -- with the target also alive.
            if effect.kind == EffectKind::Tether {
                let hand = moves::get(effect.class, effect.slot).hand;
                if let Some(owner) = self.players.get(effect.owner as usize) {
                    effect.pos = aim::hand_origin(owner.pos, owner.facing, hand);
                }
                effect.home = match effect.caught() {
                    Some(victim) => aim::origin(self.players[victim].pos),
                    None => effect.pos.add(effect.dir.scale(effect.reach)),
                };
            }
            // A tornado's expiry is not its growth's question. `age`/`life`
            // keep answering how grown it is, continuously, from whenever the
            // pillar it came from was first planted -- see
            // `Effect::pillar_volumes`. Whether it has *burned out* is asked
            // on the clock `Effect::tornado_pos` already keeps, frames since
            // it was cut loose rather than frames since it was planted, plus
            // the one thing a planted effect never has to check: whether it
            // has wandered off the map, the one effect whose centre actually
            // can.
            // A pool drains rather than ages out: a fixed volume a second,
            // so a big one outlives a small one, and it is gone when it is
            // dry. The drain is taken here, before the frame's drinks, so a
            // pool that is about to vanish cannot be drunk from on the frame
            // it does.
            if effect.is_a_pool() {
                effect.banked -= effect.pool_drained_this_frame();
            }
            let expired = if effect.kind == EffectKind::FireTornado {
                let flying = effect.age.saturating_sub(effect.banked as u16);
                flying >= t::tornado_travel_life() || !arena::inside(effect.tornado_pos())
            } else if effect.is_a_pool() {
                effect.banked <= 0
            } else {
                effect.age >= effect.life
            };
            if expired {
                self.effects[i] = None;
                self.pay_out(&effect);
                continue;
            }
            self.apply_effect(&mut effect);
            // The bolt is spent by what it just did -- it reached a body --
            // and goes now rather than being drawn one more frame at the far
            // end of a flight it never made. Nothing else expires here: a
            // tornado, for one, is older than its life by design.
            let spent = effect.kind == EffectKind::Haemorrhage && effect.age >= effect.life;
            self.effects[i] = (!spent).then_some(effect);
        }
        self.step_bleeds();
        // The slow's tail, for every source of one -- a drain field here, a
        // stone churning under your feet in `stones`. It runs down before either
        // of them gets to refresh it, so standing in one holds the slow at full
        // strength and walking out of it lets the tail run. A grab's bind has
        // no tail and is not ticked here: it belongs to the hold, and
        // `drag_the_held` runs it down as part of running the hold.
        for p in self.players.iter_mut() {
            p.slowed = p.slowed.saturating_sub(1);
            if p.slowed == 0 {
                p.slow_mul = Fx::ONE;
            }
            // And its mirror, run down on the same tick and for the same
            // reason: whatever hastened her gets to refresh it before it
            // expires, so standing in the field holds it and leaving lets the
            // tail run.
            p.hasted = p.hasted.saturating_sub(1);
        }
    }

    /// Every bleed runs down a frame, and on its tick takes its damage and
    /// spills it under the victim -- wherever they have got to, which is the
    /// whole idea: a bleeding fighter who keeps moving lays a trail of her
    /// pools, and a spike on any pool of the trail chains along it toward
    /// them. Standing still merges the ticks into one growing pool instead,
    /// which is worse for them, since one pool is one eruption.
    ///
    /// Not a hit: no stun, no block, no knockback, and it goes through a
    /// guard, because it was the bolt that had to land. It cannot kill --
    /// `wound` stops at nought like any damage -- and it stops when the
    /// victim is dead.
    fn step_bleeds(&mut self) {
        for i in 0..MAX_PLAYERS {
            let p = self.players[i];
            if p.bleeding == 0 {
                continue;
            }
            let owner = p.bled_by as usize;
            if p.health <= 0 || owner >= MAX_PLAYERS {
                self.players[i].bleeding = 0;
                continue;
            }
            self.players[i].bleeding -= 1;
            if self.players[i].bleeding % t::bleed_tick() != 0 {
                continue;
            }
            let dealt = self.players[i].wound(t::bleed_damage());
            let class = self.players[owner].class;
            self.spill_under(owner as u8, class, moves::blood::HAEMORRHAGE, &p, dealt);
        }
    }

    /// What an expiring effect still owes its caster.
    ///
    /// Only the blade owes anything: everything else paid as it went. **Being
    /// caught is the payday**, which is what makes the ability a small
    /// commitment rather than a free poke -- the cut lands at once and the
    /// health has to survive the flight home.
    ///
    /// The blade expiring *is* it arriving, and that is a property of
    /// `Effect::blade_at` rather than a coincidence of two clocks: the return
    /// leg is drawn to the caster's live position and reaches it exactly as the
    /// life runs out. The one way not to be paid is not to be there -- a dead
    /// caster catches nothing, which `Player::heal` enforces on its own.
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
        // Nothing here reads the caster's bar directly: what the Dual mage's
        // depth is worth to a thing she left behind is folded into
        // `Effect::field_radius` and `Effect::damage` at the moment it was
        // cast, so that the hit test and the overlay are the same size without
        // the overlay having to find her. See [`depth`] and `Effect::power`.
        match effect.kind {
            EffectKind::FirePillar => {
                if effect.ticks_now() {
                    self.burn_the_pillar(effect, effect.pos);
                }
            }

            // The exact same burn a standing pillar gives -- same two
            // volumes, same tick, same number -- centred on the tornado's own
            // live position instead of the spot it was cast on. What is new
            // is the catch: the first frame it reaches somebody, the same
            // eruption a fire pillar already throws the instant it is cast
            // (see `hitbox`, reused via `effect.source()`) lands on them once
            // more, as a real hitstun -- not extra damage, the burn already
            // has that. That stagger is not a flourish: `Player`'s own
            // grounded movement *sets* velocity from the stick every frame a
            // fighter is free to act, which would erase the pull and the
            // carry below before either ever moved anyone. Stunned, movement
            // instead decays whatever velocity is already there (see
            // `step_player`'s `stunned()` branch), which is the one state
            // they can actually win inside. Once the stagger runs out a
            // fighter is free again and can walk or jump clear -- the pull
            // and the carry keep trying regardless, but their own legs win
            // the moment they are allowed to use them.
            EffectKind::FireTornado => {
                let at = effect.tornado_pos();
                if effect.ticks_now() {
                    self.burn_the_pillar(effect, at);
                }
                let (base, column) = effect.pillar_volumes();
                let radius = t::body_radius();
                let height = t::body_height();
                for i in 0..MAX_PLAYERS {
                    let p = self.players[i];
                    if !self.effects_reach(i, effect.owner)
                        || (!base.contains(at, p.pos, radius, height)
                            && !column.contains(at, p.pos, radius, height))
                    {
                        continue;
                    }
                    if effect.take_hit(0, i) {
                        let m = effect.source();
                        let (guarding, parried) =
                            guard_against(&self.players[i], at, m.unblockable);
                        apply_hit(
                            &mut self.players[i],
                            Hit {
                                damage: 0,
                                hitstun: m.hitstun,
                                blockstun: m.blockstun,
                                knockback: Fx::ZERO,
                                launch: Fx::ZERO,
                                grabs: 0,
                                by: effect.owner,
                                dir: V3::ZERO,
                                blocked: guarding,
                                parried,
                                interrupts: true,
                            },
                        );
                        if parried {
                            self.players[i].parried = PARRY_FLOURISH;
                        }
                    }
                    // Carried first: moved by exactly as far as the tornado's
                    // own centre moves this frame, the same way a rising
                    // stone carries whoever is standing on it (see
                    // `stones::resolve_body`) -- not by accelerating his
                    // velocity toward the tornado's, which can never actually
                    // catch up: `step_player`'s `stunned` branch decays
                    // whatever velocity a stunned fighter already has every
                    // single frame, so a steering force has to first cancel
                    // that decay before it can add anything, and it never
                    // fully does. Moving him with it directly cannot fall
                    // behind at all.
                    let step = effect.dir.scale(t::tornado_speed().mul(DT));
                    self.players[i].pos.x = self.players[i].pos.x.add(step.x);
                    self.players[i].pos.z = self.players[i].pos.z.add(step.z);
                    // The suck, against where the carry just put him rather
                    // than where he stood at the top of the frame -- `at` is
                    // already this frame's centre, and the carry just moved
                    // him to keep pace with it, so measuring against his
                    // stale, pre-carry position would read the tornado's own
                    // stride as if it were distance to close, and pull him
                    // forward on top of a ride that already keeps up on its
                    // own. What is left once the carry is accounted for is
                    // genuine radial drift -- off to one side of the axis,
                    // say -- and that is what the pull is actually for.
                    let apart = V3::new(
                        self.players[i].pos.x.sub(at.x),
                        Fx::ZERO,
                        self.players[i].pos.z.sub(at.z),
                    );
                    let dist = apart.flat_len();
                    if dist.raw() > 0 {
                        let inward = apart.normalized();
                        let pull = t::tornado_pull().mul(DT);
                        self.players[i].vel.x = self.players[i].vel.x.sub(inward.x.mul(pull));
                        self.players[i].vel.z = self.players[i].vel.z.sub(inward.z.mul(pull));
                    }
                }
            }

            // Drain *and* slow. The slow is the part that matters: damage alone
            // makes a puddle you step out of, and the slow is what makes leaving
            // cost time -- which is what turns it into something you put
            // *between* yourself and someone else.
            // The spike standing out of the floor. On bare ground it has
            // already done its work -- the move's own disc hit on the frame
            // it appeared -- and this is the thing you can see from across
            // the arena. Erupted from a pool it is the hit: everything
            // standing in the pool's disc, on its first frame, launched and
            // slowed at the move's damage. Nothing ticks and nothing drains;
            // the spike either seeds a pool or cashes one in, and both are
            // one event the other player can watch happen.
            EffectKind::BlackSpike => {
                if effect.erupted() && effect.age == 1 {
                    let volume = effect.spike_volume();
                    let m = effect.source();
                    for i in 0..MAX_PLAYERS {
                        let p = self.players[i];
                        if !self.effects_reach(i, effect.owner)
                            || effect.already_hit(0, i)
                            || !volume.contains(
                                effect.pos,
                                p.pos,
                                t::body_radius(),
                                p.hurt_height(),
                            )
                        {
                            continue;
                        }
                        effect.take_hit(0, i);
                        let blow = Fx::from_int(m.damage).mul(t::erupt_damage()).to_int();
                        self.cut(i, effect, effect.pos, blow);
                        self.players[i].slow(t::slow_frames(), t::spike_slow());
                    }
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
                    effect.banked += self.cut(i, effect, at, effect.source().damage);
                }
                effect.banked += self.gore_the_creature(effect, pass, at, radius);
                // The return leg crossing pools on the floor is a small drink
                // on the way back -- each pool once per blade. Only on the way
                // home: the throw is a cut, and the health is where the cut
                // lands.
                if effect.returning() {
                    let m = effect.source();
                    for slot in 0..MAX_EFFECTS {
                        if effect.drank_from(slot) {
                            continue;
                        }
                        let over = self.effects[slot].is_some_and(|pool| {
                            pool.is_a_pool()
                                && pool.owner == effect.owner
                                && pool.covers(V3::new(at.x, pool.pos.y, at.z))
                        });
                        if over {
                            effect.mark_drank(slot);
                            self.drink_from(slot, effect.owner as usize, &m);
                        }
                    }
                }
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
                let half_thick = t::lotus_blade_thickness();
                for blade in 0..LOTUS_BLADES {
                    let (was, at) = effect.lotus_span(blade, effect.pos);
                    for i in 0..MAX_PLAYERS {
                        if !self.effects_reach(i, effect.owner)
                            || effect.already_hit(blade, i)
                            || !self.sliced(i, was, at, radius, half_thick)
                        {
                            continue;
                        }
                        effect.take_hit(blade, i);
                        let share = if coming_back {
                            t::lotus_return_damage()
                        } else {
                            Fx::ONE
                        };
                        let blow = Fx::from_int(effect.source().damage).mul(share).to_int();
                        self.cut(i, effect, at, blow);
                        self.players[i].slow(t::slow_frames(), t::lotus_slow());
                    }
                    self.gore_the_creature(effect, blade, at, radius);
                }
            }

            // The light Lance's burst, where the line ran out. One pass: it
            // catches each body once and then hangs there for a few frames
            // being looked at, which is the whole of what a detonation is.
            EffectKind::LanceBurst => {
                let radius = effect.field_radius();
                for i in 0..MAX_PLAYERS {
                    if !self.effects_reach(i, effect.owner)
                        || effect.already_hit(0, i)
                        || !self.inside(i, effect.pos, radius)
                    {
                        continue;
                    }
                    effect.take_hit(0, i);
                    let dealt =
                        self.cut(i, effect, effect.pos, effect.kind.damage(&effect.source()));
                    let owed = effect.leeched(dealt);
                    self.players[effect.owner as usize].heal(owed);
                }
                // And the creature, once. A burst is not a field: it is marked
                // spent the moment it actually connects, rather than every
                // frame it exists, so that walking a Ridgeback into one late
                // still costs it something.
                let dealt = self.gore_the_creature(effect, 0, effect.pos, radius);
                if dealt > 0 {
                    let owed = effect.leeched(dealt);
                    self.players[effect.owner as usize].heal(owed);
                    effect.take_hit(0, QUARRY_VICTIM);
                }
            }

            // The dark Lance. Two effects in one lifetime: a line looking for
            // something, and then a leash.
            //
            // The catch window is the **move's own active frames**, read off
            // its row rather than written down here, so the line is live for
            // exactly as long as the hitbox of any other move would be. Caught
            // nothing by the end of it and the effect is spent -- the line is
            // still thrown and still drawn for those frames, because a whiff
            // you cannot see is a whiff you cannot learn from, but a tether
            // with nothing on the end of it is a thing the overlay would draw
            // and the player would believe.
            EffectKind::Tether => {
                let thrown_for = effect.source().active.max(1);
                if effect.caught().is_none() {
                    let radius = effect.field_radius();
                    let far = effect.pos.add(effect.dir.scale(effect.reach));
                    for i in 0..MAX_PLAYERS {
                        if !self.effects_reach(i, effect.owner)
                            || !self.on_the_line(i, effect.pos, far, radius)
                        {
                            continue;
                        }
                        effect.take_hit(0, i);
                        // **Guard denies the hold.** The line still lands as an
                        // ordinary blow -- blockstun, the shove, all of it --
                        // and then goes slack, which is the counterplay a drain
                        // that lasts two seconds has to have. Asked before
                        // `cut`, because `cut` answers the same question
                        // internally and only gives back damage, and a tether
                        // whose attach condition was "dealt more than nought"
                        // would also refuse to take hold of somebody at one
                        // health.
                        let (guarding, _) = guard_against(
                            &self.players[i],
                            effect.pos,
                            effect.source().unblockable,
                        );
                        self.cut(i, effect, effect.pos, effect.source().damage);
                        if guarding {
                            effect.forget_hits();
                            effect.life = effect.age;
                        }
                        break;
                    }
                    // The one pass it gets at the creature, on the same frames.
                    self.feed_the_caster(effect, 0, far, radius);
                    if effect.caught().is_none() && effect.age >= thrown_for {
                        effect.life = effect.age;
                    }
                }
                if let Some(victim) = effect.caught() {
                    let owner = effect.owner as usize;
                    let apart = V3::new(
                        self.players[victim].pos.x.sub(self.players[owner].pos.x),
                        Fx::ZERO,
                        self.players[victim].pos.z.sub(self.players[owner].pos.z),
                    )
                    .flat_len();
                    // **The leash is the cost of the ability, and it is the one
                    // number on this class the depth curve does not touch.**
                    // Depth buys her a harder drain and a longer hold, which is
                    // power; how far she may stray from what she caught is a
                    // *rule*, and the rule is the whole ability -- a fragile
                    // melee mage has to stand next to the thing she is draining.
                    // A leash that grew with the bar would hand the deep version
                    // of the cast the one thing it is supposed to have to pay
                    // for, and at the edge it would reach most of the arena.
                    let leash = t::tether_leash();
                    if apart.raw() > leash.raw() || self.players[victim].health <= 0 {
                        effect.life = effect.age;
                    } else if effect.ticks_now() {
                        self.drain(victim, effect);
                    }
                }
            }

            // Judgement's field. The strike already happened; this is the
            // ground it left, and it points two ways at once -- it burns
            // anybody standing in it and it makes her fast while she is in it.
            EffectKind::JudgementField => {
                let radius = effect.field_radius();
                if effect.ticks_now() {
                    for i in 0..MAX_PLAYERS {
                        if self.effects_reach(i, effect.owner) && self.inside(i, effect.pos, radius)
                        {
                            self.drain(i, effect);
                        }
                    }
                    self.feed_the_caster(effect, 0, effect.pos, radius);
                }
                let owner = effect.owner as usize;
                if self.inside(owner, effect.pos, radius) {
                    // Refreshed every frame she is in it and given the same
                    // tail a slow has, so walking out of it runs down rather
                    // than snapping off at the edge.
                    self.players[owner].hasten(t::slow_frames());
                }
            }

            // Four arms, each its own skillshot, and all four of them the price
            // of the root. Every arm is tested separately and remembers who it
            // has already caught, because "hit by all four" is a question about
            // *different* arms and one shared mask could not tell them apart.
            // A pool does nothing to anybody on its own. It is read by the
            // moves put through it -- see `drink_over` -- and by the dodge.
            EffectKind::Pool => {}

            // The bolt. Straight out along the line and spent on the first
            // body it reaches: the cut lands and the bleed opens, and from
            // then on the victim's own feet make the pools. A guarded bolt is
            // still spent -- it hit something -- and opens nothing.
            EffectKind::Haemorrhage => {
                let at = effect.bolt_at();
                let radius = effect.field_radius();
                let mut spent = false;
                for i in 0..MAX_PLAYERS {
                    if !self.effects_reach(i, effect.owner)
                        || effect.already_hit(0, i)
                        || !self.inside(i, at, radius)
                    {
                        continue;
                    }
                    effect.take_hit(0, i);
                    let dealt = self.cut(i, effect, at, effect.source().damage);
                    if dealt > 0 {
                        self.players[i].bleed(effect.owner, t::bleed_lasts());
                    }
                    spent = true;
                }
                if self.gore_the_creature(effect, 0, at, radius) > 0 {
                    spent = true;
                }
                if spent {
                    effect.age = effect.life;
                }
            }

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
                        let dealt = self.cut(i, effect, at, effect.source().damage);
                        let owed = effect.leeched(dealt);
                        self.players[effect.owner as usize].heal(owed);
                        // Caught by every one of them. The arms close, hold you
                        // where they closed, and then drag you back to the
                        // caster -- see `drag_the_held`.
                        //
                        // **The grab waits for all four**, and that is not a
                        // choice about flavour. A grab hauls its victim toward
                        // the caster, so one applied by the first arm to connect
                        // would pull them out from under the other three -- the
                        // bottom pair land a frame before the top pair -- and
                        // the catch would cancel itself.
                        if effect.parts_landed(i, GRASP_ARMS) == GRASP_ARMS {
                            let caught = effect.source().grabs;
                            if caught > 0 {
                                self.players[i].seized(effect.owner, caught, t::grasp_bind());
                            }
                        }
                    }
                    let dealt = self.gore_the_creature(effect, arm, at, radius);
                    let owed = effect.leeched(dealt);
                    self.players[effect.owner as usize].heal(owed);
                    // Against something that cannot be hauled, the catch hauls
                    // *her*: all four arms on a Ridgeback's flank pull her
                    // across the gap to the contact point. The same ability
                    // doing the same thing, with the heavier body winning.
                    if effect.parts_landed(QUARRY_VICTIM, GRASP_ARMS) == GRASP_ARMS {
                        let owner = effect.owner as usize;
                        if self.players[owner].haul == 0 {
                            let far = at.sub(self.players[owner].pos).len();
                            // The reel speed's slider does not reach zero, so
                            // a frame's step is never nothing to divide by.
                            let frames = far.div(t::reel_speed().mul(DT)).to_int().max(0) + 2;
                            self.players[owner].haul = frames.min(u16::MAX as i32) as u16;
                            self.players[owner].haul_to = at;
                        }
                    }
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

    /// Did a blade sweeping from `was` to `at` slice this fighter?
    ///
    /// **A disc, not a ball**, and the shape is the ability. A Guillotine blade
    /// is a shuriken thrown flat: wide in the plane the flower lies in, and
    /// barely there at all across it. Tested as a sphere it was a beach ball --
    /// at a radius wider than a fighter it swallowed everything near the line
    /// whatever its height, which is both the wrong picture and the wrong rule.
    ///
    /// So two tests rather than one. **Width is measured flat**, against the
    /// swept line, because that is the direction a blade is wide in. **Height
    /// is a slab**: the blade occupies `half_thick` either side of the plane,
    /// and has to overlap the body to touch it.
    ///
    /// The second one is the interesting half. The flower is planar and at
    /// waist height, so a slab that thin is something a fighter can **jump**,
    /// which a ball of the old radius was not -- it reached from the shins to
    /// the chest. That is the counterplay the shape was always supposed to
    /// imply and never did.
    fn sliced(&self, victim: usize, was: V3, at: V3, radius: Fx, half_thick: Fx) -> bool {
        let p = self.players[victim];
        let flat = |v: V3| V3::new(v.x, Fx::ZERO, v.z);
        let here = flat(p.pos);
        let wide = crate::math::segment_gap(flat(was), flat(at), here, here);
        if wide.raw() > radius.add(t::body_radius()).raw() {
            return false;
        }
        // The slab the blade sweeps this frame: it is level, so both ends are
        // at the same height and either will do.
        let lo = at.y.sub(half_thick);
        let hi = at.y.add(half_thick);
        p.pos.y.raw() <= hi.raw() && p.pos.y.add(p.hurt_height()).raw() >= lo.raw()
    }

    /// Is this fighter's body on the line between two points?
    ///
    /// The capsule test `resolve_hit` uses for a beam, in the shape an effect
    /// can ask it: measured against the whole standing body rather than flat,
    /// so a line thrown over somebody's head goes over it.
    fn on_the_line(&self, victim: usize, from: V3, to: V3, radius: Fx) -> bool {
        let p = self.players[victim];
        if p.action.invulnerable() {
            return false;
        }
        let spine = V3::new(p.pos.x, p.pos.y.add(p.hurt_height()), p.pos.z);
        crate::math::segment_gap(from, to, p.pos, spine).raw() <= radius.add(t::body_radius()).raw()
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

    /// One tick of a fire pillar's burn, centred on `at` -- the pillar's own
    /// `pos` normally, or a tornado's live, moving one. Shared so that a
    /// tornado's tick is not a second copy of a pillar's own numbers with a
    /// chance to disagree with them; see `crate::effects::EffectKind::FireTornado`.
    fn burn_the_pillar(&mut self, effect: &mut Effect, at: V3) {
        let (base, column) = effect.pillar_volumes();
        let radius = t::body_radius();
        let height = t::body_height();
        for i in 0..MAX_PLAYERS {
            let p = self.players[i];
            if !self.effects_reach(i, effect.owner) {
                continue;
            }
            if base.contains(at, p.pos, radius, height)
                || column.contains(at, p.pos, radius, height)
            {
                self.drain(i, effect);
            }
        }
        self.feed_the_caster(effect, 0, at, base.radius);
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
        let dealt = self.players[victim].wound(damage);
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
    /// `damage` is what this particular contact is worth, before the caster's
    /// own power and before any bonus against a disabled target. Almost always
    /// the move's own number, because for an effect that carries the hit the
    /// move *is* the ability -- but not always, and the two cases it is not are
    /// the reason it is an argument rather than read off the move here. The
    /// lotus coming home is a fraction of what its eruption did, because the
    /// two are different events and pricing them the same would make a recall
    /// through a crowd worth double an execute. And the Dual mage's light Lance
    /// throws a line that pokes and a burst that kills, which are two numbers
    /// in two places for one cast.
    fn cut(&mut self, victim: usize, effect: &Effect, from: V3, damage: i32) -> i32 {
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
        // Then the bonus against a target who cannot move, and the caster's own
        // power -- which is one for everybody but the Dual mage. See
        // `Effect::power`.
        let damage = Fx::from_int(damage)
            .mul(preying(effect.class, p.disabled()))
            .mul(effect.power)
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
                interrupts: true,
            },
        );
        if parried {
            self.players[victim].parried = PARRY_FLOURISH;
        }
        // The Blood mage's double duty, for a hit her effects deliver: drink
        // from a pool the victim stands in, then spill what was dealt. See the
        // melee loop in `advance` for the order. **Not for the blade**: it
        // drinks on its way home, from each pool it crosses, once -- a cut
        // that also drank would take three shares of one pool in one throw.
        if dealt > 0 {
            if !effect.kind.comes_home() {
                self.drink_over(effect.owner as usize, &m, from, from, Some(victim));
            }
            self.spill_under(effect.owner, effect.class, effect.slot, &p, dealt);
        }
        dealt
    }

    /// A field ticking on the creature, and the caster's share of it straight
    /// back -- the same payment the fighter path makes in `drain`.
    ///
    /// It exists because the two field effects used to call `gore_the_creature`
    /// and **throw the answer away**, so a Blood mage draining a Ridgeback was
    /// taking health off it and getting none of it back. Invisible in versus,
    /// where a field never touches the creature at all, and invisible to a test
    /// that only asked whether her health went up: the eruption's own hit pays
    /// out on the same cast, a few frames earlier, which is enough to make a
    /// broken field look like a working one.
    fn feed_the_caster(&mut self, effect: &mut Effect, part: usize, at: V3, radius: Fx) {
        let dealt = self.gore_the_creature(effect, part, at, radius);
        let owed = effect.leeched(dealt);
        self.players[effect.owner as usize].heal(owed);
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
        // The same control a fighter would have taken, offered rather than
        // applied: the creature decides how much of it it is currently in a
        // state to feel. See `monster::Monster::take_control`.
        beast.take_control(effect.control());
        self.monster = Some(beast);
        // A field has one part and hits over and over on its tick; a blade or an
        // arm has a pass to spend and spends it here.
        if effect.kind.travels() {
            effect.take_hit(part, QUARRY_VICTIM);
        }
        // The creature bleeds too: under the struck part, projected to the
        // floor, which is what makes the pool under a toppled Ridgeback a
        // door onto its back.
        if dealt > 0 {
            let m = effect.source();
            self.drink_over(effect.owner as usize, &m, at, at, None);
            self.spill(
                effect.owner,
                effect.class,
                effect.slot,
                floor_under(at),
                dealt,
            );
        }
        dealt
    }
}

/// Bind a grabbed fighter, then haul them to their captor's arm's length.
///
/// **Two phases, and the first one is why this is not a teleport.** For
/// `Player::bound` frames nothing moves: you are caught, and the arms are
/// holding you exactly where they closed. Only then does the haul start, and it
/// covers real ground at `tuning::reel_speed` metres a second, so what the
/// other player sees is a body travelling rather than a body that has already
/// arrived.
///
/// The bind is not decoration. It is the window the Blood mage spends starting
/// whatever is supposed to meet the victim at the end of the trip -- and
/// because it is short, that cast has to have been started *before* the arms
/// connected. See `docs/design/kits/blood-mage.md`.
///
/// Once the haul arrives the victim is pinned at arm's length for whatever is
/// left of the hold, which is the Bulwark's Grapple unchanged: its catch is
/// already at arm's length, so there is nothing to haul and the two phases
/// collapse into the behaviour it always had.
/// Which of this fighter's pools the crosshair is on, with a clear line to
/// it -- the two questions the Reaver's dash asks about her shadow, pointed
/// at the class's object instead. Both are `aim`'s to answer; this only
/// decides which pool, and the nearest is the one a player means.
fn pool_under_the_crosshair(p: &Player, who: usize, input: Input, scene: &Scene) -> Option<usize> {
    if !p.class.wounds_go_grey() {
        return None;
    }
    let mut best: Option<(usize, Fx)> = None;
    for (slot, e) in scene.effects.iter().enumerate() {
        let Some(e) = e else { continue };
        if !e.is_a_pool() || e.owner != who as u8 {
            continue;
        }
        let wide = e.pool_radius().add(t::body_radius());
        let tall = e.pool_height().add(t::pool_lock());
        if !aim::pointing_at_disc(who, input, e.pos, wide, tall, scene)
            || !aim::clear_between(p.pos, e.pos, scene)
        {
            continue;
        }
        let far = e.pos.sub(p.pos).len();
        if best.is_none_or(|(_, seen)| far.raw() < seen.raw()) {
            best = Some((slot, far));
        }
    }
    best.map(|(slot, _)| slot)
}

/// Being hauled by her own Grasp: the velocity that takes her there this
/// frame, or `None` if she is not.
///
/// Arrives exactly rather than overshooting -- the last step is the gap --
/// and counts the frames down so a haul that meets something solid on the
/// way still ends.
fn haul_drive(p: &mut Player) -> Option<V3> {
    if p.haul == 0 {
        return None;
    }
    p.haul -= 1;
    let gap = p.haul_to.sub(p.pos);
    let far = gap.len();
    let step = t::reel_speed().mul(DT);
    if far.raw() <= step.raw() {
        p.haul = 0;
        return Some(gap.scale(Fx::ONE.div(DT)));
    }
    Some(gap.normalized().scale(t::reel_speed()))
}

/// The patch of floor under a point in the air: the arena's ground, and the
/// creature's contact points are always over it. Stones are not consulted --
/// a pool spilled onto a raised structure is an open question in the design,
/// and until it is answered blood falls to the floor.
fn floor_under(at: V3) -> V3 {
    V3::new(at.x, GROUND_Y, at.z)
}

impl World {
    /// Spill `volume` of `victim`'s blood under their feet, for a hit `owner`
    /// landed on them.
    ///
    /// Only the Blood mage spills anybody (`Class::wounds_go_grey` is the
    /// same predicate read the other way: she is the one class with a use for
    /// blood on the floor), and only a fighter on the ground leaves a pool.
    /// One hit in the air spills nowhere: whether it should land where they
    /// do is an open question in `docs/design/blood-mage.md`, and the simpler
    /// answer is the one built until somebody plays it.
    ///
    /// `victim` is the body **as it stood when the hit landed**, not as it is
    /// afterwards: a spike launches what it hits, and a victim read after the
    /// launch is airborne and spills nowhere, which would make the one move
    /// meant to seed a pool at range the one move that never does.
    fn spill_under(&mut self, owner: u8, class: Class, slot: u8, victim: &Player, volume: i32) {
        if !victim.grounded {
            return;
        }
        self.spill(owner, class, slot, victim.pos, volume);
    }

    /// Put `volume` of blood on the floor at `at`, owned by `owner`.
    ///
    /// Three rules, and they are the whole of what keeps four pools readable:
    /// a spill onto a pool of hers merges into it rather than stacking a
    /// second disc on the same floor; a spill past `tuning::pool_cap` merges
    /// into the newest; and her own costs never come through here at all --
    /// a cost is paid into the ability, not onto the floor, or every cast
    /// would leave a free heal at her own feet.
    fn spill(&mut self, owner: u8, class: Class, slot: u8, at: V3, volume: i32) {
        if volume <= 0 || !class.wounds_go_grey() {
            return;
        }
        let born = Effect::pool(owner, class, slot, at, volume);
        // Onto one of hers already there: merge.
        let overlapping = self.effects.iter().position(|e| {
            e.is_some_and(|e| {
                e.is_a_pool()
                    && e.owner == owner
                    // Within a body of each other: a figure spilled where
                    // one already stands joins it rather than crowding it.
                    && V3::new(e.pos.x.sub(at.x), Fx::ZERO, e.pos.z.sub(at.z))
                        .flat_len()
                        .raw()
                        <= e.pool_radius()
                            .add(born.pool_radius())
                            .add(t::body_radius())
                            .raw()
            })
        });
        if let Some(i) = overlapping {
            if let Some(pool) = self.effects[i].as_mut() {
                pool.banked += volume;
            }
            return;
        }
        // Past the cap: into the newest.
        let mut count = 0;
        let mut newest: Option<(usize, u16)> = None;
        for (i, e) in self.effects.iter().enumerate() {
            let Some(e) = e else { continue };
            if !e.is_a_pool() || e.owner != owner {
                continue;
            }
            count += 1;
            if newest.is_none_or(|(_, age)| e.age < age) {
                newest = Some((i, e.age));
            }
        }
        if count >= t::pool_cap() {
            if let Some((i, _)) = newest {
                if let Some(pool) = self.effects[i].as_mut() {
                    pool.banked += volume;
                }
            }
            return;
        }
        spawn_effect(&mut self.effects, born);
    }

    /// The pool of `owner`'s that a point on the floor is inside, if any.
    fn pool_covering(&self, owner: u8, at: V3) -> Option<usize> {
        self.effects
            .iter()
            .position(|e| e.is_some_and(|e| e.is_a_pool() && e.owner == owner && e.covers(at)))
    }

    /// A move of `owner`'s landed with its volume between `from` and `to`,
    /// on `victim` if it hit a fighter: drink from a pool of hers under the
    /// hit, if there is one.
    ///
    /// **The one rule of the heal**: it is where the blood is, and she has to
    /// put something through it. A pool counts if the victim is standing in
    /// it or the hit volume passes over its disc; the fullest one is drunk.
    /// What comes back is the move's own share of the pool, converted out of
    /// grey and never past it, and the pool loses exactly what she got.
    fn drink_over(
        &mut self,
        owner: usize,
        m: &moves::Move,
        from: V3,
        to: V3,
        victim: Option<usize>,
    ) -> i32 {
        if m.drink == 0 {
            return 0;
        }
        let feet = victim.map(|v| self.players[v].pos);
        let mut best: Option<(usize, i32)> = None;
        for (slot, e) in self.effects.iter().enumerate() {
            let Some(e) = e else { continue };
            if !e.is_a_pool() || e.owner != owner as u8 {
                continue;
            }
            let standing_in = feet.is_some_and(|f| e.covers(f));
            let flat = |v: V3| V3::new(v.x, e.pos.y, v.z);
            let passes_over = crate::math::segment_gap(flat(from), flat(to), e.pos, e.pos).raw()
                <= e.pool_radius().add(t::body_radius()).raw();
            if (standing_in || passes_over) && best.is_none_or(|(_, v)| e.banked > v) {
                best = Some((slot, e.banked));
            }
        }
        match best {
            Some((slot, _)) => self.drink_from(slot, owner, m),
            None => 0,
        }
    }

    /// A spike came up on the pool in `slot`: the pool erupts, and so does
    /// every pool of hers the eruption covers, each at its own size.
    ///
    /// **The chain is the skill curve.** A spike on one pool is a bigger spike;
    /// a spike on a pool standing among others is the floor coming up across
    /// the whole fight, and arranging that is a thing a player learns. Each
    /// eruption drinks its own pool first, so the chain cannot run twice over
    /// the same blood, and it is bounded by the pools there are.
    fn erupt_from(&mut self, owner: usize, kind: u8, slot: usize) {
        let Some(pool) = self.effects[slot].filter(|e| e.is_a_pool()) else {
            return;
        };
        let m = moves::get(self.players[owner].class, kind);
        // Sized by the pool's essence, before it is drunk: the eruption is the
        // whole pool coming up at once.
        let radius = t::erupt_radius()
            .mul(Fx::from_int(pool.pool_volume().max(0)).sqrt())
            .max(m.radius);
        self.drink_from(slot, owner, &m);
        let mut born = Effect::cast(
            EffectKind::BlackSpike,
            owner as u8,
            self.players[owner].class,
            kind,
            pool.pos,
            V3::ZERO,
            radius,
        );
        born.banked = 1;
        spawn_effect(&mut self.effects, born);
        for next in 0..MAX_EFFECTS {
            let covered = self.effects[next].is_some_and(|e| {
                e.is_a_pool()
                    && e.owner == owner as u8
                    && V3::new(e.pos.x.sub(pool.pos.x), Fx::ZERO, e.pos.z.sub(pool.pos.z))
                        .flat_len()
                        .raw()
                        <= radius.add(e.pool_radius()).raw()
            });
            if covered {
                self.erupt_from(owner, kind, next);
            }
        }
    }

    /// Drink every pool of this fighter's that her scythe's volume is passing
    /// over this frame. Only the two scythe moves, and only while the volume
    /// is out.
    ///
    /// **Only pools that were there before the swing.** A hit spills a pool
    /// under whoever it cut, and the blade is over that spot on the same
    /// frame; if the swing collected it, every hit would refund itself on the
    /// way through and nothing would ever be left on the floor for the next
    /// swing or the spike. So a pool younger than the swing is left alone --
    /// the hit itself already drank whatever the blade passed *through*, in
    /// `drink_over`, before it spilled.
    fn collect_with_the_scythe(&mut self, who: usize) {
        let p = self.players[who];
        let Some(kind) = p.action.attack_kind() else {
            return;
        };
        if !p.class.wounds_go_grey() || !moves::blood::scythe(kind) {
            return;
        }
        let Some(hb) = hitbox(&p) else {
            return;
        };
        let m = moves::get(p.class, kind);
        let swing_so_far = m.startup + m.active;
        for slot in 0..MAX_EFFECTS {
            let over = self.effects[slot].is_some_and(|e| {
                e.is_a_pool()
                    && e.owner == who as u8
                    && e.age > swing_so_far
                    && crate::math::segment_gap(
                        V3::new(hb.from.x, e.pos.y, hb.from.z),
                        V3::new(hb.to.x, e.pos.y, hb.to.z),
                        e.pos,
                        e.pos,
                    )
                    .raw()
                        <= hb.radius.add(e.pool_radius()).add(t::body_radius()).raw()
            });
            if over {
                self.drink_from(slot, who, &m);
            }
        }
    }

    /// Drink the pool in `slot`: `m`'s share of what is left in it comes back
    /// as red, grey is the ceiling, and **the pool is gone**. One and done: a
    /// pool is a heal you take once, worth what it has left, and what she
    /// could not fill is lost with it -- a pool she could stand in and hit
    /// forever would be a heal with no decision in it.
    fn drink_from(&mut self, slot: usize, owner: usize, m: &moves::Move) -> i32 {
        let Some(pool) = self.effects[slot] else {
            return 0;
        };
        if !pool.is_a_pool() {
            return 0;
        }
        let take = m.drinks(pool.banked);
        let got = self.players[owner].drink(take);
        self.effects[slot] = None;
        got
    }
}

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
        // Held is held either way: no momentum of your own, and no falling out
        // of the arms that have you.
        victim.vel = V3::ZERO;
        victim.grounded = holder.grounded;
        if victim.bound > 0 {
            victim.bound -= 1;
            continue;
        }
        let arm = t::body_radius().add(t::body_radius());
        let want = V3::new(
            holder.pos.x.add(holder.facing.x.mul(arm)),
            holder.pos.y,
            holder.pos.z.add(holder.facing.z.mul(arm)),
        );
        let gap = want.sub(victim.pos);
        let far = gap.len();
        let step = t::reel_speed().mul(crate::DT);
        victim.pos = if far.raw() <= step.raw() {
            want
        } else {
            victim.pos.add(gap.scale(step.div(far)))
        };
    }
}

/// How long a thing this fighter leaves behind gets to last.
///
/// The move table's own number for everybody, and that number on the Dual
/// mage's depth curve. Never below one frame: an effect with no life at all is
/// one that is removed before it is ever applied, which is a cast that silently
/// does nothing.
fn lasting(p: &Player, life: u16) -> u16 {
    Fx::from_int(life as i32)
        .mul(depth(p))
        .to_int()
        .clamp(1, u16::MAX as i32) as u16
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
    let speed = if p.hasted > 0 {
        speed.mul(t::judgement_field_speed())
    } else {
        speed
    };
    if p.slowed == 0 {
        return speed;
    }
    // A slow applies **after** the haste rather than instead of it, so being
    // slowed inside your own field is the two of them arguing and the slow
    // winning by however much it is worth. Either order gives the same product;
    // what matters is that neither cancels the other outright, because a class
    // that could not be slowed while standing on her own ground would have
    // written herself an immunity nobody agreed to.
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
fn step_rider(
    p: &mut Player,
    who: usize,
    input: Input,
    beast: &Monster,
    scene: &Scene,
    out: &[bool; moves::MAX_SLOTS],
) {
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
    let look = V3::from_turns(input.aim_turns());
    // A channel is the aiming, so the body keeps turning through it. Every
    // other action locks the facing -- see `Action::Channel`.
    if p.action.actionable() || p.action.stunned() || p.action.channelling().is_some() {
        p.facing = look;
    } else if p.action.guarding() {
        p.facing = p
            .facing
            .add(look.sub(p.facing).scale(t::guard_turn_rate()))
            .normalized();
    }
    p.crouching = input.has(Input::CROUCH) && p.action.actionable();

    // The buck. Acceleration is read in the frame of **the part underfoot**,
    // where that surface's normal is simply `+y`, and the part of it pressing
    // the rider *into* the surface does not count -- being shoved down onto
    // something is not being thrown off it.
    //
    // The part's frame rather than the body's, because the creature is a
    // skeleton: a tail that whips throws whoever is on the tail and does
    // nothing to somebody standing on the shoulder, and reading both in one
    // shared frame is how that distinction gets lost.
    let rig = beast.rig();
    let felt = rig.dir_to_part(part, accel);
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
        thrown_off(p, &rig, part, surface);
        return;
    }

    step_mechanic(p);
    // Aboard, the Champion reads the standing row of its grid, so the chain is
    // live on the creature's back and cancels the same way it does on the
    // floor. There is no takeoff up here: jumping is how you *leave*, and a
    // move that spent the jump on an attack would take that away.
    chain_cancel(p, input);
    queue_the_shadow(p, input);
    let want_guard = input.has(Input::RIGHT) && p.shield().is_some_and(|sh| sh.in_hand());

    // Resolved instead of the countdown, exactly as on the ground -- see
    // `step_player`.
    if let Some((kind, held)) = p.action.channelling() {
        p.action = step_channel(p, who, kind, held, input, scene);
    } else {
        p.action = match countdown(p, want_guard) {
            Some(next) => next,
            None => {
                if input.has(Input::SPECIAL)
                    && p.class != Class::Champion
                    && p.can_throw(SLOT_SPECIAL, out)
                {
                    begin_move(p, who, SLOT_SPECIAL, input, scene, false)
                } else if pressed_mechanic {
                    match keyed_move(p).filter(|slot| p.can_throw(*slot, out)) {
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
                else if let Some(kind) = clicked_move(p, input).filter(|k| p.can_throw(*k, out)) {
                    begin_champion(p, kind);
                    begin_move(p, who, kind, input, scene, false)
                } else if want_guard {
                    Action::Guard { held: 0 }
                } else {
                    Action::Free
                }
            }
        };
    }

    // Jumping is how you leave, and it carries the surface's own velocity with
    // you -- which is what makes stepping off the back of a charging animal a
    // real option rather than a mistake.
    if input.has(Input::SPACE) && p.action.actionable() {
        let mob = p.class.mobility();
        p.mount = monster::NO_PART;
        // **What the leap carries is capped.** You take the surface's own
        // velocity with you -- that is what makes stepping off a charging
        // animal work -- but only as much of it as you had time to push off
        // against. Without the cap, jumping during a shake means being flung by
        // a back that is whipping sideways at forty metres a second, and the
        // one answer to the buck that is neither bracing nor leaving would not
        // exist. See `tuning::leap_carry`.
        let carried = carry_off(surface);
        p.vel = carried.add(V3::new(Fx::ZERO, t::jump_speed().mul(mob.jump), Fx::ZERO));
        p.grounded = false;
        p.air_dodged = false;
        p.jump_hold = t::jump_hold_frames();
        return;
    }

    // Movement, in the frame of the part underfoot. The wish direction arrives
    // camera-relative as it always does; `carry_yaw` has already turned the
    // camera with the animal, so "forward" still means the same part of its
    // back that it did before it turned.
    let (ax, az) = input.move_axis();
    let speed = rider_speed(p);
    let wish = rig.dir_to_part(part, move_dir(input.aim_turns(), ax, az));
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

    // A step you can walk up. The creature is terrain rather than a set of
    // ledges, and the climb from the tail to the nape crosses five parts: a
    // rider who had to jump every seam would never make it past the first one.
    //
    // Asked as a band that reaches **upward only**: from the feet to a full
    // step above them. Letting it reach downward as well made a rider standing
    // where two treads overlap step onto the lower one, then onto the higher
    // one, then back, once a frame -- and every swap moves them a few
    // centimetres, which the buck reads as an enormous acceleration and throws
    // them for. Descending is the ordinary landing test's job, a few lines
    // below. See `Rig::surface_within`.
    let stepped = beast
        .rig()
        .surface_within(beast.world_of(part, next), radius, Fx::ZERO, t::step_up())
        .filter(|(up, _)| *up != part);
    if let Some((up, top)) = stepped {
        let mut rest = beast.rest_frame(up, beast.world_of(part, next));
        rest.y = top;
        p.mount = up as u8;
        p.local = rest;
        p.pos = beast.world_of(up, rest);
        p.grip_settle = t::mount_settle() as u8;
        return;
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
            // wide -- capped the same way a jump is, and for the same reason.
            p.mount = monster::NO_PART;
            p.pos = moved.pos;
            p.vel = carry_off(surface);
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

/// How much of the surface's momentum you take with you when you leave it.
///
/// See `tuning::leap_carry` for why this is capped at all. The direction is
/// kept and only the magnitude is clipped, so a leap off a charging animal
/// still goes the way the animal was going.
fn carry_off(surface: V3) -> V3 {
    let speed = crate::math::big_len(surface);
    let cap = t::leap_carry();
    if speed.raw() <= cap.raw() || speed.raw() <= 0 {
        return surface;
    }
    surface.scale(cap.div(speed))
}

/// Lose your footing.
///
/// You leave with the velocity the surface had, capped, plus a push along the
/// creature's own up axis. The cap is what keeps a shake from firing someone
/// over the arena wall.
fn thrown_off(p: &mut Player, rig: &monster::Rig, part: usize, surface: V3) {
    let flat = V3::new(surface.x, Fx::ZERO, surface.z);
    let speed = flat.flat_len().min(t::throw_kick());
    // Up off the patch of animal you were standing on, not up off the animal.
    // On a skeleton those differ, and being thrown off a tail that is pointing
    // at the floor should not fire you into the ceiling.
    let up = rig.of(part).rot.apply(V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO));
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
    // The fall. See `tuning::throw_damage`: the back is safe from everything
    // the creature swings, so the buck has to be what the ride costs.
    p.wound(t::throw_damage());
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

    /// Fire Cataclysm, if this fighter has it out.
    ///
    /// Structurally the same trick the auto is: an instant line, resolved on
    /// the spot rather than by the hitbox loop, because what it does depends
    /// on what it meets first and none of the three answers is a bubble that
    /// lives for a few frames in front of her body. What it *does* on each
    /// answer is her heavy's own, not the auto's -- a fighter takes a real hit
    /// instead of a poke, a structure is destroyed rather than kicked, and a
    /// fire pillar is torn loose into a travelling tornado rather than merely
    /// charging the shot. See `docs/design/kits/elementalist.md`.
    fn fire_the_cataclysm(&mut self, i: usize) {
        let shooter = self.players[i];
        let Action::Active { kind, .. } = shooter.action else {
            return;
        };
        if !throws_a_cataclysm(&shooter, kind) || shooter.health <= 0 {
            return;
        }

        let beam = beam_of(&shooter);
        let m = moves::get(shooter.class, kind);
        let field = stones::gather(&self.players);
        let seen = self.players;
        let effects = self.effects;
        let beast = self.monster;
        let versus = beast.is_none();
        let scene = Scene {
            stones: &field,
            players: &seen,
            effects: &effects,
            quarry: beast.as_ref(),
        };
        let met = aim::first_along(beam, m.radius, i as u8, &scene, bolt::targets(versus));

        self.players[i].beam_reach = met.map_or(beam.length(), |c| c.dist());
        if shooter.hit_used {
            return;
        }

        match met {
            // A real hit, not the auto's no-stagger poke: this is the class's
            // heaviest single swing, and it costs a long wind-up to throw.
            Some(Contact::Fighter { index, .. }) => {
                let victim = self.players[index];
                let (guarding, parried) = guard_against(&victim, shooter.pos, m.unblockable);
                apply_hit(
                    &mut self.players[index],
                    Hit {
                        damage: m.damage,
                        hitstun: m.hitstun,
                        blockstun: m.blockstun,
                        knockback: m.knockback,
                        launch: Fx::ZERO,
                        grabs: 0,
                        by: i as u8,
                        dir: beam.dir(),
                        blocked: guarding,
                        parried,
                        interrupts: true,
                    },
                );
                if parried {
                    self.players[i].action = Action::Stagger {
                        left: t::parry_stagger(),
                    };
                    self.players[i].stun_total = t::parry_stagger();
                    self.players[index].parried = PARRY_FLOURISH;
                }
                self.players[i].hit_used = true;
            }
            // Broken outright, and thrown outward as debris rather than
            // detonated on the spot -- see `crate::debris`.
            Some(Contact::Stone { index, .. }) => {
                if let Some(at) = stones::destroy(&mut self.players, index) {
                    debris::blast(&mut self.debris, i as u8, at, beam.dir());
                }
                self.players[i].hit_used = true;
            }
            // Not charged -- torn loose. The same `Effect`, the same two
            // volumes, now moving. `age` and `life` are untouched: a pillar
            // that had barely started keeps growing on the same clock it was
            // already on, now while travelling, and one that was already
            // fully grown stays that way rather than snapping back down to
            // nothing and regrowing -- either would be a visible glitch at
            // the exact moment nothing about its *size* actually changed.
            // What is left of the original pillar's life is what the tornado
            // gets to travel on; the arena's own edge is the other way it can
            // run out. `banked` takes a snapshot of `age` right here, so its
            // flight can be measured from this moment rather than from
            // whenever the pillar was first planted -- see
            // `Effect::tornado_pos`. See `crate::effects::EffectKind::FireTornado`.
            Some(Contact::Fire { dist }) => {
                let at = beam.at(dist);
                if let Some(slot) = fire_pillar_slot_at(&self.effects, at) {
                    if let Some(effect) = self.effects[slot].as_mut() {
                        effect.kind = EffectKind::FireTornado;
                        // Flattened, not the raw beam: Cataclysm is only
                        // level when it was aimed at the ground (see
                        // `aim::skillshot_path`) -- anywhere else it meets a
                        // wall, a body height, or the range sphere at
                        // whatever pitch the camera happened to be at, and a
                        // tornado is a ground hazard the same way the pillar
                        // it came from was. A tilted `dir` walks `tornado_pos`
                        // straight through the floor (or into the sky) a
                        // couple of frames after it starts moving, and
                        // `arena::inside` reads that as having wandered off
                        // the map -- the tornado never gets to be seen at
                        // all, it just vanishes where the pillar stood.
                        effect.dir = V3::new(beam.dir().x, Fx::ZERO, beam.dir().z).normalized();
                        effect.struck = 0;
                        effect.banked = effect.age as i32;
                    }
                }
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

/// Is this the Elementalist's heavy?
///
/// The same shape of question `bolt::throws_a_beam` asks, and for the same
/// reason: being a skillshot decides how the move is aimed, not what it does
/// when it lands, so the slot has to be checked too. See `moves::SLOT_HEAVY`.
fn throws_a_cataclysm(p: &Player, kind: u8) -> bool {
    p.class == Class::Elementalist
        && kind == SLOT_HEAVY
        && moves::get(p.class, kind).aim() == aim::Kind::Skillshot
}

/// Which fire pillar a point sits inside, if any.
///
/// `aim::Contact::Fire` says a shot met fire and how far along its path, not
/// which of the (at most two, one per fighter) pillars on the field it was --
/// that bookkeeping already happens once inside `aim::first_along`, and
/// redoing the search here rather than threading an index back out of it is
/// the smaller change against a module that fails the build if anything but
/// `crate::aim` reaches for its own intersection arithmetic.
///
/// Nearest by flat distance to its own centre, rather than strict containment:
/// the contact point is where the shot entered the pillar's radius *plus its
/// own girth*, which sits just outside the pillar's true footprint, so a
/// contains-test tuned to the bare radius misses the very point that found it.
fn fire_pillar_slot_at(effects: &[Option<Effect>; MAX_EFFECTS], at: V3) -> Option<usize> {
    effects
        .iter()
        .enumerate()
        .filter_map(|(i, slot)| {
            let e = (*slot)?;
            (e.kind == EffectKind::FirePillar).then_some((i, e))
        })
        .min_by_key(|(_, e)| {
            V3::new(at.x.sub(e.pos.x), Fx::ZERO, at.z.sub(e.pos.z))
                .flat_len()
                .raw()
        })
        .map(|(i, _)| i)
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
                .map(|(a, _, _, _)| a)
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
                        interrupts: true,
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
            let m = moves::get(attacker.class, kind);
            let raw = Fx::from_int(m.damage)
                .mul(preying(attacker.class, beast.disabled()))
                .to_int();
            let dealt = beast.take_hit(part, raw);
            // Her double duty, against the creature: drink over the pool the
            // blade passes through, then spill under the part it struck.
            if dealt > 0 {
                self.drink_over(i, &m, box_out.from, box_out.to, None);
                self.spill(
                    i as u8,
                    attacker.class,
                    kind,
                    floor_under(box_out.centre()),
                    dealt,
                );
            }
            // An uppercut does not put thirteen metres of animal in the air and
            // a grab does not drag it anywhere, but throwing one at a creature
            // that is already reeling should still be a decision -- so the move
            // offers what it would have done to a fighter and the creature
            // takes what it can. Nothing at all, unless it is susceptible.
            beast.take_control(monster::Control {
                launch: m.launch,
                grabs: m.grabs,
                ..monster::Control::default()
            });
            self.players[i].heal(m.leeched(dealt));
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
